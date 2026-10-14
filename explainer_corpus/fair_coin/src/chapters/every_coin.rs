//! Chapter 03, every coin: why the two blocks are exactly equal. Each is
//! four fifths one way and one fifth the other (03_1); flip one across the
//! diagonal and it lands on the other, the same two numbers multiplied in
//! the other order (03_2). Make the coin more unfair (03_3) and the blocks
//! are still equal, wherever the cuts slide (03_4); in letters, p times the
//! rest against the rest times p (03_5). So nobody needs to know the coin
//! (03_6), and the trick leans on two things only (03_7).

use std::rc::Rc;

use super::draw::predict_pause;
use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, hex, play, tween, v};
use fmn::animation::{grow_arrow, indicate, transform_from_copy};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::draw::{
    Block, SQUARE_AT, SQUARE_SIDE, Square, SquarePose, SquareView, alice, bob, fade_all_but, fade_in_all, fade_out_all,
    fail, flat, gold, heads, line_style, panel, pause, rgb, say, tails, thrown, wait_until,
};
use crate::facts::stops;
use crate::model::{Face, PairLaw, Q, bent};

pub struct EveryCoin {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const HEADS_THEN_TAILS: &str = "heads then tails";
const TAILS_THEN_HEADS: &str = "tails then heads";
const CHANCE_OF_HEADS: &str = "chance of heads";
const PROGRAMMERS: &str = "for programmers:";
const NOTHING_TO_MEASURE: &str = "nothing to measure first, and no stored constant to go stale";
const SAME_CHANCE: &str = "the same chance on both tosses";
const NO_EFFECT: &str = "one toss doesn't affect the next";
const INDEPENDENT: &str = "(independent)";
const NEEDS: &str = "what it needs, exactly:";
const NEEDS_TEXT: &str =
    "tosses that don't affect each other, with the same chance p of heads, for any p strictly between 0 and 1";
// the letters of 03_5 (TeX)
const P: &str = "p";
const REST: &str = "1-p";
const THE_REST: &str = "(1-p)";
const TIMES: &str = r"\times";

/// The right-hand column: heads then tails on the left, tails then heads on
/// the right, an equals sign between them.
const LEFT_X: f64 = 1.15;
const MID_X: f64 = 3.05;
const RIGHT_X: f64 = 4.95;
/// The blocks' names, their shares, and the equation, top to bottom.
const NAMES_Y: f64 = 2.45;
const READ_Y: f64 = 1.8;
const EQ_Y: f64 = 0.45;
const READ_SIZE: f64 = 40.0;
const EQ_SIZE: f64 = 46.0;
/// The space between two pieces of the equation.
const EQ_GAP: f64 = 0.2;
/// The slider under the square.
const SLIDER_Y: f64 = -2.45;
/// A label on a block's long side sits this far outside the square, past a
/// stretch of its side drawn this far out (as the edges' stretches are).
const LABEL_OUT: f64 = 0.42;
const RULE_OUT: f64 = 0.1;
/// A label on a block's short side sits this far along the block (0 to 1).
const LABEL_IN: f64 = 0.085;

/// A point's mirror image in the square's diagonal (lower left to upper
/// right).
fn mirrored(pose: &SquarePose, p: Vec3) -> Vec3 {
    let (u, w) = ((p[0] - pose.left) / pose.side, (p[1] - pose.bottom) / pose.side);
    pose.point(w, u)
}

/// A chance on a dark chip edged in its face's colour, so it reads on any
/// block: the chip's back and its words, detached.
fn chip(kit: &Kit, stage: &mut Stage<'_>, words: &str, color: Srgb, at: Vec3) -> fmn::Result<[Mob; 2]> {
    let text = kit.text(stage, words, 24.0, color, at)?;
    let (w, h) = (stage.get_width(text), stage.get_height(text));
    let back = Rectangle::new()
        .width(w + 0.22)
        .height(h + 0.2)
        .corner_radius(0.08)
        .style(Style::default().fill(hex("#0E1116"), 0.92).stroke(color, 2.0, 1.0))
        .build()?
        .moved_to(at);
    Ok([stage.arena_mut().add(back), text])
}

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

/// The share of a flip spent lifting the copy off its block.
const LIFT_SHARE: f64 = 0.15;
/// How far the copy lifts, away from the diagonal.
const LIFT_BY: f64 = 0.14;

/// One flip of the heads-then-tails block across the diagonal: a copy of the
/// block (the real one stays), and whatever rides with it, drawn from one
/// tracker. Every point travels straight to its own mirror image, so the
/// copy turns over on the diagonal and lands on tails then heads.
struct Flip {
    /// 0: on its own block. 1: on the mirror image.
    turn: Mob,
    /// Carries the updater.
    anchor: Mob,
    /// The copy first, then what rides with it.
    parts: Vec<Mob>,
}

impl Flip {
    /// The flip of block `b` as the square stands now. `lines` are shapes on
    /// stage that turn over with the copy (their points are mirrored);
    /// `riders` are labels that travel with it and stay upright. Copies of
    /// both are made; the originals stay where they are.
    fn build(
        stage: &mut Stage<'_>,
        square: &Square,
        b: Block,
        color: Srgb,
        lines: &[Mob],
        riders: &[Mob],
    ) -> fmn::Result<Self> {
        let pose = square.pose(stage);
        let points = |stage: &Stage<'_>, m: Mob| {
            stage
                .get_points(m)
                .ok_or_else(|| fail("a flipped shape has no points".into()))
        };
        let [ghost, mirror] = square.ghost_and_mirror(stage, b, color);
        let mut shapes = vec![(ghost, points(stage, ghost)?, points(stage, mirror)?)];
        for &l in lines {
            let c = stage.copy_family(l)?;
            let from = points(stage, c)?;
            let to = from.iter().map(|&q| mirrored(&pose, q)).collect();
            shapes.push((c, from, to));
        }
        let mut carried = Vec::new();
        for &r in riders {
            let c = stage.copy_family(r)?;
            let from = stage.get_center(c);
            carried.push((c, from, mirrored(&pose, from)));
        }
        let mut parts: Vec<Mob> = shapes.iter().map(|s| s.0).collect();
        parts.extend(carried.iter().map(|c| c.0));
        let turn = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let lift_by = LIFT_BY / 2f64.sqrt();
        let mut last: Option<f64> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = s.tracker_value(turn).unwrap_or(0.0).clamp(0.0, 1.0);
                if last == Some(t) {
                    return;
                }
                let lift = smooth((t / LIFT_SHARE).min(1.0));
                let e = smooth(((t - LIFT_SHARE) / (1.0 - LIFT_SHARE)).clamp(0.0, 1.0));
                // lifted up and to the left, away from the diagonal, until it lands
                let off = lift_by * lift * (1.0 - e);
                let at = |a: Vec3, b: Vec3| v(a[0] + (b[0] - a[0]) * e - off, a[1] + (b[1] - a[1]) * e + off);
                for (m, from, to) in &shapes {
                    let now: Vec<Vec3> = from.iter().zip(to).map(|(&a, &b)| at(a, b)).collect();
                    // halfway over, the copy is edge-on (a line on the diagonal): keep the last shape
                    if (e - 0.5).abs() >= 0.01 {
                        let _ = s.set_points(*m, &now);
                    }
                }
                for &(m, from, to) in &carried {
                    s.move_to(m, at(from, to), ORIGIN);
                }
                last = Some(t);
            },
            true,
        )?;
        Ok(Self { turn, anchor, parts })
    }

    fn all(&self) -> Vec<Mob> {
        let mut all = vec![self.anchor];
        all.extend(&self.parts);
        all
    }

    /// Bring the copy in, on its block.
    fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_all(stage, &self.all(), rt)
    }

    /// Lift it, turn it over and land it, `rt` seconds.
    fn play(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let a = self
            .turn
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                rate_func: Some(linear),
                ..AnimateArgs::default()
            })?
            .set_value(1.0)?;
        stage.play(a)?;
        Ok(())
    }

    /// The same flip played by the clock, through whatever comes next (a
    /// listen window has no animation of its own): it starts `delay` seconds
    /// from now and takes `rt` seconds.
    fn play_by_the_clock(&self, stage: &mut Stage<'_>, delay: f64, rt: f64) -> fmn::Result<()> {
        let turn = self.turn;
        let mut clock = 0.0;
        stage.add_dt_updater(
            self.anchor,
            move |s, _, dt| {
                clock += dt;
                let _ = s.set_tracker_value(turn, ((clock - delay) / rt).clamp(0.0, 1.0));
            },
            false,
        )?;
        Ok(())
    }

    fn hide(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_out_all(stage, &self.all(), rt)
    }
}

