//! Chapter 15, three shares: the door, explained. Read at position 0,
//! chapter 14's helpers give the trio's weights, 3, −3 and 1, which use only
//! the positions 1, 2 and 3 (15_1, 15_2). On paper the weights turn the
//! heights 19, 47 and 89 into the secret, 57 − 141 + 89 = 5 (15_3). The
//! trustees hold only the wrapped cards 8, 3 and 1; asked whether the same
//! weights work on those (15_4), they do: 24 − 9 + 1 = 16, which lands on 5,
//! the vault's needle runs sixteen marks, once round and on to 5, and the
//! door opens (15_5). Why: each height is its card and whole elevens (15_6),
//! and whole weights turn whole elevens into whole elevens, so the sums on
//! paper and on the cards are whole elevens apart, 16 = 5 + 1 × 11; on the
//! dial the weights are 3, 8 and 1 (15_7).
//!
//! The scene opens on chapter 14's last picture, built by the same
//! [`Setting`]. Every number it writes comes from `model` through
//! [`Working`], and the scene stops unless the weights are whole, the paper
//! sum is the secret, each height is its card and whole elevens, the cards'
//! sum lands on the secret and the two sums are whole elevens apart.
//! Emphasis never restyles or scales typeset text: rings and outlines of
//! their own point at it, and only shapes (dots, the door) change color or
//! swell.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::a_ten_mark_dial::text_after;
use super::draw::{
    DOOR_X, Rail, Spoken, Vault, card, disc, fade_in_all, fail, pause, read_by, recolor_all, recolorings, say, swaps,
    think, trustee_at, wait_until,
};
use super::envelope::{dealt, door_glow, frame_halos, outline};
use super::helper_curves::{
    DOT_R, HELPER_X0, HELPER_Y, POINT_R, Setting, TAG_SIZE, TAG_X, UNKNOWN_SIZE, WORK_SIZE, WORK_X, WORK_Y,
    check_helpers, column_outline, divide_text, on_board, paper_at, paper_trio, product_text, signed,
};
use super::two_keys::{TEXT_PAD, glow, glowing, leaving, moving, swell_all, text_halo};
use super::two_shares::disc_ring;
use crate::model::{self, Dial, MARKS, OPENING_TRIO, Ratio, Recipe, TRUSTEES};
use crate::palette;

pub struct ThreeShares {
    pub kit: Kit,
}

// ------------------------------------------------------------------ layout

/// The weights by the boards' side, left of position 0's dots (the side has
/// no labels of its own), and their size.
pub const WEIGHT_DX: f64 = 0.36;
pub const WEIGHT_SIZE: f64 = 16.0;
/// The weights under the rail's discs.
pub const RAIL_WEIGHT_Y: f64 = -2.66;
pub const RAIL_WEIGHT_SIZE: f64 = 16.0;
/// The space between a weight and its tag, and between a tag and its term
/// (`a_ten_mark_dial::text_after`'s).
const PIECE_GAP: f64 = 0.14;
/// The think ring, where the plot was (15_4).
const THINK_X: f64 = DOOR_X;
const THINK_Y: f64 = 1.1;
/// Why it works, at the upper right: each height split into its card and
/// elevens, and the weighted elevens under them.
pub const SPLIT_X: f64 = 0.8;
pub const SPLIT_Y: [f64; 3] = [2.9, 2.35, 1.8];
pub const SPLIT_SIZE: f64 = 26.0;
pub const ELEVENS_Y: f64 = 1.1;
pub const ELEVENS_SIZE: f64 = 20.0;
/// The needle's run through the cards' sum, the hold on the open door, and
/// the hold after the card is read.
const NEEDLE_RT: f64 = 2.0;
const DOOR_HOLD: f64 = 0.8;
const HOLD_AT_END: f64 = 0.6;

// -------------------------------------------------------------------- data

/// The chapter's arithmetic, from the model: each trustee's weight, whole;
/// the heights on paper and the cards; the weighted terms and sums of both;
/// the whole elevens in each height and their weighted sum; and the weights
/// on the dial.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Working {
    pub weights: Vec<i64>,
    pub heights: Vec<i64>,
    pub shares: Vec<i64>,
    pub paper_terms: Vec<i64>,
    pub paper_sum: i64,
    pub share_terms: Vec<i64>,
    pub total: i64,
    pub elevens: Vec<i64>,
    pub weighted_elevens: i64,
    pub dial_weights: Vec<u32>,
}

