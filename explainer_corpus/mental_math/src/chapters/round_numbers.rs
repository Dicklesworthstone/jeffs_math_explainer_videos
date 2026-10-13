//! Chapter 07, round numbers: five times eighty-six as ten times
//! eighty-six cut in half (07_1); the same cut between two numbers, the
//! top half of fourteen by thirty-five stood beside the bottom half (07_2);
//! a quiz (07_3) worked by the same restack, and where it stops paying
//! (07_4); four things at nineteen ninety-nine, rounded up and the extra
//! given back (07_5: round, then fix); a quiz (07_6) worked the same way,
//! and where that stops paying (07_7).

use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::prelude::*;

use super::draw::{
    Board, NOTE_PT, QuizCard, QuizLook, SIDE_PT, Side, cut_style, dollars, eq, fade_in_all, fade_in_lagged,
    fade_out_all, fail, fit_width, flat, fly, glyphs, gone, grey_line, group, line_style, middle_of, mover, num, panel,
    price, repaint, rgb, say, slate, swap,
};
use crate::facts::shown;
use crate::model::{Ask, by_parts, halve_and_double, money, round_then_fix, times_five};

pub struct RoundNumbers {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const TRUE_NO_EASIER: &str = r"\text{true, and no easier}";
const NO_EASIER_THAN: &str = r"\text{no easier than}";
const CENT: &str = "¢";
/// Cents in a dollar, and the tens a count is rounded to in the aside.
const CENTS_PER_DOLLAR: u64 = 100;
const TENS: u64 = 10;

/// The sums over and under the pictures, and the working beside a card.
const SUM_PT: f64 = 64.0;
const WORK_PT: f64 = 60.0;
/// The width of the gold outline on a piece while it moves.
const EDGE_W: f64 = 5.0;
/// Lines drawn over fills stay above them even when a fill is animated
/// (which brings it to the front); a piece that travels passes over them.
const OVER: i32 = 1;
const TRAVELLING: i32 = 2;

/// 07_1: ten by eighty-six, a long flat bar across the frame.
const FIVE_AT: Vec3 = [-5.59, 0.0, 0.0];
const FIVE_UNIT: f64 = 0.13;
const FIVE_EQ_Y: (f64, f64) = (2.3, -1.65);
/// 07_2: fourteen by thirty-five, and seven by seventy after the restack.
const HALVE_AT: Vec3 = [-5.6, -0.9, 0.0];
const HALVE_UNIT: f64 = 0.16;
const HALVE_EQ_Y: f64 = 2.6;
/// The quiz cards (07_3, 07_6) stand at the right; the working is built at
/// the left: the picture replayed small on top, the sum under it.
const CARD_AT: Vec3 = [3.75, 0.9, 0.0];
const CARD_SCALE: f64 = 0.68;
const SMALL_AT: Vec3 = [-6.1, 1.2, 0.0];
const SMALL_UNIT: f64 = 0.07;
const WORK_AT: Vec3 = [-2.9, -0.5, 0.0];
/// 07_5: the price tags along the top, the cents they shed under them, the
/// rectangle of the things at the round price, and the sum at the foot.
const TAG_Y: f64 = 2.75;
const TAG_STEP: f64 = 3.1;
const TAG_SIZE: (f64, f64) = (2.4, 0.95);
const TAG_PT: f64 = 52.0;
const SHED_Y: f64 = 1.6;
const SHED_PT: f64 = 36.0;
const ROUND_AT: Vec3 = [-5.0, -1.1, 0.0];
const ROUND_ROOM: f64 = 10.0;
/// The extra, a few cents on dollars, is far too thin to see at scale: it is
/// drawn this wide (scene units), and nothing calls it to scale.
const SLIVER: f64 = 0.2;
const SLIVER_GOES: f64 = 0.7;
const ROUND_EQ_Y: f64 = -2.6;
/// 07_7: the round-then-fix rectangle replayed small, and the aside.
const REPLAY_AT: Vec3 = [-5.4, 0.9, 0.0];
const REPLAY_UNIT: f64 = 0.5;
const REPLAY_SLIVER: f64 = 0.125;
const ASIDE_Y: f64 = -2.55;
const ASIDE_ROOM: f64 = 12.6;

/// Keep `mob` (a line over fills) above the fills.
fn keep_over(stage: &mut Stage<'_>, mob: Mob) -> Mob {
    stage.arena_mut().set_z_index(mob, OVER, true);
    mob
}

/// Move `mob` to `to`, shrinking by `scale` and fading away on the way, `rt`
/// seconds: working that goes into the answer already there.
fn sink(mob: Mob, to: Vec3, scale: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .scale(scale)?
        .move_to(to, ORIGIN)?
        .set_opacity(0.0)?)
}

