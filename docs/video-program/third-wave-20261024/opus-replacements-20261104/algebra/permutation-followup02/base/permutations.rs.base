//! Chapter 11: permutations as numbered cards moved between fixed slots.
//! Every map, arrangement, chip and product on screen is computed by the pure
//! helpers below; the tests pin them and `oracle.py` recomputes them another
//! way. Draft scene; native and visual review remain open.
use crate::d4;
use crate::d4_group::{self as d4g, ALL, R, S};
use crate::tile_model::CORNERS;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Facts, Kit, Palette, Timed, hex, ink_style, play, stroke_style, v};
use fmn::prelude::*;
use std::f64::consts::{FRAC_PI_2, PI};

pub struct Permutations {
    pub kit: Kit,
}

// ---- the arithmetic on screen -----------------------------------------------

// Images of 1,2,3, in zero-based notation. These maps also drive the paths.
const SWAP_12: [usize; 3] = [1, 0, 2];
const CYCLE_123: [usize; 3] = [1, 2, 0];
// The same idea on the board whose rows are labelled 1, 3, 4.
const SWAP_13_ON_134: [usize; 3] = [1, 0, 2];
const SWAP_14_ON_134: [usize; 3] = [2, 1, 0];
const CYCLE_134_ON_134: [usize; 3] = [1, 2, 0];
const INVERSE_134_ON_134: [usize; 3] = [2, 0, 1];

// 11_1: a legal map, then one with a collision and a missing destination.
const LEGAL_MAP: [usize; 3] = [1, 2, 0];
const BAD_MAP: [usize; 3] = [0, 0, 2];
const BAD_DOUBLED: usize = 0;
const BAD_MISSING: usize = 1;

// 11_7 and 11_8: the numbers placed on four labels, and the swap routes.
const WITNESS: [i32; 4] = [1, 2, 4, 8];
const PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
const ONE_SWAP: [(usize, usize); 1] = [(0, 2)];
const THREE_SWAPS: [(usize, usize); 3] = [(0, 1), (1, 2), (0, 1)];
// How each chip behaves when the numbers on labels 1 and 3 swap.
const REVERSES_IN_PLACE: usize = 1;
const TRADE_AND_REVERSE: [usize; 2] = [0, 3];
const TRADE_ONLY: [usize; 2] = [2, 5];
const UNTOUCHED: usize = 4;

// 11_9: the square's corners, in the order of `tile_model::CORNERS`.
const CORNER_POINTS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

fn route(origin: usize, first: [usize; 3], second: [usize; 3]) -> [usize; 3] {
    [origin, first[origin], second[first[origin]]]
}

/// One-based cycles, as written on screen, to zero-based images: the card in
/// slot `i` moves to slot `images[i]`.
fn cycle_images<const N: usize>(cycles: &[&[usize]]) -> [usize; N] {
    let mut images: [usize; N] = std::array::from_fn(|slot| slot);
    for cycle in cycles {
        for (index, &label) in cycle.iter().enumerate() {
            images[label - 1] = cycle[(index + 1) % cycle.len()] - 1;
        }
    }
    images
}

/// `first`, then `second`.
fn then<const N: usize>(first: [usize; N], second: [usize; N]) -> [usize; N] {
    first.map(|slot| second[slot])
}

/// Which card (one-based) sits in each slot once card `i` has reached `at[i]`.
fn cards_by_slot<const N: usize>(at: [usize; N]) -> [usize; N] {
    let mut cards = [0; N];
    for (card, &slot) in at.iter().enumerate() {
        cards[slot] = card + 1;
    }
    cards
}

/// The swaps that build `p`: labels minus cycles, fixed labels counting as cycles.
fn swaps_needed<const N: usize>(p: [usize; N]) -> usize {
    let mut seen = [false; N];
    let mut cycles = 0;
    for start in 0..N {
        let mut at = start;
        if !seen[at] {
            cycles += 1;
        }
        while !seen[at] {
            seen[at] = true;
            at = p[at];
        }
    }
    N - cycles
}

fn parity_word(swaps: usize) -> &'static str {
    if swaps.is_multiple_of(2) { "even" } else { "odd" }
}

/// `p` in the on-screen cycle notation, as TeX; fixed labels are omitted.
fn cycle_tex<const N: usize>(p: [usize; N]) -> String {
    let mut seen = [false; N];
    let mut written = String::new();
    for start in 0..N {
        let mut at = start;
        if p[at] == at {
            continue;
        }
        let mut labels = Vec::new();
        while !seen[at] {
            seen[at] = true;
            labels.push((at + 1).to_string());
            at = p[at];
        }
        if !labels.is_empty() {
            written.push_str(&format!("({})", labels.join(r"\;")));
        }
    }
    written
}

/// How many arrows land in each slot.
fn arrivals(map: [usize; 3]) -> [usize; 3] {
    let mut counts = [0; 3];
    for slot in map {
        counts[slot] += 1;
    }
    counts
}

fn differences(x: [i32; 4]) -> [i32; 6] {
    PAIRS.map(|(i, j)| x[j] - x[i])
}

fn after_swaps(mut x: [i32; 4], swaps: &[(usize, usize)]) -> [i32; 4] {
    for &(a, b) in swaps {
        x.swap(a, b);
    }
    x
}

fn delta(x: [i32; 4]) -> i32 {
    differences(x).iter().product()
}

/// Where each chip's size turns up after the swap. The witness's six sizes
/// are all different, so this is a real tracking of each factor.
fn chip_destinations(before: [i32; 6], after: [i32; 6]) -> [usize; 6] {
    before.map(|old| {
        after
            .iter()
            .position(|new| new.abs() == old.abs())
            .expect("the witness has six different difference sizes")
    })
}

/// The arrangements along a swap route, starting from the witness.
fn route_arrangements(swaps: &[(usize, usize)]) -> Vec<[i32; 4]> {
    let mut x = WITNESS;
    let mut arrangements = vec![x];
    for &(a, b) in swaps {
        x.swap(a, b);
        arrangements.push(x);
    }
    arrangements
}

/// The product written out with its factors: `\Delta=1\cdot3\cdots=+1008`.
fn tally_tex(x: [i32; 4]) -> String {
    let factors = differences(x)
        .map(|n| if n < 0 { format!("({n})") } else { n.to_string() })
        .join(r"\cdot");
    format!("\\Delta={factors}={:+}", delta(x))
}

/// A square symmetry as a corner map, through the film's own model in `d4.rs`.
fn corner_images(turns: u8, flip: bool) -> [usize; 4] {
    CORNER_POINTS.map(|point| {
        let image = d4::apply(point, turns, flip);
        CORNER_POINTS
            .iter()
            .position(|&corner| corner == image)
            .expect("a square symmetry permutes the corners")
    })
}

fn symmetries() -> Vec<[usize; 4]> {
    ALL.iter().map(|g| corner_images(g.turns, g.flip)).collect()
}

/// Every arrangement of four cards in reading order: `p[slot]` is a card.
fn arrangements() -> Vec<[usize; 4]> {
    let mut found = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let p = [a, b, c, d];
                    if (0..4).all(|card| p.contains(&card)) {
                        found.push(p);
                    }
                }
            }
        }
    }
    found
}

/// The label pairs (zero-based) that are square edges but land on a diagonal.
fn broken_edges(at: [usize; 4]) -> Vec<(usize, usize)> {
    (0..4)
        .map(|i| (i, (i + 1) % 4))
        .filter(|&(i, j)| at[i].abs_diff(at[j]).is_multiple_of(2))
        .collect()
}

fn one_based(images: &[usize]) -> String {
    crate::listed(images.iter().map(|image| image + 1))
}

fn arrangement_text(cards: &[usize]) -> String {
    cards.iter().map(|card| card.to_string()).collect::<Vec<_>>().join(" ")
}

/// A four-digit cell of the 24 grid: the card in each corner, in corner order.
fn cell_text(p: [usize; 4]) -> String {
    p.map(|card| (card + 1).to_string()).concat()
}

pub(crate) fn add_facts(facts: &mut Facts) -> Result<(), String> {
    facts.add("permutations.bad_map_arrivals", crate::listed(arrivals(BAD_MAP)))?;
    let all = arrangements();
    let symmetric = symmetries();
    facts.add("permutations.corner_count", all.len())?;
    let kept: Vec<String> = all
        .iter()
        .filter(|p| symmetric.contains(p))
        .map(|&p| cell_text(p))
        .collect();
    facts.add("permutations.edge_keeping", kept.len())?;
    facts.add("permutations.edge_keeping_arrangements", kept.join(","))?;
    let trade = cycle_images::<4>(&[&[1, 2]]);
    let broken: Vec<String> = broken_edges(trade)
        .iter()
        .map(|&(i, j)| format!("{}-{}", i + 1, j + 1))
        .collect();
    facts.add("permutations.trade_broken_edges", broken.join(","))?;

    let swapped = after_swaps(WITNESS, &ONE_SWAP);
    let (before, after) = (differences(WITNESS), differences(swapped));
    facts.add("permutations.witness", crate::listed(WITNESS))?;
    facts.add("permutations.witness_swapped", crate::listed(swapped))?;
    facts.add("permutations.chips_start", crate::listed(before))?;
    facts.add("permutations.chips_swapped", crate::listed(after))?;
    facts.add(
        "permutations.chip_destinations",
        one_based(&chip_destinations(before, after)),
    )?;
    facts.add("permutations.delta_start", delta(WITNESS))?;
    facts.add("permutations.delta_swapped", delta(swapped))?;
    let route = route_arrangements(&THREE_SWAPS);
    let steps: Vec<String> = route[1..].iter().map(|&x| crate::listed(x)).collect();
    facts.add("permutations.route_three_arrangements", steps.join(";"))?;
    facts.add(
        "permutations.route_three_deltas",
        crate::listed(route.iter().map(|&x| delta(x))),
    )?;

    let (r, s) = (corner_images(R.turns, R.flip), corner_images(S.turns, S.flip));
    facts.add("permutations.quarter_images", one_based(&r))?;
    facts.add("permutations.flip_images", one_based(&s))?;
    facts.add("permutations.half_images", one_based(&then(r, r)))?;
    facts.add("permutations.rs_images", one_based(&then(s, r)))?;
    facts.add("permutations.sr_images", one_based(&then(r, s)))?;
    facts.add("permutations.quarter_swaps", swaps_needed(r))?;
    facts.add("permutations.half_swaps", swaps_needed(then(r, r)))?;

    facts.add(
        "permutations.inverse_images",
        one_based(&cycle_images::<4>(&[&[1, 4, 3]])),
    )?;
    let tri = cycle_images::<6>(&[&[1, 3, 5]]);
    let swap = cycle_images::<6>(&[&[2, 6]]);
    facts.add(
        "permutations.disjoint_final",
        crate::listed(cards_by_slot(then(tri, swap))),
    )?;
    Ok(())
}

