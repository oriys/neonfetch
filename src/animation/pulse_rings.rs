use crate::animation::styles::hsv_to_rgb;

// Pulse Rings: expanding concentric brightness/color rings from a moving center.
// We treat the text area as a 2D field; center drifts slowly. Rings expand and fade.
pub fn calculate_pulse_rings_color_at(
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
    let t = time * speed.max(0.05);
    let w = term_w as f32;
    let h = term_h as f32;
    // Moving center (Lissajous-like path)
    let cx = (w * 0.5) + ((t * 0.35).sin() * w * 0.18);
    let cy = (h * 0.5) + ((t * 0.27 + 1.3).sin() * h * 0.22);
    let x = col as f32 + 0.5; // center of cell
    let y = row as f32 + 0.4; // slight vertical bias
    let dx = x - cx;
    let dy = y - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    // Ring parameters
    let ring_spacing = 6.5_f32; // distance between ring peaks
    let ring_speed = 24.0; // expansion speed in cells/sec at speed=1
    let base = dist - t * ring_speed;
    // Compute ring phase relative to spacing
    let phase = base / ring_spacing;
    // Use fractional part to form pulses, with fade over age
    let frac = phase - phase.floor();
    // Age of the current ring traveling through this point
    let age = phase.floor();
    // Envelope: sharp near center of ring (frac near 0), fade by gaussian around 0
    // shift frac so ring center at 0
    let d = (frac).min(1.0); // 0..1
    // Mirror around 0.5 for symmetrical ring thickness
    let centered = (d - 0.5).abs() * 2.0; // 0 at center, 1 at edge
    let thickness = 0.55; // controls thickness of bright area
    let ring_env = (-(centered * centered) / (thickness * thickness)).exp();
    // Dampen older inner rings (age increases as rings pass)
    let age_fade = (1.0 - (age * 0.06).max(0.0)).clamp(0.0, 1.0);
    let intensity = ring_env * age_fade;
    if intensity < 0.01 {
        return (0, 0, 0);
    }
    // Color: hue cycles with time + small radial modulation
    let hue = ((t * 50.0) + dist * 1.3) % 360.0;
    let sat = 0.55 + intensity * 0.35;
    // Global slow breathing overlay
    let breath = (t * 0.8).sin() * 0.08 + 0.92;
    let val = (0.15 + intensity.powf(0.9) * 0.95) * breath;
    hsv_to_rgb(hue, sat.min(0.95), val.min(1.0))
}
