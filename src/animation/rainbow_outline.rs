use super::styles::hsv_to_rgb;

// Rainbow outline effect: brighten & color cycle only characters at the edge of the ASCII logo/text block.
// Interior characters glow subtly.
// Strategy:
// 1. Caller provides (row,col) within logical text grid.
// 2. We receive a precomputed mask from main (edge determination). Here, to stay stateless, we recompute edge cheaply based on surrounding occupancy info passed inline.
// Simplicity tradeoff: We'll approximate edge by using spacing pattern: treat space ' ' as empty; any non-space neighboring space => edge.
// We'll require main to pass a closure? Simpler: we pass a small neighborhood occupancy bitmask encoded into a u8 computed in main.
// For minimal invasive change we'll (for now) duplicate a quick edge detection by letting main give us neighbor mask: bit0 up, bit1 down, bit2 left, bit3 right empty flags.
// If that's too intrusive, fallback: if neighbor mask all zeros we treat as interior.

// Bits meaning for neighbor_mask: 1=up empty, 2=down empty, 4=left empty, 8=right empty.
pub fn calculate_rainbow_outline_color_at(
    time: f32,
    _row: usize,
    _col: usize,
    neighbor_mask: u8,
) -> (u8,u8,u8) {
    let is_edge = neighbor_mask & 0b1111 != 0; // any side adjacent to empty
    if is_edge {
        // Hue cycles fast; slight brightness pulse.
        let hue = (time * 120.0) % 360.0;
        let pulse = (time * 2.2).sin() * 0.25 + 0.75; // 0.5..1.0
        let (r,g,b) = hsv_to_rgb(hue, 0.85, pulse.min(1.0));
        (r,g,b)
    } else {
        // Interior subtle desaturated breathing.
        let hue = (time * 25.0) % 360.0;
        let breath = (time * 1.1).sin() * 0.05 + 0.85; // 0.80..0.90
        hsv_to_rgb(hue, 0.25, breath)
    }
}
