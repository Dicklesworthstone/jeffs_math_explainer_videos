//! Spikes S-4 and S-6 (plan §6.6): Video A's 4,845 netfuls streaming into
//! five bins (chapter 04_4) and Video C's progress-square path sets morphing
//! from 28 orders to 322 (chapter 05_6), timed through the kit's delivery
//! encoder.
//!
//! ```text
//! cargo run --release -p explainer_kit --example spike_s4_s6 -- OUT_DIR s4 [--res WxH] [--preset P] [--glyph dots|outline]
//! cargo run --release -p explainer_kit --example spike_s4_s6 -- OUT_DIR s6 --paths DIR [--res WxH] [--preset P] [--set strands|ribbons] [--fresh yes|no]
//! ```
//!
//! OUT_DIR is created and must not exist; the render is `OUT_DIR/s4.mp4` or
//! `OUT_DIR/s6.mp4` at WxH (default 1920x1080) and 60 fps, x264 CRF 16 at
//! preset P (default slow, the corpus's delivery setting).
//!
//! - **s4** (≈ 8 s): every 4-fish netful of a 20-fish lake with 5 marked
//!   fish, enumerated and counted by marked fish k = 0..4 (checked against
//!   C(5,k)·C(15,4−k) and the plan's A-F2). Each netful is one glyph of four
//!   dots, its marked fish gold. `--glyph dots` (default): a group of four
//!   `Dot`s, which keep their shape tag and so the renderer's analytic disc
//!   kernel; `--glyph outline`: one multi-subpath outline per colour (fewer
//!   records, general curve fill). All 4,845 start in a block in
//!   lexicographic order and fly in a shuffled order into the bins along
//!   precomputed quadratic arcs, moved by ONE dt updater that only shifts
//!   (placement) the glyphs in flight, each a family of at most five
//!   members; no larger group is ever transformed. Five live counts follow
//!   the arrivals.
//! - **s6** (≈ 10 s): the exact schedule sets of the bakery research model
//!   (two customers, one visit each), exported from
//!   `docs/research/bakery_algorithm/scripts/tree_count.py` as one
//!   `path <steps> <both-inside>` line per schedule into
//!   `DIR/paths_noflag.txt` and `DIR/paths_flag.txt` (the export script
//!   lives outside the repository). Each schedule is a monotone staircase in
//!   the progress square; the 28 no-flag staircases grow (the 4 through the
//!   red "both inside" square in red), the lattice morphs into the flag
//!   program's (the four no-flag steps slide to their places among the
//!   seven, the three new steps appear), then the 322 flag staircases grow
//!   and none touches the red square; a zoom about the red square (placement
//!   scaling of each piece, never a group) and back. Growing a strand
//!   rewrites its points and so retires its polyline shape hint; `--fresh
//!   yes` (default) swaps pristine strands in once they have grown.
//!   `--set ribbons` draws the 322 instead as one rectangle per lattice edge,
//!   as thick as the number of staircases through it, thickening as they
//!   arrive (the plan's fallback: the set and its count, not each path).
//!
//! One line per render on stdout (mode, resolution, mobject records at the
//! peak, frames, wall seconds, fps), then one line per phase and notes.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use explainer_kit::encode::{self, Quality};
use explainer_kit::{Kit, Palette, ink_style, play, stroke_style, thousands, v};
use explainer_score::rng::SplitMix64;
use fmn::animation::UpdateFromFunc;
use fmn::geometry::QuadPath;
use fmn::prelude::*;

// ------------------------------------------------------------ shared plumbing

/// What a render reports back to `main`: records at the peak and phases.
#[derive(Default)]
struct Log {
    records: usize,
    /// (phase, frames, wall seconds)
    phases: Vec<(String, i64, f64)>,
    notes: Vec<String>,
}

type Shared = Rc<RefCell<Log>>;

struct Phase {
    name: &'static str,
    frame: i64,
    started: Instant,
}

fn begin(stage: &Stage<'_>, name: &'static str) -> Phase {
    Phase {
        name,
        frame: stage.scene().time().frames(),
        started: Instant::now(),
    }
}

fn end(stage: &Stage<'_>, phase: Phase, log: &Shared) {
    let frames = stage.scene().time().frames() - phase.frame;
    log.borrow_mut()
        .phases
        .push((phase.name.to_owned(), frames, phase.started.elapsed().as_secs_f64()));
}

/// Every arena record in the scene's draw list (each root's family).
fn records(stage: &Stage<'_>) -> usize {
    let roots = stage.scene().mobjects().to_vec();
    roots.iter().map(|&m| stage.family(m).len()).sum()
}

fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(SceneError::InvalidConfig(Box::leak(message.into_boxed_str())))
}

