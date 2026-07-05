use super::styles::hsv_to_rgb;

// Marquee: a bright band sweeps across the text from left to right on a dim
// base, like a theater-sign chase light. Slightly slanted per row so the band
// reads as a moving diagonal highlight.
pub fn calculate_marquee_color_at(
    time: f32,
    row: usize,
    col: usize,
    term_w: usize,
) -> (u8, u8, u8) {
    let w = term_w.max(1) as f32;
    let band_len = (w * 0.22).clamp(8.0, 28.0);
    let period = w + band_len * 2.0;
    let head = (time * 32.0) % period;
    let pos = (col as f32 + row as f32 * 1.5) % period;
    let mut d = head - pos;
    if d < 0.0 {
        d += period;
    }
    let intensity = if d < band_len {
        (1.0 - d / band_len).powf(1.4)
    } else {
        0.0
    };
    let hue = (time * 24.0 + col as f32 * 1.2) % 360.0;
    let sat = 0.5 + intensity * 0.4;
    let val = 0.22 + intensity * 0.78;
    hsv_to_rgb(hue, sat.min(0.95), val.min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_moves_over_time() {
        // The same cell must change brightness as the band passes.
        let samples: Vec<(u8, u8, u8)> = (0..40)
            .map(|i| calculate_marquee_color_at(i as f32 * 0.1, 0, 10, 80))
            .collect();
        let lum = |(r, g, b): (u8, u8, u8)| r as u32 + g as u32 + b as u32;
        let min = samples.iter().copied().map(lum).min().unwrap();
        let max = samples.iter().copied().map(lum).max().unwrap();
        assert!(max > min + 100, "marquee looks static: {} vs {}", min, max);
    }
}
