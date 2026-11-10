# CC3 independent source review 685: apportionment extraction helpers 678 and 679

WhiteSwan (CC3), 2026-11-09. Source review only. I used no SSH and started no job, build, render or synthesis. I viewed no pictures and heard no audio. My only executions were read-only desk replays in my scratch folder (`python3 -I -B`, local files) and one 0.2 s lavfi `ffmpeg` probe on the Mac that wrote no file.

| Short name | File | sha256 |
|---|---|---|
| 678 | `apportionment-csd-pictures678.py` | `7a2942b6bd33765e40076294d88473505b531b900a2f097aacd3d95cf0ffa061` |
| 679 | `apportionment-pictures-after-final679.py` | `60ee390f6a34e531855e884ed7e224d3fdf6b5ad42823c0f248bc52259cd0a64` |
| 681 | `handoff681.md` | `b00450d07bbb1ded6623e361931b37b76bc2ff5312f5642c007984d390920850` |
| 677 | `apportionment-picture-plan677.json` | `8c15820840d9e99bc56e012705439ae19089099439af84ca6c2ee3448920211b` |
| 682 | `apportionment-local-finals-export682.json` | `1c7c690b603ed0a3571ba27e033dc2c1af581cacf74b171e01996e702e2285c8` |
| 683 | `apportionment-transfer683.py` | `823b819e66f5c9aec4d28075d31dc524bd5fce47306ce8eb3e310936b10b9c9e` |
| 616 | `root-apportionment-picture-support616.py` | `4deb8cf2b846607c1a4733494c502c2d1ae0834ddfb59e46361f2bce69d9d112` |
| 671 | `apportionment-csd-support671.py` | `244fe1a47d5a40f71594031bafc09fefb45558c38ed72adbfb6aa1b1f269c40a` |
| 672 | `apportionment-csd-final672.py` | `54262bf86dda15e86d08c23d90f19d58d7d0c0e5c61c466720072646be544f24` |
| 673 | `apportionment-after-timeline673.py` | `89f4bb4e6cb8116a7c0e67fe33f11a88fa1da66e028856cd1f91945ae3f80cf7` |
| 666 | `apportionment-timeline666.py` | `ebdca53b0569cc9039bf289f4a9e86515fec6e3e5af466e43f5f4adee2083287` |
| 667 | `apportionment-after-assembly667.py` | `20d2a46cc448ed03003db962d4c99430afef17f4308796e10064152c55905ca3` |

Also read: `root-apportionment-picture-repair562/pictures634.py` and its actual Mac outputs for chapters 01 and 02; `cc2-final-verify330-notes.md:284`; kit `crates/explainer_kit/src/layout.rs` and `kit.rs`; the base guard of the lineage, `narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py`. Citations are `file:line`. "The build host" is the CSD.

## Result

- **No concrete blocker found in the source.** Nothing here requires a change to 678 or 679 before launch.
- **Every seed sample survives the merge.** Replaying 678's own `merge` on plan 677 and the local timeline567 tables kept 1,147 of 1,147 samples with all their reasons in all 15 chapters, and did not change the seed.
- **The mapping is complete.** All 62 seed bindings are in export 682 at equal hashes, 16 in `files` and 46 in `reuse_from_prior_imports`, and the two suffixes `one()` looks up each match exactly one row.
- **Five Low findings.** F1 is the only one to act on before launch. It is a one-command probe of the single ffmpeg path nothing has run on the build host.
- **Root's report since my reading:** 678 preflight and read-only builds of 03 and 08 passed on the build host with 5,977 dependencies. I did not see that evidence, but its counts equal my replay (03: 48 samples, no anchors; 08: 121 samples, 2 anchors, none new). Those runs do not reach `extract`, so the limits listed at the end remain open.

## Findings, most important first

### F1. Low, do before launch: `-/filter:v` and the JPEG path have not run on the build host

616:299-301 is the first use of `-/filter:v` on the build host, as 681:18 says. Preflight (678:117-143) checks versions and Pillow only, and a read-only `build` stops before `extract_frames`.

The failure would stop safely, but it is expensive. 679:64-66 refuses to start if any chapter's job or manifest exists, and job 03 would already exist, so one bad option costs a new 678 and 679 pair for all 13 chapters.

