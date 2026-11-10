# Export camera scenes from the console

The installed host-Python portal can record a user scene as a portable camera
bundle. No exporter code has to be added to the authored scene:

```sh
python -m fmn_python --robot scene.py Orbit --format fmtl --bundle-camera \
    --resolution 960x540 --fps 30 --video_dir orbit.fmtl

fmn --robot orbit.fmtl --format y4m --resolution 960x540 --threads 4 \
    --video_dir replay
```

`--bundle-camera` selects the existing minor-1 recorder, the same path as
`export_bundle(Orbit, "orbit.fmtl", camera=True)`. Camera motion, lighting and
background are captured alongside vector, surface, point-cloud and image
geometry after the ordinary frame/updater boundary. The native consumer needs
neither the original source nor a Python interpreter. Keep the capture
resolution for exact replay; changing pixel size is supported by the native
camera player, but it is a different output.

The flag requires exactly one `--format fmtl` and may appear only once. The
camera recording capability is checked before importing or constructing the
source. Unsupported range, skipping, presenter, batch and certification modes
remain refusals; selecting camera recording does not silently enable them.
Audio is not part of either bundle format.

The default command without `--bundle-camera` keeps the planar minor-0 format
and its existing receipt. Camera mode adds `camera_track: true` and
`fmtl_minor: 1` inside the robot receipt's `bundle` object. Both modes report
`certified_source: false`: recording observed frames does not certify arbitrary
Python source effects. Neither mode overwrites an existing destination or
publishes a partial artifact after an authored/capture failure.

The current camera-less WASM player refuses minor-1 bundles. Use the planar
format for that player, or the native `fmn` player for camera content.

`camera_bundle_console.py` compares actual console exports with programmatic
native exports, including non-16:9 and portrait output, moving camera/updaters,
robot output, option refusals and failed-generation recovery. The separate
`camera_bundle_cli_native.py` acceptance program compares camera bundle replay
through the actual standalone binary with direct native Y4M rendering.
