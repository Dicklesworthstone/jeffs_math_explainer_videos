//! Chapter 05, where it breaks: the two things the trick leans on, broken
//! one at a time. A fair coin for the second toss (05_1) moves one cut and
//! not the other, so the blocks are no longer mirror images: forty percent
//! against ten (05_2). A coin with a memory looks fair by count (05_3), but
//! after a thrown-away pair of heads one pair needs one change and the other
//! two: a step in the second cut, and whoever saw the last pair knows who is
//! ahead (05_4). The cure is to keep only the tosses that follow a head, toss
//! by toss, and pair those (05_5). It opens and ends on the resting square.

use std::rc::Rc;

use super::draw::predict_pause;
use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{RowStrips, lower_third};
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, hex, play, thousands, tween, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::draw::{
    Block, SQUARE_AT, SQUARE_SIDE, SplitBar, Square, SquarePose, SquareView, alice, bob, caption, disc, disc_row,
    face_color, fade_all_but, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, gold, heads, lettered_disc,
    line_style, rgb, say, tails, thrown, tween_all, wait_until, warn,
};
use crate::facts::{after_heads, cited, decided_after, second_coin, sticky, sticky_run, two_coin_run};
use crate::model::{Face, PairLaw, Q, bent, von_neumann};

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
const REAL_COINS: &str = "real tossed coins do this a little:";
const ABOUT: &str = "about";
const LAND_AS_STARTED: &str = "in 100 land the way they started";
const DIACONIS: &str = "Diaconis, Holmes and Montgomery";
const THROWN_AWAY: &str = "thrown away";
const ONE_CHANGE: &str = "one change";
const TWO_CHANGES: &str = "two changes";
const AFTER_TWO_HEADS: &str = "after two heads";
const AFTER_TWO_TAILS: &str = "after two tails";
const THE_PAIR: &str = "the pair";
const AND: &str = "and";
const WHAT_BROKE: &str = "what broke:";
const STILL_HALF: &str =
    "the wins still split half and half overall; after a thrown-away pair of heads Alice wins about";
