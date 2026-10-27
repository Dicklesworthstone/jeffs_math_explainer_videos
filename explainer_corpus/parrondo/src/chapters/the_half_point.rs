//! Chapter 13, the half point returns. The dial as 12 left it, with the
//! round-number coins (the outlined badge): B's line, B alone's violet
//! point on the dashed 50 % line, the mix's gold point at 52.5 %, their
//! marks at 38.5 % and 34.6 %; at the top left the three coins in a row with
//! their chances 50, 10, 75 %; under them a ledger, "long-run average per
//! round, in cents": game A 0.00, game B 0.00, the mix +2.54. Shave the half
//! point off every coin: a "−0.5" falls toward each chance as it becomes
//! 49.5, 9.5, 74.5 %, and the badge turns to the filled "published coins"
//! (13_1).
//! Game A: "→ −1.00" (13_2). The crowds barely move: the round coins' points
//! dim, and one tracker shaves the coins from the round ones to the
//! published ones while both marks follow their corner 0's share, B's
//! 38.5 → 38.4 %, the mix's 34.6 → 34.5 % (13_3, a beat). B's line drops by
//! half a point everywhere: a second tracker lowers the line, the old one
//! left faint (the dimmed round points go, as they would all but touch the
//! published ones), and a hero point where it crosses 50 % slides to 37.7 %
//! ("break even: 37.7 %"; 74.5 % − 65 % × share, at 50 % where share =
//! 24.5/65 = 49/130); B's own crowd sits just past it, so B's point is under
//! 50 %, and game B: "→ −0.87" (13_4, a beat). The mix had an edge of +2.54;
//! does it survive? ("→ ?", 13_5, a prediction.) It does: at 34.5 % the
//! mix's B rounds win 52.1 % (a gold point on the lowered line); the A
//! rounds lose a little; 1/2 · 49.5 % + 1/2 · 52.1 % ≈ 50.8 % (13_6). The
//! half point costs the mix about one cent, +2.54 − 0.97 ≈ +1.57, and the
//! ledger's "→ +1.57" (13_7). The player's runs showed these same shares:
//! "model, published coins: …" over "the player's runs, 100,000 rounds each:
//! …", and a teal chip on each mark (13_8). Every number from the opening:
//! the bars of 05_3 return exactly, halfway struck (13_9). The paper:
//! Harmer and Abbott, "Losing strategies can win by Parrondo's paradox",
//! Nature 402, 864 (1999) (13_10, held).
//!
//! The model's long-run shares are labelled "model", the seeded runs with
//! their length; they agree only to the one decimal shown. Every value
//! comes from `crate::model` and is checked against `crate::facts`; the
//! scene refuses to draw if the words would stop being true of them. The
//! marks, the line and its crossing are drawn from the coins less a
//! tracker's share of the half point, so their ends are the two coin sets'
//! computed values. No typeset text is indicated, scaled or recoloured.

use explainer_kit::{Kit, Palette, Timed, fade_all, rect, thousands, tween, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    chip, coin, fade_in_all, fade_out_all, fade_to, fail, flash_all, flat, keep_between, line_style, now, pause, say,
    screen, stand, think, together,
};
use super::looks_hopeless::{
    self, Averages, EDGE, FAINT, POINT_PAD, ZERO_X, averages, badge_text, bars, frame_around, full_badge, point_at,
};
use super::one_dial::{
    B_ALONE, BEAT, DIAL, MARK_H, MIX_WORDS, PANEL_RIGHT, PERCENT, POINT_R, THINK_AT, WIN_RANGE, average_tex, b_line,
    even_line, line_tex, mark_tick, mark_words, on_line, pct_value, point, point_words, share_axis, upright, win_axis,
};
use super::one_player_to_a_crowd::{PANEL_X, round_badge, round_badge_text};
use super::three_corners::{keep_apart, swap};
use crate::facts;
use crate::model::{self, DEN, HANDICAP, PUBLISHED, Play, ROUND, Ratio};
use crate::palette;

