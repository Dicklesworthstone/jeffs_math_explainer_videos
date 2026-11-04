# Complex-analysis Cauchy scene: prediction order

Author observation, 2026-11-02. This is a concrete pacing issue, not an
independent pedagogical review or a listening result.

The current `03_cauchy` source starts cue `03_1`, then displays the heading
“A boundary determines the center” alongside “Can boundary values recover
f(0)?” before calling `kit.hold` and waiting three seconds. The script for
`03_1` also states that boundary values determine the center. In the existing
silent scratch contact sheet `complex-native-picture-sheets01/03_cauchy-001.png`,
the answer-bearing heading and the question are visible together at all four
sampled times: 1.0, 1.1, 6.0, and 11.0 seconds. The sample shows the visual
leak; the unlistened recording does not establish what was actually spoken.

## Narrow proposed repair

Keep the current script text and complete narration takes. Change the initial
heading to a neutral question, for example “What can the boundary tell us?”
Show the circle, function, and question **before starting** `03_1`; leave a
visible three-second prediction interval; then start `03_1` and call
`kit.hold` before advancing to the existing sampled-average and formula
reveal. The answer is then introduced by the narration after the prediction,
without cutting or changing the retained audio. Keep the closed-disk
holomorphicity condition and finite-example qualification visible in the
answer phase.

This is a proposal only. The source file and direction map have not been
changed. A future authorized implementation needs to update the current
source-bound map and record the new before-cue prediction separately from
the post-cue hold, rebuild under the existing guard, and inspect the actual
question, pause, answer, formula and later holds on the resulting pictures.
The current 13-cue preparation manifest still matches the unchanged script;
do not start a bulk synthesis from it before the scene sequence is settled.
Any newly synthesized cue remains unselected until actual listening, and the
grouped original takes must remain intact.

## Bound evidence

- Script `explainer_corpus/complex_analysis_visual/script.tsv` SHA-256
  `5722bc1c2b1844bfc7be96ee63d6b3c847f1448a98fd94486d89108da320b931`.
- Scene `explainer_corpus/complex_analysis_visual/src/chapters/cauchy.rs`
  SHA-256 `7ed28b9fb1db3008a798969c0c5428351defd63ff24bf8aa0e47e5af731bec5b`.
- Viewed scratch sheet SHA-256
  `878b542c20e4cac1286b27fbe86523f40459e2fb7792d79306c5ffef01ff8bc1`.
- Prepared-cue manifest SHA-256
  `1b8947e3b4422795074e82b29bc461f26bd6260ff2c38783568c7721a1a273f8`.

The sample is not a full-motion, full-resolution, audible, or independently
accepted review. The proposed repair has not been built or rendered.

**Scope clarification:** The proposed staging repair preserves the exact script. It does not by itself require changing or regenerating the other 12 unsynthesized cues. The 03_1 request should be confirmed against the final staged script before it is made; other distinct cues can proceed independently when the resource guard admits them. The original grouped takes remain available.
