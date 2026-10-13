//! Chapter 10, near fifty: Bethe's way, seen. On the square of forty-eight
//! inside the outline of fifty, where chapter 09 leaves them, a strip two
//! deep comes off the top and stands against the side (10_1, the one move);
//! the rectangle is as wide as fifty squared and four rows short, two
//! hundred under twenty-five hundred, which is Bethe's twenty-three hundred
//! (10_2); the end of the strip sticks up, two by two, and is added (10_3).
//! Above fifty (10_4) the same move on fifty-three (10_5) leaves a
//! rectangle fifty tall and six columns wider than fifty squared, three
//! hundred more, and a small square of nine (10_6). Both give the quick way
//! to say it: twenty-five plus or minus the distance for the hundreds, then
//! the distance squared as two digits (10_7).

use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::draw::{
    Board, Lens, MoveFigure, NOTE_PT, SIDE_PT, Side, corner, cut_style, eq, fade_in_all, fade_in_lagged, fade_out_all,
    fail, fifty_board, flat, glyphs, gone, hundreds, line_style, lower_third_tex, middle_of, num, rgb, say, slate,
    strip, swap, why_card, why_pause,
};
use crate::facts::shown;
use crate::model::{FIFTY, OneMove, near_fifty, one_move};

pub struct NearFifty {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const WHAT_CHANGES: &str = "what changes above";
const THE_HUNDREDS: &str = "the hundreds";
const PRECISELY: &str = "precisely:";
/// The precise line's words, set inside its TeX.
const B_IS: &str = r"b = \text{the distance from }";
const SIGNS: &str = r"\text{: } + \text{ above, } - \text{ below}";

/// The sums in the column, left-aligned beside the figure, one row each.
const COL_LEFT: f64 = 2.0;
const COL_ROOM: f64 = 4.85;
const ROW_Y: [f64; 6] = [3.2, 2.2, 1.2, 0.2, -0.8, -1.8];
const SUM_PT: f64 = 60.0;
/// Fifty squared, small, at the head of the column (as chapter 09 leaves it).
const HEAD_PT: f64 = 48.0;
/// The band the rectangle falls short of fifty squared by, and the columns
/// it runs past it by: a translucent red, and a light over the blue.
const BAND_OPACITY: f64 = 0.45;
const LIT_OPACITY: f64 = 0.28;
/// The framed enlargement of the small square: its middle, its zoom, and
/// how many units of the board it shows each way.
const LENS_AT: Vec3 = [0.2, 1.9, 0.0];
const LENS_ZOOM: f64 = 3.5;
const LENS_SPAN: f64 = 8.0;
/// Where the question with no countdown sits.
const WHY_AT: Vec3 = [3.6, 0.1, 0.0];
/// The two rows of 10_7: their heights, the square each works, the two
/// steps and the answer read as two pairs of digits.
const PAIR_ROWS_Y: [f64; 2] = [1.1, -1.0];
const ROW_LABEL_X: f64 = -5.6;
const STEP_ONE_LEFT: f64 = -4.45;
const STEP_TWO_LEFT: f64 = -0.1;
const PAIR_X: f64 = 5.0;
const ROW_PT: f64 = 64.0;
const PAIR_PT: f64 = 96.0;

/// How far round the hundreds of 10_7 swing on their way to the answer
/// (radians), so they pass over or under the step between.
const HUNDREDS_ARC: f64 = 1.1;

/// Move a mobject so its centre is at `to` along an arc of `arc` radians
/// (positive bends below a rightward path, negative above), scaling it by
/// `scale` on the way, `rt` seconds.
fn arc_fly(mob: Mob, to: Vec3, scale: f64, rt: f64, arc: f64) -> fmn::Result<AnimBuilder> {
    let a = mob.animate().set_anim_args(AnimateArgs {
        run_time: Some(rt),
        path_arc: (arc != 0.0).then_some(arc),
        ..AnimateArgs::default()
    })?;
    Ok(a.scale(scale)?.move_to(to, ORIGIN)?)
}

/// The one move on `number`: a strip as deep as its distance from fifty.
fn move_of(number: u64) -> fmn::Result<OneMove> {
    let deep = number.abs_diff(FIFTY);
    one_move(number, deep).ok_or_else(|| fail(format!("no strip {deep} deep comes off a square of {number}")))
}

/// A part of a line of mathematics and its colour.
fn part(src: impl Into<String>, color: Srgb) -> (String, Srgb) {
    (src.into(), color)
}

/// A line of mathematics set whole, then cut into its parts: one TeX
/// mobject per part, in its own colour and exactly where it sits in the
/// whole line, so that a part can come in, flash, fly or be replaced on its
/// own. The line stands with its `edge` (LEFT, or ORIGIN for its middle) at
/// `at` and is no wider than `room` (smaller type if need be). Detached.
fn line_parts(
    kit: &Kit,
    stage: &mut Stage<'_>,
    parts: &[(String, Srgb)],
    size: f64,
    at: Vec3,
    edge: Vec3,
    room: f64,
) -> fmn::Result<Vec<Mob>> {
    let src = parts.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>().join(" ");
    let mut size = size;
    let mut whole = eq(kit, stage, &src, size, at)?;
    let width = stage.get_width(whole);
    if width > room {
        size *= room / width;
        whole = eq(kit, stage, &src, size, at)?;
    }
    stage.move_to(whole, at, edge);
    let all = glyphs(stage, whole);
    let mut out = Vec::with_capacity(parts.len());
    let mut next = 0;
    for (s, color) in parts {
        let m = kit.tex(stage, s, size, *color, &[], at)?;
        let n = glyphs(stage, m).len();
        let span = all
            .get(next..next + n)
            .filter(|g| !g.is_empty())
            .ok_or_else(|| fail(format!("{s:?} has no place in {src:?}")))?;
        let middle = middle_of(stage, span)?;
        stage.move_to(m, middle, ORIGIN);
        out.push(m);
        next += n;
    }
    if next != all.len() {
        return Err(fail(format!("the parts of {src:?} do not add up to it")));
    }
    Ok(out)
}

/// A row of the column beside the figure.
fn row(kit: &Kit, stage: &mut Stage<'_>, parts: &[(String, Srgb)], y: f64) -> fmn::Result<Vec<Mob>> {
    line_parts(kit, stage, parts, SUM_PT, v(COL_LEFT, y), LEFT, COL_ROOM)
}

/// The outline of fifty squared back on top of the fills just brought in.
fn outline_on_top(stage: &mut Stage<'_>, outline: Mob) -> fmn::Result<()> {
    stage.scene_mut().bring_to_front(&[outline])?;
    Ok(())
}

/// The rectangle the move made becomes one piece in the hundreds' colour,
/// as `MoveFigure::merge` does; that one also brings the small square to the
/// front, which would show it before its words, so this one only swaps the
/// seam for the merged piece.
fn merge(stage: &mut Stage<'_>, figure: &MoveFigure, also: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut ins = vec![figure.merged];
    ins.extend(also);
    swap(stage, &[figure.seam], &ins, rt)
}

/// The window the magnifier enlarges: [`LENS_SPAN`] units each way, centred
/// between the small square and the corner of the outline of fifty, so both
/// show.
fn lens_window(figure: &MoveFigure) -> [f64; 4] {
    let [x, y, w, h] = figure.corner_rect();
    let fifty = FIFTY as f64;
    let (cx, cy) = ((x + w / 2.0 + fifty) / 2.0, (y + h / 2.0 + fifty) / 2.0);
    [cx - LENS_SPAN / 2.0, cy - LENS_SPAN / 2.0, LENS_SPAN, LENS_SPAN]
}

/// The part of the straight line from `a` to `b` (level or upright) inside
/// `window`, if any.
fn clip_line(window: [f64; 4], a: [f64; 2], b: [f64; 2]) -> Option<([f64; 2], [f64; 2])> {
    let [x, y, w, h] = window;
    let (x0, x1) = (a[0].min(b[0]).max(x), a[0].max(b[0]).min(x + w));
    let (y0, y1) = (a[1].min(b[1]).max(y), a[1].max(b[1]).min(y + h));
    (x0 <= x1 && y0 <= y1 && (x1 - x0) + (y1 - y0) > 1e-9).then_some(([x0, y0], [x1, y1]))
}

/// The magnifier with the picture near the small square drawn again inside
/// it: each fill clipped to the window, then each line; everything back to
/// front, the frame last. Detached.
fn magnify(
    stage: &mut Stage<'_>,
    source: Board,
    window: [f64; 4],
    fills: &[([f64; 4], Style)],
    lines: &[([f64; 2], [f64; 2], Style)],
) -> Vec<Mob> {
    let lens = Lens::build(stage, source, window, LENS_AT, LENS_ZOOM);
    let mut out = vec![lens.mark, lens.rays[0], lens.rays[1], lens.back];
    for &(r, style) in fills {
        if let Some(r) = lens.clip(r) {
            out.push(lens.board.rect(stage, r, style));
        }
    }
    for &(a, b, style) in lines {
        if let Some((a, b)) = clip_line(window, a, b) {
            out.push(lens.board.line(stage, a, b, style));
        }
    }
    out.push(lens.frame);
    out
}

/// Where a note on the magnifier sits: under its frame.
fn lens_note_at() -> Vec3 {
    let span = LENS_SPAN * fifty_board().unit * LENS_ZOOM;
    v(LENS_AT[0], LENS_AT[1] - span / 2.0 - 0.32)
}

/// The two edges of the outline of fifty that run near the small square,
/// for the magnifier.
fn outline_edges() -> [([f64; 2], [f64; 2], Style); 2] {
    let (fifty, edge) = (FIFTY as f64, line_style(Palette::ink(), 2.5));
    [
        ([0.0, fifty], [fifty, fifty], edge),
        ([fifty, 0.0], [fifty, fifty], edge),
    ]
}

impl SceneConstruct for NearFifty {
    fn name(&self) -> &str {
        "10_near_fifty"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (blue, green, orange, red, ink) = (hundreds(), strip(), corner(), gone(), Palette::ink());
        let fifty = FIFTY as f64;
        let board = fifty_board();
        // fifty squared is twenty-five hundred: the distance there is nothing
        let base = near_fifty(FIFTY);
        let below = near_fifty(shown::BELOW);
        let above = near_fifty(shown::ABOVE);
        let (m48, m53) = (move_of(shown::BELOW)?, move_of(shown::ABOVE)?);
        // below fifty the rectangle is short of fifty squared by rows of
        // fifty; above, it runs past it by columns of fifty
        let rows = FIFTY - m48.tall();
        let cols = m53.wide() - FIFTY;
        if rows != 2 * below.distance.unsigned_abs()
            || base.square - rows * FIFTY != below.rough
            || below.rough != m48.rectangle()
            || below.rough + m48.corner() != below.square
        {
            return Err(fail(format!(
                "the picture of {} does not add up: {below:?}",
                shown::BELOW
            )));
        }
        if cols != 2 * above.distance.unsigned_abs()
            || m53.tall() != FIFTY
            || base.square + cols * FIFTY + m53.corner() != above.square
        {
            return Err(fail(format!(
                "the picture of {} does not add up: {above:?}",
                shown::ABOVE
            )));
        }
        let times = r"\times";

        // where chapter 09 leaves the stage, in the scene from the first
        // frame: the outline of fifty, the square of forty-eight inside it
        // on the same corner (one plain square), fifty squared at the head
        // of the column
        let outline = board.rect(stage, [0.0, 0.0, fifty, fifty], line_style(ink, 2.5));
        let figure = MoveFigure::build(stage, board, m48, slate(), slate(), 0.0)?;
        let head = eq(
            &kit,
            stage,
            &format!("{FIFTY}^2 = {}", base.square),
            HEAD_PT,
            v(COL_LEFT, ROW_Y[0]),
        )?;
        stage.move_to(head, v(COL_LEFT, ROW_Y[0]), LEFT);
        for m in figure.square() {
            stage.add_to_scene(m)?;
        }
        stage.add_to_scene(outline)?;
        stage.add_to_scene(head)?;

        // 10_1: the one move, on forty-eight
        let line = say(&kit, stage, "10_1")?;
        line.wait_sentence(stage, 1)?;
        let side_words = num(m48.side);
        let square48 = figure.square_rect();
        let left = board.label(&kit, stage, &side_words, square48, Side::Left, SIDE_PT, ink)?;
        let under = board.label(&kit, stage, &side_words, square48, Side::Below, SIDE_PT, ink)?;
        let answer48 = row(
            &kit,
            stage,
            &[
                part(format!("{}^2", m48.side), ink),
                part("=", ink),
                part(num(below.square), ink),
            ],
            ROW_Y[1],
        )?;
        fade_in_all(stage, &[left, under, answer48[0]], 0.6)?;
        line.wait_sentence(stage, 2)?;
        figure.slice(stage, 0.8)?;
        outline_on_top(stage, outline)?;
        let deep = board.label(
            &kit,
            stage,
            &num(m48.strip),
            figure.top_rect(),
            Side::Right,
            SIDE_PT,
            green,
        )?;
        play!(stage; fade_in(stage.arena_mut(), deep, ORIGIN, 1.0)?.rt(0.4));
        line.wait_for(stage, "and stand it")?;
        fade_out_all(stage, &[left, under, deep], 0.4)?;
        figure.play(&kit, stage, "10_1")?;
        outline_on_top(stage, outline)?;

        // 10_2: as wide as fifty squared and four rows short: two hundred
        // under twenty-five hundred, Bethe's twenty-three hundred
        let line = say(&kit, stage, "10_2")?;
        let rect48 = figure.rectangle_rect();
        let wide = board.label(&kit, stage, &num(m48.wide()), rect48, Side::Below, SIDE_PT, ink)?;
        play!(stage; fade_in(stage.arena_mut(), wide, ORIGIN, 1.0)?.rt(0.5));
        line.wait_for(stage, "four rows short")?;
        let band_rect = [0.0, m48.tall() as f64, fifty, rows as f64];
        let band = board.rect(stage, band_rect, flat(red, BAND_OPACITY));
        let tall = board.label(&kit, stage, &num(m48.tall()), rect48, Side::Left, SIDE_PT, ink)?;
        // on the left, out of the way of the magnifier's mark at the right
        let band_label = board.label(&kit, stage, &num(rows), band_rect, Side::Left, SIDE_PT, red)?;
        fade_in_all(stage, &[band, tall, band_label], 0.6)?;
        outline_on_top(stage, outline)?;
        line.wait_sentence(stage, 1)?;
        let fifties = row(
            &kit,
            stage,
            &[
                part(num(rows), ink),
                part(times, ink),
                part(num(FIFTY), ink),
                part("=", ink),
                part(num(rows * FIFTY), red),
            ],
            ROW_Y[2],
        )?;
        fade_in_all(stage, &fifties, 0.5)?;
        line.wait_sentence(stage, 2)?;
        let rough = row(
            &kit,
            stage,
            &[
                part(num(base.square), ink),
                part("-", ink),
                part(num(rows * FIFTY), red),
                part("=", ink),
                part(num(below.rough), blue),
            ],
            ROW_Y[3],
        )?;
        merge(stage, &figure, &rough, 0.6)?;
        outline_on_top(stage, outline)?;

        // 10_3: the end of the strip sticks up, two by two, and is added
        let line = say(&kit, stage, "10_3")?;
        figure.show_corner(stage, 0.6)?;
        outline_on_top(stage, outline)?;
        let [ox, oy] = [figure.corner_rect()[0], figure.corner_rect()[1]];
        let lens48 = magnify(
            stage,
            board,
            lens_window(&figure),
            &[
                (rect48, flat(blue, 1.0)),
                (band_rect, flat(red, BAND_OPACITY)),
                (figure.corner_rect(), flat(orange, 1.0)),
            ],
            &[
                ([ox, oy], [ox + m48.strip as f64, oy], cut_style()),
                outline_edges()[0],
                outline_edges()[1],
            ],
        );
        fade_in_all(stage, &lens48, 0.7)?;
        line.wait_sentence(stage, 1)?;
        let note48 = kit.text(stage, &format!("{0} × {0}", m48.strip), NOTE_PT, orange, lens_note_at())?;
        let small48 = row(
            &kit,
            stage,
            &[
                part(num(m48.strip), ink),
                part(times, ink),
                part(num(m48.strip), ink),
                part("=", ink),
                part(num(m48.corner()), orange),
            ],
            ROW_Y[4],
        )?;
        let mut ins = vec![note48];
        ins.extend(&small48);
        fade_in_all(stage, &ins, 0.5)?;
        line.wait_sentence(stage, 2)?;
        let exact48 = row(
            &kit,
            stage,
            &[
                part(num(below.rough), blue),
                part("+", ink),
                part(num(m48.corner()), orange),
                part("=", ink),
                part(num(below.square), ink),
            ],
            ROW_Y[5],
        )?;
        let mut ins = exact48.clone();
        ins.extend(&answer48[1..]);
        fade_in_all(stage, &ins, 0.6)?;

        // 10_4: above fifty; the square of fifty-three over the outline,
        // sticking out past it on top and at the side
        let line = say(&kit, stage, "10_4")?;
        let mut gone48 = vec![
            figure.body,
            figure.strip.anchor,
            figure.strip.fill,
            figure.strip.edge,
            figure.merged,
            figure.corner,
            figure.corner_cut,
            band,
            wide,
            tall,
            band_label,
            note48,
        ];
        for list in [&lens48, &fifties, &rough, &small48, &exact48, &answer48] {
            gone48.extend(list.iter());
        }
        fade_out_all(stage, &gone48, 0.6)?;
        let figure53 = MoveFigure::build(stage, board, m53, slate(), slate(), 0.0)?;
        let square53 = figure53.square_rect();
        let words53 = num(m53.side);
        let left53 = board.label(&kit, stage, &words53, square53, Side::Left, SIDE_PT, ink)?;
        let under53 = board.label(&kit, stage, &words53, square53, Side::Below, SIDE_PT, ink)?;
        let square_of53 = format!("{}^2", m53.side);
        let question = row(
            &kit,
            stage,
            &[part(&square_of53, ink), part("=", ink), part(r"\;?", ink)],
            ROW_Y[1],
        )?;
        let answer53 = row(
            &kit,
            stage,
            &[part(&square_of53, ink), part("=", ink), part(num(above.square), ink)],
            ROW_Y[1],
        )?;
        let mut ins = figure53.square();
        ins.extend([left53, under53]);
        fade_in_all(stage, &ins, 0.7)?;
        outline_on_top(stage, outline)?;
        line.wait_sentence(stage, 1)?;
        let why = why_card(&kit, stage, &format!("{WHAT_CHANGES} {FIFTY}?"), WHY_AT)?;
        let mut ins = question.clone();
        ins.push(why);
        fade_in_all(stage, &ins, 0.5)?;
        why_pause(&kit, stage)?;

        // 10_5: the same move, three deep
        let line = say(&kit, stage, "10_5")?;
        play!(stage; fade_out(stage.arena_mut(), why, ORIGIN, 1.0)?.rt(0.4));
        line.wait_sentence(stage, 1)?;
        figure53.slice(stage, 0.8)?;
        outline_on_top(stage, outline)?;
        let deep53 = board.label(
            &kit,
            stage,
            &num(m53.strip),
            figure53.top_rect(),
            Side::Right,
            SIDE_PT,
            green,
        )?;
        play!(stage; fade_in(stage.arena_mut(), deep53, ORIGIN, 1.0)?.rt(0.4));
        line.wait_for(stage, "and stand it")?;
        fade_out_all(stage, &[left53, under53, deep53], 0.4)?;
        figure53.play(&kit, stage, "10_5")?;
        outline_on_top(stage, outline)?;

        // 10_6: fifty tall and six columns wider: three hundred over
        // twenty-five hundred, and the small square of nine
        let line = say(&kit, stage, "10_6")?;
        let rect53 = figure53.rectangle_rect();
        let tall53 = board.label(&kit, stage, &num(m53.tall()), rect53, Side::Left, SIDE_PT, ink)?;
        merge(stage, &figure53, &[tall53], 0.5)?;
        outline_on_top(stage, outline)?;
        line.wait_for(stage, "and six columns")?;
        let lit_rect = [fifty, 0.0, cols as f64, fifty];
        let lit = board.rect(stage, lit_rect, flat(ink, LIT_OPACITY));
        // the width in its two parts, fifty under the outline and six under
        // the columns past it (one label for all 56 under the whole read as
        // 56 + 6: R2, M1)
        let fifty_rect = [0.0, 0.0, fifty, fifty];
        let wide53 = board.label(&kit, stage, &num(FIFTY), fifty_rect, Side::Below, SIDE_PT, ink)?;
        let cols_label = board.label(&kit, stage, &num(cols), lit_rect, Side::Below, SIDE_PT, ink)?;
        fade_in_all(stage, &[lit, wide53, cols_label], 0.6)?;
        outline_on_top(stage, outline)?;
        line.wait_sentence(stage, 1)?;
        let fifties53 = row(
            &kit,
            stage,
            &[
                part(num(cols), ink),
                part(times, ink),
                part(num(FIFTY), ink),
                part("=", ink),
                part(num(cols * FIFTY), blue),
            ],
            ROW_Y[2],
        )?;
        fade_in_all(stage, &fifties53, 0.5)?;
        line.wait_sentence(stage, 2)?;
        figure53.show_corner(stage, 0.6)?;
        outline_on_top(stage, outline)?;
        let [ox, oy] = [figure53.corner_rect()[0], figure53.corner_rect()[1]];
        let lens53 = magnify(
            stage,
            board,
            lens_window(&figure53),
            &[
                (rect53, flat(blue, 1.0)),
                (lit_rect, flat(ink, LIT_OPACITY)),
                (figure53.corner_rect(), flat(orange, 1.0)),
            ],
            &[
                ([ox, oy], [ox + m53.strip as f64, oy], cut_style()),
                outline_edges()[0],
                outline_edges()[1],
            ],
        );
        fade_in_all(stage, &lens53, 0.7)?;
        line.wait_sentence(stage, 3)?;
        let note53 = kit.text(stage, &format!("{0} × {0}", m53.strip), NOTE_PT, orange, lens_note_at())?;
        let small53 = row(
            &kit,
            stage,
            &[
                part(num(m53.strip), ink),
                part(times, ink),
                part(num(m53.strip), ink),
                part("=", ink),
                part(num(m53.corner()), orange),
            ],
            ROW_Y[3],
        )?;
        let mut ins = vec![note53];
        ins.extend(&small53);
        fade_in_all(stage, &ins, 0.5)?;
        line.wait_sentence(stage, 4)?;
        let exact53 = row(
            &kit,
            stage,
            &[
                part(num(base.square), ink),
                part("+", ink),
                part(num(cols * FIFTY), blue),
                part("+", ink),
                part(num(m53.corner()), orange),
                part("=", ink),
                part(num(above.square), ink),
            ],
            ROW_Y[4],
        )?;
        let mut ins = exact53.clone();
        ins.push(answer53[2]);
        swap(stage, &[question[2]], &ins, 0.6)?;

        // 10_7: the quick way to say it, in two rows: twenty-five, plus or
        // minus the distance, for the hundreds; then the distance squared
        // as two digits
        let line = say(&kit, stage, "10_7")?;
        fade_all(stage, 0.6)?;
        let start = base.hundreds;
        let mut labels = Vec::new();
        let mut firsts = Vec::new();
        let mut step_ones = Vec::new();
        let mut step_twos = Vec::new();
        // each row's answer as two pairs: where they land, and their digits
        let mut pairs = Vec::new();
        let mut pair_words = Vec::new();
        for (n, y) in [(above, PAIR_ROWS_Y[0]), (below, PAIR_ROWS_Y[1])] {
            let (front, back) = n
                .digit_pairs()
                .ok_or_else(|| fail(format!("{}'s small square runs past two digits", n.number)))?;
            pair_words.push([front.clone(), back.clone()]);
            let d = n.distance.unsigned_abs();
            let sign = if n.distance >= 0 { "+" } else { "-" };
            labels.push(eq(&kit, stage, &format!("{}^2", n.number), ROW_PT, v(ROW_LABEL_X, y))?);
            let one = line_parts(
                &kit,
                stage,
                &[
                    part(start.to_string(), ink),
                    part(sign, ink),
                    part(num(d), ink),
                    part("=", ink),
                    part(&front, blue),
                ],
                ROW_PT,
                v(STEP_ONE_LEFT, y),
                LEFT,
                f64::INFINITY,
            )?;
            let two = line_parts(
                &kit,
                stage,
                &[
                    part(format!("{d}^2"), ink),
                    part("=", ink),
                    part(num(n.corner), ink),
                    part(r"\to", ink),
                    part(&back, orange),
                ],
                ROW_PT,
                v(STEP_TWO_LEFT, y),
                LEFT,
                f64::INFINITY,
            )?;
            // the answer, read as two pairs with a gap between them
            let gap = 0.16;
            let lead = kit.tex(stage, &front, PAIR_PT, blue, &[], v(PAIR_X, y))?;
            stage.move_to(lead, v(PAIR_X - gap, y), RIGHT);
            let tail = kit.tex(stage, &back, PAIR_PT, orange, &[], v(PAIR_X, y))?;
            stage.move_to(tail, v(PAIR_X + gap, y), LEFT);
            firsts.push(one[0]);
            step_ones.push(one);
            step_twos.push(two);
            pairs.push([lead, tail]);
        }
        fade_in_all(stage, &labels, 0.5)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &firsts, 0.5)?;
        line.wait_sentence(stage, 2)?;
        fade_in_all(stage, &step_ones[0][1..], 0.6)?;
        line.wait_sentence(stage, 3)?;
        fade_in_all(stage, &step_ones[1][1..], 0.6)?;
        line.wait_sentence(stage, 4)?;
        let hundreds_at = stage.get_center(step_ones[0][4]);
        let named = kit.text(
            stage,
            THE_HUNDREDS,
            NOTE_PT,
            blue,
            v(hundreds_at[0], hundreds_at[1] + 0.72),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), named, v(0.0, -0.1), 1.0)?.rt(0.5),
            indicate(stage.arena_mut(), step_ones[0][4], 1.25, Some(rgb(blue)))?.rt(0.8),
            indicate(stage.arena_mut(), step_ones[1][4], 1.25, Some(rgb(blue)))?.rt(0.8),
        );
        line.wait_sentence(stage, 5)?;
        let mut seconds = step_twos[0].clone();
        seconds.extend(&step_twos[1]);
        fade_in_lagged(stage, &seconds, 1.2, 0.08)?;
        line.wait_sentence(stage, 6)?;
        play!(stage; indicate(stage.arena_mut(), step_twos[0][4], 1.3, Some(rgb(orange)))?.rt(0.6));
        // each row's two pairs fly to the end of the row: its answer. The
        // hundreds arc over the top row and under the bottom one, clear of
        // the digits they pass
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let a = fade_out(stage.arena_mut(), named, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.4))?);
        for k in 0..pairs.len() {
            let froms = [step_ones[k][4], step_twos[k][4]];
            let over = if k == 0 { -HUNDREDS_ARC } else { HUNDREDS_ARC };
            for (j, (color, arc)) in [(blue, over), (orange, 0.0)].into_iter().enumerate() {
                let (from, to) = (stage.get_center(froms[j]), stage.get_center(pairs[k][j]));
                let copy = kit.tex(stage, &pair_words[k][j], ROW_PT, color, &[], from)?;
                stage.add_to_scene(copy)?;
                let flight = arc_fly(copy, to, PAIR_PT / ROW_PT, 1.0, arc)?;
                anims.push(stage.prepare(flight)?);
            }
        }
        stage.play_prepared(anims)?;
        let signs = format!(
            r"({FIFTY} \pm b)^2 = {} \pm {}b + b^2,\quad {B_IS}{FIFTY}{SIGNS}",
            base.square,
            2 * FIFTY
        );
        let precise = lower_third_tex(&kit, stage, PRECISELY, &signs)?;
        play!(stage; fade_in(stage.arena_mut(), precise, v(0.0, 0.1), 1.0)?.rt(0.5));
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