fn smooth(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ------------------------------------------------------------------- S-4

const LAKE: usize = 20;
const NET: usize = 4;
/// The toy lake's marked fish (any five; scattered so glyph patterns vary).
const MARKED_FISH: [usize; 5] = [3, 7, 10, 14, 18];
/// A-F2 (plan §9.5): netfuls holding k = 0..4 marked fish.
const A_F2: [usize; 5] = [1_365, 2_275, 1_050, 150, 5];
/// One glyph's cell (scene units): 8.9 px at 1080p, 17.8 px at 4K.
const CELL: f64 = 0.066;
const SRC_COLS: usize = 190;
const SRC_TOP: f64 = 3.35;
const BIN_COLS: usize = 36;
const BIN_GAP: f64 = 0.36;
const BIN_FLOOR: f64 = -2.95;
/// Departures spread over SPREAD seconds; each flight takes FLIGHT.
const SPREAD: f64 = 3.8;
const FLIGHT: f64 = 0.8;

fn binom(n: usize, k: usize) -> usize {
    (0..k).fold(1, |acc, i| acc * (n - i) / (i + 1))
}

/// Every netful, in lexicographic order.
fn netfuls() -> Vec<[usize; NET]> {
    let mut out = Vec::new();
    for a in 0..LAKE {
        for b in a + 1..LAKE {
            for c in b + 1..LAKE {
                for d in c + 1..LAKE {
                    out.push([a, b, c, d]);
                }
            }
        }
    }
    out
}

fn marked_count(net: &[usize; NET]) -> usize {
    net.iter().filter(|f| MARKED_FISH.contains(f)).count()
}

/// The netful counts by k, enumerated and checked against the closed form
/// and the plan's facts ledger.
fn bin_counts(nets: &[[usize; NET]]) -> Result<[usize; 5], String> {
    let mut counts = [0usize; 5];
    for net in nets {
        counts[marked_count(net)] += 1;
    }
    let m = MARKED_FISH.len();
    let closed: Vec<usize> = (0..=NET).map(|k| binom(m, k) * binom(LAKE - m, NET - k)).collect();
    if nets.len() != binom(LAKE, NET) || counts[..] != closed[..] || counts != A_F2 {
        return Err(format!(
            "netful counts {counts:?} (total {}) disagree with C(5,k)·C(15,4−k) = {closed:?} or A-F2 {A_F2:?}",
            nets.len()
        ));
    }
    Ok(counts)
}

fn source_cell(i: usize) -> Vec3 {
    let (r, c) = (i / SRC_COLS, i % SRC_COLS);
    v(
        -(SRC_COLS as f64) * CELL / 2.0 + (c as f64 + 0.5) * CELL,
        SRC_TOP - (r as f64 + 0.5) * CELL,
    )
}

fn bin_width() -> f64 {
    BIN_COLS as f64 * CELL
}

fn bin_left(k: usize) -> f64 {
    let total = 5.0 * bin_width() + 4.0 * BIN_GAP;
    -total / 2.0 + k as f64 * (bin_width() + BIN_GAP)
}

fn bin_slot(k: usize, j: usize) -> Vec3 {
    v(
        bin_left(k) + ((j % BIN_COLS) as f64 + 0.5) * CELL,
        BIN_FLOOR + ((j / BIN_COLS) as f64 + 0.5) * CELL,
    )
}

/// How a glyph's four dots are built.
#[derive(Clone, Copy, PartialEq, Eq)]
enum GlyphKind {
    /// Four `Dot`s in a group (five records): each keeps its `Dot` shape tag,
    /// so the renderer fills it with its analytic disc kernel.
    Dots,
    /// One multi-subpath outline per colour (one to three records): fewer
    /// records, but general curve fill (16 quadratics per dot).
    Outline,
}

/// One netful: four dots in a 2×2 cell, its marked fish gold, the dots of one
/// glyph closer to each other than to the next glyph's.
fn glyph(net: &[usize; NET], center: Vec3, kind: GlyphKind) -> VMobject {
    let (a, r) = (0.2 * CELL, 0.16 * CELL);
    let offsets = [(-a, a), (a, a), (-a, -a), (a, -a)];
    let color = |fish: &usize| {
        if MARKED_FISH.contains(fish) {
            Palette::hero()
        } else {
            Palette::blue()
        }
    };
    let dot = |fish: &usize, (dx, dy): (f64, f64)| {
        Dot::new()
            .point(v(center[0] + dx, center[1] + dy))
            .radius(r)
            .color(color(fish))
            .build()
    };
    if kind == GlyphKind::Dots {
        return v_group(net.iter().zip(offsets).map(|(fish, o)| dot(fish, o)));
    }
    let (mut gold, mut blue) = (QuadPath::new(), QuadPath::new());
    for (fish, o) in net.iter().zip(offsets) {
        let path = if MARKED_FISH.contains(fish) {
            &mut gold
        } else {
            &mut blue
        };
        path.add_subpath(dot(fish, o).points())
            .expect("a dot is an odd point run");
    }
    let part = |path: &QuadPath, color: Srgb| VMobject::from_path(path).with_style(ink_style(color));
    match (gold.has_points(), blue.has_points()) {
        (true, true) => v_group([part(&gold, Palette::hero()), part(&blue, Palette::blue())]),
        (true, false) => part(&gold, Palette::hero()),
        _ => part(&blue, Palette::blue()),
    }
}

struct Flight {
    mob: Mob,
    from: Vec3,
    mid: Vec3,
    to: Vec3,
    depart: f64,
    bin: usize,
    at: Vec3,
}

impl Flight {
    fn point(&self, u: f64) -> Vec3 {
        let (a, b, c) = ((1.0 - u) * (1.0 - u), 2.0 * u * (1.0 - u), u * u);
        [
            a * self.from[0] + b * self.mid[0] + c * self.to[0],
            a * self.from[1] + b * self.mid[1] + c * self.to[1],
            0.0,
        ]
    }
}

struct Netfuls {
    kind: GlyphKind,
    log: Shared,
}

impl SceneConstruct for Netfuls {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let setup = begin(stage, "setup (construct, 0 frames)");
        let kit = Kit::new()?;
        let nets = netfuls();
        let counts = bin_counts(&nets).map_err(fail)?;
        let total = nets.len();

        // the block of all netfuls, lexicographic, added in one batch
        let mut glyphs = Vec::with_capacity(total);
        for (i, net) in nets.iter().enumerate() {
            glyphs.push(stage.arena_mut().add(glyph(net, source_cell(i), self.kind)));
        }
        stage.add_many_to_scene(&glyphs)?;

        // bins: open boxes, labels, live counts
        let rows_max = counts.iter().max().copied().unwrap_or(0).div_ceil(BIN_COLS);
        let wall = rows_max as f64 * CELL + 0.12;
        let mut trackers = Vec::with_capacity(5);
        for k in 0..=NET {
            let (l, r, pad) = (bin_left(k), bin_left(k) + bin_width(), 0.04);
            let corners = [
                v(l - pad, BIN_FLOOR + wall),
                v(l - pad, BIN_FLOOR - pad),
                v(r + pad, BIN_FLOOR - pad),
                v(r + pad, BIN_FLOOR + wall),
            ];
            stage.add(polyline(corners).style(stroke_style(Palette::faint(), 2.0)).build())?;
            let cx = (l + r) / 2.0;
            let label = kit.text(
                stage,
                &format!("{k} marked"),
                24.0,
                Palette::dim(),
                v(cx, BIN_FLOOR - 0.3),
            )?;
            stage.add_to_scene(label)?;
            let t = stage.add_value_tracker(0.0);
            kit.live_count(stage, t, 30.0, Palette::ink(), v(cx, BIN_FLOOR - 0.7))?;
            trackers.push(t);
        }
        let title = format!("{} netfuls", thousands(total as u32));
        let title = kit.text(stage, &title, 30.0, Palette::ink(), v(0.0, 3.72))?;
        stage.add_to_scene(title)?;
        self.log.borrow_mut().records = records(stage);

        // flights: a shuffled departure order; each bin fills bottom-up in
        // the order its glyphs depart (equal flight times keep that order)
        let mut order: Vec<usize> = (0..total).collect();
        let mut rng = SplitMix64::new(0x5EED_0404);
        for i in (1..total).rev() {
            order.swap(i, (rng.next_u64() % (i as u64 + 1)) as usize);
        }
        let mut filled = [0usize; 5];
        let flights: Vec<Flight> = order
            .iter()
            .enumerate()
            .map(|(j, &i)| {
                let bin = marked_count(&nets[i]);
                let (from, to) = (source_cell(i), bin_slot(bin, filled[bin]));
                filled[bin] += 1;
                Flight {
                    mob: glyphs[i],
                    from,
                    mid: v((from[0] + to[0]) / 2.0, from[1] + 0.5),
                    to,
                    depart: j as f64 / total as f64 * SPREAD,
                    bin,
                    at: from,
                }
            })
            .collect();
        end(stage, setup, &self.log);

        let p = begin(stage, "static block");
        stage.wait(0.6)?;
        end(stage, p, &self.log);

        // one controller, one dt updater, placement-only motion
        let arrived = Rc::new(RefCell::new([0usize; 5]));
        let ctl = stage.add(VMobject::new())?;
        {
            let arrived = Rc::clone(&arrived);
            let mut flights = flights;
            let (mut clock, mut next, mut active) = (0.0f64, 0usize, Vec::<usize>::new());
            stage.add_dt_updater(
                ctl,
                move |s, _, dt| {
                    clock += dt;
                    while next < flights.len() && flights[next].depart <= clock {
                        active.push(next);
                        next += 1;
                    }
                    let mut still = Vec::with_capacity(active.len());
                    for &i in &active {
                        let f = &mut flights[i];
                        let tau = ((clock - f.depart) / FLIGHT).min(1.0);
                        let p = f.point(smooth(tau));
                        s.shift(f.mob, [p[0] - f.at[0], p[1] - f.at[1], 0.0]);
                        f.at = p;
                        if tau >= 1.0 {
                            let mut a = arrived.borrow_mut();
                            a[f.bin] += 1;
                            s.set_tracker_value(trackers[f.bin], a[f.bin] as f64)
                                .expect("trackers live for the whole scene");
                        } else {
                            still.push(i);
                        }
                    }
                    active = still;
                },
                false,
            )?;
        }
        let p = begin(stage, "stream (dt updater)");
        stage.wait(SPREAD + FLIGHT + 2.0 / 60.0)?;
        end(stage, p, &self.log);
        stage.clear_updaters(ctl, true);
        if *arrived.borrow() != counts {
            return Err(fail(format!("arrivals {:?} != counts {counts:?}", arrived.borrow())));
        }

        let p = begin(stage, "hold (static)");
        stage.wait(2.55)?;
        end(stage, p, &self.log);
        Ok(())
    }
}

