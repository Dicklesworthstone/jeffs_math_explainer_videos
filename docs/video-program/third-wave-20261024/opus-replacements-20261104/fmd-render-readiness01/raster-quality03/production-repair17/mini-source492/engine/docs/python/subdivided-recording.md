# Per-play and per-wait native clips

`fmn_python.record_subdivided_scene` records each subsequent selected top-level
`Scene.play` or `Scene.wait` into its own native Reel generation. It is usable
with an already populated Scene and with ordinary `scene.run()` authoring.
It does not replay `construct`, replace the Stage, reseed randomness, or create
another animation clock, renderer, or encoder.

```python
from manimlib import Scene, Square, RIGHT, linear
from fmn_python import record_subdivided_scene

scene = Scene()
square = Square(fill_opacity=1)
scene.add(square)
with record_subdivided_scene(
    scene, "clips", format="gif", resolution=(640, 360), threads=4,
) as recording:
    scene.play(square.animate.shift(RIGHT), run_time=1, rate_func=linear)
    scene.wait(.5)

assert recording.result.completed
for segment in recording.result.segments:
    print(segment.play_index, segment.render.destination, segment.render.digest)
```

The destination directory must be new. Children retain original zero-based
`Scene.num_plays` indices, for example `segment-000003.gif`; PNG sequences use
`segment-000003/`. Source range selection and temporary skipping still execute
the native scene semantics, but omitted segments do not publish clips. An empty
`play()` produces no clip. Nested animation groups remain a single top-level
clip. A selected zero-duration call uses the existing recording contract of
capturing one final frame without advancing time.

Supported formats are **GIF, y4m, PNG sequences, WAV, MP4, and MOV**. GIF, y4m,
and PNG sequences are silent. WAV uses the native mixer/codec; MP4 and MOV
use the existing governed ffmpeg encoder and audio muxer. Automatic prerun is
not provided. CLI subdivision supports one scene or named/write-all batches, but not checkpoint recovery.
Ordinary whole-scene and
live single-clip recording remain available through the existing APIs. FPS
must match the live rational clock; changing `camera.fps` is not resampling.

## Publication and failure

Each clip is an independent atomic, no-clobber publication. A failed or
cancelled segment never rolls back scene effects or deletes earlier completed
clips. `partial_result` identifies those completed clips; it does not claim a
successful collection. Explicit cancellation leaves `result` unset. A rare
receipt failure after native publication is reported through
`partial_result.unreceipted_artifacts`, rather than losing the artifact's path
or counting it as a verified receipt. Such a collection cannot finish as a
success. The default budget is 10,000 completed segments and can be lowered via
`max_segments` (maximum 100,000).

The collection must begin and finish between play/wait calls. It cannot nest
another output owner or steal a recording. Existing mobject identities, live
views, updater registration and native time remain intact. Finish before
restoring checkpoints or resetting the clock. The returned receipts describe
standard-mode execution; recording arbitrary host history is not a certified
input closure.

## Native acceptance

`crates/fmn-python/tests/subdivided_recording.py` exercises the actual installed
extension, including byte equality between concatenated split frames and an
uninterrupted recording at one and four render threads, moving-object pixels,
source ranges, nested groups, all three formats, cancellation, primary-error
preservation, ownership refusals, publication races and partial receipts.
`.github/workflows/subdivided-recording.yml` builds the exact committed source
and runs the installed-wheel tests without source activation patches.

## Fresh scenes and the console

`render_subdivided_scene(SceneClass, "clips", format="gif", fps=24)` runs the
constructor and lifecycle once and returns the collection receipt. It accepts
`scene_kwargs`, `resolution`, `threads`, `animation_range=(start, exclusive_end)`
and `max_segments`. `subdivided_render_session(scene, "clips", fps=24)` provides
the same configured native clock for imperative add/play/wait authoring.
Both require a pristine scene; neither silently discards populated geometry,
live views, prior time, or an active output. Use `record_subdivided_scene` for an
existing live scene instead.

```sh
fmn-python --robot source.py Motion --subdivide --format gif \
  --resolution 640x360 --fps 24 --threads 4 --video_dir clips -n 2,5
```

`--video_dir` names a fresh collection directory. Without it, the default is
`media/videos/<source-stem>/<Scene>/clips/`. Native format, dimension, FPS,
thread and profile parsing remains shared with ordinary renders. The
`--subdivide` lexer does not consume tokens belonging to other option values.
Checkpoint recovery, final-state still selection, and certified output are
explicitly rejected before importing scene source. Multiple named scenes and
`--write_all` use the existing batch API with one `<Scene>/clips/` collection
per job; `--keep-going` retains partial clips while continuing later scenes.
The one terminal robot record has `kind="render-subdivided"` and a
`subdivision` collection receipt. On failure or interruption it includes the
partial collection, its published clip paths and `artifact_published` rather
than claiming rollback or losing successful earlier outputs.

The fresh-scene path needs the matching native wheel, including the native
pristine-scene clock configuration boundary. It never opens and cancels a
dummy output just to configure FPS. `subdivided_rendering.py` adds real native
acceptance for that boundary and the public console entrypoint.

## Audio-bearing clips

```python
from fmn_python import render_subdivided_scene

result = render_subdivided_scene(Motion, "clips", format="mp4", fps=30)
```

The live `record_subdivided_scene` API and console `--subdivide` accept the same
formats. Existing `--vcodec`, `--pix_fmt`, `--ffmpeg_bin`, and transparent MOV
options use the native negotiation and sandboxed publication path unchanged.
A missing encoder is a capability refusal, not a silent format substitution.

Each audio clip uses all cues authored by that clip's completion, including
background cues added before the collection or in an earlier segment. Mixing
keeps absolute scene placement, resampling phase, channel conversion, gain and
ducking. The half-open interval is bounded by independently rounded rational
scene-to-sample endpoints, so adjacent WAV windows neither lose nor repeat a
sample, even at frame rates such as 29 FPS. Long cue tails cannot extend a
clip; absent cues produce exact-duration silence. Skipped calls publish
nothing; the next selected clip samples the background at its later scene
position instead of restarting it. A discontinuous skip inside a single
audio-bearing call is refused rather than flattened into different audio.

A zero-duration WAV clip has zero sample frames. A zero-duration MP4/MOV call
uses the established one-final-video-frame rule and the corresponding bounded
audio interval. AAC encoding may pad decoded output; encoded video is not a
bit-certified artifact. Native WAV samples are the exact mixing boundary.

Streaming publication is deliberate: a later cue with a negative offset cannot
rewrite an already published earlier clip. Sound source bytes are re-read at
clip completion, and each receipt records the actual input content hashes.
Changing an input file affects later publications, not earlier ones. Whole-mix
versus concatenated-WAV equivalence requires the same cue set and source bytes
to be available for every compared interval. Automatic offline prerun that
discovers future authored effects is not implied.

Ordinary `record_scene` is unchanged: a single live insert includes only cues
added during that recording and retains its existing extend-to-cue-end rule.
Subdivision explicitly selects a separate bounded scene-audio mode instead of
silently changing the established recording contract.

`SubdividedRenderResult.sample_frames` and the robot receipt's `sample_frames`
report WAV sample-frame totals separately from video frame counts. Per-clip
receipts include input/PCM hashes and governed encoder/decoder/mux invocation
facts. Failed mixing or muxing cancels that clip without deleting previous
publications. Tests in `subdivided_audio.py` compare real native WAV samples
against the ordinary whole-scene mixer and independently decode motion, alpha
and audio from the MP4/MOV clips.
