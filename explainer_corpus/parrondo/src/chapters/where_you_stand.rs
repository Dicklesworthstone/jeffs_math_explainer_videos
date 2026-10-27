//! Chapter 07, where you stand decides game B. The picture 06 left: game
//! A's triangle on the left and B's on the right, every arrow with its
//! published chance, the teal token on B's corner 2 and the counter at 8
//! dollars. Six dollars and six hundred both leave nothing over and both
//! slide into B's corner 0: game B plays them the same way, so what decides
//! it is how often the player stands at each corner; a "precisely" card
//! (K03) says the capital may be any whole number, debts included, and play
//! never stops (07_1). Playing only game A for a long time, what share of
//! the rounds does the player spend at corner 0? (07_2, a prediction) One
//! third: "1/3" in each of A's corners, and under A's triangle the player's
//! own game A run, 33.5 % of its 100,000 rounds at corner 0 (07_3). Under
//! B's triangle the player's game B run, 38.4 %; B's corner 0 chances are
//! outlined, the ones that differ (07_4). Rounder numbers: the three coins
//! at the right with their chances, each with half a point shaved off; the
//! half point goes back (+0.5 rises from each), and the badge turns from
//! "published coins" to the outlined "round-number coins" (07_5). The
//! arrows relabel, A's to a fair 50 % and B's to 10 % and 75 % (07_6).
//! These round coins are the worked example; with equal thirds B would win
//! 53.3 % of its rounds, the 03_2 sum with the new coins (07_7, held).
//!
//! Occupancy is only where the same player spends rounds: A's "1/3" is its
//! long-run limit and B's 38.4 % is its finite run's count; no weight of B's corners
//! is shown (that is 08 onward). Every count, chance and readout comes from
//! `crate::model` and `crate::facts`, and the scene refuses to draw if the
//! words would stop being true of them. No text is recoloured or indicated
//! in place: a changed number is replaced, a pointed-at label is outlined.
//! The scene ends as 08 starts: both triangles, the token on B's corner 2
//! and 8 dollars.

use explainer_kit::props::precisely;
use explainer_kit::{Facts, Kit, Palette, Timed, fade_all, rect, thousands, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    BADGE_AT, badge, coin, fade_in_all, fade_out_all, fail, flash_all, keep_between, line_style, now, pause, read_s,
    say, screen_int, think, token, wait_until,
};
use super::looks_hopeless::{EDGE, ROOM, badge_text, full_badge};
use super::three_corners::{
    A_AT, B_AT, CORNER_R, Chances, TITLE_DY, Which, chance_labels, chances, corner, corner_of, counter, keep_apart,
    printed_remainder, remainder_line, remainder_text, swap, triangle, walk,
};
use crate::facts;
use crate::model::{DEN, HANDICAP, MODULUS, PUBLISHED, ROUND, RUN_ROUNDS, Ratio};
use crate::palette;

