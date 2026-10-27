//! Chapter 02, two ways to count. Chapter 01's timetable again, its gaps now
//! bracketed and measured: two, ten, two, ten, five short gaps and five long
//! (02_1). The bus company's count: a tag on each bus, the gap before it.
//! Half the tags say two, half say ten, and they average six, the sign's
//! number (02_2). Your count is different: you arrive at a moment, inside
//! one gap (02_3). What is the chance a random moment lands in a long gap?
//! (02_4, a prediction, with no dots and no fraction on screen.) Five in
//! six: sixty passengers, one in the middle of every minute, fifty of them
//! in long gaps (02_5). The two counts become two rows to scale, the tags'
//! and the passengers', before any fraction is written. Only then come half
//! against five sixths (02_6). Each gap has one tag, but a dot for every
//! minute it lasts (02_7).
//!
//! The tags, the passengers and you are the objects chapter 03 moves.
//! Every number on screen is read from the facts. Every count the picture
//! makes (brackets, tags, dots, cells) is checked against the fact it
//! shows before anything is drawn.

use explainer_kit::{Facts, Kit, Palette, fade_all, rect, v};
use fmn::prelude::*;

use super::draw::{
    LINE_Y, fact, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flash_all, flash_lagged, flat, fly, group,
    now, say, shrink_to, stand, steady, think, x_of, you,
};
use super::timetable::{
    self, Gap, brighten, drop_in_lagged, fade_in_with, fade_out_with, gap_color, paint_in_order, play_lagged, recolor,
    stroke_to, tag_at, x_at, your_moment,
};
use crate::facts;
use crate::model::{self, HOUR, OFFSET, Q};
use crate::palette;

