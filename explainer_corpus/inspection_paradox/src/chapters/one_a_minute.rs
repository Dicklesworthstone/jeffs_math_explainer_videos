//! Chapter 04, one a minute. From the hour back to one twelve-minute
//! stretch of it, and the wait of every passenger in it:
//! - 04_1: the hour as chapter 03 left it, the same twelve minutes five
//!   times over. What lies past the first twelve minutes leaves, and the
//!   stretch widens to fill the frame: the same line, buses, brackets and
//!   passengers, each zoomed into its twin.
//! - 04_2: a tick at every minute puts each passenger at a minute's
//!   middle; your two mornings of 01_4 come back as ghosts on the long
//!   gap's first and last passengers.
//! - 04_3, 04_4: from each passenger a bar rises to its wait, the short
//!   gap's two and then the long gap's ten as a staircase, and each gap's
//!   average; your two mornings are the staircase's first and last steps.
//! - 04_5: the average over all twelve? (a prediction; no sum on screen.)
//! - 04_6, 04_7: the waits added up gap by gap, then all the waiting shared
//!   by the twelve, boxed, with the exact version (K02).
//! - 04_8: the guess of half the average gap is the plain average of the
//!   gaps' average waits, as if both gaps held as many passengers; the long
//!   gap holds five times as many.
//!
//! Every number on screen is read from the facts, and
//! `twelve_minutes::numbers` checks each against the stretch before the
//! chapter starts.

use explainer_kit::props::precisely;
use explainer_kit::{Facts, Kit, Palette, Timed, fade_all, rect, v};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{
    LINE_Y, bus_marker, fact, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flash_lagged, hour_line,
    line_style, min_s, now, read_s, say, stand, think, wait_until, x_of,
};
use super::timetable::{self, Bracket, gap_color};
use super::twelve_minutes::{
    self, MIN, Numbers, PANEL_X, Passenger, Stretch, at, bar_top, fade_in_flashing, grow_lagged, in_gap, pick, row,
};
use crate::facts;
use crate::model::{self, HOUR, OFFSET, PERIOD, Q};
use crate::palette;

pub struct OneAMinute {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const TIMES: &str = "×";
const IN_AN_HOUR: &str = "in an hour";
const AVERAGE: &str = "average";
const ASK_HEAD: &str = "average wait, all";
const ASK_TAIL: &str = "passengers?";
const AND_SO_ON: &str = "…";
const OF_WAITING: &str = "minutes of waiting";
const GUESS: &str = "guess: half of";
const PASSENGERS: &str = "passengers";
/// The exact version of the chapter's claim (K02).
const K02: &str = "passengers at the middle of each minute give exactly the average for a moment chosen at \
                   random, because the wait falls in a straight line inside each gap";

// ------------------------------------------------------------------ layout

/// 04_1: the zoom, and "× 5 in an hour" right of the stretch's last clock
/// label, clear of the long gap's bracket.
const ZOOM_RT: f64 = 1.6;
const PER_HOUR_AT: Vec3 = [0.25, -2.2, 0.0];
const SMALL: f64 = 18.0;
/// A ghost of you (01_4): `draw.rs`'s ring (its radius is private there)
/// without the dot, so the passenger's color shows inside, dimmed as
/// chapter 01 dims its ghosts, and over the passenger.
const GHOST_R: f64 = 0.16;
const GHOST_W: f64 = 3.0;
const GHOST_DIM: f64 = 0.45;
const GHOST_Z: i32 = 3;
/// Over each bar its wait, and over the two mornings' bars their clocks.
const BAR_LABEL_DY: f64 = 0.18;
const BAR_LABEL_SIZE: f64 = 16.0;
const MORNING_DY: f64 = 0.3;
/// Each gap's average: over the short gap's two bars, and in the open
/// corner above the long gap's staircase.
const SHORT_MEAN_AT: Vec3 = [-5.7, -0.3, 0.0];
const LONG_MEAN_AT: Vec3 = [-1.7, 1.9, 0.0];
/// 04_3's words for the short gap's waits, one bar rising on each.
const SHORT_WAITS_SAID: [&str; 2] = ["a minute and a half", "and half a minute"];
/// Timing: a short gap's bar, the staircase.
const BAR_RT: f64 = 0.5;
const STAIRS_RT: f64 = 2.2;
const STAIRS_LAG: f64 = 0.25;
/// The panel (`twelve_minutes::row`): the question in row 1 with its think
/// ring under it; the sums in rows 0 to 2 and the verdict boxed in row 3;
/// then the guess, its other form and the passengers' counts in rows 4
/// to 6.
const PANEL_SIZE: f64 = 24.0;
const ASK_ROW: usize = 1;
const THINK_AT: Vec3 = [PANEL_X, 0.9, 0.0];
const SUM_ROWS: [usize; 3] = [0, 1, 2];
const VERDICT_ROW: usize = 3;
const BOX_PAD: f64 = 0.15;
const GUESS_ROW: usize = 4;
const OTHER_ROW: usize = 5;
const COUNT_ROW: usize = 6;
const COUNT_SIZE: f64 = 22.0;
const COUNT_XS: [f64; 3] = [2.3, PANEL_X, 5.5];
const STRIKE_PAD: f64 = 0.12;

// ----------------------------------------------------------------- the hour

/// The hour as chapter 03 leaves it, without its sign, tags and you: the
/// line with a label every twelve minutes, both routes' markers, each gap's
/// bracket and the sixty passengers in their gaps' colors, all detached.
struct Hour {
    /// The line's first twelve minutes, laid over the hour's line: the zoom
    /// widens it into the stretch's line.
    piece: Mob,
    /// The hour's line, its ticks and its labels.
    line: Vec<Mob>,
    /// Every bus's marker, in time order, with its minute.
    markers: Vec<(i64, Mob)>,
    brackets: Vec<Bracket>,
    /// The passengers, in time order.
    dots: Vec<Mob>,
}

impl Hour {
    fn mobs(&self) -> Vec<Mob> {
        let mut out = self.line.clone();
        out.push(self.piece);
        out.extend(self.markers.iter().map(|&(_, m)| m));
        out.extend(self.brackets.iter().flat_map(|b| [b.line, b.label]));
        out.extend(&self.dots);
        out
    }

