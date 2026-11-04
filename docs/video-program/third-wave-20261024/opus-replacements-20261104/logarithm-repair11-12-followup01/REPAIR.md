# Logarithm 11–12 follow-up: R9–R14, the rule's reading hold, and the cue splits

Bead `mx-video-program-program-87do.63`. Author RubyRaven, staging only, 2026-11-04.
Answers BlueSalmon's Mail 1516 and Mail 1541. Review: `../logarithm/preview02-review-11-12/REVIEW.md`.

**Status: uncompiled, unrendered, nothing heard.** The only checks run are scoped `rustfmt`, plain
arithmetic and text scans (`checks/`). The tests below have never run. Root does the real build.
Narration stays ElevenLabs Oliver and no word changed. Listening, fine sync and independent AV
review remain open.

## What is here

| Path | What it is |
|---|---|
| `explainer_corpus/logarithm_mysteries/` | The candidate: both chapters, `facts.rs`, `oracle.py`, `script.tsv`, `docs/storyboard.md`, `docs/sources.md` |
| `before-splits/` | Repairs only, before Mail 1541's splits: both chapters and the storyboard |
| `patches/cumulative-vs-canonical.patch` | Everything against canonical. **The candidate patch** |
| `patches/01-repairs-on-frozen01.patch` | Chapters from frozen01 to `before-splits/`; facts, oracle, storyboard from canonical |
| `patches/02-cue-splits.patch` | The splits in the chapters, storyboard and sources |
| `patches/script-splits-vs-canonical.patch` | The script delta alone, as Mail 1541 asked |
| `base.sha256`, `candidate.sha256` | Exact hashes; run `shasum -a 256 -c` from the repo root |
| `checks/` | `pure_math`, `split_check`, the oracle diff and the merge results, each with its output |

Apply with `patch -p1` from the repo root. Either the cumulative patch on the canonical bytes in
`base.sha256`, or `../logarithm-repair11-12/repair11-12.patch`, then 01, 02 and the script patch.
Both routes were round-tripped in scratch and reproduce the candidate byte for byte. Frozen01 and
every earlier file are unchanged.

## R9–R14, item by item

Line numbers are the candidate's.

| Item | Fix | Where |
|---|---|---|
| R9 reds crowd the end ring | The other axis is now (0, 1, 1). Nearest approach over 61 poses: line 0.51 to the start ring and 2.08 to the end ring; pin 0.71 and tip arc 0.71 to the end ring | `what_it_forgets.rs` 114 |
| R10 hard resets | All five pose changes go under a background veil, 0.2 s in and 0.2 s out (`redeal`, 672): both lap resets (1138), the direction reset (1164), the red spin (1186) and its return (1199) | `what_it_forgets.rs` |
| R11 gold on gold | 11_6's axes are white, width 3, as in 11_7 | `unreachable.rs` 1259 |
| R12 11_2 leads | The backward run waits for "run backwards" and ends 0.6 s before the next sentence | `unreachable.rs` 922 |
| R12 11_5 leads | The probe stays alone until "send it along both routes", then the routes fade in | `unreachable.rs` 1189 |
| R12 11_6 leads | The stretch goes home first, then the journey runs on "running one after the other" | `unreachable.rs` 1333 |
| R13 tight stack | Panel C's zero moves up 0.1 and its axes shrink to 0.9. Lowest label to the sentence is about 0.16, axis foot about 0.19 | `what_it_forgets.rs` 89 |

R14 and the known leftovers:

