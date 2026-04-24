//! SVG attachment sanitizer (§13 Security)
//!
//! Strips XSS vectors from SVG documents before storage:
//! - `<script>` elements and all descendants
//! - `<foreignObject>` elements and all descendants
//! - `on*` event-handler attributes (onclick, onload, onerror, …)
//! - `xlink:href` and `href` pointing to http(s) or javascript: URLs
//! - `<!DOCTYPE>` declarations (prevents XXE entity injection)
//!
//! The sanitizer is a streaming pass over the XML event stream produced by
//! `quick-xml`.  Clean SVGs are reproduced faithfully; attribute quoting is
//! normalised to double-quotes during serialisation.

use anyhow::Result;
use quick_xml::{
    events::{BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText, Event},
    Reader, Writer,
};
use std::io::Cursor;

// ── Public types ──────────────────────────────────────────────────────────────

/// Record of what was removed during sanitization.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SanitizeRecord {
    /// Whether the document was modified at all.
    pub was_modified: bool,
    /// Local names of elements that were removed (e.g. `"script"`, `"foreignObject"`).
    pub removed_elements: Vec<String>,
    /// Descriptions of attributes that were removed (e.g. `"onclick"`, `"href=http://evil.com"`).
    pub removed_attrs: Vec<String>,
}

/// Output of the SVG sanitizer.
pub struct SanitizeResult {
    /// The sanitized SVG bytes, safe for rendering.
    pub safe_bytes: Vec<u8>,
    /// What was removed (if anything).
    pub record: SanitizeRecord,
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Sanitize an SVG document.
///
/// Parses `input` as XML and rewrites it, stripping all dangerous constructs.
/// Returns an error if the input is not well-formed XML.
pub fn sanitize(input: &[u8]) -> Result<SanitizeResult> {
    let mut reader = Reader::from_reader(input);
    {
        let cfg = reader.config_mut();
        cfg.trim_text(false);
        // Be lenient about mismatched end tags (real-world SVGs sometimes have them)
        cfg.check_end_names = false;
    }

    let mut out = Writer::new(Cursor::new(Vec::new()));
    let mut record = SanitizeRecord::default();
    // skip_depth > 0 while inside a blocked element subtree
    let mut skip_depth: u32 = 0;
    let mut buf = Vec::new();

    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| anyhow::anyhow!("SVG parse error at byte {}: {}", reader.error_position(), e))?;

