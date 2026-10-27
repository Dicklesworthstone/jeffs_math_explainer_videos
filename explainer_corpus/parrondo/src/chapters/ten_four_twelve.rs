//! Chapter 09, a crowd that stays put. The picture 08 left: B's triangle
//! with the round-number coins' chances and the badge, the split crowd's
//! puddles and the panel's bars at 10, 4 and 12, "players: 26" and the dim
//! "split by the chances (expected numbers)". To check that the crowd stays
//! put, the bars and puddles become 26 dots again, 10 on corner 0, 4 on 1
//! and 12 on 2, still split exactly by the chances (09_1). Corner by corner
//! the dots split and wait at their arrows' starts, each group with its
//! count: corner 0's bad coin sends 1 forward (green) and 9 back (dim), the
//! good coin at corner 1 sends 3 on and 1 back (09_2), at corner 2 9 around
//! to corner 0 and 3 back (09_3). How many land on each corner? (09_4, a
//! prediction; nothing is summed yet.) All 26 move at once, straight to
//! the corners their arrows point at, while the groups' counts and the
//! question fade. The chance labels disappear during travel so the dots
//! cannot cross their digits, and return once the dots land. Corner by corner, the
//! arrivals' sum appears beside the corner, each summand in its stream's
//! color, listed by the corner it comes from: 1 + 9 = 10, 1 + 3 = 4,
//! 9 + 3 = 12, the counts as they were (09_5, held). The 13 forward movers
//! pulse; "winners: 1 + 3 + 9 = 13" and "13 of 26 = 1/2": without the half
//! point game B is fair (09_6). Corner 0's dots are ringed; a red bar grows
//! to 10 of 26, 38.5 %, past a dim mark at 33.3 % (09_7). The bar falls back
//! to the third, where B would win 53.3 % (07_7's hypothetical), and grows
//! again through the extra rounds at corner 0 while B's wins slide to 50.0 %
//! (09_8). Each corner's long-run share of the rounds is its weight: the
//! sums give way to 10/26 = 5/13, 4/26 = 2/13 and 12/26 = 6/13, and the
//! names to search come last (09_9).
//!
//! Every count, flow, sum and share comes from `crate::model` (the crowd's
//! flows, B's shares and chances) and is checked against `crate::facts`;
//! the scene refuses to draw if the words would stop being true of them.
//! The share bar, its percentage and B's wins read one tracker. No typeset
//! text is indicated, scaled or recoloured: dots, rings and the mark pulse
//! (a corner by a separate hollow ring, never its filled disc under the
//! dots), words get an outline, a changed line is replaced.

use explainer_kit::props::names_to_search;
use explainer_kit::{Kit, Palette, Timed, fade_all, live_bar, rect, tween, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    fade_in_all, fail, flash_all, flat, fly, keep_between, line_style, now, pause, read_s, say, stand, think,
    wait_until,
};
use super::looks_hopeless::{EDGE, POINT_PAD, ROOM, frame_around, point_at};
use super::one_player_to_a_crowd::{
    BAR_FULL, BAR_H, BAR_X, CLUSTER_R, DOT_R, NUM_X, PANEL_X, PLAYERS_NUM_X, PLAYERS_Y, ROW_YS, dot, panel_head,
    places, ring_corners, round_badge, round_badge_text, row_name, shown, slot,
};
use super::three_corners::{
    B_AT, CORNER_R, Chances, Which, arrow_ends, chance_labels, chances, corner, edge_of, keep_apart, swap, triangle,
};
use crate::facts;
use crate::model::{self, PUBLISHED, ROUND, Ratio};
use crate::palette;

