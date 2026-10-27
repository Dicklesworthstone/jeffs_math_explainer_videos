//! Chapter 03, does game B win? A strip of counts 6 to 14, coloured by their
//! remainder: two in every three are green, every third red (03_1). If the
//! good coin really got two rounds in three, B would win about 52.8 % of
//! them: a hypothetical, labelled "if" and written ≈ (03_2). Win or lose?
//! (03_3, a prediction) It loses: "B would win" is struck through, card B
//! shrinks to the top where 01 kept its cards, and the long window of 01_9
//! returns with A's run faint; the same player plays 100,000 rounds of game
//! B, the token riding the run's tip, 548 dollars down, less far than A
//! (03_4). Then the run replays from its start, brighter, over the first
//! drawing, and one readout counts the rounds that began at a multiple of
//! three over exactly the rounds drawn so far, beside "a third of the rounds
//! would be 33.3 %": it ends at 38.4 % (03_5). Those extra rounds are what
//! sink B; the picture leaves a thumbnail (03_6).
//!
//! The readout and the replayed path are one prefix of one run: the same
//! tracker picks the drawn point and the share, and the share at the last
//! point is the run's own count. At round 0 no rounds have been drawn, so
//! there is no share: the number and its "%" come in only with the first
//! drawn point past it, never early. Every number comes from `crate::model` and
//! `crate::facts`; the scene refuses to draw if the words would stop being
//! true of them (B loses, less than A; more than 38 % at a multiple of three;
//! the hypothetical above a half).

use explainer_kit::{Frame2, Kit, Palette, fade_all, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    BADGE_AT, badge, card, coin, draw_run, fade_in_all, fade_out_all, fade_to, fail, flash_all, fly, growing_run,
    keep_between, line_style, now, say, shrink_all, stand, think, thinned, together, token,
};
use crate::facts;
use crate::model::{self, MODULUS, PUBLISHED, Play, RUN_ROUNDS, Ratio, START};
use crate::palette;

