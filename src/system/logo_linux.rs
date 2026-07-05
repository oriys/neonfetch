use super::logo_distro::{distro_id_from_os_release, logo_for_distro};
use std::fs;

fn logo_fallback() -> Vec<&'static str> {
    vec![
        "       #####           ",
        "      #######          ",
        "      ##O#O##          ",
        "      #######          ",
        "    ###########        ",
        "   #############       ",
        "  ###############      ",
        "  ################     ",
        " ###################   ",
        "#####################  ",
        "#####################  ",
        "#####################  ",
    ]
}

pub fn ascii_logo() -> Vec<&'static str> {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| distro_id_from_os_release(&content))
        .and_then(|id| logo_for_distro(&id))
        .unwrap_or_else(logo_fallback)
}
