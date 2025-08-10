mod animation; // animations & color logic
mod system; // system information collection
mod util; // shared utilities (e.g. ANSI parsing)

use animation::{
    AnimationStyle, FireMode, calculate_color, calculate_fire_color_at, calculate_matrix_color_at,
};
use system::generate_system_info;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::size,
};
use std::{
    env,
    io::{self, Write, stdout},
    thread,
    time::{Duration, Instant},
};

use util::ansi::parse_ansi_text;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // Default behavior now: animated loop ON unless --fetch provided.
    // Flags:
    //   --fetch | -f   -> one-shot static output (disable loop)
    //   --loop  | -l   -> explicit (kept for backward compatibility)
    let fetch_mode = args.iter().any(|a| a == "--fetch" || a == "-f");
    let explicit_loop = args.iter().any(|a| a == "--loop" || a == "-l");
    let loop_mode = if fetch_mode { false } else { explicit_loop || true }; // default true
    let speed = parse_speed_argument(&args);
    let style = parse_style_argument(&args);
    let fire_mode = parse_fire_mode_argument(&args);
    let color_fps = parse_color_fps_argument(&args);
    let sysinfo = generate_system_info();
    if loop_mode {
        show_animation_mode(&sysinfo, speed, style, fire_mode, color_fps)
    } else {
        show_static_output(&sysinfo, style, fire_mode)
    }
}

fn show_animation_mode(
    _text: &[String],
    speed: f32,
    style: AnimationStyle,
    fire_mode: FireMode,
    color_fps: f32,
) -> io::Result<()> {
    let freq = 0.1f32;
    let spread = 3.0f32;
    // Generate system info only once to avoid flicker from content changes.
    let lines = generate_system_info();
    let parsed: Vec<Vec<(String, char)>> = lines.iter().map(|l| parse_ansi_text(l)).collect();
    let start = Instant::now();
    let mut prev_widths: Vec<usize> = Vec::new();
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
                let (r, g, b) = if style == AnimationStyle::Matrix {
                    calculate_matrix_color_at(elapsed, li, printed, th as usize)
                } else if style == AnimationStyle::Fire && fire_mode == FireMode::Advanced {
                    calculate_fire_color_at(elapsed, li, printed, th as usize)
                } else {
                    let ci = line_offset + char_idx / spread;
                    // stable id per cell for smoother hue (avoid flicker from ever-growing global counter)
                    let stable_id = li * tw as usize + printed;
                    calculate_color(&style, freq, ci, elapsed, stable_id)
                };
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

fn show_static_output(
    lines: &[String],
    style: AnimationStyle,
    fire_mode: FireMode,
) -> io::Result<()> {
    let freq = 0.1f32;
    let spread = 3.0f32;
    let mut gidx = 0f32;
    let mut gpos = 0usize;
    let (tw, th) = size()?;
    let mut line_no = 0usize;
    for line in lines {
        let parsed = parse_ansi_text(line);
        let visible = line.chars().filter(|c| !c.is_control()).count();
        let indent = if visible < tw as usize {
            (tw as usize - visible) / 2
        } else {
            0
        };
        if indent > 0 {
            print!("{}", " ".repeat(indent));
        }
        let mut printed = 0usize;
        for (ansi, ch) in parsed {
            if !ansi.is_empty() {
                print!("{}", ansi);
            } else if ch != '\0' {
                let (r, g, b) = if style == AnimationStyle::Matrix {
                    let sc = indent + printed;
                    calculate_matrix_color_at(0.0, line_no, sc, th as usize)
                } else if style == AnimationStyle::Fire {
                    if fire_mode == FireMode::Advanced {
                        let sc = indent + printed;
                        calculate_fire_color_at(0.0, line_no, sc, th as usize)
                    } else {
                        let ci = gidx / spread;
                        calculate_color(&style, freq, ci, 0.0, gpos)
                    }
                } else {
                    let ci = gidx / spread;
                    calculate_color(&style, freq, ci, 0.0, gpos)
                };
                execute!(
                    stdout(),
                    SetForegroundColor(Color::Rgb { r, g, b }),
                    Print(ch)
                )?;
                gidx += 1.0;
                gpos += 1;
                printed += 1;
            }
        }
        execute!(stdout(), ResetColor)?;
        println!();
        line_no += 1;
    }
    Ok(())
}

fn parse_speed_argument(args: &[String]) -> f32 {
    for i in 0..args.len() {
        if args[i] == "--speed" || args[i] == "-s" {
            if i + 1 < args.len() {
                if let Ok(v) = args[i + 1].parse::<f32>() {
                    return v.clamp(0.1, 10.0);
                }
            }
        } else if let Some(rest) = args[i].strip_prefix("--speed=") {
            if let Ok(v) = rest.parse::<f32>() {
                return v.clamp(0.1, 10.0);
            }
        }
    }
    1.0
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
fn parse_fire_mode_argument(args: &[String]) -> FireMode {
    for i in 0..args.len() {
        if args[i] == "--fire-mode" {
            if i + 1 < args.len() {
                return FireMode::from_str(&args[i + 1]);
            }
        } else if let Some(rest) = args[i].strip_prefix("--fire-mode=") {
            return FireMode::from_str(rest);
        }
    }
    FireMode::Advanced
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
