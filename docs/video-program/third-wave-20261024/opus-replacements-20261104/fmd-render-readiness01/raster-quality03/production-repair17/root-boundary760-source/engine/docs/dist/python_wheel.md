# Python Wheel and `manimlib` Namespace Policy (W11)

The `franken-manim` wheel is the optional Python portal described by
ADR-0015 and ADR-0017. It is not the standalone `fmn` distribution: the wheel
requires a supported host CPython and NumPy, while the native binary remains
CPython-free.

## ABI and dependency policy

- The declared interpreter range is CPython `>=3.13,<3.14`.
- Wheels use the full CPython 3.13 ABI (`cp313-cp313`), never `abi3`. The
  portal's audited buffer slots and read-only `tp_version_tag` observer depend
  on the full interpreter ABI.
- The wheel requires exactly NumPy 2.5.2. NumPy is a host-side Python
  dependency; it does not enter the standalone Rust dependency closure.
- The build backend is exactly Maturin 1.14.1. It is a source-build tool, not
  a runtime dependency.
- Wheels use the portable CPU tier. Platform- or microarchitecture-specific
  acceleration must not make an ordinary wheel fail on an otherwise supported
  host.
- The wheel bundles the Rust engine and its compiled-in font/math assets,
  ships the engine license and all bundled-font OFL texts, and does not bundle
  CPython, NumPy, ffmpeg, the private Reference corpus, or a second renderer.

Native Linux x86-64, macOS AArch64, and Windows x86-64 wheels have been built
and smoke-tested from exact source commits; immutable size and digest receipts
are recorded below, including the published `v0.2.0` release-tag artifacts.
The published `v0.3.0` receipts additionally cover the atomic final-state PNG
route and all eight locked source-unedited corpus scenes.
Linux AArch64 remains unproven because no native
certification host is currently configured, so `fm-vsq` stays open rather than
substituting cross-compilation or emulation for native evidence.

## Namespace ownership

Installing `franken-manim` makes this distribution the sole owner of the
top-level `manimlib` package in that environment. The package contains the
private full-ABI extension `manimlib.manimlib` and re-exports the exact 663-name
root schema. It deliberately has no `__all__`, matching the pinned Reference's
wildcard behavior. The separate `fmn_python` package owns console composition
and does not widen the compatibility namespace.

