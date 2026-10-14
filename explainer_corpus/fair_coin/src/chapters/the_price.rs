//! Chapter 04, the price: it opens on the resting square chapter 03 leaves,
//! whose grey is what the trick pays (04_1); how many tosses a game takes,
//! asked over one slow game (04_2) and answered from the chance that a pair
//! decides (04_3); ten thousand seeded games as a histogram with their mean
//! (04_4); the same sum for a fair coin and for a coin at ninety-nine in a
//! hundred, read only where the slider stops (04_5); and the tail of the ten
//! thousand, one tick a game, the last at the longest game (04_6). It ends on
//! the resting square, which chapter 05 opens on.

use std::rc::Rc;

use super::draw::predict_pause;
use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::lower_third;
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, children, hex, play, thousands, v};
use fmn::prelude::*;

use super::draw::{
    Block, COLUMN_MID, SQUARE_AT, SQUARE_SIDE, Square, SquarePose, SquareView, alice, bob, column, fade_all_but,
    fade_in_all, fade_in_lagged, fade_out_all, fail, flat, group, line_style, pair_tile, panel, say, thrown,
    thrown_lit, wait_until,
};
use crate::facts::{price_run, stops};
use crate::model::{Face, PairLaw, Q, bent, expected_tosses, tosses_used, undecided_after};

