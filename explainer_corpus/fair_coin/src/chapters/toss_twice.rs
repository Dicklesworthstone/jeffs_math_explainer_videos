//! Chapter 02, toss twice: two results are a pair (02_1), one of four
//! (02_2); every way two tosses can go is a square, cut by the first toss
//! (02_3) and by the second at the same mark (02_4); a block's share is its
//! pair's chance, counted in small squares (02_5), and the two thin blocks
//! are four each (02_6); so Alice takes one and Bob the other (02_7), a
//! thousand seeded games land in the square as dots (02_8), and the coin's
//! own tally sits over the game's (02_9).

use super::draw::predict_pause;
use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, hex, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Block, COLUMN_LEFT, COLUMN_MID, SQUARE_AT, SQUARE_SIDE, SplitBar, Square, alice, bob, caption, disc, dot_batches,
    fade_all_but, fade_in_all, fade_out_all, flat, group, heads, line_style, pair_tile, say, tails, thrown, tween_all,
};
use crate::facts::{call_it_run, runs, trick_draws, trick_run};
use crate::model::{Face, PairLaw, Q, bent, heads_first_wins, two_sigma_band};

pub struct TossTwice {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const A_PAIR: &str = "a pair";
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const FIRST_TOSS: &str = "first toss";
const SECOND_TOSS: &str = "second toss";
const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const ALICE_CALLS: &str = "Alice calls heads then tails";
const BOB_CALLS: &str = "Bob calls tails then heads";
const MATCH: &str = "a matching pair: toss two fresh ones";
const PROGRAMMERS: &str = "for programmers:";
const FRESH: &str =
    "fresh tosses for every pair. Reuse a toss from a thrown-away pair and you get plain heads-or-tails back";
const GAMES_CAPTION: &str = "games of our coin,";
const PAIRS_CAPTION: &str = "pairs, one dot each:";
const THROWN_CAPTION: &str = "thrown away";
const THE_COIN: &str = "the coin:";
const THE_GAME: &str = "the game:";
const TOSSES: &str = "tosses";
const FAIR_BAND_A: &str = "a perfectly fair game rarely comes out";
const FAIR_BAND_B: &str = "each:";
const FAIR_BAND_C: &str = "about 19 times in 20 (model)";

/// Where a block's tile, its count and its percent sit, in the square's own
/// coordinates (across, up).
fn spots(b: Block) -> [(f64, f64); 3] {
    match b {
        Block::HH => [(0.4, 0.52), (0.4, 0.34), (0.4, 0.2)],
        Block::HT => [(0.2, 0.9), (0.42, 0.9), (0.62, 0.9)],
        Block::TH => [(0.9, 0.66), (0.9, 0.5), (0.9, 0.36)],
        Block::TT => [(0.9, 0.945), (0.9, 0.85), (0.9, 0.85)],
    }
}

/// A dot's colour: dark on Alice's green block, light everywhere else.
fn dot_color(a: Face, c: Face) -> Srgb {
    if (a, c) == (Face::H, Face::T) {
        hex("#0E1116")
    } else {
        Palette::ink()
    }
}

/// A player's name on a dark chip edged in the player's colour, so it reads
/// on a coloured block and with dots around it. Detached, one group.
fn chip(kit: &Kit, stage: &mut Stage<'_>, name: &str, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let text = kit.text(stage, name, 24.0, Palette::ink(), at)?;
    let (w, h) = (stage.get_width(text), stage.get_height(text));
    let back = Rectangle::new()
        .width(w + 0.24)
        .height(h + 0.2)
        .corner_radius(0.08)
        .style(Style::default().fill(hex("#0E1116"), 0.92).stroke(color, 2.0, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    group(stage, &[back, text])
}

fn pair_of(b: Block) -> (Face, Face) {
    match b {
        Block::HH => (Face::H, Face::H),
        Block::HT => (Face::H, Face::T),
        Block::TH => (Face::T, Face::H),
        Block::TT => (Face::T, Face::T),
    }
}

/// How many of the 25 small squares a block holds at four fifths.
fn small_squares(b: Block) -> fmn::Result<i128> {
    let (a, c) = pair_of(b);
    let share = PairLaw::same_coin(bent()).prob(a, c) * Q::int(25);
    share
        .exact()
        .parse()
        .map_err(|_| super::draw::fail(format!("{b:?} is not a whole number of small squares")))
}

impl SceneConstruct for TossTwice {
    fn name(&self) -> &str {
        "02_toss_twice"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = bent().f64();

        // 02_1: two results are a pair
        let line = say(&kit, stage, "02_1")?;
        let first = trick_run()
            .first()
            .and_then(|g| g.first().copied())
            .ok_or_else(|| super::draw::fail("run 2 has no games".into()))?;
        let big = pair_tile(&kit, stage, first, v(COLUMN_MID, 0.9), 1.5)?;
        fade_in_all(stage, &big[1..3], 0.5)?;
        fade_in_all(stage, &big[3..5], 0.5)?;
        line.wait_for(stage, "as a pair")?;
        let word = kit.text(stage, A_PAIR, 30.0, Palette::dim(), v(COLUMN_MID, -0.05))?;
        fade_in_all(stage, &[big[0], word], 0.5)?;
        stage.scene_mut().bring_to_front(&big[1..])?;

        // 02_2: the four things a pair can be, in the order they are said
        let line = say(&kit, stage, "02_2")?;
        let mut gone = big.clone();
        gone.push(word);
        let said = [Block::HH, Block::TT, Block::HT, Block::TH];
        let phrases = ["Two heads", "two tails", "heads then tails", "tails then heads"];
        let mut tiles: Vec<(Block, Mob)> = Vec::new();
        for (i, (&b, phrase)) in said.iter().zip(phrases).enumerate() {
            line.wait_for(stage, phrase)?;
            let at = v(COLUMN_LEFT + 0.9 + 1.85 * i as f64, 1.7);
            let parts = pair_tile(&kit, stage, pair_of(b), at, 1.0)?;
            let tile = group(stage, &parts)?;
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            if i == 0 {
                // the opening pair stays until the first of the four lands,
                // so the stage is never empty
                for &m in &gone {
                    let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                    anims.push(stage.prepare(a.rt(0.45))?);
                }
            }
            let a = fade_in(stage.arena_mut(), tile, v(0.0, -0.2), 1.0)?;
            anims.push(stage.prepare(a.rt(0.45))?);
            stage.play_prepared(anims)?;
            tiles.push((b, tile));
        }
        kit.hold(stage)?;

        // 02_3: every way the two tosses could go, as a square; the first
        // toss cuts it
        let line = say(&kit, stage, "02_3")?;
        let square = Square::build(stage, SQUARE_AT, SQUARE_SIDE, 1.0)?;
        fade_in_all(stage, &square.mobs(), 0.8)?;
        line.wait_for(stage, "The first toss cuts")?;
        play!(stage; tween(square.view.x, p, 1.3)?);
        let edges = square.edges(&kit, stage, HEADS, TAILS)?;
        let first_toss = kit.text(
            stage,
            FIRST_TOSS,
            22.0,
            Palette::dim(),
            v(SQUARE_AT[0] + SQUARE_SIDE / 2.0, SQUARE_AT[1] - 0.78),
        )?;
        let mut bottom = vec![edges.anchor, edges.stretches[0], edges.stretches[1], edges.words[0]];
        bottom.extend([edges.words[1], first_toss]);
        line.wait_for(stage, "with four fifths")?;
        fade_in_all(stage, &bottom, 0.6)?;

        // 02_4: the second toss cuts it again, at the same mark
        let line = say(&kit, stage, "02_4")?;
        line.wait_for(stage, "four fifths of the way up")?;
        tween_all(stage, &[(square.view.y_h, p), (square.view.y_t, p)], 1.3)?;
        // the left edge's words were placed for a cut at the top: put them
        // where the cut now is
        let pose = square.pose(stage);
        let beside = pose.left - 0.55;
        stage.move_to(edges.words[2], v(beside, pose.point(0.0, pose.y_h / 2.0)[1]), ORIGIN);
        stage.move_to(
            edges.words[3],
            v(beside, pose.point(0.0, (1.0 + pose.y_h) / 2.0)[1]),
            ORIGIN,
        );
        let second_toss = kit.text_left(
            stage,
            SECOND_TOSS,
            22.0,
            Palette::dim(),
            v(SQUARE_AT[0] - 0.95, SQUARE_AT[1] + SQUARE_SIDE + 0.32),
        )?;
        let left = [
            edges.stretches[2],
            edges.stretches[3],
            edges.words[2],
            edges.words[3],
            second_toss,
        ];
        fade_in_all(stage, &left, 0.6)?;
        line.wait_for(stage, "because it's the same coin")?;
        let small = square.small_squares(stage, 5)?;
        play!(stage; fade_in(stage.arena_mut(), small, ORIGIN, 1.0)?.rt(0.6));
        // each block takes its pair tile (the tiles were drawn before the
        // square, so they come to the front first)
        let tile_mobs: Vec<Mob> = tiles.iter().map(|&(_, t)| t).collect();
        stage.scene_mut().bring_to_front(&tile_mobs)?;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for &(b, tile) in &tiles {
            let (u, w) = spots(b)[0];
            let to = pose.point(u, w);
            let a = tile
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .scale(0.4)?
                .move_to(to, ORIGIN)?;
            moves.push(stage.prepare(a)?);
        }
        stage.play_prepared(moves)?;

        // 02_5: a block's share is its pair's chance; count the small squares
        let line = say(&kit, stage, "02_5")?;
        line.wait_for(stage, "Two heads has sixteen")?;
        let mut counts: Vec<(Block, Mob)> = Vec::new();
        for (b, phrase) in [(Block::HH, "Two heads has sixteen"), (Block::TT, "two tails has one")] {
            line.wait_for(stage, phrase)?;
            let (u, w) = spots(b)[1];
            let t = stage.add_value_tracker(0.0);
            let n = kit.live_number(stage, t, 0, false, 34.0, Palette::ink(), pose.point(u, w), ORIGIN)?;
            play!(stage; tween(t, small_squares(b)? as f64, 0.8)?);
            counts.push((b, n));
        }
        let mut asks = Vec::new();
        for b in [Block::HT, Block::TH] {
            let (u, w) = spots(b)[1];
            asks.push(kit.text(stage, "?", 34.0, Palette::hero(), pose.point(u, w))?);
        }
        line.wait_for(stage, "What about heads then tails")?;
        fade_in_all(stage, &asks, 0.4)?;
        let q = predict_pause(&kit, stage, v(COLUMN_MID, 0.4))?;

        // 02_6: four each; four in twenty-five is sixteen percent
        let line = say(&kit, stage, "02_6")?;
        asks.push(q);
        fade_out_all(stage, &asks, 0.3)?;
        let mut trackers = Vec::new();
        for b in [Block::HT, Block::TH] {
            let (u, w) = spots(b)[1];
            let t = stage.add_value_tracker(0.0);
            let n = kit.live_number(stage, t, 0, false, 34.0, Palette::ink(), pose.point(u, w), ORIGIN)?;
            trackers.push((t, small_squares(b)? as f64));
            counts.push((b, n));
        }
        tween_all(stage, &trackers, 0.7)?;
        line.wait_for(stage, "Four in twenty-five")?;
        let law = PairLaw::same_coin(bent());
        let mut percents = Vec::new();
        for b in [Block::HT, Block::TH] {
            let (a, c) = pair_of(b);
            let (u, w) = spots(b)[2];
            let text = format!("{}%", law.prob(a, c).percent(0));
            percents.push(kit.text(stage, &text, 30.0, Palette::ink(), pose.point(u, w))?);
        }
        // two sixteens must not share the square (16 small squares, and 16
        // percent): the counts for the matching pairs go before the percents land
        let (matching, deciding): (Vec<_>, Vec<_>) = counts
            .into_iter()
            .partition(|&(b, _)| matches!(b, Block::HH | Block::TT));
        counts = deciding;
        let matching: Vec<Mob> = matching.iter().map(|&(_, m)| m).collect();
        fade_out_all(stage, &matching, 0.3)?;
        fade_in_all(stage, &percents, 0.5)?;

        // 02_7: Alice calls heads then tails, Bob calls tails then heads; a
        // matching pair is thrown away
        let line = say(&kit, stage, "02_7")?;
        // the counts and percents have said their piece: a coloured block
        // carries its pair and its owner's name
        let mut count_mobs: Vec<Mob> = counts.iter().map(|&(_, m)| m).collect();
        count_mobs.extend(&percents);
        fade_out_all(stage, &count_mobs, 0.3)?;
        let names = [
            (Block::HT, ALICE, alice(), "Alice calls"),
            (Block::TH, BOB, bob(), "Bob calls"),
        ];
        let legend_x = COLUMN_LEFT + 0.3;
        let mut legend = Vec::new();
        let mut chips = Vec::new();
        for (i, (b, name, color, phrase)) in names.into_iter().enumerate() {
            line.wait_for(stage, phrase)?;
            let (u, w) = spots(b)[1];
            let label = chip(&kit, stage, name, color, pose.point(u, w))?;
            chips.push(label);
            let y = 2.45 - 0.55 * i as f64;
            let swatch = Rectangle::new()
                .width(0.36)
                .height(0.36)
                .style(flat(color, 1.0))
                .build()?;
            let swatch = stage.arena_mut().add(swatch.moved_to(v(legend_x, y)));
            let words = [ALICE_CALLS, BOB_CALLS][i];
            let text = kit.text_left(stage, words, 26.0, Palette::ink(), v(legend_x + 0.4, y))?;
            play!(stage;
                square.repaint(b, color, 0.6)?,
                fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.6),
                fade_in(stage.arena_mut(), swatch, ORIGIN, 1.0)?.rt(0.6),
                fade_in(stage.arena_mut(), text, ORIGIN, 1.0)?.rt(0.6),
            );
            legend.extend([swatch, text]);
        }
        line.wait_for(stage, "If the pair matches")?;
        let y = 2.45 - 0.55 * 2.0;
        let swatch = Rectangle::new()
            .width(0.36)
            .height(0.36)
            .style(flat(thrown(), 1.0))
            .build()?;
        let swatch = stage.arena_mut().add(swatch.moved_to(v(legend_x, y)));
        let text = kit.text_left(stage, MATCH, 26.0, Palette::dim(), v(legend_x + 0.4, y))?;
        play!(stage;
            square.repaint(Block::HH, thrown(), 0.6)?,
            square.repaint(Block::TT, thrown(), 0.6)?,
            fade_in(stage.arena_mut(), swatch, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), text, ORIGIN, 1.0)?.rt(0.6),
        );
        legend.extend([swatch, text]);
        let fresh = lower_third(&kit, stage, PROGRAMMERS, FRESH)?;
        play!(stage; fade_in(stage.arena_mut(), fresh, ORIGIN, 1.0)?.rt(0.5));

        // 02_8: a thousand games, every pair a dot in its own block
        let line = say(&kit, stage, "02_8")?;
        fade_out_all(stage, &[fresh, small], 0.4)?;
        let draws = trick_draws();
        let games = trick_run();
        let wins = heads_first_wins(&games);
        let n = games.len();
        // the first eight games at reading speed: each pair a small mark in
        // the column and a dot in the square
        let shown = 8.min(n);
        let (row_h, step) = (0.36, 0.56);
        let mut marks: Vec<Mob> = Vec::new();
        let mut early: Vec<Box<dyn Animation>> = Vec::new();
        for (g, game) in games.iter().take(shown).enumerate() {
            let y = 0.75 - row_h * g as f64;
            for (k, (&(a, c), &(d1, d2))) in game.iter().zip(&draws[g]).enumerate() {
                let x = COLUMN_LEFT + 0.25 + step * k as f64;
                let decided = a != c;
                let fade = if decided { 1.0 } else { 0.45 };
                let mut parts = Vec::new();
                for (f, dx) in [(a, -0.1), (c, 0.1)] {
                    let mut d = disc(f, v(x + dx, y), 0.085);
                    d = d.with_style(flat(if f == Face::H { heads() } else { tails() }, fade));
                    parts.push(stage.arena_mut().add(d));
                }
                if decided {
                    let winner = if a == Face::H { alice() } else { bob() };
                    let under = polyline(vec![v(x - 0.2, y - 0.14), v(x + 0.2, y - 0.14)])
                        .style(line_style(winner, 4.0))
                        .build();
                    parts.push(stage.arena_mut().add(under));
                }
                let dot = Dot::new()
                    .point(pose.point(d1, d2))
                    .radius(0.03)
                    .style(flat(dot_color(a, c), 0.95))
                    .build();
                parts.push(stage.arena_mut().add(dot));
                for &m in &parts {
                    let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                    early.push(stage.prepare(a)?);
                }
                marks.extend(&parts[..parts.len() - 1]);
            }
        }
        let reveal = AnimationGroup::with_lag_ratio(stage.arena_mut(), early, 0.06)?.with_run_time(2.6);
        stage.play(reveal)?;
        // then every pair of the run
        line.wait_for(stage, "Alice wins")?;
        fade_out_all(stage, &marks, 0.4)?;
        let (mut light, mut dark) = (Vec::new(), Vec::new());
        for (game, game_draws) in games.iter().zip(&draws).skip(shown) {
            for (&(a, c), &(d1, d2)) in game.iter().zip(game_draws) {
                let at = pose.point(d1, d2);
                if dot_color(a, c) == Palette::ink() {
                    light.push(at)
                } else {
                    dark.push(at)
                }
            }
        }
        let mut batches = dot_batches(stage, &light, Palette::ink(), 0.02, 30);
        let dark_batches = dot_batches(stage, &dark, hex("#0E1116"), 0.02, 30);
        // interleave, so both kinds land through the whole reveal
        batches = batches
            .into_iter()
            .zip(dark_batches.iter().copied().map(Some).chain(std::iter::repeat(None)))
            .flat_map(|(l, d)| std::iter::once(l).chain(d))
            .collect();
        let count_y = -0.45;
        // the scores are shown once every game has landed (never a ramp
        // toward the final count)
        let (ta, tb) = (
            stage.add_value_tracker(wins as f64),
            stage.add_value_tracker((n - wins) as f64),
        );
        let alice_word = kit.text_left(stage, ALICE, 30.0, alice(), v(COLUMN_LEFT + 0.3, count_y))?;
        let bob_word = kit.text_left(stage, BOB, 30.0, bob(), v(COLUMN_LEFT + 4.0, count_y))?;
        fade_in_all(stage, &[alice_word, bob_word], 0.3)?;
        let mut all: Vec<Box<dyn Animation>> = Vec::new();
        let mut lands: Vec<Box<dyn Animation>> = Vec::new();
        for &b in &batches {
            let a = fade_in(stage.arena_mut(), b, ORIGIN, 1.0)?;
            lands.push(stage.prepare(a)?);
        }
        let lands = AnimationGroup::with_lag_ratio(stage.arena_mut(), lands, 0.5)?.with_run_time(2.6);
        all.push(Box::new(lands));
        stage.play_prepared(all)?;
        kit.live_number(
            stage,
            ta,
            0,
            false,
            34.0,
            Palette::ink(),
            v(COLUMN_LEFT + 2.4, count_y),
            RIGHT,
        )?;
        kit.live_number(
            stage,
            tb,
            0,
            false,
            34.0,
            Palette::ink(),
            v(COLUMN_LEFT + 5.6, count_y),
            RIGHT,
        )?;
        stage.scene_mut().bring_to_front(&chips)?;
        // the game's bar, with where a perfectly fair game would land
        let bar_left = v(COLUMN_LEFT + 0.3, -1.05);
        let bar_w = 6.4;
        let game_bar = SplitBar::build(stage, bar_left, (bar_w, 0.34), (alice(), bob()), wins as f64 / n as f64)?;
        let (lo, hi) = two_sigma_band(runs::TRICK as u32, Q::HALF);
        let mut band = Vec::new();
        for k in [lo, hi] {
            let x = bar_left[0] + bar_w * f64::from(k) / n as f64;
            let tick = polyline(vec![v(x, bar_left[1] - 0.3), v(x, bar_left[1] + 0.3)])
                .style(line_style(Palette::ink(), 2.5))
                .build();
            band.push(stage.arena_mut().add(tick));
        }
        let fair_1 = format!("{FAIR_BAND_A} {} {FAIR_BAND_B}", n / 2);
        let fair_2 = format!("{lo} to {hi}, {FAIR_BAND_C}");
        let fair = kit.text_left(stage, &fair_1, 20.0, Palette::dim(), v(bar_left[0], -1.58))?;
        let fair_2 = kit.text_left(stage, &fair_2, 20.0, Palette::dim(), v(bar_left[0], -1.88))?;
        // a dot is a pair: say how many there were and how many were thrown away
        let pairs: usize = games.iter().map(|g| g.len()).sum();
        let cap = caption(
            &kit,
            stage,
            &format!(
                "{} {GAMES_CAPTION} {} {PAIRS_CAPTION} {} {THROWN_CAPTION}",
                explainer_kit::thousands(n as u64),
                explainer_kit::thousands(pairs as u64),
                explainer_kit::thousands((pairs - n) as u64)
            ),
        )?;
        let mut ins = game_bar.mobs().to_vec();
        ins.extend(&band);
        ins.extend([fair, fair_2, cap]);
        fade_in_all(stage, &ins, 0.6)?;
        // the run heard: its first games pair by pair (a tick thrown away,
        // the motif's two notes in the order a deciding pair fell), then the
        // race as an even patter (R1's C11)
        let wav = super::sound::the_trick()?;
        kit.sonify(stage, super::sound::WINDOW_S, "02_toss_twice:02_8", &wav)?;
        super::draw::pause(stage, super::sound::WINDOW_S)?;

        // 02_9: the coin is as unfair as ever; the game isn't
        say(&kit, stage, "02_9")?;
        let coin = call_it_run();
        let coin_heads = coin.iter().filter(|&&f| f == Face::H).count();
        let coin_bar = SplitBar::build(
            stage,
            v(bar_left[0], 0.75),
            (bar_w, 0.34),
            (heads(), tails()),
            coin_heads as f64 / coin.len() as f64,
        )?;
        let coin_words = format!(
            "{THE_COIN} {coin_heads} {HEADS}, {} {TAILS} in {} {TOSSES}",
            coin.len() - coin_heads,
            explainer_kit::thousands(coin.len() as u64)
        );
        let coin_words = kit.text_left(stage, &coin_words, 24.0, Palette::ink(), v(bar_left[0], 1.25))?;
        let game_words = kit.text_left(stage, THE_GAME, 24.0, Palette::ink(), v(bar_left[0], 0.05))?;
        let mut ins = coin_bar.mobs().to_vec();
        ins.extend([coin_words, game_words]);
        // the legend makes room
        fade_out_all(stage, &legend, 0.4)?;
        fade_in_all(stage, &ins, 0.7)?;
        kit.music_breath(stage, 1.5)?;
        // everything but the square goes: chapter 03 opens on it as it rests
        // (painted, with its edges), so the picture stays through the cut
        let mut keep = square.mobs();
        keep.extend(edges.mobs());
        fade_all_but(stage, &keep, 0.8)?;
        kit.music_breath(stage, 1.0)?;
        kit.finish(stage)
    }
}