pub struct WhereYouStand {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const READOUT_HEAD: &str = "the player's";
const READOUT_TAIL: &str = "at corner 0";
const ROUND_BADGE: &str = "round-number coins";
const NO_HANDICAP: &str = "no handicap";
const A_FACE: &str = "A";
const BAD_FACE: &str = "bad";
const GOOD_FACE: &str = "good";
const IF_LINE: &str = "with equal thirds at every corner:";
const WOULD_WIN: &str = "B would win";

/// Where each picture lands: (line, the phrase it waits for).
const AT_SIX: (&str, &str) = ("07_1", "six dollars");
const AT_SIX_HUNDRED: (&str, &str) = ("07_1", "six hundred");
const AT_SAME: (&str, &str) = ("07_1", "plays the same way");
const AT_STAND: (&str, &str) = ("07_1", "stand at each corner");
const AT_ONLY_A: (&str, &str) = ("07_2", "play only game A");
const AT_THIRD: (&str, &str) = ("07_3", "One third");
const AT_EQUALLY: (&str, &str) = ("07_3", "share the rounds equally");
const AT_A_RUN: (&str, &str) = ("07_3", "game A run spent");
const AT_NOT_ALIKE: (&str, &str) = ("07_4", "does not treat its corners alike");
const AT_B_RUN: (&str, &str) = ("07_4", "more than thirty-eight percent");
const AT_ROUNDER: (&str, &str) = ("07_5", "make the numbers rounder");
const AT_SHAVED: (&str, &str) = ("07_5", "half a point shaved off");
const AT_BACK: (&str, &str) = ("07_5", "Put the half point back");
const AT_FAIR: (&str, &str) = ("07_6", "becomes a fair coin");
const AT_TEN: (&str, &str) = ("07_6", "a ten percent coin");
const AT_SEVENTY_FIVE: (&str, &str) = ("07_6", "seventy-five percent coin");
const AT_EXAMPLE: (&str, &str) = ("07_7", "our worked example");
const AT_THIRDS: (&str, &str) = ("07_7", "With equal thirds");
const AT_WOULD: (&str, &str) = ("07_7", "would now win");
#[cfg(test)]
const CUES: [(&str, &str); 19] = [
    AT_SIX,
    AT_SIX_HUNDRED,
    AT_SAME,
    AT_STAND,
    AT_ONLY_A,
    AT_THIRD,
    AT_EQUALLY,
    AT_A_RUN,
    AT_NOT_ALIKE,
    AT_B_RUN,
    AT_ROUNDER,
    AT_SHAVED,
    AT_BACK,
    AT_FAIR,
    AT_TEN,
    AT_SEVENTY_FIVE,
    AT_EXAMPLE,
    AT_THIRDS,
    AT_WOULD,
];

// ------------------------------------------------------------------ layout

/// The column between the two triangles' top corners, where 07_1's examples,
/// 07_2's question and 07_7's sum stand; text there keeps [`ROOM`] clear of
/// both corner-0 discs.
const MID_X: f64 = (A_AT[0] + B_AT[0]) / 2.0;
const MID_LEFT: f64 = A_AT[0] + CORNER_R + ROOM;
const MID_RIGHT: f64 = B_AT[0] - CORNER_R - ROOM;
const SIX_Y: f64 = 2.75;
const SIX_HUNDRED_Y: f64 = 2.2;
const THINK_AT: Vec3 = [MID_X, 2.3, 0.0];
const IF_Y: f64 = 2.85;
const SUM_Y: f64 = 2.3;
const WOULD_Y: f64 = 1.7;
/// Each run's readout, under its triangle's title: two lines this far
/// below the triangle's centre.
const READOUT_DY: [f64; 2] = [TITLE_DY + 0.5, TITLE_DY + 0.9];
/// The coins at the right, top to bottom A's, the bad and the good, each
/// with its chance to the right of it.
const LEGEND_X: f64 = 5.35;
const LEGEND_R: f64 = 0.36;
const LEGEND_YS: [f64; 3] = [2.55, 1.7, 0.85];
const LEGEND_NUM_X: f64 = 5.85;
/// "+0.5" starts this far above its chance and rises this much more.
const PLUS_DY: f64 = 0.28;
const PLUS_RISE: f64 = 0.25;
/// 07_1's examples shrink to this as they slide into corner 0.
const INTO_SCALE: f64 = 0.4;
/// An outline keeps this far around what it points at.
const PAD: f64 = 0.07;
/// Seconds: the two runs' readouts side by side before the coins change,
/// the turn after "Put the half point back", and the thinking hold after
/// the last line.
const COMPARE_S: f64 = 1.5;
const TURN: f64 = 1.0;
const THINK_HOLD: f64 = 3.0;

// ---------------------------------------------------------------- numbers

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    /// The player's dollars as 06 left them.
    here: i64,
    published_badge: String,
    round_badge: String,
    /// "6 → remainder 0", "600 → remainder 0".
    examples: [String; 2],
    k03: String,
    /// Game A's share at each corner, "1/3".
    a_share: String,
    /// Under A, then under B: "the player's 100,000 rounds", "33.5 % at
    /// corner 0".
    readouts: [[String; 2]; 2],
    /// The coins' chances, published then round: "49.5 %" → "50 %".
    legend: [[String; 3]; 2],
    plus: String,
    /// Game A's and game B's arrows, published then round.
    published: [Chances; 2],
    round: [Chances; 2],
    sum_tex: String,
}

/// The facts' remainder of `count`, checked against the model's.
fn remainder_fact(examples: &str, count: i64) -> Result<i64, String> {
    let r = printed_remainder(examples, count)?;
    if r != count.rem_euclid(MODULUS) {
        return Err(format!("the remainder fact for {count} is not its remainder"));
    }
    Ok(r)
}

/// "49.5 %", "9.5 %", "74.5 %": three chances as the legend writes them.
fn pct_labels(c: [Ratio; 3]) -> Result<[String; 3], String> {
    let [x, y, z] = c.map(|r| r.exact_pct().map(|p| format!("{p} %")));
    Ok([x?, y?, z?])
}

