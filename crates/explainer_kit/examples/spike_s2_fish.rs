//! Spikes S-2 and S-8 (plan §6.6; Video A 01 and 03, §9.4 and §9.7): a lake
//! of procedural fish, and a live 3×3 region map held for 80 s.
//!
//! ```text
//! cargo run --release -p explainer_kit --example spike_s2_fish -- OUT_DIR fish [WxH] [SECONDS] [COUNTS] [LAYERS]
//! cargo run --release -p explainer_kit --example spike_s2_fish -- OUT_DIR grid [WxH] [SECONDS]
//! ```
//!
//! LAYERS (default `lines,glow`) splits the cost: `lines` draws the
//! streamlines; `glow` gives marked fish the plan's `v_highlight` (stroked)
//! halo, `halo` a filled one (two translucent enlarged bodies) instead;
//! `none` draws neither.
//!
//! Writes into OUT_DIR (created; it must not exist):
//! - `fish` (S-2): `fish_<n>.mp4` for each n in the comma-separated COUNTS
//!   (default `0,600,1000,1500`; 0 is the control, the lake and its
//!   streamlines alone), SECONDS long (default 10) at WxH (default
//!   1920x1080) and 60 fps. Each fish is built once at the origin (a
//!   teardrop body and a forked tail, the tail a child) and moved only by
//!   placement: one dt updater shifts and rotates every fish and rotates
//!   each tail about its hinge, never rewriting points. One fish in ten is
//!   marked: gold, with a `v_highlight` halo; the rest are blue. The fish
//!   swim an overdamped drift −∇U plus a circulation c·J∇U along the
//!   contours of a habitat potential U (four haunts and a shore), with
//!   smooth velocity noise, so they circle their haunts and stay clumped
//!   (stationary density ∝ e^(−U/T)). A few dozen faint contours of U are
//!   drawn once underneath.
//! - `grid` (S-8): `grid.mp4`, SECONDS long (default 80) at WxH (default
//!   960x540): chapter 03_4's split view, two 3×3 region maps ("how many
//!   fish", "what fraction is marked") over a simulated population of
//!   20,000 fish (2,000 marked, released at the dock), recoloured in place
//!   every frame by one updater, with a live number in each of the 18 tiles
//!   (the kit's `live_number`, an `always_redraw` readout).
//! - one line per render on stdout: name, resolution, fps, frames, wall
//!   seconds, load average before and after, attempt; then the fps of each
//!   window of the scene's own clock (wall time between the updater calls
//!   that start and end the window). Every 10 s of scene time, stderr gets
//!   that window's fps and the process's CPU milliseconds per frame (all
//!   threads), which a loaded host's wall clock cannot hide.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use explainer_kit::encode::{self, Quality};
use explainer_kit::narration::wait_frames;
use explainer_kit::{BACKGROUND, Kit, Palette, children, hex, ink_style};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

const FPS: u32 = 60;

/// The lake: an ellipse centred on the origin with these semi-axes.
const LAKE_A: f64 = 6.7;
const LAKE_B: f64 = 3.7;
/// The favourite haunts (weeds, the deep end, lily pads, a log): centre x,
/// centre y, width σ and depth of each Gaussian well of U.
const HAUNTS: [[f64; 4]; 4] = [
    [-3.9, 1.4, 1.0, 2.4],
    [3.6, -1.2, 1.4, 2.8],
    [0.9, 2.0, 0.75, 1.6],
    [-1.2, -1.9, 0.9, 1.5],
];
/// The shore: U rises as SHORE_K (r − SHORE_R0)² past the elliptical
/// radius SHORE_R0.
const SHORE_R0: f64 = 0.82;
const SHORE_K: f64 = 60.0;
/// Temperature (stationary density ∝ e^(−U/T)), mobility and circulation.
const TEMP: f64 = 0.65;
const MOBILITY: f64 = 0.15;
const CIRCULATION: f64 = 3.0;
/// The drawn fish's velocity noise is an Ornstein–Uhlenbeck process with
/// this correlation time (s) and diffusion MOBILITY × TEMP: smooth
/// wandering instead of white-noise jitter.
const NOISE_TAU: f64 = 1.0;
/// Heading follows the velocity at this rate (1/s).
const TURN_RATE: f64 = 6.0;
/// Tail beat: amplitude (rad), base frequency (Hz) and extra Hz per unit
/// of speed.
const WAG: f64 = 0.4;
const BEAT_HZ: f64 = 2.0;
const BEAT_PER_SPEED: f64 = 4.0;
/// The tail's hinge on the body's axis (object space, fish facing +x).
const TAIL_PIVOT: f64 = -0.068;

