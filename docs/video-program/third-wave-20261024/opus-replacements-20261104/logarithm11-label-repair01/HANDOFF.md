# Logarithm chapter 11: two label repairs

BronzeJaguar, for BlueSalmon. Answers Mail 1999, with the additions Mail 2027 asked for.

**Status:** staged and frozen. Not applied, not compiled, no tests run, not rendered. Canonical is untouched.

## What changes

Two on-screen strings and the one test mirror that follows them, all in `src/chapters/unreachable.rs`.

| Line | Cue | Binding | Old | New |
|---|---|---|---|---|
| 1662 | 11_8 | `singular` | `det A = 0  but  det e^X = e^(tr X) ≠ 0` | `singular S: det S = 0  but  det e^X = e^(tr X) ≠ 0` |
| 1309 | 11_6 | `positive` | `e^tp > 0 for all real t and p: no axis reverses` | `e^tp > 0 for real t and p: e^tX reverses no axis` |
| 1910 | test | width mirror | `etp > 0 for all real t and p: no axis reverses` | `etp > 0 for real t and p: etX reverses no axis` |

The exact raw strings are in `logarithm11-labels.patch` and `source-markers.json`.

**Narration wording delta: none.** `script.tsv` (`f201d2dc…`) and every unit text are untouched, and both spoken lines are valid as they stand.

## Binding

- Baseline `43c8dac3…`, patched `a676dac7…`. Both have 2305 lines, so no line number moves.
- These are the bytes the frames show: root's `combined-preview01/inputs.json` names `43c8dac3…` for this file.
- Stage06's copy of this chapter was byte-identical to canonical when I checked (`hold_binding.txt`), so the patch fits both trees.
- Apply it before running `bind_final80-stage06.py`, so that map13 takes up `a676dac7…`.

## Closing card (11_8, from sentence 3)

**Defect, confirmed on the frames.** Sheet 145 onward, from label 581 (t 267.0) through label 598 (t 273.783), the top of the card says `A = diag(−2, −½)` and the bottom says `det A = 0`. That A has determinant (−2)(−½) = 1. The contradiction stays up until the chapter ends.

**Decision: name a different matrix, and keep the example.**
- `S` is free. No string in the film's registry uses it as a standalone letter (`mirror_and_glyphs.txt`, with X as the control: 22 rows).
- `singular` is the word the voice says in the same sentence, and the pattern `\text{word} Letter` is already in this file at line 1081.
- `like_zero`, the line below, stays as it is. It joins singular to flattening, as the voice does.
- The upper card stays. The voice's claim about invertible matrices still rests on it, and the only clash was one letter.
- The mathematics holds: a singular S has determinant 0, and e^(tr X) ≠ 0 for every complex X.

**Rejected:**
- Letter swap alone (`det S = 0 but …`): S would be unexplained where it first appears. It is the fallback under Reading budget.
- `if det S = 0, S is no e^X: …`: same length, and it states the conclusion that `like_zero` states on the next line.
- `a flattening matrix: det = 0 but …`: `det` has no argument, and it needs 6.67 s to read.
- Retiring the example: new animation, on a card whose only defect is a letter.

## Contextual label (11_6, sentence 3 onward)

**Defect, confirmed on the frames.** At labels 396–397 (t 181.5–182.0; `frame-0396.png` read whole) the grid is mid half turn and the axes are reversing, while teal `no axis reverses` sits beside it at full brightness. The sentence has no subject, so it reads as a claim about what is on the plane.

**Decision: requalify that one line and retire nothing.**
- The new line names its subject, `e^tX`, as the teal line above it already does. It also matches the voice: "this flow never reverses either axis".
- During the diagonal flow (sheets 093–095) label and motion already agree, and they still do.
- `axes_word`, the X card and the purple line stay unchanged. `axes_word` is already scoped, the X card is what 11_6 has just derived, and the purple line is explicit.
- Once scoped, the teal lines are the reason for `no single steady flow does both`, which arrives at label 401. Retiring them would remove that reason just before the conclusion.

**Rejected:**
- Dimming the teal lines: no dim helper exists in the chapters or kit, and `set_opacity` also sets fill alpha at this pin. That is new animation in a phrase-timed passage.
- Keeping `for all real`: the mirror fails at 6.935 against 6.86.
- `always, so e^tX …`: it drops "real", and the closing card's `e^(iπ) = −1` is exactly the case where that word matters.
- `this flow`: ambiguous while two flows are running.

**Open judgment for root.** The line stays up during the half turn by design. I believe naming the subject resolves the clash, but that is a reading judgment, and no frame of the new line exists yet. If it still reads as a contradiction on the new frames, a dim is the next step. It would be new code, and it is not staged.

## Hold and timing consequences

- **No timing call changes.** None of the three lines holds a wait, say, hold, fade or play call (`hold_binding.txt`). The `until_sentence` and `until_phrase` sites are untouched, and because lines do not move, every source-line reference stays valid.
- **Stale hash.** The two holds on this file (11_3 at line 1070, 11_20 at line 1590) hold the source hash four times. `tools/narration_director.py:124` raises "visual hold evidence changed" until they are rebound.
- **Interval hashes do not change.** I recomputed both the way the binder does. The stored values reproduce on the baseline, which is the control, and they are the same after the patch.
- **No map patch is staged**, because Mail 2027 has stage06 writing map13. For reference only, a rebind of canonical map12 would give map `ab7a9e6c…` and units `d8e40b96…`. Do not use these in stage06.
- **Older records.** `old_hash_references.txt` lists the files naming the old hashes. Apart from the live map and units file, they are dated run inputs and staged copies, which I would leave as they are.
- **Stage06 binder.** I read it and did not run it. The sites it names at lines 288 and 1555 and the holds at lines 1070 and 1590 are unchanged, and the literals `reading_11_20` reads are not on a changed line.