// ---- timing inside a recorded sentence --------------------------------------

const SCRIPT: &str = include_str!("../script.tsv");

/// A moment the picture waits for: cue, sentence, the words that begin it.
type Anchor = (&'static str, usize, &'static str);

const NO_COLLISIONS: Anchor = ("11_1", 0, "with no collisions");
const NO_MISSING: Anchor = ("11_1", 0, "no missing destinations");
const THEY_COMMUTE: Anchor = ("11_4", 2, "they commute");
const TWO_INPUTS: Anchor = ("11_4", 4, "because that would");
const ALL_DIFFERENCES: Anchor = ("11_7", 2, "multiply all the differences");
const THE_PAIRS: Anchor = ("11_7", 2, "for pairs with");
const BOTH_REVERSE: Anchor = ("11_11", 2, "sometimes both");
const SAME_PRODUCT: Anchor = ("11_11", 2, "so their product");
const NOT_EVERY: Anchor = ("11_9", 0, "but not every");
const OUR_EIGHT: Anchor = ("11_9", 1, "and our eight");
const AS_SUBGROUP: Anchor = ("11_9", 1, "as a subgroup");
const HALF_TURN: Anchor = ("11_9", 2, "a half-turn");
const RS_SWAP: Anchor = ("11_9", 3, "r s is the swap");
const SR_SWAP: Anchor = ("11_9", 3, "and s r is the swap");
const THE_POSITIONS: Anchor = ("11_10", 0, "the positions of cards");

#[cfg(test)]
const ANCHORS: [Anchor; 15] = [
    NO_COLLISIONS,
    NO_MISSING,
    THEY_COMMUTE,
    TWO_INPUTS,
    ALL_DIFFERENCES,
    THE_PAIRS,
    BOTH_REVERSE,
    SAME_PRODUCT,
    NOT_EVERY,
    OUR_EIGHT,
    AS_SUBGROUP,
    HALF_TURN,
    RS_SWAP,
    SR_SWAP,
    THE_POSITIONS,
];

/// The recorded text of `cue`.
fn line_text(cue: &str) -> Option<&'static str> {
    SCRIPT.lines().find_map(|l| l.strip_prefix(cue)?.strip_prefix('\t'))
}

/// `text[from..to]` without its surrounding whitespace, as a byte range.
fn trimmed(text: &str, from: usize, to: usize) -> Option<(usize, usize)> {
    let s = &text[from..to];
    let start = from + (s.len() - s.trim_start().len());
    let end = from + s.trim_end().len();
    (start < end).then_some((start, end))
}

/// The byte ranges of a line's sentences, split exactly as the kit splits
/// them (after `.`, `?` or `!` followed by a space or the end).
fn sentence_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0;
    let b = text.as_bytes();
    for (i, &c) in b.iter().enumerate() {
        if matches!(c, b'.' | b'?' | b'!') && (i + 1 == b.len() || b[i + 1] == b' ') {
            out.extend(trimmed(text, from, i + 1));
            from = i + 1;
        }
    }
    out.extend(trimmed(text, from, text.len()));
    out
}

/// When the anchor's words begin (scene seconds): the narration's phrase mark
/// at that byte when one exists, otherwise the phrase's share of the
/// sentence's characters (the house rule, as in `axioms.rs`).
fn phrase_time(kit: &Kit, anchor: Anchor) -> Option<f64> {
    let (cue, j, phrase) = anchor;
    let narrator = kit.narrator.as_ref()?;
    let text = line_text(cue)?;
    let &(from, to) = sentence_ranges(text).get(j)?;
    let byte = from + text[from..to].find(phrase)?;
    let said = narrator.sentence_at(cue, 0)? - narrator.sentence_starts(cue).first().copied()?;
    if let Some(&seconds) = narrator.phrase_marks(cue).get(&byte) {
        return Some(said + seconds);
    }
    let start = narrator.sentence_at(cue, j)?;
    let end = match narrator.sentence_at(cue, j + 1) {
        Some(next) => next,
        None => said + narrator.length(cue)?,
    };
    let share = text[from..byte].chars().count() as f64 / text[from..to].chars().count() as f64;
    Some(start + (end - start) * share)
}

/// Wait for the anchor's sentence, then its words. Without a narrator only
/// the sentence wait applies; nothing waits once the moment has passed.
fn until_phrase(stage: &mut Stage<'_>, kit: &Kit, anchor: Anchor) -> fmn::Result<()> {
    kit.until_sentence(stage, anchor.0, anchor.1)?;
    let Some(at) = phrase_time(kit, anchor) else {
        return Ok(());
    };
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}

// ---- shared pieces of picture -----------------------------------------------

/// Where the four cards of 11_1, 11_2 and the start of 11_4 sit.
const TRAIL: [(f64, f64); 4] = [(-3.2, 1.3), (4.7, -1.1), (3.2, 1.3), (-3.2, -1.1)];
/// A moving card's bend: left-to-right moves pass above their row.
const CARD_ARC: f64 = -0.9;
/// Two cards trading places pass on opposite sides of each other.
const SWAP_ARC: f64 = -1.0;

fn fade(stage: &mut Stage<'_>, items: &[Mob], on: bool) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &item in items {
        let a = if on {
            fade_in(stage.arena_mut(), item, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), item, ORIGIN, 1.0)?
        };
        anims.push(stage.prepare(a.rt(0.4))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Fade `old` out and `new` in over the same 0.3 s.
fn exchange(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob]) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &item in old {
        let a = fade_out(stage.arena_mut(), item, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.3))?);
    }
    for &item in new {
        let a = fade_in(stage.arena_mut(), item, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.3))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Move several mobjects at once along arcs of `arc` radians (0 is straight;
/// negative bends a left-to-right move upward).
fn travel(stage: &mut Stage<'_>, moves: &[(Mob, (f64, f64))], arc: f64, run_time: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &(mob, to) in moves {
        let a = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(run_time),
                path_arc: Some(arc),
                ..AnimateArgs::default()
            })?
            .move_to(v(to.0, to.1), ORIGIN)?;
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn pulse(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), mob, 1.25, None)?.rt(0.6);
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn recolor(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb, run_time: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(run_time),
                ..AnimateArgs::default()
            })?
            .set_color(color)?;
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn morph(stage: &mut Stage<'_>, from: Mob, to: Mob, run_time: f64) -> fmn::Result<()> {
    play!(stage; fmn::animation::replacement_transform(from, to).rt(run_time));
    Ok(())
}

/// A labelled disk as one group above arrows and rings, so it moves as a unit.
fn token(stage: &mut Stage<'_>, k: &Kit, label: &str, color: Srgb, at: (f64, f64), radius: f64) -> fmn::Result<Mob> {
    let disk = stage.arena_mut().add(
        Circle::new()
            .radius(radius)
            .style(ink_style(color))
            .build()
            .moved_to(v(at.0, at.1)),
    );
    let text = k.text(stage, label, radius * 80.0, hex("#0E1116"), v(at.0, at.1))?;
    let group = d4g::group_of(stage, &[disk, text])?;
    stage.arena_mut().set_z_index(group, 3, true);
    Ok(group)
}

/// A card keeps its label's colour everywhere in the chapter.
fn card(stage: &mut Stage<'_>, k: &Kit, label: usize, at: (f64, f64), radius: f64) -> fmn::Result<Mob> {
    token(stage, k, &label.to_string(), d4g::corner_color(label - 1), at, radius)
}

fn ring(stage: &mut Stage<'_>, at: (f64, f64), radius: f64, color: Srgb, width: f64) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(radius)
            .style(Style::default().stroke(color, width, 1.0))
            .build()
            .moved_to(v(at.0, at.1)),
    )
}

/// A ring drawn above the cards: a tracer, or a mark on a slot that holds one.
fn halo(stage: &mut Stage<'_>, at: (f64, f64), radius: f64, color: Srgb, width: f64) -> Mob {
    let circle = ring(stage, at, radius, color, width);
    stage.arena_mut().set_z_index(circle, 4, true);
    circle
}

fn straight(stage: &mut Stage<'_>, from: (f64, f64), to: (f64, f64), buff: f64, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Arrow::new(v(from.0, from.1), v(to.0, to.1))
            .buff(buff)
            .thickness(3.0)
            .style(Style::default().fill(color, 0.9).stroke(color, 3.0, 0.9))
            .build()?,
    ))
}

fn curved(stage: &mut Stage<'_>, from: (f64, f64), to: (f64, f64), arc: f64, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Arrow::new(v(from.0, from.1), v(to.0, to.1))
            .path_arc(arc)
            .buff(0.08)
            .thickness(3.0)
            .style(Style::default().fill(color, 0.9).stroke(color, 3.0, 0.9))
            .build()?,
    ))
}

// ---- three-slot boards ------------------------------------------------------

fn lane(column: usize, row: usize) -> Vec3 {
    v(-4.2 + column as f64 * 4.2, 1.1 - row as f64 * 1.1)
}

fn lane_xy(column: usize, row: usize) -> (f64, f64) {
    let point = lane(column, row);
    (point[0], point[1])
}

fn route_board(stage: &mut Stage<'_>, k: &Kit, reverse: bool) -> fmn::Result<Vec<Mob>> {
    let labels = if reverse {
        ["Start", "After (1 2 3)", "After (1 2)"]
    } else {
        ["Start", "After (1 2)", "After (1 2 3)"]
    };
    named_route_board(stage, k, labels, [1, 2, 3])
}
fn named_route_board(stage: &mut Stage<'_>, k: &Kit, labels: [&str; 3], slots: [usize; 3]) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    for (column, label) in labels.iter().enumerate() {
        mobs.push(k.text(stage, label, 27.0, Palette::ink(), v(lane(column, 0)[0], 2.0))?);
        for (row, slot_number) in slots.iter().enumerate() {
            mobs.push(
                stage.arena_mut().add(
                    Circle::new()
                        .radius(0.31)
                        .style(Style::default().stroke(Palette::dim(), 2.0, 0.7))
                        .build()
                        .moved_to(lane(column, row)),
                ),
            );
            // Keep destination labels visible when a moving input card occupies the ring.
            let slot = lane(column, row);
            mobs.push(k.text(
                stage,
                &slot_number.to_string(),
                23.0,
                Palette::dim(),
                v(slot[0], slot[1] + 0.46),
            )?);
        }
    }
    fade(stage, &mobs, true)?;
    Ok(mobs)
}

