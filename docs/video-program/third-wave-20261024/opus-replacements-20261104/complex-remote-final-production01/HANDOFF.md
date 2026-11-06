# Complex final production on ts1: handoff

Three helpers that take the finished chain on ts1 through 4K chapters, master and captions, and technical
verification. Root launches them. Nothing here has run on ts1, and nothing starts on import.

**Status: unsealed, awaiting a delta read. Do not launch.** Root's Mail 5589 holds final complex renders. The
bundle is now pinned to WhiteSwan's sealed `chain03`, the corrected-film chain from Mail 5814.

**Author:** LavenderPike adapted WhiteSwan's `complex-optimized-final-production01` helpers. This is the
author's rehearsal, not an independent review.

**Independent review:** RubyRaven's source review (Mail 5600) found no launch or identity bug, on earlier bytes:
render `73590bfd` in full, master and verify before their pins. The provenance, chain-name, x264 and `chain03`
changes came after it, so they have had no independent read.

No SSH, no launch, no heavy job, no live input changed, no Beads or TODO change. The author made no commit.
Repository sweeps have picked up most of this folder, so read the working tree, not a commit, as current.

## Before this bundle can launch

State at 2026-11-06 02:30 UTC, from root's records. The author can't see ts1, so root's receipts decide each row.

| # | Needs | Shown by | State |
|---|---|---|---|
| 1 | `chain02` ends, with `timeline02` at exit 0. It is `chain03`'s baseline | `O/chain-timeline-controller02.exit` | Running on ts1, last seen in chapter 04 |
| 2 | `complex-remote-build02` ends at exit 0, which makes `release02` | `binary.sha256`, `Q.check_build()` | Not launched at the last record. It waits for row 1 |
| 3 | `chain03`'s derive, timeline, score, preview and pictures, in order | `O/chain-<stage>-controller03.exit` | Not started |
| 4 | Root's decision on the stroke-index candidate | csd A/B records | Open, candidate not compiled. Adopting it changes the binary, so expect a new qualifier and chain, then a new re-pin and rehearsal here |
| 5 | Root's decision on the chapter time limit | See "What was not run or shown" | Open. A longer limit changes the render helper, so settle it before the seal |
| 6 | A delta read of this bundle, then its seal | `SHA256SUMS` | Open. Bronze's ASR bundle also waits for this seal |
| 7 | Root lifts the Mail 5589 hold | Root | In force |
| 8 | The preview review file, from real viewing of real `preview03` sheets | Step 5 | Cannot start before row 3 |

The stage07 audio is already on ts1 (490 files verified, `O/stage07-transfer-receipt01.json`). So is `chain03`, whose six
sealed files root verified by hash.

## What root does, in order

