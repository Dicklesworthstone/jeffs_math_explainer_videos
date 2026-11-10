# Record a live scene or a checkpointed edit

`record_scene` attaches native Lumen/Reel output to a scene that already owns
mobjects and has advanced its clock. It does not replay `construct`, replace
the arena, reseed randomness, change the camera, or write frames in Python.
The existing `render_session` still configures a pristine full render.

```python
from manimlib import Scene, Square, RIGHT
from fmn_python import record_scene

scene = Scene()
square = Square(fill_opacity=1)
scene.add(square)
scene.wait(1)

with record_scene(scene, "first.y4m", resolution=(960, 540), threads=4) as clip:
    scene.play(square.animate.shift(RIGHT), run_time=1)
print(clip.result.as_dict())
print(clip.start_frame, clip.end_frame)

with record_scene(scene, "second.gif", resolution=(480, 270), threads=2) as clip:
    scene.play(square.animate.shift(RIGHT), run_time=1)
```

The live native clock is authoritative. Omit `fps` to inherit it; an explicitly
different FPS is refused rather than resetting or resampling the scene.
Changing `camera.fps` alone does not resample a live scene. Clip frame
numbers start at zero in the output, while Scene time keeps advancing. Skipped
segments keep their existing semantics and do not contribute output time.

Native PNG sequences, GIF, Y4M, WAV, MP4 and MOV are supported. MP4/MOV reuse
the existing negotiated `SceneFileWriter` codec, pixel format and ffmpeg
boundary. A transparent MOV retains alpha. Missing ffmpeg is a capability
error, not silent format substitution. Only sound cues added after recording
starts are included, rebased to the clip's origin. WAV requires at least one
new cue. Empty visual recordings capture the current frame once at finish.
Endpoint-only PNG/SVG output is deliberately excluded: its skip-to-final
behavior would change the live scene's sampled updater execution. Use camera
readback for a still, or the existing pristine-scene render API.

## Record a checkpointed edit

```python
from fmn_python import SceneConsole

with SceneConsole(scene, {"square": square, "RIGHT": RIGHT}, capture=False) as editor:
    editor.run_cell(
        "# placement\nscene.play(square.animate.shift(RIGHT), run_time=1)",
        record_to="placement-v1.y4m",
        recording_options={"resolution": (960, 540), "threads": 4},
    )
    print(editor.last_recording.as_dict())
    editor.run_cell(
        "# placement\nscene.play(square.animate.shift(2 * RIGHT), run_time=1)",
        record_to="placement-v2.y4m",
        recording_options={"resolution": (960, 540), "threads": 4},
    )
```

The second cell restores the named checkpoint **before** opening its recording.
It does not accumulate the first cell's shift. `capture=False` disables the
separate preview/readback channel, not native file recording. The same
`record_to`/`recording_options` arguments work with `SceneConsole.checkpoint_paste`
and the active embedded shell's `checkpoint_paste` shortcut. Clipboard access
remains an explicit host capability. `record=True` without a destination still
refuses to guess where an insert should be written.

## Ownership, errors and limits

A recording is confined to the creating thread and begins/finishes between
play/wait calls. An existing render or Studio preview generation cannot be
stolen. Finish or abort a recording before restoring Scene checkpoints. A
successful context sets `result`; finishing again returns the same receipt.
`abort()` is idempotent. Exceptions and interrupts cancel unpublished output
and retain their identity, but do **not** rewind actual scene or host effects.
A recorded cell failure retains the previous successful `last_recording`.

Every clip is a no-clobber publication. Retry a successful edit with a new
output path: an existing file or frame directory is never overwritten.
Native frame, pixel, memory and encoder limits continue to apply. The public
adapter does not implement another encoder, timer, subprocess or frame loop.
Recordings are standard-mode artifacts, not a certified replay of arbitrary
interactive host history. The original render/session API still configures
pristine scenes and remains appropriate for complete lifecycle renders.
