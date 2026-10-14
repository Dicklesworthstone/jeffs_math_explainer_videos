//! Chapter 09, follow the arrows: a matrix is read as a rule, and the rule
//! is followed. The machine of 01_4 turns into a field (09_1): what comes
//! out of J for each position is not where you land but where you are
//! headed. Tiny steps along the arrows, four, then sixteen, then none at
//! all, trace a smooth path (09_2). Following the whole field for a time t
//! carries each point to its finish, and the matrix that does the carrying
//! is e^{tA}; the rule's four entries then slide to a spiral and the arrows
//! and trails re-form (09_3). The series shows why: take its rate, and every
//! term slides one place over and leaves one copy of A out front (09_4).
//! Following for a time s and then a time t more is one longer run (09_5),
//! the staged trip and the single run land on the same dots, and the matrix
//! is the rule while its exponential is where the rule takes you (09_6).
//!
//! Every number is a fact or a model value pinned by the tests: the step
//! counts are `shown::TURN_STEPS`, the spiral is `F-M.spiral.rule`, the
//! year is `F-B.peano.year`.

use std::f64::consts::PI;

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, fade_all, live_bar, play, tween, v};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use super::draw::{
    Flow, FlowSpec, Plane, Velocity, arrow, arrow_points, chip, corners, fade_in_all, faded_line, flat, good, map2,
    music_hold, predict_hold, result, rule, run, say, turning, tween_linear,
};
use super::flow::{
    FIELD_COLS, LiveMatrix, RuleTrackers, drive_rule, fade_in_anim, fade_out_anim, fade_out_drifting, field_starts,
    path, precisely_tex, tiny_path, velocity_ends,
};
use crate::facts::{cited, shown, spiral_rule};
use crate::model::{Mat, expm_t, quarter_turn};

// ------------------------------------------------------------------ choices

/// The plane of the field sits left of centre, so the rule's grid and its
/// readouts have the right of the frame.
const PLANE: Plane = Plane {
    centre: [-2.9, 0.15, 0.0],
    unit: 1.3,
};
/// Plane units of arrow per unit of speed, and the arrow's head, on screen.
const GAIN: f64 = 0.22;
const HEAD: f64 = 0.13;
/// How long the whole field is followed at 09_3 (time units).
const FLOW_T: f64 = 2.0;
/// 09_5 and 09_6: one trip in two stages, s and then t more.
const STAGE_S: f64 = 1.0;
const STAGE_T: f64 = 1.5;
/// Time units followed per second in 09_5 and 09_6: both stages and the
/// single run go at the same pace, so the last dots meet at the same moment.
const SPEED: f64 = 0.5;
/// 09_2: the one dot whose steps are drawn starts one unit from the centre
/// and follows the quarter turn for the time of half a circle, as 06_4 did.
const TRAVELLER: [f64; 2] = [1.0, 0.0];
const TRAVELLER_TIME: f64 = PI;
/// The positions fed to the machine at 09_1: (column, row) of the field's
/// grid, three columns by four rows, spread over the field.
const SAMPLES: [(usize, usize); 12] = [
    (1, 1),
    (5, 1),
    (8, 1),
    (1, 3),
    (5, 3),
    (8, 3),
    (1, 4),
    (5, 4),
    (8, 4),
    (1, 6),
    (5, 6),
    (8, 6),
];
/// The centre of the right column, where the matrices sit.
const COLUMN_X: f64 = 4.9;
/// The two small planes of 09_5 and 09_6, the bars under them, and the
/// scene units a bar spends on one unit of time.
const SMALL_UNIT: f64 = 0.9;
const SMALL_Y: f64 = 1.2;
const BAR_Y: f64 = -0.7;
const BAR_UNIT: f64 = 1.5;

// ------------------------------------------------------------- the geometry

/// The grid index of position (column, row).
pub fn sample_index(col: usize, row: usize) -> usize {
    row * FIELD_COLS + col
}

/// 09_1: the arrow the machine hands back for position `p`, `slide` of the
/// way (0 to 1) from sitting at the centre, as long as the rule says, to
/// sitting on the tip of its input at the field's arrow length `gain`.
/// Returns (tail, head) in plane units.
pub fn sliding(rule: &Mat, p: [f64; 2], gain: f64, slide: f64) -> ([f64; 2], [f64; 2]) {
    let out = rule.apply(&p);
    let k = 1.0 + (gain - 1.0) * slide;
    let tail = [p[0] * slide, p[1] * slide];
    (tail, [tail[0] + k * out[0], tail[1] + k * out[1]])
}

/// Term k as TeX: I, tA, t²A²/2!, …
pub fn term_tex(k: u32) -> String {
    match k {
        0 => "I".to_owned(),
        1 => "tA".to_owned(),
        _ => format!(r"\frac{{t^{k}A^{k}}}{{{k}!}}"),
    }
}

