pub mod info;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod logo_default;
mod logo_distro;
#[cfg(target_os = "linux")]
mod logo_linux;
#[cfg(target_os = "macos")]
mod logo_macos;

pub use info::generate_system_info;
pub use logo_distro::{logo_for_distro, supported_distro_ids};

#[cfg(target_os = "macos")]
pub fn ascii_logo() -> Vec<&'static str> {
    logo_macos::ascii_logo()
}
#[cfg(target_os = "linux")]
pub fn ascii_logo() -> Vec<&'static str> {
    logo_linux::ascii_logo()
}
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn ascii_logo() -> Vec<&'static str> {
    logo_default::ascii_logo()
}

pub fn ascii_logo_with_distro(distro_id: Option<&str>) -> Vec<&'static str> {
    if let Some(id) = distro_id {
        if let Some(logo) = logo_for_distro(id) {
            return logo;
        }
        eprintln!("neonfetch: unknown distro id `{id}`, using platform default logo");
    }

    ascii_logo()
}
