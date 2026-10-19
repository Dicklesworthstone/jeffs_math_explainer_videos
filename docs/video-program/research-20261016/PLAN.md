# Next video program

Status: four bounded plan reviews completed; cross-model duel remains pending provider access. Research date: 2026-10-16. Scope: reviewed plan and executable backlog; the owner additionally authorized establishing the canonical catalog and cloning the current repository locally. Narration and pipeline changes remain proposed.

## 1. The change we should make first

Make the next video with a stable narrator and a clearer continuous argument before increasing throughput. The first eleven prove that the pipeline can ship; the audience feedback identifies defects its current technical gates did not catch. Words can be correct while the voice changes identity, and a sequence of correct scenes can still fail to teach an argument.

Three workstreams can proceed together: a small narration experiment, production-state tracking, and research/storyboards for a short slate. High-resolution final rendering waits for voice and teaching acceptance. Topic selection does not wait for the tracking software to be finished.

The operational target is one convincing next pilot, followed by a small batch using the same proven method. One thousand videos is not a success metric. Track minutes of accepted explanation, repair burden, and viewer understanding alongside publication count.

## 2. Editorial contract

Every proposed video must answer one concrete question. Its first thirty seconds should expose a real puzzle and show the object the viewer will learn to reason about. The ending must resolve that same puzzle using an idea the viewer has watched being constructed.

Selection gates:

1. Jeff finds the question worth explaining. Explicit enthusiasm is stronger than a like; a lack of response is not a rejection.
2. There is a visually meaningful mechanism, not merely motion behind a spoken lecture.
3. The central result can be explained honestly within the chosen scope. State what is proved, what is assumed, and what remains a deeper extension.
4. A fresh coverage check identifies a material reason for this treatment to exist. Better rendering or a new voice alone is not enough.
5. A tiny numerical or geometric example carries the abstraction. The same example survives through the proof and failure case.
6. There is an independent mathematical source and an executable oracle where the claim is computational.

Use the four owner-reported winners as a prior: useful inference from surprising data; a result hidden by ordinary intuition; a trick whose reason is genuinely satisfying; or a proof whose picture does real work. This is a pattern hypothesis, not an analytics conclusion. Early views are confounded by exposure, upload age, promotion, topic and packaging.

Do not queue generic Fourier transforms, Monty Hall, introductory eigenvectors or a broad AI-math survey merely because replies request them. A narrowly different payoff can still qualify. Do not revive the already-made submarine idea as a new task. Agent-tooling material belongs on Jeff's other channel.

The independent candidate lists, cross-scores, reactions and coverage evidence will determine the final slate; rankings are editorial judgments, never predicted view counts.

## 3. A video is not a task status

Keep three objects separate:

- Video: stable identity, topic and public history.
- Revision: one specific script/audio/scene/package combination, potentially replacing an earlier cut.
- Work item: a claimable task with owner, status, blockers and acceptance evidence.

A video may have a public revision while a new revision is in script review. A task may be closed while the video is not publishable. A failed check does not erase a successful earlier public release. A changed script invalidates downstream evidence for the changed revision, not the historical record.

Beads remains the source of truth for work ownership, dependency edges, task status and decision discussion. Versioned per-video production manifests hold artifact identity and content-specific fields. The manifest is not a parallel task board: it must not duplicate assignees, task statuses or editable scheduling state. A derived view joins Beads and manifests and can be rebuilt into a fresh output location without losing information.

Native custom-field support must be verified before use. The installed CLI visibly supports labels, external-ref, notes/design/acceptance/prerequisites and governing agent_context. Do not stuff arbitrary video metadata into agent_context: that field can inject instructions into descendants. Do not patch SQLite or JSONL directly. If richer native metadata later becomes necessary, propose an explicit br schema/API extension as a separate optional change, with migration and round-trip tests.

The current `br` 0.7.4 contract has native `gate report/list` and policy support. Its gate verdicts bind to the issue's **status revision**, not video content digests, and are documented as project-local metadata that **does not sync through JSONL**. Reuse native transition enforcement after the application validates a portable artifact receipt. Keep the receipt/reference in normal exported issue comments and versioned manifests; a destination checkout must revalidate and report its local gate. Never treat a copied task status as portable release proof.

