//! Chapter 06, twenty-one patterns: the arrow test on the truss, where one
//! push at one joint moves every joint and is turned 64 degrees from the
//! move (06_1); the move swaps for K's softest pattern, whose push lines up
//! (0 degrees, the truss gold), and the wall of all 21 patterns on a ladder
//! spaced by rank, each rung's bar the log of its stiffness (06_2); the
//! quiz, which of two patterns is the softest (06_3); the softest against
//! the stiffest, 3.12 and 1,226 kN a mm, nearly four hundred times (06_4);
//! the five deck loads split into the patterns, each share divided by its
//! stiffness so that the soft ones win, and the sum that says so (06_5);
//! the truss rebuilt from the softest pattern alone, then from more, its
//! midspan sag wobbling around the true one and settling (06_6); and the
//! forces, which the same few patterns do not give, against a ladder of a
//! million (06_7).
//!
//! The rung order is chapter 04's: the softest pattern on top, the stiffest
//! at the foot. Every number is a fact (`K-P1.*`, `K-P2.*`, `K-P3.*`,
//! `K-Q.06_3.answer`), computed by `model::truss`.

use explainer_kit::animatic::wrap;
use explainer_kit::props::{names_to_search, precisely};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::library::TransformMatchingTex;
use fmn::prelude::*;

use super::draw::{
    QuizCard, arrow_points, fact, fade_in_all, fade_in_lagged, fade_out_all, faded_line, fail, flat, group, moves, own,
    pause, pushes, say, tween_linear,
};
use super::truss::{Layout, displaced, load_arrow, outline, plate, supports};
use crate::model::linalg::{Eigen, dot};
use crate::model::truss::{Member, Truss};

pub struct TwentyOnePatterns {
    pub kit: Kit,
}

// --------------------------------------------------------------- the model

/// What a joint's arrow is drawn from: its (right, up) at every joint.
pub type Arrows = Vec<(f64, f64)>;

/// A pattern (or any move) turned so that its biggest move is a sag, not a lift.
pub fn sag_oriented(t: &Truss, q: &[f64]) -> Vec<f64> {
    let free = t.free();
    let up: f64 = free.iter().zip(q).filter(|(d, _)| **d % 2 == 1).map(|(_, x)| *x).sum();
    let flip = if up.abs() > 1e-9 {
        up > 0.0
    } else {
        let big = q
            .iter()
            .copied()
            .fold(0.0f64, |s, x| if x.abs() > s.abs() { x } else { s });
        big > 0.0
    };
    if flip {
        q.iter().map(|x| -x).collect()
    } else {
        q.to_vec()
    }
}

/// `q` scaled so that its largest entry has size `peak`.
pub fn scaled_to(q: &[f64], peak: f64) -> Vec<f64> {
    let top = q.iter().fold(0.0f64, |s, x| s.max(x.abs())).max(1e-300);
    q.iter().map(|x| x * peak / top).collect()
}

/// The move (right, up) of every joint for moves of the free directions.
pub fn joint_moves(t: &Truss, free_moves: &[f64]) -> Vec<(f64, f64)> {
    let all = t.all_moves(free_moves);
    (0..t.joints.len()).map(|j| (all[2 * j], all[2 * j + 1])).collect()
}

/// The angle in degrees between `u` and the push `K u`.
pub fn turn_of(t: &Truss, u: &[f64]) -> f64 {
    let f = t.stiffness().mul_vec(u);
    let c = dot(u, &f) / (dot(u, u) * dot(&f, &f)).sqrt();
    c.clamp(-1.0, 1.0).acos().to_degrees()
}

/// What the arrow test draws for shape `k`: 0 is the move of one push at
/// one joint, 1 the softest pattern, 2 the stiffest; the move and the push
/// that holds it, over the free directions.
pub fn shape(t: &Truss, e: &Eigen, k: usize) -> (Vec<f64>, Vec<f64>) {
    match k {
        0 => {
            let f = t.lone_load();
            (t.moves(&f), f)
        }
        1 => {
            let q = sag_oriented(t, &e.vectors[0]);
            let f = t.stiffness().mul_vec(&q);
            (q, f)
        }
        _ => {
            let q = sag_oriented(t, &e.vectors[e.vectors.len() - 1]);
            let f = t.stiffness().mul_vec(&q);
            (q, f)
        }
    }
}

// ------------------------------------------------------------- the layout

/// The full-size truss of 06_1 and 06_2, 0.36 scene units a metre.
const MAIN: Layout = Layout {
    origin: [-6.5, -0.9, 0.0],
    scale: 0.36,
};
/// The longest arrow, scene units.
const REACH: f64 = 0.75;
/// The ladder of 21 rungs.
const RUNGS: usize = 21;
const LADDER_TOP: f64 = 3.25;
const LADDER_STEP: f64 = 0.3;
/// A small truss on a rung: its left foot and its scale.
const MINI_X: f64 = -6.3;
const MINI_SCALE: f64 = 0.05;
/// Where a rung's bar starts and its longest length.
const BAR_X: f64 = -4.4;
const BAR_LONG: f64 = 4.0;
const BAR_H: f64 = 0.16;