fn trace_route(
    stage: &mut Stage<'_>,
    k: &Kit,
    origin: usize,
    first: [usize; 3],
    second: [usize; 3],
) -> fmn::Result<Vec<Mob>> {
    named_trace_route(stage, k, origin, first, second, [1, 2, 3], 1.0)
}
/// One card's route, column by column. `pace` scales every run time: 1.0 is
/// 2.9 s per card.
fn named_trace_route(
    stage: &mut Stage<'_>,
    k: &Kit,
    origin: usize,
    first: [usize; 3],
    second: [usize; 3],
    slots: [usize; 3],
    pace: f64,
) -> fmn::Result<Vec<Mob>> {
    let rows = route(origin, first, second);
    let color = d4g::corner_color(slots[origin] - 1);
    let disk = stage.arena_mut().add(
        Circle::new()
            .radius(0.25)
            .style(ink_style(color))
            .build()
            .moved_to(lane(0, origin)),
    );
    // The moving card keeps its input label. The fixed slot number is its destination.
    let label = k.text(stage, &slots[origin].to_string(), 22.0, hex("#0E1116"), lane(0, origin))?;
    // Paths are created later; explicit layers keep traveling numerals unobscured.
    stage.arena_mut().set_z_index(disk, 2, true);
    stage.arena_mut().set_z_index(label, 3, true);
    let mut mobs = vec![disk, label];
    d4g::show(stage, &mobs, 0.4 * pace)?;
    for column in 0..2 {
        let a = lane(column, rows[column]);
        let b = lane(column + 1, rows[column + 1]);
        let path = stage.arena_mut().add(
            Arrow::new(a, b)
                .buff(0.36)
                .thickness(2.0)
                .style(Style::default().fill(color, 0.65).stroke(color, 2.0, 0.65))
                .build()?,
        );
        d4g::show(stage, &[path], 0.4 * pace)?;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for mob in [disk, label] {
            let movement = mob
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.85 * pace),
                    ..AnimateArgs::default()
                })?
                .move_to(b, ORIGIN)?;
            moves.push(stage.prepare(movement)?);
        }
        stage.play_prepared(moves)?;
        mobs.push(path);
    }
    Ok(mobs)
}

/// Every card of a board moves at once, one column per operation: the whole
/// arrangement changes together, in about 2.2 s rather than three traces' 8.7 s.
fn all_routes(
    stage: &mut Stage<'_>,
    k: &Kit,
    first: [usize; 3],
    second: [usize; 3],
    slots: [usize; 3],
) -> fmn::Result<Vec<Mob>> {
    let mut cards = Vec::new();
    for (origin, &label) in slots.iter().enumerate() {
        cards.push(card(stage, k, label, lane_xy(0, origin), 0.25)?);
    }
    fade(stage, &cards, true)?;
    let mut mobs = cards.clone();
    for column in 0..2 {
        let mut paths = Vec::new();
        let mut moves = Vec::new();
        for (origin, &label) in slots.iter().enumerate() {
            let rows = route(origin, first, second);
            let to = lane_xy(column + 1, rows[column + 1]);
            paths.push(straight(
                stage,
                lane_xy(column, rows[column]),
                to,
                0.36,
                d4g::corner_color(label - 1),
            )?);
            moves.push((cards[origin], to));
        }
        fade(stage, &paths, true)?;
        d4g::glide_all(stage, &moves, 0.5)?;
        mobs.extend(paths);
    }
    Ok(mobs)
}

// A tracer follows fixed destination labels; a closed trail spells one cycle.
// Three links take about 5.9 s, so "We write the cycle" is not left waiting.
fn cycle_trace(stage: &mut Stage<'_>, k: &Kit, order: &[usize], positions: &[(f64, f64); 4]) -> fmn::Result<Vec<Mob>> {
    let (x, y) = positions[order[0]];
    let ring = stage.arena_mut().add(
        Circle::new()
            .radius(0.47)
            .style(Style::default().stroke(Palette::ink(), 3.0, 1.0))
            .build()
            .moved_to(v(x, y)),
    );
    stage.arena_mut().set_z_index(ring, 4, true);
    fade(stage, &[ring], true)?;
    let mut paths = Vec::new();
    for pair in order.windows(2) {
        let (ax, ay) = positions[pair[0]];
        let (bx, by) = positions[pair[1]];
        let path = stage.arena_mut().add(
            Arrow::new(v(ax, ay), v(bx, by))
                .buff(0.5)
                .thickness(3.0)
                .style(
                    Style::default()
                        .fill(Palette::blue(), 0.9)
                        .stroke(Palette::blue(), 3.0, 0.9),
                )
                .build()?,
        );
        d4g::show(stage, &[path], 0.25)?;
        let motion = ring
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.7),
                ..AnimateArgs::default()
            })?
            .move_to(v(bx, by), ORIGIN)?;
        let animation = stage.prepare(motion)?;
        stage.play_prepared(vec![animation])?;
        let mapping = k.tex(
            stage,
            &format!("{}\\mapsto{}", pair[0] + 1, pair[1] + 1),
            30.0,
            Palette::blue(),
            &[],
            v(0.0, -2.6),
        )?;
        d4g::show(stage, &[mapping], 0.25)?;
        stage.wait(0.25)?;
        d4g::hide(stage, &[mapping], 0.25)?;
        paths.push(path);
    }
    fade(stage, &[ring], false)?;
    Ok(paths)
}

/// The arrow half of `cycle_trace`, in one colour.
fn links(stage: &mut Stage<'_>, order: &[usize], color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut paths = Vec::new();
    for pair in order.windows(2) {
        paths.push(straight(stage, TRAIL[pair[0]], TRAIL[pair[1]], 0.5, color)?);
    }
    fade(stage, &paths, true)?;
    Ok(paths)
}

/// The ring walks arrows already on screen: about 3.35 s, not a whole trace's 8.
fn ring_walk(stage: &mut Stage<'_>, order: &[usize]) -> fmn::Result<()> {
    let walker = halo(stage, TRAIL[order[0]], 0.47, Palette::ink(), 3.0);
    fade(stage, &[walker], true)?;
    for &next in &order[1..] {
        d4g::glide(stage, walker, TRAIL[next], 0.85)?;
    }
    fade(stage, &[walker], false)
}

// ---- 11_1: what a permutation is not ----------------------------------------

fn map_source(row: usize) -> (f64, f64) {
    (-2.6, 1.1 - row as f64 * 1.1)
}

fn map_slot(row: usize) -> (f64, f64) {
    (2.6, 1.1 - row as f64 * 1.1)
}

fn map_arrows(stage: &mut Stage<'_>, map: [usize; 3], color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut arrows = Vec::new();
    for (row, &slot) in map.iter().enumerate() {
        arrows.push(straight(stage, map_source(row), map_slot(slot), 0.45, color)?);
    }
    Ok(arrows)
}

/// A legal map, then the two ways a map can fail to be a permutation. Returns
/// what is still on screen.
fn collisions(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let mut picture = Vec::new();
    for row in 0..3 {
        let slot = map_slot(row);
        picture.push(card(stage, k, row + 1, map_source(row), 0.31)?);
        picture.push(ring(stage, slot, 0.36, Palette::dim(), 2.0));
        picture.push(k.text(
            stage,
            &(row + 1).to_string(),
            23.0,
            Palette::dim(),
            v(slot.0 + 0.6, slot.1),
        )?);
    }
    fade(stage, &picture, true)?;
    let legal = map_arrows(stage, LEGAL_MAP, Palette::dim())?;
    fade(stage, &legal, true)?;

    until_phrase(stage, k, NO_COLLISIONS)?;
    fade(stage, &legal, false)?;
    let clashing = map_arrows(stage, BAD_MAP, Palette::red())?;
    let crowded = ring(stage, map_slot(BAD_DOUBLED), 0.46, Palette::red(), 4.0);
    let crowded_note = k.text(
        stage,
        "collision",
        27.0,
        Palette::red(),
        v(4.4, map_slot(BAD_DOUBLED).1),
    )?;
    let mut showing = clashing.clone();
    showing.extend([crowded, crowded_note]);
    fade(stage, &showing, true)?;

    until_phrase(stage, k, NO_MISSING)?;
    let empty = ring(stage, map_slot(BAD_MISSING), 0.46, Palette::gold(), 4.0);
    let empty_note = k.text(stage, "missing", 27.0, Palette::gold(), v(4.4, map_slot(BAD_MISSING).1))?;
    fade(stage, &[empty, empty_note], true)?;

    picture.extend(showing);
    picture.extend([empty, empty_note]);
    Ok(picture)
}

// ---- 11_4: running a cycle backwards, then two separate loops ---------------

const ROW_TOP: f64 = 0.75;
const ROW_BOTTOM: f64 = -1.25;
const CAPTION_Y: f64 = -2.45;
const LOOP_BEND: f64 = 1.2;
const SWAP_BEND: f64 = 1.0;
const WRONG_BEND: f64 = 2.2;

fn row_slot(slot: usize, y: f64) -> (f64, f64) {
    (-3.4 + 1.5 * slot as f64, y)
}

fn above_top(slot: usize) -> (f64, f64) {
    let (x, y) = row_slot(slot, ROW_TOP);
    (x, y + 0.66)
}

fn below_top(slot: usize) -> (f64, f64) {
    let (x, y) = row_slot(slot, ROW_TOP);
    (x, y - 0.42)
}

fn loop_color(label: usize) -> Srgb {
    match label {
        1 | 3 | 5 => Palette::hero(),
        2 | 6 => Palette::blue(),
        _ => Palette::dim(),
    }
}

struct SixRow {
    y: f64,
    cards: Vec<Mob>,
    at: [usize; 6],
    rest: Vec<Mob>,
}

impl SixRow {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.rest.clone();
        all.extend(&self.cards);
        all
    }

    /// Perform `p` on the row's bookkeeping, returning the cards that move.
    fn moves(&mut self, p: [usize; 6]) -> Vec<(Mob, (f64, f64))> {
        let mut moves = Vec::new();
        for (card, slot) in self.cards.iter().zip(self.at.iter_mut()) {
            if p[*slot] != *slot {
                *slot = p[*slot];
                moves.push((*card, row_slot(*slot, self.y)));
            }
        }
        moves
    }
}

fn six_row(stage: &mut Stage<'_>, k: &Kit, y: f64, name: &str) -> fmn::Result<SixRow> {
    let mut rest = vec![k.text(stage, name, 24.0, Palette::dim(), v(-5.3, y))?];
    let mut cards = Vec::new();
    for slot in 0..6 {
        let at = row_slot(slot, y);
        rest.push(ring(stage, at, 0.31, Palette::dim(), 2.0));
        rest.push(k.text(stage, &(slot + 1).to_string(), 23.0, Palette::dim(), v(at.0, y + 0.46))?);
        cards.push(token(
            stage,
            k,
            &(slot + 1).to_string(),
            loop_color(slot + 1),
            at,
            0.27,
        )?);
    }
    Ok(SixRow {
        y,
        cards,
        at: std::array::from_fn(|slot| slot),
        rest,
    })
}

