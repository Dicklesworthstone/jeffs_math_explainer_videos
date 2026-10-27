//! Chapter 12, one dial decides game B. Still the round-number coins (the
//! outlined badge): the panel 11 left, "players: 26" over the dim "split by
//! the chances (expected numbers)", each corner's teal bar at the mix's
//! share with B's dim ghost outline and "38.5 → 34.6 %", and where the dial
//! will stand, "game A alone: breaks even" and "game B alone: breaks even".
//! A fair coin splits the rounds: every bar becomes a blue half (A rounds)
//! over a violet half (B rounds) of the same length (12_1). The coin never
//! looks at the money, so half of every corner's rounds are A rounds; A's
//! coin is the same everywhere ("A rounds: 50 % at every corner"), and the
//! blue halves dim: the winning is not there (12_2, a held beat).
//! Everything rests on the B rounds; one number decides their chance, the
//! share of them at corner 0, where a 75 % coin becomes a 10 % one ("B
//! rounds: 10 % at corner 0, 75 % elsewhere", 12_3). That share is a dial:
//! a scale from 32 % to 40 % at the right, "the dial: share of B's rounds at
//! corner 0"; game B alone sets it at 38.5 % (a violet mark, while corner
//! 0's ghost pulses, 12_4). In the mix, B's rounds sit where the crowd sits;
//! the weight the mix moved off corner 0 is outlined in red, and one tracker
//! turns the dial: a gold mark slides from 38.5 % to 34.6 % as a marker on
//! corner 0's bar slides from the ghost's end to the bar's (12_5). What is
//! that turn worth? The win axis "B wins"; at a third (a dim mark) B wins
//! 53.3 % (an ink point); a point slides along the line to B's own share,
//! exactly half, on the dashed 50 % line (12_6, a beat). In between it is a
//! straight line, 75 % − 65 % × share, at 50 % where share = 25/65 = 5/13,
//! the weight 09 found, which is why B alone breaks even (12_7). At the
//! mix's 34.6 %, what do B's rounds win? (12_8, a prediction: a gold dashed
//! marker, no value.) 52.5 %; with the A rounds' 50 %, 1/2 · 50 % + 1/2 ·
//! 52.5 % ≈ 51.3 % (12_9); "+2.54 cents a round" and a "precisely" card,
//! 2 · 727/1418 − 1 = 18/709 (12_10). A's rounds only break even; they move
//! the crowd off the bad corner and B's good coin does the winning (12_11).
//! Game A alone keeps a third at every corner (dashed thirds on the bars);
//! the mix pulls corner 0 most of the way there: an arrow on the dial from
//! 38.5 % to 34.6 %, dashed on to a third, "76 % of the way to a third"
//! (12_12, held).
//!
//! Every value comes from `crate::model` and is checked against
//! `crate::facts`; the scene refuses to draw if the words would stop being
//! true of them. The dial's mark and the panel's marker read one tracker,
//! so the dial and the crowd's corner 0 move together. The sums show one
//! decimal, so they say ≈; the card has the exact values. No typeset text
//! is indicated, scaled or recoloured: shapes pulse, words get a separate
//! outline, a changed label is replaced. The dial's helpers are shared with
//! chapter 13, which shaves the half point off the same picture.

use explainer_kit::props::precisely;
use explainer_kit::{Frame2, Kit, Palette, fade_all, rect, tween, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    arrow, dashed, fade_in_all, fade_out_all, fade_to, fail, flash_all, flat, keep_between, line_style, now, pause,
    read_s, say, stand, think, together, wait_until,
};
use super::looks_hopeless::{EDGE, FAINT, POINT_PAD, frame_around, point_at};
use super::one_player_to_a_crowd::{
    BAR_H, BAR_X, PANEL_X, PLAYERS_NUM_X, PLAYERS_Y, ROW_YS, panel_head, round_badge, round_badge_text, row_name,
};
use super::three_corners::{keep_apart, swap};
use crate::facts;
use crate::model::{self, Coins, ROUND, Ratio};
use crate::palette;

