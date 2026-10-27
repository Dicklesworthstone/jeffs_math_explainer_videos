//! Chapter 08, the line on the dial: chapter 07's line of any two of five,
//! wrapped. The paper plot comes back as chapter 07 left it, its heights in
//! the cards, and the small dial it stood ready grows to the right. Each
//! height winds round that dial as a coil, one loop for every whole turn,
//! and lands: 9 stays 9, 13 goes once round to 2, then 6, 10 and 3; each
//! point drops onto its mark and its card is emptied and refilled (08_1).
//! The plot's band of marks 0 to 10 stretches into the dial board, the
//! straight line wraps into parallel pieces through every point, and from
//! the secret each step turns on 4, off the top and back in at the bottom
//! (08_2). Trustees 2 and 3, one step apart, differ by 4 (08_3), and two
//! steps back from trustee 2, each a turn back of 4 on the dial, find 9 and
//! then the secret 5 (08_4). Trustee 2 alone holds 2, the secret plus twice
//! the spin, and the viewer is asked how many secrets fit before anything
//! is counted (08_5). Then the strip comes in for trustee 2: one wrapped
//! line through (2, 2) for each spin, each meeting position 0 at its own
//! mark, all eleven (08_6), because doubling visits every mark once (08_7).
//! So the share is equally likely under every secret, a fact about the
//! fair spin, not an assumption about the secret, just like B's 7 in
//! chapter 03 (08_8).
//!
//! The heights, shares, turns, steps back and secrets by spin are the
//! model's (`Recipe`, `Dial`, `View`); the board, coils and wrapped pieces
//! `board.rs`'s. Emphasis never restyles or scales typeset text: labels and
//! cards are pointed at by outlines and rings of their own, a card is
//! emptied before its new number drops in, and only shapes (points, discs,
//! lines) change color or swell.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::board::{
    BOARD_POINT_R, Board, Grid, WRAPPED_W, coil, create_lagged, nearest_top, now, ring, taken_off, turns_and_mark,
    wrapped,
};
use super::draw::{
    Rail, STRIP_LEFT, STRIP_Y, Strip, TICK_H, TRUSTEE_R, card, disc, fade_in_all, fail, pause, read_by, recolorings,
    say, swaps, think, tick_at, wait_until,
};
use super::envelope::{PAD, frame_halos, outline, swelling};
use super::plot::{
    LAST, LINE_W, MARKER_DY, Marker, PLOT_DX, PLOT_X0, PLOT_Y0, Plot, TALLEST, at, band, marker_at, point, point_label,
    secret_name, segment,
};
use super::two_keys::{Face, check_strip, dial_secrets, dial_view, glow, glowing, leaving, moving, text_halo};
use crate::model::{A_SHARE, ADJACENT, Dial, LINE_HOLDER, LINE_SLOPE, Recipe, SECRET, TRUSTEES, View};
use crate::palette;

pub struct TheLineOnTheDial {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model)
const BOARD_TITLE: &str = "on the dial: points only";
const RULE: &str = "share = secret + 2 \u{d7} spin";
const FOR_TWO: &str = "for: trustee 2";
const SPIN_TITLE: &str = "spin";
const DOUBLE_TITLE: &str = "2 \u{d7} spin";
const PAST_TOP: &str = "past 10, take away 11";

/// The phrase each height starts winding at, trustee by trustee (08_1).
const WIND_CUES: [&str; 5] = [
    "Nine stays nine",
    "Thirteen goes once round",
    "Seventeen lands on six",
    "twenty-one on ten",
    "twenty-five on three",
];

// ------------------------------------------------------------------ layout

/// Chapter 08's board, under chapter 07's plot's columns, so a point that
/// drops onto its mark keeps its position, and the position discs stay.
pub const LINE_BOARD: Grid = Grid {
    x0: PLOT_X0,
    y0: -0.12,
    dx: PLOT_DX,
    dy: 0.29,
    marks: crate::model::MARKS,
    last: LAST,
};
/// Chapter 07's last dial, where it stood ready (a_line_through_the_secret),
/// and the wheel it grows into, right of the plot.
const READY_X: f64 = -0.3;
const READY_Y: f64 = 1.6;
const READY_R: f64 = 0.7;
const WHEEL_X: f64 = 2.4;
const WHEEL_Y: f64 = 1.2;
const WHEEL_R: f64 = 1.0;
/// What each wrapping takes off, one line a height, under the wheel.
const WORK_X: f64 = 2.4;
const WORK_Y: f64 = -0.45;
const WORK_DY: f64 = 0.36;
const WORK_SIZE: f64 = 24.0;
/// How far a card lifts out and drops in.
const DROP: f64 = 0.3;
/// A ring round a disc or a point, outside it.
const RING_OUT: f64 = 0.08;
/// A step's stroke, and its "+4" between the position discs.
const STEP_W: f64 = 3.0;
const STEP_LABEL_SIZE: f64 = 18.0;
/// 08_3's run and rise, and the rise's label right of it.
const RUN_W: f64 = 2.0;
const RISE_LABEL_DX: f64 = 0.22;
const RISE_LABEL_SIZE: f64 = 22.0;
/// The rule, top right, and trustee 2's own version of it under it.
const RULE_X: f64 = 3.6;
const RULE_Y: f64 = 3.3;
const RULE_SIZE: f64 = 26.0;
const OWN_Y: f64 = 2.8;
const OWN_SIZE: f64 = 24.0;
/// The think ring (as chapters 04 and 07's).
const THINK_X: f64 = 5.4;
const THINK_Y: f64 = -1.2;
/// The spins' lines through (2, 2), and the dot over each strip tick a
/// spin reaches.
const FAN_W: f64 = 2.0;
const DOT_R: f64 = 0.07;
const DOT_GAP: f64 = 0.14;
/// The spins' table, under the strip's caption.
const TABLE_Y: [f64; 3] = [-0.65, -1.05, -1.45];
const TABLE_SIZE: f64 = 22.0;
/// The doubling diagram: the spins' row and the doubles' row, under the
/// strip's ticks, their titles right-aligned left of them, the arrows
/// between and the note under.
const SPIN_ROW_Y: f64 = -0.8;
const DOUBLE_ROW_Y: f64 = -2.05;
const ROW_SIZE: f64 = 22.0;
const ROW_TITLE_SIZE: f64 = 20.0;
const ROW_TITLE_GAP: f64 = 0.15;
const ARROW_GAP: f64 = 0.2;
const ARROW_W: f64 = 2.0;
const HIT_R: f64 = 0.2;
const NOTE_DY: f64 = 0.45;
const NOTE_SIZE: f64 = 18.0;
/// How long the gold point holds after 08_4, and the cards after 08_8's
/// card is read.
const HOLD_ON_SECRET: f64 = 1.0;
const HOLD_AT_END: f64 = 0.5;

// -------------------------------------------------------------------- data