pub struct TenFourTwelve {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const QUESTION: &str = "how many land on each corner?";
const WINNERS: &str = "winners:";
const SHARE_HEAD: &str = "share at corner 0:";
const B_WINS: &str = "B wins";
const PERCENT: &str = "%";
const WEIGHT_LINE: &str = "weight: a corner's long-run share of the rounds";
const NAMES: [&str; 2] = ["stationary distribution", "Markov chain"];

/// Where each picture lands: (line, the phrase it waits for).
const AT_STAYS_PUT: (&str, &str) = ("09_1", "really stays put");
const AT_STILL_SPLIT: (&str, &str) = ("09_1", "still split exactly by the chances");
const AT_TEN_TOSS: (&str, &str) = ("09_1", "ten players toss the bad coin");
const AT_ONE_WINS: (&str, &str) = ("09_1", "one wins and moves forward");
const AT_NINE_DROP: (&str, &str) = ("09_1", "nine drop back to corner two");
const AT_GOOD_FOUR: (&str, &str) = ("09_2", "the good coin splits four players");
const AT_THREE_ON: (&str, &str) = ("09_2", "Three move on to corner two");
const AT_ONE_BACK: (&str, &str) = ("09_2", "one falls back to zero");
const AT_SPLITS_TWELVE: (&str, &str) = ("09_3", "it splits twelve");
const AT_NINE_WRAP: (&str, &str) = ("09_3", "Nine wrap around to zero");
const AT_THREE_BACK: (&str, &str) = ("09_3", "three fall back to one");
const AT_HOW_MANY: (&str, &str) = ("09_4", "How many players land");
const AT_GETS: [(&str, &str); 3] = [
    ("09_5", "Corner zero gets"),
    ("09_5", "Corner one gets"),
    ("09_5", "Corner two gets"),
];
const AT_AS_THEY_WERE: (&str, &str) = ("09_5", "exactly as they were");
const AT_WINNERS: (&str, &str) = ("09_6", "Count the winners");
const AT_THIRTEEN: (&str, &str) = ("09_6", "make thirteen");
const AT_HALF: (&str, &str) = ("09_6", "exactly half of twenty-six");
const AT_FAIR: (&str, &str) = ("09_6", "perfectly fair");
const AT_SITS: (&str, &str) = ("09_7", "where the crowd sits");
const AT_TEN_OF: (&str, &str) = ("09_7", "Ten of twenty-six");
const AT_MORE_THIRD: (&str, &str) = ("09_7", "more than a third");
const AT_WITH_BAD: (&str, &str) = ("09_8", "with the bad coin");
const AT_CANCEL: (&str, &str) = ("09_8", "cancel the good coin's advantage exactly");
const AT_WEIGHT: (&str, &str) = ("09_9", "is called its weight");
const AT_WEIGHTS: [(&str, &str); 3] = [
    ("09_9", "five thirteenths"),
    ("09_9", "two thirteenths"),
    ("09_9", "six thirteenths"),
];
/// Each corner's split: (its words, its forward group's words, its back
/// group's), the phrases of 09_1 to 09_3.
const SPLITS: [[(&str, &str); 3]; 3] = [
    [AT_TEN_TOSS, AT_ONE_WINS, AT_NINE_DROP],
    [AT_GOOD_FOUR, AT_THREE_ON, AT_ONE_BACK],
    [AT_SPLITS_TWELVE, AT_NINE_WRAP, AT_THREE_BACK],
];
#[cfg(test)]
const CUES: [(&str, &str); 29] = [
    AT_STAYS_PUT,
    AT_STILL_SPLIT,
    AT_TEN_TOSS,
    AT_ONE_WINS,
    AT_NINE_DROP,
    AT_GOOD_FOUR,
    AT_THREE_ON,
    AT_ONE_BACK,
    AT_SPLITS_TWELVE,
    AT_NINE_WRAP,
    AT_THREE_BACK,
    AT_HOW_MANY,
    AT_GETS[0],
    AT_GETS[1],
    AT_GETS[2],
    AT_AS_THEY_WERE,
    AT_WINNERS,
    AT_THIRTEEN,
    AT_HALF,
    AT_FAIR,
    AT_SITS,
    AT_TEN_OF,
    AT_MORE_THIRD,
    AT_WITH_BAD,
    AT_CANCEL,
    AT_WEIGHT,
    AT_WEIGHTS[0],
    AT_WEIGHTS[1],
    AT_WEIGHTS[2],
];

// ------------------------------------------------------------------ layout

/// A departing group waits past its arrow's start: this far further along
/// the arrow and this far to the arrow's bow side (out of the triangle for
/// a forward arrow, into it for a back one); its count this much further.
const GROUP_ALONG: f64 = 0.3;
const GROUP_SIDE: f64 = 0.35;
const GROUP_LABEL_SIDE: f64 = 0.8;
/// An arrival sum beside its corner: right of corners 0 and 1, left of
/// corner 2, raised this much at the two lower corners (their names sit
/// below and outside them); its pieces this far apart.
const SUM_DY: [f64; 3] = [0.0, 0.35, 0.35];
const PIECE_GAP: f64 = 0.12;
const SUM_SIZE: f64 = 26.0;
/// The left panel below its head: the question and the winners' lines,
/// the share's words, bar and mark, B's wins and the weight's line.
const QUESTION_Y: f64 = 1.75;
const THINK_AT: Vec3 = [-3.9, 0.55, 0.0];
const WINNERS_YS: [f64; 2] = [1.75, 1.3];
const SHARE_HEAD_Y: f64 = 0.65;
const SHARE_BAR_X: f64 = -6.3;
const SHARE_BAR_Y: f64 = 0.2;
/// Scene units per percentage point of the share bar.
const SHARE_UNIT: f64 = 0.1;
const SHARE_NUM_X: f64 = -2.3;
const SHARE_PCT_X: f64 = -1.75;
const MARK_H: f64 = 0.25;
const MARK_LABEL_DY: f64 = 0.42;
const WINS_Y: f64 = -0.65;
const WINS_NUM_RIGHT: f64 = -4.85;
const WINS_PCT_X: f64 = -4.75;
const WEIGHT_Y: f64 = -1.45;
/// Seconds: a corner's group leaving for its arrow, everyone moving at
/// once, the hold on the unchanged counts, the beat that ends 09_6, the
/// share bar's growth and its fall back to a third, and the slide of B's
/// wins.
const SPLIT_S: f64 = 0.8;
const MOVE_S: f64 = 2.0;
const AS_WERE_HOLD: f64 = 2.0;
const BEAT: f64 = 1.0;
const GROW_S: f64 = 1.2;
const BACK_S: f64 = 0.6;
const SLIDE_S: f64 = 2.0;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    round_badge: String,
    b_round: Chances,
    n: usize,
    counts: [usize; 3],
    forward: [usize; 3],
    back: [usize; 3],
    /// At each corner, what arrives, listed by the corner it comes from:
    /// (players, forward or back), then the total.
    arrivals: [[(usize, bool); 2]; 3],
    winners: [String; 2],
    share_head: String,
    third: String,
    /// The settled share at corner 0 and a third, in percent.
    share_pct: f64,
    third_pct: f64,
    /// B's chance of a win at corner 0 and at the other two corners.
    p_bad: f64,
    p_good: f64,
    weights: [String; 3],
}