/// The run of `key` ("a", "b") as its readout's two lines, and its share at
/// corner 0 (from the facts' count of rounds begun there).
fn readout(f: &Facts, key: &str) -> Result<([String; 2], Ratio), String> {
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let rounds: i128 = get(&format!("run.{key}.at0.rounds"))?
        .parse()
        .map_err(|e| format!("run.{key}.at0.rounds: {e}"))?;
    if get("run.rounds")? != RUN_ROUNDS.to_string() {
        return Err("the readouts say 100,000 rounds, but the runs are longer or shorter".into());
    }
    let share = Ratio::new(rounds, RUN_ROUNDS as i128);
    let pct = get(&format!("run.{key}.at0.pct"))?;
    if share.pct(1) != pct {
        return Err(format!("run {key}'s share at corner 0 disagrees with its count"));
    }
    Ok((
        [
            format!("{READOUT_HEAD} {} rounds", thousands(RUN_ROUNDS as u64)),
            format!("{pct} % {READOUT_TAIL}"),
        ],
        share,
    ))
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let here = walk()[3];
    let examples = get("remainder.examples")?;
    // 07_1: six dollars and six hundred both stand on corner 0
    let six = [6, 600];
    if [remainder_fact(&examples, six[0])?, remainder_fact(&examples, six[1])?] != [0, 0] {
        return Err("07_1 says six and six hundred both stand at corner 0".into());
    }
    // K03: a count below zero still has its corner
    let (minus_one, minus_three) = (remainder_fact(&examples, -1)?, remainder_fact(&examples, -3)?);
    let k03 = format!(
        "the capital is any whole number, negatives included, and play never stops; the remainder is 0, 1 or 2, \
         so {} leaves {minus_one} and {} leaves {minus_three}",
        screen_int(-1),
        screen_int(-3),
    );
    // 07_3: game A's corners share the rounds equally, whatever its coin
    let third = Ratio::new(1, 3);
    for coins in [&PUBLISHED, &ROUND] {
        if coins.game_a().shares() != [third; 3] {
            return Err("07_3 says game A's corners share the rounds equally".into());
        }
    }
    if get("thirds.pct")? != third.pct(1) {
        return Err("a third disagrees with the facts".into());
    }
    let (a_lines, a_share) = readout(&f, "a")?;
    let (b_lines, b_share) = readout(&f, "b")?;
    // the voice: A's run thirty-three and a half percent, B's more than
    // thirty-eight
    if a_share.pct(1) != "33.5" || b_share <= Ratio::new(38, 100) || b_share >= Ratio::new(39, 100) {
        return Err("07_3 and 07_4 say 33.5 % and more than 38 % at corner 0".into());
    }
    // 07_5: each round coin is its published coin with half a point back
    let half_point = Ratio::new(HANDICAP.into(), DEN.into());
    if get("rules.handicap")? != half_point.to_string() || half_point.pct(1) != "0.5" {
        return Err("07_5 says half a point, but the handicap differs".into());
    }
    let (from, to) = (facts::coins(&PUBLISHED), facts::coins(&ROUND));
    for (&p, &r) in from.iter().zip(&to) {
        if r - p != half_point || !(r * Ratio::int(100)).is_integer() {
            return Err("07_5's round coins are not the published ones plus half a point".into());
        }
    }
    let legend = [pct_labels(from)?, pct_labels(to)?];
    let pcts = |c: &[Ratio; 3]| -> Result<String, String> {
        Ok(c.iter()
            .map(|x| x.exact_pct())
            .collect::<Result<Vec<_>, _>>()?
            .join(","))
    };
    if get("coins.round.heads.pct")? != pcts(&to)? || get("coins.published.heads.pct")? != pcts(&from)? {
        return Err("the legend's chances disagree with the facts".into());
    }
    // 07_6: game A becomes a fair coin, which neither wins nor loses on
    // average; B gets ten percent at corner 0 and seventy-five elsewhere
    let (round_a, round_b) = (ROUND.game_a(), ROUND.game_b());
    if round_a.chance(0) != Ratio::new(1, 2) || round_a.mean() != Ratio::ZERO {
        return Err("07_6 says game A becomes a fair coin".into());
    }
    let round = [chances(&round_a)?, chances(&round_b)?];
    fn words(c: &[String; 3]) -> [&str; 3] {
        c.each_ref().map(String::as_str)
    }
    if words(&round[1].heads) != ["10", "75", "75"] || words(&round[0].heads) != ["50"; 3] {
        return Err("07_6 says a ten percent coin at corner 0 and seventy-five elsewhere".into());
    }
    // 07_7: with equal thirds, B would win 53.3 % of its rounds
    let thirds = facts::thirds_win(&ROUND);
    if thirds.pct(1) != get("round.b.thirds.win.pct")? || thirds.pct(1) != "53.3" || thirds <= Ratio::new(1, 2) {
        return Err("07_7 says B would win fifty-three point three percent".into());
    }
    let equals = if thirds.exact_pct().is_ok() { "=" } else { r"\approx" };
    let sum_tex = format!(
        r"\tfrac{{1}}{{3}} \cdot {}\% + \tfrac{{2}}{{3}} \cdot {}\% {equals} {}\%",
        to[1].exact_pct()?,
        to[2].exact_pct()?,
        thirds.pct(1)
    );
    let round_pcts: Vec<String> = to.iter().map(|x| x.exact_pct()).collect::<Result<_, _>>()?;
    Ok(Numbers {
        here,
        published_badge: badge_text(&f)?,
        round_badge: format!(
            "{ROUND_BADGE} \u{b7} {} \u{b7} {} \u{b7} {} % \u{b7} {NO_HANDICAP}",
            round_pcts[0], round_pcts[1], round_pcts[2]
        ),
        examples: six.map(remainder_text),
        k03,
        a_share: third.to_string(),
        readouts: [a_lines, b_lines],
        legend,
        plus: format!("+{}", half_point.pct(1)),
        published: [chances(&PUBLISHED.game_a())?, chances(&PUBLISHED.game_b())?],
        round,
        sum_tex,
    })
}

