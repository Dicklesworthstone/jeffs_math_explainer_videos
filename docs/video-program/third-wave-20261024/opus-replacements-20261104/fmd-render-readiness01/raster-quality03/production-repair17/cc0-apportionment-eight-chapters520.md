# 520: apportionment preview458's finished chapters 01–08 against timeline444 (CC0)

cc0, 2026-11-09, 06:20 UTC. Q is this folder. W is `narration_lab/third-wave-20261024/root/five-film-release-20261027`. The evidence is `cc0-apportionment-eight-chapters520.json`.

All checks were read-only:
- sha256 of each file;
- ffprobe headers plus a packet count (demux only, nothing decoded);
- the x264 option string, read from the movie bytes;
- WAV headers and byte comparisons.

There was no decode, extraction, render, launch or SSH. Failed 458 and every file are preserved.

## Result

**Chapters 01–08 of the failed preview458 agree with timeline444 on every metadata check.**
- **Frames:** in each chapter, the frame counts in timeline444's summary, both movies' headers and packets, and 458's log all agree.
- **Video:** 60/1 fps; h264 High at yuv420p, 960×540, crf 16.
- **Audio:** AAC LC, 48 kHz stereo, with packet counts and durations equal to timeline444's.
- **Side tables:** `timeline`, `cues` and `layout` are byte-equal to timeline444's.
- **Inputs:** all 3,820 rows of `preview458-inputs.json` (`5817cb64`, the receipt's hash) still match, 1.57 GB re-hashed at 06:16:41Z.

**Why 458 failed.** 458 (receipt `a8d25c31`) exited 1 at 05:48:55Z. The guard admitted it and recorded no resource stop. After 08 finished, the binary could not spawn its ffmpeg helper for 09: os error 35, Resource temporarily unavailable. As a result:
- 09–15 have no files of any kind;
- `preview458-summary.json` was never written;
- 458's own check that all 15 timelines match was never run. This note runs that check for 01–08.

## Differences (send to CC2 / root)

- **D1. The cuelog has one extra row per chapter: the score stem.**
  - Every timeline444 row is present, in the same order.
  - The extra row is `score445/score/stems/<chapter>.music.f32.wav`. It starts at sample 0, at 0 dB gain, and runs the chapter's full length.
  - Each stem matches its row in the 458 manifest.
  - This is expected: 458 used `--score` and 444 did not.
- **D2. The soundtrack bytes differ from timeline444's in all 8 chapters, but format and length are the same.**
  - The difference follows from D1.
  - Format: 48 kHz, stereo, 16-bit, chapter frames × 800 samples.
  - AAC packet counts match.
  - I did not decode or compare the audio content.
- **D3. In `04_sixteenth`, the video header is 0.33 ms short.**
  - `duration_ts` is 1,084,923 against 4,238 × 256 = 1,084,928 (70.633008 s against 70.633333 s).
  - The last packets' pts, dts and durations equal timeline444's, and so do the frame and packet counts. Only the header duration differs.
  - The other 7 chapters are exact.
- **D4. Two differences are by design:**
  - the size (960×540 against 64×36);
  - x264 `threads` 17 vs 1 and `lookahead_threads` 2 vs 1. All other x264 options are identical.
- **Retries.**
  - 01 took 2 attempts, 06 took 5 and 07 took 2, each after a transient Darwin EPERM following ffmpeg's exit.
  - The movies on disk come from the last attempts.
  - The earlier 06 and 07 soundtracks, in `soundtrack/previous/*.v1`, are byte-identical to the current ones.
- **Colour tags.** These match timeline444's. Both tag range tv and bt709 matrix only, leaving transfer and primaries unset.

## Movies (sha256)

| Chapter | Frames | Bytes | sha256 (first 16) |
|---|---|---|---|
| 01_one_more_seat | 4,806 | 3,123,069 | `0d30be361dafad25` |
| 02_shares | 6,647 | 4,548,306 | `921d635c45938302` |
| 03_largest_tails | 5,527 | 3,670,435 | `23a9cc1a4a2d5c26` |
| 04_sixteenth | 4,238 | 2,892,911 | `d6e69f37f152f56a` |
| 05_dials | 6,316 | 4,428,309 | `5499f041ca1e3775` |
| 06_not_every_time | 4,947 | 3,501,326 | `06a69e2269db1a0f` |
| 07_yardstick | 5,049 | 3,424,936 | `eaf0b19071289e99` |
| 08_signposts | 6,448 | 4,522,357 | `cbec602aa1cc7155` |

The JSON has the full hashes of the movies, soundtracks and tables. Its sha256 is `8995aa7d525c0a1dddba364b88a6888bf790c5133531a1ad0c33eb45a1ca7c1f`.

## Not checked

- any decoded frame or sample;
- the AAC audio against `soundtrack_0.wav`;
- loudness;
- 09–15;
- CC2's 516 helper, and how a resumed run would bind these files.

No audio heard or checked. This is metadata only. It accepts no picture, motion, sound, sync or AV result.