fn whole(x: Ratio, what: &str) -> Result<usize, String> {
    if !x.is_integer() {
        return Err(format!("{what} is not a whole number of players"));
    }
    usize::try_from(x.num()).map_err(|e| format!("{what}: {e}"))
}

fn csv<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    values.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let g = ROUND.game_b();
    let n = model::smallest_whole_crowd(&g);
    if get("crowd.size")? != n.to_string() || n != 26 {
        return Err("09_6 says twenty-six".into());
    }
    let shares = g.shares();
    let counts_r = shares.map(|w| Ratio::int(n) * w);
    let (forward_r, back_r) = model::flows(&g, n);
    if get("crowd.counts")? != csv(counts_r)
        || get("crowd.forward")? != csv(forward_r)
        || get("crowd.back")? != csv(back_r)
    {
        return Err("the crowd's counts or flows disagree with the facts".into());
    }
    let all_whole = |x: [Ratio; 3], what: &str| -> Result<[usize; 3], String> {
        let [a, b, c] = x.map(|r| whole(r, what));
        Ok([a?, b?, c?])
    };
    let counts = all_whole(counts_r, "a corner's crowd")?;
    let forward = all_whole(forward_r, "a forward flow")?;
    let back = all_whole(back_r, "a back flow")?;
    if (0..3).any(|i| forward[i] + back[i] != counts[i]) {
        return Err("a corner's split loses a player".into());
    }
    // 09_1 to 09_3: ten toss the bad coin, one wins and nine drop back;
    // the good coin splits four, three on and one back; twelve, nine
    // around to zero and three back
    if counts != [10, 4, 12] || forward != [1, 3, 9] || back != [9, 1, 3] {
        return Err("09_1 to 09_3 say 10: 1 and 9, 4: 3 and 1, 12: 9 and 3".into());
    }
    if g.chance(0) >= g.chance(1) || g.chance(1) != g.chance(2) {
        return Err("09_1 says the bad coin at corner 0 and the good coin at the others".into());
    }
    // 09_5: what arrives at each corner, by the corner it comes from, comes
    // back to the counts exactly
    let mut arrivals = [[(0usize, false); 2]; 3];
    for (j, arriving) in arrivals.iter_mut().enumerate() {
        let mut from: Vec<(usize, usize, bool)> = Vec::with_capacity(2);
        for (i, (&fw, &bk)) in forward.iter().zip(&back).enumerate() {
            if (i + 1) % 3 == j {
                from.push((i, fw, true));
            } else if (i + 2) % 3 == j {
                from.push((i, bk, false));
            }
        }
        from.sort_by_key(|&(i, ..)| i);
        if from.len() != 2 || from[0].1 + from[1].1 != counts[j] {
            return Err(format!("corner {j}'s arrivals do not come back to its count"));
        }
        *arriving = [(from[0].1, from[0].2), (from[1].1, from[1].2)];
    }
    let printed = facts::arrivals(&forward_r, &back_r);
    let said: Vec<String> = arrivals.iter().map(|a| format!("{}+{}", a[0].0, a[1].0)).collect();
    if get("crowd.arrivals")? != csv(&printed) || said != printed {
        return Err("09_5's sums disagree with the facts".into());
    }
    // 09_6: one, three and nine make thirteen, exactly half of twenty-six;
    // without the half point B is fair
    let winners = forward.iter().sum::<usize>();
    let half = Ratio::new(winners as i128, i128::from(n));
    if get("crowd.winners")? != winners.to_string()
        || half != Ratio::new(1, 2)
        || get("round.b.win")? != half.to_string()
    {
        return Err("09_6 says thirteen winners, exactly half".into());
    }
    if g.mean() != Ratio::ZERO || PUBLISHED.game_b().mean() >= Ratio::ZERO {
        return Err("09_6 says B is fair without the half point, and loses with it".into());
    }
    // 09_7: ten of twenty-six is 38.5 %, more than a third
    let third = Ratio::new(1, 3);
    if get("crowd.share0.pct")? != shares[0].pct(1) || shares[0].pct(1) != "38.5" || shares[0] <= third {
        return Err("09_7 says thirty-eight and a half percent, more than a third".into());
    }
    if get("thirds.pct")? != third.pct(1) {
        return Err("a third disagrees with the facts".into());
    }
    // 09_8: B's wins at a third (07_7's 53.3 %) and at its true share (a half)
    let (p_bad, p_good) = (g.chance(0), g.chance(1));
    let wins_at = |s: Ratio| s * p_bad + (Ratio::ONE - s) * p_good;
    if wins_at(third) != facts::thirds_win(&ROUND)
        || wins_at(third).pct(1) != get("round.b.thirds.win.pct")?
        || wins_at(shares[0]) != g.win()
        || g.win() != half
    {
        return Err("09_8 says the extra rounds at corner 0 bring 53.3 % down to a half".into());
    }
    // 09_9: the weights, five, two and six thirteenths, pinned in the facts
    if get("round.b.weights")? != csv(shares) || csv(shares) != "5/13,2/13,6/13" {
        return Err("09_9 says five, two and six thirteenths".into());
    }
    let weights = [0, 1, 2].map(|i| format!("{}/{n} = {}", counts[i], shares[i]));
    let [f0, f1, f2] = forward;
    Ok(Numbers {
        round_badge: round_badge_text(&f)?,
        b_round: chances(&g)?,
        n: usize::try_from(n).map_err(|e| format!("the crowd: {e}"))?,
        counts,
        forward,
        back,
        arrivals,
        winners: [
            format!("{WINNERS} {f0} + {f1} + {f2} = {winners}"),
            format!("{winners} of {n} = {half}"),
        ],
        share_head: format!("{SHARE_HEAD} {} of {n}", counts[0]),
        third: format!("{} {PERCENT}", third.pct(1)),
        share_pct: 100.0 * shares[0].to_f64(),
        third_pct: 100.0 * third.to_f64(),
        p_bad: p_bad.to_f64(),
        p_good: p_good.to_f64(),
        weights,
    })
}

