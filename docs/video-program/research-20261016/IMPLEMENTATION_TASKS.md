# Implementation task specifications

Source-backed future tasks. Native Beads in the canonical repository owns status and dependencies. This snapshot contains no completion claims; use BEADS_INDEX.md there to resolve keys.

## A1 — Reconcile canonical production checkout and tracker before migrating work

Lane: tracking; priority: 1; prerequisites: none; conditional: False.

**Why:** The local video checkout is older than mounted production; a second authoritative board would lose or duplicate work.

**Scope:** Owner explicitly requested cloning the current production directory to the same local project path. Preserve the old local checkout intact, copy production source/media and exported Beads, verify transfer checksums, initialize a fresh local database from the export through br and document the canonical local tracker. Preserve all original issue IDs and dirty source. Resolve eleven public video IDs and all known proposal identities, with research-ID to canonical-ID crosswalk. Do not restore obsolete local task statuses over newer production work.

**Acceptance:** One documented canonical checkout and Beads database; active tasks reconciled by identity; dirty files preserved; all eleven video IDs matched to corpus slugs. Research tracker is clearly marked staging until handoff succeeds.

**Validation:** Compare counts, IDs, statuses, dependency edges and owners before/after any approved CLI migration. Re-running reconciliation produces no duplicate issues. No task is marked done merely because a mount lacks host-local markers.

## A2 — Specify and validate stable video, revision and artifact manifests

Lane: tracking; priority: 1; prerequisites: A1; conditional: False.

**Why:** One video can have a public older cut and an unfinished replacement; task status cannot represent both.

**Scope:** Extend the established docs/video-program/catalog.json schema, which already separates stable video IDs, candidate decisions, legacy revisions and venue publications. Specify complete per-cut script, lexicon, voice/take, cue, scene, music, master and package identities. Every idea remains represented before corpus scaffolding. Reference existing video epics and evidence receipts without copying task owner/status. Absent artifacts remain absent. Paths are repository-relative. Evidence states are pass/fail/unknown/stale/waived; waivers retain measurement, owner disposition and scope. No second task database.

**Acceptance:** Schema validates two cuts of one video and a legacy cut with unknown provenance. Duplicate revision IDs, dangling artifact refs and unsupported schema versions fail clearly. Manifest has no second editable scheduler or task status store.

**Validation:** Unit fixtures for complete, draft, unknown legacy and invalid manifests; round-trip test; input changes have deterministic digest effects. A producer can explain the current cut and artifact lineage from the manifest alone.

## A3 — Implement content-bound evidence receipts and deterministic invalidation

Lane: tracking; priority: 1; prerequisites: A2; conditional: False.

**Why:** Modification times and final_v4 filenames cannot prove a gate checked current bytes.

**Scope:** Add machine-readable receipts containing gate name, scoped input digests, command/version, UTC, reviewer, output refs and bounded verdict. Define dependencies for script, lexicon, dry voice, cues, visuals, mix, master and package. Preserve historical receipts; mark stale relative to the requested cut rather than deleting past evidence.

**Acceptance:** Changing script invalidates affected narration/timing/downstream artifacts; music-only edits preserve dry voice but invalidate integrated/final checks; metadata-only edits preserve master but invalidate package/public-metadata checks. Failed or stale evidence cannot authorize transition.

**Validation:** Table-driven mutation tests for each input class plus missing/corrupt receipt, renamed files with changed bytes, unchanged bytes with new mtime, and two simultaneous revisions. Log why each gate is blocked. A required checker crash or missing report cannot mint pass; advisory metrics remain advisory. Validate subject voice as well as cut. Bind relevant dirty source digests rather than HEAD alone.

## A4 — Wire exact-artifact checks into existing x and native Beads gates

Lane: tracking; priority: 1; prerequisites: A3; conditional: False.

**Why:** Existing x status and workflow gates already exist; a disconnected replacement would create more ambiguity.

**Scope:** Extend the canonical x entrypoint and voice/revision identity handling. Validate application receipts before br gate report and status transition. Reuse supported native fields, labels and gate policy; keep ordinary statuses unless br/bv integration explicitly qualifies custom ones. Native br gate status revision is not an artifact hash. Preserve portable evidence in normal comments plus referenced manifests; gate metadata itself is not JSONL-synced. Recheck digests immediately before transition under an immutable selected-cut snapshot or short producer reservation. Preserve existing narration fingerprints around scoring.

**Acceptance:** x status explains current cut, stale gate and next blocker for each voice/revision. A passed marker on a different voice or changed audio cannot pass. A destination clone revalidates evidence and re-reports native gates rather than inheriting an unsupported local verdict.

**Validation:** Integration test real br CLI in isolated temporary trackers: report pass, alter content, attempt transition, leave/reenter status, export/import JSONL, revalidate. Do not edit SQLite or JSONL directly. Old working commands remain usable.

## A5 — Extend the canonical catalog into a derived live video board

Lane: tracking; priority: 2; prerequisites: A4; conditional: False.

**Why:** The owner needs to see every video stage without checking many folders.

**Scope:** Use the established catalog and existing Beads anchors; do not re-import or duplicate the eleven public videos or candidate epics. Build a rebuildable CLI/HTML view of topic, current cut, editorial readiness, artifact readiness, publication state, next blocker and evidence date. Existing dated public records remain historical, and missing historical provenance stays unknown. A simple derived table is enough; no server or second database. Preserve old private review cuts alongside public cuts.

**Acceptance:** Each of the eleven appears once; a public cut and private replacement remain visible together. Filtering by blocker/lane works. Missing links and stale evidence are explicit, not green. Rebuilding into a fresh output location loses no state; do not delete existing files.

**Validation:** Golden fixture with public+replacement, deferred idea, blocked draft, and legacy unknown. Verify imported IDs against docs/video-program/publication-audit-20261015.json, the portable dated audit at repository root, not title matching alone. Check generated links resolve locally.