/// The marks a share `from` passes through stepping back `steps` times by
/// `step` on the dial, `from` first: 2, 9, 5 for trustee 2's 2 back twice
/// by 4.
fn chain(dial: Dial, from: u32, step: u32, steps: u32) -> Vec<u32> {
    let mut marks = vec![from];
    for _ in 0..steps {
        let last = *marks.last().unwrap_or(&from);
        marks.push(dial.sub(last, step));
    }
    marks
}

/// For each spin 0 to marks − 1, the secret that gives trustee `x` the
/// share `share`: share − x × spin, on the dial.
fn secrets_by_spin(dial: Dial, x: u32, share: u32) -> Vec<u32> {
    dial.all().map(|a| dial.sub(share, dial.mul(a, x))).collect()
}

/// `k` times each mark, as (mark, where it lands): the doubling's arrows.
fn doubling(dial: Dial, k: u32) -> Vec<(u32, u32)> {
    dial.all().map(|a| (a, dial.mul(k, a))).collect()
}

/// A step back on the dial: "2 − 4 → 9".
fn back_text(dial: Dial, from: u32, step: u32) -> String {
    format!("{from} \u{2212} {step} \u{2192} {}", dial.sub(from, step))
}

/// The rise between two neighbours: "6 − 2 = 4".
fn rise_text(dial: Dial, low: u32, high: u32) -> String {
    format!("{high} \u{2212} {low} = {}", dial.sub(high, low))
}

/// Trustee `x`'s own rule: "trustee 2: 2 = secret + 2 × spin".
fn own_rule(x: u32, share: u32) -> String {
    format!("trustee {x}: {share} = secret + {x} \u{d7} spin")
}

/// The spins' table: spin 0's secret, spin 1's worked out, the rest.
fn spin_rows(by_spin: &[u32], x: u32, share: u32) -> [String; 3] {
    let rest: Vec<String> = by_spin[2..].iter().map(ToString::to_string).collect();
    [
        format!("spin 0: secret {}", by_spin[0]),
        format!("spin 1: secret {share} \u{2212} {x} \u{d7} 1 = {}", by_spin[1]),
        format!("spins 2 to {}: secrets {}", by_spin.len() - 1, rest.join(", ")),
    ]
}

/// 08_8's precisely card: the chance of the share given each secret, one
/// spin in `marks` each, is the same for every secret. That comes from the
/// fair spin, not from assuming every secret equally likely, so whatever
/// anyone believed about the secret beforehand stands (Bayes: equal
/// likelihoods leave any prior as it was; Mail 6906).
fn card_text(share: u32, marks: u32) -> String {
    format!(
        "chance of share {share}, given secret s: 1/{marks} for every s (one spin each). \
         Equal likelihoods, not a uniform prior: any belief about s survives"
    )
}

/// The strip's caption for chapter 03's B and its share `b`.
fn for_b(b: u32) -> String {
    format!("for: B, holding {b} (chapter 03)")
}

/// The unwrapped mark nearest `u` that lands on `k`: where the needle turns
/// to, the short way.
fn straight_to(u: i64, k: u32, marks: u32) -> i64 {
    nearest_top(u - i64::from(k), marks) + i64::from(k)
}

/// A ring round position `t`'s disc under the board, in `color`. Detached.
fn marker_ring(stage: &mut Stage<'_>, t: u32, color: Srgb) -> Mob {
    ring(stage, marker_at(t), TRUSTEE_R + RING_OUT, color)
}

/// Where a step's "+4" sits: between position `x`'s and `x + 1`'s discs.
fn step_label_at(x: u32) -> Vec3 {
    v(PLOT_X0 + PLOT_DX * (f64::from(x) + 0.5), PLOT_Y0 - MARKER_DY)
}

/// The pieces of `pieces` growing one after another along the line, the
/// whole taking `rt`, prepared.
fn growing(stage: &mut Stage<'_>, pieces: &[Mob], rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let each = rt / pieces.len().max(1) as f64;
    let mut anims = Vec::with_capacity(pieces.len());
    for &p in pieces {
        anims.push(stage.prepare(show_creation(p).rt(each))?);
    }
    let s = Succession::new(stage.arena_mut(), anims)?;
    stage.prepare(s)
}

/// Rings at the board's edges where a wrapped line leaves and comes back.
fn wrap_rings(stage: &mut Stage<'_>, board: &Board, from: (f64, f64), to: (f64, f64), color: Srgb) -> Vec<Mob> {
    let pieces = wrapped(from, to, board.grid.marks);
    let mut rings = Vec::new();
    for w in pieces.windows(2) {
        for (x, k) in [w[0][1], w[1][0]] {
            rings.push(ring(stage, board.grid.at(x, k), 2.0 * BOARD_POINT_R, color));
        }
    }
    rings
}

