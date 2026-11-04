# Algebra 17 patch: cross-review

BronzeJaguar, for BlueSalmon, Mail 1857, Bead mx-video-program-program-87do.64 (root alone edits Beads).

Reviewed: RubyRaven's `algebra-picture-review-17_applications/` (`proposed.patch` 10b96b98, `proposed-applications.rs` 686c2273, baseline 7cc3f0d0).

**Verdict: the patch is sound and I recommend integrating it with C1.** All twenty findings (F1 to F19, L1) are real and the patch corrects each of them. Its arithmetic, hide and re-show lifecycle and cue order hold, and its new waits delay no line. It does introduce two false on-screen statements, and four of its beats still run seconds ahead of their words.

- **C1, recommended before integration:** two on-screen corrections, one literal replaced by the computed count, and one set of test guards. It fixes a heading that becomes false for 15 s in total and a green "passes" left beside the red 7 for 3.6 s.
- **C2, recommended, root's call:** four beats moved onto their words, plus the test table for the new phrases.

Line numbers below are in `proposed-applications.rs`. Nothing here has been compiled or rendered, and no spoken word changes in either layer.

## Author overlap

I did not write `applications.rs` or the patch. RubyRaven wrote the review and patch, and Mail 880 and the shared memory point to LavenderPike for the chapter.

This is still not fully independent. I share a memory directory with those lanes, read the algebra rebuild note and RubyRaven's REVIEW before checking, am the same kind of agent, and used the same `until_phrase` pattern and method in my own 13_arithmetic patch. C1 and C2 are my own edits, and no one else has reviewed them.

## Evidence and its limits

| Evidence | What it covers | Limit |
|---|---|---|
| `checks/clock_mirror.py` | Hand translation of the chapter's waits, plays and fades into Python, for baseline, patch, C1 and C1+C2 | A model, not the Rust. The positive control is the baseline only: its 17 line starts match the render's cue table with a largest difference of 0.000 s, and its lifecycle check finds F1 and nothing else |
| `checks/model_arithmetic.py` | Every displayed number, recomputed from the parity-check equations and the ISBN rule | Does not read the Rust values |
| `checks/new_text_extents.py` | New text against the safe area, from CMR10 advance widths | Calibrated at 0.997 to 0.999 on five strings from the render. Ignores kerning. Needs `python3 -B` (PIL) |
| 14 full-size frames (`coverage/frames-viewed.tsv`) | Targeted, to confirm findings | Scratch pixels of the **baseline**: 1280x720, 30 fps, scratch voice. I did not view all 949, and no frame of the patched chapter exists |
| `checks/run_desk_checks.py`, `rustfmt --check` | The staged text-only checkers and format | Not a compile. Patch, C1 and C1+C2 report the same problem counts as the baseline tree (arity 7, scope 11, shape 0, unused 2, desk 49); the only changed lines are totals and line numbers |
| `checks/oliver_ratio.py` | Oliver against scratch line length on 23 cues with take01, chapters 10 and 13: min 0.866, p10 0.988, median 1.074, p90 1.196, max 1.247 | **Chapter 17 has no Oliver takes.** The scales below are borrowed from other chapters |

Voice times inside a line are estimates by share of characters. No listening, narrated sync or final AV review was done, and I claim none of them.

## The questions in Mail 1857

### Does the patch correct real defects?

Yes, all twenty. I found no finding to dispute, so the frames below confirm findings rather than settle arguments.

