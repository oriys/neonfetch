use std::f32;

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationStyle {
    Wave,
    Pulse,
    Neon,
    Matrix,
    Fire,
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
            _ => AnimationStyle::Neon,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FireMode {
    Basic,
    Advanced,
}

impl FireMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "basic" | "b" => FireMode::Basic,
            "advanced" | "a" => FireMode::Advanced,
            _ => FireMode::Advanced,
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

fn rainbow(freq: f32, i: f32) -> (u8, u8, u8) {
    let red = (f32::sin(freq * i + 0.0) * 127.0 + 128.0) as u8;
    let green = (f32::sin(freq * i + 2.0 * std::f32::consts::PI / 3.0) * 127.0 + 128.0) as u8;
    let blue = (f32::sin(freq * i + 4.0 * std::f32::consts::PI / 3.0) * 127.0 + 128.0) as u8;
    (red, green, blue)
}

pub fn calculate_color(
    style: &AnimationStyle,
    freq: f32,
    i: f32,
    time: f32,
    char_pos: usize,
) -> (u8, u8, u8) {
    match style {
        AnimationStyle::Wave => {
            let wave_offset = (char_pos as f32 * 0.5 + time * 0.1).sin() * 50.0;
            rainbow(freq, i + wave_offset)
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
    }
}