/// One side of the equation: three pieces (a chance, the times sign, a
/// chance) set side by side about a fixed middle.
struct Product {
    mid: f64,
    pieces: [Mob; 3],
    colors: [Srgb; 3],
    /// How far below the line's middle each piece's own middle sits.
    drops: [f64; 3],
}

impl Product {
    /// Where the pieces belong as they are now.
    fn spots(&self, stage: &Stage<'_>) -> [Vec3; 3] {
        let widths = self.pieces.map(|m| stage.get_width(m));
        let total = widths.iter().sum::<f64>() + 2.0 * EQ_GAP;
        let mut x = self.mid - total / 2.0;
        std::array::from_fn(|i| {
            let c = x + widths[i] / 2.0;
            x += widths[i] + EQ_GAP;
            v(c, EQ_Y + self.drops[i])
        })
    }

    /// Put the pieces in their places (before they are shown).
    fn set(&self, stage: &mut Stage<'_>) {
        for (m, at) in self.pieces.into_iter().zip(self.spots(stage)) {
            stage.move_to(m, at, ORIGIN);
        }
    }

    /// The two chances trade places, each going round the times sign.
    fn trade(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let (a, b) = (self.pieces[0], self.pieces[2]);
        let (at_a, at_b) = (stage.get_center(a), stage.get_center(b));
        // wide enough an arc to clear the times sign between them
        let round = || AnimateArgs {
            run_time: Some(rt),
            path_arc: Some(2.6),
            ..AnimateArgs::default()
        };
        play!(stage;
            a.animate().set_anim_args(round())?.move_to(at_b, ORIGIN)?,
            b.animate().set_anim_args(round())?.move_to(at_a, ORIGIN)?,
        );
        Ok(())
    }

