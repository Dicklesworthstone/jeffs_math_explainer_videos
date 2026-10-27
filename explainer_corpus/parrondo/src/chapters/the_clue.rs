//! Chapter 05, the clue in the counts. The picture as 04 left it, in a
//! lower window: A's and B's runs faint, the mix's in gold, the token at its
//! end. Above it a scale of the share of rounds that began at a multiple of
//! three, 32 % to 40 %, with a third (33.3 %) marked dim. The coins did not
//! change (the badge's edge lights up); in game B the player stood there
//! 38.4 % of the time (B's mark, violet, its label briefly framed); in the
//! mix, more often or less? The mix's readout stays empty through the
//! prediction (05_1). Less: the mix's run replays from its start, and its
//! readout and gold mark follow exactly the rounds drawn so far, ending at
//! 34.5 %, much closer to a third (05_2). The puzzle is real: the bars of
//! 04_4 return, the dashed "halfway" guess is struck through, and the mix's
//! bar grows to +1.57 cents a round, with the exact averages in a
//! "precisely" card; then the title (05_3). The catch is in game B's rule:
//! card B returns with "count your dollars" outlined, the outline pulsing,
//! and the player beside it (05_4).
//!
//! Words are pointed at with a separate frame that comes and goes (04's
//! `point_at`), never by animating the typeset text, which has shown
//! doubled glyphs. The readout and the mark are one prefix of one run, as
//! in 03_5: the same tracker picks the drawn point and the share; at round
//! 0 there is no share, so neither comes in before the first drawn point.
//! Every number comes from `crate::model` and `crate::facts`; the scene
//! refuses to draw if the words would stop being true of them (B above
//! 38 %, the mix below B and much closer to a third, the mix's coins A's
//! and B's averaged, both games losing and the mix winning just over one
//! and a half cents a round).

use explainer_kit::props::precisely;
use explainer_kit::{Frame2, Kit, Palette, fade_all, rect, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    card, coin, draw_run, fade_in_all, fade_out_all, fade_to, fail, fly, growing_run, keep_between, line_style, now,
    pause, read_s, say, screen, stand, think, thinned, together, token, wait_until,
};
use super::glyphs::{SUBTITLE, TITLE};
use super::looks_hopeless::{
    Averages, EDGE, FAINT, PLAYER_LABEL_AT, POINT_PAD, ROOM, ROW_YS, THIN, TOKEN_AT, ZERO_X, averages, badge_text,
    bars, end_label, faint_run, frame_around, full_badge, grow, point_at, window,
};
use crate::facts;
use crate::model::{self, MODULUS, PUBLISHED, Play, Ratio};
use crate::palette;

