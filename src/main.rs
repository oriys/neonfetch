mod animation; // animations & color logic
mod system; // system information collection
mod util; // shared utilities (e.g. ANSI parsing)

use animation::{
    AnimationStyle, FallSim, calculate_aurora_color_at, calculate_color, calculate_fire_color_at,
    calculate_lava_color_at, calculate_marquee_color_at, calculate_matrix_color_at,
    calculate_meteor_color_at, calculate_plasma_color_at, calculate_pulse_rings_color_at,
};
use system::{
    INFO_FIELD_KEYS, InfoFieldSelection, SystemInfoOptions, generate_system_info,
    generate_system_info_json, info_field_key,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use std::{
    env,
    io::{self, IsTerminal, Write, stdout},
    thread,
    time::{Duration, Instant},
};

use util::ansi::parse_ansi_text;
use util::framebuf::FrameBuf;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        print_version();
        return Ok(());
    }
    if args.iter().any(|a| a == "--list-styles") {
        println!(
            "{}",
            animation::styles::AnimationStyle::available_styles().join("\n")
        );
        return Ok(());
    }
    if args.iter().any(|a| a == "--list-fields") {
        println!("{}", INFO_FIELD_KEYS.join("\n"));
        return Ok(());
    }
    let show_logo = !parse_no_logo_argument(&args);
    let field_selection = match parse_field_selection_argument(&args) {
        Ok(selection) => selection,
        Err(message) => {
            eprintln!("error: {}", message);
            std::process::exit(2);
        }
    };
    let info_options = SystemInfoOptions::new(show_logo, field_selection);
    let mono = parse_mono_argument(&args);
    let no_color = parse_no_color_argument(&args);
    let max_frames = if parse_frame_argument(&args) {
        Some(1usize)
    } else {
        None
    };
    if let Some(seed) = parse_seed_argument(&args) {
        fastrand::seed(seed);
    }
    // Auto fallback to one-shot in non-TTY pipelines
    let is_tty = stdout().is_terminal();
    if !is_tty && !parse_json_argument(&args) {
        let lines = generate_system_info(&info_options);
        for line in lines {
            println!("{}", line);
        }
        return Ok(());
    }
    if parse_json_argument(&args) {
        println!("{}", generate_system_info_json(&info_options));
        return Ok(());
    }
    if parse_fetch_argument(&args) {
        // One-shot system info output, no animation
        let lines = generate_system_info(&info_options);
        let mut out = stdout();
        for line in lines {
            writeln!(out, "{}", line)?;
        }
        return Ok(());
    }
    // Parse style first so we can decide default speed for Matrix.
    let style = parse_style_argument(&args);
    let (mut speed, speed_set) = parse_speed_argument(&args);
    if style == AnimationStyle::Matrix && !speed_set {
        speed = 10.0; // Matrix default speed = 10 when not specified
    }
    let color_fps = parse_color_fps_argument(&args);
    let duration = parse_duration_argument(&args);
    let sysinfo = generate_system_info(&info_options);
    let options = AnimationOptions {
        speed,
        style,
        color_fps,
        duration,
        mono,
        no_color,
        max_frames,
    };
    show_animation_mode(&sysinfo, options)
}

struct AnimationOptions {
    speed: f32,
    style: AnimationStyle,
    color_fps: f32,
    duration: Option<f32>,
    mono: bool,
    no_color: bool,
    max_frames: Option<usize>,
}

/// RAII guard: raw mode + hidden cursor on entry, always restored on exit
/// (including early `?` returns and panics, via Drop).
struct TermGuard {
    raw: bool,
}

impl TermGuard {
    fn new() -> Self {
        let raw = enable_raw_mode().is_ok();
        let mut out = stdout();
        let _ = out.write_all(b"\x1b[?25l\x1b[2J\x1b[H");
        let _ = out.flush();
        TermGuard { raw }
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        if self.raw {
            let _ = disable_raw_mode();
        }
        let mut out = stdout();
        let _ = out.write_all(b"\x1b[?25h\x1b[0m");
        let _ = out.flush();
    }
}