/// A line of mathematics typeset once whole, for its spacing, and returned
/// as its parts, each typeset on its own and centred where it sits in the
/// whole: a part can then come in or move on its own, and no text is ever
/// morphed into another. The whole is centred at `at` (it is only measured,
/// never shown). Detached.
fn line_parts(kit: &Kit, stage: &mut Stage<'_>, parts: &[(&str, Srgb)], size: f64, at: Vec3) -> fmn::Result<Vec<Mob>> {
    let src = parts.iter().map(|(s, _)| *s).collect::<Vec<_>>().join(" ");
    let whole = eq(kit, stage, &src, size, at)?;
    let marks = glyphs(stage, whole);
    let mut out = Vec::new();
    let mut k = 0;
    for &(part, color) in parts {
        let m = kit.tex(stage, part, size, color, &[], at)?;
        let n = glyphs(stage, m).len();
        let Some(spot) = marks.get(k..k + n) else {
            return Err(fail(format!("{part:?} runs past the end of {src:?}")));
        };
        let spot = middle_of(stage, spot)?;
        stage.move_to(m, spot, ORIGIN);
        out.push(m);
        k += n;
    }
    if k != marks.len() {
        return Err(fail(format!("the parts of {src:?} are not all of it")));
    }
    Ok(out)
}

/// The working's parts go into the answer on the card (copies, so the
/// working stays), and the answer answers back.
fn into_answer(kit: &Kit, stage: &mut Stage<'_>, parts: &[(&str, Srgb)], at: Vec3, card: &QuizCard) -> fmn::Result<()> {
    let copies = line_parts(kit, stage, parts, WORK_PT, at)?;
    for &m in &copies {
        stage.add_to_scene(m)?;
    }
    let to = card.answer_at();
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in &copies {
        anims.push(stage.prepare(sink(m, to, 0.5, 0.7)?)?);
    }
    stage.play_prepared(anims)?;
    for &m in &copies {
        stage.remove_from_scene(m);
    }
    let ink = Palette::ink();
    play!(stage; fmn::animation::indicate(stage.arena_mut(), card.answer, 1.12, Some(rgb(ink)))?.rt(0.6));
    Ok(())
}

/// A lower-third whose whole text is one line of mathematics, in the band
/// of `draw::lower_third_tex` but with no label (07_4's needs none).
/// Detached, one group.
fn lower_third_math(kit: &Kit, stage: &mut Stage<'_>, src: &str) -> fmn::Result<Mob> {
    let w = 2.0 * FRAME_X_RADIUS - 0.8;
    let h = 0.78;
    let centre = v(0.0, -FRAME_Y_RADIUS + 0.2 + h / 2.0);
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(centre);
    let back = stage.arena_mut().add(back);
    let math = eq(kit, stage, src, 34.0, centre)?;
    fit_width(stage, math, w - 0.5);
    group(stage, &[back, math])
}