        match event {
            // ── Opening tags ─────────────────────────────────────────────────
            Event::Start(ref e) => {
                if skip_depth > 0 {
                    skip_depth += 1;
                    record.was_modified = true;
                } else {
                    let local = e.local_name().into_inner().to_ascii_lowercase();
                    if is_blocked_element(&local) {
                        push_unique(&mut record.removed_elements, local_name_str(e));
                        record.was_modified = true;
                        skip_depth = 1;
                    } else {
                        let (new_e, changed) = filter_attrs(e, &mut record)?;
                        if changed {
                            record.was_modified = true;
                        }
                        out.write_event(Event::Start(new_e))
                            .map_err(|e| anyhow::anyhow!("SVG write error: {}", e))?;
                    }
                }
            }

            // ── Self-closing tags ─────────────────────────────────────────────
            Event::Empty(ref e) => {
                if skip_depth > 0 {
                    record.was_modified = true;
                } else {
                    let local = e.local_name().into_inner().to_ascii_lowercase();
                    if is_blocked_element(&local) {
                        push_unique(&mut record.removed_elements, local_name_str(e));
                        record.was_modified = true;
                    } else {
                        let (new_e, changed) = filter_attrs(e, &mut record)?;
                        if changed {
                            record.was_modified = true;
                        }
                        out.write_event(Event::Empty(new_e))
                            .map_err(|e| anyhow::anyhow!("SVG write error: {}", e))?;
                    }
                }
            }

            // ── Closing tags ──────────────────────────────────────────────────
            Event::End(ref e) => {
                if skip_depth > 0 {
                    if skip_depth == 1 {
                        skip_depth = 0;
                    } else {
                        skip_depth -= 1;
                    }
                } else {
                    let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    out.write_event(Event::End(BytesEnd::new(name)))
                        .map_err(|e| anyhow::anyhow!("SVG write error: {}", e))?;
                }
            }

            // ── Text content ──────────────────────────────────────────────────
            Event::Text(ref e) => {
                if skip_depth == 0 {
                    let s = String::from_utf8_lossy(e.as_ref()).into_owned();
                    out.write_event(Event::Text(BytesText::from_escaped(s)))
                        .map_err(|e| anyhow::anyhow!("SVG write error: {}", e))?;
                } else {
                    record.was_modified = true;
                }
            }

            // ── CDATA sections ────────────────────────────────────────────────
            Event::CData(ref e) => {
                if skip_depth == 0 {
                    let s = String::from_utf8_lossy(e.as_ref()).into_owned();
                    out.write_event(Event::CData(BytesCData::new(s)))
                        .map_err(|e| anyhow::anyhow!("SVG write error: {}", e))?;
                } else {
                    record.was_modified = true;
                }
            }

            // ── Comments ──────────────────────────────────────────────────────
            Event::Comment(ref e) => {
                if skip_depth == 0 {
                    let s = String::from_utf8_lossy(e.as_ref()).into_owned();
                    out.write_event(Event::Comment(BytesText::from_escaped(s)))
                        .map_err(|e| anyhow::anyhow!("SVG write error: {}", e))?;
                }
            }

            // ── XML declaration ───────────────────────────────────────────────
            Event::Decl(ref e) => {
                // Reconstruct from parsed fields to preserve declaration safely.
                let ver = e
                    .version()
                    .map_err(|err| anyhow::anyhow!("SVG decl version: {}", err))?;
                let ver_str = String::from_utf8_lossy(&ver).into_owned();
                let enc_owned = e
                    .encoding()
                    .transpose()
                    .map_err(|err| anyhow::anyhow!("SVG decl encoding: {}", err))?
                    .map(|c| String::from_utf8_lossy(&c).into_owned());
                let stan_owned = e
                    .standalone()
                    .transpose()
                    .map_err(|err| anyhow::anyhow!("SVG decl standalone: {}", err))?
                    .map(|c| String::from_utf8_lossy(&c).into_owned());
                let new_decl =
                    BytesDecl::new(&ver_str, enc_owned.as_deref(), stan_owned.as_deref());
                out.write_event(Event::Decl(new_decl))
                    .map_err(|e| anyhow::anyhow!("SVG write error: {}", e))?;
            }

            // ── Processing instructions ───────────────────────────────────────
            Event::PI(ref e) => {
                if skip_depth == 0 {
                    let s = String::from_utf8_lossy(e.as_ref()).into_owned();
                    out.write_event(Event::PI(BytesPI::new(s)))
                        .map_err(|e| anyhow::anyhow!("SVG write error: {}", e))?;
                }
            }

            // ── DOCTYPE — always strip (prevents XXE) ─────────────────────────
            Event::DocType(_) => {
                record.was_modified = true;
            }

            Event::Eof => break,
        }