// --------------------------------------------------------------- pictures

/// The round-number coins' badge, outlined, its right edge kept inside the
/// frame as [`full_badge`] keeps the published one's. Detached.
fn round_badge(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<[Mob; 2]> {
    let tag = badge(kit, stage, text, false)?;
    let right = FRAME_X_RADIUS - ROOM;
    if stage.get_bounding_box_point(tag[0], RIGHT)[0] > right {
        stage.move_to(tag[0], v(right, BADGE_AT[1]), RIGHT);
        stage.move_to(tag[1], v(right - 0.15, BADGE_AT[1]), RIGHT);
    }
    keep_between(stage, tag[0], MID_RIGHT - 1.0, right, "the round-number badge")?;
    Ok(tag)
}

/// A hero outline around `mobs` as typeset (one or several): pointing at
/// text without touching its glyphs. Detached.
fn outline(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<Mob> {
    let (mut l, mut b, mut r, mut t) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for &m in mobs {
        l = l.min(stage.get_bounding_box_point(m, LEFT)[0]);
        b = b.min(stage.get_bounding_box_point(m, DOWN)[1]);
        r = r.max(stage.get_bounding_box_point(m, RIGHT)[0]);
        t = t.max(stage.get_bounding_box_point(m, UP)[1]);
    }
    let frame = rect(
        v(l - PAD, b - PAD),
        r - l + 2.0 * PAD,
        t - b + 2.0 * PAD,
        line_style(Palette::hero(), 2.0),
    )?;
    Ok(stage.arena_mut().add(frame))
}

/// Keep the fraction's dark background while pointing at its corner.
/// The native12 frame at 35.5 s showed white thirds losing contrast on
/// indicate's bright yellow fill. Only separate hollow rings change here.
fn point_at_corners(stage: &mut Stage<'_>, discs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut rings = Vec::with_capacity(discs.len());
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(discs.len());
    for &disc in discs {
        let centre = stage.get_center(disc);
        let ring = Circle::new()
            .radius(CORNER_R + 0.05)
            .arc_center(centre)
            .style(
                Style::default()
                    .fill(Palette::hero(), 0.0)
                    .stroke(Palette::hero(), 3.0, 0.0),
            )
            .build();
        let ring = stage.arena_mut().add(ring);
        stage.add_to_scene(ring)?;
        let pulse = ring
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                rate_func: Some(fmn::core::rate::there_and_back),
                ..AnimateArgs::default()
            })?
            .set_stroke(None, None, Some(1.0))?;
        anims.push(stage.prepare(pulse)?);
        rings.push(ring);
    }
    stage.play_prepared(anims)?;
    stage.scene_mut().remove(&rings);
    Ok(())
}

/// One run's readout under its triangle. Detached.
fn readout_lines(kit: &Kit, stage: &mut Stage<'_>, which: Which, lines: &[String; 2]) -> fmn::Result<[Mob; 2]> {
    let at = which.at();
    let color = match which {
        Which::A => palette::game_a(),
        Which::B => palette::game_b(),
    };
    let head = kit.text(stage, &lines[0], 18.0, Palette::dim(), v(at[0], at[1] - READOUT_DY[0]))?;
    let share = kit.text(stage, &lines[1], 22.0, color, v(at[0], at[1] - READOUT_DY[1]))?;
    for m in [head, share] {
        keep_between(
            stage,
            m,
            -FRAME_X_RADIUS + EDGE,
            FRAME_X_RADIUS - EDGE,
            "a run's readout",
        )?;
    }
    Ok([head, share])
}

