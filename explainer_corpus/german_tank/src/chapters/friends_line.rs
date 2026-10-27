//! The line with the friends' five boxes, as chapter 01 builds it, for the
//! chapters after it: one dot per number from 1 to 84, the thin line past
//! them, the five small boxes with their gold numbers, and the ticks
//! joining each box to its dot. Everything is bound through `draw::x_at` to
//! trackers, as in chapter 01 (the line's last number shown, how far the
//! boxes have spread into the fan, how far the ticks have grown), plus
//! chapter 02's bin: how far the dots have poured into it, the shake, and
//! how far one handful has risen out of it. A binding moves or reshapes
//! mobjects that already exist, on the frames where one of its trackers
//! changed; nothing is redrawn.
//!
//! Also here, because chapters 02 and 03 both draw them: a computed
//! handful's rings (gold outlines at half strength) bound to the line's
//! end, spacing arrows bound to how far they have run, gap arcs, the badge
//! "if the handful is fair" (the same every time it appears), and readers
//! for the facts the scenes draw.
//!
//! Chapter 01 keeps its own copy of the line's binding (`five_boxes::Bound`,
//! frozen while root qualifies that chapter); moving it here is a later,
//! separate change.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Facts, Kit, Palette, Timed, v};
use fmn::prelude::*;

use super::draw::{
    BASE_PIECES, BASE_W, DOT_OPACITY, DOT_R, NUMBER_SIZE, OFF, SMALL, STRIP_Y, TICK_W, base_piece, box_spot,
    dot_opacity, flat, game_box, line_style, stroke, tick_ends, x_at,
};
use crate::palette;

// ------------------------------------------------------------ facts readers

/// A whole-number fact.
pub fn whole(f: &Facts, key: &str) -> Result<u64, String> {
    f.get(key)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("no whole-number fact {key}"))
}

/// A list fact, comma separated.
pub fn list(f: &Facts, key: &str) -> Result<Vec<u64>, String> {
    let s = f.get(key).ok_or_else(|| format!("no fact {key}"))?;
    s.split(',')
        .map(|x| x.parse().map_err(|_| format!("{key} holds {x:?}")))
        .collect()
}

/// Rows of comma lists, separated by semicolons (the drawn handfuls).
pub fn handfuls(f: &Facts, key: &str) -> Result<Vec<Vec<u64>>, String> {
    let s = f.get(key).ok_or_else(|| format!("no fact {key}"))?;
    s.split(';')
        .map(|row| {
            row.split(',')
                .map(|x| x.parse().map_err(|_| format!("{key} holds {x:?}")))
                .collect()
        })
        .collect()
}

/// A fact as the facts print it (a decimal, a fraction, a flag).
pub fn printed(f: &Facts, key: &str) -> Result<String, String> {
    f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"))
}

/// A printed number (a whole number or a decimal) as a float, to compare a
/// readout with the fact it must end on.
pub fn value(printed: &str) -> Result<f64, String> {
    printed
        .parse()
        .map_err(|_| format!("{printed:?} is not a number a readout can show"))
}

/// A handful a scene can draw: `k` different numbers in order, from 1 to
/// `n`.
pub fn is_handful(hand: &[u64], k: usize, n: u64) -> bool {
    hand.len() == k
        && hand.first().is_some_and(|&a| a >= 1)
        && hand.last().is_some_and(|&b| b <= n)
        && hand.windows(2).all(|w| w[0] < w[1])
}

// ----------------------------------------------------------------- layout

/// The panel, left-aligned at `PANEL_X`, one line of arithmetic or words at
/// each of `PANEL_Y`, above the rows of computed handfuls; the badge at
/// the top left, where it stays through chapters 02 and 03; the prediction
/// countdown on the right.
pub const PANEL_X: f64 = 0.3;
pub const PANEL_Y: [f64; 4] = [3.5, 3.05, 2.6, 2.15];
pub const PANEL_SIZE: f64 = 26.0;
pub const BADGE_AT: [f64; 2] = [-5.1, 3.5];
pub const THINK_AT: [f64; 2] = [4.6, 1.3];

pub fn at(p: [f64; 2]) -> Vec3 {
    v(p[0], p[1])
}

/// The panel's line `j`, at its left edge.
pub fn panel(j: usize) -> Vec3 {
    v(PANEL_X, PANEL_Y[j])
}

/// Scene time now.
pub fn now(stage: &Stage<'_>) -> f64 {
    stage.scene().time().to_f64()
}

/// Seconds a card needs on screen to be read: fifteen characters a second,
/// a quick reader (the video can be paused), counted from its completed
/// fade-in.
pub fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

// ---------------------------------------------------------------- strokes

/// Somewhere off the frame: a stroke the binding has not placed yet.
pub fn hidden() -> Vec<Vec3> {
    vec![v(OFF[0], OFF[1]), v(OFF[0] + 0.01, OFF[1])]
}