// ------------------------------------------------------------------- S-6

/// One exported schedule set.
#[derive(Clone)]
struct PathSet {
    steps: (usize, usize),
    /// The lattice point where both customers are inside.
    inside: (usize, usize),
    labels: Vec<String>,
    /// No-flag only: progress k ↦ the flag program's progress after the same op.
    embed: Vec<usize>,
    /// (one char per step: the customer that moved; both inside?)
    paths: Vec<(String, bool)>,
}

fn pair(words: &[&str]) -> Result<(usize, usize), String> {
    match words {
        [a, b] => Ok((
            a.parse().map_err(|e| format!("{a}: {e}"))?,
            b.parse().map_err(|e| format!("{b}: {e}"))?,
        )),
        _ => Err(format!("expected two numbers, got {words:?}")),
    }
}

fn load_paths(file: &Path) -> Result<PathSet, String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let mut set = PathSet {
        steps: (0, 0),
        inside: (0, 0),
        labels: Vec::new(),
        embed: Vec::new(),
        paths: Vec::new(),
    };
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words[0] {
            "steps" => set.steps = pair(&words[1..])?,
            "inside" => set.inside = pair(&words[1..])?,
            "labels" => set.labels = words[1..].iter().map(|s| (*s).to_owned()).collect(),
            "embed" => {
                set.embed = words[1..]
                    .iter()
                    .map(|s| s.parse().map_err(|e| format!("{s}: {e}")))
                    .collect::<Result<_, _>>()?;
            }
            "path" if words.len() == 3 => set.paths.push((words[1].to_owned(), words[2] == "1")),
            _ => return Err(format!("{}: unreadable line {line:?}", file.display())),
        }
    }
    set.paths.sort();
    Ok(set)
}

