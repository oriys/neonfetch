use crate::animation::styles::hsv_to_rgb;

// Lava effect: layered moving pseudo-noise (sum of sines) mapped to warm palette.
// Avoids external noise libraries; fast enough for per-cell each frame.
pub fn calculate_lava_color_at(time: f32, row: usize, col: usize, term_w: usize, term_h: usize, speed: f32) -> (u8,u8,u8) {
    if term_w == 0 || term_h == 0 { return (0,0,0); }
    let t = time * 0.35 * speed.max(0.05);
    let x = col as f32 / term_w as f32;
    let y = row as f32 / term_h as f32;
    // Distort coordinates with simple domain warp
    let warp1 = ( (x*6.3 + t*1.9).sin() + (y*7.1 - t*1.3).sin() ) * 0.15;
    let warp2 = ( (x*4.2 - t*0.8).sin() + (y*5.7 + t*1.6).sin() ) * 0.12;
    let nx = x + warp1;
    let ny = y + warp2;
    // Layered value (like fractal brownian motion using sines)
    let mut v = 0.0;
    let mut amp = 0.6;
    let mut freq = 3.5;
    for _ in 0..4 {
        v += ((nx*freq + t*1.2).sin() * (ny*freq - t*0.9).cos()) * amp;
        amp *= 0.55;
        freq *= 1.9;
    }
    // Normalize v roughly to 0..1
    let v_norm = (v * 0.5 + 0.5).clamp(0.0, 1.0);
    // Temperature-like curve: highlight hotter pockets
    let heat = v_norm.powf(1.4);
    // Hue from deep red (10) through orange (35) to yellow (50)
    let hue = 10.0 + heat * 40.0;
    // Saturation high but reduce for very hot to simulate whitening
    let sat = 0.85 - heat * 0.25;
    // Value with additional slow global flicker
    let flicker = (t*3.5 + x*10.0).sin() * 0.04 + 0.96;
    let val = (0.25 + heat * 0.9) * flicker;
    hsv_to_rgb(hue % 360.0, sat.clamp(0.4, 1.0), val.clamp(0.05, 1.0))
}