// ------------------------------------------------------------- the machine

/// The twelve arrows of 09_1: each input is the position as an arrow from
/// the centre; each output is what the rule hands back, which slides to sit
/// on the tip of its input, shrinks and turns teal.
struct Machine {
    anchor: Mob,
    slide: Mob,
    inputs: Vec<Mob>,
    outputs: Vec<Mob>,
}

impl Machine {
    fn build(stage: &mut Stage<'_>, rule_m: &Mat, samples: &[[f64; 2]]) -> fmn::Result<Self> {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for &p in samples {
            let a = polyline(arrow_points(PLANE.centre, PLANE.p(p[0], p[1]), HEAD))
                .style(faded_line(Palette::dim(), 3.0, 1.0))
                .build();
            inputs.push(stage.arena_mut().add(a));
            let (from, to) = sliding(rule_m, p, GAIN, 0.0);
            let b = polyline(arrow_points(PLANE.p(from[0], from[1]), PLANE.p(to[0], to[1]), HEAD))
                .style(faded_line(turning(), 3.5, 1.0))
                .build();
            outputs.push(stage.arena_mut().add(b));
        }
        let slide = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let (rule_m, samples, outs) = (rule_m.clone(), samples.to_vec(), outputs.clone());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let u = s.tracker_value(slide).unwrap_or(0.0);
                if u.to_bits() == last.to_bits() {
                    return;
                }
                last = u;
                let c = interpolate_color(turning(), rule(), u);
                for (&m, &p) in outs.iter().zip(&samples) {
                    let (from, to) = sliding(&rule_m, p, GAIN, u);
                    let pts = arrow_points(PLANE.p(from[0], from[1]), PLANE.p(to[0], to[1]), HEAD);
                    let _ = s.set_points(m, &corners(&pts));
                    s.set_stroke(m, Some(c), None, None, None, false);
                }
            },
            true,
        )?;
        Ok(Self {
            anchor,
            slide,
            inputs,
            outputs,
        })
    }
}

// ----------------------------------------------------------- the series row

/// A row of mobjects laid left to right, their left edges `gap` apart from
/// `left`, centred on `y`; returns each one's centre x.
pub fn lay_row(stage: &mut Stage<'_>, mobs: &[Mob], left: f64, y: f64, gap: f64) -> Vec<f64> {
    let mut x = left;
    let mut centres = Vec::with_capacity(mobs.len());
    for &m in mobs {
        let w = stage.get_width(m);
        let c = x + w / 2.0;
        stage.move_to(m, v(c, y), ORIGIN);
        centres.push(c);
        x += w + gap;
    }
    centres
}

/// How many terms of the series 09_4 writes before "…".
const SHOWN_TERMS: usize = 4;
const ROW_Y: f64 = 1.55;
const RATE_Y: f64 = -0.1;

/// The series of 09_4: e^{tA} = I + tA + t²A²/2! + t³A³/3! + …, each term
/// its own mobject so a row of rates can slide under them.
struct SeriesRow {
    label: Mob,
    terms: Vec<Mob>,
    pluses: Vec<Mob>,
    dots: Mob,
    /// The centres of the terms, then of the closing dots.
    xs: Vec<f64>,
}

