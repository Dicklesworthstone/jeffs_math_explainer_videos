# pictures1008 70dfce04: independent review

CC2 (RubyRaven), 2026-11-10, about 23:40Z. This review covers CC1's sealed candidate `pictures1008.py` `70dfce04`, its handoff `768ab14f`, test1008 `01c79efa`, pin1008 `c615fdd5` and pictures1008-serial01.sh `dedc8638`. It was checked against verify1005 `d96795c4`, pictures1007 `cf3e9f16` and explainer_kit `master.rs` `join()`.

**What I did not do:**
- I launched nothing, pinned nothing, staged nothing and wrote nothing on CSS.
- I made no Beads edits.
- No production media was decoded beyond ffprobe's first frame per file.
- I compared JPEGs by bytes and by decoded RGB only. I did not look at any image. This is not picture, motion, listening, fine-sync or AV acceptance.

## Verdict

No blocker. 1008 can replace 1007 once verify1005 is terminal and root pins line 57.

Each chapter job decodes its own chapter file once. In total that is at most 164,420 frames instead of 1,280,843, which is about 2.4 h instead of about 18.6 h at verify1005's measured 19.16 fps.

These are kept:
- the 3163 samples and 402 sheets;
- the joined-master frame numbers, reasons, labels and master sha256;
- the verification pin and the refusal while it is unset;
- the consumed closure against the verify1005 manifest.

## Root's colour question

Equal yuv420p framehashes prove equal decoded planes, not equal JPEG conversion. So I checked the tags that reach the conversion.

### Production probe

This was read-only on CSS at 23:27Z and 23:32Z. CSS has ffmpeg `8.0.1-3ubuntu2`. Two runs:
- `css_probe01`: stream headers, extradata hash, and the first decoded frame of each file;
- `css_probe02`: 1008's own `probe_argv()`, fed to 1008's own `stream_match()`.

| File | range | matrix | transfer | primaries | pix_fmt, chroma, depth, order | extradata |
|---|---|---|---|---|---|---|
| master | tv | bt709 | bt709 | bt709 | yuv420p, left, 8, progressive | `09351dc0` |
| all 15 chapter files | tv | bt709 | unset | unset | yuv420p, left, 8, progressive | `caf9d50d` (all 15 equal) |

The stream headers and the first frame agree. Each file has exactly one h264 video stream and one aac audio stream, with no attached picture.

The transfer and primaries difference comes from the join. `master.rs` `join()` remuxes each chapter video-only, concatenates them, and then applies `-bsf:v h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1`. That rewrites the master's SPS colour description, and the extradata changes with it. CSS's repo copy has the same line, `master.rs:244`.

With 70dfce04's rule (equal STREAM_KEYS; transfer and primaries unset in the chapter or equal to the master's), `stream_match()` passes for all 15 real pairs. Trial 3578d8c0 required all tags to be equal and would have refused all 15 after its probes. CC1 had already changed this before my heads-up mail.

### Fixture

`fixture01.py`, run with local ffmpeg 8.1.2:
- **Chapters:** two synthetic 4K chapters of 97 and 131 frames, libx264 High with B-frames plus AAC. Their tags are the CSS pattern: range tv, matrix bt709, transfer and primaries unset.
- **Master:** built with `join()`'s exact argv (video-only parts, concat demuxer, wav, the `h264_metadata` rewrite, AAC 256k, faststart). The probes reproduce the CSS table exactly.
- **Identity:** verify1005's identity decode and 1008's `framehash_frames()` show each chapter equal to its master slice.
- **Extraction:**
  - the 1007 path is verify1005 line 128's argv on the master, with joined-master frame numbers;
  - the 1008 path is 1008's own `extract_argv()` and `select_filter(local_frames())` on the chapter file.

Results, run against both 3578d8c0 and 70dfce04:

| Case (1008 path vs 1007 master path) | JPEG bytes equal | decoded RGB equal |
|---|---|---|
| chapter 1, offset 0, 7 picks (keyframe, B-frames, GOP edge, last) | 7/7 | 7/7 |
| chapter 2, offset 97, 13 picks | 13/13 | 13/13 |
| control: chapter 2 retagged range pc | 0/13 | 0/13 |
| control: chapter 2 retagged matrix bt470bg | 13/13 | 13/13 |
| control: chapter 2 retagged bt2020 primaries, PQ transfer | 13/13 | 13/13 |
| control: chapter 2 retagged bt709 like the master | 13/13 | 13/13 |
| explicit match: `setparams=...bt709...:range=tv,` before select | 13/13 | 13/13 |

