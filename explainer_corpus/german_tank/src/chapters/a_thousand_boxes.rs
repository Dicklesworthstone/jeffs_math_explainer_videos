//! Chapter 02, "If there were a thousand": the friends' line from chapter
//! 01 pours into a bin, every dot still a box, and five come out blindly,
//! the fair handful, its condition kept as a badge (02_1 to 02_3); the
//! prediction, with the line shown to a thousand (02_4); ten computed
//! handfuls from a thousand, every one reaching past six hundred, and the
//! stretch 1 to 60 that none of them fits (02_5, 02_6); ten from seventy,
//! some of which fit, nearly half of all of its handfuls (02_7); the
//! spacing of a row from each count and of the friends' own five, counted
//! by a live readout (02_8); the clue, and what no handful can tell (02_9).
//!
//! The line, the boxes, the bin's dots, the rows and the arrows are bound
//! to trackers through `draw::x_at`, so every zoom moves what is already
//! there; each zoom plays `draw::zoom_legs`, the boxes spreading before the
//! line squeezes and coming home after it stretches. Every number on the
//! screen is a fact, and the rows are the facts' own computed draws.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    DOT_OPACITY, LABEL_DY, LABEL_SIZE, Leg, STRIP_Y, Spoken, dot_opacity, fade_in_all, fade_in_lagged, fade_out_all,
    fail, flash_all, line_style, pause, play_legs, refill, say, stroke, think, wait_until, x_at, zoom_legs,
};
use super::friends_line::{
    ARROW_UNDER_LINE, ARROW_UNDER_ROW, BADGE_AT, FriendsLine, PANEL_SIZE, Rows, Start, THINK_AT, Trackers, at, badge,
    bin, drop_in_all, handfuls, is_handful, list, now, panel, printed, read_s, row_y, spacing_arrows, value, whole,
};
use crate::facts;
use crate::model::{self, LINE_DOTS};
use crate::palette;

pub struct AThousandBoxes {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FAIR: &str = "fair handful";
const SPACING: &str = "average spacing";
/// The spacing panel's exact averages are over every handful of a count;
/// the readout under them counts one row.
const ALL_FROM: &str = "all handfuls from";
const READOUT_WORDS: &str = "arrows' length ÷";

/// The live readout (02_8): its number's right edge, and the words over
/// it; their sizes.
const READOUT_AT: [f64; 2] = [5.6, -1.95];
const READOUT_WORDS_AT: [f64; 2] = [4.6, -1.5];
const READOUT_SIZE: f64 = 34.0;
const READOUT_WORDS_SIZE: f64 = 22.0;
/// An upright mark across the line (600, 70): its stroke, how far it
/// reaches past the line or the top row; the outline round the stretch 1
/// to 60, from just under the first row to just over the last (teal, the
/// certain role: strokes only, never a fill).
const MARK_W: f64 = 2.0;
const MARK_O: f64 = 0.6;
const MARK_REACH: f64 = 0.1;
const STRETCH_W: f64 = 2.0;
const STRETCH_O: f64 = 0.8;
const BAND_PAD: f64 = 0.1;
/// The pour, the shake and the settled bin's beat; the handful's rise out
/// of the bin and everything's way back; a box spreading into its row and
/// the line's zoom under it; a row landing and its largest number's flash;
/// the arrows' run; the beat after the last line.
const POUR_S: f64 = 2.4;
const SHAKE_S: f64 = 1.2;
const SETTLE_S: f64 = 0.6;
const LIFT_S: f64 = 1.0;
const BACK_S: f64 = 1.6;
const FAN_S: f64 = 0.7;
const ZOOM_S: f64 = 2.0;
const ROW_S: f64 = 0.3;
const ROW_FLASH_S: f64 = 0.25;
const ARROW_S: f64 = 1.8;
const END_BEAT: f64 = 0.6;

/// What chapter 02 shows, from the facts: the friends' five (and so the
/// largest, the handful size and the line's first number), the two counts
/// and the zoomed line's marks, the bin and the handful that leaves it,
/// the ten computed handfuls from each count, the mark every row from a
/// thousand reaches past, the two exact chances and the three spacings.
#[derive(Clone, Debug)]
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    largest: u64,
    k: usize,
    big: u64,
    small: u64,
    marks: Vec<u64>,
    bin_dots: u64,
    bin_handful: Vec<u64>,
    big_hands: Vec<Vec<u64>>,
    small_hands: Vec<Vec<u64>>,
    over: u64,
    /// `G-F2.big_all_at_most_60_under_one_in_a_million`, as printed.
    rare: String,
    small_percent: u64,
    /// How many of the rows from the small count fit inside 1 to the
    /// friends' largest, as 02_7 says ("Three of these ten do").
    fit_shown: usize,
    big_spacing: String,
    small_spacing: String,
    friends_spacing: String,
    /// The first row of each count's average hop, as the facts print it:
    /// where the readout ends.
    big_row: String,
    small_row: String,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    checked(Picture {
        numbers: list(&f, "G-F1.numbers")?,
        first: whole(&f, "G-F1.first")?,
        largest: whole(&f, "G-F1.largest")?,
        k: whole(&f, "G-F1.handful")? as usize,
        big: whole(&f, "G-F2.big")?,
        small: whole(&f, "G-F2.small")?,
        marks: list(&f, "G-F1.zoom_marks")?,
        bin_dots: whole(&f, "G-F2.bin_dots")?,
        bin_handful: list(&f, "G-F2.bin_handful")?,
        big_hands: handfuls(&f, "G-F2.big_handfuls")?,
        small_hands: handfuls(&f, "G-F2.small_handfuls")?,
        over: whole(&f, "G-F2.over")?,
        rare: printed(&f, "G-F2.big_all_at_most_60_under_one_in_a_million")?,
        small_percent: whole(&f, "G-F2.small_all_at_most_60.percent")?,
        fit_shown: whole(&f, "G-F2.small_fit_shown")? as usize,
        big_spacing: printed(&f, "G-F2.big_spacing_shown")?,
        small_spacing: printed(&f, "G-F2.small_spacing_shown")?,
        friends_spacing: printed(&f, "G-F2.friends_spacing")?,
        big_row: printed(&f, "G-F2.big_row_spacing")?,
        small_row: printed(&f, "G-F2.small_row_spacing")?,
    })
}

