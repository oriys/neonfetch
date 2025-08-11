use crate::animation::styles::hsv_to_rgb;

// ASCII Kaleidoscope: Fold coordinates into N angular sectors and reuse mirrored angle & radius
// to produce repeating color wedges with radial pulsation.
pub fn calculate_kaleidoscope_color_at(time: f32, row: usize, col: usize, term_w: usize, term_h: usize, speed: f32) -> (u8,u8,u8) {
    if term_w == 0 || term_h == 0 { return (0,0,0); }
    let t = time * speed.max(0.05);
    let w = term_w as f32;
    let h = term_h as f32;
    let cx = w * 0.5;
    let cy = h * 0.5;
    // Normalized coordinates with simple aspect correction
    let x = (col as f32 + 0.5 - cx) / (w * 0.5);
    let mut y = (row as f32 + 0.2 - cy) / (h * 0.5);
    y *= w / h.max(1.0);
    let r = (x*x + y*y).sqrt();
    if r > 1.25 { return (0,0,0); } // hard cut outside circle for clearer shape
    let mut ang = y.atan2(x); // -PI..PI
    ang = ang.rem_euclid(std::f32::consts::PI * 2.0);
    // Fixed sector count for stable symmetry
    let sectors: f32 = 8.0;
    let slice = std::f32::consts::PI * 2.0 / sectors;
    let mut folded = ang % slice;
    let half = slice * 0.5;
    if folded > half { folded = slice - folded; }
    let norm_ang = (folded / half).clamp(0.0,1.0); // 0 center line => 1 edge
    // Center-line highlight & edge highlight components
    let center_line = (1.0 - norm_ang).powf(2.4); // bright where norm_ang small
    let edge_line = norm_ang.powf(3.2); // optional subtle edge glow
    // Radial petal waves
    let petals = ( (r * sectors * 1.2) - t * 3.0 ).sin() * 0.5 + 0.5;
    let radial_shell = ((r * 9.5) - t * 4.5).sin() * 0.5 + 0.5;
    // Hue composition: base rotation + angular + petal + shell coloring
    let hue = ( t * 40.0 + norm_ang * 160.0 + petals * 120.0 + radial_shell * 90.0 ) % 360.0;
    // Saturation: stronger in petals & center lines; desaturate far radius
    let sat_radius = (1.0 - r.powf(1.3)).clamp(0.0,1.0);
    let sat = (0.35 + (petals * 0.4 + center_line * 0.5) * sat_radius).min(0.95);
    // Value: combine center-line, petals, and radial shells; fade to edge
    let fade_edge = (1.0 - (r/1.1).powf(2.4)).clamp(0.0,1.0);
    let core = center_line * 0.7 + petals * 0.5 + radial_shell * 0.4 + edge_line * 0.25;
    let val = (0.10 + core * 0.9) * fade_edge;
    hsv_to_rgb(hue, sat, val.clamp(0.05,1.0))
}
