//! Chapter 15, two products decide. B's triangle and settled crowd as 14
//! left them (its violet and green chances, teal puddles, the outlined
//! round badge). What 14 showed, "net flow = 0 ⇒ p₀ p₁ p₂ = q₀ q₁ q₂", and
//! the claim the other way with a "?", on the condition "every chance
//! strictly between 0 and 100 %" (15_1). B's crowd rebuilt from nothing:
//! the puddles drain; ten players at corner 0, "10 × 1/10 = ? × 1/4 = 1",
//! the "?" becoming 4 as corner 1's puddle fills, then "4 × 3/4 = ? × 1/4 =
//! 3" and 12, each count carried down from the row before (15_2). The third
//! balance, "12 × 3/4 ? 10 × 9/10", has nothing left to choose; the "?"
//! becomes "=" and "= 9"; the counts are 10, 4, 12, B's settled crowd
//! (15_3). The same steps in letters: "w₁ = w₀ p₀ / q₁" and "w₂ = w₀ p₀ p₁ /
//! (q₁ q₂)" (15_4); put into the third balance, w₀ is struck on both sides
//! and "p₀ p₁ p₂ = q₀ q₁ q₂" is left (15_5). A balanced crowd stays put, only
//! the settled one does, so equal products give net flow 0 and a fair game;
//! the "?" goes (15_6). With the net flow kept in the balances, "w₁ q₁ = w₀ p₀
//! − net", "w₂ q₁ q₂ = w₀ p₀ p₁ − net (p₁ + q₁)" with "p₁ + q₁ = 1", and one
//! line, "net × (p₂ + q₁ q₂) = w₀ × (p₀ p₁ p₂ − q₀ q₁ q₂)" (15_7). Both
//! factors beside the products are positive, so the net flow has the sign
//! of the difference: bigger wins, smaller loses; the two claims become one
//! rule (15_8). B's 9/160 against 9/160: net flow 0, fair; "averaging B's
//! forward chances: 53.3 %, a winner?" struck (15_9). The mix's triangle:
//! "30 % · 62.5 % · 62.5 %" against "70 % · 37.5 % · 37.5 %", values
//! withheld (15_10, a prediction); "= 75/640" and "= 63/640", gold paths
//! forward (15_11). The one line on the mix's crowd of 1,418: "p₂ + q₁ q₂ =
//! 62.5 % + 37.5 % · 37.5 % = 49/64", "75/640 − 63/640 = 3/160", and "12 ×
//! 49/64 = 490 × 3/160 = 147/16" set over the line's own factors (15_12).
//! "3 × 12 / 1,418 = 18/709 dollars ≈ 2.54 cents per player per round" and
//! a "precisely" card, K06, held for its reading time (15_13). The
//! published coins (the filled badge, the mix's 29.5 and 62 %, the puddles
//! barely moving): a table, game A "0.1213 < 0.1288 loses −1.00", game B
//! "0.0527 < 0.0588 loses −0.87", the mix "0.1134 > 0.1018 wins +1.57",
//! "average: cents per player per round" (15_14). No crowd and no weights:
//! the puddles drain while the forward and back chances are pointed at
//! (15_15, held).
//!
//! Every count, chance, product and cent is computed from `crate::model`
//! and checked against `crate::facts`; the scene refuses to draw if the
//! words would stop being true of them. No typeset text is indicated,
//! scaled or recoloured: corners get hollow rings, words a frame, a "?" is
//! replaced by its answer, and a cancelled factor gets a red stroke.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    fade_in_all, fade_out_all, fail, flash_all, fly, keep_between, now, pause, read_s, say, screen, screen_int, think,
    together, wait_until,
};
use super::looks_hopeless::{POINT_PAD, badge_text, frame_around, full_badge, point_at};
use super::net_flow::{
    CELL_SIZE, CELL_X, CHANGE_S, EACH, EDGE_X, INSPECT_S, LETTERS, LINE_SIZE, MIX_NAME, PANEL_END, PATH_W, PIECE_GAP,
    THINK_AT, WORDS_SIZE, cancel, csv, draw_bands, edge_names, edge_nets, frac_tex, mix_labels, mix_title, panel_tex,
    panel_words, pieces, puddles, stream, strike, symbol_rows, unit_after, whole, whole_flows,
};
use super::one_dial::{BEAT, PERCENT};
use super::one_player_to_a_crowd::{PANEL_X, SPLIT_LABEL, ring_corners, round_badge, round_badge_text};
use super::three_corners::{Chances, Which, chance_labels, chances, keep_apart, swap, triangle};
use crate::facts;
use crate::model::{self, PUBLISHED, ROUND, Ratio};
use crate::palette;

pub struct TwoProducts {
    pub kit: Kit,
}

// the chapter's fixed words and letters (numbers are built from the model
// and facts)
const MINUS: &str = "\u{2212}";
const CONDITION: &str = "every chance strictly between 0 and 100 %";
const KNOWN: &str = r"\text{net flow} = 0 \quad\Rightarrow\quad p_0 p_1 p_2 = q_0 q_1 q_2";
const CONVERSE: &str = r"p_0 p_1 p_2 = q_0 q_1 q_2 \quad\Rightarrow\quad \text{net flow} = 0";
const ASKED: &str = "?";
const SIGN_RULE: &str = r"\text{the net flow has the sign of } p_0 p_1 p_2 - q_0 q_1 q_2";
const WEIGHT_1: &str = r"w_1 = w_0 p_0 / q_1";
const WEIGHT_2: &str = r"w_2 = w_1 p_1 / q_2 = w_0 p_0 p_1 / (q_1 q_2)";
const THIRD: [&str; 5] = ["w_0", r"p_0 p_1 p_2 / (q_1 q_2)", "=", "w_0", "q_0"];
const PRODUCTS: &str = r"p_0 p_1 p_2 = q_0 q_1 q_2";
const CHAIN: [&str; 3] = [
    "every edge balanced \u{2192} the crowd stays put",
    "only the settled crowd stays put: any corner reaches any other",
    "so equal products \u{2192} net flow 0 \u{2192} fair",
];
const NET_ROWS: [&str; 3] = [
    r"w_0 p_0 - w_1 q_1 = \text{net}",
    r"w_1 p_1 - w_2 q_2 = \text{net}",
    r"w_2 p_2 - w_0 q_0 = \text{net}",
];
const BACK_FLOW: &str = r"w_1 q_1 = w_0 p_0 - \text{net}";
const SECOND_RAW: &str = r"w_2 q_1 q_2 = w_0 p_0 p_1 - \text{net}\,(p_1 + q_1)";
const SECOND: &str = r"w_2 q_1 q_2 = w_0 p_0 p_1 - \text{net}";
const ADD_UP: &str = r"p_1 + q_1 = 1";
/// The one line, as pieces: net × (p₂ + q₁ q₂) = w₀ × (p₀ p₁ p₂ − q₀ q₁ q₂).
const ONE_LINE: [&str; 5] = [
    r"\text{net}",
    r"\times (p_2 + q_1 q_2)",
    "=",
    "w_0",
    r"\times (p_0 p_1 p_2 - q_0 q_1 q_2)",
];
const POSITIVE: &str = "positive";
const VERDICTS: [&str; 2] = [
    "forward product bigger \u{2192} net flow above 0 \u{2192} wins",
    "forward product smaller \u{2192} net flow below 0 \u{2192} loses",
];
const GAME_B: &str = "game B";
const B_FAIR: &str = r"\text{net flow} = 0 \quad \text{fair}";
const AVERAGING: &str = "averaging B's forward chances:";
const A_WINNER: &str = "a winner?";
const FORWARD_WORD: &str = "forward:";
const BACK_WORD: &str = "back:";
const MIX_CROWD: &str = "the mix's";
const PLAYERS_AT: &str = "players:";
const AT_CORNER_0: &str = "at corner 0, net flow";
const PRODUCTS_HEAD: &str = "products: forward vs back";
const AVERAGE_HEAD: &str = "average";
const CENTS_UNIT: &str = "average: cents per player per round";
const GAMES: [&str; 2] = ["game A", "game B"];
const WINS: &str = "wins";
const LOSES: &str = "loses";