fn rung_y(i: usize) -> f64 {
    LADDER_TOP - LADDER_STEP * i as f64
}

/// A bar's length for stiffness `lambda`: the log of it, from a stub for the
/// softest to the full length for the stiffest.
pub fn bar_length(lambda: f64, softest: f64, stiffest: f64) -> f64 {
    let share = (lambda / softest).ln() / (stiffest / softest).ln();
    0.3 + (BAR_LONG - 0.3) * share
}

/// The share of the move each pattern holds for load `f`, before and after
/// dividing by its stiffness, each scaled so that the largest is 1:
/// `|qᵢ·f|` and `|qᵢ·f| / λᵢ`.
pub fn shares(e: &Eigen, f: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let before: Vec<f64> = e.vectors.iter().map(|q| dot(q, f).abs()).collect();
    let after: Vec<f64> = before.iter().zip(&e.values).map(|(b, l)| b / l).collect();
    let (hb, ha) = (
        before.iter().copied().fold(0.0f64, f64::max),
        after.iter().copied().fold(0.0f64, f64::max),
    );
    (
        before.iter().map(|b| b / hb).collect(),
        after.iter().map(|a| a / ha).collect(),
    )
}

/// A vertex list as the run of points a mobject keeps (and `set_points`
/// wants): a handle at the midpoint of each pair of vertices, as `polyline`
/// builds.
pub fn corners(vertices: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(2 * vertices.len());
    for w in vertices.windows(2) {
        out.push(w[0]);
        out.push([
            (w[0][0] + w[1][0]) / 2.0,
            (w[0][1] + w[1][1]) / 2.0,
            (w[0][2] + w[1][2]) / 2.0,
        ]);
    }
    if let Some(&last) = vertices.last() {
        out.push(last);
    }
    out
}

/// The members of `t` as segments between the displaced joints: `field` is
/// the moves of the free directions (metres), `mag` times magnified, laid on
/// `layout`.
pub fn deformed(t: &Truss, layout: Layout, field: &[f64], mag: f64) -> Vec<(Vec3, Vec3)> {
    let all = t.all_moves(field);
    t.members
        .iter()
        .map(|m: &Member| {
            (
                displaced(t, layout, &all, mag, 1.0, m.a),
                displaced(t, layout, &all, mag, 1.0, m.b),
            )
        })
        .collect()
}

/// A small truss for one rung, gold, bent by `field` (scaled so that its
/// largest move is `bend` metres); detached, one group.
fn mini_truss(stage: &mut Stage<'_>, t: &Truss, field: &[f64], y: f64, colour: Srgb) -> fmn::Result<Mob> {
    let layout = Layout {
        origin: [MINI_X, y - 0.08, 0.0],
        scale: MINI_SCALE,
    };
    let field = scaled_to(field, 1.4);
    let mut mobs = Vec::new();
    for (a, b) in deformed(t, layout, &field, 1.0) {
        mobs.push(
            stage
                .arena_mut()
                .add(polyline(vec![a, b]).style(faded_line(colour, 2.2, 1.0)).build()),
        );
    }
    group(stage, &mobs)
}

