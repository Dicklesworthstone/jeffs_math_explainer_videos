"""Real native range/skip capture, endpoint and soundtrack acceptance.

No Scene, clock, animation, renderer, or mixer is substituted. The same cases
can run against a built wheel or from the permanent native embedding suite.
"""
from pathlib import Path
import struct
import sys
import tempfile
import wave

import numpy as np
import manimlib as m

# Embedded acceptance has no wheel package initializer. Install the identical
# shipped hook owner, not a test-only imitation of pre_play/post_play.
_entry = str(Path(__file__).resolve().parents[1] / "python")
sys.path.insert(0, _entry)
try:
    from fmn_python.scene_execution import install_scene_execution
    install_scene_execution(m)
finally:
    sys.path.remove(_entry)

ROOT = Path(tempfile.mkdtemp(prefix="fmn-playback-selection-"))


def render(scene, name, format="y4m", threads=1):
    destination = ROOT / (name + "." + format)
    scene.camera._core.set_pixel_shape(96, 54)
    scene.camera.fps = 8
    scene._begin_native_output(str(destination), format, 96, 54, 8, threads, 0)
    try:
        try:
            scene.run()
        except m.EndScene:
            pass  # Selected-range termination is normal completion.
        report = scene._finish_render()
    except BaseException:
        scene._abort_render()
        raise
    assert Path(report[0]) == destination and destination.exists()
    return destination, report


def frames(path):
    data = path.read_bytes()
    header, body = data.split(b"\n", 1)
    assert header.startswith(b"YUV4MPEG2 ")
    assert b"W96" in header and b"H54" in header and b"F8:1" in header
    size = 96 * 54 * 3 // 2
    result = []
    while body:
        assert body.startswith(b"FRAME\n")
        assert len(body) >= size + 6
        result.append(body[6:size + 6])
        body = body[size + 6:]
    return result


class Motion(m.Scene):
    def construct(self):
        self.square = m.Square(side_length=1, fill_opacity=1, stroke_width=0)
        self.square.shift(-2 * m.RIGHT)
        self.add(self.square)
        self.play(m.Transform(self.square, self.square.copy().shift(m.RIGHT)),
                  run_time=.5, rate_func=m.linear)
        self.play(m.Transform(self.square, self.square.copy().shift(m.RIGHT)),
                  run_time=.5, rate_func=m.linear)
        self.wait(.25)
        self.reached_tail = True

    def tear_down(self):
        self.torn_down = True


def range_is_the_same_frames_after_real_preroll():
    full = Motion()
    all_path, all_report = render(full, "full")
    selected = Motion(start_at_animation_number=1, end_at_animation_number=2)
    selected_path, selected_report = render(selected, "range")
    assert all_report[1] == 10 and selected_report[1] == 4
    assert frames(selected_path) == frames(all_path)[4:8]
    np.testing.assert_allclose(selected.square.get_center(), [0, 0, 0], atol=1e-6)
    assert selected.num_plays == 2 and selected.torn_down
    assert not hasattr(selected, "reached_tail")
    repeat = Motion(start_at_animation_number=1, end_at_animation_number=2)
    repeat_path, _ = render(repeat, "range-threads", threads=4)
    assert selected_path.read_bytes() == repeat_path.read_bytes()


def temporary_skip_keeps_endpoints_and_omits_frames():
    class Temporary(m.Scene):
        def construct(self):
            square = m.Square()
            self.add(square)
            self.wait(.25)
            with self.temp_skip():
                self.play(m.Transform(square, square.copy().shift(m.RIGHT)), run_time=.5)
            self.wait(.25)
            np.testing.assert_allclose(square.get_center(), m.RIGHT, atol=1e-6)
    scene = Temporary()
    path, report = render(scene, "temporary")
    assert report[1] == len(frames(path)) == 4
    assert scene.get_time() == 1 and scene.num_plays == 3
    assert not scene.skip_animations


def skipped_wait_until_keeps_predicate_samples():
    class Conditional(m.Scene):
        def construct(self):
            self.add(m.Square())
            self.skip_animations = True
            self.observed = []
            def stop():
                self.observed.append(self.get_time())
                return self.get_time() >= .25
            self.wait(1, stop_condition=stop)
            self.stop_skipping()
            self.wait(.125)
    scene = Conditional()
    path, report = render(scene, "conditional")
    assert scene.observed == [.125, .25]
    assert report[1] == len(frames(path)) == 1 and scene.get_time() == .375


