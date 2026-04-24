//! PDF attachment sanitizer (§13 Security)
//!
//! Detects and neutralises embedded JavaScript in PDF documents:
//! - `/JS` entries in action dictionaries (direct string or indirect reference)
//! - `/S /JavaScript` action types
//! - `/AA` (Additional Actions) dictionaries
//! - `/OpenAction` pointing to JavaScript actions
//! - `/Names` trees containing JavaScript name nodes
//!
//! The sanitizer performs byte-level replacement, padding substitutions to the same
//! length so the PDF cross-reference table remains valid.

use anyhow::Result;

// ── Public types ──────────────────────────────────────────────────────────────

/// Record of what was found/removed during PDF sanitization.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PdfSanitizeRecord {
    /// Whether the document was modified.
    pub was_modified: bool,
    /// Descriptions of threats that were neutralised.
    pub removed_threats: Vec<String>,
}

/// Output of the PDF sanitizer.
#[derive(Debug)]
pub struct PdfSanitizeResult {
    /// The sanitized PDF bytes.
    pub safe_bytes: Vec<u8>,
    /// What was removed (if anything).
    pub record: PdfSanitizeRecord,
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Sanitize a PDF document by neutralising embedded JavaScript.
///
/// Scans for JavaScript-related constructs and replaces them with null/safe
/// equivalents, preserving byte offsets so the cross-reference table stays valid.
pub fn sanitize(input: &[u8]) -> Result<PdfSanitizeResult> {
    // Quick check: must look like a PDF
    if !input.starts_with(b"%PDF-") {
        anyhow::bail!("not a valid PDF: missing %PDF- header");
    }

    let mut out = input.to_vec();
    let mut record = PdfSanitizeRecord::default();

    // Order matters: replace longer patterns first to avoid partial overlaps.
    neutralise_javascript_actions(&mut out, &mut record);
    neutralise_aa_dictionaries(&mut out, &mut record);
    neutralise_js_entries(&mut out, &mut record);
    neutralise_open_action_js(&mut out, &mut record);
    neutralise_names_js(&mut out, &mut record);

    Ok(PdfSanitizeResult {
        safe_bytes: out,
        record,
    })
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Replace `/S /JavaScript` action types with `/S /None` (padded).
fn neutralise_javascript_actions(data: &mut Vec<u8>, record: &mut PdfSanitizeRecord) {
    replace_all(
        data,
        b"/S /JavaScript",     // 14 bytes
        b"/S /None      ",     // 14 bytes: "/S /None" (8) + 6 spaces
        record,
        "JavaScript action type (/S /JavaScript)",
    );
    replace_all(
        data,
        b"/S/JavaScript",      // 13 bytes
        b"/S /None     ",      // 13 bytes: "/S /None" (8) + 5 spaces
        record,
        "JavaScript action type (/S/JavaScript)",
    );
}

/// Replace `/AA << ... >>` dictionaries with `/AA null`.
///
/// Additional Actions can contain JavaScript triggers for various PDF events
/// (page open, button press, etc.). We neutralise the entire AA dictionary.
fn neutralise_aa_dictionaries(data: &mut Vec<u8>, record: &mut PdfSanitizeRecord) {
    // Scan for /AA << patterns and replace the dictionary with null.
    // We replace `/AA` followed by dictionary `<<...>>` with `/AA null` padded
    // to match the original length. This is tricky because dictionaries can
    // be nested. We use a conservative approach: replace `/AA <<` with padding.
    let pattern = b"/AA <<";
    let mut offset = 0;
    while let Some(pos) = find_bytes(&data[offset..], pattern) {
        let abs_pos = offset + pos;
        // Find the matching `>>` by tracking nesting depth.
        if let Some(end) = find_matching_end(&data[abs_pos..], b"<<", b">>") {
            let dict_len = end + 2; // include the >>
            let replacement = format!("/AA null{}", " ".repeat(dict_len - "/AA null".len()));
            if replacement.len() == dict_len {
                data[abs_pos..abs_pos + dict_len].copy_from_slice(replacement.as_bytes());
            } else {
                // Length mismatch: replace just the `/AA` key with a no-op
                let pad = dict_len.saturating_sub(4);
                let patch = format!("    {}", " ".repeat(pad));
                let copy_len = dict_len.min(data.len() - abs_pos);
                data[abs_pos..abs_pos + copy_len]
                    .copy_from_slice(&patch.as_bytes()[..copy_len]);
            }
            record.was_modified = true;
            push_unique(
                &mut record.removed_threats,
                "Additional Actions dictionary (/AA)".to_string(),
            );
            offset = abs_pos + dict_len;
        } else {
            offset = abs_pos + pattern.len();
        }
    }

    // Also handle indirect reference form: /AA 123 0 R
    replace_all_regex(data, b"/AA ", record, "Additional Actions reference (/AA ref)");
}

/// Replace `/JS` entries with `/JS null` (padded).
///
/// `/JS` can be followed by:
/// - A string literal: `/JS (code...)`
/// - An indirect reference: `/JS 123 0 R`
fn neutralise_js_entries(data: &mut Vec<u8>, record: &mut PdfSanitizeRecord) {
    // Handle `/JS (` — string literal containing JS code
    let mut offset = 0;
    while let Some(pos) = find_bytes(&data[offset..], b"/JS (") {
        let abs_pos = offset + pos;
        // Find the closing `)` handling escaped `\)` within the string.
        // Skip past the opening `(` (5 bytes into the pattern) so find_closing_paren
        // scans the content, not the opening delimiter.
        if let Some(end) = find_closing_paren(&data[abs_pos + 5..]) {
            let entry_len = 5 + end + 1; // "/JS (" + content + ")"
            neutralise_entry(data, abs_pos, entry_len, record, "/JS string");
            offset = abs_pos + entry_len;
        } else {
            offset = abs_pos + 5;
        }
    }

    // Handle `/JS <` — hex string containing JS code
    offset = 0;
    while let Some(pos) = find_bytes(&data[offset..], b"/JS <") {
        let abs_pos = offset + pos;
        if let Some(end) = find_closing_angle(&data[abs_pos + 4..]) {
            let entry_len = 4 + end + 1;
            neutralise_entry(data, abs_pos, entry_len, record, "/JS hex string");
            offset = abs_pos + entry_len;
        } else {
            offset = abs_pos + 5;
        }
    }

    // Handle `/JS N M R` — indirect reference
    offset = 0;
    while let Some(pos) = find_bytes(&data[offset..], b"/JS ") {
        let abs_pos = offset + pos;
        // Check if followed by digits (indirect reference)
        if abs_pos + 4 < data.len() && data[abs_pos + 4].is_ascii_digit() {
            if let Some(end) = find_end_of_indirect_ref(&data[abs_pos + 4..]) {
                let entry_len = 4 + end;
                neutralise_entry(data, abs_pos, entry_len, record, "/JS reference");
                offset = abs_pos + entry_len;
            } else {
                offset = abs_pos + 4;
            }
        } else {
            offset = abs_pos + 4;
        }
    }
}

/// Replace `/OpenAction` pointing to JS with `/OpenAction null`.
fn neutralise_open_action_js(data: &mut Vec<u8>, record: &mut PdfSanitizeRecord) {
    // /OpenAction can be:
    // - An indirect reference: /OpenAction 123 0 R
    // - A dictionary: /OpenAction << ... /S /JavaScript ... >>
    // We already handle /S /JavaScript above, so handle indirect refs here
    // only if the referenced object contains JS (detected by other passes).
    // For safety, replace /OpenAction << dict >> with null if it looks dangerous.
    let pattern = b"/OpenAction <<";
    let mut offset = 0;
    while let Some(pos) = find_bytes(&data[offset..], pattern) {
        let abs_pos = offset + pos;
        if let Some(end) = find_matching_end(&data[abs_pos + 11..], b"<<", b">>") {
            let dict_content = &data[abs_pos..abs_pos + 11 + end + 2];
            if contains_js_indicator(dict_content) {
                let total_len = 11 + end + 2;
                let replacement = format!("/OpenAction null{}", " ".repeat(total_len - "/OpenAction null".len()));
                if replacement.len() == total_len {
                    data[abs_pos..abs_pos + total_len]
                        .copy_from_slice(replacement.as_bytes());
                    record.was_modified = true;
                    push_unique(
                        &mut record.removed_threats,
                        "OpenAction with JavaScript".to_string(),
                    );
                }
                offset = abs_pos + total_len;
            } else {
                offset = abs_pos + pattern.len();
            }
        } else {
            offset = abs_pos + pattern.len();
        }
    }
}

/// Replace JavaScript name-tree nodes under `/Names`.
fn neutralise_names_js(data: &mut Vec<u8>, record: &mut PdfSanitizeRecord) {
    // Look for /Names with JavaScript subtree references
    // Pattern: /Names << /JavaScript ... >> or embedded JS name nodes
    let pattern = b"/JavaScript";
    let mut offset = 0;
    while let Some(pos) = find_bytes(&data[offset..], pattern) {
        let abs_pos = offset + pos;
        // Only replace if it appears as a name (not already handled as /S /JavaScript)
        if abs_pos >= 2 && &data[abs_pos - 2..abs_pos] == b"/S " {
            offset = abs_pos + pattern.len();
            continue; // Already handled by neutralise_javascript_actions
        }
        // Replace "JavaScript" with "None        " (same length)
        let replacement = b"None        ";
        let len = pattern.len().min(replacement.len());
        data[abs_pos..abs_pos + len].copy_from_slice(&replacement[..len]);
        record.was_modified = true;
        push_unique(
            &mut record.removed_threats,
            "JavaScript name-tree entry (/Names/JavaScript)".to_string(),
        );
        offset = abs_pos + pattern.len();
    }
}

// ── Byte-level utilities ──────────────────────────────────────────────────────

/// Find a byte sequence in a slice, returning the offset.
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

/// Replace all occurrences of `from` with `to` (must be same length).
fn replace_all(
    data: &mut Vec<u8>,
    from: &[u8],
    to: &[u8],
    record: &mut PdfSanitizeRecord,
    description: &str,
) {
    assert_eq!(from.len(), to.len(), "replacement must be same length");
    let mut offset = 0;
    while let Some(pos) = find_bytes(&data[offset..], from) {
        let abs_pos = offset + pos;
        data[abs_pos..abs_pos + from.len()].copy_from_slice(to);
        record.was_modified = true;
        push_unique(&mut record.removed_threats, description.to_string());
        offset = abs_pos + from.len();
    }
}

/// Replace `/AA N M R` patterns (indirect reference after /AA) by overwriting
/// the space and digits with spaces.
fn replace_all_regex(
    data: &mut Vec<u8>,
    prefix: &[u8],
    record: &mut PdfSanitizeRecord,
    description: &str,
) {
    let mut offset = 0;
    while let Some(pos) = find_bytes(&data[offset..], prefix) {
        let abs_pos = offset + pos;
        // Check if followed by a digit (indirect reference)
        let after = abs_pos + prefix.len();
        if after < data.len() && data[after].is_ascii_digit() {
            // Find the end of "N M R"
            if let Some(end_offset) = find_end_of_indirect_ref(&data[after..]) {
                let total_len = prefix.len() + end_offset;
                // Replace the whole thing with spaces
                for b in &mut data[abs_pos..abs_pos + total_len] {
                    *b = b' ';
                }
                record.was_modified = true;
                push_unique(&mut record.removed_threats, description.to_string());
                offset = abs_pos + total_len;
            } else {
                offset = abs_pos + prefix.len();
            }
        } else {
            offset = abs_pos + prefix.len();
        }
    }
}

/// Neutralise a PDF entry at `pos` with `len` bytes by replacing with spaces.
fn neutralise_entry(
    data: &mut Vec<u8>,
    pos: usize,
    len: usize,
    record: &mut PdfSanitizeRecord,
    description: &str,
) {
    // Replace the entire entry with spaces (null content)
    for b in &mut data[pos..pos + len] {
        *b = b' ';
    }
    record.was_modified = true;
    push_unique(&mut record.removed_threats, description.to_string());
}

/// Find the closing `)` for a PDF string literal, handling `\)` escapes.
fn find_closing_paren(data: &[u8]) -> Option<usize> {
    let mut depth = 1;
    let mut i = 0;
    while i < data.len() {
        match data[i] {
            b'\\' => {
                i += 2; // skip escaped char
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find the closing `>` for a PDF hex string `<...>`.
fn find_closing_angle(data: &[u8]) -> Option<usize> {
    for (i, &b) in data.iter().enumerate() {
        if b == b'>' {
            return Some(i);
        }
    }
    None
}

/// Find the end of an indirect reference like `123 0 R`.
fn find_end_of_indirect_ref(data: &[u8]) -> Option<usize> {
    // Expect: <digits> <digits> R
    let mut i = 0;
    // First number (object number)
    while i < data.len() && data[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    // Space
    if i >= data.len() || !data[i].is_ascii_whitespace() {
        return None;
    }
    while i < data.len() && data[i].is_ascii_whitespace() {
        i += 1;
    }
    // Second number (generation number)
    let gen_start = i;
    while i < data.len() && data[i].is_ascii_digit() {
        i += 1;
    }
    if i == gen_start {
        return None;
    }
    // Space
    if i >= data.len() || !data[i].is_ascii_whitespace() {
        return None;
    }
    while i < data.len() && data[i].is_ascii_whitespace() {
        i += 1;
    }
    // 'R'
    if i < data.len() && data[i] == b'R' {
        Some(i + 1)
    } else {
        None
    }
}

/// Find the matching closing delimiter for a nested structure.
fn find_matching_end(data: &[u8], open: &[u8], close: &[u8]) -> Option<usize> {
    let mut depth = 0;
    let mut i = 0;
    while i <= data.len().saturating_sub(close.len()) {
        if i <= data.len() - open.len() && &data[i..i + open.len()] == open {
            depth += 1;
            i += open.len();
        } else if i <= data.len() - close.len() && &data[i..i + close.len()] == close {
            if depth == 0 {
                return Some(i);
            }
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
            i += close.len();
        } else {
            i += 1;
        }
    }
    None
}

/// Check if a byte slice contains JavaScript indicators.
fn contains_js_indicator(data: &[u8]) -> bool {
    find_bytes(data, b"/JavaScript").is_some()
        || find_bytes(data, b"/JS").is_some()
}

/// Append to vec only if not already present.
fn push_unique(vec: &mut Vec<String>, s: String) {
    if !vec.contains(&s) {
        vec.push(s);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sanitize_bytes(input: &[u8]) -> (Vec<u8>, PdfSanitizeRecord) {
        let result = sanitize(input).expect("sanitize should not fail");
        (result.safe_bytes, result.record)
    }

    // ── Minimal PDF helpers ─────────────────────────────────────────────────

    fn make_minimal_pdf(body: &str) -> Vec<u8> {
        // Build a valid-ish PDF with a single page and the given extra objects
        format!(
            "%PDF-1.4\n{}\n%%EOF\n",
            body
        )
        .into_bytes()
    }

    // ── JavaScript action detection ──────────────────────────────────────────

    #[test]
    fn detects_javascript_action() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Action /S /JavaScript /JS (alert(1)) >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified, "should detect JS action");
        assert!(!String::from_utf8_lossy(&out).contains("/JavaScript"));
        assert!(!String::from_utf8_lossy(&out).contains("alert(1)"));
    }

    #[test]
    fn detects_javascript_action_no_space() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Action /S/JavaScript /JS (evil()) >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified);
        assert!(!String::from_utf8_lossy(&out).contains("/JavaScript"));
    }

    #[test]
    fn detects_js_inline_string() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /S /JavaScript /JS (app.alert('XSS')) >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified);
        assert!(
            !String::from_utf8_lossy(&out).contains("app.alert"),
            "JS code should be stripped"
        );
    }

    #[test]
    fn detects_js_indirect_reference() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /S /JavaScript /JS 5 0 R >>\nendobj\n\
             5 0 obj\n(app.launchURL('http://evil.com'))\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified);
        // The /S /JavaScript should be neutralised
        assert!(!String::from_utf8_lossy(&out).contains("/JavaScript"));
    }

