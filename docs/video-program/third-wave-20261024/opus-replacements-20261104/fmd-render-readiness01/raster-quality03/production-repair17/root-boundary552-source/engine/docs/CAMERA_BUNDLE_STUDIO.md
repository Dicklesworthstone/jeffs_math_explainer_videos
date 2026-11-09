# Inspect portable camera scenes in native Studio

Camera-bearing FMTL bundles can be opened by the same `fmn studio` command as
planar artifacts. The scene source and host Python are not required for playback:

```sh
python -m fmn_python scene.py Orbit --format fmtl --bundle-camera \
    --resolution 960x540 --fps 30 --video_dir orbit.fmtl
fmn studio orbit.fmtl --resolution 960x540 --threads 4
```

The compiled artifact's FPS remains authoritative; an explicit conflicting
`--fps` is refused. Timeline scrubbing reconstructs the selected geometry and
recorded camera together through the shared bundle reader. Camera orientation,
zoom, lighting and background are not replaced by Studio's affine defaults.
Resolution changes use the existing camera-bundle viewport adaptation contract.

The worker retains the immutable source, one decoded endpoint cache and one
native renderer, not a pre-rendered PNG movie. Inspector fields describe the
actual reconstructed Stage and its recorded clock. The current inspector does
not supply projected camera-space editing or diagnostic overlays, so mouse/key
editing and affine debug overlays are refused rather than applied incorrectly.
This is inspection and playback of a recorded scene, not live Python execution.

Committed positions use the existing canonical Studio seek command and journal.
Every entry binds both the full artifact's content identity and the resolved
viewport/kernel/build/seed inputs. A failed or divergent replay cannot install
a validated prefix before encountering a bad suffix. Cold worker recovery
reconstructs positions directly from that input-verified seek journal; it does
not claim that a live `SceneState` checkpoint can reconstruct a missing camera
or an original Python callback. Pure seek entries describe immutable artifact
access only; the exporter classification of authored segments is unchanged.

The camera route uses the existing CPU camera kernel. Accelerator requests and
forced affine AA policies are explicit capability refusals, not silent CPU or
coverage substitutions. Frame payloads and inspector documents retain the
existing negotiated protocol limits. One backend identity binds the entire
immutable camera track, avoiding an accumulating identity catalog for each pose.

Acceptance coverage lives in `fmn-studio/tests/camera_bundle_worker.rs` and
`fmn-cli/tests/camera_studio.rs`. The first compares direct Scene captures with
out-of-order worker output; the second exercises the production CLI composition
root and actual isolated `fmn` process, including cold journal recovery with an
empty executable search path. Native compilation/execution must pass before
claiming those cases as validated on a particular source tree.
