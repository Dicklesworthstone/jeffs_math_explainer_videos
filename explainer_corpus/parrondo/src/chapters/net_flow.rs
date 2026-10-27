//! Chapter 14, the crowd circulates. A new question: can the coins alone
//! say whether a game wins, without finding where its crowd settles? (14_1:
//! the mix's triangle as 11 left it, B's corners and arrows with the mix's
//! gold chances 30, 62.5, 62.5 % forward and dim 70, 37.5, 37.5 % back, its
//! gold title; the badge still the filled "published coins" of 13.) Back to
//! the round coins (the outlined badge): the mix's settled crowd scaled to
//! 1,418 players so every count is whole; the panel's "players: 1,418" over
//! the dim "split by the chances (expected numbers)", and as each count is
//! said its row, "corner 0 490", and a teal puddle growing at its corner;
//! then the shares 34.6, 25.4, 40.1 % (14_2). One edge at a time: teal
//! streams along the arrows of edge 0 → 1, as wide as the players they
//! carry, "30 % × 490 = 147" forward, "37.5 % × 360 = 135" back (14_3). The
//! other two edges: those sums give way to a table, "edge", "forward −
//! back", and a row as each edge is said, "0 → 1 147 − 135", then 225 − 213
//! and 355 − 343, with their streams (14_4). Count the surplus on each edge
//! (14_5, held; nothing subtracted on screen). Twelve, every time: "= 12" on
//! every row, gold paths forward around the triangle, the column named "net
//! flow" (14_6). Every forward step wins a dollar; how many dollars does the
//! crowd gain in a round? (14_7, a prediction.) "3 × 12 = 36 = 727 − 691
//! dollars a round", and "36 dollars / 1,418 ≈ 2.54 cents per player per
//! round", 12's average (14_8). Why the same twelve? Corner 1 ringed: twelve
//! net arrive across the edge behind it, twelve net leave across the edge
//! ahead, "corner 1: 360 + 12 − 12 = 360" (14_9). These are expected
//! numbers (the dim label pointed at); one real round of the 1,418 real
//! players from the video's seed, labelled a seeded illustration, replaces
//! the table's numbers in teal: nets 7, 4, −17, each corner changing by net
//! in minus net out (−17 − 7 = −24, 7 − 4 = 3, 4 − (−17) = 21) to 466, 363,
//! 589 as the puddles follow, and "7 + 4 − 17 = −6 dollars" (14_10). Over
//! 1,000 real rounds each edge averages 11.9 and the crowd gains 35.8
//! dollars a round; the expected numbers return (14_11). In any settled
//! crowd, "gain per round = 3 × net flow" and "fair ⟺ net flow = 0": game
//! B's settled 26 (its title, chances, puddles; 1 − 1, 3 − 3, 9 − 9, = 0)
//! (14_12). Each edge balances: 10 × 1/10 = 4 × 1/4 = 1, 4 × 3/4 = 12 × 1/4
//! = 3, 12 × 3/4 = 10 × 9/10 = 9 (14_13). Multiply them: the six counts are
//! struck and go, and copies of the chances fly into one line, 1/10 · 3/4 ·
//! 3/4 = 9/10 · 1/4 · 1/4, both 9/160 (14_14). The same for any game: "w:
//! weight · p: chance forward · q: chance back"; w₀ p₀ = w₁ q₁, w₁ p₁ = w₂
//! q₂, w₂ p₂ = w₀ q₀; every corner holds players (the puddles ringed), so
//! each weight is struck; p₀ p₁ p₂ = q₀ q₁ q₂, "win, win, win" against
//! "lose, lose, lose" while teal paths run all the way round each way
//! (14_15, held).
//!
//! Every count is computed from `crate::model` (the settled crowd's split
//! flows, and the real crowd's tosses from the video's one seed) and checked
//! against `crate::facts`; the scene refuses to draw if the words would stop
//! being true of them. The expected and the real crowd are never on screen
//! unlabelled. Products and cancellations are separate pieces of TeX that
//! move as wholes; no typeset text is indicated, scaled or recoloured:
//! corners get hollow rings, words a frame, a changed label is replaced.
//! The pieces, strikes, labels, puddles and panel are shared with chapter
//! 15, which runs the test the other way on the same picture.

use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    fade_in_all, fade_out_all, fail, flash_all, flat, fly, keep_between, line_style, pause, say, screen_int, think,
    together,
};
use super::looks_hopeless::{POINT_PAD, badge_text, frame_around, full_badge, point_at};
use super::one_dial::{BEAT, PERCENT};
use super::one_player_to_a_crowd::{
    LABEL_Y, PANEL_X, PLAYERS_NUM_X, PLAYERS_Y, ROW_YS, SPLIT_LABEL, panel_head, ring_corners, round_badge,
    round_badge_text, row_name,
};
use super::the_trap::shaft;
use super::three_corners::{
    B_AT, CORNER_R, Chances, TITLE_DY, Which, chance_at, chance_labels, chances, corner, keep_apart, swap, triangle,
};
use crate::facts;
use crate::model::{self, Game, PUBLISHED, ROUND, Ratio, SEED};
use crate::palette;

