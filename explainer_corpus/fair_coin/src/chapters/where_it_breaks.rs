//! Chapter 05, where it breaks: the two things the trick leans on, broken
//! one at a time. A fair coin for the second toss (05_1) moves one cut and
//! not the other, so the blocks are no longer mirror images: forty percent
//! against ten (05_2). A coin with a memory puts a step in the second cut:
//! after two heads, heads then tails is four times as likely, and after two
//! tails the other way round (05_3). The cure is to keep only the tosses
//! that follow a head (05_4).

use std::rc::Rc;

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{RowStrips, lower_third, predict_pause};
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, fade_all, hex, play, thousands, tween, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::draw::{
    Block, SQUARE_AT, SQUARE_SIDE, SplitBar, Square, SquarePose, SquareView, alice, bob, caption, disc, disc_row,
    face_color, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, gold, heads, lettered_disc, line_style, rgb,
    say, tails, thrown, tween_all, warn,
};
use crate::facts::{after_heads, cited, decided_after, second_coin, sticky, sticky_run, two_coin_run};
use crate::model::{Face, PairLaw, Q, bent, heads_first_wins, von_neumann};

pub struct WhereItBreaks {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const HEADS_THEN_TAILS: &str = "heads then tails";
const TAILS_THEN_HEADS: &str = "tails then heads";
const FIRST_TOSS: &str = "first toss";
const SECOND_TOSS: &str = "second toss";
const OUR_COIN: &str = "our unfair coin";
const FAIR_COIN: &str = "a fair coin";
const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const COUNTED_IN: &str = "counted in";
const GAMES: &str = "games";
const FOR_ALICE: &str = "for Alice";
const TOSSES: &str = "tosses";
const AGAINST: &str = "against";
const MODEL: &str = "a model:";
const HALF_HEADS: &str = "half heads in the long run,";
const REPEATS: &str = "repeats its last result";
const OF_THE_TIME: &str = "of the time";
const ONE_CHANGE: &str = "one change";
const TWO_CHANGES: &str = "two changes";
const AFTER_TWO_HEADS: &str = "after two heads";
const AFTER_TWO_TAILS: &str = "after two tails";
const THE_PAIR: &str = "the pair";
const AND: &str = "and";
const AFTER_A_HEAD: &str = "two tosses that each follow a head";
const ONLY_AFTER_A_HEAD: &str = "only the tosses that come straight after a head";
const REAL_COINS: &str = "real coins:";
const DO_THIS_A_LITTLE: &str = "tossed coins do this a little: about";
const LAND_AS_STARTED: &str = "in 100 land the way they started";
const DIACONIS: &str = "Diaconis, Holmes and Montgomery";
const BARTOS: &str = "Bartoš and others";
const THAT_IS_A_MEMORY: &str = "That is a memory when each toss starts the way the last one landed";
const ONE_TOSS_MEMORY: &str = "for a coin that remembers only its last toss:";
const SAMUELSON: &str = "Samuelson";
const FAIR_TO_THE_END: &str = "Fair when played to the end. Using every toss takes more care: Blum";

/// The right-hand column: heads then tails on the left, tails then heads on
/// the right, the sign between them.
const LEFT_X: f64 = 1.15;
const MID_X: f64 = 3.05;
const RIGHT_X: f64 = 4.95;
/// The blocks' names, their shares and their products, top to bottom.
const NAMES_Y: f64 = 2.45;
const READ_Y: f64 = 1.8;
const PRODUCT_Y: f64 = 1.12;
const READ_SIZE: f64 = 40.0;
/// The first toss's slider, under the square; the second's stands this far
/// to the square's right.
const SLIDER_Y: f64 = -2.45;
const SLIDER_OUT: f64 = 0.45;
/// The row of the remembering coin's tosses, and the row of those kept.
const ROW_LEFT: f64 = -0.5;
const ROW_STEP: f64 = 0.27;
const ROW_R: f64 = 0.105;
const ROW_N: usize = 26;
const TOSS_Y: f64 = -0.5;
const KEPT_Y: f64 = -0.95;

/// An equals sign as two gold strokes, `w` wide, centred at `at` (the fonts'
/// own is a pair of hairlines at draft size). Detached, one group.
fn equals_sign(stage: &mut Stage<'_>, at: Vec3, w: f64) -> Mob {
    let bars = [0.075, -0.075].map(|dy| {
        polyline(vec![v(at[0] - w / 2.0, at[1] + dy), v(at[0] + w / 2.0, at[1] + dy)])
            .style(line_style(gold(), 4.5))
            .build()
    });
    stage.arena_mut().add(v_group(bars))
}

/// Keep four edge labels (under the bottom edge's heads and tails stretches,
/// then beside the left edge's) at the middle of their stretches as the cuts
/// move. Returns the anchor that carries the updater: it must be on stage.
fn follow_edges(stage: &mut Stage<'_>, view: SquareView, labels: [Mob; 4]) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last: Option<SquarePose> = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let pose = view.pose(s);
            if last == Some(pose) {
                return;
            }
            let (below, beside) = (pose.bottom - 0.38, pose.left - 0.55);
            let spots = [
                v(pose.point(pose.x / 2.0, 0.0)[0], below),
                v(pose.point((1.0 + pose.x) / 2.0, 0.0)[0], below),
                v(beside, pose.point(0.0, pose.y_h / 2.0)[1]),
                v(beside, pose.point(0.0, (1.0 + pose.y_h) / 2.0)[1]),
            ];
            for (&m, at) in labels.iter().zip(spots) {
                s.move_to(m, at, ORIGIN);
            }
            last = Some(pose);
        },
        true,
    )?;
    Ok(anchor)
}