// --------------------------------------------------------------- pictures

/// Where corner `i`'s group going forward (or back) waits: past the start
/// of its arrow, on the arrow's bow side; and where its count goes.
fn group_at(i: usize, forward: bool, side: f64) -> Vec3 {
    let to = if forward { (i + 1) % 3 } else { (i + 2) % 3 };
    let (e, fw) = edge_of(i, to);
    let (start, end) = arrow_ends(B_AT, e, fw);
    let len = (end[0] - start[0]).hypot(end[1] - start[1]);
    let u = [(end[0] - start[0]) / len, (end[1] - start[1]) / len];
    // the bow is to the left of travel: out of the triangle forward, in back
    let left = [-u[1], u[0]];
    v(
        start[0] + u[0] * GROUP_ALONG + left[0] * side,
        start[1] + u[1] * GROUP_ALONG + left[1] * side,
    )
}

/// The pieces of a sum laid out from `at` (its left edge, or its right edge
/// when `right`), [`PIECE_GAP`] apart, each in its own color. Detached.
fn sum_row(
    kit: &Kit,
    stage: &mut Stage<'_>,
    pieces: &[(String, Srgb)],
    at: Vec3,
    right: bool,
) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::with_capacity(pieces.len());
    let mut x = 0.0;
    for (words, color) in pieces {
        let m = kit.text_left(stage, words, SUM_SIZE, *color, v(x, at[1]))?;
        x = stage.get_bounding_box_point(m, RIGHT)[0] + PIECE_GAP;
        mobs.push(m);
    }
    let width = x - PIECE_GAP;
    let shift = if right { at[0] - width } else { at[0] };
    for &m in &mobs {
        let c = stage.get_center(m);
        stage.move_to(m, v(c[0] + shift, c[1]), ORIGIN);
    }
    Ok(mobs)
}

/// Where corner `j`'s sum stands: (its anchor, whether that is its right
/// edge).
fn sum_at(j: usize) -> (Vec3, bool) {
    let c = corner(B_AT, j);
    let reach = CORNER_R + ROOM;
    if j == 2 {
        (v(c[0] - reach, c[1] + SUM_DY[j]), true)
    } else {
        (v(c[0] + reach, c[1] + SUM_DY[j]), false)
    }
}

/// A forward count green, a back count dim.
fn flow_color(forward: bool) -> Srgb {
    if forward { palette::good() } else { Palette::dim() }
}

