use std::fs;

// Detect Linux distribution via /etc/os-release (ID=) or env override NEONFETCH_FORCE_DISTRO
fn detect_distro_id() -> Option<String> {
    if let Ok(val) = std::env::var("NEONFETCH_FORCE_DISTRO") {
        if !val.trim().is_empty() {
            return Some(val.to_ascii_lowercase());
        }
    }
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            let l = line.trim();
            if let Some(rest) = l.strip_prefix("ID=") {
                let raw = rest.trim().trim_matches('"').trim_matches('\'');
                if !raw.is_empty() {
                    return Some(raw.to_ascii_lowercase());
                }
            }
        }
    }
    None
}

fn logo_arch() -> Vec<&'static str> {
    vec![
        "               ",
        "      /\\      ",
        "     /  \\     ",
        "    / /\\ \\    ",
        "   / ____ \\   ",
        "  /_/    \\_\\  ",
        "               ",
    ]
}

fn logo_ubuntu() -> Vec<&'static str> {
    vec![
        "                ",
        "    ####  ####  ",
        "   #    ##    # ",
        "   #  ######  # ",
        "   #  ######  # ",
        "   #    ##    # ",
        "    ####  ####  ",
        "                ",
    ]
}

fn logo_debian() -> Vec<&'static str> {
    vec![
        "                 ",
        "      ######     ",
        "    ##      #    ",
        "   #   ####  #   ",
        "   #  #    # #   ",
        "   #   ####  #   ",
        "    ##      #    ",
        "      ######     ",
        "                 ",
    ]
}

fn logo_fedora() -> Vec<&'static str> {
    vec![
        "              ",
        "   #######    ",
        "  #   #  ##   ",
        "  #  ##  #    ",
        "  # # #  #    ",
        "  ##  #  #    ",
        "    ######    ",
        "              ",
    ]
}

fn logo_manjaro() -> Vec<&'static str> {
    vec![
        "               ",
        "  ###########  ",
        "  ###########  ",
        "  ####   ####  ",
        "  ####   ####  ",
        "  ####   ####  ",
        "  ####   ####  ",
        "               ",
    ]
}

fn logo_gentoo() -> Vec<&'static str> {
    vec![
        "                ",
        "    #######     ",
        "  ##   ##  ##   ",
        "  #  ##  ## #   ",
        "  #  #  ##  #   ",
        "  ##  ##   ##   ",
        "    #######     ",
        "                ",
    ]
}

fn logo_alpine() -> Vec<&'static str> {
    vec![
        "               ",
        "     /\\\\      ",
        "    / /\\\\     ",
        "   / /  \\ \\    ",
        "  /_/____\\_\\   ",
        "  \\ \\    / /   ",
        "   \\ \\__/ /    ",
        "    \\____/     ",
        "               ",
    ]
}

fn logo_opensuse() -> Vec<&'static str> {
    vec![
        "                 ",
        "    #########    ",
        "   #   ###   #   ",
        "   #  #   #  #   ",
        "   #  #   #  #   ",
        "   #   ###   #   ",
        "    #########    ",
        "        #        ",
        "                 ",
    ]
}

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
    match detect_distro_id().as_deref() {
        Some("arch") => logo_arch(),
        Some("ubuntu") => logo_ubuntu(),
        Some("debian") => logo_debian(),
        Some("fedora") => logo_fedora(),
        Some("manjaro") => logo_manjaro(),
        Some("gentoo") => logo_gentoo(),
        Some("alpine") => logo_alpine(),
        Some("opensuse" | "suse") => logo_opensuse(),
        _ => logo_fallback(),
    }
}
