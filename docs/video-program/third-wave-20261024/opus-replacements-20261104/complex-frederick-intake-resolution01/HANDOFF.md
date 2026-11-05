# complex-frederick-intake-resolution01

WhiteSwan, for root Mail 4190 (implementing the 05_1 contract from `complex-frederick-timing-independent01`, Mail 4042). Frozen once sealed.

**Nothing here was compiled, rendered, synthesized, transcribed or heard. No production file was written.**
- The real take folder and lane were only read.
- The real 05_1 receipts and takes were copied into scratch.
- The resolution used there is a labelled fixture; **root authors the real one.**
- No fw, guard launch, Bead, TODO or canonical edit.

## What this is

`frederick_intake.py` is a candidate. It is the sealed intake (`complex-frederick-audio-preparation01`, `edc154c6`) plus one rule, `check_resolution`, called from `check_receipt`.

**Without a resolution file, behaviour is unchanged.**
- The sealed 54 checks pass with an identical transcript (s1).
- On root's live take folder, `status` prints the same output as the sealed intake (r1).
- 05_1 stays refused: "2 submission receipts bind '05_1-request01.json'; expected one".

**With one, the intake can plan 05_1's two withheld takes,** on the retry `05_1-submission02.json`. The failed first attempt `05_1-submission01.json` is set aside but stays in the folder, frozen with its evidence.

## The contract

The file is `CUE-submission-resolutionNN.json` in the take folder. The new name pattern does not match `CUE-submissionNN.json`, so the uniqueness rule's own scan never counts it.

A resolution applies to a downloads receipt whose request it names. Then all of the following must hold, or that cue is refused with a message naming the file:

1. **One per cue.** Two or more resolution files are refused as ambiguous (t05). An unreadable resolution refuses the cue.
2. **Exact form.** Exactly the keys `schema`, `cue`, `request`, `downloads`, `selected`, `superseded`, `by`, `basis`, `at` and `listening`.
   - `schema` is `frederick-submission-resolution-v1`.
   - `by` and `basis` are non-empty.
   - `at` is an ISO time with its zone.
   - `listening` is false (t15).
3. **This request and this downloads receipt, at their current hashes.** These are the hashes of the very bytes the intake checked. A later download set needs a new resolution (t03a, t03b, t10).
4. **`selected` and `superseded` are exactly the submission receipts that bind the request,** two or more, each at its current hash.
   - An unnamed third binder is refused (t04).
   - So is a resolution where only one receipt binds (t06).
   - So are a name used twice, the selected receipt also superseded, and a receipt of another cue or of another request (t07a–d, t03c, t03d).
5. **Evidence.** Each superseded entry names a `CUE-submission-observationNN.json` at its hash. Its `supersedes` must be that receipt's name, or begin with the name followed by a space.
   - A missing file, an edited file, a file of another kind and an observation of another receipt are refused (t03e, t03f, t08a–d).
   - So is a name that only shares the prefix, as in `05_1-submission01.json.old` (t08c).
6. **Selected state.** `submitted: true` and an integer `generation_requests` ≥ 1 (not a bool) (t09a, t09b).
   - This is how root supplies what `05_1-submission02.json` lacks.
   - Anything the selected receipt states itself must agree: `submitted`, `submitted_once` (which means 1), `generation_requests` and `cue` (t09c, t13).
   - The take count must then be 2 × `generation_requests` (t09d).
7. **Times in order.** selected ≤ downloads ≤ resolution, and each superseded attempt ≤ its evidence ≤ the resolution (t14a–d).

**Effect.** The cue's takes rest on:
- `submission` (the selected receipt);
- `resolution`;
- `superseded01` and `superseded01_evidence` (and so on, one pair per superseded receipt).

`frozen-inputs.json` freezes every one of these, so the guard hashes them before and after fw. All are bare names at their plan hashes, which the bridge candidate's F2 rule requires (t02). Other cues' plan rows and frozen entries are byte-identical with and without the file (t12).

An observation alone changes nothing (t11). No free text decides which submission counts.

## For 05_1: what root would write

Run the intake's `status` first: the hashes must still be current. These are today's, checked by this bundle's pins:

```json
{
 "schema": "frederick-submission-resolution-v1",
 "cue": "05_1",
 "request": {"name": "05_1-request01.json", "sha256": "cf1623eda56ddab6c227eb2e2c77f885bb0807fa8ea1b8aaa3f0afd70b90911d"},
 "downloads": {"name": "05_1-downloads01.json", "sha256": "87191b79effc8c012c292477428b8af8d3d992b2507555a253f105447e3f231e"},
 "selected": {"name": "05_1-submission02.json", "sha256": "4c88d6d918d0e9ee2fda8dc914f8a99acc53e087e296fa3d8e3d31c9fc159dbf",
              "submitted": true, "generation_requests": 1},
 "superseded": [{"name": "05_1-submission01.json", "sha256": "77859fad33bcf30036d8af74daa95d709f8c90b7f838f30ebfd1cff47e9418ed",
                 "evidence": {"name": "05_1-submission-observation01.json",
                              "sha256": "138bc84b1802281c567034864735d4359d3ccc64d20c370789db82bdc49d8106"}}],
 "by": "BlueSalmon",
 "basis": "<root's reason, e.g. the CDP timeout, provider history and the single retry>",
 "at": "<UTC now, ISO with +00:00>",
 "listening": false
}
```

**The times already in order:** submission02 05:02:26 ≤ downloads 05:03:00, and submission01 04:57:46 ≤ observation 04:59:49. The resolution's `at` must be later than both.

**`generation_requests: 1`** is root's statement that the retry was clicked once (submission02 says "Retry clicked once successfully"); the two takes in downloads01 match it. Root should state it only if root stands behind it.

The real data was staged in scratch to show this works (g1):
- **What was copied:** the 7 real 05_1 files.
  - The downloads copy differs only in its two take paths, which point at the copies.
  - Every other copy keeps its real hash.
- **Without a resolution,** both intakes refuse 05_1.
- **With a fixture resolution of exactly this form** (by "fixture …", basis "staging test …", at 06:00:00Z):
  - the sealed intake still refuses 05_1;
  - the candidate plans both real takes (`a76aa56c`, `cf33a532`) on submission02, with submission01 as superseded01;
  - root's guard `check_inputs` accepts the manifest.
- **The real folder:** still at its pins, with no resolution file in it (g2).

## Files

| File | |
|---|---|
| `frederick_intake.py` | the candidate, `8760be91` |
| `patch/intake-resolution01.diff` | sealed intake → candidate (+145 −1) |
| `checks/intake_selftest.py` | the sealed self-test, copied unchanged (`0fca8e16`); next to the candidate, it tests the candidate |
| `checks/resolution_checks.py` | my checks |
| `checks/outputs/resolution_checks.txt` | their transcript: 40 PASS, 0 FAIL |

**Scope (d2).** Only three functions differ:
- `check_receipt`: every sealed line is kept except `if len(submissions) != 1:`, which becomes `elif` after the 6 added lines;
- `check_resolution` and `instant`, which are new.

NAMES gains the resolution and observation patterns, and `RESOLUTION_SCHEMA` and `RESOLUTION_KEYS` are new. Everything else, including the plan writer and `frozen-inputs.json`, is the sealed code.

## Replay

```
I=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-intake-resolution01
(cd $I && shasum -a 256 -c SHA256SUMS)
python3 -I -B $I/checks/resolution_checks.py NEW_SCRATCH_DIR
```

Run it from the repository root. It takes about 95 s, writes only under the new scratch folder, and expects `RESULT PASS, 0 failed`.

- It exits 3 if a pin moved. That includes the 7 real 05_1 files, and a resolution file appearing in the real take folder.
- Sections 5 and 6 read the live take folder and lane. Once root writes the real resolution, the run exits 3 at the pins by design, since those sections assume the folder before it. So replay before root authors it.

## For integration (root's)

1. **Install the candidate** in a new folder, or run it from here. `HERE` is the file's folder, and the lane may not be inside it.
2. **Author `05_1-submission-resolution01.json`** in the take folder as above.
3. **Run `status`.** 05_1 should leave `refused` and appear in `eligible_wavs`.
4. **Then run `plan`.**
   - The next plan holds 05_1 download01 and download02.
   - Earlier plans and ingests are read as before, since `lane_state` is unchanged.

**The bridge** pins the sealed intake. It reads such plans unchanged, because it uses none of the changed names (see the bridge candidate's HANDOFF).

## Limits

- **The rule checks consistency and hashes, not truth.** Whether submission02 really produced these two takes, and submission01 none, is root's statement in `basis`, backed by root's provider-history observation. The rule only makes that statement explicit, hash-bound and the only thing that can decide.
- **Two separate requests of one cue are not modelled.** One resolution per cue is allowed. A cue whose second request also had a failed attempt would need its first resolution moved aside first.
- **The fixtures are synthetic,** and the staging resolution is a labelled fixture.
