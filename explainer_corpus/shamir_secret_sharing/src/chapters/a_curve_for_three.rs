//! Chapter 09, a curve for three: the vault's own rule, any three of five,
//! and how the dealer made the five cards chapter 06 handed out from the
//! sealed envelope (8, 3, 1, 2, 6). They lie face down in the rail's frames
//! here, so nothing on screen answers 09_5's question before it is asked,
//! and turn over only in 09_6. The line gives way to a curve (09_1) and
//! the dealer spins twice, 7 and then 7, any of the 121 pairs being
//! possible. Each trustee's height is the secret, plus the first spin
//! times its position, plus the second spin times its position squared,
//! and squared is shown as position times position, 1, 4, 9, 16, 25,
//! before any name is given (09_2). Trustee 1's 19 and trustee 2's 47 are worked out term by term
//! (09_3). On ordinary paper the five heights sit on one smooth parabola,
//! drawn on its own plot, and any three of the points redraw it (09_4).
//! Then the dial board, points only and no curve, and the viewer is asked
//! where 47 lands before anything wraps (09_5). 47 winds four times round
//! a small dial and lands on 3; the other heights wrap too, and the five
//! face-down cards turn over one by one: 8, 3, 1, 2 and 6, the vault's
//! shares (09_6). A rule like this is a polynomial: one spin, a line, for
//! any two; two spins, a curve, for any three (09_7).
//!
//! The heights, terms, turns and shares are the model's (`Recipe`, `Dial`,
//! `envelope::dealt`); the paper curve's recovery from any three points is
//! `model::paper_sum` and `model::paper_curve_spins`; the board and coil
//! are `board.rs`'s. Emphasis never restyles or scales typeset text: cards,
//! labels and the formula are pointed at by outlines of their own, a card
//! turns over by its back going before its number comes in, and only
//! shapes (points, discs, lines) change color or swell.

use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::board::{BOARD_POINT_R, Board, Grid, coil, now, ring, turns_and_mark, turns_and_more};
use super::draw::{
    FRAME_H, FRAME_W, Rail, card, disc, fade_in_all, fail, frame_at, panel, pause, recolorings, say, stroke, swaps,
    think,
};
use super::envelope::{dealt, frame_halos};
use super::the_line_on_the_dial::LINE_BOARD;
use super::two_keys::{ARC_LABEL_IN, Face, SPARE_R, glow, glowing, leaving, text_halo};
use crate::model::{
    self, ADJACENT, Dial, LINE_SLOPE, OPENING_TRIO, Ratio, Recipe, SECRET, SPINS, THRESHOLD, TRUSTEES, combinations,
};
use crate::palette;

pub struct ACurveForThree {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model)
const SPIN_TITLES: [&str; 2] = ["1st spin", "2nd spin"];
const TERM_TAGS: [&str; 3] = ["the secret", "1st spin", "2nd spin"];
const PAPER_TITLE: &str = "on paper: ordinary numbers";
const BOARD_TITLE: &str = "on the dial: points only, no curve";
const LINE_BOARD_TITLE: &str = "on the dial: points only";
const POLY_LABEL: &str = "a polynomial:";
const POLY_BODY: &str = "secret + spin \u{d7} position + spin \u{d7} position \u{d7} position + \u{2026}";

/// Whole turns the spins make before they stop.
const TURNS: u32 = 2;

/// The phrase each card turns over at, trustee by trustee (09_6's list).
const FLIP_CUES: [&str; 5] = [
    "eight, three",
    "three, one, two",
    "one, two and six",
    "two and six",
    "and six",
];

// ------------------------------------------------------------------ layout

/// The rule card, upper left (the plot comes there in 09_4).
const RULE_X: f64 = -3.5;
const RULE_Y: f64 = 3.4;
const RULE_SIZE: f64 = 26.0;
/// The sketch of a line giving way to a curve, left.
const SKETCH_FROM: (f64, f64) = (-5.2, 0.4);
const SKETCH_TO: (f64, f64) = (-2.4, 2.4);
const SKETCH_W: f64 = 4.0;
/// The two spins' dials, right, their titles over them, and the note on
/// every pair under them.
const SPIN_X: [f64; 2] = [2.3, 4.9];
const SPIN_Y: f64 = 1.2;
const SPIN_TITLE_DY: f64 = SPARE_R + 0.35;
const SPIN_TITLE_SIZE: f64 = 22.0;
const SPUN_SIZE: f64 = 22.0;
const NOTE_X: f64 = 3.6;
const NOTE_Y: f64 = -0.35;
const NOTE_SIZE: f64 = 18.0;
/// The formula, one term a line, left-aligned top right, and each term's
/// tag right of its line.
const FORM_X: f64 = 0.8;
const FORM_Y: [f64; 3] = [3.3, 2.85, 2.4];
const FORM_SIZE: f64 = 28.0;
const TAG_GAP: f64 = 0.3;
const TAG_SIZE: f64 = 20.0;
/// Squared as position times position: two rows, their titles left and
/// the numbers in columns.
const SQUARE_Y: [f64; 2] = [1.75, 1.35];
const SQUARE_TITLE_SIZE: f64 = 20.0;
const SQUARE_X0: f64 = 4.3;
const SQUARE_DX: f64 = 0.55;
const SQUARE_SIZE: f64 = 22.0;
/// Trustees 1 and 2's working, two rows each, the second indented.
const WORK_X: f64 = 0.8;
const WORK_Y: [f64; 4] = [0.75, 0.35, -0.15, -0.55];
const WORK_INDENT: f64 = 1.45;
const WORK_GAP: f64 = 0.15;
const WORK_SIZE: f64 = 24.0;
/// The paper plot, left: position 0's place and step, height 0's place
/// and one unit, the labelled heights, the title, and the points.
const PAPER_X0: f64 = -5.6;
const PAPER_DX: f64 = 0.85;
const PAPER_Y0: f64 = -0.3;
const PAPER_DY: f64 = 0.0145;
const PAPER_TOP: i64 = 220;
const PAPER_STEP: usize = 50;
const PAPER_LABEL_SIZE: f64 = 18.0;
const PAPER_LABEL_DX: f64 = 0.36;
const PAPER_POSITION_DY: f64 = 0.25;
const PAPER_TITLE_Y: f64 = 3.45;
const PAPER_TITLE_SIZE: f64 = 22.0;
const PAPER_RUN_ON: f64 = 0.45;
const AXIS_W: f64 = 2.5;
const TICK: f64 = 0.08;
const POINT_R: f64 = 0.08;
const POINT_LABEL_SIZE: f64 = 22.0;
const POINT_LABEL_DX: f64 = -0.3;
const POINT_LABEL_DY: f64 = 0.26;
const CURVE_W: f64 = 4.0;
const CURVE_SAMPLES: usize = 100;
/// The board, right, from 09_5.
const CURVE_BOARD: Grid = Grid {
    x0: 1.5,
    y0: -0.15,
    dx: 0.85,
    dy: 0.27,
    marks: crate::model::MARKS,
    last: 5,
};
/// The think ring, under the board.
const THINK_X: f64 = 5.6;
const THINK_Y: f64 = -1.35;
/// The small dial that 47 winds round, between the plot and the board, and
/// each height's turns under it.
const SMALL_X: f64 = -0.05;
const SMALL_Y: f64 = 2.25;
const SMALL_R: f64 = 0.45;
const TURNS_X: f64 = 0.0;
const TURNS_Y0: f64 = 1.2;
const TURNS_DY: f64 = 0.35;
const TURNS_SIZE: f64 = 18.0;
/// The vault's shares, under the board.
const VAULT_X: f64 = 3.8;
const VAULT_Y: f64 = -0.95;
const VAULT_SIZE: f64 = 20.0;
/// 09_7's card over the boards, and each board's tag under it.
const POLY_Y: f64 = 3.55;
const POLY_SIZE: f64 = 24.0;
const POLY_GAP: f64 = 0.2;
const BOARD_TAG_DY: f64 = 0.7;
const BOARD_TAG_SIZE: f64 = 22.0;
/// 09_7's left board: chapter 08's columns, the right board's rows.
const SMALL_LINE_BOARD: Grid = Grid {
    y0: CURVE_BOARD.y0,
    dy: CURVE_BOARD.dy,
    ..LINE_BOARD
};
/// How far a card lifts out and drops in, and the matched cards' hold.
const DROP: f64 = 0.3;
const HOLD_MATCHED: f64 = 1.5;
const HOLD_AT_END: f64 = 0.8;

