//! Chapter 12, heat and chance: now let something flow. Put heat on node
//! four; each link carries heat from its hotter end to its cooler end, as
//! fast as the difference between them, so the rule is minus the matrix L,
//! whose diagonal is the number of links at each node (12_1). The answer is
//! its exponential: the heat spreads and levels out, a quarter on each node
//! (12_2). Read the amber as heat or as the chance of finding a random
//! hopper; the matrix is the same (12_3). On four hundred nodes the
//! exponential still answers in one stroke (12_4). The two tanks were this
//! all along: two nodes, one link, settling at half and half; now put the
//! letter i into the exponent, and does it still settle (12_5)? Never: the
//! chance sloshes across and back, for ever, which is how a quantum state
//! moves in time (12_6).
//!
//! Every number is a fact: the rule is `F-D.L`, the heat `F-D.heat.*`, the
//! level `F-V.12_2.level`, the hopper's long-run chances `F-D.chance.*`, the
//! four hundred nodes `F-D.big.*` and `F-V.12_4.nodes`, the pair
//! `F-D.pair.*`.

use super::draw::lower_third;
use explainer_kit::stats::Data;
use explainer_kit::{Kit, Palette, fade_all, play, v};
use fmn::prelude::*;

use super::draw::{
    MatrixGrid, arrow_points, corners, fact, fade_in_all, fade_out_all, faded_line, fail, flat, music_hold, panel,
    predict_hold, result, rule, say, substance, tween_linear, wait_until, whole,
};
use super::flow::{Levels, fade_in_anim, fade_out_anim};
use super::sound;
use super::walks::{NODE_R, TOKEN_R, node, token_at};
use crate::facts::{quantum_pair, shown};
use crate::model::{KITE, Mat, Network, PAIR, expm_t, grid_heat};

// ------------------------------------------------------------------ choices

/// The bars of 12_2: x of each, the baseline, the width and the height of a
/// bar holding all of the heat.
/// The four bars of the heat: with the last one's right edge 0.4 left of the
/// matrix's left bracket (at 0.9 .. 3.15 the fourth bar stood against the
/// bracket and the last row's numbers: the dense read of the scored draft).
const BAR_X: [f64; 4] = [0.5, 1.25, 2.0, 2.75];
/// The middle of the bars, where "heat", "chance" and "total" are centred.
const BARS_MID: f64 = 1.625;
const BAR_BASE: f64 = -1.55;
const BAR_W: f64 = 0.46;
const BAR_H: f64 = 2.6;
/// Where the rule's grid sits (12_1), its type size, and how far over its
/// middle its name "-L" stands. The grid sits 0.1 lower than it did (2.05):
/// the name stood 0.23 from the top edge, under the 0.25 a text keeps.
const GRID_AT: Vec3 = [4.95, 1.95, 0.0];
const GRID_PT: f64 = 36.0;
const GRID_NAME_DY: f64 = 1.55;
/// How far the heat time runs at 12_2.
const HEAT_T: f64 = 5.0;
/// The four hundred nodes (12_4): where their grid sits and how far apart.
const BIG_AT: Vec3 = [-2.6, 0.1, 0.0];
const BIG_STEP: f64 = 0.3;
/// How long a crossing of a link takes on screen (seconds).
const HOP_S: f64 = 0.3;
/// The hopper's seed, and how long its schedule is drawn for (seconds).
const HOPPER_SEED: u64 = 20_261_013;
const HOPPER_UNTIL: f64 = 14.0;
/// The panels of 12_5 and 12_6: scene units to one unit of the complex
/// plane, and a bar's height for one whole unit.
const PANEL_UNIT: f64 = 2.0;
/// How long a panel takes to wind back to its start: the pair's of 12_5 as
/// the i goes in, and the panels of 12_6 before the sweep is played again
/// with its sound (seconds).
const REWIND_S: f64 = 0.8;
const PANEL_BAR_H: f64 = 2.0;

// ------------------------------------------------------------- the geometry

/// The rule: minus the Laplacian of the kite.
pub fn minus_l() -> Mat {
    KITE.laplacian().scale(-1.0)
}

/// The heat on each node of the kite `t` after one unit was put on node four
/// (`F-D.heat.*`).
pub fn heat_at(t: f64) -> [f64; 4] {
    let col = expm_t(&minus_l(), t).column(shown::HEAT_START);
    [col[0], col[1], col[2], col[3]]
}

/// How strongly a node glows with `heat` on it.
pub fn glow(heat: f64) -> f64 {
    (0.06 + 0.9 * heat).clamp(0.0, 0.95)
}

/// The corners of a bar `width` wide, `full` tall for a whole unit, standing
/// on `base` at `x` with `value` in it (never quite flat: a shape with no
/// height is not a shape).
pub fn bar_corners(x: f64, base: f64, width: f64, full: f64, value: f64) -> Vec<Vec3> {
    let top = base + full * value.max(0.004);
    // a closed run of vertices, to be set through `corners`
    vec![
        v(x - width / 2.0, base),
        v(x + width / 2.0, base),
        v(x + width / 2.0, top),
        v(x - width / 2.0, top),
        v(x - width / 2.0, base),
    ]
}

/// One crossing of a link by the hopper, and when it sets out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hop {
    pub from: usize,
    pub to: usize,
    pub depart: f64,
}

/// A hopper that leaves a node along each of its links at rate one: it waits
/// an exponential time (mean one over the number of links) and then crosses
/// a link chosen at random. Every crossing up to `until`, from `start`.
pub fn hopper_schedule(net: &Network, start: usize, seed: u64, until: f64) -> Vec<Hop> {
    let mut data = Data::new(seed);
    let (mut at, mut t) = (start, 0.0);
    let mut hops = Vec::new();
    loop {
        let degree = net.degree(at) as f64;
        t += -(1.0 - data.unit()).ln() / degree;
        if t > until {
            return hops;
        }
        let around: Vec<usize> = net
            .links
            .iter()
            .filter_map(|&(a, b)| match (a == at, b == at) {
                (true, _) => Some(b),
                (_, true) => Some(a),
                _ => None,
            })
            .collect();
        let to = around[((data.unit() * around.len() as f64) as usize).min(around.len() - 1)];
        hops.push(Hop {
            from: at,
            to,
            depart: t,
        });
        at = to;
    }
}