pub struct TheHalfPoint {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const A_FACE: &str = "A";
const BAD_FACE: &str = "bad";
const GOOD_FACE: &str = "good";
const LEDGER_HEAD: &str = "long-run average per round, in cents";
const ROWS: [&str; 3] = ["game A", "game B", "the mix"];
const ARROW_TO: &str = "\u{2192}";
const ASK: &str = "?";
const EVEN_WORDS: &str = "break even:";
const MODEL_WORDS: &str = "model, published coins:";
const RUNS_WORDS: &str = "the player's runs,";
const ROUNDS_EACH: &str = "rounds each:";
const GAME_B: &str = "game B";
const THE_MIX: &str = "the mix";
const CENTS: &str = "cents";
const AUTHORS: &str = "G. P. Harmer and D. Abbott";
const PAPER_TITLE: &str = "Losing strategies can win by Parrondo's paradox";
const JOURNAL: &str = "Nature";

/// Where each picture lands: (line, the phrase it waits for).
const AT_SHAVE: (&str, &str) = ("13_1", "shave the half point off every coin");
const AT_PUBLISHED: (&str, &str) = ("13_1", "the coins Harmer and Abbott published");
const AT_LOSE: (&str, &str) = ("13_1", "both games really lose");
const AT_A_SLIPS: (&str, &str) = ("13_2", "losing a cent a round");
const AT_BARELY: (&str, &str) = ("13_3", "The crowds barely move");
const AT_B_FOUR: (&str, &str) = ("13_3", "to thirty-eight point four percent");
const AT_MIX_HALF: (&str, &str) = ("13_3", "to thirty-four and a half");
const AT_DROPS: (&str, &str) = ("13_4", "B's line drops by half a point everywhere");
const AT_FALLS: (&str, &str) = ("13_4", "falls to thirty-seven point seven percent");
const AT_PAST: (&str, &str) = ("13_4", "sits just past that");
const AT_B_LOSES: (&str, &str) = ("13_4", "B loses a little under a cent a round");
const AT_EDGE: (&str, &str) = ("13_5", "an edge of just over two and a half cents a round");
const AT_SURVIVE: (&str, &str) = ("13_5", "Does it survive the handicap");
const AT_DOES: (&str, &str) = ("13_6", "It does");
const AT_STILL: (&str, &str) = ("13_6", "still win fifty-two point one percent");
const AT_A_LOSE: (&str, &str) = ("13_6", "The A rounds now lose a little");
const AT_FIFTY_EIGHT: (&str, &str) = ("13_6", "fifty point eight percent");
const AT_COSTS: (&str, &str) = ("13_7", "costs the mix about one cent a round");
const AT_BIGGER: (&str, &str) = ("13_7", "the edge was bigger than that");
const AT_STILL_WINS: (&str, &str) = ("13_7", "The mix still wins");
const AT_RUNS: (&str, &str) = ("13_8", "The player's own runs");
const AT_RUN_B: (&str, &str) = ("13_8", "thirty-eight point four percent in game B");
const AT_RUN_MIX: (&str, &str) = ("13_8", "thirty-four and a half in the mix");
const AT_EVERY: (&str, &str) = ("13_9", "every number from the opening");
const AT_LOSERS: (&str, &str) = ("13_9", "A loses a cent a round");
const AT_MIX_WINS: (&str, &str) = ("13_9", "the coin-flip mix of the two wins");
const AT_PAPER: (&str, &str) = ("13_10", "Harmer and Abbott's paper");
const AT_NATURE: (&str, &str) = ("13_10", "It appeared in Nature");
#[cfg(test)]
const CUES: [(&str, &str); 28] = [
    AT_SHAVE,
    AT_PUBLISHED,
    AT_LOSE,
    AT_A_SLIPS,
    AT_BARELY,
    AT_B_FOUR,
    AT_MIX_HALF,
    AT_DROPS,
    AT_FALLS,
    AT_PAST,
    AT_B_LOSES,
    AT_EDGE,
    AT_SURVIVE,
    AT_DOES,
    AT_STILL,
    AT_A_LOSE,
    AT_FIFTY_EIGHT,
    AT_COSTS,
    AT_BIGGER,
    AT_STILL_WINS,
    AT_RUNS,
    AT_RUN_B,
    AT_RUN_MIX,
    AT_EVERY,
    AT_LOSERS,
    AT_MIX_WINS,
    AT_PAPER,
    AT_NATURE,
];

// ------------------------------------------------------------------ layout

/// The coins in a row at the top left (07's legend, laid flat): each coin,
/// its chance right of it; a "−0.5" starts above the chance and falls
/// toward it as it fades, stopping clear of its digits.
const COINS_Y: f64 = 2.9;
const COIN_XS: [f64; 3] = [-6.4, -4.4, -2.4];
const COIN_R: f64 = 0.34;
const CHANCE_DX: f64 = 0.44;
const MINUS_DY: f64 = 0.45;
const MINUS_DROP: f64 = 0.15;
/// The ledger: its head, its rows; each row's name, the round coins' value,
/// then "→" and the published coins'.
const LEDGER_HEAD_Y: f64 = 2.2;
const LEDGER_YS: [f64; 3] = [1.75, 1.3, 0.85];
const FROM_X: f64 = -4.9;
const TO_X: f64 = -3.95;
/// The panel's sums, under the ledger.
const TEX_YS: [f64; 2] = [0.15, -0.55];
/// 13_8's two lines, across the foot of the picture.
const CAPTION_YS: [f64; 2] = [-2.6, -3.0];
/// 05_3's strike through the halfway guess overhangs it this far.
const STRIKE_PAD: f64 = 0.12;
/// 13_10's card.
const CARD_AT: Vec3 = [0.0, 0.4, 0.0];
const CARD_W: f64 = 10.0;
const CARD_H: f64 = 2.4;
const CARD_DYS: [f64; 3] = [0.7, 0.0, -0.7];
/// Seconds: the crowds' move, the line's drop; a look at the bars.
const MOVE_S: f64 = 1.5;
const DROP_S: f64 = 1.4;
const INSPECT_S: f64 = 1.5;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    round_badge: String,
    published_badge: String,
    /// The coins' chances, round then published ("50 %", "49.5 %"), and
    /// the half point taken off each ("−0.5").
    coins: [[String; 3]; 2],
    minus: String,
    /// The ledger's values: the round coins' ("0.00", "+2.54"), then "→" and
    /// the published coins'; the mix's question.
    ledger: [[String; 3]; 2],
    ask: String,
    /// The dial in percent: B's line's top for each coin set, its slope, the
    /// even line; the half point.
    top: [f64; 2],
    slope: f64,
    even: f64,
    eps: f64,
    even_words: String,
    /// B alone's and the mix's places and chances, round then published.
    shares: [[f64; 2]; 2],
    wins: [[f64; 2]; 2],
    marks: [[String; 2]; 2],
    /// The mix's point's words, round then published.
    mix_words: [String; 2],
    /// Where the published line is even, and its mark's words.
    pub_even: f64,
    even_mark: String,
    /// The published line, where it is even, the mix's average, the cost.
    tex: [String; 4],
    captions: [String; 2],
    /// The player's runs' shares at corner 0, percent: game B, the mix.
    runs: [f64; 2],
    averages: Averages,
    citation: [String; 3],
}

/// The coins' heads chances less `h` half points (0: the round coins, 1:
/// the published ones), as floats: game B's and the mix's at corners 0, 1
/// and 2.
fn shaved(h: f64) -> [[f64; 3]; 2] {
    let eps = HANDICAP as f64 / DEN as f64;
    let [a, bad, good] = facts::coins(&ROUND).map(|c| c.to_f64() - h * eps);
    [[bad, good, good], [(a + bad) / 2.0, (a + good) / 2.0, (a + good) / 2.0]]
}

/// Corner 0's long-run share for heads chances `p` at corners 0, 1 and 2:
/// its spanning-tree weight over all three (`Game::tree_weights`, in
/// floats).
fn share0(p: [f64; 3]) -> f64 {
    let q = p.map(|x| 1.0 - x);
    let w = [
        q[1] * p[2] + p[1] * p[2] + q[1] * q[2],
        p[0] * q[2] + p[0] * p[2] + q[0] * q[2],
        q[0] * p[1] + p[0] * p[1] + q[0] * q[1],
    ];
    w[0] / (w[0] + w[1] + w[2])
}