/// A handful that stays inside the stretch 1 to `largest`.
fn fits(hand: &[u64], largest: u64) -> bool {
    hand.last().is_some_and(|&b| b <= largest)
}

/// A handful's average hop from zero (its largest number over its size),
/// as the readout shows it.
fn average_hop(hand: &[u64]) -> f64 {
    hand.last().map_or(0.0, |&b| b as f64 / hand.len() as f64)
}

/// The picture only if the scene can draw it truthfully and the words say
/// what it shows: the friends' five and the bin's five are handfuls on the
/// opening line; the line zooms out past it to the count whose marks it
/// shows; the small count's mark is on the opening line, past the friends'
/// largest; every row is a handful from its count; every row from the big
/// count reaches past the mark, which is past the friends' largest (so no
/// such row fits inside 1 to 60), and the chance of fitting is under one
/// in a million, as 02_5 and 02_6 say; as many rows from the small count
/// fit as 02_7 says, some and not all (it flashes the ones that do); the
/// readout ends on each row's average hop as the facts print it.
fn checked(pic: Picture) -> Result<Picture, String> {
    if !is_handful(&pic.numbers, pic.k, LINE_DOTS) || pic.numbers.last() != Some(&pic.largest) || pic.first != 1 {
        return Err(format!(
            "the friends' {:?} must be {} numbers in order on the line from 1, the largest {}",
            pic.numbers, pic.k, pic.largest
        ));
    }
    if pic.big <= LINE_DOTS || pic.marks != model::marks(pic.first, pic.big, model::ZOOM_PARTS)? {
        return Err(format!(
            "the zoom to {} must pass the opening line and show its own marks",
            pic.big
        ));
    }
    if pic.small >= LINE_DOTS || pic.small <= pic.largest {
        return Err(format!(
            "the small count {} must lie between the friends' largest and the opening line's end",
            pic.small
        ));
    }
    if pic.bin_dots != LINE_DOTS || !is_handful(&pic.bin_handful, pic.k, pic.bin_dots) {
        return Err(format!(
            "the bin's handful {:?} must come from its {} dots",
            pic.bin_handful, pic.bin_dots
        ));
    }
    let rows_ok = |hands: &[Vec<u64>], n: u64| !hands.is_empty() && hands.iter().all(|h| is_handful(h, pic.k, n));
    if !rows_ok(&pic.big_hands, pic.big)
        || !rows_ok(&pic.small_hands, pic.small)
        || pic.big_hands.len() != pic.small_hands.len()
    {
        return Err("each count's rows must be as many handfuls from that count".to_owned());
    }
    if pic.over <= pic.largest
        || pic.over >= pic.big
        || !pic.big_hands.iter().all(|h| h.last().is_some_and(|&b| b > pic.over))
    {
        return Err(format!("every row from {} must reach past {}", pic.big, pic.over));
    }
    if pic.rare != "true" {
        return Err("fitting inside 1 to 60 must be under one in a million, as 02_6 says".to_owned());
    }
    let fit = pic.small_hands.iter().filter(|h| fits(h, pic.largest)).count();
    if fit != pic.fit_shown || fit == 0 || fit == pic.small_hands.len() {
        return Err(format!(
            "{fit} of the {} rows from {} fit, but 02_7 says {} do: some, not all",
            pic.small_hands.len(),
            pic.small,
            pic.fit_shown
        ));
    }
    for s in [&pic.big_spacing, &pic.small_spacing, &pic.friends_spacing] {
        value(s)?;
    }
    for (hand, shown) in [
        (&pic.big_hands[0], &pic.big_row),
        (&pic.small_hands[0], &pic.small_row),
        (&pic.numbers, &pic.friends_spacing),
    ] {
        if (average_hop(hand) - value(shown)?).abs() > 0.05 {
            return Err(format!(
                "the readout for {hand:?} ends at {} but the facts print {shown}",
                average_hop(hand)
            ));
        }
    }
    Ok(pic)
}