    /// Piece `i` becomes `new` (detached): the old one fades out, the new one
    /// fades in where it belongs and the others close up around it.
    fn replace(
        &mut self,
        stage: &mut Stage<'_>,
        i: usize,
        (new, color, drop): (Mob, Srgb, f64),
        rt: f64,
    ) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let old = self.pieces[i];
        (self.pieces[i], self.colors[i], self.drops[i]) = (new, color, drop);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (k, (m, at)) in self.pieces.into_iter().zip(self.spots(stage)).enumerate() {
            if k == i {
                stage.move_to(m, at, ORIGIN);
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(rt))?);
            } else {
                let a = m
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(rt),
                        ..AnimateArgs::default()
                    })?
                    .move_to(at, ORIGIN)?;
                anims.push(stage.prepare(a)?);
            }
        }
        let a = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
        Ok(anims)
    }

    /// Swell every piece once, each in its own colour.
    fn pulse(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (m, c) in self.pieces.into_iter().zip(self.colors) {
            let a = indicate(stage.arena_mut(), m, 1.25, Some(rgb(c)))?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        stage.play_prepared(anims)?;
        Ok(())
    }
}

/// The slider under the square: its track runs the square's width, so the
/// knob sits under the first toss's cut.
struct Slider {
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
    fn build(kit: &Kit, stage: &mut Stage<'_>, square: &Square) -> fmn::Result<Self> {
        let (view, pose) = (square.view, square.pose(stage));
        let (x0, x1, y) = (pose.left, pose.left + pose.side, SLIDER_Y);
        let rule = line_style(Palette::dim(), 3.0);
        let track = polyline(vec![v(x0, y), v(x1, y)]).style(rule).build();
        let track = stage.arena_mut().add(track);
        // a tick at each end and in the middle, where the coin is fair
        let ticks = [0.0, Q::HALF.f64(), 1.0].map(|t| {
            let x = x0 + (x1 - x0) * t;
            polyline(vec![v(x, y - 0.09), v(x, y + 0.09)]).style(rule).build()
        });
        let ticks = stage.arena_mut().add(v_group(ticks));
        let knob = Circle::new().radius(0.11).style(flat(Palette::ink(), 1.0)).build();
        let knob = stage.arena_mut().add(knob);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last: Option<f64> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let x = view.pose(s).x;
                if last == Some(x) {
                    return;
                }
                s.move_to(knob, v(x0 + (x1 - x0) * x, y), ORIGIN);
                last = Some(x);
            },
            true,
        )?;
        let label = kit.text_left(stage, CHANCE_OF_HEADS, 24.0, Palette::dim(), v(x1 + 0.35, y))?;
        let number_at = v(stage.get_right(label)[0] + 0.2, y);
        Ok(Self {
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

    /// The slider's chance as a live percentage ("80%"). It appears at once.
    fn number(&self, kit: &Kit, stage: &mut Stage<'_>, view: SquareView) -> fmn::Result<Mob> {
        let book = Rc::clone(&kit.book);
        let at = self.number_at;
        let m = stage.always_redraw(move |s| {
            // rounded here: a number with no decimals is cut short, not rounded
            let d = DecimalNumber::new((100.0 * view.pose(s).x).round())
                .num_decimal_places(0)
                .unit("%")
                .font_size(26.0)
                .color(Palette::ink())
                .build(&book)
                .expect("bundled digits always lay out");
            let m = s.add(d.into_vmob());
            s.move_to(m, at, LEFT);
            m
        });
        stage.add_to_scene(m)?;
        Ok(m)
    }
}

/// A block's share of the square as a live percentage, its right edge at
/// `at`. As `Square::share_readout`, but rounded to the nearest percent: a
/// number with no decimals is cut short, not rounded, and four fifths of one
/// fifth is a hair under 16 in floating point. A block thinner than three
/// percent shows one decimal (99% heads). It appears at once.
fn share_readout(kit: &Kit, stage: &mut Stage<'_>, view: SquareView, b: Block, at: Vec3) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let value = 100.0 * view.pose(s).share(b);
        let places = usize::from(value < 3.0);
        let shown = if places == 0 { value.round() } else { value };
        let d = DecimalNumber::new(shown)
            .num_decimal_places(places)
            .unit("%")
            .font_size(READ_SIZE)
            .color(Palette::ink())
            .build(&book)
            .expect("bundled digits always lay out");
        let m = s.add(d.into_vmob());
        s.move_to(m, at, RIGHT);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A dot where the two cuts cross. While both tosses have the same chance it
/// rides the diagonal. Returns the anchor (which carries the updater) and
/// the dot.
fn crossing_dot(stage: &mut Stage<'_>, view: SquareView) -> fmn::Result<[Mob; 2]> {
    let dot = Circle::new()
        .radius(0.075)
        .style(Style::default().fill(Palette::ink(), 1.0).stroke(gold(), 3.0, 1.0))
        .build();
    let dot = stage.arena_mut().add(dot);
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last: Option<SquarePose> = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let pose = view.pose(s);
            if last == Some(pose) {
                return;
            }
            s.move_to(dot, pose.point(pose.x, pose.y_h), ORIGIN);
            last = Some(pose);
        },
        true,
    )?;
    Ok([anchor, dot])
}

