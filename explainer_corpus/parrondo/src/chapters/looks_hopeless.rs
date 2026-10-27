//! Chapter 04, why switching looks hopeless. The player, as 01 and 02 left
//! them at 7 dollars, meets the mix: a fair gold coin flipped before every
//! round, heads for game A, tails for game B (04_1). Two ordinary coins
//! that both lose, 49 % and 48 %, chosen between by a fair coin, are
//! exactly one coin at 48.5 %, which loses too (04_2, 04_3). The games'
//! own averages seem to agree: bars to scale, A −1.00 and B −0.87 cents a
//! round, and a dashed "halfway: −0.93 ?" (04_4). Win or lose? (04_5, a
//! prediction) They win: the same player, 100,000 rounds of the mix,
//! climbing to 1,748 dollars up over A's and B's faint runs (04_6).
//!
//! Both attractive wrong answers are on screen before the reveal, and both
//! are computed: the ordinary coins' mix (which really does lose), and the
//! average of the two averages (which is not the mix's, as 05 shows). The
//! bars are exact long-run averages; the run is one seeded player. Every
//! number comes from `crate::model` and `crate::facts`; the scene refuses to
//! draw if the words would stop being true of them (the ordinary coins and
//! their mix lose, both games lose and A more, halfway is a loss, the mix's
//! run ends well over a thousand dollars up).
//!
//! The window, the faint runs, the averages' bars, the badge and the
//! player's place are shared with chapter 05 (`the_clue.rs`), which returns
//! to them.

use explainer_kit::{Facts, Frame2, Kit, Palette, Timed, fade_all, rect, signed_thousands, thousands, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    BADGE_AT, TOKEN_R, arrow, badge, coin, dashed, draw_run, fade_in_all, fade_out_all, fade_to, fail, flash_all, flat,
    fly, growing_run, keep_between, line_style, now, say, screen, stand, think, thinned, together, token,
};
use crate::facts::{self, ORDINARY};
use crate::model::{self, Game, PUBLISHED, Play, RUN_ROUNDS, Ratio, START};
use crate::palette;

