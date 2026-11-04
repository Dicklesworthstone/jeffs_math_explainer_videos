//! Expanded treatment, in development. This is not a complete video.
mod applications;
mod arithmetic;
mod axioms;
mod centralizer;
mod composition;
mod cosets;
mod cyclic;
mod d4;
mod d4_group;
mod eight_construction;
mod equivalence;
mod isomorphism;
mod opening;
mod permutations;
mod products;
mod quotients;
mod subgroups;
mod tile_actor;
mod tile_model;
mod transitivity;

use explainer_kit::{Chapter, Explainer, Facts};
use explainer_score::instr::Instrument;
use explainer_score::score::{BASS_UNDER_PAD_DB, Identity, Palette};
use explainer_score::theory::{AEOLIAN, Chord, Key, Motif};

const LANE_FOUR: [i32; 4] = [4, 16, 28, -8];
const LANE_FIVE: [i32; 2] = [5, 17];
const COUNTEREXAMPLE: [i32; 3] = [0, 1, 2];

fn chapters() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "05_order",
            make: |kit| Box::new(composition::Composition { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "07_eight_moves",
            make: |kit| Box::new(eight_construction::EightConstruction { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "13_arithmetic",
            make: |kit| Box::new(arithmetic::Arithmetic { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "01_clock",
            make: |kit| Box::new(opening::ClockOpening { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "06_axioms",
            make: |kit| Box::new(axioms::Axioms { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "08_subgroups",
            make: |kit| Box::new(subgroups::Subgroups { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "09_centralizer",
            make: |kit| Box::new(centralizer::Centralizer { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "10_cyclic",
            make: |kit| Box::new(cyclic::Cyclic { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "11_permutations",
            make: |kit| Box::new(permutations::Permutations { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "12_isomorphism",
            make: |kit| Box::new(isomorphism::Isomorphism { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_equivalence",
            make: |kit| Box::new(equivalence::Equivalence { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_counterexample",
            make: |kit| Box::new(transitivity::Counterexample { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "14_cosets",
            make: |kit| Box::new(cosets::Cosets { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "15_products",
            make: |kit| Box::new(products::Products { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "16_quotients",
            make: |kit| Box::new(quotients::Quotients { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "17_applications",
            make: |kit| Box::new(applications::Applications { kit }),
            intensity: 0.4,
        },
    ]
}

fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    facts.add("clock.modulus", 12)?;
    facts.add("clock.start", 11)?;
    facts.add("clock.advance", 5)?;
    facts.add("clock.sum", 11 + 5)?;
    facts.add("clock.position", (11_i32 + 5).rem_euclid(12))?;
    for (index, value) in LANE_FOUR.iter().enumerate() {
        facts.add(&format!("lane.four.{index}"), *value)?;
    }
    for (index, value) in LANE_FIVE.iter().enumerate() {
        facts.add(&format!("lane.five.{index}"), *value)?;
    }
    for (index, value) in COUNTEREXAMPLE.iter().enumerate() {
        facts.add(&format!("close.example.{index}"), *value)?;
    }
    facts.add("square.rotation_order", 4)?;
    let corners = [(1, 1), (1, -1), (-1, -1), (-1, 1)];
    let symmetries: std::collections::BTreeSet<Vec<(i32, i32)>> = (0..2)
        .flat_map(|flip| (0..4).map(move |turns| (turns, flip == 1)))
        .map(|(turns, flip)| corners.iter().map(|&p| d4::apply(p, turns, flip)).collect())
        .collect();
    facts.add("square.symmetry_count", symmetries.len())?;
    facts.add("order.turn_then_flip.1.x", -1)?;
    facts.add("order.turn_then_flip.1.y", -1)?;
    facts.add("order.flip_then_turn.1.x", 1)?;
    facts.add("order.flip_then_turn.1.y", 1)?;
    permutations::add_facts(&mut facts)?;
    rebuild_facts(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

fn listed<T: std::fmt::Display>(values: impl IntoIterator<Item = T>) -> String {
    values
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn move_names(moves: &[d4_group::Elem]) -> String {
    listed(moves.iter().map(|g| g.tex()))
}

/// What the rebuilt chapters 03, 06, 08, 09, 14, 16 and 17 say or show,
/// computed by the same helpers their scenes draw with; oracle.py recomputes
/// each from integer arithmetic, corner maps and parity checks.
fn rebuild_facts(facts: &mut Facts) -> Result<(), String> {
    use d4_group::{ALL, Elem, R, R2, S};

    // 03: closeness groups overlap without coinciding; fractions as classes.
    let near_one = transitivity::close_to(1);
    let near_zero = transitivity::close_to(0);
    facts.add("close.near_one", listed(&near_one))?;
    facts.add("close.near_zero", listed(&near_zero))?;
    facts.add(
        "close.shared",
        near_one.iter().filter(|n| near_zero.contains(n)).count(),
    )?;
    facts.add(
        "fraction.half_meets_two_quarters",
        transitivity::class_in_lattice(1, 2).contains(&(4, 2)),
    )?;

    // 06: the clock examples and the marked-square motions, from the constants
    // and step lists the scene animates.
    {
        use axioms::{
            CLOSURE, GROUPED, LAPS, MODULUS, REGROUP, REVERSE_UNDO, SAME_ORDER_UNDO, SEQ_A, SEQ_B, SOLVE, SOLVE_A,
            SOLVE_B, START, SWAPPED, Step, WRAP, WRONG_SIDE, clock_add, clock_inverse, steps_product,
        };
        let product = |steps: &[Step]| steps_product(steps).tex();
        let (a, b) = CLOSURE;
        facts.add("axioms.closure.moves", listed([a, b]))?;
        facts.add("axioms.closure.result", clock_add(a, b))?;
        let (p, q) = WRAP;
        facts.add("axioms.wrap.moves", listed([p, q]))?;
        facts.add("axioms.wrap.total", p + q)?;
        facts.add("axioms.wrap.position", clock_add(p, q))?;
        let [x, y, z] = GROUPED;
        facts.add("axioms.grouped.moves", listed(GROUPED))?;
        facts.add("axioms.grouped.first_two", clock_add(x, y))?;
        facts.add("axioms.grouped.last_two", clock_add(y, z))?;
        facts.add("axioms.grouped.total", x + y + z)?;
        facts.add("axioms.grouped.position", clock_add(clock_add(x, y), z))?;
        facts.add("axioms.grouped.position_regrouped", clock_add(x, clock_add(y, z)))?;
        let [u, w, t] = LAPS;
        facts.add("axioms.laps.moves", listed(LAPS))?;
        facts.add("axioms.laps.total", u + w + t)?;
        facts.add("axioms.laps.full_laps", (u + w + t) / MODULUS)?;
        facts.add("axioms.laps.first_two_dropped", clock_add(u, w))?;
        facts.add("axioms.laps.then_last", clock_add(u, w) + t)?;
        facts.add("axioms.laps.position", clock_add(clock_add(u, w), t))?;
        facts.add("axioms.identity.start", START)?;
        facts.add("axioms.identity.result", clock_add(START, 0))?;
        facts.add("axioms.inverse.of_start", clock_inverse(START))?;
        facts.add("axioms.inverse.result", clock_add(START, clock_inverse(START)))?;
        facts.add("axioms.regroup.product", product(&REGROUP))?;
        facts.add("axioms.swapped.product", product(&SWAPPED))?;
        facts.add("axioms.solve.a", SOLVE_A.tex())?;
        facts.add("axioms.solve.b", SOLVE_B.tex())?;
        facts.add("axioms.solve.x", product(&SOLVE[..2]))?;
        facts.add("axioms.solve.check", product(&SOLVE))?;
        facts.add("axioms.wrong_side.candidate", product(&WRONG_SIDE[..2]))?;
        facts.add("axioms.wrong_side.check", product(&WRONG_SIDE))?;
        facts.add("axioms.undo.a", SEQ_A.tex())?;
        facts.add("axioms.undo.b", SEQ_B.tex())?;
        facts.add("axioms.undo.reverse_order", product(&REVERSE_UNDO))?;
        facts.add("axioms.undo.same_order", product(&SAME_ORDER_UNDO))?;
    }

    // 08: the closure machine's passes and the ten subgroups.
    for (name, generators) in [
        ("flip", &subgroups::FLIP_ALONE[..]),
        ("flip_half_turn", &subgroups::FLIP_AND_HALF_TURN[..]),
        ("quarter_turn_flip", &subgroups::QUARTER_TURN_AND_FLIP[..]),
    ] {
        let rounds = d4_group::closure_rounds(generators);
        facts.add(
            &format!("closure.{name}.new_per_pass"),
            listed(rounds[1..].iter().map(Vec::len)),
        )?;
        facts.add(
            &format!("closure.{name}.size"),
            rounds.iter().map(Vec::len).sum::<usize>(),
        )?;
    }
    let all = d4_group::subgroups();
    facts.add("subgroups.count", all.len())?;
    for size in 1..=ALL.len() {
        facts.add(
            &format!("subgroups.of_size.{size}"),
            all.iter().filter(|h| h.len() == size).count(),
        )?;
    }

    // 09: the centralizer of the flip, the carried mirror line, the center.
    let centralizer = d4_group::centralizer(S);
    facts.add("centralizer.s.members", move_names(&centralizer))?;
    facts.add("centralizer.s.checks", centralizer.len())?;
    facts.add("centralizer.s.crosses", ALL.len() - centralizer.len())?;
    for g in ALL {
        facts.add(&format!("mirror.carry.{}", g.tex()), g.carry_line(0))?;
    }
    let returned: Vec<Elem> = ALL.into_iter().filter(|g| g.carry_line(0) == 0).collect();
    facts.add("mirror.returned_by", move_names(&returned))?;
    facts.add("center.members", move_names(&d4_group::center()))?;

    // 14: representatives on the clock, left and right cosets in the square.
    for (name, (g, k)) in [("same", cosets::SAME_COSET), ("other", cosets::OTHER_COSET)] {
        facts.add(&format!("coset.clock.{name}.step"), (k - g).rem_euclid(12))?;
        facts.add(&format!("coset.clock.{name}.in_h"), cosets::same_clock_coset(g, k))?;
    }
    let clock_cosets: std::collections::BTreeSet<Vec<i32>> = (0..12)
        .map(|n| {
            let mut coset: Vec<i32> = cosets::CLOCK_H.iter().map(|h| (n + h) % 12).collect();
            coset.sort_unstable();
            coset
        })
        .collect();
    facts.add("coset.clock.count", clock_cosets.len())?;
    facts.add("coset.clock.size", cosets::CLOCK_H.len())?;
    let mut left = cosets::left_cosets(&cosets::FLIP_H);
    let mut right = cosets::right_cosets(&cosets::FLIP_H);
    facts.add("coset.flip.left_count", left.len())?;
    facts.add("coset.flip.right_count", right.len())?;
    facts.add(
        "coset.flip.r_left",
        move_names(&d4_group::left_coset(R, &cosets::FLIP_H)),
    )?;
    facts.add(
        "coset.flip.r_right",
        move_names(&d4_group::right_coset(R, &cosets::FLIP_H)),
    )?;
    left.sort();
    right.sort();
    facts.add("coset.flip.cuts_agree", left == right)?;

    // 16: the four-product failure and the two sorted tables.
    let rh = d4_group::left_coset(R, &quotients::FLIP_H);
    let products: Vec<Elem> = rh.iter().flat_map(|&a| rh.iter().map(move |&b| a.mul(b))).collect();
    let r2h = d4_group::left_coset(R2, &quotients::FLIP_H);
    facts.add("quotient.rh_products.count", products.len())?;
    facts.add(
        "quotient.rh_products.in_r2h",
        products.iter().filter(|p| r2h.contains(p)).count(),
    )?;
    facts.add(
        "quotient.rh_products.in_h",
        products.iter().filter(|p| quotients::FLIP_H.contains(p)).count(),
    )?;
    facts.add("quotient.table.entries", ALL.len() * ALL.len())?;
    let flip_parity = |g: Elem| u8::from(g.flip);
    let rotation_blocks = (ALL.len() / 4).pow(2);
    let rotation_mixed = quotients::mixed_blocks(&quotients::ROTATION_ORDER, 4, flip_parity).len();
    facts.add("quotient.rotations.solid_blocks", rotation_blocks - rotation_mixed)?;
    facts.add("quotient.flip_h.blocks", (ALL.len() / 2).pow(2))?;
    let coset_column = |g: Elem| g.turns;
    facts.add(
        "quotient.flip_h.mixed_blocks",
        quotients::mixed_blocks(&quotients::COSET_ORDER, 2, coset_column).len(),
    )?;

    // 17: Hamming(7,4) as a subgroup, its cosets, one repair, one miscorrection.
    use applications::{SENT, binary3, bits, flip, syndrome};
    let checks = applications::CHECK_POSITIONS.len();
    let length = (1_usize << checks) - 1;
    let codewords = applications::codewords();
    let mut per_coset = std::collections::BTreeMap::new();
    for word in 0..(1_u16 << length) {
        *per_coset
            .entry(syndrome(u8::try_from(word).expect("a seven-bit word")))
            .or_insert(0_usize) += 1;
    }
    let positions: Vec<u8> = (1..).take(length).collect();
    let situations: Vec<u8> = std::iter::once(SENT)
        .chain(positions.iter().map(|&p| flip(SENT, p)))
        .collect();
    let labels: std::collections::BTreeSet<u8> = situations.iter().map(|&w| syndrome(w)).collect();
    facts.add("hamming.check_bits", checks)?;
    facts.add("hamming.length", length)?;
    facts.add("hamming.data_bits", length - checks)?;
    facts.add("hamming.words", 1_usize << length)?;
    facts.add("hamming.codewords", codewords.len())?;
    facts.add("hamming.cosets", per_coset.len())?;
    facts.add(
        "hamming.coset_sizes",
        listed(per_coset.values().collect::<std::collections::BTreeSet<_>>()),
    )?;
    facts.add("hamming.situations", situations.len())?;
    facts.add("hamming.distinct_labels", labels.len())?;
    facts.add("hamming.sent", bits(SENT))?;
    let received = flip(SENT, applications::SINGLE_ERROR);
    facts.add("hamming.single.position", applications::SINGLE_ERROR)?;
    facts.add("hamming.single.received", bits(received))?;
    facts.add("hamming.single.syndrome", binary3(syndrome(received)))?;
    facts.add("hamming.single.syndrome_value", syndrome(received))?;
    facts.add("hamming.single.repaired", bits(flip(received, syndrome(received))))?;
    let garbled = applications::DOUBLE_ERROR.iter().fold(SENT, |word, &p| flip(word, p));
    let label = syndrome(garbled);
    let decoded = flip(garbled, label);
    facts.add("hamming.double.positions", listed(applications::DOUBLE_ERROR))?;
    facts.add("hamming.double.received", bits(garbled))?;
    facts.add("hamming.double.syndrome", binary3(label))?;
    facts.add("hamming.double.syndrome_value", label)?;
    facts.add("hamming.double.decoded", bits(decoded))?;
    facts.add("hamming.double.decoded_is_codeword", codewords.contains(&decoded))?;
    facts.add("hamming.double.decoded_is_sent", decoded == SENT)?;
    Ok(())
}

fn identity() -> Identity {
    Identity {
        name: "abstract_algebra_expanded".into(),
        key: Key::new(9, AEOLIAN),
        tonic_midi: 57,
        progression: vec![Chord::min9(9), Chord::maj9(5), Chord::maj9(0), Chord::dom7sus4(4)],
        motif: Motif::new(&[1, 3, 2, 1], &[0.5, 0.5, 1.0, 1.5]),
        resolution: Motif::new(&[3, 2, 1], &[0.75, 0.75, 2.0]),
        beat_s: 60.0 / 74.0,
        palette: Palette {
            bed: Instrument::Pad { brightness: 0.25 },
            melody: Instrument::Marimba,
            answer: Instrument::Vibraphone,
            roll: Some(Instrument::FeltPiano),
        },
        chord_s: (6.0, 12.0),
        bed_t60: 1.8,
        bed_lu: -13.0,
        bass_db: BASS_UNDER_PAD_DB,
        melody_lu: -13.5,
        duck_db: 8.0,
        hold_duck: 0.5,
        steady_bed: true,
        settle_s: 0.8,
        sfx_lu: -16.0,
        listen_lu: -10.0,
        swing: 0.5,
        moods: Vec::new(),
        listen_chord: None,
        themes: Vec::new(),
        theme_chord: None,
    }
}

const EXPLAINER: Explainer = Explainer {
    crate_name: "abstract_algebra_expanded",
    script: include_str!("../script.tsv"),
    chapters,
    facts,
    glyph_text: &[
        "Arithmetic with actions",
        "Same sequence; different packaging",
        "x  →  h(x)  →  g(h(x))  →  f(g(h(x)))",
        "Closure: the result stays in the rack",
        "Associativity: package, do not reorder",
        "Identity: leave the tile alone",
        "Inverse: restore every corner",
        "Stay inside the calculation",
        "Bundle a long instruction",
        "Do nothing",
        "Undo to solve",
        "11 o'clock + 5 hours",
        "Where will the marker land?",
        "16 and 4: same clock position",
        "Equal as integers?",
        "Clock position",
        "Reflexive · symmetric · transitive",
        "4 16 28 −8 5 17",
        "Difference is a multiple of 12",
        "Why this makes equivalence classes",
        "Reflexive: 4 − 4 = 0",
        "Symmetric: 16 − 4 = 12; 4 − 16 = −12",
        "Transitive: (28 − 16) + (16 − 4) = 28 − 4",
        "12 + 12 = 24",
        "Close means at most 1 apart",
        "Must 0 be close to 2?",
        "0 and 2 are 2 apart",
        "Not transitive",
        "Mark the corners",
        "How many quarter-turns return every corner?",
        "4 quarter-turns = 1 full turn",
        "Turn then flip",
        "Flip then turn",
        "Same result?",
        "Where will the blue corner finish?",
        "The order matters",
        "What makes a group?",
        "Clock moves: add modulo 12",
        "Closure: 3 + 4 = 7",
        "Associativity: (3 + 4) + 5 = 3 + (4 + 5)",
        "Identity: 3 + 0 = 3",
        "Inverse: 3 + 9 = 0 (mod 12)",
        "Commutativity is optional",
        "Closure: combine two clock moves",
        "Two ways to group the same moves",
        "(3 + 4) + 5",
        "3 + (4 + 5)",
        "Same destination: 0",
        "Identity and inverse",
        "3 + 0 = 3",
        "3 + 9 = 0 (mod 12)",
        "Example, not a proof for every move",
        "The square has eight symmetries",
        "Could there be a ninth move?",
        "Track one corner and its neighbor",
        "Rotations",
        "Reflections",
        "e",
        "r",
        "r²",
        "r³",
        "s",
        "rs",
        "r²s",
        "r³s",
        "Rightmost move first",
        "4 choices × 2 neighbors = 8",
        "Corner 1: four choices",
        "Neighbor 2: either orange point",
        "A group inside a group",
        "the four rotations",
        "Closed under composition?",
        "r * r² = r³ is outside",
        "Nonempty H: a, b in H -> a b^(-1) in H",
        "h, h -> h h^(-1) = e in H",
        "e, h -> e h^(-1) = h^(-1) in H",
        "a, b^(-1) -> a (b^(-1))^(-1) = ab in H",
        "Identity",
        "Inverses",
        "Closure",
        "Associativity is inherited",
        "Which moves commute with the flip?",
        "Move g, then flip s",
        "Flip s, then move g",
        "Same result",
        "Different results",
        "Centralizer of s: e, r², s, r²s",
        "a and b commute with s -> a b^(-1) does too",
        "Center: commutes with EVERY move: e, r²",
        "One move, repeated",
        "Add 8: how many positions?",
        "Every subgroup of a cyclic group is cyclic",
        "No element generates all four",
        "Multiply modulo 12",
        "Permutations: follow the labels",
        "Separate loops do not interfere",
        "A length-m cycle can be built from m − 1 swaps",
        "First factor reverses; the other two exchange",
        "Can the same move be both even and odd?",
        "4 corners: 24 permutations, 8 square symmetries",
        "Quarter-turn: 4-cycle, odd",
        "Half-turn: two swaps, even",
        "N = number of swaps",
        "Same structure, different objects",
        "Clock moves modulo 4",
        "Square rotations",
        "2 + 3 on the four-clock?",
        "Bijection + operation preservation",
        "The first positive return is preserved",
        "Square rotation: an element of order 4",
        "U(12): no element has order 4",
        "An automorphism: structure-preserving relabelling",
        "0123456789−",
    ],
    glyph_tex: &[
        "r^4=e",
        "s^2=e",
        "srs=r^{-1}",
        "s r s r^2 s",
        "r^{-1} r^2 s",
        r"r^k\quad\mathrm{or}\quad r^k s",
        "r^3r=e",
        r"\text{Nonempty }H:\quad a,b\in H\Rightarrow ab^{-1}\in H",
        r"h,h\Rightarrow hh^{-1}=e\in H",
        r"e,h\Rightarrow eh^{-1}=h^{-1}\in H",
        r"a,b^{-1}\Rightarrow a(b^{-1})^{-1}=ab\in H",
        r"bs=sb\Rightarrow b^{-1}s=sb^{-1}",
        r"(ab^{-1})s=a(b^{-1}s)=a(sb^{-1})=(as)b^{-1}=s(ab^{-1})",
        r"\langle 8\rangle=\{0,8,4\},\quad \operatorname{ord}(8)=3",
        r"0\rightarrow8\rightarrow4\rightarrow0",
        r"\langle 8\rangle=\{0,8,4\}",
        r"\operatorname{ord}(8)=3",
        r"8m\equiv 0\pmod {12}",
        r"n=dn',\quad k=dk',\quad \gcd(n',k')=1",
        r"n\mid mk\Longleftrightarrow n'\mid mk'\Longrightarrow n'\mid m",
        r"\operatorname{ord}(k)=\frac{n}{\gcd(n,k)}",
        r"\gcd(k,12)=1:\quad k=1,5,7,11",
        r"d=\min\{j>0:g^j\in H\}",
        r"m=qd+r,\qquad 0\le r<d",
        r"g^r=g^m(g^d)^{-q}\in H\quad\Longrightarrow\quad r=0",
        r"U(12)=\{1,5,7,11\}",
        r"5^2\equiv7^2\equiv11^2\equiv1\pmod{12}",
        r"(1\;3\;4)=(3\;4\;1)",
        r"(1\;2\;3)(1\;2)\quad\text{rightmost first}",
        r"1\xrightarrow{(1\;2)}2\xrightarrow{(1\;2\;3)}3",
        r"3\xrightarrow{(1\;2)}3\xrightarrow{(1\;2\;3)}1",
        r"2\xrightarrow{(1\;2)}1\xrightarrow{(1\;2\;3)}2",
        r"(1\;3\;4)^{-1}=(1\;4\;3)",
        r"(1\;3\;4)=(1\;4)(1\;3)",
        r"\Delta(x)=\prod_{i<j}(x_j-x_i)",
        r"(x_2-x_1)(x_3-x_1)(x_3-x_2)",
        r"(x_1-x_2)(x_3-x_2)(x_3-x_1)",
        r"\text{one swap}:\quad \Delta\mapsto-\Delta",
        r"\Delta_{\text{final}}=(-1)^N\Delta_{\text{initial}}",
        r"\downarrow",
        r"2+3\equiv1\pmod4\quad\longleftrightarrow\quad r^2r^3=r",
        r"\phi(k)=r^k",
        r"\phi(a+b)=r^{a+b}=r^ar^b=\phi(a)\phi(b)",
        r"r^4=e",
        r"g^m=e\quad\Longleftrightarrow\quad\phi(g)^m=e",
        r"L_g(x)=gx,\qquad L_gL_h=L_{gh}",
        r"L_1:\quad 0\rightarrow1\rightarrow2\rightarrow3\rightarrow0",
        r"L_1(L_2(0))=3",
        r"L_{g^{-1}}L_g=\operatorname{id}",
        r"L_g(e)=g\quad\Longrightarrow\quad L_g=L_h\Rightarrow g=h",
        r"k\longmapsto-k\pmod 4",
    ],
    glyph_mono: &[],
    identity,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_gaps: explainer_kit::narration::LineGaps::Constant,
    line_owner: None,
    thumbnail: None,
};

fn main() {
    std::process::exit(explainer_kit::cli::run_with_contract(
        &EXPLAINER,
        std::env::args().skip(1),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    ));
}

#[cfg(test)]
mod tests {
    #[test]
    fn clock_to_rotation_map_preserves_all_four_by_four_products() {
        use crate::d4::apply;
        let p = (1, 1);
        for a in 0..4 {
            for b in 0..4 {
                assert_eq!(apply(apply(p, b, false), a, false), apply(p, (a + b) % 4, false));
            }
        }
    }
    #[test]
    fn permutation_composition_examples_use_rightmost_first() {
        let compose = |a: [usize; 4], b: [usize; 4]| b.map(|i| a[i]);
        assert_eq!(compose([1, 2, 0, 3], [1, 0, 2, 3]), [2, 1, 0, 3]);
        assert_eq!(compose([3, 1, 2, 0], [2, 1, 0, 3]), [2, 1, 3, 0]);
    }
    #[test]
    fn clock_orders_and_noncyclic_units_match_examples() {
        fn gcd(mut a: usize, mut b: usize) -> usize {
            while b != 0 {
                (a, b) = (b, a % b);
            }
            a
        }
        for n in 1..=60 {
            for k in 0..n {
                let first_return = (1..=n).find(|m| (m * k) % n == 0).unwrap();
                assert_eq!(first_return, n / gcd(n, k));
            }
        }
        let units = [1, 5, 7, 11];
        for a in units {
            assert_eq!(a * a % 12, 1);
            for b in units {
                assert!(units.contains(&(a * b % 12)));
            }
        }
    }
    #[test]
    fn clock_lands_at_four_after_five_steps_from_eleven() {
        let visited: Vec<i32> = (0..=5).map(|step| (11_i32 + step).rem_euclid(12)).collect();
        assert_eq!(visited, [11, 0, 1, 2, 3, 4]);
        assert_eq!(16_i32.rem_euclid(12), 4);
        assert_ne!(16, 4);
    }

    #[test]
    fn proposed_lanes_match_modulo_twelve() {
        assert!(super::LANE_FOUR.iter().all(|n| n.rem_euclid(12) == 4));
        assert!(super::LANE_FIVE.iter().all(|n| n.rem_euclid(12) == 5));
        assert_ne!(4_i32.rem_euclid(12), 5_i32.rem_euclid(12));
    }

    #[test]
    fn closeness_counterexample_really_fails_transitivity() {
        let close = |a: i32, b: i32| (a - b).abs() <= 1;
        let [a, b, c] = super::COUNTEREXAMPLE;
        assert!(close(a, b));
        assert!(close(b, c));
        assert!(!close(a, c));
    }

    #[test]
    fn quarter_turn_has_order_four_on_each_corner() {
        for original in [(1_i32, 1_i32), (1, -1), (-1, -1), (-1, 1)] {
            let mut point = original;
            for turn in 1..=4 {
                point = (-point.1, point.0);
                assert_eq!(point == original, turn == 4);
            }
        }
    }

    #[test]
    fn clock_addition_satisfies_group_axioms() {
        let add = |a, b| (a + b) % 12;
        for a in 0..12 {
            assert_eq!(add(a, 0), a);
            assert_eq!(add(0, a), a);
            assert_eq!(add(a, (12 - a) % 12), 0);
            for b in 0..12 {
                assert!((0..12).contains(&((a + b) % 12)));
                for c in 0..12 {
                    assert_eq!(((a + b) % 12 + c) % 12, (a + (b + c) % 12) % 12);
                }
            }
        }
    }

    #[test]
    fn square_symmetries_are_eight_distinct_closed_permutations() {
        let corners = [(1, 1), (1, -1), (-1, -1), (-1, 1)];
        let maps: Vec<Vec<(i32, i32)>> = (0..2)
            .flat_map(|flip| (0..4).map(move |turns| (turns, flip == 1)))
            .map(|(turns, flip)| corners.iter().map(|&p| super::d4::apply(p, turns, flip)).collect())
            .collect();
        let distinct: std::collections::HashSet<_> = maps.iter().collect();
        assert_eq!(distinct.len(), 8);
        for first in &maps {
            for second in &maps {
                let composed: Vec<_> = second
                    .iter()
                    .map(|p| first[corners.iter().position(|q| q == p).unwrap()])
                    .collect();
                assert!(maps.contains(&composed));
            }
        }
    }

    #[test]
    fn subgroup_test_matches_full_axioms_for_every_nonempty_square_subset() {
        let corners = [(1, 1), (1, -1), (-1, -1), (-1, 1)];
        let maps: Vec<Vec<(i32, i32)>> = (0..2)
            .flat_map(|flip| (0..4).map(move |turns| (turns, flip == 1)))
            .map(|(turns, flip)| corners.iter().map(|&p| super::d4::apply(p, turns, flip)).collect())
            .collect();
        let mut product = [[0_usize; 8]; 8];
        for a in 0..8 {
            for b in 0..8 {
                let composed: Vec<_> = maps[b]
                    .iter()
                    .map(|p| maps[a][corners.iter().position(|q| q == p).unwrap()])
                    .collect();
                product[a][b] = maps.iter().position(|p| *p == composed).unwrap();
            }
        }
        let inverses: Vec<_> = (0..8)
            .map(|a| (0..8).find(|&b| product[a][b] == 0 && product[b][a] == 0).unwrap())
            .collect();
        for mask in 1_u16..256 {
            let includes = |a: usize| (mask & (1 << a)) != 0;
            let full = includes(0)
                && (0..8).all(|a| {
                    !includes(a) || (includes(inverses[a]) && (0..8).all(|b| !includes(b) || includes(product[a][b])))
                });
            let test =
                (0..8).all(|a| !includes(a) || (0..8).all(|b| !includes(b) || includes(product[a][inverses[b]])));
            assert_eq!(test, full, "subset mask {mask}");
        }
        assert_eq!(product[1][2], 3);
    }

    #[test]
    fn turn_and_horizontal_reflection_do_not_commute() {
        let turn = |(x, y): (i32, i32)| (-y, x);
        let flip = |(x, y): (i32, i32)| (x, -y);
        assert_eq!(flip(turn((1, 1))), (-1, -1));
        assert_eq!(turn(flip((1, 1))), (1, 1));
    }
    #[test]
    fn horizontal_reflection_centralizer_has_exactly_four_members() {
        let actual: Vec<(u8, bool)> = (0..2)
            .flat_map(|flip| (0..4).map(move |turns| (turns, flip == 1)))
            .filter(|&(turns, flip)| crate::centralizer::commutes_with_flip(turns, flip))
            .collect();
        assert_eq!(actual, vec![(0, false), (2, false), (0, true), (2, true)]);
    }
}
