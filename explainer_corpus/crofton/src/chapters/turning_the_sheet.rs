//! Chapter 11, turning the sheet. From 10's end (the coast cut into sticks,
//! its shadows laid end to end on the bar, 7.63 crossings on average over
//! every slide, for lines at 0°) the lines have so far run one way (11_1):
//! the count, its crossings, 04's readings and 10's rows go, a table of
//! averages over every slide comes up at the card's upper right with its
//! 0° row, and the sheet turns an eighth of a turn, the bar following it
//! live from the same sticks' shadows, to 8.74. Another eighth, and the
//! beach, which cast no shadow, now runs straight across the lines: its
//! shadow on the bar is its whole 3.5 cm; 8.23 (11_2). At three eighths,
//! 7.91 (11_3). Average the four and multiply by π: will that beat the
//! first placement's 22.0 cm? (11_4, a prediction.) Easily: 25.53 cm, about
//! 1% short (11_5). But four directions are only four: the sheet turns back
//! and, in silence, through half a turn, a plot drawing the total shadow
//! against the direction (11_6). The four directions are four points on it;
//! eight give 25.74 and thirty-two 25.81 (11_7). Flattened, as 09 flattened
//! the stick's arch, the curve stands at 16.433 cm, so 25.81 cm, the true
//! length to two places (11_8): the computer averaging 3,600 directions,
//! which shows the bet working here but does not prove it must (11_9).
//! The bar, the plot's curve and its moving point are drawn every frame
//! from the coast at the sheet's direction; the table, the list and the
//! flattening from the same model. Starts where chapter 10 ends, on 10's
//! [`Shore`]. Not registered yet: root wires it in
//! (`docs/candidate-10-11.md`).

use std::f64::consts::PI;

use explainer_kit::{Kit, Palette, tween, v};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::a_crowd_of_sticks::{
    self, LEFT_X, LOW_ROW, Marks, Shore, TOP_ROW, above, bar_axis, bar_spans, count_mark, direction_tag, piece,
    readings, sticks,
};
use super::draw::{
    dim_line, fade_in_all, fade_out_all, fade_to, fail, flat, line_style, now, say, stand, think, together,
};
use super::placement::{self, HALO_OPACITY, Placement, Pose, at_once, faded_line, glide, tracked};
use super::sheet;
use super::the_shadow_rule::{ROW_SIZE, hidden_back, left_at, row};
use crate::facts::{self, SHOWN_DIRECTIONS};
use crate::model::{self, BEACH, Curve, GAP, P2};
use crate::palette;

pub struct TurningTheSheet {
    pub kit: Kit,
}

// ---------------------------------------------------------------- the model

/// How many evenly spread directions 11_7 and 11_9 average over: each
/// grid starts at 0° and steps 180° ÷ m, so the first is 11_1's four
/// directions and each of the next three holds the one before it (11_7's
/// points add only the new ones). The last is the computer's.
pub const GRID: [usize; 5] = [4, 8, 16, 32, 3600];
pub const MOST: usize = GRID[GRID.len() - 1];

/// The coast's total shadow, cm, averaged over `m` directions evenly
/// spread over half a turn from 0°, as [`GRID`] spreads them.
pub fn mean_total(coast: &Curve, m: usize) -> f64 {
    (0..m).map(|k| coast.travel(180.0 * k as f64 / m as f64)).sum::<f64>() / m as f64
}

// ------------------------------------------------------------------ words

/// The chapter's words that carry numbers, each from the facts, and where
/// the beach's 3.5 cm and the flattened curve stand. Refused unless the
/// model gives what the voice says and every shown result follows from
/// its shown operands: each direction's travel to three places halved
/// gives its average; the four averages, averaged and times π, give
/// 25.53 cm, short of the true length and nearer to it than one
/// placement's 22.0; the beach turned a quarter casts its whole length;
/// more directions come nearer, from below; and the curve's height, the
/// average over 3,600 directions, halved and times π, gives the true
/// length to two places.
pub struct Words {
    /// "average over every slide"; each direction, `0^\circ` to
    /// `135^\circ`; its row, `15.260 \div 2 = 7.63`.
    pub header: String,
    pub directions: Vec<String>,
    pub rows: Vec<String>,
    /// "beach 3.5 cm", centred over the beach's shadow at 90°.
    pub beach: String,
    pub beach_x: f64,
    /// 11_4's rows: one placement; the four averages, averaged, times π;
    /// 11_5's result and "about 1% short".
    pub one: String,
    pub four: String,
    pub four_result: String,
    pub short: String,
    /// The plot's directions and heights, and its curve's flat height.
    pub ticks: [String; 3],
    pub heights: [String; 3],
    pub mean: String,
    pub mean_cm: f64,
    /// 11_7's list: its two names, each grid's directions and estimate.
    pub names: [String; 2],
    pub counts: Vec<String>,
    pub estimates: Vec<String>,
    /// 11_8's row: the height halved, times π; the true length.
    pub halve: String,
    pub times: String,
    pub truth: String,
}

