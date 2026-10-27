//! Chapter 05, "Every possible handful": the handfuls of five a company of
//! seventy-one boxes could sell, counted up live past thirteen million, too
//! many to look at; so a company of five boxes, of which you see two, the
//! two turning gold (05_1). Its ten pairs as a table, one row each, the
//! seen two squares gold, every row as likely as the next, held to be read
//! (05_2). The pair two and five worked on the five boxes: its unseen boxes
//! turn blue, its two gaps' arcs and counts, their total five minus two,
//! and the recipe for a pair (05_3); its estimate, six and a half, into its
//! row, and the pair one and two's, two (05_4). All ten estimates, each
//! row's on a small plot beside a line at the count, and the prediction
//! (05_5); the estimates drop onto a balance that settles exactly at five,
//! then the largest numbers alone, which settle at four (05_6); what
//! unbiased means, and that one small company is not a proof (05_7).
//!
//! Every estimate is the exact fraction the model computes, shown exactly
//! ("3.5", never rounded); the scene recomputes each one from its pair's
//! gaps and refuses a table it would contradict. Highlights are halos and
//! colour changes are replacements (chapter 04's builders): no typeset text
//! is indicated or recoloured in place.

use explainer_kit::{Kit, Palette, choose, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    BODY_O, BOX_H, BOX_W, NUMBER_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, game_box, line_style,
    move_all, pause, say, stroke, think, wait_until,
};
use super::friends_line::{
    ARC_LIFT, ARC_W, PANEL_SIZE, THINK_AT, arc_points, at, gap_spans, handfuls, is_handful, list, printed, value, whole,
};
use super::what_we_assume::{
    formula, formula_colors, gaps_line, glow, instance, replace, roles, sum_texts, unseen_below, words_left,
};
use crate::facts;
use crate::model::Q;
use crate::palette;