impl SceneConstruct for TwentyOnePatterns {
    fn name(&self) -> &str {
        "06_twenty_one_patterns"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let dim = Palette::dim();
        let t = Truss::pratt();
        let e = t.patterns();
        let joints = t.joints.len();

        // ---- 06_1: the arrow test on the truss
        say(&kit, stage, "06_1")?;
        let shapes: Vec<(Arrows, Arrows)> = (0..3)
            .map(|k| {
                let (u, f) = shape(&t, &e, k);
                // blue: the moves at every joint, its largest REACH long; orange:
                // the pushes, for one load its own scale, for a pattern the moves'
                (
                    joint_moves(&t, &scaled_to(&u, REACH)),
                    joint_moves(&t, &scaled_to(&f, REACH)),
                )
            })
            .collect();
        let turns: Vec<f64> = (0..3)
            .map(|k| {
                let (u, _) = shape(&t, &e, k);
                turn_of(&t, &u)
            })
            .collect();
        let frame = outline(stage, &t, MAIN, ink, 1.0);
        let feet = supports(stage, &t, MAIN);
        let mut blue = Vec::new();
        let mut orange = Vec::new();
        for j in 0..joints {
            let at = MAIN.joint(&t, j);
            blue.push(
                stage.arena_mut().add(
                    polyline(arrow_points(at, at, 0.17))
                        .style(faded_line(moves(), 7.0, 1.0))
                        .build(),
                ),
            );
            orange.push(
                stage.arena_mut().add(
                    polyline(arrow_points(at, at, 0.17))
                        .style(faded_line(pushes(), 4.0, 1.0))
                        .build(),
                ),
            );
        }
        let which = stage.add_value_tracker(0.0);
        let veil = stage.add_value_tracker(1.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        {
            let (blue, orange, shapes, turns) = (blue.clone(), orange.clone(), shapes.clone(), turns.clone());
            let layout = MAIN;
            let tt = t.clone();
            let mut shown = usize::MAX;
            let mut seen = f64::NAN;
            stage.add_updater(
                anchor,
                move |s, _| {
                    let k = s.tracker_value(which).unwrap_or(0.0).round().clamp(0.0, 2.0) as usize;
                    let a = s.tracker_value(veil).unwrap_or(1.0);
                    if k == shown && a.to_bits() == seen.to_bits() {
                        return;
                    }
                    let aligned = turns[k] < 0.05;
                    for j in 0..blue.len() {
                        let at = layout.joint(&tt, j);
                        let (bu, bf) = (shapes[k].0[j], shapes[k].1[j]);
                        let tip = |m: (f64, f64)| v(at[0] + m.0, at[1] + m.1);
                        let blen = bu.0.hypot(bu.1);
                        let olen = bf.0.hypot(bf.1);
                        if k != shown {
                            let _ = s.set_points(blue[j], &corners(&arrow_points(at, tip(bu), 0.17)));
                            let _ = s.set_points(orange[j], &corners(&arrow_points(at, tip(bf), 0.17)));
                            let c = if aligned { own() } else { moves() };
                            s.set_stroke(blue[j], Some(c), None, None, None, false);
                        }
                        // an arrow that is not there leaves no dot
                        s.set_stroke(
                            blue[j],
                            None,
                            None,
                            Some(if blen < 0.03 { 0.0 } else { a }),
                            None,
                            false,
                        );
                        s.set_stroke(
                            orange[j],
                            None,
                            None,
                            Some(if olen < 0.03 { 0.0 } else { a }),
                            None,
                            false,
                        );
                    }
                    shown = k;
                    seen = a;
                },
                true,
            )?;
        }
        stage.add_to_scene(anchor)?;
        let angle = kit.live_value(
            stage,
            {
                let turns = turns.clone();
                move |s| turns[s.tracker_value(which).unwrap_or(0.0).round().clamp(0.0, 2.0) as usize]
            },
            0,
            false,
            72.0,
            ink,
            v(4.9, 1.6),
            RIGHT,
        )?;
        let degrees = kit.text(stage, "\u{b0}", 72.0, ink, v(5.2, 1.7))?;
        let angle_words = kit.text(stage, "the push is turned from the move by", 26.0, dim, v(4.4, 2.6))?;
        let scale_words = kit.text(
            stage,
            "each set of arrows drawn to its own scale",
            24.0,
            dim,
            v(4.4, 0.5),
        )?;
        let mut ins: Vec<Mob> = frame.clone();
        ins.extend(&feet);
        ins.extend(&blue);
        ins.push(angle_words);
        ins.push(degrees);
        ins.push(scale_words);
        fade_in_all(stage, &ins, 0.8)?;
        fade_in_all(stage, &orange, 0.5)?;
        kit.hold(stage)?;

        // ---- 06_2: some moves come back as themselves: the softest pattern
        let line2 = say(&kit, stage, "06_2")?;
        line2.wait_for(stage, "But some moves come back as themselves")?;
        play!(stage; tween_linear(veil, 0.0, 0.3)?);
        stage.set_tracker_value(which, 1.0)?;
        play!(stage; tween_linear(veil, 1.0, 0.4)?);
        // the truss turns gold
        for &m in &frame {
            stage.set_stroke(m, Some(own()), None, None, None, false);
        }
        line2.wait_for(stage, "This table has twenty-one own patterns")?;
        // the wall: all 21 patterns as small trusses on a ladder, by rank
        let softest = e.values[0];
        let stiffest = e.values[RUNGS - 1];
        let mut rails: Vec<Mob> = Vec::new();
        let mut rungs: Vec<Mob> = Vec::new();
        for i in 0..RUNGS {
            let y = rung_y(i);
            let q = sag_oriented(&t, &e.vectors[i]);
            rungs.push(mini_truss(stage, &t, &q, y, own())?);
            let len = bar_length(e.values[i], softest, stiffest);
            rails.push(
                stage.arena_mut().add(
                    Rectangle::new()
                        .width(len)
                        .height(BAR_H)
                        .style(flat(own(), 0.9))
                        .build()?
                        .moved_to(v(BAR_X + len / 2.0, y)),
                ),
            );
        }
        let top_name = kit.text(stage, "softest", 24.0, dim, v(-0.9, rung_y(0)))?;
        let foot_name = kit.text(stage, "stiffest", 24.0, dim, v(-0.9, rung_y(RUNGS - 1)))?;
        let count = kit.text(
            stage,
            &format!("{} patterns, each with a stiffness of its own", fact("K-P1.count")?),
            28.0,
            ink,
            v(3.7, 0.2),
        )?;
        let mut leaving: Vec<Mob> = frame.clone();
        leaving.extend(&feet);
        leaving.extend(&blue);
        leaving.extend(&orange);
        leaving.extend([angle, degrees, angle_words, scale_words]);
        fade_out_all(stage, &leaving, 0.6)?;
        stage.remove_from_scene(anchor);
        let mut wall: Vec<Mob> = rungs.clone();
        wall.extend(&rails);
        fade_in_lagged(stage, &wall, 1.8, 0.04)?;
        fade_in_all(stage, &[top_name, foot_name, count], 0.5)?;
        kit.hold(stage)?;

        // ---- 06_3: your turn: which is the softest?
        let line3 = say(&kit, stage, "06_3")?;
        let card_at = v(3.55, 0.0);
        let card = QuizCard::build(&kit, stage, "06_3", card_at, (6.5, 5.6))?;
        let (centre, _, _) = card.content();
        // two patterns lifted out of the wall: the stiffest as A, the softest as B
        let a_field = sag_oriented(&t, &e.vectors[RUNGS - 1]);
        let b_field = sag_oriented(&t, &e.vectors[0]);
        let layout_a = Layout {
            origin: [centre[0] - 1.8, centre[1] + 0.45, 0.0],
            scale: 0.17,
        };
        let layout_b = Layout {
            origin: [centre[0] - 1.8, centre[1] - 1.0, 0.0],
            scale: 0.17,
        };
        let mut choices = Vec::new();
        for (layout, field) in [(layout_a, &a_field), (layout_b, &b_field)] {
            let field = scaled_to(field, 2.2);
            for (a, b) in deformed(&t, layout, &field, 1.0) {
                choices.push(
                    stage
                        .arena_mut()
                        .add(polyline(vec![a, b]).style(faded_line(own(), 3.0, 1.0)).build()),
                );
            }
        }
        let name_a = kit.text(stage, "A", 40.0, ink, v(centre[0] - 2.65, centre[1] + 0.75))?;
        let name_b = kit.text(stage, "B", 40.0, ink, v(centre[0] - 2.65, centre[1] - 0.7))?;
        let words_a = kit.text(
            stage,
            "neighbouring joints move opposite ways",
            22.0,
            dim,
            v(centre[0] + 0.3, centre[1] + 0.05),
        )?;
        let words_b = kit.text(
            stage,
            "the whole span sags as one",
            22.0,
            dim,
            v(centre[0] + 0.3, centre[1] - 1.45),
        )?;
        let ask = kit.text(
            stage,
            "which is the softest?",
            30.0,
            ink,
            v(centre[0], centre[1] + 1.45),
        )?;
        let mut ins = choices.clone();
        ins.extend([name_a, name_b, words_a, words_b, ask]);
        // the wall steps back while the card is up
        fade_out_all(stage, &[top_name, foot_name, count], 0.3)?;
        card.show_frame(stage, &ins, 0.6)?;
        line3.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // ---- 06_4: the single sag; the stiffest is nearly four hundred times stiffer
        let line4 = say(&kit, stage, "06_4")?;
        let answer = kit.text(
            stage,
            &format!("{}: B", fact("K-Q.06_3.answer")?),
            30.0,
            own(),
            v(card_at[0] + 0.9, card.ring_at()[1]),
        )?;
        card.land(stage, &[answer], 0.3)?;
        let soft_n = fact("K-P1.softest_mn_per_m")?;
        let stiff_n = grouped(fact("K-P1.stiffest_mn_per_m")?);
        let soft_tag = kit.text_left(
            stage,
            &format!("{soft_n} kN per mm"),
            34.0,
            own(),
            v(BAR_X + 0.7, rung_y(0)),
        )?;
        let stiff_tag = kit.text_left(
            stage,
            &format!("{stiff_n} kN per mm"),
            34.0,
            own(),
            v(BAR_X + BAR_LONG + 0.3, rung_y(RUNGS - 1)),
        )?;
        let times = kit.text(
            stage,
            &format!("\u{d7} {}", fact("K-P1.ratio")?),
            60.0,
            ink,
            v(2.6, 0.2),
        )?;
        line4.wait_for(stage, "The single sag")?;
        let mut away: Vec<Mob> = card.mobs();
        away.extend(&choices);
        away.extend([name_a, name_b, words_a, words_b, ask, answer]);
        fade_out_all(stage, &away, 0.5)?;
        fade_in_all(stage, &[soft_tag, stiff_tag], 0.6)?;
        line4.wait_for(stage, "nearly four hundred times")?;
        fade_in_all(stage, &[times], 0.6)?;
        kit.hold(stage)?;

        // ---- 06_5: split the load into the patterns; the soft ones win
        let line5 = say(&kit, stage, "06_5")?;
        fade_out_all(stage, &[soft_tag, stiff_tag, times], 0.4)?;
        let load = t.deck_load();
        let (before, after) = shares(&e, &load);
        let bars: Vec<Mob> = (0..RUNGS)
            .map(|i| {
                Ok(stage.arena_mut().add(
                    Rectangle::new()
                        .width(1.0)
                        .height(BAR_H)
                        .style(flat(pushes(), 0.95))
                        .build()?
                        .moved_to(v(BAR_X + 0.5, rung_y(i))),
                ))
            })
            .collect::<fmn::Result<_>>()?;
        let unit_bar = Rectangle::new().width(1.0).height(BAR_H).build()?.points().to_vec();
        let divide = stage.add_value_tracker(0.0);
        let divider = stage.arena_mut().add(VMobject::new());
        {
            let (bars, before, after, unit_bar) = (bars.clone(), before.clone(), after.clone(), unit_bar.clone());
            let mut last = f64::NAN;
            stage.add_updater(
                divider,
                move |s, _| {
                    let d = s.tracker_value(divide).unwrap_or(0.0);
                    if d.to_bits() == last.to_bits() {
                        return;
                    }
                    last = d;
                    let colour = {
                        let (a, b) = (pushes(), moves());
                        Srgb {
                            r: a.r * (1.0 - d) + b.r * d,
                            g: a.g * (1.0 - d) + b.g * d,
                            b: a.b * (1.0 - d) + b.b * d,
                        }
                    };
                    for (i, &bar) in bars.iter().enumerate() {
                        let len = (BAR_LONG * (before[i] * (1.0 - d) + after[i] * d)).max(0.02);
                        let pts: Vec<Vec3> = unit_bar
                            .iter()
                            .map(|p| [BAR_X + len / 2.0 + p[0] * len, rung_y(i) + p[1], p[2]])
                            .collect();
                        let _ = s.set_points(bar, &pts);
                        s.set_fill(bar, Some(colour), None, None, false);
                    }
                },
                true,
            )?;
        }
        stage.add_to_scene(divider)?;
        // the load: five equal orange arrows at the deck joints, drawn on a small truss
        let small = Layout {
            origin: [2.0, 2.0, 0.0],
            scale: 0.2,
        };
        let small_frame = outline(stage, &t, small, dim, 1.0);
        let deck_arrows: Vec<Mob> = t
            .deck_joints()
            .iter()
            .map(|&j| load_arrow(stage, small.joint(&t, j), 0.6, pushes()))
            .collect();
        let load_words = kit.text(stage, "the load", 26.0, pushes(), v(4.4, 1.0))?;
        let mut ins = small_frame.clone();
        ins.extend(&deck_arrows);
        ins.extend([load_words]);
        fade_in_all(stage, &ins, 0.6)?;
        line5.wait_for(stage, "Now split the load")?;
        // the stiffnesses' bars step back behind the shares
        for &r in &rails {
            stage.set_fill(r, None, Some(0.22), None, false);
        }
        fade_in_lagged(stage, &bars, 1.2, 0.04)?;
        line5.wait_for(stage, "Each share is divided by its stiffness")?;
        play!(stage; tween_linear(divide, 1.0, 1.8)?);
        // the sum, arriving from the carts' fractions
        let (fractions, f_spans) = kit.tex_spans(
            stage,
            r"u = \tfrac{1}{2} q_1 + \tfrac{1}{6} q_2",
            52.0,
            ink,
            &[],
            v(3.6, -1.6),
        )?;
        let (letters, l_spans) = kit.tex_spans(
            stage,
            r"u = \sum_i \frac{q_i \cdot f}{\lambda_i \, q_i \cdot q_i}\, q_i",
            52.0,
            ink,
            &[],
            v(3.6, -1.6),
        )?;
        fade_in_all(stage, &[fractions], 0.4)?;
        let morph = TransformMatchingTex::new(stage.arena_mut(), fractions, letters, &f_spans, &l_spans)?;
        play!(stage; morph.rt(1.2));
        let tag = kit.text(
            stage,
            "q\u{b7}f: how much of the load lies along pattern i",
            24.0,
            dim,
            v(3.6, -2.45),
        )?;
        let tag2 = kit.text(stage, "a library call finds all 21", 24.0, dim, v(3.6, -2.95))?;
        fade_in_all(stage, &[tag, tag2], 0.5)?;
        kit.hold(stage)?;

        // ---- 06_6: the softest pattern alone is within half a percent of the sag
        let line6 = say(&kit, stage, "06_6")?;
        let mut away = bars.clone();
        away.extend(&rungs);
        away.extend(&rails);
        away.extend(&small_frame);
        away.extend(&deck_arrows);
        away.extend([load_words, letters, tag, tag2, top_name, foot_name, count]);
        fade_out_all(stage, &away, 0.6)?;
        stage.remove_from_scene(divider);
        let big = Layout {
            origin: [-6.4, -0.2, 0.0],
            scale: 0.36,
        };
        let true_u = t.moves(&load);
        let counts: Vec<usize> = vec![1, 2, 3, 4, 5, 10, 15];
        let true_frame = deformed(&t, big, &true_u, 90.0);
        let rest = outline(stage, &t, big, Palette::faint(), 1.0);
        let truth: Vec<Mob> = true_frame
            .iter()
            .map(|&(a, b)| {
                stage
                    .arena_mut()
                    .add(polyline(vec![a, b]).style(faded_line(ink, 3.0, 1.0)).build())
            })
            .collect();
        let first = deformed(&t, big, &t.moves_from_patterns(&e, &load, counts[0]), 90.0);
        let approx: Vec<Mob> = first
            .iter()
            .map(|&(a, b)| {
                stage
                    .arena_mut()
                    .add(polyline(vec![a, b]).style(faded_line(own(), 6.0, 0.9)).build())
            })
            .collect();
        let step = stage.add_value_tracker(0.0);
        let rebuild = stage.arena_mut().add(VMobject::new());
        {
            let (approx, tt, e2, load2) = (approx.clone(), t.clone(), e.clone(), load.clone());
            let counts = counts.clone();
            let mut shown = usize::MAX;
            stage.add_updater(
                rebuild,
                move |s, _| {
                    let k = s
                        .tracker_value(step)
                        .unwrap_or(0.0)
                        .round()
                        .clamp(0.0, (counts.len() - 1) as f64) as usize;
                    if k == shown {
                        return;
                    }
                    shown = k;
                    let segs = deformed(&tt, big, &tt.moves_from_patterns(&e2, &load2, counts[k]), 90.0);
                    for (&m, (a, b)) in approx.iter().zip(segs) {
                        let _ = s.set_points(m, &corners(&[a, b]));
                    }
                },
                true,
            )?;
        }
        stage.add_to_scene(rebuild)?;
        let percents: Vec<f64> = counts
            .iter()
            .map(|n| {
                fact(&format!("K-P2.sag_percent.{n}"))?
                    .parse::<f64>()
                    .map_err(|x| fail(format!("sag percent {n}: {x}")))
            })
            .collect::<fmn::Result<_>>()?;
        let readout = kit.live_value(
            stage,
            {
                let percents = percents.clone();
                move |s| {
                    percents[s
                        .tracker_value(step)
                        .unwrap_or(0.0)
                        .round()
                        .clamp(0.0, (percents.len() - 1) as f64) as usize]
                }
            },
            1,
            false,
            72.0,
            own(),
            v(4.9, 1.3),
            RIGHT,
        )?;
        let percent_sign = kit.text(stage, "%", 54.0, own(), v(5.3, 1.35))?;
        let readout_words = kit.text(stage, "of the sag at the middle", 28.0, dim, v(4.2, 2.4))?;
        let how_many = kit.text(stage, "with the softest few patterns", 26.0, dim, v(4.2, 0.4))?;
        let caption = kit.text(
            stage,
            "adding patterns wobbles around the true sag, then settles",
            26.0,
            dim,
            v(0.0, -2.7),
        )?;
        let this_load = kit.text(
            stage,
            "this load: five equal pushes along the deck",
            26.0,
            dim,
            v(0.0, -3.3),
        )?;
        let mut ins: Vec<Mob> = rest.clone();
        ins.extend(&truth);
        ins.extend(&approx);
        ins.extend([readout_words, percent_sign, how_many]);
        fade_in_all(stage, &ins, 0.7)?;
        line6.wait_for(stage, "The softest pattern alone")?;
        fade_in_all(stage, &[caption], 0.5)?;
        for k in 1..counts.len() {
            pause(stage, 0.7)?;
            stage.set_tracker_value(step, k as f64)?;
        }
        fade_in_all(stage, &[this_load], 0.5)?;
        line6.wait_for(stage, "Good enough")?;
        kit.hold(stage)?;

        // ---- 06_7: not for what is inside it: the forces
        let line7 = say(&kit, stage, "06_7")?;
        let mut away: Vec<Mob> = rest.clone();
        away.extend(&truth);
        away.extend(&approx);
        away.extend([readout, readout_words, percent_sign, how_many, caption, this_load]);
        fade_out_all(stage, &away, 0.5)?;
        stage.remove_from_scene(rebuild);
        let counts_shown = [1usize, 5, 15, 21];
        let mut error_mobs = Vec::new();
        for (i, n) in counts_shown.iter().enumerate() {
            let words = format!(
                "{n} pattern{}: the worst force is off by {} kN",
                if *n == 1 { "" } else { "s" },
                fact(&format!("K-P2.force_error_kn.{n}"))?
            );
            let colour = if i == 0 { Palette::red() } else { ink };
            error_mobs.push(kit.text(stage, &words, 38.0, colour, v(0.0, 2.2 - 0.85 * i as f64))?);
        }
        let shrank = precisely(
            &kit,
            stage,
            "dividing by stiffness shrank the stiff patterns' share of the move; a force multiplies it back",
        )?;
        line7.wait_for(stage, "A force is a stretch times a stiffness")?;
        fade_in_all(stage, &[shrank], 0.5)?;
        line7.wait_for(stage, "with one pattern")?;
        fade_in_all(stage, &error_mobs[..1], 0.5)?;
        // the other three come while the line is still being said
        fade_in_lagged(stage, &error_mobs[1..], 1.2, 0.5)?;
        kit.hold(stage)?;
        // one closing frame, so that the chapter ends on a read and not on a
        // string of pauses: the card on the right, the wall of 21 beside a
        // ladder of a million on the left with the softest few lit
        let eight = fact("K-P2.lone_pct.8m")?;
        let four = fact("K-P2.lone_pct.4m")?;
        let need = fact("K-P2.lone_within_2p5.12m")?;
        let mut fading = error_mobs.clone();
        fading.push(shrank);
        fade_out_all(stage, &fading, 0.4)?;
        let card_at = v(3.45, 1.1);
        let card = plate(stage, 6.8, 5.0, card_at, own())?;
        let mut card_texts = Vec::new();
        let blocks: [(String, f64, Srgb, usize); 3] = [
            (
                "a few patterns tell you how a structure moves for this load, not everything inside it".to_owned(),
                28.0,
                ink,
                30,
            ),
            (
                format!(
                    "one sharp load needs many more: the softest alone gives {eight}% of the drop under a lone 100 kN load at 8 m, and {four}% at 4 m"
                ),
                24.0,
                dim,
                36,
            ),
            (
                format!("{need} patterns bring a load at the middle within 2.5%"),
                24.0,
                dim,
                36,
            ),
        ];
        let mut y = card_at[1] + 2.0;
        for (words, size, colour, width) in &blocks {
            for l in wrap(words, *width, 5) {
                y -= 0.4;
                card_texts.push(kit.text(stage, &l, *size, *colour, v(card_at[0], y))?);
            }
            y -= 0.25;
        }
        let wall_x = -5.6;
        let tall_x = -3.0;
        let mut ladders: Vec<Mob> = Vec::new();
        for i in 0..RUNGS {
            let c = if i < 3 { own() } else { dim };
            ladders.push(
                stage.arena_mut().add(
                    Rectangle::new()
                        .width(1.4)
                        .height(0.06)
                        .style(flat(c, 0.9))
                        .build()?
                        .moved_to(v(wall_x, 3.0 - 0.14 * i as f64)),
                ),
            );
        }
        // a million rungs, drawn as 140 whose spacing is a fraction of the wall's
        for i in 0..140 {
            let c = if i < 4 { own() } else { dim };
            ladders.push(
                stage.arena_mut().add(
                    Rectangle::new()
                        .width(1.4)
                        .height(0.012)
                        .style(flat(c, if i < 4 { 0.9 } else { 0.5 }))
                        .build()?
                        .moved_to(v(tall_x, 3.0 - 0.0205 * i as f64)),
                ),
            );
        }
        let wall_words = kit.text(stage, "21 patterns", 26.0, ink, v(wall_x, -0.35))?;
        let tall_words = kit.text(stage, "a million", 26.0, ink, v(tall_x, -0.35))?;
        let both_words = kit.text(
            stage,
            "a library call finds all 21;\nonly the softest few are asked for",
            22.0,
            dim,
            v((wall_x + tall_x) / 2.0, -1.2),
        )?;
        let finder = names_to_search(
            &kit,
            stage,
            &[
                "modal truncation",
                "static correction",
                "a library call finds all 21: numpy.linalg.eigh",
            ],
        )?;
        let mut ins = ladders.clone();
        ins.extend([card, wall_words, tall_words, both_words, finder]);
        ins.extend(&card_texts);
        fade_in_all(stage, &ins, 0.7)?;
        pause(stage, 3.0)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// `1226` as `1,226`.
fn grouped(s: &str) -> String {
    let digits: Vec<char> = s.chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::draw::{show, testing};
    use super::*;

    /// 06_1 and 06_2: one push at one joint is turned 64 degrees from the
    /// move it causes (the fact), the softest pattern is not turned at all,
    /// and the arrows the picture draws are those moves.
    #[test]
    fn the_lone_push_is_turned_and_the_softest_pattern_is_not() {
        let t = Truss::pratt();
        let e = t.patterns();
        let (u, f) = shape(&t, &e, 0);
        assert_eq!(show(turn_of(&t, &u), 0).unwrap(), fact("K-P3.lone_turn_deg").unwrap());
        // the lone move moves every free direction but the supports', and the push is one joint's
        assert!(u.iter().all(|x| x.abs() > 0.0));
        assert_eq!(f.iter().filter(|x| **x != 0.0).count(), 1);
        let (q, _) = shape(&t, &e, 1);
        assert!(turn_of(&t, &q) < 0.05);
        assert_eq!(
            show(turn_of(&t, &q), 1).unwrap(),
            fact("K-P3.softest_turn_deg").unwrap()
        );
        let (s, _) = shape(&t, &e, 2);
        assert!(turn_of(&t, &s) < 0.05, "every own pattern is lined up with its push");
        // a pattern is shown as a sag: its deck moves down
        let all = t.all_moves(&q);
        assert!(t.deck_joints().iter().all(|&j| all[2 * j + 1] <= 0.0));
        // arrows: the largest is REACH long, and a held joint has none
        let arrows = joint_moves(&t, &scaled_to(&q, REACH));
        let biggest = arrows.iter().map(|a| a.0.hypot(a.1)).fold(0.0f64, f64::max);
        // its largest entry is REACH, and a joint's arrow joins two of them
        assert!(biggest >= REACH - 1e-12 && biggest <= REACH * 2f64.sqrt() + 1e-12);
        assert_eq!(arrows[0], (0.0, 0.0));
    }

    /// 06_2 and 06_4: 21 patterns, the ladder's bars the log of the stiffness
    /// from a stub to the full length, and the softest and stiffest are the
    /// facts' 3.12 and 1,226 kN per mm, a ratio of 393.
    #[test]
    fn the_ladder_has_twenty_one_rungs_and_the_facts_stiffnesses() {
        let t = Truss::pratt();
        let e = t.patterns();
        assert_eq!(e.values.len().to_string(), fact("K-P1.count").unwrap());
        assert_eq!(e.values.len(), RUNGS);
        assert_eq!(
            show(e.values[0] / 1e6, 2).unwrap(),
            fact("K-P1.softest_mn_per_m").unwrap()
        );
        assert_eq!(
            show(e.values[RUNGS - 1] / 1e6, 0).unwrap(),
            fact("K-P1.stiffest_mn_per_m").unwrap()
        );
        assert_eq!(
            show(e.values[RUNGS - 1] / e.values[0], 0).unwrap(),
            fact("K-P1.ratio").unwrap()
        );
        let lens: Vec<f64> = e
            .values
            .iter()
            .map(|&l| bar_length(l, e.values[0], e.values[RUNGS - 1]))
            .collect();
        assert!((lens[0] - 0.3).abs() < 1e-12);
        assert!((lens[RUNGS - 1] - BAR_LONG).abs() < 1e-9);
        assert!(lens.windows(2).all(|w| w[0] <= w[1]));
        // rungs fit the frame with a margin
        assert!(rung_y(0) <= 3.5 && rung_y(RUNGS - 1) >= -3.5);
        assert_eq!(grouped("1226"), "1,226");
        assert_eq!(grouped("393"), "393");
    }

    /// 06_5: the soft patterns win: after dividing by stiffness the share of
    /// the stiffest is a tiny part of the softest's, and the shares are
    /// scaled so that the largest is one.
    #[test]
    fn dividing_by_stiffness_lets_the_soft_patterns_win() {
        let t = Truss::pratt();
        let e = t.patterns();
        let (before, after) = shares(&e, &t.deck_load());
        assert!(before.iter().copied().fold(0.0f64, f64::max) <= 1.0 + 1e-12);
        assert!((after.iter().copied().fold(0.0f64, f64::max) - 1.0).abs() < 1e-12);
        // the two facts' sags: one pattern gives 99.6% of the midspan sag
        let direct = t.moves(&t.deck_load());
        let one = t.moves_from_patterns(&e, &t.deck_load(), 1);
        let mid = t.free().iter().position(|&d| d == 2 * t.midspan() + 1).unwrap();
        assert_eq!(
            show(one[mid] / direct[mid] * 100.0, 1).unwrap(),
            fact("K-P2.sag_percent.1").unwrap()
        );
        // the stiffest patterns' shares are nearly gone
        let top: f64 = after[RUNGS - 5..].iter().copied().fold(0.0f64, f64::max);
        assert!(top < 0.05, "{top}");
    }

    /// 06_6 and 06_7: the readout steps through the facts, and the forces
    /// are off by the facts' 82, 44, 8 and 0 kN.
    #[test]
    fn the_sags_and_the_force_errors_are_the_facts() {
        let t = Truss::pratt();
        let e = t.patterns();
        let load = t.deck_load();
        let direct = t.moves(&load);
        let want = t.member_forces(&direct);
        let mid = t.free().iter().position(|&d| d == 2 * t.midspan() + 1).unwrap();
        for n in [1usize, 2, 3, 4, 5, 10, 15] {
            let u = t.moves_from_patterns(&e, &load, n);
            assert_eq!(
                show(u[mid] / direct[mid] * 100.0, 1).unwrap(),
                fact(&format!("K-P2.sag_percent.{n}")).unwrap(),
                "{n}"
            );
        }
        for n in [1usize, 5, 15, 21] {
            let u = t.moves_from_patterns(&e, &load, n);
            let off = t
                .member_forces(&u)
                .iter()
                .zip(&want)
                .fold(0.0f64, |s, (a, b)| s.max((a - b).abs()));
            assert_eq!(
                show(off / 1e3, 0).unwrap(),
                fact(&format!("K-P2.force_error_kn.{n}")).unwrap(),
                "{n}"
            );
        }
        // the deformed members keep the truss's own shape at no magnification
        let rest = deformed(&t, MAIN, &vec![0.0; t.free().len()], 90.0);
        for (m, (a, b)) in t.members.iter().zip(rest) {
            assert_eq!(a, MAIN.joint(&t, m.a));
            assert_eq!(b, MAIN.joint(&t, m.b));
        }
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("06_twenty_one_patterns").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "the push is turned from the move by",
            "21 patterns, each with a stiffness of its own",
            "which is the softest?",
            "3.12 kN per mm",
            "1,226 kN per mm",
            "\u{d7} 393",
            "adding patterns wobbles around the true sag, then settles",
            "this load: five equal pushes along the deck",
            "1 pattern: the worst force is off by 82 kN",
            "21 patterns: the worst force is off by 0 kN",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never sets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 35.0 && ran.seconds < 120.0, "{} s", ran.seconds);
    }

    #[test]
    fn the_quiz_is_asked_and_counted_down_as_section_five_asks() {
        let a = testing::audit_quiz("06_3").unwrap();
        assert!((a.window.1 - a.window.0 - a.seconds).abs() <= 0.15, "{a:?}");
        assert_eq!(a.seconds, 5.0);
        assert!(a.card_shown <= a.question_ends, "{a:?}");
        assert!(a.ring_armed <= a.question_ends, "{a:?}");
        assert!(a.window.0 >= a.question_ends - 1e-9, "{a:?}");
        assert!(a.spoken_inside.is_empty(), "{a:?}");
        assert!(a.answer_lands >= a.window.1 - 1e-9, "{a:?}");
        if let Some((next, at)) = &a.next_line {
            assert!(*at >= a.window.1 - 0.05, "{next} starts inside the countdown");
        }
    }
}