## A6 — Qualify lifecycle end-to-end and hand off the canonical task graph

Lane: tracking; priority: 2; prerequisites: A4; conditional: False.

**Why:** Schema correctness alone does not prove a producer can operate the workflow across machines.

**Scope:** Run one copied pilot through real br/x operations and intentional invalidations. Test export/import and recovery. Preserve evidence and a short operator guide. The initial catalog/backlog handoff is already established; qualify it rather than duplicating issues. A5 full board does not block minimal pilot qualification.

**Acceptance:** Fresh producer can locate next ready task, claim it, record scoped proof, discover a stale check, recover and export. No cycles or duplicate video IDs; copied/public/private state stays distinct. Research workspace points at canonical destination.

**Validation:** Real CLI integration log with exact versions and digests, br doctor, dep cycles, bv robot triage/plan, and a second-reader walkthrough. Do not claim production qualification from an isolated fixture alone.

## N1 — Freeze a reproducible Michael baseline and defect set

Lane: narration; priority: 1; prerequisites: none; conditional: False.

**Why:** A voice experiment is uninterpretable if scripts, assets, binaries and old caches change together.

**Scope:** Select three semantically fixed passages: narrative, numeric and proof, about ninety seconds each. Identify actual timestamped between-line defects in existing source audio with the owner-approved Michael reference. Record exact executable/model/voice/skill hashes, normalized text, seed, profile, route, generation boundaries and edits. Resolve local-bin 0.1.11 versus Homebrew 0.1.7 explicitly. Preserve existing originals. Separate historical assembly A0 from strongest currently supported pipeline A1, including real SKILLS/JOINT/SEEDS settings. Inspect flagged raw take, assembled dry voice and final mix to distinguish generation from editing/masking. Preserve an approved synthetic Michael reference bank as original WAVs plus verbatim transcripts and non-destructive sample spans. A .spk vector is architecture-specific, not a portable voice. Keep enrollment text separate from evaluation passages; record owner-selected identity and restrained expressive anchors, or the lack of one. Include owner-defined phonetic canaries only for actual suspected accent errors; do not invent a dialect standard.

**Acceptance:** Baseline pack has raw and assembled WAVs or explicit historical unknowns, exact scripts/cues, reference consent/approval and a machine-readable defect register. Do not invent timestamps from general X complaints. A clean approved anchor is available. If no historical seam can be localized, record that bounded absence and proceed with prospective ratings on representative passages; do not fabricate a defect to pass.

**Validation:** Hash/metadata verification and listen to flagged boundaries with surrounding context. Confirm baseline can be identified unambiguously; if historical provenance cannot be recovered, label it historical and generate a separately pinned control later.

## N2 — Make narration caches and take manifests reflect every generation input

Lane: narration; priority: 1; prerequisites: N1; conditional: False.

**Why:** The newer production-facing skill already protects seeded tags, and x pins a version, but neither proves complete executable/model/voice content identity. The older Codex skill copy is not an adequate production baseline.

**Scope:** Change the actually deployed narration orchestrator after resolving project.env skill path. Include raw/normalized text and normalization version, absolute executable resolved to content hash, voice/model content identity, seed, sampler/profile/route/options, and relevant environment in generation manifests and cache keys. Retain old outputs under legacy IDs; do not overwrite originals. Cache hits require manifest plus WAV digest match. Preserve the newer Claude skill seeded-tag and sidecar checks; do not replace them with weaker behavior.

**Acceptance:** A seed, model asset, voice asset, executable, normalization or route change cannot silently reuse a prior take. Partial/crashed writes are not valid hits. Same inputs reuse immutable output safely, with explicit provenance.

**Validation:** Unit key-change matrix; integration fixture with a fake deterministic producer only for cache logic plus a real tiny FTTS smoke when authorized. Simulate truncation/interruption and concurrent requests; prove corrupted or provenance-free WAVs are not accepted as qualified.

## N3 — Productize the accepted take plan with stable semantic cue IDs

Lane: narration; priority: 1; prerequisites: N2, N7; conditional: False.

**Why:** One generation per sentence creates repeated synthesis contexts; the selected method may reuse existing joint selection.

**Scope:** Consume a positive versioned method-decision receipt from N7, or a later qualified recovery evaluation attached by N8. Implement only its missing production capabilities. Reuse and qualify the existing sentence/joint planner if it wins; paragraph or longer-take grouping is required only if selected. All selected paths retain stable semantic cue IDs, pinned text/conditioning/seed provenance and actual request/reset identity. Record the implementation delta before work; defer rejected/unused modes. The experimental N5 path does not depend on this task.

**Acceptance:** Selected method has inspectable cue-to-take mapping and no hidden dropped/repeated content. Existing functionality may qualify through a documented real no-change reproduction; do not invent a long-take feature to close this task. A paragraph winner maps paragraphs to verified single requests and fails explicitly on truncation. The original baseline remains recoverable.

**Validation:** Plan fixtures for numbers, equations, abbreviations, names, paragraph punctuation and stable cue mapping. Capture a real trace for the selected productized or reused path. Repeat the broader comparison only if implementation changes invalidate its method decision. No claim of quality from a command exit code.

## N4 — Qualify selected-method audio assembly and sample-to-cue provenance

Lane: narration; priority: 1; prerequisites: N3; conditional: False.

**Why:** Longer takes help only if their timing can drive existing scenes and corrections can be made without clipping words.

**Scope:** Implement only the assembly delta required by the accepted method. Reuse qualified sentence/joint assembly when it suffices. A paragraph/long-take winner requires alignment proposals and an immutable edit-decision list with source WAV hashes, sample spans, cue IDs, gain and silence edits. Existing full-line spans still need reconstructable provenance and stable cues. Context pickups are implemented only when the selected repair method needs them. fw checks transcription separately from alignment; no stretch/pitch changes. Reuse existing semantic gap rules and prosody QA. If the winning method produces continuous takes, adapt only its missing cue-span interface and listen to interior cuts with consonant context. Regenerate an enclosing thought for a pickup, then retain a verified interior span. Never force every sentence to a new TTS call just to satisfy an old doctrine; change that doctrine only after the selected method passes qualification.