/// Where the hopper is at time `t`: the node it left, the node it is going
/// to, and how far along that crossing it is (0 sits on the first, 1 on the
/// second). A crossing takes [`HOP_S`], or less if the next one sets out
/// sooner.
pub fn hopper_state(start: usize, hops: &[Hop], t: f64) -> (usize, usize, f64) {
    let mut here = start;
    for (i, h) in hops.iter().enumerate() {
        if t < h.depart {
            break;
        }
        let next = hops.get(i + 1).map_or(f64::INFINITY, |n| n.depart);
        let take = HOP_S.min(next - h.depart);
        if t < h.depart + take {
            return (h.from, h.to, (t - h.depart) / take);
        }
        here = h.to;
    }
    (here, here, 0.0)
}

/// The two-node heat: what each of the pair holds after `t`, from one unit
/// on the first (`F-D.pair.heat.*`).
pub fn pair_heat(t: f64) -> [f64; 2] {
    let col = expm_t(&PAIR.laplacian(), -t).column(0);
    [col[0], col[1]]
}

/// The first node's amplitude in the two-state quantum system (ħ = 1) with
/// the pair's Laplacian as its energy: (1 + e^{−2it}) / 2, as (real, imag).
pub fn pair_amplitude(t: f64) -> [f64; 2] {
    [0.5 * (1.0 + (2.0 * t).cos()), -0.5 * (2.0 * t).sin()]
}

/// The heat's version of that same number: (1 + e^{−2t}) / 2 on the real
/// line, as (real, imag).
pub fn pair_heat_point(t: f64) -> [f64; 2] {
    [0.5 * (1.0 + (-2.0 * t).exp()), 0.0]
}

/// How strongly one of four hundred nodes glows: its heat as a share of the
/// hottest node's now (so the spread stays visible while it thins).
pub fn big_glow(heat: f64, hottest: f64) -> f64 {
    0.07 + 0.88 * (heat / hottest.max(1e-12)).clamp(0.0, 1.0).sqrt()
}

// ------------------------------------------------------------ the pictures

/// The kite as 12_1 to 12_3 draw it: dim links, a panel disc and an amber
/// glow on each node (the glow follows the heat), the numbers on top.
struct HeatNet {
    links: Vec<Mob>,
    discs: Vec<Mob>,
    glows: [Mob; 4],
    numbers: Vec<Mob>,
}

impl HeatNet {
    fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let mut links = Vec::new();
        for &(a, b) in KITE.links {
            let l = polyline(vec![node(a), node(b)])
                .style(faded_line(Palette::dim(), 4.0, 0.7))
                .build();
            links.push(stage.arena_mut().add(l));
        }
        let mut discs = Vec::new();
        let mut glows = Vec::new();
        let mut numbers = Vec::new();
        for i in 0..KITE.n {
            let d = Circle::new()
                .radius(NODE_R)
                .arc_center(node(i))
                .style(Style::default().fill(panel(), 1.0).stroke(Palette::ink(), 3.5, 1.0))
                .build();
            discs.push(stage.arena_mut().add(d));
            let g = Circle::new()
                .radius(NODE_R - 0.03)
                .arc_center(node(i))
                .style(flat(substance(), 0.0))
                .build();
            glows.push(stage.arena_mut().add(g));
            numbers.push(kit.text(stage, &(i + 1).to_string(), 34.0, Palette::ink(), node(i))?);
        }
        Ok(Self {
            links,
            discs,
            glows: [glows[0], glows[1], glows[2], glows[3]],
            numbers,
        })
    }

    fn all(&self) -> Vec<Mob> {
        let mut out = self.links.clone();
        out.extend(&self.discs);
        out.extend(&self.glows);
        out.extend(&self.numbers);
        out
    }
}

