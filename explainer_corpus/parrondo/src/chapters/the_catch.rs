//! Chapter18 returns to the same two games, then changes only which coin
//! is used at each corner. The last prediction is held before its verdict.
//! Crowd areas interpolate between computed long-run shares; this is a
//! comparison of equilibria, not a simulation of transient convergence.

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    coin, fade_in_all, fade_out_all, fail, keep_between, now, pause, read_s, say, screen, think, together, token,
    wait_until,
};
use super::net_flow::{chance_label, puddles, stream};
use super::three_corners::{Which, chance_labels, chances, corner, triangle};
use crate::facts;
use crate::model::{Game, PUBLISHED, Ratio};
use crate::palette;

pub struct TheCatch {
    pub kit: Kit,
}

const EXACT: &str = "fixed coins with chances strictly between zero and one; one dollar forward or back. The sign of the long-run gain is the sign of the forward product minus the backward product; a tie gives zero";
const SEARCH: &str = "Parrondo's paradox · Markov chain · flashing ratchet";
const RECEIPTS: [&str; 5] = [
    "rules: Harmer & Abbott",
    "Nature 402, 864 (1999)",
    "exact means and long-run law: Ethier & Lee",
    "EJP 14, 1827–1862 (2009)",
    "numbers: src/facts.rs and oracle.py",
];

/// The loss/win labels and crowd readouts must be the existing native
/// facts, recomputed from the game used by the actual scene.
struct Numbers {
    a: Game,
    b: Game,
    mix: Game,
    reverse: Game,
    swapped: Game,
    bad_share: String,
    mix_share: String,
    b_win: String,
    loss: String,
    gain: String,
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let a = PUBLISHED.game_a();
    let b = PUBLISHED.game_b();
    let mix = PUBLISHED.mix();
    let reverse = PUBLISHED.b_at_zero();
    let swapped = PUBLISHED.a_at_zero();
    for (key, actual) in [
        ("pub.a.mean.cents", a.mean().cents(2)),
        ("pub.reverse.mean.cents", reverse.mean().cents(1)),
        ("pub.swapped.mean.cents", swapped.mean().cents(1)),
        ("pub.mix.b_rounds.win.pct", b.win_at(&mix.shares()).pct(1)),
    ] {
        if f.get(key) != Some(actual.as_str()) {
            return Err(format!("closing scene disagrees with {key}"));
        }
    }
    if a.mean() != Ratio::new(-1, 100)
        || reverse.mean() != Ratio::new(-2_490_243, 9_016_300)
        || swapped.mean() != Ratio::new(2_902_797, 9_230_300)
        || (0..3).any(|i| reverse.chance(i) >= Ratio::new(1, 2))
    {
        return Err("the closing puzzle's words no longer describe its coins".into());
    }
    Ok(Numbers {
        a,
        b,
        mix,
        reverse,
        swapped,
        bad_share: b.shares()[0].pct(1),
        mix_share: mix.shares()[0].pct(1),
        b_win: b.win_at(&mix.shares()).pct(1),
        loss: screen(&reverse.mean().cents(1)),
        gain: screen(&swapped.mean().cents(1)),
    })
}

/// Left panel deliberately stays separate from the persistent right triangle.
fn words(kit: &Kit, stage: &mut Stage<'_>, text: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    let m = kit.text_left(stage, text, 23.0, color, v(-6.5, y))?;
    keep_between(stage, m, -6.51, -0.6, "closing panel")?;
    Ok(m)
}

fn panel(kit: &Kit, stage: &mut Stage<'_>, rows: &[&str], color: Srgb) -> fmn::Result<Vec<Mob>> {
    rows.iter()
        .enumerate()
        .map(|(i, s)| words(kit, stage, s, 2.4 - i as f64 * 0.55, color))
        .collect()
}

/// Each label carries the coin's role, not the old game B label color.
fn labels(kit: &Kit, stage: &mut Stage<'_>, game: &Game, colors: [Srgb; 3]) -> fmn::Result<Vec<Mob>> {
    let c = chances(game).map_err(fail)?;
    let mut out = Vec::new();
    for forward in [true, false] {
        for e in 0..3 {
            let i = if forward { e } else { (e + 1) % 3 };
            let pct = if forward { &c.heads[i] } else { &c.tails[i] };
            out.push(chance_label(kit, stage, &format!("{pct} %"), colors[i], e, forward)?);
        }
    }
    Ok(out)
}

