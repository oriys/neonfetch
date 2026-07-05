# Neonfetch 🌈

A fast, colorful, and animated system information fetch tool written in Rust. Neonfetch displays your system information with beautiful ANSI animations and
multiple visual styles.

![Neon Style](neon.png) ![Fire Style](fire.png) ![Matrix Style](matrix.png) ![Wave Style](wave.png)

## Features

- **17 Animation Styles**: Choose from various eye-catching visual effects
- **Real-time System Info**: CPU, memory, disk, GPU, network, and more
- **Cross-platform**: Works on macOS and Linux
- **Customizable Speed**: Adjust animation speed to your preference
- **Smooth Performance**: Optimized for minimal CPU usage
- **ASCII Art**: Platform-specific logos or your own UTF-8 logo file

## Installation

### From Source

Install directly from the latest commit (Git HEAD) without cloning:

```bash
cargo install --locked --git https://github.com/oriys/neonfetch neonfetch
```

Or clone and then install the binary into your Cargo bin directory (adds stripping & reuse of dependencies):

```bash
git clone https://github.com/oriys/neonfetch
cd neonfetch
cargo install --path . --locked
```

Just building locally (binary stays in target/):

```bash
git clone https://github.com/oriys/neonfetch
cd neonfetch
cargo build --release
```

The binary will be available at `target/release/neonfetch`.

### Dependencies

- Rust 2024 edition or later
- System dependencies are automatically handled by Cargo

## Usage

### Basic Usage

```bash
# Run with default neon style
neonfetch

# Show system info once without animation
neonfetch --fetch
```

While an animation is running, press `q`, `Esc`, or `Ctrl+C` to quit. The
terminal (cursor, colors, input mode) is always restored on exit, including
on panics.

### Animation Styles

Choose from 17 different animation styles:

```bash
# Neon glow effect (default)
neonfetch --style neon

# Matrix digital rain
neonfetch --style matrix

# Fire particle system
neonfetch --style fire

# Falling letters with physics
neonfetch --style fall

# Aurora borealis effect
neonfetch --style aurora

# Plasma waves
neonfetch --style plasma

# And many more...

# Pick a random style
neonfetch --style random

# Pick one style per local day
neonfetch --style daily

# Reproduce a random style choice
neonfetch --style random --seed 7

# List all available styles
neonfetch --list-styles
```

#### Available Styles

| Style         | Alias            | Description                   |
| ------------- | ---------------- | ----------------------------- |
| `neon`        | `n`              | Glowing neon effect (default) |
| `wave`        | `w`              | Color wave animation          |
| `pulse`       | `p`              | Pulsing color effect          |
| `matrix`      | `m`              | Matrix-style digital rain     |
| `fire`        | `f`              | Fire particle system          |
| `fall`        | `s`, `stack`     | Physics-based falling letters |
| `marquee`     | `mq`             | Scrolling marquee effect      |
| `typing`      | `t`, `type`      | Typewriter animation          |
| `plasma`      | `ps`             | Plasma wave patterns          |
| `glow`        | `g`              | Soft glow effect              |
| `pixel`       | `px`             | Retro pixel color cycling     |
| `aurora`      | `au`, `northern` | Aurora borealis               |
| `glitch`      | `gl`             | Digital glitch effect         |
| `pulse-rings` | `pr`, `rings`    | Expanding pulse rings         |
| `meteor-rain` | `mr`, `meteor`   | Falling meteors               |
| `lava`        | `lv`             | Lava flow effect              |
| `edge-glow`   | `eg`             | Edge highlighting             |

Pseudo-styles:

| Style    | Description                                                |
| -------- | ---------------------------------------------------------- |
| `random` | Pick a random real style each run; honors `--seed`         |
| `daily`  | Pick one real style from the local date, stable all day    |

#### Shell Startup

Add one of these to your shell rc file for zero-config variety:

```bash
# Fresh style each terminal
neonfetch --style random --duration 3

# Stable style for the day
neonfetch --style daily --duration 3
```

### Command Line Options

