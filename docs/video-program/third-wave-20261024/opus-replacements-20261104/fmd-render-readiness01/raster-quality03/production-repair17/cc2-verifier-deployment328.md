# CC2 328: CSD prerequisites for verifier 321 (read-only check)

cc2, 2026-11-08, probed 19:40–19:48 UTC on CSD (`sensedemobox`). D is the CSD root that `csd-final-verify321.py` names, and Q is `D/qualification01/production-repair17`. Source paths are relative to the repository root.

**What I did on CSD.**
- Three Python probes under `/usr/bin/python3 -I -B -`, fed over ssh stdin. They read files, list folders and list processes.
- A few lavfi capability runs of 0.1 s each, under `nice -n 19`. Every output went to ffmpeg's null muxer or to stdout.
- One `ffprobe` read of an existing preview movie.

Nothing was written, deployed, launched or changed on any host. I checked afterwards that no probe process was left running. 321's `check` was not run, because master 318 does not exist. No Beads, TODO or source edits. No picture, decode, AV or listening acceptance is claimed.

## 1. Missing on CSD: deploy these exact files to Q

All six of these are **missing** from CSD Q. None is **mismatched**, because none is present. I also found no same-named copy in D, `D/qualification01`, `D/work` or `D/work/bin` that root could deploy from instead.

