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
    // For Fall style we keep original layout; we'll build falling letters later from parsed grid.
    let start = Instant::now();
    let mut prev_widths: Vec<usize> = Vec::new();
    // Spark particle system for Fire mode (star-like popping embers)
    #[derive(Clone, Copy)]
    struct Spark { x: usize, y: usize, life: f32, age: f32, peak: f32, hue_jitter: f32 }
    let mut sparks: Vec<Spark> = Vec::new();
    let mut last_frame_time: f32 = 0.0;
    let target_fps = color_fps.clamp(5.0, 240.0); // reuse color_fps as desired smoothness
    let frame_interval = 1.0 / target_fps;
    // --- Fall style simulation state ---
    #[derive(Clone, Copy)]
    struct FallingLetter {
        ch: char,
        orig_row: usize,
        orig_col: usize,
        xf: f32,
        y: f32,
        vx: f32,
        vy: f32,
        release: f32, // time offset when it starts to fall
    }
    #[derive(Clone, Copy)]
    struct SettledUnit { ch: char, tilt: f32 }
    #[derive(PartialEq)]
    enum FallPhase { Static, Falling, Settled }
    let mut fall_letters: Vec<FallingLetter> = Vec::new(); // active & not yet settled
    let mut fall_pile: Vec<Vec<SettledUnit>> = Vec::new(); // per column bottom-up (settled chars with tilt)
    let mut fall_phase = FallPhase::Static;
    let mut fall_phase_start: f32 = 0.0; // elapsed when entered current phase
    let mut last_dims: (u16,u16) = (0,0);
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
        // Resize handling for Fall
        if style == AnimationStyle::Fall && (tw, th) != last_dims {
            // Reset simulation on resize
            fall_pile = vec![Vec::new(); tw as usize];
            fall_letters.clear();
            fall_phase = FallPhase::Static;
            fall_phase_start = elapsed;
            last_dims = (tw, th);
        }
        let mut frame_buf = String::with_capacity(8192);
        let mut new_widths = Vec::with_capacity(lines.len());
        let max_lines = th as usize;
        let mut line_offset = elapsed;
        if style == AnimationStyle::Fall {
            let term_h = th as usize;
            let term_w = tw as usize;
            if fall_pile.is_empty() { fall_pile = vec![Vec::new(); term_w]; }
            // Initialize letters (once) from parsed grid when entering Static
            if fall_phase == FallPhase::Static && fall_letters.is_empty() {
                let total_rows = parsed.len().min(term_h);
                for (row_i, row) in parsed.iter().enumerate().take(term_h) {
                    let mut col_i = 0usize;
                    for (ansi, ch) in row {
                        if !ansi.is_empty() { continue; }
                        if *ch == '\0' { continue; }
                        if col_i >= term_w { break; }
                        // Release spread (bottom-first): lower rows (larger row_i) start earlier
                        // so row at bottom gets the smallest delay.
                        let inv = (total_rows.saturating_sub(1)).saturating_sub(row_i) as f32; // top has largest inv
                        let base_delay = 0.5 + inv * 0.035; // top delayed more
                        let jitter = fastrand::f32() * 0.4; // smaller jitter to keep ordering clear
                        let release = base_delay + jitter;
                        // small initial horizontal drift
                        let vx = (fastrand::f32() - 0.5) * 3.0; // -1.5..1.5
                        fall_letters.push(FallingLetter {
                            ch: *ch,
                            orig_row: row_i,
                            orig_col: col_i,
                            xf: col_i as f32,
                            y: row_i as f32,
                            vx,
                            vy: 0.0,
                            release,
                        });
                        col_i += 1;
                    }
                }
                fall_phase_start = elapsed;
            }
            // Phase transition trigger: start releasing when first release time reached
            if fall_phase == FallPhase::Static {
                if fall_letters.iter().any(|fl| elapsed >= fall_phase_start + fl.release) {
                    fall_phase = FallPhase::Falling;
                }
            }
            let gravity = 55.0 * speed.max(0.05);
            if fall_phase == FallPhase::Falling {
                // Update only released letters
                for fl in fall_letters.iter_mut() {
                    if elapsed < fall_phase_start + fl.release { continue; }
                    // horizontal drift with mild random perturbation & friction
                    fl.vx += (fastrand::f32() - 0.5) * 0.6 * dt_since; // small random accel
                    fl.vx *= 0.985_f32.powf(dt_since * 60.0); // friction
                    fl.xf += fl.vx * dt_since;
                    if fl.xf < 0.0 { fl.xf = 0.0; fl.vx = -fl.vx * 0.3; }
                    if fl.xf > (term_w as f32 - 1.0) { fl.xf = term_w as f32 - 1.0; fl.vx = -fl.vx * 0.3; }
                    fl.vy += gravity * dt_since;
                    fl.y += fl.vy * dt_since;
                    let col_index = fl.xf.round().clamp(0.0, term_w as f32 - 1.0) as usize;
                    let col_height = fall_pile[col_index].len();
                    let ground_y = (term_h as f32 - 1.0 - col_height as f32).max(0.0);
                    if fl.y >= ground_y {
                        // bounce or settle
                        if fl.vy > 18.0 {
                            fl.y = ground_y;
                            fl.vy = -fl.vy * 0.25; // restitution
                            // slight horizontal energy loss
                            fl.vx *= 0.6;
                        } else {
                            fl.y = ground_y;
                            fl.vy = 0.0;
                            // settle
                            // Assign tilt influenced by "center of mass" style: lean toward larger drop-off.
                            // We evaluate prospective height after adding this unit (col_height + 1) vs neighbors.
                            let prospective_height = col_height + 1;
                            let lh = if col_index>0 { fall_pile[col_index-1].len() } else { prospective_height };
                            let rh = if col_index+1 < fall_pile.len() { fall_pile[col_index+1].len() } else { prospective_height };
                            let diff_left = (prospective_height as isize - lh as isize).max(0) as f32; // how much higher this column is vs left
                            let diff_right = (prospective_height as isize - rh as isize).max(0) as f32; // vs right
                            // Determine direction with larger unsupported side (bigger positive diff)
                            let mut tilt_dir: f32 = 0.0;
                            let mut tilt_mag: f32 = 0.0;
                            if diff_left > 0.0 || diff_right > 0.0 {
                                if (diff_left - diff_right).abs() < 0.1 {
                                    // similar differences, random choice
                                    if diff_left > 0.0 || diff_right > 0.0 {
                                        tilt_dir = if fastrand::f32() < 0.5 { -1.0 } else { 1.0 };
                                        tilt_mag = diff_left.max(diff_right);
                                    }
                                } else if diff_left > diff_right { // lean toward lower left side
                                    tilt_dir = -1.0; tilt_mag = diff_left;
                                } else { tilt_dir = 1.0; tilt_mag = diff_right; }
                            }
                            if fastrand::f32() < 0.25 { tilt_dir = 0.0; tilt_mag = 0.0; } // some remain vertical
                            // Scale magnitude into a reasonable range (~0.3..1.4)
                            if tilt_mag > 0.0 {
                                // Non-linear compression to avoid huge values dominating
                                tilt_mag = (tilt_mag / 2.5).min(1.4);
                                // Add slight randomness (±15%)
                                tilt_mag *= 0.85 + fastrand::f32()*0.3;
                            }
                            let tilt_val = tilt_dir * tilt_mag;
                            // Jitter parameters: phase random, amplitude proportional to |tilt| (so vertical ones barely move)
                            fall_pile[col_index].push(SettledUnit { ch: fl.ch, tilt: tilt_val });
                            // mark as to-remove by setting release far future
                            fl.release = f32::INFINITY;
                        }
                    }
                }
                // Remove settled letters
                fall_letters.retain(|fl| fl.release.is_finite());
                if fall_letters.is_empty() { fall_phase = FallPhase::Settled; fall_phase_start = elapsed; }
            } else if fall_phase == FallPhase::Settled {
                if elapsed - fall_phase_start > 4.0 {
                    // restart cycle
                    fall_pile.iter_mut().for_each(|v| v.clear());
                    fall_letters.clear();
                    fall_phase = FallPhase::Static;
                    fall_phase_start = elapsed;
                }
            }
            // Precompute support-aware settled pile placement to avoid floating letters
            let mut settled_grid: Vec<Vec<Option<char>>> = vec![vec![None; term_w]; term_h];
            {
                let bottom = term_h - 1;
                // For each column, place bottom-up enforcing support
                for col in 0..term_w {
                    let stack = &fall_pile[col];
                    if stack.is_empty() { continue; }
                    let h = stack.len();
                    for level in 0..h { // level 0 = bottom
                        let su = stack[level];
                        let row = bottom - level; // target row
                        if row >= term_h { continue; }
                        let rel = level as f32 / h as f32; // 0 bottom -> 1 top
                        let max_shift = 1.3;
                        let raw_shift = su.tilt * rel * max_shift;
                        let mut step = raw_shift.abs().floor() * raw_shift.signum();
                        // Limit shift to 2 cells for stability
                        if step > 2.0 { step = 2.0; } else if step < -2.0 { step = -2.0; }
                        let mut target_col = (col as isize + step as isize).clamp(0, term_w as isize - 1);
                        // Enforce support: if not bottom level ensure cell directly below is occupied; otherwise reduce shift magnitude toward 0
                        if level > 0 {
                            while target_col != col as isize && settled_grid[row + 1][target_col as usize].is_none() {
                                if target_col > col as isize { target_col -= 1; } else { target_col += 1; }
                            }
                        }
                        // Collision handling: if target already occupied at this row, fallback to original column if free
                        if settled_grid[row][target_col as usize].is_some() {
                            if settled_grid[row][col].is_none() {
                                target_col = col as isize;
                            } else {
                                // search outward 1 cell each side for free spot (rare)
                                let mut found = false;
                                for d in [1isize, -1] {
                                    let cand = target_col + d;
                                    if cand >= 0 && cand < term_w as isize && settled_grid[row][cand as usize].is_none() {
                                        target_col = cand; found = true; break;
                                    }
                                }
                                if !found { /* leave overlap (will be overwritten) */ }
                            }
                        }
                        settled_grid[row][target_col as usize] = Some(su.ch);
                    }
                }
            }
            // Rendering
            for row in 0..term_h.min(max_lines) {
                frame_buf.push_str(&format!("\x1b[{};1H", row + 1));
                // During Static phase we selectively keep letters not yet released.
                // We'll render by constructing a per-cell char from (unreleased letters) / active falling / pile.
                // Build occupancy map for current frame
                let mut active_map: Vec<Option<char>> = vec![None; term_w];
                for fl in fall_letters.iter() {
                    if fall_phase == FallPhase::Static && elapsed < fall_phase_start + fl.release { continue; }
                    let ry = fl.y.round() as isize;
                    let cx = fl.xf.round().clamp(0.0,term_w as f32-1.0) as usize;
                    if ry == row as isize { active_map[cx] = Some(fl.ch); }
                }
                // Use precomputed settled grid for this row
                let pile_row: &Vec<Option<char>> = &settled_grid[row];
                for col in 0..term_w {
                    let mut printed_char: char = pile_row[col].unwrap_or(' ');
                    let mut r=0u8; let mut g=0u8; let mut b=0u8;
                    if printed_char != ' ' { r=200; g=200; b=200; }
                    if fall_phase == FallPhase::Static {
                        if printed_char == ' ' {
                            if let Some(fl) = fall_letters.iter().find(|fl| fl.orig_row==row && fl.orig_col==col) {
                                if elapsed < fall_phase_start + fl.release { printed_char=fl.ch; r=200; g=200; b=200; }
                            }
                        }
                    } else if fall_phase == FallPhase::Falling {
                        if printed_char == ' ' {
                            if let Some(fl)= fall_letters.iter().find(|fl| fl.orig_row==row && fl.orig_col==col && elapsed < fall_phase_start + fl.release) {
                                printed_char=fl.ch; r=120; g=120; b=120;
                            }
                        }
                    }
                    if let Some(ch) = &active_map[col] { printed_char=*ch; r=220; g=220; b=220; }
                    if printed_char == ' ' { frame_buf.push(' ');} else { frame_buf.push_str(&format!("\x1b[38;2;{};{};{}m{}", r,g,b, printed_char)); }
                }
                frame_buf.push_str("\x1b[0m");
                new_widths.push(term_w);
            }
            let mut out = stdout(); out.write_all(frame_buf.as_bytes())?; out.flush()?; prev_widths = new_widths; continue;
        }
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