/// Restore the terminal before the default panic message prints, so a panic
/// mid-animation doesn't leave the shell with a hidden cursor in raw mode.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let mut out = io::stdout();
        let _ = out.write_all(b"\x1b[?25h\x1b[0m\r\n");
        let _ = out.flush();
        default_hook(info);
    }));
}

fn is_quit_key(k: &KeyEvent) -> bool {
    if k.kind != KeyEventKind::Press {
        return false;
    }
    matches!(
        k.code,
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc
    ) || (matches!(k.code, KeyCode::Char('c') | KeyCode::Char('C'))
        && k.modifiers.contains(KeyModifiers::CONTROL))
}

// Spark particle overlay for the Fire style.
#[derive(Clone, Copy)]
struct Spark {
    x: usize,
    y: usize,
    life: f32,
    age: f32,
    peak: f32,
    hue_jitter: f32,
}

struct GlitchBurst {
    start: f32,
    dur: f32,
}

type GlitchCell = (char, (u8, u8, u8));

fn show_animation_mode(lines: &[String], options: AnimationOptions) -> io::Result<()> {
    let AnimationOptions {
        speed,
        style,
        color_fps,
        duration,
        mono,
        no_color,
        max_frames,
    } = options;
    let parsed: Vec<Vec<(String, char)>> = lines.iter().map(|l| parse_ansi_text(l)).collect();
    // Plain printable-character grid (ANSI stripped) for styles that need
    // cell-level layout knowledge: Fall physics, Typing reveal, edge detection.
    let plain: Vec<Vec<char>> = parsed
        .iter()
        .map(|row| {
            row.iter()
                .filter(|(a, c)| a.is_empty() && *c != '\0')
                .map(|(_, c)| *c)
                .collect()
        })
        .collect();
    let edge_mask = (style == AnimationStyle::EdgeGlow).then(|| build_edge_mask(&plain));
    let total_chars: usize = plain.iter().map(|r| r.len()).sum();

    let speed = speed.max(0.05);
    // Frame pacing is wall-clock based: --speed accelerates the animation
    // clock, never the frame rate, so CPU cost stays flat at any speed.
    let target_fps = color_fps.clamp(5.0, 120.0);
    let frame_dt = Duration::from_secs_f32(1.0 / target_fps);

    install_panic_hook();
    let guard = TermGuard::new();

    let start = Instant::now();
    let mut next_frame = Instant::now();
    let mut last_anim_time = 0.0f32;
    let mut sparks: Vec<Spark> = Vec::new();
    let mut glitch_bursts: Vec<GlitchBurst> = Vec::new();
    let mut last_glitch_check = 0.0f32;
    let mut glitch_shift: Vec<i32> = Vec::new();
    let mut glitch_line: Vec<Option<GlitchCell>> = Vec::new();
    let mut fall = FallSim::new();
    let mut fb = FrameBuf::new(mono, no_color);
    let mut last_dims: (u16, u16) = (0, 0);
    let mut frames_rendered = 0usize;
    let mut rows_drawn = 0usize;

    loop {
        if duration.is_some_and(|d| start.elapsed().as_secs_f32() >= d) {
            break;
        }
        let now = Instant::now();
        if now < next_frame {
            // Sleep via the event queue so a quit key wakes us immediately.
            let wait = next_frame - now;
            if guard.raw {
                if event::poll(wait)?
                    && let Event::Key(k) = event::read()?
                    && is_quit_key(&k)
                {
                    break;
                }
            } else {
                thread::sleep(wait);
            }
            continue;
        }
        next_frame += frame_dt;
        if next_frame < now {
            next_frame = now + frame_dt; // dropped behind; don't try to catch up
        }

        let elapsed = start.elapsed().as_secs_f32() * speed; // animation clock
        let dt = (elapsed - last_anim_time).max(0.0);
        last_anim_time = elapsed;

        let (tw, th) = size()?;
        let (twu, thu) = (tw as usize, th as usize);
        if twu == 0 || thu == 0 {
            continue;
        }
        fb.begin();
        if (tw, th) != last_dims {
            if style == AnimationStyle::Fall {
                fall.resize(twu, thu, elapsed);
            }
            if last_dims != (0, 0) {
                fb.push_ansi("\x1b[2J"); // wipe stale cells after a resize
            }
            last_dims = (tw, th);
        }

        rows_drawn = match style {
            AnimationStyle::Fall => {
                fall.step(&plain, elapsed, dt);
                fall.render(&mut fb, elapsed);
                thu
            }
            AnimationStyle::Typing => {
                render_typing(&mut fb, &plain, elapsed, twu, thu, total_chars)
            }
            AnimationStyle::Glitch => render_glitch(
                &mut fb,
                &plain,
                elapsed,
                twu,
                thu,
                &mut glitch_bursts,
                &mut last_glitch_check,
                &mut glitch_shift,
                &mut glitch_line,
            ),
            _ => {
                if style == AnimationStyle::Fire {
                    update_sparks(&mut sparks, dt, twu, thu);
                }
                render_generic(
                    &mut fb,
                    &parsed,
                    &style,
                    elapsed,
                    twu,
                    thu,
                    speed,
                    &sparks,
                    edge_mask.as_deref(),
                )
            }
        };
        fb.write_to(&mut stdout())?;
        frames_rendered += 1;
        if max_frames.is_some_and(|limit| frames_rendered >= limit) {
            break;
        }
    }
    // Park the cursor below the rendered content, then let the guard restore
    // cursor visibility, colors and raw mode.
    let mut out = stdout();
    write!(out, "\x1b[{};1H", rows_drawn + 1)?;
    out.flush()?;
    drop(guard);
    println!();
    Ok(())
}

