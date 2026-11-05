# complex-optimized-downstream01

WhiteSwan, for root. These are the score, preview, pictures and thumbnail controllers for the optimized recovery. Frozen once sealed. I launched nothing.

## One deviation you should decide on

**`.cuelog.tsv` can't be fully equal, so I didn't require it.** `Kit::finish` logs the music stem as a cue, and a timeline-only run has no stem. In all 56 chapters of the three finished films I compared (logarithm, crofton, stokes), `.timeline.tsv` and `.cues.tsv` were equal to the timeline's, and `.cuelog.tsv` never was. A plain equality check would have refused every chapter.

The preview enforces this instead:

- `.timeline.tsv` and `.cues.tsv` must equal the new timeline's, line for line.
- `.cuelog.tsv` must start with every timeline row, unchanged and in order. Only this chapter's own score files may follow, in order, each with its real hash, at the last frame, at gain 0.0.

The same check passes on 13 real logarithm chapters, 4 of them with segmented stems. It refuses a dropped score row, an added row and a changed narration row on those real tables.

## Start

Score and preview are ready. All four can start now, each once, in its own tmux session:

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-optimized-downstream01
bash $P/start-optimized-score01.sh
bash $P/start-optimized-preview01.sh
bash $P/start-optimized-pictures01.sh
bash $P/start-optimized-thumbnail01.sh
```

Each one waits for its upstream exit files, then runs the unchanged guard. The lane lock keeps them serial. If an upstream exit file isn't 0, the controller stops with `Upstream controller failed` and launches nothing.

| Stage | New job | Output in job | Waits for |
|---|---|---|---|
| score | `complex-stage06-optimized-score01` | `scored/` | timeline |
| preview | `complex-stage06-optimized-preview01` | `preview/` | timeline, score |
| pictures | `complex-stage06-optimized-pictures01` | `pictures/` | timeline, score, preview |
| thumbnail | `complex-stage06-optimized-thumbnail01` | `output/` | timeline, score, preview |

Logs, exit files and manifests go in the new `EC/optimized-downstream01/`, where `EC` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`. The first start creates it. Names there are `optimized-<stage>-controller01.log`, `.exit` and `optimized-<stage>-inputs01.json`; the pictures manifest is `optimized-picture-inputs01.json`.

## What each stage requires

**Score refuses unless:**

- the timeline controller exited 0 and its receipt is a success under the unchanged guard policy;
- the summary names the release binary, whose hash still matches, and keeps `full_baseline_equivalence_established: false`;
- all 12 prior tables compared equal, and both sides' hashes still match;
- all three tables exist for all 15 chapters, and the 105 cues match the script in order;
- all 15 QA reports pass, with every metric true or informational.

**Preview refuses unless:**

- score succeeded, and the timeline and QA checks pass again;
- the tables pass the rule above;
- each movie has one video and one audio stream at 960x540, 60/1, with the table's exact frame count;
- a rendered `.layout.tsv` exists for all 15 chapters.

## Changes from your helpers

Run `diff` against each `native-stage06-root03` helper to audit these. Beyond the new names:

- **Exit dependencies:** the wait reads only `optimized-timeline-recovery01/controller01.exit` and the new controllers' exit files. The old `timeline-controller04.exit`, `score-controller01.exit` and `preview-controller01.exit` still hold 1, and no controller reads them.
- **Imports:** score imports the sealed `optimized_timeline_recovery01.py`, pinned by hash.
- **Time limits:** score (7200 s) and preview (14400 s) now use `T.RENDER_TIMEOUT_SECONDS`, which is 43200 s. The wait goes from 12 h to 48 h, because 12 h was no longer than the timeline's own limit. These are bounds, not estimates.
- **Thumbnail:** it uses the recovery manifest and `complex-stage06-optimized-build01` in place of `inputs04.json` and `complex-stage06-root-build01`.
- **Pictures:** lint comes only from the rendered preview. The optional timeline folder is gone, because timeline-only runs install no linter.
- **Added:** a guard policy check in every child, and a read-only `--check`. A second start of a script is refused.

Summaries keep root's scope lines. Preview adds `full_baseline_equivalence_established: false`.

## What was tested

`checks/rehearse_downstream.py` passes all 65 checks. It uses a fake guard, fake binary and fake job tree, and writes nothing in the repository. Real parts are each controller's wait and launch code, ffmpeg and ffprobe on a tiny movie, PIL, the start scripts' shell, the real script and narration files, and `--check` on the real tree.

The four controllers ran through once end to end. All 39 damaged runs and 2 failed waits were refused at the expected check.

## Remaining limits

- **Real paths are unrun.** The real guard hasn't run these controllers. The release binary hasn't run `score` or `thumbnail`, or rendered at 960x540.
- **One attempt per name.** If the guard refuses or a stage fails, its folders stay. A retry needs `02` names in copies of the helper and start script, and downstream controllers need the same change.
- **Lint is counted, not judged.** Findings land in `layout_findings`, and pictures pulls those frames, but a nonzero count doesn't stop the run.
- **EPERM retries are finite.** `03_two_planes` needed attempt 2 in the running timeline. The kit tries 5 times, then the whole `render all` fails.
- **Free space stays the likeliest stop.** The guard still stops a job below 8 GiB, whatever is using the space. By precedent the job sizes are small: logarithm's score took 0.69 GiB, and its 4K render's movies and soundtrack took 0.73 GiB.
- **The final stage isn't covered.** `final-render01.py` and the helpers after it import the old job names, so they can't run on these jobs. No new copies exist.

Frederick's 105 takes already exist and nothing here synthesizes. No listening, picture viewing, fine sync or final AV acceptance is claimed. Pictures records `pictures_viewed: false`.

## Replay

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-optimized-downstream01
(cd $P && shasum -a 256 -c SHA256SUMS)
python3 -I -B $P/score_optimized01.py --check
python3 -I -B $P/checks/rehearse_downstream.py
```

Run these from the repository root. Each `--check` hashes all frozen files and works while the timeline runs. `--check` and the rehearsal stop being valid replays once a start script has run, because they refuse used names.
