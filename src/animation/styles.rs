use std::f32;

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationStyle {
    Wave,
    Pulse,
    Neon,
    Matrix,
    Fire,
    Fall,
    Marquee,
}

impl AnimationStyle {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            // classic removed; map legacy aliases to Neon for backward compatibility
            "classic" | "c" => AnimationStyle::Neon,
            "wave" | "w" => AnimationStyle::Wave,
            "pulse" | "p" => AnimationStyle::Pulse,
            "neon" | "n" => AnimationStyle::Neon,
            "matrix" | "m" => AnimationStyle::Matrix,
            "fire" | "f" => AnimationStyle::Fire,
            "fall" | "stack" | "s" => AnimationStyle::Fall,
            "marquee" | "mq" => AnimationStyle::Marquee,
            _ => AnimationStyle::Neon,
        }
    }
}

// HSV to RGB conversion (used by neon / pulse styles)
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let r = ((r_prime + m) * 255.0) as u8;
    let g = ((g_prime + m) * 255.0) as u8;
    let b = ((b_prime + m) * 255.0) as u8;
    (r, g, b)
}

// (former rainbow helper removed after Classic style removal)

pub fn calculate_color(
    style: &AnimationStyle,
    _freq: f32,
    _i: f32,
    time: f32,
    char_pos: usize,
) -> (u8, u8, u8) {
    match style {
        AnimationStyle::Wave => {
            let spatial = char_pos as f32 * 0.35;
            let phase_primary = spatial - time * 3.0;
            let phase_secondary = char_pos as f32 * 0.10 - time * 0.7;
            let w = phase_primary.sin();
            let w_norm = w * 0.5 + 0.5;
            let mut v = 0.35 + w_norm.powf(1.2) * 0.65;
            let env = (phase_secondary.sin() * 0.5 + 0.5) * 0.25 + 0.75;
            v = (v * env).min(1.0);
            let hue = (time * 8.0) % 360.0;
            let sat = 0.65;
            hsv_to_rgb(hue, sat, v)
        }
        AnimationStyle::Pulse => {
            let base_hue = (time * 25.0) % 360.0;
            let hue_ripple = ((char_pos as f32) * 0.035 + time * 0.6).sin() * 6.0;
            let hue = (base_hue + hue_ripple + 360.0) % 360.0;
            let phase = time * 2.2 - (char_pos as f32) * 0.10;
            let wave = phase.sin();
            let wave_norm = (wave * 0.5 + 0.5).powf(1.3);
            let val = 0.25 + wave_norm * 0.75;
            let sat = 0.55 + wave_norm * 0.35;
            let breath = (time * 0.9).sin() * 0.04 + 0.96;
            let (mut r, mut g, mut b) = hsv_to_rgb(hue, sat, (val * breath).min(1.0));
            if val > 0.8 {
                let glow_mix = ((val - 0.8) / 0.2).clamp(0.0, 1.0) * 0.18;
                let gr = 230.0;
                r = (r as f32 * (1.0 - glow_mix) + gr * glow_mix) as u8;
                g = (g as f32 * (1.0 - glow_mix) + gr * glow_mix) as u8;
                b = (b as f32 * (1.0 - glow_mix) + gr * glow_mix) as u8;
            }
            (r, g, b)
        }
        AnimationStyle::Neon => {
            let base_hue = (time * 20.0) % 360.0;
            let span = 20.0_f32;
            let direction = (time * 0.9).sin();
            let centered = (((char_pos as f32) * 0.08).sin() * 0.5 + 0.5) - 0.5;
            let offset = centered * direction * span;
            let hue = (base_hue + offset + 360.0) % 360.0;
            let breath = (time * 1.2).sin() * 0.04 + 1.0;
            let sat = 0.72;
            let val = 0.82 * breath;
            let (mut r, mut g, mut b) = hsv_to_rgb(hue, sat, val);
            let mix = 0.05;
            r = ((r as f32 * (1.0 - mix)) + 128.0 * mix) as u8;
            g = ((g as f32 * (1.0 - mix)) + 128.0 * mix) as u8;
            b = ((b as f32 * (1.0 - mix)) + 128.0 * mix) as u8;
            (r, g, b)
        }
    AnimationStyle::Matrix => (0, 255, 0), // Actual color generated in matrix::calculate_matrix_color_at
    AnimationStyle::Fire => (255, 80, 0),  // Actual color generated in fire::calculate_fire_color_at
    AnimationStyle::Fall => (200, 200, 200), // Actual color generated in fall simulation renderer
    AnimationStyle::Marquee => (160,160,160), // Actual color generated in marquee::calculate_marquee_color_at
    }
}
