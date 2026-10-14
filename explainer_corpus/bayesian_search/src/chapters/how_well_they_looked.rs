//! Chapter 13, how well they looked: the Navy's map reached the ship two
//! months into the search, and every pass since June was then scored against
//! it with the update the viewer did with dots, as the paper printed it
//! (13_1); by October the ship's numbers had searched the wreck's own cell
//! ninety-four percent (13_2); what went wrong was the estimate of how well
//! they were looking (13_3, a question to carry), a strip believed eight
//! hundred feet wide (13_4) that measured, afterwards, nothing for the sonar
//! and about two hundred feet for the camera and the magnetic detector
//! (13_5); so the better estimate was sixty-five percent, and the dots
//! relight, the one time in the video (13_6); and she was found where the
//! sled went back to a scrap of metal photographed that summer, beside the
//! map's top cell (13_7).
//!
//! The strips are drawn to scale against the printed cell (a mile north to
//! south); the curve is chapter 09's, with the two coverages the paper's
//! two sets of numbers come to. Every number is a fact (`F-S.*`, `F-R.F6.*`,
//! `F-H.feet_per_sea_mile`) or follows from them; the tests redo the paper's
//! update from the facts' own table figures.

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::lower_third;
use explainer_kit::stats::{Data, permutation};
use explainer_kit::{Facts, Frame2, Kit, Palette, Timed, fade_all, rect, v};
use fmn::library::{SpanMapData, TransformMatchingTex};
use fmn::prelude::*;

use super::draw::{
    Heat, Lattice, QUIZ_LINES_Y, Spoken, VEIL_OPACITY, carry_pause, fade_in_all, fade_in_lagged, fade_out_all,
    faded_line, fail, flat, last_known, line_style, map as lit_blue, out as slate, say, scatter, search as teal,
    true_place, veil, wrong,
};
use crate::facts::{cited, facts};
use crate::model::{days, koopman, y1968};

pub struct HowWellTheyLooked {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 7] = ["13_1", "13_2", "13_3", "13_4", "13_5", "13_6", "13_7"];

// the chapter's fixed words (numbers are built from facts)
const SEARCH_BEGINS: &str = "search begins";
const MAP_ARRIVES: &str = "the map arrives";
const FOUND_WORD: &str = "found";
const OLD_LETTERS: &str = r"p(1-d) \div (1-pd)";
const NEW_LETTERS: &str = r"L' = L(1-E) \div (1-SEP)";
const L_WORDS: &str = "L: the chance she is in the cell";
const E_WORDS: &str = "E: how thoroughly it has been searched";
const SEP_WORDS: &str = "SEP: the chance the search so far would have found her";
const PAPER: &str = "Richardson and Stone,";
const F6_PER_100: &str = "F6, per 100 dots";
const OUT_WORD: &str = "out,";
const STILL_LIT: &str = "still lit";
const BELIEVED: &str = "believed at sea:";
const MEASURED: &str = "measured afterwards:";
const SONAR: &str = "sonar";
const CAMERA: &str = "camera";
const MAGNETOMETER: &str = "magnetometer";
const FEET: &str = "ft";
const X_LABEL: &str = "coverage: strip × track ÷ area";
const Y_LABEL: &str = "chance a search of F6 saw it";
const TIMES_OVER: &str = "times over";
const BELIEVED_COVERAGE: &str = "believed:";
const MEASURED_COVERAGE: &str = "measured:";
const BEFORE_ANY: &str = "before any search";
const BY_SHIP: &str = "by the ship’s numbers";
const BY_MEASURED: &str = "by the measured ones";
const PHOTOGRAPHED: &str = "photographed in the summer; not found again";
const EFFORT_LABEL: &str = "Richardson and Stone:";
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

// ------------------------------------------------------------------ geometry

/// The 1968 map's centre and a row's height.
const MAP_AT: Vec3 = [-4.6, 0.9, 0.0];
const MAP_CELL_H: f64 = 0.2;
/// The wreck's cell, enlarged: its centre and its height, which stands for
/// the printed cell's one mile north to south (so a strip's width in feet is
/// drawn to the same scale as the cell).
const BIG_AT: Vec3 = [2.3, 0.55, 0.0];
const BIG_H: f64 = 3.0;
/// Where the four strips of 13_4 and 13_5 stand in the enlarged cell, as
/// heights from its centre: the believed one, the camera's, the
/// magnetometer's, and the sonar's baseline.
const ROWS: [f64; 4] = [0.85, 0.1, -0.45, -0.95];
/// The dots in the enlarged cell, and a dot's radius.
const DOTS_IN_F6: usize = 100;
const DOT_R: f64 = 0.075;
/// The hour line's, here the date line's, ends and height: the right end
/// leaves room for "28 October 1968", centred under it, inside the frame.
const LINE_X: (f64, f64) = (-6.2, 5.8);
const LINE_Y: f64 = -1.45;
/// The curve's y label stands this far over the window's top, above the
/// "believed" point's words.
const Y_LABEL_RISE: f64 = 0.75;
/// The believed strip's two labels stand over and on its band, and the
/// measured header over the camera's: three lines that never meet.
const BELIEVED_WORDS_RISE: f64 = 0.33;
const BELIEVED_VALUE_DROP: f64 = 0.02;
const MEASURED_HEADER_RISE: f64 = 0.4;
/// The scrap of metal's words: this far right of the map and this far over
/// the metal.
const METAL_WORDS_GAP: f64 = 0.35;
const METAL_WORDS_RISE: f64 = 0.25;
/// The chapter 09 curve's window, at the left of the enlarged cell.
fn curve_window() -> Frame2 {
    Frame2 {
        center: v(-3.7, 0.55),
        width: 5.2,
        height: 3.2,
        x: (0.0, 3.5),
        y: (0.0, 100.0),
    }
}

/// The enlarged cell's width: the printed cell is 0.84 as wide as it is
/// high.
fn big_w() -> f64 {
    BIG_H * y1968::CELL_MILES.1.f64() / y1968::CELL_MILES.0.f64()
}

/// A strip `feet` wide as a height on the enlarged cell, to scale against
/// the cell's mile of `feet_per_mile` feet.
fn band_h(feet: f64, feet_per_mile: f64) -> f64 {
    BIG_H * feet / feet_per_mile
}