impl SceneConstruct for TenFourTwelve {
    fn name(&self) -> &str {
        "09_ten_four_twelve"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let size = n.n;

        // 09_1: 08's picture; the bars and puddles become 26 dots, still
        // split by the chances
        let line = say(&kit, stage, "09_1")?;
        let b = triangle(&kit, stage, Which::B)?;
        let labels = chance_labels(&kit, stage, Which::B, &n.b_round)?;
        let tag = round_badge(&kit, stage, &n.round_badge)?;
        let head = panel_head(&kit, stage)?;
        let total = kit.text_left(
            stage,
            &size.to_string(),
            22.0,
            Palette::ink(),
            v(PLAYERS_NUM_X, PLAYERS_Y),
        )?;
        let mut rows = Vec::with_capacity(9);
        let mut puddles = Vec::with_capacity(3);
        for (i, (&count, &y)) in n.counts.iter().zip(&ROW_YS).enumerate() {
            let share = count as f64 / size as f64;
            rows.push(row_name(&kit, stage, i)?);
            let bar = rect(
                v(BAR_X, y - BAR_H / 2.0),
                BAR_FULL * share,
                BAR_H,
                flat(palette::player(), 0.85),
            )?;
            rows.push(stage.arena_mut().add(bar));
            rows.push(kit.text_left(stage, &count.to_string(), 22.0, palette::player(), v(NUM_X, y))?);
            let puddle = Circle::new()
                .radius(CORNER_R * share.sqrt())
                .arc_center(corner(B_AT, i))
                .style(flat(palette::player(), 0.85))
                .build();
            puddles.push(stage.arena_mut().add(puddle));
        }
        let mut texts: Vec<Mob> = labels.to_vec();
        texts.extend(b.names);
        texts.extend([b.title, tag[1], head[0], head[1], total]);
        keep_apart(stage, &texts, 0.08, "08's picture")?;
        let mut arriving = b.parts();
        arriving.extend(labels);
        arriving.extend(tag);
        arriving.extend(head);
        arriving.push(total);
        arriving.extend(&rows);
        arriving.extend(&puddles);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, AT_STAYS_PUT.1)?;
        // the players listed corner by corner, each corner's forward
        // movers first
        let mut corner_of_player = Vec::with_capacity(size);
        for (i, &c) in n.counts.iter().enumerate() {
            corner_of_player.extend(std::iter::repeat_n(i, c));
        }
        let at_corners = places(&corner_of_player, |i| corner(B_AT, i));
        let dots: Vec<Mob> = at_corners.iter().map(|&p| dot(stage, p)).collect();
        let mut leaving = rows.clone();
        leaving.extend(&puddles);
        let anims = swap(stage, &leaving, &dots, 0.7)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, AT_STILL_SPLIT.1)?;
        let pointer = frame_around(stage, &[head[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;

        // 09_1 to 09_3: corner by corner, its players split and wait at
        // their arrows' starts, each group with its count
        let mut first = 0;
        let mut groups: Vec<[Vec<Mob>; 2]> = Vec::with_capacity(3);
        let mut group_labels = Vec::with_capacity(6);
        let mut line = line;
        for (i, (&count, cues)) in n.counts.iter().zip(&SPLITS).enumerate() {
            if i > 0 {
                kit.hold(stage)?;
                line = say(&kit, stage, &format!("09_{}", i + 1))?;
            }
            let here = &dots[first..first + count];
            first += count;
            let [whose, ahead, behind] = *cues;
            line.wait_for(stage, whose.1)?;
            ring_corners(stage, &[b.discs[i]], here, 0.6)?;
            let (go_on, go_back) = here.split_at(n.forward[i]);
            let mut pair: [Vec<Mob>; 2] = [Vec::new(), Vec::new()];
            for (k, (part, forward, cue)) in [(go_on, true, ahead), (go_back, false, behind)].into_iter().enumerate() {
                line.wait_for(stage, cue.1)?;
                let centre = group_at(i, forward, GROUP_SIDE);
                let spots: Vec<Vec3> = (0..part.len()).map(|j| slot(centre, j, size)).collect();
                let moving = if forward { n.forward[i] } else { n.back[i] };
                let label = kit.text(
                    stage,
                    &moving.to_string(),
                    24.0,
                    flow_color(forward),
                    group_at(i, forward, GROUP_LABEL_SIDE),
                )?;
                let mut moves = Vec::with_capacity(part.len());
                for (&d, &p) in part.iter().zip(&spots) {
                    moves.push(stage.prepare(fly(d, p, SPLIT_S)?)?);
                }
                let shown_label = fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?;
                moves.push(stage.prepare(shown_label.rt(SPLIT_S))?);
                stage.play_prepared(moves)?;
                pair[k] = part.to_vec();
                group_labels.push(label);
            }
            groups.push(pair);
        }
        let mut all_texts: Vec<Mob> = texts.clone();
        all_texts.extend(&group_labels);
        keep_apart(stage, &all_texts, 0.08, "the waiting groups' counts")?;

        // 09_4: everyone at once: how many land on each corner? (a
        // prediction; the dots still wait, nothing is summed)
        let line = say(&kit, stage, "09_4")?;
        line.wait_for(stage, AT_HOW_MANY.1)?;
        let question = kit.text_left(stage, QUESTION, 22.0, Palette::ink(), v(PANEL_X, QUESTION_Y))?;
        keep_between(stage, question, PANEL_X - 0.01, NUM_X + 0.6, "the question")?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 09_5: all 26 move along their arrows together; corner by corner
        // the arrivals' sum, each summand in its stream's color (held)
        let line = say(&kit, stage, "09_5")?;
        // where each group lands: at its destination corner, the arrivals
        // listed by the corner they come from, lowest first
        let mut landing: Vec<(Mob, Vec3)> = Vec::with_capacity(size);
        let mut forward_movers = Vec::with_capacity(size);
        for (j, &want) in n.counts.iter().enumerate() {
            let mut taken = 0;
            for (i, pair) in groups.iter().enumerate() {
                for (k, forward) in [(0, true), (1, false)] {
                    let to = if forward { (i + 1) % 3 } else { (i + 2) % 3 };
                    if to != j {
                        continue;
                    }
                    for &d in &pair[k] {
                        landing.push((d, slot(corner(B_AT, j), taken, size)));
                        taken += 1;
                        if forward {
                            forward_movers.push(d);
                        }
                    }
                }
            }
            if taken != want {
                return Err(fail(format!("corner {j} would not get back its {want} players")));
            }
        }
        let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(size + 14);
        for &(d, p) in &landing {
            anims.push(stage.prepare(fly(d, p, MOVE_S)?)?);
        }
        for &m in group_labels.iter().chain([&q]) {
            let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(gone.rt(0.4))?);
        }
        let gone = fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?;
        anims.push(stage.prepare(gone.rt(0.4))?);
        // The straight group flights cross these labels in actual frames.
        // Keep the arrows visible, hide their digits before the crossing,
        // and restore the chances only after every dot has landed.
        for &label in &labels {
            let gone = fade_out(stage.arena_mut(), label, ORIGIN, 1.0)?;
            anims.push(stage.prepare(gone.rt(0.2))?);
        }
        stage.play_prepared(anims)?;
        fade_in_all(stage, &labels, 0.3)?;
        let mut sums = Vec::with_capacity(3);
        for (j, at) in AT_GETS.into_iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let [(a, fa), (b2, fb)] = n.arrivals[j];
            let pieces = [
                (a.to_string(), flow_color(fa)),
                ("+".to_owned(), Palette::ink()),
                (b2.to_string(), flow_color(fb)),
                ("=".to_owned(), Palette::ink()),
                (n.counts[j].to_string(), palette::player()),
            ];
            let (anchor, right) = sum_at(j);
            let row = sum_row(&kit, stage, &pieces, anchor, right)?;
            for &m in &row {
                keep_between(
                    stage,
                    m,
                    -FRAME_X_RADIUS + EDGE,
                    FRAME_X_RADIUS - EDGE,
                    "an arrival sum",
                )?;
            }
            fade_in_all(stage, &row, 0.5)?;
            sums.push(row);
        }
        let mut all_texts: Vec<Mob> = texts.clone();
        all_texts.extend(sums.iter().flatten());
        keep_apart(stage, &all_texts, 0.06, "the arrival sums")?;
        line.wait_for(stage, AT_AS_THEY_WERE.1)?;
        for row in &sums {
            let pointer = frame_around(stage, &row[4..], POINT_PAD, Palette::hero())?;
            point_at(stage, pointer, &[], 0.5)?;
        }
        kit.hold(stage)?;
        pause(stage, AS_WERE_HOLD)?;

        // 09_6: the winners, 1 + 3 + 9 = 13 of 26, a half: B is fair
        let line = say(&kit, stage, "09_6")?;
        line.wait_for(stage, AT_WINNERS.1)?;
        flash_all(stage, &forward_movers, 0.8)?;
        line.wait_for(stage, AT_THIRTEEN.1)?;
        let winners = kit.text_left(stage, &n.winners[0], 22.0, Palette::ink(), v(PANEL_X, WINNERS_YS[0]))?;
        keep_between(stage, winners, PANEL_X - 0.01, NUM_X + 0.6, "the winners' line")?;
        fade_in_all(stage, &[winners], 0.5)?;
        line.wait_for(stage, AT_HALF.1)?;
        let half = kit.text_left(stage, &n.winners[1], 22.0, palette::good(), v(PANEL_X, WINNERS_YS[1]))?;
        keep_between(stage, half, PANEL_X - 0.01, NUM_X + 0.6, "the half")?;
        fade_in_all(stage, &[half], 0.5)?;
        line.wait_for(stage, AT_FAIR.1)?;
        let pointer = frame_around(stage, &[tag[1]], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;
        pause(stage, BEAT)?;

        // 09_7: where the crowd sits: corner 0's ten ringed; a bar to 10 of
        // 26, 38.5 %, past a mark at a third
        let line = say(&kit, stage, "09_7")?;
        line.wait_for(stage, AT_SITS.1)?;
        let ring = Circle::new()
            .radius(CLUSTER_R + DOT_R + 0.06)
            .arc_center(corner(B_AT, 0))
            .style(line_style(palette::bad(), 3.0))
            .build();
        let ring = stage.arena_mut().add(ring);
        fade_in_all(stage, &[ring], 0.4)?;
        flash_all(stage, &[ring], 0.6)?;
        line.wait_for(stage, AT_TEN_OF.1)?;
        let share_head = kit.text_left(stage, &n.share_head, 20.0, Palette::ink(), v(PANEL_X, SHARE_HEAD_Y))?;
        keep_between(stage, share_head, PANEL_X - 0.01, NUM_X + 0.6, "the share's words")?;
        let percent = kit.text_left(stage, PERCENT, 22.0, Palette::ink(), v(SHARE_PCT_X, SHARE_BAR_Y))?;
        fade_in_all(stage, &[share_head, percent], 0.4)?;
        let share = stage.add_value_tracker(0.0);
        live_bar(
            stage,
            share,
            v(SHARE_BAR_X, SHARE_BAR_Y),
            SHARE_UNIT,
            BAR_H,
            palette::bad(),
        )?;
        kit.live_value(
            stage,
            move |s| shown(s.tracker_value(share).unwrap_or(0.0), 1),
            1,
            false,
            22.0,
            Palette::ink(),
            v(SHARE_NUM_X, SHARE_BAR_Y),
            LEFT,
        )?;
        stage.play(tween(share, n.share_pct, GROW_S)?)?;
        line.wait_for(stage, AT_MORE_THIRD.1)?;
        let mark_x = SHARE_BAR_X + SHARE_UNIT * n.third_pct;
        let mark = polyline([v(mark_x, SHARE_BAR_Y - MARK_H), v(mark_x, SHARE_BAR_Y + MARK_H)])
            .style(line_style(Palette::dim(), 3.0))
            .build();
        let mark = stage.arena_mut().add(mark);
        let mark_label = kit.text(
            stage,
            &n.third,
            18.0,
            Palette::dim(),
            v(mark_x, SHARE_BAR_Y - MARK_LABEL_DY),
        )?;
        fade_in_all(stage, &[mark, mark_label], 0.4)?;
        flash_all(stage, &[mark], 0.6)?;

        // 09_8: back to a third, where B would win 53.3 %; through the extra
        // rounds at corner 0 B's wins slide to a half
        let line = say(&kit, stage, "09_8")?;
        stage.play(tween(share, n.third_pct, BACK_S)?)?;
        let wins_words = kit.text_left(stage, B_WINS, 22.0, palette::good(), v(PANEL_X, WINS_Y))?;
        keep_between(
            stage,
            wins_words,
            PANEL_X - 0.01,
            WINS_NUM_RIGHT - 0.6,
            "B's wins' words",
        )?;
        let wins_pct = kit.text_left(stage, PERCENT, 22.0, Palette::ink(), v(WINS_PCT_X, WINS_Y))?;
        fade_in_all(stage, &[wins_words, wins_pct], 0.3)?;
        let (p_bad, p_good) = (n.p_bad, n.p_good);
        let wins = kit.live_value(
            stage,
            move |s| {
                let at0 = s.tracker_value(share).unwrap_or(0.0) / 100.0;
                shown(100.0 * (at0 * p_bad + (1.0 - at0) * p_good), 1)
            },
            1,
            false,
            22.0,
            Palette::ink(),
            v(WINS_NUM_RIGHT, WINS_Y),
            RIGHT,
        )?;
        line.wait_for(stage, AT_WITH_BAD.1)?;
        stage.play(tween(share, n.share_pct, SLIDE_S)?)?;
        line.wait_for(stage, AT_CANCEL.1)?;
        let pointer = frame_around(stage, &[wins_words, wins, wins_pct], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        kit.hold(stage)?;

        // 09_9: each corner's long-run share is its weight: the sums give
        // way to 10/26 = 5/13, 4/26 = 2/13, 12/26 = 6/13; names to search
        let line = say(&kit, stage, "09_9")?;
        line.wait_for(stage, AT_WEIGHT.1)?;
        let weight = kit.text_left(stage, WEIGHT_LINE, 20.0, Palette::ink(), v(PANEL_X, WEIGHT_Y))?;
        keep_between(stage, weight, PANEL_X - 0.01, NUM_X + 0.6, "the weight's line")?;
        fade_in_all(stage, &[weight], 0.5)?;
        let mut shown_weights = Vec::with_capacity(3);
        for (j, at) in AT_WEIGHTS.into_iter().enumerate() {
            line.wait_for(stage, at.1)?;
            let (anchor, right) = sum_at(j);
            let w = sum_row(&kit, stage, &[(n.weights[j].clone(), palette::player())], anchor, right)?;
            for &m in &w {
                keep_between(stage, m, -FRAME_X_RADIUS + EDGE, FRAME_X_RADIUS - EDGE, "a weight")?;
            }
            let anims = swap(stage, &sums[j], &w, 0.5)?;
            stage.play_prepared(anims)?;
            shown_weights.extend(w);
        }
        let mut last_texts: Vec<Mob> = texts.clone();
        last_texts.extend(&shown_weights);
        last_texts.extend([winners, half, share_head, mark_label, wins_words, wins_pct, weight]);
        keep_apart(stage, &last_texts, 0.06, "the weights' picture")?;
        // the names to search as the chapter closes, after the line
        kit.hold(stage)?;
        let card = names_to_search(&kit, stage, &NAMES)?;
        if stage.get_bounding_box_point(card, UP)[1] > stage.get_bounding_box_point(b.title, DOWN)[1] - 0.1 {
            return Err(fail("the names to search run into game B's title".into()));
        }
        fade_in_all(stage, &[card], 0.5)?;
        let card_up = now(stage);
        wait_until(stage, card_up + read_s(&NAMES.join(" \u{b7} ")))?;
        stand(&kit, stage, card_up)?;
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
    use super::super::three_corners::name_at;
    use super::*;

    /// The chapter's words are true of its numbers: 10 at corner 0 split 1
    /// and 9, 4 at corner 1 split 3 and 1, 12 at corner 2 split 9 and 3;
    /// the arrivals 1 + 9, 1 + 3 and 9 + 3, listed by the corner they come
    /// from, are 10, 4 and 12; 13 of the 26 win, a half; 10 of 26 is
    /// 38.5 %; at a third B would win 53.3 %, at its share exactly 50 %; the
    /// weights are 5/13, 2/13 and 6/13.
    #[test]
    fn ten_four_twelve_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.n, 26);
        assert_eq!(n.counts, [10, 4, 12]);
        assert_eq!(n.forward, [1, 3, 9]);
        assert_eq!(n.back, [9, 1, 3]);
        assert_eq!(
            n.arrivals,
            [
                [(1, false), (9, true)],
                [(1, true), (3, false)],
                [(9, false), (3, true)]
            ]
        );
        assert_eq!(n.winners, ["winners: 1 + 3 + 9 = 13", "13 of 26 = 1/2"]);
        assert_eq!(n.share_head, "share at corner 0: 10 of 26");
        assert_eq!(n.third, "33.3 %");
        assert_eq!(n.weights, ["10/26 = 5/13", "4/26 = 2/13", "12/26 = 6/13"]);
        assert_eq!(format!("{:.1}", shown(n.share_pct, 1)), "38.5");
        let wins = |s: f64| {
            format!(
                "{:.1}",
                shown(100.0 * (s / 100.0 * n.p_bad + (1.0 - s / 100.0) * n.p_good), 1)
            )
        };
        assert_eq!(wins(n.third_pct), "53.3");
        assert_eq!(wins(n.share_pct), "50.0");
    }

    /// The crowd stays put: every corner sends out exactly its players and
    /// what arrives is the count it had, so one more round of the model
    /// leaves 10, 4, 12 exactly. Failed controls: a third each does not
    /// stay put; moving only the forward flows sends 9, 1 and 3, not 10, 4
    /// and 12; the published coins do not keep 10, 4, 12; and 26 real
    /// players at 10, 4, 12 come back to exactly 10, 4, 12 after one round
    /// only about 7 % of the time (0.0707: wins b at corner 1 with b − 2 at
    /// corner 0 and b + 6 at corner 2, b = 2, 3 or 4), so the check is of
    /// expected numbers, not of tosses.
    #[test]
    fn the_crowd_stays_put_and_the_controls_do_not() {
        let n = numbers().unwrap();
        let g = ROUND.game_b();
        let counts = n.counts.map(|c| Ratio::int(c as i64));
        assert_eq!(model::evolve(&counts, &g), counts);
        let sums: Vec<usize> = n.arrivals.iter().map(|a| a[0].0 + a[1].0).collect();
        assert_eq!(sums, n.counts);
        let third = [Ratio::new(26, 3); 3];
        assert_ne!(model::evolve(&third, &g), third);
        let mut forward_only = [0; 3];
        for (i, &fw) in n.forward.iter().enumerate() {
            forward_only[(i + 1) % 3] += fw;
        }
        assert_eq!(forward_only, [9, 1, 3]);
        assert_ne!(model::evolve(&counts, &PUBLISHED.game_b()), counts);
        let binomial = |trials: u64, wins: u64, p: f64| -> f64 {
            let mut c = 1.0;
            for k in 0..wins {
                c = c * (trials - k) as f64 / (k + 1) as f64;
            }
            c * p.powi(wins as i32) * (1.0 - p).powi((trials - wins) as i32)
        };
        let (p_bad, p_good) = (n.p_bad, n.p_good);
        let back_exactly: f64 = (2..=4)
            .map(|b| binomial(10, b - 2, p_bad) * binomial(4, b, p_good) * binomial(12, b + 6, p_good))
            .sum();
        assert!((back_exactly - 0.0707).abs() < 5e-5, "{back_exactly}");
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: every waiting group sits on its arrow's
    /// bow side (out of the triangle forward, in it back), clear of its
    /// corner's disc, with its count further out; the sums stand inside the
    /// frame clear of the corners' discs and names; the panel's lines stack
    /// without touching and the share bar ends before its number.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            assert!(QUESTION_Y - 0.15 > THINK_AT[1] + 0.62 + 0.1);
            assert!(WINNERS_YS[0] - WINNERS_YS[1] >= 0.4 && WINNERS_YS[1] - 0.15 > SHARE_HEAD_Y + 0.15);
            assert!(SHARE_HEAD_Y - 0.15 > SHARE_BAR_Y + MARK_H);
            assert!(SHARE_BAR_Y - MARK_LABEL_DY - 0.1 > WINS_Y + 0.15);
            assert!(WINS_Y - 0.15 > WEIGHT_Y + 0.15);
            assert!(WEIGHT_Y - 0.15 > -FRAME_Y_RADIUS + 0.2 + 0.66 + 0.1);
            assert!(GROUP_LABEL_SIDE > GROUP_SIDE + 0.36);
            assert!(SHARE_BAR_X + SHARE_UNIT * 100.0 * 10.0 / 26.0 < SHARE_NUM_X - 0.1);
        }
        let n = numbers().unwrap();
        // how far inside edge e (corner e to e + 1) a point is, up to the
        // edge's length: positive on the centroid's side
        let inward = |p: Vec3, e: usize| {
            let (a, b) = (corner(B_AT, e), corner(B_AT, (e + 1) % 3));
            let cross = |q: Vec3| (b[0] - a[0]) * (q[1] - a[1]) - (b[1] - a[1]) * (q[0] - a[0]);
            cross(p) * cross(B_AT).signum()
        };
        for (i, (&fw, &bk)) in n.forward.iter().zip(&n.back).enumerate() {
            for (forward, moving) in [(true, fw), (false, bk)] {
                let to = if forward { (i + 1) % 3 } else { (i + 2) % 3 };
                let (e, _) = edge_of(i, to);
                let g = group_at(i, forward, GROUP_SIDE);
                let label = group_at(i, forward, GROUP_LABEL_SIDE);
                let c = corner(B_AT, i);
                let spread = CLUSTER_R * (moving as f64 / 26.0).sqrt() + DOT_R;
                assert!(
                    (g[0] - c[0]).hypot(g[1] - c[1]) - spread > CORNER_R + 0.05,
                    "corner {i}'s group (forward {forward}) touches its disc"
                );
                if forward {
                    assert!(
                        inward(g, e) < 0.0 && inward(label, e) < inward(g, e),
                        "corner {i}'s forward group"
                    );
                } else {
                    assert!(
                        inward(g, e) > 0.0 && inward(label, e) > inward(g, e),
                        "corner {i}'s back group"
                    );
                }
            }
        }
        for j in 0..3 {
            let (anchor, right) = sum_at(j);
            let c = corner(B_AT, j);
            assert!((anchor[0] - c[0]).abs() >= CORNER_R + ROOM - 1e-12);
            assert_eq!(right, j == 2);
            let name = name_at(B_AT, j, 0.15, 0.25);
            assert!((anchor[1] - name[1]).abs() > 0.3, "corner {j}'s sum meets its name");
            assert!(anchor[0].abs() < FRAME_X_RADIUS - EDGE - 1.2);
        }
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 09 is said once, in order;
    /// the prediction (09_4) gets its silent think time; the unchanged
    /// counts are held after 09_5; 09_6 ends on a beat; the names to search
    /// stand their reading time at the end.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TenFourTwelve { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("09_ten_four_twelve: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=9).map(|k| format!("09_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("09_5") - end_of("09_4") >= THINK_S,
            "{} s to think",
            at("09_5") - end_of("09_4")
        );
        assert!(at("09_6") - end_of("09_5") >= AS_WERE_HOLD, "the counts are not held");
        assert!(at("09_7") - end_of("09_6") >= BEAT, "09_6 ends without a beat");
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(
            end - end_of("09_9") >= read_s(&NAMES.join(" \u{b7} ")) - 1e-9,
            "the names cannot be read: {end}"
        );
    }
}