pub struct TwoWaysToCount {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const SHORT_GAPS: &str = "short gaps";
const LONG_GAPS: &str = "long gaps";
const TAGS_SAY: &str = "tags say";
const AVERAGE: &str = "average";
const ASK: &str = "chance of landing in a long gap?";
const IN_LONG: &str = "in long gaps:";
const OF: &str = "of";
const SHARE_OF_TAGS: &str = "share of tags:";
const SHARE_OF_PASSENGERS: &str = "share of passengers:";
const TAG: &str = "tag";
const PASSENGERS: &str = "passengers";

// ------------------------------------------------------------------ layout

/// The row under the passengers, right of the sign's card (its right edge
/// is near x = −2.45): 02_1's counters, 02_2's tally, 02_4's question, then
/// 02_5's counter at the right.
const ROW_Y: f64 = -1.2;
const ROW_SIZE: f64 = 22.0;
const COUNTER_XS: [f64; 2] = [1.4, 3.8];
const TALLY_XS: [f64; 3] = [0.3, 2.3, 4.3];
const ASK_X: f64 = 1.6;
const ASK_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [5.4, -2.3, 0.0];
/// 02_5's counter: its words from COUNT_X, then the live count and "of 60"
/// at fixed steps, so the count never pushes the words around.
const COUNT_X: f64 = 2.3;
const COUNT_DX: [f64; 2] = [1.5, 1.95];
/// 02_6's two rows, to scale and equally wide: the tags' (one cell per tag)
/// above, the passengers' (one dot per passenger, side by side) below, each
/// gold first. A caption over each row's left end, the fraction beside the
/// passengers' row.
const ROWS_X0: f64 = -3.6;
const ROWS_W: f64 = 7.2;
const TAG_ROW_Y: f64 = -2.0;
const DOT_ROW_Y: f64 = -2.8;
const CELL_H: f64 = 0.22;
const CELL_GAP: f64 = 0.04;
/// A cell is born at its tag this small, then grows into its place.
const CELL_BORN: f64 = 0.25;
const CAPTION_DY: f64 = 0.34;
const CAPTION_SIZE: f64 = 20.0;
const EQUATION_AT: Vec3 = [5.3, DOT_ROW_Y, 0.0];
const EQUATION_SIZE: f64 = 24.0;
/// 02_7: the counts beside one long gap and one short gap; each tag's note
/// stands beside its tag, each gap's passenger count under its dots.
const NOTE_SIZE: f64 = 18.0;
const NOTE_DX: f64 = 0.12;
const DOTS_NOTE_Y: f64 = -0.95;
/// How far what is not being looked at dims.
const DIM: f64 = 0.3;
/// You step down onto the line; the passengers drop onto their row.
const STEP_ON: f64 = 0.45;
const DROP: f64 = 0.35;
/// 02_4: you slide slowly from one end of the hour to the other.
const SLIDE_RT: f64 = 4.0;
/// 02_5: the passengers take their gaps' colors, left to right.
const SWEEP_RT: f64 = 2.0;

/// What chapter 02 draws, from the model, checked against the facts it
/// prints: the hour's gaps, which tags say long, the gap each passenger
/// stands in, and your gap with the short gap after it.
struct Counts {
    gaps: Vec<Gap>,
    gap_of: Vec<usize>,
    long_tags: Vec<bool>,
    long_people: Vec<bool>,
    yours: usize,
    next: usize,
}

/// The counts, refused when a fact the picture prints no longer matches
/// what the picture draws.
fn counts(f: &Facts) -> Result<Counts, String> {
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let (short, long) = facts::short_and_long();
    let gaps = timetable::gaps(OFFSET);
    let tags = model::gap_before_each_bus(OFFSET);
    let gap_of = timetable::gap_of_each(&gaps)?;
    let printed: Vec<String> = get("F-S1.bus_gap_tags")?.split(',').map(str::to_owned).collect();
    if printed != tags.iter().map(i64::to_string).collect::<Vec<_>>() {
        return Err("the printed tags are not the gap before each bus".into());
    }
    for g in &gaps {
        if tags[g.ends] != g.minutes() {
            return Err(format!(
                "the bus ending the gap from :{:02} carries the wrong tag",
                g.from
            ));
        }
    }
    let lengths: Vec<i64> = gaps.iter().map(Gap::minutes).collect();
    let long_tags: Vec<bool> = tags.iter().map(|&t| t == long).collect();
    let long_people: Vec<bool> = gap_of.iter().map(|&g| gaps[g].minutes() == long).collect();
    let how_many = |xs: &[i64], n: i64| xs.iter().filter(|&&x| x == n).count().to_string();
    let people = long_people.len() as i64;
    let in_long = long_people.iter().filter(|&&l| l).count() as i64;
    let tags_long = long_tags.iter().filter(|&&l| l).count() as i64;
    let at = your_moment();
    let yours = gaps
        .iter()
        .position(|g| g.holds(at))
        .ok_or("your moment is in no gap")?;
    let next = yours + 1;
    if gaps[yours].minutes() != long || gaps.get(next).map(Gap::minutes) != Some(short) {
        return Err("you are not in a long gap followed by a short one".into());
    }
    let in_gap = |g: usize| gap_of.iter().filter(|&&x| x == g).count().to_string();
    let checks = [
        ("short gaps", how_many(&lengths[..], short), "F-T1.short_count"),
        ("long gaps", how_many(&lengths[..], long), "F-T1.long_count"),
        ("tags saying short", how_many(&tags[..], short), "F-T1.short_count"),
        ("tags saying long", how_many(&tags[..], long), "F-T1.long_count"),
        (
            "the tags' average",
            model::mean_gap(&tags).to_string(),
            "F-S1.bus_mean_gap",
        ),
        ("the tags' average", model::mean_gap(&tags).to_string(), "F-T1.mean_gap"),
        (
            "tags per gap",
            Q::new(tags.len() as i64, gaps.len() as i64).to_string(),
            "F-S1.tags_per_gap",
        ),
        ("passengers", people.to_string(), "F-S1.passengers"),
        (
            "passengers in long gaps",
            in_long.to_string(),
            "F-S1.passengers_in_long",
        ),
        (
            "passengers in short gaps",
            (people - in_long).to_string(),
            "F-S1.passengers_in_short",
        ),
        (
            "the tags' long share",
            Q::new(tags_long, tags.len() as i64).to_string(),
            "F-T1.long_share_of_gaps",
        ),
        (
            "the passengers' long share",
            Q::new(in_long, people).to_string(),
            "F-S1.person_long_share",
        ),
        ("passengers in your gap", in_gap(yours), "F-W2.long_people"),
        (
            "passengers in the short gap after it",
            in_gap(next),
            "F-W2.short_people",
        ),
    ];
    for (what, drawn, key) in checks {
        let printed = get(key)?;
        if drawn != printed {
            return Err(format!("{what}: the picture draws {drawn}, but {key} is {printed}"));
        }
    }
    Ok(Counts {
        gaps,
        gap_of,
        long_tags,
        long_people,
        yours,
        next,
    })
}

/// The cells of one of 02_6's rows: for `longs.len()` things, equal slots
/// across the rows' width, the long ones first (in their order), then the
/// short ones. Each is (the thing's index, its slot's centre x).
fn sorted_row(longs: &[bool]) -> Vec<(usize, f64)> {
    let w = ROWS_W / longs.len() as f64;
    let n = longs.len();
    let order: Vec<usize> = (0..n)
        .filter(|&i| longs[i])
        .chain((0..n).filter(|&i| !longs[i]))
        .collect();
    order
        .into_iter()
        .enumerate()
        .map(|(slot, i)| (i, ROWS_X0 + w * (slot as f64 + 0.5)))
        .collect()
}

impl SceneConstruct for TwoWaysToCount {
    fn name(&self) -> &str {
        "02_two_ways_to_count"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let c = counts(&f).map_err(fail)?;
        let (_, long) = facts::short_and_long();
        let times = model::bus_times(OFFSET);
        let people = model::passengers();
        let at = your_moment();

        // 02_1: chapter 01's timetable; a bracket under each gap, lagged left
        // to right; five short gaps and five long ones
        let line = say(&kit, stage, "02_1")?;
        let table = timetable::timetable(&kit, stage, &f, OFFSET)?;
        fade_in_all(stage, &table.mobs(), 0.6)?;
        let mut brackets = Vec::with_capacity(c.gaps.len());
        for g in &c.gaps {
            let key = if g.minutes() == long { "F-T1.long" } else { "F-T1.short" };
            brackets.push(timetable::bracket(
                &kit,
                stage,
                g,
                &fact(&f, key)?,
                gap_color(g.minutes()),
            )?);
        }
        let pairs: Vec<Mob> = brackets
            .iter()
            .map(|b| group(stage, &[b.line, b.label]))
            .collect::<fmn::Result<_>>()?;
        line.wait_for(stage, "Two minutes")?;
        fade_in_lagged(stage, &pairs, 2.4, 0.3)?;
        line.wait_for(stage, "Five short gaps")?;
        let counters = [
            kit.text(
                stage,
                &format!("{} {SHORT_GAPS}", fact(&f, "F-T1.short_count")?),
                ROW_SIZE,
                timetable::short_gap(),
                v(COUNTER_XS[0], ROW_Y),
            )?,
            kit.text(
                stage,
                &format!("{} {LONG_GAPS}", fact(&f, "F-T1.long_count")?),
                ROW_SIZE,
                palette::long_gap(),
                v(COUNTER_XS[1], ROW_Y),
            )?,
        ];
        fade_in_all(stage, &counters, 0.5)?;

        // 02_2: the bus company's count, a tag on each bus, the gap before
        // it; the tally, and its average is the sign's
        let line = say(&kit, stage, "02_2")?;
        fade_out_all(stage, &counters, 0.4)?;
        let printed: Vec<String> = fact(&f, "F-S1.bus_gap_tags")?.split(',').map(str::to_owned).collect();
        let tags: Vec<Mob> = times
            .iter()
            .zip(&printed)
            .map(|(&t, value)| timetable::tag(&kit, stage, x_of(Q::int(t)), value))
            .collect::<fmn::Result<_>>()?;
        line.wait_for(stage, "Tag each bus")?;
        fade_in_lagged(stage, &tags, 2.0, 0.3)?;
        line.wait_for(stage, "Half the tags")?;
        flash_lagged(stage, &tags, 1.0, 0.15)?;
        let tally = [
            kit.text(
                stage,
                &format!(
                    "{} {TAGS_SAY} {}",
                    fact(&f, "F-T1.short_count")?,
                    fact(&f, "F-T1.short")?
                ),
                ROW_SIZE,
                timetable::short_gap(),
                v(TALLY_XS[0], ROW_Y),
            )?,
            kit.text(
                stage,
                &format!("{} {TAGS_SAY} {}", fact(&f, "F-T1.long_count")?, fact(&f, "F-T1.long")?),
                ROW_SIZE,
                palette::long_gap(),
                v(TALLY_XS[1], ROW_Y),
            )?,
            kit.text(
                stage,
                &format!("{AVERAGE} {}", fact(&f, "F-S1.bus_mean_gap")?),
                ROW_SIZE,
                Palette::ink(),
                v(TALLY_XS[2], ROW_Y),
            )?,
        ];
        fade_in_all(stage, &tally[..2], 0.5)?;
        line.wait_for(stage, "they average six")?;
        fade_in_all(stage, &tally[2..], 0.4)?;
        flash_all(stage, &[tally[2], table.sign.every], 0.8)?;

        // 02_3: not how you see it; the bus company's tally and sign leave;
        // you step onto the line at your moment, and your gap brightens
        let line = say(&kit, stage, "02_3")?;
        let mut leaving = tally.to_vec();
        leaving.extend(table.sign.mobs());
        fade_out_all(stage, &leaving, 0.5)?;
        line.wait_for(stage, "You arrive at some moment")?;
        let me = you(stage, v(x_of(at), LINE_Y))?;
        drop_in_lagged(stage, &[me], STEP_ON, 0.6, 0.0)?;
        line.wait_for(stage, "inside a gap")?;
        let yours = c.gaps[c.yours];
        let my_span = timetable::span(stage, yours.from as f64, yours.to as f64, gap_color(yours.minutes()));
        let mut dims = Vec::new();
        for (i, b) in brackets.iter().enumerate() {
            if i != c.yours {
                dims.push(stroke_to(b.line, DIM, 0.5)?);
                dims.push(fade_to(b.label, DIM, 0.5)?);
            }
        }
        fade_in_with(stage, &[my_span], dims, 0.5)?;

        // 02_4: no gap shaded; you slide from one end of the hour to the
        // other; the question, and think time (no dots, no fraction)
        let line = say(&kit, stage, "02_4")?;
        let mut back = Vec::new();
        for (i, b) in brackets.iter().enumerate() {
            if i != c.yours {
                back.push(stroke_to(b.line, 1.0, 0.4)?);
                back.push(fade_to(b.label, 1.0, 0.4)?);
            }
        }
        fade_out_with(stage, &[my_span], back, 0.4)?;
        line.wait_for(stage, "a random moment")?;
        stage.play(fly(me, v(x_of(Q::int(0)), LINE_Y), 0.6)?)?;
        stage.play(fly(me, v(x_of(Q::int(HOUR)), LINE_Y), SLIDE_RT)?)?;
        line.wait_for(stage, "What's the chance")?;
        let ask = kit.text(stage, ASK, ASK_SIZE, Palette::ink(), v(ASK_X, ROW_Y))?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 02_5: five in six; you go back to your moment; the other
        // passengers drop in, one in the middle of every minute, and take
        // their gaps' colors while the counter counts the gold ones
        let line = say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        stage.play(fly(me, v(x_of(at), LINE_Y), 0.6)?)?;
        let dots: Vec<Mob> = people
            .iter()
            .map(|&t| timetable::passenger(stage, t, Palette::ink()))
            .collect();
        line.wait_for(stage, "Picture the other passengers")?;
        drop_in_lagged(stage, &dots, DROP, 2.4, 0.05)?;
        line.wait_for(stage, "Sixty in the hour")?;
        let landed = stage.add_value_tracker(0.0);
        let words = kit.text_left(stage, IN_LONG, ROW_SIZE, Palette::ink(), v(COUNT_X, ROW_Y))?;
        let total = kit.text_left(
            stage,
            &format!("{OF} {}", fact(&f, "F-S1.passengers")?),
            ROW_SIZE,
            Palette::ink(),
            v(COUNT_X + COUNT_DX[1], ROW_Y),
        )?;
        fade_in_all(stage, &[words, total], 0.4)?;
        let flags = c.long_people.clone();
        kit.live_value(
            stage,
            move |s| {
                let n = timetable::reached(s.tracker_value(landed), flags.len());
                flags[..n].iter().filter(|&&l| l).count() as f64
            },
            0,
            false,
            ROW_SIZE,
            palette::long_gap(),
            v(COUNT_X + COUNT_DX[0], ROW_Y),
            LEFT,
        )?;
        line.wait_for(stage, "fifty of them")?;
        let colors: Vec<Srgb> = c.gap_of.iter().map(|&g| gap_color(c.gaps[g].minutes())).collect();
        let painter = paint_in_order(stage, &dots, colors, landed)?;
        stage.play(steady(landed, dots.len() as f64, SWEEP_RT)?)?;
        stage.clear_updaters(painter, true);

        // 02_6: the two counts as rows of one width, the geometry first:
        // each tag's cell, born at its tag, and a copy of each passenger fly
        // into place, gold first; then the shares, then the fraction
        let line = say(&kit, stage, "02_6")?;
        let cell_w = ROWS_W / c.long_tags.len() as f64 - CELL_GAP;
        let mut cells = Vec::with_capacity(c.long_tags.len());
        for (i, x) in sorted_row(&c.long_tags) {
            let color = if c.long_tags[i] {
                palette::long_gap()
            } else {
                timetable::short_gap()
            };
            let born = tag_at(x_of(Q::int(times[i])));
            let cell = rect(
                v(born[0] - cell_w * CELL_BORN / 2.0, born[1] - CELL_H * CELL_BORN / 2.0),
                cell_w * CELL_BORN,
                CELL_H * CELL_BORN,
                flat(color, 1.0),
            )?;
            let cell = stage.arena_mut().add(cell);
            cells.push(shrink_to(cell, 1.0 / CELL_BORN, v(x, TAG_ROW_Y), 1.0)?);
        }
        play_lagged(stage, cells, 1.0, 0.15)?;
        let mut copies = Vec::with_capacity(people.len());
        for (i, x) in sorted_row(&c.long_people) {
            let copy = timetable::passenger(stage, people[i], gap_color(c.gaps[c.gap_of[i]].minutes()));
            copies.push(fly(copy, v(x, DOT_ROW_Y), 1.0)?);
        }
        play_lagged(stage, copies, 1.4, 0.03)?;
        line.wait_for(stage, "Half the gaps")?;
        let tag_caption = kit.text_left(
            stage,
            &format!("{SHARE_OF_TAGS} {}", fact(&f, "F-T1.long_share_of_gaps")?),
            CAPTION_SIZE,
            Palette::dim(),
            v(ROWS_X0, TAG_ROW_Y + CAPTION_DY),
        )?;
        fade_in_all(stage, &[tag_caption], 0.4)?;
        line.wait_for(stage, "five sixths")?;
        let share = fact(&f, "F-S1.person_long_share")?;
        let dot_caption = kit.text_left(
            stage,
            &format!("{SHARE_OF_PASSENGERS} {share}"),
            CAPTION_SIZE,
            palette::long_gap(),
            v(ROWS_X0, DOT_ROW_Y + CAPTION_DY),
        )?;
        let equation = kit.text(
            stage,
            &format!(
                "{}/{} = {share}",
                fact(&f, "F-S1.passengers_in_long")?,
                fact(&f, "F-S1.passengers")?
            ),
            EQUATION_SIZE,
            palette::long_gap(),
            EQUATION_AT,
        )?;
        fade_in_all(stage, &[dot_caption, equation], 0.4)?;

        // 02_7: your long gap, its one tag and its ten dots bright, the rest
        // dim; then the short gap after it, one tag and two dots
        let line = say(&kit, stage, "02_7")?;
        let next = c.gaps[c.next];
        let one = fact(&f, "F-S1.tags_per_gap")?;
        line.wait_for(stage, "Each gap gets one tag")?;
        let mut dims = Vec::new();
        for (i, b) in brackets.iter().enumerate() {
            if i != c.yours {
                dims.push(stroke_to(b.line, DIM, 0.5)?);
                dims.push(fade_to(b.label, DIM, 0.5)?);
            }
        }
        for (j, &t) in tags.iter().enumerate() {
            dims.push(if j == yours.ends {
                recolor(t, Palette::ink(), 0.5)?
            } else {
                fade_to(t, DIM, 0.5)?
            });
        }
        for (k, &d) in dots.iter().enumerate() {
            if c.gap_of[k] != c.yours {
                dims.push(fade_to(d, DIM, 0.5)?);
            }
        }
        let long_span = timetable::span(stage, yours.from as f64, yours.to as f64, gap_color(yours.minutes()));
        fade_in_with(stage, &[long_span], dims, 0.5)?;
        // the long gap's tag note stands left of its tag, the short gap's right
        let long_tag = tags[yours.ends];
        let tag_y = tag_at(0.0)[1];
        let half = stage.get_width(long_tag) / 2.0;
        let long_note = kit.text(stage, &format!("{one} {TAG}"), NOTE_SIZE, Palette::ink(), v(0.0, tag_y))?;
        stage.move_to(
            long_note,
            v(x_of(Q::int(times[yours.ends])) - half - NOTE_DX, tag_y),
            RIGHT,
        );
        fade_in_all(stage, &[long_note], 0.4)?;
        flash_all(stage, &[long_tag], 0.6)?;
        line.wait_for(stage, "a dot for every minute")?;
        let long_dots: Vec<Mob> = (0..dots.len())
            .filter(|&k| c.gap_of[k] == c.yours)
            .map(|k| dots[k])
            .collect();
        let long_count = kit.text(
            stage,
            &format!("{} {PASSENGERS}", fact(&f, "F-W2.long_people")?),
            NOTE_SIZE,
            palette::long_gap(),
            v(x_at((yours.from + yours.to) as f64 / 2.0), DOTS_NOTE_Y),
        )?;
        fade_in_all(stage, &[long_count], 0.4)?;
        flash_lagged(stage, &long_dots, 1.0, 0.1)?;
        line.wait_for(stage, "Counting buses")?;
        let short_tag = tags[next.ends];
        let mut lift = vec![
            stroke_to(brackets[c.next].line, 1.0, 0.5)?,
            fade_to(brackets[c.next].label, 1.0, 0.5)?,
            brighten(short_tag, Palette::ink(), 0.5)?,
        ];
        for (k, &d) in dots.iter().enumerate() {
            if c.gap_of[k] == c.next {
                lift.push(fade_to(d, 1.0, 0.5)?);
            }
        }
        let short_span = timetable::span(stage, next.from as f64, next.to as f64, gap_color(next.minutes()));
        fade_in_with(stage, &[short_span], lift, 0.5)?;
        let half = stage.get_width(short_tag) / 2.0;
        let short_note = kit.text_left(
            stage,
            &format!("{one} {TAG}"),
            NOTE_SIZE,
            Palette::ink(),
            v(x_of(Q::int(times[next.ends])) + half + NOTE_DX, tag_y),
        )?;
        let short_count = kit.text(
            stage,
            &format!("{} {PASSENGERS}", fact(&f, "F-W2.short_people")?),
            NOTE_SIZE,
            timetable::short_gap(),
            v(x_at((next.from + next.to) as f64 / 2.0), DOTS_NOTE_Y),
        )?;
        fade_in_all(stage, &[short_note, short_count], 0.4)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        // After the conclusion, let the viewer inspect one tag versus the
        // different passenger counts on the same two gaps before leaving.
        let inspected_until = now(stage) + 4.0;
        super::draw::wait_until(stage, inspected_until)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};

