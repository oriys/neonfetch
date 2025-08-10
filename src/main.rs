mod animation; // animations & color logic
mod system; // system information collection
mod util; // shared utilities (e.g. ANSI parsing)

use animation::{
    AnimationStyle, calculate_color, calculate_fire_color_at, calculate_matrix_color_at,
};
use system::generate_system_info;

use crossterm::terminal::size;
use std::{
    env,
    io::{self, Write, stdout},
    thread,
    time::{Duration, Instant},
};

use util::ansi::parse_ansi_text;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // Parse style first so we can decide default speed for Matrix.
    let style = parse_style_argument(&args);
    let (mut speed, speed_set) = parse_speed_argument(&args);
    if style == AnimationStyle::Matrix && !speed_set {
        speed = 10.0; // Matrix default speed = 10 when not specified
    }
    let color_fps = parse_color_fps_argument(&args);
    let sysinfo = generate_system_info();
    show_animation_mode(&sysinfo, speed, style, color_fps)
}

fn show_animation_mode(
    _text: &[String],
    speed: f32,
    style: AnimationStyle,
    color_fps: f32,
) -> io::Result<()> {
    let freq = 0.1f32;
    let spread = 3.0f32;
    // Generate system info only once to avoid flicker from content changes.
    let lines = generate_system_info();
    let parsed: Vec<Vec<(String, char)>> = lines.iter().map(|l| parse_ansi_text(l)).collect();
    let start = Instant::now();
    let mut prev_widths: Vec<usize> = Vec::new();
    // Spark particle system for Fire mode (star-like popping embers)
    #[derive(Clone, Copy)]
    struct Spark { x: usize, y: usize, life: f32, age: f32, peak: f32, hue_jitter: f32 }
    let mut sparks: Vec<Spark> = Vec::new();
    let mut last_frame_time: f32 = 0.0;
    let target_fps = color_fps.clamp(5.0, 240.0); // reuse color_fps as desired smoothness
    let frame_interval = 1.0 / target_fps;
    print!("\x1b[?25l\x1b[H");
    stdout().flush()?;
    loop {
        let elapsed = start.elapsed().as_secs_f32() * speed.max(0.05);
        // Frame pacing for smooth continuous transitions
        let dt_since = elapsed - last_frame_time;
        if dt_since < frame_interval {
            let sleep_ms = ((frame_interval - dt_since) * 1000.0).clamp(1.0, 20.0) as u64;
            thread::sleep(Duration::from_millis(sleep_ms));
            continue;
        }
        last_frame_time = elapsed;
        let (tw, th) = size()?;
        let mut frame_buf = String::with_capacity(8192);
        let mut new_widths = Vec::with_capacity(lines.len());
        let max_lines = th as usize;
        let mut line_offset = elapsed;
        if style == AnimationStyle::Fire {
            // Update ages
            for sp in sparks.iter_mut() { sp.age += dt_since; }
            // Remove finished
            sparks.retain(|s| s.age < s.life);
            // Spawn new scattered embers (cap <10) with random rows (prefer upper hotter zone?)
            if sparks.len() < 10 {
                // Spawn probability scaled by gap
                let remaining = 10 - sparks.len();
                let spawn_prob = 0.25_f32 * remaining as f32 / 10.0; // up to 0.25 when empty
                if fastrand::f32() < spawn_prob {
                    let sx = fastrand::usize(..(tw as usize).max(1));
                    // Bias y toward middle/top of visible flames using quadratic distribution
                    let h = th as usize;
                    let r = fastrand::f32();
                    let by = (r * r * h as f32).min(h.saturating_sub(1) as f32) as usize; // more weight near top
                    let life = 0.18 + fastrand::f32() * 0.55; // short pop
                    let peak = 0.25 + fastrand::f32() * 0.45; // peak time fraction
                    let hue_jitter = fastrand::f32() * 80.0;
                    sparks.push(Spark { x: sx, y: by, life, age: 0.0, peak, hue_jitter });
                }
            }
        }
        for (li, row) in parsed.iter().take(max_lines).enumerate() {
            frame_buf.push_str(&format!("\x1b[{};1H", li + 1));
            let mut char_idx = 0f32;
            let mut printed = 0usize;
            for (ansi, ch) in row {
                if !ansi.is_empty() {
                    frame_buf.push_str(ansi);
                    continue;
                }
                if *ch == '\0' {
                    continue;
                }
                if printed >= tw as usize {
                    break;
                }
                let (mut r, mut g, mut b) = if style == AnimationStyle::Matrix {
                    calculate_matrix_color_at(elapsed, li, printed, th as usize)
                } else if style == AnimationStyle::Fire {
                    calculate_fire_color_at(elapsed, li, printed, th as usize, tw as usize)
                } else {
                    let ci = line_offset + char_idx / spread;
                    // stable id per cell for smoother hue (avoid flicker from ever-growing global counter)
                    let stable_id = li * tw as usize + printed;
                    calculate_color(&style, freq, ci, elapsed, stable_id)
                };
                // In Matrix style, hide non-trail characters (r=g=b=0 marker)
                if style == AnimationStyle::Matrix && r == 0 && g == 0 && b == 0 {
                    frame_buf.push(' ');
                    printed += 1;
                    char_idx += 1.0;
                    continue;
                }
                if style == AnimationStyle::Fire {
                    for sp in sparks.iter() {
                        if sp.x == printed && sp.y == li {
                            let t = (sp.age / sp.life).clamp(0.0, 1.0);
                            // Asymmetric envelope: rise to peak then fall
                            let up = (t / sp.peak).clamp(0.0, 1.0);
                            let down = ((t - sp.peak) / (1.0 - sp.peak)).clamp(0.0, 1.0);
                            let envelope = if t < sp.peak { up.powf(0.8) } else { (1.0 - down).powf(1.6) };
                            let flicker = 0.85 + (elapsed * 60.0 + (sp.x as f32 * 1.3)).sin() * 0.15;
                            let w = (envelope * flicker).clamp(0.0, 1.0);
                            // Color: hot core -> slight jitter orange/yellow -> hint of white
                            let hot_r = 255.0;
                            let hot_g = 160.0 + sp.hue_jitter.min(70.0);
                            let hot_b = (sp.hue_jitter * 0.9).min(120.0);
                            r = (r as f32 * (1.0 - w) + hot_r * w) as u8;
                            g = (g as f32 * (1.0 - w) + hot_g * w).min(255.0) as u8;
                            b = (b as f32 * (1.0 - w) + hot_b * w) as u8;
                            break;
                        }
                    }
                }
                frame_buf.push_str(&format!("\x1b[38;2;{};{};{}m{}", r, g, b, ch));
                printed += 1;
                char_idx += 1.0;
            }
            frame_buf.push_str("\x1b[0m");
            if let Some(pw) = prev_widths.get(li) {
                if *pw > printed {
                    frame_buf.push_str(&" ".repeat(pw - printed));
                }
            }
            new_widths.push(printed);
            line_offset += char_idx / spread;
        }
        // No dynamic line count changes now; skip clearing extra lines logic
        let mut out = stdout();
        out.write_all(frame_buf.as_bytes())?;
        out.flush()?;
        prev_widths = new_widths;
    }
}