pub struct NetFlow {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
pub(super) const MIX_NAME: &str = "the mix";
const ARROW: &str = "\u{2192}";
const EDGE_HEAD: &str = "edge";
const PAIR_HEAD: &str = "forward \u{2212} back";
const NET_HEAD: &str = "net";
const NET_FLOW: &str = "net flow";
const ASK: &str = "how many dollars does this whole crowd gain in one round?";
const DOLLARS_A_ROUND: &str = "dollars a round";
pub(super) const EACH: &str = "per player per round";
const CORNER_ONE: &str = "corner 1:";
const REAL_LABEL: &str = "one real round: real tosses, seeded illustration";
const THOUSAND_WORDS: &str = "real rounds, the same seeded illustration:";
const AVERAGE_NET: &str = "average net on each edge:";
const AVERAGE_GAIN: &str = "average gain:";
const BALANCES_HEAD: &str = "each edge balances: forward = back";
pub(super) const LETTERS: &str = "w: weight \u{b7} p: chance forward \u{b7} q: chance back";
pub(super) const WINS3: &str = "win, win, win";
pub(super) const LOSSES3: &str = "lose, lose, lose";

/// Where each picture lands: (line, the phrase it waits for).
const AT_QUESTION: (&str, &str) = ("14_1", "Could you tell whether a game like these wins");
const AT_TEST: (&str, &str) = ("14_1", "There is a test");
const AT_WATCH: (&str, &str) = ("14_1", "watching a settled crowd move around the triangle");
const AT_ROUND: (&str, &str) = ("14_2", "Go back to the round coins");
const AT_SCALED: (&str, &str) = ("14_2", "scaled up to one thousand four hundred eighteen players");
const AT_C0: (&str, &str) = ("14_2", "Four hundred ninety stand at corner zero");
const AT_C1: (&str, &str) = ("14_2", "three hundred sixty at corner one");
const AT_C2: (&str, &str) = ("14_2", "five hundred sixty-eight at corner two");
const AT_STILL: (&str, &str) = ("14_2", "still thirty-four point six percent");
const AT_ONE_EDGE: (&str, &str) = ("14_3", "one edge at a time");
const AT_FWD0: (&str, &str) = ("14_3", "one hundred forty-seven players step forward");
const AT_BACK0: (&str, &str) = ("14_3", "one hundred thirty-five step back");
const AT_OTHER: (&str, &str) = ("14_4", "The other two edges");
const AT_EDGE1: (&str, &str) = ("14_4", "Two hundred twenty-five forward");
const AT_EDGE2: (&str, &str) = ("14_4", "Three hundred fifty-five forward");
const AT_COUNT: (&str, &str) = ("14_5", "count the surplus on each edge");
const AT_TWELVE: (&str, &str) = ("14_6", "Twelve, every time");
const AT_CIRCULATING: (&str, &str) = ("14_6", "keeps circulating forward");
const AT_NET_FLOW: (&str, &str) = ("14_6", "That surplus is the net flow");
const AT_DOLLAR: (&str, &str) = ("14_7", "Every forward step wins a dollar");
const AT_HOW_MANY: (&str, &str) = ("14_7", "how many dollars does this whole crowd gain");
const AT_THIRTY_SIX: (&str, &str) = ("14_8", "Thirty-six dollars");
const AT_WIN_LOSE: (&str, &str) = ("14_8", "Seven hundred twenty-seven players win");
const AT_SHARED: (&str, &str) = ("14_8", "Shared among the one thousand four hundred eighteen players");
const AT_AGAIN: (&str, &str) = ("14_8", "the mix's average once again");
const AT_WHY: (&str, &str) = ("14_9", "Why twelve on every edge");
const AT_BEHIND: (&str, &str) = ("14_9", "Across the edge behind it");
const AT_AHEAD: (&str, &str) = ("14_9", "Across the edge ahead of it");
const AT_KEEPS: (&str, &str) = ("14_9", "keeps its three hundred sixty");
const AT_DISAGREED: (&str, &str) = ("14_9", "If the two edges disagreed");
const AT_EXPECTED: (&str, &str) = ("14_10", "these are expected numbers");
const AT_REAL: (&str, &str) = ("14_10", "Here is one real round");
const AT_CARRY: (&str, &str) = ("14_10", "Its edges carry seven, four and minus seventeen");
const AT_CHANGES: (&str, &str) = ("14_10", "each corner changes by exactly the difference");
const AT_LOST: (&str, &str) = ("14_10", "the crowd even lost six dollars");
const AT_THOUSAND: (&str, &str) = ("14_11", "Over a thousand real rounds");
const AT_AVERAGES: (&str, &str) = ("14_11", "each edge averages eleven point nine");
const AT_GAINS: (&str, &str) = ("14_11", "thirty-five point eight dollars a round");
const AT_SCATTER: (&str, &str) = ("14_11", "Real rounds scatter");
const AT_ANY: (&str, &str) = ("14_12", "in any settled crowd");
const AT_THREE_TIMES: (&str, &str) = ("14_12", "gains three times it each round");
const AT_FAIR: (&str, &str) = ("14_12", "A game is fair exactly when");
const AT_B26: (&str, &str) = ("14_12", "Game B's settled twenty-six show it");
const AT_BALANCES: (&str, &str) = ("14_13", "every edge balances on its own");
const AT_ROW0: (&str, &str) = ("14_13", "ten players times a tenth");
const AT_ROW1: (&str, &str) = ("14_13", "Four times three quarters");
const AT_ROW2: (&str, &str) = ("14_13", "And twelve times three quarters");
const AT_MULTIPLY: (&str, &str) = ("14_14", "Multiply the three balances together");
const AT_SAME_COUNTS: (&str, &str) = ("14_14", "the same three counts");
const AT_CANCEL: (&str, &str) = ("14_14", "so they cancel");
const AT_LEFT: (&str, &str) = ("14_14", "What's left is a tenth");
const AT_AGAINST: (&str, &str) = ("14_14", "against nine tenths");
const AT_BOTH: (&str, &str) = ("14_14", "Both come to nine over one hundred sixty");
const AT_ANY_GAME: (&str, &str) = ("14_15", "The same cancelling works for any game");
const AT_LETTERS: (&str, &str) = ("14_15", "Write each corner's weight");
const AT_NO_WEIGHT: (&str, &str) = ("14_15", "no weight vanishes");
const AT_CANCELS: (&str, &str) = ("14_15", "each one cancels");
const AT_AROUND: (&str, &str) = ("14_15", "winning three times in a row");
const AT_OTHER_WAY: (&str, &str) = ("14_15", "losing three times in a row the other way");
#[cfg(test)]
const CUES: [(&str, &str); 59] = [
    AT_QUESTION,
    AT_TEST,
    AT_WATCH,
    AT_ROUND,
    AT_SCALED,
    AT_C0,
    AT_C1,
    AT_C2,
    AT_STILL,
    AT_ONE_EDGE,
    AT_FWD0,
    AT_BACK0,
    AT_OTHER,
    AT_EDGE1,
    AT_EDGE2,
    AT_COUNT,
    AT_TWELVE,
    AT_CIRCULATING,
    AT_NET_FLOW,
    AT_DOLLAR,
    AT_HOW_MANY,
    AT_THIRTY_SIX,
    AT_WIN_LOSE,
    AT_SHARED,
    AT_AGAIN,
    AT_WHY,
    AT_BEHIND,
    AT_AHEAD,
    AT_KEEPS,
    AT_DISAGREED,
    AT_EXPECTED,
    AT_REAL,
    AT_CARRY,
    AT_CHANGES,
    AT_LOST,
    AT_THOUSAND,
    AT_AVERAGES,
    AT_GAINS,
    AT_SCATTER,
    AT_ANY,
    AT_THREE_TIMES,
    AT_FAIR,
    AT_B26,
    AT_BALANCES,
    AT_ROW0,
    AT_ROW1,
    AT_ROW2,
    AT_MULTIPLY,
    AT_SAME_COUNTS,
    AT_CANCEL,
    AT_LEFT,
    AT_AGAINST,
    AT_BOTH,
    AT_ANY_GAME,
    AT_LETTERS,
    AT_NO_WEIGHT,
    AT_CANCELS,
    AT_AROUND,
    AT_OTHER_WAY,
];

/// The panel's right edge: clear of B's triangle by 08's rule (its centre
/// less the triangle's radius, a corner's disc and a margin), so its corner
/// 2 and that corner's name stand right of every line.
pub(super) const PANEL_END: f64 = -0.85;
// the corner rows (08's), then each corner's count and share
const COUNT_X: f64 = -5.3;
const INFO_X: f64 = -4.35;
/// The edge table: its head, one row per edge (edge e, from corner e to e +
/// 1), the edge's name, its pair and its net.
pub(super) const HEAD_Y: f64 = 0.1;
pub(super) const TABLE_YS: [f64; 3] = [-0.35, -0.9, -1.45];
pub(super) const EDGE_X: f64 = PANEL_X;
pub(super) const CELL_X: f64 = -5.55;
const NET_X: f64 = -3.4;
/// An edge's name ends before the multiplication's "×" (centred at
/// [`TIMES_X`]) begins.
const EDGE_END: f64 = CELL_X - 0.5;
const TIMES_X: f64 = CELL_X - 0.3;
/// Lines under the table: sums, the real crowd's runs, the product row.
pub(super) const LINE_YS: [f64; 3] = [-2.15, -2.65, -3.15];
const RULE_Y: f64 = -1.8;
pub(super) const CELL_SIZE: f64 = 28.0;
pub(super) const LINE_SIZE: f64 = 26.0;
pub(super) const WORDS_SIZE: f64 = 20.0;
/// The gap between two pieces of a row of TeX, and how far a strike runs
/// past the piece it crosses.
pub(super) const PIECE_GAP: f64 = 0.12;
const STRIKE_PAD: f64 = 0.06;
/// The prediction's countdown, right of the triangle, clear of edge 0's
/// labels and under the badge.
pub(super) const THINK_AT: Vec3 = [5.5, 2.0, 0.0];
/// A label's size beside an arrow (as 06's and 11's).
const CHANCE_SIZE: f64 = 20.0;
/// Streams: the narrowest and widest band, its opacity (as 11's), and the
/// fixed width of a path round the triangle.
const STREAM_MIN_W: f64 = 1.5;
const STREAM_MAX_W: f64 = 16.0;
const STREAM_O: f64 = 0.7;
pub(super) const PATH_W: f64 = 4.0;
/// Seconds: a stream drawn, a puddle growing, the counting hold, a piece's
/// flight, a real round's change, inspection after a reveal.
const STREAM_S: f64 = 1.0;
const GROW_S: f64 = 0.6;
const COUNT_S: f64 = 3.0;
const FLY_S: f64 = 1.2;
pub(super) const CHANGE_S: f64 = 1.2;
pub(super) const INSPECT_S: f64 = 1.5;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    published_badge: String,
    round_badge: String,
    published_mix: Chances,
    mix: Chances,
    b: Chances,
    players: [String; 2],
    /// The mix's crowd of 1,418 and B's of 26: counts, shares as fractions
    /// of the crowd, and the shares as the rows print them.
    counts: [[i64; 3]; 2],
    shares: [[f64; 3]; 2],
    pct: [[String; 3]; 2],
    /// Each edge's pair and net, "147 - 135" and "= 12" (the mix's, then
    /// B's), and the streams' widths for the mix's.
    pairs: [[String; 3]; 2],
    nets: [[String; 3]; 2],
    flows: [[f64; 2]; 3],
    /// 14_3's two products of a chance and a count.
    edge0: [String; 2],
    /// 14_8: the crowd's dollars a round, and a player's cents.
    gain: [String; 2],
    keeps: String,
    /// The real round: its pairs, nets, the corners' changes and counts
    /// after, its gain; then the thousand rounds' averages.
    real_pairs: [String; 3],
    real_nets: [String; 3],
    real_changes: [String; 3],
    real_after: [String; 3],
    real_shares: [f64; 3],
    real_gain: String,
    thousand: [String; 3],
    /// gain = 3 × net flow, fair exactly at 0.
    rule: [String; 2],
    /// B's balances as pieces: count, × chance, =, count, × chance, = flow.
    balances: [[String; 6]; 3],
    /// 14_14's line: B's three forward chances, then its three back ones in
    /// the corners' order, and what each product comes to.
    forward: [String; 3],
    back: [String; 3],
    product: String,
}

pub(super) fn csv<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    values.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

/// A fraction as TeX: "1/10" → "\tfrac{1}{10}", a whole number as itself.
pub(super) fn frac_tex(x: Ratio) -> String {
    if x.is_integer() {
        x.num().to_string()
    } else {
        format!(r"\tfrac{{{}}}{{{}}}", x.num(), x.den())
    }
}

/// A count that must be whole players.
pub(super) fn whole(x: Ratio, what: &str) -> Result<i64, String> {
    if !x.is_integer() {
        return Err(format!("{what} is {x}, not whole players"));
    }
    i64::try_from(x.num()).map_err(|_| format!("{what} does not fit"))
}

/// A difference as TeX, a negative subtrahend in parentheses: "4 - (-17)".
fn minus_tex(a: i64, b: i64) -> String {
    if b < 0 {
        format!("{a} - ({b})")
    } else {
        format!("{a} - {b}")
    }
}

/// A sum of nets as TeX: "7 + 4 - 17".
fn sum_tex(xs: &[i64]) -> String {
    let mut out = String::new();
    for (k, &x) in xs.iter().enumerate() {
        match (k, x < 0) {
            (0, _) => out.push_str(&x.to_string()),
            (_, true) => out.push_str(&format!(" - {}", -x)),
            (_, false) => out.push_str(&format!(" + {x}")),
        }
    }
    out
}

/// Each edge's net: forward from corner e less back from corner e + 1.
pub(super) fn edge_nets(forward: &[i64; 3], back: &[i64; 3]) -> [i64; 3] {
    [0, 1, 2].map(|e| forward[e] - back[(e + 1) % 3])
}

/// A crowd's split flows (forward, back from each corner) as whole players.
pub(super) fn whole_flows(g: &Game, n: i64) -> Result<([i64; 3], [i64; 3]), String> {
    let (f, b) = model::flows(g, n);
    let [f0, f1, f2] = f.map(|x| whole(x, "a forward flow"));
    let [b0, b1, b2] = b.map(|x| whole(x, "a back flow"));
    Ok(([f0?, f1?, f2?], [b0?, b1?, b2?]))
}