/// A whole percentage as a live number with its sign. A number with no
/// decimals is cut short, not rounded, so it is rounded here (a half of four
/// fifths is a hair over 40 in floating point, and could as well be under).
fn percent(book: &FontBook, value: f64, size: f64) -> VMobject {
    DecimalNumber::new(value.round())
        .num_decimal_places(0)
        .unit("%")
        .font_size(size)
        .color(Palette::ink())
        .build(book)
        .expect("bundled digits always lay out")
        .into_vmob()
}

/// A block's share of the square as a live percentage, its right edge at
/// `at` (as `Square::share_readout`, but rounded to the nearest percent). It
/// appears at once.
fn share_readout(kit: &Kit, stage: &mut Stage<'_>, view: SquareView, b: Block, at: Vec3) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let m = s.add(percent(&book, 100.0 * view.pose(s).share(b), READ_SIZE));
        s.move_to(m, at, RIGHT);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Which cut a slider moves, and so where it lies.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cut {
    /// The first toss's: under the square.
    First,
    /// The second toss's: beside the square, to its right.
    Second,
}

/// A slider for one of the square's cuts: its track runs the length of the
/// square's side, so the knob sits at the end of the cut it moves.
struct Slider {
    cut: Cut,
    /// Carries the updater.
    anchor: Mob,
    track: Mob,
    ticks: Mob,
    knob: Mob,
    label: Mob,
    /// Where the chance is written, after the label (its left edge).
    number_at: Vec3,
}

impl Slider {
    fn build(kit: &Kit, stage: &mut Stage<'_>, square: &Square, cut: Cut, words: &str) -> fmn::Result<Self> {
        let (view, pose) = (square.view, square.pose(stage));
        let (right, top) = (pose.left + pose.side, pose.bottom + pose.side);
        let (from, to, tick) = match cut {
            Cut::First => (v(pose.left, SLIDER_Y), v(right, SLIDER_Y), v(0.0, 0.09)),
            Cut::Second => (
                v(right + SLIDER_OUT, pose.bottom),
                v(right + SLIDER_OUT, top),
                v(0.09, 0.0),
            ),
        };
        let at = move |t: f64| v(from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t);
        let rule = line_style(Palette::dim(), 3.0);
        let track = stage.arena_mut().add(polyline(vec![from, to]).style(rule).build());
        // a tick at each end and in the middle, where a coin is fair
        let ticks = [0.0, Q::HALF.f64(), 1.0].map(|t| {
            let c = at(t);
            polyline(vec![
                v(c[0] - tick[0], c[1] - tick[1]),
                v(c[0] + tick[0], c[1] + tick[1]),
            ])
            .style(rule)
            .build()
        });
        let ticks = stage.arena_mut().add(v_group(ticks));
        let knob = Circle::new().radius(0.11).style(flat(Palette::ink(), 1.0)).build();
        let knob = stage.arena_mut().add(knob);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last: Option<f64> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let pose = view.pose(s);
                let t = if cut == Cut::First { pose.x } else { pose.y_h };
                if last == Some(t) {
                    return;
                }
                s.move_to(knob, at(t), ORIGIN);
                last = Some(t);
            },
            true,
        )?;
        let label_at = match cut {
            Cut::First => v(right + 0.35, SLIDER_Y),
            Cut::Second => v(right + SLIDER_OUT - 1.05, top + 0.42),
        };
        let label = kit.text_left(stage, words, 24.0, Palette::dim(), label_at)?;
        let number_at = v(stage.get_right(label)[0] + 0.18, label_at[1]);
        Ok(Self {
            cut,
            anchor,
            track,
            ticks,
            knob,
            label,
            number_at,
        })
    }

    fn mobs(&self) -> [Mob; 5] {
        [self.anchor, self.track, self.ticks, self.knob, self.label]
    }

    /// The cut's chance of heads as a live percentage ("80%"). It appears at
    /// once.
    fn number(&self, kit: &Kit, stage: &mut Stage<'_>, view: SquareView) -> fmn::Result<Mob> {
        let book = Rc::clone(&kit.book);
        let (cut, at) = (self.cut, self.number_at);
        let m = stage.always_redraw(move |s| {
            let pose = view.pose(s);
            let t = if cut == Cut::First { pose.x } else { pose.y_h };
            let m = s.add(percent(&book, 100.0 * t, 26.0));
            s.move_to(m, at, LEFT);
            m
        });
        stage.add_to_scene(m)?;
        Ok(m)
    }
}

