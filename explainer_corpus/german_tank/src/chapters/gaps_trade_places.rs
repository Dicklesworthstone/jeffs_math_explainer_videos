//! Chapter 06, "Gaps that can trade places": chapter 05's tiny table comes
//! back square for square, and every row's three unseen squares become its
//! three gaps, below the smaller number, between the two and above the
//! larger, each column named as it is said (06_1). The two worked rows'
//! gap sizes, then the other eight, always adding to three (06_2). The
//! list for two and five read backwards on five empty squares, gap by gap,
//! rebuilds the pair (06_3); so pairs and lists match exactly, the lists
//! are every split of three into three, and each list is as likely as its
//! pair, held to be read (06_4). The first row's list, written again at
//! the right, has its gaps below and above trade places, each number
//! keeping its colour, and lands beside the row whose list it is, held to
//! be compared (06_5). Every row's partner, a pair of halos at a time; the
//! below and above columns hold the same ten numbers (06_6), and so does
//! the between column (06_7); so every column sums to ten and averages
//! one, the top gap's too, held to be read (06_8). Back on the friends'
//! line, five numbers and six gaps, five arcs, and the prediction (06_9);
//! the sixth is the top gap, above sixty, which averaged one box in the
//! small company like the gaps below it (06_10).
//!
//! The scene computes every row's list from its pair, every row's partner
//! and every column from the lists, and refuses facts it would contradict.
//! Highlights are halos and colour changes are replacements (chapter 04's
//! builders): no typeset text is indicated, recoloured or scaled in place,
//! and nothing is drawn on top of a copy of itself; the trade moves a copy
//! written at the right, never the table's own row. The friends' line with
//! every gap drawn is built here for chapter 07 too.

use explainer_kit::{Kit, Palette, choose, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    BODY_O, BRACKET_DY, BRACKET_W, LABEL_DY, STRIP_X1, STRIP_Y, dash_spans, fade_in_all, fade_in_lagged, fade_out_all,
    fail, flat, line_style, move_all, pause, say, stroke, think, wait_until, x_at,
};
use super::friends_line::{
    ARC_H, ARC_W, BADGE_AT, FriendsLine, PANEL_SIZE, Start, THINK_AT, arc_points, at, badge, gap_spans, handfuls,
    is_handful, list, panel, printed, whole,
};
use super::what_we_assume::{HALO_HOLD_S, HALO_IN_S, glow, halo, replace, words_left};
use crate::facts;
use crate::model::{LINE_DOTS, Q};
use crate::palette;

