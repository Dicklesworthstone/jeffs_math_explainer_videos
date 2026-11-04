# Chapter 11 permutations: independent teaching review

RubyRaven, 2026-11-04, for BlueSalmon (Mail 1001, thread `mx-video-program-program-87do.64`).

**Status:** source review plus the still frames listed below. I heard no audio. All code here is an uncompiled, unrendered sketch, and I ran no Cargo. This review accepts nothing. I edited no canonical file, and this file is the only thing I added to the logarithm staging.

Paths starting `src/` are under `explainer_corpus/abstract_algebra_expanded/`. Line numbers refer to `src/permutations.rs` at sha256 `1301e02d188c…`, which matches `root-permutations-cycle03/permutations.rs`.

## Verdict

The mathematics is right. The chapter does not yet deliver the intuitive visual build-up the owner asked for.

Up to 11_3 the viewer can follow a picture. From 11_4 on, most of the chapter is text on a dark screen:

- No card is ever physically rearranged.
- No permutation with two nontrivial cycles appears.
- The parity proof uses no numbers.
- The square comparison draws no square.
- The film never says that permutations form a group, though 12_9 and 14_10 rely on it.

There are also two outright errors: 11_1 describes the trail backwards, and the 46 s caption sits on arrows that contradict it.

The preview chapter runs 4:41 (`times.json`, 280.866 s, placeholder cue lengths). The treatment budgets 8–12 min for this sequence, so most of its planned pictures are not in the film yet. The treatment's own example, the square's corner permutations, has been replaced by a (1 3 4) unrelated to the square.

## Findings in priority order

| ID | Cue | Defect | Repair size |
|---|---|---|---|
| P0-1 | 11_1 | Narration describes the trail in the wrong direction | One sentence |
| P0-2 | 11_2 | "1↦4 in the reverse cycle" sits on forward arrows 1→3→4→1 | About 25 lines |
| P1-3 | 11_9 | Square comparison has no square; (1 4 3 2) and rs/sr are missing | New beat map |
| P1-4 | 11_7 | Parity argument is algebra only and omits one case of factors | New beat map, 1 sentence |
| P1-5 | 11_8 | Two swap routes to one permutation are never shown | New beat map |
| P1-6 | 11_4 | 33 s of two static lines; no disjoint cycles on screen | New beat map |
| P1-7 | 11_9 | Permutations are never said to form a group | 1 sentence, 1 caption |
| P1-8 | 11_10 | 11_9's text stays up for the entire 28 s cue | About 10 lines |
| P2-9 | 11_3, 11_5 | Card colours change between cues | 1 line |
| P2-10 | 11_6 | One description shown for "many"; prediction is not the last beat | 1 caption, reorder |
| P2-11 | 11_1 | "No collisions, no missing destinations" has no picture | Small beat |
| P2-12 | 11_3→11_4 | About 8.9 s without narration while traces run (preview timing) | Root's call |

If time is short, I would do P0-1, P0-2, then 11_9, 11_7/11_8 and 11_4, in that order. 11_9 comes first of the larger repairs because it pays off the promise made in 10_11 and needs the fewest new helpers.

## P0-1. 11_1 walks the trail backwards

**Script:** "Let's follow card one, then the card that takes its place, until the trail closes."

**Defect:** 11_2 goes on to say "one goes to three, three goes to four, and four goes back to one". The card that takes card 1's place is the one moving to slot 1, which is card 4. Following that instruction gives 1, 4, 3, the inverse of the cycle the next cue writes down. A viewer who obeys 11_1 gets 11_2 wrong.

**Repair, same sentence count:** "Let's follow card one to its new position, then the card it displaced, until the trail closes."

## P0-2. Caption contradicts the arrows (46 s)

**Inspected:** sheet 024 (43.5–45 s) and frame 103 (46.0 s). The blue arrows 1→3→4→1 are on screen under "1↦4 in the reverse cycle", with `(1 3 4)=(3 4 1)` beside them.

**Source causes:**

- L227 pushes the forward arrows into `scene`, so nothing removes them until L251.
- L240–249 adds a caption and draws nothing reversed. `(1 4 3)` does not appear until 11_4.
- L233 replays the whole `cycle_trace`. It takes 8.0 s (three links at 2.4 s, plus the ring) and adds 1.2 s of fades. That pushes the caption to about 23 s of a 25.0 s preview cue, so it lives about 3 s. This arithmetic agrees with the frames.