/// Highlight a three-step cycle in its already established direction.
fn cycle(stage: &mut Stage<'_>, forward: bool, width: f64, color: Srgb) -> Vec<Mob> {
    (0..3).map(|e| stream(stage, e, forward, width, color, 0.85)).collect()
}

impl SceneConstruct for TheCatch {
    fn name(&self) -> &str {
        "18_the_catch"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;
        let (ink, dim, red, blue, gold, green) = (
            Palette::ink(),
            Palette::dim(),
            palette::bad(),
            palette::game_a(),
            palette::mix(),
            palette::good(),
        );

        let line = say(&kit, stage, "18_1")?;
        let a = triangle(&kit, stage, Which::A)?;
        let b = triangle(&kit, stage, Which::B)?;
        let a_labels = chance_labels(&kit, stage, Which::A, &chances(&n.a).map_err(fail)?)?;
        let mut a_parts = a.parts();
        a_parts.extend(a_labels);
        let player = token(stage, corner(Which::A.at(), 1));
        a_parts.push(player);
        fade_in_all(stage, &a_parts, 0.5)?;
        line.wait_for(stage, "loses a cent a round")?;
        let cost = kit.text(stage, "−1 cent a round", 25.0, red, v(-3.9, -2.95))?;
        a_parts.push(cost);
        fade_in_all(stage, &[cost], 0.4)?;

        let line = say(&kit, stage, "18_2")?;
        fade_in_all(stage, &b.parts(), 0.5)?;
        let mut right_labels = labels(&kit, stage, &n.b, [red, green, green])?;
        fade_in_all(stage, &right_labels, 0.4)?;
        line.wait_for(stage, "piles up at corner zero")?;
        let crowd = puddles(stage, n.b.shares().map(|x| x.to_f64()))?;
        let mut share = kit.text(
            stage,
            &format!("long-run corner 0: {} %", n.bad_share),
            24.0,
            ink,
            v(2.3, 3.55),
        )?;
        let trace = [
            stream(stage, 2, true, 5.0, gold, 0.8),
            stream(stage, 2, false, 5.0, gold, 0.8),
        ];
        fade_in_all(stage, &[share, trace[0], trace[1]], 0.4)?;
        kit.hold(stage)?;
        pause(stage, 1.0)?;

        let line = say(&kit, stage, "18_3")?;
        fade_out_all(stage, &a_parts, 0.4)?;
        fade_out_all(stage, &trace, 0.3)?;
        let toss = coin(&kit, stage, v(-5.9, 2.45), 0.34, gold, "?")?;
        let mut left = panel(
            &kit,
            stage,
            &["Fair choice: A or B each round", "same coins; different crowd"],
            ink,
        )?;
        // Keep the fair-choice coin above, away from its text rows.
        stage.move_to(toss[0], v(-5.9, 3.35), ORIGIN);
        stage.move_to(toss[1], v(-5.9, 3.35), ORIGIN);
        left.extend(toss);
        fade_in_all(stage, &left, 0.4)?;
        line.wait_for(stage, "push the crowd off corner zero")?;
        together(stage, crowd.to(n.mix.shares().map(|x| x.to_f64()), 1.5)?)?;
        let next_share = kit.text(
            stage,
            &format!("long-run corner 0: {} %", n.mix_share),
            24.0,
            ink,
            v(2.3, 3.55),
        )?;
        fade_out_all(stage, &[share], 0.25)?;
        fade_in_all(stage, &[next_share], 0.25)?;
        share = next_share;
        line.wait_for(stage, "now win more than half the time")?;
        let b_win = words(&kit, stage, &format!("B rounds: {} % wins", n.b_win), 0.8, green)?;
        left.push(b_win);
        fade_in_all(stage, &[b_win], 0.4)?;
        kit.hold(stage)?;
        pause(stage, 2.0)?;

        let line = say(&kit, stage, "18_4")?;
        fade_out_all(stage, &left, 0.4)?;
        fade_out_all(stage, &[share, b.title], 0.3)?;
        left = panel(
            &kit,
            stage,
            &["Switch by the player's corner", "B at 0; A at 1 and 2"],
            ink,
        )?;
        line.wait_for(stage, "Play game B only")?;
        fade_in_all(stage, &left, 0.4)?;
        fade_out_all(stage, &right_labels, 0.3)?;
        right_labels = labels(&kit, stage, &n.reverse, [red, blue, blue])?;
        fade_in_all(stage, &right_labels, 0.4)?;
        // Remove the crowd from the prediction: only the rule's coins,
        // not its computed stationary distribution, should be available.
        together(stage, crowd.to([0.0; 3], 0.4)?)?;

        say(&kit, stage, "18_5")?;
        let q = think(&kit, stage, v(-3.9, -0.4))?;

        let line = say(&kit, stage, "18_6")?;
        line.wait_for(stage, "It loses, and badly")?;
        fade_out_all(stage, &[q], 0.3)?;
        let loss = words(&kit, stage, &format!("{} cents a round", n.loss), 0.55, red)?;
        left.push(loss);
        fade_in_all(stage, &[loss], 0.4)?;
        line.wait_for(stage, "Every coin around the triangle")?;
        let mut bands = cycle(stage, true, 3.0, dim);
        bands.extend(cycle(stage, false, 7.0, red));
        fade_in_all(stage, &bands, 0.4)?;
        together(stage, crowd.to(n.reverse.shares().map(|x| x.to_f64()), 1.0)?)?;
        kit.hold(stage)?;
        pause(stage, 2.0)?;

        let line = say(&kit, stage, "18_7")?;
        line.wait_for(stage, "Swap the roles")?;
        fade_out_all(stage, &left, 0.4)?;
        fade_out_all(stage, &bands, 0.3)?;
        left = panel(&kit, stage, &["Swap the roles", "A at 0; B at 1 and 2"], ink)?;
        fade_in_all(stage, &left, 0.4)?;
        fade_out_all(stage, &right_labels, 0.3)?;
        right_labels = labels(&kit, stage, &n.swapped, [blue, green, green])?;
        fade_in_all(stage, &right_labels, 0.4)?;
        // Recolor this same corner beneath the crowd; a new foreground
        // disc would obscure its area and break the object's identity.
        together(stage, vec![b.discs[0].animate().set_fill(Some(blue), None)?])?;
        together(stage, crowd.to(n.swapped.shares().map(|x| x.to_f64()), 1.0)?)?;
        line.wait_for(stage, "win about thirty-one cents")?;
        let gain = words(&kit, stage, &format!("{} cents a round", n.gain), 0.55, gold)?;
        left.push(gain);
        fade_in_all(stage, &[gain], 0.4)?;
        bands = cycle(stage, true, 7.0, gold);
        bands.extend(cycle(stage, false, 3.0, dim));
        fade_in_all(stage, &bands, 0.4)?;
        kit.hold(stage)?;
        pause(stage, 2.0)?;

        let line = say(&kit, stage, "18_8")?;
        fade_out_all(stage, &left, 0.4)?;
        left = panel(&kit, stage, &["win × win × win", "versus", "lose × lose × lose"], ink)?;
        line.wait_for(stage, "Win, win, win")?;
        fade_in_all(stage, &left, 0.4)?;
        let exact = precisely(&kit, stage, EXACT)?;
        fade_in_all(stage, &[exact], 0.4)?;
        let exact_read = now(stage) + read_s(EXACT);
        line.wait_for(stage, "If the two products tie")?;
        let tie = words(&kit, stage, "if products tie: zero long-run gain", -0.1, ink)?;
        fade_in_all(stage, &[tie], 0.4)?;
        wait_until(stage, exact_read)?;
        kit.hold(stage)?;
        fade_out_all(stage, &[exact], 0.4)?;

        let line = say(&kit, stage, "18_9")?;
        line.wait_for(stage, "To look further")?;
        fade_all(stage, 0.5)?;
        let names = lower_third(&kit, stage, "names to search:", SEARCH)?;
        let receipt = panel(&kit, stage, &RECEIPTS, ink)?;
        let mut ending = receipt;
        ending.push(names);
        fade_in_all(stage, &ending, 0.5)?;
        let budget = read_s(SEARCH) + RECEIPTS.iter().map(|x| read_s(x)).sum::<f64>();
        let until = now(stage) + budget;
        kit.hold(stage)?;
        wait_until(stage, until)?;
        pause(stage, 1.0)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_rules_use_the_same_coins_in_different_places() {
        let n = numbers().unwrap();
        assert_eq!(n.reverse.up, [19, 99, 99]);
        assert_eq!(n.swapped.up, [99, 149, 149]);
        assert_eq!(n.loss, "−27.6");
        assert_eq!(n.gain, "+31.4");
        assert_eq!(
            (n.bad_share.as_str(), n.mix_share.as_str(), n.b_win.as_str()),
            ("38.4", "34.5", "52.1")
        );
        let (f, b) = n.reverse.products();
        assert!(f < b);
        let (f, b) = n.swapped.products();
        assert!(f > b);
        assert_eq!(Game { up: [1, 1, 1], den: 2 }.mean(), Ratio::ZERO);
    }
}