| Q file on CSD | Pin (sha256) | Exact local source (repo-relative), verified at the pin today |
|---|---|---|
| `verify_stage09_final.py` | `f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94` | `docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/verify_stage09_final.py` (tracked) |
| `verify_video_preserve_logs01.sh` | `3f4c68dbdcb02c1c90ff03cb4f20afc36792b7f20c6cc111a5eef21617a11410` | `narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/verify_video_preserve_logs01.sh` (untracked) |
| `caption-interval-audit01.py` | `3cc2643615ce5263b01d429ae6f72326c95e777af3e7f29ef9266a05155fa0c4` | `narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/caption-interval-audit01.py` (untracked) |
| `final-picture-plan182.json` | `20a36f91989e0f1e0ca5149a74483ec22fe3728be44b872fc31a0859747b962a` | `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/final-picture-plan182.json` (untracked; see note) |
| `csd-final-verify321.py` (the helper itself) | `0a956ec8c0613971ad33e90a0a9eb4a496ba42bd2cb06d8044105397bfc5b7fb` | `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/csd-final-verify321.py` |
| `cc0-combined-master318.py` (CC0's) | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` | same folder, `cc0-combined-master318.py` |

- **The first four** are 321's `PINS` (helper :67–70). `shared()` refuses unless all four are at these hashes.
- **The 318 helper** is listed because 321 reads it at `Q/cc0-combined-master318.py` and requires the hash that 318's own job bound. 318 needs it there anyway. It is the frozen `478b44a5…` that root reviewed; CC0's independent review 323 is still open.
- **plan182** (F1 in the 321 notes) is fine to deploy as these exact bytes. Its 1,501 home-path lines matter only if it is committed to the public repo.
- **Check after deploying.** Hash the six files in Q, read-only, and compare them with the pins above. The bytes must match exactly: 321 compares raw sha256, so no line-ending or permission normalization.

## 2. Present and adequate

| Requirement | CSD state |
|---|---|
| Guard `D/guard_csd02.py` | present. 2,106 bytes, sha256 `f1cae1dbc80075311b0e1a2abf8637439d2d8d0edbc643e771e2eefe2b3bdfb6`. It loads the owner guard and the owned-process stop module at their pinned hashes and asserts hostname `sensedemobox`: native admission 10 GiB, stop at 8 GiB free, native phase cap 4 GiB. It sets no CPU affinity. It is the guard the running preview313 job uses. |
| `/usr/bin/python3` | 3.14.4. Under `-I -B`, `hashlib.file_digest` exists. |
| PIL (needed only for `pictures`) | **available under `-I`**: Pillow 12.1.1 from the system dist-packages. `Image.new` and `ImageDraw.text` ran. |
| `bash`, `ffmpeg`, `ffprobe`, `timeout` on `/usr/bin:/bin` | all in `/usr/bin`. ffmpeg and ffprobe are 8.0.1-3ubuntu2. |
| What the strict script also calls | `awk`, `grep`, `sed`, `tr`, `wc`, `cut`, `mktemp`, `date`, `uname`, `head`: all in `/usr/bin`. |
| ffmpeg components 321 uses | Muxers: `framemd5`, `image2`, `null`. Filters: `ebur128`, `select`, `split`, `scale`, `volumedetect`. Encoders: `png`, `mjpeg`, `rawvideo`. |
| framemd5 | a lavfi run to stdout gave 6 rows in the six-column form that `frame_hashes()` parses, exit 0. |
| ebur128 | exit 0, with a `Summary:` block holding `I:`, `LRA:` and `Peak:`, which is what `loudness_summary()` reads. |
| 321's single picture pass | **exit 0** on a finite lavfi source with both outputs to null. It used the same options as 321: `split=2` / `select` / `scale` from `-filter_complex_script`, `-vsync 0`, `-q:v 2` mjpeg, png `-compression_level 3`. |
| `ffprobe -show_data_hash MD5` | exit 0 on the existing preview274 `01_the_bump.mp4` (h264). It returns `extradata_hash` (`MD5:9eb154c2…`, 49-byte extradata), the key that 321 records. |
| CPU affinity | 64 CPUs. A plain ssh shell has `Cpus_allowed_list` 0–63, so 321's ≥16 check passes, and it would pin itself to CPUs 0–15. |
| Free space on `D/jobs` | 100.1 GiB. |
| `D/work/cache` (321's `XDG_CACHE_HOME`) | present. |

## 3. Things root should know (no file change asked)

- **No hardware second opinion on CSD.** CSD has no `nvidia-smi` on `/usr/bin:/bin` and no `/dev/nvidia0`. The strict script's `VERIFY_HWACCEL=auto` therefore resolves to `none`, and only the software `-xerror` decode runs. The 321 notes (§3) describe a hardware second opinion; that does not happen on CSD. 321 still works, because `software_decode_clean` reads the first decode line, and the software decode decides.
- **Two options are deprecated.** ffmpeg 8.0.1 warns about `-filter_complex_script` ("use -/filter_complex") and `-vsync 0` ("use -fps_mode"), but exits 0. `-fps_mode passthrough` was also accepted. These are warnings on this build, not failures. A later ffmpeg could drop them. 321 is frozen, so I changed nothing.
- **CPUs 0–15 are shared with the native lane.** The running preview313 render (binary 313, PID 4124491) is pinned to CPUs 0–15, the same CPUs 321 takes. The one-native-job rule keeps them apart. Root should still confirm that the lane is free before each 321 launch, as for 318. CSD's load average was 28–30 during the probe.

## 4. Duplicate check: nothing owns 321

- **CSD jobs and inputs.** There is no `D/jobs/csd-final-verify321` or `D/jobs/csd-final-pictures321`, and no `Q/final-verify321-inputs.json` or `Q/final-pictures321-inputs.json`.
- **CSD files.** No file in Q has 321 in its name. No `.py`, `.sh`, `.service` or `.timer` file directly in Q, D or `D/qualification01` names 321's jobs or inputs.
- **CSD processes and schedulers.**
  - No process on CSD names 321 or master 318.
  - The only guard process is preview313's: PID 4124112, `--job csd-review-repair-preview313`.
  - No user systemd unit or crontab line names 321.
  - CSD tmux has `jmev-review-repair313-preview` and 15 `nvme-*` agent sessions; none names 321.
- **Elsewhere.** No process or tmux session names 321 or master 318 on TRJ or on the local machine.

So no existing controller owns 321. The only thing that may create its jobs is root running `launch-verify`, then `launch-pictures`, once each.

## 5. Where the lane is (why `check` was not run)

The 313 chain is still running: preview313 is live, and its `summary.json` does not exist yet. The final317 job, the master318 job and `combined-master318-inputs.json` do not exist. Of what 321 reads, these exist now: build268, build313, rebind269, rebind313, assembly19, timeline269, timeline21, both workspace scripts and `D/work/cache`. `check` would refuse until 318 is complete and stable, so it waits for root's order: 313, then the permit, then 317, then 318, then 321 `check`.