The production `x` entrypoint already implements `status`. Extend it: the inspected marker directory is crate-scoped even when media is voice-variant-scoped, and freshness uses modification times. Add exact revision/voice/hash identity there rather than inventing a competing pipeline. Start with ordinary Beads statuses and derived video stages; a custom lifecycle status is optional only after actual br/bv round-trip compatibility is demonstrated.

### Minimal representation

Use an epic for every video, including vague and unapproved proposals, and child tasks only for work actually selected for development. The canonical entry point is `docs/video-program/README.md` in the production repository; `catalog.json` holds stable video identities, source suggestions, editorial decisions, revision records and venue publications. The existing `.beads` owns all work. Candidate epics stay deferred until selected; inclusion is not approval. A stable label `video:<slug>` joins tasks; a bounded lane label identifies research, narration, animation, review or publishing. Native status remains open/in_progress/blocked/deferred/closed according to the installed contract. A label is a query aid, not proof that a stage passed. The chat's research tracker is superseded by the canonical repository backlog; do not create a second active schedule here.

One revision manifest per produced cut references, at minimum:

| Field | Meaning | Validation |
|---|---|---|
| schema_version | Contract version | Known supported schema |
| video_id | Stable corpus slug | Unique and immutable |
| revision_id | Human-readable cut identity | Unique within video |
| parent_revision | Prior cut, if any | Exists or explicitly external legacy |
| work_epic | Bead anchor | Resolves in canonical tracker |
| script_hash | Approved spoken script | Content digest |
| lexicon_hash | Pronunciation contract | Content digest |
| narration_manifest | Selected takes and edits | All files and parent takes resolve |
| narration_hash | Assembled dry voice | Digest and sample rate |
| scene_revision | Code plus dependency pins | Reproducible tree or marked draft |
| cue_map_hash | Audio-to-animation alignment | Monotonic, complete, no orphan cues |
| music_hash | Rendered bed/stems | Matches approved mix |
| master_hash | Exact uploaded bytes | Digest, measured media properties |
| package_hash | Title/description/captions/chapters | Deterministic package digest |
| publications | Records keyed by venue/external ID, URL, visibility, observed_at | YouTube and GitHub obligations and authority stay independent |
| gates | Evidence references and verdicts | Bound to relevant input hashes |

Fields for artifacts not yet created are absent with an explicit stage blocker; never fill them with dummy success values. Evidence contains producer command/tool version, exact inputs, output hashes, UTC observation, reviewer identity and a bounded conclusion. Local paths are portable relative paths or private machine aliases, never committed absolute home paths.

Use explicit evidence states: pass, fail, unknown, stale and waived. A waiver names the missed threshold, actual measurement, exact cut, owner disposition and replacement criterion. Historical quiet-peak/loudness exceptions remain visible during import. A required checker that crashes or produces no report is unknown/fail, never a pass. Optional prosody flags retain their documented advisory meaning; no universal pitch cutoff is invented.

### Derived lifecycle

Inbox → shortlisted → researched → story approved → script approved → voice approved → animated draft → integrated review → final verified → uploaded private → publication authorized → public verified → learning captured.

This is the default progression, not one giant scalar status. Research, voice tooling and visual feasibility can run in parallel. Expose three axes in the board: editorial readiness, artifact readiness and distribution state. Also show the next blocking task, current revision and last evidence time. This prevents a public URL from hiding an unfinished replacement cut.

Candidate rejection/defer decisions retain a short reason and source so the same weak idea is not rediscovered indefinitely. A production stage advances only when its gate's actual prerequisites pass. A transient work item can be blocked independently of the video stage.

### Gate contract

| Gate | Required evidence | Main prevented defect |
|---|---|---|
| Select | Hook, owner fit, coverage comparison, scope | Generic or unsuitable topic |
| Research | Claim/source map, exact definitions, oracle, counterexamples | Wrong or overclaimed mathematics |
| Story | One continuous argument, proof spine, visual test, prerequisite budget | Correct pieces that do not cohere |
| Script | Spoken-number normalization, pronunciation, copy audit | Awkward or wrong narration text |
| Voice | Word fidelity, identity/accent continuity, prosody, full dry assembly listening | ASR-green but jarring speech |
| Draft | Cue synchronization, frame/layout inspection, comprehension checks | Labels too fast, animation explains a different claim |
| Integrated | Narration plus music, transition and end-word listening | Music masks speech, edits clip consonants |
| Final | Exact 4K60 master media/hash/AV checks | Wrong or stale render |
| Package | Exact captions, chapters, description, sources, title | Stale metadata or timing |
| Publish | Authorization bound to exact cut and package | Wrong revision or unapproved posting |
| Public | Actual player, captions, native chapters and URLs | Upload success mistaken for viewer success |