pub struct ThePrice {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const THROWN_AWAY: &str = "thrown away";
const DECIDES: &str = "decides";
const MATCH: &str = "match";
const QUESTION: [&str; 2] = ["How many tosses does a game take,", "on average?"];
const ABOUT: &str = "about";
const PAIRS: &str = "pairs";
const TOSSES: &str = "tosses";
const EXACTLY: &str = "exactly";
const MODEL: &str = "(model)";
const DIVIDED_BY: &str = "÷";
const A_GAME: &str = "tosses a game,";
const ON_AVERAGE: &str = "on average (model)";
const COUNTED_IN: &str = "counted in";
const GAMES: &str = "games:";
const TOSSES_IN_A_GAME: &str = "tosses in a game";
const PROGRAMMERS: &str = "for programmers:";
const RETRY: &str = "rejection sampling, a retry loop with no upper bound. If you cap it, the cap returns no result, never a default winner";
const CHANCE_OF_HEADS: &str = "chance of heads";
const TAIL_WORDS: [&str; 3] = ["games of", "tosses or more,", "one tick each"];
const THE_LONGEST: &str = "the longest:";
const AFTER: &str = "after";
const STILL_GOING: &str = "still going (model)";
const PRECISELY: &str = "precisely:";
const NO_RULE: &str = "no rule that always names a winner within a fixed number of tosses is fair for every coin: it would have to hand an unbroken run of heads to somebody";

/// A pair is two tosses.
const TOSSES_A_PAIR: usize = 2;
/// Games of up to this many pairs get a bar; longer ones are the tail's
/// ticks (fact keys `E-S4.price.pairs_1_to_8` and `.pairs_9_plus`).
const BAR_PAIRS: usize = 8;
/// The model's mark on the tail: the chance a game is still going after
/// this many pairs (`E-F3.undecided_after.10_pairs_percent`).
const STILL_GOING_PAIRS: u32 = 10;
/// How long a lower-third stays up, at least (seconds).
const LOWER_THIRD_S: f64 = 8.1;

/// The legend above the square: the first row's baseline, the step down to
/// the second, and where its percentages end.
const LEGEND_BASE: f64 = SQUARE_AT[1] + SQUARE_SIDE + 0.72;
const LEGEND_STEP: f64 = 0.46;
const LEGEND_RIGHT: f64 = SQUARE_AT[0] + 3.3;
/// The sum under the square: its first line's baseline, the step to its
/// second, its size; then the slider under it.
const SUM_BASE: f64 = SQUARE_AT[1] - 0.78;
const SUM_STEP: f64 = 0.3;
const SUM_SIZE: f64 = 24.0;
const SLIDER_Y: f64 = SQUARE_AT[1] - 1.45;

/// The histogram's axis: where no tosses would sit, the width of one toss,
/// the axis's height and the height of the tallest bar.
const AXIS_LEFT: f64 = -0.6;
const TOSS_W: f64 = 0.118;
const AXIS_Y: f64 = -0.3;
const BAR_H: f64 = 3.0;
/// The tail's band under the axis numbers: its floor, how many ticks stand
/// side by side in a slot, the step from row to row and a tick's length.
const BAND_FLOOR: f64 = -2.45;
const TAIL_ROW: usize = 5;
const TAIL_ROW_STEP: f64 = 0.05;
const TICK_LEN: f64 = 0.03;

/// Where a game of `tosses` tosses sits along the axis.
fn toss_x(tosses: f64) -> f64 {
    AXIS_LEFT + TOSS_W * tosses
}

fn block_of(pair: (Face, Face)) -> Block {
    match pair {
        (Face::H, Face::H) => Block::HH,
        (Face::H, Face::T) => Block::HT,
        (Face::T, Face::H) => Block::TH,
        (Face::T, Face::T) => Block::TT,
    }
}

/// Put `text`'s left edge at `at` with its baseline on `at`'s height. The
/// first glyph (a digit or a letter that stands on the line) carries the
/// baseline, so words and numbers set as separate mobjects share one line
/// whatever descenders or brackets each has.
fn on_baseline(stage: &mut Stage<'_>, text: Mob, at: Vec3) {
    let first = children(stage, text).first().copied().unwrap_or(text);
    let dy = at[1] - stage.get_bottom(first)[1];
    let dx = at[0] - stage.get_left(text)[0];
    stage.shift(text, v(dx, dy));
}

/// A decimal as the screen prints it ("0.0198"), read back exactly.
fn parse_decimal(s: &str) -> Option<Q> {
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    let scale = 10i128.checked_pow(u32::try_from(frac.len()).ok()?)?;
    let digits: i128 = format!("{whole}{frac}").parse().ok()?;
    Some(Q::new(digits, scale))
}

/// A share as the legend prints it: whole percent, one decimal under 3%.
fn percent_text(share: Q) -> String {
    let places = if share < Q::new(3, 100) { 1 } else { 0 };
    format!("{}%", share.percent(places))
}

/// The sum under the square for a coin with chance `c` of heads, as fixed
/// text in two lines, centred under the square: "2 tosses ÷ 0.32 = 6.25
/// tosses a game," over "on average (model)". It is built whole at a stop of
/// the slider, so the printed divisor always gives the printed quotient (an
/// error otherwise); nothing in it counts. Detached.
fn sum_lines(kit: &Kit, stage: &mut Stage<'_>, c: Q) -> fmn::Result<[Mob; 2]> {
    let decides = PairLaw::same_coin(c).decides();
    let cost = expected_tosses(c).ok_or_else(|| fail("a coin that never decides has no price".into()))?;
    let divisor = decides.decimal(if decides < Q::new(1, 10) { 4 } else { 2 });
    let per_pair = Q::int(TOSSES_A_PAIR as i128);
    let printed = parse_decimal(&divisor).ok_or_else(|| fail(format!("{divisor} is not a decimal")))?;
    let quotient = cost.decimal(2);
    if (per_pair / printed).decimal(2) != quotient {
        return Err(fail(format!(
            "{TOSSES_A_PAIR} ÷ {divisor} does not print as {quotient}"
        )));
    }
    let first = format!("{TOSSES_A_PAIR} {TOSSES} {DIVIDED_BY} {divisor} = {quotient} {A_GAME}");
    let mid = SQUARE_AT[0] + SQUARE_SIDE / 2.0;
    let mut out = Vec::new();
    for (words, base) in [(first.as_str(), SUM_BASE), (ON_AVERAGE, SUM_BASE - SUM_STEP)] {
        let t = kit.text(stage, words, SUM_SIZE, Palette::ink(), ORIGIN)?;
        let w = stage.get_width(t);
        on_baseline(stage, t, v(mid - w / 2.0, base));
        out.push(t);
    }
    Ok([out[0], out[1]])
}

/// The legend's two percentages for a coin with chance `c` of heads (thrown
/// away, then decides), right-aligned on their rows. Detached.
fn legend_values(kit: &Kit, stage: &mut Stage<'_>, c: Q) -> fmn::Result<[Mob; 2]> {
    let decides = PairLaw::same_coin(c).decides();
    let mut out = Vec::new();
    for (i, share) in [decides.other(), decides].into_iter().enumerate() {
        let t = kit.text(stage, &percent_text(share), 28.0, Palette::ink(), ORIGIN)?;
        let w = stage.get_width(t);
        on_baseline(stage, t, v(LEGEND_RIGHT - w, LEGEND_BASE - LEGEND_STEP * i as f64));
        out.push(t);
    }
    Ok([out[0], out[1]])
}

/// Keep the four edge words (under the bottom edge's heads and tails, then
/// beside the left edge's) at the middle of their stretches while the cuts
/// move, where `Square::edges` first put them. Returns the anchor that
/// carries the updater: it must be on stage.
fn follow_edges(stage: &mut Stage<'_>, view: SquareView, words: [Mob; 4]) -> fmn::Result<Mob> {
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
            for (&m, at) in words.iter().zip(spots) {
                s.move_to(m, at, ORIGIN);
            }
            last = Some(pose);
        },
        true,
    )?;
    Ok(anchor)
}