pub struct TheClue {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const PLAYER: &str = "player";
const CAPTION: &str = "rounds at a multiple of 3";
const MIX_READOUT: &str = "the mix:";
const PERCENT: &str = "%";
const THIRD_WORDS: &str = "a third";
const B_WORDS: &str = "game B";
const A_END: &str = "A";
const B_END: &str = "B";
const MIX_END: &str = "mix";
const GAME_B: &str = "Game B";
const COUNT_LINE: &str = "count your dollars";
const BAD_FACE: &str = "bad";
const GOOD_FACE: &str = "good";
const BAD_RULE: &str = "remainder 0 \u{2192} bad coin";
const GOOD_RULE: &str = "remainder 1 or 2 \u{2192} good coin";

/// Where each picture lands: (line, the phrase it waits for).
const AT_COINS: (&str, &str) = ("05_1", "Nothing about the coins changed");
const AT_IN_B: (&str, &str) = ("05_1", "In game B");
const AT_IN_MIX: (&str, &str) = ("05_1", "In the mix, more often or less");
const AT_STOOD: (&str, &str) = ("05_2", "In the mix, the player stood");
const AT_SHARE: (&str, &str) = ("05_2", "thirty-four and a half percent");
const AT_THIRD: (&str, &str) = ("05_2", "much closer to a third");
const AT_EACH_LOSE: (&str, &str) = ("05_3", "Two games that each lose");
const AT_FAIR: (&str, &str) = ("05_3", "mixed by a fair coin");
const AT_WINS: (&str, &str) = ("05_3", "make a game that wins");
const AT_READS: (&str, &str) = ("05_4", "the part that reads your money");
const AT_STANDS: (&str, &str) = ("05_4", "where the player stands");
#[cfg(test)]
const CUES: [(&str, &str); 11] = [
    AT_COINS,
    AT_IN_B,
    AT_IN_MIX,
    AT_STOOD,
    AT_SHARE,
    AT_THIRD,
    AT_EACH_LOSE,
    AT_FAIR,
    AT_WINS,
    AT_READS,
    AT_STANDS,
];

// ------------------------------------------------------------------ layout

/// 05_1 and 05_2: 04's long window, lower and shorter, under the scale.
const CLUE: Frame2 = Frame2 {
    center: [0.8, -1.6, 0.0],
    width: 10.0,
    height: 3.6,
    x: (0.0, 100_000.0),
    y: (-1200.0, 1900.0),
};
/// The scale of shares at a multiple of three, in percent, one unit per
/// percentage point.
const SCALE: (i32, i32) = (32, 40);
const SCALE_X0: f64 = -3.4;
const SCALE_U: f64 = 1.0;
const SCALE_Y: f64 = 1.25;
const TICK_H: f64 = 0.07;
const END_DY: f64 = 0.32;
/// A share's mark crosses the scale this far each way; its label stands
/// this far above (B) or below (a third).
const MARK_H: f64 = 0.2;
const MARK_LABEL_DY: f64 = 0.42;
/// The caption and the mix's readout, top left.
const CAPTION_AT: Vec3 = [-3.4, 2.75, 0.0];
const READOUT_Y: f64 = 2.2;
const VALUE_X: f64 = -2.0;
const PERCENT_DX: f64 = 0.7;
/// 05_1's prediction, right of the empty readout.
const THINK_AT: Vec3 = [0.8, 2.45, 0.0];
#[cfg(test)]
const THINK_R: f64 = 0.62;
#[cfg(test)]
const NOTE_DROP: f64 = 0.92;
const REPLAY_S: f64 = 4.5;
/// 05_3: the strike through the halfway guess overhangs it this far.
const STRIKE_PAD: f64 = 0.12;
/// The title card, and how long it stands alone before 05_4.
const TITLE_AT: Vec3 = [0.0, 0.5, 0.0];
const SUBTITLE_AT: Vec3 = [0.0, -0.4, 0.0];
const TITLE_HOLD: f64 = 2.5;
/// 05_4: card B, laid out as in 02 and 03, near the centre; the player
/// stays in their place on the left third.
const CARD_AT: Vec3 = [0.5, 0.2, 0.0];
const CARD_W: f64 = 6.2;
const CARD_H: f64 = 4.0;
const COUNT_LINE_DY: f64 = 0.8;
const COIN_R: f64 = 0.42;
const COIN_DX: f64 = -2.3;
const BAD_DY: f64 = 0.4;
const GOOD_DY: f64 = -1.2;
const RULE_DX: f64 = -1.6;
const RULE_DY: f64 = 0.17;
const HEADS_DY: f64 = 0.25;
const OUTLINE_PAD: f64 = 0.12;
/// The outline's stroke at the height of its pulse (it is drawn at 2.5).
const OUTLINE_PULSE_W: f64 = 5.0;

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    full_badge: String,
    bad_heads: String,
    good_heads: String,
    averages: Averages,
    a: model::Run,
    b: model::Run,
    mix: model::Run,
    /// A third and B's share at a multiple of three, in percent, and their
    /// marks' labels.
    third: f64,
    b_share: f64,
    third_label: String,
    b_label: String,
    /// At each drawn point of the mix's run (rounds 0, 250, …, 100,000), the
    /// share in percent of the rounds so far that began at a multiple of
    /// three; none at round 0, which has no rounds behind it.
    shares: Vec<Option<f64>>,
    k02: String,
}

/// The rounds of a run, among its first `rounds`, that began at a multiple
/// of three (as in 03).
fn at_zero_before(dollars: &[i64], rounds: usize) -> usize {
    dollars[..rounds].iter().filter(|d| d.rem_euclid(MODULUS) == 0).count()
}

/// The point a run drawn by `growing_run` shows at tracker value `f`, out of
/// `len` points (the same rounding as `growing_run`).
fn drawn_index(len: usize, f: f64) -> usize {
    ((len - 1) as f64 * f.clamp(0.0, 1.0)).round() as usize
}

/// The share the readout and the mark show at tracker value `f`: the one
/// for exactly the rounds drawn so far, and none while only round 0 is.
fn shown_share(shares: &[Option<f64>], f: f64) -> Option<f64> {
    shares[drawn_index(shares.len(), f)]
}

/// The tracker value at which a run drawn by `growing_run` over `len`
/// points shows its first point past round 0.
fn first_drawn(len: usize) -> f64 {
    1.0 / (len - 1) as f64
}