// -------------------------------------------------------------------- data

/// The rule card: any `k` of `n`.
fn rule(k: usize, n: usize) -> String {
    format!("the vault's rule: any {k} of {n}")
}

/// Every pair of spins can come up.
fn pairs_note(marks: u32) -> String {
    format!(
        "any of the {} pairs can come up, repeats and zeros included",
        marks * marks
    )
}

/// The formula's three lines, one term each.
fn formula(recipe: &Recipe) -> [String; 3] {
    [
        format!("height = {}", recipe.secret),
        format!("+ {} \u{d7} position", recipe.spins[0]),
        format!("+ {} \u{d7} position \u{d7} position", recipe.spins[1]),
    ]
}

/// Each term of the height at `x`: the secret, the first spin times `x`,
/// the second times `x` times `x`.
fn terms(recipe: &Recipe, x: u32) -> [i64; 3] {
    let x = i64::from(x);
    [
        i64::from(recipe.secret),
        i64::from(recipe.spins[0]) * x,
        i64::from(recipe.spins[1]) * x * x,
    ]
}

/// Trustee `x`'s working: the formula at `x`, the terms, the height.
fn working(recipe: &Recipe, x: u32) -> [String; 3] {
    let [s, a, b] = terms(recipe, x);
    [
        format!(
            "trustee {x}: {} + {} \u{d7} {x} + {} \u{d7} {x} \u{d7} {x}",
            recipe.secret, recipe.spins[0], recipe.spins[1]
        ),
        format!("= {s} + {a} + {b}"),
        format!("= {}", s + a + b),
    ]
}

/// The vault's shares, as chapter 01's cards held them.
fn vault_shares(shares: &[u32]) -> String {
    let words: Vec<String> = shares.iter().map(ToString::to_string).collect();
    format!("the vault's shares: {}", words.join(", "))
}

/// A board's tag: `spins` spins draw a line or a curve, for any `k` of `n`.
fn board_tag(spins: usize, k: usize, n: usize) -> String {
    let (spin, shape) = if spins == 1 {
        ("spin", "a line")
    } else {
        ("spins", "a curve")
    };
    format!("{spins} {spin}: {shape}, any {k} of {n}")
}

/// The scene point at position `x`, height `h`, on the paper plot.
fn paper_at(x: f64, h: f64) -> Vec3 {
    v(PAPER_X0 + PAPER_DX * x, PAPER_Y0 + PAPER_DY * h)
}

/// The parabola s + a·x + b·x² from position 0 to `last`, sampled.
fn parabola(recipe: &Recipe, last: u32) -> Vec<(f64, f64)> {
    let (s, a, b) = (
        f64::from(recipe.secret),
        f64::from(recipe.spins[0]),
        f64::from(recipe.spins[1]),
    );
    (0..=CURVE_SAMPLES)
        .map(|i| {
            let x = f64::from(last) * i as f64 / CURVE_SAMPLES as f64;
            (x, s + a * x + b * x * x)
        })
        .collect()
}

/// The paper curve through three ordinary points, rebuilt from them alone:
/// its height at 0 and its two spins.
fn rebuilt(trio: &[(u32, i64)]) -> Option<(Ratio, Ratio, Ratio)> {
    let secret = model::paper_sum(trio);
    let s = u32::try_from(secret.whole()?).ok()?;
    let (a, b) = model::paper_curve_spins([trio[0], trio[1]], s)?;
    // the third point lies on it too
    let (x, h) = (i64::from(trio[2].0), trio[2].1);
    (Ratio::int(i64::from(s)).plus(a.times(x)).plus(b.times(x * x)) == Ratio::int(h)).then_some((secret, a, b))
}

/// The paper plot: the axes, a tick and a label at every 50th height and
/// under every position, and the title. Detached.
fn paper_axes(kit: &Kit, stage: &mut Stage<'_>, last: u32) -> fmn::Result<Vec<Mob>> {
    let origin = paper_at(0.0, 0.0);
    let right = paper_at(f64::from(last), 0.0)[0] + PAPER_RUN_ON;
    let top = paper_at(0.0, PAPER_TOP as f64)[1];
    let mut parts = vec![
        stroke(stage, vec![origin, v(right, PAPER_Y0)], Palette::dim(), AXIS_W),
        stroke(stage, vec![origin, v(PAPER_X0, top)], Palette::dim(), AXIS_W),
    ];
    for h in (0..=PAPER_TOP).step_by(PAPER_STEP) {
        let p = paper_at(0.0, h as f64);
        if h > 0 {
            parts.push(stroke(stage, vec![p, v(p[0] - TICK, p[1])], Palette::dim(), AXIS_W));
        }
        parts.push(kit.text(
            stage,
            &h.to_string(),
            PAPER_LABEL_SIZE,
            Palette::dim(),
            v(PAPER_X0 - PAPER_LABEL_DX, p[1]),
        )?);
    }
    for x in 0..=last {
        let p = paper_at(f64::from(x), 0.0);
        if x > 0 {
            parts.push(stroke(stage, vec![p, v(p[0], p[1] - TICK)], Palette::dim(), AXIS_W));
        }
        parts.push(kit.text(
            stage,
            &x.to_string(),
            PAPER_LABEL_SIZE,
            Palette::dim(),
            v(p[0], PAPER_Y0 - PAPER_POSITION_DY),
        )?);
    }
    parts.push(kit.text_left(
        stage,
        PAPER_TITLE,
        PAPER_TITLE_SIZE,
        Palette::dim(),
        v(PAPER_X0 - PAPER_LABEL_DX, PAPER_TITLE_Y),
    )?);
    Ok(parts)
}

