//! Chapter 04, ten thousand dots: the Navy's map was made the same way. A
//! hydrogen bomb off Palomares and the idea that stayed (04_1); nine
//! stories, each with a weight (04_2); a computer plays them out ten
//! thousand times and the dots rain onto the 1968 lattice under a counter
//! (04_3); the dots in each cell become its printed number and the cell
//! takes its heat (04_4); one cell stands out, one dot in eight (04_5);
//! almost: the wreck lay in a neighbour holding fewer than one in a hundred
//! (04_6); the map leaves and the made-up sea returns (04_7).
//!
//! The picture is the real map as printed: ten thousand dots, each one run
//! of nine weighted stories. A dot is a run of what two men believed, and
//! the lower-third says so, against reading 1,250 as a count of sinkings.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, fade_all, thousands, v};
use fmn::prelude::*;

use super::draw::{
    Heat, Lattice, Sea, balanced_lines, carry_pause, carry_question, fade_in_all, fade_in_lagged, fade_out_all,
    faded_line, fail, flat, last_known, line_style, map, panel, pause, quote_card, say, sea_dots, sea_slot, sea_split,
    text_lines, true_place, tween_linear,
};
use crate::facts::cited;
use crate::model::{Q, sea, y1968};

pub struct TenThousandDots {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 7] = ["04_1", "04_2", "04_3", "04_4", "04_5", "04_6", "04_7"];

// the phrases the pictures land on (a test finds each in its line)
const P_PALOMARES: &str = "lost in the sea off Palomares";
const P_FISHERMAN: &str = "where a fisherman watched it fall";
const P_NINE: &str = "set out nine stories";
const P_WEIGHT: &str = "They gave each story a weight";
const P_COMPUTER: &str = "had a computer play";
const P_TEN_THOUSAND: &str = "Ten thousand dots";
const P_COUNTED: &str = "counted the dots";
const P_PRINTED: &str = "This is the map they printed";
const P_STANDS_OUT: &str = "One cell stands out";
const P_E_FIVE: &str = "They called it E five";
const P_ALMOST: &str = "Almost";
const P_NEIGHBOUR: &str = "in a neighbouring cell";
const P_YARDS: &str = "within two hundred and sixty yards";
const P_ALONE: &str = "But the map alone";
const P_METHOD: &str = "we need a method";

// fixed words
const PALOMARES: &str = "Palomares, Spain,";
const FOUND_BY: &str = "found by the submersible Alvin,";
const RUN_WORDS: &str = "each run drew its own course, speed and position";
/// The map's source, as two lines whose break is not inside "Figure 2".
const SOURCE: [&str; 2] = ["Richardson and Stone, 1971, Figure 2.", "Each cell about a mile across"];
const SOURCE_PRECISELY: &str = "10,000 runs of nine weighted stories, not 10,000 sinkings";
const SCENARIO: &str = "scenario";
/// What the card says after the date: two sentences, so the card breaks
/// between them.
const SCENARIO_WORDS: &str =
    "nine stories, each with a weight (the paper\u{2019}s word: credence). Not printed in the 1971 paper.";
const OF: &str = "of";
const ONE_IN: &str = "1 in";
const FEWER: &str = "fewer than 1 in 100";
const QUOTED_YARDS: &str = "within 260 yards of the edge of rectangle E5";
const IN_PAPER: &str = "as the paper\u{2019}s asterisk sits";
/// What the idea did at Palomares (R2: M9, D15): three lines under the date
/// of the find. It kept the books; it did not find the bomb.
const IDEA_LINES: [&str; 3] = [
    "what the idea did there: kept the books",
    "on which areas had been searched,",
    "and how well. It did not find the bomb.",
];
const IDEA_AT: [f64; 2] = [-2.2, -1.35];
const IDEA_STEP: f64 = 0.3;
/// The room the left half has (the date line above is shrunk to it).
const IDEA_ROOM: f64 = 4.4;

