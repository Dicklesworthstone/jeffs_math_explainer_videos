//! Chapter 11, mix the games: what switching changes on the triangle. The
//! picture 10 left: B's triangle with the round-number coins' chances, the
//! badge, and the 26 dots settled at 10, 4 and 12. Now mix the games, as the
//! player did: 04's gold coin, "the mix", "heads → game A", "tails → game
//! B" (11_1). Every arrow becomes an average: at corner 0 game A's blue
//! "50 %" joins B's "10 %" and both become the mix's gold "30 %", the back
//! arrow "70 %", with 1/2 · 50 % + 1/2 · 10 % = 30 % in the panel (11_2);
//! at corners 1 and 2, 1/2 · 50 % + 1/2 · 75 % = 62.5 %, back 37.5 %, and
//! the triangle's title becomes "the mix" (11_3). The mix starts from B's
//! settled crowd: the coin leaves, the dots merge into puddles, the panel
//! gets "round 0", "players: 26", the dim "split by the chances (expected
//! numbers)", a row per corner with a live bar and its live share (38.5,
//! 15.4, 46.2 %), and a ghost outline at each of B's heights. Which way will
//! the crowd shift? (11_4, a prediction: no settled value is drawn yet.)
//! Off corner 0: round 1 (slowly, with streams) drops its share at once
//! to 34.6 %, and rounds 2 to 6 (a second each) slosh it above and below
//! that, drawn as a trace of corner 0's share round by round; rounds 7 to
//! 12 follow in silence (11_5). Rounds 13 to 18 and the crowd settles: dim
//! ticks at the settled heights and a dashed line through the trace; the
//! rows become "38.5 → 34.6 %", "15.4 → 25.4 %", "46.2 → 40.1 %" (11_6).
//! Game A's coin ignores the corners and stirs the crowd toward equal
//! thirds (dashed thirds on the bars and the trace); the weight lost off
//! corner 0, between its ghost and its bar, is outlined: the shift is where
//! the winning comes from (11_7, held).
//!
//! Every round is computed from B's settled 10, 4 and 12 under the
//! round-number mix (`model::evolve_f64`), checked exactly against the
//! facts' `mixshift.round.1..6`, and played by gliding one tracker per
//! corner and one for the round: the puddles, bars, shares, total, round
//! counter and trace all read them, so the total is 26 in every frame. The
//! settled values appear only once the crowd has settled, never during the
//! prediction. The published coins' 34.5 % (05_2's run, 13_8's callback) is
//! not this model's number and is never drawn here. No typeset text is
//! indicated, scaled or recoloured: corners and the coin get hollow rings,
//! words an outline, a changed label is replaced.

use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, live_bar, rect, v};
use fmn::prelude::*;

use super::draw::{
    coin, dashed, fade_in_all, fade_out_all, fail, flash_all, flat, keep_between, line_style, now, pause, say, stand,
    think,
};
use super::looks_hopeless::{POINT_PAD, frame_around, point_at};
use super::one_player_to_a_crowd::{
    BAR_H, BAR_X, PANEL_X, PLAYERS_NUM_X, PLAYERS_Y, ROW_YS, dot, panel_head, places, ring_corners, round_badge,
    round_badge_text, row_name, shown,
};
use super::the_trap::shaft;
use super::three_corners::{
    B_AT, CORNER_R, Chances, TITLE_DY, Which, apex, chance_at, chance_labels, chances, corner, edge_of, keep_apart,
    swap, triangle,
};
use crate::facts::{self, SETTLE_LOOK};
use crate::model::{self, Game, ROUND, Ratio};
use crate::palette;