Migration imports the eleven known public videos as legacy verified-public records with dated evidence references. Do not claim new voice gates passed historically. Preserve unknown provenance as unknown. Reconcile local and production checkout divergence before writing the canonical tracker or moving any files. Use an ID crosswalk rather than creating a second video for a different title or render filename.

### Invalidation examples

- Script/lexicon/take change invalidates affected voice assembly, cues, scenes timed to those cues, mix, master and package captions/chapters.
- Music-only change preserves dry-voice acceptance but invalidates integrated listening, master checks and upload identity.
- Scene-only change preserves narration when timings are unchanged; it invalidates visual and final checks.
- Description typo preserves audio and master; it invalidates package/public-metadata verification only.
- Engine/model/build change requires the relevant regression qualification before new takes inherit voice approval.
- Publication verification expires as an observation, not as a historical fact. Recheck before a new release claim; do not create a perpetual polling ceremony.

Invalidation must be deterministic from an explicit dependency table. Consumers reject stale evidence whose hashes do not match. Never infer validity from a filename like `final_v4` or from a closed bead.

Immediately before final authorization, recheck selected artifact digests under a short producer reservation or immutable selected-cut snapshot. A Beads row transition cannot freeze external media. Preserve the existing narration-byte check around score generation. Test wrong-voice markers, missing reports, preserved mtimes, changed dirty source and destination-clone gate reconstruction explicitly.

## 4. Narration: diagnose before standardizing

Audience reports support the existence of audible discontinuity; they do not establish its cause. Candidate mechanisms:

| Hypothesis | Mechanism | Discriminator |
|---|---|---|
| H1 context reset | Independent short synthesis starts repeatedly choose different delivery/accent | Same script and pinned voice in sentence, paragraph and verified single-context conditions |
| H2 take selection | ASR-only per-line winners combine incompatible speaker realizations | Compare original selected montage with same-seed/coherence-selected alternatives |
| H3 conditioning/route mismatch | Different builds, model bundles, voice assets or routes yield different speakers | Hash and trace every input/route; compare under fixed provenance |
| H4 text/prosody normalization | Number expansions, punctuation or phonemes trigger unnatural cadence | Same lexical content with controlled script forms; check accent and fidelity separately |
| H5 within-take drift | Longer context itself destabilizes voice or drops/repeats words | Compare opening/middle/end inside continuous takes; longer is not automatically better |

The third alternative is operational: larger chunks may fail because the backend already splits or because conditioning is insufficient. A long string passed to a CLI is not evidence of one coherent generation. Inspect actual text planner, reset boundaries, conditioning, sampler and codec assembly before claiming the experiment's treatment was delivered.

### What the source investigation established

- Existing one-shot `ftts say` has a whole-request text path, active Production sampling with a propagated seed, and richer ICL conditioning for qualified voice packs. A new long-form engine is not required to start testing paragraphs.
- Talk continuation is a different path and currently reduces ICL packs to x-vector conditioning. Resident model warmth is not persistent speech context. Do not combine those claims.
- There are divergent installed skill copies. The older local Codex copy chooses sentences independently and lacks seed handling; the newer local Claude copy already has seeded tags, a seed sidecar check, and joint dynamic-programming selection using pitch/register costs. Mounted production `x` exposes that joint chooser as opt-in. Both inspected planners still split into sentences. The pilot must resolve the actual production path and reuse the existing selector.
- Production already pins an FTTS version, but an executable version string is weaker than binary/asset content identity. Two local PATH resolutions found 0.1.11 and 0.1.7. The remaining cache task adds full provenance and validates immutable outputs; it must preserve seed protections already present.

These are source observations, not evidence that longer takes sound better. See `evidence/narration-source-trace.md` and `CODEX_TECH.md` for paths and limits. The first benchmark must include the **best current production configuration**, including its joint selector when qualified, rather than beating an artificially weak obsolete baseline.

