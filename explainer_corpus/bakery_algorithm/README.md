# The Algorithm Lamport Is Proudest Of

Video C explains the bakery algorithm through customers, slates and a room:
take a ticket, break ties by customer number, and show a choosing flag until
the ticket is ready. The proof survives a read overlapping a write that
returns garbage. The later CPU chapter separates that register model from
real memory ordering.

**Status: final candidate; owner acceptance and human listening remain open.**
The candidate below was built from `0c5aa67124a7ee9630c70bd7fc71ed9cc6a5a17f`
on 2026-10-12. It has not been published.

## Watch and inspect

Production artifacts are local and gitignored:

- `renders/final/bakery_algorithm_master.mp4`: the joined 4K candidate.
- `renders/final/master/bakery_algorithm.master.wav`: its PCM master.
- `renders/final/bakery_algorithm.en.srt` and `.en.vtt`: sentence captions.
- `renders/final/cue_sheets/`: the final chapter cue sheets.
- `stems/qa/`: the score measurements for each chapter.

The MP4's MD5 is `19f2e859c0ee1a0841c0115ac69ab656`. A private review
copy of the master, PCM, chapters, captions, cue records, sheets and score QA
matched source MD5s before and after transfer.

The master measures **−17.6 LUFS at −1.5 dBTP**. The verifier passes it through
its fixed-gain peak-ceiling exception; it does **not** meet the literal
−16 ±1 LUFS requirement in [AGENTS.md](../../AGENTS.md). The peaks occur in
narration throughout the film. This quieter result needs the owner's explicit
acceptance. Narration has not been time-stretched, pitch-shifted or loudnormed.

## How this candidate was made

| Item | Recorded result |
|---|---|
| Animation engine | franken_manim `a399d626c43ecc852a5c795288979f52b446bc5c`, pinned git dependency |
| Rust toolchain | `nightly-2026-08-31` |
| Narration | Michael; FrankenTTS 0.1.11, FrankenWhisper 0.10.0 |
| Voice inputs | 85 lines, 218 sentences; current line ASR mean 0.997 |
| Render host class | Mac build host; software x264 |
| Encoder | H.264 High, x264 CRF 16, preset slow for every chapter, including the camera chapter |
| Picture | 3840 × 2160, 60 fps, all three BT.709 tags on the master |
| Duration | 56,508 frames; 941.8 seconds (15:41.8), 13 chapters |
| Longest chapter | 84.6167 seconds |
| Chapter render batch | 714 seconds elapsed, up to five jobs concurrently; individual logged render times total 1,510.3 seconds |
| PCM assembly | Exact logged cues replayed through the pinned fmn mixer; all 13 replays null against captured soundtracks bit for bit |
| Master gain | One fixed gain, +2.93 dB; the −1.50 dBTP ceiling binds |
| Final verification | A/V delta below 0.001 seconds; producer's software and VideoToolbox full decodes both return zero errors |
| Captions | 252 captions covering the complete script; SRT and VTT text and timing agree |
| Score QA | All 13 chapters pass; median voice-minus-music 19.38–21.70 LU, maximum SII loss 0.0025, zero clicks |
| Picture review | All 13 final cue sheets plus 189 frames sampled every five seconds from the joined master; no new material visual finding |

File footprint is measured from this candidate, not estimated:

| Artifact | Bytes | MiB |
|---|---:|---:|
| Joined MP4 | 130,417,002 | 124.38 |
| PCM WAV | 361,651,280 | 344.90 |
| All 13 chapter MP4s | 131,101,996 | 125.03 |
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
- [ ] Owner disposition of the quieter fixed-gain master.
- [ ] Signed human narration/score listening and full-watch acceptance.
- [ ] Owner approval to publish.

Review evidence is on `mx-e-c-7v0.18`; final integration is
`mx-e-c-7v0.15`, and the listening protocol is `mx-e-c-7v0.13`.
Only the orchestrator closes the production beads.