/// The lattice's place and size: left of the column, above the question's
/// band (nine tenths of the sixteen rows fit the stage).
const LATTICE_AT: [f64; 2] = [-2.9, 0.45];
const ROW_H: f64 = 0.38;
/// The right-hand column: its middle and the room a line has.
const COLUMN_X: f64 = 4.4;
const COLUMN_ROOM: f64 = 4.8;
/// The run counter's height, how far a 64 pt number reaches below its
/// centre, and the top of the inset of E5 and F6 (04_6), which stands under
/// the counter (the narrated draft's 04_6 had the inset's top edge through
/// the number).
const COUNTER_Y: f64 = 2.3;
const COUNTER_REACH: f64 = 0.3;
const INSET_TOP: f64 = COUNTER_Y - COUNTER_REACH - 0.05;
/// The coast of Palomares, drawn small on the left, and its two marks.
const COAST: [[f64; 2]; 6] = [
    [-6.3, 3.0],
    [-5.6, 1.8],
    [-5.9, 0.6],
    [-5.2, -0.6],
    [-5.6, -1.8],
    [-4.9, -2.4],
];
const FISHERMAN_AT: [f64; 2] = [-3.5, 0.5];
const BOMB_AT: [f64; 2] = [-3.25, 0.25];
/// The nine story cards: three by three on the right.
const CARD_X: [f64; 3] = [2.6, 4.2, 5.8];
const CARD_Y: [f64; 3] = [2.2, 0.95, -0.3];
const CARD_SIZE: [f64; 2] = [1.4, 1.0];
/// How long the dots rain and the counter runs.
const RAIN_S: f64 = 4.0;

/// A date in words: "17 January 1966" (every month a chapter names).
pub fn long_date(date: (i64, i64, i64)) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    format!("{} {} {}", date.2, MONTHS[(date.1 - 1) as usize], date.0)
}

/// The right-hand column's lines, 0.34 apart. One group, detached.
fn block(kit: &Kit, stage: &mut Stage<'_>, lines: &[String], size: f64, color: Srgb, top: f64) -> fmn::Result<Mob> {
    text_lines(kit, stage, lines, size, color, COLUMN_X, top, 0.34)
}

/// `words` as at most two balanced lines of `chars` (a column line).
fn column_lines(words: &str, chars: usize) -> Vec<String> {
    balanced_lines(words, chars)
}

/// A fact's value, or the error that names it.
fn fact(facts: &explainer_kit::Facts, key: &str) -> fmn::Result<String> {
    facts
        .get(key)
        .map(str::to_owned)
        .ok_or_else(|| fail(format!("no fact {key}")))
}

/// A cell of the 1968 map by its printed name.
pub fn cell(name: &str) -> fmn::Result<(usize, usize)> {
    y1968::at(name).ok_or_else(|| fail(format!("no cell {name} on the 1968 map")))
}

