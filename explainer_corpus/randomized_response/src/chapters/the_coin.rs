//! Chapter 03, the class on average: the class routed through the coin rule
//! on average, half to the truth and half to the second coin (03_1, 03_2);
//! the survey sees seven yes and thirteen no (03_3) and is asked how it could
//! get back to four (03_4, a prediction); take away the second coin's yeses,
//! a quarter of the class, and double what is left (03_5, 03_6).
//!
//! The chapter opens on the picture chapter 02 ends its rule in (the class,
//! its rings and key, the coin, both lanes and the four places), rebuilt by
//! [`RulePicture`] and faded in; chapters 01 and 02 share the class's layout
//! and that picture from here.
//!
//! Every student is routed by `model::route` and answers by `model::answer`
//! with the coins `Class::average_coins` gives it; every count drawn is
//! checked against the facts' tally before the survey's view is shown.

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, Timed, add, fade_all, signed_thousands, v};
use fmn::prelude::*;

use super::draw::{
    coin, disc, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, hollow, lane, move_all, pause, place, ring,
    say, swap, think, wait_until,
};
use crate::facts::{self, Average};
use crate::model::{self, CLASS_COLUMNS, COIN_PAIRS, Channel, Coin, Q, Route};
use crate::palette;

pub struct TheClassOnAverage {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
pub(super) const QUESTION: &str = "Have you ever copied homework?";
pub(super) const KEY: &str = "copied: only we can see this";
const COIN_WORD: &str = "coin";
const HEADS_TRUTH: &str = "heads: the truth";
const TAILS: &str = "tails";
const SECOND_COIN: &str = "second coin";
const HEADS: &str = "heads";
const YES: &str = "yes";
const NO: &str = "no";
const PROGRAMMERS: &str = "for programmers:";

// ------------------------------------------------------------------ layout

/// The class: four rows of five on the left, `SEAT_DX` apart.
const SEAT_X0: f64 = -6.35;
const SEAT_Y0: f64 = 0.75;
const SEAT_DX: f64 = 0.5;
/// The question across the top.
pub(super) const QUESTION_Y: f64 = 3.35;
/// The key under the class: a ring and its words.
pub(super) const KEY_X: f64 = -6.45;
pub(super) const KEY_Y: f64 = -2.3;
/// The key's words start this far right of its ring.
pub(super) const KEY_DX: f64 = 0.32;
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

/// Seat `i` of the class.
pub(super) fn seat_at(i: usize) -> Vec3 {
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

/// The centre of the answer `said`'s cluster once the survey's view leaves
/// only the answers (03_3 on): the upper place's, so the truthful keep the
/// slots they landed in and the second coin's students join them.
fn answers_at(said: bool) -> Vec3 {
    place_at(Route::Truth, said)
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

/// Pair the points `from` with the slots `to`, both taken left to right and
/// then bottom to top. Slots and points here share the same five x
/// positions, so each student then moves straight up or down and no two
/// paths cross. Each index into `from` comes with its slot.
fn matched(from: &[Vec3], mut to: Vec<Vec3>) -> Vec<(usize, Vec3)> {
    let order = |a: &Vec3, b: &Vec3| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1]));
    let mut which: Vec<usize> = (0..from.len()).collect();
    which.sort_by(|&a, &b| order(&from[a], &from[b]));
    to.sort_by(order);
    which.into_iter().zip(to).collect()
}

/// A whole number as the chapter writes it ("−10" with a typographic minus).
fn whole_text(q: Q) -> fmn::Result<String> {
    q.integer()
        .map(signed_thousands)
        .ok_or_else(|| fail(format!("{q} is not a whole number")))
}

/// 03_1's exact version: what "on average" means here.
fn average_note() -> String {
    format!(
        "fair coins, flipped in private. These are the average counts, each of the {} coin outcomes equally \
         often. A real class strays from them",
        COIN_PAIRS.len()
    )
}

/// 03_6's code card: the estimate, and its edges for a class of `n`.
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

/// The coin rule as chapter 02 draws it and chapter 03 opens on: the
/// question, the key, the first coin and its truth lane, the column heads,
/// the tails lane through the second coin, the four places and the two
/// words in the lower places. Every part is detached; a chapter brings in
/// the parts it needs when it needs them.
pub(super) struct RulePicture {
    pub question: Mob,
    pub key_ring: Mob,
    pub key_text: Mob,
    pub first_coin: Mob,
    pub coin_label: Mob,
    pub truth_lane: Mob,
    pub truth_label: Mob,
    pub yes_head: Mob,
    pub no_head: Mob,
    pub truth_places: [Mob; 2],
    pub tails_lane: Mob,
    pub tails_label: Mob,
    pub second: Mob,
    pub second_label: Mob,
    pub coin_lane: Mob,
    pub coin_places: [Mob; 2],
    pub heads_word: Mob,
    pub tails_word: Mob,
}

