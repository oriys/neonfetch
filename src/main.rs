mod animation; // animations & color logic
mod config; // config file loading and parsing
mod system; // system information collection
mod util; // shared utilities (e.g. ANSI parsing)

use animation::{
    AnimationStyle, FallSim, Palette, calculate_aurora_color_with_palette,
    calculate_color_with_palette, calculate_fire_color_with_palette,
    calculate_lava_color_with_palette, calculate_marquee_color_with_palette,
    calculate_matrix_color_with_palette, calculate_meteor_color_with_palette,
    calculate_plasma_color_with_palette, calculate_pulse_rings_color_with_palette,
};
use config::Config;
use system::{
    INFO_FIELD_KEYS, InfoFieldSelection, SystemInfoOptions, generate_system_info,
    generate_system_info_json, info_field_key,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use std::{
    env, fs,
    io::{self, IsTerminal, Write, stdout},
    process::Command,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use util::ansi::parse_ansi_text;
use util::framebuf::FrameBuf;

const MAX_LOGO_LINES: usize = 60;
const MAX_LOGO_COLUMNS: usize = 120;
const TAB_WIDTH: usize = 4;
const SHOWCASE_STYLE_POOL: &[AnimationStyle] = &[
    AnimationStyle::Neon,
    AnimationStyle::Wave,
    AnimationStyle::Pulse,
    AnimationStyle::Matrix,
    AnimationStyle::Fire,
    AnimationStyle::Fall,
    AnimationStyle::Marquee,
    AnimationStyle::Typing,
    AnimationStyle::Plasma,
    AnimationStyle::Glow,
    AnimationStyle::Aurora,
    AnimationStyle::PulseRings,
    AnimationStyle::MeteorRain,
    AnimationStyle::Lava,
    AnimationStyle::EdgeGlow,
];

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
        print_style_list();
        return Ok(());
    }
    if args.iter().any(|a| a == "--list-palettes") {
        println!("{}", animation::available_palette_names().join("\n"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--list-fields") {
        println!("{}", INFO_FIELD_KEYS.join("\n"));
        return Ok(());
    }
    let config_path = parse_config_path_argument(&args);
    let config = Config::load(config_path.as_deref(), parse_no_config_argument(&args));
    let seed = parse_seed_argument(&args, &config);
    if let Some(seed) = seed {
        fastrand::seed(seed);
    }
    // Parse style first so we can decide default speed for Matrix.
    let style = parse_style_argument(&args, &config, seed);
    let (mut speed, speed_set) = parse_speed_argument(&args, &config);
    if style == AnimationStyle::Matrix && !speed_set {
        speed = 10.0; // Matrix default speed = 10 when not specified
    }
    let effective_config = EffectiveConfig {
        speed,
        style,
        color_fps: parse_color_fps_argument(&args, &config),
        duration: parse_duration_argument(&args, &config),
        no_logo: parse_no_logo_argument(&args, &config),
        no_packages: parse_no_packages_argument(&args, &config),
        no_header: parse_no_header_argument(&args, &config),
        mono: parse_mono_argument(&args, &config),
        no_color: parse_no_color_argument(&args, &config),
        seed,
        palette: parse_palette_argument(&args),
    };
    if parse_print_config_argument(&args) {
        print_effective_config(&effective_config);
        return Ok(());
    }
    let show_logo = !effective_config.no_logo;
    let show_packages = !effective_config.no_packages;
    let show_header = !effective_config.no_header;
    let logo_override = if show_logo {
        parse_logo_file_argument(&args).and_then(|path| match load_logo_file(&path) {
            Ok(lines) => Some(lines),
            Err(err) => {
                eprintln!(
                    "warning: could not read logo file '{}': {}; using built-in logo",
                    path, err
                );
                None
            }
        })
    } else {
        None
    };
    let distro_id = parse_distro_argument(&args);
    let field_selection = match parse_field_selection_argument(&args, show_packages, show_header) {
        Ok(selection) => selection,
        Err(message) => {
            eprintln!("error: {}", message);
            std::process::exit(2);
        }
    };
    let info_options = SystemInfoOptions::new(show_logo, field_selection)
        .with_logo_override(logo_override)
        .with_distro_id(distro_id);
    let max_frames = if parse_frame_argument(&args) {
        Some(1usize)
    } else {
        None
    };
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
    let sysinfo = generate_system_info(&info_options);
    let options = AnimationOptions {
        speed: effective_config.speed,
        style: effective_config.style,
        color_fps: effective_config.color_fps,
        duration: effective_config.duration,
        mono: effective_config.mono,
        no_color: effective_config.no_color,
        max_frames,
        palette: effective_config.palette,
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
    palette: &'static Palette,
}

struct EffectiveConfig {
    speed: f32,
    style: AnimationStyle,
    color_fps: f32,
    duration: Option<f32>,
    no_logo: bool,
    no_packages: bool,
    no_header: bool,
    mono: bool,
    no_color: bool,
    seed: Option<u64>,
    palette: &'static Palette,
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
        palette,
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
                fall.render(&mut fb, elapsed, palette);
                thu
            }
            AnimationStyle::Typing => {
                render_typing(&mut fb, &plain, elapsed, twu, thu, total_chars, palette)
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
                palette,
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
                    palette,
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
    palette: &Palette,
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
                AnimationStyle::Matrix => {
                    calculate_matrix_color_with_palette(elapsed, li, printed, th, palette)
                }
                AnimationStyle::Fire => {
                    calculate_fire_color_with_palette(elapsed, li, printed, th, tw, palette)
                }
                AnimationStyle::Plasma => calculate_plasma_color_with_palette(
                    elapsed, li, printed, tw, th, speed, palette,
                ),
                AnimationStyle::Aurora => calculate_aurora_color_with_palette(
                    elapsed, li, printed, tw, th, speed, palette,
                ),
                AnimationStyle::PulseRings => calculate_pulse_rings_color_with_palette(
                    elapsed, li, printed, tw, th, speed, palette,
                ),
                AnimationStyle::Lava => {
                    calculate_lava_color_with_palette(elapsed, li, printed, tw, th, speed, palette)
                }
                AnimationStyle::Marquee => {
                    calculate_marquee_color_with_palette(elapsed, li, printed, tw, palette)
                }
                AnimationStyle::MeteorRain => {
                    calculate_meteor_color_with_palette(elapsed, li, printed, tw, th, palette)
                }
                // EdgeGlow rides on the Neon palette, adjusted below.
                AnimationStyle::EdgeGlow => {
                    calculate_color_with_palette(&AnimationStyle::Neon, elapsed, stable_id, palette)
                }
                _ => calculate_color_with_palette(style, elapsed, stable_id, palette),
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
                        rgb = blend_spark(rgb, sp, elapsed, palette);
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

fn blend_spark(base: (u8, u8, u8), sp: &Spark, elapsed: f32, palette: &Palette) -> (u8, u8, u8) {
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
    let (hot_r, hot_g, hot_b) = if palette.is_default() {
        (
            255.0,
            160.0 + sp.hue_jitter.min(70.0),
            (sp.hue_jitter * 0.9).min(120.0),
        )
    } else {
        let rgb = palette.sample_tinted(sp.hue_jitter / 80.0 + elapsed * 0.06, 0.80, 1.0);
        (rgb.0 as f32, rgb.1 as f32, rgb.2 as f32)
    };
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
    palette: &Palette,
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
                let rgb = if palette.is_default() {
                    animation::styles::hsv_to_rgb(hue, 0.25, 0.92)
                } else {
                    palette.sample_tinted(hue / 360.0, 0.25, 0.92)
                };
                fb.put(ch, rgb);
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
    palette: &Palette,
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
            let (mut r, mut g, mut b) = if palette.is_default() {
                animation::styles::hsv_to_rgb(base_hue, 0.9, 0.85)
            } else {
                palette.sample_tinted(base_hue / 360.0, 0.9, 0.85)
            };
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

fn parse_config_path_argument(args: &[String]) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == "--config" {
            if i + 1 < args.len() {
                return Some(args[i + 1].clone());
            }
        } else if let Some(rest) = args[i].strip_prefix("--config=") {
            return Some(rest.to_string());
        }
    }
    None
}

fn parse_no_config_argument(args: &[String]) -> bool {
    args.iter().any(|a| a == "--no-config")
}

fn parse_print_config_argument(args: &[String]) -> bool {
    args.iter().any(|a| a == "--print-config")
}

fn parse_speed_argument(args: &[String], config: &Config) -> (f32, bool) {
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
    if let Some(value) = config.speed {
        return (value.clamp(0.1, 20.0), true);
    }
    (1.0, false)
}
fn parse_style_argument(args: &[String], config: &Config, seed: Option<u64>) -> AnimationStyle {
    for i in 0..args.len() {
        if args[i] == "--style" || args[i] == "--animation" {
            if i + 1 < args.len() {
                return parse_style_value(&args[i + 1], seed);
            }
        } else if let Some(rest) = args[i].strip_prefix("--style=") {
            return parse_style_value(rest, seed);
        } else if let Some(rest) = args[i].strip_prefix("--animation=") {
            return parse_style_value(rest, seed);
        }
    }
    if let Some(style) = &config.style {
        return parse_style_value(style, seed);
    }
    AnimationStyle::Neon
}

fn parse_style_value(value: &str, seed: Option<u64>) -> AnimationStyle {
    if value.eq_ignore_ascii_case("random") || value.eq_ignore_ascii_case("rand") {
        return pick_random_style(seed);
    }
    if value.eq_ignore_ascii_case("daily") {
        return pick_daily_style_for_date(current_local_yyyymmdd());
    }
    AnimationStyle::from_str(value)
}

fn parse_palette_argument(args: &[String]) -> &'static Palette {
    for i in 0..args.len() {
        if args[i] == "--palette" {
            if i + 1 < args.len() {
                return resolve_palette_argument(&args[i + 1]);
            }
            eprintln!("warning: --palette requires a name; using default palette");
            return animation::default_palette();
        } else if let Some(rest) = args[i].strip_prefix("--palette=") {
            return resolve_palette_argument(rest);
        }
    }
    animation::default_palette()
}

fn resolve_palette_argument(name: &str) -> &'static Palette {
    if let Some(palette) = animation::find_palette(name) {
        palette
    } else {
        eprintln!("warning: unknown palette '{}'; using default palette", name);
        animation::palette::palette_or_default(name)
    }
}

fn pick_random_style(seed: Option<u64>) -> AnimationStyle {
    let styles = SHOWCASE_STYLE_POOL;
    let idx = if let Some(seed) = seed {
        let mut rng = fastrand::Rng::with_seed(seed);
        rng.usize(..styles.len())
    } else {
        fastrand::usize(..styles.len())
    };
    styles[idx]
}

fn pick_daily_style_for_date(yyyymmdd: u32) -> AnimationStyle {
    let styles = SHOWCASE_STYLE_POOL;
    let idx = daily_style_index(yyyymmdd, styles.len());
    styles[idx]
}

fn daily_style_index(yyyymmdd: u32, len: usize) -> usize {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in yyyymmdd.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (hash as usize) % len
}

fn current_local_yyyymmdd() -> u32 {
    if let Ok(output) = Command::new("date").arg("+%Y%m%d").output()
        && output.status.success()
        && let Ok(text) = std::str::from_utf8(&output.stdout)
        && let Ok(date) = text.trim().parse::<u32>()
    {
        return date;
    }
    current_utc_yyyymmdd()
}

fn current_utc_yyyymmdd() -> u32 {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400;
    yyyymmdd_from_unix_days(days as i64)
}

fn yyyymmdd_from_unix_days(days: i64) -> u32 {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (year * 10_000 + month * 100 + day) as u32
}

fn parse_color_fps_argument(args: &[String], config: &Config) -> f32 {
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
    if let Some(value) = config.color_fps {
        return (value as f32).clamp(5.0, 120.0);
    }
    30.0
}

fn parse_no_logo_argument(args: &[String], config: &Config) -> bool {
    for arg in args {
        if arg == "--no-logo" || arg == "-L" {
            return true;
        }
    }
    config.no_logo.unwrap_or(false)
}

fn parse_field_selection_argument(
    args: &[String],
    show_packages: bool,
    show_header: bool,
) -> Result<InfoFieldSelection, String> {
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

    if show_values.is_empty() {
        if !show_packages {
            hide_values.push("packages".to_string());
        }
        if !show_header {
            hide_values.push("header".to_string());
        }
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

fn parse_no_color_argument(args: &[String], config: &Config) -> bool {
    args.iter().any(|a| a == "--no-color" || a == "-C") || config.no_color.unwrap_or(false)
}

fn parse_mono_argument(args: &[String], config: &Config) -> bool {
    args.iter().any(|a| a == "--mono") || config.mono.unwrap_or(false)
}

fn parse_frame_argument(args: &[String]) -> bool {
    args.iter().any(|a| a == "--frame")
}

fn parse_duration_argument(args: &[String], config: &Config) -> Option<f32> {
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
    if let Some(value) = config.duration
        && value > 0.0
        && value <= f32::MAX as f64
    {
        return Some(value as f32);
    }
    None
}

fn parse_no_packages_argument(args: &[String], config: &Config) -> bool {
    for arg in args {
        if arg == "--no-packages" || arg == "--no-pkgs" || arg == "-P" {
            return true;
        }
    }
    config.no_packages.unwrap_or(false)
}

fn parse_no_header_argument(args: &[String], config: &Config) -> bool {
    args.iter().any(|a| a == "--no-header") || config.no_header.unwrap_or(false)
}

fn parse_logo_file_argument(args: &[String]) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == "--logo-file" {
            if i + 1 < args.len() {
                return Some(args[i + 1].clone());
            }
        } else if let Some(rest) = args[i].strip_prefix("--logo-file=") {
            return Some(rest.to_string());
        }
    }
    None
}

fn parse_distro_argument(args: &[String]) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == "--distro" {
            if i + 1 < args.len() {
                let value = args[i + 1].trim();
                if !value.is_empty() {
                    return Some(value.to_ascii_lowercase());
                }
            }
        } else if let Some(rest) = args[i].strip_prefix("--distro=") {
            let value = rest.trim();
            if !value.is_empty() {
                return Some(value.to_ascii_lowercase());
            }
        }
    }
    None
}

