const SUPPORTED_DISTRO_IDS: &[&str] = &[
    "arch",
    "ubuntu",
    "debian",
    "fedora",
    "alpine",
    "nixos",
    "manjaro",
    "opensuse",
    "gentoo",
    "linuxmint",
    "kali",
    "void",
];

const ARCH: &[&str] = &[
    "            /\\            ",
    "           /  \\           ",
    "          /\\   \\          ",
    "         /      \\         ",
    "        /   ,,   \\        ",
    "       /   |  |   \\       ",
    "      /_-''    ''-_\\      ",
    "     /              \\     ",
    "    /      ____      \\    ",
    "   /      / __ \\      \\   ",
    "  /      / /  \\ \\      \\  ",
    " /______/ /____\\ \\______\\ ",
    "/________________________\\",
    "          /______\\        ",
    "         /________\\       ",
    "        /__________\\      ",
    "                          ",
    "                          ",
];

const UBUNTU: &[&str] = &[
    "                          ",
    "          .------.        ",
    "       .-'  .--.  '-.     ",
    "      /   .'    '.   \\    ",
    "     |   /  o  o  \\   |   ",
    "     |  |    __    |  |   ",
    "      \\  \\  (__)  /  /    ",
    "       '-.        .-'      ",
    "      .--'  ____  '--.     ",
    "    .'   .-'    '-.   '.   ",
    "   /    /  .--.    \\    \\  ",
    "  |    |  (    )    |    | ",
    "   \\    \\  '--'    /    /  ",
    "    '.   '-.____.-'   .'   ",
    "      '--.        .--'     ",
    "          '------'         ",
    "                          ",
    "                          ",
];

const DEBIAN: &[&str] = &[
    "                          ",
    "          ______          ",
    "       .-'      '-.       ",
    "      /   .----.   \\      ",
    "     /   /      \\   \\     ",
    "    |   |  .--.  |   |    ",
    "    |   | (    ) |   |    ",
    "     \\   \\ '--' /   /     ",
    "      '.  '----'  .'      ",
    "        '-.____.-'        ",
    "           .--.           ",
    "          /    \\          ",
    "          \\    /          ",
    "           '--'           ",
    "                          ",
    "                          ",
    "                          ",
    "                          ",
];

const FEDORA: &[&str] = &[
    "                          ",
    "        __________        ",
    "      .'  ______  '.      ",
    "     /   /      \\   \\     ",
    "    |   |  ____  |   |    ",
    "    |   | |  __| |   |    ",
    "    |   | | |__  |   |    ",
    "    |   | |____| |   |    ",
    "    |   |  ____  |   |    ",
    "    |   | |    | |   |    ",
    "     \\   \\|____|/   /     ",
    "      '.          .'      ",
    "        '--------'        ",
    "                          ",
    "                          ",
    "                          ",
    "                          ",
    "                          ",
];

const ALPINE: &[&str] = &[
    "                          ",
    "            /\\            ",
    "           /  \\           ",
    "          / /\\ \\          ",
    "         / /  \\ \\         ",
    "        / /____\\ \\        ",
    "       /_/      \\_\\       ",
    "       \\ \\  /\\  / /       ",
    "        \\ \\/  \\/ /        ",
    "         \\      /         ",
    "          \\ /\\ /          ",
    "           '  '           ",
    "                          ",
    "                          ",
    "                          ",
    "                          ",
    "                          ",
    "                          ",
];

const NIXOS: &[&str] = &[
    "                          ",
    "       \\\\      //         ",
    "        \\\\    //          ",
    "     ====\\\\==//====       ",
    "          \\\\//            ",
    "          //\\\\            ",
    "     ====//==\\\\====       ",
    "        //    \\\\          ",
    "       //      \\\\         ",
    "       \\\\      //         ",
    "        \\\\    //          ",
    "     ====\\\\==//====       ",
    "          \\\\//            ",
    "          //\\\\            ",
    "     ====//==\\\\====       ",
    "        //    \\\\          ",
    "                          ",
    "                          ",
];

