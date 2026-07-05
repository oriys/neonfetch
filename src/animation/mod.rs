pub mod aurora;
pub mod fall;
pub mod fire;
pub mod lava;
pub mod marquee;
pub mod matrix;
pub mod meteor;
pub mod palette;
pub mod plasma;
pub mod pulse_rings;
pub mod styles;

pub use aurora::calculate_aurora_color_with_palette;
pub use fall::FallSim;
pub use fire::calculate_fire_color_with_palette;
pub use lava::calculate_lava_color_with_palette;
pub use marquee::calculate_marquee_color_with_palette;
pub use matrix::calculate_matrix_color_with_palette;
pub use meteor::calculate_meteor_color_with_palette;
pub use palette::{Palette, available_palette_names, default_palette, find_palette};
pub use plasma::calculate_plasma_color_with_palette;
pub use pulse_rings::calculate_pulse_rings_color_with_palette;
pub use styles::{AnimationStyle, calculate_color_with_palette};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_palette_preserves_color_functions() {
        let palette = default_palette();
        let generic_styles = [
            AnimationStyle::Wave,
            AnimationStyle::Pulse,
            AnimationStyle::Neon,
            AnimationStyle::Glow,
            AnimationStyle::Pixel,
        ];
        for style in generic_styles {
            assert_eq!(
                calculate_color_with_palette(&style, 1.25, 17, palette),
                styles::calculate_color(&style, 1.25, 17)
            );
        }

        assert_eq!(
            calculate_matrix_color_with_palette(1.25, 4, 7, 24, palette),
            matrix::calculate_matrix_color_at(1.25, 4, 7, 24)
        );
        assert_eq!(
            calculate_fire_color_with_palette(1.25, 4, 7, 24, 80, palette),
            fire::calculate_fire_color_at(1.25, 4, 7, 24, 80)
        );
        assert_eq!(
            calculate_marquee_color_with_palette(1.25, 4, 7, 80, palette),
            marquee::calculate_marquee_color_at(1.25, 4, 7, 80)
        );
        assert_eq!(
            calculate_plasma_color_with_palette(1.25, 4, 7, 80, 24, 1.0, palette),
            plasma::calculate_plasma_color_at(1.25, 4, 7, 80, 24, 1.0)
        );
        assert_eq!(
            calculate_aurora_color_with_palette(1.25, 4, 7, 80, 24, 1.0, palette),
            aurora::calculate_aurora_color_at(1.25, 4, 7, 80, 24, 1.0)
        );
        assert_eq!(
            calculate_pulse_rings_color_with_palette(1.25, 4, 7, 80, 24, 1.0, palette),
            pulse_rings::calculate_pulse_rings_color_at(1.25, 4, 7, 80, 24, 1.0)
        );
        assert_eq!(
            calculate_lava_color_with_palette(1.25, 4, 7, 80, 24, 1.0, palette),
            lava::calculate_lava_color_at(1.25, 4, 7, 80, 24, 1.0)
        );
        assert_eq!(
            calculate_meteor_color_with_palette(1.25, 4, 7, 80, 24, palette),
            meteor::calculate_meteor_color_at(1.25, 4, 7, 80, 24)
        );
    }

    #[test]
    fn non_default_palette_changes_color_output() {
        let dracula = find_palette("dracula").expect("dracula palette exists");
        assert_ne!(
            calculate_color_with_palette(&AnimationStyle::Neon, 1.25, 17, dracula),
            styles::calculate_color(&AnimationStyle::Neon, 1.25, 17)
        );
    }
}
