//! Chapter 02, ends in five: the trick on thirty-five (02_1 to 02_3), the
//! question it leaves (02_4), and the picture that answers it: a square of
//! thirty-five, a strip five deep sliced off its top (02_5) and stood
//! against its side (02_6, the one move); the square has become a rectangle
//! thirty tall and forty wide, with a small square sticking up (02_7); the
//! rectangle is twelve squares of a hundred (02_8) and the small square is
//! the twenty-five (02_9).

use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::prelude::*;

use super::draw::{
    Board, COLUMN_ROOM, COLUMN_X, FIGURE_AT, MoveFigure, NOTE_PT, SIDE_PT, Side, corner, eq, fade_in_all,
    fade_in_lagged, fade_out_all, fail, fit_width, fly, glyphs, grey_line, hundreds, middle_of, num, pause, say, slate,
    strip, swap, why_card, why_pause,
};
use crate::facts::shown;
use crate::model::{OneMove, ends_in_five, one_move};

pub struct EndsInFive {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const IN_FRONT: &str = "the number in front";
const WHY: &str = "why";
const AND_NOT: &str = "and not";
const TRUE_NO_EASIER: &str = r"\text{true, and no easier}";

/// Scene units per one: the square of thirty-five is 4.4 tall, and the
/// strip's turn stays inside the frame.
const UNIT: f64 = 0.125;
/// The trick's own numbers, and the line that joins them.
const BIG_PT: f64 = 130.0;
const MID_PT: f64 = 90.0;
/// The sums in the column beside the figure.
const SUM_PT: f64 = 64.0;
/// The faint grid of tens behind the figure, in units.
const GRID: (f64, f64) = (50.0, 40.0);

/// The move on `side` with a strip `deep`, or a scene error.
fn the_move_of(side: u64, deep: u64) -> fmn::Result<OneMove> {
    one_move(side, deep).ok_or_else(|| fail(format!("no strip {deep} deep comes off a square of {side}")))
}

impl SceneConstruct for EndsInFive {
    fn name(&self) -> &str {
        "02_ends_in_five"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let five = ends_in_five(shown::FIVE).ok_or_else(|| fail(format!("{} does not end in five", shown::FIVE)))?;
        let m = the_move_of(five.number, five.number % 10)?;
        let (blue, orange) = (hundreds(), corner());

        // 02_1: thirty-five squared; the number in front of the five is three.
        // The number in front is typeset on its own, where it sits in the
        // whole, so it can take its colour without the rest.
        let line = say(&kit, stage, "02_1")?;
        line.wait_sentence(stage, 1)?;
        let whole_square = eq(&kit, stage, &format!("{}^2", five.number), BIG_PT, v(0.0, 2.3))?;
        let all = glyphs(stage, whole_square);
        let split = num(five.n).len();
        if all.len() <= split {
            return Err(fail(format!("{}^2 has nothing after its number in front", five.number)));
        }
        let (front_at, rest_at) = (middle_of(stage, &all[..split])?, middle_of(stage, &all[split..])?);
        let front = eq(&kit, stage, &num(five.n), BIG_PT, front_at)?;
        let rest = eq(&kit, stage, &format!("{}^2", five.number % 10), BIG_PT, rest_at)?;
        fade_in_all(stage, &[front, rest], 0.6)?;
        line.wait_sentence(stage, 2)?;
        let label = kit.text(stage, IN_FRONT, 34.0, blue, v(front_at[0] - 0.55, front_at[1]))?;
        stage.move_to(label, v(front_at[0] - 0.55, front_at[1]), RIGHT);
        play!(stage;
            front.animate().set_anim_args(AnimateArgs { run_time: Some(0.5), ..AnimateArgs::default() })?.set_color(blue)?,
            fade_in(stage.arena_mut(), label, v(0.2, 0.0), 1.0)?.rt(0.5),
        );

        // 02_2: three times four is twelve
        let line = say(&kit, stage, "02_2")?;
        line.wait_sentence(stage, 1)?;
        let product_src = format!(r"{} \times {} = {}", five.n, five.n + 1, five.hundreds);
        let product = kit.tex(
            stage,
            &product_src,
            MID_PT,
            Palette::ink(),
            &[(&num(five.hundreds), blue)],
            v(0.0, 0.5),
        )?;
        play!(stage; fade_in(stage.arena_mut(), product, v(0.0, -0.15), 1.0)?.rt(0.6));

        // 02_3: then write twenty-five after it: 1225
        let line = say(&kit, stage, "02_3")?;
        let whole = eq(&kit, stage, &num(five.square), BIG_PT, v(0.0, -1.3))?;
        let digits = glyphs(stage, whole);
        let split = num(five.hundreds).len();
        if digits.len() <= split {
            return Err(fail(format!("{} has no digits after its hundreds", five.square)));
        }
        let (lead_at, tail_at) = (middle_of(stage, &digits[..split])?, middle_of(stage, &digits[split..])?);
        let product_glyphs = glyphs(stage, product);
        let from = middle_of(stage, &product_glyphs[product_glyphs.len() - split..])?;
        let lead = kit.tex(stage, &num(five.hundreds), MID_PT, blue, &[], from)?;
        let tail = kit.tex(stage, &num(m.corner()), BIG_PT, orange, &[], tail_at)?;
        stage.add_to_scene(lead)?;
        play!(stage;
            fly(lead, lead_at, BIG_PT / MID_PT, 0.8)?,
            fade_in(stage.arena_mut(), tail, v(0.6, 0.0), 1.0)?.rt(0.8),
        );
        line.wait_sentence(stage, 1)?;
        let plain = kit.tex(
            stage,
            &format!(r"{0} \times {0} = {1}", five.number, five.square),
            44.0,
            Palette::dim(),
            &[],
            v(0.0, -2.75),
        )?;
        play!(stage; fade_in(stage.arena_mut(), plain, ORIGIN, 1.0)?.rt(0.5));

        // 02_4: but why the next number up? The square on its grid of tens
        let line = say(&kit, stage, "02_4")?;
        let claim_src = format!("{}^2 = {}", five.number, five.square);
        let claim = eq(&kit, stage, &claim_src, 68.0, v(COLUMN_X, 2.9))?;
        let claim_glyphs = glyphs(stage, claim);
        let n = claim_glyphs.len();
        let digits_of = |x: u64| num(x).len();
        let (tail_len, lead_len) = (digits_of(m.corner()), digits_of(five.hundreds));
        for &g in &claim_glyphs[n - tail_len..] {
            stage.set_color(g, orange, None, true);
        }
        for &g in &claim_glyphs[n - tail_len - lead_len..n - tail_len] {
            stage.set_color(g, blue, None, true);
        }
        swap(stage, &[front, rest, label, product, lead, tail, plain], &[claim], 0.6)?;
        let board = Board::new(FIGURE_AT, UNIT);
        let grid = board.tens(stage, GRID.0, GRID.1)?;
        let figure = MoveFigure::build(stage, board, m, slate(), slate(), 0.0)?;
        let side_words = num(m.side);
        let ink = Palette::ink();
        let left = board.label(&kit, stage, &side_words, figure.square_rect(), Side::Left, SIDE_PT, ink)?;
        let below = board.label(
            &kit,
            stage,
            &side_words,
            figure.square_rect(),
            Side::Below,
            SIDE_PT,
            ink,
        )?;
        let mut ins = vec![grid];
        ins.extend(figure.square());
        ins.extend([left, below]);
        fade_in_all(stage, &ins, 0.8)?;
        line.wait_end(stage)?;
        let question = format!("{WHY} {0} × {1}, {AND_NOT} {0} × {0}?", five.n, five.n + 1);
        let why = why_card(&kit, stage, &question, v(COLUMN_X, 0.9))?;
        play!(stage; fade_in(stage.arena_mut(), why, ORIGIN, 1.0)?.rt(0.4));
        why_pause(&kit, stage)?;

        // 02_5: slice a strip off the top, five deep
        let line = say(&kit, stage, "02_5")?;
        play!(stage; fade_out(stage.arena_mut(), why, ORIGIN, 1.0)?.rt(0.4));
        line.wait_sentence(stage, 1)?;
        figure.slice(stage, 0.8)?;
        let deep = board.label(
            &kit,
            stage,
            &num(m.strip),
            figure.top_rect(),
            Side::Right,
            SIDE_PT,
            strip(),
        )?;
        play!(stage; fade_in(stage.arena_mut(), deep, ORIGIN, 1.0)?.rt(0.4));

        // 02_6: the one move
        say(&kit, stage, "02_6")?;
        fade_out_all(stage, &[deep, left, below], 0.4)?;
        figure.play(&kit, stage, "02_6")?;

        // 02_7: a rectangle, thirty tall and forty wide, and a small square
        let line = say(&kit, stage, "02_7")?;
        let rectangle = figure.rectangle_rect();
        line.wait_sentence(stage, 1)?;
        let tall = board.label(&kit, stage, &num(m.tall()), rectangle, Side::Left, SIDE_PT, ink)?;
        play!(stage; fade_in(stage.arena_mut(), tall, ORIGIN, 1.0)?.rt(0.5));
        line.wait_sentence(stage, 2)?;
        let wide = board.label(&kit, stage, &num(m.wide()), rectangle, Side::Below, SIDE_PT, ink)?;
        play!(stage; fade_in(stage.arena_mut(), wide, ORIGIN, 1.0)?.rt(0.5));
        line.wait_sentence(stage, 3)?;
        figure.show_corner(stage, 0.6)?;
        line.wait_sentence(stage, 4)?;
        let small = format!("{0} × {0}", m.strip);
        let small = board.label(&kit, stage, &small, figure.corner_rect(), Side::Right, NOTE_PT, orange)?;
        play!(stage; fade_in(stage.arena_mut(), small, ORIGIN, 1.0)?.rt(0.4));

        // 02_8: thirty by forty is three times four, in hundreds
        let line = say(&kit, stage, "02_8")?;
        figure.merge(stage, 0.6)?;
        let tiles = board.hundreds(&kit, stage, rectangle, true)?;
        play!(stage; fade_in(stage.arena_mut(), tiles.lines, ORIGIN, 1.0)?.rt(0.6));
        fade_in_lagged(stage, &tiles.numerals, 2.2, 0.5)?;
        if tiles.numerals.len() as u64 != five.hundreds {
            return Err(fail(format!(
                "{} tiles of a hundred, but the trick says {}",
                tiles.numerals.len(),
                five.hundreds
            )));
        }
        line.wait_sentence(stage, 1)?;
        let area_src = format!(r"{} \times {} = {}", m.tall(), m.wide(), m.rectangle());
        let area = kit.tex(
            stage,
            &area_src,
            SUM_PT,
            Palette::ink(),
            &[(&num(m.rectangle()), blue)],
            v(COLUMN_X, 1.4),
        )?;
        play!(stage; fade_in(stage.arena_mut(), area, ORIGIN, 1.0)?.rt(0.5));

        // 02_9: and the small square is the twenty-five
        let line = say(&kit, stage, "02_9")?;
        let small_src = format!(r"{0} \times {0} = {1}", m.strip, m.corner());
        let small_area = kit.tex(
            stage,
            &small_src,
            SUM_PT,
            Palette::ink(),
            &[(&num(m.corner()), orange)],
            v(COLUMN_X, 0.4),
        )?;
        play!(stage; fade_in(stage.arena_mut(), small_area, ORIGIN, 1.0)?.rt(0.5));
        line.wait_sentence(stage, 1)?;
        let sum_src = format!(
            r"{}^2 = {} \times {} + {} = {}",
            m.side,
            m.tall(),
            m.wide(),
            m.corner(),
            m.square()
        );
        let sum = eq(&kit, stage, &sum_src, 56.0, v(COLUMN_X, -1.0))?;
        fit_width(stage, sum, COLUMN_ROOM);
        play!(stage; fade_in(stage.arena_mut(), sum, v(0.0, -0.12), 1.0)?.rt(0.6));
        line.wait_sentence(stage, 2)?;
        let (other, other_deep) = shown::NOT_FIVE;
        let o = the_move_of(other, other_deep)?;
        let aside_src = format!(
            r"{}^2 = {} \times {} + {}:\ {TRUE_NO_EASIER}",
            o.side,
            o.tall(),
            o.wide(),
            o.corner()
        );
        let aside = grey_line(&kit, stage, &aside_src, v(COLUMN_X, -2.1))?;
        fit_width(stage, aside, COLUMN_ROOM);
        pause(stage, 1.2)?;
        play!(stage; fade_in(stage.arena_mut(), aside, ORIGIN, 1.0)?.rt(0.5));
        kit.music_breath(stage, super::draw::END_HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