/// The table's title, the list's names and 11_9's caption.
const HEADER: &str = "average over every slide";
const NAMES: [&str; 2] = ["directions", "estimate in cm"];
const CAPTION: &str = "computed, not proved";

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
    let number = |s: &str| s.parse::<f64>().map_err(|e| format!("{s}: {e}"));
    let coast = model::coast();
    let length = coast.length;
    let sticks = sticks(&coast);
    let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
    let marks = Marks::new(&coast, &sticks, first)?;
    let gap = agree("sheet.gap", fixed(GAP, 0))?;

    // the table: each direction's travel to three places, halved, gives
    // its average over every slide to two
    let (mut directions, mut rows, mut means) = (Vec::new(), Vec::new(), Vec::new());
    for beta in SHOWN_DIRECTIONS {
        let travel = coast.travel(f64::from(beta));
        let reach = agree(&format!("turn.{beta}.reach_3dp"), fixed(travel, 3))?;
        let mean = agree(&format!("turn.{beta}.slide_mean"), fixed(travel / GAP, 2))?;
        let from_shown = fixed(number(&reach)? / number(&gap)?, 2);
        if from_shown != mean {
            return Err(format!(
                "{beta}°: the table shows {mean}, its shown operands give {from_shown}"
            ));
        }
        directions.push(format!(r"{beta}^\circ"));
        rows.push(format!(r"{reach} \div {gap} = {mean}"));
        means.push(mean);
    }

    // 11_2: turned a quarter, the beach runs straight across the lines and
    // its shadow is its whole length
    let along = piece(&coast, coast.parts[BEACH].start, coast.parts[BEACH].end());
    if (along.travel(90.0) - along.length).abs() > 1e-9 {
        return Err("11_2 says the beach's shadow at 90° is its whole length".into());
    }
    let full = agree("beach.shadow_90", fixed(along.travel(90.0), 1))?;
    let spans = bar_spans(&coast, &sticks, PI / 2.0, length);
    let beach_x = (spans[marks.beach.start][0] + spans[marks.beach.end - 1][1]) / 2.0;

    // 11_4: one placement, its seven crossings times π (one crossing's
    // worth on this sheet)
    if (model::worth(GAP) - PI).abs() > 1e-12 {
        return Err("the rows write a crossing's worth as π cm".into());
    }
    let count = agree(
        "open.crossings",
        sheet::crossings(&coast, first.beta, first.slide).len().to_string(),
    )?;
    let one_cm = agree("open.estimate", fixed(number(&count)? * model::worth(GAP), 1))?;
    if fixed(number(&count)? * PI, 1) != one_cm {
        return Err(format!("the screen shows {one_cm}, its shown operands give another"));
    }

    // 11_5: the four averages as shown, averaged and times π (no rounded
    // mean between: 8.1275 shown to two places would give 25.54)
    let four_cm = coast.fixed_directions_estimate(SHOWN_DIRECTIONS.len(), GAP);
    let estimate = agree("turn.slide_estimate", fixed(four_cm, 2))?;
    let shown: f64 = means.iter().map(|m| number(m)).sum::<Result<f64, _>>()?;
    let from_shown = fixed(shown / means.len() as f64 * PI, 2);
    if from_shown != estimate {
        return Err(format!("11_5 shows {estimate}, its shown operands give {from_shown}"));
    }
    let one = number(&one_cm)?;
    if four_cm >= length || (length - one).abs() <= (length - four_cm).abs() {
        return Err(format!(
            "11_5 says {estimate} beats {one_cm} and falls short of {length}"
        ));
    }
    let percent = agree(
        "turn.slide_short_percent",
        fixed((length - four_cm) / length * 100.0, 0),
    )?;

    // the plot holds the total shadow at every direction
    let (low, high) = (0..MOST)
        .map(|k| coast.travel(180.0 * k as f64 / MOST as f64))
        .fold((f64::MAX, f64::MIN), |(lo, hi), t| (lo.min(t), hi.max(t)));
    if low < PLOT_LOW + 0.5 || high > PLOT_HIGH - 0.3 {
        return Err(format!(
            "the plot runs {PLOT_LOW} to {PLOT_HIGH} cm, the shadow {low} to {high}"
        ));
    }

    // 11_7: more directions come nearer, from below
    let mut counts = Vec::with_capacity(GRID.len());
    let mut estimates = Vec::with_capacity(GRID.len());
    let mut last = 0.0;
    for m in GRID {
        let est = coast.fixed_directions_estimate(m, GAP);
        if est <= last || est >= length {
            return Err(format!(
                "11_7 shows {m} directions nearer from below: {est} after {last}"
            ));
        }
        last = est;
        counts.push(m.to_string());
        estimates.push(agree(&format!("grid.{m}.estimate"), fixed(est, 2))?);
    }
    if estimates[0] != estimate {
        return Err("11_7's four directions are 11_5's".into());
    }
    agree("grid.most", MOST.to_string())?;

    // 11_8: the curve's height over the gap, times π; it agrees with the
    // exact identity's 2L/π at the places shown, and gives the true length
    let height = mean_total(&coast, MOST);
    let mean = agree(&format!("grid.{MOST}.mean_reach"), fixed(height, 3))?;
    if mean != get("crofton.mean_reach_3dp")? {
        return Err(format!("the computed {mean} is not the identity's at three places"));
    }
    let half = agree(&format!("grid.{MOST}.mean_crossings"), fixed(height / GAP, 4))?;
    if fixed(number(&mean)? / number(&gap)?, 4) != half {
        return Err(format!("11_8 shows {mean} ÷ {gap} = {half}"));
    }
    let result = fixed(number(&half)? * PI, 2);
    let truth = agree("coast.length_2dp", fixed(length, 2))?;
    if result != estimates[GRID.len() - 1] || result != truth {
        return Err(format!("11_8 shows {half} × π ≈ {result}, the true length {truth}"));
    }

    let tick = |d: f64| format!(r"{}^\circ", fixed(d, 0));
    Ok(Words {
        header: HEADER.into(),
        directions,
        rows,
        beach: format!("beach {full} cm"),
        beach_x,
        one: format!(r"\text{{one placement}}\quad {count} \times \pi \approx {one_cm}\text{{ cm}}"),
        four: format!(r"({}) \div {} \times \pi", means.join(" + "), means.len()),
        four_result: format!(r"\approx {estimate}\text{{ cm}}"),
        short: format!("about {percent}% short"),
        ticks: [tick(0.0), tick(90.0), tick(180.0)],
        heights: [PLOT_LOW, (PLOT_LOW + PLOT_HIGH) / 2.0, PLOT_HIGH].map(|h| fixed(h, 0)),
        mean: format!("{mean} cm"),
        mean_cm: height,
        names: NAMES.map(str::to_owned),
        counts,
        estimates,
        halve: format!(r"{mean} \div {gap} = {half}"),
        times: format!(r"{half} \times \pi \approx {result}\text{{ cm}}"),
        truth: format!("true length {truth} cm"),
    })
}

// ------------------------------------------------------------------ layout

