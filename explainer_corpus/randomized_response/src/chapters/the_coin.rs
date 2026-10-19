//! Chapter 01, the coin: a direct question exposes whoever says yes (01_1);
//! only we can see who copied (01_2); the coin rule, drawn before any number
//! is derived (01_3, 01_4); the class routed through it on average, half to
//! the truth and half to the second coin (01_5, 01_6); the survey sees seven
//! yes and thirteen no (01_7) and is asked how it could get back to four
//! (01_8, a prediction); take away the second coin's yeses, a quarter of the
//! class, and double what is left (01_9, 01_10); but a yes is three times as
//! likely from someone who copied (01_11).
//!
//! Every student is routed by `model::route` and answers by `model::answer`
//! with the coins `Class::average_coins` gives it; every count drawn is
//! checked against the facts' tally before the survey's view is shown.

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, add, fade_all, signed_thousands, v};
use fmn::prelude::*;

use super::draw::{
    cell, coin, disc, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, hollow, lane, move_all, pause, place,
    ring, say, swap, think,
};
use crate::facts::{self, Average};
use crate::model::{self, CLASS_COLUMNS, COIN_PAIRS, Channel, Coin, Q, Route};
use crate::palette;

pub struct TheCoin {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const QUESTION: &str = "Have you ever copied homework?";
const KEY: &str = "copied: only we can see this";
const COIN_WORD: &str = "coin";
const HEADS_TRUTH: &str = "heads: the truth";
const TAILS: &str = "tails";
const SECOND_COIN: &str = "second coin";
const HEADS: &str = "heads";
const YES: &str = "yes";
const NO: &str = "no";
const TWO_COINS: &str = "the two coins:";
const COPIED: &str = "copied";
const DIDNT: &str = "didn't";
const PROGRAMMERS: &str = "for programmers:";

// ------------------------------------------------------------------ layout

/// The class: four rows of five on the left, `SEAT_DX` apart.
const SEAT_X0: f64 = -6.35;
const SEAT_Y0: f64 = 0.75;
const SEAT_DX: f64 = 0.5;
/// The question across the top.
const QUESTION_Y: f64 = 3.35;
/// The key under the class: a ring and its words.
const KEY_X: f64 = -6.45;
const KEY_Y: f64 = -2.3;
/// The first coin, its word to its right; the lanes leave it straight up
/// and straight down, then turn right along their rows.
const COIN_X: f64 = -3.1;
const COIN_R: f64 = 0.3;
const TAILS_Y: f64 = -0.95;
/// The rows: the truth on top, the second coin below.
const TRUTH_Y: f64 = 1.55;
const COIN_ROW_Y: f64 = -1.75;
/// The second coin on the lower row.
const SECOND_X: f64 = -1.55;
const SECOND_R: f64 = 0.24;
/// Where both lanes end, before the places.
const LANE_END_X: f64 = 0.25;
/// The two columns of places: yes and no, their heads and totals above.
const YES_X: f64 = 1.8;
const NO_X: f64 = 4.85;
const CELL_W: f64 = 2.8;
const CELL_H: f64 = 1.25;
const HEAD_Y: f64 = 2.6;
const TOTAL_Y: f64 = 3.3;
/// Inside a place: slots in rows of five from its left, the count at its
/// right, a small word at its top left.
const SLOT_X0: f64 = -1.0;
const SLOT_Y0: f64 = 0.12;
const SLOT_DX: f64 = 0.42;
const SLOTS_PER_ROW: usize = 5;
const COUNT_DX: f64 = 1.15;
const WORD_DY: f64 = 0.47;
/// Students travelling together keep this far apart.
const CLUSTER: f64 = 0.16;
/// The survey's arithmetic, on the left once the class has gone.
const DERIVE_X: f64 = -3.5;
const DERIVE_Y: [f64; 3] = [1.3, 0.3, -0.7];
const THINK_X: f64 = -3.3;
const THINK_Y: f64 = 0.6;
/// 01_11's grid of the four coin outcomes: one column each, a row for
/// someone who copied and a row for someone who didn't.
const GRID_X: [f64; 4] = [-1.95, -0.65, 0.65, 1.95];
const GRID_HEAD_Y: f64 = 1.75;
const GRID_ROW_Y: [f64; 2] = [0.8, -0.35];
const GRID_W: f64 = 1.1;
const GRID_H: f64 = 0.75;
const GRID_LABEL_X: f64 = -4.6;
const SHARE_X: f64 = 3.35;
const TIMES_X: f64 = 4.75;

/// Seat `i` of the class.
fn seat_at(i: usize) -> Vec3 {
    v(
        SEAT_X0 + SEAT_DX * (i % CLASS_COLUMNS) as f64,
        SEAT_Y0 - SEAT_DX * (i / CLASS_COLUMNS) as f64,
    )
}

/// The centre of the place for the answer `said` given by way of `route`.
fn place_at(route: Route, said: bool) -> Vec3 {
    let x = if said { YES_X } else { NO_X };
    let y = if route == Route::Truth { TRUTH_Y } else { COIN_ROW_Y };
    v(x, y)
}

/// The places in the order the tally lists them: the truth's yes and no,
/// then the second coin's.
fn cell_index(route: Route, said: bool) -> usize {
    let row = if route == Route::Truth { 0 } else { 2 };
    row + usize::from(!said)
}

/// Where the k-th student to land in the place centred at `centre` sits.
fn slot(centre: Vec3, k: usize) -> Vec3 {
    v(
        centre[0] + SLOT_X0 + SLOT_DX * (k % SLOTS_PER_ROW) as f64,
        centre[1] + SLOT_Y0 - SLOT_DX * (k / SLOTS_PER_ROW) as f64,
    )
}

/// Where a place's count stands.
fn count_at(centre: Vec3) -> Vec3 {
    v(centre[0] + COUNT_DX, centre[1] + SLOT_Y0)
}

/// Where a place's small word starts (its left edge).
fn word_at(centre: Vec3) -> Vec3 {
    v(centre[0] - CELL_W / 2.0 + 0.12, centre[1] + WORD_DY)
}

/// The k-th member's offset in a travelling cluster.
fn cluster(k: usize) -> Vec3 {
    v(
        CLUSTER * ((k % SLOTS_PER_ROW) as f64 - 2.0),
        CLUSTER * ((k / SLOTS_PER_ROW) as f64 - 0.5),
    )
}

/// A coin's face as the grid heads write it.
fn face(c: Coin) -> &'static str {
    match c {
        Coin::Heads => "H",
        Coin::Tails => "T",
    }
}