/// Per-cell color styles (everything except Fall / Typing / Glitch, which
/// need their own layout logic). Returns the number of rows drawn.
#[allow(clippy::too_many_arguments)]
fn render_generic(
    fb: &mut FrameBuf,
    parsed: &[Vec<(String, char)>],
    style: &AnimationStyle,
    elapsed: f32,
    tw: usize,
    th: usize,
    speed: f32,
    sparks: &[Spark],
    edge_mask: Option<&[Vec<bool>]>,
) -> usize {
    let mut rows = 0usize;
    for (li, row) in parsed.iter().take(th).enumerate() {
        fb.goto_line(li + 1);
        let mut printed = 0usize;
        for (ansi, ch) in row {
            if !ansi.is_empty() {
                fb.push_ansi(ansi);
                continue;
            }
            if *ch == '\0' {
                continue;
            }
            if printed >= tw {
                break;
            }
            let stable_id = li * tw + printed;
            let mut rgb = match style {
                AnimationStyle::Matrix => calculate_matrix_color_at(elapsed, li, printed, th),
                AnimationStyle::Fire => calculate_fire_color_at(elapsed, li, printed, th, tw),
                AnimationStyle::Plasma => {
                    calculate_plasma_color_at(elapsed, li, printed, tw, th, speed)
                }
                AnimationStyle::Aurora => {
                    calculate_aurora_color_at(elapsed, li, printed, tw, th, speed)
                }
                AnimationStyle::PulseRings => {
                    calculate_pulse_rings_color_at(elapsed, li, printed, tw, th, speed)
                }
                AnimationStyle::Lava => {
                    calculate_lava_color_at(elapsed, li, printed, tw, th, speed)
                }
                AnimationStyle::Marquee => calculate_marquee_color_at(elapsed, li, printed, tw),
                AnimationStyle::MeteorRain => {
                    calculate_meteor_color_at(elapsed, li, printed, tw, th)
                }
                // EdgeGlow rides on the Neon palette, adjusted below.
                AnimationStyle::EdgeGlow => {
                    calculate_color(&AnimationStyle::Neon, elapsed, stable_id)
                }
                _ => calculate_color(style, elapsed, stable_id),
            };
            // Matrix marks non-trail cells with pure black: hide them.
            if *style == AnimationStyle::Matrix && rgb == (0, 0, 0) {
                fb.put(' ', rgb);
                printed += 1;
                continue;
            }
            if *style == AnimationStyle::Fire {
                for sp in sparks {
                    if sp.x == printed && sp.y == li {
                        rgb = blend_spark(rgb, sp, elapsed);
                        break;
                    }
                }
            }
            if *style == AnimationStyle::EdgeGlow && *ch != ' ' {
                let edge = edge_mask
                    .and_then(|m| m.get(li))
                    .and_then(|r| r.get(printed))
                    .copied()
                    .unwrap_or(false);
                rgb = if edge {
                    // Outline cells: brighten and pull toward white.
                    let mix = 0.30f32;
                    (
                        (rgb.0 as f32 * (1.0 - mix) * 1.2 + 255.0 * mix).min(255.0) as u8,
                        (rgb.1 as f32 * (1.0 - mix) * 1.2 + 255.0 * mix).min(255.0) as u8,
                        (rgb.2 as f32 * (1.0 - mix) * 1.2 + 255.0 * mix).min(255.0) as u8,
                    )
                } else {
                    // Interior cells stay dim so the outline stands out.
                    (
                        (rgb.0 as f32 * 0.55) as u8,
                        (rgb.1 as f32 * 0.55) as u8,
                        (rgb.2 as f32 * 0.55) as u8,
                    )
                };
            }
            fb.put(*ch, rgb);
            printed += 1;
        }
        fb.end_line();
        rows = li + 1;
    }
    rows
}

