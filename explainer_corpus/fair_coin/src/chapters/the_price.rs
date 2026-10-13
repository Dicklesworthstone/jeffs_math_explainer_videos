//! Chapter 04, the price: the grey of the square is what the trick pays
//! (04_1); how many tosses a game takes, asked over one slow game (04_2) and
//! answered from the chance that a pair decides (04_3); ten thousand seeded
//! games as a histogram with their mean (04_4); the same sum for a fair coin
//! and for a coin at ninety-nine in a hundred, on the square's own tracker
//! (04_5); and the tail of the ten thousand as single ticks, the last at the
//! longest game (04_6).

use std::rc::Rc;

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, children, fade_all, hex, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    Block, COLUMN_MID, SQUARE_AT, SQUARE_SIDE, Square, SquarePose, alice, bob, column, fade_in_all, fade_in_lagged,
    fade_out_all, fail, flat, group, line_style, pair_tile, panel, say, thrown, thrown_lit, wait_until,
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
const DIVIDED_BY: &str = "÷";
const A_GAME_MODEL: &str = "tosses a game (model)";
const COUNTED_IN: &str = "counted in";
const GAMES: &str = "games:";
const TOSSES_IN_A_GAME: &str = "tosses in a game";
const PROGRAMMERS: &str = "for programmers:";
const RETRY: &str = "rejection sampling, a retry loop with no upper bound. If you cap it, the cap returns no result, never a default winner";
const CHANCE_OF_HEADS: &str = "chance of heads";
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

/// The legend above the square: the first row's baseline and the step down
/// to the second.
const LEGEND_BASE: f64 = SQUARE_AT[1] + SQUARE_SIDE + 0.72;
const LEGEND_STEP: f64 = 0.46;
/// The baseline of the sum under the square, and the slider under that.
const SUM_BASE: f64 = SQUARE_AT[1] - 0.58;
const SLIDER_Y: f64 = SQUARE_AT[1] - 1.25;

/// The histogram's axis: where no tosses would sit, the width of one toss,
/// the axis's height and the height of the tallest bar.
const AXIS_LEFT: f64 = -0.6;
const TOSS_W: f64 = 0.118;
const AXIS_Y: f64 = -1.0;
const BAR_H: f64 = 3.3;

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