Adding reversed arrows without fixing the timing would leave them on screen for about 2 s, so the repair covers both.

**Repair:**

```rust
// New: the arrow half of cycle_trace, in one colour.
fn links(stage: &mut Stage<'_>, order: &[usize], positions: &[(f64, f64); 4], color: Srgb)
    -> fmn::Result<Vec<Mob>> { /* Arrow::new(..).buff(0.5).thickness(3.0) per pair, then fade in */ }

// New: the ring walks arrows already on screen. About 3.35 s, not 8 s.
fn ring_walk(stage: &mut Stage<'_>, order: &[usize], positions: &[(f64, f64); 4])
    -> fmn::Result<()> { /* ring fade in, 0.85 s per link, fade out */ }

// L227: keep the forward arrows out of `scene`.
let forward = cycle_trace(stage, &k, &[0, 2, 3, 0], &positions)?;

// L233-234 become one call. It shows the same arrows, entered at another vertex.
ring_walk(stage, &[2, 3, 0, 2], &positions)?;

// L238: delete `scene.push(same)`. `same` leaves at sentence 4.

// L239-249
k.until_sentence(stage, "11_2", 4)?;
let mut leaving = forward.clone();
leaving.push(same);
fade(stage, &leaving, false)?;
let backward = links(stage, &[0, 3, 2, 0], &positions, Palette::hero())?; // 1→4→3→1
let reverse = k.tex(
    stage,
    r"(1\;3\;4):\quad1\mapsto3\qquad(1\;4\;3):\quad1\mapsto4",
    28.0,
    Palette::hero(),
    &[],
    v(0.0, -2.6),
)?;
fade(stage, &[reverse], true)?;
scene.extend(backward);
scene.push(reverse);
k.hold(stage)?;      // same pattern as L299-300
stage.wait(1.0)?;
k.say(stage, "11_3")?;
```

Keep faded mobs out of `scene`. `fade_out` restores the mob when it ends, so fading it again at L251 can flash. That comes from my notes on another lane and I did not test it here.

Optional, since the script says "usually": a small note that a two-label swap is its own reverse. Checked below.

## P1-3. 11_9 draws no square

**Inspected:** frame 541 (245 s). The cue is three lines of text with no anchors (L421–430).

**Why it matters:** 10_11 ends "Next we will make the moves themselves into permutations of labelled objects." The film keeps that promise only in this cue, and only in words. The viewer never sees why a permutation can fail to be a symmetry, or why a quarter-turn is odd.

The treatment already specifies the picture, and every value below agrees with `d4::apply` and `tile_model::CORNERS`:

- Labels run 1 top right, 2 bottom right, 3 bottom left, 4 top left.
- The film's counterclockwise turn is `r=(1 4 3 2)`.
- The film's flip (`y → −y`) is `s=(1 2)(3 4)`.
- Rightmost first, `rs=(2 4)` and `sr=(1 3)`.

**Beat map, four sentences kept:**

| Sentence | Picture |
|---|---|
| s0 "…not every permutation of four corners preserves a square" | Labelled square at the left, outline through 1-2-3-4. Cards 1 and 2 trade corners. Redraw the outline in label order: it becomes a bow-tie. Edge 2–3 lands on the diagonal, drawn in `Palette::red()`. Caption: "A permutation, not a symmetry". |
| s1 "twenty-four permutations and only eight square symmetries" | `4\times3\times2\times1=24` against the existing "4 choices × 2 neighbors = 8" (07_5/07_6 callback). Eight `d4g::thumb` at right. |
| s2 "A quarter-turn is a four-cycle, so it is odd; a half-turn…" | Cards glide a quarter turn. `r=(1\;4\;3\;2)=(1\;2)(1\;3)(1\;4)`, "3 swaps: odd". Glide again. `r^2=(1\;3)(2\;4)`, "2 swaps: even". |
| s3 (see the wording below) | `rs=(2\;4)` and `sr=(1\;3)` side by side, following 2→1→4 with a ring. |

**Script change for s3.** The current sentence, "The same abstract move can be studied geometrically or as a rearrangement of labels", asserts the link without showing it. Replacement, wording yours to change:

> "Now read the square's own moves as labels: r s is the swap two, four, and s r is the swap one, three, the same refusal to commute we saw on the tile."

