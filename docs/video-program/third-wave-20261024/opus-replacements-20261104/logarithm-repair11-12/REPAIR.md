# Logarithm chapters 11 and 12: staged repairs

RubyRaven, for BlueSalmon, Mail 1459. **This is author staging, not independent acceptance.** The findings come from my own review (`logarithm/preview02-review-11-12/REVIEW.md`, frozen and unchanged).

No canonical file, shared `draw.rs`, script, take, Bead or commit changed. There was no Cargo run, render or synthesis. Narration remains ElevenLabs Oliver. No line needs new words, so no line needs a new take.

## Checked, and not checked

| Check | Result |
|---|---|
| `rustfmt --check` on both candidates | Pass. Proves parsing and format only |
| The 16 new tests, lifted verbatim with their pure helpers, under plain `rustc --test` | 16 of 16 pass (`checks/pure_tests.out`). `model.rs` and `letter_f` are the repo's bytes. `Vec3`, `v()`, a small `Script` parser and the fact lookups are stand-ins (`checks/pure_harness.py`) |
| `clippy-driver -D warnings` on those lifted files | Clean, lifted code only |
| Python mirror: layout, scratch timeline, fade-list scan | 115 checks, 0 failures (`checks/mirror.out`) |
| Patch applied to scratch copies of the baselines | Reproduces both candidates byte for byte |
| **Scene code against the fmn API** | **Not compiled.** Only root's build can show this |
| **Any picture** | **Never rendered or seen.** Every layout claim is computed |
| **Sound and Oliver timing** | **Not heard, not measured.** Every time below is the kit's scratch voice |

## Repairs

Lines are in the candidates under `explainer_corpus/logarithm_mysteries/src/chapters/`.

| ID | Cue | Change | Lines |
|---|---|---|---|
| R1 | 12_6 s1 | The 1½ run lands by "also ends here" and holds until "and so does two and a half", with a 0.5 s floor so a short take can't bring back the one-frame arrival. The 2½ run is shortened if it would leave the chip no room before s2 | `what_it_forgets.rs` 976–994 |
| R2 | 12_2 | Gold dot on the rim and gold "2.4" outside it, in the fade and in the `panels` list | `what_it_forgets.rs` 719–726, 733, 840 |
| R3 | 11_3, 11_4 | `set_z_index(e1, 1, true)` just before the gold F's fade, so the arrow never goes under it and doesn't pop back | `unreachable.rs` 945 |
| R4 | 11_5 s2 | `routes` gains `show_gap`. Bar and caption land together on "the two routes end in different places" | `unreachable.rs` 352, 1049–1053, 1059 |
| R5 | 11_7 s3–s5 | See below | `unreachable.rs` 1248–1370 |
| R6 | 11_1 s4 | See below | `unreachable.rs` 715–740 |
| R7 | 11_1, 11_3 | The "before" F is `flat(Palette::dim(), 0.65)`, predicted 3.25:1. Re-measure on frames | two sites |
| R8 | 11_3 | Both arrows use `arrow_parts` with filled heads. `arrow` left the imports; `draw::arrow` is untouched | two sites |

### R6: the whole plane collapses

1. **"Averaging the two matrices":** the first caption, as now.
2. **"a halfway point":** red dots at the midpoints of the four dashes, all on the vertical axis. The "before" and "after" words leave, since the grid is about to cross them.
3. **"squashes the whole plane":** the dashes fade first. They "only pair" corners, and moving corners must not read as travelling along them. The grid and a red F outline then redraw live under `lerp(I, mirror, k)`, k from 0 to ½.
4. **As the collapse completes:** a full-height red axis line and the second caption.

The live grid takes the static grid's slot in the draw list through `Scene::replace`, so it stays under both Fs. At k = 0 the two match. At k = ½ the horizontal lines are single points, so the grid switches to the one line it has become rather than draw zero-length strokes.

### R5: motion under the rule

**s3, "so nothing forces a logarithm to keep the axes".** The plane rewinds to the identity with white axes marked. On "the spin, which turns every axis" it runs the same half lap with the axes turning. Each axis ends back on its own line, reversed.

I first planned an extra full lap and dropped it. That lap would put 1½ laps on screen under a caption reading "½ lap", and it would show the answer before 12_5 asks.