    use super::*;
    use crate::chapters::draw::PASSENGER_R;

    /// Needs chapter 02's lines in script.tsv (the integration step moves
    /// them from the draft). The whole chapter runs headless at 10 fps with
    /// the scratch voice: every phrase it waits for is in its line, every
    /// fact it reads exists, every count it draws is its fact, and its seven
    /// lines are said in order. It ends with `kit.finish`, whose timeline
    /// runs past the end of the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TwoWaysToCount { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["02_1", "02_2", "02_3", "02_4", "02_5", "02_6", "02_7"]);
        let (last, at) = said_at.last().unwrap();
        let last_ends = at + narrator.length(last).unwrap();
        let end = scene
            .kit
            .finished
            .borrow()
            .as_ref()
            .expect("no timeline: the chapter must end with kit.finish")
            .seconds();
        assert!(
            end > last_ends,
            "the timeline ends at {end:.2} s, before {last} ends at {last_ends:.2} s"
        );
    }

    /// The counts the picture draws are the facts it prints, and a fact that
    /// stops matching the picture is refused (one passenger fewer in the
    /// long gaps).
    #[test]
    fn the_pictures_counts_are_its_facts_and_a_wrong_fact_is_refused() {
        let f = facts::facts().unwrap();
        let c = counts(&f).unwrap();
        assert!(c.gaps[c.yours].holds(your_moment()));
        assert_eq!(c.gaps[c.yours].to, c.gaps[c.next].from);
        let in_long: i64 = f.get("F-S1.passengers_in_long").unwrap().parse().unwrap();
        let fewer = (in_long - 1).to_string();
        let mut wrong = Facts::new();
        for row in f.print().lines() {
            let (key, value) = row.split_once('\t').unwrap();
            wrong
                .add(
                    key,
                    if key == "F-S1.passengers_in_long" {
                        fewer.as_str()
                    } else {
                        value
                    },
                )
                .unwrap();
        }
        let refused = counts(&wrong).err().expect("a wrong count must be refused");
        assert!(refused.contains("F-S1.passengers_in_long"), "{refused}");
    }

