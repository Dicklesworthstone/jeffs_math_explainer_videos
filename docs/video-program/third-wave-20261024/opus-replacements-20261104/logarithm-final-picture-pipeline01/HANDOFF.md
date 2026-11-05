# Logarithm final picture: timeline and score continuations

RubyRaven, bead 87do.63, for root. Prepared and tested on stand-ins; nothing was launched, and no job here has run on the
real workspace. This is my own work, so it is not an independent review.

`S` is `EL/final-picture-repair01`. `B` is `five-film-release-20261027`, the guard's lane.

## Launch

Run these from the repository root, each in its own pane. They can start now: the timeline controller waits for
`logarithm-final-picture-build01`, then for `logarithm-final-picture-assemble01`, and launches no audio job. The score
controller waits for the timeline's comparison. Each wait lasts up to 12 hours.

```bash
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-final-picture-pipeline01
SEAL=<the SHA256SUMS hash in my Mail>
"$H/timeline-after-final-picture-assemble01.sh" "$SEAL" \
  8f7c4a9f45c4dd256536d8d5992bd14ebef44ac4fc80e5563f84c9de2bce5952 \
  ff1ece5acfa8fe5f2fff52efb683ed76ac1b010330c526d46b64cdc2bfaa808c
"$H/score-after-final-picture-timeline01.sh" "$SEAL"
```

The second argument is the frozen `integration.json`. The third is your approval of `holds-bound01.json`. The value above
is Lavender's prediction, which I reproduced with the real `rebind_audio.py --check` at that integration. If `--apply`
writes other bytes, `bind` refuses and nothing launches. To approve the actual file instead, start the controller after
`--apply` and pass its hash.

Exit codes: 0 verified, 1 refused or failed, 2 wait deadline. A guard non-admission exits 75.

## Names

| Item | Path |
|---|---|
| Jobs | `logarithm-final-picture-timeline01`, `logarithm-final-picture-score01` |
| Outputs | `B/<timeline job>/timeline-output`, `B/<score job>/scored/stems` |
| Summaries | `timeline-summary.json`, `score-summary.json`, written by version 4's worker unchanged |
| Job scripts and manifests | `S/jobs/`, created by the first step |
| New files in `S` | `final-picture-binding01.json`, `schedule-comparison01.json`, `score-comparison01.json` |

## Reused, not rewritten

`prepare_final_picture_pipeline.py` loads the sealed `prepare_stage09_render.py` version 4 by hash (`debd6f75…`, seal
`34b04c88…`). Version 4's `write_step`, `verify_step`, `verify_job`, job template, argv, timeouts and success lines run
unchanged. Two names change in memory on the loaded copy: `STAGE`, and `load_binding`, because version 4 refuses any
workspace but its own. Both jobs run through the same guard wrapper, with the same lane, lock and baseline.

One cosmetic leftover: each job script's docstring still says "logarithm stage09", because the template is verbatim.

## What `bind` requires before freezing inputs

1. The workspace tools and the guard match their pins.
2. Build and assembly receipts are successful and stable, each for exactly `S/build01.sh` or `S/assemble01.py`.
3. `integration.json` has your hash, approves the source, and every row is on disk. The unit plan is the one row allowed
   to move; its row must match `S/narration_units-before-rebind01.json`.
4. Each frozen manifest is the one its guard hashed, and every row is still true (4,347 for the build, as in your
   `render-master01.py` freeze). The build's manifest must bind `build01.sh`, every `.rs` file, `Cargo.toml`, `Cargo.lock`,
   `script.tsv` and `oracle.py`. The assembly's must bind `assemble01.py`, `script.tsv`, the units, `spans01.json` and
   `coherent_narration.py`. `audio-after-build01.py` does this.
5. There are 13 chapters and 80 cues, the binary names the repaired crate, and live `chapters`, `facts` and `oracle.py`
   equal the build's record.
6. All 80 WAVs and `phrase-timings.json` are byte-identical to stage09.
7. `holds-bound01.json` has your hash, audits this integration, and binds the same map as the narration. All 15 holds and
   the closing hold are checked against the source: file hash, marker on its line, and say-to-hold interval.
8. The real `coherent verify` and delivery contract pass.

Step 7 is needed: `coherent verify` passes the repaired source with the stale stage09 map.

## What the steps verify

- **Timeline:** version 4's checks, then all three tables for all 13 chapters. Cue order matches the script, every cue
  plays from `narration01` with the right hash, the tables agree with each other, and no cue runs past its chapter.
- **Score:** version 4's checks, then every QA report and every stem. A stem must hold exactly frames × 800 samples,
  whole or split, 48 kHz stereo float. A QA `pass` of null is accepted, as in `render-master01.py`.

## Comparisons

`schedule-comparison01.json` holds per-chapter frames, seconds and added or removed breath and mark rows, plus local and
film frames for all 80 cues. Moved frames never refuse anything. Only a changed cue list or changed cue audio goes in
`hard_failures`; that fails the compare step and stops the score.

`not_as_predicted` is informational. It lists any chapter not at its prediction (chapter 04 +24 frames, chapter 12 +420,
others 0), and any unpredicted chapter whose cues moved locally. Those two figures are estimates from the repairs, so a
listing there is a finding to read, not an error.

`score-comparison01.json` holds per-chapter stem identity, QA metric changes and the fingerprint.

## Checks

`python3 -I -B checks/run_checks.py SCRATCH` ran with 89/89 PASS, saved in `checks/outputs/run_checks.out`.

- **Real, read only:** stage09's tables, stems and QA; root's frozen `build-inputs01.json`; the stage09 binary; map16.
- **Real, in a scratch repository:** repaired source, `rebind_audio.py --apply` (`072e74a5…`), root's `assemble01.py`, the
  guard's code, `coherent_narration`, `narration_delivery`, `oracle.py`, both controllers, version 4's worker, and the
  prerequisites from `render-master01.py` run against the outputs.
- **Stand-ins:** the compiled kit and `build01.sh`. The stand-in retimes stage09's tables by the predicted amounts and
  writes silent sparse stems, so the +444 frames in that run prove the reporting, not the real schedule.

Real data caught four false refusals that stand-ins would have hidden, and all are fixed: QA `pass: null`, extensible WAV
headers on the real stems, a quoted `kit.say` in `what_it_forgets.rs`'s test module, and the stage09-inherited
`doing_nothing.rs.orig`, which nothing compiles.

## Resources, measured 2026-11-05 about 21:50Z

| Measure | Value |
|---|---|
| Free disk | 23 GiB (admission 10, stop 8) |
| Lane | 46.61 of 56 GiB |
| `target/` | 16.69 GiB, 12.87 of 20 GiB growth over baseline |
| Lane holder | `complex-stage06-optimized-timeline01`, still unfinished |

Stage09 allocated 0.03 GiB for its timeline and 0.69 GiB for its score, so these two jobs should fit. The build's new
target folder comes out of the same 7.13 GiB of target headroom, and an over-limit `target/` would refuse admission.
Prior times: kit timeline 3,812 s, chapter 12 alone 805 s; score about 18 minutes. Chapter 12 grows by the closing hold.

## Limits

- No real build, rebind, assembly, timeline or score has run with this code. The real kit's timing and real stems are
  unknown until they do.
- The guard's job folder blocks reuse of its name. After a non-admission or failure, preserve that folder. A retry needs a
  new series here (`MM`, `SERIES`) and matching names in `render-master01.py`, which is your call.
- Nothing here checks fine sync, picture, listening or the scene behind a hold. A passing marker is not a passing hold.
- No render or master step exists here. `render-master01.py` remains yours.