**s4, a direction sweep on a faint identity grid.** A white direction turns, a gold arrow shows where A sends it, and dashes run straight back. Gold lies on the dashes exactly when the direction is reversed. A purple mark stays on each such direction, decided by `reverses(m, d)`, not by a list of angles.

| Phrase | Matrix | What shows |
|---|---|---|
| "a matrix that sends some direction" | diag(−2, −½), half a lap | Marks at the two axes only |
| "only when it does that to every direction alike" | −I, a whole lap | An unbroken purple ring |
| "a half turn combined with an equal stretch" | −cI, c grows to 1.5 (never printed) | Gold grows along the dashes |

**s5, block rows.** Tiles arrive during the citation. Outlines and verdicts come on "the matrix's blocks for each negative stretch", and gold brackets on "come in matching pairs".

| Row | Verdict | Why it is true |
|---|---|---|
| [−1][−1] | matching pair: real log | −I = e^{πJ} |
| [−2][−½] | no partners: no real log | Each stretch has one block; this is the chapter's own A |
| [−2][−2][3] | −2 paired, 3 positive: real log | (ln 2·I + πJ) ⊕ ln 3 |
| [−1 1; 0 −1] | one size-2 block: no real log | See below |

Verdict colours come from `pairs_up`, and a test ties each verdict's wording to it.

The last row: if a real 2×2 X has e^X with eigenvalue −1, X's eigenvalues are ±iπ(2k+1), a conjugate pair. Then X is diagonalisable over the complex numbers, so e^X is too, and e^X = −I, which is not a Jordan block. The new test `a_real_spin_with_any_shear…` checks that identity numerically. The statement itself remains the Higham and Gallier condition already cited on screen; Culver's own text is still unread.

## The phrase hook

`until_phrase` (`unreachable.rs` 197–260, copied in `what_it_forgets.rs` 532–595, because shared `draw.rs` is out of scope):

- **With a mark:** it uses `Narrator::phrase_marks` when that word has one.
- **Without one (provisional):** it takes a share by characters of the measured sentence. This is a draft cue, not a heard boundary.
- **Today, every hook is provisional:** the narration root holds 12 `phrase-timings.json` files, for abstract, paradox and golay, and none for logarithm. Once a complete one bound to logarithm's WAVs and text exists, the hooks use its marks with no code change.
- **Never a guess:** a phrase missing from the line, or a sentence-count mismatch, stops the render with a named error. No Oliver time is written anywhere.
- **In a last sentence:** the sentence closes at the line's length, tail included. On scratch that puts 11_1's "squashes" at 24.75 against 24.51 in my review's table, so the picture is slightly late, not early. Only 11_1's two hooks fall in a last sentence.

The script is read through `include_str!("../../script.tsv")`, the same path `chapters/mod.rs` uses in its test.

The existing `phrase-timings.json` files mark only chosen phrases, by the byte where each starts. A hook uses a mark only if it sits on its exact byte, and otherwise falls back to the share. A logarithm index should therefore mark these twelve:

| Cue | Byte | Phrase |
|---|---|---|
| 11_1 | 320 | a halfway point |
| 11_1 | 337 | squashes the whole plane |
| 11_5 | 178 | the two routes end in different places |
| 11_7 | 174 | so nothing forces a logarithm to keep the axes |
| 11_7 | 226 | the spin, which turns every axis |
| 11_7 | 315 | a matrix that sends some direction |
| 11_7 | 404 | only when it does that to every direction alike |
| 11_7 | 453 | a half turn combined with an equal stretch |
| 11_7 | 588 | the matrix's blocks for each negative stretch |
| 11_7 | 634 | come in matching pairs |
| 12_6 | 89 | also ends here |
| 12_6 | 105 | and so does two and a half |

## Scratch timeline

Scratch voice only, from `checks/mirror.out`. By this arithmetic each last beat ends before its limit, so no cue should move on scratch. Root can confirm that from `cues.tsv`.

| Cue | First new beat | Last new beat ends | Limit |
|---|---|---|---|
| 11_1 s4 | 19.70 | 27.82 | Line end 27.92 |
| 11_5 s2 | 139.74 | 140.14 | s3 142.64 |
| 11_7 s3 | 197.31 | 205.30 | s4 206.01 |
| 11_7 s4 | 206.01 | 220.41 | s5 221.48 |
| 11_7 s5 | 221.48 | 231.34 | s6 232.53 |
| 12_6 s1 | 107.56 | 116.66 | s2 117.78 |

