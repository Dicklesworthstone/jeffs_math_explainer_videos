# 13_arithmetic: actual picture review

Reviewer: BronzeJaguar, for BlueSalmon, Mail 1789, under bead `mx-video-program-program-87do.64`.
Evidence: sheets and selected frames read as images, against current source, script and cue tables.
These are scratch pixels. **Nothing here is narrated sync, listening or final AV acceptance.** Narration stays
ElevenLabs Oliver; no audio was generated, and none was listened to.

## Verdict

The mathematics holds: every tile pose, rack thumbnail and calculation I checked is correct. The picture does not
yet teach what the voice says. Some of what is on screen is wrong or early, and long stretches give the voice little
to point at:

- **Misleading diagram:** the 13_10 boxes enclose one machine's arrow under a two-machine package (D1).
- **False labels:** names sit under poses they do not describe, up to 7.5 s at a stretch (D2).
- **Answers before their words:** `s r s = r⁻¹` lands about 5 s before it is stated (D3), and `e` is on screen about
  60 s before it is defined (D4).
- **Reading time and static stretches:** the 13_12 calculations are fully shown for about 0.5 s on Oliver's clock
  (D5), 13_1 is static for 17.5 s (D6), and 13_7 is static for 13.2 s (D7).

A narrow native patch is staged in two layers (below). It is **uncompiled and unrendered**, and root owns integration.

## Disclosure

- I did not write `arithmetic.rs` in this session, and this is not my earlier complex-analysis code.
- 13_9 to 13_12 were staged by LavenderPike (Mail 945, `algebra/patches/abstract_algebra_expanded-13-followup.patch`),
  a different session of the same kind of agent, sharing a memory store. I read that session's lane note before
  reviewing, so treat this as independent of the author's session, not of its lineage.
- Authorship of 13_1 to 13_8 is not established: the crate is untracked in git. Root's lab records show root proposed
  the 13_2 and 13_8 demonstrations.
- I wrote the staged patch after the review, so **the patch has had no independent review.**
- Root's `arithmetic-voice-review09.json` already noted "labels update after completed movements", and
  `arithmetic-undo-gap01.json` warned that the 13_8 demonstrations might overrun. D2 and D9 turn those observations
  into defects on this build, with anchors.

## Coverage

| Item | Value |
|---|---|
| Sheets read as images | all 149, `sheet-000` to `sheet-148` |
| Selected frames seen | all 595, `frame-0000` to `frame-0594`, as sheet cells (388 unique by SHA-256) |
| Cell against frame | 0 of 595 cells differ from their frame downscaled to 960×540 (`checks/outputs/manifest.out`) |
| Opened at full 1280×720 | 0151, 0303, 0358, 0371, 0480, 0504, 0510, 0548 |
| Pixel crops | 0540 to 0552 against 0552, to settle a faint speck (no defect, below) |
| Per-frame and per-sheet table | `coverage/frames.tsv`, `coverage/sheets.tsv`, with time, cue and SHA-256 |

Sampling is half-second plus cue boundaries. Motion between samples was not examined. I read sheets 000 to 137 and
138 to 148 in two sittings; findings from the first were carried in working notes, and I reopened full-size frames
for the worst cases rather than relying on those notes.

### Provenance

| Input | SHA-256 |
|---|---|
| `render/13_arithmetic.mp4` | `d2dc1bf30421e82412e8667f5a2281b18359175e8c66ea827a86d4241dda1090` |
| `render/13_arithmetic.cues.tsv` | `b6d4a85ad2d369e6e161e855060a54e0a1e32c42574ca9baf32404eda671b75a` |
| `render/13_arithmetic.layout.tsv` | `e3b0c442…b855`: **0 bytes** |
| `src/arithmetic.rs` | `78ed9b54477a452da6f459089d8dca289e71bd548b244a79e88641a5b4d4a651` |
| `script.tsv` | `83619d48676b32d5d93bf7527477aa752a902dd16927c548e4b3052cc2d689c0` |
| `oracle.py` | `803782696455cf666c983b561929d524c4140f9a391a0c3338283ee5ec8e5631` |

The `render/` files sit in `narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-combined-repair-preview01/`.
The eight crate sources (including the three above) match the frames build's input record; `manifest.out` shows each.

### Which clock

Cue lengths in this render are scratch estimates. Where a finding depends on timing I also give it on Oliver take01,
using root's ASR sentence starts (`arithmetic-sentence-audit02.json`). Those are recognition estimates, not hearing.

