use crate::util::framebuf::FrameBuf;

// Fall style: the rendered text detaches letter by letter (bottom rows first),
// falls under gravity with a little horizontal drift, bounces, and piles up at
// the bottom of the terminal. After everything settles the cycle restarts.
//
// All timing is in *animation seconds* (already scaled by --speed), so the
// physics constants below are speed-independent.

const GRAVITY: f32 = 55.0; // cells / s^2
const SETTLED_PAUSE: f32 = 4.0; // seconds to admire the pile before restarting

const STATIC_RGB: (u8, u8, u8) = (200, 200, 200);
const WAITING_RGB: (u8, u8, u8) = (120, 120, 120);
const ACTIVE_RGB: (u8, u8, u8) = (220, 220, 220);
const PILE_RGB: (u8, u8, u8) = (200, 200, 200);

#[derive(Clone, Copy)]
struct FallingLetter {
    ch: char,
    orig_row: usize,
    orig_col: usize,
    xf: f32,
    y: f32,
    vx: f32,
    vy: f32,
    release: f32, // animation-time offset (from phase start) when it lets go
}

#[derive(Clone, Copy)]
struct SettledUnit {
    ch: char,
    tilt: f32,
}

type OverlayCell = (char, (u8, u8, u8));

#[derive(PartialEq)]
enum Phase {
    Static,
    Falling,
    Settled,
}

pub struct FallSim {
    letters: Vec<FallingLetter>,
    pile: Vec<Vec<SettledUnit>>, // per column, bottom-up
    phase: Phase,
    phase_start: f32,
    w: usize,
    h: usize,
    // Reusable per-frame grids (row-major, w*h)
    settled: Vec<Option<char>>,
    overlay: Vec<Option<OverlayCell>>,
}

impl FallSim {
    pub fn new() -> Self {
        FallSim {
            letters: Vec::new(),
            pile: Vec::new(),
            phase: Phase::Static,
            phase_start: 0.0,
            w: 0,
            h: 0,
            settled: Vec::new(),
            overlay: Vec::new(),
        }
    }

    /// Reset the simulation for a (new) terminal size.
    pub fn resize(&mut self, w: usize, h: usize, now: f32) {
        self.w = w;
        self.h = h;
        self.pile = vec![Vec::new(); w];
        self.letters.clear();
        self.phase = Phase::Static;
        self.phase_start = now;
        self.settled = vec![None; w * h];
        self.overlay = vec![None; w * h];
    }

