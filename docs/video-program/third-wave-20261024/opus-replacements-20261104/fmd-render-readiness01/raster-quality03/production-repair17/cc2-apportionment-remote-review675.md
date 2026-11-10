# CC2 review 675: apportionment remote adaptation 671, 672, 673

Source review only, written 2026-11-09 about 23:10 UTC. I did not use SSH, did not touch or observe the running jobs, and changed no file except this note.

## Independence limit

I wrote the original 528 helper. `run_owned`, `end_group`, `settle`, `chapter` and the identity code in 671 are mine, so this review is not independent on the process code or on those checks. I also read CC0's review 530 before writing, so F5 is anchored on it. Only root's changes get a fresh reading: the explicit CSD root, the ref=5, level and normalization edits in `chapter`, and all of 672 and 673.

## Bytes reviewed

| File | SHA256 | Lines |
|---|---|---|
| `apportionment-csd-support671.py` | `244fe1a47d5a40f71594031bafc09fefb45558c38ed72adbfb6aa1b1f269c40a` | 508 |
| `apportionment-csd-final672.py` | `54262bf86dda15e86d08c23d90f19d58d7d0c0e5c61c466720072646be544f24` | 46 |
| `apportionment-after-timeline673.py` | `89f4bb4e6cb8116a7c0e67fe33f11a88fa1da66e028856cd1f91945ae3f80cf7` | 33 |

Diffs: 671 differs from 668 at line 90 only (the explicit CSD root replaces `parents[7]`). 672 against 669 and 673 against 670 differ in names and pins only. 672:31 sets `o.R` to the same root again, which is harmless.

## Result

No finding calls for interrupting the chain. I found no path that signals an unverified process. The x264 option comparison is correct for what it compares. The main gap is cross-host proof at the master join (F1), which 672:45 already lists as open and which can be checked after the chain.

**Do not edit 671, 672 or 673 while 673 runs.** 673:22-27 re-hashes them before each chapter, so an edit stops the chain at the next chapter. Every recommendation below is a read-only check after the chain or belongs in a fresh numbered controller.

