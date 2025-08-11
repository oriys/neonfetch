pub mod fire;
pub mod matrix;
pub mod styles;
pub mod marquee;
pub mod plasma;
pub mod embers;

pub use fire::calculate_fire_color_at;
pub use matrix::calculate_matrix_color_at;
pub use marquee::calculate_marquee_color_at;
pub use plasma::calculate_plasma_color_at;
pub use embers::calculate_embers_color_at;
pub use styles::{AnimationStyle, calculate_color};