| # | Evidence I used | Corrected? | Note |
|---|---|---|---|
| F1 ghost | Frames 229, 231; engine `begin_animations` at the pin; mirror finds exactly `digit3` and `product3` at 102.733 | Yes | Lifecycle now clean. The way it splits the wipe leaves the old verdict up: **C1b** |
| F2 parity over wrong word | Frames 734, 749 | Yes | The arithmetic is right, and it had to return the word. Timing leads its words: **C2a** |
| F3 conclusion first | Frame 185 | Yes | Same side effect as F1: **C1b** |
| F4 walk behind voice | Frame 156 | Yes | Walk leads numbers by 1.72 s, falling to 0.53 s. See "Not patched" |
| F5 argument early | Frame 244 | Yes | Caption states the conclusion 4.58 s before its reason: **C2d** |
| F6 swap done early | Frame 358 | Yes | Weights fit between the lane and the formula. Crossing leads its words by 3.32 s: **C2c** |
| F7 recap early | Frame 779 | Yes | Square fits: thumb y −3.12 to −1.88, caption bottom about −1.42. Widest caption left edge −0.02, clock right edge about −0.88 |
| F8 third answer early | Frame 899 | Yes for sentence 2 | The sentence 1 wait does nothing. See "Not patched" |
| F9 slow fills | Mirror (17_15 sentence 2 late by 3.25 s at baseline, 0 now); frames not opened | Yes | |
| F10 up before spoken | Mirror (holds 29.57 s and 25.70 s at baseline); frames not opened | Yes | A 22.78 s hold remains and the review's table does not state it. Optional O1 |
| F11 unlabelled rows | Frame 244 | Yes | Label x −6.74 to −5.36, inside the safe area and clear of the first column at about −5.05 |
| F12 announced early | Frame 411 | Yes | Setup still leads "Here is a concrete failure" by 5.38 s, but it is neutral. See "Not patched" |
| F13 prompts | Frame 830 | Yes | All fit; widest right edge 3.70 |
| F14 no heading | Frame 608 | **Not as worded** | The heading is true only until the shown word changes: **C1a** |
| F15 grid meaning | Frame 608 | Yes | Hard-coded 16: **C1c**. The name leads its definition by 6.42 s: **C2b** |
| F16 unnamed frames | Frame 707 | Yes | Must travel with F2, or the names at y 0.03 collide with the two-flips line at y 0.05 |
| F17 `x 30` risk | Frame 411 | Yes | Read in source, not measured. Root's eyes on the render |
| F18 x against × | Frames 358, 411 | Yes | `×` already renders through `k.text` (`main.rs:570`, `permutations.rs`). The probe captures chapter strings, so no `main.rs` edit is needed |
| F19 built/designed | Frame 749 | Yes | One word. Width 6.47, fits |
| L1 lint | Frame 749 | Yes, by computed spans | Root's lint run decides |

### Is the model arithmetic right?

Yes. Everything passes when recomputed independently (`checks/outputs/model_arithmetic.out`, "problems: none"):

- **ISBN:** products, running totals, 132 = 12 × 11, 139, remainder 7, the 7Δd permutation, 66, 64 and −2, 130, 142, 187, +45 = 4 × 11 + 1, and the marker's 11 steps making a full circle.
- **Hamming:** 16 codewords, 8 cosets, labels 101 and 111. 0010111 has four ones, so parity bit 0 holds; 0010110 has three, so it breaks.
- **Extended code:** 128 corrected and 448 reported.
- **Grid and groups:** sent column 6 at x −1.48, decoded column 2 at x −3.96, two-flip word at (7, 2), cosets of {0, 4, 8}, and 6 generating {0, 6}.

### Does hide and re-show hold?

Yes. The mirror follows every handle through both scenes.

- **Patch, C1 and C1+C2:** 0 lifecycle findings at all five voice scales. Nothing fades out while off stage, nothing is re-shown after a hide, and nothing is left on stage after the Hamming wipe.
- **The patch's replaced handles** leave `digits` and `products` before the 17_5 wipe, which is what removes the re-root in F1.

### Is the cue order right?

Yes. Film order is unchanged: 17_1 to 17_8, 17_14, 17_9, 17_15 to 17_18, 17_11, 17_12, 17_13. `k.say` source order is the same in baseline, patch and both candidates. Each wait in the chain names the cue then being spoken, and all five phrases in C1+C2 occur in the sentence they name (`package.out`). `script.tsv` is unchanged, 83619d48676b32d5.

`patch_check`'s "k.say order" compares source order, not call order, because `hamming_film` holds 17_15 to 17_18. The mirror confirms the call order separately.

### Can the new waits delay actions?

Not at any measured voice length. `Narrator::say` waits out the running picture, so a picture that overruns delays the next line silently; this is the risk I checked. At 60 fps the chapter is 413.517 s long in both baseline and patch, and no line start moves.

| Voice scale | Lines delayed, patch | Beats late, patch (cue, sentence, seconds) |
|---|---|---|
| 0.75 (below anything measured) | Only the authored 5 s before 17_13 | 17_5 s4 0.41; 17_8 s4 0.20, s5 0.89; 17_15 s2 0.34; 17_17 s4 0.23; 17_13 s1 1.72 |
| 0.85 | Same | 17_8 s5 0.30; 17_13 s1 1.44 |
| 1.00 | Same | 17_13 s1 1.02 |
| 1.07 (measured median) | Same | 17_13 s1 0.83 |
| 1.25 | Same | 17_13 s1 0.33 |

- **The patch removes delays.** At 0.75 the baseline delays 17_4 by 2.01 s, and at every scale it runs 17_15 sentence 2 late by 2.3 to 4.2 s.
- **C1 and C1+C2 give the same table,** and neither moves a line start.
- **Quiz:** the 5 s before 17_13 is the authored think gap, so it counts as a delay only in the mirror's bookkeeping. The prompts are fully up at 372.900 and 17_13 starts at 387.233 at scale 1.
- **Longest holds:** down from 29.57 s to 22.78 s (17_1 into 17_2). Others at 18.92 s (17_6, N9), 18.52 s (17_8 into 17_14, N2) and 14.35 s (the 17_12 think time).

## Corrections with exact lines

