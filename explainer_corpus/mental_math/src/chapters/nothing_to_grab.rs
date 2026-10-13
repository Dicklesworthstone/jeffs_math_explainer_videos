//! Chapter 13, nothing to grab: the last quiz of the mixed round, whose
//! ring empties into silence with no answer (13_1); why no trick fits, one
//! note a sentence (13_2); parts still works, in four slow steps (13_3);
//! the sheet of paper and a calculator are the right tools (13_4); and the
//! habit to keep even then: round first, which catches a slipped decimal
//! point but no small slip (13_5). The six tiles of the round stay along
//! the top, dark, and only the parts tile lights.

use explainer_kit::{Kit, Palette, Timed, play, v};
use fmn::prelude::*;

use super::draw::{
    Board, Calculator, CutRect, NOTE_PT, NumberLine1, QuizLook, SIDE_PT, Sheet, Side, TileKind, WHICH_TRICK, eq,
    fade_in_all, fade_out_all, fail, fit_width, fly, glyphs, gone, grey_line, line_style, middle_of, num, paper_ink,
    pause, quiz_at, repaint, say, slate, swap, tag, tiles_rest_and_clear, tiles_top, your_turn,
};
use crate::facts::shown;
use crate::model::{Ask, by_parts, estimate, pair, parts, round_then_fix};

