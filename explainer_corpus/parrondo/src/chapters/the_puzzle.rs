//! Chapter 01, the puzzle. Two games that each lose slowly; can switching
//! between them make money (01_1)? Before any machinery, one player with
//! seven dollars (01_2) tosses a coin each round, a dollar up or down (01_3,
//! a labelled example). Game A's bent coin lands heads 49.5 % of the time
//! (01_4). The player's first eight rounds of game A, from the one seeded
//! run, climb (01_5): short runs mislead. Think in averages: 49.5 wins and
//! 50.5 losses per hundred rounds (01_6); what does a round cost? (01_7, a
//! prediction) A cent (01_8). The same player's 100,000 rounds drift down
//! about a thousand dollars, the token riding the run's tip (01_9); luck
//! decides short stretches, the cent a round the long run (01_10): the
//! first 1,000 rounds, too small to read on the long plot, are drawn again
//! in an inset at their own scale, where they swing by tens of dollars
//! while the cent a round costs ten.
//!
//! Every number on screen comes from `crate::model` and `crate::facts`; the
//! scene refuses to draw if the words would stop being true of them (the
//! first rounds must end above the start, the long run within a hundred
//! dollars of a thousand down, game A's average one cent a round lost).

use explainer_kit::props::precisely;
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, rect, tween, v};
use fmn::core::constants::FRAME_X_RADIUS;
#[cfg(test)]
use fmn::core::constants::FRAME_Y_RADIUS;
use fmn::prelude::*;

use super::draw::{
    CHIP_R, arrow, badge, card, chip, coin, curved_arrow, dashed, draw_run, fade_in_all, fade_out_all, fade_to, fail,
    flash_all, fly, growing_run, keep_between, line_style, now, pause, read_s, say, screen, screen_int, shrink_all,
    stand, think, thinned, together, token, wait_until,
};
use crate::facts::{self, FIRST, SHORT};
use crate::model::{self, PUBLISHED, Play, RUN_ROUNDS, Ratio, START};
use crate::palette;