impl SceneConstruct for TenThousandDots {
    fn name(&self) -> &str {
        "04_ten_thousand_dots"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let dim = Palette::dim();
        let ink = Palette::ink();

        // 04_1: the Navy's map was made the same way; Craven and a hydrogen
        // bomb off Palomares, and a few blank story cards
        let line = say(&kit, stage, LINES[0])?;
        let coast = stage.arena_mut().add(
            polyline(COAST.iter().map(|p| v(p[0], p[1])))
                .style(faded_line(dim, 3.0, 0.9))
                .build(),
        );
        let mut cards: Vec<Mob> = Vec::new();
        let mut weights: Vec<Mob> = Vec::new();
        for &y in &CARD_Y {
            for &x in &CARD_X {
                let back = Rectangle::new()
                    .width(CARD_SIZE[0])
                    .height(CARD_SIZE[1])
                    .corner_radius(0.1)
                    .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
                    .build()?
                    .moved_to(v(x, y));
                cards.push(stage.arena_mut().add(back));
                // its weight, drawn as an outline: the number is not known
                let w = Rectangle::new()
                    .width(CARD_SIZE[0] - 0.5)
                    .height(0.16)
                    .style(faded_line(dim, 2.0, 1.0))
                    .build()?
                    .moved_to(v(x, y - 0.25));
                weights.push(stage.arena_mut().add(w));
            }
        }
        // the chapter opens on the first story cards with its first sentence
        // (they came at "A Navy scientist", 2.6 s into the line in the
        // finished read, over a bare stage)
        let first_three: Vec<Mob> = cards.iter().take(3).copied().collect();
        fade_in_lagged(stage, &first_three, 1.0, 0.6)?;
        line.wait_for(stage, P_PALOMARES)?;
        let bomb_date = kit.text(
            stage,
            &format!("{PALOMARES} {}", long_date(cited::PALOMARES)),
            24.0,
            dim,
            v(-2.2, 1.35),
        )?;
        let off_shore = stage.arena_mut().add(
            Dot::new()
                .point(v(BOMB_AT[0], BOMB_AT[1]))
                .radius(0.1)
                .style(flat(Palette::ink(), 1.0))
                .build(),
        );
        fade_in_all(stage, &[coast, off_shore, bomb_date], 0.7)?;
        line.wait_end(stage)?;

        // 04_2: the bomb was found near where a fisherman watched it fall,
        // but the idea stayed; in July of 1968 nine stories, each weighted
        let line = say(&kit, stage, LINES[1])?;
        line.wait_for(stage, P_FISHERMAN)?;
        // a small open diamond where he pointed, and a white ring beside it
        let d = 0.13;
        let diamond = stage.arena_mut().add(
            polyline(vec![
                v(FISHERMAN_AT[0], FISHERMAN_AT[1] + d),
                v(FISHERMAN_AT[0] + d, FISHERMAN_AT[1]),
                v(FISHERMAN_AT[0], FISHERMAN_AT[1] - d),
                v(FISHERMAN_AT[0] - d, FISHERMAN_AT[1]),
                v(FISHERMAN_AT[0], FISHERMAN_AT[1] + d),
            ])
            .style(line_style(last_known(), 3.0))
            .build(),
        );
        let ring = stage.arena_mut().add(
            Circle::new()
                .radius(0.17)
                .style(line_style(true_place(), 3.5))
                .build()
                .moved_to(v(BOMB_AT[0], BOMB_AT[1])),
        );
        let found = kit.text(
            stage,
            &format!("{FOUND_BY} {}", long_date(cited::PALOMARES_FOUND)),
            24.0,
            dim,
            v(-2.2, -0.8),
        )?;
        super::draw::fit_width(stage, found, IDEA_ROOM);
        // what the idea did there: it kept the books, and did not find it
        let did = text_lines(
            &kit,
            stage,
            &IDEA_LINES.map(str::to_owned),
            20.0,
            dim,
            IDEA_AT[0],
            IDEA_AT[1],
            IDEA_STEP,
        )?;
        fade_in_all(stage, &[diamond, ring, found, did], 0.7)?;
        line.wait_for(stage, P_NINE)?;
        // the card that dates and names the nine comes in with them and
        // stays through the rest of the line: twenty words need more than the
        // second the line's end left them (the narrated draft's 04_2)
        let scenario = quote_card(
            &kit,
            stage,
            SCENARIO,
            &format!("Set out {}: {SCENARIO_WORDS}", long_date(cited::SCENARIOS_SET)),
        )?;
        fade_in_all(stage, &[scenario], 0.5)?;
        let rest: Vec<Mob> = cards.iter().skip(3).copied().collect();
        fade_in_lagged(stage, &rest, 1.6, 0.8)?;
        line.wait_for(stage, P_WEIGHT)?;
        fade_in_lagged(stage, &weights, 1.2, 0.6)?;
        line.wait_end(stage)?;
        pause(stage, 1.0)?;
        fade_out_all(stage, &[scenario], 0.4)?;

        // 04_3: a computer plays the stories out ten thousand times: the
        // dots rain onto the empty lattice, a counter running to 10,000
        let mut clearing: Vec<Mob> = vec![coast, off_shore, bomb_date, diamond, ring, found, did];
        clearing.extend(&cards);
        clearing.extend(&weights);
        fade_out_all(stage, &clearing, 0.7)?;
        let map_1968 = Lattice::build(stage, v(LATTICE_AT[0], LATTICE_AT[1]), ROW_H, &[], Heat::LATTICE)?;
        fade_in_lagged(stage, &map_1968.lines, 0.8, 0.6)?;
        let line = say(&kit, stage, LINES[2])?;
        line.wait_for(stage, P_COMPUTER)?;
        let runs = stage.add_value_tracker(0.0);
        kit.live_count(stage, runs, 64.0, ink, v(COLUMN_X, COUNTER_Y))?;
        let run_words = block(&kit, stage, &column_lines(RUN_WORDS, 34), 22.0, dim, 1.5)?;
        fade_in_all(stage, &[run_words], 0.5)?;
        // the dots rain: each cell's dots drop in, the middle first, while
        // the counter runs to the number of runs
        let total = f64::from(y1968::REPLICATIONS);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let mut falls: Vec<Box<dyn Animation>> = Vec::new();
        for &g in &map_1968.dots {
            let a = fade_in(stage.arena_mut(), g, v(0.0, -0.5), 1.0)?;
            falls.push(stage.prepare(a)?);
        }
        let rain = AnimationGroup::with_lag_ratio(stage.arena_mut(), falls, 0.9)?.with_run_time(RAIN_S);
        anims.push(Box::new(rain));
        anims.push(stage.prepare(tween_linear(runs, total, RAIN_S)?)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, P_TEN_THOUSAND)?;
        line.wait_end(stage)?;

        // 04_4: then he counted the dots in every cell; this is the map
        // they printed: each cell's dots become its number, and it takes
        // its heat
        let line = say(&kit, stage, LINES[3])?;
        line.wait_for(stage, P_COUNTED)?;
        let numbers = map_1968.numbers(&kit, stage, 11.0)?;
        let mut under = vec![map_1968.anchor];
        under.extend(&map_1968.heat);
        fade_in_all(stage, &under, 0.8)?;
        let mut swap: Vec<Box<dyn Animation>> = Vec::new();
        for &g in &map_1968.dots {
            let a = fade_out(stage.arena_mut(), g, ORIGIN, 1.0)?;
            swap.push(stage.prepare(a.rt(0.9))?);
        }
        let ripple = {
            let mut ins: Vec<Box<dyn Animation>> = Vec::new();
            for &m in &numbers {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                ins.push(stage.prepare(a)?);
            }
            AnimationGroup::with_lag_ratio(stage.arena_mut(), ins, 0.8)?.with_run_time(1.8)
        };
        swap.push(Box::new(ripple));
        stage.play_prepared(swap)?;
        line.wait_for(stage, P_PRINTED)?;
        fade_out_all(stage, &[run_words], 0.3)?;
        let source_lines: Vec<String> = SOURCE.iter().map(|&l| l.to_owned()).collect();
        let source = block(&kit, stage, &source_lines, 22.0, dim, 1.5)?;
        fade_in_all(stage, &[source], 0.5)?;
        line.wait_end(stage)?;
        kit.hold(stage)?;
        let exact = precisely(&kit, stage, SOURCE_PRECISELY)?;
        fade_in_all(stage, &[exact], 0.5)?;
        pause(stage, 1.2)?;
        fade_out_all(stage, &[exact], 0.4)?;

        // 04_5: one cell stands out, one dot in every eight: E5. A question
        // to carry; the next highest, B7, is nearly as hot
        let line = say(&kit, stage, LINES[4])?;
        let (e5_row, e5_col) = cell("E5")?;
        let (b7_row, b7_col) = cell("B7")?;
        let outline = |stage: &mut Stage<'_>, row: usize, col: usize, color: Srgb, w: f64| -> fmn::Result<Mob> {
            let (x0, y0, x1, y1) = map_1968.cell_rect(row, col);
            let r = Rectangle::new()
                .width(x1 - x0)
                .height(y1 - y0)
                .style(faded_line(color, w, 1.0))
                .build()?
                .moved_to(v((x0 + x1) / 2.0, (y0 + y1) / 2.0));
            Ok(stage.arena_mut().add(r))
        };
        let e5_box = outline(stage, e5_row, e5_col, ink, 4.0)?;
        let b7_box = outline(stage, b7_row, b7_col, dim, 2.5)?;
        let e5_n = fact(&facts, "F-R.E5")?;
        let e5_one_in = fact(&facts, "F-R.E5.one_in")?;
        let b7_n = fact(&facts, "F-R.B7")?;
        let e5_note = kit.text(
            stage,
            &format!(
                "E5: {} {OF} {}: {ONE_IN} {e5_one_in}",
                thousands(e5_n.parse::<u64>().map_err(|_| fail("F-R.E5".into()))?),
                thousands(u64::from(y1968::REPLICATIONS))
            ),
            26.0,
            ink,
            v(COLUMN_X, 1.4),
        )?;
        super::draw::fit_width(stage, e5_note, COLUMN_ROOM);
        let b7_note = kit.text(
            stage,
            &format!(
                "B7: {}",
                thousands(b7_n.parse::<u64>().map_err(|_| fail("F-R.B7".into()))?)
            ),
            24.0,
            dim,
            v(COLUMN_X, 0.7),
        )?;
        fade_out_all(stage, &[source], 0.3)?;
        let q = carry_question(&kit, stage, LINES[4])?;
        line.wait_for(stage, P_STANDS_OUT)?;
        fade_in_all(stage, &[e5_box, e5_note], 0.6)?;
        line.wait_for(stage, P_E_FIVE)?;
        fade_in_all(stage, &[b7_box, b7_note, q], 0.6)?;
        line.wait_end(stage)?;
        carry_pause(&kit, stage)?;
        fade_out_all(stage, &[q], 0.3)?;

