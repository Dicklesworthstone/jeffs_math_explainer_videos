# Complex encoded-master ASR, two hosts

BronzeJaguar, 2026-11-05, for BlueSalmon. This answers Mail 5511, with the route root approved in Mail 5560. It was refreshed for
stage07 and the 09_5 correction from WhiteSwan's Mail 5814.

**Not launchable yet.** The bindings now name the corrected film, but three things are still missing:

- `complex-remote-build02`'s binary hash. No local copy of `binary.sha256` exists.
- The chain03 jobs, then a real final master, verify job and post-check.
- A sealed `complex-remote-final-production01`. Its helpers name chain03 in source, but it's unsealed.

Root's hold (Mail 5589) stands until root lifts it. The mini also needs the stage07 `script.tsv` at the path below.

**Nothing here has run on ts1 or mac-mini-max.** No SSH, launch, render, build, synthesis or production edit. All checks use
stand-ins on this host, so they prove the helpers' logic only. **ASR isn't hearing proof: listening and fine sync remain
separate and open.**

- P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- EC = `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`
- O = `EC/remote-ts1-01`
- MINI = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01`

## Why two hosts

ts1 holds the master but has no `fw` (Mail 5523). The mini has the verified `fw` and model but doesn't hold the master. So ts1
proves the master and decodes its audio directly from it, and the mini recognizes that WAV. The summary's hash, passed by root
as an argument, is the only link. The mini makes no claim about Linux verification.

## Start it

Transfer this folder to both hosts at the same path, without `__pycache__`. Both wrappers refuse at once if an input isn't ready,
because the chain has a human preview gate before final production. While another job holds a lane, its guard queues steps 2 and 3
behind it. Each attempt number is used once, from 01.

**Step 1, on ts1, from the repository root,** once the selected verify job's post-check record exists:

```sh
bash P/complex-remote-asr01/start-asr-audio01.sh complex-remote-final-verify01 complex-remote-master01 01
```

Name a retry directly, for example `complex-remote-final-verify01-r2 complex-remote-master01-r2 02`. The last line of
`O/asr-audioNN.log` is `ASR AUDIO PASS: ... audio-summary.json sha256 HASH; ...`. Carry that hash to step 2.

**Transfer to the mini, at the same repository paths:**

| File | Note |
|---|---|
| `O/jobs/complex-remote-asr-audioNN/audio-summary.json` | its hash is step 2's argument |
| `O/jobs/complex-remote-asr-audioNN/encoded-master-16k-mono.wav` | 16 kHz mono PCM, 32 kB/s |
| the frozen `script.tsv` | at the repository path `SCRIPT` names in `mini_asr01.py` |

**Step 2, on mac-mini-max, from the repository root:**

```sh
bash P/complex-remote-asr01/start-asr01.sh complex-remote-asr-audio01 HASH 01
```

| Step | ts1 | mac-mini-max | Stops on |
|---|---|---|---|
| Before logging | names, unused paths, `sbh`, `ffmpeg` | names, unused paths, `sbh`, pinned interpreter | any failure, nothing used |
| 1 of 3 | `--check`: every gate on the selected jobs | `--check`: tools, summary hash, script, WAV | any gate, no job name used |
| 2 of 3 | guard `--phase asr --preflight-only`, spare job `...-preflight` | the same on the mini's guard | exit 75 |
| 3 of 3 | manifest, guard, decode as `complex-remote-asr-audioNN` | manifest, guard, `fw` as `complex-remote-asrNN` | any gate |

Logs: `O/asr-audioNN.log` and `.exit` on ts1, `MINI/complex-asrNN.log` and `.exit` on the mini. Manifests:
`O/asr-audio-inputsNN.json` and `MINI/complex-asr-inputsNN.json`. A refused start before step 3 leaves the job name unused, but
the spare preflight name is spent, so use the next number. Outputs on the mini, in `MINI/jobs/complex-remote-asrNN/`:
`full-large.json`, `full-large.stderr.log`, `cue-boundaries.json`, `comparison.json`.

## Refreshed after 09_5

Mail 5599 has the reasoning. The script is compiled into the binary (`src/main.rs:22`), and `crates/explainer_kit/src/cli.rs:400`
refuses narrated render, captions, score and master when the topic's script differs from it. So the correction brought a new
binary and a reissued qualifier, not only a new take.

The values come from WhiteSwan's Mail 5814. I measured the qualifier, both seals, the chain file and the script hash here.

| File | Binding | Now |
|---|---|---|
| `ts1_asr_audio01.py` | `QUALIFIER`, `QUALIFIER_SHA256` | `complex-remote-ts1-pipeline01/qualify02/ts1_qualify02.py`, `604ef338…60ca` |
| `ts1_asr_audio01.py` | `SCRIPT_SHA256` | `946a9c07…7cdc` |
| `ts1_asr_audio01.py` | `N2` | `complex-remote-narration-derive02/narration-n2` under the jobs folder |
| `ts1_asr_audio01.py` | `TIMELINE_JOB` | `complex-remote-timeline03` |
| `mini_asr01.py` | `SCRIPT`, `SCRIPT_SHA256` | `EC/native-stage07-root01/workspace/explainer_corpus/complex_analysis_visual/script.tsv`, the same hash |
| `checks/run_checks.py` | `REL_SCRIPT`, `REL_Q`, `CHAIN_SEAL`, new `QUALIFY_SEAL`, the job names | the same, with seals `bbf00548…daf9` (chain03) and `f6878e68…860` (qualify02) |
| `checks/run_checks.py` | expected words | 4,662, measured from the stage07 script with `tokens()`. Stage06 was 4,658 |

**Unchanged:** `FINAL`, `VERIFY_NAME`, `MASTER_NAME`, both success lines and the name patterns in `start-asr-audio01.sh`, because
Lavender kept those names. `mini_asr01.py`'s `SOURCES_SHA256` and `TOOLS_SHA256` stay too.

**`Q.BIN_SHA256` is no longer a literal.** `ts1_qualify02.py` reads it from `jobs/complex-remote-build02/binary.sha256` at import, so
it's `None` until build02 finishes. This bundle compares it only after the chain has run. A film with no `binary.sha256` refuses,
and the fixture now has that case.

`ts1_qualify02.py` keeps every name step 1 loads: `R`, `O`, `JOBS`, `C`, `read`, `sha`, `PHYSICAL`, `POLICY`, `guard_policy`, `success`,
`BIN_SHA256`, `CHAPTERS`, `GUARD`. `Q.C` is now the stage07 crate.

**Not affected:** the 105-cue and 15-chapter counts, the decode, the `fw` settings, the mini's tool pins, both guards' phase and all
cue-edge windows. Between the stage06 and stage07 scripts only `09_5` differs. Its start frames and length come from the actual
final tables.

The new bindings refuse the older film: by script hash on both hosts, and on ts1 also by binary, narration path and timeline job.
The fixture covers all four refusals. No real corrected film exists to try.

## What step 1 requires

- **Selection:** the named verify and master jobs are the newest attempts, by name pattern.
- **Both jobs passed:** `Q.success`, phase `native`, Lavender's exact child command, frozen inputs, the helper's row in those
  inputs, and the exact success line last in `command.log`.
- **Proof:** `verification.json` names the selected master, ts1, Frederick, the qualifier's binary, timeline03, N2 and the logical
  master path. `pictures_viewed` is false and the strict log ends `VERIFY PASS`.
- **Placements:** rebuilt from the actual final tables and N2 WAV headers, as the verify job builds them. Each table equals
  timeline03's, each `line` row resolves to N2 (White's rule), the 105 cues are in script order, and the proof's chapters and
  placements are equal to the rebuilt ones.
- **Master and captions:** the master's hash equals the proof and `master-summary.json`. Both caption files match their records.
- **Post-check:** `O/final-production01/<verify job>-postcheck.json` agrees on job, master job, verification hash, master hash,
  frames and both caption hashes, and says `pictures_viewed` false.
- **ffmpeg:** `/usr/bin/ffmpeg` as PATH finds it must be the row in the verify job's frozen inputs, by hash.
- **After the decode:** 16 kHz, mono, 16-bit, and within 0.2 s of `frames / fps`.

## What the summary binds

Every item from Mail 5560, in `audio-summary.json`:

| Mail 5560 | Key |
|---|---|
| master hash, frame count, fps | `sha256`, `frames`, `fps` |
| all 105 placements | `placements`, with `chapters` |
| script hash | `script_sha256` |
| verification and post-check hashes | `verification_sha256`, `postcheck_sha256` |
| extraction command, ffmpeg hash | `decode_command`, `ffmpeg.sha256` |
| WAV hash and sample count | `wav_sha256`, `wav_samples` |

Step 2 reads only the summary whose hash root passed, then checks it against the frozen script, the 15 chapters, frame sums, the
105 cues in order and the WAV's hash. It carries master facts only as told by that summary, and writes them under the same keys
asr02 uses. It verifies nothing about the master.

## Unchanged from logarithm

Proved by AST in `checks/run_checks.py`:

- **`run()` from the decode on:** `logarithm-remote-asr02/asr02.py` is exactly step 1's two decode statements, then step 2's
  statements from `command = [fw, ...` to the end. That covers the decode arguments, 1800 s timeout, every `fw` argument, 7200 s
  and 7260 s timeouts, outputs and scope strings.
- **`read`, `sha`, `check_files`, `script_rows`, `tokens`, `boundary_review`:** equal to root's complex `after-verify01.py`, with its
  105-cue count, so every cue-edge window is kept. All but `script_rows` also equal `asr02.py`.
- **Signs:** on the real script the token rule loses only `.`, `,` and `?`. In 8 retained complex `full-large.json` transcripts it
  dropped only ordinary punctuation and kept `-`, `/`, `+` and `%`.

Both steps run in guard phase `asr`, each as its own job. Model and engine come from root's `MINI/asr-source-bindings01.json` and
`MINI/tools01.json`, pinned by hash, so this bundle holds no tool path of its own.

## Differences from logarithm

- **The decode tool differs.** Logarithm's WAV comes from the mini's ffmpeg 8.1.2. Complex's comes from ts1's ffmpeg 8.0.1, the
  same bytes that verified this master. Treat a small transcript difference between films accordingly.
- **Fixed by source, not by this bundle:** step 1's manifest binds `python3`, ffmpeg, both of Lavender's helpers, both job records,
  all final and timeline tables and the 105 N2 WAVs. A later change to any of them refuses at the guard's final check.

## Traps

- **Same `python3`.** Lavender's master child command holds `sys.executable`. Step 1 must find the same interpreter, so run it under
  the wrapper's PATH, which is the chain's.
- **Post-check file first.** Lavender's helper writes the record after the guard's final receipt. If the launch stopped before that,
  `ts1_final_verify01.py --postcheck JOB` writes it. Step 1 refuses until it exists.
- **Timeline and narration are fixed** at the constants above. A timeline retry under a new job name refuses here.
- **ts1 disk:** phase `asr` needs 12 GiB free at preflight and caps growth at 1 GiB. The 56 GiB work cap counts all of `O/jobs`,
  including the 4K chapters and master.
- **The mini's lane is logarithm's.** Step 2's job folder sits in its `jobs/`, the lane lock queues it behind any logarithm job, and
  the 56 GiB cap there also counts the logarithm render.
- **The WAV's path is compared by its repository-relative part,** so ts1's logical spelling and the mini's physical one both work.
  A WAV at another relative path refuses.
- **To move recognition to another host,** only `LANE` and the two pins in `mini_asr01.py` change.
- **Cache (Mail 5610).** Three `.pyc` files sit here, in `__pycache__` and `checks/__pycache__`. They're preserved and non-runtime. These
  scripts didn't write them, and they were rewritten at 19:15:06 local time by something I didn't identify. The scripts start as
  `python -I -B file.py`, which never reads a cache, and nothing imports them. Git ignores the folders. At sealing, `ANCILLARY.json` names
  them outside `SHA256SUMS`, with hashes as observations only, because another syntax check can rewrite them.

## Checks

From the repository root, with a fresh scratch path: `python3 -I -B P/complex-remote-asr01/checks/run_checks.py SCRATCH`.
It takes about a minute and writes only small files there. The transcript is `checks/outputs/run_checks.txt` (90 of 90). The run
before this refresh is kept as `checks/outputs/run_checks-before-stage07.txt`.

The real helpers and wrappers run as subprocesses in a tiny scratch repository, with a physical path and a `logical` symlink
standing for ts1's alias. Real: the sealed qualifier, the real `script.tsv`. Stand-ins: both guards, ffmpeg (a Python stand-in
writing a real WAV), `fw`, `sbh`, the `python3` shim, the tools files and every job record. The mini helper's fixture copy
differs only in its two pin literals, and one case edits a wrapper copy's PATH line.

- **Good runs:** a normal film across both hosts, retries beside failed attempts, and a live receipt that's partial for a moment.
  The fixture's tables spell the physical path, as a guard against a spelling comparison.
- **Refusals:** each new gate fires in a case by its own message. A compound assert is exercised through one of its conditions.
  This refresh added one case, build02 not finished, and turned the first-timeline case into an older-timeline case.
- **Peers, source text, not run:** White's chain03 and qualify02 seals, and Lavender's job names, both success lines, child
  commands, output, manifest and post-check names, and that each child prints its line last.

No mutation sweep was run.

## Limits

- **No real records exist.** No final master, verify job or post-check exists yet. Their forms come from Lavender's helpers as of
  20:36 local time: `ts1_final_render01.py` `23d99bfb8a5cfc30…`, `ts1_final_master01.py` `6ed3cb2ae08ec292…` and
  `ts1_final_verify01.py` `13cf9a09f320dc9c…`. They're unsealed, so no Lavender seal is bound here. The render helper pins chain03,
  but that bundle's `HANDOFF.md` still says `chain02-n2`.
- **Mail 5814 is helper-source approval only:** no compile, real table, render or audiovisual acceptance.
- **`fw transcribe` hasn't run on the worker,** as far as this bundle knows. Nothing here exercised it.
- **`tools01.json`'s hash was measured on this host only.** The bindings file's was also confirmed remotely (Mail 5503).
- **Python versions:** these checks ran under local Python 3.15. Neither host's interpreter was exercised.
- **Not independent.** I wrote all of it. RubyRaven read the binding lines for Mail 5600, which isn't a review of these helpers.
- **A passing ASR run gives transcript leads for triage only.** It doesn't show natural delivery, pronunciation or sync, and this
  bundle accepts nothing.
