"""Build the candidate from the preserved baseline. Run from the repo root.

Every edit names exact text that must occur exactly once in the baseline.
"""
import os
import shutil

C = "explainer_corpus/abstract_algebra_expanded/src/permutations.rs"
O = "docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-picture-review-11_permutations/"

os.makedirs(O + "baseline", exist_ok=True)
os.makedirs(O + "candidate", exist_ok=True)
if not os.path.exists(O + "baseline/permutations.rs"):
    shutil.copyfile(C, O + "baseline/permutations.rs")
s = open(O + "baseline/permutations.rs", encoding="utf-8").read()

E = []


def edit(tag, old, new):
    E.append((tag, old, new))


# ---- shared helpers ---------------------------------------------------------
edit(
    "AE/G/J helper",
    """/// Move several mobjects at once along arcs of `arc` radians (0 is straight;
""",
    """/// Old captions out, then new in, for words on the same spot: a crossfade prints one over the other.
fn swap_caption(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob]) -> fmn::Result<()> {
    fade(stage, old, false)?;
    fade(stage, new, true)
}

/// Move several mobjects at once along arcs of `arc` radians (0 is straight;
""",
)
edit(
    "M constant",
    """const WRONG_BEND: f64 = 2.2;
""",
    """const WRONG_BEND: f64 = 2.2;
/// How far a link's head or tail stands from a slot's centre in a cycle longer than a swap.
const LINK_SEP: f64 = 0.16;
""",
)
edit(
    "M helper",
    """fn below_top(slot: usize) -> (f64, f64) {
    let (x, y) = row_slot(slot, ROW_TOP);
    (x, y - 0.42)
}
""",
    """fn below_top(slot: usize) -> (f64, f64) {
    let (x, y) = row_slot(slot, ROW_TOP);
    (x, y - 0.42)
}

fn shifted((x, y): (f64, f64), dx: f64) -> (f64, f64) {
    (x + dx, y)
}
""",
)

# ---- L: the slot number hidden by the 4-to-1 link -----------------------------
edit(
    "L",
    """    for (index, &at) in TRAIL.iter().enumerate() {
        board.push(ring(stage, at, 0.44, Palette::dim(), 2.0));
        board.push(k.text(
            stage,
            &(index + 1).to_string(),
            23.0,
            Palette::dim(),
            v(at.0, at.1 + 0.62),
        )?);
""",
    """    for (index, &at) in TRAIL.iter().enumerate() {
        board.push(ring(stage, at, 0.44, Palette::dim(), 2.0));
        // Lower-row numbers sit under their rings: the link from slot 4 to slot 1 runs up through the space above.
        let label_y = if at.1 < 0.0 { at.1 - 0.62 } else { at.1 + 0.62 };
        board.push(k.text(
            stage,
            &(index + 1).to_string(),
            23.0,
            Palette::dim(),
            v(at.0, label_y),
        )?);
""",
)

# ---- M: 11_4's (1 3 5) arcs ---------------------------------------------------
edit(
    "M 11_4 arcs",
    """    on_screen.push(curved(stage, above_top(0), above_top(2), -LOOP_BEND, Palette::hero())?);
    on_screen.push(curved(stage, above_top(2), above_top(4), -LOOP_BEND, Palette::hero())?);
""",
    """    // A head lands left of a slot's centre and a tail leaves right of it, so only the swap is double-headed.
    let tail = |slot| shifted(above_top(slot), LINK_SEP);
    let head = |slot| shifted(above_top(slot), -LINK_SEP);
    on_screen.push(curved(stage, tail(0), head(2), -LOOP_BEND, Palette::hero())?);
    on_screen.push(curved(stage, tail(2), head(4), -LOOP_BEND, Palette::hero())?);
""",
)
edit(
    "M 11_4 closing",
    """    on_screen.push(curved(stage, below_top(5), below_top(1), -SWAP_BEND, Palette::blue())?);
    let closing = curved(stage, above_top(4), above_top(0), LOOP_BEND, Palette::hero())?;
""",
    """    on_screen.push(curved(stage, below_top(5), below_top(1), -SWAP_BEND, Palette::blue())?);
    let closing = curved(stage, tail(4), head(0), LOOP_BEND, Palette::hero())?;
""",
)
edit(
    "M 11_4 wrong",
    """    let wrong = curved(stage, above_top(4), above_top(2), WRONG_BEND, Palette::red())?;
""",
    """    // Its head lands on the head of the link from slot 1: two inputs at one point.
    let wrong = curved(stage, tail(4), head(2), WRONG_BEND, Palette::red())?;
""",
)
edit(
    "M 11_4 closing again",
    """    fade(stage, &[wrong, clash, warning], false)?;
    let closing = curved(stage, above_top(4), above_top(0), LOOP_BEND, Palette::hero())?;
""",
    """    fade(stage, &[wrong, clash, warning], false)?;
    let closing = curved(stage, tail(4), head(0), LOOP_BEND, Palette::hero())?;
""",
)