pub struct NothingToGrab {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const NO: &str = "no";
const MIDDLE: &str = "middle";
const TRUE_NO_EASIER: &str = r"\text{true, and no easier}";
const ABOUT: &str = "about";

/// The quiz card, under the tiles at the left, with room beside it.
const CARD_AT: Vec3 = [-3.5, -0.55, 0.0];
const CARD_SCALE: f64 = 0.8;
/// The notes of 13_2 beside the card: their left edge, heights and size.
const NOTES_LEFT: f64 = 0.3;
const NOTE_YS: [f64; 3] = [0.85, -0.35, -1.55];
const NOTE_SIZE: f64 = 56.0;
/// The rectangle cut into four pieces (13_3), beside the card: its lower
/// left corner and the room it may take.
const RECT_AT: Vec3 = [0.75, -3.05, 0.0];
const RECT_ROOM: (f64, f64) = (5.8, 4.95);
/// A piece's area written on it: the block, and the thinner pieces.
const BLOCK_PT: f64 = 52.0;
/// The pause between two steps of the running total.
const STEP_S: f64 = 1.0;
/// The tag under the steps while they are a running total, not the answer
/// (R2, M13).
const RUNNING: &str = "running total";
/// The sheet of paper (13_4) and the calculator beside it.
const SHEET_AT: Vec3 = [1.6, -0.7, 0.0];
const SHEET_PT: f64 = 52.0;
const CALC_AT: Vec3 = [5.3, -0.7, 0.0];
const CALC_SCALE: f64 = 1.15;
/// The estimate (13_5), between the card and the calculator.
const EST_X: f64 = 1.6;
const EST_YS: [f64; 3] = [0.85, -0.3, -1.25];
const EST_PT: f64 = 64.0;
/// The number line along the foot, and the size of its two numbers.
const AXIS: (f64, f64, f64) = (-6.4, 6.4, -3.1);
const AXIS_PT: f64 = 40.0;

/// Two red lines across `over`, corner to corner: a reading that is wrong.
/// Detached.
fn cross(stage: &mut Stage<'_>, over: Mob) -> [Mob; 2] {
    let (l, r) = (stage.get_left(over)[0] - 0.12, stage.get_right(over)[0] + 0.12);
    let (b, t) = (stage.get_bottom(over)[1] - 0.1, stage.get_top(over)[1] + 0.1);
    let style = line_style(gone(), 4.5);
    [(v(l, b), v(r, t)), (v(l, t), v(r, b))]
        .map(|(p, q)| stage.arena_mut().add(polyline(vec![p, q]).style(style).build()))
}

/// Draw two lines one after the other, `rt` seconds in all.
fn strike(stage: &mut Stage<'_>, lines: [Mob; 2], rt: f64) -> fmn::Result<()> {
    play!(stage; show_creation(lines[0]).rt(rt / 2.0));
    play!(stage; show_creation(lines[1]).rt(rt / 2.0));
    Ok(())
}

impl SceneConstruct for NothingToGrab {
    fn name(&self) -> &str {
        "13_nothing_to_grab"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let (a, b) = shown::NOTHING;
        let quiz = quiz_at("13_1")?;
        if quiz.ask != Ask::Times(a, b) {
            return Err(fail(format!("quiz 13_1 asks {:?}, not {a} × {b}", quiz.ask)));
        }
        let steps = quiz.worked().map_err(fail)?.steps;
        let by = by_parts(a, b);
        if steps.len() != by.pieces.len() {
            return Err(fail(format!("{} steps for {} pieces", steps.len(), by.pieces.len())));
        }
        let product = by.product();
        let (round_a, round_b, about) = estimate(a, b);

        // the six tiles along the top from the first frame, where chapter 12
        // leaves them, all at rest: none is put out before the viewer has
        // thought (R2, M14)
        let tiles = tiles_top(stage)?;
        for t in &tiles {
            for m in t.mobs() {
                stage.add_to_scene(m)?;
            }
        }

        // 13_1: the last quiz of the round; the ring empties into silence
        // and nothing lands
        let look = QuizLook {
            at: CARD_AT,
            label: Some(WHICH_TRICK),
            pause_line: true,
            lands: false,
            scale: CARD_SCALE,
        };
        let card = your_turn(&kit, stage, "13_1", look)?;

        // 13_2: nothing to grab: a note by the problem for each trick that
        // does not fit, one a sentence
        let line = say(&kit, stage, "13_2")?;
        let five = shown::FIVE % 10;
        if a % 10 == five || b % 10 == five {
            return Err(fail(format!("{a} × {b} has a number that ends in {five}")));
        }
        let no_five = kit.tex_left(
            stage,
            &format!(r"\text{{{NO}}}\ {five}"),
            NOTE_SIZE,
            ink,
            &[],
            v(NOTES_LEFT, NOTE_YS[0]),
        )?;
        // its middle, in halves: an odd sum, so a half
        if pair(a, b).is_some() {
            return Err(fail(format!("{a} and {b} have a whole middle")));
        }
        let half = kit.tex_left(
            stage,
            &format!(r"\text{{{MIDDLE}}}\ {}\tfrac{{1}}{{2}}", (a + b) / 2),
            NOTE_SIZE,
            ink,
            &[],
            v(NOTES_LEFT, NOTE_YS[1]),
        )?;
        // round, then fix: the round number is close, but there are sixty-
        // seven things to fix
        let fix = round_then_fix(a, b, round_b);
        let aside_src = format!(r"{} \times {} - {}:\ {TRUE_NO_EASIER}", fix.count, fix.round, fix.fix);
        let aside = grey_line(&kit, stage, &aside_src, v(0.0, NOTE_YS[2]))?;
        fit_width(stage, aside, 6.6);
        stage.move_to(aside, v(NOTES_LEFT, NOTE_YS[2]), LEFT);
        let notes = [no_five, half, aside];
        // nothing to grab: every tile goes dark on the line's first words
        let mut anims = Vec::new();
        for t in &tiles {
            anims.extend(t.light(stage, false, 0.6)?);
        }
        stage.play_prepared(anims)?;
        for (j, &note) in notes.iter().enumerate() {
            line.wait_sentence(stage, j + 1)?;
            play!(stage; fade_in(stage.arena_mut(), note, v(-0.25, 0.0), 1.0)?.rt(0.5));
        }

        // 13_3: parts still works. Only the parts tile lights; the rectangle
        // in four pieces beside the card, and the total steps up where the
        // ring was, piece by piece, to the answer the card never got
        let line = say(&kit, stage, "13_3")?;
        let parts_tile = tiles
            .iter()
            .find(|t| t.kind == TileKind::Parts)
            .ok_or_else(|| fail("no parts tile in the row".into()))?;
        let mut anims = parts_tile.light(stage, true, 0.5)?;
        for &m in &notes {
            let f = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(f.rt(0.4))?);
        }
        stage.play_prepared(anims)?;
        let board = Board::fit(RECT_AT, b as f64, a as f64, RECT_ROOM);
        let cut = CutRect::build(stage, board, a, b, slate());
        let ((a_tens, a_ones), (b_tens, b_ones)) = (parts(a), parts(b));
        let sides = [
            board.label(&kit, stage, &num(a_tens), cut.rects[0], Side::Left, SIDE_PT, ink)?,
            board.label(&kit, stage, &num(a_ones), cut.rects[2], Side::Left, SIDE_PT, ink)?,
            board.label(&kit, stage, &num(b_tens), cut.rects[0], Side::Below, SIDE_PT, ink)?,
            board.label(&kit, stage, &num(b_ones), cut.rects[1], Side::Below, SIDE_PT, ink)?,
        ];
        let mut rect_mobs = cut.pieces.clone();
        rect_mobs.extend(&cut.cuts);
        rect_mobs.extend(sides);
        fade_in_all(stage, &rect_mobs, 0.6)?;
        let mut shown_total: Option<Mob> = None;
        let at = card.answer_at();
        let running = tag(&kit, stage, RUNNING, v(at[0], at[1] - 0.95), 28.0)?;
        for (i, step) in steps.iter().enumerate() {
            if i + 1 == steps.len() {
                line.wait_sentence(stage, 1)?;
                play!(stage; fade_out(stage.arena_mut(), running, ORIGIN, 1.0)?.rt(0.3));
            } else if i > 0 {
                pause(stage, STEP_S)?;
            } else {
                play!(stage; fade_in(stage.arena_mut(), running, ORIGIN, 1.0)?.rt(0.3));
            }
            let size = if i == 0 { BLOCK_PT } else { NOTE_PT };
            let area = kit.text(stage, &num(by.pieces[i].area()), size, paper_ink(), cut.centre(i))?;
            let total = eq(&kit, stage, step, look.pt(), card.answer_at())?;
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            anims.push(stage.prepare(repaint(cut.pieces[i], cut.role(i), 1.0, 0.4)?)?);
            for m in [area, total] {
                let f = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(f.rt(0.4))?);
            }
            if let Some(old) = shown_total {
                let f = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
                anims.push(stage.prepare(f.rt(0.4))?);
            }
            stage.play_prepared(anims)?;
            rect_mobs.push(area);
            shown_total = Some(total);
        }
        let total = shown_total.ok_or_else(|| fail("no steps to the total".into()))?;