    /// 02_6's rows are the two counts to scale: each row's gold run, from
    /// the rows' left edge, is its long share of the rows' width (the tags'
    /// half, the passengers' five sixths), every slot is used once, and the
    /// passengers' dots fit side by side without overlapping.
    #[test]
    fn the_rows_are_the_two_counts_to_scale() {
        let f = facts::facts().unwrap();
        let c = counts(&f).unwrap();
        for (flags, key) in [
            (&c.long_tags, "F-T1.long_share_of_gaps"),
            (&c.long_people, "F-S1.person_long_share"),
        ] {
            let slots = sorted_row(flags);
            let w = ROWS_W / flags.len() as f64;
            let gold_end = slots
                .iter()
                .filter(|&&(i, _)| flags[i])
                .map(|&(_, x)| x + w / 2.0)
                .fold(ROWS_X0, f64::max);
            let (n, d) = f.get(key).unwrap().split_once('/').unwrap();
            let share = n.parse::<f64>().unwrap() / d.parse::<f64>().unwrap();
            assert!(((gold_end - ROWS_X0) / ROWS_W - share).abs() < 1e-12, "{key}");
            let mut used: Vec<usize> = slots.iter().map(|&(i, _)| i).collect();
            used.sort_unstable();
            assert_eq!(used, (0..flags.len()).collect::<Vec<_>>());
            for &(_, x) in &slots {
                assert!(x - w / 2.0 >= ROWS_X0 - 1e-12 && x + w / 2.0 <= ROWS_X0 + ROWS_W + 1e-12);
            }
        }
        assert!(ROWS_W / c.long_people.len() as f64 >= 2.0 * PASSENGER_R);
    }

