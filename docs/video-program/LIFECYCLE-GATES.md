# Exact-cut checks in x and native Beads

`./x VIDEO status [--revision REVISION] [--voice VOICE] [--porcelain]` now reads the catalog, qualified checker policy, current receipts and actual file bytes. It lists every recorded revision by default, including different voices and historical publications. A voice mismatch never falls back to another cut. An idea without a crate can have status. Unrecorded evidence is blocked, not reconstructed from `.x` timestamps. `legacy-status` retains the old diagnostic output and labels it as historical hints.

`./x VIDEO gate-close --revision REVISION --issue ISSUE [--voice VOICE]` is the exact-cut task-closure path. It uses ordinary Beads statuses and the installed `br` CLI, never SQLite or direct JSONL editing. It does not generate media, approve publication, or qualify a narration method.

## Configure only qualified checks

[gate-policy.json](gate-policy.json) contains checker profiles and bindings between a task and one video/revision/voice. It does not duplicate task status. The initial production mapping is deliberately empty: historical files have no complete manifests and no newly qualified acoustic checkers. Enrolling an arbitrary passing receipt would fabricate authority. N11 owns method qualification; a technical prosody pass alone cannot establish Michael identity or accent.

A profile records:

```json
{
  "gate": "dry_voice",
  "checker_sha256": "SHA256_OF_CANONICAL_RECEIPT_CHECKER_OBJECT",
  "scope": "The precise property this qualified checker establishes",
  "qualification_ref": "repository/relative/qualification.md"
}
```

Compute the digest with `video_catalog.digest(receipt["checker"])` **after separately qualifying the complete checker recipe**, including executable, source/configuration files and actual acceptance semantics. Merely naming a profile does not qualify it. The required qualification file must exist. Changes to the checker recipe require a separately reviewed profile update and fresh receipt. Profiles are trusted repository configuration, not signed or adversary-proof attestations.

Each binding has `issue`, `video_id`, `revision_id`, `voice`, and a nonempty `checks` array of `{ "profile": "PROFILE_NAME", "receipt": "relative/receipt.json" }`. Each profile appears at most once in a binding; a task has exactly one binding. Bind only the checks needed to complete that particular task. Passing one task does not imply that the entire production cut passed every gate. Add the native `video:exact-cut` label to that task through `br label add ISSUE video:exact-cut`.

The production `.beads/policy.yaml` uses JSON syntax, a strict subset of YAML. `workflow.strict: true` is essential: the installed br 0.7.4 **lists gates but does not enforce them when strict is false**. Conditional rules require `exact_cut_receipts` only for labeled tasks entering `closed` from open, in_progress, blocked or deferred. No custom statuses or transition graph are introduced. Unlabeled ordinary work is unchanged. Keep this file JSON-compatible so the adapter can validate strict enforcement without introducing a second YAML parser.

## Closure sequence and portability

1. Acquire the retained `.x/producer.lock` advisory lock; per-video `x` producers and global `new`/`fetch` cooperate. Direct producers can use `uv run --no-project --with jsonschema python3 -I tools/video_lifecycle.py reserve -- COMMAND ...`.
2. Check the real native issue status, exact-cut label, strict policy and required transition. Revoke this provider's prior pass.
3. Validate every bound receipt against current catalog inputs, transitive artifact hashes, voice/revision, checker source/executable, output hashes and the qualified profile digest. Advisory/missing/stale/wrong-voice evidence blocks.
4. Append a normal Beads comment containing portable receipt paths and hashes, subject, qualification-file hashes, policy digest and adapter hash. That comment travels through native JSONL export/import.
5. Reload and rehash immediately before reporting the native pass, and again immediately before closing. Preserve the existing narration fingerprints around both scoring passes.
6. Confirm native closure. On a handled failure while the issue remains in its original state, revoke this provider's pass. A failure of some other native requirement is not bypassed.

Native gate metadata does not travel in supported JSONL sync. A clone needs the committed catalog, policy, receipts and their actual input/output files. Import ordinary issue data using native `br`; then rerun `gate-close` locally. Changed executable bytes or missing media block until new local evidence is produced. Do not copy the SQLite database or import a source machine's local verdict as qualification. No model or media file is downloaded by this adapter.

The producer reservation is cooperative, not a filesystem security boundary. Direct file writers, direct `br gate report`/`br close`, policy edits and bypass flags can evade this application contract. Native Beads does not monitor media bytes. A SIGKILL/power loss in the narrow pass-to-close interval can leave a native pass behind; always resume through `gate-close`, which revokes and revalidates first. This is not an atomic transaction spanning SQLite and media. A closed historical task remains closed if its files later change; `x status` then reports its receipt as blocked. Creating a new cut/task is distinct from rewriting history.

## Existing prosody path repaired