def final_png_remains_one_frame_when_python_stops_skipping():
    scene = Motion(start_at_animation_number=1, end_at_animation_number=2)
    path, report = render(scene, "still", format="png")
    assert report[1] == 1 and path.read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
    np.testing.assert_allclose(scene.square.get_center(), [0, 0, 0], atol=1e-6)


def soundtrack_uses_output_time_without_losing_original_cue_time():
    cue = ROOT / "cue.wav"
    with wave.open(str(cue), "wb") as stream:
        stream.setnchannels(1)
        stream.setsampwidth(2)
        stream.setframerate(48000)
        stream.writeframes(struct.pack("<h", 4000) * 6000)
    class Sound(m.Scene):
        def construct(self):
            self.skip_animations = True
            self.add_sound(str(ROOT / "deliberately-missing.wav"))
            self.wait(1)
            self.stop_skipping()
            # There is no intervening play: the native previous-segment skip
            # status must not drop this first cue of the selected output.
            self.add_sound(str(cue))
            self.wait(.25)
            with self.temp_skip():
                self.wait(.5)
            self.add_sound(str(cue), time_offset=.0625)
            self.wait(.25)
    scene = Sound()
    path, report = render(scene, "sound", format="wav")
    facts = scene._sound_request_facts()
    assert [(item[1], item[2], item[3]) for item in facts] == [(8, 8, 0.), (14, 8, .0625)]
    with wave.open(str(path), "rb") as stream:
        assert stream.getnframes() == report[1] == 24000
        assert stream.getframerate() == 48000 and stream.getnchannels() == 2
        samples = np.frombuffer(stream.readframes(24000), dtype="<i2").reshape(-1, 2)
    assert np.max(np.abs(samples[:6000])) > 0
    assert not np.any(samples[6000:15000])
    assert np.max(np.abs(samples[15000:21000])) > 0
    assert not np.any(samples[21000:])


def exception_after_selected_capture_does_not_publish():
    class Broken(m.Scene):
        def construct(self):
            self.add(m.Square())
            self.wait(.125)
            raise ValueError("selected segment failed")
    try:
        render(Broken(), "failed")
    except ValueError as error:
        assert str(error) == "selected segment failed"
    else:
        raise AssertionError("scene failure was swallowed")
    assert not (ROOT / "failed.y4m").exists()


def public_render_session_uses_the_same_native_range():
    from fmn_python.rendering import render_scene
    scene = Motion()
    report = render_scene(scene, ROOT / "public.y4m", format="y4m", resolution=(96, 54),
                          fps=8, threads=2, animation_range=(1, 2))
    assert report.frame_count == 4 and report.animation_range == (1, 2)
    assert frames(report.destination) == frames(ROOT / "full.y4m")[4:8]
    assert scene.torn_down and not hasattr(scene, "reached_tail")


def shipped_console_renders_selected_project_frames_and_named_batches():
    import contextlib
    import io
    import json
    from fmn_python.console_rendering import try_render_cli
    project = ROOT / "project"
    project.mkdir()
    source = project / "selected_scene.py"
    source.write_text("""from manimlib import Scene, Square, Transform, RIGHT, linear
class Clip(Scene):
    def __init__(self):
        super().__init__()
    def construct(self):
        square = Square(side_length=1, fill_opacity=1, stroke_width=0)
        square.shift(-2 * RIGHT)
        self.add(square)
        self.play(Transform(square, square.copy().shift(RIGHT)), run_time=.5, rate_func=linear)
        self.play(Transform(square, square.copy().shift(RIGHT)), run_time=.5, rate_func=linear)
        self.wait(.25)
class Other(Clip):
    pass
""")
    def invoke(*options):
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = try_render_cli(m, ["--robot", str(source), "--resolution", "96x54",
                                     "--fps", "8", "--threads", "2", *options])
        assert code == 0, (stdout.getvalue(), stderr.getvalue())
        assert len(stdout.getvalue().splitlines()) == 1
        return json.loads(stdout.getvalue())
    clip = invoke("Clip", "-n", "1,2", "--format", "y4m", "--video_dir", str(ROOT / "cli.y4m"))
    assert clip["frame_count"] == 4 and clip["animation_range"] == [1, 2]
    assert frames(Path(clip["destination"])) == frames(ROOT / "full.y4m")[4:8]
    still = invoke("Clip", "-n1,2", "-s", "--video_dir", str(ROOT / "cli.png"))
    assert still["frame_count"] == 1 and still["format"] == "png"
    assert Path(still["destination"]).read_bytes() == (ROOT / "still.png").read_bytes()
    result = invoke("Other", "Clip", "--start_at_animation_number=1,2", "--format=y4m",
                    "--video_dir", str(ROOT / "cli-batch"))
    outcomes = result["batch"]["outcomes"]
    assert [item["name"] for item in outcomes] == ["Other", "Clip"]
    for item in outcomes:
        assert item["result"]["frame_count"] == 4
        assert frames(Path(item["destination"])) == frames(ROOT / "full.y4m")[4:8]


