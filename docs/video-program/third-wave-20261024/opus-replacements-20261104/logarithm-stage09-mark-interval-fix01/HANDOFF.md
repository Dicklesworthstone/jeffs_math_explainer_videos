# L1 fix: each phrase mark inside its own sentence (87do.63, Mail 4018)

LavenderPike, 2026-11-05. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

**Root's request (Mail 4018):**
- a narrow patch for L1 from my review `P/logarithm-stage09-audio-independent01`;
- the check start(k) ≤ mark < start(k+1) for the sentence holding the mark's exact byte, with the sentence-final edge consistent with the kit;
- positive and negative regressions, run on every real recognized take and on a synthetic shifted-onset counterexample;
- preserve every sealed bundle, and make no production edits and no assembly.

**Status: staged, not installed.** Bronze's sealed helper and every sealed bundle are unchanged. Root integrates the fix into a new versioned helper.

## The rule

**Which sentence holds a byte.** The last of the kit's sentences that opens at or before that byte. Sentences open at byte 0, then at the first non-space byte after each `.`, `?` or `!` that a space follows. This is the kit's `sentences()` split, measured in bytes.
- On all 80 stage09 cues, it equals the rule the films use to place a phrase in its sentence. `product_rule.rs` `phrase_seconds` handles 08_5, and `unreachable.rs`, `lifting_loops.rs`, `complex_turns.rs` and `doing_nothing.rs` (as `sentence_offsets`) use the same rule.
- All 33 anchors land in the same sentence both ways. 08_5 bytes 267 and 306 are in sentence 4 (M03).

**The interval.** start(k) ≤ mark < start(k+1). The last sentence ends at the recording's length, and that end is excluded. The kit's loader (`narration.rs` 937–945) already refuses a mark at or past the length, so the last edge is the kit's own rule.

**Units.** Seconds are compared as the kit computes them:
- an onset is samples / 24000 (`report_starts`);
- the length is frames / 24000;
- both are 64-bit floats.

So a mark exactly at its sentence's onset passes, and a mark exactly at the next onset is refused (M06, M07).

## The patch

`prepare_audio.patch` (+41/−4) applies to Bronze's sealed `prepare_audio.py` (`c6c854ce…`). Running `patch -p1` on a copy gives `staged/prepare_audio.py` (`2f080ab7…`).