### Controlled pilot specification

Use three ninety-second passages drawn from approved scripts: a narrative passage, a dense numerical passage, and a proof/explanation passage. Preserve exact semantic content across conditions. Include an adjacent transition that the owner has actually flagged when one is identified; do not invent a timestamp from a general complaint.

First record the current production baseline: executable hash, model and voice hashes, seed, route/resident mode, generation configuration, raw text, normalized text, chunk plan, take-selection method, edits and original file hashes. Ensure all arms share the same supported backend; record accidental differences instead of calling them a clean experiment.

Call the complained-about historical assembly **A0**, preserving unknown provenance; call the strongest supported current configuration **A1**. Inspect the same defective passage as raw take, assembled dry voice and final mix before synthesizing anything. A mismatch, clipped join or masking artifact can be cheaper to isolate than a generation defect. A long-take change must beat A1 to become the default; merely beating an obsolete A0 does not qualify it.

Start with a 25–45-second smoke panel using existing `say`, fresh output folders and explicit provenance. It checks route, EOS, lexical fidelity and gross drift; it is not a general quality result. Do not make this cheap experiment wait for a production-grade cache, planner, EDL editor or dashboard. If treatment delivery is valid, freeze the ninety-second confirmation panel. Three primary conditions × three passages × three seeds yield about **40.5 minutes of candidate speech** before retries, not 40.5 minutes of compute. Context pickups have a separate budget and are not a mandatory fourth full arm.

Proposed treatments, subject to live capability findings:

1. Existing sentence-level workflow as actually used.
2. Paragraph takes of approximately 20–45 seconds, generation boundaries verified rather than presumed.
3. Longer 60–90-second takes only if backend supports them coherently without a hidden reset or truncation.
4. Optional targeted context-carrying pickup follow-up: regenerate a neighboring phrase/paragraph around a defect, then retain only the needed region with a safe edit. This is a separate treatment and budget, not assumed equivalent to persistent model state.

Use three predeclared seeds per condition when sampling is supported; if the route is greedy, use one and record that seed sweeps cannot provide distinct takes there. Do not conflate a wrapper seed with an actual sampler seed. Keep synthesis outputs immutable. Record failures, retries, wall time and emitted audio duration.

Separate raw-generation evaluation from selection-policy evaluation. First compare equal-budget candidate outputs; then compare the current independent best-line policy with a continuity-aware policy using the same candidate pool. Otherwise better selection could be mistaken for a chunk-length effect.

For selection, qualify the existing joint chooser before modifying it. Compare its output against plain selection and contiguous-take preference. Its pitch/register penalties are useful candidate signals, but they may smooth a normal question inflection or miss an accent/identity change. A human veto and lexical hard constraints outrank its numeric objective. Record which defaults and opt-in settings actually produced the baseline.

Candidate screening: fw transcript against the exact spoken script; omitted/repeated clauses; clipped onset/offset; unintended silence; peak and loudness; duration/alignment; voice-identity/accent flags. ASR and speaker embeddings are triage signals. They do not certify the British accent, naturalness, or identity.

Listening is the primary perceptual gate. Randomize anonymous conditions, include a hidden repeated anchor to detect inconsistent judgments, and compare equal playback gain without time/pitch manipulation. Jeff supplies the target Michael reference and approves its sound. Prefer three listeners, including a listener able to judge the intended English accent. If only Jeff is available, label the result an owner preference pilot; do not claim population-level superiority.

Rate identity continuity, accent continuity, sentence emphasis, pacing and join quality separately. Include isolated boundaries, a full unbroken passage, and beginning-to-end comparisons. Record timestamped defects with severity and semantic impact. High ASR correctness does not compensate for identity switching.

Predeclared practical adoption gate: no critical missing/changed mathematical content; no severe identity/accent jump accepted by the owner in the assembled pilot; a clear majority preference over baseline on each of the three passages; no systematic within-take deterioration; manageable measured retry burden. These are operational thresholds proposed for the pilot, not statistically powered claims. Conflicting results mean keep the best reversible condition and run a targeted discriminator, not manufacture an aggregate win.