/// A quiz on line `id` with its card at the right: say the line, clear the
/// stage as it begins, bring the card in as the question begins, count
/// down, land the answer.
fn quiz_beat(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<QuizCard> {
    let line = say(kit, stage, id)?;
    fade_all(stage, 0.5)?;
    let card = QuizCard::build(kit, stage, id, QuizLook::at(CARD_AT, CARD_SCALE))?;
    line.wait_last(stage)?;
    card.show(stage, 0.5)?;
    card.count(kit, stage)?;
    Ok(card)
}

/// A rectangle `tall` by `wide` cut in half across: the top half can fade
/// away (times five as times ten, halved) or slide down beside the bottom
/// half, outlined in gold while it moves, to make a rectangle half as tall
/// and twice as wide (halve and double). On a board of ones, the bottom
/// half at the origin.
struct Halves {
    board: Board,
    tall: f64,
    wide: f64,
    bottom: Mob,
    top: Mob,
    edge: Mob,
    cut: Mob,
    seam: Mob,
}

impl Halves {
    /// Detached.
    fn build(stage: &mut Stage<'_>, board: Board, tall: u64, wide: u64) -> Self {
        let (tall, wide) = (tall as f64, wide as f64);
        let h = tall / 2.0;
        let bottom = board.rect(stage, [0.0, 0.0, wide, h], flat(slate(), 1.0));
        let top = board.rect(stage, [0.0, h, wide, h], flat(slate(), 1.0));
        let edge = board.rect(stage, [0.0, h, wide, h], line_style(mover(), EDGE_W));
        let cut = board.line(stage, [0.0, h], [wide, h], cut_style());
        let seam = board.line(stage, [wide, 0.0], [wide, h], cut_style());
        Self {
            board,
            tall,
            wide,
            bottom,
            top,
            edge,
            cut: keep_over(stage, cut),
            seam: keep_over(stage, seam),
        }
    }

    /// The whole rectangle, the half that stays, and the restacked one, as
    /// `[x, y, wide, tall]`.
    fn whole(&self) -> [f64; 4] {
        [0.0, 0.0, self.wide, self.tall]
    }

    fn kept(&self) -> [f64; 4] {
        [0.0, 0.0, self.wide, self.tall / 2.0]
    }

    fn restacked(&self) -> [f64; 4] {
        [0.0, 0.0, 2.0 * self.wide, self.tall / 2.0]
    }

    /// The rectangle before it is cut.
    fn shape(&self) -> [Mob; 2] {
        [self.bottom, self.top]
    }

    /// The cut across the middle.
    fn cut(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_all(stage, &[self.cut], rt)
    }

    /// The top half is taken away: it takes the colour of what goes, and
    /// fades.
    fn drop_top(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        play!(stage; repaint(self.top, gone(), 1.0, 0.35)?);
        fade_out_all(stage, &[self.top, self.cut], 0.6)
    }

    /// The top half slides down beside the bottom half, outlined in gold
    /// while it moves, `rt` seconds.
    fn slide(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.remove_from_scene(self.cut);
        for m in [self.top, self.edge] {
            stage.arena_mut().set_z_index(m, TRAVELLING, true);
        }
        stage.add_to_scene(self.top)?;
        fade_in_all(stage, &[self.edge], 0.15)?;
        let to = self.board.centre([self.wide, 0.0, self.wide, self.tall / 2.0]);
        play!(stage; fly(self.top, to, 1.0, rt)?, fly(self.edge, to, 1.0, rt)?);
        fade_out_all(stage, &[self.edge], 0.2)?;
        stage.arena_mut().set_z_index(self.top, 0, true);
        fade_in_all(stage, &[self.seam], 0.15)
    }
}

/// Round, then fix, as a rectangle: `count` rows, one a thing, each the
/// round price wide (in dollars), with the extra at the end of each row (a
/// cent or so, drawn `sliver` units wide) sliced off in red.
struct Rounded {
    board: Board,
    count: usize,
    wide: f64,
    sliver: f64,
    /// Each row less its extra, back to front.
    rows: Vec<Mob>,
    /// The extra at the end of each row, from the bottom row up.
    extras: Vec<Mob>,
    cuts: Vec<Mob>,
}

impl Rounded {
    /// Detached.
    fn build(stage: &mut Stage<'_>, board: Board, count: u64, wide: f64, sliver: f64) -> Self {
        let count = count as usize;
        let body = wide - sliver;
        let rows = (0..count)
            .map(|i| board.rect(stage, [0.0, i as f64, body, 1.0], flat(slate(), 1.0)))
            .collect();
        let extras = (0..count)
            .map(|i| board.rect(stage, [body, i as f64, sliver, 1.0], flat(slate(), 1.0)))
            .collect();
        let cuts = (1..count)
            .map(|i| {
                let m = board.line(stage, [0.0, i as f64], [wide, i as f64], cut_style());
                keep_over(stage, m)
            })
            .collect();
        Self {
            board,
            count,
            wide,
            sliver,
            rows,
            extras,
            cuts,
        }
    }

    fn whole(&self) -> [f64; 4] {
        [0.0, 0.0, self.wide, self.count as f64]
    }

    /// The rectangle before anything is given back.
    fn shape(&self) -> Vec<Mob> {
        let mut out = self.rows.clone();
        out.extend(&self.extras);
        out.extend(&self.cuts);
        out
    }

    /// Where the extra of row `i` (from the top) sits.
    fn extra_at(&self, i: usize) -> Vec3 {
        let row = (self.count - 1 - i.min(self.count - 1)) as f64;
        self.board.centre([self.wide - self.sliver, row, self.sliver, 1.0])
    }

    /// The extras turn red, `rt` seconds, as animations to play with others.
    fn redden(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        self.extras
            .iter()
            .map(|&m| stage.prepare(repaint(m, gone(), 1.0, rt)?))
            .collect()
    }

    /// The red extras slide off the end, `shift` scene units, `rt` seconds.
    fn slice_off(&self, stage: &mut Stage<'_>, shift: f64, rt: f64) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &self.extras {
            stage.arena_mut().set_z_index(m, TRAVELLING, true);
            let c = stage.get_center(m);
            anims.push(stage.prepare(fly(m, v(c[0] + shift, c[1]), 1.0, rt)?)?);
        }
        stage.play_prepared(anims)?;
        Ok(())
    }
}

