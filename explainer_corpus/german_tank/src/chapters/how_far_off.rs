//! Chapter 09, "How far off could it be?": on the friends' line, their count
//! a fixed number nobody knows, at least sixty; the boxes spread into their
//! row and the line's end slides out, and one dim bar per possible count
//! rises, as tall as the chance that a fair handful of five from that many
//! boxes is all sixty or less (09_1). About forty-two percent at seventy-one,
//! about seven at a hundred (09_2); just over five percent at a hundred and
//! seven, just under at a hundred and eight, where the bars turn faint
//! (09_3); the range, sixty to a hundred and seven, held to be read (09_4).
//! The coverage experiment: the count fixed at seventy-one, chapter 08's
//! twenty handfuls each building its own range, the rows landing after the
//! words end (09_5); nineteen cover it, the one that misses was topped by
//! thirty-eight, ninety-six in a hundred over every handful and never fewer
//! than ninety-five whatever the count, held to be inspected (09_6); what
//! ninety-five percent confidence means, a promise about the method (09_7).
//! Only then the division behind each bar, held to be read (09_8); the
//! prediction, ten boxes with the same largest number (09_9); narrower, the
//! bars for ten and the range sixty to seventy-nine, with card F (09_10).
//!
//! Every chance is an exact fraction from the maximum's distribution, the
//! range's top is where those fractions cross the level, and the scene
//! checks every value it shows against the facts. The screen never writes
//! the ninety-five percent as a chance about the count. Highlights are halos
//! and colour changes are replacements: no typeset text is indicated,
//! recoloured or scaled in place.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, choose, fade_all, thousands, v};
use fmn::prelude::*;

use super::draw::{
    Leg, STRIP_X1, STRIP_Y, dash_spans, dot_opacity, fade_in_all, fade_out_all, fail, line_style, pause, play_legs,
    refill, say, stroke, think, wait_until, x_at, zoom_legs,
};
use super::friends_line::{
    BADGE_AT, COMPUTED_O, FriendsLine, PANEL_SIZE, RING_W, Start, THINK_AT, at, badge, drop_in_all, handfuls,
    is_handful, list, now, panel, printed, read_s, whole,
};
use super::one_estimate::{
    BAR_O, SPAN_W, approx, bar, count_mark, count_text, fraction, grow_bars, plain_dot, right_at, span_points,
};
use super::what_we_assume::{glow, replace, words_left};
use crate::facts;
use crate::model::{self, LINE_DOTS, Q};
use crate::palette;

pub struct HowFarOff {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FIXED: &str = "count: fixed, unknown";

/// The line's last number once its end has slid out (09_1): past the first
/// rare count and the ten-box range's top, so the bars can be seen to keep
/// falling; never written or said.
const VIEW_LAST: u64 = 130;
const VIEW_END: f64 = VIEW_LAST as f64;
/// The bars: one per count, standing on the line, dim, `CHANCE_H` tall for
/// a chance of one and faint once the chance is rare.
const CHANCE_W: f64 = 0.07;
const CHANCE_H: f64 = 3.6;
const FAINT_O: f64 = 0.3;
/// The level: a dashed ink line across the bars at their height for the
/// level, its percent just past the line's end.
const LEVEL_W: f64 = 2.0;
const LEVEL_SIZE: f64 = 20.0;
const LEVEL_GAP: f64 = 0.15;
/// Two counts' chances under the line (09_2), and the range: an ink span
/// under the line with its words under it (09_4, 09_10).
const UNDER_Y: f64 = STRIP_Y - 0.35;
const RANGE_Y: f64 = STRIP_Y - 0.62;
const RANGE_WORDS_Y: f64 = RANGE_Y - 0.3;
/// The coverage experiment's rows (09_5), bottom to top: each a small gold
/// ring at its handful's largest number and its range from there; the
/// count's mark just over the top row; the missing range's largest number
/// left of its ring.
const ROWS_Y0: f64 = -0.33;
const ROWS_DY: f64 = 0.11;
const ROW_RING_R: f64 = 0.038;
const ROW_W: f64 = 2.0;
const MARK_OVER: f64 = 0.08;
const MISS_SIZE: f64 = 20.0;
const MISS_GAP: f64 = 0.1;
/// The zoom (as chapter 02 zooms), the bars rising, a row landing; the
/// inspection holds (09_4, 09_6, 09_8) and the beat after the last line.
const FAN_S: f64 = 0.7;
const EVEN_S: f64 = 0.4;
const ZOOM_S: f64 = 2.0;
const BARS_S: f64 = 2.4;
const BARS_LAG: f64 = 0.05;
const ROW_S: f64 = 0.25;
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;

/// What chapter 09 shows, from the facts: the friends' five, the line's
/// first number, the handful size, their largest number and the lower bound;
/// the two counts the voice tries and their chances, exact and as percents;
/// the level and the confidence; the range, its top, the first rare count,
/// whether its chance is under the level, both chances exact and as shown;
/// the division at the range's top; ten boxes and their range; the twenty
/// draws, their ranges, how many cover the fixed count, how many miss and
/// the missing ones' largest numbers; the coverage over every handful and
/// its percent; whether every count is covered at least at the confidence.
#[derive(Clone, Debug)]
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    k: usize,
    largest: u64,
    at_least: u64,
    count: u64,
    second: u64,
    at_count: Q,
    at_count_percent: u64,
    at_second: Q,
    at_second_percent: u64,
    level: Q,
    level_percent: u64,
    confidence: u64,
    range: Vec<u64>,
    top: u64,
    first_rare: u64,
    rare_below: String,
    at_top: Q,
    at_top_shown: String,
    at_rare: Q,
    at_rare_shown: String,
    from_largest: u64,
    from_top: u64,
    ten: u64,
    ten_range: Vec<u64>,
    ten_top: u64,
    draws: Vec<Vec<u64>>,
    ranges: Vec<[u64; 2]>,
    cover: usize,
    miss: usize,
    miss_largest: Vec<u64>,
    cover_count: Q,
    cover_percent: u64,
    worst: String,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    let q = |key: &str| -> Result<Q, String> { fraction(&printed(&f, key)?) };
    let m = whole(&f, "G-F1.largest")?;
    let count = whole(&f, "G-F8.count")?;
    let second = whole(&f, "G-F9.second_count")?;
    let top = whole(&f, "G-F9.range_top")?;
    let rare = whole(&f, "G-F9.first_rare")?;
    let chance = |n: u64| format!("G-F9.at_most_{m}.{n}");
    checked(Picture {
        numbers: list(&f, "G-F1.numbers")?,
        first: whole(&f, "G-F1.first")?,
        k: whole(&f, "G-F1.handful")? as usize,
        largest: m,
        at_least: whole(&f, "G-F1.at_least")?,
        count,
        second,
        at_count: q(&chance(count))?,
        at_count_percent: whole(&f, &format!("{}.percent", chance(count)))?,
        at_second: q(&chance(second))?,
        at_second_percent: whole(&f, &format!("{}.percent", chance(second)))?,
        level: q("G-F9.level")?,
        level_percent: whole(&f, "G-F9.level_percent")?,
        confidence: whole(&f, "G-F9.confidence_percent")?,
        range: list(&f, "G-F9.range")?,
        top,
        first_rare: rare,
        rare_below: printed(&f, "G-F9.first_rare.percent_below")?,
        at_top: q(&chance(top))?,
        at_top_shown: printed(&f, &format!("{}.percent1", chance(top)))?,
        at_rare: q(&chance(rare))?,
        at_rare_shown: printed(&f, &format!("{}.percent1", chance(rare)))?,
        from_largest: whole(&f, "G-F9.from_largest")?,
        from_top: whole(&f, "G-F9.from_range_top")?,
        ten: whole(&f, "G-F9.ten_boxes")?,
        ten_range: list(&f, "G-F9.ten_boxes_range")?,
        ten_top: whole(&f, "G-F9.ten_boxes_top")?,
        draws: handfuls(&f, "G-F8.again_handfuls")?,
        ranges: ranges_of(&printed(&f, "G-F9.again_ranges")?)?,
        cover: whole(&f, "G-F9.again_cover")? as usize,
        miss: whole(&f, "G-F9.again_miss")? as usize,
        miss_largest: list(&f, "G-F9.again_miss_largest")?,
        cover_count: q("G-F9.cover_71")?,
        cover_percent: whole(&f, "G-F9.cover_71_percent")?,
        worst: printed(&f, "G-F9.worst_coverage_at_least_95")?,
    })
}