/// The first of a fact's comma-separated values.
fn first(values: &str) -> &str {
    values.split(',').next().unwrap_or(values)
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let half = Ratio::new(1, 2);
    // 13_1: every published coin is its round coin less the half point
    let eps = Ratio::new(HANDICAP.into(), DEN.into());
    if get("rules.handicap")? != eps.to_string() || eps.pct(1) != "0.5" {
        return Err("13_1 says half a point, but the handicap differs".into());
    }
    let (from, to) = (facts::coins(&ROUND), facts::coins(&PUBLISHED));
    if from.iter().zip(&to).any(|(&r, &p)| r - p != eps) {
        return Err("13_1's coins are not the round ones less the half point".into());
    }
    let pcts = |c: &[Ratio; 3]| -> Result<[String; 3], String> {
        let [x, y, z] = c.map(|r| r.exact_pct());
        Ok([x?, y?, z?])
    };
    let (from_pct, to_pct) = (pcts(&from)?, pcts(&to)?);
    if get("coins.round.heads.pct")? != from_pct.join(",") || get("coins.published.heads.pct")? != to_pct.join(",") {
        return Err("the coins disagree with the facts".into());
    }
    let (ra, rb, rm) = (ROUND.game_a(), ROUND.game_b(), ROUND.mix());
    let (pa, pb, pm) = (PUBLISHED.game_a(), PUBLISHED.game_b(), PUBLISHED.mix());
    // 13_1 and 13_2: with the round coins A and B break even; with the
    // published ones both lose, A a cent a round
    if ra.mean() != Ratio::ZERO || rb.mean() != Ratio::ZERO || pa.mean() >= Ratio::ZERO || pb.mean() >= Ratio::ZERO {
        return Err("13_1 says both games really lose with the published coins".into());
    }
    if pa.mean() != Ratio::new(-1, 100) || pa.mean().cents(2) != get("pub.a.mean.cents")? {
        return Err("13_2 says game A loses a cent a round".into());
    }
    // 13_3: the crowds barely move (each corner 0 by under a fifth of a
    // point) but print one tenth lower
    let (rbw, pbw) = (rb.shares()[0], pb.shares()[0]);
    let (rmw, pmw) = (rm.shares()[0], pm.shares()[0]);
    if get("crowd.share0.pct")? != rbw.pct(1)
        || first(&get("pub.b.weights.pct")?) != pbw.pct(1)
        || first(&get("round.mix.weights.pct")?) != rmw.pct(1)
        || first(&get("pub.mix.weights.pct")?) != pmw.pct(1)
    {
        return Err("13_3's shares disagree with the facts".into());
    }
    let barely = Ratio::new(2, 1000);
    for (r, p) in [(rbw, pbw), (rmw, pmw)] {
        if r <= p || r - p >= barely || r.pct(1) == p.pct(1) {
            return Err("13_3 says each crowd slips a little".into());
        }
    }
    // 13_4: the line drops by exactly the half point at every share, so it
    // is even at 49/130 = 37.7 %; B's own crowd sits just past that, so B
    // loses a little under a cent
    let third = Ratio::new(1, 3);
    for s in [Ratio::ZERO, third, pbw, pmw, Ratio::ONE] {
        let rest = (Ratio::ONE - s) / Ratio::int(2);
        if rb.win_at(&[s, rest, rest]) - pb.win_at(&[s, rest, rest]) != eps {
            return Err("13_4 says the line drops by half a point everywhere".into());
        }
    }
    let pub_even = facts::fair_share(&PUBLISHED);
    if pub_even.to_string() != get("pub.b.fair_share")? || pub_even.pct(1) != get("pub.b.fair_share.pct")? {
        return Err("13_4's even share disagrees with the facts".into());
    }
    if facts::fair_share(&ROUND) - pub_even != eps / (from[2] - from[1]) {
        return Err("the even share moves by the half point over the line's slope".into());
    }
    if pbw <= pub_even || pbw - pub_even >= Ratio::new(1, 100) {
        return Err("13_4 says B's own crowd sits just past its even share".into());
    }
    let pb_mean = pb.mean();
    if pb_mean <= Ratio::new(-1, 100) || pb_mean.cents(2) != get("pub.b.mean.cents")? {
        return Err("13_4 says game B loses a little under a cent".into());
    }
    let [tex_line, tex_even] = line_tex(&PUBLISHED)?;
    // 13_5: the round mix's edge, just over two and a half cents
    let r_mean = rm.mean();
    if r_mean.cents(2) != get("round.mix.mean.cents")?
        || r_mean <= Ratio::new(25, 1000)
        || r_mean >= Ratio::new(26, 1000)
    {
        return Err("13_5 says the edge was just over two and a half cents".into());
    }
    // 13_6: at 34.5 % the mix's B rounds win 52.1 %, its A rounds 49.5 %
    // (a little losing); the mix wins 50.8 %
    let pm_w = pm.shares();
    let (pa_in, pb_in) = (pa.win_at(&pm_w), pb.win_at(&pm_w));
    if pb_in.pct(1) != get("pub.mix.b_rounds.win.pct")? || pa_in != to[0] || pa_in >= half {
        return Err("13_6's rounds disagree with the facts".into());
    }
    if pm.win() != (pa_in + pb_in) / Ratio::int(2) || pm.win().pct(1) != get("pub.mix.win.pct")? {
        return Err("13_6 says the mix wins fifty point eight percent".into());
    }
    let tex_sum = average_tex(pa_in, pb_in, pm.win())?;
    // 13_7: the half point costs the mix about one cent; the edge was
    // bigger; the mix still wins just over one and a half cents
    let p_mean = pm.mean();
    let cost = r_mean - p_mean;
    let cost_cents = (cost * Ratio::int(100)).fixed(2);
    if cost_cents != get("handicap.mix_cost.cents")?
        || cost <= Ratio::new(9, 1000)
        || cost >= Ratio::new(11, 1000)
        || r_mean <= cost
    {
        return Err("13_7 says the half point costs the mix about one cent, less than its edge".into());
    }
    if p_mean.cents(2) != get("pub.mix.mean.cents")? || p_mean <= Ratio::new(15, 1000) || p_mean >= Ratio::new(16, 1000)
    {
        return Err("13_7 says the mix still wins just over one and a half cents".into());
    }
    let [shown_edge, shown_cost, shown_left] = [r_mean.cents(2), cost_cents, p_mean.cents(2)];
    if pct_value(&shown_edge)? - pct_value(&shown_cost)? != pct_value(&shown_left)? {
        return Err("the printed cost does not take the printed edge to the printed gain".into());
    }
    let tex_cost = format!(r"{shown_edge} - {shown_cost} \approx {shown_left} \text{{ {CENTS}}}");
    // 13_8: the player's runs print these same shares, at one decimal
    let (run_b, run_mix) = (
        model::player(Play::B).share_at_zero(),
        model::player(Play::Mix).share_at_zero(),
    );
    if run_b.pct(1) != get("run.b.at0.pct")?
        || run_b.pct(1) != pbw.pct(1)
        || run_mix.pct(1) != get("run.mix.at0.pct")?
        || run_mix.pct(1) != pmw.pct(1)
    {
        return Err("13_8 says the player's runs showed these same shares".into());
    }
    if get("run.rounds")? != model::RUN_ROUNDS.to_string() {
        return Err("13_8's run length disagrees with the facts".into());
    }
    let rounds = thousands(model::RUN_ROUNDS as u64);
    // 13_9: the opening's bars, as 05_3 drew them
    let averages = averages(&f)?;
    if averages.a != pa.mean() || averages.b != pb_mean || averages.mix != p_mean {
        return Err("13_9's bars are not this chapter's averages".into());
    }
    let pct = |x: Ratio| 100.0 * x.to_f64();
    let (rb_in, rb_alone) = (rb.win_at(&rm.shares()), rb.win());
    let to_words = |x: Ratio| format!("{ARROW_TO} {}", screen(&x.cents(2)));
    Ok(Numbers {
        round_badge: round_badge_text(&f)?,
        published_badge: badge_text(&f)?,
        coins: [
            from_pct.map(|p| format!("{p} {PERCENT}")),
            to_pct.map(|p| format!("{p} {PERCENT}")),
        ],
        minus: screen(&format!("-{}", eps.pct(1))),
        ledger: [
            [ra.mean().cents(2), rb.mean().cents(2), r_mean.cents(2)],
            [to_words(pa.mean()), to_words(pb_mean), to_words(p_mean)],
        ],
        ask: format!("{ARROW_TO} {ASK}"),
        top: [pct(from[2]), pct(to[2])],
        slope: pct(from[2] - from[1]),
        even: pct(half),
        eps: pct(eps),
        even_words: format!("{} {PERCENT}", half.exact_pct()?),
        shares: [[pct(rbw), pct(rmw)], [pct(pbw), pct(pmw)]],
        wins: [[pct(rb_alone), pct(rb_in)], [pct(pb.win()), pct(pb_in)]],
        marks: [
            [
                format!("{B_ALONE} {} {PERCENT}", rbw.pct(1)),
                format!("{MIX_WORDS} {} {PERCENT}", rmw.pct(1)),
            ],
            [
                format!("{B_ALONE} {} {PERCENT}", pbw.pct(1)),
                format!("{MIX_WORDS} {} {PERCENT}", pmw.pct(1)),
            ],
        ],
        mix_words: [
            format!("{} {PERCENT}", rb_in.pct(1)),
            format!("{} {PERCENT}", pb_in.pct(1)),
        ],
        pub_even: pct(pub_even),
        even_mark: format!("{EVEN_WORDS} {} {PERCENT}", pub_even.pct(1)),
        tex: [tex_line, tex_even, tex_sum, tex_cost],
        captions: [
            format!(
                "{MODEL_WORDS} {GAME_B} {} {PERCENT}, {THE_MIX} {} {PERCENT}",
                pbw.pct(1),
                pmw.pct(1)
            ),
            format!(
                "{RUNS_WORDS} {rounds} {ROUNDS_EACH} {GAME_B} {} {PERCENT}, {THE_MIX} {} {PERCENT}",
                run_b.pct(1),
                run_mix.pct(1)
            ),
        ],
        runs: [pct(run_b), pct(run_mix)],
        averages,
        citation: [
            AUTHORS.to_owned(),
            PAPER_TITLE.to_owned(),
            format!(
                "{JOURNAL} {}, {} ({})",
                get("source.volume")?,
                get("source.page")?,
                get("source.year")?
            ),
        ],
    })
}