/// The lattice points a schedule visits, from (0, 0).
fn lattice(seq: &str) -> Vec<(usize, usize)> {
    let mut at = (0, 0);
    let mut out = vec![at];
    for c in seq.chars() {
        if c == '0' {
            at.0 += 1;
        } else {
            at.1 += 1;
        }
        out.push(at);
    }
    out
}

/// The staircase's corners: start, every turn, end.
fn corners(seq: &str) -> Vec<(usize, usize)> {
    let pts = lattice(seq);
    let b = seq.as_bytes();
    let mut out = vec![pts[0]];
    for k in 1..b.len() {
        if b[k] != b[k - 1] {
            out.push(pts[k]);
        }
    }
    out.push(pts[b.len()]);
    out
}

/// The schedule sets' own invariants, and C-F3 (plan §11.5).
fn check_sets(plain: &PathSet, flagged: &PathSet) -> Result<(), String> {
    for (name, set, want) in [("no flag", plain, (28, 4)), ("flag", flagged, (322, 0))] {
        let n = set.steps.0 + set.steps.1;
        let distinct: HashSet<&str> = set.paths.iter().map(|(s, _)| s.as_str()).collect();
        if distinct.len() != set.paths.len() {
            return Err(format!("{name}: repeated schedules"));
        }
        for (s, bad) in &set.paths {
            let ones = s.chars().filter(|&c| c == '1').count();
            if s.len() != n || ones != set.steps.1 || s.chars().any(|c| c != '0' && c != '1') {
                return Err(format!(
                    "{name}: {s} is not a {}×{} staircase",
                    set.steps.0, set.steps.1
                ));
            }
            if *bad != lattice(s).contains(&set.inside) {
                return Err(format!(
                    "{name}: {s}'s both-inside flag disagrees with its lattice points"
                ));
            }
        }
        let bad = set.paths.iter().filter(|(_, b)| *b).count();
        if (set.paths.len(), bad) != want {
            return Err(format!(
                "{name}: {} schedules, {bad} both inside; C-F3 says {want:?}",
                set.paths.len()
            ));
        }
        if set.labels.len() != set.steps.0 {
            return Err(format!("{name}: {} labels for {} steps", set.labels.len(), set.steps.0));
        }
    }
    let e = &plain.embed;
    if e.len() != plain.steps.0 + 1
        || e[0] != 0
        || e.windows(2).any(|w| w[0] >= w[1])
        || e[plain.steps.0] != flagged.steps.0
        || e[plain.inside.0] != flagged.inside.0
    {
        return Err(format!(
            "embedding {e:?} does not map the no-flag lattice into the flag lattice"
        ));
    }
    for (i, label) in plain.labels.iter().enumerate() {
        if flagged.labels[e[i + 1] - 1] != *label {
            return Err(format!("step {label} embeds onto {}", flagged.labels[e[i + 1] - 1]));
        }
    }
    Ok(())
}

const SIDE: f64 = 5.4;
const SQ: Vec3 = [-2.4, -0.1, 0.0];
const LABEL_Y: f64 = -3.05;
const PANEL_X: f64 = 1.6;

/// Progress fractions → scene.
fn sq(fx: f64, fy: f64) -> Vec3 {
    v(SQ[0] + (fx - 0.5) * SIDE, SQ[1] + (fy - 0.5) * SIDE)
}

/// A staircase through `pts` (lattice → fractions by `map`), pushed `offset`
/// along the anti-diagonal so a bundle of strands fans out instead of
/// collapsing onto shared edges.
fn strand(pts: &[(usize, usize)], map: &dyn Fn(usize) -> f64, offset: f64, style: Style) -> VMobject {
    let d = offset * FRAC_1_SQRT_2;
    polyline(pts.iter().map(|&(i, j)| {
        let p = sq(map(i), map(j));
        v(p[0] + d, p[1] - d)
    }))
    .style(style)
    .build()
}

