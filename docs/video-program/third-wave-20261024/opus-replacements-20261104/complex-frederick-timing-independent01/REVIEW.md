# complex-frederick-timing-independent01

WhiteSwan, for root Mail 4036 (independent review of the sealed `complex-frederick-timing-bridge01`, SHA256SUMS `f31fd5c5`) and Mail 4042 (a contract for the 05_1 submission). Frozen once sealed.

**Nothing here was compiled, rendered, synthesized, transcribed or heard. No production audio was decoded or changed.**
- The bridge's own `report` hashed the real takes, as it always does.
- ffmpeg ran only on synthetic WAVs and on the self-test's noise fixtures.
- The one real-lane decode used a stub that writes synthetic PCM, and its receipt names the stub.
- No canonical, root, sealed, Bead or TODO file was edited.

**Independence.** I did not write the bridge, the intake or the logarithm helper it imports. I did write review01 of the coherent v2 checks and the regression tests root installed (`19487bd0`, `cd4d8dfa`). So where this review leans on coherent's v2 rules, it is not independent.

## Verdict

**The bridge's core contract holds on today's lane, but four gaps should close before root's first `prepare`.** None of them is hit by today's data.

**What holds:**
- `report` reads all 5 ingests (62 takes of 31 cues).
- The decode is ffmpeg conversion only: ⌈n/2⌉ frames and no sample shift.
- The assembled cue is the decoded take, sample for sample.
- Sentence starts and anchor bytes follow the film's own rules for every cue.
- The real request → downloads → submission chain is intact for every take.

**The gaps:**
- F1 is reachable by an ordinary root action: a `sample` answer.
- F2–F4 each need a file edit that nothing checks.

`patch/bridge-fixes.diff` (+49 −5, one file) closes all four. It changes no `report` byte, and the author's 37 checks still pass. Apply it in a new bundle, never in the sealed folder.

| # | Severity | Sealed bridge lines | Failure | Planted proof |
|---|---|---|---|---|
| F1 | Medium | `resolve` 999–1005, `cmd_timings` 1292–1298 | A mark can sit before its own sentence's onset, or after the next one. Only the order between marks and the WAV range are checked, and the kit checks no more. | i3, i4, i5, w1, w4 |
| F2 | Low–Medium | `recognitions` 613–618 and 657, `cmd_decode` 1037–1042 | `plan.json`, the only source of each take's request and downloads names, is never compared with its `plan` entry in the ingest-bound `frozen-inputs.json`. Receipt names need not be bare, and the downloads receipt is never re-read. | b1, b2, b3, w5 |
| F3 | Low–Medium | `check_decoded` 1115–1138 | The decoded PCM is checked only against its own receipt row. A changed file with a re-hashed row goes into the spans, and timings then compares the assembly with that same file. | w2 |
| F4 | Low–Medium | `cmd_prepare` 1219–1229, `cmd_timings` 1242 | `phrase-marks-plan.json`, whose marks become `phrase-timings.json`, is not hashed in `prepare-receipt.json`, and timings reads it unverified. | w3 |

## F1: the sentence/phrase interval (the bridge's copy of L1)

**Rule.** For a mark in sentence k of a take with onsets s₀ = 0 < s₁ < … < sₙ₋₁ and length N (24 kHz samples), require sₖ ≤ mark < sₖ₊₁, with sₙ = N. `resolve` and `timings` check neither bound.

**Why it matters now:**
- Root will correct onsets with `sample` answers. Bronze's Mail 4061 measured logarithm's recognizer onsets as early by 0.2–0.73 s and recommends exactly those answers.
- Today the tightest complex mark is **02_6 download02, byte 139: 0.540 s after sentence 2's onset** (i2). A correction of that size on 02_6 s2 puts the mark before its own sentence, and nothing refuses.
- The tightest room before a next onset is 1.260 s (02_1 download02, byte 91).
- Nothing outside the bridge catches it:
  - the kit's loader (`narration.rs:924–949`) accepts all three planted mark sets;
  - the pinned assembler's dry run accepts the planted prepare (w1).

