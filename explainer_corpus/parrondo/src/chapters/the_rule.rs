//! Chapter 02, the rule. Game B's card appears beside the player of 01 (the
//! teal token, its count and its seven chips) with two coins in it, red and
//! green, and the line "count your dollars" (02_1). The same seven chips
//! slide into rows of three: two full rows and one chip in an outlined
//! leftover row (02_2). Seven leaves one, a chip joins and eight leaves two,
//! another and nine fills three rows, the leftover row empty and red; that
//! leftover is the remainder, always 0, 1 or 2 (02_3). Remainder 0 picks the
//! bad coin, heads 9.5 % (02_4); 1 or 2 the good coin, heads 74.5 %, and the
//! badge gains the three published chances (02_5). With ten dollars, in a
//! loose heap, which coin? (02_6, a prediction) The good coin: the ten chips
//! make three rows and one (02_7). A demonstration from seven: the good coin
//! wins twice, the chips go 8 then 9, the leftover row empties, and the bad
//! coin comes to the player (02_8).
//!
//! Every count, remainder and chance on screen comes from `crate::model`
//! and `crate::facts`; the chips themselves are the count (the leftover row
//! holds exactly the remainder's chips), and the scene refuses to draw if
//! the words would stop being true of the facts.

use explainer_kit::{Kit, Palette, Timed, fade_all, rect, tween, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    badge, card, chip, coin, fade_in_all, fade_out_all, fade_to, fail, flash_all, fly, keep_between, line_style, now,
    say, stand, think, together, token,
};
use crate::facts;
use crate::model::{MODULUS, PUBLISHED, Ratio, START};
use crate::palette;