Before unblinding freeze seeds 1, 2 and 3 and retain every performance. Score identical textual boundary locations even inside continuous takes. Each listener gives one passage-level preference after the whole prespecified seed panel; require at least 2 of 3 listeners to favor the treatment on **each** passage, with ties inconclusive. With only Jeff, label it a preference decision. A severe switch is a clearly perceived change of speaker or loss of the intended accent that the owner would reject in a published cut; record anchor examples and timestamps. Rate beginning/middle/end separately. No cherry-picked seed can hide repeated late drift. Set a provisional ceiling of twice A1 synthesis seconds per accepted spoken minute, including retries, before the run. Any budget change must precede unblinding; this is a practical budget, not a scientific constant.

If all long-take conditions fail, keep short/medium takes with continuity-aware selection and investigate conditioning before adding more processing. If Michael cannot meet the gate under a fixed reproducible setup, compare a small explicitly authorized alternative/reference experiment; do not silently replace the channel voice or buy a service.

Separate **absolute acceptability** from **comparative improvement**. If A1 already meets the owner's lexical/identity/quality floor and new methods do not improve it, retaining A1 is a valid positive method decision with no improvement claim. If all methods fail the absolute floor, production remains blocked. A successful ICL/targeted repair gets a new evaluation using the same protocol and a new versioned decision receipt; it does not rewrite the negative experiment. Record producer/evaluation ID, exact hashes, absolute and comparative verdicts, selected method and scope. Add the actual recovery producer to implementation dependencies without creating a cycle. N11 consumes the current positive receipt, not just a closed N7/N8 task.

Productize only the selected method's missing capabilities. A sentence/joint winner need not wait for an unused long-take planner or alignment UI. Reused assembly still needs reconstructable source spans, stable cues and real join listening. A long-take winner must provide the corresponding sample alignment and EDL. Record the implementation delta before starting; listener, editing and implementation time remain separate from the synthesis-cost ceiling.

### From takes to exact cue timing

Keep raw takes, alignment, edit decisions and accepted assembly separate. Use a stable cue identifier for each semantic beat even when multiple cues share one raw take. Forced alignment proposes word boundaries; fw checks text fidelity separately. Treat alignment confidence as fallible around equations, names and repeated phrases.

Each edit records source take hash, start/end sample, target cue IDs, gain and any silence insertion. Cuts occur in verified quiet gaps with speech-edge listening; never cut solely at a transcript timestamp. Preserve consonant tails and breaths that carry phrasing. Tiny fades can suppress clicks in silence but crossfades across voiced phonemes can create doubled speech; verify the actual join.

Regenerate an enclosing thought when a line has a bad accent or emphasis. Sentence-only replacement is allowed when the neighborhood still sounds coherent, not because it is the smallest file. Prefer contiguous regions from one good take. Keep alternative takes for recovery; no destructive overwrite.

The final cue map comes from the accepted assembly, not estimated word durations. Re-time animation to voice. Generate captions and chapter times from that same map. Do not stretch narration to fit frames; do not use pitch correction or loudness processing as a substitute for a good take.

### Expanded voice-model research requested by the owner

Research accent, identity and cadence separately, alongside expressive open-weight generators. Use the model matrix and the 30-to-5-plus-10 voice idea study as the source map. Audio plus a verified transcript and approval/provenance is the portable bridge for existing Michael presets; unrelated model embedding tensors are not interchangeable. First compare reference cloning without training, using the same held-out passages and human identity floor. Productize a new backend only if it improves accepted narration at a measured cost.

For output analysis, reuse existing `tools/prosody_qa.py` and thought-sensitive gaps before adding tools. Add calibrated accent and speaker change-point triage, plus word/phoneme alignment and rhythm diagnostics, as separate advisory measurements. Ordinary F0/emotion variation is not a defect. Models trained on natural speech or acted emotion need a synthetic-narration validation set with held-out texts, source takes and speakers. No global model score can override lexical correctness or the owner's Michael identity judgment.

A TTS model cannot simply be run backwards. A conditional token-likelihood comparison under a small vocabulary of delivery descriptions, or a probe on an actual style encoder, is a bounded research experiment. It must beat cheaper acoustic/dedicated detectors on held-out human labels; generated instruction tags are not ground-truth realized emotion. Flow-model residuals are not automatically likelihoods. Preserve uncertainty and allow the detector to abstain.