/// "1968-06-10" as (1968, 6, 10).
fn parse_iso(s: &str) -> Option<(i64, i64, i64)> {
    let mut it = s.split('-').map(|p| p.parse::<i64>().ok());
    Some((it.next()??, it.next()??, it.next()??))
}

/// "1968-06-10" as "10 June", or with the year as "28 October 1968".
fn day_month(iso: &str, year: bool) -> Option<String> {
    let (y, m, d) = parse_iso(iso)?;
    let name = MONTHS.get(usize::try_from(m - 1).ok()?)?;
    Some(if year {
        format!("{d} {name} {y}")
    } else {
        format!("{d} {name}")
    })
}

/// Where the date line puts the date `iso`, between the search's beginning
/// (left end) and the wreck's finding (right end).
fn date_x(iso: &str) -> Option<f64> {
    let at = parse_iso(iso)?;
    let (a, b) = (cited::SEARCH_BEGAN, cited::FOUND);
    let share = days::between(a, at) as f64 / days::between(a, b) as f64;
    Some(LINE_X.0 + (LINE_X.1 - LINE_X.0) * share)
}

/// The `i`-th number of a fact that is a comma-separated list.
fn nth(f: &Facts, key: &str, i: usize) -> Result<u32, String> {
    f.get(key)
        .ok_or_else(|| format!("no fact {key}"))?
        .split(',')
        .nth(i)
        .ok_or_else(|| format!("{key} has no item {i}"))?
        .trim()
        .parse()
        .map_err(|_| format!("{key} item {i} is not a number"))
}

/// A fact's value, as the scene reads it.
fn fact(f: &Facts, key: &str) -> fmn::Result<String> {
    f.get(key)
        .map(str::to_owned)
        .ok_or_else(|| fail(format!("no fact {key}")))
}

/// How the hundred dots of the enlarged cell go out and come back: the
/// order they go out in (the dots the ship's numbers put out), the ones
/// left lit, and the ones the measured numbers light again.
struct Dots {
    at: Vec<Vec3>,
    /// Indices of the dots that go out, in the order they go.
    out: Vec<usize>,
    /// Indices of the dots still lit once the ship's numbers have searched.
    lit: Vec<usize>,
    /// Indices that relight: the last of `out` to go.
    relit: Vec<usize>,
    /// Where the true place is: a corner of the cell, next to the map's top
    /// cell (04_6).
    ring: Vec3,
}

/// How far a dot that stays lit stands from the true place's ring, so the
/// ring is seen among them.
const RING_CLEAR: f64 = 0.45;

fn dots(searched_at_sea: u32, searched_after: u32) -> Result<Dots, String> {
    let (w, h) = (big_w(), BIG_H);
    let at: Vec<Vec3> = scatter(DOTS_IN_F6, 0.9 * w, 0.9 * h, 0x13_6F6)
        .into_iter()
        .map(|(dx, dy)| v(BIG_AT[0] + dx, BIG_AT[1] + dy))
        .collect();
    let ring = v(BIG_AT[0] - w / 2.0 + 0.2, BIG_AT[1] + h / 2.0 - 0.2);
    let order = permutation(DOTS_IN_F6, &mut Data::new(0x13_94));
    let left = DOTS_IN_F6 - searched_at_sea as usize;
    let lit: Vec<usize> = order
        .iter()
        .copied()
        .filter(|&i| (at[i][0] - ring[0]).hypot(at[i][1] - ring[1]) >= RING_CLEAR)
        .take(left)
        .collect();
    if lit.len() != left {
        return Err(format!("{left} dots stand clear of the ring: only {}", lit.len()));
    }
    let out: Vec<usize> = order.iter().copied().filter(|i| !lit.contains(i)).collect();
    let still_out = searched_after as usize;
    let relight = out
        .len()
        .checked_sub(still_out)
        .ok_or_else(|| format!("the measured numbers put out {still_out} of {}", out.len()))?;
    let relit = out[out.len() - relight..].to_vec();
    Ok(Dots {
        at,
        out,
        lit,
        relit,
        ring,
    })
}

/// The morphing formula: the 07_3 line as letters and the paper's line, with
/// the glyphs that stand for the same thing keyed alike so each letter
/// becomes its own: p to L, d to E, and the denominator's p and d to S and
/// E. The left-hand L' is keyed apart, so only the old chance becomes L.
fn formula(kit: &Kit, stage: &mut Stage<'_>, pt: f64, at: Vec3) -> fmn::Result<(Mob, SpanMapData, Mob, SpanMapData)> {
    let colors_old = [("p", lit_blue()), ("d", teal())];
    let colors_new = [("L", lit_blue()), ("SEP", teal()), ("E", teal())];
    let (old, old_spans) = kit.tex_spans(stage, OLD_LETTERS, pt, Palette::ink(), &colors_old, at)?;
    let (new, new_spans) = kit.tex_spans(stage, NEW_LETTERS, pt, Palette::ink(), &colors_new, at)?;
    let old_keys = keys(&old_spans);
    let ps: Vec<usize> = positions(&old_keys, "p");
    let ds: Vec<usize> = positions(&old_keys, "d");
    if ps.len() != 2 || ds.len() != 2 {
        return Err(fail(format!(
            "{OLD_LETTERS} has {} p and {} d glyphs, not two of each: {old_keys:?}",
            ps.len(),
            ds.len()
        )));
    }
    let mut old_spans = old_spans;
    for (i, key) in [(ps[0], "L"), (ds[0], "E"), (ps[1], "S"), (ds[1], "E")] {
        old_spans = rekey(&old_spans, i, key);
    }
    let new_keys = keys(&new_spans);
    let ls = positions(&new_keys, "L");
    let mut new_spans = new_spans;
    if let Some(&first) = ls.first() {
        new_spans = rekey(&new_spans, first, "L'");
    }
    Ok((old, old_spans, new, new_spans))
}

/// The span map with entry `index` keyed `key` instead of its source text,
/// so `TransformMatchingTex` pairs it with the glyph of the other formula
/// that has that key (or, with a key of its own, with none).
fn rekey(spans: &SpanMapData, index: usize, key: &str) -> SpanMapData {
    let mut source = spans.source.to_string();
    let start = source.len();
    source.push_str(key);
    let mut entries = spans.entries.clone();
    if let Some(e) = entries.get_mut(index) {
        e.start = start;
        e.end = source.len();
    }
    SpanMapData {
        source: source.into(),
        entries,
    }
}

