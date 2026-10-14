//! Chapter 14, four maps: three seconds of the opening shot with a strip that
//! sees nothing, and then the line alone, because an empty search only
//! tells you something if it could have found what you were looking for
//! (14_1); four searches side by side, each the last frame of its story,
//! built left to right as they are named (14_2); the three flaws that put
//! three of the maps wrong, lit in red inside their pictures, and the
//! question of what the method asks of you (14_3); the cold open's shot
//! again with each part named as its clause is said, and the update and the
//! rule in the hold, with the edge the claim has (14_4); the 99, in a
//! silence, on the card chapter 02 left (14_5); the method that now searches
//! for people who can be saved, and the motif's resolution (14_6); and the
//! receipts card (14_7, no words).
//!
//! The pictures are small redraws from primitives of what chapters 04, 11,
//! 12 and 13 show, so this chapter imports nothing private from them. Every
//! number it prints is a fact (`F-P`, `F-S`, `F-A`, `F-C`, `F-N`, `F-U`) or a
//! `model::y1968` value.

use explainer_kit::animatic::{HOLD_S, wrap};
use explainer_kit::props::{names_to_search, receipts_card};
use explainer_kit::{Kit, Palette, fade_all, thousands, v};
use fmn::prelude::*;

use super::draw::{
    Heat, Lattice, Probe, QUIZ_LINES_Y, Spoken, Strip, carry_pause, fade_in_all, fade_in_lagged, fade_out_all, fail,
    find, flat, found, last_known, line_style, map, panel as card_fill, pause, say, scatter, search, true_place,
    tween_linear, wrong,
};
use crate::facts::{facts, shown};
use crate::model::{Q, y1968};

pub struct FourMaps {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order (14_7, the receipts, has
/// no words).
pub const LINES: [&str; 6] = ["14_1", "14_2", "14_3", "14_4", "14_5", "14_6"];

// the chapter's fixed words (numbers and dates are built from facts)
const NAMES: [&str; 4] = ["Palomares", "Scorpion", "Air France 447", "John Aldridge"];
const WHAT_PUT_IT_RIGHT: [&str; 4] = [
    "found near the fisherman’s mark",
    "widths re-measured after the find",
    "a second map",
    "re-run with new evidence",
];
const FOUND: &str = "found";
const BELIEF: &str = "what you believe";
const LOOKED: &str = "how well you looked";
const UPDATE: &str = "the update:";
const UPDATE_RULE: &str = "p (1 − d) ÷ (1 − p d)";
const RULE: &str = "search where the chance of a find is greatest";
const NEVER_ADDS: &str = "the update multiplies and never adds: a place with no dots can never gain one";
const BELIEVED: &str = "believed:";
const SILENCE_LABEL: &str = "the silence";
const OFF_BY: &str = "hours off";
const MEN: &str = "men";
const RECEIPTS_CLAIM: &str = "Every number in this video is in the code, and each was checked a second way.";
const REPOSITORY: &str = "github.com/Dicklesworthstone/jeffs_math_explainer_videos";
const SYNTHETIC: &str = "narrated by synthetic speech";
/// The facts that say the day each story ended: Palomares's bomb found, the
/// Scorpion found, the Air France wreck found, and the day of John Aldridge's
/// search.
const FOUND_KEYS: [&str; 4] = ["F-P.found", "F-S.found", "F-A.found", "F-C.aldridge.date"];
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

/// The cold open's map fills the stage: a row of the lattice, scene units.
const COLD_ROW_H: f64 = 0.46;
/// 14_4's map is the same picture smaller, so the update and the rule fit
/// under it.
const BELIEF_ROW_H: f64 = 0.30;
const BELIEF_AT: Vec3 = [0.0, 1.15, 0.0];
/// The update, the rule and the small print of 14_4's hold, as heights.
const HOLD_LINES_Y: [f64; 3] = [-2.0, -2.65, -3.3];
/// The strip's bar against its cell's width, as the cold open draws it.
const BAR_SHARE: f64 = 0.22;
/// The strip's crossing and the dots going out, seconds.
const CROSS_S: f64 = 1.0;
const MOVE_S: f64 = 0.9;

/// The four panels: centres across the stage, their common height and width.
const PANEL_X: [f64; 4] = [-5.2, -1.75, 1.75, 5.2];
const PANEL_Y: f64 = 0.7;
const PANEL_W: f64 = 3.1;
const PANEL_H: f64 = 2.7;

/// Scorpion's panel: a block of the printed map, columns D to G and rows 4
/// to 6, a cell's width and height in scene units.
const BLOCK_COLS: [usize; 4] = [6, 7, 8, 9];
const BLOCK_ROWS: [usize; 3] = [5, 6, 7];
const BLOCK_CELL: (f64, f64) = (0.5, 0.6);

/// Air France's panel: a disc's radius and where the two stand from the
/// panel's centre.
const DISC_R: f64 = 0.62;
const DISC_X: f64 = 0.76;
/// North-northeast, in degrees east of north.
const NNE_DEG: f64 = 22.5;

/// How many listening tracks the red flaw draws across the disc (a drawing
/// choice: the paper's ships made many).
const TRACKS: usize = 3;
/// How many dots the small clouds of Aldridge's panel and 14_6 carry.
const SMALL_CLOUD: usize = 36;

/// Chapter 02's card for the 99, exactly as `ninety_nine.rs` sets it (its
/// `RING_AT`, `RING_R`, and `NUMBER_DROP` of 1.3 under the ring's centre):
/// a white ring, and "99 men" in ink under it. A test reads that file's
/// source and fails if the card there moves.
const CARD_RING_AT: Vec3 = [0.0, 0.6, 0.0];
const CARD_RING_R: f64 = 0.5;
const CARD_PT: f64 = 64.0;
const CARD_TEXT_Y: f64 = -0.7;

/// The silence after the sentence that names the 99: nothing sounds, no
/// music breath and no cue (storyboard, "Sound plan"), seconds.
const SILENCE_S: f64 = HOLD_S;

/// One panel's centre.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pane {
    at: Vec3,
}

impl Pane {
    fn nth(i: usize) -> Self {
        Self {
            at: v(PANEL_X[i], PANEL_Y),
        }
    }

