//! ANSI escape sequence stripper.
//!
//! Strips terminal control sequences from text at the session-parser boundary.
//! This prevents ANSI codes from corrupting search, embeddings, and UI rendering.
//!
//! Supported sequences:
//! - CSI (Control Sequence Introducer): ESC [ ...
//! - OSC (Operating System Command): ESC ] ... BEL\ST
//! - SGR (Select Graphic Rendition): colors, styles, resets
//! - Cursor movement: CSI n A/B/C/D (cursor up/down/left/right)
//! - 256-color mode: ESC [ 38;5;n m and ESC [ 48;5;n m
//! - RGB color mode: ESC [ 38;2;r;g;b m
//! - Bracketed paste mode: ESC [ 200~ / ESC [ 201~
//! - Title sequences: ESC ] 0; ... BEL
//!
//! Plan reference: notes/orchestrator-problems-and-solutions.md §F4

/// Strip ANSI escape sequences from a string.
///
/// This function removes all terminal control sequences while preserving
/// the visible text content. It handles:
/// - CSI sequences (ESC [ ... followed by a final byte in @-~)
/// - OSC sequences (ESC ] ... terminated by BEL or ST)
/// - Simple ESC followed by a single character
///
/// # Examples
///
/// ```
/// use hoop_daemon::ansi_strip::strip_ansi;
///
/// assert_eq!(strip_ansi("\x1b[31mRed text\x1b[0m"), "Red text");
/// assert_eq!(strip_ansi("\x1b[2K"), "");  // Erase line command
/// assert_eq!(strip_ansi("Normal text"), "Normal text");
/// ```
pub fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let byte = bytes[i];

        // Look for ESC (0x1B)
        if byte == 0x1B {
            // Check for CSI sequence: ESC [
            if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                // Skip to the final byte (0x40-0x7E, i.e., @-~)
                let mut j = i + 2;
                while j < bytes.len() {
                    let b = bytes[j];
                    // CSI sequences end with a byte in range 0x40-0x7E
                    if (0x40..=0x7E).contains(&b) {
                        i = j + 1; // Skip the entire sequence
                        break;
                    }
                    // Safety: don't scan forever if we encounter malformed input
                    if j > i + 256 {
                        // Malformed - treat as literal text
                        result.push(byte as char);
                        i += 1;
                        break;
                    }
                    j += 1;
                }
                if j >= bytes.len() {
                    // Incomplete sequence at end of input, skip it
                    break;
                }
                continue;
            }

            // Check for OSC sequence: ESC ]
            if i + 1 < bytes.len() && bytes[i + 1] == b']' {
                // Skip to BEL (0x07) or ST (ESC \)
                let mut j = i + 2;
                while j < bytes.len() {
                    if bytes[j] == 0x07 {
                        // BEL terminator
                        i = j + 1;
                        break;
                    }
                    if bytes[j] == 0x1B && j + 1 < bytes.len() && bytes[j + 1] == b'\\' {
                        // ST terminator (ESC \)
                        i = j + 2;
                        break;
                    }
                    // Safety limit
                    if j > i + 1024 {
                        result.push(byte as char);
                        i += 1;
                        break;
                    }
                    j += 1;
                }
                if j >= bytes.len() {
                    // Incomplete sequence, skip it
                    break;
                }
                continue;
            }

            // Other ESC sequences: skip ESC and the next character
            if i + 1 < bytes.len() {
                i += 2;
                continue;
            }

            // ESC at end of input - skip it
            break;
        }

        // Not an escape sequence - copy the character
        // Handle UTF-8 properly
        if byte < 0x80 {
            // ASCII - single byte
            result.push(byte as char);
            i += 1;
        } else {
            // UTF-8 multi-byte sequence
            let code_point = utf8_len(byte);
            if i + code_point <= bytes.len() {
                // Valid UTF-8 - copy all bytes
                for b in &bytes[i..i + code_point] {
                    result.push(*b as char);
                }
                i += code_point;
            } else {
                // Invalid UTF-8 at end - just copy what we have
                result.push(byte as char);
                i += 1;
            }
        }
    }

    result
}