const MANJARO: &[&str] = &[
    "                          ",
    "   ###################    ",
    "   ###################    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   #######       #####    ",
    "   ###################    ",
    "                          ",
    "                          ",
];

const OPENSUSE: &[&str] = &[
    "                          ",
    "        .--------.        ",
    "     .-'          '-.     ",
    "    /   .------.     \\    ",
    "   |   /  .--.  \\     |   ",
    "   |  |  ( oo )  |    |   ",
    "   |   \\  '--'  /    /    ",
    "    \\   '------'    /     ",
    "     '-.        .--'      ",
    "        '--.__.-'         ",
    "            \\             ",
    "             \\____        ",
    "                  '-.     ",
    "                    |     ",
    "              .____.'     ",
    "                          ",
    "                          ",
    "                          ",
];

const GENTOO: &[&str] = &[
    "                          ",
    "          ________        ",
    "       .-'        '-.     ",
    "     .'   .----.     '.   ",
    "    /    /      \\      \\  ",
    "   |    |  .--.  |      | ",
    "   |    | (    ) |      | ",
    "    \\    \\ '--' /      /  ",
    "     '.   '----'    .-'   ",
    "       '-.       .-'      ",
    "          '.__.-'         ",
    "            /             ",
    "           /____          ",
    "                '-.       ",
    "                  |       ",
    "             ____.'       ",
    "                          ",
    "                          ",
];

const LINUXMINT: &[&str] = &[
    "                          ",
    "     ################     ",
    "    ##################    ",
    "   ####            ####   ",
    "   ###   ########   ###   ",
    "   ###   ########   ###   ",
    "   ###   ###  ###   ###   ",
    "   ###   ###  ###   ###   ",
    "   ###   ###  ###   ###   ",
    "   ###   ###  ###   ###   ",
    "   ###   ###  ###   ###   ",
    "   ###   ########   ###   ",
    "   ###   ########   ###   ",
    "   ####            ####   ",
    "    ##################    ",
    "     ################     ",
    "                          ",
    "                          ",
];

const KALI: &[&str] = &[
    "                          ",
    "      ______________      ",
    "   .-'              '-.   ",
    "  /   _.-''''''-._     \\  ",
    " |   /  .------.  \\     | ",
    " |  |  /  /\\    \\  |    | ",
    " |  | |  /  \\    | |    | ",
    " |  | | /____\\   | |    | ",
    " |   \\ \\      \\ / /    /  ",
    "  \\   '-.____.-'     .'   ",
    "   '-.            .-'     ",
    "      '--.____.--'        ",
    "          / /             ",
    "         /_/              ",
    "                          ",
    "                          ",
    "                          ",
    "                          ",
];

const VOID: &[&str] = &[
    "                          ",
    "          ______          ",
    "       .-'      '-.       ",
    "      /   .--.     \\      ",
    "     |   /    \\     |     ",
    "     |  |  ()  |    |     ",
    "     |   \\    /     |     ",
    "      \\   '--'     /      ",
    "       '-.      .-'       ",
    "          '--.-'          ",
    "          .-'-.           ",
    "       .-'     '-.        ",
    "      /   .--.    \\       ",
    "     |   (    )    |      ",
    "      \\   '--'    /       ",
    "       '-.____.-'         ",
    "                          ",
    "                          ",
];

pub fn supported_distro_ids() -> &'static [&'static str] {
    SUPPORTED_DISTRO_IDS
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn distro_id_from_os_release(content: &str) -> Option<String> {
    let mut id = None;
    let mut id_like = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        match key.trim() {
            "ID" => id = Some(parse_os_release_value(value)),
            "ID_LIKE" => id_like = Some(parse_os_release_value(value)),
            _ => {}
        }
    }

    if let Some(id) = id.as_deref().and_then(canonical_distro_id) {
        return Some(id.to_string());
    }

    id_like.and_then(|likes| {
        likes
            .split_whitespace()
            .find_map(canonical_distro_id)
            .map(str::to_string)
    })
}

