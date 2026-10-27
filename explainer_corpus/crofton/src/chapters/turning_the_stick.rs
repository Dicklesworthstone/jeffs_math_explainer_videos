//! Chapter 09, turning the stick. From 08's end (the stick at a 4.5 cm
//! shadow, the step plot of its slide, the rule and its four uses) the
//! slide is settled one direction at a time, but the shadow still depends
//! on the direction (09_1): the plot and uses go, the count and its
//! crossings too, and the stick turns to lie along the lines. Turning the
//! stick is turning the sheet the other way, and half a turn tries every
//! direction once (09_2): in silence it turns half a turn while a plot at
//! the right draws its shadow against its direction, an arch from none to
//! the full 6 cm and back, through 08's two shadows (09_3). The average
//! over every slide and every direction is the arch's average height over
//! the gap: more than half the stick, or less? (09_4, a prediction.) More:
//! the arch, flattened into a rectangle of the same area, is 3.82 cm high,
//! about 64% of the stick (09_5); over the gap, 1.91 crossings (09_6);
//! times π cm, 6.0 cm, the stick's own length, where one placement said
//! 9.4 or 0 (09_7). The 64% is 2/π, which a coin will explain (09_8). The
//! stick, its shadow, the arch and its column are drawn every frame from
//! the stick's direction through `straight_sticks`'s model; the flattening
//! from the same arch. Starts where chapter 08 ends, on 08's
//! [`Bench`]. Not registered yet: root wires it in
//! (`docs/candidate-08-09.md`).

use std::f64::consts::PI;

use explainer_kit::{Kit, Palette, tween, v};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::draw::{card_point, fade_in_all, fade_out_all, fade_to, fail, flat, line_style, now, say, stand, think};
use super::placement::{Pose, at_once, faded_line, glide, tracked};
use super::sheet::{self, LABEL_SIZE};
use super::straight_sticks::{ALONG, HOME, LENGTH};
use super::the_shadow_rule::{
    self, Bench, RULE, RULE_AT, TILTED, USES_AT, direction, hidden_back, left_at, live_trace, mean_mark, plot_axes,
    reach, row, shadow_label, tilted_to, turn_to,
};
use crate::facts;
use crate::model::{self, COIN, GAP};
use crate::palette;

pub struct TurningTheStick {
    pub kit: Kit,
}

// ---------------------------------------------------------------- the model

/// The directions the half turn's average shadow is read at: the middle of
/// each of this many equal steps of half a turn, as `Curve::mean_travel`
/// reads a curve's travel (`stick.mean_reach`).
pub const TURN_STEPS: usize = 3600;

/// The stick's average shadow over half a turn, cm: the arch's average
/// height.
pub fn mean_shadow() -> f64 {
    let n = TURN_STEPS as f64;
    (0..TURN_STEPS).map(|i| reach(PI * (i as f64 + 0.5) / n)).sum::<f64>() / n
}

/// One crossing's worth on the bet, cm, as 06 has it: π × [`COIN`] / 2.
fn rate() -> f64 {
    PI * COIN / 2.0
}

/// 09_8's two over pi, the share the coin will explain.
const TWO_OVER_PI: f64 = 2.0 / PI;

/// The chapter's words that carry numbers, each from the facts, and the
/// average the flattening settles at. Refused unless the model gives what
/// the voice says: the average shadow more than half the stick, the arch
/// above half for most of the turn and at zero only at its ends, the bet
/// on the average the stick's own length, and every number on screen what
/// its shown operands give.
pub struct Words {
    /// The plot's directions, `0^\circ`, `90^\circ`, `180^\circ`, and its
    /// heights, "0", "3", "6".
    pub ticks: [String; 3],
    pub heights: [String; 3],
    /// "3.82 cm", "about 64%".
    pub mean: String,
    pub percent: String,
    /// The rows: one placement; over every direction; the bet on that.
    pub one: String,
    pub over_directions: String,
    pub estimate: String,
    /// 09_7's "6 cm" by the stick, 09_8's share.
    pub length: String,
    pub ratio: String,
    pub mean_cm: f64,
}