fn blend_spark(base: (u8, u8, u8), sp: &Spark, elapsed: f32) -> (u8, u8, u8) {
    let t = (sp.age / sp.life).clamp(0.0, 1.0);
    // Asymmetric envelope: rise to peak then fall.
    let up = (t / sp.peak).clamp(0.0, 1.0);
    let down = ((t - sp.peak) / (1.0 - sp.peak)).clamp(0.0, 1.0);
    let envelope = if t < sp.peak {
        up.powf(0.8)
    } else {
        (1.0 - down).powf(1.6)
    };
    let flicker = 0.85 + (elapsed * 60.0 + sp.x as f32 * 1.3).sin() * 0.15;
    let w = (envelope * flicker).clamp(0.0, 1.0);
    let (hot_r, hot_g, hot_b) = (
        255.0,
        160.0 + sp.hue_jitter.min(70.0),
        (sp.hue_jitter * 0.9).min(120.0),
    );
    (
        (base.0 as f32 * (1.0 - w) + hot_r * w) as u8,
        (base.1 as f32 * (1.0 - w) + hot_g * w).min(255.0) as u8,
        (base.2 as f32 * (1.0 - w) + hot_b * w) as u8,
    )
}

fn update_sparks(sparks: &mut Vec<Spark>, dt: f32, tw: usize, th: usize) {
    for sp in sparks.iter_mut() {
        sp.age += dt;
    }
    sparks.retain(|s| s.age < s.life);
    // Spawn new scattered embers (cap 10), biased toward the hotter top zone.
    if sparks.len() < 10 {
        let remaining = 10 - sparks.len();
        let spawn_rate = 7.5_f32 * remaining as f32 / 10.0;
        let spawn_prob = (spawn_rate * dt).min(1.0);
        if fastrand::f32() < spawn_prob {
            let sx = fastrand::usize(..tw.max(1));
            let r = fastrand::f32();
            let by = ((r * r * th as f32) as usize).min(th.saturating_sub(1));
            sparks.push(Spark {
                x: sx,
                y: by,
                life: 0.18 + fastrand::f32() * 0.55,
                age: 0.0,
                peak: 0.25 + fastrand::f32() * 0.45,
                hue_jitter: fastrand::f32() * 80.0,
            });
        }
    }
}

/// Typewriter reveal: characters appear at a fixed rate on the animation
/// clock (so --speed scales it linearly), with a subtle hue drift.
fn render_typing(
    fb: &mut FrameBuf,
    plain: &[Vec<char>],
    elapsed: f32,
    tw: usize,
    th: usize,
    total_chars: usize,
) -> usize {
    let reveal_speed = 120.0f32; // chars per animation second
    let chars_to_show = ((elapsed * reveal_speed) as usize).min(total_chars);
    let mut shown = 0usize;
    let mut rows = 0usize;
    for (li, row) in plain.iter().take(th).enumerate() {
        fb.goto_line(li + 1);
        for (ci, &ch) in row.iter().take(tw).enumerate() {
            let visible = shown < chars_to_show;
            shown += 1;
            if visible {
                let hue = (elapsed * 35.0 + ci as f32 * 1.5 + li as f32 * 4.0) % 360.0;
                fb.put(ch, animation::styles::hsv_to_rgb(hue, 0.25, 0.92));
            } else {
                fb.put(' ', (0, 0, 0));
            }
        }
        fb.end_line();
        rows = li + 1;
    }
    rows
}