**Acceptance:** Every accepted cue maps monotonically and completely to source samples and assembled samples. No orphan/repeated cue, destructive overwrite, time stretch or pitch shift. Cuts preserve consonant onsets/tails; quiet-gap fades do not double voiced phonemes. An unchanged sentence/joint winner does not wait for an unused long-take aligner; prove its existing full-line spans/cue mapping and actual joins instead.

**Validation:** Round-trip EDL reconstruction is sample-consistent; fixtures cover repeated phrases, uncertain alignment, breath pauses, multichannel/sample-rate mismatch and boundaries. Human listening of actual joins is required; numeric alignment alone cannot close the item.

## N5 — Run the matched sentence-versus-long-take narration pilot

Lane: narration; priority: 1; prerequisites: N1; conditional: False.

**Why:** Source plausibility cannot establish whether longer takes fix Michael or introduce within-take drift.

**Scope:** Use existing one-shot say and current joint chooser, fresh non-reused output directories, explicit executable/asset/seed provenance and temporary sample-span records. First do a 25–45-second smoke panel; do not wait for production cache/planner/EDL work. Then Use N1 passages, pinned provenance and equal candidate budgets. Generate sentence baseline, 20–45s paragraph, and supported 60–90s single-request conditions with three declared seeds each. First score raw generation; context-bearing pickups and selection-policy experiments stay separately identified. Record retries, failure reasons, wall time and accepted-duration denominator. Run fw against exact spoken scripts and inspect mathematical errors manually. Store director intent separately from spoken text: contrast, invitation, uncertainty and reveal, with intended word emphasis. No directions may leak into narration. The initial smoke screen may include one separately budgeted existing QUALITY ICL probe to detect a cheap reference-conditioning benefit; preserve the original three-arm confirmation design, and carry a promising extra route into a separately declared N8 evaluation. No all-models-by-all-lengths Cartesian experiment. Include a hidden anchor and beginning/middle/end samples; do not infer long-form quality from the first line.

**Acceptance:** Immutable candidate set, complete condition manifest, lexical error report and beginning/middle/end continuity flags. No condition gets extra undisclosed retries. Capped failed long takes are reported as failures rather than silently repaired into another treatment. A1 is the strongest supported baseline, not the obsolete Codex skill. Primary three-arm/three-seed/three-passage panel is about40.5 minutes of emitted speech before retries; pickup follow-up has a separate budget.

**Validation:** Verify reset/route trace and hashes for every candidate; independently spot-check fw mismatches including plurals, numbers and proper names. No public upload or full video render. Stop a condition on repeated truncation after its predeclared retry budget.

## N6 — Qualify existing joint take selection against contiguous takes and plain selection

Lane: narration; priority: 2; prerequisites: N5; conditional: False.

**Why:** Choosing the highest per-line ASR score can make individually acceptable lines sound like different speakers when joined.

**Scope:** First inspect and reuse the newer narration skill joint_choice/best_path implementation and mounted x JOINT option. Compare its current pitch/register-cost output, plain per-line picks and contiguous-take preference using the same N5 pool. Add only missing controls exposed by the benchmark. Human accent/identity vetoes and hard lexical constraints outrank smoothness. Selection policy remains separate from chunk length; retain alternatives and reasons. Reuse tools/prosody_qa.py, implemented thought-based gaps (mx-hjti) and calibrated prosody QA (mx-uuao); do not recreate them. Verify report inputs and current advisory semantics. New phrase-rate/emphasis or accent/speaker features are optional and must show useful localization on held-out human labels before affecting selection. N12 detector work is not a blocker of the original selection/listening experiment; if its features are selected later, add the actual qualification dependency then.

**Acceptance:** All variants use the same candidate pool and gain treatment. Fewer seams does not hide word errors or inside-take drift. Chosen sequences can be reconstructed and compared with raw candidates. No opaque score is allowed to certify accent identity.

**Validation:** Small constructed selection cases prove hard lexical vetoes cannot be outweighed by smoothness; actual boundary/full-passage listening compares selections. Report no improvement if none is audible.

## N7 — Conduct blinded Michael auditions and select a production method

Lane: narration; priority: 1; prerequisites: N6; conditional: False.

**Why:** Voice embeddings and fw cannot tell us whether the narrator sounds consistently like the intended Michael.

**Scope:** Randomize anonymous baseline/treatment assemblies at matched playback gain, no pitch/time changes, include a repeated anchor and full-passage listening. Prefer three listeners including one familiar with intended English accent; owner-only is a labeled preference pilot. Rate identity, accent, emphasis, pace, joins and lexical defects separately. Use N5 equal-budget results and N6 same-pool comparisons. Emit a versioned method-decision receipt: producer/evaluation ID, exact input hashes, absolute lexical/identity/acceptability verdict, comparative-improvement verdict and selected method. A1 may be positively retained if it meets the absolute owner quality floor even when no treatment improves it; label no improvement honestly. If all fail the absolute floor, activate N8/recovery, add that producer as a blocker of N3, and keep positive qualification blocked. Do not create a backwards N8-to-N7 dependency cycle.

**Acceptance:** All prespecified results and decision are recorded; closure means the evaluation is complete, not that a method is qualified. A new treatment is adopted only with no critical math omissions/changes, no severe identity/accent jumps accepted by the owner, at least2/3 listener preference on every passage (or explicitly owner-only preference), no repeatable late deterioration, and the predeclared cost bound. Ties mean no comparative improvement. The unchanged A1 can receive a positive absolute-quality decision without beating itself. If no method meets the absolute floor, no positive receipt is issued and N11 remains blocked. Fixed seeds1,2,3, textual boundary denominator, all-seed early/middle/late ratings and provisional twice-A1 synthesis-seconds-per-accepted-minute ceiling apply. Track listener/editing/implementation time separately from synthesis cost.