| Cue | 13_1 | 13_2 | 13_3 | 13_4 | 13_5 | 13_6 | 13_7 | 13_8 | 13_9 | 13_10 | 13_11 | 13_12 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| This render, s | 18.48 | 23.86 | 16.94 | 18.09 | 18.09 | 21.55 | 24.63 | 18.09 | 23.86 | 21.17 | 22.32 | 21.94 |
| Oliver take01, s | 16.00 | 24.64 | 20.08 | 19.92 | 19.60 | 22.56 | 24.08 | 21.68 | 25.28 | 22.72 | 27.84 | 26.24 |

### Lint persistence

`layout.tsv` is empty, so the native lint recorded nothing and the sampler had no layout anchors to add. My read agrees
that nothing overlaps persistently. The empty file is not acceptance, though: it did not flag the near-touches in L5.

## Defects

Times are this render. "Source" is `explainer_corpus/abstract_algebra_expanded/src/arithmetic.rs`, baseline lines.
"Layer" says which staged layer repairs the defect, or "root" if it is left open.

### High

**D1. 13_10: the row boxes contradict the package they sit under.** Layer A.
- Anchor: frames 0468 to 0492, t 206.0 to 216.6; full size 0480. Source 339, 358.
- Row one boxes `3 → 9` under `f∘(g∘h)`, but g∘h takes 1 to 9 and would take 3 to 15. Row two boxes `9 → 8` under
  `(f∘g)∘h`, but f∘g takes 3 to 8 and would take 9 to 26. Each box holds one machine's arrow, not two.
- As drawn the boxes line up under the two packaged machines, which is plausibly the intent, but a box around an arrow
  reads as what the package does. The trail values themselves are correct.
- Repair: span input through output, `[1 → 3 → 9] → 8` and `1 → [3 → 9 → 8]`. Measured clear on frame 0480.

**D2. 13_2 to 13_6 and 13_8: names sit under poses they do not describe.** Layer A.
- Cause: `advance` runs and only then `replace` fades the old name (source 44, 90, and their call sites 150 to 187, 218 to 222).

| Frames | t | On screen | True pose |
|---|---|---|---|
| 0081 to 0084 | 36.0 to 37.5 | `s` | r s |
| 0148 to 0153 (0151 full size) | 64.1 to 66.0 | `r⁴ = e` | brown flipped tile, s |
| 0200 to 0203 | 86.5 to 88.0 | `r` | s r |
| 0206 to 0209 | 89.5 to 91.0 | `s² = e` | brown flipped tile, s |
| 0212 to 0215 | 92.5 to 94.0 | `s` | r s |
| 0233 to 0240 | 101.3 to 104.0 | `r s` | s r s |
| 0241 to 0246 | 104.5 to 107.0 | `s r` | r |
| 0248 to 0252 | 108.0 to 110.0 | `r` | e |
| 0254 to 0258 | 111.0 to 113.0 | `e` | r⁻¹ |
| 0347 to 0361 | 151.5 to 158.5 | `r` | r², r³, then e |

- Repair: clear the old name before the motion and name the result after it. This takes the same 3.0 s per step, so
  nothing downstream moves.

**D3. 13_6: the answer lands before the voice invites the check.** Layer A.
- Anchor: frames 0260 to 0283, t 114.0 to 123.5. Source 185 to 189.
- `s r s = r⁻¹` appears at about t 114. On Oliver the source's fixed steps put it about 12.4 s into the cue, while
  "Track all four corners to check it" starts at 15.14 s and "s r s equals" at 17.66 s. That is about 5.3 s early, so
  the viewer is told to check a result already shown.
- Repair: wait on the phrase "s r s equals".

### Medium-high

**D4. 13_1 and 13_3: `e` before its definition.** Layer A.
- Anchor: frames 0005 to 0045, t 1.1 to 19.5; again from about t 46 under both tiles, and under the right tile until about t 85.
  Source 123, 124, 158, 159.
- `e` is defined by 13_3's last sentence: about t 60 on this clock, 16.62 s into the cue on Oliver. On Oliver `r⁴ = e`
  also lands about 1.6 s before that sentence starts. Neither r nor s has been defined when `e` first appears.
- Repair: label the fixed pose "start" until then, and hold `r⁴ = e` for sentence 6.

**D5. 13_12: each calculation is whole for under a second.** Layer B.
- Anchor: frames 0555 to 0581, t 244.5 to 257.5. Source 497 to 592.
- On this clock: `= (r⁻¹ r²) s = rs` is whole in frames 0565 to 0566 and already dimming at 0567. "Do nothing" appears in
  one sample (0571). The `r (r³s) = s` check is whole only in 0579, fading in at 0578 and out at 0580.