impl SceneConstruct for TheLineOnTheDial {
    fn name(&self) -> &str {
        "08_the_line_on_the_dial"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let marks = dial.marks();
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        let heights: Vec<i64> = TRUSTEES.iter().map(|&t| line.height(t)).collect();
        let shares = line.deal(dial, &TRUSTEES);
        // the dial's wrap of each height is the recipe's share there
        for (&h, &s) in heights.iter().zip(&shares) {
            if turns_and_mark(h, marks).1 != s || h > TALLEST {
                return Err(fail(format!(
                    "height {h} wraps to {:?}, the share is {s}",
                    turns_and_mark(h, marks)
                )));
            }
        }
        // neighbours 2 and 3: their step, and two steps back to the secret
        let [low, high] = ADJACENT.map(|x| line.share(dial, x));
        let step = dial.sub(high, low);
        let back = chain(dial, low, step, ADJACENT[0]);
        if ADJACENT[1] != ADJACENT[0] + 1 || step != LINE_SLOPE || back.last() != Some(&SECRET) {
            return Err(fail(format!(
                "trustees {ADJACENT:?} hold {low} and {high}; back from {low} by {step}: {back:?}"
            )));
        }
        // trustee 2 alone: one secret for each spin, every mark once
        let held = line.share(dial, LINE_HOLDER);
        let by_spin = secrets_by_spin(dial, LINE_HOLDER, held);
        let alone = View::Shares {
            points: vec![(LINE_HOLDER, held)],
            spins: line.spins.len(),
        };
        let mut sorted = by_spin.clone();
        sorted.sort_unstable();
        let everyone: Vec<u32> = dial.all().collect();
        if sorted != everyone || alone.possible(dial) != everyone || !dial.shuffles(LINE_HOLDER) {
            return Err(fail(format!("trustee {LINE_HOLDER}'s {held} leaves {by_spin:?}")));
        }
        let index = |t: u32| (t - TRUSTEES[0]) as usize;

        // the opening: chapter 07's plot as it ended, heights in the cards,
        // and its ready dial growing into the wheel
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        let plot = Plot::build(&kit, stage)?;
        let top = (f64::from(LAST), line.height(LAST) as f64);
        let drawn = segment(stage, (0.0, f64::from(SECRET)), top, Palette::ink(), LINE_W);
        let dial_band = band(stage, 0, i64::from(marks - 1), palette::secret())?;
        let (mut points, mut labels, mut cards, mut markers) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (&t, &h) in TRUSTEES.iter().zip(&heights) {
            points.push(point(stage, f64::from(t), h as f64, palette::share()));
            labels.push(point_label(&kit, stage, f64::from(t), h, palette::share())?);
            cards.push(card(&kit, stage, t, &h.to_string(), palette::share())?);
            markers.push(Marker::build(&kit, stage, t, palette::share(), marker_at(t))?);
        }
        let zero = Marker::build(&kit, stage, 0, palette::secret(), marker_at(0))?;
        let name = secret_name(&kit, stage)?;
        let secret_point = point(stage, 0.0, f64::from(SECRET), palette::secret());
        rail.reveal(stage, 0.6)?;
        let mut opening: Vec<Mob> = plot.parts().to_vec();
        opening.extend([drawn, dial_band, zero.whole, name, secret_point]);
        opening.extend(&points);
        opening.extend(&labels);
        opening.extend(&cards);
        opening.extend(markers.iter().map(|m| m.whole));
        fade_in_all(stage, &opening, 0.8)?;
        let ready = Face::build(&kit, stage, dial, v(READY_X, READY_Y), READY_R, Palette::dim(), 0)?;
        let wheel = Face::build(&kit, stage, dial, v(WHEEL_X, WHEEL_Y), WHEEL_R, Palette::dim(), 0)?;
        ready.show(stage, 0.4)?;
        ready.zoom(stage, &wheel, 0.8, Vec::new())?;

        // 08_1: each height winds round the dial and lands on its share
        let spoken = say(&kit, stage, "08_1")?;
        let anims = recolorings(stage, &[(drawn, palette::ghost())], 0.5)?;
        stage.play_prepared(anims)?;
        let mut works = Vec::with_capacity(TRUSTEES.len());
        for (i, (&t, &h)) in TRUSTEES.iter().zip(&heights).enumerate() {
            spoken.wait_for(stage, WIND_CUES[i])?;
            let (turns, k) = turns_and_mark(h, marks);
            let start = nearest_top(wheel.at(stage), marks);
            if start != wheel.at(stage) {
                wheel.turn(stage, start, 0.25)?;
            }
            // thirteen lands as "lands on two" is said; the rest keep up
            // with the connected list
            let rt = match i {
                0 => 0.9,
                1 => (spoken.at("lands on two")? - now(stage)).clamp(0.8, 2.0),
                _ => 0.8,
            };
            let wound = coil(stage, &wheel, marks, h as u32, palette::share());
            let anims = vec![
                stage.prepare(show_creation(wound).rt(rt))?,
                stage.prepare(tween(wheel.needle, (start + h) as f64, rt)?)?,
            ];
            stage.play_prepared(anims)?;
            let landed = wheel.halo(stage, k, palette::share());
            let work = kit.text(
                stage,
                &taken_off(h, marks),
                WORK_SIZE,
                Palette::ink(),
                v(WORK_X, WORK_Y - WORK_DY * i as f64),
            )?;
            works.push(work);
            let mut anims = glowing(stage, &[landed], 0.7)?;
            anims.extend(swaps(stage, &[], &[work], 0.4)?);
            anims.extend(leaving(stage, &[wound], ORIGIN, 0.6)?);
            if turns == 0 {
                // nine stays nine: its label and card are pointed at, and
                // nothing moves
                let mut halos = vec![text_halo(stage, labels[i], palette::share())?];
                halos.extend(frame_halos(stage, &[t], palette::share())?);
                anims.extend(glowing(stage, &halos, 0.7)?);
            } else {
                // the point drops down its column onto its mark; the old
                // label goes and the new one comes in by the point; the
                // card is emptied first, then the share drops in
                anims.push(moving(stage, points[i], at(f64::from(t), f64::from(k)), 0.6)?);
                let new_label = point_label(&kit, stage, f64::from(t), i64::from(k), palette::share())?;
                anims.extend(swaps(stage, &[labels[i]], &[new_label], 0.5)?);
                labels[i] = new_label;
                let new_card = card(&kit, stage, t, &k.to_string(), palette::share())?;
                let a = fade_out(stage.arena_mut(), cards[i], v(0.0, DROP), 1.0)?;
                let out = stage.prepare(a.rt(0.25))?;
                let a = fade_in(stage.arena_mut(), new_card, v(0.0, -DROP), 1.0)?;
                let inn = stage.prepare(a.rt(0.3))?;
                let both = Succession::new(stage.arena_mut(), vec![out, inn])?;
                anims.push(stage.prepare(both)?);
                cards[i] = new_card;
            }
            stage.play_prepared(anims)?;
        }
        spoken.wait_end(stage)?;
        pause(stage, 0.6)?;

        // 08_2: on the board; the same rule, wrapped
        let spoken = say(&kit, stage, "08_2")?;
        let board = Board::build(&kit, stage, LINE_BOARD, BOARD_TITLE, false)?;
        let mut going: Vec<Mob> = plot.parts().to_vec();
        going.extend([drawn, dial_band]);
        going.extend(&labels);
        let mut anims = leaving(stage, &going, ORIGIN, 0.6)?;
        for (i, (&t, &s)) in TRUSTEES.iter().zip(&shares).enumerate() {
            anims.push(moving(
                stage,
                points[i],
                board.grid.at(f64::from(t), f64::from(s)),
                0.9,
            )?);
        }
        anims.push(moving(stage, secret_point, board.grid.at(0.0, f64::from(SECRET)), 0.9)?);
        anims.extend(swaps(stage, &[], board.parts(), 0.9)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "but the rule is the same")?;
        let whole = board.line(
            stage,
            (0.0, f64::from(SECRET)),
            (f64::from(LAST), line.height(LAST) as f64),
            Palette::dim(),
            WRAPPED_W,
        );
        create_lagged(stage, &whole, 1.6, 0.9)?;
        let mut halos: Vec<Mob> = TRUSTEES
            .iter()
            .zip(&shares)
            .map(|(&t, &s)| {
                ring(
                    stage,
                    board.grid.at(f64::from(t), f64::from(s)),
                    2.0 * BOARD_POINT_R,
                    palette::share(),
                )
            })
            .collect();
        halos.push(ring(
            stage,
            board.grid.at(0.0, f64::from(SECRET)),
            2.0 * BOARD_POINT_R,
            palette::secret(),
        ));
        glow(stage, &halos, 0.8)?;
        spoken.wait_for(stage, "Start at the secret")?;
        let at_secret = straight_to(wheel.at(stage), SECRET, marks);
        let mut anims = vec![swelling(stage, secret_point, 1.8, palette::secret(), 0.7)?];
        anims.push(stage.prepare(tween(wheel.needle, at_secret as f64, 0.6)?)?);
        let r = ring(
            stage,
            board.grid.at(0.0, f64::from(SECRET)),
            2.0 * BOARD_POINT_R,
            palette::secret(),
        );
        anims.extend(glowing(stage, &[r], 0.7)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "turn on four for each step")?;
        // each step from a share to the next: on 4, the needle with it; a
        // step that runs off the top comes back in at the bottom
        let (mut steps, mut step_labels) = (Vec::new(), Vec::new());
        let mut u = at_secret;
        for x in 0..LAST {
            let s = f64::from(line.share(dial, x));
            let (from, to) = ((f64::from(x), s), (f64::from(x + 1), s + f64::from(LINE_SLOPE)));
            let pieces = board.line(stage, from, to, palette::spin(), STEP_W);
            let label = kit.text(
                stage,
                &format!("+{LINE_SLOPE}"),
                STEP_LABEL_SIZE,
                palette::spin(),
                step_label_at(x),
            )?;
            u += i64::from(LINE_SLOPE);
            let mut anims = vec![growing(stage, &pieces, 0.6)?];
            anims.push(stage.prepare(tween(wheel.needle, u as f64, 0.6)?)?);
            anims.extend(swaps(stage, &[], &[label], 0.4)?);
            let rings = wrap_rings(stage, &board, from, to, palette::spin());
            anims.extend(glowing(stage, &rings, 0.9)?);
            stage.play_prepared(anims)?;
            steps.extend(pieces);
            step_labels.push(label);
        }
        if dial.wrap(u) != line.share(dial, LAST) {
            return Err(fail(format!("the needle stepped to {u}, not the last share")));
        }
        spoken.wait_end(stage)?;

        // 08_3: neighbours 2 and 3, one step apart, differ by 4
        let spoken = say(&kit, stage, "08_3")?;
        let mut going = steps;
        going.extend(step_labels);
        going.extend(whole);
        going.extend(&works);
        let mut anims = leaving(stage, &going, ORIGIN, 0.5)?;
        let mut ghosts: Vec<(Mob, Srgb)> = TRUSTEES
            .iter()
            .filter(|t| !ADJACENT.contains(t))
            .map(|&t| (points[index(t)], palette::ghost()))
            .collect();
        ghosts.push((secret_point, palette::ghost()));
        anims.extend(recolorings(stage, &ghosts, 0.5)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Take trustees two and three")?;
        let mut halos: Vec<Mob> = ADJACENT
            .iter()
            .map(|&t| {
                ring(
                    stage,
                    board.grid.at(f64::from(t), f64::from(line.share(dial, t))),
                    2.0 * BOARD_POINT_R,
                    palette::share(),
                )
            })
            .collect();
        halos.extend(ADJACENT.iter().map(|&t| marker_ring(stage, t, palette::share())));
        halos.extend(frame_halos(stage, &ADJACENT, palette::share())?);
        glow(stage, &halos, 0.8)?;
        spoken.wait_for(stage, "holding two and six")?;
        let mut halos = Vec::new();
        for k in [low, high] {
            let label = board
                .row_label(k)
                .ok_or_else(|| fail(format!("the board has no row {k}")))?;
            halos.push(text_halo(stage, label, palette::share())?);
        }
        glow(stage, &halos, 0.8)?;
        spoken.wait_for(stage, "one step apart")?;
        let [px, qx] = ADJACENT.map(f64::from);
        let run = stroke_on(
            stage,
            &board,
            (px, f64::from(low)),
            (qx, f64::from(low)),
            Palette::dim(),
            RUN_W,
        );
        let a = stage.prepare(show_creation(run).rt(0.6))?;
        stage.play_prepared(vec![a])?;
        spoken.wait_for(stage, "differ by four")?;
        let rise = stroke_on(
            stage,
            &board,
            (qx, f64::from(low)),
            (qx, f64::from(high)),
            palette::spin(),
            STEP_W,
        );
        let mid = board.grid.at(qx, (f64::from(low) + f64::from(high)) / 2.0);
        let rise_label = kit.text_left(
            stage,
            &rise_text(dial, low, high),
            RISE_LABEL_SIZE,
            palette::spin(),
            v(mid[0] + RISE_LABEL_DX, mid[1]),
        )?;
        let mut anims = vec![stage.prepare(show_creation(rise).rt(0.6))?];
        anims.extend(swaps(stage, &[], &[rise_label], 0.5)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "each step adds four")?;
        let from_low = straight_to(wheel.at(stage), low, marks);
        wheel.turn(stage, from_low, 0.4)?;
        let landed = wheel.click(stage, i64::from(step), 0.3, None)?;
        if landed.last() != Some(&high) {
            return Err(fail(format!("on from {low} by {step} the needle landed on {landed:?}")));
        }
        let on = wheel.arc(
            &kit,
            stage,
            from_low as f64,
            (from_low + i64::from(step)) as f64,
            &format!("+{step}"),
            palette::spin(),
        )?;
        fade_in_all(stage, &on, 0.4)?;
        spoken.wait_end(stage)?;

        // 08_4: two steps back from trustee 2: 9, then the secret
        let spoken = say(&kit, stage, "08_4")?;
        let mut going = vec![run, rise, rise_label];
        going.extend(on);
        let mut anims = leaving(stage, &going, ORIGIN, 0.4)?;
        let to_low = straight_to(wheel.at(stage), low, marks);
        anims.push(stage.prepare(tween(wheel.needle, to_low as f64, 0.5)?)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Step back twice")?;
        for t in (0..ADJACENT[0]).rev() {
            let r = marker_ring(stage, t, palette::secret());
            glow(stage, &[r], 0.45)?;
        }
        let mut found = Vec::new();
        for (j, (cue, w)) in ["Two take away four is nine", "nine take away four is five"]
            .iter()
            .zip(back.windows(2))
            .enumerate()
        {
            spoken.wait_for(stage, cue)?;
            let (from_mark, to_mark) = (w[0], w[1]);
            let x = f64::from(ADJACENT[0]) - j as f64;
            let u0 = wheel.at(stage);
            let landed = wheel.click(stage, -i64::from(step), 0.25, None)?;
            if landed.last() != Some(&to_mark) {
                return Err(fail(format!(
                    "back from {from_mark} by {step} the needle landed on {landed:?}"
                )));
            }
            // the same step on the board: one position left, 4 down,
            // wrapping off the bottom and in at the top
            let (from, to) = (
                (x, f64::from(from_mark)),
                (x - 1.0, f64::from(from_mark) - f64::from(step)),
            );
            let pieces = board.line(stage, from, to, palette::spin(), STEP_W);
            let arc = wheel.arc(
                &kit,
                stage,
                (u0 - i64::from(step)) as f64,
                u0 as f64,
                &format!("\u{2212}{step}"),
                palette::spin(),
            )?;
            let text = kit.text(
                stage,
                &back_text(dial, from_mark, step),
                WORK_SIZE,
                Palette::ink(),
                v(WORK_X, WORK_Y - WORK_DY * j as f64),
            )?;
            let mut anims = vec![growing(stage, &pieces, 0.7)?];
            anims.extend(swaps(stage, &[], &[arc[0], arc[1], text], 0.5)?);
            let rings = wrap_rings(stage, &board, from, to, palette::spin());
            anims.extend(glowing(stage, &rings, 0.9)?);
            stage.play_prepared(anims)?;
            found.extend(pieces);
            found.extend(arc);
            found.push(text);
        }
        spoken.wait_for(stage, "the secret")?;
        let secret_again = board.point(stage, 0, SECRET, palette::secret());
        fade_in_all(stage, &[secret_again], 0.3)?;
        let five = board
            .row_label(SECRET)
            .ok_or_else(|| fail(format!("the board has no row {SECRET}")))?;
        let mut anims = vec![swelling(stage, secret_again, 1.8, palette::secret(), 0.8)?];
        let halos = [
            text_halo(stage, five, palette::secret())?,
            wheel.halo(stage, SECRET, palette::secret()),
            marker_ring(stage, 0, palette::secret()),
        ];
        anims.extend(glowing(stage, &halos, 0.9)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_ON_SECRET)?;

        // 08_5: trustee 2 alone; how many secrets fit its 2?
        let spoken = say(&kit, stage, "08_5")?;
        let mut going = found;
        going.extend(wheel.parts());
        going.push(secret_again);
        going.push(secret_point);
        let others: Vec<u32> = TRUSTEES.iter().copied().filter(|&t| t != LINE_HOLDER).collect();
        for &t in &others {
            going.push(points[index(t)]);
            going.push(cards[index(t)]);
        }
        let mut anims = leaving(stage, &going, ORIGIN, 0.5)?;
        let mut dimmed = rail.tint(&others, palette::ghost());
        dimmed.extend(others.iter().map(|&t| (markers[index(t)].disc, palette::ghost())));
        dimmed.push((zero.disc, palette::ghost()));
        anims.extend(recolorings(stage, &dimmed, 0.5)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "trustee two is alone")?;
        let mut halos = vec![
            ring(
                stage,
                board.grid.at(f64::from(LINE_HOLDER), f64::from(held)),
                2.0 * BOARD_POINT_R,
                palette::share(),
            ),
            marker_ring(stage, LINE_HOLDER, palette::share()),
        ];
        halos.extend(frame_halos(stage, &[LINE_HOLDER], palette::share())?);
        glow(stage, &halos, 0.8)?;
        spoken.wait_for(stage, "holding two")?;
        let held_label = board
            .row_label(held)
            .ok_or_else(|| fail(format!("the board has no row {held}")))?;
        let halo = text_halo(stage, held_label, palette::share())?;
        glow(stage, &[halo], 0.7)?;
        spoken.wait_for(stage, "the secret plus twice the spin")?;
        let rule = kit.text(stage, RULE, RULE_SIZE, Palette::ink(), v(RULE_X, RULE_Y))?;
        let own = kit.text(
            stage,
            &own_rule(LINE_HOLDER, held),
            OWN_SIZE,
            palette::share(),
            v(RULE_X, OWN_Y),
        )?;
        fade_in_all(stage, &[rule, own], 0.5)?;
        spoken.wait_end(stage)?;
        // nothing counts yet: no strip, no lines, until the answer
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 08_6: all eleven, one for each spin
        let spoken = say(&kit, stage, "08_6")?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "All eleven")?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        strip.reveal(&kit, stage, 1.2)?;
        strip.show(&kit, stage, &alone, FOR_TWO, 0.4)?;
        check_strip(&strip, &by_spin_sorted(&by_spin), "for trustee 2 alone")?;
        let rows = spin_rows(&by_spin, LINE_HOLDER, held);
        let mut table = Vec::with_capacity(rows.len());
        for (row, &y) in rows.iter().zip(&TABLE_Y) {
            table.push(kit.text_left(stage, row, TABLE_SIZE, Palette::ink(), v(STRIP_LEFT, y))?);
        }
        // each spin's wrapped line from (2, 2) back to position 0, where it
        // meets its own secret, a gold point there and a dot over its tick
        let mut spun = Vec::with_capacity(by_spin.len());
        for (a, &s) in (0u32..).zip(&by_spin) {
            let from = (f64::from(LINE_HOLDER), f64::from(held));
            let to = (0.0, f64::from(held) - f64::from(LINE_HOLDER * a));
            let pieces = board.line(stage, from, to, Palette::dim(), FAN_W);
            let hit = board.point(stage, 0, s, palette::secret());
            let t = tick_at(s);
            let dot = disc(
                stage,
                palette::share(),
                v(t[0], STRIP_Y + TICK_H / 2.0 + DOT_GAP),
                DOT_R,
            );
            spun.push((pieces, hit, dot));
        }
        for (a, cue) in ["Spin zero means the secret is two", "Spin one means"]
            .iter()
            .enumerate()
        {
            spoken.wait_for(stage, cue)?;
            let (pieces, hit, dot) = &spun[a];
            let mut anims = vec![growing(stage, pieces, 0.8)?];
            anims.extend(swaps(stage, &[], &[*hit, *dot, table[a]], 0.5)?);
            stage.play_prepared(anims)?;
        }
        spoken.wait_end(stage)?;
        // the other nine, one after another, then position 0's column: each
        // mark met once
        let mut rest = Vec::new();
        for (pieces, hit, dot) in &spun[2..] {
            let mut each = vec![growing(stage, pieces, 0.6)?];
            each.extend(swaps(stage, &[], &[*hit, *dot], 0.4)?);
            let g = AnimationGroup::new(stage.arena_mut(), each)?;
            rest.push(stage.prepare(g)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), rest, 0.35)?.with_run_time(2.6);
        stage.play(g)?;
        fade_in_all(stage, &table[2..], 0.4)?;
        let column = column_outline(stage, &board, palette::secret())?;
        glow(stage, &[column], 1.0)?;

        // 08_7: doubling visits every mark once
        let spoken = say(&kit, stage, "08_7")?;
        let anims = leaving(stage, &table, ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Doubling is the reason")?;
        let halo = text_halo(stage, own, palette::share())?;
        glow(stage, &[halo], 0.8)?;
        spoken.wait_for(stage, "the doubles of zero to ten")?;
        let arrows = doubling(dial, LINE_HOLDER);
        let (mut spin_row, mut double_row) = (Vec::new(), Vec::new());
        for k in dial.all() {
            let x = tick_at(k)[0];
            spin_row.push(kit.text(stage, &k.to_string(), ROW_SIZE, palette::spin(), v(x, SPIN_ROW_Y))?);
            double_row.push(kit.text(stage, &k.to_string(), ROW_SIZE, Palette::ink(), v(x, DOUBLE_ROW_Y))?);
        }
        let title_x = STRIP_LEFT - ROW_TITLE_GAP;
        let spin_title = kit.text(
            stage,
            SPIN_TITLE,
            ROW_TITLE_SIZE,
            palette::spin(),
            v(title_x, SPIN_ROW_Y),
        )?;
        stage.move_to(spin_title, v(title_x, SPIN_ROW_Y), RIGHT);
        let double_title = kit.text(
            stage,
            DOUBLE_TITLE,
            ROW_TITLE_SIZE,
            Palette::dim(),
            v(title_x, DOUBLE_ROW_Y),
        )?;
        stage.move_to(double_title, v(title_x, DOUBLE_ROW_Y), RIGHT);
        let mut rows_in = spin_row.clone();
        rows_in.extend(&double_row);
        rows_in.extend([spin_title, double_title]);
        fade_in_all(stage, &rows_in, 0.5)?;
        let mut drawn_arrows = Vec::with_capacity(arrows.len());
        let mut hits = Vec::with_capacity(arrows.len());
        for &(a, d) in &arrows {
            let start = v(tick_at(a)[0], SPIN_ROW_Y - ARROW_GAP);
            let end = v(tick_at(d)[0], DOUBLE_ROW_Y + ARROW_GAP);
            let arrow = Arrow::new(start, end)
                .buff(0.0)
                .thickness(ARROW_W)
                .color(palette::spin())
                .build()?;
            drawn_arrows.push(stage.arena_mut().add(arrow));
            hits.push(ring(stage, v(tick_at(d)[0], DOUBLE_ROW_Y), HIT_R, palette::spin()));
        }
        // the doubles below 11 as they are said, then past the top
        let wrap_from = arrows
            .iter()
            .position(|&(a, d)| d < a)
            .ok_or_else(|| fail("doubling never runs past the top".into()))?;
        spoken.wait_for(stage, "are zero, two, four")?;
        let rt = (spoken.at("then one")? - now(stage)).clamp(1.2, 3.0);
        lagged_in(stage, &drawn_arrows[..wrap_from], &hits[..wrap_from], rt)?;
        spoken.wait_for(stage, "then one")?;
        let note = kit.text_left(
            stage,
            PAST_TOP,
            NOTE_SIZE,
            Palette::dim(),
            v(STRIP_LEFT, DOUBLE_ROW_Y - NOTE_DY),
        )?;
        fade_in_all(stage, &[note], 0.3)?;
        let rt = (spoken.at("Every mark, once")? - now(stage)).clamp(1.2, 3.0);
        lagged_in(stage, &drawn_arrows[wrap_from..], &hits[wrap_from..], rt)?;
        spoken.wait_for(stage, "Every mark, once")?;
        let row_outline = outline(
            stage,
            v((tick_at(0)[0] + tick_at(marks - 1)[0]) / 2.0, DOUBLE_ROW_Y),
            tick_at(marks - 1)[0] - tick_at(0)[0] + 2.0 * HIT_R + 2.0 * PAD,
            2.0 * HIT_R + 2.0 * PAD,
            palette::spin(),
        )?;
        glow(stage, &[row_outline], 1.0)?;
        spoken.wait_end(stage)?;

        // 08_8: equally likely under every secret, as B's 7 was
        let spoken = say(&kit, stage, "08_8")?;
        spoken.wait_for(stage, "each spin gives its own secret")?;
        let column = column_outline(stage, &board, palette::secret())?;
        glow(stage, &[column], 0.8)?;
        spoken.wait_for(stage, "fits every secret equally well")?;
        let text = card_text(held, marks);
        let exact = precisely(&kit, stage, &text)?;
        let anims = swaps(stage, &[], &[exact], 0.5)?;
        stage.play_prepared(anims)?;
        let read = read_by(stage, &text);
        spoken.wait_for(stage, "just like B's seven")?;
        let b = b_share(dial);
        strip.show(&kit, stage, &dial_view(b), &for_b(b), 0.5)?;
        check_strip(&strip, &dial_secrets(dial, b), "for B's 7")?;
        spoken.wait_end(stage)?;
        wait_until(stage, read)?;
        pause(stage, HOLD_AT_END)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

/// B's share on the dial in chapter 03: the turn from A's spin to the
/// secret.
fn b_share(dial: Dial) -> u32 {
    dial.sub(SECRET, A_SHARE)
}

/// The secrets, sorted, as the strip lists them.
fn by_spin_sorted(by_spin: &[u32]) -> Vec<u32> {
    let mut s = by_spin.to_vec();
    s.sort_unstable();
    s
}

/// A straight stroke on `board` from `from` to `to` (position, row), in
/// `color`, `w` wide; it grows from `from`. Detached.
fn stroke_on(stage: &mut Stage<'_>, board: &Board, from: (f64, f64), to: (f64, f64), color: Srgb, w: f64) -> Mob {
    super::draw::stroke(
        stage,
        vec![board.grid.at(from.0, from.1), board.grid.at(to.0, to.1)],
        color,
        w,
    )
}

/// An outline round position 0's column, marks 0 to the top. Detached.
fn column_outline(stage: &mut Stage<'_>, board: &Board, color: Srgb) -> fmn::Result<Mob> {
    let g = board.grid;
    let mid = g.at(0.0, (f64::from(g.marks) - 1.0) / 2.0);
    outline(
        stage,
        mid,
        4.0 * BOARD_POINT_R + 2.0 * PAD,
        g.top() - g.bottom() + 2.0 * PAD,
        color,
    )
}

/// The arrows coming in one after another, each with its landing ring, the
/// whole taking `rt`; the rings stay.
fn lagged_in(stage: &mut Stage<'_>, arrows: &[Mob], rings: &[Mob], rt: f64) -> fmn::Result<()> {
    if arrows.is_empty() {
        return Ok(());
    }
    let mut each = Vec::with_capacity(arrows.len());
    for (&a, &r) in arrows.iter().zip(rings) {
        let both = swaps(stage, &[], &[a, r], 1.0)?;
        let g = AnimationGroup::new(stage.arena_mut(), both)?;
        each.push(stage.prepare(g)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), each, 0.5)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::board::coil_radius;
    use super::super::draw::{STRIP_CAPTION_Y, STRIP_TITLE_Y, THINK_R, strip_right};
    use super::super::plot::{POINT_LABEL_SIZE, axis_end, label_at};
    use super::super::two_keys::ARC_LABEL_IN;
    use super::*;
    use crate::facts;
    use crate::model::{MARKS, csv};

    /// The production narration: registering chapter 08 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// Every phrase the scene waits for is in its line of the script.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        let mut phrases: Vec<(&str, &str)> = WIND_CUES.iter().map(|&p| ("08_1", p)).collect();
        phrases.extend([
            ("08_1", "lands on two"),
            ("08_2", "but the rule is the same"),
            ("08_2", "Start at the secret"),
            ("08_2", "turn on four for each step"),
            ("08_3", "Take trustees two and three"),
            ("08_3", "holding two and six"),
            ("08_3", "one step apart"),
            ("08_3", "differ by four"),
            ("08_3", "each step adds four"),
            ("08_4", "Step back twice"),
            ("08_4", "Two take away four is nine"),
            ("08_4", "nine take away four is five"),
            ("08_4", "the secret"),
            ("08_5", "trustee two is alone"),
            ("08_5", "holding two"),
            ("08_5", "the secret plus twice the spin"),
            ("08_6", "All eleven"),
            ("08_6", "Spin zero means the secret is two"),
            ("08_6", "Spin one means"),
            ("08_7", "Doubling is the reason"),
            ("08_7", "the doubles of zero to ten"),
            ("08_7", "are zero, two, four"),
            ("08_7", "then one"),
            ("08_7", "Every mark, once"),
            ("08_8", "each spin gives its own secret"),
            ("08_8", "fits every secret equally well"),
            ("08_8", "just like B's seven"),
        ]);
        for (id, phrase) in phrases {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
            // a cue that occurs twice would land on the first
            assert_eq!(text.matches(phrase).count(), 1, "{id}: {phrase:?} twice");
        }
    }

    /// What the scene draws and writes is the facts': each height's turns
    /// and landing, the neighbours' step, the two steps back, trustee 2's
    /// share and the secret for every spin, the doubles, and B's 7.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        for &t in &TRUSTEES {
            let h = line.height(t);
            let (turns, k) = turns_and_mark(h, MARKS);
            assert_eq!(k, line.share(dial, t));
            assert_eq!(
                f.get(&format!("S-F4.wrap.{t}")),
                Some(format!("{h},{turns},{k}").as_str())
            );
        }
        assert_eq!(taken_off(13, MARKS), "13 \u{2212} 11 = 2");
        let [low, high] = ADJACENT.map(|x| line.share(dial, x));
        assert_eq!(f.get("S-F4.adjacent.shares"), Some(csv(&[low, high]).as_str()));
        assert_eq!(rise_text(dial, low, high), "6 \u{2212} 2 = 4");
        let back = chain(dial, low, dial.sub(high, low), ADJACENT[0]);
        assert_eq!(back, vec![2, 9, 5]);
        assert_eq!(
            f.get("S-F4.back.1"),
            Some(csv(&[back[0], LINE_SLOPE, back[1]]).as_str())
        );
        assert_eq!(
            f.get("S-F4.back.2"),
            Some(csv(&[back[1], LINE_SLOPE, back[2]]).as_str())
        );
        assert_eq!(f.get("S-F4.back.steps"), Some(ADJACENT[0].to_string().as_str()));
        assert_eq!(back_text(dial, 2, 4), "2 \u{2212} 4 \u{2192} 9");
        assert_eq!(back_text(dial, 9, 4), "9 \u{2212} 4 \u{2192} 5");
        let held = line.share(dial, LINE_HOLDER);
        assert_eq!(f.get("S-F5.share"), Some(held.to_string().as_str()));
        assert_eq!(own_rule(LINE_HOLDER, held), "trustee 2: 2 = secret + 2 \u{d7} spin");
        let by_spin = secrets_by_spin(dial, LINE_HOLDER, held);
        assert_eq!(f.get("S-F5.secret_by_slope"), Some(csv(&by_spin).as_str()));
        assert_eq!(f.get("S-F5.spin.0"), Some(by_spin[0].to_string().as_str()));
        assert_eq!(f.get("S-F5.spin.1"), Some(by_spin[1].to_string().as_str()));
        assert_eq!(
            spin_rows(&by_spin, LINE_HOLDER, held),
            [
                "spin 0: secret 2".to_owned(),
                "spin 1: secret 2 \u{2212} 2 \u{d7} 1 = 0".to_owned(),
                "spins 2 to 10: secrets 9, 7, 5, 3, 1, 10, 8, 6, 4".to_owned(),
            ]
        );
        let doubles: Vec<u32> = doubling(dial, LINE_HOLDER).iter().map(|p| p.1).collect();
        assert_eq!(f.get("S-F4.times2"), Some(csv(&doubles).as_str()));
        assert_eq!(f.get("S-F2.b_share"), Some(b_share(dial).to_string().as_str()));
        assert_eq!(for_b(b_share(dial)), "for: B, holding 7 (chapter 03)");
        assert_eq!(dial_secrets(dial, b_share(dial)).len(), 11);
    }

    /// Two steps back from trustee 2, by the step its neighbour shows, land
    /// on the secret, and each step drawn on the board ends on the mark the
    /// needle lands on. Planted negatives: one step too few or too many,
    /// or the wrong step, miss it.
    #[test]
    fn two_steps_back_find_the_secret() {
        let dial = Dial::vault();
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        let [low, high] = ADJACENT.map(|x| line.share(dial, x));
        let step = dial.sub(high, low);
        let back = chain(dial, low, step, ADJACENT[0]);
        assert_eq!(back.last(), Some(&SECRET));
        for (j, w) in back.windows(2).enumerate() {
            let x = f64::from(ADJACENT[0]) - j as f64;
            let pieces = wrapped(
                (x, f64::from(w[0])),
                (x - 1.0, f64::from(w[0]) - f64::from(step)),
                MARKS,
            );
            let end = pieces.last().unwrap()[1];
            assert!((end.0 - (x - 1.0)).abs() < 1e-9 && (end.1 - f64::from(w[1])).abs() < 1e-9);
        }
        // the first step back runs off the bottom: two pieces
        assert_eq!(wrapped((2.0, 2.0), (1.0, -2.0), MARKS).len(), 2);
        assert_ne!(chain(dial, low, step, 1).last(), Some(&SECRET));
        assert_ne!(chain(dial, low, step, 3).last(), Some(&SECRET));
        assert_ne!(chain(dial, low, step - 1, 2).last(), Some(&SECRET));
        // on ordinary paper the same steps leave the dial: 2 − 4 − 4 = −6
        assert_eq!(i64::from(low) - 2 * i64::from(step), -6);
    }

    /// Trustee 2's one share fits every secret, one spin each: each spin's
    /// wrapped line from (2, 2) meets position 0 on its own mark, and the
    /// marks met are all eleven. The planted negative: on a ten-mark dial
    /// the same share fits only five secrets, since doubling there lands on
    /// even marks only.
    #[test]
    fn one_share_fits_every_secret_once() {
        let dial = Dial::vault();
        let by_spin = secrets_by_spin(dial, LINE_HOLDER, 2);
        for (a, &s) in (0u32..).zip(&by_spin) {
            let pieces = wrapped((2.0, 2.0), (0.0, 2.0 - 2.0 * f64::from(a)), MARKS);
            let end = pieces.last().unwrap()[1];
            assert!(end.0.abs() < 1e-9 && (end.1 - f64::from(s)).abs() < 1e-9, "spin {a}");
            assert!(Recipe::new(s, &[a]).deals(dial, &[(LINE_HOLDER, 2)]));
        }
        assert_eq!(by_spin_sorted(&by_spin), dial.all().collect::<Vec<_>>());
        let ten = Dial::new(10);
        let mut tens = secrets_by_spin(ten, LINE_HOLDER, 2);
        tens.sort_unstable();
        tens.dedup();
        assert_eq!(tens, vec![0, 2, 4, 6, 8]);
    }

    /// Doubling sends every mark to a different mark, so every mark is hit
    /// once, and the first double past the top is 6's; on ten marks it
    /// hits only the even ones (the negative control).
    #[test]
    fn doubling_visits_every_mark_once() {
        let dial = Dial::vault();
        let arrows = doubling(dial, LINE_HOLDER);
        let mut hit: Vec<u32> = arrows.iter().map(|p| p.1).collect();
        hit.sort_unstable();
        assert_eq!(hit, dial.all().collect::<Vec<_>>());
        assert_eq!(arrows.iter().position(|&(a, d)| d < a), Some(6));
        assert!(dial.shuffles(LINE_HOLDER));
        let ten = Dial::new(10);
        assert!(!ten.shuffles(LINE_HOLDER));
        let mut tens: Vec<u32> = doubling(ten, LINE_HOLDER).iter().map(|p| p.1).collect();
        tens.sort_unstable();
        tens.dedup();
        assert_eq!(tens.len(), 5);
    }

    /// The precisely card stays two lines at every wrap the lower third
    /// can narrow to.
    #[test]
    fn the_card_is_two_lines() {
        let text = card_text(2, MARKS);
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
    }

    /// The wheel, its coils and arcs, the wrap texts, the rule, the think
    /// ring, the spins' table, the doubling rows and the strip's dots keep
    /// apart, inside the frame, clear of the plot, the board, the rail and
    /// the lower third.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use super::super::draw::{FRAME_H, FRAME_Y, TRUSTEE_R as DISC_R, trustee_at};
        let edge = FRAME_X_RADIUS - 0.25;
        // the ready dial (as chapter 07 left it) and the wheel, right of
        // the plot's position title; the longest coil inside the arcs'
        // labels and the frame
        let plot_right = axis_end() + 0.11 + text_w("position", 18.0);
        let reach = WHEEL_R * coil_radius(25.0, MARKS).max(ARC_LABEL_IN);
        assert!(WHEEL_X - reach > plot_right + 0.3, "{plot_right}");
        assert!(WHEEL_Y + reach < RULE_Y - text_h(RULE_SIZE));
        const {
            assert!(
                READY_Y - READY_R > PLOT_Y0 + 0.3,
                "the ready dial over the position title"
            )
        };
        assert!(READY_X - READY_R > at(f64::from(LAST), 0.0)[0] + 0.3);
        // the work texts under the wheel, over the bottom of the frame and
        // right of the board
        let lowest = WORK_Y - WORK_DY * 4.0;
        assert!(WHEEL_Y - reach > WORK_Y + text_h(WORK_SIZE) / 2.0 + 0.1);
        assert!(lowest - text_h(WORK_SIZE) / 2.0 > -FRAME_Y_RADIUS + 0.25);
        let widest = text_w(&taken_off(25, MARKS), WORK_SIZE);
        assert!(WORK_X - widest / 2.0 > LINE_BOARD.right() + 0.3);
        assert!(WORK_X + widest / 2.0 < edge);
        // the board between the position discs and the top, its title
        // inside; the old plot's tallest label under the frame's top
        let g = LINE_BOARD;
        assert!(g.bottom() > PLOT_Y0 - MARKER_DY + DISC_R + 0.05);
        assert!(g.title_at()[1] + text_h(20.0) / 2.0 < FRAME_Y_RADIUS - 0.25);
        assert!(label_at(5.0, 25.0)[1] + text_h(POINT_LABEL_SIZE) / 2.0 < FRAME_Y_RADIUS - 0.25);
        // the rule over the strip's title, inside the edges
        for (text, size, y) in [(RULE.to_owned(), RULE_SIZE, RULE_Y), (own_rule(2, 2), OWN_SIZE, OWN_Y)] {
            assert!(RULE_X + text_w(&text, size) / 2.0 < edge, "{text}");
            assert!(RULE_X - text_w(&text, size) / 2.0 > g.right() + 0.3, "{text}");
            assert!(y - text_h(size) / 2.0 > STRIP_TITLE_Y + 0.3, "{text}");
        }
        assert!(RULE_Y + text_h(RULE_SIZE) / 2.0 < FRAME_Y_RADIUS - 0.25);
        // the strip's dots over the ticks, under the title
        let dot_top = STRIP_Y + TICK_H / 2.0 + DOT_GAP + DOT_R;
        assert!(STRIP_Y + TICK_H / 2.0 < dot_top - 2.0 * DOT_R);
        assert!(dot_top < STRIP_TITLE_Y - 0.15);
        // the think ring, the table and the doubling rows under the
        // caption, inside the edge; the rows clear of the lower third
        // (two lines: its top at −2.78)
        const { assert!(THINK_Y + THINK_R < STRIP_CAPTION_Y - 0.2) };
        assert!(THINK_X + THINK_R < edge);
        let by_spin = secrets_by_spin(Dial::vault(), 2, 2);
        for (row, &y) in spin_rows(&by_spin, 2, 2).iter().zip(&TABLE_Y) {
            assert!(STRIP_LEFT + text_w(row, TABLE_SIZE) < edge, "{row}");
            assert!(y + text_h(TABLE_SIZE) / 2.0 < STRIP_CAPTION_Y - 0.2);
        }
        let lower_third_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        assert!(SPIN_ROW_Y + text_h(ROW_SIZE) / 2.0 < STRIP_CAPTION_Y - 0.3);
        assert!(DOUBLE_ROW_Y - HIT_R > lower_third_top + 0.2);
        assert!(DOUBLE_ROW_Y - NOTE_DY - text_h(NOTE_SIZE) / 2.0 > lower_third_top + 0.05);
        const { assert!(SPIN_ROW_Y - ARROW_GAP > DOUBLE_ROW_Y + ARROW_GAP + 0.5) };
        assert!(STRIP_LEFT + text_w(PAST_TOP, NOTE_SIZE) < edge);
        // the rows' titles right of the board and of trustee 5's disc and
        // frame
        let title_x = STRIP_LEFT - ROW_TITLE_GAP;
        for (text, y) in [(SPIN_TITLE, SPIN_ROW_Y), (DOUBLE_TITLE, DOUBLE_ROW_Y)] {
            let left = title_x - text_w(text, ROW_TITLE_SIZE);
            assert!(left > g.right() + 0.2, "{text}");
            assert!(left > trustee_at(5)[0] + DISC_R + 0.2, "{text}");
            assert!(y + text_h(ROW_TITLE_SIZE) / 2.0 < STRIP_CAPTION_Y - 0.3, "{text}");
        }
        const { assert!(FRAME_Y + FRAME_H / 2.0 < SPIN_ROW_Y) };
        // the strip's last tick inside the edge, and the ticks' rows under
        // it
        assert!(strip_right(MARKS) < edge);
        assert!(tick_at(MARKS - 1)[0] + HIT_R < edge);
    }
}