/// A share's place on the scale.
fn scale_x(pct: f64) -> f64 {
    SCALE_X0 + (pct - f64::from(SCALE.0)) * SCALE_U
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let a = model::player(Play::A);
    let b = model::player(Play::B);
    let mix = model::player(Play::Mix);
    let third = Ratio::new(1, 3);
    let (b_share, mix_share) = (b.share_at_zero(), mix.share_at_zero());
    // 05_1: nothing about the coins changed, the mix's are A's and B's
    // averaged at every remainder; in game B, more than 38 % at a multiple
    // of three
    let (game_a, game_b, the_mix) = (PUBLISHED.game_a(), PUBLISHED.game_b(), PUBLISHED.mix());
    if (0..3).any(|i| the_mix.chance(i) != (game_a.chance(i) + game_b.chance(i)) / Ratio::int(2)) {
        return Err("05_1 says nothing about the coins changed, but the mix's coins are not A's and B's".into());
    }
    if b_share <= Ratio::new(38, 100) || b_share.pct(1) != get("run.b.at0.pct")? {
        return Err(format!(
            "05_1 says more than 38 %, but B's run shows {}",
            b_share.pct(1)
        ));
    }
    // 05_2: less in the mix, about 34.5 %, much closer to a third
    if mix_share >= b_share || mix_share.pct(1) != "34.5" || mix_share.pct(1) != get("run.mix.at0.pct")? {
        return Err(format!(
            "05_2 says less, about 34.5 %, but the mix's run shows {}",
            mix_share.pct(1)
        ));
    }
    if mix_share <= third || Ratio::int(3) * (mix_share - third) >= b_share - mix_share {
        return Err("05_2 says much closer to a third than to game B".into());
    }
    if mix.at_zero.to_string() != get("run.mix.at0.rounds")? {
        return Err("05_2's count disagrees with the facts".into());
    }
    // 05_3: two games that each lose, mixed into one that wins just over
    // one and a half cents a round
    let averages = averages(&f)?;
    if averages.mix <= Ratio::new(15, 1000) || averages.mix >= Ratio::new(16, 1000) {
        return Err("05_3 says just over one and a half cents a round".into());
    }
    for (key, x) in [
        ("pub.a.mean", averages.a),
        ("pub.b.mean", averages.b),
        ("pub.mix.mean", averages.mix),
    ] {
        if x.to_string() != get(key)? {
            return Err(format!("K02 disagrees with the fact {key}"));
        }
    }
    let shares: Vec<Option<f64>> = thinned(&mix.dollars, THIN)
        .iter()
        .map(|&(k, _)| {
            let k = k as usize;
            (k > 0).then(|| 100.0 * at_zero_before(&mix.dollars, k) as f64 / k as f64)
        })
        .collect();
    let (lo, hi) = (f64::from(SCALE.0), f64::from(SCALE.1));
    if shares.iter().flatten().any(|&s| s <= lo || s >= hi) {
        return Err("a share of the mix's run would leave 05's scale".into());
    }
    let [_, bad, good] = facts::coins(&PUBLISHED);
    Ok(Numbers {
        full_badge: badge_text(&f)?,
        bad_heads: format!("heads {} %", bad.exact_pct()?),
        good_heads: format!("heads {} %", good.exact_pct()?),
        third: third.to_f64() * 100.0,
        b_share: b_share.to_f64() * 100.0,
        third_label: format!("{THIRD_WORDS} {} {PERCENT}", get("thirds.pct")?),
        b_label: format!("{B_WORDS} {} {PERCENT}", b_share.pct(1)),
        k02: format!(
            "exact long-run averages, dollars a round: game A {}, game B {}, the mix +{}; a run's average settles \
             on these with probability one (Ethier & Lee, 2009)",
            screen(&averages.a.to_string()),
            screen(&averages.b.to_string()),
            averages.mix
        ),
        averages,
        a,
        b,
        mix,
        shares,
    })
}

