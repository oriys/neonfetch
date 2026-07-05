# Neonfetch 🌈

A fast, colorful, and animated system information fetch tool written in Rust. Neonfetch displays your system information with beautiful ANSI animations and
multiple visual styles.

![Neon Style](neon.png) ![Fire Style](fire.png) ![Matrix Style](matrix.png) ![Wave Style](wave.png)

## Features

- **17 Animation Styles**: Choose from various eye-catching visual effects
- **Theme Palettes**: Recolor any animation with built-in palettes
- **Real-time System Info**: CPU, memory, disk, GPU, network, and more
- **Cross-platform**: Works on macOS and Linux
- **Customizable Speed**: Adjust animation speed to your preference
- **Smooth Performance**: Optimized for minimal CPU usage
- **ASCII Art**: Platform-specific logos with Linux distro auto-detection, or your own UTF-8 logo file

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
| `random` | Pick a random showcase style each run; honors `--seed`     |
| `daily`  | Pick one showcase style from the local date, stable all day |

#### Shell Startup

Add one of these to your shell rc file for zero-config variety:

```bash
# Fresh style each terminal
neonfetch --style random --duration 3

# Stable style for the day
neonfetch --style daily --duration 3
```

### Color Palettes

Keep the animation motion you like and switch its color theme with
`--palette <name>`. The `default` palette preserves each style's original
colors.

```bash
# Dracula colors on the wave animation
neonfetch --style wave --palette dracula

# Pastel matrix rain
neonfetch --style matrix --palette pastel

# Monochrome lava motion
neonfetch --style lava --palette mono

# List all available palettes
neonfetch --list-palettes
```

#### Available Palettes

| Palette     | Description                          |
| ----------- | ------------------------------------ |
| `default`   | Original per-style colors            |
| `cyberpunk` | Neon cyan, magenta, yellow, and blue |
| `dracula`   | Dracula-inspired terminal colors     |
| `pastel`    | Soft low-contrast candy colors       |
| `sunset`    | Purple, red, orange, and gold        |
| `ocean`     | Deep blue through bright aqua        |
| `mono`      | White and gray monochrome            |

### Command Line Options

```bash
# Set animation speed (0.1 - 20.0, default: 1.0)
neonfetch --speed 2.0
neonfetch -s 0.5

# Recolor any animation style
neonfetch --style wave --palette dracula
neonfetch --style fire --palette ocean

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

# Force a Linux distro logo on any platform
neonfetch --distro ubuntu --fetch
neonfetch --distro=arch --style neon

# JSON output (machine-readable keyed object, prints and exits)
neonfetch --json
neonfetch --json --hide network

# Grayscale or plain text (no ANSI colors)
neonfetch --mono
neonfetch --no-color

# Show only selected info fields in the requested order
neonfetch --show os,cpu,memory

# Hide selected info fields
neonfetch --hide network,packages

# List available info field keys
neonfetch --list-fields

# Hide package manager detection
neonfetch --no-packages
neonfetch -P

# Hide the username@hostname header
neonfetch --no-header

# Deterministic animations with a fixed random seed
neonfetch --seed 42
neonfetch --style random --seed 7

# Load and inspect a specific config file
neonfetch --config /path/to/config.toml --print-config

# Ignore all config files
neonfetch --no-config

# Combine options
neonfetch --style fire --palette sunset --speed 1.5 --color-fps 45 --no-logo
```

### Configuration

Neonfetch 可以从配置文件读取默认偏好。命令行参数始终优先于配置文件；例如配置里写了 `style = "matrix"`，运行 `neonfetch --style fire` 仍会使用 `fire`。

配置文件查找顺序：

1. `--config <path>` 或 `--config=<path>`
2. `NEONFETCH_CONFIG`
3. `$XDG_CONFIG_HOME/neonfetch/config.toml`
4. `~/.config/neonfetch/config.toml`

文件不存在时会当作空配置处理。配置文件解析失败时，neonfetch 会向 stderr 打印一行警告并继续使用内置默认值和命令行参数。

```toml
style = "matrix"
speed = 2.0
duration = 3.0
color_fps = 60
no_logo = false
no_packages = false
no_header = false
mono = false
no_color = false
seed = 42
```

```bash
# Print the merged effective configuration and exit
neonfetch --print-config

# Try a one-off config file
neonfetch --config /tmp/neonfetch.toml --print-config
```

### Examples

```bash
# Slow matrix effect
neonfetch --style matrix --speed 0.3

# Fast fire animation
neonfetch --style fire --speed 3.0

# Fire animation with a custom ASCII logo
neonfetch --logo-file examples/custom-logo.txt --style fire

# Wave animation with Dracula colors
neonfetch --style wave --palette dracula

# Smooth aurora with high refresh rate
neonfetch --style aurora --color-fps 60

# Quick system info without animation
neonfetch --fetch

# Preview an Ubuntu logo on any platform
neonfetch --distro ubuntu --fetch

# Quick info without network or package fields
neonfetch --fetch --hide network,packages

# Only OS, CPU, and memory in that order
neonfetch --fetch --show os,cpu,memory
```

### Linux Distribution Logos

On Linux, Neonfetch reads `/etc/os-release` and selects a matching logo from
`ID`; if that is unknown, it falls back to the first known entry in `ID_LIKE`.
Unknown distros use the generic Linux logo. Use `--distro <id>` to force a
logo on any platform.

Supported distro IDs:

`arch`, `ubuntu`, `debian`, `fedora`, `alpine`, `nixos`, `manjaro`,
`opensuse`, `gentoo`, `linuxmint`, `kali`, `void`.

## System Information Displayed

Neonfetch shows comprehensive system information including:

- **Header**: Username and hostname
- **OS**: Operating system and version
- **Host**: Computer model
- **Kernel**: Kernel version
- **Uptime**: System uptime
- **Shell**: Current shell
- **Terminal**: Terminal emulator
- **CPU**: Processor model, core count, architecture, and base frequency when available
- **Cores**: Physical/logical core detail
- **GPU**: Graphics card information
- **Resolution**: Display resolution
- **Battery**: Battery percentage and status when available
- **Packages**: Installed package count
- **Temperature**: Average thermal sensor reading when available
- **Memory**: RAM usage and total
- **Swap**: Swap usage or disabled status
- **Disk**: Storage usage
- **Network**: Active network interface and IP
- **Locale**: Current locale

Use `--show <keys>` to whitelist fields and control their output order, or
`--hide <keys>` to remove selected fields from the default order. `--show` and
`--hide` are mutually exclusive. Unknown keys print a warning to stderr and are
ignored. Legacy `--no-packages`/`-P` and `--no-header` are kept as hide aliases.
Available keys are listed by `neonfetch --list-fields`.

## Technical Details

### Performance

- Frame pacing is wall-clock based: `--color-fps` caps the real frame rate,
  and `--speed` only accelerates the animation clock, so CPU usage stays flat
  at any speed
- Each frame is rendered into a single reused buffer and written to the
  terminal with one `write` + `flush`; consecutive cells with the same color
  share one ANSI escape sequence
- System info probes (GPU, packages, battery, ...) run in parallel threads at
  startup only when their fields are selected

### Platform Support

- **macOS**: Full support with native system information gathering
- **Linux**: Full support with comprehensive hardware detection and distro logos
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
├── config.rs            # Config file loading and parsing
├── animation/           # Animation styles and effects
│   ├── mod.rs          # Animation module exports
│   ├── palette.rs      # Shared color palette definitions
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
│   ├── logo_distro.rs # Linux distro logo mapping and os-release parsing
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