/// Card B, the exact version of the fair handful (02_2).
fn card_b(pic: &Picture) -> String {
    format!(
        "the count N is fixed and unknown; the handful is random: every set of {} numbers from {} to N \
         is equally likely (a uniform draw without replacement). Equal chances for each box are not enough.",
        pic.k, pic.first
    )
}

/// A mark's number under the line, at number `n` on a line shown to
/// `end`. Detached.
fn mark_label(kit: &Kit, stage: &mut Stage<'_>, n: u64, end: f64) -> fmn::Result<Mob> {
    kit.text(
        stage,
        &thousands(n),
        LABEL_SIZE,
        palette::outline(),
        v(x_at(n as f64, end), STRIP_Y - LABEL_DY),
    )
}

/// An upright mark at number `n` on a line shown to `end`, from just under
/// the line up to `top`, and its number under the line. Detached.
fn upright(kit: &Kit, stage: &mut Stage<'_>, n: u64, end: f64, top: f64) -> fmn::Result<[Mob; 2]> {
    let x = x_at(n as f64, end);
    let mark = stroke(
        stage,
        vec![v(x, STRIP_Y - MARK_REACH), v(x, top)],
        line_style(palette::outline(), MARK_W, MARK_O),
    );
    Ok([mark, mark_label(kit, stage, n, end)?])
}

/// The rows' band: from just under the first row to just over row
/// `rows - 1`.
fn band(rows: usize) -> [f64; 2] {
    [row_y(0) - BAND_PAD, row_y(rows.saturating_sub(1)) + BAND_PAD]
}

/// The stretch from `first` to `largest` outlined across the rows' band,
/// on a line shown to `end`. Detached.
fn stretch(stage: &mut Stage<'_>, first: u64, largest: u64, end: f64, rows: usize) -> fmn::Result<Mob> {
    let (x0, x1) = (x_at(first as f64 - 0.5, end), x_at(largest as f64 + 0.5, end));
    let [y0, y1] = band(rows);
    let r = Rectangle::new()
        .width(x1 - x0)
        .height(y1 - y0)
        .style(line_style(palette::certain(), STRETCH_W, STRETCH_O))
        .build()?
        .moved_to(v((x0 + x1) / 2.0, (y0 + y1) / 2.0));
    Ok(stage.arena_mut().add(r))
}

/// Play the legs from the view `from` to the view `to` (each `[end, fan]`).
fn zoom(stage: &mut Stage<'_>, t: Trackers, from: [f64; 2], to: [f64; 2]) -> fmn::Result<()> {
    play_legs(stage, t.end, t.fan, &zoom_legs(from, to), FAN_S, ZOOM_S)
}

/// As [`zoom`], the line's end waiting for `phrase` of `line` (the boxes
/// spread first, while the words lead up to it).
fn zoom_on(
    stage: &mut Stage<'_>,
    t: Trackers,
    from: [f64; 2],
    to: [f64; 2],
    line: &Spoken,
    phrase: &str,
) -> fmn::Result<()> {
    for leg in zoom_legs(from, to) {
        if matches!(leg, Leg::End(_)) {
            line.wait_for(stage, phrase)?;
        }
        play_legs(stage, t.end, t.fan, &[leg], FAN_S, ZOOM_S)?;
    }
    Ok(())
}

/// `hand`'s spacing arrows at height `y`, run from zero to its largest
/// number in `ARROW_S` while the `readout` tracker counts their length so
/// far, divided by the handful's size, up from zero: both tweens share a
/// run time and a rate, so the readout is the arrows' length over the size
/// on every frame and ends on the handful's average hop.
fn run_hops(stage: &mut Stage<'_>, hand: &[u64], y: f64, end: Mob, readout: Mob) -> fmn::Result<Vec<Mob>> {
    let tip = stage.add_value_tracker(0.0);
    let arrows = spacing_arrows(stage, hand, y, tip, end)?;
    for &a in &arrows {
        stage.add_to_scene(a)?;
    }
    if stage.tracker_value(readout).unwrap_or(0.0) > 0.0 {
        stage.play(tween(readout, 0.0, 0.3)?)?;
    }
    let last = hand.last().map_or(0.0, |&b| b as f64);
    play!(stage; tween(tip, last, ARROW_S)?, tween(readout, average_hop(hand), ARROW_S)?);
    Ok(arrows)
}