    /// The blue markers, one at the start of each twelve minutes.
    fn blue(&self) -> Vec<Mob> {
        let blue = model::route(0);
        self.markers
            .iter()
            .filter(|(t, _)| blue.contains(t))
            .map(|&(_, m)| m)
            .collect()
    }
}

fn hour(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Hour> {
    let marks: Vec<i64> = (0..=HOUR).step_by(PERIOD as usize).collect();
    let line = hour_line(kit, stage, &marks)?.mobs();
    let piece = stage.arena_mut().add(
        Line::new(v(x_of(Q::int(0)), LINE_Y), v(x_of(Q::int(PERIOD)), LINE_Y))
            .style(line_style(Palette::dim(), 2.0))
            .build()?,
    );
    let mut markers = Vec::new();
    for t in model::bus_times(OFFSET) {
        markers.push((t, bus_marker(stage, t, twelve_minutes::bus_color(t))?));
    }
    let gaps = timetable::gaps(OFFSET);
    let (_, long) = facts::short_and_long();
    let mut brackets = Vec::new();
    for g in &gaps {
        let key = if g.minutes() == long { "F-T1.long" } else { "F-T1.short" };
        brackets.push(timetable::bracket(
            kit,
            stage,
            g,
            &fact(f, key)?,
            gap_color(g.minutes()),
        )?);
    }
    let of = timetable::gap_of_each(&gaps).map_err(fail)?;
    let dots = model::passengers()
        .into_iter()
        .zip(&of)
        .map(|(t, &g)| timetable::passenger(stage, t, gap_color(gaps[g].minutes())))
        .collect();
    Ok(Hour {
        piece,
        line,
        markers,
        brackets,
        dots,
    })
}

/// 04_1's zoom: what lies past the stretch leaves while the stretch's own
/// objects in the hour widen into their twins, one transform each (the
/// first twelve minutes' line, the stretch's three buses, its two brackets
/// and its twelve passengers), all together over [`ZOOM_RT`].
fn zoom(stage: &mut Stage<'_>, hour: &Hour, st: &Stretch) -> fmn::Result<()> {
    let kept: Vec<Mob> = hour
        .markers
        .iter()
        .filter(|(t, _)| *t <= PERIOD)
        .map(|&(_, m)| m)
        .collect();
    if kept.len() != st.markers.len() || hour.brackets.len() < st.brackets.len() || hour.dots.len() < st.dots.len() {
        return Err(fail("the hour does not hold the stretch".into()));
    }
    let mut pairs: Vec<(Mob, Mob)> = vec![(hour.piece, st.axis)];
    pairs.extend(kept.iter().copied().zip(st.markers.iter().copied()));
    for (h, s) in hour.brackets.iter().zip(&st.brackets) {
        pairs.extend([(h.line, s.line), (h.label, s.label)]);
    }
    pairs.extend(hour.dots.iter().copied().zip(st.dots.iter().copied()));
    let moving: Vec<Mob> = pairs.iter().map(|&(h, _)| h).collect();
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for m in hour.mobs() {
        if !moving.contains(&m) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(ZOOM_RT))?);
        }
    }
    for (h, s) in pairs {
        anims.push(stage.prepare(replacement_transform(h, s).rt(ZOOM_RT))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

// ------------------------------------------------------------- the stretch

/// A ghost of you on passenger `p`, as chapter 01 leaves its ghosts: a dim
/// ring in your color, over the passenger.
fn ghost(stage: &mut Stage<'_>, p: &Passenger) -> Mob {
    let ring = stage.arena_mut().add(
        Circle::new()
            .radius(GHOST_R)
            .arc_center(at(p.t.to_f64(), 0.0))
            .style(
                Style::default()
                    .fill(palette::you(), 0.0)
                    .stroke(palette::you(), GHOST_W, GHOST_DIM),
            )
            .build(),
    );
    stage.set_z_index(ring, GHOST_Z, true);
    ring
}

/// Over each passenger's bar, its wait ("1½"), in the gap's color.
fn bar_labels(kit: &Kit, stage: &mut Stage<'_>, n: &Numbers, people: &[Passenger]) -> fmn::Result<Vec<Mob>> {
    let gs = twelve_minutes::gaps();
    let mut out = Vec::new();
    for (p, wait) in people.iter().zip(&n.waits) {
        let top = bar_top(p);
        out.push(kit.text(
            stage,
            wait,
            BAR_LABEL_SIZE,
            gap_color(gs[p.gap].minutes()),
            v(top[0], top[1] + BAR_LABEL_DY),
        )?);
    }
    Ok(out)
}

/// Your morning `k` (0 the unlucky, 1 the lucky) over its bar's label: its
/// wait as chapter 01's clock gave it.
fn morning_label(kit: &Kit, stage: &mut Stage<'_>, n: &Numbers, people: &[Passenger], k: usize) -> fmn::Result<Mob> {
    let top = bar_top(&people[n.mornings[k]]);
    kit.text(
        stage,
        &n.morning_waits[k],
        SMALL,
        palette::you(),
        v(top[0], top[1] + BAR_LABEL_DY + MORNING_DY),
    )
}

// ---------------------------------------------------------------- the panel

/// A gap's waits added up, in its bars' order: all of them when there are
/// three or fewer, else the first two, "…" and the last.
fn sum_text(waits: &[String], total: &str) -> String {
    let terms: Vec<&str> = if waits.len() <= 3 {
        waits.iter().map(String::as_str).collect()
    } else {
        vec![&waits[0], &waits[1], AND_SO_ON, &waits[waits.len() - 1]]
    };
    format!("{} = {total}", terms.join(" + "))
}

/// The guess's other form, "= (1 + 5) ÷ 2", with each gap's average in the
/// gap's color: refused unless each average occurs exactly once in it, so
/// that the coloring finds the right figure.
fn other_form(n: &Numbers) -> Result<(String, Vec<(String, Srgb)>), String> {
    let text = format!("= ({} + {}) ÷ {}", n.gap_means[0], n.gap_means[1], n.gap_count);
    let colors = vec![
        (n.gap_means[0].clone(), timetable::short_gap()),
        (n.gap_means[1].clone(), palette::long_gap()),
    ];
    for (key, _) in &colors {
        if text.matches(key.as_str()).count() != 1 {
            return Err(format!("{key} is not one figure of {text:?}"));
        }
    }
    Ok((text, colors))
}

/// A box around `mob`, [`BOX_PAD`] clear of it, in `color`.
fn boxed(stage: &mut Stage<'_>, mob: Mob, color: Srgb) -> fmn::Result<Mob> {
    let c = stage.get_center(mob);
    let (w, h) = (
        stage.get_width(mob) + 2.0 * BOX_PAD,
        stage.get_height(mob) + 2.0 * BOX_PAD,
    );
    let r = rect(v(c[0] - w / 2.0, c[1] - h / 2.0), w, h, line_style(color, 2.5))?;
    Ok(stage.arena_mut().add(r))
}

/// A slash through `mobs` together, from the lower left to the upper right
/// of their bounds.
fn strike(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<Mob> {
    let left = mobs
        .iter()
        .map(|&m| stage.get_bounding_box_point(m, LEFT)[0])
        .fold(f64::INFINITY, f64::min);
    let right = mobs
        .iter()
        .map(|&m| stage.get_bounding_box_point(m, RIGHT)[0])
        .fold(f64::NEG_INFINITY, f64::max);
    let low = mobs
        .iter()
        .map(|&m| stage.get_bounding_box_point(m, DOWN)[1])
        .fold(f64::INFINITY, f64::min);
    let high = mobs
        .iter()
        .map(|&m| stage.get_bounding_box_point(m, UP)[1])
        .fold(f64::NEG_INFINITY, f64::max);
    let slash = Line::new(
        v(left - STRIKE_PAD, low - STRIKE_PAD),
        v(right + STRIKE_PAD, high + STRIKE_PAD),
    )
    .style(line_style(Palette::ink(), 4.0))
    .build()?;
    Ok(stage.arena_mut().add(slash))
}

impl SceneConstruct for OneAMinute {
    fn name(&self) -> &str {
        "04_one_a_minute"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let n = twelve_minutes::numbers(&f).map_err(fail)?;
        if fact(&f, "F-X2.midpoint_exact")? != "true" {
            return Err(fail(
                "K02 needs F-X2.midpoint_exact: the midpoints are not exact".into(),
            ));
        }
        let people = twelve_minutes::passengers().map_err(fail)?;
        let short = in_gap(&people, 0);
        let long = in_gap(&people, 1);
        let (teal, gold) = (timetable::short_gap(), palette::long_gap());
        let (other, other_colors) = other_form(&n).map_err(fail)?;
        if short.len() != SHORT_WAITS_SAID.len() {
            return Err(fail(format!(
                "04_3 says {} waits for the short gap's {} passengers",
                SHORT_WAITS_SAID.len(),
                short.len()
            )));
        }

        // 04_1: the hour; the same twelve minutes five times over; the zoom
        // into the first of them; each bus of the stretch named with its
        // minute, and the gap it ends
        let line = say(&kit, stage, "04_1")?;
        let hr = hour(&kit, stage, &f)?;
        fade_in_all(stage, &hr.mobs(), 0.8)?;
        line.wait_for(stage, "five times over")?;
        flash_lagged(stage, &hr.blue(), 1.2, 0.2)?;
        line.wait_for(stage, "so zoom in")?;
        let st = twelve_minutes::stretch(&kit, stage, &n, &people)?;
        zoom(stage, &hr, &st)?;
        let per_hour = kit.text_left(
            stage,
            &format!("{TIMES} {} {IN_AN_HOUR}", n.cycles),
            SMALL,
            Palette::dim(),
            PER_HOUR_AT,
        )?;
        fade_in_all(stage, &[per_hour], 0.4)?;
        line.wait_for(stage, "A blue bus")?;
        fade_in_flashing(stage, &[st.clocks[0]], &[st.markers[0]], 0.6)?;
        line.wait_for(stage, "a red bus")?;
        let ends_short = [st.markers[1], st.brackets[0].line, st.brackets[0].label];
        fade_in_flashing(stage, &[st.clocks[1]], &ends_short, 0.6)?;
        line.wait_for(stage, "the next blue bus")?;
        let ends_long = [st.markers[2], st.brackets[1].line, st.brackets[1].label];
        fade_in_flashing(stage, &[st.clocks[2]], &ends_long, 0.6)?;

        // 04_2: the twelve passengers, each at a minute's middle; your two
        // mornings as ghosts
        let line = say(&kit, stage, "04_2")?;
        line.wait_for(stage, "twelve of our passengers")?;
        flash_lagged(stage, &st.dots, 1.2, 0.15)?;
        line.wait_for(stage, "one in the middle of each minute")?;
        fade_in_all(stage, &st.ticks, 0.6)?;
        line.wait_for(stage, "none of them looking")?;
        let ghosts = [
            ghost(stage, &people[n.mornings[0]]),
            ghost(stage, &people[n.mornings[1]]),
        ];
        fade_in_all(stage, &ghosts, 0.5)?;

        // 04_3: the short gap's two bars, one on each wait said; their
        // average
        let line = say(&kit, stage, "04_3")?;
        let bars = twelve_minutes::bars(stage, &people)?;
        let labels = bar_labels(&kit, stage, &n, &people)?;
        line.wait_for(stage, "The two passengers")?;
        flash_all(stage, &pick(&st.dots, &short), 0.6)?;
        for (&i, phrase) in short.iter().zip(SHORT_WAITS_SAID) {
            line.wait_for(stage, phrase)?;
            grow_lagged(stage, &[bars[i]], BAR_RT, 0.0)?;
            fade_in_all(stage, &[labels[i]], 0.3)?;
        }
        line.wait_for(stage, "One minute each")?;
        let short_mean = kit.text(
            stage,
            &format!("{AVERAGE} {} {MIN}", n.gap_means[0]),
            SMALL,
            teal,
            SHORT_MEAN_AT,
        )?;
        fade_in_all(stage, &[short_mean], 0.4)?;

        // 04_4: the long gap's staircase, its average, and your two
        // mornings at its two ends
        let line = say(&kit, stage, "04_4")?;
        line.wait_for(stage, "The ten passengers")?;
        flash_lagged(stage, &pick(&st.dots, &long), 1.0, 0.1)?;
        line.wait_for(stage, "nine and a half minutes")?;
        grow_lagged(stage, &pick(&bars, &long), STAIRS_RT, STAIRS_LAG)?;
        fade_in_lagged(stage, &pick(&labels, &long), 0.8, 0.2)?;
        line.wait_for(stage, "Five minutes each")?;
        let long_mean = kit.text(
            stage,
            &format!("{AVERAGE} {} {MIN}", n.gap_means[1]),
            SMALL,
            gold,
            LONG_MEAN_AT,
        )?;
        fade_in_all(stage, &[long_mean], 0.4)?;
        line.wait_for(stage, "Your unlucky morning")?;
        let unlucky = morning_label(&kit, stage, &n, &people, 0)?;
        fade_in_flashing(stage, &[unlucky], &[ghosts[0]], 0.6)?;
        line.wait_for(stage, "your lucky one")?;
        let lucky = morning_label(&kit, stage, &n, &people, 1)?;
        fade_in_flashing(stage, &[lucky], &[ghosts[1]], 0.6)?;

        // 04_5: the question; think time; no sum on screen
        let line = say(&kit, stage, "04_5")?;
        line.wait_for(stage, "the average wait")?;
        let ask = kit.text(
            stage,
            &format!("{ASK_HEAD} {} {ASK_TAIL}", n.people),
            PANEL_SIZE,
            Palette::ink(),
            v(PANEL_X, row(ASK_ROW)),
        )?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 04_6: not three; each gap's waits added up as it is said, its
        // bars' labels flashing
        let line = say(&kit, stage, "04_6")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        let gap_waits = |places: &[usize]| -> Vec<String> { places.iter().map(|&i| n.waits[i].clone()).collect() };
        line.wait_for(stage, "two minutes between them")?;
        let short_sum = kit.text(
            stage,
            &sum_text(&gap_waits(&short), &n.gap_totals[0]),
            PANEL_SIZE,
            teal,
            v(PANEL_X, row(SUM_ROWS[0])),
        )?;
        fade_in_flashing(stage, &[short_sum], &pick(&labels, &short), 0.6)?;
        line.wait_for(stage, "waited fifty")?;
        let long_sum = kit.text(
            stage,
            &sum_text(&gap_waits(&long), &n.gap_totals[1]),
            PANEL_SIZE,
            gold,
            v(PANEL_X, row(SUM_ROWS[1])),
        )?;
        fade_in_flashing(stage, &[long_sum], &pick(&labels, &long), 0.6)?;

        // 04_7: all the waiting, shared by the twelve; the verdict boxed;
        // the exact version, read before it leaves
        let line = say(&kit, stage, "04_7")?;
        let all = kit.text(
            stage,
            &format!("{} + {} = {} {OF_WAITING}", n.gap_totals[0], n.gap_totals[1], n.total),
            PANEL_SIZE,
            Palette::ink(),
            v(PANEL_X, row(SUM_ROWS[2])),
        )?;
        let k02 = precisely(&kit, stage, K02)?;
        fade_in_all(stage, &[all, k02], 0.5)?;
        let card_up = now(stage);
        line.wait_for(stage, "Four and a third")?;
        let verdict = kit.text(
            stage,
            &format!(
                "{} ÷ {} = {} {MIN} = {}",
                n.total,
                n.people,
                n.mean,
                min_s(&n.mean_clock)?
            ),
            PANEL_SIZE,
            palette::verdict(),
            v(PANEL_X, row(VERDICT_ROW)),
        )?;
        let frame = boxed(stage, verdict, palette::verdict())?;
        fade_in_all(stage, &[verdict, frame], 0.5)?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(K02))?;

        // 04_8: the guess back, and its other form, the gaps' averages
        // averaged; struck through; the passengers' counts, five times as
        // many in the long gap
        let line = say(&kit, stage, "04_8")?;
        fade_out_all(stage, &[short_sum, long_sum, all, k02], 0.4)?;
        let guess = kit.text(
            stage,
            &format!("{GUESS} {} = {} {MIN}", n.mean_gap, n.half_mean_gap),
            PANEL_SIZE,
            Palette::dim(),
            v(PANEL_X, row(GUESS_ROW)),
        )?;
        fade_in_all(stage, &[guess], 0.4)?;
        line.wait_for(stage, "the average of one and five")?;
        let colors: Vec<(&str, Srgb)> = other_colors.iter().map(|(k, c)| (k.as_str(), *c)).collect();
        let other = kit.text_t2c(
            stage,
            &other,
            PANEL_SIZE,
            Palette::dim(),
            &colors,
            v(PANEL_X, row(OTHER_ROW)),
        )?;
        fade_in_flashing(stage, &[other], &[short_mean, long_mean], 0.6)?;
        line.wait_for(stage, "They don't")?;
        let slash = strike(stage, &[guess, other])?;
        fade_in_all(stage, &[slash], 0.3)?;
        line.wait_for(stage, "The long gap has")?;
        let y = row(COUNT_ROW);
        let counts = [
            kit.text(
                stage,
                &format!("{} {PASSENGERS}", n.gap_people[0]),
                COUNT_SIZE,
                teal,
                v(COUNT_XS[0], y),
            )?,
            kit.text(
                stage,
                &format!("{TIMES} {}", n.people_ratio),
                COUNT_SIZE,
                Palette::ink(),
                v(COUNT_XS[1], y),
            )?,
            kit.text(
                stage,
                &format!("{} {PASSENGERS}", n.gap_people[1]),
                COUNT_SIZE,
                gold,
                v(COUNT_XS[2], y),
            )?,
        ];
        fade_in_lagged(stage, &counts, 0.9, 0.3)?;
        flash_lagged(stage, &pick(&st.dots, &long), 1.0, 0.1)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};

    use super::*;

    /// A marker's width as `draw.rs` builds it (private there).
    const MARKER_W: f64 = 0.14;

    /// Needs chapter 04's lines in script.tsv (the integration step copies
    /// them from the draft). The whole chapter runs headless at 10 fps with
    /// the scratch voice: every phrase it waits for is in its line, every
    /// fact it reads exists and matches the stretch, and its eight lines
    /// are said in order. It ends with `kit.finish`, whose timeline runs
    /// past the end of the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = OneAMinute { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["04_1", "04_2", "04_3", "04_4", "04_5", "04_6", "04_7", "04_8"]);
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

    /// Needs chapter 04's section in docs/storyboard.md (the integration
    /// step copies it from the draft). The card is the storyboard's exact
    /// version of the chapter's claim (K02), word for word.
    #[test]
    fn the_card_is_the_storyboards_exact_version() {
        let board = crate::STORYBOARD.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(board.contains(&format!("\"{K02}\"")), "{K02}");
    }

    /// The sums print as the panel writes them, the short gap's whole and
    /// the long gap's shortened, in the bars' order.
    #[test]
    fn the_sums_print_in_the_bars_order() {
        let f = facts::facts().unwrap();
        let n = twelve_minutes::numbers(&f).unwrap();
        let people = twelve_minutes::passengers().unwrap();
        let waits = |g: usize| -> Vec<String> { in_gap(&people, g).iter().map(|&i| n.waits[i].clone()).collect() };
        assert_eq!(sum_text(&waits(0), &n.gap_totals[0]), "1½ + ½ = 2");
        assert_eq!(sum_text(&waits(1), &n.gap_totals[1]), "9½ + 8½ + … + ½ = 50");
    }

    /// Each gap's average is one figure of the guess's other form, so the
    /// coloring finds it; a form where an average also stands for another
    /// figure is refused.
    #[test]
    fn each_average_is_colored_once() {
        let f = facts::facts().unwrap();
        let mut n = twelve_minutes::numbers(&f).unwrap();
        let (text, colors) = other_form(&n).unwrap();
        assert_eq!(text, "= (1 + 5) ÷ 2");
        assert_eq!(colors.len(), 2);
        n.gap_means[0] = n.gap_count.clone();
        assert!(other_form(&n).is_err());
    }

    /// The two ghosts ring the long gap's first and last passengers (your
    /// two mornings, F-M1), half a minute from a bus, and each ring clears
    /// that bus's marker.
    #[test]
    fn the_ghosts_ring_your_mornings_clear_of_the_markers() {
        let f = facts::facts().unwrap();
        let n = twelve_minutes::numbers(&f).unwrap();
        let people = twelve_minutes::passengers().unwrap();
        assert_eq!(people[n.mornings[0]].t, model::unlucky_morning());
        assert_eq!(people[n.mornings[1]].t, model::lucky_morning());
        for &i in &n.mornings {
            let x = at(people[i].t.to_f64(), 0.0)[0];
            for b in twelve_minutes::buses() {
                let apart = (x - at(b as f64, 0.0)[0]).abs();
                assert!(
                    apart >= GHOST_R + MARKER_W / 2.0,
                    "{} min from the bus at {b}",
                    people[i].t
                );
            }
        }
    }

    /// The panel, the averages and the morning labels stay apart and on
    /// screen; the panel stays right of the stretch; the bars' labels stay
    /// clear of the averages; everything stays above K02's lower-third (two
    /// lines, its top near y = −2.78). Widths are estimated, not measured:
    /// 0.0045 units a point a character, a line 0.6 of its size tall.
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let f = facts::facts().unwrap();
        let n = twelve_minutes::numbers(&f).unwrap();
        let people = twelve_minutes::passengers().unwrap();
        let w = |s: &str, pt: f64| s.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        type Box4 = (f64, f64, f64, f64);
        let centred = |s: &str, pt: f64, at: Vec3| -> Box4 {
            (
                at[0] - w(s, pt) / 2.0,
                at[0] + w(s, pt) / 2.0,
                at[1] - h(pt),
                at[1] + h(pt),
            )
        };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -2.78 && a.3 < 4.0;
        let ask = format!("{ASK_HEAD} {} {ASK_TAIL}", n.people);
        let verdict = format!(
            "{} ÷ {} = {} {MIN} = {}",
            n.total,
            n.people,
            n.mean,
            min_s(&n.mean_clock).unwrap()
        );
        let (other, _) = other_form(&n).unwrap();
        let p = |i: usize| v(PANEL_X, row(i));
        let ring = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred("pause for more time", 18.0, v(THINK_AT[0], THINK_AT[1] - 0.62 - 0.3));
        let mut v_box = centred(&verdict, PANEL_SIZE, p(VERDICT_ROW));
        v_box = (
            v_box.0 - BOX_PAD,
            v_box.1 + BOX_PAD,
            v_box.2 - BOX_PAD,
            v_box.3 + BOX_PAD,
        );
        let waits = |g: usize| -> Vec<String> { in_gap(&people, g).iter().map(|&i| n.waits[i].clone()).collect() };
        // the question, its ring and note are up alone, then the panel
        let asking = [centred(&ask, PANEL_SIZE, p(ASK_ROW)), ring, note];
        let panel = [
            centred(&sum_text(&waits(0), &n.gap_totals[0]), PANEL_SIZE, p(SUM_ROWS[0])),
            centred(&sum_text(&waits(1), &n.gap_totals[1]), PANEL_SIZE, p(SUM_ROWS[1])),
            centred(
                &format!("{} + {} = {} {OF_WAITING}", n.gap_totals[0], n.gap_totals[1], n.total),
                PANEL_SIZE,
                p(SUM_ROWS[2]),
            ),
            v_box,
            centred(
                &format!("{GUESS} {} = {} {MIN}", n.mean_gap, n.half_mean_gap),
                PANEL_SIZE,
                p(GUESS_ROW),
            ),
            centred(&other, PANEL_SIZE, p(OTHER_ROW)),
            centred(
                &format!("{} {PASSENGERS}", n.gap_people[0]),
                COUNT_SIZE,
                v(COUNT_XS[0], row(COUNT_ROW)),
            ),
            centred(
                &format!("{TIMES} {}", n.people_ratio),
                COUNT_SIZE,
                v(COUNT_XS[1], row(COUNT_ROW)),
            ),
            centred(
                &format!("{} {PASSENGERS}", n.gap_people[1]),
                COUNT_SIZE,
                v(COUNT_XS[2], row(COUNT_ROW)),
            ),
        ];
        let stretch_right = at(PERIOD as f64, 0.0)[0];
        for group in [&asking[..], &panel[..]] {
            for (i, &a) in group.iter().enumerate() {
                assert!(on_screen(a) && a.0 > stretch_right, "{a:?}");
                for &b in &group[i + 1..] {
                    assert!(apart(a, b), "{a:?} {b:?}");
                }
            }
        }
        // the bars' labels, the averages and the mornings' clocks
        let mut picture: Vec<Box4> = people
            .iter()
            .zip(&n.waits)
            .map(|(q, s)| {
                let top = bar_top(q);
                centred(s, BAR_LABEL_SIZE, v(top[0], top[1] + BAR_LABEL_DY))
            })
            .collect();
        for k in 0..2 {
            let top = bar_top(&people[n.mornings[k]]);
            picture.push(centred(
                &n.morning_waits[k],
                SMALL,
                v(top[0], top[1] + BAR_LABEL_DY + MORNING_DY),
            ));
        }
        picture.push(centred(
            &format!("{AVERAGE} {} {MIN}", n.gap_means[0]),
            SMALL,
            SHORT_MEAN_AT,
        ));
        picture.push(centred(
            &format!("{AVERAGE} {} {MIN}", n.gap_means[1]),
            SMALL,
            LONG_MEAN_AT,
        ));
        for (i, &a) in picture.iter().enumerate() {
            assert!(on_screen(a) && a.1 < stretch_right + 0.3, "{a:?}");
            for &b in &picture[i + 1..] {
                assert!(apart(a, b), "{a:?} {b:?}");
            }
        }
        // each average clears every bar: the staircase falls a unit a
        // minute's width to the right, so a label's lower left corner is
        // the point to test
        for q in &people {
            let top = bar_top(q);
            for (g, mean) in [SHORT_MEAN_AT, LONG_MEAN_AT].into_iter().enumerate() {
                let label = centred(&format!("{AVERAGE} {} {MIN}", n.gap_means[g]), SMALL, mean);
                let x = top[0];
                if x >= label.0 && x <= label.1 {
                    assert!(top[1] + BAR_LABEL_DY + h(BAR_LABEL_SIZE) < label.2, "bar at {}", q.t);
                }
            }
        }
        // "× 5 in an hour" clears the long bracket's right end and the last
        // clock label
        let per_hour = format!("{TIMES} {} {IN_AN_HOUR}", n.cycles);
        assert!(PER_HOUR_AT[0] > twelve_minutes::bracket_points(OFFSET as f64, PERIOD as f64)[3][0]);
        assert!(PER_HOUR_AT[0] + w(&per_hour, SMALL) < 7.1);
        assert!(PER_HOUR_AT[0] > stretch_right + w("12 min", SMALL) / 2.0);
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            TIMES, IN_AN_HOUR, AVERAGE, ASK_HEAD, ASK_TAIL, AND_SO_ON, OF_WAITING, GUESS, PASSENGERS, K02,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
