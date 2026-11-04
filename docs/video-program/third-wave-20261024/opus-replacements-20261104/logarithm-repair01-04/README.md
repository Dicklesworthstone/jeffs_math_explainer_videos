# Logarithm chapters 01 to 04: staged picture repairs

LavenderPike for BlueSalmon, Mails 1447 and 1449, 2026-11-04. Bead `mx-video-program-program-87do.63`.

**Status: staged and frozen for root.** This source has never been compiled against real fmn or the real Kit, and it has never been rendered, heard or watched. Nothing here accepts a picture. All four chapters stay on HOLD until root's native checks and new pictures.

## What to apply

The patch touches four chapter files only. Run this from the repo root:

```
patch -p1 --fuzz=0 -i docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair01-04/logarithm-repair01-04.patch
```

If canonical has moved by then, run `git merge-file` per file, with the canonical file as ours and the `base/` and `candidate/` copies as base and theirs.

| File under `explainer_corpus/logarithm_mysteries/` | Base SHA-256 | Candidate SHA-256 |
|---|---|---|
| `src/chapters/building_ruler.rs` | `6ed5ef5b9bf170ad9f043e11b208d17b1948190e7fcea137313bf8a3ac05a4c2` | `e9a6cd9d4d75362a34a22e1ab7c87840c0e1c91edfe15aae860152bc30b6d809` |
| `src/chapters/slide_rule.rs` | `c3ec0193b7cee97e30d10438501da201faa715fe673b29005025afc775cc8d4b` | `6771c36b3ee96e8f35b35da112cad76e3ab8a0df0842de3a5fad551ca78b6e27` |
| `src/chapters/folded_decade.rs` | `c753d51b9f0cfddab08645a10eee7cff0afee692ac040d3a2b43a47e89d679f4` | `acfc7f9c198b3d3d0f99e1b160ec6c2d4e22a4ce28aeb4263deb29df9efa670d` |
| `src/chapters/undoing_growth.rs` | `eafafef71a9c31b8df893f605d14dd08f0530b12238d6f1c3acb0a7f294fba96` | `bb3fd7990f90189ed2bd6cbfe9c34ef0e4b3e9a23ad00ee219f2ac47be73465e` |

Patch SHA-256: `696e89f02bb227c3a4128c7cda841b64226c694e8c218c910a6de90ba01c0f49`. `HASHES.sha256` lists every file in this folder; check it with `shasum -a 256 -c HASHES.sha256` from here.

**Unchanged, same SHA-256 as canonical when staged:**

| File | SHA-256 |
|---|---|
| `script.tsv` | `3e4e34af7090e448d65a2731f0d0d78673679730c0f95fdc695e3e8f2e6f5243` |
| `src/chapters/draw.rs` | `59ebd09d2fa5c793fbb2f9ab414a4b951325cbc68e69d15c488a949eba88f9a3` |
| `src/facts.rs` | `7bf886f9762f1904389cd553d2f10da6f0ae23dd6fd625faa57bcc1ff727a0d2` |
| `src/model.rs` | `8c9e3c6abfcc48d50f56a032130e8c567af7de4e7db1c6b2a9a718d7f7ccddd6` |
| `oracle.py` | `05a0dc9c669b5a554db48bce396053d030a04c4ddc606d478d08a49d53a06f0e` |

No spoken word changed, so every current take still applies. There is no unavoidable mathematical correction to the script.

## Repairs, by finding

Finding numbers are from `review-baseline/REVIEW.md`. "Claim" means `checks/state_checks.py` asks about the snapshot at that moment. "Test" means a chapter test pins it. "None" means the edit has no check of its own, so only the new pictures can confirm it.

### Chapter 01, `building_ruler.rs`