**Planted cases** (the sealed bridge accepts each; the patched one refuses each, naming the mark, sentence and samples):

| Check | Case |
|---|---|
| i3 | 02_6 dl02 sentence-2 onset answered 240 samples after the byte-139 mark |
| i4 | the 02_1 dl02 byte-91 mark answered 240 samples after sentence 2's onset |
| i5 | the 02_6 dl02 byte-139 mark answered 240 samples before its own onset |
| w1 | the whole `prepare` in the self-test world: 01_2 sentence 1 answered at 120000, past the byte-65 mark at 108000 |
| w4 | `timings` with the mark moved to 70000, before sentence 1 at 79200 (the receipt hash forged so the F4 check passes) |

**Same rule as Lavender's L1, adapted to complex.** It is the rule Lavender's L1 enforces in the logarithm helper (Mail 4018), which Bronze's Mail 4151 says to integrate. I did not touch that helper. Two differences:
- **Sentences by character:** complex's `draw.rs` places sentences by character (`sentence_offsets`, `Spoken::at`). The bridge's `sentence` field already follows that rule for all 93 anchors (x2), and its sentence starts equal the kit-rule starts for all 105 cues (x1).
- **Integer samples:** the patch compares integer samples throughout. A mark on a sentence's first word reuses the onset sample exactly, so Bronze's 1 µs rounding case cannot arise.

**Effect on real takes:** with the patch, all 62 real takes resolve exactly as before (i1); the only change is a `sentence` field on each mark.

## F2: the producer chain is checked once, at plan time

**How the request path is built today.** The v2 receipt's `request` is the take's folder joined to the request name from `plan.json` (`:657`). The bridge loads `plan.json` through the intake's `lane_state` and checks only that `frozen-inputs.json` still matches the ingest (`:617`). Yet the guard hashed `plan.json` (role `plan`) and every receipt (role `receipt`) into that file before and after recognition.

**What the planted cases show:**
- **b1:** `plan.json` alone is edited to point 01_1-download01 at a request file outside the take folder, with the same cue, text and voice. The sealed `report` records it.
- **b2:** a consistent forgery (`plan.json`, its frozen entry and the ingest hash all rewritten) also passes the sealed report.
- **b3:** the sealed `decode` writes that request into the v2 receipt (stubbed ffmpeg), and the pinned assembler's `unit_spans`, from the bridge's own workspace, accepts it as Frederick.
- **w5:** a downloads receipt rewritten to name another request does not stop the sealed `decode`.

**Today's chain is intact (c1):**
- 5/5 plans equal their frozen entry.
- Every receipt of the 62 takes is a bare name in the take folder, current and frozen at the plan's hash.
- Every request is current text and voice with `submitted: false`, as prepared.
- Every downloads receipt lists its take from that request.
- Exactly one submission binds each request.

**Fix:**
- `recognitions` requires `plan.json` to equal its frozen entry, and each receipt name to be bare and frozen at the plan's hash.
- `decode` re-reads the downloads receipt: its hash, cue and request, and that it lists the take.
- Positive control (b4): the patched decode accepts the real chapter 01 chain, and `unit_spans` accepts its receipt.

## F3 and F4: two files read without their binding

**F3.** The fix re-converts each selected source with the same arguments into a temporary folder and requires identical PCM. That adds one ffmpeg run per take to `prepare`; ffmpeg 8.1.2's output is deterministic for these arguments (s1, w2).

**F4.** `prepare-receipt.json` gains `phrase_marks_plan_sha256`, and `timings` refuses a mismatch. This is the binding delta Bronze's 4151 reviewed for the logarithm helper. As there, `prepare` writes the plan before the receipt that hashes it.

## Notes (no change needed)