pub fn words() -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let fixed = facts::fixed;
    let agree = |key: &str, got: String| -> Result<String, String> {
        let want = get(key)?;
        if want == got {
            Ok(want)
        } else {
            Err(format!("{key}: the model gives {got}, the facts {want}"))
        }
    };

    // the arch: more than half on average, above half for most of the
    // turn, zero only at its two ends
    let mean = mean_shadow();
    let n = TURN_STEPS as f64;
    let shadows: Vec<f64> = (0..TURN_STEPS).map(|i| reach(PI * (i as f64 + 0.5) / n)).collect();
    let high = shadows.iter().filter(|&&s| s > LENGTH / 2.0).count() as f64 / n;
    if mean <= LENGTH / 2.0 || high <= 0.5 || shadows.iter().any(|&s| s <= 0.0) {
        return Err(format!(
            "09_5 says more than half and high for most of the turn: the model averages {mean:.4} and is high {high:.3} of it"
        ));
    }
    if reach(0.0) > 1e-12 || reach(ALONG) > 1e-12 {
        return Err("the arch does not end at zero".into());
    }

    let gap = agree("sheet.gap", fixed(GAP, 0))?;
    let length = agree("stick.length", fixed(LENGTH, 0))?;
    let average = agree("stick.mean_reach", fixed(mean, 2))?;
    let share = agree("stick.mean_reach_share", fixed(mean / LENGTH, 2))?;
    let percent = agree("stick.mean_reach_percent", fixed(mean / LENGTH * 100.0, 0))?;
    let turned = agree("stick.turned_mean", fixed(mean / GAP, 2))?;
    let estimate = agree("stick.turned_estimate", fixed(mean / GAP * rate(), 1))?;
    agree("coin.worth", fixed(rate(), 2))?;
    let (across, along) = (get("stick.across_estimate")?, get("stick.along_estimate")?);
    // the two shadows 08 measured, on the arch
    agree("stick.reach30", fixed(reach(TILTED), 0))?;
    agree(
        "stick.test_shadow",
        fixed(reach(tilted_to(the_shadow_rule::TEST_SHADOW)), 1),
    )?;

    // what the screen shows, from its shown operands: 3.82 ÷ 2 = 1.91,
    // 1.91 × π ≈ 6.0 (the stick's own length), 3.82 ÷ 6 ≈ 0.64 ≈ 2/π, and
    // 64% the same share
    let number = |s: &str| s.parse::<f64>().map_err(|e| format!("{s}: {e}"));
    let (m, g, l, t) = (number(&average)?, number(&gap)?, number(&length)?, number(&turned)?);
    for (shown, from_shown) in [
        (&turned, fixed(m / g, 2)),
        (&estimate, fixed(t * PI, 1)),
        (&estimate, fixed(l, 1)),
        (&share, fixed(m / l, 2)),
        (&share, fixed(TWO_OVER_PI, 2)),
        (&percent, fixed(number(&share)? * 100.0, 0)),
    ] {
        if *shown != from_shown {
            return Err(format!(
                "the screen shows {shown}, its shown operands give {from_shown}"
            ));
        }
    }

    let tick = |d: f64| format!(r"{}^\circ", fixed(d, 0));
    Ok(Words {
        ticks: [tick(0.0), tick(90.0), tick(180.0)],
        heights: [0.0, LENGTH / 2.0, LENGTH].map(|h| fixed(h, 0)),
        mean: format!("{average} cm"),
        percent: format!("about {percent}%"),
        one: format!(r"\text{{one placement}}\quad {across}\text{{ cm or }} {along}\text{{ cm}}"),
        over_directions: format!(r"\text{{and over every direction}} = {average} \div {gap} = {turned}"),
        estimate: format!(r"{turned} \times \pi\text{{ cm}} \approx {estimate}\text{{ cm}}"),
        length: format!("{length} cm"),
        ratio: format!(r"\tfrac{{{average}}}{{{length}}} \approx {share} \approx \tfrac{{2}}{{\pi}}"),
        mean_cm: mean,
    })
}

// ------------------------------------------------------------------ layout

/// The arch's plot at the card's upper right, frame: the direction across,
/// 0° to 180°; the shadow up, 0.36 a centimetre; its words, the title
/// left-aligned; the words at its right, left-aligned, the percentage one
/// row over the average's. The panel it stands on is two opaque
/// rectangles, the plot's and a strip at its right for those words, which
/// runs on over the ends of the lines it hides so no stub of them shows;
/// both clear of the coast, the strip over the hook.
const PLOT_X: [f64; 2] = [1.45, 4.85];
const PLOT_Y0: f64 = 0.9;
const PLOT_K: f64 = 0.36;
const PLOT_SIZE: f64 = 18.0;
const TICK_Y: f64 = 0.63;
const HEIGHT_X: f64 = 1.2;
const TITLE_AT: Vec3 = [1.1, 3.4, 0.0];
const TITLE: &str = "shadow in cm";
const RIGHT_X: f64 = 4.95;
const RIGHT_SIZE: f64 = 20.0;
const HALF: &str = "half";
const PERCENT_RISE: f64 = 0.32;
const BACKS: [(Vec3, f64, f64); 2] = [([3.1, 2.025, 0.0], 4.2, 3.19), ([5.64, 2.61, 0.0], 1.28, 2.02)];
const AXIS_W: f64 = 2.0;
const ARCH_W: f64 = 4.0;
const COLUMN_W: f64 = 5.0;
const HALF_W: f64 = 2.0;
const MEAN_W: f64 = 2.5;
/// Points along the drawn arch, and the flattening's fill.
const ARCH_SAMPLES: usize = 360;
const FILL_OPACITY: f64 = 0.3;
const MEASURED_R: f64 = 0.07;
/// The countdown, where the hidden count stood (the plot holds 08's place
/// for it); 09_7's "6 cm" over the stick lying along, cm; 09_8's share,
/// the coin and its question, frame.
const THINK_HERE: Vec3 = [-2.6, 2.9, 0.0];
const LENGTH_AT: [f64; 2] = [HOME[0], HOME[1] + 0.6];
/// The one placement row, left-aligned at the console's left, midway
/// between the rows of the rule's column so it reads as its own.
const ONE_AT: Vec3 = [-5.9, -2.65, 0.0];
const RATIO_AT: Vec3 = [-3.4, 3.0, 0.0];
const RATIO_SIZE: f64 = 28.0;
const COIN_AT: Vec3 = [-1.25, 3.0, 0.0];
const COIN_R: f64 = 0.25;
const COIN_W: f64 = 4.0;
const ASK_AT: Vec3 = [-0.65, 3.0, 0.0];
const ASK_SIZE: f64 = 34.0;

