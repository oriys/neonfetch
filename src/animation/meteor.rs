use super::styles::hsv_to_rgb;

// Meteor rain: bright streaks fall diagonally across the text, each with a
// fading tail, over a dim night-sky base so the layout stays readable.
// Stateless per cell: every meteor's path is derived from its index and the
// current cycle number, so no per-frame state is needed.

const METEOR_COUNT: usize = 7;

fn hash01(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    (x as f32) / (u32::MAX as f32)
}

pub fn calculate_meteor_color_at(
    time: f32,
    row: usize,
    col: usize,
    term_w: usize,
    term_h: usize,
) -> (u8, u8, u8) {
    if term_w == 0 || term_h == 0 {
        return (0, 0, 0);
    }
    let w = term_w as f32;
    let h = term_h as f32;

    let mut best_intensity = 0.0f32;
    let mut best_hue = 0.0f32;
    let mut is_head = false;

    for i in 0..METEOR_COUNT {
        let seed = (i as u32).wrapping_mul(0x9E3779B9).wrapping_add(1);
        let r1 = hash01(seed);
        let r2 = hash01(seed ^ 0x85EBCA6B);
        let r3 = hash01(seed ^ 0xC2B2AE35);

        let fall_speed = 12.0 + r1 * 10.0; // rows per animation second
        let slope = 0.8 + r2 * 0.9; // columns advanced per row
        let trail = 6.0 + r3 * 8.0;
        let cycle_len = h + trail + 4.0;
        let progress = time * fall_speed + r2 * cycle_len;
        let cycle = (progress / cycle_len) as u32;
        let head_row = progress % cycle_len;

        // New entry column every pass, spanning enough range that the whole
        // diagonal flight crosses the visible area.
        let x0 = hash01(seed ^ cycle.wrapping_mul(0x27220A95)) * (w + h * slope) - h * slope;

        let u = head_row - row as f32; // distance behind the head, along rows
        if !(0.0..trail).contains(&u) {
            continue;
        }
        let col_on_path = x0 + row as f32 * slope;
        let dc = col as f32 - col_on_path;
        if dc.abs() > 0.7 {
            continue;
        }
        let intensity = (1.0 - u / trail).powf(1.5);
        if intensity > best_intensity {
            best_intensity = intensity;
            best_hue = (20.0 + r1 * 240.0 + time * 10.0) % 360.0;
            is_head = u < 1.2;
        }
    }

    if best_intensity <= 0.02 {
        // Dim night-sky base with a few faint twinkling cells.
        let cell_hash =
            hash01((row as u32).wrapping_mul(0x9E3779B9) ^ (col as u32).wrapping_mul(0x85EBCA6B));
        let twinkle = if cell_hash > 0.97 {
            ((time * 2.0 + cell_hash * 40.0).sin() * 0.5 + 0.5) * 0.25
        } else {
            0.0
        };
        return hsv_to_rgb(225.0, 0.30, 0.16 + twinkle);
    }
    if is_head {
        // White-hot head.
        let v = 0.85 + best_intensity * 0.15;
        return hsv_to_rgb(best_hue, 0.15, v.min(1.0));
    }
    let sat = 0.65 + best_intensity * 0.25;
    let val = 0.25 + best_intensity * 0.75;
    hsv_to_rgb(best_hue, sat.min(0.95), val.min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meteors_actually_move() {
        // Sampling a grid over time must show large brightness variation
        // (the old implementation rendered constant gray).
        let lum = |(r, g, b): (u8, u8, u8)| r as u32 + g as u32 + b as u32;
        let mut min = u32::MAX;
        let mut max = 0;
        for t in 0..60 {
            for row in 0..24 {
                for col in 0..80 {
                    let l = lum(calculate_meteor_color_at(t as f32 * 0.1, row, col, 80, 24));
                    min = min.min(l);
                    max = max.max(l);
                }
            }
        }
        assert!(max > 500, "no bright meteor heads found (max={})", max);
        assert!(min < 200, "no dim background found (min={})", min);
    }
}