impl SceneConstruct for AThousandBoxes {
    fn name(&self) -> &str {
        "02_a_thousand_boxes"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let (opening, far) = (LINE_DOTS as f64, pic.big as f64);
        let (near_view, far_view) = ([opening, 0.0], [far, 1.0]);
        let rows = pic.big_hands.len();
        let ys: Vec<f64> = (0..rows).map(row_y).collect();
        let top = band(rows)[1];

        // chapter 01's line, its boxes on their numbers and their ticks
        let strip = FriendsLine::build(
            &kit,
            stage,
            pic.first,
            LINE_DOTS,
            &pic.numbers,
            &pic.bin_handful,
            Start {
                end: opening,
                fan: 0.0,
                rise: 1.0,
                widest: far,
            },
        )?;
        let t = strip.t;
        for &m in &strip.base {
            stage.add_to_scene(m)?;
        }
        fade_in_all(stage, &strip.opening(), 0.8)?;
        let mut boxes = strip.bodies.clone();
        boxes.extend(&strip.labels);
        boxes.extend(&strip.ticks);
        let others: Vec<(u64, Mob)> = (pic.first..=LINE_DOTS)
            .zip(strip.dots.iter().copied())
            .filter(|(n, _)| !pic.numbers.contains(n))
            .collect();

        // 02_1: every box in one bin: the boxes step aside, the dots pour
        // in, each still a box (all at one strength now, none faded), and
        // the bin is shaken
        let line = say(&kit, stage, "02_1")?;
        line.wait_for(stage, "Imagine every box")?;
        let bin = bin(stage);
        fade_out_all(stage, &boxes, 0.4)?;
        fade_in_all(stage, &[bin], 0.4)?;
        let even: Vec<(Mob, Srgb, f64)> = others.iter().map(|&(_, d)| (d, palette::line(), DOT_OPACITY)).collect();
        refill(stage, &even, 0.3)?;
        stage.play(tween(t.pour, 1.0, POUR_S)?)?;
        line.wait_for(stage, "shaken well")?;
        stage.play(tween(t.shake, 1.0, SHAKE_S)?)?;
        pause(stage, SETTLE_S)?;

        // 02_2: five come out blindly and line up in gold outlines; any five
        // as likely as any other; the fair handful, and its exact statement
        let line = say(&kit, stage, "02_2")?;
        line.wait_for(stage, "come out blindly")?;
        fade_in_all(stage, &strip.rings, 0.3)?;
        stage.play(tween(t.lift, 1.0, LIFT_S)?)?;
        line.wait_for(stage, "as likely as any other five")?;
        flash_all(stage, &strip.rings, 0.6)?;
        line.wait_for(stage, "Call that a fair handful")?;
        let fair = kit.text_left(stage, FAIR, PANEL_SIZE, Palette::ink(), panel(0))?;
        let card_text = card_b(&pic);
        let card = precisely(&kit, stage, &card_text)?;
        fade_in_all(stage, &[fair, card], 0.4)?;
        let card_up = now(stage);

        // 02_3: an assumption: the panel's words become the badge, which
        // stays; everything goes back onto the line
        let line = say(&kit, stage, "02_3")?;
        line.wait_for(stage, "this is an assumption")?;
        let badge = badge(&kit, stage, at(BADGE_AT))?;
        fade_out_all(stage, &[fair], 0.3)?;
        fade_in_all(stage, &[badge], 0.4)?;
        line.wait_for(stage, "Later we'll break it")?;
        play!(stage; tween(t.lift, 0.0, BACK_S)?, tween(t.pour, 0.0, BACK_S)?);
        let restored: Vec<(Mob, Srgb, f64)> = others
            .iter()
            .map(|&(n, d)| (d, palette::line(), dot_opacity(x_at(n as f64, opening))))
            .collect();
        refill(stage, &restored, 0.3)?;
        let mut gone = strip.rings.clone();
        gone.push(bin);
        fade_out_all(stage, &gone, 0.4)?;
        fade_in_all(stage, &boxes, 0.4)?;
        // card B stays until it can be read, then makes way for the question
        wait_until(stage, card_up + read_s(&card_text))?;
        fade_out_all(stage, &[card], 0.4)?;

        // 02_4: the prediction: the boxes spread, the line shows a thousand
        // under them, and its marks
        let line = say(&kit, stage, "02_4")?;
        line.wait_for(stage, "If the company")?;
        zoom_on(stage, t, near_view, far_view, &line, "a thousand boxes")?;
        let mut marks = Vec::new();
        for &n in pic.marks.iter().chain([pic.big].iter()) {
            marks.push(mark_label(&kit, stage, n, far)?);
        }
        fade_in_lagged(stage, &marks, 0.8, 0.2)?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 02_5: all over the line: ten computed handfuls land, each row's
        // largest number flashing as it lands; the mark they all pass
        let line = say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[q], 0.2)?;
        line.wait_for(stage, "Here are ten")?;
        let big_rows = Rows::bind(stage, &pic.big_hands, &ys, t.end, far)?;
        for row in &big_rows.rings {
            drop_in_all(stage, row, ROW_S)?;
            if let Some(&largest) = row.last() {
                flash_all(stage, &[largest], ROW_FLASH_S)?;
            }
        }
        line.wait_for(stage, "past six hundred")?;
        let over = upright(&kit, stage, pic.over, far, top)?;
        fade_in_all(stage, &over, 0.4)?;
        let largest: Vec<Mob> = big_rows.rings.iter().filter_map(|r| r.last().copied()).collect();
        flash_all(stage, &largest, 0.6)?;