pub struct ThePuzzle {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const GAME_A: &str = "Game A";
const GAME_B: &str = "Game B";
const LOSES: &str = "loses slowly";
const ASK: &str = "?";
const PLAYER: &str = "player";
const DOLLARS: &str = "dollars";
const EXAMPLE: &str = "for example";
const UNTOSSED: &str = "?";
const HEADS_FACE: &str = "H";
const TAILS_FACE: &str = "T";
const PLUS_ONE: &str = "+1";
const MINUS_ONE: &str = "\u{2212}1";
const ROUNDS: &str = "rounds";
const QUESTION: &str = "average loss per round?";
const CENT_LINE: &str = "\u{2212}1 cent a round, on average";
/// The coin badge names the coins in play; their chances join it in 02_5,
/// once all three coins have been introduced.
const BADGE: &str = "published coins";

/// Where each picture lands: (line, the phrase it waits for).
const AT_LOSES: (&str, &str) = ("01_1", "slowly loses");
const AT_SWITCH: (&str, &str) = ("01_1", "Can switching");
const AT_PLAYER: (&str, &str) = ("01_2", "The player starts");
const AT_HEADS: (&str, &str) = ("01_3", "Heads wins");
const AT_TAILS: (&str, &str) = ("01_3", "and tails");
const AT_CHANCES: (&str, &str) = ("01_4", "forty-nine");
const AT_HUNDRED: (&str, &str) = ("01_6", "In a hundred rounds");
const AT_CENT: (&str, &str) = ("01_8", "a cent a round");
const AT_RUN: (&str, &str) = ("01_9", "for a hundred thousand rounds");
const AT_LUCK: (&str, &str) = ("01_10", "Luck decides");
const AT_LONG_RUN: (&str, &str) = ("01_10", "the cent a round decides");
#[cfg(test)]
const CUES: [(&str, &str); 11] = [
    AT_LOSES,
    AT_SWITCH,
    AT_PLAYER,
    AT_HEADS,
    AT_TAILS,
    AT_CHANCES,
    AT_HUNDRED,
    AT_CENT,
    AT_RUN,
    AT_LUCK,
    AT_LONG_RUN,
];

// ------------------------------------------------------------------ layout

/// 01_1: the two cards, large, and what passes between them.
const CARD_XS: [f64; 2] = [-3.2, 3.2];
const CARD_Y: f64 = 0.6;
const CARD_W: f64 = 4.4;
const CARD_H: f64 = 2.8;
const LOSES_DY: f64 = -0.2;
const LOSES_ARROW: (f64, f64) = (-0.5, -1.1);
const SWITCH_Y: f64 = 1.0;
const SWITCH_HALF: f64 = 0.9;
const SWITCH_BOW: f64 = 0.6;
const ASK_Y: f64 = 2.0;
/// 01_2: the cards shrink about the middle of the pair to the top left.
const SHRINK: f64 = 0.42;
const SHRINK_SHIFT: Vec3 = [-3.95, 2.65, 0.0];
/// The player: token, label, live count and chips, on the left third.
const TOKEN_AT: Vec3 = [-5.6, 0.9, 0.0];
const PLAYER_LABEL_AT: Vec3 = [-5.6, 0.35, 0.0];
const COUNT_AT: Vec3 = [-4.9, 1.5, 0.0];
const DOLLARS_X: f64 = -4.1;
const CHIP_X0: f64 = -4.85;
const CHIP_PITCH: f64 = 0.3;
const CHIP_Y: f64 = 0.9;
/// The coin and its lines.
const COIN_AT: Vec3 = [-5.6, -1.0, 0.0];
const COIN_R: f64 = 0.36;
const COIN_LINES_X: f64 = -4.95;
const COIN_LINE_YS: [f64; 2] = [-0.8, -1.3];
const EXAMPLE_AT: Vec3 = [-5.6, -1.7, 0.0];
const STEP_LABEL_AT: Vec3 = [-4.6, -1.0, 0.0];
/// 01_5: the first rounds' step plot.
const STEPS: Frame2 = Frame2 {
    center: [3.3, -0.3, 0.0],
    width: 5.0,
    height: 3.0,
    x: (0.0, 8.0),
    y: (6.0, 13.0),
};
const STEP_S: f64 = 0.55;
/// 01_6 to 01_8: the panel on the right.
const PANEL_X: f64 = 2.6;
const HUNDRED_Y: f64 = 1.9;
const QUESTION_Y: f64 = 1.1;
const THINK_AT: Vec3 = [2.6, -0.6, 0.0];
const SUM_Y: f64 = 0.6;
const CENT_Y: f64 = -0.2;
/// 01_9: the long run's window: 100,000 rounds, dollars −1,200 to +200,
/// clear of the shrunk cards above and the lower third below.
const LONG: Frame2 = Frame2 {
    center: [0.8, 0.15, 0.0],
    width: 10.6,
    height: 4.0,
    x: (0.0, 100_000.0),
    y: (-1200.0, 200.0),
};
const TICK_DY: f64 = 0.22;
/// Every 250th round of the run is drawn (401 points).
const THIN: usize = 250;
const RUN_S: f64 = 7.0;
const END_AT: Vec3 = [3.6, 1.7, 0.0];
const DASH: f64 = 0.12;
/// 01_10: the first rounds again, in an inset at their own scale (rounds 0
/// to [`SHORT`], dollars from the run's low to its high over them, padded),
/// in the long window's empty lower left, below the run and its trend.
const INSET_AT: Vec3 = [-2.0, -1.15, 0.0];
const INSET_W: f64 = 3.2;
const INSET_H: f64 = 1.0;
const INSET_PAD: f64 = 4.0;
const INSET_TITLE_DY: f64 = 0.22;
const INSET_LABEL_DX: f64 = 0.12;
/// Every 4th of the first rounds is drawn (251 points).
const INSET_THIN: usize = 4;
const INSET_S: f64 = 1.4;
const INSET_DASH: f64 = 0.06;
/// 01_10: the thumbnail the run leaves at the top left (the full draft's
/// chapters 03 and 04 return to it there).
const THUMB_FACTOR: f64 = 0.26;
const THUMB_AT: Vec3 = [-5.0, 1.75, 0.0];
/// The lower third's top with three lines of text (`props::lower_third`:
/// 0.2 margin, 0.3 padding, 0.36 a line, from the frame's foot).
#[cfg(test)]
const LOWER_THIRD_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 3.0 * 0.36;
/// Labels keep this far from their neighbours and the frame's edge.
const ROOM: f64 = 0.2;

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    wins: String,
    losses: String,
    heads: String,
    tails: String,
    first: Vec<i64>,
    run: model::Run,
    sum_tex: String,
    end_line: String,
    card: String,
    /// The run's lowest and highest dollars over its first [`SHORT`] rounds.
    short_low: i64,
    short_high: i64,
    inset_title: String,
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let [a, ..] = facts::coins(&PUBLISHED);
    let wins = a.exact_pct()?;
    let losses = (Ratio::ONE - a).exact_pct()?;
    if wins != get("a.hundred.wins")? || losses != get("a.hundred.losses")? {
        return Err("game A's hundred rounds disagree with its coin".into());
    }
    let run = model::player(Play::A);
    let first = run.dollars[..=FIRST].to_vec();
    if first[FIRST] <= START {
        return Err("01_5 says the first rounds climb, but this run's do not".into());
    }
    let net = run.net();
    let expected = -((RUN_ROUNDS / 100) as i64);
    if (net - expected).abs() > 100 {
        return Err(format!(
            "01_9 says about a thousand dollars down, but the run ends {net}"
        ));
    }
    if get("pub.a.mean.cents")? != "-1.00" || get("a.hundred.net")? != "-1" {
        return Err("01_8 says a cent a round, but game A's average is not".into());
    }
    if net >= 0 {
        return Err(format!("01_9 says the run sinks, but it ends {net}"));
    }
    let short = &run.dollars[..=SHORT];
    let (short_low, short_high) = (
        *short.iter().min().expect("a run has a start"),
        *short.iter().max().expect("a run has a start"),
    );
    // 01_10's inset shows luck: its swing must dwarf the cents it costs
    if short_high - short_low <= 2 * short_cost() {
        return Err(format!(
            "01_10 says luck decides a short stretch, but the first {SHORT} rounds swing only {} dollars",
            short_high - short_low
        ));
    }
    Ok(Numbers {
        heads: format!("heads: {wins} %, +1 dollar"),
        tails: format!("tails: {losses} %, {} dollar", screen("-1")),
        sum_tex: format!(r"{wins} - {losses} = -1 \text{{ dollar per 100 rounds}}"),
        // the change since the start (the plotted count ends at START + net)
        end_line: format!(
            "after {} rounds: {} dollars down",
            explainer_kit::thousands(RUN_ROUNDS as u64),
            explainer_kit::thousands(net.unsigned_abs())
        ),
        card: format!(
            "one simulated player: SplitMix64 seed {}, fixed before any run was looked at; start {START} dollars; game \
             A's exact long-run average is {} cent a round; this run ends {} dollars down",
            model::SEED,
            screen("-1"),
            explainer_kit::thousands(net.unsigned_abs()),
        ),
        short_low,
        short_high,
        inset_title: format!("first {} rounds, zoomed in", explainer_kit::thousands(SHORT as u64)),
        wins,
        losses,
        first,
        run,
    })
}