/// The simulated population behind the S-8 maps (not drawn).
const POPULATION: usize = 20_000;
const POPULATION_MARKED: usize = 2_000;
/// Where the marked fish are let go, and the radius of the release.
const DOCK: [f64; 2] = [-5.2, -1.0];
const DOCK_R: f64 = 0.35;
/// Simulation substeps per frame (the map's population mixes 3× faster
/// than the drawn fish swim, so 80 s covers the flattening).
const SUBSTEPS: usize = 3;

/// Wall-clock stamps of the scene's frame updates: (scene seconds, when).
type Clock = Rc<RefCell<Vec<(f64, Instant)>>>;

/// This process's CPU time so far (all threads, from `ps`), in seconds.
fn cpu_seconds() -> Option<f64> {
    let out = std::process::Command::new("ps")
        .args(["-o", "time=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    // [[dd-]hh:]mm:ss.cc
    let text = String::from_utf8_lossy(&out.stdout);
    let (days, rest) = match text.trim().split_once('-') {
        Some((d, r)) => (d.parse::<f64>().ok()?, r.to_owned()),
        None => (0.0, text.trim().to_owned()),
    };
    let secs = rest
        .split(':')
        .try_fold(0.0, |acc, part| part.parse::<f64>().ok().map(|v| acc * 60.0 + v))?;
    Some(days * 86_400.0 + secs)
}

/// Stamp the frame update at scene time `t`; at each whole 10 s, also
/// print the wall fps and the CPU milliseconds per frame (all threads) of
/// the last 10 s to stderr: live progress, so a render paused from outside
/// still leaves its earlier windows, and a per-frame cost that a loaded
/// host's wall clock cannot hide.
fn tick(clock: &Clock, mark: &mut (usize, Option<f64>), t: f64) {
    let mut stamps = clock.borrow_mut();
    let now = Instant::now();
    stamps.push((t, now));
    let n = stamps.len();
    if n == 1 {
        *mark = (0, cpu_seconds());
    } else if n.is_multiple_of(10 * FPS as usize) {
        // frames since the window's first stamp
        let frames = (n - 1 - mark.0) as f64;
        let secs = now.duration_since(stamps[mark.0].1).as_secs_f64();
        let used = cpu_seconds();
        let per_frame = match (used, mark.1) {
            (Some(b), Some(a)) => format!("{:.1}", 1000.0 * (b - a) / frames),
            _ => "?".into(),
        };
        *mark = (n - 1, used);
        eprintln!(
            "t = {t:.0} s: {:.2} fps, {per_frame} CPU ms/frame over the last 10 s",
            frames / secs
        );
    }
}

/// The elliptical radius: 1 on the shoreline.
fn lake_r(x: f64, y: f64) -> f64 {
    ((x / LAKE_A).powi(2) + (y / LAKE_B).powi(2)).sqrt()
}

/// The habitat potential U and its gradient at (x, y).
fn habitat(x: f64, y: f64) -> (f64, f64, f64) {
    let (mut u, mut gx, mut gy) = (0.0, 0.0, 0.0);
    for [cx, cy, s, depth] in HAUNTS {
        let (dx, dy) = (x - cx, y - cy);
        let e = depth * (-(dx * dx + dy * dy) / (2.0 * s * s)).exp();
        u -= e;
        gx += e * dx / (s * s);
        gy += e * dy / (s * s);
    }
    let r = lake_r(x, y);
    if r > SHORE_R0 {
        let k = r - SHORE_R0;
        u += SHORE_K * k * k;
        let f = 2.0 * SHORE_K * k / r;
        gx += f * x / (LAKE_A * LAKE_A);
        gy += f * y / (LAKE_B * LAKE_B);
    }
    (u, gx, gy)
}

/// The deterministic drift μ(−∇U + c·J∇U), J the quarter turn.
fn drift(x: f64, y: f64) -> (f64, f64) {
    let (_, gx, gy) = habitat(x, y);
    (MOBILITY * (-gx - CIRCULATION * gy), MOBILITY * (-gy + CIRCULATION * gx))
}

/// Pull a point that left the water back onto the 0.97 shell.
fn keep_in_lake(x: f64, y: f64) -> (f64, f64) {
    let r = lake_r(x, y);
    if r > 0.97 { (x * 0.97 / r, y * 0.97 / r) } else { (x, y) }
}

fn normal(rng: &mut Pcg64Dxsm) -> f64 {
    let u = 1.0 - rng.next_f64();
    (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * rng.next_f64()).cos()
}

/// A lower bound on U over the lake (grid minimum less a margin).
fn u_floor() -> f64 {
    let mut lo = f64::INFINITY;
    for i in 0..=280 {
        for j in 0..=160 {
            let (x, y) = (
                -LAKE_A + 2.0 * LAKE_A * i as f64 / 280.0,
                -LAKE_B + 2.0 * LAKE_B * j as f64 / 160.0,
            );
            lo = lo.min(habitat(x, y).0);
        }
    }
    lo - 0.05
}

/// A position drawn from the stationary density ∝ e^(−U/T) (rejection).
fn sample_habitat(rng: &mut Pcg64Dxsm, floor: f64) -> (f64, f64) {
    loop {
        let x = (2.0 * rng.next_f64() - 1.0) * LAKE_A;
        let y = (2.0 * rng.next_f64() - 1.0) * LAKE_B;
        if lake_r(x, y) > 0.92 {
            continue;
        }
        if rng.next_f64() < (-(habitat(x, y).0 - floor) / TEMP).exp() {
            return (x, y);
        }
    }
}

/// The level set of U through `seed`, traced along its unit tangent
/// J∇U/|∇U| (midpoint steps, each pulled back onto the level by one Newton
/// step); `true` when it closed on itself.
fn contour(seed: (f64, f64)) -> (Vec<Vec3>, bool) {
    const H: f64 = 0.04;
    let tangent = |x: f64, y: f64| {
        let (_, gx, gy) = habitat(x, y);
        let n = gx.hypot(gy).max(1e-9);
        (-gy / n, gx / n)
    };
    let u0 = habitat(seed.0, seed.1).0;
    let (mut x, mut y) = seed;
    let mut pts = vec![[x, y, 0.0]];
    for k in 0..1_500 {
        let (tx, ty) = tangent(x, y);
        let (tx, ty) = tangent(x + 0.5 * H * tx, y + 0.5 * H * ty);
        x += H * tx;
        y += H * ty;
        let (u, gx, gy) = habitat(x, y);
        let g2 = (gx * gx + gy * gy).max(1e-12);
        x -= (u - u0) * gx / g2;
        y -= (u - u0) * gy / g2;
        if lake_r(x, y) > 0.99 {
            return (pts, false);
        }
        if k > 10 && (x - seed.0).hypot(y - seed.1) < H {
            return (pts, true);
        }
        pts.push([x, y, 0.0]);
    }
    (pts, false)
}

/// The water's fill.
const WATER: &str = "#112131";
/// Segments per streamline piece. fmn's stroke shader scans every segment
/// of a stroke for each pixel it shades, over the row span of all of them
/// (`fmn-render` `stroke.rs`, `PreparedStroke::nearest` and `row_span`), so
/// a long closed contour costs about (its width × its segment count) per
/// row of every tile a moving fish dirties. Short pieces keep both small.
const PIECE: usize = 8;

/// A few dozen faint contours of U (rings around each haunt, and the
/// open-water and shore levels), each cut into [`PIECE`]-segment polylines
/// drawn opaque in a colour pre-mixed with the water, so the shared
/// endpoints of the pieces do not double their alpha.
fn streamlines() -> Vec<VMobject> {
    let mut seeds = Vec::new();
    for [cx, cy, s, _] in HAUNTS {
        for f in [0.45, 0.85, 1.3, 1.8] {
            seeds.push((cx + f * s, cy));
        }
    }
    seeds.extend([
        (0.0, 0.0),
        (-1.8, 0.4),
        (2.0, 0.8),
        (0.6, -0.7),
        (-3.4, -0.9),
        (-2.1, 2.5),
        (2.6, 2.3),
        (4.8, 1.3),
        (5.25, 0.0),
        (5.6, 0.0),
        (5.9, 0.0),
        (6.2, 0.0),
    ]);
    let ink = interpolate_color(hex(WATER), Palette::blue(), 0.2);
    let style = Style::default().fill(ink, 0.0).stroke(ink, 1.4, 1.0);
    seeds
        .into_iter()
        .map(|seed| {
            let (mut pts, closed) = contour(seed);
            if closed {
                pts.push(pts[0]);
            }
            let pieces = (0..pts.len().saturating_sub(1)).step_by(PIECE).map(|i| {
                Polygon::polyline(pts[i..=(i + PIECE).min(pts.len() - 1)].to_vec())
                    .style(style)
                    .build()
            });
            v_group(pieces)
        })
        .collect()
}

/// How a marked fish glows.
#[derive(Clone, Copy, PartialEq)]
enum Glow {
    None,
    /// The plan's `v_highlight`: widening stroked copies of the body.
    Strokes,
    /// Filled, translucent, enlarged copies of the body (no strokes).
    Fills,
}

/// The fish body: a teardrop, blunt head at +x, tapering to the tail's hinge.
fn body_outline(scale: f64) -> Vec<Vec3> {
    (0..24)
        .map(|i| {
            let t = std::f64::consts::TAU * f64::from(i) / 24.0;
            [
                scale * 0.075 * t.cos(),
                scale * 0.034 * t.sin() * (0.6 + 0.4 * t.cos()),
                0.0,
            ]
        })
        .collect()
}

/// A fish at the origin facing +x: [glow (marked only), tail, body]; the
/// tail is drawn under the body, the glow under both.
fn fish_shape(marked: bool, glow: Glow) -> VMobject {
    let color = if marked { Palette::hero() } else { Palette::blue() };
    let body = Polygon::new(body_outline(1.0)).style(ink_style(color)).build();
    let tail = Polygon::new([
        [TAIL_PIVOT, 0.0, 0.0],
        [-0.13, 0.036, 0.0],
        [-0.112, 0.0, 0.0],
        [-0.13, -0.036, 0.0],
    ])
    .style(ink_style(color))
    .build();
    let mut parts = Vec::with_capacity(3);
    if marked && glow == Glow::Strokes {
        let source = body
            .clone()
            .with_style(Style::default().fill(color, 0.0).stroke(color, 0.0, 0.3));
        let outer = interpolate_color(color, hex(BACKGROUND), 0.5);
        parts.push(v_highlight(&source, 3, 6.0, [color, outer]));
    }
    if marked && glow == Glow::Fills {
        parts.push(v_group([1.7, 1.4].map(|s| {
            Polygon::new(body_outline(s))
                .style(Style::default().fill(color, 0.2).stroke(color, 0.0, 0.0))
                .build()
        })));
    }
    parts.push(tail);
    parts.push(body);
    v_group(parts)
}

/// One drawn fish: its handles and its swimming state.
struct Fish {
    root: Mob,
    tail: Mob,
    x: f64,
    y: f64,
    heading: f64,
    /// The velocity noise (OU), per axis.
    nx: f64,
    ny: f64,
    phase: f64,
    wag: f64,
}

impl Fish {
    /// One frame of swimming, applied by placement only: shift and turn
    /// the whole fish, then swing the tail about its hinge.
    fn swim(&mut self, s: &mut fmn::mobject::Stage, dt: f64, rng: &mut Pcg64Dxsm) {
        let decay = (-dt / NOISE_TAU).exp();
        let kick = (MOBILITY * TEMP / NOISE_TAU).sqrt() * (1.0 - decay * decay).sqrt();
        self.nx = self.nx * decay + kick * normal(rng);
        self.ny = self.ny * decay + kick * normal(rng);
        let (dx, dy) = drift(self.x, self.y);
        let (vx, vy) = (dx + self.nx, dy + self.ny);
        let (x, y) = keep_in_lake(self.x + vx * dt, self.y + vy * dt);
        let speed = vx.hypot(vy);
        let mut turn = 0.0;
        if speed > 1e-6 {
            let want = vy.atan2(vx) - self.heading;
            let want = want - std::f64::consts::TAU * (want / std::f64::consts::TAU).round();
            turn = want * (1.0 - (-TURN_RATE * dt).exp());
        }
        self.heading += turn;
        self.phase += std::f64::consts::TAU * (BEAT_HZ + BEAT_PER_SPEED * speed) * dt;
        let wag = WAG * self.phase.sin();
        s.shift(self.root, [x - self.x, y - self.y, 0.0]);
        s.rotate(self.root, turn, OUT, Some([x, y, 0.0]), None);
        let hinge = [
            x + self.heading.cos() * TAIL_PIVOT,
            y + self.heading.sin() * TAIL_PIVOT,
            0.0,
        ];
        s.rotate(self.tail, wag - self.wag, OUT, Some(hinge), None);
        (self.x, self.y, self.wag) = (x, y, wag);
    }
}

/// S-2: the lake, its streamlines and `count` fish.
struct Lake {
    count: usize,
    seconds: f64,
    lines: bool,
    glow: Glow,
    clock: Clock,
}

impl SceneConstruct for Lake {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let water = Ellipse::new()
            .width(2.0 * LAKE_A)
            .height(2.0 * LAKE_B)
            // no outline: a stroked closed loop is shaded across its full width (see PIECE)
            .style(ink_style(hex(WATER)))
            .build();
        let water = stage.add(water)?;
        if self.lines {
            for line in streamlines() {
                stage.add(line)?;
            }
        }
        let floor = u_floor();
        let mut rng = Pcg64Dxsm::from_seed(2_026);
        let mut fish = Vec::with_capacity(self.count);
        for i in 0..self.count {
            // every tenth fish is marked: exactly 10%, spread through the draw order
            let root = stage.add(fish_shape(i % 10 == 0, self.glow))?;
            let kids = children(stage, root);
            let tail = kids[kids.len() - 2];
            let (x, y) = sample_habitat(&mut rng, floor);
            let heading = std::f64::consts::TAU * rng.next_f64();
            let phase = std::f64::consts::TAU * rng.next_f64();
            let wag = WAG * phase.sin();
            stage.rotate(root, heading, OUT, Some(ORIGIN), None);
            stage.shift(root, [x, y, 0.0]);
            let hinge = [x + heading.cos() * TAIL_PIVOT, y + heading.sin() * TAIL_PIVOT, 0.0];
            stage.rotate(tail, wag, OUT, Some(hinge), None);
            fish.push(Fish {
                root,
                tail,
                x,
                y,
                heading,
                nx: 0.0,
                ny: 0.0,
                phase,
                wag,
            });
        }
        let clock = Rc::clone(&self.clock);
        let (mut t, mut cpu) = (0.0, (0, None));
        stage.add_dt_updater(
            water,
            move |s, _, dt| {
                if dt <= 0.0 {
                    return;
                }
                t += dt;
                tick(&clock, &mut cpu, t);
                for f in &mut fish {
                    f.swim(s, dt, &mut rng);
                }
            },
            false,
        )?;
        wait_frames(stage, (self.seconds * f64::from(FPS)).round() as u64)
    }
}

/// The S-8 population: overdamped Langevin (white noise; never drawn).
struct Population {
    x: Vec<f64>,
    y: Vec<f64>,
    rng: Pcg64Dxsm,
}

impl Population {
    /// The unmarked fish at the stationary density, the marked ones at the dock.
    fn new(seed: u64) -> Self {
        let mut rng = Pcg64Dxsm::from_seed(seed);
        let floor = u_floor();
        let (mut x, mut y) = (Vec::with_capacity(POPULATION), Vec::with_capacity(POPULATION));
        for i in 0..POPULATION {
            let (px, py) = if i < POPULATION_MARKED {
                let r = DOCK_R * rng.next_f64().sqrt();
                let a = std::f64::consts::TAU * rng.next_f64();
                (DOCK[0] + r * a.cos(), DOCK[1] + r * a.sin())
            } else {
                sample_habitat(&mut rng, floor)
            };
            x.push(px);
            y.push(py);
        }
        Self { x, y, rng }
    }

    fn step(&mut self, h: f64) {
        let kick = (2.0 * MOBILITY * TEMP * h).sqrt();
        for i in 0..self.x.len() {
            let (dx, dy) = drift(self.x[i], self.y[i]);
            let nx = self.x[i] + dx * h + kick * normal(&mut self.rng);
            let ny = self.y[i] + dy * h + kick * normal(&mut self.rng);
            (self.x[i], self.y[i]) = keep_in_lake(nx, ny);
        }
    }

    /// Fish and marked fish in each of the 3×3 regions (row 0 at the top).
    fn regions(&self) -> ([f64; 9], [f64; 9]) {
        let (mut all, mut marked) = ([0.0; 9], [0.0; 9]);
        for i in 0..self.x.len() {
            let col = (((self.x[i] + LAKE_A) / (2.0 * LAKE_A / 3.0)) as usize).min(2);
            let row = (((LAKE_B - self.y[i]) / (2.0 * LAKE_B / 3.0)) as usize).min(2);
            all[3 * row + col] += 1.0;
            if i < POPULATION_MARKED {
                marked[3 * row + col] += 1.0;
            }
        }
        (all, marked)
    }
}

/// S-8: two 3×3 maps recoloured in place, with a live number per tile.
struct Maps {
    seconds: f64,
    clock: Clock,
}

const TILE_W: f64 = 1.9;
const TILE_H: f64 = 1.4;

impl SceneConstruct for Maps {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = Kit::new()?;
        let low = hex("#16202C");
        let mut tiles = Vec::with_capacity(18);
        let mut centres = Vec::with_capacity(18);
        for (cx, title) in [(-3.4, "how many fish"), (3.4, "what fraction is marked")] {
            let t = kit.text(stage, title, 36.0, Palette::ink(), [cx, 2.75, 0.0])?;
            stage.add_to_scene(t)?;
            for row in 0..3 {
                for col in 0..3 {
                    let at = [
                        cx + (col as f64 - 1.0) * TILE_W,
                        -0.3 - (row as f64 - 1.0) * TILE_H,
                        0.0,
                    ];
                    let r = Rectangle::new()
                        .width(TILE_W - 0.08)
                        .height(TILE_H - 0.08)
                        .style(Style::default().fill(low, 1.0).stroke(Palette::faint(), 1.5, 1.0))
                        .build()?
                        .moved_to(at);
                    tiles.push(stage.add(r)?);
                    centres.push(at);
                }
            }
        }
        let mut trackers = Vec::with_capacity(18);
        for (i, &at) in centres.iter().enumerate() {
            let t = stage.add_value_tracker(0.0);
            kit.live_number(stage, t, usize::from(i >= 9), false, 30.0, Palette::ink(), at, ORIGIN)?;
            trackers.push(t);
        }
        let mut pop = Population::new(80);
        let clock = Rc::clone(&self.clock);
        let (mut t, mut cpu) = (0.0, (0, None));
        let mean = POPULATION as f64 / 9.0;
        let driver = tiles[0];
        stage.add_dt_updater(
            driver,
            move |s, _, dt| {
                if dt <= 0.0 {
                    return;
                }
                t += dt;
                tick(&clock, &mut cpu, t);
                for _ in 0..SUBSTEPS {
                    pop.step(dt);
                }
                let (all, marked) = pop.regions();
                for k in 0..9 {
                    let share = if all[k] > 0.0 { marked[k] / all[k] } else { 0.0 };
                    let dense = interpolate_color(low, Palette::blue(), (all[k] / mean / 2.5).min(1.0));
                    let hot = interpolate_color(low, Palette::hero(), (share / 0.35).min(1.0));
                    s.set_fill(tiles[k], Some(dense), Some(1.0), None, false);
                    s.set_fill(tiles[9 + k], Some(hot), Some(1.0), None, false);
                    let _ = s.set_tracker_value(trackers[k], all[k]);
                    let _ = s.set_tracker_value(trackers[9 + k], 100.0 * share);
                }
            },
            false,
        )?;
        wait_frames(stage, (self.seconds * f64::from(FPS)).round() as u64)
    }
}