| Finding | Repair | Checked by |
|---|---|---|
| 1.1 ×10 bar did not span 1 to 10 | No squeeze. A live `step_bar` grows from one step to the 4 mark ("one step = ×4"), then to the 10 mark ("now one step = ×10") | Test, claim |
| 1.2 "810" | Every mark stays put, so 8 and 10 sit 0.644 apart | Test, claim |
| 1.3 Base-two claims under a base-ten ruler | `later_steps`, `first`, `name`, `samples` and `true_word` fade at 01_6 sentence 3, before the bar moves | Test, claim |
| 1.4 Equal doublings merge into one bar | `length()` is inset 0.06 per end, leaving a true 0.06 gap between the round caps | Test, claim |
| 1.5 Two half steps are never visible as two | Same inset; the words move to `WORD_Y`, clear of row 1 | Test, claim |
| 1.6 Twelve threes against nineteen doublings cannot be counted | `piece()` alternates opacity 1.0 and 0.55 and keeps a real gap | Claim |
| 1.7 "19 doublings" margin | Rows are shorter and labels are left-aligned at 5.15 | Test, claim |
| 1.8 The hunter gives away the answer | It rests at the midpoint, position 1.5 | Test, claim |
| 1.9 Clutter | The keyboard line replaces the power row (`swap`), at y −1.45 | Claim |
| 1.10 Timing: static through "a different base" | The ×4 step is that sentence's picture | Test, claim |
| 1.11 Rounded equals | `\approx` in both words | None |
| 1.12 Early title | "Building the ruler", then "Where does three go?" at 01_3 sentence 1, then "The logarithm" at 01_6 sentence 1 | Claim |
| 1.13 Word against picture: "stretched" over a compression | The marks hold and the unit bar stretches, so the picture now agrees with the voice | Test, claim |
| 1.15 Fade-out nodes | The lengths no longer touch, so their ends cannot overlap at the joins as they fade | Test (gap) |

### Chapter 02, `slide_rule.rs`

| Finding | Repair | Checked by |
|---|---|---|
| 2.1 The reading line strikes the digit | `HAIR_REACH` 0.42 passes the ticks and stops short of the numbers | Test, claim |
| 2.2 Red end tick and stale caption | The end tick uses the same reach, `stops` drops to `D_Y − 1.1`, and `end`, `stops` and `c_name` join the fade | Test, claim |
| 2.3 The slide runs ahead of the voice | Local `until_phrase` waits for "slide the moving" | Test |
| 2.4 0.398 has no unit | Caption reads "one full length = 1: from 1 to 10, ×10" | Test, claim |
| 2.5 Stale card | Card swap comes before the slide home | Test |
| 2.6 No slide on "slides" | A 0.8 nudge and return in 02_1 | None |
| 2.7 C's name off frame | `c_again`, "C", at `c.x(10) + 0.5` | Test, claim |
| 2.9 "one mark" | A bracket over the rows, with left-aligned "one mark" | Claim |
| Mine, not in the review | C's length number fades before its bar drops and returns beside the dropped bar, so it does not float away from it | None |

### Chapter 03, `folded_decade.rs`

| Finding | Repair | Checked by |
|---|---|---|
| 3.1 Orphaned "one lap = ×10" | It leaves with the arc; a new `lap_tag` by the counter serves 03_2 sentence 4 | Test, claim |
| 3.2 Helix through the caption | `keep` moves under the floor, y −3.55 | Test, claim |
| 3.3 The dial never says 2.4 | Hero tick is dark until the turn lands (tracker `lit_tick`); "2.4" at the landing; caption "the 2.4 mark: one place on the dial" | Test, claim |
| 3.4 Stale title | `heading3` swap at the first sentence of 03_3 | Test, claim |
| 3.5, 3.14 One-second caption; sparse frames | `forgets` holds through sentence 1 | Test, claim |
| 3.6 Tick in the caption | Moved right | Test, claim |
| 3.7 Unlabelled right panel | Caption at 03_4 sentence 1 | Test, claim |
| 3.9 Tight readings | `READ_DX` 0.65 | Test, claim |
| 3.10 Irregular dashes | One `drop_line` with 16 dashes | Claim |
| 3.13 Detached value | Value moved to x 3.6, beside its label | None |

### Chapter 04, `undoing_growth.rs`