        // 04_6: almost: the wreck lay in a neighbouring cell, within 260
        // yards of the edge of E5; an inset of the two, four times the size
        let line = say(&kit, stage, LINES[5])?;
        line.wait_for(stage, P_ALMOST)?;
        fade_out_all(stage, &[e5_note, b7_note, b7_box], 0.4)?;
        let (cw, ch) = (4.0 * map_1968.cell_w, 4.0 * map_1968.cell_h);
        let e5_at = v(COLUMN_X - cw / 2.0, INSET_TOP - ch / 2.0);
        let f6_at = v(COLUMN_X + cw / 2.0, INSET_TOP - 1.5 * ch);
        let share = |name: &str| -> fmn::Result<f64> {
            let n = y1968::points(name).ok_or_else(|| fail(format!("no cell {name}")))?;
            Ok(f64::from(n) / f64::from(y1968::REPLICATIONS))
        };
        let mut inset: Vec<Mob> = Vec::new();
        for (name, at) in [("E5", e5_at), ("F6", f6_at)] {
            let blue = Rectangle::new()
                .width(cw)
                .height(ch)
                .style(flat(map(), Heat::LATTICE.opacity(share(name)?)))
                .build()?
                .moved_to(at);
            inset.push(stage.arena_mut().add(blue));
            let edge = Rectangle::new()
                .width(cw)
                .height(ch)
                .style(faded_line(if name == "E5" { ink } else { dim }, 3.0, 1.0))
                .build()?
                .moved_to(at);
            inset.push(stage.arena_mut().add(edge));
            inset.push(kit.text(
                stage,
                name,
                28.0,
                ink,
                v(at[0] - cw / 2.0 + 0.3, at[1] + ch / 2.0 - 0.3),
            )?);
        }
        // the white ring in the corner of F6 that touches E5, where the
        // paper's asterisk sits
        let corner = v(f6_at[0] - cw / 2.0 + 0.22, f6_at[1] + ch / 2.0 - 0.22);
        let wreck = stage.arena_mut().add(
            Circle::new()
                .radius(0.11)
                .style(line_style(true_place(), 3.5))
                .build()
                .moved_to(corner),
        );
        inset.push(wreck);
        let f6_n = fact(&facts, "F-R.F6")?;
        let f6_note = kit.text(
            stage,
            &format!("F6: {f6_n} {OF} {}: {FEWER}", thousands(u64::from(y1968::REPLICATIONS))),
            24.0,
            ink,
            v(COLUMN_X, -1.55),
        )?;
        super::draw::fit_width(stage, f6_note, COLUMN_ROOM);
        let yards = fact(&facts, "F-S.yards")?;
        let quoted = kit.text(
            stage,
            &format!("\u{201C}{}\u{201D}", QUOTED_YARDS.replace("260", &yards)),
            22.0,
            dim,
            v(COLUMN_X, -2.15),
        )?;
        super::draw::fit_width(stage, quoted, COLUMN_ROOM);
        let asterisk = kit.text(stage, IN_PAPER, 20.0, dim, v(COLUMN_X, -2.6))?;
        super::draw::fit_width(stage, asterisk, COLUMN_ROOM);
        inset.push(f6_note);
        // the neighbour is on the stage as it is named, the quoted distance
        // as it is said
        line.wait_for(stage, P_NEIGHBOUR)?;
        fade_in_lagged(stage, &inset, 1.2, 0.4)?;
        line.wait_for(stage, P_YARDS)?;
        fade_in_all(stage, &[quoted, asterisk], 0.5)?;
        line.wait_end(stage)?;