pub struct OneDial {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
pub(super) const PERCENT: &str = "%";
const ALONE_EVEN: [&str; 2] = ["game A alone: breaks even", "game B alone: breaks even"];
const A_ROUNDS: &str = "A rounds";
const B_ROUNDS: &str = "B rounds";
const EVERY_CORNER: &str = "at every corner";
const BAD_PLACE: &str = "at corner 0";
const ELSEWHERE: &str = "elsewhere";
pub(super) const DIAL_TITLE: &str = "the dial: share of B's rounds at corner 0";
pub(super) const WINS_AXIS: &str = "B wins";
pub(super) const B_ALONE: &str = "game B alone:";
pub(super) const MIX_WORDS: &str = "the mix:";
const THIRD_WORDS: &str = "a third:";
const ASK: &str = "what do B's rounds win?";
pub(super) const GAIN_WORDS: &str = "cents a round";
const TOWARD: &str = "of the way to a third";
const ETHIER_LEE: &str = "Ethier and Lee";
const MINUS: &str = "\u{2212}";

/// Where each picture lands: (line, the phrase it waits for).
const AT_STAY: (&str, &str) = ("12_1", "Stay with the round coins");
const AT_EVEN: (&str, &str) = ("12_1", "each break even");
const AT_SPLITS: (&str, &str) = ("12_1", "a fair coin splits the rounds");
const AT_LOOKS: (&str, &str) = ("12_2", "never looks at the money");
const AT_HALF_EACH: (&str, &str) = ("12_2", "half the rounds are A rounds");
const AT_A_COIN: (&str, &str) = ("12_2", "A's coin wins exactly half the time");
const AT_NOT_A: (&str, &str) = ("12_2", "can't be where the winning comes from");
const AT_RESTS: (&str, &str) = ("12_3", "Everything rests on the B rounds");
const AT_ONE_NUMBER: (&str, &str) = ("12_3", "just one number");
const AT_SWAPS: (&str, &str) = ("12_3", "swaps a seventy-five percent coin");
const AT_DIAL: (&str, &str) = ("12_4", "Think of that share as a dial");
const AT_B_SETS: (&str, &str) = ("12_4", "sets it at thirty-eight point five percent");
const AT_SIT: (&str, &str) = ("12_5", "B's rounds sit wherever the crowd sits");
const AT_MOVED: (&str, &str) = ("12_5", "moved the crowd off corner zero");
const AT_TURNS: (&str, &str) = ("12_5", "turns the dial down");
const AT_WORTH: (&str, &str) = ("12_6", "What is that turn worth");
const AT_A_THIRD: (&str, &str) = ("12_6", "With a third of B's rounds");
const AT_FIFTY_THREE: (&str, &str) = ("12_6", "fifty-three point three percent");
const AT_OWN: (&str, &str) = ("12_6", "At game B's own");
const AT_EXACTLY_HALF: (&str, &str) = ("12_6", "exactly half");
const AT_STRAIGHT: (&str, &str) = ("12_7", "it's a straight line");
const AT_FORMULA: (&str, &str) = ("12_7", "seventy-five percent, minus sixty-five percent");
const AT_THIRTEENTHS: (&str, &str) = ("12_7", "exactly five thirteenths");
const AT_WEIGHT: (&str, &str) = ("12_7", "the weight we found");
const AT_WHY: (&str, &str) = ("12_7", "That's why game B on its own breaks even");
const AT_AT_MIX: (&str, &str) = ("12_8", "At the mix's thirty-four point six percent");
const AT_WHAT: (&str, &str) = ("12_8", "what do B's rounds win");
const AT_ANSWER: (&str, &str) = ("12_9", "Fifty-two point five percent");
const AT_AVERAGED: (&str, &str) = ("12_9", "Averaged with the A rounds' fifty");
const AT_ALL: (&str, &str) = ("12_9", "of all its rounds");
const AT_GAIN: (&str, &str) = ("12_10", "a gain of just over two and a half cents a round");
const AT_ONLY: (&str, &str) = ("12_11", "game A's rounds only break even");
const AT_MOVE_OFF: (&str, &str) = ("12_11", "move the crowd off the bad corner");
const AT_GOOD: (&str, &str) = ("12_11", "game B's good coin does the winning");
const AT_A_KEEPS: (&str, &str) = ("12_12", "keeps a third of the rounds at every corner");
const AT_MOST: (&str, &str) = ("12_12", "most of the way");
#[cfg(test)]
const CUES: [(&str, &str); 36] = [
    AT_STAY,
    AT_EVEN,
    AT_SPLITS,
    AT_LOOKS,
    AT_HALF_EACH,
    AT_A_COIN,
    AT_NOT_A,
    AT_RESTS,
    AT_ONE_NUMBER,
    AT_SWAPS,
    AT_DIAL,
    AT_B_SETS,
    AT_SIT,
    AT_MOVED,
    AT_TURNS,
    AT_WORTH,
    AT_A_THIRD,
    AT_FIFTY_THREE,
    AT_OWN,
    AT_EXACTLY_HALF,
    AT_STRAIGHT,
    AT_FORMULA,
    AT_THIRTEENTHS,
    AT_WEIGHT,
    AT_WHY,
    AT_AT_MIX,
    AT_WHAT,
    AT_ANSWER,
    AT_AVERAGED,
    AT_ALL,
    AT_GAIN,
    AT_ONLY,
    AT_MOVE_OFF,
    AT_GOOD,
    AT_A_KEEPS,
    AT_MOST,
];

// ------------------------------------------------------------------ layout

/// 11's scale: a bar this long is the whole crowd; the rows' settled shares
/// start at `SHARE_X`; the panel keeps left of `PANEL_RIGHT`.
const BAR_ALL: f64 = 6.0;
const SHARE_X: f64 = -2.3;
pub(super) const PANEL_RIGHT: f64 = -0.7;
/// A tick across a bar reaches this far above and below it (11's).
const TICK_OUT: f64 = 0.07;
/// Under the rows: the two kinds of round; the panel's sums; the
/// prediction's question.
const LEGEND_YS: [f64; 2] = [0.15, -0.25];
const TEX_YS: [f64; 2] = [-0.85, -1.5];
const QUESTION_Y: f64 = -2.1;
/// 12_1's reminders, where the dial will stand.
const ALONE_YS: [f64; 2] = [1.3, 0.75];

/// The dial: B's share of rounds at corner 0 across, B's chance of winning
/// up, both in percent.
pub(super) const SCALE: (i32, i32) = (32, 40);
pub(super) const WIN_RANGE: (f64, f64) = (48.0, 56.0);
pub(super) const DIAL: Frame2 = Frame2 {
    center: [3.5, 0.8, 0.0],
    width: 5.6,
    height: 3.4,
    x: (SCALE.0 as f64, SCALE.1 as f64),
    y: WIN_RANGE,
};
pub(super) const DIAL_TITLE_Y: f64 = 2.95;
/// The win axis's words end here, left of the axis.
pub(super) const AXIS_WORDS_RIGHT: f64 = 0.55;
/// Below the share axis: its ends' percentages, then two rows of marks'
/// words.
pub(super) const BELOW: [f64; 3] = [0.3, 0.65, 1.0];
/// A scale tick and a mark cross the axis this far each way; a point's
/// radius; a point's words stand this far right and up from it.
pub(super) const TICK: f64 = 0.06;
pub(super) const MARK_H: f64 = 0.16;
pub(super) const POINT_R: f64 = 0.09;
pub(super) const POINT_WORDS: (f64, f64) = (0.12, 0.25);
/// The prediction's countdown, in the dial's empty top right.
pub(super) const THINK_AT: Vec3 = [5.3, 1.75, 0.0];
#[cfg(test)]
pub(super) const THINK_R: f64 = 0.62;
/// 12_12's arrow runs this far above the share axis, its words above it.
const ARROW_DY: f64 = 0.25;
const ARROW_WORDS_DY: f64 = 0.5;
/// Seconds: the dial's turn, the point's slide; a beat after a thought.
const TURN_S: f64 = 1.4;
const SLIDE_S: f64 = 1.6;
pub(super) const BEAT: f64 = 1.0;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    round_badge: String,
    players: String,
    b_w: [f64; 3],
    mix_w: [f64; 3],
    /// Each row's settled line, "38.5 → 34.6 %".
    settled: [String; 3],
    /// "A rounds: 50 % at every corner", "B rounds: 10 % at corner 0, …".
    legend: [String; 2],
    /// The dial's places in percent: a third, B alone, the mix.
    shares: [f64; 3],
    /// B's line, `top` − `slope` × share, in percent; where B breaks even.
    top: f64,
    slope: f64,
    even: f64,
    /// The marks' words: B alone, the mix, a third.
    marks: [String; 3],
    /// B's chance at a third and at the mix's share, in percent, and their
    /// words.
    wins: [f64; 2],
    win_words: [String; 2],
    even_words: String,
    /// The line, where it is even, and the mix's average.
    tex: [String; 3],
    gain: String,
    k05: String,
    question: String,
    toward: String,
}

fn csv<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    values.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

/// A chance as the scene prints it: exact if it is a whole number of
/// tenths of a percent, else to one decimal.
fn shown_pct(x: Ratio) -> String {
    x.exact_pct().unwrap_or_else(|_| x.pct(1))
}

/// A printed percentage back as a chance: "52.5" → 21/40 (and printed
/// cents back as dollars: "+2.54" → 127/5000).
pub(super) fn pct_value(text: &str) -> Result<Ratio, String> {
    let (whole, dec) = text.split_once('.').unwrap_or((text, ""));
    let digits: i128 = format!("{whole}{dec}")
        .parse()
        .map_err(|_| format!("{text} is not a percentage"))?;
    let places = u32::try_from(dec.len()).map_err(|_| format!("{text} has too many places"))?;
    Ok(Ratio::new(digits, 100 * 10i128.pow(places)))
}

/// "1/2 · a % + 1/2 · m % ≈ m %": the mix's chance as the average of its A
/// rounds' and B rounds', each printed as the scene prints it. "=" only if
/// every printed value is exact; the printed terms must round to the
/// printed result.
pub(super) fn average_tex(a: Ratio, b: Ratio, m: Ratio) -> Result<String, String> {
    if (a + b) / Ratio::int(2) != m {
        return Err(format!("{m} is not halfway between {a} and {b}"));
    }
    let [ta, tb, tm] = [a, b, m].map(shown_pct);
    let from_shown = (pct_value(&ta)? + pct_value(&tb)?) / Ratio::int(2);
    if from_shown.pct(1) != m.pct(1) {
        return Err(format!("{ta} % and {tb} % do not average to {tm} %"));
    }
    let exact = [a, b, m].iter().all(|x| x.exact_pct().is_ok());
    let op = if exact { "=" } else { r"\approx" };
    Ok(format!(
        r"\tfrac{{1}}{{2}} \cdot {ta}\% + \tfrac{{1}}{{2}} \cdot {tb}\% {op} {tm}\%"
    ))
}