In the 1½ run, the arrival holds 110.86–112.56.

The longest stills in 11_7's s4 and s5 were 15.0 s and 10.5 s. The longest gaps between new beats there now compute to about 2.6 s and 1.9 s. Those are computed gaps, not measured frames.

The run times are clamped. Scaling the scratch line by 0.8, 0.9, 1.1 and 1.25 left every limit with time to spare, 0.10 s at least. Those factors are arbitrary stress values, not predictions of any take. A real take's sentences won't scale evenly, so this needs rechecking on Oliver.

## Decisions for root

1. **New on-screen numbers.** The rows show −1, 3 and the size-2 block, which aren't in `facts.rs` or `oracle.py`, both out of my scope. They are chapter-local constants checked by tests. The −2 and −½ are tested against `matrix.flip_pair`. The film's rule is that every number is a fact, so you may want to add keys.
2. **Storyboard.** `docs/storyboard.md` is unchanged and doesn't describe these beats. Its line "09_1 and 11_1: endpoint comparisons, never paths" now needs a qualifier, because 11_1 shows motion from I to the average. I think the motion is honest: it stops at the average, never reaches the mirror, claims no flow, and the dashes leave first. But it is your call against that rule.
3. **Reading load at s6, not caused by this repair but made worse.** The four-line rule has 159 characters and about 3.8 s before the 11_8 fade on scratch: about 42 chars/s against the 15 guideline. That timing is the baseline's, and my review didn't list it. The verdicts alone are about 12.5 chars/s, but they now share the screen with it. A fix needs a longer gap before 11_8, which moves a cue, so I left it.
4. **Duplicate hook.** The same 60 lines live in both chapters. Moving them to `draw.rs` later would be a plain refactor.

## Integrating

1. Check `baseline.sha256`. Root edits canonical between syncs, so if a hash differs, use `git merge-file` rather than the patch.
2. Apply `repair11-12.patch` from the repo root with `-p1`, or copy the two candidates.
3. Build, test, Clippy and format. Expect 12 new tests in `unreachable.rs` and 4 in `what_it_forgets.rs`. The compile risk is in the scene code, above all these calls, which no compiler has seen in place: `stage.scene_mut().replace(..)`, `stage.set_z_index(..)`, and the `BlockRows` partial moves.
4. Run the guarded preview and read the frames.

**Look at these first:**

- **11_1:** no pop at the grid swap, grid under both Fs, no zero-length artefact at k = ½, and the pop from nine stacked lines to one is hidden by the red line.
- **11_3:** arrow over the gold F during the fade; filled heads.
- **11_5:** no red before "the two routes".
- **11_7:** gold on the dashes only at the axes; ring complete; size-2 card inside its outline; rows and heading clear of the column.
- **12_2:** dial label legible at 14 pt.
- **12_6:** arrival pose held, then a clean run.

Re-measure R7's contrast, and re-run the layout lint, since the new text and strokes have never been linted.

## Still open

| ID | State |
|---|---|
| R9 | Open. Reds crowd the end ring at 12_6 s3. Not touched |
| R10 | Open. R1's hold softens the first reset only; the pose still cuts at it and at the other three |
| R11 | Open. F's bar on the gold axis in 11_6. The new 11_7 axes are white, so they don't repeat it |
| R12 | Open. Only the 11_5 gap bar moved. Routes at 11_5 s1, the 11_2 backward run and the 11_6 journey still lead the words. The hook could bind them once Oliver is checked |
| R13 | Open. Tight stack at 12_2 |
| R14 | Open, all items, including the stale "uncompiled, unrendered, unlistened" headers, which stay literally true of these candidates. The "plane empty 188.0–190.5" leftover in 11_7 s0–s1 also stands |

## Files

- `explainer_corpus/logarithm_mysteries/src/chapters/unreachable.rs`, `what_it_forgets.rs`: candidates with tests.
- `repair11-12.patch`: both files, against the baselines.
- `baseline.sha256`, `candidate.sha256`: run `shasum -a 256 -c` from the repo root. The baseline list includes the frozen review and the inputs that must not change.
- `checks/mirror.py`, `checks/mirror.out`: the Python mirror.
- `checks/pure_harness.py`, `checks/pure_tests.out`: the lifted tests.