pub struct MixTheGames {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const MIX_NAME: &str = "the mix";
const HEADS_FACE: &str = "A";
const HEADS_RULE: &str = "heads \u{2192} game A";
const TAILS_RULE: &str = "tails \u{2192} game B";
const ROUND_WORD: &str = "round";
const QUESTION: &str = "which way will the crowd shift?";
const TRACE_HEAD: &str = "share at corner 0, round by round";
const ROUNDS_AXIS: &str = "rounds";
const PERCENT: &str = "%";

/// Where each picture lands: (line, the phrase it waits for).
const AT_MIX: (&str, &str) = ("11_1", "Now mix the games");
const AT_PLAYER: (&str, &str) = ("11_1", "as the player did");
const AT_FAIR: (&str, &str) = ("11_1", "a fair coin decides");
const AT_AVERAGE: (&str, &str) = ("11_2", "every arrow becomes an average");
const AT_A_FIFTY: (&str, &str) = ("11_2", "A's fifty percent");
const AT_B_TEN: (&str, &str) = ("11_2", "B's ten percent");
const AT_THIRTY: (&str, &str) = ("11_2", "forward thirty percent in all");
const AT_ONE_TWO: (&str, &str) = ("11_3", "At corners one and two");
const AT_HALFWAY: (&str, &str) = ("11_3", "halfway between fifty and seventy-five");
const AT_SIXTY: (&str, &str) = ("11_3", "forward sixty-two and a half percent");
const AT_START: (&str, &str) = ("11_4", "Start the mix from game B's settled crowd");
const AT_COUNTS: (&str, &str) = ("11_4", "ten, four and twelve");
const AT_STILL: (&str, &str) = ("11_4", "still split exactly by the chances");
const AT_WHICH: (&str, &str) = ("11_4", "Which way will the crowd shift");
const AT_OFF: (&str, &str) = ("11_5", "Off corner zero");
const AT_DROPS: (&str, &str) = ("11_5", "Its share drops at once");
const AT_ABOUT: (&str, &str) = ("11_5", "to about thirty-four point six");
const AT_SLOSH: (&str, &str) = ("11_5", "then sloshes a little above and below that");
const AT_SETTLES: (&str, &str) = ("11_6", "the crowd settles");
const AT_KEEPS: (&str, &str) = ("11_6", "Corner zero keeps about thirty-four point six percent");
const AT_ONE_UP: (&str, &str) = ("11_6", "corner one ends up at twenty-five point four percent");
const AT_UP_FROM: (&str, &str) = ("11_6", "up from fifteen point four");
const AT_IGNORES: (&str, &str) = ("11_7", "Game A's coin ignores the corners");
const AT_THIRDS: (&str, &str) = ("11_7", "toward equal thirds");
const AT_SHIFT: (&str, &str) = ("11_7", "That shift of weight off corner zero");
const AT_WINNING: (&str, &str) = ("11_7", "where the winning comes from");
#[cfg(test)]
const CUES: [(&str, &str); 26] = [
    AT_MIX, AT_PLAYER, AT_FAIR, AT_AVERAGE, AT_A_FIFTY, AT_B_TEN, AT_THIRTY, AT_ONE_TWO, AT_HALFWAY, AT_SIXTY,
    AT_START, AT_COUNTS, AT_STILL, AT_WHICH, AT_OFF, AT_DROPS, AT_ABOUT, AT_SLOSH, AT_SETTLES, AT_KEEPS, AT_ONE_UP,
    AT_UP_FROM, AT_IGNORES, AT_THIRDS, AT_SHIFT, AT_WINNING,
];

// ------------------------------------------------------------------ layout

/// 04's mix: the gold coin with its name above and its two rules beside it
/// (04's own places, so the callback is the same picture).
const MIX_COIN_AT: Vec3 = [-5.6, 2.55, 0.0];
const MIX_COIN_R: f64 = 0.4;
const MIX_NAME_AT: Vec3 = [-5.6, 3.35, 0.0];
const RULE_X: f64 = -4.95;
const RULE_YS: [f64; 2] = [2.78, 2.32];
/// The averages' sums in the panel, under the coin.
const SUM_YS: [f64; 2] = [1.45, 0.6];
/// The panel's text keeps left of this (B's corner 2, its name and labels).
const PANEL_RIGHT: f64 = -0.7;
/// The round counter, as 08's: its word, then its number.
const ROUND_Y: f64 = 3.3;
const ROUND_NUM_X: f64 = -5.95;
/// A row's share: its number's left edge, then the percent sign; a bar is
/// this long for the whole crowd (100 %).
const SHARE_X: f64 = -2.3;
const PCT_X: f64 = -1.75;
const BAR_ALL: f64 = 6.0;
/// The precision note, the question, the countdown.
const NOTE_Y: f64 = 0.2;
const QUESTION_Y: f64 = -0.25;
const THINK_AT: Vec3 = [-3.9, -1.4, 0.0];
/// The trace of corner 0's share against rounds: its head, its box, and
/// its y range in percent (a third less this much to B's share plus this
/// much; no axis numbers are drawn).
const TRACE_HEAD_Y: f64 = -0.3;
const TRACE_LEFT: f64 = -6.3;
const TRACE_RIGHT: f64 = -2.3;
const TRACE_BOTTOM: f64 = -2.55;
const TRACE_TOP: f64 = -0.65;
const TRACE_BELOW: f64 = 2.0;
const TRACE_ABOVE: f64 = 1.0;
const TRACE_LABEL_DX: f64 = 0.1;
/// A tick across a bar reaches this far above and below it.
const TICK_OUT: f64 = 0.07;
/// A label for an arrow's chance, as the triangles' own.
const CHANCE_SIZE: f64 = 20.0;
/// Past a chance label, A's chance waits this far from it.
const CHIP_GAP: f64 = 0.12;
/// The streams of a round, as 08's.
const STREAM_MIN_W: f64 = 1.5;
const STREAM_MAX_W: f64 = 16.0;
const STREAM_O: f64 = 0.7;
const STREAM_FADE: f64 = 0.3;
/// Seconds: round 1, rounds 2 to 6, rounds 7 to 12 (silent), rounds 13 on;
/// a beat after the silent rounds.
const R1_S: f64 = 2.2;
const ROUND_S: f64 = 1.0;
const QUICK_S: f64 = 0.5;
const FAST_S: f64 = 0.3;
const BEAT: f64 = 1.0;
/// The rounds played slowly, then quickly in silence.
const SLOSH_TO: usize = facts::MIX_SHIFT_ROUNDS;
const QUICK_TO: usize = 12;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    round_badge: String,
    b_round: Chances,
    mix_round: Chances,
    /// Game A's chance, as its blue label says it.
    a_chance: String,
    n: i64,
    counts: [usize; 3],
    /// The split crowd from 10, 4, 12 after each round to `last`.
    rounds: Vec<[f64; 3]>,
    /// The first round from which every share shows its settled value for
    /// good (to [`SETTLE_LOOK`]).
    last: usize,
    sums: [String; 2],
    /// The mix's settled shares, as percentages, and a third.
    mix_pct: [String; 3],
    third: String,
    /// Each row's settled line, "38.5 → 34.6 %".
    settled: [String; 3],
    note: String,
    /// The exact weights, for ticks, outlines and the trace's lines.
    b_w: [f64; 3],
    mix_w: [f64; 3],
}

fn csv<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    values.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

/// The split crowd from `start` before and after each of `rounds` rounds of
/// `g`, in floats.
fn mix_rounds(start: [f64; 3], g: &Game, rounds: usize) -> Vec<[f64; 3]> {
    let mut v = start;
    let mut out = Vec::with_capacity(rounds + 1);
    out.push(v);
    for _ in 0..rounds {
        v = model::evolve_f64(&v, g);
        out.push(v);
    }
    out
}

/// The shares of a crowd of `n`, as the readouts print them.
fn shares_shown(v: &[f64; 3], n: i64) -> [String; 3] {
    v.map(|x| format!("{:.1}", shown(100.0 * x / n as f64, 1)))
}

/// The first round from which every share of `rounds` reads `target` for
/// good.
fn settled_from(rounds: &[[f64; 3]], n: i64, target: &[String; 3]) -> Option<usize> {
    let ok: Vec<bool> = rounds.iter().map(|r| &shares_shown(r, n) == target).collect();
    let last_bad = ok.iter().rposition(|&x| !x);
    match last_bad {
        None => Some(0),
        Some(k) if k + 1 < ok.len() => Some(k + 1),
        Some(_) => None,
    }
}

/// How often corner 0's share crosses `limit` (a share) in `rounds`.
fn crossings(rounds: &[[f64; 3]], n: i64, limit: f64) -> usize {
    let sides: Vec<bool> = rounds.iter().map(|r| r[0] / n as f64 > limit).collect();
    sides.windows(2).filter(|w| w[0] != w[1]).count()
}

