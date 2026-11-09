# Recovering animation + final-frame batches

`render_scenes(..., save_last_frame=True)` and `fmn-python --save-last-frame`
can now use the existing batch checkpoint journal. Recovery is per completed
**scene pair**, not per file: the primary output and its final PNG are both
verified before the scene can be skipped. No second scene execution is used
to regenerate the PNG. Restored outcomes retain `PairedRenderResult` and both
ordinary native publication receipts.

```python
from fmn_python import render_scenes

options = dict(
    format="mp4", save_last_frame=True,
    checkpoint="progress.json", resume_key="lesson-source-and-assets-v3",
    resolution=(1920, 1080), fps=60, threads=8,
)
report = render_scenes({"Intro": Intro, "Proof": Proof}, "movies", **options)
# After interruption, use exactly the same plan and inputs:
report = render_scenes({"Intro": Intro, "Proof": Proof}, "movies",
                       resume=True, **options)
```

```sh
fmn-python lesson.py Intro Proof --format mp4 --save-last-frame \
  --video_dir movies --checkpoint progress.json --resume-key lesson-v3
# Repeat the command with --resume to recover.
```

This applies to MP4, MOV, GIF, Y4M, PNG sequences, and WAV paired with a final
PNG. The companion naming policy is unchanged: file stems acquire `.png`, and
sequence directories acquire an adjacent directory-name `.png`.

## What is admitted

The saved plan explicitly binds paired mode, selected format, scene identities,
JSON constructor values, order, resolution, FPS, playback range and output
options. Existing version-2 single-output checkpoints retain their previous
shape and remain readable; they cannot masquerade as paired checkpoints.

Each successful pair has two role-labelled inventories. File members must
match their own native byte counts and SHA-256 digests. Sequence primaries
use the existing bounded per-file inventory. The final PNG has an independent
native receipt and must contain one frame. Both receipts must agree on scene
seed, view dimensions, FPS and playback selection. Readback retains its own
thread count and engine identity. Symlinks, special files, changed bytes,
missing members, foreign paths, malformed receipts and incomplete pairs are
refused. The combined file budget covers the sequence and final PNG together.

The journal and its persistent lock must lie outside **both** output paths.
The existing exclusive lock, atomic journal replacement, fsync and
write-before-observer ordering remain in force.

## Failure semantics

A successful primary followed by failed PNG publication is still a failed
pair. Its published primary is retained, and a subsequent attempt stops on
no-clobber preflight instead of deleting, overwriting or silently adopting it.
The same applies to a crash after publication but before checkpointing. This
feature does not implement orphan adoption or in-place continuation of a
partially rendered scene.

`resume_key` remains the caller's assertion about source and asset versions;
output hashing is not a source-code cache or a certified input closure.
Paired output remains standard-only. Subdivision recovery is not included.

## Validation

```sh
python crates/fmn-python/tests/paired_checkpoint_unit.py
python crates/fmn-python/tests/batch_checkpoint_acceptance.py
```

The first exercises the real batch loop and journal with filesystem fixtures
and a renderer double. The second uses the installed native wheel and existing
console entry point; it retains the six single-format regressions and adds
four native paired formats plus paired CLI failure/resume/corruption coverage.
Passing the protocol tests is not evidence that the native render suite ran.
`scripts/check_portal_runtime.sh` runs both: the first over the checked-in
`fmn_python` sources (it needs no wheel), the second against the freshly
installed wheel. The gate's paired console refusal table
(`paired_output_cli.py`) keeps subdivided paired recovery refused before the
source is imported.