/// A dot at the end of each column's cut, where it meets the first toss's
/// cut. With one coin they are one dot on the diagonal; a second coin takes
/// both off it; a memory takes one off. Returns the anchor (which carries
/// the updater) and the two dots.
fn crossing_dots(stage: &mut Stage<'_>, view: SquareView) -> fmn::Result<[Mob; 3]> {
    let style = Style::default().fill(Palette::ink(), 1.0).stroke(gold(), 3.0, 1.0);
    let dots = [(); 2].map(|()| stage.arena_mut().add(Circle::new().radius(0.075).style(style).build()));
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last: Option<SquarePose> = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let pose = view.pose(s);
            if last == Some(pose) {
                return;
            }
            s.move_to(dots[0], pose.point(pose.x, pose.y_h), ORIGIN);
            s.move_to(dots[1], pose.point(pose.x, pose.y_t), ORIGIN);
            last = Some(pose);
        },
        true,
    )?;
    Ok([anchor, dots[0], dots[1]])
}

/// How a coin is told from the others.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Look {
    /// Our unfair coin, as chapter 01 draws it.
    Plain,
    /// The fair coin: a light rim.
    Rimmed,
    /// The coin with a memory: its last result still showing behind it.
    Remembering,
}

/// A coin showing heads, centred at `at`: its parts, back to front,
/// detached.
fn coin(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, look: Look) -> fmn::Result<Vec<Mob>> {
    let r = 0.32;
    let mut parts = Vec::new();
    if look == Look::Remembering {
        let behind = disc(Face::H, v(at[0] - 0.15, at[1] + 0.11), r).with_style(flat(heads(), 0.4));
        parts.push(stage.arena_mut().add(behind));
    }
    parts.extend(lettered_disc(kit, stage, Face::H, at, r, 34.0)?);
    if look == Look::Rimmed {
        let rim = Circle::new()
            .radius(r + 0.08)
            .arc_center(at)
            .style(line_style(Palette::ink(), 2.5))
            .build();
        parts.push(stage.arena_mut().add(rim));
    }
    Ok(parts)
}

/// One coin of the two-coin game, as a row of the right-hand column: the
/// coin, which toss it makes, and a bar of its chances with heads written
/// out. Detached, back to front.
fn coin_row(
    kit: &Kit,
    stage: &mut Stage<'_>,
    y: f64,
    (toss, name): (&str, &str),
    chance: Q,
    look: Look,
) -> fmn::Result<Vec<Mob>> {
    let mut parts = coin(kit, stage, v(-0.3, y - 0.05), look)?;
    let words = format!("{toss}: {name}");
    parts.push(kit.text_left(stage, &words, 26.0, Palette::ink(), v(0.3, y + 0.14))?);
    let bar = SplitBar::build(stage, v(0.3, y - 0.26), (3.6, 0.18), (heads(), tails()), chance.f64())?;
    parts.extend(bar.mobs());
    let share = format!("{HEADS} {}%", chance.percent(0));
    parts.push(kit.text_left(stage, &share, 24.0, Palette::dim(), v(4.1, y - 0.26))?);
    Ok(parts)
}

/// How many games a row of the grid of games holds.
const PATTER_COLS: usize = 100;

/// Every game of a run as a small cell in its winner's colour, in the order
/// they were played, [`PATTER_COLS`] to a row, `width` across from
/// `top_left`. The cells are on stage unseen (the background's colour) and
/// take their colours by the clock, since a listen window has no animation
/// of its own: the first `delay` seconds from now, the last `span` seconds
/// later. (The run's first forty games alone would mislead: Alice happens to
/// win thirty-eight of them.) Returns what to clear away: the anchor, which
/// carries the updater, and the grid.
fn patter(
    stage: &mut Stage<'_>,
    winners: &[Srgb],
    top_left: Vec3,
    width: f64,
    (delay, span): (f64, f64),
) -> fmn::Result<[Mob; 2]> {
    let cell = width / PATTER_COLS as f64;
    let rows = winners.len().div_ceil(PATTER_COLS);
    let unseen = hex(BACKGROUND);
    let colors = move |played: usize, winners: &[Srgb]| -> Vec<Vec<Srgb>> {
        (0..rows)
            .map(|r| {
                (0..PATTER_COLS)
                    .map(|c| match winners.get(r * PATTER_COLS + c) {
                        Some(&w) if r * PATTER_COLS + c < played => w,
                        _ => unseen,
                    })
                    .collect()
            })
            .collect()
    };
    let grid = RowStrips::build(stage, top_left, (cell, cell), &colors(0, winners))?;
    let cells = grid.group;
    stage.add_to_scene(cells)?;
    let anchor = stage.add(VMobject::new())?;
    let winners = winners.to_vec();
    let (mut clock, mut played) = (0.0, 0);
    stage.add_dt_updater(
        anchor,
        move |s, _, dt| {
            clock += dt;
            let due = (((clock - delay) / span).clamp(0.0, 1.0) * winners.len() as f64).ceil() as usize;
            if due != played {
                let _ = grid.recolor(s, &colors(due, &winners));
                played = due;
            }
        },
        false,
    )?;
    Ok([anchor, cells])
}

