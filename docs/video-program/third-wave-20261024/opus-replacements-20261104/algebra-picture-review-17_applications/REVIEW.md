# Expanded algebra 17_applications: actual picture review

RubyRaven, for BlueSalmon, Mail 1790, Bead mx-video-program-program-87do.64 (root alone edits Beads).

**Verdict: not acceptable as rendered.** The maths on screen is right and the quiz answers stay hidden. There is one visible render defect, one diagram that contradicts its own caption, and in most of the ISBN half and the recap the picture finishes well before the voice gets there and then sits still. A narrow native patch is staged here. It is uncompiled and unrendered.

## Disclosure

I did not write `applications.rs` in my RubyRaven work this session. A mail search for that file finds only Mail 880 from LavenderPike, and the shared memory note `algebra-rebuild-lane-state` records LavenderPike staging chapter 17. That is a different session and identity, but I share that memory directory and can't rule out that it was the same model, so don't count this as fully independent of the chapter's author.

## What these findings rest on

These are scratch pixels: 1280x720, 30 fps, CRF 26, `--scratch-voice --lint`. This is **not** narrated sync, listening or final AV acceptance.

- **No Oliver evidence exists for this chapter.** The take folder `elevenlabs-expanded-algebra-20261103/` has no `17_*` file, and there is no `expanded-algebra-17-*` ASR folder under `five-film-release-20261027/`. All 17 cues still need takes.
- **Sentence times are estimates.** Every "voice at" time below is the scratch voice's sentence start by share of characters (`checks/measure.out.txt`), not a measured word time. A claim that the picture leads or trails by N seconds is good to the size of that estimate. Findings where the picture finishes within seconds of a line's start hold whatever the voice does.
- **Frames are samples.** `sampling.json`: "Half-second, cue-boundary and layout anchor/persistence samples; later observations may be nonconsecutive. Not continuous-motion or listening proof." I saw no motion, only sampled stills.
- **Static holds rest on reading the sheets, not on a measurement.** The byte-identity scan found no run of 10 s or more, because the encode is lossy, so it adds no support either way.

## Exact coverage

| Item | Coverage |
|---|---|
| Contact sheets | All 238 (`sheet-000` to `sheet-237`), read in order as images and logged in `viewed-log.tsv` as each was read |
| Sampled frames | All 949 (`frame-0000` to `frame-0948`), each read once as a tile on its sheet (sheet k holds frames 4k to 4k+3; sheet 237 holds frame 948 only) |
| Full-resolution frames | 33, listed with time, hash and what they show in `viewed-frames.tsv` |
| Other chapter | `11_permutations` sheet 282 and frame 1133, only to confirm the `×` glyph in `k.text` |
| Source and script | `applications.rs` read in full (1226 lines), plus the 17 script lines |
| Cue and layout evidence | `17_applications.cues.tsv`, `17_applications.layout.tsv` (2 entries), `resource-receipt.json` |
| Hashes | `base.sha256` has all 238 sheets, `sampling.json`, the full-resolution frames, cues, layout, receipt, inputs list, script and source |

**One limit on that coverage.** Older images drop out of my working context once a request carries too many, so sheets 000 to 171 were no longer visible to me when I wrote this. Their log rows were written at viewing time. Every finding that cites a frame in that range was re-read at full resolution in the last pass. Timings marked "(log)" below come from the log row and were not re-read.

**Binding.** The preview's `inputs.json` records `applications.rs` 7cc3f0d0 and `script.tsv` 83619d48, and both match the current canonical files, so these frames show the current source. The receipt shows exit 0, with `inputs_sha256` equal to `final_inputs_sha256` (50c72ae2).

## Findings

Source lines are baseline 7cc3f0d0. F1 to F19 and L1 each have edits labelled with their number in `make_proposed.py`. N1 to N9 are not patched.

### High