    /// The text this chapter places on its rows stays apart and on screen.
    /// Widths are estimated, not measured: 0.0045 units a point a character
    /// (bayesian_search's measured face), a line 0.6 of its size tall.
    #[test]
    fn the_rows_text_stays_apart_and_on_screen() {
        let f = facts::facts().unwrap();
        let get = |k: &str| f.get(k).unwrap().to_owned();
        let w = |s: &str, pt: f64| s.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        // (left, right, bottom, top) of centred and left-aligned text
        let centred = |s: &str, pt: f64, x: f64, y: f64| (x - w(s, pt) / 2.0, x + w(s, pt) / 2.0, y - h(pt), y + h(pt));
        let left = |s: &str, pt: f64, x: f64, y: f64| (x, x + w(s, pt), y - h(pt), y + h(pt));
        let apart = |a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: (f64, f64, f64, f64)| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        let sign_right = -4.3 + 3.7 / 2.0;
        let counters = [
            centred(
                &format!("{} {SHORT_GAPS}", get("F-T1.short_count")),
                ROW_SIZE,
                COUNTER_XS[0],
                ROW_Y,
            ),
            centred(
                &format!("{} {LONG_GAPS}", get("F-T1.long_count")),
                ROW_SIZE,
                COUNTER_XS[1],
                ROW_Y,
            ),
        ];
        let tally = [
            centred(
                &format!("{} {TAGS_SAY} {}", get("F-T1.short_count"), get("F-T1.short")),
                ROW_SIZE,
                TALLY_XS[0],
                ROW_Y,
            ),
            centred(
                &format!("{} {TAGS_SAY} {}", get("F-T1.long_count"), get("F-T1.long")),
                ROW_SIZE,
                TALLY_XS[1],
                ROW_Y,
            ),
            centred(
                &format!("{AVERAGE} {}", get("F-S1.bus_mean_gap")),
                ROW_SIZE,
                TALLY_XS[2],
                ROW_Y,
            ),
        ];
        let count = [
            left(IN_LONG, ROW_SIZE, COUNT_X, ROW_Y),
            left(&get("F-S1.passengers_in_long"), ROW_SIZE, COUNT_X + COUNT_DX[0], ROW_Y),
            left(
                &format!("{OF} {}", get("F-S1.passengers")),
                ROW_SIZE,
                COUNT_X + COUNT_DX[1],
                ROW_Y,
            ),
        ];
        for beat in [&counters[..], &tally[..], &count[..]] {
            for (i, &a) in beat.iter().enumerate() {
                assert!(on_screen(a) && a.0 > sign_right, "{a:?}");
                for &b in &beat[i + 1..] {
                    assert!(apart(a, b), "{a:?} {b:?}");
                }
            }
        }
        let ask = centred(ASK, ASK_SIZE, ASK_X, ROW_Y);
        assert!(ask.0 > sign_right && ask.1 < THINK_AT[0] - 0.62);
        let captions = [
            left(
                &format!("{SHARE_OF_TAGS} {}", get("F-T1.long_share_of_gaps")),
                CAPTION_SIZE,
                ROWS_X0,
                TAG_ROW_Y + CAPTION_DY,
            ),
            left(
                &format!("{SHARE_OF_PASSENGERS} {}", get("F-S1.person_long_share")),
                CAPTION_SIZE,
                ROWS_X0,
                DOT_ROW_Y + CAPTION_DY,
            ),
        ];
        let rows = [
            (
                ROWS_X0,
                ROWS_X0 + ROWS_W,
                TAG_ROW_Y - CELL_H / 2.0,
                TAG_ROW_Y + CELL_H / 2.0,
            ),
            (
                ROWS_X0,
                ROWS_X0 + ROWS_W,
                DOT_ROW_Y - PASSENGER_R,
                DOT_ROW_Y + PASSENGER_R,
            ),
        ];
        let equation = centred(
            &format!(
                "{}/{} = {}",
                get("F-S1.passengers_in_long"),
                get("F-S1.passengers"),
                get("F-S1.person_long_share")
            ),
            EQUATION_SIZE,
            EQUATION_AT[0],
            EQUATION_AT[1],
        );
        let mut all = captions.to_vec();
        all.extend(rows);
        all.push(equation);
        all.extend(count);
        for (i, &a) in all.iter().enumerate() {
            assert!(on_screen(a), "{a:?}");
            for &b in &all[i + 1..] {
                assert!(apart(a, b), "{a:?} {b:?}");
            }
        }
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            SHORT_GAPS,
            LONG_GAPS,
            TAGS_SAY,
            AVERAGE,
            ASK,
            IN_LONG,
            OF,
            SHARE_OF_TAGS,
            SHARE_OF_PASSENGERS,
            TAG,
            PASSENGERS,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