/// Where each picture lands: (line, the phrase it waits for).
const AT_ZERO: (&str, &str) = ("15_1", "when the net flow is zero, the two products match");
const AT_OTHER: (&str, &str) = ("15_1", "The test also runs the other way");
const AT_CERTAIN: (&str, &str) = ("15_1", "no coin is certain to win or certain to lose");
const AT_NONE: (&str, &str) = ("15_1", "none of the coins in this video is");
const AT_TRY: (&str, &str) = ("15_2", "Try it on game B");
const AT_PRETEND: (&str, &str) = ("15_2", "pretend we never found its settled crowd");
const AT_TEN: (&str, &str) = ("15_2", "Put ten players at corner zero");
const AT_TENTH: (&str, &str) = ("15_2", "A tenth of them step forward");
const AT_NEEDS_ONE: (&str, &str) = ("15_2", "the first balance needs one player stepping back");
const AT_QUARTER: (&str, &str) = ("15_2", "where a quarter step back");
const AT_FOUR: (&str, &str) = ("15_2", "That takes four players");
const AT_THREE_OF: (&str, &str) = ("15_2", "three of those four step forward");
const AT_NEEDS_THREE: (&str, &str) = ("15_2", "the second balance needs three stepping back");
const AT_TAKES_TWELVE: (&str, &str) = ("15_2", "that takes twelve");
const AT_NOTHING: (&str, &str) = ("15_3", "nothing left to choose");
const AT_NINE_FWD: (&str, &str) = ("15_3", "Twelve times three quarters is nine");
const AT_NINE_BACK: (&str, &str) = ("15_3", "ten times nine tenths is nine");
const AT_HOLDS: (&str, &str) = ("15_3", "so it holds");
const AT_SETTLED_INTO: (&str, &str) = ("15_3", "exactly the crowd game B settled into");
const AT_WHY_RIGHT: (&str, &str) = ("15_4", "Why did the third balance come out right");
const AT_FOLLOW: (&str, &str) = ("15_4", "Follow the same steps for any game");
const AT_GIVES_ONE: (&str, &str) = ("15_4", "The first balance gives corner one's weight");
const AT_GIVES_TWO: (&str, &str) = ("15_4", "the second gives corner two's");
const AT_SAFE: (&str, &str) = ("15_4", "Dividing by the back chances is safe");
const AT_PUT_BOTH: (&str, &str) = ("15_5", "Put both into the third balance");
const AT_W0_CANCELS: (&str, &str) = ("15_5", "appears on both sides and cancels");
const AT_THROUGH: (&str, &str) = ("15_5", "Multiply through by the two back chances");
const AT_EXACTLY_WHEN: (&str, &str) = ("15_5", "holds exactly when the products match");
const AT_STAYS: (&str, &str) = ("15_6", "A crowd that balances every edge stays put");
const AT_GETS_BACK: (&str, &str) = ("15_6", "each corner gets back exactly the players it sends");
const AT_ONLY_ONE: (&str, &str) = ("15_6", "only one way of spreading a crowd stays put");
const AT_ANY_CORNER: (&str, &str) = ("15_6", "players can get from any corner to any other");
const AT_MATCHING: (&str, &str) = ("15_6", "So matching products mean the net flow is zero");
const AT_DONT: (&str, &str) = ("15_7", "When the products don't match");
const AT_KEEP: (&str, &str) = ("15_7", "Keep the net flow in each balance");
const AT_BACK_FLOW: (&str, &str) = ("15_7", "The first balance gives corner one's back flow");
const AT_SECOND: (&str, &str) = ("15_7", "Put it into the second");
const AT_ADD_UP: (&str, &str) = ("15_7", "each corner's two chances add up to one");
const AT_COLLECTS: (&str, &str) = ("15_7", "everything collects into one line");
const AT_SIDES: (&str, &str) = ("15_8", "Look at the two sides");
const AT_POSITIVE: (&str, &str) = ("15_8", "so it's positive");
const AT_W0_POSITIVE: (&str, &str) = ("15_8", "Corner zero's weight is positive too");
const AT_SAME_SIGN: (&str, &str) = ("15_8", "the net flow has the same sign");
const AT_BIGGER: (&str, &str) = ("15_8", "When the forward product is bigger");
const AT_SMALLER: (&str, &str) = ("15_8", "When it's smaller, the game loses");
const AT_FOR_B: (&str, &str) = ("15_9", "For game B with the round coins");
const AT_B_FAIR: (&str, &str) = ("15_9", "so B's net flow is zero and B is fair");
const AT_AVERAGING: (&str, &str) = ("15_9", "Averaging B's forward chances");
const AT_FOOLED: (&str, &str) = ("15_9", "The products aren't fooled");
const AT_FOR_MIX: (&str, &str) = ("15_10", "For the mix, the forward product");
const AT_BACKWARD: (&str, &str) = ("15_10", "The backward product is");
const AT_WHICH: (&str, &str) = ("15_10", "Which is bigger");
const AT_THE_FORWARD: (&str, &str) = ("15_11", "The forward product.");
const AT_SEVENTY_FIVE: (&str, &str) = ("15_11", "Seventy-five over six hundred forty");
const AT_SIXTY_THREE: (&str, &str) = ("15_11", "against sixty-three over six hundred forty");
const AT_TURNS: (&str, &str) = ("15_11", "the mix's crowd turns forward");
const AT_CHECKS: (&str, &str) = ("15_12", "the one line checks out");
const AT_POS_NUMBER: (&str, &str) = ("15_12", "Its positive number comes to forty-nine sixty-fourths");
const AT_DIFFER: (&str, &str) = ("15_12", "its products differ by three over one hundred sixty");
const AT_TWELVE_TIMES: (&str, &str) = ("15_12", "Twelve players of net flow times");
const AT_AGAINST_490: (&str, &str) = ("15_12", "against four hundred ninety players at corner zero");
const AT_BOTH_SIDES: (&str, &str) = ("15_12", "Both sides come to");
const AT_SPREAD: (&str, &str) = ("15_13", "Spread over the players and tripled");
const AT_EXACT: (&str, &str) = ("15_13", "gives the mix's average exactly");
const AT_CENTS: (&str, &str) = ("15_13", "just over two and a half cents a round");
const AT_SHAVE: (&str, &str) = ("15_14", "shave the half point off again");
const AT_PUBLISHED: (&str, &str) = ("15_14", "the coins Harmer and Abbott published");
const AT_A: (&str, &str) = ("15_14", "Game A's forward product falls below");
const AT_B: (&str, &str) = ("15_14", "Game B's falls below as well");
const AT_MIX: (&str, &str) = ("15_14", "The mix's forward product stays above");
const AT_DECIDE: (&str, &str) = ("15_14", "Two products per game decide");
const AT_NEVER: (&str, &str) = ("15_15", "what the test never needed");
const AT_NO_CROWD: (&str, &str) = ("15_15", "No crowd and no weights");
const AT_FORWARD3: (&str, &str) = ("15_15", "the three chances forward");
const AT_BACK3: (&str, &str) = ("15_15", "the three chances back");
const AT_MULTIPLIED: (&str, &str) = ("15_15", "each set multiplied together");
#[cfg(test)]
const CUES: [(&str, &str); 76] = [
    AT_ZERO,
    AT_OTHER,
    AT_CERTAIN,
    AT_NONE,
    AT_TRY,
    AT_PRETEND,
    AT_TEN,
    AT_TENTH,
    AT_NEEDS_ONE,
    AT_QUARTER,
    AT_FOUR,
    AT_THREE_OF,
    AT_NEEDS_THREE,
    AT_TAKES_TWELVE,
    AT_NOTHING,
    AT_NINE_FWD,
    AT_NINE_BACK,
    AT_HOLDS,
    AT_SETTLED_INTO,
    AT_WHY_RIGHT,
    AT_FOLLOW,
    AT_GIVES_ONE,
    AT_GIVES_TWO,
    AT_SAFE,
    AT_PUT_BOTH,
    AT_W0_CANCELS,
    AT_THROUGH,
    AT_EXACTLY_WHEN,
    AT_STAYS,
    AT_GETS_BACK,
    AT_ONLY_ONE,
    AT_ANY_CORNER,
    AT_MATCHING,
    AT_DONT,
    AT_KEEP,
    AT_BACK_FLOW,
    AT_SECOND,
    AT_ADD_UP,
    AT_COLLECTS,
    AT_SIDES,
    AT_POSITIVE,
    AT_W0_POSITIVE,
    AT_SAME_SIGN,
    AT_BIGGER,
    AT_SMALLER,
    AT_FOR_B,
    AT_B_FAIR,
    AT_AVERAGING,
    AT_FOOLED,
    AT_FOR_MIX,
    AT_BACKWARD,
    AT_WHICH,
    AT_THE_FORWARD,
    AT_SEVENTY_FIVE,
    AT_SIXTY_THREE,
    AT_TURNS,
    AT_CHECKS,
    AT_POS_NUMBER,
    AT_DIFFER,
    AT_TWELVE_TIMES,
    AT_AGAINST_490,
    AT_BOTH_SIDES,
    AT_SPREAD,
    AT_EXACT,
    AT_CENTS,
    AT_SHAVE,
    AT_PUBLISHED,
    AT_A,
    AT_B,
    AT_MIX,
    AT_DECIDE,
    AT_NEVER,
    AT_NO_CROWD,
    AT_FORWARD3,
    AT_BACK3,
    AT_MULTIPLIED,
];