fn undo_and_disjoint(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    // Sentence 0: the first physical rearrangement. The cards run along the
    // arrows, then along the same arrows reversed, and land where they began.
    let cycle = cycle_images::<4>(&[&[1, 3, 4]]);
    let inverse = cycle_images::<4>(&[&[1, 4, 3]]);
    let mut board = Vec::new();
    let mut cards = Vec::new();
    for (index, &at) in TRAIL.iter().enumerate() {
        board.push(ring(stage, at, 0.44, Palette::dim(), 2.0));
        board.push(k.text(
            stage,
            &(index + 1).to_string(),
            23.0,
            Palette::dim(),
            v(at.0, at.1 + 0.62),
        )?);
        cards.push(card(stage, k, index + 1, at, 0.37)?);
    }
    let mut appearing = board.clone();
    appearing.extend(&cards);
    fade(stage, &appearing, true)?;
    let forward = links(stage, &[0, 2, 3, 0], Palette::blue())?;
    let outward: Vec<(Mob, (f64, f64))> = cards.iter().zip(cycle).map(|(&c, slot)| (c, TRAIL[slot])).collect();
    d4g::glide_all(stage, &outward, 0.7)?;
    fade(stage, &forward, false)?;
    let backward = links(stage, &[0, 3, 2, 0], Palette::hero())?;
    let homeward: Vec<(Mob, (f64, f64))> = cards
        .iter()
        .zip(then(cycle, inverse))
        .map(|(&c, slot)| (c, TRAIL[slot]))
        .collect();
    d4g::glide_all(stage, &homeward, 0.7)?;

    k.until_sentence(stage, "11_4", 1)?;
    let written = k.tex(
        stage,
        r"(1\;3\;4)^{-1}=(1\;4\;3)",
        38.0,
        Palette::ink(),
        &[],
        v(0.0, -2.4),
    )?;
    fade(stage, &[written], true)?;

    // Sentence 2: two loops that share no label, run in both orders at once.
    k.until_sentence(stage, "11_4", 2)?;
    let mut leaving = board;
    leaving.extend(&cards);
    leaving.extend(backward);
    leaving.push(written);
    fade(stage, &leaving, false)?;
    let tri = cycle_images::<6>(&[&[1, 3, 5]]);
    let swap = cycle_images::<6>(&[&[2, 6]]);
    let mut top = six_row(stage, k, ROW_TOP, "(1 3 5), then (2 6)")?;
    let mut bottom = six_row(stage, k, ROW_BOTTOM, "(2 6), then (1 3 5)")?;
    let mut on_screen = top.mobs();
    on_screen.extend(bottom.mobs());
    on_screen.push(curved(stage, above_top(0), above_top(2), -LOOP_BEND, Palette::hero())?);
    on_screen.push(curved(stage, above_top(2), above_top(4), -LOOP_BEND, Palette::hero())?);
    on_screen.push(curved(stage, below_top(1), below_top(5), SWAP_BEND, Palette::blue())?);
    on_screen.push(curved(stage, below_top(5), below_top(1), -SWAP_BEND, Palette::blue())?);
    let closing = curved(stage, above_top(4), above_top(0), LOOP_BEND, Palette::hero())?;
    let mut appearing = on_screen.clone();
    appearing.push(closing);
    fade(stage, &appearing, true)?;

    until_phrase(stage, k, THEY_COMMUTE)?;
    let mut moves = top.moves(tri);
    moves.extend(bottom.moves(swap));
    travel(stage, &moves, CARD_ARC, 0.8)?;
    stage.wait(0.2)?;
    let mut moves = top.moves(swap);
    moves.extend(bottom.moves(tri));
    travel(stage, &moves, CARD_ARC, 0.8)?;
    let equal = k.tex(
        stage,
        r"(1\;3\;5)(2\;6)=(2\;6)(1\;3\;5)",
        34.0,
        Palette::ink(),
        &[],
        v(0.0, 3.0),
    )?;
    let contrast = k.text(
        stage,
        "Overlapping: order matters. Separate: it does not.",
        27.0,
        Palette::green(),
        v(0.0, CAPTION_Y),
    )?;
    let top_reading = arrangement_text(&cards_by_slot(top.at));
    let bottom_reading = arrangement_text(&cards_by_slot(bottom.at));
    let top_result = k.text(stage, &top_reading, 24.0, Palette::ink(), v(5.7, ROW_TOP))?;
    let bottom_result = k.text(stage, &bottom_reading, 24.0, Palette::ink(), v(5.7, ROW_BOTTOM))?;
    fade(stage, &[equal, contrast, top_result, bottom_result], true)?;
    on_screen.extend([equal, top_result, bottom_result]);

    // Sentence 3: one trail closes, on the fixed slots of the top row.
    k.until_sentence(stage, "11_4", 3)?;
    let walker = halo(stage, row_slot(0, ROW_TOP), 0.44, Palette::ink(), 3.0);
    fade(stage, &[walker], true)?;
    travel(stage, &[(walker, row_slot(2, ROW_TOP))], -LOOP_BEND, 0.85)?;
    travel(stage, &[(walker, row_slot(4, ROW_TOP))], -LOOP_BEND, 0.85)?;
    travel(stage, &[(walker, row_slot(0, ROW_TOP))], LOOP_BEND, 0.85)?;
    fade(stage, &[walker], false)?;

    // Sentence 4: the impossible arrow. Slot 3 would have two inputs.
    k.until_sentence(stage, "11_4", 4)?;
    fade(stage, &[contrast, closing], false)?;
    let wrong = curved(stage, above_top(4), above_top(2), WRONG_BEND, Palette::red())?;
    fade(stage, &[wrong], true)?;
    until_phrase(stage, k, TWO_INPUTS)?;
    let clash = halo(stage, row_slot(2, ROW_TOP), 0.44, Palette::red(), 4.0);
    let warning = k.text(
        stage,
        "Two cards into slot 3: impossible",
        27.0,
        Palette::red(),
        v(0.0, CAPTION_Y),
    )?;
    fade(stage, &[clash, warning], true)?;

    // Sentence 5: the trail closes properly, and the permutation is written.
    k.until_sentence(stage, "11_4", 5)?;
    fade(stage, &[wrong, clash, warning], false)?;
    let closing = curved(stage, above_top(4), above_top(0), LOOP_BEND, Palette::hero())?;
    let cycles = k.tex(
        stage,
        r"(1\;3\;5)(2\;6)",
        34.0,
        Palette::green(),
        &[],
        v(-1.6, CAPTION_Y),
    )?;
    let fixed = k.text(stage, "4 stays put", 27.0, Palette::dim(), v(2.4, CAPTION_Y))?;
    fade(stage, &[closing, cycles, fixed], true)?;
    pulse(stage, &[top.cards[3], bottom.cards[3]])?;

    k.until_sentence(stage, "11_4", 6)?;
    let restated = k.tex(
        stage,
        r"(1\;3\;5)(2\;6)=(2\;6)(3\;5\;1)",
        34.0,
        Palette::green(),
        &[],
        v(-1.6, CAPTION_Y),
    )?;
    morph(stage, cycles, restated, 0.5)?;
    on_screen.extend([closing, restated, fixed]);
    Ok(on_screen)
}

// ---- 11_7: one swap on a numeric witness ------------------------------------

const WITNESS_Y: f64 = 1.6;
const CHIP_Y: f64 = 0.1;

fn witness_slot(slot: usize) -> (f64, f64) {
    (-2.25 + 1.5 * slot as f64, WITNESS_Y)
}

fn chip_xy(index: usize) -> (f64, f64) {
    (-4.0 + 1.6 * index as f64, CHIP_Y)
}

/// A factor: a frame and its number. The two are separate mobjects so the
/// number can be replaced while the frame stays.
struct Chip {
    frame: Mob,
    value: Mob,
}

fn chip_number(stage: &mut Stage<'_>, k: &Kit, n: i32, at: (f64, f64), color: Srgb) -> fmn::Result<Mob> {
    k.tex(stage, &n.to_string(), 26.0, color, &[], v(at.0, at.1))
}

fn chip(stage: &mut Stage<'_>, k: &Kit, n: i32, at: (f64, f64)) -> fmn::Result<Chip> {
    let frame = Rectangle::new()
        .width(0.98)
        .height(0.58)
        .corner_radius(0.12)
        .style(stroke_style(Palette::ink(), 3.0))
        .build()?
        .moved_to(v(at.0, at.1));
    let frame = stage.arena_mut().add(frame);
    stage.arena_mut().set_z_index(frame, 2, true);
    let value = chip_number(stage, k, n, at, Palette::ink())?;
    stage.arena_mut().set_z_index(value, 2, true);
    Ok(Chip { frame, value })
}

/// Colour two chips, then let them trade places.
fn trade(stage: &mut Stage<'_>, chips: &[Chip], pair: [usize; 2], to: &[usize; 6], color: Srgb) -> fmn::Result<()> {
    let mut parts = Vec::new();
    let mut moves = Vec::new();
    for index in pair {
        let (chip, target) = (&chips[index], chip_xy(to[index]));
        parts.extend([chip.frame, chip.value]);
        moves.extend([(chip.frame, target), (chip.value, target)]);
    }
    recolor(stage, &parts, color, 0.3)?;
    travel(stage, &moves, SWAP_ARC, 0.8)
}