fn series_row(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<SeriesRow> {
    let ink = Palette::ink();
    let label = kit.tex(stage, r"e^{tA}=", 46.0, ink, &[("e", result()), ("tA", rule())], ORIGIN)?;
    let mut terms = Vec::new();
    let mut pluses = Vec::new();
    for k in 0..SHOWN_TERMS {
        terms.push(kit.tex(stage, &term_tex(k as u32), 44.0, ink, &[("A", rule())], ORIGIN)?);
        pluses.push(kit.tex(stage, "+", 44.0, ink, &[], ORIGIN)?);
    }
    let dots = kit.tex(stage, r"\cdots", 44.0, ink, &[], ORIGIN)?;
    let mut order = vec![label];
    for k in 0..SHOWN_TERMS {
        order.push(terms[k]);
        order.push(pluses[k]);
    }
    order.push(dots);
    let total = order.iter().map(|&m| stage.get_width(m)).sum::<f64>() + 0.25 * (order.len() as f64 - 1.0);
    let centres = lay_row(stage, &order, -total / 2.0, ROW_Y, 0.25);
    let mut xs: Vec<f64> = (0..SHOWN_TERMS).map(|k| centres[1 + 2 * k]).collect();
    xs.push(centres[centres.len() - 1]);
    Ok(SeriesRow {
        label,
        terms,
        pluses,
        dots,
        xs,
    })
}

/// A shift animation of `m` by `dx` over `rt` seconds.
fn slide_by(m: Mob, dx: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(m.animate().set_anim_args(run(rt))?.shift([dx, 0.0, 0.0])?)
}

/// A track for the time bars of 09_5 and 09_6: `w` wide from `x`, faint,
/// with its outline.
fn track(stage: &mut Stage<'_>, x: f64, w: f64) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(w)
        .height(0.22)
        .style(
            Style::default()
                .fill(turning(), 0.25)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(v(x + w / 2.0, BAR_Y));
    Ok(stage.arena_mut().add(r))
}

/// What a flow on a small plane is made of: dots only, and a trail behind
/// every fourth.
fn small_spec(plane: Plane, starts: &[[f64; 2]]) -> FlowSpec {
    FlowSpec {
        plane,
        starts: starts.to_vec(),
        dot_radius: 0.045,
        dot_color: Palette::ink(),
        dot_level: 0.0,
        velocity: None,
        trails: (0..starts.len()).step_by(4).collect(),
        trail_color: result(),
        trail_width: 3.0,
    }
}

// -------------------------------------------------------------- the scene

pub struct FollowTheArrows {
    pub kit: Kit,
}

impl SceneConstruct for FollowTheArrows {
    fn name(&self) -> &str {
        "09_follow_the_arrows"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let j = quarter_turn();
        let starts = field_starts();
        let (ink, teal, gold) = (Palette::ink(), rule(), result());
        let exp_colors = [("e", gold), ("tA", teal)];

        // the field: ink dots, a teal arrow on each, a gold trail behind
        // every other one; one set of trackers drives the lot
        let spec = FlowSpec {
            plane: PLANE,
            starts: starts.clone(),
            dot_radius: 0.05,
            dot_color: ink,
            dot_level: 0.0,
            velocity: Some(Velocity {
                rule: map2(&j),
                scale: GAIN,
                color: teal,
                width: 3.0,
                head: HEAD,
            }),
            trails: (0..starts.len()).step_by(2).collect(),
            trail_color: gold,
            trail_width: 3.0,
        };
        let flow = Flow::build(stage, spec.clone())?;
        let trackers = RuleTrackers::new(stage, &j);
        drive_rule(stage, &flow, &spec, trackers)?;
        stage.add_to_scene(flow.group)?;
        // the dot at the centre, drawn on purpose: it never moves
        let centre = stage.arena_mut().add(
            Dot::new()
                .point(PLANE.centre)
                .radius(0.085)
                .style(flat(gold, 1.0))
                .build(),
        );
        let [dots_level, arrows_level, trails_level] = flow.levels;

        // ---- 09_1: the machine becomes a field
        let line = say(&kit, stage, "09_1")?;
        play!(stage; tween(dots_level, 0.55, 0.8)?, fade_in(stage.arena_mut(), centre, ORIGIN, 1.0)?.rt(0.8));
        let samples: Vec<[f64; 2]> = SAMPLES.iter().map(|&(c, r)| starts[sample_index(c, r)]).collect();
        let machine = Machine::build(stage, &j, &samples)?;
        stage.add_to_scene(machine.anchor)?;
        line.wait_for(stage, "Read the matrix a new way")?;
        let rule_label = kit.tex(stage, "J=", 50.0, teal, &[], v(COLUMN_X - 2.1, 2.4))?;
        let rule_grid = LiveMatrix::build(
            &kit,
            stage,
            v(COLUMN_X, 2.4),
            2,
            2,
            (1.25, 0.8),
            46.0,
            1,
            teal,
            |i, k| Box::new(trackers.rule_entry(i, k)),
        )?;
        fade_in_all(stage, &[rule_label, rule_grid.brackets], 0.6)?;
        line.wait_for(stage, "Feed it your position")?;
        fade_in_all(stage, &machine.inputs, 0.6)?;
        line.wait_for(stage, "and what comes back")?;
        fade_in_all(stage, &machine.outputs, 0.6)?;
        line.wait_for(stage, "is your velocity")?;
        play!(stage; tween(machine.slide, 1.0, 1.6)?);
        // the slid arrows are the field's own: hand over to it
        let mut handover: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(arrows_level, 1.0, 0.6)?)?];
        for &m in machine.inputs.iter().chain(&machine.outputs) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            handover.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(handover)?;
        let caption_a = kit.text(
            stage,
            "what comes out is not where you land.",
            30.0,
            Palette::dim(),
            v(PLANE.centre[0], -2.95),
        )?;
        let caption_b = kit.text(
            stage,
            "It is where you are headed.",
            30.0,
            teal,
            v(PLANE.centre[0], -3.4),
        )?;
        fade_in_all(stage, &[caption_a, caption_b], 0.6)?;

        // ---- 09_2: tiny steps along the arrows
        let line = say(&kit, stage, "09_2")?;
        let four = shown::TURN_STEPS[0] as usize;
        let sixteen = shown::TURN_STEPS[1] as usize;
        let mut dim: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(arrows_level, 0.12, 0.6)?)?,
            stage.prepare(tween(dots_level, 0.22, 0.6)?)?,
        ];
        for &m in &[caption_a, caption_b] {
            dim.push(fade_out_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(dim)?;
        let (_, tip) = velocity_ends(&j, TRAVELLER, GAIN);
        let rider = stage.arena_mut().add(
            Dot::new()
                .point(PLANE.p(TRAVELLER[0], TRAVELLER[1]))
                .radius(0.085)
                .style(flat(gold, 1.0))
                .build(),
        );
        let rider_arrow = arrow(
            stage,
            PLANE.p(TRAVELLER[0], TRAVELLER[1]),
            PLANE.p(tip[0], tip[1]),
            teal,
            3.0,
        )?;
        fade_in_all(stage, &[rider, rider_arrow], 0.4)?;
        let formula = kit.tex(
            stage,
            r"\left(I+\frac{t}{n}A\right)^{n}",
            52.0,
            ink,
            &[("A", teal)],
            v(COLUMN_X, 0.85),
        )?;
        let steps_note = kit.text(stage, "n tiny steps", 28.0, Palette::dim(), v(COLUMN_X, 0.0))?;
        fade_in_all(stage, &[formula, steps_note], 0.6)?;
        let on_screen = |pts: Vec<[f64; 2]>| pts.into_iter().map(|p| PLANE.p(p[0], p[1])).collect::<Vec<_>>();
        let four_path = stage.arena_mut().add(
            polyline(on_screen(tiny_path(&j, TRAVELLER_TIME, TRAVELLER, four)))
                .style(faded_line(teal, 4.0, 1.0))
                .build(),
        );
        let sixteen_path = stage.arena_mut().add(
            polyline(on_screen(tiny_path(&j, TRAVELLER_TIME, TRAVELLER, sixteen)))
                .style(faded_line(teal, 4.0, 1.0))
                .build(),
        );
        let smooth = stage.arena_mut().add(
            polyline(on_screen(path(&j, TRAVELLER_TIME, TRAVELLER, 72)))
                .style(faded_line(gold, 5.0, 1.0))
                .build(),
        );
        line.wait_for(stage, "Four.")?;
        let four_label = kit.text(stage, &format!("{four} steps"), 32.0, teal, v(COLUMN_X, -0.7))?;
        play!(stage; show_creation(four_path).rt(1.2), fade_in(stage.arena_mut(), four_label, ORIGIN, 1.0)?.rt(0.6));
        line.wait_for(stage, "Sixteen.")?;
        let sixteen_label = kit.text(stage, &format!("{sixteen} steps"), 32.0, teal, v(COLUMN_X, -1.4))?;
        play!(stage;
            fade_out(stage.arena_mut(), four_path, ORIGIN, 1.0)?.rt(0.4),
            show_creation(sixteen_path).rt(1.2),
            fade_in(stage.arena_mut(), sixteen_label, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "Shrink them to nothing")?;
        let smooth_label = kit.text(stage, "a smooth path", 32.0, gold, v(COLUMN_X, -2.1))?;
        play!(stage;
            fade_out(stage.arena_mut(), sixteen_path, ORIGIN, 1.0)?.rt(0.4),
            show_creation(smooth).rt(1.6),
            fade_in(stage.arena_mut(), smooth_label, ORIGIN, 1.0)?.rt(0.6),
        );

        // ---- 09_3: follow the field for a time t
        let line = say(&kit, stage, "09_3")?;
        let mut restore: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(arrows_level, 1.0, 0.6)?)?,
            stage.prepare(tween(dots_level, 1.0, 0.6)?)?,
            stage.prepare(tween(trails_level, 0.7, 0.6)?)?,
        ];
        for m in [
            four_label,
            sixteen_label,
            smooth_label,
            formula,
            steps_note,
            rider,
            rider_arrow,
            smooth,
        ] {
            restore.push(fade_out_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(restore)?;
        let clock_label = kit.tex(stage, "t=", 44.0, turning(), &[], v(COLUMN_X - 0.9, 0.85))?;
        let time = trackers.time;
        let clock = kit.live_value(
            stage,
            move |s| s.tracker_value(time).unwrap_or(0.0),
            1,
            false,
            44.0,
            turning(),
            v(COLUMN_X - 0.4, 0.85),
            LEFT,
        )?;
        fade_in_all(stage, &[clock_label], 0.4)?;
        line.wait_for(stage, "Follow the arrows for a time t")?;
        stage.play(trackers.run_to(FLOW_T, 3.2)?)?;
        line.wait_for(stage, "The matrix that does the carrying")?;
        // the rule is a general A now, and e^{tA} is read off beside it
        let rule_name = kit.tex(stage, "A=", 50.0, teal, &[], v(COLUMN_X - 2.1, 2.4))?;
        let exp_label = kit.tex(stage, "e^{tA}=", 50.0, ink, &exp_colors, v(COLUMN_X - 2.35, -0.55))?;
        let exp_grid = LiveMatrix::build(
            &kit,
            stage,
            v(COLUMN_X + 0.1, -0.55),
            2,
            2,
            (1.4, 0.8),
            44.0,
            2,
            gold,
            |i, k| Box::new(trackers.exp_entry(i, k)),
        )?;
        let mut swap: Vec<Box<dyn Animation>> = vec![
            fade_out_anim(stage, rule_label, 0.4)?,
            fade_in_anim(stage, rule_name, 0.4)?,
            fade_in_anim(stage, exp_label, 0.5)?,
            fade_in_anim(stage, exp_grid.brackets, 0.5)?,
        ];
        // the rule slides from the quarter turn to an inward spiral; the
        // arrows and trails re-form while the entries of e^{tA} tick
        for b in trackers.slide_to(&spiral_rule(), 3.0)? {
            swap.push(stage.prepare(b)?);
        }
        stage.play_prepared(swap)?;

        // ---- 09_4: why the series is that motion
        let line = say(&kit, stage, "09_4")?;
        let mut gone = vec![
            clock_label,
            clock,
            rule_name,
            rule_grid.brackets,
            exp_label,
            exp_grid.brackets,
            centre,
        ];
        gone.extend(&rule_grid.cells);
        gone.extend(&exp_grid.cells);
        let mut away: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(arrows_level, 0.0, 0.6)?)?,
            stage.prepare(tween(trails_level, 0.0, 0.6)?)?,
            // (the dots went to a tenth: they still ran through the series'
            // first terms, grey dots under the letters)
            stage.prepare(tween(dots_level, 0.0, 0.6)?)?,
        ];
        for &m in &gone {
            // (a live readout's updaters are cleared first, or it would be
            // redrawn at full opacity every frame)
            away.push(fade_out_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(away)?;
        let row = series_row(&kit, stage)?;
        let mut row_mobs = vec![row.label, row.dots];
        row_mobs.extend(&row.terms);
        row_mobs.extend(&row.pluses);
        fade_in_all(stage, &row_mobs, 0.7)?;
        line.wait_for(stage, "Take its rate")?;
        // the rate of each term, written under it: A times the term before
        let rate_label = kit.text(stage, "its rate", 28.0, Palette::dim(), v(row.xs[0] - 2.6, RATE_Y))?;
        let zero = kit.tex(stage, "0", 44.0, Palette::dim(), &[], v(row.xs[0], RATE_Y))?;
        let mut small_a = Vec::new();
        let mut copies = Vec::new();
        for (k, &x) in row.xs.iter().enumerate().skip(1) {
            // the rate of term k sits under term k (the last one under the dots)
            let body = kit.tex(stage, &term_tex(k as u32 - 1), 44.0, ink, &[("A", teal)], v(x, RATE_Y))?;
            let half = stage.get_width(body) / 2.0;
            small_a.push(kit.tex(stage, "A", 44.0, teal, &[], v(x - half - 0.3, RATE_Y))?);
            copies.push(body);
        }
        let mut rate_mobs = vec![rate_label, zero];
        rate_mobs.extend(&small_a);
        rate_mobs.extend(&copies);
        fade_in_all(stage, &rate_mobs, 0.7)?;
        line.wait_for(stage, "Every term slides one place over")?;
        let mut slide: Vec<Box<dyn Animation>> = Vec::new();
        for k in 1..=SHOWN_TERMS {
            let dx = row.xs[k - 1] - row.xs[k];
            slide.push(stage.prepare(slide_by(small_a[k - 1], dx, 1.2)?)?);
            slide.push(stage.prepare(slide_by(copies[k - 1], dx, 1.2)?)?);
        }
        // nothing is left of the first term's rate: the zero falls away
        slide.push(fade_out_drifting(stage, zero, v(-0.4, 0.0), 1.2)?);
        stage.play_prepared(slide)?;
        line.wait_for(stage, "and leaves one copy of A out front")?;
        // every little A gathers in front of the row, and one is left
        let front = v(row.xs[0] - 1.1, RATE_Y);
        let mut gather: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &a) in small_a.iter().enumerate() {
            let go = a.animate().set_anim_args(run(1.0))?.move_to(front, ORIGIN)?;
            gather.push(stage.prepare(go)?);
            if i > 0 {
                gather.push(fade_out_anim(stage, a, 1.0)?);
            }
        }
        gather.push(fade_out_anim(stage, rate_label, 0.6)?);
        stage.play_prepared(gather)?;
        line.wait_for(stage, "So its rate is A")?;
        let rate_is = kit.tex(
            stage,
            r"\frac{d}{dt}\,e^{tA}=A\,e^{tA}",
            58.0,
            ink,
            &[("e", gold), ("tA", teal)],
            v(0.0, -1.75),
        )?;
        // the exact version comes in with the rate, so that it is up for the
        // rest of the line (it came with the last sentence, a second and a
        // half before the next picture)
        let exact = precisely_tex(
            &kit,
            stage,
            r"x'=Ax\ \text{ has the solution }\ x(t)=e^{tA}x(0)\text{, and no other}",
        )?;
        fade_in_all(stage, &[rate_is, exact], 0.7)?;

        // ---- 09_5: a time s, then a time t more
        let line = say(&kit, stage, "09_5")?;
        let mut tidy = vec![row.label, row.dots, rate_is, exact, small_a[0]];
        tidy.extend(&row.terms);
        tidy.extend(&row.pluses);
        tidy.extend(&copies);
        let left_plane = Plane {
            centre: v(-3.5, SMALL_Y),
            unit: SMALL_UNIT,
        };
        let right_plane = Plane {
            centre: v(3.5, SMALL_Y),
            unit: SMALL_UNIT,
        };
        let spiral = spiral_rule();
        let two_stage = Flow::build(stage, small_spec(left_plane, &starts))?;
        let two_time = stage.add_value_tracker(0.0);
        let map_left = {
            let spiral = spiral.clone();
            move |t: f64| map2(&expm_t(&spiral, t))
        };
        two_stage.drive(stage, two_time, map_left)?;
        stage.add_to_scene(two_stage.group)?;
        let one_run = Flow::build(stage, small_spec(right_plane, &starts))?;
        let one_time = stage.add_value_tracker(0.0);
        let map_right = {
            let spiral = spiral.clone();
            move |t: f64| map2(&expm_t(&spiral, t))
        };
        one_run.drive(stage, one_time, map_right)?;
        let mut tidy_up: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(dots_level, 0.0, 0.5)?)?,
            stage.prepare(tween(two_stage.levels[0], 1.0, 0.5)?)?,
            stage.prepare(tween(two_stage.levels[2], 0.8, 0.5)?)?,
        ];
        for &m in &tidy {
            tidy_up.push(fade_out_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(tidy_up)?;
        let (s_bar, t_bar) = (STAGE_S * BAR_UNIT, STAGE_T * BAR_UNIT);
        let left_edge = left_plane.centre[0] - (s_bar + t_bar) / 2.0;
        let s_track = track(stage, left_edge, s_bar)?;
        let t_track = track(stage, left_edge + s_bar, t_bar)?;
        let s_name = kit.tex(stage, "s", 40.0, ink, &[], v(left_edge + s_bar / 2.0, BAR_Y - 0.45))?;
        let t_name = kit.tex(
            stage,
            "t",
            40.0,
            ink,
            &[],
            v(left_edge + s_bar + t_bar / 2.0, BAR_Y - 0.45),
        )?;
        live_bar(stage, two_time, v(left_edge, BAR_Y), BAR_UNIT, 0.22, turning())?;
        let heading = kit.text(
            stage,
            "one trip, in two stages",
            30.0,
            Palette::dim(),
            v(left_plane.centre[0], 3.5),
        )?;
        fade_in_all(stage, &[s_track, t_track, s_name, t_name, heading], 0.6)?;
        line.wait_for(stage, "Follow the arrows for a time s")?;
        stage.play(tween_linear(two_time, STAGE_S, STAGE_S / SPEED)?)?;
        line.wait_for(stage, "Then for a time t more")?;
        stage.play(tween_linear(two_time, STAGE_S + STAGE_T, STAGE_T / SPEED)?)?;
        line.wait_for(stage, "What single matrix does the whole trip")?;
        let question = kit.text(stage, "?", 110.0, gold, v(0.0, SMALL_Y))?;
        fade_in_all(stage, &[question], 0.3)?;
        predict_hold(&kit, stage)?;

        // ---- 09_6: the same dots, one longer run
        let line = say(&kit, stage, "09_6")?;
        stage.add_to_scene(one_run.group)?;
        let right_edge = right_plane.centre[0] - (s_bar + t_bar) / 2.0;
        let whole_track = track(stage, right_edge, s_bar + t_bar)?;
        let whole_name = kit.tex(stage, "s+t", 40.0, ink, &[], v(right_plane.centre[0], BAR_Y - 0.45))?;
        live_bar(stage, one_time, v(right_edge, BAR_Y), BAR_UNIT, 0.22, turning())?;
        let heading2 = kit.text(
            stage,
            "one run, all at once",
            30.0,
            Palette::dim(),
            v(right_plane.centre[0], 3.5),
        )?;
        let mut enter: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(one_run.levels[0], 1.0, 0.5)?)?,
            stage.prepare(tween(one_run.levels[2], 0.8, 0.5)?)?,
            fade_out_anim(stage, question, 0.3)?,
        ];
        for m in [whole_track, whole_name, heading2] {
            enter.push(fade_in_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(enter)?;
        stage.play(tween_linear(one_time, STAGE_S + STAGE_T, (STAGE_S + STAGE_T) / SPEED)?)?;
        line.wait_for(stage, "One rule, run twice")?;
        let sum = kit.tex(
            stage,
            r"e^{tA}\,e^{sA}=e^{(s+t)A}",
            50.0,
            ink,
            &[("e", gold), ("tA", teal), ("sA", teal), ("(s+t)A", teal)],
            v(-0.6, -1.95),
        )?;
        let holds = chip(&kit, stage, "holds", 34.0, good(), v(5.0, -1.95))?;
        fade_in_all(stage, &[sum, holds], 0.6)?;
        line.wait_for(stage, "The matrix is the rule")?;
        // (a little above the lower third that follows: at -2.85 their feet
        // stood on its top edge)
        let rule_card = kit.tex(stage, r"A:\text{ the rule}", 42.0, teal, &[], v(-3.4, -2.65))?;
        let where_card = kit.tex(
            stage,
            r"e^{tA}:\text{ where it takes you}",
            42.0,
            gold,
            &[("tA", teal)],
            v(3.2, -2.65),
        )?;
        let between = arrow(stage, v(-1.2, -2.65), v(0.1, -2.65), ink, 4.0)?;
        fade_in_all(stage, &[rule_card, where_card, between], 0.7)?;
        let peano = lower_third(
            &kit,
            stage,
            "who and when:",
            &format!("Peano, {}: solving x′ = Ax this way", cited::PEANO_YEAR),
        )?;
        fade_in_all(stage, &[peano], 0.5)?;
        music_hold(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{numbers, testing};
    use super::*;
    use crate::chapters::flow::placed;
    use crate::model::{factorial, series};

    /// Term k of the series for e^{tA}: t^k A^k / k!.
    fn term(a: &Mat, t: f64, k: u32) -> Mat {
        a.pow(k).scale(t.powi(k as i32) / factorial(k) as f64)
    }

    /// The rate of term k as 09_4 writes it: A times term k − 1 (the first
    /// term, I, has rate zero).
    fn written_rate(a: &Mat, t: f64, k: u32) -> Mat {
        if k == 0 {
            Mat::zeros(a.n())
        } else {
            a.mul(&term(a, t, k - 1))
        }
    }

    fn staged(rule: &Mat, s: f64, t: f64, starts: &[[f64; 2]]) -> Vec<[f64; 2]> {
        placed(rule, t, &placed(rule, s, starts))
    }

    /// 09_1: the arrow that comes out of the machine starts at the centre,
    /// as long as the rule says, and ends on the tip of its input at the
    /// field's own arrow length: the same arrow the field draws there.
    #[test]
    fn the_machines_output_slides_into_the_fields_arrow() {
        let j = quarter_turn();
        let starts = field_starts();
        let len = |a: [f64; 2], b: [f64; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
        for &(c, r) in &SAMPLES {
            let p = starts[sample_index(c, r)];
            let (tail0, head0) = sliding(&j, p, GAIN, 0.0);
            assert_eq!(tail0, [0.0, 0.0]);
            assert_eq!(head0, [-p[1], p[0]], "what comes out of J for p is (−y, x)");
            let (tail1, head1) = sliding(&j, p, GAIN, 1.0);
            let (from, to) = velocity_ends(&j, p, GAIN);
            assert!(
                len(tail1, from) < 1e-12 && len(head1, to) < 1e-12,
                "it ends as the field's arrow"
            );
            let (t5, h5) = sliding(&j, p, GAIN, 0.5);
            assert!(len(t5, h5) < len(tail0, head0) && len(t5, h5) > len(tail1, head1));
        }
        let mut idx: Vec<usize> = SAMPLES.iter().map(|&(c, r)| sample_index(c, r)).collect();
        idx.sort_unstable();
        idx.dedup();
        assert_eq!(idx.len(), 12, "twelve distinct positions");
        assert!(idx.iter().all(|&i| i < starts.len()));
    }

    /// 09_5 and 09_6, an always sentence: one rule run for s and then for t
    /// more is one longer run, for every start, for rules that turn,
    /// spiral, flow and shear.
    #[test]
    fn a_rule_run_twice_is_one_longer_run() {
        let starts = field_starts();
        let rules = [
            quarter_turn(),
            spiral_rule(),
            crate::model::tanks(shown::TANK_LITRES, shown::TANK_PUMPED),
            crate::facts::shear_right(),
            Mat::from_rows(&[&[1.0, 2.0], &[0.5, -0.7]]),
        ];
        for rule in &rules {
            for (s, t) in [(STAGE_S, STAGE_T), (0.0, 1.0), (2.5, 0.01)] {
                let two = staged(rule, s, t, &starts);
                let one = placed(rule, s + t, &starts);
                for (a, b) in two.iter().zip(&one) {
                    let scale = 1.0 + b[0].abs().max(b[1].abs());
                    assert!((a[0] - b[0]).abs() < 1e-9 * scale && (a[1] - b[1]).abs() < 1e-9 * scale);
                }
            }
        }
    }

    /// 09_4: the rate of each term is A times the term before it. The rate
    /// is taken by a centred difference of the term itself, so the claim is
    /// checked, not copied.
    #[test]
    fn the_rate_of_each_term_is_a_times_the_one_before() {
        let a = Mat::from_rows(&[&[0.4, -1.1, 0.3], &[0.9, 0.2, -0.5], &[-0.6, 0.7, 0.1]]);
        let h = 1e-5;
        for t in [0.3, 1.0, 2.2] {
            for k in 0..=6u32 {
                let numeric = term(&a, t + h, k).sub(&term(&a, t - h, k)).scale(1.0 / (2.0 * h));
                assert!(
                    numeric.sub(&written_rate(&a, t, k)).max_abs() < 1e-6,
                    "k = {k}, t = {t}"
                );
            }
        }
    }

    /// 09_4: through any number of terms, the rate of the row is A times the
    /// row one term shorter (the row slides one place and leaves A in
    /// front), and the whole row is the exponential.
    #[test]
    fn the_rate_of_the_row_is_a_times_the_row() {
        let a = spiral_rule();
        let t = 1.3;
        for kmax in 1..=8u32 {
            let rates = (0..=kmax).fold(Mat::zeros(2), |m, k| m.add(&written_rate(&a, t, k)));
            let row = (0..kmax).fold(Mat::zeros(2), |m, k| m.add(&term(&a, t, k)));
            assert!(rates.sub(&a.mul(&row)).max_abs() < 1e-12, "kmax = {kmax}");
        }
        let total = series(&a.scale(t), 40).pop().unwrap().total;
        assert!(total.sub(&expm_t(&a, t)).max_abs() < 1e-12);
    }

    #[test]
    fn terms_are_written_as_the_series_writes_them() {
        assert_eq!(term_tex(0), "I");
        assert_eq!(term_tex(1), "tA");
        assert_eq!(term_tex(2), r"\frac{t^2A^2}{2!}");
        assert_eq!(term_tex(3), r"\frac{t^3A^3}{3!}");
    }

    /// 09_2: the step counts on screen are the facts' (4 and 16), the
    /// traveller's straight steps end where the facts say, and the smooth
    /// path ends at half a circle. 09_3's spiral is the facts' spiral.
    #[test]
    fn the_traveller_ends_where_the_facts_say() {
        let j = quarter_turn();
        for (n, key) in [(shown::TURN_STEPS[0], "n4"), (shown::TURN_STEPS[1], "n16")] {
            let end = *tiny_path(&j, TRAVELLER_TIME, TRAVELLER, n as usize).last().unwrap();
            let want = numbers(&format!("F-N.turn.steps.{key}")).unwrap();
            assert!(
                (end[0] - want[0]).abs() < 1e-4 && (end[1] - want[1]).abs() < 1e-4,
                "{key}"
            );
        }
        let smooth = *path(&j, TRAVELLER_TIME, TRAVELLER, 72).last().unwrap();
        assert!(
            (smooth[0] + 1.0).abs() < 1e-9 && smooth[1].abs() < 1e-9,
            "half a circle ends at −1"
        );
        assert_eq!(
            super::super::draw::fact("F-M.spiral.rule").unwrap(),
            crate::facts::mat(&spiral_rule(), 1).unwrap()
        );
        assert_eq!(cited::PEANO_YEAR, 1888);
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("09_follow_the_arrows").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "4 steps",
            "16 steps",
            r"\left(I+\frac{t}{n}A\right)^{n}",
            r"\frac{t^3A^3}{3!}",
            r"\frac{d}{dt}\,e^{tA}=A\,e^{tA}",
            "e^{tA}\\,e^{sA}=e^{(s+t)A}",
            "Peano, 1888: solving x′ = Ax this way",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
