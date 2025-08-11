use super::styles::hsv_to_rgb;

// Plasma effect: composite sine fields forming smooth color waves.
// Stateless per-cell color; relies on time, row, col.
// Formula: v = sin(a*x + t*ω1) + sin(b*y + t*ω2) + sin((x+y)*k + t*ω3)
// Map normalized v into hue/brightness.

pub fn calculate_plasma_color_at(
    time: f32,
    row: usize,
    col: usize,
    term_w: usize,
    term_h: usize,
    speed: f32,
) -> (u8, u8, u8) {
    if term_w == 0 || term_h == 0 {
        return (0, 0, 0);
    }
    let s = speed.clamp(0.05, 10.0);
    // Normalize coordinates to 0..1 for scale invariance
    let x = col as f32 / term_w as f32;
    let y = row as f32 / term_h as f32;
    // Frequency scalers (tweak for diversity)
    let f1 = 7.0; // x frequency
    let f2 = 5.0; // y frequency
    let f3 = 9.5; // diagonal frequency
    let t1 = time * 0.9 * s;
    let t2 = time * 0.55 * s;
    let t3 = time * 0.35 * s;
    let v = (f1 * x + t1).sin() + (f2 * y + t2).sin() + (f3 * (x + y) + t3).sin();
    // v in [-3,3]; normalize to 0..1
    let vn = (v + 3.0) / 6.0;
    // Hue sweeps plus a small local perturbation for more detail
    let hue = (vn * 300.0 + time * 25.0) % 360.0;
    // Saturation/Value shaping
    let sat = 0.55 + (vn - 0.5).abs() * 0.9; // higher saturation toward extremes
    let mut val = 0.30 + (vn.powf(0.8)) * 0.70;
    // Subtle global breathing
    val *= (time * 0.6).sin() * 0.05 + 0.97;
    let (r, g, b) = hsv_to_rgb(hue, sat.min(1.0), val.min(1.0));
    (r, g, b)
}
