//! Chapter 06, percents: a tip on a bill of forty-eight dollars, from its
//! tenth (06_1: the bill as a bar cut into ten, one tenth lifted out, the
//! point one place to the left), half of the tenth and the two together
//! (06_2) and the tenth doubled (06_3); a quiz on sixty dollars (06_4)
//! worked on a bar of ten (06_5); eight percent of twenty-five dollars
//! (06_6) by the swap (06_7); and why the swap holds: a grid of cents read
//! by its columns and by its rows, which fill two squares of a hundred
//! (06_8).

use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Board, QuizCard, QuizLook, SIDE_PT, Side, cut_style, dim, dollars, eq, fade_in_all, fade_in_lagged, fade_out_all,
    fail, fit_width, flat, fly, glyphs, grey_line, ground, group, hundreds, line_style, lower_third_tex, middle_of,
    mover, num, panel, pause, quiz, repaint, rgb, say, slate, swap, why_pause,
};
use crate::facts::shown;
use crate::model::{Ask, blocks, money, percent_of};

pub struct Percents {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ONE_CELL: &str = "one cell, one cent";
const CENT: &str = "¢";
const FROM_EACH_DOLLAR: &str = "from each dollar";
const FROM_EACH: &str = "from each";
const A_QUARTER_OF: &str = "a quarter of";
/// The words after the seventeen percent (06_3); `FOUR` is the count they say.
const FOUR_PIECES: &str = r"\text{four pieces, a running total again}";
const FOUR: usize = 4;
const PRECISELY: &str = "precisely:";
const PRECISELY_TEX: &str = r"x\% \text{ of } y = x \times y \div 100 = y\% \text{ of } x";
/// The middle of "8% of $25", which stays put while the numbers trade places.
const PERCENT_OF: &str = r"\% \text{ of } \$";
const ASK: &str = "?";
/// The friendly percents (06_8): a part of a dollar by its name, and how
/// many such parts make the dollar (the percent is a hundred over it).
const FRIENDLY: [(&str, u64); 3] = [("a tenth", 10), ("a quarter", 4), ("a half", 2)];
/// Percent: parts in a hundred.
const PER_CENT: u64 = 100;

/// The fill of a lit piece of a bill (the tenth, its half), as the bill's
/// small picture lights its tenth: ink at 85% over the background, made
/// opaque so that a lit piece looks the same over the bar or beside it.
const LIT: &str = "#CBCED3";
/// Lines drawn over fills (cuts, a grid's lines) stay above the fills even
/// when a fill is animated (which brings it to the front); a piece that
/// travels passes over them.
const OVER: i32 = 1;
const TRAVELLING: i32 = 2;
/// The width of the gold outline on a piece while it moves.
const EDGE_W: f64 = 5.0;

/// The bill's bar (06_1 to 06_3): its lower-left corner, scene units per
/// tenth, and its height in tenths.
const BAR_AT: Vec3 = [-6.0, 0.8, 0.0];
const BAR_UNIT: f64 = 1.2;
const BAR_TALL: f64 = 1.0;
/// The bill's digits over the bar, and the room a point takes between two.
const PLACES_PT: f64 = 96.0;
const PLACES_Y: f64 = 3.0;
const POINT_ROOM: f64 = 0.16;
/// How far the point clears the digit it hops over.
const HOP_CLEAR: f64 = 0.25;
/// The pieces lifted out of the bill stand in four columns, with their
/// percent and their dollars under them.
const COLS_X: [f64; 4] = [-4.65, -1.55, 1.55, 4.65];
const PIECE_Y: f64 = -0.35;
const PCT_Y: f64 = -1.5;
const MONEY_Y: f64 = -2.3;
const PCT_PT: f64 = 48.0;
const MONEY_PT: f64 = 56.0;
const GREY_Y: f64 = -3.25;
const GREY_ROOM: f64 = 12.6;

/// The quiz card at the top (06_4), and the bar of ten under it (06_5).
const CARD_AT: Vec3 = [0.0, 1.75, 0.0];
const CARD_SCALE: f64 = 0.72;
const SMALL_BAR_AT: Vec3 = [-5.0, -1.55, 0.0];
const SMALL_UNIT: f64 = 1.0;
const SMALL_TALL: f64 = 0.9;
const SUM_Y: f64 = -2.5;
const SUM_PT: f64 = 60.0;

/// The question with no ring (06_6) and the swap (06_7).
const SWAP_PT: f64 = 96.0;
const SWAP_AT: Vec3 = [0.0, 0.8, 0.0];
const SWAP_ARC: f64 = -2.6;
const SWAP_S: f64 = 1.1;
const QUARTER_Y: f64 = -1.15;

/// The grid of cents (06_8): its lower-left corner and the size of a cell;
/// the squares of a hundred, the lower-left of the first and the gap
/// between them (scene units); the line of the swap moved to the top.
const GRID_AT: Vec3 = [-5.0, -1.0, 0.0];
const CELL: f64 = 0.4;
const SQUARES_AT: [f64; 2] = [-6.4, -1.6];
const SQUARE_GAP: f64 = 0.6;
const CAPTION_Y: f64 = -2.1;
const CAPTION_PT: f64 = 40.0;
const TOP_LINE: Vec3 = [0.0, 3.25, 0.0];
const TOP_SCALE: f64 = 0.62;
const FRIENDLY_X: (f64, f64) = (3.75, 4.5);
const FRIENDLY_Y: [f64; 3] = [1.7, 0.45, -0.8];
const FRIENDLY_PT: (f64, f64) = (56.0, 36.0);

/// A lit piece of a bill.
fn lit_color() -> Srgb {
    hex(LIT)
}

fn lit() -> Style {
    flat(lit_color(), 1.0)
}

/// Keep `mob` (a line over fills) above the fills.
fn keep_over(stage: &mut Stage<'_>, mob: Mob) -> Mob {
    stage.arena_mut().set_z_index(mob, OVER, true);
    mob
}

/// A rectangle `w` wide and `h` tall centred at `at`. Detached.
fn block(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, style: Style) -> Mob {
    let (x, y) = (at[0] - w / 2.0, at[1] - h / 2.0);
    let quad = [v(x, y), v(x + w, y), v(x + w, y + h), v(x, y + h)];
    stage.arena_mut().add(Polygon::new(quad).style(style).build())
}

/// A cut across a fill: a line in the background's colour from `a` to `b`.
/// Detached.
fn cut_line(stage: &mut Stage<'_>, a: Vec3, b: Vec3) -> Mob {
    let m = stage.arena_mut().add(polyline(vec![a, b]).style(cut_style()).build());
    keep_over(stage, m)
}

/// Move `mob` so its centre is at `to` along an arc of `arc` radians
/// (positive: counter-clockwise), `rt` seconds.
fn glide(mob: Mob, to: Vec3, arc: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            path_arc: Some(arc),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?)
}