    /// Advance the simulation. `plain` is the static character grid of the
    /// source text; `elapsed` / `dt` are in animation seconds.
    pub fn step(&mut self, plain: &[Vec<char>], elapsed: f32, dt: f32) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        if self.phase == Phase::Static && self.letters.is_empty() {
            self.spawn_letters(plain, elapsed);
        }
        match self.phase {
            Phase::Static => {
                if self
                    .letters
                    .iter()
                    .any(|fl| elapsed >= self.phase_start + fl.release)
                {
                    self.phase = Phase::Falling;
                }
            }
            Phase::Falling => {
                self.integrate(elapsed, dt);
                if self.letters.is_empty() {
                    self.phase = Phase::Settled;
                    self.phase_start = elapsed;
                }
            }
            Phase::Settled => {
                if elapsed - self.phase_start > SETTLED_PAUSE {
                    self.pile.iter_mut().for_each(|v| v.clear());
                    self.letters.clear();
                    self.phase = Phase::Static;
                    self.phase_start = elapsed;
                }
            }
        }
    }

    fn spawn_letters(&mut self, plain: &[Vec<char>], elapsed: f32) {
        let total_rows = plain.len().min(self.h);
        for (row_i, row) in plain.iter().enumerate().take(self.h) {
            for (col_i, &ch) in row.iter().enumerate().take(self.w) {
                if ch == ' ' {
                    continue; // blanks don't fall (and must not occupy pile space)
                }
                // Bottom rows release first so the text visibly crumbles upward.
                let inv = (total_rows.saturating_sub(1)).saturating_sub(row_i) as f32;
                let release = 0.5 + inv * 0.035 + fastrand::f32() * 0.4;
                let vx = (fastrand::f32() - 0.5) * 3.0;
                self.letters.push(FallingLetter {
                    ch,
                    orig_row: row_i,
                    orig_col: col_i,
                    xf: col_i as f32,
                    y: row_i as f32,
                    vx,
                    vy: 0.0,
                    release,
                });
            }
        }
        self.phase_start = elapsed;
    }

    fn integrate(&mut self, elapsed: f32, dt: f32) {
        let (w, h) = (self.w as f32, self.h as f32);
        for fl in self.letters.iter_mut() {
            if elapsed < self.phase_start + fl.release {
                continue;
            }
            fl.vx += (fastrand::f32() - 0.5) * 0.6 * dt;
            fl.vx *= 0.985_f32.powf(dt * 60.0);
            fl.xf += fl.vx * dt;
            if fl.xf < 0.0 {
                fl.xf = 0.0;
                fl.vx = -fl.vx * 0.3;
            }
            if fl.xf > w - 1.0 {
                fl.xf = w - 1.0;
                fl.vx = -fl.vx * 0.3;
            }
            fl.vy += GRAVITY * dt;
            fl.y += fl.vy * dt;
            let col = fl.xf.round().clamp(0.0, w - 1.0) as usize;
            let col_height = self.pile[col].len();
            let ground_y = (h - 1.0 - col_height as f32).max(0.0);
            if fl.y >= ground_y {
                if fl.vy > 18.0 {
                    fl.y = ground_y;
                    fl.vy = -fl.vy * 0.25;
                    fl.vx *= 0.6;
                } else {
                    fl.y = ground_y;
                    let tilt = settle_tilt(&self.pile, col, col_height);
                    self.pile[col].push(SettledUnit { ch: fl.ch, tilt });
                    fl.release = f32::INFINITY; // mark for removal
                }
            }
        }
        self.letters.retain(|fl| fl.release.is_finite());
    }

    /// Render the current state into `fb`, one full terminal row per line.
    pub fn render(&mut self, fb: &mut FrameBuf, elapsed: f32) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let (w, h) = (self.w, self.h);
        self.settled.fill(None);
        self.overlay.fill(None);

        // Place the pile with support-aware tilt shifting.
        let bottom = h - 1;
        for col in 0..w {
            let stack_len = self.pile[col].len();
            if stack_len == 0 {
                continue;
            }
            for level in 0..stack_len {
                let su = self.pile[col][level];
                let Some(row) = bottom.checked_sub(level) else {
                    continue;
                };
                let rel = level as f32 / stack_len as f32;
                let raw_shift = su.tilt * rel * 1.3;
                let step = (raw_shift.abs().floor() * raw_shift.signum()).clamp(-2.0, 2.0);
                let mut target = (col as isize + step as isize).clamp(0, w as isize - 1);
                // Require support below (except at the bottom row).
                if level > 0 {
                    while target != col as isize
                        && self.settled[(row + 1) * w + target as usize].is_none()
                    {
                        target += if target > col as isize { -1 } else { 1 };
                    }
                }
                // Resolve collisions: scan outward from home before overwriting.
                if self.settled[row * w + target as usize].is_some()
                    && let Some(free_col) = find_free_cell_in_row(&self.settled, row, w, col)
                {
                    target = free_col as isize;
                }
                self.settled[row * w + target as usize] = Some(su.ch);
            }
        }

        // Overlay pass: one linear scan over the letters instead of a
        // per-cell search.
        for fl in self.letters.iter() {
            if elapsed < self.phase_start + fl.release {
                // Not yet released: keep showing it at its home position.
                if fl.orig_row < h && fl.orig_col < w {
                    let rgb = if self.phase == Phase::Static {
                        STATIC_RGB
                    } else {
                        WAITING_RGB
                    };
                    let idx = fl.orig_row * w + fl.orig_col;
                    if self.overlay[idx].is_none() {
                        self.overlay[idx] = Some((fl.ch, rgb));
                    }
                }
            } else {
                let ry = fl.y.round();
                let cx = fl.xf.round().clamp(0.0, w as f32 - 1.0) as usize;
                if ry >= 0.0 && (ry as usize) < h {
                    // Airborne letters draw over everything else.
                    self.overlay[ry as usize * w + cx] = Some((fl.ch, ACTIVE_RGB));
                }
            }
        }

        for row in 0..h {
            fb.goto_line(row + 1);
            let base = row * w;
            for col in 0..w {
                match (self.overlay[base + col], self.settled[base + col]) {
                    (Some((ch, rgb)), _) => fb.put(ch, rgb),
                    (None, Some(ch)) => fb.put(ch, PILE_RGB),
                    (None, None) => fb.put(' ', PILE_RGB),
                }
            }
            fb.end_line();
        }
    }
}

