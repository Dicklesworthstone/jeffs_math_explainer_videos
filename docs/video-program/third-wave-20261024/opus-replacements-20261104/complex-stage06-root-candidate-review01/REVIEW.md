# complex-stage06-root-candidate-review01

WhiteSwan, for root Mails 4620 and 4632. This is an independent, read-only review of two things:
- root's stage06 candidate, `EC/combined-repair-stage06-root01` (seal `44cb2a3f`, 58 files), against stage05;
- the two later source changes in the `native-stage06-root03` build workspace.

Frozen once sealed.

**What was not done.**
- Nothing was built, tested, linted, rendered, heard, decoded or synthesized.
- No production file, frozen input, request, take, Bead or TODO was touched.
- The Rust was read, not run. Its arithmetic was mirrored in Python where this review needed it.
- Ruby's stage05→06 reuse helper and the native pictures were not reviewed.

**Independence.** I wrote the five wording proposals in `complex-frederick-remaining-cues-review01`, so on wording this review is not independent. I did not write root's implementation: the bindings, the caption, the map and units, or the root03 changes.

## Verdict

**No blocker. The candidate is correct as sealed, and the build workspace carries it unchanged.**

- **The five corrections:** each replaces exactly one span. 05_8, 10_3, 14_1 and 15_2 match the accepted wording word for word. The fifth, the "recentre" spelling (my finding F4), is resolved as "Recentre" in 14_3 rather than in 09_8.
- **Everything else in the script:** the other 100 cue texts are byte-equal, the 105 ids keep their order, and every changed cue keeps its sentence count.
- **The other files:**
  - Units, the new Frederick map, the storyboard and `closing.rs` change only where these corrections need them.
  - The other 52 crate files are byte-equal to stage05.
  - The root03 workspace adds only:
    - native-stage05's reviewed Cargo files;
    - the zeta branch near 0, in `model.rs` and its mirror in `oracle.py`;
    - the oracle's exterior Cauchy average.
- **Hooks and holds:** the bridge's full source side, run on the candidate with its own hashes swapped in memory, finds the same 93 anchors at the same 85 hook sites. The phrase, sentence, file, line and call of every anchor are unchanged, and all 23 end-of-cue holds are the same, with no problem.

The four findings are Low. None is worth a rebuild on its own.