pub struct GapsTradePlaces {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HEADS: [&str; 3] = ["below", "between", "above"];
const PAIR_WORD: &str = "pair";
const GAP_LIST: &str = "gap list";
const SUM_WORD: &str = "sum";
/// The top gap's "?", under its dashed bracket near the strip's right end,
/// where chapter 03 put it.
const UNKNOWN: &str = "?";
/// The phrases of 06_3 that each step of reading the list backwards lands
/// on, in order: skip a gap, take a box, skip, take, skip the empty gap.
const READ_BACK: [&str; 5] = [
    "skip one box",
    "take the next, skip",
    "skip two",
    "take the next, and stop",
    "and stop",
];

/// The tiny table, as chapter 05 lays it out (every_handful.rs): row `j`'s
/// centre height, the squares' side, step, first centre and corner, the
/// seen squares' fill, the column numbers over the first row and their
/// size, and the note under the last row.
const TABLE_TOP: f64 = 3.0;
const ROW_STEP: f64 = 0.55;
const SQ: f64 = 0.34;
const SQ_STEP: f64 = 0.44;
const SQ_X0: f64 = -6.2;
const SQ_R: f64 = 0.04;
const SEEN_O: f64 = 0.9;
const HEAD_DY: f64 = 0.45;
const HEAD_SIZE: f64 = 20.0;
const EACH_DY: f64 = 0.6;
const EACH_SIZE: f64 = 22.0;
/// A gap square below the larger number: blue, a little lighter than a
/// seen square; a top gap's square: a hero outline with no fill.
const GAP_O: f64 = 0.6;
const TOP_LINE_W: f64 = 2.0;
/// The three gap columns right of the squares (where chapter 05 had its
/// estimates): the first column's centre, the step, the numbers' size; the
/// second note under the first; each column's sum and average under the
/// last row, with their labels right of the last column.
const LIST_X0: f64 = -3.5;
const LIST_STEP: f64 = 0.95;
const LIST_SIZE: f64 = 24.0;
const EACH_STEP: f64 = 0.4;
const SUM_DY: f64 = 0.6;
const AVERAGE_DY: f64 = 0.45;
const COLUMN_LABEL_X: f64 = -1.0;
/// The right-hand notes: their left edge and the lines at the top; the
/// matching's two-headed arrow (06_4, 07_2), the space either side of it,
/// its length, stroke and heads.
const RIGHT_X: f64 = 0.9;
const TOP_Y: [f64; 3] = [3.1, 2.6, 2.1];
pub const MATCH_GAP: f64 = 0.15;
pub const MATCH_ARROW: f64 = 0.55;
const MATCH_W: f64 = 3.0;
const MATCH_HEAD: f64 = 0.1;
const MATCH_SPREAD: f64 = 0.7;
/// Reading a list backwards (06_3): the list's copy, over five empty
/// squares, their side and step, and the box numbers under them.
const BUILD_X0: f64 = 1.6;
const BUILD_LIST_Y: f64 = 1.55;
const BUILD_Y: f64 = 0.9;
const BUILD_SQ: f64 = 0.45;
const BUILD_STEP: f64 = 0.6;
const BUILD_NUMBER_DY: f64 = 0.42;
const BUILD_S: f64 = 0.4;
/// The trade (06_5): the copy's height and its caption's left edge; how far
/// the two trading numbers lift off the line to pass each other, each step
/// of the move, and the slide to its partner's row.
const TRADE_Y: f64 = 0.9;
const CAPTION_X: f64 = 0.3;
const TRADE_LIFT: f64 = 0.45;
const TRADE_STEP_S: f64 = 0.5;
const SLIDE_S: f64 = 1.0;
/// Partners' halos (06_6), a pair at a time: quicker than a single halo,
/// so the ten rows fit the sentence that names them.
const PARTNER_IN_S: f64 = 0.25;
const PARTNER_HOLD_S: f64 = 0.35;
/// The columns' numbers in order (06_6, 06_7), one line each, and the even
/// share (06_8) under them.
const SORTED_Y: [f64; 3] = [0.9, 0.45, 0.0];
const SHARED_Y: f64 = -0.6;
/// The top gap's "?" under its bracket, and a gap's count under the line.
const TAIL_X: f64 = 5.4;
const UNDER_BRACKET: f64 = 0.35;
/// The inspection holds (06_4, 06_5, 06_8) and the beat after the last line.
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;

/// What chapter 06 shows, from the facts: the tiny company, its pairs and
/// each pair's gap list, how many unseen boxes and gaps a pair has, the
/// matching's flag and a list's chance; the two worked rows and their
/// lists; the first row's traded list and the pair whose list it is; every
/// row's partner, rows counted from 1; each column's values in order, its
/// sum and average, the top gap's average; and the friends' line, its first
/// number, the five numbers, their gaps, how many gaps and how many seen.
#[derive(Clone, Debug)]
struct Picture {
    boxes: u64,
    seen: usize,
    pairs: Vec<Vec<u64>>,
    lists: Vec<Vec<u64>>,
    unseen: u64,
    positions: usize,
    one_list_each: String,
    chance: String,
    low: Vec<u64>,
    low_gaps: Vec<u64>,
    high: Vec<u64>,
    high_gaps: Vec<u64>,
    low_traded: Vec<u64>,
    low_partner: Vec<u64>,
    partners: Vec<usize>,
    columns: Vec<Vec<u64>>,
    sums: Vec<u64>,
    averages: Vec<u64>,
    top_average: u64,
    first: u64,
    numbers: Vec<u64>,
    gaps: Vec<u64>,
    friends_positions: usize,
    friends_seen: usize,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    checked(Picture {
        boxes: whole(&f, "G-F5.boxes")?,
        seen: whole(&f, "G-F5.seen")? as usize,
        pairs: handfuls(&f, "G-F5.pair_list")?,
        lists: handfuls(&f, "G-F6.gap_lists")?,
        unseen: whole(&f, "G-F6.unseen")?,
        positions: whole(&f, "G-F6.positions")? as usize,
        one_list_each: printed(&f, "G-F6.one_pair_per_list")?,
        chance: printed(&f, "G-F6.each_chance")?,
        low: list(&f, "G-F5.first_pair")?,
        low_gaps: list(&f, "G-F6.low_gaps")?,
        high: list(&f, "G-F5.example_high_pair")?,
        high_gaps: list(&f, "G-F6.high_gaps")?,
        low_traded: list(&f, "G-F6.low_traded")?,
        low_partner: list(&f, "G-F6.low_partner")?,
        partners: list(&f, "G-F6.partners")?.into_iter().map(|p| p as usize).collect(),
        columns: vec![
            list(&f, "G-F6.below_values")?,
            list(&f, "G-F6.between_values")?,
            list(&f, "G-F6.above_values")?,
        ],
        sums: list(&f, "G-F6.column_sums")?,
        averages: list(&f, "G-F6.position_averages")?,
        top_average: whole(&f, "G-F6.top_average")?,
        first: whole(&f, "G-F1.first")?,
        numbers: list(&f, "G-F1.numbers")?,
        gaps: list(&f, "G-F3.gaps")?,
        friends_positions: whole(&f, "G-F6.friends_positions")? as usize,
        friends_seen: whole(&f, "G-F6.friends_seen_gaps")? as usize,
    })
}

/// A sorted handful's gap list up to `boxes`: how many boxes lie below its
/// first number, between each number and the next, and above its last. The
/// handful must be in order, from 1, its last at most `boxes`.
pub fn list_of(hand: &[u64], boxes: u64) -> Vec<u64> {
    let mut prev = 0;
    let mut out = Vec::new();
    for &x in hand {
        out.push(x - prev - 1);
        prev = x;
    }
    out.push(boxes - prev);
    out
}

/// A list with its first and last gaps, below and above, traded.
fn traded(l: &[u64]) -> Vec<u64> {
    let mut t = l.to_vec();
    if !t.is_empty() {
        let last = t.len() - 1;
        t.swap(0, last);
    }
    t
}

/// One step of reading a gap list backwards (06_3): skip gap `i`'s boxes
/// (none, for an empty gap), or take the next box.
#[derive(Clone, Debug, PartialEq)]
enum Step {
    Skip(usize, Vec<u64>),
    Take(u64),
}

/// A gap list read backwards from box 1: skip the first gap's boxes, take
/// the next, and so on, skipping the last gap's boxes at the end.
fn read_back(l: &[u64]) -> Vec<Step> {
    let mut next = 1;
    let mut out = Vec::new();
    for (i, &g) in l.iter().enumerate() {
        out.push(Step::Skip(i, (next..next + g).collect()));
        next += g;
        if i + 1 < l.len() {
            out.push(Step::Take(next));
            next += 1;
        }
    }
    out
}

/// The boxes a reading takes: the pair it rebuilds.
fn taken(steps: &[Step]) -> Vec<u64> {
    steps
        .iter()
        .filter_map(|s| match s {
            Step::Take(n) => Some(*n),
            Step::Skip(..) => None,
        })
        .collect()
}

/// The rows the partners' halos mark together (06_6), rows from 0: each row
/// with its partner, in the order of the first, then each row that is its
/// own partner, alone.
fn partner_groups(partners: &[usize]) -> Vec<Vec<usize>> {
    let mut pairs = Vec::new();
    let mut alone = Vec::new();
    for (j, &p) in partners.iter().enumerate() {
        match p.checked_sub(1) {
            Some(q) if q > j => pairs.push(vec![j, q]),
            Some(q) if q == j => alone.push(vec![j]),
            _ => {}
        }
    }
    pairs.extend(alone);
    pairs
}

/// The picture only if the scene can draw it truthfully: the table has
/// every pair once, in order, with three column heads for its gaps; each
/// row's list is its pair's gaps, adding to the unseen boxes; the lists are
/// all different and as many as the ways to split the unseen boxes into
/// the gaps, so each split is one list, and each list's chance is one
/// pair's; the worked rows' lists are their pairs', and the second read
/// backwards rebuilds it; the first row's traded list is the partner row's,
/// and every row's partner is the row its traded list is, each the other's;
/// the three columns hold the same values, their sums and averages are
/// the values' and an even share of the unseen boxes; the friends' five
/// are a handful on the line from 1, every gap below the largest non-empty,
/// with one more gap above.
fn checked(pic: Picture) -> Result<Picture, String> {
    if pic.pairs.len() as u128 != choose(pic.boxes, pic.seen as u64)
        || !pic.pairs.iter().all(|p| is_handful(p, pic.seen, pic.boxes))
        || pic.pairs.windows(2).any(|w| w[0] >= w[1])
        || pic.positions != pic.seen + 1
        || pic.positions != HEADS.len()
    {
        return Err(format!(
            "the pairs {:?} must be every pair from {} boxes once, in order, with {} gaps each",
            pic.pairs, pic.boxes, pic.positions
        ));
    }
    let own: Vec<Vec<u64>> = pic.pairs.iter().map(|p| list_of(p, pic.boxes)).collect();
    if pic.lists != own
        || pic.unseen + pic.seen as u64 != pic.boxes
        || own.iter().any(|l| l.iter().sum::<u64>() != pic.unseen)
    {
        return Err(format!(
            "the lists {:?} must be each pair's gaps, adding to {}",
            pic.lists, pic.unseen
        ));
    }
    let mut distinct = own.clone();
    distinct.sort();
    distinct.dedup();
    let splits = choose(pic.unseen + pic.positions as u64 - 1, pic.positions as u64 - 1);
    if distinct.len() != own.len()
        || distinct.len() as u128 != splits
        || pic.one_list_each != "true"
        || pic.chance != Q::new(1, pic.pairs.len() as i128).to_string()
    {
        return Err(format!(
            "the lists must be every split of {} into {} gaps once each, each one in {}, not {}",
            pic.unseen,
            pic.positions,
            pic.pairs.len(),
            pic.chance
        ));
    }
    let row = |p: &[u64]| pic.pairs.iter().position(|q| q.as_slice() == p);
    let high = read_back(&pic.high_gaps);
    if row(&pic.low) != Some(0)
        || pic.low_gaps != own[0]
        || row(&pic.high).map(|j| &own[j]) != Some(&pic.high_gaps)
        || taken(&high) != pic.high
        || high.len() != READ_BACK.len()
    {
        return Err(format!(
            "the worked rows {:?} and {:?} must have the lists {:?} and {:?}, the second read back into its pair",
            pic.low, pic.high, pic.low_gaps, pic.high_gaps
        ));
    }
    let partner_of: Vec<usize> = own
        .iter()
        .map(|l| own.iter().position(|m| *m == traded(l)).map_or(0, |j| j + 1))
        .collect();
    let n = pic.partners.len();
    if pic.low_traded != traded(&pic.low_gaps)
        || own.iter().position(|l| *l == pic.low_traded).map(|j| &pic.pairs[j]) != Some(&pic.low_partner)
        || pic.partners != partner_of
        || pic
            .partners
            .iter()
            .enumerate()
            .any(|(j, &p)| p == 0 || p > n || pic.partners[p - 1] != j + 1)
    {
        return Err(format!(
            "the trade: {:?} traded is {:?}, the list of {:?}; every row's partner {:?}, each the other's",
            pic.low_gaps, pic.low_traded, pic.low_partner, pic.partners
        ));
    }
    let columns: Vec<Vec<u64>> = (0..pic.positions)
        .map(|i| {
            let mut c: Vec<u64> = own.iter().map(|l| l[i]).collect();
            c.sort_unstable();
            c
        })
        .collect();
    let rows = pic.pairs.len() as u64;
    if pic.columns != columns
        || columns.windows(2).any(|w| w[0] != w[1])
        || pic.sums != columns.iter().map(|c| c.iter().sum()).collect::<Vec<u64>>()
        || pic.averages.len() != pic.positions
        || pic.sums.iter().zip(&pic.averages).any(|(&s, &a)| s != a * rows)
        || pic.averages.iter().any(|&a| a * pic.positions as u64 != pic.unseen)
        || pic.averages.last() != Some(&pic.top_average)
    {
        return Err(format!(
            "the columns {:?} must hold the same values, their sums {:?} and averages {:?} an even share",
            pic.columns, pic.sums, pic.averages
        ));
    }
    let sizes: Vec<u64> = gap_spans(&pic.numbers).iter().map(|&(a, b)| b - a + 1).collect();
    if pic.first != 1
        || !is_handful(&pic.numbers, pic.friends_seen, LINE_DOTS)
        || pic.gaps != sizes
        || pic.gaps.len() != pic.friends_seen
        || pic.friends_positions != pic.friends_seen + 1
    {
        return Err(format!(
            "the friends' {:?} must be {} numbers with the gaps {:?}, and {} gaps with the top one",
            pic.numbers, pic.friends_seen, pic.gaps, pic.friends_positions
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

/// 06_2's sum: the three gaps' names add to the unseen boxes.
fn total_line(unseen: u64) -> String {
    format!("{} + {} + {} = {unseen}", HEADS[0], HEADS[1], HEADS[2])
}

/// The gaps' names in their roles' colours: below and between blue, the
/// top gap hero.
fn head_colors() -> Vec<(String, Srgb)> {
    vec![
        (HEADS[0].to_owned(), palette::unseen()),
        (HEADS[1].to_owned(), palette::unseen()),
        (HEADS[2].to_owned(), palette::top_gap()),
    ]
}

/// The colour of gap column `i` of `positions`: the last is the top gap.
fn column_color(i: usize, positions: usize) -> Srgb {
    if i + 1 == positions {
        palette::top_gap()
    } else {
        palette::unseen()
    }
}

/// 06_4's line: the lists are every split, once each.
fn split_line(unseen: u64, positions: usize) -> String {
    format!("every split of {unseen} into {positions} gaps, once each")
}

/// The two notes under the table (05_2, 06_4).
fn each_line(what: &str, rows: usize) -> String {
    format!("each {what}: 1 in {rows}")
}

/// One column's values in order (06_6, 06_7).
fn column_line(head: &str, values: &[u64]) -> String {
    let shown: Vec<String> = values.iter().map(u64::to_string).collect();
    format!("{head}: {}", shown.join(", "))
}

/// The even share (06_8).
fn shared_line(unseen: u64, positions: usize, each: u64) -> String {
    format!("{unseen} unseen ÷ {positions} gaps = {each} each")
}

/// What the small company showed (06_10).
fn small_line(top_average: u64) -> String {
    format!("small company: each gap averaged {top_average}")
}

/// The average's label under the sums (06_8).
fn average_label(rows: usize) -> String {
    format!("average = sum ÷ {rows}")
}

/// A pair as the trade's caption names it (06_5).
fn pair_caption(pair: &[u64]) -> String {
    let shown: Vec<String> = pair.iter().map(u64::to_string).collect();
    format!("{}:", shown.join(", "))
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

/// Gap column `i`'s centre in the table.
fn list_x(i: usize) -> f64 {
    LIST_X0 + LIST_STEP * i as f64
}

/// Gap column `i`'s centre in a copy at the right (06_3, 06_5).
fn copy_x(i: usize) -> f64 {
    BUILD_X0 + LIST_STEP * i as f64
}

/// Box `n`'s centre among the squares a list is read back onto.
fn build_x(n: u64) -> f64 {
    BUILD_X0 + BUILD_STEP * (n as f64 - 1.0)
}

/// Where gap `a..=b`'s count sits on a line shown to `end`: under the line,
/// centred on its stretch (as chapter 03 places it).
pub fn count_spot(a: u64, b: u64, end: f64) -> Vec3 {
    v(
        (x_at(a as f64 - 0.5, end) + x_at(b as f64 + 0.5, end)) / 2.0,
        STRIP_Y - LABEL_DY,
    )
}

// ------------------------------------------------------- shared builders

/// Point at several things at once, each with its own halo in its own
/// colour: the halos come in together, stay a moment and go together.
pub fn glow_each(stage: &mut Stage<'_>, marks: &[(Vec<Mob>, Srgb)], in_s: f64, hold_s: f64) -> fmn::Result<()> {
    let mut halos = Vec::new();
    for (mobs, color) in marks {
        halos.push(halo(stage, mobs, *color)?);
    }
    fade_in_all(stage, &halos, in_s)?;
    pause(stage, hold_s)?;
    fade_out_all(stage, &halos, in_s)
}

/// A two-headed arrow along height `y` from `x0` to `x1`.
fn two_way(x0: f64, x1: f64, y: f64) -> Vec<Vec3> {
    let h = MATCH_HEAD.min((x1 - x0) / 2.0);
    let s = MATCH_SPREAD * h;
    vec![
        v(x0 + h, y + s),
        v(x0, y),
        v(x0 + h, y - s),
        v(x0, y),
        v(x1, y),
        v(x1 - h, y + s),
        v(x1, y),
        v(x1 - h, y - s),
    ]
}

/// A matching (06_4, 07_2): words, a two-headed arrow, words, each in its
/// colour, the first's left edge at `left`. Detached.
pub fn matched(
    kit: &Kit,
    stage: &mut Stage<'_>,
    from: (&str, Srgb),
    to: (&str, Srgb),
    left: Vec3,
) -> fmn::Result<Vec<Mob>> {
    let a = kit.text_left(stage, from.0, PANEL_SIZE, from.1, left)?;
    let x0 = left[0] + stage.get_width(a) + MATCH_GAP;
    let x1 = x0 + MATCH_ARROW;
    let arrow = stroke(
        stage,
        two_way(x0, x1, left[1]),
        line_style(Palette::ink(), MATCH_W, 1.0),
    );
    let b = kit.text_left(stage, to.0, PANEL_SIZE, to.1, v(x1 + MATCH_GAP, left[1]))?;
    Ok(vec![a, arrow, b])
}

/// The line from `first` to `last` with boxes on `numbers` and every gap
/// drawn, as chapter 06 leaves it and chapter 07 opens on: the line and the
/// boxes, each gap's arc below the largest and its count under the line,
/// the top gap's dashed bracket from the number after the largest off the
/// faded end, left to right, and its "?" under it.
pub struct GapLine {
    pub strip: FriendsLine,
    pub arcs: Vec<Mob>,
    pub counts: Vec<Mob>,
    pub dashes: Vec<Mob>,
    pub unknown: Mob,
}

impl GapLine {
    /// Build it, detached (the line's binding anchor in the scene).
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, first: u64, last: u64, numbers: &[u64]) -> fmn::Result<GapLine> {
        let end = last as f64;
        let strip = FriendsLine::build(
            kit,
            stage,
            first,
            last,
            numbers,
            &[],
            Start {
                end,
                fan: 0.0,
                rise: 1.0,
                widest: end,
            },
        )?;
        let (mut arcs, mut counts) = (Vec::new(), Vec::new());
        for (a, b) in gap_spans(numbers) {
            arcs.push(stroke(
                stage,
                arc_points(x_at(a as f64 - 0.5, end), x_at(b as f64 + 0.5, end), STRIP_Y, ARC_H),
                line_style(palette::unseen(), ARC_W, 1.0),
            ));
            counts.push(kit.text(
                stage,
                &(b - a + 1).to_string(),
                PANEL_SIZE,
                palette::unseen(),
                count_spot(a, b, end),
            )?);
        }
        let m = numbers.last().copied().unwrap_or(first);
        let y = STRIP_Y - BRACKET_DY;
        let dashes: Vec<Mob> = dash_spans(x_at(m as f64 + 1.0, end), STRIP_X1)
            .into_iter()
            .rev()
            .map(|(x0, x1)| {
                stroke(
                    stage,
                    vec![v(x0, y), v(x1, y)],
                    line_style(palette::top_gap(), BRACKET_W, 1.0),
                )
            })
            .collect();
        let unknown = kit.text(
            stage,
            UNKNOWN,
            PANEL_SIZE,
            palette::top_gap(),
            v(TAIL_X, y - UNDER_BRACKET),
        )?;
        Ok(GapLine {
            strip,
            arcs,
            counts,
            dashes,
            unknown,
        })
    }

    /// Everything, as chapter 06 leaves it.
    pub fn all(&self) -> Vec<Mob> {
        let mut all = self.strip.opening();
        all.extend(&self.arcs);
        all.extend(&self.counts);
        all.extend(&self.dashes);
        all.push(self.unknown);
        all
    }
}

// ------------------------------------------------------- chapter builders

/// A square of the tiny table at `at`, `side` wide, in `style`. Detached.
fn square(stage: &mut Stage<'_>, at: Vec3, side: f64, style: Style) -> fmn::Result<Mob> {
    let sq = Rectangle::new()
        .width(side)
        .height(side)
        .corner_radius(SQ_R)
        .style(style)
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(sq))
}

/// A gap list's numbers, gap column `i` at `x(i)`, on height `y`, each in
/// its column's colour. Detached.
fn list_texts(kit: &Kit, stage: &mut Stage<'_>, l: &[u64], x: fn(usize) -> f64, y: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for (i, g) in l.iter().enumerate() {
        out.push(kit.text(stage, &g.to_string(), LIST_SIZE, column_color(i, l.len()), v(x(i), y))?);
    }
    Ok(out)
}

/// The tiny table: the column numbers; each row's five squares, the pair's
/// two gold (chapter 05's); every row's unseen squares in row order, and
/// the gap squares that replace them; each row's squares once replaced; the
/// gap columns' heads and each row's list; the two notes under the table;
/// each column's sum and average and their labels. Detached.
struct Table {
    head: Vec<Mob>,
    rows: Vec<Vec<Mob>>,
    unseen: Vec<Mob>,
    gap_squares: Vec<Mob>,
    shown: Vec<Vec<Mob>>,
    heads: Vec<Mob>,
    lists: Vec<Vec<Mob>>,
    each_pair: Mob,
    each_list: Mob,
    sums: Vec<Mob>,
    sum_label: Mob,
    averages: Vec<Mob>,
    average_label: Mob,
}

impl Table {
    fn build(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<Table> {
        let head_y = TABLE_TOP + HEAD_DY;
        let mut head = Vec::new();
        for n in 1..=pic.boxes {
            head.push(kit.text(stage, &n.to_string(), HEAD_SIZE, Palette::dim(), v(sq_x(n), head_y))?);
        }
        let (mut rows, mut unseen, mut gap_squares, mut shown) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (j, pair) in pic.pairs.iter().enumerate() {
            let larger = pair.last().copied().unwrap_or(0);
            let (mut row, mut now) = (Vec::new(), Vec::new());
            for n in 1..=pic.boxes {
                let at = v(sq_x(n), table_y(j));
                if pair.contains(&n) {
                    let sq = square(stage, at, SQ, flat(palette::seen(), SEEN_O))?;
                    row.push(sq);
                    now.push(sq);
                    continue;
                }
                let sq = square(stage, at, SQ, flat(palette::outline(), BODY_O))?;
                let gap_style = if n < larger {
                    flat(palette::unseen(), GAP_O)
                } else {
                    line_style(palette::top_gap(), TOP_LINE_W, 1.0)
                };
                let gap = square(stage, at, SQ, gap_style)?;
                row.push(sq);
                unseen.push(sq);
                gap_squares.push(gap);
                now.push(gap);
            }
            rows.push(row);
            shown.push(now);
        }
        let mut heads = Vec::new();
        for (i, h) in HEADS.iter().enumerate() {
            heads.push(kit.text(stage, h, HEAD_SIZE, column_color(i, HEADS.len()), v(list_x(i), head_y))?);
        }
        let mut lists = Vec::new();
        for (j, l) in pic.lists.iter().enumerate() {
            lists.push(list_texts(kit, stage, l, list_x, table_y(j))?);
        }
        let last = pic.pairs.len().saturating_sub(1);
        let note_left = SQ_X0 - SQ / 2.0;
        let each_pair = kit.text_left(
            stage,
            &each_line(PAIR_WORD, pic.pairs.len()),
            EACH_SIZE,
            Palette::dim(),
            v(note_left, table_y(last) - EACH_DY),
        )?;
        let each_list = kit.text_left(
            stage,
            &each_line("list", pic.pairs.len()),
            EACH_SIZE,
            Palette::dim(),
            v(note_left, table_y(last) - EACH_DY - EACH_STEP),
        )?;
        let sum_y = table_y(last) - SUM_DY;
        let average_y = sum_y - AVERAGE_DY;
        let (mut sums, mut averages) = (Vec::new(), Vec::new());
        for i in 0..pic.positions {
            let color = column_color(i, pic.positions);
            sums.push(kit.text(stage, &pic.sums[i].to_string(), LIST_SIZE, color, v(list_x(i), sum_y))?);
            averages.push(kit.text(
                stage,
                &pic.averages[i].to_string(),
                LIST_SIZE,
                color,
                v(list_x(i), average_y),
            )?);
        }
        let sum_label = kit.text_left(stage, SUM_WORD, HEAD_SIZE, Palette::dim(), v(COLUMN_LABEL_X, sum_y))?;
        let average_label = kit.text_left(
            stage,
            &average_label(pic.pairs.len()),
            HEAD_SIZE,
            Palette::dim(),
            v(COLUMN_LABEL_X, average_y),
        )?;
        Ok(Table {
            head,
            rows,
            unseen,
            gap_squares,
            shown,
            heads,
            lists,
            each_pair,
            each_list,
            sums,
            sum_label,
            averages,
            average_label,
        })
    }
}

/// Reading a list backwards (06_3): the list's copy at the right, five
/// empty squares under it with their box numbers, and each box's square
/// filled as it is read, gold where it is taken and blue where it is
/// skipped. Detached.
struct Builder {
    list: Vec<Mob>,
    empty: Vec<Mob>,
    numbers: Vec<Mob>,
    filled: Vec<Mob>,
}

impl Builder {
    fn build(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture) -> fmn::Result<Builder> {
        let list = list_texts(kit, stage, &pic.high_gaps, copy_x, BUILD_LIST_Y)?;
        let (mut empty, mut numbers, mut filled) = (Vec::new(), Vec::new(), Vec::new());
        for n in 1..=pic.boxes {
            let at = v(build_x(n), BUILD_Y);
            empty.push(square(
                stage,
                at,
                BUILD_SQ,
                line_style(palette::outline(), TOP_LINE_W, 1.0),
            )?);
            let fill = if pic.high.contains(&n) {
                flat(palette::seen(), SEEN_O)
            } else {
                flat(palette::unseen(), GAP_O)
            };
            filled.push(square(stage, at, BUILD_SQ, fill)?);
            numbers.push(kit.text(
                stage,
                &n.to_string(),
                HEAD_SIZE,
                Palette::dim(),
                v(build_x(n), BUILD_Y - BUILD_NUMBER_DY),
            )?);
        }
        Ok(Builder {
            list,
            empty,
            numbers,
            filled,
        })
    }

    /// The squares of boxes `which`, from `set`.
    fn of(set: &[Mob], which: &[u64]) -> Vec<Mob> {
        which.iter().map(|&n| set[(n - 1) as usize]).collect()
    }
}

/// Trade two numbers on height `y`, `a` at `xa` and `b` at `xb`: `a` lifts
/// and `b` drops, they slide past each other (and past anything between
/// them on the line) and settle in each other's places.
fn trade(stage: &mut Stage<'_>, (a, xa): (Mob, f64), (b, xb): (Mob, f64), y: f64) -> fmn::Result<()> {
    let (up, down) = (y + TRADE_LIFT, y - TRADE_LIFT);
    move_all(stage, &[(a, v(xa, up)), (b, v(xb, down))], 1.0, TRADE_STEP_S)?;
    move_all(stage, &[(a, v(xb, up)), (b, v(xa, down))], 1.0, TRADE_STEP_S)?;
    move_all(stage, &[(a, v(xb, y)), (b, v(xa, y))], 1.0, TRADE_STEP_S)
}

impl SceneConstruct for GapsTradePlaces {
    fn name(&self) -> &str {
        "06_gaps_trade_places"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let row_of = |pair: &[u64]| pic.pairs.iter().position(|p| p == pair).unwrap_or(0);
        let (low_row, high_row) = (row_of(pic.low.as_slice()), row_of(pic.high.as_slice()));
        let partner_row = row_of(pic.low_partner.as_slice());
        let top = pic.positions - 1;
        let table = Table::build(&kit, stage, &pic)?;

        // chapter 05's table, square for square, and its note
        let mut opened = table.head.clone();
        opened.extend(table.rows.iter().flatten());
        opened.push(table.each_pair);
        fade_in_all(stage, &opened, 0.8)?;

        // 06_1: every row's unseen squares become its gaps; the three
        // columns' heads, each as it is named
        let line = say(&kit, stage, "06_1")?;
        line.wait_for(stage, "Each pair leaves")?;
        replace(stage, &table.unseen, &table.gap_squares, 0.6)?;
        for (&h, phrase) in table
            .heads
            .iter()
            .zip(["below the smaller", "between the two", "above the larger"])
        {
            line.wait_for(stage, phrase)?;
            fade_in_all(stage, &[h], 0.3)?;
        }

        // 06_2: the two worked rows, a halo round each and its gaps; the
        // other eight rows' gaps; they always add up to the unseen boxes
        let line = say(&kit, stage, "06_2")?;
        line.wait_for(stage, "One and two leave")?;
        glow(stage, &table.shown[low_row], palette::seen())?;
        line.wait_for(stage, "zero, zero and three")?;
        fade_in_lagged(stage, &table.lists[low_row], 0.6, 0.3)?;
        line.wait_for(stage, "Two and five leave")?;
        glow(stage, &table.shown[high_row], palette::seen())?;
        line.wait_for(stage, "one, two and zero")?;
        fade_in_lagged(stage, &table.lists[high_row], 0.6, 0.3)?;
        line.wait_for(stage, "Every pair has")?;
        let rest: Vec<Mob> = (0..pic.pairs.len())
            .filter(|&j| j != low_row && j != high_row)
            .flat_map(|j| table.lists[j].iter().copied())
            .collect();
        fade_in_lagged(stage, &rest, 1.6, 0.1)?;
        line.wait_for(stage, "add up to three")?;
        let total = words_left(
            &kit,
            stage,
            &total_line(pic.unseen),
            &head_colors(),
            v(RIGHT_X, TOP_Y[0]),
        )?;
        fade_in_all(stage, &[total], 0.5)?;

        // 06_3: the second worked row's list read backwards on five empty
        // squares, gap by gap, a halo round the gap being read; then halos
        // compare the row it builds with the table's
        let line = say(&kit, stage, "06_3")?;
        let build = Builder::build(&kit, stage, &pic)?;
        let mut start = build.list.clone();
        start.extend(&build.empty);
        start.extend(&build.numbers);
        fade_in_all(stage, &start, 0.5)?;
        let mut lit_halo: Vec<Mob> = Vec::new();
        for (step, phrase) in read_back(&pic.high_gaps).iter().zip(READ_BACK) {
            line.wait_for(stage, phrase)?;
            let (boxes, gap) = match step {
                Step::Skip(i, boxes) => (boxes.clone(), Some(*i)),
                Step::Take(n) => (vec![*n], None),
            };
            // the previous gap's halo goes as this step's squares fill
            let mut out = Builder::of(&build.empty, &boxes);
            out.append(&mut lit_halo);
            let mut into = Builder::of(&build.filled, &boxes);
            if let Some(i) = gap {
                let h = halo(stage, &[build.list[i]], column_color(i, pic.positions))?;
                into.push(h);
                lit_halo.push(h);
            }
            replace(stage, &out, &into, BUILD_S)?;
        }
        line.wait_for(stage, "That rebuilds")?;
        fade_out_all(stage, &lit_halo, HALO_IN_S)?;
        glow_each(
            stage,
            &[
                (build.filled.clone(), palette::seen()),
                (table.shown[high_row].clone(), palette::seen()),
            ],
            HALO_IN_S,
            HALO_HOLD_S,
        )?;

        // 06_4: the built row goes; the matching, then every split once
        // and a halo round the lists; each pair's chance, then each
        // list's; held to be read
        let line = say(&kit, stage, "06_4")?;
        let mut built = build.list.clone();
        built.extend(&build.filled);
        built.extend(&build.numbers);
        fade_out_all(stage, &built, 0.4)?;
        line.wait_for(stage, "match up exactly")?;
        let matching = matched(
            &kit,
            stage,
            (PAIR_WORD, palette::seen()),
            (GAP_LIST, palette::unseen()),
            v(RIGHT_X, TOP_Y[1]),
        )?;
        fade_in_all(stage, &matching, 0.5)?;
        line.wait_for(stage, "every way to split")?;
        let every = words_left(
            &kit,
            stage,
            &split_line(pic.unseen, pic.positions),
            &[],
            v(RIGHT_X, TOP_Y[2]),
        )?;
        fade_in_all(stage, &[every], 0.5)?;
        let numbers: Vec<Mob> = table.lists.iter().flatten().copied().collect();
        glow(stage, &numbers, Palette::ink())?;
        line.wait_for(stage, "each pair is one in ten")?;
        glow(stage, &[table.each_pair], Palette::ink())?;
        line.wait_for(stage, "so is each list")?;
        fade_in_all(stage, &[table.each_list], 0.4)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 06_5: the right side clears; the first row's list written again
        // at the right, its gaps below and above trading places; the traded
        // list beside the row whose list it is; held to be compared
        let line = say(&kit, stage, "06_5")?;
        let mut notes = vec![total, every];
        notes.extend(&matching);
        fade_out_all(stage, &notes, 0.4)?;
        line.wait_for(stage, "trade places")?;
        let mut first_row = table.shown[low_row].clone();
        first_row.extend(&table.lists[low_row]);
        glow(stage, &first_row, palette::seen())?;
        let copy = list_texts(&kit, stage, &pic.low_gaps, copy_x, TRADE_Y)?;
        let caption = kit.text_left(
            stage,
            &pair_caption(&pic.low),
            LIST_SIZE,
            palette::seen(),
            v(CAPTION_X, TRADE_Y),
        )?;
        let mut written = vec![caption];
        written.extend(&copy);
        fade_in_all(stage, &written, 0.4)?;
        line.wait_for(stage, "zero, zero and three becomes")?;
        trade(stage, (copy[0], copy_x(0)), (copy[top], copy_x(top)), TRADE_Y)?;
        line.wait_for(stage, "That's the list for four and five")?;
        // after the trade the copy reads its numbers in traded order
        let in_place: Vec<Mob> = (0..pic.positions)
            .map(|i| match i {
                0 => copy[top],
                i if i == top => copy[0],
                i => copy[i],
            })
            .collect();
        let slides: Vec<(Mob, Vec3)> = in_place
            .iter()
            .enumerate()
            .map(|(i, &m)| (m, v(copy_x(i), table_y(partner_row))))
            .collect();
        fade_out_all(stage, &[caption], 0.3)?;
        move_all(stage, &slides, 1.0, SLIDE_S)?;
        let mut partner = table.shown[partner_row].clone();
        partner.extend(&table.lists[partner_row]);
        let traded_halo = halo(stage, &in_place, Palette::ink())?;
        let partner_halo = halo(stage, &partner, Palette::ink())?;
        fade_in_all(stage, &[traded_halo, partner_halo], HALO_IN_S)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;
        fade_out_all(stage, &[traded_halo, partner_halo], HALO_IN_S)?;

        // 06_6: the traded list goes; every row and its partner, a pair of
        // halos at a time, then the rows that are their own partners; the
        // below and above columns' values in order
        let line = say(&kit, stage, "06_6")?;
        fade_out_all(stage, &copy, 0.4)?;
        line.wait_for(stage, "Every row has a partner")?;
        for group in partner_groups(&pic.partners) {
            let marks: Vec<(Vec<Mob>, Srgb)> = group
                .iter()
                .map(|&j| (table.lists[j].clone(), Palette::ink()))
                .collect();
            glow_each(stage, &marks, PARTNER_IN_S, PARTNER_HOLD_S)?;
        }
        // below, then above, then between (06_7)
        let order = [0, top, 1];
        let mut sorted = Vec::new();
        for (&i, &y) in order.iter().zip(&SORTED_Y) {
            sorted.push(kit.text_left(
                stage,
                &column_line(HEADS[i], &pic.columns[i]),
                PANEL_SIZE,
                column_color(i, pic.positions),
                v(RIGHT_X, y),
            )?);
        }
        line.wait_for(stage, "So the below column")?;
        fade_in_all(stage, &[sorted[0]], 0.5)?;
        line.wait_for(stage, "and the above column")?;
        fade_in_all(stage, &[sorted[1]], 0.5)?;

        // 06_7: the between column's values too; a halo round the three
        let line = say(&kit, stage, "06_7")?;
        line.wait_for(stage, "Trading the gap between")?;
        fade_in_all(stage, &[sorted[2]], 0.5)?;
        line.wait_for(stage, "all three columns")?;
        glow(stage, &sorted, Palette::ink())?;

        // 06_8: each column's sum, then its average, the top gap's in hero;
        // the unseen boxes shared evenly by the gaps; a halo round the top
        // gap's average; held to be read
        let line = say(&kit, stage, "06_8")?;
        line.wait_for(stage, "Add up each column")?;
        let mut sums = table.sums.clone();
        sums.push(table.sum_label);
        fade_in_all(stage, &sums, 0.5)?;
        line.wait_for(stage, "so each gap averages")?;
        let mut averages = table.averages.clone();
        averages.push(table.average_label);
        fade_in_all(stage, &averages, 0.5)?;
        line.wait_for(stage, "Three unseen boxes")?;
        let shared = words_left(
            &kit,
            stage,
            &shared_line(pic.unseen, pic.positions, pic.top_average),
            &[("unseen".to_owned(), palette::unseen())],
            v(RIGHT_X, SHARED_Y),
        )?;
        fade_in_all(stage, &[shared], 0.5)?;
        line.wait_for(stage, "That includes the top gap")?;
        glow(stage, &[table.averages[top]], palette::top_gap())?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 06_9: everything fades; the friends' line as chapter 04 drew it
        // and the badge; the five gaps' arcs and counts, a halo round the
        // arcs; the countdown, with nothing above sixty yet
        let line = say(&kit, stage, "06_9")?;
        fade_all(stage, 0.6)?;
        let gaps = GapLine::build(&kit, stage, pic.first, LINE_DOTS, &pic.numbers)?;
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let mut back = gaps.strip.opening();
        back.push(corner);
        fade_in_all(stage, &back, 0.8)?;
        line.wait_for(stage, "into six gaps")?;
        let drawn: Vec<Mob> = gaps.arcs.iter().zip(&gaps.counts).flat_map(|(&a, &c)| [a, c]).collect();
        fade_in_lagged(stage, &drawn, 1.0, 0.2)?;
        line.wait_for(stage, "only five arcs")?;
        glow(stage, &gaps.arcs, palette::unseen())?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 06_10: the countdown goes; the top gap's dashed bracket from 61
        // off the faded end and its "?"; a halo round it; what the small
        // company showed; a halo round the gaps below it
        let line = say(&kit, stage, "06_10")?;
        fade_out_all(stage, &[q], 0.3)?;
        fade_in_lagged(stage, &gaps.dashes, 0.8, 0.2)?;
        fade_in_all(stage, &[gaps.unknown], 0.4)?;
        line.wait_for(stage, "It's the top gap")?;
        glow(stage, &[gaps.unknown], palette::top_gap())?;
        line.wait_for(stage, "In the small company")?;
        let small = words_left(&kit, stage, &small_line(pic.top_average), &[], panel(0))?;
        fade_in_all(stage, &[small], 0.5)?;
        line.wait_for(stage, "just like the gaps below it")?;
        glow(stage, &gaps.arcs, palette::unseen())?;
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
    use crate::chapters::draw::{BOX_H, BOX_RISE, SMALL, THINK_R};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, PANEL_X, PANEL_Y, est_width};
    use crate::chapters::what_we_assume::{HALO_PAD, halo_frame};
    use crate::facts::{TINY, TINY_SEEN, gap_list};
    use crate::model::{FRIENDS, all_handfuls, gaps, splits};

    /// A line's half height at the panel's size, at the table's sizes, and
    /// the space kept between things that must not touch.
    const HALF_LINE: f64 = 0.16;
    const HALF_HEAD: f64 = 0.12;
    const HALF_NUMBER: f64 = 0.14;
    const CLEAR: f64 = 0.1;

    fn good() -> Picture {
        picture().unwrap()
    }

    /// The scene draws the facts: the pairs are the model's, each row's
    /// list is the facts' gap list with its top gap; the lists are the
    /// model's splits; the words read as the voice says them.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!((pic.boxes, pic.seen), (TINY, TINY_SEEN));
        assert_eq!(pic.pairs, all_handfuls(TINY, TINY_SEEN));
        for (p, l) in pic.pairs.iter().zip(&pic.lists) {
            assert_eq!(*l, gap_list(p, TINY), "{p:?}");
        }
        let mut sorted = pic.lists.clone();
        sorted.sort();
        assert_eq!(sorted, splits(pic.unseen, pic.positions));
        assert_eq!(total_line(pic.unseen), "below + between + above = 3");
        assert_eq!(
            split_line(pic.unseen, pic.positions),
            "every split of 3 into 3 gaps, once each"
        );
        assert_eq!(each_line(PAIR_WORD, pic.pairs.len()), "each pair: 1 in 10");
        assert_eq!(each_line("list", pic.pairs.len()), "each list: 1 in 10");
        assert_eq!(
            column_line(HEADS[0], &pic.columns[0]),
            "below: 0, 0, 0, 0, 1, 1, 1, 2, 2, 3"
        );
        assert_eq!(
            shared_line(pic.unseen, pic.positions, pic.top_average),
            "3 unseen ÷ 3 gaps = 1 each"
        );
        assert_eq!(small_line(pic.top_average), "small company: each gap averaged 1");
        assert_eq!(average_label(pic.pairs.len()), "average = sum ÷ 10");
        assert_eq!(pair_caption(&pic.low), "1, 2:");
        assert_eq!(pic.gaps, gaps(&FRIENDS));
        // the trade's copy, read in its places after the trade, is the
        // partner row's list
        assert_eq!(traded(&pic.low_gaps), pic.lists[pic.partners[0] - 1]);
    }

    /// Reading a list backwards: skip, take, skip, take, skip, one step
    /// per phrase, rebuilding the pair; an empty last gap is a step with no
    /// boxes. Every list of the table reads back into its own pair.
    #[test]
    fn a_list_read_backwards_rebuilds_its_pair() {
        let pic = good();
        assert_eq!(
            read_back(&pic.high_gaps),
            [
                Step::Skip(0, vec![1]),
                Step::Take(2),
                Step::Skip(1, vec![3, 4]),
                Step::Take(5),
                Step::Skip(2, vec![]),
            ]
        );
        for (p, l) in pic.pairs.iter().zip(&pic.lists) {
            assert_eq!(taken(&read_back(l)), *p, "{l:?}");
        }
        // the friends' gaps with their top gap, to any count, read back too
        for n in [60, 71, 84] {
            assert_eq!(taken(&read_back(&list_of(&FRIENDS, n))), FRIENDS);
        }
        assert_eq!(list_of(&[1, 2], 5), [0, 0, 3]);
        assert_eq!(list_of(&[2, 5], 5), [1, 2, 0]);
    }

    /// The partners: every row's traded list is another row's, each the
    /// other's; the halos mark the four pairs of rows, then the two rows
    /// that are their own partners.
    #[test]
    fn every_row_has_a_partner() {
        let pic = good();
        let groups = partner_groups(&pic.partners);
        assert_eq!(
            groups,
            [vec![0, 9], vec![1, 8], vec![2, 6], vec![4, 7], vec![3], vec![5]]
        );
        let mut marked: Vec<usize> = groups.iter().flatten().copied().collect();
        marked.sort_unstable();
        assert_eq!(marked, (0..pic.pairs.len()).collect::<Vec<_>>(), "every row once");
        for g in &groups {
            let l = &pic.lists[g[0]];
            assert_eq!(traded(l), pic.lists[*g.last().unwrap()]);
            assert_eq!(
                g.len() == 1,
                l[0] == l[l.len() - 1],
                "alone exactly when below equals above"
            );
        }
        // partners that are not each the other's would mark a row twice and
        // leave a row's partner never pointing back (checked() refuses them)
        assert_eq!(partner_groups(&[2, 3, 1]), [vec![0, 1], vec![1, 2]]);
        assert_eq!(traded(&[]), Vec::<u64>::new());
    }

    /// The trade is fair only because every pair is: for every small
    /// company (handfuls of 1 to 4 from up to 9 boxes), trading any two
    /// gap positions maps the lists onto the lists, so every position has
    /// the same values and averages (N − k) ÷ (k + 1). The negative control:
    /// weighting each pair of the tiny company by its larger number, the
    /// top gap averages 3/4 and the gap below 9/8.
    #[test]
    fn trading_gaps_keeps_the_lists_when_every_handful_is_as_likely() {
        for k in 1..=4usize {
            for n in k as u64..=9 {
                let lists: Vec<Vec<u64>> = all_handfuls(n, k).iter().map(|h| list_of(h, n)).collect();
                let mut sorted = lists.clone();
                sorted.sort();
                for i in 0..=k {
                    for j in i + 1..=k {
                        let mut swapped: Vec<Vec<u64>> = lists
                            .iter()
                            .map(|l| {
                                let mut t = l.clone();
                                t.swap(i, j);
                                t
                            })
                            .collect();
                        swapped.sort();
                        assert_eq!(swapped, sorted, "k {k}, n {n}: positions {i} and {j}");
                    }
                    let sum: u64 = lists.iter().map(|l| l[i]).sum();
                    assert_eq!(
                        Q::new(sum as i128, lists.len() as i128),
                        Q::new((n - k as u64) as i128, k as i128 + 1),
                        "k {k}, n {n}: position {i}"
                    );
                }
            }
        }
        let pic = good();
        let weight = |p: &[u64]| p[p.len() - 1] as i128;
        let total: i128 = pic.pairs.iter().map(|p| weight(p.as_slice())).sum();
        let weighted = |i: usize| -> Q {
            let s: i128 = pic
                .pairs
                .iter()
                .zip(&pic.lists)
                .map(|(p, l)| weight(p.as_slice()) * l[i] as i128)
                .sum();
            Q::new(s, total)
        };
        assert_eq!(weighted(pic.positions - 1), Q::new(3, 4));
        assert_eq!(weighted(0), Q::new(9, 8));
        assert_ne!(weighted(0), weighted(pic.positions - 1));
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
        let mut swapped = pic.pairs.clone();
        swapped.swap(0, 1);
        refused(
            Picture {
                pairs: swapped,
                ..pic.clone()
            },
            "every pair",
        );
        refused(
            Picture {
                positions: 4,
                ..pic.clone()
            },
            "every pair",
        );
        // a box moved from one gap to the next: the sum still holds, but
        // it is not the pair's list
        let mut moved = pic.lists.clone();
        moved[1] = vec![1, 0, 2];
        refused(
            Picture {
                lists: moved,
                ..pic.clone()
            },
            "each pair's gaps",
        );
        refused(
            Picture {
                unseen: 4,
                ..pic.clone()
            },
            "each pair's gaps",
        );
        refused(
            Picture {
                one_list_each: "false".to_owned(),
                ..pic.clone()
            },
            "every split",
        );
        refused(
            Picture {
                chance: "1/9".to_owned(),
                ..pic.clone()
            },
            "every split",
        );
        refused(
            Picture {
                low: pic.pairs[1].clone(),
                ..pic.clone()
            },
            "worked rows",
        );
        refused(
            Picture {
                high_gaps: vec![2, 1, 0],
                ..pic.clone()
            },
            "worked rows",
        );
        refused(
            Picture {
                low_traded: pic.low_gaps.clone(),
                ..pic.clone()
            },
            "the trade",
        );
        refused(
            Picture {
                low_partner: pic.pairs[8].clone(),
                ..pic.clone()
            },
            "the trade",
        );
        let mut crossed = pic.partners.clone();
        crossed.swap(0, 1);
        refused(
            Picture {
                partners: crossed,
                ..pic.clone()
            },
            "the trade",
        );
        let mut columns = pic.columns.clone();
        columns[2] = vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 3];
        refused(Picture { columns, ..pic.clone() }, "the columns");
        refused(
            Picture {
                averages: vec![1, 1, 2],
                top_average: 2,
                ..pic.clone()
            },
            "the columns",
        );
        refused(
            Picture {
                top_average: 0,
                ..pic.clone()
            },
            "the columns",
        );
        refused(
            Picture {
                gaps: vec![13, 12, 5, 17, 9],
                ..pic.clone()
            },
            "friends'",
        );
        refused(
            Picture {
                friends_positions: pic.friends_seen,
                ..pic
            },
            "friends'",
        );
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let mut back: Vec<&str> = READ_BACK.to_vec();
        back.push("That rebuilds");
        let waits: &[(&str, &[&str])] = &[
            (
                "06_1",
                &[
                    "Each pair leaves",
                    "below the smaller",
                    "between the two",
                    "above the larger",
                ],
            ),
            (
                "06_2",
                &[
                    "One and two leave",
                    "zero, zero and three",
                    "Two and five leave",
                    "one, two and zero",
                    "Every pair has",
                    "add up to three",
                ],
            ),
            ("06_3", back.as_slice()),
            (
                "06_4",
                &[
                    "match up exactly",
                    "every way to split",
                    "each pair is one in ten",
                    "so is each list",
                ],
            ),
            (
                "06_5",
                &[
                    "trade places",
                    "zero, zero and three becomes",
                    "That's the list for four and five",
                ],
            ),
            (
                "06_6",
                &["Every row has a partner", "So the below column", "and the above column"],
            ),
            ("06_7", &["Trading the gap between", "all three columns"]),
            (
                "06_8",
                &[
                    "Add up each column",
                    "so each gap averages",
                    "Three unseen boxes",
                    "That includes the top gap",
                ],
            ),
            ("06_9", &["into six gaps", "only five arcs"]),
            (
                "06_10",
                &[
                    "It's the top gap",
                    "In the small company",
                    "just like the gaps below it",
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

    /// The table: the column numbers and the heads under the frame's top;
    /// the gap columns right of the squares, their heads apart; the notes,
    /// the sums and the averages under the last row and over the frame's
    /// foot, the notes left of the columns, the labels right of them.
    #[test]
    fn the_table_fits_the_frame() {
        let pic = good();
        let last = pic.pairs.len() - 1;
        let safe_x = FRAME_X_RADIUS - 0.25;
        const { assert!(TABLE_TOP + HEAD_DY + HALF_HEAD < FRAME_Y_RADIUS - 0.15) };
        const { assert!(SQ + CLEAR / 2.0 < SQ_STEP && SQ + CLEAR < ROW_STEP) };
        let widths: Vec<f64> = HEADS.iter().map(|h| est_width(h, HEAD_SIZE)).collect();
        assert!(list_x(0) - widths[0] / 2.0 > sq_x(pic.boxes) + SQ / 2.0 + CLEAR);
        for i in 1..HEADS.len() {
            assert!(
                list_x(i) - list_x(i - 1) > (widths[i] + widths[i - 1]) / 2.0 + CLEAR,
                "{i}"
            );
        }
        let average_y = table_y(last) - SUM_DY - AVERAGE_DY;
        assert!(average_y - HALF_NUMBER > -FRAME_Y_RADIUS + 0.15);
        assert!(table_y(last) - SUM_DY + HALF_NUMBER < table_y(last) - HALF_NUMBER);
        let note_y = table_y(last) - EACH_DY - EACH_STEP;
        assert!(note_y - HALF_HEAD > -FRAME_Y_RADIUS + 0.15);
        const { assert!(EACH_STEP > 2.0 * HALF_HEAD + 0.05) };
        let note_w = est_width(&each_line(PAIR_WORD, pic.pairs.len()), EACH_SIZE);
        assert!(SQ_X0 - SQ / 2.0 + note_w < list_x(0) - HALF_NUMBER - CLEAR);
        let last_col = list_x(pic.positions - 1) + est_width(&pic.sums[0].to_string(), LIST_SIZE) / 2.0;
        assert!(COLUMN_LABEL_X > last_col + CLEAR);
        // the average's label, the longer one, on the frame and under the
        // even share at the right
        assert!(COLUMN_LABEL_X + est_width(&average_label(pic.pairs.len()), HEAD_SIZE) < safe_x);
        assert!(average_y + HALF_HEAD < SHARED_Y - HALF_LINE - CLEAR);
        // the halo round every list stays on the frame
        let lists: Vec<([f64; 2], [f64; 2])> = (0..pic.positions)
            .flat_map(|i| {
                [0, last].map(|j| {
                    let (x, y) = (list_x(i), table_y(j));
                    ([x - HALF_NUMBER, y - HALF_NUMBER], [x + HALF_NUMBER, y + HALF_NUMBER])
                })
            })
            .collect();
        let (c, _, h) = halo_frame(&lists).unwrap();
        assert!(
            c[1] + h / 2.0 < TABLE_TOP + HEAD_DY - HALF_HEAD,
            "the halo under the heads"
        );
        // neighbouring rows' halos keep apart (the partners' halos)
        const { assert!(2.0 * (HALF_NUMBER + HALO_PAD) < ROW_STEP) };
    }

    /// The right-hand side: every note and column line in the title-safe
    /// area, the lines apart; the list read backwards on its squares under
    /// the notes, its squares apart; the trade's copy clear of its caption,
    /// its numbers passing the middle one without touching; the slide's
    /// row right of the table's own list.
    #[test]
    fn the_right_side_fits_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let match_w =
            est_width(PAIR_WORD, PANEL_SIZE) + 2.0 * MATCH_GAP + MATCH_ARROW + est_width(GAP_LIST, PANEL_SIZE);
        let mut widths = vec![
            est_width(&total_line(pic.unseen), PANEL_SIZE),
            match_w,
            est_width(&split_line(pic.unseen, pic.positions), PANEL_SIZE),
            est_width(&shared_line(pic.unseen, pic.positions, pic.top_average), PANEL_SIZE),
        ];
        for (i, h) in HEADS.iter().enumerate() {
            widths.push(est_width(&column_line(h, &pic.columns[i]), PANEL_SIZE));
        }
        for w in widths {
            assert!(RIGHT_X + w < safe_x, "{w}");
        }
        for w in TOP_Y.windows(2) {
            assert!(w[0] - w[1] > 2.0 * HALF_LINE + CLEAR);
        }
        for w in SORTED_Y.windows(2) {
            assert!(w[0] - w[1] > 2.0 * HALF_LINE + CLEAR);
        }
        const { assert!(SORTED_Y[2] - HALF_LINE > SHARED_Y + HALF_LINE + CLEAR) };
        // the builder: its list under the notes, its squares apart, its
        // box numbers under them, all on the frame
        const { assert!(BUILD_LIST_Y + HALF_NUMBER < TOP_Y[2] - HALF_LINE - CLEAR) };
        const { assert!(BUILD_Y + BUILD_SQ / 2.0 < BUILD_LIST_Y - HALF_NUMBER - CLEAR) };
        const { assert!(BUILD_STEP > BUILD_SQ + CLEAR) };
        assert!(build_x(pic.boxes) + BUILD_SQ / 2.0 < safe_x);
        const { assert!(BUILD_Y - BUILD_NUMBER_DY + HALF_HEAD < BUILD_Y - BUILD_SQ / 2.0) };
        assert!(copy_x(pic.positions - 1) + HALF_NUMBER < safe_x);
        // the trade: the caption left of the copy; the trading numbers
        // pass over and under the middle one
        let caption_w = est_width(&pair_caption(&pic.low), LIST_SIZE);
        assert!(CAPTION_X + caption_w < copy_x(0) - HALF_NUMBER - CLEAR);
        const { assert!(TRADE_LIFT > 2.0 * HALF_NUMBER + 0.05) };
        // the copy lands beside the partner's row, right of its list
        let partner = pic.pairs.iter().position(|p| *p == pic.low_partner).unwrap();
        assert!(table_y(partner) - HALF_NUMBER > -FRAME_Y_RADIUS + 0.15);
        assert!(copy_x(0) - HALF_NUMBER - HALO_PAD > COLUMN_LABEL_X + CLEAR);
    }

    /// The friends' line: the counts under the line keep apart and clear
    /// the boxes; the "?" sits under its dashes; the badge and the small
    /// company's line keep apart; the countdown's ring clears the boxes.
    #[test]
    fn the_friends_line_fits_the_frame() {
        let pic = good();
        let end = LINE_DOTS as f64;
        let spans = gap_spans(&pic.numbers);
        let spots: Vec<(f64, f64)> = spans
            .iter()
            .map(|&(a, b)| {
                (
                    count_spot(a, b, end)[0],
                    est_width(&(b - a + 1).to_string(), PANEL_SIZE),
                )
            })
            .collect();
        for w in spots.windows(2) {
            assert!(w[1].0 - w[0].0 > (w[0].1 + w[1].1) / 2.0 + 0.3);
        }
        const { assert!(STRIP_Y - LABEL_DY + 0.2 < STRIP_Y + BOX_RISE - SMALL * BOX_H / 2.0) };
        // the "?" under the bracket, which runs from the number after the
        // largest to the strip's end
        let m = *pic.numbers.last().unwrap() as f64;
        assert!(x_at(m + 1.0, end) < TAIL_X - CLEAR && TAIL_X + CLEAR < STRIP_X1);
        assert!(!dash_spans(x_at(m + 1.0, end), STRIP_X1).is_empty());
        const { assert!(STRIP_Y - BRACKET_DY - UNDER_BRACKET - HALF_LINE > -FRAME_Y_RADIUS + 0.15) };
        let safe_x = FRAME_X_RADIUS - 0.25;
        assert!(PANEL_X + est_width(&small_line(pic.top_average), PANEL_SIZE) < safe_x);
        let badge_right = BADGE_AT[0] + est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0];
        assert!(badge_right < PANEL_X - CLEAR, "the badge left of the panel's line");
        const { assert!(PANEL_Y[0] + HALF_LINE < FRAME_Y_RADIUS - 0.15) };
        const { assert!(THINK_AT[1] - THINK_R > STRIP_Y + BOX_RISE + SMALL * BOX_H / 2.0) };
    }
}
