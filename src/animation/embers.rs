use super::styles::hsv_to_rgb;

// Embers effect: drifting upward glowing particles over a dim base.
// We'll keep logic stateless here; actual particle positions updated in main loop.
// This module only converts intensity + jitter into RGB.

pub fn ember_color(hue_base: f32, intensity: f32, jitter: f32) -> (u8,u8,u8) {
    // Map intensity (0..1) into temperature gradient.
    // We shift hue slightly with jitter for variation.
    let hue = (hue_base + jitter * 25.0 + (1.0 - intensity) * 10.0) % 360.0;
    let sat = 0.70 + intensity * 0.25; // hotter => more saturated
    let val = 0.15 + intensity * 0.85;
    hsv_to_rgb(hue, sat.min(1.0), val.min(1.0))
}

// Base text background glow for embers (cooler, darker)
pub fn calculate_embers_color_at(time: f32, _row: usize, _col: usize) -> (u8,u8,u8) {
    let hue = (time * 8.0) % 360.0; // slow drift
    let sat = 0.18;
    let val = 0.10 + (time * 0.5).sin()*0.02; // faint breathing
    hsv_to_rgb(hue, sat, val)
}