fn video_options(path: PathBuf, res: (u32, u32)) -> Result<RenderOptions, Box<dyn std::error::Error>> {
    let mut o = encode::delivery_options(path, RenderFormat::Mp4, Quality::delivery())?;
    o.config.camera.resolution = res;
    o.config.camera.fps = FPS;
    o.config.camera.background_color = BACKGROUND.into();
    Ok(o)
}

/// The 1-minute load average (Linux `/proc/loadavg`, else macOS `sysctl`).
fn load() -> String {
    let text = std::fs::read_to_string("/proc/loadavg").ok().or_else(|| {
        std::process::Command::new("sysctl")
            .args(["-n", "vm.loadavg"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
    });
    text.and_then(|t| {
        t.split_whitespace()
            .find(|w| w.parse::<f64>().is_ok())
            .map(str::to_owned)
    })
    .unwrap_or_else(|| "?".into())
}

/// Render one scene (again on fmn's transient Darwin EPERM), then report
/// it and the fps of each of `windows` equal windows of its clock.
fn run<S: SceneConstruct>(
    out: &Path,
    name: &str,
    res: (u32, u32),
    seconds: f64,
    windows: usize,
    mut scene: impl FnMut(Clock) -> S,
) -> Result<(), Box<dyn std::error::Error>> {
    let opts = video_options(out.join(format!("{name}.mp4")), res)?;
    let clock: Clock = Rc::default();
    let before = load();
    let mut wall = 0.0;
    let (result, attempt) = encode::retrying(name, || {
        clock.borrow_mut().clear();
        let started = Instant::now();
        let r = fmn::render(&mut scene(Rc::clone(&clock)), opts.clone());
        wall = started.elapsed().as_secs_f64();
        r
    });
    let frames = result?.artifact.frame_count;
    println!(
        "{name}\t{}x{}\t{:.2} fps\t{frames} frames\t{wall:.1} s\tload {before} -> {}\tattempt {attempt}",
        res.0,
        res.1,
        frames as f64 / wall,
        load()
    );
    let stamps = clock.borrow();
    let width = seconds / windows as f64;
    let first_at = |t: f64| stamps.iter().position(|(s, _)| *s >= t - 1e-9);
    let mut line = format!("{name}\t{} frame updates; window fps:", stamps.len());
    for w in 0..windows {
        let a = first_at(w as f64 * width + 0.5 / f64::from(FPS)).unwrap_or(0);
        let b = first_at((w + 1) as f64 * width).unwrap_or(stamps.len().saturating_sub(1));
        let secs = stamps[b].1.duration_since(stamps[a].1).as_secs_f64();
        line += &format!(
            "  {:.0}-{:.0}s {:.2}",
            w as f64 * width,
            (w + 1) as f64 * width,
            (b - a) as f64 / secs
        );
    }
    println!("{line}");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let usage = "usage: spike_s2_fish OUT_DIR fish|grid [WxH] [SECONDS] [COUNTS] [LAYERS]";
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = PathBuf::from(args.first().ok_or(usage)?);
    let mode = args.get(1).ok_or(usage)?.as_str();
    let (res, seconds) = match mode {
        "fish" => ((1920, 1080), 10.0),
        "grid" => ((960, 540), 80.0),
        _ => return Err(usage.into()),
    };
    let res = match args.get(2) {
        Some(r) => {
            let (w, h) = r.split_once('x').ok_or("WxH")?;
            (w.parse()?, h.parse()?)
        }
        None => res,
    };
    let seconds: f64 = args.get(3).map_or(Ok(seconds), |s| s.parse())?;
    let counts = args
        .get(4)
        .map_or("0,600,1000,1500", String::as_str)
        .split(',')
        .map(str::parse)
        .collect::<Result<Vec<usize>, _>>()?;
    let layers: Vec<&str> = args.get(5).map_or("lines,glow", String::as_str).split(',').collect();
    if let Some(bad) = layers.iter().find(|l| !["lines", "glow", "halo", "none"].contains(l)) {
        return Err(format!("unknown layer {bad} (lines, glow, halo, none)").into());
    }
    let lines = layers.contains(&"lines");
    let glow = if layers.contains(&"glow") {
        Glow::Strokes
    } else if layers.contains(&"halo") {
        Glow::Fills
    } else {
        Glow::None
    };
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    std::fs::create_dir_all(&out)?;
    if mode == "grid" {
        return run(&out, "grid", res, seconds, 8, |clock| Maps { seconds, clock });
    }
    for count in counts {
        run(&out, &format!("fish_{count}"), res, seconds, 2, |clock| Lake {
            count,
            seconds,
            lines,
            glow,
            clock,
        })?;
    }
    Ok(())
}