pub struct TheRule {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const GAME_B: &str = "Game B";
const PLAYER: &str = "player";
const DOLLARS: &str = "dollars";
const COUNT_LINE: &str = "count your dollars";
const LEFT_OVER: &str = "left over";
const REMAINDER_LINE: &str = "the remainder: always 0, 1 or 2";
const BAD_FACE: &str = "bad";
const GOOD_FACE: &str = "good";
const BAD_RULE: &str = "remainder 0 \u{2192} bad coin";
const GOOD_RULE: &str = "remainder 1 or 2 \u{2192} good coin";
const EXAMPLE: &str = "for example";
const PLUS_ONE: &str = "+1";
/// 01's badge words (`the_puzzle::BADGE`), until the chances join at 02_5.
const BADGE: &str = "published coins";

/// Where each picture lands: (line, the phrase it waits for).
const AT_COINS: (&str, &str) = ("02_1", "two coins");
const AT_COUNT: (&str, &str) = ("02_1", "counts their dollars");
const AT_ROWS: (&str, &str) = ("02_2", "rows of three");
const AT_LEFT: (&str, &str) = ("02_2", "left over");
const AT_SEVEN: (&str, &str) = ("02_3", "Seven dollars make");
const AT_EIGHT: (&str, &str) = ("02_3", "Eight leave");
const AT_NINE: (&str, &str) = ("02_3", "Nine fill");
const AT_NAMED: (&str, &str) = ("02_3", "That leftover is the remainder");
const AT_BAD: (&str, &str) = ("02_4", "the bad coin");
const AT_BAD_HEADS: (&str, &str) = ("02_4", "nine and a half");
const AT_GOOD: (&str, &str) = ("02_5", "the good coin");
const AT_GOOD_HEADS: (&str, &str) = ("02_5", "seventy-four");
const AT_TEN: (&str, &str) = ("02_6", "With ten dollars");
const AT_TEN_ROWS: (&str, &str) = ("02_7", "Ten dollars make");
const AT_TEN_LEFT: (&str, &str) = ("02_7", "so the remainder is one");
const AT_TO_EIGHT: (&str, &str) = ("02_8", "Seven becomes eight");
const AT_TO_NINE: (&str, &str) = ("02_8", "eight becomes nine");
const AT_BAD_OUT: (&str, &str) = ("02_8", "out comes the bad coin");
#[cfg(test)]
const CUES: [(&str, &str); 18] = [
    AT_COINS,
    AT_COUNT,
    AT_ROWS,
    AT_LEFT,
    AT_SEVEN,
    AT_EIGHT,
    AT_NINE,
    AT_NAMED,
    AT_BAD,
    AT_BAD_HEADS,
    AT_GOOD,
    AT_GOOD_HEADS,
    AT_TEN,
    AT_TEN_ROWS,
    AT_TEN_LEFT,
    AT_TO_EIGHT,
    AT_TO_NINE,
    AT_BAD_OUT,
];

// ------------------------------------------------------------------ layout

/// The player where 01 had them: token, label, live count, a row of chips.
const TOKEN_AT: Vec3 = [-5.6, 0.9, 0.0];
const PLAYER_LABEL_AT: Vec3 = [-5.6, 0.35, 0.0];
const COUNT_AT: Vec3 = [-4.9, 1.5, 0.0];
const DOLLARS_X: f64 = -4.1;
const ROW_X0: f64 = -4.85;
const ROW_PITCH: f64 = 0.3;
const ROW_Y: f64 = 0.9;
/// The chips in rows of three: chip i in column i % 3 of row i / 3, rows
/// going down from the token's height.
const GRID_X0: f64 = -3.9;
const GRID_Y0: f64 = 0.9;
const PITCH: f64 = 0.42;
const LABEL_GAP: f64 = 0.15;
/// The lines under the rows: one per count, then the remainder named.
const EX_Y0: f64 = -1.0;
const EX_DY: f64 = 0.4;
/// 02_6: ten chips in a loose heap (a sunflower spiral, so no two touch and
/// no three line up as a row).
const LOOSE_AT: Vec3 = [-3.48, 0.2, 0.0];
const LOOSE_C: f64 = 0.21;
const GOLDEN: f64 = 2.399_963_229_728_653;
const THINK_AT: Vec3 = [-1.3, 0.4, 0.0];
/// Card B on the right, its two coins and their rules.
const CARD_AT: Vec3 = [3.5, 0.4, 0.0];
const CARD_W: f64 = 6.2;
const CARD_H: f64 = 4.0;
const COUNT_LINE_DY: f64 = 0.8;
const COIN_R: f64 = 0.42;
const BAD_AT: Vec3 = [1.2, 0.8, 0.0];
const GOOD_AT: Vec3 = [1.2, -0.8, 0.0];
const RULE_X: f64 = 1.9;
const RULE_DY: f64 = 0.17;
const HEADS_DY: f64 = 0.25;
/// Dimmed coins and rules while the viewer predicts.
const DIM: f64 = 0.3;
/// 02_8: the demonstration's labels, and where the bad coin comes to rest
/// (01's coin, under the token).
const EXAMPLE_AT: Vec3 = [-5.6, -1.75, 0.0];
const PLUS_AT: Vec3 = [-4.75, 0.9, 0.0];
const BAD_HOME: Vec3 = [-5.6, -1.0, 0.0];
/// Labels keep this far from their neighbours and the frame's edge.
const ROOM: f64 = 0.2;

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    bad_heads: String,
    good_heads: String,
    full_badge: String,
    /// The counts the chapter divides (7, 8, 9, 10) and their remainders.
    rem: Vec<(usize, usize)>,
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let [a, bad, good] = facts::coins(&PUBLISHED);
    let (a_pct, bad_pct, good_pct) = (a.exact_pct()?, bad.exact_pct()?, good.exact_pct()?);
    if get("coins.published.heads.pct")? != format!("{a_pct},{bad_pct},{good_pct}") {
        return Err("the badge's chances disagree with the published coins".into());
    }
    let half = Ratio::new(1, 2);
    if !(bad < half && half < good) {
        return Err("02_4 and 02_5 call one coin bad and the other good, but they are not".into());
    }
    if get("rules.modulus")? != "3" || MODULUS != 3 {
        return Err("02_2 says rows of three, but game B's rule divides by another number".into());
    }
    // the remainders the facts print ("6:0,7:1,…"), for the counts the words use
    let examples = get("remainder.examples")?;
    let printed = |count: usize| -> Result<usize, String> {
        examples
            .split(',')
            .find_map(|e| e.strip_prefix(&format!("{count}:")))
            .ok_or_else(|| format!("no remainder fact for {count}"))?
            .parse()
            .map_err(|e| format!("remainder of {count}: {e}"))
    };
    let start = usize::try_from(START).map_err(|_| "the player starts below zero".to_owned())?;
    let rem = (start..=10)
        .map(|count| Ok((count, printed(count)?)))
        .collect::<Result<Vec<_>, String>>()?;
    // the words: seven leaves one, eight two, nine nothing, ten one
    if rem != [(7, 1), (8, 2), (9, 0), (10, 1)] {
        return Err(format!(
            "02_3 and 02_7 say 7, 8, 9, 10 leave 1, 2, 0, 1, but the facts say {rem:?}"
        ));
    }
    if rem.iter().any(|&(count, r)| count % 3 != r) {
        return Err("a remainder fact is not the count's remainder".into());
    }
    Ok(Numbers {
        bad_heads: format!("heads {bad_pct} %"),
        good_heads: format!("heads {good_pct} %"),
        full_badge: format!("{BADGE} \u{b7} {a_pct} \u{b7} {bad_pct} \u{b7} {good_pct} %"),
        rem,
    })
}