`x narrated` now checks both the checker exit status and recognizable report verdicts using `tools/check_prosody_report.py`. A crash, absent report, empty report or FAIL cannot write `narrated.ok`. FLAG/ADVISORY retain their original nonblocking semantics. No pitch bands, thresholds, audio samples, scoring fingerprints or listening criteria were weakened. This shell path was source-checked and its report decision executed in CLI tests; no full render was run.

## Verification, 2026-10-17

The local suite executes real subprocesses, byte hashing and native br against retained, explicitly synthetic trackers. It proves mechanism behavior, not acoustic acceptance. Run:

```sh
uv run --no-project --with jsonschema python3 -I tools/test_video_catalog.py
uv run --no-project --with jsonschema python3 -I tools/test_video_evidence.py
uv run --no-project --with jsonschema python3 -I tools/test_video_lifecycle.py
```

Coverage includes positive native closure, stale bytes with an old pass, changed catalog hashes, wrong voice/revision, leaving/reentering status, export/import with comments but no gate pass, changed clone media, fresh destination re-report, unqualified/advisory/missing evidence, strict=false, missing label, unaffected ordinary tasks, producer exclusion/inheritance, another required native gate and seven prosody execution cases. Native tests explicitly skip when br is absent; that is not native qualification. Existing tool-test loops discover these tests automatically. Fixtures are retained; a previous A3 test cleanup violated this repository's no-deletion instruction and has been removed.

This A4 implementation was freshly re-executed and reviewed by the same Codex agent. No independent reviewer, successful narration method, production gate pass, full render or publication is claimed. A5's live board and A6's full production lifecycle qualification remain separate work. This document serves the producer using `x`; retire/update it when the receipt contract is superseded.

### Bounded completion audit

Window: this A4 work block, 2026-10-17 04:23–04:40 UTC; no commits. [Exact dirty-byte verification](a4-verification.json) pins the tested code. Acceptance was rechecked against the original A4 description using the actual native CLI and current production catalog. Results: 15 catalog + 17 receipt + 12 lifecycle/report test methods passed, including 11 native tracker cases with no skips. Separate real `x` probes covered an idea without a crate, explicit Michael cut, wrong-voice rejection, legacy diagnostics and producer re-exec. `sh -n x` and `git diff --check` passed. Test fixtures and outputs remain in the recorded local directories.

Honesty inventory (same-agent audit, not independent certification):

- Tests/gates 1–6: No weakening or golden replacement (checked: complete new tests, receipt-test delta, x/CI diff). Synthetic media fixtures are explicitly scoped, while subprocess/hash/br behavior is real. Yes, feature and gate code were edited together because gate integration is the requested feature; no relaxed thresholds or bypasses were used. The native tests skip only when br is absent and said so in their declaration; zero were skipped in these local runs.
- Claims 7–12: No unrun command, auditory judgment, fixture-as-production claim, hidden nonzero result, stderr suppression in cited verification, unmet positive closure or rewritten acceptance (checked: transcripts, A4 original criteria and diff). Strict=false's surprising successful close and N13's rejected claim are recorded. A4 does not enroll an unqualified production checker or close N1/N13.
- Delegation 13–17: Not applicable; this was solo, no peer approvals or cross-model consensus. No independent review is claimed (checked: tool history).
- Metrics 18: The test denominator is all three named suites; the acoustic denominator is all four existing matched-text excerpts, with no preference inferred (checked: test outputs and automatic-triage.json). The one-pair listening budget is a proposal, not collected approval.
- Mirror 19: Yes, the earlier A3 tests used automatic TemporaryDirectory cleanup despite this repository's no-deletion instruction. Correction: retain fixtures with mkdtemp; no file deletion in this run. Also corrected an initial publication-key mistake after the real catalog status probe failed; the later `x` probes pass. No false green or completion was reported for either failed probe. Pattern RH-2 (proof-class inflation) is countered by explicit executed counts and synthetic scope; fixture preservation is now in code.
- Strongest proof 20: rerun `tools/test_video_lifecycle.py` with installed br: valid receipts close a real issue, changed bytes revoke an old pass and block closure, and a JSONL-imported fresh tracker cannot inherit that pass. Older unrelated sessions were not searched, respecting this task's session boundary.

Real-work disposition: HEALTHY for this bounded request. USER: exact-cut status and repaired report failure behavior; ENABLER: reserved receipt-to-native closure exercised by real CLI tests; PROCESS: these usage/verification notes and model follow-through. Running gates now consume the runtime policy. No new audible improvement is credited. The oldest important unresolved work is Michael identity/naturalness qualification (N1/N5/N7), which explains why model/voice work stays open. No new task was minted to hide unfinished A4 acceptance. Consumer/gate for the verification snapshot: owner-requested A4 closure; observed defect: mtime/voice ambiguity and native verdict portability; retire as current proof when these source hashes change, preserve as history.