| Item | Fix |
|---|---|
| "same end point" near the limit, far from its dot | On a plate under the dot, centre (−4.52, −0.86); the dot's bottom is −0.547 |
| Mixed caret and typeset notation | Typeset everywhere on screen; the Liouville line reads exp(∫ trace dt) |
| Dashes look like notches | Dots on the paired corners, dashes only in the open gap between the two Fs. Corners 3 and 7 had chords lying along the F's edges |
| "suppose" centred | All of 11_4 and 11_5's TeX column is left-aligned to x 2.2 (`tex_left`) |
| "u" plate over an empty plane | The probe arrow stays with its plate |
| Dim purple e^{iπ} line | White |
| Family has no continuation | "…" past each end of the turning line |
| Movers pop on | Fade in over 0.3 s |
| 16 pt purple labels | White; the dots stay purple |
| Long hold at the end of 12 | The last sentence now has beats: convention on "a convention broke the same tie", then gold dots on all four spins and "here: the whole family" on "the honest answer is the whole family" |
| 11_7 plane empty | The plane returns at rest as the old picture leaves, and the spin takes its slot |
| 11_8 title only | "A = diag(−2, −½): no real logarithm" stands under the title |
| Title swaps overlap | Old words leave, then new ones arrive (`reword`), also for the other same-place word swaps |
| Stale headers | Status words removed from both chapter files |
| "Fold the ruler" has no fold | A real fold: the ruler lies flat across panel B, bends shut with its length fixed, then turns its 1 to the top. Closed, it equals `dial_at` to 2e-16 and the 10 lands on the 1. Lowest point 0.237 against the panel's 0.2 |
| 12_1 has no keyboard | Two octaves from A to A, a gold dot on each A, on "the keyboard and the slide rule" |
| "9 10" crowded | Ticks at every whole number, numbers only at 1, 2, 4, 8 |
| Cut crosses the −1 target | −1 gets z-index 1 |
| f111 ghosts | **No scene edit.** Low-level residue in f112–f117 (48.5–49.775 s) is gone at f118, exactly 50.000 s, and ffprobe puts a keyframe there (multiples of 8.333 s). That is the preview's H.264 encode, not scene content |

## The rule's hold

The rule replaces the words in the column as its sentence starts, then the voice's end opens a music
window (the `draw::predict` idiom, `unreachable.rs` 1555–1591). At 15 characters a second:

| Part | Characters | Seconds |
|---|---|---|
| The rule alone, four lines at 22 pt | 42 + 45 + 42 + 30 = 159 | 10.6 |
| Each row's verdict in turn, with a gold mark beside the row | 23 + 24 + 31 + 29 = 107 | 7.13 |
| **Window** | | **17.73** |

- The 10.6 s starts only when the voice stops, so the voice never competes with it. On the scratch
  timing the rule is also fully on screen about 1.7 s before that.
- The row walk is a silent marker that moves only after the rule's time. If 17.7 s feels long on
  Oliver, shorten the rows; the rule's 10.6 s is the floor.
- The rows' heading, "each row is one matrix…", leaves with the old column so that fewer words
  compete. Drop `blocks_head` from that fade if you would rather keep it.

Everything from 11_8 on moves by about +17.7 s. **Root re-scores.** No voice is stretched.

## Cue splits (Mail 1541)

`checks/split_check.out` has the full map. Joined text and order are identical to canonical.

| Old | New, with words | Split before |
|---|---|---|
| 11_7 (123) | 11_7 (91), 11_20 (32) | "In higher dimensions," |
| 12_6 (150) | 12_6 (39), 12_20 (61), 12_21 (50) | "Direction next:", "The two slowest," |

All 14 sentences map to the same exact words. Every `until_sentence`, `sentence_len`, `until_share`
and phrase hook is rebased, and `kit.say` sits at each boundary. `kit.say` order in both chapters
matches the script. Two hooks that used the next sentence's start as a deadline now use the line's
end: 11_7 s4's tween and the 2½-lap run, through `line_end`, which chapter 12 gains.

The new cues have no takes yet, and the old 11_7 and 12_6 takes are untouched. The full plan, map and
hash rebinding are root's. Each new boundary also adds the kit's between-line gap to the timeline.

Merged with root's `narration-binding-stage01/candidate/script.tsv`: no conflict, 80 cues, every word
as canonical, longest cue 95 words.

### Phrase bytes for `phrase-timings.json`

| Cue | Byte and phrase |
|---|---|
| 11_1 | 320 a halfway point; 337 squashes the whole plane |
| 11_2 | **185 run backwards** |
| 11_5 | **72 send it along both routes**; 178 the two routes end in different places |
| 11_6 | **273 running one after the other** |
| 11_7 | 174, 226, 315, 404, 453, as before |
| 11_20 | **91** the matrix's blocks for each negative stretch; **137** come in matching pairs (were 11_7 588, 634) |
| 12_1 | **167 the keyboard and the slide rule** |
| 12_6 | 89 also ends here; 105 and so does two and a half |
| 12_21 | **192 a convention broke the same tie; 230 the honest answer is the whole family** |

Bold entries are new or moved.

