# CC2 final verification 321: the mixed master 318

cc2, 2026-11-08. Prepared only. Nothing here ran on CSD, CSS, TRJ or HZ2: no launch, job, render, decode, extraction, permit, deploy, import, canonical, TODO or Beads change. Paths are relative to D (the CSD root that the helper names) and Q (`D/qualification01/production-repair17`). `:N` is a line of `csd-final-verify321.py` unless another file is named.

**Not claimed.** No picture, motion, 4K, fine-sync, caption-reading, listening or independent AV acceptance, and no audio heard. The narration stays the 105 retained ElevenLabs Frederick takes, bound by bytes only; nothing is synthesized. Every review flag that 321 writes is `False` (`FLAGS`, :154).

| File | sha256 | State |
|---|---|---|
| `csd-final-verify321.py` (682 lines) | `0a956ec8c0613971ad33e90a0a9eb4a496ba42bd2cb06d8044105397bfc5b7fb` | new |
| `cc2-final-verify321-notes.md` | this file | new |
| `cc0-combined-master318.py` (CC0's, 950 lines) | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` | read, not modified. It is the frozen handoff: `cc0-master318-notes.md` (`423132ff…`), and root's source review `root-master318-review305.json` (`c1619b8b…`, no findings). CC1's review 322 is still pending. 321 does not pin it: the hash comes from the inputs that the completed 318 job bound. |
| `trj-final-verify19-02.py` | `355781524ca0df4634b88642bc58930d8bedde82bd1429cbb8c74e401d2d7f5c` | preserved, unchanged |
| `trj-verify-after-master40.py` | `416c4088b58fea1695ffa645be2f86f202cb35cc12eb436bd049eed115f6bf32` | preserved, unchanged |
| `trj-pictures-after-verify182.py` | `86e8224bf738fb913ecb8e06e2c7dbb9f7daa75e88efb9c66d04bf3b9c043334` | preserved, unchanged |
| `trj-final-pictures182.py` | `0f9b2de7db9a48c8bc0cc1f5fef4762fe2cd13348e8e9cfdbea28658d8efd072` | preserved, unchanged |
| `final-chapter-pictures142.py` | `a0430c030b48e9defdbaa50b22494cb7f96a763ce9ec1419233dff752edd8dc5` | preserved, unchanged |
| my 317, 311 and 303 files | `19abeee9…`, `47e0f30a…`, `08be95be…`; `86296883…`, `ff39db34…`, `22f3a0ce…`; `db3ce8e4…`, `def224f0…` | unchanged, re-hashed today |

## 1. The existing path, as inspected

The complex film's only final technical-verification and final-picture path is the TRJ production19 chain:
- `trj-verify-after-master40.py` waits for the master and launches `trj-final-verify19-02.py`.
- `trj-final-verify19-02.py` runs:
  - the strict script and the decode lines;
  - `media_facts`;
  - exact captions and phrase marks through the shared `verify_stage09_final.py`;
  - ebur128, with TP at most −1 dBTP;
  - the caption screen, whose output is recorded but does not gate.
- `trj-pictures-after-verify182.py` waits for the verifier and launches `trj-final-pictures182.py`.
- `trj-final-pictures182.py` takes plan182's 5,341 overview and 39 native samples and adds every final layout row at deltas −12…+600 frames, plus persistence every 300 frames for "later looks" rows. It makes one ffmpeg pass (`split=2`, a balanced `eq(n,N)` select), writing 960×540 JPEGs (`-q:v 2`) and native PNGs.

**None of it has run.** Root retired the master184, verify40 and pictures182 controllers at 06:01 UTC today, before any master existed (`third-wave-20261024/TODO.md` :69). The chain's gates have no runtime record. 321 reuses what it can: the shared module, the strict script and the caption screen, at the pins TRJ used, and plan182 by hash.

## 2. What 318 changes, and how 321 adapts

| 318 fact (from its source at `478b44a5`) | What the TRJ chain assumes | 321 |
|---|---|---|
| CSD host, job `csd-combined-master318`, out `job/final` (318 :76–77, :863) | TRJ paths, master01 | CSD constants (:47–61). The 318 helper's hash comes from the inputs its completed job bound. |
| Three binaries: 09 on binary 313, ten chapters on 268, four retained on 19 (318 :694–713) | one binary | Each row's kind and binary is checked (:226–231). Binary 313's hash is written nowhere: it is read from build313's qualification, which must name base 268, differ from it, and change only `boundary.rs` (:198–203). `chapter_binaries` and `boundary_replay` must name the same binaries. |
| The kit joins video by stream copy plus `h264_metadata` (`crates/explainer_kit/src/master.rs:193–251`) | one encoder run | **Decoded-frame identity.** The master's framemd5 must equal the 15 staged chapters' framemd5, concatenated in registry order (:503–520). A mismatch names the global frame, the chapter and the local frame. Each movie's extradata MD5 is recorded (:521–524). |
| Foreign rows' jobs sit at their original absolute paths after import (318 :595) | one host | Each row's staged movie must be `host/jobs/job/render/<key>.mp4` (:258). The source job's `render-summary.json` must name the chapter, frames and binary, and list each staged file at its hash. 09's summary must also carry repair310's hash and build313's qualification hash (:258–266). |
| Staged hard links and `staged-inputs.json` (318 :868–884) | copies | The staged set must be exactly 15×5 tables and movies plus 15 soundtracks. Each must be the same inode as its source, at the digest that 318's bound inputs give it (:243–249). The bytes are not hashed a second time. |
| A narration folder per row: rebind313 for 09, foreign folders for the retained four (318 :681) | one folder | All 15 schedules are checked (:276–314). Each staged timeline and cues table must equal timeline269's and timeline21's under the row's folder. Each cue log must have timeline269's as its prefix. Each row's `start_frame` must match (:283). Every line's WAV must sit in the row's folder under its cue name and equal rebind269's and assembly19's bytes. Exactly 105 cues, in script order. The 105 rebind269 WAVs must equal rebind313's and assembly19's. |
| Captions written by binary 268 from rebind269's report (318 :886–893) | delivery02's report | `check_captions` checks the rebind269 report against placements taken from the staged tables, and `check_phrases` checks against rebind269 (:325–327). |
| 318's own replay proof (318 :819–836, :896–902) | none | `master-summary.json` must report 15 bit-identical replays in registry order. Each must have its staged cue-log row count and frames × 800 samples. The `boundary_replay` capture must be the staged 09 soundtrack at its digest (:253–254). Each staged soundtrack must be 48 kHz, 2 channels, 16-bit, frames × 800 samples (:267–270). |

## 3. The gates the task names

**Full decode.**
- The strict script runs with `EXPECT_*` on the master: software decode with `-xerror`, then the hardware second opinion (:487–496).
- Its decode log goes into the job's own `decode-logs` folder (`TMPDIR`), and the last line must be `VERIFY PASS`.
- `software_decode_clean` must hold.
- On top of this, framemd5 makes two more complete video decodes: all 15 chapters, then the master.

**Frame and audio contracts.**
- `media_facts`:
  - exactly one h264 3840×2160 60/1 stream and one aac 48 kHz stereo stream;
  - counted video packets must equal 133,662;
  - the video must last the timelines' total within one frame, and the audio within one AAC frame.
- Frame identity, as in §2.
- Encoded loudness, with TP at most −1 dBTP (:525–529).
- 318's own replay and soundtrack checks, as in §2.

**Exact captions.**
- SRT must equal VTT.
- The captions must stay inside the video and in order.
- Their text must equal the script both as a whole and cue by cue, and the report's sentences must equal the script.
- The caption screen is run, and its `srt_sha256` must be the master's SRT (:530–534). Its counts are recorded but do not gate, as in TRJ 19-02 :42.

**Previous layout anchors.** `PREVIOUS_LAYOUT` (:122–153) holds every distinct non-empty finding from 18 local snapshots: the production19-era `final-picture-source41`, the 268-era terminals, and previews 21, 70, 166, 243 and 274. That is 30 rows. Their hashes are kept in `PREVIOUS_LAYOUT_SOURCES` and copied into `sampling.json`. They are sampled with the current final layouts at the TRJ deltas, with 300-frame persistence for "later looks" rows. Current anchors also get a native 4K frame.

**The clocks disagree.** I checked the stamps against the current schedule. The old 09 rows in `final-picture-source41/037` and `/038` have `@t` 115.3 s early. For example, `09_3 +6.9s @41.6s`: 09_3 now starts at frame 8,998 (149.97 s), so the moment is at 156.9 s. That matches the later stamp `09_3 +6.8s @156.8s`.

First-cue rows run the other way. Their `+s` counts from the chapter start, so cue start plus offset lands 0.6 s late. `anchors()` (:335–348) therefore keeps `@t` and adds "cue start + offset" whenever the two differ by more than 6 frames, and every sample says which clock placed it. An unrecognized row stops `check` and `launch-pictures` before anything is written.

**Final picture extraction.**
- It runs only from the actual 318 master, after the verification job has finished, its inputs hold, and its record names this master (`verified`, :560–568).
- The plan (:351–423) is:
  - all of plan182, whose chapters must equal the master's;
  - the layouts above;
  - 09's frames 2,084–2,102;
  - the seven review312 and preview313 anchors at +0, +30 and +60, each with a native frame;
  - persistence over 39–133 s;
  - preview313's measured exit, −2 to +15 frames, with a native frame;
  - preview313's own picks;
  - preview313's plate sites;
  - three seam frames at each end of every chapter;
  - a frame every 10 s.

  The probe's enforced window is 2,086–2,100; 312's 2,087–2,099 was by eye (review320 L1). The extraction itself is one ffmpeg pass, as in 182 (:603–606).
- Every picture is size-checked and hashed. Sheets hold eight pictures at 1920×2260 and are written with `'xb'`.
- `sampling.json` records the plan, the hashes and the flags.

**Plate sites (review320 observation 2).** No gate reads preview313's plate-site `flags` or `not_localized`. Each placed site is sampled at its start, middle, end and 30 frames after. A flagged or unlocalized site is also sampled every 30 frames across its whole search window (:401–413).

## 4. The 318 keys 321 relies on (checked against `478b44a5`)

| What | 318 source | 321 |
|---|---|---|
| Guard command `PYTHON -I -B <318 resolved> master --job J --inputs I --manifest M` | :812–814 | :211–213; `summary.manifest` bound at its hash (:222–223) |
| `staged-inputs.json` rows `{source, path, sha256}` | :868–884 | :241–249 |
| Summary keys `chapters`, `frames`, `native_bit_identical_replays`, `replays{cues, samples}`, `boundary_replay{chapter, capture{path, source, sha256}, captured_by_binary_sha256, replayed_by_binary_sha256, bit_identical}`, `chapter_binaries`, `outputs{master, srt, vtt}`, `version`, `host`, `consumer_binary_sha256`, `narration`, `manifest` | :896–931 | :215–254 |
| Rows `chapter`, `kind`, `host`, `job`, `binary_sha256`, `narration`, `frames`, `movie_sha256`, `start_frame` | :679–682, :708 | :226–270, :283 |
| Render summary `chapter`, `frames`, `binary_sha256`, `files`; for 09, `boundary_sha256` and `build313_qualification_sha256` | :611–626 | :260–266 |
| Output names `complex_analysis_visual_master.mp4`, `.en.srt`, `.en.vtt` | :903–904 | :236–240 |

If CC1's review 322 changes 318, these must be re-read before `check` is trusted.

## 5. Modes and order for root

`python3 -I -B Q/csd-final-verify321.py MODE`
1. `check`: read-only. It runs every static gate and the full picture plan, and prints the counts and an upper estimate of the picture job's growth. It writes nothing.
2. `launch-verify`: refuses an existing job or inputs, then gates. It freezes `Q/final-verify321-inputs.json` with `'x'`, then runs one guard call: phase native, `--` then `/usr/bin/timeout 86400 python3 -I -B helper verify` (:639–651).
3. `verify` runs only inside that guard. It is admitted with policy 8/4, its inputs are re-hashed, it uses 16 CPUs, and PATH is `/usr/bin:/bin`. It writes `verification.json` with `'x'` and prints `FINAL VERIFY PASS…`.
4. `launch-pictures`: refuses unless the verification job is stable. Its inputs freeze the verify job's files and preview313's records.
5. `pictures` runs only inside that guard.

Each version gets one attempt. A refusal or failure needs a new version, and nothing is retried.

## 6. Deploy prerequisites on CSD (root)

- In Q, at their pins (:67–70):
  - `verify_stage09_final.py` `f266a231…`;
  - `verify_video_preserve_logs01.sh` `3f4c68db…`;
  - `caption-interval-audit01.py` `3cc26436…`;
  - `final-picture-plan182.json` `20a36f91…`.

  Plus this helper. The local copies are at those hashes.
- `/usr/bin/python3` with PIL, needed for `pictures` only.
- `bash`, `ffmpeg` and `ffprobe` on `/usr/bin:/bin`.
- At least 16 CPUs in the inherited affinity.
- 318 complete and stable, and no other native job holding the lane.

## 7. Findings and decisions

- **F1. plan182 hygiene (pre-existing, not mine).** `final-picture-plan182.json` is untracked. 1,501 of its lines carry local home paths (`local_snapshot_path`). 321 pins its exact bytes, so the CSD copy must be this file. Committing it to the public repo as it is would publish those paths. Root's call: for example, keep the pinned copy off the repo, or commit a sanitized copy under another name.
- **F2. Two layout clocks** (§3). Without the cue-relative clock, the old 09 findings at 156.8 s and 223.2 s would be sampled at 41.6 s and 108.0 s.
- **F3. The TRJ chain never ran** (§1). 321 is the first version of this path written for a real master, and none of its gates has a runtime record.
- **F4. Picture-job growth against the native phase's 4 GiB cap.** No measured full-film extraction exists. I made a local estimate from these inputs:
  - timeline21's tables, which have timeline269's starts;
  - plan182;
  - the 30 previous rows;
  - three 268-era layouts standing in for the finals;
  - no preview313 picks.

  That gives 5,770 overview frames, 52 native frames and 722 sheets. The size is 0.53 GiB with local medians: 40 KB per JPEG, 233 KB per sheet and an assumed 3 MB per PNG. The upper bound is 2.25 GiB, from local maxima (78 KB, 897 KB) and raw RGB size for each PNG. `check` prints the same upper estimate for the real plan.
- **F5. Cost of frame identity.** framemd5 makes two complete video decodes beyond the strict script's two. Timeouts are 3,600 s per chapter and 14,400 s for the master. Run time is unmeasured.
- **Decisions.**
  - The extradata MD5 is recorded but not gated, because the kit copies the chapters' streams, so their parameter sets may legitimately differ.
  - The caption screen is recorded, not gated, as in TRJ.
  - The TP gate is TRJ's.
  - An unrecognized layout row refuses before launch.
- **Residual.** If two chapters decoded to different pixel formats, framemd5 on the master would convert at the change. The frames would then differ and verify would refuse, naming the frame: a stop, not a false pass.

## 8. Local checks (scratch, outside the repo; not host runs)

- `compile()` passes under `python3 -I -B`. The hygiene grep finds no home, scratch or tmp paths in either file.
- **The 30 embedded rows are exact.** They equal the distinct non-empty rows of the 18 snapshots, both ways, and all 18 are at the recorded hashes.
- **A synthetic fixture met 31 of 31 expectations.** It used the real helper text, with only the root, the frame total and the plan pin swapped, and the real shared module, PIL and caption screen. ffmpeg, ffprobe, the strict script, the guard and CPU affinity were fakes. It covered:
  - `check` passes;
  - eleven refusals:
    - 09 on binary 268;
    - a non-identical 09 replay;
    - a retained row relabelled;
    - a replay sample count;
    - a retained WAV unlike the take;
    - a table unlike timeline269;
    - a VTT edit;
    - a render summary naming another binary;
    - a workspace313 script;
    - a changed bound input of 318;
    - a second launch;
  - the guard argv;
  - a master frame unlike the staged 09 render, refused at "09_boundary local 5";
  - launch-pictures refused before verification and while it is unfinished;
  - a complete verify, then pictures: 105 cues, 105 exact captions, flags `False`, both clocks, seams, plan182 kept, preview313 exit, picks and plate sites, and native frames.
- **Limits.**
  - No real 318 records exist yet.
  - No ffmpeg, guard or strict script ran.
  - The framemd5 and extraction paths ran only against fakes.

## 9. Still open after a successful 321

- root's viewing of every sheet and native frame;
- motion;
- joins heard and seen;
- fine sync;
- listening;
- independent final AV review;
- public 4K and uploaded-caption verification;
- publication.

No audio heard or checked, here or by any check above.