**Validation:** Check randomized labels, listener instructions and repeated-anchor consistency before unblinding. Report ratings, timestamped defects, sample size, exclusions, cost and limitations. A failed pilot closes only the experiment task, never a positive voice-quality capability claim.

## N8 — Evaluate richer reference conditioning only if the fixed-provenance pilot warrants it

Lane: narration; priority: 3; prerequisites: N7; conditional: True.

**Why:** One-shot QUALITY .ftvoice ICL already exists, but current talk continuation strips richer conditioning; implementing a new engine prematurely is costly.

**Scope:** Conditional branch: if the chosen existing path fails continuity, compare approved Michael reference packs using existing one-shot ICL against the pinned x-vector baseline. Confirm codec codes, transcript and reference provenance. Only propose continuation-capable ICL or other engine work after this test identifies a missing capability. Do not silently substitute another narrator or buy a service. After a promising recovery treatment, conduct a bounded follow-up using the complete N7 absolute/comparative listening/content/cost protocol on those exact inputs. Store a NEW versioned method-decision receipt with its evaluation ID, not a rewritten historical N7 result. Add this task as a blocker for N3 (and any needed conditional implementation tasks) when activated; no reverse dependency on N7 is introduced. Use approved synthetic reference audio from N1, not a renamed/resized preset vector. Compare fixed versus delivery-selected references only after a stable bank exists; changing reference text is a confound. If a separately authorized N13 sidecar wins its screen, add its task as a blocker of this recovery producer and apply the same full follow-up acceptance. Never treat a same-named preset in another model as Michael. Preset-versus-QUALITY is a practical whole-pipeline comparison: it changes both reference representation and route. If attributing benefit specifically to ICL, add an embedding-only enrollment from the exact same cleaned audio as a separately budgeted diagnostic control; QUALITY ICL uses reference continuation rather than the pack embedding. Do not claim a causal ICL benefit from the two-arm pipeline comparison alone.

**Acceptance:** Explicit trigger from N7, fixed comparison, listening result and decision: keep existing route, adopt validated ICL, or scope a separate engine patch with falsifier. If N7 passes, defer/retire this task with that reason. A successful follow-up supplies the positive receipt consumed by N3/N11; a completed negative follow-up leaves them blocked and specifies the next narrow discriminator. Closing N8 alone never qualifies a method.

**Validation:** Inspect actual conditioning and bypass route; same passages and quality gates as N7. No claim that saved model warmth means saved linguistic context. Quantify latency/retry cost only when measured.

## N9 — Refresh narration and FTTS skills around the proven workflow

Lane: narration; priority: 2; prerequisites: N11; conditional: False.

**Why:** The current skills contain stale greedy/seed claims and unconditional sentence-only doctrine.

**Scope:** Update canonical skill sources and deployed copies through the established skills workflow after locating their source of truth. Explain sampled seeds, reproducibility scope, generation provenance, context-sized takes, EDL/cue timing, human identity/accent checks and context pickups. Pin tested commands. Established seed/help/installation corrections belong to N10 and can proceed earlier; this item documents only the demonstrated winning method. Do not claim experimental methods are qualified before N7.

**Acceptance:** A fresh producer can reproduce the accepted tiny example and determine when to use another condition. No stale claim that all synthesis is greedy or seeds never matter remains in relevant instructions. Old videos/original WAVs remain untouched.

**Validation:** Run skill validation and the documented tiny path with pinned artifacts; independently compare deployed content hashes to canonical source. Document partial adoption if an untested route remains.

## N10 — Expose inspectable FTTS generation provenance and correct stale seed help

Lane: narration; priority: 2; prerequisites: N1; conditional: False.

**Why:** The CLI documentation contradicts actual Production sampling, and wrappers must know effective configuration.

**Scope:** Inspect existing --trace and diagnostics first; extend only missing machine-readable fields for effective seed/sampling, resolved model/voice hashes, conditioning route, normalization, reset/continuation mode and truncation reason. Correct stale future-sampler/greedy comments. Keep secrets and private prompts out of default logs. No model/kernel optimization in this item. Correct established false greedy/no-op-seed claims and explain installation divergence in canonical skill documentation without waiting for a winning method. Locate source-of-truth and preserve unrelated differences; no blanket tree synchronization.

**Acceptance:** Consumers can distinguish requested and effective configuration and prove which route produced a take. Existing trace fields are reused. CLI help matches inspected runtime semantics. Any no-change finding is documented instead of creating redundant instrumentation.

**Validation:** CLI contract test and tiny authorized real synthesis demonstrating the reported route; negative unsupported-option and truncation cases. Version schema intentionally and redact sensitive filesystem data in public receipts.

## N11 — Qualify the selected narration method end-to-end before production use

Lane: narration; priority: 1; prerequisites: N7, N3, N4; conditional: False.

**Why:** A completed experiment can have a negative result; downstream production needs positive acceptance, not a closed experiment task.

**Scope:** Consume the latest positive versioned method-decision receipt from N7 or an explicit N8/recovery follow-up, for the exact inputs now selected. Absolute qualification of unchanged A1 is allowed with no improvement claim. Exercise only that method through productized/reused cache, planner and assembly on a real small passage. Check hashes, words, identity, cue mapping and recovery. If no absolute-quality receipt exists, remain blocked. The implementation DAG must include the actual recovery producer when needed. Include a staged early/middle/late stress check on one longer held-out passage only after the short screen passes. Reproduced existing prosody QA is evidence of tool operation, not proof of accent or identity. Record optional detector versions and calibration-set IDs separately from mandatory lexical/human acceptance.

**Acceptance:** Revision-bound positive method acceptance and a reproducible end-to-end passage with no critical lexical or continuity defect. A fresh producer can regenerate or identify exact cached bytes and reconstruct edits. This authorizes only the demonstrated production method, not unlimited long-form quality.