impl SceneConstruct for WhereYouStand {
    fn name(&self) -> &str {
        "07_where_you_stand"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;

        // 07_1: 06's picture; six and six hundred both stand on corner 0,
        // so what decides B is how often the player stands at each corner
        let line = say(&kit, stage, "07_1")?;
        let a = triangle(&kit, stage, Which::A)?;
        let b = triangle(&kit, stage, Which::B)?;
        let mut a_labels = chance_labels(&kit, stage, Which::A, &n.published[0])?;
        let mut b_labels = chance_labels(&kit, stage, Which::B, &n.published[1])?;
        let hud = counter(&kit, stage, n.here)?;
        let rem = remainder_line(&kit, stage, n.here)?;
        let player = token(stage, corner(B_AT, corner_of(n.here)));
        let mut tag = full_badge(&kit, stage, &n.published_badge, MID_RIGHT - 1.0)?;
        let mut texts = vec![hud[0], hud[1], rem, tag[1], a.title, b.title];
        texts.extend(a.names);
        texts.extend(b.names);
        texts.extend(a_labels);
        texts.extend(b_labels);
        keep_apart(stage, &texts, 0.08, "06's picture")?;
        let mut arriving = a.parts();
        arriving.extend(b.parts());
        arriving.extend(a_labels);
        arriving.extend(b_labels);
        arriving.extend([hud[0], hud[1], rem, player]);
        arriving.extend(tag);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, AT_SIX.1)?;
        let six = kit.text(stage, &n.examples[0], 22.0, palette::bad(), v(MID_X, SIX_Y))?;
        keep_between(stage, six, MID_LEFT, MID_RIGHT, "six dollars")?;
        fade_in_all(stage, &[six], 0.4)?;
        line.wait_for(stage, AT_SIX_HUNDRED.1)?;
        let six_hundred = kit.text(stage, &n.examples[1], 22.0, palette::bad(), v(MID_X, SIX_HUNDRED_Y))?;
        keep_between(stage, six_hundred, MID_LEFT, MID_RIGHT, "six hundred dollars")?;
        fade_in_all(stage, &[six_hundred], 0.4)?;
        line.wait_for(stage, AT_SAME.1)?;
        // both slide into B's corner 0 and are gone; the card says what the
        // remainder of any capital is
        let zero = corner(B_AT, 0);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in [six, six_hundred] {
            let c = stage.get_center(m);
            let into = fade_out(stage.arena_mut(), m, v(zero[0] - c[0], zero[1] - c[1]), INTO_SCALE)?;
            anims.push(stage.prepare(into.rt(0.8))?);
        }
        stage.play_prepared(anims)?;
        flash_all(stage, &[b.discs[0]], 0.5)?;
        let k03 = precisely(&kit, stage, &n.k03)?;
        if stage.get_bounding_box_point(k03, UP)[1] > stage.get_bounding_box_point(a.title, DOWN)[1] - 0.1 {
            return Err(fail("K03 runs into the triangles' titles".into()));
        }
        fade_in_all(stage, &[k03], 0.5)?;
        let card_up = now(stage);
        line.wait_for(stage, AT_STAND.1)?;
        for &d in &b.discs {
            flash_all(stage, &[d], 0.45)?;
        }
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&n.k03))?;
        fade_out_all(stage, &[k03], 0.4)?;

        // 07_2: playing only game A, what share of the rounds at corner 0?
        // (a prediction; no share is shown)
        let line = say(&kit, stage, "07_2")?;
        line.wait_for(stage, AT_ONLY_A.1)?;
        flash_all(stage, &a.discs, 0.6)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 07_3: a third at each of A's corners; the player's game A run
        let line = say(&kit, stage, "07_3")?;
        line.wait_for(stage, AT_THIRD.1)?;
        let mut thirds = Vec::with_capacity(3);
        for i in 0..3 {
            let m = kit.text(stage, &n.a_share, 22.0, Palette::ink(), corner(A_AT, i))?;
            if stage.get_width(m) > 2.0 * CORNER_R - 0.12 {
                return Err(fail("a third does not fit inside its corner".into()));
            }
            thirds.push(m);
        }
        fade_out_all(stage, &[q], 0.2)?;
        fade_in_all(stage, &thirds, 0.5)?;
        line.wait_for(stage, AT_EQUALLY.1)?;
        point_at_corners(stage, &a.discs, 0.6)?;
        line.wait_for(stage, AT_A_RUN.1)?;
        let a_run = readout_lines(&kit, stage, Which::A, &n.readouts[0])?;
        fade_in_all(stage, &a_run, 0.5)?;
        point_at_corners(stage, &[a.discs[0]], 0.5)?;

        // 07_4: game B's corner 0 is not like the others; the player's game
        // B run spent more than 38 % there; both runs side by side
        let line = say(&kit, stage, "07_4")?;
        line.wait_for(stage, AT_NOT_ALIKE.1)?;
        // corner 0's two ways: forward along edge 0, back along edge 2
        let marks = [outline(stage, &[b_labels[0]])?, outline(stage, &[b_labels[5]])?];
        fade_in_all(stage, &marks, 0.4)?;
        line.wait_for(stage, AT_B_RUN.1)?;
        let b_run = readout_lines(&kit, stage, Which::B, &n.readouts[1])?;
        let mut all_texts = texts.clone();
        all_texts.extend(a_run);
        all_texts.extend(b_run);
        all_texts.extend(&thirds);
        keep_apart(stage, &all_texts, 0.08, "the runs' readouts")?;
        fade_in_all(stage, &b_run, 0.5)?;
        flash_all(stage, &[b.discs[0]], 0.6)?;
        kit.hold(stage)?;
        pause(stage, COMPARE_S)?;

        // 07_5: rounder numbers: each coin's half point goes back, and the
        // badge names the round-number coins
        let line = say(&kit, stage, "07_5")?;
        let mut leaving = a_run.to_vec();
        leaving.extend(b_run);
        leaving.extend(marks);
        fade_out_all(stage, &leaving, 0.4)?;
        line.wait_for(stage, AT_ROUNDER.1)?;
        let faces = [
            (palette::game_a(), A_FACE),
            (palette::bad(), BAD_FACE),
            (palette::good(), GOOD_FACE),
        ];
        let mut coins = Vec::with_capacity(3);
        let mut chances_from = Vec::with_capacity(3);
        for (i, (color, face)) in faces.into_iter().enumerate() {
            let y = LEGEND_YS[i];
            coins.push(coin(&kit, stage, v(LEGEND_X, y), LEGEND_R, color, face)?);
            let m = kit.text_left(stage, &n.legend[0][i], 20.0, Palette::ink(), v(LEGEND_NUM_X, y))?;
            keep_between(stage, m, LEGEND_NUM_X - 0.01, FRAME_X_RADIUS - EDGE, "a coin's chance")?;
            chances_from.push(m);
        }
        let mut legend: Vec<Mob> = coins.iter().flatten().copied().collect();
        legend.extend(&chances_from);
        fade_in_all(stage, &legend, 0.5)?;
        line.wait_for(stage, AT_SHAVED.1)?;
        let shaved = outline(stage, &chances_from)?;
        fade_in_all(stage, &[shaved], 0.4)?;
        line.wait_for(stage, AT_BACK.1)?;
        fade_out_all(stage, &[shaved], 0.3)?;
        // each chance gains its half point: "+0.5" rises from it and fades
        // as the round chance replaces it; the badge turns with them
        let mut pluses = Vec::with_capacity(3);
        let mut chances_to = Vec::with_capacity(3);
        for (i, &from) in chances_from.iter().enumerate() {
            let c = stage.get_center(from);
            pluses.push(kit.text(stage, &n.plus, 18.0, Palette::hero(), v(c[0], LEGEND_YS[i] + PLUS_DY))?);
            let m = kit.text_left(
                stage,
                &n.legend[1][i],
                20.0,
                Palette::ink(),
                v(LEGEND_NUM_X, LEGEND_YS[i]),
            )?;
            keep_between(
                stage,
                m,
                LEGEND_NUM_X - 0.01,
                FRAME_X_RADIUS - EDGE,
                "a coin's round chance",
            )?;
            chances_to.push(m);
        }
        fade_in_all(stage, &pluses, 0.3)?;
        let round_tag = round_badge(&kit, stage, &n.round_badge)?;
        let mut anims = swap(stage, &chances_from, &chances_to, 0.7)?;
        anims.extend(swap(stage, &tag, &round_tag, 0.7)?);
        for &p in &pluses {
            let rise = fade_out(stage.arena_mut(), p, v(0.0, PLUS_RISE), 1.0)?;
            anims.push(stage.prepare(rise.rt(0.9))?);
        }
        stage.play_prepared(anims)?;
        tag = round_tag;
        kit.hold(stage)?;
        pause(stage, TURN)?;

        // 07_6: the arrows relabel with the round coins: A fair, B ten
        // percent at corner 0 and seventy-five elsewhere
        let line = say(&kit, stage, "07_6")?;
        let a_round = chance_labels(&kit, stage, Which::A, &n.round[0])?;
        let b_round = chance_labels(&kit, stage, Which::B, &n.round[1])?;
        line.wait_for(stage, AT_FAIR.1)?;
        let anims = swap(stage, &a_labels, &a_round, 0.6)?;
        stage.play_prepared(anims)?;
        a_labels = a_round;
        flash_all(stage, &[coins[0][0]], 0.5)?;
        line.wait_for(stage, AT_TEN.1)?;
        // corner 0's two ways: forward along edge 0, back along edge 2
        let anims = swap(stage, &[b_labels[0], b_labels[5]], &[b_round[0], b_round[5]], 0.6)?;
        stage.play_prepared(anims)?;
        flash_all(stage, &[coins[1][0]], 0.5)?;
        line.wait_for(stage, AT_SEVENTY_FIVE.1)?;
        let anims = swap(stage, &b_labels[1..5], &b_round[1..5], 0.6)?;
        stage.play_prepared(anims)?;
        b_labels = b_round;
        flash_all(stage, &[coins[2][0]], 0.5)?;

        // 07_7: the worked example; B's win if its corners shared equally
        let line = say(&kit, stage, "07_7")?;
        line.wait_for(stage, AT_EXAMPLE.1)?;
        flash_all(stage, &[tag[0]], 0.6)?;
        line.wait_for(stage, AT_THIRDS.1)?;
        let if_line = kit.text(stage, IF_LINE, 20.0, Palette::dim(), v(MID_X, IF_Y))?;
        let sum = kit.tex(stage, &n.sum_tex, 30.0, Palette::ink(), &[], v(MID_X, SUM_Y))?;
        keep_between(stage, if_line, MID_LEFT, MID_RIGHT, "the hypothetical's label")?;
        keep_between(stage, sum, MID_LEFT, MID_RIGHT, "the hypothetical")?;
        fade_in_all(stage, &[if_line, sum], 0.5)?;
        line.wait_for(stage, AT_WOULD.1)?;
        let would = kit.text(stage, WOULD_WIN, 26.0, palette::good(), v(MID_X, WOULD_Y))?;
        keep_between(stage, would, MID_LEFT, MID_RIGHT, "the verdict")?;
        let mut last = vec![hud[0], hud[1], rem, tag[1], if_line, sum, would];
        last.extend(a_labels);
        last.extend(b_labels);
        last.extend(a.names);
        last.extend(b.names);
        last.extend(chances_to);
        keep_apart(stage, &last, 0.08, "the round coins' picture")?;
        fade_in_all(stage, &[would], 0.4)?;
        kit.hold(stage)?;
        pause(stage, THINK_HOLD)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};
    use fmn::core::constants::FRAME_Y_RADIUS;
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::THINK_S;
    use super::super::three_corners::{chance_at, name_at};
    use super::*;
    use crate::model::{Coins, Game};

    /// The chapter's words are true of its numbers: six and six hundred
    /// leave nothing; K03 puts −1 on corner 2 and −3 on corner 0; game A's
    /// corners each hold a third; its run spent 33.5 % at corner 0 and B's
    /// more than 38 %; each coin gains half a point (49.5 → 50, 9.5 → 10,
    /// 74.5 → 75); with equal thirds B would win 53.3 %.
    #[test]
    fn where_you_stand_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.here, 8);
        assert_eq!(n.examples, ["6 \u{2192} remainder 0", "600 \u{2192} remainder 0"]);
        assert!(n.k03.contains("negatives included") && n.k03.contains("play never stops"));
        assert!(
            n.k03.ends_with("so \u{2212}1 leaves 2 and \u{2212}3 leaves 0"),
            "{}",
            n.k03
        );
        assert_eq!(n.a_share, "1/3");
        assert_eq!(n.readouts[0], ["the player's 100,000 rounds", "33.5 % at corner 0"]);
        assert_eq!(n.readouts[1], ["the player's 100,000 rounds", "38.4 % at corner 0"]);
        assert_eq!(n.legend, [["49.5 %", "9.5 %", "74.5 %"], ["50 %", "10 %", "75 %"]]);
        assert_eq!(n.plus, "+0.5");
        assert_eq!(
            n.published_badge,
            "published coins \u{b7} 49.5 \u{b7} 9.5 \u{b7} 74.5 %"
        );
        assert_eq!(
            n.round_badge,
            "round-number coins \u{b7} 50 \u{b7} 10 \u{b7} 75 % \u{b7} no handicap"
        );
        fn words(c: &[String; 3]) -> [&str; 3] {
            c.each_ref().map(String::as_str)
        }
        assert_eq!(words(&n.round[0].heads), ["50"; 3]);
        assert_eq!(words(&n.round[0].tails), ["50"; 3]);
        assert_eq!(words(&n.round[1].heads), ["10", "75", "75"]);
        assert_eq!(words(&n.round[1].tails), ["90", "25", "25"]);
        assert_eq!(
            n.sum_tex,
            r"\tfrac{1}{3} \cdot 10\% + \tfrac{2}{3} \cdot 75\% \approx 53.3\%"
        );
    }

    /// The controlled change is the half point and nothing else: each round
    /// coin is its published coin plus 1/200, the same at every corner of
    /// both games, so the arrows' chances move by half a point each way.
    #[test]
    fn the_half_point_is_the_only_change() {
        let step = Ratio::new(HANDICAP.into(), DEN.into());
        for (p, r) in facts::coins(&PUBLISHED).into_iter().zip(facts::coins(&ROUND)) {
            assert_eq!(r - p, step);
        }
        for (p, r) in [
            (PUBLISHED.game_a(), ROUND.game_a()),
            (PUBLISHED.game_b(), ROUND.game_b()),
        ] {
            for i in 0..3 {
                assert_eq!(r.chance(i) - p.chance(i), step, "corner {i}");
            }
        }
        // game A becomes fair: a win half the time, nothing on average
        assert_eq!(ROUND.game_a().chance(0), Ratio::new(1, 2));
        assert_eq!(ROUND.game_a().mean(), Ratio::ZERO);
    }

    /// Negative control: equal thirds is not where game B stands. With the
    /// published coins, thirds would have B win 52.8 % of its rounds, yet B
    /// loses on average and its run spent 38.4 % at corner 0, not a third.
    /// With the round coins, thirds would give 8/15, yet B's long-run share
    /// of wins is exactly a half and its average exactly zero. (The share B
    /// really keeps at each corner is 08's subject; the scene shows none.)
    #[test]
    fn equal_thirds_is_not_where_b_stands() {
        let f = facts::facts().unwrap();
        let thirds_pub = facts::thirds_win(&PUBLISHED);
        assert_eq!(thirds_pub.pct(1), "52.8");
        assert!(thirds_pub > Ratio::new(1, 2) && PUBLISHED.game_b().mean() < Ratio::ZERO);
        let (_, b_share) = readout(&f, "b").unwrap();
        assert!(b_share > Ratio::new(1, 3) + Ratio::new(4, 100), "{b_share}");
        assert_eq!(facts::thirds_win(&ROUND), Ratio::new(8, 15));
        assert_eq!(ROUND.game_b().win(), Ratio::new(1, 2));
        assert_eq!(ROUND.game_b().mean(), Ratio::ZERO);
        assert!(ROUND.game_b().shares()[0] > Ratio::new(1, 3));
    }

    /// Game A's corners share the rounds equally whatever its one coin: the
    /// "1/3" in its corners is not a property of 49.5 % or of 50 %.
    #[test]
    fn game_a_shares_its_corners_equally_for_any_coin() {
        for a in 1..200 {
            let g: Game = Coins { a, bad: 20, good: 150 }.game_a();
            assert_eq!(g.shares(), [Ratio::new(1, 3); 3], "a = {a}/200");
        }
    }

    /// K03 fits two lines of the lower third (about 72 characters a line
    /// beside "precisely:" at 26 px), so its top stays below the titles.
    #[test]
    fn the_card_fits_two_lines() {
        let n = numbers().unwrap();
        let lines = explainer_kit::animatic::wrap(&n.k03, 72, usize::MAX);
        assert!(lines.len() <= 2, "{lines:?}");
        assert_eq!(lines.join(" "), n.k03);
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the middle column clears both corner-0
    /// discs and the badge; the coins at the right clear game B's triangle
    /// and the frame, and their "+0.5" stays under the coin above; the
    /// runs' readouts sit under the titles, inside the frame.
    #[test]
    fn the_layout_keeps_its_rooms() {
        const {
            assert!(MID_LEFT < MID_X - 2.0 && MID_RIGHT > MID_X + 2.0);
            assert!(IF_Y + 0.15 < BADGE_AT[1] - 0.19 && WOULD_Y > 1.2);
            assert!(THINK_AT[1] + 0.62 + 0.2 < BADGE_AT[1] - 0.19);
        }
        // the coins: right of B's widest label on its right-hand edge, above
        // its corner 1 and that corner's name, inside the frame, below the
        // badge
        let label = chance_at(B_AT, 0, true, 0.85, 0.22);
        assert!(label[0] + 0.425 + ROOM < LEGEND_X - LEGEND_R, "{label:?}");
        let name = name_at(B_AT, 1, 0.2, 0.25);
        assert!(LEGEND_YS[2] - LEGEND_R > name[1] + 0.125 + 1.0);
        const {
            assert!(LEGEND_NUM_X + 0.8 < FRAME_X_RADIUS - EDGE);
            assert!(LEGEND_YS[0] + LEGEND_R < BADGE_AT[1] - 0.19 - 0.2);
        }
        for pair in LEGEND_YS.windows(2) {
            let rise_top = pair[1] + PLUS_DY + PLUS_RISE + 0.1;
            assert!(rise_top < pair[0] - 0.11, "a +0.5 at {} reaches the row above", pair[1]);
        }
        // the readouts: under the titles, above the frame's foot
        for at in [A_AT, B_AT] {
            assert!(at[1] - READOUT_DY[0] + 0.1 < at[1] - TITLE_DY - 0.13);
            assert!(at[1] - READOUT_DY[1] - 0.15 > -FRAME_Y_RADIUS + 0.25);
        }
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` and `keep_apart` guard meets the
    /// true text widths): every line of chapter 07 is said once, in order;
    /// the prediction (07_2) gets its silent think time; the runs stand side
    /// by side before 07_5, 07_5 gets its turn and 07_7 its thinking hold.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = WhereYouStand { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("07_where_you_stand: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=7).map(|k| format!("07_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let end_of = |id: &str| at(id) + narrator.length(id).unwrap();
        assert!(
            at("07_3") - end_of("07_2") >= THINK_S,
            "{} s to think",
            at("07_3") - end_of("07_2")
        );
        assert!(at("07_5") - end_of("07_4") >= COMPARE_S, "the runs are not compared");
        assert!(at("07_6") - end_of("07_5") >= TURN, "no turn after the half point");
        let n = numbers().unwrap();
        assert!(at("07_2") - at("07_1") >= read_s(&n.k03), "K03 cannot be read");
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end - end_of("07_7") >= THINK_HOLD, "07_7 has no thinking hold: {end}");
    }
}
