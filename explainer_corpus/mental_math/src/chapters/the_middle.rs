//! Chapter 08, the middle: forty-seven times fifty-three, and the number
//! halfway between them (08_1); fifty, three either side (08_2); the number
//! line becomes the picture, a square of fifty with the rectangle laid over
//! it, a strip sticking out at the side and a gap along the top (08_3); the
//! one move, backwards: the strip lies down in the gap and leaves a hole
//! three by three (08_4); fifty squared less three squared (08_5); a quiz
//! with a middle of twenty, worked with the move in small (08_6, 08_7); and
//! what the trick needs: a whole middle, and a distance easy to square
//! (08_8).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::animation::{grow_from_center, replacement_transform};
use fmn::prelude::*;

use super::draw::{
    Board, Lens, NOTE_PT, NumberLine1, QuizCard, QuizLook, SIDE_PT, Side, Strip, Turn, cut_style, eq, fade_in_all,
    fade_out_all, fail, fifty_board, fit_width, flat, fly, glyphs, gone, grey_line, hundreds, line_style,
    lower_third_tex, middle_of, num, quiz_at, repaint, say, strip, swap, why_card, why_pause,
};
use crate::facts::shown;
use crate::model::{Ask, Pair, pair};

pub struct TheMiddle {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HALFWAY: &str = "what sits halfway between";
const AND: &str = "and";
const THE_MIDDLE: &str = "the middle";
const THE_DISTANCE: &str = "the distance";
const SQUARES: &str = "the difference of squares:";
const TRUE_NO_EASIER: &str = r"\text{true, and no easier}";
const WHOLE_MIDDLE: &str = "a whole middle needs both odd or both even";

/// The problem over its number line (08_1, 08_8), and the line itself.
const PROBLEM_AT: Vec3 = [0.0, 2.3, 0.0];
const PROBLEM_PT: f64 = 110.0;
const LINE_Y: f64 = 0.1;
const LINE_X: (f64, f64) = (-5.5, 5.5);
/// How high the bows of the distance rise over the line.
const BOW_LIFT: f64 = 0.9;
/// The column of the working beside the figure: the problem at its head,
/// then the rows, their left edges lined up.
const HEAD_LEFT: f64 = 1.9;
const ROWS_LEFT: f64 = 2.35;
const ROW_YS: [f64; 4] = [2.9, 1.75, 0.6, -0.65];
const ROW_PT: f64 = 64.0;
const ANSWER_PT: f64 = 90.0;
/// The gap's faint red, under the strip once it lies in it.
const GAP_OPACITY: f64 = 0.35;
/// The magnifier over the corner: its middle, and how wide it is.
const LENS_AT: Vec3 = [-0.05, 1.3, 0.0];
const LENS_SIZE: f64 = 3.0;
/// The quiz card shares the screen with its working (08_6, 08_7).
const CARD_AT: Vec3 = [3.8, 1.05, 0.0];
const CARD_SCALE: f64 = 0.75;
const WORK_YS: [f64; 2] = [-1.55, -2.65];
const WORK_PT: f64 = 56.0;
/// The quiz's own number line, and the move replayed small under it.
const SMALL_LINE: (f64, f64, f64) = (-6.3, -1.3, 2.5);
const SMALL_AT: Vec3 = [-5.9, -2.9, 0.0];
const SMALL_UNIT: f64 = 0.14;
/// The grey aside of 08_8, over the lower-third.
const ASIDE_Y: f64 = -2.25;

/// The two numbers `(x, y)` worked from their middle, or a scene error.
fn pair_of((x, y): (u64, u64)) -> fmn::Result<Pair> {
    pair(x, y).ok_or_else(|| fail(format!("{x} and {y} have no whole middle")))
}

/// A bow from `a` to `b` that bulges `out` to the left of the way from `a`
/// to `b` (a measure along a side: up the left of a gap, under a strip).
/// Detached.
fn bow_across(stage: &mut Stage<'_>, a: Vec3, b: Vec3, out: f64, style: Style) -> Mob {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy).max(1e-9);
    let (nx, ny) = (-dy / len, dx / len);
    let points: Vec<Vec3> = (0..=24)
        .map(|k| {
            let t = f64::from(k) / 24.0;
            let h = out * 4.0 * t * (1.0 - t);
            v(a[0] + dx * t + nx * h, a[1] + dy * t + ny * h)
        })
        .collect();
    stage.arena_mut().add(polyline(points).style(style).build())
}

/// The faint tick at every whole number of `line`. Detached.
fn unit_ticks(stage: &mut Stage<'_>, line: &NumberLine1) -> Vec<Mob> {
    let mut out = Vec::new();
    let mut k = line.lo.ceil();
    while k <= line.hi + 1e-9 {
        out.push(line.tick(stage, k, 0.16, Palette::faint()));
        k += 1.0;
    }
    out
}

/// A number set over (`dy` > 0) or under the line at `value`. Detached.
#[allow(clippy::too_many_arguments)]
fn mark(
    kit: &Kit,
    stage: &mut Stage<'_>,
    line: &NumberLine1,
    value: f64,
    src: &str,
    size: f64,
    color: Srgb,
    dy: f64,
) -> fmn::Result<Mob> {
    let p = line.p(value);
    kit.tex(stage, src, size, color, &[], v(p[0], p[1] + dy))
}

/// Colour glyphs `from..to` of a typeset formula.
fn tint(stage: &mut Stage<'_>, tex: Mob, from: usize, to: usize, color: Srgb) -> fmn::Result<()> {
    let all = glyphs(stage, tex);
    if to > all.len() || from > to {
        return Err(fail(format!(
            "a formula of {} glyphs has no glyphs {from}..{to}",
            all.len()
        )));
    }
    for &g in &all[from..to] {
        stage.set_color(g, color, None, true);
    }
    Ok(())
}

/// A line of the column, its left edge at `left`.
fn row(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, left: f64, y: f64) -> fmn::Result<Mob> {
    kit.tex_left(stage, src, size, Palette::ink(), &[], v(left, y))
}

impl SceneConstruct for TheMiddle {
    fn name(&self) -> &str {
        "08_the_middle"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = pair_of(shown::PAIR)?;
        let (low, high, mid, d) = (p.low as f64, p.high as f64, p.middle as f64, p.distance as f64);
        let ink = Palette::ink();
        let number_line = NumberLine1 {
            x0: LINE_X.0,
            x1: LINE_X.1,
            y: LINE_Y,
            lo: low - d,
            hi: high + d,
        };

        // 08_1: forty-seven times fifty-three; the two numbers drop on to a
        // number line, with nothing between them yet; what sits halfway?
        // (the problem is up before the line's first words: the line starts
        // 0.6 to 1.0 s in, the gap before it follows the thought)
        let problem = eq(
            &kit,
            stage,
            &format!(r"{} \times {}", p.low, p.high),
            PROBLEM_PT,
            PROBLEM_AT,
        )?;
        play!(stage; fade_in(stage.arena_mut(), problem, v(0.0, -0.15), 1.0)?.rt(0.5));
        let line = say(&kit, stage, "08_1")?;
        line.wait_sentence(stage, 1)?;
        let axis = number_line.axis(stage);
        let ticks = unit_ticks(stage, &number_line);
        let mut on_line = vec![axis];
        on_line.extend(&ticks);
        fade_in_all(stage, &on_line, 0.5)?;
        // the two numbers of the problem drop on to the line, where they
        // stand
        let all = glyphs(stage, problem);
        let (low_len, high_len) = (num(p.low).len(), num(p.high).len());
        if all.len() != low_len + 1 + high_len {
            return Err(fail(format!("{} × {} typeset as {} glyphs", p.low, p.high, all.len())));
        }
        let low_from = middle_of(stage, &all[..low_len])?;
        let high_from = middle_of(stage, &all[low_len + 1..])?;
        let low_mark = eq(&kit, stage, &num(p.low), PROBLEM_PT, low_from)?;
        let high_mark = eq(&kit, stage, &num(p.high), PROBLEM_PT, high_from)?;
        let under = |x: f64| v(number_line.p(x)[0], LINE_Y - 0.42);
        let dots = [
            number_line.dot(stage, low, 0.09, ink),
            number_line.dot(stage, high, 0.09, ink),
        ];
        stage.add_to_scene(low_mark)?;
        stage.add_to_scene(high_mark)?;
        play!(stage;
            fly(low_mark, under(low), SIDE_PT / PROBLEM_PT, 0.9)?,
            fly(high_mark, under(high), SIDE_PT / PROBLEM_PT, 0.9)?,
            fade_in(stage.arena_mut(), dots[0], ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), dots[1], ORIGIN, 1.0)?.rt(0.9),
        );
        line.wait_sentence(stage, 2)?;
        let question = format!("{HALFWAY} {} {AND} {}?", p.low, p.high);
        let why = why_card(&kit, stage, &question, v(0.0, -2.2))?;
        play!(stage; fade_in(stage.arena_mut(), why, ORIGIN, 1.0)?.rt(0.4));
        why_pause(&kit, stage)?;

