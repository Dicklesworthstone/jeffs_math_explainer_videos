# pictures1008: per-chapter decode successor to pictures1007 (CC1 scratch, 2026-11-10 23:40 UTC)

**Status.**
- Candidate for review by root and CC2. Nothing was pinned, staged, launched or written on CSS.
- CSS access was read-only: one script reading recorded probe JSON, one `ffmpeg -version`, and `pin1008.py check`.
- `pin1008.py check` refused at 23:32:40 because verify1005 is still live (pids 563272, 573582 and ffmpeg 3297480).
- The fixture encodes and decodes were local and synthetic. No production media was decoded.

## Files (scratch: `/private/tmp/claude-501/-Users-jemanuel-projects-jeffs-math-explainer-videos/b0a72b9b-6c8d-4794-942e-44d50a9d8626/scratchpad/`)

| File | sha256 |
|---|---|
| **pictures1008.py** (278 lines, candidate) | **70dfce04fc178332d0ef9928dcbc11c35616e234be7e33f5a52eb01fcdc01b13** |
| pictures1008-trial1-3578d8c0.py (earlier trial: strict transfer/primaries match; tested in runs 1–2) | 3578d8c0409887b5f5244e8af5e97b1bcc18de75a8ab95098d74809f73807c10 |
| test1008.py (run3) | 01c79efa94ee9e434318b82e90cc3b66abc33c2b8343bbd011e4d239f51532f4 |
| test1008-run3.log, `test1008-run3/` (fixture, mutants, `diff-1007-1008.txt`, `recheck-fixture.json`) | see SUMS |
| pin1008.py (derived from pin1007 by `make-pin1008.py`, 13 counted substitutions) | c615fdd56ef31363d172c038a41f823f5d9127d7271f3ddc43fce8afc3a82f02 |
| pictures1008-serial01.sh | see SUMS |
| streams-css01.py / streams-css01-out1.json / streams-compare01.py | see SUMS |

Earlier runs are kept as well:
- run1: `test1008-run1.log`. It stopped on an over-strict test assertion, with the colour-control result already printed.
- run2: `test1008-run2.log`, run with `test1008-run2-version.py`. All passed against trial1.
- `test1008-draft0.py` is the test before it was trimmed. All files are hashed in `pictures1008-readiness-SHA256SUMS.txt`.

## The change (diff against 1007: −73 / +139 lines)

1. **Per-chapter decode.**
   - Each chapter job reads `OUT/KEY.mp4` with `select.filter` built from local indices, which are the master frame minus the chapter offset.
   - `extract_argv()` equals verify1005 line 128 and 1007 line 211 except for the input. The test checks this by evaluating those source lines.
   - The decode drops from 1,280,843 frames (about 18.6 h at 19.16 fps) to 164,420 (about 2.4 h). The largest chapter is 07, at 14,351 frames (about 12.5 min).
2. **Identity re-proved, not trusted.** `chapter_identity()` requires all of these:
   - exactly one `chapter_frame_identity` record, naming `OUT/KEY.mp4` at the chapter's offset and frame count;
   - the chapter file's sha256 equal in the verify1005 manifest row, in the record and on disk;
   - `KEY.framehash` and `master-video.framehash` present as plain files, at the sha256s that the pinned `verification1005.json` records;
   - verify1005's own `frame_sequence` (lines 95–102, byte for byte) applied to both, with chapter equal to `master[offset:offset+frames]`.

   It runs in `gather()` at make-inputs and at run, and for all 15 chapters in the recheck.