Against `proposed-applications.rs` 686c2273. Apply canonical 7cc3f0d0, then `proposed.patch`, then `patches/C1-…`, then `patches/C2-…`.

### C1: false statements and guards

**C1a, line 885. The heading goes false.** "positions holding a 1" stays up over stack 1 for 3.83 s after bit 5 is repaired (294.983 to 298.817), and over stack 2 for 11.13 s after the miscorrection (314.517 to 325.650). Over those spans the word on screen no longer has a 1 at the positions listed.

```rust
    // Names the received word: the word on screen is repaired or miscorrected under it.
    let heading = k.text(stage, "1s in the received word", 22., Palette::dim(), v(3.2, 2.8))?;
```

Width 2.23, right edge 4.32. The key starts at 4.75.

**C1b, lines 156 to 159 and 163. A stale verdict.** F3 separated the green `132 = 12 × 11: passes` from the digit change, so it sits beside the red 7 for 3.63 s (81.600 to 85.233). The baseline removed the two together.

```rust
        // Sentence 0 changes the symbol and retires the verdict, which no longer holds. The
        // replaced handles leave `digits` and `products`, so the wipe below cannot re-root
        // them: fading a removed mobject out shows it at full opacity first.
        fade(stage, &[digits[3], valid], false)?;
```

Line 163 becomes `fade(stage, &[products[3], total], false)?;`.

**C1c, line 1000. A literal where the film computes.** The rest of `hamming_film` uses `codes.len()`.

```rust
        &format!("each row: a coset of {} words", codes.len()),
```

**C1d, after line 1352, in `narrated_double_error_is_miscorrected_to_another_codeword`.** F2's reasoning has no test; the existing test only pins the word.

```rust
        // 17_18 shows the parity bit beside the received word. Parity holds there and would
        // break beside the miscorrected word, so the word has to return first.
        assert_eq!(SENT.count_ones() % 2, 0);
        assert_eq!(received.count_ones() % 2, 0);
        assert_eq!(decoded.count_ones() % 2, 1);
        // The ring's return runs straight along the decoded word's column.
        assert_eq!(grid_place(received), (7, grid_place(decoded).1));
```

### C2: beats onto their words

Times are at scale 1, by share of characters. The full text is `patches/C2-voice-sync.patch`.

| Edit | Lines | Change | Picture leads its words, patch → C2 |
|---|---|---|---|
| Phrases | 18–19 | Replace `SWAP_CHANGE_PHRASE` with `SWAP_PHRASE` "exchanging the values", `SYNDROME_PHRASE` "called the syndrome", `ONE_FLIP_PHRASE` "correct one flipped bit", `TWO_FLIPS_PHRASE` "while reporting two" | |
| **C2a** 17_18 | 1191–1228 | Reorder, below | Word returns: 5.57 s ahead of "instead of miscorrecting" → on "while reporting two". Two-flips line: 2.18 s ahead of "while reporting two" → starts 0.22 s after "instead of miscorrecting". One-flip line: 1.04 s → 0.00 s |
| **C2b** 17_16 | 1086–1089 | Move `syndrome_name` after the stack, behind `until_phrase(stage, k, "17_16", 1, SYNDROME_PHRASE)?;` | 6.42 s → 0.00 s |
| **C2c** 17_7 | After 323; delete 364 | Add `until_phrase(stage, &k, "17_7", 1, SWAP_PHRASE)?;` before the crossing, and drop the later wait | Crossing: 3.32 s → 0.00 s. ΔS then starts 0.24 s after "by the product" with no wait of its own |
| **C2d** 17_5 | 219, 242 | `fade(stage, &[change_label, product_label], true)?;` and `fade(stage, &[caption, condition], true)?;` | "nonzero changes give nonzero remainders": 4.58 s → 0.00 s, now with its reason |
| Test | 1271 | Table of 5 waited phrases, array size 5 | |

C2a's order, with the text builders unchanged:

1. `k.until_sentence(stage, "17_18", 1)?;`, then `parity` and `parity_name` fade in. The box leads "parity check" by 1.11 s, because that is where the sentence opens.
2. `until_phrase(… ONE_FLIP_PHRASE)?;`, then `one_flip`.
3. `until_phrase(… TWO_FLIPS_PHRASE)?;`, then `word.flip(…)`, the ring's glide, and the column frames and names fade out.
4. `parity_bit` and `two_flips` fade in together.
5. `k.until_sentence(stage, "17_18", 2)?;`

The order is not free. **The parity bit must not appear before the word returns,** because 0010110 + 0 is odd and would show a false statement. **The names at y 0.03 must leave before `two_flips` at y 0.05.** The last beat ends at 331.783 against sentence 2 at 333.073, a margin of 1.29 s, and no 17_18 beat runs late at any tested scale.