        // 13_4: four pieces with carrying is slow; the sheet of paper from
        // 01_4 slides back in with the pieces written on it and their sum,
        // and a calculator beside it
        let line = say(&kit, stage, "13_4")?;
        let mut rows: Vec<String> = by.pieces.iter().map(|p| num(p.area())).collect();
        if let Some(last) = rows.last_mut() {
            *last = format!("+ {last}");
        }
        rows.push(num(product));
        let row_refs: Vec<&str> = rows.iter().map(String::as_str).collect();
        let sheet = Sheet::build(&kit, stage, SHEET_AT, &row_refs, &[rows.len() - 2], SHEET_PT)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &rect_mobs {
            let f = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(f.rt(0.5))?);
        }
        for m in sheet.mobs() {
            let f = fade_in(stage.arena_mut(), m, v(0.0, 1.4), 1.0)?;
            anims.push(stage.prepare(f.rt(0.9))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "or a calculator")?;
        let calc = Calculator::build(stage, CALC_AT, CALC_SCALE)?;
        fade_in_all(stage, &calc.parts, 0.5)?;

        // 13_5: round first. Seventy times eighty, about fifty-six hundred;
        // a slipped decimal point either way is caught, crossed out in red;
        // then the answer beside the estimate on a number line
        let line = say(&kit, stage, "13_5")?;
        fade_out_all(stage, &sheet.mobs(), 0.5)?;
        line.wait_sentence(stage, 1)?;
        let rounded = eq(
            &kit,
            stage,
            &format!(r"{round_a} \times {round_b}"),
            EST_PT,
            v(EST_X, EST_YS[0]),
        )?;
        play!(stage; fade_in(stage.arena_mut(), rounded, v(0.0, -0.12), 1.0)?.rt(0.5));
        line.wait_sentence(stage, 2)?;
        let estimated = eq(&kit, stage, &format!("= {about}"), EST_PT, v(EST_X, EST_YS[1]))?;
        let about_tag = tag(&kit, stage, ABOUT, v(EST_X, EST_YS[2]), 30.0)?;
        play!(stage;
            fade_in(stage.arena_mut(), estimated, v(0.0, -0.12), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), about_tag, ORIGIN, 1.0)?.rt(0.5),
        );
        line.wait_for(stage, "five hundred")?;
        // a decimal point slipped one place either way
        let slipped_low = format!("{}.{}", product / 10, product % 10);
        let low = calc.reading(&kit, stage, &slipped_low, ink)?;
        play!(stage; fade_in(stage.arena_mut(), low, ORIGIN, 1.0)?.rt(0.3));
        let low_cross = cross(stage, low);
        strike(stage, low_cross, 0.4)?;
        line.wait_for(stage, "or fifty thousand")?;
        let high = calc.reading(&kit, stage, &num(product * 10), ink)?;
        swap(stage, &[low, low_cross[0], low_cross[1]], &[high], 0.3)?;
        let high_cross = cross(stage, high);
        strike(stage, high_cross, 0.4)?;
        // the number line: the estimate first, then the answer beside it
        line.wait_sentence(stage, 4)?;
        let axis_line = NumberLine1 {
            x0: AXIS.0,
            x1: AXIS.1,
            y: AXIS.2,
            lo: 0.0,
            hi: 2.0 * about as f64,
        };
        // a tick at every unit of the estimate's leading digit (every
        // thousand for fifty-six hundred)
        let step = 10f64.powi(num(about).len() as i32 - 1);
        let mut on_axis = vec![axis_line.axis(stage)];
        let mut k = 0.0;
        while k <= axis_line.hi + 1e-9 {
            on_axis.push(axis_line.tick(stage, k, 0.18, Palette::faint()));
            k += step;
        }
        let about_dot = axis_line.dot(stage, about as f64, 0.09, ink);
        on_axis.push(about_dot);
        fade_in_all(stage, &on_axis, 0.5)?;
        // the estimate's number comes down to its place over the line
        let estimate_glyphs = glyphs(stage, estimated);
        let from = middle_of(stage, &estimate_glyphs[1..])?;
        let about_mark = eq(&kit, stage, &num(about), EST_PT, from)?;
        stage.add_to_scene(about_mark)?;
        let about_at = axis_line.p(about as f64);
        play!(stage; fly(about_mark, v(about_at[0], AXIS.2 + 0.45), AXIS_PT / EST_PT, 0.8)?);
        line.wait_sentence(stage, 5)?;
        // and the answer from the card comes down beside it, under the line
        let answer_mark = eq(&kit, stage, &num(product), look.pt(), stage.get_center(total))?;
        let answer_dot = axis_line.dot(stage, product as f64, 0.09, ink);
        let answer_at = axis_line.p(product as f64);
        stage.add_to_scene(answer_mark)?;
        play!(stage;
            fly(answer_mark, v(answer_at[0], AXIS.2 - 0.45), AXIS_PT / look.pt(), 0.9)?,
            fade_in(stage.arena_mut(), answer_dot, ORIGIN, 1.0)?.rt(0.9),
        );
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        // the tiles stay, all at rest, for chapter 14's quiz
        tiles_rest_and_clear(stage, &tiles, &[], 0.8)?;
        kit.finish(stage)
    }
}