/// The bundle offset of strand `r` of `n`, lexicographic order: right-first
/// (lower-right) staircases go to the lower right.
fn offset(r: usize, n: usize, width: f64) -> f64 {
    (0.5 - (r as f64 + 0.5) / n as f64) * width
}

fn grid_line(f: f64, vertical: bool) -> fmn::Result<VMobject> {
    let (a, b) = if vertical {
        (sq(f, 0.0), sq(f, 1.0))
    } else {
        (sq(0.0, f), sq(1.0, f))
    };
    Ok(Line::new(a, b).style(stroke_style(Palette::faint(), 1.5)).build()?)
}

fn red_square(fx: f64, fy: f64, side: f64) -> VMobject {
    let red = Palette::red();
    Square::new()
        .side_length(side)
        .style(Style::default().fill(red, 0.35).stroke(red, 2.0, 1.0))
        .build()
        .moved_to(sq(fx, fy))
}

/// How the 322 staircases are drawn.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SetKind {
    /// One stroked polyline per staircase, fanned out in bundles.
    Strands,
    /// One filled rectangle per lattice edge, as thick as the number of
    /// staircases through it (at most 112 records; no individual paths).
    Ribbons,
}

/// The flag set as edge ribbons that thicken as the staircases arrive (in a
/// shuffled order, over 2.2 s): each lattice edge one rectangle, stretched by
/// placement to its running multiplicity, added when its first staircase
/// arrives. Returns the ribbons.
fn ribbons(
    stage: &mut Stage<'_>,
    set: &PathSet,
    map: &dyn Fn(usize) -> f64,
    width: f64,
    log: &Shared,
) -> fmn::Result<Vec<Mob>> {
    let n = set.paths.len();
    let mut arrival: Vec<usize> = (0..n).collect();
    let mut rng = SplitMix64::new(0x5EED_0506);
    for i in (1..n).rev() {
        arrival.swap(i, (rng.next_u64() % (i as u64 + 1)) as usize);
    }
    // each edge's staircases, as arrival ranks, sorted
    let mut users: std::collections::BTreeMap<((usize, usize), bool), Vec<usize>> = Default::default();
    for (r, (s, _)) in set.paths.iter().enumerate() {
        for w in lattice(s).windows(2) {
            users.entry((w[0], w[1].0 > w[0].0)).or_default().push(arrival[r]);
        }
    }
    let m_max = users.values().map(Vec::len).max().unwrap_or(1);
    let thick = move |m: usize| (width * m as f64 / m_max as f64).max(0.012);
    let cell = SIDE / set.steps.0 as f64;
    struct Ribbon {
        mob: Mob,
        center: Vec3,
        dim: usize,
        full: f64,
        ranks: Vec<usize>,
        factor: f64,
        shown: bool,
    }
    let mut all = Vec::with_capacity(users.len());
    for (((i, j), horizontal), mut ranks) in users {
        ranks.sort_unstable();
        let a = sq(map(i), map(j));
        let (center, dim, w, h) = if horizontal {
            (v(a[0] + cell / 2.0, a[1]), 1, cell, thick(ranks.len()))
        } else {
            (v(a[0], a[1] + cell / 2.0), 0, thick(ranks.len()), cell)
        };
        let teal = Palette::teal();
        let r = Rectangle::new()
            .width(w)
            .height(h)
            .style(Style::default().fill(teal, 0.8).stroke(teal, 0.0, 0.0))
            .build()?
            .moved_to(center);
        let full = thick(ranks.len());
        let mob = stage.arena_mut().add(r);
        let factor = thick(1) / full;
        stage.stretch_about(mob, factor, dim, Some(center), None);
        all.push(Ribbon {
            mob,
            center,
            dim,
            full,
            ranks,
            factor,
            shown: false,
        });
    }
    let mobs: Vec<Mob> = all.iter().map(|r| r.mob).collect();
    log.borrow_mut().notes.push(format!(
        "ribbons: {} lattice edges used, widest carries {m_max}",
        mobs.len()
    ));
    let anchor = stage.add(VMobject::new())?;
    let grow = UpdateFromFunc::new_alpha(anchor, move |s, _, alpha| {
        let c = (alpha * n as f64).round() as usize;
        for r in &mut all {
            let m = r.ranks.partition_point(|&k| k < c);
            if m == 0 {
                continue;
            }
            if !r.shown {
                s.add_to_scene(r.mob).expect("ribbons are live");
                r.shown = true;
            }
            let target = thick(m) / r.full;
            if target != r.factor {
                s.stretch_about(r.mob, target / r.factor, r.dim, Some(r.center), None);
                r.factor = target;
            }
        }
    });
    stage.play_prepared_with(
        vec![Box::new(grow)],
        PlayOverrides {
            run_time: Some(2.2),
            rate_func: Some(RateFunc::linear()),
            ..PlayOverrides::default()
        },
    )?;
    Ok(mobs)
}

struct PathMorph {
    plain: PathSet,
    flagged: PathSet,
    set: SetKind,
    /// Swap pristine strands in after the growth (see `construct`).
    fresh: bool,
    log: Shared,
}