fn parse_seed_argument(args: &[String], config: &Config) -> Option<u64> {
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
    config.seed
}

fn print_effective_config(config: &EffectiveConfig) {
    println!("style = \"{}\"", animation_style_name(&config.style));
    println!("speed = {}", format_float(config.speed));
    if let Some(duration) = config.duration {
        println!("duration = {}", format_float(duration));
    }
    println!("color_fps = {}", format_fps(config.color_fps));
    println!("no_logo = {}", config.no_logo);
    println!("no_packages = {}", config.no_packages);
    println!("no_header = {}", config.no_header);
    println!("mono = {}", config.mono);
    println!("no_color = {}", config.no_color);
    if let Some(seed) = config.seed {
        println!("seed = {}", seed);
    }
    println!("palette = \"{}\"", config.palette.name);
}

fn animation_style_name(style: &AnimationStyle) -> &'static str {
    match style {
        AnimationStyle::Wave => "wave",
        AnimationStyle::Pulse => "pulse",
        AnimationStyle::Neon => "neon",
        AnimationStyle::Matrix => "matrix",
        AnimationStyle::Fire => "fire",
        AnimationStyle::Fall => "fall",
        AnimationStyle::Marquee => "marquee",
        AnimationStyle::Typing => "typing",
        AnimationStyle::Plasma => "plasma",
        AnimationStyle::Glow => "glow",
        AnimationStyle::Pixel => "pixel",
        AnimationStyle::Aurora => "aurora",
        AnimationStyle::Glitch => "glitch",
        AnimationStyle::PulseRings => "pulse-rings",
        AnimationStyle::MeteorRain => "meteor-rain",
        AnimationStyle::Lava => "lava",
        AnimationStyle::EdgeGlow => "edge-glow",
    }
}