/// Ranges as the facts print them: "60,107;69,123", each from its bottom to
/// its top.
fn ranges_of(s: &str) -> Result<Vec<[u64; 2]>, String> {
    s.split(';')
        .map(|r| {
            let bad = || format!("{r:?} is not a range");
            let ends: Vec<u64> = r
                .split(',')
                .map(|x| x.parse().map_err(|_| bad()))
                .collect::<Result<_, _>>()?;
            match ends[..] {
                [lo, hi] if lo <= hi => Ok([lo, hi]),
                _ => Err(bad()),
            }
        })
        .collect()
}

/// Whether range `r` holds count `n`.
fn covers(r: [u64; 2], n: u64) -> bool {
    (r[0]..=r[1]).contains(&n)
}

/// The picture only if the scene can draw it truthfully: the friends' five
/// are a handful on the line from 1, topped by the lower bound; the level
/// and the confidence are its percents; the two counts' chances are exact
/// and rounded as said; the range runs from the largest number to the last
/// count whose chance is at least the level, the next count's under it,
/// both shown as the facts print them; the bars fall all along the line
/// shown, which runs past the first rare count and the ten-box top; the
/// division at the range's top is the chance there; ten boxes give a
/// narrower range; each draw is a handful from the fixed count and builds
/// its range by the same rule inside the line shown, one of them missing
/// the count as the words say; and the coverage over every handful and over
/// every count shown is the facts'.
fn checked(pic: Picture) -> Result<Picture, String> {
    let k = pic.k as u64;
    let m = pic.largest;
    let level = pic.level;
    let chance = |n: u64| model::all_at_most(m, n, k);
    let percent = |q: Q| (q * Q::int(100)).nearest();
    if pic.first != 1
        || !is_handful(&pic.numbers, pic.k, LINE_DOTS)
        || pic.numbers.iter().max() != Some(&m)
        || pic.at_least != m
    {
        return Err(format!(
            "the friends' handful {:?} must be topped by {m}, the count's lower bound",
            pic.numbers
        ));
    }
    if level <= Q::ZERO
        || level >= Q::ONE
        || pic.level_percent as i128 != percent(level)
        || pic.confidence as i128 != percent(Q::ONE - level)
    {
        return Err(format!(
            "the level {level} must be {}%, with {}% confidence",
            pic.level_percent, pic.confidence
        ));
    }
    if pic.count <= m
        || pic.second <= pic.count
        || pic.at_count != chance(pic.count)
        || pic.at_count_percent as i128 != percent(pic.at_count)
        || pic.at_second != chance(pic.second)
        || pic.at_second_percent as i128 != percent(pic.at_second)
    {
        return Err(format!(
            "a handful is all at most {m} with chance {} ({}%) from {} boxes and {} ({}%) from {}",
            pic.at_count, pic.at_count_percent, pic.count, pic.at_second, pic.at_second_percent, pic.second
        ));
    }
    if pic.range != [m, pic.top]
        || pic.top != model::upper_bound(m, k, level)
        || pic.first_rare != pic.top + 1
        || chance(pic.top) < level
        || chance(pic.first_rare) >= level
        || pic.rare_below != "true"
        || pic.at_top != chance(pic.top)
        || pic.at_rare != chance(pic.first_rare)
        || pic.at_top_shown != (pic.at_top * Q::int(100)).about1()?
        || pic.at_rare_shown != (pic.at_rare * Q::int(100)).about1()?
    {
        return Err(format!(
            "the range must run from {m} to {}, the last count at which that chance is at least {level} ({}% and \
             {}% either side)",
            pic.top, pic.at_top_shown, pic.at_rare_shown
        ));
    }
    let falling = |kk: u64| (m..VIEW_LAST).all(|n| model::all_at_most(m, n + 1, kk) < model::all_at_most(m, n, kk));
    if VIEW_LAST <= pic.first_rare.max(pic.ten_top) || !falling(k) || !falling(pic.ten) {
        return Err(format!(
            "the bars must fall all along the line shown to {VIEW_LAST}, past {} and {}",
            pic.first_rare, pic.ten_top
        ));
    }
    if u128::from(pic.from_largest) != choose(m, k)
        || u128::from(pic.from_top) != choose(pic.top, k)
        || Q::count(u128::from(pic.from_largest)) / Q::count(u128::from(pic.from_top)) != pic.at_top
    {
        return Err(format!(
            "the chance at {} must be the handfuls of {k} from {m}, {}, over those from {}, {}",
            pic.top, pic.from_largest, pic.top, pic.from_top
        ));
    }
    if pic.ten <= k
        || pic.ten > m
        || pic.ten_range != [m, pic.ten_top]
        || pic.ten_top != model::upper_bound(m, pic.ten, level)
        || pic.ten_top >= pic.top
    {
        return Err(format!(
            "{} boxes topped by {m} must give the range {m} to {}, narrower than to {}",
            pic.ten, pic.ten_top, pic.top
        ));
    }
    if pic.draws.is_empty() || !pic.draws.iter().all(|h| is_handful(h, pic.k, pic.count)) {
        return Err(format!("each draw must be a handful of {k} from {}", pic.count));
    }
    let built: Vec<[u64; 2]> = pic
        .draws
        .iter()
        .map(|h| {
            let lo = h.iter().copied().max().unwrap_or(m);
            [lo, model::upper_bound(lo, k, level)]
        })
        .collect();
    let missed: Vec<u64> = built.iter().filter(|&&r| !covers(r, pic.count)).map(|r| r[0]).collect();
    if pic.ranges != built
        || built.iter().any(|r| r[1] > VIEW_LAST)
        || pic.cover != built.len() - missed.len()
        || pic.miss != missed.len()
        || pic.miss_largest != missed
        || missed.len() != 1
    {
        return Err(format!(
            "each draw must build its range from its largest number, {built:?}, inside the line shown, {} covering \
             {} and the one the words name missing it",
            pic.cover, pic.count
        ));
    }
    let cover = model::share_by_largest(pic.count, k, |mm| model::upper_bound(mm, k, level) >= pic.count);
    if pic.cover_count != cover
        || pic.cover_percent as i128 != percent(cover)
        || pic.worst != "true"
        || model::worst_coverage(k, VIEW_LAST, level) < Q::ONE - level
    {
        return Err(format!(
            "over every handful the ranges must cover {} with chance {cover} ({}%), and every count shown at least \
             {}% of the time",
            pic.count, pic.cover_percent, pic.confidence
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

fn chance_head(k: usize, m: u64) -> String {
    format!("chance all {k} are at most {m}")
}

fn under_text(n: u64, percent: u64) -> String {
    format!("{n}: {percent}%")
}

fn level_text(percent: u64) -> String {
    format!("{percent}%")
}

fn boxes_text(n: u64, shown: &str) -> String {
    format!("{n} boxes: {shown}%")
}

fn range_text(r: [u64; 2]) -> String {
    format!("range: {} to {}", r[0], r[1])
}

fn largest_text(m: u64) -> String {
    format!("largest {m}")
}

fn cover_text(percent: u64, n: u64) -> String {
    format!("every handful: {percent}% cover {n}")
}

fn any_text(percent: u64) -> String {
    format!("any count: at least {percent}%")
}

fn confidence_text(percent: u64) -> String {
    format!("{percent}% confidence")
}

/// The division behind each bar (09_8), on three panel lines.
fn division_texts(pic: &Picture) -> [String; 3] {
    let k = pic.k;
    [
        format!("chance = handfuls of {k} from {}", pic.largest),
        format!("÷ handfuls of {k} from N"),
        format!(
            "N = {}: {} ÷ {} ≈ {}%",
            pic.top,
            thousands(pic.from_largest),
            thousands(pic.from_top),
            pic.at_top_shown
        ),
    ]
}

fn ten_text(ten: u64, m: u64) -> String {
    format!("{ten} boxes, largest still {m}")
}

/// Card F (09_10), from the facts for the ten-box experiment now on screen.
fn card_f(pic: &Picture) -> String {
    let m = pic.largest;
    format!(
        "range: each N from {m} up where {} fair numbers from {} to N are all at most {m} at least {}% of the time. \
         For any fixed N, such ranges cover N at least {}% of the time; not a chance about N.",
        pic.ten, pic.first, pic.level_percent, pic.confidence
    )
}

// ------------------------------------------------------------- builders

/// One bar per count from `from` to the line's end, as tall as the chance
/// that a fair handful of `k` from that many boxes is all at most `m`, at
/// the opacity `o` gives that chance. Detached.
fn chance_bars(stage: &mut Stage<'_>, m: u64, k: u64, from: u64, o: impl Fn(Q) -> f64) -> fmn::Result<Vec<Mob>> {
    (from..=VIEW_LAST)
        .map(|n| {
            let p = model::all_at_most(m, n, k);
            let h = CHANCE_H * approx(p).map_err(fail)?;
            bar(stage, x_at(n as f64, VIEW_END), CHANCE_W, h, Palette::dim(), o(p))
        })
        .collect()
}

/// The level's dashed line across the bars from `m`, at height `y`.
/// Detached.
fn level_line(stage: &mut Stage<'_>, m: u64, y: f64) -> Vec<Mob> {
    let from = x_at(m as f64, VIEW_END) - CHANCE_W / 2.0;
    let to = x_at(VIEW_END, VIEW_END) + CHANCE_W / 2.0;
    dash_spans(from, to)
        .into_iter()
        .map(|(a, b)| stroke(stage, vec![v(a, y), v(b, y)], line_style(Palette::ink(), LEVEL_W, 1.0)))
        .collect()
}

/// Count `n`'s chance as a percent under the line at its place (09_2).
/// Detached.
fn under_label(kit: &Kit, stage: &mut Stage<'_>, n: u64, percent: u64) -> fmn::Result<Mob> {
    kit.text(
        stage,
        &under_text(n, percent),
        PANEL_SIZE,
        Palette::ink(),
        v(x_at(n as f64, VIEW_END), UNDER_Y),
    )
}

/// The range `r` under the line, with its words under it. Detached.
fn range_marks(kit: &Kit, stage: &mut Stage<'_>, r: [u64; 2]) -> fmn::Result<[Mob; 2]> {
    let (x0, x1) = (x_at(r[0] as f64, VIEW_END), x_at(r[1] as f64, VIEW_END));
    let span = stroke(
        stage,
        span_points(x0, x1, RANGE_Y),
        line_style(Palette::ink(), SPAN_W, 1.0),
    );
    let words = kit.text(
        stage,
        &range_text(r),
        PANEL_SIZE,
        Palette::ink(),
        v((x0 + x1) / 2.0, RANGE_WORDS_Y),
    )?;
    Ok([span, words])
}

/// Row `j`'s height in the coverage experiment.
fn row_y(j: usize) -> f64 {
    ROWS_Y0 + ROWS_DY * j as f64
}

/// The coverage experiment's rows: each draw's small gold ring at its
/// largest number, and its range from there in dim and in ink. Detached.
struct Rows {
    rings: Vec<Mob>,
    dim: Vec<Mob>,
    ink: Vec<Mob>,
}

impl Rows {
    fn build(stage: &mut Stage<'_>, ranges: &[[u64; 2]]) -> Rows {
        let (mut rings, mut dim, mut ink) = (Vec::new(), Vec::new(), Vec::new());
        for (j, r) in ranges.iter().enumerate() {
            let y = row_y(j);
            let (x0, x1) = (x_at(r[0] as f64, VIEW_END), x_at(r[1] as f64, VIEW_END));
            let ring = Dot::new()
                .point(v(x0, y))
                .radius(ROW_RING_R)
                .style(line_style(palette::seen(), RING_W, COMPUTED_O))
                .build();
            rings.push(stage.arena_mut().add(ring));
            let points = vec![v(x0 + ROW_RING_R, y), v(x1, y)];
            dim.push(stroke(stage, points.clone(), line_style(Palette::dim(), ROW_W, 1.0)));
            ink.push(stroke(stage, points, line_style(Palette::ink(), ROW_W, 1.0)));
        }
        Rows { rings, dim, ink }
    }

    /// The rings and the ranges as they stand once the covering ones
    /// (`hits`) are ink.
    fn shown(&self, hits: &[usize]) -> Vec<Mob> {
        let mut all = self.rings.clone();
        all.extend(
            self.dim
                .iter()
                .zip(&self.ink)
                .enumerate()
                .map(|(j, (&dim, &ink))| if hits.contains(&j) { ink } else { dim }),
        );
        all
    }
}

impl SceneConstruct for HowFarOff {
    fn name(&self) -> &str {
        "09_how_far_off"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let k = pic.k as u64;
        let m = pic.largest;
        let opening = LINE_DOTS as f64;
        let index = |n: u64| (n - m) as usize;

        // the friends' line as chapter 03 left it, the boxes on their
        // numbers, and the badge
        let strip = FriendsLine::build(
            &kit,
            stage,
            pic.first,
            LINE_DOTS,
            &pic.numbers,
            &[],
            Start {
                end: opening,
                fan: 0.0,
                rise: 1.0,
                widest: VIEW_END,
            },
        )?;
        for &piece in &strip.base {
            stage.add_to_scene(piece)?;
        }
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let mut opened = strip.opening();
        opened.push(corner);
        fade_in_all(stage, &opened, 0.8)?;
        let mut boxes = strip.bodies.clone();
        boxes.extend(&strip.labels);
        boxes.extend(&strip.ticks);
        let top_box = pic
            .numbers
            .iter()
            .position(|&n| n == m)
            .ok_or_else(|| fail(format!("no box at {m}")))?;

        // 09_1: the count fixed and unknown; a halo round the box at the
        // lower bound; the boxes spread, the line's fading end fills in and
        // slides out; the chances rise as bars, left to right
        let line = say(&kit, stage, "09_1")?;
        line.wait_for(stage, "a fixed number nobody knows")?;
        let fixed = words_left(&kit, stage, FIXED, &[], panel(0))?;
        fade_in_all(stage, &[fixed], 0.5)?;
        line.wait_for(stage, "at least sixty")?;
        glow(
            stage,
            &[strip.bodies[top_box], strip.labels[top_box]],
            palette::certain(),
        )?;
        line.wait_for(stage, "How large could it be")?;
        let even: Vec<(Mob, Srgb, f64)> = (pic.first..=LINE_DOTS)
            .zip(strip.dots.iter().copied())
            .filter(|(n, _)| !pic.numbers.contains(n))
            .map(|(n, d)| (d, palette::line(), dot_opacity(x_at(n as f64, VIEW_END))))
            .collect();
        for leg in zoom_legs([opening, 0.0], [VIEW_END, 1.0]) {
            if matches!(leg, Leg::End(_)) {
                refill(stage, &even, EVEN_S)?;
            }
            play_legs(stage, strip.t.end, strip.t.fan, &[leg], FAN_S, ZOOM_S)?;
        }
        line.wait_for(stage, "For each possible count")?;
        let head = words_left(&kit, stage, &chance_head(pic.k, m), &[], panel(1))?;
        fade_in_all(stage, &[head], 0.5)?;
        let bars = chance_bars(stage, m, k, m, |_| BAR_O)?;
        grow_bars(stage, &bars, BARS_S, BARS_LAG)?;

        // 09_2: two counts' chances under the line, a halo round each bar;
        // a halo round the bars from the second on
        let line = say(&kit, stage, "09_2")?;
        line.wait_for(stage, "If the count were seventy-one")?;
        let under_count = under_label(&kit, stage, pic.count, pic.at_count_percent)?;
        fade_in_all(stage, &[under_count], 0.4)?;
        glow(stage, &[bars[index(pic.count)]], Palette::ink())?;
        line.wait_for(stage, "If it were a hundred")?;
        let under_second = under_label(&kit, stage, pic.second, pic.at_second_percent)?;
        fade_in_all(stage, &[under_second], 0.4)?;
        glow(stage, &[bars[index(pic.second)]], Palette::ink())?;
        line.wait_for(stage, "The bigger the count")?;
        glow(stage, &bars[index(pic.second)..], Palette::ink())?;

        // 09_3: the level's dashed line; the range's top and the first rare
        // count on the panel, a halo round each bar; the rare bars turn
        // faint
        let line = say(&kit, stage, "09_3")?;
        line.wait_for(stage, "At a hundred and seven boxes")?;
        let level_y = STRIP_Y + CHANCE_H * approx(pic.level).map_err(fail)?;
        let mut level = level_line(stage, m, level_y);
        level.push(kit.text_left(
            stage,
            &level_text(pic.level_percent),
            LEVEL_SIZE,
            Palette::ink(),
            v(STRIP_X1 + LEVEL_GAP, level_y),
        )?);
        let at_top = words_left(&kit, stage, &boxes_text(pic.top, &pic.at_top_shown), &[], panel(2))?;
        let mut shown = level.clone();
        shown.push(at_top);
        fade_in_all(stage, &shown, 0.5)?;
        glow(stage, &[bars[index(pic.top)]], Palette::ink())?;
        line.wait_for(stage, "At a hundred and eight")?;
        let at_rare = words_left(
            &kit,
            stage,
            &boxes_text(pic.first_rare, &pic.at_rare_shown),
            &[],
            panel(3),
        )?;
        fade_in_all(stage, &[at_rare], 0.4)?;
        glow(stage, &[bars[index(pic.first_rare)]], Palette::ink())?;
        line.wait_for(stage, "it keeps falling from there")?;
        let faint = chance_bars(stage, m, k, pic.first_rare, |_| FAINT_O)?;
        replace(stage, &bars[index(pic.first_rare)..], &faint, 0.6)?;
        let mut kept = bars[..index(pic.first_rare)].to_vec();
        kept.extend(&faint);

        // 09_4: the two chances go and the range comes in under the line; a
        // halo round the level; held to be read
        let line = say(&kit, stage, "09_4")?;
        line.wait_for(stage, "So keep every count")?;
        fade_out_all(stage, &[under_count, under_second], 0.3)?;
        let range = range_marks(&kit, stage, [m, pic.top])?;
        fade_in_all(stage, &range, 0.5)?;
        line.wait_for(stage, "at least five times in a hundred")?;
        glow(stage, &level, Palette::ink())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 09_5: the bars, the level, the panel's chances, the range and the
        // boxes go, the friends' dots turn plain; the count's mark rises at
        // the fixed count; the twenty rows land one after another, bottom
        // to top, and keep landing after the words end
        let line = say(&kit, stage, "09_5")?;
        line.wait_for(stage, "Test it where we know")?;
        let mut gone = kept.clone();
        gone.extend(&level);
        gone.extend([head, at_top, at_rare]);
        gone.extend(range);
        gone.extend(&boxes);
        gone.extend(pic.numbers.iter().map(|&n| strip.dots[(n - pic.first) as usize]));
        let plain: Vec<Mob> = pic.numbers.iter().map(|&n| plain_dot(stage, n, VIEW_END)).collect();
        replace(stage, &gone, &plain, 0.6)?;
        line.wait_for(stage, "Fix the count at seventy-one")?;
        let mark = count_mark(stage, pic.count, VIEW_END, row_y(pic.ranges.len() - 1) + MARK_OVER);
        let count = words_left(&kit, stage, &count_text(pic.count), &[], panel(0))?;
        replace(stage, &[fixed], &[count], 0.5)?;
        fade_in_all(stage, &[mark], 0.5)?;
        line.wait_for(stage, "each build their own range")?;
        let rows = Rows::build(stage, &pic.ranges);
        for (&ring, &dim) in rows.rings.iter().zip(&rows.dim) {
            drop_in_all(stage, &[ring, dim], ROW_S)?;
        }

        // 09_6: the covering ranges turn ink; the missing one's largest
        // number and a halo round it; the coverage over every handful and
        // over every count on the panel; held to be inspected
        let line = say(&kit, stage, "09_6")?;
        line.wait_for(stage, "Nineteen of the twenty ranges cover")?;
        let (hits, misses): (Vec<usize>, Vec<usize>) =
            (0..pic.ranges.len()).partition(|&j| covers(pic.ranges[j], pic.count));
        let dim_hits: Vec<Mob> = hits.iter().map(|&j| rows.dim[j]).collect();
        let ink_hits: Vec<Mob> = hits.iter().map(|&j| rows.ink[j]).collect();
        replace(stage, &dim_hits, &ink_hits, 0.6)?;
        line.wait_for(stage, "The range that misses")?;
        let j = *misses.first().ok_or_else(|| fail("no range misses".to_owned()))?;
        let lo = pic.ranges[j][0];
        let largest = kit.text(stage, &largest_text(lo), MISS_SIZE, Palette::ink(), ORIGIN)?;
        right_at(
            stage,
            largest,
            x_at(lo as f64, VIEW_END) - ROW_RING_R - MISS_GAP,
            row_y(j),
        );
        fade_in_all(stage, &[largest], 0.4)?;
        glow(stage, &[rows.rings[j], rows.dim[j]], Palette::ink())?;
        line.wait_for(stage, "Over every handful")?;
        let cover = words_left(&kit, stage, &cover_text(pic.cover_percent, pic.count), &[], panel(1))?;
        fade_in_all(stage, &[cover], 0.5)?;
        line.wait_for(stage, "never fewer than ninety-five")?;
        let any = words_left(&kit, stage, &any_text(pic.confidence), &[], panel(2))?;
        fade_in_all(stage, &[any], 0.5)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 09_7: what the confidence means, on the panel; halos round the
        // fixed mark, round the twenty ranges and round the confidence
        let line = say(&kit, stage, "09_7")?;
        line.wait_for(stage, "ninety-five percent confidence")?;
        let confidence = words_left(&kit, stage, &confidence_text(pic.confidence), &[], panel(3))?;
        fade_in_all(stage, &[confidence], 0.5)?;
        line.wait_for(stage, "The count stays fixed")?;
        glow(stage, &[mark], Palette::ink())?;
        line.wait_for(stage, "the range changes")?;
        let experiment = rows.shown(&hits);
        glow(stage, &experiment, Palette::ink())?;
        line.wait_for(stage, "a promise about the method")?;
        glow(stage, &[confidence], Palette::ink())?;

        // 09_8: the experiment goes and the bars, the level and the range
        // come back; the division behind each bar on the panel, a halo round
        // the bar at the range's top; held to be read
        let line = say(&kit, stage, "09_8")?;
        line.wait_for(stage, "Behind each bar")?;
        let mut out = experiment.clone();
        out.extend([largest, mark, count, cover, any, confidence]);
        let mut back = kept.clone();
        back.extend(&level);
        back.extend(range);
        replace(stage, &out, &back, 0.6)?;
        line.wait_for(stage, "Count the handfuls of five")?;
        let [d0, d1, d2] = division_texts(&pic);
        let over = words_left(&kit, stage, &d0, &[], panel(0))?;
        let under = words_left(&kit, stage, &d1, &[], panel(1))?;
        fade_in_all(stage, &[over, under], 0.5)?;
        line.wait_for(stage, "At a hundred and seven boxes")?;
        let instance = words_left(&kit, stage, &d2, &[], panel(2))?;
        fade_in_all(stage, &[instance], 0.5)?;
        glow(stage, &[bars[index(pic.top)]], Palette::ink())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 09_9: the prediction; the new handful size on the panel, the
        // countdown at the right
        let line = say(&kit, stage, "09_9")?;
        line.wait_for(stage, "If the friends had ten boxes")?;
        let ten = words_left(&kit, stage, &ten_text(pic.ten, m), &[], panel(0))?;
        replace(stage, &[over, under, instance], &[ten], 0.5)?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 09_10: the bars become the chances for ten, the range narrows,
        // card F, held until it can be read
        let line = say(&kit, stage, "09_10")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "Ten numbers all sixty or less")?;
        let tens = chance_bars(stage, m, pic.ten, m, |p| if p >= pic.level { BAR_O } else { FAINT_O })?;
        replace(stage, &kept, &tens, 0.8)?;
        line.wait_for(stage, "the range shrinks")?;
        let narrower = range_marks(&kit, stage, [m, pic.ten_top])?;
        replace(stage, &range, &narrower, 0.6)?;
        let text = card_f(&pic);
        let card = precisely(&kit, stage, &text)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = now(stage);
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;
        // card F stays until it can be read
        wait_until(stage, card_up + read_s(&text))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::Script;
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{BOX_W, DOT_R, FAN_STEP, FAN_X0, PAUSE_NOTE, SMALL, THINK_R};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, PANEL_X, PANEL_Y, est_width};
    use crate::chapters::one_estimate::SPAN_TICK;
    use crate::model::{AGAIN_STREAM, FRIENDS, KNOWN, all_handfuls, drawn_handfuls};

    /// The countdown's note, at the size `think` writes it.
    const NOTE_SIZE: f64 = 18.0;
    /// A line's half height at the panel's size and at the small size, a
    /// stroke width's length in scene units, and the space kept between
    /// things that must not touch.
    const HALF_LINE: f64 = 0.16;
    const HALF_SMALL: f64 = 0.12;
    const STROKE_UNIT: f64 = 0.01;
    const CLEAR: f64 = 0.1;
    /// A "precisely:" card's line, beside its label: the kit starts at 92
    /// characters and narrows by 4 until the widest line fits.
    const CARD_CHARS_WIDEST: usize = 92;
    const CARD_CHARS_NARROWEST: usize = 64;

    fn good() -> Picture {
        picture().unwrap()
    }

    /// The scene draws the facts, and its lines read as the voice says
    /// them: the friends' five and the fixed count are the model's, the
    /// twenty draws are chapter 08's, and every line the panel, the words
    /// under the line, the rows and card F show. The confidence is never
    /// written as a chance about the count.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(pic.numbers, FRIENDS);
        assert_eq!(pic.count, KNOWN);
        assert_eq!(pic.draws, drawn_handfuls(AGAIN_STREAM, KNOWN, pic.k, pic.draws.len()));
        assert_eq!(chance_head(pic.k, pic.largest), "chance all 5 are at most 60");
        assert_eq!(under_text(pic.count, pic.at_count_percent), "71: 42%");
        assert_eq!(under_text(pic.second, pic.at_second_percent), "100: 7%");
        assert_eq!(level_text(pic.level_percent), "5%");
        assert_eq!(boxes_text(pic.top, &pic.at_top_shown), "107 boxes: 5.1%");
        assert_eq!(boxes_text(pic.first_rare, &pic.at_rare_shown), "108 boxes: 4.9%");
        assert_eq!(range_text([pic.largest, pic.top]), "range: 60 to 107");
        assert_eq!(count_text(pic.count), "count = 71");
        assert_eq!((pic.draws.len(), pic.cover, pic.miss), (20, 19, 1));
        assert_eq!(largest_text(pic.miss_largest[0]), "largest 38");
        assert_eq!(cover_text(pic.cover_percent, pic.count), "every handful: 96% cover 71");
        assert_eq!(any_text(pic.confidence), "any count: at least 95%");
        assert_eq!(confidence_text(pic.confidence), "95% confidence");
        assert_eq!(
            division_texts(&pic),
            [
                "chance = handfuls of 5 from 60",
                "÷ handfuls of 5 from N",
                "N = 107: 5,461,512 ÷ 106,308,566 ≈ 5.1%",
            ]
            .map(str::to_owned)
        );
        assert_eq!(ten_text(pic.ten, pic.largest), "10 boxes, largest still 60");
        assert_eq!(range_text([pic.largest, pic.ten_top]), "range: 60 to 79");
        let card = card_f(&pic);
        assert_eq!(
            card,
            "range: each N from 60 up where 10 fair numbers from 1 to N are all at most 60 at least 5% of the time. \
             For any fixed N, such ranges cover N at least 95% of the time; not a chance about N."
        );
        for text in [
            any_text(pic.confidence),
            confidence_text(pic.confidence),
            cover_text(pic.cover_percent, pic.count),
        ] {
            assert!(!text.contains("chance"), "{text}");
        }
        assert_eq!(card.matches("chance").count(), 1);
        assert!(card.contains("not a chance about N"));
    }

    /// Ranges are read as the facts print them, and a range holds a count
    /// at both its ends and nowhere outside them.
    #[test]
    fn ranges_are_read_as_printed() {
        assert_eq!(ranges_of("60,107;38,67").unwrap(), [[60, 107], [38, 67]]);
        for bad in ["", "60", "60,107,1", "107,60", "60,x", "60,107;"] {
            assert!(ranges_of(bad).is_err(), "{bad:?}");
        }
        assert!(covers([38, 67], 38) && covers([38, 67], 67));
        assert!(!covers([38, 67], 37) && !covers([38, 67], 68));
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
                numbers: vec![14, 27, 33, 51, 59],
                ..pic.clone()
            },
            "friends' handful",
        );
        refused(
            Picture {
                at_least: 59,
                ..pic.clone()
            },
            "friends' handful",
        );
        refused(
            Picture {
                level: Q::new(1, 10),
                ..pic.clone()
            },
            "the level",
        );
        refused(
            Picture {
                confidence: 90,
                ..pic.clone()
            },
            "the level",
        );
        refused(
            Picture {
                at_count: model::all_at_most(60, 70, 5),
                ..pic.clone()
            },
            "with chance",
        );
        refused(
            Picture {
                at_second_percent: 8,
                ..pic.clone()
            },
            "with chance",
        );
        // the range's top one count too far, and where the first guess
        // (60/N)⁵ would put it
        for top in [108, 109] {
            refused(
                Picture {
                    range: vec![60, top],
                    top,
                    ..pic.clone()
                },
                "the range must run",
            );
        }
        refused(
            Picture {
                first_rare: pic.top,
                ..pic.clone()
            },
            "the range must run",
        );
        refused(
            Picture {
                rare_below: "false".to_owned(),
                ..pic.clone()
            },
            "the range must run",
        );
        refused(
            Picture {
                at_top_shown: "5.2".to_owned(),
                ..pic.clone()
            },
            "the range must run",
        );
        refused(
            Picture {
                at_rare_shown: "5.0".to_owned(),
                ..pic.clone()
            },
            "the range must run",
        );
        refused(
            Picture {
                from_top: choose(108, 5) as u64,
                ..pic.clone()
            },
            "handfuls of 5 from 60",
        );
        // "more boxes, more uncertainty": a wider range for ten
        refused(
            Picture {
                ten_range: vec![60, 107],
                ten_top: 107,
                ..pic.clone()
            },
            "boxes topped by",
        );
        refused(Picture { ten: 4, ..pic.clone() }, "boxes topped by");
        let mut draws = pic.draws.clone();
        draws[0] = vec![6, 10, 43, 44, 72];
        refused(Picture { draws, ..pic.clone() }, "each draw must be a handful");
        // ranges topped at the estimate instead
        let at_estimate: Vec<[u64; 2]> = pic
            .ranges
            .iter()
            .map(|r| [r[0], model::gap_of(r[0], 5).nearest() as u64])
            .collect();
        refused(
            Picture {
                ranges: at_estimate,
                ..pic.clone()
            },
            "build its range",
        );
        refused(
            Picture {
                cover: pic.cover + 1,
                miss: 0,
                ..pic.clone()
            },
            "build its range",
        );
        refused(
            Picture {
                miss_largest: vec![45],
                ..pic.clone()
            },
            "build its range",
        );
        // the 95% read as the coverage at 71
        refused(
            Picture {
                cover_count: Q::new(19, 20),
                ..pic.clone()
            },
            "over every handful",
        );
        refused(
            Picture {
                cover_percent: 95,
                ..pic.clone()
            },
            "over every handful",
        );
        refused(
            Picture {
                worst: "false".to_owned(),
                ..pic
            },
            "over every handful",
        );
    }

    /// The range's top is where the chance crosses the level, counted
    /// handful by handful: for handfuls of 1 to 4 topped by up to 12, the
    /// share of every handful from the top count that is all at most the
    /// largest is at least the level, and from one count more it is under
    /// it. The negative control: the first guess from (60/N)⁵ puts the
    /// friends' top elsewhere.
    #[test]
    fn the_range_is_where_the_chance_crosses_the_level() {
        let pic = good();
        let share = |m: u64, n: u64, k: usize| {
            let hands = all_handfuls(n, k);
            let low = hands.iter().filter(|h| h[k - 1] <= m).count();
            Q::new(low as i128, hands.len() as i128)
        };
        for k in 1..=4usize {
            for m in k as u64..=12 {
                let top = model::upper_bound(m, k as u64, pic.level);
                assert!(share(m, top, k) >= pic.level, "k {k}, m {m}");
                assert!(share(m, top + 1, k) < pic.level, "k {k}, m {m}");
                assert_eq!(share(m, top, k), model::all_at_most(m, top, k as u64));
            }
        }
        let level = approx(pic.level).unwrap();
        let ratio = |n: u64| (pic.largest as f64 / n as f64).powi(pic.k as i32);
        let guess = (pic.largest..).find(|&n| ratio(n + 1) < level).unwrap();
        assert_ne!(guess, pic.top);
    }

    /// Ranges built this way cover the count at least 95 times in a
    /// hundred whatever it is: for handfuls of 1 to 4 from every count up to
    /// 30, listing every handful, and as the scene counts them by the
    /// largest number. The negative control: at a level of 1 in 10 the same
    /// ranges fall short of 95 somewhere. A range's top depends only on the
    /// largest number, so each top is found once per largest.
    #[test]
    fn ranges_built_this_way_cover_every_small_count() {
        let pic = good();
        let last = 30;
        let tops = |level: Q, k: usize| -> Vec<u64> {
            (0..=last)
                .map(|m| {
                    if m < k as u64 {
                        0
                    } else {
                        model::upper_bound(m, k as u64, level)
                    }
                })
                .collect()
        };
        let cover = |tops: &[u64], k: usize, n: u64| {
            let hands = all_handfuls(n, k);
            let held = hands.iter().filter(|h| tops[h[k - 1] as usize] >= n).count();
            Q::new(held as i128, hands.len() as i128)
        };
        let mut short = 0;
        for k in 1..=4usize {
            let (ours, loose) = (tops(pic.level, k), tops(Q::new(1, 10), k));
            for n in k as u64..=last {
                let c = cover(&ours, k, n);
                assert!(c >= Q::ONE - pic.level, "k {k}, n {n}: {c}");
                assert_eq!(
                    c,
                    model::share_by_largest(n, k as u64, |m| model::upper_bound(m, k as u64, pic.level) >= n)
                );
                if cover(&loose, k, n) < Q::ONE - pic.level {
                    short += 1;
                }
            }
        }
        assert!(short > 0);
    }

    /// The 95% is a promise about the method, not a chance about the count:
    /// with every count from 60 to 200 (or to 1,000) equally likely
    /// beforehand, the chance afterwards that the count is at most 107 is
    /// neither 95% nor the same for both; and ranges topped at the estimate
    /// cover 71 far less often.
    #[test]
    fn the_confidence_is_neither_a_posterior_nor_the_estimate() {
        let pic = good();
        let k = pic.k as u64;
        let after = |last: u64| {
            let weight = |n: u64| 1.0 / choose(n, k) as f64;
            let kept: f64 = (pic.largest..=pic.top).map(weight).sum();
            kept / (pic.largest..=last).map(weight).sum::<f64>()
        };
        let confidence = pic.confidence as f64 / 100.0;
        let (narrow, wide) = (after(200), after(1000));
        assert!(
            narrow < confidence - 0.02 && wide < confidence - 0.02,
            "{narrow}, {wide}"
        );
        assert!((narrow - wide).abs() > 0.005);
        let at_estimate = model::share_by_largest(pic.count, k, |m| model::gap_of(m, k) >= Q::int(pic.count as i128));
        assert!(at_estimate < Q::new(2, 3));
        assert!(pic.cover_count > Q::ONE - pic.level);
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            (
                "09_1",
                &[
                    "a fixed number nobody knows",
                    "at least sixty",
                    "How large could it be",
                    "For each possible count",
                ],
            ),
            (
                "09_2",
                &[
                    "If the count were seventy-one",
                    "If it were a hundred",
                    "The bigger the count",
                ],
            ),
            (
                "09_3",
                &[
                    "At a hundred and seven boxes",
                    "At a hundred and eight",
                    "it keeps falling from there",
                ],
            ),
            ("09_4", &["So keep every count", "at least five times in a hundred"]),
            (
                "09_5",
                &[
                    "Test it where we know",
                    "Fix the count at seventy-one",
                    "each build their own range",
                ],
            ),
            (
                "09_6",
                &[
                    "Nineteen of the twenty ranges cover",
                    "The range that misses",
                    "Over every handful",
                    "never fewer than ninety-five",
                ],
            ),
            (
                "09_7",
                &[
                    "ninety-five percent confidence",
                    "The count stays fixed",
                    "the range changes",
                    "a promise about the method",
                ],
            ),
            (
                "09_8",
                &[
                    "Behind each bar",
                    "Count the handfuls of five",
                    "At a hundred and seven boxes",
                ],
            ),
            ("09_9", &["If the friends had ten boxes"]),
            ("09_10", &["Ten numbers all sixty or less", "the range shrinks"]),
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
        // the prediction's line shows nothing of the answer
        let asked = script.text("09_9").unwrap();
        for answer in ["seventy-nine", "shrinks", "rare"] {
            assert!(!asked.contains(answer), "{answer}");
        }
    }

    /// The panel, the bars, the level, the words under the line, the rows,
    /// the countdown and card F keep apart: every panel line in the
    /// title-safe area; every bar, for five boxes and for ten, inside the
    /// frame, right of the badge and of the boxes in their row, under the
    /// panel where it reaches across it, apart from its neighbours and too
    /// low to reach the countdown; the level over the line's dots and its
    /// percent past the line's end; the two chances under the line apart;
    /// both ranges' words under their spans, card F under them; the rows
    /// over the line and under the panel, same-number rings in neighbouring
    /// rows apart, the missing range's words clear of its neighbours.
    #[test]
    fn the_bars_the_rows_and_the_words_fit_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let k = pic.k as u64;
        let m = pic.largest;
        let [d0, d1, d2] = division_texts(&pic);
        for text in [
            FIXED.to_owned(),
            chance_head(pic.k, m),
            boxes_text(pic.top, &pic.at_top_shown),
            boxes_text(pic.first_rare, &pic.at_rare_shown),
            count_text(pic.count),
            cover_text(pic.cover_percent, pic.count),
            any_text(pic.confidence),
            confidence_text(pic.confidence),
            d0,
            d1,
            d2,
            ten_text(pic.ten, m),
        ] {
            assert!(PANEL_X + est_width(&text, PANEL_SIZE) < safe_x, "{text}");
        }
        let panel_floor = PANEL_Y[3] - HALF_LINE - CLEAR;
        let badge_right = BADGE_AT[0] + est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0];
        let boxes_right = FAN_X0 + FAN_STEP * (pic.k - 1) as f64 + BOX_W * SMALL / 2.0;
        let first_left = x_at(m as f64, VIEW_END) - CHANCE_W / 2.0;
        assert!(first_left > badge_right.max(boxes_right) + CLEAR);
        // the countdown (09_9) over the five-box bars, its note under it
        let note_y = THINK_AT[1] - THINK_R - 0.3;
        let note_half = est_width(PAUSE_NOTE, NOTE_SIZE) / 2.0;
        const { assert!(THINK_AT[1] + THINK_R < PANEL_Y[0] - HALF_LINE - CLEAR) };
        for kk in [k, pic.ten] {
            let mut last: Option<f64> = None;
            for n in m..=VIEW_LAST {
                let x = x_at(n as f64, VIEW_END);
                let top = STRIP_Y + CHANCE_H * approx(model::all_at_most(m, n, kk)).unwrap();
                assert!(x - CHANCE_W / 2.0 > -safe_x && x + CHANCE_W / 2.0 < safe_x, "{kk}: {n}");
                assert!(top < FRAME_Y_RADIUS - 0.25, "{kk}: {n}");
                if x + CHANCE_W / 2.0 > PANEL_X - CLEAR {
                    assert!(top < panel_floor, "{kk}: {n} under the panel");
                }
                if kk == k && (x - THINK_AT[0]).abs() < note_half.max(THINK_R) + CHANCE_W {
                    assert!(top < note_y - HALF_SMALL - CLEAR, "{n} under the countdown");
                }
                if let Some(p) = last {
                    assert!(x - p > CHANCE_W + 0.02, "{kk}: {n}");
                }
                last = Some(x);
            }
        }
        // the level
        let level_y = STRIP_Y + CHANCE_H * approx(pic.level).unwrap();
        assert!(level_y - LEVEL_W * STROKE_UNIT / 2.0 > STRIP_Y + DOT_R + CLEAR / 2.0);
        let label_left = STRIP_X1 + LEVEL_GAP;
        assert!(label_left > x_at(VIEW_END, VIEW_END) + CHANCE_W / 2.0 + CLEAR / 2.0);
        assert!(label_left + est_width(&level_text(pic.level_percent), LEVEL_SIZE) < safe_x);
        // the two chances under the line
        let unders = [
            (pic.count, under_text(pic.count, pic.at_count_percent)),
            (pic.second, under_text(pic.second, pic.at_second_percent)),
        ];
        let spans: Vec<[f64; 2]> = unders
            .iter()
            .map(|(n, text)| {
                let (x, half) = (x_at(*n as f64, VIEW_END), est_width(text, PANEL_SIZE) / 2.0);
                [x - half, x + half]
            })
            .collect();
        assert!(spans[0][1] + CLEAR < spans[1][0]);
        assert!(spans[0][0] > -safe_x && spans[1][1] < safe_x);
        const { assert!(UNDER_Y + HALF_LINE < STRIP_Y - DOT_R - CLEAR / 2.0) };
        // both ranges, their words, and card F's three lines (0.3 plus 0.36
        // a line, 0.2 over the frame's foot) under them
        for r in [[m, pic.top], [m, pic.ten_top]] {
            let (x0, x1) = (x_at(r[0] as f64, VIEW_END), x_at(r[1] as f64, VIEW_END));
            let half = est_width(&range_text(r), PANEL_SIZE) / 2.0;
            assert!(
                (x0 + x1) / 2.0 - half > -safe_x && (x0 + x1) / 2.0 + half < safe_x,
                "{r:?}"
            );
        }
        const { assert!(RANGE_Y + SPAN_TICK < STRIP_Y - DOT_R - CLEAR) };
        const { assert!(RANGE_WORDS_Y + HALF_LINE < RANGE_Y - CLEAR / 2.0) };
        const { assert!(-FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 3.0 < RANGE_WORDS_Y - HALF_LINE - CLEAR / 2.0) };
        // the rows, the mark over them, and the missing range's words
        let ring_out = ROW_RING_R + RING_W * STROKE_UNIT / 2.0;
        let rows = pic.ranges.len();
        assert!(row_y(0) - ring_out > STRIP_Y + DOT_R + CLEAR / 2.0);
        assert!(row_y(rows - 1) + MARK_OVER < panel_floor);
        const { assert!(ROWS_DY > 2.0 * (ROW_RING_R + RING_W * STROKE_UNIT / 2.0)) };
        assert!(pic.ranges.iter().all(|r| x_at(r[1] as f64, VIEW_END) < safe_x));
        let j = pic.ranges.iter().position(|&r| !covers(r, pic.count)).unwrap();
        let lo = pic.ranges[j][0];
        let right = x_at(lo as f64, VIEW_END) - ROW_RING_R - MISS_GAP;
        assert!(right - est_width(&largest_text(lo), MISS_SIZE) > -safe_x);
        for (i, r) in pic.ranges.iter().enumerate() {
            if i != j && (row_y(i) - row_y(j)).abs() < HALF_SMALL + ring_out + CLEAR / 2.0 {
                assert!(x_at(r[0] as f64, VIEW_END) - ring_out > right + CLEAR, "row {i}");
            }
        }
    }

    /// Card F says the range and its promise in at most three lines at
    /// every width the lower third might narrow to, and is up long enough
    /// to be read.
    #[test]
    fn card_f_fits_and_is_read() {
        let text = card_f(&good());
        for chars in (CARD_CHARS_NARROWEST..=CARD_CHARS_WIDEST).step_by(4) {
            assert!(wrap(&text, chars, 4).len() <= 3, "{chars}");
        }
        assert!(read_s(&text) < 15.0);
        // the negative control: one clause more would not fit
        let longer = format!("{text} The count is one number.");
        assert!(wrap(&longer, CARD_CHARS_NARROWEST, 4).len() > 3);
    }
}