/// A point on the paper plot and its height's label, up and to the left.
/// Detached.
fn paper_point(kit: &Kit, stage: &mut Stage<'_>, x: u32, h: i64, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let p = paper_at(f64::from(x), h as f64);
    let dot = disc(stage, color, p, POINT_R);
    let label = kit.text(
        stage,
        &h.to_string(),
        POINT_LABEL_SIZE,
        color,
        v(p[0] + POINT_LABEL_DX, p[1] + POINT_LABEL_DY),
    )?;
    Ok([dot, label])
}

/// A card face down in trustee `t`'s frame on the rail. Detached.
fn card_back(stage: &mut Stage<'_>, t: u32) -> fmn::Result<Mob> {
    panel(stage, Palette::dim(), 0.85, frame_at(t), FRAME_W - 0.14, FRAME_H - 0.12)
}

/// Text left-aligned at `at`: each piece of a row after the last one's
/// right edge.
fn text_after(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, after: Mob) -> fmn::Result<Mob> {
    let c = stage.get_center(after);
    let x = c[0] + stage.get_width(after) / 2.0 + WORK_GAP;
    kit.text_left(stage, s, size, color, v(x, c[1]))
}

/// The card turning over: its back lifts away, then its number drops in,
/// prepared.
fn turning_over(stage: &mut Stage<'_>, back: Mob, face: Mob) -> fmn::Result<Box<dyn Animation>> {
    let a = fade_out(stage.arena_mut(), back, v(0.0, DROP), 1.0)?;
    let out = stage.prepare(a.rt(0.2))?;
    let a = fade_in(stage.arena_mut(), face, v(0.0, -DROP), 1.0)?;
    let inn = stage.prepare(a.rt(0.25))?;
    let both = Succession::new(stage.arena_mut(), vec![out, inn])?;
    stage.prepare(both)
}

impl SceneConstruct for ACurveForThree {
    fn name(&self) -> &str {
        "09_a_curve_for_three"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let marks = dial.marks();
        let recipe = Recipe::example();
        let last = *TRUSTEES.last().ok_or_else(|| fail("no trustees".into()))?;
        let heights: Vec<i64> = TRUSTEES.iter().map(|&t| recipe.height(t)).collect();
        let shares = recipe.deal(dial, &TRUSTEES);
        // the heights are their terms added up, they wrap onto the shares,
        // and those are the cards the vault was dealt
        let vault: Vec<u32> = dealt(dial, &TRUSTEES).iter().map(|p| p.1).collect();
        for ((&t, &h), &s) in TRUSTEES.iter().zip(&heights).zip(&shares) {
            if terms(&recipe, t).iter().sum::<i64>() != h || turns_and_mark(h, marks).1 != s || h > PAPER_TOP {
                return Err(fail(format!("trustee {t}: height {h}, share {s}")));
            }
        }
        if shares != vault {
            return Err(fail(format!("the curve deals {shares:?}, the vault holds {vault:?}")));
        }
        // on paper, any three of the five points give back the same curve
        let paper: Vec<(u32, i64)> = TRUSTEES.iter().copied().zip(heights.iter().copied()).collect();
        let want = (
            Ratio::int(i64::from(SECRET)),
            Ratio::int(i64::from(SPINS[0])),
            Ratio::int(i64::from(SPINS[1])),
        );
        for idx in combinations(&TRUSTEES, THRESHOLD) {
            let trio: Vec<(u32, i64)> = idx.iter().map(|&x| paper[(x - TRUSTEES[0]) as usize]).collect();
            if rebuilt(&trio) != Some(want) {
                return Err(fail(format!("trustees {idx:?} rebuild {:?} on paper", rebuilt(&trio))));
            }
        }
        let index = |t: u32| (t - TRUSTEES[0]) as usize;

        // the opening: the rail, every card face down
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;
        let mut backs = Vec::with_capacity(TRUSTEES.len());
        for &t in &TRUSTEES {
            backs.push(card_back(stage, t)?);
        }
        fade_in_all(stage, &backs, 0.4)?;

        // 09_1: any three of five; the line gives way to a curve; two spins
        let spoken = say(&kit, stage, "09_1")?;
        spoken.wait_for(stage, "For any three of five")?;
        let rule_card = kit.text(
            stage,
            &rule(THRESHOLD, TRUSTEES.len()),
            RULE_SIZE,
            Palette::ink(),
            v(RULE_X, RULE_Y),
        )?;
        let mut anims = swaps(stage, &[], &[rule_card], 0.5)?;
        let cards_held = frame_halos(stage, &TRUSTEES, palette::share())?;
        anims.extend(glowing(stage, &cards_held, 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "replace the line")?;
        let (p, q) = (SKETCH_FROM, SKETCH_TO);
        let sketch_line = stroke(stage, vec![v(p.0, p.1), v(q.0, q.1)], Palette::dim(), SKETCH_W);
        let drawn = stage.prepare(show_creation(sketch_line).rt(0.6))?;
        stage.play_prepared(vec![drawn])?;
        spoken.wait_for(stage, "with a curve")?;
        // the same two ends, bending: a curve where the line was
        let bend: Vec<Vec3> = (0..=40)
            .map(|i| {
                let t = f64::from(i) / 40.0;
                v(p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t * t)
            })
            .collect();
        let sketch_curve = stroke(stage, bend, Palette::ink(), SKETCH_W);
        let mut anims = vec![stage.prepare(show_creation(sketch_curve).rt(0.8))?];
        anims.extend(recolorings(stage, &[(sketch_line, palette::ghost())], 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "The dealer spins twice")?;
        let mut faces = Vec::with_capacity(SPINS.len());
        let mut titles = Vec::with_capacity(SPINS.len());
        for (i, &x) in SPIN_X.iter().enumerate() {
            let f = Face::build(&kit, stage, dial, v(x, SPIN_Y), SPARE_R, palette::spin(), 0)?;
            titles.push(kit.text(
                stage,
                SPIN_TITLES[i],
                SPIN_TITLE_SIZE,
                palette::spin(),
                v(x, SPIN_Y + SPIN_TITLE_DY),
            )?);
            faces.push(f);
        }
        fade_in_all(stage, &titles, 0.3)?;
        for f in &faces {
            f.show(stage, 0.3)?;
        }
        let mut spun = Vec::with_capacity(SPINS.len());
        for (i, (cue, f)) in ["say the dial gives seven", "then seven again"]
            .iter()
            .zip(&faces)
            .enumerate()
        {
            // the first spin turns as "spins twice" is said; it stops, and
            // its mark is named, at its cue
            if i == 1 {
                spoken.wait_for(stage, cue)?;
            }
            let stop = f64::from(SPINS[i] + TURNS * marks);
            let spin_it = f
                .needle
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.4),
                    rate_func: Some(fmn::core::rate::rush_from),
                    ..AnimateArgs::default()
                })?
                .set_value(stop)?;
            stage.play(spin_it)?;
            if i == 0 {
                spoken.wait_for(stage, cue)?;
            }
            let tag = kit.text(
                stage,
                &SPINS[i].to_string(),
                SPUN_SIZE,
                palette::spin(),
                f.point(f64::from(SPINS[i]), SPARE_R * ARC_LABEL_IN),
            )?;
            let halo = f.halo(stage, SPINS[i], palette::spin());
            let mut anims = swaps(stage, &[], &[tag], 0.4)?;
            anims.extend(glowing(stage, &[halo], 0.6)?);
            stage.play_prepared(anims)?;
            spun.push(tag);
        }
        let note = kit.text(stage, &pairs_note(marks), NOTE_SIZE, Palette::dim(), v(NOTE_X, NOTE_Y))?;
        fade_in_all(stage, &[note], 0.4)?;
        spoken.wait_end(stage)?;