fn format_float(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn format_fps(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as u32)
    } else {
        format_float(value)
    }
}

fn load_logo_file(path: &str) -> io::Result<Vec<String>> {
    let text = fs::read_to_string(path)?;
    Ok(sanitize_logo_text(&text))
}

fn sanitize_logo_text(text: &str) -> Vec<String> {
    let lines: Vec<String> = text
        .lines()
        .take(MAX_LOGO_LINES)
        .map(sanitize_logo_line)
        .collect();

    if lines.iter().all(|line| line.trim().is_empty()) {
        Vec::new()
    } else {
        lines
    }
}

fn sanitize_logo_line(line: &str) -> String {
    let line = line.strip_suffix('\r').unwrap_or(line);
    let expanded = line.replace('\t', &" ".repeat(TAB_WIDTH));
    parse_ansi_text(&expanded)
        .into_iter()
        .filter_map(|(ansi, ch)| (ansi.is_empty() && ch != '\0').then_some(ch))
        .take(MAX_LOGO_COLUMNS)
        .collect()
}

fn print_help() {
    let mut style_names = animation::styles::AnimationStyle::available_styles();
    style_names.push("random");
    style_names.push("daily");
    let styles = style_names.join(", ");
    let distros = system::supported_distro_ids().join(", ");
    let palettes = animation::available_palette_names().join(", ");
    println!(
        "neonfetch - fast colorful animated system info\n\nUsage:\n  neonfetch [options]\n\nOptions:\n  --style <name>        Animation style (default: neon; real style, random, or daily)\n  --palette <name>      Color palette (default: default)\n  --speed <val>         Animation speed (0.1-20.0, default 1.0)\n  --color-fps <val>     Color refresh FPS (5-120, default 30)\n  --duration <sec>      Auto-exit after N seconds (animation mode)\n  --frame               Render one frame and exit (animation mode)\n  --fetch               Print info once and exit\n  --json                Print keyed JSON object and exit\n  --show <keys>         Show only comma-separated info fields in that order\n  --hide <keys>         Hide comma-separated info fields\n  --list-fields         List available info field keys\n  --mono                Render in grayscale (animations/info)\n  --no-color, -C        Disable ANSI colors (plain text)\n  --logo-file <path>    Use a UTF-8 text file as the ASCII logo\n  --no-logo, -L         Hide ASCII logo\n  --distro <id>         Force a distro logo on any platform\n  --no-packages, -P     Hide packages field and skip package detection\n  --no-header           Hide username@hostname header divider\n  --seed <u64>          Deterministic random seed for animations and --style random\n  --config <path>       Load config from path\n  --no-config           Ignore config files\n  --print-config        Print effective config and exit\n  --list-styles         List available styles\n  --list-palettes       List available palettes\n  -h, --help            Show this help\n  -V, --version         Show version\n\nConfig search:\n  --config, NEONFETCH_CONFIG, XDG_CONFIG_HOME, ~/.config/neonfetch/config.toml\n\nInfo fields:\n  {}\n\nKeys (animation mode):\n  q / Esc / Ctrl+C      Quit and restore the terminal\n\nDistros:\n  {}\n\nStyles:\n  {}\n\nPalettes:\n  {}\n\nPseudo-styles:\n  random                Pick a random showcase style each run; honors --seed\n  daily                 Pick one showcase style from the local date",
        INFO_FIELD_KEYS.join(", "),
        distros,
        styles,
        palettes
    );
}