/// A live percentage computed from the arena every frame, rounded to a whole
/// percent, its left end at `at`. On screen as soon as it is built; clear
/// its updaters before fading it (`fade_out_all` does).
fn live_percent(
    kit: &Kit,
    stage: &mut Stage<'_>,
    f: impl Fn(&fmn::mobject::Stage) -> f64 + 'static,
    size: f64,
    at: Vec3,
) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        // with no decimals a `DecimalNumber` cuts the fraction off: round first
        let d = DecimalNumber::new(f(s).round())
            .num_decimal_places(0)
            .unit("%")
            .font_size(size)
            .color(Palette::ink())
            .build(&book)
            .expect("bundled digits always lay out");
        let m = s.add(d.into_vmob());
        s.move_to(m, at, DL);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A white outline around block `b` as the square stands, detached: where a
/// pair of the slow game lands.
fn outline(stage: &mut Stage<'_>, pose: &SquarePose, b: Block) -> Mob {
    let (u0, w0, u1, w1) = pose.unit_rect(b);
    let quad = [
        pose.point(u0, w0),
        pose.point(u1, w0),
        pose.point(u1, w1),
        pose.point(u0, w1),
    ];
    stage
        .arena_mut()
        .add(Polygon::new(quad).style(line_style(Palette::ink(), 5.0)).build())
}

/// Swap what the square's coin reads: the old legend values and sum fade
/// out, the cuts move to `c` with nothing printed, and the new values snap
/// in once the square has landed. Returns the new readings.
fn read_at(kit: &Kit, stage: &mut Stage<'_>, square: &Square, old: &[Mob], c: Q, rt: f64) -> fmn::Result<Vec<Mob>> {
    fade_out_all(stage, old, 0.25)?;
    square.set_coin(stage, c.f64(), rt)?;
    let mut new = legend_values(kit, stage, c)?.to_vec();
    new.extend(sum_lines(kit, stage, c)?);
    fade_in_all(stage, &new, 0.3)?;
    Ok(new)
}