State taken from the TODO top (root's record, not observed by me): final 03 finished 22:38:18 UTC with 5,527 frames rendered in 243.6 s, 04 finished 22:42:21, 05 finished 22:48:11, all exit 0 with no resource stop, and 06 launched. The comparison at 671:381 has therefore passed on real CSD output for at least three chapters. Chapters 01 and 02 were transferred to CSD by 683.

## Findings

### F1, Medium: nothing compares H.264 parameter sets across hosts, and the master joins by stream copy

671:360-381 checks each chapter alone. `master.rs:202-248` copies every chapter's video through concat, then applies `h264_metadata`. Chapters 01 and 02 come from Homebrew FFmpeg 8.1.2 with x264 core 165. Chapters 03 to 15 come from CSD's FFmpeg 8.0.1 and whichever x264 it links.

Equal option text does not prove equal SPS and PPS bytes. Every option that feeds them matches, so I expect identical bytes, but no check establishes that. Concat's `auto_convert` defaults to true (confirmed from `ffmpeg -h demuxer=concat`, Mac 8.1.2), so later parameter sets probably travel in band while the sample entry keeps the first chapter's. That last part is my reading of the path and is untested. A difference could decode in software and still fail on a hardware decoder. `encode.rs` already records Apple rejecting a level 6.0 master.

Reference values, measured at 23:03 UTC with read-only ffprobe on the local Mac movies. Chapters 01 and 02 agree exactly:

- extradata 50 bytes, SHA256 `f8e7fad1d2b8641f47fcfe4408e16ffbf31ab099a289cab4a4273dea68e49f2e`
- High, level 52, `has_b_frames` 2, time base 1/15360

After the chain, run this on all 15 chapter movies and assert equality with those values:

```
ffprobe -v error -select_streams v:0 -show_data_hash SHA256 \
  -show_entries stream=extradata_size,extradata_hash,has_b_frames,time_base -of json CHAPTER.mp4
```

`csd-final-verify647.py:970-989` and `:1022` record `extradata_md5` but never assert it, so a later gate needs its own assert or a reading by hand. A difference confined to VUI colour fields is rewritten by the master's `h264_metadata` filter. Any other difference needs an answer before the join. Then apply the per-chapter and master framemd5 identity from 647, and a hardware decode of the master on the Mac.

### F2, Low-Medium: the tracker can be blind on Linux without saying so

671:109 passes `-o pid=,ppid=,pgid=,lstart=,comm=` as one argument. The procps man page warns that this form may parse as one column depending on personality. My reading of procps-ng source says five columns with empty headers, but that was a summarized source read and I did not see CSD's version.

671:146-149 drops any line that does not split into nine fields. A format mismatch would therefore give `rows == {}` with `missed` still 0. 671:431 and 672:45 record `tracking`, and no gate reads it (673:31-32 checks only the receipt and frames).

Renders and `chapter` are unaffected. Guard stops are still covered by stop02's own `/proc` walk. What would be lost is `settle` and the helper's own timeout path.

Read `tracking` in `final672-summary.json` for 03 to 05. The Mac 528 job saw 2 per chapter, so expect `escaped_seen` of 2 or more and `missed` near 0. If `escaped_seen` is 0 everywhere, the tracker was blind on CSD. A successor should use separate `-o` options and fail when non-empty ps output parses to no rows.

### F3, Low: `lstart` is derived from wall-clock boot time on Linux

As I recall procps, `lstart` is boot time plus start ticks, and boot time moves when the wall clock is stepped. I did not test this on CSD. A step between snapshots changes the string, so `same()` at 671:179-181 fails. The target is logged "another process now" and is not signalled, which is the safe direction.

A descendant still attached is found again under a new key. An orphan is not. stop02 compares raw `/proc` start ticks and is unaffected. A successor should read field 22 of `/proc/PID/stat`.

### F4, Low: a guard stop also sends SIGTERM to the helper's own `/bin/ps`

stop02's `stop_owned` signals every descendant, including a `ps` the helper has just started. That raises `CalledProcessError`. `look()` then counts a miss (671:172-174) and `hit()` logs "ps could not start; no target was signalled" (671:188), which is untrue and makes `cleanup.json` misleading. `alive_targets()` returns `[]`, which ends the grace wait early at 671:231.

Real cleanup is still covered by the guard's pidfd loop. On the positive side, stop02's subreaper and `/proc` walk close the "under one poll before a stop" gap stated at 671:39-41, and `same()` ignores ppid, so reparenting to the guard does not break identity.

### F5, Low: 528's known limits carried into 671 unfixed

CC0's 530 items apply unchanged, because this stop path is mine:

- L1: `settle` at 671:401-404 treats a `ps` failure as nothing left. 672:42 also discards `tracked` on a non-zero exit.
- N1: the sweep can run past 20 s. Under stop02 that means SIGKILL at 20 s and RuntimeError at 30 s, with no `cleanup.json` if the sweep overruns.
- N2: a zombie's `comm` carries `<defunct>`, so an unreaped process is logged as "another process now".
- L2: my 528 notes say the guard's 20 s SIGKILL ends the group after a helper-only SIGKILL. That is wrong, as CC0 said. stop02 does not run in that case. asupersync c48399f `process.rs` has no `PDEATHSIG` or `prctl`, so an escaped encoder then ends only on stdin EOF when the binary dies.

The TODO's "no real ffmpeg stop proof" is true of the 528 job. There is one real stop of the derived code, on the real binary: Mac job `apportionment-final632-03_largest_tails`, file `03_largest_tails.cleanup.json`. It shows reason `signal 15`, child exit -15, one `fmn-bound-ffmpeg` target, SIGTERM sent to nobody with skip reason "another process now", SIGKILL skipped as "gone", 873 polls, 1 unexplained miss. It proves the stop path runs and leaves a record. It does not prove the helper ends a live encoder, since this encoder was already leaving. `<defunct>` as the cause is my inference.

### F6, Low: 4K layout equality is unproven for non-empty tables

671:341-342 requires `layout.tsv` to be byte-equal to timeline666's 64×36 output. 666 records `layout_sha256` but does not compare it with 567. Chapters 03 to 05 have empty layouts, so their passes say nothing here. On the Mac, 4K finished only for 01 and 02, also empty. 540p passed.

Chapter 08 is the first real exercise. A refusal costs one chapter render, stops the chain with no retry, and leaves the files in place. Root can compare these Mac 567 values against `comparison666.json` now, read-only:

| Chapter | Bytes | SHA256 |
|---|---|---|
| `08_signposts` | 300 | `a2367895d6db8949fac273e68ba119dd5fcacbccc4a67d3d27e9c77062cf5eb0` |
| `11_webster_quota` | 157 | `46cf8f06ba3ef7852a4af651cb0dc9d60929cd8681866425ff23d7829936eec4` |
| `12_both_promises` | 718 | `fd5bcaac35fcc882ddcf762228d6ecb1c930847607c5c1e1a80ccf358ec65212` |
| `13_population` | 174 | `2aff6cd54b7fafa388ba6a4d0513f7acdfbd5a6e62ef635de5ba1a8a342dff53` |
| `15_recap` | 340 | `732cb89434b9ca482dd836c52e7f44b86657669b8905ef4e3b84ab22c0fa9c84` |

The other ten chapters are empty at 567.

### F7, Low: the encoder libraries are not bound, and banners are not compared across chapters

The frozen set at `apportionment-after-assembly667.py:25` holds the resolved Python, ffmpeg, ffprobe, stop02, binary and repo, with no libx264 or libav shared objects. Whether CSD's ffmpeg links them dynamically, I could not see. The comment at 671:366 ("binds this host's frozen FFmpeg library versions") overclaims: `ENCODERS` is re-derived from `ffmpeg -version` at every start (672:34-38), and 671:380 discards the build banner.

A package update between chapters would pass unnoticed. After the chain, assert that the 13 `encoder_settings` strings and `ffmpeg_version` values are identical.

## Smaller points

- Thread counts differ by design (Mac 54 and 9, CSD expected 6 and 1 from the four-CPU affinity, not observed), so same-CRF bitstreams are not comparable across hosts.
- No check compares pixels across architectures. CSD final pictures need their own review. A short CSD render of chapter 01 or 02 would give a same-frame comparison against the Mac 4K, but 672:27 refuses those chapters, so it would need its own controller.
- Soundtrack WAV bytes are checked for format and length only (671:390-391), never across hosts.
- 672:13-14 executes the module before asserting its hash. The guard manifest already binds that file.
- 672:17 lost `read_receipt`'s retry against the guard's non-atomic `save()`. The window is tiny and a failure comes before any render.
- 671's docstring is stale (540p, launchd). `pressure()` is unused, so CSD has no process-pressure check.
- After any failure 673:26 refuses a restart, and 673:28 writes the inputs file before admission, so a resume needs a fresh numbered controller.

## Checked, no defect found

- **Time limits.** Chapter 12 needs 0.94 fps to fit 14,400 s. The recorded 17 to 23 fps (04: 243 s and 05: 350 s finish to finish, including checks) puts it near 13 minutes, about 18 times inside. The 1,800 s decode limit at 671:383 is also clear. Decode fit inside 05's 350 s, so chapter 12 needs at most about 745 s at that rate.
- **Colour.** 671:361 checks range and space only, which is right for chapters. Mac 01 reports `color_primaries` and `color_transfer` as unknown (ffprobe with optional fields shown, 23:04 UTC) although `negotiate.rs:339-345` passes bt709 flags. `master.rs` sets all three with `h264_metadata`. Assert primaries and transfer on the master. Do not add them to `chapter`. I did not trace why the flags do not appear in the chapter.
- **Identity.** Pid, pgid, start and comm are all compared, with no name matching. `killpg` is used only on the helper's own group, which 671:138 asserts it leads.
- **Linux spawn.** `host_exact_image_mechanism` uses the same new-group spawn on Linux as on macOS (fmn-platform a399d62:1818-1823).
- **Parsing.** `lstart` is five tokens under `LC_ALL=C`, so `split(None, 8)` gives nine fields. Targets are keyed by pid and start and overwritten each poll, so a row first seen between the spawn's setpgid and exec is corrected by the next poll.
- **Stopped binary.** SIGTERM stays pending until the SIGCONT at 671:224 (kernel semantics, untested).
- **Normalization.** The lookbehind at 671:376 excludes `sliced_threads` and matches `threads` and `lookahead_threads` once each.
- **Level.** Level 5.2 clipping ref 10 to 5 matches the Mac 4K SEI, and 671:362 asserts level 52.
- **Encoder tags.** 672:36 takes the first version tuple, the compile-time one, which is the source of the file tags.
- **Inputs.** The narration path is the same in 666 and 672, a `current_dir` search of kit cli, kit, narration and master found no use, and the paths are absolute, so the cwd difference does not reach them.
- **Disk.** 4K jobs are far under the 4 GiB per-job cap (Mac 5.09 and 8.44 MB movies).

## Not checked, and no claims

Not checked: CSD's real `ps` output, real banner, extradata, linked libraries and summaries; the jobs that have finished or are running; how the binary locates its ffmpeg (the tag check at 671:366 would refuse a different version); voice identity beyond the shared narration path. Nothing here was run on CSD.

This note accepts no render. It makes no listening, picture, motion, 4K, fine-sync, caption-playback or independent audiovisual claim.