On the Mac, ffmpeg 8.1.2 ran the probe below with exit 0. The build host has 8.0.1 (330 notes:284), so that result does not transfer.

Root action: run this on the build host. It writes no file. Expect exit 0 and no output.

```
/bin/bash -c "/usr/bin/ffmpeg -nostdin -v error -xerror -f lavfi -i testsrc2=size=320x180:rate=60:duration=0.2,format=yuv420p -an -/filter:v <(printf \"select='(eq(n,1)+eq(n,3))'\") -fps_mode passthrough -c:v mjpeg -q:v 2 -f null -"
```

It proves the option and the MJPEG negotiation from limited-range yuv420p, and nothing about decoding the 4K H.264 finals.

### F2. Low: sheet captions can hide the reason a new lint flag was sampled

616:330 cuts each caption at 118 characters. 678:160-163 places seed reasons first and appends new reasons after them, so a new flag's reason is the part that gets cut.

With one synthetic final row added to chapter 03, 32 samples carried the new anchor. Two captions exceeded 118 characters, and on one of those no new-flag reason reached the caption even by its first 40 characters.

Root action, no code change: when `merge.final_rows_not_in_seed` is above 0, take the frame list from `sampling.json` (reasons starting with the new anchor's id, numbered after the seed's anchors) and check those frames by number on the sheets. `sampling.json` keeps the full reasons.

### F3. Low: how to read the merge counts

678:149-156 keys anchors on the whole row, including the `(still there at N later looks)` tail (616:127-128 keeps `row` unchanged, layout.rs:318 prints the tail). A flag whose count differs from 567 therefore appears as a new row, with extra samples at the same time. That is harmless but is not evidence of a new defect.

A seed flag absent from the final is not counted anywhere. It shows only as a seed anchor whose `sources` lacks `final672` (678:155-156). Its seed frames are still extracted and still need viewing.

666:31 records only a layout hash for timeline666 and never compares it with 567, so these counts are the first comparison of final lint rows against the seed.

### F4. Low, wording: the seed-retention assertion cannot fail

681:33 says an assertion proves every seed sample and reason is kept. 678:164-165 checks a dictionary that 678:160 built from those same samples, and 678:162-163 only appends, so it passes by construction. The retention itself is correct, by reading and by my replay. For the real finals, the evidence should be the post-run comparison in step 3 below.

Plan 677's "every prior sample and reason" is accurate for its 15 bound sampling files. For 08, 13 and 15 the seed binds 584, and the 545 sampling files of those chapters are not among the 62 bindings. All 545 frame numbers there (115, 113, 82) are in the seed. I did not compare 545's reason text for those three chapters.

### F5. Low, records only: two small inaccuracies

- 681:15 says 678 uses the import result "after `success` and `stable`". In fact 678:87 executes support616 before 678:88-89, because `success` comes from it. The pin at 678:71-73 authenticates it first, so there is no effect.
- 683:18 `EXPORT_SHA` (`95c6ed1f...`) is a stale, unused constant. 683:60 and 683:102 use the literal `1c7c690b...`, which is the hash of 682. Do not cite line 18.

## Checked, no finding

**Mapping.** 678:90-94 needs manifest `1c7c690b...` and 4,005 unique rows, which 683:41 and 683:136-138 produce. `local` from 678:98 resolves on the real first binding, and all 62 bindings start with it. Each physical file is re-hashed and refused if a symlink (678:101-105).

**Loader.** Replacing the one line and executing it ran locally without error, with `__name__` not `__main__`. Setting `h.FFMPEG` and `h.FFPROBE` reaches the functions, because their globals are `h.__dict__`.

**Merge.** All 15 chapters replayed from real tables: no sample added, none lost, holds, predictions and anchor rows equal to the seed, seed object unmodified. Seed anchor ids stay in place (678:159). The synthetic flag at 40.0 s received its first look at frame 2400 and samples through frame 5526, under id `03-L1`.

**Layout persistence.** Checkpoint rows carry `@` (kit.rs:244) as sampled rows do (layout.rs:270), so 616:124-125 will not refuse them. The `@` value on a sampled row is the first look, because a persisted finding is recorded with its pending `at` (layout.rs:302). Each flag gets first look, +0.2 s, +1 s, +2 s, the 2 s grid to chapter end, and span samples (616:179-195). The final's `layout.tsv` is hash-bound through `record['files']` (671:392-394, 678:197-201), and 671:342 holds it byte-equal to timeline666.

