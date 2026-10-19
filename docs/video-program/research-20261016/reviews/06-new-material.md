# Independent review of the new voice research and catalog

2026-10-16. Reviewer: Codex claims-audit agent, independently reviewing material authored by the voice wizard and orchestrator. Scope: `VOICE_IDEA_WIZARD.md`, `evidence/voice-model-matrix.md`, and `catalog-staging.json`, plus their relevant primary sources and local source mechanisms. No synthesis, listening, model download, training, browser interaction, Beads mutation, repository mutation, or remote execution occurred. Only this review file was written.

Reviewed snapshot SHA-256:

- `VOICE_IDEA_WIZARD.md`: `d072948c37a603b0fd6f7da4c0c90f37dd4ceb90dc4ea7cb8e039dba3c982a3a`
- `evidence/voice-model-matrix.md`: `c311f1d599f4d84b0f4d87faffb1a2521d4fd10e8e9ea4262d69bdf97cba720f`
- `catalog-staging.json`: `28f742471bb14f610049e5c725d1bfc83d44abce86540ef2970ddeef9708f67d`

The voice proposals are suitable for bounded experiments. No critical compatibility claim was contradicted in the inspected paths. The catalog's identity/cut/publication separation is sound, but its initial migration is incomplete. The findings below do not reopen the settled plan or require speculative research to block narration production.

## Actionable refinements before interpreting experiment results

### R6-1 — Distinguish a practical ICL pipeline comparison from a mechanism-isolating experiment

`VOICE_IDEA_WIZARD.md:65–71` compares the bundled Michael vector against a QUALITY pack enrolled from synthetic Michael audio. This changes the voice representation and the conditioning route together. Local `crates/ftts-cli/src/synth.rs:2169–2225` constructs reference-continuation conditioning for ICL; it does not synthesize through the pack's embedding in that branch. Enrollment also derives new material from the chosen reference. The comparison can answer whether that complete recipe is preferable, but cannot isolate the benefit of ICL from the selected reference and re-enrollment.

**Correction:** label the two-arm result a pipeline comparison. If a causal attribution to ICL would change the next engineering decision, add an embedding-only control enrolled from the **same cleaned reference PCM**, with preprocessing pinned, within the separate diagnostic budget. Keep the existing preset as the production baseline. Do not add this control to every confirmation run or require it before a useful practical audition.

