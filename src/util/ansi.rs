/// Parse text containing ANSI escape sequences.
/// Returns a vector of (ansi_sequence, char); ansi_sequence is empty for printable chars; char='\0' for raw ANSI parts.
pub fn parse_ansi_text(text: &str) -> Vec<(String, char)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == 0x1b {
            // Start of an escape sequence
            let start = i;
            i += 1;
            if i >= len {
                out.push((text[start..i].to_string(), '\0'));
                break;
            }
            match bytes[i] {
                // CSI: ESC [
                b'[' => {
                    i += 1;
                    // Check for ], P, X, ^, _ (OSC / DCS / SOS / PM / APC)
                    if i < len && matches!(bytes[i], b']' | b'P' | b'X' | b'^' | b'_') {
                        i += 1;
                        // Consume until BEL (0x07) or next ESC
                        while i < len && bytes[i] != 0x07 && bytes[i] != 0x1b {
                            i += 1;
                        }
                        if i < len && bytes[i] == 0x07 {
                            i += 1;
                        }
                    } else {
                        // Standard CSI: parameter bytes [0x30-0x3F]*, then final byte [0x40-0x7E]
                        while i < len && (0x30..=0x3F).contains(&bytes[i]) {
                            i += 1;
                        }
                        // Intermediate bytes [0x20-0x2F]*
                        while i < len && (0x20..=0x2F).contains(&bytes[i]) {
                            i += 1;
                        }
                        // Final byte [0x40-0x7E]
                        if i < len && (0x40..=0x7E).contains(&bytes[i]) {
                            i += 1;
                        }
                    }
                }
                // Intermediate bytes (Fe / Fs sequences): ESC <space>-/ then one char
                0x20..=0x2F => {
                    while i < len && (0x20..=0x2F).contains(&bytes[i]) {
                        i += 1;
                    }
                    if i < len {
                        i += 1; // final byte
                    }
                }
                // Any other single char after ESC
                _ => {
                    i += 1;
                }
            }
            out.push((text[start..i].to_string(), '\0'));
        } else {
            // Regular character — advance by one UTF-8 char
            let ch = text[i..].chars().next().unwrap();
            i += ch.len_utf8();
            out.push((String::new(), ch));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text() {
        let result = parse_ansi_text("hello");
        assert_eq!(result.len(), 5);
        for (seq, _) in &result {
            assert!(seq.is_empty());
        }
        let chars: String = result.iter().map(|(_, c)| *c).collect();
        assert_eq!(chars, "hello");
    }

    #[test]
    fn color_escape() {
        let result = parse_ansi_text("\x1b[31mA\x1b[0m");
        // ESC[31m, 'A', ESC[0m
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "\x1b[31m");
        assert_eq!(result[0].1, '\0');
        assert_eq!(result[1].0, "");
        assert_eq!(result[1].1, 'A');
        assert_eq!(result[2].0, "\x1b[0m");
        assert_eq!(result[2].1, '\0');
    }

    #[test]
    fn truecolor_escape() {
        let result = parse_ansi_text("\x1b[38;2;255;0;128mX");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "\x1b[38;2;255;0;128m");
        assert_eq!(result[0].1, '\0');
        assert_eq!(result[1].1, 'X');
    }

    #[test]
    fn empty_input() {
        let result = parse_ansi_text("");
        assert!(result.is_empty());
    }

    #[test]
    fn mixed_ansi_and_text() {
        let result = parse_ansi_text("A\x1b[1mB");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].1, 'A');
        assert_eq!(result[1].0, "\x1b[1m");
        assert_eq!(result[2].1, 'B');
    }
}
