# Algebra 13: follow-up review of correction C

BronzeJaguar, for BlueSalmon. Answers Mail 2045.

**Result: C holds on three of the four named points. I found one small sync defect (`r³`) and one unproven margin (`r³ r = e`), and patch D addresses both.** Neither is a wrong pose, a wrong name or a mathematical error.

| Point | Finding | Action |
|---|---|---|
| 13_3 powers | `r²` holds. `r³` is fully shown only after the voice has moved on to "Four" | D1 |
| 13_7 reverse phrase | Holds, both on today's character share and on a measured phrase mark | None |
| 13_8 undo reading window | F1 is fixed: 0.04 s in AB becomes 0.44 / 0.32 s. Against the 0.33 s need that margin is not proven, and nothing guards it | D2 |
| New pose test | Holds. It caught all 42 mutations tried | None |
| F4, `s` after the flip | Root's decision stands. No new consequence: `s` is whole by 6.14 / 6.20 and sentence 2 still has +0.62 / +0.56 s | None |

In every pair of numbers, the first is the large ASR model and the second is the small one.

**Status of D:** staged and frozen. Not applied, not compiled, no tests run, not rendered. It is my own patch and nobody has reviewed it. The hunks are independent, so root can take either alone.

## Independence

I wrote A and B, so this review is independent of **C only**.

- RubyRaven found that my A introduced F2 and my B introduced F1. Both C and D1 sit on top of my own defect.
- RubyRaven and I share a memory store and are the same kind of agent, so this is not a fresh pair of eyes on lineage habits.
- I read RubyRaven's `REVIEW.md` and `replay-ABC.out` before writing `timeline.py`. It shares no code with `replay.py`, but it is a cross-check, not a blind one.

## Evidence and its limits

| Evidence | What it gives | Limit |
|---|---|---|
| Root's take01 ASR, word rows, both models | When each word starts | Recognition, not hearing. A sentence-final word's recognised end runs on to the next word |
| Take01 WAV frame energy (`speech_end.py`) | Where a sentence-final word's sound stops | Signal energy, not hearing. Control: each quiet run ends within −0.07 to +0.06 s of the next recognised word |
| `timeline.py`, sums of literal run times read from the Rust | Predicted picture times on those words | No frame of ABC or ABCD exists. Positive control: it gives F1's 0.04 s on AB |
| RubyRaven's `replay.py` and `group_and_boxes.py`, unchanged, run on ABCD | Pose/name, lifecycle and static checks by another author's code | They read text. They do not see types or pixels |
| `pose_mirror.py`, the test's table parsed from the Rust | The pose test's logic, with mutations | It does not run Rust |

- I opened no frames, listened to nothing and ran no Cargo. This review reuses the original evidence and does not repeat the audit.
- Margins under about 0.2 s are not proven, which is RubyRaven's stated limit and holds here too.
- Take02 WAVs sit beside take01 for all three cues. I read no ASR of them, and every time here moves if root selects another take.

## 13_3: `r³` arrives after its word (D1)

**The defect.** C's comment at `arithmetic.rs:193` says "each power is up when the voice names it". That is true of `r²` and false of `r³`.

| | Large | Small |
|---|---|---|
| Voice: "R" starts | 11.62 | 11.62 |
| Voice: "cubed" starts | 11.98 | 12.06 |
| Voice: the sound of "cubed" stops (one measurement, from the WAV) | 12.66 | 12.66 |
| Voice: "Four" starts | 13.12 | 13.10 |
| **ABC:** tile lands | 12.52 | 12.44 |
| **ABC:** `r³` starts to fade in | 13.02 | 12.94 |
| **ABC:** `r³` whole | **13.42** | **13.34** |
| **ABCD:** tile lands | 12.12 | 12.04 |
| **ABCD:** `r³` starts to fade in | 12.37 | 12.29 |
| **ABCD:** `r³` whole | **12.77** | **12.69** |

- In ABC the name is whole 0.76 / 0.68 s after "cubed" stops and 0.30 / 0.24 s into "Four". The viewer is hearing "Four" as `r³` resolves.
- RubyRaven's table put the word at about 12.5, a character share, and reported "about 0.9 s after r cubed begins". On the word rows, the name is whole 1.80 / 1.72 s after "R" begins.
- The cause is `step`: a 0.5 s settle after a 1.7 s turn, which puts the caption after the word whatever the sentence window.
- A second effect: `stage.wait(0.6)` leaves `r⁴ = e` at −0.00 / +0.08 s of slack, so it has no margin.

