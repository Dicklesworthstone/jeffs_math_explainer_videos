# complex-optimized-final-production01

WhiteSwan, for root. These are `final-render01.py`, `final-master01.py` and `final-verify01.py` adapted to the optimized timeline, score, preview, pictures and release binary. I launched nothing. Frozen once sealed.

I'm the source author, so this isn't independent acceptance. The sealed recovery, downstream and thumbnail02 bundles are unchanged, and so are root's three final helpers.

## Order

`EC` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`, and `B` is root's `five-film-release-20261027` folder. There are no waiters: start each step yourself, once, when the step before it exits 0.

1. View every preview sheet, resolve the findings, and write `EC/optimized-final-production01/preview-pictures-viewed01.json`. `final_render_optimized01.py --check-review` tests that file without starting anything.
2. Run `start-optimized-final-render01.sh`, which runs `--all`.
3. Run `start-optimized-final-master01.sh`, which runs `--launch`.
4. Run `start-optimized-final-verify01.sh`, which runs `--launch`.

| Item | Name |
|---|---|
| Chapter jobs | `complex-stage06-optimized-render-<chapter>-01` |
| Chapter output | `B/complex-stage06-optimized-render-work01/` |
| Master job | `complex-stage06-optimized-master01` |
| Final master | `B/complex-stage06-optimized-render-work01/complex_analysis_visual_master.mp4` |
| Captions | `complex_analysis_visual.en.srt` and `.en.vtt`, same folder |
| Verify job | `complex-stage06-optimized-final-verify01`, holding `verification.json`, the frames and the sheets |
| Manifests, controller logs and exits | `EC/optimized-final-production01/` |

## The review file

Nothing starts without this file. There is no flag or mode that skips it.

```json
{
  "reviewer": "who viewed the sheets",
  "review_scope": "source-author",
  "pictures_viewed": true,
  "chapters": ["the 15 chapter keys, in order"],
  "unresolved_visual_defects": [],
  "sheet_sha256": {"absolute sheet path": "sha256, one entry per sheet"},
  "extraction_sha256": "sha256 of pictures/extraction.json",
  "preview_summary_sha256": "sha256 of preview-summary.json"
}
```

- **`review_scope`:** this is the field I added for your "keep review scope explicit" request. It must be `source-author` or `independent`, and anything else is refused.
- **`pictures_viewed`:** it covers the preview sheets only. It says nothing about listening or the final pictures.
- **Coverage:** the set of sheets must equal what `extraction.json` implies, one sheet per six samples. A sheet removed from both the folder and the review is still refused.
- **Later changes:** a changed sheet, preview movie, table, extraction or summary is refused. The review is rechecked at launch, in each chapter child, and in the master and verify steps.

## Changes from root's helpers

Run `diff` against root's three files to audit these.

- **Render:**
  - It imports the sealed chain under five hash pins.
  - It also checks `.cues.tsv`, the full cue log (narration prefix plus every score row field) and the lint report.
  - Manifests start from the pictures manifest, so the pictures, preview and score helpers are all bound.
- **Master:** it pins the render helper. Its summary adds the binary hash, the timeline job and `full_baseline_equivalence_established: false`.
- **Verify:**
  - It pins the master helper and `verify_stage09_final.py`, and takes picture selection from the pinned pictures module.
  - `verification.json` still records `pictures_viewed: false`. It adds the preview review's path, hash and scope, and says that review does not carry over to the final pictures.
- **All three:** each child refuses a changed guard policy. Each launch calls `T.guard_policy()`.

These are unchanged from root's helpers:

- 4K60, CRF 16 and `--lint`
- all timeouts, which the check compares
- soundtrack capture and the bit-for-bit replay proof for all 15 chapters
- captions
- the strict check with full decode
- frame, A/V and loudness checks
- current and preview lint samples, including persistence
- cue, hold and phrase samples

## What was tested

`checks/check_final_production.py` passes all 75 checks in under 3 minutes, working in a temporary folder.

- **Real:** the three helpers and their pins, the sealed score, preview and pictures controllers, the pinned strict check and caption screen, ffmpeg, ffprobe, ebur128, the 105 real cue texts and takes.
- **Fake:** the guard, the binary, the job tree and the tiny movies. No real guard, render or master ran.
- **End to end:** the fixture ran 15 chapters, the master and the verifier, including a real full decode and 248 extracted final frames.
- **Refused:** 15 review faults, 12 render faults, 9 master faults and 9 verify faults. These include a too-quiet master, which the real strict check fails, and captions that differ from the script.
- **Failed chapter:** it stops `--all`, stays preserved and isn't retried.
- **Real controls:** a logarithm soundtrack capture, and the replay-proof lines in logarithm's master log, 13 of 13.

**One defect found and fixed:** my first draft of the render helper reused a variable name, which would have made every chapter fail at the very end of its 4K render. The rehearsal caught it. Root's code didn't have this problem; I introduced it.

**One fixture adjustment:** real phrase marks start at 2.52 s and fixture chapters last 0.6 s, so the verifier's fixture reads the real marks divided by 10. Without that, no phrase sample would land.

## Remaining limits

- **Real paths are unrun.** The real guard and release binary haven't run these helpers at 4K, or for the master or captions.
- **Sizes are unknown.** Complex 4K output hasn't been measured against the guard's per-job and total limits. A resource stop is a failed attempt.
- **Retries are by hand, as you confirmed.** A retry needs copied helpers with new `-02` names. All chapters share one work folder, and render refuses an existing `.mp4`, so the attempt also needs a new work folder, or the failed chapter's files moved aside and preserved.
- **Resume.** The start script refuses a second start. If a run is interrupted before a chapter job begins, run the helper with `--all` directly; finished chapters are skipped.
- **`--check` differs by helper.** On render it's light. On master it hashes all chapters. On verify it's your full plan and needs the master. `--pins` on verify is the light check, and it loads the whole pinned chain.

No listening, picture viewing, ASR, fine sync, AV acceptance or publication is claimed. Frederick's 105 takes are unchanged, and nothing was synthesized. The final pictures still need viewing after verification.

## Replay

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-optimized-final-production01
(cd $P && shasum -a 256 -c SHA256SUMS)
python3 -I -B $P/final_verify_optimized01.py --pins
python3 -I -B $P/final_render_optimized01.py --check
python3 -I -B $P/checks/check_final_production.py
```

Run these from the repository root. Once a start script has run, these stop being valid replays, because they refuse used names.