/// B's line for coins `c`, "B wins = 75 % − 65 % × share", and where it is
/// 50 %, "share = 25/65 = 5/13 ≈ 38.5 %", from the coins: the good coin, the
/// gap down to the bad coin, and the good coin's lead over a fair one.
pub(super) fn line_tex(c: &Coins) -> Result<[String; 2], String> {
    let [_, bad, good] = facts::coins(c);
    let half = Ratio::new(1, 2);
    let even = facts::fair_share(c);
    let (top, slope, rise) = (good.exact_pct()?, (good - bad).exact_pct()?, (good - half).exact_pct()?);
    if pct_value(&rise)? / pct_value(&slope)? != even || even.exact_pct().is_ok() {
        return Err(format!("{rise}/{slope} is not the even share {even}"));
    }
    Ok([
        format!(r"\text{{B wins}} = {top}\% - {slope}\% \times \text{{share}}"),
        format!(
            r"{}\% \text{{ at share}} = \tfrac{{{rise}}}{{{slope}}} = \tfrac{{{}}}{{{}}} \approx {}\%",
            half.exact_pct()?,
            even.num(),
            even.den(),
            even.pct(1)
        ),
    ])
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let (a, b, mix) = (ROUND.game_a(), ROUND.game_b(), ROUND.mix());
    let [a_coin, bad, good] = facts::coins(&ROUND);
    let (half, third) = (Ratio::new(1, 2), Ratio::new(1, 3));
    // 12_1: alone, each round-coin game breaks even
    if a.mean() != Ratio::ZERO || b.mean() != Ratio::ZERO {
        return Err("12_1 says game A and game B each break even alone".into());
    }
    // 12_2: a fair coin picks the game without looking at the money: the
    // mix's chances are A's and B's averaged at every corner; A's coin is
    // the same fair coin wherever it is tossed
    let fair = a.mixed(&b, 1, 2);
    if (0..3).any(|i| mix.chance(i) != fair.chance(i)) {
        return Err("12_2 says a fair coin splits the rounds".into());
    }
    let mix_w_r = mix.shares();
    let a_in_mix = a.win_at(&mix_w_r);
    if (0..3).any(|i| a.chance(i) != half) || a_in_mix != half || get("round.mix.a_rounds.win")? != half.to_string() {
        return Err("12_2 says A's coin wins exactly half the time wherever it's tossed".into());
    }
    let heads: Vec<String> = [a_coin, bad, good]
        .iter()
        .map(|x| x.exact_pct())
        .collect::<Result<_, _>>()?;
    if get("coins.round.heads.pct")? != heads.join(",") {
        return Err("the coins disagree with the facts".into());
    }
    // 12_3: B's chance is one line in its share at corner 0, however the
    // other two corners split the rest
    let line_at = |s: Ratio| good - (good - bad) * s;
    for s in [Ratio::ZERO, third, Ratio::new(5, 13), mix_w_r[0], Ratio::ONE] {
        for split in [Ratio::ZERO, Ratio::new(1, 7), half, Ratio::ONE] {
            let rest = Ratio::ONE - s;
            if b.win_at(&[s, rest * split, rest - rest * split]) != line_at(s) {
                return Err("12_3 says one number decides B's chance".into());
            }
        }
    }
    // 12_4 and 12_5: B alone sets the dial at 38.5 %; in the mix, B's
    // rounds sit where the crowd sits, which the mix moved off corner 0
    let size = model::smallest_whole_crowd(&b);
    if get("crowd.size")? != size.to_string() {
        return Err("the crowd is not the facts' 26".into());
    }
    let b_w_r = b.shares();
    let b_pct = b_w_r.map(|w| w.pct(1));
    let mix_pct = mix_w_r.map(|w| w.pct(1));
    if get("round.b.weights.pct")? != csv(&b_pct)
        || get("crowd.share0.pct")? != b_pct[0]
        || get("round.mix.weights.pct")? != csv(&mix_pct)
    {
        return Err("the settled shares disagree with the facts".into());
    }
    if mix_w_r[0] >= b_w_r[0] {
        return Err("12_5 says the mix turns the dial down".into());
    }
    let b_in_mix = b.win_at(&mix_w_r);
    if mix.win() != (a_in_mix + b_in_mix) / Ratio::int(2) {
        return Err("the mix's rounds are not half A rounds and half B rounds".into());
    }
    // 12_6: at a third, 53.3 %; at B's own share, exactly half
    let at_third = facts::thirds_win(&ROUND);
    if at_third != line_at(third)
        || at_third.pct(1) != get("round.b.thirds.win.pct")?
        || third.pct(1) != get("thirds.pct")?
    {
        return Err("12_6 says fifty-three point three percent at a third".into());
    }
    if line_at(b_w_r[0]) != half || b.win() != half || get("round.b.win")? != half.to_string() {
        return Err("12_6 says exactly half at B's own share".into());
    }
    // 12_7: the straight line, and where it is even: B's own weight
    let [tex_line, tex_even] = line_tex(&ROUND)?;
    if get("round.b.line.pct")? != format!("{},{}", good.exact_pct()?, (good - bad).exact_pct()?) {
        return Err("12_7's line disagrees with the facts".into());
    }
    let even = facts::fair_share(&ROUND);
    if even != b_w_r[0] || get("round.b.fair_share")? != even.to_string() || even != Ratio::new(5, 13) {
        return Err("12_7 says it breaks even at five thirteenths, B's own weight".into());
    }
    // 12_8 and 12_9: at the mix's share B's rounds win 52.5 %, between the
    // two points; with the A rounds' 50 %, the mix wins 51.3 %
    if b_in_mix != line_at(mix_w_r[0]) || b_in_mix.pct(1) != get("round.mix.b_rounds.win.pct")? {
        return Err("12_9 says fifty-two point five percent".into());
    }
    if b_in_mix <= half || b_in_mix >= at_third {
        return Err("12_8's answer is not between the two points".into());
    }
    let mix_win = mix.win();
    if mix_win.pct(1) != get("round.mix.win.pct")? || mix_win.to_string() != get("round.mix.win")? {
        return Err("12_9 says fifty-one point three percent".into());
    }
    let tex_sum = average_tex(a_in_mix, b_in_mix, mix_win)?;
    // 12_10: just over two and a half cents a round, exactly 2 × 727/1418 −
    // 1 = 18/709 (Ethier and Lee's value)
    let mean = mix.mean();
    if mean != Ratio::int(2) * mix_win - Ratio::ONE
        || mean.to_string() != get("round.mix.mean")?
        || mean.cents(2) != get("round.mix.mean.cents")?
    {
        return Err("12_10's gain disagrees with the facts".into());
    }
    if mean <= Ratio::new(25, 1000) || mean >= Ratio::new(26, 1000) {
        return Err("12_10 says just over two and a half cents".into());
    }
    // 12_12: A alone keeps a third at every corner; the mix moves corner 0
    // most of the way from B's share to it
    if a.shares() != [third; 3] {
        return Err("12_12 says game A keeps a third at every corner".into());
    }
    let toward = (b_w_r[0] - mix_w_r[0]) / (b_w_r[0] - third);
    if toward.pct(0) != get("round.mix.toward_thirds.pct")? || toward <= half || mix_w_r[0] <= third {
        return Err("12_12 says most of the way to a third".into());
    }
    let pct = |x: Ratio| 100.0 * x.to_f64();
    Ok(Numbers {
        round_badge: round_badge_text(&f)?,
        players: size.to_string(),
        b_w: b_w_r.map(Ratio::to_f64),
        mix_w: mix_w_r.map(Ratio::to_f64),
        settled: [0, 1, 2].map(|i| format!("{} \u{2192} {} {PERCENT}", b_pct[i], mix_pct[i])),
        legend: [
            format!("{A_ROUNDS}: {} {PERCENT} {EVERY_CORNER}", heads[0]),
            format!(
                "{B_ROUNDS}: {} {PERCENT} {BAD_PLACE}, {} {PERCENT} {ELSEWHERE}",
                heads[1], heads[2]
            ),
        ],
        shares: [pct(third), pct(b_w_r[0]), pct(mix_w_r[0])],
        top: pct(good),
        slope: pct(good - bad),
        even: pct(half),
        marks: [
            format!("{B_ALONE} {} {PERCENT}", b_pct[0]),
            format!("{MIX_WORDS} {} {PERCENT}", mix_pct[0]),
            format!("{THIRD_WORDS} {} {PERCENT}", third.pct(1)),
        ],
        wins: [pct(at_third), pct(b_in_mix)],
        win_words: [
            format!("{} {PERCENT}", at_third.pct(1)),
            format!("{} {PERCENT}", b_in_mix.pct(1)),
        ],
        even_words: format!("{} {PERCENT}", half.exact_pct()?),
        tex: [tex_line, tex_even, tex_sum],
        gain: format!("{} {GAIN_WORDS}", mean.cents(2)),
        k05: format!(
            "exactly: the mix wins {mix_win} of its rounds, and every round wins or loses one dollar, so it \
             gains 2 \u{b7} {mix_win} {MINUS} 1 = {mean} of a dollar a round ({ETHIER_LEE} also give {mean})"
        ),
        question: format!("at {} {PERCENT}, {ASK}", mix_pct[0]),
        toward: format!("{} {PERCENT} {TOWARD}", toward.pct(0)),
    })
}