| Step | Run on ts1, by the `/Users` path | Stops unless |
|---|---|---|
| 1 | Transfer this folder's helpers and the three files in `transfer-files01.json` | |
| 2 | `python3 -I -B ts1_final_render01.py --check` | pins, tools, x264 and the three transfers hold. Valid before the chain ends |
| 3 | `python3 -I -B ts1_final_verify01.py --pins` | strict check, caption screen and imports hold. Valid before any master |
| 4 | Wait for `pictures03`, view every preview sheet, write the review file below | a person or agent actually viewed the sheets |
| 5 | `python3 -I -B ts1_final_render01.py --check-review` | the review binds these exact sheets, this preview and this N2, and x264's record reads as CRF 18 from a real preview movie |
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
| Chain, current pin | `complex-remote-ts1-pipeline01/chain03`, seal `bbf00548…daf9`, `ts1_chain03.py` `681199ff…5c62`. The helpers load no earlier chain |
| Narration, current pin | `O/jobs/complex-remote-narration-derive02/narration-n2`, which is `M.N`, derived from `L.N1`, the stage07 audio. Bound by `L.check_derived()` and the manifests. Its `native-install-receipt.json` is byte-identical to `L.N1`'s and describes `L.N1`, so never verify N2 against it |
| Binary, current pin | `O/complex_analysis_visual-release02`, from `complex-remote-build02`. Its hash is read from `binary.sha256` in that job, so `Q.BIN_SHA256` is `None` until the build ends. Nothing here pins it |
| Inputs, current pin | `complex-remote-{timeline,score,preview,pictures}03`, 15 chapters, 105 Frederick cues |
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
vtt_sha256, derived_from, derive_summary_sha256, verification_sha256, frames, pictures, sheets, pictures_viewed,
scope`, and identical bytes are accepted on a repeat. The record proves that records agree. It is not acceptance.

**Narration provenance.** `render-summary.json`, `master-summary.json` and `verification.json` keep `narration`
and add `derived_from`, `derive_job` and `derive_summary_sha256`, read from the derive job's own summary. Each
later step refuses a record whose values differ. A later reader should cite these, not the retained install
receipt.

## Moving to the new chain

The 09_5 correction brought a new binary, narration folder and chain (Mails 5589, 5600, 5612, 5814). This bundle
needed only new pins. The binary, narration folder and input job names all come from the chain: the helpers
hard-code only the chain's own path, so the "current pin" rows above changed with it. This bundle's job, output
and stage-folder names are unused and stay.

| Step | State |
|---|---|
| 1. WhiteSwan seals the chain and sends its path, hash, stage job names and fixture | Done, Mail 5814 |
| 2. `python3 -I -B <this folder>/checks/repin_chain01.py --chain <path relative to P> --sha256 <hash>`, on this host, then `--write` | Done. It set `CHAIN`, `CHAIN_SHA256`, `RENDER_SHA256` and the `MASTER_HELPER` pin. It refuses a hash the chain's own `SHA256SUMS` does not list, and it refuses a sealed bundle |
| 3. Update the block marked at the top of `checks/rehearse_final.py` and rerun it | Done, see "What the rehearsal shows" |
| 4. A delta read, then the seal, then the seal and hashes go to root and BronzeJaguar | Open |

The same steps apply if a later chain replaces `chain03`, but a sealed bundle is preserved, so that move needs a
new folder.

| Pin | Result, render `23d99bfb…fc70` |
|---|---|
| Master | `6ed3cb2a…cdb2` |
| Verify | `13cf9a09…0667` |

`--check` refuses a chain that lacks any name in `CHAIN_NAMES`, which a rehearsal check holds equal to what the
helpers use. If the new chain drops derive, `narration_source()` needs a small change first, which no pin covers.

The review file must bind the new sheets, extraction and preview summary, and must list every sheet, including
chapters whose bytes match the earlier preview. Whether a viewing carries over to matching chapters is root's call.

No change needed: `transfer-files01.json`, the shared checks, the strict script, the caption screen, 105 cues and
15 chapters.

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
| RubyRaven's separate estimate for the master (Mail 5600) | about 1.1 to 1.7 GiB |

The current film has heavier scenes than that older one: `05_gallery` was the largest preview. Step 7 prints the
real estimate from the real chapters, with `over_cap`. It decides nothing. If it says `true`, stop and raise it,
because `Q.POLICY` is checked on every job and no flag here changes it. A stopped master leaves a preserved job.

## Changes from WhiteSwan's helpers

Diff against `complex-optimized-final-production01/final_{render,master,verify}_optimized01.py`.

- **Chain:** imports the one sealed chain file pinned at the top of the render helper, not the five macOS modules.
  Every file comes from `M`, `Q` and `L`, so all paths are logical, built from `R`.
- **Review:** also runs `L.check_qualified()` and `L.check_derived()`, refuses a preview whose summary names
  another voice or binary, and reads x264's record from a preview movie so the 4K gate's reader is tried on ts1's
  own kit output before the first long job.
- **Chain names and provenance:** `CHAIN_NAMES` and `narration_source()`, both new.
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

`checks/rehearse_final.py`, output in `checks/outputs/rehearse_final.txt`: **151 checks, 0 failed, on stand-ins.
None of this is native evidence.** It shows the helpers' rules run and refuse. It doesn't show that the real binary,
ts1 or the corrected film will pass them. The `chain02-n2` transcript is kept as
`checks/outputs/rehearse_final-chain02-n2.txt`.

The same 151 checks ran against `chain03`. One expected string changed: `v-qualification-changed` now expects
`Build summary differs`, which is `ts1_qualify02.py`'s refusal of the same planted change.

| Real | Stand-in |
|---|---|
| The three helpers and start scripts, with only hash pins changed | The binary |
| The sealed qualifier, chain01, chain02 and `chain03`, run first through their own start scripts, with the stage07 audio | 36-frame movies of black and a sine |
| The real guard, hostname changed | Chapter and master tables, made by the fake binary |
| All 105 cues and takes, N and N2 | macOS receipts, `sbh` |
| Unmodified `narration_delivery.py` and `coherent_narration.py`, including its `master` consumer | The review file's viewer |
| Strict video check, caption screen, ebur128, ffmpeg, ffprobe, Pillow | Local macOS tools, not ts1's |

It also reads real logarithm files, read-only: x264's record in a real 4K60 chapter, a real capture, a real
master log's proof lines. A positive and a negative control show the fake tables satisfy the real tool's `master`
check: dropping one cue is refused.

**Stage07 fixture.** `checks/rehearse_final.py` read its stage07 audio, read-only, from WhiteSwan's finished
rehearsal root `rehearse08`, by setting `STAGE07_REHEARSAL_ROOT` as Mail 5814 allows. A fresh `rehearse_stage07.py`
did not finish on this host: its audio `--launch` failed, and its diagnostic then stopped on a linked preflight
folder with no `command.log`. Its fixture links the real job folders, and root's real stage07 job folders now exist,
so that is the likely cause. The launch output was not captured, so this is not confirmed. It says nothing about
the real audio.

## What was not run or shown

- No 4K frame was rendered for this film, and 4K speed on ts1 is unknown. `preview03` will be the first real trial
  of `render` on the corrected film.
- **The 43,200 s chapter limit may be too short for chapter 04.** It has 6,774 frames, so it must average at least
  0.157 fps at 4K. The only ts1 reading is 0.2 fps at 64×36, over 30 s (`O/timeline04-live-rate01.json`). The 4K rate
  is unmeasured, so this is a risk and not a prediction. A timed-out chapter retried under `--retry` gets the same
  limit, because the helper fixes it. `preview03`'s chapter 04 time is the first useful evidence, and step 6 should
  wait for it.
- The real binary has never run `master` or `captions` on ts1. The preview path says nothing about either. The
  first master is their first real run on that host.
- The guard's 4 GiB stop was not triggered: tiny fixtures cannot reach it.
- Chapter 09's growth was not rehearsed. The rehearsal sets `Real.shift = 0`, so every fixture chapter keeps its
  length, while the real 09_5 adds frames to `09_boundary`. Nothing here tests the corrected film's pictures or
  sound.
- `chain03` has run only on the fake binary, here and in White's rehearsal. It has not run on ts1.
- Phrase-mark samples in the final picture selection did not land, because real marks fall past a 36-frame
  chapter. That selection is `M.select` unchanged, which the real `pictures03` will exercise.
- Linux strict decoding, and ts1's ffmpeg 8.0.1, were not run here. The strict script reads as Linux-safe, and a
  hardware decode that cannot run is only noted, so the software decode decides.

## Limits

These are technical records only. Nothing here shows the pictures look right, the sound is right, the sync is
fine, or the film is accepted. Hashes, ASR, loudness numbers and these checks are not listening or viewing. Root
owns final picture viewing, continuous listening, AV acceptance and publication.