// Lean a settling letter toward whichever neighbor column is lower, so the
// pile edges slump outward instead of forming perfect towers.
fn settle_tilt(pile: &[Vec<SettledUnit>], col: usize, col_height: usize) -> f32 {
    let prospective = col_height + 1;
    let lh = if col > 0 {
        pile[col - 1].len()
    } else {
        prospective
    };
    let rh = if col + 1 < pile.len() {
        pile[col + 1].len()
    } else {
        prospective
    };
    let diff_left = (prospective as isize - lh as isize).max(0) as f32;
    let diff_right = (prospective as isize - rh as isize).max(0) as f32;
    let (mut dir, mut mag) = (0.0f32, 0.0f32);
    if diff_left > 0.0 || diff_right > 0.0 {
        if (diff_left - diff_right).abs() < 0.1 {
            dir = if fastrand::f32() < 0.5 { -1.0 } else { 1.0 };
            mag = diff_left.max(diff_right);
        } else if diff_left > diff_right {
            dir = -1.0;
            mag = diff_left;
        } else {
            dir = 1.0;
            mag = diff_right;
        }
    }
    if fastrand::f32() < 0.25 {
        return 0.0; // some stay upright
    }
    if mag > 0.0 {
        mag = (mag / 2.5).min(1.4);
        mag *= 0.85 + fastrand::f32() * 0.3;
    }
    dir * mag
}

fn find_free_cell_in_row(
    settled: &[Option<char>],
    row: usize,
    width: usize,
    home_col: usize,
) -> Option<usize> {
    let base = row * width;
    if settled[base + home_col].is_none() {
        return Some(home_col);
    }
    for d in 1..width {
        if let Some(left) = home_col.checked_sub(d)
            && settled[base + left].is_none()
        {
            return Some(left);
        }
        let right = home_col + d;
        if right < width && settled[base + right].is_none() {
            return Some(right);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaces_do_not_become_letters() {
        let mut sim = FallSim::new();
        sim.resize(10, 5, 0.0);
        let plain = vec![vec!['a', ' ', 'b'], vec![' ', ' ', ' ']];
        sim.step(&plain, 0.0, 0.016);
        assert_eq!(sim.letters.len(), 2);
    }

    #[test]
    fn full_cycle_settles_all_letters() {
        fastrand::seed(7);
        let mut sim = FallSim::new();
        sim.resize(20, 10, 0.0);
        let plain = vec![vec!['x'; 10]; 3];
        let dt = 1.0 / 30.0;
        let mut t = 0.0;
        for _ in 0..3000 {
            t += dt;
            sim.step(&plain, t, dt);
            if sim.phase == Phase::Settled {
                break;
            }
        }
        assert!(sim.phase == Phase::Settled, "letters never settled");
        let piled: usize = sim.pile.iter().map(|c| c.len()).sum();
        assert_eq!(piled, 30);
    }
}
