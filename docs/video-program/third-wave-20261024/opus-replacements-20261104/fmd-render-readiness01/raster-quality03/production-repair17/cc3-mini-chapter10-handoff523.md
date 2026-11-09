# 523: one native chapter 10 timeline and preview on mac-mini-max (CC3)

2026-11-09, CC3 (WhiteSwan). This note prepares exactly one `10_rigidity` timeline run and one
960×540 preview on mac-mini-max, to follow a successful 521 build. It uses the reviewed 268 source and
the original Frederick delivery: rebind269, which holds the 105 assembly19 WAVs byte for byte. No take
is regenerated. I did no SSH, transfer, build or render, I heard no audio, and I read no new picture.
Root does the transfer, the launches and the eventual host withdrawal. The build host's chapter 10
waiter is not touched.

Names used below:

- **ROOT** is `~/scratch/jmev-complex-native-20261109` on mac-mini-max.
- **D** is the build-host root, the constant `D` in `css-final-repair352.py`.
- **QREL** is this folder's repository path:
  `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17`.

Manifest: `cc3-mini-chapter10-handoff523-manifest.json`, sha256
`5be46fe5fa39de86d6f9169233cc5093912d94caef49a560d9cfc7c32af35e02`.

## 1. Why one chapter needs the whole film's narration evidence

rebind269's `assembly.json` lists all 105 outputs. `narration_delivery.py` therefore leaves the
chapter scope empty, and `coherent.verify` rebuilds all 105 cues from their take sources, even for
`render 10_rigidity`. The binary runs that check itself before it renders, and `--timeline-only`
does not skip it. A chapter-only assembly would be a new narration artifact, so I do not propose one.
The minimum set is therefore every file the verifier opens for 105 cues, plus chapter 10's
reference tables.

## 2. What goes to the mini

| Group | Files | Source | Destination on the mini |
|---|---:|---|---|
| evidence | 317 | local, in the manifest | `ROOT/repo/<repository path>` |
| narration replica | 106 | local, in the manifest | `ROOT/inputs523/rebind269-narration/` |
| reference | 5 | local, in the manifest | `ROOT/inputs523/reference/` |
| build host only | 4 | root fetches, read-only | as listed below |
| already on the mini | 14 scene files, plus the pre-269 plan | the 506 tree | no transfer |

Every manifest row was hashed when the manifest was written. Each one equals the build-host row in
the 309 transfer receipt (`css-runtime-delivery305.json`, 9992cea7) or in manifest 75
(`css-runtime-manifest75.json`, 5e1b0b03). Each evidence row also equals the sha256 recorded in its
take receipt. The manifest holds 428 files and 382,483,649 bytes.

**Evidence (317 files).**

- For the 103 kept cues, `decoded01/decode-receipt.json` names one decoded WAV, one downloaded take
  and one request file per cue. That is 309 files, plus the receipt itself.
- For 08_5 and 09_1, `audio17/mapped-receipt.json` names the WAV, the raw take and the request for
  each cue. That is 6 files, plus that receipt.

Manifest 75 carried 11 more files that the verifier never opens: the ASR and timing files in
`audio17`, and the superseded stage05/06 and decoded files for 08_5 and 09_1. They are left out.

The destination is `ROOT/repo` because guard 508 runs its child there. The 103 kept units and
everything inside `decode-receipt.json` use repository-relative paths, which resolve from that
working directory. That is also why 269 and 352 ran with the repository copy as their working
directory.

**Narration replica (106 of the 109 files).** This is a byte replica of the build host's flat
rebind269 folder. It is built from three local sources:

- 103 WAVs from `…/production-audio-stage07-01/native-narration01/`;
- 08_5 and 09_1 from `production-overlays-preserved220/jobs/csd-production-assembly19/narration/`;
- `phrase-timings.json`, copied from `final-picture-source41/001-phrase-timings.json` under the
  name `phrase-timings.json`.