    fn p(&self, dx: f64, dy: f64) -> Vec3 {
        v(self.at[0] + dx, self.at[1] + dy)
    }

    /// The panel's name, the day its story ended and the label under it, as
    /// heights from the centre.
    fn title_y(&self) -> f64 {
        -PANEL_H / 2.0 - 0.3
    }

    fn date_y(&self) -> f64 {
        -PANEL_H / 2.0 - 0.65
    }

    fn label_y(&self) -> f64 {
        -PANEL_H / 2.0 - 1.0
    }
}

/// "17 January 1966" from "1966-01-17".
fn long_date(iso: &str) -> Option<String> {
    let mut parts = iso.split('-');
    let (y, m, d) = (parts.next()?, parts.next()?, parts.next()?);
    let month = MONTHS.get(m.parse::<usize>().ok()?.checked_sub(1)?)?;
    Some(format!("{} {month} {y}", d.parse::<u32>().ok()?))
}

/// The centre of block cell (`row`, `col`) of Scorpion's panel.
fn block_cell(pane: Pane, row: usize, col: usize) -> Option<Vec3> {
    let ci = BLOCK_COLS.iter().position(|&c| c == col)? as f64;
    let ri = BLOCK_ROWS.iter().position(|&r| r == row)? as f64;
    let (w, h) = BLOCK_CELL;
    Some(pane.p((ci - 1.5) * w, 0.15 + (1.0 - ri) * h))
}

/// Where the ring of Air France's panel stands from the last known position,
/// in scene units: `share` of the disc's radius (the found place's distance
/// over the circle's radius), north-northeast.
fn ring_offset(share: f64) -> (f64, f64) {
    let a = NNE_DEG.to_radians();
    (share * DISC_R * a.sin(), share * DISC_R * a.cos())
}

/// The share of a cell's height a strip of `feet` feet is, a cell being a
/// sea mile of `feet_per_sea_mile` feet north to south.
fn strip_share(feet: f64, feet_per_sea_mile: f64) -> f64 {
    feet / feet_per_sea_mile
}

/// The text of a "hours off" flaw, from the fact's hours.
fn hours_off_words(hours: &str) -> String {
    format!("{hours} {OFF_BY}")
}

// --------------------------------------------------------------- primitives

fn dot(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    stage
        .arena_mut()
        .add(Dot::new().point(at).radius(r).style(flat(color, 1.0)).build())
}

/// A hollow ring.
fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb, width: f64) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(r)
            .arc_center(at)
            .style(line_style(color, width))
            .build(),
    )
}

/// An open diamond, a last known position.
fn diamond(stage: &mut Stage<'_>, at: Vec3, r: f64) -> Mob {
    stage.arena_mut().add(
        polyline(vec![
            v(at[0], at[1] + r),
            v(at[0] + r, at[1]),
            v(at[0], at[1] - r),
            v(at[0] - r, at[1]),
            v(at[0], at[1] + r),
        ])
        .style(line_style(last_known(), 2.5))
        .build(),
    )
}

/// A closed outline of `w` by `h` centred at `at`, detached.
fn outline(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, color: Srgb, width: f64) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(w)
        .height(h)
        .style(line_style(color, width))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

/// The coast along the top of a panel: land in `faint`, its edge uneven.
fn coast(stage: &mut Stage<'_>, pane: Pane) -> Mob {
    let top = PANEL_H / 2.0 - 0.08;
    let (l, r) = (-PANEL_W / 2.0 + 0.08, PANEL_W / 2.0 - 0.08);
    let pts = vec![
        pane.p(l, top),
        pane.p(r, top),
        pane.p(r, top - 0.42),
        pane.p(0.9, top - 0.55),
        pane.p(0.3, top - 0.38),
        pane.p(-0.4, top - 0.62),
        pane.p(-1.0, top - 0.45),
        pane.p(l, top - 0.55),
    ];
    stage
        .arena_mut()
        .add(Polygon::new(pts).style(flat(Palette::faint(), 0.9)).build())
}

/// A panel's card.
fn card(stage: &mut Stage<'_>, pane: Pane) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(PANEL_W)
        .height(PANEL_H)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(card_fill(), 0.96)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(pane.at);
    Ok(stage.arena_mut().add(r))
}

/// A small cloud of lit dots around `at`.
fn small_cloud(stage: &mut Stage<'_>, at: Vec3, n: usize, w: f64, h: f64, seed: u64, color: Srgb) -> Vec<Mob> {
    scatter(n, w, h, seed)
        .into_iter()
        .map(|(dx, dy)| dot(stage, v(at[0] + dx, at[1] + dy), 0.04, color))
        .collect()
}