        // 04_7: but the map alone did not find her, and the search took five
        // months; the map leaves and the made-up sea comes back where it
        // stood (the sea is as large as the lattice, so "beside it" put one
        // on the other: the narrated draft's 04_7)
        let line = say(&kit, stage, LINES[6])?;
        line.wait_for(stage, P_ALONE)?;
        fade_all(stage, 0.7)?;
        let lit0: [u32; 4] = sea::prior().map(|p| (p * Q::int(sea::DOTS)).fixed(0).parse().unwrap_or(0));
        let sea_map = Sea::build(&kit, stage, lit0)?;
        let dots = sea_dots(stage, sea_split());
        let mut returning: Vec<Mob> = sea_map.mobs();
        returning.push(sea_map.tag);
        returning.extend(dots.iter().map(|d| d.dot));
        fade_in_lagged(stage, &returning, 1.2, 0.3)?;
        line.wait_for(stage, P_METHOD)?;
        let mut lit_labels = Vec::new();
        for (c, &p) in sea::prior().iter().enumerate() {
            lit_labels.push(kit.text(stage, &format!("{}%", p.percent(0)), 28.0, map(), sea_slot(c, 0))?);
        }
        fade_in_all(stage, &lit_labels, 0.5)?;
        line.wait_end(stage)?;
        // chapter 05 opens on the made-up sea, so it stays through the cut
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{CHAR_UNITS_PER_POINT, QUIZ_FRAME, QUIZ_LINES_Y};
    use super::*;
    use explainer_kit::Script;