impl SceneConstruct for PathMorph {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = Kit::new()?;
        let (pa, pb) = (&self.plain, &self.flagged);
        let (na, nb) = (pa.steps.0 as f64, pb.steps.0 as f64);
        let embed = pa.embed.clone();
        let map_a = move |i: usize| i as f64 / na;
        let map_ab = {
            let e = embed.clone();
            move |i: usize| e[i] as f64 / nb
        };
        let map_b = move |i: usize| i as f64 / nb;
        let mut zoom_set: Vec<Mob> = Vec::new();

        // the empty square: border, the no-flag lattice, red square, labels
        let p = begin(stage, "square in");
        let border = Square::new()
            .side_length(SIDE)
            .style(stroke_style(Palette::dim(), 2.0))
            .build()
            .moved_to(SQ);
        let border = stage.arena_mut().add(border);
        let mut fades: Vec<Mob> = vec![border];
        let mut grid_a = Vec::new();
        for i in 1..pa.steps.0 {
            for vertical in [true, false] {
                let m = stage.arena_mut().add(grid_line(map_a(i), vertical)?);
                grid_a.push((m, i, vertical));
                fades.push(m);
            }
        }
        let (ia, ja) = pa.inside;
        let red = stage
            .arena_mut()
            .add(red_square(map_a(ia), map_a(ja), 0.42 * SIDE / na));
        fades.push(red);
        let mut labels_a = Vec::new();
        for (i, s) in pa.labels.iter().enumerate() {
            let at = v(sq((i as f64 + 0.5) / na, 0.0)[0], LABEL_Y);
            let m = kit.text(stage, s, 20.0, Palette::dim(), at)?;
            labels_a.push(m);
            fades.push(m);
        }
        let x_title = kit.text(
            stage,
            "customer 1's progress",
            24.0,
            Palette::ink(),
            v(SQ[0], LABEL_Y - 0.42),
        )?;
        let y_title = kit.text(
            stage,
            "customer 2's progress",
            24.0,
            Palette::ink(),
            v(SQ[0] - SIDE / 2.0 - 0.32, SQ[1]),
        )?;
        stage.rotate(y_title, FRAC_PI_2, OUT, None, None);
        fades.extend([x_title, y_title]);
        let mut v_fade = Vec::new();
        for &m in &fades {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            v_fade.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            v_fade,
            PlayOverrides {
                run_time: Some(0.5),
                ..PlayOverrides::default()
            },
        )?;
        zoom_set.extend(&fades);
        end(stage, p, &self.log);