| # | Where | Defect | Repair |
|---|---|---|---|
| **F1** | Frames 230 (102.733), 231 (102.933), 232 (103.000). Lines 130, 149 to 152 | **Ghost at the 17_5 wipe.** The old 6 sits over the red 7 and the old 42 over the red 49, at about full opacity, then fading. Frames 228 and 229 are clean (log). Line 130 already faded `digits[3]` and `products[3]`, but the wipe list still holds them. At pin a399d626, `begin_animations` (`fmn-anim/src/frame.rs:195`) re-roots any animated mobject not on stage, and `fade_out` restores its begin-time paint. | Replace the two handles in `digits` and `products`, and drop `digit` and `product` from the list. |
| **F2** | Frames 734 (327.0), 749 (333.7); sheets 182 to 191. Lines 1041 to 1086, box 1052 | **The parity lines sit over the wrong word.** "Two flips: parity holds, label not 000, so report" is beside 0010110, the miscorrected word, which has three ones. The stack (3, 5, 6, 7, XOR 111) belongs to the two-flip word 0010111, which is no longer on screen. The parity box stays empty for the rest of the chapter, and the red and green column frames from the previous beat stay up. | At sentence 1, return to 0010111 (bit 7 back, ring back to its row, column frames out) and put the parity bit 0 in the box. Four ones plus 0 is even, so parity does hold, and the stack and "label not 000" now describe what is shown. |
| **F3** | Frame 185 (82.0). Lines 130 to 147 | **17_4 shows its conclusion first.** Red 7 and 49 by 81.5 (log). "132 + 7 = 139" and "Remainder 7: copying error detected" by 82.0, 1.4 s into the line. Voice reaches the weight at 85.247 and "not divisible" at 93.157, so the verdict leads by 11.2 s. Static to 102.5. | Symbol at sentence 0, new product and total at 1, remainder at 2. |
| **F4** | Frames 156 (70.5), 171 (78.0); frame 169 at 77.0 (log). Lines 88 to 116 | **17_3 runs behind the voice with a different number.** Voice says "The total is one hundred thirty-two" from 70.434 while the screen reads "Running total: 93" on the seventh column. 132 arrives at 77.0, 6.6 s late, and "passes" at 78.0 against 74.596. Five 0.35 s fades plus 0.15 s is 1.9 s a column, started at "Let's calculate one". | Start the walk at "gives weighted contributions" (about 63.0) at 0.7 s a column, ending about 70.0. "Passes" at sentence 3. |
| **F5** | Frame 244 (108.5). Lines 162 to 194 | **17_5 finishes the argument in the first quarter of the line.** Rule by 104 (log), both rows by 108 (log), "Prime modulus" by 108.5 against "Because eleven is prime" at 121.181, so 12.7 s early. Static to 130.3. | Rule at sentence 1, caption and rows at 3, condition at 4. |
| **F7** | Frames 779 (342.5), 818 (360.0). Lines 436 to 476 | **17_11 is complete 3.4 s into a 22 s recap, and three things the voice names aren't there.** All triangles by 342 and captions by 342.5, against "Cosets partition" at 351.676. No square for "our square and clock". Nothing for "a group" or "an isomorphism". "An operation on those pieces?" asks what the voice asserts. Static to 362.0. | One beat per sentence, a square thumbnail at sentence 0 (`d4g::thumb`), and five named lines: group, subgroup, isomorphism, cosets, normal subgroup. |

### Medium

