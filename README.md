# neonfetch

A fast, colorful, animated system info fetch tool for your terminal. Written in Rust with performance-focused rendering and multiple eye‑candy animation styles.

## Features

- System information (host, OS, kernel, uptime, CPU model / cores, memory usage, network interfaces, GPU if available, etc.)
- Per‑OS ASCII logo (macOS, Linux, fallback) cleanly separated into dedicated modules
- Multiple animation styles:
  - Classic: Smooth continuous rainbow cycling across the output
  - Wave: Horizontal sine hue wave rippling across columns
  - Pulse: Traveling brightness pulse layered over a slower hue wave
  - Neon: Narrow moving hue band with subtle breathing (glow) effect
  - Matrix: Falling code rain style vertical columns with randomized brightness
  - Fire: Two flame modes (basic / advanced) simulating heat diffusion
- Fire mode selection: `--fire-mode basic|advanced`
- Adjustable animation speed: `--speed <float>` (higher = faster phase progression)
- Set color refresh rate independently of frame rate: `--color-fps <int>` (reduces flicker & CPU)
- Animated mode is now the default (no flag needed)
- Static (one‑shot) output mode with `--fetch` / `-f`
- Efficient incremental redraw: only updates when color phase actually changes
- HSV color space utilities with conversion to RGB
- ANSI escape parsing utility separated for reuse
- Modular architecture (animation, system, util) for easier extension
- Cross‑platform (tested macOS + Linux)

## Installation

### Build from source
Requires Rust (stable).

```
cargo build --release
```
Binary will be at `target/release/neonfetch`.

### Run directly
```
cargo run -- --help
```

## Usage

```
neonfetch [options]
```

Common flags:
- `--fetch, -f`           One-shot system info (disable animation loop)
- `--loop, -l`            Force loop mode (normally already default; kept for backward compatibility)
- `--style <name>`        Animation style: classic | wave | pulse | neon | matrix | fire (default classic)
- `--fire-mode <mode>`    For fire style: basic | advanced (default advanced)
- `--speed <float>`       Animation speed multiplier (default 1.0)
- `--color-fps <int>`     Color phase updates per second (decoupled from terminal frame rate, default 30)
- `--help`                Show help

Examples:
```
# One-shot system info (static)
neonfetch --fetch

# Continuous pulse animation (loop is default, flag optional)
neonfetch --style pulse --speed 1.6

# Neon style with reduced color updates to save CPU
neonfetch --style neon --color-fps 20

# Fire animation in advanced mode (explicit)
neonfetch --style fire --fire-mode advanced
```

## Performance Notes

- The render loop computes a discrete color phase; frames only redraw when the phase changes.
- A sleep targets the next phase boundary instead of a fixed interval, smoothing animation and lowering CPU.
- Color generation uses lightweight math; heavy operations (system info gathering) are performed once per run unless rerun.
- ASCII logos & system info are pre-built into a string buffer; only color wrapping changes per frame.

## Architecture Overview

```
src/
  main.rs               CLI + render loop
  animation/
    mod.rs              Style enums & exports
    styles.rs           Shared style enum + color dispatcher + HSV utilities
    matrix.rs           Matrix column color logic
    fire.rs             Fire gradient + noise based coloring
  system/
    mod.rs              System info + ascii_logo dispatcher (cfg per OS)
    info.rs             Collects & formats system details
    logo_*.rs           OS-specific ASCII art data
  util/
    ansi.rs             ANSI escape parsing helper
```

Adding a new animation:
1. Create a new `<name>.rs` in `animation/` with a function that returns an RGB tuple given coordinates & time.
2. Add a variant to `AnimationStyle` and extend the dispatcher in `styles.rs`.
3. Document it in this README.

Adding a new OS logo:
1. Add `logo_<os>.rs` with an `ascii_logo()` returning `&'static [&'static str]`.
2. Extend `system/mod.rs` with the proper `#[cfg(...)]` gated module + re-export.

## Roadmap / Ideas

- Graceful Ctrl+C handling to always restore cursor state
- Optional frame rate limiter independent of color phase updates
- Config file for default style & speeds
- Windows support (logo + system info adjustments)
- Caching / lazy static for regex & logos (micro-optimization)

## Contributing

PRs and issues welcome. Please format with `cargo fmt` and run `cargo clippy -- -D warnings` before submitting.

## License

MIT