/// The table at the card's upper right, on 09's panel: its title,
/// left-aligned; each row's direction and its halving, left-aligned in two
/// columns, a row a reading.
const HEADER_AT: Vec3 = [1.15, 3.35, 0.0];
const HEADER_SIZE: f64 = 20.0;
const TABLE_Y: [f64; 4] = [2.85, 2.4, 1.95, 1.5];
const DIRECTION_X: f64 = 1.2;
const HALVING_X: f64 = 2.1;
/// The plot, in the table's place: the direction across, 0° to 180°; the
/// total shadow up from 14 cm, 0.54 a centimetre (so the shadow's change
/// with direction fills it); its words, the title left-aligned; the
/// flattened height's words at its right, on 09's strip.
const PLOT_X: [f64; 2] = [1.45, 4.85];
const PLOT_Y0: f64 = 0.9;
const PLOT_LOW: f64 = 14.0;
const PLOT_HIGH: f64 = 18.0;
const PLOT_K: f64 = 0.54;
const PLOT_SIZE: f64 = 18.0;
const TICK_Y: f64 = 0.63;
const HEIGHT_X: f64 = 1.2;
const TITLE_AT: Vec3 = [1.1, 3.4, 0.0];
const TITLE: &str = "total shadow in cm";
const RIGHT_X: f64 = 4.95;
const RIGHT_SIZE: f64 = 20.0;
const AXIS_W: f64 = 2.0;
const CURVE_W: f64 = 4.0;
const MEAN_W: f64 = 2.5;
const CURVE_SAMPLES: usize = 360;
const FILL_OPACITY: f64 = 0.3;
const HEAD_R: f64 = 0.06;
const GRID_R: f64 = 0.04;
/// 11_7's list on the console, its names left-aligned at the console's
/// left and each grid in its column; 11_8's row under it, its pieces in a
/// line; 11_9's caption right of the list.
const LIST_Y: [f64; 2] = [-1.95, -2.5];
const LIST_SIZE: f64 = 20.0;
const COLUMN_X: [f64; 5] = [-2.4, -1.2, 0.0, 1.2, 2.6];
const ROW_C: Vec3 = [LEFT_X, -3.1, 0.0];
const CHAIN_GAP: f64 = 0.25;
const CAPTION_AT: Vec3 = [3.4, -2.22, 0.0];
/// 11_4's countdown where 09's stood, on its own backing so the turned
/// lines stay behind it (clear of the coast, which never rises above 1.9).
const THINK_HERE: Vec3 = [-2.6, 2.9, 0.0];
const THINK_BACK: (Vec3, f64, f64) = ([-2.6, 2.75, 0.0], 2.7, 2.0);

pub fn plot_x(beta: f64) -> f64 {
    PLOT_X[0] + (PLOT_X[1] - PLOT_X[0]) * beta / PI
}

pub fn plot_y(cm: f64) -> f64 {
    PLOT_Y0 + PLOT_K * (cm - PLOT_LOW)
}

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
/// Each eighth of a turn, the turn back, and the half turn at a steady
/// pace.
const EIGHTH_S: f64 = 3.0;
const BACK_S: f64 = 3.0;
const SWEEP_S: f64 = 12.0;
const FLATTEN_S: f64 = 2.5;

// --------------------------------------------------------- what is drawn

/// Turn the sheet by `by` radians about `pin` (card cm), `rt` seconds at
/// `rate`; it ends exactly there.
fn turn(stage: &mut Stage<'_>, place: Placement, by: f64, pin: P2, rt: f64, rate: fn(f64) -> f64) -> fmn::Result<()> {
    place.begin(stage, by, 0.0, pin)?;
    place.run(stage, &[(1.0, rt)], rate)
}