/// `rt` seconds, for an `animate()` chain.
fn run(rt: f64) -> AnimateArgs {
    AnimateArgs {
        run_time: Some(rt),
        ..AnimateArgs::default()
    }
}

/// The words for a number of changes of face.
fn changes(n: usize) -> fmn::Result<&'static str> {
    match n {
        1 => Ok(ONE_CHANGE),
        2 => Ok(TWO_CHANGES),
        _ => Err(fail(format!("a pair after a head makes one change or two, not {n}"))),
    }
}

/// One way on from a head, as a row with its left end at `left`: the head
/// before (faded), the pair, a tick wherever the face changes, and the
/// pair's name with how many changes it took. Detached.
fn way_on(kit: &Kit, stage: &mut Stage<'_>, left: Vec3, pair: (Face, Face), name: &str) -> fmn::Result<Vec<Mob>> {
    let (r, step) = (0.13, 0.38);
    let faces = [Face::H, pair.0, pair.1];
    let mut parts = Vec::new();
    for (k, f) in faces.into_iter().enumerate() {
        let fade = if k == 0 { 0.45 } else { 1.0 };
        let d = disc(f, v(left[0] + r + step * k as f64, left[1]), r).with_style(flat(face_color(f), fade));
        parts.push(stage.arena_mut().add(d));
    }
    let mut count = 0;
    for k in 0..2 {
        if faces[k] != faces[k + 1] {
            count += 1;
            let x = left[0] + r + step * (k as f64 + 0.5);
            let tick = polyline(vec![v(x, left[1] - r), v(x, left[1] + r)])
                .style(line_style(Palette::ink(), 3.0))
                .build();
            parts.push(stage.arena_mut().add(tick));
        }
    }
    let words = format!("{name}: {}", changes(count)?);
    let at = v(left[0] + 2.0 * r + 2.0 * step + 0.22, left[1]);
    parts.push(kit.text_left(stage, &words, 24.0, Palette::ink(), at)?);
    Ok(parts)
}

impl SceneConstruct for WhereItBreaks {
    fn name(&self) -> &str {
        "05_where_it_breaks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coin_chance = bent();
        let p = coin_chance.f64();
        let one_coin = PairLaw::same_coin(coin_chance);

        // the square as chapter 03 left it: one coin, the two blocks mirror
        // images in the gold diagonal, the cuts crossing on it
        let square = Square::build(stage, SQUARE_AT, SQUARE_SIDE, p)?;
        square.paint(stage, Block::HT, alice(), 1.0);
        square.paint(stage, Block::TH, bob(), 1.0);
        square.paint(stage, Block::HH, thrown(), 1.0);
        square.paint(stage, Block::TT, thrown(), 1.0);
        let edges = square.edges(&kit, stage, HEADS, TAILS)?;
        let words_follow = follow_edges(stage, square.view, edges.words)?;
        let diagonal = square.diagonal(stage)?;
        let dots = crossing_dots(stage, square.view)?;
        let pose = square.pose(stage);
        let top_edge = pose.bottom + pose.side;
        // beside it: each block's name over its share, the sign between
        let mut legend = Vec::new();
        for (x, name, color) in [(LEFT_X, HEADS_THEN_TAILS, alice()), (RIGHT_X, TAILS_THEN_HEADS, bob())] {
            let text = kit.text(stage, name, 26.0, Palette::ink(), v(x + 0.25, NAMES_Y))?;
            let left = stage.get_left(text)[0];
            let swatch = Rectangle::new()
                .width(0.3)
                .height(0.3)
                .style(flat(color, 1.0))
                .build()?
                .moved_to(v(left - 0.32, NAMES_Y));
            legend.extend([stage.arena_mut().add(swatch), text]);
        }
        let mut reads = Vec::new();
        for (x, (a, c)) in [(LEFT_X, (Face::H, Face::T)), (RIGHT_X, (Face::T, Face::H))] {
            let text = format!("{}%", one_coin.prob(a, c).percent(0));
            reads.push(kit.text(stage, &text, READ_SIZE, Palette::ink(), v(x, READ_Y))?);
        }
        let equals = equals_sign(stage, v(MID_X, READ_Y), 0.36);
        // the stroke that says the two shares are no longer equal
        let slash = polyline(vec![v(MID_X - 0.2, READ_Y - 0.27), v(MID_X + 0.2, READ_Y + 0.27)])
            .style(line_style(warn(), 5.0))
            .build();
        let slash = stage.arena_mut().add(slash);
        let live_reads = |stage: &mut Stage<'_>| -> fmn::Result<[Mob; 2]> {
            Ok([
                share_readout(&kit, stage, square.view, Block::HT, v(LEFT_X + 0.45, READ_Y))?,
                share_readout(&kit, stage, square.view, Block::TH, v(RIGHT_X + 0.45, READ_Y))?,
            ])
        };