        // 02_6: the stretch 1 to 60 outlined across every row, which none
        // fits; the exact chance in the panel
        let line = say(&kit, stage, "02_6")?;
        line.wait_for(stage, "sixty or less")?;
        let stretch_far = stretch(stage, pic.first, pic.largest, far, rows)?;
        fade_in_all(stage, &[stretch_far], 0.5)?;
        line.wait_for(stage, "fewer than one handful")?;
        let rule = kit.text_left(
            stage,
            &format!("all {} at most {}", pic.k, pic.largest),
            PANEL_SIZE,
            Palette::ink(),
            panel(0),
        )?;
        let rare = kit.text_left(
            stage,
            &format!("from {}: fewer than one in a million", thousands(pic.big)),
            PANEL_SIZE,
            Palette::ink(),
            panel(1),
        )?;
        fade_in_lagged(stage, &[rule, rare], 0.8, 0.3)?;

        // 02_7: a company of seventy: the rows go, the line comes back in
        // under the boxes with its end mark at seventy, and ten rows from
        // seventy land; the exact share of all its handfuls, then the ones
        // of these ten inside 1 to 60 flash
        let line = say(&kit, stage, "02_7")?;
        line.wait_for(stage, "Now try a company")?;
        let mut gone = big_rows.all();
        gone.extend(over);
        gone.push(stretch_far);
        gone.extend(&marks);
        fade_out_all(stage, &gone, 0.5)?;
        zoom(stage, t, far_view, near_view)?;
        let seventy = upright(&kit, stage, pic.small, opening, STRIP_Y + MARK_REACH)?;
        fade_in_all(stage, &seventy, 0.4)?;
        line.wait_for(stage, "Its handfuls look")?;
        let small_rows = Rows::bind(stage, &pic.small_hands, &ys, t.end, opening)?;
        for row in &small_rows.rings {
            drop_in_all(stage, row, ROW_S)?;
        }
        line.wait_for(stage, "Nearly half")?;
        let stretch_near = stretch(stage, pic.first, pic.largest, opening, rows)?;
        let share = kit.text_left(
            stage,
            &format!("from {}: {}% of all handfuls", pic.small, pic.small_percent),
            PANEL_SIZE,
            Palette::ink(),
            panel(2),
        )?;
        fade_in_all(stage, &[stretch_near, share], 0.5)?;
        line.wait_for(stage, "Three of these ten")?;
        let fitting: Vec<Mob> = small_rows
            .rings
            .iter()
            .zip(&pic.small_hands)
            .filter(|(_, h)| fits(h, pic.largest))
            .flat_map(|(r, _)| r.iter().copied())
            .collect();
        flash_all(stage, &fitting, 0.7)?;

        // 02_8: the spacing: blue arrows from zero to each number, a row
        // from a thousand (the line out again), a row from seventy, then
        // the friends' own five; the readout counts the arrows' length over
        // five, and the panel gives the exact averages
        let line = say(&kit, stage, "02_8")?;
        let mut gone = small_rows.all();
        gone.push(stretch_near);
        gone.extend(seventy);
        gone.extend([rule, rare, share]);
        fade_out_all(stage, &gone, 0.5)?;
        line.wait_for(stage, "Numbers from a thousand")?;
        zoom(stage, t, near_view, far_view)?;
        let head = kit.text_left(stage, SPACING, PANEL_SIZE, Palette::ink(), panel(0))?;
        let words = kit.text(
            stage,
            &format!("{READOUT_WORDS} {}", pic.k),
            READOUT_WORDS_SIZE,
            palette::unseen(),
            at(READOUT_WORDS_AT),
        )?;
        fade_in_all(stage, &[head, words], 0.4)?;
        let counted = stage.add_value_tracker(0.0);
        let readout = kit.live_number(
            stage,
            counted,
            1,
            false,
            READOUT_SIZE,
            palette::unseen(),
            at(READOUT_AT),
            RIGHT,
        )?;
        let big_row = Rows::bind(stage, &pic.big_hands[..1], &ys[..1], t.end, far)?;
        drop_in_all(stage, &big_row.all(), ROW_S)?;
        let big_arrows = run_hops(stage, &pic.big_hands[0], ys[0] - ARROW_UNDER_ROW, t.end, counted)?;
        let from_big = kit.text_left(
            stage,
            &format!("{ALL_FROM} {}: {}", thousands(pic.big), pic.big_spacing),
            PANEL_SIZE,
            palette::unseen(),
            panel(1),
        )?;
        fade_in_all(stage, &[from_big], 0.4)?;
        line.wait_for(stage, "From seventy")?;
        let mut gone = big_row.all();
        gone.extend(&big_arrows);
        fade_out_all(stage, &gone, 0.4)?;
        zoom(stage, t, far_view, near_view)?;
        let small_row = Rows::bind(stage, &pic.small_hands[..1], &ys[..1], t.end, opening)?;
        drop_in_all(stage, &small_row.all(), ROW_S)?;
        let small_arrows = run_hops(stage, &pic.small_hands[0], ys[0] - ARROW_UNDER_ROW, t.end, counted)?;
        let from_small = kit.text_left(
            stage,
            &format!("{ALL_FROM} {}: {}", pic.small, pic.small_spacing),
            PANEL_SIZE,
            palette::unseen(),
            panel(2),
        )?;
        fade_in_all(stage, &[from_small], 0.4)?;
        line.wait_for(stage, "just like the friends'")?;
        let friends_arrows = run_hops(stage, &pic.numbers, STRIP_Y - ARROW_UNDER_LINE, t.end, counted)?;
        let friends = kit.text_left(
            stage,
            &format!("the friends' handful: {}", pic.friends_spacing),
            PANEL_SIZE,
            palette::unseen(),
            panel(3),
        )?;
        fade_in_all(stage, &[friends], 0.4)?;

