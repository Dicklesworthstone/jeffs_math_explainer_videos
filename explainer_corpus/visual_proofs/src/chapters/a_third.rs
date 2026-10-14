//! Chapter 11, a third: the owner's second picture. A unit square is cut in
//! four, one quarter coloured, two left alone, and the last cut in four
//! again (11_1), and so on without end, the first three coloured pieces
//! carrying their fractions (11_2); the quiz, what fraction of the square is
//! coloured (11_3, a third); the three squares of every size slide onto one
//! another, so the other two are given their own colours (11_4); each colour
//! with its share of the last corner is exactly one third, and the corner
//! goes below any size (11_5); and the card, with Archimedes and Ajose and
//! Nelsen (11_6).
//!
//! Every number is a fact or the model's exact fraction: the pieces
//! `F-R.pieces`, the running shares `F-R.share`, the corner `F-R.left`, a
//! strip `F-R.short`, the limit `F-R.limit` and the quiz's answer
//! `F-Q.share.answer`; the pieces drawn are `model::series::quarters`
//! exactly (floats are used to place them only).

use explainer_kit::{Kit, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, CardRow, LENS_AT, ProofCard, QuizCard, QuizLook, Words, breath, copy, dim, fact, fade_in_all,
    fade_in_lagged, fade_out_all, faded_line, fail, file_on_shelf, flash, flat, fly, formula_third, frac_tex, ground,
    hero, ink, lens, lens_mark, lower_third, pause, say, slide, tex_row,
};
use crate::model::{Q, series};

pub struct AThird {
    pub kit: Kit,
}

// ------------------------------------------------------------------ choices

/// The lower left corner of the unit square on screen, and its side. The
/// square keeps below the shelf's row (`SHELF_FLOOR`) and above the card's
/// top, so it never moves. The card rises (its statement is two lines of
/// fractions), and since its foot stands `CARD_FOOT_CLEAR` above the frame's
/// edge its top stands above `CARD_TOP`: the square's foot stands
/// [`CARD_GAP`] over that top (it was 4.3 wide and ended 0.06 over a card
/// whose foot stood 9 px from the edge, under a player's control bar).
const SQUARE_AT: Vec3 = [-6.0, -0.95, 0.0];
const SIDE: f64 = 3.9;
/// The least room between the square's foot and the card's top.
#[cfg(test)]
const CARD_GAP: f64 = 0.1;
/// How many levels of the cascade are drawn.
const LEVELS: u32 = 8;
/// The type size of the label of each of the first three coloured pieces.
const LABELLED: [f64; 3] = [46.0, 32.0, 21.0];
/// The slot this proof takes on the shelf (03, 04, 06, 09, 10, then this).
pub const SLOT: usize = 5;
/// The corner after this many sizes is the one the readouts are about.
const READ_AFTER: u32 = 3;
/// The centre of the right-hand panel and its width.
const PANEL_X: f64 = 2.9;
const PANEL_W: f64 = 7.0;
/// How far the corner is magnified at the end of 11_5: the corner after
/// `READ_AFTER` sizes is `1 / 2^READ_AFTER` of the square's side.
const ZOOM: f64 = 8.0;
/// Where the source line sits, clear of the lower-third's band.
const SOURCE_Y: f64 = -2.45;
/// How far under the question's lowest line the answer's top stands at
/// least (the stacked third is taller than the ring it lands over: its
/// numerator stood on the sum's "64", the dense read of the narrated draft),
/// and how far over the card's foot its bottom stands at least.
const ANSWER_CLEAR: f64 = 0.15;
const ANSWER_FOOT: f64 = 0.2;

// the picture keeps below the shelf's row and above the card's top, and the
// quiz card and the right-hand panel stand clear of it
const _: () = assert!(SQUARE_AT[1] + SIDE <= super::draw::SHELF_FLOOR + 1e-9);
const _: () = assert!(SQUARE_AT[1] > super::draw::CARD_TOP);
const _: () = assert!(PANEL_X - PANEL_W / 2.0 > SQUARE_AT[0] + SIDE);

// the chapter's fixed words
const AFTER_THREE: &str = "after three sizes";
const THE_CORNER: &str = "the corner";
const A_QUARTER: &str = "a quarter as big";
const PRECISELY: &str = "precisely:";
const ARCHIMEDES: &str = "Archimedes, Quadrature of the Parabola, Prop. 23:";
/// Archimedes's hypothesis as well as his result (R2, 11_6): each piece a
/// quarter of the one before, then the finite sum and the third of its last
/// piece. The voice says only "quarters of quarters, stopping at a last piece".
const ARCHIMEDES_SAID: &str = "if each piece is a quarter of the one before, a finite sum plus a third of its last piece is exactly 4/3 of its first";
const AJOSE: &str = "S. A. Ajose and R. B. Nelsen, Mathematics Magazine";
const EACH_SLIDES: &str = "each slides onto the other";
const THIRD_OF_CORNER: &str = "each a third of the corner";
const THREE_EQUAL: &str = "the three squares of each size are equal";
const THREE_STRIPS: &str = "the corner is cut into three equal strips";