- **N1. ffmpeg `-n` exits 0 when the target exists** (f3). It leaves the file untouched and prints "already exists", so `check=True` cannot see a skipped write. The bridge never meets this, because every decode target is a new file in a new folder. Anyone reusing these arguments should not rely on the exit code.
- **N2. The ⌈n/2⌉ rule holds from 64 input samples.** Below that, ffmpeg returns fewer frames (1→0, 7→0, 63→31), which `expected_frames` refuses. The shortest real take has 272,640 frames (f1). Impulses keep their position to within half a sample, so `sample_24k = round(t × 24000)` addresses the decoded file (f2).
- **N3. The self-test leaves its world changed.** Its last case (n22) appends a space to `01_2-request01.json`. Reusing that world needs the space removed (w0).
- **N4. Answers bind the source side only.** `check_answers` matches by question id (`:923–942`), and `content:CUE:FILE` does not change when a supplement adds differences. So a `proceed` written against a report without `--supplement` still applies to the longer list. Write answers against the same `report` flags that `prepare` will use.
- **N5. A `sample` answer must be > 0.** A byte-0 anchor therefore cannot be answered at 0, but `omit_mark` gives the same film time. Today no anchor is at byte 0.
- **N6. Fragments stay previews.** A complete `phrase-timings.json` needs one 105-cue `prepare` (the author's Q1). Answers are keyed by point id, so they carry over.
- **Supplement pairing is safe by construction** (`:769`): both point lists come from the same text and the same anchors, in the same order.

## 05_1: a supersession record for the intake (Mail 4042)

**Facts (k1, read-only):**
- `05_1-submission01.json` (`77859fad`, the CDP-timeout attempt, `submitted_once: true`) and `05_1-submission02.json` (`4c88d6d9`, the retry) both bind `05_1-request01.json` (`cf1623ed`) by `request_sha256`. So plans 04 and 05 refuse 05_1 ("2 submission receipts bind … expected one").
- **submission02 has no `submitted`, `submitted_once` or `generation_requests`.** Even alone, the sealed intake refuses it: "not marked submitted" and "generation_requests None is not a positive count".
- `05_1-submission-observation01.json` (`138bc84b`) names submission01 in `supersedes`, but no rule reads it.
- `05_1-downloads01.json` (`87191b79`) lists 2 takes.

**Proposed contract** (not implemented; a separate candidate after root reviews it). It keeps the uniqueness rule and deletes nothing.

A new file `CUE-submission-resolutionNN.json` in the take folder, matched by a new intake name pattern `(\d\d_\d+)-submission-resolution(\d\d)\.json`, which the `submission` pattern does not match:

```json
{"schema": "frederick-submission-resolution-v1", "cue": "05_1",
 "request":   {"name": "05_1-request01.json",   "sha256": "<current>"},
 "downloads": {"name": "05_1-downloads01.json", "sha256": "<current>"},
 "selected":  {"name": "05_1-submission02.json", "sha256": "<current>", "submitted": true, "generation_requests": 1},
 "superseded": [{"name": "05_1-submission01.json", "sha256": "<current>",
                 "evidence": {"name": "05_1-submission-observation01.json", "sha256": "<current>"}}],
 "by": "BlueSalmon", "basis": "<why>", "at": "<utc>", "listening": false}
```

`check_receipt` applies it only when one exists for the cue:

1. **At most one per cue.** Two or more refuse the cue as ambiguous.
2. **Form.** The schema is exact, the cue matches, `by` and `basis` are non-empty, and `listening` is false.
3. **This request and this download set.** `request` and `downloads` equal the downloads receipt's request name and this receipt's name, each at its current hash. A later download set needs a new resolution.
4. **Named receipts.** `selected` and each `superseded` are distinct submission-pattern files of this cue, at their stated hashes, and each binds the request.
5. **Exact cover.** `{selected} ∪ superseded` must equal every submission receipt of the cue that binds the request:
   - an unnamed extra binder refuses;
   - a resolution where only one receipt binds refuses, since there is nothing to resolve.
6. **Evidence.** Each superseded entry names a non-submission file of the cue at its hash, whose `supersedes` begins with that receipt's name.
7. **Selected state.**
   - The resolution states `submitted: true` and an integer `generation_requests` ≥ 1.
   - Any state the selected receipt itself carries must agree (`submitted_once` means 1).
   - The usual count rule then applies: takes = 2 × `generation_requests`.
   - For 05_1, this is how root's hash-bound statement supplies what submission02 lacks.
8. **Order (recommended).** The selected `at` ≤ the downloads `at`, and each superseded `at` ≤ its evidence `at`. Today: 05:02:26 ≤ 05:03:00, and 04:57:46 ≤ 04:59:49.
9. **Effect.** The cue proceeds as with one submission. `rests` gains `submission` (the selected receipt) and `resolution`, so the guard freezes both. The superseded receipt and its evidence are listed for audit and stay in the folder.
10. **Free text never decides.** Without a resolution, today's rule stands.

**Tests for the candidate** (on the intake self-test's fixture writer):

| Test | Expected |
|---|---|
| T1: two binders, no resolution | refused, today's message (ambiguous success) |
| T2: the 05_1 shape with a valid resolution | ready; only submission01 excluded; the plan names the selected receipt and the resolution |
| T3: each named hash wrong in turn (request, downloads, selected, superseded, evidence) | refused, naming the file |
| T4: a third binder not named | refused |
| T5: two resolution files | refused |
| T6: a resolution where only one receipt binds | refused |
| T7: selected also superseded; duplicate names; a submission of another cue or of another request | refused |
| T8: superseded with missing evidence, or evidence that supersedes another receipt | refused |
| T9: selected state missing, contradicting the receipt, or a take count ≠ 2 × n | refused |
| T10: a resolution for another downloads receipt | refused for this one |
| T11: the observation file alone | still refused |
| T12: other cues' plan entries | byte-identical with and without the resolution file |
| T13: a selected receipt with its own complete state and an agreeing resolution | ready |

## Replay

```
O=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-timing-independent01
(cd $O && shasum -a 256 -c SHA256SUMS)
python3 -I -B $O/checks/independent_checks.py NEW_SCRATCH_DIR
```

- It runs from the repository root in about 30 s, writes only under `NEW_SCRATCH_DIR`, and expects `RESULT PASS, 0 failed`. That is `checks/outputs/independent_checks.txt`: 27 checks, byte-identical over two runs with different scratch names.
- It exits 3 if any pinned input moved. That covers the bridge seal, 9 tools and sources, and the lane's plan and ingest files 01–05.
- Later ingests are additive and not read. Section 9 and section 11 read the live take folder, so a new receipt for one of those cues changes their lines.

**Sections:**
1. Pins.
2. The patch: it applies to a scratch copy and to the sealed file (`--check`).
3. The author's self-test on the sealed and patched bridge.
4. The real-lane `report` (byte-identical under the patch).
5. F1 on real rows.
6. F2 on frozen lane copies.
7. Planted cases in the self-test world.
8. The decode contract.
9. The real chain.
10. The source side.
11. The 05_1 facts.
12. Hygiene.

## Limits

- **Source and synthetic checks only.** No Rust was compiled: the kit's loader is re-ported, as is `draw.rs`'s sentence rule (its pinned lines are asserted).
- **Nothing was heard or measured in the audio.** Whether any real onset is early is not measured here, for complex; the logarithm figures are Bronze's (Mail 4061).
- **Some inputs are fixtures.** The fixture selections and answers are labelled as planted cases; they are not root's.
- **The patch is a tested reference, not an integration.** Root or Lavender may prefer to port Lavender's own L1 and binding code.
- **The 05_1 contract is not implemented.**
