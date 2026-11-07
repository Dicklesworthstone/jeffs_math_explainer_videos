# Independent source review 17b

2026-11-07. Delegated reviewer under rootBlueSalmon's Q reservation. Source-only review; no builds, renders, synthesis, audio playback, Beads or production edits. Read governing AGENTS, the two required explainer skills, production-plan sections 0/1/3/4, and independent-math-review17.md. The current ElevenLabs direction supersedes historical narration defaults.

## Verdict

No new release-blocking defect found in the reviewed narration-validation delta or counting/two_planes/gallery/continuing deltas at the hashes below. This qualifies source inspection only. Chapter 09 remains unreviewed pending author handoff; its changing file is not a frozen review candidate. Chapter 08 scene implementation belongs to the other assigned reviewer. This is not the missing independent Claude quorum.

## Checks and findings

- **External unit routing:** read_units admits the native historical maximum of 100 words unless the unit explicitly declares `external_ui_provider == 'elevenlabs'`; the exception requires exactly one complete nonempty cue and at most 10,000 Python characters. Exact script text, script hash, complete ordered cue coverage, and direction-contract checks remain enforced. generate rejects any external-marked unit before accessing native synthesis arguments. unit_spans now requires such a unit to take the external receipt branch; existing text/audio/source/request/voice bindings remain in that branch. No bypass found in this delta. A receipt remains operator metadata and hash binding, not a provider signature or listening proof.
- **Measured:** independently replayed all eight retained fixture cases read-only, using external-unit-check01/tools/coherent_narration.py after byte-comparing it to Q/coherent_narration.py. **8/8 passed**: native100; native101 refusal; actual reviewed08; external10000; external10001 refusal; unknown provider refusal; multiple-cue refusal; native-generation refusal. No assembly performed.
- **Coverage gap, not a demonstrated production bug:** check-external-units17.py does not exercise unit_spans' newly added missing-external-receipt rejection or a successful external full-cue assembly. Add a narrowly scoped receipt-branch negative control in the owning test suite when qualifying assembly. These eight passes cannot close assembly acceptance.
- **Harness reproducibility:** check-external-units17.py calls work.mkdir() on the fixed existing external-unit-check01 directory, so rerunning the script fails before tests. Preserve the existing evidence; use a fresh output directory or a read-only fixture replay. The eight passes above came from the latter, not from falsely claiming the original one-shot script reran.
- **Counting B1/B2:** independently mirrored the exact source-order assertions. All six markers remain in increasing order (relative character offsets 5780 < 6273 < 7110 < 7526 < 8327 < 8904). The removed caption is no longer pushed into left, and is included in the refinement fade. Therefore the identified second-final-fade route is removed in source. This was not a Cargo test or observed absence of a rendered flash. Source-string tests remain coupled to formatting and are not scene execution.
- **Counting mathematical scope:** subarc length is now bounded by step rather than confusing average total-length/n with arbitrary parameter-step length. The CODE panel gives a sufficient local exclusion test; its zeros-minus-poles interpretation uses the film's simple counterclockwise circle and meromorphic-function context. The added nonuniform-parameter example is a separate arithmetic counterexample, not exercise of the chapter model.
- **two_planes:** modulo 2 pi and z != 0 qualifications are mathematically appropriate. The reduced 23 pt rule needs a real glyph/layout/legibility check.
- **gallery:** the zeta defining series carries Re s > 1 in its title throughout that title's lifetime. The 24 pt replacement needs actual rendered inspection; source cannot establish fit.
- **continuing:** the latest source now says “all these roots together are dense; drawn: through order 64”, resolving the optional grammatical concern in the earlier independent review. The finite drawn subset is not described as dense.
- **Frozen narration pair:** reviewed exact 08_5 and 09_1 text hashes below. Their corrections retain size bounds, cancel the 4^n factors correctly, supply the triangle step, and keep the punctured-annulus polygon limit before shrinking the inner radius. No new mathematical blocker found in those texts. The pair is required: 09_1 explicitly depends on 08_5's triangle argument. Source text review does not accept the already synthesized takes, pronunciation or synchronization.

## Pending qualification

At review time no 09 handoff was present in Q. Do not transfer this verdict to boundary.rs. After author handoff, check triangle tiling/orientations, polygon-to-circle order, all new wait markers against the exact frozen09_1 sentences, and absence of early shrink or removable-singularity assumptions. The parent owns that follow-up coordination.

Native build/test/strict lint, actual glyph and picture review, narration sample/receipt assembly, complete take listening, measured cue-word synchronization and chapter resource limits remain open here. No full-film or release verdict is issued.

## Exact reviewed bytes (SHA-256)