// ---------------------------------------------------------------- the model

/// A rectangle (x, y, width, height) of the unit square, as floats.
pub type Frame = [f64; 4];

/// A piece of the model as floats, for placing it.
pub fn frame(p: series::Piece) -> Frame {
    [p.0.to_f64(), p.1.to_f64(), p.2.to_f64(), p.3.to_f64()]
}

/// The corner after `j` sizes (the square still uncut) as floats.
pub fn corner(j: u32) -> Frame {
    frame(series::left_after_quarters(j))
}

/// A fact that is a list of fractions (`1/4,1/16,1/64`).
fn list(key: &str) -> fmn::Result<Vec<String>> {
    Ok(fact(key)?.split(',').map(str::to_owned).collect())
}

/// The middle of a rectangle of the unit square on `board`.
fn centre_of(board: &Board, f: Frame) -> Vec3 {
    board.p(f[0] + f[2] / 2.0, f[1] + f[3] / 2.0)
}

/// A rectangle of the unit square on `board` as one flat polygon, inset by
/// a hair so neighbours stay apart. Detached.
fn rect(stage: &mut Stage<'_>, board: &Board, r: Frame, style: Style) -> Mob {
    let g = (board.cell * r[2].min(r[3]) * 0.06).min(0.03);
    let (a, b) = (board.p(r[0], r[1]), board.p(r[0] + r[2], r[1] + r[3]));
    let quad = [
        v(a[0] + g, a[1] + g),
        v(b[0] - g, a[1] + g),
        v(b[0] - g, b[1] - g),
        v(a[0] + g, b[1] - g),
    ];
    stage.arena_mut().add(Polygon::new(quad).style(style).build())
}

/// A `w` by `h` rectangle centred at `at`, detached.
fn block(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, style: Style) -> fmn::Result<Mob> {
    let r = Rectangle::new().width(w).height(h).style(style).build()?.moved_to(at);
    Ok(stage.arena_mut().add(r))
}

/// One level of the cascade, every part detached.
struct Level {
    j: u32,
    /// The three squares: coloured, the one above it, the one beside it.
    frames: [Frame; 3],
    /// Outlines of the three, in `dim`.
    outlines: [Mob; 3],
    /// The coloured square, filled blue.
    fill: Mob,
    /// The upper and the right-hand square, filled copper and pink (hidden
    /// until 11_4).
    fills: [Mob; 2],
    /// Outlines in copper and pink that slide onto the coloured square.
    ghosts: [Mob; 2],
}

impl Level {
    fn build(stage: &mut Stage<'_>, board: &Board, j: u32) -> Self {
        let frames = series::quarters(j).map(frame);
        let thin = (2.5 / f64::from(1u32 << (j - 1))).max(0.6);
        let heavy = (3.0 / f64::from(1u32 << (j - 1))).max(0.9);
        let outlines = frames.map(|f| rect(stage, board, f, faded_line(dim(), thin, 1.0)));
        let fill = rect(stage, board, frames[0], flat(copy(0), 1.0));
        let fills = [
            rect(stage, board, frames[1], flat(copy(1), 1.0)),
            rect(stage, board, frames[2], flat(copy(2), 1.0)),
        ];
        let ghosts = [
            rect(stage, board, frames[1], faded_line(copy(1), heavy, 1.0)),
            rect(stage, board, frames[2], faded_line(copy(2), heavy, 1.0)),
        ];
        Self {
            j,
            frames,
            outlines,
            fill,
            fills,
            ghosts,
        }
    }

    /// The side of the three squares on screen.
    fn side(&self, board: &Board) -> f64 {
        self.frames[0][2] * board.cell
    }

    /// Everything of the level that is on the stage by the end of 11_4.
    fn staged(&self) -> Vec<Mob> {
        let mut out = self.outlines.to_vec();
        out.push(self.fill);
        out.extend(self.fills);
        out
    }
}

/// "S. A. Ajose and R. B. Nelsen, Mathematics Magazine, 1994": the year is
/// the first number of `F-X.ajose_nelsen` (`1994,230`: year and page).
fn ajose_line() -> fmn::Result<String> {
    let value = fact("F-X.ajose_nelsen")?;
    let year = value
        .split(',')
        .next()
        .ok_or_else(|| fail("F-X.ajose_nelsen is empty".into()))?;
    Ok(format!("{AJOSE}, {year}"))
}