/// Digital glitch: intermittent bursts shift random columns sideways and gate
/// extra color distortion while the base rainbow hue keeps cycling.
#[allow(clippy::too_many_arguments)]
fn render_glitch(
    fb: &mut FrameBuf,
    plain: &[Vec<char>],
    elapsed: f32,
    tw: usize,
    th: usize,
    bursts: &mut Vec<GlitchBurst>,
    last_check: &mut f32,
    col_shift: &mut Vec<i32>,
    line_scratch: &mut Vec<Option<GlitchCell>>,
) -> usize {
    if elapsed - *last_check > 0.08 {
        *last_check = elapsed;
        if bursts.len() < 3 && fastrand::f32() < 0.25 {
            bursts.push(GlitchBurst {
                start: elapsed,
                dur: 0.12 + fastrand::f32() * 0.25,
            });
        }
        bursts.retain(|gb| elapsed < gb.start + gb.dur);
    }
    col_shift.clear();
    col_shift.resize(tw, 0);
    let mut burst_energy = 0.0f32;
    for gb in bursts.iter() {
        let energy = glitch_burst_energy(gb, elapsed);
        burst_energy += energy;
        let shifts = (energy * 6.0).ceil() as usize;
        for _ in 0..shifts {
            let col = fastrand::usize(..tw.max(1));
            let dir = if fastrand::bool() { 1 } else { -1 };
            col_shift[col] += dir * (1 + fastrand::i32(0..2));
        }
    }
    let distortion_energy = burst_energy;
    let mut rows = 0usize;
    for (li, row) in plain.iter().take(th).enumerate() {
        fb.goto_line(li + 1);
        line_scratch.clear();
        line_scratch.resize(tw, None);
        for (source_col, &ch) in row.iter().take(tw).enumerate() {
            let shift = col_shift.get(source_col).copied().unwrap_or(0);
            let dest = (source_col as i32 + shift).clamp(0, tw as i32 - 1) as usize;
            let base_hue = (elapsed * 120.0 + source_col as f32 * 3.0) % 360.0;
            let (mut r, mut g, mut b) = animation::styles::hsv_to_rgb(base_hue, 0.9, 0.85);
            if distortion_energy > 0.0 && fastrand::f32() < 0.08 * distortion_energy {
                std::mem::swap(&mut r, &mut g);
            }
            if distortion_energy > 0.0 && fastrand::f32() < 0.06 * distortion_energy {
                b = b.saturating_add(70);
            }
            line_scratch[dest] = Some((ch, (r, g, b)));
        }
        for &cell in line_scratch.iter() {
            match cell {
                Some((ch, rgb)) => fb.put(ch, rgb),
                None => fb.put(' ', (0, 0, 0)),
            }
        }
        fb.end_line();
        rows = li + 1;
    }
    rows
}

fn glitch_burst_energy(gb: &GlitchBurst, elapsed: f32) -> f32 {
    if elapsed < gb.start || elapsed >= gb.start + gb.dur {
        return 0.0;
    }
    let phase = ((elapsed - gb.start) / gb.dur).clamp(0.0, 1.0);
    if phase < 0.5 {
        (phase / 0.5).powf(0.6)
    } else {
        (1.0 - (phase - 0.5) / 0.5).powf(1.4)
    }
}

/// A cell is an "edge" if it is printable and touches a blank (or the text
/// boundary) on any of its four sides.
fn build_edge_mask(plain: &[Vec<char>]) -> Vec<Vec<bool>> {
    let blank = |r: isize, c: isize| -> bool {
        if r < 0 || c < 0 {
            return true;
        }
        plain
            .get(r as usize)
            .and_then(|row| row.get(c as usize))
            .is_none_or(|&ch| ch == ' ')
    };
    plain
        .iter()
        .enumerate()
        .map(|(ri, row)| {
            row.iter()
                .enumerate()
                .map(|(ci, &ch)| {
                    let (r, c) = (ri as isize, ci as isize);
                    ch != ' '
                        && (blank(r, c - 1)
                            || blank(r, c + 1)
                            || blank(r - 1, c)
                            || blank(r + 1, c))
                })
                .collect()
        })
        .collect()
}

