//! Chapter 11, your turn near fifty: the moved-strip picture of chapter 10
//! stays on the left, drawn from one tracker that holds the square's side,
//! so it redraws itself honestly as the side slides. A quiz, forty-six
//! squared (11_1); the side slides to forty-six and the answer is read off,
//! twenty-one then sixteen (11_2); then to sixty-two, where the small square
//! is twelve by twelve and overflows a hundred, which peels off and joins
//! the hundreds: thirty-seven becomes thirty-eight and forty-four is left
//! (11_3). Feynman's sentence about Bethe closes the story (11_4).

use explainer_kit::{Kit, Palette, Timed, fade_all, play, props, tween, v};
use fmn::prelude::*;

use super::draw::{
    Board, QuizLook, QuoteCard, corner, cut_style, eq, fade_from_black, fade_in_all, fade_in_lagged, fade_out_all,
    fail, fifty_board, flat, fly, glyphs, gone, group, hundreds, line_style, middle_of, num, quiz_at, say, strip,
    wait_until, your_turn,
};
use crate::facts::{cited, digit_pairs_range, shown};
use crate::model::{Ask, FIFTY, near_fifty};

pub struct YourTurnFifty {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const READS_OFF: &str = "reads straight off from";
const TO: &str = "to";
/// One of the video's two verbatim quotations, set exactly, and its source.
const QUOTE: &str = "“It was easy for him—every number was near something he knew.”";
const SOURCE_WHO: &str = "Richard P. Feynman, on Hans Bethe";
const SOURCE_BOOK: &str = "“Surely You’re Joking, Mr. Feynman!”";
const SOURCE_CHAPTER: &str = "“Lucky Numbers”";

/// The quiz card at the right, in small beside the picture.
const CARD_AT: Vec3 = [3.75, 1.0, 0.0];
const CARD_SCALE: f64 = 0.7;
/// The working under the card.
const WORK_Y: [f64; 2] = [-1.5, -2.45];
const WORK_PT: f64 = 60.0;
/// The square being worked, named above the outline of fifty.
const NAME_AT: Vec3 = [-4.1, 1.95, 0.0];
const NAME_PT: f64 = 52.0;
/// Sixty-two's working, left-aligned in the right half once the card has
/// gone, and the line where the hundred spills.
const COL_LEFT: f64 = 1.0;
const ROW_Y: [f64; 3] = [2.5, 1.4, -0.15];
const SUM_PT: f64 = 64.0;
const SPILL_PT: f64 = 72.0;
/// As chapter 10: the band short of fifty squared, the columns past it.
const BAND_OPACITY: f64 = 0.45;
const LIT_OPACITY: f64 = 0.28;
/// A square of a hundred is ten on a side.
const HUNDRED_SIDE: f64 = 10.0;
/// How small the hundred gets as it joins the hundreds, how far above them
/// it and the 1 of 144 meet before they drop in, and how far round the 1
/// swings on its way (radians: positive bends over a leftward path).
const PEEL_SCALE: f64 = 0.45;
const DROP: f64 = 0.75;
const SPILL_ARC: f64 = 1.2;
/// The spill starts this long before "Thirty-eight forty-four." begins (the
/// 0.3 s that ends "plus one hundred forty-four", and 0.4 s of its last
/// word) and the hundred and the 1 meet in `SPILL_FLY_S`, so that "3800 + 44
/// = 3844" is whole as the sentence ends: it came 0.9 s after it in the
/// scored render of 14 Oct.
const SPILL_LEAD: f64 = 0.7;
const SPILL_FLY_S: f64 = 0.6;
const QUOTE_AT: Vec3 = [0.0, 0.2, 0.0];
const QUOTE_CHARS: usize = 34;

/// The picture after the move for a square of side `n` (any side, so that
/// a tracker can slide it): the rectangle, the strip standing at its side,
/// the small square on top of the strip, the band the rectangle falls short
/// of fifty squared by (below fifty) and the columns it runs past it by
/// (above), and the two cut lines; each `[x, y, wide, tall]` in units, and a
/// piece that is not there has no size. Below fifty the rectangle is 2n − 50
/// tall and 50 wide; above, 50 tall and 2n − 50 wide; the small square is
/// |n − 50| on a side.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Moved {
    rectangle: [f64; 4],
    strip: [f64; 4],
    corner: [f64; 4],
    band: [f64; 4],
    past: [f64; 4],
    seam: [f64; 4],
    corner_cut: [f64; 4],
}