| # | Where | Defect | Repair |
|---|---|---|---|
| **F6** | Frame 358 (160.0). Lines 254 to 310 | **17_7 is done 4.9 s into a 26.6 s line, then static to 180.8.** ΔS is up by 158.5 (log) against "changes the total by the product" at about 162.4. The weights 7 and 6 are never drawn, so in `7×6+6×4` the 6 is a digit in one term and a weight in the other with nothing to tell them apart. Sentences 2 to 4 get no picture. | Fixed `× 7` and `× 6` under the crossing digits. Formula at sentence 1, ΔS at the phrase, and one line each for sentences 2, 3 and 4. |
| **F8** | Frame 899 (392.5). Lines 554 to 563 | **17_13's third answer leads its sentence.** hH=H by 392.0 (log) and the closure line by 392.5, against sentence 2 at 397.185, so about 5 s early. This comes after the think time, so it isn't a quiz violation. | Answer 2 at sentence 1, answer 3 at sentence 2. |
| **F9** | Frames 564 (251.5), 573 (256.0). Lines 885 to 905 | **17_15 fills are slow.** Seven rows at the default of about 1 s run from 248.5 to 255.0 (log), and only four of eight rows are filled at 251.5. "128/16 = 8" starts at 255.5 against "By Lagrange" at 252.201, about 3.3 s late. | 0.4 s a row, finishing about 251.6. |
| **F10** | Frames 7 (2.0), 109 (50.0). Lines 65 to 86 | **17_1 and 17_2 are up before they are spoken, and the rule is never shown.** Note by 1.5 against "not a security system" at 16.949. Weights at 31.233 against "Multiply" at 40.097. Total by 32.0 against "Add" at 47.275. "Divisible by eleven" (48.447) has no picture at all. Static 2.0 to 31.2 and 33.5 to 57.6. | Digits at 17_1 sentence 1, note at 3. Weights at 17_2 sentence 1, total at 2, a new line "Valid only if the total is divisible by 11" at 3. |
| **F11** | Frame 244. Lines 173 to 186 | **The 17_5 rows are unlabelled.** Two rows of numbers under "Weight 7" with nothing saying what either row is. | `Δd` and `7Δd (mod 11)` row labels. |
| **F12** | Frame 411 (184.0). Lines 312 to 359 | **17_8 announces the failure 13.4 s early.** "Two errors can cancel" by 181.3 (log) and the full setup by 183.5 (log), while the voice is on swaps until 189.270 and identical symbols until 194.666 ("Here is a concrete failure"). | Hold the swap picture through sentence 0, setup at 1, title at 2. |
| **F13** | Frame 830 (363.7). Lines 480 to 490 | **17_12 prompts.** All three are up 1.7 s into the line, against spoken 364.415, 367.821, 372.528. Prompt 2 says "equal" where the voice and the answer say isomorphic. They are centred, so the numerals don't line up. No answer shows (see below). | Each at its sentence, "be isomorphic to", left-aligned at x = −3.6. |
| **F14** | Frame 608 (270.5). Lines 772 to 786 | **The XOR stack has no heading.** Rows like "2 → 010" appear with nothing saying they are the positions holding a 1, and the voice never says it either. | Heading "positions holding a 1". |
| **F15** | Frames 573, 608. Lines 879 to 891, 947 to 957 | **The grid's meaning is never on screen.** That rows are cosets is stated only in a source comment (lines 1 to 7). "Syndrome" is spoken and never written. | "each row: a coset of 16 words" at 17_15 sentence 1, "syndrome" over the label column at 17_16 sentence 1. |

### Low

| # | Where | Defect | Repair |
|---|---|---|---|
| **F16** | Frame 707 (316.5). Lines 1015 to 1037 | The red and green column frames are unnamed. | "sent" and "decoded" above them, faded with them at F2. |
| **F17** | Frames 411, 452, 490. Lines 322, 342, 354 | The wheel's green 0 nearly touches the `x 3` weight label for about 40 s and could read as `x 30`. | Wheel centre y −0.6, radius 1.4, marker radius 1.05. |
| **F18** | Frames 171, 185, 452 against 358. Lines 80, 111, 322, 399 | Letter x for times here, a real × in 17_7. `k.text` renders × (ch 11 frame 1133, `permutations.rs:1979`). | Use ×. |
| **F19** | Frame 749. Line 1080 | "built to keep" against spoken "designed to keep". | "designed". |

## Lint persistence

Both `layout.tsv` entries are real but tiny, and they aren't visibly clipped at their anchors.