/// The source text of each glyph of a span map.
fn keys(spans: &SpanMapData) -> Vec<&str> {
    spans
        .entries
        .iter()
        .map(|e| spans.source.get(e.start..e.end).unwrap_or(""))
        .collect()
}

/// The places in `keys` of the glyphs whose source is exactly `key`.
fn positions(keys: &[&str], key: &str) -> Vec<usize> {
    keys.iter()
        .enumerate()
        .filter(|(_, k)| **k == key)
        .map(|(i, _)| i)
        .collect()
}

/// The last sentence of `line` as a question to carry: set in ink in the
/// caption band like the quiz's question, no ring. Detached.
fn carry_last(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<Mob> {
    let words = line
        .sentence_texts()
        .pop()
        .ok_or_else(|| fail(format!("{} has no sentences", line.id)))?;
    kit.text(stage, &words, 26.0, Palette::ink(), v(0.0, QUIZ_LINES_Y[0]))
}

/// A band across the enlarged cell, `feet` of its mile high, centred at
/// height `y`: one flat fill, detached.
fn band(stage: &mut Stage<'_>, y: f64, feet: f64, per_mile: f64, color: Srgb) -> fmn::Result<Mob> {
    let (w, h) = (big_w(), band_h(feet, per_mile));
    let r = rect(v(BIG_AT[0] - w / 2.0, y - h / 2.0), w, h, flat(color, 0.55))?;
    Ok(stage.arena_mut().add(r))
}

/// Text left-aligned at `at`, in `color`, detached.
fn words(kit: &Kit, stage: &mut Stage<'_>, s: &str, pt: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    kit.text_left(stage, s, pt, color, at)
}

impl SceneConstruct for HowWellTheyLooked {
    fn name(&self) -> &str {
        "13_how_well_they_looked"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts().map_err(fail)?;
        let searched_sea = nth(&f, "F-S.F6.searched.at_sea", 1).map_err(fail)?;
        let searched_after = nth(&f, "F-S.F6.searched.after", 1).map_err(fail)?;
        let per_mile: f64 = fact(&f, "F-H.feet_per_sea_mile")?
            .parse()
            .map_err(|_| fail("F-H.feet_per_sea_mile is not a number".into()))?;
        let believed_ft = nth(&f, "F-S.sweep.at_sea", 0).map_err(fail)?;
        let after = [
            nth(&f, "F-S.sweep.after", 0).map_err(fail)?,
            nth(&f, "F-S.sweep.after", 1).map_err(fail)?,
            nth(&f, "F-S.sweep.after", 2).map_err(fail)?,
        ];
        let d = dots(searched_sea, searched_after).map_err(fail)?;
        let (bw, bh) = (big_w(), BIG_H);

        // ------------------------------------------------------- 13_1
        // the 1968 map, the two dates, and the update as the paper printed it
        let line = say(&kit, stage, LINES[0])?;
        let (f6_row, f6_col) = y1968::at("F6").ok_or_else(|| fail("the map has no F6".into()))?;
        let map = Lattice::build(stage, MAP_AT, MAP_CELL_H, &[], Heat::LATTICE)?;
        fade_in_lagged(stage, &map.lines, 0.9, 0.6)?;
        let mut under = vec![map.anchor];
        under.extend(&map.heat);
        fade_in_all(stage, &under, 0.5)?;
        fade_in_lagged(stage, &map.dots, 1.1, 0.5)?;
        // from here on everything is drawn over the map's veil (which dims
        // the map in 13_4 and goes again in 13_7): it comes in at no opacity
        let dim = veil(stage)?;
        stage.set_opacity(dim, 0.0, true);
        stage.add_to_scene(dim)?;

        let begin_iso = fact(&f, "F-S.search_began")?;
        let arrive_iso = fact(&f, "F-S.map_arrived")?;
        let found_iso = fact(&f, "F-S.found")?;
        let (xa, xb, xc) = (
            date_x(&begin_iso).ok_or_else(|| fail("the search's first day".into()))?,
            date_x(&arrive_iso).ok_or_else(|| fail("the map's day".into()))?,
            date_x(&found_iso).ok_or_else(|| fail("the finding's day".into()))?,
        );
        let tick = |stage: &mut Stage<'_>, x: f64| {
            stage.arena_mut().add(
                polyline(vec![v(x, LINE_Y - 0.12), v(x, LINE_Y + 0.12)])
                    .style(line_style(Palette::dim(), 2.5))
                    .build(),
            )
        };
        line.wait_for(stage, "Its map reached")?;
        let axis = stage.arena_mut().add(
            polyline(vec![v(LINE_X.0, LINE_Y), v(LINE_X.1, LINE_Y)])
                .style(line_style(Palette::faint(), 2.5))
                .build(),
        );
        let (t_a, t_b) = (tick(stage, xa), tick(stage, xb));
        let begin_label = kit.text(
            stage,
            &day_month(&begin_iso, false).ok_or_else(|| fail("a date".into()))?,
            24.0,
            Palette::ink(),
            v(xa, LINE_Y - 0.4),
        )?;
        let begin_words = kit.text(stage, SEARCH_BEGINS, 22.0, Palette::dim(), v(xa, LINE_Y - 0.75))?;
        let arrive_label = kit.text(
            stage,
            &day_month(&arrive_iso, false).ok_or_else(|| fail("a date".into()))?,
            24.0,
            Palette::ink(),
            v(xb, LINE_Y - 0.4),
        )?;
        let arrive_words = kit.text(stage, MAP_ARRIVES, 22.0, Palette::dim(), v(xb, LINE_Y - 0.75))?;
        let months = fact(&f, "F-S.months.search_to_map")?;
        let span = stage.arena_mut().add(
            polyline(vec![v(xa, LINE_Y + 0.3), v(xb, LINE_Y + 0.3)])
                .style(line_style(teal(), 4.0))
                .build(),
        );
        let span_label = kit.text(
            stage,
            &format!("{months} months"),
            22.0,
            teal(),
            v((xa + xb) / 2.0, LINE_Y + 0.6),
        )?;
        let dates = [
            axis,
            t_a,
            t_b,
            begin_label,
            begin_words,
            arrive_label,
            arrive_words,
            span,
            span_label,
        ];
        fade_in_all(stage, &dates, 0.7)?;
        // the update, in the hold: 07_3's line turns part by part into the
        // one the paper printed, its words carried across
        line.wait_end(stage)?;
        let (old, old_spans, new, new_spans) = formula(&kit, stage, 44.0, v(2.4, 1.85))?;
        fade_in_all(stage, &[old], 0.6)?;
        let morph = TransformMatchingTex::new(stage.arena_mut(), old, new, &old_spans, &new_spans)
            .map_err(|e| fail(format!("07_3's line into the paper's: {e}")))?;
        let morphing = vec![stage.prepare(morph.rt(1.8))?];
        stage.play_prepared(morphing)?;
        let meaning_x = -0.9;
        let meanings = [
            (L_WORDS, "L", lit_blue(), 0.85),
            (E_WORDS, "E", teal(), 0.45),
            (SEP_WORDS, "SEP", teal(), 0.05),
        ];
        let mut explained: Vec<Mob> = Vec::new();
        for (text, key, color, y) in meanings {
            let m = kit.text_t2c(stage, text, 22.0, Palette::ink(), &[(key, color)], v(meaning_x, y))?;
            stage.move_to(m, v(meaning_x, y), LEFT);
            explained.push(m);
        }
        let cite = kit.text_left(
            stage,
            &format!(
                "{PAPER} {}, p. {}",
                fact(&f, "F-N.paper")?,
                fact(&f, "F-S.update.page")?
            ),
            22.0,
            Palette::dim(),
            v(meaning_x, -0.5),
        )?;
        explained.push(cite);
        fade_in_all(stage, &explained, 0.7)?;

        // ------------------------------------------------------- 13_2
        // the wreck's own cell, enlarged: a hundred dots, ninety-four out
        let line = say(&kit, stage, LINES[1])?;
        let mut leaving = dates.to_vec();
        leaving.push(new);
        leaving.extend(&explained);
        fade_out_all(stage, &leaving, 0.6)?;
        let cell = stage.arena_mut().add(rect(
            v(BIG_AT[0] - bw / 2.0, BIG_AT[1] - bh / 2.0),
            bw,
            bh,
            line_style(Palette::dim(), 3.0),
        )?);
        let (x0, y0, x1, y1) = map.cell_rect(f6_row, f6_col);
        let small = stage
            .arena_mut()
            .add(rect(v(x0, y0), x1 - x0, y1 - y0, line_style(Palette::ink(), 2.0))?);
        let leaders: Vec<Mob> = [
            (v(x1, y1), v(BIG_AT[0] - bw / 2.0, BIG_AT[1] + bh / 2.0)),
            (v(x1, y0), v(BIG_AT[0] - bw / 2.0, BIG_AT[1] - bh / 2.0)),
        ]
        .into_iter()
        .map(|(a, b)| {
            stage
                .arena_mut()
                .add(polyline(vec![a, b]).style(faded_line(Palette::dim(), 1.5, 0.8)).build())
        })
        .collect();
        let per_hundred = kit.text(
            stage,
            F6_PER_100,
            26.0,
            Palette::ink(),
            v(BIG_AT[0], BIG_AT[1] + bh / 2.0 + 0.4),
        )?;
        let mut frame_mobs = vec![cell, small, per_hundred];
        frame_mobs.extend(&leaders);
        fade_in_all(stage, &frame_mobs, 0.6)?;
        line.wait_for(stage, "Of every hundred dots")?;
        let lit_dots: Vec<Mob> =
            d.at.iter()
                .map(|&p| {
                    stage
                        .arena_mut()
                        .add(Dot::new().point(p).radius(DOT_R).style(flat(lit_blue(), 1.0)).build())
                })
                .collect();
        let twins: Vec<Mob> =
            d.at.iter()
                .map(|&p| {
                    stage.arena_mut().add(
                        Dot::new()
                            .point(p)
                            .radius(DOT_R * 1.12)
                            .style(flat(slate(), 1.0))
                            .build(),
                    )
                })
                .collect();
        fade_in_lagged(stage, &lit_dots, 0.9, 0.3)?;
        line.wait_for(stage, "ninety-four went out")?;
        let going: Vec<Mob> = d.out.iter().map(|&i| twins[i]).collect();
        fade_in_lagged(stage, &going, 1.4, 0.4)?;
        let tally = kit.text(
            stage,
            &format!("{} {OUT_WORD} {} {STILL_LIT}", d.out.len(), d.lit.len()),
            24.0,
            Palette::dim(),
            v(BIG_AT[0], BIG_AT[1] - bh / 2.0 - 0.35),
        )?;
        fade_in_all(stage, &[tally], 0.4)?;
        // the true place stands among the dots still lit
        line.wait_for(stage, "It was the cell")?;
        let ring = stage.arena_mut().add(
            Circle::new()
                .radius(0.14)
                .arc_center(d.ring)
                .style(line_style(true_place(), 3.5))
                .build(),
        );
        fade_in_all(stage, &[ring], 0.5)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 13_3
        // the question to carry
        let line = say(&kit, stage, LINES[2])?;
        line.wait_last(stage)?;
        let question = carry_last(&kit, stage, &line)?;
        fade_in_all(stage, &[question], 0.4)?;
        carry_pause(&kit, stage)?;
        fade_out_all(stage, &[question], 0.3)?;

        // ------------------------------------------------------- 13_4
        // the estimate of how well they were looking: a strip eight hundred
        // feet wide, to the scale of the mile the cell stands for
        let line = say(&kit, stage, LINES[3])?;
        let dimming = dim
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .set_opacity(VEIL_OPACITY)?;
        let dims = vec![stage.prepare(dimming)?];
        stage.play_prepared(dims)?;
        line.wait_for(stage, "At sea, they believed")?;
        let believed_y = BIG_AT[1] + ROWS[0];
        let believed_band = band(stage, believed_y, f64::from(believed_ft), per_mile, wrong())?;
        let label_x = BIG_AT[0] + bw / 2.0 + 0.25;
        let believed_words = words(
            &kit,
            stage,
            BELIEVED,
            22.0,
            wrong(),
            v(label_x, believed_y + BELIEVED_WORDS_RISE),
        )?;
        let believed_value = words(
            &kit,
            stage,
            &format!("{SONAR} {believed_ft} {FEET}"),
            22.0,
            wrong(),
            v(label_x, believed_y - BELIEVED_VALUE_DROP),
        )?;
        fade_in_all(stage, &[believed_band, believed_words, believed_value], 0.7)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 13_5
        // afterwards, against the wreck itself, to the same scale
        let line = say(&kit, stage, LINES[4])?;
        line.wait_for(stage, "had not seen her at all")?;
        let sonar_y = BIG_AT[1] + ROWS[3];
        let baseline = stage.arena_mut().add(
            polyline(vec![v(BIG_AT[0] - bw / 2.0, sonar_y), v(BIG_AT[0] + bw / 2.0, sonar_y)])
                .style(line_style(Palette::ink(), 2.0))
                .build(),
        );
        let sonar_words = words(
            &kit,
            stage,
            &format!("{SONAR} {} {FEET}", after[2]),
            22.0,
            Palette::ink(),
            v(label_x, sonar_y),
        )?;
        fade_in_all(stage, &[baseline, sonar_words], 0.5)?;
        line.wait_for(stage, "The camera and the magnetic detector")?;
        let (camera_y, magnetometer_y) = (BIG_AT[1] + ROWS[1], BIG_AT[1] + ROWS[2]);
        let camera_band = band(stage, camera_y, f64::from(after[0]), per_mile, teal())?;
        let magnetometer_band = band(stage, magnetometer_y, f64::from(after[1]), per_mile, teal())?;
        let measured = words(
            &kit,
            stage,
            MEASURED,
            22.0,
            Palette::dim(),
            v(label_x, camera_y + MEASURED_HEADER_RISE),
        )?;
        let camera_words = words(
            &kit,
            stage,
            &format!("{CAMERA} {} {FEET}", after[0]),
            22.0,
            teal(),
            v(label_x, camera_y),
        )?;
        let magnetometer_words = words(
            &kit,
            stage,
            &format!("{MAGNETOMETER} {} {FEET}", after[1]),
            22.0,
            teal(),
            v(label_x, magnetometer_y),
        )?;
        fade_in_all(
            stage,
            &[
                camera_band,
                magnetometer_band,
                measured,
                camera_words,
                magnetometer_words,
            ],
            0.7,
        )?;
        // chapter 09's curve, with the two coverages the two sets of
        // numbers come to
        let w = curve_window();
        let curve_pts: Vec<Vec3> = (0..=120)
            .map(|i| {
                let c = 3.5 * f64::from(i) / 120.0;
                w.p(c, 100.0 * koopman::random_search(c))
            })
            .collect();
        let mut plot: Vec<Mob> = vec![stage.arena_mut().add(w.border(Palette::faint())?)];
        plot.push(
            stage
                .arena_mut()
                .add(polyline(curve_pts).style(line_style(Palette::ink(), 3.0)).build()),
        );
        plot.push(kit.text(stage, X_LABEL, 22.0, Palette::dim(), v(w.center[0], w.bottom() - 0.35))?);
        plot.push(kit.text_left(
            stage,
            Y_LABEL,
            22.0,
            Palette::dim(),
            v(w.left(), w.top() + Y_LABEL_RISE),
        )?);
        // both points' words stand above the curve and left of their dots:
        // to the right of the measured one the two lines to the enlarged
        // cell cross the words
        for (key, label, color) in [
            ("F-S.F6.coverage.at_sea", BELIEVED_COVERAGE, wrong()),
            ("F-S.F6.coverage.after", MEASURED_COVERAGE, teal()),
        ] {
            let c: f64 = fact(&f, key)?
                .parse()
                .map_err(|_| fail(format!("{key} is not a number")))?;
            let at = w.p(c, 100.0 * koopman::random_search(c));
            plot.push(
                stage
                    .arena_mut()
                    .add(Dot::new().point(at).radius(0.12).style(flat(color, 1.0)).build()),
            );
            let text = if key.ends_with("at_sea") {
                format!("{label} {c:.1} {TIMES_OVER}")
            } else {
                format!("{label} {c:.2}")
            };
            let m = if key.ends_with("at_sea") {
                kit.text(stage, &text, 22.0, color, v(at[0] - 1.0, at[1] + 0.35))?
            } else {
                kit.text_left(stage, &text, 22.0, color, v(w.left() + 0.1, at[1] + 0.45))?
            };
            plot.push(m);
        }
        fade_in_all(stage, &plot, 0.7)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 13_6
        // the better estimate: dots relight, the one time in the video
        let line = say(&kit, stage, LINES[5])?;
        let mut done = vec![
            believed_band,
            believed_words,
            believed_value,
            baseline,
            sonar_words,
            camera_band,
            magnetometer_band,
            measured,
            camera_words,
            magnetometer_words,
        ];
        done.push(tally);
        fade_out_all(stage, &done, 0.5)?;
        line.wait_for(stage, "sixty-five percent")?;
        let coming_back: Vec<Mob> = d.relit.iter().map(|&i| twins[i]).collect();
        let relighting: Vec<Box<dyn Animation>> = coming_back
            .iter()
            .map(|&m| {
                let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                stage.prepare(gone.rt(0.8))
            })
            .collect::<fmn::Result<_>>()?;
        let tally = kit.text(
            stage,
            &format!(
                "{} {OUT_WORD} {} {STILL_LIT}",
                d.out.len() - d.relit.len(),
                d.lit.len() + d.relit.len()
            ),
            24.0,
            Palette::dim(),
            v(BIG_AT[0], BIG_AT[1] - bh / 2.0 - 0.35),
        )?;
        let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), relighting, 0.2)?.with_run_time(1.6);
        // the new count comes in as the first dot relights, so the number is
        // on the screen while it is being said
        let count_in = fade_in(stage.arena_mut(), tally, ORIGIN, 1.0)?;
        let together: Vec<Box<dyn Animation>> = vec![stage.prepare(group)?, stage.prepare(count_in.rt(0.6))?];
        stage.play_prepared(together)?;
        line.wait_for(stage, "And at that width")?;
        let chances = [
            (BEFORE_ANY, "F-R.F6.percent", Palette::ink()),
            (BY_SHIP, "F-S.F6.chance.at_sea", wrong()),
            (BY_MEASURED, "F-S.F6.chance.after", teal()),
        ];
        let mut under_cell: Vec<Mob> = Vec::new();
        for (i, (label, key, color)) in chances.iter().enumerate() {
            under_cell.push(kit.text(
                stage,
                &format!("{label} {}%", fact(&f, key)?),
                22.0,
                *color,
                v(BIG_AT[0], BIG_AT[1] - bh / 2.0 - 0.8 - 0.34 * i as f64),
            )?);
        }
        fade_in_all(stage, &under_cell, 0.6)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 13_7
        // she was found beside the top cell, where the sled went back to a
        // scrap of metal
        let line = say(&kit, stage, LINES[6])?;
        let mut leaving = plot.clone();
        leaving.extend(&under_cell);
        leaving.push(tally);
        leaving.extend(&lit_dots);
        leaving.push(cell);
        leaving.extend(&leaders);
        leaving.extend([per_hundred, small, ring]);
        // every twin still in the scene
        leaving.extend(d.out.iter().map(|&i| twins[i]).filter(|m| !coming_back.contains(m)));
        let lifting = dim
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .set_opacity(0.0)?;
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(lifting)?];
        for &m in &leaving {
            let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(gone.rt(0.8))?);
        }
        stage.play_prepared(anims)?;
        // the map again: the top cell's outline, the ring in F6's corner,
        // and the scrap of metal on the border of F5 and F6
        let (e_row, e_col) = y1968::at("E5").ok_or_else(|| fail("the map has no E5".into()))?;
        let (ex0, ey0, ex1, ey1) = map.cell_rect(e_row, e_col);
        let top_cell = stage.arena_mut().add(rect(
            v(ex0, ey0),
            ex1 - ex0,
            ey1 - ey0,
            line_style(Palette::ink(), 2.0),
        )?);
        let (fx0, fy0, fx1, fy1) = map.cell_rect(f6_row, f6_col);
        let place = v(fx0 + 0.25 * (fx1 - fx0), fy1 - 0.25 * (fy1 - fy0));
        let wreck = stage.arena_mut().add(
            Circle::new()
                .radius(0.045)
                .arc_center(place)
                .style(line_style(true_place(), 2.5))
                .build(),
        );
        let metal_at = v((fx0 + fx1) / 2.0, fy1);
        let r = 0.05;
        let metal = stage.arena_mut().add(
            polyline(vec![
                v(metal_at[0], metal_at[1] + r),
                v(metal_at[0] + r, metal_at[1]),
                v(metal_at[0], metal_at[1] - r),
                v(metal_at[0] - r, metal_at[1]),
                v(metal_at[0], metal_at[1] + r),
            ])
            .style(line_style(last_known(), 2.5))
            .build(),
        );
        // its words stand clear of the map, to its right, on a leader: beside
        // the metal they were set over the map's cells and their first
        // letters were lost in them
        let map_right = MAP_AT[0] + map.size().0 / 2.0;
        let words_at = v(map_right + METAL_WORDS_GAP, metal_at[1] + METAL_WORDS_RISE);
        let metal_words = words(&kit, stage, PHOTOGRAPHED, 22.0, Palette::dim(), words_at)?;
        let leader = stage.arena_mut().add(
            polyline(vec![
                v(metal_at[0] + r, metal_at[1] + 0.02),
                v(words_at[0] - 0.1, words_at[1]),
            ])
            .style(faded_line(Palette::dim(), 1.5, 0.8))
            .build(),
        );
        let e5 = kit.text(stage, "E5", 20.0, Palette::ink(), v(ex0 - 0.2, (ey0 + ey1) / 2.0))?;
        let f6 = kit.text(stage, "F6", 20.0, Palette::ink(), v(fx0 - 0.2, (fy0 + fy1) / 2.0 - 0.1))?;
        fade_in_all(stage, &[top_cell, wreck, metal, leader, metal_words, e5, f6], 0.6)?;
        // the date line comes back with the day she was found
        line.wait_for(stage, "The sled had gone back")?;
        // the paper's own reason stands under the picture from the sentence
        // it explains, long enough to read its twenty-six words
        let effort = lower_third(
            &kit,
            stage,
            EFFORT_LABEL,
            "effort went to F6 “because of proximity to the piece of metal and the fact that this was a region with high a priori [prior] probability”",
        )?;
        fade_in_all(stage, &[effort], 0.5)?;
        let found_axis = stage.arena_mut().add(
            polyline(vec![v(LINE_X.0, LINE_Y), v(LINE_X.1, LINE_Y)])
                .style(line_style(Palette::faint(), 2.5))
                .build(),
        );
        let found_tick = stage.arena_mut().add(
            polyline(vec![v(xc, LINE_Y - 0.12), v(xc, LINE_Y + 0.12)])
                .style(line_style(Palette::ink(), 3.0))
                .build(),
        );
        let found_date = kit.text(
            stage,
            &day_month(&found_iso, true).ok_or_else(|| fail("a date".into()))?,
            24.0,
            Palette::ink(),
            v(xc, LINE_Y - 0.4),
        )?;
        let found_label = kit.text(stage, FOUND_WORD, 22.0, Palette::dim(), v(xc, LINE_Y - 0.75))?;
        fade_in_all(stage, &[found_axis, found_tick, found_date, found_label], 0.6)?;
        line.wait_end(stage)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Q, q};

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut fmn::scene::NullSceneSink).expect("the probe scene runs");
    }

    /// The chapter speaks exactly the script's 13_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("13_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// 13_1 and 13_7: the dates are the facts', the date line puts the
    /// map's day where its place among the days is, and a day is said the
    /// way the screen writes it.
    #[test]
    fn the_dates_are_the_facts_and_sit_where_their_days_do() {
        let f = facts().unwrap();
        assert_eq!(day_month(f.get("F-S.search_began").unwrap(), false).unwrap(), "10 June");
        assert_eq!(
            day_month(f.get("F-S.map_arrived").unwrap(), false).unwrap(),
            "12 August"
        );
        assert_eq!(day_month(f.get("F-S.found").unwrap(), true).unwrap(), "28 October 1968");
        assert_eq!(f.get("F-S.months.search_to_map"), Some("2"));
        let (a, b, c) = (
            date_x(f.get("F-S.search_began").unwrap()).unwrap(),
            date_x(f.get("F-S.map_arrived").unwrap()).unwrap(),
            date_x(f.get("F-S.found").unwrap()).unwrap(),
        );
        assert_eq!(
            (a, c),
            (LINE_X.0, LINE_X.1),
            "the line runs from the first day to the finding"
        );
        let days_to_map: f64 = f.get("F-S.days.search_to_map").unwrap().parse().unwrap();
        let days_to_found = days::between(cited::SEARCH_BEGAN, cited::FOUND) as f64;
        assert!(((b - a) / (c - a) - days_to_map / days_to_found).abs() < 1e-12);
        assert!(a < b && b < c);
        assert!(parse_iso("1968-13-40").is_some() && day_month("1968-13-40", false).is_none());
    }

    /// 13_2 and 13_6: of the hundred dots the ship's numbers put out
    /// ninety-four, six stay lit and stand clear of the ring; the measured
    /// numbers leave sixty-five out, so the last twenty-nine to go come
    /// back, and thirty-five are lit: the facts' counts, exactly.
    #[test]
    fn the_dots_go_out_and_relight_as_the_facts_say() {
        let f = facts().unwrap();
        let (sea, after) = (
            nth(&f, "F-S.F6.searched.at_sea", 1).unwrap(),
            nth(&f, "F-S.F6.searched.after", 1).unwrap(),
        );
        let d = dots(sea, after).unwrap();
        assert_eq!(d.at.len(), 100);
        assert_eq!(d.out.len(), 94);
        assert_eq!(
            d.out.len().to_string(),
            f.get("F-S.F6.searched.at_sea").unwrap().split(',').nth(1).unwrap()
        );
        assert_eq!(d.lit.len().to_string(), f.get("F-S.F6.left.at_sea.percent").unwrap());
        let lit_after = d.lit.len() + d.relit.len();
        assert_eq!(lit_after.to_string(), f.get("F-S.F6.left.after.percent").unwrap());
        assert_eq!(d.relit.len(), 94 - 65);
        // every dot is in exactly one of the lit and the out; a relit dot is one that went out
        let mut all: Vec<usize> = d.out.iter().chain(&d.lit).copied().collect();
        all.sort_unstable();
        assert_eq!(all, (0..100).collect::<Vec<_>>());
        assert!(d.relit.iter().all(|i| d.out.contains(i)));
        // the ring stands among dots that stay lit, none on top of it
        for &i in &d.lit {
            let at = d.at[i];
            assert!((at[0] - d.ring[0]).hypot(at[1] - d.ring[1]) >= RING_CLEAR);
        }
        // the edges: nothing lit leaves nothing to relight and nothing to
        // clear of the ring; more lit than stand clear of it, or more kept
        // out than went out, is refused
        let none = dots(100, 100).unwrap();
        assert!(none.lit.is_empty() && none.relit.is_empty() && none.out.len() == 100);
        assert!(dots(0, 0).is_err(), "a hundred dots cannot all stand clear of the ring");
        assert!(dots(95, 99).is_err(), "more kept out than went out is refused");
    }

    /// 13_4 and 13_5: the strips are to the scale of a mile the cell stands
    /// for, 6,076 feet, so eight hundred feet is a little over an eighth of
    /// it; and the facts' coverages are the curve's at the facts' percents.
    #[test]
    fn the_strips_are_to_scale_and_the_curve_gives_the_papers_percents() {
        let f = facts().unwrap();
        let per_mile: f64 = f.get("F-H.feet_per_sea_mile").unwrap().parse().unwrap();
        assert!((per_mile - 1852.0 / 0.3048).abs() < 1.0);
        let sea = f64::from(nth(&f, "F-S.sweep.at_sea", 0).unwrap());
        let after: Vec<f64> = (0..3)
            .map(|i| f64::from(nth(&f, "F-S.sweep.after", i).unwrap()))
            .collect();
        let believed = band_h(sea, per_mile);
        assert!((believed / BIG_H - 800.0 / 6076.0).abs() < 1e-3);
        assert!(band_h(after[0], per_mile) < band_h(after[1], per_mile) && band_h(after[1], per_mile) < believed);
        assert_eq!(band_h(after[2], per_mile), 0.0, "the sonar saw nothing: no bar");
        // the four rows of 13_4 and 13_5 stand inside the cell, each clear of
        // the next, with the heights their feet give them
        let feet = [sea, after[0], after[1], after[2]];
        for (row, ft) in ROWS.iter().zip(feet) {
            let half = band_h(ft, per_mile) / 2.0;
            assert!(row.abs() + half < BIG_H / 2.0, "row {row} with {ft} ft");
        }
        for w in 0..3 {
            let gap = ROWS[w] - ROWS[w + 1];
            let reach = band_h(feet[w], per_mile) / 2.0 + band_h(feet[w + 1], per_mile) / 2.0;
            assert!(gap > reach, "rows {w} and {} touch: {gap} against {reach}", w + 1);
        }
        // coverage to the percent of a cell searched
        for (key, searched) in [
            ("F-S.F6.coverage.at_sea", nth(&f, "F-S.F6.searched.at_sea", 1).unwrap()),
            ("F-S.F6.coverage.after", nth(&f, "F-S.F6.searched.after", 1).unwrap()),
        ] {
            let c: f64 = f.get(key).unwrap().parse().unwrap();
            let on_curve = 100.0 * koopman::random_search(c);
            assert!(
                (on_curve - f64::from(searched)).abs() < 0.5,
                "{key}: {on_curve} against {searched}"
            );
        }
        assert_eq!(
            format!(
                "{:.1}",
                f.get("F-S.F6.coverage.at_sea").unwrap().parse::<f64>().unwrap()
            ),
            "2.8"
        );
    }

    /// 13_6: the three chances under the cell are the paper's update,
    /// L (1 − E) ÷ (1 − SEP), worked from the table figures the facts hold
    /// for 9 October: on the ship's numbers 0.19%, on the measured ones
    /// 0.54%; both below the 0.85% the map began with, the ship's the lower.
    #[test]
    fn the_chances_under_the_cell_are_the_papers_update() {
        let f = facts().unwrap();
        let prior = Q::new(i128::from(y1968::points("F6").unwrap()), 10_000);
        assert_eq!(prior.percent(2), f.get("F-R.F6.percent").unwrap());
        let hundredth = |n: u32| q(i128::from(n), 100);
        let update = |e: u32, sep: u32| prior * (Q::ONE - hundredth(e)) / (Q::ONE - hundredth(sep));
        let sea = update(
            nth(&f, "F-S.F6.searched.at_sea", 1).unwrap(),
            nth(&f, "F-S.SEP.at_sea", 1).unwrap(),
        );
        let measured = update(
            nth(&f, "F-S.F6.searched.after", 1).unwrap(),
            nth(&f, "F-S.SEP.after", 1).unwrap(),
        );
        assert_eq!(sea.percent(2), f.get("F-S.F6.chance.at_sea").unwrap());
        assert_eq!(measured.percent(2), f.get("F-S.F6.chance.after").unwrap());
        assert!(
            sea < measured && measured < prior,
            "the measured numbers keep more of F6 than the ship's"
        );
        // the ship's numbers kept six hundredths of F6, the measured ones thirty-five
        assert_eq!(f.get("F-S.F6.left.ratio"), Some("5.8"));
    }

    /// 13_5 to 13_7, the narrated draft's overlaps: the curve's y label
    /// clears the believed point's words; the believed strip's two labels
    /// and the measured header are at least a third of a unit apart; the
    /// scrap of metal's words stand right of the map and end inside the
    /// frame; the date line's last label is inside the frame.
    #[test]
    fn the_labels_of_13_5_to_13_7_clear_each_other_and_the_frame() {
        use super::super::draw::{CHAR_UNITS_PER_POINT, LAT_COLS};
        let f = facts().unwrap();
        let (w, pt) = (curve_window(), 22.0);
        let c: f64 = f.get("F-S.F6.coverage.at_sea").unwrap().parse().unwrap();
        let at = w.p(c, 100.0 * koopman::random_search(c));
        let (believed_top, title_bottom) = (at[1] + 0.35 + 0.15, w.top() + Y_LABEL_RISE - 0.15);
        assert!(believed_top < title_bottom, "{believed_top} against {title_bottom}");
        // the measured point's words begin at the plot's left and stand over
        // the curve all the way to their end (the curve is below them there:
        // it rises to the right, to the dot under them)
        let c_after: f64 = f.get("F-S.F6.coverage.after").unwrap().parse().unwrap();
        let after = w.p(c_after, 100.0 * koopman::random_search(c_after));
        let measured_end = w.left()
            + 0.1
            + format!("{MEASURED_COVERAGE} {c_after:.2}").chars().count() as f64 * pt * CHAR_UNITS_PER_POINT;
        let end_c = (measured_end - w.left()) / w.width * 3.5;
        let curve_at_end = w.p(end_c, 100.0 * koopman::random_search(end_c))[1];
        let words_bottom = after[1] + 0.45 - 0.12;
        assert!(
            words_bottom > curve_at_end.max(after[1]) + 0.1,
            "words over {words_bottom}, curve at their end {curve_at_end}, dot {}",
            after[1]
        );
        let (believed_y, camera_y) = (BIG_AT[1] + ROWS[0], BIG_AT[1] + ROWS[1]);
        let heights = [
            believed_y + BELIEVED_WORDS_RISE,
            believed_y - BELIEVED_VALUE_DROP,
            camera_y + MEASURED_HEADER_RISE,
            camera_y,
        ];
        for pair in heights.windows(2) {
            assert!(pair[0] - pair[1] >= 0.3, "{heights:?}");
        }
        let map_right =
            MAP_AT[0] + LAT_COLS as f64 * MAP_CELL_H * y1968::CELL_MILES.1.f64() / y1968::CELL_MILES.0.f64() / 2.0;
        let words_end = map_right + METAL_WORDS_GAP + PHOTOGRAPHED.chars().count() as f64 * pt * CHAR_UNITS_PER_POINT;
        assert!(words_end < FRAME_X_RADIUS - 0.3, "{words_end}");
        let date_half = "28 October 1968".chars().count() as f64 * 24.0 * CHAR_UNITS_PER_POINT / 2.0;
        assert!(LINE_X.1 + date_half < FRAME_X_RADIUS - 0.2, "{}", LINE_X.1 + date_half);
    }

    /// 13_1: the formula's glyphs are the ones the morph is keyed on: two p
    /// and two d in the 07_3 line, and an L (twice), an E (twice), an S and
    /// a P in the paper's; the pieces pair (the morph builds) and the
    /// left-hand L' is the one keyed apart. A real scene, headless.
    #[test]
    fn the_morph_pairs_each_letter_with_its_own() {
        run(|stage| {
            let kit = Kit::new()?;
            let (_plain, old) = kit.tex_spans(stage, OLD_LETTERS, 44.0, Palette::ink(), &[], ORIGIN)?;
            let k = keys(&old);
            assert_eq!(positions(&k, "p").len(), 2, "{k:?}");
            assert_eq!(positions(&k, "d").len(), 2, "{k:?}");
            let (a, a_spans, b, b_spans) = formula(&kit, stage, 44.0, ORIGIN)?;
            let nk = keys(&b_spans);
            assert_eq!(positions(&nk, "L").len(), 1, "one L pairs: {nk:?}");
            assert_eq!(positions(&nk, "L'").len(), 1, "the left-hand L is keyed apart: {nk:?}");
            assert_eq!(positions(&nk, "E").len(), 2);
            assert_eq!(positions(&nk, "S").len(), 1);
            let ok = keys(&a_spans);
            assert_eq!(positions(&ok, "L").len(), 1);
            assert_eq!(positions(&ok, "E").len(), 2);
            assert_eq!(positions(&ok, "S").len(), 1);
            TransformMatchingTex::new(stage.arena_mut(), a, b, &a_spans, &b_spans)
                .map_err(|e| fail(format!("the morph: {e}")))?;
            Ok(())
        });
    }
}