impl SceneConstruct for ThePrice {
    fn name(&self) -> &str {
        "04_the_price"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = bent();
        let never = || fail("a coin that never decides has no price".into());
        let cost = expected_tosses(p).ok_or_else(never)?;
        let per_pair = Q::int(TOSSES_A_PAIR as i128);
        // the slider's far end: the stop the sum is read at for a nearly sure coin
        let far = stops().into_iter().max().ok_or_else(never)?;
        let games = price_run();
        let ink = Palette::ink();

        // the resting square, as chapter 03 leaves it and chapter 05 finds
        // it: on stage from the first frame, so the picture stays through the
        // cut. Alice's block green, Bob's violet, the matching pairs thrown
        // away, heads and tails along two edges
        let square = Square::build(stage, SQUARE_AT, SQUARE_SIDE, p.f64())?;
        let paints = [
            (Block::HT, alice()),
            (Block::TH, bob()),
            (Block::HH, thrown()),
            (Block::TT, thrown()),
        ];
        for (b, c) in paints {
            square.paint(stage, b, c, 1.0);
        }
        let edges = square.edges(&kit, stage, HEADS, TAILS)?;
        let mut resting = square.mobs();
        resting.extend(edges.mobs());
        for &m in &resting {
            stage.add_to_scene(m)?;
        }
        let words_follow = follow_edges(stage, square.view, edges.words)?;
        stage.add_to_scene(words_follow)?;
        let view = square.view;

        // 04_1: the grey is what gets thrown away; the colour decides
        let line = say(&kit, stage, "04_1")?;
        let values = legend_values(&kit, stage, p)?;
        let legend = [
            (THROWN_AWAY, [thrown_lit(), thrown_lit()], "most pairs get thrown away"),
            (DECIDES, [alice(), bob()], "A pair decides"),
        ];
        for (i, (word, colors, phrase)) in legend.into_iter().enumerate() {
            line.wait_for(stage, phrase)?;
            let base = LEGEND_BASE - LEGEND_STEP * i as f64;
            let mut parts = Vec::new();
            for (k, c) in colors.into_iter().enumerate() {
                let swatch = Rectangle::new()
                    .width(0.3)
                    .height(0.3)
                    .style(flat(c, 1.0))
                    .build()?
                    .moved_to(v(SQUARE_AT[0] + 0.15 + 0.36 * k as f64, base + 0.1));
                parts.push(stage.arena_mut().add(swatch));
            }
            let label = kit.text(stage, word, 26.0, ink, ORIGIN)?;
            on_baseline(stage, label, v(SQUARE_AT[0] + 0.86, base));
            parts.extend([label, values[i]]);
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for &m in &parts {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.6))?);
            }
            if i == 0 {
                // the two grey blocks lift while they are the subject
                for b in [Block::HH, Block::TT] {
                    anims.push(stage.prepare(square.repaint(b, thrown_lit(), 0.6)?)?);
                }
            }
            stage.play_prepared(anims)?;
        }

        // 04_2: how many tosses does a game take? One game, slowly
        let line = say(&kit, stage, "04_2")?;
        let card_at = v(COLUMN_MID, 2.45);
        let q1 = kit.text(stage, QUESTION[0], 30.0, ink, v(card_at[0], card_at[1] + 0.24))?;
        let q2 = kit.text(stage, QUESTION[1], 30.0, ink, v(card_at[0], card_at[1] - 0.24))?;
        let w = stage.get_width(q1).max(stage.get_width(q2));
        let back = Rectangle::new()
            .width(w + 0.7)
            .height(1.3)
            .corner_radius(0.14)
            .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
            .build()?
            .moved_to(card_at);
        let back = stage.arena_mut().add(back);
        let mut asked = vec![back, q1, q2];
        fade_in_all(stage, &asked, 0.5)?;
        // a game of the run with as many pairs as a game takes on average, to the nearest pair
        let about_pairs: usize = (cost / per_pair)
            .decimal(0)
            .parse()
            .map_err(|_| fail("the average number of pairs does not round to a count".into()))?;
        let game = games
            .iter()
            .find(|g| g.len() == about_pairs)
            .cloned()
            .ok_or_else(|| fail(format!("run 4 has no game of {about_pairs} pairs")))?;
        let pose = square.pose(stage);
        let tiles_y = 0.95;
        let mut played: Vec<Mob> = Vec::new();
        for (i, &pair) in game.iter().enumerate() {
            wait_until(stage, line.start + 0.9 + 1.05 * i as f64)?;
            let x = COLUMN_MID + 2.0 * (i as f64 - (game.len() as f64 - 1.0) / 2.0);
            let parts = pair_tile(&kit, stage, pair, v(x, tiles_y), 0.85)?;
            let tile = group(stage, &parts)?;
            let (word, color) = match pair {
                (Face::H, Face::T) => (DECIDES, alice()),
                (Face::T, Face::H) => (DECIDES, bob()),
                _ => (MATCH, Palette::dim()),
            };
            let word = kit.text(stage, word, 24.0, color, v(x, tiles_y - 0.62))?;
            let ring = outline(stage, &pose, block_of(pair));
            play!(stage;
                fade_in(stage.arena_mut(), tile, v(0.0, -0.2), 1.0)?.rt(0.45),
                fade_in(stage.arena_mut(), word, ORIGIN, 1.0)?.rt(0.45),
                fade_in(stage.arena_mut(), ring, ORIGIN, 1.0)?.rt(0.45),
            );
            fade_out_all(stage, &[ring], 0.3)?;
            played.extend([tile, word]);
        }
        asked.push(predict_pause(&kit, stage, v(COLUMN_MID, -1.0))?);

        // 04_3: about one time in three, so about three pairs, about six
        // tosses; exactly, two tosses over the chance a pair decides. The
        // answer snaps in whole
        let line = say(&kit, stage, "04_3")?;
        fade_out_all(stage, &asked, 0.3)?;
        let mark = Rectangle::new()
            .width(3.75)
            .height(0.5)
            .corner_radius(0.1)
            .style(line_style(ink, 2.5))
            .build()?
            .moved_to(v(SQUARE_AT[0] + 1.6, LEGEND_BASE - LEGEND_STEP + 0.1));
        let mark = stage.arena_mut().add(mark);
        play!(stage; fade_in(stage.arena_mut(), mark, ORIGIN, 1.0)?.rt(0.4));
        line.wait_for(stage, "so about three pairs")?;
        let pairs_note = kit.text(
            stage,
            &format!("{ABOUT} {about_pairs} {PAIRS}"),
            30.0,
            ink,
            v(COLUMN_MID, -0.35),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), mark, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), pairs_note, ORIGIN, 1.0)?.rt(0.4),
        );
        line.wait_for(stage, "That's about six tosses")?;
        let tosses_note = kit.text(
            stage,
            &format!("{ABOUT} {} {TOSSES}", cost.decimal(0)),
            30.0,
            ink,
            v(COLUMN_MID, -0.95),
        )?;
        fade_in_all(stage, &[tosses_note], 0.4)?;
        line.wait_for(stage, "Exactly")?;
        let exactly = kit.text(
            stage,
            &format!("{EXACTLY} {} {MODEL}", cost.decimal(2)),
            40.0,
            ink,
            v(COLUMN_MID, -1.72),
        )?;
        let mut readings = sum_lines(&kit, stage, p)?.to_vec();
        let mut ins = vec![exactly];
        ins.extend(&readings);
        fade_in_all(stage, &ins, 0.4)?;
        readings.extend(values);
        played.extend([pairs_note, tosses_note, exactly]);
        // the programmers' note, once the answer is in, for at least eight seconds
        line.wait_for(stage, "six and a quarter")?;
        let retry = lower_third(&kit, stage, PROGRAMMERS, RETRY)?;
        play!(stage; fade_in(stage.arena_mut(), retry, ORIGIN, 1.0)?.rt(0.5));
        let retry_at = stage.scene().time().to_f64();

        // 04_4: ten thousand games, one bar for each number of tosses
        let line = say(&kit, stage, "04_4")?;
        fade_out_all(stage, &played, 0.4)?;
        let longest = games.iter().map(Vec::len).max().unwrap_or(0) * TOSSES_A_PAIR;
        let axis = polyline(vec![v(toss_x(0.0), AXIS_Y), v(toss_x(longest as f64 + 1.5), AXIS_Y)])
            .style(line_style(Palette::dim(), 2.5))
            .build();
        let mut plot = vec![stage.arena_mut().add(axis)];
        // numbers under the axis: the first bar's, then every ten
        for t in std::iter::once(TOSSES_A_PAIR).chain((10..=longest).step_by(10)) {
            plot.push(kit.text(
                stage,
                &t.to_string(),
                20.0,
                Palette::dim(),
                v(toss_x(t as f64), AXIS_Y - 0.32),
            )?);
        }
        // the axis's name under its first numbers, clear of the tail's band
        let named = kit.text(stage, TOSSES_IN_A_GAME, 22.0, Palette::dim(), ORIGIN)?;
        stage.move_to(named, v(toss_x(0.0), AXIS_Y - 0.72), LEFT);
        plot.push(named);
        fade_in_all(stage, &plot, 0.5)?;
        let counts: Vec<usize> = (1..=BAR_PAIRS)
            .map(|k| games.iter().filter(|g| g.len() == k).count())
            .collect();
        let tallest = counts.iter().copied().max().unwrap_or(1).max(1) as f64;
        let pitch = TOSSES_A_PAIR as f64 * TOSS_W;
        let mut grows: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &c) in counts.iter().enumerate() {
            let t = ((i + 1) * TOSSES_A_PAIR) as f64;
            let bar = column(
                v(toss_x(t), AXIS_Y),
                0.8 * pitch,
                BAR_H * c as f64 / tallest,
                Palette::dim(),
            );
            let bar = stage.arena_mut().add(bar);
            let a = fmn::animation::grow_from_edge(stage.arena_mut(), bar, DOWN, None)?;
            grows.push(stage.prepare(a)?);
        }
        let grow = AnimationGroup::with_lag_ratio(stage.arena_mut(), grows, 0.3)?.with_run_time(1.4);
        stage.play(grow)?;
        line.wait_for(stage, "They average")?;
        let used = tosses_used(&games);
        let mean_x = toss_x(used as f64 / games.len() as f64);
        let mark_top = AXIS_Y + BAR_H + 0.4;
        let marker = polyline(vec![v(mean_x, AXIS_Y), v(mean_x, mark_top)])
            .style(line_style(ink, 3.0))
            .build();
        let marker = stage.arena_mut().add(marker);
        let counted = format!(
            "{COUNTED_IN} {} {GAMES} {}",
            thousands(games.len() as u64),
            Q::new(used as i128, games.len() as i128).decimal(2)
        );
        let counted = kit.text(stage, &counted, 26.0, Palette::dim(), ORIGIN)?;
        on_baseline(stage, counted, v(mean_x + 0.2, mark_top - 0.24));
        fade_in_all(stage, &[marker, counted], 0.5)?;
        // the programmers' note stays long enough to be read
        wait_until(stage, retry_at + LOWER_THIRD_S)?;

        // 04_5: the same sum for a fair coin, and for ninety-nine in a
        // hundred; the readings go blank while the slider moves
        let line = say(&kit, stage, "04_5")?;
        let track = polyline(vec![v(SQUARE_AT[0], SLIDER_Y), v(SQUARE_AT[0] + SQUARE_SIDE, SLIDER_Y)])
            .style(line_style(Palette::dim(), 3.0))
            .build();
        let track = stage.arena_mut().add(track);
        let knob = Dot::new()
            .point(v(pose.point(pose.x, 0.0)[0], SLIDER_Y))
            .radius(0.1)
            .style(flat(ink, 1.0))
            .build();
        let knob = stage.arena_mut().add(knob);
        // the knob rides under the first cut
        let rider = stage.arena_mut().add(VMobject::new());
        let mut last = None;
        stage.add_updater(
            rider,
            move |s, _| {
                let pose = view.pose(s);
                if last == Some(pose.x) {
                    return;
                }
                s.move_to(knob, v(pose.point(pose.x, 0.0)[0], SLIDER_Y), ORIGIN);
                last = Some(pose.x);
            },
            true,
        )?;
        let chance = kit.text(stage, CHANCE_OF_HEADS, 24.0, Palette::dim(), ORIGIN)?;
        on_baseline(stage, chance, v(SQUARE_AT[0], SLIDER_Y - 0.48));
        let chance_right = stage.get_right(chance)[0];
        // the counted games are for our coin only: they stand back while the coin changes
        let scrim = Rectangle::new()
            .width(7.7)
            .height(4.6)
            .style(flat(hex(BACKGROUND), 0.72))
            .build()?
            .moved_to(v(3.1, 1.05));
        let scrim = stage.arena_mut().add(scrim);
        // the note leaves before the slider takes its place, so the two
        // never cross-fade on top of each other
        play!(stage; fade_out(stage.arena_mut(), retry, ORIGIN, 1.0)?.rt(0.3));
        play!(stage;
            fade_in(stage.arena_mut(), track, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), knob, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), rider, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), chance, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), scrim, ORIGIN, 1.0)?.rt(0.4),
        );
        let percent = live_percent(
            &kit,
            stage,
            move |s| 100.0 * view.pose(s).x,
            26.0,
            v(chance_right + 0.15, SLIDER_Y - 0.48),
        )?;
        readings = read_at(&kit, stage, &square, &readings, Q::HALF, 1.1)?;
        kit.lint_here(stage, "04_5 at one half");
        // the fair coin's sum stands while its sentence is said
        wait_until(stage, line.at("A coin that lands heads")? + 0.4)?;
        readings = read_at(&kit, stage, &square, &readings, far, 2.4)?;
        kit.lint_here(stage, "04_5 at the far end");
        wait_until(stage, line.end() - 0.2)?;
        read_at(&kit, stage, &square, &readings, p, 1.1)?;

        // 04_6: nobody can promise when a game stops; the tail, one tick a
        // game, in its own band under the axis numbers
        let line = say(&kit, stage, "04_6")?;
        fade_out_all(stage, &[track, knob, rider, chance, percent, scrim], 0.4)?;
        let no_rule = lower_third(&kit, stage, PRECISELY, NO_RULE)?;
        play!(stage; fade_in(stage.arena_mut(), no_rule, ORIGIN, 1.0)?.rt(0.5));
        let slot_w = 0.8 * pitch;
        let tick_step = slot_w / TAIL_ROW as f64;
        let mut tail: Vec<Mob> = Vec::new();
        let mut in_tail = 0;
        for k in BAR_PAIRS + 1..=longest / TOSSES_A_PAIR {
            let n = games.iter().filter(|g| g.len() == k).count();
            if n == 0 {
                continue;
            }
            in_tail += n;
            // rows of TAIL_ROW ticks stacked up from the floor, one tick a game
            let centre = toss_x((k * TOSSES_A_PAIR) as f64);
            let across = n.min(TAIL_ROW) as f64;
            let ticks = (0..n).map(|i| {
                let (row, col) = (i / TAIL_ROW, i % TAIL_ROW);
                let x = centre + tick_step * (col as f64 - (across - 1.0) / 2.0);
                let y = BAND_FLOOR + TAIL_ROW_STEP * row as f64;
                polyline(vec![v(x, y), v(x, y + TICK_LEN)])
                    .style(line_style(Palette::dim(), 1.8))
                    .build()
            });
            tail.push(stage.arena_mut().add(v_group(ticks)));
        }
        let shortest = (BAR_PAIRS + 1) * TOSSES_A_PAIR;
        let floor = polyline(vec![
            v(toss_x(shortest as f64 - 1.0), BAND_FLOOR - 0.04),
            v(toss_x(longest as f64 + 1.5), BAND_FLOOR - 0.04),
        ])
        .style(line_style(Palette::faint(), 2.0))
        .build();
        let floor = stage.arena_mut().add(floor);
        let tail_left = toss_x(34.0) + 0.1;
        let tail_words = [
            format!(
                "{} {} {shortest} {}",
                thousands(in_tail as u64),
                TAIL_WORDS[0],
                TAIL_WORDS[1]
            ),
            TAIL_WORDS[2].to_owned(),
        ];
        let mut tail_notes = Vec::new();
        for (i, words) in tail_words.iter().enumerate() {
            let t = kit.text(stage, words, 22.0, Palette::dim(), ORIGIN)?;
            on_baseline(stage, t, v(tail_left, -1.4 - 0.32 * i as f64));
            tail_notes.push(t);
        }
        let mut band = vec![floor];
        band.extend(&tail_notes);
        fade_in_all(stage, &band, 0.4)?;
        fade_in_lagged(stage, &tail, 1.4, 0.3)?;
        // the model's word on the tail
        let going_tosses = STILL_GOING_PAIRS as usize * TOSSES_A_PAIR;
        let going_at = going_tosses as f64;
        let going = DashedLine::new(v(toss_x(going_at), AXIS_Y), v(toss_x(going_at), AXIS_Y + 1.55))
            .dash_length(0.1)
            .style(line_style(ink, 2.5))
            .build()
            .map_err(|e| fail(format!("the model's mark on the tail: {e}")))?;
        let going = stage.arena_mut().add(going);
        let going_note = format!(
            "{AFTER} {going_tosses} {TOSSES}: {}% {STILL_GOING}",
            undecided_after(p, STILL_GOING_PAIRS).percent(1)
        );
        let going_note = kit.text(stage, &going_note, 24.0, ink, ORIGIN)?;
        on_baseline(stage, going_note, v(toss_x(going_at) + 0.18, AXIS_Y + 1.3));
        fade_in_all(stage, &[going, going_note], 0.5)?;
        line.wait_for(stage, "The longest")?;
        let last_x = toss_x(longest as f64);
        // a small arrowhead over the last tick (a line would read as a tick)
        let pointer = Polygon::new([
            v(last_x - 0.07, BAND_FLOOR + 0.24),
            v(last_x + 0.07, BAND_FLOOR + 0.24),
            v(last_x, BAND_FLOOR + 0.1),
        ])
        .style(flat(Palette::dim(), 1.0))
        .build();
        let pointer = stage.arena_mut().add(pointer);
        let longest_note = kit.text(
            stage,
            &format!("{THE_LONGEST} {longest} {TOSSES}"),
            24.0,
            Palette::dim(),
            ORIGIN,
        )?;
        stage.move_to(longest_note, v(last_x + 0.25, BAND_FLOOR + 0.45), RIGHT);
        fade_in_all(stage, &[pointer, longest_note], 0.5)?;
        // back to the resting square, which chapter 05 opens on: the grey
        // settles, everything else leaves, and the square holds through the cut
        wait_until(stage, line.end() + 1.0)?;
        play!(stage;
            square.repaint(Block::HH, thrown(), 0.4)?,
            square.repaint(Block::TT, thrown(), 0.4)?,
        );
        fade_all_but(stage, &resting, 0.6)?;
        kit.music_breath(stage, HOLD_S)?;
        kit.finish(stage)
    }
}