fn witness_proof(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    // Sentence 2: numbers on fixed labels, then every difference as a chip.
    k.until_sentence(stage, "11_7", 2)?;
    let mut on_screen = Vec::new();
    let mut cards = Vec::new();
    for (slot, &value) in WITNESS.iter().enumerate() {
        let at = witness_slot(slot);
        on_screen.push(ring(stage, at, 0.36, Palette::dim(), 2.0));
        on_screen.push(k.tex(
            stage,
            &format!("x_{}", slot + 1),
            26.0,
            Palette::dim(),
            &[],
            v(at.0, at.1 + 0.58),
        )?);
        cards.push(token(stage, k, &value.to_string(), d4g::corner_color(slot), at, 0.31)?);
    }
    on_screen.push(k.tex(stage, "n=4", 26.0, Palette::dim(), &[], v(3.7, WITNESS_Y))?);
    let mut appearing = on_screen.clone();
    appearing.extend(&cards);
    fade(stage, &appearing, true)?;
    on_screen.extend(&cards);

    until_phrase(stage, k, ALL_DIFFERENCES)?;
    let before = differences(WITNESS);
    let after = differences(after_swaps(WITNESS, &ONE_SWAP));
    let to = chip_destinations(before, after);
    let mut chips = Vec::new();
    for (index, &(i, j)) in PAIRS.iter().enumerate() {
        let at = chip_xy(index);
        let header = k.tex(
            stage,
            &format!("x_{}-x_{}", j + 1, i + 1),
            24.0,
            Palette::dim(),
            &[],
            v(at.0, at.1 + 0.62),
        )?;
        let made = chip(stage, k, before[index], at)?;
        d4g::show(stage, &[header, made.frame, made.value], 0.2)?;
        on_screen.push(header);
        chips.push(made);
    }

    until_phrase(stage, k, THE_PAIRS)?;
    let tally = k.tex(stage, &tally_tex(WITNESS), 30.0, Palette::ink(), &[], v(0.0, -1.6))?;
    fade(stage, &[tally], true)?;

    // 11_11 sentence 0: the numbers on labels 1 and 3 change places; labels stay.
    k.say(stage, "11_11")?;
    let (a, b) = ONE_SWAP[0];
    travel(
        stage,
        &[(cards[a], witness_slot(b)), (cards[b], witness_slot(a))],
        SWAP_ARC,
        0.9,
    )?;

    // Sentence 1: the factor between the swapped labels reverses in place.
    k.until_sentence(stage, "11_11", 1)?;
    recolor(
        stage,
        &[chips[REVERSES_IN_PLACE].frame, chips[REVERSES_IN_PLACE].value],
        Palette::red(),
        0.3,
    )?;
    let reversed = chip_number(
        stage,
        k,
        after[to[REVERSES_IN_PLACE]],
        chip_xy(to[REVERSES_IN_PLACE]),
        Palette::red(),
    )?;
    stage.arena_mut().set_z_index(reversed, 2, true);
    morph(stage, chips[REVERSES_IN_PLACE].value, reversed, 0.5)?;
    chips[REVERSES_IN_PLACE].value = reversed;
    pulse(stage, &[chips[REVERSES_IN_PLACE].frame])?;

    // Sentence 2: two factors through label 4 trade; two through label 2
    // trade and both reverse. Each pair keeps its product.
    k.until_sentence(stage, "11_11", 2)?;
    trade(stage, &chips, TRADE_ONLY, &to, Palette::blue())?;
    until_phrase(stage, k, BOTH_REVERSE)?;
    trade(stage, &chips, TRADE_AND_REVERSE, &to, Palette::gold())?;
    for index in TRADE_AND_REVERSE {
        let target = to[index];
        let reversed = chip_number(stage, k, after[target], chip_xy(target), Palette::gold())?;
        stage.arena_mut().set_z_index(reversed, 2, true);
        morph(stage, chips[index].value, reversed, 0.4)?;
        chips[index].value = reversed;
    }
    until_phrase(stage, k, SAME_PRODUCT)?;
    let gold_note = k.tex(
        stage,
        &format!(
            r"({})({})={}\cdot{}",
            after[TRADE_AND_REVERSE[0]],
            after[TRADE_AND_REVERSE[1]],
            before[TRADE_AND_REVERSE[0]],
            before[TRADE_AND_REVERSE[1]]
        ),
        26.0,
        Palette::gold(),
        &[],
        v(-2.4, -0.75),
    )?;
    let blue_note = k.tex(
        stage,
        &format!(
            r"{}\cdot{}={}\cdot{}",
            after[TRADE_ONLY[0]], after[TRADE_ONLY[1]], before[TRADE_ONLY[0]], before[TRADE_ONLY[1]]
        ),
        26.0,
        Palette::blue(),
        &[],
        v(2.4, -0.75),
    )?;
    fade(stage, &[gold_note, blue_note], true)?;

    // Sentence 3: the factor that avoids both labels is left exactly alone.
    k.until_sentence(stage, "11_11", 3)?;
    recolor(
        stage,
        &[chips[UNTOUCHED].frame, chips[UNTOUCHED].value],
        Palette::dim(),
        0.4,
    )?;

    // Sentence 4: same size, opposite sign.
    k.until_sentence(stage, "11_11", 4)?;
    let swapped_tally = k.tex(
        stage,
        &tally_tex(after_swaps(WITNESS, &ONE_SWAP)),
        30.0,
        Palette::ink(),
        &[],
        v(0.0, -1.6),
    )?;
    morph(stage, tally, swapped_tally, 0.4)?;
    let sign = k.tex(
        stage,
        &format!(
            r"\text{{one swap}}:\quad \Delta\mapsto-\Delta,\quad |\Delta|={}",
            delta(WITNESS)
        ),
        34.0,
        Palette::hero(),
        &[],
        v(0.0, -2.5),
    )?;
    fade(stage, &[sign], true)?;

    for made in &chips {
        on_screen.extend([made.frame, made.value]);
    }
    on_screen.extend([gold_note, blue_note, swapped_tally, sign]);
    Ok(on_screen)
}

// ---- 11_8: two routes to one arrangement ------------------------------------

const ROUTE_SLOTS: [f64; 4] = [-2.4, -1.2, 0.0, 1.2];

/// One swap route on the witness: fixed labels, moving numbers, a sign lamp.
struct RouteRow {
    y: f64,
    x: [i32; 4],
    cards: Vec<Mob>,
    at: [usize; 4],
    lamp: Mob,
    value: Mob,
    rest: Vec<Mob>,
}

impl RouteRow {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.rest.clone();
        all.extend(&self.cards);
        all.extend([self.lamp, self.value]);
        all
    }
}

/// The sign lamp and the product it reports, for the arrangement `x`.
fn lamp_for(stage: &mut Stage<'_>, k: &Kit, x: [i32; 4], y: f64) -> fmn::Result<(Mob, Mob)> {
    let product = delta(x);
    let (color, sign) = if product > 0 {
        (Palette::green(), "+")
    } else {
        (Palette::red(), "−")
    };
    let lamp = token(stage, k, sign, color, (2.7, y), 0.3)?;
    let value = k.tex(stage, &format!(r"\Delta={product:+}"), 28.0, color, &[], v(4.4, y))?;
    Ok((lamp, value))
}

fn route_row(stage: &mut Stage<'_>, k: &Kit, name: &str, steps: &str, y: f64) -> fmn::Result<RouteRow> {
    let mut rest = vec![
        k.text(stage, name, 26.0, Palette::ink(), v(-5.0, y + 0.18))?,
        k.text(stage, steps, 21.0, Palette::dim(), v(-5.0, y - 0.2))?,
    ];
    let mut cards = Vec::new();
    for (slot, (&value, &slot_x)) in WITNESS.iter().zip(&ROUTE_SLOTS).enumerate() {
        rest.push(ring(stage, (slot_x, y), 0.36, Palette::dim(), 2.0));
        rest.push(k.tex(
            stage,
            &format!("x_{}", slot + 1),
            22.0,
            Palette::dim(),
            &[],
            v(slot_x, y + 0.52),
        )?);
        cards.push(token(
            stage,
            k,
            &value.to_string(),
            d4g::corner_color(slot),
            (slot_x, y),
            0.31,
        )?);
    }
    let (lamp, value) = lamp_for(stage, k, WITNESS, y)?;
    Ok(RouteRow {
        y,
        x: WITNESS,
        cards,
        at: [0, 1, 2, 3],
        lamp,
        value,
        rest,
    })
}

/// One swap on a row: the two numbers cross, then the lamp reports the product.
fn step_route(stage: &mut Stage<'_>, k: &Kit, row: &mut RouteRow, swap: (usize, usize)) -> fmn::Result<()> {
    let (a, b) = swap;
    let mut moves = Vec::new();
    for (card, slot) in row.cards.iter().zip(row.at.iter_mut()) {
        if *slot == a || *slot == b {
            *slot = if *slot == a { b } else { a };
            moves.push((*card, (ROUTE_SLOTS[*slot], row.y)));
        }
    }
    travel(stage, &moves, SWAP_ARC, 0.7)?;
    row.x.swap(a, b);
    let (lamp, value) = lamp_for(stage, k, row.x, row.y)?;
    exchange(stage, &[row.lamp, row.value], &[lamp, value])?;
    row.lamp = lamp;
    row.value = value;
    Ok(())
}

fn route_frame(stage: &mut Stage<'_>, y: f64) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(8.45)
        .height(1.25)
        .corner_radius(0.14)
        .style(stroke_style(Palette::green(), 3.0))
        .build()?
        .moved_to(v(1.225, y + 0.1));
    Ok(stage.arena_mut().add(frame))
}

fn two_routes(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let mut one = route_row(stage, k, "One swap", "(1 3)", 1.35)?;
    let mut three = route_row(stage, k, "Three swaps", "(1 2), (2 3), (1 2)", -0.35)?;
    let mut showing = one.mobs();
    showing.extend(three.mobs());
    fade(stage, &showing, true)?;

    // Sentence 0: the longer route, one lamp flip per swap.
    for swap in THREE_SWAPS {
        step_route(stage, k, &mut three, swap)?;
        stage.wait(0.15)?;
    }

    // Sentence 1: the shorter route reaches the same arrangement and product.
    k.until_sentence(stage, "11_8", 1)?;
    step_route(stage, k, &mut one, ONE_SWAP[0])?;
    let frames = [route_frame(stage, one.y)?, route_frame(stage, three.y)?];
    let nonzero = k.tex(
        stage,
        &format!(r"\Delta_{{\text{{initial}}}}={}\neq0", delta(WITNESS)),
        30.0,
        Palette::green(),
        &[],
        v(0.0, -1.35),
    )?;
    fade(stage, &[frames[0], frames[1], nonzero], true)?;

    // Sentence 2: the two descriptions, and why no even route can exist.
    k.until_sentence(stage, "11_8", 2)?;
    let identity = k.tex(
        stage,
        r"(1\;3)=(1\;2)(2\;3)(1\;2)",
        30.0,
        Palette::ink(),
        &[],
        v(-2.6, -2.1),
    )?;
    let both_odd = k.text(stage, "1 swap, 3 swaps: both odd", 26.0, Palette::ink(), v(2.7, -2.1))?;
    let impossible = k.text(
        stage,
        &format!("2 swaps would give {:+}: impossible", delta(WITNESS)),
        27.0,
        Palette::red(),
        v(0.0, -2.85),
    )?;
    fade(stage, &[identity, both_odd, impossible], true)?;

    // Sentence 3: the rule itself.
    k.until_sentence(stage, "11_8", 3)?;
    fade(stage, &[identity, both_odd, impossible], false)?;
    let parity = k.tex(
        stage,
        r"\Delta_{\text{final}}=(-1)^N\Delta_{\text{initial}}",
        34.0,
        Palette::green(),
        &[],
        v(0.0, -2.15),
    )?;
    let swap_count = k.text(stage, "N = number of swaps", 26.0, Palette::green(), v(0.0, -2.95))?;
    fade(stage, &[parity, swap_count], true)?;

    let mut on_screen = one.mobs();
    on_screen.extend(three.mobs());
    on_screen.extend([frames[0], frames[1], nonzero, parity, swap_count]);
    Ok(on_screen)
}

// ---- 11_9: the square's corners as labelled cards ---------------------------

const SQUARE_HALF: f64 = 1.15;
const SQUARE_LEFT: (f64, f64) = (-3.7, 0.4);
const SQUARE_RIGHT: (f64, f64) = (3.7, 0.4);
const PANEL_X: f64 = 2.95;