pub fn logo_for_distro(id: &str) -> Option<Vec<&'static str>> {
    Some(
        match canonical_distro_id(id)? {
            "arch" => ARCH,
            "ubuntu" => UBUNTU,
            "debian" => DEBIAN,
            "fedora" => FEDORA,
            "alpine" => ALPINE,
            "nixos" => NIXOS,
            "manjaro" => MANJARO,
            "opensuse" => OPENSUSE,
            "gentoo" => GENTOO,
            "linuxmint" => LINUXMINT,
            "kali" => KALI,
            "void" => VOID,
            _ => return None,
        }
        .to_vec(),
    )
}

fn canonical_distro_id(id: &str) -> Option<&'static str> {
    let id = id.trim().trim_matches('"').trim_matches('\'');
    let normalized = id.to_ascii_lowercase();
    match normalized.as_str() {
        "arch" | "archlinux" => Some("arch"),
        "ubuntu" => Some("ubuntu"),
        "debian" => Some("debian"),
        "fedora" => Some("fedora"),
        "alpine" => Some("alpine"),
        "nixos" | "nix" => Some("nixos"),
        "manjaro" => Some("manjaro"),
        "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" | "suse" => Some("opensuse"),
        "gentoo" => Some("gentoo"),
        "linuxmint" | "mint" => Some("linuxmint"),
        "kali" | "kali-linux" => Some("kali"),
        "void" | "voidlinux" => Some("void"),
        _ => None,
    }
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_os_release_value(value: &str) -> String {
    let value = value.trim();
    let bytes = value.as_bytes();
    if value.len() >= 2
        && ((bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\''))
    {
        return value[1..value.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\'", "'")
            .replace("\\\\", "\\");
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::{distro_id_from_os_release, logo_for_distro, supported_distro_ids};

    #[test]
    fn parses_known_distro_ids() {
        let cases = [
            ("ID=arch\n", "arch"),
            ("ID=ubuntu\n", "ubuntu"),
            ("ID=debian\n", "debian"),
            ("ID=fedora\n", "fedora"),
            ("ID=alpine\n", "alpine"),
            ("ID=nixos\n", "nixos"),
            ("ID=manjaro\n", "manjaro"),
            ("ID=opensuse-tumbleweed\n", "opensuse"),
            ("ID=gentoo\n", "gentoo"),
            ("ID=linuxmint\n", "linuxmint"),
            ("ID=kali\n", "kali"),
            ("ID=void\n", "void"),
        ];

        for (content, expected) in cases {
            assert_eq!(
                distro_id_from_os_release(content).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn falls_back_to_first_known_id_like() {
        let content = r#"
NAME="Example Linux"
ID=unknown
ID_LIKE="arch debian"
"#;

        assert_eq!(distro_id_from_os_release(content).as_deref(), Some("arch"));

        let elementary = r#"
NAME="elementary OS"
ID=elementary
ID_LIKE="ubuntu debian"
"#;

        assert_eq!(
            distro_id_from_os_release(elementary).as_deref(),
            Some("ubuntu")
        );
    }

    #[test]
    fn returns_none_for_unknown_distros() {
        let content = r#"
NAME="Example Linux"
ID=unknown
ID_LIKE="other base"
"#;

        assert_eq!(distro_id_from_os_release(content), None);
    }

    #[test]
    fn logos_are_non_empty_and_do_not_contain_ansi() {
        for id in supported_distro_ids() {
            let logo = logo_for_distro(id).expect("known distro should have a logo");
            assert!(!logo.is_empty(), "{id} logo must not be empty");
            for line in logo {
                assert!(!line.is_empty(), "{id} logo line must not be empty");
                assert!(
                    !line.as_bytes().contains(&0x1b),
                    "{id} logo line must not contain ANSI escapes"
                );
            }
        }
    }
}