        // 09_2: the height, term by term; squared is position × position
        let spoken = say(&kit, stage, "09_2")?;
        let mut going = vec![sketch_line, sketch_curve, note];
        going.extend(titles);
        going.extend(spun);
        for f in &faces {
            going.extend(f.parts());
        }
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        let lines = formula(&recipe);
        let colors = [palette::secret(), palette::spin(), palette::spin()];
        let keys = [
            recipe.secret.to_string(),
            recipe.spins[0].to_string(),
            recipe.spins[1].to_string(),
        ];
        let mut form = Vec::with_capacity(lines.len());
        let mut tags = Vec::with_capacity(lines.len());
        for (i, cue) in [
            "the secret,",
            "plus seven times their position,",
            "plus seven times their position squared",
        ]
        .iter()
        .enumerate()
        {
            spoken.wait_for(stage, cue)?;
            // the number alone in its role's color: each line holds just
            // the one number, so its key colors nothing else
            let m = kit.text_t2c(
                stage,
                &lines[i],
                FORM_SIZE,
                Palette::ink(),
                &[(keys[i].as_str(), colors[i])],
                v(FORM_X, FORM_Y[i]),
            )?;
            stage.move_to(m, v(FORM_X, FORM_Y[i]), LEFT);
            let tag_x = FORM_X + stage.get_width(m) + TAG_GAP;
            let tag = kit.text_left(stage, TERM_TAGS[i], TAG_SIZE, colors[i], v(tag_x, FORM_Y[i]))?;
            let anims = swaps(stage, &[], &[m, tag], 0.5)?;
            stage.play_prepared(anims)?;
            form.push(m);
            tags.push(tag);
        }
        // squared, concretely: each position times itself
        let mut squares = Vec::new();
        let titles = ["position".to_owned(), "squared: position \u{d7} position".to_owned()];
        for (title, &y) in titles.iter().zip(&SQUARE_Y) {
            squares.push(kit.text_left(stage, title, SQUARE_TITLE_SIZE, Palette::dim(), v(FORM_X, y))?);
        }
        for (i, &t) in TRUSTEES.iter().enumerate() {
            let x = SQUARE_X0 + SQUARE_DX * i as f64;
            squares.push(kit.text(stage, &t.to_string(), SQUARE_SIZE, palette::share(), v(x, SQUARE_Y[0]))?);
            squares.push(kit.text(
                stage,
                &(t * t).to_string(),
                SQUARE_SIZE,
                Palette::ink(),
                v(x, SQUARE_Y[1]),
            )?);
        }
        fade_in_all(stage, &squares, 0.6)?;
        spoken.wait_end(stage)?;

        // 09_3: trustees 1 and 2 worked out, term by term
        let spoken = say(&kit, stage, "09_3")?;
        let mut work = Vec::new();
        let mut sums = Vec::with_capacity(2);
        for (j, (&t, cues)) in TRUSTEES[..2]
            .iter()
            .zip([
                ["Trustee one gets", "five plus seven plus seven", "nineteen"],
                ["Trustee two gets", "five plus fourteen", "forty-seven"],
            ])
            .enumerate()
        {
            let [formula_at, added, total] = working(&recipe, t);
            spoken.wait_for(stage, cues[0])?;
            let first = kit.text_left(stage, &formula_at, WORK_SIZE, Palette::ink(), v(WORK_X, WORK_Y[2 * j]))?;
            let mut anims = swaps(stage, &[], &[first], 0.4)?;
            let mut halos = frame_halos(stage, &[t], palette::share())?;
            halos.push(ring(
                stage,
                rail_disc_at(t),
                super::draw::TRUSTEE_R + 0.08,
                palette::share(),
            ));
            anims.extend(glowing(stage, &halos, 0.7)?);
            stage.play_prepared(anims)?;
            spoken.wait_for(stage, cues[1])?;
            let second = kit.text_left(
                stage,
                &added,
                WORK_SIZE,
                Palette::ink(),
                v(WORK_X + WORK_INDENT, WORK_Y[2 * j + 1]),
            )?;
            fade_in_all(stage, &[second], 0.4)?;
            spoken.wait_for(stage, cues[2])?;
            let third = text_after(&kit, stage, &total, WORK_SIZE, palette::share(), second)?;
            fade_in_all(stage, &[third], 0.4)?;
            let halo = text_halo(stage, third, palette::share())?;
            glow(stage, &[halo], 0.6)?;
            work.extend([first, second, third]);
            sums.push(third);
        }
        spoken.wait_end(stage)?;