pub fn reshape(s: &mut fmn::mobject::Stage, m: Mob, points: Vec<Vec3>) {
    let _ = s.set_points(m, polyline(points).build().points());
}

/// A stroke a binding places (off the frame until it does). Detached.
pub fn thin(stage: &mut Stage<'_>, color: Srgb, w: f64, o: f64) -> Mob {
    stroke(stage, hidden(), line_style(color, w, o))
}

/// One updater, on a new anchor in the scene, that calls `place` with the
/// trackers' values on the first frame and on every frame where one of
/// them changed: a binding moves what exists and never redraws.
pub fn bind_to(
    stage: &mut Stage<'_>,
    trackers: &[Mob],
    mut place: impl FnMut(&mut fmn::mobject::Stage, &[f64]) + 'static,
) -> fmn::Result<Mob> {
    let trackers = trackers.to_vec();
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last: Option<Vec<f64>> = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let now: Vec<f64> = trackers.iter().map(|&t| s.tracker_value(t).unwrap_or(0.0)).collect();
            if last.as_ref() == Some(&now) {
                return;
            }
            place(s, &now);
            last = Some(now);
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// Bring detached mobjects in together, each dropping `DROP` into place,
/// over `rt` seconds (a row of a computed handful landing).
pub fn drop_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, v(0.0, -DROP), 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// How far a row falls as it lands.
pub const DROP: f64 = 0.3;

// ------------------------------------------------------------------ the bin

/// The bin (02_1): two walls and a floor, centred at `BIN_AT`. The heap on
/// its floor: rows of `HEAP_COLS` dots `HEAP_STEP` apart, the first row
/// `HEAP_FLOOR` above the floor, each dot nudged at most `HEAP_JITTER` each
/// way so it reads as a heap, not a grid.
pub const BIN_AT: [f64; 2] = [-2.2, 2.0];
pub const BIN_W: f64 = 3.4;
pub const BIN_H: f64 = 1.7;
pub const BIN_STROKE: f64 = 3.0;
pub const HEAP_COLS: usize = 14;
pub const HEAP_STEP: f64 = 0.2;
pub const HEAP_FLOOR: f64 = 0.25;
pub const HEAP_JITTER: f64 = 0.05;
/// The dots go into the heap shuffled: dot `i` (from 0) lands in slot
/// `i × HEAP_MIX` mod the dot count, one slot each while the two numbers
/// share no factor.
pub const HEAP_MIX: usize = 37;
/// The pour: the last dot sets off when the first is this share of the way
/// through its own pour, and each dot arcs up `POUR_LIFT` on its way.
pub const POUR_STAGGER: f64 = 0.5;
pub const POUR_LIFT: f64 = 1.2;
/// The shake: how far a dot in the heap swings, and how many half swings
/// one shake makes (a whole number, so the heap comes to rest where it
/// started); each slot swings its own way, a golden share of a turn on
/// from the slot before.
pub const SHAKE_AMP: f64 = 0.06;
pub const SHAKE_HALF_SWINGS: f64 = 6.0;
pub const SWING_TURN: f64 = 0.618_034;
/// The handful that rises out of the bin (02_2) lines up here, a box
/// `RISEN_STEP` apart.
pub const RISEN_AT: [f64; 2] = [0.4, 2.2];
pub const RISEN_STEP: f64 = 0.45;

/// The heap slot dot `i` of `count` lands in.
pub fn heap_slot(i: usize, count: usize) -> usize {
    (i * HEAP_MIX) % count
}

/// A small fixed nudge for heap slot `slot`, each way at most
/// `HEAP_JITTER`.
fn jitter(slot: usize) -> [f64; 2] {
    let a = ((slot * 7 + 3) % 11) as f64 / 10.0 - 0.5;
    let b = ((slot * 5 + 1) % 13) as f64 / 12.0 - 0.5;
    [2.0 * HEAP_JITTER * a, 2.0 * HEAP_JITTER * b]
}

/// Where heap slot `slot` sits on the bin's floor.
pub fn heap_spot(slot: usize) -> Vec3 {
    let (col, row) = (slot % HEAP_COLS, slot / HEAP_COLS);
    let x0 = BIN_AT[0] - HEAP_STEP * (HEAP_COLS - 1) as f64 / 2.0;
    let y0 = BIN_AT[1] - BIN_H / 2.0 + HEAP_FLOOR;
    let j = jitter(slot);
    v(x0 + HEAP_STEP * col as f64 + j[0], y0 + HEAP_STEP * row as f64 + j[1])
}

/// The bin's walls and floor, an open box. Detached.
pub fn bin(stage: &mut Stage<'_>) -> Mob {
    let (x0, x1) = (BIN_AT[0] - BIN_W / 2.0, BIN_AT[0] + BIN_W / 2.0);
    let (y0, y1) = (BIN_AT[1] - BIN_H / 2.0, BIN_AT[1] + BIN_H / 2.0);
    stroke(
        stage,
        vec![v(x0, y1), v(x0, y0), v(x1, y0), v(x1, y1)],
        line_style(palette::outline(), BIN_STROKE, 1.0),
    )
}

/// How far through its own pour dot `i` of `count` is when the whole pour
/// is at `pour` (0: every dot on the line; 1: every dot in the heap).
pub fn pour_share(i: usize, count: usize, pour: f64) -> f64 {
    let start = if count > 1 {
        POUR_STAGGER * i as f64 / (count - 1) as f64
    } else {
        0.0
    };
    ((pour - start) / (1.0 - POUR_STAGGER)).clamp(0.0, 1.0)
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    v(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
}

/// Where a dot is: from `on`, its place on the line, to `heap`, its slot,
/// as its own pour share `p` goes from 0 to 1, arcing up on the way; once
/// in, it swings with the shake (0 to 1, still at both ends).
pub fn poured(on: Vec3, heap: Vec3, p: f64, shake: f64, slot: usize) -> Vec3 {
    let e = smooth(p);
    let lift = POUR_LIFT * (PI * p).sin();
    let swing = SHAKE_AMP * (PI * SHAKE_HALF_SWINGS * shake).sin() * p;
    let way = TAU * (slot as f64 * SWING_TURN).fract();
    v(
        on[0] + (heap[0] - on[0]) * e + swing * way.cos(),
        on[1] + (heap[1] - on[1]) * e + lift + swing * way.sin(),
    )
}

/// The `k`th risen box's place above the bin.
pub fn risen_spot(k: usize) -> Vec3 {
    v(RISEN_AT[0] + RISEN_STEP * k as f64, RISEN_AT[1])
}

// ----------------------------------------------------------------- the line

/// A computed handful's number: a gold outline at half strength, never
/// filled, around where its dot would be. Small enough that the closest
/// numbers in a drawn row (12 apart on a line to a thousand, 1 apart on the
/// opening line) keep their rings apart.
pub const RING_R: f64 = 0.06;
pub const RING_W: f64 = 2.5;
pub const COMPUTED_O: f64 = 0.5;

/// A computed handful's ring centred at `at`. Detached.
pub fn ring(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    let r = Dot::new()
        .point(at)
        .radius(RING_R)
        .style(line_style(palette::seen(), RING_W, COMPUTED_O))
        .build();
    stage.arena_mut().add(r)
}

/// The trackers the line is bound to.
#[derive(Clone, Copy)]
pub struct Trackers {
    /// The last number the line shows.
    pub end: Mob,
    /// How far the boxes have spread into the fan, 0 to 1.
    pub fan: Mob,
    /// How far the ticks have grown, 0 to 1.
    pub rise: Mob,
    /// How far the dots have poured into the bin, 0 to 1.
    pub pour: Mob,
    /// The bin's shake, 0 to 1.
    pub shake: Mob,
    /// How far the handful drawn from the bin has risen out of it, 0 to 1.
    pub lift: Mob,
}

/// The trackers' values at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Now {
    pub end: f64,
    pub fan: f64,
    pub rise: f64,
    pub pour: f64,
    pub shake: f64,
    pub lift: f64,
}

impl Now {
    /// The line shown to `end`, the boxes spread by `fan`, the ticks grown
    /// to `rise`, every dot on the line.
    #[cfg(test)]
    pub fn on_line(end: f64, fan: f64, rise: f64) -> Now {
        Now {
            end,
            fan,
            rise,
            pour: 0.0,
            shake: 0.0,
            lift: 0.0,
        }
    }
}

/// Where the line's pieces go for given tracker values: the pure part of
/// the binding, which the scenes' updater applies and the tests check.
#[derive(Clone, Debug)]
pub struct Layout {
    /// The first number; dot `i` is number `first + i`.
    pub first: u64,
    pub count: usize,
    /// The friends' numbers, one box each.
    pub numbers: Vec<u64>,
    /// The dots (by index) of the handful drawn from the bin.
    pub drawn: Vec<usize>,
}

impl Layout {
    /// Where dot `i` is.
    pub fn dot(&self, i: usize, now: Now) -> Vec3 {
        let n = self.first + i as u64;
        let on = v(x_at(n as f64, now.end), STRIP_Y);
        let slot = heap_slot(i, self.count);
        let at = poured(
            on,
            heap_spot(slot),
            pour_share(i, self.count, now.pour),
            now.shake,
            slot,
        );
        match self.drawn.iter().position(|&j| j == i) {
            Some(k) => lerp(at, risen_spot(k), smooth(now.lift)),
            None => at,
        }
    }
}

/// The line's mobjects, in the binding.
#[derive(Clone)]
struct Pieces {
    dots: Vec<Mob>,
    base: Vec<Mob>,
    bodies: Vec<Mob>,
    labels: Vec<Mob>,
    ticks: Vec<Mob>,
    rings: Vec<Mob>,
}

impl Pieces {
    fn place(&self, s: &mut fmn::mobject::Stage, at: &Layout, now: Now) {
        for (i, &d) in self.dots.iter().enumerate() {
            s.move_to(d, at.dot(i, now), ORIGIN);
        }
        for (k, &i) in at.drawn.iter().enumerate() {
            s.move_to(self.rings[k], at.dot(i, now), ORIGIN);
        }
        let last = at.first + at.count as u64 - 1;
        for (k, &m) in self.base.iter().enumerate() {
            let points = match base_piece(k, last, now.end) {
                Some((a, b)) => vec![v(a, STRIP_Y), v(b, STRIP_Y)],
                None => hidden(),
            };
            reshape(s, m, points);
        }
        for (i, &n) in at.numbers.iter().enumerate() {
            let spot = box_spot(i, n, now.end, now.fan);
            s.move_to(self.bodies[i], spot, ORIGIN);
            s.move_to(self.labels[i], spot, ORIGIN);
            let [a, b] = tick_ends(n, now.end, spot, now.rise);
            reshape(s, self.ticks[i], vec![a, b]);
        }
    }
}

/// What the line shows when it is built, and the furthest it will zoom out
/// to (the line past the dots fades as it would at that end).
#[derive(Clone, Copy)]
pub struct Start {
    pub end: f64,
    pub fan: f64,
    pub rise: f64,
    pub widest: f64,
}

/// The line, the friends' boxes and the bin handful's rings, bound to
/// [`Trackers`].
pub struct FriendsLine {
    /// dots[i] is number `first + i`.
    pub dots: Vec<Mob>,
    pub base: Vec<Mob>,
    pub bodies: Vec<Mob>,
    pub labels: Vec<Mob>,
    pub ticks: Vec<Mob>,
    /// One ring per number of the handful drawn from the bin.
    pub rings: Vec<Mob>,
    pub t: Trackers,
}

impl FriendsLine {
    /// The line from `first` to `last`, the friends' boxes on `numbers`
    /// showing their numbers (their lids went when they turned over in
    /// 01_4), and rings for the handful `drawn` (numbers on the line) that
    /// chapter 02 draws from the bin; bound to new trackers that start at
    /// `start`, every dot on the line. The pieces are detached; the
    /// binding's anchor is in the scene.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        first: u64,
        last: u64,
        numbers: &[u64],
        drawn: &[u64],
        start: Start,
    ) -> fmn::Result<FriendsLine> {
        let count = (last - first + 1) as usize;
        let dots: Vec<Mob> = (first..=last)
            .map(|n| {
                let x = x_at(n as f64, start.end);
                let (color, o) = if numbers.contains(&n) {
                    (palette::seen(), 1.0)
                } else {
                    (palette::line(), dot_opacity(x))
                };
                let dot = Dot::new()
                    .point(v(x, STRIP_Y))
                    .radius(DOT_R)
                    .style(flat(color, o))
                    .build();
                stage.arena_mut().add(dot)
            })
            .collect();
        let base: Vec<Mob> = (0..BASE_PIECES)
            .map(|k| {
                let o = base_piece(k, last, start.widest).map_or(DOT_OPACITY, |(a, b)| dot_opacity((a + b) / 2.0));
                thin(stage, palette::line(), BASE_W, o)
            })
            .collect();
        let (mut bodies, mut labels, mut ticks) = (Vec::new(), Vec::new(), Vec::new());
        for (i, &n) in numbers.iter().enumerate() {
            let spot = box_spot(i, n, start.end, start.fan);
            // a box that has turned over: its body, no lid
            let [body, _lid] = game_box(stage, spot)?;
            stage.scale(body, SMALL);
            let label = kit.text(stage, &n.to_string(), NUMBER_SIZE, palette::seen(), spot)?;
            stage.scale(label, SMALL);
            bodies.push(body);
            labels.push(label);
            ticks.push(thin(stage, palette::seen(), TICK_W, 1.0));
        }
        let rings: Vec<Mob> = drawn
            .iter()
            .map(|&n| ring(stage, v(x_at(n as f64, start.end), STRIP_Y)))
            .collect();
        let t = Trackers {
            end: stage.add_value_tracker(start.end),
            fan: stage.add_value_tracker(start.fan),
            rise: stage.add_value_tracker(start.rise),
            pour: stage.add_value_tracker(0.0),
            shake: stage.add_value_tracker(0.0),
            lift: stage.add_value_tracker(0.0),
        };
        let at = Layout {
            first,
            count,
            numbers: numbers.to_vec(),
            drawn: drawn.iter().map(|&n| (n - first) as usize).collect(),
        };
        let pieces = Pieces {
            dots: dots.clone(),
            base: base.clone(),
            bodies: bodies.clone(),
            labels: labels.clone(),
            ticks: ticks.clone(),
            rings: rings.clone(),
        };
        bind_to(
            stage,
            &[t.end, t.fan, t.rise, t.pour, t.shake, t.lift],
            move |s, now| {
                let now = Now {
                    end: now[0],
                    fan: now[1],
                    rise: now[2],
                    pour: now[3],
                    shake: now[4],
                    lift: now[5],
                };
                pieces.place(s, &at, now);
            },
        )?;
        Ok(FriendsLine {
            dots,
            base,
            bodies,
            labels,
            ticks,
            rings,
            t,
        })
    }

    /// What a chapter opens on: the dots, the boxes and their ticks.
    pub fn opening(&self) -> Vec<Mob> {
        let mut all = self.dots.clone();
        all.extend(&self.bodies);
        all.extend(&self.labels);
        all.extend(&self.ticks);
        all
    }
}

