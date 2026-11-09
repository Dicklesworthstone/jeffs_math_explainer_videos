# Python-authored standalone scene bundles

`fmn_python.export_bundle`, `BundleExportSession`, and `fmn-python --format fmtl`
connect the optional Python authoring portal to the same FMTL/1 reader used by
standalone `fmn` and the WASM timeline player. The host interpreter runs the
scene once. Actual post-updater snapshots are recorded; playback does not need
Python, execute callbacks, approximate a rate function, or re-run the source.

## Export and replay

With a matching installed wheel and native binary:

```sh
fmn-python crates/fmn-python/examples/portable_scene.py PortableScene \
  --format fmtl --resolution 960x540 --fps 30 --video_dir portable.fmtl

fmn portable.fmtl --format png_sequence --resolution 960x540 \
  --video_dir replayed
```

The portal's `--video_dir` names the **exact file** for a single export. The
native replay command's `--video_dir` names an **output directory**. Without a
portal destination, the export is `media/videos/SOURCE_STEM/SCENE.fmtl`.
One source and one scene are accepted; the scene name can be omitted when that
source declares exactly one scene. `--robot` reserves stdout for one JSON
receipt; authored import/setup/construct/teardown chatter goes to stderr.
`--format=fmtl` is also accepted.

The bundle stores its rational FPS and exact frame schedule, but **not the
output resolution**. Replay with the same resolution to preserve framing and
pixel dimensions. A conflicting explicit FPS is rejected by the native player.
The native/WASM readers retain their own resource limits and capability checks.

## Library API

```python
from fmn_python import export_bundle
from portable_scene import PortableScene

receipt = export_bundle(
    PortableScene,
    "portable.fmtl",
    resolution=(960, 540),
    fps=30,
    max_frames=30_000,
    max_capture_bytes=64 * 1024 * 1024,
    max_output_bytes=64 * 1024 * 1024,
)
print(receipt.frame_count, receipt.digest)
```

A Scene class is constructed once. A supplied Scene instance must still be
pristine. `scene_kwargs` applies only to Scene classes. The ordinary `run()`
lifecycle owns setup, construct, and teardown; successful `EndScene` termination
is retained. Alternatively, use `BundleExportSession(scene, path, ...)` as a
context manager and author operations inside it. That context does not call
`run()` implicitly.

The recorded artifact preserves stateful Python updater output, arbitrary
supported easing callbacks, play/wait boundaries, explicit captures, and the
actual sampled frame count. Empty/static scenes get one terminal still; an
empty `play()` creates no phantom segment. Recorded segments are snapshots,
not assertions that a callback is pure or serializable.

## Explicit boundaries

FMTL/1 export currently admits **planar vector content**, including native text
and math outlines, at the default camera, light, and background. Camera motion,
non-default camera/background/light state, depth, nonplanar geometry, raster
primitives, clip planes, shading, and audio are refused rather than silently
dropped. Use the existing PNG/video route for those scenes. Not every aspect
ratio can preserve the current fixed-frame mapping; camera validation remains
authoritative. The examples use the default 16:9 frame.

Skipping, animation ranges, presenter mode, subdivided output, and source
certification are not supported by this full-scene export command. The export
is serial authoring; set playback thread counts on `fmn`, not on the exporter.
The receipt explicitly says `certified_source: false`: storing snapshots does
not attest arbitrary Python imports, asset reads, or side effects.

Capture defaults are one million frames and 256 MiB of charged snapshot/table
storage; output defaults to the production reader's 256 MiB cap. Both byte
budgets are independently adjustable downward in the API. They are not a bound
on arbitrary authored Python allocations or execution time. In-flight native
animation cleanup retains its existing semantics after a capture failure.

No destination is published until execution, bounded encoding, and production
reader validation succeed. Publication is atomic and create-only, including a
race with another writer after initial checks. An existing file, directory, or
symlink is never replaced. An updater/capture/segment failure stays fatal even
when authored code catches it: truncated animation is not a successful export.

## Validation entry points

The host-only tests explicitly spy on native output boundaries:

```sh
python -m unittest discover -s crates/fmn-python/tests -p 'test_bundle*protocol.py' -v
```

The native integration test exercises the actual shared recorder/reader and
compares every decoded RGBA frame with independent direct rendering:

```sh
cargo test --locked -p fmn-python portal_studio::bundle -- --nocapture
```

The installed-wheel/standalone test exercises both public entry points and
compares complete Y4M output at 24/30/60 FPS and one/four replay workers:

```sh
FMN_TEST_BIN="$PWD/target/portable/debug/fmn" \
  python crates/fmn-python/tests/bundle_cli_native.py
```

These are validation commands, not a claim that the native build has passed.
The implementation session ran the Python protocol suites; Rust and fresh-wheel
acceptance require the matching pinned-toolchain build.