fn corner_xy(center: (f64, f64), slot: usize) -> (f64, f64) {
    let corner = CORNERS[slot];
    (center.0 + SQUARE_HALF * corner[0], center.1 + SQUARE_HALF * corner[1])
}

/// The outline drawn through the corners that hold cards 1, 2, 3, 4, in order.
fn outline_through(stage: &mut Stage<'_>, center: (f64, f64), at: [usize; 4]) -> Mob {
    let vertices = at.map(|slot| {
        let (x, y) = corner_xy(center, slot);
        v(x, y)
    });
    stage
        .arena_mut()
        .add(Polygon::new(vertices).style(stroke_style(Palette::dim(), 3.0)).build())
}

/// Four fixed numbered corners, an outline and four moving cards.
struct Square {
    center: (f64, f64),
    cards: Vec<Mob>,
    at: [usize; 4],
    outline: Mob,
    corners: Vec<Mob>,
}

impl Square {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.corners.clone();
        all.push(self.outline);
        all.extend(&self.cards);
        all
    }
}

fn new_square(stage: &mut Stage<'_>, k: &Kit, center: (f64, f64)) -> fmn::Result<Square> {
    let at = [0, 1, 2, 3];
    let outline = outline_through(stage, center, at);
    let mut corners = Vec::new();
    let mut cards = Vec::new();
    for slot in at {
        let (x, y) = corner_xy(center, slot);
        let corner = CORNERS[slot];
        corners.push(ring(stage, (x, y), 0.36, Palette::dim(), 2.0));
        corners.push(k.text(
            stage,
            &(slot + 1).to_string(),
            22.0,
            Palette::dim(),
            v(x + 0.52 * corner[0], y + 0.52 * corner[1]),
        )?);
        cards.push(card(stage, k, slot + 1, (x, y), 0.3)?);
    }
    Ok(Square {
        center,
        cards,
        at,
        outline,
        corners,
    })
}

/// Perform `p` on the square's cards; `follow` carries a ring along with one card.
fn perform(
    stage: &mut Stage<'_>,
    square: &mut Square,
    p: [usize; 4],
    arc: f64,
    run_time: f64,
    follow: Option<(Mob, usize)>,
) -> fmn::Result<()> {
    let mut moves = Vec::new();
    for (card, slot) in square.cards.iter().zip(square.at.iter_mut()) {
        if p[*slot] != *slot {
            *slot = p[*slot];
            moves.push((*card, corner_xy(square.center, *slot)));
        }
    }
    if let Some((walker, followed)) = follow {
        moves.push((walker, corner_xy(square.center, square.at[followed])));
    }
    travel(stage, &moves, arc, run_time)
}

/// The map that returns every card of `at` to its own corner.
fn way_home(at: [usize; 4]) -> [usize; 4] {
    let mut back = [0; 4];
    for (card, &slot) in at.iter().enumerate() {
        back[slot] = card;
    }
    back
}

fn square_as_labels(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let r = corner_images(R.turns, R.flip);
    let s = corner_images(S.turns, S.flip);

    // Sentence 0: a permutation that tears the square.
    let mut left = new_square(stage, k, SQUARE_LEFT)?;
    fade(stage, &left.mobs(), true)?;
    until_phrase(stage, k, NOT_EVERY)?;
    let trade = cycle_images::<4>(&[&[1, 2]]);
    perform(stage, &mut left, trade, SWAP_ARC, 0.9, None)?;
    let bow_tie = outline_through(stage, SQUARE_LEFT, left.at);
    morph(stage, left.outline, bow_tie, 0.6)?;
    left.outline = bow_tie;
    let mut torn = Vec::new();
    for (i, j) in broken_edges(left.at) {
        let (a, b) = (corner_xy(SQUARE_LEFT, left.at[i]), corner_xy(SQUARE_LEFT, left.at[j]));
        torn.push(
            stage.arena_mut().add(
                Line::new(v(a.0, a.1), v(b.0, b.1))
                    .style(stroke_style(Palette::red(), 4.0))
                    .build()?,
            ),
        );
    }
    let not_symmetry = k.text(
        stage,
        "A permutation, not a symmetry",
        27.0,
        Palette::red(),
        v(SQUARE_LEFT.0, -1.9),
    )?;
    let why = k.text(
        stage,
        "Neighbors 2 and 3 land on opposite corners",
        27.0,
        Palette::ink(),
        v(PANEL_X, 0.4),
    )?;
    let mut showing = torn.clone();
    showing.extend([not_symmetry, why]);
    fade(stage, &showing, true)?;

    // Sentence 1: 24 arrangements form a group; 8 of them keep the square.
    k.until_sentence(stage, "11_9", 1)?;
    fade(stage, &showing, false)?;
    let home = way_home(left.at);
    perform(stage, &mut left, home, SWAP_ARC, 0.6, None)?;
    let square_again = outline_through(stage, SQUARE_LEFT, left.at);
    morph(stage, left.outline, square_again, 0.4)?;
    left.outline = square_again;
    let count = k.tex(
        stage,
        r"4\times3\times2\times1=24:\quad\text{the group }S_4",
        30.0,
        Palette::ink(),
        &[],
        v(PANEL_X, 2.75),
    )?;
    let mut panel = vec![count];
    let symmetric = symmetries();
    let mut kept = Vec::new();
    for (index, p) in arrangements().into_iter().enumerate() {
        let (column, row) = (index % 8, index / 8);
        let cell = k.text(
            stage,
            &cell_text(p),
            22.0,
            Palette::dim(),
            v(PANEL_X - 2.8 + 0.8 * column as f64, 2.05 - 0.45 * row as f64),
        )?;
        panel.push(cell);
        if symmetric.contains(&p) {
            kept.push(cell);
        }
    }
    fade(stage, &panel, true)?;
    until_phrase(stage, k, OUR_EIGHT)?;
    recolor(stage, &kept, Palette::gold(), 0.4)?;
    let eight = k.text(
        stage,
        "4 choices × 2 neighbors = 8",
        26.0,
        Palette::gold(),
        v(PANEL_X, 0.55),
    )?;
    let mut rack = vec![eight];
    for (index, &g) in ALL.iter().enumerate() {
        let at = (
            PANEL_X - 1.725 + 1.15 * (index % 4) as f64,
            -0.35 - 1.1 * (index / 4) as f64,
        );
        rack.push(d4g::thumb(stage, k, g, at, 0.85)?);
    }
    fade(stage, &rack, true)?;
    panel.extend(rack);
    until_phrase(stage, k, AS_SUBGROUP)?;
    let subgroup = k.text(
        stage,
        "8 inside 24: a subgroup",
        28.0,
        Palette::gold(),
        v(PANEL_X, -2.5),
    )?;
    fade(stage, &[subgroup], true)?;
    panel.push(subgroup);

    // Sentence 2: the cards themselves turn; the swap counts are computed.
    k.until_sentence(stage, "11_9", 2)?;
    fade(stage, &panel, false)?;
    perform(stage, &mut left, r, FRAC_PI_2, 0.9, None)?;
    let quarter = k.tex(
        stage,
        &format!(r"r={}=(1\;2)(1\;3)(1\;4)", cycle_tex(r)),
        30.0,
        Palette::blue(),
        &[],
        v(PANEL_X, 1.4),
    )?;
    let quarter_swaps = swaps_needed(r);
    let quarter_count = k.text(
        stage,
        &format!("{quarter_swaps} swaps: {}", parity_word(quarter_swaps)),
        28.0,
        Palette::blue(),
        v(PANEL_X, 0.65),
    )?;
    fade(stage, &[quarter, quarter_count], true)?;
    until_phrase(stage, k, HALF_TURN)?;
    perform(stage, &mut left, r, FRAC_PI_2, 0.9, None)?;
    let half = then(r, r);
    let half_written = k.tex(
        stage,
        &format!(r"r^2={}", cycle_tex(half)),
        30.0,
        Palette::hero(),
        &[],
        v(PANEL_X, -0.55),
    )?;
    let half_swaps = swaps_needed(half);
    let half_count = k.text(
        stage,
        &format!("{half_swaps} swaps: {}", parity_word(half_swaps)),
        28.0,
        Palette::hero(),
        v(PANEL_X, -1.3),
    )?;
    fade(stage, &[half_written, half_count], true)?;

    // Sentence 3: r s and s r, performed by the cards on two squares.
    k.until_sentence(stage, "11_9", 3)?;
    fade(stage, &[quarter, quarter_count, half_written, half_count], false)?;
    let home = way_home(left.at);
    perform(stage, &mut left, home, PI, 0.8, None)?;
    let mut right = new_square(stage, k, SQUARE_RIGHT)?;
    fade(stage, &right.mobs(), true)?;

    until_phrase(stage, k, RS_SWAP)?;
    let left_walker = halo(stage, corner_xy(SQUARE_LEFT, 1), 0.42, Palette::ink(), 3.0);
    fade(stage, &[left_walker], true)?;
    perform(stage, &mut left, s, SWAP_ARC, 0.6, Some((left_walker, 1)))?;
    perform(stage, &mut left, r, FRAC_PI_2, 0.6, Some((left_walker, 1)))?;
    let rs = k.tex(
        stage,
        &format!("rs={}", cycle_tex(then(s, r))),
        32.0,
        Palette::hero(),
        &[],
        v(SQUARE_LEFT.0, -1.9),
    )?;
    fade(stage, &[rs], true)?;

    until_phrase(stage, k, SR_SWAP)?;
    let right_walker = halo(stage, corner_xy(SQUARE_RIGHT, 0), 0.42, Palette::ink(), 3.0);
    fade(stage, &[right_walker], true)?;
    perform(stage, &mut right, r, FRAC_PI_2, 0.6, Some((right_walker, 0)))?;
    perform(stage, &mut right, s, SWAP_ARC, 0.6, Some((right_walker, 0)))?;
    let sr = k.tex(
        stage,
        &format!("sr={}", cycle_tex(then(r, s))),
        32.0,
        Palette::blue(),
        &[],
        v(SQUARE_RIGHT.0, -1.9),
    )?;
    fade(stage, &[sr], true)?;

    // Sentence 4, the callback: the same two products as poses of the tile.
    k.until_sentence(stage, "11_9", 4)?;
    let rs_tile = d4g::thumb(stage, k, R.mul(S), (-0.85, 0.6), 1.0)?;
    let sr_tile = d4g::thumb(stage, k, S.mul(R), (0.85, 0.6), 1.0)?;
    let rs_name = k.tex(stage, "rs", 26.0, Palette::hero(), &[], v(-0.85, -0.1))?;
    let sr_name = k.tex(stage, "sr", 26.0, Palette::blue(), &[], v(0.85, -0.1))?;
    let differ = k.tex(stage, r"rs\neq sr", 38.0, Palette::gold(), &[], v(0.0, -2.75))?;
    fade(stage, &[rs_tile, sr_tile, rs_name, sr_name, differ], true)?;

    let mut on_screen = left.mobs();
    on_screen.extend(right.mobs());
    on_screen.extend([
        left_walker,
        right_walker,
        rs,
        sr,
        rs_tile,
        sr_tile,
        rs_name,
        sr_name,
        differ,
    ]);
    Ok(on_screen)
}