- The cause is sequential card changes (1.35 s) inside short sentences. It persists on Oliver:

| Beat | Sentence window, s | Baseline: asks / full hold | Layer B: asks / full hold |
|---|---|---|---|
| Closure | 3.84 | 1.85 / 1.99 | 1.10 / 2.74 |
| Associativity | 4.64 | 4.10 / **0.54** | 2.85 / 1.79 |
| Identity | 2.88 | 2.05 / 0.83 | 0.75 / 2.13 |
| Inverse | 4.50 | 4.00 / **0.50** | 2.70 / 1.80 |

- Repair: one 0.4 s card change, and captions arriving with the beat's last marks. If 1.8 s is still short for a
  three-line calculation, only a sentence pause in the narration plan can buy more. That is root's call.

**D6. 13_1: static for the chapter's opening.** Layer B.
- Anchor: frames 0007 to 0045, t 2.0 to 19.5 (17.5 s; Oliver cue 16.0 s). Source 117 to 124.
- Two identical tiles labelled `e` and nothing else, while the voice speaks of two sequences, corner destinations and "one
  of these eight". Chapter 07 ends on `4 choices × 2 neighbors = 8` with no rack (source only, `eight_construction.rs`:197 to 206;
  its frames are not in this set), so "these eight" has no referent.
- Repair: a long route and one quarter-turn reaching the same pose, then the eight thumbnails, unnamed.

**D7. 13_7: the reduction does not show the rules being spoken.** Layer B.
- Anchor: frames 0288 to 0339, t 125.0 to 148.7; full size 0303. Source 197 to 212.
- All beats run on fixed waits and finish by about t 135.5, leaving 13.2 s static (frames 0309 to 0339).
- The word collapses with a hole (`r⁻¹ _ r² s`, frames 0300 to 0304), then ends as `r` at x = 0 and `s` at x = 2, never
  reading "r s".
- The rule shown is `s r s = r⁻¹` while the voice says "move the flip past the turn". No flip moves, and "pairs of flips
  cancel" is unpictured.
- Repair: state `s r = r⁻¹ s`, then rewrite `s r s r² s` to `r⁻¹ s s r² s` to `r⁻¹ r² s` to `r s`, in place and on the
  sentences, ending on the rack's two rows lit.

### Medium

**D8. The rack is spoken but absent.** Layer B for 13_7, 13_8; 13_3, 13_5 stay with root.
- "in our rack" (13_3), "elements of the rack" (13_5), "two rows of our rack" (13_7), "stays inside the rack" (13_8). It
  first appears at frame 0508, t 224.0, for about 2.5 s.

**D9. 13_8: motions miss their sentences and run into silence.** Layer B.
- Anchor: frames 0345 to 0388, t 150.5 to 170.5. Source 218 to 239.
- The quarter-turn run starts under "We have found an arithmetic". Two half-turns take 4.4 s against a four-word sentence
  (frames 0366 to 0373). The flip pair finishes after the cue ends (frames 0378 to 0388).
- The scene then leaves a 3.94 s silent gap before 13_9 (cue ends 166.79, next starts 170.73). Every other gap in the
  chapter is about 0.6 s except 13_2 to 13_3, at 2.77 s.

**D10. 13_9: the lit letter is above the idle tile.** Layer A.
- Anchor: frames 0407 to 0408, t 178.5 to 179.0; frames 0423 to 0424, t 186.5 to 187.0. Source 246, 286, 293.
- Letters at x = −3, 0, 3 read as tile labels: the yellow `s` sits above the right tile while the left one flips, and the
  reverse. Spaced that far apart, `s r s` also does not read as one word.
- Repair: a centred word at x = −0.9, 0, 0.9, with boxes to match.

**D11. 13_4: no think time, and the two sequences are never together.** Layer B.
- Anchor: frames 0146 to 0190, t 63.9 to 82.6. Source 166 to 171.
- The flip starts on "What does s squared do?", leaving no think time. After that it is 11.6 s static (frames 0163 to 0190) while the
  voice speaks of "these two particular sequences", though `r⁴ = e` has gone.
- Repair: wait for sentence 1, then show `r⁴ = e` beside `s² = e` on sentence 5. "Same horizontal hinge" still has no hinge line (root).

**D12. 13_6: the picture runs a history nobody speaks.** Layer B.
- Anchor: frames 0233 to 0283, t 101.3 to 123.5. Source 182 to 188.
- "Flip, turn a quarter-turn, and flip again" shows only the last flip, the first two having run in 13_5. Meanwhile the
  right tile undoes `s r` through `r`, `e`, `r⁻¹` with no words. "Which is also r cubed" has no `r³`.
