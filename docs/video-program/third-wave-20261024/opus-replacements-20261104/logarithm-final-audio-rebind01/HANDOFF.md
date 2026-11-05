# Logarithm final audio rebind: helper handoff (87do.63)

LavenderPike, 2026-11-05, for BlueSalmon. Paths:
- `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- `S` = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/final-picture-repair01`

## Result

**`rebind_audio.py` is ready for root to run.** I ran only its read-only `--check` on `S`, and it passed against `integration.json` 8f7c4a9f. I did not run `--apply` or `--verify` on `S`, and it wrote nothing there.

- **Originals valid:** 51 of 51 files, all 15 cue-to-hold intervals, 15 helper markers, 4 shared helpers.
- **Final source:** 13 prediction holds and 2 reading holds found, plus the documented closing hold, kept separate.
- **Dry run through the copied tool:** 80 units, 15 directions, 15 dispositions, 364 sentence rows. All 80 cues' rows and samples equal prepared02.
- **Tests:** 86 of 86 passed on scratch copies, and two runs gave byte-equal results.

This is source and hash evidence only (see Limits).

## Root's run order

From the repository root, with `H` = `P/logarithm-final-audio-rebind01/rebind_audio.py`:

```
python3 -I -B H --check     # read-only; prints the hashes below
python3 -I -B H --apply     # root only
python3 -I -B H --verify    # read-only; run last, immediately before freezing assemble-inputs01.json
```

Exit codes: 0 pass, 1 refused, 2 usage, 3 originals valid but `integration.json` absent. On a refusal, `--apply` writes nothing unless the failure comes during its own writes.

**Run `--verify` last.** The copied tool hashes the map but not the source the map names, so a source edit after `--apply` would still pass assemble. `--verify` regenerates all outputs from the approved source and compares them byte for byte.

**A source change after `--apply` means a new rebind.** Move the outputs aside, restore `narration_units.json` from `narration_units-before-rebind01.json`, and rerun. The helper refuses until that is done.

## Predicted outputs

`--apply` should produce exactly these. They hold only for `integration.json` 8f7c4a9f and the current source. The spans hash also assumes the run is from the repository root.

| File | sha256 |
|---|---|
| `workspace/.../docs/narration-direction-map-oliver17.json` | `e9cc0e4bd497ebd5d67e788103a6a6d67a0ced64ffa54ad039f08e938cc21d93` |
| `workspace/.../narration_units.json` | `7cb6dbcd0b99d30b2d81cedc75d6046709935bc39ad2ad351439ed25228c2ed6` |
| `S/holds-bound01.json` | `ff1ece5acfa8fe5f2fff52efb683ed76ac1b010330c526d46b64cdc2bfaa808c` |
| `S/spans01.json` | `32d75a3e89c07a2791ef8a954f15769915a26c5d9d3ced5ec7eb2c5761c5b8c0` |

`--apply` also writes `S/narration_units-before-rebind01.json` (the old bytes, 1132a5a4) and `S/rebind-report01.json`. Nothing else changes. The test `apply_touches_only_the_named_files` proves that on a copy.

**The queued build is not affected.** None of these 6 paths is in `S/build-inputs01.json` (4,347 rows). The two `narration_units.json` rows there are original copies outside `S`, which the helper only reads. `integration.json` and `script.tsv` are frozen, and the helper only reads them. I checked this against the manifest as it stood at sealing.

## What the rebind changes

- **Holds with a new source hash:** 8 of 15, because their files changed. The other 7 rows keep their hashes.
- **Lines moved:** 0 of 15. All 15 helper markers also stay on their lines; only `draw.rs`'s hash changes.
- **Markers and intervals changed:** 2 of 15, the same two.

| Cue | Before | After |
|---|---|---|
| 05_5 | `predict(&kit, stage, v(OUT.centre[0], -2.75), 3.0)?;` | `predict(&kit, stage, v(IN.centre[0] + 2.2, 1.85), 3.0)?;` |
| 06_2 | `predict(&kit, stage, v(Z.centre[0], 2.55), 2.6)?;` | `predict(&kit, stage, v(read_x + 1.3, read_y - 1.4), 2.6)?;` |

Hash-only rows: 04_6, 05_1, 06_4, 11_3, 11_20, 12_5.