impl Working {
    /// The working for the trustees at `trio` (position, card) whose
    /// heights on paper are `heights`, on `dial`. An error unless the
    /// weights are whole, each height is its card and whole marks, the
    /// paper sum is `secret`, the cards' sum lands on it, the two sums are
    /// whole marks apart, and the weights on the dial are the model's.
    pub fn new(dial: Dial, trio: &[(u32, u32)], heights: &[i64], secret: u32) -> Result<Self, String> {
        let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
        let weights = model::ordinary_weights(&xs)
            .into_iter()
            .map(|w| w.whole().ok_or_else(|| format!("the weight {w} is not whole")))
            .collect::<Result<Vec<i64>, String>>()?;
        let shares: Vec<i64> = trio.iter().map(|p| i64::from(p.1)).collect();
        let marks = i64::from(dial.marks());
        let mut elevens = Vec::with_capacity(heights.len());
        for (&h, &y) in heights.iter().zip(&shares) {
            if !(0..marks).contains(&y) || (h - y).rem_euclid(marks) != 0 {
                return Err(format!("{h} is not the card {y} and whole {marks}s"));
            }
            elevens.push((h - y) / marks);
        }
        let times = |values: &[i64]| -> Vec<i64> { weights.iter().zip(values).map(|(w, v)| w * v).collect() };
        let paper_terms = times(heights);
        let share_terms = times(&shares);
        let paper_sum: i64 = paper_terms.iter().sum();
        let total: i64 = share_terms.iter().sum();
        let weighted_elevens: i64 = weights.iter().zip(&elevens).map(|(w, k)| w * k).sum();
        if paper_sum != i64::from(secret) {
            return Err(format!("on paper the weights give {paper_sum}, not {secret}"));
        }
        if paper_sum != total + marks * weighted_elevens {
            return Err(format!(
                "{paper_sum} and {total} are not {weighted_elevens} {marks}s apart"
            ));
        }
        if dial.wrap(total) != secret {
            return Err(format!(
                "the cards' sum {total} lands on {}, not {secret}",
                dial.wrap(total)
            ));
        }
        let dial_weights: Vec<u32> = weights.iter().map(|&w| dial.wrap(w)).collect();
        if model::weights_at(dial, &xs, 0).as_deref() != Some(&dial_weights[..]) {
            return Err(format!("the dial's weights are not {dial_weights:?}"));
        }
        if model::rebuild(dial, trio) != Some(secret) {
            return Err("the model rebuilds another secret".into());
        }
        Ok(Self {
            weights,
            heights: heights.to_vec(),
            shares,
            paper_terms,
            paper_sum,
            share_terms,
            total,
            elevens,
            weighted_elevens,
            dial_weights,
        })
    }

    /// The opening trio's working for the worked example.
    pub fn example() -> Result<Self, String> {
        let dial = Dial::vault();
        let recipe = Recipe::example();
        let heights: Vec<i64> = paper_trio(&recipe).iter().map(|p| p.1).collect();
        Self::new(dial, &dealt(dial, &OPENING_TRIO), &heights, recipe.secret)
    }
}

/// 15_1 and 15_2's working for trustee `i` at position 0, and what comes
/// after it: "trustee 1 at 0: (0 − 2) × (0 − 3)", "= 6, ÷ 2: 3".
pub fn zero_working(xs: &[u32], i: usize) -> Result<[String; 2], String> {
    let at0 = model::helper_at(xs, i, 0);
    let own = model::helper_at_own(xs, i);
    let weight = Ratio::new(at0, own)
        .whole()
        .ok_or_else(|| format!("trustee {}'s weight is not whole", xs[i]))?;
    Ok([
        format!("trustee {} at 0: {}", xs[i], product_text(xs, i, "0")),
        format!("= {}, {}: {}", signed(at0), divide_text(own), signed(weight)),
    ])
}

/// A trustee's weight under its disc: "weight −3".
pub fn weight_tag(w: i64) -> String {
    format!("weight {}", signed(w))
}

/// Terms as a sum, the minus signs typeset: "57 − 141 + 89".
pub fn signed_sum(terms: &[i64]) -> String {
    let parts: Vec<String> = terms
        .iter()
        .enumerate()
        .map(|(k, &t)| match (k, t < 0) {
            (0, _) => signed(t),
            (_, true) => format!("\u{2212} {}", -t),
            (_, false) => format!("+ {t}"),
        })
        .collect();
    parts.join(" ")
}

/// Each weight times its piece, as a sum: "3 × 8 − 3 × 3 + 1 × 1".
fn weighted_terms(weights: &[i64], pieces: &[String]) -> String {
    let parts: Vec<String> = weights
        .iter()
        .zip(pieces)
        .enumerate()
        .map(|(k, (&w, p))| match (k, w < 0) {
            (0, _) => format!("{} \u{d7} {p}", signed(w)),
            (_, true) => format!("\u{2212} {} \u{d7} {p}", -w),
            (_, false) => format!("+ {w} \u{d7} {p}"),
        })
        .collect();
    parts.join(" ")
}

/// A weight in front of its scale tag (15_3): "3", "−3", "1".
pub fn weight_piece(w: i64) -> String {
    signed(w)
}

/// A weighted height's term after its tag: "= −141".
pub fn term_text(term: i64) -> String {
    format!("= {}", signed(term))
}

/// 15_3's sum on paper, and its total after it: "on paper: 57 − 141 + 89",
/// "= 5".
pub fn paper_line(work: &Working) -> [String; 2] {
    [
        format!("on paper: {}", signed_sum(&work.paper_terms)),
        format!("= {}", signed(work.paper_sum)),
    ]
}

/// 15_5's sum on the cards, its terms and total, and the mark it lands on:
/// "on the cards: 3 × 8 − 3 × 3 + 1 × 1", "= 24 − 9 + 1 = 16", "→ 5".
pub fn cards_line(dial: Dial, work: &Working) -> [String; 3] {
    let cards: Vec<String> = work.shares.iter().map(ToString::to_string).collect();
    [
        format!("on the cards: {}", weighted_terms(&work.weights, &cards)),
        format!("= {} = {}", signed_sum(&work.share_terms), work.total),
        format!("\u{2192} {}", dial.wrap(work.total)),
    ]
}

/// 15_6's row for one trustee: its height, its card and whole elevens:
/// "19 = 8 + 1 × 11".
pub fn split_row(h: i64, y: i64, k: i64, marks: u32) -> String {
    format!("{h} = {y} + {k} \u{d7} {marks}")
}

/// 15_7's weighted elevens: "3 × (1 × 11) − 3 × (4 × 11) + 1 × (8 × 11) =
/// −1 × 11".
pub fn elevens_line(work: &Working, marks: u32) -> String {
    let pieces: Vec<String> = work.elevens.iter().map(|k| format!("({k} \u{d7} {marks})")).collect();
    format!(
        "{} = {} \u{d7} {marks}",
        weighted_terms(&work.weights, &pieces),
        signed(work.weighted_elevens)
    )
}