    const PHRASES: [(&str, &str); 15] = [
        ("04_1", P_PALOMARES),
        ("04_2", P_FISHERMAN),
        ("04_2", P_NINE),
        ("04_2", P_WEIGHT),
        ("04_3", P_COMPUTER),
        ("04_3", P_TEN_THOUSAND),
        ("04_4", P_COUNTED),
        ("04_4", P_PRINTED),
        ("04_5", P_STANDS_OUT),
        ("04_5", P_E_FIVE),
        ("04_6", P_ALMOST),
        ("04_6", P_NEIGHBOUR),
        ("04_6", P_YARDS),
        ("04_7", P_ALONE),
        ("04_7", P_METHOD),
    ];

    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("04_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    #[test]
    fn every_phrase_the_pictures_land_on_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in PHRASES {
            let text = script.text(id).unwrap();
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The dates and counts the chapter prints are the facts: Palomares on
    /// 17 January 1966, found 15 March; nine stories set out on 18 July
    /// 1968; ten thousand runs; E5 1,250 (one in eight), B7 1,096, F6 85.
    #[test]
    fn the_dates_and_counts_are_the_facts() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(long_date(cited::PALOMARES), "17 January 1966");
        assert_eq!(long_date(cited::PALOMARES_FOUND), "15 March 1966");
        assert_eq!(long_date(cited::SCENARIOS_SET), "18 July 1968");
        assert_eq!(f.get("F-P.date"), Some("1966-01-17"));
        assert_eq!(f.get("F-P.found"), Some("1966-03-15"));
        assert_eq!(f.get("F-S.scenarios_set"), Some("1968-07-18"));
        assert_eq!(f.get("F-S.scenarios"), Some("9"));
        assert_eq!(cited::SCENARIOS as usize, CARD_X.len() * CARD_Y.len());
        assert_eq!(f.get("F-R.replications"), Some("10000"));
        assert_eq!(f.get("F-R.E5"), Some("1250"));
        assert_eq!(f.get("F-R.E5.one_in"), Some("8"));
        assert_eq!(f.get("F-R.B7"), Some("1096"));
        assert_eq!(f.get("F-R.F6"), Some("85"));
        assert_eq!(f.get("F-R.F6.percent"), Some("0.85"));
        assert_eq!(f.get("F-S.yards"), Some("260"));
        assert_eq!(thousands(1250u64), "1,250");
        assert_eq!(QUOTED_YARDS.replace("260", f.get("F-S.yards").unwrap()), QUOTED_YARDS);
    }

    /// 04_5 and 04_6 say "one dot in every eight", "nearly as hot",
    /// "fewer than 1 in 100", and "a neighbouring cell"; each is true of the
    /// printed numbers: E5 is the highest cell and exactly 1 in 8, B7 the
    /// next, F6 holds 85 of 10,000, and F6 shares a corner with E5.
    #[test]
    fn the_cells_are_what_the_line_says_they_are() {
        let hottest = y1968::cells().max_by_key(|&(_, _, n)| n).unwrap();
        assert_eq!((hottest.0, hottest.1), cell("E5").unwrap());
        assert_eq!(hottest.2, 1250);
        assert_eq!(y1968::REPLICATIONS % hottest.2, 0, "an exact one in eight");
        assert_eq!(y1968::REPLICATIONS / hottest.2, 8);
        assert_eq!(y1968::rank("E5"), Some(1));
        assert_eq!(y1968::rank("B7"), Some(2));
        // "nearly as hot": B7 holds seven eighths of E5
        let (e5, b7) = (y1968::points("E5").unwrap(), y1968::points("B7").unwrap());
        assert!(f64::from(b7) / f64::from(e5) > 0.85);
        // fewer than one in a hundred, at the edge: 99 would still be fewer
        // than a hundredth only below 100 of 10,000
        let f6 = y1968::points("F6").unwrap();
        assert!(f6 * 100 < y1968::REPLICATIONS);
        // 100 of 10,000 would be exactly 1 in 100, not fewer (a constant,
        // checked as the crate builds)
        const {
            assert!(100 * 100 >= y1968::REPLICATIONS);
        }
        // the neighbour shares a corner with E5: one column east, one row south
        let (er, ec) = cell("E5").unwrap();
        let (fr, fc) = cell("F6").unwrap();
        assert_eq!((fr, fc), (er + 1, ec + 1));
        assert_eq!(f6, 85);
    }

    /// The glyph pages showed the source line broken inside "Figure 2." and
    /// the scenario card missing its colon. The source is two whole lines;
    /// the card's two sentences are its two lines; apostrophes are curly.
    #[test]
    fn the_source_and_the_scenario_card_break_where_they_read() {
        assert_eq!(SOURCE[0], "Richardson and Stone, 1971, Figure 2.");
        assert!(
            SOURCE
                .iter()
                .all(|l| l.chars().count() as f64 * 22.0 * CHAR_UNITS_PER_POINT <= COLUMN_ROOM)
        );
        // the card: the label "scenario" leaves 100 characters a line
        let text = format!("Set out {}: {SCENARIO_WORDS}", long_date(cited::SCENARIOS_SET));
        let lines = balanced_lines(&text, 100);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].starts_with("Set out 18 July 1968: nine stories"));
        assert_eq!(lines[1], "Not printed in the 1971 paper.");
        for s in [SCENARIO_WORDS, IN_PAPER] {
            assert!(s.contains('\u{2019}') && !s.contains('\''), "{s}");
        }
    }

    /// R2 (M9, D15): what the idea did at Palomares is said on screen in
    /// three whole lines in the left half, under the date of the find (at
    /// -0.8) and above the scenario card's top (three lines reach -2.42).
    #[test]
    fn what_the_idea_did_at_palomares_stands_in_the_left_half() {
        assert_eq!(
            IDEA_LINES.join(" "),
            "what the idea did there: kept the books on which areas had been searched, and how well. \
             It did not find the bomb."
        );
        for l in IDEA_LINES {
            assert!(
                l.chars().count() as f64 * 20.0 * CHAR_UNITS_PER_POINT <= IDEA_ROOM,
                "{l}"
            );
        }
        const {
            assert!(IDEA_AT[1] <= -0.8 - 0.3);
            assert!(IDEA_AT[0] + IDEA_ROOM / 2.0 < CARD_X[0] - CARD_SIZE[0] / 2.0);
            let last = IDEA_AT[1] - IDEA_STEP * (IDEA_LINES.len() - 1) as f64;
            let card_top = -4.0 + 0.2 + 0.3 + 0.36 * 3.0;
            assert!(last - 0.12 > card_top);
        }
    }

    /// Cards, coast and column stand inside the stage; the lattice stands
    /// above the question's band and left of the column.
    #[test]
    fn the_pictures_sit_on_the_stage() {
        let cell_w = ROW_H * y1968::CELL_MILES.1.f64();
        let half_h = 16.0 * ROW_H / 2.0;
        let half_w = 20.0 * cell_w / 2.0;
        assert!(LATTICE_AT[1] - half_h > QUIZ_LINES_Y[0] + 0.2);
        assert!(LATTICE_AT[1] + half_h < QUIZ_FRAME.1);
        assert!(LATTICE_AT[0] + half_w < COLUMN_X - COLUMN_ROOM / 2.0);
        assert!(COLUMN_X + COLUMN_ROOM / 2.0 < QUIZ_FRAME.0);
        for &x in &CARD_X {
            assert!(x + CARD_SIZE[0] / 2.0 < QUIZ_FRAME.0);
            assert!(x - CARD_SIZE[0] / 2.0 > LATTICE_AT[0] + half_w - 0.5);
        }
        for pair in CARD_X.windows(2) {
            assert!(pair[1] - pair[0] >= CARD_SIZE[0], "cards do not touch");
        }
        for pair in CARD_Y.windows(2) {
            assert!(pair[0] - pair[1] >= CARD_SIZE[1], "cards do not touch");
        }
        // the inset's two cells, four times the lattice's, fit the column
        assert!(2.0 * 4.0 * cell_w <= COLUMN_ROOM);
        // and stand under the run counter and above the F6 note (04_6)
        const {
            assert!(INSET_TOP < COUNTER_Y - COUNTER_REACH);
            assert!(INSET_TOP - 2.0 * 4.0 * ROW_H > -1.55 + 0.12 + 0.1);
        }
        assert!(
            COAST
                .iter()
                .all(|p| p[0].abs() < QUIZ_FRAME.0 && p[1].abs() < QUIZ_FRAME.1)
        );
    }

    /// 04_1 (blank_stretches on the scored draft: the chapter opened on 3.4 s
    /// of nothing): the first story cards come in with the line's first
    /// sentence, "The Navy's map was made the same way.", not at "A Navy
    /// scientist" 2.6 s later. The line is placed on the finished read's own
    /// onsets (the scratch voice puts the second sentence at its share of
    /// the characters).
    #[test]
    fn the_chapter_opens_on_the_story_cards_with_its_first_sentence() {
        use super::super::draw::watch;
        let w = watch::run_timed("04_ten_thousand_dots", &[("04_1", &[0.0, 2.64, 8.29], 13.41)]);
        let late = w.first_arrival() - w.line_start("04_1");
        eprintln!("04_1: the first picture begins {late:.2} s after the line");
        assert!(late <= 0.1, "the first picture comes {late:.2} s after the line starts");
    }
}
