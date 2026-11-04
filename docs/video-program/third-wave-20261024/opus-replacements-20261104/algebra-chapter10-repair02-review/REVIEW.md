# Independent review: algebra chapter 10 picture repair02

**Reviewer:** RubyRaven, for BlueSalmon's Mail 1696 (Bead 87do.64).
**Reviewed:** root's candidate `algebra-chapter10-picture-repair02/cyclic.rs`, sha256 `105f5cf8…`, against baseline `before-cyclic.rs`, `946f1da4…`.

**Nothing here has been compiled, rendered, measured from a render, or heard.** Every frame I looked at shows the baseline, so no frame is evidence about how the candidate looks. Narration stays ElevenLabs Oliver and no spoken word changes. No take is selected for these cues, so listening, fine sync and independent final AV acceptance all remain open.

## Verdict

**I found no blocker, and the maths holds.** From source and engine reading, the box repair and the trivial-case separation do what root intended, and showing hidden objects again looks sound. The recap is in the right sentences, but most of its beats run 2.4 to 3.5 s ahead of the words they illustrate. Root's first native frames still have to confirm all of this.

I recommend taking `proposed.patch` before the guarded native run. It's 26 added and 9 removed lines against the candidate, and each finding's change can be taken or left alone.

| # | Severity | Finding | In root's change? |
|---|---|---|---|
| F1 | Medium | 10_11 recap beats finish well before their words | Yes |
| F2 | Medium | 10_10's two results appear about 8 and 10 s before they're said | No, already in the baseline |
| F3 | Low | The exponent line is built before the invitation, not during it, as the comment and README claim | Yes |
| F4 | Low | The last pulse highlights exponent dots while the voice talks about moves | Yes |
| F5 | Low | The path pulse lifts the triangle off its ticks and reaches the `+8` label | Yes |
| F6 | Low | No test row guards 10_11's sentence indices | Yes |
| F7 | Housekeeping | Absolute home paths in root's two bindings files | Yes |

## How to read the times

All times are seconds from the start of the cue, from `checks/timing.out`. **take 1** and **take 2** are the real Oliver takes, with word onsets from root's large-model ASR. Those are provisional estimates, not listening. **Scratch** is the voice in the preview pictures, where sentence starts are shares of the line. "Lead" is how long the picture starts before the word.

The script replays the source's own run times and the kit's wait rule. Where the kit has no phrase mark, it estimates a phrase's time from its share of the sentence's characters. For the six new hooks that estimate lands within 0.40 s of the ASR onset, so the proposal is usable with or without marks.

## Findings

### F1. The recap's beats run ahead of the voice

Root's README says the recap is timed to four existing sentences, and it is. But each sentence runs about 5 to 9 s, and its beats are finished within about 2.4 s of the sentence starting. In sentence 2, the chunks and the ring pulse are done before the voice has reached "divide exponents".

| Beat | Word it illustrates | Word at (take 1 / take 2) | Candidate starts | Lead | Proposal starts |
|---|---|---|---|---|---|
| Path drawn | "repeating a move" | 1.46 / 1.46 | 0.80 | +0.66 | 1.69 / 1.66 |
| `ord(8)=3` | "measures its order" | 5.08 / 4.98 | 2.00 | +3.08 / +2.98 | 5.39 / 5.29 |
| Pulse | "predicts that return" | 9.40 / 9.28 | 7.02 / 6.90 | +2.38 | 9.54 / 9.49 |
| Chunks of d | "divide exponents" | 14.96 / 15.00 | 12.52 | +2.44 / +2.48 | 14.98 / 14.91 |
| `d` ring pulse | "the smallest positive step" | 17.08 / 16.86 | 13.56 | +3.52 / +3.30 | 16.90 / 16.78 |

**Proposal.** Add five `until_phrase` hooks in the file's existing style. Also slow the path from 1.2 s to 2.2 s, so the triangle closes as the voice reaches the first return: it closes at 3.89 / 3.86 s against "and the first" at 3.90 / 3.88 s. No beat is late on any of the three voices.

### F2. 10_10 states its results before the voice does (already in the baseline)

This isn't one of root's changes, but it's the largest lead in the chapter's closing stretch. `squares` waits for sentence 2, "Each is relatively prime…", and `no` waits for sentence 3, "Five squared…".

| Text on screen | Starts (take 1 / take 2) | Said at (take 1 / take 2) | Lead |
|---|---|---|---|
| `5²≡7²≡11²≡1` | 10.54 / 10.54 | "Five squared" 18.40 / 18.20 | +7.86 / +7.66 |
| "No element generates all four" | 18.40 / 18.20 | "No single one" 28.36 / 28.14 | +9.96 / +9.94 |

