# Complex final production on ts1: handoff

Three helpers that take the finished chain on ts1 through 4K chapters, master and captions, and technical
verification. Root launches them. Nothing here has run on ts1, and nothing starts on import.

**Author:** LavenderPike adapted WhiteSwan's `complex-optimized-final-production01` helpers. This is the
author's rehearsal, not an independent review. Root's independent narrow review (Mail 5544) is separate.

**Status:** prepared and sealed. No SSH, no launch, no heavy job, no Beads or TODO change, no commit.

## What root does, in order

| Step | Run on ts1, by the `/Users` path | Stops unless |
|---|---|---|
| 1 | Transfer this folder's helpers and the three files in `transfer-files01.json` | |
| 2 | `python3 -I -B ts1_final_render01.py --check` | pins, tools, x264 and the three transfers hold. Valid before the chain ends |
| 3 | `python3 -I -B ts1_final_verify01.py --pins` | strict check, caption screen and imports hold. Valid before any master |
| 4 | Wait for `pictures02`, view every preview sheet, write the review file below | a person or agent actually viewed the sheets |
| 5 | `python3 -I -B ts1_final_render01.py --check-review` | the review binds these exact sheets, this preview and this N2 |
| 6 | `start-final-render01.sh 01` | |
| 7 | `python3 -I -B ts1_final_master01.py --check`, then read its growth line | see "Master growth" |
| 8 | `start-final-master01.sh 01` | |
| 9 | `python3 -I -B ts1_final_verify01.py --check` | the full plan holds, captions included. Read-only |
| 10 | `start-final-verify01.sh 01` | it also writes the postcheck record |
| 11 | Actual viewing of the final sheets, then BronzeJaguar's ASR, then continuous listening | these are separate and none of them is here |

Run the direct `python3` steps from the repository root, with the start scripts' PATH, or the tool lookups fail:
`export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin"`.

The start scripts need a new two-digit run number each time. A repeated number is refused before it touches
that run's log or exit file. Their logs and exit files are `final-{render,master,verify}-controller<RUN>.log`
and `.exit` in `O/final-production01`.

## Files root must transfer

`complex-remote-final-production01/`: the three `ts1_final_*.py`, the three `start-final-*.sh`,
`transfer-files01.json`. `checks/` stays here; it needs this host's tree.

Three files are in no ts1 transfer list yet, and step 2 names any that is missing:

| File | sha256 |
|---|---|
| `P/logarithm-stage09-final-verification02/verify_stage09_final.py` | `f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94` |
| `narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/verify_video_preserve_logs01.sh` | `3f4c68dbdcb02c1c90ff03cb4f20afc36792b7f20c6cc111a5eef21617a11410` |
| `narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/caption-interval-audit01.py` | `3cc2643615ce5263b01d429ae6f72326c95e777af3e7f29ef9266a05155fa0c4` |

## The preview review file

`O/final-production01/preview-pictures-viewed01.json`. Nothing here writes it, and no flag skips it.

```json
{
  "reviewer": "who actually viewed the sheets",
  "review_scope": "source-author or independent",
  "pictures_viewed": true,
  "chapters": ["the 15 chapter names, in order"],
  "unresolved_visual_defects": [],
  "sheet_sha256": {"<logical path of each sheet>": "<sha256>"},
  "extraction_sha256": "<sha256 of pictures/extraction.json>",
  "preview_summary_sha256": "<sha256 of preview-summary.json>"
}
```

`sheet_sha256` keys use the `/Users` spelling, not `/data`, and must name every `sheet-*.png`. A sheet that
leaves the folder and the review together is still refused, from `extraction.json`. It covers preview sheets
only. `verification.json` says so and records the final pictures as not viewed.

## Names and records

| Thing | Name |
|---|---|
| Chain | `complex-remote-ts1-pipeline01/chain02-n2`, seal `625d162d…227c`, `ts1_chain02_n2.py` `64502efa…8a45`. The unused `chain02/` attempt is never loaded |
| Narration | `O/jobs/complex-remote-narration-derive01/narration-n2`, from `M.N`. Bound by `L.check_derived()` and the manifests. Its `native-install-receipt.json` is byte-identical to N's and describes N, so never verify N2 against it |
| Binary | `O/complex_analysis_visual-release01`, `3c90fb54…c168` (build01 failed only on missing SciPy; qualifier01 qualified it separately) |
| Inputs | `complex-remote-{timeline,score,preview,pictures}02`, 15 chapters, 105 Frederick cues |
| Chapter jobs | `complex-remote-render-<chapter>-01`, then `-r2` to `-r9` |
| Shared outputs | `complex-remote-render-work01`: `<chapter>.mp4` and tables, `complex_analysis_visual_master.mp4`, `.en.srt`, `.en.vtt` |
| Moved attempts | `complex-remote-render-aside01/<failed job>/`, with `moved.json` |
| Master, verify | `complex-remote-master01`, `complex-remote-final-verify01`, each then `-r2` to `-r9` |
| Manifests | `O/final-production01/<job>-inputs.json` |
| Postcheck | `O/final-production01/<verify job>-postcheck.json` |

**Each job's last `command.log` line, exactly:**

- Render: `RENDER PASS: 4K60 chapter; frame and cue schedule equal the timeline; final AV, pictures and listening remain.`
- Master: `MASTER WRITTEN: candidate master and captions; strict verify, pictures, ASR, loudness and listening remain.`
- Verify: `FINAL VERIFY PASS: strict video check, frame count, A/V length and exact captions passed; every sheet still needs actual viewing.`

