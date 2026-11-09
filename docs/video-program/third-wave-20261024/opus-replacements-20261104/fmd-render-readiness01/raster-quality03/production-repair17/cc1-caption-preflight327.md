# CC1 caption preflight recipe 327

Author: CC1. Date: 2026-11-08. A recipe only: nothing here has run.

I launched no job, edited no helper, source, TODO or Beads, and reached no remote host. Running it is root's.

**Scope.** This is a guarded native run of binary 268's `captions all` on timeline269's complete schedules and rebind269's audio. It exports actual caption files.

**What it is not.** It is not the final master, and not caption acceptance, listening, fine sync or AV acceptance.

**Why.** It answers 322 F1 without changing 318: the kit's own caption build and validation get run before 318 spends anything.

Paths use 318's names:
- D = `/data/projects/jmev-stroke-probe-20261106`
- Q = `D/qualification01/production-repair17`
- REPO07 = `D/work/production-repo07`
- TIMELINES = `D/jobs/csd-review-repair-timeline269/timeline-output`
- NARRATION = `D/jobs/csd-review-repair-rebind269/narration`
- BIN = `D/work/bin/review-repair268`

## Answer

**It is possible now, without any final MP4.**
- Stage only the 15 `KEY.timeline.tsv` files as symlinks in a new folder inside a new guarded job.
- Run `BIN captions all --narration NARRATION --out <that folder>` with cwd REPO07.
- The guard input closure is 318's own `film()` set, which already includes `contract_reads()` (N7).