/// Move `mob` to `to`, shrinking by `scale` and fading away on the way, `rt`
/// seconds: a sum that goes into the answer already there.
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

/// Prepared animations one after another (a lagged group), `rt` in all.
fn play_lagged(stage: &mut Stage<'_>, anims: Vec<Box<dyn Animation>>, rt: f64, lag: f64) -> fmn::Result<()> {
    if anims.is_empty() {
        return Ok(());
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// A line of mathematics typeset once whole, for its spacing, and returned
/// as its parts, each typeset on its own and centred where it sits in the
/// whole: a part can then come in, move or trade places on its own, and no
/// text is ever morphed into another. The whole is centred at `at` (it is
/// only measured, never shown). Detached.
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

/// Pieces that lift out of a picture as copies and travel, outlined in the
/// move's gold while they travel (never filled gold): each is `(from, w,
/// h, to)`, a lit rectangle `w` by `h` centred at `from` going to `to`. The
/// copies are returned, in the scene.
fn lift(stage: &mut Stage<'_>, pieces: &[(Vec3, f64, f64, Vec3)], rt: f64) -> fmn::Result<Vec<Mob>> {
    let mut fills = Vec::new();
    let mut edges = Vec::new();
    for &(from, w, h, _) in pieces {
        fills.push(block(stage, from, w, h, lit()));
        edges.push(block(stage, from, w, h, line_style(mover(), EDGE_W)));
    }
    for (&f, &e) in fills.iter().zip(&edges) {
        stage.arena_mut().set_z_index(f, TRAVELLING, true);
        stage.arena_mut().set_z_index(e, TRAVELLING, true);
        stage.add_to_scene(f)?;
    }
    fade_in_all(stage, &edges, 0.2)?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (i, &(_, _, _, to)) in pieces.iter().enumerate() {
        anims.push(stage.prepare(fly(fills[i], to, 1.0, rt)?)?);
        anims.push(stage.prepare(fly(edges[i], to, 1.0, rt)?)?);
    }
    stage.play_prepared(anims)?;
    fade_out_all(stage, &edges, 0.25)?;
    // landed: an ordinary fill again, under the cuts drawn over it later
    for &f in &fills {
        stage.arena_mut().set_z_index(f, 0, true);
    }
    Ok(fills)
}

/// A bill as a long bar cut into tenths: abutting rectangles on a board
/// whose unit is one tenth wide, with a cut over each seam.
struct Bar {
    board: Board,
    tall: f64,
    tenths: Vec<Mob>,
    cuts: Vec<Mob>,
}

impl Bar {
    /// `n` tenths, `tall` units high, in the shape's first colour. Detached.
    fn build(stage: &mut Stage<'_>, board: Board, n: u64, tall: f64) -> Self {
        let n = n as usize;
        let tenths = (0..n)
            .map(|i| board.rect(stage, [i as f64, 0.0, 1.0, tall], flat(slate(), 1.0)))
            .collect();
        let cuts = (1..n)
            .map(|i| {
                let m = board.line(stage, [i as f64, 0.0], [i as f64, tall], cut_style());
                keep_over(stage, m)
            })
            .collect();
        Self {
            board,
            tall,
            tenths,
            cuts,
        }
    }

    /// Everything of the bar, back to front.
    fn mobs(&self) -> Vec<Mob> {
        let mut out = self.tenths.clone();
        out.extend(&self.cuts);
        out
    }

    /// The scene centre of the stretch from `x` tenths, `w` tenths wide.
    fn centre(&self, x: f64, w: f64) -> Vec3 {
        self.board.centre([x, 0.0, w, self.tall])
    }

    /// A tenth's width and height in scene units.
    fn tenth(&self) -> (f64, f64) {
        (self.board.unit, self.board.unit * self.tall)
    }
}

/// An amount's digits in fixed places with a point that can sit between
/// any two of them (06_1: the point moves, the digits stay).
struct Places {
    sign: Mob,
    digits: Vec<Mob>,
    point: Mob,
    /// Where the point sits after digit `k + 1`, for each `k`.
    spots: Vec<Vec3>,
    /// How many digits stand before the point now.
    before: usize,
}

impl Places {
    /// `cents` as dollars, its digits typeset once each, `size` points,
    /// centred at `at`. Detached.
    fn build(kit: &Kit, stage: &mut Stage<'_>, cents: u64, size: f64, at: Vec3) -> fmn::Result<Self> {
        let amount = money(cents);
        let (whole, part) = amount
            .split_once('.')
            .ok_or_else(|| fail(format!("{amount} has no point")))?;
        let digits: Vec<String> = whole.chars().chain(part.chars()).map(String::from).collect();
        let ink = Palette::ink();
        let mut parts: Vec<(&str, Srgb)> = vec![(r"\$", ink)];
        parts.extend(digits.iter().map(|d| (d.as_str(), ink)));
        let mobs = line_parts(kit, stage, &parts, size, at)?;
        // each digit after the first moves right, to make room for a point
        // in every gap; then the whole row is centred again
        let n = mobs.len();
        let shift = POINT_ROOM * (n as f64 - 2.0) / 2.0;
        for (i, &m) in mobs.iter().enumerate() {
            let c = stage.get_center(m);
            let room = POINT_ROOM * (i.max(1) as f64 - 1.0);
            stage.move_to(m, v(c[0] + room - shift, c[1]), ORIGIN);
        }
        let (sign, digits) = (mobs[0], mobs[1..].to_vec());
        let base = stage.get_bottom(digits[0])[1];
        let radius = size * 0.0006;
        let spots: Vec<Vec3> = (1..digits.len())
            .map(|k| {
                let x = (stage.get_right(digits[k - 1])[0] + stage.get_left(digits[k])[0]) / 2.0;
                v(x, base + radius)
            })
            .collect();
        let before = whole.chars().count();
        let at_point = *spots
            .get(before - 1)
            .ok_or_else(|| fail(format!("{amount} has no place for its point")))?;
        let point = stage
            .arena_mut()
            .add(Dot::new().point(at_point).radius(radius).style(flat(ink, 1.0)).build());
        Ok(Self {
            sign,
            digits,
            point,
            spots,
            before,
        })
    }

    fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.sign];
        out.extend(&self.digits);
        out.push(self.point);
        out
    }

    /// The amount as it reads now.
    fn reads(&self, cents: u64) -> String {
        let amount = money(cents).replace('.', "");
        let (a, b) = amount.split_at(self.before.min(amount.len()));
        format!("{a}.{b}")
    }

    /// The point hops one place to the left, over the digit it passes, `rt`
    /// seconds (an arc's own path is too low to clear a digit this close).
    fn hop_left(&mut self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        if self.before < 2 {
            return Err(fail("the point has no place to its left".into()));
        }
        let from = self.spots[self.before - 1];
        self.before -= 1;
        let to = self.spots[self.before - 1];
        let lift = stage.get_height(self.digits[self.before]) + HOP_CLEAR;
        let (point, radius) = (self.point, stage.get_width(self.point) / 2.0);
        let t = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_updater(
            anchor,
            move |s, _| {
                let u = s.tracker_value(t).unwrap_or(0.0);
                let at = v(
                    from[0] + (to[0] - from[0]) * u,
                    from[1] + (to[1] - from[1]) * u + lift * 4.0 * u * (1.0 - u),
                );
                let dot = Dot::new().point(at).radius(radius).build();
                let _ = s.set_points(point, dot.points());
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        stage.play(tween(t, 1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        Ok(())
    }
}

/// The question with no ring and the swap (06_6, 06_7): "8% of $25 = ?" on
/// a card in the question's look, every part typeset once on its own so
/// the two numbers can trade places on arcs between fixed places.
struct SwapLine {
    back: Mob,
    /// The percent and the dollars, each typeset on its own.
    a: Mob,
    b: Mob,
    middle: Mob,
    equals: Mob,
    ask: Mob,
    answer: Mob,
    /// Where `a` and `b` stand when they have traded places.
    a_away: Vec3,
    b_away: Vec3,
    a_home: Vec3,
    b_home: Vec3,
}

impl SwapLine {
    fn build(kit: &Kit, stage: &mut Stage<'_>, percent: u64, amount: u64, cents: u64) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let (a_src, b_src, answer_src) = (num(percent), num(amount), dollars(cents));
        let parts = line_parts(
            kit,
            stage,
            &[
                (&a_src, ink),
                (PERCENT_OF, ink),
                (&b_src, ink),
                ("=", ink),
                (&answer_src, ink),
            ],
            SWAP_PT,
            SWAP_AT,
        )?;
        let [a, middle, b, equals, answer] = parts[..] else {
            return Err(fail("the swap's line is not five parts".into()));
        };
        // the question mark stands where the answer will, on the same line
        let ask = eq(kit, stage, ASK, SWAP_PT, SWAP_AT)?;
        let (w, h) = (stage.get_width(ask), stage.get_height(ask));
        let at = v(stage.get_left(answer)[0] + w / 2.0, stage.get_bottom(a)[1] + h / 2.0);
        stage.move_to(ask, at, ORIGIN);
        // centre the question as it is first asked
        let dx = SWAP_AT[0] - (stage.get_left(a)[0] + stage.get_right(ask)[0]) / 2.0;
        for m in [a, middle, b, equals, answer, ask] {
            let c = stage.get_center(m);
            stage.move_to(m, v(c[0] + dx, c[1]), ORIGIN);
        }
        let (a_home, b_home) = (stage.get_center(a), stage.get_center(b));
        let (wa, wb) = (stage.get_width(a), stage.get_width(b));
        // traded: each keeps its side's edge against the middle
        let a_away = v(stage.get_left(b)[0] + wa / 2.0, a_home[1]);
        let b_away = v(stage.get_right(a)[0] - wb / 2.0, b_home[1]);
        let left = stage.get_left(a)[0].min(b_away[0] - wb / 2.0);
        let right = stage.get_right(ask)[0].max(stage.get_right(answer)[0]);
        let top = [a, middle, b, ask]
            .iter()
            .map(|&m| stage.get_top(m)[1])
            .fold(f64::MIN, f64::max);
        let bottom = [a, middle, b, ask]
            .iter()
            .map(|&m| stage.get_bottom(m)[1])
            .fold(f64::MAX, f64::min);
        let back = Rectangle::new()
            .width(right - left + 1.0)
            .height(top - bottom + 0.9)
            .corner_radius(0.14)
            .style(Style::default().fill(panel(), 0.96).stroke(quiz(), 2.5, 1.0))
            .build()?
            .moved_to(v((left + right) / 2.0, (top + bottom) / 2.0));
        let back = stage.arena_mut().add(back);
        Ok(Self {
            back,
            a,
            b,
            middle,
            equals,
            ask,
            answer,
            a_away,
            b_away,
            a_home,
            b_home,
        })
    }

    /// The two numbers trade places (`away`) or go back, on arcs.
    fn trade(&self, stage: &mut Stage<'_>, away: bool) -> fmn::Result<()> {
        let (to_a, to_b) = if away {
            (self.a_away, self.b_away)
        } else {
            (self.a_home, self.b_home)
        };
        play!(stage;
            glide(self.a, to_a, SWAP_ARC, SWAP_S)?,
            glide(self.b, to_b, SWAP_ARC, SWAP_S)?,
        );
        Ok(())
    }

    /// The line's parts as they stand once answered, left to right.
    fn answered(&self) -> [Mob; 5] {
        [self.a, self.middle, self.b, self.equals, self.answer]
    }
}

impl SceneConstruct for Percents {
    fn name(&self) -> &str {
        "06_percents"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let bill = shown::BILL;
        let b = blocks(bill).ok_or_else(|| fail(format!("a tenth of {bill} cents is not whole cents")))?;
        let pct = |part: u64| PER_CENT * part / b.cents;

        // 06_1: the bill as a bar cut into ten; one tenth lifts out; above
        // it the point of 48.00 moves one place to the left
        let line = say(&kit, stage, "06_1")?;
        let bar = Bar::build(stage, Board::new(BAR_AT, BAR_UNIT), b.cents / b.ten, BAR_TALL);
        let mut places = Places::build(&kit, stage, b.cents, PLACES_PT, v(0.0, PLACES_Y))?;
        line.wait_sentence(stage, 1)?;
        let mut ins = bar.mobs();
        ins.extend(places.mobs());
        fade_in_all(stage, &ins, 0.7)?;
        line.wait_for(stage, "one tenth")?;
        play!(stage; repaint(bar.tenths[0], lit_color(), 1.0, 0.4)?);
        line.wait_for(stage, "the decimal point")?;
        places.hop_left(stage, 0.8)?;
        let tenth_reads = format!("{}0", money(b.ten));
        if places.reads(b.cents) != tenth_reads {
            return Err(fail(format!(
                "the point moved reads {}, not {tenth_reads}",
                places.reads(b.cents)
            )));
        }
        let (w, h) = bar.tenth();
        let col = |i: usize| v(COLS_X[i], PIECE_Y);
        lift(stage, &[(bar.centre(0.0, 1.0), w, h, col(0))], 0.9)?;
        line.wait_sentence(stage, 3)?;
        // the last zero is only a place: it dims, and the digits read 4.80
        let last = *places.digits.last().ok_or_else(|| fail("no digits".into()))?;
        let column_labels = |kit: &Kit, stage: &mut Stage<'_>, i: usize, part: u64| -> fmn::Result<[Mob; 2]> {
            Ok([
                eq(kit, stage, &format!(r"{}\%", pct(part)), PCT_PT, v(COLS_X[i], PCT_Y))?,
                eq(kit, stage, &dollars(part), MONEY_PT, v(COLS_X[i], MONEY_Y))?,
            ])
        };
        let ten_labels = column_labels(&kit, stage, 0, b.ten)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        anims.push(stage.prepare(dim(last, 0.0, 0.5)?)?);
        for m in ten_labels {
            let a = fade_in(stage.arena_mut(), m, v(0.0, 0.15), 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        stage.play_prepared(anims)?;
        // the bar is still the whole bill: its own amount comes back over it
        // once the tenth has its label (R2, M2)
        let bill_label = eq(&kit, stage, &dollars(b.cents), PLACES_PT, v(0.0, PLACES_Y))?;
        swap(stage, &places.mobs(), &[bill_label], 0.5)?;

        // 06_2: half of the tenth, and the tenth and the half side by side
        let line = say(&kit, stage, "06_2")?;
        let half_cut = cut_line(stage, v(COLS_X[0], PIECE_Y - h / 2.0), v(COLS_X[0], PIECE_Y + h / 2.0));
        play!(stage; fade_in(stage.arena_mut(), half_cut, ORIGIN, 1.0)?.rt(0.3));
        lift(stage, &[(v(COLS_X[0] - w / 4.0, PIECE_Y), w / 2.0, h, col(1))], 0.8)?;
        line.wait_sentence(stage, 1)?;
        let five_labels = column_labels(&kit, stage, 1, b.five)?;
        fade_in_all(stage, &five_labels, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let left = COLS_X[2] - 0.75 * w;
        lift(
            stage,
            &[
                (col(0), w, h, v(left + w / 2.0, PIECE_Y)),
                (col(1), w / 2.0, h, v(left + 1.25 * w, PIECE_Y)),
            ],
            0.8,
        )?;
        let seam = cut_line(stage, v(left + w, PIECE_Y - h / 2.0), v(left + w, PIECE_Y + h / 2.0));
        let fifteen_labels = column_labels(&kit, stage, 2, b.fifteen)?;
        let mut ins = vec![seam];
        ins.extend(fifteen_labels);
        fade_in_all(stage, &ins, 0.4)?;

        // 06_3: twenty percent is the tenth doubled; seventeen is four pieces,
        // said aloud as where the trick stops paying (R2, M15)
        let line = say(&kit, stage, "06_3")?;
        lift(
            stage,
            &[
                (col(0), w, h, v(COLS_X[3] - w / 2.0, PIECE_Y)),
                (col(0), w, h, v(COLS_X[3] + w / 2.0, PIECE_Y)),
            ],
            0.8,
        )?;
        let seam = cut_line(stage, v(COLS_X[3], PIECE_Y - h / 2.0), v(COLS_X[3], PIECE_Y + h / 2.0));
        let twenty_labels = column_labels(&kit, stage, 3, b.twenty)?;
        let mut ins = vec![seam];
        ins.extend(twenty_labels);
        fade_in_all(stage, &ins, 0.4)?;
        // seventeen percent: the tenth, its half, and two hundredths
        let one = b.cents / PER_CENT;
        let pieces = [b.ten, b.five, one, one];
        if pieces.len() != FOUR {
            return Err(fail(format!("the aside says four pieces, not {}", pieces.len())));
        }
        let terms: Vec<String> = pieces.iter().map(|&p| format!(r"{}\%", pct(p))).collect();
        let aside_src = format!(
            r"{}\% = {}:\ {FOUR_PIECES}",
            pct(pieces.iter().sum()),
            terms.join(" + ")
        );
        let aside = grey_line(&kit, stage, &aside_src, v(0.0, GREY_Y))?;
        fit_width(stage, aside, GREY_ROOM);
        line.wait_sentence(stage, 1)?;
        play!(stage; fade_in(stage.arena_mut(), aside, ORIGIN, 1.0)?.rt(0.5));
        // a moment to read the aside before the quiz
        kit.hold(stage)?;
        pause(stage, 0.8)?;

        // 06_4: your turn: fifteen percent of sixty dollars
        let line = say(&kit, stage, "06_4")?;
        fade_all(stage, 0.5)?;
        let look = QuizLook::at(CARD_AT, CARD_SCALE);
        let card = QuizCard::build(&kit, stage, "06_4", look)?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 06_5: a tenth, half of it, together: on a bar of ten under the card,
        // and into the answer already there
        let line = say(&kit, stage, "06_5")?;
        let Ask::PercentOfMoney(percent, cents) = card.quiz.ask else {
            return Err(fail("06_4 does not ask a percent of money".into()));
        };
        let q = blocks(cents).ok_or_else(|| fail(format!("a tenth of {cents} cents is not whole cents")))?;
        if PER_CENT * q.fifteen / q.cents != percent {
            return Err(fail(format!("06_4 asks {percent} percent, not a tenth and a half")));
        }
        let small = Bar::build(stage, Board::new(SMALL_BAR_AT, SMALL_UNIT), q.cents / q.ten, SMALL_TALL);
        fade_in_all(stage, &small.mobs(), 0.4)?;
        let (ten_src, five_src) = (dollars(q.ten), dollars(q.five));
        let sum_parts = [(ten_src.as_str(), ink), ("+", ink), (five_src.as_str(), ink)];
        let sum = line_parts(&kit, stage, &sum_parts, SUM_PT, v(0.0, SUM_Y))?;
        play!(stage;
            repaint(small.tenths[0], lit_color(), 1.0, 0.4)?,
            fade_in(stage.arena_mut(), sum[0], v(0.0, 0.12), 1.0)?.rt(0.4),
        );
        line.wait_sentence(stage, 1)?;
        let (sw, sh) = small.tenth();
        let small_half = block(stage, small.centre(1.0, 0.5), sw / 2.0, sh, lit());
        let half_seam = small.board.line(stage, [1.5, 0.0], [1.5, SMALL_TALL], cut_style());
        keep_over(stage, half_seam);
        fade_in_all(stage, &[small_half, half_seam, sum[1], sum[2]], 0.4)?;
        line.wait_sentence(stage, 2)?;
        // the sum goes into the answer on the card, which answers back
        let copies = line_parts(&kit, stage, &sum_parts, SUM_PT, v(0.0, SUM_Y))?;
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
        play!(stage; fmn::animation::indicate(stage.arena_mut(), card.answer, 1.12, Some(rgb(ink)))?.rt(0.6));

        // 06_6: eight percent of twenty-five dollars, a card
        // with no ring
        let line = say(&kit, stage, "06_6")?;
        fade_all(stage, 0.5)?;
        let (percent, amount) = shown::SWAP;
        let one_way = percent_of(percent, amount * PER_CENT)
            .ok_or_else(|| fail(format!("{percent}% of {amount} dollars is not whole cents")))?;
        let other_way = percent_of(amount, percent * PER_CENT)
            .ok_or_else(|| fail(format!("{amount}% of {percent} dollars is not whole cents")))?;
        if one_way != other_way {
            return Err(fail(format!("the swap gives {one_way} and {other_way} cents")));
        }
        let swap_line = SwapLine::build(&kit, stage, percent, amount, one_way)?;
        line.wait_last(stage)?;
        fade_in_all(
            stage,
            &[
                swap_line.back,
                swap_line.a,
                swap_line.middle,
                swap_line.b,
                swap_line.equals,
                swap_line.ask,
            ],
            0.5,
        )?;
        why_pause(&kit, stage)?;

        // 06_7: swap the two numbers: a quarter of eight dollars; the same
        // answer both ways
        let line = say(&kit, stage, "06_7")?;
        swap_line.trade(stage, true)?;
        line.wait_for(stage, "a quarter of eight")?;
        let (_, quarter) = FRIENDLY[1];
        if amount * quarter != PER_CENT {
            return Err(fail(format!("{amount} percent is not a quarter")));
        }
        let note = kit.text(
            stage,
            &format!("{A_QUARTER_OF} ${percent}"),
            44.0,
            Palette::dim(),
            v(0.0, QUARTER_Y),
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.12), 1.0)?.rt(0.4));
        line.wait_sentence(stage, 2)?;
        swap(stage, &[swap_line.ask], &[swap_line.answer], 0.5)?;
        line.wait_sentence(stage, 3)?;
        fade_out_all(stage, &[note], 0.4)?;
        swap_line.trade(stage, false)?;

        // 06_8: cents on the dollar: a grid 25 across and 8 deep, read by its
        // columns and by its rows; the same cents fill two squares of a
        // hundred
        let line = say(&kit, stage, "06_8")?;
        let (across, deep) = (amount, percent);
        let cells = across * deep;
        if cells != one_way {
            return Err(fail(format!("{cells} cells, but the swap is {one_way} cents")));
        }
        // the line goes up to the top, smaller, and the card's back goes
        let parts = swap_line.answered();
        let centre = middle_of(stage, &parts)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        stage.clear_updaters(swap_line.back, true);
        let a = fade_out(stage.arena_mut(), swap_line.back, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
        for &m in &parts {
            let c = stage.get_center(m);
            let to = v(
                TOP_LINE[0] + (c[0] - centre[0]) * TOP_SCALE,
                TOP_LINE[1] + (c[1] - centre[1]) * TOP_SCALE,
            );
            anims.push(stage.prepare(fly(m, to, TOP_SCALE, 0.6)?)?);
        }
        stage.play_prepared(anims)?;
        let board = Board::new(GRID_AT, CELL);
        let (wide, tall) = (across as f64, deep as f64);
        let whole = [0.0, 0.0, wide, tall];
        let columns: Vec<Mob> = (0..across)
            .map(|i| board.rect(stage, [i as f64, 0.0, 1.0, tall], flat(slate(), 1.0)))
            .collect();
        // the rows, from the top down, lit over the columns when they are read
        let rows: Vec<Mob> = (0..deep)
            .map(|j| board.rect(stage, [0.0, tall - 1.0 - j as f64, wide, 1.0], lit()))
            .collect();
        let thin = line_style(ground(), 1.5);
        let lines = board.grid(stage, whole, 1.0, thin)?;
        keep_over(stage, lines);
        let left_label = board.label(&kit, stage, &num(deep), whole, Side::Left, SIDE_PT, ink)?;
        let below_label = board.label(&kit, stage, &num(across), whole, Side::Below, SIDE_PT, ink)?;
        let caption_at = v(board.centre(whole)[0], CAPTION_Y);
        let caption = kit.text(stage, ONE_CELL, CAPTION_PT, Palette::dim(), caption_at)?;
        let mut ins = columns.clone();
        ins.extend([lines, left_label, below_label, caption]);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_sentence(stage, 1)?;
        // eight cents from each of the twenty-five dollars: the columns
        let by_columns = kit.text(
            stage,
            &format!("{deep}{CENT} {FROM_EACH_DOLLAR}"),
            CAPTION_PT,
            ink,
            caption_at,
        )?;
        swap(stage, &[caption], &[by_columns], 0.3)?;
        let anims = columns
            .iter()
            .map(|&c| stage.prepare(repaint(c, lit_color(), 1.0, 0.3)?))
            .collect::<fmn::Result<Vec<_>>>()?;
        play_lagged(stage, anims, 2.2, 0.15)?;
        line.wait_sentence(stage, 2)?;
        // twenty-five cents from each of the eight: the rows
        let by_rows = kit.text(
            stage,
            &format!("{across}{CENT} {FROM_EACH}"),
            CAPTION_PT,
            ink,
            caption_at,
        )?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &c in &columns {
            anims.push(stage.prepare(repaint(c, slate(), 1.0, 0.4)?)?);
        }
        stage.clear_updaters(by_columns, true);
        let a = fade_out(stage.arena_mut(), by_columns, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.3))?);
        let a = fade_in(stage.arena_mut(), by_rows, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.3))?);
        stage.play_prepared(anims)?;
        let anims = rows
            .iter()
            .map(|&r| {
                let a = fade_in(stage.arena_mut(), r, ORIGIN, 1.0)?;
                stage.prepare(a.rt(0.3))
            })
            .collect::<fmn::Result<Vec<_>>>()?;
        play_lagged(stage, anims, 1.6, 0.35)?;
        line.wait_sentence(stage, 3)?;
        // the same two hundred cents: the cells flow into two squares of a
        // hundred. Ten columns make the lower part of each square; the five
        // columns left over, cut in two, make each square's top.
        let side = (PER_CENT as f64).sqrt();
        let left_over = wide - 2.0 * side;
        if cells != 2 * PER_CENT
            || left_over <= 0.0
            || (left_over * tall - 2.0 * side * (side - tall)).abs() > 1e-9
            || deep % 2 != 0
        {
            return Err(fail(format!(
                "a grid {across} by {deep} does not fill two squares of a hundred this way"
            )));
        }
        let half = tall / 2.0;
        let to_cells = |x: f64, y: f64| [(x - GRID_AT[0]) / CELL, (y - GRID_AT[1]) / CELL];
        let [x1, y0] = to_cells(SQUARES_AT[0], SQUARES_AT[1]);
        let x2 = x1 + side + SQUARE_GAP / CELL;
        let flows = [
            ([0.0, 0.0, side, tall], [x1, y0, side, tall]),
            ([side, 0.0, side, tall], [x2, y0, side, tall]),
            ([2.0 * side, 0.0, left_over, half], [x1, y0 + tall, side, side - tall]),
            ([2.0 * side, half, left_over, half], [x2, y0 + tall, side, side - tall]),
        ];
        let blue = flat(hundreds(), 1.0);
        let pieces: Vec<Mob> = flows.iter().map(|&(from, _)| board.rect(stage, from, blue)).collect();
        let mut out = columns.clone();
        out.extend(&rows);
        out.extend([lines, left_label, below_label, by_rows]);
        swap(stage, &out, &pieces, 0.5)?;
        // the ten columns first, then the five left over close each square
        for pair in pieces.chunks(2).zip(flows.chunks(2)) {
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for (&m, &(_, to)) in pair.0.iter().zip(pair.1) {
                let target = board.rect(stage, to, blue);
                anims.push(stage.prepare(Transform::new(m, target).rt(0.8))?);
            }
            stage.play_prepared(anims)?;
        }
        let squares = [[x1, y0, side, side], [x2, y0, side, side]];
        let mut ins = Vec::new();
        for r in squares {
            let g = board.grid(stage, r, 1.0, thin)?;
            ins.push(keep_over(stage, g));
        }
        let cents_caption = kit.text(
            stage,
            &format!("{cells}{CENT} = ${}", money(cells)),
            CAPTION_PT,
            ink,
            v((SQUARES_AT[0] + board.p(x2 + side, 0.0)[0]) / 2.0, CAPTION_Y),
        )?;
        ins.push(cents_caption);
        fade_in_all(stage, &ins, 0.4)?;
        line.wait_for(stage, "like a tenth")?;
        // the friendly percents: a tenth, a quarter, a half
        let mut friendly = Vec::new();
        for (&(name, parts), &y) in FRIENDLY.iter().zip(&FRIENDLY_Y) {
            if !PER_CENT.is_multiple_of(parts) {
                return Err(fail(format!("{name} is not a whole percent")));
            }
            let p = eq(
                &kit,
                stage,
                &format!(r"{}\%", PER_CENT / parts),
                FRIENDLY_PT.0,
                v(FRIENDLY_X.0, y),
            )?;
            let words = kit.text_left(stage, name, FRIENDLY_PT.1, Palette::dim(), v(FRIENDLY_X.1, y))?;
            friendly.push(group(stage, &[p, words])?);
        }
        fade_in_lagged(stage, &friendly, 1.5, 0.6)?;
        line.wait_end(stage)?;
        let precisely = lower_third_tex(&kit, stage, PRECISELY, PRECISELY_TEX)?;
        play!(stage; fade_in(stage.arena_mut(), precisely, v(0.0, 0.15), 1.0)?.rt(0.5));
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
