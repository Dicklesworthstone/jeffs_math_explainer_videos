# Michael continuity and cadence — 2026-10-17 UTC

This is the current narration handoff, not a second task board. Beads owns status. The reusable direction is in the private canonical [narration skill](https://github.com/Dicklesworthstone/je_private_skills_repo/blob/63496ee742c57f78c6de5a8675b281c96dc7ce96/.claude/skills/natural-sounding-narration-with-frankentts/SKILL.md) and its [consistency experiments](https://github.com/Dicklesworthstone/je_private_skills_repo/blob/63496ee742c57f78c6de5a8675b281c96dc7ce96/.claude/skills/natural-sounding-narration-with-frankentts/references/CONSISTENCY-EXPERIMENTS.md). All 41 tracked skill files match the canonical and both installed copies after the N9.3 refresh.

## Latest owner direction and evidence

Return to **FrankenTTS Michael**, preserving his **English accent**. Other models primarily support diagnosis. Preserve accepted audio, use bounded coherent context, author total semantic gaps, and repair whole thoughts instead of grafting words or stretching speech. Screen concrete failures before offering one short, worthwhile comparison. This is the owner's selected direction; broader N7/N11 qualification still requires actual evidence.

- **Accepted B is explicit now.** [`michael-baseline-20261017/baseline.json`](../../narration_lab/michael-baseline-20261017/baseline.json) pins the identity anchor, transcript, source span and gain history. B came from the QUICK FTTS reference route, not native `michael.spk`. This does not resolve the earlier ambiguous preference between two contextual pickups.
- **The delivered CosyVoice X/Y pair was rejected as not Michael.** The owner explicitly said X lost the English accent; do not invent an accent-specific verdict for Y. [`owner-feedback-20261017T181917Z.json`](../../narration_lab/model-sidecar-20261017/run-02/owner-feedback-20261017T181917Z.json) retains the exact mapping and quote.
- **Detectors remain advisory.** The [speaker/accent pilot](../../narration_lab/speaker-triage-20261017/RESULTS.md) gave high similarity/England scores to wrong-person and broken clips, and an accent false alarm within accepted A. The [bounded Qwen screen](../../narration_lab/qwen-base17-20261017/RESULTS.md) withheld a comparison on an automated concern; it was not heard and rejected by an agent. Use held-out source/text groups, matched material and known-good controls before escalating these tools.
- **Generation provenance now catches capped partials.** [Native N10 evidence](../../narration_lab/provenance-20261017/RESULTS.md) distinguishes actual EOS from frame-cap exhaustion and connects the emitted receipt to cache schema 2. This is runtime integrity evidence, not better voice quality. The canonical skill now separates generation, sample-edit, exact-cut review and native workflow contracts.
- **The antecedent pilot has already generated its four candidates.** Its [review manifest](../../narration_lab/michael-antecedent-20261017/review-manifest.json) still marks all four unreviewed. Do not rediscover it as an unexecuted proposal or claim that restoring the antecedent won.

N9.3 (`mx-video-program-program-87do.2.9.3`) covers the canonical skill refresh: nine files updated, 13 behavioral regression cases and five dated incidents added, structure validation and 21 relative-link checks passed, and all 41 tracked files verified identical in the canonical/Codex/Claude installations. The scripts were unchanged and no synthesis, training or new listening occurred in this documentation pass. N9 remains open for broader proven-workflow work. The existing Beads export conflict is separate from the native database updates; do not force an unrelated merge to export this task.

For the next audio decision, retain B and the liked pause baseline, inspect the existing contextual candidates, and spend new generation only on one unresolved defect that can change the decision. Do not restart replacement-model shopping. Any backwards-TTS likelihood, latent probe or audio-language analysis must beat the cheap existing diagnostic baseline on held-out labels and pass its actual license constraints. Drama 3 outputs/results are not a default teacher for improving FTTS; the pinned IV(b) restriction is recorded in the skill.

## Earlier listening history, retained for scope

The first A/B screen was positive; C sounded Irish-like, D like another person and E broken. Later 3X was rejected for severe accent and sound-quality drift; 3Y was rushed. Added pauses improved 3Y's pacing. Continuous rendition 1 with longer, manually placed semantic gaps then sounded much better, but the phrase “much harder job” remained odd. Contextual whole-thought pickups subsequently received positive feedback.

The earlier response, “That sounds better,” followed two clips: a same-word contextual pickup and a simpler-wording pickup. It does not name a winner, approve a new reference voice or qualify full chapters. Preserve both candidates and the liked baseline. Correct transcripts alone did not catch the voice/cadence failures.

The local audio lab retains raw/rejected outcomes, exact inputs, receipts, transcripts and feedback under `narration_lab/michael-20261016`, `narration_lab/michael-followup-20261016`, and `narration_lab/michael-continuous-pauses-20261016/cadence-repair`. Large local media are not included in this documentation commit. The two latest delivered excerpts are `review/same_words_pickup_context.wav` (SHA-256 `a8cfdf37a04cfd3b8807b8cd47be1820e4a21a285e1c234910edf0889f192e0b`) and `review/simpler_phrasing_pickup_context.wav` (`10af81947e4f2e65271771c93aaea9705ff54f8975340379fbf7df48aefcc506`) relative to the cadence-repair folder.

## Retained editing direction

- Direct complete thoughts and include the actual context they depend on. One inspected pickup says “the first two questions” but omits the sentence defining those questions. That omission is real; an audible causal effect is unproven.
- Separate five judgments: words/meaning, identity/accent, within-phrase delivery, spacing, and sound quality. Do not flatten intentional questions and reveals to satisfy a generic consistency score.
- Select a complete acceptable rendition when possible. For a pickup, include synthesis context and retain only the intended target at clean boundaries. Recalculate the final gap including both clips' quiet margins. Avoid word grafts and time stretching.
- Preserve the existing pause helper and selector; do not rebuild them as new capabilities. Five real composition receipts passed fresh verification. Independent FFmpeg PCM hashing tied the played pause baseline to its exact reconstruction.
- Declare a small repair budget. If repair keeps spreading, regenerate the coherent passage. Fewer cuts is only a tie-break among acceptable performances.

## Earlier work sequence (superseded by the update above and live Beads)

1. **N5 / N7:** use the existing frozen good/bad targets and raw/gain/final artifacts before generating more. If a new test changes the decision, choose one contrast: restore the missing antecedent with target words fixed, or remove parenthetical punctuation with words/context fixed. One thought × two arms × two fixed seeds = four requests. Keep all outcomes. Both seed pairs must improve the targeted delivery without content/identity failure before advancing; confirm later on unused material with a separate budget. No such new test ran in this pass.
2. **N5:** use the optional 16-request syntax/context matrix only if their interaction matters. Its schedule is structurally checked, not an executed benchmark. Different text with the same seed is not the same internal random draw; repeated extracts from one request are correlated.
3. **N12:** calibrate optional accent/cadence triage against owner labels, comparing similar phonetic material and rhetorical roles. Entire unseen source takes belong in evaluation. Detectors remain advisory; none is calibrated here.
4. **N11:** qualify longer passages with real listening before changing production defaults. A good repair, clean transcript or completed negative experiment does not satisfy this task.
5. **N9.2:** the scoped canonical skill refinement is complete. Its parent also covers the broader FTTS skill and proven production workflow, so the parent remains open.

Existing IDs: N5 `mx-video-program-program-87do.2.5`; N7 `.2.7`; N9.2 `.2.9.2`; N11 `.2.11`; N12 `.2.12` share the same `mx-video-program-program-87do` prefix. Consult the [Beads index](BEADS_INDEX.md) and native `br show` for current state.

The independent Codex idea pass generated 30 proposals and ranked five: frozen-target replay, missing-antecedent restoration, punctuation-only comparison, raw/gain/final inspection, and recurring accent-sensitive words across rhetorical roles. These are self-ranked proposals, not measured benefits or cross-model agreement. The requested actual two-family duel remains incomplete: a fresh Claude instance hit its weekly limit and the Gemini substitute requires login. No second slate, cross-scores or reveal reactions were fabricated.

## Earlier refinement validation and scope

The final skill passed structure/link checks and fresh review. Its existing 17 editor tests and five real receipt checks passed; the optional 16-cell schedule is complete and balanced. UBS found no supported code language in the five Markdown/JSON changes, so no scanner ran. Nine research claims were independently audited: six verified within narrow scopes, three hypotheses. No new audio, model benchmark, engine change, full video or upload occurred in this refinement.

Detailed local evidence lives in the research run `outputs/next-video-program-20261016/narration-consistency-20261017` in the originating chat workspace; the Bead records the commit and receipt scope. Those run files are evidence, not a second active backlog.