fn parse_speed_argument(args: &[String]) -> (f32, bool) {
    for i in 0..args.len() {
        if (args[i] == "--speed" || args[i] == "-s")
            && i + 1 < args.len()
            && let Ok(v) = args[i + 1].parse::<f32>()
        {
            return (v.clamp(0.1, 20.0), true);
        } else if let Some(rest) = args[i].strip_prefix("--speed=")
            && let Ok(v) = rest.parse::<f32>()
        {
            return (v.clamp(0.1, 20.0), true);
        }
    }
    (1.0, false)
}
fn parse_style_argument(args: &[String]) -> AnimationStyle {
    for i in 0..args.len() {
        if args[i] == "--style" || args[i] == "--animation" {
            if i + 1 < args.len() {
                let s = &args[i + 1];
                if s.eq_ignore_ascii_case("random") || s.eq_ignore_ascii_case("rand") {
                    return pick_random_style();
                }
                return AnimationStyle::from_str(s);
            }
        } else if let Some(rest) = args[i].strip_prefix("--style=") {
            if rest.eq_ignore_ascii_case("random") || rest.eq_ignore_ascii_case("rand") {
                return pick_random_style();
            }
            return AnimationStyle::from_str(rest);
        } else if let Some(rest) = args[i].strip_prefix("--animation=") {
            if rest.eq_ignore_ascii_case("random") || rest.eq_ignore_ascii_case("rand") {
                return pick_random_style();
            }
            return AnimationStyle::from_str(rest);
        }
    }
    // Default style now Neon
    AnimationStyle::Neon
}

fn pick_random_style() -> AnimationStyle {
    use animation::styles::AnimationStyle as AS;
    let styles = [
        AS::Neon,
        AS::Wave,
        AS::Pulse,
        AS::Matrix,
        AS::Fire,
        AS::Fall,
        AS::Marquee,
        AS::Typing,
        AS::Plasma,
        AS::Glow,
        AS::Pixel,
        AS::Aurora,
        AS::Glitch,
        AS::PulseRings,
        AS::MeteorRain,
        AS::Lava,
        AS::EdgeGlow,
    ];
    let idx = fastrand::usize(..styles.len());
    styles[idx].clone()
}

fn parse_color_fps_argument(args: &[String]) -> f32 {
    for i in 0..args.len() {
        if args[i] == "--color-fps"
            && i + 1 < args.len()
            && let Ok(v) = args[i + 1].parse::<f32>()
        {
            return v.clamp(5.0, 120.0);
        } else if let Some(rest) = args[i].strip_prefix("--color-fps=")
            && let Ok(v) = rest.parse::<f32>()
        {
            return v.clamp(5.0, 120.0);
        }
    }
    30.0
}

fn parse_no_logo_argument(args: &[String]) -> bool {
    for arg in args {
        if arg == "--no-logo" || arg == "-L" {
            return true;
        }
    }
    false
}

fn parse_field_selection_argument(args: &[String]) -> Result<InfoFieldSelection, String> {
    let mut show_values = Vec::new();
    let mut hide_values = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--show" {
            let value = parse_required_value(args, i, "--show")?;
            show_values.push(value);
            i += 2;
            continue;
        }
        if let Some(rest) = arg.strip_prefix("--show=") {
            show_values.push(rest.to_string());
            i += 1;
            continue;
        }
        if arg == "--hide" {
            let value = parse_required_value(args, i, "--hide")?;
            hide_values.push(value);
            i += 2;
            continue;
        }
        if let Some(rest) = arg.strip_prefix("--hide=") {
            hide_values.push(rest.to_string());
            i += 1;
            continue;
        }
        if arg == "--no-packages" || arg == "--no-pkgs" || arg == "-P" {
            hide_values.push("packages".to_string());
        } else if arg == "--no-header" {
            hide_values.push("header".to_string());
        }
        i += 1;
    }

    if !show_values.is_empty() && !hide_values.is_empty() {
        return Err("--show and --hide cannot be used together".to_string());
    }
    if !show_values.is_empty() {
        return Ok(InfoFieldSelection::Show(parse_field_key_list(
            &show_values,
            "--show",
        )));
    }
    if !hide_values.is_empty() {
        return Ok(InfoFieldSelection::Hide(parse_field_key_list(
            &hide_values,
            "--hide",
        )));
    }
    Ok(InfoFieldSelection::All)
}