/// 01's row of chips, where the chapter picks the player up.
fn row_at(i: usize) -> Vec3 {
    v(ROW_X0 + ROW_PITCH * i as f64, ROW_Y)
}

/// Chip `i` in rows of three.
fn slot(i: usize) -> Vec3 {
    v(GRID_X0 + PITCH * (i % 3) as f64, GRID_Y0 - PITCH * (i / 3) as f64)
}

/// The leftover row for `count` chips: the row after the full ones, which
/// holds `count % 3` chips (none when the rows close).
fn leftover_row(count: usize) -> usize {
    count / 3
}

/// Chip `i` of the loose heap.
fn loose_at(i: usize) -> Vec3 {
    let a = GOLDEN * i as f64;
    let r = LOOSE_C * (i as f64 + 0.5).sqrt();
    v(LOOSE_AT[0] + r * a.cos(), LOOSE_AT[1] + r * a.sin())
}

/// The line under the rows for example `k`: "7 → remainder 1".
fn example_text(n: &Numbers, k: usize) -> String {
    let (count, r) = n.rem[k];
    format!("{count} \u{2192} remainder {r}")
}

/// The outlined leftover row for `count` chips and its label. Detached.
fn mark_leftover(kit: &Kit, stage: &mut Stage<'_>, count: usize, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let y = GRID_Y0 - PITCH * leftover_row(count) as f64;
    let outline = rect(
        v(GRID_X0 - PITCH / 2.0, y - PITCH / 2.0),
        3.0 * PITCH,
        PITCH,
        line_style(color, 2.5),
    )?;
    let outline = stage.arena_mut().add(outline);
    let label = kit.text_left(stage, LEFT_OVER, 20.0, color, v(GRID_X0 + 2.5 * PITCH + LABEL_GAP, y))?;
    keep_between(
        stage,
        label,
        GRID_X0 + 2.5 * PITCH,
        CARD_AT[0] - CARD_W / 2.0 - ROOM,
        "the leftover label",
    )?;
    Ok([outline, label])
}

/// One chip joins at the next slot of the rows while the count tweens up.
fn add_chip(stage: &mut Stage<'_>, count: Mob, chips: &mut Vec<Mob>, rt: f64) -> fmn::Result<()> {
    let c = chip(stage, slot(chips.len()));
    chips.push(c);
    let appear = fade_in(stage.arena_mut(), c, ORIGIN, 1.0)?;
    let a = stage.prepare(appear.rt(rt))?;
    let t = stage.prepare(tween(count, chips.len() as f64, rt)?)?;
    stage.play_prepared(vec![a, t])?;
    Ok(())
}

/// Every chip slides to `place(i)` together.
fn arrange(stage: &mut Stage<'_>, chips: &[Mob], place: fn(usize) -> Vec3, rt: f64) -> fmn::Result<()> {
    let moves = chips
        .iter()
        .enumerate()
        .map(|(i, &c)| fly(c, place(i), rt))
        .collect::<fmn::Result<Vec<_>>>()?;
    together(stage, moves)
}