/// A price tag: a rounded card with a hole at its left and a price.
struct Tag {
    back: Mob,
    hole: Mob,
    price: Mob,
}

impl Tag {
    /// The tag centred at `at`, its price `src` (TeX). Detached.
    fn build(kit: &Kit, stage: &mut Stage<'_>, src: &str, at: Vec3) -> fmn::Result<Self> {
        let (w, h) = TAG_SIZE;
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.14)
            .style(Style::default().fill(panel(), 0.95).stroke(Palette::dim(), 2.0, 1.0))
            .build()?
            .moved_to(at);
        let back = stage.arena_mut().add(back);
        let hole = stage.arena_mut().add(
            Circle::new()
                .radius(0.09)
                .arc_center(v(at[0] - w / 2.0 + 0.27, at[1]))
                .style(line_style(Palette::dim(), 2.0))
                .build(),
        );
        let price = eq(kit, stage, src, TAG_PT, Self::price_at(at))?;
        Ok(Self { back, hole, price })
    }

    fn price_at(at: Vec3) -> Vec3 {
        v(at[0] + 0.15, at[1])
    }

    fn mobs(&self) -> [Mob; 3] {
        [self.back, self.hole, self.price]
    }
}

impl SceneConstruct for RoundNumbers {
    fn name(&self) -> &str {
        "07_round_numbers"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();

        // 07_1: five times eighty-six is ten times eighty-six, cut in half
        let line = say(&kit, stage, "07_1")?;
        let x = shown::TIMES_FIVE;
        let (ten_times, half_of) = times_five(x);
        let (tall, short) = (ten_times / x, half_of / x);
        if !ten_times.is_multiple_of(x) || !half_of.is_multiple_of(x) || tall != 2 * short {
            return Err(fail(format!(
                "{ten_times} and {half_of} are not ten and five times {x}"
            )));
        }
        let board = Board::new(FIVE_AT, FIVE_UNIT);
        let bar = Halves::build(stage, board, tall, x);
        let ten_label = board.label(&kit, stage, &num(tall), bar.whole(), Side::Left, SIDE_PT, ink)?;
        let wide_label = board.label(&kit, stage, &num(x), bar.whole(), Side::Below, SIDE_PT, ink)?;
        let mut ins = bar.shape().to_vec();
        ins.extend([ten_label, wide_label]);
        fade_in_all(stage, &ins, 0.6)?;
        let (ten_src, ten_area) = (format!(r"{tall} \times {x}"), num(ten_times));
        let ten_sum = line_parts(
            &kit,
            stage,
            &[(&ten_src, ink), ("=", ink), (&ten_area, ink)],
            SUM_PT,
            v(0.0, FIVE_EQ_Y.0),
        )?;
        line.wait_for(stage, "ten times eighty-six")?;
        play!(stage; fade_in(stage.arena_mut(), ten_sum[0], v(0.0, -0.12), 1.0)?.rt(0.5));
        line.wait_for(stage, "cut in half. Eight")?;
        bar.cut(stage, 0.4)?;
        line.wait_sentence(stage, 2)?;
        fade_in_all(stage, &ten_sum[1..], 0.4)?;
        line.wait_for(stage, "cut in half. Four")?;
        let five_label = board.label(&kit, stage, &num(short), bar.kept(), Side::Left, SIDE_PT, ink)?;
        bar.drop_top(stage)?;
        swap(stage, &[ten_label], &[five_label], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let five_sum = eq(
            &kit,
            stage,
            &format!(r"{short} \times {x} = {half_of}"),
            SUM_PT,
            v(0.0, FIVE_EQ_Y.1),
        )?;
        play!(stage; fade_in(stage.arena_mut(), five_sum, v(0.0, 0.12), 1.0)?.rt(0.5));

        // 07_2: the same cut between two numbers: half of fourteen, double
        // thirty-five; the top half slides down beside the bottom half
        let line = say(&kit, stage, "07_2")?;
        fade_all(stage, 0.5)?;
        let (a, b) = shown::HALVE;
        let (half, double, product) = halve_and_double(a, b).ok_or_else(|| fail(format!("{a} cannot be halved")))?;
        let board = Board::new(HALVE_AT, HALVE_UNIT);
        let rect = Halves::build(stage, board, a, b);
        let a_label = board.label(&kit, stage, &num(a), rect.whole(), Side::Left, SIDE_PT, ink)?;
        let b_label = board.label(&kit, stage, &num(b), rect.whole(), Side::Below, SIDE_PT, ink)?;
        let (before, after, area) = (
            format!(r"{a} \times {b}"),
            format!(r"{half} \times {double}"),
            num(product),
        );
        let sum = line_parts(
            &kit,
            stage,
            &[(&before, ink), ("=", ink), (&after, ink), ("=", ink), (&area, ink)],
            SUM_PT,
            v(0.0, HALVE_EQ_Y),
        )?;
        fade_in_all(stage, &rect.shape(), 0.6)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[a_label, b_label, sum[0]], 0.5)?;
        line.wait_sentence(stage, 2)?;
        rect.cut(stage, 0.4)?;
        line.wait_for(stage, "and double")?;
        fade_out_all(stage, &[a_label, b_label], 0.3)?;
        rect.slide(stage, 1.2)?;
        line.wait_sentence(stage, 3)?;
        let half_label = board.label(&kit, stage, &num(half), rect.kept(), Side::Left, SIDE_PT, ink)?;
        let double_label = board.label(&kit, stage, &num(double), rect.restacked(), Side::Below, SIDE_PT, ink)?;
        fade_in_all(stage, &[half_label, double_label, sum[1], sum[2]], 0.5)?;
        line.wait_sentence(stage, 4)?;
        fade_in_all(stage, &sum[3..], 0.5)?;

        // 07_3: your turn: sixteen times forty-five
        let card = quiz_beat(&kit, stage, "07_3")?;

        // 07_4: half of one, double the other, with the restack replayed
        // small beside the card; it pays when doubling makes a round number
        let line = say(&kit, stage, "07_4")?;
        let Ask::Times(p, q) = card.quiz.ask else {
            return Err(fail("07_3 does not ask a product".into()));
        };
        let (half, double, product) = halve_and_double(p, q).ok_or_else(|| fail(format!("{p} cannot be halved")))?;
        let answer = card.quiz.worked().map_err(fail)?.answer;
        if num(product) != answer {
            return Err(fail(format!("the restack gives {product}, the card {answer}")));
        }
        let board = Board::new(SMALL_AT, SMALL_UNIT);
        let small = Halves::build(stage, board, p, q);
        let p_label = board.label(&kit, stage, &num(p), small.whole(), Side::Left, NOTE_PT, ink)?;
        let q_label = board.label(&kit, stage, &num(q), small.whole(), Side::Below, NOTE_PT, ink)?;
        let (p_src, q_src, half_src, double_src) = (num(p), num(q), num(half), num(double));
        let work_parts = [
            (p_src.as_str(), ink),
            (r"\times", ink),
            (q_src.as_str(), ink),
            ("=", ink),
            (half_src.as_str(), ink),
            (r"\times", ink),
            (double_src.as_str(), ink),
        ];
        let work = line_parts(&kit, stage, &work_parts, WORK_PT, WORK_AT)?;
        let mut ins = small.shape().to_vec();
        ins.extend([p_label, q_label]);
        ins.extend(&work[..3]);
        fade_in_all(stage, &ins, 0.4)?;
        small.cut(stage, 0.25)?;
        fade_out_all(stage, &[p_label, q_label], 0.2)?;
        small.slide(stage, 0.9)?;
        let half_label = board.label(&kit, stage, &num(half), small.kept(), Side::Left, NOTE_PT, ink)?;
        let double_label = board.label(&kit, stage, &num(double), small.restacked(), Side::Below, NOTE_PT, ink)?;
        fade_in_all(stage, &[half_label, double_label], 0.3)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &work[3..], 0.5)?;
        line.wait_sentence(stage, 2)?;
        into_answer(&kit, stage, &work_parts[4..], middle_of(stage, &work[4..])?, &card)?;
        line.wait_sentence(stage, 3)?;
        // the double lands on a round number
        play!(stage; fmn::animation::indicate(stage.arena_mut(), work[6], 1.25, Some(rgb(ink)))?.rt(0.8));
        let (x, y) = shown::HALVE_NO_EASIER;
        let (xh, yd, _) = halve_and_double(x, y).ok_or_else(|| fail(format!("{x} cannot be halved")))?;
        let no_easier = lower_third_math(
            &kit,
            stage,
            &format!(r"{x} \times {y} = {xh} \times {yd}:\ {TRUE_NO_EASIER}"),
        )?;
        play!(stage; fade_in(stage.arena_mut(), no_easier, v(0.0, 0.15), 1.0)?.rt(0.5));
        line.wait_sentence(stage, 4)?;
        // one of them even, the other ending in five
        play!(stage;
            fmn::animation::indicate(stage.arena_mut(), work[0], 1.25, Some(rgb(ink)))?.rt(0.8),
            fmn::animation::indicate(stage.arena_mut(), work[2], 1.25, Some(rgb(ink)))?.rt(0.8),
        );