| Where | Change |
|---|---|
| new `sentence_opens(text)` | the byte where each kit sentence opens |
| new `mark_outside_sentence(text, onsets_s, length_s, byte, seconds)` | `None`, or the reason a mark is not inside the sentence holding its byte |
| `resolved_points(entry, applied, frames24, text)` | new required `text` (the cue's script text). After resolving each mark it checks the mark against its sentence. Every mark outside is listed in one refusal, with its point id |
| `cmd_prepare` | passes `inputs["script"][cue]` (one line) |
| `cmd_timings` | computes each cue's onsets from the assembly's `report.json` the way the kit's `report_starts` does (running total of `out_samples + gap_after_samples` over `meta.rate`), and applies the same check to every mark it writes |

**Unchanged:** every other function, the report bytes, selection, answers, decode, spans and receipts (M02, M04). Each of the 8 target texts of Bronze's `mutation_check.py` still occurs exactly once (M02).

**Why both places:**
- `prepare` refuses early, naming the point ids root can answer.
- `timings` checks the exact onsets the kit will load. `phrase-marks-plan.json` is not hashed in `prepare-receipt.json` (O3), so `timings` is the last place a changed plan can be caught.

**The refusal root would see** (M05, with 08_5-download01's sentence 4 onset moved 0.1 s after its first mark):

```
anchor:08_5:b267:<08_5-download01 path> 'leaves a half turn': the mark at byte 267 (22.42 s) is not inside its
sentence 4, which runs from 22.52 s up to the onset of sentence 5 at 27.52 s; answer this mark (sample or omit_mark)
or correct the onset
```

## Evidence

- `checks/outputs/mark_interval_checks.txt`: 10/10, byte-equal over two runs.
- `checks/outputs/bronze_suite_on_staged.txt`: Bronze's own suite on the staged helper, 79/79, byte-equal over two runs.

| Check | What it shows |
|---|---|
| M01 | Bronze's bundle (`2bb8e280…`, 14 files) and my review (`3e7217bf…`, 3 files) verify; Bronze's helper is still `c6c854ce…` |
| M02 | The patch is exactly the diff base → staged, and `patch -p1` reproduces the staged file. Only the five functions above differ; all other top-level code is unchanged and in its place |
| M03 | The byte-to-sentence rule matches the films' rule and the pinned assembler's sentence count on all 80 cues and 33 anchors. The kit pin (`8072720d…`) holds `report_starts` and the at-or-past-length refusal. No stage09 cue has a non-ASCII byte, so a synthetic multi-byte text shows the byte rule where a character rule would pick the wrong sentence |
| M04 | **All real recognized takes.** All 162 inventoried recordings resolve from the recognizer with no refusal and exactly the base's starts, ends and marks (570 onsets, 66 marks). The report equals Bronze's sealed `report/` byte for byte. The smallest lead of a mark over its onset is 1.76 s (11_7 byte 174); the smallest room before the next onset or the end is 1.56 s (05_20 byte 276) |
| M05 | **Shifted-onset counterexample.** On 08_5-download01, moving sentence 4 to 0.1 s after the first mark is accepted by the base and refused by the staged helper, naming only `anchor:08_5:b267`. Moving sentence 5 to 25.0 s, before the second mark, is likewise refused, naming only `anchor:08_5:b306` |
| M06 | Answers that resolve it: omitting the mark, or a mark sample at the new onset, is accepted; one sample before the onset is refused. For sentence 5: one sample before the moved onset is accepted, exactly at it is refused. Bronze's D answer (08_5 s1 + 12000) and s1 at the end of its 'Here' segment are accepted |
| M07 | Edges on synthetic entries: last sentence, mark at the last sample is accepted and at the length is refused, which is what the kit loader does too; a mark at its own onset is accepted; sentence 0 at 0.0 s is accepted, and at the next onset is refused; the multi-byte mark is accepted |
| M08 | `cmd_prepare` in process (chapter 08, fixture selection and answers, zero-filled stand-ins of the decoded length). With sentence 4 moved, the base reaches its output step and the staged helper refuses with the single b267 line. With that mark omitted, or with recognizer values only, it reaches the output step. Nothing is written |
| M09 | `cmd_timings` in process on chapter 08 of synthetic noise assembled by the pinned assembler copy (`--require-directions`). With marks inside their sentences it writes the fragment. A plan with 08_5's first mark 0.1 s before sentence 4 (still in order and range) is written by the base and refused by the staged helper. The second mark exactly at sentence 5's onset gets the same result |
| M10 | **Bronze's measured onsets (Mail 4061).** His sealed timing inspection (`8c18a243…`) has 181 recognizer_early points (180 onsets, 1 mark). As sample answers at `leave_24k`, with or without his 08_5 dl02 s2 exception (167040), all 162 recordings are accepted. The closest mark to a sentence edge is 1.56 s (05_20 byte 276) |

**Bronze's suite on the staged helper.** `bronze_suite_on_staged.py` imports his sealed `run_checks.py` (`85ff6c42…`) unchanged and points only its `HELPER` constant at the staged file. Its transcript equals his sealed `run_checks-01.txt` except the helper hash line and my one header line. That covers:
- his full-film prepare with his answers, and timings with 32 marks;
- chapter 01 under the production binding;
- the chapter 04 replacement;
- every planted fault.

**On the real takes,** the fix refuses nothing today, with the recognizer's values (M04) or with Bronze's measured onsets (M10). It refuses only an onset or mark answer that would move a mark out of its sentence.

## For root, when integrating

1. **Make the new versioned helper** from Bronze's `c6c854ce…` plus `prepare_audio.patch`, or copy `staged/prepare_audio.py`, in a new folder.
2. **Set `HERE` in the new helper to its own folder.** `HERE` sets `TOOL`. `prepare` writes `TOOL` into the `then` timings command of `assemble-command.json`, and `guard_out` refuses OUT inside `HERE`. If it is left as it is, the printed timings command runs Bronze's sealed helper, which has no timings check.
3. **Rerunning Bronze's suites on the new helper.** `run_checks.py` and `mutation_check.py` name his folder; point `HELPER` (and `HERE` for `mutation_check.py`) at the new one. `bronze_suite_on_staged.py` shows the `HELPER` override.
4. **My K13 (independent review)** calls `resolved_points` with three arguments. It targets Bronze's sealed helper and stays valid there. Against the new helper it needs the text argument, and its "L1 reproduced" expectation fails by design; M05 replaces it.

## O3 (observation, not fixed)

`prepare` writes `phrase-marks-plan.json` but does not hash it into `prepare-receipt.json`, and `timings` reads it back unchecked.
- This patch's timings check refuses a changed mark that leaves its sentence.
- A mark changed within its own sentence would still pass.

Binding the plan means a hash in `prepare-receipt.json` that `timings` compares. That changes the receipt format, so I left it for root.

## Limits and what was not done

- No Rust compiled; the kit and the films are unchanged. The rule mirrors their source, as quoted in M03.
- No production decode, assembly, timing or selection. No recognition run. Nothing heard. Recognizer times and Bronze's energy measurements are estimates, not heard boundaries.
- **In-process seams.** The M08 and M09 calls replace only `make_out`: M08 stops at the output step, and M09 writes to scratch because the guard wants a repository path. Fixtures are labelled TEST FIXTURE.
- **Bronze's suite** runs his labelled stand-in decimator on real recordings, inside a scratch mirror with his test bindings and fixture answers, as he designed it. That is not production audio.
- `mutation_check.py` was not run; M02 shows its 8 targets are intact.
- Not independent: I found L1 and wrote this fix.

## Files

| File | What it is |
|---|---|
| `HANDOFF.md` | This note |
| `prepare_audio.patch` | The fix, against Bronze's `c6c854ce…` |
| `staged/prepare_audio.py` | Bronze's helper with the patch applied (`2f080ab7…`); `HERE` unchanged |
| `checks/mark_interval_checks.py` | M01–M10; read-only on the repository, writes only under a fresh scratch folder |
| `checks/bronze_suite_on_staged.py` | Bronze's sealed suite against the staged helper |
| `checks/outputs/mark_interval_checks.txt`, `checks/outputs/bronze_suite_on_staged.txt` | Their output at seal |

**Replay**, from the repository root:

```
python3 -I -B P/logarithm-stage09-mark-interval-fix01/checks/mark_interval_checks.py NEW_SCRATCH
python3 -I -B P/logarithm-stage09-mark-interval-fix01/checks/bronze_suite_on_staged.py NEW_SCRATCH2
```