/// Palomares: the coast, the place the fisherman pointed (an open diamond:
/// not the true place) and the white ring beside it.
fn palomares(stage: &mut Stage<'_>, pane: Pane) -> Vec<Mob> {
    let mark = pane.p(-0.15, -0.05);
    vec![
        coast(stage, pane),
        diamond(stage, mark, 0.09),
        ring(stage, v(mark[0] + 0.24, mark[1] - 0.12), 0.07, true_place(), 2.5),
    ]
}

/// Scorpion: a block of the printed map, E5 outlined, its number and F6's,
/// and the white ring in the corner of F6 that touches E5.
fn scorpion(kit: &Kit, stage: &mut Stage<'_>, pane: Pane) -> fmn::Result<Vec<Mob>> {
    let (w, h) = BLOCK_CELL;
    let mut out = Vec::new();
    let total = f64::from(y1968::REPLICATIONS);
    for &row in &BLOCK_ROWS {
        for &col in &BLOCK_COLS {
            let at = block_cell(pane, row, col).ok_or_else(|| fail(format!("no block cell {row},{col}")))?;
            let points = f64::from(y1968::POINTS[row][col]);
            if points > 0.0 {
                let fill = Rectangle::new()
                    .width(w)
                    .height(h)
                    .style(flat(map(), Heat::LATTICE.opacity(points / total)))
                    .build()?
                    .moved_to(at);
                out.push(stage.arena_mut().add(fill));
            }
            out.push(outline(stage, at, w, h, Palette::faint(), 1.2)?);
        }
    }
    let (e5, f6) = (e5_cell()?, f6_cell()?);
    let e5_at = block_cell(pane, e5.0, e5.1).ok_or_else(|| fail("E5 is not in the block".into()))?;
    let f6_at = block_cell(pane, f6.0, f6.1).ok_or_else(|| fail("F6 is not in the block".into()))?;
    out.push(outline(stage, e5_at, w, h, Palette::ink(), 3.0)?);
    out.push(kit.text(
        stage,
        &thousands(u32::from(y1968::POINTS[e5.0][e5.1])),
        18.0,
        Palette::ink(),
        e5_at,
    )?);
    out.push(kit.text(
        stage,
        &y1968::POINTS[f6.0][f6.1].to_string(),
        18.0,
        Palette::ink(),
        v(f6_at[0] + 0.04, f6_at[1] - 0.1),
    )?);
    // the white ring: in F6's corner that E5's corner touches
    out.push(ring(
        stage,
        v(f6_at[0] - w / 2.0 + 0.1, f6_at[1] + h / 2.0 - 0.1),
        0.07,
        true_place(),
        2.5,
    ));
    Ok(out)
}

/// The cell E5 of the printed map, and F6.
fn e5_cell() -> fmn::Result<(usize, usize)> {
    y1968::at("E5").ok_or_else(|| fail("the 1968 map has no E5".into()))
}

fn f6_cell() -> fmn::Result<(usize, usize)> {
    y1968::at("F6").ok_or_else(|| fail("the 1968 map has no F6".into()))
}

/// Air France: the two discs, the last known position at each one's centre
/// and the ring at the same place on both.
fn air_france(stage: &mut Stage<'_>, pane: Pane, share: f64) -> Vec<Mob> {
    let (dx, dy) = ring_offset(share);
    let mut out = Vec::new();
    for sx in [-DISC_X, DISC_X] {
        let c = pane.p(sx, 0.1);
        out.push(
            stage.arena_mut().add(
                Circle::new()
                    .radius(DISC_R)
                    .arc_center(c)
                    .style(flat(map(), 0.22))
                    .build(),
            ),
        );
        out.push(ring(stage, c, DISC_R, Palette::dim(), 2.0));
        out.push(diamond(stage, c, 0.05));
        out.push(ring(stage, v(c[0] + dx, c[1] + dy), 0.05, true_place(), 2.5));
    }
    out
}

/// John Aldridge: the coast, the boat's track south, the cloud moved south of
/// where the first map had it, and the green point he was found at. With
/// `helicopter`, the track that flew through the dots, in the search's teal.
fn aldridge(stage: &mut Stage<'_>, pane: Pane, helicopter: bool) -> Vec<Mob> {
    let found_at = pane.p(-0.15, -0.35);
    let mut out = vec![coast(stage, pane)];
    out.push(
        stage.arena_mut().add(
            polyline(vec![pane.p(-0.4, 0.9), pane.p(-0.3, 0.1), pane.p(-0.2, -1.05)])
                .style(line_style(Palette::dim(), 2.0))
                .build(),
        ),
    );
    out.extend(small_cloud(stage, found_at, SMALL_CLOUD, 1.0, 0.9, 2013, map()));
    if helicopter {
        out.push(
            stage.arena_mut().add(
                polyline(vec![pane.p(-0.9, -0.05), pane.p(0.5, -0.62)])
                    .style(line_style(search(), 3.0))
                    .build(),
            ),
        );
    }
    out.push(dot(stage, found_at, 0.09, found()));
    out
}

/// Where the words of the card for the 99 are centred, under the ring.
fn card_words_at() -> Vec3 {
    v(CARD_RING_AT[0], CARD_TEXT_Y)
}

/// The card for the 99: a white ring and "99 men".
fn ninety_nine(kit: &Kit, stage: &mut Stage<'_>, crew: &str) -> fmn::Result<Vec<Mob>> {
    let r = ring(stage, CARD_RING_AT, CARD_RING_R, true_place(), 4.0);
    let words = kit.text(
        stage,
        &format!("{crew} {MEN}"),
        CARD_PT,
        Palette::ink(),
        card_words_at(),
    )?;
    Ok(vec![r, words])
}

