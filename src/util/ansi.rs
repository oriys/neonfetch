use regex::Regex;
use std::sync::OnceLock;

static ANSI_RE: OnceLock<Regex> = OnceLock::new();

/// Parse text containing ANSI escape sequences.
/// Returns a vector of (ansi_sequence, char); ansi_sequence is empty for printable chars; char='\0' for raw ANSI parts.
pub fn parse_ansi_text(text: &str) -> Vec<(String, char)> {
    let re = ANSI_RE.get_or_init(|| {
        Regex::new(r"(\x1b(?:[ -/]+.|\[[\]PX^_][^\x07\x1b]*|\[[0-?]*.|.))|(.)").unwrap()
    });
    let mut out = Vec::new();
    for cap in re.captures_iter(text) {
        if let Some(seq) = cap.get(1) {
            out.push((seq.as_str().to_string(), '\0'));
        } else if let Some(c) = cap.get(2).and_then(|m| m.as_str().chars().next()) {
            out.push((String::new(), c));
        }
    }
    out
}