/// The panel, top down: what 14 showed and the claim the other way; the
/// condition; the legend (or the split label, or the mix's crowd); a row per
/// edge (edge e, from corner e to e + 1); the steps under them; the words
/// under the one line; the verdicts. Everything is left of [`PANEL_END`].
const CLAIM_YS: [f64; 2] = [2.95, 2.45];
const COND_Y: f64 = 1.95;
const LEGEND_Y: f64 = 1.5;
const ROWS: [f64; 3] = [0.95, 0.4, -0.15];
const STEP_YS: [f64; 3] = [-0.75, -1.35, -1.95];
const UNDER_Y: f64 = -2.35;
const LOW_YS: [f64; 2] = [-2.8, -3.25];
/// 15_5's products line, under the steps; it rises to the first step.
const L4_Y: f64 = -2.6;
/// 15_6's three lines of words, under the risen products line.
const CHAIN_YS: [f64; 3] = [-1.35, -1.9, -2.45];
/// 15_14's table: the game's name at [`EDGE_X`], its products, its verdict,
/// its average.
const PROD_X: f64 = -5.75;
const VERDICT_X: f64 = -3.3;
const AVG_X: f64 = -2.3;
/// The gap before 15_7's side note.
const NOTE_GAP: f64 = 0.5;
/// The rule's size (its words make it the panel's widest line), and the
/// size of the one line's numbers (a little under its letters', so 490
/// stays inside the room around "w₀").
const RULE_SIZE: f64 = 24.0;
const NUMBERS_SIZE: f64 = 24.0;
/// Seconds: a count carried down to the next balance, a puddle growing, the
/// puddles draining.
const MOVE_S: f64 = 0.9;
const GROW_S: f64 = 0.6;
const DRAIN_S: f64 = 1.2;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    round_badge: String,
    published_badge: String,
    b: Chances,
    mix: Chances,
    published_mix: Chances,
    /// The puddles: B's crowd, the mix's, the published mix's.
    shares: [[f64; 3]; 3],
    /// 15_2, 15_3: B's crowd rebuilt from corner 0, a corner at a time (as
    /// puddles of the 26), and the three balances as pieces: count, ×
    /// chance, =, count, × chance, = flow.
    built: [f64; 3],
    b_rows: [[String; 6]; 3],
    /// 15_9: B's products, and the chances averaged.
    b_line: String,
    averaged: String,
    /// 15_10, 15_11: the mix's products in percent, then their values.
    mix_products: [String; 2],
    mix_values: [String; 2],
    /// 15_12: the crowd, the positive factor, the difference, the one line
    /// in numbers and what both sides come to.
    crowd_words: String,
    factor: String,
    difference: String,
    one_line: [String; 5],
    sides: String,
    /// 15_13: a player's average, and the exact card.
    per_player: String,
    k06: String,
    /// 15_14: game A, game B and the mix with the published coins: name,
    /// products, verdict, average in cents.
    table: [[String; 4]; 3],
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let (b, mix) = (ROUND.game_b(), ROUND.mix());
    let p = [0, 1, 2].map(|i| b.chance(i));
    let q = p.map(|x| Ratio::ONE - x);
    if p.iter().chain(&q).any(|&x| x <= Ratio::ZERO) {
        return Err("15_1 says no coin is certain to win or certain to lose".into());
    }
    // 15_2: B's crowd rebuilt from corner 0's ten, one balance at a time:
    // corner 1 sends back what corner 0 sends forward, then corner 2 what
    // corner 1 sends forward
    let n_b = model::smallest_whole_crowd(&b);
    let start = whole(Ratio::int(n_b) * b.shares()[0], "corner 0's players")?;
    let w0 = Ratio::int(start);
    let w1 = w0 * p[0] / q[1];
    let w2 = w1 * p[1] / q[2];
    let built = [
        start,
        whole(w1, "corner 1's players")?,
        whole(w2, "corner 2's players")?,
    ];
    if csv(built) != get("crowd.counts")? || built.iter().sum::<i64>() != n_b {
        return Err("15_2 builds ten, four and twelve, B's settled crowd".into());
    }
    // 15_3: the third balance has nothing left to choose, and holds
    let weights = [w0, w1, w2];
    let [f0, f1, f2] = [0, 1, 2].map(|i| whole(weights[i] * p[i], "a forward flow"));
    let forward = [f0?, f1?, f2?];
    let [b0, b1, b2] = [0, 1, 2].map(|i| whole(weights[i] * q[i], "a back flow"));
    let back = [b0?, b1?, b2?];
    if csv(forward) != get("crowd.forward")? || csv(back) != get("crowd.back")? {
        return Err("15_2 and 15_3 say one, three and nine each way".into());
    }
    if edge_nets(&forward, &back) != [0; 3] {
        return Err("15_3 says the third balance holds".into());
    }
    let b_rows = [0, 1, 2].map(|e| {
        let j = (e + 1) % 3;
        [
            built[e].to_string(),
            format!(r"\times {}", frac_tex(p[e])),
            "=".to_owned(),
            built[j].to_string(),
            format!(r"\times {}", frac_tex(q[j])),
            format!("= {}", forward[e]),
        ]
    });
    // 15_9: B's two products, and the averaged forward chance that misjudges it
    let (pf, pb) = b.products();
    if pf != pb || csv([pf, pb]) != get("round.products.b")? || b.mean() != Ratio::ZERO {
        return Err("15_9 says B's products are equal and B is fair".into());
    }
    let b_line = format!(
        r"{} \cdot {} \cdot {} = {} = {} \cdot {} \cdot {}",
        frac_tex(p[0]),
        frac_tex(p[1]),
        frac_tex(p[2]),
        frac_tex(pf),
        frac_tex(q[0]),
        frac_tex(q[1]),
        frac_tex(q[2])
    );
    let average = (p[0] + p[1] + p[2]) / Ratio::int(3);
    if average.pct(1) != get("round.b.thirds.win.pct")? || average <= Ratio::new(1, 2) {
        return Err("15_9 says averaging B's chances says fifty-three point three percent".into());
    }
    let averaged = format!("{AVERAGING} {} {PERCENT}, {A_WINNER}", average.pct(1));
    // 15_10, 15_11: the mix's two products, in percent and in 640ths
    let mc = chances(&mix)?;
    let coins = get("round.mix.coins.pct")?;
    let tails = get("round.mix.tails.pct")?;
    if [mc.heads[0].as_str(), mc.heads[1].as_str()].join(",") != coins
        || mc.heads[1] != mc.heads[2]
        || [mc.tails[0].as_str(), mc.tails[1].as_str()].join(",") != tails
        || mc.tails[1] != mc.tails[2]
    {
        return Err("15_10 says thirty and sixty-two and a half, seventy and thirty-seven and a half".into());
    }
    let mix_products = [
        format!(r"{}\% \cdot {}\% \cdot {}\%", mc.heads[0], mc.heads[1], mc.heads[2]),
        format!(r"{}\% \cdot {}\% \cdot {}\%", mc.tails[0], mc.tails[1], mc.tails[2]),
    ];
    let (fm, bm) = mix.products();
    let common = model::lcm(fm.den(), bm.den());
    let [nf, nb] = [fm, bm].map(|x| x.num() * (common / x.den()));
    if format!("{nf}/{common},{nb}/{common}") != get("round.products.mix.common")? || fm <= bm {
        return Err("15_11 says seventy-five over six hundred forty, the bigger".into());
    }
    let mix_values = [nf, nb].map(|x| format!(r"= \tfrac{{{x}}}{{{common}}}"));
    // 15_12: the one line on the mix's crowd of 1,418
    let n_mix = model::smallest_whole_crowd(&mix);
    let count0 = whole(Ratio::int(n_mix) * mix.shares()[0], "corner 0's players")?;
    let (mf, mb) = whole_flows(&mix, n_mix)?;
    let nets = edge_nets(&mf, &mb);
    let net = nets[0];
    if nets.iter().any(|&x| x != net)
        || n_mix.to_string() != get("mixcrowd.size")?
        || csv(nets) != get("mixcrowd.net")?
        || get("mixcrowd.counts")?.split(',').next() != Some(count0.to_string().as_str())
    {
        return Err("15_12 says twelve players of net flow and four hundred ninety at corner zero".into());
    }
    let mp = [0, 1, 2].map(|i| mix.chance(i));
    let mq = mp.map(|x| Ratio::ONE - x);
    let factor = mp[2] + mq[1] * mq[2];
    let diff = fm - bm;
    let left = Ratio::int(net) * factor;
    if left != Ratio::int(count0) * diff || factor <= Ratio::ZERO {
        return Err("15_12 says both sides come to the same".into());
    }
    if factor.to_string() != get("round.mix.sign.factor")?
        || diff.to_string() != get("round.products.mix.diff")?
        || left.to_string() != get("round.mix.sign.sides")?
    {
        return Err("15_12 says forty-nine sixty-fourths, three over one hundred sixty, and 147/16".into());
    }
    let crowd_words = format!(
        "{MIX_CROWD} {} {PLAYERS_AT} {} {AT_CORNER_0} {net}",
        screen_int(n_mix),
        screen_int(count0)
    );
    let factor_tex = format!(
        r"p_2 + q_1 q_2 = {}\% + {}\% \cdot {}\% = {}",
        mc.heads[2],
        mc.tails[1],
        mc.tails[2],
        frac_tex(factor)
    );
    let difference = format!(
        r"p_0 p_1 p_2 - q_0 q_1 q_2 = \tfrac{{{nf}}}{{{common}}} - \tfrac{{{nb}}}{{{common}}} = {}",
        frac_tex(diff)
    );
    let one_line = [
        net.to_string(),
        format!(r"\times {}", frac_tex(factor)),
        "=".to_owned(),
        count0.to_string(),
        format!(r"\times {}", frac_tex(diff)),
    ];
    // 15_13: spread over the players and tripled, the mix's average exactly
    let three = model::MODULUS;
    let mean = Ratio::int(three) * Ratio::int(net) / Ratio::int(n_mix);
    let cents = mean.cents(2);
    if mean != mix.mean()
        || mean != Ratio::int(three) * mix.shares()[0] * diff / factor
        || mean.to_string() != get("round.mix.mean")?
        || cents != get("round.mix.mean.cents")?
    {
        return Err("15_13 says the net flow gives the mix's average exactly".into());
    }
    let per_player = format!(
        r"\tfrac{{{three} \times {net}}}{{{}}} = {} \text{{ dollars}} \approx {} \text{{ cents}}",
        screen_int(n_mix).replace(',', "{,}"),
        frac_tex(mean),
        cents.trim_start_matches('+')
    );
    let k06 = format!(
        "if every chance is strictly between 0 and 1, the average gain per player per round is {three} w0 (p0 p1 p2 \
         {MINUS} q0 q1 q2) / (p2 + q1 q2), {three} times the net flow per player, where w0 is corner 0's share \
         of the settled crowd"
    );
    // 15_14: the published coins decide A, B and the mix by their products
    let mut table: [[String; 4]; 3] = Default::default();
    let games = [
        (GAMES[0], PUBLISHED.game_a(), "a"),
        (GAMES[1], PUBLISHED.game_b(), "b"),
        (MIX_NAME, PUBLISHED.mix(), "mix"),
    ];
    for (row, (name, g, key)) in table.iter_mut().zip(games) {
        let (fw, bw) = g.products();
        let fixed = [fw.fixed(4), bw.fixed(4)];
        if csv(&fixed) != get(&format!("pub.products.{key}.fixed"))?
            || csv([fw, bw]) != get(&format!("pub.products.{key}"))?
        {
            return Err(format!("15_14 shows {name}'s products as the facts print them"));
        }
        let mean = g.mean();
        if mean == Ratio::ZERO || mean.signum() != (fw - bw).signum() {
            return Err(format!("15_14 says {name}'s products decide it"));
        }
        let cents = mean.cents(2);
        if cents != get(&format!("pub.{key}.mean.cents"))? {
            return Err(format!("15_14 says {name}'s average"));
        }
        let (rel, verdict) = if fw > bw { (">", WINS) } else { ("<", LOSES) };
        *row = [
            name.to_owned(),
            format!("{} {rel} {}", fixed[0], fixed[1]),
            verdict.to_owned(),
            screen(&cents),
        ];
    }
    if table[0][2] != LOSES || table[1][2] != LOSES || table[2][2] != WINS {
        return Err("15_14 says A and B lose and the mix wins".into());
    }
    let pmc = chances(&PUBLISHED.mix())?;
    if [pmc.heads[0].as_str(), pmc.heads[1].as_str()].join(",") != get("pub.mix.coins.pct")? {
        return Err("15_14's triangle carries the published mix's chances".into());
    }
    let share = |g: &model::Game| g.shares().map(Ratio::to_f64);
    Ok(Numbers {
        round_badge: round_badge_text(&f)?,
        published_badge: badge_text(&f)?,
        b: chances(&b)?,
        mix: mc,
        published_mix: pmc,
        shares: [share(&b), share(&mix), share(&PUBLISHED.mix())],
        built: built.map(|c| c as f64 / n_b as f64),
        b_rows,
        b_line,
        averaged,
        mix_products,
        mix_values,
        crowd_words,
        factor: factor_tex,
        difference,
        one_line,
        sides: format!("= {}", frac_tex(left)),
        per_player,
        k06,
        table,
    })
}