        // the 28 no-flag staircases grow; the both-inside ones last, in red
        let p = begin(stage, "28 grow");
        let width_a = 0.5;
        let (good, bad): (Vec<_>, Vec<_>) = pa.paths.iter().enumerate().partition(|(_, (_, b))| !*b);
        let mut strands_a = Vec::new();
        for (r, (s, is_bad)) in good.into_iter().chain(bad) {
            let style = if *is_bad {
                stroke_style(Palette::red(), 3.5)
            } else {
                Style::default()
                    .fill(Palette::blue(), 0.0)
                    .stroke(Palette::blue(), 2.5, 0.85)
            };
            let pts = corners(s);
            let off = offset(r, pa.paths.len(), width_a);
            let m = stage.arena_mut().add(strand(&pts, &map_a, off, style));
            strands_a.push((m, pts, off, style));
        }
        let grow: Vec<Box<dyn Animation>> = strands_a
            .iter()
            .map(|(m, ..)| Box::new(show_creation(*m)) as Box<dyn Animation>)
            .collect();
        let lag = (1.6 / 0.5 - 1.0) / (grow.len() as f64 - 1.0);
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), grow, lag)?.with_run_time(1.6);
        stage.play(g)?;
        let n_bad_a = pa.paths.iter().filter(|(_, b)| *b).count();
        let head_a = kit.text_left(stage, "without flags", 30.0, Palette::ink(), v(PANEL_X, 1.75))?;
        let count_a = format!("{} orders", thousands(pa.paths.len() as u32));
        let count_a = kit.text_left(stage, &count_a, 36.0, Palette::blue(), v(PANEL_X, 1.2))?;
        let bad_a = format!("{} put both inside", thousands(n_bad_a as u32));
        let bad_a = kit.text_left(stage, &bad_a, 30.0, Palette::red(), v(PANEL_X, 0.68))?;
        play!(stage, 0.5; write(stage.arena(), head_a), write(stage.arena(), count_a), write(stage.arena(), bad_a));
        stage.wait(0.3)?;
        end(stage, p, &self.log);

        // the morph: the four steps slide to their places among the seven
        let p = begin(stage, "lattice morph");
        let mut morph: Vec<Box<dyn Animation>> = Vec::new();
        for &(m, i, vertical) in &grid_a {
            let target = stage.arena_mut().add(grid_line(map_ab(i), vertical)?);
            morph.push(Box::new(Transform::new(m, target)));
        }
        let (ib, jb) = pb.inside;
        let red_b = stage.arena_mut().add(red_square(map_b(ib), map_b(jb), 0.5 * SIDE / nb));
        morph.push(Box::new(Transform::new(red, red_b)));
        for (m, pts, off, style) in &strands_a {
            let target = stage.arena_mut().add(strand(pts, &map_ab, *off, *style));
            morph.push(Box::new(Transform::new(*m, target)));
        }
        for (i, &m) in labels_a.iter().enumerate() {
            let at = v(sq((embed[i + 1] as f64 - 0.5) / nb, 0.0)[0], LABEL_Y);
            let a = m.animate().move_to(at, ORIGIN)?;
            morph.push(stage.prepare(a)?);
        }
        let covered = embed.clone();
        for i in 1..pb.steps.0 {
            if covered.contains(&i) {
                continue;
            }
            for vertical in [true, false] {
                let m = stage.arena_mut().add(grid_line(map_b(i), vertical)?);
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                morph.push(stage.prepare(a)?);
                zoom_set.push(m);
            }
        }
        for (i, s) in pb.labels.iter().enumerate() {
            if covered.contains(&(i + 1)) {
                continue;
            }
            let at = v(sq((i as f64 + 0.5) / nb, 0.0)[0], LABEL_Y);
            let m = kit.text(stage, s, 20.0, Palette::gold(), at)?;
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            morph.push(stage.prepare(a)?);
            zoom_set.push(m);
        }
        stage.play_prepared_with(
            morph,
            PlayOverrides {
                run_time: Some(1.4),
                ..PlayOverrides::default()
            },
        )?;
        let mut outs: Vec<Box<dyn Animation>> = Vec::new();
        for (m, ..) in &strands_a {
            let a = fade_out(stage.arena_mut(), *m, ORIGIN, 1.0)?;
            outs.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            outs,
            PlayOverrides {
                run_time: Some(0.4),
                ..PlayOverrides::default()
            },
        )?;
        end(stage, p, &self.log);

        // the 322 flag staircases grow; none touches the red square
        let p = begin(stage, "322 grow");
        let width_b = 0.42;
        let set_b = match self.set {
            SetKind::Strands => {
                let style_b = Style::default()
                    .fill(Palette::teal(), 0.0)
                    .stroke(Palette::teal(), 1.4, 0.5);
                let mut strands_b = Vec::new();
                for (r, (s, _)) in pb.paths.iter().enumerate() {
                    let off = offset(r, pb.paths.len(), width_b);
                    strands_b.push(stage.arena_mut().add(strand(&corners(s), &map_b, off, style_b)));
                }
                let grow: Vec<Box<dyn Animation>> = strands_b
                    .iter()
                    .map(|&m| Box::new(show_creation(m)) as Box<dyn Animation>)
                    .collect();
                let lag = (2.2 / 0.5 - 1.0) / (grow.len() as f64 - 1.0);
                let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), grow, lag)?.with_run_time(2.2);
                stage.play(g)?;
                // ShowPartial rewrote every grown strand's points, which retires
                // its polyline shape hint for good (general quadratic stroke
                // distance from then on). With `fresh`, swap in pristine
                // strands, hints intact.
                let hinted =
                    |stage: &Stage<'_>, set: &[Mob]| set.iter().filter(|&&m| stage.primitive_hint(m).is_some()).count();
                let note = format!(
                    "polyline hints after growth: {}/{}",
                    hinted(stage, &strands_b),
                    strands_b.len()
                );
                self.log.borrow_mut().notes.push(note);
                if self.fresh {
                    stage.remove_many_from_scene(&strands_b);
                    for (r, (s, _)) in pb.paths.iter().enumerate() {
                        let off = offset(r, pb.paths.len(), width_b);
                        strands_b[r] = stage.arena_mut().add(strand(&corners(s), &map_b, off, style_b));
                    }
                    stage.add_many_to_scene(&strands_b)?;
                    let note = format!(
                        "polyline hints after the swap: {}/{}",
                        hinted(stage, &strands_b),
                        strands_b.len()
                    );
                    self.log.borrow_mut().notes.push(note);
                }
                strands_b
            }
            SetKind::Ribbons => ribbons(stage, pb, &map_b, width_b, &self.log)?,
        };
        let n_bad_b = pb.paths.iter().filter(|(_, b)| *b).count();
        let head_b = kit.text_left(stage, "with flags", 30.0, Palette::ink(), v(PANEL_X, -0.55))?;
        let count_b = format!("{} orders", thousands(pb.paths.len() as u32));
        let count_b = kit.text_left(stage, &count_b, 36.0, Palette::teal(), v(PANEL_X, -1.1))?;
        let bad_b = format!("{} put both inside", thousands(n_bad_b as u32));
        let bad_b = kit.text_left(stage, &bad_b, 30.0, Palette::green(), v(PANEL_X, -1.62))?;
        play!(stage, 0.5; write(stage.arena(), head_b), write(stage.arena(), count_b), write(stage.arena(), bad_b));
        zoom_set.extend(&set_b);
        self.log.borrow_mut().records = records(stage);
        end(stage, p, &self.log);

        // the zoom: every piece of the square scaled about the red square
        // by placement, one small family at a time (never a group)
        let p = begin(stage, "zoom in, out");
        let center = sq(map_b(ib), map_b(jb));
        let scale = Rc::new(Cell::new(1.0f64));
        for (from, to, run, hold) in [(1.0, 2.2, 1.2, 0.4), (2.2, 1.0, 1.1, 0.3)] {
            let (set, scale) = (zoom_set.clone(), Rc::clone(&scale));
            let zoom = UpdateFromFunc::new_alpha(border, move |s, _, alpha| {
                let f = from + (to - from) * alpha;
                let ratio = f / scale.get();
                for &m in &set {
                    s.scale_about(m, ratio, Some(center), None);
                }
                scale.set(f);
            });
            stage.play_prepared_with(
                vec![Box::new(zoom)],
                PlayOverrides {
                    run_time: Some(run),
                    ..PlayOverrides::default()
                },
            )?;
            stage.wait(hold)?;
        }
        end(stage, p, &self.log);
        Ok(())
    }
}