/// What a cent a round costs over the first [`SHORT`] rounds, in dollars.
fn short_cost() -> i64 {
    (SHORT / 100) as i64
}

/// 01_10's inset: the first [`SHORT`] rounds at their own scale.
fn inset(n: &Numbers) -> Frame2 {
    Frame2 {
        center: INSET_AT,
        width: INSET_W,
        height: INSET_H,
        x: (0.0, SHORT as f64),
        y: (n.short_low as f64 - INSET_PAD, n.short_high as f64 + INSET_PAD),
    }
}

fn chip_at(i: usize) -> Vec3 {
    v(CHIP_X0 + CHIP_PITCH * i as f64, CHIP_Y)
}

/// One toss: the count tweens to `to` while a chip comes or goes.
fn step_count(stage: &mut Stage<'_>, count: Mob, row: &mut Vec<Mob>, to: i64, rt: f64) -> fmn::Result<()> {
    let have = row.len() as i64;
    let chip_anim = if to > have {
        let c = chip(stage, chip_at(row.len()));
        row.push(c);
        fade_in(stage.arena_mut(), c, ORIGIN, 1.0)?
    } else if to < have {
        let c = row.pop().expect("a chip to take");
        fade_out(stage.arena_mut(), c, ORIGIN, 1.0)?
    } else {
        return Ok(());
    };
    let a = stage.prepare(chip_anim.rt(rt))?;
    let t = stage.prepare(tween(count, to as f64, rt)?)?;
    stage.play_prepared(vec![a, t])?;
    Ok(())
}