// ------------------------------------------------------- rows over the line

/// Rows of computed handfuls over the line (02_5, 02_7): row `j` at
/// `ROW_Y0 + j × ROW_DY`, clear of the boxes' tops below them and the
/// panel above.
pub const ROW_Y0: f64 = 0.55;
pub const ROW_DY: f64 = 0.18;

pub fn row_y(j: usize) -> f64 {
    ROW_Y0 + ROW_DY * j as f64
}

/// Computed handfuls as rows of rings, each ring at its number's place on
/// the line shown to the `end` tracker.
pub struct Rows {
    pub rings: Vec<Vec<Mob>>,
}

impl Rows {
    /// Rings for `hands`, row `j` at height `ys[j]`, placed for the line
    /// shown to `end_now` and bound to the `end` tracker from then on.
    /// Detached rings; the binding's anchor is in the scene.
    pub fn bind(stage: &mut Stage<'_>, hands: &[Vec<u64>], ys: &[f64], end: Mob, end_now: f64) -> fmn::Result<Rows> {
        let rings: Vec<Vec<Mob>> = hands
            .iter()
            .zip(ys)
            .map(|(h, &y)| h.iter().map(|&n| ring(stage, v(x_at(n as f64, end_now), y))).collect())
            .collect();
        let placed: Vec<(Vec<Mob>, Vec<u64>, f64)> = rings
            .iter()
            .zip(hands)
            .zip(ys)
            .map(|((r, h), &y)| (r.clone(), h.clone(), y))
            .collect();
        bind_to(stage, &[end], move |s, now| {
            for (row, hand, y) in &placed {
                for (&r, &n) in row.iter().zip(hand) {
                    s.move_to(r, v(x_at(n as f64, now[0]), *y), ORIGIN);
                }
            }
        })?;
        Ok(Rows { rings })
    }