fn print_style_list() {
    for name in animation::styles::AnimationStyle::available_styles() {
        println!("{}", name);
    }
    println!("random (pseudo): pick a random showcase style each run; honors --seed");
    println!("daily (pseudo): pick one showcase style from the local date");
}

fn print_version() {
    println!("neonfetch {}", env!("CARGO_PKG_VERSION"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_style_is_deterministic_for_date() {
        let today = pick_daily_style_for_date(20_260_706);
        let same_today = pick_daily_style_for_date(20_260_706);
        let tomorrow = pick_daily_style_for_date(20_260_707);
        let same_tomorrow = pick_daily_style_for_date(20_260_707);

        assert_eq!(today, same_today);
        assert_eq!(tomorrow, same_tomorrow);
    }

    #[test]
    fn seeded_random_style_is_deterministic() {
        assert_eq!(pick_random_style(Some(7)), pick_random_style(Some(7)));
    }

    #[test]
    fn seeded_random_style_uses_showcase_pool() {
        for seed in 0..256 {
            let style = pick_random_style(Some(seed));
            assert!(SHOWCASE_STYLE_POOL.contains(&style));
        }
    }

    #[test]
    fn showcase_style_pool_excludes_noisy_styles() {
        assert!(!SHOWCASE_STYLE_POOL.contains(&AnimationStyle::Pixel));
        assert!(!SHOWCASE_STYLE_POOL.contains(&AnimationStyle::Glitch));

        for style in SHOWCASE_STYLE_POOL {
            assert!(AnimationStyle::all().contains(style));
        }
    }

    #[test]
    fn seeded_random_style_covers_more_than_one_style() {
        let mut seen = Vec::new();

        for seed in 0..100 {
            let style = pick_random_style(Some(seed));
            if !seen.contains(&style) {
                seen.push(style);
            }
        }

        assert!(seen.len() > 1);
    }

    #[test]
    fn unix_days_to_yyyymmdd_handles_known_dates() {
        assert_eq!(yyyymmdd_from_unix_days(0), 19_700_101);
        assert_eq!(yyyymmdd_from_unix_days(20_000), 20_241_004);
    }
}