// --------------------------------------------------------------- pictures

/// A value in the ledger's second column, row `row`. Detached.
fn ledger_to(kit: &Kit, stage: &mut Stage<'_>, words: &str, row: usize, color: Srgb) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, words, 22.0, color, v(TO_X, LEDGER_YS[row]))?;
    keep_between(stage, m, TO_X - 0.01, PANEL_RIGHT, "a published value")?;
    Ok(m)
}

/// A sum in the panel at height `y`. Detached.
fn panel_tex(kit: &Kit, stage: &mut Stage<'_>, src: &str, y: f64) -> fmn::Result<Mob> {
    let m = kit.tex(stage, src, 26.0, Palette::ink(), &[], ORIGIN)?;
    stage.move_to(m, v(PANEL_X, y), LEFT);
    keep_between(stage, m, PANEL_X - 0.01, PANEL_RIGHT, "a sum in the panel")?;
    Ok(m)
}

/// A mark following its crowd's corner 0 as the coins are shaved by the
/// `shave` tracker's share of the half point: `k` 0 for game B, 1 for the
/// mix. In the scene.
fn live_mark(stage: &mut Stage<'_>, shave: Mob, k: usize, color: Srgb) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let h = s.tracker_value(shave).unwrap_or(0.0);
        let c = DIAL.p(100.0 * share0(shaved(h)[k]), WIN_RANGE.0);
        s.add(
            polyline([v(c[0], c[1] - MARK_H), v(c[0], c[1] + MARK_H)])
                .style(line_style(color, 4.0))
                .build(),
        )
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

impl SceneConstruct for TheHalfPoint {
    fn name(&self) -> &str {
        "13_the_half_point"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let ([round_top, _], slope, even_at) = (n.top, n.slope, n.even);
        let eps = n.eps;

        // 13_1: the dial as 12 left it; the coins in a row and the ledger;
        // shave the half point: the chances drop and the badge turns
        let line = say(&kit, stage, "13_1")?;
        let tag = round_badge(&kit, stage, &n.round_badge)?;
        let scale = share_axis(&kit, stage)?;
        let wins = win_axis(&kit, stage)?;
        let even = even_line(&kit, stage, even_at, &n.even_words)?;
        let old_line = b_line(stage, round_top, slope, line_style(palette::game_b(), 3.0));
        let [b_at, mix_at] = n.shares[0];
        let round_points = [
            point(stage, b_at, n.wins[0][0], palette::game_b()),
            point(stage, mix_at, n.wins[0][1], palette::mix()),
        ];
        let round_mix_words = point_words(&kit, stage, mix_at, n.wins[0][1], palette::mix(), &n.mix_words[0])?;
        let ticks = [
            mark_tick(stage, b_at, palette::game_b()),
            mark_tick(stage, mix_at, palette::mix()),
        ];
        let round_marks = [
            mark_words(&kit, stage, b_at, palette::game_b(), &n.marks[0][0], 1)?,
            mark_words(&kit, stage, mix_at, palette::mix(), &n.marks[0][1], 1)?,
        ];
        let faces = [
            (palette::game_a(), A_FACE),
            (palette::bad(), BAD_FACE),
            (palette::good(), GOOD_FACE),
        ];
        let mut coins = Vec::with_capacity(3);
        let mut chances = Vec::with_capacity(3);
        let mut shaved_chances = Vec::with_capacity(3);
        for (i, (color, face)) in faces.into_iter().enumerate() {
            let at = v(COIN_XS[i], COINS_Y);
            coins.push(coin(&kit, stage, at, COIN_R, color, face)?);
            let right = COIN_XS.get(i + 1).map_or(PANEL_RIGHT, |x| x - COIN_R - 0.1);
            for (words, out) in [(&n.coins[0][i], &mut chances), (&n.coins[1][i], &mut shaved_chances)] {
                let m = kit.text_left(stage, words, 20.0, Palette::ink(), v(at[0] + CHANCE_DX, COINS_Y))?;
                keep_between(stage, m, at[0] + COIN_R, right, "a coin's chance")?;
                out.push(m);
            }
        }
        let head = kit.text_left(stage, LEDGER_HEAD, 18.0, Palette::dim(), v(PANEL_X, LEDGER_HEAD_Y))?;
        keep_between(stage, head, PANEL_X - 0.01, PANEL_RIGHT, "the ledger's head")?;
        let colors = [palette::game_a(), palette::game_b(), palette::mix()];
        let mut names = Vec::with_capacity(3);
        let mut from_values = Vec::with_capacity(3);
        for (i, (name, color)) in ROWS.iter().zip(colors).enumerate() {
            names.push(kit.text_left(stage, name, 20.0, color, v(PANEL_X, LEDGER_YS[i]))?);
            let m = kit.text_left(stage, &n.ledger[0][i], 22.0, Palette::ink(), v(FROM_X, LEDGER_YS[i]))?;
            keep_between(stage, m, FROM_X - 0.01, TO_X - 0.1, "a round-coin value")?;
            from_values.push(m);
        }
        let mut arriving = scale.clone();
        arriving.extend(wins);
        arriving.extend(&even);
        arriving.extend(tag);
        arriving.push(old_line);
        arriving.extend(round_points);
        arriving.push(round_mix_words);
        arriving.extend(ticks);
        arriving.extend(round_marks);
        arriving.extend(coins.iter().flatten());
        arriving.extend(&chances);
        arriving.push(head);
        arriving.extend(&names);
        arriving.extend(&from_values);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, AT_SHAVE.1)?;
        let mut minus = Vec::with_capacity(3);
        for &m in &chances {
            let c = stage.get_center(m);
            minus.push(kit.text(stage, &n.minus, 18.0, Palette::hero(), v(c[0], COINS_Y + MINUS_DY))?);
        }
        fade_in_all(stage, &minus, 0.3)?;
        let mut anims = swap(stage, &chances, &shaved_chances, 0.7)?;
        for &p in &minus {
            let fall = fade_out(stage.arena_mut(), p, v(0.0, -MINUS_DROP), 1.0)?;
            anims.push(stage.prepare(fall.rt(0.9))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_PUBLISHED.1)?;
        let pub_tag = full_badge(&kit, stage, &n.published_badge, PANEL_RIGHT)?;
        let anims = swap(stage, &tag, &pub_tag, 0.7)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_LOSE.1)?;
        let pointer = frame_around(stage, &names[..2], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;

        // 13_2: game A slips from fair to a cent a round
        let line = say(&kit, stage, "13_2")?;
        line.wait_for(stage, AT_A_SLIPS.1)?;
        let a_to = ledger_to(&kit, stage, &n.ledger[1][0], 0, Palette::ink())?;
        fade_in_all(stage, &[a_to], 0.4)?;
        let pointer = frame_around(stage, &[a_to], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;

        // 13_3: the crowds barely move: the round coins' points dim, and the
        // marks follow corner 0 as one tracker shaves the coins, then settle
        // as fixed ticks on the published shares (a beat)
        let line = say(&kit, stage, "13_3")?;
        line.wait_for(stage, AT_BARELY.1)?;
        let shave = stage.add_value_tracker(0.0);
        stage.scene_mut().remove(&ticks);
        let live = [
            live_mark(stage, shave, 0, palette::game_b())?,
            live_mark(stage, shave, 1, palette::mix())?,
        ];
        let dims = round_points
            .iter()
            .map(|&m| fade_to(m, FAINT, 0.5))
            .collect::<fmn::Result<Vec<_>>>()?;
        together(stage, dims)?;
        fade_out_all(stage, &[round_mix_words], 0.3)?;
        stage.play(tween(shave, 1.0, MOVE_S)?)?;
        let [pub_b_at, pub_mix_at] = n.shares[1];
        // a redrawn mark would redraw over a flash
        for m in live {
            stage.clear_updaters(m, true);
        }
        stage.scene_mut().remove(&live);
        let pub_ticks = [
            mark_tick(stage, pub_b_at, palette::game_b()),
            mark_tick(stage, pub_mix_at, palette::mix()),
        ];
        for m in pub_ticks {
            stage.add_to_scene(m)?;
        }
        line.wait_for(stage, AT_B_FOUR.1)?;
        let b_marks = mark_words(&kit, stage, pub_b_at, palette::game_b(), &n.marks[1][0], 1)?;
        let anims = swap(stage, &round_marks[..1], &[b_marks], 0.5)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[b_marks], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &pub_ticks[..1], 0.7)?;
        line.wait_for(stage, AT_MIX_HALF.1)?;
        let mix_marks = mark_words(&kit, stage, pub_mix_at, palette::mix(), &n.marks[1][1], 1)?;
        let anims = swap(stage, &round_marks[1..], &[mix_marks], 0.5)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[mix_marks], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &pub_ticks[1..], 0.7)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 13_4: B's line drops by the half point everywhere (a second
        // tracker; the old line stays faint) and its crossing of 50 % slides
        // to 37.7 %; B's own crowd sits past it: B's point is under 50 %
        // and B loses a little under a cent (a beat)
        let line = say(&kit, stage, "13_4")?;
        line.wait_for(stage, AT_DROPS.1)?;
        let lower = stage.add_value_tracker(0.0);
        let live_line = stage.always_redraw(move |s| {
            let top = round_top - eps * s.tracker_value(lower).unwrap_or(0.0);
            let (x0, x1) = DIAL.x;
            s.add(
                polyline([DIAL.p(x0, on_line(top, slope, x0)), DIAL.p(x1, on_line(top, slope, x1))])
                    .style(line_style(palette::game_b(), 3.0))
                    .build(),
            )
        });
        let crossing = stage.always_redraw(move |s| {
            let top = round_top - eps * s.tracker_value(lower).unwrap_or(0.0);
            let share = 100.0 * (top - even_at) / slope;
            s.add(
                Circle::new()
                    .radius(POINT_R)
                    .arc_center(DIAL.p(share, even_at))
                    .style(flat(Palette::hero(), 1.0))
                    .build(),
            )
        });
        stage.add_to_scene(live_line)?;
        stage.add_to_scene(crossing)?;
        let faint_old = old_line
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(DROP_S),
                ..AnimateArgs::default()
            })?
            .set_stroke(None, None, Some(FAINT))?;
        let mut anims: Vec<Box<dyn Animation>> =
            vec![stage.prepare(faint_old)?, stage.prepare(tween(lower, 1.0, DROP_S)?)?];
        // the round coins' points would all but touch the published ones;
        // the faint old line keeps the comparison
        for &m in &round_points {
            let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(gone.rt(DROP_S))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_FALLS.1)?;
        let mut fair = vec![
            mark_tick(stage, n.pub_even, Palette::hero()),
            mark_words(&kit, stage, n.pub_even, Palette::hero(), &n.even_mark, 2)?,
        ];
        fair.extend(upright(stage, n.pub_even, even_at, Palette::hero()));
        let tex_line = panel_tex(&kit, stage, &n.tex[0], TEX_YS[0])?;
        let tex_even = panel_tex(&kit, stage, &n.tex[1], TEX_YS[1])?;
        fair.extend([tex_line, tex_even]);
        fade_in_all(stage, &fair, 0.5)?;
        line.wait_for(stage, AT_PAST.1)?;
        let mut b_point = vec![point(stage, pub_b_at, n.wins[1][0], palette::game_b())];
        b_point.extend(upright(stage, pub_b_at, n.wins[1][0], palette::game_b()));
        fade_in_all(stage, &b_point, 0.4)?;
        flash_all(stage, &b_point[..1], 0.6)?;
        line.wait_for(stage, AT_B_LOSES.1)?;
        let b_to = ledger_to(&kit, stage, &n.ledger[1][1], 1, Palette::ink())?;
        fade_in_all(stage, &[b_to], 0.4)?;
        let pointer = frame_around(stage, &[b_to], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 13_5: the mix's edge; does it survive? (a prediction: "→ ?")
        let line = say(&kit, stage, "13_5")?;
        line.wait_for(stage, AT_EDGE.1)?;
        let pointer = frame_around(stage, &from_values[2..], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_SURVIVE.1)?;
        let ask = ledger_to(&kit, stage, &n.ask, 2, Palette::hero())?;
        let anims = swap(stage, &[tex_line, tex_even], &[ask], 0.5)?;
        stage.play_prepared(anims)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 13_6: it does: at 34.5 % the mix's B rounds win 52.1 % on the
        // lowered line; its A rounds lose a little; the mix wins 50.8 %
        let line = say(&kit, stage, "13_6")?;
        line.wait_for(stage, AT_DOES.1)?;
        let mut mix_point = vec![
            point(stage, pub_mix_at, n.wins[1][1], palette::mix()),
            point_words(&kit, stage, pub_mix_at, n.wins[1][1], palette::mix(), &n.mix_words[1])?,
        ];
        mix_point.extend(upright(stage, pub_mix_at, n.wins[1][1], palette::mix()));
        let anims = swap(stage, &[q], &mix_point, 0.5)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_STILL.1)?;
        flash_all(stage, &mix_point[..1], 0.6)?;
        line.wait_for(stage, AT_A_LOSE.1)?;
        let pointer = frame_around(stage, &shaved_chances[..1], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &coins[0][..1], 0.7)?;
        line.wait_for(stage, AT_FIFTY_EIGHT.1)?;
        let tex_sum = panel_tex(&kit, stage, &n.tex[2], TEX_YS[0])?;
        fade_in_all(stage, &[tex_sum], 0.5)?;

        // 13_7: the half point costs the mix about a cent, less than its
        // edge; the mix still wins
        let line = say(&kit, stage, "13_7")?;
        line.wait_for(stage, AT_COSTS.1)?;
        let tex_cost = panel_tex(&kit, stage, &n.tex[3], TEX_YS[1])?;
        fade_in_all(stage, &[tex_cost], 0.5)?;
        line.wait_for(stage, AT_BIGGER.1)?;
        let pointer = frame_around(stage, &[tex_cost], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_STILL_WINS.1)?;
        let mix_to = ledger_to(&kit, stage, &n.ledger[1][2], 2, palette::mix())?;
        let anims = swap(stage, &[ask], &[mix_to], 0.5)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[mix_to], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;

        // 13_8: the player's runs showed these shares: the model's line and
        // the runs' line, and a teal chip on each mark (a beat)
        let line = say(&kit, stage, "13_8")?;
        line.wait_for(stage, AT_RUNS.1)?;
        let mut captions = Vec::with_capacity(2);
        let looks = [Palette::dim(), palette::player()];
        for ((words, color), y) in n.captions.iter().zip(looks).zip(CAPTION_YS) {
            let m = kit.text(stage, words, 20.0, color, v(0.0, y))?;
            keep_between(
                stage,
                m,
                -FRAME_X_RADIUS + EDGE,
                FRAME_X_RADIUS - EDGE,
                "a line about the runs",
            )?;
            captions.push(m);
        }
        let chips = n.runs.map(|share| chip(stage, DIAL.p(share, WIN_RANGE.0)));
        let mut arriving = captions.clone();
        arriving.extend(chips);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, AT_RUN_B.1)?;
        flash_all(stage, &chips[..1], 0.6)?;
        line.wait_for(stage, AT_RUN_MIX.1)?;
        flash_all(stage, &chips[1..], 0.6)?;
        let dial_texts = [
            scale[scale.len() - 3],
            scale[scale.len() - 2],
            scale[scale.len() - 1],
            wins[1],
            even[even.len() - 1],
            b_marks,
            mix_marks,
            fair[1],
            mix_point[1],
            pub_tag[1],
        ];
        keep_apart(stage, &dial_texts, 0.04, "the dial's words")?;
        let panel_texts = [
            head,
            names[0],
            names[1],
            names[2],
            from_values[0],
            from_values[1],
            from_values[2],
            a_to,
            b_to,
            mix_to,
            tex_sum,
            tex_cost,
            shaved_chances[0],
            shaved_chances[1],
            shaved_chances[2],
            captions[0],
            captions[1],
        ];
        keep_apart(stage, &panel_texts, 0.04, "the panel's words")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 13_9: every number from the opening: 05_3's bars, the halfway
        // guess struck (a look)
        let line = say(&kit, stage, "13_9")?;
        line.wait_for(stage, AT_EVERY.1)?;
        let leaving: Vec<Mob> = stage
            .scene()
            .mobjects()
            .iter()
            .copied()
            .filter(|m| !pub_tag.contains(m))
            .collect();
        fade_out_all(stage, &leaving, 0.6)?;
        let bars = bars(&kit, stage, &n.averages)?;
        let from = stage.get_bounding_box_point(bars.labels[2], LEFT)[0] - STRIKE_PAD;
        let strike_y = looks_hopeless::ROW_YS[2];
        let strike = polyline([v(from, strike_y), v(ZERO_X + STRIKE_PAD, strike_y)])
            .style(line_style(palette::bad(), 4.0))
            .build();
        let strike = stage.arena_mut().add(strike);
        kit.allow_crossing(strike);
        let mut standing = bars.axis.clone();
        standing.extend(bars.games);
        standing.extend(&bars.halfway);
        standing.extend(&bars.labels);
        standing.extend([bars.mix, strike]);
        fade_in_all(stage, &standing, 0.6)?;
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
        line.wait_for(stage, AT_LOSERS.1)?;
        let pointer = frame_around(stage, &bars.labels[..2], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_MIX_WINS.1)?;
        let pointer = frame_around(stage, &bars.labels[3..], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[bars.mix], 0.8)?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 13_10: the paper (held)
        let line = say(&kit, stage, "13_10")?;
        line.wait_for(stage, AT_PAPER.1)?;
        fade_out_all(stage, &standing, 0.5)?;
        let frame = rect(
            v(CARD_AT[0] - CARD_W / 2.0, CARD_AT[1] - CARD_H / 2.0),
            CARD_W,
            CARD_H,
            line_style(Palette::dim(), 2.0),
        )?;
        let mut card = vec![stage.arena_mut().add(frame)];
        let looks = [(26.0, Palette::ink()), (26.0, Palette::hero()), (24.0, Palette::dim())];
        for ((words, (size, color)), dy) in n.citation.iter().zip(looks).zip(CARD_DYS) {
            let m = kit.text(stage, words, size, color, v(CARD_AT[0], CARD_AT[1] + dy))?;
            keep_between(
                stage,
                m,
                CARD_AT[0] - CARD_W / 2.0 + 0.2,
                CARD_AT[0] + CARD_W / 2.0 - 0.2,
                "a line of the citation",
            )?;
            card.push(m);
        }
        fade_in_all(stage, &card, 0.6)?;
        line.wait_for(stage, AT_NATURE.1)?;
        let up = now(stage);
        let pointer = frame_around(stage, &card[3..], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
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
    use super::super::one_dial::{BELOW, THINK_R};
    use super::*;

    /// The chapter's words are true of its numbers: the coins 50, 10, 75 %
    /// less 0.5 each; the ledger 0.00, 0.00, +2.54 then −1.00, −0.87, +1.57;
    /// the marks 38.5 → 38.4 % and 34.6 → 34.5 %; the published line 74.5 %
    /// − 65 % × share, even at 24.5/65 = 49/130 ≈ 37.7 %; the mix's 52.5 %
    /// then 52.1 %, 1/2 · 49.5 % + 1/2 · 52.1 % ≈ 50.8 %, +2.54 − 0.97 ≈
    /// +1.57; the model and the runs both 38.4 % and 34.5 %; the paper.
    #[test]
    fn the_half_point_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.coins, [["50 %", "10 %", "75 %"], ["49.5 %", "9.5 %", "74.5 %"]]);
        assert_eq!(n.minus, "\u{2212}0.5");
        assert_eq!(
            n.ledger,
            [
                ["0.00", "0.00", "+2.54"],
                ["\u{2192} \u{2212}1.00", "\u{2192} \u{2212}0.87", "\u{2192} +1.57"],
            ]
        );
        assert_eq!(n.ask, "\u{2192} ?");
        assert_eq!(
            n.marks,
            [
                ["game B alone: 38.5 %", "the mix: 34.6 %"],
                ["game B alone: 38.4 %", "the mix: 34.5 %"],
            ]
        );
        assert_eq!(n.mix_words, ["52.5 %", "52.1 %"]);
        assert_eq!(n.even_mark, "break even: 37.7 %");
        assert_eq!(
            n.tex,
            [
                r"\text{B wins} = 74.5\% - 65\% \times \text{share}",
                r"50\% \text{ at share} = \tfrac{24.5}{65} = \tfrac{49}{130} \approx 37.7\%",
                r"\tfrac{1}{2} \cdot 49.5\% + \tfrac{1}{2} \cdot 52.1\% \approx 50.8\%",
                r"+2.54 - 0.97 \approx +1.57 \text{ cents}",
            ]
        );
        assert_eq!(
            n.captions,
            [
                "model, published coins: game B 38.4 %, the mix 34.5 %",
                "the player's runs, 100,000 rounds each: game B 38.4 %, the mix 34.5 %",
            ]
        );
        assert_eq!(
            n.citation,
            [
                "G. P. Harmer and D. Abbott",
                "Losing strategies can win by Parrondo's paradox",
                "Nature 402, 864 (1999)",
            ]
        );
        assert!((n.eps - 0.5).abs() < 1e-12 && (n.top[0] - n.top[1] - n.eps).abs() < 1e-12);
    }

    /// The trackers' floats end on the exact model: with no half point
    /// shaved the marks stand at the round coins' 5/13 and 245/709, with the
    /// whole half point at the published 32401/84463 and 49/142; the line's
    /// crossing of 50 % moves from 5/13 to 49/130. Every shaved point
    /// between is a real game, and its marks move one way: down.
    #[test]
    fn the_shaved_coins_end_on_the_model() {
        let exact = |c: &model::Coins| [c.game_b().shares()[0].to_f64(), c.mix().shares()[0].to_f64()];
        for (h, c) in [(0.0, ROUND), (1.0, PUBLISHED)] {
            let s = shaved(h);
            let want = exact(&c);
            assert!((share0(s[0]) - want[0]).abs() < 1e-12, "{h}: {}", share0(s[0]));
            assert!((share0(s[1]) - want[1]).abs() < 1e-12, "{h}: {}", share0(s[1]));
        }
        assert_eq!(PUBLISHED.game_b().shares()[0], Ratio::new(32401, 84463));
        assert_eq!(PUBLISHED.mix().shares()[0], Ratio::new(49, 142));
        let mut last = [f64::INFINITY; 2];
        for k in 0..=20 {
            let s = shaved(f64::from(k) / 20.0);
            let here = [share0(s[0]), share0(s[1])];
            assert!(here[0] < last[0] && here[1] < last[1], "{k}");
            last = here;
        }
        let n = numbers().unwrap();
        let crossing = |h: f64| 100.0 * (n.top[0] - n.eps * h - n.even) / n.slope;
        assert!((crossing(0.0) - 100.0 * facts::fair_share(&ROUND).to_f64()).abs() < 1e-9);
        assert!((crossing(1.0) - n.pub_even).abs() < 1e-9);
    }

    /// The half point moves everything a little, and the mix's edge
    /// survives it. Failed controls: "every coin is half a point weaker, so
    /// the mix loses a whole cent more" says 1.00 cent, but the cost is 0.97
    /// (the mix's crowd also moves off corner 0); shaving only the good coin
    /// would put B's even share at 49/129, not 49/130; and B's own published
    /// crowd sits past the published even share, while the mix's sits well
    /// short of it.
    #[test]
    fn the_edge_survives_the_half_point() {
        let eps = Ratio::new(1, 200);
        let (rm, pm, pb) = (ROUND.mix(), PUBLISHED.mix(), PUBLISHED.game_b());
        let cost = rm.mean() - pm.mean();
        assert!(cost > Ratio::ZERO && cost < rm.mean() && pm.mean() > Ratio::ZERO);
        assert_eq!((cost * Ratio::int(100)).fixed(2), "0.97");
        assert_eq!((Ratio::int(2) * eps * Ratio::int(100)).fixed(2), "1.00");
        let [_, bad, good] = facts::coins(&ROUND);
        let good_only = (good - eps - Ratio::new(1, 2)) / (good - eps - bad);
        assert_eq!(good_only, Ratio::new(49, 129));
        assert_ne!(good_only, facts::fair_share(&PUBLISHED));
        let even = facts::fair_share(&PUBLISHED);
        assert!(pb.shares()[0] > even && pm.shares()[0] < even - Ratio::new(3, 100));
    }

    /// The player's runs and the model print alike at one decimal, and only
    /// there: 38.37 % against 38.36 %, 34.52 % against 34.51 % (one seeded
    /// run of 100,000 rounds each, the model's long run).
    #[test]
    fn the_runs_agree_with_the_model_at_one_decimal() {
        let (run_b, run_mix) = (
            model::player(Play::B).share_at_zero(),
            model::player(Play::Mix).share_at_zero(),
        );
        let (model_b, model_mix) = (PUBLISHED.game_b().shares()[0], PUBLISHED.mix().shares()[0]);
        for (run, model, shown) in [(run_b, model_b, "38.4"), (run_mix, model_mix, "34.5")] {
            assert_eq!(run.pct(1), shown);
            assert_eq!(model.pct(1), shown);
        }
        assert_ne!(run_b.pct(2), model_b.pct(2));
        assert_ne!(run_mix.pct(2), model_mix.pct(2));
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the coins under the badge's height and
    /// their "−0.5" inside the frame; the ledger, the sums and the runs'
    /// lines stack without touching and keep below the dial's rows of
    /// words; the panel stays left of the dial; the card fits the frame.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            assert!(COINS_Y + MINUS_DY + 0.15 < FRAME_Y_RADIUS - 0.3);
            // where the "−0.5" stops, its foot stays above the chance's top
            assert!(COINS_Y + MINUS_DY - MINUS_DROP - 0.12 > COINS_Y + 0.15);
            assert!(COINS_Y - COIN_R > LEDGER_HEAD_Y + 0.15);
            assert!(LEDGER_HEAD_Y - 0.12 > LEDGER_YS[0] + 0.15);
            assert!(LEDGER_YS[0] - LEDGER_YS[1] >= 0.4 && LEDGER_YS[1] - LEDGER_YS[2] >= 0.4);
            assert!(LEDGER_YS[2] - 0.15 > TEX_YS[0] + 0.3);
            assert!(TEX_YS[0] - TEX_YS[1] >= 0.6);
            assert!(COIN_XS[0] - COIN_R > -FRAME_X_RADIUS + 0.3);
            assert!(COIN_XS[1] - COIN_XS[0] >= 2.0 && COIN_XS[2] - COIN_XS[1] >= 2.0);
            assert!(FROM_X + 0.85 < TO_X);
            assert!(COINS_Y + COIN_R < BADGE_AT[1] - 0.19 - 0.1);
            assert!(CARD_AT[1] + CARD_H / 2.0 < BADGE_AT[1] - 0.3);
            assert!(CARD_AT[0] + CARD_W / 2.0 < FRAME_X_RADIUS - 0.5);
            assert!(CAPTION_YS[0] - CAPTION_YS[1] >= 0.35);
            assert!(CAPTION_YS[1] - 0.15 > -FRAME_Y_RADIUS + 0.3);
        }
        // the runs' lines keep below the dial's lowest row of words, and
        // the sums above them
        let lowest_row = DIAL.bottom() - BELOW[2] - 0.15;
        assert!(CAPTION_YS[0] + 0.15 < lowest_row, "{lowest_row}");
        const { assert!(TEX_YS[1] - 0.3 > CAPTION_YS[0] + 0.15) };
        // the panel's second column stays left of the dial
        assert!(TO_X + 1.3 < PANEL_RIGHT && PANEL_RIGHT < DIAL.left() - 1.0);
        // the countdown stands clear of the lowered line too
        let n = numbers().unwrap();
        let note_left = THINK_AT[0] - 1.2;
        let (x0, x1) = DIAL.x;
        let share_there = x0 + (note_left - DIAL.left()) / DIAL.width * (x1 - x0);
        let line_y = DIAL.p(share_there, on_line(n.top[0], n.slope, share_there))[1];
        assert!(THINK_AT[1] - THINK_R - 0.3 - 0.15 > line_y + 0.1);
        // every place on the dial is inside its scale
        for s in n.shares.iter().flatten().chain(&n.runs).chain([&n.pub_even]) {
            assert!(*s > x0 && *s < x1, "{s}");
        }
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting: every line of chapter 13 is said once, in order; 13_3,
    /// 13_4 and 13_8 end with a beat; the prediction (13_5) gets its silent
    /// think time; the bars (13_9) get a look; the citation stands.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TheHalfPoint { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("13_the_half_point: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=10).map(|k| format!("13_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        for (before, after) in [("13_3", "13_4"), ("13_4", "13_5"), ("13_8", "13_9")] {
            assert!(
                at(after) - end_of(before) >= BEAT,
                "{before}: {} s",
                at(after) - end_of(before)
            );
        }
        assert!(at("13_10") - end_of("13_9") >= INSPECT_S);
        assert!(
            at("13_6") - end_of("13_5") >= THINK_S,
            "{} s to think",
            at("13_6") - end_of("13_5")
        );
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end - end_of("13_10") >= 1.0, "the citation is not held: {end}");
    }
}