**Validation:** Real tiny synthesis/assembly/fw plus human listening; replay same EDL and inspect route/hash receipts. Negative-experiment fixture demonstrates downstream work cannot proceed from task closure alone. Walk improved-treatment, acceptable-unchanged-baseline and failed-then-recovered scenarios. All require their own exact-input decision; stale earlier evaluation and negative task closure cannot authorize use.

## S1 — Qualify the next small slate against sources, competing treatments and owner taste

Lane: editorial; priority: 1; prerequisites: none; conditional: False.

**Why:** The owner wants deep intuitive topics with a reason to exist beyond better rendering.

**Scope:** Use the independent 30-to-5 slates, X owner replies and final duel. For every proposed first-batch topic watch the closest complete competing treatment, record URLs/dates and exact overlap/difference; absence of search results is not novelty. Write one-page briefs with hook, central theorem, continuing example, visual mechanism, assumptions, scope and kill criterion. Preserve alternatives and owner selection. When owner selects the pilot, add only its qualified-brief task as the blocking edge for S6 and release S6 from deferred status; do not require all briefs.

**Acceptance:** A small selected slate has an explicit distinct payoff and primary source map. Crowded generic Fourier/Monty Hall/eigenvector tours stay parked unless reframed. Owner approval is recorded before full production; conceptual research can proceed meanwhile.

**Validation:** Independent brief review checks that central theorem is actually explainable in allotted length and that title does not promise more. Owner-reported winners remain qualitative evidence, not an analytics ranking.

## S2 — Prove the square-packing brief and its exact visual construction

Lane: editorial; priority: 1; prerequisites: S1; conditional: False.

**Why:** The owner explicitly liked square packing; exact geometry fits the visual-proofs audience, but construction and optimality must stay distinct.

**Scope:** Default candidate: five unit squares in side 2+sqrt(2)/2, exact corner/rotated-center construction, then an intelligible lower-bound proof if feasible. Investigate the owner-suggested recent larger cases as optional coda only after independent proof-status verification. Compare complete closest packing videos. Do not silently substitute n=5 if owner interest is specifically the recent n=11 proof.

**Acceptance:** Exact coordinate/collision oracle, complete selected theorem proof storyboard and clear upper-bound/local-rigidity/global-optimum distinctions. Kill or retitle if the lower bound requires unexplained case analysis; no optimality claim from numerical search alone.

**Validation:** Independent arithmetic/geometric checks at contacts and small perturbations; citation anchors to actual proof, not only summary page. Demonstrate two-page proof spine to a fresh reader before script. Current reported results are rechecked at production time.

## S3 — Build and verify the Golay complementary-echo story

Lane: editorial; priority: 1; prerequisites: S1; conditional: False.

**Why:** Four signed blocks yield a complete surprising cancellation proof and a concrete engineering payoff.

**Scope:** Use A=(1,1,1,-1), B=(1,1,-1,1); compute aperiodic correlations and derive concatenation recursion (A,B),(A,-B). Connect matched filtering to one unchanged linear channel response. Verify historical/spectroscopy attribution from readable primary sources, avoiding unrelated Golay error-correcting-code conflation. Compare closest specialist and general videos.

**Acceptance:** One continuing numerical echo model, exact cancellation proof, practical interpretation and explicit motion/Doppler/common-channel failure case. No claim arbitrary ambient noise vanishes or either sequence alone has zero sidelobes.

**Validation:** Exact integer correlation oracle and recursion tests for small lengths; adversarial different-channel example breaks cancellation as claimed. Human storyboard reader explains why two measurements share an assumption.

## S4 — Develop the medical-survey p-value story and explain conditional reversal

Lane: editorial; priority: 1; prerequisites: S1; conditional: False.

**Why:** The owner specifically requests the doctors survey opening, why smart people confuse p-values with truth probabilities, and medical diagnoses as the explanatory bridge.

**Scope:** Verify Windish/Huot/Green JAMA 2007 and Gao 2020 against the original question and ASA statement. Open with 277 internal-medicine residents in 11 Connecticut programs; 88% reported confidence and 58.8% picked the keyed answer, later challenged. Do not claim a representative survey of all US doctors, a majority chose the 95%-correct distractor, or a flawed forced-choice item proves no doctor understands. Use synthetic medical natural frequencies to show P(test|disease) versus P(disease|test), explicitly separate sensitivity/false-positive-rate/predictive value from study p-values. Move to a null-world study tail probability, effect size and uncertainty, then a nonmedical study/test. Explain the seductive question reversal and ambiguous everyday chance/significance language as pedagogical mechanisms, not measured psychological causes from the survey. Preserve optional-stopping/anytime-valid mathematics as a separate extension or sequel if it overloads this core.

**Acceptance:** A compassionate verified survey hook, exact conditional statements, synthetic-data labels, coherent medical-to-study bridge and a fresh-reader explanation of what a p-value does and does not answer. Survey sample and answer-key controversy accurately scoped. No diagnosis or treatment recommendation, no universal posterior probability inferred from p alone. Coverage review must justify the full treatment without changing the owner-requested premise.

**Validation:** Independent source and math review. Recompute 10000-person examples: prevalence1%, sensitivity90%, FPR5% gives90 true and495 false positives; prevalence10% gives900 and450. Explicitly audit denominators and distinguish diagnostic rates from p-values. Use an exact small randomization/binomial study oracle for the null tail, including at-least-as-extreme and prespecified design. Test a near-transfer example on a fresh reader.

## S5 — Qualify the randomized-response inference brief

Lane: editorial; priority: 2; prerequisites: S1; conditional: False.

**Why:** The slate should include a fish-like inference payoff and a visual proof without committing every candidate to a full render.

**Scope:** Prepare the randomized-response brief independently of every other reserve. Derive aggregate inversion, variance cost, the exact likelihood-ratio privacy guarantee under the stated coin protocol, and repeated-answer leakage. Compare complete closest accessible treatments and identify the distinct payoff before production.