// -------------------------------------------------------------------- main

fn options(path: PathBuf, res: (u32, u32), preset: &'static str) -> Result<RenderOptions, Box<dyn std::error::Error>> {
    let mut o = encode::delivery_options(
        path,
        RenderFormat::Mp4,
        Quality {
            preset,
            ..Quality::delivery()
        },
    )?;
    o.config.camera.resolution = res;
    o.config.camera.fps = 60;
    o.config.camera.background_color = explainer_kit::BACKGROUND.into();
    Ok(o)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let usage = "usage: spike_s4_s6 OUT_DIR s4|s6 [--res WxH] [--preset P] [--glyph dots|outline (s4)] [--paths DIR (s6)] [--set strands|ribbons (s6)] [--fresh yes|no (s6)]";
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (out, mode) = match args.as_slice() {
        [o, m, ..] if m == "s4" || m == "s6" => (PathBuf::from(o), m.clone()),
        _ => return Err(usage.into()),
    };
    let (mut res, mut preset, mut paths) = ((1920u32, 1080u32), "slow", None::<PathBuf>);
    let (mut kind, mut set, mut fresh) = (GlyphKind::Dots, SetKind::Strands, true);
    let mut rest = args[2..].iter();
    while let Some(flag) = rest.next() {
        let value = rest.next().ok_or(usage)?;
        match (flag.as_str(), value.as_str()) {
            ("--res", _) => {
                let (w, h) = value.split_once('x').ok_or("WxH")?;
                res = (w.parse()?, h.parse()?);
            }
            ("--preset", _) => preset = Box::leak(value.clone().into_boxed_str()),
            ("--paths", _) => paths = Some(PathBuf::from(value)),
            ("--glyph", "dots") => kind = GlyphKind::Dots,
            ("--glyph", "outline") => kind = GlyphKind::Outline,
            ("--fresh", "yes") => fresh = true,
            ("--fresh", "no") => fresh = false,
            ("--set", "strands") => set = SetKind::Strands,
            ("--set", "ribbons") => set = SetKind::Ribbons,
            _ => return Err(usage.into()),
        }
    }
    let sets = match (&*mode, &paths) {
        ("s6", Some(dir)) => {
            let plain = load_paths(&dir.join("paths_noflag.txt"))?;
            let flagged = load_paths(&dir.join("paths_flag.txt"))?;
            check_sets(&plain, &flagged)?;
            Some((plain, flagged))
        }
        ("s6", None) => return Err("s6 needs --paths DIR (paths_noflag.txt, paths_flag.txt)".into()),
        _ => None,
    };
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    std::fs::create_dir_all(&out)?;
    let opts = options(out.join(format!("{mode}.mp4")), res, preset)?;
    let log: Shared = Rc::default();
    // the wall clock of the attempt that succeeded (a transient-EPERM retry
    // starts it again)
    let started = Cell::new(Instant::now());
    let sets = sets.map(Rc::new);
    let (report, attempt) = encode::retrying(&mode, || {
        *log.borrow_mut() = Log::default();
        started.set(Instant::now());
        match &sets {
            Some(s) => {
                let (plain, flagged) = &**s;
                let mut scene = PathMorph {
                    plain: plain.clone(),
                    flagged: flagged.clone(),
                    set,
                    fresh,
                    log: Rc::clone(&log),
                };
                fmn::render(&mut scene, opts.clone())
            }
            None => fmn::render(
                &mut Netfuls {
                    kind,
                    log: Rc::clone(&log),
                },
                opts.clone(),
            ),
        }
    });
    let report = report?;
    let secs = started.get().elapsed().as_secs_f64();
    let frames = report.artifact.frame_count;
    let log = log.borrow();
    let variant = match (&*mode, kind) {
        ("s6", _) if set == SetKind::Ribbons => "\tribbons",
        ("s6", _) if fresh => "\tfresh strands",
        ("s6", _) => "\tgrown strands",
        (_, GlyphKind::Dots) => "\tglyph dots",
        (_, GlyphKind::Outline) => "\tglyph outline",
    };
    println!(
        "{mode}\t{}x{}\t{} records\t{frames} frames\t{secs:.1} s\t{:.2} fps\tpreset {preset}{variant}\tattempt {attempt}",
        res.0,
        res.1,
        log.records,
        frames as f64 / secs
    );
    for (name, frames, secs) in &log.phases {
        let fps = if *frames > 0 { *frames as f64 / secs } else { 0.0 };
        println!("phase\t{name}\t{frames} frames\t{secs:.2} s\t{fps:.2} fps");
    }
    for note in &log.notes {
        println!("note\t{note}");
    }
    Ok(())
}