/// The scale: its line, a tick at every percentage point, its end values
/// and its caption. Detached.
fn scale(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let (x0, x1) = (scale_x(f64::from(SCALE.0)), scale_x(f64::from(SCALE.1)));
    let line = polyline([v(x0, SCALE_Y), v(x1, SCALE_Y)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut parts = vec![stage.arena_mut().add(line)];
    for p in SCALE.0..=SCALE.1 {
        let x = scale_x(f64::from(p));
        let tick = polyline([v(x, SCALE_Y - TICK_H), v(x, SCALE_Y + TICK_H)])
            .style(line_style(Palette::dim(), 1.5))
            .build();
        parts.push(stage.arena_mut().add(tick));
    }
    for p in [SCALE.0, SCALE.1] {
        let x = scale_x(f64::from(p));
        parts.push(kit.text(
            stage,
            &format!("{p} {PERCENT}"),
            18.0,
            Palette::dim(),
            v(x, SCALE_Y + END_DY),
        )?);
    }
    let caption = kit.text_left(stage, CAPTION, 22.0, Palette::dim(), CAPTION_AT)?;
    keep_between(
        stage,
        caption,
        CAPTION_AT[0] - 0.01,
        THINK_AT[0] - 0.62 - ROOM,
        "the scale's caption",
    )?;
    parts.push(caption);
    Ok(parts)
}

/// A share's mark on the scale: a tick across it in `color`, and its label
/// above (`up`) or below. Detached.
fn mark(kit: &Kit, stage: &mut Stage<'_>, pct: f64, color: Srgb, words: &str, up: bool) -> fmn::Result<[Mob; 2]> {
    let x = scale_x(pct);
    let tick = polyline([v(x, SCALE_Y - MARK_H), v(x, SCALE_Y + MARK_H)])
        .style(line_style(color, 4.0))
        .build();
    let tick = stage.arena_mut().add(tick);
    let y = if up {
        SCALE_Y + MARK_LABEL_DY
    } else {
        SCALE_Y - MARK_LABEL_DY
    };
    let label = kit.text(stage, words, 20.0, color, v(x, y))?;
    keep_between(
        stage,
        label,
        -FRAME_X_RADIUS + EDGE,
        FRAME_X_RADIUS - EDGE,
        "a share's label",
    )?;
    Ok([tick, label])
}

/// Card B as 02 and 03 built it (frame and title, the two coins, their
/// rules and chances), about [`CARD_AT`], and its "count your dollars".
/// Detached.
fn card_b(kit: &Kit, stage: &mut Stage<'_>, n: &Numbers) -> fmn::Result<(Vec<Mob>, Mob)> {
    let (left, right) = (CARD_AT[0] - CARD_W / 2.0, CARD_AT[0] + CARD_W / 2.0);
    let bad_at = v(CARD_AT[0] + COIN_DX, CARD_AT[1] + BAD_DY);
    let good_at = v(CARD_AT[0] + COIN_DX, CARD_AT[1] + GOOD_DY);
    let rule_x = CARD_AT[0] + RULE_DX;
    let mut parts = card(kit, stage, GAME_B, CARD_AT, CARD_W, CARD_H, palette::game_b())?.to_vec();
    parts.extend(coin(kit, stage, bad_at, COIN_R, palette::bad(), BAD_FACE)?);
    parts.extend(coin(kit, stage, good_at, COIN_R, palette::good(), GOOD_FACE)?);
    for (words, size, color, at) in [
        (BAD_RULE, 24.0, palette::bad(), v(rule_x, bad_at[1] + RULE_DY)),
        (
            n.bad_heads.as_str(),
            22.0,
            palette::bad(),
            v(rule_x, bad_at[1] - HEADS_DY),
        ),
        (GOOD_RULE, 24.0, palette::good(), v(rule_x, good_at[1] + RULE_DY)),
        (
            n.good_heads.as_str(),
            22.0,
            palette::good(),
            v(rule_x, good_at[1] - HEADS_DY),
        ),
    ] {
        let m = kit.text_left(stage, words, size, color, at)?;
        keep_between(stage, m, left, right - ROOM, "card B's rules")?;
        parts.push(m);
    }
    let count_line = kit.text(
        stage,
        COUNT_LINE,
        24.0,
        Palette::hero(),
        v(CARD_AT[0], CARD_AT[1] + CARD_H / 2.0 - COUNT_LINE_DY),
    )?;
    Ok((parts, count_line))
}

impl SceneConstruct for TheClue {
    fn name(&self) -> &str {
        "05_the_clue"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;

        // 05_1: the runs as 04 left them, lower; the scale with a third
        // marked; the coins did not change; B's share; the mix's, empty
        let line = say(&kit, stage, "05_1")?;
        let tag = full_badge(&kit, stage, &n.full_badge, THINK_AT[0] + 0.62 + ROOM)?;
        let mut picture = window(&kit, stage, &CLUE)?;
        let points = thinned(&n.mix.dollars, THIN);
        let mix_line = polyline(points.iter().map(|&(k, d)| CLUE.p(k, d)).collect::<Vec<_>>())
            .style(line_style(palette::mix(), 3.0))
            .build();
        let mix_line = stage.arena_mut().add(mix_line);
        let ends = [
            end_label(&kit, stage, &CLUE, &n.a, A_END, palette::game_a())?,
            end_label(&kit, stage, &CLUE, &n.b, B_END, palette::game_b())?,
            end_label(&kit, stage, &CLUE, &n.mix, MIX_END, palette::mix())?,
        ];
        picture.extend([
            faint_run(stage, &CLUE, &n.a, palette::game_a()),
            faint_run(stage, &CLUE, &n.b, palette::game_b()),
            mix_line,
        ]);
        picture.extend(ends);
        let last = *points.last().expect("a run has rounds");
        let player = token(stage, CLUE.p(last.0, last.1));
        let ruler = scale(&kit, stage)?;
        let third = mark(&kit, stage, n.third, Palette::dim(), &n.third_label, false)?;
        let mut arriving = picture.clone();
        arriving.extend(&ruler);
        arriving.extend(third);
        arriving.push(player);
        arriving.extend(tag);
        fade_in_all(stage, &arriving, 0.5)?;
        together(
            stage,
            vec![fade_to(ends[0], FAINT, 0.3)?, fade_to(ends[1], FAINT, 0.3)?],
        )?;
        line.wait_for(stage, AT_COINS.1)?;
        // the badge's own edge, in ink: a hero outline is 07's sign of the
        // round-number coins, and here nothing changed
        let pointer = frame_around(stage, &tag[..1], 0.0, Palette::ink())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_IN_B.1)?;
        let b_mark = mark(&kit, stage, n.b_share, palette::game_b(), &n.b_label, true)?;
        fade_in_all(stage, &b_mark, 0.4)?;
        let pointer = frame_around(stage, &b_mark[1..], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &b_mark[..1], 0.6)?;
        line.wait_for(stage, AT_IN_MIX.1)?;
        let mix_words = kit.text_left(stage, MIX_READOUT, 24.0, palette::mix(), v(CAPTION_AT[0], READOUT_Y))?;
        keep_between(
            stage,
            mix_words,
            CAPTION_AT[0] - 0.01,
            VALUE_X - 0.1,
            "the mix's readout label",
        )?;
        // the mix's readout is empty until the answer
        fade_in_all(stage, &[mix_words], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 05_2: less; the mix's run again from its start, and its share at a
        // multiple of three over exactly the rounds drawn so far
        let line = say(&kit, stage, "05_2")?;
        fade_out_all(stage, &[q], 0.2)?;
        line.wait_for(stage, AT_STOOD.1)?;
        // Dim only the stroke. Family opacity would fill this open path.
        let dim_run = mix_line
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.5),
                ..AnimateArgs::default()
            })?
            .set_stroke(None, None, Some(FAINT))?;
        let start = CLUE.p(0.0, points[0].1);
        together(stage, vec![dim_run, fly(player, start, 0.5)?])?;
        let percent = kit.text_left(stage, PERCENT, 24.0, palette::mix(), v(VALUE_X + PERCENT_DX, READOUT_Y))?;
        keep_between(stage, percent, VALUE_X, THINK_AT[0] - 0.62, "the share's percent sign")?;
        let (replay_line, replay_anchor, replay) = growing_run(stage, &CLUE, &points, palette::mix(), Some(player))?;
        fade_in_all(stage, &[replay_anchor, replay_line], 0.2)?;
        // the first drawn point past round 0; from here on the path, the
        // readout and the mark are one prefix, and the tracker only rises
        let first = first_drawn(points.len());
        stage.set_tracker_value(replay, first)?;
        let Some(first_share) = shown_share(&n.shares, stage.tracker_value(replay).unwrap_or(0.0)) else {
            return Err(fail("05_2's share would come in before any round is drawn".into()));
        };
        let shares = n.shares.clone();
        let readout = kit.live_value(
            stage,
            move |s| {
                shown_share(&shares, s.tracker_value(replay).unwrap_or(0.0))
                    .expect("the share comes in only once a round is drawn")
            },
            1,
            false,
            24.0,
            palette::mix(),
            v(VALUE_X, READOUT_Y),
            LEFT,
        )?;
        let x = scale_x(first_share);
        let tick = polyline([v(x, SCALE_Y - MARK_H), v(x, SCALE_Y + MARK_H)])
            .style(line_style(palette::mix(), 4.0))
            .build();
        let mix_mark = stage.arena_mut().add(tick);
        let mark_anchor = stage.arena_mut().add(VMobject::new());
        let shares = n.shares.clone();
        let mut shown = None;
        stage.add_updater(
            mark_anchor,
            move |s, _| {
                let k = drawn_index(shares.len(), s.tracker_value(replay).unwrap_or(0.0));
                if shown == Some(k) {
                    return;
                }
                if let Some(p) = shares[k] {
                    s.move_to(mix_mark, v(scale_x(p), SCALE_Y), ORIGIN);
                }
                shown = Some(k);
            },
            true,
        )?;
        for m in [mark_anchor, mix_mark, percent] {
            stage.add_to_scene(m)?;
        }
        draw_run(stage, replay, REPLAY_S * (1.0 - first))?;
        stage.clear_updaters(replay_anchor, true);
        stage.clear_updaters(mark_anchor, true);
        line.wait_for(stage, AT_SHARE.1)?;
        // one frame for the readout's row (the number stands between its
        // label and its percent sign); the gold mark pulses
        let pointer = frame_around(stage, &[mix_words, percent], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[mix_mark], 0.8)?;
        line.wait_for(stage, AT_THIRD.1)?;
        let pointer = frame_around(stage, &third[1..], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &third[..1], 0.8)?;

        // 05_3: the puzzle is real; the halfway guess is struck, the mix's
        // bar grows to its exact average; the "precisely" card; the title
        let line = say(&kit, stage, "05_3")?;
        let mut leaving = picture.clone();
        leaving.extend(&ruler);
        leaving.extend(third);
        leaving.extend(b_mark);
        leaving.extend([
            mix_words,
            percent,
            readout,
            mix_mark,
            mark_anchor,
            replay_line,
            replay_anchor,
            player,
        ]);
        fade_out_all(stage, &leaving, 0.5)?;
        let bars = bars(&kit, stage, &n.averages)?;
        let mut standing = bars.axis.clone();
        standing.extend(bars.games);
        standing.extend(&bars.halfway);
        standing.extend(&bars.labels[..3]);
        fade_in_all(stage, &standing, 0.5)?;
        line.wait_for(stage, AT_EACH_LOSE.1)?;
        // one frame for both losers' labels; the bars stay to scale
        let pointer = frame_around(stage, &bars.labels[..2], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_FAIR.1)?;
        let from = stage.get_bounding_box_point(bars.labels[2], LEFT)[0] - STRIKE_PAD;
        let strike = polyline([v(from, ROW_YS[2]), v(ZERO_X + STRIKE_PAD, ROW_YS[2])])
            .style(line_style(palette::bad(), 4.0))
            .build();
        let strike = stage.arena_mut().add(strike);
        kit.allow_crossing(strike);
        fade_in_all(stage, &[strike], 0.3)?;
        let mut dim = vec![fade_to(bars.labels[2], FAINT, 0.4)?];
        for &dash in &bars.halfway {
            dim.push(
                dash.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.4),
                        ..AnimateArgs::default()
                    })?
                    .set_stroke(None, None, Some(FAINT))?,
            );
        }
        together(stage, dim)?;
        line.wait_for(stage, AT_WINS.1)?;
        grow(stage, bars.mix, LEFT, 0.8, &[bars.labels[3]])?;
        let k02 = precisely(&kit, stage, &n.k02)?;
        fade_in_all(stage, &[k02], 0.5)?;
        let card_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&n.k02))?;
        let mut leaving = standing.clone();
        leaving.extend([strike, bars.mix, bars.labels[3], k02]);
        fade_out_all(stage, &leaving, 0.5)?;
        let title = kit.text(stage, TITLE, 56.0, Palette::ink(), TITLE_AT)?;
        let subtitle = kit.text(stage, SUBTITLE, 34.0, Palette::hero(), SUBTITLE_AT)?;
        for (m, what) in [(title, "the title"), (subtitle, "the subtitle")] {
            keep_between(stage, m, -FRAME_X_RADIUS + EDGE, FRAME_X_RADIUS - EDGE, what)?;
        }
        fade_in_all(stage, &[title, subtitle], 0.6)?;
        pause(stage, TITLE_HOLD)?;

        // 05_4: the catch is in game B's rule; card B again, "count your
        // dollars" outlined, the player in their place beside it
        let line = say(&kit, stage, "05_4")?;
        fade_out_all(stage, &[title, subtitle], 0.4)?;
        let (card_parts, count_line) = card_b(&kit, stage, &n)?;
        let mut arriving = card_parts.clone();
        arriving.push(count_line);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, AT_READS.1)?;
        let (l, r) = (
            stage.get_bounding_box_point(count_line, LEFT)[0],
            stage.get_bounding_box_point(count_line, RIGHT)[0],
        );
        let (b, t) = (
            stage.get_bounding_box_point(count_line, DOWN)[1],
            stage.get_bounding_box_point(count_line, UP)[1],
        );
        let outline = rect(
            v(l - OUTLINE_PAD, b - OUTLINE_PAD),
            r - l + 2.0 * OUTLINE_PAD,
            t - b + 2.0 * OUTLINE_PAD,
            line_style(Palette::hero(), 2.5),
        )?;
        let outline = stage.arena_mut().add(outline);
        fade_in_all(stage, &[outline], 0.3)?;
        // the outline itself thickens and settles; the words stay as typeset
        let pulse = outline
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                rate_func: Some(fmn::core::rate::there_and_back),
                ..AnimateArgs::default()
            })?
            .set_stroke(None, Some(OUTLINE_PULSE_W), None)?;
        together(stage, vec![pulse])?;
        line.wait_for(stage, AT_STANDS.1)?;
        let player = token(stage, TOKEN_AT);
        let label = kit.text(stage, PLAYER, 18.0, Palette::dim(), PLAYER_LABEL_AT)?;
        fade_in_all(stage, &[player, label], 0.5)?;
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

    use super::super::draw::{BADGE_AT, THINK_S, TOKEN_R};
    use super::super::looks_hopeless::END_LABEL_DX;
    use super::*;
    use crate::model::RUN_ROUNDS;

    /// The chapter's words are true of its numbers: B's mark reads 38.4 %
    /// and a third's 33.3 %; the mix's run began 34,520 of its 100,000
    /// rounds at a multiple of three (34.5 %); the card gives the exact
    /// averages; card B's chances are 02's.
    #[test]
    fn the_clue_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.b_label, "game B 38.4 %");
        assert_eq!(n.third_label, "a third 33.3 %");
        assert_eq!((n.b.at_zero, n.mix.at_zero), (38_371, 34_520));
        assert_eq!(n.mix.share_at_zero().pct(1), "34.5");
        assert_eq!(
            n.k02,
            "exact long-run averages, dollars a round: game A \u{2212}1/100, game B \u{2212}73443/8446300, the \
             mix +223/14200; a run's average settles on these with probability one (Ethier & Lee, 2009)"
        );
        assert_eq!(n.full_badge, "published coins \u{b7} 49.5 \u{b7} 9.5 \u{b7} 74.5 %");
        assert_eq!(
            (n.bad_heads.as_str(), n.good_heads.as_str()),
            ("heads 9.5 %", "heads 74.5 %")
        );
        assert_eq!(n.a.net(), -1062);
    }

    /// The readout and the mark are the run's own prefix. At every tracker
    /// value the replay passes through they show the share of exactly the
    /// rounds the path shows, never later ones; at round 0 there is none;
    /// they come in at the first drawn point, 250 rounds; at the last point
    /// the share is the run's whole count (34,520 of 100,000, shown 34.5).
    #[test]
    fn the_readout_follows_the_drawn_rounds() {
        let n = numbers().unwrap();
        let points = thinned(&n.mix.dollars, THIN);
        let len = points.len();
        assert_eq!(len, n.shares.len());
        assert_eq!(points[0].0, 0.0);
        assert_eq!(n.shares[0], None);
        assert_eq!(shown_share(&n.shares, 0.0), None);
        assert_eq!(shown_share(&n.shares, 0.499 / (len - 1) as f64), None);
        let first = first_drawn(len);
        assert_eq!(drawn_index(len, first), 1);
        assert_eq!(points[1].0, THIN as f64);
        assert_eq!(shown_share(&n.shares, first), n.shares[1]);
        assert!(n.shares[1].is_some());
        // a hand count of the rounds so far at a multiple of three
        let so_far: Vec<usize> = std::iter::once(0)
            .chain(n.mix.dollars[..RUN_ROUNDS].iter().scan(0, |count, d| {
                *count += usize::from(d.rem_euclid(3) == 0);
                Some(*count)
            }))
            .collect();
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
        let last = shown_share(&n.shares, 1.0).expect("a share at the end");
        assert_eq!(so_far[RUN_ROUNDS], n.mix.at_zero);
        assert!((last - 100.0 * n.mix.at_zero as f64 / RUN_ROUNDS as f64).abs() < 1e-12);
        assert_eq!(
            format!("{last:.1}"),
            facts::facts().unwrap().get("run.mix.at0.pct").unwrap()
        );
    }

    /// The mark stays on the scale for every drawn prefix, and ends between
    /// a third and B's mark, more than three times closer to a third.
    #[test]
    fn the_mark_stays_on_the_scale_and_ends_near_a_third() {
        let n = numbers().unwrap();
        let (x0, x1) = (scale_x(f64::from(SCALE.0)), scale_x(f64::from(SCALE.1)));
        for s in n.shares.iter().flatten() {
            assert!(scale_x(*s) > x0 && scale_x(*s) < x1, "{s}");
        }
        let end = n.shares.last().copied().flatten().unwrap();
        assert!(n.third < end && end < n.b_share);
        assert!(3.0 * (end - n.third) < n.b_share - end);
        assert!(scale_x(n.third) > x0 && scale_x(n.b_share) < x1);
    }

    /// Negative control: the occupancy, not the coins, decides. With the
    /// mix's own coins, standing at a multiple of three as often as game B
    /// did would lose (below a half of the rounds won), while the mix's own
    /// share wins; and the halfway guess the scene strikes is a loss where
    /// the mix's bar is a gain.
    #[test]
    fn the_share_not_the_coins_decides() {
        let n = numbers().unwrap();
        let mix = PUBLISHED.mix();
        // the coins at corners 1 and 2 are the same, so the share at 0 alone
        // gives the share of rounds won
        assert_eq!(mix.chance(1), mix.chance(2));
        let won = |at_zero: Ratio| at_zero * mix.chance(0) + (Ratio::ONE - at_zero) * mix.chance(1);
        let half = Ratio::new(1, 2);
        assert!(won(n.b.share_at_zero()) < half);
        assert!(won(n.mix.share_at_zero()) > half);
        assert!(won(mix.shares()[0]) > half);
        assert_eq!(won(mix.shares()[0]), mix.win());
        assert!(n.averages.halfway < Ratio::ZERO && n.averages.mix > Ratio::ZERO);
    }

    /// The "precisely" card fits the lower third's three lines with room to
    /// spare (the band wraps at up to 92 characters, fewer when the text is
    /// wide; past three lines it cuts the text short).
    #[test]
    fn the_card_fits_three_lines() {
        let n = numbers().unwrap();
        let lines = explainer_kit::animatic::wrap(&n.k02, 76, usize::MAX);
        assert!(lines.len() <= 3, "{lines:?}");
        assert_eq!(lines.join(" "), n.k02);
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the scale, its marks and their labels
    /// sit between the readout and the window; the think ring and its note
    /// clear the caption, the readout, B's label and the badge; every drawn
    /// point of the three runs stays inside the window, and the mix's run
    /// with the token riding it stays below a third's label; the end labels
    /// keep apart; card B stays inside the frame, right of the player.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let n = numbers().unwrap();
        const {
            // a third's label clears the window's "dollars" (0.22 over its top)
            assert!(SCALE_Y - MARK_LABEL_DY - 0.1 > CLUE.center[1] + CLUE.height / 2.0 + 0.4);
            assert!(SCALE_Y + MARK_LABEL_DY + 0.1 < READOUT_Y - 0.15);
            assert!(READOUT_Y + 0.15 < CAPTION_AT[1] - 0.12);
            assert!(CAPTION_AT[1] + 0.15 < FRAME_Y_RADIUS - EDGE);
            assert!(THINK_AT[1] + THINK_R < BADGE_AT[1] - 0.3);
            assert!(THINK_AT[1] - NOTE_DROP - 0.08 > SCALE_Y + TICK_H + 0.1);
            assert!(TITLE_AT[1] - SUBTITLE_AT[1] > 0.7);
            assert!(CARD_AT[1] + CARD_H / 2.0 < BADGE_AT[1] - 0.3);
            assert!(CARD_AT[1] - CARD_H / 2.0 > -FRAME_Y_RADIUS + 0.15);
            assert!(CARD_AT[0] - CARD_W / 2.0 > TOKEN_AT[0] + TOKEN_R + 1.5);
            assert!(CARD_AT[0] + CARD_W / 2.0 < FRAME_X_RADIUS - 0.15);
        }
        // the note "pause for more time" (about 1.75 wide) ends left of B's
        // label (about 1.35 wide, centred on B's mark)
        assert!(THINK_AT[0] + 0.9 < scale_x(n.b_share) - 0.7);
        assert!(scale_x(f64::from(SCALE.1)) + 0.25 < FRAME_X_RADIUS - EDGE);
        assert!(scale_x(f64::from(SCALE.0)) - 0.25 > -FRAME_X_RADIUS + EDGE);
        let label_floor = SCALE_Y - MARK_LABEL_DY - 0.08;
        for run in [&n.a, &n.b, &n.mix] {
            for (k, d) in thinned(&run.dollars, THIN) {
                let [_, y, _] = CLUE.p(k, d);
                assert!(y > CLUE.bottom() && y < CLUE.top(), "round {k} leaves the window");
            }
        }
        for (k, d) in thinned(&n.mix.dollars, THIN) {
            let [_, y, _] = CLUE.p(k, d);
            assert!(
                y + TOKEN_R < label_floor,
                "round {k} at {d} dollars reaches the scale ({y:.2})"
            );
        }
        let end_y = |run: &model::Run| CLUE.p(0.0, *run.dollars.last().unwrap() as f64)[1];
        let (a_y, b_y, mix_y) = (end_y(&n.a), end_y(&n.b), end_y(&n.mix));
        assert!(b_y - a_y > 0.4 && mix_y - b_y > 0.4, "the end labels overlap");
        assert!(CLUE.bottom() - 0.22 - 0.1 > -FRAME_Y_RADIUS + 0.15);
        assert!(CLUE.right() + END_LABEL_DX + 0.4 < FRAME_X_RADIUS - EDGE);
    }

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// The frames pointing at the marks' labels (05_1 B's, above the scale;
    /// 05_2 a third's, below it) stay clear of their own ticks on the real
    /// typesetting: a frame across its tick would box in the scale itself.
    #[test]
    fn the_pointing_frames_clear_their_ticks() {
        let n = numbers().unwrap();
        run(|stage| {
            let kit = Kit::new()?;
            for (pct, words, up) in [(n.b_share, &n.b_label, true), (n.third, &n.third_label, false)] {
                let [tick, label] = mark(&kit, stage, pct, Palette::dim(), words, up)?;
                let frame = frame_around(stage, &[label], POINT_PAD, Palette::hero())?;
                let (frame_bottom, frame_top) = (
                    stage.get_bounding_box_point(frame, DOWN)[1],
                    stage.get_bounding_box_point(frame, UP)[1],
                );
                let (tick_bottom, tick_top) = (
                    stage.get_bounding_box_point(tick, DOWN)[1],
                    stage.get_bounding_box_point(tick, UP)[1],
                );
                if up {
                    assert!(
                        frame_bottom > tick_top,
                        "{words}: frame {frame_bottom:.3}, tick {tick_top:.3}"
                    );
                } else {
                    assert!(
                        frame_top < tick_bottom,
                        "{words}: frame {frame_top:.3}, tick {tick_bottom:.3}"
                    );
                }
            }
            Ok(())
        });
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` guard meets the true text
    /// widths): every line of chapter 05 is said once, in order, and the
    /// prediction (05_1) gets its silent think time before the answer.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TheClue { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("05_the_clue: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=4).map(|k| format!("05_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let asked_end = at("05_1") + narrator.length("05_1").unwrap();
        assert!(
            at("05_2") - asked_end >= THINK_S,
            "{} s to think",
            at("05_2") - asked_end
        );
        // the title stands alone before 05_4
        let title_end = at("05_3") + narrator.length("05_3").unwrap();
        assert!(at("05_4") - title_end >= TITLE_HOLD, "the title has no time");
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end > at("05_4"), "the chapter ends after its last line: {end}");
    }
}