pub fn arch_x(theta: f64) -> f64 {
    PLOT_X[0] + (PLOT_X[1] - PLOT_X[0]) * theta / PI
}

pub fn arch_y(cm: f64) -> f64 {
    PLOT_Y0 + PLOT_K * cm
}

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
/// 09_1's turn to lie along, 48.6°, and the half turn, at a steady pace.
const ALONG_S: f64 = 2.5;
const HALF_TURN_S: f64 = 12.0;
const FLATTEN_S: f64 = 2.5;

// --------------------------------------------------------- what is drawn

/// The arch from direction 0 to the direction `upto(s)`, at the opacity
/// `alpha` holds. In the scene.
fn live_arch(
    stage: &mut Stage<'_>,
    upto: impl Fn(&fmn::mobject::Stage) -> f64 + 'static,
    alpha: Mob,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let to = upto(s).clamp(0.0, PI);
        let steps = ((to / PI * ARCH_SAMPLES as f64).ceil() as usize).max(1);
        let arch: Vec<VMobject> = (to > 1e-6)
            .then(|| {
                let pts: Vec<Vec3> = (0..=steps)
                    .map(|j| {
                        let theta = to * j as f64 / steps as f64;
                        v(arch_x(theta), arch_y(reach(theta)))
                    })
                    .collect();
                polyline(pts)
                    .style(faded_line(palette::stick(), ARCH_W, tracked(s, alpha)))
                    .build()
            })
            .into_iter()
            .collect();
        s.add(v_group(arch))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The shadow at the stick's direction, upright on the plot from the axis
/// to the arch, at the opacity `alpha` holds. In the scene.
fn live_column(stage: &mut Stage<'_>, turn: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let theta = direction(s, turn).clamp(0.0, PI);
        let column = polyline(vec![v(arch_x(theta), PLOT_Y0), v(arch_x(theta), arch_y(reach(theta)))])
            .style(faded_line(palette::stick(), COLUMN_W, tracked(s, alpha)))
            .build();
        s.add(v_group(vec![column]))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The region under the arch, its top moved the share `morph` holds of
/// the way to the flat height `mean` (each point straight up or down, so
/// its area never changes), filled at the opacity `alpha` holds. In the
/// scene.
fn live_fill(stage: &mut Stage<'_>, mean: f64, morph: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let pts = fill_points(mean, tracked(s, morph));
        let fill = Polygon::new(pts)
            .style(flat(palette::stick(), FILL_OPACITY * tracked(s, alpha)))
            .build();
        s.add(v_group(vec![fill]))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The fill's outline at the share `t` of the flattening, frame points:
/// along the axis, then back along its top.
pub fn fill_points(mean: f64, t: f64) -> Vec<Vec3> {
    let mut pts = vec![v(PLOT_X[0], PLOT_Y0), v(PLOT_X[1], PLOT_Y0)];
    for j in (0..=ARCH_SAMPLES).rev() {
        let theta = PI * j as f64 / ARCH_SAMPLES as f64;
        let h = reach(theta);
        pts.push(v(arch_x(theta), arch_y(h + (mean - h) * t)));
    }
    pts
}

/// Frame points as one open polyline. Detached.
fn frame_path(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points).style(line_style(color, width)).build())
}

/// The arch's plot: its axes and words, `ticks` across and `heights` up.
/// Detached.
fn arch_axes(kit: &Kit, stage: &mut Stage<'_>, words: &Words) -> fmn::Result<Vec<Mob>> {
    let top = arch_y(LENGTH) + 0.12;
    let mut out = vec![
        frame_path(
            stage,
            vec![v(PLOT_X[0], PLOT_Y0), v(PLOT_X[1], PLOT_Y0)],
            Palette::dim(),
            AXIS_W,
        ),
        frame_path(
            stage,
            vec![v(PLOT_X[0], PLOT_Y0), v(PLOT_X[0], top)],
            Palette::dim(),
            AXIS_W,
        ),
    ];
    for (tex, theta) in words.ticks.iter().zip([0.0, PI / 2.0, PI]) {
        out.push(kit.tex(stage, tex, PLOT_SIZE, Palette::dim(), &[], v(arch_x(theta), TICK_Y))?);
    }
    for (text, cm) in words.heights.iter().zip([0.0, LENGTH / 2.0, LENGTH]) {
        out.push(kit.text(stage, text, PLOT_SIZE, Palette::dim(), v(HEIGHT_X, arch_y(cm)))?);
    }
    let title = kit.text(stage, TITLE, PLOT_SIZE, palette::stick(), TITLE_AT)?;
    out.push(left_at(stage, title, TITLE_AT));
    Ok(out)
}

/// A word at the plot's right, left-aligned at the frame height `y`.
/// Detached.
fn right_word(kit: &Kit, stage: &mut Stage<'_>, words: &str, y: f64) -> fmn::Result<Mob> {
    let at = v(RIGHT_X, y);
    let m = kit.text(stage, words, RIGHT_SIZE, Palette::ink(), at)?;
    Ok(left_at(stage, m, at))
}

impl SceneConstruct for TurningTheStick {
    fn name(&self) -> &str {
        "09_turning_the_stick"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let earlier = the_shadow_rule::words(first.slide).map_err(fail)?;
        let words = words().map_err(fail)?;
        let tested = tilted_to(the_shadow_rule::TEST_SHADOW);

        // where 08 ended: the bench with the stick at the 4.5 cm shadow,
        // its backings and footprints up; the step plot with the test's
        // trace and average; the label, the rule and its uses. The arch's
        // panel, unseen, comes next, behind every word made later
        let bench = Bench::new(&kit, stage, &coast, first, tested)?;
        for &m in [bench.backs.count, bench.backs.label, bench.backs.console]
            .iter()
            .chain(&bench.footprints)
        {
            stage.set_family_opacity(m, 1.0);
        }
        let panels = [hidden_back(stage, BACKS[0])?, hidden_back(stage, BACKS[1])?];
        let mut slide_plot = plot_axes(&kit, stage, &earlier.gap)?;
        slide_plot.extend(mean_mark(&kit, stage, earlier.means[1], &earlier.averages[1])?);
        let trace_alpha = stage.add_value_tracker(1.0);
        live_trace(stage, earlier.runs[1].clone(), |_| 1.0, trace_alpha)?;
        let label = shadow_label(&kit, stage, &earlier.shadows[1])?;
        let rule = row(&kit, stage, RULE, Palette::ink(), RULE_AT)?;
        let mut uses = Vec::with_capacity(USES_AT.len());
        for (tex, at) in earlier.uses.iter().zip(USES_AT) {
            uses.push(row(&kit, stage, tex, Palette::ink(), at)?);
        }
        for &m in slide_plot.iter().chain(&uses).chain([&label, &rule]) {
            stage.add_to_scene(m)?;
        }

        // 09_1: the slide is settled; the step plot, its uses and the
        // footprints go; then the count, its crossings, the label and their
        // backings, and the stick turns to lie along the lines
        let line = say(&kit, stage, "09_1")?;
        line.wait_for(stage, "That settles the slide")?;
        let mut settled = slide_plot.clone();
        settled.extend(&uses);
        settled.extend(&bench.footprints);
        at_once(stage, &[], &settled, vec![tween(trace_alpha, 0.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "which way the stick points")?;
        at_once(
            stage,
            &[],
            &[label],
            vec![
                tween(bench.count_alpha, 0.0, FADE_S)?,
                tween(bench.dots_alpha, 0.0, FADE_S)?,
                fade_to(bench.backs.count, 0.0, FADE_S)?,
                fade_to(bench.backs.label, 0.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        turn_to(stage, bench.turn, ALONG, ALONG_S, smooth)?;

        // 09_2: the arch's empty plot; then, in silence, half a turn from
        // along (the same stick named at 0°), the arch drawn behind the
        // column as the stick turns
        let line = say(&kit, stage, "09_2")?;
        line.wait_for(stage, "Watch the shadow")?;
        let axes = arch_axes(&kit, stage, &words)?;
        let [drawing, column_alpha, arch_alpha] = [0.0, 0.0, 1.0].map(|x| stage.add_value_tracker(x));
        let held = stage.add_value_tracker(0.0);
        // 09_5's fill, unseen until then, made before the arch so the arch,
        // its column, 08's two points and the half line are drawn over it
        let [morph, fill_alpha] = [0.0, 0.0].map(|x| stage.add_value_tracker(x));
        live_fill(stage, words.mean_cm, morph, fill_alpha)?;
        let turn = bench.turn;
        live_arch(
            stage,
            move |s| {
                if tracked(s, drawing) > 0.5 {
                    direction(s, turn)
                } else {
                    s.tracker_value(held).unwrap_or(0.0)
                }
            },
            arch_alpha,
        )?;
        live_column(stage, turn, column_alpha)?;
        at_once(
            stage,
            &axes,
            &[],
            vec![fade_to(panels[0], 1.0, FADE_S)?, fade_to(panels[1], 1.0, FADE_S)?],
            FADE_S,
        )?;
        kit.hold(stage)?;
        stage.set_tracker_value(turn, 0.0)?;
        stage.set_tracker_value(drawing, 1.0)?;
        stage.play(tween(column_alpha, 1.0, 0.3)?)?;
        stage.play(glide(turn, PI, HALF_TURN_S, linear)?)?;
        stage.set_tracker_value(turn, PI)?;
        stage.set_tracker_value(held, PI)?;
        stage.set_tracker_value(drawing, 0.0)?;
        stage.play(tween(column_alpha, 0.0, 0.3)?)?;

        // 09_3: none along, the full shadow across, the arch; 08's two
        // measured shadows on it
        let line = say(&kit, stage, "09_3")?;
        line.wait_for(stage, "like the two we measured")?;
        let measured: Vec<Mob> = [TILTED, tested]
            .iter()
            .map(|&theta| {
                let dot = Circle::new()
                    .radius(MEASURED_R)
                    .arc_center(v(arch_x(theta), arch_y(reach(theta))))
                    .style(flat(palette::touch(), 1.0))
                    .build();
                stage.arena_mut().add(dot)
            })
            .collect();
        fade_in_all(stage, &measured, FADE_S)?;

        // 09_4: half the stick, dashed; more or less?
        let line = say(&kit, stage, "09_4")?;
        line.wait_for(stage, "more than half the stick")?;
        let half_line = DashedLine::new(v(PLOT_X[0], arch_y(LENGTH / 2.0)), v(PLOT_X[1], arch_y(LENGTH / 2.0)))
            .dash_length(0.1)
            .style(line_style(Palette::dim(), HALF_W))
            .build()?;
        let half_line = stage.arena_mut().add(half_line);
        let half = right_word(&kit, stage, HALF, arch_y(LENGTH / 2.0))?;
        fade_in_all(stage, &[half_line, half], FADE_S)?;
        let q = think(&kit, stage, THINK_HERE)?;

        // 09_5: more; the arch flattened into a rectangle of the same area,
        // 3.82 cm high, about 64% of the stick
        let line = say(&kit, stage, "09_5")?;
        line.wait_for(stage, "More")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "Join the two ends of the arch")?;
        let tent = frame_path(
            stage,
            vec![
                v(arch_x(0.0), arch_y(0.0)),
                v(arch_x(PI / 2.0), arch_y(LENGTH)),
                v(arch_x(PI), arch_y(0.0)),
            ],
            Palette::ink(),
            MEAN_W,
        );
        fade_in_all(stage, &[tent], 0.6)?;
        line.wait_for(stage, "So its average height is half the stick")?;
        let halfway = frame_path(
            stage,
            vec![v(PLOT_X[0], arch_y(LENGTH / 2.0)), v(PLOT_X[1], arch_y(LENGTH / 2.0))],
            Palette::ink(),
            MEAN_W,
        );
        fade_in_all(stage, &[halfway], 0.4)?;
        fade_out_all(stage, &[halfway], 0.4)?;
        line.wait_for(stage, "Flatten it")?;
        at_once(stage, &[], &[half, tent], vec![tween(fill_alpha, 1.0, FADE_S)?], FADE_S)?;
        stage.play(glide(morph, 1.0, FLATTEN_S, smooth)?)?;
        stage.set_tracker_value(morph, 1.0)?;
        line.wait_for(stage, "three point eight two centimetres")?;
        let top = frame_path(
            stage,
            vec![v(PLOT_X[0], arch_y(words.mean_cm)), v(PLOT_X[1], arch_y(words.mean_cm))],
            Palette::ink(),
            MEAN_W,
        );
        let mean = right_word(&kit, stage, &words.mean, arch_y(words.mean_cm))?;
        fade_in_all(stage, &[top, mean], FADE_S)?;
        line.wait_for(stage, "about sixty-four percent")?;
        let percent = right_word(&kit, stage, &words.percent, arch_y(words.mean_cm) + PERCENT_RISE)?;
        fade_in_all(stage, &[percent], FADE_S)?;

        // 09_6: over the gap, 1.91 crossings, over every slide and every
        // direction, under the rule
        let line = say(&kit, stage, "09_6")?;
        line.wait_for(stage, "one point nine one crossings")?;
        let over = row(&kit, stage, &words.over_directions, Palette::ink(), USES_AT[0])?;
        fade_in_all(stage, &[over], FADE_S)?;

        // 09_7: the bet on the average, 6.0 cm, the stick's own; one
        // placement said 9.4 or 0
        let line = say(&kit, stage, "09_7")?;
        line.wait_for(stage, "six point zero centimetres")?;
        let estimate = row(&kit, stage, &words.estimate, palette::length(), USES_AT[1])?;
        fade_in_all(stage, &[estimate], FADE_S)?;
        line.wait_for(stage, "the stick's own length")?;
        let length = kit.text(
            stage,
            &words.length,
            LABEL_SIZE,
            palette::length(),
            card_point(LENGTH_AT),
        )?;
        fade_in_all(stage, &[length], FADE_S)?;
        line.wait_for(stage, "A single placement said")?;
        let one = row(&kit, stage, &words.one, Palette::ink(), ONE_AT)?;
        fade_in_all(stage, &[one], FADE_S)?;

        // 09_8: the share is two over pi; a coin will explain it
        let line = say(&kit, stage, "09_8")?;
        line.wait_for(stage, "two over pi")?;
        let ratio = kit.tex(stage, &words.ratio, RATIO_SIZE, Palette::ink(), &[], RATIO_AT)?;
        fade_in_all(stage, &[ratio], FADE_S)?;
        line.wait_for(stage, "a coin will explain it")?;
        let coin = Circle::new()
            .radius(COIN_R)
            .arc_center(COIN_AT)
            .style(line_style(palette::coin(), COIN_W))
            .build();
        let coin = stage.arena_mut().add(coin);
        let ask = kit.text(stage, "?", ASK_SIZE, palette::touch(), ASK_AT)?;
        fade_in_all(stage, &[coin, ask], FADE_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::THINK_R;
    use super::super::straight_sticks::{ACROSS, ends, shadow};
    use super::*;
    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W, Curve, P2, Part, Seg};

    fn first_slide() -> f64 {
        sheet::shown_slide(&model::coast(), 0.0)
    }

    /// The stick lying across as a one-piece curve, for `Curve::travel`.
    fn across_stick() -> Curve {
        let [from, to] = ends(HOME, ACROSS);
        Curve {
            parts: vec![Part {
                seg: Seg::Line { from, to },
                start: 0.0,
            }],
            length: LENGTH,
        }
    }

    fn half_height(size: f64) -> f64 {
        size / 150.0
    }

    fn width(text: &str, size: f64) -> f64 {
        text.chars().count() as f64 * size / 150.0
    }

    fn bounds((at, w, h): (Vec3, f64, f64)) -> [f64; 4] {
        [at[0] - w / 2.0, at[0] + w / 2.0, at[1] - h / 2.0, at[1] + h / 2.0]
    }

    fn inside(p: Vec3, b: [f64; 4], pad: f64) -> bool {
        p[0] > b[0] + pad && p[0] < b[1] - pad && p[1] > b[2] + pad && p[1] < b[3] - pad
    }

    /// Turning the stick by φ is turning the sheet by −φ: at every degree
    /// the stick's shadow turned from across equals the travel of the
    /// stick lying across over lines turned the other way, and the half
    /// turn's average equals `Curve::mean_travel` over the same midpoints.
    /// That average is 2/π of the stick, within the reading's step.
    #[test]
    fn turning_the_stick_is_turning_the_sheet() {
        let stick = across_stick();
        for d in -90..=90 {
            let phi = f64::from(d).to_radians();
            assert!((reach(ACROSS + phi) - stick.travel(-f64::from(d))).abs() < 1e-12, "{d}");
        }
        assert!((mean_shadow() - stick.mean_travel(TURN_STEPS)).abs() < 1e-9);
        assert!((mean_shadow() / LENGTH - TWO_OVER_PI).abs() < 1e-6);
        assert_eq!(facts::fixed(mean_shadow(), 2), "3.82");
        assert_eq!(facts::fixed(mean_shadow() / GAP, 2), "1.91");
    }

    /// "More than half": the average is 3.82 of 6 cm; the arch is above
    /// half for two thirds of the turn (30° to 150°), at zero only at its
    /// ends, and through 08's two measured shadows, 3 cm at 150° and
    /// 4.5 cm at 131.41°.
    #[test]
    fn the_arch_stays_high_for_most_of_the_turn() {
        let n = TURN_STEPS as f64;
        let shadows: Vec<f64> = (0..TURN_STEPS).map(|i| reach(PI * (i as f64 + 0.5) / n)).collect();
        let high = shadows.iter().filter(|&&s| s > LENGTH / 2.0).count() as f64 / n;
        assert!((high - 2.0 / 3.0).abs() < 1e-3, "{high}");
        assert!(shadows.iter().all(|&s| s > 0.0));
        assert!(reach(0.0) < 1e-12 && reach(PI) < 1e-12);
        assert!(mean_shadow() > LENGTH / 2.0);
        assert!((reach(TILTED) - 3.0).abs() < 1e-12);
        let tested = tilted_to(the_shadow_rule::TEST_SHADOW);
        assert!((reach(tested) - 4.5).abs() < 1e-9);
        assert_eq!(facts::fixed(tested.to_degrees(), 2), "131.41");
        // the shadow is where the ends are, at every degree of the half turn
        for d in 0..=180 {
            let theta = f64::from(d).to_radians();
            let [lo, hi] = shadow(HOME, theta);
            assert!((reach(theta) - (hi - lo)).abs() < 1e-15);
            assert!((reach(theta) - LENGTH * theta.sin().abs()).abs() < 1e-12);
        }
    }

    /// The flattening keeps the area: the fill's outline has the same
    /// shoelace area at every share of the way, within the drawn arch's own
    /// step, and ends as a rectangle the average high.
    #[test]
    fn the_flattening_keeps_the_area() {
        let mean = mean_shadow();
        let area = |pts: &[Vec3]| -> f64 {
            let n = pts.len();
            (0..n)
                .map(|i| {
                    let (a, b) = (pts[i], pts[(i + 1) % n]);
                    a[0] * b[1] - b[0] * a[1]
                })
                .sum::<f64>()
                .abs()
                / 2.0
        };
        let rectangle = (PLOT_X[1] - PLOT_X[0]) * (arch_y(mean) - PLOT_Y0);
        for k in 0..=10 {
            let a = area(&fill_points(mean, f64::from(k) / 10.0));
            assert!((a - rectangle).abs() / rectangle < 1e-4, "{k}: {a} {rectangle}");
        }
        let flat = fill_points(mean, 1.0);
        assert!(flat[2..].iter().all(|p| (p[1] - arch_y(mean)).abs() < 1e-12));
    }

    /// The words on screen say what the facts and the model say.
    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words().unwrap();
        assert_eq!(w.ticks, [r"0^\circ", r"90^\circ", r"180^\circ"]);
        assert_eq!(w.heights, ["0", "3", "6"]);
        assert_eq!(w.mean, "3.82 cm");
        assert_eq!(w.percent, "about 64%");
        assert_eq!(w.one, r"\text{one placement}\quad 9.4\text{ cm or } 0\text{ cm}");
        assert_eq!(
            w.over_directions,
            r"\text{and over every direction} = 3.82 \div 2 = 1.91"
        );
        assert_eq!(w.estimate, r"1.91 \times \pi\text{ cm} \approx 6.0\text{ cm}");
        assert_eq!(w.length, "6 cm");
        assert_eq!(w.ratio, r"\tfrac{3.82}{6} \approx 0.64 \approx \tfrac{2}{\pi}");
        // never 0.64 × 6, which gives 3.84, not the 3.82 on screen
        assert_eq!(facts::fixed(0.64 * LENGTH, 2), "3.84");
    }

    /// The plot sits on its panel, clear of the coast, the stick at every
    /// direction and the card's edge; no line at rest grazes a panel's
    /// edge, and the strip covers the end of each line it hides; the
    /// plot's words on the panel; the countdown, the share, the coin and
    /// "6 cm" clear of the stick lying along and of the lines at rest; the
    /// rows on 08's console.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let coast = model::coast();
        let s = first_slide();
        let panels = BACKS.map(bounds);
        let on_panel = |p: Vec3, pad: f64| panels.iter().any(|&b| inside(p, b, pad));
        // the panel keeps off the coast, and inside the card
        let n = (coast.length / 0.02).ceil() as usize;
        for i in 0..=n {
            let c = card_point(coast.at(coast.length * i as f64 / n as f64));
            assert!(!on_panel(c, -0.1), "the panel covers the coast at {c:?}");
        }
        let card = [
            CARD_CENTRE[0] - CARD_UNIT * CARD_W / 2.0,
            CARD_CENTRE[0] + CARD_UNIT * CARD_W / 2.0,
            CARD_CENTRE[1] - CARD_UNIT * CARD_H / 2.0,
            CARD_CENTRE[1] + CARD_UNIT * CARD_H / 2.0,
        ];
        for b in panels {
            assert!(b[0] > card[0] + 0.1 && b[1] < card[1] - 0.1 && b[2] > card[2] && b[3] < card[3] - 0.1);
        }
        let lines: Vec<f64> = sheet::lines_on_card(s)
            .iter()
            .map(|&h| CARD_CENTRE[1] + CARD_UNIT * h)
            .collect();
        let line_end = CARD_CENTRE[0] + CARD_UNIT * (CARD_W / 2.0 - sheet::LINE_INSET);
        for &y in &lines {
            for b in panels {
                assert!((y - b[2]).abs() > 0.15 && (y - b[3]).abs() > 0.15, "{y} grazes {b:?}");
            }
            if (panels[1][2]..=panels[1][3]).contains(&y) {
                assert!(panels[1][1] > line_end + 0.05, "a stub of the line at {y}");
            }
        }
        // the stick, at every tenth of a degree of the half turn, keeps
        // well left of the panel
        for i in 0..=1800 {
            for e in ends(HOME, PI * f64::from(i) / 1800.0) {
                assert!(card_point(e)[0] < panels[0][0] - 1.5, "{e:?}");
            }
        }
        // the plot and its words on the panel
        let pad = 0.05;
        for theta in [0.0, PI / 2.0, PI] {
            let at = v(arch_x(theta), TICK_Y);
            assert!(on_panel(v(at[0] - 0.3, at[1] - half_height(PLOT_SIZE)), pad));
            assert!(on_panel(v(at[0] + 0.3, at[1] + half_height(PLOT_SIZE)), pad));
        }
        for cm in [0.0, LENGTH / 2.0, LENGTH] {
            assert!(on_panel(v(HEIGHT_X - 0.1, arch_y(cm) - half_height(PLOT_SIZE)), pad));
        }
        // the heights left of the axis, the right words right of the plot,
        // the share, coin and question in order, and the one placement row
        // left of the rule's column, between its rows' heights
        const {
            assert!(HEIGHT_X + 0.1 < PLOT_X[0] - 0.1);
            assert!(RIGHT_X > PLOT_X[1] + 0.05);
            assert!(RATIO_AT[0] + 1.5 < COIN_AT[0] - COIN_R && COIN_AT[0] + COIN_R < ASK_AT[0] - 0.15);
            assert!(ONE_AT[0] + 28.0 * the_shadow_rule::ROW_SIZE / 150.0 < RULE_AT[0] - 0.3);
            assert!((ONE_AT[1] - USES_AT[0][1]).abs() > 0.2 && (ONE_AT[1] - USES_AT[1][1]).abs() > 0.2);
        }
        let title_right = v(
            TITLE_AT[0] + width(TITLE, PLOT_SIZE),
            TITLE_AT[1] + half_height(PLOT_SIZE),
        );
        assert!(on_panel(v(TITLE_AT[0], TITLE_AT[1] + half_height(PLOT_SIZE)), pad) && on_panel(title_right, pad));
        assert!(TITLE_AT[1] - half_height(PLOT_SIZE) > arch_y(LENGTH) + 0.1);
        // the words at the plot's right, on the strip: "half" by its line
        // (gone before the average's words come); the average's words by
        // the flattened top, the percentage a row over them
        let mean = mean_shadow();
        let hh = half_height(RIGHT_SIZE);
        let percent_y = arch_y(mean) + PERCENT_RISE;
        for (text, y) in [
            (HALF, arch_y(LENGTH / 2.0)),
            ("3.82 cm", arch_y(mean)),
            ("about 64%", percent_y),
        ] {
            assert!(on_panel(v(RIGHT_X + width(text, RIGHT_SIZE), y + hh), pad), "{text}");
            assert!(on_panel(v(RIGHT_X, y - hh), pad), "{text}");
        }
        assert!(percent_y - hh > arch_y(mean) + hh + 0.03);
        // 08's two measured points sit inside the plot
        for theta in [TILTED, tilted_to(the_shadow_rule::TEST_SHADOW)] {
            assert!(arch_x(theta) > PLOT_X[0] && arch_x(theta) < PLOT_X[1]);
        }
        // the countdown, the share, the coin and its question in the band
        // between the lines at rest, clear of the stick lying along and of
        // the coast
        let along = ends(HOME, ALONG).map(card_point);
        let coast_top = (0..=n)
            .map(|i| card_point(coast.at(coast.length * i as f64 / n as f64)))
            .filter(|c| c[0] < panels[0][0])
            .map(|c| c[1])
            .fold(f64::MIN, f64::max);
        for lower_edge in [THINK_HERE[1] - THINK_R, RATIO_AT[1] - 0.3] {
            assert!(lower_edge > coast_top + 0.3);
        }
        assert!(lines.iter().all(|&y| (y - THINK_HERE[1]).abs() > THINK_R));
        assert!(THINK_HERE[1] - THINK_R > along[0][1] + 0.5);
        for (at, half_h) in [(RATIO_AT, 0.3), (COIN_AT, COIN_R), (ASK_AT, half_height(ASK_SIZE))] {
            assert!(lines.iter().all(|&y| (y - at[1]).abs() > half_h + 0.15), "{at:?}");
            assert!(at[0] < panels[0][0] - 0.3);
        }
        // "6 cm" over the stick lying along, under the next line up
        let label: P2 = LENGTH_AT;
        let hh = half_height(LABEL_SIZE) / CARD_UNIT;
        assert!(label[1] - hh > HOME[1] + 0.15);
        assert!(
            sheet::lines_on_card(s)
                .iter()
                .all(|&h| (h - label[1]).abs() > hh + 0.15)
        );
        // the rows on 08's console: the one placement row left of the
        // rule's column and between its rows' heights; the rule's column
        // inside the console's right
        let console = bounds(the_shadow_rule::CONSOLE);
        let row_h = half_height(the_shadow_rule::ROW_SIZE);
        assert!(ONE_AT[0] > console[0] + 0.2);
        assert!(ONE_AT[1] - row_h > console[2] + 0.1 && ONE_AT[1] + row_h < console[3] - 0.1);
        assert!(USES_AT[0][0] + 42.0 * the_shadow_rule::ROW_SIZE / 150.0 < console[1]);
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("09_1", "That settles the slide"),
            ("09_1", "which way the stick points"),
            ("09_2", "Watch the shadow"),
            ("09_3", "like the two we measured"),
            ("09_4", "more than half the stick"),
            ("09_5", "More"),
            ("09_5", "Join the two ends of the arch"),
            ("09_5", "So its average height is half the stick"),
            ("09_5", "Flatten it"),
            ("09_5", "three point eight two centimetres"),
            ("09_5", "about sixty-four percent"),
            ("09_6", "one point nine one crossings"),
            ("09_7", "six point zero centimetres"),
            ("09_7", "the stick's own length"),
            ("09_7", "A single placement said"),
            ("09_8", "two over pi"),
            ("09_8", "a coin will explain it"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