        // 02_9: the drawn row goes; the friends' five keep their arrows, the
        // clue; the badge, what no handful can tell
        let line = say(&kit, stage, "02_9")?;
        let mut gone = small_row.all();
        gone.extend(&small_arrows);
        gone.extend([readout, words]);
        fade_out_all(stage, &gone, 0.4)?;
        line.wait_for(stage, "their spacing is a clue")?;
        flash_all(stage, &friends_arrows, 0.6)?;
        line.wait_for(stage, "whether the handful was fair")?;
        flash_all(stage, &[badge], 0.7)?;
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
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{BOX_H, BOX_RISE, SMALL, STRIP_X0, THINK_R, all_clear, box_spot, leg_views};
    use crate::chapters::friends_line::{
        ARROW_HEAD, ARROW_SPREAD, BADGE, BADGE_PAD, BADGE_SIZE, BIN_AT, BIN_H, BIN_W, PANEL_X, PANEL_Y, RING_R,
        est_width,
    };
    use crate::model::{BIG, FRIENDS};

    /// The rings of one row are this far apart at least, outline to
    /// outline.
    const RING_GAP: f64 = 0.02;
    /// Characters a line of a "precisely:" card holds beside its label: the
    /// kit starts at 92 and narrows by 4 until the widest line fits, which
    /// only rendered glyphs settle (the lint measures); the card must fit
    /// at every width down to 64.
    const CARD_CHARS_WIDEST: usize = 92;
    const CARD_CHARS_NARROWEST: usize = 64;

    fn good() -> Picture {
        picture().unwrap()
    }

    /// The scene draws the facts: the friends' five, the oracle's computed
    /// rows (ten from each count, as many as the facts print), the bin's
    /// handful, the marks, and the readout's three ends.
    #[test]
    fn the_picture_is_the_facts() {
        let f = facts::facts().unwrap();
        let pic = good();
        assert_eq!(pic.numbers, FRIENDS);
        assert_eq!(pic.big, BIG);
        assert_eq!(pic.big_hands.len().to_string(), f.get("G-F2.draws").unwrap());
        assert_eq!(pic.small_hands.len(), pic.big_hands.len());
        assert_eq!(pic.big_hands, facts::big_handfuls());
        assert_eq!(pic.small_hands, facts::small_handfuls());
        assert_eq!(pic.bin_handful, facts::bin_handful());
        assert_eq!(pic.marks, model::marks(1, BIG, model::ZOOM_PARTS).unwrap());
        // the readout shows one decimal: the printed facts, digit for digit
        assert_eq!(format!("{:.1}", average_hop(&pic.big_hands[0])), pic.big_row);
        assert_eq!(format!("{:.1}", average_hop(&pic.small_hands[0])), pic.small_row);
        assert_eq!(average_hop(&pic.numbers), value(&pic.friends_spacing).unwrap());
    }