**Acceptance:** Primary sources, a tiny exact example, model assumptions, visual mechanism and distinct treatment are independently accepted or the candidate is parked. No unconditional promise of personal secrecy. Completing another reserve does not satisfy this brief.

**Validation:** Independently enumerate the protocol response probabilities, unbiased estimator and variance; check likelihood ratios and repetition caveat. One-page proof/coverage review can reject without creating a full production epic.

## S7 — Qualify the discrete-tomography visual-puzzle brief

Lane: editorial; priority: 2; prerequisites: S1; conditional: False.

**Why:** This reserve must be independently selectable; a chosen randomized-response video must not wait for it, or vice versa.

**Scope:** Build a binary-grid story with equal row/column sums, 2x2 switching components, a flow feasibility model and the distinction between existence and uniqueness. Compare complete closest treatments and select a precise accessible theorem. Keep this independently deferred if another topic wins.

**Acceptance:** Primary-source anchors, exact ambiguous tiny example, proof spine, model boundaries and a material differentiation. No claim arbitrary imaging is binary line sums or that insufficient projections uniquely reconstruct an object. This brief can pass/fail independently of S5.

**Validation:** Enumerate small binary grids and projection fibers; verify a switching move preserves margins, feasible solutions respect constraints and an explicit ambiguous pair differs. Fresh proof/coverage review before any full script.

## S6 — Make the chosen pilot storyboard a continuous teachable argument

Lane: editorial; priority: 1; prerequisites: S1; conditional: False.

**Why:** Correct facts can still make an incoherent explanation; high resolution cannot repair a missing inference.

**Scope:** Execution prerequisite: at least one chosen topic brief has passed its independent proof/coverage gate; use S2, S3, S4, S5 or S7 as applicable, and add only that chosen branch as a blocker in Beads. Do not require every candidate to finish. Choose one qualified pilot with owner input from the selected slate. Build hook -> continuing toy example -> failure -> new idea -> derivation -> boundary -> resolved hook. For every beat state viewer prediction, invariant objects/colors, changing quantity, spoken sentence and justified inference. Time dense diagrams and remove interrupting asides. Record prerequisite budget and imported theorems. Add a small director-intent note per semantic beat for audition and take choice; it is neither a new cue language nor automatically spoken text.

**Acceptance:** Complete proof/argument spine and low-cost visual prototype. Two real comprehension probes where available: restate opening question, then solve/explain a near-transfer case. If only owner review is available, state that limitation; model personas do not count as learner validation.

**Validation:** Independent mathematical and pedagogical walkthrough; test diagram legibility and hold duration at phone size. Any gap that requires the viewer to guess an unintroduced step blocks expensive rendering.

## P1 — Lock the pilot spoken script and pronunciation contract

Lane: production; priority: 1; prerequisites: S6, N11; conditional: False.

**Why:** Editing spoken text after scene timing creates repeated downstream rework.

**Scope:** Approve one script against the proof spine; spell spoken numbers and mathematical symbols explicitly while retaining display notation separately. Maintain a lexicon including owner pronunciation preferences, proper names and acronyms. Generate accepted dry narration using the N11-qualified selected method, reconcile fw and manually review actual terms/plurals. Bind script, lexicon, takes and cue map.

**Acceptance:** Owner/assigned reviewers accept words, pronunciation, continuous Michael identity and pacing across the full dry assembly. No critical ASR mismatch remains unexplained. Complete immutable cue map and provenance support scene timing.

**Validation:** Full listen plus timestamped lexical/mathematical audit, including boundaries and closing words. Script/cue/hash consistency check. New edits trigger only the documented downstream invalidations.

## P2 — Produce and review the low-resolution animated pilot

Lane: production; priority: 1; prerequisites: P1, A4; conditional: False.

**Why:** Animation must follow the accepted narration and demonstrate the mechanism before expensive final rendering.

**Scope:** Use existing franken_manim pipeline and selected exact oracles. Render a low-cost cut with stable visual language, cue-driven holds and synchronized changes. Review every chapter, interfaces/labels at phone size, proof transitions and comprehension probes. Do not add motion merely to fill narration.

**Acceptance:** All mathematical, layout, timing, legibility and coherence blockers resolved with revision-bound receipts. Full pilot is understandable on its own, with theorem boundaries and source claims intact. Quality review distinguishes actual real-user evidence from model review.

**Validation:** Automated numerical/layout/cue checks where meaningful plus complete audiovisual inspection and selected mobile-size frames. Record defects at timestamps; corrections invalidate affected gates.

## P3 — Mix and render the accepted pilot as a final 4K60 master

Lane: production; priority: 2; prerequisites: P2, A6; conditional: False.

**Why:** Final encode is valuable only after content, voice and timing have passed.

**Scope:** Create music/mix after dry voice acceptance; compare intelligibility with/without bed and omit music where it harms explanation. Render true target 3840x2160 60fps with supported codec/export settings, retaining originals and exact master hash. No upscaling claim disguised as authored 4K detail. Check host capacity through existing project policy before heavy work.

**Acceptance:** Exact final master passes media/AV/layout checks, integrated listening, ending/tails/music transitions, and independent fresh-eyes inspection. Voice method and every input are traceable to the cut. No critical defect is waived by checksum correctness. Owner approval to final-render must be recorded for the exact reviewed draft as required by repository policy. Explicit exceptions are separate from literal measured passes.

**Validation:** ffprobe frame/rate/duration/stream validation, AV sync/end checks, full listening and visual sampling focused on edits; replay significant scenes. Evidence references exact output bytes and actual render inputs.

## P4 — Package the pilot and deliver a private review link

Lane: production; priority: 2; prerequisites: P3; conditional: False.

**Why:** Titles, chapters, captions and sources must match the final cut, not a prior review export.