pub struct DoesBWin {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const GAME_B: &str = "Game B";
const COUNT_LINE: &str = "count your dollars";
const BAD_FACE: &str = "bad";
const GOOD_FACE: &str = "good";
const BAD_RULE: &str = "remainder 0 \u{2192} bad coin";
const GOOD_RULE: &str = "remainder 1 or 2 \u{2192} good coin";
const DOLLARS: &str = "dollars";
const REMAINDER: &str = "remainder";
const IF_LINE: &str = "if the good coin got two rounds in three:";
const WOULD_WIN: &str = "B would win";
const ROUNDS: &str = "rounds";
const A_END: &str = "A";
const B_END: &str = "B";
const AT_ZERO_LINE: &str = "rounds at a multiple of 3:";
const PERCENT: &str = "%";
/// 01's and 02's badge words; by now with the chances 02_5 gave them.
const BADGE: &str = "published coins";

/// Where each picture lands: (line, the phrase it waits for).
const AT_EVERY_THREE: (&str, &str) = ("03_1", "Two counts in every three");
const AT_WOULD: (&str, &str) = ("03_2", "would win");
const AT_PLAYER: (&str, &str) = ("03_4", "Here's the same player");
const AT_RUN: (&str, &str) = ("03_4", "a hundred thousand rounds of game B");
const AT_SLOWER: (&str, &str) = ("03_4", "more slowly");
const AT_MORE: (&str, &str) = ("03_5", "but more than thirty-eight percent");
const AT_SINK: (&str, &str) = ("03_6", "what sink game B");
#[cfg(test)]
const CUES: [(&str, &str); 7] = [AT_EVERY_THREE, AT_WOULD, AT_PLAYER, AT_RUN, AT_SLOWER, AT_MORE, AT_SINK];

// ------------------------------------------------------------------ layout

/// 03_1: the counts 6 to 14 across the left half, their remainders under
/// them.
const STRIP: (i64, i64) = (6, 14);
const STRIP_X0: f64 = -6.2;
const STRIP_PITCH: f64 = 0.75;
const STRIP_Y: f64 = 1.95;
const STRIP_REM_DY: f64 = 0.45;
const STRIP_CAPTION_DY: f64 = 0.45;
/// 03_2 and 03_3: the hypothetical and the prediction, under the strip.
const LEFT_X: f64 = -3.2;
const IF_Y: f64 = 0.6;
const HYPO_Y: f64 = -0.15;
const WOULD_Y: f64 = -0.9;
const THINK_AT: Vec3 = [-3.2, -2.0, 0.0];
/// Card B as 02 left it, on the right.
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
/// 03_4: card B shrinks into card B's slot of 01's shrunk pair, and its
/// "count your dollars" stays readable beside it.
const CARD_SLOT: Vec3 = [-2.6, 3.15, 0.0];
const CARD_SHRINK: f64 = 0.3;
const COUNT_SLOT: Vec3 = [0.15, 3.15, 0.0];
/// The long run's window of 01_9: 100,000 rounds, dollars −1,200 to +200.
const LONG: Frame2 = Frame2 {
    center: [0.8, 0.15, 0.0],
    width: 10.6,
    height: 4.0,
    x: (0.0, 100_000.0),
    y: (-1200.0, 200.0),
};
const TICK_DY: f64 = 0.22;
/// Every 250th round of the runs is drawn (401 points), as in 01.
const THIN: usize = 250;
const RUN_S: f64 = 7.0;
const REPLAY_S: f64 = 6.0;
const FAINT: f64 = 0.35;
const END_LABEL_DX: f64 = 0.25;
/// 03_4 and 03_5: the readouts, top right, where B's run and the token
/// riding it stay below them (the layout test checks every drawn point).
const END_AT: Vec3 = [3.6, 2.05, 0.0];
const PANEL_X: f64 = 1.6;
const AT_ZERO_Y: f64 = 1.68;
const VALUE_X: f64 = 4.85;
const PERCENT_DX: f64 = 0.08;
const THIRD_Y: f64 = 1.32;
/// 03_6: the thumbnail the run leaves at the top left, where 01's was.
const THUMB_FACTOR: f64 = 0.26;
const THUMB_AT: Vec3 = [-5.0, 1.75, 0.0];
/// Labels keep this far from their neighbours and the frame's edge.
const ROOM: f64 = 0.2;

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    strip: Vec<(i64, i64)>,
    bad_heads: String,
    good_heads: String,
    full_badge: String,
    hypo_tex: String,
    third_line: String,
    a: model::Run,
    b: model::Run,
    end_line: String,
    /// At each drawn point of B's run (rounds 0, 250, …, 100,000), the share
    /// in percent of the rounds so far that began at a multiple of three;
    /// none at round 0, which has no rounds behind it.
    shares: Vec<Option<f64>>,
}

/// The rounds of a run, among its first `rounds`, that began at a multiple
/// of three (game B's bad-coin rounds).
fn at_zero_before(dollars: &[i64], rounds: usize) -> usize {
    dollars[..rounds].iter().filter(|d| d.rem_euclid(MODULUS) == 0).count()
}

/// The point a run drawn by `growing_run` shows at tracker value `f`, out of
/// `len` points (the same rounding as `growing_run`).
fn drawn_index(len: usize, f: f64) -> usize {
    ((len - 1) as f64 * f.clamp(0.0, 1.0)).round() as usize
}

/// The share the readout shows at tracker value `f`: the one for exactly
/// the rounds drawn so far, and none while only round 0 is drawn.
fn shown_share(shares: &[Option<f64>], f: f64) -> Option<f64> {
    shares[drawn_index(shares.len(), f)]
}

