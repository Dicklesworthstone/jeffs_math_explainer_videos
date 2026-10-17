# Narration source trace — parent investigation

2026-10-16. Source and read-only CLI evidence, not an audio experiment. No synthesis, listening verdict, product edit, or quality improvement is claimed.

## Correction after independent source audit

The initial trace below inspected the **older local `<CODEX_HOME>/skills` copy**. It cannot stand for the current production wrapper. The mounted production `x:643,666–689` resolves `$SKILLS`, defaults to `<CLAUDE_HOME>/skills`, and already exposes opt-in seeded reads and joint choice. The local newer `<CLAUDE_HOME>/skills/.../scripts/narrate.py:1045–1082` has join pitch/register penalties and dynamic-programming selection; `:1183–1187` hashes seeded tags; `:1236` still splits sentences; `:1300–1302` invokes joint choice. Its `tts_batch.py:69–80` validates a sidecar seed and passes it to FTTS. Actual skill hashes and JOINT settings used by each public cut remain unproven here.

Production `x:644–652` already resolves one FTTS path and checks its version. Its comment records a previous mixed-build incident; this is repository documentation, not a reconstruction of those original files. The remaining gap is **content-level provenance**, not an absence of all seed or version protection. Seed identity and a sophisticated joint chooser must be reused and qualified before designing replacements. Pitch continuity is still not proof of speaker/accent continuity.

## Initial findings, scoped to the inspected copies

1. **The installed orchestration intentionally creates independent sentence takes.** In the deployed narration skill, `scripts/narrate.py:855` enumerates `sentences(text)`; `:921–935` evaluates candidates independently and chooses `cands[0]`. This gives no inspected adjacent-speaker compatibility constraint. That is a plausible contributor to jumps *between* lines, not proof that it caused the reported videos' defects. Production must record which skill installation and script hash it actually called.
2. **Existing `ftts say` already has a whole-request synthesis path.** `crates/ftts-cli/src/lib.rs:2162–2339` resolves conditioning and passes the request into synthesis; `synth.rs:2102–2288` prepares its text and constructs a generator. The inspected path does not split the supplied prose into sentence calls. A new long-form engine is therefore not the first dependency of a paragraph-take experiment. A synthesis trace and emitted-audio check must still verify treatment delivery in the pilot, including EOS and admission limits.
3. **Sampling is real; the old greedy/no-op-seed doctrine is stale.** `lib.rs:2200` defaults the seed to 0 and propagates it; `synth.rs:2258–2271` uses `SamplingMode::Production` with the seed. The CLI field documentation at `lib.rs:325` still calls it a future sampler. Do not repeat that help text as implementation truth, or describe different seeds as guaranteed distinct useful takes.
4. **Cache identity is weaker than generation identity.** `narrate.py:801–808` derives tags from text/voice/model strings, not executable/model/voice contents, explicit seed, or resolved route. `tts_batch.py:30–37` reuses a WAV over 1,000 bytes and invokes `say --no-resident --voice`; it does not supply a seed. `:24–26` removes FTTS_PROFILE from child environment but that does not exhaust possible provenance. Old cache reuse can conceal a changed treatment. This is a concrete correctness gap worth fixing before evaluating voice changes.
5. **There are two local CLI versions.** Parent PATH resolves `<LOCAL_BIN>/ftts` **0.1.11**; the Codex pane resolved `/opt/homebrew/bin/ftts` **0.1.7**. Both were live read-only checks. Pin the executable and hash it; do not assume the machine needs an upgrade. Source checkout HEAD was `7718a5f6` when inspected; a source checkout and installed binary are distinct evidence objects.
6. **Richer reference conditioning already exists.** `synth.rs:524–558` selects ICL when a voice pack contains codec codes and reference transcript; otherwise it uses an x-vector. `lib.rs:2178` routes ICL away from resident transport to preserve conditioning. The README describes built-in presets as enrolled x-vectors. A QUALITY `.ftvoice` experiment can test an already available capability, if an approved reference is available. Do not claim Michael's present preset contains richer reference prosody.
7. **Resident warmth and utterance continuation differ.** `talk.rs:499–501` explicitly limits the continuation path to x-vector streaming and reduces ICL packs to embeddings there. `talk.rs:283–292` addresses incremental text/token boundaries. Loading one model once does not itself carry linguistic/audio context between independent `say` requests. A continuation-capable ICL implementation is a possible later engineering task, not a prerequisite for one-shot longer ICL takes.
8. **A generous frame cap is not a quality promise.** `crates/ftts-core/src/admission.rs:111–115,368–400` computes text-derived EOS/admission limits. Long takes must be checked for truncation, repetition and within-take drift. Never infer minutes of reliable narration from the theoretical maximum frame count.

## Minimal discriminating sequence

- Freeze one actual existing narration run and its full provenance, including the precise orchestration script; preserve its original WAVs.
- Verify a short sentence and a paragraph both take the expected single-request path without generating an entire video. Trace raw versus normalized text and generation resets.
- Evaluate equal-budget sentence, paragraph, and longer-take conditions using identical semantic passages, pinned asset hashes, supported explicit seeds, and immutable outputs.
- Separately reselect from the same candidate pool using continuity-aware choices, so selection effects are not attributed to context length.
- Only if medium/long takes fail under fixed provenance, test richer ICL reference conditioning or a narrowly scoped engine change. Keep within-take drift as a serious competing mechanism.

## Where source evidence stops

No source inspection establishes that paragraph takes sound better, that the English accent is stable, or that an embedding model detects accent accurately. No timestamped original defect was reconstructed in this pass. Human listening against an owner-approved Michael anchor remains the decisive check. ASR establishes bounded lexical evidence and can miss both accent changes and correct-sounding wrong mathematics.

## Sources

- `<FTTS_REPO>/crates/ftts-cli/src/{lib.rs,synth.rs,talk.rs}` and `crates/ftts-core/src/admission.rs` at the inspected checkout.
- `<CODEX_SKILLS>/natural-sounding-narration-with-frankentts/scripts/{narrate.py,tts_batch.py}` deployed local copies.
- Live `command -v ftts`, explicit-path `ftts --version`, and `ftts say --help` calls. Installed executable provenance must be recaptured at implementation time.

These absolute paths belong only to this private research dossier. Production manifests must use repository-relative references or private host aliases, following the production repository's public-data rules.