This restores the treatment's calculation in one sentence. The treatment wanted it as a prediction, which would need its own cue.

**Native pieces:**

- The moving square uses `d4g::glide_all(stage, &[(card, corner)], 0.9)`.
- The outline is `Polygon::new([...]).style(stroke_style(Palette::dim(), 2.0))`, as in `d4_group.rs:208`.
- A two-point `Polygon` draws the red diagonal, as in `arithmetic.rs:341`.

## P1-4. 11_7 is algebra with no witness

**Inspected:** frame 431 (196 s).

**Defects:**

- Three lines of symbols, with the first factor pulled out of an adjacent swap. "Sometimes both reversing signs" is spoken but never occurs on screen.
- It is unclear what "swapping labels" acts on: the numbers, or the `x_i` headers.
- s1 claims the result for n labels, but the script covers only factors that touch a swapped label. For n ≥ 4 the proof also needs factors that avoid both labels. No sentence says those stay fixed.

**Repair: one four-slot witness showing all four cases in one swap.** Put `x=(1,2,4,8)` on four fixed slots, as the treatment asks. Its six differences are all different, so each chip can be followed by value.

| Header | `x_2−x_1` | `x_3−x_1` | `x_4−x_1` | `x_3−x_2` | `x_4−x_2` | `x_4−x_3` | Product |
|---|---|---|---|---|---|---|---|
| Start (1,2,4,8) | 1 | 3 | 7 | 2 | 6 | 4 | 1008 |
| Swap slots 1,3 → (4,2,1,8) | −2 | −3 | 4 | −1 | 6 | 7 | −1008 |

Swapping slots 1 and 3 shows every case at once:

- `x_3−x_1` stays in place and reverses, 3 → −3. This is the only net sign change.
- `x_2−x_1` and `x_3−x_2` trade places and both reverse: `(−2)(−1)=1·2`.
- `x_4−x_1` and `x_4−x_3` trade places without reversing: `4·7=7·4`.
- `x_4−x_2` is untouched.

Slots (2,4) would also show all four. (1,3) reads better on the row. An adjacent swap never shows double reversal, which is why the current example cannot illustrate its own sentence.

**Script:**

- s2 becomes "Swap the numbers on any two labels, and this product changes sign."
- Add after s4: "Factors that avoid both labels do not change at all." The old s5 becomes s6.

**Beat map:**

| Sentence | Picture |
|---|---|
| s0 | `\Delta(x)=\prod_{i<j}(x_j-x_i)` stays at the top (the existing `product`) |
| s1 | Slots and cards; six chips under fixed headers; tally `+1008` |
| s2 | Cards in slots 1 and 3 glide across |
| s3 | Ring chip `x_3−x_1`; 3 → −3 in red |
| s4 | Gold pair trades and both reverse; blue pair trades |
| s5 (new) | Dim the 6 and leave it in place |
| s6 | Tally `−1008`; `\|\Delta\|` unchanged; the existing `sign` caption |

This retires `example`, `swapped` and `explanation` (L368–390). Update the fade lists at L408 and L420 to match.

**Pure helper, driving the chips, tests and facts:**

```rust
pub(crate) const WITNESS: [i32; 4] = [1, 2, 4, 8];
const PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
pub(crate) fn differences(x: [i32; 4]) -> [i32; 6] {
    PAIRS.map(|(i, j)| x[j] - x[i])
}
pub(crate) fn after_swaps(mut x: [i32; 4], swaps: &[(usize, usize)]) -> [i32; 4] {
    for &(a, b) in swaps {
        x.swap(a, b);
    }
    x
}
```

## P1-5. 11_8 never shows two routes

**Inspected:** frame 489 (222 s), which shows the formula and "N = number of swaps" only. The cue has no anchors (L407–418).

**Defect:** the cue's whole point is that two descriptions reach one arrangement, and the screen never shows two descriptions.

**Repair: two rows of the same witness, each with a sign lamp.**

| Route | Arrangements | Sign |
|---|---|---|
| One swap, (1 3) | (1,2,4,8) → (4,2,1,8) | + → − |
| Three swaps, (1 2), (2 3), (1 2) | (1,2,4,8) → (2,1,4,8) → (2,4,1,8) → (4,2,1,8) | + → − → + → − |

