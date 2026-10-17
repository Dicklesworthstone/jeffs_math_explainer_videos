# Michael continuity and cadence — 2026-10-17 UTC

This is the current narration handoff, not a second task board. Beads owns status. The reusable direction is in the private canonical [narration skill](https://github.com/Dicklesworthstone/je_private_skills_repo/blob/2a30bf931ef0347fb5138fc8cb7f2dfd9f132d91/.claude/skills/natural-sounding-narration-with-frankentts/SKILL.md) and its [consistency experiments](https://github.com/Dicklesworthstone/je_private_skills_repo/blob/2a30bf931ef0347fb5138fc8cb7f2dfd9f132d91/.claude/skills/natural-sounding-narration-with-frankentts/references/CONSISTENCY-EXPERIMENTS.md). Commit `2a30bf93` was pushed; all 41 skill files match the canonical and two installed copies.

## What the owner has heard

The first A/B screen was positive; C sounded Irish-like, D like another person and E broken. Later 3X was rejected for severe accent and sound-quality drift; 3Y was rushed. Added pauses improved 3Y's pacing. Continuous rendition 1 with longer, manually placed semantic gaps then sounded much better, but the phrase “much harder job” remained odd. Contextual whole-thought pickups subsequently received positive feedback.

The latest response, “That sounds better,” followed two clips: a same-word contextual pickup and a simpler-wording pickup. It does not name a winner, approve a new reference voice or qualify full chapters. Preserve both candidates and the liked baseline. Correct transcripts alone did not catch the voice/cadence failures.

The local audio lab retains raw/rejected outcomes, exact inputs, receipts, transcripts and feedback under `narration_lab/michael-20261016`, `narration_lab/michael-followup-20261016`, and `narration_lab/michael-continuous-pauses-20261016/cadence-repair`. Large local media are not included in this documentation commit. The two latest delivered excerpts are `review/same_words_pickup_context.wav` (SHA-256 `a8cfdf37a04cfd3b8807b8cd47be1820e4a21a285e1c234910edf0889f192e0b`) and `review/simpler_phrasing_pickup_context.wav` (`10af81947e4f2e65271771c93aaea9705ff54f8975340379fbf7df48aefcc506`) relative to the cadence-repair folder.

## Current direction

- Direct complete thoughts and include the actual context they depend on. One inspected pickup says “the first two questions” but omits the sentence defining those questions. That omission is real; an audible causal effect is unproven.
- Separate five judgments: words/meaning, identity/accent, within-phrase delivery, spacing, and sound quality. Do not flatten intentional questions and reveals to satisfy a generic consistency score.
- Select a complete acceptable rendition when possible. For a pickup, include synthesis context and retain only the intended target at clean boundaries. Recalculate the final gap including both clips' quiet margins. Avoid word grafts and time stretching.
- Preserve the existing pause helper and selector; do not rebuild them as new capabilities. Five real composition receipts passed fresh verification. Independent FFmpeg PCM hashing tied the played pause baseline to its exact reconstruction.
- Declare a small repair budget. If repair keeps spreading, regenerate the coherent passage. Fewer cuts is only a tie-break among acceptable performances.

## Next work in order

1. **N5 / N7:** use the existing frozen good/bad targets and raw/gain/final artifacts before generating more. If a new test changes the decision, choose one contrast: restore the missing antecedent with target words fixed, or remove parenthetical punctuation with words/context fixed. One thought × two arms × two fixed seeds = four requests. Keep all outcomes. Both seed pairs must improve the targeted delivery without content/identity failure before advancing; confirm later on unused material with a separate budget. No such new test ran in this pass.
2. **N5:** use the optional 16-request syntax/context matrix only if their interaction matters. Its schedule is structurally checked, not an executed benchmark. Different text with the same seed is not the same internal random draw; repeated extracts from one request are correlated.
3. **N12:** calibrate optional accent/cadence triage against owner labels, comparing similar phonetic material and rhetorical roles. Entire unseen source takes belong in evaluation. Detectors remain advisory; none is calibrated here.
4. **N11:** qualify longer passages with real listening before changing production defaults. A good repair, clean transcript or completed negative experiment does not satisfy this task.
5. **N9.2:** the scoped canonical skill refinement is complete. Its parent also covers the broader FTTS skill and proven production workflow, so the parent remains open.

Existing IDs: N5 `mx-video-program-program-87do.2.5`; N7 `.2.7`; N9.2 `.2.9.2`; N11 `.2.11`; N12 `.2.12` share the same `mx-video-program-program-87do` prefix. Consult the [Beads index](BEADS_INDEX.md) and native `br show` for current state.

The independent Codex idea pass generated 30 proposals and ranked five: frozen-target replay, missing-antecedent restoration, punctuation-only comparison, raw/gain/final inspection, and recurring accent-sensitive words across rhetorical roles. These are self-ranked proposals, not measured benefits or cross-model agreement. The requested actual two-family duel remains incomplete: a fresh Claude instance hit its weekly limit and the Gemini substitute requires login. No second slate, cross-scores or reveal reactions were fabricated.

## Validation and scope

The final skill passed structure/link checks and fresh review. Its existing 17 editor tests and five real receipt checks passed; the optional 16-cell schedule is complete and balanced. UBS found no supported code language in the five Markdown/JSON changes, so no scanner ran. Nine research claims were independently audited: six verified within narrow scopes, three hypotheses. No new audio, model benchmark, engine change, full video or upload occurred in this refinement.

Detailed local evidence lives in the research run `outputs/next-video-program-20261016/narration-consistency-20261017` in the originating chat workspace; the Bead records the commit and receipt scope. Those run files are evidence, not a second active backlog.
