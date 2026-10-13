# The Algorithm Lamport Is Proudest Of

Video C explains the bakery algorithm through customers, slates and a room:
take a ticket, break ties by customer number, and show a choosing flag until
the ticket is ready. The proof survives a read overlapping a write that
returns garbage. The later CPU chapter separates that register model from
real memory ordering.

**Status: final cut (review 3). The owner has authorized publication; it is not
public yet.** The cut below was built on 2026-10-12 from the frozen narration
(report `b142acacd3eb1aaa17ccd3e762917c1d`), with picture and mux at
`7767d25a77c7b2f540166a3f637018d06d750a53`. Publishing is the upload lane's step;
the YouTube title, description and tags are in [publish/](publish/youtube.md).

## Watch and inspect

Production artifacts are local and gitignored:

- `renders/final/bakery_algorithm_master.mp4`: the joined 4K master.
- `renders/final/master/bakery_algorithm.master.wav`: its PCM master.
- `renders/final/bakery_algorithm.en.srt` and `.en.vtt`: sentence captions.
- `renders/final/cue_sheets/`: the final chapter cue sheets.
- `stems/qa/`: the score measurements for each chapter.

The MP4's MD5 is `9ed06e7d12d38a87d748d8a58806d296`. A private review
copy of the master, PCM, chapters, captions, cue records, sheets and score QA
matched source MD5s before and after transfer. The two earlier review cuts are
kept beside it: `renders/final_review1/` (`19f2e859c0ee1a0841c0115ac69ab656`)
and `renders/final_review2/` (`7df3679f4f52d3e5844425fb7b868048`).

The master measures **−17.6 LUFS at −1.5 dBTP**. The verifier passes it through
its fixed-gain peak-ceiling exception; it does **not** meet the literal
−16 ±1 LUFS requirement in [AGENTS.md](../../AGENTS.md). The peaks occur in
narration throughout the film. The owner said the latest michael cuts "sound
pretty good" and then authorized publication; he made no separate statement
about the level. Narration has not been time-stretched, pitch-shifted or
loudnormed.

## How this cut was made

| Item | Recorded result |
|---|---|
| Animation engine | franken_manim `a399d626c43ecc852a5c795288979f52b446bc5c`, pinned git dependency |
| Rust toolchain | `nightly-2026-08-31` |
| Narration | Michael; FrankenTTS 0.1.11, FrankenWhisper 0.10.0 |
| Voice inputs | 85 lines, 218 sentences; current line ASR mean 0.997 |
| Render host class | Mac build host; software x264 |
| Encoder | H.264 High, x264 CRF 16, preset slow for every chapter, including the camera chapter |
| Encoder check | Chapter 09 (the static, the hardest picture) against a lossless render of the same source: PSNR median 63.4 dB, worst frame 48.9 dB; SSIM median 0.99991, minimum 0.9995 |
| Picture | 3840 × 2160, 60 fps, all three BT.709 tags on the master |
| Duration | 56,617 frames; 943.6 seconds (15:43.6), 13 chapters |
| Longest chapter | 84.80 seconds |
| Chapter render batch | 1,149 seconds elapsed, four jobs concurrently (part of it waiting on a shared build lock) |
| PCM assembly | Exact logged cues replayed through the pinned fmn mixer; all 13 replays null against captured soundtracks bit for bit |
| Master gain | One fixed gain, +2.90 dB; the −1.50 dBTP ceiling binds |
| Final verification | A/V delta 0.000 seconds; producer's software and VideoToolbox full decodes both return zero errors |
| Captions | 252 captions covering the complete script; SRT and VTT text and timing agree |
| Score QA | All 13 chapters pass; median voice-minus-music 19.31–21.80 LU, maximum SII loss 0.0017, zero clicks |
| Picture review | All 13 final cue sheets, 20 sheets sampled from the joined master and the boundary frames of the three re-read lines; no material picture finding |
| Final-audio ASR | A full speech-recognition pass over the joined master against its captions clears the repaired reads; no window dropped |

File footprint is measured from this cut, not estimated:

| Artifact | Bytes | MiB |
|---|---:|---:|
| Joined MP4 | 130,474,729 | 124.43 |
| PCM WAV | 362,348,880 | 345.56 |
| All 13 chapter MP4s | 131,138,805 | 125.06 |
| SRT and VTT together | 39,576 | 0.04 |

These measurements establish source, timing, image and signal checks. They do
not substitute for a full watch or the listening protocol on actual devices.

## Reproduce and check

Run from the repository root on the configured build host, following the
toolchain and private target-directory instructions in
[AGENTS.md](../../AGENTS.md). The driver preserves earlier takes and outputs.

```sh
./x bakery_algorithm test
./x bakery_algorithm facts-check
./x bakery_algorithm lint
```

For the reviewed source, tests pass (56 unit and 6 integration), clippy and fmt
pass, and 217 computed video facts agree with 294 oracle keys. Narration
validation and complete sentence-edge coverage pass on the current WAVs.

After approved narration is available, the production sequence is:

```sh
./x bakery_algorithm narrated
./x bakery_algorithm score
./x bakery_algorithm final
./x bakery_algorithm master
./x bakery_algorithm captions
./x bakery_algorithm verify
```

Coordinate with the pipeline owner before running production jobs. Use the
pinned dependency; a local or dirty animation-engine checkout cannot qualify
a final.

## Claims and remaining acceptance

The [storyboard](docs/storyboard.md) defines words and pictures. The
[fact sheet](docs/fact-sheet.md), [claim walk and dispositions](docs/claims.md),
[description and runnable code](docs/description.md), and
[research scripts and records](../../docs/research/bakery_algorithm/) explain
the assumptions and sources. Finite model searches and measured CPU stress
runs support their stated cases; neither replaces the general mathematical
proof or proves a memory-ordering implementation correct.

- [x] Current source tests, facts/oracle, narration binding and sentence-edge checks.
- [x] Fresh score QA, 4K chapters, PCM null tests, caption coverage and checksum copy.
- [x] Fresh final cue sheets and sampled joined-master frames inspected.
- [x] Final-audio speech-recognition pass over the exact master.
- [x] The owner said the latest michael cuts "sound pretty good" and authorized publication (2026-10-12).
- [ ] An uninterrupted full watch on a real device: none is recorded.
- [ ] Published on YouTube by the upload lane (4K processing and captions confirmed).

Review evidence is on `mx-e-c-7v0.18`; final integration is
`mx-e-c-7v0.15`, and the listening protocol is `mx-e-c-7v0.13`.
Only the orchestrator closes the production beads.