# ---- M and AB: loop_arrows and walk_cycle --------------------------------------
edit(
    "M loop_arrows",
    """    let mut arrows = Vec::new();
    for (index, &label) in cycle.iter().enumerate() {
        let next = cycle[(index + 1) % cycle.len()];
        arrows.push(curved(
            stage,
            anchor(label - 1),
            anchor(next - 1),
            link_bend(label, next, above),
            color,
        )?);
    }
    Ok(arrows)
}
""",
    """    let offsets = link_offsets(cycle);
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
            let head = if tail_right((index + 1) % n) { -LINK_SEP } else { LINK_SEP };
            (tail, head)
        })
        .collect()
}
""",
)
edit(
    "AB walk_cycle offsets",
    """    let anchor: fn(usize) -> (f64, f64) = if above { deck_above } else { deck_below };
    let walker = halo(stage, deck_slot(cycle[0] - 1), 0.37, Palette::ink(), 3.0);
""",
    """    let anchor: fn(usize) -> (f64, f64) = if above { deck_above } else { deck_below };
    let offsets = link_offsets(cycle);
    let walker = halo(stage, deck_slot(cycle[0] - 1), 0.37, Palette::ink(), 3.0);
""",
)
edit(
    "AB walk_cycle arrow",
    """        let arrow = curved(stage, anchor(label - 1), anchor(next - 1), bend, color)?;
""",
    """        let (tail, head) = offsets[index];
        let arrow = curved(stage, shifted(anchor(label - 1), tail), shifted(anchor(next - 1), head), bend, color)?;
""",
)

# ---- same-spot crossfades ------------------------------------------------------
edit("G", "    exchange(stage, &leaving, &[later, first])?;\n", "    swap_caption(stage, &leaving, &[later, first])?;\n")
edit("J", "    exchange(stage, &[interfere], &[rule])?;\n", "    swap_caption(stage, &[interfere], &[rule])?;\n")
edit("AE", "    exchange(stage, &changing, &[name, restart])?;\n", "    swap_caption(stage, &changing, &[name, restart])?;\n")
edit("AE 645.5", "    exchange(stage, &[name], &[still])?;\n", "    swap_caption(stage, &[name], &[still])?;\n")
edit(
    "AE 657.5",
    "    exchange(stage, &[scene.notes[1]], &[loop_length])?;\n",
    "    swap_caption(stage, &[scene.notes[1]], &[loop_length])?;\n",
)

# ---- X: composite moves, with a rest between the factors ------------------------
edit(
    "X rs",
    """    perform(stage, &mut left, s, SWAP_ARC, 0.6, Some((left_walker, 1)))?;
    perform(stage, &mut left, r, FRAC_PI_2, 0.6, Some((left_walker, 1)))?;
""",
    """    perform(stage, &mut left, s, SWAP_ARC, 0.6, Some((left_walker, 1)))?;
    stage.wait(0.35)?;
    perform(stage, &mut left, r, FRAC_PI_2, 0.6, Some((left_walker, 1)))?;
""",
)
edit(
    "X sr",
    """    perform(stage, &mut right, r, FRAC_PI_2, 0.6, Some((right_walker, 0)))?;
    perform(stage, &mut right, s, SWAP_ARC, 0.6, Some((right_walker, 0)))?;
""",
    """    perform(stage, &mut right, r, FRAC_PI_2, 0.6, Some((right_walker, 0)))?;
    stage.wait(0.35)?;
    perform(stage, &mut right, s, SWAP_ARC, 0.6, Some((right_walker, 0)))?;
""",
)