**Finals and provenance.** Receipt, stability, command, manifest and every frozen row are checked (678:183-191). Tables are compared with 567 after the same path substitution 666:28 applies (678:206-210). The movie is the hash-bound file from the summary.

**Guards.** The 679 call matches 673's form and 678:228's exact command. The guard binary is bound transitively: 667:25 lists it, 673:21 inherits it, and 678:189-196 re-hashes and binds it. In the base guard the job folder is created after the lane lock (lines 105-107), `preflight_only` is written at creation (108-109), the child gets a new session (147-148), and `finished` is written in one save with `exit` (177-178). The final 679 stops on every refusal path I traced (679:33-37, 54-56).

**Process group.** ffmpeg runs as a direct child in the guard's group, unlike fmn's escaping encoder, so 672's stop handlers are not needed here.

**Frames.** Sample lists are sorted (678:166), so `zip` at 678:240 pairs names correctly, and a wrong count refuses (616:303).

**Chapters 01 and 02.** Excluded from 678 and 679 by design. The Mac 634 extractions equal the seed exactly: 48 and 55 frames, reasons equal. Both final layouts are 0 bytes.

**Budgets, as estimates.** Chapter 12 needs 11.2 fps within 616's 1,200 s. From TODO times, whole final jobs took at most 243 s for 04 (4,238 frames) and 350 s for 05 (6,316 frames), each including its own full decode at the same `-threads 2`. That gives at least 17 fps and about 773 s for chapter 12, before JPEG cost. The Mac decode passes took 13.8 s and 25.8 s. Using the Mac's mean JPEG of 139 kB and sheet of 278 kB, 03 to 15 total about 175 MiB and 137 sheets. The largest chapter at seed counts is about 33 MiB, and even the 2,500-sample ceiling is about 415 MiB, against the 4 GiB phase cap.

## Exact steps for root

1. Run the F1 probe on the build host.
2. Start one 679 exactly as 681:24 gives it. Do not duplicate it.
3. After each chapter, before viewing, compare `sampling.json` with plan 677: every seed frame present with every seed reason, `merge.seed_samples` equal to the seed count, sheets covering all samples.
4. Record `merge.final_rows_not_in_seed` and any seed anchor lacking `final672`. Apply F2 and F3.
5. View every sheet, as the TODO already requires, and record frames and sheets viewed by hash.

## If something stops

| Event | Meaning | Action |
|---|---|---|
| F1 probe fails | Option or encoder path unusable on this ffmpeg | Do not launch. Pre-launch change needed: new helper names, with the option replaced in memory as the REPO line is (`-filter_script:v` is the form used by 560, 572 and 587). |
| 679 stops at any point after a job or manifest exists | Refused by 679:64-66 on any restart | Preserve everything. New 678 and 679 names covering only the unfinished chapters. Do not repeat finished chapters. |
| Guard returns 75 | Admission or resource stop; job folder retained | As above, after reading the receipt. |
| `final_rows_not_in_seed` above 0 | Not a failure | Apply F2 and F3. |
| Seed anchor without `final672` | Not a failure | Note it; view its frames anyway. |

## Not established here

- `extract` mode, `sheets` and the guard-child path have not run on the build host. The first real chapter is their first test.
- I had no copy of `guard_csd02.py`. Guard statements rest on the base guard above, 672's and 683's run-time assertions, and root's records of the guard hash and policy, which I did not inspect.
- Root's reported preflight, 5,977 dependencies, parameter hash `f8e7fad1...` and tracker counts are root's reports. I did not recompute them.
- Actual final lint rows for 03 to 15 were not available to me. Only 567's were replayed.
- The speed and disk figures are estimates, not measurements of these extractions.
- `extract_frames` proves a frame count and a clean decode. It does not prove frame identity against a timestamp, and sheets are 960x540 thumbnails.
- No picture, motion, listening, fine-sync, mixed-host master, caption, loudness, independent AV or release acceptance is claimed. Narration is ElevenLabs Frederick and rendering native FrankenManim per the reviewed sources; I heard and viewed nothing.