**Scope:** Write a compelling accurate final title without review markers, concise description with assumptions and readable source links, thumbnail, captions from accepted script/alignment and chapters from final cue map. Upload exact approved master privately/unlisted according to owner review preference, verify account/channel identity, retain YouTube ID and wait for high-res processing. Preserve current publisher-of-record restriction: only BronzeRidge may upload or change visibility unless the owner explicitly reassigns that role. A preparation worker hands off local artifacts and receipts without uploading. Record applicable upload visibility/authority and exact public-release approval separately; do not request again when existing authority already covers the same action/cut. Earlier-video authority is not assumed to cover this new pilot.

**Acceptance:** Private review link resolves for owner with correct cut/package; no accidental public release. Captions match narration, chapter timestamps are monotonic and valid, source URLs open intended primary pages, final title accurately promises the content.

**Validation:** Reload Studio and player; verify visibility, exact title/description, caption cues, native chapters where available and actual 4K60 playback. Upload completion alone is not acceptance. Explicitly record processing still pending if so.

## P5 — Obtain exact-cut authorization and verify public publication

Lane: production; priority: 2; prerequisites: P4; conditional: False.

**Why:** Historical permission to publish earlier videos does not authorize a new unreviewed pilot.

**Scope:** Present exact private cut and package for owner decision. Record authorization scoped to master/package hashes. Publish only that revision after approval. Independently fresh-reload the public page and Studio; preserve canonical URL and observation receipts. No replacing earlier public uploads for trivial subjective changes without a new decision. Preserve current publisher-of-record restriction: only BronzeRidge may upload or change visibility unless the owner explicitly reassigns that role. A preparation worker hands off local artifacts and receipts without uploading. Record applicable upload visibility/authority and exact public-release approval separately; do not request again when existing authority already covers the same action/cut. Earlier-video authority is not assumed to cover this new pilot.

**Acceptance:** Explicit owner approval plus correct public cut. Actual 4K60 playback, captions, native chapters, title, description, source links and visibility verified. A failed or incomplete public check remains an open blocker; successful API/upload response is not enough.

**Validation:** Public viewer-state audit with dated screenshots/DOM and playable rendition evidence where available. Test all source destinations. Reconcile manifest and Beads without erasing private/revision history.

## P6 — Measure the pilot and decide whether to scale the method

Lane: production; priority: 2; prerequisites: P5; conditional: False.

**Why:** Views alone combine exposure, packaging, topic and production quality; a small pilot should teach us what to repeat.

**Scope:** At owner-agreed post-publication windows collect views/impressions/CTR, retention including early exit and flagged seams, like/comment signals, reviewer comprehension and production repair cost. Compare age/exposure and acquisition sources; avoid claiming causality from one upload. Classify substantive voice complaints with timestamps, not generic sentiment.

**Acceptance:** One concise decision: adopt, adjust or reject method/slate pattern with evidence and uncertainties. Report accepted narration minutes and retries/time, not only total outputs. Close feedback actions only when tested; no perpetual dashboard without a consumer.

**Validation:** Check denominators, observation windows, sample sizes, missing data and selection effects. Retention changes near seams are associations until controlled evidence exists. No automated recurring monitoring unless owner separately requests it.

## N12 — Calibrate independent accent, speaker and cadence review triage

Lane: narration; priority: 2; prerequisites: N1; conditional: False.

**Why:** Accent changes and speaker changes are different audible failures; existing pitch QA cannot certify either.

**Scope:** Begin with existing prosody reports, CommonAccent XLSR English and SpeechBrain ECAPA VoxCeleb. Pin code/weights/preprocessing and review any custom loader before running. Resample only analysis copies to required 16k mono. Keep broad accent distributions, similarity against multiple approved Michael anchors and temporal change as separate evidence; England is not an RP or Michael certificate. Split whole takes/texts/references before tuning; use similar-accent other speakers as hard negatives and expressive approved Michael as hard positives. Select detector-disputed excerpts plus random and beginning/middle/end coverage. Optional SER, WavLM or audio-language features require their own terms and demonstrated incremental value; emotion labels are perceived delivery, never inner state. This task can proceed beside N5; it must not delay the cheap human listening pilot.

**Acceptance:** A versioned advisory report and held-out confusion counts quantify utility and abstention. Proposed local qualification requires at least 20 held-out clean and 20 human-labeled defect examples, >=80% severe-event recall and <=10% false flags on clean examples, with uncertainty and denominators. These are proposed operating targets, not achieved statistical proof. With fewer examples, exploratory ranking only. Reject or retire features that add no review-time benefit at preserved defect recall; human rejection always overrides a green panel.

**Validation:** Grouped split/leakage audit; multiple-window aggregation does not count windows as independent takes. Cross text and delivery, check codec/gain/channel sensitivity on analysis copies, compare against existing acoustics and random review. Log exact models, dataset version, hashes, false positives/negatives and reviewer time. Never copy published benchmark thresholds into a production gate.

## N13 — Screen one expressive Michael clone in an isolated sidecar

Lane: narration; priority: 2; prerequisites: N1; conditional: True.

**Why:** A documented reference-plus-direction path may add expressiveness without a new native engine, but a convincing demo may lose Michael.

**Scope:** Choose one contender and state the missing capability before downloads or setup. Default Fun-CosyVoice3-0.5B-2512, whose released inference_instruct2 accepts text, instruction and reference audio; named code/weights Apache-2.0. Use approved Michael WAV plus exact transcript where required; derive the destination model conditioning, never pass .spk as a universal vector. Start four matched short lines with neutral versus restrained explanatory direction and declared seeds/retries, then one coherent excerpt if identity survives. Pin tokenizer, vocoder, runtime, reference and licenses; measure actual peak memory, wall time, failures and accepted speech. Original Chatterbox or Pocket TTS is a separately chosen fallback, not an automatic additional matrix. Preserve watermarking; Pocket weights are CC-BY-4.0 with access/usage terms. Qwen1.7B Base cloning is separate from CustomVoice/VoiceDesign instructions. Parler stock named speakers do not establish Michael cloning. Fish Drama3 and IndexTTS2 remain separately gated reserves: actual terms and capacity must be resolved first; no assumed Fish/Index outputs-to-FTTS training or optimization. No native Rust port, public service, paid API or model training in this task.