| Sentence | Picture |
|---|---|
| s0 "multiplied by minus one once per swap" | Lower row steps through its three swaps, with the lamp toggling each time |
| s1 "final arrangement determines the final product… not zero" | Upper row makes its one swap. Ring both final rows: same numbers, same `−1008`. Caption `1008\ne0`. |
| s2 "must have the same parity" | `(1\;3)=(1\;2)(2\;3)(1\;2)`, "1 swap, 3 swaps: both odd". In red: "2 swaps would give +1008: impossible". |
| s3 | Keep the existing `\Delta_{\text{final}}=(-1)^N\Delta_{\text{initial}}` and "N = number of swaps" |

## P1-6. 11_4 is two static lines for 33 s

**Inspected:** frame 221 (100 s). L301–318 has no anchors, so "Separate loops do not interfere" appears at cue start, before s2 says it. The film never shows a permutation with two nontrivial cycles, and "disjoint cycles" remains words.

**Repair, seven sentences kept, anchors s1–s6:**

| Sentence | Picture |
|---|---|
| s0 "run its arrows backwards" | Cards 1, 3, 4 on the 11_2 triangle. They glide along the forward arrows. Those arrows fade, hero arrows reverse them, and the cards glide home. This is the chapter's first physical rearrangement. |
| s1 | The existing `(1\;3\;4)^{-1}=(1\;4\;3)` |
| s2 "they commute" | Six slots, loop {1,3,5} in hero, {2,6} in blue, 4 in dim. Top row runs the 3-cycle then the swap; bottom row the swap then the 3-cycle. Both end `5 6 1 4 3 2`. Caption `(1\;3\;5)(2\;6)=(2\;6)(1\;3\;5)`. |
| s3 "eventually returns" | Ring walks 1→3→5→1 on the top row |
| s4 "cannot first enter an earlier label" | Red arrow 5→3, red ring on 3: "two cards into slot 3". Then remove it. |
| s5 "disjoint cycles" | `(1\;3\;5)(2\;6)` and "4 stays put" |
| s6 "order and starting point" | `=(2\;6)(3\;5\;1)` |

Use `d4g::glide_all` for the rows. Colour by loop here, since a colour per label would repeat. The contrast with the (1 2 3), (1 2) pair from 11_3 is one short caption: "Overlapping: order matters. Separate: it does not." `disjoint` (L311) can retire or become that caption.

## P1-7. Permutations are never called a group

No sentence in chapter 11 says that the permutations of a set form a group, and no cue names the symmetric group. Yet 12_9 says "a permutation group", and 14_10 asserts that "the even permutations of four labels form a group of twelve elements".

**Repair, in 11_9 s1:**

> "All twenty-four permutations form a group, and our eight square symmetries sit inside it as a subgroup."

Composition (11_3) and inverses (11_4) are already taught, and subgroups come earlier in registry order. Naming `S_4` on the caption would be cheap. For 14_10, see the decisions list below.

## P1-8. 11_10 keeps stale text for 28 s

**Inspected:** frame 607 (275 s), which matches frame 541 (245 s). L431–432 only says and finishes. Three lines about corners stay up while the voice talks about wires and data. I found this one independently; it is not on `repair.json`'s open list.

**Repair:** fade them and show the undo on the existing board. The route test belongs with the two at L440+.

```rust
k.say(stage, "11_10")?;
fade(stage, &[square, turn, half], false)?; // whatever 11_9 leaves after P1-3
named_route_board(stage, &k, ["Start", "After (1 3 4)", "Then its inverse"], [1, 3, 4])?;
k.until_sentence(stage, "11_10", 1)?;
for origin in 0..3 {
    named_trace_route(stage, &k, origin, [1, 2, 0], [2, 0, 1], [1, 3, 4])?;
}
k.finish(stage)

// test: for i in 0..3, route(i, [1, 2, 0], [2, 0, 1])[2] == i
```

The three traces take about 8.7 s and overrun s1, which is why this sketch has no further captions.

## P2 findings

**P2-9. Card colours change identity.** Card 3 is green in 11_2 and yellow in 11_5, and card 4 swaps the other way (sheets 024 and 077, frame 333). Card 2 is gold in 11_1 and hero in 11_3. L88 colours by row. The fix uses the same colours as L206:

```rust
let color = crate::d4_group::corner_color(slots[origin] - 1);
```

