# complex-frederick-timing-bridge-repair01

WhiteSwan, for root Mail 4190. Frozen once sealed.

**Nothing here was compiled, rendered, synthesized, transcribed or heard. No production audio was decoded or changed.**
- ffmpeg ran only on the author's self-test noise fixtures.
- The real lane was only read: plans and ingests 01–05 were copied into scratch for the `report` comparison.
- No canonical, root, sealed, Bead or TODO file was edited.

## What this is

`frederick_timing_bridge.py` is a candidate. It is the sealed `complex-frederick-timing-bridge01` bridge (`c820376e`) plus the four fixes reviewed in `complex-frederick-timing-independent01` (patch `fe5c3096`, giving `7d689793`), with one repair on top.

**The repair (root's preservation finding).** The reviewed F3 fix re-converted each selected take inside `tempfile.TemporaryDirectory()`, which removed the conversions when it closed. Check v2 shows it: the reviewed patch refuses the planted case, so it did convert and compare the takes, yet its `TMPDIR` is empty afterwards.

The candidate keeps them:

1. **`prepare` checks first and writes nothing on a refusal.** It runs the decode receipt's row checks, the F1 interval checks and the disposition checks. Any refusal here leaves no `--out` (check f1-f4).
2. **It then creates `--out`.** `make_out` refuses an existing folder, as before.
3. **`retain_conversions` makes `--out/decode-verification/`** with an exclusive `mkdir()`. It converts every selected take again there, with the decode's own ffmpeg arguments, and compares the PCM with the decoded file.
4. **It writes `--out/decode-verification.json`.** For each cue this records the source, its hash, the kept file and its hash, the PCM hash, the decoded file and its hash, and `matches_decoded`. It also records the ffmpeg version and arguments.
   - **On a mismatch** it also writes `REFUSED.txt` and stops. Every conversion stays, including the one that exposed the change. The kept 01_4 file is the true conversion, not the altered decode (v3).
5. **`prepare-receipt.json` gains `decode_verification`:** the path of the record, its sha256 and the file count.
6. **`timings` refuses** when `decode-verification.json` no longer has the receipt's hash. It also refuses when any kept conversion under `--prepared/decode-verification/` is missing, changed or not matching (v5). It reads the kept files under `--prepared`, like `spans.json`, so a copied prepared folder checks its own copies.

Nothing in the candidate removes a file or uses a temporary folder (d3).

**Cost.** `prepare` writes one extra 24 kHz mono PCM16 file per selected take: the same bytes as the decode's `decoded/` folder, about 48 kB per second of audio, once per `prepare` run. It also runs ffmpeg once more per take.

## Files

| File | |
|---|---|
| `frederick_timing_bridge.py` | the candidate, `47069583` |
| `patch/bridge-repair01.diff` | sealed bridge → candidate (+84 −5) |
| `patch/delta-from-review01.diff` | the reviewed file `7d689793` → candidate (+49 −14): what changed since the review |
| `checks/bridge_selftest.py` | the author's self-test, copied unchanged (`ab7f53ff`); next to the candidate, it tests the candidate |
| `checks/repair_checks.py` | my checks |
| `checks/outputs/repair_checks.txt` | their transcript: 15 PASS, 0 FAIL |

## What the checks show (`checks/outputs/repair_checks.txt`)

- **d1:** both diffs apply to copies, and `git apply --check` passes on the sealed file in place.
- **d2:** against the reviewed file, only `check_decoded`, `cmd_prepare`, `cmd_timings` and the new `retain_conversions` differ.
  - `check_decoded` is byte-identical to the sealed bridge's again.
  - The only import removed is `tempfile`.
  - The F1, F2 and F4 code is exactly the code reviewed.
- **s1:** the author's 37/37 pass on the sealed bridge and on the candidate, with identical transcripts. In the candidate's world, `prepare` kept 6 conversions and the author's `timings` accepted them.
- **r1:** `report` on the real lane copy is byte-identical to the sealed bridge's: 62 takes of 31 cues.
- **v1:** `prepare` keeps one conversion per cue.
  - Each kept file is at its recorded hash with the decoded PCM; 6/6 are byte-identical files.
  - `spans.json` and `phrase-marks-plan.json` equal the reviewed patch's.
- **v2, v3:** the reviewed patch lost its conversions; the candidate keeps them, as described above.
- **v4:** an existing `decode-verification/` is refused (`FileExistsError`) with its planted content untouched, and an existing `--out` is refused with its files unchanged.
- **v5:** `timings` accepts a copy of a prepared folder. It refuses, writing nothing, when a kept conversion is changed by one byte or moved aside, or when the record is edited.
- **f1-f4:** the review's F1 (in `prepare` and in `timings`), F2 (downloads re-read) and F4 plants are still refused on the candidate.

## Replay

```
B=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-timing-bridge-repair01
(cd $B && shasum -a 256 -c SHA256SUMS)
python3 -I -B $B/checks/repair_checks.py NEW_SCRATCH_DIR
```

Run it from the repository root. It takes about 20 s, writes only under the new scratch folder, and expects `RESULT PASS, 0 failed`; it exits 3 if a pin moved.

## For integration (root's)

- **Location.** Run the candidate from this folder or from a new installed folder. `HERE` is the file's own folder, and `--out` may not be inside it.
- **Dependencies are unchanged.** The intake stays pinned to the sealed `edc154c6`. The bridge uses only that intake's `lane_state`, `record_problems`, `align`, `check_binding`, `direction_words` and constants. The intake candidate in `complex-frederick-intake-resolution01` leaves all of these byte-identical, so:
  - plans it writes read the same here;
  - re-pinning `DEPENDENCIES["intake"]` to it would change nothing;
  - its new receipt kinds (`resolution`, `superseded01`, `superseded01_evidence`) are bare names frozen at their plan hashes, which the F2 rule requires.
- **Run order.** Run `decode`, `prepare` and `timings` from the same file.
  - A `prepare` folder from the sealed bridge or from the reviewed patch has no `decode_verification`, so this `timings` refuses it.
  - No production `prepare` exists yet.

## Limits

- No listening, fine sync or acceptance is claimed. Recognizer times are still estimates.
- The self-test world is synthetic noise, and its selections and answers are fixtures.
- **Hash chains inside one prepared folder can be forged consistently,** as the review's w4 does for F4. A planted conversion, the record and the receipt rewritten together would pass `timings`. The record shows what `prepare` saw; it does not stop a deliberate in-folder forgery.