| # | Sev | Where | Finding | Suggested handling |
|---|---|---|---|---|
| R1 | Low | 14_3 | The script now says "Recentre the series", but the same cue's on-screen card still reads "step by step: re-centre inside the disk" (`continuing.rs:39`). The script uses one spelling now, but within 14_3 the script and the screen still differ. That shows only where captions are built from the script. | Next time `continuing.rs` is opened, change the card to "recentre". That rebinds the `continuing.rs` hash in holds 14_3 and 14_4. |
| R2 | Low, comment | `closing.rs:7–8` | The module comment still says the first caption says "on an open region". It now reads "nonzero derivative:". | Update it with the next real edit. A comment-only edit still moves the file hash that hold 15_3 binds. |
| R3 | Low, comment | `oracle.py` `zeta_borwein` | The docstring says "for Re s > 0", but root03 now also calls it for \|s\| < 0.25, which includes Re s ≤ 0. Numerically it is fine there (z1 below, and root's mpmath check). | Reword the docstring at the next oracle cleanup. |
| R4 | Low, hygiene | sealed crate | `.ee/hook-state/session_start_orient.last.json` is listed in the seal. It is the session-start hook's file, byte-equal to stage05's copy and unused by the build. | Leave the frozen copies alone. Leave it out when integrating into the canonical crate. |

## The five corrections

| Cue | Exact change | Checked |
|---|---|---|
| 05_8 | "No zero found so far sits off that line. Whether all of them" → "Apart from those at minus two, minus four, and on, no zero found so far sits off that line. Whether all the rest" (+10 words) | ζ(−n) = (−1)ⁿ B(n+1)/(n+1) is exactly 0 for n = 2, 4, …, 12 and not for odd n (ζ(−1) = −1/12). So 05_7's "Zeros at minus two, minus four, and on." are real zeros off the line, and the oracle prints `-2,-4,-6` on screen. The four ringed zeros reach \|ζ(½ + it)\| ≤ 6e−15 at t = 14.134725, 21.022040, 25.010858 and 30.424876 (standard-library Borwein, n = 60), matching the printed heights. The marks stay on screen through 05_8. |
| 10_3 | "misses the curve completely." → "misses the whole rise." | The series 0 equals e^(−1/x) at every grid point of the graph's [−3, 0] and differs at every point of (0, 4], where the curve strictly rises. |
| 14_1 | "We know it everywhere." → "We know it everywhere except at i and minus i." (+6 words) | On the exact 1/50 grid of [−2, 2]², 1 + z² = 0 only at ±i. The cue now agrees with its own card, "the bump: defined everywhere but ±i". |
| 14_3 | "Re-centre the series" → "Recentre the series" | 09_8 and 14_3 now both say "Recentre". See R1. |
| 15_2 | "On an open region, that makes" → "On an open region, away from critical points, that makes" (+4 words) | At the critical point 0, z² turns a right angle into 180° and z³ into 270°. At 0.6 + 0.8i, z² keeps 90° (90.0004° at step 1e−5) and stretches by 2. The new caption "nonzero derivative:" / "stretch and turn" uses 07_2's own words ("So a nonzero derivative keeps angles."). It is 19 characters, under the test's 22, and about 1.49 of the 2.3-wide panel by the CMR10 estimate. Panel 0 lights on sentence 2, while the voice says "On an open region", so the caption can leave the region out. Native layout is still unseen. |

## Bindings and source

- **Units:** they differ from stage05 only in:
  - the five texts;
  - the script hash, twice;
  - the map's path and hash;
  - the scope.

  All 105 ids, line ids and texts equal the script.
- **Map:** the new `narration-direction-map-frederick-stage06.json` differs from oliver12 only in:
  - the two override texts, 05_8 and 14_3;
  - the script hash;
  - hold 15_3's `closing.rs` hash;
  - the scope.

  All 23 overrides quote their cue exactly. All 23 holds and 5 helpers match their file's hash and carry their marker on their line. oliver12 stays in the crate, bound to stage05, and no source file names a map. One consequence for any intake run on stage06: `frederick_intake.py` still sets `MAP_REL` to oliver12, so it has to be pointed at the new map. This check did that in memory.
- **Storyboard:** `tools/storyboard.py`'s parser gives exactly the script's 105 lines on stage05 and on the candidate. That is what the crate's own `the_storyboard_the_registry_and_the_script_agree` test requires. The tool's gate check reports the same three "no prediction pause" chapters (05, 08, 12) on both, so they predate stage06.
- **Script header:** only the comment line changed, to name Frederick.
- **Cargo:** root03's `Cargo.toml` and `Cargo.lock` are byte-equal to native-stage05's.

## Root03's two source changes

- **z1, ζ near 0.** `model.rs` now uses Euler–Maclaurin for \|s\| < 0.25, with N = 30 and 10 Bernoulli terms. Its Bernoulli table equals the exact B₂…B₂₄.
  - My line-for-line Python mirror is within 3e−14 of an independent Euler–Maclaurin reference (N = 40, 25 exact terms) at 81 points of the disk.
  - It is within 3e−13 at 37 points with \|s\| up to 1 and Re s < ½, so the branch is sound well past its edge.
  - The oracle's Borwein branch is within 3e−14 over the same disk.
  - Both give ζ(0) = −½, and ζ′(0) = −log(2π)/2 along both axes.
- **z2, exterior Cauchy average.**
  - Why −1/65535 is right: every surviving term of the series outside the poles has k divisible by 8, so every one is negative. The 16-point average of the bump on \|z\| = 2 is therefore −1/65535; the sampled sum agrees to 3e−17.
  - The old error: stage05's closed form −2⁻¹⁶/(1 + 2⁻¹⁶) was off by 4.7e−10, below the printed −0.000015.
  - Rust: `facts.rs` always sampled the average directly.
  - Oracle output: the stage05 and root03 oracles both exit 0 and print the same 131 facts, and root03's two new cross-checks pass.
  - So no displayed number changed.

## Takes, by text only

- **stage05 downloads:** 56 cues have them. None is a corrected cue, and their texts are byte-equal in stage06.
- **stage06 requests:** all 51 carry the stage06 text, its hash, script `0bae7acb` and Frederick's settings. All 51 have download receipts, the five corrected cues among them.
- **Coverage:** together the two folders cover all 105 cues.
- **Not checked:** I opened no audio, and nothing here says how any take sounds.

## Replay

```
R=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage06-root-candidate-review01
(cd $R && shasum -a 256 -c SHA256SUMS)
python3 -I -B $R/checks/stage06_candidate_review.py
```

- **Where to run it:** from the repository root.
- **What it writes and runs:** it writes nothing. It runs the candidate's and root03's `oracle.py` once each, read-only, and the pinned bridge's source side in memory.
- **Exit codes:** 0 PASS, 1 FAIL, 3 if an input moved.
- **Transcript:** `checks/outputs/stage06_candidate_review.txt`. While the take folders are unchanged, a fresh run reproduces it byte for byte. The r1 counts change as takes are added.

## Limits

- Source reading is not viewing: the caption and holds are as coded, not as rendered.
- The mathematics checks what the words claim, not how they are spoken.
- The width figure is an estimate.