**Reference (5 files).** All come from `preview274-review305/`:

- `preview/10_rigidity.timeline.tsv` (99fd22e6), `.cues.tsv` (a58e14f5) and `.cuelog.tsv` (11bc0949).
  Per `timeline269-comparison305.json` (d2bf8e9b), these equal timeline269's chapter 10 tables.
- `preview/10_rigidity.layout.tsv`, which is empty.
- `10_rigidity/select.filter` (8c6ebb2b), the 113 frames that preview 274 sampled.

**Build host only: four files, 544,012 bytes, read-only.** Expected hashes come from the 309
transfer receipt.

| Build-host path | sha256 | Bytes | Destination on the mini |
|---|---|---:|---|
| `D/jobs/csd-review-repair-rebind269/narration/report.json` | `b4a4c91e8e8fd26dbad621d118b762660e60532a2bbcf535ad88b87a45406bdd` | 365,593 | `ROOT/inputs523/rebind269-narration/` |
| `D/jobs/csd-review-repair-rebind269/narration/assembly.json` | `f0366ea40b3c23fd17c00a0222003b9041368bea995b3c1bac3c9d34f4743864` | 106,672 | same |
| `D/jobs/csd-review-repair-rebind269/narration/external-units.json` | `fd90880fbe7c1737786a9695e3c627dad0e4bfbaad0efba706e052e901b71968` | 39,639 | same |
| `D/work/review-repair268/workspace/explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-review269.json` | `7eb0e40d6366950169eb10727b3386fa1d290eaeb60078d609d5e57a0ef344e3` | 32,108 | `ROOT/inputs523/topic269/` |

`external-units.json` has the same bytes as the build host's post-269 `narration_units.json`. One
fetch serves both.

**Already on the mini.**

- The 23 disposition holds point into 14 scene files of the 506 tree. Each file's sha256 equals the
  rebind269 hold hash (`rebind-summary.json`, fa263d8a).
- The 506 tree's `narration_units.json` (995e25c7, 39,643 bytes) equals the build host's
  `narration_units.before-review269.json`.

## 3. Absolute paths: the one decision for root

rebind269 kept assembly19's sentence rows and unit records unchanged, and the preserved assembly19
`report.json` and `assembly.json` are local. Those copies, plus `review-repair-rebind269.py`, show
every absolute build-host path that the mini will meet:

- **`report.json`:**
  - 461 sentence-row `source` paths: 403 under `D/work/production-repo07/narration_lab/` and 58
    under `D/qualification01/production-repair17/audio17/` (08_5 and 09_1);
  - 23 disposition `evidence.path` values under `D/work/review-repair268/workspace/`.
- **`assembly.json`:**
  - the `wav` and `generation_receipt` of the 08_5 and 09_1 units;
  - the same 23 disposition paths.
- **`audio17/mapped-receipt.json`:** the `wav`, `source` and `request` paths of its two records.

The mini cannot hold the build-host root path. The macOS system volume is read-only, and only a
privileged `/etc/synthetic.conf` entry could create such a path.

**Recommended: rebind the paths in new copies, inside ROOT.** This is 352's `delivery()` widened to
the paths above. Let P be the realpath of `ROOT/repo` and W the realpath of `ROOT/source/workspace`.

1. Check that `ROOT/inputs523/rebind269-narration` holds 109 regular files whose hashes equal the
   309 rows (the manifest rows plus the three fetched JSONs).
2. Copy the folder to J/narration in a new job J, for example `ROOT/jobs/mini-delivery523`.
3. Write `P/QREL/audio17/mapped-receipt-mini523.json`, a new file. It is `mapped-receipt.json` with
   only the `wav`, `source` and `request` strings changed, from `D/qualification01/production-repair17/<rest>`
   to `P/QREL/<rest>`. Check each target against `wav_sha256`, `source_sha256` and `request_sha256`.
   The original receipt stays as it is.