fn parse_speed_argument(args: &[String]) -> (f32, bool) {
    for i in 0..args.len() {
        if args[i] == "--speed" || args[i] == "-s" {
            if i + 1 < args.len() {
                if let Ok(v) = args[i + 1].parse::<f32>() {
                    return (v.clamp(0.1, 20.0), true);
                }
            }
        } else if let Some(rest) = args[i].strip_prefix("--speed=") {
            if let Ok(v) = rest.parse::<f32>() {
                return (v.clamp(0.1, 20.0), true);
            }
        }
    }
    (1.0, false)
}
fn parse_style_argument(args: &[String]) -> AnimationStyle {
    for i in 0..args.len() {
        if args[i] == "--style" || args[i] == "--animation" {
            if i + 1 < args.len() {
                return AnimationStyle::from_str(&args[i + 1]);
            }
        } else if let Some(rest) = args[i].strip_prefix("--style=") {
            return AnimationStyle::from_str(rest);
        } else if let Some(rest) = args[i].strip_prefix("--animation=") {
            return AnimationStyle::from_str(rest);
        }
    }
    // Default style now Neon
    AnimationStyle::Neon
}

fn parse_color_fps_argument(args: &[String]) -> f32 {
    for i in 0..args.len() {
        if args[i] == "--color-fps" {
            if i + 1 < args.len() {
                if let Ok(v) = args[i + 1].parse::<f32>() {
                    return v.clamp(5.0, 120.0);
                }
            }
        } else if let Some(rest) = args[i].strip_prefix("--color-fps=") {
            if let Ok(v) = rest.parse::<f32>() {
                return v.clamp(5.0, 120.0);
            }
        }
    }
    30.0
}