/// The tracker value at which a run drawn by `growing_run` over `len`
/// points shows its first point past round 0.
fn first_drawn(len: usize) -> f64 {
    1.0 / (len - 1) as f64
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let [a_coin, bad, good] = facts::coins(&PUBLISHED);
    let (a_pct, bad_pct, good_pct) = (a_coin.exact_pct()?, bad.exact_pct()?, good.exact_pct()?);
    let strip: Vec<(i64, i64)> = (STRIP.0..=STRIP.1).map(|c| (c, c.rem_euclid(MODULUS))).collect();
    // 03_1: two counts in every three leave something over
    if strip.iter().filter(|&&(_, r)| r != 0).count() * 3 != strip.len() * 2 {
        return Err("03_1 says two counts in three, but the strip disagrees".into());
    }
    // 03_2: the good coin two rounds in three, the bad one in one
    let thirds = Ratio::new(1, 3) * bad + Ratio::new(2, 3) * good;
    if thirds != facts::thirds_win(&PUBLISHED) || thirds.pct(1) != get("pub.b.thirds.win.pct")? {
        return Err("03_2's hypothetical disagrees with the facts".into());
    }
    if thirds <= Ratio::new(1, 2) {
        return Err("03_2 says B would win, but its hypothetical share is not above a half".into());
    }
    // the hypothetical is rounded, so the screen writes ≈
    let equals = if thirds.exact_pct().is_ok_and(|s| s == thirds.pct(1)) {
        "="
    } else {
        r"\approx"
    };
    let a = model::player(Play::A);
    let b = model::player(Play::B);
    // 03_4: B loses, less far than A; 03_5: more than 38 % at a multiple of 3
    if b.net() >= 0 || b.net() <= a.net() {
        return Err(format!(
            "03_4 says B loses more slowly than A, but they end {} and {}",
            b.net(),
            a.net()
        ));
    }
    let share = b.share_at_zero();
    if share <= Ratio::new(38, 100) || share.pct(1) != get("run.b.at0.pct")? {
        return Err(format!("03_5 says more than 38 %, but B's run shows {}", share.pct(1)));
    }
    if b.dollars[0] != START || get("run.rounds")? != RUN_ROUNDS.to_string() {
        return Err("03_4 says the same player, but the run starts elsewhere".into());
    }
    let points = thinned(&b.dollars, THIN);
    let shares = points
        .iter()
        .map(|&(k, _)| {
            let k = k as usize;
            (k > 0).then(|| 100.0 * at_zero_before(&b.dollars, k) as f64 / k as f64)
        })
        .collect();
    Ok(Numbers {
        strip,
        bad_heads: format!("heads {bad_pct} %"),
        good_heads: format!("heads {good_pct} %"),
        full_badge: format!("{BADGE} \u{b7} {a_pct} \u{b7} {bad_pct} \u{b7} {good_pct} %"),
        hypo_tex: format!(
            r"\tfrac{{1}}{{3}} \cdot {bad_pct}\% + \tfrac{{2}}{{3}} \cdot {good_pct}\% {equals} {}\%",
            thirds.pct(1)
        ),
        third_line: format!("a third of the rounds would be {} %", get("thirds.pct")?),
        end_line: format!(
            "after {} rounds: {} dollars down",
            explainer_kit::thousands(RUN_ROUNDS as u64),
            explainer_kit::thousands(b.net().unsigned_abs())
        ),
        a,
        b,
        shares,
    })
}