/// 15_7's cards' sum split into its mark and whole elevens: "16 = 5 + 1 ×
/// 11".
pub fn turns_line(total: i64, marks: u32) -> String {
    let m = i64::from(marks);
    format!(
        "{total} = {} + {} \u{d7} {marks}",
        total.rem_euclid(m),
        total.div_euclid(m)
    )
}

/// 15_7's precisely card: with whole weights the mark does not depend on
/// when the heights are wrapped; the weights on the dial, and the method's
/// name.
pub fn card_text(dial_weights: &[u32]) -> String {
    let list = match dial_weights {
        [] => String::new(),
        [only] => only.to_string(),
        [rest @ .., last] => {
            let rest: Vec<String> = rest.iter().map(ToString::to_string).collect();
            format!("{} and {last}", rest.join(", "))
        }
    };
    format!(
        "wrapped before the sum or after, whole weights land on the same mark; on the dial they are {list} \
         (Lagrange interpolation)"
    )
}

// ---------------------------------------------------------------- drawing

/// Where helper `b`'s weight is written, by its board's side at height `w`.
pub fn weight_label_at(b: usize, w: i64) -> Vec3 {
    let p = on_board(b, 0.0, w as f64);
    v(HELPER_X0 - WEIGHT_DX, p[1])
}

/// Where trustee `t`'s weight sits under its disc on the rail.
pub fn rail_weight_at(t: u32) -> Vec3 {
    v(trustee_at(t)[0], RAIL_WEIGHT_Y)
}

/// An outline round all of `mobs` as they stand, in `color`. Detached.
fn around(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb) -> fmn::Result<Mob> {
    if mobs.is_empty() {
        return Err(fail("an outline round nothing".into()));
    }
    let (mut l, mut r, mut b, mut t) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &m in mobs {
        let (at, w, h) = (stage.get_center(m), stage.get_width(m), stage.get_height(m));
        l = l.min(at[0] - w / 2.0);
        r = r.max(at[0] + w / 2.0);
        b = b.min(at[1] - h / 2.0);
        t = t.max(at[1] + h / 2.0);
    }
    let centre = v((l + r) / 2.0, (b + t) / 2.0);
    outline(stage, centre, r - l + 2.0 * TEXT_PAD, t - b + 2.0 * TEXT_PAD, color)
}

/// The phrases 15_1 and 15_2 wait for as trustee `i`'s helper is read at
/// 0: its product there, and its value and weight. Trustee 1's weight is
/// named after a phrase of its own; 2's and 3's come with their values.
pub fn weight_cues(i: usize) -> [&'static str; 2] {
    match i {
        0 => [
            "Trustee one's is minus two times minus three",
            "which is six, halved, so three",
        ],
        1 => [
            "Trustee two's is minus one times minus three",
            "which is three, divided by minus one",
        ],
        _ => [
            "Trustee three's is minus one times minus two",
            "which is two, halved, so one",
        ],
    }
}

/// What reading one helper at 0 leaves: the working's two lines, the gold
/// dot where the helper meets position 0 and its weight there, and the
/// weight under the trustee's disc.
struct Read {
    lines: [Mob; 2],
    dot: Mob,
    label: Mob,
    tag: Mob,
}

/// Trustee `i`'s helper read at position 0 as `spoken` says it: the product
/// there under the boards, then its value and the weight, a gold dot on the
/// board at that height with the weight beside it, and the weight under the
/// disc (trustee 1's after "That's trustee one's weight").
fn read_weight(
    kit: &Kit,
    stage: &mut Stage<'_>,
    spoken: &Spoken,
    xs: &[u32],
    i: usize,
    working: &[String; 2],
    w: i64,
) -> fmn::Result<Read> {
    let cues = weight_cues(i);
    spoken.wait_for(stage, cues[0])?;
    let line = kit.text_left(stage, &working[0], WORK_SIZE, Palette::ink(), v(WORK_X, WORK_Y[i]))?;
    fade_in_all(stage, &[line], 0.5)?;
    spoken.wait_for(stage, cues[1])?;
    let value = text_after(kit, stage, line, &working[1], WORK_SIZE, Palette::ink())?;
    let dot = disc(stage, palette::secret(), on_board(i, 0.0, w as f64), DOT_R);
    let label = kit.text(stage, &signed(w), WEIGHT_SIZE, palette::trio(), weight_label_at(i, w))?;
    let tag = kit.text(
        stage,
        &weight_tag(w),
        RAIL_WEIGHT_SIZE,
        palette::trio(),
        rail_weight_at(xs[i]),
    )?;
    let mut anims = swaps(stage, &[], &[value, dot, label], 0.5)?;
    if i == 0 {
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "That's trustee one's weight")?;
        anims = Vec::new();
    }
    let r = disc_ring(stage, xs[i], palette::trio());
    anims.extend(swaps(stage, &[], &[tag], 0.5)?);
    anims.extend(glowing(stage, &[r], 1.0)?);
    stage.play_prepared(anims)?;
    Ok(Read {
        lines: [line, value],
        dot,
        label,
        tag,
    })
}

