# neonfetch

Animated system info fetch (Rust). Fast, colorful, minimal.

## What it shows
Host / OS / kernel / uptime / shell / terminal / CPU / GPU / memory / swap / disk usage / IP / locale + an OS ASCII logo.

## Styles
- neon (default) – moving hue band + gentle breathing
- wave – brightness wave (light/dark flow)
- pulse – traveling brightness pulse over a hue drift
- matrix – green code rain style (per‑cell variation)
- fire – flame gradient (basic / advanced)

`classic` is deprecated and now aliases to `neon`.

## Install
Requires Rust.

```
cargo install --path .
# or build manually
cargo build --release
```
Binary: `target/release/neonfetch` (if built manually).

## Usage
```
neonfetch [options]
```
Flags:
- `--fetch` / `-f`     One-shot (no animation)
- `--style <name>`     neon | wave | pulse | matrix | fire (default neon)
- `--fire-mode <m>`    basic | advanced (default advanced)
- `--speed <x>`        Animation speed (0.1–10, default 1.0)
- `--color-fps <n>`    Color refresh rate (5–120, default 30)

Examples:
```
neonfetch --fetch
neonfetch --style wave --speed 0.8
neonfetch --style fire --fire-mode basic --color-fps 20
```

## Notes
- System info gathered once then reused (avoids flicker).
- Frame pacing smooths color changes.
- Wave now uses brightness modulation (not full rainbow sweep).

## Dev
```
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## License
MIT

## Release (packaging)
Tag a version to trigger multi-target build & release assets:
```
git tag -a v0.1.0 -m "v0.1.0"
git push origin v0.1.0
```
Artifacts produced:
- Linux: x86_64, aarch64
- macOS: x86_64, arm64
- Windows: x86_64
Each archive name: `neonfetch-v<version>-<target>.{tar.gz|zip}`.