        // 07_5: a price just under a round number: round up, then give the
        // extra back
        let line = say(&kit, stage, "07_5")?;
        fade_all(stage, 0.5)?;
        let (count, each) = shown::PRICE;
        let r = round_then_fix(count, each, each.next_multiple_of(CENTS_PER_DOLLAR));
        let extra = r.round - r.each;
        if r.fix != count * extra {
            return Err(fail(format!("{} cents back is not {count} times {extra}", r.fix)));
        }
        let n = count as usize;
        let tag_x = |i: usize| (i as f64 - (n as f64 - 1.0) / 2.0) * TAG_STEP;
        let tags: Vec<Tag> = (0..n)
            .map(|i| Tag::build(&kit, stage, &price(r.each), v(tag_x(i), TAG_Y)))
            .collect::<fmn::Result<_>>()?;
        let tag_groups: Vec<Mob> = tags
            .iter()
            .map(|t| group(stage, &t.mobs()))
            .collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &tag_groups[..1], 0.4)?;
        line.wait_sentence(stage, 1)?;
        fade_in_lagged(stage, &tag_groups[1..], 0.9, 0.4)?;
        line.wait_sentence(stage, 2)?;
        // each rounds up and sheds a red cent
        let rounded_prices: Vec<Mob> = (0..n)
            .map(|i| {
                eq(
                    &kit,
                    stage,
                    &dollars(r.round),
                    TAG_PT,
                    Tag::price_at(v(tag_x(i), TAG_Y)),
                )
            })
            .collect::<fmn::Result<_>>()?;
        let old_prices: Vec<Mob> = tags.iter().map(|t| t.price).collect();
        swap(stage, &old_prices, &rounded_prices, 0.5)?;
        let shed: Vec<Mob> = (0..n)
            .map(|i| kit.text(stage, &format!("{extra}{CENT}"), SHED_PT, gone(), v(tag_x(i), SHED_Y)))
            .collect::<fmn::Result<_>>()?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &shed {
            let a = fade_in(stage.arena_mut(), m, v(0.0, -0.35), 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, 0.3)?.with_run_time(0.8);
        stage.play(g)?;
        // the things at the round price: a rectangle, a row a thing
        let wide = (r.round / CENTS_PER_DOLLAR) as f64;
        let unit = ROUND_ROOM / wide;
        let board = Board::new(ROUND_AT, unit);
        let things = Rounded::build(stage, board, count, wide, SLIVER / unit);
        let count_label = board.label(&kit, stage, &num(count), things.whole(), Side::Left, SIDE_PT, ink)?;
        let round_label = board.label(
            &kit,
            stage,
            &format!("${}", money(r.round)),
            things.whole(),
            Side::Below,
            SIDE_PT,
            ink,
        )?;
        let mut ins = things.shape();
        ins.extend([count_label, round_label]);
        fade_in_all(stage, &ins, 0.5)?;
        let (rounded_src, fix_src, total_src) = (dollars(r.rounded), dollars(r.fix), dollars(r.total));
        let sum = line_parts(
            &kit,
            stage,
            &[
                (&rounded_src, ink),
                ("-", ink),
                (&fix_src, gone()),
                ("=", ink),
                (&total_src, ink),
            ],
            SUM_PT,
            v(0.0, ROUND_EQ_Y),
        )?;
        line.wait_for(stage, "eighty dollars")?;
        play!(stage; fade_in(stage.arena_mut(), sum[0], v(0.0, 0.12), 1.0)?.rt(0.4));
        line.wait_sentence(stage, 3)?;
        // give back the cents: each falls into the end of its row, which
        // turns red, and the red sliver is sliced off
        let mut anims = things.redden(stage, 0.6)?;
        for (i, &m) in shed.iter().enumerate() {
            anims.push(stage.prepare(sink(m, things.extra_at(i), 0.6, 0.7)?)?);
        }
        stage.play_prepared(anims)?;
        things.slice_off(stage, SLIVER_GOES, 0.6)?;
        fade_in_all(stage, &sum[1..3], 0.4)?;
        line.wait_sentence(stage, 4)?;
        fade_in_all(stage, &sum[3..], 0.5)?;

        // 07_6: your turn: three things at four ninety-nine
        let card = quiz_beat(&kit, stage, "07_6")?;

        // 07_7: three times five, give back three cents; it pays when the fix
        // is small
        let line = say(&kit, stage, "07_7")?;
        let Ask::TimesMoney(count, each) = card.quiz.ask else {
            return Err(fail("07_6 does not ask a count of a price".into()));
        };
        let r = round_then_fix(count, each, each.next_multiple_of(CENTS_PER_DOLLAR));
        let answer = card.quiz.worked().map_err(fail)?.answer;
        if money(r.total) != answer {
            return Err(fail(format!("round, then fix gives {}, the card {answer}", r.total)));
        }
        let board = Board::new(REPLAY_AT, REPLAY_UNIT);
        let wide = (r.round / CENTS_PER_DOLLAR) as f64;
        let replay = Rounded::build(stage, board, count, wide, REPLAY_SLIVER / REPLAY_UNIT);
        let count_label = board.label(&kit, stage, &num(count), replay.whole(), Side::Left, NOTE_PT, ink)?;
        let round_label = board.label(
            &kit,
            stage,
            &format!("${}", money(r.round)),
            replay.whole(),
            Side::Below,
            NOTE_PT,
            ink,
        )?;
        let (rounded_src, fix_src) = (dollars(r.rounded), dollars(r.fix));
        let work_parts = [(rounded_src.as_str(), ink), ("-", ink), (fix_src.as_str(), gone())];
        let work = line_parts(&kit, stage, &work_parts, WORK_PT, WORK_AT)?;
        let mut ins = replay.shape();
        ins.extend([count_label, round_label]);
        fade_in_all(stage, &ins, 0.5)?;
        line.wait_for(stage, "fifteen dollars")?;
        play!(stage; fade_in(stage.arena_mut(), work[0], v(0.0, 0.12), 1.0)?.rt(0.4));
        line.wait_sentence(stage, 1)?;
        let anims = replay.redden(stage, 0.4)?;
        stage.play_prepared(anims)?;
        replay.slice_off(stage, SLIVER_GOES / 2.0, 0.5)?;
        fade_in_all(stage, &work[1..], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let at = middle_of(stage, &work)?;
        into_answer(&kit, stage, &work_parts, at, &card)?;
        // where it stops paying: a round number too far away
        line.wait_sentence(stage, 3)?;
        let (c, e) = shown::FIX_NO_EASIER;
        let far = round_then_fix(c, e, e.next_multiple_of(TENS));
        let parts = by_parts(c, e);
        let [first, second] = parts.pieces[..] else {
            return Err(fail(format!("{c} times {e} is not two pieces")));
        };
        let aside_src = format!(
            r"{c} \times {e} = {} - {}:\ {NO_EASIER_THAN}\ {} + {}",
            far.rounded,
            far.fix,
            first.area(),
            second.area()
        );
        let aside = grey_line(&kit, stage, &aside_src, v(0.0, ASIDE_Y))?;
        fit_width(stage, aside, ASIDE_ROOM);
        play!(stage; fade_in(stage.arena_mut(), aside, ORIGIN, 1.0)?.rt(0.5));
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