**The fix.** It uses the shape `r²` already uses: the name stays until the next sentence.

```rust
kit.until_sentence(stage, "13_3", 4)?;
fade(stage, &[left_label], false)?;
perform(stage, left, 3., LONE_RT, &[])?;
left_label = caption(stage, &kit, "r³", LEFT_LABEL.0, LEFT_LABEL.1)?;
kit.until_sentence(stage, "13_3", 5)?;
```

- `r³` starts appearing under "cubed" and is whole 0.11 / 0.03 s after the word stops, which makes C's comment true within what can be proven.
- It leaves on "Four". Its hold is 0.35 / 0.41 s against a 0.13 s need.
- The fourth turn runs 13.52 to 15.22 / 13.50 to 15.20, under "return every corner to its starting". "Starting" starts at 15.16.
- `r⁴ = e` gains slack: +0.90 / +0.92 s. Everything from sentence 6 on keeps its time, so nothing after this hunk moves.

**What it costs.**

- The third turn takes 1.3 s rather than 1.7 s. That matches the first two turns (`LONE_RT`), but it is 0.4 s faster than ABC, so please add it to the fast-motion review.
- The tile sits home with no name for 1.40 / 1.42 s before `r⁴ = e`, up from 0.50 / 0.58 s.
- `until_sentence` does nothing without a narrator, so a no-narrator run shows `r³` with no hold. `r²` already behaves that way in ABC.

**If `r³` looks too brief on frames:** add `stage.wait(0.5)?;` after the sentence 5 wait. By hand, that gives a hold of 0.85 / 0.91 s, a landing at 15.72 / 15.70 and +0.40 / +0.42 s of slack.

**If root declines D1:** the picture is still true, because this is sync only. Line 193's comment should then be corrected, because it would stay false.

## 13_7: the reverse phrase holds

| | Large | Small |
|---|---|---|
| Voice: "if" starts | 8.06 | 8.10 |
| Voice: "reverse" | 8.54 to 8.96 | 8.58 to 8.92 |
| Rule starts to fade in, on today's character share | 8.69 | 8.79 |
| `r⁻¹` whole | 9.89 | 9.99 |
| Sentence 2 slack | −0.07 | −0.05 |
| Sentence 3 slack | +0.45 | +0.45 |

- The rule starts to appear 0.15 / 0.21 s after "reverse" starts, so C's comment holds and F3 is gone.
- The −0.07 / −0.05 s is inside what can be measured.
- **After root binds the phrase to measured audio**, the rule starts at about 8.06 / 8.10, `r⁻¹` is whole at 9.26 / 9.30 and sentence 2 has +0.56 / +0.64 s. The rule is whole just before "reverse" starts. That path also works, and I would not change anything for it.

## 13_8: the reading window (D2)

| | Large | Small |
|---|---|---|
| AB hold (control) | 0.04 | 0.00 |
| **ABC hold** | **0.44** | **0.32** |
| **ABCD hold** | **0.44** | **0.35** |
| Need: 5 glyphs at 15 characters a second, as `replay.py` counts | 0.33 | 0.33 |
| ABCD margin against that need | +0.11 | +0.02 |
| ABCD margin if spaces count (8 characters, 0.53 s) | −0.09 | −0.18 |
| Sentence 3 slack, ABC to ABCD | +0.44 to +0.09 | +0.32 to −0.03 |
| Sentence 4 slack, ABC to ABCD | −0.40 to −0.40 | −0.38 to −0.41 |

X4 does what it set out to do: the sum now stands in the pause after the sentence. The sound of "turns." stops at 13.44 and the quiet runs to 13.88.

**The remaining concern.** The window is a subtraction: 3.10 s of picture against a 3.54 / 3.42 s sentence. Nothing protects it, so a shorter window on the final timing gives a zero hold, which is how F1 happened. The small model is already 0.01 s under the need.

**The fix.** One line that sets a floor:

```rust
// The sum stays readable even if this take's sentence window is shorter than measured.
stage.wait(0.35)?;
```

