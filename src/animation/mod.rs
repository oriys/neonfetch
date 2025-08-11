pub mod aurora;
pub mod fire;
pub mod lava;
pub mod matrix;
pub mod plasma;
pub mod pulse_rings;
pub mod styles;

pub use aurora::calculate_aurora_color_at;
pub use fire::calculate_fire_color_at;
pub use lava::calculate_lava_color_at;
pub use matrix::calculate_matrix_color_at;
pub use plasma::calculate_plasma_color_at;
pub use pulse_rings::calculate_pulse_rings_color_at;
pub use styles::{AnimationStyle, calculate_color};