/// A whole number as the chapter writes it ("−10" with a typographic minus).
fn whole_text(q: Q) -> fmn::Result<String> {
    q.integer()
        .map(signed_thousands)
        .ok_or_else(|| fail(format!("{q} is not a whole number")))
}

/// 01_5's exact version: what "on average" means here.
fn average_note() -> String {
    format!(
        "fair coins, flipped in private. These are the average counts, each of the {} coin outcomes equally \
         often. A real class strays from them",
        COIN_PAIRS.len()
    )
}

/// 01_10's code card: the estimate, and its edges for a class of `n`.
fn code_card(n: usize) -> fmn::Result<String> {
    let factor = Q::ONE / Channel::two_coins().truth();
    let [_, _, lowest] = model::undo(0, n);
    let [_, _, highest] = model::undo(n, n);
    Ok(format!(
        "estimate = {factor} * yes - n / {factor}, in real arithmetic. It can fall outside 0 to n: no yeses \
         give {}, {n} give {}",
        whole_text(lowest)?,
        whole_text(highest)?
    ))
}

/// 01_11's exact version: both answers' ratios, and the bound.
fn ratio_note() -> fmn::Result<String> {
    let worst = Channel::two_coins()
        .worst_ratio()
        .ok_or_else(|| fail("the two coins leave a ratio unbounded".into()))?;
    Ok(format!(
        "a yes comes from {} of the {} coin outcomes if you copied, {} of {} if you didn't: {} times as likely, \
         never more. A no is {} times as likely if you didn't copy",
        model::outcomes_saying(true, true),
        COIN_PAIRS.len(),
        model::outcomes_saying(false, true),
        COIN_PAIRS.len(),
        worst,
        model::likelihood_ratio(false)
    ))
}