/// The chance a pair decides, as the square stands: its two coloured blocks.
fn decides(pose: &SquarePose) -> f64 {
    pose.share(Block::HT) + pose.share(Block::TH)
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

/// A live number computed from the arena every frame: `f` gives the value
/// and how many decimals to print, `unit` follows it ("%" or nothing), and
/// the corner `pin.1` of the number stays at `pin.0`. It is on screen as soon
/// as it is built; clear its updaters before fading it (`fade_out_all` does).
fn live_readout(
    kit: &Kit,
    stage: &mut Stage<'_>,
    f: impl Fn(&fmn::mobject::Stage) -> (f64, usize) + 'static,
    unit: &'static str,
    ink: (f64, Srgb),
    pin: (Vec3, Vec3),
) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let (value, places) = f(s);
        // with no decimals a `DecimalNumber` cuts the fraction off (31.99…
        // would read 31): round first
        let value = if places == 0 { value.round() } else { value };
        let mut d = DecimalNumber::new(value)
            .num_decimal_places(places)
            .font_size(ink.0)
            .color(ink.1);
        if !unit.is_empty() {
            d = d.unit(unit);
        }
        let d = d.build(&book).expect("bundled digits always lay out");
        let m = s.add(d.into_vmob());
        s.move_to(m, pin.0, pin.1);
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

impl SceneConstruct for ThePrice {
    fn name(&self) -> &str {
        "04_the_price"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = bent();
        let law = PairLaw::same_coin(p);
        let never = || fail("a coin that never decides has no price".into());
        let cost = expected_tosses(p).ok_or_else(never)?;
        let per_pair = Q::int(TOSSES_A_PAIR as i128);
        // the sum on screen, two tosses over the chance a pair decides, is the model's cost
        if per_pair / law.decides() != cost {
            return Err(fail(
                "two tosses over the chance a pair decides is not the model's cost".into(),
            ));
        }
        // the slider's far end and the fair coin: the two stops the sum is read at
        let far = stops().into_iter().max().ok_or_else(never)?;
        for s in [Q::HALF, p, far] {
            let x = s.f64();
            let shown = format!("{:.2}", per_pair.f64() / (2.0 * x * (1.0 - x)));
            let model = expected_tosses(s).ok_or_else(never)?.decimal(2);
            if shown != model {
                return Err(fail(format!(
                    "at {} the live sum would read {shown}, the model says {model}",
                    s.exact()
                )));
            }
        }
        let games = price_run();
        let ink = Palette::ink();

        // 04_1: the square at four fifths; the grey is what gets thrown away.
        // The square is the picture the last chapter left: it is back during
        // the breath before the first word
        let square = Square::build(stage, SQUARE_AT, SQUARE_SIDE, p.f64())?;
        let paints = [
            (Block::HH, thrown()),
            (Block::HT, alice()),
            (Block::TH, bob()),
            (Block::TT, thrown()),
        ];
        for (b, c) in paints {
            square.paint(stage, b, c, 1.0);
        }
        let edges = square.edges(&kit, stage, HEADS, TAILS)?;
        let mut ins = square.mobs();
        ins.push(edges.anchor);
        ins.extend(edges.stretches);
        fade_in_all(stage, &ins, 0.6)?;
        let line = say(&kit, stage, "04_1")?;
        let view = square.view;
        let legend = [
            (
                THROWN_AWAY,
                [Block::HH, Block::TT],
                [thrown_lit(), thrown_lit()],
                "most pairs get thrown away",
            ),
            (DECIDES, [Block::HT, Block::TH], [alice(), bob()], "A pair decides"),
        ];
        for (i, (word, blocks, colors, phrase)) in legend.into_iter().enumerate() {
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
            parts.push(label);
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for &m in &parts {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.6))?);
            }
            if i == 0 {
                // the two grey blocks lift while they are the subject
                for b in blocks {
                    anims.push(stage.prepare(square.repaint(b, thrown_lit(), 0.6)?)?);
                }
            }
            stage.play_prepared(anims)?;
            live_readout(
                &kit,
                stage,
                move |s| {
                    let pose = view.pose(s);
                    let percent = 100.0 * blocks.iter().map(|&b| pose.share(b)).sum::<f64>();
                    (percent, usize::from(percent < 3.0))
                },
                "%",
                (28.0, ink),
                (v(SQUARE_AT[0] + 3.3, base), DR),
            )?;
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
        // tosses; exactly, two tosses over the chance a pair decides
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
        played.extend([pairs_note, tosses_note]);
        line.wait_for(stage, "Exactly")?;
        // the sum under the square: fixed words, and two numbers that follow the square's cuts
        let size = 24.0;
        let lead = kit.text(
            stage,
            &format!("{TOSSES_A_PAIR} {TOSSES} {DIVIDED_BY}"),
            size,
            ink,
            ORIGIN,
        )?;
        let equals = kit.text(stage, "=", size, ink, ORIGIN)?;
        let tail = kit.text(stage, A_GAME_MODEL, size, ink, ORIGIN)?;
        // room for each number at its widest: the far end of the slider
        let far_cost = expected_tosses(far).ok_or_else(never)?;
        let widest = [PairLaw::same_coin(far).decides().decimal(4), far_cost.decimal(2)];
        let mut slot = [0.0; 2];
        for (w, text) in slot.iter_mut().zip(&widest) {
            let probe = kit.text(stage, text, size, ink, ORIGIN)?;
            *w = stage.get_width(probe);
        }
        let gap = 0.13;
        let widths = [
            stage.get_width(lead),
            slot[0],
            stage.get_width(equals),
            slot[1],
            stage.get_width(tail),
        ];
        let total = widths.iter().sum::<f64>() + gap * (widths.len() - 1) as f64;
        let mut x = SQUARE_AT[0] + SQUARE_SIDE / 2.0 - total / 2.0;
        on_baseline(stage, lead, v(x, SUM_BASE));
        let digit = children(stage, lead).first().copied().unwrap_or(lead);
        let digit_h = stage.get_height(digit);
        x += widths[0] + gap;
        let divisor_at = v(x + widths[1], SUM_BASE);
        x += widths[1] + gap;
        stage.move_to(equals, v(x, SUM_BASE + digit_h / 2.0), LEFT);
        x += widths[2] + gap;
        let result_at = v(x, SUM_BASE);
        x += widths[3] + gap;
        on_baseline(stage, tail, v(x, SUM_BASE));
        fade_in_all(stage, &[lead, equals, tail], 0.4)?;
        live_readout(
            &kit,
            stage,
            move |s| {
                let d = decides(&view.pose(s));
                (d, if d < 0.1 { 4 } else { 2 })
            },
            "",
            (size, ink),
            (divisor_at, DR),
        )?;
        let reveal = stage.add_value_tracker(0.0);
        let tosses_a_pair = per_pair.f64();
        live_readout(
            &kit,
            stage,
            move |s| {
                let shown = s.tracker_value(reveal).unwrap_or(1.0);
                (shown * tosses_a_pair / decides(&view.pose(s)), 2)
            },
            "",
            (size, ink),
            (result_at, DL),
        )?;
        play!(stage; tween(reveal, 1.0, 0.8)?);

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
                v(toss_x(t as f64), AXIS_Y - 0.5),
            )?);
        }
        plot.push(kit.text(
            stage,
            TOSSES_IN_A_GAME,
            22.0,
            Palette::dim(),
            v(toss_x(longest as f64 / 2.0), AXIS_Y - 0.92),
        )?);
        let retry = lower_third(&kit, stage, PROGRAMMERS, RETRY)?;
        plot.push(retry);
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
        // the programmers' note needs a moment to be read
        wait_until(stage, line.end() + 1.0)?;

        // 04_5: the same sum for a fair coin, and for ninety-nine in a hundred
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
        on_baseline(stage, chance, v(SQUARE_AT[0], SLIDER_Y - 0.5));
        let chance_right = stage.get_right(chance)[0];
        // the counted games are for our coin only: they stand back while the coin changes
        let scrim = Rectangle::new()
            .width(7.7)
            .height(5.6)
            .style(flat(hex(BACKGROUND), 0.72))
            .build()?
            .moved_to(v(3.05, 0.5));
        let scrim = stage.arena_mut().add(scrim);
        play!(stage;
            fade_out(stage.arena_mut(), retry, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), track, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), knob, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), rider, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), chance, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), scrim, ORIGIN, 1.0)?.rt(0.4),
        );
        let percent = live_readout(
            &kit,
            stage,
            move |s| (100.0 * view.pose(s).x, 0),
            "%",
            (26.0, ink),
            (v(chance_right + 0.15, SLIDER_Y - 0.5), DL),
        )?;
        square.set_coin(stage, Q::HALF.f64(), 1.1)?;
        kit.lint_here(stage, "04_5 at one half");
        // the fair coin's sum stands while its sentence is said
        wait_until(stage, line.at("A coin that lands heads")? + 0.4)?;
        square.set_coin(stage, far.f64(), 3.0)?;
        kit.lint_here(stage, "04_5 at the far end");
        wait_until(stage, line.end() - 0.2)?;
        square.set_coin(stage, p.f64(), 1.4)?;

        // 04_6: nobody can promise when a game stops; the tail, one tick a game
        let line = say(&kit, stage, "04_6")?;
        fade_out_all(stage, &[track, knob, rider, chance, percent, scrim], 0.4)?;
        let mut slots: Vec<Mob> = Vec::new();
        for k in BAR_PAIRS + 1..=longest / TOSSES_A_PAIR {
            let n = games.iter().filter(|g| g.len() == k).count();
            if n == 0 {
                continue;
            }
            let centre = toss_x((k * TOSSES_A_PAIR) as f64);
            let ticks = (0..n).map(|i| {
                let x = centre + 0.8 * pitch * ((i as f64 + 0.5) / n as f64 - 0.5);
                polyline(vec![v(x, AXIS_Y - 0.08), v(x, AXIS_Y - 0.27)])
                    .style(line_style(Palette::dim(), 2.5))
                    .build()
            });
            slots.push(stage.arena_mut().add(v_group(ticks)));
        }
        fade_in_lagged(stage, &slots, 1.6, 0.3)?;
        let no_rule = lower_third(&kit, stage, PRECISELY, NO_RULE)?;
        play!(stage; fade_in(stage.arena_mut(), no_rule, ORIGIN, 1.0)?.rt(0.5));
        line.wait_for(stage, "The longest")?;
        let last_x = toss_x(longest as f64);
        let pointer = polyline(vec![v(last_x, AXIS_Y + 0.44), v(last_x, AXIS_Y + 0.08)])
            .style(line_style(Palette::dim(), 2.5))
            .build();
        let pointer = stage.arena_mut().add(pointer);
        let longest_note = kit.text(
            stage,
            &format!("{THE_LONGEST} {longest} {TOSSES}"),
            24.0,
            Palette::dim(),
            ORIGIN,
        )?;
        stage.move_to(longest_note, v(last_x + 0.25, AXIS_Y + 0.64), RIGHT);
        fade_in_all(stage, &[pointer, longest_note], 0.5)?;
        // the model's word on the tail, once the line has been said
        wait_until(stage, line.end())?;
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
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