**P2-10. 11_6.** Frame 333 (150 s) shows only the stale (1 3 4) board and the question.

- s0 says "many different swap descriptions" but one is shown. Add `(1\;3\;4)=(3\;4)(1\;4)` near `factor`. The identity is correct; the placement is unchecked, because the board's column labels sit at y = 2.0.
- The question appears at cue start (L347–354). Anchor it at s1.
- "If the answer is no…" primes the answer before the prediction hold. Replacement: "There may be many different swap descriptions of the same permutation. Can one description use an even number of swaps and another use an odd number? Predict before we prove it." Open 11_7 with the removed clause: "If the answer is no, parity belongs to the permutation itself, not to our chosen description." That shifts every 11_7 index in P1-4 up by one.

The script and the direction map must change together (see Consequences).

**P2-11. 11_1.** Frames at 8.5–10 s (sheet 005) are static. For s0, a brief bad map: two arrows into one slot with a red ring ("collision") and an empty slot ("missing"). Otherwise the bijection condition stays verbal, though 11_4 s4 later depends on it.

**P2-12. Silence after 11_3.** Arithmetic only, from preview timing: 11_3 ends at 75.7 s and 11_4 starts at 84.6 s. The three reverse traces (L286–288) carry no anchors and run past s7. If that is an intended silent "watch this" beat, it needs a prompt on screen. If not, shorten the traces. Recheck with real takes.

## Treatment coverage

Compared with `docs/video-program/research-20261016/evidence/abstract-algebra-expanded-treatment-20261103.md` (the permutations outline at lines 65–69 and the narration sequence at lines 451–540).

| Treatment item | In the film now | Covered by |
|---|---|---|
| `4×3×2×1=24`, with the 8 highlighted inside the 24 | One line of text at 11_9 | P1-3 |
| Neighbouring-corner swap as a non-symmetry | Words only | P1-3 |
| `r=(1 4 3 2)`, `s=(1 2)(3 4)` on real corners | Absent | P1-3 |
| Prediction: compute `rs`, then `sr` | Absent; 11_3 uses an unrelated pair | P1-3, without the prediction |
| Inverse: same arrows backward | Caption only | P0-2, P1-6 |
| Disjoint three-cycle and two-cycle | Absent | P1-6 |
| Order 6 as the lcm, with a clock | Absent; 15_5 states the lcm for pairs | **Decision** |
| `(1 4 3 2)` as three swaps, checked by arrows | Absent; (1 3 4) as two swaps instead | P1-3 states it |
| Parity by a product of differences on four numbers | Three symbolic factors | P1-4, P1-5 |
| Sign rule, even/odd table, even permutations as a subgroup | Absent | **Decision** |
| Worked check-digit or coding example, guarantee qualified | Absent; 11_10 only names uses | **Decision** |

**Decisions for root or the owner:**

1. **Order as lcm.** The (1 3 5)(2 6) board already fits: repeat the move and count to 6. It needs one sentence, or preferably a new cue, since 11_4 is already seven sentences.
2. **Sign rule.** One sentence in 11_8 would earn what 14_10 later assumes: "Swap counts add, so an even followed by an even stays even, and the even permutations form a subgroup." On four labels there are 12, and they are closed. I recommend this; it is the cheapest of the three.
3. **Worked application in 11_10.** A larger addition, and chapter 17 may be the better home.

**Gallian comparison.** The book is not in the repo, so I judged against the owner note and the treatment. From memory, unverified, the chapter on permutations also uses two-row array notation, the symmetric group by name, order as an lcm, and the alternating group. Two-row notation is the only item here the treatment omits. The owner asked for inspired-by, not an adaptation, so I would not add it.

## What checks out

Brute force, independent of the Rust, all passing:

- 11_3: `(1 2 3)(1 2)=(1 3)` and `(1 2)(1 2 3)=(2 3)`.
- 11_5: `(1 3 4)=(1 4)(1 3)`, also `=(3 4)(1 4)`.
- 11_4: `(1 3 4)⁻¹=(1 4 3)`. None of the 120 permutations of five labels has a trail entering mid-way (the s4 claim). `(1 3 5)(2 6)` commutes, equals `(2 6)(3 5 1)`, and first returns after 6 repetitions.
- 11_7: any swap negates the product for n = 2 to 6. On the witness, all six swaps give −1008, and double reversal occurs exactly when the third label lies between the swapped two.
- 11_8: every swap word on four slots up to length 6 gives `1008·(−1)^length`.
- 11_9: 24 corner permutations, 8 keep edges as edges. Quarter-turn odd, half-turn even. The 8 split 4 even and 4 odd. `(1 4 3 2)=(1 2)(1 3)(1 4)`.
- 14_10: 12 of 24 are even and closed. `sign(ab)=sign(a)sign(b)` on all 576 pairs.
- 11_2 "usually": a two-label swap equals its reverse, a longer cycle does not.

Test at `src/main.rs:604` covers the 11_3 and 11_5 products. Test at `src/main.rs:683` covers closure of the eight.

The repaired sentences in P1-3, P1-4 and P1-5 are checked as mathematics only. Root should test them against the listening.

## Consequences for root

- **Script rows that change:** 11_1 s3; 11_6; 11_7 s2 and a new sentence; 11_9 s1 and s3; optionally 11_8. Sentence indexes bind the waits, so rerun the anchors after any rewording. I found no `expanded-algebra-11-*` take directories under `five-film-release-20261027`, so rewording appears to cost no existing Oliver take. I did not search other lanes. Every changed cue still needs Oliver synthesis through root's guarded lane.
- **Facts and oracle:** chapter 11 has no `facts.add` key of its own. Of its on-screen numbers only the 8 is covered, by `square.symmetry_count` (`src/main.rs:139`); the 24 at L423 is not. The standing rule calls for keys in `rebuild_facts` and matching `emit` lines in `oracle.py`. Suggested: `permutations.corner_count` 24, `permutations.delta_start` 1008, `permutations.delta_swapped` −1008, the two chip lists, and `permutations.even_on_four` 12 if the sign sentence goes in. Add Rust tests for `differences` and `after_swaps`.
- **Glyph lists:** add new strings to the text list (`src/main.rs`, about L498–510) and the TeX list (about L552–566). `\times` and `\cdot` already appear in scene source (`applications.rs:248`, `cosets.rs:706`), but `main.rs` lists no `\cdot` string, and I did not work out why. "4 choices × 2 neighbors = 8" is already listed (`main.rs:474`).
- **Direction map:** `explainer_corpus/abstract_algebra_expanded/docs/narration-direction-map-oliver22.json` is already stale before any repair. It pins `script.tsv` at `9e7195d6…` (now `e5ba1c3e…`) and the 11_6 hold in `permutations.rs` at `0964b994…` line 166 (now `1301e02d…`, hold at L355; L166 is `.animate()`). I found no later map in that directory, and I do not know what enforces these pins. P2-10 also changes the 11_6 `after` text.
- **Layout:** none of the coordinates above has been seen rendered. Six chips, four slots and a tally (P1-4) is the tightest screen, so check that one first.

## Evidence and gaps

**Read:**

- `src/permutations.rs` in full.
- `script.tsv` chapter 11, plus 07_6, 10_11, 12_1, 12_9, 14_10, 15_5.
- `src/d4.rs`, parts of `src/d4_group.rs`, `src/main.rs` (facts, glyph lists, tests).
- The owner note `abstract-algebra-visual-story-20261101.md` (matching lines only), and the treatment (lines 1–12, 62–74, 449–560).
- `root-permutations-cycle03/repair.json` and the direction map.

**How independent this is:** I reached these findings from the source and frames, but not blind. Mail 1001 named four focus points and I read `repair.json`. P0-1, P1-4's missing case, P1-7, P1-8 and the treatment coverage gaps are additions to those lists.

**Still frames inspected** (1280×720, `narration_lab/third-wave-20261024/root/five-film-release-20261027/expanded-algebra-permutation-cycle03-frames01/`): frames 103, 221, 333, 431, 489, 541, 607; sheets 005, 024, 077. All timings come from the preview cue file in the sibling `-preview01` directory, which has placeholder lengths, not takes. I did not match that render to a source hash in its receipt, though it ran after the source's last modification.

**Not done:**

- No listening, no full-motion viewing, and no frames besides those listed.
- No compile, render, Clippy or test run. The sketches may need signature and import fixes, including `Srgb` in `links`.
- Gallian was not consulted.
- The checks ran from a scratch script that is not in this staging. I can supply it on request.