# ---- T: 11_20's first sentence ---------------------------------------------------
edit(
    "T sentence 0",
    """    fade(stage, &[group], true)?;

    k.until_sentence(stage, "11_20", 1)?;
""",
    """    fade(stage, &[group], true)?;

    // Sentence 0: two swaps compose, then undo in reverse, and the row is home for the count below.
    let mut row = card_row(stage, k, 4, sign_top, corner_paint)?;
    fade(stage, &row.mobs(), true)?;
    trade_slots(stage, &mut row, (0, 1), 0.6)?;
    trade_slots(stage, &mut row, (1, 2), 0.6)?;
    stage.wait(0.3)?;
    trade_slots(stage, &mut row, (1, 2), 0.6)?;
    trade_slots(stage, &mut row, (0, 1), 0.6)?;

    k.until_sentence(stage, "11_20", 1)?;
""",
)
edit(
    "T sentence 1",
    """    let mut row = card_row(stage, k, 4, sign_top, corner_paint)?;
    let mut count = new_counter(stage, k, "Swaps", (-3.4, -0.15))?;
    let mut appearing = row.mobs();
    appearing.extend([test, named, count.mob]);
    fade(stage, &appearing, true)?;
""",
    """    let mut count = new_counter(stage, k, "Swaps", (-3.4, -0.15))?;
    fade(stage, &[test, named, count.mob], true)?;
""",
)

# ---- V: 11_21's colours wait for the pairing -------------------------------------
edit(
    "V comment",
    "    // Sentence 1: all twenty-four, sorted into two kinds on screen.\n",
    "    // Sentence 1: all twenty-four, one colour until the pairing tells the two kinds apart.\n",
)
edit(
    "V cells",
    """        left_cells.push(k.text(stage, &cell_text(p), 24.0, Palette::gold(), v(even_at.0, even_at.1))?);
        right_cells.push(k.text(
            stage,
            &cell_text(with_first_two_swapped(p)),
            24.0,
            Palette::blue(),
            v(odd_at.0, odd_at.1),
        )?);
""",
    """        left_cells.push(k.text(stage, &cell_text(p), 24.0, Palette::ink(), v(even_at.0, even_at.1))?);
        right_cells.push(k.text(
            stage,
            &cell_text(with_first_two_swapped(p)),
            24.0,
            Palette::ink(),
            v(odd_at.0, odd_at.1),
        )?);
""",
)
edit(
    "V sentence 3",
    """    k.until_sentence(stage, "11_21", 3)?;
    pulse(stage, &right_cells)?;
    pulse(stage, &left_cells)?;
""",
    """    k.until_sentence(stage, "11_21", 3)?;
    let even_key = k.text(stage, "Gold: even", 26.0, Palette::gold(), v(-0.15, 0.3))?;
    let odd_key = k.text(stage, "Blue: odd", 26.0, Palette::blue(), v(-0.15, -0.3))?;
    recolor(stage, &left_cells, Palette::gold(), 0.3)?;
    recolor(stage, &right_cells, Palette::blue(), 0.3)?;
    fade(stage, &[even_key, odd_key], true)?;
    pulse(stage, &right_cells)?;
    pulse(stage, &left_cells)?;
""",
)
edit(
    "V on_screen",
    "    on_screen.extend([title, count, split, kinds]);\n",
    "    on_screen.extend([title, count, even_key, odd_key, split, kinds]);\n",
)

# ---- AA: how to read the deck after one shuffle -----------------------------------
edit(
    "AA",
    """    k.until_sentence(stage, "11_22", 2)?;
    walk_cycle(stage, &trails[0], true, Palette::hero(), &mut scene.arrows)?;
    k.until_sentence(stage, "11_22", 3)?;
    walk_cycle(stage, &trails[1], false, Palette::blue(), &mut scene.arrows)?;
""",
    """    k.until_sentence(stage, "11_22", 2)?;
    // The deck has shuffled once, so an arrow from a slot lands on the card that started there.
    let (from, to) = (trails[0][0], trails[0][1]);
    let reading = k.text(
        stage,
        &format!("Card {from} went from slot {from} to slot {to}"),
        27.0,
        Palette::hero(),
        v(0.0, -0.75),
    )?;
    fade(stage, &[reading], true)?;
    walk_cycle(stage, &trails[0], true, Palette::hero(), &mut scene.arrows)?;
    k.until_sentence(stage, "11_22", 3)?;
    fade(stage, &[reading], false)?;
    walk_cycle(stage, &trails[1], false, Palette::blue(), &mut scene.arrows)?;
""",
)