        // 09_4: on paper: the five points on one parabola; any three redraw it
        let spoken = say(&kit, stage, "09_4")?;
        let anims = leaving(stage, &[rule_card], ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        let axes = paper_axes(&kit, stage, last)?;
        let secret_dot = disc(stage, palette::secret(), paper_at(0.0, f64::from(SECRET)), POINT_R);
        let mut dots = Vec::with_capacity(TRUSTEES.len());
        let mut labels = Vec::with_capacity(TRUSTEES.len());
        for (&t, &h) in TRUSTEES.iter().zip(&heights) {
            let [d, l] = paper_point(&kit, stage, t, h, palette::share())?;
            dots.push(d);
            labels.push(l);
        }
        let mut first_in = axes.clone();
        first_in.extend([secret_dot, dots[0], labels[0], dots[1], labels[1]]);
        fade_in_all(stage, &first_in, 0.7)?;
        let halos: Vec<Mob> = sums
            .iter()
            .map(|&m| text_halo(stage, m, palette::share()))
            .collect::<fmn::Result<_>>()?;
        glow(stage, &halos, 0.6)?;
        spoken.wait_for(stage, "Three, four and five get")?;
        let rest = &TRUSTEES[2..];
        let rings: Vec<Mob> = rest
            .iter()
            .map(|&t| ring(stage, rail_disc_at(t), super::draw::TRUSTEE_R + 0.08, palette::share()))
            .collect();
        glow(stage, &rings, 0.6)?;
        spoken.wait_for(stage, "eighty-nine")?;
        // a connected list: each point and its label as its height is said
        let rt = (spoken.at("On paper")? - now(stage)).clamp(1.2, 3.0);
        let mut each = Vec::with_capacity(rest.len());
        for &t in rest {
            let i = index(t);
            let both = swaps(stage, &[], &[dots[i], labels[i]], 1.0)?;
            let g = AnimationGroup::new(stage.arena_mut(), both)?;
            each.push(stage.prepare(g)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), each, 0.6)?.with_run_time(rt);
        stage.play(g)?;
        spoken.wait_for(stage, "one smooth curve")?;
        let samples: Vec<Vec3> = parabola(&recipe, last)
            .into_iter()
            .map(|(x, h)| paper_at(x, h))
            .collect();
        let curve = stroke(stage, samples.clone(), Palette::ink(), CURVE_W);
        let a_ = stage.prepare(show_creation(curve).rt(1.4))?;
        stage.play_prepared(vec![a_])?;
        spoken.wait_for(stage, "any three of them pin it down")?;
        // two trios, each pointed at, and the curve redrawn through them
        for trio in [[1, 3, 5], [2, 3, 4]] {
            let pts: Vec<(u32, i64)> = trio.iter().map(|&t| paper[index(t)]).collect();
            if rebuilt(&pts) != Some(want) {
                return Err(fail(format!("trustees {trio:?} do not rebuild the curve")));
            }
            let rings: Vec<Mob> = trio
                .iter()
                .map(|&t| {
                    let i = index(t);
                    ring(
                        stage,
                        paper_at(f64::from(t), heights[i] as f64),
                        2.2 * POINT_R,
                        palette::trio(),
                    )
                })
                .collect();
            let again = stroke(stage, samples.clone(), palette::trio(), CURVE_W);
            let mut anims = vec![stage.prepare(show_creation(again).rt(0.9))?];
            anims.extend(glowing(stage, &rings, 1.2)?);
            stage.play_prepared(anims)?;
            let anims = leaving(stage, &[again], ORIGIN, 0.4)?;
            stage.play_prepared(anims)?;
        }
        spoken.wait_end(stage)?;

        // 09_5: wrap them: where does 47 land? (nothing wraps yet)
        let spoken = say(&kit, stage, "09_5")?;
        let mut going = form;
        going.extend(tags);
        going.extend(squares);
        going.extend(work);
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Now wrap them")?;
        let board = Board::build(&kit, stage, CURVE_BOARD, BOARD_TITLE, true)?;
        board.show(stage, 0.7)?;
        // trustee 2's 47, worked out in 09_3
        let asked = TRUSTEES[1];
        spoken.wait_for(stage, "trustee two's forty-seven")?;
        let mut halos = vec![text_halo(stage, labels[index(asked)], palette::share())?];
        halos.extend(frame_halos(stage, &[asked], palette::share())?);
        glow(stage, &halos, 0.9)?;
        spoken.wait_end(stage)?;
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 09_6: 47 winds four times round and lands on 3; every card turns
        // over: the vault's shares
        let spoken = say(&kit, stage, "09_6")?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        let small = Face::build(&kit, stage, dial, v(SMALL_X, SMALL_Y), SMALL_R, Palette::dim(), 0)?;
        small.show(stage, 0.4)?;
        let i2 = index(asked);
        let (h2, s2) = (heights[i2], shares[i2]);
        let rt = (spoken.at("so it lands on three")? - now(stage)).clamp(1.6, 3.2);
        let wound = coil(stage, &small, marks, h2 as u32, palette::share());
        let anims = vec![
            stage.prepare(show_creation(wound).rt(rt))?,
            stage.prepare(tween(small.needle, h2 as f64, rt)?)?,
        ];
        stage.play_prepared(anims)?;
        let turned: Vec<Mob> = TRUSTEES
            .iter()
            .zip(&heights)
            .enumerate()
            .map(|(i, (_, &h))| {
                kit.text(
                    stage,
                    &turns_and_more(h, marks),
                    TURNS_SIZE,
                    Palette::ink(),
                    v(TURNS_X, TURNS_Y0 - TURNS_DY * i as f64),
                )
            })
            .collect::<fmn::Result<_>>()?;
        let landed = small.halo(stage, s2, palette::share());
        let mut anims = glowing(stage, &[landed], 0.8)?;
        anims.extend(swaps(stage, &[], &[turned[i2]], 0.4)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "so it lands on three")?;
        let mut board_points = Vec::with_capacity(TRUSTEES.len());
        for (&t, &s) in TRUSTEES.iter().zip(&shares) {
            board_points.push(board.point(stage, t, s, palette::share()));
        }
        let mut anims = swaps(stage, &[], &[board_points[i2]], 0.4)?;
        anims.push(super::envelope::swelling(
            stage,
            board_points[i2],
            1.8,
            palette::share(),
            0.7,
        )?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "All five shares")?;
        let others: Vec<Mob> = (0..TRUSTEES.len()).filter(|&i| i != i2).map(|i| turned[i]).collect();
        let anims = swaps(stage, &[], &others, 0.6)?;
        stage.play_prepared(anims)?;
        let mut faces_up = Vec::with_capacity(TRUSTEES.len());
        for (i, (&t, &s)) in TRUSTEES.iter().zip(&shares).enumerate() {
            spoken.wait_for(stage, FLIP_CUES[i])?;
            let up = card(&kit, stage, t, &s.to_string(), palette::share())?;
            let mut anims = vec![turning_over(stage, backs[i], up)?];
            if i != i2 {
                anims.extend(swaps(stage, &[], &[board_points[i]], 0.3)?);
            }
            let r = ring(
                stage,
                board.grid.at(f64::from(t), f64::from(s)),
                2.0 * BOARD_POINT_R,
                palette::share(),
            );
            anims.extend(glowing(stage, &[r], 0.5)?);
            stage.play_prepared(anims)?;
            faces_up.push(up);
        }
        spoken.wait_for(stage, "the very shares from the vault")?;
        let caption = kit.text(
            stage,
            &vault_shares(&vault),
            VAULT_SIZE,
            Palette::dim(),
            v(VAULT_X, VAULT_Y),
        )?;
        let mut anims = swaps(stage, &[], &[caption], 0.5)?;
        let all_cards = frame_halos(stage, &TRUSTEES, palette::share())?;
        anims.extend(glowing(stage, &all_cards, 1.2)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_MATCHED)?;

        // 09_7: a polynomial; one spin a line for any two, two a curve for
        // any three
        let spoken = say(&kit, stage, "09_7")?;
        let mut going = axes;
        going.extend([secret_dot, curve, wound, caption]);
        going.extend(dots);
        going.extend(labels);
        going.extend(turned);
        going.extend(small.parts());
        // These cards belong to the two-spin board. Retire them before
        // the different one-spin example appears in their old position.
        going.extend(rail.parts());
        going.extend(faces_up);
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "the secret plus spins times powers")?;
        let body = kit.text(stage, POLY_BODY, POLY_SIZE, Palette::ink(), v(0.0, POLY_Y))?;
        let label = kit.text(stage, POLY_LABEL, POLY_SIZE, Palette::ink(), v(0.0, POLY_Y))?;
        // the label then the body, centred together across the frame
        let (lw, bw) = (stage.get_width(label), stage.get_width(body));
        let left = -(lw + POLY_GAP + bw) / 2.0;
        stage.move_to(label, v(left, POLY_Y), LEFT);
        stage.move_to(body, v(left + lw + POLY_GAP, POLY_Y), LEFT);
        fade_in_all(stage, &[body], 0.5)?;
        spoken.wait_for(stage, "is called a polynomial")?;
        fade_in_all(stage, &[label], 0.4)?;
        let halo = text_halo(stage, label, Palette::ink())?;
        glow(stage, &[halo], 0.7)?;
        spoken.wait_for(stage, "One spin gives a line")?;
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        let left_board = Board::build(&kit, stage, SMALL_LINE_BOARD, LINE_BOARD_TITLE, true)?;
        let mut line_points = vec![left_board.point(stage, 0, SECRET, palette::secret())];
        for &t in &TRUSTEES {
            line_points.push(left_board.point(stage, t, line.share(dial, t), palette::share()));
        }
        let pieces = left_board.line(
            stage,
            (0.0, f64::from(SECRET)),
            (f64::from(last), line.height(last) as f64),
            Palette::dim(),
            super::board::WRAPPED_W,
        );
        let line_tag = kit.text(
            stage,
            &board_tag(line.spins.len(), line.coefficients().len(), TRUSTEES.len()),
            BOARD_TAG_SIZE,
            Palette::ink(),
            v(
                (SMALL_LINE_BOARD.left() + SMALL_LINE_BOARD.right()) / 2.0,
                SMALL_LINE_BOARD.bottom() - BOARD_TAG_DY,
            ),
        )?;
        let mut parts_in = left_board.parts().to_vec();
        parts_in.extend(&line_points);
        parts_in.extend(&pieces);
        parts_in.push(line_tag);
        fade_in_all(stage, &parts_in, 0.6)?;
        spoken.wait_for(stage, "for any two")?;
        let two: Vec<Mob> = ADJACENT
            .iter()
            .map(|&t| {
                ring(
                    stage,
                    left_board.grid.at(f64::from(t), f64::from(line.share(dial, t))),
                    2.0 * BOARD_POINT_R,
                    palette::share(),
                )
            })
            .collect();
        glow(stage, &two, 0.8)?;
        spoken.wait_for(stage, "Two spins give a curve")?;
        let curve_tag = kit.text(
            stage,
            &board_tag(recipe.spins.len(), recipe.coefficients().len(), TRUSTEES.len()),
            BOARD_TAG_SIZE,
            Palette::ink(),
            v(
                (CURVE_BOARD.left() + CURVE_BOARD.right()) / 2.0,
                CURVE_BOARD.bottom() - BOARD_TAG_DY,
            ),
        )?;
        let secret_on_board = board.point(stage, 0, SECRET, palette::secret());
        fade_in_all(stage, &[curve_tag, secret_on_board], 0.5)?;
        spoken.wait_for(stage, "for any three")?;
        let three: Vec<Mob> = OPENING_TRIO
            .iter()
            .map(|&t| {
                ring(
                    stage,
                    board.grid.at(f64::from(t), f64::from(shares[index(t)])),
                    2.0 * BOARD_POINT_R,
                    palette::trio(),
                )
            })
            .collect();
        let anims = glowing(stage, &three, 0.9)?;
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_AT_END)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

