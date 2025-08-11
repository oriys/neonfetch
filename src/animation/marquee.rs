use super::styles::hsv_to_rgb;

// Minimal marquee implementation: a moving bright band across columns with soft feather.
pub fn calculate_marquee_color_at(
    time: f32,
    _row: usize,
    col: usize,
    term_width: usize,
    speed: f32,
) -> (u8, u8, u8) {
    if term_width == 0 { return (180,180,180); }
    let w = term_width as f32;
    // band travel speed factor: scaled so default speed=1 gives moderate motion
    let travel = time * (4.0 * speed.clamp(0.05, 50.0));
    let band_center = travel % w; // 0..w
    // Config (could later be parameters)
    let band_half = 1.2_f32; // core half-width
    let feather = 2.5_f32;   // soft edge additional width
    let mut dx = (col as f32 - band_center).abs();
    // wrap-around shortest distance on ring
    if dx > w * 0.5 { dx = w - dx; }
    let intensity = if dx <= band_half {
        1.0
    } else if dx <= band_half + feather {
        let t = (dx - band_half) / feather; // 0..1
        (1.0 - t).powf(1.5)
    } else { 0.0 };
    // Base dim background hue drift
    let base_hue = (time * 10.0) % 360.0;
    let (br, bg, bb) = hsv_to_rgb(base_hue, 0.15, 0.18);
    if intensity <= 0.001 { return (br, bg, bb); }
    // Band hue shifts faster for visual interest
    let band_hue = (base_hue + time * 120.0) % 360.0;
    let (r2, g2, b2) = hsv_to_rgb(band_hue, 0.75, 0.95);
    // Mix using intensity as weight (non-linear already applied)
    let mix = intensity;
    let r = (br as f32 * (1.0 - mix) + r2 as f32 * mix).clamp(0.0,255.0) as u8;
    let g = (bg as f32 * (1.0 - mix) + g2 as f32 * mix).clamp(0.0,255.0) as u8;
    let b = (bb as f32 * (1.0 - mix) + b2 as f32 * mix).clamp(0.0,255.0) as u8;
    (r,g,b)
}
