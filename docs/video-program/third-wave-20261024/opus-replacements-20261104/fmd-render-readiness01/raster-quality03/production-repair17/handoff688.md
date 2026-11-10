# Handoff 688: guarded CSD master and verification, mixed-host apportionment (CC0)

cc0, 2026-11-09, 23:19 UTC. Source only. **Nothing has run:** no SSH, build, render, launch, Beads change or commit. Root reviews and launches. D is `/data/projects/jmev-stroke-probe-20261106`; on the CSD, Q is `D/qualification01/production-repair17`.

| File | sha256 | |
|---|---|---|
| `apportionment-csd-master686.py` | `f1dde8735fac0bcd7075030a2e74c1cccdc0e295cbc1e57265ca69ceb680857b` | 442 lines; modes `inputs`, `run` |
| `apportionment-csd-verify687.py` | `8920ba67279a8fa6d614c71125c3c4ebbee13e4e0a003328aedceef5fb6ea43f` | 288 lines; modes `inputs`, `run` |
| `controller689.py` | `15135743773db70ab0207214c9824d9f416c8638d9100ceeb063d4e64da2b479` | 97 lines; pins 686 and 687 |

## A defect in master636 that root should know about

`master636.py:57-59` runs `master` without `--score`. The chapters' cue logs contain score stem rows, and `tools/narration_delivery.py:153-180` refuses a stem row that is not declared, so **master636 would be refused as written**. The earlier CSD master passes it (`before-ts2-124-trj-final-master54.py:149-150`). 686 passes `--score`. No master636 or verify638 job folder exists locally.

## Prerequisites

1. Copy 686, 687 and 689 to Q at the hashes above. They post-date export682, so 683 does not carry them.
2. import683 and all 13 final672 runs must succeed. 689 waits for each, with a 24 h limit per wait.

```
/usr/bin/python3 -I -B Q/controller689.py          # not run; or the four steps it makes:
/usr/bin/python3 -I -B Q/apportionment-csd-master686.py inputs
/usr/bin/python3 -I -B D/guard_csd02.py --phase native --job csd-apportionment-master686 --inputs Q/csd-apportionment-master686-inputs.json -- /usr/bin/python3 -I -B Q/apportionment-csd-master686.py run
    (then the same two steps for csd-apportionment-verify687 and apportionment-csd-verify687.py)
```

## How each chapter is resolved

- **01 and 02:** only through `csd-apportionment-import683-a1/import-result.json`, after `success`, with manifest `1c7c690b…` and 4,005 rows matched row by row to export682. The Mac prefix is derived from the export, not written in the source. Receipt, summary, input manifest and all six result files come from the path → physical records.
- **03 to 15:** `success` on each final672 receipt, the exact guarded command, policy and manifest hash, then the summary and every file at its recorded hash. `inputs` also requires 15 s of stable bytes on the receipts and summaries.

## What had to change for the mixed host

| Point | master636 / verify638 | 686 / 687 |
|---|---|---|
| Mac paths in the 01/02 tables | not applicable | The kit opens these paths. `final/` holds a copy with **the path column only** rebased, which must equal timeline666's table byte for byte (cue log: its rows plus the exact stem rows). Imported files are untouched. |
| Encoder settings | raw equal across all 15 | raw equal within each host; across hosts, equal to x264's reference with only support671's two exemptions (build banner, auto thread counts). Each group's tags bound to its own ffmpeg. |
| Chapter checks for imported 01/02 | not applicable | support671's full `chapter()` runs here too: this ffprobe, this `-xerror` decode, level 5.2, packets, AAC, sample count. |
| Tools | Mac ffmpeg | `/usr/bin` ffmpeg, ffprobe and python, with their libraries bound, cores 0-3 as in final672. |
| Sound-review key in the verification record | verify638's own name | renamed `audio_heard`, still false. No other field changes meaning. |

**Kept unchanged:** the kit's bit-for-bit replay of every cue log against each captured soundtrack (686 also requires all 15 replay lines in order), the score QA, the delivery contract, frame totals, per-cue placement from the 24 kHz recordings, script order, all caption limits, stream checks, the full decode, per-frame equality of master against all 15 chapters, loudness and true peak. `stamp`, `captions` and `frame_digests` are AST-identical to verify638. The guard is `guard_csd02` with policy 10/8/4/56/20/10; `inputs` and `run` refuse duplicates and there is no retry.

Recordings stay the exact source files. Of the 93 generation receipts, 93 read `Frederick Surrey - Smooth and Velvety` (read on the Mac). 686 does not read voice labels itself; identity rests on the hash chain and the kit's `coherent.verify`, which runs again under `master` and `captions`.

## Evidence I could not get, for root to supply or accept

1. **Cross-host mixing: the main open risk.** 01/02 were captured on the arm Mac, and the replay runs on the CSD. If the two mixers differ by one sample the master stops, and that is the right result; nothing here relaxes it. There is a cheap test before launch: Mac preview569 used the same narration, stems and timeline at 960x540. Compare its soundtracks with `result.files` in the CSD `final672-summary.json`:
   - `08_signposts`: `4e7ad48ecf33171acc3423cb9344cb48254a28a57434f231ade8af68ee6b47b3`
   - `13_population`: `72eb33a9323f4b68a95431523035fda936f450d752af27b5bad02cbec0d6b509`
   - `15_recap`: `ff9f83e4174c8d00e6ae593026bdda3b6609344b291682e71ed2991a14f58421`

   The soundtrack comes from the cue log, so resolution should not matter, but that is reasoning, not a test. If they differ, I would expect 686 to stop on 01/02, and the decision goes to root before spending the run.
2. **Layouts for 01/02.** The imported layout tables are empty. `comparison666.json` must record `e3b0c442…` for both, or `inputs` stops. I cannot see that file.
3. **Terminal states.** The import683 result, finals 05 to 15 and 673's progress are all unseen.

## What was checked

`compile350` and the banned-string scan on all three files; AST checks that every support671 name used exists and no name is undefined; a statement-level diff of verify638 `main` against 687 `run`, whose differences are the ones above plus paths and names.

Read-only exercise of 686's functions on the real local 01/02 files: `local_chapter`, `check_tables`, `rebase_table` (cue log round trip exact, three malformed rows refused) and `options`. The stand-in timeline666 tables were derived from timeline567 with 666's own path replacement. `materialize` ran on synthetic scratch files only.

The kit source I read has the hashes in the export closure (`master.rs`, `cli.rs`, `captions.rs`, `narration_delivery.py`, `coherent_narration.py`).

## Limits

- **Never executed on the CSD.** `run`, `inputs`, the guard path, `freeze`, `survey` against real records and 689 are untested. Any defect should stop the run, but costs new job names.
- **First use of these kit paths on the CSD.** final672 used only the `render` consumer. `cue_log`, `directed_segments`, the master join and caption generation run here for the first time.
- **Framehash form.** verify638's arguments and its `duration == 1` assertion are kept as written. They have never run on this ffmpeg. A wrong expectation would stop the job only after the master's decode.
- **Hard links** need the import, finals and job on one filesystem; all are under `D/jobs`, and a failure stops the run before any output.
- **Size and time are not measured.** I estimate 1.5 to 2.5 GiB of growth for 686 under the 4 GiB phase cap, including a 645 MB master WAV. 687 makes two full 4K decodes on four cores, with 14,400 s per step.
- **Not reviewed:** picture, motion, sound, fine sync, or perceptual continuity across the join between the two encoders. The frame comparison shows the join kept every decoded frame, nothing more.

No audio heard or checked. This accepts no picture, motion, sound, sync, AV or release result and authorises nothing; root owns launches.