Baseline `sheet-134` shows `no` fading in at 241.0 s under the scratch voice, a little over 7 s before its sentence, which starts near 248.6 s by share.

**Proposal.** Change the two hook indices, 2 to 3 and 3 to 5. Each picture then starts on the first word of its sentence. Both indices are inside the line, and the test already counts 10_10 at six sentences.

The multiplication table also fades in about 3 s before "Consider the numbers". I left that alone, because waiting would leave the screen empty after the proof fades.

### F3. The line is built before the invitation

The comment says "Build the exponent line during the invitation to see the proof", and the README says the same. The invitation is sentence 1, "Let's prove it…", at 5.52 / 5.78 s. The candidate finishes the line at 2.58 s, about 3 s earlier. The baseline was the version building it during the invitation.

Removing the hook does cure what baseline `sheet-085` shows, a title over an empty screen. But it also starts 42 parts moving at the moment the title lands, when the viewer needs about 2.8 s to read 42 characters at 15 per second. It then leaves the picture still for longer than the baseline's empty stretch.

| Take 1 | Title alone | Line builds | Still until "Suppose" |
|---|---|---|---|
| Baseline | 4.52 | 5.52 to 7.09 | 0.99 |
| Candidate | 0.00 | 1.00 to 2.58 | 5.50 |
| Proposal | 2.21 | 3.21 to 4.79 | 3.29 |

**Proposal.** Wait for "of a cyclic group" in sentence 0. The title gets 2.2 s alone, a little under the reading figure, while the voice is saying the same words. The line is then finished before the invitation on all three voices. This is a judgment call and root's to make. If root prefers the candidate's timing, the comment and README still need correcting.

### F4. The last pulse points at the wrong thing

`pulse(&members)` runs on "Next we will make the moves themselves into permutations of labelled objects". The pulsed dots are exponents on the number line, not moves. Moves aren't on screen at that point, and the comment's "positions" doesn't change that.

**Proposal.** Drop the pulse and the wait before it, which no longer has anything to wait for. `k.finish` holds the proof to the end of the line, as the baseline's ending did.

### F5. The path pulse leaves the clock

`Indicate` scales about the object's own centre. The triangle's centre is at y = 0.375 and the clock's at y = −0.1, so at the 1.25 peak the base drops to −1.406 and the apex rises to 2.156. The `+8` label is centred at y = 2.1, so the apex reaches it for the middle of the 0.6 s pulse. That's arithmetic on the candidate. For the motion itself, baseline `sheet-064` (frame 256, 114.5 s) shows the same thing on `triangles[0]`, with the base off its dots. There's no label above the triangle in that scene.

**Proposal.** Pulse `recap_order` only.

### F6. No guard on 10_11's indices

`every_waited_phrase_is_in_its_sentence` has no 10_11 row, so nothing checks the new hooks' sentence indices. **Proposal:** add `("10_11", 4)`, plus `PHRASES` rows for the new phrases.

### F7. Absolute paths in root's folder

`bindings.json` and `bindings-after-format.json` carry an absolute home path in `source`. The folder is frozen for me, so I didn't touch them. They need a relative path before they're committed.

## Checked, no defect