This is an exclusive namespace, not a PEP 420 namespace package. Installing
another distribution which writes `manimlib` into the same environment is
unsupported: package installers can overwrite files without providing a
coherent semantic winner, and uninstalling either distribution can remove
files needed by the other. FrankenManim therefore promises no import-order or
last-install-wins behavior. Use one dedicated virtual environment per
`manimlib` provider. This is a concrete collision, not a hypothetical one:
[PyPI's `manimlib==0.2.0`](https://pypi.org/project/manimlib/) ships its own
`manimlib/__init__.py` and package tree. The predictable supported states are:

| Environment state | Result |
|---|---|
| only `franken-manim` installed | `import manimlib` resolves to FrankenManim |
| no `manimlib` provider installed | ordinary `ModuleNotFoundError` |
| another `manimlib` provider installed | use that provider without `franken-manim` |
| both providers installed | unsupported package-file collision; create separate environments |

The distribution publishes `manimlib.__distribution__ == "franken-manim"`,
`manimlib.__franken_manim__ is True`, and
`manimlib.__abi_policy__ == "cpython-3.13-full-abi"` so diagnostics can verify
which supported provider is active. When FrankenManim's package files remain
loadable and another installed distribution's metadata also claims the
`manimlib` tree, both direct import and the console fail before native-module
loading. Robot mode reports exit 4 and kind `namespace-collision`. This is a
diagnostic for a detectable broken environment, not support for co-installing
providers: another installer may already have overwritten the very wrapper
which performs the check.

## Console boundary

The wheel installs `fmn-python` and supports these currently implemented
operations:

```text
fmn-python [--robot] --version
fmn-python [--robot] --list-scenes SOURCE.py
fmn-python [--robot] --construct-only SOURCE.py [SCENE]
fmn-python [--robot] SOURCE.py [SCENE ...] [--format png_sequence|png|gif|y4m|wav|mp4|mov]
           [--resolution WIDTHxHEIGHT] [--fps FPS] [--threads N]
           [--video_dir PATH] [--transparent]
           [--vcodec ENCODER] [--pix_fmt FORMAT] [--ffmpeg_bin PATH]
```

`--construct-only` is an explicit engine-lifecycle diagnostic and reports
`rendered=false`; it does not claim pixels or output files. The standard
PNG-sequence route captures immutable Python-scene lifecycle frames, renders
them through the shared retained Lumen CPU renderer, and publishes one atomic,
no-clobber generation through Reel. The `png` route uses the same production
path and atomically publishes the final captured frame as one still. GIF and
y4m use that retained renderer, clock and ordered emitter with Reel's native
sinks. GIF records rational-frame delays in centiseconds; y4m uses limited-range
BT.709 YUV 4:2:0 with left chroma siting and requires even dimensions. These
success records report `rendered=true`, frame and byte counts, the artifact
digest (ordered-tree digest for a sequence), engine identity, and render-team
width. `--video_dir` names the output directory for a PNG sequence and the
destination file for the other formats.

WAV output runs the sampled scene lifecycle without rasterization and mixes
`Scene.add_sound` cues through Reel. PCM/float WAV inputs remain native-only;
compressed audio uses the optional governed decoder described in
[Python audio](python_audio.md). Missing, malformed or over-budget audio fails before atomic publication. At
least one cue is required. Output is 48 kHz, stereo, signed 16-bit PCM, with
frame-clock cue placement, `time_offset`, `gain`, and `gain_to_background`.
Robot output identifies `native-sound-mixer` and reports `sample_frames`,
`sample_rate`, and `channels`; its `frame_count` also counts sample frames.
The permanent native, installed-wheel and registered E2E suites independently
decode GIF/y4m/WAV bytes and check motion, primary colors, cue placement,
thread-count replay and preservation of destinations after failures.

MP4 and MOV use Reel's content-hashed, governed ffmpeg process boundary, not a
Python subprocess encoder. `SceneFileWriter.ffmpeg_bin`, `video_codec`, and
`pixel_format` are negotiated before acquiring an output generation. The
default is libx264 with NV12 transport; `auto` selects the container's software
encoder. `--pix_fmt` selects the native **input wire**, not arbitrary ffmpeg
filters: `rgba`/`rgba8`, `bgra`/`bgra8`, `nv12`/`yuv420p`, or
`p010`/`p010le`/`yuv420p10le`. P010 retains 10-bit transport/output planes; it
does not assert an HDR pipeline or additional source color precision.

`--transparent` (`-t`) preserves the scene's RGB background and camera pose,
changing only background alpha. PNG and PNG sequences carry alpha natively.
MOV automatically promotes the ordinary writer defaults to RGBA/qtrle; an
explicit transparent video profile must use RGBA/BGRA and qtrle/auto. MP4,
GIF, y4m and WAV reject this switch rather than silently flattening it.
Ordinary video requires even dimensions. Alpha cannot be introduced midway
through an already-negotiated opaque video generation.

```bash
fmn-python scene.py Example -s --transparent --video_dir still.png
fmn-python scene.py Example --format mov -t --pix_fmt bgra --video_dir alpha.mov
fmn-python scene.py Example --format mp4 --vcodec libx264 --pix_fmt p010le --video_dir tenbit.mp4
fmn-python scene.py First Second --format mov -t --video_dir batch
```

All selected scenes receive these overrides **after** their normal constructors
run, including constructors that accept no keyword arguments. `--write_all`
and named batches retain independent atomic destinations and ordered receipts.
For programmatic rendering, set `camera_config={"background_opacity": 0}` and
the corresponding `file_writer_config` on a Scene, then call
`fmn_python.render_scene(scene, "alpha.mov")`. An explicit `ffmpeg_bin` can
contain spaces and is resolved without shell tokenization. Invalid profiles,
missing encoders and failed frames do not replace existing destinations.

Python `studio` now captures existing scenes in disposable host-CPython workers:

```bash
fmn-python studio scene.py Example --resolution 640x360 --fps 30
```

The authenticated native browser UI supports playback, reverse scrubbing,
per-frame native inspection and explicit source reload. Preview is bounded,
silent and read-only; it does not expose arbitrary callback checkpoints or
live `InteractiveScene` editing. See [Python Studio](python_studio.md) for the
programmatic context manager, resource controls and worker-failure semantics.

Certified output (`--reproducible`) and opener flags remain fail-closed
capability errors. In particular, the portal
does not expose a partial certified path before the Python input closure and
provenance sidecar are complete, and it does not label lifecycle-only work as
a render. Python scenes execute with the host interpreter's full user
authority; the portal is not a sandbox.

`--robot` emits one compact, deterministic JSON record using schema
`fmn-python.cli`, version 1.

## Pinned build and smoke ritual

From a clean checkout, with the pinned nightly and CPython 3.13 available:

```bash
SOURCE_DATE_EPOCH=<commit-time> \
  uvx --from 'maturin==1.14.1' maturin build \
    --release --locked --interpreter /path/to/python3.13 --out /tmp/wheels

uv venv --python /path/to/python3.13 /tmp/fmn-wheel-smoke
uv pip install --python /tmp/fmn-wheel-smoke/bin/python \
  /tmp/wheels/franken_manim-*.whl
/tmp/fmn-wheel-smoke/bin/python -c \
  'import manimlib; assert manimlib.__franken_manim__'
/tmp/fmn-wheel-smoke/bin/fmn-python --robot --version
/tmp/fmn-wheel-smoke/bin/python crates/fmn-python/tests/wheel_smoke.py \
  --wheel /tmp/wheels/franken_manim-*.whl \
  --schema API_SCHEMA.tsv \
  --scene crates/fmn-python/tests/console_scene.py \
  --probe-collision
```

The final command runs the installed-artifact contract, then creates one inert
foreign-provider `.dist-info` fixture inside that disposable environment to
prove the collision refusal. Run it last; it intentionally leaves that virtual
environment in the refused state and never edits either provider's package
files.

The permanent release matrix must additionally verify the wheel tag, exact
663-name wildcard surface, license inventory, clean-venv scene discovery,
production pixel render, certified-mode refusal, and the exclusive-namespace
states above. Current measured size evidence lives in
[`PYTHON_WHEEL_SIZE.tsv`](PYTHON_WHEEL_SIZE.tsv).

The source distribution is also buildable: its include set carries the two
root API-schema inputs needed by `fmn-python`, and rebuilding that sdist under
the same locked toolchain produces an installable wheel. This does not yet
constitute a byte-reproducible release proof. Maturin prunes unrelated Cargo
workspace members when it writes the sdist manifest, so the resulting native
extension is not byte-identical to the checkout-built extension; its generated
CycloneDX SBOM also records absolute `file://` source URIs. The functional
round trip is observed, while eliminating those two artifact-identity drifts
remains open `fm-vsq`/W11 work.