`--postcheck JOB` repeats the postcheck read-only. It rechecks the guard's final receipt and command, every
manifest row, the success line, master and caption hashes, every picture and sheet, the strict log, the caption
screen and the two recorded `false` claims. Its record holds `job, master_job, master_sha256, srt_sha256,
vtt_sha256, verification_sha256, frames, pictures, sheets, pictures_viewed, scope`, and identical bytes are
accepted on a repeat. The record proves that records agree. It is not acceptance.

## When a job fails

Nothing retries by itself. `--all` stops at the first failed chapter, and later runs refuse until you read its
log and name the next attempt. Chapters that passed are skipped once their hashes hold.

```
start-final-render01.sh 02 --retry 04_reading_color=r2
start-final-master01.sh 02 --retry master=r2
start-final-verify01.sh 02 --retry verify=r2
```

- A wrong attempt name is refused and the message names the right one.
- A retry of an attempt that passed is refused. If a passed chapter's file later changes, `--all` stops at it and
  no retry exists: that is root's call.
- A chapter or master retry first moves the failed outputs aside, with hashes, and only while the lane is free.
- A launch `sbh` or the free-space test refuses uses no job name. The helpers run that test first because the
  guard makes the job folder before it asks.
- Nothing is deleted.

## Master growth

**Risk: low, not proven.** The master job has the guard's 4 GiB growth cap. It holds a second copy of the video
and of the soundtrack, so it would hit the cap only if the 15 chapters sum to about 1.6 GiB, an average near
7 Mbit/s over about 33 to 34 minutes. Logarithm's estimate of 1.07 GiB compares with 1.02 GiB measured.

| Measured here, read-only | Result |
|---|---|
| Six retained complex 4K60 chapters, older film, x264 record says CRF 16 | 0.55 to 0.75 Mbit/s, sound included |
| Soundtrack for 34 minutes, 48 kHz 16-bit stereo | about 0.36 GiB |
| Fixed cost, two soundtrack copies | about 0.7 GiB |

The current film has heavier scenes than that older one: `05_gallery` was the largest preview. Step 7 prints the
real estimate from the real chapters, with `over_cap`. It decides nothing. If it says `true`, stop and raise it,
because `Q.POLICY` is checked on every job and no flag here changes it. A stopped master leaves a preserved job.

## Changes from WhiteSwan's helpers

Diff against `complex-optimized-final-production01/final_{render,master,verify}_optimized01.py`.

- **Chain:** imports sealed `chain02-n2`, not the five macOS modules. Every file comes from `M`, `Q` and `L`, so
  all paths are logical, built from `R`.
- **Review:** also runs `L.check_qualified()` and `L.check_derived()`, and refuses a preview whose summary names
  another voice or binary.
- **Gate:** `live_receipt()` rereads only a half-written receipt, since the real guard rewrites it in place just
  after starting a job. `gate()` also requires python3, ffmpeg and ffprobe in the manifest.
- **CRF 16:** read from x264's own record in each movie, and kept in `render-summary.json`.
- **Retries:** named retries, the aside folder, a first-launch admission test, and identical-manifest resume.
- **Success lines and `--postcheck`:** new, for Bronze's ASR step.
- **Verify child:** runs under `M.PICTURE_PYTHON`, because the shared sheet maker needs Pillow and ts1's python3
  has none.
- **Transfers:** `--check` names any of the three missing files.

Unchanged: `--crf 16 --preset veryfast --lint`, timeouts of 43200 s per chapter, 14400 s master, 1200 s captions
and all verify timeouts, the native bit-identical replay proof, exact table comparison, the shared media and
caption checks, and `pictures_viewed: false`. New timeouts: 45 s for `sbh`, 60 s for the x264 preflight.

## What the rehearsal shows

`checks/rehearse_final.py`, output in `checks/outputs/rehearse_final.txt`: **CHECK_COUNT checks, 0 failed.**

| Real | Stand-in |
|---|---|
| The three helpers and start scripts, with only hash pins changed | The binary |
| The sealed qualifier, chain01 and chain02-n2, run first through their own start scripts | 36-frame movies of black and a sine |
| The real guard, hostname changed | Chapter and master tables, made by the fake binary |
| All 105 cues and takes, N and N2 | macOS receipts, `sbh` |
| Unmodified `narration_delivery.py` and `coherent_narration.py`, including its `master` consumer | The review file's viewer |
| Strict video check, caption screen, ebur128, ffmpeg, ffprobe, Pillow | Local macOS tools, not ts1's |

It also reads real logarithm files, read-only: x264's record in a real 4K60 chapter, a real capture, a real
master log's proof lines. A positive and a negative control show the fake tables satisfy the real tool's `master`
check: dropping one cue is refused.

## What was not run or shown

- No 4K frame was rendered for this film. Speed on ts1 is unknown: the guard sets four Rayon threads, and chapter
  04 was the slow one earlier. `preview02` is the only real trial of `render`.
- The real binary has never run `master` or `captions` on ts1. The preview path says nothing about either. The
  first master is their first real run on that host.
- The guard's 4 GiB stop was not triggered: tiny fixtures cannot reach it.
- Phrase-mark samples in the final picture selection did not land, because real marks fall past a 36-frame
  chapter. That selection is `M.select` unchanged, which the real `pictures02` exercises.
- Linux strict decoding, and ts1's ffmpeg 8.0.1, were not run here. The strict script reads as Linux-safe, and a
  hardware decode that cannot run is only noted, so the software decode decides.

## Limits

These are technical records only. Nothing here shows the pictures look right, the sound is right, the sync is
fine, or the film is accepted. Hashes, ASR, loudness numbers and these checks are not listening or viewing. Root
owns final picture viewing, continuous listening, AV acceptance and publication.