pub struct LooksHopeless {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const PLAYER: &str = "player";
const DOLLARS: &str = "dollars";
const HEADS_FACE: &str = "A";
const TAILS_FACE: &str = "B";
const HEADS_RULE: &str = "heads \u{2192} game A";
const TAILS_RULE: &str = "tails \u{2192} game B";
const LOSES: &str = "loses";
const HALF: &str = "1/2";
const ROUNDS: &str = "rounds";
const A_END: &str = "A";
const B_END: &str = "B";
const MIX_END: &str = "mix";
/// 02_5's badge words, shown with the chances from then on.
const BADGE: &str = "published coins";
const MIX_NAME: &str = "the mix";
/// The bars are the games' exact long-run averages, not a run's outcome.
const BARS_CAPTION: &str = "long-run average per round, cents";
const ZERO: &str = "0";
const A_ROW: &str = "game A";
const B_ROW: &str = "game B";
const HALFWAY_ROW: &str = "halfway:";
const ASK: &str = "?";

/// Where each picture lands: (line, the phrase it waits for).
const AT_FLIPS: (&str, &str) = ("04_1", "flips a fair coin");
const AT_HEADS: (&str, &str) = ("04_1", "Heads means play game A");
const AT_TAILS: (&str, &str) = ("04_1", "tails means play game B");
const AT_CALL: (&str, &str) = ("04_1", "Call this the mix");
const AT_ORDINARY: (&str, &str) = ("04_2", "two ordinary coins");
const AT_49: (&str, &str) = ("04_2", "forty-nine");
const AT_48: (&str, &str) = ("04_2", "one forty-eight");
const AT_CHOOSE: (&str, &str) = ("04_3", "Choosing between them");
const AT_ONE_COIN: (&str, &str) = ("04_3", "one coin that lands heads halfway between");
const AT_LOSER: (&str, &str) = ("04_3", "Two losers mix into a loser");
const AT_A_CENT: (&str, &str) = ("04_4", "Game A loses a cent a round");
const AT_B_CENT: (&str, &str) = ("04_4", "game B a little under a cent");
const AT_HALFWAY: (&str, &str) = ("04_4", "so halfway between");
const AT_PLAYER: (&str, &str) = ("04_6", "Here's the same player");
const AT_RUN: (&str, &str) = ("04_6", "a hundred thousand rounds of the mix");
const AT_WELL_OVER: (&str, &str) = ("04_6", "well over a thousand dollars");
#[cfg(test)]
const CUES: [(&str, &str); 16] = [
    AT_FLIPS,
    AT_HEADS,
    AT_TAILS,
    AT_CALL,
    AT_ORDINARY,
    AT_49,
    AT_48,
    AT_CHOOSE,
    AT_ONE_COIN,
    AT_LOSER,
    AT_A_CENT,
    AT_B_CENT,
    AT_HALFWAY,
    AT_PLAYER,
    AT_RUN,
    AT_WELL_OVER,
];

/// "Well over a thousand dollars" (04_6): at least half as much again.
const WELL_OVER: i64 = 1_500;

// ------------------------------------------------------------------ layout

/// The player where 01 and 02 kept them: token, label, count.
pub(super) const TOKEN_AT: Vec3 = [-5.6, 0.9, 0.0];
pub(super) const PLAYER_LABEL_AT: Vec3 = [-5.6, 0.35, 0.0];
const COUNT_AT: Vec3 = [-4.9, 1.5, 0.0];
const DOLLARS_X: f64 = -4.1;
/// The player's count ("7 dollars") ends left of this; what stands beside
/// it keeps to the right.
const COUNT_RIGHT: f64 = -2.8;
/// 04_1: the mix's gold coin above the token, its two rules beside it.
const MIX_COIN_AT: Vec3 = [-5.6, 2.55, 0.0];
const MIX_COIN_R: f64 = 0.4;
const MIX_NAME_AT: Vec3 = [-5.6, 3.35, 0.0];
const RULE_X: f64 = -4.95;
const HEADS_Y: f64 = 2.78;
const TAILS_Y: f64 = 2.32;
/// 04_2 and 04_3: the ordinary coins, their chooser, the coin they make.
const ORD_AT: [Vec3; 2] = [[0.4, 1.0, 0.0], [3.8, 1.0, 0.0]];
const ORD_R: f64 = 0.55;
const LOSES_GAP: f64 = 0.3;
const CHOOSER_AT: Vec3 = [2.1, 2.65, 0.0];
const CHOOSER_R: f64 = 0.22;
/// The chooser's arrows stop this far short of a coin's centre line and
/// above its top; each "1/2" stands this far off its arrow, outward.
const ARROW_INSET: f64 = 0.3;
const ARROW_CLEAR: f64 = 0.08;
const HALF_OFF: f64 = 0.32;
const MERGED_AT: Vec3 = [2.1, 1.0, 0.0];
const TEX_AT: Vec3 = [2.1, -0.55, 0.0];
/// 04_4 (and 05_3): bars to scale about a zero line, one row each for A,
/// B, halfway and the mix, labels right-aligned in one column.
pub(super) const ZERO_X: f64 = 2.4;
/// Scene units per cent a round.
const CENT_U: f64 = 1.9;
pub(super) const BAR_H: f64 = 0.4;
pub(super) const ROW_YS: [f64; 4] = [1.3, 0.55, -0.2, -0.95];
const LABEL_GAP: f64 = 0.2;
const AXIS_OVER: f64 = 0.2;
const CAPTION_DY: f64 = 0.25;
const DASH: f64 = 0.12;
/// 04_5: the prediction, in the empty lower left.
const THINK_AT: Vec3 = [-3.8, -1.6, 0.0];
/// The think ring's radius and its note's drop (`draw::think`).
#[cfg(test)]
const THINK_R: f64 = 0.62;
#[cfg(test)]
const NOTE_DROP: f64 = 0.92;
/// 04_6: the long window, tall enough for the mix's climb (the runs reach
/// −1,138 and +1,794 dollars).
const WIDE: Frame2 = Frame2 {
    center: [0.8, -0.1, 0.0],
    width: 10.0,
    height: 5.0,
    x: (0.0, 100_000.0),
    y: (-1200.0, 1900.0),
};
pub(super) const TICK_DY: f64 = 0.22;
/// Every 250th round of the runs is drawn (401 points), as in 01 and 03.
pub(super) const THIN: usize = 250;
const RUN_S: f64 = 6.5;
pub(super) const FAINT: f64 = 0.35;
/// A run's end label and the dollars ticks keep clear of the token, which
/// waits at a run's start or end.
pub(super) const END_LABEL_DX: f64 = TOKEN_R + 0.1;
const TICK_DX: f64 = TOKEN_R + 0.1;
/// 04_6's readout, above the window and below the badge (the drawn run and
/// the token riding it stay below it; the layout test checks every point).
const END_AT: Vec3 = [2.4, 2.95, 0.0];
/// Labels keep this far from their neighbours.
pub(super) const ROOM: f64 = 0.2;
/// Texts keep this far from the frame's left, right and top edges (the
/// layout lint's title-safe margin is 0.25).
pub(super) const EDGE: f64 = 0.3;
/// A pointing frame keeps this far around the words it points at, in a
/// stroke this wide.
pub(super) const POINT_PAD: f64 = 0.07;
const POINT_W: f64 = 2.0;

/// The games' long-run averages per round, exactly, and the bars' labels
/// (04_4 and 05_3): A, B, halfway, the mix.
pub(super) struct Averages {
    pub a: Ratio,
    pub b: Ratio,
    pub halfway: Ratio,
    pub mix: Ratio,
    pub labels: [String; 4],
}

/// The averages, checked against the words: A loses a cent a round, B a
/// little under a cent, halfway between is a loss, and the mix wins.
pub(super) fn averages(f: &Facts) -> Result<Averages, String> {
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    let (a, b, mix) = (
        PUBLISHED.game_a().mean(),
        PUBLISHED.game_b().mean(),
        PUBLISHED.mix().mean(),
    );
    let halfway = facts::halfway(&PUBLISHED);
    if a != Ratio::new(-1, 100) {
        return Err(format!("04_4 says game A loses a cent a round, but its average is {a}"));
    }
    if b >= Ratio::ZERO || b <= a {
        return Err(format!(
            "04_4 says game B loses a little under a cent, but its average is {b}"
        ));
    }
    if halfway >= Ratio::ZERO {
        return Err("04_4 says halfway between is still a loss".into());
    }
    if mix <= Ratio::ZERO {
        return Err("04_6 says the mix wins, but its average is not above zero".into());
    }
    for (key, x) in [
        ("pub.a.mean.cents", a),
        ("pub.b.mean.cents", b),
        ("pub.halfway.mean.cents", halfway),
        ("pub.mix.mean.cents", mix),
    ] {
        if x.cents(2) != get(key)? {
            return Err(format!("the bars disagree with the fact {key}"));
        }
    }
    Ok(Averages {
        a,
        b,
        halfway,
        mix,
        labels: [
            format!("{A_ROW} {}", screen(&a.cents(2))),
            format!("{B_ROW} {}", screen(&b.cents(2))),
            format!("{HALFWAY_ROW} {} {ASK}", screen(&halfway.cents(2))),
            format!("{MIX_NAME} {}", mix.cents(2)),
        ],
    })
}

/// 02_5's badge with the chances of the published coins.
pub(super) fn badge_text(f: &Facts) -> Result<String, String> {
    let [a, bad, good] = facts::coins(&PUBLISHED);
    let pcts = [a.exact_pct()?, bad.exact_pct()?, good.exact_pct()?];
    if f.get("coins.published.heads.pct") != Some(pcts.join(",").as_str()) {
        return Err("the badge's chances disagree with the facts".into());
    }
    Ok(format!(
        "{BADGE} \u{b7} {} \u{b7} {} \u{b7} {} %",
        pcts[0], pcts[1], pcts[2]
    ))
}

/// The chapter's numbers, checked against the words they illustrate.
struct Numbers {
    full_badge: String,
    ordinary_faces: [String; 2],
    merged_face: String,
    merged_tex: String,
    averages: Averages,
    a: model::Run,
    b: model::Run,
    mix: model::Run,
    end_line: String,
}

fn numbers() -> Result<Numbers, String> {
    let f = facts::facts()?;
    let get = |k: &str| f.get(k).map(str::to_owned).ok_or_else(|| format!("no fact {k}"));
    // 04_2: two ordinary coins that both lose; 04_3: a fair coin choosing
    // between them is exactly one coin halfway between, which loses
    let ordinary = ORDINARY.map(|p| Ratio::new(p.into(), 100));
    let merged = (ordinary[0] + ordinary[1]) / Ratio::int(2);
    let chosen = Game {
        up: [ORDINARY[0]; 3],
        den: 100,
    }
    .mixed(
        &Game {
            up: [ORDINARY[1]; 3],
            den: 100,
        },
        1,
        2,
    );
    if (0..3).any(|i| chosen.chance(i) != merged) {
        return Err("04_3 says the fair choice is one coin halfway between, but the model disagrees".into());
    }
    let half = Ratio::new(1, 2);
    if ordinary.iter().any(|&p| p >= half) || merged >= half || chosen.mean() >= Ratio::ZERO {
        return Err("04_2 and 04_3 say the ordinary coins and their mix all lose".into());
    }
    let pcts = [ordinary[0].exact_pct()?, ordinary[1].exact_pct()?];
    // exactly a whole number of tenths, so the screen writes =
    let merged_pct = merged.exact_pct()?;
    if pcts.join(",") != get("ordinary.coins.pct")? || merged_pct != get("ordinary.mixed.pct")? {
        return Err("04's ordinary coins disagree with the facts".into());
    }
    let a = model::player(Play::A);
    let b = model::player(Play::B);
    let mix = model::player(Play::Mix);
    // 04_6: the same player, a hundred thousand rounds, well over a
    // thousand dollars up
    if mix.dollars[0] != START || get("run.rounds")? != RUN_ROUNDS.to_string() {
        return Err("04_6 says the same player, but the run starts elsewhere".into());
    }
    if mix.net() <= WELL_OVER || mix.net().to_string() != get("run.mix.net")? {
        return Err(format!(
            "04_6 says well over a thousand dollars up, but the mix's run ends {}",
            mix.net()
        ));
    }
    Ok(Numbers {
        full_badge: badge_text(&f)?,
        ordinary_faces: [format!("{} %", pcts[0]), format!("{} %", pcts[1])],
        merged_face: format!("{merged_pct} %"),
        merged_tex: format!(
            r"\tfrac{{1}}{{2}} \cdot {}\% + \tfrac{{1}}{{2}} \cdot {}\% = {merged_pct}\%",
            pcts[0], pcts[1]
        ),
        averages: averages(&f)?,
        end_line: format!(
            "after {} rounds: {} dollars up",
            thousands(RUN_ROUNDS as u64),
            thousands(mix.net().unsigned_abs())
        ),
        a,
        b,
        mix,
    })
}

/// Where a chooser's arrow toward the ordinary coin on `side` (−1 left, +1
/// right) starts and ends, and where its "1/2" stands: off the arrow's
/// middle, outward and up, clear of the stroke.
fn half_arrow(side: f64) -> (Vec3, Vec3, Vec3) {
    let at = if side < 0.0 { ORD_AT[0] } else { ORD_AT[1] };
    let from = v(CHOOSER_AT[0] + side * CHOOSER_R, CHOOSER_AT[1] - 0.5 * CHOOSER_R);
    let to = v(at[0] - side * ARROW_INSET, at[1] + ORD_R + ARROW_CLEAR);
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = (-side * dy / len, side * dx / len);
    let label = v(
        (from[0] + to[0]) / 2.0 + HALF_OFF * nx,
        (from[1] + to[1]) / 2.0 + HALF_OFF * ny,
    );
    (from, to, label)
}

// ------------------------------------------------- shared with chapter 05

/// The badge with the chances, its right edge kept inside the frame and its
/// left edge right of `left`.
pub(super) fn full_badge(kit: &Kit, stage: &mut Stage<'_>, text: &str, left: f64) -> fmn::Result<[Mob; 2]> {
    let tag = badge(kit, stage, text, true)?;
    let right = FRAME_X_RADIUS - ROOM;
    if stage.get_bounding_box_point(tag[0], RIGHT)[0] > right {
        stage.move_to(tag[0], v(right, BADGE_AT[1]), RIGHT);
        stage.move_to(tag[1], v(right - 0.15, BADGE_AT[1]), RIGHT);
    }
    keep_between(stage, tag[0], left, right, "the coin badge")?;
    Ok(tag)
}

/// A long window's axes and ticks (rounds 0 to 100,000; dollars −1,000, 0
/// and +1,000 marked). Detached.
pub(super) fn window(kit: &Kit, stage: &mut Stage<'_>, frame: &Frame2) -> fmn::Result<Vec<Mob>> {
    let (left, right, bottom, top) = (frame.left(), frame.right(), frame.bottom(), frame.top());
    let corner = polyline([v(left, top), v(left, bottom), v(right, bottom)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut parts = vec![
        stage.arena_mut().add(corner),
        kit.text(stage, ZERO, 18.0, Palette::dim(), v(left, bottom - TICK_DY))?,
        kit.text(
            stage,
            &format!("{} {ROUNDS}", thousands(RUN_ROUNDS as u64)),
            18.0,
            Palette::dim(),
            v(right - 0.9, bottom - TICK_DY),
        )?,
        kit.text(stage, DOLLARS, 18.0, Palette::dim(), v(left, top + TICK_DY))?,
    ];
    for (dollars, words) in [
        (0, ZERO.to_owned()),
        (1000, format!("+{}", thousands(1000u64))),
        (-1000, signed_thousands(-1000)),
    ] {
        let y = frame.p(0.0, f64::from(dollars))[1];
        let tick = kit.text(stage, &words, 18.0, Palette::dim(), v(left, y))?;
        stage.move_to(tick, v(left - TICK_DX, y), RIGHT);
        keep_between(stage, tick, -FRAME_X_RADIUS + EDGE, left, "a dollars tick")?;
        parts.push(tick);
    }
    Ok(parts)
}

/// A whole run drawn faint along `frame` (stroke only: no fill, so it stays
/// a line). Detached.
pub(super) fn faint_run(stage: &mut Stage<'_>, frame: &Frame2, run: &model::Run, color: Srgb) -> Mob {
    let points: Vec<Vec3> = thinned(&run.dollars, THIN)
        .iter()
        .map(|&(k, d)| frame.p(k, d))
        .collect();
    stage.arena_mut().add(
        polyline(points)
            .style(Style::default().fill(color, 0.0).stroke(color, 2.0, FAINT))
            .build(),
    )
}

/// A run's name at its end, just right of `frame`. Detached.
pub(super) fn end_label(
    kit: &Kit,
    stage: &mut Stage<'_>,
    frame: &Frame2,
    run: &model::Run,
    words: &str,
    color: Srgb,
) -> fmn::Result<Mob> {
    let end = *run.dollars.last().expect("a run has rounds") as f64;
    let m = kit.text_left(
        stage,
        words,
        20.0,
        color,
        v(frame.right() + END_LABEL_DX, frame.p(0.0, end)[1]),
    )?;
    keep_between(stage, m, frame.right(), FRAME_X_RADIUS - EDGE, "a run's end label")?;
    Ok(m)
}

/// The player as 01 and 02 left them: token, "player", the count. Detached.
fn player_home(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 4]> {
    let player = token(stage, TOKEN_AT);
    let label = kit.text(stage, PLAYER, 18.0, Palette::dim(), PLAYER_LABEL_AT)?;
    let count = kit.text_left(stage, &START.to_string(), 34.0, palette::player(), COUNT_AT)?;
    let dollars = kit.text_left(stage, DOLLARS, 26.0, Palette::dim(), v(DOLLARS_X, COUNT_AT[1]))?;
    keep_between(stage, dollars, DOLLARS_X - 0.01, COUNT_RIGHT, "the player's count")?;
    Ok([player, label, count, dollars])
}

/// A bar's length for an average of `x` dollars a round, to scale.
pub(super) fn bar_len(x: Ratio) -> f64 {
    (x * Ratio::int(100)).to_f64().abs() * CENT_U
}

/// The right edge of the bars' labels: left of the longest bar growing
/// left from zero.
pub(super) fn label_right(av: &Averages) -> f64 {
    ZERO_X - [av.a, av.b, av.halfway].into_iter().map(bar_len).fold(0.0, f64::max) - LABEL_GAP
}

/// The averages' bars, detached: the zero line with its "0" and caption;
/// A's and B's bars; halfway's dashed outline; the mix's bar; and the four
/// labels (A, B, halfway, the mix).
pub(super) struct Bars {
    pub axis: Vec<Mob>,
    pub games: [Mob; 2],
    pub halfway: Vec<Mob>,
    pub mix: Mob,
    pub labels: Vec<Mob>,
}

pub(super) fn bars(kit: &Kit, stage: &mut Stage<'_>, av: &Averages) -> fmn::Result<Bars> {
    let top = ROW_YS[0] + BAR_H / 2.0 + AXIS_OVER;
    let bottom = ROW_YS[3] - BAR_H / 2.0 - AXIS_OVER;
    let zero = polyline([v(ZERO_X, top), v(ZERO_X, bottom)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let axis = vec![
        stage.arena_mut().add(zero),
        kit.text(stage, ZERO, 18.0, Palette::dim(), v(ZERO_X, top + 0.2))?,
        kit.text(
            stage,
            BARS_CAPTION,
            20.0,
            Palette::dim(),
            v(ZERO_X, bottom - CAPTION_DY),
        )?,
    ];
    keep_between(stage, axis[2], COUNT_RIGHT, FRAME_X_RADIUS - EDGE, "the bars' caption")?;
    let mut bar = |x: Ratio, y: f64, color: Srgb| -> fmn::Result<Mob> {
        let len = bar_len(x);
        let left = if x < Ratio::ZERO { ZERO_X - len } else { ZERO_X };
        Ok(stage
            .arena_mut()
            .add(rect(v(left, y - BAR_H / 2.0), len, BAR_H, flat(color, 0.85))?))
    };
    let games = [
        bar(av.a, ROW_YS[0], palette::game_a())?,
        bar(av.b, ROW_YS[1], palette::game_b())?,
    ];
    let mix = bar(av.mix, ROW_YS[3], palette::mix())?;
    // halfway: the same length to scale, only outlined, a guess
    let (l, r) = (ZERO_X - bar_len(av.halfway), ZERO_X);
    let (t, b) = (ROW_YS[2] + BAR_H / 2.0, ROW_YS[2] - BAR_H / 2.0);
    let mut halfway = Vec::new();
    for (from, to) in [
        (v(l, t), v(r, t)),
        (v(r, t), v(r, b)),
        (v(r, b), v(l, b)),
        (v(l, b), v(l, t)),
    ] {
        halfway.extend(dashed(stage, from, to, DASH, palette::mix(), 2.5));
    }
    let right = label_right(av);
    let colors = [palette::game_a(), palette::game_b(), palette::mix(), palette::mix()];
    let mut labels = Vec::with_capacity(4);
    for ((words, color), y) in av.labels.iter().zip(colors).zip(ROW_YS) {
        let m = kit.text(stage, words, 24.0, color, v(right, y))?;
        stage.move_to(m, v(right, y), RIGHT);
        keep_between(stage, m, COUNT_RIGHT, right + 0.01, "a bar's label")?;
        labels.push(m);
    }
    Ok(Bars {
        axis,
        games,
        halfway,
        mix,
        labels,
    })
}

/// Grow `bar` from its `edge` (the zero line's side) over `rt` seconds,
/// fading `with` in alongside.
pub(super) fn grow(stage: &mut Stage<'_>, bar: Mob, edge: Vec3, rt: f64, with: &[Mob]) -> fmn::Result<()> {
    let g = fmn::animation::grow_from_edge(stage.arena_mut(), bar, edge, None)?;
    let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(g.rt(rt))?];
    for &m in with {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// A frame around `mobs` as typeset (the box of them all), `pad` outside
/// it, in a stroke of `color` not yet seen ([`point_at`] brings it up).
/// Detached.
pub(super) fn frame_around(stage: &mut Stage<'_>, mobs: &[Mob], pad: f64, color: Srgb) -> fmn::Result<Mob> {
    if mobs.is_empty() {
        return Err(fail("a pointing frame around nothing".into()));
    }
    let (mut l, mut b, mut r, mut t) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for &m in mobs {
        l = l.min(stage.get_bounding_box_point(m, LEFT)[0]);
        b = b.min(stage.get_bounding_box_point(m, DOWN)[1]);
        r = r.max(stage.get_bounding_box_point(m, RIGHT)[0]);
        t = t.max(stage.get_bounding_box_point(m, UP)[1]);
    }
    let unseen = Style::default().fill(color, 0.0).stroke(color, POINT_W, 0.0);
    let frame = rect(v(l - pad, b - pad), r - l + 2.0 * pad, t - b + 2.0 * pad, unseen)?;
    Ok(stage.arena_mut().add(frame))
}

/// Point at words without changing them: `frame` (from [`frame_around`])
/// comes up and goes again while `shapes` (a mark on a scale) pulse, in one
/// play of `rt` seconds, as many frames as the flash it replaces (a play
/// is rounded up to whole frames, so two half-plays can run one longer),
/// and the picture is left as it was. An indicate or recolour of typeset
/// text has shown doubled glyphs in native scenes, so nothing here
/// animates the words.
pub(super) fn point_at(stage: &mut Stage<'_>, frame: Mob, shapes: &[Mob], rt: f64) -> fmn::Result<()> {
    stage.add_to_scene(frame)?;
    let up_and_back = frame
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(fmn::core::rate::there_and_back),
            ..AnimateArgs::default()
        })?
        .set_stroke(None, None, Some(1.0))?;
    let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(up_and_back)?];
    for &m in shapes {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    stage.scene_mut().remove(&[frame]);
    Ok(())
}

impl SceneConstruct for LooksHopeless {
    fn name(&self) -> &str {
        "04_looks_hopeless"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let n = numbers().map_err(fail)?;

        // 04_1: the player, and the mix's fair coin: heads A, tails B
        let line = say(&kit, stage, "04_1")?;
        let home = player_home(&kit, stage)?;
        let tag = full_badge(&kit, stage, &n.full_badge, CHOOSER_AT[0] + CHOOSER_R)?;
        let mut arriving = home.to_vec();
        arriving.extend(tag);
        fade_in_all(stage, &arriving, 0.5)?;
        let player = home[0];
        line.wait_for(stage, AT_FLIPS.1)?;
        let mix_coin = coin(&kit, stage, MIX_COIN_AT, MIX_COIN_R, palette::mix(), HEADS_FACE)?;
        fade_in_all(stage, &mix_coin, 0.4)?;
        flash_all(stage, &[mix_coin[0]], 0.6)?;
        line.wait_for(stage, AT_HEADS.1)?;
        let heads = kit.text_left(stage, HEADS_RULE, 24.0, palette::game_a(), v(RULE_X, HEADS_Y))?;
        let tails = kit.text_left(stage, TAILS_RULE, 24.0, palette::game_b(), v(RULE_X, TAILS_Y))?;
        keep_between(stage, heads, RULE_X - 0.01, COUNT_RIGHT, "the mix's rules")?;
        keep_between(stage, tails, RULE_X - 0.01, COUNT_RIGHT, "the mix's rules")?;
        fade_in_all(stage, &[heads], 0.4)?;
        line.wait_for(stage, AT_TAILS.1)?;
        // the coin lands the other way: its face turns from A to B
        let tails_face = kit.text(
            stage,
            TAILS_FACE,
            22.0,
            explainer_kit::hex(explainer_kit::BACKGROUND),
            MIX_COIN_AT,
        )?;
        fade_out_all(stage, &[mix_coin[1]], 0.2)?;
        fade_in_all(stage, &[tails_face, tails], 0.4)?;
        line.wait_for(stage, AT_CALL.1)?;
        let name = kit.text(stage, MIX_NAME, 26.0, palette::mix(), MIX_NAME_AT)?;
        keep_between(stage, name, -FRAME_X_RADIUS + EDGE, COUNT_RIGHT, "the mix's name")?;
        fade_in_all(stage, &[name], 0.4)?;
        let rules = [heads, tails];

        // 04_2: two ordinary coins that both lose, 49 % and 48 %
        let line = say(&kit, stage, "04_2")?;
        line.wait_for(stage, AT_ORDINARY.1)?;
        let mut ordinary: Vec<[Mob; 3]> = Vec::with_capacity(2);
        for (at, face) in ORD_AT.into_iter().zip(&n.ordinary_faces) {
            let c = coin(&kit, stage, at, ORD_R, Palette::ink(), face)?;
            keep_between(stage, c[1], at[0] - ORD_R, at[0] + ORD_R, "an ordinary coin's face")?;
            let loses = kit.text(stage, LOSES, 22.0, palette::bad(), v(at[0], at[1] - ORD_R - LOSES_GAP))?;
            ordinary.push([c[0], c[1], loses]);
        }
        let ordinary_parts: Vec<Mob> = ordinary.iter().flatten().copied().collect();
        fade_in_all(stage, &ordinary_parts, 0.5)?;
        line.wait_for(stage, AT_49.1)?;
        flash_all(stage, &[ordinary[0][0]], 0.6)?;
        line.wait_for(stage, AT_48.1)?;
        flash_all(stage, &[ordinary[1][0]], 0.6)?;

        // 04_3: a fair coin choosing between them is one coin halfway
        // between, 48.5 %, and it loses
        let line = say(&kit, stage, "04_3")?;
        line.wait_for(stage, AT_CHOOSE.1)?;
        let chooser = stage.arena_mut().add(
            Circle::new()
                .radius(CHOOSER_R)
                .arc_center(CHOOSER_AT)
                .style(flat(palette::mix(), 1.0))
                .build(),
        );
        let mut choosing = vec![chooser];
        for side in [-1.0, 1.0] {
            let (from, to, at) = half_arrow(side);
            choosing.extend(arrow(stage, from, to, Palette::dim(), 2.0));
            choosing.push(kit.text(stage, HALF, 20.0, Palette::dim(), at)?);
        }
        fade_in_all(stage, &choosing, 0.5)?;
        flash_all(stage, &[chooser], 0.6)?;
        line.wait_for(stage, AT_ONE_COIN.1)?;
        // the faces and verdicts go, the two coins meet halfway and become
        // one, and the one coin gets its own face and verdict
        let mut going: Vec<Mob> = ordinary.iter().flat_map(|c| [c[1], c[2]]).collect();
        going.extend(&choosing);
        fade_out_all(stage, &going, 0.3)?;
        together(
            stage,
            vec![
                fly(ordinary[0][0], MERGED_AT, 0.7)?,
                fly(ordinary[1][0], MERGED_AT, 0.7)?,
            ],
        )?;
        fade_out_all(stage, &[ordinary[1][0]], 0.1)?;
        let merged_disc = ordinary[0][0];
        let merged_face = kit.text(
            stage,
            &n.merged_face,
            22.0,
            explainer_kit::hex(explainer_kit::BACKGROUND),
            MERGED_AT,
        )?;
        keep_between(
            stage,
            merged_face,
            MERGED_AT[0] - ORD_R,
            MERGED_AT[0] + ORD_R,
            "the merged coin's face",
        )?;
        let merged_loses = kit.text(
            stage,
            LOSES,
            22.0,
            palette::bad(),
            v(MERGED_AT[0], MERGED_AT[1] - ORD_R - LOSES_GAP),
        )?;
        let tex = kit.tex(stage, &n.merged_tex, 30.0, Palette::ink(), &[], TEX_AT)?;
        keep_between(stage, tex, COUNT_RIGHT, FRAME_X_RADIUS - EDGE, "the halfway coin's sum")?;
        fade_in_all(stage, &[merged_face, merged_loses, tex], 0.4)?;
        line.wait_for(stage, AT_LOSER.1)?;
        let pointer = frame_around(stage, &[merged_loses], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;

        // 04_4: the games' averages, to scale, and halfway between
        let line = say(&kit, stage, "04_4")?;
        fade_out_all(stage, &[merged_disc, merged_face, merged_loses, tex], 0.4)?;
        let bars = bars(&kit, stage, &n.averages)?;
        fade_in_all(stage, &bars.axis, 0.4)?;
        line.wait_for(stage, AT_A_CENT.1)?;
        grow(stage, bars.games[0], RIGHT, 0.8, &[bars.labels[0]])?;
        line.wait_for(stage, AT_B_CENT.1)?;
        grow(stage, bars.games[1], RIGHT, 0.8, &[bars.labels[1]])?;
        line.wait_for(stage, AT_HALFWAY.1)?;
        let mut guess = bars.halfway.clone();
        guess.push(bars.labels[2]);
        fade_in_all(stage, &guess, 0.6)?;

        // 04_5: win or lose in the long run? (a prediction; the dashed bar
        // waits)
        say(&kit, stage, "04_5")?;
        let q = think(&kit, stage, THINK_AT)?;

        // 04_6: they win; the same player, 100,000 rounds of the mix
        let line = say(&kit, stage, "04_6")?;
        fade_out_all(stage, &[q], 0.2)?;
        line.wait_for(stage, AT_PLAYER.1)?;
        let mut leaving = bars.axis.clone();
        leaving.extend(bars.games);
        leaving.extend(&guess);
        leaving.extend(&bars.labels[..2]);
        leaving.extend(rules);
        leaving.extend(&home[1..]);
        fade_out_all(stage, &leaving, 0.4)?;
        let mut picture = window(&kit, stage, &WIDE)?;
        let faint = [
            faint_run(stage, &WIDE, &n.a, palette::game_a()),
            faint_run(stage, &WIDE, &n.b, palette::game_b()),
        ];
        let ends = [
            end_label(&kit, stage, &WIDE, &n.a, A_END, palette::game_a())?,
            end_label(&kit, stage, &WIDE, &n.b, B_END, palette::game_b())?,
        ];
        picture.extend(faint);
        picture.extend(ends);
        fade_in_all(stage, &picture, 0.5)?;
        together(
            stage,
            vec![
                fade_to(ends[0], FAINT, 0.3)?,
                fade_to(ends[1], FAINT, 0.3)?,
                fly(player, WIDE.p(0.0, START as f64), 0.6)?,
            ],
        )?;
        line.wait_for(stage, AT_RUN.1)?;
        let points = thinned(&n.mix.dollars, THIN);
        let (mix_line, anchor, progress) = growing_run(stage, &WIDE, &points, palette::mix(), Some(player))?;
        fade_in_all(stage, &[anchor, mix_line], 0.1)?;
        draw_run(stage, progress, RUN_S)?;
        stage.clear_updaters(anchor, true);
        let mix_end = end_label(&kit, stage, &WIDE, &n.mix, MIX_END, palette::mix())?;
        let end = kit.text(stage, &n.end_line, 24.0, palette::mix(), END_AT)?;
        keep_between(stage, end, WIDE.left(), FRAME_X_RADIUS - EDGE, "the end readout")?;
        fade_in_all(stage, &[mix_end, end], 0.5)?;
        line.wait_for(stage, AT_WELL_OVER.1)?;
        let pointer = frame_around(stage, &[end], POINT_PAD, Palette::hero())?;
        point_at(stage, pointer, &[], 0.8)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};
    use fmn::core::constants::FRAME_Y_RADIUS;
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::THINK_S;
    use super::*;

    /// The chapter's words are true of its numbers: the ordinary coins are
    /// 49 % and 48 % and their fair mix exactly 48.5 %; the bars read A
    /// −1.00, B −0.87, halfway −0.93 and the mix +1.57 cents a round; and
    /// the mix's run ends 1,748 dollars up.
    #[test]
    fn looks_hopeless_numbers_are_the_words() {
        let n = numbers().unwrap();
        assert_eq!(n.ordinary_faces, ["49 %", "48 %"]);
        assert_eq!(n.merged_face, "48.5 %");
        assert_eq!(
            n.merged_tex,
            r"\tfrac{1}{2} \cdot 49\% + \tfrac{1}{2} \cdot 48\% = 48.5\%"
        );
        assert_eq!(
            n.averages.labels,
            [
                "game A \u{2212}1.00",
                "game B \u{2212}0.87",
                "halfway: \u{2212}0.93 ?",
                "the mix +1.57"
            ]
        );
        assert_eq!(n.full_badge, "published coins \u{b7} 49.5 \u{b7} 9.5 \u{b7} 74.5 %");
        assert_eq!((n.a.net(), n.b.net(), n.mix.net()), (-1062, -548, 1748));
        assert_eq!(n.end_line, "after 100,000 rounds: 1,748 dollars up");
        assert!(n.mix.net() > WELL_OVER);
    }

    /// Negative control, the first attractive wrong answer, which is right
    /// for ordinary coins: any two coins below a half, chosen between by a
    /// fair coin, make one coin exactly halfway between, below a half. The
    /// model's mixing agrees for every pair from 1 % to 49 %, and the facts'
    /// sweep (shares a tenth, a half, nine tenths) finds no winner.
    #[test]
    fn two_losing_coins_always_mix_into_a_loser() {
        for p in 1..50 {
            for q in 1..50 {
                let mixed = Game { up: [p; 3], den: 100 }.mixed(&Game { up: [q; 3], den: 100 }, 1, 2);
                let halfway = Ratio::new((p + q).into(), 200);
                assert!((0..3).all(|i| mixed.chance(i) == halfway), "{p} and {q}");
                assert!(halfway < Ratio::new(1, 2) && mixed.mean() < Ratio::ZERO, "{p} and {q}");
            }
        }
        let f = facts::facts().unwrap();
        assert_eq!(f.get("ordinary.sweep.cases"), Some("7203"));
        assert_eq!(f.get("ordinary.sweep.winning"), Some("0"));
    }

    /// Negative control, the second attractive wrong answer: the mix's
    /// coins are exactly the average of A's and B's at every remainder, as
    /// with the ordinary coins, yet the average of the two games' averages
    /// (a loss) is not the mix's average (a gain). Averaging the chances is
    /// right; averaging the averages is not, because where the player
    /// stands changes.
    #[test]
    fn the_average_of_the_averages_is_not_the_mix() {
        let av = averages(&facts::facts().unwrap()).unwrap();
        let (a, b, mix) = (PUBLISHED.game_a(), PUBLISHED.game_b(), PUBLISHED.mix());
        for i in 0..3 {
            assert_eq!(
                mix.chance(i),
                (a.chance(i) + b.chance(i)) / Ratio::int(2),
                "remainder {i}"
            );
        }
        assert_eq!(av.halfway, (av.a + av.b) / Ratio::int(2));
        assert!(av.halfway < Ratio::ZERO && av.mix > Ratio::ZERO);
        assert_eq!(av.mix, Ratio::new(223, 14200));
    }

    /// The bars are to scale: each length is its average in cents times the
    /// same unit, A's the longest leftward, the labels' column clear of it,
    /// the mix's bar inside the frame, the rows apart.
    #[test]
    fn the_bars_are_drawn_to_scale() {
        let av = averages(&facts::facts().unwrap()).unwrap();
        assert!((bar_len(av.a) - CENT_U).abs() < 1e-12);
        assert!((bar_len(av.b) / bar_len(av.a) - 73443.0 / 84463.0).abs() < 1e-12);
        assert!(bar_len(av.a) > bar_len(av.halfway) && bar_len(av.halfway) > bar_len(av.b));
        assert!(label_right(&av) < ZERO_X - bar_len(av.a));
        assert!(ZERO_X + bar_len(av.mix) < FRAME_X_RADIUS - ROOM);
        assert!(ROW_YS.windows(2).all(|w| w[0] - w[1] > BAR_H + 0.2));
    }

    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in CUES {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its rooms: the mix's coin, name and rules sit above
    /// the player's count; the ordinary coins, their chooser and the merged
    /// coin stay inside the frame, under the badge and above the sum; each
    /// "1/2" stays off its arrow; the think ring clears the frame's foot,
    /// the player and the bars' labels; every drawn point of the three runs
    /// stays inside the window, and the mix's run with the token riding it
    /// stays below the readout; the end labels keep apart.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let n = numbers().unwrap();
        const {
            assert!(MIX_COIN_AT[1] - MIX_COIN_R > COUNT_AT[1] + 0.3);
            assert!(TAILS_Y - 0.15 > COUNT_AT[1] + 0.3 && HEADS_Y - TAILS_Y > 0.35);
            assert!(MIX_NAME_AT[1] + 0.2 < FRAME_Y_RADIUS - EDGE);
            assert!(MIX_NAME_AT[1] - 0.2 > MIX_COIN_AT[1] + MIX_COIN_R);
            assert!(ORD_AT[1][0] - ORD_AT[0][0] > 2.0 * ORD_R + 1.0);
            assert!(CHOOSER_AT[1] + CHOOSER_R < BADGE_AT[1] - 0.4);
            assert!(THINK_AT[1] - NOTE_DROP - 0.15 > -FRAME_Y_RADIUS + 0.15);
            assert!(THINK_AT[0] + THINK_R < COUNT_RIGHT && THINK_AT[1] + THINK_R < PLAYER_LABEL_AT[1] - 0.2);
            assert!(END_AT[1] + 0.2 < BADGE_AT[1] - 0.2);
        }
        for at in ORD_AT {
            assert!(at[0] - ORD_R > COUNT_RIGHT && at[0] + ORD_R < FRAME_X_RADIUS - ROOM);
            assert!(at[1] - ORD_R - LOSES_GAP - 0.2 > TEX_AT[1] + 0.3);
            assert!(CHOOSER_AT[1] - CHOOSER_R > at[1] + ORD_R + 0.3);
        }
        // each "1/2" (about 0.32 by 0.14 at 20 px) stands wholly on the
        // outer side of its arrow's line, clear of the stroke
        for side in [-1.0, 1.0] {
            let (from, to, at) = half_arrow(side);
            let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
            let len = (dx * dx + dy * dy).sqrt();
            let off = |x: f64, y: f64| ((x - from[0]) * dy - (y - from[1]) * dx) / len;
            let centre = off(at[0], at[1]);
            for (cx, cy) in [(-0.17, -0.08), (0.17, -0.08), (-0.17, 0.08), (0.17, 0.08)] {
                let d = off(at[0] + cx, at[1] + cy);
                assert!(d * centre.signum() > 0.1, "a 1/2 touches its arrow ({d:.3})");
            }
            assert!(at[1] + 0.08 < BADGE_AT[1] - 0.3);
        }
        // the ticks ("+1,000", about 0.55 wide) and the end labels ("mix",
        // about 0.35) fit inside the title-safe band, clear of the token
        // waiting at a run's start or end
        assert!(WIDE.left() - TICK_DX - 0.6 > -FRAME_X_RADIUS + EDGE);
        assert!(WIDE.right() + END_LABEL_DX + 0.4 < FRAME_X_RADIUS - EDGE);
        assert!(WIDE.bottom() - TICK_DY - 0.15 > -FRAME_Y_RADIUS + 0.15);
        // The chooser sits above the dollars ticks; their horizontal
        // projections may meet without the objects meeting in the picture.
        for dollars in [-1000, 0, 1000] {
            let tick_top = WIDE.p(0.0, f64::from(dollars))[1] + 0.15;
            assert!(MIX_COIN_AT[1] - MIX_COIN_R > tick_top + ROOM);
        }
        let readout_floor = END_AT[1] - 0.17;
        for run in [&n.a, &n.b, &n.mix] {
            for (k, d) in thinned(&run.dollars, THIN) {
                let [_, y, _] = WIDE.p(k, d);
                assert!(y > WIDE.bottom() && y < WIDE.top(), "round {k} leaves the window");
            }
        }
        for (k, d) in thinned(&n.mix.dollars, THIN) {
            let [_, y, _] = WIDE.p(k, d);
            assert!(
                y + TOKEN_R < readout_floor,
                "round {k} at {d} dollars reaches the readout ({y:.2})"
            );
        }
        let end_y = |run: &model::Run| WIDE.p(0.0, *run.dollars.last().unwrap() as f64)[1];
        let (a_y, b_y, mix_y) = (end_y(&n.a), end_y(&n.b), end_y(&n.mix));
        assert!(b_y - a_y > 0.4 && mix_y - b_y > 0.4, "the end labels overlap");
    }

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// The left, right, bottom and top points of `m`'s box.
    fn corners(stage: &Stage<'_>, m: Mob) -> [Vec3; 4] {
        [LEFT, RIGHT, DOWN, UP].map(|d| stage.get_bounding_box_point(m, d))
    }

    /// Pointing at words leaves them as typeset, in the old flash's time:
    /// the frame is the words' box and the pad each side; it takes exactly
    /// as long as a flash of the same length; afterwards the words' box is
    /// unchanged, a mark pulsed alongside is back in its place, and the draw
    /// list holds what it held before (the frame gone, no copy left behind).
    #[test]
    fn pointing_leaves_the_words_as_typeset() {
        run(|stage| {
            let kit = Kit::new()?;
            let words = kit.text(stage, LOSES, 22.0, palette::bad(), v(2.1, 0.15))?;
            let tick = polyline([v(-1.0, 1.05), v(-1.0, 1.45)])
                .style(line_style(palette::mix(), 4.0))
                .build();
            let tick = stage.arena_mut().add(tick);
            fade_in_all(stage, &[words, tick], 0.2)?;
            let (words_box, tick_box) = (corners(stage, words), corners(stage, tick));
            let before: HashSet<Mob> = stage.scene().mobjects().iter().copied().collect();
            let frame = frame_around(stage, &[words], POINT_PAD, Palette::hero())?;
            let (fc, wc) = (stage.get_center(frame), stage.get_center(words));
            // Native point buffers use f32 records; a newly built rectangle
            // rounds its dimensions once. This is below 0.001 pixel at 4K.
            const GEOMETRY_EPS: f64 = 1e-6;
            let width_error = stage.get_width(frame) - stage.get_width(words) - 2.0 * POINT_PAD;
            let height_error = stage.get_height(frame) - stage.get_height(words) - 2.0 * POINT_PAD;
            assert!(width_error.abs() < GEOMETRY_EPS, "frame width error: {width_error}");
            assert!(height_error.abs() < GEOMETRY_EPS, "frame height error: {height_error}");
            assert!((fc[0] - wc[0]).abs() < GEOMETRY_EPS && (fc[1] - wc[1]).abs() < GEOMETRY_EPS);
            let start = now(stage);
            flash_all(stage, &[tick], 0.8)?;
            let flash_s = now(stage) - start;
            let start = now(stage);
            point_at(stage, frame, &[tick], 0.8)?;
            let point_s = now(stage) - start;
            assert!(
                (point_s - flash_s).abs() < 1e-9,
                "{point_s} s against the flash's {flash_s} s"
            );
            assert_eq!(corners(stage, words), words_box, "the words moved");
            for (now_at, then_at) in corners(stage, tick).iter().zip(&tick_box) {
                for (x, x0) in now_at.iter().zip(then_at) {
                    assert!(
                        (x - x0).abs() < GEOMETRY_EPS,
                        "the mark is not back in place: {x} vs {x0}"
                    );
                }
            }
            let after: HashSet<Mob> = stage.scene().mobjects().iter().copied().collect();
            assert_eq!(after, before, "the draw list changed");
            assert!(frame_around(stage, &[], POINT_PAD, Palette::hero()).is_err());
            Ok(())
        });
    }

    /// The chapter runs headless to its end with the scratch voice and real
    /// typesetting (so every `keep_between` guard meets the true text
    /// widths): every line of chapter 04 is said once, in order, and the
    /// prediction (04_5) gets its silent think time before the answer.
    #[test]
    fn the_chapter_runs_and_leaves_room_to_think() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = LooksHopeless { kit: kit.clone() };
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut NullSceneSink).unwrap_or_else(|e| panic!("04_looks_hopeless: {e}"));
        let said = narrator.said();
        let ids: Vec<&str> = said.iter().map(|(id, _)| id.as_str()).collect();
        let expected: Vec<String> = (1..=6).map(|k| format!("04_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        let at = |id: &str| said.iter().find(|(l, _)| l == id).unwrap().1;
        let asked_end = at("04_5") + narrator.length("04_5").unwrap();
        assert!(
            at("04_6") - asked_end >= THINK_S,
            "{} s to think",
            at("04_6") - asked_end
        );
        let end = kit.finished.borrow().as_ref().expect("a timeline").seconds();
        assert!(end > at("04_6"), "the chapter ends after its last line: {end}");
    }
}