/// "1/2 · a % + 1/2 · b % = m %", each chance as an exact percentage.
fn average_tex(a: Ratio, b: Ratio, m: Ratio) -> Result<String, String> {
    if (a + b) / Ratio::int(2) != m {
        return Err(format!("{m} is not halfway between {a} and {b}"));
    }
    Ok(format!(
        r"\tfrac{{1}}{{2}} \cdot {}\% + \tfrac{{1}}{{2}} \cdot {}\% = {}\%",
        a.exact_pct()?,
        b.exact_pct()?,
        m.exact_pct()?
    ))
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let (a, b, mix) = (ROUND.game_a(), ROUND.game_b(), ROUND.mix());
    // 11_1: a fair coin picks the game; 11_2 and 11_3: every arrow is the
    // average of A's and B's
    let fair = a.mixed(&b, 1, 2);
    if (0..3).any(|i| mix.chance(i) != fair.chance(i)) {
        return Err("the mix is not a fair coin between A and B".into());
    }
    let [a_c, bad, good] = facts::coins(&ROUND);
    let heads: Vec<String> = [a_c, bad, good]
        .iter()
        .map(|x| x.exact_pct())
        .collect::<Result<_, _>>()?;
    if get("coins.round.heads.pct")? != heads.join(",") {
        return Err("the coins disagree with the facts".into());
    }
    let sums = [
        average_tex(a.chance(0), b.chance(0), mix.chance(0))?,
        average_tex(a.chance(1), b.chance(1), mix.chance(1))?,
    ];
    if mix.chance(1) != mix.chance(2) || b.chance(1) != b.chance(2) || a.chance(0) != a.chance(1) {
        return Err("11_3 says corners 1 and 2 share one average".into());
    }
    let mix_round = chances(&mix)?;
    if get("round.mix.coins.pct")? != csv([&mix_round.heads[0], &mix_round.heads[1]])
        || get("round.mix.tails.pct")? != csv([&mix_round.tails[0], &mix_round.tails[1]])
    {
        return Err("the mix's chances disagree with the facts".into());
    }
    // 11_4: from B's settled crowd, ten, four and twelve, of twenty-six
    let n = model::smallest_whole_crowd(&b);
    let counts_r = b.shares().map(|w| Ratio::int(n) * w);
    if get("crowd.size")? != n.to_string() || get("crowd.counts")? != csv(counts_r) {
        return Err("11_4 says B's settled ten, four and twelve of twenty-six".into());
    }
    let counts = counts_r.map(|x| usize::try_from(x.num()).unwrap_or(usize::MAX));
    if counts != [10, 4, 12] || counts_r.iter().any(|x| !x.is_integer()) {
        return Err("11_4 says ten, four and twelve".into());
    }
    // every round from that crowd: exact through the facts' six, then in
    // floats to the settled round
    let exact = facts::rounds_from(counts_r, &mix, SLOSH_TO);
    let start = counts.map(|c| c as f64);
    let look = mix_rounds(start, &mix, SETTLE_LOOK);
    for (k, v) in exact.iter().enumerate() {
        let printed = csv(v.map(|x| (x / Ratio::int(n)).pct(1)));
        if get(&format!("mixshift.round.{}", k + 1))? != printed {
            return Err(format!("round {} disagrees with the facts", k + 1));
        }
        if csv(shares_shown(&look[k + 1], n)) != printed {
            return Err(format!("round {}'s readouts disagree with the exact crowd", k + 1));
        }
        if v.iter().fold(Ratio::ZERO, |s, x| s + *x) != Ratio::int(n) {
            return Err(format!("round {} loses players", k + 1));
        }
    }
    // the limits: B's settled shares and the mix's
    let b_w = b.shares();
    let mix_w_r = mix.shares();
    let b_pct = b_w.map(|w| w.pct(1));
    let mix_pct = mix_w_r.map(|w| w.pct(1));
    if get("round.b.weights.pct")? != csv(&b_pct)
        || get("crowd.share0.pct")? != b_pct[0]
        || get("round.mix.weights")? != csv(mix_w_r)
        || get("round.mix.weights.pct")? != csv(&mix_pct)
    {
        return Err("the settled shares disagree with the facts".into());
    }
    let mix_w = mix_w_r.map(Ratio::to_f64);
    // 11_5: off corner zero: its share drops at once to about 34.6, then
    // sloshes above and below it
    let share0 = |r: &[f64; 3]| r[0] / n as f64;
    if mix_w[0] >= b_w[0].to_f64() || share0(&look[1]) >= share0(&look[0]) {
        return Err("11_5 says the crowd moves off corner zero".into());
    }
    if shares_shown(&look[1], n)[0] != mix_pct[0] || (share0(&look[1]) - mix_w[0]).abs() >= 0.001 {
        return Err("11_5 says round 1 drops it at once to about the settled share".into());
    }
    if crossings(&look[..=SLOSH_TO], n, mix_w[0]) < 2 {
        return Err("11_5 says it sloshes above and below the settled share".into());
    }
    // 11_6: the swings shrink and the crowd settles; corner 1 ends up at
    // its settled share, up from B's
    let last = settled_from(&look, n, &mix_pct).ok_or("the crowd never settles")?;
    if last <= QUICK_TO {
        return Err("the crowd settles before 11_6 says so".into());
    }
    for (k, pair) in look[2..=last].windows(2).enumerate() {
        if (share0(&pair[1]) - mix_w[0]).abs() >= (share0(&pair[0]) - mix_w[0]).abs() {
            return Err(format!("the swing at corner 0 grows in round {}", k + 3));
        }
    }
    if mix_w_r[1] <= b_w[1] {
        return Err("11_6 says corner 1 ends up higher".into());
    }
    // 11_7: toward equal thirds, off corner zero
    let third = Ratio::new(1, 3);
    if get("thirds.pct")? != third.pct(1) || mix_w_r[0] <= third || mix_w_r[0] >= b_w[0] {
        return Err("11_7 says the mix moves corner 0 toward a third".into());
    }
    let settled = [0, 1, 2].map(|i| format!("{} \u{2192} {} {PERCENT}", b_pct[i], mix_pct[i]));
    Ok(Numbers {
        round_badge: round_badge_text(&f)?,
        b_round: chances(&b)?,
        mix_round,
        a_chance: format!("{} {PERCENT}", a.chance(0).exact_pct()?),
        n,
        counts,
        rounds: look[..=last].to_vec(),
        last,
        sums,
        mix_pct,
        third: format!("{} {PERCENT}", third.pct(1)),
        settled,
        note: format!("shares shown to one decimal; the total is exactly {n}"),
        b_w: b_w.map(Ratio::to_f64),
        mix_w,
    })
}

// --------------------------------------------------------------- pictures

/// A tracker to `value` in `rt` seconds at `rate` (08's `glide`: an
/// animation's arguments are set once, so the kit's `tween` cannot take a
/// rate).
fn glide(tracker: Mob, value: f64, rt: f64, rate: fn(f64) -> f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(rate),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}

/// The split crowd's live state, as 08's: the round (gliding between whole
/// rounds) and one tracker per corner, moved together.
struct Crowd {
    progress: Mob,
    count: [Mob; 3],
}

impl Crowd {
    /// Play round `k`: every tracker glides to the crowd after round `k` in
    /// the same `rt` at the same `rate`, `along` drawn along their arrows.
    fn round(
        &self,
        stage: &mut Stage<'_>,
        rounds: &[[f64; 3]],
        k: usize,
        rt: f64,
        rate: fn(f64) -> f64,
        along: &[Mob],
    ) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(glide(self.progress, k as f64, rt, rate)?)?];
        for (i, &c) in self.count.iter().enumerate() {
            anims.push(stage.prepare(glide(c, rounds[k][i], rt, rate)?)?);
        }
        for &s in along {
            anims.push(stage.prepare(show_creation(s).rt(rt))?);
        }
        stage.play_prepared(anims)?;
        Ok(())
    }
}

/// The streams of the round that starts from `from` under `g`: each
/// corner's share forward along its forward arrow and the rest back, a
/// teal band wider the more it carries (of `n`). Detached.
fn streams(stage: &mut Stage<'_>, from: &[f64; 3], g: &Game, n: i64) -> Vec<Mob> {
    let p = g.chances_f64();
    let mut out = Vec::with_capacity(6);
    for (i, &here) in from.iter().enumerate() {
        if here < 1e-9 {
            continue;
        }
        for (to, flow) in [((i + 1) % 3, here * p[i]), ((i + 2) % 3, here * (1.0 - p[i]))] {
            let (e, forward) = edge_of(i, to);
            let w = STREAM_MIN_W + (STREAM_MAX_W - STREAM_MIN_W) * flow / n as f64;
            let style = Style::default()
                .fill(palette::player(), 0.0)
                .stroke(palette::player(), w, STREAM_O);
            out.push(stage.arena_mut().add(polyline(shaft(e, forward)).style(style).build()));
        }
    }
    out
}