    /// Every ring, row by row.
    pub fn all(&self) -> Vec<Mob> {
        self.rings.iter().flatten().copied().collect()
    }
}

// --------------------------------------------------------- spacing arrows

/// Spacing arrows (02_8): hop `j` runs from the handful's number before
/// (zero, for the first) to its `j`th number, at a fixed height; a shaft
/// and a head at its running end, `ARROW_GAP` short of each number so
/// neighbouring hops stay apart; a head's barbs reach `ARROW_SPREAD` of
/// its length up and down.
pub const ARROW_W: f64 = 3.0;
pub const ARROW_HEAD: f64 = 0.09;
pub const ARROW_SPREAD: f64 = 0.7;
pub const ARROW_GAP: f64 = 0.025;
/// Below a row's rings, or below the line for the friends' handful.
pub const ARROW_UNDER_ROW: f64 = 0.15;
pub const ARROW_UNDER_LINE: f64 = 0.42;

/// Hop `j` of `hand`'s spacing arrows at height `y`, run up to number
/// `tip` on a line shown to `end`: its points, or `None` before it starts.
pub fn hop(hand: &[u64], j: usize, tip: f64, end: f64, y: f64) -> Option<Vec<Vec3>> {
    let from = if j == 0 { 0.0 } else { hand[j - 1] as f64 };
    if tip <= from {
        return None;
    }
    let to = (hand[j] as f64).min(tip);
    let x0 = x_at(from, end) + ARROW_GAP;
    let x1 = (x_at(to, end) - ARROW_GAP).max(x0);
    let h = ARROW_HEAD.min((x1 - x0) / 2.0);
    Some(vec![
        v(x0, y),
        v(x1, y),
        v(x1 - h, y + ARROW_SPREAD * h),
        v(x1, y),
        v(x1 - h, y - ARROW_SPREAD * h),
    ])
}

