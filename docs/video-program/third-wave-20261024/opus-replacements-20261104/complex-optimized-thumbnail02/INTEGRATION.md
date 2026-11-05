# complex-optimized-thumbnail02

WhiteSwan, for root. This replaces the unstarted `thumbnail_optimized01.py`. Frozen once sealed. I launched nothing, and the sealed `complex-optimized-downstream01` bundle is unchanged.

## Start

Run this once, in its own tmux session. It can start now:

```
bash docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-optimized-thumbnail02/start-optimized-thumbnail02.sh
```

It waits for the timeline, score and preview exit files, then runs the unchanged guard. If any of them isn't 0, it stops and launches nothing. Don't start `start-optimized-thumbnail01.sh`.

| Item | New name |
|---|---|
| Helper | `thumbnail_optimized02.py` |
| Job | `complex-stage06-optimized-thumbnail02`, with output in `output/` |
| Stage folder | `EC/optimized-thumbnail02/`, created by the start script |
| Manifest | `optimized-thumbnail02-inputs01.json` |
| Controller | `optimized-thumbnail02-controller01.log` and `.exit` |

`EC` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`.

## The repair

Your finding was right. `thumbnail_optimized01.py` froze `T.MANIFEST` but imports the preview and score helpers, and neither was bound.

- **Manifest:** it now starts from `P.MANIFEST`, as pictures does. That carries the score and timeline rows and the preview and score helpers.
- **Imports:** all four imported files are also listed by name: `preview_optimized01.py`, `score_optimized01.py`, `optimized_timeline_recovery01.py` and `timeline-after-audio04.py`. That last one imports no other helper.
- **Pins:** the helper refuses to import any of the four if its hash changed. The hashes match the `01` seal and the score helper's own recovery pin.
- **`--check`:** it hashes only those four files and checks the new names are unused. It doesn't call the `01` chain's preflight, which hashes 5085 files and refuses once score has started.

`render` is unchanged apart from reading the new manifest. Run `diff` against the sealed `thumbnail_optimized01.py` to audit it.

## What was tested

`checks/check_thumbnail02.py` passes all 13 checks with a fake guard, binary and job tree. It runs no guard or render, hashes no frozen input, samples no disk and writes nothing in the repository.

- On the same fixture, the `01` helper binds neither the preview nor the score helper. The `02` manifest binds both, along with the preview rows and all four imports.
- A changed preview manifest row is refused at launch and again in the child.
- A preview exit of 1 launches nothing, a wrong-size PNG is refused, and a second start is refused.

As you asked, I didn't rerun the full suite or `--check` on the `01` chain.

## Remaining limits

- **Real paths are unrun.** The real guard hasn't run this helper, and the release binary hasn't run `thumbnail`.
- **Real rows are hashed only at launch.** The fixture has two rows. The real `P.MANIFEST` rows are first hashed here when the wait ends.
- **One attempt per name.** A retry needs `03` names in copies of the helper and start script.

No listening, picture viewing or publication is claimed. The thumbnail receipt records `visual_review: pending` and `publication: not uploaded`.

## Replay

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-optimized-thumbnail02
(cd $P && shasum -a 256 -c SHA256SUMS)
python3 -I -B $P/thumbnail_optimized02.py --check
python3 -I -B $P/checks/check_thumbnail02.py
```

Run these from the repository root. Once the start script has run, `--check` and the check script stop being valid replays, because they refuse used names.