// --------------------------------------------------------------- pictures

/// `piece` comes up over `source` and moves to its own place: the same
/// players carried from one balance to the next.
fn arrive_from(stage: &mut Stage<'_>, piece: Mob, source: Mob) -> fmn::Result<()> {
    let home = stage.get_center(piece);
    let start = stage.get_center(source);
    stage.move_to(piece, start, ORIGIN);
    fade_in_all(stage, &[piece], 0.3)?;
    together(stage, vec![fly(piece, home, MOVE_S)?])
}

/// A "?" where `piece` will stand, until its answer replaces it. Detached.
fn asked_at(kit: &Kit, stage: &mut Stage<'_>, piece: Mob) -> fmn::Result<Mob> {
    let at = stage.get_center(piece);
    kit.tex(stage, ASKED, CELL_SIZE, Palette::hero(), &[], at)
}

/// TeX just right of `m`, on its line: a value after a product. Detached.
fn tex_after(kit: &Kit, stage: &mut Stage<'_>, m: Mob, src: &str, color: Srgb) -> fmn::Result<Mob> {
    let right = stage.get_bounding_box_point(m, RIGHT)[0];
    let y = stage.get_center(m)[1];
    let t = kit.tex_left(stage, src, LINE_SIZE, color, &[], v(right + PIECE_GAP, y))?;
    keep_between(stage, t, right, PANEL_END, "a value after a product")?;
    Ok(t)
}

/// Each of `srcs` typeset centred over the matching piece of `under`, on
/// `y`. Detached.
fn centred_over(
    kit: &Kit,
    stage: &mut Stage<'_>,
    srcs: &[String],
    under: &[Mob],
    color: Srgb,
    y: f64,
) -> fmn::Result<Vec<Mob>> {
    if srcs.len() != under.len() {
        return Err(fail("every number needs its letter".into()));
    }
    let mut out = Vec::with_capacity(srcs.len());
    for (src, &u) in srcs.iter().zip(under) {
        let x = stage.get_center(u)[0];
        out.push(kit.tex(stage, src, NUMBERS_SIZE, color, &[], v(x, y))?);
    }
    Ok(out)
}

/// `words` (18 px) centred under `m` at `y`. Detached.
fn word_under(kit: &Kit, stage: &mut Stage<'_>, m: Mob, words: &str, color: Srgb, y: f64) -> fmn::Result<Mob> {
    let x = stage.get_center(m)[0];
    let w = kit.text(stage, words, 18.0, color, v(x, y))?;
    keep_between(stage, w, PANEL_X - 0.01, PANEL_END, "a word under the line")?;
    Ok(w)
}

/// Paths all the way round the triangle, forward or back. Detached.
fn round_paths(stage: &mut Stage<'_>, forward: bool, color: Srgb) -> Vec<Mob> {
    (0..3).map(|e| stream(stage, e, forward, PATH_W, color, 0.9)).collect()
}

