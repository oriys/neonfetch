use std::io::{self, Write};

/// Reusable per-frame output buffer.
///
/// Collects one frame of ANSI output, then writes it to stdout in a single
/// syscall. Tracks the last emitted foreground color so runs of identically
/// colored cells produce only one escape sequence, and applies the global
/// `--mono` / `--no-color` transforms in one place.
pub struct FrameBuf {
    buf: String,
    cur: Option<(u8, u8, u8)>,
    mono: bool,
    no_color: bool,
}

impl FrameBuf {
    pub fn new(mono: bool, no_color: bool) -> Self {
        FrameBuf {
            buf: String::with_capacity(64 * 1024),
            cur: None,
            mono,
            no_color,
        }
    }

    /// Start a new frame, keeping the allocation from the previous one.
    pub fn begin(&mut self) {
        self.buf.clear();
        self.cur = None;
    }

    /// Move the cursor to the start of a 1-based terminal row.
    pub fn goto_line(&mut self, row1: usize) {
        self.buf.push_str("\x1b[");
        push_usize(&mut self.buf, row1);
        self.buf.push_str(";1H");
    }

    /// Emit a raw escape sequence taken from the source text. The sequence may
    /// change color state, so color-run tracking is invalidated.
    pub fn push_ansi(&mut self, seq: &str) {
        self.buf.push_str(seq);
        self.cur = None;
    }

    /// Emit one printable cell with the given foreground color.
    pub fn put(&mut self, ch: char, rgb: (u8, u8, u8)) {
        if ch == ' ' {
            // Foreground color is invisible on blanks; skip the escape.
            self.buf.push(' ');
            return;
        }
        if !self.no_color {
            let rgb = if self.mono { grayscale(rgb) } else { rgb };
            if self.cur != Some(rgb) {
                self.buf.push_str("\x1b[38;2;");
                push_u8(&mut self.buf, rgb.0);
                self.buf.push(';');
                push_u8(&mut self.buf, rgb.1);
                self.buf.push(';');
                push_u8(&mut self.buf, rgb.2);
                self.buf.push('m');
                self.cur = Some(rgb);
            }
        }
        self.buf.push(ch);
    }

    /// Reset attributes and erase to end of line, clearing stale cells from
    /// previous frames or from whatever was on screen before we started.
    pub fn end_line(&mut self) {
        self.buf.push_str("\x1b[0m\x1b[K");
        self.cur = None;
    }

    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        out.write_all(self.buf.as_bytes())?;
        out.flush()
    }
}

fn grayscale((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let y = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32)
        .round()
        .clamp(0.0, 255.0) as u8;
    (y, y, y)
}

fn push_u8(buf: &mut String, v: u8) {
    if v >= 100 {
        buf.push((b'0' + v / 100) as char);
    }
    if v >= 10 {
        buf.push((b'0' + (v / 10) % 10) as char);
    }
    buf.push((b'0' + v % 10) as char);
}

fn push_usize(buf: &mut String, mut v: usize) {
    let mut digits = [0u8; 20];
    let mut n = 0;
    loop {
        digits[n] = b'0' + (v % 10) as u8;
        v /= 10;
        n += 1;
        if v == 0 {
            break;
        }
    }
    while n > 0 {
        n -= 1;
        buf.push(digits[n] as char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contents(fb: &FrameBuf) -> &str {
        &fb.buf
    }

    #[test]
    fn color_runs_deduplicated() {
        let mut fb = FrameBuf::new(false, false);
        fb.begin();
        fb.put('a', (10, 20, 30));
        fb.put('b', (10, 20, 30));
        fb.put('c', (11, 20, 30));
        let s = contents(&fb);
        assert_eq!(s.matches("\x1b[38;2;").count(), 2);
        assert!(s.contains("\x1b[38;2;10;20;30mab"));
    }

    #[test]
    fn no_color_emits_plain_text() {
        let mut fb = FrameBuf::new(false, true);
        fb.begin();
        fb.put('x', (1, 2, 3));
        assert_eq!(contents(&fb), "x");
    }

    #[test]
    fn mono_collapses_to_gray() {
        let mut fb = FrameBuf::new(true, false);
        fb.begin();
        fb.put('x', (255, 0, 0));
        assert!(contents(&fb).starts_with("\x1b[38;2;76;76;76m"));
    }

    #[test]
    fn spaces_skip_color_escapes() {
        let mut fb = FrameBuf::new(false, false);
        fb.begin();
        fb.put(' ', (1, 2, 3));
        assert_eq!(contents(&fb), " ");
    }
}