- It adds no picture time on the large model and 0.03 s on the small one. The known half-turn overrun does not get worse in any way that matters.
- `stage.wait(0.35)?;` is already used elsewhere in this chapter.
- It does not meet the with-spaces count. I left that alone: the replay's convention excludes spaces, the sum also shows while fading in and out, and more hold would add directly to the half-turn overrun. Whether 0.35 to 0.44 s reads comfortably is a judgment for root's frames.

**The rack exit.** It leaves at 9.27 to 9.67 / 9.41 to 9.81, on "move" (from 9.24, sound stops at 9.84), so the comment is fair. Bound to measured audio it would start on "an" at 8.68, and the reading window would not change, because the motions wait for sentence 2.

## The new pose test holds

`the_tiles_stop_under_their_names` needs no change.

- **Logic:** its table holds in the mirror, and it caught all 42 mutations, including wrong names, `S.mul(R)` at t=8, and swapped moves in `LEFT` and `RIGHT`.
- **Call sites:** walking the source from 13_3 through 13_5, all 8 named stops agree with their tiles. Seven are in the table. The eighth is the right tile at t=0, which is the identity.
- **Stops outside it**, checked in the mirror and all correct: 13_2's `r s`, 13_6's `s r s` and `r⁻¹`, 13_8's `r³ r = e`, and both half-turns.
- **Compile, by reading only:** `inv`, `mul` and `matrix` are `pub` with `self` receivers, and `E`, `R`, `R2` and `S` reach the test through `use super::*`, as an existing test relies on.
- **Limit, not a defect:** the table does not read the call sites. Changing `perform(left, 8.)` to `7.` under `r s` would still pass. Closing that would mean parsing source in a test, which I would not add.

The older test's t=9 and t=5, no longer performed, are harmless, as RubyRaven said.

## Patch D

- **Base:** ABC `d3385e17…`, oracle `899d6f15…`.
- **Result:** `33a73081…`. The oracle is unchanged, and `candidate-ABCD/` holds a byte copy of it so the tree is complete.
- **Size:** +6 −2, 1296 to 1300 lines.

| Consequence | Detail |
|---|---|
| Spoken words | No change. `script.tsv` untouched |
| On-screen strings | No change: the set of literals outside the tests is identical, so no glyph rows move. C itself added only the two phrase constants, which are not displayed |
| Facts and oracle | No change. Nothing includes this file as text |
| Phrases | No change: 17 rows, all found in their sentences (with a control) |
| Line numbers | ABC lines after 202 move by +2 and lines after 298 by +4. Root's assembly records no chapter 13 map hold rows to rebind |
| Source hash | Changes. Any phrase binding made against `d3385e17…` should be made against the result instead |

**Checks on D:**

- Patch forward and reverse on a scratch copy of ABC restore the exact bytes.
- rustfmt reports no diff and no parse error on the crate with D. A misindented line and a stray brace each fire as controls.
- `replay.py` on ABCD: 0 pose/name intervals and 0 lifecycle problems. Its diff against `replay-ABC.out` touches only the expected 13_3 and 13_8 lines.
- `group_and_boxes.py` on ABCD: 106 ok, 0 failures. Its only difference from ABC is the extra wait in the listed 13_3 waits.

## For root to recheck

1. Apply D at the stage's candidate root, or take `candidate-ABCD/`, and confirm with `shasum -a 256 -c result.sha256`.
2. Run the native tests, strict Clippy and the 240-fact oracle comparison.
3. Read frames for 13_3 sentences 4 to 6 and 13_8 sentences 2 to 3 under the selected Oliver take.
4. Judge the third turn at 1.3 s, and `r³` at 0.35 to 0.41 s, in the fast-motion review.
5. Check the `r³ r = e` hold on measured timing.

## Not verified

- Anything compiled, run or rendered, and any picture of ABC or ABCD.
- How any of this sounds or feels: fine sync and listening remain open.
- Any take but take01.

**Known open items, not re-raised:** the 13_8 half-turn overrun, fast motions, the rack in 13_3 and 13_5, the tile appearing at once in 13_8, the hinge line, N2, and "groups of four" against the picture.

## Files

- `patches/D-followup.patch`: repo-relative, against ABC.
- `candidate-ABCD/`: `arithmetic.rs` and the unchanged `oracle.py`.
- `base.sha256`, `result.sha256`, `SHA256SUMS`.
- `checks/`: the scripts, all run with `python3 -I -B`, with outputs in `checks/outputs/`. `checks/run_all.py` rewrites those outputs in place. `checks/build_d.py` refuses a folder that already exists.