| Finding | Repair | Checked by |
|---|---|---|
| 4.1 The red marks survive the mirror | Live `wall` and `barred` reflect with `mirror`: the floor becomes ln's wall on the y-axis and the bar lands on the negative x-axis. Red text "no real logarithm" arrives after the flip | Test, claim |
| 4.2 "ln 2 ≈ 0.693" on the tick labels | `ln2_word` at p(1.1, −0.75) | Test, claim |
| 4.3 The product law has no picture | Blue 1→2, then teal 1→4 slides to start at 2 and lands on 8, with a ring and "2 × 4 = 8" | Test, claim |
| 4.4 "2¹⁰ = 1024" reads as the dashed line's label | Two rows at (4.4, −1.95) and (4.4, −2.4) | Test, claim |
| 4.5 "Only ever climbs" is frozen | Tangent and dot climb during 04_2 sentence 3, over 4 s | Test, claim |
| 4.6 The triangle never shows the trade | "run" and "rise" before the flip; swapped after, with "rise and run trade places" | Test, claim |
| 4.8 Name left behind | `name` fades as the flip starts. Both trees have lost it by the next snapshot, so only the order can tell them apart | Test |
| 4.9 No mark for ln 1 = 0 | `at_one_ring` at (0, 1) | Claim |
| 4.10 Stale law | `law` joins the 04_3 fade | Test, claim |
| 4.11 ln is never named | `named`, "ln: the natural logarithm", and `back` wait for sentence 1 | Test, claim |
| 4.12 Blank stretches | `swap(stage, &graph, &marks, 0.6)` puts the ruler up as the voice says "ruler" | Test, claim |
| 4.13 Ratio ruler | The line grows on tracker `grown`; axis label "growth factor on a ratio ruler: each mark is ×2" | None |
| 4.14 Unnamed results | "rule of seventy: time ≈ 70 ÷ (percent a year)"; `far_word` "×1000" | Claim |
| 4.15 Far-off think bar | Under the question | Test |
| 4.16 Tangent over triangle | `tri`, `rise1` and `run1` fade at sentence 3 | Claim |
| 4.17 Rounded equals | `\approx`, and "100 × ln 2 ≈" | Claim |
| 4.18, part | Curve turns blue past m = 0.5, when edge-on; `\log_2:` through `tex_left`; `hit_foot` (fact `real.double_years`, 13.9); `ln_name` at p(2.75, 1.7) | Claim |

## Not repaired, and why

| Finding | Why |
|---|---|
| 1.14 No keyboard picture, no picture for closer matches | Needs a new picture or new numbers: a design decision, not a repair |
| 2.8 C's "10" margin | The review accepted it: 0.24 against a wanted 0.25 |
| 3.8 Unvoiced "240 / 2 laps" row | The review named no fix, and I did not choose between holding the row back and accepting it |
| 3.11 Busy end state | Needs a look at real frames first |
| 3.12 Pointer pop | Cosmetic; left to keep the patch smaller |
| 4.7 Hollow arrows | Shared `draw.rs`, which Mail 1447 says not to edit; see below |
| 4.18, the rest | Right-column left edges, a tie for the green dots, the empty right third of 04_5. Needs a look at real frames first |

## Where this departs from the review

- **No squeeze (1.1, 1.2, 1.10, 1.13).** The review proposed fading and a new bar. Holding every mark and changing only the unit fixes all four findings with one picture, and it is the truer statement of "a different base".
- **Inset 0.06, not the review's 0.03 (1.4, 1.5).** Strokes have round caps that reach half the width past each end (`fmn-render/src/stroke.rs`). At width 6 that is 0.03, so the review's inset leaves no gap at all. The visible gap is `2 × inset − width × 0.01`. The review was wrong on this point.
- **Local `until_phrase` (2.3).** The review cited `kit.until_phrase`, which does not exist. The helper follows `abstract_algebra_expanded/src/axioms.rs`.
- **`ln2_word` at p(1.1, −0.75), not p(1.5, …) (4.2).** This sits under the foot of the drop and clear of the red bar.
- **The richer option for 4.1.** The red marks reflect with the curve instead of fading, because a wall and a barred half-line are what ln's domain looks like.

## New on-screen literals

"×4", "×1000", "2 × 4 = 8" and "= 1" have no facts key. `facts.rs` and `oracle.py` are untouched. Chapter tests check the geometry these words describe, and state claims check that they are on screen. If root wants them as facts, that needs a change to both files.

## For root: hollow arrows

`draw::arrow` (`draw.rs:175`) strokes the arrow's outline with no fill, so it renders as a double shaft with an outlined head. `arrow_parts` (`draw.rs:182`) is solid. This is common to all reviewers' chapters, so I left it. Candidate call sites:

- `slide_rule.rs:168` (`carried`)
- `folded_decade.rs:179` (`tens`)
- `undoing_growth.rs:377` (`down`)

## Checks run

**None of this is native evidence (Mail 1485).** All of these ran with bare `rustc` and `clippy-driver` against hand-written stand-ins for fmn and the Kit. No Cargo, render or TTS was used. `facts.rs`, `model.rs` and `draw.rs` are the real baseline files, and the Kit's `Facts` is its real source. Output is saved beside each script in `checks/`.

| Check | Result |
|---|---|
| Chapter tests under stand-ins | Candidate 75 pass (46 chapter, 29 facts and model); base 41 pass (12 chapter, 29 facts and model) |
| `clippy-driver -D warnings`, with and without tests | Clean on the candidate |
| `rustfmt --check`, edition 2024, width 120 | Clean on all four chapters |
| Object ledger at 2.3, 2.68 and 3.1 words per second | 0 problems in each chapter |
| `state_checks.py`: 52 claims about what is on screen when | All hold on the candidate at all three rates; all fail on the base at 2.68 |
| `stub_sensitivity.py`: 17 deliberate breakages | All caught; the unchanged copy is clean |
| `patch_round_trip.py` | base + patch = candidate, and back; canonical + patch = candidate |

Existing tests: all 12 chapter tests survive. Two bodies changed. `every_mark_from_a_half_to_ten_fits_the_ruler` loses its squeeze assertion and adds 10 to the marks it checks. `twelve_threes_overshoot_nineteen_doublings_by_a_sliver` uses the new `ROW_SPAN`.

The estimated overlaps left on the candidate at 2.68 words per second:

- 02: "1" off frame, deliberate and pinned by a baseline test
- 02: C's "10" margin, finding 2.8
- 04: "2" meets the probe for 0.4 s, ruled harmless in the review

The other overlaps the base reports at rest are gone. In motion, 04 reports 12 pairs on both trees: tick labels swept by the curve, the flip and the probe, and "y = e^x" crossed by the probe. The base also has "y = e^x" meeting the curve and its mirror; the candidate does not.

Every newly called API was checked by reading its real definition at fmn pin `a399d626` or in `crates/explainer_kit`: `tex_left`, `text_left`, `narrator`, `sentence_at`, `sentence_starts`, `phrase_marks`, `length`, `frames_covering`, `wait_frames`, `attach`, `arena_mut`. `polyline` and `scene().time()` were checked against existing chapters that already call them. That is reading, not compiling.

## What the stand-ins cannot show

- A wrong real signature, a TeX failure or a missing glyph
- Any frame: text boxes are estimates from a CMR10 em table
- Real timing: the clock is words over rate, with no audio

Under that clock, 04_2's picture overruns by about 2.5 s (base 2.2), which pushes 04_3 back. Chapter ends are 123.0, 86.4, 72.1 and 180.0 s against the base's 123.0, 86.4, 72.1 and 179.7 s. Mail 1447 sets no duration cap.

The first length in 01_1 is drawn twice, once as `first` and once in the doublings loop, exactly on top of each other. Both trees do this and it should not show, so I left it.

## Root's checklist

1. Compile the real crate, then run tests, Clippy and `facts-check`. Check `const { assert! }` on the pinned toolchain.
2. Typeset `\log_2:\ \text{step 1} = \times 2`. Check "×" and "≈" in `kit.text` captions against the glyph probe.
3. Render new pictures and open at least these moments:
   - 01_6 sentences 3 to 5, the growing unit bar
   - 02_5, the bracket
   - 03_2, the landing
   - 04_4, the red handover from still to live marks and its 22 redrawn dashes per frame
   - 04_5, the product law
4. Check the opacity tones of `piece()` on both row colours.
5. On Oliver's real cues, check the `until_phrase` wait in 02_2 and the 04_2 overrun. Take reuse is unaffected, since no word changed.
6. Decide 3.8, 3.11, 3.12 and the rest of 4.18 from those pictures, and the hollow arrows across reviewers.

Any look I take at this repair is author review, not independent review. See `review-baseline/PROVENANCE.md`.