/// The long window's axes and ticks, as 01_9 drew them. Detached.
fn window(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let (left, right, bottom, top) = (LONG.left(), LONG.right(), LONG.bottom(), LONG.top());
    let corner = polyline([v(left, top), v(left, bottom), v(right, bottom)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    Ok(vec![
        stage.arena_mut().add(corner),
        kit.text(stage, "0", 18.0, Palette::dim(), v(left, bottom - TICK_DY))?,
        kit.text(
            stage,
            &format!("{} {ROUNDS}", explainer_kit::thousands(RUN_ROUNDS as u64)),
            18.0,
            Palette::dim(),
            v(right - 0.9, bottom - TICK_DY),
        )?,
        kit.text(stage, DOLLARS, 18.0, Palette::dim(), v(left, top + TICK_DY))?,
        kit.text(stage, "0", 18.0, Palette::dim(), v(left - 0.5, LONG.p(0.0, 0.0)[1]))?,
        kit.text(
            stage,
            "\u{2212}1,000",
            18.0,
            Palette::dim(),
            v(left - 0.75, LONG.p(0.0, -1000.0)[1]),
        )?,
    ])
}

/// The badge with the chances, its right edge kept inside the frame.
fn full_badge(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<[Mob; 2]> {
    let tag = badge(kit, stage, text, true)?;
    let right = FRAME_X_RADIUS - ROOM;
    if stage.get_bounding_box_point(tag[0], RIGHT)[0] > right {
        stage.move_to(tag[0], v(right, BADGE_AT[1]), RIGHT);
        stage.move_to(tag[1], v(right - 0.15, BADGE_AT[1]), RIGHT);
    }
    keep_between(stage, tag[0], CARD_AT[0] - CARD_W / 2.0, right, "the coin badge")?;
    Ok(tag)
}

impl SceneConstruct for DoesBWin {
    fn name(&self) -> &str {
        "03_does_b_win"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let (card_left, card_right) = (CARD_AT[0] - CARD_W / 2.0, CARD_AT[0] + CARD_W / 2.0);

        // 03_1: card B as 02 left it; the counts 6 to 14 by remainder
        let line = say(&kit, stage, "03_1")?;
        let mut card_parts = card(&kit, stage, GAME_B, CARD_AT, CARD_W, CARD_H, palette::game_b())?.to_vec();
        card_parts.extend(coin(&kit, stage, BAD_AT, COIN_R, palette::bad(), BAD_FACE)?);
        card_parts.extend(coin(&kit, stage, GOOD_AT, COIN_R, palette::good(), GOOD_FACE)?);
        for (words, color, at) in [
            (BAD_RULE, palette::bad(), v(RULE_X, BAD_AT[1] + RULE_DY)),
            (n.bad_heads.as_str(), palette::bad(), v(RULE_X, BAD_AT[1] - HEADS_DY)),
            (GOOD_RULE, palette::good(), v(RULE_X, GOOD_AT[1] + RULE_DY)),
            (n.good_heads.as_str(), palette::good(), v(RULE_X, GOOD_AT[1] - HEADS_DY)),
        ] {
            let m = kit.text_left(
                stage,
                words,
                if words.starts_with("heads") { 22.0 } else { 24.0 },
                color,
                at,
            )?;
            keep_between(stage, m, card_left, card_right - ROOM, "card B's rules")?;
            card_parts.push(m);
        }
        let count_line = kit.text(
            stage,
            COUNT_LINE,
            24.0,
            Palette::hero(),
            v(CARD_AT[0], CARD_AT[1] + CARD_H / 2.0 - COUNT_LINE_DY),
        )?;
        let tag = full_badge(&kit, stage, &n.full_badge)?;
        let mut arriving = card_parts.clone();
        arriving.push(count_line);
        arriving.extend(tag);
        fade_in_all(stage, &arriving, 0.5)?;
        let x_at = |i: usize| STRIP_X0 + STRIP_PITCH * i as f64;
        let mut strip: Vec<Mob> = vec![
            kit.text(
                stage,
                DOLLARS,
                20.0,
                Palette::dim(),
                v(LEFT_X, STRIP_Y + STRIP_CAPTION_DY),
            )?,
            kit.text(
                stage,
                REMAINDER,
                18.0,
                Palette::dim(),
                v(LEFT_X, STRIP_Y - STRIP_REM_DY - STRIP_CAPTION_DY + 0.05),
            )?,
        ];
        fade_in_all(stage, &strip, 0.3)?;
        let mut leaving_ones: Vec<Mob> = Vec::new();
        for (i, &(count, r)) in n.strip.iter().enumerate() {
            let color = if r == 0 { palette::bad() } else { palette::good() };
            let cell = [
                kit.text(stage, &count.to_string(), 30.0, color, v(x_at(i), STRIP_Y))?,
                kit.text(
                    stage,
                    &r.to_string(),
                    20.0,
                    Palette::dim(),
                    v(x_at(i), STRIP_Y - STRIP_REM_DY),
                )?,
            ];
            keep_between(stage, cell[0], -FRAME_X_RADIUS + ROOM, card_left - ROOM, "the strip")?;
            fade_in_all(stage, &cell, 0.15)?;
            if r != 0 {
                leaving_ones.push(cell[0]);
            }
            strip.extend(cell);
        }
        line.wait_for(stage, AT_EVERY_THREE.1)?;
        flash_all(stage, &leaving_ones, 0.8)?;

        // 03_2: if the good coin got two rounds in three (a hypothetical)
        let line = say(&kit, stage, "03_2")?;
        let if_line = kit.text(stage, IF_LINE, 20.0, Palette::dim(), v(LEFT_X, IF_Y))?;
        let hypo = kit.tex(stage, &n.hypo_tex, 32.0, Palette::ink(), &[], v(LEFT_X, HYPO_Y))?;
        keep_between(
            stage,
            if_line,
            -FRAME_X_RADIUS + ROOM,
            card_left - ROOM,
            "the hypothetical's label",
        )?;
        keep_between(
            stage,
            hypo,
            -FRAME_X_RADIUS + ROOM,
            card_left - ROOM,
            "the hypothetical",
        )?;
        fade_in_all(stage, &[if_line, hypo], 0.5)?;
        line.wait_for(stage, AT_WOULD.1)?;
        let would = kit.text(stage, WOULD_WIN, 26.0, palette::good(), v(LEFT_X, WOULD_Y))?;
        fade_in_all(stage, &[would], 0.4)?;

        // 03_3: win or lose in the long run? (a prediction)
        say(&kit, stage, "03_3")?;
        let q = think(&kit, stage, THINK_AT)?;

        // 03_4: it loses; the same player, 100,000 rounds of game B
        let line = say(&kit, stage, "03_4")?;
        let (l, r) = (
            stage.get_bounding_box_point(would, LEFT)[0],
            stage.get_bounding_box_point(would, RIGHT)[0],
        );
        let strike = polyline([v(l - 0.1, WOULD_Y), v(r + 0.1, WOULD_Y)])
            .style(line_style(palette::bad(), 4.0))
            .build();
        let strike = stage.arena_mut().add(strike);
        fade_out_all(stage, &[q], 0.2)?;
        fade_in_all(stage, &[strike], 0.3)?;
        together(stage, vec![fade_to(if_line, FAINT, 0.4)?, fade_to(hypo, FAINT, 0.4)?])?;
        line.wait_for(stage, AT_PLAYER.1)?;
        let mut leaving = strip.clone();
        leaving.extend([if_line, hypo, would, strike]);
        fade_out_all(stage, &leaving, 0.4)?;
        shrink_all(
            stage,
            &card_parts,
            CARD_SHRINK,
            CARD_AT,
            v(CARD_SLOT[0] - CARD_AT[0], CARD_SLOT[1] - CARD_AT[1]),
            0.9,
        )?;
        together(stage, vec![fly(count_line, COUNT_SLOT, 0.6)?])?;
        let mut picture = window(&kit, stage)?;
        let a_points = thinned(&n.a.dollars, THIN);
        let a_line = polyline(a_points.iter().map(|&(k, d)| LONG.p(k, d)).collect::<Vec<_>>())
            .style(
                Style::default()
                    .fill(palette::game_a(), 0.0)
                    .stroke(palette::game_a(), 2.0, FAINT),
            )
            .build();
        let a_line = stage.arena_mut().add(a_line);
        let a_end = *n.a.dollars.last().expect("a run has rounds") as f64;
        let a_label = kit.text(
            stage,
            A_END,
            20.0,
            palette::game_a(),
            v(LONG.right() + END_LABEL_DX, LONG.p(0.0, a_end)[1]),
        )?;
        picture.extend([a_line, a_label]);
        let start_at = LONG.p(0.0, START as f64);
        let player = token(stage, start_at);
        let mut arriving = picture.clone();
        arriving.push(player);
        fade_in_all(stage, &arriving, 0.5)?;
        together(stage, vec![fade_to(a_label, FAINT, 0.1)?])?;
        line.wait_for(stage, AT_RUN.1)?;
        let points = thinned(&n.b.dollars, THIN);
        let (b_line, b_anchor, b_progress) = growing_run(stage, &LONG, &points, palette::game_b(), Some(player))?;
        fade_in_all(stage, &[b_anchor, b_line], 0.1)?;
        draw_run(stage, b_progress, RUN_S)?;
        stage.clear_updaters(b_anchor, true);
        let b_end = *n.b.dollars.last().expect("a run has rounds") as f64;
        let b_label = kit.text(
            stage,
            B_END,
            20.0,
            palette::game_b(),
            v(LONG.right() + END_LABEL_DX, LONG.p(0.0, b_end)[1]),
        )?;
        let end = kit.text(stage, &n.end_line, 24.0, palette::game_b(), END_AT)?;
        keep_between(stage, end, LONG.left(), FRAME_X_RADIUS - ROOM, "the end readout")?;
        fade_in_all(stage, &[b_label, end], 0.5)?;
        picture.extend([b_line, b_label, end]);
        line.wait_for(stage, AT_SLOWER.1)?;
        flash_all(stage, &[a_label, b_label], 0.8)?;

        // 03_5: the run again from its start, and the share of its rounds at
        // a multiple of three over exactly the rounds drawn so far
        let line = say(&kit, stage, "03_5")?;
        // Dim only the stroke. Family opacity would fill this open path.
        let dim_run = b_line
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.5),
                ..AnimateArgs::default()
            })?
            .set_stroke(None, None, Some(FAINT))?;
        together(stage, vec![dim_run, fly(player, start_at, 0.5)?])?;
        let at_zero_words = kit.text_left(stage, AT_ZERO_LINE, 22.0, Palette::ink(), v(PANEL_X, AT_ZERO_Y))?;
        let third = kit.text_left(stage, &n.third_line, 20.0, Palette::dim(), v(PANEL_X, THIRD_Y))?;
        let percent = kit.text_left(
            stage,
            PERCENT,
            22.0,
            palette::game_b(),
            v(VALUE_X + 0.62 + PERCENT_DX, AT_ZERO_Y),
        )?;
        keep_between(stage, at_zero_words, PANEL_X - 0.01, VALUE_X - 0.1, "the share's label")?;
        keep_between(stage, third, PANEL_X - 0.01, FRAME_X_RADIUS - ROOM, "a third's line")?;
        keep_between(
            stage,
            percent,
            VALUE_X,
            FRAME_X_RADIUS - ROOM,
            "the share's percent sign",
        )?;
        let (replay_line, replay_anchor, replay) = growing_run(stage, &LONG, &points, palette::game_b(), Some(player))?;
        // round 0 has no rounds behind it, so no share: the label, a third's
        // line and the path's start come in without a number
        fade_in_all(stage, &[at_zero_words, third, replay_anchor, replay_line], 0.4)?;
        // the first drawn point past round 0; from here on the path and the
        // share are one prefix, and the tracker only rises
        let first = first_drawn(points.len());
        stage.set_tracker_value(replay, first)?;
        if shown_share(&n.shares, stage.tracker_value(replay).unwrap_or(0.0)).is_none() {
            return Err(fail("03_5's share would come in before any round is drawn".into()));
        }
        let shares = n.shares.clone();
        let readout = kit.live_value(
            stage,
            move |s| {
                shown_share(&shares, s.tracker_value(replay).unwrap_or(0.0))
                    .expect("the share comes in only once a round is drawn")
            },
            1,
            false,
            22.0,
            palette::game_b(),
            v(VALUE_X, AT_ZERO_Y),
            LEFT,
        )?;
        stage.add_to_scene(percent)?;
        draw_run(stage, replay, REPLAY_S * (1.0 - first))?;
        line.wait_for(stage, AT_MORE.1)?;
        flash_all(stage, &[at_zero_words, percent], 0.8)?;
        picture.push(replay_line);

        // 03_6: those extra rounds sink B; the picture leaves a thumbnail
        let line = say(&kit, stage, "03_6")?;
        line.wait_for(stage, AT_SINK.1)?;
        flash_all(stage, &[count_line, at_zero_words, percent], 0.8)?;
        kit.hold(stage)?;
        stage.clear_updaters(replay_anchor, true);
        fade_out_all(stage, &[at_zero_words, percent, third, readout], 0.4)?;
        let shift = v(THUMB_AT[0] - LONG.center[0], THUMB_AT[1] - LONG.center[1]);
        let mut moves = Vec::with_capacity(picture.len() + 1);
        for &m in &picture {
            moves.push(
                m.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(1.0),
                        ..AnimateArgs::default()
                    })?
                    .scale_about(THUMB_FACTOR, Some(LONG.center), None)?
                    .shift(shift)?,
            );
        }
        moves.push(fade_to(player, 0.0, 1.0)?);
        together(stage, moves)?;
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
    use fmn::core::constants::FRAME_Y_RADIUS;
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{THINK_S, TOKEN_R};
    use super::*;

    /// The chapter's words are true of its numbers: two counts in three
    /// leave something over; the hypothetical is ⅓ · 9.5 % + ⅔ · 74.5 %, about
    /// 52.8 % and not exactly; B's run ends 548 dollars down, less far than
    /// A's 1,062; and it began 38.4 % of its rounds at a multiple of three.
    #[test]
    fn does_b_win_numbers_are_the_words() {
        let n = numbers().unwrap();
        let reds: Vec<i64> = n.strip.iter().filter(|&&(_, r)| r == 0).map(|&(c, _)| c).collect();
        assert_eq!(reds, [6, 9, 12]);
        assert_eq!(
            n.hypo_tex,
            r"\tfrac{1}{3} \cdot 9.5\% + \tfrac{2}{3} \cdot 74.5\% \approx 52.8\%"
        );
        assert_eq!(n.third_line, "a third of the rounds would be 33.3 %");
        assert_eq!((n.b.net(), n.a.net()), (-548, -1062));
        assert_eq!(n.end_line, "after 100,000 rounds: 548 dollars down");
        assert_eq!(n.full_badge, "published coins \u{b7} 49.5 \u{b7} 9.5 \u{b7} 74.5 %");
        assert_eq!(
            (n.bad_heads.as_str(), n.good_heads.as_str()),
            ("heads 9.5 %", "heads 74.5 %")
        );
    }

    /// The readout is the run's own prefix. At every tracker value the
    /// replay passes through, it shows the share of exactly the rounds the
    /// path shows, never later ones. At round 0 there is no share (the first
    /// version showed the first 250 rounds' share there, ahead of the path).
    /// It comes in at the first drawn point, 250 rounds. At the last point
    /// it is the run's whole count (38,371 of 100,000, shown 38.4).
    #[test]
    fn the_readout_follows_the_drawn_rounds() {
        let n = numbers().unwrap();
        let points = thinned(&n.b.dollars, THIN);
        let len = points.len();
        assert_eq!(len, n.shares.len());
        // round 0: no rounds drawn, no share, for every tracker value that
        // still shows only the start
        assert_eq!(points[0].0, 0.0);
        assert_eq!(n.shares[0], None);
        assert_eq!(shown_share(&n.shares, 0.0), None);
        assert_eq!(shown_share(&n.shares, 0.499 / (len - 1) as f64), None);
        // the readout comes in at the first drawn point, with its own share
        let first = first_drawn(len);
        assert_eq!(drawn_index(len, first), 1);
        assert_eq!(points[1].0, THIN as f64);
        assert!(shown_share(&n.shares, first).is_some());
        assert_eq!(shown_share(&n.shares, first), n.shares[1]);
        // a hand count of the rounds so far at a multiple of three
        let so_far: Vec<usize> = std::iter::once(0)
            .chain(n.b.dollars[..RUN_ROUNDS].iter().scan(0, |count, d| {
                *count += usize::from(d.rem_euclid(3) == 0);
                Some(*count)
            }))
            .collect();
        // every tracker value: the path's point (growing_run's rounding)
        // and the readout's share are of the same rounds
        for step in 0..=4000 {
            let f = f64::from(step) / 4000.0;
            let k = drawn_index(len, f);
            assert_eq!(k, ((len - 1) as f64 * f).round() as usize);
            let rounds = points[k].0 as usize;
            match shown_share(&n.shares, f) {
                None => assert_eq!(rounds, 0, "no share at tracker {f}"),
                Some(s) => assert!(
                    (s - 100.0 * so_far[rounds] as f64 / rounds as f64).abs() < 1e-12,
                    "tracker {f}, {rounds} rounds"
                ),
            }
        }
        assert_eq!(drawn_index(len, 1.0), len - 1);
        let last = shown_share(&n.shares, 1.0).expect("a share at the end");
        assert_eq!(at_zero_before(&n.b.dollars, RUN_ROUNDS), n.b.at_zero);
        assert_eq!(so_far[RUN_ROUNDS], n.b.at_zero);
        assert!((last - 100.0 * n.b.at_zero as f64 / RUN_ROUNDS as f64).abs() < 1e-12);
        assert_eq!(
            format!("{last:.1}"),
            facts::facts().unwrap().get("run.b.at0.pct").unwrap()
        );
        assert_eq!(n.b.at_zero, 38_371);
        // past round 0, every share is above a third (the run is never near
        // the hypothetical's equal thirds)
        assert!(
            n.shares[1..].iter().all(|s| s.is_some_and(|s| s > 100.0 / 3.0 + 3.0)),
            "{:?}",
            &n.shares[..4]
        );
    }

    /// Negative control: the hypothetical's equal thirds are not what the
    /// run did. Weighting the coins by the run's own share instead of a
    /// third gives a win rate below a half.
    #[test]
    fn the_runs_own_share_turns_the_hypothetical_into_a_loss() {
        let n = numbers().unwrap();
        let [_, bad, good] = facts::coins(&PUBLISHED);
        let s = n.b.share_at_zero();
        let by_the_run = s * bad + (Ratio::ONE - s) * good;
        assert!(by_the_run < Ratio::new(1, 2), "{}", by_the_run.pct(2));
        let by_thirds = Ratio::new(1, 3) * bad + Ratio::new(2, 3) * good;
        assert!(by_thirds > Ratio::new(1, 2));
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the strip stays left of card B; the
    /// think ring clears the frame's foot; the shrunk card and its line clear
    /// the long window and the badge; the drawn runs (and the token riding
    /// B's) stay below the readouts and inside the window; the end labels
    /// fit; the thumbnail stays inside the frame below the shrunk card.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let n = numbers().unwrap();
        let card_left = CARD_AT[0] - CARD_W / 2.0;
        let strip_right = STRIP_X0 + STRIP_PITCH * (n.strip.len() - 1) as f64 + 0.3;
        assert!(
            STRIP_X0 - 0.3 > -FRAME_X_RADIUS && strip_right < card_left - ROOM,
            "{strip_right}"
        );
        const {
            assert!(THINK_AT[1] - 0.62 - 0.3 - 0.15 > -FRAME_Y_RADIUS);
        }
        const {
            assert!(WOULD_Y - 0.2 > THINK_AT[1] + 0.62);
        }
        let (slot_w, slot_h) = (CARD_W * CARD_SHRINK, CARD_H * CARD_SHRINK);
        assert!(CARD_SLOT[1] - slot_h / 2.0 > LONG.top() + TICK_DY + 0.15);
        assert!(CARD_SLOT[1] + slot_h / 2.0 < FRAME_Y_RADIUS && CARD_SLOT[0] - slot_w / 2.0 > -FRAME_X_RADIUS);
        assert!(
            COUNT_SLOT[0] - 1.25 > CARD_SLOT[0] + slot_w / 2.0,
            "the count line runs into the shrunk card"
        );
        const {
            assert!(
                COUNT_SLOT[0] + 1.25 < BADGE_AT[0] - 2.2,
                "the count line runs into the badge"
            );
        }
        assert!(END_AT[1] + 0.2 < CARD_SLOT[1] - slot_h / 2.0);
        let readout_floor = THIRD_Y - 0.13;
        for run in [&n.a, &n.b] {
            for (k, d) in thinned(&run.dollars, THIN) {
                let [x, y, _] = LONG.p(k, d);
                assert!(y > LONG.bottom() && y < LONG.top(), "round {k} leaves the window");
                if x > PANEL_X - TOKEN_R - 0.1 {
                    assert!(
                        y + TOKEN_R < readout_floor,
                        "round {k} at {d} dollars reaches the readouts ({y:.2})"
                    );
                }
            }
        }
        assert!(LONG.right() + END_LABEL_DX + 0.2 < FRAME_X_RADIUS);
        let (a_y, b_y) = (
            LONG.p(0.0, *n.a.dollars.last().unwrap() as f64)[1],
            LONG.p(0.0, *n.b.dollars.last().unwrap() as f64)[1],
        );
        assert!(b_y - a_y > 0.4, "the end labels overlap");
        let (half_w, half_h) = (LONG.width * THUMB_FACTOR / 2.0, LONG.height * THUMB_FACTOR / 2.0);
        assert!(THUMB_AT[0] - half_w > -FRAME_X_RADIUS && THUMB_AT[1] + half_h < CARD_SLOT[1] - slot_h / 2.0);
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` guard meets the true text
    /// widths): every line of chapter 03 is said once, in order, and the
    /// prediction (03_3) gets its silent think time before the answer.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = DoesBWin { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("03_does_b_win: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=6).map(|k| format!("03_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let asked_end = at("03_3") + narrator.length("03_3").unwrap();
        assert!(
            at("03_4") - asked_end >= THINK_S,
            "{} s to think",
            at("03_4") - asked_end
        );
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end > at("03_6"), "the chapter ends after its last line: {end}");
    }
}
