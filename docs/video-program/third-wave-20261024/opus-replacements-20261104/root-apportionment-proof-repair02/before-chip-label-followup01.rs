//! 12 Both promises at once: Balinski and Young's quota method, exactly as
//! their 1974 paper defines it. Jefferson's list is read from the top, one
//! seat at a time, but a town whose next seat would lift it above its share
//! rounded up is skipped for now. At fifteen seats Cedar's eleventh signpost
//! is skipped and Birch's fourth is taken (1, 4, 10); at sixteen Cedar's
//! ceiling rises and it gets the seat (1, 4, 11). Then the film's own
//! argument that the method keeps the quota promise: each town's seats as
//! tiles along the house sizes (12_5), a marker whose passed and entered
//! tiles are the shares rounded down and up (12_6; a tile is entered once
//! the marker is strictly past its start), the method as "the entered tile
//! that ends soonest" walked to sixteen seats (12_7), and the counting
//! argument on a stretch of house sizes (12_8, 12_9). Every count from
//! `model::quota_rule` and `model::tile_end`.

use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

use super::draw::{
    CARD_H, CARD_W, LIST_X, NAME_SIZE, NAME_X, ROW_H, ROW_Y, SCORE_LABEL_X, SCORE_STEP, SCORE_X, card_y, check, cutoff,
    cutoff_y, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flat, grid, line_style, morph_all, move_all,
    pause, quota_box, run_tracker, say, score_head, score_row, seat_x, signpost_cards, stroke, think, token_at,
    town_color, wipe,
};
use crate::model::{self, ASH, BIRCH, CEDAR, KEYS, NAMES, POP, Ratio, Rule, grouped, share};

pub struct BothPromises {
    pub kit: Kit,
}

// ------------------------------------------------------------------ tiles

/// 12_5 to 12_9: house sizes from 0 to `TILE_HOUSES` along an axis across
/// the frame, and above it one row of seat tiles per town (Ash's top).
/// `TILE_HOUSES` is where both Ash's second and Cedar's thirteenth tiles end.
const TILE_HOUSES: f64 = 19.5;
const AXIS_X: (f64, f64) = (-5.2, 6.6);
const AXIS_Y: f64 = -1.45;
const TILE_Y: [f64; 3] = [1.75, 0.5, -0.75];
const TILE_H: f64 = 0.5;
/// 12_6's marker stops at this house size to count passed and entered tiles.
const MARK_AT: u64 = 8;
/// 12_7 numbers the first tiles of Jefferson's list (the 18th and 19th tie).
const RANKED: usize = 17;
/// 12_7's walk: seconds for the marker to move one house size.
const STEP: f64 = 0.3;
/// 12_9's three bounds, one per town.
const BOUND_X: [f64; 3] = [-4.3, 0.0, 4.3];
/// 12_8 and 12_9: the count panel under the axis, one row per step. Row 0
/// holds one hollow chip per tile the supposition needs, between the count
/// that names them and the verdict on them.
const PANEL_Y: [f64; 3] = [-2.35, -2.9, -3.45];
const NEED_X: f64 = -4.9;
const CHIP: f64 = 0.22;
const CHIP_X: f64 = -0.2;
const CHIP_STEP: f64 = 0.3;
const CANNOT_X: f64 = 2.35;
/// 12_8's labels inside the stretch: just right of the `then` marker, in the
/// gaps between the rows.
const MARK_GAP: f64 = 0.1;
const LATER_T_DY: f64 = 0.6;
const WAIT_T_DY: f64 = 0.63;
/// Under "suppose", over `then` and `now`: which tiles are real.
const LEGEND_Y: f64 = 2.95;
/// 12_1's think ring: left of the list, so it covers no card.
const PREDICT_AT: (f64, f64) = (1.2, -2.9);
/// For pulsing a line already drawn in the highlight's yellow.
const PULSE_WHITE: [f32; 3] = [1.0, 1.0, 1.0];

/// Where house size `h` sits along the axis.
fn hx(h: f64) -> f64 {
    AXIS_X.0 + h * (AXIS_X.1 - AXIS_X.0) / TILE_HOUSES
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Seat `k` of town `town` as a tile from house size `start` to `end`.
#[derive(Clone, Copy)]
struct Tile {
    town: usize,
    k: u64,
    start: Ratio,
    end: Ratio,
    outline: Mob,
}

/// The left and right edges on screen of a tile from `start` to `end` (one
/// running past the axis is cut at its end).
fn span(start: Ratio, end: Ratio) -> (f64, f64) {
    (hx(start.to_f64()), hx(end.to_f64().min(TILE_HOUSES)))
}

fn tile_span(t: &Tile) -> (f64, f64) {
    span(t.start, t.end)
}

/// Every tile that starts on the axis, each outlined in its town's color,
/// detached; a tile running past the axis is left open at the axis end.
fn tile_rows(stage: &mut Stage<'_>) -> fmn::Result<Vec<Tile>> {
    let mut out = Vec::new();
    for (i, &y) in TILE_Y.iter().enumerate() {
        for k in 1_u64.. {
            let start = model::tile_end(&POP, i, k - 1);
            if start.to_f64() >= TILE_HOUSES {
                break;
            }
            let end = model::tile_end(&POP, i, k);
            let (x0, x1) = span(start, end);
            let (lo, hi) = (y - TILE_H / 2.0, y + TILE_H / 2.0);
            let points = if end.to_f64() > TILE_HOUSES {
                vec![v(x1, hi), v(x0, hi), v(x0, lo), v(x1, lo)]
            } else {
                vec![v(x0, lo), v(x0, hi), v(x1, hi), v(x1, lo), v(x0, lo)]
            };
            let outline = stroke(stage, points, line_style(town_color(i), 2.5, 0.9));
            out.push(Tile {
                town: i,
                k,
                start,
                end,
                outline,
            });
        }
    }
    Ok(out)
}

/// Town `town`'s tile `k`.
fn tile(tiles: &[Tile], town: usize, k: u64) -> fmn::Result<Tile> {
    tiles
        .iter()
        .find(|t| t.town == town && t.k == k)
        .copied()
        .ok_or_else(|| fail(format!("no tile for {}'s seat {k}", NAMES[town])))
}

/// A flat block inside tile `t` (a seat handed out, or a veil), detached.
fn tile_rect(stage: &mut Stage<'_>, t: &Tile, style: Style) -> fmn::Result<Mob> {
    let (x0, x1) = tile_span(t);
    let r = Rectangle::new()
        .width(x1 - x0 - 0.06)
        .height(TILE_H - 0.08)
        .style(style)
        .build()?
        .moved_to(v((x0 + x1) / 2.0, TILE_Y[t.town]));
    Ok(stage.arena_mut().add(r))
}

/// A thin green bar under tile `t` (a tile the marker has passed), detached.
fn under_bar(stage: &mut Stage<'_>, t: &Tile) -> fmn::Result<Mob> {
    let (x0, x1) = tile_span(t);
    let r = Rectangle::new()
        .width(x1 - x0 - 0.06)
        .height(0.08)
        .style(flat(Palette::green(), 0.95))
        .build()?
        .moved_to(v((x0 + x1) / 2.0, TILE_Y[t.town] - TILE_H / 2.0 - 0.1));
    Ok(stage.arena_mut().add(r))
}

/// A bold outline just outside tile `t`, detached.
fn ring(stage: &mut Stage<'_>, t: &Tile, color: Srgb) -> Mob {
    let (x0, x1) = tile_span(t);
    let (lo, hi) = (
        TILE_Y[t.town] - TILE_H / 2.0 - 0.04,
        TILE_Y[t.town] + TILE_H / 2.0 + 0.04,
    );
    stroke(
        stage,
        vec![v(x0, lo), v(x0, hi), v(x1, hi), v(x1, lo), v(x0, lo)],
        line_style(color, 4.5, 1.0),
    )
}

/// A marker at house size `tracker` (now `h0`): a line down through the
/// rows to the axis, redrawn as the tracker moves. [line, anchor] to fade in.
fn marker(stage: &mut Stage<'_>, tracker: Mob, h0: f64, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let ends = |h: f64| vec![v(hx(h), TILE_Y[0] + 0.5), v(hx(h), AXIS_Y + 0.05)];
    let line = stroke(stage, ends(h0), line_style(color, 4.0, 1.0));
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let h = s.tracker_value(tracker).unwrap_or(0.0);
            if last == Some(h) {
                return;
            }
            let _ = s.set_points(line, polyline(ends(h)).build().points());
            last = Some(h);
        },
        true,
    )?;
    Ok([line, anchor])
}