/// `hand`'s spacing arrows at height `y`, bound to how far they have run
/// (`tip`, a number on the line) and to the line's `end`. Detached arrows,
/// off the frame until the tip reaches them; the binding's anchor is in
/// the scene.
pub fn spacing_arrows(stage: &mut Stage<'_>, hand: &[u64], y: f64, tip: Mob, end: Mob) -> fmn::Result<Vec<Mob>> {
    let mobs: Vec<Mob> = hand
        .iter()
        .map(|_| thin(stage, palette::unseen(), ARROW_W, 1.0))
        .collect();
    let (placed, hand) = (mobs.clone(), hand.to_vec());
    bind_to(stage, &[tip, end], move |s, now| {
        for (j, &m) in placed.iter().enumerate() {
            reshape(s, m, hop(&hand, j, now[0], now[1], y).unwrap_or_else(hidden));
        }
    })?;
    Ok(mobs)
}

// -------------------------------------------------------------------- gaps

/// A gap's arc: over its unseen numbers, from half a step before the first
/// to half a step after the last (screen `x0` to `x1`), from `ARC_LIFT`
/// above the dots at `y` up to `h` more.
pub const ARC_LIFT: f64 = 0.06;
pub const ARC_H: f64 = 0.26;
pub const ARC_POINTS: usize = 24;
pub const ARC_W: f64 = 3.0;