impl SceneConstruct for Permutations {
    fn name(&self) -> &str {
        "11_permutations"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        k.say(stage, "11_1")?;
        let title = k.text(
            stage,
            "Permutations: follow the labels",
            36.0,
            Palette::ink(),
            v(0.0, 3.0),
        )?;
        fade(stage, &[title], true)?;
        let bad_map = collisions(stage, &k)?;
        k.until_sentence(stage, "11_1", 1)?;
        fade(stage, &bad_map, false)?;
        let mut cards = Vec::new();
        for (index, &at) in TRAIL.iter().enumerate() {
            cards.push(card(stage, &k, index + 1, at, 0.37)?);
        }
        fade(stage, &cards, true)?;
        let mut scene = vec![title];
        scene.extend(&cards);
        k.until_sentence(stage, "11_1", 3)?;
        pulse(stage, &[cards[0]])?;

        k.say(stage, "11_2")?;
        let forward = cycle_trace(stage, &k, &[0, 2, 3, 0], &TRAIL)?;
        k.until_sentence(stage, "11_2", 1)?;
        let cycle = k.tex(stage, r"(1\;3\;4)", 36.0, Palette::green(), &[], v(1.3, -1.7))?;
        fade(stage, &[cycle], true)?;
        k.until_sentence(stage, "11_2", 2)?;
        pulse(stage, &[cards[1]])?;
        k.until_sentence(stage, "11_2", 3)?;
        // The same arrows, entered at another vertex, give the same map.
        ring_walk(stage, &[2, 3, 0, 2])?;
        fade(stage, &[cycle], false)?;
        let same = k.tex(stage, r"(1\;3\;4)=(3\;4\;1)", 36.0, Palette::green(), &[], v(1.3, -1.7))?;
        fade(stage, &[same], true)?;
        k.until_sentence(stage, "11_2", 4)?;
        // The caption now sits on arrows that agree with it: 1→4→3→1.
        let mut leaving = forward;
        leaving.push(same);
        fade(stage, &leaving, false)?;
        let backward = links(stage, &[0, 3, 2, 0], Palette::hero())?;
        let reverse = k.tex(
            stage,
            r"(1\;3\;4):\quad1\mapsto3\qquad(1\;4\;3):\quad1\mapsto4",
            28.0,
            Palette::hero(),
            &[],
            v(0.0, -2.4),
        )?;
        let two_label = k.text(
            stage,
            "A two-label swap is its own reverse",
            24.0,
            Palette::dim(),
            v(0.0, -3.05),
        )?;
        fade(stage, &[reverse, two_label], true)?;
        scene.extend(backward);
        scene.extend([reverse, two_label]);
        k.hold(stage)?;
        stage.wait(1.0)?;

        k.say(stage, "11_3")?;
        fade(stage, &scene, false)?;
        let heading = k.tex(
            stage,
            r"(1\;2\;3)(1\;2)\quad\text{rightmost first}",
            34.0,
            Palette::ink(),
            &[],
            v(0.0, 3.0),
        )?;
        fade(stage, &[heading], true)?;
        let mut composition = route_board(stage, &k, false)?;
        composition.push(heading);
        k.until_sentence(stage, "11_3", 3)?;
        composition.extend(trace_route(stage, &k, 0, SWAP_12, CYCLE_123)?);
        k.until_sentence(stage, "11_3", 4)?;
        composition.extend(trace_route(stage, &k, 2, SWAP_12, CYCLE_123)?);
        k.until_sentence(stage, "11_3", 5)?;
        composition.extend(trace_route(stage, &k, 1, SWAP_12, CYCLE_123)?);
        k.until_sentence(stage, "11_3", 6)?;
        let result = k.tex(
            stage,
            r"(1\;2\;3)(1\;2)=(1\;3)",
            34.0,
            Palette::green(),
            &[],
            v(0.0, -2.3),
        )?;
        fade(stage, &[result], true)?;
        composition.push(result);
        k.until_sentence(stage, "11_3", 7)?;
        fade(stage, &composition, false)?;
        composition = route_board(stage, &k, true)?;
        let reverse_title = k.text(stage, "Reverse the operations", 34.0, Palette::ink(), v(0.0, 3.0))?;
        fade(stage, &[reverse_title], true)?;
        composition.push(reverse_title);
        composition.extend(all_routes(stage, &k, CYCLE_123, SWAP_12, [1, 2, 3])?);
        let reverse_result = k.tex(
            stage,
            r"(1\;2)(1\;2\;3)=(2\;3)",
            34.0,
            Palette::hero(),
            &[],
            v(0.0, -2.3),
        )?;
        fade(stage, &[reverse_result], true)?;
        composition.push(reverse_result);
        k.hold(stage)?;
        stage.wait(1.0)?;

        k.say(stage, "11_4")?;
        fade(stage, &composition, false)?;
        let disjoint = undo_and_disjoint(stage, &k)?;

        k.say(stage, "11_5")?;
        fade(stage, &disjoint, false)?;
        let factor = k.tex(
            stage,
            r"(1\;3\;4)=(1\;4)(1\;3)",
            36.0,
            Palette::hero(),
            &[],
            v(0.0, 3.0),
        )?;
        fade(stage, &[factor], true)?;
        let mut factor_scene = named_route_board(stage, &k, ["Start", "After (1 3)", "After (1 4)"], [1, 3, 4])?;
        k.until_sentence(stage, "11_5", 3)?;
        for origin in 0..3 {
            factor_scene.extend(named_trace_route(
                stage,
                &k,
                origin,
                SWAP_13_ON_134,
                SWAP_14_ON_134,
                [1, 3, 4],
                0.6,
            )?);
        }
        k.until_sentence(stage, "11_5", 4)?;
        let count = k.text(
            stage,
            "A length-m cycle can be built from m − 1 swaps",
            29.0,
            Palette::ink(),
            v(0.0, -2.2),
        )?;
        fade(stage, &[count], true)?;
        factor_scene.push(count);

        // One permutation, two swap descriptions; the prediction is the last beat.
        k.say(stage, "11_6")?;
        let two_descriptions = k.tex(
            stage,
            r"(1\;3\;4)=(1\;4)(1\;3)=(3\;4)(1\;4)",
            36.0,
            Palette::hero(),
            &[],
            v(0.0, 3.0),
        )?;
        morph(stage, factor, two_descriptions, 0.6)?;
        factor_scene.push(two_descriptions);
        k.until_sentence(stage, "11_6", 1)?;
        let question = k.text(
            stage,
            "Can the same move be both even and odd?",
            31.0,
            Palette::blue(),
            v(0.0, -3.0),
        )?;
        fade(stage, &[question], true)?;
        k.hold(stage)?;
        stage.wait(3.0)?;

        k.say(stage, "11_7")?;
        let belongs = k.text(
            stage,
            "If not: parity belongs to the permutation",
            31.0,
            Palette::blue(),
            v(0.0, -3.0),
        )?;
        morph(stage, question, belongs, 0.6)?;
        factor_scene.push(belongs);
        k.until_sentence(stage, "11_7", 1)?;
        fade(stage, &factor_scene, false)?;
        let product = k.tex(
            stage,
            r"\Delta(x)=\prod_{i<j}(x_j-x_i)",
            36.0,
            Palette::ink(),
            &[],
            v(0.0, 3.05),
        )?;
        fade(stage, &[product], true)?;
        let witness = witness_proof(stage, &k)?;

        k.say(stage, "11_8")?;
        fade(stage, &witness, false)?;
        let routes = two_routes(stage, &k)?;

        k.say(stage, "11_9")?;
        let mut leaving = routes;
        leaving.push(product);
        fade(stage, &leaving, false)?;
        let square = square_as_labels(stage, &k)?;

        k.say(stage, "11_10")?;
        fade(stage, &square, false)?;
        named_route_board(stage, &k, ["Start", "After (1 3 4)", "Then its inverse"], [1, 3, 4])?;
        until_phrase(stage, &k, THE_POSITIONS)?;
        let uses = k.text(
            stage,
            "Positions of cards, wires, the order of data",
            27.0,
            Palette::dim(),
            v(0.0, -2.4),
        )?;
        fade(stage, &[uses], true)?;
        // Every card leaves its row and comes back: the inverse undoes the move.
        k.until_sentence(stage, "11_10", 1)?;
        fade(stage, &[uses], false)?;
        all_routes(stage, &k, CYCLE_134_ON_134, INVERSE_134_ON_134, [1, 3, 4])?;
        k.until_sentence(stage, "11_10", 2)?;
        let limits = k.text(
            stage,
            "The group records the moves, not how well a device performs them",
            27.0,
            Palette::dim(),
            v(0.0, -2.4),
        )?;
        fade(stage, &[limits], true)?;
        k.until_sentence(stage, "11_10", 3)?;
        let next = k.text(
            stage,
            "Next: when do two groups have the same structure?",
            27.0,
            Palette::ink(),
            v(0.0, -2.4),
        )?;
        exchange(stage, &[limits], &[next])?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;

    #[test]
    fn two_swap_routes_give_the_displayed_three_cycle() {
        let first = [1, 0, 2];
        let second = [2, 1, 0];
        let labels = [1, 3, 4];
        let endpoints = (0..3).map(|i| labels[route(i, first, second)[2]]).collect::<Vec<_>>();
        assert_eq!(endpoints, vec![3, 4, 1]);
    }

    #[test]
    fn displayed_compositions_have_different_endpoints_and_no_collisions() {
        let forward = (0..3).map(|i| route(i, SWAP_12, CYCLE_123)).collect::<Vec<_>>();
        let reverse = (0..3).map(|i| route(i, CYCLE_123, SWAP_12)).collect::<Vec<_>>();
        assert_eq!(forward, vec![[0, 1, 2], [1, 0, 1], [2, 2, 0]]);
        assert_eq!(reverse, vec![[0, 1, 0], [1, 2, 2], [2, 0, 1]]);
        for routes in [&forward, &reverse] {
            for column in 0..3 {
                let mut destinations = routes.iter().map(|r| r[column]).collect::<Vec<_>>();
                destinations.sort();
                assert_eq!(destinations, vec![0, 1, 2]);
            }
        }
    }

    #[test]
    fn the_inverse_board_returns_every_card_to_its_start() {
        for origin in 0..3 {
            assert_eq!(route(origin, CYCLE_134_ON_134, INVERSE_134_ON_134)[2], origin);
        }
    }
}

#[cfg(test)]
mod repair_tests {
    use super::*;