If root drops part of C2: every entry in `checks/build_corrections.py` is labelled. Keep each phrase constant with its call and its row in the test table, because an unused constant would fail Clippy. Dropping C2c means keeping `SWAP_CHANGE_PHRASE` and line 364.

## Not patched

- **17_8 setup** leads "Here is a concrete failure" by 5.38 s, with 3.3 s of neutral setup. It makes no false claim. A wait on "so there is no changed number" would raise the static hold to about 14.6 s, so I would accept it as it is.
- **17_3 walk** leads the numbers by 1.72 s, falling to 0.53 s, with the total up 0.83 s before sentence 2. At scale 1.07 that is 1.84 s, 1.01 s and 1.35 s; at scale 0.85 the total runs about 0.4 s late. Acceptable, but re-check on Oliver. The 0.1 s total out then in may blink, and a same-play cross-fade is the fix if it does.
- **17_13 answer 2** misses sentence 1 by 1.02 s at scale 1 (0.83 s at 1.07). The setup takes 3.8 s against a 2.78 s sentence, and it starts at the same time as the baseline. The patch's sentence 1 wait does nothing here. Low.

**O1, optional, in neither candidate and not on the mirror.** "X for ten" is spoken and never shown, and the longest hold left (22.78 s, 17.300 to 40.083) runs across it. After line 89:

```rust
        let symbols = k.text(
            stage,
            "Values 0 to 9; X = 10 only in the last place",
            25.,
            Palette::dim(),
            v(0., -1.5),
        )?;
        fade(stage, &[symbols], true)?;
```

and after line 96, `fade(stage, &[symbols], false)?;`. Width about 4.79. This pushes the total back 0.35 s; by hand it is still up about 0.48 s before sentence 3 at scale 1 and about 0.29 s at scale 0.85.

## Decisions for root

1. **Integrate with C1?** I recommend yes. Without it, F14's heading and F3's verdict ship false.
2. **C2?** I recommend yes. It is timing only, and it changes neither what is shown nor when any line starts.
3. **"Adding" against "multiplying" (RubyRaven's decision 1).** 17_12 says "adding a subgroup's elements to a coset representative", while 17_13 says "multiplying a subgroup by one of its own elements" with `hH=H` on screen. The mismatch is real, and fixing it is cheapest before chapter 17 takes exist. One option for 17_12 sentence 3: "Finally, explain why multiplying a coset representative by a subgroup element does not change that coset." I changed no text.
4. **Square in F7 (their decision 2).** It fits, and the voice names it. I would keep it.
5. **O1:** root's or the owner's taste.

## Small corrections to RubyRaven's notes

None of these changes a finding.

- `viewed-frames.tsv`, frame 156: "93 fading in" is fading out. The mirror and the frame agree.
- Estimated-timing table: "17_4 new total 85.247" starts at about 85.93, after the product.
- A fade written as 0.35 s is 0.3667 s at the preview's 30 fps, so a baseline column there is 2.0 s, not 1.9 s.

## Files

| Path | What it is |
|---|---|
| `base/proposed-applications.rs` | RubyRaven's proposed source, unchanged (686c2273) |
| `candidate-C1/applications.rs` | Patch + C1, 8f3358e6c75a32ae |
| `candidate-C1C2/applications.rs` | Patch + C1 + C2, 9d3ed41d35cc8879 |
| `patches/` | C1 on the patch, C2 on C1 |
| `checks/`, `checks/outputs/` | Scripts and their outputs. **Copy `checks/` to scratch before re-running:** several scripts write beside themselves. Run them from the repo root with `python3 -I -B`, except `new_text_extents.py`, which needs `python3 -B` |
| `coverage/frames-viewed.tsv` | The 14 frames with hashes and what each confirmed |
| `SHA256SUMS` | Every other file, relative to this folder |

In a scratch copy of canonical, `git apply --whitespace=nowarn` of the three patches reproduces each candidate byte for byte, and reversing them returns canonical (`checks/outputs/package.out`). If canonical moves, use `git merge-file` rather than `git apply`.

## After a scratch render

Everything RubyRaven lists under "What root should look at" still applies, except that with C2 the F2 picture arrives on "while reporting two". In addition:

- **C1b:** at 17_4 sentence 0, the green "passes" goes as the 6 goes.
- **C1a:** "1s in the received word" clears the key at 4.75 and sits clear of the title.
- **C2a:** at 17_18, the parity bit appears only beside 0010111, the names are gone before the two-flips line, and sentence 2's line does not arrive on top of it.
- **C2b, C2c:** the stack stands alone until "called the syndrome", and the digits hold still until "exchanging the values".

## Not done

Compile, tests, Clippy, render, layout lint, take generation, listening, narrated sync and final AV review are all open, and all are root's. Every timing above must be re-read against real Oliver takes for the 17 cues. Beads, commits and canonical files were not touched.