pub fn arc_points(x0: f64, x1: f64, y: f64, h: f64) -> Vec<Vec3> {
    (0..=ARC_POINTS)
        .map(|i| {
            let t = i as f64 / ARC_POINTS as f64;
            v(x0 + (x1 - x0) * t, y + ARC_LIFT + h * (PI * t).sin())
        })
        .collect()
}

/// The gaps of a sorted handful as (first unseen, last unseen) number
/// pairs, below the smallest and between neighbours; an empty gap has no
/// pair.
pub fn gap_spans(hand: &[u64]) -> Vec<(u64, u64)> {
    let mut prev = 0;
    let mut out = Vec::new();
    for &x in hand {
        if x > prev + 1 {
            out.push((prev + 1, x - 1));
        }
        prev = x;
    }
    out
}

// ------------------------------------------------------------------- badge

/// The condition, the same badge every time it appears: ink words in a dim
/// outline.
pub const BADGE: &str = "if the handful is fair";
pub const BADGE_SIZE: f64 = 22.0;
pub const BADGE_PAD: [f64; 2] = [0.18, 0.12];

/// The badge centred at `at`, one group. Detached.
pub fn badge(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let words = kit.text(stage, BADGE, BADGE_SIZE, Palette::ink(), at)?;
    let (w, h) = (stage.get_width(words), stage.get_height(words));
    let frame = Rectangle::new()
        .width(w + 2.0 * BADGE_PAD[0])
        .height(h + 2.0 * BADGE_PAD[1])
        .corner_radius(0.08)
        .style(line_style(palette::outline(), 2.0, 1.0))
        .build()?
        .moved_to(at);
    let frame = stage.arena_mut().add(frame);
    let group = stage.arena_mut().add(VMobject::new());
    stage.attach(group, frame)?;
    stage.attach(group, words)?;
    Ok(group)
}