    #[test]
    fn sentence_counts_bind_every_wait_in_the_chapter() {
        let counts = [
            ("11_1", 4),
            ("11_2", 5),
            ("11_3", 8),
            ("11_4", 7),
            ("11_5", 5),
            ("11_6", 3),
            ("11_7", 3),
            ("11_11", 5),
            ("11_8", 4),
            ("11_9", 5),
            ("11_10", 4),
        ];
        for (cue, count) in counts {
            let text = line_text(cue).expect("chapter 11 cue");
            assert_eq!(sentence_ranges(text).len(), count, "{cue}");
            // The assembler refuses a cue over 100 words.
            assert!(text.split_whitespace().count() <= 100, "{cue}");
        }
    }

    #[test]
    fn every_anchor_names_words_in_its_own_sentence() {
        for (cue, j, phrase) in ANCHORS {
            let text = line_text(cue).expect("anchored cue");
            let (from, to) = sentence_ranges(text)[j];
            assert!(text[from..to].contains(phrase), "{cue} sentence {j}: {phrase}");
        }
    }

    #[test]
    fn the_first_map_is_legal_and_the_second_collides_and_misses() {
        assert_eq!(arrivals(LEGAL_MAP), [1, 1, 1]);
        assert_eq!(arrivals(BAD_MAP)[BAD_DOUBLED], 2);
        assert_eq!(arrivals(BAD_MAP)[BAD_MISSING], 0);
    }

    #[test]
    fn cycle_notation_reads_back_and_a_loop_has_no_first_label() {
        let cycle = cycle_images::<4>(&[&[1, 3, 4]]);
        assert_eq!(cycle, [2, 1, 3, 0]);
        assert_eq!(cycle, cycle_images::<4>(&[&[3, 4, 1]]));
        assert_eq!(cycle_tex(cycle), r"(1\;3\;4)");
        assert_ne!(cycle, cycle_images::<4>(&[&[1, 4, 3]]));
        // "Usually": a two-label swap is the one cycle equal to its reverse.
        assert_eq!(cycle_images::<4>(&[&[1, 2]]), cycle_images::<4>(&[&[2, 1]]));
    }

    #[test]
    fn running_the_arrows_backwards_undoes_the_cycle() {
        let cycle = cycle_images::<4>(&[&[1, 3, 4]]);
        let inverse = cycle_images::<4>(&[&[1, 4, 3]]);
        assert_eq!(then(cycle, inverse), [0, 1, 2, 3]);
        assert_eq!(then(inverse, cycle), [0, 1, 2, 3]);
        assert_eq!(way_home(cycle), inverse);
    }

    #[test]
    fn separate_loops_commute_and_reach_the_row_on_screen() {
        let tri = cycle_images::<6>(&[&[1, 3, 5]]);
        let swap = cycle_images::<6>(&[&[2, 6]]);
        assert_eq!(then(tri, swap), then(swap, tri));
        assert_eq!(cards_by_slot(then(tri, swap)), [5, 6, 1, 4, 3, 2]);
        assert_eq!(then(tri, swap), cycle_images::<6>(&[&[2, 6], &[3, 5, 1]]));
        assert_eq!(cycle_tex(then(tri, swap)), r"(1\;3\;5)(2\;6)");
        // Label 4 is the fixed one, so "4 stays put" holds in both rows.
        assert_eq!(then(tri, swap)[3], 3);
    }

    #[test]
    fn the_overlapping_pair_of_11_3_does_not_commute() {
        let cycle = cycle_images::<3>(&[&[1, 2, 3]]);
        let swap = cycle_images::<3>(&[&[1, 2]]);
        assert_eq!(then(swap, cycle), cycle_images::<3>(&[&[1, 3]]));
        assert_eq!(then(cycle, swap), cycle_images::<3>(&[&[2, 3]]));
    }

    #[test]
    fn both_swap_descriptions_of_the_three_cycle_agree() {
        let cycle = cycle_images::<4>(&[&[1, 3, 4]]);
        let swap = |a: usize, b: usize| cycle_images::<4>(&[&[a, b]]);
        assert_eq!(then(swap(1, 3), swap(1, 4)), cycle);
        assert_eq!(then(swap(1, 4), swap(3, 4)), cycle);
    }

    #[test]
    fn the_witness_chips_show_every_case_of_the_proof() {
        let before = differences(WITNESS);
        let after = differences(after_swaps(WITNESS, &ONE_SWAP));
        assert_eq!(before, [1, 3, 7, 2, 6, 4]);
        assert_eq!(after, [-2, -3, 4, -1, 6, 7]);
        assert_eq!(delta(WITNESS), 1008);
        assert_eq!(delta(after_swaps(WITNESS, &ONE_SWAP)), -1008);
        let to = chip_destinations(before, after);
        assert_eq!(to, [3, 1, 5, 0, 4, 2]);
        assert_eq!(to[REVERSES_IN_PLACE], REVERSES_IN_PLACE);
        assert_eq!(after[REVERSES_IN_PLACE], -before[REVERSES_IN_PLACE]);
        assert_eq!(to[UNTOUCHED], UNTOUCHED);
        assert_eq!(after[UNTOUCHED], before[UNTOUCHED]);
        for (pair, reversed) in [(TRADE_AND_REVERSE, true), (TRADE_ONLY, false)] {
            let [a, b] = pair;
            assert_eq!((to[a], to[b]), (b, a));
            assert_eq!(before[a] * before[b], after[a] * after[b]);
            assert_eq!(after[b] == -before[a], reversed);
            assert_eq!(after[a] == -before[b], reversed);
        }
    }

    #[test]
    fn the_tallies_print_the_factors_on_screen() {
        assert_eq!(tally_tex(WITNESS), r"\Delta=1\cdot3\cdot7\cdot2\cdot6\cdot4=+1008");
        assert_eq!(
            tally_tex(after_swaps(WITNESS, &ONE_SWAP)),
            r"\Delta=(-2)\cdot(-3)\cdot4\cdot(-1)\cdot6\cdot7=-1008"
        );
    }

    #[test]
    fn every_swap_word_up_to_six_flips_the_sign_once_per_swap() {
        let mut words = vec![WITNESS];
        for length in 1..=6 {
            let mut next = Vec::with_capacity(words.len() * PAIRS.len());
            for &x in &words {
                for pair in PAIRS {
                    next.push(after_swaps(x, &[pair]));
                }
            }
            let expected = if length % 2 == 0 { 1008 } else { -1008 };
            assert!(next.iter().all(|&x| delta(x) == expected), "length {length}");
            words = next;
        }
    }

    #[test]
    fn both_routes_end_on_one_arrangement_and_one_product() {
        let one = route_arrangements(&ONE_SWAP);
        let three = route_arrangements(&THREE_SWAPS);
        assert_eq!(three[1..], [[2, 1, 4, 8], [2, 4, 1, 8], [4, 2, 1, 8]]);
        assert_eq!(one.last(), three.last());
        assert_eq!(
            three.iter().map(|&x| delta(x)).collect::<Vec<_>>(),
            [1008, -1008, 1008, -1008]
        );
    }

    #[test]
    fn the_route_row_lands_its_cards_where_the_arithmetic_says() {
        // The bookkeeping in `step_route`, on plain slots: which value is where.
        let mut at = [0, 1, 2, 3];
        for (a, b) in THREE_SWAPS {
            for slot in &mut at {
                if *slot == a || *slot == b {
                    *slot = if *slot == a { b } else { a };
                }
            }
        }
        let mut read = [0; 4];
        for (value, slot) in WITNESS.iter().zip(at) {
            read[slot] = *value;
        }
        assert_eq!(read, after_swaps(WITNESS, &THREE_SWAPS));
    }

    #[test]
    fn the_square_moves_read_as_the_cycles_on_screen() {
        let r = corner_images(1, false);
        let s = corner_images(0, true);
        assert_eq!(r, cycle_images::<4>(&[&[1, 4, 3, 2]]));
        assert_eq!(s, cycle_images::<4>(&[&[1, 2], &[3, 4]]));
        assert_eq!(cycle_tex(r), r"(1\;4\;3\;2)");
        assert_eq!(cycle_tex(then(r, r)), r"(1\;3)(2\;4)");
        assert_eq!(cycle_tex(then(s, r)), r"(2\;4)");
        assert_eq!(cycle_tex(then(r, s)), r"(1\;3)");
        assert_eq!(then(s, r), corner_images(1, true));
        // The same products the tile thumbnails draw.
        assert_eq!(then(s, r), corner_images(R.mul(S).turns, R.mul(S).flip));
        assert_eq!(then(r, s), corner_images(S.mul(R).turns, S.mul(R).flip));
        assert_ne!(then(s, r), then(r, s));
        let swap = |a: usize, b: usize| cycle_images::<4>(&[&[a, b]]);
        assert_eq!(then(then(swap(1, 4), swap(1, 3)), swap(1, 2)), r);
        assert_eq!(swaps_needed(r), 3);
        assert_eq!(swaps_needed(then(r, r)), 2);
        assert_eq!(parity_word(swaps_needed(r)), "odd");
        assert_eq!(parity_word(swaps_needed(then(r, r))), "even");
    }

    #[test]
    fn the_rings_follow_the_trails_the_review_names() {
        let r = corner_images(1, false);
        let s = corner_images(0, true);
        assert_eq!((s[1], r[s[1]]), (0, 3)); // rs: 2 to 1 to 4
        assert_eq!((r[0], s[r[0]]), (3, 2)); // sr: 1 to 4 to 3
    }

    #[test]
    fn twenty_four_arrangements_hold_eight_that_keep_the_edges() {
        let all = arrangements();
        assert_eq!(all.len(), 24);
        let kept: Vec<_> = all.iter().copied().filter(|&p| broken_edges(p).is_empty()).collect();
        assert_eq!(kept.len(), 8);
        let symmetric = symmetries();
        for p in &kept {
            assert!(symmetric.contains(p));
        }
        // A permutation and its inverse keep or break the square together, so
        // reading a cell as "card in each corner" highlights the right eight.
        for &p in &all {
            assert_eq!(symmetric.contains(&p), symmetric.contains(&way_home(p)));
        }
        for &a in &symmetric {
            for &b in &symmetric {
                assert!(symmetric.contains(&then(a, b)));
            }
        }
    }

    #[test]
    fn trading_neighbours_tears_exactly_the_edges_drawn_in_red() {
        assert!(broken_edges([0, 1, 2, 3]).is_empty());
        assert_eq!(broken_edges(cycle_images::<4>(&[&[1, 2]])), vec![(1, 2), (3, 0)]);
    }

    #[test]
    fn the_way_home_inverts_any_arrangement_of_corners() {
        for p in arrangements() {
            assert_eq!(then(p, way_home(p)), [0, 1, 2, 3]);
        }
    }
}