/// Trustee `t`'s disc on the rail.
fn rail_disc_at(t: u32) -> Vec3 {
    super::draw::trustee_at(t)
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::board::coil_radius;
    use super::super::draw::{FRAME_Y, THINK_R};
    use super::*;
    use crate::facts;
    use crate::model::{MARKS, OTHER_SPINS, csv};

    /// The production narration: registering chapter 09 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// The facts' list form of the terms at `x`, `x` first: "1,5,7,7,19".
    fn terms_row(recipe: &Recipe, x: u32) -> String {
        let [s, a, b] = terms(recipe, x);
        csv(&[i64::from(x), s, a, b, s + a + b])
    }

    /// Every phrase the scene waits for is in its line of the script, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        let mut phrases: Vec<(&str, &str)> = FLIP_CUES.iter().map(|&p| ("09_6", p)).collect();
        phrases.extend([
            ("09_1", "For any three of five"),
            ("09_1", "replace the line"),
            ("09_1", "with a curve"),
            ("09_1", "The dealer spins twice"),
            ("09_1", "say the dial gives seven"),
            ("09_1", "then seven again"),
            ("09_2", "the secret,"),
            ("09_2", "plus seven times their position,"),
            ("09_2", "plus seven times their position squared"),
            ("09_3", "Trustee one gets"),
            ("09_3", "five plus seven plus seven"),
            ("09_3", "nineteen"),
            ("09_3", "Trustee two gets"),
            ("09_3", "five plus fourteen"),
            ("09_3", "forty-seven"),
            ("09_4", "Three, four and five get"),
            ("09_4", "eighty-nine"),
            ("09_4", "On paper"),
            ("09_4", "one smooth curve"),
            ("09_4", "any three of them pin it down"),
            ("09_5", "Now wrap them"),
            ("09_5", "trustee two's forty-seven"),
            ("09_6", "so it lands on three"),
            ("09_6", "All five shares"),
            ("09_6", "the very shares from the vault"),
            ("09_7", "the secret plus spins times powers"),
            ("09_7", "is called a polynomial"),
            ("09_7", "One spin gives a line"),
            ("09_7", "for any two"),
            ("09_7", "Two spins give a curve"),
            ("09_7", "for any three"),
        ]);
        for (id, phrase) in phrases {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
            assert_eq!(text.matches(phrase).count(), 1, "{id}: {phrase:?} twice");
        }
        // the cards turn over in the order the shares are said
        let text = script.text("09_6").unwrap();
        let at: Vec<usize> = FLIP_CUES.iter().map(|c| text.find(c).unwrap()).collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]));
    }

    /// What the scene draws and writes is the facts': the rule, the spins
    /// and their 121 pairs, the formula, the squares, the working, the five
    /// heights, 47's turns and every height's, and the shares.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let recipe = Recipe::example();
        assert_eq!(rule(THRESHOLD, TRUSTEES.len()), "the vault's rule: any 3 of 5");
        assert_eq!(f.get("S-F6.spins"), Some(csv(&recipe.spins).as_str()));
        assert_eq!(
            pairs_note(MARKS),
            format!(
                "any of the {} pairs can come up, repeats and zeros included",
                f.get("S-F6.spin_pairs").unwrap()
            )
        );
        assert_eq!(
            formula(&recipe),
            [
                "height = 5".to_owned(),
                "+ 7 \u{d7} position".to_owned(),
                "+ 7 \u{d7} position \u{d7} position".to_owned(),
            ]
        );
        // each formula line holds its colored number once, and no other
        // digit that number's key would also color
        for (line, key) in formula(&recipe).iter().zip(["5", "7", "7"]) {
            assert_eq!(line.matches(key).count(), 1, "{line}");
        }
        assert_eq!(f.get("S-F6.terms.1"), Some(terms_row(&recipe, 1).as_str()));
        assert_eq!(f.get("S-F6.terms.2"), Some(terms_row(&recipe, 2).as_str()));
        assert_eq!(
            working(&recipe, 2),
            [
                "trustee 2: 5 + 7 \u{d7} 2 + 7 \u{d7} 2 \u{d7} 2".to_owned(),
                "= 5 + 14 + 28".to_owned(),
                "= 47".to_owned(),
            ]
        );
        let heights: Vec<i64> = TRUSTEES.iter().map(|&t| recipe.height(t)).collect();
        assert_eq!(f.get("S-F6.heights"), Some(csv(&heights).as_str()));
        assert_eq!(f.get("S-F6.rest.heights"), Some(csv(&heights[2..]).as_str()));
        assert_eq!(f.get("S-F6.asked"), Some(csv(&[2, heights[1]]).as_str()));
        let (turns, k) = turns_and_mark(heights[1], MARKS);
        assert_eq!(
            f.get("S-F6.wrap.2"),
            Some(csv(&[heights[1], turns, i64::from(k)]).as_str())
        );
        assert_eq!(turns_and_more(heights[1], MARKS), "47 = 4 \u{d7} 11 + 3");
        let all_turns: Vec<i64> = heights.iter().map(|&h| turns_and_mark(h, MARKS).0).collect();
        assert_eq!(f.get("S-F6.turns"), Some(csv(&all_turns).as_str()));
        let shares = recipe.deal(dial, &TRUSTEES);
        assert_eq!(f.get("S-F6.shares"), Some(csv(&shares).as_str()));
        assert_eq!(vault_shares(&shares), "the vault's shares: 8, 3, 1, 2, 6");
        assert_eq!(board_tag(1, 2, 5), "1 spin: a line, any 2 of 5");
        assert_eq!(board_tag(2, 3, 5), "2 spins: a curve, any 3 of 5");
        assert_eq!(f.get("S-F6.paper.curves"), Some("1"));
    }

    /// The face-down cards turn up as the vault's own: each height wrapped
    /// is the recipe's share there (Horner's rule on the dial), which is
    /// what chapter 04's sealed method dealt. The planted negative: chapter
    /// 05's other spins deal different cards from the same secret.
    #[test]
    fn the_cards_are_the_vaults() {
        let dial = Dial::vault();
        let recipe = Recipe::example();
        let vault = dealt(dial, &TRUSTEES);
        for (&t, &(x, s)) in TRUSTEES.iter().zip(&vault) {
            assert_eq!(x, t);
            assert_eq!(dial.wrap(recipe.height(t)), s);
            assert_eq!(recipe.share(dial, t), s);
        }
        assert_eq!(vault.iter().map(|p| p.1).collect::<Vec<_>>(), vec![8, 3, 1, 2, 6]);
        let other = Recipe::new(SECRET, &OTHER_SPINS);
        assert_ne!(other.deal(dial, &TRUSTEES), recipe.deal(dial, &TRUSTEES));
        // the secret is the same for both: the cards differ, not the vault
        assert_eq!(other.share(dial, 0), recipe.share(dial, 0));
    }

    /// The paper curve through three points, solved here by Cramer's rule
    /// on the three equations s + a·x + b·x² = h, independently of the
    /// model's weights.
    fn cramer(trio: &[(u32, i64)]) -> (Ratio, Ratio, Ratio) {
        let rows: Vec<[i64; 4]> = trio
            .iter()
            .map(|&(x, h)| {
                let x = i64::from(x);
                [1, x, x * x, h]
            })
            .collect();
        let det3 = |c: [usize; 3]| -> i64 {
            let m = |r: usize, k: usize| rows[r][c[k]];
            m(0, 0) * (m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1)) - m(0, 1) * (m(1, 0) * m(2, 2) - m(1, 2) * m(2, 0))
                + m(0, 2) * (m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0))
        };
        let d = det3([0, 1, 2]);
        (
            Ratio::new(det3([3, 1, 2]), d),
            Ratio::new(det3([0, 3, 2]), d),
            Ratio::new(det3([0, 1, 3]), d),
        )
    }

    /// On paper, any three of the five points give back the dealer's
    /// curve, 5 + 7x + 7x², and the scene's rebuilding agrees; two points
    /// are not enough: with spins allowed to be fractions, every secret 0
    /// to 10 has a curve through trustees 1 and 2's 19 and 47.
    #[test]
    fn any_three_points_pin_the_paper_curve() {
        let recipe = Recipe::example();
        let paper: Vec<(u32, i64)> = TRUSTEES.iter().map(|&t| (t, recipe.height(t))).collect();
        let want = (Ratio::int(5), Ratio::int(7), Ratio::int(7));
        let trios = combinations(&TRUSTEES, 3);
        assert_eq!(trios.len(), 10);
        for idx in trios {
            let trio: Vec<(u32, i64)> = idx.iter().map(|&x| paper[(x - 1) as usize]).collect();
            assert_eq!(cramer(&trio), want, "{idx:?}");
            assert_eq!(rebuilt(&trio), Some(want), "{idx:?}");
        }
        let pair = [paper[0], paper[1]];
        let secrets = (0..MARKS)
            .filter(|&s| model::paper_curve_spins(pair, s).is_some())
            .count();
        assert_eq!(secrets, 11);
        // a planted negative: a wrong height breaks the rebuilding
        let bad = [paper[0], paper[1], (3, paper[2].1 + 1)];
        assert_ne!(cramer(&bad), want);
        assert_eq!(rebuilt(&[paper[0], paper[2], (5, paper[4].1 + 1)]), None);
    }

    /// The drawn parabola passes exactly through the five points and the
    /// secret, and stays on the plot.
    #[test]
    fn the_parabola_meets_every_point() {
        let recipe = Recipe::example();
        let samples = parabola(&recipe, 5);
        for x in 0..=5u32 {
            let (sx, sh) = samples[(x as usize) * CURVE_SAMPLES / 5];
            assert!((sx - f64::from(x)).abs() < 1e-12);
            assert!((sh - recipe.height(x) as f64).abs() < 1e-9);
        }
        assert!(samples.iter().all(|&(_, h)| (0.0..=PAPER_TOP as f64).contains(&h)));
        assert!(samples.windows(2).all(|w| w[1].1 > w[0].1), "it climbs");
    }

    /// The pieces keep apart, inside the frame: the rule, sketch, spins,
    /// note, formula and tags, squares and working on the right in their
    /// turn; the paper plot left; the board right; the small dial and the
    /// turns between them; the think ring, the caption, the card and the
    /// boards' tags.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let edge = FRAME_X_RADIUS - 0.25;
        let recipe = Recipe::example();
        // 09_1
        let rule_w = text_w(&rule(3, 5), RULE_SIZE);
        assert!(RULE_X - rule_w / 2.0 > -edge && RULE_X + rule_w / 2.0 < SPIN_X[0] - SPARE_R * ARC_LABEL_IN);
        assert!(SKETCH_TO.1 < RULE_Y - text_h(RULE_SIZE));
        const {
            assert!(SKETCH_FROM.1 > FRAME_Y + 0.6);
            assert!(SKETCH_TO.0 < SPIN_X[0] - SPARE_R * ARC_LABEL_IN - 0.3);
            assert!(SPIN_X[0] + SPARE_R * ARC_LABEL_IN < SPIN_X[1] - SPARE_R * ARC_LABEL_IN);
        }
        assert!(SPIN_X[1] + SPARE_R * ARC_LABEL_IN < edge);
        assert!(SPIN_Y + SPIN_TITLE_DY + text_h(SPIN_TITLE_SIZE) < FRAME_Y_RADIUS - 0.25);
        let note_w = text_w(&pairs_note(MARKS), NOTE_SIZE);
        assert!(NOTE_X + note_w / 2.0 < edge && NOTE_X - note_w / 2.0 > SKETCH_TO.0);
        assert!(NOTE_Y + text_h(NOTE_SIZE) < SPIN_Y - SPARE_R * ARC_LABEL_IN);
        // 09_2, 09_3: the formula and tags, the squares and the working
        for (line, tag) in formula(&recipe).iter().zip(TERM_TAGS) {
            let right = FORM_X + text_w(line, FORM_SIZE) + TAG_GAP + text_w(tag, TAG_SIZE);
            assert!(right < edge, "{line}");
        }
        assert!(FORM_Y[0] + text_h(FORM_SIZE) / 2.0 < FRAME_Y_RADIUS - 0.25);
        assert!(FORM_Y[2] - text_h(FORM_SIZE) / 2.0 > SQUARE_Y[0] + text_h(SQUARE_SIZE) / 2.0);
        let square_title = text_w("squared: position \u{d7} position", SQUARE_TITLE_SIZE);
        assert!(FORM_X + square_title < SQUARE_X0 - text_w("25", SQUARE_SIZE));
        assert!(SQUARE_X0 + 4.0 * SQUARE_DX + text_w("25", SQUARE_SIZE) / 2.0 < edge);
        assert!(SQUARE_Y[1] - text_h(SQUARE_SIZE) / 2.0 > WORK_Y[0] + text_h(WORK_SIZE) / 2.0);
        for x in [1, 2] {
            let [first, added, total] = working(&recipe, x);
            assert!(WORK_X + text_w(&first, WORK_SIZE) < edge, "{first}");
            let right = WORK_X + WORK_INDENT + text_w(&added, WORK_SIZE) + WORK_GAP + text_w(&total, WORK_SIZE);
            assert!(right < edge, "{added} {total}");
        }
        assert!(WORK_Y[3] - text_h(WORK_SIZE) / 2.0 > FRAME_Y + 0.3);
        assert!(FORM_X > paper_at(5.0, 0.0)[0] + PAPER_RUN_ON + 0.3);
        // 09_4: the paper plot, its tallest label under its title, its
        // labels clear of the rail
        let top = label_y(215);
        assert!(top + text_h(POINT_LABEL_SIZE) / 2.0 < PAPER_TITLE_Y - text_h(PAPER_TITLE_SIZE) / 2.0);
        assert!(PAPER_TITLE_Y + text_h(PAPER_TITLE_SIZE) / 2.0 < FRAME_Y_RADIUS - 0.25);
        assert!(PAPER_Y0 - PAPER_POSITION_DY - text_h(PAPER_LABEL_SIZE) / 2.0 > FRAME_Y + 0.3);
        assert!(PAPER_X0 - PAPER_LABEL_DX - text_w("200", PAPER_LABEL_SIZE) / 2.0 > -edge);
        // 09_5 on: the board right of the plot, the small dial and the turns
        // between them
        let g = CURVE_BOARD;
        let plot_right = paper_at(5.0, 0.0)[0] + PAPER_RUN_ON;
        let row_labels_left = g.row_label_at(10)[0] - text_w("10", 16.0) / 2.0;
        let reach = SMALL_R * coil_radius(47.0, MARKS);
        assert!(SMALL_X - reach > paper_at(5.0, 0.0)[0] + POINT_R + 0.3);
        assert!(SMALL_X + reach < row_labels_left - 0.2);
        assert!(SMALL_Y + reach < g.top() + 0.3);
        assert!(SMALL_Y + reach < FRAME_Y_RADIUS - 0.25);
        for (i, &h) in [19, 47, 89, 145, 215].iter().enumerate() {
            let w = text_w(&turns_and_more(h, MARKS), TURNS_SIZE);
            let y = TURNS_Y0 - TURNS_DY * i as f64;
            assert!(TURNS_X + w / 2.0 < row_labels_left - 0.1, "{h}");
            assert!(TURNS_X - w / 2.0 > plot_right - 0.1, "{h}");
            assert!(y + text_h(TURNS_SIZE) / 2.0 < SMALL_Y - reach, "{h}");
        }
        assert!(g.right() < edge);
        assert!(g.title_at()[1] + text_h(20.0) / 2.0 < POLY_Y - text_h(POLY_SIZE) / 2.0);
        assert!(g.position_label_at(0)[1] - text_h(16.0) / 2.0 > THINK_Y + THINK_R);
        assert!(THINK_X + THINK_R < edge);
        assert!(THINK_X + text_w(super::super::draw::PAUSE_NOTE, 18.0) / 2.0 < edge);
        let caption_w = text_w(&vault_shares(&[8, 3, 1, 2, 6]), VAULT_SIZE);
        assert!(VAULT_X + caption_w / 2.0 < edge && VAULT_X - caption_w / 2.0 > g.left());
        assert!(VAULT_Y + text_h(VAULT_SIZE) / 2.0 < g.position_label_at(0)[1] - 0.1);
        assert!(VAULT_Y - text_h(VAULT_SIZE) / 2.0 > FRAME_Y + 0.3);
        // 09_7: the card across the top, the boards' tags over the rail
        let card_w = text_w(POLY_LABEL, POLY_SIZE) + POLY_GAP + text_w(POLY_BODY, POLY_SIZE);
        assert!(card_w / 2.0 < edge);
        assert!(POLY_Y + text_h(POLY_SIZE) / 2.0 < FRAME_Y_RADIUS - 0.2);
        let l = SMALL_LINE_BOARD;
        assert!(l.right() < g.left() - 0.5);
        assert!(l.row_label_at(0)[0] - text_w("10", 16.0) / 2.0 > -edge);
        for (board, tag) in [(l, board_tag(1, 2, 5)), (g, board_tag(2, 3, 5))] {
            let y = board.bottom() - BOARD_TAG_DY;
            assert!(y - text_h(BOARD_TAG_SIZE) / 2.0 > FRAME_Y + 0.25, "{tag}");
            assert!(y + text_h(BOARD_TAG_SIZE) / 2.0 < board.position_label_at(0)[1] - text_h(16.0) / 2.0);
            assert!(text_w(&tag, BOARD_TAG_SIZE) < board.right() - board.left(), "{tag}");
        }
    }

    /// The centre of the label of the point at position 5 with height `h`.
    fn label_y(h: i64) -> f64 {
        paper_at(5.0, h as f64)[1] + POINT_LABEL_DY
    }
}