/// A text's width at `size`, estimated as chapter 01's layout test does
/// (0.13 per character at size 24); the layout lint measures real widths.
#[cfg(test)]
pub fn est_width(s: &str, size: f64) -> f64 {
    s.chars().count() as f64 * 0.13 * size / 24.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::STRIP_X1;
    use crate::model::{BIG, FRIENDS, LINE_DOTS, gaps};

    const NEAR: f64 = 1e-9;

    fn layout(drawn: &[usize]) -> Layout {
        Layout {
            first: 1,
            count: LINE_DOTS as usize,
            numbers: FRIENDS.to_vec(),
            drawn: drawn.to_vec(),
        }
    }

    fn close(a: Vec3, b: Vec3) -> bool {
        (a[0] - b[0]).abs() < NEAR && (a[1] - b[1]).abs() < NEAR
    }

    /// Every dot gets its own heap slot; with a mix sharing a factor with
    /// the count, two dots would land in one slot (the negative control).
    #[test]
    fn every_dot_has_its_own_heap_slot() {
        let count = LINE_DOTS as usize;
        let mut slots: Vec<usize> = (0..count).map(|i| heap_slot(i, count)).collect();
        slots.sort_unstable();
        assert_eq!(slots, (0..count).collect::<Vec<_>>());
        // 37 divides 74: dots 0 and 2 share slot 0
        assert_eq!(heap_slot(0, 74), heap_slot(2, 74));
        // the order is shuffled: neighbours on the line are not neighbours
        // in the heap
        assert!((0..count - 1).all(|i| heap_slot(i + 1, count).abs_diff(heap_slot(i, count)) > 1));
    }

    /// The heap lies on the bin's floor, inside its walls, below its top,
    /// and its dots never touch.
    #[test]
    fn the_heap_fits_in_the_bin() {
        let count = LINE_DOTS as usize;
        let (left, right) = (BIN_AT[0] - BIN_W / 2.0, BIN_AT[0] + BIN_W / 2.0);
        let (floor, top) = (BIN_AT[1] - BIN_H / 2.0, BIN_AT[1] + BIN_H / 2.0);
        let spots: Vec<Vec3> = (0..count).map(heap_spot).collect();
        for p in &spots {
            assert!(p[0] - DOT_R > left + 0.1 && p[0] + DOT_R < right - 0.1, "{p:?}");
            assert!(p[1] - DOT_R > floor + 0.1 && p[1] + DOT_R < top - 0.3, "{p:?}");
        }
        for (i, a) in spots.iter().enumerate() {
            for b in &spots[i + 1..] {
                assert!(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() > 2.0 * DOT_R);
            }
        }
        for slot in 0..count {
            let j = jitter(slot);
            assert!(j[0].abs() <= HEAP_JITTER + NEAR && j[1].abs() <= HEAP_JITTER + NEAR);
        }
    }

    /// The pour starts with every dot on the line and ends with every dot
    /// in the heap; each dot only moves forward, and a later dot is never
    /// further along than an earlier one.
    #[test]
    fn the_pour_runs_in_order_and_ends_in_the_heap() {
        let count = LINE_DOTS as usize;
        for i in 0..count {
            assert_eq!(pour_share(i, count, 0.0), 0.0);
            assert_eq!(pour_share(i, count, 1.0), 1.0);
            let mut prev = 0.0;
            for step in 0..=100 {
                let p = pour_share(i, count, step as f64 / 100.0);
                assert!(p >= prev);
                prev = p;
            }
        }
        for step in 0..=100 {
            let pour = step as f64 / 100.0;
            for i in 0..count - 1 {
                assert!(pour_share(i + 1, count, pour) <= pour_share(i, count, pour));
            }
        }
        assert_eq!(pour_share(0, 1, 0.3), 0.6, "one dot pours alone");
    }

    /// Off the line and back: at no pour a dot is on its number, shaken or
    /// not; fully poured it is in its slot at rest, before and after the
    /// shake, and it swings only in between.
    #[test]
    fn a_dot_is_on_its_number_or_in_its_slot() {
        let on = v(1.2, STRIP_Y);
        for slot in [0, 13, 83] {
            let heap = heap_spot(slot);
            for shake in [0.0, 0.3, 1.0] {
                assert!(close(poured(on, heap, 0.0, shake, slot), on));
            }
            assert!(close(poured(on, heap, 1.0, 0.0, slot), heap));
            let rest = poured(on, heap, 1.0, 1.0, slot);
            assert!((rest[0] - heap[0]).abs() < 1e-12 && (rest[1] - heap[1]).abs() < 1e-12);
            let mid = poured(on, heap, 1.0, 1.0 / (2.0 * SHAKE_HALF_SWINGS), slot);
            let d = ((mid[0] - heap[0]).powi(2) + (mid[1] - heap[1]).powi(2)).sqrt();
            assert!((d - SHAKE_AMP).abs() < 1e-9, "a full swing mid-shake");
            // half way, the dot is above the straight path
            let half = poured(on, heap, 0.5, 0.0, slot);
            assert!(half[1] > (on[1] + heap[1]) / 2.0 + POUR_LIFT - NEAR);
        }
    }

    /// The binding's dots: on their numbers with nothing poured, at either
    /// end of the zoom; all in the heap when poured; the drawn handful in
    /// its risen row when lifted, and back on its numbers when the pour and
    /// the lift are both undone.
    #[test]
    fn the_line_pours_and_comes_back() {
        let bin_handful = [10u64, 13, 16, 61, 83];
        let drawn: Vec<usize> = bin_handful.iter().map(|&n| (n - 1) as usize).collect();
        let at = layout(&drawn);
        for end in [LINE_DOTS as f64, BIG as f64] {
            let now = Now::on_line(end, 0.0, 1.0);
            for i in 0..at.count {
                assert!(close(at.dot(i, now), v(x_at((i + 1) as f64, end), STRIP_Y)));
            }
        }
        let poured_in = Now {
            pour: 1.0,
            ..Now::on_line(LINE_DOTS as f64, 0.0, 0.0)
        };
        for i in 0..at.count {
            assert!(close(at.dot(i, poured_in), heap_spot(heap_slot(i, at.count))));
        }
        let risen = Now { lift: 1.0, ..poured_in };
        for (k, &i) in drawn.iter().enumerate() {
            assert!(close(at.dot(i, risen), risen_spot(k)));
        }
        // a dot not drawn stays in the heap while the five rise
        assert!(close(at.dot(0, risen), heap_spot(heap_slot(0, at.count))));
        let back = Now::on_line(LINE_DOTS as f64, 0.0, 0.0);
        for &i in &drawn {
            assert!(close(
                at.dot(i, back),
                v(x_at((i + 1) as f64, LINE_DOTS as f64), STRIP_Y)
            ));
        }
    }

    /// The risen row sits right of the bin, apart, inside the frame.
    #[test]
    fn the_risen_handful_lines_up_beside_the_bin() {
        assert!(risen_spot(0)[0] - RING_R > BIN_AT[0] + BIN_W / 2.0 + 0.3);
        for k in 0..4 {
            assert!(risen_spot(k + 1)[0] - risen_spot(k)[0] > 2.0 * RING_R + 0.2);
        }
        assert!(risen_spot(4)[0] + RING_R < STRIP_X1);
    }

    /// Spacing arrows: none before the tip passes a hop's start; run to the
    /// last number, every hop is whole, in order and apart, its head no
    /// longer than half its shaft; part way, the running hop ends at the
    /// tip and the later ones are not there.
    #[test]
    fn spacing_arrows_run_from_zero_to_each_number() {
        let end = LINE_DOTS as f64;
        let y = STRIP_Y - ARROW_UNDER_LINE;
        let hand = FRIENDS;
        assert!(hop(&hand, 0, 0.0, end, y).is_none());
        let last = *hand.last().unwrap() as f64;
        let whole: Vec<Vec<Vec3>> = (0..hand.len()).map(|j| hop(&hand, j, last, end, y).unwrap()).collect();
        assert!(
            (whole[0][0][0] - (x_at(0.0, end) + ARROW_GAP)).abs() < NEAR,
            "from zero"
        );
        for (j, h) in whole.iter().enumerate() {
            assert!((h[1][0] - (x_at(hand[j] as f64, end) - ARROW_GAP)).abs() < NEAR);
            assert!(h.iter().all(|p| (p[1] - y).abs() <= ARROW_SPREAD * ARROW_HEAD + NEAR));
            assert!(h[0][0] - h[2][0] <= 0.0 && h[1][0] - h[2][0] <= (h[1][0] - h[0][0]) / 2.0 + NEAR);
        }
        for w in whole.windows(2) {
            assert!(w[0][1][0] < w[1][0][0], "apart");
        }
        // run to 30: hops to 14 and 27 whole, the third running, no fourth
        let part: Vec<Option<Vec<Vec3>>> = (0..hand.len()).map(|j| hop(&hand, j, 30.0, end, y)).collect();
        assert!((part[2].as_ref().unwrap()[1][0] - (x_at(30.0, end) - ARROW_GAP)).abs() < NEAR);
        assert!(part[3].is_none() && part[4].is_none());
        // the first hop of a thousand's row is short, and its head shrinks
        let tiny = hop(&[7, 158, 824, 866, 953], 0, 953.0, BIG as f64, y).unwrap();
        assert!(tiny[1][0] - tiny[2][0] <= (tiny[1][0] - tiny[0][0]) / 2.0 + NEAR);
    }

    /// The friends' gaps as spans: below 14, then between neighbours; their
    /// sizes are the model's gaps; an empty gap (neighbouring numbers)
    /// makes no span.
    #[test]
    fn gap_spans_are_the_gaps() {
        let spans = gap_spans(&FRIENDS);
        assert_eq!(spans, [(1, 13), (15, 26), (28, 32), (34, 50), (52, 59)]);
        let sizes: Vec<u64> = spans.iter().map(|&(a, b)| b - a + 1).collect();
        assert_eq!(sizes, gaps(&FRIENDS));
        assert_eq!(gap_spans(&[1, 2, 5]), [(3, 4)]);
        assert!(gap_spans(&[1, 2, 3]).is_empty());
    }

    /// An arc starts and ends just above the dots, peaks in its middle, and
    /// neighbouring gaps' arcs end before the friend's tick between them.
    #[test]
    fn arcs_sit_over_their_gaps() {
        let end = LINE_DOTS as f64;
        for &(a, b) in &gap_spans(&FRIENDS) {
            let p = arc_points(x_at(a as f64 - 0.5, end), x_at(b as f64 + 0.5, end), STRIP_Y, ARC_H);
            assert!((p[0][1] - (STRIP_Y + ARC_LIFT)).abs() < NEAR);
            assert!((p[ARC_POINTS][1] - (STRIP_Y + ARC_LIFT)).abs() < NEAR);
            assert!((p[ARC_POINTS / 2][1] - (STRIP_Y + ARC_LIFT + ARC_H)).abs() < NEAR);
            // the friend's number just past the gap has its tick there
            assert!(p[ARC_POINTS][0] < x_at(b as f64 + 1.0, end));
            if a > 1 {
                assert!(p[0][0] > x_at(a as f64 - 1.0, end));
            }
        }
    }

    /// The handful check, at its edges.
    #[test]
    fn a_drawable_handful_is_k_different_numbers_in_order() {
        assert!(is_handful(&FRIENDS, 5, 84));
        assert!(is_handful(&[1, 2, 3, 4, 5], 5, 5));
        assert!(!is_handful(&[0, 2, 3, 4, 5], 5, 5), "no box 0");
        assert!(!is_handful(&[1, 2, 3, 4, 6], 5, 5), "past the count");
        assert!(!is_handful(&[1, 2, 2, 4, 5], 5, 5), "a box twice");
        assert!(!is_handful(&[2, 1, 3, 4, 5], 5, 5), "out of order");
        assert!(!is_handful(&[1, 2, 3, 4], 5, 5), "four");
    }

    #[test]
    fn printed_numbers_read_back() {
        assert_eq!(value("190.6").unwrap(), 190.6);
        assert_eq!(value("12").unwrap(), 12.0);
        assert!(value("61/5").is_err(), "a fraction is not a readout");
    }
}