| Q file | SHA-256 |
|---|---|
| `coherent_narration.py` | `a7fc6705aead7999134d8619458c8b2973ef771be069d4ce98462ac5f97fc42f` |
| `before-coherent_narration.py` | `95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5` |
| `check-external-units17.py` | `b57dd66d47b2915712543591f78ce53d30c2787d8e00ec13d3290aaf753d28d3` |
| `counting.rs` | `ecff48f7ca1aea98b3ac8eac65ffe8f4a74d5366fd30ce25546def3b627339d1` |
| `two_planes.rs` | `0f22d63febc18e05dc4123bedf84eeadf967938bcbc4260cfb0d9e8925ff4cb9` |
| `gallery.rs` | `7d1cf20fd25c8d9233f3c14426b0196fd4fd6673e4b25becba47b50f2635b282` |
| `continuing.rs` | `9cb4fc0bb87e48d9ce9793d09c0d31b4bfbfbfab7057074db204cf8d6b164f96` |
| `08_5-script.txt` | `15e727f0bf7696a24123a6c7c8d5ac1f56a31ebccb08c0e561e164d4ad7ee8a7` |
| `09_1-script.txt` | `a24b71272c5c2c5560227625ce999c1cf00e5e5208a436d1021efcea1a61a491` |

## Follow-up: direct receipt guard and provisional boundary source

Parent expanded the assignment to permit `review17b-fixtures/`. Its preserved check.py/result.json record direct validation at the same coherent_narration hash above:

- The small synthetic 24 kHz mono PCM16 silence fixture and native-shaped receipt are **test fixtures only**, not fabricated provider evidence. The external-marked unit is rejected with the exact new guard message: “declared external UI unit requires a verified external UI receipt”.
- The real retained earlier Frederick `01_1` take remains accepted by unit_spans when explicitly externally marked: **5 sentence spans, 397440 source samples**, no new assembly artifact. Its receipt is production-audio-stage07-01/decoded01/decode-receipt.json, SHA-256 `c0af1e2fa7d8d8c42b0d38846ef72c220c795881e435a9297a8da639df28223b`. This closes the direct branch-validation gap for one positive and one negative case; it does not qualify new 08/09 audio or whole-assembly output.

Provisional boundary.rs SHA-256 `22456dcfd0c3b9d0a1af1ec28de793719a98b4db1355c59964a308743ae6a15d` was reviewed while explicitly incomplete. It adds proof constants and helpers but **construct still runs the old 09_1 sequence**: REST appears at sentence 3 and RESULT at sentence 4, then the source proceeds to 09_2 without calling the new ring-proof helpers. The approved 25-sentence cue therefore has no implemented proof sequence in this snapshot. This is an integration blocker for rendering that snapshot, not a defect attributed to a finished author handoff. Sent directly to the replacement scene author.

Helper review found no concrete mathematical error for authored parameters. The independent Python arithmetic mirror checks **240 triangles** over n=8/16/32/64: all are counterclockwise, all mixed-radius internal edges have reverse partners, and inner chord clearance remains positive. polygon agrees with model::circle's n distinct CCW vertices. Plane::unit, group, flat and arrow_head_points helper APIs exist in draw.rs. Actual compilation is not established. circle/ring helper pixels, filled-ring seam behavior, text fits, finite arrowhead spacing and every final wait marker still require the completed scene and native picture review. Frozen 09_1 requires small rho fixed during refinement, circle equality before shrinking, and no holomorphy assertion at the hole.

## Completed-scene source review: 08 and 09

Reviewed `loop_sums.rs` SHA-256 `fc0b49ab10fdddc292ad8ab6149ac4a78da0d12da21a64805b786ec55674b577` and `boundary.rs` SHA-256 `ebe528222b771063b02289438ce6cae9ba03df0b2c9ee046160d4f20b6307179`. **No new blocking source/math/cleanup defect found in these implementations.** The previous provisional boundary integration blocker is resolved at this hash. These are staged candidates still subject to final author hash/handoff; source pass does not imply native test pass.

### Chapter08

- Independently checked **33 literal phrase waits**: each occurs exactly once in approved08_5 and positions are monotone. The two dynamic waits in the affine demonstration name the constant and linear sentences in their correct order; the following remainder wait comes later.
- proof_children uses oriented midpoint subdivisions; the centre triangle remains CCW. Independent Python mirror passes12 child orientation/area/half-diameter/half-perimeter cases including a thin scalene triangle, and96 nested scale-cancellation cases. These are arithmetic mirrors, not execution of Rust.
- Primitive-difference chains integrate the constant and affine terms exactly. No finite quadrature of the remainder is substituted for the proof. The displayed chain selection is illustrative and does not fabricate nonzero holomorphic loop integrals. The single-point derivative estimate and compensating4^n factor are correct; epsilon is introduced after the selected nested chain.
- Triangle conclusion retains its neighborhood hypothesis on screen. Longest-side selection is computed, not assumed to be the base. The moving perimeter updater is cleared and its anchor removed. The returned final object set contains the remaining triangle display; previously faded objects are not included for a second final fade.
- Pixel/teaching follow-up: the selected fifth-level square and delta neighborhood are tiny before magnification; verify the viewer can see their relationship. The constant/linear chain equations and bottom triangle-inequality line need actual fit/legibility checks. The code comment claiming the nested chain remains as a locator is stale: nested outlines are faded before the magnified view. That comment is a minor documentation inconsistency, not a mathematical or runtime blocker.