    /// The negative controls: each picture the words would contradict is
    /// refused, one wrong fact at a time, for that fact's reason.
    #[test]
    fn a_picture_the_words_contradict_is_refused() {
        let pic = good();
        let refused = |p: Picture, why: &str| {
            let e = checked(p).expect_err(why);
            assert!(e.contains(why), "refused for {e:?}, not {why:?}");
        };
        assert!(checked(pic.clone()).is_ok());
        let mut short = pic.clone();
        short.big_hands[3] = vec![2, 9, 40, 300, 590];
        refused(short, "reach past");
        refused(
            Picture {
                over: 960,
                ..pic.clone()
            },
            "reach past",
        );
        refused(
            Picture {
                rare: "false".to_owned(),
                ..pic.clone()
            },
            "one in a million",
        );
        refused(
            Picture {
                marks: vec![1, 200, 400, 600],
                ..pic.clone()
            },
            "marks",
        );
        refused(
            Picture {
                bin_handful: vec![10, 13, 13, 61, 83],
                ..pic.clone()
            },
            "bin",
        );
        refused(
            Picture {
                small: 90,
                ..pic.clone()
            },
            "small count",
        );
        refused(
            Picture {
                big_row: "190.7".to_owned(),
                ..pic.clone()
            },
            "readout",
        );
        // rows that fit as 02_7 says, but none or all of them; then the
        // words against the rows the facts drew
        let says = "but 02_7 says";
        for hand in [vec![2, 30, 41, 61, 65], vec![7, 12, 17, 26, 35]] {
            let mut rows = pic.clone();
            rows.small_hands = rows.small_hands.iter().map(|_| hand.clone()).collect();
            rows.fit_shown = rows.small_hands.iter().filter(|h| fits(h, rows.largest)).count();
            refused(rows, says);
        }
        refused(
            Picture {
                fit_shown: pic.fit_shown + 1,
                ..pic.clone()
            },
            says,
        );
        refused(
            Picture {
                small_hands: pic.small_hands[..3].to_vec(),
                ..pic
            },
            "rows must be",
        );
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            ("02_1", &["Imagine every box", "shaken well"]),
            (
                "02_2",
                &[
                    "come out blindly",
                    "as likely as any other five",
                    "Call that a fair handful",
                ],
            ),
            ("02_3", &["this is an assumption", "Later we'll break it"]),
            ("02_4", &["If the company", "a thousand boxes"]),
            ("02_5", &["Here are ten", "past six hundred"]),
            ("02_6", &["sixty or less", "fewer than one handful"]),
            (
                "02_7",
                &[
                    "Now try a company",
                    "Its handfuls look",
                    "Nearly half",
                    "Three of these ten",
                ],
            ),
            (
                "02_8",
                &["Numbers from a thousand", "From seventy", "just like the friends'"],
            ),
            ("02_9", &["their spacing is a clue", "whether the handful was fair"]),
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

    /// The chapter's views, in order: the opening line, a thousand (02_4),
    /// back in (02_7), out (02_8) and in again. Every state the legs pass
    /// through keeps the boxes apart; only the two views the tests show
    /// clear are ever an end of a leg.
    #[test]
    fn every_view_the_chapter_passes_through_keeps_the_boxes_apart() {
        let (near, far) = ([LINE_DOTS as f64, 0.0], [BIG as f64, 1.0]);
        let views = [near, far, near, far, near];
        for w in views.windows(2) {
            for view in leg_views(w[0], &zoom_legs(w[0], w[1]), 400) {
                let boxes: Vec<(Vec3, f64)> = FRIENDS
                    .iter()
                    .enumerate()
                    .map(|(i, &n)| (box_spot(i, n, view[0], view[1]), SMALL))
                    .collect();
                assert!(all_clear(&boxes), "{view:?}");
            }
        }
    }

    /// Two neighbouring rings of a row clear each other on the line where
    /// that row is shown: rows from a thousand on the line to a thousand,
    /// rows from seventy and the bin's handful on the opening line. The
    /// negative control: neighbouring numbers on the line to a thousand
    /// would put their rings on top of each other.
    #[test]
    fn a_rows_rings_stay_apart() {
        let pic = good();
        let apart = |hand: &[u64], end: f64| {
            hand.windows(2)
                .all(|w| x_at(w[1] as f64, end) - x_at(w[0] as f64, end) >= 2.0 * RING_R + RING_GAP)
        };
        assert!(pic.big_hands.iter().all(|h| apart(h, BIG as f64)));
        assert!(pic.small_hands.iter().all(|h| apart(h, LINE_DOTS as f64)));
        assert!(apart(&pic.bin_handful, LINE_DOTS as f64));
        assert!(!apart(&[1, 2, 3, 4, 5], BIG as f64), "neighbours at a thousand");
        // rows stack far enough apart for their rings
        assert!(row_y(1) - row_y(0) >= 2.0 * RING_R + RING_GAP);
    }

    /// The rows sit between the boxes and the panel: the first row's arrows
    /// clear the small boxes' tops, the top row's rings and the outline end
    /// under the panel's third line (the fourth is used only when one row
    /// is shown), and the friends' arrows stay under the dots.
    #[test]
    fn the_rows_fit_between_the_boxes_and_the_panel() {
        let box_top = STRIP_Y + BOX_RISE + SMALL * BOX_H / 2.0;
        let arrows = row_y(0) - ARROW_UNDER_ROW;
        let barb = ARROW_SPREAD * ARROW_HEAD;
        assert!(arrows - barb > box_top + 0.05, "{arrows} over {box_top}");
        assert!(row_y(0) - RING_R > arrows + barb + 0.02, "rings over their arrows");
        let text_half = 0.16;
        let rows = facts::big_handfuls().len();
        assert!(
            band(rows)[1] < PANEL_Y[2] - text_half,
            "the outline under the third line"
        );
        assert!(row_y(rows - 1) + RING_R < band(rows)[1]);
        assert!(band(rows)[0] > box_top, "the outline over the boxes");
        assert!(
            row_y(0) + RING_R < PANEL_Y[3] - text_half,
            "the fourth line over a single row"
        );
        const {
            assert!(
                STRIP_Y - ARROW_UNDER_LINE + ARROW_SPREAD * ARROW_HEAD < STRIP_Y - 0.05,
                "under the dots"
            )
        };
    }

    /// The panel's lines, the badge, the readout and the countdown stay in
    /// the title-safe area; the badge clears the bin and the panel; the
    /// countdown clears the panel's lines and the marks under the line.
    #[test]
    fn the_words_fit_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let lines = [
            format!("all {} at most {}", pic.k, pic.largest),
            format!("from {}: fewer than one in a million", thousands(pic.big)),
            format!("from {}: {}% of all handfuls", pic.small, pic.small_percent),
            format!("{ALL_FROM} {}: {}", thousands(pic.big), pic.big_spacing),
            format!("{ALL_FROM} {}: {}", pic.small, pic.small_spacing),
            format!("the friends' handful: {}", pic.friends_spacing),
        ];
        for l in &lines {
            assert!(PANEL_X + est_width(l, PANEL_SIZE) < safe_x, "{l}");
        }
        const { assert!(PANEL_Y[0] + 0.2 < FRAME_Y_RADIUS - 0.15) };
        let badge_w = est_width(BADGE, BADGE_SIZE) + 2.0 * BADGE_PAD[0];
        let (badge_left, badge_right) = (BADGE_AT[0] - badge_w / 2.0, BADGE_AT[0] + badge_w / 2.0);
        assert!(badge_left > -safe_x && badge_right < PANEL_X - 0.3);
        const {
            assert!(
                BADGE_AT[1] - 0.15 - BADGE_PAD[1] > BIN_AT[1] + BIN_H / 2.0 + 0.1,
                "over the bin"
            )
        };
        const { assert!(BIN_AT[0] + BIN_W / 2.0 < PANEL_X, "the bin left of the panel") };
        let words_w = est_width(&format!("{READOUT_WORDS} {}", pic.k), READOUT_WORDS_SIZE);
        assert!(READOUT_WORDS_AT[0] + words_w / 2.0 < safe_x && READOUT_AT[0] < safe_x);
        const {
            assert!(
                READOUT_AT[1] - 0.25 > -FRAME_Y_RADIUS + 1.58,
                "above a three-line card's top"
            )
        };
        const { assert!(READOUT_WORDS_AT[1] - READOUT_AT[1] > 0.35, "the words over the number") };
        const {
            assert!(
                THINK_AT[1] - THINK_R - 0.45 > STRIP_Y + 0.3,
                "the countdown's note over the line"
            )
        };
        const { assert!(THINK_AT[1] + THINK_R < PANEL_Y[3] - 0.2, "under the panel") };
        const { assert!(THINK_AT[0] + THINK_R < FRAME_X_RADIUS - 0.25) };
        // the marks' labels are on the frame, the last one at the strip's
        // end, and 02_5's mark keeps its label clear of theirs
        let last = x_at(pic.big as f64, pic.big as f64) + est_width(&thousands(pic.big), LABEL_SIZE) / 2.0;
        assert!(last < safe_x && STRIP_X0 - est_width("1", LABEL_SIZE) / 2.0 > -safe_x);
        let half = |n: u64| est_width(&thousands(n), LABEL_SIZE) / 2.0;
        let far = pic.big as f64;
        for &m in pic.marks.iter().chain([pic.big].iter()) {
            let apart = (x_at(pic.over as f64, far) - x_at(m as f64, far)).abs();
            assert!(apart > half(pic.over) + half(m) + 0.1, "{} beside {m}", pic.over);
        }
    }

    /// Card B says the exact condition in at most three lines, with the
    /// facts' numbers, at every width the lower third might narrow to
    /// beside its label, and is up long enough to be read before the
    /// prediction; the reading time is the corpus's fifteen characters a
    /// second. The negative control: a card a third longer would not fit.
    #[test]
    fn card_b_fits_and_is_read() {
        let text = card_b(&good());
        assert!(text.contains("every set of 5 numbers from 1 to N"));
        for chars in (CARD_CHARS_NARROWEST..=CARD_CHARS_WIDEST).step_by(4) {
            assert!(wrap(&text, chars, 4).len() <= 3, "{chars}: {text}");
        }
        let third: String = text.chars().take(text.chars().count() / 3).collect();
        let longer = format!("{text} {third}");
        assert!(wrap(&longer, CARD_CHARS_NARROWEST, 4).len() > 3);
        assert!(read_s(&text) < 15.0);
        assert!((read_s("fifteen chars..") - 1.0).abs() < 1e-12);
    }
}
