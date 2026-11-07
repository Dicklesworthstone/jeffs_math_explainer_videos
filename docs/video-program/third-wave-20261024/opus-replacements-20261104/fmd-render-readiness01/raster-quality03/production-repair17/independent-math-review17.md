# Independent mathematical and source review 17

RubyRaven, for root, under `mx-video-program-program-87do.39`. This covers what was in `production-repair17` when the review started. It is source and proof reading only. I built, rendered and played nothing, and looked at no pictures. Nothing here accepts audio, sync or the final film. Two source checks were run as Python copies of the Rust test logic, not with cargo.

## Release-blocking

**B0. 09_1 wording (`19eb8550…`): two sentences put a complex number in an order.** Fix both before synthesis.
- "Its integral is at most its circumference, …" → "The magnitude of its integral is at most its circumference, …" (root's edit, agreed).
- "The difference comes only from how much the quotient wobbles there, and it is at most that wobble times their length." Here "it" is the arc-minus-chord difference, a complex number. Say "and its size is at most that wobble times their combined length". "combined" is optional; the bound uses arc plus chord.

I found nothing else in 09_1 that blocks. "Near a its size stays below some bound" already says "size". B3 is a dependency on 08_5, not a wording fault.

**B0a. 08_5 (`b1b6334033878f33291b96af20360a0e6987e7b0dec1505ed262f3a736576b30`): not approved.** It has one false step and orders complex numbers in four more sentences.
- **False step** (root caught it): "So after n cuts, the original integral is at most four to the n times the small square's integral. And that is at most epsilon, times the original diagonal and perimeter, divided by four to the n." Here "that" is four to the n times the small integral, and its bound is epsilon·D·P, not epsilon·D·P/4^n. Replace with: "So after n cuts, the size of the original integral is at most four to the n times the size of the small square's integral. And the small square's integral has size at most epsilon, times the original diagonal and perimeter, divided by four to the n." The next sentence, "The fours cancel", then follows.
- **Size wording**, in the same style as 09_1's "its size":
  - "the remainder is at most epsilon times the distance to c" → "the size of the remainder is at most epsilon times the distance to c"
  - "So the integral around that square is at most epsilon, …" → "So the size of the integral around that square is at most epsilon, …"
  - "our rule kept at least a quarter of the integral at every cut" → "our rule kept at least a quarter of the integral's size at every cut"
  - "an integral at least a quarter as large" and "whose integral is at least a quarter as large as its parent's" already read as size. To match the rest, say "an integral at least a quarter of that size" and "whose integral has at least a quarter of its parent's size".

Everything else in 08_5 is correct. It cites the general theorem and proves "the pieces we need". The cancellation, nested selection, constant and linear terms, and the diagonal and perimeter bound are all right. The triangle case uses the midpoints, has all four children counterclockwise, and uses the longest side for the diagonal. That last part settles B3 and B5 in the text; the pictures are still unchecked. Optional: "Now suppose that integral were not zero" isn't used by the direct bound and could go. "So around every triangle" could say "around every such triangle". The cue is long, which affects timing only.

**B1. `counting.rs`: a test fails.** `the_reach_labels_take_turns_over_the_window` looks for the exact line `fade_out_all(stage, &[covered, covers, refused, band], 0.3)?;`. The edit added `say_row` to that line. `body` runs from `fn coarse_loop(` to the end of the file, so the search now finds the test's own copy of that text at line 1645. That puts `gone` after `name` (line 954), and the order check fails. I copied the test into Python: it passes on `before-counting.rs` and fails on `counting.rs`. `build17.py` runs `cargo test` before Clippy, so this stops the build.
Fix: put `say_row` into the test's string as well: `"fade_out_all(stage, &[covered, covers, refused, band, say_row], 0.3)?;"`.

**B2. `counting.rs`: the removed caption comes back at the end of 11_8.** `say_row` is still pushed into `left` at line 834, and `coarse_loop` returns `left`. At line 606 the chapter ends with `fade_out_all(stage, &last, 0.5)`, which fades it out a second time. The engine's `fade_out` (`crates/fmn-anim/src/fading.rs`) is a remover with `final_alpha_value = 0`. It detaches the mob and restores its starting opacity, so a second fade re-roots it at full opacity. The "8 samples say 0" caption that this repair removes would then show again for the last 0.5 s. The same mechanism caused the square_packing 05_7 marker flash.
Fix: delete `left.push(say_row);` at line 834. Nothing uses `say_row` after line 937, and the function returns only at its end. This is from the source; I have not seen a render.

**B1 and B2 are repaired in `counting.rs` `ecff48f7ca1aea98b3ac8eac65ffe8f4a74d5366fd30ce25546def3b627339d1`** (root). The push is gone, and the test now matches the new line and asserts that the push does not come back. Mirrored in Python, the order test passes: lines 924 < 937 < 954 < 964 < 982 < 996. This is a source mirror, not cargo.

**B3. 09_1 needs chapter 08's triangle case.** The new 09_1 says "We saw that the shrinking argument works for triangles". It is true only if the delivered 08_5 says and shows the triangle subdivision. When I last looked, `loop_sums.rs` was unchanged (`f7a46143…`) and there was no 08_5 text yet.

**B4. Wording in the draft 08 proof, if it is reused.** In `complex-proof-repair-narration-draft01.md`, "Their product shrinks by a quarter" means it loses 25%. The proof needs the product to fall to a quarter, or "the cancellation" does not follow. Say "drops to a quarter" or "is divided by four". Also, "At least one child must carry a quarter of its magnitude" should be "at least a quarter".

**B5. 08_5 must keep the theorem's scope straight.** The current 08_5 states the theorem for any loop. Later chapters rely on that statement. The replacement may prove squares and triangles and then say the same cancellation extends to other loops. It must not claim that the square argument alone proves the theorem for every loop. I'll check this when 08_5 arrives.

## Optional

- **`continuing.rs` DENSE[1].** "all orders together are dense" makes the orders dense, but the roots are. Suggested: "all these roots together are dense; drawn: through order 64" (59 characters, the same length as now).
- **`counting.rs` CODE[5].** "zeros − poles inside" also needs a simple counterclockwise loop. The film's loop is the unit circle, so the line is true as shown.
- **`counting.rs` new test.** It proves the counterexample exists (mean reach 0.785 < 1, last-arc reach about 50.3 > 1, sampled sum about 0). It does not run the chapter's own model on that loop.
- **`gallery.rs`.** The separate "series converges: Re s > 1" text now repeats the qualifier in the title. Harmless.
- **09_1 length.** 361 words and 1,984 characters in one cue. The longest current cue is 14_3 at 98 words. Measure chapter 09's length and raw size from real audio. Splitting it into two or three cues would make takes and sync easier. This is not a maths issue.
- **`real_integral.rs` (unchanged).** The displayed "2.094" is below the exact bound 2.0944 but well above the actual arc integral (0.927 at R = 2), so the inequality on screen is true. You could add "≈" or round up instead.
- **`two_planes.rs` / `gallery.rs`.** I did not measure the 23 pt and 24 pt fit or legibility.

## What passes

**Winding proof** (`complex-winding-repair-handoff01.md`): correct.
- Along each subarc, the fundamental theorem keeps the output inside the closed disk about a_k of radius L_k·ell_k < |a_k|.
- That disk lies in the open half-plane Re(z·conj(a_k)) > 0. So the chord homotopy avoids 0, and each increment lies within (−π/2, π/2), tighter than the stated (−π, π).
- The handoff states the conditions for reading the count as zeros minus poles correctly.
- For z^8 on the unit circle: 8 steps have reach 2π ≥ 1 and are refused; 64 steps have reach π/4 < 1 and certify 8. The smallest equal-step count that passes is 51, since n > 16π.

**New CODE panel:** the test is sufficient. Each subarc has ell_k ≤ step, so L·ell_k ≤ L·step < |a|. Line lengths are 59, 36, 28, 64, 46 and 56, within the test's 64. The 11_8 narration already asks for steps "so small that the output cannot move as far as its distance from zero", so it needs no new audio.

**`two_planes.rs`:** "arg z² ≡ 2 arg z (mod 2π), z ≠ 0" is correct. The test strings at lines 1055–1056 and the `RULE_ANGLE_TEX` labels still match. The engine knows `\mathrm`, `\ne` and `\;` from its corpus and tests, but I did not check them rendered.

**`gallery.rs`:** the series is now qualified by Re s > 1 for as long as the title is on screen (created at line 744, removed at line 870).

**`continuing.rs`:** true as written (wording optional above).

**Draft 08 proof:** correct with B4's wording.
- The nested squares are chosen before ε.
- |I_0| ≤ 4^n·|I_n|.
- Once Q_n ⊂ D(c, δ), |I_n| ≤ ε·D_n·P_n = ε·D_0·P_0/4^n.
- It does not assume f′ is continuous.

**Bridge (`complex-cauchy-bridge-draft02.md`):** correct and not circular.
- The central midpoint triangle is a 180° turn, so it keeps counterclockwise orientation.
- Inscribed polygons keep the ring at least ρ·cos(π/n) from a.
- The arc-minus-chord error is at most the oscillation times (arc + chord), which tends to zero by uniform continuity on a fixed compact ring.
- ρ stays fixed until the circle equality is proved, then ρ → 0.

**`09_1-script.txt`:** correct and not circular, given B0 and B3.
- It never calls the quotient holomorphic at a; boundedness comes from the derivative limit.
- The triangulated ring keeps the hole empty.
- The chord-and-arc step, "every small radius", then "only now do we shrink", and "the big circle's integral never changed" are each stated.
- It uses no Taylor series, residues or removable-singularity theorem.

## Sources reviewed (sha256)

| File | sha256 |
|---|---|
| `before-counting.rs` → `counting.rs` | `1177c885b6815842bc0ac6e6ac71b01808f00a73137002c05d04e1874decb984` → `16161606582030b0380a916e1acbbb74fecc0d7ac1cfa36e0be1a821546756c7` |
| `before-two_planes.rs` → `two_planes.rs` | `98cd46c5cc825095600ae431ae22c142a65c7b0cb8d2507d4c4ba0644ca3c432` → `0f22d63febc18e05dc4123bedf84eeadf967938bcbc4260cfb0d9e8925ff4cb9` |
| `before-gallery.rs` → `gallery.rs` | `da7b806fee85584e111c8a66b95e0f9f8c87bbb42501d79a1129d6b032663290` → `7d1cf20fd25c8d9233f3c14426b0196fd4fd6673e4b25becba47b50f2635b282` |
| `before-continuing.rs` → `continuing.rs` | `c24f264b98377aa323a714588d3e1d8b46201693592886d62138ef2a0e4ec96b` → `4e5bd3555a7cb00b8dee58d079f69d205c18777db304b56b14f1ca9654bb6a27` |
| `loop_sums.rs` (unchanged) | `f7a46143d709797ff03e748b4b8efad68d9379fea31b8ce0cd6de0d2f25de6d8` |
| `boundary.rs` (unchanged) | `6142165b7e9a49e99bb80216ed83117f6bf752a3beda812af29fa5081bc347e9` |
| `real_integral.rs` (unchanged) | `83eadb5372812fc540738bbfe9c980ffc1cc35e959be9db9f75906ba425299ae` |
| `09_1-script.txt` | `19eb85504d93210d4f8758a4f58d8a7656604cb7d4b80303f5d2032e48ade055` |
| `complex-winding-repair-handoff01.md` | `da3d818997e055339d958df1eb387d3786cf2391e88367c1292e20a1140d1785` |
| `complex-proof-repair-narration-draft01.md` | `a8b3f7f3517590084f53fabb887f356021fb4942b71caed03e3400f06ca48878` |
| `complex-cauchy-bridge-draft02.md` | `98d2afae9c0b57c6f5c87e59b99ebd2b28c751c2c56d9838ae7d730a80d0b6b7` |

The script cues compared are 08_4, 08_5, 09_1, 09_2 and 11_8, from `complex-cauchy-source-audit01-script.tsv.txt` (`946a9c07…`).

## Not reviewed yet
- Pane 0's `loop_sums.rs`. Its 08_5 text is covered by the verdict below.
- Pane 1's `boundary.rs` changes, including the wait markers that 09_1 must rebind. The file has changed to 49,518 bytes, not yet read.
- Any build, test run, render, picture, take or sync.

## Narration verdict (exact hashes)

**APPROVED for TTS, mathematics of the spoken text only:**
- `08_5-script.txt` `15e727f0bf7696a24123a6c7c8d5ac1f56a31ebccb08c0e561e164d4ad7ee8a7`
- `09_1-script.txt` `a24b71272c5c2c5560227625ce999c1cf00e5e5208a436d1021efcea1a61a491`

Reasons:
- **B0a is fixed in 08_5.** The 4^n step is now correct: |I_0| ≤ 4^n·|I_n| and |I_n| ≤ ε·D_0·P_0/4^n, so |I_0| ≤ ε·D_0·P_0. Every "at most" or "at least" now applies to a size or a length (nine sentences).
- **B0 is fixed in 09_1.** Compared sentence by sentence with `09_1-script-initial01.txt` (`19eb8550…`), only the two flagged sentences changed: "its size is at most that wobble times their combined length" and "The magnitude of its integral is at most …".
- **B3 and B5 are settled in the text.** 08_5 proves the triangle case, which 09_1's "We saw … triangles" needs. It cites the general theorem without claiming the square alone proves it.
- **Nothing in either cue is false or circular.**

**Conditions:**
- Approve the two hashes as a pair. A 09_1 used without this 08_5 loses its triangle step.
- Any further edit to either file needs a new verdict.
- This verdict does not cover the pictures, the wait markers, pronunciation, takes, listening or sync.
- 08_5 is 478 words and 09_1 is 361; chapter length must come from real audio.

**Optional, not blocking:**
- "Now suppose that integral were not zero" is never used.
- "around every triangle" could be "around every such triangle".
- "epsilon times the original diagonal and perimeter" could say "times the original diagonal, times the original perimeter", matching the earlier sentence.