/// The real crowd's crossings as signed counts.
fn signed(xs: [usize; 3]) -> Result<[i64; 3], String> {
    let [a, b, c] = xs.map(|x| i64::try_from(x).map_err(|_| "a count does not fit".to_owned()));
    Ok([a?, b?, c?])
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let (b, mix) = (ROUND.game_b(), ROUND.mix());
    // 14_2: the mix's settled crowd, scaled until every flow is whole
    let n_mix = model::smallest_whole_crowd(&mix);
    let n_b = model::smallest_whole_crowd(&b);
    if n_mix.to_string() != get("mixcrowd.size")? || n_b.to_string() != get("crowd.size")? {
        return Err("14_2 scales the mix's crowd to 1,418 players and 14_12 brings back B's 26".into());
    }
    let mut counts = [[0i64; 3]; 2];
    let mut shares = [[0f64; 3]; 2];
    let mut pct: [[String; 3]; 2] = Default::default();
    for (k, (g, n)) in [(&mix, n_mix), (&b, n_b)].into_iter().enumerate() {
        for (i, &w) in g.shares().iter().enumerate() {
            counts[k][i] = whole(Ratio::int(n) * w, "a corner's count")?;
            shares[k][i] = w.to_f64();
            pct[k][i] = format!("{} {PERCENT}", w.pct(1));
        }
    }
    if csv(counts[0]) != get("mixcrowd.counts")? || csv(counts[1]) != get("crowd.counts")? {
        return Err("14_2 says 490, 360 and 568, and 14_12 says 10, 4 and 12".into());
    }
    let shown = |k: usize| {
        pct[k]
            .iter()
            .map(|p| p.trim_end_matches(PERCENT).trim().to_owned())
            .collect::<Vec<_>>()
    };
    if shown(0).join(",") != get("round.mix.weights.pct")? || shown(1).join(",") != get("round.b.weights.pct")? {
        return Err("14_2 says thirty-four point six percent at corner zero".into());
    }
    // 14_3, 14_4: the pairs across each edge
    let (fm, bm) = whole_flows(&mix, n_mix)?;
    let (fb, bb) = whole_flows(&b, n_b)?;
    if csv(fm) != get("mixcrowd.forward")? || csv(bm) != get("mixcrowd.back")? {
        return Err("14_3 and 14_4 say 147, 225, 355 forward and 135, 213, 343 back".into());
    }
    if csv(fb) != get("crowd.forward")? || csv(bb) != get("crowd.back")? {
        return Err("14_12 says one and one, three and three, nine and nine".into());
    }
    let mix_nets = edge_nets(&fm, &bm);
    let b_nets = edge_nets(&fb, &bb);
    if csv(mix_nets) != get("mixcrowd.net")? || mix_nets.iter().any(|&x| x != mix_nets[0]) {
        return Err("14_6 says twelve on every edge".into());
    }
    if csv(b_nets) != get("crowd.net")? || b_nets.iter().any(|&x| x != 0) {
        return Err("14_12 says game B's settled crowd has no net flow".into());
    }
    let pair = |f: &[i64; 3], b: &[i64; 3], e: usize| format!("{} - {}", f[e], b[(e + 1) % 3]);
    let pairs = [
        [0, 1, 2].map(|e| pair(&fm, &bm, e)),
        [0, 1, 2].map(|e| pair(&fb, &bb, e)),
    ];
    let nets = [mix_nets.map(|x| format!("= {x}")), b_nets.map(|x| format!("= {x}"))];
    let flows = [0, 1, 2].map(|e| {
        [fm[e], bm[(e + 1) % 3]].map(|x| STREAM_MIN_W + (STREAM_MAX_W - STREAM_MIN_W) * x as f64 / n_mix as f64)
    });
    let mc = chances(&mix)?;
    if mc.heads[0] != get("round.mix.coins.pct")?.split(',').next().unwrap_or_default()
        || mc.tails[1] != get("round.mix.tails.pct")?.split(',').nth(1).unwrap_or_default()
    {
        return Err("14_3 says thirty percent and thirty-seven and a half percent".into());
    }
    let edge0 = [
        format!(r"{}\% \times {} = {}", mc.heads[0], counts[0][0], fm[0]),
        format!(r"{}\% \times {} = {}", mc.tails[1], counts[0][1], bm[1]),
    ];
    // 14_8: the crowd's dollars a round, and a player's cents
    let (winners, losers): (i64, i64) = (fm.iter().sum(), bm.iter().sum());
    let gain = winners - losers;
    let three = model::MODULUS;
    if gain != three * mix_nets[0]
        || gain.to_string() != get("mixcrowd.gain")?
        || winners.to_string() != get("mixcrowd.winners")?
        || losers.to_string() != get("mixcrowd.losers")?
    {
        return Err("14_8 says three times twelve, 36 = 727 - 691".into());
    }
    let per_player = Ratio::new(gain.into(), n_mix.into());
    let cents = per_player.cents(2);
    if per_player != mix.mean() || cents != get("round.mix.mean.cents")? {
        return Err("14_8 says the mix's average once again".into());
    }
    let gain_tex = [
        format!(r"{three} \times {} = {gain} = {winners} - {losers}", mix_nets[0]),
        format!(
            r"{gain} \text{{ dollars}} / {} \approx {} \text{{ cents}}",
            screen_int(n_mix).replace(',', "{,}"),
            cents.trim_start_matches('+')
        ),
    ];
    // 14_9: corner 1 gets the net of the edge behind it and sends the net of
    // the edge ahead; its count stays
    if counts[0][1] + mix_nets[0] - mix_nets[1] != counts[0][1] {
        return Err("14_9 says corner one keeps its 360".into());
    }
    let keeps = format!(r"{c} + {} - {} = {c}", mix_nets[0], mix_nets[1], c = counts[0][1]);
    // 14_10: one real round of 1,418 real players from 490, 360, 568, the
    // video's seed; 14_11: a thousand of them
    let start = counts[0].map(|c| usize::try_from(c).unwrap_or(0));
    let real = model::real_crossings(&mix, start, facts::REAL_ROUNDS, SEED);
    let first = real.first().ok_or("no real rounds")?;
    let (rf, rb) = (signed(first[0])?, signed(first[1])?);
    let real_nets = edge_nets(&rf, &rb);
    let after = [0, 1, 2].map(|i| rf[(i + 2) % 3] + rb[(i + 1) % 3]);
    let change = [0, 1, 2].map(|i| real_nets[(i + 2) % 3] - real_nets[i]);
    if (0..3).any(|i| after[i] - counts[0][i] != change[i]) || after.iter().sum::<i64>() != n_mix {
        return Err("14_10 says each corner changes by net in minus net out".into());
    }
    if csv(rf) != get("realmix.round.1.forward")?
        || csv(rb) != get("realmix.round.1.back")?
        || csv(real_nets) != get("realmix.round.1.net")?
        || csv(after) != get("realmix.round.1.after")?
    {
        return Err("14_10 shows the real round the facts print".into());
    }
    if real_nets.iter().all(|&x| x == real_nets[0]) {
        return Err("14_10 says a real round's edges disagree".into());
    }
    let real_gain: i64 = real_nets.iter().sum();
    if real_gain.to_string() != get("realmix.round.1.gain")? || real_gain >= 0 {
        return Err("14_10 says this round the crowd lost dollars".into());
    }
    let real_pairs = [0, 1, 2].map(|e| pair(&rf, &rb, e));
    let real_changes =
        [0, 1, 2].map(|i| format!("{} = {}", minus_tex(real_nets[(i + 2) % 3], real_nets[i]), change[i]));
    let mut totals = [0i64; 3];
    let mut gained = 0i64;
    for round in &real {
        let nets = edge_nets(&signed(round[0])?, &signed(round[1])?);
        for (t, x) in totals.iter_mut().zip(nets) {
            *t += x;
        }
        gained += nets.iter().sum::<i64>();
    }
    let rounds = i128::try_from(real.len()).map_err(|_| "too many rounds")?;
    let mean_net = totals.map(|t| Ratio::new(t.into(), rounds).fixed(1));
    let mean_gain = Ratio::new(gained.into(), rounds).fixed(1);
    if csv(&mean_net) != get("realmix.mean.net")?
        || mean_gain != get("realmix.mean.gain")?
        || real.len().to_string() != get("realmix.rounds")?
    {
        return Err("14_11 says eleven point nine on each edge and 35.8 dollars a round".into());
    }
    let thousand = [
        format!("{} {THOUSAND_WORDS}", screen_int(real.len() as i64)),
        format!("{AVERAGE_NET} {}", mean_net.join(", ")),
        format!("{AVERAGE_GAIN} {mean_gain} {DOLLARS_A_ROUND}"),
    ];
    // 14_12: three times the net flow, fair exactly when it is zero
    if b.mean() != Ratio::ZERO || get("round.b.mean")? != "0" {
        return Err("14_12 says game B with the round coins is fair".into());
    }
    let rule = [
        format!(r"\text{{gain per round}} = {three} \times \text{{net flow}}"),
        r"\text{fair} \iff \text{net flow} = 0".to_owned(),
    ];
    // 14_13, 14_14: B's balances, and their product with the counts gone
    let p = [0, 1, 2].map(|i| b.chance(i));
    let q = p.map(|x| Ratio::ONE - x);
    for e in 0..3 {
        let j = (e + 1) % 3;
        if Ratio::int(counts[1][e]) * p[e] != Ratio::int(counts[1][j]) * q[j] {
            return Err(format!("14_13 says edge {e} balances"));
        }
    }
    let balances = [0, 1, 2].map(|e| {
        let j = (e + 1) % 3;
        [
            counts[1][e].to_string(),
            format!(r"\times {}", frac_tex(p[e])),
            "=".to_owned(),
            counts[1][j].to_string(),
            format!(r"\times {}", frac_tex(q[j])),
            format!("= {}", fb[e]),
        ]
    });
    let (pf, pb) = b.products();
    if pf != p[0] * p[1] * p[2] || pb != q[0] * q[1] * q[2] || pf != pb {
        return Err("14_14 says both products come to the same".into());
    }
    if get("round.products.b")? != csv([pf, pb]) {
        return Err("14_14 says nine over one hundred sixty".into());
    }
    Ok(Numbers {
        published_badge: badge_text(&f)?,
        round_badge: round_badge_text(&f)?,
        published_mix: chances(&PUBLISHED.mix())?,
        mix: mc,
        b: chances(&b)?,
        players: [screen_int(n_mix), screen_int(n_b)],
        counts,
        shares,
        pct,
        pairs,
        nets,
        flows,
        edge0,
        gain: gain_tex,
        keeps,
        real_pairs,
        real_nets: real_nets.map(|x| format!("= {x}")),
        real_changes,
        real_after: after.map(screen_int),
        real_shares: after.map(|c| c as f64 / n_mix as f64),
        real_gain: format!(r"{} = {real_gain} \text{{ dollars}}", sum_tex(&real_nets)),
        thousand,
        rule,
        balances,
        forward: p.map(frac_tex),
        back: q.map(frac_tex),
        product: format!("{} = {}", frac_tex(pf), frac_tex(pb)),
    })
}

// --------------------------------------------------------------- pictures

