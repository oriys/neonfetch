#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub name: &'static str,
    pub stops: &'static [(u8, u8, u8)],
}

const DEFAULT_STOPS: &[(u8, u8, u8)] =
    &[(0x00, 0xff, 0xff), (0xff, 0x4f, 0xc3), (0xff, 0xff, 0xff)];
const CYBERPUNK_STOPS: &[(u8, u8, u8)] = &[
    (0x00, 0xff, 0xea),
    (0xff, 0x00, 0x7f),
    (0xff, 0xf2, 0x00),
    (0x7a, 0x5c, 0xff),
    (0x00, 0xb3, 0xff),
];
const DRACULA_STOPS: &[(u8, u8, u8)] = &[
    (0xff, 0x79, 0xc6),
    (0xbd, 0x93, 0xf9),
    (0x8b, 0xe9, 0xfd),
    (0x50, 0xfa, 0x7b),
    (0xf1, 0xfa, 0x8c),
    (0xff, 0xb8, 0x6c),
    (0xff, 0x55, 0x55),
];
const PASTEL_STOPS: &[(u8, u8, u8)] = &[
    (0xff, 0xd1, 0xdc),
    (0xff, 0xda, 0xc1),
    (0xff, 0xff, 0xd1),
    (0xc1, 0xe1, 0xc1),
    (0xb5, 0xea, 0xd7),
    (0xc7, 0xce, 0xea),
];
const SUNSET_STOPS: &[(u8, u8, u8)] = &[
    (0x52, 0x1b, 0x7a),
    (0x8a, 0x23, 0x87),
    (0xe9, 0x40, 0x57),
    (0xf2, 0x71, 0x21),
    (0xff, 0xd1, 0x66),
];
const OCEAN_STOPS: &[(u8, u8, u8)] = &[
    (0x02, 0x3e, 0x8a),
    (0x00, 0x77, 0xb6),
    (0x00, 0xb4, 0xd8),
    (0x48, 0xca, 0xe4),
    (0x90, 0xe0, 0xef),
    (0xca, 0xf0, 0xf8),
];
const MONO_STOPS: &[(u8, u8, u8)] = &[
    (0xff, 0xff, 0xff),
    (0xd8, 0xd8, 0xd8),
    (0xb0, 0xb0, 0xb0),
    (0x88, 0x88, 0x88),
    (0xe8, 0xe8, 0xe8),
];

pub const DEFAULT_PALETTE: Palette = Palette {
    name: "default",
    stops: DEFAULT_STOPS,
};

pub const PALETTES: &[Palette] = &[
    DEFAULT_PALETTE,
    Palette {
        name: "cyberpunk",
        stops: CYBERPUNK_STOPS,
    },
    Palette {
        name: "dracula",
        stops: DRACULA_STOPS,
    },
    Palette {
        name: "pastel",
        stops: PASTEL_STOPS,
    },
    Palette {
        name: "sunset",
        stops: SUNSET_STOPS,
    },
    Palette {
        name: "ocean",
        stops: OCEAN_STOPS,
    },
    Palette {
        name: "mono",
        stops: MONO_STOPS,
    },
];

impl Palette {
    pub fn sample(&self, t: f32) -> (u8, u8, u8) {
        match self.stops {
            [] => (255, 255, 255),
            [only] => *only,
            stops => {
                let t = if t.is_finite() {
                    t.rem_euclid(1.0)
                } else {
                    0.0
                };
                let scaled = t * (stops.len() - 1) as f32;
                let idx = scaled.floor() as usize;
                let next = (idx + 1).min(stops.len() - 1);
                let local = scaled - idx as f32;
                lerp_rgb(stops[idx], stops[next], local)
            }
        }
    }

    pub fn sample_tinted(&self, t: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
        let (r, g, b) = self.sample(t);
        let (rf, gf, bf) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let max_channel = rf.max(gf).max(bf).max(1.0 / 255.0);
        let sat = saturation.clamp(0.0, 1.0);
        let val = value.clamp(0.0, 1.0);
        let tint = |channel: f32| -> u8 {
            (((1.0 - sat) + (channel / max_channel) * sat) * val * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        (tint(rf), tint(gf), tint(bf))
    }

    pub fn is_default(&self) -> bool {
        self.name == DEFAULT_PALETTE.name
    }
}

pub fn available_palette_names() -> Vec<&'static str> {
    PALETTES.iter().map(|palette| palette.name).collect()
}

pub fn default_palette() -> &'static Palette {
    &PALETTES[0]
}

pub fn find_palette(name: &str) -> Option<&'static Palette> {
    PALETTES
        .iter()
        .find(|palette| palette.name.eq_ignore_ascii_case(name))
}

pub fn palette_or_default(name: &str) -> &'static Palette {
    find_palette(name).unwrap_or_else(default_palette)
}

fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let lerp = |start: u8, end: u8| -> u8 {
        (start as f32 + (end as f32 - start as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    (lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PALETTE: Palette = Palette {
        name: "test",
        stops: &[(0, 0, 0), (255, 255, 255)],
    };

    #[test]
    fn sample_starts_at_first_stop() {
        assert_eq!(TEST_PALETTE.sample(0.0), (0, 0, 0));
    }

    #[test]
    fn sample_wraps_at_one() {
        assert_eq!(TEST_PALETTE.sample(1.0), (0, 0, 0));
    }

    #[test]
    fn sample_interpolates_midpoint_between_two_stops() {
        assert_eq!(TEST_PALETTE.sample(0.5), (128, 128, 128));
    }

    #[test]
    fn unknown_palette_falls_back_to_default() {
        assert_eq!(palette_or_default("missing").name, "default");
    }
}