- Repair: restart both tiles: left performs flip, turn, flip on sentence 1, right performs one opposite turn on sentence 3,
  then `s r s = r⁻¹ = r³`. The direction marker for "reverse our sense of clockwise" stays with root.

### Medium-low

**D13. 13_5: the right tile moves before "Turning".** Layer B. Frames 0190 to 0195, t 82.6 to 84.0, source 173.
It starts at the cue's start, and on Oliver "Turning" begins 4.60 s later. The four separate moves also take 12 s against two
3.5 s sentences. Repair: one motion per sentence.

**D14. 13_12: closure reads as a tautology, `r s = rs`.** Layer B. Frames 0555 to 0558, t 244.5 to 246.0, source 508, 712.
Repair: `s r = r³ s`, where `s r` is not a rack name and `r³ s` is. This changes one fact value on both sides.

### Low

- **L1, 13_2:** 9 s static during "one function placed inside another", with no `f(g(x))`. Frames 0062 to 0080. Root.
- **L2, sequential fades:** tiles leave, then labels, so `e e` and `r  r s` float alone. Frames 0048 to 0050, 0101 to 0102,
  0284 to 0287; source 128 to 130, 153 to 155, 191 to 193. Identical tiles also fade out and return at t 20. Layer A.
- **L3, 13_10:** stage empty except the heading while waiting for "any object". Frames 0447 to 0451. Root.
- **L4, 13_8:** "different reflections use different axes" has no second axis; only one is ever shown. Root.
- **L5, near-touch:** a corner disc comes within about 5 px at 720p of a label below it. Frames 0358 (`r`), 0371
  (`Half-turn, then half-turn`). Layer A clears the moving labels and moves those two captions to y = −2.3.
- **L6, 13_11 caption:** "Undoing +3 needs -3" uses a hyphen under mathematics that shows −3. Frames 0532 to 0543,
  source 467. U+2212 is already in the crate's text glyph lists. Layer A.
- **L7, `rᵏ or rᵏ s`:** no range for k. Frames 0309 to 0339. Layer B's lit rows carry it; otherwise root.
- **L8, 13_9:** "Rightmost move first" is removed just before the rightmost-first word runs (source 242). Root.
- **L9, TeX minus:** the minus in `x − 1` and `r⁻¹` looks heavier and shorter than the plus. Frames 0480, 0303.
  Engine or font level, cosmetic. Root.
- **N1, source only:** caption arms at source 76 and 77 are unused. Layer B removes them.
- **N2, script, outside this review:** 13_12 ends "Next we will see how these four properties make deductions possible",
  but `main.rs` puts `01_clock` next, opening "It is eleven o'clock." Cue text is untouched; root may want to check it.

## Looked at, no defect

- **Poses:** every tile pose and all eight thumbnails, by hand against pixels (rightmost move first, r counter-clockwise
  on screen, s the top-bottom flip).
- **13_9:** both packagings end at TL3 TR4 BL2 BR1, and `(sr)s = s(rs)` beside `sr ≠ rs` is a strong beat from t 191.
- **13_10:** chips and trail values 1, 3, 9, 8.
- **13_11:** cards land on their phrases. `2+0=2`, `2+3=5` and `5+(−3)=2` are right, ✗ marks only Identity and Inverse, and
  the refused `5+(−3)` with −3 circled reads clearly.
- **13_12:** `s r s r² s = (s r s) r² s = r⁻¹ r² s = (r⁻¹ r²) s = rs`, `e r = r = r e`, and `r x = s`, `x = r⁻¹ s`, `x = r³s`,
  `r(r³s) = s` are all right. The ending holds cleanly with the heading and cards.
- **Transforms:** garble only mid-morph (frames 0458, 0507, 0520, 0522).
- **Rack thumbnails:** small but legible at 720p (0510). The gold frame's 20 px margin is tight but clean (0504).
- **Speck near centre, frames 0546 to 0549:** an 8-pixel-aligned strip with peak difference 4/255, gone by 0550. It fits
  compression residue of the number line, not a scene object. `checks/speck.py`.
- **Quiz:** no timed quiz in this chapter. The only question is D11's rhetorical one. Premature material is D3 and D4.

## Staged patch

Uncompiled, unrendered, not independently reviewed. Cue text is unchanged (`script.tsv` is not touched), so no take is
staled by wording. Baseline was byte-identical to canonical when built; **`cmp` before applying, since canonical may move.**

