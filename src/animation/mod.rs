pub mod fire;
pub mod matrix;
pub mod styles;
pub mod marquee;
pub mod plasma;
pub mod aurora;
pub mod pulse_rings;
pub mod meteor_rain;
pub mod lava;

pub use fire::calculate_fire_color_at;
pub use matrix::calculate_matrix_color_at;
pub use marquee::calculate_marquee_color_at;
pub use plasma::calculate_plasma_color_at;
pub use aurora::calculate_aurora_color_at;
pub use pulse_rings::calculate_pulse_rings_color_at;
pub use meteor_rain::{Meteor, update_meteors, sample_cell_color as sample_meteor_color};
pub use lava::calculate_lava_color_at;
pub use styles::{AnimationStyle, calculate_color};