impl RulePicture {
    /// Every part, detached.
    pub(super) fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<RulePicture> {
        let question = kit.text(stage, QUESTION, 34.0, palette::student(), v(0.0, QUESTION_Y))?;
        let key_ring = ring(stage, v(KEY_X, KEY_Y));
        let key_text = kit.text_left(stage, KEY, 20.0, Palette::dim(), v(KEY_X + KEY_DX, KEY_Y))?;
        let first_coin = coin(stage, palette::student(), v(COIN_X, 0.0), COIN_R);
        let coin_label = kit.text_left(stage, COIN_WORD, 20.0, Palette::dim(), v(COIN_X + COIN_R + 0.12, 0.0))?;
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
        let heads_word = kit.text_left(
            stage,
            HEADS,
            18.0,
            palette::forced(),
            word_at(place_at(Route::ForcedYes, true)),
        )?;
        let tails_word = kit.text_left(
            stage,
            TAILS,
            18.0,
            palette::forced(),
            word_at(place_at(Route::ForcedNo, false)),
        )?;
        Ok(RulePicture {
            question,
            key_ring,
            key_text,
            first_coin,
            coin_label,
            truth_lane,
            truth_label,
            yes_head,
            no_head,
            truth_places,
            tails_lane,
            tails_label,
            second,
            second_label,
            coin_lane,
            coin_places,
            heads_word,
            tails_word,
        })
    }

    /// "Heads, tell the truth": the truth lane, its words, the column heads
    /// and the upper places.
    pub(super) fn truth_parts(&self) -> Vec<Mob> {
        let mut out = vec![self.truth_lane, self.truth_label, self.yes_head, self.no_head];
        out.extend(self.truth_places);
        out
    }

    /// "Tails, flip a second coin": the tails lane through the second coin,
    /// their words and the lower places.
    pub(super) fn tails_parts(&self) -> Vec<Mob> {
        let mut out = vec![
            self.tails_lane,
            self.tails_label,
            self.second,
            self.second_label,
            self.coin_lane,
        ];
        out.extend(self.coin_places);
        out
    }

    /// The rule's routes, which the survey never sees: everything but the
    /// question, the key and the column heads.
    pub(super) fn routes(&self) -> Vec<Mob> {
        let mut out = vec![
            self.first_coin,
            self.coin_label,
            self.truth_lane,
            self.truth_label,
            self.tails_lane,
            self.tails_label,
            self.second,
            self.second_label,
            self.coin_lane,
            self.heads_word,
            self.tails_word,
        ];
        out.extend(self.truth_places);
        out.extend(self.coin_places);
        out
    }