| Layer | Patch (apply from repo root) | Changes | Lines |
|---|---|---|---|
| A: wrong or early | `patches/A-wrong-or-early.patch` | `arithmetic.rs` | 162 non-blank added or removed |
| B: teaching and sync, on A | `patches/B-teaching-and-sync.patch` | `arithmetic.rs`, one `oracle.py` line | 238 non-blank added or removed |

Full copies are in `baseline/`, `candidate-A/`, `candidate-AB/`. A can ship alone. B needs A.

**Layer A** repairs D1, D2, D3, D4, D10, L2, L5, L6. Timing is almost unchanged:
- `step` takes the same 3.0 s as `advance` plus `replace`, and 13_8's run stays 10.0 s.
- The three fade sites at 13_2's start and end and 13_7's start lose time (1.2 s at 13_2's start, 0.8 s at each of the others).
- 13_3 waits for sentence 6 and 13_6 waits for its phrase.
- 13_1's tiles become 13_2's demonstration actors, which removes the fade-out and return.
- 13_2's `r` arrives after its quarter-turn rather than before it, in the same total time.

**Layer B** repairs D5, D6, D7, D8 (13_7, 13_8), D9, D11, D12, D13, D14, L7, N1. On Oliver take01 sentence starts:

| Beat | Window, s | Asks, s |
|---|---|---|
| 13_1 routes | 6.10 | 5.20 |
| 13_5 each pair | 3.50, 3.46 | 3.45 each |
| 13_6 flip, turn, flip | 3.46 | 3.45 |
| 13_7 flip past turn; flips cancel; rack | 6.80; 4.72; 3.06 | 3.60; 4.20; 2.01 |
| 13_8 quarter-turns; half-turns; flips | 3.54; 2.30; 5.56 | 3.50; **2.70**; 3.70 (ends at 20.2 of 21.68) |

The half-turns still overrun by 0.4 s. These are sums of literal run times, not measured motion.

**Facts:** keys unchanged. Layer B changes `arithmetic.closure.product` from `rs` to `r^3s`, through `CLOSURE_PAIR` in Rust and
one line in `oracle.py`; that is the only oracle line that differs. `main.rs` is unchanged.

**Tests:** new `each_trail_box_holds_its_packages_input_and_output`, `the_routes_and_the_restarted_tiles_end_as_spoken` and
`the_word_reduction_uses_the_spoken_rules`; `PHRASES` grows to 15; the sentence-count table now covers all twelve cues; two
`const` asserts guard the trail boxes. None has been run.

### Desk checks run

Text and Python only, on scratch overlays; outputs are in `checks/outputs/`.

| Check | Result |
|---|---|
| `rustfmt --check` on both candidates | clean |
| `desk_mirror.py`: waits and phrases against the script, tile and `Elem` mirrors, oracle, glyph coverage, pixel clearances | 0 problems |
| Staged `arity`, `scope`, `shape`, `unused`, `desk` checkers, baseline against A and AB | no new findings; counts 7, 11, 0, 2, 49 unchanged; only line numbers and totals move |
| Patches applied to a scratch copy of baseline, then compared | both byte-equal to the candidates (`package.out`) |

These tools cannot see types, borrows or traits, so rustc may find more. `desk_mirror.py` and `manifest.py` import PIL, so run
them with `python3 -B`, not `-I`. `build_candidates.py` is the exact recipe.

### Root's part

1. `cmp` baseline against canonical, then apply A, or A then B.
2. Run guarded build, tests, Clippy, oracle and facts check.
3. Re-render and read the changed sheets, since **no candidate pixel exists.** Check especially: `normal` gliding to y 1.25
   over the rack; the row frames; the 13_1 thumbnails; and the tiles appearing at once where they follow a fade (`add_tile`
   has no fade-in, as on baseline).
4. Rebind any narration plan that hashes `arithmetic.rs`. `narration-direction-map-oliver24.json` has no chapter 13 rows.
5. Decide the open items: 13_12 sentence pauses; L1, L3, L4, L8, L9; the 13_3 and 13_5 rack; the hinge line and direction marker; N2.

## Not verified

- No listening, no narrated sync, no fine sync, no final AV.
- Motion between samples.
- Oliver timings are root's ASR estimates on take01, unverified by me against the takes root will select. Layer B's budgets
  inherit that limit.
- Chapter 07's ending and the 13 to 01 join are read from source and script, not pixels.
- Whether the candidates compile, pass tests or Clippy, or look right. None has been built or rendered.