/// A statement on a card: the card's back and its words, detached. `at` is
/// the middle of the card's left edge (`edge` is `LEFT`) or its centre.
fn statement(kit: &Kit, stage: &mut Stage<'_>, words: &str, at: Vec3, edge: Vec3) -> fmn::Result<[Mob; 2]> {
    let text = kit.text(stage, words, 26.0, Palette::ink(), at)?;
    let (w, h) = (stage.get_width(text), stage.get_height(text));
    let centre = if edge == LEFT {
        v(at[0] + w / 2.0 + 0.2, at[1])
    } else {
        at
    };
    stage.move_to(text, centre, ORIGIN);
    let back = Rectangle::new()
        .width(w + 0.4)
        .height(h + 0.36)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(centre);
    Ok([stage.arena_mut().add(back), text])
}

/// A thin outline round the stretch of an edge from `a` to `b`: a mark on
/// that stretch, not a new one. Detached.
fn capsule(stage: &mut Stage<'_>, a: Vec3, b: Vec3) -> fmn::Result<Mob> {
    let (w, h) = ((b[0] - a[0]).abs() + 0.22, (b[1] - a[1]).abs() + 0.22);
    let mark = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.1)
        .style(line_style(Palette::ink(), 2.5))
        .build()?
        .moved_to(v((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0));
    Ok(stage.arena_mut().add(mark))
}

/// `rt` seconds, for an `animate()` chain.
fn run(rt: f64) -> AnimateArgs {
    AnimateArgs {
        run_time: Some(rt),
        ..AnimateArgs::default()
    }
}

/// An arrow from a card to the thing it speaks of, detached.
fn pointer(stage: &mut Stage<'_>, from: Vec3, to: Vec3) -> fmn::Result<Mob> {
    let arrow = Arrow::new(from, to)
        .buff(0.0)
        .thickness(3.0)
        .style(
            Style::default()
                .fill(Palette::ink(), 1.0)
                .stroke(Palette::ink(), 3.0, 1.0),
        )
        .build()?;
    Ok(stage.arena_mut().add(arrow))
}