/// Fade `old` out while `new` fades in, `rt` seconds.
fn swap(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in old {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in new {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Bring `mobs` to `opacity` together.
fn all_to(stage: &mut Stage<'_>, mobs: &[Mob], opacity: f64, rt: f64) -> fmn::Result<()> {
    let moves = mobs
        .iter()
        .map(|&m| fade_to(m, opacity, rt))
        .collect::<fmn::Result<Vec<_>>>()?;
    together(stage, moves)
}

impl SceneConstruct for TheRule {
    fn name(&self) -> &str {
        "02_the_rule"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let start = n.rem[0].0;
        let (card_left, card_right) = (CARD_AT[0] - CARD_W / 2.0, CARD_AT[0] + CARD_W / 2.0);

        // 02_1: the player as 01 left them, and game B's card with two coins
        let line = say(&kit, stage, "02_1")?;
        let player = token(stage, TOKEN_AT);
        let label = kit.text(stage, PLAYER, 18.0, Palette::dim(), PLAYER_LABEL_AT)?;
        let dollars_word = kit.text_left(stage, DOLLARS, 26.0, Palette::dim(), v(DOLLARS_X, COUNT_AT[1]))?;
        let mut chips: Vec<Mob> = (0..start).map(|i| chip(stage, row_at(i))).collect();
        let tag = badge(&kit, stage, BADGE, true)?;
        keep_between(stage, tag[0], card_left, FRAME_X_RADIUS - ROOM, "the coin badge")?;
        let card_b = card(&kit, stage, GAME_B, CARD_AT, CARD_W, CARD_H, palette::game_b())?;
        let mut arriving = vec![player, label, dollars_word];
        arriving.extend(&chips);
        arriving.extend(tag);
        arriving.extend(card_b);
        fade_in_all(stage, &arriving, 0.6)?;
        let count_value = stage.add_value_tracker(start as f64);
        kit.live_number(stage, count_value, 0, false, 34.0, palette::player(), COUNT_AT, LEFT)?;
        line.wait_for(stage, AT_COINS.1)?;
        let bad = coin(&kit, stage, BAD_AT, COIN_R, palette::bad(), BAD_FACE)?;
        let good = coin(&kit, stage, GOOD_AT, COIN_R, palette::good(), GOOD_FACE)?;
        fade_in_all(stage, &[bad[0], bad[1], good[0], good[1]], 0.5)?;
        line.wait_for(stage, AT_COUNT.1)?;
        let count_line = kit.text(
            stage,
            COUNT_LINE,
            24.0,
            Palette::hero(),
            v(CARD_AT[0], CARD_AT[1] + CARD_H / 2.0 - COUNT_LINE_DY),
        )?;
        keep_between(
            stage,
            count_line,
            card_left + ROOM,
            card_right - ROOM,
            "count your dollars",
        )?;
        fade_in_all(stage, &[count_line], 0.4)?;
        flash_all(stage, &[count_line], 0.6)?;

        // 02_2: the same chips in rows of three; what is left over
        let line = say(&kit, stage, "02_2")?;
        line.wait_for(stage, AT_ROWS.1)?;
        arrange(stage, &chips, slot, 0.9)?;
        line.wait_for(stage, AT_LEFT.1)?;
        let mut leftover = mark_leftover(&kit, stage, chips.len(), Palette::hero())?;
        fade_in_all(stage, &leftover, 0.4)?;

        // 02_3: seven, eight, nine; the leftover is the remainder
        let line = say(&kit, stage, "02_3")?;
        line.wait_for(stage, AT_SEVEN.1)?;
        let first = kit.text_left(
            stage,
            &example_text(&n, 0),
            22.0,
            Palette::ink(),
            v(GRID_X0 - PITCH / 2.0, EX_Y0),
        )?;
        fade_in_all(stage, &[first], 0.4)?;
        let mut examples = vec![first];
        for (k, at) in [(1, AT_EIGHT), (2, AT_NINE)] {
            line.wait_for(stage, at.1)?;
            add_chip(stage, count_value, &mut chips, 0.5)?;
            let closed = n.rem[k].1 == 0;
            let color = if closed { palette::bad() } else { Palette::hero() };
            let next = mark_leftover(&kit, stage, chips.len(), color)?;
            let words = kit.text_left(
                stage,
                &example_text(&n, k),
                22.0,
                if closed { palette::bad() } else { Palette::ink() },
                v(GRID_X0 - PITCH / 2.0, EX_Y0 - EX_DY * k as f64),
            )?;
            swap(stage, &leftover, &[next[0], next[1], words], 0.35)?;
            leftover = next;
            examples.push(words);
        }
        line.wait_for(stage, AT_NAMED.1)?;
        let named = kit.text_left(
            stage,
            REMAINDER_LINE,
            24.0,
            Palette::hero(),
            v(GRID_X0 - PITCH / 2.0, EX_Y0 - EX_DY * 3.0),
        )?;
        keep_between(
            stage,
            named,
            -FRAME_X_RADIUS + ROOM,
            card_left - ROOM,
            "the remainder named",
        )?;
        fade_in_all(stage, &[named], 0.4)?;

        // 02_4: remainder 0, the bad coin
        let line = say(&kit, stage, "02_4")?;
        line.wait_for(stage, AT_BAD.1)?;
        let bad_rule = kit.text_left(stage, BAD_RULE, 24.0, palette::bad(), v(RULE_X, BAD_AT[1] + RULE_DY))?;
        keep_between(stage, bad_rule, card_left, card_right - ROOM, "the bad coin's rule")?;
        fade_in_all(stage, &[bad_rule], 0.4)?;
        flash_all(stage, &[bad[0]], 0.6)?;
        line.wait_for(stage, AT_BAD_HEADS.1)?;
        let bad_heads = kit.text_left(
            stage,
            &n.bad_heads,
            22.0,
            palette::bad(),
            v(RULE_X, BAD_AT[1] - HEADS_DY),
        )?;
        fade_in_all(stage, &[bad_heads], 0.4)?;

        // 02_5: remainder 1 or 2, the good coin; the badge gains the chances
        let line = say(&kit, stage, "02_5")?;
        line.wait_for(stage, AT_GOOD.1)?;
        let good_rule = kit.text_left(stage, GOOD_RULE, 24.0, palette::good(), v(RULE_X, GOOD_AT[1] + RULE_DY))?;
        keep_between(stage, good_rule, card_left, card_right - ROOM, "the good coin's rule")?;
        fade_in_all(stage, &[good_rule], 0.4)?;
        flash_all(stage, &[good[0]], 0.6)?;
        line.wait_for(stage, AT_GOOD_HEADS.1)?;
        let good_heads = kit.text_left(
            stage,
            &n.good_heads,
            22.0,
            palette::good(),
            v(RULE_X, GOOD_AT[1] - HEADS_DY),
        )?;
        fade_in_all(stage, &[good_heads], 0.4)?;
        let full = badge(&kit, stage, &n.full_badge, true)?;
        // the longer badge keeps its right edge inside the frame
        let right = FRAME_X_RADIUS - ROOM;
        if stage.get_bounding_box_point(full[0], RIGHT)[0] > right {
            stage.move_to(full[0], v(right, tag_y()), RIGHT);
            stage.move_to(full[1], v(right - 0.15, tag_y()), RIGHT);
        }
        keep_between(stage, full[0], card_left, right, "the coin badge with its chances")?;
        kit.hold(stage)?;
        swap(stage, &tag, &full, 0.5)?;

        // 02_6: ten loose chips; which coin? (a prediction)
        let line = say(&kit, stage, "02_6")?;
        let mut clearing = examples.clone();
        clearing.push(named);
        clearing.extend(leftover);
        fade_out_all(stage, &clearing, 0.4)?;
        let rules = [
            bad[0], bad[1], bad_rule, bad_heads, good[0], good[1], good_rule, good_heads,
        ];
        all_to(stage, &rules, DIM, 0.4)?;
        line.wait_for(stage, AT_TEN.1)?;
        add_chip(stage, count_value, &mut chips, 0.4)?;
        arrange(stage, &chips, loose_at, 0.8)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 02_7: the good coin; three rows and one
        let line = say(&kit, stage, "02_7")?;
        fade_out_all(stage, &[q], 0.3)?;
        all_to(stage, &rules[4..], 1.0, 0.4)?;
        flash_all(stage, &[good[0]], 0.6)?;
        line.wait_for(stage, AT_TEN_ROWS.1)?;
        arrange(stage, &chips, slot, 0.9)?;
        leftover = mark_leftover(&kit, stage, chips.len(), Palette::hero())?;
        fade_in_all(stage, &leftover, 0.3)?;
        line.wait_for(stage, AT_TEN_LEFT.1)?;
        let ten = kit.text_left(
            stage,
            &example_text(&n, 3),
            22.0,
            Palette::ink(),
            v(GRID_X0 - PITCH / 2.0, EX_Y0),
        )?;
        fade_in_all(stage, &[ten], 0.4)?;

        // 02_8: a demonstration from seven: two wins, and the bad coin
        let line = say(&kit, stage, "02_8")?;
        let mut leaving: Vec<Mob> = chips.drain(start..).collect();
        leaving.push(ten);
        leaving.extend(leftover);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &leaving {
            let away = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(away.rt(0.5))?);
        }
        anims.push(stage.prepare(tween(count_value, start as f64, 0.5)?)?);
        stage.play_prepared(anims)?;
        let example = kit.text(stage, EXAMPLE, 18.0, Palette::dim(), EXAMPLE_AT)?;
        leftover = mark_leftover(&kit, stage, chips.len(), Palette::hero())?;
        fade_in_all(stage, &[example, leftover[0], leftover[1]], 0.4)?;
        let mut shown: Vec<Mob> = Vec::new();
        for (k, at) in [(1, AT_TO_EIGHT), (2, AT_TO_NINE)] {
            line.wait_for(stage, at.1)?;
            let plus = kit.text(stage, PLUS_ONE, 30.0, palette::good(), PLUS_AT)?;
            fade_in_all(stage, &[plus], 0.2)?;
            flash_all(stage, &[good[0]], 0.4)?;
            add_chip(stage, count_value, &mut chips, 0.4)?;
            let closed = n.rem[k].1 == 0;
            let next = mark_leftover(
                &kit,
                stage,
                chips.len(),
                if closed { palette::bad() } else { Palette::hero() },
            )?;
            let words = kit.text_left(
                stage,
                &example_text(&n, k),
                22.0,
                if closed { palette::bad() } else { Palette::ink() },
                v(GRID_X0 - PITCH / 2.0, EX_Y0),
            )?;
            let mut old = leftover.to_vec();
            old.push(plus);
            old.append(&mut shown);
            swap(stage, &old, &[next[0], next[1], words], 0.3)?;
            leftover = next;
            shown.push(words);
        }
        line.wait_for(stage, AT_BAD_OUT.1)?;
        // the rows close, and the bad coin, lit again with its rule, comes to
        // the player
        let mut comes = vec![fade_to(bad_rule, 1.0, 0.8)?, fade_to(bad_heads, 1.0, 0.8)?];
        for &m in &bad {
            comes.push(
                m.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.8),
                        ..AnimateArgs::default()
                    })?
                    .move_to(BAD_HOME, ORIGIN)?
                    .set_opacity(1.0)?,
            );
        }
        together(stage, comes)?;
        flash_all(stage, &[leftover[0], bad[0]], 0.6)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// The badge's height on screen (both of its parts are centred on it).
fn tag_y() -> f64 {
    super::draw::BADGE_AT[1]
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};
    use fmn::core::constants::FRAME_Y_RADIUS;
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{CHIP_R, THINK_S, TOKEN_R};
    use super::*;

    /// The chapter's words are true of its numbers: 7, 8, 9 and 10 leave
    /// 1, 2, 0 and 1; the coins are the published ones, the bad one below
    /// a half and the good one above; the badge gains exactly those chances.
    #[test]
    fn the_rule_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.rem, [(7, 1), (8, 2), (9, 0), (10, 1)]);
        assert_eq!(
            (n.bad_heads.as_str(), n.good_heads.as_str()),
            ("heads 9.5 %", "heads 74.5 %")
        );
        assert_eq!(n.full_badge, "published coins \u{b7} 49.5 \u{b7} 9.5 \u{b7} 74.5 %");
        assert_eq!(example_text(&n, 0), "7 \u{2192} remainder 1");
        assert_eq!(example_text(&n, 2), "9 \u{2192} remainder 0");
        assert_eq!(example_text(&n, 3), "10 \u{2192} remainder 1");
    }

    /// The chips are the count: for every count the chapter shows, the full
    /// rows hold three chips each, the outlined leftover row holds exactly
    /// the remainder's chips (none at nine), and no chip sits below it.
    #[test]
    fn the_leftover_row_holds_the_remainder() {
        let n = numbers().unwrap();
        for &(count, r) in &n.rem {
            let row = leftover_row(count);
            let y = GRID_Y0 - PITCH * row as f64;
            let in_row = (0..count).filter(|&i| (slot(i)[1] - y).abs() < 1e-9).count();
            assert_eq!(in_row, r, "{count} chips");
            for full in 0..row {
                let yf = GRID_Y0 - PITCH * full as f64;
                assert_eq!((0..count).filter(|&i| (slot(i)[1] - yf).abs() < 1e-9).count(), 3);
            }
            assert!(
                (0..count).all(|i| slot(i)[1] >= y - 1e-9),
                "{count}: a chip below the leftover row"
            );
        }
    }

    /// 02_6's heap is not rows: no two chips touch, and the ten sit at more
    /// than four heights (rows of three would give four).
    #[test]
    fn the_loose_heap_is_not_in_rows() {
        let heap: Vec<Vec3> = (0..10).map(loose_at).collect();
        for i in 0..heap.len() {
            for j in i + 1..heap.len() {
                let d = ((heap[i][0] - heap[j][0]).powi(2) + (heap[i][1] - heap[j][1]).powi(2)).sqrt();
                assert!(d > 2.0 * CHIP_R + 0.05, "chips {i} and {j} touch ({d:.3})");
            }
        }
        let mut heights: Vec<i64> = heap.iter().map(|p| (p[1] * 100.0).round() as i64).collect();
        heights.sort_unstable();
        heights.dedup();
        assert!(heights.len() > 4, "{heights:?}");
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the rows, the heap and their labels stay
    /// between the token and card B, the lines under the rows clear the
    /// fourth row and the frame's foot, the think ring clears the heap and
    /// the card, the coins sit inside the card, and the bad coin's resting
    /// place clears the player's label and the example label.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let card_left = CARD_AT[0] - CARD_W / 2.0;
        let rows_left = GRID_X0 - PITCH / 2.0;
        assert!(rows_left > TOKEN_AT[0] + TOKEN_R + ROOM && row_at(0)[0] - CHIP_R > TOKEN_AT[0] + TOKEN_R);
        assert!(PLUS_AT[0] - 0.25 > TOKEN_AT[0] + TOKEN_R && PLUS_AT[0] + 0.25 < rows_left);
        let fourth_row_foot = GRID_Y0 - PITCH * 3.0 - PITCH / 2.0;
        assert!(EX_Y0 + 0.15 < fourth_row_foot, "{fourth_row_foot}");
        const {
            assert!(EX_Y0 - EX_DY * 3.0 - 0.2 > -FRAME_Y_RADIUS + 1.0);
        }
        let heap_right = (0..10).map(|i| loose_at(i)[0]).fold(f64::MIN, f64::max) + CHIP_R;
        let heap_left = (0..10).map(|i| loose_at(i)[0]).fold(f64::MAX, f64::min) - CHIP_R;
        assert!(heap_left > TOKEN_AT[0] + TOKEN_R + ROOM && heap_right < THINK_AT[0] - 0.62 - ROOM);
        assert!(THINK_AT[0] + 0.62 + ROOM < card_left);
        let count_line_y = CARD_AT[1] + CARD_H / 2.0 - COUNT_LINE_DY;
        for c in [BAD_AT, GOOD_AT] {
            assert!(c[0] - COIN_R > card_left + 0.1 && c[0] + COIN_R < RULE_X);
            assert!(c[1] + COIN_R < count_line_y - 0.15 && c[1] - COIN_R > CARD_AT[1] - CARD_H / 2.0 + 0.1);
            // the rule's two lines stay inside the card too
            assert!(c[1] + RULE_DY + 0.15 < count_line_y - 0.15 && c[1] - HEADS_DY - 0.15 > CARD_AT[1] - CARD_H / 2.0);
        }
        const {
            assert!(BAD_HOME[1] + COIN_R < PLAYER_LABEL_AT[1] - 0.15);
        }
        const {
            assert!(BAD_HOME[1] - COIN_R > EXAMPLE_AT[1] + 0.12);
        }
        assert!(CARD_AT[0] + CARD_W / 2.0 < FRAME_X_RADIUS && CARD_AT[1] + CARD_H / 2.0 < tag_y() - 0.3);
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` guard meets the true text
    /// widths): every line of chapter 02 is said once, in order, and the
    /// prediction (02_6) gets its silent think time before the answer.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TheRule { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("02_the_rule: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=8).map(|k| format!("02_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let asked_end = at("02_6") + narrator.length("02_6").unwrap();
        assert!(
            at("02_7") - asked_end >= THINK_S,
            "{} s to think",
            at("02_7") - asked_end
        );
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end > at("02_8"), "the chapter ends after its last line: {end}");
    }
}