impl SceneConstruct for TwoProducts {
    fn name(&self) -> &str {
        "15_two_products"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let (ink, dim, hero) = (Palette::ink(), Palette::dim(), Palette::hero());

        // 15_1: B's triangle and settled crowd as 14 left them; what 14
        // showed, the claim the other way, and its condition
        let line = say(&kit, stage, "15_1")?;
        let tag = round_badge(&kit, stage, &n.round_badge)?;
        let t = triangle(&kit, stage, Which::B)?;
        let mut labels = chance_labels(&kit, stage, Which::B, &n.b)?.to_vec();
        let mut title = t.title;
        let mut arriving = vec![tag[0], tag[1], title];
        arriving.extend(t.discs);
        arriving.extend(t.names);
        arriving.extend(t.forward.iter().flatten());
        arriving.extend(t.back.iter().flatten());
        arriving.extend(&labels);
        fade_in_all(stage, &arriving, 0.8)?;
        let crowd = puddles(stage, [0.0; 3])?;
        together(stage, crowd.to(n.shares[0], GROW_S)?)?;
        let mut texts = labels.clone();
        texts.extend(t.names);
        texts.push(title);
        keep_apart(stage, &texts, 0.08, "game B's triangle")?;
        let shafts: Vec<Mob> = t.forward.iter().chain(&t.back).map(|a| a[0]).collect();
        line.wait_for(stage, AT_ZERO.1)?;
        let known = panel_tex(&kit, stage, KNOWN, ink, PANEL_X, CLAIM_YS[0])?;
        fade_in_all(stage, &[known], 0.5)?;
        line.wait_for(stage, AT_OTHER.1)?;
        let converse = pieces(
            &kit,
            stage,
            &[CONVERSE.to_owned(), ASKED.to_owned()],
            &[ink, hero],
            LINE_SIZE,
            PANEL_X,
            CLAIM_YS[1],
        )?;
        keep_between(stage, converse[1], PANEL_X, PANEL_END, "the claim the other way")?;
        fade_in_all(stage, &converse, 0.5)?;
        line.wait_for(stage, AT_CERTAIN.1)?;
        let condition = panel_words(&kit, stage, CONDITION, dim, COND_Y)?;
        fade_in_all(stage, &[condition], 0.5)?;
        line.wait_for(stage, AT_NONE.1)?;
        let pointer = frame_around(stage, &labels, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;

        // 15_2: B's crowd rebuilt from ten players at corner 0, a balance at
        // a time, each "?" answered as its corner's puddle fills
        let line = say(&kit, stage, "15_2")?;
        line.wait_for(stage, AT_TRY.1)?;
        let pointer = frame_around(stage, &converse, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_PRETEND.1)?;
        together(stage, crowd.to([0.0; 3], DRAIN_S)?)?;
        let colors = [palette::player(), palette::good(), ink, palette::player(), dim, dim];
        let rows: Vec<Vec<Mob>> = (0..3)
            .map(|e| pieces(&kit, stage, &n.b_rows[e], &colors, CELL_SIZE, CELL_X, ROWS[e]))
            .collect::<fmn::Result<_>>()?;
        for row in &rows {
            if let Some(&last) = row.last() {
                keep_between(stage, last, CELL_X, PANEL_END, "a balance")?;
            }
        }
        let asked = [
            asked_at(&kit, stage, rows[0][3])?,
            asked_at(&kit, stage, rows[1][3])?,
            asked_at(&kit, stage, rows[2][2])?,
        ];
        let edges = edge_names(&kit, stage, &ROWS)?;
        line.wait_for(stage, AT_TEN.1)?;
        let split = panel_words(&kit, stage, SPLIT_LABEL, dim, LEGEND_Y)?;
        fade_in_all(stage, &[edges[0], split, rows[0][0]], 0.5)?;
        stage.play(tween(crowd.share[0], n.built[0], GROW_S)?)?;
        line.wait_for(stage, AT_TENTH.1)?;
        fade_in_all(stage, &[rows[0][1]], 0.4)?;
        flash_all(stage, &[t.forward[0][0]], 0.7)?;
        line.wait_for(stage, AT_NEEDS_ONE.1)?;
        fade_in_all(stage, &[rows[0][2], asked[0], rows[0][5]], 0.4)?;
        flash_all(stage, &[t.back[0][0]], 0.7)?;
        line.wait_for(stage, AT_QUARTER.1)?;
        fade_in_all(stage, &[rows[0][4]], 0.4)?;
        line.wait_for(stage, AT_FOUR.1)?;
        let anims = swap(stage, &[asked[0]], &[rows[0][3]], 0.5)?;
        stage.play_prepared(anims)?;
        stage.play(tween(crowd.share[1], n.built[1], GROW_S)?)?;
        line.wait_for(stage, AT_THREE_OF.1)?;
        fade_in_all(stage, &[edges[1]], 0.3)?;
        arrive_from(stage, rows[1][0], rows[0][3])?;
        fade_in_all(stage, &[rows[1][1]], 0.4)?;
        flash_all(stage, &[t.forward[1][0]], 0.7)?;
        line.wait_for(stage, AT_NEEDS_THREE.1)?;
        fade_in_all(stage, &[rows[1][2], asked[1], rows[1][4], rows[1][5]], 0.4)?;
        flash_all(stage, &[t.back[1][0]], 0.7)?;
        line.wait_for(stage, AT_TAKES_TWELVE.1)?;
        let anims = swap(stage, &[asked[1]], &[rows[1][3]], 0.5)?;
        stage.play_prepared(anims)?;
        stage.play(tween(crowd.share[2], n.built[2], GROW_S)?)?;

        // 15_3: the third balance has nothing left to choose, and holds; the
        // counts are B's settled crowd
        let line = say(&kit, stage, "15_3")?;
        line.wait_for(stage, AT_NOTHING.1)?;
        fade_in_all(stage, &[edges[2]], 0.3)?;
        arrive_from(stage, rows[2][0], rows[1][3])?;
        arrive_from(stage, rows[2][3], rows[0][0])?;
        fade_in_all(stage, &[rows[2][1], asked[2], rows[2][4]], 0.4)?;
        line.wait_for(stage, AT_NINE_FWD.1)?;
        flash_all(stage, &[t.forward[2][0]], 0.7)?;
        line.wait_for(stage, AT_NINE_BACK.1)?;
        flash_all(stage, &[t.back[2][0]], 0.7)?;
        line.wait_for(stage, AT_HOLDS.1)?;
        let anims = swap(stage, &[asked[2]], &[rows[2][2]], 0.4)?;
        stage.play_prepared(anims)?;
        fade_in_all(stage, &[rows[2][5]], 0.4)?;
        line.wait_for(stage, AT_SETTLED_INTO.1)?;
        let counts: Vec<Mob> = rows.iter().map(|r| r[0]).collect();
        let pointer = frame_around(stage, &counts, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        ring_corners(stage, &t.discs, &[], 0.8)?;
        let mut panel: Vec<Mob> = rows.iter().flatten().copied().collect();
        panel.extend(&edges);
        panel.extend([known, converse[0], converse[1], condition, split]);
        keep_apart(stage, &panel, 0.05, "B's balances")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 15_4: the same steps for any game, in letters: corner 1's weight
        // from corner 0's, corner 2's from corner 1's
        let line = say(&kit, stage, "15_4")?;
        line.wait_for(stage, AT_WHY_RIGHT.1)?;
        let pointer = frame_around(stage, &rows[2], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_FOLLOW.1)?;
        let letters = symbol_rows(&kit, stage, CELL_X, &ROWS)?;
        let legend = panel_words(&kit, stage, LETTERS, dim, LEGEND_Y)?;
        let mut old: Vec<Mob> = rows.iter().flatten().copied().collect();
        old.push(split);
        let mut new: Vec<Mob> = letters.iter().flatten().copied().collect();
        new.push(legend);
        let anims = swap(stage, &old, &new, CHANGE_S)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_GIVES_ONE.1)?;
        let weight1 = panel_tex(&kit, stage, WEIGHT_1, ink, PANEL_X, STEP_YS[0])?;
        fade_in_all(stage, &[weight1], 0.5)?;
        let pointer = frame_around(stage, &letters[0], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_GIVES_TWO.1)?;
        let weight2 = panel_tex(&kit, stage, WEIGHT_2, ink, PANEL_X, STEP_YS[1])?;
        fade_in_all(stage, &[weight2], 0.5)?;
        let pointer = frame_around(stage, &letters[1], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_SAFE.1)?;
        let pointer = frame_around(stage, &[condition], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;

        // 15_5: into the third balance: w₀ cancels, the two products are
        // left
        let line = say(&kit, stage, "15_5")?;
        line.wait_for(stage, AT_PUT_BOTH.1)?;
        let pointer = frame_around(stage, &letters[2], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        let third: Vec<String> = THIRD.iter().map(|s| (*s).to_owned()).collect();
        let third_colors = [palette::player(), palette::good(), ink, palette::player(), dim];
        let third = pieces(&kit, stage, &third, &third_colors, LINE_SIZE, PANEL_X, STEP_YS[2])?;
        if let Some(&last) = third.last() {
            keep_between(stage, last, PANEL_X, PANEL_END, "the third balance")?;
        }
        fade_in_all(stage, &third, 0.5)?;
        line.wait_for(stage, AT_W0_CANCELS.1)?;
        cancel(&kit, stage, &[third[0], third[3]])?;
        line.wait_for(stage, AT_THROUGH.1)?;
        let products = panel_tex(&kit, stage, PRODUCTS, palette::mix(), PANEL_X, L4_Y)?;
        fade_in_all(stage, &[products], 0.5)?;
        line.wait_for(stage, AT_EXACTLY_WHEN.1)?;
        let pointer = frame_around(stage, &[products], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut steps = vec![known, converse[0], converse[1], condition, legend, weight1, weight2];
        steps.extend([third[1], third[2], third[4], products]);
        steps.extend(letters.iter().flatten());
        steps.extend(&edges);
        keep_apart(stage, &steps, 0.05, "the letters' steps")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 15_6: a balanced crowd stays put, only the settled one does: equal
        // products give zero net flow, a fair game
        let line = say(&kit, stage, "15_6")?;
        line.wait_for(stage, AT_STAYS.1)?;
        fade_out_all(stage, &[weight1, weight2, third[1], third[2], third[4]], 0.4)?;
        let c = stage.get_center(products);
        together(stage, vec![fly(products, v(c[0], STEP_YS[0]), MOVE_S)?])?;
        let chain: Vec<Mob> = CHAIN
            .iter()
            .zip(CHAIN_YS)
            .map(|(words, y)| {
                let m = kit.text_left(stage, words, 18.0, ink, v(PANEL_X, y))?;
                keep_between(stage, m, PANEL_X - 0.01, PANEL_END, "a step of the argument")?;
                Ok(m)
            })
            .collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &chain[..1], 0.5)?;
        line.wait_for(stage, AT_GETS_BACK.1)?;
        ring_corners(stage, &t.discs, &[], 0.8)?;
        line.wait_for(stage, AT_ONLY_ONE.1)?;
        fade_in_all(stage, &chain[1..2], 0.5)?;
        line.wait_for(stage, AT_ANY_CORNER.1)?;
        flash_all(stage, &shafts, 0.9)?;
        line.wait_for(stage, AT_MATCHING.1)?;
        fade_in_all(stage, &chain[2..], 0.5)?;
        let answered = panel_tex(&kit, stage, CONVERSE, palette::mix(), PANEL_X, CLAIM_YS[1])?;
        let anims = swap(stage, &converse, &[answered], 0.6)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[answered], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut argued = vec![known, answered, condition, legend, products];
        argued.extend(letters.iter().flatten());
        argued.extend(&edges);
        argued.extend(&chain);
        keep_apart(stage, &argued, 0.05, "the argument")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 15_7: with the net flow kept in each balance, the same
        // substitutions collect into one line
        let line = say(&kit, stage, "15_7")?;
        line.wait_for(stage, AT_DONT.1)?;
        let mut leaving = chain.clone();
        leaving.push(products);
        fade_out_all(stage, &leaving, 0.4)?;
        line.wait_for(stage, AT_KEEP.1)?;
        let net_rows: Vec<Mob> = NET_ROWS
            .iter()
            .zip(ROWS)
            .map(|(src, y)| panel_tex(&kit, stage, src, ink, CELL_X, y))
            .collect::<fmn::Result<_>>()?;
        let all_letters: Vec<Mob> = letters.iter().flatten().copied().collect();
        let anims = swap(stage, &all_letters, &net_rows, CHANGE_S)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_BACK_FLOW.1)?;
        let back_flow = panel_tex(&kit, stage, BACK_FLOW, ink, PANEL_X, STEP_YS[0])?;
        fade_in_all(stage, &[back_flow], 0.5)?;
        let pointer = frame_around(stage, &net_rows[..1], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_SECOND.1)?;
        let second_raw = panel_tex(&kit, stage, SECOND_RAW, ink, PANEL_X, STEP_YS[1])?;
        fade_in_all(stage, &[second_raw], 0.5)?;
        let pointer = frame_around(stage, &net_rows[1..2], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_ADD_UP.1)?;
        let second = panel_tex(&kit, stage, SECOND, ink, PANEL_X, STEP_YS[1])?;
        let note_x = stage.get_bounding_box_point(second, RIGHT)[0] + NOTE_GAP;
        let note = panel_tex(&kit, stage, ADD_UP, dim, note_x, STEP_YS[1])?;
        let anims = swap(stage, &[second_raw], &[second, note], 0.7)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_COLLECTS.1)?;
        let one_colors = [palette::mix(), ink, ink, palette::player(), ink];
        let srcs: Vec<String> = ONE_LINE.iter().map(|s| (*s).to_owned()).collect();
        let one = pieces(&kit, stage, &srcs, &one_colors, LINE_SIZE, PANEL_X, STEP_YS[2])?;
        if let Some(&last) = one.last() {
            keep_between(stage, last, PANEL_X, PANEL_END, "the one line")?;
        }
        fade_in_all(stage, &one, 0.6)?;
        let pointer = frame_around(stage, &one, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut derived = vec![known, answered, condition, legend, back_flow, second, note];
        derived.extend(&net_rows);
        derived.extend(&edges);
        derived.extend(&one);
        keep_apart(stage, &derived, 0.05, "the derivation")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 15_8: both factors beside the products are positive, so the net
        // flow has the sign of the difference; the claims become one rule
        let line = say(&kit, stage, "15_8")?;
        line.wait_for(stage, AT_SIDES.1)?;
        let pointer = frame_around(stage, &one[..2], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.6)?;
        let pointer = frame_around(stage, &one[3..], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.6)?;
        line.wait_for(stage, AT_POSITIVE.1)?;
        let positive_factor = word_under(&kit, stage, one[1], POSITIVE, palette::good(), UNDER_Y)?;
        fade_in_all(stage, &[positive_factor], 0.4)?;
        let pointer = frame_around(stage, &one[1..2], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_W0_POSITIVE.1)?;
        let positive_weight = word_under(&kit, stage, one[3], POSITIVE, palette::good(), UNDER_Y)?;
        fade_in_all(stage, &[positive_weight], 0.4)?;
        ring_corners(stage, &t.discs[..1], &[], 0.7)?;
        line.wait_for(stage, AT_SAME_SIGN.1)?;
        let pointer = frame_around(stage, &one[..1], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.6)?;
        let pointer = frame_around(stage, &one[4..], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.6)?;
        line.wait_for(stage, AT_BIGGER.1)?;
        let bigger = panel_words(&kit, stage, VERDICTS[0], palette::good(), LOW_YS[0])?;
        fade_in_all(stage, &[bigger], 0.4)?;
        let wins_round = round_paths(stage, true, palette::good());
        draw_bands(stage, &wins_round, 1.2)?;
        line.wait_for(stage, AT_SMALLER.1)?;
        let smaller = panel_words(&kit, stage, VERDICTS[1], palette::bad(), LOW_YS[1])?;
        fade_in_all(stage, &[smaller], 0.4)?;
        let loses_round = round_paths(stage, false, palette::bad());
        draw_bands(stage, &loses_round, 1.2)?;
        let rule = kit.tex_left(
            stage,
            SIGN_RULE,
            RULE_SIZE,
            palette::mix(),
            &[],
            v(PANEL_X, CLAIM_YS[0]),
        )?;
        keep_between(stage, rule, PANEL_X - 0.01, PANEL_END, "the rule")?;
        let anims = swap(stage, &[known, answered], &[rule], 0.7)?;
        stage.play_prepared(anims)?;
        let pointer = frame_around(stage, &[rule], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut signed = vec![rule, condition, legend, back_flow, second, note];
        signed.extend([positive_factor, positive_weight, bigger, smaller]);
        signed.extend(&net_rows);
        signed.extend(&edges);
        signed.extend(&one);
        keep_apart(stage, &signed, 0.05, "the sign")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 15_9: B's equal products: net flow 0, fair; the averaged chance
        // that would have called it a winner is struck
        let line = say(&kit, stage, "15_9")?;
        line.wait_for(stage, AT_FOR_B.1)?;
        let mut leaving = vec![
            legend,
            back_flow,
            second,
            note,
            positive_factor,
            positive_weight,
            bigger,
            smaller,
        ];
        leaving.extend(&net_rows);
        leaving.extend(&edges);
        leaving.extend(&wins_round);
        leaving.extend(&loses_round);
        fade_out_all(stage, &leaving, 0.5)?;
        let b_name = kit.text_left(stage, GAME_B, WORDS_SIZE, palette::game_b(), v(EDGE_X, ROWS[0]))?;
        keep_between(stage, b_name, EDGE_X - 0.01, CELL_X - 0.1, "B's name")?;
        let b_line = panel_tex(&kit, stage, &n.b_line, ink, CELL_X, ROWS[0])?;
        fade_in_all(stage, &[b_name, b_line], 0.5)?;
        line.wait_for(stage, AT_B_FAIR.1)?;
        let b_fair = panel_tex(&kit, stage, B_FAIR, palette::mix(), CELL_X, ROWS[1])?;
        fade_in_all(stage, &[b_fair], 0.5)?;
        line.wait_for(stage, AT_AVERAGING.1)?;
        let averaged = panel_words(&kit, stage, &n.averaged, dim, ROWS[2])?;
        fade_in_all(stage, &[averaged], 0.5)?;
        line.wait_for(stage, AT_FOOLED.1)?;
        let struck = strike(&kit, stage, averaged);
        draw_bands(stage, &[struck], 0.5)?;
        let mut b_panel = vec![rule, condition, b_name, b_line, b_fair, averaged];
        b_panel.extend(&one);
        keep_apart(stage, &b_panel, 0.05, "B's products")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 15_10: the mix's triangle; its two products in percent, values
        // withheld (a prediction)
        let line = say(&kit, stage, "15_10")?;
        line.wait_for(stage, AT_FOR_MIX.1)?;
        fade_out_all(stage, &[b_name, b_line, b_fair, averaged, struck], 0.4)?;
        let mix_name = mix_title(&kit, stage)?;
        let new_labels = mix_labels(&kit, stage, &n.mix)?;
        let mut old = vec![title];
        old.extend(&labels);
        let mut new = vec![mix_name];
        new.extend(&new_labels);
        let mut anims = swap(stage, &old, &new, CHANGE_S)?;
        for m in crowd.to(n.shares[1], CHANGE_S)? {
            anims.push(stage.prepare(m)?);
        }
        stage.play_prepared(anims)?;
        title = mix_name;
        labels = new_labels;
        let mut texts = labels.clone();
        texts.extend(t.names);
        texts.push(title);
        keep_apart(stage, &texts, 0.08, "the mix's triangle")?;
        let words = [
            kit.text_left(stage, FORWARD_WORD, 18.0, dim, v(EDGE_X, ROWS[0]))?,
            kit.text_left(stage, BACK_WORD, 18.0, dim, v(EDGE_X, ROWS[1]))?,
        ];
        for &w in &words {
            keep_between(stage, w, EDGE_X - 0.01, CELL_X - 0.1, "a product's word")?;
        }
        let mix_products = [
            panel_tex(&kit, stage, &n.mix_products[0], palette::good(), CELL_X, ROWS[0])?,
            panel_tex(&kit, stage, &n.mix_products[1], dim, CELL_X, ROWS[1])?,
        ];
        fade_in_all(stage, &[words[0], mix_products[0]], 0.5)?;
        let pointer = frame_around(stage, &labels[..3], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_BACKWARD.1)?;
        fade_in_all(stage, &[words[1], mix_products[1]], 0.5)?;
        let pointer = frame_around(stage, &labels[3..], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_WHICH.1)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 15_11: the forward product, 75/640 against 63/640; the mix's crowd
        // turns forward
        let line = say(&kit, stage, "15_11")?;
        line.wait_for(stage, AT_THE_FORWARD.1)?;
        fade_out_all(stage, &[q], 0.3)?;
        let pointer = frame_around(stage, &mix_products[..1], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.6)?;
        line.wait_for(stage, AT_SEVENTY_FIVE.1)?;
        let values = [
            tex_after(&kit, stage, mix_products[0], &n.mix_values[0], palette::mix())?,
            tex_after(&kit, stage, mix_products[1], &n.mix_values[1], ink)?,
        ];
        fade_in_all(stage, &values[..1], 0.5)?;
        line.wait_for(stage, AT_SIXTY_THREE.1)?;
        fade_in_all(stage, &values[1..], 0.5)?;
        line.wait_for(stage, AT_TURNS.1)?;
        let turning = round_paths(stage, true, palette::mix());
        draw_bands(stage, &turning, 1.4)?;
        let mut mix_panel = vec![rule, condition, words[0], words[1]];
        mix_panel.extend(mix_products);
        mix_panel.extend(values);
        mix_panel.extend(&one);
        keep_apart(stage, &mix_panel, 0.05, "the mix's products")?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 15_12: the one line on the mix's crowd of 1,418, its numbers set
        // over its letters
        let line = say(&kit, stage, "15_12")?;
        line.wait_for(stage, AT_CHECKS.1)?;
        let crowd_words = panel_words(&kit, stage, &n.crowd_words, palette::player(), LEGEND_Y)?;
        fade_in_all(stage, &[crowd_words], 0.5)?;
        let pointer = frame_around(stage, &one, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_POS_NUMBER.1)?;
        let factor = panel_tex(&kit, stage, &n.factor, ink, PANEL_X, ROWS[2])?;
        fade_in_all(stage, &[factor], 0.5)?;
        line.wait_for(stage, AT_DIFFER.1)?;
        let difference = panel_tex(&kit, stage, &n.difference, ink, PANEL_X, STEP_YS[0])?;
        fade_in_all(stage, &[difference], 0.5)?;
        line.wait_for(stage, AT_TWELVE_TIMES.1)?;
        let numbers_line = centred_over(&kit, stage, &n.one_line, &one, palette::player(), STEP_YS[1])?;
        fade_in_all(stage, &numbers_line[..2], 0.5)?;
        line.wait_for(stage, AT_AGAINST_490.1)?;
        fade_in_all(stage, &numbers_line[2..], 0.5)?;
        line.wait_for(stage, AT_BOTH_SIDES.1)?;
        let sides = tex_after(&kit, stage, numbers_line[4], &n.sides, palette::mix())?;
        fade_in_all(stage, &[sides], 0.5)?;
        let pointer = frame_around(stage, &[sides], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut checked = mix_panel.clone();
        checked.extend([crowd_words, factor, difference, sides]);
        checked.extend(&numbers_line);
        keep_apart(stage, &checked, 0.05, "the one line in numbers")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 15_13: a player's average from the net flow, and the exact card
        let line = say(&kit, stage, "15_13")?;
        line.wait_for(stage, AT_SPREAD.1)?;
        let per_player = panel_tex(&kit, stage, &n.per_player, palette::mix(), PANEL_X, STEP_YS[0])?;
        let unit = unit_after(&kit, stage, per_player, EACH)?;
        let anims = swap(stage, &[difference], &[per_player, unit], 0.6)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_EXACT.1)?;
        let pointer = frame_around(stage, &[per_player, unit], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_CENTS.1)?;
        let k06 = precisely(&kit, stage, &n.k06)?;
        fade_in_all(stage, &[k06], 0.5)?;
        let card_up = now(stage);
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&n.k06))?;
        fade_out_all(stage, &[k06], 0.4)?;

        // 15_14: the published coins; a table of A, B and the mix decided by
        // their products
        let line = say(&kit, stage, "15_14")?;
        line.wait_for(stage, AT_SHAVE.1)?;
        let mut leaving = vec![crowd_words, factor, per_player, unit, sides, words[0], words[1]];
        leaving.extend(mix_products);
        leaving.extend(values);
        leaving.extend(&numbers_line);
        leaving.extend(&one);
        leaving.extend(&turning);
        fade_out_all(stage, &leaving, 0.5)?;
        line.wait_for(stage, AT_PUBLISHED.1)?;
        let pub_tag = full_badge(&kit, stage, &n.published_badge, PANEL_END)?;
        let pub_labels = mix_labels(&kit, stage, &n.published_mix)?;
        let mut old = tag.to_vec();
        old.extend(&labels);
        let mut new = pub_tag.to_vec();
        new.extend(&pub_labels);
        let mut anims = swap(stage, &old, &new, CHANGE_S)?;
        for m in crowd.to(n.shares[2], CHANGE_S)? {
            anims.push(stage.prepare(m)?);
        }
        stage.play_prepared(anims)?;
        labels = pub_labels;
        let mut texts = labels.clone();
        texts.extend(t.names);
        texts.push(title);
        keep_apart(stage, &texts, 0.08, "the published mix's triangle")?;
        let heads = [
            kit.text_left(stage, PRODUCTS_HEAD, 18.0, dim, v(PROD_X, ROWS[0]))?,
            kit.text_left(stage, AVERAGE_HEAD, 18.0, dim, v(AVG_X, ROWS[0]))?,
        ];
        keep_between(stage, heads[0], PROD_X - 0.01, VERDICT_X - 0.1, "the products' head")?;
        keep_between(stage, heads[1], AVG_X - 0.01, PANEL_END, "the averages' head")?;
        let cents_unit = panel_words(&kit, stage, CENTS_UNIT, dim, STEP_YS[1])?;
        fade_in_all(stage, &[heads[0], heads[1], cents_unit], 0.5)?;
        let table_ys = [ROWS[1], ROWS[2], STEP_YS[0]];
        let colors = [palette::game_a(), palette::game_b(), palette::mix()];
        let mut table = Vec::with_capacity(3);
        for (k, at) in [AT_A, AT_B, AT_MIX].iter().enumerate() {
            let y = table_ys[k];
            let verdict_color = if n.table[k][2] == WINS {
                palette::good()
            } else {
                palette::bad()
            };
            let row = [
                kit.text_left(stage, &n.table[k][0], WORDS_SIZE, colors[k], v(EDGE_X, y))?,
                kit.tex_left(stage, &n.table[k][1], LINE_SIZE, ink, &[], v(PROD_X, y))?,
                kit.text_left(stage, &n.table[k][2], WORDS_SIZE, verdict_color, v(VERDICT_X, y))?,
                kit.text_left(stage, &n.table[k][3], WORDS_SIZE, verdict_color, v(AVG_X, y))?,
            ];
            keep_between(stage, row[0], EDGE_X - 0.01, PROD_X - 0.1, "a game's name")?;
            keep_between(stage, row[1], PROD_X - 0.01, VERDICT_X - 0.1, "a game's products")?;
            keep_between(stage, row[2], VERDICT_X - 0.01, AVG_X - 0.1, "a game's verdict")?;
            keep_between(stage, row[3], AVG_X - 0.01, PANEL_END, "a game's average")?;
            line.wait_for(stage, at.1)?;
            fade_in_all(stage, &row[..3], 0.5)?;
            fade_in_all(stage, &row[3..], 0.4)?;
            table.push(row);
        }
        line.wait_for(stage, AT_DECIDE.1)?;
        let products_column: Vec<Mob> = table.iter().map(|r| r[1]).collect();
        let pointer = frame_around(stage, &products_column, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        let mut published = vec![rule, condition, heads[0], heads[1], cents_unit];
        published.extend(table.iter().flatten());
        keep_apart(stage, &published, 0.05, "the published table")?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;

        // 15_15: no crowd and no weights: just the chances forward and back,
        // each set multiplied (held)
        let line = say(&kit, stage, "15_15")?;
        line.wait_for(stage, AT_NEVER.1)?;
        let pointer = frame_around(stage, &[rule], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        line.wait_for(stage, AT_NO_CROWD.1)?;
        together(stage, crowd.to([0.0; 3], DRAIN_S)?)?;
        line.wait_for(stage, AT_FORWARD3.1)?;
        let forward_round = round_paths(stage, true, palette::mix());
        draw_bands(stage, &forward_round, 1.2)?;
        let pointer = frame_around(stage, &labels[..3], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_BACK3.1)?;
        let back_round = round_paths(stage, false, dim);
        draw_bands(stage, &back_round, 1.2)?;
        let pointer = frame_around(stage, &labels[3..], POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.7)?;
        line.wait_for(stage, AT_MULTIPLIED.1)?;
        let pointer = frame_around(stage, &products_column, POINT_PAD, hero)?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;
        pause(stage, INSPECT_S)?;
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
    use super::super::three_corners::{B_AT, TITLE_DY};
    use super::*;
    use crate::model::Game;

    /// Every game whose chances are tenths 1 to 9 (as the oracle's 729).
    fn tenths() -> Vec<Game> {
        let mut out = Vec::with_capacity(729);
        for p0 in 1..10 {
            for p1 in 1..10 {
                for p2 in 1..10 {
                    out.push(Game {
                        up: [p0, p1, p2].map(|t| t * model::DEN / 10),
                        den: model::DEN,
                    });
                }
            }
        }
        out
    }

    fn chances_of(g: &Game) -> ([Ratio; 3], [Ratio; 3]) {
        let p = [0, 1, 2].map(|i| g.chance(i));
        (p, p.map(|x| Ratio::ONE - x))
    }

    #[test]
    fn the_numbers_are_the_facts() {
        let n = numbers().unwrap();
        assert_eq!(n.built, [10.0 / 26.0, 4.0 / 26.0, 12.0 / 26.0]);
        assert_eq!(n.built, n.shares[0]);
        assert_eq!(
            n.b_rows[0],
            ["10", r"\times \tfrac{1}{10}", "=", "4", r"\times \tfrac{1}{4}", "= 1"]
        );
        assert_eq!(
            n.b_rows[1],
            ["4", r"\times \tfrac{3}{4}", "=", "12", r"\times \tfrac{1}{4}", "= 3"]
        );
        assert_eq!(
            n.b_rows[2],
            ["12", r"\times \tfrac{3}{4}", "=", "10", r"\times \tfrac{9}{10}", "= 9"]
        );
        assert_eq!(
            n.b_line,
            r"\tfrac{1}{10} \cdot \tfrac{3}{4} \cdot \tfrac{3}{4} = \tfrac{9}{160} = \tfrac{9}{10} \cdot \tfrac{1}{4} \cdot \tfrac{1}{4}"
        );
        assert_eq!(n.averaged, "averaging B's forward chances: 53.3 %, a winner?");
        assert_eq!(
            n.mix_products,
            [r"30\% \cdot 62.5\% \cdot 62.5\%", r"70\% \cdot 37.5\% \cdot 37.5\%"]
        );
        assert_eq!(n.mix_values, [r"= \tfrac{75}{640}", r"= \tfrac{63}{640}"]);
        assert_eq!(n.crowd_words, "the mix's 1,418 players: 490 at corner 0, net flow 12");
        assert_eq!(
            n.factor,
            r"p_2 + q_1 q_2 = 62.5\% + 37.5\% \cdot 37.5\% = \tfrac{49}{64}"
        );
        assert_eq!(
            n.difference,
            r"p_0 p_1 p_2 - q_0 q_1 q_2 = \tfrac{75}{640} - \tfrac{63}{640} = \tfrac{3}{160}"
        );
        assert_eq!(
            n.one_line,
            ["12", r"\times \tfrac{49}{64}", "=", "490", r"\times \tfrac{3}{160}"]
        );
        assert_eq!(n.sides, r"= \tfrac{147}{16}");
        assert_eq!(
            n.per_player,
            r"\tfrac{3 \times 12}{1{,}418} = \tfrac{18}{709} \text{ dollars} \approx 2.54 \text{ cents}"
        );
        assert_eq!(
            n.table,
            [
                ["game A", "0.1213 < 0.1288", "loses", "\u{2212}1.00"],
                ["game B", "0.0527 < 0.0588", "loses", "\u{2212}0.87"],
                ["the mix", "0.1134 > 0.1018", "wins", "+1.57"],
            ]
        );
        assert_eq!(n.published_mix.heads, ["29.5", "62", "62"]);
    }

    /// The converse, on every game of tenths: put w₀ = 1 and let the first
    /// two balances fix w₁ = p₀/q₁ and w₂ = w₁ p₁/q₂ (every q positive);
    /// the third balance then holds exactly when the products match, and
    /// then the built crowd is the settled one (proportional to its shares,
    /// unmoved by a round) with no net flow, a fair game. Failed control:
    /// when the products differ, the built crowd fails the third balance and
    /// a round moves it, so it is not the settled crowd.
    #[test]
    fn equal_products_balance_every_edge() {
        let (mut equal, mut unequal) = (0, 0);
        for g in tenths() {
            let (p, q) = chances_of(&g);
            let w = [Ratio::ONE, p[0] / q[1], p[0] * p[1] / (q[1] * q[2])];
            assert!(w.iter().all(|&x| x > Ratio::ZERO), "{g:?}");
            assert_eq!(w[0] * p[0], w[1] * q[1]);
            assert_eq!(w[1] * p[1], w[2] * q[2]);
            let (pf, pb) = g.products();
            let third = w[2] * p[2] == w[0] * q[0];
            assert_eq!(third, pf == pb, "{g:?}");
            let total = w[0] + w[1] + w[2];
            let as_shares = w.map(|x| x / total);
            let moved = model::evolve(&w, &g) != w;
            if third {
                equal += 1;
                assert_eq!(as_shares, g.shares(), "{g:?}");
                assert!(!moved, "{g:?}");
                assert_eq!(g.mean(), Ratio::ZERO, "{g:?}");
            } else {
                unequal += 1;
                assert_ne!(as_shares, g.shares(), "{g:?}");
                assert!(moved, "{g:?}");
                assert_ne!(g.mean(), Ratio::ZERO, "{g:?}");
            }
        }
        assert!(equal > 0 && unequal > 0, "{equal} equal, {unequal} unequal");
    }

    /// The one line, on every game of tenths: net × (p₂ + q₁ q₂) = w₀ ×
    /// (p₀ p₁ p₂ − q₀ q₁ q₂) with the settled shares, the factor and w₀
    /// positive, so the average (three times the net flow) has the sign of
    /// the difference. Failed controls: a coin certain to win at corner 1
    /// and one certain to lose at corner 2 make both the factor and w₀ zero,
    /// so the line decides nothing; and the difference's size does not
    /// order the gains (it gives the sign only).
    #[test]
    fn the_one_line_gives_the_sign() {
        for g in tenths() {
            let (p, q) = chances_of(&g);
            let w = g.shares();
            let net = w[0] * p[0] - w[1] * q[1];
            let (pf, pb) = g.products();
            let factor = p[2] + q[1] * q[2];
            assert!(factor > Ratio::ZERO && w[0] > Ratio::ZERO, "{g:?}");
            assert_eq!(net * factor, w[0] * (pf - pb), "{g:?}");
            assert_eq!(g.mean(), Ratio::int(model::MODULUS) * net, "{g:?}");
            assert_eq!(g.mean().signum(), (pf - pb).signum(), "{g:?}");
        }
        let certain = Game {
            up: [model::DEN / 2, model::DEN, 0],
            den: model::DEN,
        };
        let (p, q) = chances_of(&certain);
        assert_eq!(p[2] + q[1] * q[2], Ratio::ZERO);
        assert_eq!(certain.shares()[0], Ratio::ZERO);
        let tenth = |t: [i64; 3]| Game {
            up: t.map(|x| x * model::DEN / 10),
            den: model::DEN,
        };
        let (small, big) = (tenth([1, 8, 9]), tenth([4, 4, 8]));
        let gap = |g: &Game| {
            let (f, b) = g.products();
            f - b
        };
        assert!(Ratio::ZERO < gap(&small) && gap(&small) < gap(&big));
        assert!(small.mean() > big.mean());
    }

    /// B's products match while its averaged forward chance is over a half
    /// (the misjudgement 15_9 strikes); the mix's forward product is the
    /// bigger, and the one line holds on its crowd of 1,418; the per-player
    /// average is exactly the model's. Failed control: B's built crowd under
    /// the mix's chances does not balance its edges.
    #[test]
    fn the_products_decide_b_and_the_mix() {
        let (b, mix) = (ROUND.game_b(), ROUND.mix());
        let (p, _) = chances_of(&b);
        assert!((p[0] + p[1] + p[2]) / Ratio::int(3) > Ratio::new(1, 2));
        assert_eq!(b.products(), (Ratio::new(9, 160), Ratio::new(9, 160)));
        assert_eq!(b.mean(), Ratio::ZERO);
        let (fm, bm) = mix.products();
        assert_eq!((fm, bm), (Ratio::new(75, 640), Ratio::new(63, 640)));
        let (mp, mq) = chances_of(&mix);
        let factor = mp[2] + mq[1] * mq[2];
        assert_eq!(factor, Ratio::new(49, 64));
        assert_eq!(fm - bm, Ratio::new(3, 160));
        assert_eq!(Ratio::int(12) * factor, Ratio::int(490) * (fm - bm));
        assert_eq!(Ratio::int(12) * factor, Ratio::new(147, 16));
        assert_eq!(Ratio::int(3) * Ratio::int(12) / Ratio::int(1418), mix.mean());
        assert_eq!(mix.mean(), Ratio::new(18, 709));
    }

    /// The published coins: each game's products order as its average's
    /// sign, A and B lose and the mix wins; the round-coin B was the fair
    /// edge between.
    #[test]
    fn the_published_products_decide_a_b_and_the_mix() {
        for (g, wins) in [
            (PUBLISHED.game_a(), false),
            (PUBLISHED.game_b(), false),
            (PUBLISHED.mix(), true),
        ] {
            let (f, b) = g.products();
            assert_eq!(f > b, wins, "{g:?}");
            assert_eq!(g.mean() > Ratio::ZERO, wins, "{g:?}");
        }
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        for pair in CUES.windows(2) {
            let [(a, x), (b, y)] = [pair[0], pair[1]];
            if a == b {
                let text = script.text(a).unwrap();
                assert!(text.find(x) < text.find(y), "{a}: {x:?} after {y:?}");
            }
        }
    }

    /// The "precisely" card fits the lower third's three lines (as 05's).
    #[test]
    fn the_card_fits_three_lines() {
        let n = numbers().unwrap();
        let lines = explainer_kit::animatic::wrap(&n.k06, 76, usize::MAX);
        assert!(lines.len() <= 3, "{lines:?}");
        assert_eq!(lines.join(" "), n.k06);
    }

    /// The layout keeps its rooms: the claims, the condition, the legend,
    /// the rows, the steps and the words under them stack without touching,
    /// above the frame's foot; the products line rises clear of the chain;
    /// the steps standing during the card stay above a three-line lower
    /// third, as does the triangle's title; the table's columns keep apart.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
            assert!(CLAIM_YS[0] + 0.2 < FRAME_Y_RADIUS - 0.3);
            assert!(CLAIM_YS[0] - CLAIM_YS[1] >= 0.45 && CLAIM_YS[1] - COND_Y >= 0.45);
            assert!(COND_Y - LEGEND_Y >= 0.4 && LEGEND_Y - ROWS[0] >= 0.5);
            assert!(ROWS[0] - ROWS[1] >= 0.5 && ROWS[1] - ROWS[2] >= 0.5);
            assert!(ROWS[2] - STEP_YS[0] >= 0.55);
            assert!(STEP_YS[0] - STEP_YS[1] >= 0.55 && STEP_YS[1] - STEP_YS[2] >= 0.55);
            assert!(STEP_YS[2] - UNDER_Y >= 0.35 && UNDER_Y - LOW_YS[0] >= 0.4);
            assert!(LOW_YS[0] - LOW_YS[1] >= 0.4 && LOW_YS[1] - 0.2 > -FRAME_Y_RADIUS + 0.3);
            assert!(STEP_YS[2] - L4_Y >= 0.55 && L4_Y - 0.2 > -FRAME_Y_RADIUS + 0.3);
            assert!(STEP_YS[0] - CHAIN_YS[0] >= 0.5);
            assert!(CHAIN_YS[0] - CHAIN_YS[1] >= 0.5 && CHAIN_YS[1] - CHAIN_YS[2] >= 0.5);
            assert!(CHAIN_YS[2] - 0.15 > -FRAME_Y_RADIUS + 0.3);
            assert!(STEP_YS[2] - 0.25 > card_top);
            assert!(B_AT[1] - TITLE_DY - 0.08 > card_top);
            assert!(EDGE_X + 0.75 < PROD_X && PROD_X + 2.2 < VERDICT_X);
            assert!(VERDICT_X + 0.7 < AVG_X && AVG_X + 0.7 < PANEL_END);
            assert!(EDGE_X + 0.75 < CELL_X);
        }
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 15 is said once, in order;
    /// the prediction (15_10) gets its silent time; the derivation's steps
    /// and the reveals stand to be read; the card is readable; the last
    /// picture stands.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TwoProducts { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("15_two_products: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=15).map(|k| format!("15_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("15_11") - end_of("15_10") >= THINK_S,
            "{} s to think",
            at("15_11") - end_of("15_10")
        );
        for (before, after) in [("15_3", "15_4"), ("15_9", "15_10"), ("15_11", "15_12")] {
            assert!(
                at(after) - end_of(before) >= BEAT,
                "{before}: {} s",
                at(after) - end_of(before)
            );
        }
        for (before, after) in [
            ("15_5", "15_6"),
            ("15_6", "15_7"),
            ("15_7", "15_8"),
            ("15_8", "15_9"),
            ("15_12", "15_13"),
            ("15_14", "15_15"),
        ] {
            assert!(
                at(after) - end_of(before) >= INSPECT_S,
                "{before}: {} s",
                at(after) - end_of(before)
            );
        }
        let n = numbers().unwrap();
        assert!(at("15_14") - at("15_13") >= read_s(&n.k06), "the card is not readable");
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(
            end - end_of("15_15") >= INSPECT_S,
            "the last picture is not held: {end}"
        );
    }
}