    #[test]
    fn detects_js_hex_string() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /S /JavaScript /JS <6170702E616C657274283129> >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified);
    }

    #[test]
    fn detects_aa_dictionary() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Page /AA << /O << /S /JavaScript /JS (evil()) >> >> >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified);
    }

    #[test]
    fn detects_aa_indirect_ref() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Page /AA 7 0 R >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified, "should neutralise /AA indirect ref");
    }

    #[test]
    fn detects_open_action_with_js() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Catalog /OpenAction << /S /JavaScript /JS (start()) >> >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified);
        assert!(
            !String::from_utf8_lossy(&out).contains("/JavaScript"),
            "should neutralise JS in OpenAction"
        );
    }

    #[test]
    fn detects_names_js() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Names << /JavaScript 5 0 R >> >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified);
    }

    // ── Legit PDFs — should not be modified ──────────────────────────────────

    #[test]
    fn legit_minimal_pdf_unchanged() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
             2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(!rec.was_modified, "clean PDF should not be modified");
        assert_eq!(out, pdf);
    }

    #[test]
    fn legit_pdf_with_metadata() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
             2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
             3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
             4 0 obj\n<< /Title (My Document) /Author (John) /Creator (Test) >>\nendobj",
        );
        let (_, rec) = sanitize_bytes(&pdf);
        assert!(!rec.was_modified);
    }

    #[test]
    fn legit_open_action_goto() {
        // /OpenAction with /S /GoTo is legitimate navigation, not JS
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Catalog /OpenAction << /S /GoTo /D [2 0 R /Fit] >> >>\nendobj",
        );
        let (_, rec) = sanitize_bytes(&pdf);
        assert!(!rec.was_modified, "/OpenAction /GoTo should not be flagged");
    }

    // ── Multiple threats ─────────────────────────────────────────────────────

    #[test]
    fn neutralises_multiple_threats() {
        let pdf = make_minimal_pdf(
            "1 0 obj\n<< /Type /Catalog /OpenAction << /S /JavaScript /JS (run()) >> \
             /AA << /O << /S /JavaScript /JS (other()) >> >> >>\nendobj",
        );
        let (out, rec) = sanitize_bytes(&pdf);
        assert!(rec.was_modified);
        let s = String::from_utf8_lossy(&out);
        assert!(!s.contains("/JavaScript"), "output: {s}");
    }

    // ── Error handling ───────────────────────────────────────────────────────

    #[test]
    fn rejects_non_pdf() {
        let result = sanitize(b"not a pdf file content");
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("not a valid PDF"),
            "should reject non-PDF input"
        );
    }

    #[test]
    fn handles_empty_pdf_header_only() {
        let pdf = b"%PDF-1.4\n%%EOF\n";
        let (_, rec) = sanitize_bytes(pdf);
        assert!(!rec.was_modified);
    }

    // ── Byte-level utility tests ─────────────────────────────────────────────

    #[test]
    fn find_closing_paren_basic() {
        assert_eq!(find_closing_paren(b"hello)"), Some(5));
        assert_eq!(find_closing_paren(b"hel\\)lo)"), Some(7));
        assert_eq!(find_closing_paren(b"(nested)out)"), Some(11));
        assert_eq!(find_closing_paren(b"no closing"), None);
    }

    #[test]
    fn find_matching_end_basic() {
        assert_eq!(find_matching_end(b"<<inner>>", b"<<", b">>"), Some(7));
        assert_eq!(find_matching_end(b"<< <<deep>> >>", b"<<", b">>"), Some(12));
    }

    #[test]
    fn find_end_of_indirect_ref_basic() {
        assert_eq!(find_end_of_indirect_ref(b"5 0 R"), Some(5));
        assert_eq!(find_end_of_indirect_ref(b"123 1 R"), Some(7));
        assert_eq!(find_end_of_indirect_ref(b"abc"), None);
        assert_eq!(find_end_of_indirect_ref(b"5"), None);
    }
}