fn moved(n: f64) -> Moved {
    let fifty = FIFTY as f64;
    let d = (n - fifty).abs();
    let (tall, wide) = (n - d, n + d);
    Moved {
        rectangle: [0.0, 0.0, wide, tall],
        strip: [n, 0.0, d, tall],
        corner: [n, tall, d, d],
        band: [0.0, tall, fifty, (fifty - tall).max(0.0)],
        past: [fifty, 0.0, (wide - fifty).max(0.0), fifty],
        seam: [n, 0.0, 0.0, tall],
        corner_cut: [n, tall, d, 0.0],
    }
}

/// One piece of the moved picture, redrawn every frame from the tracker
/// `side`. In the scene at once.
fn live_piece(
    stage: &mut Stage<'_>,
    board: Board,
    side: Mob,
    start: f64,
    pick: fn(&Moved) -> [f64; 4],
    style: Style,
) -> fmn::Result<Mob> {
    board.live_rect(
        stage,
        move |s| pick(&moved(s.tracker_value(side).unwrap_or(start))),
        style,
    )
}

/// Move a mobject so its centre is at `to` along an arc of `arc` radians,
/// scaling it by `scale` on the way, `rt` seconds.
fn arc_fly(mob: Mob, to: Vec3, scale: f64, rt: f64, arc: f64) -> fmn::Result<AnimBuilder> {
    let a = mob.animate().set_anim_args(AnimateArgs {
        run_time: Some(rt),
        path_arc: Some(arc),
        ..AnimateArgs::default()
    })?;
    Ok(a.scale(scale)?.move_to(to, ORIGIN)?)
}

/// A part of a line of mathematics and its colour.
fn part(src: impl Into<String>, color: Srgb) -> (String, Srgb) {
    (src.into(), color)
}