impl SceneConstruct for ThreeShares {
    fn name(&self) -> &str {
        "15_three_shares"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let recipe = Recipe::example();
        let shares = dealt(dial, &OPENING_TRIO);
        let trio = paper_trio(&recipe);
        check_helpers(&trio, &recipe).map_err(fail)?;
        let work = Working::example().map_err(fail)?;
        let xs: Vec<u32> = trio.iter().map(|p| p.0).collect();
        let zeros: Vec<[String; 2]> = (0..xs.len())
            .map(|i| zero_working(&xs, i))
            .collect::<Result<_, _>>()
            .map_err(fail)?;
        let paper_words = paper_line(&work);
        let cards_words = cards_line(dial, &work);
        let card_words = card_text(&work.dial_weights);
        let others: Vec<u32> = TRUSTEES.iter().copied().filter(|t| !xs.contains(t)).collect();

        // chapter 14's last picture: the rail with the trio's cards, the
        // paper plot with the sum on the parabola, the helper boards with
        // their curves and scales
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        let mut cards = Vec::with_capacity(shares.len());
        for &(t, y) in &shares {
            cards.push(card(&kit, stage, t, &y.to_string(), palette::share())?);
        }
        let setting = Setting::build(&kit, stage, &trio, &recipe, palette::trio())?;
        rail.reveal(stage, 0.6)?;
        let mut anims = swaps(stage, &[], &cards, 0.6)?;
        anims.extend(recolorings(stage, &rail.tint(&xs, palette::trio()), 0.6)?);
        anims.extend(recolorings(stage, &rail.tint(&others, palette::ghost()), 0.6)?);
        anims.extend(swaps(stage, &[], &setting.all(), 0.6)?);
        stage.play_prepared(anims)?;

        // 15_1: each helper at position 0; trustee 1's weight
        let spoken = say(&kit, stage, "15_1")?;
        spoken.wait_for(stage, "read each helper at position zero")?;
        let column = column_outline(stage, 0, palette::secret())?;
        fade_in_all(stage, &[column], 0.5)?;
        let first = read_weight(&kit, stage, &spoken, &xs, 0, &zeros[0], work.weights[0])?;
        spoken.wait_end(stage)?;

        // 15_2: trustees 2's and 3's; the weights use only the positions
        let spoken = say(&kit, stage, "15_2")?;
        let mut read = vec![first];
        for (i, working) in zeros.iter().enumerate().skip(1) {
            read.push(read_weight(&kit, stage, &spoken, &xs, i, working, work.weights[i])?);
        }
        spoken.wait_for(stage, "use only the positions")?;
        let mut halos = Vec::with_capacity(2 * xs.len());
        for &x in &xs {
            halos.push(text_halo(stage, setting.heads[x as usize], palette::trio())?);
        }
        for r in &read {
            halos.push(text_halo(stage, r.tag, palette::trio())?);
        }
        glow(stage, &halos, 1.2)?;
        spoken.wait_end(stage)?;

        // 15_3: each scale takes its weight; on paper, the secret
        let spoken = say(&kit, stage, "15_3")?;
        let mut gone: Vec<Mob> = read.iter().flat_map(|r| r.lines).collect();
        gone.push(column);
        let anims = leaving(stage, &gone, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        let mut pieces = Vec::with_capacity(xs.len());
        for (b, phrase) in [
            "three times nineteen",
            "minus three times forty-seven",
            "plus one times eighty-nine",
        ]
        .into_iter()
        .enumerate()
        {
            spoken.wait_for(stage, phrase)?;
            // the weight comes over from the board and the tag makes room
            let piece = kit.text_left(
                stage,
                &weight_piece(work.weights[b]),
                TAG_SIZE,
                palette::trio(),
                v(TAG_X, HELPER_Y[b]),
            )?;
            let (to, w) = (stage.get_center(piece), stage.get_width(piece));
            let from = stage.get_center(read[b].label);
            stage.move_to(piece, from, ORIGIN);
            fade_in_all(stage, &[piece], 0.3)?;
            let tag = setting.tags[b];
            let at = stage.get_center(tag);
            let anims = vec![
                moving(stage, piece, to, 0.7)?,
                moving(stage, tag, v(at[0] + w + PIECE_GAP, at[1]), 0.7)?,
            ];
            stage.play_prepared(anims)?;
            pieces.push(piece);
        }
        let mut terms = Vec::with_capacity(xs.len());
        for (b, phrase) in [
            "That's fifty-seven",
            "minus a hundred and forty-one",
            "plus eighty-nine",
        ]
        .into_iter()
        .enumerate()
        {
            spoken.wait_for(stage, phrase)?;
            let term = text_after(
                &kit,
                stage,
                setting.tags[b],
                &term_text(work.paper_terms[b]),
                TAG_SIZE,
                palette::trio(),
            )?;
            fade_in_all(stage, &[term], 0.4)?;
            terms.push(term);
        }
        let paper = kit.text_left(stage, &paper_words[0], WORK_SIZE, Palette::ink(), v(WORK_X, WORK_Y[0]))?;
        fade_in_all(stage, &[paper], 0.5)?;
        spoken.wait_for(stage, "which is five")?;
        let paper_total = text_after(&kit, stage, paper, &paper_words[1], WORK_SIZE, palette::secret())?;
        let meets = paper_at(0.0, f64::from(recipe.secret));
        let secret_dot = disc(stage, palette::secret(), meets, POINT_R);
        let unknown_at = stage.get_center(setting.unknown);
        let five = kit.text(
            stage,
            &recipe.secret.to_string(),
            UNKNOWN_SIZE,
            palette::secret(),
            unknown_at,
        )?;
        let anims = swaps(stage, &[setting.unknown], &[paper_total, secret_dot, five], 0.5)?;
        stage.play_prepared(anims)?;
        swell_all(stage, &[secret_dot], palette::secret(), 0.8)?;
        spoken.wait_end(stage)?;

        // 15_4: the trustees hold only the cards; will the weights work on
        // those?
        let spoken = say(&kit, stage, "15_4")?;
        spoken.wait_for(stage, "never see those heights")?;
        let mut plot = setting.plot.clone();
        plot.extend([setting.parabola, setting.sum, secret_dot, five]);
        plot.extend(setting.points.iter().flatten());
        plot.extend(&setting.tags);
        plot.extend(&pieces);
        plot.extend(&terms);
        let anims = leaving(stage, &plot, ORIGIN, 0.6)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "eight, three and one")?;
        let halos = frame_halos(stage, &xs, palette::trio())?;
        glow(stage, &halos, 1.0)?;
        spoken.wait_for(stage, "Will the same weights")?;
        let halos: Vec<Mob> = read
            .iter()
            .map(|r| text_halo(stage, r.tag, palette::trio()))
            .collect::<fmn::Result<_>>()?;
        glow(stage, &halos, 1.0)?;
        spoken.wait_end(stage)?;
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 15_5: yes: 16, which lands on 5; the needle runs sixteen marks and
        // the door opens
        let spoken = say(&kit, stage, "15_5")?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Three times eight, minus three times three, plus one times one")?;
        let on_cards = kit.text_left(stage, &cards_words[0], WORK_SIZE, Palette::ink(), v(WORK_X, WORK_Y[1]))?;
        fade_in_all(stage, &[on_cards], 0.5)?;
        spoken.wait_for(stage, "is sixteen")?;
        let cards_total = text_after(&kit, stage, on_cards, &cards_words[1], WORK_SIZE, Palette::ink())?;
        fade_in_all(stage, &[cards_total], 0.4)?;
        spoken.wait_for(stage, "sixteen lands on five")?;
        let landed = text_after(&kit, stage, cards_total, &cards_words[2], WORK_SIZE, palette::secret())?;
        fade_in_all(stage, &[landed], 0.4)?;
        spoken.wait_for(stage, "The dial turns to five")?;
        let vault = Vault::build(&kit, stage, dial)?;
        vault.show(stage, 0.5)?;
        stage.play(tween(vault.needle, work.total as f64, NEEDLE_RT)?)?;
        spoken.wait_for(stage, "the door opens")?;
        recolor_all(stage, &[(vault.door, palette::open())], 0.3)?;
        door_glow(stage, &vault, palette::open(), 0.6)?;
        spoken.wait_end(stage)?;
        pause(stage, DOOR_HOLD)?;

        // 15_6: each height is its card and whole elevens
        let spoken = say(&kit, stage, "15_6")?;
        let mut boards: Vec<Mob> = setting.boards.iter().flatten().copied().collect();
        boards.extend(&setting.heads);
        boards.extend(&setting.curves);
        boards.extend(setting.dots.iter().flatten().flatten());
        boards.extend(read.iter().flat_map(|r| [r.dot, r.label]));
        let anims = leaving(stage, &boards, ORIGIN, 0.6)?;
        stage.play_prepared(anims)?;
        for (j, phrase) in [
            "Nineteen is eight and one eleven",
            "forty-seven is three and four elevens",
            "eighty-nine is one and eight elevens",
        ]
        .into_iter()
        .enumerate()
        {
            spoken.wait_for(stage, phrase)?;
            let row = kit.text_left(
                stage,
                &split_row(work.heights[j], work.shares[j], work.elevens[j], MARKS),
                SPLIT_SIZE,
                Palette::ink(),
                v(SPLIT_X, SPLIT_Y[j]),
            )?;
            let halo = frame_halos(stage, &[xs[j]], palette::trio())?;
            let mut anims = swaps(stage, &[], &[row], 0.5)?;
            anims.extend(glowing(stage, &halo, 1.0)?);
            stage.play_prepared(anims)?;
        }
        spoken.wait_end(stage)?;

        // 15_7: whole weights keep whole elevens whole, so the two sums are
        // whole elevens apart; the weights on the dial
        let spoken = say(&kit, stage, "15_7")?;
        spoken.wait_for(stage, "Whole weights turn whole elevens into whole elevens")?;
        let elevens = kit.text_left(
            stage,
            &elevens_line(&work, MARKS),
            ELEVENS_SIZE,
            Palette::ink(),
            v(SPLIT_X, ELEVENS_Y),
        )?;
        fade_in_all(stage, &[elevens], 0.6)?;
        spoken.wait_for(stage, "can differ only by whole elevens")?;
        let both = [
            around(stage, &[paper, paper_total], palette::trio())?,
            around(stage, &[on_cards, cards_total, landed], palette::trio())?,
        ];
        glow(stage, &both, 1.2)?;
        spoken.wait_for(stage, "sixteen is five and one eleven")?;
        let turns = kit.text_left(
            stage,
            &turns_line(work.total, MARKS),
            WORK_SIZE,
            Palette::ink(),
            v(WORK_X, WORK_Y[2]),
        )?;
        fade_in_all(stage, &[turns], 0.5)?;
        spoken.wait_for(stage, "On the dial, they're three, eight and one")?;
        // a negative weight gives way to its mark: −3 lands on 8
        let mut old = Vec::new();
        let mut tags = Vec::with_capacity(read.len());
        for (i, r) in read.iter().enumerate() {
            let on_dial = i64::from(work.dial_weights[i]);
            if on_dial == work.weights[i] {
                tags.push(r.tag);
                continue;
            }
            let at = stage.get_center(r.tag);
            let tag = kit.text(stage, &weight_tag(on_dial), RAIL_WEIGHT_SIZE, palette::trio(), at)?;
            old.push(r.tag);
            tags.push(tag);
        }
        let new: Vec<Mob> = tags
            .iter()
            .copied()
            .filter(|t| !read.iter().any(|r| r.tag == *t))
            .collect();
        let anims = swaps(stage, &old, &new, 0.6)?;
        stage.play_prepared(anims)?;
        let halos: Vec<Mob> = tags
            .iter()
            .map(|&m| text_halo(stage, m, palette::trio()))
            .collect::<fmn::Result<_>>()?;
        glow(stage, &halos, 1.0)?;
        let exact = precisely(&kit, stage, &card_words)?;
        let anims = swaps(stage, &[], &[exact], 0.5)?;
        stage.play_prepared(anims)?;
        let read_at = read_by(stage, &card_words);
        spoken.wait_end(stage)?;
        wait_until(stage, read_at)?;
        pause(stage, HOLD_AT_END)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::draw::{DOOR_H, DOOR_W, DOOR_Y, PAUSE_NOTE, THINK_R, TRUSTEE_R};
    use super::super::helper_curves::{
        COLUMN_W, HEAD_SIZE, HEAD_Y, HELPER_SPAN, NAME_DY, NAME_GAP, NAME_SIZE, column_at, factors_at, helper_name,
        scale_tag,
    };
    use super::*;
    use crate::facts;
    use crate::model::{SECOND_TRIO, SECRET, csv};

    /// The production narration: registering chapter 15 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// Every phrase the scene waits for is in its line of the script, once,
    /// and they come in the order they are said.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        let mut phrases = vec![("15_1", "read each helper at position zero")];
        let c = weight_cues(0);
        phrases.extend([("15_1", c[0]), ("15_1", c[1]), ("15_1", "That's trustee one's weight")]);
        for i in 1..3 {
            let c = weight_cues(i);
            phrases.extend([("15_2", c[0]), ("15_2", c[1])]);
        }
        phrases.extend([
            ("15_2", "use only the positions"),
            ("15_3", "three times nineteen"),
            ("15_3", "minus three times forty-seven"),
            ("15_3", "plus one times eighty-nine"),
            ("15_3", "That's fifty-seven"),
            ("15_3", "minus a hundred and forty-one"),
            ("15_3", "plus eighty-nine"),
            ("15_3", "which is five"),
            ("15_4", "never see those heights"),
            ("15_4", "eight, three and one"),
            ("15_4", "Will the same weights"),
            ("15_5", "Three times eight, minus three times three, plus one times one"),
            ("15_5", "is sixteen"),
            ("15_5", "sixteen lands on five"),
            ("15_5", "The dial turns to five"),
            ("15_5", "the door opens"),
            ("15_6", "Nineteen is eight and one eleven"),
            ("15_6", "forty-seven is three and four elevens"),
            ("15_6", "eighty-nine is one and eight elevens"),
            ("15_7", "Whole weights turn whole elevens into whole elevens"),
            ("15_7", "can differ only by whole elevens"),
            ("15_7", "sixteen is five and one eleven"),
            ("15_7", "On the dial, they're three, eight and one"),
        ]);
        let mut last: Option<(&str, usize)> = None;
        for (id, phrase) in phrases {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id}: {phrase:?} in {text}");
            let at = text.find(phrase).unwrap();
            if let Some((prev_id, prev_at)) = last {
                assert!(
                    prev_id != id || prev_at < at,
                    "{id}: {phrase:?} is waited for out of order"
                );
            }
            last = Some((id, at));
        }
        // the question ends 15_4, the think time after it
        assert!(script.text("15_4").unwrap().ends_with("on what they hold?"));
    }

    /// What the scene writes is the facts': the helpers at 0 and the
    /// weights, both sums term by term, the cards' landing, each height's
    /// split, the weighted elevens and the weights on the dial.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}")).to_owned();
        let work = Working::example().unwrap();
        let xs = OPENING_TRIO;
        assert_eq!(get("S-F9.weights"), csv(&work.weights));
        assert_eq!(get("S-F9.weights_on_dial"), csv(&work.dial_weights));
        assert_eq!(get("S-F9.paper.heights"), csv(&work.heights));
        assert_eq!(get("S-F1.opening_shares"), csv(&work.shares));
        assert_eq!(get("S-F9.paper.terms"), csv(&work.paper_terms));
        assert_eq!(get("S-F9.paper.sum"), work.paper_sum.to_string());
        assert_eq!(get("S-F9.share_terms"), csv(&work.share_terms));
        assert_eq!(get("S-F9.sum_unwrapped"), work.total.to_string());
        assert_eq!(get("S-F9.paper.elevens"), csv(&work.elevens));
        assert_eq!(get("S-F9.weighted_elevens"), work.weighted_elevens.to_string());
        let dial = Dial::vault();
        assert_eq!(
            get("S-F9.sum_lands"),
            csv(&[work.total, i64::from(dial.wrap(work.total))])
        );
        assert_eq!(get("S-F9.secret"), dial.wrap(work.total).to_string());
        let m = i64::from(MARKS);
        assert_eq!(
            get("S-F9.sum_split"),
            csv(&[work.total, work.total.rem_euclid(m), work.total.div_euclid(m)])
        );
        let interleave =
            |values: &[i64]| -> Vec<i64> { work.weights.iter().zip(values).flat_map(|(&w, &v)| [w, v]).collect() };
        assert_eq!(get("S-F9.paper.weighted"), csv(&interleave(&work.heights)));
        assert_eq!(get("S-F9.cards.weighted"), csv(&interleave(&work.shares)));
        for i in 0..xs.len() {
            let n = i + 1;
            assert_eq!(
                get(&format!("S-F9.helper.{n}.factors_at_zero")),
                csv(&factors_at(&xs, i, 0))
            );
            assert_eq!(
                get(&format!("S-F9.helper.{n}.at_zero")),
                model::helper_at(&xs, i, 0).to_string()
            );
            assert_eq!(get(&format!("S-F9.helper.{n}.weight")), work.weights[i].to_string());
            assert_eq!(
                get(&format!("S-F9.split.{n}")),
                csv(&[work.heights[i], work.shares[i], work.elevens[i]])
            );
        }
        // the written lines, as the scene builds them
        assert_eq!(
            zero_working(&xs, 0).unwrap(),
            [
                "trustee 1 at 0: (0 \u{2212} 2) \u{d7} (0 \u{2212} 3)".to_owned(),
                "= 6, \u{f7} 2: 3".to_owned()
            ]
        );
        assert_eq!(zero_working(&xs, 1).unwrap()[1], "= 3, \u{f7} (\u{2212}1): \u{2212}3");
        assert_eq!(zero_working(&xs, 2).unwrap()[1], "= 2, \u{f7} 2: 1");
        assert_eq!(
            paper_line(&work),
            ["on paper: 57 \u{2212} 141 + 89".to_owned(), "= 5".to_owned()]
        );
        assert_eq!(
            cards_line(dial, &work),
            [
                "on the cards: 3 \u{d7} 8 \u{2212} 3 \u{d7} 3 + 1 \u{d7} 1".to_owned(),
                "= 24 \u{2212} 9 + 1 = 16".to_owned(),
                "\u{2192} 5".to_owned()
            ]
        );
        assert_eq!(split_row(19, 8, 1, MARKS), "19 = 8 + 1 \u{d7} 11");
        assert_eq!(
            elevens_line(&work, MARKS),
            "3 \u{d7} (1 \u{d7} 11) \u{2212} 3 \u{d7} (4 \u{d7} 11) + 1 \u{d7} (8 \u{d7} 11) = \u{2212}1 \u{d7} 11"
        );
        assert_eq!(turns_line(work.total, MARKS), "16 = 5 + 1 \u{d7} 11");
        assert_eq!(weight_tag(-3), "weight \u{2212}3");
        assert_eq!(term_text(-141), "= \u{2212}141");
    }

    /// The door counted by hand: the weights as fractions of whole
    /// numbers, the heights from the recipe's definition, the cards as the
    /// heights less whole elevens, and both sums; and the controls fail as
    /// they should: a wrong weight misses on paper and on the dial, the
    /// dial's weights on the heights land on the secret but are not it, and
    /// the trio 2, 4, 5's weights are not whole.
    #[test]
    fn the_door_is_counted_by_hand() {
        // each weight: the product of (0 − other) over the product of
        // (own − other), which divides exactly
        let xs = [1i64, 2, 3];
        let mut weights = Vec::with_capacity(xs.len());
        for (i, &own) in xs.iter().enumerate() {
            let (mut top, mut bottom) = (1, 1);
            for (j, &other) in xs.iter().enumerate() {
                if j != i {
                    top *= -other;
                    bottom *= own - other;
                }
            }
            assert_eq!(top % bottom, 0, "trustee {own}");
            weights.push(top / bottom);
        }
        assert_eq!(weights, [3, -3, 1]);
        let heights: Vec<i64> = xs.iter().map(|x| 5 + 7 * x + 7 * x * x).collect();
        assert_eq!(heights, [19, 47, 89]);
        let cards: Vec<i64> = heights.iter().map(|h| h % 11).collect();
        assert_eq!(cards, [8, 3, 1]);
        let dot = |w: &[i64], v: &[i64]| -> i64 { w.iter().zip(v).map(|(a, b)| a * b).sum() };
        let (on_paper, on_cards) = (dot(&weights, &heights), dot(&weights, &cards));
        assert_eq!((on_paper, on_cards), (5, 16));
        assert_eq!((on_cards % 11, on_cards - on_paper), (5, 11));
        let work = Working::example().unwrap();
        assert_eq!(work.weights, weights);
        assert_eq!(work.paper_sum, i64::from(SECRET));
        assert_eq!(work.total, on_cards);
        // controls: a wrong weight gives 94 on paper and 17, landing on 6
        let wrong = [3, -3, 2];
        assert_eq!(dot(&wrong, &heights), 94);
        assert_eq!(dot(&wrong, &cards) % 11, 6);
        // the dial's weights on the heights: 522, which lands on 5 but is
        // not 5, so on paper only the ordinary weights give the secret
        let on_dial: Vec<i64> = work.dial_weights.iter().map(|&w| i64::from(w)).collect();
        let tall = dot(&on_dial, &heights);
        assert_eq!((tall, tall % 11), (522, 5));
        // a trio with fractional weights is refused
        let dial = Dial::vault();
        let recipe = Recipe::example();
        let second: Vec<i64> = SECOND_TRIO.iter().map(|&x| recipe.height(x)).collect();
        assert!(Working::new(dial, &dealt(dial, &SECOND_TRIO), &second, SECRET).is_err());
        // a card that is not its height less whole elevens is refused
        let mut nudged = dealt(dial, &OPENING_TRIO);
        nudged[1].1 += 1;
        assert!(Working::new(dial, &nudged, &heights, SECRET).is_err());
    }

    /// The card stays two lines at every wrap the lower third narrows to,
    /// and never claims more than the counting shows.
    #[test]
    fn the_card_is_two_lines() {
        let text = card_text(&Working::example().unwrap().dial_weights);
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
        assert!(text.contains("3, 8 and 1"), "{text}");
        for word in ["always", "any", "secure", "safe"] {
            assert!(!text.contains(word), "{word}");
        }
    }

    /// The pieces keep apart, inside the frame: the weights by the boards'
    /// side clear of the names, the heads and the gold column, the terms
    /// right of the boards, the weights under the rail's discs over the
    /// lower third, the sums in the working, the think ring where the plot
    /// was, and the splits and the weighted elevens right of the vault.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let edge = FRAME_X_RADIUS - 0.25;
        let top = FRAME_Y_RADIUS - 0.25;
        let work = Working::example().unwrap();
        // the weights by the side: right of the names, left of the column
        let (col, _) = column_at(0);
        let names_right = HELPER_X0 - NAME_GAP;
        for (b, &w) in work.weights.iter().enumerate() {
            let at = weight_label_at(b, w);
            let half = text_w(&signed(w), WEIGHT_SIZE) / 2.0;
            assert!(at[0] + half < col[0] - COLUMN_W / 2.0 - 0.03, "{b}");
            assert!(at[0] + half < HELPER_X0 - DOT_R - 0.03, "{b}");
            // a name sits just under its board's 0 line; the weight is off it
            let name_top = HELPER_Y[b] - NAME_DY + text_h(NAME_SIZE) / 2.0;
            let name_bottom = HELPER_Y[b] - NAME_DY - text_h(NAME_SIZE) / 2.0;
            let label_bottom = at[1] - text_h(WEIGHT_SIZE) / 2.0;
            let label_top = at[1] + text_h(WEIGHT_SIZE) / 2.0;
            assert!(
                at[0] - half > names_right + 0.03 || label_bottom > name_top + 0.1 || label_top < name_bottom - 0.1,
                "{b}: the weight {w} on the name"
            );
            assert!(label_top < HEAD_Y - text_h(HEAD_SIZE) / 2.0 - 0.05, "{b}");
            assert!((w as f64).abs() <= HELPER_SPAN, "{b}");
        }
        // the terms right of the boards, inside the frame
        for b in 0..3 {
            let row = format!(
                "{} {} {}",
                weight_piece(work.weights[b]),
                scale_tag(work.heights[b]),
                term_text(work.paper_terms[b])
            );
            assert!(TAG_X + text_w(&row, TAG_SIZE) + 2.0 * PIECE_GAP < edge, "{row}");
        }
        // the rail's weights: under the discs, apart, over the lower third
        // (a band at most 2.0 high from the foot, two lines 1.02)
        let band_top = -FRAME_Y_RADIUS + 0.2 + 1.02;
        let h = text_h(RAIL_WEIGHT_SIZE);
        assert!(RAIL_WEIGHT_Y + h / 2.0 < super::super::draw::RAIL_Y - TRUSTEE_R - 0.03);
        assert!(RAIL_WEIGHT_Y - h / 2.0 > band_top + 0.03);
        let widest = text_w(&weight_tag(-3), RAIL_WEIGHT_SIZE);
        assert!(widest < super::super::draw::RAIL_DX - 0.1);
        assert!(rail_weight_at(1)[0] - widest / 2.0 > -edge);
        // the working: each line inside the frame, the third over the band
        let dial = Dial::vault();
        let [c0, c1, c2] = cards_line(dial, &work);
        let cards_w = text_w(&format!("{c0} {c1} {c2}"), WORK_SIZE) + 2.0 * PIECE_GAP;
        assert!(WORK_X + cards_w < edge, "{}", WORK_X + cards_w);
        let [p0, p1] = paper_line(&work);
        assert!(WORK_X + text_w(&format!("{p0} {p1}"), WORK_SIZE) < edge);
        for i in 0..3 {
            let [a, b] = zero_working(&OPENING_TRIO, i).unwrap();
            assert!(WORK_X + text_w(&format!("{a} {b}"), WORK_SIZE) < edge, "{a} {b}");
        }
        assert!(WORK_Y[2] - text_h(WORK_SIZE) / 2.0 > band_top + 0.1);
        // the think ring where the plot was: left of the boards' names
        let names_left = names_right - text_w(&helper_name(3), NAME_SIZE);
        assert!(THINK_X + THINK_R < names_left - 0.3);
        assert!(THINK_X - text_w(PAUSE_NOTE, 18.0) / 2.0 > -edge);
        assert!(THINK_Y + THINK_R < top);
        assert!(THINK_Y - THINK_R - 0.3 - text_h(18.0) / 2.0 > WORK_Y[0] + text_h(WORK_SIZE) / 2.0 + 0.3);
        // the splits and the weighted elevens: right of the vault, inside
        // the frame, over the working
        let door_right = DOOR_X + DOOR_W / 2.0;
        assert!(SPLIT_X > door_right + 0.5);
        let line = elevens_line(&work, MARKS);
        assert!(
            SPLIT_X + text_w(&line, ELEVENS_SIZE) < edge,
            "{}",
            SPLIT_X + text_w(&line, ELEVENS_SIZE)
        );
        let row = split_row(89, 1, 8, MARKS);
        assert!(SPLIT_X + text_w(&row, SPLIT_SIZE) < edge);
        assert!(SPLIT_Y[0] + text_h(SPLIT_SIZE) / 2.0 < top);
        for j in 0..2 {
            assert!(SPLIT_Y[j] - SPLIT_Y[j + 1] > text_h(SPLIT_SIZE) + 0.2);
        }
        assert!(SPLIT_Y[2] - text_h(SPLIT_SIZE) / 2.0 > ELEVENS_Y + text_h(ELEVENS_SIZE) / 2.0 + 0.2);
        assert!(ELEVENS_Y - text_h(ELEVENS_SIZE) / 2.0 > WORK_Y[0] + text_h(WORK_SIZE) / 2.0 + 0.5);
        // the vault's door stays left of the working
        assert!(door_right < WORK_X - 0.3);
        const {
            assert!(DOOR_Y - DOOR_H / 2.0 > super::super::draw::FRAME_Y + super::super::draw::FRAME_H / 2.0);
        }
    }

    /// The needle runs the cards' whole sum: sixteen marks are once round
    /// and five more.
    #[test]
    fn the_needle_runs_the_sum() {
        let work = Working::example().unwrap();
        let m = i64::from(MARKS);
        assert_eq!((work.total.div_euclid(m), work.total.rem_euclid(m)), (1, 5));
        assert_eq!(Dial::vault().wrap(work.total), SECRET);
    }
}
