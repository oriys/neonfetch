pub mod info;
mod logo_macos;
mod logo_linux;
mod logo_default;

pub use info::generate_system_info;

#[cfg(target_os = "macos")]
pub fn ascii_logo() -> Vec<&'static str> { logo_macos::ascii_logo() }
#[cfg(target_os = "linux")]
pub fn ascii_logo() -> Vec<&'static str> { logo_linux::ascii_logo() }
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn ascii_logo() -> Vec<&'static str> { logo_default::ascii_logo() }