/// One student as the scene moves them: their truth and coins, what is
/// drawn for them now, and where.
struct Student {
    copied: bool,
    coins: (Coin, Coin),
    disc: Mob,
    ring: Option<Mob>,
    at: Vec3,
}

impl Student {
    fn route(&self) -> Route {
        model::route(self.coins.0, self.coins.1)
    }

    fn said(&self) -> bool {
        model::answer(self.copied, self.coins.0, self.coins.1)
    }
}

/// Move the students `which` along `path`, one leg at a time, `rt` seconds
/// a leg, as a small cluster (their rings with them).
fn travel(stage: &mut Stage<'_>, students: &mut [Student], which: &[usize], path: &[Vec3], rt: f64) -> fmn::Result<()> {
    for &point in path {
        let mut moves = Vec::new();
        for (k, &i) in which.iter().enumerate() {
            let at = add(point, cluster(k));
            let s = &mut students[i];
            moves.push((s.disc, at));
            if let Some(r) = s.ring {
                moves.push((r, at));
            }
            s.at = at;
        }
        move_all(stage, &moves, rt)?;
    }
    Ok(())
}

/// Bring the students `which` from the end of their lane into the places
/// their answers name, each to its place's next free slot, then recolour
/// them by their answer (fresh discs and rings, drawn over the places).
fn land(
    stage: &mut Stage<'_>,
    students: &mut [Student],
    which: &[usize],
    filled: &mut [usize; 4],
    rt: f64,
) -> fmn::Result<()> {
    let mut moves = Vec::new();
    let mut landed = Vec::new();
    for &i in which {
        let s = &students[i];
        let (route, said) = (s.route(), s.said());
        let c = cell_index(route, said);
        let at = slot(place_at(route, said), filled[c]);
        filled[c] += 1;
        moves.push((s.disc, at));
        if let Some(r) = s.ring {
            moves.push((r, at));
        }
        landed.push((i, at, said));
    }
    move_all(stage, &moves, rt)?;
    let (mut old, mut new) = (Vec::new(), Vec::new());
    for (i, at, said) in landed {
        let fresh = disc(stage, palette::answer(said), at);
        let s = &mut students[i];
        old.push(s.disc);
        new.push(fresh);
        s.disc = fresh;
        s.at = at;
        if let Some(r) = s.ring {
            let fresh_ring = ring(stage, at);
            old.push(r);
            new.push(fresh_ring);
            s.ring = Some(fresh_ring);
        }
    }
    swap(stage, &old, &new, 0.3)
}

/// The counts of the places on the row of `route`, beside them. Detached.
fn row_counts(kit: &Kit, stage: &mut Stage<'_>, filled: &[usize; 4], route: Route) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for said in [true, false] {
        let n = filled[cell_index(route, said)];
        out.push(kit.text(
            stage,
            &n.to_string(),
            26.0,
            palette::answer(said),
            count_at(place_at(route, said)),
        )?);
    }
    Ok(out)
}