/// Where chip `i` of the count row sits.
fn chip_x(i: usize) -> f64 {
    CHIP_X + i as f64 * CHIP_STEP
}

/// Chip `i` of the count row: one tile the supposition needs, hollow because
/// it stands for a count and not for any real seat, detached.
fn chip(stage: &mut Stage<'_>, i: usize, color: Srgb) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(CHIP)
        .height(CHIP)
        .corner_radius(0.03)
        .style(line_style(color, 2.5, 1.0))
        .build()?
        .moved_to(v(chip_x(i), PANEL_Y[0]));
    Ok(stage.arena_mut().add(r))
}

/// Recolor each mobject and back, in place: a flash with no growth, so a
/// tile's drawn length and its neighbours stay true.
fn pulse_all(stage: &mut Stage<'_>, mobs: &[Mob], color: Option<[f32; 3]>, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.0, color)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// The house-size axis: a line, a tick at every whole house size (taller
/// every five, with its number) and its name, detached.
fn house_axis(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![stroke(
        stage,
        vec![v(hx(0.0), AXIS_Y), v(hx(TILE_HOUSES), AXIS_Y)],
        line_style(Palette::dim(), 3.0, 0.9),
    )];
    for h in 0..=(TILE_HOUSES as u64) {
        let x = hx(h as f64);
        let tall = h % 5 == 0;
        let half = if tall { 0.14 } else { 0.07 };
        out.push(stroke(
            stage,
            vec![v(x, AXIS_Y - half), v(x, AXIS_Y + half)],
            line_style(Palette::dim(), 2.5, 0.9),
        ));
        if tall {
            out.push(kit.text(stage, &h.to_string(), 20.0, Palette::dim(), v(x, AXIS_Y - 0.38))?);
        }
    }
    out.push(kit.text_left(stage, "house size", 20.0, Palette::dim(), v(NAME_X, AXIS_Y))?);
    Ok(out)
}

/// How many Jefferson signposts the list shows (down to Ash's second).
const SHOWN: usize = 18;
/// The left panel: names, seat counters and each town's ceiling.
const COUNT_X: f64 = -2.2;
const CEIL_X: f64 = -0.4;

fn counters(kit: &Kit, stage: &mut Stage<'_>, c: &[u64]) -> fmn::Result<Vec<Mob>> {
    (0..3)
        .map(|i| kit.text(stage, &c[i].to_string(), 44.0, town_color(i), v(COUNT_X, ROW_Y[i])))
        .collect()
}

/// "at most 4": each town's share rounded up at `house` seats.
fn ceilings(kit: &Kit, stage: &mut Stage<'_>, house: u64, hot: Option<usize>) -> fmn::Result<Vec<Mob>> {
    (0..3)
        .map(|i| {
            let c = share(&POP, i, house).ceil();
            let color = if hot == Some(i) { Palette::red() } else { Palette::dim() };
            kit.text(stage, &format!("at most {c}"), 20.0, color, v(CEIL_X, ROW_Y[i]))
        })
        .collect()
}

/// A soft glow behind card `rank`, detached.
fn glow(stage: &mut Stage<'_>, rank: usize, color: Srgb) -> fmn::Result<Mob> {
    let g = Rectangle::new()
        .width(CARD_W + 0.14)
        .height(CARD_H + 0.1)
        .corner_radius(0.06)
        .style(flat(color, 0.3))
        .build()?
        .moved_to(v(LIST_X, card_y(rank)));
    Ok(stage.arena_mut().add(g))
}

fn triple(s: &[u64]) -> String {
    s.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
}

impl SceneConstruct for BothPromises {
    fn name(&self) -> &str {
        "12_both_promises"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let fact = |key: &str| {
            facts
                .get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };
        let list = model::signposts(Rule::Jefferson, &POP, SHOWN);
        let history = model::quota_rule(&POP, 16).map_err(fail)?;
        let jefferson15 = model::divisor(Rule::Jefferson, &POP, 15).map_err(fail)?;
        let rank_of = |town: usize, seat: u64| list.iter().position(|e| e.0 == town && e.1 == seat).map(|r| r + 1);
        let skipped = rank_of(CEDAR, 11).ok_or_else(|| fail("no card for Cedar's eleventh seat".into()))?;
        let taken = rank_of(BIRCH, 4).ok_or_else(|| fail("no card for Birch's fourth seat".into()))?;
        let ash2 = rank_of(ASH, 2).ok_or_else(|| fail("no card for Ash's second seat".into()))?;
        // the story the chapter tells, checked against the model
        if history[14] != model::divisor(Rule::Jefferson, &POP, 14).map_err(fail)?
            || skipped != 15
            || taken != 16
            || history[15] != [history[14][0], history[14][1] + 1, history[14][2]]
            || history[16] != [history[15][0], history[15][1], history[15][2] + 1]
            || share(&POP, CEDAR, 15).ceil() != u128::from(history[14][CEDAR])
        {
            return Err(fail("12: the quota method's walk differs from the one narrated".into()));
        }

        // 12_1 Jefferson's fifteenth seat lifts Cedar above its ceiling
        let s = say(&kit, stage, "12_1")?;
        let cards = signpost_cards(&kit, stage, &POP, SHOWN)?;
        let mut names = Vec::new();
        for i in 0..3 {
            names.push(kit.text_left(stage, NAMES[i], 30.0, town_color(i), v(-5.8, ROW_Y[i]))?);
        }
        let head = kit.text(stage, "Jefferson's list", 20.0, Palette::dim(), v(LIST_X, 3.78))?;
        let line = cutoff(stage, 15);
        let c_j = counters(&kit, stage, &jefferson15)?;
        let mut first: Vec<Mob> = cards.iter().flatten().copied().collect();
        first.extend(&names);
        first.push(head);
        fade_in_lagged(stage, &first, 1.4, 0.05)?;
        fade_in_all(stage, &[vec![line], c_j.clone()].concat(), 0.5)?;
        s.wait_for(stage, "by giving Cedar its eleventh seat")?;
        let hot = glow(stage, skipped, Palette::red())?;
        fade_in_all(stage, &[hot], 0.4)?;
        flash_all(stage, &[c_j[CEDAR]], 0.9)?;
        s.wait_for(stage, "Suppose a rule refuses")?;
        let ceil15 = ceilings(&kit, stage, 15, Some(CEDAR))?;
        let ceil_head = kit.text(stage, "15 seats", 18.0, Palette::dim(), v(CEIL_X, 2.35))?;
        fade_in_lagged(stage, &[vec![ceil_head], ceil15.clone()].concat(), 1.0, 0.25)?;
        let q = think(&kit, stage, v(PREDICT_AT.0, PREDICT_AT.1))?;

        // 12_2 Birch; the rule in its simple form
        let s = say(&kit, stage, "12_2")?;
        fade_out_all(stage, &[q], 0.2)?;
        let take = glow(stage, taken, Palette::hero())?;
        fade_in_all(stage, &[take], 0.4)?;
        flash_all(stage, &cards[taken - 1][1..], 0.9)?;
        s.wait_for(stage, "Michel Balinski and Peyton Young")?;
        let credit = kit.text(
            stage,
            &format!(
                "the quota method: Balinski and Young, {}",
                fact("history.quota_method_year")?
            ),
            22.0,
            Palette::ink(),
            v(-3.4, 3.3),
        )?;
        fade_in_all(stage, &[credit], 0.5)?;
        // the walk to fourteen is the quota method's own: at house h the card
        // it takes (`rank`) and Jefferson's next untaken card (`next`); where
        // they differ, `next` would lift its town above its ceiling and waits
        let step_cards = |h: usize| -> fmn::Result<(usize, usize)> {
            let town = (0..3)
                .find(|&i| history[h][i] == history[h - 1][i] + 1)
                .ok_or_else(|| fail(format!("12_2: no seat handed out at {h}")))?;
            let rank = rank_of(town, history[h][town]).ok_or_else(|| fail(format!("12_2: no card for seat {h}")))?;
            let next = list
                .iter()
                .position(|e| e.1 > history[h - 1][e.0])
                .map(|r| r + 1)
                .ok_or_else(|| fail(format!("12_2: the list runs out at {h}")))?;
            Ok((rank, next))
        };
        let walk: Vec<(usize, usize)> = (1..=14).map(step_cards).collect::<fmn::Result<_>>()?;
        let waits: Vec<usize> = (1..=14).filter(|&h| walk[h - 1].0 != walk[h - 1].1).collect();
        // the narration shows one early skip: the waiting card is taken at
        // the very next house size
        let wait_h = match waits[..] {
            [h] if h < 14 && walk[h].0 == walk[h - 1].1 => h,
            _ => {
                return Err(fail(format!(
                    "12_2 shows exactly one skip before fifteen seats, resolved one seat later; got {waits:?}"
                )));
            }
        };
        let house_head = |h: usize| {
            if h == 1 {
                "1 seat".to_string()
            } else {
                format!("{h} seats")
            }
        };
        // each step morphs the ceilings to house h and the counters to the
        // quota method's count, and puts the line below the deepest card read
        let mut depth: usize = 0;
        let mut ceil_now: Vec<Mob> = ceil15.clone();
        ceil_now.push(ceil_head);
        s.wait_for(stage, "Hand out seats one at a time")?;
        fade_out_all(stage, &[hot, take], 0.3)?;
        let zero = counters(&kit, stage, &[0, 0, 0])?;
        let mut pairs: Vec<(Mob, Mob)> = c_j.iter().copied().zip(zero.iter().copied()).collect();
        let mut c1 = ceilings(&kit, stage, 1, None)?;
        c1.push(kit.text(stage, &house_head(1), 18.0, Palette::dim(), v(CEIL_X, 2.35))?);
        pairs.extend(ceil_now.iter().copied().zip(c1.iter().copied()));
        morph_all(stage, &pairs, 0.4)?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(0)))], 0.5)?;
        for h in 1..wait_h {
            depth = depth.max(walk[h - 1].0);
            let next = counters(&kit, stage, &history[h])?;
            let mut pairs: Vec<(Mob, Mob)> = c_j.iter().copied().zip(next.iter().copied()).collect();
            let mut ch = ceilings(&kit, stage, h as u64, None)?;
            ch.push(kit.text(stage, &house_head(h), 18.0, Palette::dim(), v(CEIL_X, 2.35))?);
            pairs.extend(ceil_now.iter().copied().zip(ch.iter().copied()));
            move_all(stage, &[(line, v(LIST_X, cutoff_y(depth)))], 0.3)?;
            morph_all(stage, &pairs, 0.15)?;
        }
        // the new house size: its ceilings, the town the next card would lift
        let (w_rank, w_next) = walk[wait_h - 1];
        let w_town = list[w_next - 1].0;
        s.wait_for(stage, "check that the seat would not lift it")?;
        let mut cw = ceilings(&kit, stage, wait_h as u64, Some(w_town))?;
        cw.push(kit.text(stage, &house_head(wait_h), 18.0, Palette::dim(), v(CEIL_X, 2.35))?);
        morph_all(
            stage,
            &ceil_now.iter().copied().zip(cw.iter().copied()).collect::<Vec<_>>(),
            0.5,
        )?;
        flash_all(stage, &ceil_now[..3], 1.0)?;
        if share(&POP, w_town, wait_h as u64).ceil() != u128::from(history[wait_h - 1][w_town]) {
            return Err(fail(format!(
                "12_2: at {wait_h} seats card {w_next} must lift {} above its ceiling",
                NAMES[w_town]
            )));
        }
        s.wait_for(stage, "If it would, skip that signpost for now")?;
        let waiting = glow(stage, w_next, Palette::red())?;
        fade_in_all(stage, &[waiting], 0.4)?;
        flash_all(stage, &[c_j[w_town], ceil_now[w_town]], 0.9)?;
        s.wait_for(stage, "and go to the next one")?;
        depth = depth.max(w_rank);
        let next = counters(&kit, stage, &history[wait_h])?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(depth)))], 0.4)?;
        morph_all(
            stage,
            &c_j.iter().copied().zip(next.iter().copied()).collect::<Vec<_>>(),
            0.3,
        )?;
        // on to fourteen; the waiting card is taken at the next house size
        for h in wait_h + 1..=14 {
            depth = depth.max(walk[h - 1].0);
            if h == wait_h + 1 {
                fade_out_all(stage, &[waiting], 0.2)?;
            }
            let next = counters(&kit, stage, &history[h])?;
            let mut pairs: Vec<(Mob, Mob)> = c_j.iter().copied().zip(next.iter().copied()).collect();
            let mut ch = ceilings(&kit, stage, h as u64, None)?;
            ch.push(kit.text(stage, &house_head(h), 18.0, Palette::dim(), v(CEIL_X, 2.35))?);
            pairs.extend(ceil_now.iter().copied().zip(ch.iter().copied()));
            move_all(stage, &[(line, v(LIST_X, cutoff_y(depth)))], 0.2)?;
            morph_all(stage, &pairs, 0.1)?;
        }
        if depth != 14 {
            return Err(fail("12_2: the walk to fourteen seats must read fourteen cards".into()));
        }
        // back to the fifteen-seat ceilings 12_1 asked about
        let mut c15 = ceilings(&kit, stage, 15, Some(CEDAR))?;
        c15.push(kit.text(stage, &house_head(15), 18.0, Palette::dim(), v(CEIL_X, 2.35))?);
        morph_all(
            stage,
            &ceil_now.iter().copied().zip(c15.iter().copied()).collect::<Vec<_>>(),
            0.5,
        )?;

        // 12_3 fifteen seats: Cedar skipped, Birch's fourth; sixteen: Cedar
        let s = say(&kit, stage, "12_3")?;
        s.wait_for(stage, "Cedar already has ten")?;
        flash_all(stage, &[c_j[CEDAR], ceil15[CEDAR]], 1.0)?;
        s.wait_for(stage, "so it is skipped")?;
        let strike = stroke(
            stage,
            vec![
                v(LIST_X - CARD_W / 2.0, card_y(skipped)),
                v(LIST_X + CARD_W / 2.0, card_y(skipped)),
            ],
            line_style(Palette::red(), 4.0, 1.0),
        );
        fade_in_all(stage, &[strike], 0.4)?;
        s.wait_for(stage, "The next signpost is Birch's fourth")?;
        let take = glow(stage, taken, Palette::hero())?;
        fade_in_all(stage, &[take], 0.4)?;
        let c15 = counters(&kit, stage, &history[15])?;
        // Only Birch changes; morphing the other digits distorts equal values.
        morph_all(stage, &[(c_j[BIRCH], c15[BIRCH])], 0.5)?;
        s.wait_for(stage, "which beats Ash's second")?;
        let ash_glow = glow(stage, ash2, Palette::dim())?;
        fade_in_all(stage, &[ash_glow], 0.3)?;
        flash_all(stage, &cards[ash2 - 1][1..], 0.8)?;
        s.wait_for(stage, "The result is one, four, ten")?;
        let r15 = kit.text(
            stage,
            &format!("15 seats: {}", triple(&history[15])),
            26.0,
            Palette::hero(),
            v(-3.6, -2.75),
        )?;
        fade_in_all(stage, &[r15], 0.4)?;
        s.wait_for(stage, "At sixteen seats Cedar's ceiling rises")?;
        let ceil16 = ceilings(&kit, stage, 16, None)?;
        let ceil_head16 = kit.text(stage, "16 seats", 18.0, Palette::dim(), v(CEIL_X, 2.35))?;
        let mut pairs: Vec<(Mob, Mob)> = ceil15.iter().copied().zip(ceil16.iter().copied()).collect();
        pairs.push((ceil_head, ceil_head16));
        morph_all(stage, &pairs, 0.6)?;
        s.wait_for(stage, "and it gets its seat")?;
        fade_out_all(stage, &[strike, ash_glow], 0.3)?;
        let take15 = glow(stage, skipped, Palette::hero())?;
        fade_in_all(stage, &[take15], 0.3)?;
        move_all(stage, &[(line, v(LIST_X, cutoff_y(16)))], 0.5)?;
        let c16 = counters(&kit, stage, &history[16])?;
        morph_all(
            stage,
            &c_j.iter().copied().zip(c16.iter().copied()).collect::<Vec<_>>(),
            0.5,
        )?;
        let r16 = kit.text(
            stage,
            &format!("16 seats: {}", triple(&history[16])),
            26.0,
            Palette::hero(),
            v(-3.6, -3.3),
        )?;
        fade_in_all(stage, &[r16], 0.4)?;
        pause(stage, 1.0)?;

        // 12_4 both promises: no take backs by construction, the quota
        // promise by Balinski and Young's theorem; the film's own argument
        // for it follows in 12_5 to 12_9
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "12_4")?;
        let y0 = 2.0;
        let head = score_head(&kit, stage, 2, y0 + 0.9)?;
        let rows = [
            score_row(&kit, stage, "Hamilton", &[(Some(true), ""), (Some(false), "")], y0)?,
            score_row(
                &kit,
                stage,
                "Jefferson",
                &[(Some(false), ""), (Some(true), "")],
                y0 - SCORE_STEP,
            )?,
            score_row(
                &kit,
                stage,
                "Webster",
                &[(Some(false), "never with 3 towns"), (Some(true), "")],
                y0 - 2.0 * SCORE_STEP,
            )?,
        ];
        let mut shown = head;
        for r in &rows {
            shown.extend(r);
        }
        fade_in_all(stage, &shown, 0.6)?;
        let quota_y = y0 - 3.2 * SCORE_STEP;
        let quota_row = score_row(&kit, stage, "quota method", &[(None, ""), (None, "")], quota_y)?;
        fade_in_all(stage, &quota_row, 0.4)?;
        s.wait_for(stage, "so nobody ever loses one")?;
        let keep = check(stage, v(SCORE_X[1], quota_y), 0.42, Palette::green());
        fade_in_all(stage, &[keep], 0.4)?;
        s.wait_for(stage, "Balinski and Young's theorem says")?;
        let quota = check(stage, v(SCORE_X[0], quota_y), 0.42, Palette::green());
        fade_in_all(stage, &[quota], 0.4)?;
        // a short, readable source; the description cites it in full
        let cited = kit.text_left(
            stage,
            &format!(
                "Balinski & Young, PNAS {}, Theorem 2",
                fact("history.quota_method_year")?
            ),
            22.0,
            Palette::dim(),
            v(SCORE_LABEL_X, quota_y - 0.8),
        )?;
        fade_in_all(stage, &[cited], 0.5)?;
        // the model's own run agrees (not a proof; 12_5 to 12_9 give one)
        if fact("quotamethod.within_quota_checked")? != "yes" || fact("quotamethod.never_takes_back_checked")? != "yes"
        {
            return Err(fail(
                "12_4: the model's run of the quota method breaks a promise".into(),
            ));
        }
        s.wait_for(stage, "Both promises, at every house size")?;
        flash_all(stage, &[keep, quota], 0.9)?;

        // 12_5 each town's seats as a row of tiles along the house sizes
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "12_5")?;
        let total = model::total(&POP);
        let g = POP.iter().fold(total, |a, &b| gcd(a, b));
        let tiles = tile_rows(stage)?;
        let outlines: Vec<Mob> = tiles.iter().map(|t| t.outline).collect();
        let mut names = Vec::new();
        for (i, &y) in TILE_Y.iter().enumerate() {
            names.push(kit.text_left(stage, NAMES[i], NAME_SIZE, town_color(i), v(NAME_X, y + 0.12))?);
        }
        let axis = house_axis(&kit, stage)?;
        s.wait_for(stage, "along the house sizes")?;
        fade_in_all(stage, &[names, axis].concat(), 0.6)?;
        s.wait_for(stage, "as a row of tiles")?;
        fade_in_lagged(stage, &outlines, 1.6, 0.04)?;
        s.wait_for(stage, "by its fraction of the people")?;
        let mut fracs = Vec::new();
        for (i, &y) in TILE_Y.iter().enumerate() {
            fracs.push(kit.text_left(
                stage,
                &format!("+{}/{} per seat", POP[i] / g, total / g),
                18.0,
                Palette::dim(),
                v(NAME_X, y - 0.28),
            )?);
        }
        fade_in_lagged(stage, &fracs, 0.9, 0.3)?;
        s.wait_for(stage, "so each tile is as long as")?;
        let firsts: Vec<Tile> = (0..3).map(|i| tile(&tiles, i, 1)).collect::<fmn::Result<_>>()?;
        // Preserve tile length during the highlight.
        pulse_all(stage, &firsts.iter().map(|t| t.outline).collect::<Vec<_>>(), None, 1.0)?;
        let mut lens = Vec::new();
        for (t, phrase) in firsts
            .iter()
            .zip(["Ash's tiles are", "Birch's four and a third", "Cedar's one and a half"])
        {
            s.wait_for(stage, phrase)?;
            let (x0, x1) = tile_span(t);
            let len = kit.text(
                stage,
                &fact(&format!("tile.len.{}", KEYS[t.town]))?,
                22.0,
                town_color(t.town),
                v((x0 + x1) / 2.0, TILE_Y[t.town] + 0.45),
            )?;
            fade_in_all(stage, &[len], 0.4)?;
            lens.push(len);
        }

        // 12_6 a marker along the house sizes: the share rounded down counts
        // the tiles it has passed, rounded up the tiles it has entered
        let s = say(&kit, stage, "12_6")?;
        let m = stage.add_value_tracker(0.0);
        s.wait_for(stage, "Now slide a marker")?;
        fade_out_all(stage, &[fracs, lens].concat(), 0.3)?;
        let mark = marker(stage, m, 0.0, Palette::hero())?;
        for (i, &y) in TILE_Y.iter().enumerate() {
            let speed = POP[i] as f64 / total as f64;
            kit.live_value(
                stage,
                move |s| s.tracker_value(m).unwrap_or(0.0) * speed,
                2,
                false,
                20.0,
                town_color(i),
                v(NAME_X, y - 0.28),
                LEFT,
            )?;
        }
        let at_t = kit.text_left(stage, "marker at", 24.0, Palette::hero(), v(NAME_X, 3.3))?;
        kit.live_count(stage, m, 30.0, Palette::hero(), v(NAME_X + 2.2, 3.3))?;
        fade_in_all(stage, &[mark[0], mark[1], at_t], 0.4)?;
        run_tracker(stage, m, MARK_AT as f64, 2.4)?;
        let at = Ratio::new(u128::from(MARK_AT), 1);
        let passed: Vec<Tile> = tiles.iter().filter(|t| t.end <= at).copied().collect();
        // entered but not passed: strictly past the start, not yet the end
        let entered: Vec<Tile> = tiles.iter().filter(|t| t.start < at && t.end > at).copied().collect();
        let not_entered: Vec<Tile> = tiles.iter().filter(|t| t.start >= at).copied().collect();
        for (i, name) in NAMES.iter().enumerate() {
            let q = share(&POP, i, MARK_AT);
            let down = passed.iter().filter(|t| t.town == i).count() as u128;
            let up = down + entered.iter().filter(|t| t.town == i).count() as u128;
            if down != q.floor() || up != q.ceil() {
                return Err(fail(format!(
                    "12_6: at {MARK_AT} seats {name}'s passed and entered tiles are not its share rounded down and up"
                )));
            }
        }
        s.wait_for(stage, "counts the tiles the marker has passed completely")?;
        let bars: Vec<Mob> = passed.iter().map(|t| under_bar(stage, t)).collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &bars, 0.8, 0.2)?;
        s.wait_for(stage, "Rounded up, it counts the tiles the marker has entered")?;
        let open: Vec<Mob> = entered.iter().map(|t| ring(stage, t, Palette::hero())).collect();
        fade_in_all(stage, &open, 0.5)?;
        s.wait_for(stage, "every tile the marker has passed is already handed out")?;
        flash_all(stage, &bars, 0.9)?;
        s.wait_for(stage, "and no tile it has not entered is")?;
        let veils: Vec<Mob> = not_entered
            .iter()
            .map(|t| tile_rect(stage, t, flat(Palette::faint(), 0.45)))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &veils, 0.8, 0.05)?;

        // 12_7 Jefferson's list is the tiles in the order they end; at each
        // house size the quota method hands out the entered tile that ends
        // soonest
        let s = say(&kit, stage, "12_7")?;
        fade_out_all(stage, &[bars, open, veils].concat(), 0.3)?;
        run_tracker(stage, m, 0.0, 0.8)?;
        s.wait_for(stage, "in the order they end")?;
        let ranked = model::signposts(Rule::Jefferson, &POP, RANKED);
        if ranked.windows(2).any(|w| w[0].2 <= w[1].2) {
            return Err(fail(
                "12_7 numbers the first tiles of Jefferson's list, which must not tie".into(),
            ));
        }
        let mut ranks = Vec::new();
        for (r, &(i, k, _)) in ranked.iter().enumerate() {
            let t = tile(&tiles, i, k)?;
            ranks.push(kit.text(
                stage,
                &(r + 1).to_string(),
                16.0,
                Palette::ink(),
                v(tile_span(&t).1 - 0.16, TILE_Y[i] + TILE_H / 2.0 + 0.16),
            )?);
        }
        fade_in_lagged(stage, &ranks, 2.4, 0.2)?;
        s.wait_for(stage, "A seat's signpost is")?;
        let rule_t = kit.text(
            stage,
            &format!("a seat's tile ends at {} ÷ its signpost", grouped(u128::from(total))),
            24.0,
            Palette::ink(),
            v(0.0, 3.3),
        )?;
        let (bt, bk, _) = list[taken - 1];
        let example = kit.text(
            stage,
            &format!(
                "{}'s seat {bk}: {} ÷ {} = {}",
                NAMES[bt],
                grouped(u128::from(total)),
                fact("quotamethod.15.birch_priority")?,
                fact("tile.end.birch.4")?
            ),
            22.0,
            Palette::dim(),
            v(0.0, 2.85),
        )?;
        fade_in_all(stage, &[rule_t], 0.4)?;
        s.wait_for(stage, "the bigger the signpost")?;
        fade_in_all(stage, &[example], 0.4)?;
        flash_all(stage, &[ranks[taken - 1]], 0.8)?;
        s.wait_for(stage, "At every house size")?;
        // the walk: at each house the entered tile, not yet held, that ends
        // soonest; where Jefferson's next tile is not yet entered, it waits
        let mut fills: Vec<(usize, Tile, Mob)> = Vec::new();
        let mut waits = Vec::new();
        for (step, pair) in history.windows(2).enumerate() {
            let (h, prev, now) = (step + 1, &pair[0], &pair[1]);
            let town = (0..POP.len())
                .find(|&i| now[i] == prev[i] + 1)
                .ok_or_else(|| fail(format!("12_7: no seat handed out at {h}")))?;
            let house = Ratio::new(h as u128, 1);
            let next: Vec<Tile> = (0..POP.len())
                .map(|i| tile(&tiles, i, prev[i] + 1))
                .collect::<fmn::Result<_>>()?;
            let soonest = next.iter().filter(|t| t.start < house).min_by(|a, b| a.end.cmp(&b.end));
            if soonest.map(|t| t.town) != Some(town) {
                return Err(fail(format!(
                    "12_7: at {h} seats the quota method does not take the entered tile that ends soonest"
                )));
            }
            run_tracker(stage, m, h as f64, STEP)?;
            let first = next.iter().min_by(|a, b| a.end.cmp(&b.end)).copied();
            if let Some(j) = first.filter(|t| t.start >= house) {
                // Jefferson's next tile has not been entered: it waits
                let (x0, x1) = tile_span(&j);
                let wait = ring(stage, &j, Palette::red());
                let wait_t = kit.text(
                    stage,
                    &format!(
                        "{} to {}: not entered at {h}",
                        model::mixed(j.start),
                        model::mixed(j.end)
                    ),
                    18.0,
                    Palette::red(),
                    v((x0 + x1) / 2.0, AXIS_Y - 0.75),
                )?;
                fade_in_all(stage, &[wait, wait_t], 0.4)?;
                pause(stage, 1.2)?;
                waits.extend([wait, wait_t]);
            }
            let t = tile(&tiles, town, now[town])?;
            let f = tile_rect(stage, &t, flat(town_color(town), 0.75))?;
            fade_in_all(stage, &[f], 0.15)?;
            fills.push((h, t, f));
        }

        // 12_8 suppose a passed tile were left out. The supposition is kept
        // apart from the run: a solid tile was really handed out by `then`, a
        // hollow one is only supposed, and no fill after `then` stays in view,
        // since no real run can show the supposition. The missing tile is
        // Cedar's fifth, the wait 12_2 and 12_7 showed: at `then` the rule
        // really handed out Birch's second, which ends later than it.
        let s = say(&kit, stage, "12_8")?;
        let (from, to) = model::STRETCH;
        let (from_r, to_r) = (Ratio::new(u128::from(from), 1), Ratio::new(u128::from(to), 1));
        let (from_h, to_h) = (from as usize, to as usize);
        let seats_n = to_h - from_h;
        let proof = model::stretch_proof(&POP, from, to).map_err(fail)?;
        let missing = tile(&tiles, proof.missing.town, proof.missing.k)?;
        let later = tile(&tiles, proof.later.town, proof.later.k)?;
        let mut inside = Vec::new();
        for t in &proof.inside {
            inside.push(tile(&tiles, t.town, t.k)?);
        }
        // 12_7's fills: those after the stretch, and those the supposition takes away
        let past_now: Vec<Mob> = fills.iter().filter(|(h, _, _)| *h > to_h).map(|f| f.2).collect();
        let real_since: Vec<(usize, u64, Mob)> = fills
            .iter()
            .filter(|(h, _, _)| (from_h + 1..=to_h).contains(h))
            .map(|(_, t, f)| (t.town, t.k, *f))
            .collect();
        if history[from_h] != proof.then_seats
            || history[to_h] != proof.now_seats
            || real_since.len() != proof.real_after.len()
            || real_since
                .iter()
                .zip(&proof.real_after)
                .any(|(f, t)| (f.0, f.1) != (t.town, t.k))
        {
            return Err(fail("12_8: the fills since then are not the real run".into()));
        }
        let since_then: Vec<Mob> = real_since.iter().map(|f| f.2).collect();
        // every tile is ruled out by one spoken clause, or is whole inside the stretch
        let entered_then: Vec<Tile> = tiles
            .iter()
            .filter(|t| t.start < from_r && !(t.town == later.town && t.k == later.k))
            .copied()
            .collect();
        let beyond_now: Vec<Tile> = tiles
            .iter()
            .filter(|t| t.start >= from_r && t.end > to_r)
            .copied()
            .collect();
        if entered_then.len() + 1 + beyond_now.len() + inside.len() != tiles.len() {
            return Err(fail(
                "12_8: the veils and the stretch do not cover every tile once".into(),
            ));
        }
        fade_out_all(stage, &[ranks, waits, vec![rule_t, example]].concat(), 0.3)?;
        s.wait_for(stage, "Could a passed tile ever be left out")?;
        // back to the end of the stretch: the real run, whole
        fade_out_all(stage, &past_now, 0.4)?;
        run_tracker(stage, m, to as f64, 1.0)?;
        let now_t = kit.text(stage, "now", 22.0, Palette::hero(), v(hx(to as f64), TILE_Y[0] + 0.75))?;
        fade_in_all(stage, &[now_t], 0.3)?;
        s.wait_for(stage, "Suppose one were")?;
        let (mx0, mx1) = tile_span(&missing);
        fade_out_all(stage, &since_then, 0.4)?;
        let gone = ring(stage, &missing, Palette::red());
        let ask = kit.text(
            stage,
            "?",
            30.0,
            Palette::red(),
            v((mx0 + mx1) / 2.0, TILE_Y[missing.town]),
        )?;
        let suppose = kit.text(
            stage,
            "suppose: a passed tile not handed out",
            22.0,
            Palette::red(),
            v(0.0, 3.3),
        )?;
        fade_in_all(stage, &[gone, ask, suppose], 0.4)?;
        s.wait_for(stage, "Go back to the last moment")?;
        let then = stage.add_value_tracker(to as f64);
        let then_mark = marker(stage, then, to as f64, Palette::ink())?;
        fade_in_all(stage, &then_mark, 0.3)?;
        run_tracker(stage, then, from as f64, 1.6)?;
        let then_t = kit.text(
            stage,
            "then",
            22.0,
            Palette::ink(),
            v(hx(from as f64), TILE_Y[0] + 0.75),
        )?;
        let legend = kit.text(
            stage,
            "solid: real up to then, hollow: only supposed",
            18.0,
            Palette::dim(),
            v(0.0, LEGEND_Y),
        )?;
        fade_in_all(stage, &[then_t, legend], 0.3)?;
        s.wait_for(stage, "a tile that ends later than the missing one")?;
        // what the rule really handed out at `then`, reaching past the missing tile's end
        let later_ring = ring(stage, &later, Palette::ink());
        let compare = stroke(
            stage,
            vec![
                v(mx1, TILE_Y[missing.town] + TILE_H / 2.0),
                v(mx1, TILE_Y[later.town] + TILE_H / 2.0 + 0.06),
            ],
            line_style(Palette::red(), 2.5, 1.0),
        );
        let later_t = kit.text_left(
            stage,
            &format!(
                "{}'s seat {} ends at {}, later than {}",
                NAMES[later.town],
                fact("stretch.later.seat")?,
                fact("stretch.later.end")?,
                fact("stretch.missing.end")?
            ),
            18.0,
            Palette::ink(),
            v(hx(from as f64) + MARK_GAP, TILE_Y[later.town] + LATER_T_DY),
        )?;
        fade_in_all(stage, &[later_ring, compare, later_t], 0.4)?;
        s.wait_for(stage, "or to the very start if it never did")?;
        let or_zero = kit.text(stage, "or 0", 20.0, Palette::dim(), v(hx(0.0), TILE_Y[0] + 0.75))?;
        fade_in_all(stage, &[or_zero], 0.3)?;
        s.wait_for(stage, "Every tile handed out since then")?;
        // one hollow chip per tile the supposition needs: a count, not a place
        let mut chips = Vec::new();
        for i in 0..seats_n {
            chips.push(chip(stage, i, Palette::ink())?);
        }
        chips.push(chip(stage, seats_n, Palette::red())?);
        let need_t = kit.text_left(
            stage,
            "handed out since then",
            22.0,
            Palette::dim(),
            v(NEED_X, PANEL_Y[0]),
        )?;
        fade_in_all(stage, &[need_t], 0.3)?;
        fade_in_lagged(stage, &chips[..seats_n], 0.9, 0.2)?;
        s.wait_for(stage, "and the missing tile too")?;
        fade_in_all(stage, &chips[seats_n..], 0.3)?;
        s.wait_for(stage, "had not been entered at that moment")?;
        // the first clause: nothing that starts before `then` can be one of them
        fade_out_all(stage, &[compare], 0.2)?;
        let wait_t = kit.text_left(
            stage,
            &format!(
                "{} to {}: not entered at {from}",
                fact("stretch.missing.start")?,
                fact("stretch.missing.end")?
            ),
            18.0,
            Palette::red(),
            v(hx(from as f64) + MARK_GAP, TILE_Y[missing.town] + WAIT_T_DY),
        )?;
        let before: Vec<Mob> = entered_then
            .iter()
            .map(|t| tile_rect(stage, t, flat(Palette::faint(), 0.45)))
            .collect::<fmn::Result<_>>()?;
        let legend3 = kit.text(
            stage,
            "solid: real up to then, hollow: only supposed, grey: ruled out",
            18.0,
            Palette::dim(),
            v(0.0, LEGEND_Y),
        )?;
        morph_all(stage, &[(legend, legend3)], 0.4)?;
        fade_in_all(stage, &[before.clone(), vec![wait_t]].concat(), 0.4)?;
        pulse_all(stage, &then_mark[..1], None, 0.5)?;
        s.wait_for(stage, "or the rule would have picked a tile ending sooner")?;
        // the missing tile ends sooner than the one the rule really picked
        pulse_all(stage, &[gone, later_ring], None, 0.6)?;
        let later_veil = tile_rect(stage, &later, flat(Palette::faint(), 0.45))?;
        fade_out_all(stage, &[later_ring, later_t], 0.2)?;
        fade_in_all(stage, &[later_veil], 0.2)?;
        s.wait_for(stage, "And every one of them ends by now")?;
        // the second clause: nothing that ends after `now` can be one of them;
        // what stays clear is what fits between the two markers
        let after: Vec<Mob> = beyond_now
            .iter()
            .map(|t| tile_rect(stage, t, flat(Palette::faint(), 0.45)))
            .collect::<fmn::Result<_>>()?;
        let band = Rectangle::new()
            .width(hx(to as f64) - hx(from as f64))
            .height(TILE_Y[0] - AXIS_Y + TILE_H / 2.0 + 0.1)
            .style(flat(Palette::hero(), 0.14))
            .build()?
            .moved_to(v(
                (hx(from as f64) + hx(to as f64)) / 2.0,
                (TILE_Y[0] + TILE_H / 2.0 + 0.1 + AXIS_Y) / 2.0,
            ));
        let band = stage.arena_mut().add(band);
        fade_in_all(stage, &[after.clone(), vec![band]].concat(), 0.4)?;
        stage.scene_mut().bring_to_front(&[gone, ask, wait_t])?;
        pulse_all(stage, &mark[..1], Some(PULSE_WHITE), 0.5)?;

        // 12_9 the stretch holds fewer whole tiles than the supposition needs
        let s = say(&kit, stage, "12_9")?;
        let mut counts = Vec::new();
        for (i, key) in KEYS.iter().enumerate() {
            let n = inside.iter().filter(|t| t.town == i).count();
            if fact(&format!("stretch.inside.{key}"))? != n.to_string() {
                return Err(fail(format!(
                    "12_9: {}'s whole tiles in the stretch differ from the facts",
                    NAMES[i]
                )));
            }
            counts.push(n);
        }
        let fits: usize = counts.iter().sum();
        let need = seats_n + 1;
        if fact("stretch.seats")? != seats_n.to_string() || fact("stretch.need")? != need.to_string() || fits > seats_n
        {
            return Err(fail(
                "12_9 says the stretch holds no more whole tiles than seats".into(),
            ));
        }
        s.wait_for(stage, "So all those tiles fit inside the stretch")?;
        // Chips count hypothetical tiles; rings show capacity in the next sentence.
        pulse_all(stage, &chips, None, 0.6)?;
        let fit: Vec<Mob> = inside
            .iter()
            .filter(|t| !(t.town == missing.town && t.k == missing.k))
            .map(|t| ring(stage, t, Palette::hero()))
            .collect();
        if fit.len() + 1 != fits {
            return Err(fail("12_9: the rings do not count the whole tiles inside".into()));
        }
        s.wait_for(stage, "one more of them than seats added")?;
        let need_count = kit.text_left(
            stage,
            &format!("{seats_n} seats from then to now: {seats_n} + 1 = {need} tiles"),
            22.0,
            Palette::red(),
            v(NEED_X, PANEL_Y[0]),
        )?;
        morph_all(stage, &[(need_t, need_count)], 0.4)?;
        pulse_all(stage, &chips[seats_n..], None, 0.6)?;
        s.wait_for(stage, "But each town's row fits at most")?;
        fade_in_lagged(stage, &fit, 1.0, 0.2)?;
        stage.scene_mut().bring_to_front(&[gone, ask, wait_t])?;
        let mut bounds = Vec::new();
        for (i, &n) in counts.iter().enumerate() {
            bounds.push(kit.text(
                stage,
                &format!("{} {n} ≤ {seats_n} × {}/{}", NAMES[i], POP[i] / g, total / g),
                22.0,
                town_color(i),
                v(BOUND_X[i], PANEL_Y[1]),
            )?);
        }
        fade_in_lagged(stage, &bounds, 1.2, 0.3)?;
        s.wait_for(stage, "and the fractions add up to one")?;
        let sum = kit.text(
            stage,
            &format!(
                "{} = {} ≤ {seats_n} × {}/{} = {seats_n}",
                counts.iter().map(usize::to_string).collect::<Vec<_>>().join(" + "),
                fact("stretch.inside.total")?,
                total / g,
                total / g
            ),
            24.0,
            Palette::hero(),
            v(0.0, PANEL_Y[2]),
        )?;
        fade_in_all(stage, &[sum], 0.5)?;
        s.wait_for(stage, "a stretch never holds more whole tiles")?;
        flash_all(stage, &[sum], 0.9)?;
        // room for one chip per seat: the last chip has nowhere to go
        let (bx0, bx1) = (chip_x(0) - CHIP / 2.0, chip_x(seats_n - 1) + CHIP / 2.0);
        let by = PANEL_Y[0] - CHIP / 2.0 - 0.06;
        let room = stroke(
            stage,
            vec![v(bx0, by + 0.05), v(bx0, by), v(bx1, by), v(bx1, by + 0.05)],
            line_style(Palette::hero(), 2.5, 1.0),
        );
        let cannot = kit.text_left(
            stage,
            &format!("{need} tiles cannot fit"),
            24.0,
            Palette::red(),
            v(CANNOT_X, PANEL_Y[0]),
        )?;
        fade_in_all(stage, &[room, cannot], 0.4)?;
        s.wait_for(stage, "no passed tile is ever left out")?;
        // the supposition fails: all it brought goes, and the real run comes
        // back whole, in one shade, its seats the facts' at this house size
        for (i, key) in KEYS.iter().enumerate() {
            if fact(&format!("stretch.now.{key}"))? != history[to_h][i].to_string() {
                return Err(fail(format!(
                    "12_9: {}'s seats at {to} differ from the facts",
                    NAMES[i]
                )));
            }
        }
        let supposed = [
            vec![
                then_mark[0],
                gone,
                ask,
                suppose,
                then_t,
                legend,
                or_zero,
                wait_t,
                later_veil,
                band,
                need_t,
                sum,
                room,
                cannot,
            ],
            before,
            after,
            chips,
            fit,
            bounds,
        ]
        .concat();
        fade_out_all(stage, &supposed, 0.4)?;
        let mut back = Vec::new();
        for t in &proof.real_after {
            let t = tile(&tiles, t.town, t.k)?;
            back.push(tile_rect(stage, &t, flat(town_color(t.town), 0.75))?);
        }
        let real_t = kit.text(
            stage,
            &format!("the real run at {to} seats: {}", triple(&history[to_h])),
            22.0,
            Palette::hero(),
            v(0.0, PANEL_Y[0]),
        )?;
        fade_in_all(stage, &[back, vec![real_t]].concat(), 0.5)?;
        s.wait_for(stage, "never hands out a tile before the marker enters it")?;
        // the other half of the promise: no tile of the real run lies under a veil
        let unentered: Vec<Mob> = tiles
            .iter()
            .filter(|t| t.start >= to_r)
            .map(|t| tile_rect(stage, t, flat(Palette::faint(), 0.45)))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &unentered, 0.8, 0.05)?;
        s.wait_for(stage, "so it keeps the quota promise at every house size")?;
        let done = kit.text(
            stage,
            "quota method: within quota at every house size",
            24.0,
            Palette::green(),
            v(0.0, 3.3),
        )?;
        fade_in_all(stage, &[done], 0.5)?;

        // 12_10 one promise, more than one answer
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "12_10")?;
        let hamilton15 = model::hamilton(&POP, 15).map_err(fail)?;
        let mut base = Vec::new();
        let mut boxes = Vec::new();
        for i in 0..3 {
            let y = ROW_Y[i];
            base.push(kit.text_left(stage, NAMES[i], NAME_SIZE, town_color(i), v(NAME_X, y))?);
            base.extend(grid(stage, y, 11)?);
            let q = share(&POP, i, 15);
            boxes.push(quota_box(stage, y, q.floor(), q.ceil(), Palette::dim())?);
        }
        let legend = [
            kit.text_left(stage, "upper: quota method", 22.0, Palette::ink(), v(1.2, 3.3))?,
            kit.text_left(stage, "lower: Hamilton", 22.0, Palette::dim(), v(1.2, 2.85))?,
        ];
        fade_in_all(stage, &[base.clone(), boxes.clone()].concat(), 0.6)?;
        let dy = ROW_H / 4.0;
        let mut upper = Vec::new();
        let mut lower = Vec::new();
        for i in 0..3 {
            for k in 0..history[15][i] {
                upper.push(token_at(
                    stage,
                    v(seat_x(k as f64 + 0.5), ROW_Y[i] + dy),
                    town_color(i),
                )?);
            }
            for k in 0..hamilton15[i] {
                lower.push(token_at(
                    stage,
                    v(seat_x(k as f64 + 0.5), ROW_Y[i] - dy),
                    Palette::dim(),
                )?);
            }
        }
        s.wait_for(stage, "the quota method gives one, four, ten")?;
        fade_in_all(stage, &[legend[0]], 0.3)?;
        fade_in_lagged(stage, &upper, 1.0, 0.1)?;
        s.wait_for(stage, "while Hamilton gives two, three, ten")?;
        fade_in_all(stage, &[legend[1]], 0.3)?;
        fade_in_lagged(stage, &lower, 1.0, 0.1)?;
        s.wait_for(stage, "Both keep the quota promise")?;
        // recolor the outlines only: the quota boxes stay hollow, so the cells
        // and tokens inside them stay readable
        let mut green = Vec::new();
        for (i, &b) in boxes.iter().enumerate() {
            let q = share(&POP, i, 15);
            green.push((b, quota_box(stage, ROW_Y[i], q.floor(), q.ceil(), Palette::green())?));
        }
        morph_all(stage, &green, 0.5)?;
        s.wait_for(stage, "can allow more than one answer")?;
        flash_all(stage, &boxes[..2], 1.0)?;
        let allowed: Vec<String> = (0..3)
            .map(|i| {
                let q = share(&POP, i, 15);
                if q.is_whole() {
                    format!("{} {}", NAMES[i], q.floor())
                } else {
                    format!("{} {} or {}", NAMES[i], q.floor(), q.ceil())
                }
            })
            .collect();
        let both = kit.text(
            stage,
            &format!("{}: both answers fit", allowed.join(", ")),
            22.0,
            Palette::green(),
            v(0.0, -2.8),
        )?;
        fade_in_all(stage, &[both], 0.5)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{PAUSE_NOTE, THINK_R};
    use super::*;

    const SAFE_X: f64 = 6.96;
    const SAFE_Y: f64 = 3.85;

    /// A generous width for `s` at `size`: this film's frames measure 0.0042
    /// to 0.0045 of the size per character.
    fn wide(s: &str, size: f64) -> f64 {
        s.chars().count() as f64 * size * 0.0048
    }

    /// Half the height of a line of text at `size`.
    fn half(size: f64) -> f64 {
        size * 0.005
    }

    /// One edge lies strictly beyond the other, up or to the right.
    #[track_caller]
    fn clears(beyond: f64, edge: f64) {
        assert!(beyond > edge, "{beyond} does not clear {edge}");
    }

    #[test]
    fn the_think_ring_covers_no_card() {
        let half_width = THINK_R.max(wide(PAUSE_NOTE, 16.0) / 2.0);
        // a glowing card reaches 0.07 past its edge
        clears(LIST_X - CARD_W / 2.0 - 0.07, PREDICT_AT.0 + half_width);
        // under the lowest row of the left panel, its note on screen
        clears(ROW_Y[2] - ROW_H / 2.0, PREDICT_AT.1 + THINK_R);
        clears(PREDICT_AT.1 - THINK_R - 0.28 - half(16.0), -SAFE_Y);
    }

    #[test]
    fn the_count_row_keeps_its_parts_apart() {
        let seats = (model::STRETCH.1 - model::STRETCH.0) as usize;
        let need = format!("{seats} seats from then to now: {seats} + 1 = {} tiles", seats + 1);
        let cannot = format!("{} tiles cannot fit", seats + 1);
        clears(NEED_X, -SAFE_X);
        clears(chip_x(0) - CHIP / 2.0, NEED_X + wide(&need, 22.0));
        clears(chip_x(1) - CHIP / 2.0, chip_x(0) + CHIP / 2.0);
        clears(CANNOT_X, chip_x(seats) + CHIP / 2.0);
        clears(SAFE_X, CANNOT_X + wide(&cannot, 24.0));
    }

    #[test]
    fn the_count_panel_clears_the_axis_and_the_frame() {
        // the axis's numbers sit 0.38 under it
        clears(AXIS_Y - 0.38 - half(22.0), PANEL_Y[0] + half(22.0));
        // the bracket under the chips stops above the bounds' row
        clears(PANEL_Y[0] - CHIP / 2.0 - 0.06, PANEL_Y[1] + half(22.0));
        clears(PANEL_Y[1] - half(22.0), PANEL_Y[2] + half(24.0));
        clears(PANEL_Y[2] - half(24.0), -SAFE_Y);
    }

    #[test]
    fn the_labels_in_the_stretch_clear_the_markers_and_rows() {
        let (from, to) = model::STRETCH;
        let (left, right) = (hx(from as f64) + MARK_GAP, hx(to as f64));
        clears(
            right,
            left + wide("Birch's seat 2 ends at 8 2/3, later than 7 1/2", 18.0),
        );
        clears(right, left + wide("6 to 7 1/2: not entered at 6", 18.0));
        // each sits in the gap above its row, rings (0.04 out) included
        let top = |row: usize| TILE_Y[row] + TILE_H / 2.0 + 0.04;
        let bottom = |row: usize| TILE_Y[row] - TILE_H / 2.0 - 0.04;
        let later_y = TILE_Y[BIRCH] + LATER_T_DY;
        clears(later_y - half(18.0), top(BIRCH));
        clears(bottom(ASH), later_y + half(18.0));
        let wait_y = TILE_Y[CEDAR] + WAIT_T_DY;
        clears(wait_y - half(18.0), top(CEDAR));
        clears(bottom(BIRCH), wait_y + half(18.0));
        // the legend sits between "suppose" and the `then` and `now` labels
        clears(3.3 - half(22.0), LEGEND_Y + half(18.0));
        clears(LEGEND_Y - half(18.0), TILE_Y[0] + 0.75 + half(22.0));
    }
}