fn parse_required_value(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    let Some(value) = args.get(index + 1) else {
        return Err(format!("missing value for {}", flag));
    };
    if value.starts_with('-') {
        return Err(format!("missing value for {}", flag));
    }
    Ok(value.clone())
}

fn parse_field_key_list(values: &[String], flag: &str) -> Vec<&'static str> {
    let mut keys = Vec::new();
    for value in values {
        for raw_key in value.split(',') {
            let trimmed = raw_key.trim();
            if trimmed.is_empty() {
                continue;
            }
            let normalized = trimmed.to_ascii_lowercase();
            if let Some(key) = info_field_key(&normalized) {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            } else {
                eprintln!(
                    "warning: unknown info field '{}' in {}; ignoring",
                    trimmed, flag
                );
            }
        }
    }
    keys
}

fn parse_fetch_argument(args: &[String]) -> bool {
    args.iter().any(|a| a == "--fetch")
}

fn parse_json_argument(args: &[String]) -> bool {
    args.iter().any(|a| a == "--json")
}

fn parse_no_color_argument(args: &[String]) -> bool {
    args.iter().any(|a| a == "--no-color" || a == "-C")
}

fn parse_mono_argument(args: &[String]) -> bool {
    args.iter().any(|a| a == "--mono")
}

fn parse_frame_argument(args: &[String]) -> bool {
    args.iter().any(|a| a == "--frame")
}

fn parse_duration_argument(args: &[String]) -> Option<f32> {
    for i in 0..args.len() {
        if args[i] == "--duration"
            && i + 1 < args.len()
            && let Ok(v) = args[i + 1].parse::<f32>()
            && v > 0.0
        {
            return Some(v);
        } else if let Some(rest) = args[i].strip_prefix("--duration=")
            && let Ok(v) = rest.parse::<f32>()
            && v > 0.0
        {
            return Some(v);
        }
    }
    None
}

fn parse_seed_argument(args: &[String]) -> Option<u64> {
    for i in 0..args.len() {
        if args[i] == "--seed"
            && i + 1 < args.len()
            && let Ok(v) = args[i + 1].parse::<u64>()
        {
            return Some(v);
        } else if let Some(rest) = args[i].strip_prefix("--seed=")
            && let Ok(v) = rest.parse::<u64>()
        {
            return Some(v);
        }
    }
    None
}

fn print_help() {
    let styles = animation::styles::AnimationStyle::available_styles().join(", ");
    println!(
        "neonfetch - fast colorful animated system info\n\nUsage:\n  neonfetch [options]\n\nOptions:\n  --style <name>        Animation style (default: neon; or 'random')\n  --speed <val>         Animation speed (0.1-20.0, default 1.0)\n  --color-fps <val>     Color refresh FPS (5-120, default 30)\n  --duration <sec>      Auto-exit after N seconds (animation mode)\n  --frame               Render one frame and exit (animation mode)\n  --fetch               Print info once and exit\n  --json                Print keyed JSON object and exit\n  --show <keys>         Show only comma-separated info fields in that order\n  --hide <keys>         Hide comma-separated info fields\n  --list-fields         List available info field keys\n  --mono                Render in grayscale (animations/info)\n  --no-color, -C        Disable ANSI colors (plain text)\n  --no-logo, -L         Hide ASCII logo\n  --no-packages, -P     Hide packages field and skip package detection\n  --no-header           Hide username@hostname header divider\n  --seed <u64>          Deterministic random seed for animations\n  --list-styles         List available styles\n  -h, --help            Show this help\n  -V, --version         Show version\n\nInfo fields:\n  {}\n\nKeys (animation mode):\n  q / Esc / Ctrl+C      Quit and restore the terminal\n\nStyles:\n  {}",
        INFO_FIELD_KEYS.join(", "),
        styles
    );
}

fn print_version() {
    println!("neonfetch {}", env!("CARGO_PKG_VERSION"));
}