        // 05_1: our unfair coin first, a fair coin second
        let line = say(&kit, stage, "05_1")?;
        let mut opening = square.mobs();
        opening.extend(edges.mobs());
        opening.extend([words_follow, diagonal]);
        opening.extend(dots);
        opening.extend(&legend);
        opening.extend(&reads);
        opening.push(equals);
        fade_in_all(stage, &opening, 0.7)?;
        let fair = second_coin();
        line.wait_for(stage, "Toss our unfair coin first")?;
        let first_coin = coin_row(&kit, stage, 0.95, (FIRST_TOSS, OUR_COIN), coin_chance, Look::Plain)?;
        let first_slider = Slider::build(&kit, stage, &square, Cut::First, FIRST_TOSS)?;
        let mut ins = first_coin.clone();
        ins.extend(first_slider.mobs());
        fade_in_all(stage, &ins, 0.5)?;
        let first_number = first_slider.number(&kit, stage, square.view)?;
        // the second cut gets a slider of its own
        line.wait_for(stage, "and a fair coin second")?;
        let second_coin_row = coin_row(&kit, stage, 0.0, (SECOND_TOSS, FAIR_COIN), fair, Look::Rimmed)?;
        let second_slider = Slider::build(&kit, stage, &square, Cut::Second, SECOND_TOSS)?;
        let mut ins = second_coin_row.clone();
        ins.extend(second_slider.mobs());
        fade_in_all(stage, &ins, 0.5)?;
        let second_number = second_slider.number(&kit, stage, square.view)?;
        line.wait_for(stage, "Is heads then tails still equal")?;
        let asks = [
            kit.text(stage, "?", READ_SIZE, gold(), v(LEFT_X, READ_Y))?,
            kit.text(stage, "?", READ_SIZE, gold(), v(RIGHT_X, READ_Y))?,
        ];
        fade_out_all(stage, &[reads[0], reads[1], equals], 0.3)?;
        fade_in_all(stage, &asks, 0.3)?;
        // each name swells as the question reaches it
        let ink = Some(rgb(Palette::ink()));
        play!(stage; indicate(stage.arena_mut(), legend[1], 1.18, ink)?.rt(0.8));
        line.wait_for(stage, "to tails then heads")?;
        play!(stage; indicate(stage.arena_mut(), legend[3], 1.18, ink)?.rt(0.8));
        let question = predict_pause(&kit, stage, v(MID_X, -1.75))?;

        // 05_2: one cut moved and the other didn't
        let line = say(&kit, stage, "05_2")?;
        fade_out_all(stage, &[asks[0], asks[1], question], 0.3)?;
        let two_coin_reads = live_reads(stage)?;
        line.wait_for(stage, "One cut moved")?;
        tween_all(
            stage,
            &[(square.view.y_h, fair.f64()), (square.view.y_t, fair.f64())],
            1.3,
        )?;
        // the flip of chapter 03 again: the copy of heads then tails no
        // longer fits tails then heads, and what hangs over is red
        line.wait_for(stage, "so the blocks aren't mirror images")?;
        let [ghost, mirror] = square.ghost_and_mirror(stage, Block::HT, alice());
        fade_in_all(stage, &[ghost], 0.3)?;
        play!(stage; Transform::new(ghost, mirror).rt(1.7));
        let pose = square.pose(stage);
        let (ht, th) = (pose.unit_rect(Block::HT), pose.unit_rect(Block::TH));
        // heads then tails, mirrored: across from its low edge to the square's
        // right side, up as far as it is wide
        let (from_u, to_w) = (ht.1, ht.2);
        if from_u >= th.0 || to_w <= th.3 {
            return Err(fail("the mirrored block does not hang over tails then heads".into()));
        }
        let over = [
            pose.point(from_u, 0.0),
            pose.point(th.0, 0.0),
            pose.point(th.0, th.3),
            pose.point(1.0, th.3),
            pose.point(1.0, to_w),
            pose.point(from_u, to_w),
        ];
        let overhang = stage
            .arena_mut()
            .add(Polygon::new(over).style(flat(warn(), 0.88)).build());
        // the copy keeps only its gold edge: inside it, tails then heads and
        // the red that does not fit
        play!(stage;
            ghost.animate().set_anim_args(run(0.5))?.set_fill(None, Some(0.0))?,
            fade_in(stage.arena_mut(), overhang, ORIGIN, 1.0)?.rt(0.5),
        );
        stage.scene_mut().bring_to_front(&[ghost])?;
        stage.scene_mut().bring_to_front(&dots)?;
        line.wait_for(stage, "Forty percent against ten")?;
        fade_in_all(stage, &[equals, slash], 0.4)?;
        // a thousand games of it: Alice's share of them
        line.wait_for(stage, "Alice wins four games in five")?;
        let games = two_coin_run();
        let wins = heads_first_wins(&games);
        let n = games.len();
        let (bar_left, bar_w, bar_h) = (v(-0.6, -1.22), 6.6, 0.26);
        let bar = SplitBar::build(stage, bar_left, (bar_w, bar_h), (alice(), bob()), 0.5)?;
        let tally_y = bar_left[1] + 0.4;
        let alice_word = kit.text_left(stage, ALICE, 28.0, alice(), v(bar_left[0], tally_y))?;
        let alice_n = v(stage.get_right(alice_word)[0] + 0.2, tally_y);
        let alice_count = kit.text_left(stage, &wins.to_string(), 30.0, Palette::ink(), alice_n)?;
        let bob_count = kit.text(stage, &(n - wins).to_string(), 30.0, Palette::ink(), ORIGIN)?;
        stage.move_to(bob_count, v(bar_left[0] + bar_w, tally_y), RIGHT);
        let bob_word = kit.text(stage, BOB, 28.0, bob(), ORIGIN)?;
        let bob_at = v(stage.get_left(bob_count)[0] - 0.2, tally_y);
        stage.move_to(bob_word, bob_at, RIGHT);
        let counted = format!("{COUNTED_IN} {} {GAMES}: {wins} {FOR_ALICE}", thousands(n as u64));
        let games_caption = caption(&kit, stage, &counted)?;
        let mut tally = bar.mobs().to_vec();
        tally.extend([alice_word, alice_count, bob_word, bob_count, games_caption]);
        fade_in_all(stage, &tally, 0.5)?;
        play!(stage; tween(bar.share, wins as f64 / n as f64, 1.0)?);
        // every game of the run, a cell each in its winner's colour, in the
        // order they were played, through the listen window: four to one
        let winners: Vec<Srgb> = games
            .iter()
            .map(|g| {
                if g.last().is_some_and(|pair| pair.0 == Face::H) {
                    alice()
                } else {
                    bob()
                }
            })
            .collect();
        kit.hold(stage)?;
        let under_bar = v(bar_left[0], bar_left[1] - bar_h / 2.0 - 0.15);
        let landed = patter(stage, &winners, under_bar, bar_w, (0.2, 2.6))?;
        kit.listen(stage, 3.0, "05_where_it_breaks:05_2")?;