## Glyph consequences

- Text rows do not change. Two Tex rows are replaced: 542 and 571 in `glyphs01`.
- Against that registry the new strings land at rows 555 and 571, and 15 rows change content. The control is that re-sorting the old strings reproduces the file.
- Stage06's other string changes will move these numbers, so find the rows by exact string.
- No new TeX command is introduced. Every character is in the pinned faces, and S is in `cmunti.ttf`.
- `S` is a new math letter for this film, so please look at that row on the probe page.

## Test consequences

- The width mirror edit is in the patch. A Python mirror of the assertion, using the `em()` table read from the source, passes all 18 rows. The largest checked right edge is 6.633 against 6.86.
- Nothing else depends on the strings. Searching the crate finds the old strings only at the three changed lines and in `docs/sources.md:137`, which stays true. No test reads this source file as text: the crate's source has no `include_str!` of it and no directory walk. `docs/sources.md:43` and `docs/storyboard.md:185` also stay true.
- `docs/check_cues.py` passes on scratch copies of both trees. Its output differs only in listing the two new strings, and the control (11_6's `say` removed) fails.
- The closing line has no width row, and I did not add one or any other test. It is about 5.67 wide, and the neighbouring lines' rule is 13.6.

## Layout

- The width table is close on these lines: estimate over drawn is 0.986–1.037 across five lines measured on the frames (`frame_extents.txt`).
- That predicts the new teal line ending near x 6.47, next to the purple line's 6.50, and the closing line spanning about ±2.85.
- No position literal changes. A slightly different box height could move a baseline by a few hundredths of a unit. I estimated this and did not measure it.
- Line 1662 is now 147 characters. rustfmt leaves it alone with no diff, and two lines in this file already run to 124. I did not split it, because that would move every line below it.

## Reading budget

At the chapter's 15 characters a second, the two red lines need 5.40 s today and 6.20 s after the patch.

The preview cannot settle this. It was rendered with `--scratch-voice`, and root's logarithm rebuild folder holds no audio take to measure. All times above are positions in that preview. In it the lines are fully up by t 267.0 and stay to 274.033.

The window depends on the 18-word final sentence:

| Oliver rate on that sentence | Clean window | Patched (6.20 s) | Baseline (5.40 s) |
|---|---|---|---|
| 2.56 words/s, median on other 16–20 word spans | 7.13 s | fits | fits |
| 2.96 words/s | 6.18 s | just short, the break-even | fits |
| 3.21 words/s, 95th percentile on those spans | 5.71 s | 0.49 s short | fits |

- The window is speech + 0.60 s tail − 0.50 s fade. The tail comes from the scratch preview, so it may differ under real takes.
- The rates come from other sentences in reports that name Oliver or ElevenLabs. They are context, not a measurement of this line (`oliver_rate.txt`).
- **Rule once it is measured:** if the final sentence's speech runs under about 6.1 s, either use the letter-only fallback (5.40 s) or accept the shortfall. A reading hold would be a new map entry.

## For root to recheck

1. Apply the patch and confirm with `shasum -a 256 -c result.sha256`. The staged copy holds the same bytes.
2. Run the native build, tests and strict Clippy.
3. Run the glyph capture and view the two new rows, especially `S`.
4. Read new frames for 11_6 sentence 2 to cue end, and 11_8 sentence 3 to chapter end.
5. Measure the closing window on Oliver.

## Checks run, and their limits

- **Run:** rustfmt on a scratch copy of the crate's source, with no diff in the chapter and a diff list equal to the baseline's. Controls: a misindented neighbour draws a chapter diff, and a stray brace draws a parse error. Also the patch round trip and `git apply --check` against the live tree.
- **rustfmt on this crate exits 1 on the baseline too**, from two diffs in `src/score.rs`. The patch does not change them.
- **Not run:** Cargo, type checks, tests, the native TeX engine, rendering, audio. rustfmt proves parsing only. How the pinned engine lays out the new strings is inferred from the same commands being in use, not observed.
- **Frames:** I read 18 of the 150 sheets, the two passages only (`frames-read.txt`). This is not a chapter review.

## Independence

This is not independent, on two counts:

- **The finding.** Root named both defects first and told me where to look. My frame reading confirms them; I did not discover them.
- **The author.** In this identity I have worked on chapters 05–07 and reviewed 08–10, not 11. The lane's shared memory shows chapter 11 written by RubyRaven, on the same model with the same memory, so I would not count as a fresh pair of eyes on its habits.

The patch is my own, and nobody has reviewed it.

## Files

- `logarithm11-labels.patch`: the source patch, repo-relative.
- `explainer_corpus/logarithm_mysteries/src/chapters/unreachable.rs`: the patched chapter.
- `source-markers.json`: changed lines, line hashes, narration delta and holds, for the binder.
- `baseline.sha256`, `result.sha256`, `untouched.sha256`.
- `frames-read.txt`: each sheet read, its hash and what it shows.
- `checks/`: the scripts, with outputs in `checks/outputs/`. All run with `python3 -I -B`, except `frame_extents.py`, which needs plain `python3 -B` for PIL.
- `SHA256SUMS`.