impl SceneConstruct for TheCoin {
    fn name(&self) -> &str {
        "01_the_coin"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let [forced, left, estimate] = facts::undo(&a).map_err(fail)?;
        let (n, tally) = (a.class.size(), a.tally);
        let (yes, no) = (tally.answers(true), tally.answers(false));
        let factor = Q::ONE / Channel::two_coins().truth();

        // 01_1: the question and the class
        let _line = say(&kit, stage, "01_1")?;
        let question = kit.text(stage, QUESTION, 34.0, palette::student(), v(0.0, QUESTION_Y))?;
        fade_in_all(stage, &[question], 0.5)?;
        let mut students = Vec::with_capacity(n);
        for (i, &coins) in a.coins.iter().enumerate() {
            let at = seat_at(i);
            students.push(Student {
                copied: a.class.copied(i),
                coins,
                disc: disc(stage, palette::student(), at),
                ring: None,
                at,
            });
        }
        let discs: Vec<Mob> = students.iter().map(|s| s.disc).collect();
        fade_in_lagged(stage, &discs, 2.2, 0.15)?;

        // 01_2: only we can see who copied
        let line = say(&kit, stage, "01_2")?;
        line.wait_for(stage, "Four of them")?;
        let mut rings = Vec::new();
        for s in students.iter_mut().filter(|s| s.copied) {
            let r = ring(stage, s.at);
            s.ring = Some(r);
            rings.push(r);
        }
        if rings.len() != a.class.copiers() {
            return Err(fail(format!("{} rings for {} copiers", rings.len(), a.class.copiers())));
        }
        fade_in_lagged(stage, &rings, 0.9, 0.4)?;
        let key_ring = ring(stage, v(KEY_X, KEY_Y));
        let key_text = kit.text_left(stage, KEY, 20.0, Palette::dim(), v(KEY_X + 0.32, KEY_Y))?;
        fade_in_all(stage, &[key_ring, key_text], 0.4)?;

        // 01_3: a coin; heads, the truth
        let line = say(&kit, stage, "01_3")?;
        let first_coin = coin(stage, palette::student(), v(COIN_X, 0.0), COIN_R);
        let coin_label = kit.text_left(stage, COIN_WORD, 20.0, Palette::dim(), v(COIN_X + COIN_R + 0.12, 0.0))?;
        fade_in_all(stage, &[first_coin, coin_label], 0.4)?;
        line.wait_for(stage, "Heads, tell")?;
        let truth_lane = lane(
            stage,
            vec![v(COIN_X, COIN_R), v(COIN_X, TRUTH_Y), v(LANE_END_X, TRUTH_Y)],
            palette::truth(),
        );
        let truth_label = kit.text_left(
            stage,
            HEADS_TRUTH,
            22.0,
            palette::truth(),
            v(COIN_X + 0.15, TRUTH_Y + 0.32),
        )?;
        let yes_head = kit.text(stage, YES, 30.0, palette::yes(), v(YES_X, HEAD_Y))?;
        let no_head = kit.text(stage, NO, 30.0, palette::no(), v(NO_X, HEAD_Y))?;
        let truth_places = [
            place(stage, place_at(Route::Truth, true), CELL_W, CELL_H)?,
            place(stage, place_at(Route::Truth, false), CELL_W, CELL_H)?,
        ];
        let mut shown = vec![truth_lane, truth_label, yes_head, no_head];
        shown.extend(truth_places);
        fade_in_all(stage, &shown, 0.6)?;

        // 01_4: tails, the second coin answers: heads yes, tails no
        let line = say(&kit, stage, "01_4")?;
        let tails_lane = lane(
            stage,
            vec![
                v(COIN_X, -COIN_R),
                v(COIN_X, COIN_ROW_Y),
                v(SECOND_X - SECOND_R, COIN_ROW_Y),
            ],
            palette::forced(),
        );
        let tails_label = kit.text_left(stage, TAILS, 22.0, palette::forced(), v(COIN_X + 0.15, TAILS_Y))?;
        let second = coin(stage, palette::forced(), v(SECOND_X, COIN_ROW_Y), SECOND_R);
        let second_label = kit.text(
            stage,
            SECOND_COIN,
            20.0,
            palette::forced(),
            v(SECOND_X, COIN_ROW_Y - SECOND_R - 0.24),
        )?;
        let coin_lane = lane(
            stage,
            vec![v(SECOND_X + SECOND_R, COIN_ROW_Y), v(LANE_END_X, COIN_ROW_Y)],
            palette::forced(),
        );
        let coin_places = [
            place(stage, place_at(Route::ForcedYes, true), CELL_W, CELL_H)?,
            place(stage, place_at(Route::ForcedNo, false), CELL_W, CELL_H)?,
        ];
        let mut shown = vec![tails_lane, tails_label, second, second_label, coin_lane];
        shown.extend(coin_places);
        fade_in_all(stage, &shown, 0.6)?;
        line.wait_for(stage, "Heads for yes")?;
        let heads_word = kit.text_left(
            stage,
            HEADS,
            18.0,
            palette::forced(),
            word_at(place_at(Route::ForcedYes, true)),
        )?;
        fade_in_all(stage, &[heads_word], 0.3)?;
        flash_all(stage, &[yes_head, heads_word], 0.5)?;
        line.wait_for(stage, "tails for no")?;
        let tails_word = kit.text_left(
            stage,
            TAILS,
            18.0,
            palette::forced(),
            word_at(place_at(Route::ForcedNo, false)),
        )?;
        fade_in_all(stage, &[tails_word], 0.3)?;
        flash_all(stage, &[no_head, tails_word], 0.5)?;

        // 01_5: heads, half the class, tells the truth
        let line = say(&kit, stage, "01_5")?;
        let note = precisely(&kit, stage, &average_note())?;
        fade_in_all(stage, &[note], 0.4)?;
        let mut filled = [0usize; 4];
        let heads: Vec<usize> = (0..n).filter(|&i| students[i].route() == Route::Truth).collect();
        let lit: Vec<Mob> = heads.iter().map(|&i| students[i].disc).collect();
        flash_all(stage, &lit, 0.5)?;
        let up = [v(COIN_X, 0.0), v(COIN_X, TRUTH_Y), v(LANE_END_X, TRUTH_Y)];
        travel(stage, &mut students, &heads, &up, 0.55)?;
        line.wait_for(stage, "Two of our four")?;
        land(stage, &mut students, &heads, &mut filled, 0.7)?;
        let mut counts = row_counts(&kit, stage, &filled, Route::Truth)?;
        fade_in_all(stage, &counts, 0.3)?;

        // 01_6: tails, the other half, through the second coin
        let line = say(&kit, stage, "01_6")?;
        let tails: Vec<usize> = (0..n).filter(|&i| students[i].route() != Route::Truth).collect();
        let lit: Vec<Mob> = tails.iter().map(|&i| students[i].disc).collect();
        flash_all(stage, &lit, 0.5)?;
        let down = [
            v(COIN_X, 0.0),
            v(COIN_X, COIN_ROW_Y),
            v(SECOND_X, COIN_ROW_Y),
            v(LANE_END_X, COIN_ROW_Y),
        ];
        travel(stage, &mut students, &tails, &down, 0.45)?;
        line.wait_for(stage, "Five yes")?;
        land(stage, &mut students, &tails, &mut filled, 0.7)?;
        let lower = row_counts(&kit, stage, &filled, Route::ForcedYes)?;
        fade_in_all(stage, &lower, 0.3)?;
        counts.extend(lower);
        let want = [
            tally.count(Route::Truth, true),
            tally.count(Route::Truth, false),
            tally.route(Route::ForcedYes),
            tally.route(Route::ForcedNo),
        ];
        if filled != want {
            return Err(fail(format!("the places hold {filled:?}, the tally {want:?}")));
        }

        // 01_7: the totals, and the survey's view
        let line = say(&kit, stage, "01_7")?;
        fade_out_all(stage, &[note, question], 0.3)?;
        let yes_total = kit.text(stage, &yes.to_string(), 44.0, palette::yes(), v(YES_X, TOTAL_Y))?;
        let no_total = kit.text(stage, &no.to_string(), 44.0, palette::no(), v(NO_X, TOTAL_Y))?;
        fade_in_all(stage, &[yes_total, no_total], 0.5)?;
        line.wait_for(stage, "That's all")?;
        let mut hidden = vec![
            first_coin,
            coin_label,
            truth_lane,
            truth_label,
            tails_lane,
            tails_label,
            second,
            second_label,
            coin_lane,
            heads_word,
            tails_word,
            key_ring,
            key_text,
        ];
        hidden.extend(truth_places);
        hidden.extend(coin_places);
        hidden.extend(counts);
        for s in &mut students {
            hidden.extend(s.ring.take());
        }
        fade_out_all(stage, &hidden, 0.8)?;

        // 01_8: the survey's question, and the think time
        let line = say(&kit, stage, "01_8")?;
        line.wait_for(stage, "two numbers")?;
        flash_all(stage, &[yes_total, no_total], 0.6)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 01_9: take away the second coin's yeses, a quarter of the class
        let line = say(&kit, stage, "01_9")?;
        fade_out_all(stage, &[q], 0.2)?;
        let (mut old, mut faint) = (Vec::new(), Vec::new());
        for s in students.iter_mut().filter(|s| s.route() == Route::ForcedYes) {
            let f = disc(stage, palette::ghost(), s.at);
            old.push(s.disc);
            faint.push(f);
            s.disc = f;
        }
        swap(stage, &old, &faint, 0.5)?;
        line.wait_for(stage, "On average that's five")?;
        let quarter = whole_text(Q::int(n as i128) / forced)?;
        let quarter_tex = kit.tex(
            stage,
            &format!(r"\tfrac{{{n}}}{{{quarter}}} = {}", whole_text(forced)?),
            40.0,
            Palette::ink(),
            &[],
            v(DERIVE_X, DERIVE_Y[0]),
        )?;
        fade_in_all(stage, &[quarter_tex], 0.5)?;
        line.wait_for(stage, "Two are left")?;
        let left_tex = kit.tex(
            stage,
            &format!("{yes} - {} = {}", whole_text(forced)?, whole_text(left)?),
            40.0,
            Palette::ink(),
            &[],
            v(DERIVE_X, DERIVE_Y[1]),
        )?;
        fade_in_all(stage, &[left_tex], 0.5)?;
        let truthful_yes: Vec<Mob> = students
            .iter()
            .filter(|s| s.route() == Route::Truth && s.said())
            .map(|s| s.disc)
            .collect();
        flash_all(stage, &truthful_yes, 0.6)?;

        // 01_10: double them; four, and the rings come back
        let line = say(&kit, stage, "01_10")?;
        line.wait_for(stage, "So double them")?;
        // the copiers among the other half, whom the second coin hid
        let doubled = estimate - left;
        let extra = doubled
            .integer()
            .and_then(|d| usize::try_from(d).ok())
            .ok_or_else(|| fail(format!("doubling adds {doubled} students")))?;
        let first_free = filled[cell_index(Route::Truth, true)];
        let mut ghosts = Vec::new();
        for k in first_free..first_free + extra {
            ghosts.push(hollow(stage, palette::yes(), slot(place_at(Route::Truth, true), k)));
        }
        ghosts.push(kit.tex(
            stage,
            &format!(r"\times {factor}"),
            30.0,
            Palette::ink(),
            &[],
            count_at(place_at(Route::Truth, true)),
        )?);
        fade_in_lagged(stage, &ghosts, 0.8, 0.4)?;
        line.wait_for(stage, "Four.")?;
        let result = kit.tex(
            stage,
            &format!(r"{factor} \times {} = {}", whole_text(left)?, whole_text(estimate)?),
            40.0,
            Palette::ink(),
            &[],
            v(DERIVE_X, DERIVE_Y[2]),
        )?;
        let mut back = vec![result];
        for s in students.iter().filter(|s| s.copied) {
            back.push(ring(stage, s.at));
        }
        fade_in_all(stage, &back, 0.5)?;
        let code = lower_third(&kit, stage, PROGRAMMERS, &code_card(n)?)?;
        fade_in_all(stage, &[code], 0.4)?;
        line.wait_end(stage)?;
        pause(stage, 1.5)?;

        // 01_11: no answer is a confession, but a yes is three times likelier
        let line = say(&kit, stage, "01_11")?;
        fade_all(stage, 0.6)?;
        let mut grid = vec![kit.text_left(stage, TWO_COINS, 22.0, Palette::dim(), v(GRID_LABEL_X, GRID_HEAD_Y))?];
        for (&(first, second), &x) in COIN_PAIRS.iter().zip(&GRID_X) {
            let faces = format!("{} {}", face(first), face(second));
            grid.push(kit.text(stage, &faces, 24.0, Palette::dim(), v(x, GRID_HEAD_Y))?);
        }
        let mut yes_cells: [Vec<Mob>; 2] = [Vec::new(), Vec::new()];
        for (row, (&copied, &y)) in [true, false].iter().zip(&GRID_ROW_Y).enumerate() {
            let (word, color) = if copied {
                (COPIED, palette::copied())
            } else {
                (DIDNT, palette::student())
            };
            grid.push(kit.text_left(stage, word, 26.0, color, v(GRID_LABEL_X, y))?);
            for (&(first, second), &x) in COIN_PAIRS.iter().zip(&GRID_X) {
                let said = model::answer(copied, first, second);
                let c = cell(stage, palette::answer(said), v(x, y), GRID_W, GRID_H)?;
                let w = kit.text(stage, if said { YES } else { NO }, 24.0, palette::answer(said), v(x, y))?;
                grid.extend([c, w]);
                if said {
                    yes_cells[row].extend([c, w]);
                }
            }
        }
        fade_in_lagged(stage, &grid, 1.4, 0.08)?;
        line.wait_for(stage, "But someone who copied")?;
        let note = precisely(&kit, stage, &ratio_note()?)?;
        let share = |copied: bool| format!("{} of {}", model::outcomes_saying(copied, true), COIN_PAIRS.len());
        let copied_share = kit.text(stage, &share(true), 28.0, palette::yes(), v(SHARE_X, GRID_ROW_Y[0]))?;
        fade_in_all(stage, &[copied_share, note], 0.4)?;
        flash_all(stage, &yes_cells[0], 0.6)?;
        line.wait_for(stage, "as someone who didn't")?;
        let didnt_share = kit.text(stage, &share(false), 28.0, palette::yes(), v(SHARE_X, GRID_ROW_Y[1]))?;
        fade_in_all(stage, &[didnt_share], 0.3)?;
        flash_all(stage, &yes_cells[1], 0.5)?;
        let times = kit.tex(
            stage,
            &format!(r"\times {}", model::likelihood_ratio(true)),
            40.0,
            Palette::ink(),
            &[],
            v(TIMES_X, (GRID_ROW_Y[0] + GRID_ROW_Y[1]) / 2.0),
        )?;
        fade_in_all(stage, &[times], 0.4)?;
        line.wait_end(stage)?;
        pause(stage, 2.0)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv (a
    /// changed line fails here, not in the middle of a render).
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("01_2", "Four of them"),
            ("01_3", "Heads, tell"),
            ("01_4", "Heads for yes"),
            ("01_4", "tails for no"),
            ("01_5", "Two of our four"),
            ("01_6", "Five yes"),
            ("01_7", "That's all"),
            ("01_8", "two numbers"),
            ("01_9", "On average that's five"),
            ("01_9", "Two are left"),
            ("01_10", "So double them"),
            ("01_10", "Four."),
            ("01_11", "But someone who copied"),
            ("01_11", "as someone who didn't"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen say what the facts say: the average is named as
    /// one, both answers' ratios are given, and the code card's edges are
    /// the facts' (−10 and 30 for the class).
    #[test]
    fn the_exact_versions_are_built_from_the_facts() {
        let avg = average_note();
        assert!(
            avg.contains("average counts") && avg.contains("A real class strays"),
            "{avg}"
        );
        let ratio = ratio_note().unwrap();
        assert!(
            ratio.contains("3 of the 4 coin outcomes if you copied, 1 of 4"),
            "{ratio}"
        );
        assert!(ratio.contains("3 times as likely, never more"), "{ratio}");
        assert!(ratio.contains("A no is 3 times as likely"), "{ratio}");
        let code = code_card(20).unwrap();
        assert!(code.starts_with("estimate = 2 * yes - n / 2"), "{code}");
        assert!(code.contains("no yeses give \u{2212}10, 20 give 30"), "{code}");
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-X2.estimate.no_yes"), Some("-10"));
        assert_eq!(f.get("R-X2.estimate.all_yes"), Some("30"));
    }

    /// The cards stay two lines at the lower-third's width, so the band
    /// clears the lower row of places.
    #[test]
    fn the_cards_are_two_lines() {
        for text in [average_note(), ratio_note().unwrap(), code_card(20).unwrap()] {
            let lines = explainer_kit::animatic::wrap(&text, 92, 3);
            assert!(lines.len() <= 2, "{} lines: {text}", lines.len());
        }
    }

    /// The scene's rule is the model's: every student lands in the place of
    /// the answer the model gives them, and the places fill as the facts
    /// count them.
    #[test]
    fn the_places_fill_as_the_facts_count() {
        let a = facts::average().unwrap();
        let mut filled = [0usize; 4];
        for (i, &(first, second)) in a.coins.iter().enumerate() {
            let said = model::answer(a.class.copied(i), first, second);
            filled[cell_index(model::route(first, second), said)] += 1;
        }
        assert_eq!(filled, [2, 8, 5, 5]);
        assert_eq!(filled[0] + filled[2], a.tally.answers(true));
        assert_eq!(filled[1] + filled[3], a.tally.answers(false));
        const { assert!(GRID_X.len() == COIN_PAIRS.len(), "one grid column per coin outcome") };
    }

    /// The layout keeps its pieces apart and inside the frame.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use crate::chapters::draw::{DISC_R, RING_R};
        // the top of a two-line lower-third band (explainer_kit::props)
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        // the class and its rings inside the safe area, left of the coin
        let (first, last) = (seat_at(0), seat_at(model::CLASS_SIZE - 1));
        assert!(first[0] - RING_R > -FRAME_X_RADIUS + 0.15 && first[1] + RING_R < FRAME_Y_RADIUS - 0.15);
        assert!(last[0] + RING_R < COIN_X - COIN_R - 0.3);
        // the key below the class and clear of the lower lane
        assert!(KEY_Y < last[1] - RING_R - 0.3);
        const { assert!(KEY_Y < COIN_ROW_Y - 0.3, "the key clears the lower lane") };
        // the places fit the frame, apart, under their heads, clear of the band
        const { assert!(NO_X + CELL_W / 2.0 < FRAME_X_RADIUS - 0.15, "inside the safe area") };
        const { assert!(YES_X + CELL_W / 2.0 < NO_X - CELL_W / 2.0, "the columns apart") };
        const { assert!(TRUTH_Y + CELL_H / 2.0 < HEAD_Y - 0.2, "under the heads") };
        const { assert!(COIN_ROW_Y - CELL_H / 2.0 > BAND_TOP + 0.1, "over the band") };
        const {
            assert!(
                COIN_ROW_Y - SECOND_R - 0.36 > BAND_TOP,
                "the second coin's word over the band"
            )
        };
        const { assert!(LANE_END_X < YES_X - CELL_W / 2.0, "the lanes end before the places") };
        // the fullest place fits its slots; its count and word clear them
        let fullest = facts::average().unwrap().tally.count(Route::Truth, false);
        let centre = place_at(Route::Truth, false);
        assert!(
            slot(centre, fullest - 1)[1] - DISC_R > centre[1] - CELL_H / 2.0,
            "the last row fits"
        );
        let right = slot(centre, SLOTS_PER_ROW - 1)[0] + RING_R;
        assert!(right < count_at(centre)[0] - 0.15, "the count clears the slots");
        assert!(slot(centre, 0)[0] - RING_R > centre[0] - CELL_W / 2.0);
        assert!(
            word_at(centre)[1] - 0.09 > slot(centre, 0)[1] + RING_R,
            "the word clears a ring"
        );
        // the grid's cells sit apart, between its labels and its shares
        for pair in GRID_X.windows(2) {
            assert!(pair[1] - pair[0] > GRID_W + 0.1);
        }
        const {
            assert!(
                GRID_X[0] - GRID_W / 2.0 > GRID_LABEL_X + 1.6,
                "the row labels clear the grid"
            )
        };
        const { assert!(GRID_X[3] + GRID_W / 2.0 < SHARE_X - 0.6, "the shares clear the grid") };
        const { assert!(GRID_ROW_Y[1] - GRID_H / 2.0 > BAND_TOP + 0.3, "the grid over the band") };
    }
}
