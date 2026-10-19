//! Chapter 01, a limit that jumps: the line y = x on [0, 1] and its powers
//! up to x⁵⁰, every one unbroken (01_1, 01_2); the fixed point x = 0.9,
//! whose heights sink toward zero (01_3), and the endpoint, which never
//! moves (01_4); the prediction (01_5) and the limit, flat at zero with a
//! single point at height one, so unbroken curves have a limit that jumps
//! (01_6); why a picture of eight curves cannot prove it and arithmetic can
//! (01_7); the title (01_8); then the moving point where each curve crosses
//! one half (01_9), whose gap to the limit stays one half while it creeps
//! toward one (01_10), against a fixed point's finite wait (01_11).
//!
//! Every curve is one group of short pieces sharing the same sample x, so
//! the bright curve morphs into the next power point by point, straight
//! down, and leaves a dim copy behind. Every number drawn comes from the
//! facts registry.

use explainer_kit::props::precisely;
use explainer_kit::{Facts, Kit, Palette, Timed, play, v};
use fmn::prelude::*;

use super::draw::{
    BRIGHT_W, Bright, GHOST_W, PLOT, curve_color, curve_label, fact, fade_in_all, fade_in_lagged, fade_out_all, fail,
    fixed, gap_color, ghost, limit_color, line_style, move_to, moving, say, step, think_pause,
};
use super::glyphs::{SUBTITLE, TITLE};
use crate::facts::{EXACT_UP_TO, FIXED, PLACES, WAITS, decimal_point, facts};
use crate::model::{POWERS, decimal_power, gap, half_height, limit, power};

