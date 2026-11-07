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

// ---- repetition, parity and shuffles: pure arithmetic ------------------------
// Everything down to "end of pure arithmetic" uses only the standard library
// and the small helpers above, so it can be compiled and run on its own.

/// How many runs each return strip of 11_12 to 11_16 shows.
const STRIP_RUNS: usize = 12;
// The two rows of repeated loops, as zero-based labels of each loop.
const THREE_LOOP: [usize; 3] = [0, 2, 4];
const TWO_LOOP: [usize; 2] = [1, 5];
const FOUR_LOOP: [usize; 4] = [0, 1, 2, 3];
const SIDE_SWAP: [usize; 2] = [4, 5];

// 11_18 and 11_19: swap lists on four slots, performed left to right.
const EVEN_FIRST: [(usize, usize); 2] = [(0, 1), (2, 3)];
const EVEN_SECOND: [(usize, usize); 2] = [(0, 2), (1, 3)];
const SHORT_ROUTE: [(usize, usize); 2] = [(0, 3), (1, 2)];
const ONE_MORE: (usize, usize) = (0, 1);
const TWO_ODD: [(usize, usize); 2] = [(0, 1), (1, 2)];

const DECK_CARDS: usize = 10;
const FULL_DECK: usize = 52;
const WIRES: usize = 8;
/// 11_27: the two neighbouring slots that trade after the first shuffle. They
/// hold a low card and a high card, so the slip shows in the staircase.
const SLIP_SLOT: usize = 20;

fn first_loops() -> [usize; 6] {
    cycle_images::<6>(&[&[1, 3, 5], &[2, 6]])
}

fn second_loops() -> [usize; 6] {
    cycle_images::<6>(&[&[1, 2, 3, 4], &[5, 6]])
}

/// 11_3's composition, the cycle (1 2 3) after the swap (1 2).
fn overlap_result() -> [usize; 3] {
    then(cycle_images::<3>(&[&[1, 2]]), cycle_images::<3>(&[&[1, 2, 3]]))
}

/// Where each label sits after `runs` repetitions of `p`.
fn power(p: &[usize], runs: usize) -> Vec<usize> {
    let mut at: Vec<usize> = (0..p.len()).collect();
    for _ in 0..runs {
        for slot in &mut at {
            *slot = p[*slot];
        }
    }
    at
}

/// True when every listed label is in its own slot after `runs` repetitions.
fn labels_home(p: &[usize], labels: &[usize], runs: usize) -> bool {
    let at = power(p, runs);
    labels.iter().all(|&label| at[label] == label)
}

/// The runs from 1 to `limit` at which all the listed labels are home.
fn home_runs(p: &[usize], labels: &[usize], limit: usize) -> Vec<usize> {
    (1..=limit).filter(|&runs| labels_home(p, labels, runs)).collect()
}

/// The first positive number of repetitions that brings every label home,
/// found by running the permutation. The return time of the whole row.
fn first_return(p: &[usize]) -> usize {
    let every: Vec<usize> = (0..p.len()).collect();
    let mut runs = 1;
    while !labels_home(p, &every, runs) {
        runs += 1;
    }
    runs
}

/// The disjoint cycles of `p`, one-based, each from its smallest label; a
/// fixed label is a one-element cycle.
fn cycles_of(p: &[usize]) -> Vec<Vec<usize>> {
    let mut seen = vec![false; p.len()];
    let mut cycles = Vec::new();
    for start in 0..p.len() {
        let mut cycle = Vec::new();
        let mut at = start;
        while !seen[at] {
            seen[at] = true;
            cycle.push(at + 1);
            at = p[at];
        }
        if !cycle.is_empty() {
            cycles.push(cycle);
        }
    }
    cycles
}

/// The cycles that actually move something.
fn moving_cycles(p: &[usize]) -> Vec<Vec<usize>> {
    cycles_of(p).into_iter().filter(|cycle| cycle.len() > 1).collect()
}

fn moving_cycles_tex(p: &[usize]) -> String {
    moving_cycles(p)
        .iter()
        .map(|cycle| {
            let labels: Vec<String> = cycle.iter().map(|label| label.to_string()).collect();
            format!("({})", labels.join(r"\;"))
        })
        .collect()
}

fn cycle_lengths(p: &[usize]) -> Vec<usize> {
    let mut lengths: Vec<usize> = cycles_of(p).iter().map(Vec::len).collect();
    lengths.sort_unstable();
    lengths
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn lcm(a: usize, b: usize) -> usize {
    a / gcd(a, b) * b
}

/// The order predicted from the cycles, without running the permutation.
fn order_by_cycles(p: &[usize]) -> usize {
    cycles_of(p).iter().fold(1, |order, cycle| lcm(order, cycle.len()))
}

/// Where each card ends after performing a list of slot swaps in order.
fn after_word<const N: usize>(word: &[(usize, usize)]) -> [usize; N] {
    let mut at: [usize; N] = std::array::from_fn(|card| card);
    for &(a, b) in word {
        for slot in &mut at {
            if *slot == a {
                *slot = b;
            } else if *slot == b {
                *slot = a;
            }
        }
    }
    at
}

/// Parity as it adds: 0 is even and 1 is odd.
fn parity_sum(first: usize, second: usize) -> usize {
    (first + second) % 2
}

fn even_arrangements() -> Vec<[usize; 4]> {
    arrangements()
        .into_iter()
        .filter(|&p| swaps_needed(p).is_multiple_of(2))
        .collect()
}

/// 11_21's pairing: one extra swap of the first two cards.
fn with_first_two_swapped(p: [usize; 4]) -> [usize; 4] {
    [p[1], p[0], p[2], p[3]]
}

/// Among even arrangements: 0 is doing nothing, 1 a double swap, 2 a three-cycle.
fn kind_of(p: [usize; 4]) -> usize {
    let moved = (0..4).filter(|&slot| p[slot] != slot).count();
    match moved {
        0 => 0,
        4 => 1,
        _ => 2,
    }
}

fn even_kind_counts() -> [usize; 3] {
    let mut counts = [0; 3];
    for p in even_arrangements() {
        counts[kind_of(p)] += 1;
    }
    counts
}

/// A perfect shuffle of an even number of cards. The top half lands in every
/// second position: from the first position when `keep_top` holds (an
/// out-shuffle), and from the second otherwise (an in-shuffle).
fn perfect_shuffle(cards: usize, keep_top: bool) -> Vec<usize> {
    let half = cards / 2;
    let (top_from, bottom_from) = if keep_top { (0, 1) } else { (1, 0) };
    (0..cards)
        .map(|slot| {
            if slot < half {
                2 * slot + top_from
            } else {
                2 * (slot - half) + bottom_from
            }
        })
        .collect()
}

/// `runs` repetitions of `p`, with slots `slot` and `slot + 1` trading places
/// right after the first: where each card ends.
fn slipped_run(p: &[usize], slot: usize, runs: usize) -> Vec<usize> {
    let mut at = p.to_vec();
    for place in &mut at {
        if *place == slot {
            *place = slot + 1;
        } else if *place == slot + 1 {
            *place = slot;
        }
    }
    for _ in 1..runs {
        for place in &mut at {
            *place = p[*place];
        }
    }
    at
}

/// The cards (one-based) that are not in their own slots.
fn out_of_place(at: &[usize]) -> Vec<usize> {
    at.iter()
        .enumerate()
        .filter(|&(card, &slot)| card != slot)
        .map(|(card, _)| card + 1)
        .collect()
}

fn parity_of(at: &[usize]) -> &'static str {
    parity_word(at.len() - cycles_of(at).len())
}

/// `n!` as (leading digit, power of ten), by logarithms so nothing overflows.
fn factorial_scientific(n: u32) -> (u32, u32) {
    let log: f64 = (2..=n).map(|k| f64::from(k).log10()).sum();
    let power_of_ten = log.floor();
    (10f64.powf(log - power_of_ten).floor() as u32, power_of_ten as u32)
}