4. In `J/narration/report.json`:
   - map each row `source` from `D/work/production-repo07/<rest>` to `P/<rest>`, and from
     `D/qualification01/production-repair17/<rest>` to `P/QREL/<rest>`, checking each file against
     `source_sha256`;
   - map each `meta.direction_contract` disposition `evidence.path` from
     `D/work/review-repair268/workspace/<rest>` to `W/<rest>`, checking each file against its sha256.
5. In `J/narration/assembly.json`:
   - set the 08_5 and 09_1 units' `wav` to `P/QREL/audio17/<cue>.wav` and their
     `generation_receipt` to the step 3 copy;
   - map the disposition paths as in step 4;
   - set `report_sha256` to the sha256 of the new `report.json`.
6. Check, as 352 did:
   - no string in either JSON still begins with D;
   - apart from the fields named above, both JSONs equal the fetched ones;
   - the other 107 files, the 105 WAVs among them, are byte-identical.
7. After section 4, run the delivery check with working directory `ROOT/repo` and the PATH from
   section 5:
   `python3 -I -B W/tools/narration_delivery.py --topic W/explainer_corpus/complex_analysis_visual --narration J/narration --consumer render --gain-db 0`,
   adding `--chapter-key K` for each of the 15 chapters. It must report `verified_cues` =
   `selected_cues` = 105, `plan_mode` `external-ui` and `chapter_scope` null.

No sample, take, sentence row or hash changes; only path strings in new copies do. Because this
writes new evidence files (two JSONs and one receipt copy), it deserves the kind of independent read
that 352's delivery had. Everything stays inside ROOT, so withdrawing the host removes all of it.
These two files are all I own, so I have not written this step's code. I can write it as one sealed
file if root wants.

**Not recommended: a `/etc/synthetic.conf` link at the build-host root path.** It would keep the
JSONs as they are apart from 352's row rewrite, but it is a sudo change outside ROOT that lasts until
someone edits the file and reboots.

## 4. Topic step: after 521 qualifies, before step 7

Under `W/explainer_corpus/complex_analysis_visual` on the mini, mirroring what rebind269 did on the
build host after build 268:

1. Create `narration_units.before-review269.json`, a new file holding the current
   `narration_units.json` bytes (995e25c7).
2. Create `docs/narration-direction-map-frederick-review269.json`, a new file holding the fetched map
   (7eb0e40d).
3. Replace `narration_units.json` with the fetched `external-units.json` bytes (fd90880f).

Afterwards, the 506 listing no longer matches, so 521 refuses this tree from then on. That is
expected. Restoring the tree needs the backup from step 1. The compiled script is unchanged, so the
binary's script check still passes.

## 5. Environment and guard

- **PATH.** The binary finds `python3` on PATH and fmn finds `ffmpeg` on PATH. The render encodes
  even with `--timeline-only`: it writes a 64×36 `timeline/10_rigidity.mp4`. The guard needs `sbh`.
  - Put a ROOT-local bin folder first, for example `ROOT/bin523` with `python3` linked to the
    Python 3.14 that root used for 521, plus `ffmpeg` and `ffprobe`.
  - Before launch, record in the job folder `command -v python3 ffmpeg ffprobe sbh`,
    `python3 -c 'import sys; print(sys.version)'`, `ffmpeg -hide_banner -version`, and the
    `ffmpeg -hide_banner -encoders` line for libx264.
  - Q records no ffmpeg on mac-mini-max (cc0 501 did not probe it), so its presence is a
    prerequisite, not a fact.
- **Guard.** Run each step as its own guarded job:
  `PYTHON -I -B cc2-mini-max-guard508.py --phase native --job NAME --inputs MANIFEST -- COMMAND`.
  - The child's working directory is `ROOT/repo`; the evidence in section 2 depends on that.
  - Build the absolute `--inputs` rows from the manifest destinations, the fetched files and the
    binary.