/// Get the expected byte length of a UTF-8 sequence from its first byte.
fn utf8_len(first_byte: u8) -> usize {
    if first_byte < 0x80 {
        1
    } else if first_byte < 0xE0 {
        2
    } else if first_byte < 0xF0 {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_basic_sgr() {
        // SGR reset
        assert_eq!(strip_ansi("\x1b[0m"), "");
        // Bold, red, green, blue
        assert_eq!(strip_ansi("\x1b[1;31;42;104m"), "");
        // Text with color
        assert_eq!(strip_ansi("\x1b[31mRed\x1b[0m"), "Red");
        assert_eq!(
            strip_ansi("Normal \x1b[1mBold\x1b[0m text"),
            "Normal Bold text"
        );
    }

    #[test]
    fn test_strip_cursor_movement() {
        // Cursor up/down/left/right
        assert_eq!(strip_ansi("\x1b[A"), "");
        assert_eq!(strip_ansi("\x1b[5B"), "");
        assert_eq!(strip_ansi("\x1b[10C"), "");
        assert_eq!(strip_ansi("\x1b[2D"), "");
        // Cursor position
        assert_eq!(strip_ansi("\x1b[12;34H"), "");
        // Erase commands
        assert_eq!(strip_ansi("\x1b[2J"), "");
        assert_eq!(strip_ansi("\x1b[K"), "");
        assert_eq!(strip_ansi("\x1b[0K"), "");
    }

    #[test]
    fn test_strip_256_color() {
        //Foreground 256-color
        assert_eq!(strip_ansi("\x1b[38;5;123m"), "");
        // Background 256-color
        assert_eq!(strip_ansi("\x1b[48;5;255m"), "");
        // Combined
        assert_eq!(strip_ansi("\x1b[38;5;42;48;5;123m"), "");
        // With text
        assert_eq!(strip_ansi("\x1b[38;5;196mError\x1b[0m"), "Error");
    }

    #[test]
    fn test_strip_rgb_color() {
        // RGB foreground
        assert_eq!(strip_ansi("\x1b[38;2;255;0;128m"), "");
        // RGB background
        assert_eq!(strip_ansi("\x1b[48;2;0;128;255m"), "");
        // Combined RGB
        assert_eq!(strip_ansi("\x1b[38;2;255;0;0;48;2;0;0;255m"), "");
        // With text
        assert_eq!(strip_ansi("\x1b[38;2;255;100;50mRGB\x1b[0m"), "RGB");
    }

    #[test]
    fn test_strip_bracketed_paste() {
        // Bracketed paste mode start/end
        assert_eq!(strip_ansi("\x1b[200~"), "");
        assert_eq!(strip_ansi("\x1b[201~"), "");
        // With content
        assert_eq!(strip_ansi("\x1b[200~pasted text\x1b[201~"), "pasted text");
    }

    #[test]
    fn test_strip_osc_sequences() {
        // Window title
        assert_eq!(strip_ansi("\x1b]0;Title\x07"), "");
        // Icon/title
        assert_eq!(strip_ansi("\x1b]1;Icon\x07"), "");
        // With ST terminator
        assert_eq!(strip_ansi("\x1b]0;Title\x1b\\"), "");
        // Color setting (OSC 4)
        assert_eq!(strip_ansi("\x1b]4;1;rgb:ff00/ff00/ff00\x07"), "");
    }

    #[test]
    fn test_strip_mixed_sequences() {
        let input = "\x1b[31m\x1b]0;Shell\x07Red\x1b[0m\x1b[2K";
        assert_eq!(strip_ansi(input), "Red");
    }

    #[test]
    fn test_preserve_normal_text() {
        assert_eq!(strip_ansi("Just normal text"), "Just normal text");
        assert_eq!(strip_ansi("Text with 🎉 emoji"), "Text with 🎉 emoji");
        assert_eq!(strip_ansi("Text\nwith\nnewlines"), "Text\nwith\nnewlines");
        assert_eq!(strip_ansi("Text\twith\ttabs"), "Text\twith\ttabs");
    }

    #[test]
    fn test_empty_and_edge_cases() {
        assert_eq!(strip_ansi(""), "");
        assert_eq!(strip_ansi("\x1b"), "");
        assert_eq!(strip_ansi("\x1b["), "");
        assert_eq!(strip_ansi("\x1b]"), "");
    }

    #[test]
    fn test_complex_sgr_sequences() {
        // Italic, underline, strikethrough
        assert_eq!(strip_ansi("\x1b[3;4;9m"), "");
        // Intensity (bold, faint, normal)
        assert_eq!(strip_ansi("\x1b[1m"), "");
        assert_eq!(strip_ansi("\x1b[2m"), "");
        assert_eq!(strip_ansi("\x1b[22m"), "");
        // All reset variants
        assert_eq!(strip_ansi("\x1b[m"), "");
        assert_eq!(strip_ansi("\x1b[0m"), "");
    }

    #[test]
    fn test_device_control_strings() {
        // DCS sequences (ESC P ... ST)
        assert_eq!(strip_ansi("\x1bP+q544e\x1b\\"), "");
        // Application Program Command (ESC _)
        assert_eq!(strip_ansi("\x1b_ignored\x1b\\"), "");
        // Privacy Message (ESC ^)
        assert_eq!(strip_ansi("\x1b^message\x1b\\"), "");
    }

    #[test]
    fn test_alternate_screen() {
        // DECSET/DECRST for alternate screen
        assert_eq!(strip_ansi("\x1b[?1049h"), "");
        assert_eq!(strip_ansi("\x1b[?1049l"), "");
        // Save/restore cursor
        assert_eq!(strip_ansi("\x1b7"), "");
        assert_eq!(strip_ansi("\x1b8"), "");
    }

    #[test]
    fn test_text_with_embedded_ansi() {
        let input = "Error: \x1b[31m\x1b[1mfile not found\x1b[0m in \x1b[4m/path/to/file\x1b[0m";
        assert_eq!(strip_ansi(input), "Error: file not found in /path/to/file");
    }

    #[test]
    fn test_only_removes_ansi_sequences() {
        // ESC followed by non-sequence character
        assert_eq!(strip_ansi("A\x1bB"), "AB");
        // Brackets that aren't part of CSI
        assert_eq!(strip_ansi("[not a sequence]"), "[not a sequence]");
    }

    #[test]
    fn test_handles_incomplete_sequences() {
        // Incomplete CSI at end
        assert_eq!(strip_ansi("text\x1b["), "text");
        // Incomplete OSC at end
        assert_eq!(strip_ansi("text\x1b]"), "text");
        // Incomplete 256-color
        assert_eq!(strip_ansi("text\x1b[38;5"), "text");
    }

    #[test]
    fn test_common_cli_patterns() {
        // Tool output patterns
        let input = "\x1b[2K\r\x1b[1mBuilding\x1b[0m project...\x1b[2K\rDone!";
        assert_eq!(strip_ansi(input), "Building project...Done!");

        // Progress bar pattern
        let input = "\x1b[0G\x1b[32m█\x1b[0m\x1b[0G\x1b[31;1m 50%\x1b[0m";
        assert_eq!(strip_ansi(input), "█ 50%");
    }
}