Fish Drama 3 is the model in the owner's X link. Its exact license and hardware path require qualification before use; its Community license restricts using materials/results to train or improve other speech generators. Do not plan Fish-to-FTTS distillation as an available shortcut. Training from approved FTTS outputs into a separately eligible model is a different direction; still preserve original voice provenance and a pristine held-out anchor. Do not start expensive training until zero-shot reference transfer has failed a defined test.

## 5. Teaching and visual quality

Build an argument graph before the storyboard: opening question → small example → observed failure → necessary new idea → derivation/proof → boundary case → payoff. Every scene must earn one edge of that graph. Remove or relocate asides that break it. A correct collection of facts is not a complete explanation.

For each beat specify what the viewer should predict, what changes on screen, what remains fixed, the sentence explaining the change, and the inference now justified. A static hold is often the correct animation. Reuse the same objects, colors and spatial positions across the proof instead of resetting the world every chapter.

Mayer/Fiorella support coherence, signaling and proximity/timing of related words and pictures. Apply them as design hypotheses: point at the relevant quantity as it is spoken; place a label beside its object; remove irrelevant motion or music; introduce unfamiliar notation before using it. This literature does not establish a retention uplift for this channel. Captions remain available for accessibility; avoid unnecessarily duplicating the entire spoken paragraph as permanent on-screen text.

Every pilot gets two cheap comprehension probes before final rendering: after the hook, can a real reviewer state the question and stakes; after the central derivation, can they explain why the mechanism works or solve a near-transfer example? A model persona review is useful preparation, never evidence that a human learner understood. If no fresh human reviewer is available, state the limitation and require owner review of the argument spine.

Keep a prerequisite budget. A new term must solve a problem the viewer has already encountered. A full result can be rigorous without proving every background theorem; label the exact imported theorem and make the central derivation complete. Do not advertise a complete proof while skipping the hard step into a montage.

The sound pass should test dry voice first, then the same passages with music. If the bed reduces comprehension or foregrounds the narrator's identity changes, remove it locally. Use sound as explanation only when the correspondence itself teaches, such as audible autocorrelation in Golay pairs.

## 6. Implementation order and stopping rules

Wave A: establish the canonical catalog and preserve the cloned source inventories; preserve current production method; run the cheap existing-audio and fresh-output narration screen; qualify candidate briefs with clear visual prototypes. In parallel, specify the minimal manifest and Beads join contract.

Wave B: complete the bounded listening comparison using existing synthesis and selection. Implement only missing experiment controls. After a method earns acceptance, productize its cache/provenance, grouping and alignment/edit assembly. Tracking exposes actual blockers; legacy eleven-video backfill does not block a new pilot's minimal qualified lifecycle.

Wave C: build one new video with the accepted narration method and complete teaching review. Freeze voice before expensive 4K60 final rendering. Validate the exact package privately, obtain the required publication decision, publish, then verify the public page.

Wave D: apply the method to a small balanced batch. Compare exposure-normalized retention, audience complaints, accepted audio minutes per synthesis minute, and repair time. Do not call a higher raw view count a causal improvement. Keep the old public URLs unless a material defect and owner decision justify replacement.

Stop adding tooling when the next producer can answer: which cut is current, what blocks it, what passed on those exact bytes, and what to do next. Stop research when the pilot differentiates an action worth taking; uncertainty may remain about a deeper mechanism. Defer automatic voice classification until a human-labeled benchmark makes it evaluable.

## 7. Deliverables and acceptance

The final plan includes a ranked slate with concrete briefs, independent cross-scores/reactions, source-grounded narration findings, a controlled pilot protocol, a minimal lifecycle contract, and implementation Beads with dependencies and meaningful tests. Four reviews must identify their actual scope, findings and dispositions. Claim audit outcomes distinguish observations from causal hypotheses and proposed features.

This planning work is complete only when a fresh agent can begin the ready tasks without needing the conversation. Audio quality, viewer comprehension and production-system correctness remain open until their implementation and real acceptance checks occur.

Current publisher of record in the production manifest is BronzeRidge. Package-preparation workers hand off artifacts; only that publisher, or an explicitly owner-reassigned successor, uploads or changes visibility. Preserve separate render, upload-visibility and exact-publication authority, honoring existing applicable approval without asking again. YouTube publication does not authorize a GitHub release.
