# Michael continuity and cadence — 2026-10-17 UTC

This is the current narration handoff, not a second task board. Beads owns status. The reusable direction is in the private canonical [narration skill](https://github.com/Dicklesworthstone/je_private_skills_repo/blob/63496ee742c57f78c6de5a8675b281c96dc7ce96/.claude/skills/natural-sounding-narration-with-frankentts/SKILL.md) and its [consistency experiments](https://github.com/Dicklesworthstone/je_private_skills_repo/blob/63496ee742c57f78c6de5a8675b281c96dc7ce96/.claude/skills/natural-sounding-narration-with-frankentts/references/CONSISTENCY-EXPERIMENTS.md). All 41 tracked skill files match the canonical and both installed copies after the N9.3 refresh.

## Latest owner direction and evidence

**2026-10-21 — expanded five-cut feedback and automated direction:** The owner
reports “unnatural pauses and rushed parts” and accepts “even 20 minutes or more”
when better pacing improves the video. The affected review set is the five
`expanded-review01-20261021` revisions in the catalog; no passage-specific times
were supplied. This is feedback on the batch, not a listening verdict identifying
each individual boundary. The owner also asks for a concrete motivating setup,
definitions before specialist terms or metaphors, and an example to return to
throughout the explanation. Apply these requirements to all new videos.

The owner explicitly requests automation: use FrankenWhisper word timestamps
and compare actual pauses with where a professional, subject-fluent narrator
would pause to teach the material. Implement a script-first director map with
reasons and proposed ranges for questions, contrasts, definitions, worked steps,
and visual/prediction holds; identify phrases that should stay connected. Compare
that map with source audio and the assembled cue timeline. Separate measured
timing discrepancies from editorial hypotheses and listening judgments. ASR can
absorb silence into a word, so check waveform evidence around candidate breaks;
never automatically cut a low-energy consonant or breath. Use local articulation
and concept density to prioritize rushed explanations, not a universal words-per-
minute pass score. Reuse existing prosody measurements and exact-sample editors.

The old 10–15 minute target is superseded as a ceiling. All five delivered files
remain preserved comparison baselines. Full-cut audio acceptance and publication
remain open. The current work is tracked under the existing narration program;
the dated earlier waiting/production-order directions below do not cancel it.

**Implemented 2026-10-21 — semantic cadence audit:**
[`tools/narration_director.py`](../../tools/narration_director.py) now compares
script-first direction with FW words, source waveform estimates and native scene
waits. [The real five-cut run](../../narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/pacing-story-20261021/RESULTS.md)
covers all 465 cues, with 89 authored semantic boundaries and explicit heuristic
defaults elsewhere. It reconstructs exact cue PCM, exposes unmatched/coarse ASR,
counts recorded and assembly silence once, protects known visual tasks, and ranks
dense/fast cues relative to each video's own pace. Six fresh master excerpts were
transcribed; timestamp changes with input/context demote uncertain internal
findings. The resistor code-panel reading deadline is explicitly protected.
Fifteen focused tests pass. Diagnostic candidates are not calibrated audible
defect labels or permission for automatic pause surgery.

For reuse, run `uv run --no-project --with numpy python3 -I tools/narration_director.py
plan --manifest INPUT.json --out MAP.json`, author semantic roles and reasons,
then run `audit` with `--manifest INPUT.json --map MAP.json --out NEW_DIRECTORY`.
The map binds script hashes; `visual_holds` bind evidence for reading/demonstration
tasks, and optional master-excerpt verification checks timing sensitivity. Use
the resulting [review page](../../narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/pacing-story-20261021/audit05/review.html)
to localize work. [Five revised setups and callback plans](../../narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/pacing-story-20261021/setup-and-callback-revisions.md)
address the missing conceptual anchor separately from pause duration. Beads
`mx-video-program-program-87do.2.17` retains the unfinished production application;
no new full cuts or listening approvals are claimed by this diagnostic work.

**2026-10-19 priority and listening feedback — supersedes the earlier production sequence:** The owner directs us to redo the narration for square packing, medical p-values and Golay before working on new videos, and to deliver the **complete revised cuts** for their own review before anything is published again. On the packing sample, the owner said it is “sort of OK, but it's too rushed” and asked for “a more measured pace” when discussing technical material. This authorizes further repair; it does not accept the sample's voice, pronunciation, cadence or a complete cut. The earlier request to wait for another sample verdict is superseded. Existing tasks `.5.10`, `.6.7` and `.7.8` own this work; new-video work is deferred with its files preserved.

Apply the feedback to the explanation itself: clearer complete thoughts, deliberate space between technical steps and visual observations, and contextual rereadings where articulation within a phrase is rushed. Extra silence after a question alone does not repair a hurried explanation. Count recorded silence and scene delay once; preserve intentional prediction windows. Do not time-stretch, alter pitch, impose a uniform pause after every sentence, or infer audible improvement from ASR or timing measurements. Retain the exact accepted Michael and medical-opening excerpts and all rejected variants. The agents cannot hear audio; technical checks and the owner's listening verdict are separate.

The three formerly published cuts remain rejected and private. Prepare new complete cuts with revised narration, synchronized pictures and captions, and checked joins/mix. Present their exact local paths for the owner's continuous audiovisual review. Neither a successful render nor owner approval of an excerpt authorizes publication; renewed explicit publication instruction is required after full-cut review.

**2026-10-18 explicit production resume:** After the accepted opening was integrated, the owner asked why all three agents were still idle and explicitly directed continued narration repair, especially questions, inflections and pauses, without more approval requests. The owner also authorized local final 4K60 renders as each video is actually ready, followed by the next three-video batch. The orchestrator had applied the earlier repair limits too broadly: they bounded failed attempts, not all subsequent production work. Continue concrete repairs with new, stated hypotheses and small generation budgets; preserve exact accepted audio and all previous variants. Do not reroll an exhausted identical condition or infer listening acceptance for new passages. Current active assignments are in existing Beads `.6.7`, `.5.10` and `.7.8`; the old idle hold is superseded. No video is presently established as finished, and the rejected medical full cut remains rejected.

**2026-10-18 medical opening A accepted:** The owner listened to “New reading A” on `narration_lab/production-20261018/p_values_medical/performance-opening-20261018/listen.html` and said, “New reading A sounds good. I think the biggest issues tend to be around questions and inflections and pauses after questions”. The page maps A to `candidate-2.wav`, 20.86 seconds, SHA-256 `60b0fa3141713045c1eadc7d7a4363e06c276dfb29ff3ad7a3d358a5a27abbe0`, rechecked against disk. Preserve that exact performance and its authored pauses; integrate its revised words into the canonical script, scene cues and captions without regenerating it. Candidate B remains unreviewed and the earlier full medical master remains rejected. This acceptance resolves the bounded opening, not full-cut or general-method qualification.

Use the owner's question feedback to prioritize rhetorical question phrasing and the transition to its answer across all three videos. Distinguish an ordinary question/answer pause from an intentional viewer prediction hold, and count the natural recorded gap plus native scene delay before adding silence. Do not impose a generic rising contour on every question, alter pitch, or infer acceptable inflection from ASR punctuation or gap metrics. Preserve accepted readings and repair only identified failures in context.

**2026-10-18 production listening rejection (supersedes promotion of the drafts):** The owner listened to `explainer_corpus/p_values_medical/renders/narrated-full-20261018T1837/p_values_medical_master.mp4` (SHA-256 `b1073a93c52836d0fe46565f17acd56ca4133e414e1672c9ede864da900b0203`) and said, “the cadence, pauses, prosody, etc. are still very poor and unnatural.” This exact rendition is rejected. The earlier approved 25.23-second comparison remains approved; the method has not transferred successfully to production. Root stopped the in-progress Golay 4K render and preserved every file. Do not resume full renders or promote existing chapter audio on technical checks alone.

The approved comparison inserted five deliberate semantic pauses (2.05 seconds total) plus 0.30 seconds of ending room. The medical assembly records 107 sentence spans with zero authored inserted gap samples. Its native scene cue gaps range from 0.104 to 5.161 seconds, including intentional prediction windows. There is natural silence inside the recordings; these measurements neither mean every pause is wrong nor explain all audible defects. They establish that the deliberate pause direction was not carried over. Within-phrase stress and intonation remain covered by the owner’s negative verdict and cannot be repaired by adding silence alone. Work now on a bounded medical passage: inspect raw/dry/mastered delivery, author total semantic gaps including existing margins, and repair whole thoughts where the source performance fails. No additional approval question is requested; no agent listening is claimed. Existing Beads `.7.8` and `.2.11` own this remediation.

**2026-10-18 owner update:** “the new passage sounds good.” This refers to the new 25.23-second passage in the coherent comparison below, SHA-256 `d2cf3cd94935291dee47d51635b197915695f1017aa60c8896cdf9f438f850d7`; the bytes were rechecked. The owner then requested autonomous completion of production gaps and script/video quality control without further approval requests. The listening hold is resolved. Preserve this accepted audio and the original B-producing QUICK pack; proceed with bounded production narration, exact content checks, semantic pauses and timed integration in the three selected topics. Keep new audio review scope explicit; no unseen chapter inherits this passage's listening verdict.

Return to **FrankenTTS Michael**, preserving his **English accent**. Other models primarily support diagnosis. Preserve accepted audio, use bounded coherent context, author total semantic gaps, and repair whole thoughts instead of grafting words or stretching speech. Screen concrete failures before offering one short, worthwhile comparison. This is the owner's selected direction; broader N7/N11 qualification still requires actual evidence.

- **Accepted B is explicit now.** [`michael-baseline-20261017/baseline.json`](../../narration_lab/michael-baseline-20261017/baseline.json) pins the identity anchor, transcript, source span and gain history. B came from the QUICK FTTS reference route, not native `michael.spk`. This does not resolve the earlier ambiguous preference between two contextual pickups.
- **The delivered CosyVoice X/Y pair was rejected as not Michael.** The owner explicitly said X lost the English accent; do not invent an accent-specific verdict for Y. [`owner-feedback-20261017T181917Z.json`](../../narration_lab/model-sidecar-20261017/run-02/owner-feedback-20261017T181917Z.json) retains the exact mapping and quote.
- **Detectors remain advisory.** The [speaker/accent pilot](../../narration_lab/speaker-triage-20261017/RESULTS.md) gave high similarity/England scores to wrong-person and broken clips, and an accent false alarm within accepted A. The [bounded Qwen screen](../../narration_lab/qwen-base17-20261017/RESULTS.md) withheld a comparison on an automated concern; it was not heard and rejected by an agent. Use held-out source/text groups, matched material and known-good controls before escalating these tools.
- **Generation provenance now catches capped partials.** [Native N10 evidence](../../narration_lab/provenance-20261017/RESULTS.md) distinguishes actual EOS from frame-cap exhaustion and connects the emitted receipt to cache schema 2. This is runtime integrity evidence, not better voice quality. The canonical skill now separates generation, sample-edit, exact-cut review and native workflow contracts.
- **The antecedent pilot has already generated its four candidates.** Its [review manifest](../../narration_lab/michael-antecedent-20261017/review-manifest.json) still marks all four unreviewed. Do not rediscover it as an unexecuted proposal or claim that restoring the antecedent won.
- **The updated skill has now been exercised on a new coherent passage.** The [four-take screen](../../narration_lab/michael-coherent-20261017/RESULTS.md) retained one original-B-lineage take and withheld both fresh QUICK enrollments from B for advisory accent mismatch. Its [single comparison](../../narration_lab/michael-coherent-20261017/listen.html) contains a 25.23-second Golay opening beside unchanged B. Final ASR matches all 75 words; actual EOS, sample-edit verification, zero clipping and matched −26.5 LUFS are recorded. Every original speech sample survives before fixed gain; pauses were inserted only at visually inspected quiet boundaries. At initial delivery owner listening was pending; the 2026-10-18 update above records approval of that exact passage. The technical screen itself does not establish audible acceptance or prove that re-enrollment generally fails.

N9.3 (`mx-video-program-program-87do.2.9.3`) covers the canonical skill refresh: nine files updated, 13 behavioral regression cases and five dated incidents added, structure validation and 21 relative-link checks passed, and all 41 tracked files verified identical in the canonical/Codex/Claude installations. The scripts were unchanged and no synthesis, training or new listening occurred in this documentation pass. N9 remains open for broader proven-workflow work. The existing Beads export conflict is separate from the native database updates; do not force an unrelated merge to export this task.

The comparison's initial listening hold is superseded by the 2026-10-18 approval above. Retain B, the accepted coherent passage and the liked pause baseline. N5.6 records the bounded technical screen and owner disposition; N5/N7/N11 remain open for their broader acceptance conditions. Produce the selected videos using the preserved route; do not reroll the accepted passage, restart replacement-model shopping or run another research batch without an observed unresolved defect. Any backwards-TTS likelihood, latent probe or audio-language analysis must beat the cheap existing diagnostic baseline on held-out labels and pass its actual license constraints. Drama 3 outputs/results are not a default teacher for improving FTTS; the pinned IV(b) restriction is recorded in the skill.

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

## 2026-10-24 — performance diagnosis and direction-preserving assembly

The owner's new report supersedes any inference that prior accent success established good pacing or prosody. Work continues autonomously under `mx-video-program-program-87do.2.18`; the [third-wave checklist](third-wave-20261024/TODO.md) is canonical. [Measured results and controls](../../narration_lab/third-wave-20261024/root/narration-performance-20261024/RESULTS.md) distinguish a production direction-loss repair from unproven perceptual improvements.

For new third-wave synthesis and assembly, use `coherent_narration.py --require-directions` with a script-hash-bound plan, exact map identity and a disposition for every selected authored boundary. The adapter rejects unknown/renamed cues, stale or ambiguous phrases, missing/duplicate dispositions, changed evidence and claims contradicted by inserted samples. Preserve visual/prediction waits in the native timeline. Final-master coverage, alternate assembly routes and explicit supersession lineage still require integration; a disposition alone does not establish the truth of a listening judgment.

The new `tools/narration_performance.py` diagnostic separates active articulation from added quiet and exposes uncertain FW edges instead of treating them as measured vowels. Dual pitch trackers and endpoint perturbations are triage, not naturalness or accent classifiers. Questions need authored rhetorical intent: a prediction, rhetorical question and continuation need not share an upward terminal contour. Protect connected word groups, contrastive focus and mathematical scope rather than using punctuation-only pause rules.

The first three experiments produced 14 TTS takes and 2 exact-sample pause controls, without changing the pinned Michael assets. Longer staged wording was inconsistent; targeted 0.42 s silence changed only the chosen boundary; the continuous-versus-separate same-word comparison was inconclusive under its predeclared alignment/voicing requirements. No automatic performance winner or changed default follows. Six prospective held-out passage contracts are frozen for the next bounded tests. No owner feedback or timestamps are needed to continue this work, but acoustic plots and transcripts must never be described as actual hearing.