3. **Colour and conversion** (root's point).
   - The job runs `ffprobe` on the chapter and on the master: verify1005's bound `/usr/bin/ffprobe` (a6dac1e9), as run_step steps `probe-source` and `probe-master`.
   - `stream_match()` requires equal codec, profile, pix_fmt, color_range, color_space (matrix), chroma_location, size, rate, field order and bit depth, plus yuv420p at 3840×2160.
   - The transfer and primaries tags must each be unset in the chapter or equal to the master's.
   - **Why not strict on all tags:** the recorded final983 probes on CSS show all 11 conversion keys equal in all 15 chapters. But `color_transfer` and `color_primaries` are unset in every chapter, while the master has bt709. Strict matching (trial1) would refuse all 15 chapters.
   - The fixture mirrors that tag difference, and its JPEGs are byte-identical. Root and CC2 should decide whether this rule is acceptable.
4. **Global metadata kept.**
   - Plan, record and sheet labels keep the joined-master frame numbers, the reasons and `master_sha256`.
   - Each sample gains `local_frame`. The plan and record gain `extract_source`; the record gains `stream`.
   - 3163 samples and 402 sheets: B checks the 15 real seeds, and the local frames equal each seed's own frames.
5. **Unchanged:**
   - the pin and refusal while it is unset;
   - the consumed closure (the chapter file is added; the master, ffmpeg and ffprobe stay);
   - `selection()`, module(), stable() and balanced();
   - the estimate, the 3.5 GiB and 8 GiB checks, and the guard receipt and run setup;
   - the sheet code (1007 lines 209 and 212–219 byte for byte), the size cap and the frozen recheck;
   - the recheck's full verify1005 manifest rehash and its exact record, plan, filter, picture and sheet checks.
6. **No mixing.** 1008 refuses any chapter that has a pictures1005, 1006 or 1007 job. Names: `css-exponential-master-pictures1008-KEY` writes `pictures1008.json`; `css-exponential-pictures1008-recheck` writes `recheck1008.json`.

## Evidence (test1008 run3, exit 0; local ffmpeg 8.1.2)

**Fixture.** Three 4K H.264 chapters (23, 41 and 17 frames; `-g 10 -bf 2`; range tv and matrix bt709, transfer and primaries unset) were joined with concat `-c copy`. The master gained bt709 primaries and transfer through `h264_metadata`. The identity was built with verify1005's framehash argv.

**Mapping.** At offsets 0, 23 and 64, all 22 pictures read back their painted global frame numbers, and 22 of 22 JPEGs are byte-identical to the 1007 path from the master.

**Controls.**
- local+1: 9 of 10 pictures, reading back the wrong frames.
- Global indices on the chapter file: 4 of 10.
- Range-flag-only remux: framehash planes equal, but the JPEGs are 0 of 10 equal. `stream_match` refuses on color_range and pix_fmt.
- A smpte2084 transfer tag: refused.

**Identity refusals (15/15).**
- a missing chapter or master framehash;
- an edited framehash;
- a consistent forgery;
- a real one-frame-off chapter, honestly hashed;
- a missing or duplicate record;
- a wrong offset, frame count or source;
- another chapter's bytes;
- a manifest row with another hash;
- a changed master pin;
- a symlinked framehash;
- a truncated, re-pinned master.

**Recheck.** It passes over the three fixture jobs and refuses all of these:
- the global (1007) filter;
- a plan local_frame off by one;
- a record local_frame off by one;
- a changed extract_source;
- a changed stream;
- a coexisting 1007 job;
- inputs bound to another chapter-file hash.

The tree was restored exactly (snapshot equal).

## Use (after review, and only after actual verify1005 terminal success)

1. `ssh css /usr/bin/python3 -I -B - check < pin1008.py`. It is read-only and prints the `verification1005.json` sha256.
2. Stage candidate 70dfce04 on CSS under a fresh name. Then `pin1008.py write <staged> $Q/pictures1008.py <sha>`. It is exclusive and changes only line 57.
3. `sh pictures1008-serial01.sh` in one tmux session runs make-inputs and the guarded run for each chapter, serially, stopping at the first failure. Then run the recheck; the commands are printed at the end.
4. The guard is unchanged (4 GiB growth, one lane). A refused or failed chapter burns its 1008 name, the same as 1007.

## Limits and checks not run

Not run:
- the real `gather()`, `finished()` and `main()` end to end. They need CSS paths, a 15-chapter qualification, the TOOL_PINS files and Linux-only `sched_setaffinity`. Their new parts were run as functions on the fixture.
- the guard path;
- fmt or lint tools.

Other limits:
- **ffmpeg version.** JPEG byte identity and the tag-only result come from local ffmpeg 8.1.2; CSS has 8.0.1. Production equality is not shown. verify1005's identity covers the decoded planes; the stream check covers range and matrix.
- **Production probes.** The probe values above are final983's recorded probes. The job probes the hard-linked chapter file itself at run time.
- **ffprobe may decode a few frames** while probing a stream; that happens inside the guarded job.
- **Lost draft.** The first, untested 1008 draft (written before the stream check) was edited in place and not kept.

**Authorship:** I wrote 1006, 1007, 1008, pin1007/pin1008, both serial scripts and test1008. None of them has had an independent review.
