use super::styles::hsv_to_rgb;

// Comet effect: moving bright heads with trailing gradient that fades & shifts hue.
// Stateless color function: head position derived from time & comet index.
// Parameters (hardcoded for now):
//   COMET_COUNT: number of comets
//   LENGTH: tail length in cells
// Each comet travels horizontally; rows distributed.

const COMET_COUNT: usize = 3;
const LENGTH: f32 = 18.0; // tail length

pub fn calculate_comet_color_at(
    time: f32,
    row: usize,
    col: usize,
    term_width: usize,
    term_height: usize,
    speed: f32,
) -> (u8,u8,u8) {
    if term_width == 0 { return (0,0,0); }
    let mut best_intensity = 0.0f32;
    let mut best_hue = 0.0f32;
    let travel_speed = 10.0 * speed.clamp(0.05, 20.0); // cells per second for heads
    for i in 0..COMET_COUNT {
        // Assign row band for each comet (spread across available lines)
        let row_band = if term_height > 0 { (i * term_height / COMET_COUNT).min(term_height.saturating_sub(1)) } else { 0 };
        // Only apply to matching row (or small vertical jitter?). We'll allow +/-1 row blending.
        let dr = (row as isize - row_band as isize).abs() as f32;
        if dr > 1.5 { continue; }
        let row_factor = if dr <= 0.5 { 1.0 } else { 0.5_f32.powf((dr-0.5)*2.0) }; // diminish for adjacent rows
        let head_pos = (time * travel_speed + i as f32 * (term_width as f32 / COMET_COUNT as f32 * 0.6)) % (term_width as f32 + LENGTH);
        // head moves from left to right; tail behind (col < head_pos)
        let dx = head_pos - col as f32;
        if dx < 0.0 || dx > LENGTH { continue; }
        let core = (1.0 - dx / LENGTH).powf(1.3); // intensity falloff
        let intensity = core * row_factor;
        if intensity > best_intensity {
            best_intensity = intensity;
            // Hue base per comet plus slight time shift & dx offset
            best_hue = ((i as f32)*90.0 + time*50.0 - dx*4.0).rem_euclid(360.0);
        }
    }
    if best_intensity <= 0.0 { return (0,0,0); }
    // Convert intensity to brightness; add a mild saturation ramp
    let sat = 0.55 + best_intensity * 0.4;
    let val = 0.25 + best_intensity * 0.75;
    hsv_to_rgb(best_hue, sat.min(1.0), val.min(1.0))
}