        buf.clear();
    }

    let safe_bytes = out.into_inner().into_inner();
    Ok(SanitizeResult {
        safe_bytes,
        record,
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Elements whose entire subtree is removed.
fn is_blocked_element(local_lower: &[u8]) -> bool {
    matches!(local_lower, b"script" | b"foreignobject")
}

/// Extract the local (non-namespaced) element name as a `String`.
fn local_name_str(e: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(e.local_name().into_inner()).into_owned()
}

/// Append `s` to `vec` only if it is not already present.
fn push_unique(vec: &mut Vec<String>, s: String) {
    if !vec.contains(&s) {
        vec.push(s);
    }
}

/// Return `true` when `key_bytes` is an event-handler attribute (`on*`).
fn is_event_handler(key_bytes: &[u8]) -> bool {
    let lower = key_bytes.to_ascii_lowercase();
    // Strip namespace prefix if present (e.g. `foo:onclick`)
    split_local(&lower).starts_with(b"on")
}

/// Return `true` when `key_bytes` names an href attribute (`href` or `xlink:href`).
fn is_href_attr(key_bytes: &[u8]) -> bool {
    let lower = key_bytes.to_ascii_lowercase();
    split_local(&lower) == b"href"
}

/// Return `true` when an href value points to a blocked external resource.
///
/// Blocked schemes: `http://`, `https://`, `javascript:`.
fn is_blocked_href_value(value: &[u8]) -> bool {
    let s = std::str::from_utf8(value).unwrap_or("").trim();
    let lower = s.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("javascript:")
}

/// Return the slice after the last `:` in `bytes`, or `bytes` itself if no `:`.
fn split_local(bytes: &[u8]) -> &[u8] {
    match bytes.iter().rposition(|&b| b == b':') {
        Some(pos) => &bytes[pos + 1..],
        None => bytes,
    }
}

/// Build a copy of `elem` with dangerous attributes removed.
///
/// Returns `(filtered_element, any_removed)`.
fn filter_attrs(
    elem: &BytesStart<'_>,
    record: &mut SanitizeRecord,
) -> Result<(BytesStart<'static>, bool)> {
    // BytesStart::new takes impl Into<Cow<'a, str>> in quick-xml 0.36
    let name = String::from_utf8_lossy(elem.name().as_ref()).into_owned();
    let mut new_elem = BytesStart::new(name);
    let mut removed = false;

    // Collect kept attributes as owned byte vecs to sidestep lifetime issues.
    let mut kept: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

    for attr_res in elem.attributes() {
        let attr =
            attr_res.map_err(|e| anyhow::anyhow!("SVG attribute error: {}", e))?;
        let key: &[u8] = attr.key.as_ref();
        let val: &[u8] = &attr.value;

        if is_event_handler(key) {
            record
                .removed_attrs
                .push(String::from_utf8_lossy(key).into_owned());
            removed = true;
            continue;
        }

        if is_href_attr(key) && is_blocked_href_value(val) {
            record.removed_attrs.push(format!(
                "{}={}",
                String::from_utf8_lossy(key),
                String::from_utf8_lossy(val)
            ));
            removed = true;
            continue;
        }

        kept.push((key.to_vec(), val.to_vec()));
    }

    for (k, v) in &kept {
        new_elem.push_attribute((k.as_slice(), v.as_slice()));
    }

    Ok((new_elem, removed))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper ──────────────────────────────────────────────────────────────

    fn sanitize_str(input: &str) -> (String, SanitizeRecord) {
        let result = sanitize(input.as_bytes()).expect("sanitize should not fail");
        (
            String::from_utf8(result.safe_bytes).unwrap(),
            result.record,
        )
    }

    // ── XSS corpus — all should be modified ─────────────────────────────────

    #[test]
    fn strips_script_element() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script><rect/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(rec.removed_elements.contains(&"script".to_string()));
        assert!(!out.contains("script"), "output: {out}");
        assert!(!out.contains("alert"), "output: {out}");
        assert!(out.contains("<rect/>") || out.contains("<rect />"), "output should still have rect: {out}");
    }

    #[test]
    fn strips_script_with_type_attribute() {
        let svg = r#"<svg><script type="text/javascript">evil()</script></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("script"));
        assert!(!out.contains("evil"));
    }

    #[test]
    fn strips_script_with_cdata() {
        let svg = r#"<svg><script><![CDATA[alert(1)]]></script></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("script"));
        assert!(!out.contains("alert"));
    }

    #[test]
    fn strips_onclick_attr() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect onclick="alert(1)" width="100" height="100"/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(rec.removed_attrs.iter().any(|a| a == "onclick"));
        assert!(!out.contains("onclick"));
        assert!(!out.contains("alert"));
        // non-dangerous attributes should survive
        assert!(out.contains("width="), "width attr should survive: {out}");
    }

    #[test]
    fn strips_onload_on_svg_root() {
        let svg = r#"<svg onload="alert(1)" xmlns="http://www.w3.org/2000/svg"/>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(rec.removed_attrs.iter().any(|a| a == "onload"));
        assert!(!out.contains("onload"));
    }

    #[test]
    fn strips_onerror_attr() {
        let svg = r#"<svg><image onerror="alert(1)" href="x"/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("onerror"));
    }

    #[test]
    fn strips_xlink_href_http() {
        let svg = r#"<svg xmlns:xlink="http://www.w3.org/1999/xlink"><use xlink:href="http://evil.com/icon.svg"/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(rec.removed_attrs.iter().any(|a| a.contains("href") && a.contains("http://evil.com")));
        assert!(!out.contains("http://evil.com"));
    }

    #[test]
    fn strips_href_https() {
        let svg = r#"<svg><a href="https://evil.com">click</a></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("https://evil.com"));
        // link text should still be present
        assert!(out.contains("click"));
    }

    #[test]
    fn strips_href_javascript() {
        let svg = r#"<svg><a href="javascript:alert(1)">click</a></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("javascript:"));
    }

    #[test]
    fn strips_href_javascript_uppercase() {
        let svg = r#"<svg><a href="JAVASCRIPT:alert(1)">click</a></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("JAVASCRIPT:"));
    }

    #[test]
    fn strips_foreignobject() {
        let svg = r#"<svg><foreignObject width="100" height="100"><div>XSS</div></foreignObject><rect/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(
            rec.removed_elements.iter().any(|e| e.eq_ignore_ascii_case("foreignObject")),
            "foreignObject should be in removed_elements: {:?}", rec.removed_elements
        );
        assert!(!out.contains("foreignObject"), "output: {out}");
        assert!(!out.contains("<div"), "output: {out}");
        assert!(!out.contains("XSS"), "output: {out}");
        assert!(out.contains("<rect/>") || out.contains("<rect />") || out.contains("rect"), "rect should survive: {out}");
    }

    #[test]
    fn strips_nested_script_inside_foreignobject() {
        let svg = r#"<svg><foreignObject><script>evil()</script></foreignObject></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("foreignObject"));
        assert!(!out.contains("script"));
    }

    #[test]
    fn strips_doctype() {
        let svg = "<!DOCTYPE foo [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><svg/>";
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("DOCTYPE"), "output: {out}");
        assert!(!out.contains("xxe"), "output: {out}");
    }

    #[test]
    fn strips_namespaced_event_attr() {
        let svg = r#"<svg><rect ns:onclick="evil()" width="10"/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("onclick"));
        assert!(out.contains("width"));
    }

    #[test]
    fn strips_onmouseover() {
        let svg = r#"<svg><circle onmouseover="steal()" r="50"/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("onmouseover"));
        assert!(out.contains("r=") || out.contains(r#"r=""#));
    }

    // ── Legit SVG corpus — none should be modified ───────────────────────────

    #[test]
    fn legit_simple_shapes() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect x="10" y="10" width="80" height="80" fill="blue"/><circle cx="50" cy="50" r="30"/></svg>"#;
        let (_, rec) = sanitize_str(svg);
        assert!(!rec.was_modified, "clean SVG should not be modified");
    }

    #[test]
    fn legit_fragment_href() {
        // Internal fragment references must be preserved
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><use xlink:href="#myIcon"/><symbol id="myIcon"><rect/></symbol></svg>"##;
        let (out, rec) = sanitize_str(svg);
        assert!(!rec.was_modified, "fragment href should not be stripped");
        assert!(out.contains("#myIcon"));
    }

    #[test]
    fn legit_relative_href() {
        let svg = r#"<svg><a href="/page">link</a></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(!rec.was_modified);
        assert!(out.contains("/page"));
    }

    #[test]
    fn legit_data_image_href() {
        let svg = r#"<svg><image href="data:image/png;base64,abc123"/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(!rec.was_modified);
        assert!(out.contains("data:image/png"));
    }

    #[test]
    fn legit_gradient_and_defs() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><defs><linearGradient id="grad1"><stop offset="0%"/><stop offset="100%"/></linearGradient></defs><rect fill="url(#grad1)" width="100" height="50"/></svg>"#;
        let (_, rec) = sanitize_str(svg);
        assert!(!rec.was_modified);
    }

    #[test]
    fn legit_class_style_id_attrs() {
        let svg = r#"<svg><rect id="r1" class="box" style="fill:red" data-foo="bar" width="10" height="10"/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(!rec.was_modified);
        assert!(out.contains("id=") || out.contains(r#"id=""#));
        assert!(out.contains("class=") || out.contains(r#"class=""#));
        assert!(out.contains("data-foo=") || out.contains(r#"data-foo=""#));
    }

    #[test]
    fn legit_xml_declaration() {
        let svg = r#"<?xml version="1.0" encoding="UTF-8"?><svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(!rec.was_modified);
        assert!(out.contains("xml") || out.contains("1.0"));
    }

    #[test]
    fn legit_svg_with_text_element() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="10" y="20">Hello World</text></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(!rec.was_modified);
        assert!(out.contains("Hello World"));
    }

    #[test]
    fn legit_comment_preserved() {
        let svg = r#"<svg><!-- a design comment --><rect/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(!rec.was_modified);
        assert!(out.contains("design comment"));
    }

    // ── Multiple threats in one document ────────────────────────────────────

    #[test]
    fn strips_multiple_threats() {
        let svg = r#"<svg onload="x()"><script>y()</script><foreignObject><div/></foreignObject><use xlink:href="http://evil.com"/><rect onclick="z()"/></svg>"#;
        let (out, rec) = sanitize_str(svg);
        assert!(rec.was_modified);
        assert!(!out.contains("onload"));
        assert!(!out.contains("script"));
        assert!(!out.contains("foreignObject"));
        assert!(!out.contains("http://evil.com"));
        assert!(!out.contains("onclick"));
        // Base SVG element must survive
        assert!(out.contains("svg") || out.contains("rect"));
    }

    // ── Error handling ───────────────────────────────────────────────────────

    #[test]
    fn handles_incomplete_input() {
        // quick-xml with check_end_names=false may or may not error on truncated input;
        // the important thing is it must not panic
        let _ = sanitize(b"<svg><unclosed");
    }
}