/// The total shadow from direction 0 to the direction `upto(s)`, on the
/// plot, at the opacity `alpha` holds. In the scene.
fn live_curve(
    stage: &mut Stage<'_>,
    coast: Curve,
    upto: impl Fn(&fmn::mobject::Stage) -> f64 + 'static,
    alpha: Mob,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let to = upto(s).clamp(0.0, PI);
        let steps = ((to / PI * CURVE_SAMPLES as f64).ceil() as usize).max(1);
        let curve: Vec<VMobject> = (to > 1e-6)
            .then(|| {
                let pts: Vec<Vec3> = (0..=steps)
                    .map(|j| {
                        let beta = to * j as f64 / steps as f64;
                        v(plot_x(beta), plot_y(coast.travel(beta.to_degrees())))
                    })
                    .collect();
                polyline(pts)
                    .style(faded_line(palette::stick(), CURVE_W, tracked(s, alpha)))
                    .build()
            })
            .into_iter()
            .collect();
        s.add(v_group(curve))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A gold point on the plot at the sheet's direction and its total
/// shadow, at the opacity `alpha` holds. In the scene.
fn live_head(stage: &mut Stage<'_>, coast: Curve, place: Placement, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, beta) = (tracked(s, alpha), place.pose(s).beta.clamp(0.0, PI));
        let head: Vec<VMobject> = (o > 1e-3)
            .then(|| {
                Circle::new()
                    .radius(HEAD_R)
                    .arc_center(v(plot_x(beta), plot_y(coast.travel(beta.to_degrees()))))
                    .style(flat(palette::touch(), o))
                    .build()
            })
            .into_iter()
            .collect();
        s.add(v_group(head))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The region under the curve down to the plot's floor, its top moved the
/// share `t` of the way to the flat height `mean` (each point straight up
/// or down), frame points: along the floor, then back along its top.
fn fill_points(coast: &Curve, mean: f64, t: f64) -> Vec<Vec3> {
    let mut pts = vec![v(PLOT_X[0], PLOT_Y0), v(PLOT_X[1], PLOT_Y0)];
    for j in (0..=CURVE_SAMPLES).rev() {
        let beta = PI * j as f64 / CURVE_SAMPLES as f64;
        let h = coast.travel(beta.to_degrees());
        pts.push(v(plot_x(beta), plot_y(h + (mean - h) * t)));
    }
    pts
}

/// The fill at the share `morph` holds of the flattening, at the opacity
/// `alpha` holds. In the scene.
fn live_fill(stage: &mut Stage<'_>, coast: Curve, mean: f64, morph: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let fill = Polygon::new(fill_points(&coast, mean, tracked(s, morph)))
            .style(flat(palette::stick(), FILL_OPACITY * tracked(s, alpha)))
            .build();
        s.add(v_group(vec![fill]))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Frame points as one open polyline. Detached.
fn frame_path(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points).style(line_style(color, width)).build())
}

/// The table's title. Detached.
fn header(kit: &Kit, stage: &mut Stage<'_>, words: &Words) -> fmn::Result<Mob> {
    let m = kit.text(stage, &words.header, HEADER_SIZE, Palette::dim(), HEADER_AT)?;
    Ok(left_at(stage, m, HEADER_AT))
}

/// Row `i` of the table: its direction and its halving. Detached.
fn table_row(kit: &Kit, stage: &mut Stage<'_>, words: &Words, i: usize) -> fmn::Result<Vec<Mob>> {
    let y = TABLE_Y[i];
    Ok(vec![
        row(kit, stage, &words.directions[i], Palette::dim(), v(DIRECTION_X, y))?,
        row(kit, stage, &words.rows[i], Palette::ink(), v(HALVING_X, y))?,
    ])
}

/// The plot's axes and words. Detached.
fn plot_axes(kit: &Kit, stage: &mut Stage<'_>, words: &Words) -> fmn::Result<Vec<Mob>> {
    let top = plot_y(PLOT_HIGH) + 0.12;
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
    for (tex, beta) in words.ticks.iter().zip([0.0, PI / 2.0, PI]) {
        out.push(kit.tex(stage, tex, PLOT_SIZE, Palette::dim(), &[], v(plot_x(beta), TICK_Y))?);
    }
    let levels = [PLOT_LOW, (PLOT_LOW + PLOT_HIGH) / 2.0, PLOT_HIGH];
    for (text, cm) in words.heights.iter().zip(levels) {
        out.push(kit.text(stage, text, PLOT_SIZE, Palette::dim(), v(HEIGHT_X, plot_y(cm)))?);
    }
    let title = kit.text(stage, TITLE, PLOT_SIZE, palette::stick(), TITLE_AT)?;
    out.push(left_at(stage, title, TITLE_AT));
    Ok(out)
}

/// The gold points on the curve at the `m` directions of a grid, leaving
/// out those of the grid of `before` directions already shown (0: none).
/// Detached.
fn grid_points(stage: &mut Stage<'_>, coast: &Curve, m: usize, before: usize) -> Mob {
    let dots: Vec<VMobject> = (0..m)
        .filter(|&k| before == 0 || !(k * before).is_multiple_of(m))
        .map(|k| {
            let beta = PI * k as f64 / m as f64;
            Circle::new()
                .radius(GRID_R)
                .arc_center(v(plot_x(beta), plot_y(coast.travel(beta.to_degrees()))))
                .style(flat(palette::touch(), 1.0))
                .build()
        })
        .collect();
    stage.arena_mut().add(v_group(dots))
}

/// The list's two names. Detached.
fn list_names(kit: &Kit, stage: &mut Stage<'_>, words: &Words) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(2);
    for (name, y) in words.names.iter().zip(LIST_Y) {
        let at = v(LEFT_X, y);
        let m = kit.text(stage, name, LIST_SIZE, Palette::dim(), at)?;
        out.push(left_at(stage, m, at));
    }
    Ok(out)
}

/// Column `i` of the list: a grid's directions and its estimate, the
/// computer's estimate in the true length's colour. Detached.
fn list_column(kit: &Kit, stage: &mut Stage<'_>, words: &Words, i: usize) -> fmn::Result<Vec<Mob>> {
    let x = COLUMN_X[i];
    let shade = if i + 1 == GRID.len() {
        palette::length()
    } else {
        Palette::ink()
    };
    Ok(vec![
        kit.text(stage, &words.counts[i], LIST_SIZE, Palette::ink(), v(x, LIST_Y[0]))?,
        kit.text(stage, &words.estimates[i], LIST_SIZE, shade, v(x, LIST_Y[1]))?,
    ])
}

/// Words at the plot's right, left-aligned at the frame height `y`.
/// Detached.
fn right_word(kit: &Kit, stage: &mut Stage<'_>, words: &str, y: f64) -> fmn::Result<Mob> {
    let at = v(RIGHT_X, y);
    let m = kit.text(stage, words, RIGHT_SIZE, Palette::ink(), at)?;
    Ok(left_at(stage, m, at))
}

/// `next` placed with its left edge `CHAIN_GAP` right of `before`.
fn chained(stage: &mut Stage<'_>, next: Mob, before: Mob) -> Mob {
    stage.next_to(next, before, RIGHT, CHAIN_GAP, ORIGIN);
    next
}

impl SceneConstruct for TurningTheSheet {
    fn name(&self) -> &str {
        "11_turning_the_sheet"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let sticks = sticks(&coast);
        let earlier = a_crowd_of_sticks::words(first).map_err(fail)?;
        let words = words().map_err(fail)?;
        let marks = Marks::new(&coast, &sticks, first).map_err(fail)?;

        // where 10 ended: the shore with the whole bar at 0°, the count and
        // the crossings, the ticks; the bar's axis, 04's readings and the
        // mark at seven gaps; 15.26 cm, the result and "lines at 0°". The
        // countdown's backing, unseen, comes next, behind every word made
        // later
        let shore = Shore::new(&kit, stage, &coast, &sticks, first, true)?;
        let axis = bar_axis(&kit, stage, &earlier)?;
        let log = readings(&kit, stage, &earlier)?;
        let mark = count_mark(stage, earlier.first)?;
        let total = above(&kit, stage, &earlier.total, marks.end_x)?;
        let result = row(&kit, stage, &earlier.result, Palette::ink(), LOW_ROW)?;
        let tag = direction_tag(&kit, stage, &earlier)?;
        for &m in axis.iter().chain(&log).chain([&mark, &total, &result, &tag]) {
            stage.add_to_scene(m)?;
        }
        let think_back = hidden_back(stage, THINK_BACK)?;
        let place = shore.place;

        // 11_1: so far one way; the count, its crossings, the readings and
        // 10's rows go, the table comes up with its 0° row; an eighth of a
        // turn, the bar following; 8.74
        let line = say(&kit, stage, "11_1")?;
        line.wait_for(stage, "So far the lines have all run one way")?;
        // The old direction tag occupies the first table row's location.
        // Clear it before creating the table, not after their overlap.
        fade_out_all(stage, &[tag], FADE_S)?;
        let title = header(&kit, stage, &words)?;
        let mut table = vec![title];
        table.extend(table_row(&kit, stage, &words, 0)?);
        let mut gone = log.clone();
        gone.extend([mark, total, result]);
        at_once(
            stage,
            &table,
            &gone,
            vec![
                tween(shore.count_alpha, 0.0, FADE_S)?,
                tween(shore.dots_alpha, 0.0, FADE_S)?,
                fade_to(shore.panels[0], 1.0, FADE_S)?,
                fade_to(shore.panels[1], 1.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        line.wait_for(stage, "Turn the sheet an eighth of a turn")?;
        turn(stage, place, PI / 4.0, [0.0, 0.0], EIGHTH_S, smooth)?;
        line.wait_for(stage, "eight point seven four crossings")?;
        let row45 = table_row(&kit, stage, &words, 1)?;
        fade_in_all(stage, &row45, FADE_S)?;
        table.extend(row45);

        // 11_2: another eighth, the beach picked out on the coast and the
        // bar; it now casts its whole 3.5 cm; 8.23
        let line = say(&kit, stage, "11_2")?;
        line.wait_for(stage, "Another eighth")?;
        together(
            stage,
            vec![
                dim_line(shore.beach, HALO_OPACITY, FADE_S)?,
                tween(shore.beach_bar, 1.0, FADE_S)?,
            ],
        )?;
        turn(stage, place, PI / 4.0, [0.0, 0.0], EIGHTH_S, smooth)?;
        line.wait_for(stage, "three and a half centimetres")?;
        let beach = above(&kit, stage, &words.beach, words.beach_x)?;
        fade_in_all(stage, &[beach], FADE_S)?;
        line.wait_for(stage, "eight point two three")?;
        let row90 = table_row(&kit, stage, &words, 2)?;
        fade_in_all(stage, &row90, FADE_S)?;
        table.extend(row90);

        // 11_3: three eighths; 7.91
        let line = say(&kit, stage, "11_3")?;
        line.wait_for(stage, "And one more eighth")?;
        at_once(
            stage,
            &[],
            &[beach],
            vec![
                dim_line(shore.beach, 0.0, FADE_S)?,
                tween(shore.beach_bar, 0.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        turn(stage, place, PI / 4.0, [0.0, 0.0], EIGHTH_S, smooth)?;
        line.wait_for(stage, "seven point nine one")?;
        let row135 = table_row(&kit, stage, &words, 3)?;
        fade_in_all(stage, &row135, FADE_S)?;
        table.extend(row135);

        // 11_4: the four averaged, times π, against one placement's 22.0;
        // better or not?
        let line = say(&kit, stage, "11_4")?;
        line.wait_for(stage, "Average those four")?;
        let four = row(&kit, stage, &words.four, Palette::ink(), LOW_ROW)?;
        fade_in_all(stage, &[four], FADE_S)?;
        line.wait_for(stage, "twenty-two point zero centimetres")?;
        let one = row(&kit, stage, &words.one, Palette::ink(), TOP_ROW)?;
        fade_in_all(stage, &[one], FADE_S)?;
        stage.play(fade_to(think_back, 1.0, FADE_S)?)?;
        let q = think(&kit, stage, THINK_HERE)?;

        // 11_5: easily; 25.53 cm, about 1% short
        let line = say(&kit, stage, "11_5")?;
        line.wait_for(stage, "Easily")?;
        at_once(stage, &[], &[q], vec![fade_to(think_back, 0.0, FADE_S)?], 0.3)?;
        line.wait_for(stage, "Twenty-five point five three centimetres")?;
        let approx = kit.tex(stage, &words.four_result, ROW_SIZE, palette::length(), &[], LOW_ROW)?;
        let approx = chained(stage, approx, four);
        fade_in_all(stage, &[approx], FADE_S)?;
        line.wait_for(stage, "about one percent short")?;
        let short = kit.text(stage, &words.short, LIST_SIZE, Palette::ink(), LOW_ROW)?;
        let short = chained(stage, short, approx);
        fade_in_all(stage, &[short], FADE_S)?;

        // 11_6: only four; the table gives way to the plot, the sheet turns
        // back, and in silence through half a turn about a point on a line
        // (so its lines end where they began), the curve drawn behind the
        // moving point and the bar following
        let line = say(&kit, stage, "11_6")?;
        line.wait_for(stage, "Turn the sheet back to the start")?;
        let axes = plot_axes(&kit, stage, &words)?;
        // 11_8's fill, unseen until then, made before the plot so its axes,
        // the curve and the moving point are drawn over it
        let [morph, fill_alpha] = [0.0, 0.0].map(|x| stage.add_value_tracker(x));
        live_fill(stage, coast.clone(), words.mean_cm, morph, fill_alpha)?;
        at_once(stage, &axes, &table, vec![], FADE_S)?;
        turn(stage, place, -3.0 * PI / 4.0, [0.0, 0.0], BACK_S, smooth)?;
        let [drawing, curve_alpha, head_alpha] = [0.0, 1.0, 0.0].map(|x| stage.add_value_tracker(x));
        let held = stage.add_value_tracker(0.0);
        live_curve(
            stage,
            coast.clone(),
            move |s| {
                if tracked(s, drawing) > 0.5 {
                    place.pose(s).beta
                } else {
                    s.tracker_value(held).unwrap_or(0.0)
                }
            },
            curve_alpha,
        )?;
        live_head(stage, coast.clone(), place, head_alpha)?;
        kit.hold(stage)?;
        stage.set_tracker_value(drawing, 1.0)?;
        stage.play(tween(head_alpha, 1.0, 0.3)?)?;
        let pin = placement::half_turn_pin(place.pose(stage), 0.0);
        turn(stage, place, PI, pin, SWEEP_S, linear)?;
        stage.set_tracker_value(held, PI)?;
        stage.set_tracker_value(drawing, 0.0)?;
        stage.play(tween(head_alpha, 0.0, 0.3)?)?;

        // 11_7: the four directions are four points on the curve; the bar
        // and the rows give way to the list; eight, then sixteen and
        // thirty-two
        let line = say(&kit, stage, "11_7")?;
        line.wait_for(stage, "Our four directions are these four points")?;
        let names = list_names(&kit, stage, &words)?;
        let mut shown = names.clone();
        shown.extend(list_column(&kit, stage, &words, 0)?);
        let points4 = grid_points(stage, &coast, GRID[0], 0);
        shown.push(points4);
        let mut rows = axis.clone();
        rows.extend([one, four, approx, short]);
        at_once(stage, &shown, &rows, vec![tween(shore.bar_alpha, 0.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "Eight give")?;
        let mut shown = list_column(&kit, stage, &words, 1)?;
        let points8 = grid_points(stage, &coast, GRID[1], GRID[0]);
        shown.push(points8);
        fade_in_all(stage, &shown, FADE_S)?;
        line.wait_for(stage, "thirty-two give")?;
        let mut shown = list_column(&kit, stage, &words, 2)?;
        shown.extend(list_column(&kit, stage, &words, 3)?);
        let points32 = grid_points(stage, &coast, GRID[3], GRID[1]);
        shown.push(points32);
        fade_in_all(stage, &shown, FADE_S)?;

        // 11_8: flattened, as 09's arch was; its height 16.433 cm, halved,
        // times π, 25.81 cm, the true length to two places
        let line = say(&kit, stage, "11_8")?;
        line.wait_for(stage, "Flatten the curve")?;
        at_once(
            stage,
            &[],
            &[points4, points8, points32],
            vec![tween(fill_alpha, 1.0, FADE_S)?],
            FADE_S,
        )?;
        stage.play(glide(morph, 1.0, FLATTEN_S, smooth)?)?;
        stage.set_tracker_value(morph, 1.0)?;
        line.wait_for(stage, "its height gives")?;
        let level = plot_y(words.mean_cm);
        let top = frame_path(
            stage,
            vec![v(PLOT_X[0], level), v(PLOT_X[1], level)],
            Palette::ink(),
            MEAN_W,
        );
        let mean = right_word(&kit, stage, &words.mean, level)?;
        let halve = row(&kit, stage, &words.halve, Palette::ink(), ROW_C)?;
        fade_in_all(stage, &[top, mean, halve], FADE_S)?;
        line.wait_for(stage, "twenty-five point eight one centimetres")?;
        let times = kit.tex(stage, &words.times, ROW_SIZE, palette::length(), &[], ROW_C)?;
        let times = chained(stage, times, halve);
        fade_in_all(stage, &[times], FADE_S)?;
        line.wait_for(stage, "the true length to two decimal places")?;
        let truth = kit.text(stage, &words.truth, LIST_SIZE, palette::length(), ROW_C)?;
        let truth = chained(stage, truth, times);
        fade_in_all(stage, &[truth], FADE_S)?;

        // 11_9: the computer's 3,600 directions; computed, not proved
        let line = say(&kit, stage, "11_9")?;
        line.wait_for(stage, "three thousand six hundred directions")?;
        let most = list_column(&kit, stage, &words, GRID.len() - 1)?;
        fade_in_all(stage, &most, FADE_S)?;
        line.wait_for(stage, "it doesn't prove that it must")?;
        let caption = kit.text(stage, CAPTION, LIST_SIZE, Palette::dim(), CAPTION_AT)?;
        let caption = left_at(stage, caption, CAPTION_AT);
        fade_in_all(stage, &[caption], FADE_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::a_crowd_of_sticks::{ABOVE_SIZE, BACKS_09, BAR_X0, BAR_Y};
    use super::super::draw::THINK_R;
    use super::super::the_shadow_rule::CONSOLE;
    use super::*;
    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W};

    fn first() -> Pose {
        Pose::new(0.0, sheet::shown_slide(&model::coast(), 0.0))
    }

    fn half_height(size: f64) -> f64 {
        size / 150.0
    }

    fn width(text: &str, size: f64) -> f64 {
        text.chars().count() as f64 * size / 150.0
    }

    /// A TeX row's shown characters, roughly: its commands dropped.
    fn shown(tex: &str) -> String {
        tex.replace(r"\text", "")
            .replace(r"\div", "÷")
            .replace(r"\times", "×")
            .replace(r"\approx", "≈")
            .replace(r"\pi", "π")
            .replace(r"\quad", "  ")
            .replace(r"^\circ", "°")
            .replace(['{', '}', '\\'], "")
    }

    fn bounds((at, w, h): (Vec3, f64, f64)) -> [f64; 4] {
        [at[0] - w / 2.0, at[0] + w / 2.0, at[1] - h / 2.0, at[1] + h / 2.0]
    }

    /// The words on screen say what the facts and the model say.
    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words().unwrap();
        assert_eq!(w.directions, [r"0^\circ", r"45^\circ", r"90^\circ", r"135^\circ"]);
        assert_eq!(
            w.rows,
            [
                r"15.260 \div 2 = 7.63",
                r"17.485 \div 2 = 8.74",
                r"16.452 \div 2 = 8.23",
                r"15.821 \div 2 = 7.91",
            ]
        );
        assert_eq!(w.beach, "beach 3.5 cm");
        assert_eq!(w.one, r"\text{one placement}\quad 7 \times \pi \approx 22.0\text{ cm}");
        assert_eq!(w.four, r"(7.63 + 8.74 + 8.23 + 7.91) \div 4 \times \pi");
        assert_eq!(w.four_result, r"\approx 25.53\text{ cm}");
        assert_eq!(w.short, "about 1% short");
        assert_eq!(w.heights, ["14", "16", "18"]);
        assert_eq!(w.mean, "16.433 cm");
        assert_eq!(w.counts, ["4", "8", "16", "32", "3600"]);
        assert_eq!(w.estimates, ["25.53", "25.74", "25.80", "25.81", "25.81"]);
        assert_eq!(w.halve, r"16.433 \div 2 = 8.2165");
        assert_eq!(w.times, r"8.2165 \times \pi \approx 25.81\text{ cm}");
        assert_eq!(w.truth, "true length 25.81 cm");
    }

    /// 11_5 shows no rounded mean of the four: shown to two places it
    /// would be 8.13 and give 25.54, not the 25.53 the model gives (a
    /// negative control on the row's design).
    #[test]
    fn the_four_directions_show_no_rounded_mean() {
        let means = [7.63, 8.74, 8.23, 7.91];
        let mean = means.iter().sum::<f64>() / 4.0;
        assert_eq!(facts::fixed(mean * PI, 2), "25.53");
        assert_eq!(facts::fixed(mean, 2), "8.13");
        assert_eq!(facts::fixed(8.13 * PI, 2), "25.54");
        let coast = model::coast();
        assert_eq!(facts::fixed(coast.fixed_directions_estimate(4, GAP), 2), "25.53");
        assert!((mean_total(&coast, 4) / GAP * PI - coast.fixed_directions_estimate(4, GAP)).abs() < 1e-12);
    }

    /// The grid's estimates rise toward the true length from below, the
    /// computer's within a hundred-thousandth of a centimetre; thirty-two
    /// directions already match it to two places and sixteen do not. Its
    /// average shadow agrees with 2L/π, and each of the plotted grids holds
    /// the one before (3,600 does not hold 32, and is never plotted as
    /// points). The four directions are the table's.
    #[test]
    fn the_grid_rises_and_settles() {
        let coast = model::coast();
        let l = coast.length;
        let est: Vec<f64> = GRID.iter().map(|&m| coast.fixed_directions_estimate(m, GAP)).collect();
        for w in est.windows(2) {
            assert!(w[0] < w[1], "{est:?}");
        }
        assert!(est.iter().all(|&e| e < l));
        assert!((est[GRID.len() - 1] - l).abs() < 1e-5);
        assert_eq!(facts::fixed(est[3], 2), facts::fixed(l, 2));
        assert_ne!(facts::fixed(est[2], 2), facts::fixed(l, 2));
        assert!((mean_total(&coast, MOST) - 2.0 * l / PI).abs() < 1e-5);
        for w in GRID[..4].windows(2) {
            assert_eq!(w[1] % w[0], 0, "{w:?}");
        }
        assert_ne!(MOST % GRID[3], 0);
        let four: Vec<u32> = (0..GRID[0]).map(|k| (180 * k / GRID[0]) as u32).collect();
        assert_eq!(four, SHOWN_DIRECTIONS);
        // a negative control: one direction alone is not the length
        assert!((coast.fixed_directions_estimate(1, GAP) - l).abs() > 1.0);
    }

    /// The plot holds the total shadow at every tenth of a degree inside
    /// its axes and on 09's panel, the flattened height between its floor
    /// and top, its words on the panel; the grid's points stay apart.
    #[test]
    fn the_plot_keeps_its_room() {
        let coast = model::coast();
        let panel = bounds(BACKS_09[0]);
        let strip = bounds(BACKS_09[1]);
        for i in 0..=1800 {
            let t = coast.travel(f64::from(i) / 10.0);
            assert!((PLOT_LOW + 0.5..=PLOT_HIGH - 0.3).contains(&t), "{i} {t}");
        }
        let top = plot_y(PLOT_HIGH) + 0.12;
        assert!(PLOT_X[0] > panel[0] + 0.3 && PLOT_X[1] < panel[1] - 0.2);
        assert!(TICK_Y - half_height(PLOT_SIZE) > panel[2] + 0.05);
        assert!(top < TITLE_AT[1] - half_height(PLOT_SIZE) - 0.05);
        assert!(TITLE_AT[1] + half_height(PLOT_SIZE) < panel[3] - 0.05);
        assert!(TITLE_AT[0] + width(TITLE, PLOT_SIZE) < panel[1] - 0.2);
        assert!(HEIGHT_X - width("18", PLOT_SIZE) / 2.0 > panel[0] + 0.05);
        assert!(HEIGHT_X + width("18", PLOT_SIZE) / 2.0 < PLOT_X[0] - 0.05);
        // the flattened height's words on the panel and its strip
        let w = words().unwrap();
        let level = plot_y(w.mean_cm);
        assert!(level > plot_y(PLOT_LOW) && level < top);
        assert!(level - half_height(RIGHT_SIZE) > strip[2] + 0.05);
        assert!(RIGHT_X > PLOT_X[1] + 0.05 && RIGHT_X + width(&w.mean, RIGHT_SIZE) < strip[1] - 0.05);
        // thirty-two points, one step of the grid apart, never touch
        let step = plot_x(PI / GRID[3] as f64) - plot_x(0.0);
        assert!(step > 2.0 * GRID_R + 0.02, "{step}");
    }

    /// The table on 09's panel: its title and rows inside it, apart.
    #[test]
    fn the_table_keeps_its_room() {
        let w = words().unwrap();
        let panel = bounds(BACKS_09[0]);
        assert!(HEADER_AT[0] > panel[0] + 0.1);
        assert!(HEADER_AT[0] + width(HEADER, HEADER_SIZE) < panel[1] - 0.2);
        assert!(HEADER_AT[1] + half_height(HEADER_SIZE) < panel[3] - 0.05);
        assert!(TABLE_Y[0] + half_height(ROW_SIZE) < HEADER_AT[1] - half_height(HEADER_SIZE) - 0.1);
        for pair in TABLE_Y.windows(2) {
            assert!(pair[0] - pair[1] > 2.0 * half_height(ROW_SIZE) + 0.1);
        }
        assert!(TABLE_Y[3] - half_height(ROW_SIZE) > panel[2] + 0.1);
        for (d, r) in w.directions.iter().zip(&w.rows) {
            assert!(DIRECTION_X + width(&shown(d), ROW_SIZE) < HALVING_X - 0.15, "{d}");
            assert!(HALVING_X + width(&shown(r), ROW_SIZE) < panel[1] - 0.1, "{r}");
        }
    }

    /// The console holds the beach's label over its 3.5 cm of bar at 90°,
    /// 11_4's rows clear of the bar at 135°, 11_5's row and 11_8's in a line
    /// inside it, the list's columns apart and the caption right of them.
    #[test]
    fn the_console_keeps_its_rooms() {
        let w = words().unwrap();
        let console = bounds(CONSOLE);
        let coast = model::coast();
        let sticks = sticks(&coast);
        let marks = Marks::new(&coast, &sticks, first()).unwrap();
        let spans = bar_spans(&coast, &sticks, PI / 2.0, coast.length);
        let (a, b) = (spans[marks.beach.start][0], spans[marks.beach.end - 1][1]);
        assert!(((b - a) / a_crowd_of_sticks::BAR_PER_CM - 3.5).abs() < 1e-9);
        assert!((w.beach_x - (a + b) / 2.0).abs() < 1e-12);
        assert!(w.beach_x - width(&w.beach, ABOVE_SIZE) / 2.0 > console[0] + 0.1);
        let before = piece(&coast, 0.0, coast.parts[BEACH].start).travel(90.0);
        assert!((a - (BAR_X0 + before * a_crowd_of_sticks::BAR_PER_CM)).abs() < 1e-9);
        // the rows: one placement over the bar, the four under it; each
        // chain inside the console
        assert!(TOP_ROW[1] - half_height(ROW_SIZE) > BAR_Y + 0.07 + 0.05);
        for chain in [
            vec![(shown(&w.one), ROW_SIZE)],
            vec![
                (shown(&w.four), ROW_SIZE),
                (shown(&w.four_result), ROW_SIZE),
                (w.short.clone(), LIST_SIZE),
            ],
            vec![
                (shown(&w.halve), ROW_SIZE),
                (shown(&w.times), ROW_SIZE),
                (w.truth.clone(), LIST_SIZE),
            ],
        ] {
            let long: f64 = chain.iter().map(|(t, s)| width(t, *s) + CHAIN_GAP).sum();
            assert!(LEFT_X + long < console[1] - 0.2, "{chain:?}");
        }
        assert!(ROW_C[1] - half_height(ROW_SIZE) > console[2] + 0.05);
        assert!(LIST_Y[1] - half_height(LIST_SIZE) > ROW_C[1] + half_height(ROW_SIZE) + 0.1);
        assert!(LIST_Y[0] + half_height(LIST_SIZE) < console[3] - 0.1);
        // the list: names clear of the first column, columns apart, the
        // caption right of the last and inside the console
        let names = w.names.iter().map(|n| width(n, LIST_SIZE)).fold(0.0, f64::max);
        let widest = |i: usize| width(&w.counts[i], LIST_SIZE).max(width(&w.estimates[i], LIST_SIZE));
        assert!(LEFT_X + names < COLUMN_X[0] - widest(0) / 2.0 - 0.2);
        for i in 1..GRID.len() {
            assert!(
                COLUMN_X[i] - widest(i) / 2.0 > COLUMN_X[i - 1] + widest(i - 1) / 2.0 + 0.2,
                "{i}"
            );
        }
        let last = GRID.len() - 1;
        assert!(CAPTION_AT[0] > COLUMN_X[last] + widest(last) / 2.0 + 0.2);
        assert!(CAPTION_AT[0] + width(CAPTION, LIST_SIZE) < console[1] - 0.1);
        const {
            assert!(LIST_Y[1] < CAPTION_AT[1] && CAPTION_AT[1] < LIST_Y[0]);
        }
    }

    /// 11_4's countdown on its backing, inside the card's upper band, the
    /// coast never under it.
    #[test]
    fn the_countdown_stands_on_its_backing() {
        let back = bounds(THINK_BACK);
        let card_top = CARD_CENTRE[1] + CARD_UNIT * CARD_H / 2.0;
        let card_left = CARD_CENTRE[0] - CARD_UNIT * CARD_W / 2.0;
        assert!(back[3] < card_top && back[0] > card_left);
        assert!(THINK_HERE[0] - THINK_R > back[0] + 0.05 && THINK_HERE[0] + THINK_R < back[1] - 0.05);
        assert!(THINK_HERE[1] + THINK_R < back[3] - 0.05);
        // the note under the ring
        let note = THINK_HERE[1] - THINK_R - 0.3;
        assert!(note - half_height(18.0) > back[2] + 0.03);
        assert!(THINK_HERE[0] - width("pause for more time", 18.0) / 2.0 > back[0] + 0.05);
        let coast = model::coast();
        let n = (coast.length / 0.02).ceil() as usize;
        for i in 0..=n {
            let p = coast.at(coast.length * i as f64 / n as f64);
            let c = [CARD_CENTRE[0] + CARD_UNIT * p[0], CARD_CENTRE[1] + CARD_UNIT * p[1]];
            assert!(
                !(back[0] - 0.1..=back[1] + 0.1).contains(&c[0]) || c[1] < back[2] - 0.1,
                "{c:?}"
            );
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("11_1", "So far the lines have all run one way"),
            ("11_1", "Turn the sheet an eighth of a turn"),
            ("11_1", "eight point seven four crossings"),
            ("11_2", "Another eighth"),
            ("11_2", "three and a half centimetres"),
            ("11_2", "eight point two three"),
            ("11_3", "And one more eighth"),
            ("11_3", "seven point nine one"),
            ("11_4", "Average those four"),
            ("11_4", "twenty-two point zero centimetres"),
            ("11_5", "Easily"),
            ("11_5", "Twenty-five point five three centimetres"),
            ("11_5", "about one percent short"),
            ("11_6", "Turn the sheet back to the start"),
            ("11_7", "Our four directions are these four points"),
            ("11_7", "Eight give"),
            ("11_7", "thirty-two give"),
            ("11_8", "Flatten the curve"),
            ("11_8", "its height gives"),
            ("11_8", "twenty-five point eight one centimetres"),
            ("11_8", "the true length to two decimal places"),
            ("11_9", "three thousand six hundred directions"),
            ("11_9", "it doesn't prove that it must"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