/// A label for the arrow along edge `e` of B's triangle, at its place beside
/// the arrow (as 11's). Detached.
pub(super) fn chance_label(
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

/// The mix's six labels, as 11 left them: gold forward along edges 0, 1, 2,
/// then dim back along them. Detached.
pub(super) fn mix_labels(kit: &Kit, stage: &mut Stage<'_>, c: &Chances) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(6);
    for (e, head) in c.heads.iter().enumerate() {
        out.push(chance_label(
            kit,
            stage,
            &format!("{head} {PERCENT}"),
            palette::mix(),
            e,
            true,
        )?);
    }
    for e in 0..3 {
        let words = format!("{} {PERCENT}", c.tails[(e + 1) % 3]);
        out.push(chance_label(kit, stage, &words, Palette::dim(), e, false)?);
    }
    Ok(out)
}

/// The mix's gold title under the triangle. Detached.
pub(super) fn mix_title(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text(stage, MIX_NAME, 24.0, palette::mix(), v(B_AT[0], B_AT[1] - TITLE_DY))
}

/// The crowd's puddles: a tracker per corner holding its share of the
/// crowd, and a teal disc drawn from it, area in proportion (as 11's).
pub(super) struct Puddles {
    pub share: [Mob; 3],
}

/// Puddles at `shares`, added to the scene.
pub(super) fn puddles(stage: &mut Stage<'_>, shares: [f64; 3]) -> fmn::Result<Puddles> {
    let share = shares.map(|x| stage.add_value_tracker(x));
    for (i, &t) in share.iter().enumerate() {
        let at = corner(B_AT, i);
        let m = stage.always_redraw(move |s| {
            let x = s.tracker_value(t).unwrap_or(0.0).clamp(0.0, 1.0);
            s.add(
                Circle::new()
                    .radius((CORNER_R * x.sqrt()).max(0.004))
                    .arc_center(at)
                    .style(flat(palette::player(), 0.85))
                    .build(),
            )
        });
        stage.add_to_scene(m)?;
    }
    Ok(Puddles { share })
}

impl Puddles {
    /// The puddles' trackers gliding to `shares`, `rt` seconds.
    pub(super) fn to(&self, shares: [f64; 3], rt: f64) -> fmn::Result<Vec<AnimBuilder>> {
        self.share.iter().zip(shares).map(|(&t, x)| tween(t, x, rt)).collect()
    }
}

/// A band along the arrow of edge `e` (forward or back), `width` wide, in
/// `color` at opacity `o`. Detached.
pub(super) fn stream(stage: &mut Stage<'_>, e: usize, forward: bool, width: f64, color: Srgb, o: f64) -> Mob {
    let style = Style::default().fill(color, 0.0).stroke(color, width, o);
    stage.arena_mut().add(polyline(shaft(e, forward)).style(style).build())
}

/// Draw `bands` along their paths together, `rt` seconds.
pub(super) fn draw_bands(stage: &mut Stage<'_>, bands: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(bands.len());
    for &m in bands {
        anims.push(stage.prepare(show_creation(m).rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// A row of TeX pieces, each typeset alone and set left to right from
/// `left` with [`PIECE_GAP`] between, centred on `y`. Detached.
pub(super) fn pieces(
    kit: &Kit,
    stage: &mut Stage<'_>,
    srcs: &[String],
    colors: &[Srgb],
    size: f64,
    left: f64,
    y: f64,
) -> fmn::Result<Vec<Mob>> {
    if srcs.len() != colors.len() {
        return Err(fail("a row of pieces needs a colour for each".into()));
    }
    let mut x = left;
    let mut out = Vec::with_capacity(srcs.len());
    for (src, &color) in srcs.iter().zip(colors) {
        let m = kit.tex(stage, src, size, color, &[], ORIGIN)?;
        stage.move_to(m, v(x, y), LEFT);
        x += stage.get_width(m) + PIECE_GAP;
        out.push(m);
    }
    Ok(out)
}

/// A red stroke through `m` from its lower left to its upper right: a
/// factor cancelled. Detached, and allowed to cross the text it strikes.
pub(super) fn strike(kit: &Kit, stage: &mut Stage<'_>, m: Mob) -> Mob {
    let (l, r) = (
        stage.get_bounding_box_point(m, LEFT)[0],
        stage.get_bounding_box_point(m, RIGHT)[0],
    );
    let (b, t) = (
        stage.get_bounding_box_point(m, DOWN)[1],
        stage.get_bounding_box_point(m, UP)[1],
    );
    let s = polyline([v(l - STRIKE_PAD, b), v(r + STRIKE_PAD, t)])
        .style(line_style(palette::bad(), 3.0))
        .build();
    let s = stage.arena_mut().add(s);
    kit.allow_crossing(s);
    s
}

/// Strike every one of `mobs`, then let them and their strikes go.
pub(super) fn cancel(kit: &Kit, stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let strikes: Vec<Mob> = mobs.iter().map(|&m| strike(kit, stage, m)).collect();
    draw_bands(stage, &strikes, 0.5)?;
    pause(stage, 0.6)?;
    let mut gone = mobs.to_vec();
    gone.extend(strikes);
    fade_out_all(stage, &gone, 0.6)
}

/// A row built partly from copies of pieces already on screen: the row
/// `srcs` is set at `left`, `y`; the pieces at `moving` start over their
/// `sources` (right edges matched, since a source's "×" is left of its
/// fraction), come up there and fly into place together; the rest (dots,
/// "=") then fade in. Returns the whole row.
#[allow(clippy::too_many_arguments)]
pub(super) fn gather(
    kit: &Kit,
    stage: &mut Stage<'_>,
    srcs: &[String],
    colors: &[Srgb],
    moving: &[usize],
    sources: &[Mob],
    left: f64,
    y: f64,
) -> fmn::Result<Vec<Mob>> {
    if moving.len() != sources.len() {
        return Err(fail("every gathered piece needs its source".into()));
    }
    let row = pieces(kit, stage, srcs, colors, CELL_SIZE, left, y)?;
    let homes: Vec<Vec3> = row.iter().map(|&m| stage.get_center(m)).collect();
    for (&k, &src) in moving.iter().zip(sources) {
        let edge = stage.get_bounding_box_point(src, RIGHT);
        stage.move_to(row[k], edge, RIGHT);
    }
    let flying: Vec<Mob> = moving.iter().map(|&k| row[k]).collect();
    fade_in_all(stage, &flying, 0.3)?;
    let moves = moving
        .iter()
        .map(|&k| fly(row[k], homes[k], FLY_S))
        .collect::<fmn::Result<Vec<_>>>()?;
    together(stage, moves)?;
    let rest: Vec<Mob> = (0..row.len()).filter(|k| !moving.contains(k)).map(|k| row[k]).collect();
    fade_in_all(stage, &rest, 0.4)?;
    Ok(row)
}

/// A line of the panel's words, left at [`PANEL_X`]. Detached.
pub(super) fn panel_words(kit: &Kit, stage: &mut Stage<'_>, words: &str, color: Srgb, y: f64) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, words, WORDS_SIZE, color, v(PANEL_X, y))?;
    keep_between(stage, m, PANEL_X - 0.01, PANEL_END, "a line of the panel")?;
    Ok(m)
}

/// A line of the panel's TeX, left at `left`. Detached.
pub(super) fn panel_tex(
    kit: &Kit,
    stage: &mut Stage<'_>,
    src: &str,
    color: Srgb,
    left: f64,
    y: f64,
) -> fmn::Result<Mob> {
    let m = kit.tex_left(stage, src, LINE_SIZE, color, &[], v(left, y))?;
    keep_between(stage, m, left - 0.01, PANEL_END, "a sum in the panel")?;
    Ok(m)
}

/// `words` (dim) just right of `m`, on its line: a unit after a sum.
/// Detached.
pub(super) fn unit_after(kit: &Kit, stage: &mut Stage<'_>, m: Mob, words: &str) -> fmn::Result<Mob> {
    let u = kit.text_left(stage, words, 18.0, Palette::dim(), ORIGIN)?;
    let right = stage.get_bounding_box_point(m, RIGHT)[0];
    let y = stage.get_center(m)[1];
    stage.move_to(u, v(right + 0.2, y), LEFT);
    keep_between(stage, u, right, PANEL_END, "a unit")?;
    Ok(u)
}

/// The edge table's head. Detached.
fn table_head(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 3]> {
    Ok([
        kit.text_left(stage, EDGE_HEAD, 18.0, Palette::dim(), v(EDGE_X, HEAD_Y))?,
        kit.text_left(stage, PAIR_HEAD, 18.0, Palette::dim(), v(CELL_X, HEAD_Y))?,
        kit.text_left(stage, NET_HEAD, 18.0, Palette::dim(), v(NET_X, HEAD_Y))?,
    ])
}

/// Each edge's name, "0 → 1", at its row of `ys`. Detached.
pub(super) fn edge_names(kit: &Kit, stage: &mut Stage<'_>, ys: &[f64; 3]) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(3);
    for (e, &y) in ys.iter().enumerate() {
        let words = format!("{e} {ARROW} {}", (e + 1) % 3);
        let m = kit.text_left(stage, &words, WORDS_SIZE, Palette::ink(), v(EDGE_X, y))?;
        keep_between(stage, m, EDGE_X - 0.01, EDGE_END, "an edge's name")?;
        out.push(m);
    }
    Ok(out)
}

/// The table's cells for `pairs`, in `color`. Detached.
fn cells(kit: &Kit, stage: &mut Stage<'_>, pairs: &[String; 3], color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(3);
    for (e, src) in pairs.iter().enumerate() {
        let m = kit.tex_left(stage, src, CELL_SIZE, color, &[], v(CELL_X, TABLE_YS[e]))?;
        keep_between(stage, m, CELL_X - 0.01, NET_X - 0.1, "an edge's pair")?;
        out.push(m);
    }
    Ok(out)
}

/// The table's net column for `nets`, in `color`. Detached.
fn net_cells(kit: &Kit, stage: &mut Stage<'_>, nets: &[String; 3], color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(3);
    for (e, src) in nets.iter().enumerate() {
        let m = kit.tex_left(stage, src, CELL_SIZE, color, &[], v(NET_X, TABLE_YS[e]))?;
        keep_between(stage, m, NET_X - 0.01, PANEL_END, "an edge's net")?;
        out.push(m);
    }
    Ok(out)
}

/// The corner rows' count or share column. Detached.
fn corner_column(kit: &Kit, stage: &mut Stage<'_>, words: &[String; 3], x: f64, right: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(3);
    for (i, w) in words.iter().enumerate() {
        let m = kit.text_left(stage, w, WORDS_SIZE, palette::player(), v(x, ROW_YS[i]))?;
        keep_between(stage, m, x - 0.01, right, "a corner's number")?;
        out.push(m);
    }
    Ok(out)
}

/// The same, TeX (the real round's changes). Detached.
fn corner_tex(kit: &Kit, stage: &mut Stage<'_>, srcs: &[String; 3], x: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(3);
    for (i, src) in srcs.iter().enumerate() {
        let m = kit.tex_left(stage, src, 24.0, palette::player(), &[], v(x, ROW_YS[i]))?;
        keep_between(stage, m, x - 0.01, PANEL_END, "a corner's change")?;
        out.push(m);
    }
    Ok(out)
}

impl SceneConstruct for NetFlow {
    fn name(&self) -> &str {
        "14_net_flow"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let to_s = |xs: &[i64; 3]| xs.map(screen_int);

        // 14_1: a new question; both the badge and the mix's chances keep
        // the published coins from 13 until 14_2 explicitly returns to ROUND.
        let line = say(&kit, stage, "14_1")?;
        let pub_tag = full_badge(&kit, stage, &n.published_badge, PANEL_END)?;
        let t = triangle(&kit, stage, Which::B)?;
        let title = mix_title(&kit, stage)?;
        let labels = mix_labels(&kit, stage, &n.published_mix)?;
        let mut arriving = vec![pub_tag[0], pub_tag[1], title];
        arriving.extend(t.discs);
        arriving.extend(t.names);
        arriving.extend(t.forward.iter().flatten());
        arriving.extend(t.back.iter().flatten());
        arriving.extend(&labels);
        fade_in_all(stage, &arriving, 0.8)?;
        let mut texts = labels.clone();
        texts.extend(t.names);
        texts.push(title);
        keep_apart(stage, &texts, 0.08, "the mix's triangle")?;
        line.wait_for(stage, AT_QUESTION.1)?;
        let shafts: Vec<Mob> = t.forward.iter().chain(&t.back).map(|a| a[0]).collect();
        line.wait_for(stage, AT_TEST.1)?;
        let pointer = frame_around(stage, &labels, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_WATCH.1)?;
        flash_all(stage, &shafts, 0.8)?;

        // 14_2: back to the round coins; the mix's crowd of 1,418 in whole
        // players, each corner's puddle growing as its count is said
        let line = say(&kit, stage, "14_2")?;
        line.wait_for(stage, AT_ROUND.1)?;
        let tag = round_badge(&kit, stage, &n.round_badge)?;
        let mut published = pub_tag.to_vec();
        published.extend(&labels);
        let labels = mix_labels(&kit, stage, &n.mix)?;
        let mut rounded = tag.to_vec();
        rounded.extend(&labels);
        let anims = swap(stage, &published, &rounded, 0.7)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_SCALED.1)?;
        let head = panel_head(&kit, stage)?;
        let players = kit.text_left(stage, &n.players[0], 22.0, Palette::ink(), v(PLAYERS_NUM_X, PLAYERS_Y))?;
        fade_in_all(stage, &[head[0], head[1], players], 0.5)?;
        let crowd = puddles(stage, [0.0; 3])?;
        let names: Vec<Mob> = (0..3).map(|i| row_name(&kit, stage, i)).collect::<fmn::Result<_>>()?;
        let mut counts = corner_column(&kit, stage, &to_s(&n.counts[0]), COUNT_X, INFO_X - 0.1)?;
        let mut info = corner_column(&kit, stage, &n.pct[0], INFO_X, PANEL_END)?;
        for (k, at) in [AT_C0, AT_C1, AT_C2].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            fade_in_all(stage, &[names[k], counts[k]], 0.4)?;
            stage.play(tween(crowd.share[k], n.shares[0][k], GROW_S)?)?;
        }
        line.wait_for(stage, AT_STILL.1)?;
        fade_in_all(stage, &info, 0.5)?;
        let pointer = frame_around(stage, &info[..1], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut panel = vec![head[0], head[1], players];
        panel.extend(&names);
        panel.extend(&counts);
        panel.extend(&info);
        keep_apart(stage, &panel, 0.06, "the crowd's panel")?;

        // 14_3: one edge at a time: edge 0 → 1, its two streams and their
        // products of a chance and a count
        let line = say(&kit, stage, "14_3")?;
        line.wait_for(stage, AT_ONE_EDGE.1)?;
        flash_all(stage, &[t.forward[0][0], t.back[0][0]], 0.8)?;
        line.wait_for(stage, AT_FWD0.1)?;
        let mut bands = Vec::with_capacity(6);
        let band = stream(stage, 0, true, n.flows[0][0], palette::player(), STREAM_O);
        draw_bands(stage, &[band], STREAM_S)?;
        bands.push(band);
        let sum0 = panel_tex(&kit, stage, &n.edge0[0], Palette::ink(), PANEL_X, HEAD_Y)?;
        fade_in_all(stage, &[sum0], 0.5)?;
        line.wait_for(stage, AT_BACK0.1)?;
        let band = stream(stage, 0, false, n.flows[0][1], palette::player(), STREAM_O);
        draw_bands(stage, &[band], STREAM_S)?;
        bands.push(band);
        let sum1 = panel_tex(&kit, stage, &n.edge0[1], Palette::ink(), PANEL_X, TABLE_YS[0])?;
        fade_in_all(stage, &[sum1], 0.5)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 14_4: the other two edges; the sums give way to the table, a row
        // as each edge is said
        let line = say(&kit, stage, "14_4")?;
        line.wait_for(stage, AT_OTHER.1)?;
        let head_row = table_head(&kit, stage)?;
        let edges = edge_names(&kit, stage, &TABLE_YS)?;
        let mut pairs = cells(&kit, stage, &n.pairs[0], Palette::ink())?;
        let mut first_row = head_row.to_vec();
        first_row.extend([edges[0], pairs[0]]);
        let anims = swap(stage, &[sum0, sum1], &first_row, 0.6)?;
        stage.play_prepared(anims)?;
        for (e, at) in [(1, AT_EDGE1), (2, AT_EDGE2)] {
            line.wait_for(stage, at.1)?;
            let pair = [
                stream(stage, e, true, n.flows[e][0], palette::player(), STREAM_O),
                stream(stage, e, false, n.flows[e][1], palette::player(), STREAM_O),
            ];
            draw_bands(stage, &pair, STREAM_S)?;
            bands.extend(pair);
            fade_in_all(stage, &[edges[e], pairs[e]], 0.4)?;
        }

        // 14_5: count the surplus on each edge (held; nothing subtracted on
        // screen)
        let line = say(&kit, stage, "14_5")?;
        line.wait_for(stage, AT_COUNT.1)?;
        let pointer = frame_around(stage, &head_row[1..2], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;
        let pointer = frame_around(stage, &pairs, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        pause(stage, COUNT_S)?;

        // 14_6: twelve, every time; the crowd circulates forward
        let line = say(&kit, stage, "14_6")?;
        line.wait_for(stage, AT_TWELVE.1)?;
        let mut nets = net_cells(&kit, stage, &n.nets[0], palette::mix())?;
        for &m in &nets {
            fade_in_all(stage, &[m], 0.35)?;
        }
        line.wait_for(stage, AT_CIRCULATING.1)?;
        fade_out_all(stage, &bands, 0.4)?;
        let paths: Vec<Mob> = (0..3)
            .map(|e| stream(stage, e, true, PATH_W, palette::mix(), 0.9))
            .collect();
        draw_bands(stage, &paths, 1.4)?;
        line.wait_for(stage, AT_NET_FLOW.1)?;
        let net_head = kit.text_left(stage, NET_FLOW, 18.0, palette::mix(), v(NET_X, HEAD_Y))?;
        keep_between(stage, net_head, NET_X - 0.01, PANEL_END, "the net column's head")?;
        let anims = swap(stage, &head_row[2..], &[net_head], 0.5)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &nets, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut whole_table = panel.clone();
        whole_table.extend([head_row[0], head_row[1], net_head]);
        whole_table.extend(&edges);
        whole_table.extend(&pairs);
        whole_table.extend(&nets);
        keep_apart(stage, &whole_table, 0.06, "the edge table")?;

        // 14_7: a dollar a step; what does the crowd gain in a round? (a
        // prediction: nothing summed)
        let line = say(&kit, stage, "14_7")?;
        line.wait_for(stage, AT_DOLLAR.1)?;
        flash_all(stage, &paths, 0.8)?;
        line.wait_for(stage, AT_HOW_MANY.1)?;
        let question = panel_words(&kit, stage, ASK, Palette::ink(), LINE_YS[0])?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 14_8: 36 dollars a round, 727 against 691; shared among the
        // players, the mix's average in cents
        let line = say(&kit, stage, "14_8")?;
        line.wait_for(stage, AT_THIRTY_SIX.1)?;
        let gain = panel_tex(&kit, stage, &n.gain[0], palette::mix(), PANEL_X, LINE_YS[0])?;
        let anims = swap(stage, &[question, q], &[gain], 0.5)?;
        stage.play_prepared(anims)?;
        let unit = unit_after(&kit, stage, gain, DOLLARS_A_ROUND)?;
        fade_in_all(stage, &[unit], 0.4)?;
        line.wait_for(stage, AT_WIN_LOSE.1)?;
        let pointer = frame_around(stage, &[gain], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_SHARED.1)?;
        let each = panel_tex(&kit, stage, &n.gain[1], palette::mix(), PANEL_X, LINE_YS[1])?;
        let each_unit = unit_after(&kit, stage, each, EACH)?;
        fade_in_all(stage, &[each, each_unit], 0.5)?;
        line.wait_for(stage, AT_AGAIN.1)?;
        let pointer = frame_around(stage, &[each, each_unit], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut sums = whole_table.clone();
        sums.extend([gain, unit, each, each_unit]);
        keep_apart(stage, &sums, 0.06, "the crowd's gain")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 14_9: why twelve on every edge: corner 1 gets twelve net from the
        // edge behind it and sends twelve net across the edge ahead
        let line = say(&kit, stage, "14_9")?;
        line.wait_for(stage, AT_WHY.1)?;
        fade_out_all(stage, &[gain, unit, each, each_unit], 0.4)?;
        ring_corners(stage, &[t.discs[1]], &[], 0.8)?;
        line.wait_for(stage, AT_BEHIND.1)?;
        let pointer = frame_around(stage, &[edges[0], pairs[0], nets[0]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[paths[0]], 0.9)?;
        line.wait_for(stage, AT_AHEAD.1)?;
        let pointer = frame_around(stage, &[edges[1], pairs[1], nets[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[paths[1]], 0.9)?;
        line.wait_for(stage, AT_KEEPS.1)?;
        let corner_one = panel_words(&kit, stage, CORNER_ONE, Palette::ink(), LINE_YS[0])?;
        let one_right = stage.get_bounding_box_point(corner_one, RIGHT)[0];
        let keeps = panel_tex(&kit, stage, &n.keeps, palette::player(), one_right + 0.2, LINE_YS[0])?;
        fade_in_all(stage, &[corner_one, keeps], 0.5)?;
        line.wait_for(stage, AT_DISAGREED.1)?;
        let pointer = frame_around(stage, &[counts[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 14_10: these are expected numbers; one real round of real players
        let line = say(&kit, stage, "14_10")?;
        line.wait_for(stage, AT_EXPECTED.1)?;
        fade_out_all(stage, &[corner_one, keeps], 0.4)?;
        let pointer = frame_around(stage, &[head[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.9)?;
        line.wait_for(stage, AT_REAL.1)?;
        let real_label = kit.text_left(stage, REAL_LABEL, 18.0, palette::player(), v(PANEL_X, LABEL_Y))?;
        keep_between(stage, real_label, PANEL_X - 0.01, PANEL_END, "the real round's label")?;
        let real_pairs = cells(&kit, stage, &n.real_pairs, palette::player())?;
        let mut old = vec![head[1]];
        old.extend(&pairs);
        old.extend(&nets);
        old.extend(&paths);
        let mut new = vec![real_label];
        new.extend(&real_pairs);
        let anims = swap(stage, &old, &new, 0.7)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_CARRY.1)?;
        let real_nets = net_cells(&kit, stage, &n.real_nets, palette::player())?;
        for &m in &real_nets {
            fade_in_all(stage, &[m], 0.35)?;
        }
        line.wait_for(stage, AT_CHANGES.1)?;
        let real_after = corner_column(&kit, stage, &n.real_after, COUNT_X, INFO_X - 0.1)?;
        let real_changes = corner_tex(&kit, stage, &n.real_changes, INFO_X)?;
        let mut old = counts.clone();
        old.extend(&info);
        let mut new = real_after.clone();
        new.extend(&real_changes);
        let mut anims = swap(stage, &old, &new, CHANGE_S)?;
        for m in crowd.to(n.real_shares, CHANGE_S)? {
            anims.push(stage.prepare(m)?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_LOST.1)?;
        let lost = panel_tex(&kit, stage, &n.real_gain, palette::bad(), PANEL_X, LINE_YS[0])?;
        fade_in_all(stage, &[lost], 0.5)?;
        let mut real_panel = vec![head[0], players, real_label, head_row[0], head_row[1], net_head, lost];
        real_panel.extend(&names);
        real_panel.extend(&real_after);
        real_panel.extend(&real_changes);
        real_panel.extend(&edges);
        real_panel.extend(&real_pairs);
        real_panel.extend(&real_nets);
        keep_apart(stage, &real_panel, 0.06, "the real round's panel")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 14_11: a thousand real rounds average close to the expected
        // numbers, which return
        let line = say(&kit, stage, "14_11")?;
        line.wait_for(stage, AT_THOUSAND.1)?;
        let label = kit.text_left(stage, SPLIT_LABEL, 18.0, Palette::dim(), v(PANEL_X, LABEL_Y))?;
        pairs = cells(&kit, stage, &n.pairs[0], Palette::ink())?;
        nets = net_cells(&kit, stage, &n.nets[0], palette::mix())?;
        counts = corner_column(&kit, stage, &to_s(&n.counts[0]), COUNT_X, INFO_X - 0.1)?;
        info = corner_column(&kit, stage, &n.pct[0], INFO_X, PANEL_END)?;
        let runs = panel_words(&kit, stage, &n.thousand[0], palette::player(), LINE_YS[0])?;
        let mut old = vec![real_label, lost];
        old.extend(&real_pairs);
        old.extend(&real_nets);
        old.extend(&real_after);
        old.extend(&real_changes);
        let mut new = vec![label, runs];
        new.extend(&pairs);
        new.extend(&nets);
        new.extend(&counts);
        new.extend(&info);
        let mut anims = swap(stage, &old, &new, CHANGE_S)?;
        for m in crowd.to(n.shares[0], CHANGE_S)? {
            anims.push(stage.prepare(m)?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_AVERAGES.1)?;
        let averages = panel_words(&kit, stage, &n.thousand[1], palette::player(), LINE_YS[1])?;
        fade_in_all(stage, &[averages], 0.5)?;
        line.wait_for(stage, AT_GAINS.1)?;
        let gains = panel_words(&kit, stage, &n.thousand[2], palette::player(), LINE_YS[2])?;
        fade_in_all(stage, &[gains], 0.5)?;
        line.wait_for(stage, AT_SCATTER.1)?;
        let pointer = frame_around(stage, &[label], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 14_12: three times the net flow; fair exactly at zero; game B's
        // settled 26 (its title, chances, puddles, counts and pairs)
        let line = say(&kit, stage, "14_12")?;
        line.wait_for(stage, AT_ANY.1)?;
        fade_out_all(stage, &[runs, averages, gains], 0.4)?;
        let pointer = frame_around(stage, &nets, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_THREE_TIMES.1)?;
        let rule = panel_tex(&kit, stage, &n.rule[0], palette::mix(), PANEL_X, LINE_YS[0])?;
        fade_in_all(stage, &[rule], 0.5)?;
        line.wait_for(stage, AT_FAIR.1)?;
        let fair = panel_tex(&kit, stage, &n.rule[1], Palette::ink(), PANEL_X, LINE_YS[1])?;
        fade_in_all(stage, &[fair], 0.5)?;
        line.wait_for(stage, AT_B26.1)?;
        let b_labels = chance_labels(&kit, stage, Which::B, &n.b)?;
        let b_players = kit.text_left(stage, &n.players[1], 22.0, Palette::ink(), v(PLAYERS_NUM_X, PLAYERS_Y))?;
        let b_counts = corner_column(&kit, stage, &to_s(&n.counts[1]), COUNT_X, INFO_X - 0.1)?;
        let b_info = corner_column(&kit, stage, &n.pct[1], INFO_X, PANEL_END)?;
        let b_pairs = cells(&kit, stage, &n.pairs[1], Palette::ink())?;
        let b_nets = net_cells(&kit, stage, &n.nets[1], palette::mix())?;
        let mut old = vec![title, players];
        old.extend(&labels);
        old.extend(&counts);
        old.extend(&info);
        old.extend(&pairs);
        old.extend(&nets);
        let mut new = vec![t.title, b_players];
        new.extend(b_labels);
        new.extend(&b_counts);
        new.extend(&b_info);
        new.extend(&b_pairs);
        new.extend(&b_nets);
        let mut anims = swap(stage, &old, &new, CHANGE_S)?;
        for m in crowd.to(n.shares[1], CHANGE_S)? {
            anims.push(stage.prepare(m)?);
        }
        stage.play_prepared(anims)?;
        let mut b_texts = b_labels.to_vec();
        b_texts.extend(t.names);
        b_texts.push(t.title);
        keep_apart(stage, &b_texts, 0.08, "game B's triangle")?;
        let pointer = frame_around(stage, &b_nets, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 14_13: each edge balances on its own: B's three balances, a row
        // each, while its arrows pulse
        let line = say(&kit, stage, "14_13")?;
        line.wait_for(stage, AT_BALANCES.1)?;
        let balance_head = kit.text_left(stage, BALANCES_HEAD, 18.0, Palette::dim(), v(CELL_X, HEAD_Y))?;
        keep_between(stage, balance_head, CELL_X - 0.01, PANEL_END, "the balances' head")?;
        let mut old = vec![head_row[1], net_head, rule, fair];
        old.extend(&b_pairs);
        old.extend(&b_nets);
        let anims = swap(stage, &old, &[balance_head], 0.6)?;
        stage.play_prepared(anims)?;
        let colors = [
            palette::player(),
            palette::good(),
            Palette::ink(),
            palette::player(),
            Palette::dim(),
            Palette::dim(),
        ];
        let mut rows = Vec::with_capacity(3);
        for (e, at) in [AT_ROW0, AT_ROW1, AT_ROW2].iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let row = pieces(&kit, stage, &n.balances[e], &colors, CELL_SIZE, CELL_X, TABLE_YS[e])?;
            if let Some(&last) = row.last() {
                keep_between(stage, last, CELL_X, PANEL_END, "a balance")?;
            }
            fade_in_all(stage, &row, 0.5)?;
            flash_all(stage, &[t.forward[e][0], t.back[e][0]], 0.7)?;
            rows.push(row);
        }
        let mut balance_texts: Vec<Mob> = rows.iter().flatten().copied().collect();
        balance_texts.extend(&edges);
        balance_texts.extend([head_row[0], balance_head]);
        balance_texts.extend(&names);
        balance_texts.extend(&b_counts);
        balance_texts.extend(&b_info);
        keep_apart(stage, &balance_texts, 0.05, "the balances")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 14_14: multiply them: the counts cancel, copies of the chances fly
        // into one line, both products 9/160
        let line = say(&kit, stage, "14_14")?;
        line.wait_for(stage, AT_MULTIPLY.1)?;
        let flows: Vec<Mob> = rows.iter().map(|r| r[5]).collect();
        fade_out_all(stage, &flows, 0.4)?;
        let times: Vec<Mob> = [1, 2]
            .iter()
            .map(|&e| {
                kit.tex(
                    stage,
                    r"\times",
                    CELL_SIZE,
                    Palette::dim(),
                    &[],
                    v(TIMES_X, TABLE_YS[e]),
                )
            })
            .collect::<fmn::Result<_>>()?;
        let row_right = rows
            .iter()
            .map(|r| stage.get_bounding_box_point(r[4], RIGHT)[0])
            .fold(CELL_X, f64::max);
        let rule_line = stage.arena_mut().add(
            polyline([v(EDGE_END, RULE_Y), v(row_right + 0.1, RULE_Y)])
                .style(line_style(Palette::dim(), 2.0))
                .build(),
        );
        let mut multiply = times.clone();
        multiply.push(rule_line);
        fade_in_all(stage, &multiply, 0.5)?;
        line.wait_for(stage, AT_SAME_COUNTS.1)?;
        let left_counts: Vec<Mob> = rows.iter().map(|r| r[0]).collect();
        let right_counts: Vec<Mob> = rows.iter().map(|r| r[3]).collect();
        let pointer = frame_around(stage, &left_counts, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        let pointer = frame_around(stage, &right_counts, POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_CANCEL.1)?;
        // We cancel factors from the product of all three balances, not
        // from each balance separately. Remove the individual equalities
        // before their counts disappear; only the product equality survives.
        let mut row_equalities: Vec<Mob> = rows.iter().map(|r| r[2]).collect();
        row_equalities.push(balance_head);
        fade_out_all(stage, &row_equalities, 0.3)?;
        let mut count_pieces = left_counts.clone();
        count_pieces.extend(&right_counts);
        cancel(&kit, stage, &count_pieces)?;
        line.wait_for(stage, AT_LEFT.1)?;
        let dot = || r"\cdot".to_owned();
        let left_side = [
            n.forward[0].clone(),
            dot(),
            n.forward[1].clone(),
            dot(),
            n.forward[2].clone(),
        ];
        let right_side = [
            "=".to_owned(),
            n.back[0].clone(),
            dot(),
            n.back[1].clone(),
            dot(),
            n.back[2].clone(),
        ];
        let (good, ink, dim) = (palette::good(), Palette::ink(), Palette::dim());
        // the forward chances first; the back ones as the voice reaches them
        let left_row = gather(
            &kit,
            stage,
            &left_side,
            &[good, ink, good, ink, good],
            &[0, 2, 4],
            &[rows[0][1], rows[1][1], rows[2][1]],
            CELL_X,
            LINE_YS[0],
        )?;
        line.wait_for(stage, AT_AGAINST.1)?;
        let eq_at = stage.get_bounding_box_point(left_row[4], RIGHT)[0] + PIECE_GAP;
        // q₀ is lost at corner 0 (back along edge 2), q₁ at 1 (edge 0), q₂
        // at 2 (edge 1)
        let right_row = gather(
            &kit,
            stage,
            &right_side,
            &[ink, dim, ink, dim, ink, dim],
            &[1, 3, 5],
            &[rows[2][4], rows[0][4], rows[1][4]],
            eq_at,
            LINE_YS[0],
        )?;
        let mut product_row = left_row.clone();
        product_row.extend(&right_row);
        if let Some(&last) = product_row.last() {
            keep_between(stage, last, CELL_X, PANEL_END, "the product")?;
        }
        line.wait_for(stage, AT_BOTH.1)?;
        let both = panel_tex(&kit, stage, &n.product, palette::mix(), CELL_X, LINE_YS[1])?;
        fade_in_all(stage, &[both], 0.5)?;
        let pointer = frame_around(stage, &[both], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 14_15: the same for any game, in letters: the weights cancel, and
        // winning all the way round is as likely as losing all the way back
        let line = say(&kit, stage, "14_15")?;
        line.wait_for(stage, AT_ANY_GAME.1)?;
        let mut leftovers: Vec<Mob> = rows.iter().flat_map(|r| [r[1], r[4]]).collect();
        leftovers.extend(&product_row);
        leftovers.push(both);
        fade_out_all(stage, &leftovers, 0.5)?;
        line.wait_for(stage, AT_LETTERS.1)?;
        let legend = kit.text_left(stage, LETTERS, 18.0, Palette::dim(), v(CELL_X, HEAD_Y))?;
        keep_between(stage, legend, CELL_X - 0.01, PANEL_END, "the letters")?;
        let letters = symbol_rows(&kit, stage, CELL_X, &TABLE_YS)?;
        fade_in_all(stage, &[legend], 0.5)?;
        let all_letters: Vec<Mob> = letters.iter().flatten().copied().collect();
        fade_in_all(stage, &all_letters, 0.6)?;
        line.wait_for(stage, AT_NO_WEIGHT.1)?;
        ring_corners(stage, &t.discs, &[], 0.8)?;
        line.wait_for(stage, AT_CANCELS.1)?;
        let letter_equalities: Vec<Mob> = letters.iter().map(|r| r[2]).collect();
        fade_out_all(stage, &letter_equalities, 0.3)?;
        let weights: Vec<Mob> = letters.iter().flat_map(|r| [r[0], r[3]]).collect();
        cancel(&kit, stage, &weights)?;
        line.wait_for(stage, AT_AROUND.1)?;
        let (wins, win_words) = products_row(&kit, stage, &letters, true, CELL_X, [LINE_YS[0], LINE_YS[1]])?;
        let around: Vec<Mob> = (0..3)
            .map(|e| stream(stage, e, true, PATH_W, palette::player(), 0.9))
            .collect();
        draw_bands(stage, &around, 1.5)?;
        line.wait_for(stage, AT_OTHER_WAY.1)?;
        let (losses, loss_words) = products_row(&kit, stage, &letters, false, CELL_X, [LINE_YS[0], LINE_YS[1]])?;
        let back_round: Vec<Mob> = (0..3)
            .map(|e| stream(stage, e, false, PATH_W, Palette::dim(), 0.9))
            .collect();
        draw_bands(stage, &back_round, 1.5)?;
        let mut last: Vec<Mob> = letters.iter().flat_map(|r| [r[1], r[4]]).collect();
        last.extend(&wins);
        last.extend(&losses);
        last.extend(&times);
        last.extend([win_words, loss_words, legend, head_row[0]]);
        last.extend(&edges);
        last.extend(&names);
        last.extend(&b_counts);
        last.extend(&b_info);
        keep_apart(stage, &last, 0.05, "the letters' panel")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// The three balances in letters, a row per edge from `left` at `ys`: w_e
/// p_e = w_{e+1} q_{e+1}, weights teal, forward chances green, back ones
/// dim. Detached.
pub(super) fn symbol_rows(kit: &Kit, stage: &mut Stage<'_>, left: f64, ys: &[f64; 3]) -> fmn::Result<Vec<Vec<Mob>>> {
    let colors = [
        palette::player(),
        palette::good(),
        Palette::ink(),
        palette::player(),
        Palette::dim(),
    ];
    (0..3)
        .map(|e| {
            let j = (e + 1) % 3;
            let srcs = [
                format!("w_{e}"),
                format!("p_{e}"),
                "=".to_owned(),
                format!("w_{j}"),
                format!("q_{j}"),
            ];
            pieces(kit, stage, &srcs, &colors, CELL_SIZE, left, ys[e])
        })
        .collect()
}

/// One side of p₀ p₁ p₂ = q₀ q₁ q₂ gathered from the letter rows, from
/// `left` on the first of `ys`, with its words under it on the second: the
/// forward side ("win, win, win") from each row's p, or the back side
/// ("lose, lose, lose", after an "=") from the rows' q, in the corners'
/// order (q₀ is row 2's, back along edge 2) and just past the forward side.
/// Returns the side and its words.
pub(super) fn products_row(
    kit: &Kit,
    stage: &mut Stage<'_>,
    letters: &[Vec<Mob>],
    forward: bool,
    left: f64,
    ys: [f64; 2],
) -> fmn::Result<(Vec<Mob>, Mob)> {
    let (srcs, colors, moving, sources, words, color) = if forward {
        (
            ["p_0", "p_1", "p_2"].map(str::to_owned).to_vec(),
            vec![palette::good(); 3],
            vec![0, 1, 2],
            vec![letters[0][1], letters[1][1], letters[2][1]],
            WINS3,
            palette::good(),
        )
    } else {
        (
            ["=", "q_0", "q_1", "q_2"].map(str::to_owned).to_vec(),
            vec![Palette::ink(), Palette::dim(), Palette::dim(), Palette::dim()],
            vec![1, 2, 3],
            vec![letters[2][4], letters[0][4], letters[1][4]],
            LOSSES3,
            Palette::dim(),
        )
    };
    let start = if forward {
        left
    } else {
        past_forward(stage, letters, left)?
    };
    let side = gather(kit, stage, &srcs, &colors, &moving, &sources, start, ys[0])?;
    let from = usize::from(!forward);
    let (l, r) = (
        stage.get_bounding_box_point(side[from], LEFT)[0],
        stage.get_bounding_box_point(side[side.len() - 1], RIGHT)[0],
    );
    let under = kit.text(stage, words, 18.0, color, v((l + r) / 2.0, ys[1]))?;
    keep_between(stage, under, PANEL_X, PANEL_END, "the products' words")?;
    fade_in_all(stage, &[under], 0.4)?;
    Ok((side, under))
}

/// Where the back side starts: past the forward side set from `left` (three
/// letters as wide as the rows' p, with their gaps).
fn past_forward(stage: &Stage<'_>, letters: &[Vec<Mob>], left: f64) -> fmn::Result<f64> {
    let widths: f64 = letters.iter().map(|r| stage.get_width(r[1])).sum();
    if widths <= 0.0 {
        return Err(fail("the forward chances have no width".into()));
    }
    Ok(left + widths + 3.0 * PIECE_GAP)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{BADGE_AT, THINK_S};
    use super::*;

    #[test]
    fn the_numbers_are_the_facts() {
        let n = numbers().unwrap();
        assert_eq!(n.published_mix.heads, ["29.5", "62", "62"]);
        assert_eq!(n.published_mix.tails, ["70.5", "38", "38"]);
        assert_eq!(n.mix.heads, ["30", "62.5", "62.5"]);
        assert_eq!(n.mix.tails, ["70", "37.5", "37.5"]);
        assert_eq!(n.players, ["1,418", "26"]);
        assert_eq!(n.counts, [[490, 360, 568], [10, 4, 12]]);
        assert_eq!(n.pct[0], ["34.6 %", "25.4 %", "40.1 %"]);
        assert_eq!(n.pairs[0], ["147 - 135", "225 - 213", "355 - 343"]);
        assert_eq!(n.nets[0], ["= 12", "= 12", "= 12"]);
        assert_eq!(n.pairs[1], ["1 - 1", "3 - 3", "9 - 9"]);
        assert_eq!(n.nets[1], ["= 0", "= 0", "= 0"]);
        assert_eq!(n.edge0, [r"30\% \times 490 = 147", r"37.5\% \times 360 = 135"]);
        assert_eq!(
            n.gain,
            [
                r"3 \times 12 = 36 = 727 - 691",
                r"36 \text{ dollars} / 1{,}418 \approx 2.54 \text{ cents}",
            ]
        );
        assert_eq!(n.keeps, "360 + 12 - 12 = 360");
        assert_eq!(n.real_pairs, ["134 - 127", "233 - 229", "339 - 356"]);
        assert_eq!(n.real_nets, ["= 7", "= 4", "= -17"]);
        assert_eq!(n.real_changes, ["-17 - 7 = -24", "7 - 4 = 3", "4 - (-17) = 21"]);
        assert_eq!(n.real_after, ["466", "363", "589"]);
        assert_eq!(n.real_gain, r"7 + 4 - 17 = -6 \text{ dollars}");
        assert_eq!(
            n.thousand,
            [
                "1,000 real rounds, the same seeded illustration:",
                "average net on each edge: 11.9, 11.9, 11.9",
                "average gain: 35.8 dollars a round",
            ]
        );
        assert_eq!(
            n.balances[0],
            ["10", r"\times \tfrac{1}{10}", "=", "4", r"\times \tfrac{1}{4}", "= 1"]
        );
        assert_eq!(
            n.balances[2],
            ["12", r"\times \tfrac{3}{4}", "=", "10", r"\times \tfrac{9}{10}", "= 9"]
        );
        assert_eq!(n.forward, [r"\tfrac{1}{10}", r"\tfrac{3}{4}", r"\tfrac{3}{4}"]);
        assert_eq!(n.back, [r"\tfrac{9}{10}", r"\tfrac{1}{4}", r"\tfrac{1}{4}"]);
        assert_eq!(n.product, r"\tfrac{9}{160} = \tfrac{9}{160}");
    }

    /// Every settled crowd on the triangle carries one net flow on all three
    /// edges, and gains three times it a round, on every game of tenths
    /// (as the oracle's 729). Failed control: a crowd that has not settled
    /// (B's 10, 4, 12 under the mix) has edges that disagree, and each
    /// corner changes by exactly net in minus net out, so equal nets are
    /// the settled crowd's, not every crowd's.
    #[test]
    fn a_settled_crowd_carries_one_net_flow() {
        for p0 in 1..10 {
            for p1 in 1..10 {
                for p2 in 1..10 {
                    let g = Game {
                        up: [p0, p1, p2].map(|t| t * model::DEN / 10),
                        den: model::DEN,
                    };
                    let w = g.shares();
                    let p = [0, 1, 2].map(|i| g.chance(i));
                    let net = [0, 1, 2].map(|e| w[e] * p[e] - w[(e + 1) % 3] * (Ratio::ONE - p[(e + 1) % 3]));
                    assert!(net[0] == net[1] && net[1] == net[2], "{g:?}");
                    assert_eq!(g.mean(), Ratio::int(3) * net[0], "{g:?}");
                }
            }
        }
        let (b, mix) = (ROUND.game_b(), ROUND.mix());
        let start = b.shares().map(|w| Ratio::int(26) * w);
        let p = [0, 1, 2].map(|i| mix.chance(i));
        let net = [0, 1, 2].map(|e| start[e] * p[e] - start[(e + 1) % 3] * (Ratio::ONE - p[(e + 1) % 3]));
        assert_eq!(net, [Ratio::new(3, 2), Ratio::int(-2), Ratio::new(1, 2)]);
        let after = model::evolve(&start, &mix);
        for i in 0..3 {
            assert_eq!(after[i] - start[i], net[(i + 2) % 3] - net[i]);
        }
    }

    /// The real round is the real crowd's: its crossings move exactly the
    /// players `model::real_crowd` counts, round after round, from the
    /// same seed; each corner changes by net in minus net out; and over the
    /// thousand rounds the edges' totals differ only by the corners' total
    /// change, so their averages agree (within 0.1) and sit near 12 (within
    /// a player). Failed control: one real round's edges disagree, and that
    /// round loses.
    #[test]
    fn the_real_round_is_the_real_crowds() {
        let mix = ROUND.mix();
        let start = [490, 360, 568];
        let crossings = model::real_crossings(&mix, start, facts::REAL_ROUNDS, SEED);
        let counted = model::real_crowd(&mix, start, facts::REAL_ROUNDS, SEED);
        assert_eq!(crossings.len(), counted.len());
        let mut before = start;
        let mut totals = [0i64; 3];
        for (moved, &after) in crossings.iter().zip(&counted) {
            let [f, b] = *moved;
            for i in 0..3 {
                assert_eq!(f[i] + b[i], before[i]);
                assert_eq!(f[(i + 2) % 3] + b[(i + 1) % 3], after[i]);
            }
            let nets = edge_nets(&signed(f).unwrap(), &signed(b).unwrap());
            for i in 0..3 {
                assert_eq!(after[i] as i64 - before[i] as i64, nets[(i + 2) % 3] - nets[i]);
                totals[i] += nets[i];
            }
            before = after;
        }
        let last = counted[counted.len() - 1];
        for i in 0..3 {
            assert_eq!(last[i] as i64 - start[i] as i64, totals[(i + 2) % 3] - totals[i]);
        }
        let rounds = facts::REAL_ROUNDS as f64;
        for &t in &totals {
            let mean = t as f64 / rounds;
            assert!((mean - totals[0] as f64 / rounds).abs() < 0.1, "{totals:?}");
            assert!((mean - 12.0).abs() < 1.0, "{totals:?}");
        }
        let [f, b] = crossings[0];
        let first = edge_nets(&signed(f).unwrap(), &signed(b).unwrap());
        assert!(first[0] != first[1] || first[1] != first[2]);
        assert!(first.iter().sum::<i64>() < 0);
    }

    /// Multiplying B's three balances leaves its two products, the counts
    /// cancelled; with the weights instead of counts the same. Failed
    /// controls: the mix's crowd does not balance its edges (12 net), and
    /// its products differ (75/640 against 63/640); and comparing the
    /// forward product with 1/8, or averaging the forward chances, misjudges
    /// B (its forward product is under 1/8 and its chances average over a
    /// half, yet it is fair).
    #[test]
    fn multiplying_the_balances_leaves_the_products() {
        let b = ROUND.game_b();
        let w = b.shares();
        let p = [0, 1, 2].map(|i| b.chance(i));
        let q = p.map(|x| Ratio::ONE - x);
        let left = (0..3).fold(Ratio::ONE, |a, e| a * w[e] * p[e]);
        let right = (0..3).fold(Ratio::ONE, |a, e| a * w[(e + 1) % 3] * q[(e + 1) % 3]);
        assert_eq!(left, right);
        let weights = w[0] * w[1] * w[2];
        assert_eq!(left / weights, Ratio::new(9, 160));
        assert_eq!(right / weights, Ratio::new(9, 160));
        let mix = ROUND.mix();
        let mw = mix.shares();
        let mp = [0, 1, 2].map(|i| mix.chance(i));
        assert_ne!(mw[0] * mp[0], mw[1] * (Ratio::ONE - mp[1]));
        assert_eq!(mix.products(), (Ratio::new(75, 640), Ratio::new(63, 640)));
        let (fb, _) = b.products();
        assert!(fb < Ratio::new(1, 8));
        assert!((p[0] + p[1] + p[2]) / Ratio::int(3) > Ratio::new(1, 2));
        assert_eq!(b.mean(), Ratio::ZERO);
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        // within a line, the scene waits for its phrases in the order said
        for pair in CUES.windows(2) {
            let [(a, x), (b, y)] = [pair[0], pair[1]];
            if a == b {
                let text = script.text(a).unwrap();
                assert!(text.find(x) < text.find(y), "{a}: {x:?} after {y:?}");
            }
        }
    }

    /// The layout keeps its rooms: the corner rows above the table's head,
    /// its rows and the lines under them apart, the multiplication's rule
    /// between the last row and the product, everything above the frame's
    /// foot; an edge's name clear of the "×"; the panel left of the
    /// triangle; the countdown right of it, under the badge, inside the
    /// frame.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            assert!(LABEL_Y - 0.15 > ROW_YS[0] + 0.15);
            assert!(ROW_YS[2] - 0.15 > HEAD_Y + 0.15);
            assert!(HEAD_Y - 0.12 > TABLE_YS[0] + 0.25);
            assert!(TABLE_YS[0] - TABLE_YS[1] >= 0.5 && TABLE_YS[1] - TABLE_YS[2] >= 0.5);
            assert!(TABLE_YS[2] - 0.25 > RULE_Y);
            assert!(RULE_Y > LINE_YS[0] + 0.25);
            assert!(LINE_YS[0] - LINE_YS[1] >= 0.45 && LINE_YS[1] - LINE_YS[2] >= 0.45);
            assert!(LINE_YS[2] - 0.2 > -FRAME_Y_RADIUS + 0.3);
            assert!(EDGE_X + 0.5 < EDGE_END && EDGE_END < TIMES_X - 0.15);
            assert!(CELL_X + 1.9 < NET_X);
            assert!(PANEL_END < B_AT[0] - 2.5 - CORNER_R - 0.2);
            assert!(THINK_AT[1] + 0.62 < BADGE_AT[1] - 0.19 - 0.2);
            assert!(THINK_AT[0] + 0.8 < FRAME_X_RADIUS - 0.3);
        }
        // corner 2 and its name stand right of the panel
        let c2 = corner(B_AT, 2);
        assert!(c2[0] - CORNER_R - 0.3 > PANEL_END, "{c2:?}");
        // the countdown, its ring and its note, clear of corner 0's disc, of
        // corner 1's and of edge 0's forward label at a generous size
        let (c0, c1) = (corner(B_AT, 0), corner(B_AT, 1));
        assert!(THINK_AT[0] - 0.62 - (c0[0] + CORNER_R) > 1.0);
        assert!(THINK_AT[1] - 0.62 - 0.3 - 0.1 > c1[1] + CORNER_R);
        let label = chance_at(B_AT, 0, true, 0.6, 0.25);
        assert!(THINK_AT[0] - 0.8 > label[0] + 0.3, "{label:?}");
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 14 is said once, in order;
    /// the counting hold (14_5) and the prediction (14_7) get their silent
    /// time; the reveals stand to be read; the last picture stands.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = NetFlow { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("14_net_flow: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=15).map(|k| format!("14_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("14_6") - end_of("14_5") >= COUNT_S,
            "{} s to count",
            at("14_6") - end_of("14_5")
        );
        assert!(
            at("14_8") - end_of("14_7") >= THINK_S,
            "{} s to think",
            at("14_8") - end_of("14_7")
        );
        for (before, after) in [("14_3", "14_4"), ("14_9", "14_10"), ("14_13", "14_14")] {
            assert!(
                at(after) - end_of(before) >= BEAT,
                "{before}: {} s",
                at(after) - end_of(before)
            );
        }
        for (before, after) in [
            ("14_8", "14_9"),
            ("14_10", "14_11"),
            ("14_11", "14_12"),
            ("14_12", "14_13"),
            ("14_14", "14_15"),
        ] {
            assert!(
                at(after) - end_of(before) >= INSPECT_S,
                "{before}: {} s",
                at(after) - end_of(before)
            );
        }
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(
            end - end_of("14_15") >= INSPECT_S,
            "the last picture is not held: {end}"
        );
    }
}