const GAMES_IN_100: &str = "games in 100 (model)";
const NOT_USED: &str = "not used: it follows a tail";
const AFTER_A_HEAD: &str = "after a head:";
const IN: &str = "in";
const EACH_FOLLOWS_A_HEAD: &str = "two tosses that each follow a head";
const PROGRAMMERS: &str = "for programmers: filter toss by toss, then pair what is left";
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
/// The coin with a memory sits over its row of tosses; while the column is
/// needed for the pair's two ways on, it waits this much higher.
const COIN_AT: Vec3 = [-0.2, 1.75, 0.0];
const COIN_UP: f64 = 1.45;
/// The row of the remembering coin's tosses, and the row of those kept.
const ROW_LEFT: f64 = -0.5;
const ROW_STEP: f64 = 0.27;
const ROW_R: f64 = 0.105;
const ROW_N: usize = 26;
const TOSS_Y: f64 = 0.75;
const KEPT_Y: f64 = 0.3;
/// The kept tosses, paired: from one pair to the next, and within a pair.
const PAIR_STEP: f64 = 0.8;
const PAIR_GAP: f64 = 0.27;

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

    /// The cut's chance of heads as a live percentage ("80%"): the coin that
    /// makes this toss, as the slider sets it. It appears at once. (A number
    /// with no decimals is cut short, not rounded, so it is rounded here.)
    fn number(&self, kit: &Kit, stage: &mut Stage<'_>, view: SquareView) -> fmn::Result<Mob> {
        let book = Rc::clone(&kit.book);
        let (cut, at) = (self.cut, self.number_at);
        let m = stage.always_redraw(move |s| {
            let pose = view.pose(s);
            let t = if cut == Cut::First { pose.x } else { pose.y_h };
            let d = DecimalNumber::new((100.0 * t).round())
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

/// What stands beside the landing games and follows them: the bar's share,
/// each player's count, and the caption that waits for the last game.
#[derive(Clone, Copy)]
struct Tally {
    /// The bar's tracker: Alice's share of the games landed so far.
    share: Mob,
    /// Trackers: Alice's wins and Bob's among the games landed so far.
    counts: [Mob; 2],
    /// On stage unseen until every game has landed; then it snaps in.
    caption: Mob,
}

/// Every game of a run as a small cell in its winner's colour (`alice_won`),
/// in the order they were played, [`PATTER_COLS`] to a row, `width` across
/// from `top_left`. The cells are on stage unseen (the background's colour)
/// and land by the clock, through whatever else is going on (a listen window
/// has no animation of its own): the first `delay` seconds from now, the
/// last `span` seconds later. The tally beside them counts what has landed,
/// never more. (All of the run lands, not its first forty games: Alice
/// happens to win thirty-eight of those.) Returns what to clear away: the
/// anchor, which carries the updater, and the grid.
fn land_games(
    stage: &mut Stage<'_>,
    alice_won: &[bool],
    (top_left, width): (Vec3, f64),
    tally: Tally,
    (delay, span): (f64, f64),
) -> fmn::Result<[Mob; 2]> {
    let n = alice_won.len();
    let cell = width / PATTER_COLS as f64;
    let rows = n.div_ceil(PATTER_COLS);
    let unseen = hex(BACKGROUND);
    let winners: Vec<Srgb> = alice_won.iter().map(|&a| if a { alice() } else { bob() }).collect();
    // Alice's wins among the first k games
    let mut wins = vec![0usize];
    for &a in alice_won {
        wins.push(wins[wins.len() - 1] + usize::from(a));
    }
    let colors = move |landed: usize| -> Vec<Vec<Srgb>> {
        (0..rows)
            .map(|r| {
                (0..PATTER_COLS)
                    .map(|c| match winners.get(r * PATTER_COLS + c) {
                        Some(&w) if r * PATTER_COLS + c < landed => w,
                        _ => unseen,
                    })
                    .collect()
            })
            .collect()
    };
    let grid = RowStrips::build(stage, top_left, (cell, cell), &colors(0))?;
    let cells = grid.group;
    stage.add_to_scene(cells)?;
    let anchor = stage.add(VMobject::new())?;
    let (mut clock, mut landed) = (0.0, 0);
    stage.add_dt_updater(
        anchor,
        move |s, _, dt| {
            clock += dt;
            let due = (((clock - delay) / span).clamp(0.0, 1.0) * n as f64).ceil() as usize;
            if due == landed {
                return;
            }
            let _ = grid.recolor(s, &colors(due));
            let hers = wins[due];
            let _ = s.set_tracker_value(tally.counts[0], hers as f64);
            let _ = s.set_tracker_value(tally.counts[1], (due - hers) as f64);
            let _ = s.set_tracker_value(tally.share, hers as f64 / due as f64);
            if due == n {
                s.set_opacity(tally.caption, 1.0, true);
            }
            landed = due;
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

/// Where the four discs of a way on stand, and how big they are.
const WAY_X: [f64; 4] = [0.2, 0.95, 2.2, 2.95];
const WAY_R: f64 = 0.3;
/// How small a way on gets as it shrinks into its product.
const WAY_SHRINK: f64 = 0.25;

/// One way on from a thrown-away pair of heads, drawn large across the
/// right-hand column: the pair before, the pair that follows, a bar wherever
/// the face changes, the pair's name and how many changes it took. Every
/// part is detached, so each can come in as it is said.
struct Way {
    /// The thrown-away pair: each disc, then its letter.
    before: Vec<Mob>,
    /// The pair that follows, likewise.
    after: Vec<Mob>,
    /// A bar for each change of face, in the order they happen.
    ticks: Vec<Mob>,
    /// The pair's colour and its name.
    name: [Mob; 2],
    /// How many changes, in words.
    count: Mob,
    /// The row's middle: what it shrinks about.
    mid: Vec3,
}

impl Way {
    fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        y: f64,
        pair: (Face, Face),
        (name, color): (&str, Srgb),
    ) -> fmn::Result<Self> {
        let faces = [Face::H, Face::H, pair.0, pair.1];
        let mut discs = Vec::new();
        for (f, x) in faces.into_iter().zip(WAY_X) {
            discs.push(lettered_disc(kit, stage, f, v(x, y), WAY_R, 34.0)?);
        }
        let mut ticks = Vec::new();
        for k in 1..3 {
            if faces[k] != faces[k + 1] {
                let x = (WAY_X[k] + WAY_X[k + 1]) / 2.0;
                let bar = polyline(vec![v(x, y - 0.4), v(x, y + 0.4)])
                    .style(line_style(Palette::ink(), 5.0))
                    .build();
                ticks.push(stage.arena_mut().add(bar));
            }
        }
        let label_x = WAY_X[3] + WAY_R + 0.5;
        let swatch = Rectangle::new()
            .width(0.3)
            .height(0.3)
            .style(flat(color, 1.0))
            .build()?
            .moved_to(v(label_x + 0.15, y + 0.22));
        let swatch = stage.arena_mut().add(swatch);
        let words = kit.text_left(stage, name, 26.0, Palette::ink(), v(label_x + 0.47, y + 0.22))?;
        let count = changes(ticks.len())?;
        let count = kit.text_left(stage, count, 26.0, Palette::ink(), v(label_x + 0.47, y - 0.22))?;
        Ok(Self {
            before: discs[..2].concat(),
            after: discs[2..].concat(),
            ticks,
            name: [swatch, words],
            count,
            mid: v((WAY_X[0] + WAY_X[3]) / 2.0, y),
        })
    }

    /// The row shrinks into the point `to` and is gone (its product comes in
    /// there), while its name goes up to head the column over `x`.
    fn shrink_to(&self, stage: &mut Stage<'_>, to: Vec3, x: f64, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        let mut leaving = self.before.clone();
        leaving.extend(&self.after);
        leaving.extend(&self.ticks);
        leaving.push(self.count);
        for m in leaving {
            let c = stage.get_center(m);
            let shift = v(
                to[0] - c[0] + WAY_SHRINK * (c[0] - self.mid[0]),
                to[1] - c[1] + WAY_SHRINK * (c[1] - self.mid[1]),
            );
            let a = fade_out(stage.arena_mut(), m, shift, WAY_SHRINK)?;
            out.push(stage.prepare(a.rt(rt))?);
        }
        let [swatch, words] = self.name;
        let name_at = v(x + 0.25, NAMES_Y);
        let swatch_at = v(name_at[0] - stage.get_width(words) / 2.0 - 0.32, NAMES_Y);
        out.push(stage.prepare(words.animate().set_anim_args(run(rt))?.move_to(name_at, ORIGIN)?)?);
        out.push(stage.prepare(swatch.animate().set_anim_args(run(rt))?.move_to(swatch_at, ORIGIN)?)?);
        Ok(out)
    }
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

        // the resting square, as chapter 04 leaves it and chapter 06 finds
        // it, on stage from the first frame so the picture stays through the
        // cut: one coin, Alice's block green, Bob's violet, the matching
        // pairs thrown away, heads and tails along two edges
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
        let cuts = [square.view.x, square.view.y_h, square.view.y_t];
        // this chapter's own instruments: the gold diagonal the two blocks
        // mirror in, and a dot where each column's cut meets the first cut
        let diagonal = square.diagonal(stage)?;
        let dots = crossing_dots(stage, square.view)?;
        let pose = square.pose(stage);
        let over_square = v(pose.left + pose.side / 2.0, pose.bottom + pose.side + 0.4);
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
        // the two shares a pair law gives, as the square shows them once it
        // has landed there (never while it is moving between two laws)
        let shares = |stage: &mut Stage<'_>, law: &PairLaw| -> fmn::Result<[Mob; 2]> {
            let share = |a, c| format!("{}%", law.prob(a, c).percent(0));
            Ok([
                kit.text(
                    stage,
                    &share(Face::H, Face::T),
                    READ_SIZE,
                    Palette::ink(),
                    v(LEFT_X, READ_Y),
                )?,
                kit.text(
                    stage,
                    &share(Face::T, Face::H),
                    READ_SIZE,
                    Palette::ink(),
                    v(RIGHT_X, READ_Y),
                )?,
            ])
        };
        let reads = shares(stage, &one_coin)?;
        let equals = equals_sign(stage, v(MID_X, READ_Y), 0.36);
        // the stroke that says the two shares are no longer equal
        let slash = polyline(vec![v(MID_X - 0.2, READ_Y - 0.27), v(MID_X + 0.2, READ_Y + 0.27)])
            .style(line_style(warn(), 5.0))
            .build();
        let slash = stage.arena_mut().add(slash);

        // 05_1: our unfair coin first, a fair coin second
        let line = say(&kit, stage, "05_1")?;
        let mut opening = vec![diagonal];
        opening.extend(dots);
        opening.extend(&legend);
        opening.extend(reads);
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
        // the second cut gets a slider of its own; its number waits until
        // the cut moves, since so far both cuts are our coin's
        line.wait_for(stage, "and a fair coin second")?;
        let second_coin_row = coin_row(&kit, stage, 0.0, (SECOND_TOSS, FAIR_COIN), fair, Look::Rimmed)?;
        let second_slider = Slider::build(&kit, stage, &square, Cut::Second, SECOND_TOSS)?;
        let mut ins = second_coin_row.clone();
        ins.extend(second_slider.mobs());
        fade_in_all(stage, &ins, 0.5)?;
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
        line.wait_for(stage, "One cut moved")?;
        let second_number = second_slider.number(&kit, stage, square.view)?;
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
        let landed = square.pose(stage);
        let (ht, th) = (landed.unit_rect(Block::HT), landed.unit_rect(Block::TH));
        // heads then tails, mirrored: across from its low edge to the square's
        // right side, up as far as it is wide
        let (from_u, to_w) = (ht.1, ht.2);
        if from_u >= th.0 || to_w <= th.3 {
            return Err(fail("the mirrored block does not hang over tails then heads".into()));
        }
        let over = [
            landed.point(from_u, 0.0),
            landed.point(th.0, 0.0),
            landed.point(th.0, th.3),
            landed.point(1.0, th.3),
            landed.point(1.0, to_w),
            landed.point(from_u, to_w),
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
        // the square has landed: its two shares, and the sign struck through
        line.wait_for(stage, "Forty percent against ten")?;
        let two_coins = PairLaw::two_coins(coin_chance, fair);
        let two_coin_reads = shares(stage, &two_coins)?;
        fade_in_all(stage, &[two_coin_reads[0], two_coin_reads[1], equals, slash], 0.4)?;
        // a thousand games of it land one by one, in the order they were
        // played; the bar and the two counts show what has landed so far,
        // and the caption waits for the last game
        line.wait_for(stage, "Alice wins four games in five")?;
        let games = two_coin_run();
        let alice_won: Vec<bool> = games
            .iter()
            .map(|g| g.last().is_some_and(|pair| pair.0 == Face::H))
            .collect();
        let n = games.len();
        let wins = alice_won.iter().filter(|&&a| a).count();
        let (bar_left, bar_w, bar_h) = (v(-0.6, -1.22), 6.6, 0.26);
        let bar = SplitBar::build(stage, bar_left, (bar_w, bar_h), (alice(), bob()), 0.5)?;
        let tally_y = bar_left[1] + 0.4;
        let alice_word = kit.text_left(stage, ALICE, 28.0, alice(), v(bar_left[0], tally_y))?;
        let alice_at = v(stage.get_right(alice_word)[0] + 0.2, tally_y);
        // Bob's name stands clear of his count at its widest
        let widest = kit.text(stage, &(n - wins).to_string(), 30.0, Palette::ink(), ORIGIN)?;
        let bob_at = v(bar_left[0] + bar_w, tally_y);
        let bob_word = kit.text(stage, BOB, 28.0, bob(), ORIGIN)?;
        let bob_word_at = v(bob_at[0] - stage.get_width(widest) - 0.2, tally_y);
        stage.move_to(bob_word, bob_word_at, RIGHT);
        let counted = format!("{COUNTED_IN} {} {GAMES}: {wins} {FOR_ALICE}", thousands(n as u64));
        let games_caption = caption(&kit, stage, &counted)?;
        stage.set_opacity(games_caption, 0.0, true);
        stage.add_to_scene(games_caption)?;
        let mut tally = bar.mobs().to_vec();
        tally.extend([alice_word, bob_word]);
        fade_in_all(stage, &tally, 0.4)?;
        let counts = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let ink = Palette::ink();
        tally.push(kit.live_number(stage, counts[0], 0, false, 30.0, ink, alice_at, LEFT)?);
        tally.push(kit.live_number(stage, counts[1], 0, false, 30.0, ink, bob_at, RIGHT)?);
        tally.push(games_caption);
        let under_bar = v(bar_left[0], bar_left[1] - bar_h / 2.0 - 0.15);
        let beside = Tally {
            share: bar.share,
            counts,
            caption: games_caption,
        };
        let window_end = line.end() + explainer_kit::narration::BREATH + 3.0;
        let span = (window_end - stage.scene().time().to_f64() - 0.8).clamp(2.2, 5.4);
        tally.extend(land_games(stage, &alice_won, (under_bar, bar_w), beside, (0.1, span))?);
        kit.listen(stage, 3.0, "05_where_it_breaks:05_2")?;

        // 05_3: a coin with a memory. One thing at a time: the coin, its
        // tosses, what a long run of them counts
        let line = say(&kit, stage, "05_3")?;
        let memory = sticky();
        if memory.long_run_heads() != Some(Q::HALF) {
            return Err(fail("the coin with a memory is not half heads in the long run".into()));
        }
        let run_of_tosses = sticky_run();
        if run_of_tosses.len() < ROW_N {
            return Err(fail(format!(
                "the run has {} tosses, the row {ROW_N}",
                run_of_tosses.len()
            )));
        }
        let mut gone = first_coin;
        gone.extend(second_coin_row);
        gone.extend(first_slider.mobs());
        gone.extend(second_slider.mobs());
        gone.extend([first_number, second_number, ghost, overhang, equals, slash]);
        gone.extend(two_coin_reads);
        gone.extend(&tally);
        gone.extend(&legend);
        fade_out_all(stage, &gone, 0.4)?;
        // one coin again, and a different-looking one beside it; real coins
        // have a little of this, for as long as the line lasts
        let mut the_coin = coin(&kit, stage, COIN_AT, Look::Remembering)?;
        let tag_x = COIN_AT[0] + 0.6;
        let tag_a = format!("{MODEL} {HALF_HEADS}");
        let tag_b = format!("{REPEATS} {}% {OF_THE_TIME}", memory.after_h.percent(0));
        the_coin.push(kit.text_left(stage, &tag_a, 24.0, Palette::dim(), v(tag_x, COIN_AT[1] + 0.17))?);
        the_coin.push(kit.text_left(stage, &tag_b, 24.0, Palette::dim(), v(tag_x, COIN_AT[1] - 0.17))?);
        let real = format!(
            "{ABOUT} {} {LAND_AS_STARTED} ({DIACONIS}, {})",
            cited::SAME_SIDE_PERCENT,
            cited::DIACONIS_YEAR
        );
        let real = lower_third(&kit, stage, REAL_COINS, &real)?;
        let mut arrive: Vec<Box<dyn Animation>> = Vec::new();
        for t in cuts {
            arrive.push(stage.prepare(tween(t, p, 0.8)?)?);
        }
        for &m in the_coin.iter().chain([&real]) {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            arrive.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(arrive)?;
        // its tosses: they come in runs of one colour
        let row = &run_of_tosses[..ROW_N];
        let at = |i: usize, y: f64| v(ROW_LEFT + ROW_STEP * i as f64, y);
        let tosses = disc_row(stage, row, at(0, TOSS_Y), ROW_R, ROW_STEP);
        fade_in_lagged(stage, &tosses, 1.4, 0.08)?;
        // and about half of a long run are heads
        line.wait_for(stage, "It looks fair")?;
        let run_heads = run_of_tosses.iter().filter(|&&f| f == Face::H).count();
        let share = Q::new(run_heads as i128, run_of_tosses.len() as i128).percent(1);
        let total = thousands(run_of_tosses.len() as u64);
        let count_x = ROW_LEFT - ROW_R;
        let looks_fair = format!("{COUNTED_IN} {total} {TOSSES}: {share}% {HEADS}");
        let looks_fair = kit.text_left(stage, &looks_fair, 24.0, Palette::dim(), v(count_x, TOSS_Y - 0.55))?;
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
        // the note on real coins is gone before the next line
        wait_until(stage, line.end())?;
        fade_out_all(stage, &[real], 0.4)?;

        // 05_4: a pair of heads was just thrown away. First the two ways on,
        // large and alone; then the square they make
        let line = say(&kit, stage, "05_4")?;
        let after_h = memory.pair_after(Face::H);
        let after_t = memory.pair_after(Face::T);
        let ways = [
            (
                (Face::H, Face::T),
                (HEADS_THEN_TAILS, alice()),
                (after_h.first, after_h.after_h.other()),
            ),
            (
                (Face::T, Face::H),
                (TAILS_THEN_HEADS, bob()),
                (after_h.first.other(), after_h.after_t),
            ),
        ];
        let one = Way::build(&kit, stage, 1.75, ways[0].0, ways[0].1)?;
        let two = Way::build(&kit, stage, 0.35, ways[1].0, ways[1].1)?;
        // the row of tosses makes room, and the coin waits above
        let mut clear: Vec<Box<dyn Animation>> = Vec::new();
        for &m in tosses.iter().chain(&copies).chain([&looks_fair]) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            clear.push(stage.prepare(a.rt(0.4))?);
        }
        for &m in &the_coin {
            clear.push(stage.prepare(m.animate().set_anim_args(run(0.6))?.shift(v(0.0, COIN_UP))?)?);
        }
        stage.play_prepared(clear)?;
        // the pair of heads lands, and is thrown away
        fade_in_all(stage, &one.before, 0.4)?;
        line.wait_for(stage, "was just thrown away")?;
        let pair_mid = (WAY_X[0] + WAY_X[1]) / 2.0;
        let thrown_words = kit.text(stage, THROWN_AWAY, 22.0, Palette::dim(), v(pair_mid, 1.75 - 0.52))?;
        play!(stage;
            one.before[0].animate().set_anim_args(run(0.4))?.set_fill(None, Some(0.4))?,
            one.before[2].animate().set_anim_args(run(0.4))?.set_fill(None, Some(0.4))?,
            fade_in(stage.arena_mut(), thrown_words, ORIGIN, 1.0)?.rt(0.4),
        );
        // heads then tails: one change, ticking as it is said
        line.wait_for(stage, "Now heads then tails")?;
        let mut ins = one.after.clone();
        ins.extend(one.name);
        fade_in_all(stage, &ins, 0.5)?;
        line.wait_for(stage, "needs one change")?;
        play!(stage;
            fade_in(stage.arena_mut(), one.ticks[0], ORIGIN, 1.8)?.rt(0.3),
            fade_in(stage.arena_mut(), one.count, ORIGIN, 1.0)?.rt(0.3),
        );
        // tails then heads: two
        line.wait_for(stage, "and tails then heads")?;
        for &m in [two.before[0], two.before[2]].iter() {
            stage.set_fill(m, None, Some(0.4), None, false);
        }
        let mut ins = two.before.clone();
        ins.extend(&two.after);
        ins.extend(two.name);
        fade_in_all(stage, &ins, 0.5)?;
        line.wait_for(stage, "needs two")?;
        play!(stage; fade_in(stage.arena_mut(), two.ticks[0], ORIGIN, 1.8)?.rt(0.25));
        play!(stage;
            fade_in(stage.arena_mut(), two.ticks[1], ORIGIN, 1.8)?.rt(0.25),
            fade_in(stage.arena_mut(), two.count, ORIGIN, 1.0)?.rt(0.25),
        );
        // so heads then tails is four times as likely: each way shrinks to
        // where its product will stand, under its name, while the square
        // takes its shape for the pair after two heads: the first toss still
        // cuts at four fifths, but the second cut has a step in it. Products,
        // shares and sign come once it has landed
        line.wait_for(stage, "So heads then tails is four times as likely")?;
        let product = |stage: &mut Stage<'_>, (a, b): (Q, Q), x: f64| {
            let src = format!(r"{} \times {}", a.exact(), b.exact());
            kit.tex(stage, &src, 34.0, Palette::ink(), &[], v(x, PRODUCT_Y))
        };
        let mut shrink = one.shrink_to(stage, v(LEFT_X, PRODUCT_Y), LEFT_X, 0.8)?;
        shrink.extend(two.shrink_to(stage, v(RIGHT_X, PRODUCT_Y), RIGHT_X, 0.8)?);
        let a = fade_out(stage.arena_mut(), thrown_words, ORIGIN, 1.0)?;
        shrink.push(stage.prepare(a.rt(0.4))?);
        for (t, to) in [
            (square.view.x, after_h.first),
            (square.view.y_h, after_h.after_h),
            (square.view.y_t, after_h.after_t),
        ] {
            shrink.push(stage.prepare(tween(t, to.f64(), 1.0)?)?);
        }
        stage.play_prepared(shrink)?;
        let names = [one.name[0], one.name[1], two.name[0], two.name[1]];
        let state = kit.text(
            stage,
            &format!("{THE_PAIR} {AFTER_TWO_HEADS}"),
            26.0,
            Palette::ink(),
            over_square,
        )?;
        let step_reads = shares(stage, &after_h)?;
        let step_products = [product(stage, ways[0].2, LEFT_X)?, product(stage, ways[1].2, RIGHT_X)?];
        let mut labels = vec![state, equals, slash];
        labels.extend(step_reads);
        labels.extend(step_products);
        fade_in_all(stage, &labels, 0.4)?;
        // what a long run counted, only now that it has been said
        let (ht_hh, th_hh) = decided_after(&run_of_tosses, (Face::H, Face::H));
        let (ht_tt, th_tt) = decided_after(&run_of_tosses, (Face::T, Face::T));
        let counted_hh = format!(
            "{COUNTED_IN} {total} {TOSSES}: {AFTER_TWO_HEADS} {} {AGAINST} {}",
            thousands(ht_hh as u64),
            thousands(th_hh as u64)
        );
        let counted_hh = kit.text_left(stage, &counted_hh, 22.0, Palette::dim(), v(count_x, 0.35))?;
        fade_in_all(stage, &[counted_hh], 0.4)?;
        // after two tails the same two products change sides, and the square
        // is its own mirror image: blank while it moves, relabelled when it
        // lands
        let mirrored = [
            (after_t.first, after_t.after_h.other()),
            (after_t.first.other(), after_t.after_t),
        ];
        if mirrored != [ways[1].2, ways[0].2] {
            return Err(fail(
                "after two tails the two products are not the same two, swapped".into(),
            ));
        }
        line.wait_for(stage, "and after two tails")?;
        fade_out_all(stage, &labels, 0.25)?;
        play!(stage; tween(square.view.x, after_t.first.f64(), 1.0)?);
        let state_tails = kit.text(
            stage,
            &format!("{THE_PAIR} {AFTER_TWO_TAILS}"),
            26.0,
            Palette::ink(),
            over_square,
        )?;
        let mirror_reads = shares(stage, &after_t)?;
        let mirror_products = [
            product(stage, mirrored[0], LEFT_X)?,
            product(stage, mirrored[1], RIGHT_X)?,
        ];
        let mut labels = vec![state_tails, equals, slash];
        labels.extend(mirror_reads);
        labels.extend(mirror_products);
        fade_in_all(stage, &labels, 0.4)?;
        let counted_tt = format!(
            "{AND} {AFTER_TWO_TAILS} {} {AGAINST} {}",
            thousands(ht_tt as u64),
            thousands(th_tt as u64)
        );
        let counted_tt = kit.text_left(stage, &counted_tt, 22.0, Palette::dim(), v(count_x, 0.0))?;
        fade_in_all(stage, &[counted_tt], 0.4)?;
        // in what sense it broke: the wins still split evenly, but whoever
        // saw the last pair knows who is ahead
        line.wait_for(stage, "Whoever saw the last pair")?;
        let stuck = || fail("the coin with a memory never decides".into());
        if memory.plain_rule_long_run() != Some(Q::HALF) {
            return Err(fail("the plain rule's wins do not split evenly in the long run".into()));
        }
        let ahead = memory.plain_rule(Face::H).ok_or_else(stuck)?;
        let broke = format!("{STILL_HALF} {} {GAMES_IN_100}", ahead.percent(0));
        let broke = lower_third(&kit, stage, WHAT_BROKE, &broke)?;
        fade_in_all(stage, &[broke], 0.5)?;

        // 05_5: the cure: only the tosses that come straight after a head,
        // toss by toss, and then the trick on those
        let line = say(&kit, stage, "05_5")?;
        // the row of tosses comes back under its coin; the stepped square
        // keeps its shape but loses its labels
        let mut back: Vec<Box<dyn Animation>> = Vec::new();
        for &m in labels.iter().chain(&names).chain(&[counted_hh, counted_tt]) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            back.push(stage.prepare(a.rt(0.4))?);
        }
        for &m in &the_coin {
            back.push(stage.prepare(m.animate().set_anim_args(run(0.6))?.shift(v(0.0, -COIN_UP))?)?);
        }
        for &m in &tosses {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            back.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(back)?;
        // toss by toss: one that follows a head drops to a second row, under
        // its own place, with a line from the head; one that follows a tail
        // fades where it is
        let mut sweep: Vec<Box<dyn Animation>> = Vec::new();
        let mut kept: Vec<(Face, Mob)> = Vec::new();
        let mut links = Vec::new();
        for i in 1..ROW_N {
            if row[i - 1] == Face::H {
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
                sweep.push(stage.prepare(a)?);
                let a = fade_in(stage.arena_mut(), d, v(0.0, KEPT_Y - TOSS_Y), 1.0)?;
                sweep.push(stage.prepare(a)?);
                links.push(link);
                kept.push((row[i], d));
            } else {
                sweep.push(stage.prepare(tosses[i].animate().set_fill(None, Some(0.28))?)?);
            }
        }
        let sweep = AnimationGroup::with_lag_ratio(stage.arena_mut(), sweep, 0.1)?.with_run_time(2.8);
        stage.play(sweep)?;
        // one of the faded ones says why
        let skipped = (1..ROW_N)
            .find(|&i| row[i - 1] == Face::T)
            .ok_or_else(|| fail("no toss of the row follows a tail".into()))?;
        let above = at(skipped, TOSS_Y + 0.42);
        let not_used = kit.text(stage, NOT_USED, 22.0, Palette::dim(), above)?;
        let leader = polyline(vec![v(above[0], above[1] - 0.14), v(above[0], TOSS_Y + ROW_R + 0.04)])
            .style(line_style(Palette::dim(), 2.5))
            .build();
        let leader = stage.arena_mut().add(leader);
        fade_in_all(stage, &[not_used, leader], 0.4)?;
        // each of those lands heads four times in five: the kept row is a
        // plain unfair coin, the step slides level, and the cuts cross on
        // the diagonal again. The names to search take the lower third
        line.wait_for(stage, "Each of those lands heads")?;
        let (often, out_of) = memory
            .after_h
            .exact()
            .split_once('/')
            .map(|(n, d)| (n.to_owned(), d.to_owned()))
            .ok_or_else(|| fail("the coin's chance after a head is not a fraction".into()))?;
        let kept_tag = format!("{AFTER_A_HEAD} {HEADS} {often} {IN} {out_of}");
        let kept_tag = kit.text_left(stage, &kept_tag, 24.0, Palette::dim(), v(count_x, KEPT_Y - 0.44))?;
        let names_to_search = format!(
            "{SAMUELSON}, {}. {FAIR_TO_THE_END}, {}",
            cited::SAMUELSON_YEAR,
            cited::BLUM_FOCS_YEAR
        );
        let names_to_search = lower_third(&kit, stage, ONE_TOSS_MEMORY, &names_to_search)?;
        play!(stage;
            fade_out(stage.arena_mut(), broke, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), names_to_search, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), kept_tag, ORIGIN, 1.0)?.rt(0.5),
        );
        tween_all(
            stage,
            &[
                (square.view.x, after_h.first.f64()),
                (square.view.y_t, after_h.after_h.f64()),
            ],
            1.2,
        )?;
        let state_cure = kit.text(stage, EACH_FOLLOWS_A_HEAD, 26.0, Palette::ink(), over_square)?;
        fade_in_all(stage, &[state_cure], 0.4)?;
        // and the trick handles that: the kept tosses close up into pairs; a
        // pair that differs is underlined in its winner's colour, as chapter
        // 02's games were, and a matching pair fades
        line.wait_for(stage, "So they behave like a plain unfair coin")?;
        let mut close: Vec<Box<dyn Animation>> = Vec::new();
        let mut seats = Vec::new();
        for (k, &(_, d)) in kept.iter().enumerate() {
            let seat = v(
                ROW_LEFT + PAIR_STEP * (k / 2) as f64 + PAIR_GAP * (k % 2) as f64,
                KEPT_Y,
            );
            close.push(stage.prepare(d.animate().set_anim_args(run(0.8))?.move_to(seat, ORIGIN)?)?);
            seats.push(seat);
        }
        for &l in &links {
            let a = fade_out(stage.arena_mut(), l, ORIGIN, 1.0)?;
            close.push(stage.prepare(a.rt(0.4))?);
        }
        stage.play_prepared(close)?;
        let mut marks: Vec<Box<dyn Animation>> = Vec::new();
        for (pair, seat) in kept.as_chunks::<2>().0.iter().zip(seats.as_chunks::<2>().0) {
            let [(first, a), (second, b)] = *pair;
            if first == second {
                for d in [a, b] {
                    marks.push(stage.prepare(d.animate().set_fill(None, Some(0.35))?)?);
                }
            } else {
                let winner = if first == Face::H { alice() } else { bob() };
                let y = KEPT_Y - ROW_R - 0.08;
                let under = polyline(vec![v(seat[0][0] - ROW_R, y), v(seat[1][0] + ROW_R, y)])
                    .style(line_style(winner, 4.5))
                    .build();
                let under = stage.arena_mut().add(under);
                let a = fade_in(stage.arena_mut(), under, ORIGIN, 1.0)?;
                marks.push(stage.prepare(a)?);
            }
        }
        let marks = AnimationGroup::with_lag_ratio(stage.arena_mut(), marks, 0.3)?.with_run_time(1.2);
        stage.play(marks)?;
        // what the whole run counted, and how to write it
        line.wait_for(stage, "and the trick handles that")?;
        let fixed = von_neumann(&after_heads(&run_of_tosses));
        let fixed_heads = fixed.iter().filter(|&&f| f == Face::H).count();
        let cure = format!(
            "{total} {TOSSES}, {} {GAMES}: {} {AGAINST} {}",
            thousands(fixed.len() as u64),
            thousands(fixed_heads as u64),
            thousands((fixed.len() - fixed_heads) as u64)
        );
        let cure = kit.text_left(stage, &cure, 22.0, Palette::dim(), v(count_x, KEPT_Y - 0.9))?;
        let how = kit.text_left(stage, PROGRAMMERS, 22.0, Palette::dim(), v(count_x, KEPT_Y - 1.25))?;
        fade_in_all(stage, &[cure, how], 0.5)?;
        // back to the resting square, which chapter 06 opens on: the cuts
        // exactly home, everything else leaves, and the square holds through
        // the cut
        wait_until(stage, line.end())?;
        for t in cuts {
            stage.set_tracker_value(t, p)?;
        }
        fade_all_but(stage, &resting, 0.6)?;
        kit.music_breath(stage, HOLD_S)?;
        kit.finish(stage)
    }
}