- **Map:** map16 is preserved. Map17 keeps its overrides, kinds, order and basis text, and adds a `predecessor` key.
- **Units:** only `direction_contract.map` changes.
- **Spans:** only `units_sha256`, the dispositions' line numbers and their evidence change. Unit rows and `boundary_review` are verbatim, so sentence samples, gaps and WAV hashes stay the same.
- **Reading basis:** the literals behind both reading rows are unchanged (11_20's theorem, rate and row verdicts; 10_4's fine print and floor). The basis text therefore stays true. A change there would be refused.
- **Other waits:** none added or removed (`other_waits_diff` is empty).

**Closing hold:** `what_it_forgets.rs:1311`, `kit.music_breath(stage, CLOSING_HOLD)?;`, after 12_21's say at line 1221 and before the finish at line 1313. The source constants match your record: 5.5 s hold, 1.5 s fade. The only code between it and the finish is that one fade. It is recorded in `holds-bound01.json` under `editorial_closing_hold`, and it is not a map row or a disposition. The helper leaves `native_timing_verified` false.

## Compatibility

| Item | Result |
|---|---|
| Growth reveal | **Correct.** Sentence index 5 is zero-based and is the sentence with the number. `reword` was out of scope; your fix (350ce26f) uses imported helpers. |
| `build01.sh:20` | Found in Mail 5228. **Fixed by root:** scoped rustfmt on the six changed files. |
| `assemble01.py:31` | Found in Mail 5228. **Your absolute launch satisfies it:** that interpreter reports the same path as `sys.executable`. The script path must be absolute too. Not launched. |
| `assemble-inputs01.json` | Build it **after** `--apply`, so it binds the new units, map17, `spans01.json` and `holds-bound01.json`. The guard refuses relative paths. |
| Tool tree | Use the copied tools in `S/workspace`. `coherent_narration.py` resolves the map against its own tree, and map17 exists only there. |
| Evidence path | Spans cite `S/holds-bound01.json` repo-relative. `assemble01.py` runs with cwd at the repository root, which matches. |

Also checked: the lock verifier printed PASS; `facts` from the prior debug binary is byte-equal to `oracle.py` (102 lines); no Rust test reads the units or maps. The facts comparison is old source and needs your build's own result.

## Tests

`checks/test_rebind.py` works on scratch copies of your final source and `integration.json`, with paths rewritten to each copy. Refusal cases also assert that the stage is unchanged.

| Group | Tests | What they prove |
|---|---|---|
| Gate | 21 | Refuses without approval, or on any unapproved, unlisted, extra, missing or frozen file; also a tampered original. |
| Holds and source | 13 | Refuses removed, extra, moved or multi-line predicts, code after predict, stray breaths, changed reading literals, missing or repeated says, changed `THINK`, predict order or bar fade. |
| Closing hold | 14 | Refuses undocumented, wrong cue, file, marker or seconds, a claimed row or timing, absent, doubled, or an extra step. |
| Rehearsal | 22 | Apply, then re-read every map row from source by separate code; units, spans and dispositions diffed against the originals; copied tool's `check` CLI passes; second apply refused; repeat rehearsal byte-equal. |
| Tamper | 9 | `--verify` catches edits to source, spans, map17, units, audit and preserved units, and passes again once restored. |
| Negative controls | 7 | Checks after the tool calls can fail: changed gap, changed review, evidence hash, missing disposition, empty reason, differing row, differing samples. |

## Limits

- **Nothing compiled, built, rendered or synthesized.** The helper has never been run against `S` with `--apply`.
- **`assemble` itself was not run,** even on a copy. In-process checks cover what it validates, but its write path and `verify` command are unexercised here.
- **Source markers are not timing.** They do not show that a hold plays as intended. The closing hold's timing and the changed 05_5 and 06_2 bars are unmeasured.
- **No listening, fine-sync, visual or independent AV acceptance.** Nothing was heard.
- **The `--check` dry run** leaves a small folder in the system temp directory, which the helper does not delete.

## Disclosure

This is not independent work. I wrote the stage09 audio preparation (including `prepare_audio.py`), the timing inspection, the ElevenLabs external voice repair that these copied tools carry, `render-preparation04` and `final-verification02`. I also raised O1, which led to the growth patch, and made earlier logarithm source repairs. Someone other than me should review `rebind_audio.py` if root wants independence on this step.

## Files

| File | What it is |
|---|---|
| `rebind_audio.py` | The helper: standard library plus the copied `coherent_narration.py` |
| `checks/test_rebind.py` | Tests: `python3 -I -B checks/test_rebind.py SCRATCH RESULTS`, neither path existing |
| `checks/outputs/test-results.json` | 86 results and the rehearsal summary |
| `checks/outputs/check-real-stage.json` | The `--check` output on `S` |