impl SceneConstruct for ThePuzzle {
    fn name(&self) -> &str {
        "01_the_puzzle"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;

        // 01_1: two cards, both losing slowly; the question between them
        let line = say(&kit, stage, "01_1")?;
        let mut cards: Vec<Mob> = Vec::new();
        for (x, (title, color)) in CARD_XS
            .iter()
            .zip([(GAME_A, palette::game_a()), (GAME_B, palette::game_b())])
        {
            cards.extend(card(&kit, stage, title, v(*x, CARD_Y), CARD_W, CARD_H, color)?);
        }
        fade_in_all(stage, &cards, 0.6)?;
        line.wait_for(stage, AT_LOSES.1)?;
        let mut loses: Vec<Mob> = Vec::new();
        for &x in &CARD_XS {
            loses.push(kit.text(stage, LOSES, 24.0, Palette::dim(), v(x, CARD_Y + LOSES_DY))?);
            loses.extend(arrow(
                stage,
                v(x + 1.4, CARD_Y + LOSES_ARROW.0),
                v(x + 1.4, CARD_Y + LOSES_ARROW.1),
                Palette::dim(),
                3.0,
            ));
        }
        fade_in_all(stage, &loses, 0.5)?;
        line.wait_for(stage, AT_SWITCH.1)?;
        let mut asking = curved_arrow(
            stage,
            v(-SWITCH_HALF, SWITCH_Y),
            v(SWITCH_HALF, SWITCH_Y),
            SWITCH_BOW,
            palette::mix(),
            4.0,
        )
        .to_vec();
        asking.push(kit.text(stage, ASK, 44.0, Palette::hero(), v(0.0, ASK_Y))?);
        fade_in_all(stage, &asking, 0.5)?;
        // the puzzle stands long enough to guess (01_1's pause intention)
        kit.hold(stage)?;
        pause(stage, 1.0)?;

        // 01_2: the cards go to the top; one player with seven dollars
        let line = say(&kit, stage, "01_2")?;
        fade_out_all(stage, &asking, 0.4)?;
        let mut kept = cards.clone();
        kept.extend(&loses);
        shrink_all(stage, &kept, SHRINK, v(0.0, CARD_Y), SHRINK_SHIFT, 0.9)?;
        line.wait_for(stage, AT_PLAYER.1)?;
        let player = token(stage, TOKEN_AT);
        let label = kit.text(stage, PLAYER, 18.0, Palette::dim(), PLAYER_LABEL_AT)?;
        let dollars_word = kit.text_left(stage, DOLLARS, 26.0, Palette::dim(), v(DOLLARS_X, COUNT_AT[1]))?;
        let mut row: Vec<Mob> = (0..START as usize).map(|i| chip(stage, chip_at(i))).collect();
        let tag = badge(&kit, stage, BADGE, true)?;
        let card_b_right = CARD_XS[1] * SHRINK + SHRINK_SHIFT[0] + CARD_W * SHRINK / 2.0;
        keep_between(
            stage,
            tag[0],
            card_b_right + ROOM,
            FRAME_X_RADIUS - ROOM,
            "the coin badge",
        )?;
        let mut arriving = vec![player, label, dollars_word];
        arriving.extend(&row);
        arriving.extend(tag);
        fade_in_all(stage, &arriving, 0.6)?;
        let count_value = stage.add_value_tracker(START as f64);
        let count = kit.live_number(stage, count_value, 0, false, 34.0, palette::player(), COUNT_AT, LEFT)?;

        // 01_3: a toss, a dollar up or down (an example, not a draw)
        let line = say(&kit, stage, "01_3")?;
        let [disc, blank] = coin(&kit, stage, COIN_AT, COIN_R, Palette::dim(), UNTOSSED)?;
        let example = kit.text(stage, EXAMPLE, 18.0, Palette::dim(), EXAMPLE_AT)?;
        fade_in_all(stage, &[disc, blank, example], 0.4)?;
        line.wait_for(stage, AT_HEADS.1)?;
        let on_coin = explainer_kit::hex(explainer_kit::BACKGROUND);
        let heads_face = kit.text(stage, HEADS_FACE, 22.0, on_coin, COIN_AT)?;
        let plus = kit.text(stage, PLUS_ONE, 30.0, palette::good(), STEP_LABEL_AT)?;
        fade_out_all(stage, &[blank], 0.15)?;
        fade_in_all(stage, &[heads_face, plus], 0.25)?;
        step_count(stage, count_value, &mut row, START + 1, 0.4)?;
        line.wait_for(stage, AT_TAILS.1)?;
        let tails_face = kit.text(stage, TAILS_FACE, 22.0, on_coin, COIN_AT)?;
        let minus = kit.text(stage, MINUS_ONE, 30.0, Palette::dim(), STEP_LABEL_AT)?;
        fade_out_all(stage, &[heads_face, plus], 0.15)?;
        fade_in_all(stage, &[tails_face, minus], 0.25)?;
        step_count(stage, count_value, &mut row, START, 0.4)?;

        // 01_4: game A's coin, and its two chances
        let line = say(&kit, stage, "01_4")?;
        fade_out_all(stage, &[tails_face, minus, example], 0.3)?;
        let to_a = disc
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.5),
                ..AnimateArgs::default()
            })?
            .set_color(palette::game_a())?;
        together(stage, vec![to_a])?;
        // card A's title answers "which game"
        flash_all(stage, &[cards[1]], 0.6)?;
        line.wait_for(stage, AT_CHANCES.1)?;
        let chances = [
            kit.text_left(stage, &n.heads, 24.0, palette::good(), v(COIN_LINES_X, COIN_LINE_YS[0]))?,
            kit.text_left(stage, &n.tails, 24.0, Palette::dim(), v(COIN_LINES_X, COIN_LINE_YS[1]))?,
        ];
        for m in chances {
            keep_between(
                stage,
                m,
                -FRAME_X_RADIUS + ROOM,
                STEPS.left() - ROOM,
                "game A's chances",
            )?;
        }
        fade_in_all(stage, &chances, 0.5)?;

        // 01_5: the player's first rounds of game A, one toss at a time
        say(&kit, stage, "01_5")?;
        let (left, right, bottom, top) = (STEPS.left(), STEPS.right(), STEPS.bottom(), STEPS.top());
        let corner = polyline([v(left, top), v(left, bottom), v(right, bottom)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        let axes = [
            stage.arena_mut().add(corner),
            kit.text(
                stage,
                ROUNDS,
                20.0,
                Palette::dim(),
                v((left + right) / 2.0, bottom - 0.35),
            )?,
            kit.text(stage, DOLLARS, 20.0, Palette::dim(), v(left, top + 0.25))?,
            kit.text(stage, "0", 18.0, Palette::dim(), v(left, bottom - 0.2))?,
            kit.text(stage, &FIRST.to_string(), 18.0, Palette::dim(), v(right, bottom - 0.2))?,
            kit.text(
                stage,
                &START.to_string(),
                18.0,
                Palette::dim(),
                v(left - 0.25, STEPS.p(0.0, START as f64)[1]),
            )?,
        ];
        fade_in_all(stage, &axes, 0.4)?;
        let mut steps: Vec<Mob> = Vec::with_capacity(FIRST);
        for k in 0..FIRST {
            let (a, b) = (n.first[k], n.first[k + 1]);
            let seg = polyline([STEPS.p(k as f64, a as f64), STEPS.p((k + 1) as f64, b as f64)])
                .style(line_style(palette::game_a(), 3.0))
                .build();
            let seg = stage.arena_mut().add(seg);
            fade_in_all(stage, &[seg], 0.15)?;
            step_count(stage, count_value, &mut row, b, STEP_S - 0.15)?;
            steps.push(seg);
        }

        // 01_6: think in averages; the panel stays right of the chips
        let line = say(&kit, stage, "01_6")?;
        let panel = (
            chip_at(row.len().saturating_sub(1))[0] + CHIP_R + ROOM,
            FRAME_X_RADIUS - ROOM,
        );
        let mut leaving = axes.to_vec();
        leaving.extend(&steps);
        fade_out_all(stage, &leaving, 0.4)?;
        line.wait_for(stage, AT_HUNDRED.1)?;
        let hundred = kit.text(
            stage,
            &format!("100 rounds: {} wins, {} losses", n.wins, n.losses),
            28.0,
            Palette::ink(),
            v(PANEL_X, HUNDRED_Y),
        )?;
        keep_between(stage, hundred, panel.0, panel.1, "the hundred rounds")?;
        fade_in_all(stage, &[hundred], 0.5)?;

        // 01_7: what does a round cost? (a prediction; nothing answers it yet)
        say(&kit, stage, "01_7")?;
        let question = kit.text(stage, QUESTION, 26.0, Palette::hero(), v(PANEL_X, QUESTION_Y))?;
        keep_between(stage, question, panel.0, panel.1, "the question")?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 01_8: a dollar per hundred rounds, a cent a round
        let line = say(&kit, stage, "01_8")?;
        fade_out_all(stage, &[q, question], 0.3)?;
        let sum = kit.tex(stage, &n.sum_tex, 32.0, Palette::ink(), &[], v(PANEL_X, SUM_Y))?;
        keep_between(stage, sum, panel.0, panel.1, "the sum")?;
        fade_in_all(stage, &[sum], 0.5)?;
        line.wait_for(stage, AT_CENT.1)?;
        let cent = kit.text(stage, CENT_LINE, 28.0, palette::bad(), v(PANEL_X, CENT_Y))?;
        keep_between(stage, cent, panel.0, panel.1, "the cent a round")?;
        fade_in_all(stage, &[cent], 0.4)?;

        // 01_9: the same player, a hundred thousand rounds of game A
        let line = say(&kit, stage, "01_9")?;
        let mut leaving = vec![hundred, sum, cent, label, dollars_word, count, disc];
        leaving.extend(chances);
        leaving.append(&mut row);
        fade_out_all(stage, &leaving, 0.5)?;
        let start_at = LONG.p(0.0, START as f64);
        together(stage, vec![fly(player, start_at, 0.8)?])?;
        let (left, right, bottom, top) = (LONG.left(), LONG.right(), LONG.bottom(), LONG.top());
        let thousand_down = START - (RUN_ROUNDS / 100) as i64;
        let corner = polyline([v(left, top), v(left, bottom), v(right, bottom)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        let mut window = vec![
            stage.arena_mut().add(corner),
            kit.text(stage, "0", 18.0, Palette::dim(), v(left, bottom - TICK_DY))?,
            kit.text(
                stage,
                &format!("{} {ROUNDS}", explainer_kit::thousands(RUN_ROUNDS as u64)),
                18.0,
                Palette::dim(),
                v(right - 0.9, bottom - TICK_DY),
            )?,
            kit.text(stage, DOLLARS, 18.0, Palette::dim(), v(left, top + TICK_DY))?,
            kit.text(stage, "0", 18.0, Palette::dim(), v(left - 0.5, LONG.p(0.0, 0.0)[1]))?,
            kit.text(
                stage,
                &screen_int(-1000),
                18.0,
                Palette::dim(),
                v(left - 0.75, LONG.p(0.0, -1000.0)[1]),
            )?,
        ];
        // a cent a round from the start: the average's line
        let trend = dashed(
            stage,
            start_at,
            LONG.p(RUN_ROUNDS as f64, thousand_down as f64),
            DASH,
            Palette::dim(),
            2.0,
        );
        window.extend(&trend);
        fade_in_all(stage, &window, 0.5)?;
        line.wait_for(stage, AT_RUN.1)?;
        let points = thinned(&n.run.dollars, THIN);
        let (run_line, anchor, progress) = growing_run(stage, &LONG, &points, palette::game_a(), Some(player))?;
        fade_in_all(stage, &[anchor, run_line], 0.1)?;
        draw_run(stage, progress, RUN_S)?;
        let end = kit.text(stage, &n.end_line, 24.0, palette::game_a(), END_AT)?;
        keep_between(stage, end, LONG.left(), FRAME_X_RADIUS - ROOM, "the end readout")?;
        let k01 = precisely(&kit, stage, &n.card)?;
        fade_in_all(stage, &[end, k01], 0.5)?;
        let card_up = now(stage);

        // 01_10: luck decides a short stretch, the cent a round the long run
        let line = say(&kit, stage, "01_10")?;
        line.wait_for(stage, AT_LUCK.1)?;
        // the first rounds, a sliver at this scale: outlined on the run, then
        // drawn again in the inset at their own scale, beside what the cent a
        // round costs over them
        let short_x = LONG.p(SHORT as f64, 0.0)[0];
        let (y_lo, y_hi) = (LONG.p(0.0, n.short_low as f64)[1], LONG.p(0.0, n.short_high as f64)[1]);
        let w = (short_x - left).max(0.3);
        let outline = rect(
            v(left - 0.1, y_lo - 0.15),
            w + 0.2,
            (y_hi - y_lo) + 0.3,
            line_style(Palette::hero(), 3.0),
        )?;
        let mut zoomed = vec![stage.arena_mut().add(outline)];
        let zoom = inset(&n);
        let frame = rect(
            v(zoom.left(), zoom.bottom()),
            zoom.width,
            zoom.height,
            line_style(Palette::hero(), 2.0),
        )?;
        zoomed.push(stage.arena_mut().add(frame));
        zoomed.extend(dashed(
            stage,
            v(left + w + 0.1, y_lo - 0.15),
            v(zoom.left(), zoom.top()),
            INSET_DASH,
            Palette::hero(),
            1.5,
        ));
        let title = kit.text(
            stage,
            &n.inset_title,
            18.0,
            Palette::hero(),
            v(zoom.center[0], zoom.top() + INSET_TITLE_DY),
        )?;
        keep_between(stage, title, left + ROOM, FRAME_X_RADIUS - ROOM, "the inset's title")?;
        zoomed.push(title);
        for d in [n.short_high, n.short_low] {
            let m = kit.text(stage, &screen_int(d), 16.0, Palette::dim(), ORIGIN)?;
            stage.move_to(m, v(zoom.left() - INSET_LABEL_DX, zoom.p(0.0, d as f64)[1]), RIGHT);
            keep_between(stage, m, left + ROOM / 2.0, zoom.left(), "the inset's dollars")?;
            zoomed.push(m);
        }
        zoomed.extend(dashed(
            stage,
            zoom.p(0.0, START as f64),
            zoom.p(SHORT as f64, (START - short_cost()) as f64),
            INSET_DASH,
            Palette::dim(),
            1.5,
        ));
        fade_in_all(stage, &zoomed, 0.4)?;
        let short = thinned(&n.run.dollars[..=SHORT], INSET_THIN);
        let (short_line, short_anchor, short_progress) = growing_run(stage, &zoom, &short, palette::game_a(), None)?;
        fade_in_all(stage, &[short_anchor, short_line], 0.1)?;
        draw_run(stage, short_progress, INSET_S)?;
        zoomed.extend([short_line, short_anchor]);
        line.wait_for(stage, AT_LONG_RUN.1)?;
        flash_all(stage, &trend, 0.8)?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&n.card))?;
        stage.clear_updaters(short_anchor, true);
        zoomed.push(k01);
        fade_out_all(stage, &zoomed, 0.4)?;
        // the run shrinks to the thumbnail the full draft returns to
        let mut picture = window.clone();
        picture.extend([run_line, end]);
        stage.clear_updaters(anchor, true);
        let shift = v(THUMB_AT[0] - LONG.center[0], THUMB_AT[1] - LONG.center[1]);
        let mut moves = Vec::with_capacity(picture.len() + 1);
        for &m in &picture {
            moves.push(
                m.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(1.0),
                        ..AnimateArgs::default()
                    })?
                    .scale_about(THUMB_FACTOR, Some(LONG.center), None)?
                    .shift(shift)?,
            );
        }
        moves.push(fade_to(player, 0.0, 1.0)?);
        together(stage, moves)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::THINK_S;
    use super::*;

    /// The chapter's words are true of its numbers: the first rounds climb,
    /// the long run ends within a hundred dollars of a thousand down, and
    /// every line the screen writes carries the facts' values.
    #[test]
    fn the_puzzle_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.first, [7, 8, 9, 8, 9, 10, 11, 12, 11]);
        assert_eq!((n.wins.as_str(), n.losses.as_str()), ("49.5", "50.5"));
        assert_eq!(n.heads, "heads: 49.5 %, +1 dollar");
        assert_eq!(n.tails, "tails: 50.5 %, \u{2212}1 dollar");
        assert_eq!(n.run.net(), -1062);
        // the readout is the change since the start, as the card says; the
        // plotted count itself ends at 7 − 1,062 = −1,055
        assert_eq!(*n.run.dollars.last().unwrap(), -1055);
        assert_eq!(n.end_line, "after 100,000 rounds: 1,062 dollars down");
        assert!(
            n.card.contains("seed 1999") && n.card.contains("1,062 dollars down"),
            "{}",
            n.card
        );
        assert_eq!(n.sum_tex, r"49.5 - 50.5 = -1 \text{ dollar per 100 rounds}");
        // the inset's first 1,000 rounds swing 71 dollars; the cent a round
        // costs 10 over them
        assert_eq!((n.short_low, n.short_high, short_cost()), (-58, 13, 10));
        assert_eq!(n.inset_title, "first 1,000 rounds, zoomed in");
        // no coin chances on the badge before chapter 02 introduces them
        assert!(!BADGE.chars().any(|c| c.is_ascii_digit()), "{BADGE}");
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the chips stop short of the step plot,
    /// the long run's window clears the shrunk cards above and the lower
    /// third below, and the thumbnail stays inside the frame.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let n = numbers().unwrap();
        let most = *n.first.iter().max().unwrap();
        let widest = CHIP_X0 + CHIP_PITCH * (most - 1) as f64 + CHIP_R;
        assert!(widest < STEPS.left() - 0.3, "{widest}");
        let shrunk_bottom = (CARD_Y - CARD_H / 2.0 - CARD_Y) * SHRINK + CARD_Y + SHRINK_SHIFT[1];
        let shrunk_left = (CARD_XS[0] - CARD_W / 2.0) * SHRINK + SHRINK_SHIFT[0];
        assert!(shrunk_left > -FRAME_X_RADIUS && shrunk_bottom + CARD_H * SHRINK < FRAME_Y_RADIUS);
        assert!(
            LONG.top() + TICK_DY + 0.15 < shrunk_bottom,
            "{} vs {shrunk_bottom}",
            LONG.top()
        );
        assert!(LONG.bottom() - TICK_DY - 0.15 > LOWER_THIRD_TOP, "{}", LONG.bottom());
        assert!(LONG.left() - 0.75 - 0.4 > -FRAME_X_RADIUS && LONG.right() < FRAME_X_RADIUS);
        let (half_w, half_h) = (LONG.width * THUMB_FACTOR / 2.0, LONG.height * THUMB_FACTOR / 2.0);
        assert!(THUMB_AT[0] - half_w > -FRAME_X_RADIUS && THUMB_AT[1] + half_h < shrunk_bottom);
        // the runs fit their windows
        assert!(n.run.low() as f64 > LONG.y.0 && (n.run.high() as f64) < LONG.y.1);
        assert!(
            n.first
                .iter()
                .all(|&d| (d as f64) > STEPS.y.0 && (d as f64) < STEPS.y.1)
        );
        // 01_10's inset sits inside the long window, its labels clear of the
        // dollars axis, and the drawn run never comes down into it or its
        // title (0.13 a title's half height, 0.5 its dollar labels' width)
        let zoom = inset(&n);
        assert!(
            zoom.left() - INSET_LABEL_DX - 0.5 > LONG.left() + 0.1,
            "{}",
            zoom.left()
        );
        assert!(zoom.bottom() > LONG.bottom() + 0.1 && zoom.right() < LONG.right());
        let ceiling = zoom.top() + INSET_TITLE_DY + 0.13;
        for (k, d) in thinned(&n.run.dollars, THIN) {
            let [x, y, _] = LONG.p(k, d);
            if x > zoom.left() - INSET_LABEL_DX - 0.5 && x < zoom.right() + 0.1 {
                assert!(
                    y > ceiling + 0.05,
                    "round {k} at {d} dollars reaches the inset ({y:.2})"
                );
            }
        }
        // the inset's own scale holds its run, and the outline it zooms is
        // more than a sliver of the long plot
        assert!(
            n.run.dollars[..=SHORT]
                .iter()
                .all(|&d| (d as f64) > zoom.y.0 && (d as f64) < zoom.y.1)
        );
        assert!(zoom.p(0.0, n.short_high as f64)[1] - zoom.p(0.0, n.short_low as f64)[1] > 0.8);
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` guard meets the true text
    /// widths): every line of chapter 01 is said once, in order, and the
    /// prediction (01_7) gets its silent think time before the answer (01_8).
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = ThePuzzle { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("01_the_puzzle: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=10).map(|k| format!("01_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let asked_end = at("01_7") + narrator.length("01_7").unwrap();
        assert!(
            at("01_8") - asked_end >= THINK_S,
            "{} s to think",
            at("01_8") - asked_end
        );
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end > at("01_10"), "the chapter ends after its last line: {end}");
    }
}