```bash
# Set animation speed (0.1 - 20.0, default: 1.0)
neonfetch --speed 2.0
neonfetch -s 0.5

# Set color refresh rate (5.0 - 120.0 FPS, default: 30.0)
neonfetch --color-fps 60

# Limit animation duration in seconds
neonfetch --duration 5

# Render a single frame and exit (useful for screenshots)
neonfetch --frame

# Hide ASCII logo (show only info list)
neonfetch --no-logo
neonfetch -L

# Use a custom UTF-8 ASCII logo file
neonfetch --logo-file examples/custom-logo.txt --style fire

# JSON output (machine-readable, prints and exits)
neonfetch --json
neonfetch --json --no-logo

# Grayscale or plain text (no ANSI colors)
neonfetch --mono
neonfetch --no-color

# Skip package manager detection
neonfetch --no-packages
neonfetch -P

# Hide the username@hostname header
neonfetch --no-header

# Deterministic animations with a fixed random seed
neonfetch --seed 42
neonfetch --style random --seed 7

# Combine options
neonfetch --style fire --speed 1.5 --color-fps 45 --no-logo
```

### Examples

```bash
# Slow matrix effect
neonfetch --style matrix --speed 0.3

# Fast fire animation
neonfetch --style fire --speed 3.0

# Fire animation with a custom ASCII logo
neonfetch --logo-file examples/custom-logo.txt --style fire

# Smooth aurora with high refresh rate
neonfetch --style aurora --color-fps 60

# Quick system info without animation
neonfetch --fetch
```

## System Information Displayed

Neonfetch shows comprehensive system information including:

- **Host**: Computer model and hostname
- **OS**: Operating system and version
- **Kernel**: Kernel version
- **Uptime**: System uptime
- **Packages**: Installed package count (Homebrew on macOS, dpkg on Linux)
- **Shell**: Current shell
- **Resolution**: Display resolution
- **DE/WM**: Desktop environment or window manager
- **Theme**: System theme information
- **Icons**: Icon theme
- **Terminal**: Terminal emulator
- **CPU**: Processor model and usage
- **GPU**: Graphics card information
- **Memory**: RAM usage and total
- **Disk**: Storage usage
- **Network**: Active network interface and IP

## Technical Details

### Performance

- Frame pacing is wall-clock based: `--color-fps` caps the real frame rate,
  and `--speed` only accelerates the animation clock, so CPU usage stays flat
  at any speed
- Each frame is rendered into a single reused buffer and written to the
  terminal with one `write` + `flush`; consecutive cells with the same color
  share one ANSI escape sequence
- System info probes (GPU, packages, battery, ...) run in parallel threads at
  startup

### Platform Support

- **macOS**: Full support with native system information gathering
- **Linux**: Full support with comprehensive hardware detection
- **Windows**: Not currently supported

### Dependencies

- `crossterm`: Cross-platform terminal manipulation and input events
- `sysinfo`: System information gathering
- `fastrand`: Fast random number generation for effects
- `get_if_addrs`: Network interface detection
- `serde_json`: `--json` output

## Development

### Building

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release

# Run tests
cargo test
```

### Project Structure

```
src/
├── main.rs              # CLI parsing, frame loop, terminal guard, renderers
├── animation/           # Animation styles and effects
│   ├── mod.rs          # Animation module exports
│   ├── styles.rs       # Style definitions and per-cell color functions
│   ├── aurora.rs       # Aurora borealis effect
│   ├── fall.rs         # Falling-letters physics simulation
│   ├── fire.rs         # Fire gradient (sparks overlaid in main loop)
│   ├── marquee.rs      # Sweeping highlight band
│   ├── matrix.rs       # Matrix digital rain
│   ├── meteor.rs       # Diagonal meteor streaks
│   ├── plasma.rs       # Plasma wave effects
│   └── ...            # Other animation implementations
├── system/             # System information gathering
│   ├── mod.rs         # System module exports
│   ├── info.rs        # Main system info collection
│   ├── logo_macos.rs  # macOS ASCII art
│   └── logo_linux.rs  # Linux ASCII art
└── util/              # Utilities
    ├── mod.rs         # Utility module exports
    ├── ansi.rs        # ANSI escape sequence parsing
    └── framebuf.rs    # Reusable frame buffer with color-run deduplication
```

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for:

- New animation styles
- Performance improvements
- Platform-specific enhancements
- Bug fixes
- Documentation improvements

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgments

- Inspired by the original `neofetch` tool
- Built with the amazing Rust ecosystem
- Thanks to all contributors and users

---

_Neonfetch - Making system information beautiful, one animation at a time._ ✨