**Acceptance:** Document a positive screen, negative result or bounded setup failure. Require Michael identity, accent and exact words before expressive preference. Cap initial setup at roughly two engineering days; unavailable resources or terms defer it. A screen does not qualify production: a promising result must become a declared N8 follow-up with its full N7 absolute/comparative/cost gates and new exact-input decision receipt.

**Validation:** Same text/reference and gain-matched blinded playback; no time/pitch transformations. Record rejected/truncated attempts and directions not honored. Distinguish model-card claims from measured host runtime. Trace actual conditioning route. If activated, add N13 as a prerequisite of the consuming N8 recovery; no reverse dependency. Compare neutral versus expressive instruction on the same instruct2 route; zero-shot versus instruct2 is a whole-pipeline comparison, not isolated emotion control.

## N14 — Test backwards-TTS analysis against simpler audio diagnostics

Lane: narration; priority: 2; prerequisites: N1; conditional: True.

**Why:** A model that can generate expressive speech may contain useful delivery information, but generation ability does not imply an exposed or calibrated detector.

**Scope:** Timebox the first feasibility spike to one engineering day, independently of production. Use an applicable permitted existing runtime and human-rated held-out material. For an autoregressive model, encode with its own codec and teacher-force target audio only; fix text, reference, tokenization and boundaries; score full unsampled log probabilities using actual temporal/codebook factorization, proper prompt/padding masks and terminal token. Validate against an existing known teacher-forced cell. FTTS Base has no established actor-instruction input, so reference changes measure relative exemplar compatibility with content/timbre confounds. Do not equate a flow residual with likelihood or a softmax over prompts with emotion probabilities. Direct alternative: Qwen2-Audio-7B-Instruct audio interpretation with transcript-only, no-audio, wrong-audio and repeated-audio controls. Later frozen latent probes or analysis-by-synthesis are subhypotheses only after cheap signal survives; they need held-out text/source-take/reference/generator and leave-speaker-out tests. No full model port or large mandatory download to obtain speculative scores.

**Acceptance:** A reproducible scoring/interpretation result adds useful held-out localization beyond pitch/pause/rate, text-only and nuisance baselines, or the branch is explicitly rejected/deferred. Report uncalibrated compatibility honestly; no inference of inner emotion. Cap plumbing and compute. No production dependency, no promise to extract a unique emotion latent. Any later probe fitting or style search has a separate capped experiment and valid terms.

**Validation:** Audit target masking and factorization, length/group normalization, text/identity/codec shortcuts, prompt-label versus actual delivery, group leakage and null controls. Score blind human relevance at a fixed review budget. Exclude restricted model outputs/results from assumed-permitted FTTS improvement; current Drama3 IV(b) is a specific blocker to that reuse.

## N15 — Localize a measured voice defect across codec, implementation and model size

Lane: narration; priority: 2; prerequisites: N1; conditional: True.

**Why:** A native implementation bug, codec bottleneck and model limitation imply different fixes; source plausibility alone does not identify the cause.

**Scope:** Activate only for an observed unresolved defect. Compare original approved reference with destination-codec reconstruction; compare FTTS Qwen0.6B output with pinned upstream same-model inference on matched text/reference and effective settings. If warranted, add upstream Qwen1.7B Base as a distinct headroom arm, not a drop-in FTTS weight swap. Pin quantization/runtime/seed, record semantic differences in sampling and conditioning; distinguish model identity from implementation equality. Existing teacher-forced quantization/logit diagnostics can explain a measured mismatch, but KL divergence is not an emotion/accent metric. Do not recreate conformance infrastructure already present.

**Acceptance:** A bounded attribution result narrows the next fix or records unresolved confounds. Any engine-change proposal names the specific missing capability, minimal reproducer and perceptual acceptance criterion. A larger model sounding better does not prove the native wrapper is wrong. No engine port or production change here.

**Validation:** Matched-text/blind listening plus lexical checks and source-route evidence. Separate codec round-trip, implementation, quantization, model-size and reference effects; do not vary them all together. Verify no test/reference leakage. Record negative results and cost.

## N16 — Decide whether synthetic-Michael adaptation has earned a capped trial

Lane: narration; priority: 3; prerequisites: N7; conditional: True.

**Why:** Training and native engine changes are costly; first establish an actual inference-only ceiling and useful headroom.

**Scope:** Deferred research decision, not automatic training authority. Require failed cheaper inference/reference controls and a specific unmet delivery capability. Curate only approved FTTS synthetic Michael WAVs/transcripts with exact teacher/model/voice/seed/script hashes and upstream asset/output rights. Proposed first trial is 10-20 curated minutes and one capped run, not a sufficiency claim. Split whole text/source-take/reference before augmentation; no recursive student-output self-training. Consider official Qwen adaptation or Parler training only with architecture/control/format checks; fine-tuning does not create a missing emotion-instruction API. EmoSteer is a flow-model method, not a transferable Qwen knob; resolve its implementation/backbone/data terms separately. Exclude Drama3-to-FTTS distillation/critique supervision under assumed rights, and resolve Index restrictions separately. Activation requires an explicitly selected experiment, resource budget, separate production-code scope and actual failed-recovery producer dependency.

**Acceptance:** An evidence-based go/no-go decision with source rights, measurable target, rollback, cost cap and held-out acceptance. If approved later, compare inference-only cloning, teacher and adapted model on unfamiliar math/names/numbers and a longer passage. Preserve Michael and content before rating expression. Reject if improvement depends on a different narrator, training leakage, restrictive weights or a full port before listening evidence.

**Validation:** Audit licensing by exact artifact including code/weights/data/reference/teacher output; inspect grouped split and no-leakage lineage. Predeclare absolute/comparative gates and stop rules. No training result claimed from dataset curation or a closed decision task.