    /// Every part.
    pub(super) fn all(&self) -> Vec<Mob> {
        let mut out = vec![self.question, self.key_ring, self.key_text, self.yes_head, self.no_head];
        out.extend(self.routes());
        out
    }
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

/// Send the students `which` to the slots `to`, paired by [`matched`], and
/// record where each one lands. Returns the moves.
fn regroup(students: &mut [Student], which: &[usize], to: Vec<Vec3>) -> Vec<(Mob, Vec3)> {
    let from: Vec<Vec3> = which.iter().map(|&i| students[i].at).collect();
    matched(&from, to)
        .into_iter()
        .map(|(j, at)| {
            let s = &mut students[which[j]];
            s.at = at;
            (s.disc, at)
        })
        .collect()
}

/// Fade `out` while the moves run, all together in `rt` seconds (the draw
/// helpers play one kind of animation at a time).
fn fade_out_moving(stage: &mut Stage<'_>, out: &[Mob], moves: &[(Mob, Vec3)], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in out {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &(m, to) in moves {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .move_to(to, ORIGIN)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
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

impl SceneConstruct for TheClassOnAverage {
    fn name(&self) -> &str {
        "03_the_class_on_average"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let [forced, left, estimate] = facts::undo(&a).map_err(fail)?;
        let (n, tally) = (a.class.size(), a.tally);
        let (yes, no) = (tally.answers(true), tally.answers(false));
        let factor = Q::ONE / Channel::two_coins().truth();

        // the picture chapter 02 left its rule in: the class with its rings
        // and key, the question, the coin, both lanes and the four places
        let rule = RulePicture::build(&kit, stage)?;
        let mut students = Vec::with_capacity(n);
        for (i, &coins) in a.coins.iter().enumerate() {
            let at = seat_at(i);
            let copied = a.class.copied(i);
            let seated = disc(stage, palette::student(), at);
            let ringed = if copied { Some(ring(stage, at)) } else { None };
            students.push(Student {
                copied,
                coins,
                disc: seated,
                ring: ringed,
                at,
            });
        }
        if students.iter().filter(|s| s.ring.is_some()).count() != a.class.copiers() {
            return Err(fail("a ring for every copier, and no other".into()));
        }
        let mut opening = rule.all();
        for s in &students {
            opening.push(s.disc);
            opening.extend(s.ring);
        }

        // 03_1: heads, half the class, tells the truth
        let line = say(&kit, stage, "03_1")?;
        fade_in_all(stage, &opening, 0.5)?;
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

        // 03_2: tails, the other half, through the second coin
        let line = say(&kit, stage, "03_2")?;
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

        // 03_3: the totals, and the survey's view
        let line = say(&kit, stage, "03_3")?;
        fade_out_all(stage, &[note, rule.question], 0.3)?;
        let yes_total = kit.text(stage, &yes.to_string(), 44.0, palette::yes(), v(YES_X, TOTAL_Y))?;
        let no_total = kit.text(stage, &no.to_string(), 44.0, palette::no(), v(NO_X, TOTAL_Y))?;
        fade_in_all(stage, &[yes_total, no_total], 0.5)?;
        line.wait_for(stage, "That's all")?;
        let mut hidden = rule.routes();
        hidden.extend([rule.key_ring, rule.key_text]);
        hidden.extend(counts);
        for s in &mut students {
            hidden.extend(s.ring.take());
        }
        // What the survey sees: one cluster of yeses and one of noes, with
        // nothing left to tell which way anyone went. As the tree fades, the
        // second coin's students rise into the free slots of their answer's
        // cluster, after the truthful (review PinkDuck F1).
        let mut gather = Vec::new();
        for said in [true, false] {
            let risers: Vec<usize> = (0..n)
                .filter(|&i| students[i].route() != Route::Truth && students[i].said() == said)
                .collect();
            let first = filled[cell_index(Route::Truth, said)];
            if first + risers.len() != tally.answers(said) {
                return Err(fail(format!(
                    "the {} cluster holds {}, the tally {}",
                    if said { YES } else { NO },
                    first + risers.len(),
                    tally.answers(said)
                )));
            }
            let slots = (first..first + risers.len())
                .map(|k| slot(answers_at(said), k))
                .collect();
            gather.extend(regroup(&mut students, &risers, slots));
        }
        fade_out_moving(stage, &hidden, &gather, 0.8)?;

        // 03_4: the survey's question, and the think time
        let line = say(&kit, stage, "03_4")?;
        line.wait_for(stage, "two numbers")?;
        flash_all(stage, &[yes_total, no_total], 0.6)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 03_5: take away the second coin's yeses, a quarter of the class
        let line = say(&kit, stage, "03_5")?;
        fade_out_all(stage, &[q], 0.2)?;
        let (mut old, mut faint) = (Vec::new(), Vec::new());
        for s in students.iter_mut().filter(|s| s.route() == Route::ForcedYes) {
            let f = disc(stage, palette::ghost(), s.at);
            old.push(s.disc);
            faint.push(f);
            s.disc = f;
        }
        swap(stage, &old, &faint, 0.5)?;
        // The faint ones step down to the row under the yeses that are left,
        // so 03_6's doubling lands beside those yeses.
        let kept = filled[cell_index(Route::Truth, true)];
        let doubled = estimate - left;
        let extra = doubled
            .integer()
            .and_then(|d| usize::try_from(d).ok())
            .ok_or_else(|| fail(format!("doubling adds {doubled} students")))?;
        let below = (kept + extra).div_ceil(SLOTS_PER_ROW) * SLOTS_PER_ROW;
        let dimmed: Vec<usize> = (0..n).filter(|&i| students[i].route() == Route::ForcedYes).collect();
        let slots = (below..below + dimmed.len())
            .map(|k| slot(answers_at(true), k))
            .collect();
        let down = regroup(&mut students, &dimmed, slots);
        move_all(stage, &down, 0.4)?;
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

        // 03_6: double them; four, and the rings come back
        let line = say(&kit, stage, "03_6")?;
        line.wait_for(stage, "So double them")?;
        // the copiers among the other half, whom the second coin hid: beside
        // the yeses that are left, in the slots the faint ones cleared, with
        // the factor at the end of that row
        let mut ghosts = Vec::new();
        for k in kept..kept + extra {
            ghosts.push(hollow(stage, palette::yes(), slot(answers_at(true), k)));
        }
        ghosts.push(kit.tex(
            stage,
            &format!(r"\times {factor}"),
            30.0,
            Palette::ink(),
            &[],
            v(count_at(answers_at(true))[0], slot(answers_at(true), kept)[1]),
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
        let code_text = code_card(n)?;
        let code = lower_third(&kit, stage, PROGRAMMERS, &code_text)?;
        fade_in_all(stage, &[code], 0.4)?;
        // count reading time from the fully visible card
        let code_read_by = stage.scene().time().to_f64() + code_text.chars().count() as f64 / 15.0;
        line.wait_end(stage)?;
        pause(stage, 1.5)?;
        wait_until(stage, code_read_by)?;

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
            ("03_1", "Two of our four"),
            ("03_2", "Five yes"),
            ("03_3", "That's all"),
            ("03_4", "two numbers"),
            ("03_5", "On average that's five"),
            ("03_5", "Two are left"),
            ("03_6", "So double them"),
            ("03_6", "Four."),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen say what the facts say: the average is named as
    /// one, and the code card's edges are the facts' (−10 and 30).
    #[test]
    fn the_exact_versions_are_built_from_the_facts() {
        let avg = average_note();
        assert!(
            avg.contains("average counts") && avg.contains("A real class strays"),
            "{avg}"
        );
        let code = code_card(20).unwrap();
        assert!(code.starts_with("estimate = 2 * yes - n / 2"), "{code}");
        assert!(code.contains("no yeses give \u{2212}10, 20 give 30"), "{code}");
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-X2.estimate.no_yes"), Some("-10"));
        assert_eq!(f.get("R-X2.estimate.all_yes"), Some("30"));
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to,
    /// so the band clears the lower row of places.
    #[test]
    fn the_cards_are_two_lines() {
        for text in [average_note(), code_card(20).unwrap()] {
            for width in (72..=92).rev().step_by(4) {
                let lines = explainer_kit::animatic::wrap(&text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
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
    }

    /// The survey's view keeps no trace of the routes (review PinkDuck F1).
    /// Each answer becomes one cluster holding its total, and every one of the
    /// second coin's students rises straight up into it. The doubling (03_6)
    /// then fits in the row of the yeses that are left.
    #[test]
    fn the_answers_gather_into_one_cluster_each() {
        let a = facts::average().unwrap();
        let mut filled = [0usize; 4];
        let mut risers: [Vec<Vec3>; 2] = [Vec::new(), Vec::new()];
        for (i, &(first, second)) in a.coins.iter().enumerate() {
            let route = model::route(first, second);
            let said = model::answer(a.class.copied(i), first, second);
            let c = cell_index(route, said);
            let at = slot(place_at(route, said), filled[c]);
            filled[c] += 1;
            if route != Route::Truth {
                risers[usize::from(!said)].push(at);
            }
        }
        for (said, from) in [true, false].into_iter().zip(&risers) {
            let first = filled[cell_index(Route::Truth, said)];
            assert_eq!(first + from.len(), a.tally.answers(said));
            let to: Vec<Vec3> = (first..first + from.len()).map(|k| slot(answers_at(said), k)).collect();
            for (j, at) in matched(from, to) {
                assert!((at[0] - from[j][0]).abs() < 1e-9, "a riser moves sideways");
                assert!(at[1] > from[j][1], "a riser does not rise");
            }
        }
        let [_, left, estimate] = facts::undo(&a).unwrap();
        let extra = usize::try_from((estimate - left).integer().unwrap()).unwrap();
        assert!(filled[cell_index(Route::Truth, true)] + extra <= SLOTS_PER_ROW);
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
        // the key below the class and clear of the lower lane and the band
        assert!(KEY_Y < last[1] - RING_R - 0.3);
        const { assert!(KEY_Y < COIN_ROW_Y - 0.3, "the key clears the lower lane") };
        const { assert!(KEY_Y - 0.15 > BAND_TOP, "the key over the band") };
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
    }
}
