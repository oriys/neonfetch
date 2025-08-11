pub mod fire;
pub mod matrix;
pub mod styles;
pub mod marquee;
pub mod rainbow_outline;

pub use fire::calculate_fire_color_at;
pub use matrix::calculate_matrix_color_at;
pub use marquee::calculate_marquee_color_at;
pub use rainbow_outline::calculate_rainbow_outline_color_at;
pub use styles::{AnimationStyle, calculate_color};