fn listed_cycles(cycles: &[Vec<usize>]) -> String {
    let written: Vec<String> = cycles
        .iter()
        .map(|cycle| {
            cycle
                .iter()
                .map(|label| label.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    written.join(";")
}

/// Every number the cues from 11_12 onward put on screen or say.
fn repetition_facts() -> Vec<(&'static str, String)> {
    let first = first_loops();
    let second = second_loops();
    let overlap = overlap_result();
    let deck = perfect_shuffle(DECK_CARDS, true);
    let full = perfect_shuffle(FULL_DECK, true);
    let other = perfect_shuffle(FULL_DECK, false);
    let wires = perfect_shuffle(WIRES, true);
    let evens = even_arrangements();
    let odd_count = arrangements().len() - evens.len();
    let partners: Vec<String> = evens
        .iter()
        .map(|&p| format!("{}>{}", cell_text(p), cell_text(with_first_two_swapped(p))))
        .collect();
    let (digit, power_of_ten) = factorial_scientific(52);
    let slipped = slipped_run(&full, SLIP_SLOT, first_return(&full));
    let even_word = [EVEN_FIRST, EVEN_SECOND].concat();
    let longer_word = [even_word.as_slice(), &[ONE_MORE]].concat();
    let every_card: Vec<usize> = (0..FULL_DECK).collect();
    vec![
        (
            "permutations.loop_three_home",
            crate::listed(home_runs(&first, &THREE_LOOP, STRIP_RUNS)),
        ),
        (
            "permutations.loop_pair_home",
            crate::listed(home_runs(&first, &TWO_LOOP, STRIP_RUNS)),
        ),
        ("permutations.loops_order", first_return(&first).to_string()),
        (
            "permutations.second_four_home",
            crate::listed(home_runs(&second, &FOUR_LOOP, STRIP_RUNS)),
        ),
        (
            "permutations.second_pair_home",
            crate::listed(home_runs(&second, &SIDE_SWAP, STRIP_RUNS)),
        ),
        ("permutations.second_order", first_return(&second).to_string()),
        ("permutations.overlap_images", one_based(&overlap)),
        ("permutations.overlap_order", first_return(&overlap).to_string()),
        (
            "permutations.even_lists_final",
            crate::listed(cards_by_slot(after_word::<4>(&even_word))),
        ),
        (
            "permutations.short_route_final",
            crate::listed(cards_by_slot(after_word::<4>(&SHORT_ROUTE))),
        ),
        (
            "permutations.one_more_final",
            crate::listed(cards_by_slot(after_word::<4>(&longer_word))),
        ),
        (
            "permutations.two_odd_final",
            crate::listed(cards_by_slot(after_word::<4>(&TWO_ODD))),
        ),
        ("permutations.even_on_four", evens.len().to_string()),
        ("permutations.odd_on_four", odd_count.to_string()),
        ("permutations.even_kinds", crate::listed(even_kind_counts())),
        ("permutations.even_partners", partners.join(",")),
        ("permutations.deck_images", one_based(&deck)),
        ("permutations.deck_cycles", listed_cycles(&moving_cycles(&deck))),
        ("permutations.deck_order", first_return(&deck).to_string()),
        (
            "permutations.deck_away_after_two",
            out_of_place(&power(&deck, 2)).len().to_string(),
        ),
        ("permutations.full_out_lengths", crate::listed(cycle_lengths(&full))),
        ("permutations.full_out_order", first_return(&full).to_string()),
        ("permutations.full_in_lengths", crate::listed(cycle_lengths(&other))),
        ("permutations.full_in_order", first_return(&other).to_string()),
        (
            "permutations.full_in_home_after_eight",
            (FULL_DECK - out_of_place(&power(&other, first_return(&full))).len()).to_string(),
        ),
        (
            "permutations.full_arrangements_scientific",
            format!("{digit},{power_of_ten}"),
        ),
        ("permutations.wire_images", one_based(&wires)),
        ("permutations.wire_cycles", listed_cycles(&moving_cycles(&wires))),
        ("permutations.wire_order", first_return(&wires).to_string()),
        ("permutations.slip_cards", crate::listed(out_of_place(&slipped))),
        ("permutations.slip_parity", parity_of(&slipped).to_string()),
        ("permutations.undo_by_more", (order_by_cycles(&full) - 1).to_string()),
        (
            "permutations.full_home_labels",
            home_runs(&full, &every_card, 8).len().to_string(),
        ),
    ]
}

// ---- end of pure arithmetic --------------------------------------------------

pub(crate) fn add_facts(facts: &mut Facts) -> Result<(), String> {
    for (key, value) in repetition_facts() {
        facts.add(key, value)?;
    }
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
const THEY_COMPOSE: Anchor = ("11_20", 0, "since they compose");
const NOT_EVERY: Anchor = ("11_9", 0, "but not every");
const OUR_EIGHT: Anchor = ("11_9", 1, "and our eight");
const AS_SUBGROUP: Anchor = ("11_9", 1, "as a subgroup");
const HALF_TURN: Anchor = ("11_9", 2, "a half-turn");
const RS_SWAP: Anchor = ("11_9", 3, "r s is the swap");
const SR_SWAP: Anchor = ("11_9", 3, "and s r is the swap");
const INTERLEAVE: Anchor = ("11_10", 3, "then interleave");
const STILL_OUT: Anchor = ("11_23", 1, "yet most");

#[cfg(test)]
const ANCHORS: [Anchor; 17] = [
    NO_COLLISIONS,
    NO_MISSING,
    THEY_COMMUTE,
    TWO_INPUTS,
    ALL_DIFFERENCES,
    THE_PAIRS,
    BOTH_REVERSE,
    SAME_PRODUCT,
    THEY_COMPOSE,
    NOT_EVERY,
    OUR_EIGHT,
    AS_SUBGROUP,
    HALF_TURN,
    RS_SWAP,
    SR_SWAP,
    INTERLEAVE,
    STILL_OUT,
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

/// Old captions out, then new in, for words on the same spot: a crossfade prints one over the other.
fn swap_caption(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob]) -> fmn::Result<()> {
    fade(stage, old, false)?;
    fade(stage, new, true)
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
        // Scale the family without repainting it: a filled card's dark numeral
        // must stay distinct from its coloured disk throughout the emphasis.
        let a = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                rate_func: Some(fmn::core::rate::there_and_back),
                ..AnimateArgs::default()
            })?
            .scale(1.25)?;
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
/// How far a link's head or tail stands from a slot's centre in a cycle longer than a swap.
const LINK_SEP: f64 = 0.16;

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

fn shifted((x, y): (f64, f64), dx: f64) -> (f64, f64) {
    (x + dx, y)
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
    painted_row(stage, k, y, name, loop_color)
}

/// Six cards in their numbered slots, each coloured by its loop.
fn painted_row(stage: &mut Stage<'_>, k: &Kit, y: f64, name: &str, paint: fn(usize) -> Srgb) -> fmn::Result<SixRow> {
    let mut rest = vec![k.text(stage, name, 24.0, Palette::dim(), v(-5.3, y))?];
    let mut cards = Vec::new();
    for slot in 0..6 {
        let at = row_slot(slot, y);
        rest.push(ring(stage, at, 0.31, Palette::dim(), 2.0));
        rest.push(k.text(stage, &(slot + 1).to_string(), 23.0, Palette::dim(), v(at.0, y + 0.46))?);
        cards.push(token(stage, k, &(slot + 1).to_string(), paint(slot + 1), at, 0.27)?);
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
        // Lower-row numbers sit under their rings: the link from slot 4 to slot 1 runs up through the space above.
        let label_y = if at.1 < 0.0 { at.1 - 0.62 } else { at.1 + 0.62 };
        board.push(k.text(stage, &(index + 1).to_string(), 23.0, Palette::dim(), v(at.0, label_y))?);
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
    // A head lands left of a slot's centre and a tail leaves right of it, so only the swap is double-headed.
    let tail = |slot| shifted(above_top(slot), LINK_SEP);
    let head = |slot| shifted(above_top(slot), -LINK_SEP);
    on_screen.push(curved(stage, tail(0), head(2), -LOOP_BEND, Palette::hero())?);
    on_screen.push(curved(stage, tail(2), head(4), -LOOP_BEND, Palette::hero())?);
    on_screen.push(curved(stage, below_top(1), below_top(5), SWAP_BEND, Palette::blue())?);
    on_screen.push(curved(stage, below_top(5), below_top(1), -SWAP_BEND, Palette::blue())?);
    let closing = curved(stage, tail(4), head(0), LOOP_BEND, Palette::hero())?;
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
        "Disjoint cycles commute; overlapping cycles may not.",
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
    let walker = halo(stage, row_slot(0, ROW_TOP), 0.37, Palette::ink(), 3.0);
    fade(stage, &[walker], true)?;
    travel(stage, &[(walker, row_slot(2, ROW_TOP))], -LOOP_BEND, 0.85)?;
    travel(stage, &[(walker, row_slot(4, ROW_TOP))], -LOOP_BEND, 0.85)?;
    travel(stage, &[(walker, row_slot(0, ROW_TOP))], LOOP_BEND, 0.85)?;
    fade(stage, &[walker], false)?;

    // Sentence 4: the impossible arrow. Slot 3 would have two inputs.
    k.until_sentence(stage, "11_4", 4)?;
    fade(stage, &[contrast, closing], false)?;
    // Its head lands on the head of the link from slot 1: two inputs at one point.
    let wrong = curved(stage, tail(4), head(2), WRONG_BEND, Palette::red())?;
    fade(stage, &[wrong], true)?;
    until_phrase(stage, k, TWO_INPUTS)?;
    let clash = halo(stage, row_slot(2, ROW_TOP), 0.37, Palette::red(), 4.0);
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
    let closing = curved(stage, tail(4), head(0), LOOP_BEND, Palette::hero())?;
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

    // Sentence 4: two runs one after the other, so the counts add.
    k.until_sentence(stage, "11_8", 4)?;
    let adding = k.tex(
        stage,
        r"(-1)^{N_1+N_2}=(-1)^{N_1}(-1)^{N_2}",
        30.0,
        Palette::gold(),
        &[],
        v(0.0, -2.95),
    )?;
    swap_caption(stage, &[swap_count], &[adding])?;

    let mut on_screen = one.mobs();
    on_screen.extend(three.mobs());
    on_screen.extend([frames[0], frames[1], nonzero, parity, adding]);
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
    stage.wait(0.35)?;
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
    stage.wait(0.35)?;
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

// ---- pieces shared by the repetition, parity and shuffle cues ----------------

/// A numbered count that changes with what the cards just did.
struct Counter {
    mob: Mob,
    label: &'static str,
    value: usize,
    at: (f64, f64),
}

fn counter_text(stage: &mut Stage<'_>, k: &Kit, label: &str, value: usize, at: (f64, f64)) -> fmn::Result<Mob> {
    k.text(stage, &format!("{label}: {value}"), 26.0, Palette::ink(), v(at.0, at.1))
}

fn new_counter(stage: &mut Stage<'_>, k: &Kit, label: &'static str, at: (f64, f64)) -> fmn::Result<Counter> {
    let mob = counter_text(stage, k, label, 0, at)?;
    Ok(Counter {
        mob,
        label,
        value: 0,
        at,
    })
}

/// Count one more, bringing in anything that should appear with the new value.
fn advance(stage: &mut Stage<'_>, k: &Kit, counter: &mut Counter, also: &[Mob]) -> fmn::Result<()> {
    counter.value += 1;
    let fresh = counter_text(stage, k, counter.label, counter.value, counter.at)?;
    let mut appearing = vec![fresh];
    appearing.extend(also);
    exchange(stage, &[counter.mob], &appearing)?;
    counter.mob = fresh;
    Ok(())
}

/// Numbered cards in numbered slots; `slot` places either by zero-based slot.
struct CardRow {
    cards: Vec<Mob>,
    at: Vec<usize>,
    rest: Vec<Mob>,
    slot: fn(usize) -> (f64, f64),
}

impl CardRow {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.rest.clone();
        all.extend(&self.cards);
        all
    }
}

fn card_row(
    stage: &mut Stage<'_>,
    k: &Kit,
    count: usize,
    slot: fn(usize) -> (f64, f64),
    paint: fn(usize) -> Srgb,
) -> fmn::Result<CardRow> {
    let mut rest = Vec::new();
    let mut cards = Vec::new();
    for index in 0..count {
        let at = slot(index);
        let label = (index + 1).to_string();
        rest.push(ring(stage, at, 0.31, Palette::dim(), 2.0));
        rest.push(k.text(stage, &label, 22.0, Palette::dim(), v(at.0, at.1 + 0.46))?);
        cards.push(token(stage, k, &label, paint(index + 1), at, 0.27)?);
    }
    Ok(CardRow {
        cards,
        at: (0..count).collect(),
        rest,
        slot,
    })
}

/// The cards in two slots change places, passing on opposite sides.
fn trade_slots(stage: &mut Stage<'_>, row: &mut CardRow, (a, b): (usize, usize), run_time: f64) -> fmn::Result<()> {
    let mut moves = Vec::new();
    for (&card, slot) in row.cards.iter().zip(row.at.iter_mut()) {
        if *slot == a {
            *slot = b;
            moves.push((card, (row.slot)(b)));
        } else if *slot == b {
            *slot = a;
            moves.push((card, (row.slot)(a)));
        }
    }
    travel(stage, &moves, SWAP_ARC, run_time)
}

/// Perform a list of swaps, counting each one.
fn run_word(
    stage: &mut Stage<'_>,
    k: &Kit,
    row: &mut CardRow,
    counter: &mut Counter,
    word: &[(usize, usize)],
) -> fmn::Result<()> {
    for &swap in word {
        trade_slots(stage, row, swap, 0.55)?;
        advance(stage, k, counter, &[])?;
    }
    Ok(())
}

/// One arrow per link of a cycle, bulging above or below the row.
fn loop_arrows(
    stage: &mut Stage<'_>,
    cycle: &[usize],
    anchor: fn(usize) -> (f64, f64),
    above: bool,
    color: Srgb,
) -> fmn::Result<Vec<Mob>> {
    let offsets = link_offsets(cycle);
    let mut arrows = Vec::new();
    for (index, &label) in cycle.iter().enumerate() {
        let next = cycle[(index + 1) % cycle.len()];
        let (tail, head) = offsets[index];
        arrows.push(curved(
            stage,
            shifted(anchor(label - 1), tail),
            shifted(anchor(next - 1), head),
            link_bend(label, next, above),
            color,
        )?);
    }
    Ok(arrows)
}

/// Each link's (tail, head) offset along the row. In a cycle longer than a swap, a slot's
/// tail and head stand apart, on the sides that keep its two arcs from crossing each other
/// there, so no link reads as double-headed. A swap shares its anchors: it is one
/// double-headed arc, the only one in the picture.
fn link_offsets(cycle: &[usize]) -> Vec<(f64, f64)> {
    let n = cycle.len();
    if n < 3 {
        return vec![(0.0, 0.0); n];
    }
    let tail_right = |index: usize| {
        let at = cycle[index] as f64;
        let from = cycle[(index + n - 1) % n] as f64;
        let to = cycle[(index + 1) % n] as f64;
        1.0 / (to - at) > 1.0 / (from - at)
    };
    (0..n)
        .map(|index| {
            let tail = if tail_right(index) { LINK_SEP } else { -LINK_SEP };
            let head = if tail_right((index + 1) % n) {
                -LINK_SEP
            } else {
                LINK_SEP
            };
            (tail, head)
        })
        .collect()
}

/// The bend that keeps a link on one side of its row, whichever way it runs.
fn link_bend(from: usize, to: usize, above: bool) -> f64 {
    if above == (to > from) { -LOOP_BEND } else { LOOP_BEND }
}

/// One loop's record: a numbered cell per run, lit when the loop is home.
struct ReturnStrip {
    y: f64,
    color: Srgb,
    labels: Vec<usize>,
    rest: Vec<Mob>,
    lamps: Vec<Mob>,
}

impl ReturnStrip {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.rest.clone();
        all.extend(&self.lamps);
        all
    }
}

const STRIP_LEFT: f64 = -2.9;
const STRIP_STEP: f64 = 0.62;

fn strip_x(run: usize) -> f64 {
    STRIP_LEFT + STRIP_STEP * (run - 1) as f64
}

fn return_strip(
    stage: &mut Stage<'_>,
    k: &Kit,
    y: f64,
    name: &str,
    color: Srgb,
    labels: &[usize],
    runs: usize,
) -> fmn::Result<ReturnStrip> {
    let mut rest = vec![k.tex(stage, name, 26.0, color, &[], v(STRIP_LEFT - 1.05, y))?];
    for run in 1..=runs {
        let cell = Rectangle::new()
            .width(0.54)
            .height(0.42)
            .corner_radius(0.06)
            .style(stroke_style(Palette::dim(), 2.0))
            .build()?
            .moved_to(v(strip_x(run), y));
        rest.push(stage.arena_mut().add(cell));
        let number = k.text(stage, &run.to_string(), 20.0, Palette::ink(), v(strip_x(run), y))?;
        stage.arena_mut().set_z_index(number, 2, true);
        rest.push(number);
    }
    Ok(ReturnStrip {
        y,
        color,
        labels: labels.iter().map(|label| label - 1).collect(),
        rest,
        lamps: Vec::new(),
    })
}

fn strip_lamp(stage: &mut Stage<'_>, run: usize, y: f64, color: Srgb) -> fmn::Result<Mob> {
    let lamp = Rectangle::new()
        .width(0.54)
        .height(0.42)
        .corner_radius(0.06)
        .style(Style::default().fill(color, 0.4).stroke(color, 3.0, 1.0))
        .build()?
        .moved_to(v(strip_x(run), y));
    Ok(stage.arena_mut().add(lamp))
}

/// Lamps for every strip whose loop is home at this run; the caller shows them.
fn lamps_for(stage: &mut Stage<'_>, strips: &mut [ReturnStrip], p: &[usize], run: usize) -> fmn::Result<Vec<Mob>> {
    let mut lamps = Vec::new();
    for strip in strips.iter_mut() {
        if labels_home(p, &strip.labels, run) {
            let lamp = strip_lamp(stage, run, strip.y, strip.color)?;
            strip.lamps.push(lamp);
            lamps.push(lamp);
        }
    }
    Ok(lamps)
}

/// The later runs of one strip, lit without running the cards.
fn light_later(stage: &mut Stage<'_>, strip: &mut ReturnStrip, p: &[usize], from: usize) -> fmn::Result<()> {
    let mut lamps = Vec::new();
    for run in from..=STRIP_RUNS {
        if labels_home(p, &strip.labels, run) {
            let lamp = strip_lamp(stage, run, strip.y, strip.color)?;
            strip.lamps.push(lamp);
            lamps.push(lamp);
        }
    }
    fade(stage, &lamps, true)
}

/// A frame around one run in both strips.
fn column_frame(stage: &mut Stage<'_>, run: usize, top: f64, low: f64, color: Srgb) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(0.6)
        .height(top - low + 0.56)
        .corner_radius(0.08)
        .style(stroke_style(color, 4.0))
        .build()?
        .moved_to(v(strip_x(run), (top + low) / 2.0));
    Ok(stage.arena_mut().add(frame))
}

// ---- 11_12 to 11_17: repeating a permutation --------------------------------

const ORDER_ROW_Y: f64 = 1.0;
const STRIP_TOP: f64 = -0.75;
const STRIP_LOW: f64 = -1.45;
const ORDER_CAPTION_Y: f64 = -2.3;
const ORDER_QUESTION_Y: f64 = -3.0;

fn order_above(slot: usize) -> (f64, f64) {
    let (x, y) = row_slot(slot, ORDER_ROW_Y);
    (x, y + 0.66)
}

fn order_below(slot: usize) -> (f64, f64) {
    let (x, y) = row_slot(slot, ORDER_ROW_Y);
    (x, y - 0.42)
}

fn second_color(label: usize) -> Srgb {
    if label <= 4 { Palette::hero() } else { Palette::blue() }
}

/// The row, its loops, and one return strip per loop.
struct LoopBoard {
    row: SixRow,
    p: [usize; 6],
    strips: [ReturnStrip; 2],
    arrows: Vec<Mob>,
    count: Counter,
    title: Mob,
    frames: Vec<Mob>,
    notes: Vec<Mob>,
}

impl LoopBoard {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.row.mobs();
        all.extend(self.strips[0].mobs());
        all.extend(self.strips[1].mobs());
        all.extend(&self.arrows);
        all.extend(&self.frames);
        all.extend(&self.notes);
        all.extend([self.count.mob, self.title]);
        all
    }

    fn strip_mobs(&self) -> Vec<Mob> {
        let mut all = self.strips[0].mobs();
        all.extend(self.strips[1].mobs());
        all.push(self.count.mob);
        all
    }
}

/// Everything above the strips appears; the strips wait for their sentence.
fn loop_board(
    stage: &mut Stage<'_>,
    k: &Kit,
    p: [usize; 6],
    name: &str,
    paint: fn(usize) -> Srgb,
    above: (&str, &[usize], Srgb),
    below: (&str, &[usize], Srgb),
) -> fmn::Result<LoopBoard> {
    let title = k.text(
        stage,
        "Repeat it until every card is home",
        32.0,
        Palette::ink(),
        v(0.0, 3.05),
    )?;
    let row = painted_row(stage, k, ORDER_ROW_Y, name, paint)?;
    let mut arrows = loop_arrows(stage, above.1, order_above, true, above.2)?;
    arrows.extend(loop_arrows(stage, below.1, order_below, false, below.2)?);
    let mut showing = row.mobs();
    showing.extend(&arrows);
    showing.push(title);
    fade(stage, &showing, true)?;
    let top = return_strip(stage, k, STRIP_TOP, above.0, above.2, above.1, STRIP_RUNS)?;
    let low = return_strip(stage, k, STRIP_LOW, below.0, below.2, below.1, STRIP_RUNS)?;
    let count = new_counter(stage, k, "Runs", (5.3, -0.75))?;
    Ok(LoopBoard {
        row,
        p,
        strips: [top, low],
        arrows,
        count,
        title,
        frames: Vec::new(),
        notes: Vec::new(),
    })
}

/// One repetition: the cards move, the count rises, any loop now home lights.
fn run_board(stage: &mut Stage<'_>, k: &Kit, board: &mut LoopBoard) -> fmn::Result<()> {
    let moves = board.row.moves(board.p);
    travel(stage, &moves, CARD_ARC, 0.75)?;
    let run = board.count.value + 1;
    let lamps = lamps_for(stage, &mut board.strips, &board.p, run)?;
    advance(stage, k, &mut board.count, &lamps)
}

fn clear_notes(stage: &mut Stage<'_>, notes: &mut Vec<Mob>) -> fmn::Result<()> {
    let leaving = std::mem::take(notes);
    fade(stage, &leaving, false)
}

fn loops_question(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<LoopBoard> {
    let mut board = loop_board(
        stage,
        k,
        first_loops(),
        "(1 3 5)(2 6)",
        loop_color,
        (r"(1\;3\;5)", &[1, 3, 5], Palette::hero()),
        (r"(2\;6)", &[2, 6], Palette::blue()),
    )?;
    k.until_sentence(stage, "11_12", 1)?;
    let strips = board.strip_mobs();
    fade(stage, &strips, true)?;

    // Sentence 2: the word from the clock chapter.
    k.until_sentence(stage, "11_12", 2)?;
    let clock = k.text(
        stage,
        "On the clock: first return = order",
        27.0,
        Palette::dim(),
        v(0.0, ORDER_CAPTION_Y),
    )?;
    fade(stage, &[clock], true)?;

    // Sentence 3: what "return" has to mean for a whole row of cards.
    k.until_sentence(stage, "11_12", 3)?;
    let strict = k.text(
        stage,
        "Here: every card in its own slot, at the same moment",
        27.0,
        Palette::gold(),
        v(0.0, ORDER_CAPTION_Y),
    )?;
    let mut rings = Vec::new();
    for slot in 0..6 {
        rings.push(halo(stage, row_slot(slot, ORDER_ROW_Y), 0.37, Palette::gold(), 3.0));
    }
    let mut appearing = rings.clone();
    appearing.push(strict);
    swap_caption(stage, &[clock], &appearing)?;
    stage.wait(0.6)?;
    fade(stage, &rings, false)?;

    k.until_sentence(stage, "11_12", 4)?;
    let question = k.text(
        stage,
        "How many runs? Predict first.",
        31.0,
        Palette::blue(),
        v(0.0, ORDER_QUESTION_Y),
    )?;
    fade(stage, &[question], true)?;
    board.notes.extend([strict, question]);
    Ok(board)
}

fn loops_run(stage: &mut Stage<'_>, k: &Kit, board: &mut LoopBoard) -> fmn::Result<()> {
    clear_notes(stage, &mut board.notes)?;
    run_board(stage, k, board)?;
    k.until_sentence(stage, "11_13", 1)?;
    run_board(stage, k, board)?;
    k.until_sentence(stage, "11_13", 2)?;
    run_board(stage, k, board)?;
    k.until_sentence(stage, "11_13", 3)?;
    run_board(stage, k, board)?;
    run_board(stage, k, board)?;
    k.until_sentence(stage, "11_13", 4)?;
    run_board(stage, k, board)?;
    let both = column_frame(stage, 6, STRIP_TOP, STRIP_LOW, Palette::green())?;
    let home = k.text(
        stage,
        "Both loops home, for the first time",
        27.0,
        Palette::green(),
        v(0.0, ORDER_CAPTION_Y),
    )?;
    fade(stage, &[both, home], true)?;
    board.frames.push(both);
    board.notes.push(home);
    k.until_sentence(stage, "11_13", 5)?;
    let order = k.tex(
        stage,
        &format!(r"\text{{order}}={}", first_return(&board.p)),
        34.0,
        Palette::green(),
        &[],
        v(0.0, ORDER_QUESTION_Y),
    )?;
    fade(stage, &[order], true)?;
    board.notes.push(order);
    Ok(())
}

fn loops_why(stage: &mut Stage<'_>, k: &Kit, board: &mut LoopBoard) -> fmn::Result<()> {
    // Sentences 1 and 2: each strip lights its own multiples, past the run so far.
    let from = board.count.value + 1;
    k.until_sentence(stage, "11_14", 1)?;
    light_later(stage, &mut board.strips[0], &board.p, from)?;
    k.until_sentence(stage, "11_14", 2)?;
    light_later(stage, &mut board.strips[1], &board.p, from)?;

    // Sentence 3: numbers on both lists, and the first of them.
    k.until_sentence(stage, "11_14", 3)?;
    let later = column_frame(stage, 12, STRIP_TOP, STRIP_LOW, Palette::dim())?;
    let first = k.tex(
        stage,
        &format!(r"\text{{lcm}}(3,2)={}", lcm(3, 2)),
        34.0,
        Palette::green(),
        &[],
        v(0.0, ORDER_CAPTION_Y),
    )?;
    let leaving = std::mem::take(&mut board.notes);
    swap_caption(stage, &leaving, &[later, first])?;
    board.frames.push(later);
    board.notes.push(first);

    // Sentence 4: a fixed card is home on every run.
    k.until_sentence(stage, "11_14", 4)?;
    let fixed = k.text(
        stage,
        "Card 4 never moves: no condition",
        27.0,
        Palette::dim(),
        v(0.0, ORDER_QUESTION_Y),
    )?;
    fade(stage, &[fixed], true)?;
    pulse(stage, &[board.row.cards[3]])?;

    // Sentence 5: the tempting sum fails on the strips themselves.
    k.until_sentence(stage, "11_14", 5)?;
    let five = column_frame(stage, 3 + 2, STRIP_TOP, STRIP_LOW, Palette::red())?;
    let sum = k.text(
        stage,
        "3 + 2 = 5: neither loop is home",
        27.0,
        Palette::red(),
        v(0.0, ORDER_QUESTION_Y),
    )?;
    swap_caption(stage, &[fixed], &[five, sum])?;
    board.frames.push(five);
    board.notes.push(sum);
    Ok(())
}

fn second_question(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<LoopBoard> {
    let mut board = loop_board(
        stage,
        k,
        second_loops(),
        "(1 2 3 4)(5 6)",
        second_color,
        (r"(1\;2\;3\;4)", &[1, 2, 3, 4], Palette::hero()),
        (r"(5\;6)", &[5, 6], Palette::blue()),
    )?;
    let strips = board.strip_mobs();
    fade(stage, &strips, true)?;
    k.until_sentence(stage, "11_15", 1)?;
    let product = k.text(
        stage,
        &format!("4 × 2 = {}?", 4 * 2),
        28.0,
        Palette::gold(),
        v(-2.2, ORDER_CAPTION_Y),
    )?;
    fade(stage, &[product], true)?;
    k.until_sentence(stage, "11_15", 2)?;
    let sum = k.text(
        stage,
        &format!("4 + 2 = {}?", 4 + 2),
        28.0,
        Palette::gold(),
        v(2.2, ORDER_CAPTION_Y),
    )?;
    fade(stage, &[sum], true)?;
    k.until_sentence(stage, "11_15", 3)?;
    let question = k.text(
        stage,
        "Work out the first return",
        31.0,
        Palette::blue(),
        v(0.0, ORDER_QUESTION_Y),
    )?;
    fade(stage, &[question], true)?;
    board.notes.extend([product, sum, question]);
    Ok(board)
}

fn second_run(stage: &mut Stage<'_>, k: &Kit, board: &mut LoopBoard) -> fmn::Result<()> {
    clear_notes(stage, &mut board.notes)?;
    run_board(stage, k, board)?;
    run_board(stage, k, board)?;
    k.until_sentence(stage, "11_16", 1)?;
    run_board(stage, k, board)?;
    run_board(stage, k, board)?;

    k.until_sentence(stage, "11_16", 2)?;
    let first = column_frame(stage, 4, STRIP_TOP, STRIP_LOW, Palette::green())?;
    let order = k.tex(
        stage,
        &format!(r"\text{{lcm}}(4,2)={}", first_return(&board.p)),
        34.0,
        Palette::green(),
        &[],
        v(0.0, ORDER_CAPTION_Y),
    )?;
    fade(stage, &[first, order], true)?;
    board.frames.push(first);
    board.notes.push(order);

    // Sentence 3: the rest of each strip shows what 8 and 6 really are.
    k.until_sentence(stage, "11_16", 3)?;
    let from = board.count.value + 1;
    light_later(stage, &mut board.strips[0], &board.p, from)?;
    light_later(stage, &mut board.strips[1], &board.p, from)?;
    let eight = column_frame(stage, 8, STRIP_TOP, STRIP_LOW, Palette::dim())?;
    let six = column_frame(stage, 6, STRIP_TOP, STRIP_LOW, Palette::red())?;
    let verdict = k.text(
        stage,
        "8 is a later return. 6 is no return.",
        27.0,
        Palette::ink(),
        v(0.0, ORDER_QUESTION_Y),
    )?;
    fade(stage, &[eight, six, verdict], true)?;
    board.frames.extend([eight, six]);
    board.notes.push(verdict);
    Ok(())
}

/// 11_17: the rule fails on overlapping cycles, on the film's own example.
fn overlap_warning(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let p = overlap_result();
    let spot = |slot: usize| (-2.2 + 2.2 * slot as f64, 1.0);
    let title = k.text(stage, "The loops must be separate", 32.0, Palette::ink(), v(0.0, 3.05))?;
    let mut on_screen = vec![title];
    let mut cards = Vec::new();
    for slot in 0..3 {
        let at = spot(slot);
        on_screen.push(ring(stage, at, 0.44, Palette::dim(), 2.0));
        on_screen.push(k.text(
            stage,
            &(slot + 1).to_string(),
            23.0,
            Palette::dim(),
            v(at.0, at.1 + 0.62),
        )?);
        cards.push(card(stage, k, slot + 1, at, 0.37)?);
    }
    on_screen.extend(&cards);
    fade(stage, &on_screen, true)?;

    k.until_sentence(stage, "11_17", 1)?;
    let composition = k.tex(
        stage,
        &format!(r"(1\;2\;3)(1\;2)={}", cycle_tex(p)),
        34.0,
        Palette::ink(),
        &[],
        v(0.0, -0.6),
    )?;
    fade(stage, &[composition], true)?;
    on_screen.push(composition);

    // Sentence 2: the tempting answer, then the cards themselves.
    k.until_sentence(stage, "11_17", 2)?;
    let tempting = k.tex(
        stage,
        &format!(r"\text{{lcm}}(3,2)={}?", lcm(3, 2)),
        30.0,
        Palette::red(),
        &[],
        v(-2.6, -1.45),
    )?;
    let mut count = new_counter(stage, k, "Runs", (2.6, -1.45))?;
    fade(stage, &[tempting, count.mob], true)?;
    let mut at = [0, 1, 2];
    for _ in 0..first_return(&p) {
        let mut moves = Vec::new();
        for (index, slot) in at.iter_mut().enumerate() {
            if p[*slot] != *slot {
                *slot = p[*slot];
                moves.push((cards[index], spot(*slot)));
            }
        }
        travel(stage, &moves, SWAP_ARC, 0.7)?;
        advance(stage, k, &mut count, &[])?;
    }
    let actual = k.text(
        stage,
        &format!("First return: {} (it also returns at {})", first_return(&p), lcm(3, 2)),
        28.0,
        Palette::green(),
        v(0.0, -2.2),
    )?;
    fade(stage, &[actual], true)?;
    on_screen.extend([tempting, actual, count.mob]);

    k.until_sentence(stage, "11_17", 3)?;
    let interfere = k.text(
        stage,
        "Overlapping cycles interfere",
        27.0,
        Palette::red(),
        v(0.0, -2.9),
    )?;
    fade(stage, &[interfere], true)?;

    k.until_sentence(stage, "11_17", 4)?;
    let rule = k.text(
        stage,
        "Order = lcm of the lengths of the disjoint cycles",
        28.0,
        Palette::green(),
        v(0.0, -2.9),
    )?;
    swap_caption(stage, &[interfere], &[rule])?;
    on_screen.push(rule);
    Ok(on_screen)
}

// ---- 11_18 to 11_21: swap counts add, and the even permutations -------------

const SIGN_TOP_Y: f64 = 1.0;
const SIGN_LOW_Y: f64 = -0.7;
const SIGN_NOTES_X: f64 = 3.6;

fn sign_top(slot: usize) -> (f64, f64) {
    (-5.2 + 1.2 * slot as f64, SIGN_TOP_Y)
}

fn sign_low(slot: usize) -> (f64, f64) {
    (-5.2 + 1.2 * slot as f64, SIGN_LOW_Y)
}

fn corner_paint(label: usize) -> Srgb {
    d4g::corner_color(label - 1)
}

fn parity_color(parity: usize) -> Srgb {
    if parity == 0 { Palette::gold() } else { Palette::blue() }
}

struct SignScene {
    title: Mob,
    top: CardRow,
    count: Counter,
    low: Vec<Mob>,
    notes: Vec<Mob>,
}

impl SignScene {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.top.mobs();
        all.extend(&self.low);
        all.extend(&self.notes);
        all.extend([self.title, self.count.mob]);
        all
    }
}

fn swaps_add(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<SignScene> {
    let title = k.text(stage, "Swap counts add", 32.0, Palette::ink(), v(0.0, 3.05))?;
    let mut top = card_row(stage, k, 4, sign_top, corner_paint)?;
    let mut count = new_counter(stage, k, "Swaps", (-3.4, 2.05))?;
    let mut showing = top.mobs();
    showing.extend([title, count.mob]);
    fade(stage, &showing, true)?;

    k.until_sentence(stage, "11_18", 1)?;
    run_word(stage, k, &mut top, &mut count, &EVEN_FIRST)?;
    let first = k.text(stage, "(1 2), (3 4): even", 26.0, Palette::gold(), v(SIGN_NOTES_X, 1.5))?;
    fade(stage, &[first], true)?;

    k.until_sentence(stage, "11_18", 2)?;
    run_word(stage, k, &mut top, &mut count, &EVEN_SECOND)?;
    let second = k.text(stage, "(1 3), (2 4): even", 26.0, Palette::gold(), v(SIGN_NOTES_X, 0.9))?;
    fade(stage, &[second], true)?;

    // Sentence 3: where the four swaps landed, as cycles.
    k.until_sentence(stage, "11_18", 3)?;
    let landed = k.tex(
        stage,
        &format!("={}", cycle_tex(after_word::<4>(&[EVEN_FIRST, EVEN_SECOND].concat()))),
        30.0,
        Palette::ink(),
        &[],
        v(SIGN_NOTES_X, 0.25),
    )?;
    fade(stage, &[landed], true)?;

    // Sentence 4: a second row reaches the same arrangement in two swaps.
    k.until_sentence(stage, "11_18", 4)?;
    let mut short = card_row(stage, k, 4, sign_low, corner_paint)?;
    let mut short_count = new_counter(stage, k, "Swaps", (-3.4, -1.65))?;
    let mut appearing = short.mobs();
    appearing.push(short_count.mob);
    fade(stage, &appearing, true)?;
    run_word(stage, k, &mut short, &mut short_count, &SHORT_ROUTE)?;
    let agree = k.text(
        stage,
        "4 swaps, 2 swaps: both even",
        28.0,
        Palette::green(),
        v(0.0, -2.5),
    )?;
    fade(stage, &[agree], true)?;
    let mut low = short.mobs();
    low.push(short_count.mob);
    Ok(SignScene {
        title,
        top,
        count,
        low,
        notes: vec![first, second, landed, agree],
    })
}

fn table_xy(row: usize, column: usize) -> (f64, f64) {
    (3.3 + 1.75 * column as f64, 0.55 - 1.0 * row as f64)
}

fn sign_symbol(parity: usize) -> &'static str {
    if parity == 0 { "+" } else { "-" }
}

/// The four results of the sign rule, in words or as multiplied signs.
fn table_cells(stage: &mut Stage<'_>, k: &Kit, rows: &[usize], in_signs: bool) -> fmn::Result<Vec<Mob>> {
    let mut cells = Vec::new();
    for &row in rows {
        for column in 0..2 {
            let (x, y) = table_xy(row, column);
            let result = parity_sum(row, column);
            cells.push(if in_signs {
                k.tex(
                    stage,
                    &format!(
                        "({})({})={}",
                        sign_symbol(row),
                        sign_symbol(column),
                        sign_symbol(result)
                    ),
                    24.0,
                    parity_color(result),
                    &[],
                    v(x, y),
                )?
            } else {
                k.text(stage, parity_word(result), 26.0, parity_color(result), v(x, y))?
            });
        }
    }
    Ok(cells)
}

fn signs_multiply(stage: &mut Stage<'_>, k: &Kit, scene: &mut SignScene) -> fmn::Result<()> {
    // Sentence 0: one more swap, and the first row of the table.
    let mut leaving = std::mem::take(&mut scene.low);
    leaving.append(&mut scene.notes);
    fade(stage, &leaving, false)?;
    run_word(stage, k, &mut scene.top, &mut scene.count, &[ONE_MORE])?;
    let mut table = vec![
        k.text(stage, "then even", 24.0, Palette::dim(), v(table_xy(0, 0).0, 1.35))?,
        k.text(stage, "then odd", 24.0, Palette::dim(), v(table_xy(0, 1).0, 1.35))?,
        k.text(stage, "even first", 24.0, Palette::dim(), v(1.6, table_xy(0, 0).1))?,
        k.text(stage, "odd first", 24.0, Palette::dim(), v(1.6, table_xy(1, 0).1))?,
    ];
    let even_row = table_cells(stage, k, &[0], false)?;
    let mut appearing = table.clone();
    appearing.extend(&even_row);
    fade(stage, &appearing, true)?;
    table.extend(&even_row);

    // Sentence 1: two single swaps from a fresh row build a three-cycle.
    k.until_sentence(stage, "11_19", 1)?;
    let mut fresh = card_row(stage, k, 4, sign_low, corner_paint)?;
    let mut fresh_count = new_counter(stage, k, "Swaps", (-3.4, -1.65))?;
    let mut appearing = fresh.mobs();
    appearing.push(fresh_count.mob);
    fade(stage, &appearing, true)?;
    run_word(stage, k, &mut fresh, &mut fresh_count, &TWO_ODD)?;
    let built = k.tex(
        stage,
        &format!("{}:\\;1+1=2", cycle_tex(after_word::<4>(&TWO_ODD))),
        28.0,
        Palette::gold(),
        &[],
        v(-3.4, -2.5),
    )?;
    let odd_row = table_cells(stage, k, &[1], false)?;
    let mut appearing = odd_row.clone();
    appearing.push(built);
    fade(stage, &appearing, true)?;
    table.extend(&odd_row);

    // Sentence 2: the same table in signs.
    k.until_sentence(stage, "11_19", 2)?;
    let in_signs = table_cells(stage, k, &[0, 1], true)?;
    let mut leaving = even_row;
    leaving.extend(&odd_row);
    let rule = k.tex(
        stage,
        r"\text{sign}(pq)=\text{sign}(p)\;\text{sign}(q)",
        30.0,
        Palette::green(),
        &[],
        v(3.6, -1.7),
    )?;
    let mut appearing = in_signs.clone();
    appearing.push(rule);
    exchange(stage, &leaving, &appearing)?;
    table.retain(|mob| !leaving.contains(mob));
    table.extend(appearing);

    scene.low = fresh.mobs();
    scene.low.extend([fresh_count.mob, built]);
    scene.notes = table;
    Ok(())
}

/// 11_20: the film's own subgroup test, run on the even permutations.
fn even_subgroup(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let group = k.text(
        stage,
        "All permutations of our labels: a group",
        29.0,
        Palette::ink(),
        v(0.0, 3.05),
    )?;
    fade(stage, &[group], true)?;

    // Sentence 0: two swaps compose, then undo in reverse, and the row is home for the count below.
    let mut row = card_row(stage, k, 4, sign_top, corner_paint)?;
    fade(stage, &row.mobs(), true)?;
    until_phrase(stage, k, THEY_COMPOSE)?;
    trade_slots(stage, &mut row, (0, 1), 0.6)?;
    trade_slots(stage, &mut row, (1, 2), 0.6)?;
    stage.wait(0.3)?;
    trade_slots(stage, &mut row, (1, 2), 0.6)?;
    trade_slots(stage, &mut row, (0, 1), 0.6)?;

    k.until_sentence(stage, "11_20", 1)?;
    let test = k.tex(
        stage,
        r"a,b\in H\ \Longrightarrow\ ab^{-1}\in H",
        30.0,
        Palette::hero(),
        &[],
        v(0.0, 2.45),
    )?;
    let named = k.text(stage, "H: the even ones", 26.0, Palette::gold(), v(SIGN_NOTES_X, 1.5))?;
    let mut count = new_counter(stage, k, "Swaps", (-3.4, -0.15))?;
    fade(stage, &[test, named, count.mob], true)?;

    // Sentence 2: nothing moves, and nothing is even.
    k.until_sentence(stage, "11_20", 2)?;
    pulse(stage, &[count.mob])?;
    let empty = k.text(
        stage,
        "Doing nothing: 0 swaps, even",
        26.0,
        Palette::green(),
        v(SIGN_NOTES_X, 0.9),
    )?;
    fade(stage, &[empty], true)?;

    k.until_sentence(stage, "11_20", 3)?;
    let a = k.tex(
        stage,
        r"a=(1\;2)(3\;4)",
        28.0,
        Palette::gold(),
        &[],
        v(SIGN_NOTES_X, 0.25),
    )?;
    let b = k.tex(
        stage,
        r"b=(1\;3)(2\;4)",
        28.0,
        Palette::gold(),
        &[],
        v(SIGN_NOTES_X, -0.35),
    )?;
    fade(stage, &[a, b], true)?;

    // Sentence 4: undo b, the same swaps in reverse.
    k.until_sentence(stage, "11_20", 4)?;
    let backwards = [EVEN_SECOND[1], EVEN_SECOND[0]];
    run_word(stage, k, &mut row, &mut count, &backwards)?;
    let undo = k.text(
        stage,
        "Undo b: same swaps, reversed",
        26.0,
        Palette::blue(),
        v(-3.0, -1.0),
    )?;
    fade(stage, &[undo], true)?;

    // Sentence 5: then do a, and the count is 2 + 2.
    k.until_sentence(stage, "11_20", 5)?;
    run_word(stage, k, &mut row, &mut count, &EVEN_FIRST)?;
    let added = k.tex(
        stage,
        &format!("{}+{}={}", backwards.len(), EVEN_FIRST.len(), count.value),
        30.0,
        Palette::gold(),
        &[],
        v(-3.0, -1.7),
    )?;
    fade(stage, &[added], true)?;

    k.until_sentence(stage, "11_20", 6)?;
    let passes = k.text(
        stage,
        "The test passes: H is a subgroup",
        28.0,
        Palette::green(),
        v(0.0, -2.45),
    )?;
    fade(stage, &[passes], true)?;
    pulse(stage, &[test])?;

    k.until_sentence(stage, "11_20", 7)?;
    let odd = k.text(
        stage,
        "The odd ones have no identity: no subgroup",
        27.0,
        Palette::red(),
        v(0.0, -3.05),
    )?;
    fade(stage, &[odd], true)?;

    let mut on_screen = row.mobs();
    on_screen.extend([group, test, named, count.mob, empty, a, b, undo, added, passes, odd]);
    Ok(on_screen)
}

/// Where the `index`-th pair of 11_21 sits: its even member, then its odd one.
fn pair_xy(index: usize) -> ((f64, f64), (f64, f64)) {
    let (block, row) = (index / 6, index % 6);
    let y = 1.5 - 0.6 * row as f64;
    let left = -5.0 + 7.2 * block as f64;
    ((left, y), (left + 2.5, y))
}

/// 11_21: one extra swap pairs each even arrangement with an odd one.
fn twelve_of_twenty_four(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let evens = even_arrangements();
    let title = k.text(stage, "How many even permutations?", 32.0, Palette::ink(), v(0.0, 3.05))?;
    fade(stage, &[title], true)?;

    // Sentence 1: all twenty-four, one colour until the pairing tells the two kinds apart.
    k.until_sentence(stage, "11_21", 1)?;
    let count = k.tex(
        stage,
        &format!(r"4\times3\times2\times1={}", arrangements().len()),
        30.0,
        Palette::ink(),
        &[],
        v(0.0, 2.4),
    )?;
    let mut left_cells = Vec::new();
    let mut right_cells = Vec::new();
    for (index, &p) in evens.iter().enumerate() {
        let (even_at, odd_at) = pair_xy(index);
        left_cells.push(k.text(stage, &cell_text(p), 24.0, Palette::ink(), v(even_at.0, even_at.1))?);
        right_cells.push(k.text(
            stage,
            &cell_text(with_first_two_swapped(p)),
            24.0,
            Palette::ink(),
            v(odd_at.0, odd_at.1),
        )?);
    }
    let mut appearing = left_cells.clone();
    appearing.extend(&right_cells);
    appearing.push(count);
    fade(stage, &appearing, true)?;

    // Sentence 2: the extra swap, as an arrow from each cell to its partner.
    k.until_sentence(stage, "11_21", 2)?;
    let mut arrows = Vec::new();
    for index in 0..evens.len() {
        let (even_at, odd_at) = pair_xy(index);
        arrows.push(straight(stage, even_at, odd_at, 0.55, Palette::dim())?);
    }
    fade(stage, &arrows, true)?;
    pulse(stage, &[left_cells[0], right_cells[0]])?;

    k.until_sentence(stage, "11_21", 3)?;
    let even_key = k.text(stage, "Even", 26.0, Palette::gold(), v(-0.15, 0.3))?;
    let odd_key = k.text(stage, "Odd", 26.0, Palette::blue(), v(-0.15, -0.3))?;
    recolor(stage, &left_cells, Palette::gold(), 0.3)?;
    recolor(stage, &right_cells, Palette::blue(), 0.3)?;
    fade(stage, &[even_key, odd_key], true)?;
    pulse(stage, &right_cells)?;
    pulse(stage, &left_cells)?;

    // Sentence 4: the count the later Lagrange chapter relies on.
    k.until_sentence(stage, "11_21", 4)?;
    let split = k.text(
        stage,
        &format!(
            "No collisions, none missing: {} even, {} odd",
            evens.len(),
            arrangements().len() - evens.len()
        ),
        28.0,
        Palette::green(),
        v(0.0, -2.45),
    )?;
    fade(stage, &[split], true)?;

    // Sentence 5: what the even ones are, colour by kind.
    k.until_sentence(stage, "11_21", 5)?;
    // The even cells are about to take three colours, so the two-colour key leaves first.
    fade(stage, &[even_key, odd_key], false)?;
    let mut nothing = Vec::new();
    let mut doubles = Vec::new();
    for (&p, &cell) in evens.iter().zip(&left_cells) {
        match kind_of(p) {
            0 => nothing.push(cell),
            1 => doubles.push(cell),
            _ => {}
        }
    }
    recolor(stage, &nothing, Palette::green(), 0.3)?;
    recolor(stage, &doubles, Palette::hero(), 0.3)?;
    let [one, three, eight] = even_kind_counts();
    let kinds = k.tex(
        stage,
        &format!(
            r"{one}+{three}+{eight}={}\quad\text{{identity, double swaps, three-cycles}}",
            evens.len()
        ),
        26.0,
        Palette::ink(),
        &[],
        v(0.0, -3.05),
    )?;
    fade(stage, &[kinds], true)?;

    let mut on_screen = left_cells;
    on_screen.extend(right_cells);
    on_screen.extend(arrows);
    on_screen.extend([title, count, split, kinds]);
    Ok(on_screen)
}

// ---- 11_10, 11_22 and 11_23: ten cards, one fixed shuffle --------------------

const DECK_Y: f64 = 1.0;
const LANE_TOP: f64 = 2.35;
const LANE_LOW: f64 = 0.05;
const DECK_STRIP_TOP: f64 = -2.0;
const DECK_STRIP_LOW: f64 = -2.7;
const WALK_BEND: f64 = 1.0;

fn deck_slot(slot: usize) -> (f64, f64) {
    (-5.4 + 1.2 * slot as f64, DECK_Y)
}

fn deck_above(slot: usize) -> (f64, f64) {
    let (x, y) = deck_slot(slot);
    (x, y + 0.66)
}

fn deck_below(slot: usize) -> (f64, f64) {
    let (x, y) = deck_slot(slot);
    (x, y - 0.42)
}

/// Before the prediction the deck is one colour, so it does not show the cycles.
fn plain_card(_: usize) -> Srgb {
    Palette::ink()
}

/// Colour by the card's cycle under the ten-card out-shuffle, once it is read.
fn deck_color(label: usize) -> Srgb {
    match label {
        1 | 10 => Palette::dim(),
        4 | 7 => Palette::blue(),
        _ => Palette::hero(),
    }
}

/// Lift the top half above the row and drop the bottom half below it.
fn cut(stage: &mut Stage<'_>, deck: &CardRow, run_time: f64) -> fmn::Result<()> {
    let half = deck.at.len() / 2;
    let mut moves = Vec::new();
    for (&card, &slot) in deck.cards.iter().zip(&deck.at) {
        let (x, _) = (deck.slot)(slot);
        moves.push((card, (x, if slot < half { LANE_TOP } else { LANE_LOW })));
    }
    d4g::glide_all(stage, &moves, run_time)
}

/// Each half slides along its own lane to its new column, then drops into place.
fn interleave(stage: &mut Stage<'_>, deck: &mut CardRow, p: &[usize], run_time: f64) -> fmn::Result<()> {
    let half = deck.at.len() / 2;
    let mut slide = Vec::new();
    let mut land = Vec::new();
    for (&card, slot) in deck.cards.iter().zip(deck.at.iter_mut()) {
        let lane = if *slot < half { LANE_TOP } else { LANE_LOW };
        *slot = p[*slot];
        let (x, y) = (deck.slot)(*slot);
        slide.push((card, (x, lane)));
        land.push((card, (x, y)));
    }
    d4g::glide_all(stage, &slide, run_time)?;
    d4g::glide_all(stage, &land, run_time)
}

struct ShuffleScene {
    deck: CardRow,
    p: Vec<usize>,
    strips: Vec<ReturnStrip>,
    count: Counter,
    arrows: Vec<Mob>,
    notes: Vec<Mob>,
}

impl ShuffleScene {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.deck.mobs();
        for strip in &self.strips {
            all.extend(strip.mobs());
        }
        all.extend(&self.arrows);
        all.extend(&self.notes);
        all.push(self.count.mob);
        all
    }
}

/// One whole shuffle, counted, lighting any cycle that is home.
fn shuffle_once(stage: &mut Stage<'_>, k: &Kit, scene: &mut ShuffleScene, run_time: f64) -> fmn::Result<()> {
    cut(stage, &scene.deck, run_time)?;
    interleave(stage, &mut scene.deck, &scene.p, run_time)?;
    let run = scene.count.value + 1;
    let lamps = lamps_for(stage, &mut scene.strips, &scene.p, run)?;
    advance(stage, k, &mut scene.count, &lamps)
}

fn shuffle_question(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<ShuffleScene> {
    let title = k.text(
        stage,
        "Repeat one fixed rearrangement",
        33.0,
        Palette::ink(),
        v(0.0, 3.05),
    )?;
    fade(stage, &[title], true)?;

    k.until_sentence(stage, "11_10", 1)?;
    let worry = k.text(
        stage,
        "Does the mixing go on forever?",
        30.0,
        Palette::blue(),
        v(0.0, 0.4),
    )?;
    fade(stage, &[worry], true)?;

    // Sentence 2: ten cards, top card on the left.
    k.until_sentence(stage, "11_10", 2)?;
    fade(stage, &[worry], false)?;
    let mut deck = card_row(stage, k, DECK_CARDS, deck_slot, plain_card)?;
    let top = k.text(stage, "top", 22.0, Palette::dim(), v(-6.35, DECK_Y))?;
    let bottom = k.text(stage, "bottom", 22.0, Palette::dim(), v(6.45, DECK_Y))?;
    let mut appearing = deck.mobs();
    appearing.extend([top, bottom]);
    fade(stage, &appearing, true)?;
    deck.rest.extend([title, top, bottom]);

    // Sentence 3: the cut, then the interleave on its words.
    k.until_sentence(stage, "11_10", 3)?;
    let p = perfect_shuffle(DECK_CARDS, true);
    cut(stage, &deck, 0.5)?;
    until_phrase(stage, k, INTERLEAVE)?;
    interleave(stage, &mut deck, &p, 0.55)?;
    let mut count = new_counter(stage, k, "Shuffles", (4.6, -2.35))?;
    fade(stage, &[count.mob], true)?;
    advance(stage, k, &mut count, &[])?;

    k.until_sentence(stage, "11_10", 4)?;
    let name = k.text(
        stage,
        "Out-shuffle: the top card stays on top",
        27.0,
        Palette::ink(),
        v(0.0, -0.75),
    )?;
    fade(stage, &[name], true)?;

    k.until_sentence(stage, "11_10", 5)?;
    let rule = k.text(
        stage,
        "Top half lands on 1, 3, 5, 7, 9. Bottom half lands on 2, 4, 6, 8, 10.",
        25.0,
        Palette::dim(),
        v(0.0, -1.4),
    )?;
    fade(stage, &[rule], true)?;

    k.until_sentence(stage, "11_10", 6)?;
    let question = k.text(
        stage,
        "How many shuffles bring every card home?",
        31.0,
        Palette::blue(),
        v(-1.3, -3.0),
    )?;
    fade(stage, &[question], true)?;

    Ok(ShuffleScene {
        deck,
        p,
        strips: Vec::new(),
        count,
        arrows: Vec::new(),
        notes: vec![name, rule, question],
    })
}

/// A ring walks one cycle's positions, drawing each link as it goes.
fn walk_cycle(
    stage: &mut Stage<'_>,
    cycle: &[usize],
    above: bool,
    color: Srgb,
    arrows: &mut Vec<Mob>,
    beside: &[Mob],
) -> fmn::Result<()> {
    let anchor: fn(usize) -> (f64, f64) = if above { deck_above } else { deck_below };
    let offsets = link_offsets(cycle);
    let walker = halo(stage, deck_slot(cycle[0] - 1), 0.37, Palette::ink(), 3.0);
    let mut appearing = vec![walker];
    appearing.extend(beside);
    fade(stage, &appearing, true)?;
    for (index, &label) in cycle.iter().enumerate() {
        let next = cycle[(index + 1) % cycle.len()];
        let bend = if above == (next > label) { -WALK_BEND } else { WALK_BEND };
        let (tail, head) = offsets[index];
        let arrow = curved(
            stage,
            shifted(anchor(label - 1), tail),
            shifted(anchor(next - 1), head),
            bend,
            color,
        )?;
        d4g::show(stage, &[arrow], 0.2)?;
        travel(stage, &[(walker, deck_slot(next - 1))], bend, 0.95)?;
        arrows.push(arrow);
    }
    fade(stage, &[walker], false)
}

fn read_the_cycles(stage: &mut Stage<'_>, k: &Kit, scene: &mut ShuffleScene) -> fmn::Result<()> {
    clear_notes(stage, &mut scene.notes)?;

    k.until_sentence(stage, "11_22", 1)?;
    let ends = [
        halo(stage, deck_slot(0), 0.37, Palette::gold(), 3.0),
        halo(stage, deck_slot(DECK_CARDS - 1), 0.37, Palette::gold(), 3.0),
    ];
    let stay = k.text(stage, "Positions 1 and 10 stay", 27.0, Palette::gold(), v(0.0, -0.75))?;
    fade(stage, &[ends[0], ends[1], stay], true)?;
    stage.wait(0.5)?;
    fade(stage, &[ends[0], ends[1], stay], false)?;

    // Sentences 2 and 3: the two trails, in the words' order.
    let trails = moving_cycles(&scene.p);
    k.until_sentence(stage, "11_22", 2)?;
    // The deck has shuffled once, so an arrow from a slot lands on the card that started there.
    let (from, to) = (trails[0][0], trails[0][1]);
    let reading = k.text(
        stage,
        &format!("Card {from} went from position {from} to position {to}"),
        27.0,
        Palette::hero(),
        v(0.0, -0.75),
    )?;
    walk_cycle(stage, &trails[0], true, Palette::hero(), &mut scene.arrows, &[reading])?;
    k.until_sentence(stage, "11_22", 3)?;
    fade(stage, &[reading], false)?;
    walk_cycle(stage, &trails[1], false, Palette::blue(), &mut scene.arrows, &[])?;

    k.until_sentence(stage, "11_22", 4)?;
    let written = k.tex(
        stage,
        &moving_cycles_tex(&scene.p),
        34.0,
        Palette::ink(),
        &[],
        v(0.0, -0.75),
    )?;
    let predicted = k.tex(
        stage,
        &format!(r"\text{{lcm}}(6,2)={}", order_by_cycles(&scene.p)),
        34.0,
        Palette::green(),
        &[],
        v(0.0, -1.4),
    )?;
    // The cycles are read, so the cards can wear their colours from here on.
    let mut painted = Vec::new();
    for (card, &slot) in scene.deck.at.iter().enumerate() {
        let at = (scene.deck.slot)(slot);
        painted.push(token(
            stage,
            k,
            &(card + 1).to_string(),
            deck_color(card + 1),
            at,
            0.27,
        )?);
    }
    let mut appearing = painted.clone();
    appearing.extend([written, predicted]);
    exchange(stage, &scene.deck.cards, &appearing)?;
    scene.deck.cards = painted;
    scene.notes.extend([written, predicted]);
    Ok(())
}

fn shuffle_for_real(stage: &mut Stage<'_>, k: &Kit, scene: &mut ShuffleScene) -> fmn::Result<()> {
    // Sentence 0: the arrows go, the strips arrive, and the second shuffle runs.
    let leaving = std::mem::take(&mut scene.arrows);
    let trails = moving_cycles(&scene.p);
    let long_cycle = return_strip(
        stage,
        k,
        DECK_STRIP_TOP,
        r"\text{six-cycle}",
        Palette::hero(),
        &trails[0],
        6,
    )?;
    let pair = return_strip(stage, k, DECK_STRIP_LOW, r"\text{swap}", Palette::blue(), &trails[1], 6)?;
    let mut appearing = long_cycle.mobs();
    appearing.extend(pair.mobs());
    scene.strips = vec![long_cycle, pair];
    exchange(stage, &leaving, &appearing)?;
    shuffle_once(stage, k, scene, 0.3)?;

    // Sentence 1: two shuffles in, the pair is home and the rest is not.
    k.until_sentence(stage, "11_23", 1)?;
    let pair_rings = [
        halo(stage, deck_slot(trails[1][0] - 1), 0.37, Palette::blue(), 3.0),
        halo(stage, deck_slot(trails[1][1] - 1), 0.37, Palette::blue(), 3.0),
    ];
    fade(stage, &pair_rings, true)?;
    until_phrase(stage, k, STILL_OUT)?;
    let away = k.text(
        stage,
        &format!(
            "{} of {DECK_CARDS} still away",
            out_of_place(&power(&scene.p, scene.count.value)).len()
        ),
        27.0,
        Palette::ink(),
        v(4.6, -1.55),
    )?;
    fade(stage, &[away], true)?;
    stage.wait(0.4)?;
    fade(stage, &[pair_rings[0], pair_rings[1], away], false)?;
    for _ in 0..3 {
        shuffle_once(stage, k, scene, 0.22)?;
    }

    // Sentence 2: the sixth shuffle lands on the words.
    k.until_sentence(stage, "11_23", 2)?;
    shuffle_once(stage, k, scene, 0.22)?;
    let both = column_frame(stage, 6, DECK_STRIP_TOP, DECK_STRIP_LOW, Palette::green())?;
    let home = k.text(stage, "All ten home", 27.0, Palette::green(), v(4.6, -1.55))?;
    fade(stage, &[both, home], true)?;
    scene.notes.extend([both, home]);

    k.until_sentence(stage, "11_23", 3)?;
    pulse(stage, &[scene.notes[0]])?;
    Ok(())
}

// ---- 11_24 to 11_27: fifty-two cards as a staircase, then eight wires --------

const BAR_BASE: f64 = -1.35;
const BAR_STEP: f64 = 0.2;
const STRIP_OUT_Y: f64 = -1.85;
const STRIP_IN_Y: f64 = -2.35;

fn bar_x(slot: usize) -> f64 {
    -5.1 + BAR_STEP * slot as f64
}

fn bar_height(card: usize) -> f64 {
    0.06 * (card + 1) as f64
}

fn bar_y(card: usize) -> f64 {
    BAR_BASE + bar_height(card) / 2.0
}

/// The four suits, thirteen cards each, so a broken staircase is also visibly mixed.
fn suit_color(card: usize) -> Srgb {
    match card / 13 {
        0 => Palette::blue(),
        1 => Palette::green(),
        2 => Palette::gold(),
        _ => Palette::hero(),
    }
}

struct Bars {
    bars: Vec<Mob>,
    at: Vec<usize>,
}

fn new_bars(stage: &mut Stage<'_>) -> fmn::Result<Bars> {
    let mut bars = Vec::new();
    for card in 0..FULL_DECK {
        let bar = Rectangle::new()
            .width(0.15)
            .height(bar_height(card))
            .style(ink_style(suit_color(card)))
            .build()?
            .moved_to(v(bar_x(card), bar_y(card)));
        bars.push(stage.arena_mut().add(bar));
    }
    fade(stage, &bars, true)?;
    Ok(Bars {
        bars,
        at: (0..FULL_DECK).collect(),
    })
}

fn run_bars(stage: &mut Stage<'_>, bars: &mut Bars, p: &[usize], run_time: f64) -> fmn::Result<()> {
    let mut moves = Vec::new();
    for (card, (&bar, slot)) in bars.bars.iter().zip(bars.at.iter_mut()).enumerate() {
        *slot = p[*slot];
        moves.push((bar, (bar_x(*slot), bar_y(card))));
    }
    d4g::glide_all(stage, &moves, run_time)
}

/// Every run of `p` from here, counted; `pause` lets the viewer see each state.
fn run_bars_counted(
    stage: &mut Stage<'_>,
    k: &Kit,
    bars: &mut Bars,
    count: &mut Counter,
    p: &[usize],
    runs: usize,
    run_time: f64,
) -> fmn::Result<()> {
    for _ in 0..runs {
        run_bars(stage, bars, p, run_time)?;
        advance(stage, k, count, &[])?;
    }
    Ok(())
}

/// One segment per cycle, as wide as the cycle is long, under the bars.
fn cycle_strip(stage: &mut Stage<'_>, lengths: &[usize], y: f64) -> fmn::Result<Vec<Mob>> {
    let mut segments = Vec::new();
    let mut from = 0;
    for &length in lengths {
        let color = match length {
            1 => Palette::dim(),
            2 => Palette::blue(),
            8 => Palette::hero(),
            _ => Palette::gold(),
        };
        let width = BAR_STEP * length as f64;
        let segment = Rectangle::new()
            .width(width - 0.04)
            .height(0.3)
            .style(Style::default().fill(color, 0.55).stroke(color, 2.0, 1.0))
            .build()?
            .moved_to(v(bar_x(from) - BAR_STEP / 2.0 + width / 2.0, y));
        segments.push(stage.arena_mut().add(segment));
        from += length;
    }
    Ok(segments)
}

struct FullDeck {
    bars: Bars,
    count: Counter,
    strips: Vec<Mob>,
    notes: Vec<Mob>,
}

impl FullDeck {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.bars.bars.clone();
        all.extend(&self.strips);
        all.extend(&self.notes);
        all.push(self.count.mob);
        all
    }
}

fn full_deck(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<FullDeck> {
    let out = perfect_shuffle(FULL_DECK, true);
    let (digit, power_of_ten) = factorial_scientific(52);
    let size = k.tex(
        stage,
        &format!(r"52!\ \text{{is about}}\ {digit}\times10^{{{power_of_ten}}}\ \text{{arrangements}}"),
        32.0,
        Palette::ink(),
        &[],
        v(0.0, 3.0),
    )?;
    let each = k.text(
        stage,
        "Each bar is one card, standing in its position",
        25.0,
        Palette::dim(),
        v(0.0, 2.35),
    )?;
    fade(stage, &[size, each], true)?;
    let bars = new_bars(stage)?;
    let count = new_counter(stage, k, "Shuffles", (5.4, 2.35))?;

    // Sentence 1: the cycle lengths, as widths that fill the fifty-two positions.
    k.until_sentence(stage, "11_24", 1)?;
    let mut strips = cycle_strip(stage, &cycle_lengths(&out), STRIP_OUT_Y)?;
    strips.push(k.text(stage, "out", 24.0, Palette::dim(), v(-6.0, STRIP_OUT_Y))?);
    let types = k.text(
        stage,
        "By size: 2 fixed, 1 swap, 6 cycles of length 8",
        27.0,
        Palette::ink(),
        v(0.0, -2.55),
    )?;
    let mut appearing = strips.clone();
    appearing.push(types);
    fade(stage, &appearing, true)?;

    k.until_sentence(stage, "11_24", 2)?;
    let predicted = k.tex(
        stage,
        &format!(r"\text{{lcm}}(1,2,8)={}", order_by_cycles(&out)),
        32.0,
        Palette::green(),
        &[],
        v(0.0, -3.15),
    )?;
    fade(stage, &[predicted], true)?;

    // Sentence 3: all eight shuffles, watched in silence after the words.
    k.until_sentence(stage, "11_24", 3)?;
    let mut scene = FullDeck {
        bars,
        count,
        strips,
        notes: vec![size, each, types, predicted],
    };
    fade(stage, &[scene.count.mob], true)?;
    let runs = first_return(&out);
    run_bars_counted(stage, k, &mut scene.bars, &mut scene.count, &out, runs, 0.5)?;
    // The staircase is back: the count meets the prediction, and both hold before anything changes.
    pulse(stage, &[scene.notes[3], scene.count.mob])?;
    stage.wait(0.6)?;
    Ok(scene)
}

fn other_way(stage: &mut Stage<'_>, k: &Kit, scene: &mut FullDeck) -> fmn::Result<()> {
    let other = perfect_shuffle(FULL_DECK, false);
    let out_runs = scene.count.value;

    // Sentence 0: the small change, one in-shuffle from the restored staircase.
    let name = k.text(
        stage,
        "In-shuffle: the top card drops to second place",
        27.0,
        Palette::gold(),
        v(0.0, -3.15),
    )?;
    let leaving = [scene.notes[1], scene.notes[2], scene.notes[3]];
    let restart = counter_text(stage, k, "In-shuffles", 0, scene.count.at)?;
    let mut changing = leaving.to_vec();
    changing.push(scene.count.mob);
    swap_caption(stage, &changing, &[name, restart])?;
    scene.notes = vec![scene.notes[0], name];
    scene.count = Counter {
        mob: restart,
        label: "In-shuffles",
        value: 0,
        at: scene.count.at,
    };
    run_bars_counted(stage, k, &mut scene.bars, &mut scene.count, &other, 1, 0.8)?;

    // Sentence 1: one segment as wide as the deck.
    k.until_sentence(stage, "11_25", 1)?;
    let mut single = cycle_strip(stage, &cycle_lengths(&other), STRIP_IN_Y)?;
    single.push(k.text(stage, "in", 24.0, Palette::dim(), v(-6.0, STRIP_IN_Y))?);
    let out_return = k.text(stage, &out_runs.to_string(), 26.0, Palette::hero(), v(5.9, STRIP_OUT_Y))?;
    let in_return = k.text(
        stage,
        &first_return(&other).to_string(),
        26.0,
        Palette::gold(),
        v(5.9, STRIP_IN_Y),
    )?;
    let mut appearing = single.clone();
    appearing.extend([out_return, in_return]);
    fade(stage, &appearing, true)?;
    scene.strips.extend(single);

    // Sentence 2: the out-shuffle's count, tried on the other shuffle.
    k.until_sentence(stage, "11_25", 2)?;
    run_bars_counted(stage, k, &mut scene.bars, &mut scene.count, &other, out_runs - 1, 0.28)?;
    let home_now = FULL_DECK - out_of_place(&power(&other, out_runs)).len();
    let still = k.text(
        stage,
        &format!("After {out_runs} in-shuffles: {home_now} cards home"),
        27.0,
        Palette::red(),
        v(0.0, -3.15),
    )?;
    swap_caption(stage, &[name], &[still])?;
    scene.notes[1] = still;

    k.until_sentence(stage, "11_25", 3)?;
    pulse(stage, &[out_return, in_return])?;
    scene.notes.extend([out_return, in_return]);
    Ok(())
}

fn wire_xy(column: usize, row: usize) -> (f64, f64) {
    (-5.2 + 2.6 * column as f64, 1.75 - 0.5 * row as f64)
}

fn wire_color(row: usize) -> Srgb {
    match row {
        0 | 7 => Palette::dim(),
        1 | 2 | 4 => Palette::hero(),
        _ => Palette::blue(),
    }
}

/// 11_26: the closed loop in words over the bars, then the same shuffle as wiring.
fn loop_and_wires(stage: &mut Stage<'_>, k: &Kit, scene: FullDeck) -> fmn::Result<Vec<Mob>> {
    let summary = k.text(
        stage,
        "One fixed rearrangement walks a closed loop",
        29.0,
        Palette::blue(),
        v(0.0, 2.35),
    )?;
    fade(stage, &[summary], true)?;
    k.until_sentence(stage, "11_26", 1)?;
    let loop_length = k.text(
        stage,
        "The order is the length of that loop",
        27.0,
        Palette::green(),
        v(0.0, -3.15),
    )?;
    swap_caption(stage, &[scene.notes[1]], &[loop_length])?;

    // Sentence 2: the same pattern on eight wires, three stages lit.
    k.until_sentence(stage, "11_26", 2)?;
    let mut leaving = scene.mobs();
    leaving.retain(|&mob| mob != scene.notes[1]);
    leaving.extend([summary, loop_length]);
    fade(stage, &leaving, false)?;
    let wires = perfect_shuffle(WIRES, true);
    let order = first_return(&wires);
    let title = k.text(
        stage,
        "The same shuffle, on eight wires",
        32.0,
        Palette::ink(),
        v(0.0, 3.05),
    )?;
    let mut on_screen = vec![title];
    for stage_index in 0..order {
        on_screen.push(k.text(
            stage,
            &format!("Stage {}", stage_index + 1),
            22.0,
            Palette::dim(),
            v(wire_xy(stage_index, 0).0 + 1.3, 2.4),
        )?);
        for (row, &to) in wires.iter().enumerate() {
            let (a, b) = (wire_xy(stage_index, row), wire_xy(stage_index + 1, to));
            let line = Line::new(v(a.0, a.1), v(b.0, b.1))
                .style(stroke_style(wire_color(row), 2.0))
                .build()?;
            on_screen.push(stage.arena_mut().add(line));
        }
    }
    let mut tokens = Vec::new();
    for row in 0..WIRES {
        tokens.push(token(
            stage,
            k,
            &(row + 1).to_string(),
            wire_color(row),
            wire_xy(0, row),
            0.21,
        )?);
    }
    on_screen.extend(&tokens);
    fade(stage, &on_screen, true)?;

    // Sentence 3: data follow the wires and arrive back on their own lines.
    k.until_sentence(stage, "11_26", 3)?;
    let mut rows: Vec<usize> = (0..WIRES).collect();
    for column in 1..=order {
        let mut moves = Vec::new();
        for (&mob, row) in tokens.iter().zip(rows.iter_mut()) {
            *row = wires[*row];
            moves.push((mob, wire_xy(column, *row)));
        }
        d4g::glide_all(stage, &moves, 0.7)?;
        if column < order {
            stage.wait(0.25)?;
        }
    }
    let back = k.text(
        stage,
        &format!("After {order} stages: all data back on their starting wires"),
        27.0,
        Palette::green(),
        v(0.0, -2.45),
    )?;
    fade(stage, &[back], true)?;
    on_screen.push(back);

    // Sentence 4: one more stage, in red, only repeats the first.
    k.until_sentence(stage, "11_26", 4)?;
    let mut extra = Vec::new();
    for (row, &to) in wires.iter().enumerate() {
        let (a, b) = (wire_xy(order, row), wire_xy(order + 1, to));
        let line = Line::new(v(a.0, a.1), v(b.0, b.1))
            .style(stroke_style(Palette::red(), 2.0))
            .build()?;
        extra.push(stage.arena_mut().add(line));
    }
    let repeat = k.text(
        stage,
        "Without switches, stage 4 repeats stage 1",
        27.0,
        Palette::red(),
        v(0.0, -3.05),
    )?;
    let fourth = k.text(
        stage,
        &format!("Stage {}", order + 1),
        22.0,
        Palette::red(),
        v(wire_xy(order, 0).0 + 1.3, 2.4),
    )?;
    let mut appearing = extra.clone();
    appearing.extend([repeat, fourth]);
    fade(stage, &appearing, true)?;
    let mut moves = Vec::new();
    for (&mob, row) in tokens.iter().zip(rows.iter_mut()) {
        *row = wires[*row];
        moves.push((mob, wire_xy(order + 1, *row)));
    }
    d4g::glide_all(stage, &moves, 0.7)?;
    on_screen.extend(extra);
    on_screen.extend([repeat, fourth]);
    Ok(on_screen)
}

/// 11_27: seven more undo the first, and one slip is one swap too many.
fn undo_and_slip(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<()> {
    let out = perfect_shuffle(FULL_DECK, true);
    let eight = first_return(&out);
    let mut bars = new_bars(stage)?;
    let returns = k.tex(stage, &format!("p^{eight}=e"), 32.0, Palette::ink(), &[], v(-2.2, 2.6))?;
    fade(stage, &[returns], true)?;

    // Sentence 1: one shuffle, then the seven that undo it.
    k.until_sentence(stage, "11_27", 1)?;
    run_bars(stage, &mut bars, &out, 0.45)?;
    let undone = k.tex(
        stage,
        &format!("p^{{-1}}=p^{}", order_by_cycles(&out) - 1),
        32.0,
        Palette::green(),
        &[],
        v(2.2, 2.6),
    )?;
    fade(stage, &[undone], true)?;
    stage.wait(0.3)?;
    for _ in 1..eight {
        run_bars(stage, &mut bars, &out, 0.22)?;
    }

    k.until_sentence(stage, "11_27", 2)?;
    let limits = k.text(
        stage,
        "The group records the moves, not how well a device performs them",
        27.0,
        Palette::dim(),
        v(0.0, -2.1),
    )?;
    fade(stage, &[limits], true)?;

    // Sentence 3: one shuffle, then two neighbouring bars trade places, in red.
    k.until_sentence(stage, "11_27", 3)?;
    run_bars(stage, &mut bars, &out, 0.45)?;
    let mut slipped = Vec::new();
    let mut moves = Vec::new();
    for (card, (&bar, slot)) in bars.bars.iter().zip(bars.at.iter_mut()).enumerate() {
        if *slot == SLIP_SLOT || *slot == SLIP_SLOT + 1 {
            *slot = 2 * SLIP_SLOT + 1 - *slot;
            slipped.push(bar);
            moves.push((bar, (bar_x(*slot), bar_y(card))));
        }
    }
    recolor(stage, &slipped, Palette::red(), 0.25)?;
    d4g::glide_all(stage, &moves, 0.5)?;

    // Sentence 4: the other seven shuffles run, and the staircase stays broken.
    k.until_sentence(stage, "11_27", 4)?;
    for _ in 1..eight {
        run_bars(stage, &mut bars, &out, 0.25)?;
    }
    let odd = k.text(
        stage,
        &format!(
            "One extra swap: {}, so not home",
            parity_of(&slipped_run(&out, SLIP_SLOT, eight))
        ),
        28.0,
        Palette::red(),
        v(0.0, -2.8),
    )?;
    fade(stage, &[odd], true)?;

    k.until_sentence(stage, "11_27", 5)?;
    let next = k.text(
        stage,
        "Next: when do two groups have the same structure?",
        27.0,
        Palette::ink(),
        v(0.0, -2.8),
    )?;
    swap_caption(stage, &[limits, odd], &[next])
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

        // Repeat one permutation: what "return" has to mean, and why the
        // lengths combine by lcm. The prediction lasts as long as the hold.
        k.say(stage, "11_12")?;
        fade(stage, &disjoint, false)?;
        let mut loops = loops_question(stage, &k)?;
        k.hold(stage)?;
        stage.wait(3.0)?;

        k.say(stage, "11_13")?;
        loops_run(stage, &k, &mut loops)?;

        k.say(stage, "11_14")?;
        loops_why(stage, &k, &mut loops)?;

        k.say(stage, "11_15")?;
        fade(stage, &loops.mobs(), false)?;
        let mut second = second_question(stage, &k)?;
        k.hold(stage)?;
        stage.wait(3.0)?;

        k.say(stage, "11_16")?;
        second_run(stage, &k, &mut second)?;

        k.say(stage, "11_17")?;
        fade(stage, &second.mobs(), false)?;
        let overlap = overlap_warning(stage, &k)?;

        k.say(stage, "11_5")?;
        fade(stage, &overlap, false)?;
        // Sentences 0 and 1: the new word, printed and performed, before any formula.
        let swap_name = k.text(
            stage,
            "Transposition: swap two labels",
            32.0,
            Palette::gold(),
            v(0.0, 1.6),
        )?;
        let swap_cards = [
            card(stage, &k, 1, (-1.5, 0.0), 0.42)?,
            card(stage, &k, 3, (1.5, 0.0), 0.42)?,
        ];
        fade(stage, &[swap_name, swap_cards[0], swap_cards[1]], true)?;
        travel(
            stage,
            &[(swap_cards[0], (1.5, 0.0)), (swap_cards[1], (-1.5, 0.0))],
            SWAP_ARC,
            0.9,
        )?;
        k.until_sentence(stage, "11_5", 2)?;
        fade(stage, &[swap_name, swap_cards[0], swap_cards[1]], false)?;
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

        // The sign rule earned on cards, then the even permutations as a
        // subgroup of twelve, which the Lagrange chapter relies on.
        k.say(stage, "11_18")?;
        let mut leaving = routes;
        leaving.push(product);
        fade(stage, &leaving, false)?;
        let mut signs = swaps_add(stage, &k)?;

        k.say(stage, "11_19")?;
        signs_multiply(stage, &k, &mut signs)?;

        k.say(stage, "11_20")?;
        fade(stage, &signs.mobs(), false)?;
        let test = even_subgroup(stage, &k)?;

        k.say(stage, "11_21")?;
        fade(stage, &test, false)?;
        let half = twelve_of_twenty_four(stage, &k)?;

        k.say(stage, "11_9")?;
        fade(stage, &half, false)?;
        let square = square_as_labels(stage, &k)?;

        // The worked application: one fixed shuffle, read through its cycles.
        k.say(stage, "11_10")?;
        fade(stage, &square, false)?;
        let mut deck = shuffle_question(stage, &k)?;
        k.hold(stage)?;
        stage.wait(3.0)?;

        k.say(stage, "11_22")?;
        read_the_cycles(stage, &k, &mut deck)?;

        k.say(stage, "11_23")?;
        shuffle_for_real(stage, &k, &mut deck)?;

        k.say(stage, "11_24")?;
        fade(stage, &deck.mobs(), false)?;
        let mut full = full_deck(stage, &k)?;

        k.say(stage, "11_25")?;
        other_way(stage, &k, &mut full)?;

        k.say(stage, "11_26")?;
        let wires = loop_and_wires(stage, &k, full)?;

        k.say(stage, "11_27")?;
        fade(stage, &wires, false)?;
        undo_and_slip(stage, &k)?;
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
}

#[cfg(test)]
mod repetition_tests {
    use super::*;

    /// Every permutation of `n` labels, as images, built by insertion.
    fn all_permutations(n: usize) -> Vec<Vec<usize>> {
        let mut all = vec![Vec::new()];
        for label in 0..n {
            let mut next = Vec::new();
            for p in &all {
                for at in 0..=label {
                    let mut longer: Vec<usize> = p.clone();
                    longer.insert(at, label);
                    next.push(longer);
                }
            }
            all = next;
        }
        all
    }

    #[test]
    fn the_first_row_is_home_at_six_and_never_before() {
        let p = first_loops();
        assert_eq!(home_runs(&p, &THREE_LOOP, STRIP_RUNS), [3, 6, 9, 12]);
        assert_eq!(home_runs(&p, &TWO_LOOP, STRIP_RUNS), [2, 4, 6, 8, 10, 12]);
        assert_eq!(first_return(&p), 6);
        assert_eq!(order_by_cycles(&p), 6);
        // Run by run, as 11_13 says it.
        assert!(labels_home(&p, &TWO_LOOP, 2) && !labels_home(&p, &THREE_LOOP, 2));
        assert!(labels_home(&p, &THREE_LOOP, 3) && !labels_home(&p, &TWO_LOOP, 3));
        assert!(labels_home(&p, &TWO_LOOP, 4) && !labels_home(&p, &THREE_LOOP, 4));
        // "At five neither loop is home", and card 4 adds no condition.
        assert!(!labels_home(&p, &TWO_LOOP, 5) && !labels_home(&p, &THREE_LOOP, 5));
        assert_eq!(p[3], 3);
    }

    #[test]
    fn the_second_row_is_neither_the_product_nor_the_sum() {
        let p = second_loops();
        let every: Vec<usize> = (0..6).collect();
        assert_eq!(home_runs(&p, &FOUR_LOOP, STRIP_RUNS), [4, 8, 12]);
        assert_eq!(home_runs(&p, &SIDE_SWAP, STRIP_RUNS), [2, 4, 6, 8, 10, 12]);
        assert_eq!(first_return(&p), 4);
        assert_eq!(lcm(4, 2), 4);
        // Eight is a later return; six is no return.
        assert!(labels_home(&p, &every, 8));
        assert!(!labels_home(&p, &every, 6));
    }

    #[test]
    fn overlapping_cycles_break_the_lcm_rule() {
        let p = overlap_result();
        assert_eq!(p, cycle_images::<3>(&[&[1, 3]]));
        assert_eq!(cycle_tex(p), r"(1\;3)");
        assert_eq!(first_return(&p), 2);
        assert_eq!(lcm(3, 2), 6);
        // The card and 11_17: the first return is two, and six is a return
        // too, the third one, so "not six" would be false.
        let every: Vec<usize> = (0..p.len()).collect();
        assert_eq!(home_runs(&p, &every, lcm(3, 2)), [2, 4, 6]);
    }

    #[test]
    fn order_is_the_lcm_of_disjoint_cycle_lengths_for_every_small_permutation() {
        for n in 1..=6 {
            for p in all_permutations(n) {
                assert_eq!(first_return(&p), order_by_cycles(&p), "{p:?}");
            }
        }
    }

    #[test]
    fn the_swap_lists_land_where_the_cue_says() {
        let long = [EVEN_FIRST, EVEN_SECOND].concat();
        assert_eq!(after_word::<4>(&long), after_word::<4>(&SHORT_ROUTE));
        assert_eq!(cards_by_slot(after_word::<4>(&long)), [4, 3, 2, 1]);
        assert_eq!(cycle_tex(after_word::<4>(&long)), r"(1\;4)(2\;3)");
        assert_eq!(parity_word(swaps_needed(after_word::<4>(&EVEN_FIRST))), "even");
        assert_eq!(parity_word(swaps_needed(after_word::<4>(&EVEN_SECOND))), "even");
        assert_eq!(parity_word(swaps_needed(after_word::<4>(&long))), "even");
        let longer = [long.as_slice(), &[ONE_MORE]].concat();
        assert_eq!(parity_word(swaps_needed(after_word::<4>(&longer))), "odd");
        // "Two odd moves make an even one... builds a three-cycle."
        assert_eq!(cycle_tex(after_word::<4>(&TWO_ODD)), r"(1\;3\;2)");
        assert_eq!(parity_word(swaps_needed(after_word::<4>(&TWO_ODD))), "even");
    }

    #[test]
    fn signs_multiply_and_the_evens_pass_the_subgroup_test() {
        let all = arrangements();
        let parity = |p: [usize; 4]| swaps_needed(p) % 2;
        for &a in &all {
            for &b in &all {
                assert_eq!(parity(then(a, b)), parity_sum(parity(a), parity(b)));
            }
            assert_eq!(parity(way_home(a)), parity(a));
        }
        // The test of 08_4: undo b, then do a.
        let evens = even_arrangements();
        assert!(evens.contains(&[0, 1, 2, 3]));
        for &a in &evens {
            for &b in &evens {
                assert!(evens.contains(&then(way_home(b), a)));
            }
        }
        // The odd ones have no identity, and are not closed either.
        let odds: Vec<[usize; 4]> = all.iter().copied().filter(|&p| parity(p) == 1).collect();
        assert!(!odds.contains(&[0, 1, 2, 3]));
        assert!(odds.iter().any(|&a| !odds.contains(&then(a, a))));
    }

    #[test]
    fn the_extra_swap_pairs_twelve_evens_with_twelve_odds() {
        let all = arrangements();
        let evens = even_arrangements();
        assert_eq!(all.len(), 24);
        assert_eq!(evens.len(), 12);
        let mut partners: Vec<[usize; 4]> = evens.iter().map(|&p| with_first_two_swapped(p)).collect();
        for &p in &evens {
            assert_eq!(with_first_two_swapped(with_first_two_swapped(p)), p);
            assert_eq!(swaps_needed(with_first_two_swapped(p)) % 2, 1);
        }
        partners.sort_unstable();
        partners.dedup();
        assert_eq!(partners.len(), 12);
        assert_eq!(even_kind_counts(), [1, 3, 8]);
        // Nothing even moves exactly two cards, so the three kinds are all of them.
        assert!(
            evens
                .iter()
                .all(|p| (0..4).filter(|&slot| p[slot] != slot).count() != 2)
        );
    }

    #[test]
    fn the_ten_card_shuffle_reads_as_the_spoken_cycles() {
        let deck = perfect_shuffle(DECK_CARDS, true);
        assert_eq!(one_based(&deck), "1,3,5,7,9,2,4,6,8,10");
        assert_eq!(moving_cycles_tex(&deck), r"(2\;3\;5\;9\;8\;6)(4\;7)");
        assert_eq!(deck[0], 0);
        assert_eq!(deck[DECK_CARDS - 1], DECK_CARDS - 1);
        assert_eq!(first_return(&deck), 6);
        assert_eq!(order_by_cycles(&deck), 6);
        // After two the pair is home and "most of the deck" is not.
        let away = out_of_place(&power(&deck, 2));
        assert!(labels_home(&deck, &[3, 6], 2));
        assert_eq!(away.len(), 6);
        assert!(away.len() * 2 > DECK_CARDS);
        assert!(away.iter().all(|&card| ![1, 10, 4, 7].contains(&card)));
    }

    #[test]
    fn the_full_deck_returns_at_eight_and_at_fifty_two() {
        let out = perfect_shuffle(FULL_DECK, true);
        let other = perfect_shuffle(FULL_DECK, false);
        assert_eq!(cycle_lengths(&out), [1, 1, 2, 8, 8, 8, 8, 8, 8]);
        assert_eq!(first_return(&out), 8);
        assert_eq!(cycle_lengths(&other), [52]);
        assert_eq!(first_return(&other), 52);
        assert_eq!(out[0], 0);
        assert_eq!(other[0], 1);
        // "Six and a half times as long."
        assert_eq!(first_return(&other) * 2, first_return(&out) * 13);
        assert_eq!(out_of_place(&power(&other, 8)).len(), FULL_DECK);
        // The one-segment-per-cycle strips fill the whole width.
        assert_eq!(cycle_lengths(&out).iter().sum::<usize>(), FULL_DECK);
        assert_eq!(factorial_scientific(52), (8, 67));
    }

    #[test]
    fn three_stages_restore_eight_wires_and_a_fourth_repeats_the_first() {
        let wires = perfect_shuffle(WIRES, true);
        assert_eq!(moving_cycles(&wires), [vec![2, 3, 5], vec![4, 7, 6]]);
        assert_eq!(first_return(&wires), 3);
        assert_eq!(power(&wires, 4), wires);
    }

    #[test]
    fn seven_more_undo_the_first_and_one_slip_can_never_come_home() {
        let out = perfect_shuffle(FULL_DECK, true);
        assert_eq!(order_by_cycles(&out) - 1, 7);
        assert!(out_of_place(&power(&out, 8)).is_empty());
        let slipped = slipped_run(&out, SLIP_SLOT, 8);
        assert_eq!(out_of_place(&slipped), [11, 37]);
        assert_eq!(parity_of(&slipped), "odd");
        for slot in 0..FULL_DECK - 1 {
            let at = slipped_run(&out, slot, 8);
            assert_eq!(out_of_place(&at).len(), 2, "slot {slot}");
            assert_eq!(parity_of(&at), "odd", "slot {slot}");
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
            ("11_12", 6),
            ("11_13", 6),
            ("11_14", 6),
            ("11_15", 4),
            ("11_16", 4),
            ("11_17", 5),
            ("11_5", 5),
            ("11_6", 3),
            ("11_7", 3),
            ("11_11", 5),
            ("11_8", 5),
            ("11_18", 5),
            ("11_19", 3),
            ("11_20", 8),
            ("11_21", 6),
            ("11_9", 5),
            ("11_10", 8),
            ("11_22", 5),
            ("11_23", 4),
            ("11_24", 4),
            ("11_25", 4),
            ("11_26", 5),
            ("11_27", 6),
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
    fn long_cycles_part_head_from_tail_and_a_swap_keeps_one_arc() {
        // Both cycle shapes the chapter draws: monotone with one return, and the shuffle's trail.
        for cycle in [&[1, 3, 5][..], &[1, 2, 3, 4], &[2, 3, 5, 9, 8, 6]] {
            let offsets = link_offsets(cycle);
            for index in 0..cycle.len() {
                let tail = offsets[index].0;
                let head = offsets[(index + cycle.len() - 1) % cycle.len()].1;
                assert_eq!(tail.abs(), LINK_SEP, "slot {}", cycle[index]);
                assert_eq!(head, -tail, "slot {}", cycle[index]);
            }
        }
        assert_eq!(link_offsets(&[1, 3, 5]), vec![(LINK_SEP, -LINK_SEP); 3]);
        assert_eq!(link_offsets(&[2, 6]), vec![(0.0, 0.0); 2]);
        assert_eq!(link_offsets(&[4, 7]), vec![(0.0, 0.0); 2]);
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

/// The real `pulse` in a real headless scene, read at its real frames: a filled token, a stroked frame, plain
/// text and typeset maths swell and come back, and none of them is ever repainted.
#[cfg(test)]
mod pulse_tests {
    use super::*;
    use fmn::mobject::{Placement, Stage as Arena, Uniforms};

    /// `pulse` runs 0.6 s, and the film renders at 60 fps.
    const FPS: u32 = 60;
    const FRAMES: usize = 36;
    const PEAK: usize = FRAMES / 2 - 1;
    const SWELL: f64 = 1.25;
    const CLOSE: f64 = 1e-5;

    /// Keeps each sampled frame, so a test can read the state the renderer would have been given.
    #[derive(Default)]
    struct Frames(Vec<FramePacket>);

    impl SceneSink for Frames {
        fn capture(&mut self, reason: CaptureReason, packet: FramePacket) -> Result<(), IntegrationError> {
            if reason == CaptureReason::Segment {
                self.0.push(packet);
            }
            Ok(())
        }
    }

    struct Probe<F>(F);

    impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            (self.0)(stage)
        }
    }

    fn headless(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) -> Vec<FramePacket> {
        let config = RuntimeConfig {
            fps: FPS,
            ..RuntimeConfig::default()
        };
        let mut frames = Frames::default();
        fmn::run_scene(&mut Probe(f), config, 7, &mut frames).expect("the headless scene runs");
        frames.0
    }

    /// Every colour column of one member, found the way `indicate` finds what it repaints.
    fn colours(arena: &Arena, mob: Mob) -> Vec<(String, Vec<f32>)> {
        let buffer = &arena.get(mob).expect("a live member").buffer;
        buffer
            .schema()
            .fields()
            .iter()
            .filter(|field| field.name.ends_with("rgba"))
            .map(|field| {
                (
                    field.name.clone(),
                    buffer.read_column(&field.name).expect("a colour column"),
                )
            })
            .collect()
    }

    /// All a pulse must hand back unchanged on one member of a family.
    #[derive(Debug, PartialEq)]
    struct Member {
        mob: Mob,
        placement: Option<Placement>,
        uniforms: Option<Uniforms>,
        z_index: i32,
        colours: Vec<(String, Vec<f32>)>,
    }

    fn look(arena: &Arena, roots: &[Mob]) -> Vec<Member> {
        roots
            .iter()
            .flat_map(|&root| arena.family(root))
            .map(|mob| Member {
                mob,
                placement: arena.placement(mob),
                uniforms: arena.uniforms(mob).copied(),
                z_index: arena.z_index(mob),
                colours: colours(arena, mob),
            })
            .collect()
    }

    struct Run {
        roots: [Mob; 4],
        before: Vec<Member>,
        /// Each root's width, height and centre before the pulse.
        boxes: Vec<(f64, f64, [f64; 3])>,
        after: Vec<Member>,
        frames: Vec<FramePacket>,
    }

    /// The chapter's four kinds of pulsed mobject, pulsed together.
    fn pulse_each_kind() -> Run {
        let mut made = None;
        let frames = headless(|stage| {
            let k = Kit::new()?;
            let token = card(stage, &k, 3, (-4.5, 1.0), 0.37)?;
            let frame = chip(stage, &k, 2, (-1.5, 1.0))?.frame;
            let text = k.text(stage, "Swaps: 0", 26.0, Palette::ink(), v(1.5, 1.0))?;
            let maths = k.tex(stage, r"(1\;3\;4)", 34.0, Palette::green(), &[], v(4.5, 1.0))?;
            let roots = [token, frame, text, maths];
            for mob in roots {
                stage.arena_mut().add_to_scene(mob)?;
            }
            let arena = stage.arena();
            let boxes = roots
                .iter()
                .map(|&mob| (arena.get_width(mob), arena.get_height(mob), arena.get_center(mob)))
                .collect();
            let before = look(arena, &roots);
            pulse(stage, &roots)?;
            made = Some((roots, before, boxes, look(stage.arena(), &roots)));
            Ok(())
        });
        let (roots, before, boxes, after) = made.expect("the scene ran");
        Run {
            roots,
            before,
            boxes,
            after,
            frames,
        }
    }

    #[test]
    fn a_pulse_peaks_at_its_factor_halfway_and_keeps_each_centre() {
        let run = pulse_each_kind();
        assert_eq!(run.frames.len(), FRAMES);
        assert_eq!(run.frames[PEAK].alpha(), 0.5);
        let arena = run.frames[PEAK].materialize_stage();
        for (&root, &(width, height, centre)) in run.roots.iter().zip(&run.boxes) {
            assert!((arena.get_width(root) - SWELL * width).abs() < CLOSE);
            assert!((arena.get_height(root) - SWELL * height).abs() < CLOSE);
            let at = arena.get_center(root);
            assert!(
                (0..3).all(|i| (at[i] - centre[i]).abs() < CLOSE),
                "{at:?} left {centre:?}"
            );
        }
        let widths: Vec<f64> = run
            .frames
            .iter()
            .map(|frame| frame.materialize_stage().get_width(run.roots[0]))
            .collect();
        assert!(
            widths[..=PEAK].windows(2).all(|w| w[0] < w[1]),
            "rises to the peak: {widths:?}"
        );
        assert!(
            widths[PEAK..].windows(2).all(|w| w[0] > w[1]),
            "falls from the peak: {widths:?}"
        );
    }

    #[test]
    fn a_pulse_never_repaints_any_member_on_any_frame() {
        let run = pulse_each_kind();
        for frame in &run.frames {
            let arena = frame.materialize_stage();
            for member in &run.before {
                assert_eq!(
                    colours(&arena, member.mob),
                    member.colours,
                    "frame {}",
                    frame.segment_frame()
                );
            }
        }
        // The repair's purpose: at the peak a card's numeral is still another colour from its disk.
        let arena = run.frames[PEAK].materialize_stage();
        let fills: Vec<Vec<f32>> = arena
            .family(run.roots[0])
            .into_iter()
            .filter_map(|mob| arena.get(mob)?.buffer.read_column("fill_rgba"))
            .filter(|column| !column.is_empty())
            .map(|column| column[..3].to_vec())
            .collect();
        assert!(
            fills.iter().any(|fill| *fill != fills[0]),
            "the numeral merged into its disk: {fills:?}"
        );
    }

    #[test]
    fn a_pulse_ends_exactly_where_it_began() {
        let run = pulse_each_kind();
        let last = run.frames.last().expect("frames");
        assert_eq!(last.alpha(), 1.0);
        assert_eq!(look(&last.materialize_stage(), &run.roots), run.before);
        assert_eq!(run.after, run.before);
    }

    #[test]
    fn a_pulsed_token_can_still_be_recoloured_moved_and_pulsed_again() {
        let mut checked = false;
        headless(|stage| {
            let k = Kit::new()?;
            let token = card(stage, &k, 3, (-3.0, 1.0), 0.37)?;
            stage.arena_mut().add_to_scene(token)?;
            let paint =
                |arena: &Arena| -> Vec<_> { arena.family(token).into_iter().map(|m| colours(arena, m)).collect() };
            let painted = paint(stage.arena());
            pulse(stage, &[token])?;
            recolor(stage, &[token], Palette::red(), 0.1)?;
            travel(stage, &[(token, (2.0, -1.0))], 0.0, 0.1)?;
            assert_ne!(paint(stage.arena()), painted, "the pulse left its colours locked");
            let centre = stage.arena().get_center(token);
            assert!(
                (centre[0] - 2.0).abs() < CLOSE && (centre[1] + 1.0).abs() < CLOSE,
                "{centre:?}"
            );
            let settled = look(stage.arena(), &[token]);
            pulse(stage, &[token])?;
            assert_eq!(
                look(stage.arena(), &[token]),
                settled,
                "a second pulse must not compound"
            );
            checked = true;
            Ok(())
        });
        assert!(checked);
    }
}