Every retagged copy keeps framehash planes equal to the master slice. The range control shows the comparison detects a conversion tag that matters. Of the tags tested, range is the only one that changes the bytes. `stream_match()` holds range, matrix, pix_fmt, chroma location, depth and field order equal. The transfer and primaries difference, which is the only difference in production, does not change the conversion output in 8.1.2.

So the conversion metadata that matters matches, and no explicit matched conversion is needed. If root wants the result independent of ffmpeg version, the tested option is a one-line change: prefix `select_filter()` with `setparams=color_primaries=bt709:color_trc=bt709,`. The recheck already rebuilds the filter from that function. I do not require it.

## The five points from Mail 24832

**1. Identity binding.**
- `chapter_identity()` requires exactly one `chapter_frame_identity` record at the chapter's source, offset and frame count.
- The chapter file's hash must be equal in the manifest row, in that record and on disk.
- Both framehash files must be plain files at the sha256s recorded in verification1005.json.
- Verify1005's own `frame_sequence()` must show chapter equal to `master[offset:offset+frames]`.
- `gather()` also requires the identity list to equal `chapters` in order.

On CSS, `VERIFY/` already holds `master-video.framehash` and the chapter framehashes up to 13. verify1005 had not finished at 23:32Z.

**2. Extraction argv.** It is verify1005 line 128 with only the input changed. My `check1008.py` evaluates both and compares them.
- `-map 0:v:0` is not needed: every chapter file has one video stream, and the files are hash-bound.
- One note on wording: `stream_match()` probes with `-select_streams v:0`. So "need exactly one video stream" can't count a second video stream. My all-stream probe covers this for the 16 files as bound.

**3. Records keep master numbering.**
- Frames, reasons, sheet frames, `masterframe{F}` labels and `F/60` times are joined-master values.
- `local_frame`, `extract_source` and `stream` are added, and only `select.filter` holds local indices.
- test1008 B shows 1008's union equal to 1007's over the 15 real seeds: 3163 samples and 402 sheets.

**4. Closure.**
- The chapter file joins `consumed`. ffmpeg, ffprobe, the master and the chapter files are all verify1005 manifest rows at the same hash (`css_probe02`).
- The master stays consumed and hashed. It is 216.6 MB, so the repeated hashing is negligible.
- 1008 refuses any chapter with a 1005, 1006 or 1007 job: in gather, in finished and at run. No picture jobs exist yet.
- `finished()` binds the helper, the pin, the movie, the seed, the chapter file and both framehashes.

**5. Scope.** `SCOPE1008` states that the stills come from the chapter file, proven identical to the master slice.

## Tests

**test1008.** I ran it from a copy, `python3 -I -B test1008.py cc2run1`: ALL PASS. My log is byte-identical to CC1's run3 log (`3de10496`). It covers the A–E sections in the handoff, including:
- 15/15 identity refusals;
- 7/7 recheck refusals;
- the local+1 and global-index controls.

**pin1008.** It differs from pin1007 only by the candidate sha, the names, and line 57 (`VERIFICATION_SHA256=None`). Its live check refused at 23:32:40 while verify1005 is running.

**serial01.** It differs from 1007's only in names. It also prints the recheck commands.

## Findings

- **F1, fixed before the seal.** Trial 3578d8c0's strict tag equality refused all 15 production chapters. 70dfce04 passes 15/15 on the real probes.
- **L1, Low, residual.** The JPEG equality was measured with ffmpeg 8.1.2. CSS has 8.0.1, so equality on CSS is not shown. Two ways to close it, both root's choice:
  - run `fixture01.py` on CSS in a scratch directory (synthetic, about 60 MB, about 40 s);
  - take the `setparams` prefix above.
- **L2, Low, wording only.** See point 2: the probe's stream count claim. No change needed for these files.

## Corrections

- My Mail 24832 gave its CSS probe time as "about 23:40Z". It was about 23:19Z.
- In my scratch, `pictures1008-a6a87231.py` holds `3578d8c0`, because CC1 edited the file before my copy. I read a6a87231, the draft the handoff calls lost, only as a diff against 1007, at about 23:23Z. It had no stream check.

## Files (CC2 scratch, `pictures1008-review01/`)

| File | sha256 |
|---|---|
| css_probe01.py / .out.json | `9ee996e5` / `8bc8cfa7` |
| css_probe02.py / .out.json | `da08f753` / `7f82b747` |
| fixture01.py | `ce68def7` |
| fixture01-run1 (3578d8c0) / run2 (70dfce04) result.json | `064d43ae` / `da93ced5` |
| check1008.py / check1008-copy3.json | `66af9b6f` / `146c3e79` |
| cc1-test-copy/cc2run1.log | `3de10496` |
| pictures1008-copy3.py | `70dfce04` |
