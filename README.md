# neonfetch
 - pulse-rings – expanding concentric color rings from a drifting center
			<code>--style neon</code><br>
			<sub>Hue band + gentle breathing</sub>
		</td>
		<td align="center" width="50%">
 - `--style <name>`     neon | wave | pulse | matrix | fire | marquee | typing | plasma | glow | aurora | pulse-rings (default neon)
			<video width="300" muted autoplay loop playsinline preload="auto" src="wave.mp4">Your browser can't embed this video. <a href="wave.mp4">Download wave.mp4</a></video><br>
			<code>--style wave</code><br>
			<sub>Luminance wave sweep</sub>
		</td>
	</tr>
	<tr>
		<td align="center">
			<b>matrix</b><br>
			<video width="300" muted autoplay loop playsinline preload="auto" src="matrix.mp4">Your browser can't embed this video. <a href="matrix.mp4">Download matrix.mp4</a></video><br>
			<code>--style matrix</code><br>
			<sub>Code rain variation</sub>
neonfetch --style pulse-rings --speed 1.1 --color-fps 50
		</td>
		<td align="center">
			<b>fire (advanced)</b><br>
			<video width="300" muted autoplay loop playsinline preload="auto" src="fire.mp4">Your browser can't embed this video. <a href="fire.mp4">Download fire.mp4</a></video><br>
			<code>--style fire --fire-mode advanced</code><br>
			<sub>Multi-band flame gradient</sub>
		</td>
	</tr>
</table>

> Autoplay hints: Chrome only autoplays if the video is muted (done) and often when not hidden. Controls removed to reduce chance of blocking. If it still doesn't autoplay (GitHub may defer), open the raw file or allow site media autoplay in Chrome site settings. Videos are MP4 (H.264).

### Static Screenshots
If videos do not load, here are static captures (click to open full size):

<p align="center">
	<img src="neon.png" alt="neon" width="320" />
	<img src="wave.png" alt="wave" width="320" />
	<img src="matrix.png" alt="matrix" width="320" />
	<img src="fire.png" alt="fire" width="320" />
</p>

## What it shows
Host / OS / kernel / uptime / shell / terminal / CPU / GPU / memory / swap / disk usage / IP / locale + an OS ASCII logo.

## Styles
- neon (default) – moving hue band + gentle breathing
- wave – brightness wave (light/dark flow)
- pulse – traveling brightness pulse over a hue drift
- matrix – green code rain style (per‑cell variation)
- fire – flame gradient (basic / advanced)
- marquee – horizontal moving highlight band (terminal marquee)
 - typing – progressive type-in reveal, then restart
 - plasma – flowing multi-sine color field
 - glow – soft global breathing glow with subtle per‑char shimmer
 - aurora – drifting multi-ribbon teal/green/blue curtains
 - glitch – intermittent column shifts & color channel distort bursts
 - pulse-rings – expanding concentric color rings from a drifting center
 - meteor-rain – diagonal multicolor shooting stars with fading trails
 - lava – molten flowing heat-map (procedural layered noise)
 - edge-glow – base neon with intensified character edge brightness

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
- `--style <name>`     neon | wave | pulse | matrix | fire | marquee | typing | plasma | glow | aurora | glitch | pulse-rings | meteor-rain | lava | edge-glow (default neon)
- `--fire-mode <m>`    basic | advanced (default advanced)
- `--speed <x>`        Animation speed (0.1–10, default 1.0)
- `--color-fps <n>`    Color refresh rate (5–120, default 30)

Examples:
```
neonfetch --fetch
neonfetch --style wave --speed 0.8
neonfetch --style fire --fire-mode basic --color-fps 20
neonfetch --style marquee --speed 1.5
neonfetch --style typing
neonfetch --style plasma --speed 0.7
neonfetch --style glow --color-fps 45
neonfetch --style aurora --speed 1.2 --color-fps 40
neonfetch --style glitch --speed 1.4 --color-fps 50
neonfetch --style pulse-rings --speed 1.1 --color-fps 50
neonfetch --style meteor-rain --speed 1.0 --color-fps 45
neonfetch --style lava --speed 0.9 --color-fps 35
neonfetch --style edge-glow --speed 1.0 --color-fps 40
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