## Facts and oracle

Three new keys, placed after `matrix.probe_gap` in both files:

| Key | Value | Checked |
|---|---|---|
| `matrix.blocks_matching` | `-1,-1` | e^(πJ) = −I |
| `matrix.blocks_mixed` | `-2,-2,3` | e^(ln 2 I + πJ) = −2I; ln 3 |
| `matrix.jordan_block` | `-1,1;0,-1` | det 1, trace −2, not −I. The oracle also samples real X with eigenvalues a ± iπ(2k+1), sheared: e^X is always −e^a I, never this block |

The staged oracle prints 96 keys, canonical's 93 plus these three. The rows and the tile now read
these facts instead of literals. `STRETCH_C` is never printed, so it has no key.

**Merging with `combined-repair-stage02` (root's six keys, 99):**

| File | Result | Resolution |
|---|---|---|
| `oracle.py` | Clean; runs and prints 102 keys | None needed |
| `facts.rs` | One conflict at the count test | Keep both new tests; `assert_eq!(f.len(), 102, "oracle.py prints 102 keys");` |
| `docs/storyboard.md` | One conflict in the risk list | Keep my two bullets and root's `matrix_card` bullet |

WhiteSwan's R5 hunks (Mail 1566) add no keys and sit on other lines. I checked that from their
message, not from their files.

## Shared helper patches

Neither patch touches this stage, which has no `draw.rs`. `shared-draw-helpers.patch` and then
`draw-arrow-solid-head.patch` both applied to a scratch copy of canonical `draw.rs`. Chapter 11 calls
`arrow_parts` directly, and chapter 12 draws no arrows.

Under the shared helper's card sizing, using the table widths:

| Card | Half width | x | Gaps |
|---|---|---|---|
| 11_1 mirror | 0.663 | 4.737..6.063 | 0.80 to the edge limit |
| 11_5 A | 0.683 | 2.967..4.333 | 0.25 after "A =", 0.57 before "X =" |
| 11_5 X | 0.615 | 5.535..6.765 | 0.10 after "X =", 0.10 to the limit |
| 11_7 −I | 0.622 | 3.028..4.272 | 0.20 after "−I =" |
| 11_20 size-2 tile | 0.414 | | under `WIDE_TILE_HALF` 0.49 |

The chapter test that used a fixed 0.615 half width now uses 0.683, so it holds with or without the
patch. The tightest gaps are X's, unchanged from today's 0.615. The `CLIP_SLACK` change doesn't reach
these chapters: CLIP 3.2 has no grid line on its box.

## Storyboard and sources

- The collapse now reads truthfully: the dashes compare two ends, and the collapse is a path along the
  straight line of matrices from I to the singular midpoint ½(I + mirror), stopping there, never on
  to the reflection. The same correction is in the drawing-aids list.
- Added: an "Authored reading holds" section, the 12_1–12_3 beats, the new 11_4–11_8 and 12_6 beats,
  corrected status and risk lines, and the cue count (80 once all splits are in).
- `docs/sources.md` changes cue labels only (11_20, 12_21). Canonical and combined-stage02 have the
  same bytes there.

## Known limits, and what to look at first

- **Compile risk.** No compiler has seen these, in place: the new `scene_mut().replace` calls in 11_5,
  11_6, 11_7 and 12_2; nested array `.map` in `block_tile`; float `const` asserts; and the new TeX
  strings, above all `p\text{: no axis reverses}`.
- **Pin end-on.** On axis (0, 1, 1) the pin points nearly at the viewer for about t 0.48–0.84, at its
  shortest 0.13 long at about t 0.68. The tip and arc carry the drag then, and the tip is never hidden.
  The old test's "pin always over 0.8" is replaced by checks at both ends. Judge this on a frame.
- **Dips.** The veil is a plain rectangle. It should be invisible on the background, but it covers only
  the picture, so look for its edge against anything I missed.
- **The fold.** Lowest point is 0.037 above panel B's lower edge at about fold 0.75, for a moment.
- **Phrase times** are shares of a sentence until Oliver's word marks exist. Recheck R12 then.
- **11 new tests**: 5 in `unreachable.rs`, 5 in `what_it_forgets.rs`, 1 in `facts.rs`. Their thresholds
  come from `checks/pure_math.out` where the geometry is involved. They have not run.
