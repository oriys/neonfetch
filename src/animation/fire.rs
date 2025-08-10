pub fn calculate_fire_color_at(
    time: f32,
    row: usize,
    col: usize,
    term_height: usize,
) -> (u8, u8, u8) {
    if term_height == 0 {
        return (255, 80, 0);
    }
    let h = 1.0 - (row as f32 / term_height as f32);
    let seed = (col as u32).wrapping_mul(1103515245).wrapping_add(12345);
    let wind = (((seed as f32) * 0.000001).sin() * 0.5 + 0.5) * 0.6 + 0.4;
    let adv = time * (0.9 + (col as f32 * 0.05).sin() * 0.2);
    let n1 = (row as f32 * 0.18 + adv).sin() * 0.5 + 0.5;
    let n2 = (col as f32 * 0.12 + adv * wind).cos() * 0.5 + 0.5;
    let n = (n1 * 0.6 + n2 * 0.4).clamp(0.0, 1.0);
    let base = h.powf(0.6);
    // Removed spark / firework random brightening effect for a steadier flame
    let intensity = (0.25 + 0.75 * base * n).clamp(0.0, 1.0);
    let (r, g, b) = if intensity < 0.5 {
        let t = intensity / 0.5;
        let r = 180.0 + t * (255.0 - 180.0);
        let g = 20.0 + t * (80.0 - 20.0);
        (r as u8, g as u8, 0)
    } else if intensity < 0.85 {
        let t = (intensity - 0.5) / 0.35;
        let r = 255.0;
        let g = 80.0 + t * (200.0 - 80.0);
        (r as u8, g as u8, 0)
    } else {
        let t = (intensity - 0.85) / 0.15;
        let r = 255.0;
        let g = 200.0 + t * (255.0 - 200.0);
        let b = 0.0 + t * (180.0 - 0.0);
        (r as u8, g as u8, b as u8)
    };
    (r, g, b)
}