### Chapter09

- Independently checked **29 phrase waits**: each occurs exactly once in frozen09_1 and is monotone. The additional `until("so near a")` target is present in the same sentence after the approach phrase. The complete09_1 end wait now precedes09_2.
- The actual construct now creates the puncture, bounded neighborhood, inner circle, polygon ring,16 triangular tiles, oriented edge arrows, reverse shared-spoke highlight, boundary orientations, and same-orientation polygon equality. It does not assert holomorphy at a.
- Radius SMALL_R stays fixed while side counts increase16,32,64. Constant arc/chord cancellation is shown before the oscillation-times-combined-length estimate. Circle equality is displayed before the shrink tracker is introduced; outer radius and M remain fixed during shrink. The finite final radius leaves the hole marker empty.
- Temporary polygon arrows, tile walks, inset/labels, equations, and small-circle arrows are explicitly faded in their respective phase; final proof cleanup removes the surviving close-up objects. Both approach and radius updater anchors are cleared and removed. I found no analogous duplicate-final-fade caption path in this sequence.
- Actual native acceptance is still required for TeX strings, proof_tex sizing (max width5.1), the23pt multi-integral estimate, arrow visibility and overlaps in the narrow middle column. Phrase existence/order is not measured fine synchronization; sequential animation may run past a phrase even if the marker resolves. Check actual emitted timing against the retained new recording before calling either chapter synchronized.

Native compiler/API compatibility is unproven here; no builds were run. Existing helper API names and types were source-checked, but only the owning native test/clippy/build lane can qualify the finished sources. No audio was played or accepted.

## Final handoff hash confirmation

**Confirmed source review at final boundary.rs `c9e6b5ba84c4cf5ae327a8268d67c4f134484be8538cb99bdf1828ebe051f630` and mod.rs `fedb9075d73901baefb03753d5b8c58417ec4dc458b0c1c55c612ddf501234ed`. No new source blocker found.** Read09-repair-handoff.md (`0c49fcf016263f10d92c22fae7ea108462bf625ee3e88ab222d4b9d5a31c66fe`).

Final boundary differences reviewed: the faint annulus uses the existing opposite-winding ring outline, appears with the between-circles statement, and is explicitly faded before polygon tiles. Polygon name/position pairing uses zip with the same two-element constants. The radius inequality test now checks adjacent windows in the same increasing sequence. These preserve the mathematical order and cleanup. Native annulus fill/hole appearance remains a pixel check, especially at its zero-width joining seam.

Independently rechecked all29 unique ordered literal waits against frozen09_1, and exact equality of09_2 onward through the end of construct against before-boundary.rs. Glyph registry review finds all42 directly literal kit.text/kit.tex arguments in the08 proof represented; the two dynamically selected affine formulas are also explicitly included. All new09 text/TEX constants, including CONSTANT_TEX, are registered. Inclusion means the glyph probe can exercise them, not that native typesetting or fit has passed. The earlier source-only verdict and remaining native/render/sync/listening limitations carry forward to these exact final hashes.

## Repair18 stale-expectation follow-up

Reviewed `mod-repair18.rs` SHA-256 `4f4b17c35bf01aaa587d8d3c97b6e79059baef6925f68c1748a121776d841ce6` against mod.rs, and `storyboard-repair18.md` SHA-256 `e45d2bb52e74557aa95d40c22dded1eebc6211812b87acd0d7a870538768b153` against before-storyboard.md. **No new blocker; these are legitimate stale-source/expectation repairs, not weakened tests.**

- mod changes exactly one expected unspoken string, from “# L bounds |f'| on the loop” to “# L bounds |f'| on this subarc”. The new string exists verbatim in counting::CODE. No assertion, reading threshold, reading window, list membership or other behavior changes. The same on-screen-substring and clean-reading-time assertions still run with the correct, slightly longer text.
- Storyboard changes only08_5/09_1 words and pictures plus the programmer code panel's four initial lines. Both spoken rows equal their exact approved frozen text. The pictures now describe the actual nested-selection/Goursat triangle proof and fixed-radius annulus limit before shrink; the code block matches the repaired per-subarc bound.
- Independently compared **105/105 unique storyboard cue rows** against script.tsv, removing the existing prediction marker as the production parser does: zero mismatches. All105 rows retain exactly five unescaped table cells. Both changed rows preserve feature/hold columns; no prediction/reading table structure changes.
- All **9/9 candidate-receipt02 file hashes** match current staged files. This is read-only hash and source validation, not build18 test execution. The earlier native17 failure remains a real failed run; root owns the fresh guarded build18 result.

Audit note: my first naive text comparison reported the13 existing prediction-marker rows until I applied the production parser's marker-removal rule. A proposed whole-test-module equality check also correctly failed because the expected string itself lives in that module; the decisive check is exactly one string replacement with every assertion unchanged. Neither exploratory check is represented as a product defect.
