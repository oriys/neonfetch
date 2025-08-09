// Simple XOR-shift style PRNG producing a float in 0..1 (deterministic per column)
fn prng01_from_u32(mut x: u32) -> f32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    (x as f32 / u32::MAX as f32).clamp(0.0, 1.0)
}

pub fn calculate_matrix_color_at(
    time: f32,
    row: usize,
    col: usize,
    term_height: usize,
) -> (u8, u8, u8) {
    let seed = (col as u32)
        .wrapping_mul(747796405u32)
        .wrapping_add(2891336453u32);
    let r1 = prng01_from_u32(seed);
    let r2 = prng01_from_u32(seed.wrapping_mul(1664525).wrapping_add(1013904223));
    let r3 = prng01_from_u32(seed ^ 0x9E3779B9);

    let speed = 0.6 + r1 * 1.0;
    let phase = r2 * term_height as f32;
    let trail_len = 8.0 + r3 * 16.0;
    let cycle = term_height as f32 + trail_len + 5.0;
    let head = (time * speed + phase) % cycle;
    let y = row as f32;
    let dist = (y - head).abs();

    let intensity = if y >= head - trail_len && y <= head {
        if dist < 0.8 {
            1.0
        } else {
            (1.0 - (dist / trail_len)).clamp(0.0, 1.0) * 0.85 + 0.15
        }
    } else {
        0.0
    };

    let flicker = if (((time * 30.0) as i32 + col as i32) % 47) == 0 {
        1.25
    } else {
        1.0
    };
    let g = (255.0 * intensity * flicker).clamp(0.0, 255.0) as u8;
    let o = (g as f32 * 0.12) as u8;
    (o, g, o)
}