        // 05_3: a coin with a memory
        let line = say(&kit, stage, "05_3")?;
        let memory = sticky();
        if memory.long_run_heads() != Some(Q::HALF) {
            return Err(fail("the coin with a memory is not half heads in the long run".into()));
        }
        let run = sticky_run();
        if run.len() < ROW_N {
            return Err(fail(format!("the run has {} tosses, the row {ROW_N}", run.len())));
        }
        let mut gone = first_coin;
        gone.extend(second_coin_row);
        gone.extend(first_slider.mobs());
        gone.extend(second_slider.mobs());
        gone.extend([first_number, second_number, ghost, overhang, equals, slash]);
        gone.extend(two_coin_reads);
        gone.extend(&tally);
        gone.extend(&landed);
        fade_out_all(stage, &gone, 0.4)?;
        // one coin again, and a different-looking one beside it
        square.set_coin(stage, p, 0.8)?;
        let mut the_coin = coin(&kit, stage, v(-0.2, 0.28), Look::Remembering)?;
        let tag_a = format!("{MODEL} {HALF_HEADS}");
        let tag_b = format!("{REPEATS} {}% {OF_THE_TIME}", memory.after_h.percent(0));
        the_coin.push(kit.text_left(stage, &tag_a, 24.0, Palette::dim(), v(0.4, 0.47))?);
        the_coin.push(kit.text_left(stage, &tag_b, 24.0, Palette::dim(), v(0.4, 0.13))?);
        fade_in_all(stage, &the_coin, 0.5)?;
        // its tosses: about half are heads, and they come in runs
        line.wait_for(stage, "Count this one's heads")?;
        let row = &run[..ROW_N];
        let at = |i: usize, y: f64| v(ROW_LEFT + ROW_STEP * i as f64, y);
        let tosses = disc_row(stage, row, at(0, TOSS_Y), ROW_R, ROW_STEP);
        fade_in_lagged(stage, &tosses, 1.6, 0.08)?;
        line.wait_for(stage, "and it looks fair")?;
        let run_heads = run.iter().filter(|&&f| f == Face::H).count();
        let share = Q::new(run_heads as i128, run.len() as i128).percent(1);
        let total = thousands(run.len() as u64);
        let looks_fair = format!("{COUNTED_IN} {total} {TOSSES}: {share}% {HEADS}");
        let looks_fair = kit.text_left(stage, &looks_fair, 22.0, Palette::dim(), v(ROW_LEFT - ROW_R, KEPT_Y))?;
        fade_in_all(stage, &[looks_fair], 0.4)?;
        line.wait_for(stage, "but it copies its last result")?;
        let mut copies = Vec::new();
        let mut start = 0;
        for i in 1..=ROW_N {
            if i == ROW_N || row[i] != row[start] {
                if i - start > 1 {
                    let y = TOSS_Y - ROW_R - 0.08;
                    let under = polyline(vec![at(start, y), at(i - 1, y)])
                        .style(line_style(face_color(row[start]), 3.5))
                        .build();
                    copies.push(stage.arena_mut().add(under));
                }
                start = i;
            }
        }
        fade_in_lagged(stage, &copies, 1.0, 0.2)?;
        // the pair after two heads: the first toss still cuts at four fifths,
        // but the second cut has a step in it
        line.wait_for(stage, "After two heads")?;
        let after_h = memory.pair_after(Face::H);
        let after_t = memory.pair_after(Face::T);
        // what the square now shows is written over the two names
        let header = v(MID_X, NAMES_Y + 0.65);
        let state = kit.text(
            stage,
            &format!("{THE_PAIR} {AFTER_TWO_HEADS}"),
            26.0,
            Palette::ink(),
            header,
        )?;
        let real = format!(
            "{DO_THIS_A_LITTLE} {} {LAND_AS_STARTED} ({DIACONIS}, {}; {} {TOSSES}: {}, {BARTOS}, {}). {THAT_IS_A_MEMORY}",
            cited::SAME_SIDE_PERCENT,
            cited::DIACONIS_YEAR,
            thousands(cited::BARTOS_FLIPS),
            cited::BARTOS_SAME_SIDE,
            cited::BARTOS_YEAR,
        );
        let real = lower_third(&kit, stage, REAL_COINS, &real)?;
        let mut step: Vec<Box<dyn Animation>> = Vec::new();
        for (t, to) in [
            (square.view.x, after_h.first),
            (square.view.y_h, after_h.after_h),
            (square.view.y_t, after_h.after_t),
        ] {
            step.push(stage.prepare(tween(t, to.f64(), 1.2)?)?);
        }
        for m in [state, real] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            step.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(step)?;
        // the two ways on from a head, and what each costs
        let product = |stage: &mut Stage<'_>, (a, b): (Q, Q), x: f64| {
            let src = format!(r"{} \times {}", a.exact(), b.exact());
            kit.tex(stage, &src, 34.0, Palette::ink(), &[], v(x, PRODUCT_Y))
        };
        let ways = [
            (
                "heads then tails needs one change",
                (Face::H, Face::T),
                HEADS_THEN_TAILS,
                (after_h.first, after_h.after_h.other()),
                LEFT_X,
            ),
            (
                "and tails then heads needs two",
                (Face::T, Face::H),
                TAILS_THEN_HEADS,
                (after_h.first.other(), after_h.after_t),
                RIGHT_X,
            ),
        ];
        let mut rows = Vec::new();
        let mut products = Vec::new();
        for (k, (phrase, pair, name, chances, x)) in ways.into_iter().enumerate() {
            line.wait_for(stage, phrase)?;
            let left = v(pose.left + 0.1, top_edge + 0.8 - 0.43 * k as f64);
            let mut ins = way_on(&kit, stage, left, pair, name)?;
            rows.extend(&ins);
            let m = product(stage, chances, x)?;
            products.push(m);
            ins.push(m);
            fade_in_all(stage, &ins, 0.6)?;
        }
        // after two tails the same two products change sides
        let mirrored = [
            (after_t.first, after_t.after_h.other()),
            (after_t.first.other(), after_t.after_t),
        ];
        if mirrored != [ways[1].3, ways[0].3] {
            return Err(fail(
                "after two tails the two products are not the same two, swapped".into(),
            ));
        }
        // four times as likely: the shares, and what a long run counted
        line.wait_for(stage, "So heads then tails is four times as likely")?;
        live_reads(stage)?;
        fade_in_all(stage, &[equals, slash], 0.4)?;
        let (ht_hh, th_hh) = decided_after(&run, (Face::H, Face::H));
        let (ht_tt, th_tt) = decided_after(&run, (Face::T, Face::T));
        let count_x = ROW_LEFT - ROW_R;
        let counted_hh = format!(
            "{COUNTED_IN} {total} {TOSSES}: {AFTER_TWO_HEADS} {} {AGAINST} {}",
            thousands(ht_hh as u64),
            thousands(th_hh as u64)
        );
        let counted_hh = kit.text_left(stage, &counted_hh, 22.0, Palette::dim(), v(count_x, -1.55))?;
        fade_in_all(stage, &[counted_hh], 0.4)?;
        // and after two tails the square is its own mirror image
        line.wait_for(stage, "and after two tails")?;
        let state_tails = kit.text(
            stage,
            &format!("{THE_PAIR} {AFTER_TWO_TAILS}"),
            26.0,
            Palette::ink(),
            header,
        )?;
        let (at_left, at_right) = (stage.get_center(products[0]), stage.get_center(products[1]));
        // a shallow arc each: enough to pass one another, not enough to
        // reach the shares above
        let round = || AnimateArgs {
            run_time: Some(1.2),
            path_arc: Some(0.3),
            ..AnimateArgs::default()
        };
        let mut flip: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(square.view.x, after_t.first.f64(), 1.2)?)?,
            stage.prepare(
                products[0]
                    .animate()
                    .set_anim_args(round())?
                    .move_to(at_right, ORIGIN)?,
            )?,
            stage.prepare(products[1].animate().set_anim_args(round())?.move_to(at_left, ORIGIN)?)?,
        ];
        for &m in rows.iter().chain([&state]) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            flip.push(stage.prepare(a.rt(0.5))?);
        }
        let a = fade_in(stage.arena_mut(), state_tails, ORIGIN, 1.0)?;
        flip.push(stage.prepare(a.rt(1.2))?);
        stage.play_prepared(flip)?;
        let counted_tt = format!(
            "{AND} {AFTER_TWO_TAILS} {} {AGAINST} {}",
            thousands(ht_tt as u64),
            thousands(th_tt as u64)
        );
        let counted_tt = kit.text_left(stage, &counted_tt, 22.0, Palette::dim(), v(count_x, -1.9))?;
        fade_in_all(stage, &[counted_tt], 0.4)?;

        // 05_4: the cure: only the tosses that come straight after a head
        let line = say(&kit, stage, "05_4")?;
        let mut gone = vec![real, looks_fair, counted_hh, counted_tt];
        gone.extend(&products);
        gone.extend(&copies);
        fade_out_all(stage, &gone, 0.4)?;
        // every toss that follows a head drops to a second row: a line from
        // the head, and the toss under its own place
        let mut drops: Vec<Box<dyn Animation>> = Vec::new();
        for i in 1..ROW_N {
            if row[i - 1] != Face::H {
                continue;
            }
            let (from, to) = (at(i - 1, TOSS_Y), at(i, KEPT_Y));
            let link = polyline(vec![
                v(from[0] + 0.06, from[1] - ROW_R - 0.02),
                v(to[0] - 0.06, to[1] + ROW_R + 0.02),
            ])
            .style(line_style(Palette::dim(), 2.5))
            .build();
            let link = stage.arena_mut().add(link);
            let d = stage.arena_mut().add(disc(row[i], to, ROW_R));
            let a = fade_in(stage.arena_mut(), link, ORIGIN, 1.0)?;
            drops.push(stage.prepare(a)?);
            let a = fade_in(stage.arena_mut(), d, v(0.0, KEPT_Y - TOSS_Y), 1.0)?;
            drops.push(stage.prepare(a)?);
        }
        let drops = AnimationGroup::with_lag_ratio(stage.arena_mut(), drops, 0.12)?.with_run_time(2.6);
        stage.play(drops)?;
        let only = kit.text_left(
            stage,
            ONLY_AFTER_A_HEAD,
            22.0,
            Palette::dim(),
            v(ROW_LEFT - ROW_R, KEPT_Y - 0.36),
        )?;
        fade_in_all(stage, &[only], 0.4)?;
        // each of those has the same chance: the step slides level, and the
        // cuts cross on the diagonal again
        line.wait_for(stage, "Each of those has the same chance")?;
        let state_cure = kit.text(stage, AFTER_A_HEAD, 26.0, Palette::ink(), header)?;
        let mut level: Vec<Box<dyn Animation>> = Vec::new();
        for (t, to) in [
            (square.view.x, after_h.first),
            (square.view.y_h, after_h.after_h),
            (square.view.y_t, after_h.after_h),
        ] {
            level.push(stage.prepare(tween(t, to.f64(), 1.4)?)?);
        }
        let a = fade_out(stage.arena_mut(), state_tails, ORIGIN, 1.0)?;
        level.push(stage.prepare(a.rt(0.5))?);
        let a = fade_in(stage.arena_mut(), state_cure, ORIGIN, 1.0)?;
        level.push(stage.prepare(a.rt(1.4))?);
        stage.play_prepared(level)?;
        fade_out_all(stage, &[slash], 0.3)?;
        // a plain unfair coin, and the trick handles that
        line.wait_for(stage, "So they behave like a plain unfair coin")?;
        let fixed = von_neumann(&after_heads(&run));
        let fixed_heads = fixed.iter().filter(|&&f| f == Face::H).count();
        let cure = format!(
            "{total} {TOSSES}, {} {GAMES}: {} {AGAINST} {}",
            thousands(fixed.len() as u64),
            thousands(fixed_heads as u64),
            thousands((fixed.len() - fixed_heads) as u64)
        );
        let cure = kit.text_left(stage, &cure, 22.0, Palette::dim(), v(count_x, KEPT_Y - 0.8))?;
        let names = format!(
            "{SAMUELSON}, {}. {FAIR_TO_THE_END}, {}",
            cited::SAMUELSON_YEAR,
            cited::BLUM_FOCS_YEAR
        );
        let names = lower_third(&kit, stage, ONE_TOSS_MEMORY, &names)?;
        fade_in_all(stage, &[cure, names], 0.5)?;
        line.wait_for(stage, "and the trick handles that")?;
        play!(stage; indicate(stage.arena_mut(), equals, 1.5, Some(rgb(gold())))?.rt(0.9));
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