- **Stray ffmpeg.** fmn's ffmpeg route starts ffmpeg in a new process group (CC1, 518), so the
  guard's group signal does not reach it. If a job is stopped, look for a leftover ffmpeg and end it
  before withdrawing the host.
- **Threads.** Use `--threads 4`, 269's value and the guard's `RAYON_NUM_THREADS`. The thread count
  does not enter the tables.

## 6. Commands

Run these only after 521's `qualification.json` exists with exit 0 and the binary's hash equals its
`binary_sha256`, and after section 4 and section 3, step 7 have both passed.

- BIN = `ROOT/jobs/native-build521/complex_analysis_visual-521`. That is the job named in root's
  admitted receipt, 525.
- N = the realpath of J/narration.

Timeline, as job `mini-timeline523`:

    BIN render 10_rigidity --timeline-only --fps 60 --threads 4 --narration N --out ROOT/jobs/mini-timeline523/timeline-output

Preview, as job `mini-preview523` (274's flags, no `--score`):

    BIN render 10_rigidity --res 960x540 --fps 60 --threads 4 --narration N --out ROOT/jobs/mini-preview523/preview --lint --crf 18 --preset veryfast

Pictures for reading, the same 113 frames that 274 sampled, into a new folder:

    ffmpeg -nostdin -n -v error -xerror -threads 2 -i ROOT/jobs/mini-preview523/preview/10_rigidity.mp4 -an -filter_script:v ROOT/inputs523/reference/10_rigidity.select.filter -fps_mode passthrough -q:v 2 NEWDIR/frame-%06d.jpg

That must give exactly 113 files. `-fps_mode passthrough` replaces 274's `-vsync 0`.

## 7. Equality and pictures: both are required before any final use

Let S be the build-host rebind269 narration folder path, which is `FMD_NARRATION` in 352.

| Output (from the timeline and the preview) | Must equal | Rule |
|---|---|---|
| `10_rigidity.cues.tsv` | `a58e14f5f22089df8902439f751b0a05cfe52b29502f7c530054745d6f7a6ca3` | byte-equal |
| `10_rigidity.timeline.tsv` | `99fd22e62d5c6e53104ec3c622135d0748c3dc68e375141d3da4a38114d5860b` | after replacing S with N; 6 occurrences, 13 lines, 7442 frames |
| `10_rigidity.cuelog.tsv` | `11bc0949b31f6483f0a5df01d2c3c71485185384c5cc41c9ba8a6cd6df14bdde` | after the same replacement; 6 occurrences |

- **Lint.** The reference `layout.tsv` is empty. A non-empty file from the mini preview is a lint
  finding to read.
- **Movie.** The preview movie is a new arm64 picture. Its hash will not equal preview 274's
  `78eb267f…`, and the hash is not a check. Someone must read the 113 pictures, and more where
  needed, before the chapter is used. A matching hash or table is not picture acceptance.
- **Stop rule.** If any table differs, stop. There is no preview read and no final until the
  difference is explained.

## 8. Later, not part of this step

The 4K final would be 352's command with `--score`. It needs score21's
`10_rigidity.music.f32.wav`: `3307c2c9bddcb58d2c21899f1fcc68a836d815d71e430de179fd41cdcf1e0083`,
47,628,880 bytes, at `D/jobs/csd-production-score21/score/stems/`. It is not local. Manifest 75
lists no `.parts.tsv` for chapter 10.

## 9. Sources read

- `css-final-repair352.py` (1074051b): the mapping and `delivery()`.
- `review-repair-rebind269.py`.
- `prepare-assembly19.py`.
- The 506 tree's `narration_delivery.py`, `coherent_narration.py` and the kit's `cli.rs`
  (`check_delivery_contract`, the timeline-only branch).
- `cc2-mini-max-guard508.py` (135e4b2f).
- The local receipts named above.
- `root-mini-max-build521-admitted525.json`.

The manifest builder is a scratch script (`manifest523.py`, d0b12e66) and is not in Q.