// ------------------------------------------------------ the dial (12, 13)

/// B's chance in percent on the line `top` − `slope` × share, at a share of
/// `share` percent.
pub(super) fn on_line(top: f64, slope: f64, share: f64) -> f64 {
    top - slope * share / 100.0
}

/// The dial's share axis: the line, a tick at every point, its ends'
/// percentages and the title. Detached.
pub(super) fn share_axis(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let (x0, x1) = DIAL.x;
    let y0 = WIN_RANGE.0;
    let axis = polyline([DIAL.p(x0, y0), DIAL.p(x1, y0)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut parts = vec![stage.arena_mut().add(axis)];
    for p in SCALE.0..=SCALE.1 {
        let c = DIAL.p(f64::from(p), y0);
        let tick = polyline([v(c[0], c[1] - TICK), v(c[0], c[1] + TICK)])
            .style(line_style(Palette::dim(), 1.5))
            .build();
        parts.push(stage.arena_mut().add(tick));
    }
    for p in [SCALE.0, SCALE.1] {
        let c = DIAL.p(f64::from(p), y0);
        let end = kit.text(
            stage,
            &format!("{p} {PERCENT}"),
            18.0,
            Palette::dim(),
            v(c[0], c[1] - BELOW[0]),
        )?;
        keep_between(stage, end, PANEL_RIGHT, FRAME_X_RADIUS - EDGE, "an end of the dial")?;
        parts.push(end);
    }
    let title = kit.text(stage, DIAL_TITLE, 20.0, Palette::dim(), v(DIAL.center[0], DIAL_TITLE_Y))?;
    keep_between(stage, title, PANEL_RIGHT, FRAME_X_RADIUS - EDGE, "the dial's title")?;
    parts.push(title);
    Ok(parts)
}

/// The dial's win axis and its words. Detached.
pub(super) fn win_axis(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    let x0 = DIAL.x.0;
    let axis = polyline([DIAL.p(x0, WIN_RANGE.0), DIAL.p(x0, WIN_RANGE.1)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let axis = stage.arena_mut().add(axis);
    let words = kit.text(stage, WINS_AXIS, 20.0, Palette::dim(), ORIGIN)?;
    stage.move_to(words, v(AXIS_WORDS_RIGHT, DIAL.top()), RIGHT);
    keep_between(stage, words, PANEL_RIGHT, DIAL.left(), "the win axis's words")?;
    Ok([axis, words])
}

/// The dashed line where B breaks even, at `even` percent, and its words
/// left of the win axis. Detached.
pub(super) fn even_line(kit: &Kit, stage: &mut Stage<'_>, even: f64, words: &str) -> fmn::Result<Vec<Mob>> {
    let (x0, x1) = DIAL.x;
    let mut parts = dashed(stage, DIAL.p(x0, even), DIAL.p(x1, even), 0.08, Palette::dim(), 2.0);
    let m = kit.text(stage, words, 18.0, Palette::dim(), ORIGIN)?;
    stage.move_to(m, v(AXIS_WORDS_RIGHT, DIAL.p(x0, even)[1]), RIGHT);
    keep_between(stage, m, PANEL_RIGHT, DIAL.left(), "the even line's words")?;
    parts.push(m);
    Ok(parts)
}

/// A tick across the share axis at `share` percent. Detached.
pub(super) fn mark_tick(stage: &mut Stage<'_>, share: f64, color: Srgb) -> Mob {
    let c = DIAL.p(share, WIN_RANGE.0);
    stage.arena_mut().add(
        polyline([v(c[0], c[1] - MARK_H), v(c[0], c[1] + MARK_H)])
            .style(line_style(color, 4.0))
            .build(),
    )
}

/// A mark's words under the share axis at `share` percent, in row `row` of
/// [`BELOW`]. Detached.
pub(super) fn mark_words(
    kit: &Kit,
    stage: &mut Stage<'_>,
    share: f64,
    color: Srgb,
    words: &str,
    row: usize,
) -> fmn::Result<Mob> {
    let c = DIAL.p(share, WIN_RANGE.0);
    let m = kit.text(stage, words, 18.0, color, v(c[0], c[1] - BELOW[row]))?;
    keep_between(stage, m, PANEL_RIGHT, FRAME_X_RADIUS - EDGE, "a mark's words")?;
    Ok(m)
}

/// A point on the dial at (`share`, `win`) percent. Detached.
pub(super) fn point(stage: &mut Stage<'_>, share: f64, win: f64, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(POINT_R)
            .arc_center(DIAL.p(share, win))
            .style(flat(color, 1.0))
            .build(),
    )
}

/// A point's words, just above and right of it. Detached.
pub(super) fn point_words(
    kit: &Kit,
    stage: &mut Stage<'_>,
    share: f64,
    win: f64,
    color: Srgb,
    words: &str,
) -> fmn::Result<Mob> {
    let c = DIAL.p(share, win);
    let m = kit.text_left(stage, words, 18.0, color, v(c[0] + POINT_WORDS.0, c[1] + POINT_WORDS.1))?;
    keep_between(stage, m, DIAL.left(), FRAME_X_RADIUS - EDGE, "a point's words")?;
    Ok(m)
}

/// B's line across the dial, `top` − `slope` × share. Detached.
pub(super) fn b_line(stage: &mut Stage<'_>, top: f64, slope: f64, style: Style) -> Mob {
    let (x0, x1) = DIAL.x;
    stage.arena_mut().add(
        polyline([DIAL.p(x0, on_line(top, slope, x0)), DIAL.p(x1, on_line(top, slope, x1))])
            .style(style)
            .build(),
    )
}

/// A dashed upright on the dial at `share` percent from the share axis to
/// `win` percent. Detached.
pub(super) fn upright(stage: &mut Stage<'_>, share: f64, win: f64, color: Srgb) -> Vec<Mob> {
    dashed(stage, DIAL.p(share, WIN_RANGE.0), DIAL.p(share, win), 0.07, color, 2.0)
}

// --------------------------------------------------------------- the panel

/// Corner `i`'s bar in the panel, `share` of the crowd long, from `dy` above
/// the row's middle and `h` high. Detached.
fn bar(stage: &mut Stage<'_>, i: usize, share: f64, dy: f64, h: f64, style: Style) -> fmn::Result<Mob> {
    let r = rect(v(BAR_X, ROW_YS[i] + dy), BAR_ALL * share, h, style)?;
    Ok(stage.arena_mut().add(r))
}

/// A dashed tick across row `i`'s bar at `share` of the crowd (11's).
/// Detached.
fn dashed_tick(stage: &mut Stage<'_>, i: usize, share: f64) -> Vec<Mob> {
    let x = BAR_X + BAR_ALL * share;
    dashed(
        stage,
        v(x, ROW_YS[i] - BAR_H / 2.0 - TICK_OUT),
        v(x, ROW_YS[i] + BAR_H / 2.0 + TICK_OUT),
        0.06,
        Palette::dim(),
        2.0,
    )
}

/// A line of the panel, its left edge at [`PANEL_X`], kept left of
/// [`PANEL_RIGHT`]. Detached.
fn panel_line(kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, y: f64) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, words, size, color, v(PANEL_X, y))?;
    keep_between(stage, m, PANEL_X - 0.01, PANEL_RIGHT, words)?;
    Ok(m)
}

/// A sum in the panel at height `y`. Detached.
fn panel_tex(kit: &Kit, stage: &mut Stage<'_>, src: &str, y: f64) -> fmn::Result<Mob> {
    let m = kit.tex(stage, src, 26.0, Palette::ink(), &[], ORIGIN)?;
    stage.move_to(m, v(PANEL_X, y), LEFT);
    keep_between(stage, m, PANEL_X - 0.01, PANEL_RIGHT, "a sum in the panel")?;
    Ok(m)
}

impl SceneConstruct for OneDial {
    fn name(&self) -> &str {
        "12_one_dial"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let [third, b_share, mix_share] = n.shares;
        let (top, slope) = (n.top, n.slope);

        // 12_1: 11's panel and the badge; where the dial will stand, what
        // each game does alone; a fair coin splits every bar into A's half
        // over B's
        let line = say(&kit, stage, "12_1")?;
        let tag = round_badge(&kit, stage, &n.round_badge)?;
        let head = panel_head(&kit, stage)?;
        let players = kit.text_left(stage, &n.players, 22.0, Palette::ink(), v(PLAYERS_NUM_X, PLAYERS_Y))?;
        let names: Vec<Mob> = (0..3).map(|i| row_name(&kit, stage, i)).collect::<fmn::Result<_>>()?;
        let mut bars = Vec::with_capacity(3);
        let mut ghosts = Vec::with_capacity(3);
        let mut settled = Vec::with_capacity(3);
        for (i, words) in n.settled.iter().enumerate() {
            bars.push(bar(
                stage,
                i,
                n.mix_w[i],
                -BAR_H / 2.0,
                BAR_H,
                flat(palette::player(), 0.85),
            )?);
            ghosts.push(bar(
                stage,
                i,
                n.b_w[i],
                -BAR_H / 2.0,
                BAR_H,
                line_style(Palette::dim(), 1.5),
            )?);
            let m = kit.text_left(stage, words, 22.0, palette::player(), v(SHARE_X, ROW_YS[i]))?;
            keep_between(stage, m, SHARE_X - 0.01, PANEL_RIGHT, "a row's settled share")?;
            settled.push(m);
        }
        let mut arriving = vec![tag[0], tag[1], head[0], head[1], players];
        arriving.extend(&names);
        arriving.extend(&bars);
        arriving.extend(&ghosts);
        arriving.extend(&settled);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, AT_STAY.1)?;
        let pointer = frame_around(stage, &tag[..1], 0.0, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_EVEN.1)?;
        let colors = [palette::game_a(), palette::game_b()];
        let mut alone = Vec::with_capacity(2);
        for ((words, color), y) in ALONE_EVEN.iter().zip(colors).zip(ALONE_YS) {
            let m = kit.text(stage, words, 22.0, color, v(DIAL.center[0], y))?;
            keep_between(stage, m, PANEL_RIGHT, FRAME_X_RADIUS - EDGE, "a game alone")?;
            alone.push(m);
        }
        fade_in_all(stage, &alone, 0.5)?;
        line.wait_for(stage, AT_SPLITS.1)?;
        let mut a_half = Vec::with_capacity(3);
        let mut b_half = Vec::with_capacity(3);
        for (i, &w) in n.mix_w.iter().enumerate() {
            a_half.push(bar(stage, i, w, 0.0, BAR_H / 2.0, flat(palette::game_a(), 0.85))?);
            b_half.push(bar(
                stage,
                i,
                w,
                -BAR_H / 2.0,
                BAR_H / 2.0,
                flat(palette::game_b(), 0.85),
            )?);
        }
        let kinds = [
            panel_line(&kit, stage, A_ROUNDS, 18.0, palette::game_a(), LEGEND_YS[0])?,
            panel_line(&kit, stage, B_ROUNDS, 18.0, palette::game_b(), LEGEND_YS[1])?,
        ];
        let mut coming = a_half.clone();
        coming.extend(&b_half);
        coming.extend(kinds);
        let anims = swap(stage, &bars, &coming, 0.7)?;
        stage.play_prepared(anims)?;

        // 12_2: the coin never looks at the money: half of every corner's
        // rounds are A rounds; A's coin is fair everywhere, so the winning
        // is not in the A rounds (a held beat)
        let line = say(&kit, stage, "12_2")?;
        line.wait_for(stage, AT_LOOKS.1)?;
        let mut halves = a_half.clone();
        halves.extend(&b_half);
        let pointer = frame_around(stage, &halves, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_HALF_EACH.1)?;
        flash_all(stage, &a_half, 0.6)?;
        flash_all(stage, &b_half, 0.6)?;
        line.wait_for(stage, AT_A_COIN.1)?;
        let a_kind = panel_line(&kit, stage, &n.legend[0], 18.0, palette::game_a(), LEGEND_YS[0])?;
        let anims = swap(stage, &kinds[..1], &[a_kind], 0.5)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[a_kind], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_NOT_A.1)?;
        let dims = a_half
            .iter()
            .map(|&m| fade_to(m, FAINT, 0.6))
            .collect::<fmn::Result<Vec<_>>>()?;
        together(stage, dims)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 12_3: everything rests on the B rounds; one number decides them,
        // their share at corner 0, where 75 % becomes 10 %
        let line = say(&kit, stage, "12_3")?;
        line.wait_for(stage, AT_RESTS.1)?;
        flash_all(stage, &b_half, 0.8)?;
        line.wait_for(stage, AT_ONE_NUMBER.1)?;
        let pointer = frame_around(stage, &b_half[..1], 0.05, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_SWAPS.1)?;
        let b_kind = panel_line(&kit, stage, &n.legend[1], 18.0, palette::game_b(), LEGEND_YS[1])?;
        let anims = swap(stage, &kinds[1..], &[b_kind], 0.5)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[b_kind], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;

        // 12_4: that share is a dial; B alone sets it at its weight
        let line = say(&kit, stage, "12_4")?;
        line.wait_for(stage, AT_DIAL.1)?;
        let scale = share_axis(&kit, stage)?;
        let anims = swap(stage, &alone, &scale, 0.6)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_B_SETS.1)?;
        let b_mark = [
            mark_tick(stage, b_share, palette::game_b()),
            mark_words(&kit, stage, b_share, palette::game_b(), &n.marks[0], 1)?,
        ];
        fade_in_all(stage, &b_mark, 0.5)?;
        flash_all(stage, &ghosts[..1], 0.7)?;

        // 12_5: B's rounds sit where the crowd sits; the mix moved weight
        // off corner 0, and one tracker turns the dial and moves the
        // marker on corner 0's bar
        let line = say(&kit, stage, "12_5")?;
        line.wait_for(stage, AT_SIT.1)?;
        let pointer = frame_around(stage, &b_half, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_MOVED.1)?;
        let (from, to) = (BAR_X + BAR_ALL * n.mix_w[0], BAR_X + BAR_ALL * n.b_w[0]);
        let lost = rect(
            v(from, ROW_YS[0] - BAR_H / 2.0),
            to - from,
            BAR_H,
            line_style(palette::bad(), 3.0),
        )?;
        let lost = stage.arena_mut().add(lost);
        fade_in_all(stage, &[lost], 0.4)?;
        flash_all(stage, &[lost], 0.7)?;
        line.wait_for(stage, AT_TURNS.1)?;
        let dial = stage.add_value_tracker(n.b_w[0]);
        let marker = stage.always_redraw(move |s| {
            let x = BAR_X + BAR_ALL * s.tracker_value(dial).unwrap_or(0.0);
            let y = ROW_YS[0];
            s.add(
                polyline([v(x, y - BAR_H / 2.0 - TICK_OUT), v(x, y + BAR_H / 2.0 + TICK_OUT)])
                    .style(line_style(Palette::hero(), 3.0))
                    .build(),
            )
        });
        let needle = stage.always_redraw(move |s| {
            let c = DIAL.p(100.0 * s.tracker_value(dial).unwrap_or(0.0), WIN_RANGE.0);
            s.add(
                polyline([v(c[0], c[1] - MARK_H), v(c[0], c[1] + MARK_H)])
                    .style(line_style(palette::mix(), 4.0))
                    .build(),
            )
        });
        stage.add_to_scene(marker)?;
        stage.add_to_scene(needle)?;
        stage.play(tween(dial, n.mix_w[0], TURN_S)?)?;
        let mix_words = mark_words(&kit, stage, mix_share, palette::mix(), &n.marks[1], 1)?;
        fade_in_all(stage, &[mix_words], 0.4)?;
        fade_out_all(stage, &[marker], 0.4)?;
        pause(stage, BEAT)?;

        // 12_6: what the turn is worth: the win axis; at a third, 53.3 %;
        // along the line to B's own share, exactly half (a beat)
        let line = say(&kit, stage, "12_6")?;
        line.wait_for(stage, AT_WORTH.1)?;
        let wins = win_axis(&kit, stage)?;
        fade_in_all(stage, &wins, 0.5)?;
        line.wait_for(stage, AT_A_THIRD.1)?;
        let mut third_mark = vec![
            mark_tick(stage, third, Palette::dim()),
            mark_words(&kit, stage, third, Palette::dim(), &n.marks[2], 2)?,
        ];
        third_mark.extend(upright(stage, third, n.wins[0], Palette::dim()));
        fade_in_all(stage, &third_mark, 0.5)?;
        line.wait_for(stage, AT_FIFTY_THREE.1)?;
        let third_point = [
            point(stage, third, n.wins[0], Palette::ink()),
            point_words(&kit, stage, third, n.wins[0], Palette::ink(), &n.win_words[0])?,
        ];
        fade_in_all(stage, &third_point, 0.4)?;
        line.wait_for(stage, AT_OWN.1)?;
        let slide = stage.add_value_tracker(third);
        let start = DIAL.p(third, on_line(top, slope, third));
        let trail = stage.always_redraw(move |s| {
            let x = s.tracker_value(slide).unwrap_or(third);
            s.add(
                polyline([start, DIAL.p(x, on_line(top, slope, x))])
                    .style(line_style(palette::game_b(), 3.0))
                    .build(),
            )
        });
        let mover = stage.always_redraw(move |s| {
            let x = s.tracker_value(slide).unwrap_or(third);
            s.add(
                Circle::new()
                    .radius(POINT_R)
                    .arc_center(DIAL.p(x, on_line(top, slope, x)))
                    .style(flat(Palette::ink(), 1.0))
                    .build(),
            )
        });
        stage.add_to_scene(trail)?;
        stage.add_to_scene(mover)?;
        stage.play(tween(slide, b_share, SLIDE_S)?)?;
        line.wait_for(stage, AT_EXACTLY_HALF.1)?;
        let even = even_line(&kit, stage, n.even, &n.even_words)?;
        let b_point = point(stage, b_share, n.even, palette::game_b());
        let mut arriving = even.clone();
        arriving.push(b_point);
        fade_in_all(stage, &arriving, 0.5)?;
        let even_words = *even.last().expect("the even line has words");
        let pointer = frame_around(stage, &[even_words], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 12_7: in between, a straight line: 75 % − 65 % × share, at 50 %
        // where share = 5/13, the weight of corner 0, so B alone is even
        let line = say(&kit, stage, "12_7")?;
        line.wait_for(stage, AT_STRAIGHT.1)?;
        let full = b_line(stage, top, slope, line_style(palette::game_b(), 3.0));
        fade_in_all(stage, &[full], 0.6)?;
        fade_out_all(stage, &[trail, mover], 0.2)?;
        line.wait_for(stage, AT_FORMULA.1)?;
        let tex_line = panel_tex(&kit, stage, &n.tex[0], TEX_YS[0])?;
        fade_in_all(stage, &[tex_line], 0.5)?;
        line.wait_for(stage, AT_THIRTEENTHS.1)?;
        let tex_even = panel_tex(&kit, stage, &n.tex[1], TEX_YS[1])?;
        fade_in_all(stage, &[tex_even], 0.5)?;
        flash_all(stage, &[b_point], 0.6)?;
        line.wait_for(stage, AT_WEIGHT.1)?;
        flash_all(stage, &ghosts[..1], 0.7)?;
        line.wait_for(stage, AT_WHY.1)?;
        let pointer = frame_around(stage, &b_mark[1..], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[b_point], 0.8)?;

        // 12_8: at the mix's share, what do B's rounds win? (a prediction:
        // a gold dashed marker, no value)
        let line = say(&kit, stage, "12_8")?;
        line.wait_for(stage, AT_AT_MIX.1)?;
        let marker_line = upright(stage, mix_share, WIN_RANGE.1, palette::mix());
        fade_in_all(stage, &marker_line, 0.5)?;
        line.wait_for(stage, AT_WHAT.1)?;
        let question = panel_line(&kit, stage, &n.question, 20.0, Palette::hero(), QUESTION_Y)?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 12_9: 52.5 %; with the A rounds' 50 %, the mix wins 51.3 %
        let line = say(&kit, stage, "12_9")?;
        line.wait_for(stage, AT_ANSWER.1)?;
        let mix_point = [
            point(stage, mix_share, n.wins[1], palette::mix()),
            point_words(&kit, stage, mix_share, n.wins[1], palette::mix(), &n.win_words[1])?,
        ];
        let mut leaving = vec![q, question];
        leaving.extend(&marker_line);
        let anims = swap(stage, &leaving, &mix_point, 0.5)?;
        stage.play_prepared(anims)?;
        flash_all(stage, &mix_point[..1], 0.6)?;
        line.wait_for(stage, AT_AVERAGED.1)?;
        let tex_sum = panel_tex(&kit, stage, &n.tex[2], TEX_YS[0])?;
        let anims = swap(stage, &[tex_line, tex_even], &[tex_sum], 0.6)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_ALL.1)?;
        let pointer = frame_around(stage, &[tex_sum], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;

        // 12_10: the gain a round, and the exact values
        let line = say(&kit, stage, "12_10")?;
        line.wait_for(stage, AT_GAIN.1)?;
        let gain = panel_line(&kit, stage, &n.gain, 24.0, palette::mix(), TEX_YS[1])?;
        let k05 = precisely(&kit, stage, &n.k05)?;
        fade_in_all(stage, &[gain, k05], 0.5)?;
        let card_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&n.k05))?;
        fade_out_all(stage, &[k05], 0.4)?;

        // 12_11: A's rounds only break even; they move the crowd off the
        // bad corner; B's good coin, at corners 1 and 2, does the winning
        let line = say(&kit, stage, "12_11")?;
        line.wait_for(stage, AT_ONLY.1)?;
        let pointer = frame_around(stage, &[a_kind], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_MOVE_OFF.1)?;
        flash_all(stage, &[lost], 0.7)?;
        line.wait_for(stage, AT_GOOD.1)?;
        let pointer = frame_around(stage, &[b_kind], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &b_half[1..], 0.8)?;

        // 12_12: A alone keeps a third at every corner; the mix pulls corner
        // 0 most of the way there (held)
        let line = say(&kit, stage, "12_12")?;
        line.wait_for(stage, AT_A_KEEPS.1)?;
        let third_w = third / 100.0;
        let thirds: Vec<Mob> = (0..3).flat_map(|i| dashed_tick(stage, i, third_w)).collect();
        fade_in_all(stage, &thirds, 0.6)?;
        flash_all(stage, &third_mark[..1], 0.6)?;
        line.wait_for(stage, AT_MOST.1)?;
        let y = DIAL.p(b_share, WIN_RANGE.0)[1] + ARROW_DY;
        let (x_b, x_mix, x_third) = (
            DIAL.p(b_share, WIN_RANGE.0)[0],
            DIAL.p(mix_share, WIN_RANGE.0)[0],
            DIAL.p(third, WIN_RANGE.0)[0],
        );
        let mut path = arrow(stage, v(x_b, y), v(x_mix, y), Palette::hero(), 3.0).to_vec();
        path.extend(dashed(stage, v(x_mix, y), v(x_third, y), 0.06, Palette::dim(), 2.0));
        let toward = kit.text(
            stage,
            &n.toward,
            18.0,
            Palette::hero(),
            v((x_b + x_mix) / 2.0, y + ARROW_WORDS_DY - ARROW_DY),
        )?;
        keep_between(stage, toward, DIAL.left(), FRAME_X_RADIUS - EDGE, "the arrow's words")?;
        path.push(toward);
        fade_in_all(stage, &path, 0.6)?;
        let dial_texts = [
            scale[scale.len() - 3],
            scale[scale.len() - 2],
            scale[scale.len() - 1],
            wins[1],
            even_words,
            b_mark[1],
            mix_words,
            third_mark[1],
            third_point[1],
            mix_point[1],
            toward,
            tag[1],
        ];
        keep_apart(stage, &dial_texts, 0.04, "the dial's words")?;
        let panel_texts = [
            head[0], head[1], players, names[0], names[1], names[2], settled[0], settled[1], settled[2], a_kind,
            b_kind, tex_sum, gain,
        ];
        keep_apart(stage, &panel_texts, 0.04, "the panel's words")?;
        let up = now(stage);
        let pointer = frame_around(stage, &[toward], POINT_PAD, Palette::hero())?;
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

    use super::super::draw::{BADGE_AT, THINK_S};
    use super::super::one_player_to_a_crowd::LABEL_Y;
    use super::*;
    use crate::model::PUBLISHED;

    /// The chapter's words are true of its numbers: 26 players, the rows'
    /// 38.5 → 34.6 %, 15.4 → 25.4 %, 46.2 → 40.1 %; A's coin 50 % at every
    /// corner, B's 10 % at corner 0 and 75 % elsewhere; the dial's marks at
    /// 38.5 %, 34.6 % and a third; 53.3 % at a third, 52.5 % at the mix's
    /// share; the line 75 % − 65 % × share, even at 25/65 = 5/13; the mix's
    /// 51.3 % and +2.54 cents; 76 % of the way to a third.
    #[test]
    fn one_dial_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.players, "26");
        assert_eq!(
            n.settled,
            ["38.5 \u{2192} 34.6 %", "15.4 \u{2192} 25.4 %", "46.2 \u{2192} 40.1 %"]
        );
        assert_eq!(
            n.legend,
            [
                "A rounds: 50 % at every corner",
                "B rounds: 10 % at corner 0, 75 % elsewhere"
            ]
        );
        assert_eq!(n.marks, ["game B alone: 38.5 %", "the mix: 34.6 %", "a third: 33.3 %"]);
        assert_eq!(n.win_words, ["53.3 %", "52.5 %"]);
        assert_eq!(n.even_words, "50 %");
        assert_eq!(
            n.tex,
            [
                r"\text{B wins} = 75\% - 65\% \times \text{share}",
                r"50\% \text{ at share} = \tfrac{25}{65} = \tfrac{5}{13} \approx 38.5\%",
                r"\tfrac{1}{2} \cdot 50\% + \tfrac{1}{2} \cdot 52.5\% \approx 51.3\%",
            ]
        );
        assert_eq!(n.gain, "+2.54 cents a round");
        assert_eq!(n.question, "at 34.6 %, what do B's rounds win?");
        assert_eq!(n.toward, "76 % of the way to a third");
        assert_eq!(
            n.k05,
            "exactly: the mix wins 727/1418 of its rounds, and every round wins or loses one dollar, so it \
             gains 2 \u{b7} 727/1418 \u{2212} 1 = 18/709 of a dollar a round (Ethier and Lee also give 18/709)"
        );
        assert!((n.top - 75.0).abs() < 1e-9 && (n.slope - 65.0).abs() < 1e-9 && (n.even - 50.0).abs() < 1e-9);
        // the dial's places sit on the line: 53.3 % at a third, 50 % at B's
        // own share, 52.5 % at the mix's
        let [third, b_share, mix_share] = n.shares;
        assert!((on_line(n.top, n.slope, third) - n.wins[0]).abs() < 1e-9);
        assert!((on_line(n.top, n.slope, b_share) - n.even).abs() < 1e-9);
        assert!((on_line(n.top, n.slope, mix_share) - n.wins[1]).abs() < 1e-9);
    }

    /// One number decides B: with the round coins its chance is 3/4 −
    /// 13/20 × (share at corner 0) whatever the other corners split, so the
    /// mix's B rounds win what the line says at the mix's share, and the mix
    /// wins half its A rounds' 1/2 plus half that. Failed controls: if the
    /// mix's B rounds were spread like B alone's they would win exactly
    /// half and the mix would only break even; A's rounds win exactly half
    /// in the mix, so "A's coin does the winning" is false; a switch that
    /// reads the corner (B only at corner 0, chapter 18) plays every B round
    /// on the bad coin, not at the crowd's shares.
    #[test]
    fn one_number_decides_b_and_the_mix() {
        let (a, b, mix) = (ROUND.game_a(), ROUND.game_b(), ROUND.mix());
        let w = mix.shares();
        let line = |s: Ratio| Ratio::new(3, 4) - Ratio::new(13, 20) * s;
        assert_eq!(b.win_at(&w), line(w[0]));
        assert_eq!(b.win_at(&[w[0], Ratio::ONE - w[0], Ratio::ZERO]), line(w[0]));
        assert_eq!(mix.win(), (a.win_at(&w) + b.win_at(&w)) / Ratio::int(2));
        assert_eq!(b.win_at(&w), Ratio::new(745, 1418));
        assert_eq!(mix.win(), Ratio::new(727, 1418));
        // controls
        let as_alone = (a.win_at(&w) + b.win_at(&b.shares())) / Ratio::int(2);
        assert_eq!(Ratio::int(2) * as_alone - Ratio::ONE, Ratio::ZERO);
        assert_eq!(a.win_at(&w), Ratio::new(1, 2));
        let switch = ROUND.b_at_zero();
        assert!(switch.shares()[0] < Ratio::ONE);
        assert_eq!(switch.chance(0), facts::coins(&ROUND)[1]);
        assert_ne!(switch.chance(1), b.chance(1));
    }

    /// The sums are honest about rounding: at one decimal 1/2 · 50 % + 1/2 ·
    /// 52.5 % is 51.25 %, which prints 51.3 % but is not it, so the scene
    /// writes ≈; the exact 1/2 · 1/2 + 1/2 · 745/1418 is 727/1418. Failed
    /// controls: an exact pair writes "="; a pair whose printed terms do not
    /// round to the printed result is refused.
    #[test]
    fn the_sums_say_approximately_when_they_round() {
        let (b, mix) = (ROUND.game_b(), ROUND.mix());
        let w = mix.shares();
        let sum = average_tex(Ratio::new(1, 2), b.win_at(&w), mix.win()).unwrap();
        assert!(sum.contains(r"\approx"), "{sum}");
        assert_eq!(pct_value("52.5").unwrap(), Ratio::new(21, 40));
        assert_eq!(pct_value("50").unwrap(), Ratio::new(1, 2));
        assert_eq!(
            ((pct_value("50").unwrap() + pct_value("52.5").unwrap()) / Ratio::int(2)),
            Ratio::new(5125, 10000)
        );
        assert_ne!(Ratio::new(5125, 10000), pct_value("51.3").unwrap());
        let exact = average_tex(Ratio::new(1, 2), Ratio::new(1, 10), Ratio::new(3, 10)).unwrap();
        assert!(exact.contains(" = 30"), "{exact}");
        // 11's 30 % sum, now through the same honest printer
        assert!(average_tex(Ratio::new(1, 2), Ratio::new(1, 10), Ratio::new(31, 100)).is_err());
        // the published coins print the same way (13_6's sum)
        let (pb, pm) = (PUBLISHED.game_b(), PUBLISHED.mix());
        let pw = pm.shares();
        let pub_sum = average_tex(facts::coins(&PUBLISHED)[0], pb.win_at(&pw), pm.win()).unwrap();
        assert_eq!(
            pub_sum,
            r"\tfrac{1}{2} \cdot 49.5\% + \tfrac{1}{2} \cdot 52.1\% \approx 50.8\%"
        );
    }

    /// The even share comes from the coins as the line says it: (75 − 50) /
    /// 65 = 5/13 for the round coins, (74.5 − 50) / 65 = 49/130 for the
    /// published ones (13_4), a 1/130 move. Failed control: the halfway
    /// between the two coins' even shares is neither.
    #[test]
    fn the_line_is_even_where_its_coins_say() {
        let [round_line, round_even] = line_tex(&ROUND).unwrap();
        assert_eq!(round_line, r"\text{B wins} = 75\% - 65\% \times \text{share}");
        assert!(round_even.contains(r"\tfrac{25}{65} = \tfrac{5}{13}"), "{round_even}");
        let [pub_line, pub_even] = line_tex(&PUBLISHED).unwrap();
        assert_eq!(pub_line, r"\text{B wins} = 74.5\% - 65\% \times \text{share}");
        assert_eq!(
            pub_even,
            r"50\% \text{ at share} = \tfrac{24.5}{65} = \tfrac{49}{130} \approx 37.7\%"
        );
        let shift = facts::fair_share(&ROUND) - facts::fair_share(&PUBLISHED);
        assert_eq!(shift, Ratio::new(1, 130));
        let middle = (facts::fair_share(&ROUND) + facts::fair_share(&PUBLISHED)) / Ratio::int(2);
        assert!(middle != Ratio::new(5, 13) && middle != Ratio::new(49, 130));
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The "precisely" card fits the lower third's three lines (as 05's).
    #[test]
    fn the_card_fits_three_lines() {
        let n = numbers().unwrap();
        let lines = explainer_kit::animatic::wrap(&n.k05, 76, usize::MAX);
        assert!(lines.len() <= 3, "{lines:?}");
        assert_eq!(lines.join(" "), n.k05);
    }

    /// The layout keeps its rooms: the panel's rows, kinds of round, sums
    /// and question stack without touching and clear a three-line lower
    /// third; the dial stands right of the panel with room for its win
    /// axis's words, its title under the badge, its rows of words above the
    /// lower third; every line both coin sets draw stays in the dial; the
    /// countdown keeps clear of the line and the title; 12_12's arrow runs
    /// between the share axis and the even line, under B's point.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
            assert!(LABEL_Y - 0.15 > ROW_YS[0] + BAR_H / 2.0 + TICK_OUT);
            assert!(ROW_YS[2] - BAR_H / 2.0 - TICK_OUT > LEGEND_YS[0] + 0.12);
            assert!(LEGEND_YS[0] - LEGEND_YS[1] >= 0.35);
            assert!(LEGEND_YS[1] - 0.12 > TEX_YS[0] + 0.3);
            assert!(TEX_YS[0] - TEX_YS[1] >= 0.6);
            assert!(TEX_YS[1] - 0.3 > QUESTION_Y + 0.12);
            assert!(QUESTION_Y - 0.12 > card_top + 0.05);
            assert!(DIAL.center[0] - DIAL.width / 2.0 > PANEL_RIGHT + 1.2);
            assert!(DIAL_TITLE_Y - 0.15 > DIAL.center[1] + DIAL.height / 2.0 + 0.2);
            assert!(DIAL_TITLE_Y + 0.15 < BADGE_AT[1] - 0.19);
            assert!(MARK_H + 0.05 < ARROW_DY);
            assert!(DIAL.center[1] - DIAL.height / 2.0 - BELOW[2] - 0.15 > card_top);
            assert!(BELOW[1] - BELOW[0] >= 0.3 && BELOW[2] - BELOW[1] >= 0.3);
            assert!(THINK_AT[1] + THINK_R < DIAL_TITLE_Y - 0.2);
            assert!(DIAL.center[0] + DIAL.width / 2.0 + 0.4 < FRAME_X_RADIUS);
            assert!(ALONE_YS[0] - ALONE_YS[1] >= 0.4);
        }
        let n = numbers().unwrap();
        // the longest bar ends before the rows' settled shares
        let longest = n.mix_w.iter().chain(&n.b_w).fold(0.0f64, |a, &b| a.max(b));
        assert!(BAR_X + BAR_ALL * longest < SHARE_X - 0.15, "{longest}");
        // both coin sets' lines stay inside the dial at both ends
        let (x0, x1) = DIAL.x;
        let pub_top = 100.0 * facts::coins(&PUBLISHED)[2].to_f64();
        for top in [n.top, pub_top] {
            assert!(on_line(top, n.slope, x0) < WIN_RANGE.1 && on_line(top, n.slope, x1) > WIN_RANGE.0);
        }
        // every place on the dial is inside its scale
        for s in n.shares {
            assert!(s > x0 && s < x1, "{s}");
        }
        // the countdown's note ("pause for more time", 0.3 below the ring)
        // stands above the line where the note begins
        let note_left = THINK_AT[0] - 1.2;
        let share_there = x0 + (note_left - DIAL.left()) / DIAL.width * (x1 - x0);
        let line_y = DIAL.p(share_there, on_line(n.top, n.slope, share_there))[1];
        assert!(THINK_AT[1] - THINK_R - 0.3 - 0.15 > line_y + 0.1, "{line_y}");
        // 12_12's arrow and words stand between the axis and B's point
        let axis_y = DIAL.p(x0, WIN_RANGE.0)[1];
        let b_point_y = DIAL.p(n.shares[1], n.even)[1];
        assert!(axis_y + ARROW_WORDS_DY + 0.15 < b_point_y - POINT_R);
        // the panel stays left of the dial
        assert!(PANEL_RIGHT < DIAL.left() - 1.0);
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 12 is said once, in order;
    /// 12_2 and 12_6 end with a beat; the prediction (12_8) gets its silent
    /// think time; the card (12_10) stands long enough to read; the last
    /// picture stands.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = OneDial { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("12_one_dial: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=12).map(|k| format!("12_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        for (before, after) in [("12_2", "12_3"), ("12_6", "12_7")] {
            assert!(
                at(after) - end_of(before) >= BEAT,
                "{before}: {} s",
                at(after) - end_of(before)
            );
        }
        assert!(
            at("12_9") - end_of("12_8") >= THINK_S,
            "{} s to think",
            at("12_9") - end_of("12_8")
        );
        let n = numbers().unwrap();
        assert!(at("12_11") - at("12_10") >= read_s(&n.k05), "the card is not readable");
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end - end_of("12_12") >= 1.0, "the reveal is not held: {end}");
    }
}