/// How far below the middle of the equation's line a lone p's own middle
/// sits: digits, slashes, signs and brackets are centred on that middle, a p
/// has a tail and nothing above the small letters' height.
fn letter_drop(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<f64> {
    let probe = kit.tex(stage, "(p)", EQ_SIZE, Palette::ink(), &[], ORIGIN)?;
    let parts = explainer_kit::children(stage, probe);
    let &[_, letter, _] = parts.as_slice() else {
        return Err(fail(format!(
            "a bracketed p is {} pieces, not a bracket, a letter and a bracket",
            parts.len()
        )));
    };
    Ok(stage.get_center(letter)[1] - stage.get_center(probe)[1])
}

impl SceneConstruct for EveryCoin {
    fn name(&self) -> &str {
        "03_every_coin"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coin = bent();
        let p = coin.f64();
        let law = PairLaw::same_coin(coin);
        // the two chances as the blocks and the equation write them
        let (wide, thin) = (coin.exact(), coin.other().exact());
        // the slider's stops: the next coin up, the two ends, and two to drift through
        let all = stops();
        let missing = || fail("the slider's stops do not straddle the coin".into());
        let more_unfair = all.iter().copied().filter(|&s| s > coin).min().ok_or_else(missing)?;
        let nearly_sure = all.iter().copied().max().ok_or_else(missing)?;
        let favours_tails = all.iter().copied().min().ok_or_else(missing)?;
        let between = |lo: Q, hi: Q| all.iter().copied().filter(move |&s| s > lo && s < hi);
        let drift_low = between(Q::HALF, coin).min().ok_or_else(missing)?;
        let drift_back = between(Q::HALF, coin).max().ok_or_else(missing)?;

        // the resting square, as chapter 02 leaves it and chapter 04 finds
        // it: Alice's block green above the diagonal, Bob's violet below, the
        // matching pairs thrown away, heads and tails along two edges. It is
        // on stage from the first frame, so the picture stays through the cut
        let square = Square::build(stage, SQUARE_AT, SQUARE_SIDE, p)?;
        square.paint(stage, Block::HT, alice(), 1.0);
        square.paint(stage, Block::TH, bob(), 1.0);
        square.paint(stage, Block::HH, thrown(), 1.0);
        square.paint(stage, Block::TT, thrown(), 1.0);
        let edges = square.edges(&kit, stage, HEADS, TAILS)?;
        let mut resting = square.mobs();
        resting.extend(edges.mobs());
        for &m in &resting {
            stage.add_to_scene(m)?;
        }
        let words_follow = follow_edges(stage, square.view, edges.words)?;
        stage.add_to_scene(words_follow)?;
        let pose = square.pose(stage);
        // beside it: each block's name over its share, an equals sign between
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
            let text = format!("{}%", law.prob(a, c).percent(0));
            reads.push(kit.text(stage, &text, READ_SIZE, Palette::ink(), v(x, READ_Y))?);
        }
        let equals = equals_sign(stage, v(MID_X, READ_Y), 0.36);

        // 03_1: each block's width and height
        let line = say(&kit, stage, "03_1")?;
        let mut opening = legend.clone();
        opening.extend(&reads);
        opening.push(equals);
        fade_in_all(stage, &opening, 0.7)?;
        // heads then tails is as wide as the first toss's heads and as tall
        // as the second toss's tails; tails then heads is its mirror image,
        // label for label
        let top = pose.point(0.0, 1.0)[1];
        let cut = pose.point(p, 0.0)[0];
        let rule_ends = [v(pose.left, top + RULE_OUT), v(cut, top + RULE_OUT)];
        let wide_at = v((pose.left + cut) / 2.0, top + LABEL_OUT);
        let tall_at = pose.point(LABEL_IN, (1.0 + p) / 2.0);
        let rule = |stage: &mut Stage<'_>, ends: [Vec3; 2]| {
            let stretch = polyline(ends.to_vec()).style(line_style(heads(), 7.0)).build();
            stage.arena_mut().add(stretch)
        };
        let green_rule = rule(stage, rule_ends);
        let violet_rule = rule(stage, rule_ends.map(|q| mirrored(&pose, q)));
        let green_wide = chip(&kit, stage, &wide, heads(), wide_at)?;
        let green_tall = chip(&kit, stage, &thin, tails(), tall_at)?;
        let violet_wide = chip(&kit, stage, &thin, tails(), mirrored(&pose, tall_at))?;
        let violet_tall = chip(&kit, stage, &wide, heads(), mirrored(&pose, wide_at))?;
        line.wait_for(stage, "four fifths wide")?;
        fade_in_all(stage, &[green_rule, green_wide[0], green_wide[1]], 0.5)?;
        line.wait_for(stage, "one fifth tall")?;
        fade_in_all(stage, &green_tall, 0.5)?;
        line.wait_for(stage, "one fifth wide")?;
        fade_in_all(stage, &violet_wide, 0.5)?;
        line.wait_for(stage, "four fifths tall")?;
        fade_in_all(stage, &[violet_rule, violet_tall[0], violet_tall[1]], 0.5)?;

        // 03_2: flip one across the diagonal and it lands on the other
        let line = say(&kit, stage, "03_2")?;
        let diagonal = square.diagonal(stage)?;
        play!(stage; show_creation(diagonal).rt(0.6));
        let riders = [green_wide[0], green_wide[1], green_tall[0], green_tall[1]];
        let flip = Flip::build(stage, &square, Block::HT, alice(), &[green_rule], &riders)?;
        flip.show(stage, 0.3)?;
        flip.play(stage, 2.0)?;
        // only then the two products, as one equation: each block's width
        // times its height
        line.wait_for(stage, "The same two numbers")?;
        flip.hide(stage, 0.3)?;
        let piece =
            |stage: &mut Stage<'_>, src: &str, color: Srgb| kit.tex(stage, src, EQ_SIZE, color, &[], v(0.0, EQ_Y));
        let mut left = Product {
            mid: LEFT_X,
            pieces: [
                piece(stage, &wide, heads())?,
                piece(stage, TIMES, Palette::ink())?,
                piece(stage, &thin, tails())?,
            ],
            colors: [heads(), Palette::ink(), tails()],
            drops: [0.0; 3],
        };
        let mut right = Product {
            mid: RIGHT_X,
            pieces: [
                piece(stage, &thin, tails())?,
                piece(stage, TIMES, Palette::ink())?,
                piece(stage, &wide, heads())?,
            ],
            colors: [tails(), Palette::ink(), heads()],
            drops: [0.0; 3],
        };
        left.set(stage);
        right.set(stage);
        let sign = equals_sign(stage, v(MID_X, EQ_Y), 0.42);
        let mut unders = Vec::new();
        for (x, color) in [(LEFT_X, alice()), (RIGHT_X, bob())] {
            let y = EQ_Y - 0.46;
            let under = polyline(vec![v(x - 1.3, y), v(x + 1.3, y)])
                .style(line_style(color, 4.0))
                .build();
            unders.push(stage.arena_mut().add(under));
        }
        let flights = [
            (green_wide[1], left.pieces[0]),
            (green_tall[1], left.pieces[2]),
            (violet_wide[1], right.pieces[0]),
            (violet_tall[1], right.pieces[2]),
        ];
        let mut settle: Vec<Box<dyn Animation>> = Vec::new();
        for (from, to) in flights {
            let a = transform_from_copy(stage.arena_mut(), from, to)?;
            settle.push(stage.prepare(a.rt(1.0))?);
        }
        for m in [left.pieces[1], right.pieces[1]] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            settle.push(stage.prepare(a.rt(1.0))?);
        }
        stage.play_prepared(settle)?;
        line.wait_for(stage, "multiplied in the other order")?;
        fade_in_all(stage, &[sign], 0.4)?;
        // multiplying doesn't care about order: the left side's two chances
        // trade places and it reads as the right side does; then back
        line.wait_for(stage, "And multiplying")?;
        left.trade(stage, 1.0)?;
        pause(stage, 0.5)?;
        left.trade(stage, 0.9)?;
        // each side takes its block's colour (once the chances have stopped
        // swinging round), and the copy is ready to flip once more, by
        // itself, in the listen window
        let again = Flip::build(stage, &square, Block::HT, alice(), &[green_rule], &riders)?;
        let mut ins = unders.clone();
        ins.extend(again.all());
        fade_in_all(stage, &ins, 0.3)?;
        kit.hold(stage)?;
        again.play_by_the_clock(stage, 0.15, 2.1)?;
        kit.listen(stage, 3.0, "03_every_coin:03_2")?;

        // 03_3: a more unfair coin; are the blocks still equal?
        let line = say(&kit, stage, "03_3")?;
        let mut equation = left.pieces.to_vec();
        equation.extend(right.pieces);
        equation.push(sign);
        equation.extend(&unders);
        // the numbers were this coin's: they leave before the cuts move
        let mut gone = again.all();
        gone.extend([green_rule, violet_rule]);
        gone.extend(green_wide.into_iter().chain(green_tall));
        gone.extend(violet_wide.into_iter().chain(violet_tall));
        gone.extend(&equation);
        gone.extend(&reads);
        gone.push(equals);
        fade_out_all(stage, &gone, 0.4)?;
        let asks = [
            kit.text(stage, "?", READ_SIZE, gold(), v(LEFT_X, READ_Y))?,
            kit.text(stage, "?", READ_SIZE, gold(), v(RIGHT_X, READ_Y))?,
        ];
        let slider = Slider::build(&kit, stage, &square)?;
        let dot = crossing_dot(stage, square.view)?;
        let mut ins = asks.to_vec();
        ins.extend(slider.mobs());
        ins.extend(dot);
        fade_in_all(stage, &ins, 0.5)?;
        let percent = slider.number(&kit, stage, square.view)?;
        line.wait_for(stage, "Heads nine times in ten")?;
        square.set_coin(stage, more_unfair.f64(), 1.2)?;
        let question = predict_pause(&kit, stage, v(MID_X, -0.9))?;

        // 03_4: yes; slide the cuts anywhere
        let line = say(&kit, stage, "03_4")?;
        fade_out_all(stage, &[asks[0], asks[1], question], 0.3)?;
        let mut live = Vec::new();
        for (x, b) in [(LEFT_X, Block::HT), (RIGHT_X, Block::TH)] {
            live.push(share_readout(&kit, stage, square.view, b, v(x + 0.45, READ_Y))?);
        }
        fade_in_all(stage, &[equals], 0.3)?;
        line.wait_for(stage, "Slide the cuts anywhere")?;
        square.set_coin(stage, nearly_sure.f64(), 1.0)?;
        // parked where the two blocks are hairlines, long enough to read
        // that their shares are still equal: this coin is nowhere near fair
        pause(stage, 1.6)?;
        square.set_coin(stage, Q::HALF.f64(), 1.0)?;
        square.set_coin(stage, favours_tails.f64(), 0.8)?;
        // still mirror images, with the coin favouring tails
        line.wait_for(stage, "The two blocks stay")?;
        let mirror = Flip::build(stage, &square, Block::HT, alice(), &[], &[])?;
        mirror.show(stage, 0.2)?;
        mirror.play(stage, 1.0)?;
        mirror.hide(stage, 0.2)?;
        // home, and the equation written for this coin comes back
        square.set_coin(stage, p, 0.9)?;
        fade_in_all(stage, &equation, 0.4)?;

        // 03_5: in letters
        let line = say(&kit, stage, "03_5")?;
        let drop = letter_drop(&kit, stage)?;
        let edge = |stage: &mut Stage<'_>, src: &str, color: Srgb| kit.tex(stage, src, 26.0, color, &[], ORIGIN);
        let letters = [
            edge(stage, P, heads())?,
            edge(stage, REST, tails())?,
            edge(stage, P, heads())?,
            edge(stage, REST, tails())?,
        ];
        let letters_follow = follow_edges(stage, square.view, letters)?;
        stage.add_to_scene(letters_follow)?;
        // the chance of heads is p: on the edges, and in the equation
        line.wait_for(stage, "chance of heads p")?;
        let letter = [piece(stage, P, heads())?, piece(stage, P, heads())?];
        let mut step = left.replace(stage, 0, (letter[0], heads(), drop), 0.9)?;
        step.extend(right.replace(stage, 2, (letter[1], heads(), drop), 0.9)?);
        for (old, new) in [(edges.words[0], letters[0]), (edges.words[2], letters[2])] {
            let a = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
            step.push(stage.prepare(a.rt(0.9))?);
            let a = fade_in(stage.arena_mut(), new, ORIGIN, 1.0)?;
            step.push(stage.prepare(a.rt(0.9))?);
        }
        stage.play_prepared(step)?;
        // tails gets the rest
        line.wait_for(stage, "so tails gets the rest")?;
        let mut step: Vec<Box<dyn Animation>> = Vec::new();
        for (old, new) in [(edges.words[1], letters[1]), (edges.words[3], letters[3])] {
            let a = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
            step.push(stage.prepare(a.rt(0.8))?);
            let a = fade_in(stage.arena_mut(), new, ORIGIN, 1.0)?;
            step.push(stage.prepare(a.rt(0.8))?);
        }
        stage.play_prepared(step)?;
        stage.clear_updaters(words_follow, true);
        line.wait_for(stage, "one minus p")?;
        let rest = [piece(stage, THE_REST, tails())?, piece(stage, THE_REST, tails())?];
        let mut step = left.replace(stage, 2, (rest[0], tails(), 0.0), 0.9)?;
        step.extend(right.replace(stage, 0, (rest[1], tails(), 0.0), 0.9)?);
        stage.play_prepared(step)?;
        // each side as it is said: its block is outlined, its rule under the
        // equation thickens and its product swells, and they stay marked
        // until the other side's turn; then the equals sign
        let outline = |stage: &mut Stage<'_>, b: Block| {
            let pose = square.pose(stage);
            let (u0, w0, u1, w1) = pose.unit_rect(b);
            let corners = [
                pose.point(u0, w0),
                pose.point(u1, w0),
                pose.point(u1, w1),
                pose.point(u0, w1),
            ];
            let edge = Polygon::new(corners).style(line_style(Palette::ink(), 4.5)).build();
            stage.arena_mut().add(edge)
        };
        let (thin_rule, thick_rule) = (4.0, 11.0);
        line.wait_for(stage, "Heads then tails is p times")?;
        let green_edge = outline(stage, Block::HT);
        play!(stage;
            fade_in(stage.arena_mut(), green_edge, ORIGIN, 1.0)?.rt(0.3),
            unders[0].animate().set_anim_args(run(0.3))?.set_stroke(None, Some(thick_rule), None)?,
        );
        left.pulse(stage, 0.9)?;
        line.wait_for(stage, "Tails then heads is the rest")?;
        let violet_edge = outline(stage, Block::TH);
        play!(stage;
            fade_out(stage.arena_mut(), green_edge, ORIGIN, 1.0)?.rt(0.3),
            unders[0].animate().set_anim_args(run(0.3))?.set_stroke(None, Some(thin_rule), None)?,
            fade_in(stage.arena_mut(), violet_edge, ORIGIN, 1.0)?.rt(0.3),
            unders[1].animate().set_anim_args(run(0.3))?.set_stroke(None, Some(thick_rule), None)?,
        );
        right.pulse(stage, 0.9)?;
        line.wait_for(stage, "Equal for every p")?;
        play!(stage;
            fade_out(stage.arena_mut(), violet_edge, ORIGIN, 1.0)?.rt(0.3),
            unders[1].animate().set_anim_args(run(0.3))?.set_stroke(None, Some(thin_rule), None)?,
            indicate(stage.arena_mut(), sign, 1.7, Some(rgb(gold())))?.rt(1.0),
        );

        // 03_6: nobody needs to know the coin
        say(&kit, stage, "03_6")?;
        let note = lower_third(&kit, stage, PROGRAMMERS, NOTHING_TO_MEASURE)?;
        // the slider's number and both shares are covered; the equals sign stays
        fade_out_all(stage, &[percent, live[0], live[1]], 0.3)?;
        let cover = kit.text_left(stage, "?", 26.0, gold(), slider.number_at)?;
        fade_in_all(stage, &[cover, asks[0], asks[1], note], 0.5)?;
        for stop in [drift_low, more_unfair, drift_back] {
            square.set_coin(stage, stop.f64(), 2.6)?;
        }

        // 03_7: the two things the trick leans on
        let line = say(&kit, stage, "03_7")?;
        let mut gone = vec![note, cover, asks[0], asks[1], equals];
        gone.extend(&legend);
        fade_out_all(stage, &gone, 0.4)?;
        slider.number(&kit, stage, square.view)?;
        // the coin comes home, and what the trick needs is written out for
        // the whole of the line
        let needs = lower_third(&kit, stage, NEEDS, NEEDS_TEXT)?;
        let cuts = [square.view.x, square.view.y_h, square.view.y_t];
        let mut home: Vec<Box<dyn Animation>> = Vec::new();
        for t in cuts {
            home.push(stage.prepare(tween(t, p, 0.8)?)?);
        }
        let a = fade_in(stage.arena_mut(), needs, ORIGIN, 1.0)?;
        home.push(stage.prepare(a.rt(0.6))?);
        stage.play_prepared(home)?;
        // exactly home: chapter 04 opens on this square
        for t in cuts {
            stage.set_tracker_value(t, p)?;
        }
        let pose = square.pose(stage);
        let crossing = pose.point(pose.x, pose.y_h);
        let (right_edge, top_edge) = (pose.left + pose.side, pose.bottom + pose.side);
        // the same chance on both tosses: the heads stretch of each edge is
        // the same length, so the two cuts cross on the gold line
        line.wait_for(stage, "The coin's chance is the same")?;
        let card_at = v(right_edge + 1.15, crossing[1] + 0.75);
        let same = statement(&kit, stage, SAME_CHANCE, card_at, LEFT)?;
        let ring = Circle::new()
            .radius(0.19)
            .arc_center(crossing)
            .style(line_style(Palette::ink(), 3.0))
            .build();
        let ring = stage.arena_mut().add(ring);
        let out = RULE_OUT;
        let first_toss = capsule(
            stage,
            v(pose.left, pose.bottom - out),
            v(crossing[0], pose.bottom - out),
        )?;
        let second_toss = capsule(stage, v(pose.left - out, pose.bottom), v(pose.left - out, crossing[1]))?;
        fade_in_all(stage, &[same[0], same[1], ring, first_toss, second_toss], 0.5)?;
        let from = v(card_at[0] - 0.1, card_at[1]);
        let reach = ((from[0] - crossing[0]).powi(2) + (from[1] - crossing[1]).powi(2)).sqrt();
        let short = 0.26 / reach;
        let tip = v(
            crossing[0] + (from[0] - crossing[0]) * short,
            crossing[1] + (from[1] - crossing[1]) * short,
        );
        let arrow = pointer(stage, from, tip)?;
        play!(stage; grow_arrow(stage.arena_mut(), arrow)?.rt(0.5));
        let blue = Some(rgb(heads()));
        play!(stage;
            indicate(stage.arena_mut(), letters[0], 1.6, blue)?.rt(0.8),
            indicate(stage.arena_mut(), letters[2], 1.6, blue)?.rt(0.8),
        );
        // one toss doesn't affect the next: the second cut runs straight
        // across both columns. Its card sits over the square and points at
        // the cut's middle, away from the first card's dot
        line.wait_for(stage, "And one toss")?;
        let middle = pose.left + pose.side / 2.0;
        let straight = statement(&kit, stage, NO_EFFECT, v(middle, top_edge + 0.65), ORIGIN)?;
        let across = polyline(vec![v(pose.left, crossing[1]), v(right_edge, crossing[1])])
            .style(line_style(Palette::ink(), 4.5))
            .build();
        let across = stage.arena_mut().add(across);
        fade_in_all(stage, &straight, 0.4)?;
        play!(stage; show_creation(across).rt(0.8));
        stage.scene_mut().bring_to_front(&[dot[0], dot[1], ring])?;
        let card_foot = stage.get_bottom(straight[0])[1];
        let arrow = pointer(stage, v(middle, card_foot - 0.05), v(middle, crossing[1] + 0.09))?;
        play!(stage; grow_arrow(stage.arena_mut(), arrow)?.rt(0.4));
        // the word for it, as it is said
        line.wait_for(stage, "which is what independent means")?;
        let named_at = v(stage.get_right(straight[0])[0] + 0.22, top_edge + 0.65);
        let named = kit.text_left(stage, INDEPENDENT, 24.0, Palette::dim(), named_at)?;
        fade_in_all(stage, &[named], 0.4)?;
        // back to the resting square, which chapter 04 opens on: heads and
        // tails return to the edges, everything else leaves, and the square
        // holds through the cut
        wait_until(stage, line.end() - 0.3)?;
        let mut words: Vec<Box<dyn Animation>> = Vec::new();
        for (word, letter) in edges.words.into_iter().zip(letters) {
            let a = fade_out(stage.arena_mut(), letter, ORIGIN, 1.0)?;
            words.push(stage.prepare(a.rt(0.3))?);
            let a = fade_in(stage.arena_mut(), word, ORIGIN, 1.0)?;
            words.push(stage.prepare(a.rt(0.3))?);
        }
        stage.play_prepared(words)?;
        fade_all_but(stage, &resting, 0.6)?;
        kit.music_breath(stage, HOLD_S)?;
        kit.finish(stage)
    }
}