**One part is impossible as asked: it cannot run *while* timeline313 runs, if it is guarded.**
- The guard takes the exclusive lane lock `D/expanded-native-lane.lock` before it creates the job folder. It waits on it with no timeout (runner `run_bounded_phase_oliver_owner48g_target14g_20261102.py:101–107`; `guard_csd02.py` sets the guard's ROOT to D).
- Timeline313 runs under the same `guard_csd02.py` (`review-repair-after-build313.py:28`).
- So a caption job launched now waits until timeline313 finishes. While it waits it creates nothing and spends nothing.
- The only way to overlap would be to run outside the guard, and this recipe does not do that.

## What the native captions read and write (checked in source)

Binary 268's `captions all` does this, in order. Kit line numbers are the repository's; `check_delivery_contract` and `captions_video` are identical in CSD's repo-view `cli.rs` (`3c5221fe…`).

1. **`run_impl`.** The facts self-check (no file), then `check_delivery_contract` (`cli.rs:390–471`).
   - Topic is the build268 crate (`main.rs:48`, `CARGO_MANIFEST_DIR`). The script check compares `topic/script.tsv` with the compiled script (`cli.rs:400–402`).
   - It runs `python3 -I -B WORKSPACE268/tools/narration_delivery.py --consumer captions --gain-db 0 --chapter-key ×15 --timelines <--out>`, with no `--score` (`:428–466`).
   - For `captions`, the contract reads all of the following (`narration_delivery.py:275–325`):
     - the 3 tool modules and `script.tsv`;
     - the canonical units file and rebind269's folder;
     - the generation receipts, with their WAVs, sources and requests;
     - the map and the disposition evidence;
     - `report.json`;
     - each `OUT/KEY.timeline.tsv`, and the WAV each line row names.

     It reads no `cuelog` (`cue_log()` is master-only, `:328–329`), and it writes nothing.
2. **`captions_video` (`cli.rs:892–941`).**
   - It reads `NARRATION/report.json` and `OUT/KEY.timeline.tsv` for all 15 chapters. `ChapterTimeline::read` (`explainer_score/src/timeline.rs:190–191`) reads only that file.
   - It reads each line's WAV by its path (`captions.rs:553`).
   - It builds and validates the captions (`cli.rs:915–919`).
   - Only then does it write `OUT/complex_analysis_visual.en.srt` and `.en.vtt` (`:925–929`). `set_aside` moves a file only if it already exists (`:261–264`), which in a new folder it never does.

**Consequences.**
- Nothing reads an MP4, `cues.tsv`, `cuelog.tsv`, `layout.tsv`, a soundtrack capture or the score stems. The staged set is the 15 timeline tables alone.
- The run writes only the two caption files, plus the guard's own `resource-receipt.json` and `command.log`.

## Step 0: what root confirms first (read-only)

- `D/jobs/csd-caption-preflight327` and `Q/caption-preflight327-inputs.json` do not exist.
- `Q/cc0-combined-master318.py` is staged on CSD at `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57`. Step 1 imports it and writes nothing if the hash differs.

## Step 1: freeze the guard inputs (read-only except one new file)

Run on CSD with `/usr/bin/python3 -I -B -`. It reads 318 as a module; nothing runs at import, since 318's top level is constants and definitions.

It calls `film()`, 318's existing read-only whole-film gate:
- binary 268 at `983954c5…`;
- the four `CONTRACT` files at build268's source-copy hashes;
- all 45 timeline269 tables equal to timeline21's under the one narration mapping, with header frames summing to 133,662;
- the 105 rebind269 WAVs equal to assembly19's;
- score21 QA;
- and, through `contract_reads()`, every N7 file.

It then writes the returned set, plus 318 itself, as the inputs file.

```python
import hashlib, importlib.util, json
from pathlib import Path
D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
H = Q/'cc0-combined-master318.py'
assert hashlib.sha256(H.read_bytes()).hexdigest() == '478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57'
spec = importlib.util.spec_from_file_location('m318', H)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
registry, frames, files = m.film()          # read-only; includes contract_reads()
assert registry == ['01_the_bump', '02_turning_numbers', '03_two_planes', '04_reading_color', '05_gallery',
                    '06_one_derivative', '07_one_point', '08_loop_sums', '09_boundary', '10_rigidity',
                    '11_counting', '12_residues', '13_real_integral', '14_continuation', '15_closing']
files = {Path(p) for p in files} | {H}
for key in registry:                         # every table and every line WAV the captions open is frozen
    table = m.TIMELINES/(key+'.timeline.tsv')
    assert table in files, table
    for row in table.read_text().splitlines():
        if row.startswith('line\t'):
            assert Path(row.split('\t')[4]) in files, row
rows = [dict(path=str(p), sha256=m.sha(p)) for p in sorted(files)]
assert rows and all(Path(r['path']).is_absolute() for r in rows)
out = Q/'caption-preflight327-inputs.json'
with out.open('x') as stream:
    json.dump({'files': rows}, stream, indent=2)
print(len(rows), hashlib.sha256(out.read_bytes()).hexdigest())
```

- **Why this set.** It is the file set 318 itself will freeze for the whole film. Its 318-specific rows (score21 stems, timeline21, build19) are not read by the captions. Freezing them costs only hashing time.
- **Smaller alternative.** The closure the captions actually read is:
  - BIN;
  - `CONTRACT` (four files);
  - `contract_reads()`;
  - NARRATION's files;
  - the 15 `TIMELINES/KEY.timeline.tsv`.

  It is safe because every line WAV lies inside NARRATION: `film()` requires timeline269 = timeline21 with assembly19's folder mapped to rebind269's.
- **Timing.** It runs read-only, so it can run while timeline313 runs. Its I/O is mostly the score stems and the narration.

## Step 2: the guarded command

The argv, in the form root's launchers use (`subprocess.run([...])`):

```python
D = '/data/projects/jmev-stroke-probe-20261106'
SCRIPT = r'''
D=/data/projects/jmev-stroke-probe-20261106
T=$D/jobs/csd-review-repair-timeline269/timeline-output
O=$D/jobs/csd-caption-preflight327/captions
mkdir "$O"
for k in 01_the_bump 02_turning_numbers 03_two_planes 04_reading_color 05_gallery 06_one_derivative 07_one_point 08_loop_sums 09_boundary 10_rigidity 11_counting 12_residues 13_real_integral 14_continuation 15_closing; do
  ln -s "$T/$k.timeline.tsv" "$O/$k.timeline.tsv"
done
cd "$D/work/production-repo07"
export PATH=/usr/bin:/bin TMPDIR="$D/work/tmp" XDG_CACHE_HOME="$D/work/cache"
exec "$D/work/bin/review-repair268" captions all --narration "$D/jobs/csd-review-repair-rebind269/narration" --out "$O"
'''
argv = ['/usr/bin/python3', '-I', '-B', D+'/guard_csd02.py', '--phase', 'native',
        '--job', 'csd-caption-preflight327',
        '--inputs', D+'/qualification01/production-repair17/caption-preflight327-inputs.json',
        '--', '/usr/bin/timeout', '3600', '/bin/sh', '-eu', '-c', SCRIPT]
```

**Why each part:**
- **Job folder and cwd.**
  - The guard creates `D/jobs/csd-caption-preflight327` and refuses an existing one (runner :106–107). It runs the command with cwd D and its own environment (`:140–148`).
  - So the script makes the new output folder itself, after the guard has created the job folder.
  - It changes to REPO07, because the contract resolves rebind269's relative receipt paths against the cwd. Root's 305 refusal, "No such file or directory: 'narration_lab/…/decode-receipt.json'", came from running elsewhere.
- **Environment.** The script sets PATH, TMPDIR and XDG_CACHE_HOME as 318's `master()` does (:885). With PATH `/usr/bin:/bin`, the kit's `python3` resolves to `/usr/bin/python3`.
- **Staging.**
  - `mkdir` and `ln -s` without `-f` fail on any existing name, and `sh -e` stops at the first failure.
  - Symlinks change nothing in TIMELINES, not even the inode change time that a hard link would. The kit and the contract follow them, as 318's preflight relies on.
  - The guard's growth walk skips symlinks (runner :48).
- **`--out` is both the input and the output folder.** For `captions`, the kit passes `--out` as the contract's `--timelines` (`cli.rs:457–463`), and `captions_video` reads its tables from `--out`. That is why the tables must be staged: pointing `--out` at TIMELINES would write the two caption files into timeline269's folder.
- **Timeout.** `/usr/bin/timeout` follows the guarded `score21` command's precedent. 3600 s is a bound, not a measurement.
- **Policy.** `--phase native` admits at 10 GiB free and stops at 8 GiB, with 4 GiB growth (the receipt's policy is 10/8/4/56/20). The run adds two text files and a log.

## Observables

**Positive: actual native captions without final MP4s.**
- `D/jobs/csd-caption-preflight327/resource-receipt.json`:
  - `admitted: true` and `exit: 0`;
  - `inputs_sha256 == final_inputs_sha256`;
  - no `error` and no resource stop.
- `D/jobs/csd-caption-preflight327/command.log` contains, in order:
  - the contract's one JSON line, with `"consumer": "captions"`, `"verified_cues": 105`, `"selected_cues": 105` and `"plan_mode": "external-ui"`. Its caption segments and prediction holds are empty, because the contract computes those only for `master`.
  - the kit's line `captions: N sentences as M captions -> …/captions/complex_analysis_visual.en.srt`.
- `captions/complex_analysis_visual.en.srt` and `.en.vtt` exist; record both hashes.

**A refusal is a result, and costs only the job name, not 318.**
- **`exit 1` with `narration delivery refused: …`**: the contract refused, and the kit wrote nothing (`cli.rs:468–470`).
- **`exit 1` with `captions refused: …`**: the kit's validation refused on order, line count, line length or overlap (`captions.rs:590–612`).
- **`exit 1` with a `recorded captions cover … but WAV has …` or `no speech found` error**: the caption build failed.
- **`124`**: timeout.
- **`75`**: resource stop.

  In every one of these cases, no caption file is written.

**Negative: no writes to an existing timeline or render folder.**
- **The command creates only these:** `captions/`, its 15 symlinks and the two caption files.
- **No other writer:**
  - the kit writes only the two caption paths (`cli.rs:925–929`);
  - the contract writes nothing;
  - `-B` stops `.pyc` writes into WORKSPACE268.
- **The guard's final check proves the content is unchanged.** It re-hashes every frozen file after the command (runner :168–170), including:
  - all 45 timeline269 tables;
  - the NARRATION files;
  - the receipts, sources, requests and evidence.
- **Extra read-only evidence for root:** a listing (name, size, mtime, ctime) of TIMELINES and NARRATION before and after, to show that no entry was added or touched.
- **Nothing is rendered,** and no render, preview or final folder is named in the command.

## What a pass means for 318

**The final master's caption step should produce the same bytes.**
- 318's `master()` runs the same binary and the same `captions all`, with the same rebind269 `report.json`.
- Its staged final tables equal timeline269's, apart from the narration folder named in the line rows. 318 enforces this:
  - `schedule()` for every row;
  - `provenance313` and 317's `schedule313` for 09's timeline313.
- The WAVs those rows name are byte-identical. The caption build reads WAV bytes and line start frames, and its output holds no path.
- So 318's `complex_analysis_visual.en.srt` and `.vtt` should be byte-identical to this run's. Root can compare the hashes after 318; a mismatch would mean an input differs.

**What a pass removes, and what it does not.**
- A pass removes 322 F1's risk without editing 318. Root may still swap 318's order (322 F1 (a)).
- It does not check 318's gates, its master, or anything that 09's final317 tables add beyond timeline313. 318 checks those itself.

## Lane and sequencing (root's call)

- If launched while timeline313 runs, the guard blocks in `flock` until timeline313 releases the lane (runner :105).
- `preview-after-timeline313.py` starts preview313's launcher as soon as timeline313 finishes. That launcher calls the same guard with no timeout (`launch-review-repair-preview313.py:90–91`). Lock order is not guaranteed, so either job may take the lane first. The other just waits; neither fails.
- To keep preview313 first, launch this after preview313 has started, or once it has finished.

## Still open after a pass

- **Exact captions.** Nobody has read the caption text against the speech or the pictures.
- **Listening.** No audio was heard.
- **Fine A/V sync** of the captions against the final master.
- **The final master and final AV acceptance,** including independent AV acceptance.
- **Publication306's binding** to the final captions.

## Limits

- The kit and contract behaviour above comes from source. I read `captions.rs` and `timeline.rs` from the repository only, not from binary 268's compiled bytes.
- The 3600 s bound and the hashing time are not measurements.
- That 318's later captions will match is a prediction from the sources, not an observation.

## Not done

- No job, launch, render, synthesis, build or transfer.
- No edit to 318, 317, the guard or any other file.
- No writes besides this file.