| Area | What I checked | Evidence |
|---|---|---|
| Patch bytes | `cyclic.patch` takes the baseline to the candidate, byte for byte; `rustfmt --check` passes on the candidate | `checks/patch_check.out` |
| Hiding and showing again | Removal is "a draw-list edit, never a deletion" and handles stay valid. `begin_animations` begins first and then roots, and `fade_in` starts at zero opacity, so a re-show shouldn't flash. Group children survive, because removal doesn't touch family edges | Engine pin `a399d626`: `fmn-mobject/src/stage.rs`, `fmn-anim/src/frame.rs`, `transform.rs`, `fading.rs`; read, not run |
| Redrawing `whole` | `ShowPartial` is a function of its begin snapshot and alpha. `AnimationGroup` begins every member at zero, so arcs waiting on the lag stay hidden. A child may have any number of parents, so a second lag group is legal | `fmn-anim/src/creation.rs`, `composition.rs`, `stage.rs` |
| No animation of a hidden object | Every re-show goes through `fade_in` or `show_creation` first, and each pulse follows its show | Source, lines 990–1004 and 1312–1352 |
| Hide lists complete | Baseline frame at 281.5 s shows only the table, `squares` and `no`, all of which are in `table` after the `extend`. All `recap_proof` objects are in 10_10's `proof` hide. The recap's own hide lists all four of its objects | `frame-0625`, source |
| Trivial case | The example leaves at 12.96 and returns by 18.28 on take 1, after about 4 s on screen, so it doesn't read as a flicker. Baseline `sheet-090` shows the defect this fixes | `checks/timing.out`, source |
| The sweep's budget | The added 0.3 s still fits before "and choose the one": the sweep ends 20.73 / 21.05 / 19.71 against the hook at 21.80 / 22.19 / 21.13 | `checks/timing.out` |
| Overruns | No beat is late on any voice, in the candidate or the proposal. So the chapter's length, and with it the score stem, shouldn't change. Only a native run can prove that | `checks/timing.out` |
| Box | Tick fully inside (y −0.17 to 0.11, box from −0.4). Orbit text ends 150 px left of the box, and still clears it at the pulse. Above the `3` the gap is 0.50, below the tick 0.23: not centred, but not a collision | `checks/geometry_math.out`, `frame-0155` |
| Recap layout | Formula at 32 pt is about 4.05 wide (scaled from 36 pt in `frame-0264`). It clears the dial by about 2 units and `ord(8)=3` by about 0.5. The returning proof uses a subset of a layout that's already clean | `checks/geometry_math.out`, `frame-0501` |
| Maths | Walk 0, 8, 4; order 3; 12/4 = 3; members −8, −4, 0, 4, 8, 12; d = 4; 5 chunks; ⟨g⁴⟩ = {0, 4, 8}. Recomputed from the definitions, and matching `frame-0501` | `checks/geometry_math.out` |
| Compile, by reading only | `d` (line 578) is in scope in the same `construct`, shadowed only in tests. `Mob` is `Copy`. `table` is the second `let mut` at 1269. The recap's TeX evaluates to strings the chapter already sets | Source |

## What the first native frames must confirm

These can't be settled from source. A text scan of the corpus's scene source found **no example of the same handle being hidden and then shown again** (`checks/reshow_scan.out`). It only sees calls written on one line, so it doesn't prove there are none, but treat this as the first time a film depends on it. Times are take 1; the render's will differ with the take.

1. **10_3, from about 14 s:** the box clears the tick.
2. **10_7, 1 to 5 s:** title, then line.
3. **10_7, 13 to 18.5 s:** the small clock returns with all its children and the green dots, with no flash.
4. **10_11, 0 to 7 s:** the table fades fully, then the clock, then the formula.
5. **10_11, 11.7 to 16 s:** the proof returns with no flash, and the arcs redraw left to right with none showing early.
6. **The chapter's length** matches the baseline for the same takes.

## Applying it

Root's call, after root's own review. `proposed.patch` applies to the candidate's bytes, and `proposed-cyclic.rs` is the result. The source hash changes again, so the source-bound directions root's README already mentions need rebinding either way.

| To take | Edits |
|---|---|
| F1 | The five `RECAP_*` hooks, their constants and rows, and the 2.2 s path |
| F2 | The two 10_10 indices, which need nothing else |
| F3 | The hook, its constant and row, and the comment |
| F4 | Remove the wait, the comment and the pulse |
| F5 | `pulse(stage, &[recap_order])`, which F1's hook sits directly above |
| F6 | The `("10_11", 4)` row |

**If you take only part of it:** remove the constant and the `PHRASES` row for any hook you drop, and set the array's size to match. An unused constant would be a dead-code warning.

The proposal has only been through `rustfmt` and a Python mirror of the phrase test. It hasn't been compiled.

## Run the checks

From the repo root, with `R` set to this folder and `EMPTY` to a new empty directory outside any git repository:

```
python3 -I -B $R/checks/timing.py
python3 -I -B $R/checks/geometry_math.py
python3 -I -B $R/checks/patch_check.py $EMPTY
python3 -I -B $R/checks/reshow_scan.py
python3 -I -B $R/checks/make_proposed.py
shasum -a 256 -c $R/base.sha256
shasum -a 256 -c $R/bundle.sha256
```

`make_proposed.py` rewrites `proposed-cyclic.rs` and `proposed.patch` from the candidate. The others only read, apart from `patch_check.py`'s scratch directory. `base.sha256` pins my inputs, including the frames I viewed, and `bundle.sha256` pins every file here except itself.

The baseline frames I viewed, in `algebra-combined-repair-frames01/10_cyclic/`: `frame-0155`, `frame-0264`, `frame-0501`, `frame-0625`, `sheet-064`, `sheet-065`, `sheet-085`, `sheet-090`, `sheet-134`.