/// The answer takes the room under the question inside the card, into the
/// space of the ring and its note, which have gone when it lands: if it is
/// taller than that room (the stacked third is, at the card's answer size)
/// it is set smaller to fit it, and it stands in the room's middle only as
/// far as it must move off the question. The card's own code is not touched.
/// Returns the answer's height after.
fn fit_answer(stage: &mut Stage<'_>, card: &QuizCard) -> f64 {
    let (_, card_h) = card.look.size();
    let top = stage.get_bottom(card.question)[1] - ANSWER_CLEAR;
    let bottom = card.look.at[1] - card_h / 2.0 + ANSWER_FOOT;
    let room = top - bottom;
    if stage.get_height(card.answer) > room {
        let at = stage.get_center(card.answer);
        stage.set_height(card.answer, room, false);
        stage.move_to(card.answer, at, ORIGIN);
    }
    let c = stage.get_center(card.answer);
    let h = stage.get_height(card.answer);
    // off the question first, then off the foot
    let mut y = c[1].min(top - h / 2.0);
    y = y.max(bottom + h / 2.0);
    stage.move_to(card.answer, v(c[0], y), ORIGIN);
    h
}

/// 11_6's card: the statement with its sum and its limit, and the two
/// things the picture took for granted. Detached.
fn proof_card(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<ProofCard> {
    let mut sum: Vec<String> = list("F-R.pieces")?.iter().map(|p| frac_tex(p)).collect();
    sum.push("\\cdots".to_owned());
    let shown = format!(
        "\\text{{for every }} n \\text{{ from 1, after }} n \\text{{ cuts each colour holds }} \
         \\frac13-\\frac{{1}}{{3\\cdot 4^{{n}}}}\\text{{, and the corner goes below any size, so }} {} = {}",
        sum.join("+"),
        frac_tex(&fact("F-R.limit")?)
    );
    ProofCard::build(
        kit,
        stage,
        &[
            CardRow::Shown(Words::Tex(&shown)),
            CardRow::Assumed(Words::Text(THREE_EQUAL)),
            CardRow::Assumed(Words::Text(THREE_STRIPS)),
        ],
    )
}

/// The picture this card files, rebuilt as it stands on the shelf (the
/// chapters after this one open with the shelf so far): the corner after
/// three sizes magnified eight times, the cascade of its pieces and the
/// last square, at thumbnail size in its slot. Detached.
pub fn thumb(_kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let board = Board::new(SQUARE_AT, SIDE);
    let levels: Vec<Level> = (1..=LEVELS).map(|j| Level::build(stage, &board, j)).collect();
    let corner_f = corner(READ_AFTER);
    let mut zoom: Vec<Mob> = levels[READ_AFTER as usize..].iter().flat_map(Level::staged).collect();
    zoom.push(rect(stage, &board, corner(LEVELS), faded_line(dim(), 0.6, 1.0)));
    zoom.push(rect(stage, &board, corner_f, faded_line(hero(), 4.0, 1.0)));
    // magnified `ZOOM` times about the corner and then filed at `SIDE`: one
    // move, the corner's middle to the slot and the sizes with it
    super::draw::shelve(stage, &zoom, centre_of(&board, corner_f), SIDE / ZOOM, SLOT);
    Ok(zoom)
}

impl SceneConstruct for AThird {
    fn name(&self) -> &str {
        "11_a_third"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        super::draw::shelf_so_far(&kit, stage, 11)?;
        let board = Board::new(SQUARE_AT, SIDE);
        let square_mid = board.p(0.5, 0.5);
        let pieces = list("F-R.pieces")?;
        let shares = list("F-R.share")?;
        let lefts = list("F-R.left")?;
        let short = fact("F-R.short")?;
        let limit = fact("F-R.limit")?;
        let answer = fact("F-Q.share.answer")?;
        if pieces.len() != LABELLED.len() || shares.len() != LABELLED.len() || lefts.len() != LABELLED.len() {
            return Err(fail("F-R lists are not as long as the sizes the chapter reads".into()));
        }

        // ---------------------------------------------------------- the parts
        let levels: Vec<Level> = (1..=LEVELS).map(|j| Level::build(stage, &board, j)).collect();
        let frame_mob = rect(stage, &board, [0.0, 0.0, 1.0, 1.0], faded_line(ink(), 3.0, 1.0));
        // the fourth quarter, cut again; and the last square of the cascade,
        // still uncut
        let quarter_four = rect(stage, &board, corner(1), faded_line(dim(), 2.5, 1.0));
        let last = rect(stage, &board, corner(LEVELS), faded_line(dim(), 0.6, 1.0));
        // the fractions of the first three coloured pieces
        let mut labels = Vec::new();
        for (i, size) in LABELLED.iter().enumerate() {
            let at = centre_of(&board, levels[i].frames[0]);
            labels.push(kit.tex(stage, &frac_tex(&pieces[i]), *size, ground(), &[], at)?);
        }

        // ------------------------------------------------------------ 11_1
        // four quarters; one coloured, two left alone, the last cut again
        let line = say(&kit, stage, "11_1")?;
        let first = &levels[0];
        let second = &levels[1];
        fade_in_all(stage, &[frame_mob], 0.6)?;
        line.wait_for(stage, "Cut the square into four quarters")?;
        let mut quarters = first.outlines.to_vec();
        quarters.push(quarter_four);
        fade_in_all(stage, &quarters, 0.7)?;
        line.wait_for(stage, "Colour one")?;
        fade_in_all(stage, &[first.fill], 0.6)?;
        line.wait_for(stage, "Leave two alone")?;
        flash(stage, &first.outlines[1..], ink(), 1.04, 0.6)?;
        line.wait_for(stage, "And cut the last one into quarters again")?;
        fade_in_all(stage, &second.outlines, 0.7)?;

        // ------------------------------------------------------------ 11_2
        // the same inside, and again, without end; three carry fractions
        let line = say(&kit, stage, "11_2")?;
        fade_in_all(stage, &[labels[0]], 0.4)?;
        line.wait_for(stage, "Colour one, leave two")?;
        fade_in_all(stage, &[second.fill, labels[1]], 0.6)?;
        line.wait_for(stage, "And again, and again")?;
        let mut march: Vec<Mob> = Vec::new();
        for lvl in &levels[2..] {
            march.extend(lvl.outlines);
            march.push(lvl.fill);
            if let Some(&label) = labels.get(lvl.j as usize - 1) {
                march.push(label);
            }
        }
        march.push(last);
        fade_in_lagged(stage, &march, 2.6, 0.12)?;

        // ------------------------------------------------------------ 11_3
        // your turn: what fraction of the whole square is coloured?
        let line = say(&kit, stage, "11_3")?;
        let mut sum: Vec<String> = pieces.iter().map(|p| frac_tex(p)).collect();
        sum.push("\\cdots".to_owned());
        let question = format!("{} = \\,?", sum.join(" + "));
        let answer_tex = frac_tex(&answer);
        let card = QuizCard::build(
            &kit,
            stage,
            "share",
            Words::Tex(&question),
            Words::Tex(&answer_tex),
            QuizLook::BESIDE,
        )?;
        fit_answer(stage, &card);
        line.wait_sentence(stage, 1)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // ------------------------------------------------------------ 11_4
        // one third: three equal squares at every size; each slides onto
        // the next and fits, so the other two get their own colours
        let line = say(&kit, stage, "11_4")?;
        line.wait_for(stage, "At every size")?;
        fade_out_all(stage, &card.mobs(), 0.4)?;
        // sizes 1 to 3, one at a time
        for lvl in &levels[..LABELLED.len()] {
            let s = lvl.side(&board);
            fade_in_all(stage, &lvl.ghosts, 0.3)?;
            let down = stage.prepare(slide(lvl.ghosts[0], v(0.0, -s), 0.9)?)?;
            let left = stage.prepare(slide(lvl.ghosts[1], v(-s, 0.0), 0.9)?)?;
            stage.play_prepared(vec![down, left])?;
            pause(stage, 0.25)?;
            let up = stage.prepare(slide(lvl.ghosts[0], v(0.0, s), 0.5)?)?;
            let right = stage.prepare(slide(lvl.ghosts[1], v(s, 0.0), 0.5)?)?;
            stage.play_prepared(vec![up, right])?;
            fade_out_all(stage, &lvl.ghosts, 0.2)?;
        }
        // sizes 4 to 8, in one lagged pass
        let rest = &levels[LABELLED.len()..];
        let ghosts: Vec<Mob> = rest.iter().flat_map(|l| l.ghosts).collect();
        fade_in_lagged(stage, &ghosts, 0.8, 0.1)?;
        let mut to: Vec<Box<dyn Animation>> = Vec::new();
        for lvl in rest {
            let s = lvl.side(&board);
            to.push(stage.prepare(slide(lvl.ghosts[0], v(0.0, -s), 0.7)?)?);
            to.push(stage.prepare(slide(lvl.ghosts[1], v(-s, 0.0), 0.7)?)?);
        }
        let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), to, 0.08)?.with_run_time(1.4);
        stage.play(group)?;
        line.wait_for(stage, "So give the other two")?;
        fade_out_all(stage, &ghosts, 0.3)?;
        let fills: Vec<Mob> = levels.iter().flat_map(|l| l.fills).collect();
        fade_in_lagged(stage, &fills, 1.4, 0.1)?;

        // ------------------------------------------------------------ 11_5
        // each colour with its share of the corner is a third
        let line = say(&kit, stage, "11_5")?;
        let at_three = READ_AFTER as usize - 1;
        let corner_f = corner(READ_AFTER);
        let corner_outline = rect(stage, &board, corner_f, faded_line(hero(), 4.0, 1.0));
        let after = kit.text(stage, AFTER_THREE, 26.0, dim(), v(PANEL_X, 2.5))?;
        let colours = [copy(0), copy(1), copy(2)];
        let each = &shares[at_three];
        let corner_value = &lefts[at_three];
        // 21/64 + 21/64 + 21/64 + 1/64 = 1, each term in its colour
        let mut parts: Vec<(String, Srgb)> = Vec::new();
        for &c in &colours {
            parts.push((frac_tex(each), c));
            parts.push(("+".to_owned(), ink()));
        }
        parts.push((frac_tex(corner_value), hero()));
        parts.push(("= 1".to_owned(), ink()));
        let parts_ref: Vec<(&str, Srgb)> = parts.iter().map(|(s, c)| (s.as_str(), *c)).collect();
        let sum_row = tex_row(&kit, stage, &parts_ref, 44.0, v(PANEL_X, 1.45), 0.18, PANEL_W)?;
        // "the corner" under its term (the gold one, last but one)
        let corner_name = kit.text(stage, THE_CORNER, 24.0, hero(), v(PANEL_X, 0.75))?;
        let under = stage.get_center(sum_row[sum_row.len() - 2])[0];
        stage.move_to(corner_name, v(under, 0.8), ORIGIN);
        line.wait_for(stage, "Three colours")?;
        flash(
            stage,
            &[levels[1].fill, levels[1].fills[0], levels[1].fills[1]],
            ink(),
            1.05,
            0.7,
        )?;
        line.wait_for(stage, "three ways")?;
        let mut readout = vec![corner_outline, after, corner_name];
        readout.extend(&sum_row);
        fade_in_all(stage, &readout, 0.8)?;
        // the corner is cut into three strips, one for each colour
        let strip_w = corner_f[2] / 3.0;
        let mut strips = Vec::new();
        for (k, &c) in colours.iter().enumerate() {
            let r = [corner_f[0] + strip_w * k as f64, corner_f[1], strip_w, corner_f[3]];
            strips.push(rect(stage, &board, r, flat(c, 0.62)));
        }
        line.wait_for(stage, "and each colour, with its share")?;
        fade_in_all(stage, &strips, 0.6)?;
        let strip_row = tex_row(
            &kit,
            stage,
            &[
                (frac_tex(each).as_str(), ink()),
                ("+", ink()),
                (frac_tex(&short).as_str(), hero()),
                ("=", ink()),
                (frac_tex(&limit).as_str(), ink()),
            ],
            44.0,
            v(PANEL_X, -0.35),
            0.2,
            PANEL_W,
        )?;
        fade_in_all(stage, &strip_row, 0.6)?;
        // the corner grows to fill the frame: the same picture again, and
        // the strip a quarter as big
        line.wait_for(stage, "And the corner shrinks")?;
        let mut gone: Vec<Mob> = levels[..READ_AFTER as usize].iter().flat_map(Level::staged).collect();
        gone.extend([frame_mob, quarter_four, after, corner_name]);
        gone.extend(&labels);
        gone.extend(&strips);
        gone.extend(&sum_row);
        gone.extend(&strip_row);
        fade_out_all(stage, &gone, 0.6)?;
        let from = centre_of(&board, corner_f);
        let mut zoom: Vec<Mob> = levels[READ_AFTER as usize..].iter().flat_map(Level::staged).collect();
        zoom.extend([last, corner_outline]);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &zoom {
            let c = stage.get_center(m);
            let target = v(
                square_mid[0] + ZOOM * (c[0] - from[0]),
                square_mid[1] + ZOOM * (c[1] - from[1]),
            );
            anims.push(stage.prepare(fly(m, target, ZOOM, 1.8)?)?);
        }
        stage.play_prepared(anims)?;
        // a strip of the corner one size further down is a quarter as big
        let next = series::area(series::left_after_quarters(READ_AFTER + 1)) / Q::int(3);
        let step = kit.tex(
            stage,
            &format!("{} \\to {}", frac_tex(&short), frac_tex(&next.to_string())),
            44.0,
            hero(),
            &[],
            v(PANEL_X, 0.9),
        )?;
        let quarter = kit.text(stage, A_QUARTER, 26.0, dim(), v(PANEL_X, 0.15))?;
        fade_in_all(stage, &[step, quarter], 0.6)?;
        let third = formula_third(
            &kit,
            stage,
            PRECISELY,
            &[
                Words::Text("after n cuts each colour holds"),
                Words::Tex("\\frac{1-1/4^{n}}{3}"),
                Words::Text(", and each strip is a third of the corner"),
            ],
        )?;
        // the statement comes with the sentence that closes the argument, not
        // at the line's end: it stood 1.3 s before 11_6's card took its place
        line.wait_for(stage, "So no number short")?;
        fade_in_all(stage, &[third], 0.5)?;
        line.wait_end(stage)?;
        pause(stage, 0.8)?;

        // ------------------------------------------------------------ 11_6
        // the card; Archimedes and Ajose and Nelsen
        let line = say(&kit, stage, "11_6")?;
        fade_out_all(stage, &[third, step, quarter], 0.4)?;
        let mut card = proof_card(&kit, stage)?;
        card.show(stage, 0.6)?;
        // the close-up of the fit the first line names: three twins
        let ring = lens(stage);
        let unit = 0.62;
        let put = |stage: &mut Stage<'_>, dx: f64, dy: f64, style: Style| {
            block(
                stage,
                v(LENS_AT[0] + dx * unit, LENS_AT[1] + dy * unit),
                unit - 0.05,
                unit - 0.05,
                style,
            )
        };
        let twins = vec![
            put(stage, -0.55, -0.55, flat(copy(0), 1.0))?,
            put(stage, -0.55, 0.55, flat(copy(1), 1.0))?,
            put(stage, 0.55, -0.55, flat(copy(2), 1.0))?,
            put(stage, 0.55, 0.55, faded_line(dim(), 2.0, 1.0))?,
        ];
        line.wait_for(stage, "Archimedes used")?;
        let mut close = vec![ring];
        close.extend(&twins);
        fade_in_all(stage, &close, 0.5)?;
        card.tick(&kit, stage, 1, EACH_SLIDES)?;
        let tick_one = lens_mark(stage, LENS_AT, 0.7, true);
        fade_in_all(stage, &[tick_one], 0.3)?;
        pause(stage, 0.7)?;
        // the second: the corner in three strips
        let mut bars = Vec::new();
        for (k, &c) in colours.iter().enumerate() {
            let at = v(LENS_AT[0] + (k as f64 - 1.0) * 0.46, LENS_AT[1]);
            bars.push(block(stage, at, 0.42, 1.3, flat(c, 0.9))?);
        }
        let mut swap_out = twins.clone();
        swap_out.push(tick_one);
        fade_out_all(stage, &swap_out, 0.3)?;
        fade_in_all(stage, &bars, 0.4)?;
        card.tick(&kit, stage, 2, THIRD_OF_CORNER)?;
        let tick_two = lens_mark(stage, LENS_AT, 0.7, true);
        fade_in_all(stage, &[tick_two], 0.3)?;
        pause(stage, 0.6)?;
        // the card has passed: its picture goes to the shelf
        let mut leaving = bars.clone();
        leaving.extend([ring, tick_two]);
        leaving.extend(card.mobs());
        fade_out_all(stage, &leaving, 0.4)?;
        file_on_shelf(stage, &zoom, square_mid, SIDE, SLOT, 1.0)?;
        // after the tick: Archimedes, and the source
        line.wait_for(stage, "Sunday Ajose")?;
        let archimedes = lower_third(&kit, stage, ARCHIMEDES, ARCHIMEDES_SAID)?;
        let source = kit.text_left(stage, &ajose_line()?, 22.0, dim(), v(-FRAME_X_RADIUS + 0.45, SOURCE_Y))?;
        fade_in_all(stage, &[archimedes, source], 0.6)?;
        breath(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact_int, testing};
    use super::*;
    use crate::model::q;

    /// The value of a fact fraction, parsed back, to compare with the model.
    fn parse_q(s: &str) -> Option<Q> {
        let (n, d) = match s.split_once('/') {
            Some((n, d)) => (n.parse::<i128>().ok()?, d.parse::<i128>().ok()?),
            None => (s.parse::<i128>().ok()?, 1),
        };
        Some(q(n, d))
    }

    /// The model's coloured square of level `j`, as exact fractions.
    fn counted(j: u32) -> Q {
        series::area(series::quarters(j)[0])
    }

    /// 11_2 and 11_5: the labels, the running shares, the corner and the
    /// strip are the facts, and they are the model's exact sums.
    #[test]
    fn the_readouts_are_the_facts_and_the_models_exact_sums() {
        let pieces = list("F-R.pieces").unwrap();
        let shares = list("F-R.share").unwrap();
        let lefts = list("F-R.left").unwrap();
        assert_eq!(pieces, ["1/4", "1/16", "1/64"]);
        let mut running = Q::ZERO;
        for j in 1..=3u32 {
            let i = j as usize - 1;
            assert_eq!(counted(j).to_string(), pieces[i], "piece {j}");
            running = running + counted(j);
            assert_eq!(running.to_string(), shares[i], "share after {j}");
            assert_eq!(
                series::area(series::left_after_quarters(j)).to_string(),
                lefts[i],
                "corner after {j}"
            );
        }
        // three colours and the corner are the whole square, at every size
        for j in 1..=LEVELS {
            let share = (1..=j).fold(Q::ZERO, |s, k| s + counted(k));
            let corner = series::area(series::left_after_quarters(j));
            assert_eq!(share + share + share + corner, Q::ONE, "after {j} sizes");
            // and each colour's share is a third short by a third of the corner
            assert_eq!(share + corner / Q::int(3), q(1, 3), "after {j} sizes");
        }
        // the strip is the corner's third, and the next one is a quarter
        let strip = series::area(series::left_after_quarters(READ_AFTER)) / Q::int(3);
        assert_eq!(strip.to_string(), fact("F-R.short").unwrap());
        assert_eq!(fact("F-R.short").unwrap(), "1/192");
        let next = series::area(series::left_after_quarters(READ_AFTER + 1)) / Q::int(3);
        assert_eq!(next, strip / Q::int(4));
        assert_eq!(next.to_string(), "1/768");
        // each colour with its strip is a third
        let each = parse_q(&shares[READ_AFTER as usize - 1]).unwrap();
        assert_eq!(each + strip, parse_q(&fact("F-R.limit").unwrap()).unwrap());
        assert_eq!(fact("F-R.limit").unwrap(), "1/3");
        // the quiz's answer is that limit, and its countdown is the model's
        assert_eq!(fact("F-Q.share.answer").unwrap(), "1/3");
        assert_eq!(fact_int("F-Q.share.seconds").unwrap(), 6);
        assert_eq!(super::super::draw::quiz_named("share").unwrap().seconds, 6);
    }

    /// 11_4: at every size the three squares are equal, the upper one is the
    /// coloured one slid up by its own side and the right-hand one is it
    /// slid right; the fourth square is the next corner. Exact.
    #[test]
    fn the_three_squares_of_every_size_are_twins_that_slide_onto_one_another() {
        for j in 1..=LEVELS + 4 {
            let [c, up, right] = series::quarters(j);
            let side = c.2;
            assert_eq!((c.2, c.3), (side, side), "level {j}: a square");
            assert_eq!(
                (up.2, up.3, right.2, right.3),
                (side, side, side, side),
                "level {j}: twins"
            );
            assert_eq!(
                (up.0, up.1),
                (c.0, c.1 + side),
                "level {j}: up is the coloured one slid up"
            );
            assert_eq!(
                (right.0, right.1),
                (c.0 + side, c.1),
                "level {j}: right is it slid right"
            );
            // the fourth square of the corner is the corner of the next size
            let fourth = series::left_after_quarters(j);
            assert_eq!(fourth, (c.0 + side, c.1 + side, side, side), "level {j}");
            // the corner square is four of them
            assert_eq!(Q::int(4) * series::area(c), q(1, 1i128 << (2 * (j - 1))), "level {j}");
        }
        // what the slide animation moves: by the side on screen, down and left
        let board = Board::new(SQUARE_AT, SIDE);
        let mut stage_free = 0;
        for j in 1..=LEVELS {
            let f = series::quarters(j).map(frame);
            let side = f[0][2] * board.cell;
            assert!((f[1][1] - f[0][1] - f[0][3]).abs() < 1e-12);
            assert!((f[2][0] - f[0][0] - f[0][2]).abs() < 1e-12);
            assert!(side > 0.0);
            stage_free += 1;
        }
        assert_eq!(stage_free, LEVELS);
    }

    /// 11_5: the zoom is the picture again. Magnifying the corner after
    /// three sizes by eight about the square's far corner takes size
    /// `j + 3` to size `j`, exactly, and the corner to the whole square.
    #[test]
    fn the_corner_magnified_is_the_same_picture_again() {
        assert_eq!(ZOOM, f64::from(1u32 << READ_AFTER));
        let about = (Q::ONE, Q::ONE);
        let map = |p: series::Piece| -> series::Piece {
            let k = Q::int(1 << READ_AFTER);
            (
                about.0 - k * (about.0 - p.0),
                about.1 - k * (about.1 - p.1),
                k * p.2,
                k * p.3,
            )
        };
        assert_eq!(
            map(series::left_after_quarters(READ_AFTER)),
            (Q::ZERO, Q::ZERO, Q::ONE, Q::ONE)
        );
        for j in 1..=LEVELS - READ_AFTER {
            let (here, there) = (series::quarters(j), series::quarters(j + READ_AFTER));
            for (a, b) in here.iter().zip(&there) {
                assert_eq!(*a, map(*b), "level {j} against level {}", j + READ_AFTER);
            }
        }
    }

    /// 11_3 and 11_4 (the dense read of the narrated draft): the answer
    /// lands under the sum it answers, its top clear of the question's lowest
    /// line (the stacked third's numerator stood on the "64"), and inside
    /// the card above the bottom edge, set no smaller than half its size.
    #[test]
    fn the_answer_lands_under_the_sum_and_inside_the_card() {
        use fmn::scene::{NullSceneSink, RuntimeConfig};
        struct Probe;
        impl SceneConstruct for Probe {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                let kit = Kit::new()?;
                let pieces = list("F-R.pieces")?;
                let answer = fact("F-Q.share.answer")?;
                let mut sum: Vec<String> = pieces.iter().map(|p| frac_tex(p)).collect();
                sum.push("\\cdots".to_owned());
                let question = format!("{} = \\,?", sum.join(" + "));
                let answer_tex = frac_tex(&answer);
                let card = QuizCard::build(
                    &kit,
                    stage,
                    "share",
                    Words::Tex(&question),
                    Words::Tex(&answer_tex),
                    QuizLook::BESIDE,
                )?;
                let before = stage.get_bottom(card.question)[1] - stage.get_top(card.answer)[1];
                let tall = stage.get_height(card.answer);
                let height = fit_answer(stage, &card);
                let after = stage.get_bottom(card.question)[1] - stage.get_top(card.answer)[1];
                eprintln!("answer {tall:.2} high, {height:.2} after, {before:.2} then {after:.2} under the question");
                assert!(
                    before < ANSWER_CLEAR,
                    "the stacked answer stood on the question: {before:.2}"
                );
                assert!(
                    after >= ANSWER_CLEAR - 1e-9,
                    "the answer's top is {after:.2} under the question"
                );
                // still a number to read across the room: at least half its size
                assert!(
                    height >= 0.5 * tall,
                    "the answer was shrunk to {height:.2} from {tall:.2}"
                );
                // inside the card, above its bottom edge and clear of the
                // sides
                let (w, h) = QuizLook::BESIDE.size();
                let at = QuizLook::BESIDE.at;
                assert!(stage.get_bottom(card.answer)[1] > at[1] - h / 2.0 + ANSWER_FOOT - 1e-9);
                assert!(stage.get_left(card.answer)[0] > at[0] - w / 2.0 + 0.2);
                assert!(stage.get_right(card.answer)[0] < at[0] + w / 2.0 - 0.2);
                Ok(())
            }
        }
        fmn::run_scene(&mut Probe, RuntimeConfig::default(), 7, &mut NullSceneSink).expect("the probe runs");
    }

    /// 11_6 (the dense read of the scored draft, mx-e-a-dza.10): the card's
    /// foot stands at least `CARD_FOOT_CLEAR` above the frame's edge (it
    /// ended 9 px up, under a player's control bar), and the card, which
    /// rises to keep it there, stays clear of the square and of the close-up
    /// ring over it.
    #[test]
    fn the_card_stands_clear_of_the_controls_and_the_square_clear_of_the_card() {
        use fmn::scene::{NullSceneSink, RuntimeConfig};
        struct Probe;
        impl SceneConstruct for Probe {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                let kit = Kit::new()?;
                let card = proof_card(&kit, stage)?;
                let (top, foot) = (stage.get_top(card.back)[1], stage.get_bottom(card.back)[1]);
                eprintln!("card top {top:.3}, foot {foot:.3}; square foot {:.3}", SQUARE_AT[1]);
                let floor = -FRAME_Y_RADIUS + super::super::draw::CARD_FOOT_CLEAR;
                assert!(foot >= floor - 1e-9, "the card's foot is {foot:.3}, under {floor:.3}");
                // it rose: this is the card that did
                assert!(top > super::super::draw::CARD_TOP + 0.1, "the card's top is {top:.3}");
                assert!(
                    SQUARE_AT[1] >= top + CARD_GAP - 1e-9,
                    "the square's foot {:.3} is under the card's top {top:.3} + {CARD_GAP}",
                    SQUARE_AT[1]
                );
                assert!(LENS_AT[1] - super::super::draw::LENS_R > top, "the ring meets the card");
                Ok(())
            }
        }
        fmn::run_scene(&mut Probe, RuntimeConfig::default(), 7, &mut NullSceneSink).expect("the probe runs");
    }

    /// The picture keeps clear of the shelf's row and the card's top (the
    /// consts above assert it at compile time), and the smallest piece is
    /// still a visible speck at 4K.
    #[test]
    fn the_smallest_piece_is_still_a_speck_at_4k() {
        // the smallest coloured square on screen, in pixels at 3840 by 2160
        let smallest = series::quarters(LEVELS)[0].2.to_f64() * SIDE;
        let px = smallest * 2160.0 / (2.0 * FRAME_Y_RADIUS);
        assert!(px > 4.0, "{px} px");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("11_a_third").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "\\frac{1}{4}",
            "\\frac{1}{16}",
            "\\frac{1}{64}",
            "\\frac{1}{4} + \\frac{1}{16} + \\frac{1}{64} + \\cdots = \\,?",
            "\\frac{1}{3}",
            "\\frac{21}{64}",
            "\\frac{1}{192}",
            "\\frac{1}{192} \\to \\frac{1}{768}",
            "a quarter as big",
            "S. A. Ajose and R. B. Nelsen, Mathematics Magazine, 1994",
            "each slides onto the other",
            "Archimedes, Quadrature of the Parabola, Prop. 23:",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // R2: the lower-third states Archimedes's hypothesis, and its words
        // (wrapped in lines) are whole, with no ellipsis cutting any of them
        let said: String = words.join(" ");
        assert!(
            said.contains(
                "if each piece is a quarter of the one before, a finite sum plus a third of its last piece is exactly 4/3 of its first"
            ),
            "the lower-third's words are not whole: {words:?}"
        );
        assert!(!said.contains(" …"), "{said}");
        assert!(ran.seconds > 40.0 && ran.seconds < 90.0, "{} s", ran.seconds);
    }
}
