# ts1 timeline, score, preview and pictures

WhiteSwan, for root (Mails 5300, 5339, 5340, 5380, 5396). This runs the qualified binary through the four native stages on ts1. It reuses the 105 Frederick takes and synthesizes nothing. I launched nothing and used no SSH. `O` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01`.

## Start

Transfer `chain01/` to the same path on ts1. `narration-closure01.json` must also sit at its repository path, one folder up. Then run these with the `/Users` path, not the `/data` one:

```
C=<this folder, by its logical path>
PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin" python3 -I -B $C/ts1_chain01.py --check timeline
bash $C/start-chain01.sh timeline
```

`--check STAGE` is read-only and spends no job name. If it refuses, send me its last line. It works before upstream jobs finish, so it can be run for `score`, `preview` and `pictures` now.

Start `score`, `preview` and `pictures` the same way, each once, whenever you like. Each waits for its upstream exit files, then repeats every upstream check before freezing its own manifest. A nonzero upstream exit stops it with nothing launched.

| Stage | Job in `O/jobs` | Output folder | Summary |
|---|---|---|---|
| timeline | `complex-remote-timeline01` | `timeline-output` | `timeline-summary.json`, `prior-table-comparison.json` |
| score | `complex-remote-score01` | `scored` | `score-summary.json` |
| preview | `complex-remote-preview01` | `preview` | `preview-summary.json` |
| pictures | `complex-remote-pictures01` | `pictures` | `pictures/extraction.json` |

Controller files are `O/chain-<stage>-controller01.log` and `.exit`. Manifests are `O/chain-<stage>-inputs01.json`. A second start of a stage is refused before it touches the first log.

## What each stage requires

**Every stage** needs the qualifier's success and the original failed build together: the sealed qualifier at its hash, `complex-remote-qualify01` terminal 0, `original_build_exit: 1`, `original_build_relabelled: false`, the original receipt and log unchanged, and the binary at `3c90fb54…c168`. The original build stays a failed job.

**Timeline** also needs:
- the macOS failed run exactly as its receipt says, with its 12 tables;
- every row of the qualifier's manifest, the 833 audio rows and the 316 closure rows at their hashes;
- no extra or missing file in `native-narration01` or `native-stage06-root03/workspace`.

It then runs native facts against the oracle under the oracle env. Both must equal the failed run's and the release run's facts. It checks the 15 chapters and 105 script ids, runs the narration verifier, renders, writes the comparison, and judges it. Each of the 12 tables must equal the macOS run line for line. `full_baseline_equivalence_established` is always `false`, because chapters 05-15 have no prior table.

**Score** repeats all timeline checks, then needs all 15 QA reports passed.

**Preview** repeats the checks, renders 960x540 at 60 fps with lint, and compares schedules and every score row of each cue log.

**Pictures** runs under the picture env's Python. It uses the sealed selection rules, extracts and builds sheets, and writes `pictures_viewed: false`. **Someone must actually view every sheet before any 4K stage.** The 4K stage isn't adapted yet.

## ffmpeg (Mail 5396)

The binary encodes through ffmpeg in the timeline and preview renders, so the timeline manifest freezes the resolved ffmpeg. Later manifests carry all earlier rows, so score, preview and pictures freeze the same bytes. Preview adds ffprobe. Each child resolves ffmpeg again from its own `PATH` and refuses unless that path is in its manifest. `--check` for timeline and preview requires `libx264` in `ffmpeg -encoders`.

## Time limits, a departure from the macOS stages

| Limit | macOS stage | ts1 chain |
|---|---|---|
| Timeline render | 43200 s (recovery); 14400 s (failed debug run) | 172800 s |
| Preview render | 14400 s | 172800 s |
| Score | 7200 s | 7200 s |
| Upstream wait | shorter | 7 days at 20 s polls |

The evidence is only these chapter times:

| Run | 01 | 02 | 03 | 04 |
|---|---|---|---|---|
| macOS debug, timed out at 14400 s after 04 | 210.7 s | 189.3 s | 181.2 s | 6739.9 s |
| macOS release, stopped | 122.8 s | 58.8 s | 197.5 s | unfinished after more than 83 min |

Chapters 05-15 have no completed time on any host, and ts1 has none at all. Fifteen chapters at the worst completed time would be 101,098.5 s, about 28 hours, so 12 hours could stop a healthy run. 172800 s is that figure with margin. It is a bound on a hung job, not an estimate or a throughput claim. Preview draws the same frames with lint added, and no completed time exists for it, so it gets the same bound. The resource guard and its policy are unchanged, and a resource stop still ends the job.

## Paths on ts1

Everything is built from the qualifier's `R`, which `samefile` proves is this repository. Paths passed to the binary stay logical, so table strings can equal the macOS strings. The rehearsal has a fake binary case that writes `/data`-style paths, and the comparison stops it.

## What was tested

`checks/rehearse_chain.py` passes all 29 checks in a tiny temporary tree whose repository path is a symlink. Real parts are the chain, both start scripts, the sealed qualifier, `guard01.py` logic, the preserved guard, ffmpeg, ffprobe, PIL and the real cue ids. Fake parts are the binary, the narration verifier, the takes, the macOS run's receipts and `sbh`. In the fixture copies only, the hostname and hash pins change.

One run went through all four stages under the real guard. 12 damaged inputs were refused, and 4 failures stopped inside a job. I also diffed the ported functions against the sealed macOS helpers, and the only differences are the ones listed here plus names and paths.

The real receipt, log and frame lines of the macOS failed run were read against these checks. The `native-narration01` and `workspace` listings match the rows locally, 110 and 60 files.

## Limits

- **The real binary, narration verifier and ts1 tools are unrun by this rehearsal.** The real verifier's file reads were traced earlier, on macOS.
- **A table difference costs the whole timeline run.** The comparison needs all 15 chapters finished. This host has a different architecture from the macOS runs, and table equality across them is untested. A difference stops the chain with the tables kept, and that needs inspection.
- **An extra file on ts1 refuses `--check`.** The message names the first three. The ts1 folders may hold something the rows don't, and I can't see them.
- **The refusal for a child's ffmpeg outside the manifest wasn't rehearsed.** Only the passing path ran.
- **Nothing here is picture, listening, fine sync or AV acceptance.** `pictures_viewed` stays `false` until an actual review is recorded elsewhere.
- **Not independently reviewed.** I wrote this chain.

## Replay

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-remote-ts1-pipeline01/chain01
(cd $P && shasum -a 256 -c SHA256SUMS)
python3 -I -B $P/checks/rehearse_chain.py
```

Run these from the repository root on the local host. The rehearsal needs 10 GiB free, because it uses the real guard's admission.