Apply the same distinction to `VOICE_IDEA_WIZARD.md:103`: CosyVoice zero-shot and `inference_instruct2` use different prompt interfaces. A neutral-versus-expressive direction comparison should hold the `inference_instruct2` route and reference constant; an ordinary-zero-shot versus instructed comparison is a whole-recipe comparison. The official [pinned examples](https://raw.githubusercontent.com/QwenAudio/CosyVoice/074ca6dc9e80a2f424f1f74b48bdd7d3fea531cc/example.py) confirm these separate interfaces. The draft already warns against the confound; make the actual experiment receipt choose one interpretation explicitly.

### R6-2 — Specify the counting unit for detector qualification

`VOICE_IDEA_WIZARD.md:95–97,163–166,178` correctly separates development from held-out material, proposes grouped splits, and labels its thresholds as unachieved. It also uses overlapping windows. Twenty overlapping windows or multiple excerpts of one defect cannot provide the same independent evidence as twenty source takes. “Defective examples” and “severe events” are different denominators.

**Correction:** record source-take/text/reference group IDs, event identity and severity, and the number of independent held-out groups beside window counts. Deduplicate a continuous defect into one scored event for severe-event recall. Compute uncertainty with the grouping respected, or explicitly label descriptive counts when the sample cannot support that. If there are no severe events, report recall as unestimable. Retain the existing proposed practical targets; do not turn this refinement into a larger mandatory dataset or a production blocker.

## Catalog findings and migration disposition

### R6-3 — Import known unuploaded cuts and preserve their disposition

The reviewed catalog has **64 unique video IDs, 17 unique YouTube records, 11 public observations and six private observations**. Every publication points to a revision within its video, all related-video IDs resolve, revision IDs are unique within each video, and all imported MD5 strings have valid 32-hex syntax. These are structural checks, not rehashes of media or fresh YouTube verification.

However, the current production `docs/publish/MANIFEST.md:144–145,209` explicitly records additional masters absent from the staging catalog: the RaptorQ leo comparison, fish Michael v1, and fish Michael v2 HOLD. Other superseded masters appear inside prose. Importing only the upload table makes known finished/held work disappear from a system intended to cover every stage.

**Correction:** include those known revisions without fabricating publication records or external IDs. Preserve explicit superseded/HOLD/do-not-upload/owner-choice reasons at revision scope; carry unknown or partial provenance honestly. A generated `legacy-<YouTube-ID>` revision identifier is an acceptable migration key for uploaded cuts, not a rule requiring every revision to have an upload. Do not create a duplicate video for an alternate voice or another cut of the same explanation.

**Parent disposition received during review:** accepted; parent will import all explicitly documented unuploaded/superseded masters, preserve unknown/partial hashes and exclusion reasons. This report did not verify the later integrated catalog and does not claim that migration fix has landed.

### R6-4 — Keep publication-event dates distinct from observation dates

The staging catalog assigns `decisions[].on = 2026-10-15` with `decision = published` to all eleven historical videos. Yet the manifest records the first four going public on October 12 and Mental Math, Fair Coin and Matrix on October 14. October 15 is useful as a dated historical observation/import boundary, but it is not those publication-event dates.

**Correction:** retain the dated observation and either use the documented event date for the publication decision or label that decision as a historical-state import. Store `published_on` only where supported, otherwise unknown. Do not derive publication age or an analytics window from `observed_on`. The global warning that observations are historical is good but does not disambiguate the individual `published` decision's date.

The current `production_approval = historical_granted` values should remain descriptive of the old work. They must not authorize a replacement revision or another venue. The existing exact-cut approval and separate-venue gates already establish that boundary; preserve it when wiring catalog consumers.

### R6-5 — Sanitize public copies of the research

The raw voice documents contain absolute home paths, a private mounted-host path, local skill paths, and agent coordination details. The freshly cloned production repository's `AGENTS.md`, Public-Repository Hygiene section, prohibits committing private host names or absolute home/machine scratch paths. The staging catalog's repository-relative references are already a better pattern.

**Correction:** publish a sanitized repository copy using repository-relative source locators and neutral runtime labels; retain the original local research evidence privately. Do not copy the entire scratch research directory blindly, especially logs or browser-derived account material.

**Parent disposition received during review:** accepted; a sanitized canonical copy is planned and originals will remain private. No completed sanitization scan is claimed here.

## Bounded positive checks

- **Existing capability is reused.** Local inspection confirms vector-backed Michael, QUALITY reference/text conditioning, and embedding-only `talk`. Teacher-forced conformance tests consume captured oracle fixtures and can skip without fixtures; the matrix does not misrepresent them as a ready arbitrary-audio classifier. The current production prosody script hashes to the matrix's stated `8e74710b…` and preserves its advisory-versus-failure distinctions. No runtime execution was needed for these source-only checks.
- **The first sidecar has a real interface.** The pinned CosyVoice example exposes reference-audio instruction synthesis. Its [checkpoint card](https://huggingface.co/FunAudioLLM/Fun-CosyVoice3-0.5B-2512/blob/29e01c4e8d000f4bcd70751be16fa94bf3d85a18/README.md) labels the weights Apache-2.0; the pinned code license also reads Apache-2.0. That supports a bounded candidate, not a claim about Michael retention, runtime cost, or every dependency's terms.
- **Qwen capabilities are correctly separated.** The [pinned official wrapper](https://raw.githubusercontent.com/QwenLM/Qwen3-TTS/022e286b98fbec7e1e916cb940cdf532cd9f488e/qwen_tts/inference/qwen3_tts_model.py) exposes Base cloning separately from CustomVoice/VoiceDesign. No portable cross-architecture `.spk` identity or free-form actor-instruction support is invented for the current FTTS path.
- **The consequential license distinctions checked out.** Direct retrieval of the pinned Drama 3 license confirms the revenue/hosted-use conditions and broad restriction on improving other generators from its materials/results; the IndexTTS license separately restricts improvement of other models, with its stated exceptions. Pocket's pinned card identifies CC-BY-4.0 weights plus gated-use terms. These are document observations, not a determination of the owner's license category. The plan correctly avoids assuming a permissive distillation teacher. [Drama 3 license](https://huggingface.co/fishaudio/drama-3/blob/e0a9218aa48888270f06ef1ed9ea67622c0e17d2/LICENSE), [IndexTTS-2 license](https://huggingface.co/IndexTeam/IndexTTS-2/blob/740dcaff396282ffb241903d150ac011cd4b1ede/LICENSE.txt), [Pocket card](https://huggingface.co/kyutai/pocket-tts/blob/7f364c05d1be3faf2b234cdbe4cbb68a026d4218/README.md).
- **Memory claims remain bounded.** The [Drama 3 installation requirements](https://speech.fish.audio/install/) and [pinned int4 card](https://huggingface.co/fishaudio/drama-3-int4/blob/a6f588691a13b6607cb09f9800556487ab7aa0fe/README.md) support separating weight size from total serving memory and excluding an assumed local Mac path. No model was installed or capacity benchmark reproduced.
- **Inverse-TTS reasoning is appropriately limited.** Target-only teacher forcing, actual factorization, unsampled log probabilities, prompt/padding masks and termination are the correct objects to investigate. Normalizing candidate compatibility scores does not calibrate emotion probabilities. The text explicitly acknowledges exemplar/reference confounding, non-identifiability, codec nuisance and the distinction between autoregressive likelihood and flow loss. Probes, latent search and audio-language interpretation remain different experiments.
- **Leakage controls are substantive.** Whole-take/text/reference grouping, held-out Michael material, text-only/acoustic baselines, wrong/no-audio controls, same-accent identity negatives and expressive-Michael positives address plausible shortcuts. These proposals still need an actual implemented split/receipt; no accuracy is established by their presence in a document.
- **Ordering is economical.** Approved existing audio, current conditioning paths and current QA precede one optional sidecar. Detector, inverse-analysis, adaptation and porting work do not block the core listening decision. The parent's proposed N12–N16 integration and clarified N5 smoke probe preserve that intended ordering, subject to subsequent verification of the actual Beads edges.

## Review boundary and next action

This was a critical-path source spot check, not a second exhaustive audit of every model/card in the matrix. No conclusion depends on undocumented access to model weights or perceived audio quality. Accept the new research for planning after folding R6-1/R6-2 into experiment interpretation and R6-3/R6-4/R6-5 into migration handling. Then inspect the integrated catalog and actual native Beads graph once IDs and dependencies exist. Avoid another full review of unchanged material.