/// A label for an arrow's chance at its place beside the arrow along edge
/// `e` of B's triangle. Detached.
fn chance_label(
    kit: &Kit,
    stage: &mut Stage<'_>,
    words: &str,
    color: Srgb,
    e: usize,
    forward: bool,
) -> fmn::Result<Mob> {
    let m = kit.text(stage, words, CHANCE_SIZE, color, ORIGIN)?;
    let at = chance_at(B_AT, e, forward, stage.get_width(m), stage.get_height(m));
    stage.move_to(m, at, ORIGIN);
    Ok(m)
}

/// Move `words` past `label` (a chance by the arrow along edge `e`),
/// further from the arrow, [`CHIP_GAP`] between them (as 08's flows).
fn past(stage: &mut Stage<'_>, label: Mob, e: usize, forward: bool, words: Mob) {
    let a = apex(B_AT, e, forward);
    let c = stage.get_center(label);
    let len = (c[0] - a[0]).hypot(c[1] - a[1]).max(1e-9);
    let d = [(c[0] - a[0]) / len, (c[1] - a[1]) / len];
    let reach = |w: f64, h: f64| w / 2.0 * d[0].abs() + h / 2.0 * d[1].abs();
    let r = reach(stage.get_width(label), stage.get_height(label))
        + CHIP_GAP
        + reach(stage.get_width(words), stage.get_height(words));
    stage.move_to(words, v(c[0] + d[0] * r, c[1] + d[1] * r), ORIGIN);
}

/// Hollow rings of radius `r` about `centres`, their strokes pulsing
/// together: a coin or a corner is never flashed itself.
fn rings_at(stage: &mut Stage<'_>, centres: &[Vec3], r: f64, rt: f64) -> fmn::Result<()> {
    let mut rings = Vec::with_capacity(centres.len());
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(centres.len());
    for &c in centres {
        let ring = Circle::new()
            .radius(r)
            .arc_center(c)
            .style(
                Style::default()
                    .fill(Palette::hero(), 0.0)
                    .stroke(Palette::hero(), 3.0, 0.0),
            )
            .build();
        let ring = stage.arena_mut().add(ring);
        stage.add_to_scene(ring)?;
        let pulse = ring
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                rate_func: Some(fmn::core::rate::there_and_back),
                ..AnimateArgs::default()
            })?
            .set_stroke(None, None, Some(1.0))?;
        anims.push(stage.prepare(pulse)?);
        rings.push(ring);
    }
    stage.play_prepared(anims)?;
    stage.scene_mut().remove(&rings);
    Ok(())
}

/// A tick across row `i`'s bar at `share` of the crowd: a solid one, or a
/// dashed one. Detached.
fn tick(stage: &mut Stage<'_>, i: usize, share: f64, dashed_line: bool) -> Vec<Mob> {
    let x = BAR_X + BAR_ALL * share;
    let (lo, hi) = (
        v(x, ROW_YS[i] - BAR_H / 2.0 - TICK_OUT),
        v(x, ROW_YS[i] + BAR_H / 2.0 + TICK_OUT),
    );
    if dashed_line {
        dashed(stage, lo, hi, 0.06, Palette::dim(), 2.0)
    } else {
        vec![
            stage
                .arena_mut()
                .add(polyline([lo, hi]).style(line_style(Palette::dim(), 2.5)).build()),
        ]
    }
}

/// A line of the panel, its left edge at [`PANEL_X`], kept left of
/// [`PANEL_RIGHT`]. Detached.
fn panel_line(kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, y: f64) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, words, size, color, v(PANEL_X, y))?;
    keep_between(stage, m, PANEL_X - 0.01, PANEL_RIGHT, words)?;
    Ok(m)
}

impl SceneConstruct for MixTheGames {
    fn name(&self) -> &str {
        "11_mix_the_games"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let size = n.n;
        let whole = size as f64;
        let mix = ROUND.mix();

        // 11_1: 10's crowd at 10, 4 and 12 on B's triangle; now mix the
        // games, as the player did: 04's gold coin, its name and its rules
        let line = say(&kit, stage, "11_1")?;
        let b = triangle(&kit, stage, Which::B)?;
        let labels = chance_labels(&kit, stage, Which::B, &n.b_round)?;
        let tag = round_badge(&kit, stage, &n.round_badge)?;
        let mut corner_of_player = Vec::with_capacity(n.counts.iter().sum());
        for (i, &c) in n.counts.iter().enumerate() {
            corner_of_player.extend(std::iter::repeat_n(i, c));
        }
        let dots: Vec<Mob> = places(&corner_of_player, |i| corner(B_AT, i))
            .into_iter()
            .map(|p| dot(stage, p))
            .collect();
        let mut arriving = b.parts();
        arriving.extend(labels);
        arriving.extend(tag);
        arriving.extend(&dots);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, AT_MIX.1)?;
        let mix_coin = coin(&kit, stage, MIX_COIN_AT, MIX_COIN_R, palette::mix(), HEADS_FACE)?;
        let mix_name = kit.text(stage, MIX_NAME, 26.0, palette::mix(), MIX_NAME_AT)?;
        fade_in_all(stage, &[mix_coin[0], mix_coin[1], mix_name], 0.5)?;
        line.wait_for(stage, AT_PLAYER.1)?;
        rings_at(stage, &[MIX_COIN_AT], MIX_COIN_R + 0.06, 0.7)?;
        line.wait_for(stage, AT_FAIR.1)?;
        let heads = kit.text_left(stage, HEADS_RULE, 24.0, palette::game_a(), v(RULE_X, RULE_YS[0]))?;
        let tails = kit.text_left(stage, TAILS_RULE, 24.0, palette::game_b(), v(RULE_X, RULE_YS[1]))?;
        keep_between(stage, heads, RULE_X - 0.01, PANEL_RIGHT, "the mix's rules")?;
        keep_between(stage, tails, RULE_X - 0.01, PANEL_RIGHT, "the mix's rules")?;
        fade_in_all(stage, &[heads, tails], 0.5)?;
        let mut texts: Vec<Mob> = labels.to_vec();
        texts.extend(b.names);
        texts.extend([b.title, tag[1], mix_coin[1], mix_name, heads, tails]);
        keep_apart(stage, &texts, 0.08, "the mix's coin")?;