def range_termination_never_masks_teardown_failure():
    from fmn_python.rendering import render_scene
    class TeardownFailure(Motion):
        def tear_down(self):
            self.torn_down = True
            raise ValueError("teardown failed after selected output")
    for name, scene in (("teardown-selected", TeardownFailure()),
                        ("teardown-full", TeardownFailure())):
        destination = ROOT / (name + ".y4m")
        options = {"animation_range": (1, 2)} if name.endswith("selected") else {}
        try:
            render_scene(scene, destination, format="y4m", resolution=(96, 54),
                         fps=8, threads=1, **options)
        except ValueError as error:
            assert str(error) == "teardown failed after selected output"
        else:
            raise AssertionError("teardown failure was swallowed by normal EndScene")
        assert scene.torn_down and not destination.exists()
    class BothFail(TeardownFailure):
        def construct(self):
            raise KeyError("primary construct failure")
    try:
        render_scene(BothFail(), ROOT / "two-failures.y4m", resolution=(96, 54), fps=8, threads=1)
    except KeyError as error:
        assert error.args == ("primary construct failure",)
    else:
        raise AssertionError("teardown replaced an actual construct failure")
    assert not (ROOT / "two-failures.y4m").exists()


def skipped_preroll_keeps_live_python_updaters_and_camera_samples():
    class Updated(m.Scene):
        def construct(self):
            self.square = m.Square(side_length=1, fill_opacity=1, stroke_width=0)
            self.add(self.square)
            self.ticks = []
            def move(mob, dt):
                self.ticks.append(dt)
                mob.shift(dt * m.RIGHT)
            self.square.add_updater(move)
            self.frame.add_updater(lambda frame, dt: frame.shift(-dt * m.RIGHT), call=False)
            self.wait(.5)
            self.wait(.5)
            self.wait(.25)
    full, selected = Updated(), Updated(start_at_animation_number=1, end_at_animation_number=2)
    full_path, _ = render(full, "updated-full")
    selected_path, report = render(selected, "updated-range")
    assert report[1] == 4
    assert frames(selected_path) == frames(full_path)[4:8]
    assert [tick for tick in selected.ticks if tick] == [.125] * 8
    np.testing.assert_allclose(selected.square.get_center(), m.RIGHT, atol=1e-6)
    np.testing.assert_allclose(selected.frame.get_center(), -m.RIGHT, atol=1e-6)


CASES = (
    range_is_the_same_frames_after_real_preroll,
    temporary_skip_keeps_endpoints_and_omits_frames,
    skipped_wait_until_keeps_predicate_samples,
    final_png_remains_one_frame_when_python_stops_skipping,
    soundtrack_uses_output_time_without_losing_original_cue_time,
    exception_after_selected_capture_does_not_publish,
    public_render_session_uses_the_same_native_range,
    shipped_console_renders_selected_project_frames_and_named_batches,
    range_termination_never_masks_teardown_failure,
    skipped_preroll_keeps_live_python_updaters_and_camera_samples,
)
assert len(CASES) == 10
for case in CASES:
    case()
    print("native playback selection:", case.__name__)
print(f"native playback selection: {len(CASES)} cases passed; artifacts: {ROOT}")