pub struct ALimitThatJumps {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const EACH_ROW: &str = "each row: ×";
const CURVES_DRAWN: &str = "curves drawn, up to n =";
const JUMP: &str = "jump";
/// Where the jump is, in the domain's own terms: [0, 1] has nothing right of 1.
const AT_THE_END: [&str; 2] = ["at x = 1, the right end of [0, 1],", "x arrives only from the left"];
const CROSSING_AT: &str = "crossing at x =";
const GAP_AT_CROSSING: &str = "gap at the crossing:";
const FIX_FIRST: &str = "fix x first";
const GAP_BELOW: &str = "gap below";
const FROM: &str = "from";
const N_IS: &str = "n =";
const MOVE_WITH: &str = "let x move with n";
const FOR_EVERY_N: &str = "for every n";

// what the pictures wait for, each in the line the test below holds it to
const SQUARE_AT: &str = "Square every height";
const HIGHER_AT: &str = "Higher powers";
const HEIGHT_AT: [&str; 3] = ["point nine,", "point eight one", "point seven two nine"];
const DOWN_AT: &str = "down toward zero";
const ANY_POWER_AT: &str = "One to any power";
const POINT_AT: &str = "then a single point";
const JUMPS_AT: &str = "a limit that jumps";
const ARITHMETIC_AT: &str = "Arithmetic can";
const HALF_AT: &str = "half height";
const GROWS_AT: &str = "As the power grows";
const MOVES_AT: &str = "Let it move with the curve";

/// The title card's listen window (storyboard 01_8: listen 2 s).
pub const TITLE_S: f64 = 2.0;
/// The listen label of the title card, the score's open motif.
pub const TITLE_LABEL: &str = "01_a_limit_that_jumps:01_8";

// --- layout (scene units; the frame is 14.2 by 8; the plot is draw::PLOT)

/// The table right of the plot: n right-aligned here, values left-aligned
/// there, a header and rows going down.
const TABLE_N: f64 = 1.0;
const TABLE_VALUE: f64 = 1.6;
const TABLE_HEAD_Y: f64 = 3.0;
const TABLE_ROW0_Y: f64 = 2.45;
const TABLE_STEP: f64 = 0.43;
/// The notes under the table, left-aligned.
const NOTE_X: f64 = 0.4;
const NOTE_Y: [f64; 2] = [-1.1, -1.6];
/// The right column's centre, and the think ring there.
const COLUMN_X: f64 = 5.0;
const RING_AT: Vec3 = [COLUMN_X, 1.6, 0.0];
const RING_R: f64 = 0.75;
/// The two contrast cards of 01_11, left-aligned, top lines.
const CARD_X: f64 = 3.9;
const CARD_Y: [f64; 2] = [2.9, 0.5];
const CARD_STEP: f64 = 0.42;
/// The jump's marker sits this far right of x = 1.
const JUMP_DX: f64 = 0.18;

// the notes, the ring and the cards stay inside the frame and above a
// three-line lower-third (its top at -2.42); the cards do not touch; the
// table's value column clears the cards by more than its longest entry
// ("≈ 0.0052") needs
const _: () = assert!(NOTE_Y[1] - 0.2 > -2.42 && NOTE_Y[0] > NOTE_Y[1]);
const _: () = assert!(RING_AT[0] + RING_R < 7.1 && RING_AT[1] - RING_R - 0.45 > -2.42);
const _: () = assert!(CARD_Y[1] - 2.0 * CARD_STEP - 0.3 > -2.42);
const _: () = assert!(CARD_Y[0] - CARD_STEP * (WAITS.len() as f64 + 1.0) > CARD_Y[1] + 0.3);
const _: () = assert!(CARD_X - TABLE_VALUE > 1.6);
// the moving point's header ("crossing at x =", about 2 units at 24 pt)
// ends before the cards begin, and the widest card line ("gap = 1/2, for
// every n", about 2.9 units) ends inside the frame
const _: () = assert!(TABLE_VALUE + 2.1 < CARD_X && CARD_X + 3.0 < 7.1);

fn row_y(i: usize) -> f64 {
    TABLE_ROW0_Y - TABLE_STEP * i as f64
}

/// The fixed point's height after n steps as the table shows it: exact up
/// to [`EXACT_UP_TO`], above it rounded after "≈".
fn shown_height(f: &Facts, n: u32) -> fmn::Result<String> {
    if n <= EXACT_UP_TO {
        fact(f, &format!("C1.fixed.n{n}"))
    } else {
        Ok(format!("≈ {}", fact(f, &format!("C1.fixed.n{n}.{PLACES}dp"))?))
    }
}

/// A table row, detached: n right-aligned in its column, the value after.
fn table_row(kit: &Kit, stage: &mut Stage<'_>, i: usize, n: u32, value: &str, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let y = row_y(i);
    let left = kit.text(stage, &n.to_string(), 26.0, Palette::dim(), v(TABLE_N, y))?;
    stage.move_to(left, v(TABLE_N, y), RIGHT);
    let right = kit.text_left(stage, value, 26.0, color, v(TABLE_VALUE, y))?;
    Ok([left, right])
}

/// The moving point's height and its limit's at x on the n-th curve: the
/// red bar stands on the limit and is as tall as the computed gap.
pub(super) fn gap_span(x: f64, n: u32) -> fmn::Result<(f64, f64)> {
    let floor = limit(x).map_err(fail)?;
    Ok((floor, floor + gap(x, n).map_err(fail)?))
}

/// The half-height line and its 1/2 by the y axis, detached.
pub(super) fn half_line(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    let line = PLOT.dashed(stage, (0.0, 0.5), (1.0, 0.5), moving())?;
    let side = PLOT.p(0.0, 0.5);
    let label = kit.tex(stage, r"\tfrac{1}{2}", 30.0, moving(), &[], v(side[0] - 0.35, side[1]))?;
    Ok([line, label])
}

/// The moving point on the n-th curve and its gap bar, at rest, detached.
pub(super) fn witness_at(stage: &mut Stage<'_>, n: u32) -> fmn::Result<[Mob; 2]> {
    let x = half_height(n).map_err(fail)?;
    let (floor, top) = gap_span(x, n)?;
    let bar = PLOT.segment(stage, (x, top), (x, floor), gap_color(), 6.0)?;
    let dot = PLOT.dot(stage, x, top, moving());
    Ok([dot, bar])
}

/// The moving point's table header, detached.
pub(super) fn witness_head(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    let n_head = kit.tex(stage, "n", 30.0, Palette::dim(), &[], v(TABLE_N - 0.07, TABLE_HEAD_Y))?;
    let x_head = kit.text_left(stage, CROSSING_AT, 24.0, moving(), v(TABLE_VALUE, TABLE_HEAD_Y))?;
    Ok([n_head, x_head])
}

/// The moving point's table row i, for the power POWERS[i], detached.
pub(super) fn witness_row(kit: &Kit, stage: &mut Stage<'_>, f: &Facts, i: usize) -> fmn::Result<[Mob; 2]> {
    let n = POWERS[i];
    let x_text = fact(f, &format!("C1.witness.n{n}.{PLACES}dp"))?;
    table_row(kit, stage, i, n, &x_text, moving())
}

/// The note under the table: the gap at the crossing, detached.
pub(super) fn gap_note(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Mob> {
    let gap = fact(f, "C1.witness.gap")?;
    kit.text_left(
        stage,
        &format!("{GAP_AT_CROSSING} {gap}"),
        24.0,
        gap_color(),
        v(NOTE_X, NOTE_Y[1]),
    )
}

/// The moving point's formula under the table, detached.
pub(super) fn witness_formula(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.tex_left(stage, r"x = 2^{-1/n}", 30.0, moving(), &[], v(NOTE_X, NOTE_Y[0]))
}

/// The fixed point back on the plot: its dashed line and its dot on the
/// n-th curve, detached.
pub(super) fn fixed_back(stage: &mut Stage<'_>, n: u32) -> fmn::Result<[Mob; 2]> {
    let x0 = decimal_point(FIXED).map_err(fail)?.f64();
    let guide = PLOT.dashed(stage, (x0, 0.0), (x0, 1.0), fixed())?;
    let sunk = PLOT.dot(stage, x0, power(x0, n).map_err(fail)?, fixed());
    Ok([guide, sunk])
}

/// The card "fix x first" with the fixed points' waits, detached.
pub(super) fn card_fix_first(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Vec<Mob>> {
    let mut card = vec![
        kit.text_left(stage, FIX_FIRST, 28.0, fixed(), v(CARD_X, CARD_Y[0]))?,
        kit.text_left(
            stage,
            &format!("{GAP_BELOW} {} {FROM}", fact(f, "C1.wait.tolerance")?),
            20.0,
            Palette::dim(),
            v(CARD_X, CARD_Y[0] - CARD_STEP),
        )?,
    ];
    for (i, &point) in WAITS.iter().enumerate() {
        let shown = decimal_power(point.0, point.1, 1).map_err(fail)?;
        let wait = fact(f, &format!("C1.wait.x{shown}"))?;
        card.push(kit.text_left(
            stage,
            &format!("{shown}: {N_IS} {wait}"),
            24.0,
            Palette::ink(),
            v(CARD_X, CARD_Y[0] - CARD_STEP * (i as f64 + 2.0)),
        )?);
    }
    Ok(card)
}

/// The card "let x move with n", detached.
pub(super) fn card_move_with(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<[Mob; 3]> {
    let gap = fact(f, "C1.witness.gap")?;
    Ok([
        kit.text_left(stage, MOVE_WITH, 28.0, moving(), v(CARD_X, CARD_Y[1]))?,
        kit.tex_left(
            stage,
            r"x = 2^{-1/n}",
            28.0,
            moving(),
            &[],
            v(CARD_X, CARD_Y[1] - CARD_STEP - 0.05),
        )?,
        kit.text_left(
            stage,
            &format!("gap = {gap}, {FOR_EVERY_N}"),
            24.0,
            gap_color(),
            v(CARD_X, CARD_Y[1] - 2.0 * CARD_STEP - 0.1),
        )?,
    ])
}

impl ALimitThatJumps {
    /// 01_1 to 01_7: the curves, the fixed point, the endpoint, the
    /// prediction, the limit and why arithmetic, not the picture, proves it.
    fn the_jump(&self, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<()> {
        let kit = &self.kit;

        // 01_1: the line y = x, squared, then higher powers
        let line = say(kit, stage, "01_1")?;
        let axes = PLOT.axes(kit, stage)?;
        fade_in_all(stage, &axes, 0.6)?;
        let first = PLOT.curve(stage, POWERS[0], curve_color(), BRIGHT_W)?;
        let label = curve_label(kit, stage, POWERS[0])?;
        play!(stage;
            show_creation(first).rt(1.2),
            fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.6)
        );
        let mut b = Bright::new(first, label, POWERS[0]);
        line.wait_for(stage, SQUARE_AT)?;
        step(kit, stage, &mut b, POWERS[1], 1.0, Vec::new())?;
        line.wait_for(stage, HIGHER_AT)?;
        for &n in &POWERS[2..] {
            step(kit, stage, &mut b, n, 0.35, Vec::new())?;
        }

        // 01_2: every curve runs unbroken from (0, 0) to (1, 1)
        say(kit, stage, "01_2")?;
        let ends = [
            PLOT.dot(stage, 0.0, 0.0, Palette::ink()),
            PLOT.dot(stage, 1.0, 1.0, Palette::ink()),
        ];
        fade_in_all(stage, &ends, 0.4)?;
        let trace = PLOT.curve(stage, b.n, Palette::hero(), 6.0)?;
        play!(stage; show_creation(trace).rt(1.6));
        fade_out_all(stage, &[trace], 0.4)?;

        // 01_3: the fixed point and its heights, three of them with the words
        let line = say(kit, stage, "01_3")?;
        let x0 = decimal_point(FIXED).map_err(fail)?.f64();
        let x_text = fact(f, "C1.fixed.x")?;
        let guide = PLOT.dashed(stage, (x0, 0.0), (x0, 1.0), fixed())?;
        let foot = PLOT.p(x0, 0.0);
        let x_label = kit.text(stage, &x_text, 24.0, fixed(), v(foot[0], foot[1] - 0.32))?;
        let n_head = kit.tex(stage, "n", 30.0, Palette::dim(), &[], v(TABLE_N - 0.07, TABLE_HEAD_Y))?;
        let h_head = kit.tex_left(
            stage,
            &format!("{x_text}^{{n}}"),
            30.0,
            fixed(),
            &[],
            v(TABLE_VALUE, TABLE_HEAD_Y),
        )?;
        fade_in_all(stage, &[guide, x_label, n_head, h_head], 0.5)?;
        let mut rest = Vec::new();
        for (i, &n) in POWERS.iter().enumerate() {
            let row = table_row(kit, stage, i, n, &shown_height(f, n)?, fixed())?;
            let mark = PLOT.dot(stage, x0, power(x0, n).map_err(fail)?, fixed());
            match HEIGHT_AT.get(i) {
                Some(at) => {
                    line.wait_for(stage, at)?;
                    fade_in_all(stage, &[row[0], row[1], mark], 0.35)?;
                }
                None => rest.extend([row[0], row[1], mark]),
            }
        }
        line.wait_for(stage, DOWN_AT)?;
        fade_in_lagged(stage, &rest, 1.2, 0.15)?;

        // 01_4: the endpoint never moves
        let line = say(kit, stage, "01_4")?;
        let top = PLOT.p(1.0, 1.0);
        let ring = Circle::new()
            .radius(0.2)
            .arc_center(top)
            .style(line_style(Palette::hero(), 4.0))
            .build();
        let ring = stage.arena_mut().add(ring);
        fade_in_all(stage, &[ring], 0.3)?;
        line.wait_for(stage, ANY_POWER_AT)?;
        let one = fact(f, "C1.endpoint.height")?;
        let still = kit.tex(
            stage,
            &format!("1^{{n}} = {one}"),
            30.0,
            Palette::ink(),
            &[],
            v(0.0, 2.6),
        )?;
        fade_in_all(stage, &[still], 0.4)?;
        fade_out_all(stage, &[ring], 0.5)?;

        // 01_5: the prediction; the answer stays hidden while the ring drains
        say(kit, stage, "01_5")?;
        let ask = kit.text(stage, "?", 90.0, Palette::hero(), RING_AT)?;
        fade_in_all(stage, &[ask], 0.3)?;
        think_pause(kit, stage, RING_AT, RING_R)?;

        // 01_6: the limit, flat at zero, then a single point at height one
        let line = say(kit, stage, "01_6")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in b.copies.iter().chain([&b.curve]) {
            let dim = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .set_stroke(None, None, Some(0.35))?;
            anims.push(stage.prepare(dim)?);
        }
        for m in [ask, ends[0], ends[1]] {
            stage.clear_updaters(m, true);
            let out = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(out.rt(0.4))?);
        }
        stage.play_prepared(anims)?;
        let [flat_part, hollow, point] = PLOT.limit(stage)?;
        let below = fact(f, "C1.limit.below_one")?;
        let at_one = fact(f, "C1.limit.at_one")?;
        let def_a = kit.tex(
            stage,
            &format!(r"f(x) = {below}\ \text{{ for }}\ 0 \le x < 1"),
            30.0,
            limit_color(),
            &[],
            v(COLUMN_X, 1.9),
        )?;
        play!(stage;
            show_creation(flat_part).rt(1.2),
            fade_in(stage.arena_mut(), hollow, ORIGIN, 1.0)?.rt(1.2),
            fade_in(stage.arena_mut(), def_a, ORIGIN, 1.0)?.rt(0.8)
        );
        line.wait_for(stage, POINT_AT)?;
        let def_b = kit.tex(
            stage,
            &format!("f(1) = {at_one}"),
            30.0,
            limit_color(),
            &[],
            v(COLUMN_X, 1.3),
        )?;
        fade_in_all(stage, &[point, def_b], 0.5)?;
        line.wait_for(stage, JUMPS_AT)?;
        let jump_x = PLOT.p(1.0, 0.0)[0] + JUMP_DX;
        let jump = DashedLine::new(v(jump_x, PLOT.p(1.0, 0.0)[1]), v(jump_x, top[1]))
            .dash_length(0.1)
            .style(line_style(gap_color(), 3.0))
            .build()?;
        let jump = stage.arena_mut().add(jump);
        let jump_word = kit.text_left(stage, JUMP, 24.0, gap_color(), v(jump_x + 0.15, PLOT.p(1.0, 0.5)[1]))?;
        let end_a = kit.text(stage, AT_THE_END[0], 20.0, Palette::dim(), v(COLUMN_X, 0.85))?;
        let end_b = kit.text(stage, AT_THE_END[1], 20.0, Palette::dim(), v(COLUMN_X, 0.55))?;
        fade_in_all(stage, &[jump, jump_word, end_a, end_b], 0.5)?;

        // 01_7: a picture of eight curves proves nothing; arithmetic does
        let line = say(kit, stage, "01_7")?;
        let drawn = format!("{} {CURVES_DRAWN} {}", fact(f, "C1.curves")?, fact(f, "C1.max_power")?);
        let drawn = kit.text(stage, &drawn, 22.0, Palette::dim(), v(PLOT.frame.center[0], -2.6))?;
        fade_in_all(stage, &[drawn], 0.5)?;
        line.wait_for(stage, ARITHMETIC_AT)?;
        let step = kit.tex(
            stage,
            r"x^{n+1} = x \cdot x^{n}",
            34.0,
            Palette::ink(),
            &[],
            v(COLUMN_X, -0.2),
        )?;
        let range = kit.tex(stage, r"0 \le x < 1", 30.0, Palette::dim(), &[], v(COLUMN_X, -0.85))?;
        let each = kit.text_left(
            stage,
            &format!("{EACH_ROW} {x_text}"),
            22.0,
            fixed(),
            v(NOTE_X, NOTE_Y[0]),
        )?;
        fade_in_lagged(stage, &[step, range, each], 1.2, 0.3)?;
        kit.hold(stage)
    }

    /// 01_8: the title. Everything crossfades into it (one move, not a wipe
    /// and then a fade).
    fn title(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let leaving: Vec<Mob> = stage.scene().mobjects().to_vec();
        let t = self.kit.text(stage, TITLE, 56.0, Palette::hero(), v(0.0, 0.45))?;
        let s = self.kit.text(stage, SUBTITLE, 36.0, Palette::ink(), v(0.0, -0.4))?;
        let mut cross: Vec<Box<dyn Animation>> = Vec::new();
        for m in leaving {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            cross.push(stage.prepare(a.rt(0.6))?);
        }
        for m in [t, s] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            cross.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(cross)?;
        self.kit.listen(stage, TITLE_S, TITLE_LABEL)?;
        fade_out_all(stage, &[t, s], 0.4)
    }

    /// 01_9 to 01_11: the moving point, its gap, and a fixed point's wait.
    fn the_moving_point(&self, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<()> {
        let kit = &self.kit;
        let half = fact(f, "C1.half")?;
        let gap = fact(f, "C1.witness.gap")?;

        // 01_9: the plot again, then the half-height line and its crossing
        let line = say(kit, stage, "01_9")?;
        let mut back = PLOT.axes(kit, stage)?;
        back.extend(PLOT.limit(stage)?);
        for &n in &POWERS[1..] {
            back.push(PLOT.curve(stage, n, ghost(), GHOST_W)?);
        }
        let first = PLOT.curve(stage, POWERS[0], curve_color(), BRIGHT_W)?;
        let label = curve_label(kit, stage, POWERS[0])?;
        back.extend([first, label]);
        fade_in_all(stage, &back, 0.8)?;
        let mut b = Bright::new(first, label, POWERS[0]);
        line.wait_for(stage, HALF_AT)?;
        let [half_mark, half_label] = half_line(kit, stage)?;
        let [dot, bar] = witness_at(stage, b.n)?;
        let [n_head, x_head] = witness_head(kit, stage)?;
        let row = witness_row(kit, stage, f, 0)?;
        fade_in_all(stage, &[half_mark, half_label, dot, n_head, x_head], 0.5)?;
        fade_in_all(stage, &row, 0.35)?;

        // 01_10: the gap at the crossing, then the crossing creeps toward 1
        let line = say(kit, stage, "01_10")?;
        let note = gap_note(kit, stage, f)?;
        play!(stage;
            show_creation(bar).rt(0.8),
            fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.6)
        );
        line.wait_for(stage, GROWS_AT)?;
        for (i, &n) in POWERS.iter().enumerate().skip(1) {
            let x = half_height(n).map_err(fail)?;
            let row = witness_row(kit, stage, f, i)?;
            // the dot rides the half line as the curve it sits on morphs
            let (floor, top) = gap_span(x, n)?;
            let dot_move = move_to(dot, PLOT.p(x, top), 0.5)?;
            let bar_move = move_to(bar, PLOT.p(x, f64::midpoint(floor, top)), 0.5)?;
            let mut with: Vec<Box<dyn Animation>> = vec![stage.prepare(dot_move)?, stage.prepare(bar_move)?];
            for m in row {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                with.push(stage.prepare(a.rt(0.5))?);
            }
            step(kit, stage, &mut b, n, 0.5, with)?;
            stage.scene_mut().bring_to_front(&[bar, dot])?;
        }
        let formula = witness_formula(kit, stage)?;
        fade_in_all(stage, &[formula], 0.5)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.6)?;

        // 01_11: a fixed point waits a finite time; the moving one never gets there
        let line = say(kit, stage, "01_11")?;
        let mut ins = fixed_back(stage, b.n)?.to_vec();
        ins.extend(card_fix_first(kit, stage, f)?);
        fade_in_lagged(stage, &ins, 1.6, 0.15)?;
        line.wait_for(stage, MOVES_AT)?;
        let card_b = card_move_with(kit, stage, f)?;
        let exact = format!(
            "every curve is continuous on [0, 1], and at each fixed x the gap to f shrinks to 0. \
             But the n-th curve crosses {half} at x = 2^(−1/n) < 1, where f is 0: a gap of exactly {gap}, for every n."
        );
        let lower = precisely(kit, stage, &exact)?;
        let mut ins = card_b.to_vec();
        ins.push(lower);
        fade_in_lagged(stage, &ins, 1.2, 0.2)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        // chapter 02 opens on this picture as it rests: only the lower-third goes
        fade_out_all(stage, &[lower], 0.4)
    }
}

impl SceneConstruct for ALimitThatJumps {
    fn name(&self) -> &str {
        "01_a_limit_that_jumps"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let f = facts().map_err(fail)?;
        self.the_jump(stage, &f)?;
        self.title(stage)?;
        self.the_moving_point(stage, &f)?;
        self.kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::sentence_offsets;

    /// Every phrase a picture waits for is said once, in the line that
    /// waits for it, and the fixed point's three heights in the order the
    /// table lists them.
    #[test]
    fn the_pictures_wait_on_words_the_script_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let waits = [
            ("01_1", SQUARE_AT),
            ("01_1", HIGHER_AT),
            ("01_3", HEIGHT_AT[0]),
            ("01_3", HEIGHT_AT[1]),
            ("01_3", HEIGHT_AT[2]),
            ("01_3", DOWN_AT),
            ("01_4", ANY_POWER_AT),
            ("01_6", POINT_AT),
            ("01_6", JUMPS_AT),
            ("01_7", ARITHMETIC_AT),
            ("01_9", HALF_AT),
            ("01_10", GROWS_AT),
            ("01_11", MOVES_AT),
        ];
        for (id, phrase) in waits {
            assert_eq!(text(id).matches(phrase).count(), 1, "{id} says {phrase:?} once");
        }
        let line = text("01_3");
        let at: Vec<usize> = HEIGHT_AT.iter().map(|p| line.find(p).unwrap()).collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]) && at[2] < line.find(DOWN_AT).unwrap());
        // the opening sentences of 01_7 and 01_10 carry their pictures' cues
        let s = sentence_offsets(text("01_7"));
        assert_eq!(text("01_7")[s[1]..].find(ARITHMETIC_AT), Some(0));
        let s = sentence_offsets(text("01_10"));
        assert_eq!(text("01_10")[s[1]..].find(GROWS_AT), Some(0));
    }

    #[test]
    fn the_title_window_is_the_storyboards() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let beat = board[0].beats.iter().find(|b| b.id == "01_8").unwrap();
        assert_eq!(beat.listen_s(), Some(TITLE_S));
        assert!(beat.spoken().is_none(), "the title is silent");
        assert_eq!(TITLE_LABEL, format!("{}:{}", board[0].key, beat.id));
    }

    /// The table and its notes sit right of the plot, the table's last row
    /// above the notes, and the jump's marker and word between the plot and
    /// the table (the rest of the layout is checked at compile time).
    #[test]
    fn the_layout_holds() {
        let plot_right = PLOT.frame.right() + 0.5;
        // the n column is right-aligned at TABLE_N; its widest entry ("50")
        // reaches about 0.5 to the left of it
        let table_left = TABLE_N - 0.5;
        assert!(table_left > plot_right, "the table's n column clears the plot");
        assert!(NOTE_X > plot_right, "the notes under the table clear the plot");
        assert!(row_y(POWERS.len() - 1) > NOTE_Y[0] + 0.3);
        assert!(PLOT.frame.right() + JUMP_DX + 0.15 + 0.7 < TABLE_N - 0.3);
    }
}