        // 11_2: every arrow becomes an average: at corner 0, A's 50 % and
        // B's 10 % become the mix's 30 %, back 70 %
        let line = say(&kit, stage, "11_2")?;
        line.wait_for(stage, AT_AVERAGE.1)?;
        let shafts: Vec<Mob> = b.forward.iter().chain(&b.back).map(|a| a[0]).collect();
        flash_all(stage, &shafts, 0.8)?;
        line.wait_for(stage, AT_A_FIFTY.1)?;
        // corner 0's forward arrow is edge 0's; its back arrow edge 2's
        let (fw0, bk0) = (edge_of(0, 1), edge_of(0, 2));
        let a_chip = kit.text(stage, &n.a_chance, CHANCE_SIZE, palette::game_a(), ORIGIN)?;
        past(stage, labels[fw0.0], fw0.0, fw0.1, a_chip);
        fade_in_all(stage, &[a_chip], 0.4)?;
        let pointer = frame_around(stage, &[a_chip], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.6)?;
        line.wait_for(stage, AT_B_TEN.1)?;
        let pointer = frame_around(stage, &[labels[fw0.0]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.6)?;
        line.wait_for(stage, AT_THIRTY.1)?;
        let mix_fw0 = chance_label(
            &kit,
            stage,
            &format!("{} {PERCENT}", n.mix_round.heads[0]),
            palette::mix(),
            fw0.0,
            true,
        )?;
        let mix_bk0 = chance_label(
            &kit,
            stage,
            &format!("{} {PERCENT}", n.mix_round.tails[0]),
            Palette::dim(),
            bk0.0,
            false,
        )?;
        let sum0 = kit.tex(stage, &n.sums[0], 30.0, Palette::ink(), &[], ORIGIN)?;
        stage.move_to(sum0, v(PANEL_X, SUM_YS[0]), LEFT);
        keep_between(stage, sum0, PANEL_X - 0.01, PANEL_RIGHT, "corner 0's average")?;
        let anims = swap(
            stage,
            &[labels[fw0.0], a_chip, labels[3 + bk0.0]],
            &[mix_fw0, mix_bk0, sum0],
            0.7,
        )?;
        stage.play_prepared(anims)?;

        // 11_3: at corners 1 and 2, halfway between 50 % and 75 %: 62.5 %,
        // back 37.5 %; the triangle becomes the mix's
        let line = say(&kit, stage, "11_3")?;
        line.wait_for(stage, AT_ONE_TWO.1)?;
        rings_at(stage, &[corner(B_AT, 1), corner(B_AT, 2)], CORNER_R + 0.05, 0.7)?;
        line.wait_for(stage, AT_HALFWAY.1)?;
        let sum1 = kit.tex(stage, &n.sums[1], 30.0, Palette::ink(), &[], ORIGIN)?;
        stage.move_to(sum1, v(PANEL_X, SUM_YS[1]), LEFT);
        keep_between(stage, sum1, PANEL_X - 0.01, PANEL_RIGHT, "the other corners' average")?;
        fade_in_all(stage, &[sum1], 0.5)?;
        line.wait_for(stage, AT_SIXTY.1)?;
        // forward from corners 1 and 2 along edges 1 and 2; their losses
        // back along edges 0 and 1
        let mut old = Vec::with_capacity(5);
        let mut new = Vec::with_capacity(5);
        for i in [1, 2] {
            let (e, _) = edge_of(i, (i + 1) % 3);
            let (eb, _) = edge_of(i, (i + 2) % 3);
            old.extend([labels[e], labels[3 + eb]]);
            let heads_words = format!("{} {PERCENT}", n.mix_round.heads[i]);
            let tails_words = format!("{} {PERCENT}", n.mix_round.tails[i]);
            new.push(chance_label(&kit, stage, &heads_words, palette::mix(), e, true)?);
            new.push(chance_label(&kit, stage, &tails_words, Palette::dim(), eb, false)?);
        }
        let mix_title = kit.text(stage, MIX_NAME, 24.0, palette::mix(), v(B_AT[0], B_AT[1] - TITLE_DY))?;
        old.push(b.title);
        new.push(mix_title);
        let anims = swap(stage, &old, &new, 0.7)?;
        stage.play_prepared(anims)?;
        // forward along edges 0, 1, 2, then back along them
        let mix_labels = [mix_fw0, new[0], new[2], new[1], new[3], mix_bk0];
        let mut texts: Vec<Mob> = mix_labels.to_vec();
        texts.extend(b.names);
        texts.extend([mix_title, tag[1], mix_coin[1], mix_name, heads, tails, sum0, sum1]);
        keep_apart(stage, &texts, 0.08, "the mix's chances")?;
        kit.hold(stage)?;

        // 11_4: from B's settled crowd, 10, 4 and 12, still split by the
        // chances: puddles, bars, live shares, ghosts at B's heights; which
        // way will the crowd shift? (a prediction: nothing settled is drawn)
        let line = say(&kit, stage, "11_4")?;
        line.wait_for(stage, AT_START.1)?;
        let crowd = Crowd {
            progress: stage.add_value_tracker(0.0),
            count: n.rounds[0].map(|x| stage.add_value_tracker(x)),
        };
        let round_word = kit.text_left(stage, ROUND_WORD, 22.0, Palette::dim(), v(PANEL_X, ROUND_Y))?;
        keep_between(stage, round_word, PANEL_X - 0.01, ROUND_NUM_X - 0.1, "the round's word")?;
        let head = panel_head(&kit, stage)?;
        let names: Vec<Mob> = (0..3).map(|i| row_name(&kit, stage, i)).collect::<fmn::Result<_>>()?;
        let percents: Vec<Mob> = ROW_YS
            .iter()
            .map(|&y| kit.text_left(stage, PERCENT, 22.0, Palette::ink(), v(PCT_X, y)))
            .collect::<fmn::Result<_>>()?;
        let note = kit.text_left(stage, &n.note, 16.0, Palette::dim(), v(PANEL_X, NOTE_Y))?;
        keep_between(stage, note, PANEL_X - 0.01, PANEL_RIGHT, "the precision note")?;
        let leaving = [mix_coin[0], mix_coin[1], mix_name, heads, tails, sum0, sum1];
        let mut coming = vec![round_word, head[0], head[1], note];
        coming.extend(&names);
        coming.extend(&percents);
        // The equations and crowd rows use the same panel. Clear it before
        // introducing the new representation, preserving the 0.7 s transition.
        fade_out_all(stage, &leaving, 0.35)?;
        fade_in_all(stage, &coming, 0.35)?;
        let progress = crowd.progress;
        let round_num = kit.live_value(
            stage,
            move |s| {
                // the round being played, or the last one played
                let t = s.tracker_value(progress).unwrap_or(0.0);
                if t <= 1e-9 { 0.0 } else { (t - 1e-9).ceil() }
            },
            0,
            false,
            22.0,
            Palette::ink(),
            v(ROUND_NUM_X, ROUND_Y),
            LEFT,
        )?;
        let cs = crowd.count;
        let total = kit.live_value(
            stage,
            // Integer live values truncate; round away floating-point drift
            // so 25.999999999999996 cannot falsely display 25 players.
            move |s| {
                cs.iter()
                    .map(|&c| s.tracker_value(c).unwrap_or(0.0))
                    .sum::<f64>()
                    .round()
            },
            0,
            false,
            22.0,
            Palette::ink(),
            v(PLAYERS_NUM_X, PLAYERS_Y),
            LEFT,
        )?;
        let unit = BAR_ALL / whole;
        let mut row_shares = Vec::with_capacity(3);
        for (i, &c) in crowd.count.iter().enumerate() {
            live_bar(stage, c, v(BAR_X, ROW_YS[i]), unit, BAR_H, palette::player())?;
            row_shares.push(kit.live_value(
                stage,
                move |s| shown(100.0 * s.tracker_value(c).unwrap_or(0.0) / whole, 1),
                1,
                false,
                22.0,
                Palette::ink(),
                v(SHARE_X, ROW_YS[i]),
                LEFT,
            )?);
        }
        line.wait_for(stage, AT_COUNTS.1)?;
        // the dots merge into a puddle at each corner, area ∝ its players
        for (i, &c) in crowd.count.iter().enumerate() {
            let at = corner(B_AT, i);
            let m = stage.always_redraw(move |s| {
                let share = (s.tracker_value(c).unwrap_or(0.0) / whole).clamp(0.0, 1.0);
                let r = (CORNER_R * share.sqrt()).max(0.004);
                s.add(
                    Circle::new()
                        .radius(r)
                        .arc_center(at)
                        .style(flat(palette::player(), 0.85))
                        .build(),
                )
            });
            stage.add_to_scene(m)?;
        }
        // B's heights, kept as ghost outlines
        let mut ghosts = Vec::with_capacity(3);
        for (&y, &w) in ROW_YS.iter().zip(&n.b_w) {
            let g = rect(
                v(BAR_X, y - BAR_H / 2.0),
                BAR_ALL * w,
                BAR_H,
                line_style(Palette::dim(), 1.5),
            )?;
            ghosts.push(stage.arena_mut().add(g));
        }
        fade_out_all(stage, &dots, 0.6)?;
        fade_in_all(stage, &ghosts, 0.4)?;
        ring_corners(stage, &b.discs, &[], 0.7)?;
        line.wait_for(stage, AT_STILL.1)?;
        let pointer = frame_around(stage, &[head[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut panel_texts = vec![round_word, round_num, head[0], head[1], note, total];
        panel_texts.extend(&names);
        panel_texts.extend(&percents);
        panel_texts.extend(&row_shares);
        keep_apart(stage, &panel_texts, 0.06, "the mix's panel")?;
        line.wait_for(stage, AT_WHICH.1)?;
        let question = panel_line(&kit, stage, QUESTION, 22.0, Palette::ink(), QUESTION_Y)?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 11_5: off corner 0: round 1, slowly, drops its share at once to
        // about the settled share; rounds 2 to 6 slosh above and below it,
        // traced; rounds 7 to 12 follow in silence
        let line = say(&kit, stage, "11_5")?;
        line.wait_for(stage, AT_OFF.1)?;
        let trace = Frame2 {
            center: v((TRACE_LEFT + TRACE_RIGHT) / 2.0, (TRACE_BOTTOM + TRACE_TOP) / 2.0),
            width: TRACE_RIGHT - TRACE_LEFT,
            height: TRACE_TOP - TRACE_BOTTOM,
            x: (0.0, n.last as f64),
            y: (100.0 / 3.0 - TRACE_BELOW, 100.0 * n.b_w[0] + TRACE_ABOVE),
        };
        let trace_head = panel_line(&kit, stage, TRACE_HEAD, 16.0, Palette::dim(), TRACE_HEAD_Y)?;
        let (lo, hi) = trace.y;
        let axes = [
            polyline([trace.p(0.0, lo), trace.p(n.last as f64, lo)]),
            polyline([trace.p(0.0, lo), trace.p(0.0, hi)]),
        ]
        .map(|p| stage.arena_mut().add(p.style(line_style(Palette::dim(), 2.0)).build()));
        let rounds_axis = kit.text(stage, ROUNDS_AXIS, 16.0, Palette::dim(), ORIGIN)?;
        stage.move_to(rounds_axis, v(TRACE_RIGHT, TRACE_BOTTOM - 0.2), RIGHT);
        let mut anims = swap(stage, &[question, q], &[trace_head, rounds_axis, axes[0], axes[1]], 0.5)?;
        // the trace: every whole round played so far, and its tip between
        // the last whole round and the next, read from the same trackers
        let points: Vec<Vec3> = n
            .rounds
            .iter()
            .enumerate()
            .map(|(k, r)| trace.p(k as f64, 100.0 * r[0] / whole))
            .collect();
        let trace_line = stage.arena_mut().add(
            polyline([points[0], points[0]])
                .style(line_style(palette::player(), 3.0))
                .build(),
        );
        let trace_anchor = stage.arena_mut().add(VMobject::new());
        let c0 = crowd.count[0];
        let mut last_tip = None;
        stage.add_updater(
            trace_anchor,
            move |s, _| {
                let at = s
                    .tracker_value(progress)
                    .unwrap_or(0.0)
                    .clamp(0.0, (points.len() - 1) as f64);
                let pct = 100.0 * s.tracker_value(c0).unwrap_or(0.0) / whole;
                if last_tip == Some((at, pct)) {
                    return;
                }
                let whole_rounds = at.floor() as usize;
                let mut drawn = points[..=whole_rounds].to_vec();
                if at > whole_rounds as f64 {
                    drawn.push(trace.p(at, pct));
                }
                if drawn.len() < 2 {
                    drawn.push(drawn[0]);
                }
                let _ = s.set_points(trace_line, polyline(drawn).build().points());
                last_tip = Some((at, pct));
            },
            true,
        )?;
        stage.add_to_scene(trace_anchor)?;
        anims.extend(swap(stage, &[], &[trace_line], 0.5)?);
        stage.play_prepared(anims)?;
        rings_at(stage, &[corner(B_AT, 0)], CORNER_R + 0.05, 0.6)?;
        line.wait_for(stage, AT_DROPS.1)?;
        let along = streams(stage, &n.rounds[0], &mix, size);
        crowd.round(stage, &n.rounds, 1, R1_S, fmn::core::rate::smooth, &along)?;
        fade_out_all(stage, &along, STREAM_FADE)?;
        line.wait_for(stage, AT_ABOUT.1)?;
        let pointer = frame_around(stage, &[row_shares[0], percents[0]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_SLOSH.1)?;
        for k in 2..=SLOSH_TO {
            let along = streams(stage, &n.rounds[k - 1], &mix, size);
            crowd.round(stage, &n.rounds, k, ROUND_S, fmn::core::rate::smooth, &along)?;
            fade_out_all(stage, &along, STREAM_FADE)?;
        }
        kit.hold(stage)?;
        for k in SLOSH_TO + 1..=QUICK_TO {
            crowd.round(stage, &n.rounds, k, QUICK_S, fmn::core::rate::smooth, &[])?;
        }
        pause(stage, BEAT)?;

        // 11_6: the swings shrink and the crowd settles: ticks at the settled
        // heights, a dashed line through the trace; the rows' readouts become
        // B's share → the mix's
        let line = say(&kit, stage, "11_6")?;
        for k in QUICK_TO + 1..=n.last {
            crowd.round(stage, &n.rounds, k, FAST_S, fmn::core::rate::linear, &[])?;
        }
        line.wait_for(stage, AT_SETTLES.1)?;
        let mut marks: Vec<Mob> = (0..3).flat_map(|i| tick(stage, i, n.mix_w[i], false)).collect();
        let limit_y = trace.p(0.0, 100.0 * n.mix_w[0])[1];
        let limit_line = dashed(
            stage,
            v(TRACE_LEFT, limit_y),
            v(TRACE_RIGHT, limit_y),
            0.08,
            Palette::dim(),
            2.0,
        );
        let limit_label = kit.text_left(
            stage,
            &format!("{} {PERCENT}", n.mix_pct[0]),
            18.0,
            Palette::dim(),
            v(TRACE_RIGHT + TRACE_LABEL_DX, limit_y),
        )?;
        keep_between(
            stage,
            limit_label,
            TRACE_RIGHT,
            PANEL_RIGHT,
            "the settled share's label",
        )?;
        marks.extend(&limit_line);
        marks.push(limit_label);
        fade_in_all(stage, &marks, 0.6)?;
        let mut settled = Vec::with_capacity(3);
        for (i, words) in n.settled.iter().enumerate() {
            let m = kit.text_left(stage, words, 22.0, palette::player(), v(SHARE_X, ROW_YS[i]))?;
            keep_between(stage, m, SHARE_X - 0.01, PANEL_RIGHT, "a row's settled share")?;
            settled.push(m);
        }
        line.wait_for(stage, AT_KEEPS.1)?;
        let anims = swap(stage, &[row_shares[0], percents[0]], &[settled[0]], 0.5)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[settled[0]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_ONE_UP.1)?;
        let anims = swap(
            stage,
            &[row_shares[1], percents[1], row_shares[2], percents[2]],
            &[settled[1], settled[2]],
            0.5,
        )?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[settled[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_UP_FROM.1)?;
        flash_all(stage, &[ghosts[1]], 0.8)?;
        let mut panel_texts = vec![
            round_word,
            round_num,
            head[0],
            head[1],
            note,
            total,
            trace_head,
            rounds_axis,
            limit_label,
        ];
        panel_texts.extend(&names);
        panel_texts.extend(&settled);
        keep_apart(stage, &panel_texts, 0.06, "the settled panel")?;
        kit.hold(stage)?;

        // 11_7: game A's coin ignores the corners and stirs the crowd toward
        // equal thirds; the weight lost off corner 0 is outlined (held)
        let line = say(&kit, stage, "11_7")?;
        line.wait_for(stage, AT_IGNORES.1)?;
        let corners: Vec<Vec3> = (0..3).map(|i| corner(B_AT, i)).collect();
        rings_at(stage, &corners, CORNER_R + 0.05, 0.8)?;
        line.wait_for(stage, AT_THIRDS.1)?;
        let third = 1.0 / 3.0;
        let mut thirds: Vec<Mob> = (0..3).flat_map(|i| tick(stage, i, third, true)).collect();
        let third_y = trace.p(0.0, 100.0 * third)[1];
        thirds.extend(dashed(
            stage,
            v(TRACE_LEFT, third_y),
            v(TRACE_RIGHT, third_y),
            0.08,
            Palette::dim(),
            2.0,
        ));
        let third_label = kit.text_left(
            stage,
            &n.third,
            18.0,
            Palette::dim(),
            v(TRACE_RIGHT + TRACE_LABEL_DX, third_y),
        )?;
        keep_between(stage, third_label, TRACE_RIGHT, PANEL_RIGHT, "the third's label")?;
        keep_apart(stage, &[limit_label, third_label], 0.04, "the trace's two lines")?;
        thirds.push(third_label);
        fade_in_all(stage, &thirds, 0.6)?;
        line.wait_for(stage, AT_SHIFT.1)?;
        let (from, to) = (BAR_X + BAR_ALL * n.mix_w[0], BAR_X + BAR_ALL * n.b_w[0]);
        let lost = rect(
            v(from, ROW_YS[0] - BAR_H / 2.0),
            to - from,
            BAR_H,
            line_style(palette::bad(), 3.0),
        )?;
        let lost = stage.arena_mut().add(lost);
        fade_in_all(stage, &[lost], 0.4)?;
        flash_all(stage, &[lost], 0.8)?;
        ring_corners(stage, &[b.discs[0]], &[], 0.7)?;
        line.wait_for(stage, AT_WINNING.1)?;
        let up = now(stage);
        let pointer = frame_around(stage, &[settled[0]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[lost], 0.8)?;
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

    use super::super::draw::THINK_S;
    use super::super::one_player_to_a_crowd::LABEL_Y;
    use super::*;
    use crate::model::PUBLISHED;

    /// The chapter's words are true of its numbers: A's 50 % and B's 10 %
    /// average to 30 %, A's 50 % and B's 75 % to 62.5 %; the crowd starts at
    /// 10, 4, 12 of 26 (38.5, 15.4, 46.2 %) and settles at 34.6, 25.4,
    /// 40.1 %, from round 18 for good.
    #[test]
    fn mix_the_games_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(
            n.sums,
            [
                r"\tfrac{1}{2} \cdot 50\% + \tfrac{1}{2} \cdot 10\% = 30\%",
                r"\tfrac{1}{2} \cdot 50\% + \tfrac{1}{2} \cdot 75\% = 62.5\%",
            ]
        );
        // a fn, not a closure: a closure cannot return what it borrows
        // (root's native10 compile, Mail 6893)
        fn words(c: &[String; 3]) -> [&str; 3] {
            c.each_ref().map(String::as_str)
        }
        assert_eq!(words(&n.mix_round.heads), ["30", "62.5", "62.5"]);
        assert_eq!(words(&n.mix_round.tails), ["70", "37.5", "37.5"]);
        assert_eq!(words(&n.b_round.heads), ["10", "75", "75"]);
        assert_eq!(n.a_chance, "50 %");
        assert_eq!((n.n, n.counts), (26, [10, 4, 12]));
        assert_eq!(words(&shares_shown(&n.rounds[0], n.n)), ["38.5", "15.4", "46.2"]);
        assert_eq!(words(&n.mix_pct), ["34.6", "25.4", "40.1"]);
        assert_eq!(n.third, "33.3 %");
        assert_eq!(
            n.settled,
            ["38.5 \u{2192} 34.6 %", "15.4 \u{2192} 25.4 %", "46.2 \u{2192} 40.1 %"]
        );
        assert_eq!(n.last, 18);
        assert_eq!(n.rounds.len(), 19);
        assert_eq!(n.note, "shares shown to one decimal; the total is exactly 26");
    }

    /// Every round is computed from B's settled crowd: rounds 1 to 6 are
    /// the facts' exact `mixshift.round` shares, every round keeps all 26
    /// players, and the start matters. Failed controls: the same mix from
    /// 08's 26 on corner 0, or from equal thirds, gives other first rounds.
    #[test]
    fn every_round_comes_from_ten_four_twelve() {
        let n = numbers().unwrap();
        let f = facts::facts().unwrap();
        for k in 1..=SLOSH_TO {
            assert_eq!(
                Some(csv(shares_shown(&n.rounds[k], n.n)).as_str()),
                f.get(&format!("mixshift.round.{k}"))
            );
        }
        for r in &n.rounds {
            assert!((r.iter().sum::<f64>() - 26.0).abs() < 1e-9, "{r:?}");
        }
        let mix = ROUND.mix();
        let first = csv(shares_shown(&n.rounds[1], n.n));
        for other in [[26.0, 0.0, 0.0], [26.0 / 3.0; 3]] {
            let r = mix_rounds(other, &mix, 1);
            assert_ne!(csv(shares_shown(&r[1], n.n)), first, "from {other:?}");
        }
        // the float rounds stay on the exact ones to the settled round
        let exact = facts::rounds_from(n.counts.map(|c| Ratio::int(c as i64)), &mix, SLOSH_TO);
        for (k, v) in exact.iter().enumerate() {
            for (x, y) in v.iter().zip(&n.rounds[k + 1]) {
                assert!((x.to_f64() - y).abs() < 1e-9);
            }
        }
    }

    /// The crowd sloshes before it settles: corner 0 drops past nothing at
    /// once (round 1 already reads 34.6), then crosses its settled share at
    /// least twice in rounds 1 to 6 (five times); corner 1 overshoots to
    /// 28.8 first; corner 0's swings shrink every round from round 2; every
    /// readout settles for good from round 18 and not from round 17. Failed
    /// control: a monotone "drops then stays" picture would have no
    /// crossings.
    #[test]
    fn the_crowd_sloshes_before_it_settles() {
        let n = numbers().unwrap();
        assert_eq!(shares_shown(&n.rounds[1], n.n)[0], "34.6");
        assert_eq!(crossings(&n.rounds[..=SLOSH_TO], n.n, n.mix_w[0]), 5);
        assert_eq!(shares_shown(&n.rounds[1], n.n)[1], "28.8");
        assert!(n.rounds[1][1] / 26.0 > n.mix_w[1]);
        let look = mix_rounds(n.rounds[0], &ROUND.mix(), SETTLE_LOOK);
        assert_eq!(settled_from(&look, n.n, &n.mix_pct), Some(18));
        assert_ne!(shares_shown(&look[17], n.n), n.mix_pct);
        let monotone: Vec<[f64; 3]> = (0..=6)
            .map(|k| {
                let s = n.mix_w[0] + 0.01 / (k as f64 + 1.0);
                [26.0 * s, 0.0, 26.0 * (1.0 - s)]
            })
            .collect();
        assert_eq!(crossings(&monotone, n.n, n.mix_w[0]), 0);
    }

    /// The round-coin model is not the published run. With the published
    /// coins the mix settles at 34.5 % on corner 0, as the player's run of
    /// 05_2 measured (13_8's callback); with the round-number coins, 34.6 %.
    /// No number this chapter draws is 34.5. Failed controls: halfway
    /// between A's thirds and B's 38.5 % would be 35.9 %, not 34.6 %; game A
    /// alone keeps every corner at a third.
    #[test]
    fn this_mix_is_the_round_model_not_the_published_run() {
        let n = numbers().unwrap();
        let f = facts::facts().unwrap();
        assert_eq!(
            f.get("pub.mix.weights.pct").map(|s| s.split(',').next()),
            Some(Some("34.5"))
        );
        assert_eq!(f.get("run.mix.at0.pct"), Some("34.5"));
        assert_eq!(PUBLISHED.mix().shares()[0].pct(1), "34.5");
        assert_ne!(n.mix_pct[0], "34.5");
        let mut drawn: Vec<String> = n.sums.to_vec();
        drawn.extend(n.settled.iter().cloned());
        drawn.extend(n.mix_pct.iter().cloned());
        drawn.extend(n.mix_round.heads.iter().chain(&n.mix_round.tails).cloned());
        drawn.push(n.third.clone());
        drawn.push(n.round_badge.clone());
        assert!(drawn.iter().all(|s| !s.contains("34.5")), "{drawn:?}");
        let halfway = (Ratio::new(1, 3) + ROUND.game_b().shares()[0]) / Ratio::int(2);
        assert_eq!(halfway.pct(1), "35.9");
        assert_ne!(halfway.pct(1), n.mix_pct[0]);
        assert_eq!(ROUND.game_a().shares(), [Ratio::new(1, 3); 3]);
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: 04's coin and rules above the averages;
    /// the panel's rows, note, question, countdown and trace stack without
    /// touching; the bars, at their longest, end before the shares; every
    /// tick stays in its bar's reach; the trace's y range holds every round.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            assert!(MIX_NAME_AT[1] - 0.2 > MIX_COIN_AT[1] + MIX_COIN_R);
            assert!(RULE_YS[1] - 0.15 > SUM_YS[0] + 0.25 && SUM_YS[0] - SUM_YS[1] > 0.6);
            assert!(SUM_YS[1] - 0.25 > -FRAME_Y_RADIUS + 0.3);
            assert!(RULE_X > MIX_COIN_AT[0] + MIX_COIN_R + 0.1);
            assert!(ROUND_Y - PLAYERS_Y >= 0.4 && PLAYERS_Y - LABEL_Y >= 0.35);
            assert!(LABEL_Y - 0.15 > ROW_YS[0] + BAR_H / 2.0 + TICK_OUT);
            assert!(ROW_YS[2] - BAR_H / 2.0 - TICK_OUT > NOTE_Y + 0.1);
            assert!(NOTE_Y - 0.1 > QUESTION_Y + 0.15);
            assert!(QUESTION_Y - 0.15 > THINK_AT[1] + 0.62 + 0.1);
            assert!(NOTE_Y - 0.1 > TRACE_HEAD_Y + 0.1 && TRACE_HEAD_Y - 0.1 > TRACE_TOP);
            assert!(TRACE_BOTTOM - 0.35 > -FRAME_Y_RADIUS + 0.2);
            assert!(TRACE_RIGHT + TRACE_LABEL_DX + 0.6 < PANEL_RIGHT);
            assert!(PCT_X - SHARE_X > 0.45 && PCT_X + 0.25 < PANEL_RIGHT);
        }
        let n = numbers().unwrap();
        // the longest bar any round draws ends before the shares
        let longest = n
            .rounds
            .iter()
            .flat_map(|r| r.iter().map(|&x| x / 26.0))
            .chain(n.b_w)
            .fold(0.0, f64::max);
        assert!(BAR_X + BAR_ALL * longest < SHARE_X - 0.15, "{longest}");
        // the trace's range holds every round of corner 0 and both lines
        let (lo, hi) = (100.0 / 3.0 - TRACE_BELOW, 100.0 * n.b_w[0] + TRACE_ABOVE);
        for r in &n.rounds {
            let s = 100.0 * r[0] / 26.0;
            assert!(s > lo && s < hi, "{s}");
        }
        assert!(100.0 * n.mix_w[0] < hi && 100.0 / 3.0 > lo);
        // the lost weight is a box of positive width inside the ghost
        assert!(n.mix_w[0] < n.b_w[0]);
        // the panel stays left of B's corner 2
        assert!(PANEL_RIGHT < corner(B_AT, 2)[0] - CORNER_R - 0.2);
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 11 is said once, in order;
    /// the prediction (11_4) gets its silent think time; the silent rounds 7
    /// to 12 and a beat play between 11_5 and 11_6; the reveal stands.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = MixTheGames { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("11_mix_the_games: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=7).map(|k| format!("11_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("11_5") - end_of("11_4") >= THINK_S,
            "{} s to think",
            at("11_5") - end_of("11_4")
        );
        let silent = (QUICK_TO - SLOSH_TO) as f64 * QUICK_S + BEAT;
        assert!(
            at("11_6") - end_of("11_5") >= silent - 1e-9,
            "the silent rounds get {} s",
            at("11_6") - end_of("11_5")
        );
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        // the last picture stands (a breath, then `draw::STAND_UP` from its
        // last cue) and fades
        assert!(end - end_of("11_7") >= 1.0, "the reveal is not held: {end}");
    }
}