| Line | Lint | Span | Over the ±3.85 safe area | Frame |
|---|---|---|---|---|
| "forgets the column" | 17_17 +5.1 s @302.0, 152 later looks | y −3.86 to −3.64 | 0.01 units (about 1 px) | 677: fully on screen |
| "The quotient keeps exactly what" | before 17_11 @333.7, 29 later looks | y −3.87 to −3.63 | 0.02 units (about 2 px) | 749: fully on screen |

The counts are large because the line stays up, not because anything gets worse. **L1 repair:** y −3.75 to −3.7 at lines 997 and 1083, giving −3.81 to −3.59 and −3.82 to −3.58, with 0.08 to 0.09 clear of the column frame bottoms at −3.5. Those spans are computed from the lint's own numbers. Root's lint run decides.

## Checked, no defect

- **Quiz answers stay hidden.** Frames 826 to 885 show only the prompts (log; 830 and 885 at full resolution). At 887 the prompts are still fading and at 888 only the new title is coming in. The think gap from 381.631 to 387.233 is 5.602 s on the scratch voice. The source holds for the line and then waits 5 s, so a real voice keeps that wait.
- **All displayed maths is right** (`checks/measure.py`, independent of the Rust): the products and 132, the 7d row, 66, 64 and −2, 187, syndromes 0, 5, 7, 0, 16 codewords and 8 cosets, the XOR example, and 6 generating {0, 6}. The four triangles are the cosets of {0, 4, 8}. Word 0110011 has parity 0, 0010111 has four ones and 0010110 has three.
- **Transitions** at 31.233, 57.633, 130.3, 153.633, 223.4, 299.0, 339.267, 362.2 and 387.4 are clean apart from F1 (log). In the source, F1 is the only re-root site in this chapter: the other wipe lists hold only live handles.
- **Dense right column** (frames 650, 707, 734): no overlaps. The "Two flips" line is tight above the frames but clear.
- **Hamming hooks** in 17_9, 17_16, 17_17 and 17_18 line up with the scratch sentence shares by construction.

## Seen, not patched

| # | Where | Note |
|---|---|---|
| N1 | Frame 677 | Empty key boxes stay from 299.0 to about 313 until they are reused. |
| N2 | Frame 490 | 17_14 has no picture change (log: 209.7 to 223.2). The voice is summarising, so this is acceptable. |
| N3 | Frames 297, 899 | Clocks have no outline, and markers cross the empty centre. |
| N4 | Frame 779 | Caption colours reuse the triangle colours, so the gold caption looks tied to the gold triangle. Only blue is meaningful. |
| N5 | Frame 899 | hH=H stops short of (gh)H = gH. The voice does too. |
| N6 | Frame 452 | "Together they add fifty-five" is never drawn. The picture shows 187 and remainder 0, which is still correct. |
| N7 | Frame 830 | Prompt 3's "keep it" is vague. |
| N8 | Frame 948 | 17_13's last sentence, "follow the operation and ask what it preserves", has no beat of its own. |
| N9 | Sheets 076 to 085 | 17_6 is static from 136.0 to 153.633 (log). |

## Decisions for root or the owner

1. **Spoken inconsistency, flagged only.** 17_12 says "adding a subgroup's elements to a coset representative" and 17_13 says "multiplying a subgroup by one of its own elements". Narration is frozen, so I changed nothing. If the wording changes, it is cheapest before chapter 17 takes are made.
2. **The square in F7.** The patch draws one because the voice names it. If you'd rather not add it, drop those three lines; the rest of the 17_11 edit doesn't depend on them.
3. **Scope of the patch.** F1 to F5 and F7 are the ones I'd insist on. F16 to F19 and L1 are optional. Dropping one means editing `make_proposed.py` and rebuilding, because some edits share a block. F16's labels are also faded in F2's edit, so remove both pieces together.

## Staged patch

