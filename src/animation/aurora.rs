use crate::animation::styles::hsv_to_rgb;

// Aurora effect: multiple vertical/diagonal translucent bands drifting with layered sine + pseudo-noise.
// Kept lightweight (no external noise lib).
pub fn calculate_aurora_color_at(
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
    let t = time * (0.25 * speed.max(0.05));
    let x = col as f32 / term_w as f32; // 0..1
    let y = row as f32 / term_h as f32; // 0..1
    // Band centers drifting (three ribbons)
    let c1 = ((t * 0.17).sin() * 0.5 + 0.5) * 1.1 - 0.05; // may exceed edges a little for smooth entrance
    let c2 = (t * 0.11 + 1.7).sin() * 0.5 + 0.5;
    let c3 = (t * 0.07 + 3.1).sin() * 0.5 + 0.5;
    // Gaussian-esque falloff
    let band_width = 0.22;
    let falloff = |center: f32, pos: f32| -> f32 {
        let d = (pos - center).abs();
        (-(d * d) / (2.0 * band_width * band_width)).exp()
    };
    let b1 = falloff(c1, x);
    let b2 = falloff(c2, x);
    let b3 = falloff(c3, x);
    // Undulation along vertical using column-based sine (simulate wave in curtains)
    let und1 = ((x * 8.0) + t * 3.0).sin() * 0.5 + 0.5;
    let und2 = ((x * 5.5) - t * 2.2).sin() * 0.5 + 0.5;
    let und3 = ((x * 6.5) + t * 1.7).sin() * 0.5 + 0.5;
    // Height gradient (fainter near bottom)
    let sky_grad = (y * 0.9 + 0.1).powf(1.4); // 0.1..1
    let mix1 = b1 * und1;
    let mix2 = b2 * und2;
    let mix3 = b3 * und3;
    let intensity = (mix1 + mix2 * 0.9 + mix3 * 1.1) * 0.9 * sky_grad;
    let mut i = (intensity * 1.25).min(1.2);
    i = (i * 0.85 + 0.15).clamp(0.0, 1.0);
    let base_hue = 170.0; // teal
    let hue = (base_hue + mix1 * 25.0 + mix2 * 60.0 + mix3 * -30.0 + (t * 15.0)) % 360.0;
    let sat = (0.35 + i * 0.55).min(0.82);
    let breath = (t * 0.8).sin() * 0.08 + 0.92; // 0.84..1.0
    let val = (i.powf(0.9) * breath).clamp(0.05, 1.0);
    hsv_to_rgb(hue, sat, val)
}