        // 08_2: fifty, the middle; three below it and three above
        let line = say(&kit, stage, "08_2")?;
        play!(stage; fade_out(stage.arena_mut(), why, ORIGIN, 1.0)?.rt(0.4));
        let mid_tick = number_line.tick(stage, mid, 0.5, ink);
        let mid_mark = eq(&kit, stage, &num(p.middle), SIDE_PT, under(mid))?;
        let mid_words = kit.text(
            stage,
            THE_MIDDLE,
            NOTE_PT,
            Palette::dim(),
            v(number_line.p(mid)[0], LINE_Y - 1.0),
        )?;
        stage.add_to_scene(mid_tick)?;
        play!(stage;
            grow_from_center(stage.arena_mut(), mid_tick, None)?.rt(0.5),
            fade_in(stage.arena_mut(), mid_mark, v(0.0, 0.15), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), mid_words, ORIGIN, 1.0)?.rt(0.5),
        );
        line.wait_sentence(stage, 1)?;
        let arc = line_style(ink, 3.0);
        let three_y = LINE_Y + BOW_LIFT + 0.3;
        let below_bow = number_line.bow(stage, low, mid, BOW_LIFT, arc);
        let below_three = mark(
            &kit,
            stage,
            &number_line,
            (low + mid) / 2.0,
            &num(p.distance),
            SIDE_PT,
            ink,
            BOW_LIFT + 0.3,
        )?;
        play!(stage;
            show_creation(below_bow).rt(0.6),
            fade_in(stage.arena_mut(), below_three, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "the other three above")?;
        let above_bow = number_line.bow(stage, mid, high, BOW_LIFT, arc);
        let above_three = mark(
            &kit,
            stage,
            &number_line,
            (mid + high) / 2.0,
            &num(p.distance),
            SIDE_PT,
            ink,
            BOW_LIFT + 0.3,
        )?;
        play!(stage;
            show_creation(above_bow).rt(0.6),
            fade_in(stage.arena_mut(), above_three, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_sentence(stage, 2)?;
        let distance_at = v(stage.get_right(above_three)[0] + 0.3, three_y);
        let distance_words = kit.text_left(stage, THE_DISTANCE, NOTE_PT, Palette::dim(), distance_at)?;
        play!(stage; fade_in(stage.arena_mut(), distance_words, v(-0.15, 0.0), 1.0)?.rt(0.5));

        // 08_3: the number line becomes the picture. The tick at fifty grows
        // into a square of fifty in outline; the rectangle, as tall as the
        // lower number and as wide as the higher, is laid over it as two
        // fills: the part inside the square and the strip that sticks out
        // past its right side. The bows wait to one side, then become the
        // strip's width and the gap along the top.
        let line = say(&kit, stage, "08_3")?;
        let board = fifty_board();
        let square = [0.0, 0.0, mid, mid];
        let rectangle = [0.0, 0.0, high, low];
        let outline = board.rect(stage, square, line_style(ink, 3.0));
        let fifty_top = board.label(&kit, stage, &num(p.middle), square, Side::Above, SIDE_PT, ink)?;
        let head_scale = ROW_PT / PROBLEM_PT;
        let head_at = v(HEAD_LEFT + stage.get_width(problem) * head_scale / 2.0, ROW_YS[0]);
        // the bows leave with the line: the brackets that carry their threes
        // come with the words that name them (R2, M4: waiting to one side,
        // they floated unattached for six seconds)
        let mut leaving = on_line.clone();
        leaving.extend(dots);
        leaving.extend([low_mark, high_mark, mid_mark, mid_words, distance_words]);
        leaving.extend([below_bow, below_three, above_bow, above_three]);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &leaving {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        anims.push(stage.prepare(replacement_transform(mid_tick, outline).rt(1.1))?);
        let a = fade_in(stage.arena_mut(), fifty_top, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(1.1))?);
        anims.push(stage.prepare(fly(problem, head_at, head_scale, 1.1)?)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, "lay the rectangle")?;
        // the strip is the moving strip from the start: standing at the
        // side now, lying in the gap after the move. Until the line names
        // it, it is one colour with the rest of the rectangle.
        let body = board.rect(stage, [0.0, 0.0, mid, low], flat(hundreds(), 1.0));
        let turn = Turn::between([low / 2.0, low + d / 2.0], [mid + d / 2.0, low / 2.0], low, d);
        let moving = Strip::build(stage, board, turn, strip(), 1.0)?;
        stage.set_color(moving.fill, hundreds(), None, true);
        stage.add_to_scene(moving.anchor)?;
        let tall = board.label(&kit, stage, &num(p.low), rectangle, Side::Left, SIDE_PT, ink)?;
        let wide = board.label(&kit, stage, &num(p.high), rectangle, Side::Below, SIDE_PT, ink)?;
        play!(stage;
            fade_in(stage.arena_mut(), body, v(0.0, -0.25), 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), moving.fill, v(0.0, -0.25), 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), tall, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), wide, ORIGIN, 1.0)?.rt(0.7),
        );
        stage.add_to_scene(moving.edge)?;
        stage.scene_mut().bring_to_front(&[outline])?;
        // it sticks out by three at the side: the bow above the middle
        // becomes the strip's width
        line.wait_sentence(stage, 1)?;
        let bottom = board.p(0.0, 0.0)[1] - 0.1;
        let strip_bow = bow_across(
            stage,
            v(board.p(high, 0.0)[0], bottom),
            v(board.p(mid, 0.0)[0], bottom),
            0.14,
            line_style(strip(), 3.0),
        );
        let strip_three_at = v(board.p(mid + d / 2.0, 0.0)[0], bottom - 0.42);
        let above_three = kit.text(stage, &num(p.distance), SIDE_PT, strip(), strip_three_at)?;
        play!(stage;
            show_creation(strip_bow).rt(0.9),
            fade_in(stage.arena_mut(), above_three, ORIGIN, 1.0)?.rt(0.9),
            repaint(moving.fill, strip(), 1.0, 0.9)?,
        );
        // and falls short by three at the top: the bow below it becomes
        // the gap's depth
        line.wait_for(stage, "falls short")?;
        let left = board.p(0.0, 0.0)[0] - 0.1;
        let gap = board.rect(stage, [0.0, low, mid, d], flat(gone(), GAP_OPACITY));
        let gap_bow = bow_across(
            stage,
            v(left, board.p(0.0, low)[1]),
            v(left, board.p(0.0, mid)[1]),
            0.14,
            line_style(gone(), 3.0),
        );
        let gap_three_at = v(left - 0.45, board.p(0.0, low + d / 2.0)[1]);
        let below_three = kit.text(stage, &num(p.distance), SIDE_PT, gone(), gap_three_at)?;
        play!(stage;
            show_creation(gap_bow).rt(0.9),
            fade_in(stage.arena_mut(), below_three, ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), gap, ORIGIN, 1.0)?.rt(0.9),
        );
        stage.scene_mut().bring_to_front(&[outline])?;
        line.wait_end(stage)?;
        fade_out_all(
            stage,
            &[tall, wide, fifty_top, strip_bow, gap_bow, above_three, below_three],
            0.4,
        )?;

        // 08_4: the one move, backwards: the strip lifts off the side,
        // turns and lies in the gap along the top, on the words that ask for
        // it (R2, M3: played in the silence before the line, it moved before
        // it was asked to); a hole three by three is left in the corner, and
        // a magnifier shows it
        let line = say(&kit, stage, "08_4")?;
        stage.scene_mut().bring_to_front(&[moving.fill, moving.edge])?;
        play!(stage; moving.turn_to(0.0)?);
        let seam = board.line(stage, [0.0, low], [turn.len, low], cut_style());
        stage.add_to_scene(seam)?;
        stage.scene_mut().bring_to_front(&[outline])?;
        line.wait_sentence(stage, 2)?;
        let hole = [low, low, d, d];
        let hole_edge = board.rect(stage, hole, line_style(gone(), 4.0));
        play!(stage; show_creation(hole_edge).rt(0.5));
        let window = [low - 2.0, low - 2.0, d + 4.0, d + 4.0];
        let lens = Lens::build(stage, board, window, LENS_AT, LENS_SIZE / (window[2] * board.unit));
        let big = lens.board;
        let mut enlarged = Vec::new();
        for (r, color, opacity) in [
            ([0.0, 0.0, mid, low], hundreds(), 1.0),
            ([0.0, low, turn.len, d], strip(), 1.0),
            (hole, gone(), GAP_OPACITY),
        ] {
            if let Some(part) = lens.clip(r) {
                enlarged.push(big.rect(stage, part, flat(color, opacity)));
            }
        }
        let [wx, wy, _, _] = window;
        enlarged.push(big.line(stage, [wx, low], [turn.len, low], cut_style()));
        enlarged.push(big.line(stage, [wx, mid], [mid, mid], line_style(ink, 3.0)));
        enlarged.push(big.line(stage, [mid, wy], [mid, mid], line_style(ink, 3.0)));
        enlarged.push(big.rect(stage, hole, line_style(gone(), 4.0)));
        let sides = [
            big.label(&kit, stage, &num(p.distance), hole, Side::Right, NOTE_PT, gone())?,
            big.label(&kit, stage, &num(p.distance), hole, Side::Above, NOTE_PT, gone())?,
        ];
        let [where_from, ray_a, ray_b, back, frame] = lens.mobs();
        let mut ins = vec![where_from, ray_a, ray_b, back];
        ins.extend(&enlarged);
        ins.push(frame);
        ins.extend(sides);
        fade_in_all(stage, &ins, 0.6)?;
        let mut magnifier = lens.mobs().to_vec();
        magnifier.extend(&enlarged);
        magnifier.extend(sides);

        // 08_5: fifty squared, minus three squared; twenty-five hundred,
        // minus nine; the hole stays red on the square. The column takes
        // the hole's colour for what it counts.
        let line = say(&kit, stage, "08_5")?;
        let fifty_left = board.label(&kit, stage, &num(p.middle), square, Side::Left, SIDE_PT, ink)?;
        let squares_src = format!("= {}^2 - {}^2", p.middle, p.distance);
        let squares = row(&kit, stage, &squares_src, ROW_PT, ROWS_LEFT, ROW_YS[1])?;
        // the distance squared is the last glyphs: its digits and the 2
        let n = glyphs(stage, squares).len();
        tint(stage, squares, n - num(p.distance).len() - 1, n, gone())?;
        play!(stage;
            fade_in(stage.arena_mut(), fifty_left, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), squares, v(0.0, -0.12), 1.0)?.rt(0.6),
        );
        line.wait_sentence(stage, 1)?;
        let values_src = format!("= {} - {}", p.middle_squared, p.distance_squared);
        let values = row(&kit, stage, &values_src, ROW_PT, ROWS_LEFT, ROW_YS[2])?;
        let n = glyphs(stage, values).len();
        tint(stage, values, n - num(p.distance_squared).len(), n, gone())?;
        let identity =
            format!(r"(a + b)(a - b) = a^2 - b^2 \qquad a = \text{{{THE_MIDDLE}}},\ b = \text{{{THE_DISTANCE}}}");
        let lower = lower_third_tex(&kit, stage, SQUARES, &identity)?;
        play!(stage;
            fade_in(stage.arena_mut(), values, v(0.0, -0.12), 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), lower, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_sentence(stage, 2)?;
        let answer = row(
            &kit,
            stage,
            &format!("= {}", p.product),
            ANSWER_PT,
            ROWS_LEFT,
            ROW_YS[3],
        )?;
        play!(stage; fade_in(stage.arena_mut(), answer, v(0.0, -0.12), 1.0)?.rt(0.6));

        // 08_6: your turn: nineteen times twenty-one, on a card at the side
        // that leaves room for its working
        let line = say(&kit, stage, "08_6")?;
        let mut leaving = vec![problem, squares, values, answer, lower, fifty_left];
        leaving.extend([outline, body, gap, seam, hole_edge]);
        leaving.extend(moving.mobs());
        leaving.extend(&magnifier);
        let look = QuizLook::at(CARD_AT, CARD_SCALE);
        let card = QuizCard::build(&kit, stage, "08_6", look)?;
        card.arrive(stage, &line, &leaving)?;
        card.count(&kit, stage)?;

        // 08_7: the middle is twenty and the distance one, on a number line
        // of its own; the move replayed small on a square of twenty; four
        // hundred minus one, and the 399 already on the card
        let quiz = quiz_at("08_6")?;
        let Ask::Times(qa, qb) = quiz.ask else {
            return Err(fail(format!("quiz 08_6 asks {:?}, not a product", quiz.ask)));
        };
        let q = pair_of((qa, qb))?;
        if quiz.worked().map_err(fail)?.answer != num(q.product) {
            return Err(fail(format!("quiz 08_6 lands on something other than {}", q.product)));
        }
        let (q_low, q_high, q_mid, q_d) = (q.low as f64, q.high as f64, q.middle as f64, q.distance as f64);
        let line = say(&kit, stage, "08_7")?;
        let small_line = NumberLine1 {
            x0: SMALL_LINE.0,
            x1: SMALL_LINE.1,
            y: SMALL_LINE.2,
            lo: q_low - q_d,
            hi: q_high + q_d,
        };
        let small_under = |x: f64| v(small_line.p(x)[0], SMALL_LINE.2 - 0.42);
        let mut small_marks = vec![small_line.axis(stage)];
        small_marks.extend(unit_ticks(stage, &small_line));
        small_marks.push(small_line.dot(stage, q_low, 0.08, ink));
        small_marks.push(small_line.dot(stage, q_high, 0.08, ink));
        small_marks.push(eq(&kit, stage, &num(q.low), SIDE_PT, small_under(q_low))?);
        small_marks.push(eq(&kit, stage, &num(q.high), SIDE_PT, small_under(q_high))?);
        fade_in_all(stage, &small_marks, 0.5)?;
        line.wait_for(stage, "twenty")?;
        let small_mid = small_line.tick(stage, q_mid, 0.45, ink);
        let small_mid_mark = eq(&kit, stage, &num(q.middle), SIDE_PT, small_under(q_mid))?;
        stage.add_to_scene(small_mid)?;
        play!(stage;
            grow_from_center(stage.arena_mut(), small_mid, None)?.rt(0.4),
            fade_in(stage.arena_mut(), small_mid_mark, v(0.0, 0.12), 1.0)?.rt(0.4),
        );
        small_marks.extend([small_mid, small_mid_mark]);
        line.wait_for(stage, "the distance is one")?;
        let small_lift = 0.45;
        let mut bows = Vec::new();
        for (a, b) in [(q_low, q_mid), (q_mid, q_high)] {
            bows.push(small_line.bow(stage, a, b, small_lift, line_style(ink, 2.5)));
            let at = v(small_line.p((a + b) / 2.0)[0], SMALL_LINE.2 + small_lift + 0.28);
            bows.push(eq(&kit, stage, &num(q.distance), SIDE_PT, at)?);
        }
        fade_in_all(stage, &bows, 0.5)?;
        small_marks.extend(bows);
        // the move in small: the square of the middle in outline, the
        // rectangle over it, and its strip lying down in the gap
        let small = Board::new(SMALL_AT, SMALL_UNIT);
        let small_square = [0.0, 0.0, q_mid, q_mid];
        let small_outline = small.rect(stage, small_square, line_style(ink, 2.5));
        let small_body = small.rect(stage, [0.0, 0.0, q_mid, q_low], flat(hundreds(), 1.0));
        let small_turn = Turn::between(
            [q_low / 2.0, q_low + q_d / 2.0],
            [q_mid + q_d / 2.0, q_low / 2.0],
            q_low,
            q_d,
        );
        let small_strip = Strip::build(stage, small, small_turn, strip(), 1.0)?;
        let small_gap = small.rect(stage, [0.0, q_low, q_mid, q_d], flat(gone(), GAP_OPACITY));
        let small_side = small.label(&kit, stage, &num(q.middle), small_square, Side::Left, NOTE_PT, ink)?;
        stage.add_to_scene(small_strip.anchor)?;
        fade_in_all(
            stage,
            &[small_body, small_strip.fill, small_gap, small_outline, small_side],
            0.5,
        )?;
        stage.add_to_scene(small_strip.edge)?;
        line.wait_sentence(stage, 1)?;
        let work_src = format!(
            "{}^2 - {}^2 = {} - {}",
            q.middle, q.distance, q.middle_squared, q.distance_squared
        );
        let work = eq(&kit, stage, &work_src, WORK_PT, v(CARD_AT[0], WORK_YS[0]))?;
        fit_width(stage, work, 5.6);
        // the distance squared, and what it is worth, in the hole's colour
        let (m_len, d_len) = (num(q.middle).len(), num(q.distance).len());
        let n = glyphs(stage, work).len();
        tint(stage, work, m_len + 2, m_len + 2 + d_len + 1, gone())?;
        tint(stage, work, n - num(q.distance_squared).len(), n, gone())?;
        stage
            .scene_mut()
            .bring_to_front(&[small_strip.fill, small_strip.edge])?;
        play!(stage;
            small_strip.turn_to(0.0)?,
            fade_in(stage.arena_mut(), work, v(0.0, -0.12), 1.0)?.rt(0.8),
        );
        let small_seam = small.line(stage, [0.0, q_low], [small_turn.len, q_low], cut_style());
        let small_hole = small.rect(stage, [q_low, q_low, q_d, q_d], line_style(gone(), 3.0));
        stage.add_to_scene(small_seam)?;
        stage.scene_mut().bring_to_front(&[small_outline])?;
        play!(stage; show_creation(small_hole).rt(0.4));
        line.wait_sentence(stage, 2)?;
        let result = eq(
            &kit,
            stage,
            &format!("= {}", q.product),
            WORK_PT,
            v(CARD_AT[0], WORK_YS[1]),
        )?;
        play!(stage; fade_in(stage.arena_mut(), result, v(0.0, -0.12), 1.0)?.rt(0.5));
        // its number goes up into the answer already on the card
        let digits = glyphs(stage, result);
        let from = middle_of(stage, &digits[1..])?;
        let copy = eq(&kit, stage, &num(q.product), WORK_PT, from)?;
        stage.add_to_scene(copy)?;
        play!(stage; fly(copy, card.answer_at(), look.pt() / WORK_PT, 0.8)?);
        stage.remove_from_scene(copy);

        // 08_8: what the trick needs. The quiz stays while the line says it
        // needs a middle and a distance both easy to square; then twelve
        // times eighty-eight, an easy middle and a distance too big to
        // square; forty-seven times fifty-six has no whole middle at all,
        // so back to parts
        let line = say(&kit, stage, "08_8")?;
        line.wait_for(stage, "both easy to square")?;
        let mut leaving = card.mobs();
        leaving.extend(small_marks);
        leaving.extend([small_outline, small_body, small_gap, small_side, small_seam, small_hole]);
        leaving.extend(small_strip.mobs());
        leaving.extend([work, result]);
        let far = pair_of(shown::FAR_PAIR)?;
        let aside_src = format!(
            r"{} \times {} = {}^2 - {}^2:\ {TRUE_NO_EASIER}",
            far.low, far.high, far.middle, far.distance
        );
        let aside = grey_line(&kit, stage, &aside_src, v(0.0, ASIDE_Y))?;
        fit_width(stage, aside, 12.0);
        swap(stage, &leaving, &[aside], 0.6)?;
        line.wait_sentence(stage, 1)?;
        let (nx, ny) = shown::NO_PAIR;
        if pair(nx, ny).is_some() {
            return Err(fail(format!("{nx} and {ny} have a whole middle")));
        }
        let (lo, hi) = (nx.min(ny) as f64, nx.max(ny) as f64);
        let no_line = NumberLine1 {
            x0: LINE_X.0,
            x1: LINE_X.1,
            y: LINE_Y,
            lo: lo - 3.0,
            hi: hi + 3.0,
        };
        let no_problem = eq(&kit, stage, &format!(r"{nx} \times {ny}"), PROBLEM_PT, PROBLEM_AT)?;
        let no_under = |x: f64| v(no_line.p(x)[0], LINE_Y - 0.42);
        let mut no_marks = vec![no_problem, no_line.axis(stage)];
        no_marks.extend(unit_ticks(stage, &no_line));
        no_marks.push(no_line.dot(stage, lo, 0.09, ink));
        no_marks.push(no_line.dot(stage, hi, 0.09, ink));
        no_marks.push(eq(&kit, stage, &num(nx.min(ny)), SIDE_PT, no_under(lo))?);
        no_marks.push(eq(&kit, stage, &num(nx.max(ny)), SIDE_PT, no_under(hi))?);
        fade_in_all(stage, &no_marks, 0.6)?;
        line.wait_for(stage, "middle at")?;
        // its middle, in halves: an odd sum, so a half
        let sum = nx + ny;
        let half_at = sum as f64 / 2.0;
        let half_tick = no_line.tick(stage, half_at, 0.5, gone());
        let half_src = format!(r"{}\tfrac{{1}}{{2}}", sum / 2);
        let half_mark = mark(&kit, stage, &no_line, half_at, &half_src, SIDE_PT, gone(), 0.62)?;
        stage.add_to_scene(half_tick)?;
        play!(stage;
            grow_from_center(stage.arena_mut(), half_tick, None)?.rt(0.5),
            fade_in(stage.arena_mut(), half_mark, v(0.0, -0.12), 1.0)?.rt(0.5),
        );
        line.wait_sentence(stage, 2)?;
        let note = precisely(&kit, stage, WHOLE_MIDDLE)?;
        play!(stage; fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.6));
        kit.music_breath(stage, super::draw::END_HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