/// The cold open's blind search: its brightest cell, searched by a search
/// that sees nothing.
fn blind_probe() -> fmn::Result<Probe> {
    let (row, col, _) = y1968::cells()
        .max_by_key(|&(_, _, n)| n)
        .ok_or_else(|| fail("the 1968 map has no cells".into()))?;
    Ok(Probe {
        row,
        col,
        seeing: Q::ZERO,
    })
}

/// The cold open's search, as 01_2 makes it: the same cell, the model's
/// two in three.
fn cold_open_probe() -> fmn::Result<Probe> {
    Ok(Probe {
        seeing: shown::MAP_SEEING,
        ..blind_probe()?
    })
}

/// The last sentence of `line` as a question to carry, in ink in the caption
/// band, as the quiz sets its question. Detached.
fn carry_last(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<Mob> {
    let words = line
        .sentence_texts()
        .pop()
        .ok_or_else(|| fail(format!("{} has no sentences", line.id)))?;
    kit.text(stage, &words, 26.0, Palette::ink(), v(0.0, QUIZ_LINES_Y[0]))
}

/// Everything of one panel but its flaw: the card, the picture, its name and
/// what put the map right. The panel's flaw is built when it is lit.
fn panel_mobs(kit: &Kit, stage: &mut Stage<'_>, i: usize, picture: Vec<Mob>, found_on: &str) -> fmn::Result<Vec<Mob>> {
    let pane = Pane::nth(i);
    let mut all = vec![card(stage, pane)?];
    all.extend(picture);
    all.push(kit.text(stage, NAMES[i], 24.0, Palette::ink(), pane.p(0.0, pane.title_y()))?);
    all.push(kit.text(
        stage,
        &format!("{FOUND} {found_on}"),
        20.0,
        Palette::ink(),
        pane.p(0.0, pane.date_y()),
    )?);
    all.push(kit.text(
        stage,
        WHAT_PUT_IT_RIGHT[i],
        20.0,
        Palette::dim(),
        pane.p(0.0, pane.label_y()),
    )?);
    Ok(all)
}

impl SceneConstruct for FourMaps {
    fn name(&self) -> &str {
        "14_four_maps"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let all = facts().map_err(fail)?;
        let fact = |key: &str| -> fmn::Result<String> {
            all.get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };

        // ------------------------------------------------------- 14_1
        // three seconds of the opening shot, with a strip that sees nothing:
        // it crosses the cell, no dot goes out, no cell changes; then the
        // line alone, in ink
        let blind = blind_probe()?;
        let map1 = Lattice::build(stage, v(0.0, 0.0), COLD_ROW_H, &[blind], Heat::LATTICE)?;
        fade_in_lagged(stage, &map1.lines, 0.6, 0.6)?;
        let mut under = vec![map1.anchor];
        under.extend(&map1.heat);
        fade_in_all(stage, &under, 0.3)?;
        fade_in_lagged(stage, &map1.dots, 0.7, 0.5)?;
        // not `Lattice::search`: that sounds the search mark, and this
        // search sounds nothing and puts nothing out
        let sweep = Strip::build(stage, map1.cell_rect(blind.row, blind.col), BAR_SHARE * map1.cell_w)?;
        for m in sweep.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.play(tween_linear(sweep.phase, 1.0, CROSS_S)?)?;
        fade_out_all(stage, &[sweep.wash, sweep.bar], 0.3)?;
        fade_out_all(stage, &map1.mobs(), 0.6)?;

        let line = say(&kit, stage, LINES[0])?;
        let words = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("14_1 has no sentences".into()))?;
        let mut lines: Vec<Mob> = Vec::new();
        let wrapped = wrap(&words, 46, 3);
        let n = wrapped.len() as f64;
        for (k, l) in wrapped.iter().enumerate() {
            lines.push(kit.text(
                stage,
                l,
                48.0,
                Palette::ink(),
                v(0.0, 0.7 * (n - 1.0) / 2.0 - 0.7 * k as f64),
            )?);
        }
        fade_in_all(stage, &lines, 0.7)?;
        line.wait_end(stage)?;
        kit.hold(stage)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_out_all(stage, &lines, 0.5)?;

        // ------------------------------------------------------- 14_2
        // four searches, each the last frame of its story, built left to
        // right as they are named
        let line = say(&kit, stage, LINES[1])?;
        let share = {
            let nm: f64 = fact("F-A.found.nm")?
                .parse()
                .map_err(|_| fail("F-A.found.nm is not a number".into()))?;
            let radius: f64 = fact("F-A.circle.nm")?
                .parse()
                .map_err(|_| fail("F-A.circle.nm is not a number".into()))?;
            nm / radius
        };
        let mut found_on: Vec<String> = Vec::new();
        for key in FOUND_KEYS {
            let iso = fact(key)?;
            found_on.push(long_date(&iso).ok_or_else(|| fail(format!("{key} {iso} is not a date")))?);
        }
        let pictures: [Vec<Mob>; 4] = [
            palomares(stage, Pane::nth(0)),
            scorpion(&kit, stage, Pane::nth(1))?,
            air_france(stage, Pane::nth(2), share),
            aldridge(stage, Pane::nth(3), false),
        ];
        let mut panels: Vec<Vec<Mob>> = Vec::new();
        for (i, picture) in pictures.into_iter().enumerate() {
            panels.push(panel_mobs(&kit, stage, i, picture, &found_on[i])?);
        }
        // the four empty cards stand from "Four searches" (the stage was blank
        // for the three seconds before the first was named), each filled as
        // its name is said
        let backs: Vec<Mob> = panels.iter().map(|p| p[0]).collect();
        fade_in_all(stage, &backs, 0.6)?;
        let phrases = ["Palomares", "Scorpion", "Air France", "John Aldridge"];
        for (phrase, mobs) in phrases.iter().zip(&panels) {
            line.wait_for(stage, phrase)?;
            fade_in_all(stage, &mobs[1..], 0.7)?;
        }
        line.wait_end(stage)?;

        // ------------------------------------------------------- 14_3
        // the three flaws light in red inside their pictures, and the
        // question: what does the method ask of you?
        let line = say(&kit, stage, LINES[2])?;
        let mut flaws: Vec<Mob> = Vec::new();
        // the strip believed too wide: 800 feet across a cell of a sea mile
        line.wait_sentence(stage, 0)?;
        {
            let pane = Pane::nth(1);
            let at_sea = fact("F-S.sweep.at_sea")?;
            let feet: f64 = at_sea
                .split(',')
                .next()
                .and_then(|s| s.trim().parse().ok())
                .ok_or_else(|| fail(format!("F-S.sweep.at_sea {at_sea}")))?;
            let per_mile: f64 = fact("F-H.feet_per_sea_mile")?
                .parse()
                .map_err(|_| fail("F-H.feet_per_sea_mile is not a number".into()))?;
            let (w, h) = BLOCK_CELL;
            let f6 = f6_cell()?;
            let at = block_cell(pane, f6.0, f6.1).ok_or_else(|| fail("F6 is not in the block".into()))?;
            let band_h = (strip_share(feet, per_mile) * h).max(0.05);
            let band = Rectangle::new()
                .width(BLOCK_COLS.len() as f64 * w)
                .height(band_h)
                .style(flat(wrong(), 0.85))
                .build()?
                .moved_to(v(pane.at[0], at[1]));
            let band = stage.arena_mut().add(band);
            let label = kit.text(
                stage,
                &format!("{BELIEVED} {} ft", feet as u32),
                20.0,
                wrong(),
                pane.p(0.0, -1.1),
            )?;
            fade_in_all(stage, &[band, label], 0.5)?;
            flaws.extend([band, label]);
        }
        // the silence: the listening tracks across the first map's disc
        line.wait_sentence(stage, 1)?;
        {
            let pane = Pane::nth(2);
            let c = pane.p(-DISC_X, 0.1);
            let mut tracks: Vec<Mob> = Vec::new();
            for k in 0..TRACKS {
                let dy = DISC_R * 0.5 * (k as f64 - (TRACKS as f64 - 1.0) / 2.0);
                let half = (DISC_R * DISC_R - dy * dy).sqrt();
                tracks.push(
                    stage.arena_mut().add(
                        polyline(vec![v(c[0] - half, c[1] + dy), v(c[0] + half, c[1] + dy)])
                            .style(line_style(wrong(), 3.0))
                            .build(),
                    ),
                );
            }
            tracks.push(kit.text(stage, SILENCE_LABEL, 20.0, wrong(), pane.p(-DISC_X, -0.85))?);
            fade_in_all(stage, &tracks, 0.5)?;
            flaws.extend(tracks);
        }
        // the story five hours off: the first cloud, up the coast
        line.wait_sentence(stage, 2)?;
        {
            let pane = Pane::nth(3);
            let up = small_cloud(stage, pane.p(-0.3, 0.55), SMALL_CLOUD, 0.9, 0.5, 2014, wrong());
            let hours = fact("F-C.aldridge.hours_off")?;
            let off = kit.text_left(stage, &hours_off_words(&hours), 20.0, wrong(), pane.p(0.3, 0.45))?;
            let mut lit = up;
            lit.push(off);
            fade_in_all(stage, &lit, 0.5)?;
            flaws.extend(lit);
        }
        // the question to carry, held two seconds
        line.wait_sentence(stage, 3)?;
        let question = carry_last(&kit, stage, &line)?;
        fade_in_all(stage, &[question], 0.4)?;
        carry_pause(&kit, stage)?;
        fade_out_all(stage, &[question], 0.3)?;

        // ------------------------------------------------------- 14_4
        // the panels fold away; the cold open's shot again, each part named
        // as its clause is said; the update and the rule in the hold
        let line = say(&kit, stage, LINES[3])?;
        let mut folding: Vec<Mob> = panels.iter().flatten().copied().collect();
        folding.extend(&flaws);
        fade_out_all(stage, &folding, 0.6)?;
        let probe = cold_open_probe()?;
        let map2 = Lattice::build(stage, BELIEF_AT, BELIEF_ROW_H, &[probe], Heat::LATTICE)?;
        fade_in_lagged(stage, &map2.lines, 0.7, 0.6)?;
        let mut under = vec![map2.anchor];
        under.extend(&map2.heat);
        fade_in_all(stage, &under, 0.3)?;
        fade_in_lagged(stage, &map2.dots, 0.8, 0.5)?;
        let mw = map2.size().0;
        // the cloud is what you believe
        let believe = kit.text(
            stage,
            BELIEF,
            26.0,
            Palette::ink(),
            v(BELIEF_AT[0] - mw / 2.0 - 1.45, BELIEF_AT[1]),
        )?;
        line.wait_for(stage, "The first map")?;
        fade_in_all(stage, &[believe], 0.5)?;
        // the strip is how well you looked: named beside its cell, which it
        // then crosses, and the dots go out and the map deepens
        line.wait_for(stage, "how well you really looked")?;
        let (x0, y0, x1, y1) = map2.cell_rect(probe.row, probe.col);
        let cell_mid = v((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let looked = kit.text(
            stage,
            LOOKED,
            26.0,
            find(),
            v(BELIEF_AT[0] + mw / 2.0 + 1.55, cell_mid[1]),
        )?;
        let leader = stage.arena_mut().add(
            polyline(vec![
                v(BELIEF_AT[0] + mw / 2.0 + 0.05, cell_mid[1]),
                v(x1 + 0.03, cell_mid[1]),
            ])
            .style(line_style(Palette::dim(), 1.5))
            .build(),
        );
        fade_in_all(stage, &[looked, leader], 0.4)?;
        map2.search(&kit, stage, 0, CROSS_S, MOVE_S)?;
        line.wait_end(stage)?;
        // in the hold: the update, and the rule in gold
        kit.hold(stage)?;
        let update = kit.text(
            stage,
            &format!("{UPDATE} {UPDATE_RULE}"),
            36.0,
            Palette::ink(),
            v(0.0, HOLD_LINES_Y[0]),
        )?;
        let rule = kit.text(stage, RULE, 34.0, find(), v(0.0, HOLD_LINES_Y[1]))?;
        let edge = kit.text(stage, NEVER_ADDS, 22.0, Palette::dim(), v(0.0, HOLD_LINES_Y[2]))?;
        fade_in_all(stage, &[update, rule, edge], 0.7)?;
        kit.music_breath(stage, HOLD_S)?;

        // ------------------------------------------------------- 14_5
        // the stage clears by the ordinary fade before the sentence; then
        // chapter 02's card returns, and nothing sounds
        fade_all(stage, 0.8)?;
        kit.mark(stage, "silence:the_ninety_nine");
        let line = say(&kit, stage, LINES[4])?;
        let card99 = ninety_nine(&kit, stage, &fact("F-S.crew")?)?;
        fade_in_all(stage, &card99, 0.6)?;
        line.wait_end(stage)?;
        // two and a half seconds of nothing: no music breath, no cue
        pause(stage, SILENCE_S)?;
        kit.mark(stage, "silence_end:the_ninety_nine");

        // ------------------------------------------------------- 14_6
        // the ring and its label stay; beside them, chapter 11's last frame:
        // the cloud, the track, the green point; the motif resolves, once
        let line = say(&kit, stage, LINES[5])?;
        kit.mark(stage, "resolve:method");
        let pane = Pane { at: v(4.2, 0.5) };
        let picture = aldridge(stage, pane, true);
        let mut frame = vec![card(stage, pane)?];
        frame.extend(picture);
        fade_in_all(stage, &frame, 0.8)?;
        line.wait_end(stage)?;
        kit.hold(stage)?;
        kit.music_breath(stage, HOLD_S)?;

        // ------------------------------------------------------- 14_7
        // the receipts: where every number came from, who spoke, what to
        // search for; the motif's last chord is the listen window
        let mut leaving: Vec<Mob> = card99;
        leaving.extend(&frame);
        fade_out_all(stage, &leaving, 0.6)?;
        let receipts = receipts_card(&kit, stage, &[RECEIPTS_CLAIM, REPOSITORY, SYNTHETIC])?;
        let paper = fact("F-N.paper")?;
        let richardson = format!("Richardson and Stone, {paper}");
        let names = names_to_search(
            &kit,
            stage,
            &[
                "Bayesian search theory",
                "Koopman, “Search and Screening”",
                &richardson,
                "Stone, “Theory of Optimal Search”",
                "SAROPS",
            ],
        )?;
        fade_in_all(stage, &[receipts, names], 0.8)?;
        kit.listen(stage, 4.0, "14_four_maps:14_7")?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::{LAT_COLS, LAT_ROWS, counts, heat_state};
    use crate::model::sea;
    use explainer_kit::Script;

    fn text_of(id: &str) -> String {
        Script::parse(crate::SCRIPT).unwrap().text(id).unwrap().to_owned()
    }

    /// The chapter speaks exactly the script's 14_ lines, in order (14_7,
    /// the receipts, has no words).
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("14_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// Every phrase the picture waits for is in its line, in the order the
    /// chapter waits for them, and the line has the sentences the chapter
    /// lights flaws on.
    #[test]
    fn the_phrases_and_sentences_the_picture_keys_to_are_the_narrations() {
        let two = text_of("14_2");
        let at: Vec<usize> = ["Palomares", "Scorpion", "Air France", "John Aldridge"]
            .iter()
            .map(|p| two.find(p).unwrap_or_else(|| panic!("14_2 lacks {p}")))
            .collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]), "named in the order they stand");
        let three = text_of("14_3");
        assert_eq!(
            three.matches(". ").count() + three.matches("? ").count(),
            3,
            "four sentences"
        );
        assert!(three.ends_with("So what does the method ask of you?"));
        let four = text_of("14_4");
        assert!(four.find("The first map").unwrap() < four.find("how well you really looked").unwrap());
        assert!(text_of("14_5").contains("ninety-nine"));
    }

    /// The day each story ended is built from the facts as the panels print
    /// it, and a month that is not one is refused.
    #[test]
    fn the_panels_dates_are_the_facts() {
        let f = facts().unwrap();
        let said: Vec<String> = FOUND_KEYS
            .iter()
            .map(|k| long_date(f.get(k).unwrap()).unwrap())
            .collect();
        assert_eq!(
            said,
            ["15 March 1966", "28 October 1968", "2 April 2011", "24 July 2013"]
        );
        assert_eq!(long_date("1968-13-01"), None, "no thirteenth month");
        assert_eq!(long_date("1968-00-01"), None, "no month nought");
        assert_eq!(long_date("nonsense"), None);
    }

    /// 14_1: a search that sees nothing puts out no dot and changes no cell
    /// (counts and blue both), and the same search seeing two in three
    /// puts out 833 of E5's 1,250 and deepens other cells: the contrast the
    /// line is about.
    #[test]
    fn a_blind_search_changes_nothing_and_a_seeing_one_changes_the_map() {
        let blind = blind_probe().unwrap();
        assert_eq!(y1968::name(blind.row, blind.col), "E5");
        assert_eq!(blind.seeing, Q::ZERO);
        let c = counts(&y1968::POINTS, &[blind]);
        assert_eq!(c.out, vec![0]);
        assert_eq!(c.lit[0], c.lit[1], "no cell has a different number of dots");
        assert_eq!(
            heat_state(&c.lit[0], Heat::LATTICE),
            heat_state(&c.lit[1], Heat::LATTICE),
            "no cell has a different blue"
        );
        let seeing = cold_open_probe().unwrap();
        assert_eq!((seeing.row, seeing.col), (blind.row, blind.col));
        let s = counts(&y1968::POINTS, &[seeing]);
        assert_eq!(s.out, vec![833]);
        let (before, after) = (
            heat_state(&s.lit[0], Heat::LATTICE),
            heat_state(&s.lit[1], Heat::LATTICE),
        );
        let i = blind.row * LAT_COLS + blind.col;
        assert!(after[i] < before[i], "the searched cell cools");
        assert!(
            before.iter().zip(&after).enumerate().any(|(j, (b, a))| j != i && a > b),
            "and another cell deepens"
        );
    }

    /// 14_4's edge: the update multiplies and never adds. A place with no
    /// dots never gains one, on the made-up sea (searched over and over, in
    /// every cell that has dots) and on the 1968 map (every cell searched in
    /// turn): the cells with no dots stay with none and with no blue.
    #[test]
    fn a_place_with_no_dots_never_gains_one() {
        let f = facts().unwrap();
        assert_eq!(f.get("F-U.zero_stays_zero"), Some("true"));
        let mut p = [Q::lowest(1, 2), Q::lowest(1, 2), Q::ZERO, Q::ZERO];
        for step in 0..30 {
            p = sea::bayes(&p, step % 2, shown::MAP_SEEING);
            assert_eq!((p[2], p[3]), (Q::ZERO, Q::ZERO), "step {step}");
        }
        // the printed map: search every cell that holds dots, once each
        let empty: Vec<usize> = y1968::cells()
            .filter(|&(_, _, n)| n == 0)
            .map(|(r, c, _)| r * LAT_COLS + c)
            .collect();
        assert!(!empty.is_empty());
        let probes: Vec<Probe> = y1968::cells()
            .filter(|&(_, _, n)| n > 0)
            .map(|(row, col, _)| Probe {
                row,
                col,
                seeing: shown::MAP_SEEING,
            })
            .collect();
        let c = counts(&y1968::POINTS, &probes);
        let last = c.lit.last().unwrap();
        let heat = heat_state(last, Heat::LATTICE);
        for &i in &empty {
            assert_eq!(last[i], 0, "cell {i} gained a dot");
            assert_eq!(heat[i], 0.0, "cell {i} gained a blue");
        }
    }

    /// 14_2's panels: the strip the flaw draws is 800 feet of a sea mile
    /// (the fact's), the ring of Air France's panel is the found place's
    /// share of the circle, north-northeast, and the Scorpion panel's ring
    /// lies in F6 against the corner it shares with E5.
    #[test]
    fn the_pictures_carry_the_facts_numbers() {
        let f = facts().unwrap();
        let feet: f64 = f
            .get("F-S.sweep.at_sea")
            .unwrap()
            .split(',')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let mile: f64 = f.get("F-H.feet_per_sea_mile").unwrap().parse().unwrap();
        let share = strip_share(feet, mile);
        assert!((share - 800.0 / 6076.0).abs() < 1e-12);
        assert_eq!(hours_off_words(f.get("F-C.aldridge.hours_off").unwrap()), "5 hours off");

        let nm: f64 = f.get("F-A.found.nm").unwrap().parse().unwrap();
        let radius: f64 = f.get("F-A.circle.nm").unwrap().parse().unwrap();
        let (dx, dy) = ring_offset(nm / radius);
        assert!((dx.hypot(dy) / DISC_R - 0.1625).abs() < 1e-12, "6.5 of 40");
        assert!(dx > 0.0 && dy > dx, "north, a little east");
        assert!((dx.atan2(dy).to_degrees() - NNE_DEG).abs() < 1e-9);
        // the ring lies inside the disc
        assert!(dx.hypot(dy) < DISC_R);

        let (e5, f6) = (e5_cell().unwrap(), f6_cell().unwrap());
        assert_eq!(
            (f6.0, f6.1),
            (e5.0 + 1, e5.1 + 1),
            "F6 is E5's neighbour to the south-east"
        );
        let pane = Pane::nth(1);
        let (a, b) = (
            block_cell(pane, e5.0, e5.1).unwrap(),
            block_cell(pane, f6.0, f6.1).unwrap(),
        );
        let (w, h) = BLOCK_CELL;
        assert!(((b[0] - a[0]) - w).abs() < 1e-12 && ((a[1] - b[1]) - h).abs() < 1e-12);
        assert_eq!(y1968::points("E5"), Some(1250));
        assert_eq!(y1968::points("F6"), Some(85));
        assert_eq!(f.get("F-R.E5"), Some("1250"));
        assert_eq!(f.get("F-R.F6"), Some("85"));
        assert_eq!(thousands(1250u32), "1,250");
        // every cell of the block is on the printed map
        for &r in &BLOCK_ROWS {
            for &c in &BLOCK_COLS {
                assert!(block_cell(pane, r, c).is_some());
            }
        }
        assert!(block_cell(pane, 0, 0).is_none());
    }

    /// The four panels stand clear of each other, inside the stage, with
    /// their name, day and label under them above the question's line.
    #[test]
    fn the_four_panels_fit_the_stage_and_each_other() {
        let panes: Vec<Pane> = (0..PANEL_X.len()).map(Pane::nth).collect();
        for w in panes.windows(2) {
            assert!(w[1].at[0] - w[0].at[0] > PANEL_W, "panels overlap");
        }
        let (left, right) = (panes[0].at[0] - PANEL_W / 2.0, panes[3].at[0] + PANEL_W / 2.0);
        assert!(left > -FRAME_X_RADIUS && right < FRAME_X_RADIUS, "{left} {right}");
        let pane = panes[0];
        let (top, bottom) = (pane.at[1] + PANEL_H / 2.0, pane.at[1] + pane.label_y() - 0.2);
        assert!(top < FRAME_Y_RADIUS - 0.2);
        assert!(
            bottom > QUIZ_LINES_Y[0] + 0.3,
            "the labels clear the question's line: {bottom}"
        );
        assert!(
            pane.title_y() > pane.date_y() && pane.date_y() > pane.label_y(),
            "name, day, label: top to bottom"
        );
        // the discs sit side by side inside their panel, a tenth from its
        // edges and a tenth apart (at 0.07 and 0.08 they looked pressed)
        assert!(pane.p(DISC_X + DISC_R, 0.0)[0] < pane.at[0] + PANEL_W / 2.0 - 0.1);
        const {
            assert!(2.0 * DISC_X - 2.0 * DISC_R >= 0.1);
        }
        // the block fits its panel
        let (w, h) = BLOCK_CELL;
        assert!(BLOCK_COLS.len() as f64 * w < PANEL_W - 0.2);
        assert!(0.15 + BLOCK_ROWS.len() as f64 * h / 2.0 < PANEL_H / 2.0 - 0.1);
    }

    /// A map of sixteen rows `row_h` tall, centred at `BELIEF_AT`: its
    /// lowest and highest edges.
    fn map_edges(row_h: f64) -> (f64, f64) {
        let half = LAT_ROWS as f64 * row_h / 2.0;
        (BELIEF_AT[1] - half, BELIEF_AT[1] + half)
    }

    /// 14_4's map leaves room under it for the update, the rule and the
    /// small print, in that order and inside the stage; and 14_5's card is
    /// the 99, the words under the ring, in a silence of two and a half
    /// seconds.
    #[test]
    fn the_hold_has_room_and_the_card_and_silence_are_the_plans() {
        let (bottom, top) = map_edges(BELIEF_ROW_H);
        assert!(
            bottom > HOLD_LINES_Y[0] + 0.4,
            "the update line needs the room under the map: {bottom}"
        );
        assert!(top < FRAME_Y_RADIUS - 0.4);
        assert!(
            HOLD_LINES_Y.windows(2).all(|w| w[0] - w[1] >= 0.55),
            "three lines, not touching"
        );
        assert!(
            HOLD_LINES_Y.iter().all(|&y| y > -FRAME_Y_RADIUS + 0.3),
            "every line is inside the stage"
        );
        let (cold_bottom, cold_top) = map_edges(COLD_ROW_H);
        assert!(
            cold_top - cold_bottom > top - bottom,
            "14_4's map is the cold open's, smaller"
        );
        assert_eq!(SILENCE_S, 2.5, "storyboard: two and a half seconds");
        let f = facts().unwrap();
        assert_eq!(f.get("F-S.crew"), Some("99"));
        assert!(
            card_words_at()[1] < CARD_RING_AT[1] - CARD_RING_R,
            "the words are under the ring"
        );
    }

    /// The card is chapter 02's: the same ring, the same place, the number
    /// the same distance under it (read from that chapter's own source).
    #[test]
    fn the_card_is_chapter_02s() {
        let src = include_str!("ninety_nine.rs");
        let want = [
            format!(
                "const RING_AT: [f64; 2] = [{:.1}, {:.1}];",
                CARD_RING_AT[0], CARD_RING_AT[1]
            ),
            format!("const RING_R: f64 = {:.1};", CARD_RING_R),
            format!("const NUMBER_DROP: f64 = {:.1};", CARD_RING_AT[1] - CARD_TEXT_Y),
        ];
        for w in &want {
            assert!(src.contains(w.as_str()), "ninety_nine.rs no longer has: {w}");
        }
    }

    /// The names to search for carry the paper's year from the facts.
    #[test]
    fn the_receipts_name_the_paper_by_its_fact() {
        let f = facts().unwrap();
        assert_eq!(f.get("F-N.paper"), Some("1971"));
        assert_eq!(NAMES.len(), PANEL_X.len());
        assert_eq!(WHAT_PUT_IT_RIGHT.len(), PANEL_X.len());
    }
}