/// A line of mathematics set whole, then cut into its parts: one TeX
/// mobject per part, in its own colour and exactly where it sits in the
/// whole line, so that a part can come in, fly or be replaced on its own.
/// The line stands with its `edge` (LEFT, or ORIGIN for its middle) at
/// `at`. Detached.
fn line_parts(
    kit: &Kit,
    stage: &mut Stage<'_>,
    parts: &[(String, Srgb)],
    size: f64,
    at: Vec3,
    edge: Vec3,
) -> fmn::Result<Vec<Mob>> {
    let src = parts.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>().join(" ");
    let whole = eq(kit, stage, &src, size, at)?;
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

impl SceneConstruct for YourTurnFifty {
    fn name(&self) -> &str {
        "11_your_turn_fifty"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (blue, green, orange, red, ink) = (hundreds(), strip(), corner(), gone(), Palette::ink());
        let fifty = FIFTY as f64;
        let board = fifty_board();
        let quiz = quiz_at("11_1")?;
        let Ask::Square(asked) = quiz.ask else {
            return Err(fail(format!("11_1 asks {:?}, not a square", quiz.ask)));
        };
        // fifty squared is twenty-five hundred: the hundreds start there
        let base = near_fifty(FIFTY);
        let first = near_fifty(shown::BELOW);
        let quizzed = near_fifty(asked);
        let far = near_fifty(shown::CARRIES);
        let (front, back) = quizzed
            .digit_pairs()
            .ok_or_else(|| fail(format!("{asked}'s small square runs past two digits")))?;
        if format!("{front}{back}") != num(quizzed.square) {
            return Err(fail(format!("{asked}^2 does not read as {front} then {back}")));
        }
        // sixty-two's small square holds one hundred and some over
        let (spilled, kept) = (far.corner / 100, far.corner % 100);
        if spilled != 1 || far.square % 100 != kept || far.rough + far.corner != far.square {
            return Err(fail(format!("the picture spills one hundred, but {far:?}")));
        }
        let kept = format!("{kept:02}");

        // the picture after the move on a tracker holding the side, in the
        // scene from the first frame at forty-eight (where chapter 10 began)
        let start = first.number as f64;
        let side = stage.add_value_tracker(start);
        // back to front; only the small square and its cut are needed again
        // (fade_all clears the others' updaters as it fades them)
        live_piece(stage, board, side, start, |m| m.band, flat(red, BAND_OPACITY))?;
        live_piece(stage, board, side, start, |m| m.rectangle, flat(blue, 1.0))?;
        live_piece(stage, board, side, start, |m| m.strip, flat(green, 1.0))?;
        live_piece(stage, board, side, start, |m| m.past, flat(ink, LIT_OPACITY))?;
        live_piece(stage, board, side, start, |m| m.seam, cut_style())?;
        let small = live_piece(stage, board, side, start, |m| m.corner, flat(orange, 1.0))?;
        let small_cut = live_piece(stage, board, side, start, |m| m.corner_cut, cut_style())?;
        let outline = board.rect(stage, [0.0, 0.0, fifty, fifty], line_style(ink, 2.5));
        stage.add_to_scene(outline)?;
        let name = eq(&kit, stage, &format!("{}^2", first.number), NAME_PT, NAME_AT)?;
        stage.add_to_scene(name)?;
        // chapter 10 ends on its rows, faded out: this picture comes up from
        // the dark rather than appearing whole on the first frame (R2, M12)
        fade_from_black(stage, 0.5)?;

        // 11_1: your turn: forty-six squared, the card at the right
        let card = your_turn(&kit, stage, "11_1", QuizLook::at(CARD_AT, CARD_SCALE))?;

        // 11_2: the side slides to forty-six, the band and the small square
        // following; twenty-one, then sixteen: the 2116 on the card
        let line = say(&kit, stage, "11_2")?;
        let name46 = eq(&kit, stage, &format!("{asked}^2"), NAME_PT, NAME_AT)?;
        play!(stage;
            tween(side, asked as f64, 1.2)?,
            fade_out(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), name46, ORIGIN, 1.0)?.rt(0.6),
        );
        let d = quizzed.distance.unsigned_abs();
        let sign = if quizzed.distance >= 0 { "+" } else { "-" };
        line.wait_sentence(stage, 1)?;
        let hundreds_row = line_parts(
            &kit,
            stage,
            &[
                part(base.hundreds.to_string(), ink),
                part(sign, ink),
                part(num(d), ink),
                part("=", ink),
                part(&front, blue),
            ],
            WORK_PT,
            v(CARD_AT[0], WORK_Y[0]),
            ORIGIN,
        )?;
        fade_in_all(stage, &hundreds_row, 0.5)?;
        line.wait_sentence(stage, 2)?;
        let corner_row = line_parts(
            &kit,
            stage,
            &[part(format!("{d}^2"), ink), part("=", ink), part(&back, orange)],
            WORK_PT,
            v(CARD_AT[0], WORK_Y[1]),
            ORIGIN,
        )?;
        fade_in_all(stage, &corner_row, 0.5)?;
        line.wait_sentence(stage, 3)?;
        // the 21 and the 16 slide up into the 2116 already on the card,
        // which takes their colours as they arrive
        let digits = glyphs(stage, card.answer);
        let split = front.chars().count();
        if digits.len() != split + back.chars().count() {
            return Err(fail(format!("the card's answer is not {front} then {back}")));
        }
        let (lead_at, tail_at) = (middle_of(stage, &digits[..split])?, middle_of(stage, &digits[split..])?);
        let (lead_from, tail_from) = (stage.get_center(hundreds_row[4]), stage.get_center(corner_row[2]));
        let lead = kit.tex(stage, &front, WORK_PT, blue, &[], lead_from)?;
        let tail = kit.tex(stage, &back, WORK_PT, orange, &[], tail_from)?;
        stage.add_to_scene(lead)?;
        stage.add_to_scene(tail)?;
        let grow = card.look.pt() / WORK_PT;
        play!(stage;
            fly(lead, lead_at, grow, 0.8)?,
            fly(tail, tail_at, grow, 0.8)?,
        );
        for &g in &digits[..split] {
            stage.set_color(g, blue, None, true);
        }
        for &g in &digits[split..] {
            stage.set_color(g, orange, None, true);
        }
        stage.remove_from_scene(lead);
        stage.remove_from_scene(tail);

        // 11_3: within nine of fifty it reads straight off; at sixty-two the
        // small square is twelve by twelve and spills a hundred into the
        // hundreds
        let line = say(&kit, stage, "11_3")?;
        let (lo, hi) = digit_pairs_range();
        let limit = props::precisely(&kit, stage, &format!("{READS_OFF} {lo} {TO} {hi}"))?;
        play!(stage; fade_in(stage.arena_mut(), limit, v(0.0, 0.1), 1.0)?.rt(0.5));
        line.wait_sentence(stage, 1)?;
        let mut done = card.mobs();
        done.extend(&hundreds_row);
        done.extend(&corner_row);
        fade_out_all(stage, &done, 0.5)?;
        let name62 = eq(&kit, stage, &format!("{}^2", far.number), NAME_PT, NAME_AT)?;
        play!(stage;
            tween(side, far.number as f64, 2.4)?,
            fade_out(stage.arena_mut(), name46, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), name62, ORIGIN, 1.0)?.rt(0.6),
        );
        // one ruled square of a hundred in the corner of the small square:
        // the small square holds more than that
        let c = moved(far.number as f64).corner;
        if c[2] <= HUNDRED_SIDE {
            return Err(fail(format!(
                "{}'s small square holds no more than a hundred",
                far.number
            )));
        }
        let hundred = [c[0], c[1], HUNDRED_SIDE, HUNDRED_SIDE];
        let rule = board.rect(stage, hundred, line_style(ink, 2.5));
        play!(stage; fade_in(stage.arena_mut(), rule, ORIGIN, 1.0)?.rt(0.5));
        line.wait_sentence(stage, 2)?;
        let d = far.distance.unsigned_abs();
        let sign = if far.distance >= 0 { "+" } else { "-" };
        let mut rows = line_parts(
            &kit,
            stage,
            &[
                part(base.hundreds.to_string(), ink),
                part(sign, ink),
                part(num(d), ink),
                part("=", ink),
                part(far.hundreds.to_string(), blue),
            ],
            SUM_PT,
            v(COL_LEFT, ROW_Y[0]),
            LEFT,
        )?;
        rows.extend(line_parts(
            &kit,
            stage,
            &[
                part(format!("{d}^2"), ink),
                part("=", ink),
                part(num(far.corner), orange),
            ],
            SUM_PT,
            v(COL_LEFT, ROW_Y[1]),
            LEFT,
        )?);
        fade_in_lagged(stage, &rows, 1.0, 0.1)?;
        line.wait_sentence(stage, 3)?;
        let before = line_parts(
            &kit,
            stage,
            &[
                part(num(far.rough), blue),
                part("+", ink),
                part(num(spilled), orange),
                part(&kept, orange),
            ],
            SPILL_PT,
            v(COL_LEFT, ROW_Y[2]),
            LEFT,
        )?;
        let after = line_parts(
            &kit,
            stage,
            &[
                part(num(far.square / 100 * 100), blue),
                part("+", ink),
                part(&kept, orange),
                part("=", ink),
                part(num(far.square / 100), blue),
                part(&kept, orange),
            ],
            SPILL_PT,
            v(COL_LEFT, ROW_Y[2]),
            LEFT,
        )?;
        fade_in_all(stage, &before, 0.6)?;
        // the last sentence says the result; the spill is its picture and
        // starts as the sentence before it ends, not at the sentence's first
        // word
        wait_until(stage, line.sentence(4)? - SPILL_LEAD)?;
        // the hundred peels off the small square and joins the hundreds:
        // from here the small square stands still, as the hundred and the
        // two strips of what is left
        stage.clear_updaters(small, true);
        stage.remove_from_scene(small);
        stage.remove_from_scene(rule);
        let left_over = [
            [c[0] + HUNDRED_SIDE, c[1], c[2] - HUNDRED_SIDE, HUNDRED_SIDE],
            [c[0], c[1] + HUNDRED_SIDE, c[2], c[3] - HUNDRED_SIDE],
        ];
        for r in left_over {
            let m = board.rect(stage, r, flat(orange, 1.0));
            stage.add_to_scene(m)?;
        }
        let peel_fill = board.rect(stage, hundred, flat(orange, 1.0));
        let peel_rule = board.rect(stage, hundred, line_style(ink, 2.5));
        let peel = group(stage, &[peel_fill, peel_rule])?;
        stage.add_to_scene(peel)?;
        stage.scene_mut().bring_to_front(&[small_cut, outline])?;
        // the hundred and the 1 of 144 meet over the hundreds of 3700 (the
        // 1 arcs up over the plus), then drop into them: 37 becomes 38, and
        // the 44 closes up
        let lead = glyphs(stage, before[0]);
        let places = num(far.hundreds.unsigned_abs()).len();
        let hundreds_at = middle_of(stage, lead.get(..places).unwrap_or(&lead))?;
        let over = v(hundreds_at[0], hundreds_at[1] + DROP);
        play!(stage;
            fly(peel, over, PEEL_SCALE, SPILL_FLY_S)?,
            arc_fly(before[2], over, 1.0, SPILL_FLY_S, SPILL_ARC)?,
        );
        let kept_at = stage.get_center(after[2]);
        play!(stage;
            fade_out(stage.arena_mut(), peel, v(0.0, -DROP), 0.5)?.rt(0.5),
            fade_out(stage.arena_mut(), before[2], v(0.0, -DROP), 0.5)?.rt(0.5),
            fade_out(stage.arena_mut(), before[0], ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), after[0], ORIGIN, 1.0)?.rt(0.5),
            fly(before[3], kept_at, 1.0, 0.5)?,
        );
        fade_in_all(stage, &after[3..], 0.5)?;
        #[cfg(test)]
        super::draw::probe::record(&kit.chapter, "sum_up", 0, stage.scene().time().to_f64(), 0.0);

        // 11_4: what made it easy for Bethe, in Feynman's words
        let line = say(&kit, stage, "11_4")?;
        // the picture of 62 squared stays while the story goes on; the
        // card comes with "what made that easy", so its words are not read
        // ahead of the voice (R2, M11)
        line.wait_sentence(stage, 2)?;
        fade_all(stage, 0.6)?;
        let year_line = format!("{SOURCE_BOOK} ({}), {SOURCE_CHAPTER}", cited::FEYNMAN_YEAR);
        let quote = QuoteCard::build(&kit, stage, QUOTE, &[SOURCE_WHO, &year_line], QUOTE_AT, QUOTE_CHARS, 3)?;
        let mut ins = vec![quote.back];
        ins.extend(&quote.lines);
        fade_in_all(stage, &ins, 0.8)?;
        line.wait_sentence(stage, 3)?;
        fade_in_all(stage, &quote.source, 0.6)?;
        kit.music_breath(stage, super::draw::END_HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