pub struct EveryHandful {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ESTIMATE_HEAD: &str = "estimate";
const UNBIASED: &str = "unbiased: average estimate = count";
const OTHER: &str = "every other count: ?";

/// The live count of the big company's handfuls (05_1) and its caption.
const BIG_AT: [f64; 2] = [0.0, 0.2];
const BIG_SIZE: f64 = 60.0;
const CAPTION_AT: [f64; 2] = [0.0, 1.1];
const COUNT_S: f64 = 2.5;
/// The tiny table (05_2): row `j`'s centre height, the squares' side, step
/// and first centre, the column numbers over the first row, the estimates'
/// column and size, the plot of each row's estimate (value 1 at
/// `PLOT_X0`, `PLOT_STEP` per unit) with its line at the count, and the
/// note under the last row.
const TABLE_TOP: f64 = 3.0;
const ROW_STEP: f64 = 0.55;
const SQ: f64 = 0.34;
const SQ_STEP: f64 = 0.44;
const SQ_X0: f64 = -6.2;
const SQ_R: f64 = 0.04;
const SEEN_O: f64 = 0.9;
const HEAD_DY: f64 = 0.45;
const HEAD_SIZE: f64 = 20.0;
const EST_X: f64 = -3.5;
const EST_SIZE: f64 = 24.0;
const PLOT_X0: f64 = -2.75;
const PLOT_STEP: f64 = 0.38;
const PLOT_R: f64 = 0.06;
const COUNT_LINE_W: f64 = 2.0;
const COUNT_LINE_O: f64 = 0.7;
const COUNT_LINE_PAD: f64 = 0.25;
const EACH_DY: f64 = 0.6;
const EACH_SIZE: f64 = 22.0;
/// The five boxes the worked pair is drawn on (05_1, 05_3), at
/// `DEMO_SCALE` of a game box, and their gap arcs and counts; the lines of
/// the right-hand panel under them.
const DEMO_Y: f64 = 2.4;
const DEMO_X0: f64 = 1.7;
const DEMO_STEP: f64 = 0.95;
const DEMO_SCALE: f64 = 0.7;
const DEMO_ARC_H: f64 = 0.22;
const DEMO_ARC_CLEAR: f64 = 0.02;
const DEMO_COUNT_DY: f64 = 0.2;
const DEMO_COUNT_SIZE: f64 = 24.0;
const RIGHT_X: f64 = 0.9;
const RIGHT_Y: [f64; 3] = [1.45, 1.0, 0.55];
/// The definition's lines (05_7), at the top right once the boxes are gone.
const TOP_Y: [f64; 3] = [3.1, 2.6, 2.1];
/// The balance (05_6): value 1 at `BAL_X0`, `BAL_STEP` per unit, the
/// estimates' axis over the largest numbers' axis; a stack's dots, the
/// first `STACK_UP` over its axis; the axis's ends past its first and last
/// values, its ticks; the fulcrum's half width and height, and the average
/// under it.
const BAL_X0: f64 = 1.3;
const BAL_STEP: f64 = 0.75;
const AXIS_Y: [f64; 2] = [-0.55, -2.45];
const STACK_R: f64 = 0.09;
const STACK_UP: f64 = 0.16;
const STACK_STEP: f64 = 0.22;
const AXIS_PAD: f64 = 0.25;
const AXIS_W: f64 = 2.0;
const AXIS_TICK: f64 = 0.06;
const FULCRUM: [f64; 2] = [0.14, 0.3];
const FULCRUM_GAP: f64 = 0.04;
const FULCRUM_LABEL_DY: f64 = 0.6;
const FLY_S: f64 = 1.2;
/// Each row landing, the inspection hold after the table (05_2), the beat
/// after the last line.
const ROWS_S: f64 = 1.6;
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;
const MILLION: u64 = 1_000_000;

/// What chapter 05 shows, from the facts: the big company's count, its
/// handful size and how many handfuls it has (and the whole millions the
/// words round down to); the tiny company, its pairs, their exact
/// estimates and how the table shows them, the two averages; the worked
/// pairs, with the high one's unseen boxes, gaps and estimate.
#[derive(Clone, Debug)]
struct Picture {
    company: u64,
    k: usize,
    handfuls: u64,
    millions: u64,
    boxes: u64,
    seen: usize,
    pairs: Vec<Vec<u64>>,
    estimates: Vec<Q>,
    shown: Vec<String>,
    average: Q,
    largest_average: Q,
    high: Vec<u64>,
    high_unseen: Vec<u64>,
    high_gaps: Vec<u64>,
    high_estimate: Q,
    high_shown: String,
    low: Vec<u64>,
    low_estimate: Q,
}

/// A fraction as the facts print it: "13/2", or "5" when whole.
fn fraction(s: &str) -> Result<Q, String> {
    let bad = || format!("{s:?} is not a fraction");
    match s.split_once('/') {
        Some((n, d)) => {
            let n: i128 = n.parse().map_err(|_| bad())?;
            let d: i128 = d.parse().map_err(|_| bad())?;
            if d == 0 {
                return Err(bad());
            }
            Ok(Q::new(n, d))
        }
        None => Ok(Q::int(s.parse().map_err(|_| bad())?)),
    }
}

/// How the table shows an exact estimate: a whole number as it is, any
/// other with its one decimal ("3.5") when that is exact; nothing else has
/// an exact short form.
fn shown_of(e: Q) -> Result<String, String> {
    match e.whole() {
        Some(w) => Ok(w.to_string()),
        None => e.decimal1(),
    }
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    let fractions = |key: &str| -> Result<Vec<Q>, String> { printed(&f, key)?.split(',').map(fraction).collect() };
    checked(Picture {
        company: whole(&f, "G-F5.company")?,
        k: whole(&f, "G-F1.handful")? as usize,
        handfuls: whole(&f, "G-F5.handfuls")?,
        millions: whole(&f, "G-F5.millions")?,
        boxes: whole(&f, "G-F5.boxes")?,
        seen: whole(&f, "G-F5.seen")? as usize,
        pairs: handfuls(&f, "G-F5.pair_list")?,
        estimates: fractions("G-F5.estimates")?,
        shown: printed(&f, "G-F5.estimates_shown")?
            .split(',')
            .map(str::to_owned)
            .collect(),
        average: fraction(&printed(&f, "G-F5.average")?)?,
        largest_average: fraction(&printed(&f, "G-F5.largest_average")?)?,
        high: list(&f, "G-F5.example_high_pair")?,
        high_unseen: list(&f, "G-F5.example_high_unseen")?,
        high_gaps: list(&f, "G-F5.example_high_gaps")?,
        high_estimate: fraction(&printed(&f, "G-F5.example_high")?)?,
        high_shown: printed(&f, "G-F5.example_high_decimal")?,
        low: list(&f, "G-F5.example_low_pair")?,
        low_estimate: fraction(&printed(&f, "G-F5.example_low")?)?,
    })
}

/// The gap recipe for a sorted pair (or any handful of `seen`): its largest
/// plus the average of the gaps below it, from the unseen numbers counted.
fn recipe(hand: &[u64], seen: usize) -> Q {
    let m = hand.last().copied().unwrap_or(0);
    Q::int(m as i128) + Q::new(unseen_below(hand) as i128, seen as i128)
}

/// The average of `values`.
fn mean(values: &[Q]) -> Q {
    values.iter().fold(Q::ZERO, |s, &x| s + x) / Q::int(values.len().max(1) as i128)
}

/// The picture only if the scene can draw it truthfully: the big company's
/// handfuls are over the millions the words say and under one more; the
/// table has every pair from the tiny company once, in order; each row's
/// estimate is its largest plus its average gap, exactly, and is shown
/// exactly; the estimates average exactly the count, the largest numbers
/// alone less; the high pair is a row, its unseen boxes the numbers below
/// its largest it lacks, in as many non-empty gaps as it has numbers; the
/// low pair is the first row, with no gaps, its estimate its largest.
fn checked(pic: Picture) -> Result<Picture, String> {
    if pic.handfuls <= pic.millions * MILLION
        || pic.handfuls >= (pic.millions + 1) * MILLION
        || choose(pic.company, pic.k as u64) != u128::from(pic.handfuls)
    {
        return Err(format!(
            "the {} handfuls of {} from {} must be all of them, over {} million and under one more",
            pic.handfuls, pic.k, pic.company, pic.millions
        ));
    }
    if pic.pairs.len() as u128 != choose(pic.boxes, pic.seen as u64)
        || !pic.pairs.iter().all(|p| is_handful(p, pic.seen, pic.boxes))
        || pic.pairs.windows(2).any(|w| w[0] >= w[1])
    {
        return Err(format!(
            "the pairs {:?} must be every pair from {} boxes once, in order",
            pic.pairs, pic.boxes
        ));
    }
    if pic.estimates.len() != pic.pairs.len() || pic.shown.len() != pic.pairs.len() {
        return Err(format!(
            "{} estimates shown as {:?} for {} pairs",
            pic.estimates.len(),
            pic.shown,
            pic.pairs.len()
        ));
    }
    for (p, (&e, s)) in pic.pairs.iter().zip(pic.estimates.iter().zip(&pic.shown)) {
        if e != recipe(p, pic.seen) || shown_of(e)? != *s {
            return Err(format!(
                "the estimate {e} of {p:?}, shown as {s:?}, must be its largest plus the average gap, shown exactly"
            ));
        }
    }
    let largest: Vec<Q> = pic
        .pairs
        .iter()
        .map(|p| Q::int(p.last().copied().unwrap_or(0) as i128))
        .collect();
    if mean(&pic.estimates) != pic.average
        || pic.average != Q::int(pic.boxes as i128)
        || mean(&largest) != pic.largest_average
        || pic.largest_average >= pic.average
    {
        return Err(format!(
            "the estimates must average exactly the count {} and the largest alone, {}, less",
            pic.average, pic.largest_average
        ));
    }
    let m = pic.high.last().copied().unwrap_or(0);
    let unseen: Vec<u64> = (1..m).filter(|n| !pic.high.contains(n)).collect();
    let sizes: Vec<u64> = gap_spans(&pic.high).iter().map(|&(a, b)| b - a + 1).collect();
    if !pic.pairs.contains(&pic.high)
        || pic.high_unseen != unseen
        || pic.high_gaps != sizes
        || pic.high_gaps.len() != pic.seen
        || pic.high_estimate != recipe(&pic.high, pic.seen)
        || shown_of(pic.high_estimate)? != pic.high_shown
    {
        return Err(format!(
            "the worked pair {:?}: unseen {:?} in the gaps {:?}, estimate {} shown as {}",
            pic.high, pic.high_unseen, pic.high_gaps, pic.high_estimate, pic.high_shown
        ));
    }
    let low_m = pic.low.last().copied().unwrap_or(0);
    if pic.pairs.first() != Some(&pic.low) || unseen_below(&pic.low) != 0 || pic.low_estimate != Q::int(low_m as i128) {
        return Err(format!(
            "the low pair {:?} must be the first row, with no gaps, its estimate {} its largest",
            pic.low, pic.low_estimate
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------ positions

/// Row `j`'s centre height in the table.
fn table_y(j: usize) -> f64 {
    TABLE_TOP - ROW_STEP * j as f64
}

/// Column `n`'s centre (box `n` of the tiny company).
fn sq_x(n: u64) -> f64 {
    SQ_X0 + SQ_STEP * (n as f64 - 1.0)
}

/// Where value `e` sits on a row's plot.
fn plot_x(e: f64) -> f64 {
    PLOT_X0 + PLOT_STEP * (e - 1.0)
}

/// Where value `e` sits on the balance.
fn bal_x(e: f64) -> f64 {
    BAL_X0 + BAL_STEP * (e - 1.0)
}

/// Box `n`'s centre among the five demo boxes.
fn demo_x(n: u64) -> f64 {
    DEMO_X0 + DEMO_STEP * (n as f64 - 1.0)
}

/// Each value's place in its stack: how many values before it are equal.
fn levels<T: PartialEq>(values: &[T]) -> Vec<usize> {
    values
        .iter()
        .enumerate()
        .map(|(i, x)| values[..i].iter().filter(|y| *y == x).count())
        .collect()
}

/// Where the `level`th dot of value `e`'s stack sits over axis `axis`.
fn stack_spot(e: f64, level: usize, axis: usize) -> Vec3 {
    v(bal_x(e), AXIS_Y[axis] + STACK_UP + STACK_STEP * level as f64)
}

/// The balance's last whole value: the largest estimate, rounded up.
fn axis_top(values: &[f64]) -> f64 {
    values.iter().fold(1.0_f64, |a, &b| a.max(b)).ceil()
}

// ------------------------------------------------------------- builders

/// The five demo boxes: their bodies, and each box's number three ways,
/// dim, gold (seen) and blue (unseen), so a box changes role by one number
/// giving way to another. Detached.
struct Demo {
    bodies: Vec<Mob>,
    dim: Vec<Mob>,
    gold: Vec<Mob>,
    blue: Vec<Mob>,
}

impl Demo {
    fn build(kit: &Kit, stage: &mut Stage<'_>, boxes: u64) -> fmn::Result<Demo> {
        let mut demo = Demo {
            bodies: Vec::new(),
            dim: Vec::new(),
            gold: Vec::new(),
            blue: Vec::new(),
        };
        for n in 1..=boxes {
            let spot = v(demo_x(n), DEMO_Y);
            let [body, _lid] = game_box(stage, spot)?;
            stage.scale(body, DEMO_SCALE);
            demo.bodies.push(body);
            for (labels, color) in [
                (&mut demo.dim, palette::outline()),
                (&mut demo.gold, palette::seen()),
                (&mut demo.blue, palette::unseen()),
            ] {
                let label = kit.text(stage, &n.to_string(), NUMBER_SIZE, color, spot)?;
                stage.scale(label, DEMO_SCALE);
                labels.push(label);
            }
        }
        Ok(demo)
    }

    /// The numbers of `which` from one of the three sets.
    fn of(labels: &[Mob], which: &[u64]) -> Vec<Mob> {
        which.iter().map(|&n| labels[(n - 1) as usize]).collect()
    }
}

/// The arcs over a pair's gaps on the demo boxes, from the left edge of a
/// gap's first box to the right edge of its last, and each gap's count
/// over its arc. Detached.
fn demo_gaps(kit: &Kit, stage: &mut Stage<'_>, pair: &[u64]) -> fmn::Result<Vec<Mob>> {
    let (half_w, half_h) = (DEMO_SCALE * BOX_W / 2.0, DEMO_SCALE * BOX_H / 2.0);
    let y = DEMO_Y + half_h + DEMO_ARC_CLEAR;
    let mut out = Vec::new();
    for (a, b) in gap_spans(pair) {
        let (x0, x1) = (demo_x(a) - half_w, demo_x(b) + half_w);
        out.push(stroke(
            stage,
            arc_points(x0, x1, y, DEMO_ARC_H),
            line_style(palette::unseen(), ARC_W, 1.0),
        ));
        out.push(kit.text(
            stage,
            &(b - a + 1).to_string(),
            DEMO_COUNT_SIZE,
            palette::unseen(),
            v((x0 + x1) / 2.0, y + ARC_LIFT + DEMO_ARC_H + DEMO_COUNT_DY),
        )?);
    }
    Ok(out)
}

/// The tiny table: the column numbers, each row's five squares (the pair's
/// two gold), the estimates' column head, each row's estimate, each row's
/// plot dot, the line at the count and its label. Detached.
struct Table {
    head: Vec<Mob>,
    rows: Vec<Vec<Mob>>,
    estimate_head: Mob,
    estimates: Vec<Mob>,
    dots: Vec<Mob>,
    count_line: Mob,
    count_label: Mob,
}

impl Table {
    fn build(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, values: &[f64]) -> fmn::Result<Table> {
        let head_y = TABLE_TOP + HEAD_DY;
        let mut head = Vec::new();
        for n in 1..=pic.boxes {
            head.push(kit.text(stage, &n.to_string(), HEAD_SIZE, Palette::dim(), v(sq_x(n), head_y))?);
        }
        let mut rows = Vec::new();
        for (j, pair) in pic.pairs.iter().enumerate() {
            let mut row = Vec::new();
            for n in 1..=pic.boxes {
                let style = if pair.contains(&n) {
                    flat(palette::seen(), SEEN_O)
                } else {
                    flat(palette::outline(), BODY_O)
                };
                let sq = Rectangle::new()
                    .width(SQ)
                    .height(SQ)
                    .corner_radius(SQ_R)
                    .style(style)
                    .build()?
                    .moved_to(v(sq_x(n), table_y(j)));
                row.push(stage.arena_mut().add(sq));
            }
            rows.push(row);
        }
        let estimate_head = kit.text(stage, ESTIMATE_HEAD, HEAD_SIZE, Palette::dim(), v(EST_X, head_y))?;
        let mut estimates = Vec::new();
        let mut dots = Vec::new();
        for (j, (s, &e)) in pic.shown.iter().zip(values).enumerate() {
            estimates.push(kit.text(stage, s, EST_SIZE, palette::estimate(), v(EST_X, table_y(j)))?);
            let d = Dot::new()
                .point(v(plot_x(e), table_y(j)))
                .radius(PLOT_R)
                .style(flat(palette::estimate(), 1.0))
                .build();
            dots.push(stage.arena_mut().add(d));
        }
        let x = plot_x(pic.boxes as f64);
        let last = pic.pairs.len().saturating_sub(1);
        let count_line = stroke(
            stage,
            vec![v(x, table_y(0) + COUNT_LINE_PAD), v(x, table_y(last) - COUNT_LINE_PAD)],
            line_style(Palette::dim(), COUNT_LINE_W, COUNT_LINE_O),
        );
        let count_label = kit.text(
            stage,
            &format!("count = {}", pic.boxes),
            HEAD_SIZE,
            Palette::dim(),
            v(x, head_y),
        )?;
        Ok(Table {
            head,
            rows,
            estimate_head,
            estimates,
            dots,
            count_line,
            count_label,
        })
    }
}

/// One axis of the balance: its stroke and ticks, its fulcrum and the
/// average under it, and its dots, each built where it starts with where
/// it lands. Detached.
struct Axis {
    lines: Vec<Mob>,
    fulcrum: [Mob; 2],
    dots: Vec<(Mob, Vec3)>,
}

/// What one axis of the balance stacks: each value, where its dot starts,
/// and the average its fulcrum marks, as the table shows it.
struct Stacks<'a> {
    values: &'a [f64],
    from: &'a [Vec3],
    average: &'a str,
}

/// Axis `axis` of the balance, from value 1 to `top`, with a tick at each
/// whole value: the dots of `stacks` in `color`, each starting where it
/// starts and landing in its stack; the fulcrum at the average, labelled
/// with it.
fn balance_axis(
    kit: &Kit,
    stage: &mut Stage<'_>,
    axis: usize,
    top: f64,
    stacks: Stacks<'_>,
    color: Srgb,
) -> fmn::Result<Axis> {
    let y = AXIS_Y[axis];
    let mut lines = vec![stroke(
        stage,
        vec![v(bal_x(1.0) - AXIS_PAD, y), v(bal_x(top) + AXIS_PAD, y)],
        line_style(Palette::dim(), AXIS_W, 1.0),
    )];
    for n in 1..=(top as u64) {
        let x = bal_x(n as f64);
        lines.push(stroke(
            stage,
            vec![v(x, y - AXIS_TICK), v(x, y + AXIS_TICK)],
            line_style(Palette::dim(), AXIS_W, 1.0),
        ));
    }
    let x = bal_x(value(stacks.average).map_err(fail)?);
    let tip = y - FULCRUM_GAP;
    let triangle = stroke(
        stage,
        vec![
            v(x - FULCRUM[0], tip - FULCRUM[1]),
            v(x + FULCRUM[0], tip - FULCRUM[1]),
            v(x, tip),
            v(x - FULCRUM[0], tip - FULCRUM[1]),
        ],
        line_style(color, AXIS_W, 1.0),
    );
    let label = kit.text(
        stage,
        &format!("average = {}", stacks.average),
        PANEL_SIZE,
        color,
        v(x, y - FULCRUM_LABEL_DY),
    )?;
    let mut dots = Vec::new();
    for ((&e, &start), level) in stacks.values.iter().zip(stacks.from).zip(levels(stacks.values)) {
        let d = Dot::new().point(start).radius(STACK_R).style(flat(color, 1.0)).build();
        dots.push((stage.arena_mut().add(d), stack_spot(e, level, axis)));
    }
    Ok(Axis {
        lines,
        fulcrum: [triangle, label],
        dots,
    })
}

/// Bring `axis`'s dots in where they start, then drop them into their
/// stacks.
fn land(stage: &mut Stage<'_>, axis: &Axis) -> fmn::Result<()> {
    for &(d, _) in &axis.dots {
        stage.add_to_scene(d)?;
    }
    move_all(stage, &axis.dots, 1.0, FLY_S)
}

impl SceneConstruct for EveryHandful {
    fn name(&self) -> &str {
        "05_every_handful"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let values: Vec<f64> = pic
            .shown
            .iter()
            .map(|s| value(s))
            .collect::<Result<_, _>>()
            .map_err(fail)?;
        let row_of = |pair: &[u64]| pic.pairs.iter().position(|p| p == pair).unwrap_or(0);
        let (high_row, low_row) = (row_of(pic.high.as_slice()), row_of(pic.low.as_slice()));
        let last_row = pic.pairs.len().saturating_sub(1);

        // 05_1: every handful of five from seventy-one, counted up; then a
        // company of five boxes, two of them seen
        let line = say(&kit, stage, "05_1")?;
        let caption = kit.text(
            stage,
            &format!("handfuls of {} from {}", pic.k, pic.company),
            PANEL_SIZE,
            Palette::dim(),
            at(CAPTION_AT),
        )?;
        fade_in_all(stage, &[caption], 0.4)?;
        let counted = stage.add_value_tracker(0.0);
        let live = kit.live_count(stage, counted, BIG_SIZE, Palette::ink(), at(BIG_AT))?;
        line.wait_for(stage, "over thirteen million")?;
        stage.play(tween(counted, pic.handfuls as f64, COUNT_S)?)?;
        line.wait_for(stage, "So let's shrink the company")?;
        fade_out_all(stage, &[caption, live], 0.5)?;
        line.wait_for(stage, "just five boxes")?;
        let demo = Demo::build(&kit, stage, pic.boxes)?;
        let mut boxes = demo.bodies.clone();
        boxes.extend(&demo.dim);
        fade_in_lagged(stage, &boxes, 1.0, 0.2)?;
        line.wait_for(stage, "you see two")?;
        replace(
            stage,
            &Demo::of(&demo.dim, &pic.high),
            &Demo::of(&demo.gold, &pic.high),
            0.5,
        )?;

        // 05_2: the ten pairs, row by row; a halo round the first and the
        // last; every pair equally likely; held to be read
        let line = say(&kit, stage, "05_2")?;
        let table = Table::build(&kit, stage, &pic, &values)?;
        line.wait_for(stage, "ten possible pairs")?;
        fade_in_all(stage, &table.head, 0.3)?;
        let squares: Vec<Mob> = table.rows.iter().flatten().copied().collect();
        fade_in_lagged(stage, &squares, ROWS_S, 0.1)?;
        line.wait_for(stage, "from one and two")?;
        glow(stage, &table.rows[0], palette::seen())?;
        line.wait_for(stage, "up to four and five")?;
        glow(stage, &table.rows[last_row], palette::seen())?;
        line.wait_for(stage, "each pair is equally likely")?;
        let each = kit.text_left(
            stage,
            &format!("each pair: 1 in {}", pic.pairs.len()),
            EACH_SIZE,
            Palette::dim(),
            v(SQ_X0 - SQ / 2.0, table_y(last_row) - EACH_DY),
        )?;
        fade_in_all(stage, &[each], 0.4)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 05_3: the pair two and five, its row's halo; its unseen boxes
        // turn blue; its two gaps and their total; the recipe for a pair
        let line = say(&kit, stage, "05_3")?;
        line.wait_for(stage, "Take the pair two and five")?;
        glow(stage, &table.rows[high_row], palette::seen())?;
        line.wait_for(stage, "the unseen boxes are")?;
        replace(
            stage,
            &Demo::of(&demo.dim, &pic.high_unseen),
            &Demo::of(&demo.blue, &pic.high_unseen),
            0.5,
        )?;
        line.wait_for(stage, "That's three boxes")?;
        let arcs = demo_gaps(&kit, stage, &pic.high)?;
        fade_in_lagged(stage, &arcs, 0.8, 0.3)?;
        let high_m = pic.high.last().copied().unwrap_or(0);
        let high_total = unseen_below(&pic.high);
        let (text, colors) = gaps_line(high_m, pic.seen, high_total, false);
        let total = words_left(&kit, stage, &text, &colors, v(RIGHT_X, RIGHT_Y[0]))?;
        fade_in_all(stage, &[total], 0.5)?;
        line.wait_for(stage, "So with a pair")?;
        let alone = words_left(
            &kit,
            stage,
            &formula(pic.seen),
            &formula_colors(),
            v(RIGHT_X, RIGHT_Y[1]),
        )?;
        fade_in_all(stage, &[alone], 0.5)?;

        // 05_4: six and a half, piece by piece, then into its row; the
        // pair one and two, its row's halo, and its estimate
        let line = say(&kit, stage, "05_4")?;
        line.wait_for(stage, "five plus three halves")?;
        let sum = sum_texts(
            &kit,
            stage,
            &instance(high_m, high_total, pic.seen, &pic.high_shown),
            RIGHT_X,
            RIGHT_Y[2],
        )?;
        fade_in_lagged(stage, &sum, 1.0, 0.3)?;
        line.wait_for(stage, "six and a half")?;
        fade_in_all(stage, &[table.estimate_head, table.estimates[high_row]], 0.4)?;
        line.wait_for(stage, "For one and two")?;
        glow(stage, &table.rows[low_row], palette::seen())?;
        line.wait_for(stage, "just two")?;
        fade_in_all(stage, &[table.estimates[low_row]], 0.4)?;

        // 05_5: the worked example goes; the other estimates cascade down;
        // the line at the count and each row's dot; the prediction
        let line = say(&kit, stage, "05_5")?;
        let mut worked = demo.bodies.clone();
        worked.extend(Demo::of(&demo.gold, &pic.high));
        worked.extend(Demo::of(&demo.blue, &pic.high_unseen));
        worked.extend(&arcs);
        worked.extend([total, alone]);
        worked.extend(&sum);
        fade_out_all(stage, &worked, 0.5)?;
        line.wait_for(stage, "all ten estimates")?;
        let rest: Vec<Mob> = (0..pic.pairs.len())
            .filter(|&j| j != high_row && j != low_row)
            .map(|j| table.estimates[j])
            .collect();
        fade_in_lagged(stage, &rest, 1.4, 0.3)?;
        line.wait_for(stage, "Some are too low")?;
        fade_in_all(stage, &[table.count_line, table.count_label], 0.4)?;
        fade_in_lagged(stage, &table.dots, 1.2, 0.2)?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 05_6: the estimates drop from their rows onto the balance, which
        // settles at five; then the largest numbers alone, from each row's
        // larger gold square, which settle at four
        let line = say(&kit, stage, "05_6")?;
        let top = axis_top(&values);
        let starts: Vec<Vec3> = values
            .iter()
            .enumerate()
            .map(|(j, &e)| v(plot_x(e), table_y(j)))
            .collect();
        let average = shown_of(pic.average).map_err(fail)?;
        let guesses = balance_axis(
            &kit,
            stage,
            0,
            top,
            Stacks {
                values: &values,
                from: &starts,
                average: &average,
            },
            palette::estimate(),
        )?;
        replace(stage, &[q], &guesses.lines, 0.4)?;
        land(stage, &guesses)?;
        fade_in_all(stage, &guesses.fulcrum, 0.4)?;
        line.wait_for(stage, "The largest number on its own")?;
        let largest: Vec<f64> = pic
            .pairs
            .iter()
            .map(|p| p.last().copied().unwrap_or(0) as f64)
            .collect();
        let squares_at: Vec<Vec3> = pic
            .pairs
            .iter()
            .enumerate()
            .map(|(j, p)| v(sq_x(p.last().copied().unwrap_or(0)), table_y(j)))
            .collect();
        let alone_average = shown_of(pic.largest_average).map_err(fail)?;
        let tops = balance_axis(
            &kit,
            stage,
            1,
            top,
            Stacks {
                values: &largest,
                from: &squares_at,
                average: &alone_average,
            },
            palette::seen(),
        )?;
        fade_in_all(stage, &tops.lines, 0.3)?;
        land(stage, &tops)?;
        fade_in_all(stage, &tops.fulcrum, 0.4)?;

        // 05_7: what unbiased means; the one count checked, every other
        // count still open; a halo round the open question
        let line = say(&kit, stage, "05_7")?;
        line.wait_for(stage, "A recipe is unbiased")?;
        let named = [("average estimate".to_owned(), palette::estimate())];
        let unbiased = words_left(&kit, stage, UNBIASED, &named, v(RIGHT_X, TOP_Y[0]))?;
        fade_in_all(stage, &[unbiased], 0.5)?;
        line.wait_for(stage, "whatever that count is")?;
        let checked_here = format!("count {}: average {}", pic.boxes, average);
        let here_colors = roles(&checked_here, &[(format!("average {average}"), palette::estimate())]);
        let here = words_left(&kit, stage, &checked_here, &here_colors, v(RIGHT_X, TOP_Y[1]))?;
        fade_in_all(stage, &[here], 0.5)?;
        line.wait_for(stage, "isn't a proof")?;
        let open = [(OTHER.to_owned(), Palette::dim())];
        let other = words_left(&kit, stage, OTHER, &open, v(RIGHT_X, TOP_Y[2]))?;
        fade_in_all(stage, &[other], 0.5)?;
        line.wait_for(stage, "So why does adding")?;
        glow(stage, &[other], Palette::ink())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::Script;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{THINK_R, all_clear};
    use crate::chapters::friends_line::est_width;
    use crate::chapters::what_we_assume::SUM_GAP;
    use crate::facts::{K, TINY, TINY_SEEN};
    use crate::model::{KNOWN, all_handfuls, doubling_recipe, gap_recipe};

    const NEAR: f64 = 1e-9;
    /// A line's half height at the panel's size, at the heads' size, and
    /// the space kept between things that must not touch.
    const HALF_LINE: f64 = 0.16;
    const HALF_HEAD: f64 = 0.12;
    const CLEAR: f64 = 0.1;

    fn good() -> Picture {
        picture().unwrap()
    }

    fn values(pic: &Picture) -> Vec<f64> {
        pic.shown.iter().map(|s| value(s).unwrap()).collect()
    }

    /// The scene draws the facts: the big company's handfuls are all the
    /// handfuls of five from seventy-one; the table's pairs and estimates
    /// are the model's, in its order; each shown value is its estimate
    /// exactly; the panel's lines read as the words say them.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!((pic.company, pic.k as u64), (KNOWN, K));
        assert_eq!(u128::from(pic.handfuls), choose(KNOWN, K));
        assert_eq!((pic.boxes, pic.seen), (TINY, TINY_SEEN));
        assert_eq!(pic.pairs, all_handfuls(TINY, TINY_SEEN));
        let model: Vec<Q> = pic.pairs.iter().map(|p| gap_recipe(p)).collect();
        assert_eq!(pic.estimates, model);
        for (s, &e) in pic.shown.iter().zip(&pic.estimates) {
            let tenths = (value(s).unwrap() * 10.0).round() as i128;
            assert_eq!(Q::new(tenths, 10), e, "{s} is {e} exactly");
        }
        assert_eq!(pic.high_shown, "6.5");
        assert_eq!(gaps_line(5, pic.seen, 3, false).0, "gaps: 5 − 2 = 3");
        assert_eq!(formula(pic.seen), "estimate = largest + (largest − 2) ÷ 2");
        let sum: Vec<String> = instance(5, 3, pic.seen, &pic.high_shown)
            .into_iter()
            .map(|(s, _)| s)
            .collect();
        assert_eq!(sum.join(" "), "5 + 3 ÷ 2 = 6.5");
        assert_eq!(shown_of(pic.average).unwrap(), "5");
        assert_eq!(shown_of(pic.largest_average).unwrap(), "4");
    }

    /// The negative controls: each picture the words or the scene would
    /// contradict is refused, one wrong fact at a time, for its reason.
    #[test]
    fn a_picture_the_words_contradict_is_refused() {
        let pic = good();
        let refused = |p: Picture, why: &str| {
            let e = checked(p).expect_err(why);
            assert!(e.contains(why), "refused for {e:?}, not {why:?}");
        };
        assert!(checked(pic.clone()).is_ok());
        refused(
            Picture {
                millions: pic.millions + 1,
                ..pic.clone()
            },
            "over",
        );
        refused(
            Picture {
                handfuls: pic.millions * MILLION,
                ..pic.clone()
            },
            "over",
        );
        refused(
            Picture {
                company: pic.company - 1,
                ..pic.clone()
            },
            "all of them",
        );
        refused(
            Picture {
                pairs: pic.pairs[1..].to_vec(),
                ..pic.clone()
            },
            "every pair",
        );
        let mut swapped = pic.pairs.clone();
        swapped.swap(0, 1);
        refused(
            Picture {
                pairs: swapped,
                ..pic.clone()
            },
            "every pair",
        );
        // the doubling recipe (chapter 10) also averages five here, but it
        // is not the gap recipe the table shows
        let doubled: Vec<Q> = pic.pairs.iter().map(|p| doubling_recipe(p)).collect();
        assert_eq!(mean(&doubled), pic.average);
        let doubled_shown: Vec<String> = doubled.iter().map(|&e| shown_of(e).unwrap()).collect();
        refused(
            Picture {
                estimates: doubled,
                shown: doubled_shown,
                ..pic.clone()
            },
            "average gap",
        );
        let mut rounded = pic.shown.clone();
        rounded[1] = "4".to_owned();
        refused(
            Picture {
                shown: rounded,
                ..pic.clone()
            },
            "shown exactly",
        );
        refused(
            Picture {
                average: Q::new(49, 10),
                ..pic.clone()
            },
            "exactly the count",
        );
        refused(
            Picture {
                largest_average: pic.average,
                ..pic.clone()
            },
            "exactly the count",
        );
        refused(
            Picture {
                high_unseen: vec![1, 3],
                ..pic.clone()
            },
            "worked pair",
        );
        refused(
            Picture {
                high_gaps: vec![3],
                ..pic.clone()
            },
            "worked pair",
        );
        refused(
            Picture {
                high_shown: "7".to_owned(),
                ..pic.clone()
            },
            "worked pair",
        );
        refused(
            Picture {
                low: pic.pairs[1].clone(),
                low_estimate: Q::new(7, 2),
                ..pic
            },
            "low pair",
        );
    }

    /// The facts' printing read back: a fraction, a whole number, a sign;
    /// a zero denominator and words are refused. An estimate is shown
    /// exactly or not at all: a third has no one-decimal form.
    #[test]
    fn fractions_are_read_and_shown_exactly() {
        assert_eq!(fraction("13/2"), Ok(Q::new(13, 2)));
        assert_eq!(fraction("5"), Ok(Q::int(5)));
        assert_eq!(fraction("-3/6"), Ok(Q::new(-1, 2)));
        for bad in ["1/0", "x", "", "1/2/3"] {
            assert!(fraction(bad).is_err(), "{bad:?}");
        }
        assert_eq!(shown_of(Q::int(2)).unwrap(), "2");
        assert_eq!(shown_of(Q::new(7, 2)).unwrap(), "3.5");
        assert!(shown_of(Q::new(1, 3)).is_err());
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            (
                "05_1",
                &[
                    "over thirteen million",
                    "So let's shrink the company",
                    "just five boxes",
                    "you see two",
                ],
            ),
            (
                "05_2",
                &[
                    "ten possible pairs",
                    "from one and two",
                    "up to four and five",
                    "each pair is equally likely",
                ],
            ),
            (
                "05_3",
                &[
                    "Take the pair two and five",
                    "the unseen boxes are",
                    "That's three boxes",
                    "So with a pair",
                ],
            ),
            (
                "05_4",
                &[
                    "five plus three halves",
                    "six and a half",
                    "For one and two",
                    "just two",
                ],
            ),
            ("05_5", &["all ten estimates", "Some are too low"]),
            ("05_6", &["The largest number on its own"]),
            (
                "05_7",
                &[
                    "A recipe is unbiased",
                    "whatever that count is",
                    "isn't a proof",
                    "So why does adding",
                ],
            ),
        ];
        for &(id, phrases) in waits {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            let mut last = 0;
            for p in phrases {
                assert_eq!(text.matches(p).count(), 1, "{id}: {p:?}");
                let i = text.find(p).unwrap();
                assert!(i >= last, "{id}: {p:?} comes after the phrase before it");
                last = i;
            }
        }
    }

    /// The balance settles where the words say: the estimates' moments
    /// about five cancel exactly, and the largest numbers' about four. The
    /// negative control: about four, the estimates tip.
    #[test]
    fn the_balance_settles_at_the_average() {
        fn moment(values: &[Q], at: Q) -> Q {
            values.iter().fold(Q::ZERO, |s, &x| s + (x - at))
        }
        let pic = good();
        assert_eq!(moment(&pic.estimates, pic.average), Q::ZERO);
        let largest: Vec<Q> = pic.pairs.iter().map(|p| Q::int(p[1] as i128)).collect();
        assert_eq!(moment(&largest, pic.largest_average), Q::ZERO);
        assert_ne!(moment(&pic.estimates, pic.largest_average), Q::ZERO);
        // the stacks: each value's dots one over another, from the axis up
        assert_eq!(levels(&[2, 5, 5, 3, 5]), [0, 0, 1, 0, 2]);
        let lv = levels(&pic.shown);
        let high = pic.shown.iter().filter(|s| **s == pic.high_shown).count();
        assert_eq!(lv.iter().max().copied(), Some(high - 1));
    }

    /// The table: the column numbers and the first row under the frame's
    /// top, the note under the last row over its foot; squares apart; the
    /// estimates' column between the squares and the plot; the plot's
    /// dots apart, left of the balance and the countdown; the line at the
    /// count over every row.
    #[test]
    fn the_table_fits_the_frame() {
        let pic = good();
        let vals = values(&pic);
        let last = pic.pairs.len() - 1;
        const { assert!(TABLE_TOP + HEAD_DY + HALF_HEAD < FRAME_Y_RADIUS - 0.15) };
        assert!(table_y(last) - EACH_DY - HALF_HEAD > -FRAME_Y_RADIUS + 0.15);
        const { assert!(SQ + CLEAR / 2.0 < SQ_STEP && SQ + CLEAR < ROW_STEP) };
        const { assert!(SQ_X0 - SQ / 2.0 > -(FRAME_X_RADIUS - 0.25)) };
        let widest = pic.shown.iter().map(|s| est_width(s, EST_SIZE)).fold(0.0, f64::max);
        let head_w = est_width(ESTIMATE_HEAD, HEAD_SIZE);
        assert!(EST_X - widest.max(head_w) / 2.0 > sq_x(pic.boxes) + SQ / 2.0 + CLEAR);
        let lowest = vals.iter().copied().fold(f64::INFINITY, f64::min);
        let highest = vals.iter().copied().fold(0.0, f64::max);
        assert!(plot_x(lowest) - PLOT_R > EST_X + widest.max(head_w) / 2.0 + CLEAR);
        // distinct values sit apart on a row's plot
        const { assert!(PLOT_STEP / 2.0 > 2.0 * PLOT_R + CLEAR / 2.0) };
        assert!(plot_x(highest) + PLOT_R < bal_x(1.0) - AXIS_PAD - 1.0);
        assert!(plot_x(highest) + PLOT_R < THINK_AT[0] - THINK_R - 1.0);
        // the count line's label over it, clear of the estimates' head
        let label_w = est_width(&format!("count = {}", pic.boxes), HEAD_SIZE);
        let count_x = plot_x(pic.boxes as f64);
        assert!(count_x - label_w / 2.0 > EST_X + head_w / 2.0 + CLEAR);
        assert!(lowest < pic.boxes as f64 && pic.boxes as f64 <= highest);
        const { assert!(TABLE_TOP + COUNT_LINE_PAD < TABLE_TOP + HEAD_DY - HALF_HEAD) };
    }

    /// The five demo boxes keep apart; their arcs' counts stay under the
    /// frame's top; the right-hand panel's lines fit under the boxes and
    /// in the title-safe area; the live count and its caption fit.
    #[test]
    fn the_worked_pair_fits_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let boxes: Vec<(Vec3, f64)> = (1..=pic.boxes).map(|n| (v(demo_x(n), DEMO_Y), DEMO_SCALE)).collect();
        assert!(all_clear(&boxes));
        assert!(demo_x(pic.boxes) + DEMO_SCALE * BOX_W / 2.0 < safe_x);
        let half_h = DEMO_SCALE * BOX_H / 2.0;
        let count_y = DEMO_Y + half_h + DEMO_ARC_CLEAR + ARC_LIFT + DEMO_ARC_H + DEMO_COUNT_DY;
        assert!(count_y + HALF_LINE < FRAME_Y_RADIUS - 0.15);
        assert!(RIGHT_Y[0] + HALF_LINE < DEMO_Y - half_h - CLEAR);
        for w in RIGHT_Y.windows(2) {
            assert!(w[0] - w[1] > 2.0 * HALF_LINE + CLEAR / 2.0);
        }
        let sum = instance(5, 3, pic.seen, &pic.high_shown);
        let sum_w: f64 =
            sum.iter().map(|(s, _)| est_width(s, PANEL_SIZE)).sum::<f64>() + SUM_GAP * (sum.len() - 1) as f64;
        for w in [
            est_width(&gaps_line(5, pic.seen, 3, false).0, PANEL_SIZE),
            est_width(&formula(pic.seen), PANEL_SIZE),
            sum_w,
            est_width(UNBIASED, PANEL_SIZE),
            est_width(OTHER, PANEL_SIZE),
        ] {
            assert!(RIGHT_X + w < safe_x, "{w}");
        }
        assert!(RIGHT_X > plot_x(axis_top(&values(&pic))) + CLEAR);
        // the live count's widest form, its digits and thousands commas
        let digits = pic.handfuls.to_string().len();
        let big = est_width(&"0".repeat(digits + (digits - 1) / 3), BIG_SIZE);
        assert!(big / 2.0 < safe_x);
        const {
            assert!(
                CAPTION_AT[1] - HALF_LINE > BIG_AT[1] + 0.4,
                "the caption over the count"
            )
        };
    }

    /// The balance: both axes and their fulcrums' labels on the frame,
    /// right of the table; the estimates' tallest stack under the
    /// definition's last line; the estimates' average over the largest
    /// numbers' tallest stack; every stacked dot apart from the next.
    #[test]
    fn the_balance_fits_the_frame() {
        let pic = good();
        let vals = values(&pic);
        let top = axis_top(&vals);
        let safe_x = FRAME_X_RADIUS - 0.25;
        assert!((top - 7.0).abs() < NEAR, "6.5 rounds up to 7");
        assert!(bal_x(top) + AXIS_PAD < safe_x);
        let label_w = est_width(&format!("average = {}", shown_of(pic.average).unwrap()), PANEL_SIZE);
        assert!(bal_x(value(&shown_of(pic.average).unwrap()).unwrap()) + label_w / 2.0 < safe_x);
        fn stack_top(vals: &[f64], axis: usize) -> f64 {
            let tallest = levels(vals).into_iter().max().unwrap_or(0);
            stack_spot(1.0, tallest, axis)[1] + STACK_R
        }
        assert!(stack_top(&vals, 0) < TOP_Y[2] - HALF_LINE - CLEAR);
        let largest: Vec<f64> = pic.pairs.iter().map(|p| p[1] as f64).collect();
        assert!(AXIS_Y[0] - FULCRUM_LABEL_DY - HALF_LINE > stack_top(&largest, 1) + CLEAR);
        const { assert!(AXIS_Y[1] - FULCRUM_LABEL_DY - HALF_LINE > -FRAME_Y_RADIUS + 0.15) };
        const { assert!(STACK_STEP > 2.0 * STACK_R + 0.02 && STACK_UP > STACK_R + AXIS_TICK) };
        const { assert!(FULCRUM_LABEL_DY - HALF_LINE > FULCRUM[1] + FULCRUM_GAP) };
        // distinct stacks keep apart: the closest values are half a unit
        const { assert!(BAL_STEP / 2.0 > 2.0 * STACK_R + 0.1) };
    }
}