| File | Purpose |
|---|---|
| `before-applications.rs` | Preserved baseline, byte copy of canonical 7cc3f0d0 |
| `make_proposed.py` | 36 exact-once edits, each labelled with its finding; refuses any other baseline |
| `proposed-applications.rs` | Result, 1387 lines |
| `proposed.patch` | Unified diff against the canonical path |
| `checks/patch_check.py`, `.out.txt` | Round trips, rustfmt, script and `k.say` unchanged, hook order and range, phrase presence, home-path scan |

**Status: uncompiled and unrendered. Cargo, Clippy and tests are root's.** What actually passed is text only: `git apply` forward and `-R` byte-exact in a scratch tree, `rustfmt --check`, `k.say` order unchanged, `script.tsv` untouched, 49 sentence and phrase waits in range and in order, and both waited phrases present in their sentences. The double-fade scan is a one-line regex and doesn't see wipe lists, so I also walked those by hand: `digits[3]`, `products[3]`, `total`, `shown[i]`, `delta`, `why`, the column frames and names, `rows_are`, `syndrome_name` and the old `stack` each leave the screen once.

I confirmed these against source, not by compiling: `AnimBuilder` has both `set_anim_args` and `set_fill` at the pin (`fmn-mobject/src/animate.rs`), `until_phrase` is `pub(crate)` and already imported this way by `arithmetic.rs`, `\pmod{..}` is used elsewhere in the corpus, `d4g::E` exists, and `k.text_left` exists. The patch adds one test, `every_waited_phrase_is_in_its_sentence`.

**No spoken change and no new takes.** The patch only moves and adds picture beats, so nothing here adds to the Oliver synthesis list.

### Estimated timing after the patch

Computed from the source's fades and the scratch sentence table. Nothing below has been rendered.

| Beat | Rendered now | Patched, estimated |
|---|---|---|
| 17_1 digits / note | 1.0 / 1.5 | 5.645 / 16.949 |
| 17_2 weights / total / rule | 31.233 / 32.0 / none | 40.097 / 47.275 / 48.447 |
| 17_3 walk / passes | 57.633 to 77.0 / 78.0 | about 63.0 to 70.0 / 74.596 |
| 17_4 new total / remainder | 82.0 / 82.0 | 85.247 / 93.157 |
| 17_5 rule / rows / condition | 104 / 104.5 to 108 / 108.5 | 106.308 / 116.585 to 120.4 / 121.181 |
| 17_7 first sum / ΔS / reasons | 155.0 / 158.5 / none | 156.478 / about 162.4 / 168.874, 172.600, 176.868 |
| 17_8 setup / title | 181.0 / 181.3 | 189.270 / 194.666 |
| 17_11 blue triangle / other triangles | 341 / 342 | 343.351 / 351.676 |
| 17_12 prompts | 362.5 to 363.7 | 364.415, 367.821, 372.528 |
| 17_13 third answer | 392.0 | 397.185 |
| 17_15 fills / 128/16 | 248.5 to 255.0 / 255.5 | 248.8 to 251.6 / 252.201 |

Every patched cue's estimated last beat finishes before its line ends, so on these estimates the patch doesn't push a later `k.say` back. Waits only hold when the picture is ahead, so with Oliver's real sentence times a beat runs late rather than jumping early.

### What root should look at after a scratch render

- **F1:** frames at the 17_5 cue start show no 6 or 42 over the red digits.
- **F2:** word 0010111 with parity 0, ring on the bit 7 row, column frames gone.
- **F4:** the 0.1 s fades in the column walk may look abrupt. Raise `WALK_FADE` if so, but 10 columns must still fit about 7.4 s.
- **New text I could only estimate:** the 17_5 row labels against the first column and the left edge, the 17_11 caption widths (at most about 5 units) and the square at (2.5, −2.5), the 17_7 weights under the swap lane, the stack heading under the title, and the prompts at x = −3.6.
- **Layout lint:** L1 should clear both entries, and the new lines shouldn't add any.
- **Still open after that:** Oliver takes for all 17 cues, then narrated sync, listening and independent final AV review. None of them is done or claimed here.