# ---- AC and AD: the full deck -------------------------------------------------------
edit(
    "AC",
    '        "2 fixed, 1 swap, 6 cycles of length 8",\n',
    '        "By size: 2 fixed, 1 swap, 6 cycles of length 8",\n',
)
edit(
    "AD",
    """    run_bars_counted(stage, k, &mut scene.bars, &mut scene.count, &out, runs, 0.5)?;
    Ok(scene)
""",
    """    run_bars_counted(stage, k, &mut scene.bars, &mut scene.count, &out, runs, 0.5)?;
    // The staircase is back: the count meets the prediction, and both hold before anything changes.
    pulse(stage, &[scene.notes[3], scene.count.mob])?;
    stage.wait(0.6)?;
    Ok(scene)
""",
)

# ---- AI, AH, AG: the wires -----------------------------------------------------------
edit(
    "AI",
    """            moves.push((mob, wire_xy(column, *row)));
        }
        d4g::glide_all(stage, &moves, 0.7)?;
    }
""",
    """            moves.push((mob, wire_xy(column, *row)));
        }
        d4g::glide_all(stage, &moves, 0.7)?;
        if column < order {
            stage.wait(0.25)?;
        }
    }
""",
)
edit(
    "AH",
    '        &format!("After {order} stages: every line back where it began"),\n',
    '        &format!("After {order} stages: all data back on their starting lines"),\n',
)
edit(
    "AG label",
    """    let mut appearing = extra.clone();
    appearing.push(repeat);
    fade(stage, &appearing, true)?;
    let mut moves = Vec::new();
    for (&mob, row) in tokens.iter().zip(rows.iter_mut()) {
""",
    """    let fourth = k.text(
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
""",
)
edit(
    "AG on_screen",
    """    on_screen.extend(extra);
    on_screen.push(repeat);
    Ok(on_screen)
""",
    """    on_screen.extend(extra);
    on_screen.extend([repeat, fourth]);
    Ok(on_screen)
""",
)

# ---- AM: a rest after the first shuffle ----------------------------------------------
edit(
    "AM",
    """    fade(stage, &[undone], true)?;
    for _ in 1..eight {
        run_bars(stage, &mut bars, &out, 0.22)?;
    }
""",
    """    fade(stage, &[undone], true)?;
    stage.wait(0.3)?;
    for _ in 1..eight {
        run_bars(stage, &mut bars, &out, 0.22)?;
    }
""",
)

# ---- I: the new word before the formula -----------------------------------------------
edit(
    "I",
    """        k.say(stage, "11_5")?;
        fade(stage, &overlap, false)?;
        let factor = k.tex(
""",
    """        k.say(stage, "11_5")?;
        fade(stage, &overlap, false)?;
        // Sentences 0 and 1: the new word, printed and performed, before any formula.
        let swap_name = k.text(stage, "Transposition: swap two labels", 32.0, Palette::gold(), v(0.0, 1.6))?;
        let swap_cards = [
            card(stage, &k, 1, (-1.5, 0.0), 0.42)?,
            card(stage, &k, 3, (1.5, 0.0), 0.42)?,
        ];
        fade(stage, &[swap_name, swap_cards[0], swap_cards[1]], true)?;
        travel(stage, &[(swap_cards[0], (1.5, 0.0)), (swap_cards[1], (-1.5, 0.0))], SWAP_ARC, 0.9)?;
        k.until_sentence(stage, "11_5", 2)?;
        fade(stage, &[swap_name, swap_cards[0], swap_cards[1]], false)?;
        let factor = k.tex(
""",
)

# ---- a test for the one new function --------------------------------------------------
edit(
    "test",
    """    #[test]
    fn the_rings_follow_the_trails_the_review_names() {
""",
    """    #[test]
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
""",
)

for tag, old, new in E:
    n = s.count(old)
    assert n == 1, (tag, n)
    s = s.replace(old, new)

open(O + "candidate/permutations.rs", "w", encoding="utf-8").write(s)
print(len(E), "edits applied")