/// What 12_2 and 12_3 drive from the heat clock: each node's glow and each
/// bar's height, from one updater.
fn drive_heat(stage: &mut Stage<'_>, net: &HeatNet, bars: [Mob; 4], clock: Mob, bars_on: Mob) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let glows = net.glows;
    let mut last = (f64::NAN, f64::NAN);
    stage.add_updater(
        anchor,
        move |s, _| {
            let t = s.tracker_value(clock).unwrap_or(0.0);
            let on = s.tracker_value(bars_on).unwrap_or(0.0);
            if (t.to_bits(), on.to_bits()) == (last.0.to_bits(), last.1.to_bits()) {
                return;
            }
            last = (t, on);
            let h = heat_at(t);
            for i in 0..4 {
                s.set_fill(glows[i], None, Some(glow(h[i])), None, false);
                let _ = s.set_points(bars[i], &corners(&bar_corners(BAR_X[i], BAR_BASE, BAR_W, BAR_H, h[i])));
                s.set_fill(bars[i], None, Some(0.9 * on), None, false);
            }
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// Rows of whole numbers as strings.
fn whole_rows(m: &Mat) -> Vec<Vec<String>> {
    (0..m.n())
        .map(|i| m.row(i).iter().map(|&x| whole(x)).collect())
        .collect()
}

/// A panel of 12_5 and 12_6: the first node's value as a dot on a complex
/// plane (its orbit as a faint circle) and the two nodes' bars beside it.
struct PairPanel {
    still: Vec<Mob>,
    dot: Mob,
    bars: [Mob; 2],
    anchor: Mob,
}

impl PairPanel {
    /// A panel with its complex-plane origin at `origin`; the dot and the
    /// bars follow `state(t)` = (point, [bar, bar]) for `clock`'s `t`.
    fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        origin: Vec3,
        clock: Mob,
        state: impl Fn(f64) -> ([f64; 2], [f64; 2]) + 'static,
    ) -> fmn::Result<Self> {
        let at = move |p: [f64; 2]| v(origin[0] + PANEL_UNIT * p[0], origin[1] + PANEL_UNIT * p[1]);
        let mut still = Vec::new();
        for (a, b) in [([-0.15, 0.0], [1.15, 0.0]), ([0.0, -0.85], [0.0, 0.85])] {
            let l = polyline(vec![at(a), at(b)])
                .style(faded_line(Palette::faint(), 2.5, 1.0))
                .build();
            still.push(stage.arena_mut().add(l));
        }
        let orbit = Circle::new()
            .radius(PANEL_UNIT * 0.5)
            .arc_center(at([0.5, 0.0]))
            .style(faded_line(Palette::dim(), 2.5, 0.7))
            .build();
        still.push(stage.arena_mut().add(orbit));
        let dot = stage.arena_mut().add(
            Dot::new()
                .point(at([1.0, 0.0]))
                .radius(0.12)
                .style(flat(result(), 1.0))
                .build(),
        );
        let base = origin[1] - PANEL_UNIT * 0.85;
        let bx = [origin[0] + PANEL_UNIT * 1.5, origin[0] + PANEL_UNIT * 1.5 + 0.7];
        let mut bars = Vec::new();
        for &x in &bx {
            let b = Polygon::new(bar_corners(x, base, 0.5, PANEL_BAR_H, 0.5))
                .style(flat(substance(), 0.9))
                .build();
            bars.push(stage.arena_mut().add(b));
        }
        let floor = polyline(vec![v(bx[0] - 0.4, base), v(bx[1] + 0.4, base)])
            .style(faded_line(Palette::dim(), 2.5, 1.0))
            .build();
        still.push(stage.arena_mut().add(floor));
        for (k, &x) in bx.iter().enumerate() {
            still.push(kit.text(stage, &(k + 1).to_string(), 26.0, Palette::dim(), v(x, base - 0.35))?);
        }
        let anchor = stage.arena_mut().add(VMobject::new());
        let (d, pair) = (dot, [bars[0], bars[1]]);
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = s.tracker_value(clock).unwrap_or(0.0);
                if t.to_bits() == last.to_bits() {
                    return;
                }
                last = t;
                let (p, h) = state(t);
                s.move_to(d, at(p), ORIGIN);
                for i in 0..2 {
                    let _ = s.set_points(pair[i], &corners(&bar_corners(bx[i], base, 0.5, PANEL_BAR_H, h[i])));
                }
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self {
            still,
            dot,
            bars: [bars[0], bars[1]],
            anchor,
        })
    }

    /// What fades in or out with the panel.
    fn mobs(&self) -> Vec<Mob> {
        let mut out = self.still.clone();
        out.extend([self.dot, self.bars[0], self.bars[1]]);
        out
    }
}

/// 12_5, as the i goes in: `old` gives way to `new` (the exponent) while the
/// panel's clock winds back to zero, so that the panel stands at its start
/// (the dot on 1, the first bar full) under the exponent with the i, ready to
/// run. It had settled to equal bars under the heat's exponent, and left so
/// it stood over the pause's "does it still settle?" with the heat's answer
/// on it (the dense read of the scored draft).
fn put_the_i_in(stage: &mut Stage<'_>, old: Mob, new: Mob, clock: Mob) -> fmn::Result<()> {
    let put_in = vec![
        fade_out_anim(stage, old, 0.4)?,
        fade_in_anim(stage, new, 0.5)?,
        stage.prepare(tween_linear(clock, 0.0, REWIND_S)?)?,
    ];
    stage.play_prepared(put_in)?;
    Ok(())
}

// -------------------------------------------------------------- the scene

pub struct HeatAndChance {
    pub kit: Kit,
}

impl SceneConstruct for HeatAndChance {
    fn name(&self) -> &str {
        "12_heat_and_chance"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, teal, gold, amber) = (Palette::ink(), rule(), result(), substance());

        // ---- 12_1: heat on node four; the rule is minus L
        let line = say(&kit, stage, "12_1")?;
        let net = HeatNet::build(&kit, stage)?;
        let clock = stage.add_value_tracker(0.0);
        let bars_on = stage.add_value_tracker(0.0);
        let mut bar_mobs = Vec::new();
        for &x in &BAR_X {
            let b = Polygon::new(bar_corners(x, BAR_BASE, BAR_W, BAR_H, 0.0))
                .style(flat(amber, 0.0))
                .build();
            bar_mobs.push(stage.arena_mut().add(b));
        }
        let bars = [bar_mobs[0], bar_mobs[1], bar_mobs[2], bar_mobs[3]];
        drive_heat(stage, &net, bars, clock, bars_on)?;
        // the bars stand on the scene from now, at no opacity; 12_2 raises
        // `bars_on` and they rise (they were built and driven but never put
        // on the scene, so no bar was ever drawn)
        for &b in &bars {
            stage.add_to_scene(b)?;
        }
        let grid = MatrixGrid::build(&kit, stage, &whole_rows(&minus_l()), GRID_PT, teal, GRID_AT)?;
        let grid_name = kit.tex(stage, "-L", 48.0, teal, &[], v(GRID_AT[0], GRID_AT[1] + GRID_NAME_DY))?;
        // the grid's diagonal is empty until the arrows have left along the links
        let diagonal: Vec<Mob> = (0..4).map(|i| grid.cells[i][i]).collect();
        let diag_levels = Levels::build(stage, &diagonal, 0.0)?;
        let mut first = net.all();
        first.extend([grid.group, grid_name]);
        fade_in_all(stage, &first, 0.8)?;
        // small arrows leave each node along its links
        let mut arrows = Vec::new();
        for &(a, b) in KITE.links {
            for (from, to) in [(a, b), (b, a)] {
                let (p, q) = (node(from), node(to));
                let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
                let len = dx.hypot(dy);
                let (s0, s1) = (NODE_R + 0.08, NODE_R + 0.6);
                let tail = v(p[0] + dx / len * s0, p[1] + dy / len * s0);
                let tip = v(p[0] + dx / len * s1, p[1] + dy / len * s1);
                let m = polyline(arrow_points(tail, tip, 0.14))
                    .style(faded_line(teal, 3.5, 1.0))
                    .build();
                arrows.push(stage.arena_mut().add(m));
            }
        }
        line.wait_for(stage, "Each link carries heat")?;
        fade_in_all(stage, &arrows, 0.7)?;
        line.wait_for(stage, "as fast as the difference between them")?;
        let mut diag: Vec<Box<dyn Animation>> = Vec::new();
        for i in 0..4 {
            diag.push(stage.prepare(diag_levels.tween(i, 1.0, 0.8)?)?);
        }
        stage.play_prepared(diag)?;
        kit.hold(stage)?;

        // ---- 12_2: the exponential: the heat spreads and levels out
        let line = say(&kit, stage, "12_2")?;
        let label = kit.text(stage, "heat", 30.0, Palette::dim(), v(BARS_MID, 1.45))?;
        let mut names = Vec::new();
        for (i, &x) in BAR_X.iter().enumerate() {
            names.push(kit.text(stage, &(i + 1).to_string(), 26.0, Palette::dim(), v(x, BAR_BASE - 0.35))?);
        }
        let mut enter: Vec<Box<dyn Animation>> = vec![
            fade_in_anim(stage, label, 0.6)?,
            stage.prepare(tween_linear(bars_on, 1.0, 0.6)?)?,
        ];
        for &m in &names {
            enter.push(fade_in_anim(stage, m, 0.6)?);
        }
        for &m in &arrows {
            enter.push(fade_out_anim(stage, m, 0.5)?);
        }
        line.wait_for(stage, "That rule is a matrix")?;
        stage.play_prepared(enter)?;
        let mut live = Vec::new();
        for (i, &x) in BAR_X.iter().enumerate() {
            live.push(kit.live_value(
                stage,
                move |s| heat_at(s.tracker_value(clock).unwrap_or(0.0))[i],
                2,
                false,
                28.0,
                ink,
                v(x + 0.24, BAR_BASE - 0.8),
                RIGHT,
            )?);
        }
        line.wait_for(stage, "The heat spreads")?;
        play!(stage; tween_linear(clock, HEAT_T, 4.4)?);
        line.wait_for(stage, "A quarter on each node")?;
        let total = kit.text(
            stage,
            &format!("total: {}, always", whole(heat_at(HEAT_T).iter().sum())),
            30.0,
            Palette::dim(),
            v(BARS_MID, 1.95),
        )?;
        fade_in_all(stage, &[total], 0.5)?;
        kit.hold(stage)?;

        // ---- 12_3: heat, or the chance of finding a hopper: the same matrix
        let line = say(&kit, stage, "12_3")?;
        let chance = kit.text(stage, "chance", 30.0, gold, v(BARS_MID, 1.45))?;
        let swap = vec![fade_out_anim(stage, label, 0.5)?, fade_in_anim(stage, chance, 0.5)?];
        stage.play_prepared(swap)?;
        line.wait_for(stage, "or as the chance of finding a random hopper")?;
        let token = stage.arena_mut().add(
            Dot::new()
                .point(token_at(node(shown::HEAT_START)))
                .radius(TOKEN_R)
                .style(flat(gold, 1.0))
                .build(),
        );
        let schedule = hopper_schedule(&KITE, shown::HEAT_START, HOPPER_SEED, HOPPER_UNTIL);
        let hop_clock = stage.add_value_tracker(0.0);
        let rider = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            rider,
            move |s, _| {
                let t = s.tracker_value(hop_clock).unwrap_or(0.0);
                if t.to_bits() == last.to_bits() {
                    return;
                }
                last = t;
                let (from, to, f) = hopper_state(shown::HEAT_START, &schedule, t);
                let (p, q) = (node(from), node(to));
                s.move_to(
                    token,
                    token_at(v(p[0] + f * (q[0] - p[0]), p[1] + f * (q[1] - p[1]))),
                    ORIGIN,
                );
            },
            true,
        )?;
        stage.add_to_scene(rider)?;
        fade_in_all(stage, &[token], 0.4)?;
        play!(stage; tween_linear(hop_clock, 7.0, 7.0)?);
        let per_tick = format!("{}/{}", KITE.degree(2), 2 * KITE.links.len());
        let strip = lower_third(
            &kit,
            stage,
            "precisely:",
            &format!(
                "a hopper that leaves along each link at rate one (books write it exp(tQ)); \
                 one that takes one hop per tick would favour the busy node, {per_tick}"
            ),
        )?;
        fade_in_all(stage, &[strip], 0.5)?;
        kit.hold(stage)?;

        // ---- 12_4: four hundred nodes, one stroke
        let line = say(&kit, stage, "12_4")?;
        // the four nodes stay up for "On four nodes you could do this by
        // hand" (they went at the line's first frame, and the sentence ran
        // over a black stage); they go as "On four hundred" is said
        line.wait_for(stage, "On four hundred")?;
        let mut gone = net.all();
        gone.extend([chance, total, token, strip, grid.group, grid_name]);
        gone.extend(&names);
        gone.extend(&live);
        gone.extend(bars);
        fade_out_all(stage, &gone, 0.6)?;
        let side = shown::BIG_SIDE;
        let (src_r, src_c) = shown::BIG_SOURCE;
        let half = (side as f64 - 1.0) / 2.0;
        let at_node = move |r: usize, c: usize| {
            v(
                BIG_AT[0] + BIG_STEP * (c as f64 - half),
                BIG_AT[1] - BIG_STEP * (r as f64 - half),
            )
        };
        let mut lattice = Vec::new();
        for k in 0..side {
            let across = polyline(vec![at_node(k, 0), at_node(k, side - 1)])
                .style(faded_line(Palette::faint(), 1.5, 1.0))
                .build();
            let down = polyline(vec![at_node(0, k), at_node(side - 1, k)])
                .style(faded_line(Palette::faint(), 1.5, 1.0))
                .build();
            lattice.push(stage.arena_mut().add(across));
            lattice.push(stage.arena_mut().add(down));
        }
        let mut dots = Vec::new();
        for r in 0..side {
            for c in 0..side {
                let d = Dot::new()
                    .point(at_node(r, c))
                    .radius(0.085)
                    .style(flat(amber, 0.0))
                    .build();
                dots.push(stage.arena_mut().add(d));
            }
        }
        let big_clock = stage.add_value_tracker(0.0);
        let big_anchor = stage.arena_mut().add(VMobject::new());
        let heated = dots.clone();
        let mut last_t = f64::NAN;
        stage.add_updater(
            big_anchor,
            move |s, _| {
                let t = s.tracker_value(big_clock).unwrap_or(0.0);
                if t.to_bits() == last_t.to_bits() {
                    return;
                }
                last_t = t;
                let h = grid_heat(side, t, src_r, src_c);
                let hottest = h.iter().copied().fold(0.0, f64::max);
                for (&m, &x) in heated.iter().zip(&h) {
                    s.set_opacity(m, big_glow(x, hottest), false);
                }
            },
            true,
        )?;
        stage.add_to_scene(big_anchor)?;
        let nodes_note = kit.text(
            stage,
            &format!("{} nodes", fact("F-V.12_4.nodes")?),
            32.0,
            ink,
            v(4.3, 2.3),
        )?;
        let mut grid_in = lattice.clone();
        grid_in.extend(&dots);
        grid_in.push(nodes_note);
        fade_in_all(stage, &grid_in, 0.7)?;
        line.wait_for(stage, "the exponential still answers in one stroke")?;
        let walks_label = kit.text(
            stage,
            "walks of 40 steps, corner to corner:",
            26.0,
            Palette::dim(),
            v(3.6, 0.9),
        )?;
        let walks_value: f64 = fact("F-D.big.walks")?
            .parse()
            .map_err(|e| fail(format!("fact F-D.big.walks: {e}")))?;
        let counter = stage.add_value_tracker(0.0);
        let counted = kit.live_count(stage, counter, 38.0, gold, v(3.6, 0.0))?;
        fade_in_all(stage, &[walks_label], 0.5)?;
        play!(stage;
            tween_linear(big_clock, shown::BIG_TIME, 3.6)?,
            tween_linear(counter, walks_value, 3.6)?,
        );
        // a breath: the music alone after the four hundred (the score bead)
        music_hold(&kit, stage)?;

        // ---- 12_5: the tanks were this all along; now put an i in
        let line = say(&kit, stage, "12_5")?;
        let mut clear = grid_in.clone();
        clear.extend([walks_label, counted]);
        stage.clear_updaters(big_anchor, true);
        fade_out_all(stage, &clear, 0.6)?;
        let pair_x = [-4.6, -2.2];
        let pair_y = 0.9;
        let mut nodes_two = Vec::new();
        let link = polyline(vec![v(pair_x[0], pair_y), v(pair_x[1], pair_y)])
            .style(faded_line(Palette::dim(), 4.0, 0.9))
            .build();
        nodes_two.push(stage.arena_mut().add(link));
        for (k, &x) in pair_x.iter().enumerate() {
            let d = Circle::new()
                .radius(NODE_R + 0.06)
                .arc_center(v(x, pair_y))
                .style(Style::default().fill(panel(), 1.0).stroke(ink, 3.5, 1.0))
                .build();
            nodes_two.push(stage.arena_mut().add(d));
            nodes_two.push(kit.text(stage, &(k + 1).to_string(), 34.0, ink, v(x, pair_y))?);
        }
        // the same two, as tanks: a tank for each, the link as two pipes
        let mut tanks_two = Vec::new();
        for &x in &pair_x {
            let w = Rectangle::new()
                .width(1.1)
                .height(1.3)
                .style(flat(amber, 0.5))
                .build()?
                .moved_to(v(x, pair_y));
            tanks_two.push(stage.arena_mut().add(w));
            let f = Rectangle::new()
                .width(1.3)
                .height(1.5)
                .style(Style::default().fill(ink, 0.0).stroke(ink, 3.5, 1.0))
                .build()?
                .moved_to(v(x, pair_y));
            tanks_two.push(stage.arena_mut().add(f));
        }
        for (dy, right) in [(0.3, true), (-0.3, false)] {
            let (a, b) = if right {
                (pair_x[0] + 0.7, pair_x[1] - 0.7)
            } else {
                (pair_x[1] - 0.7, pair_x[0] + 0.7)
            };
            let p = polyline(arrow_points(v(a, pair_y + dy), v(b, pair_y + dy), 0.16))
                .style(faded_line(Palette::dim(), 4.0, 1.0))
                .build();
            tanks_two.push(stage.arena_mut().add(p));
        }
        let pair_clock = stage.add_value_tracker(0.0);
        let heat_panel = PairPanel::build(&kit, stage, v(0.7, 0.9), pair_clock, |t| {
            (pair_heat_point(t), pair_heat(t))
        })?;
        let exponent = kit.tex(stage, r"e^{-tL}", 60.0, ink, &[("e", gold), ("tL", teal)], v(2.6, 3.0))?;
        let with_i = kit.tex(
            stage,
            r"e^{-itL}",
            60.0,
            ink,
            &[("e", gold), ("itL", teal)],
            v(2.6, 3.0),
        )?;
        line.wait_for(stage, "The two tanks were this all along")?;
        let mut ins = nodes_two.clone();
        ins.extend([exponent]);
        fade_in_all(stage, &ins, 0.7)?;
        line.wait_for(stage, "Two nodes, one link")?;
        fade_in_all(stage, &heat_panel.mobs(), 0.6)?;
        let mut morph: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &nodes_two {
            morph.push(fade_out_anim(stage, m, 0.8)?);
        }
        for &m in &tanks_two {
            morph.push(fade_in_anim(stage, m, 0.8)?);
        }
        stage.play_prepared(morph)?;
        line.wait_for(stage, "settling at half and half")?;
        play!(stage; tween_linear(pair_clock, 3.0, 3.0)?);
        line.wait_for(stage, "Now put the letter i into the exponent")?;
        put_the_i_in(stage, exponent, with_i, pair_clock)?;
        predict_hold(&kit, stage)?;

        // ---- 12_6: never: the chance sloshes across, and back
        let line = say(&kit, stage, "12_6")?;
        let mut away = tanks_two.clone();
        away.extend(heat_panel.mobs());
        away.extend([with_i]);
        stage.clear_updaters(heat_panel.anchor, true);
        fade_out_all(stage, &away, 0.6)?;
        let slosh = stage.add_value_tracker(0.0);
        let left = PairPanel::build(&kit, stage, v(-6.0, 0.4), slosh, |t| (pair_heat_point(t), pair_heat(t)))?;
        let right = PairPanel::build(&kit, stage, v(0.9, 0.4), slosh, |t| {
            (pair_amplitude(t), quantum_pair(t))
        })?;
        let title_l = kit.tex(stage, r"e^{-tL}", 52.0, ink, &[("e", gold), ("tL", teal)], v(-3.9, 2.9))?;
        let title_r = kit.tex(
            stage,
            r"e^{-itL}",
            52.0,
            ink,
            &[("e", gold), ("itL", teal)],
            v(3.0, 2.9),
        )?;
        let cap_l = kit.text(stage, "settles", 28.0, Palette::dim(), v(-3.9, 2.35))?;
        let cap_r = kit.text(stage, "goes round", 28.0, Palette::dim(), v(3.0, 2.35))?;
        let mut ins = left.mobs();
        ins.extend(right.mobs());
        ins.extend([title_l, title_r, cap_l, cap_r]);
        fade_in_all(stage, &ins, 0.7)?;
        line.wait_for(stage, "Remember what i did to growth")?;
        play!(stage; tween_linear(slosh, sound::SLOSH_END, sound::SLOSH_S)?);
        let strip = lower_third(
            &kit,
            stage,
            "precisely:",
            "a two-state quantum system, \u{127} = 1: exp(\u{2212}iHt) moves two complex numbers; \
             each bar is the squared size of one",
        )?;
        fade_in_all(stage, &[strip], 0.5)?;
        // once the line is spoken, the same sweep again, heard (sound that
        // teaches): two pitches a panel, as loud as their bars; heat's come
        // together and stay, the quantum pair's trade places and come back.
        // It is the chapter's last music alone, so no hold follows it.
        kit.hold(stage)?;
        play!(stage; tween_linear(slosh, 0.0, REWIND_S)?);
        let wav = sound::slosh()?;
        kit.sonify(stage, sound::SLOSH_WINDOW_S, &format!("{}:12_6", kit.chapter), &wav)?;
        let start = stage.scene().time().to_f64();
        play!(stage; tween_linear(slosh, sound::SLOSH_END, sound::SLOSH_S)?);
        wait_until(stage, start + sound::SLOSH_WINDOW_S)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};

    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{fact, numbers, show, testing};
    use super::*;
    use crate::model::grid_corner_walks;

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// 12_1 and 12_2 (the dense read of the scored draft): the heat's bars
    /// stand clear of the matrix -L (its left bracket is 0.4 right of the
    /// last bar's edge, and the bars are evenly spaced), and the words
    /// centred over them stand over their middle.
    #[test]
    fn the_bars_stand_clear_of_the_matrix() {
        run(|stage| {
            let kit = Kit::new()?;
            let grid = MatrixGrid::build(&kit, stage, &whole_rows(&minus_l()), GRID_PT, Palette::teal(), GRID_AT)?;
            let left = GRID_AT[0] - grid.size.0 / 2.0;
            let last_edge = BAR_X[3] + BAR_W / 2.0;
            assert!(
                left - last_edge >= 0.35,
                "the bracket at {left:.2}, the last bar's edge {last_edge:.2}"
            );
            let pitches: Vec<f64> = BAR_X.windows(2).map(|w| w[1] - w[0]).collect();
            assert!(pitches.iter().all(|p| (p - pitches[0]).abs() < 1e-9), "{pitches:?}");
            assert!(pitches[0] > BAR_W + 0.2, "bars apart: {pitches:?}");
            assert!((BARS_MID - (BAR_X[0] + BAR_X[3]) / 2.0).abs() < 1e-9);
            // the node 4 of the graph (right-most) is left of the first bar
            assert!(node(3)[0] + NODE_R + 0.3 < BAR_X[0] - BAR_W / 2.0);
            Ok(())
        });
    }

    /// 12_1 (the pre-final check): the grid's name "-L" keeps the 0.25 a
    /// text keeps from the frame's top edge (it stood 0.23), and stands clear
    /// of the grid's top bracket.
    #[test]
    fn the_name_of_the_grid_keeps_off_the_top_edge() {
        use fmn::core::constants::FRAME_Y_RADIUS;
        run(|stage| {
            let kit = Kit::new()?;
            let grid = MatrixGrid::build(&kit, stage, &whole_rows(&minus_l()), GRID_PT, Palette::teal(), GRID_AT)?;
            let name = kit.tex(
                stage,
                "-L",
                48.0,
                Palette::teal(),
                &[],
                v(GRID_AT[0], GRID_AT[1] + GRID_NAME_DY),
            )?;
            let (top, foot) = (stage.get_top(name)[1], stage.get_bottom(name)[1]);
            assert!(
                FRAME_Y_RADIUS - top >= 0.25,
                "the name's top {top:.2} is {:.2} from the frame's top",
                FRAME_Y_RADIUS - top
            );
            let bracket = stage.get_top(grid.group)[1];
            assert!(
                foot - bracket >= 0.1,
                "the name's foot {foot:.2}, the bracket's top {bracket:.2}"
            );
            Ok(())
        });
    }

    /// 12_5 (the dense read of the scored draft): the pair's panel has
    /// settled to equal bars under the heat's exponent; when the i goes in it
    /// is wound back to its start, the dot on 1 and the first bar full with
    /// the second flat, so the pause asks "does it still settle?" over a
    /// panel that has not run.
    #[test]
    fn the_panel_stands_at_its_start_when_the_i_goes_in() {
        run(|stage| {
            let kit = Kit::new()?;
            let origin = v(0.7, 0.9);
            let clock = stage.add_value_tracker(0.0);
            let panel = PairPanel::build(&kit, stage, origin, clock, |t| (pair_heat_point(t), pair_heat(t)))?;
            let old = kit.tex(stage, r"e^{-tL}", 60.0, Palette::ink(), &[], v(2.6, 3.0))?;
            let new = kit.tex(stage, r"e^{-itL}", 60.0, Palette::ink(), &[], v(2.6, 3.0))?;
            fade_in_all(stage, &[old], 0.1)?;
            stage.play(tween_linear(clock, 3.0, 0.5)?)?;
            let settled = [stage.get_height(panel.bars[0]), stage.get_height(panel.bars[1])];
            assert!(
                (settled[0] - settled[1]).abs() < 0.05 * PANEL_BAR_H,
                "the heat's panel has settled first: {settled:?}"
            );
            put_the_i_in(stage, old, new, clock)?;
            let at = stage.tracker_value(clock).unwrap_or(f64::NAN);
            assert!(at.abs() < 1e-9, "the clock is back at zero: {at}");
            let (first, second) = (stage.get_height(panel.bars[0]), stage.get_height(panel.bars[1]));
            assert!((first - PANEL_BAR_H).abs() < 1e-6, "the first bar is full: {first}");
            assert!(second < 0.02 * PANEL_BAR_H, "the second is flat: {second}");
            let dot = stage.get_center(panel.dot);
            assert!(
                (dot[0] - (origin[0] + PANEL_UNIT)).abs() < 1e-6 && (dot[1] - origin[1]).abs() < 1e-6,
                "the dot is on 1: {dot:?}"
            );
            Ok(())
        });
    }

    fn close(a: &[f64], b: &[f64], tol: f64) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < tol)
    }

    /// 12_1 and 12_2: the rule's grid is `F-D.L` negated, the heat at each
    /// time is the facts' row, the total is one always (tested from time zero
    /// to far past levelling), and it levels at a quarter each.
    #[test]
    fn the_heat_is_the_facts_and_nothing_is_created_or_lost() {
        let l = KITE.laplacian();
        let rows: Vec<Vec<f64>> = (0..4).map(|i| l.row(i).to_vec()).collect();
        assert_eq!(fact("F-D.L").unwrap(), "2,-1,-1,0;-1,2,-1,0;-1,-1,3,-1;0,0,-1,1");
        for (i, row) in rows.iter().enumerate() {
            assert!(close(
                &minus_l().row(i).iter().map(|x| -x).collect::<Vec<_>>(),
                row,
                1e-12
            ));
        }
        // the diagonal of the rule is minus the number of links at each node
        for i in 0..4 {
            assert_eq!(-minus_l().get(i, i), KITE.degree(i) as f64);
        }
        for t in shown::HEAT_TIMES {
            let key = if t.fract() == 0.0 {
                format!("{t:.0}")
            } else {
                format!("{t}")
            };
            let want = numbers(&format!("F-D.heat.t{key}")).unwrap();
            assert!(close(&heat_at(t), &want, 1e-3), "t = {t}");
        }
        for t in [0.0, 1e-9, 0.3, 2.0, 5.0, 50.0, 1e4] {
            let h = heat_at(t);
            assert!((h.iter().sum::<f64>() - 1.0).abs() < 1e-9, "t = {t}");
            assert!(h.iter().all(|&x| x >= -1e-12), "t = {t}");
        }
        assert!(close(&heat_at(200.0), &numbers("F-V.12_2.level").unwrap(), 5e-3));
        assert!(close(&heat_at(200.0), &[0.25; 4], 1e-9));
        assert_eq!(whole(heat_at(HEAT_T).iter().sum()), "1");
        // brighter for more heat, never solid and never gone
        assert!(glow(0.0) > 0.0 && glow(1.0) < 1.0 && glow(0.25) < glow(0.5));
    }

    /// 12_3: the hopper crosses only links, waits as long as the number of
    /// its links says, spends a quarter of its time on each node in the
    /// long run (the levelling the bars show), and a hopper that takes one
    /// hop per tick would favour the busy node, 3/8, as the facts say.
    #[test]
    fn the_hopper_leaves_at_rate_one_per_link_and_levels_at_a_quarter() {
        let far = 200_000.0;
        let hops = hopper_schedule(&KITE, 3, HOPPER_SEED, far);
        let linked = |a: usize, b: usize| KITE.links.iter().any(|&l| l == (a, b) || l == (b, a));
        assert!(hops.iter().all(|h| linked(h.from, h.to)));
        assert!(
            hops.windows(2)
                .all(|w| w[1].from == w[0].to && w[1].depart > w[0].depart)
        );
        // time spent on each node, and the mean wait where it is
        let mut time = [0.0_f64; 4];
        let mut waits = [(0.0_f64, 0usize); 4];
        let mut now = 0.0;
        let mut at = 3;
        for h in &hops {
            time[at] += h.depart - now;
            waits[at].0 += h.depart - now;
            waits[at].1 += 1;
            now = h.depart;
            at = h.to;
        }
        for i in 0..4 {
            let share = time[i] / now;
            assert!((share - 0.25).abs() < 0.01, "node {i}: {share}");
            let mean = waits[i].0 / waits[i].1 as f64;
            let want = 1.0 / KITE.degree(i) as f64;
            assert!(
                (mean - want).abs() < 0.03 * want + 0.01,
                "node {i}: mean wait {mean}, want {want}"
            );
        }
        // the one-hop-per-tick hopper: visits in proportion to the links
        let mut data = Data::new(HOPPER_SEED + 1);
        let mut visits = [0usize; 4];
        let mut node_now = 3;
        let steps = 200_000;
        for _ in 0..steps {
            let around: Vec<usize> = KITE
                .links
                .iter()
                .filter_map(|&(a, b)| match (a == node_now, b == node_now) {
                    (true, _) => Some(b),
                    (_, true) => Some(a),
                    _ => None,
                })
                .collect();
            node_now = around[((data.unit() * around.len() as f64) as usize).min(around.len() - 1)];
            visits[node_now] += 1;
        }
        let per_tick = numbers("F-D.chance.per_tick").unwrap();
        for i in 0..4 {
            assert!((visits[i] as f64 / steps as f64 - per_tick[i]).abs() < 0.01, "node {i}");
        }
        assert_eq!(per_tick[2], 3.0 / 8.0);
        assert_eq!(format!("{}/{}", KITE.degree(2), 2 * KITE.links.len()), "3/8");
        // where the hopper is, between crossings and during them
        let (from, to, f) = hopper_state(3, &hops, 0.0);
        assert_eq!((from, to, f), (3, 3, 0.0));
        let h = hops[10];
        let (from, to, f) = hopper_state(3, &hops, h.depart + 0.5 * HOP_S.min(hops[11].depart - h.depart));
        assert_eq!((from, to), (h.from, h.to));
        assert!((f - 0.5).abs() < 1e-9);
    }

    /// 12_4: the four hundred nodes' numbers are the facts', and the heat
    /// there is conserved too.
    #[test]
    fn four_hundred_nodes_are_the_facts() {
        let (side, (r, c)) = (shown::BIG_SIDE, shown::BIG_SOURCE);
        let h = grid_heat(side, shown::BIG_TIME, r, c);
        assert_eq!(h.len(), 400);
        assert_eq!(fact("F-V.12_4.nodes").unwrap(), "400");
        assert_eq!(show(h[r * side + c], 4).unwrap(), fact("F-D.big.heat.source").unwrap());
        assert!((h.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        let walks = grid_corner_walks(side, shown::BIG_WALK_STEPS);
        assert_eq!(walks.to_string(), fact("F-D.big.walks").unwrap());
        assert_eq!(walks, 4_988_731_519_200);
        // the glow keeps the spread visible while it thins
        let hottest = h.iter().copied().fold(0.0, f64::max);
        assert!((big_glow(hottest, hottest) - 0.95).abs() < 1e-12);
        assert!(big_glow(0.0, hottest) > 0.0);
    }

    /// 12_5 and 12_6: the pair is the tanks, its heat is the facts', the
    /// dot goes round a circle in the quantum case with the bars trading
    /// places (all across at π/2, all back at π), and in the heat case it
    /// slides to one half.
    #[test]
    fn the_pair_settles_with_heat_and_sloshes_with_an_i() {
        assert_eq!(fact("F-D.pair.is_the_tanks").unwrap(), "true");
        for t in shown::PAIR_TIMES {
            let key = if t.fract() == 0.0 {
                format!("{t:.0}")
            } else {
                format!("{t}")
            };
            let want = numbers(&format!("F-D.pair.heat.t{key}")).unwrap();
            assert!(close(&pair_heat(t), &want, 1e-5), "heat, t = {t}");
            let want = numbers(&format!("F-D.pair.quantum.t{key}")).unwrap();
            assert!(close(&quantum_pair(t), &want, 1e-5), "quantum, t = {t}");
        }
        for t in [0.0, 0.2, 1.0, FRAC_PI_2, 2.5, PI] {
            let [re, im] = pair_amplitude(t);
            assert!(
                ((re - 0.5).hypot(im) - 0.5).abs() < 1e-12,
                "the dot is on its circle at t = {t}"
            );
            assert!(
                (re * re + im * im - quantum_pair(t)[0]).abs() < 1e-12,
                "squared size is the bar"
            );
            assert!((quantum_pair(t).iter().sum::<f64>() - 1.0).abs() < 1e-12);
            let [hre, him] = pair_heat_point(t);
            assert_eq!(him, 0.0);
            assert!(
                (hre - pair_heat(t)[0]).abs() < 1e-12,
                "the same node's value, with no i"
            );
        }
        assert!(close(&quantum_pair(FRAC_PI_2), &[0.0, 1.0], 1e-12), "all across at π/2");
        assert!(close(&quantum_pair(PI), &[1.0, 0.0], 1e-12), "all back at π");
        // with heat the first node never gets below one half
        for t in [0.5, 2.0, 10.0, 100.0] {
            assert!(pair_heat(t)[0] >= 0.5 - 1e-12);
        }
        assert!((pair_heat(100.0)[0] - 0.5).abs() < 1e-9);
        // the bars stand on the floor and are never flat
        let b = bar_corners(0.0, -1.0, 0.5, 2.0, 0.0);
        assert!(b[2][1] > b[0][1]);
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("12_heat_and_chance").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "heat",
            "chance",
            "total: 1, always",
            "400 nodes",
            "e^{-itL}",
            "goes round",
            // the strips wrap into lines
            "one hop per tick would favour the busy node, 3/8",
            "squared size of one",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
