"""Real-extension camera-pose and choreography acceptance; no fixture storage."""
import math
import numpy as np
from manimlib import CameraFrame, Transform, Scene, Square, AnimationGroup, Succession, ValueTracker, linear


def state(frame):
    return (frame.get_center().copy(), frame.get_shape(), frame.get_field_of_view(),
            frame.get_orientation().copy())


def assert_state(frame, expected):
    for actual, wanted in zip(state(frame), expected):
        np.testing.assert_allclose(actual, wanted, atol=1e-10, rtol=1e-10)


def pose_interpolates_every_native_component():
    live = CameraFrame()
    start = live.copy()
    target = live.copy().shift((4., 2., -2.)).scale(.5).set_field_of_view(math.pi / 2.)
    target.set_orientation((0., 0., 1., 0.))
    core = live._core
    assert live.interpolate(start, target, .5) is live
    assert live._core is core
    np.testing.assert_allclose(live.get_center(), [2., 1., -1.])
    np.testing.assert_allclose(live.get_shape(), .75 * np.array(start.get_shape()))
    np.testing.assert_allclose(live.get_orientation(), [0., 0., math.sqrt(.5), math.sqrt(.5)])
    assert math.isclose(live.get_field_of_view(), 3. * math.pi / 8.)
    live.interpolate(start, target, 1.)
    assert_state(live, state(target))


def direct_transform_uses_pose_not_empty_records():
    live = CameraFrame()
    target = live.copy().shift((4., 0., 0.)).scale(.5)
    animation = Transform(live, target, rate_func=linear)
    animation.begin()
    animation.interpolate(.5)
    np.testing.assert_allclose(live.get_center(), [2., 0., 0.])
    np.testing.assert_allclose(live.get_shape(), np.array(target.get_shape()) * 1.5)
    animation.finish()
    assert_state(live, state(target))
    assert not live.is_changing()
    assert live.locked_data_keys == set()


def native_camera_locks_and_failure_are_atomic():
    live = CameraFrame()
    start, target = live.copy(), live.copy().shift((4., 0., 0.))
    before = state(live)
    for callback in (lambda a, b, t: np.zeros((5, 3)),
                     lambda a, b, t: np.full((5, 3), math.nan)):
        try:
            live.interpolate(start, target, .5, callback)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid camera path was accepted")
        assert_state(live, before)
    live.locked_data_keys.add("point")
    live.interpolate(start, target, 1.)
    assert_state(live, before)
    live.locked_data_keys.clear()


def closed_camera_excursion_uses_reference_control_points():
    live = CameraFrame()
    start = live.copy()
    seen = []
    def path(a, b, t):
        seen.append((a.shape, b.shape, t))
        return (1. - t) * a + t * b + np.array([0., 4. * t * (1. - t), 0.])
    live.interpolate(start, start, .5, path)
    np.testing.assert_allclose(live.get_center(), [0., 1., 0.])
    live.interpolate(start, start, 1., path)
    assert_state(live, state(start))
    assert seen == [((5, 3), (5, 3), .5), ((5, 3), (5, 3), 1.)]


def record_camera(scene):
    samples = []
    live = scene.frame
    def record(frame, dt):
        if frame is live and dt > 0:
            samples.append(frame.get_center().copy())
    live.add_updater(record, call=False)
    return samples


def camera_builder_owns_runtime_and_renderer_identity():
    scene = Scene()
    live, core = scene.frame, scene.frame._core
    samples = record_camera(scene)
    scene.play(live.animate(run_time=4 / 30, rate_func=linear).shift((4., 0., 0.)))
    np.testing.assert_allclose(np.array(samples)[:, 0], [1., 2., 3., 4.], atol=1e-8)
    assert math.isclose(scene.time, 4 / 30)
    assert scene.frame is live and scene.camera.frame is live and live._core is core
    assert not live._is_bound()
    assert scene.mobjects == []


def nested_camera_succession_uses_predecessor_pose():
    scene = Scene()
    live = scene.frame
    first = Transform(live, live.copy().shift((1., 0., 0.)), run_time=2 / 30, rate_func=linear)
    second = Transform(live, live.copy().shift((3., 0., 0.)), run_time=4 / 30, rate_func=linear)
    samples = record_camera(scene)
    group = AnimationGroup(Succession(first, second))
    scene.play(group)
    # BN-02: the binary-float sum 0.2 lies above 1/5, so the rational
    # clock emits a seventh sample which holds the final state.
    np.testing.assert_allclose(np.array(samples)[:, 0], [.5, 1., 1.5, 2., 2.5, 3., 3.], atol=1e-8)
    np.testing.assert_allclose(second.starting_mobject.get_center(), [1., 0., 0.])
    assert first.mobject is second.mobject is live
    assert not live._is_bound()
    assert scene.mobjects == []


def authored_camera_path_runs_during_native_drawable_coplay():
    scene = Scene()
    square = Square()
    scene.add(square)
    samples = []
    def path(a, b, alpha):
        return (1. - alpha) * a + alpha * b + [0., 4. * alpha * (1. - alpha), 0.]
    def record(frame, dt):
        if frame is scene.frame and dt > 0:
            samples.append((frame.get_center().copy(), square.get_center().copy()))
    scene.frame.add_updater(record, call=False)
    scene.play(Transform(square, square.copy().shift((4., 0., 0.)), rate_func=linear),
               Transform(scene.frame, scene.frame.copy(), path_func=path, rate_func=linear),
               run_time=4 / 30)
    np.testing.assert_allclose([sample[0][1] for sample in samples], [.75, 1., .75, 0.], atol=1e-8)
    np.testing.assert_allclose([sample[1][0] for sample in samples], [1., 2., 3., 4.], atol=1e-6)
    assert scene.mobjects == [square]
    assert not scene.frame._is_bound()


def camera_authored_lifecycle_keeps_source_identity():
    scene = Scene()
    calls = []
    class Authored(Transform):
        def begin(self):
            calls.append(("begin", self.mobject))
            super().begin()
        def interpolate_submobject(self, current, starting, target, alpha):
            calls.append(("interpolate", current))
            return super().interpolate_submobject(current, starting, target, alpha)
    animation = Authored(scene.frame, scene.frame.copy().shift((2., 0., 0.)), run_time=2 / 30, rate_func=linear)
    scene.play(AnimationGroup(animation))
    assert calls[0] == ("begin", scene.frame)
    assert all(frame is scene.frame for _, frame in calls)
    assert sum(name == "interpolate" for name, _ in calls) >= 4
    np.testing.assert_allclose(scene.frame.get_center(), [2., 0., 0.])


def camera_play_keeps_present_roots_in_draw_order():
    # fm-kpvx: like the Reference's begin_animations, a camera play adds
    # only animated roots missing from the scene; a present root keeps its
    # place in the draw order.
    scene = Scene()
    first, tracker, last = Square(), ValueTracker(0.), Square()
    scene.add(first, tracker, last)
    order = [id(first), id(tracker), id(last)]
    scene.play(first.animate.shift((1., 0., 0.)), scene.frame.animate.shift((1., 0., 0.)), run_time=2 / 30)
    assert [id(m) for m in scene.mobjects] == order
    scene.play(tracker.animate.set_value(1.), scene.frame.animate.shift((-1., 0., 0.)), run_time=2 / 30)
    assert [id(m) for m in scene.mobjects] == order
    fresh = Square()
    scene.play(fresh.animate.shift((0., 1., 0.)), scene.frame.animate.shift((0., 1., 0.)), run_time=2 / 30)
    assert [id(m) for m in scene.mobjects] == order + [id(fresh)]


def authored_timing_table_controls_camera_delay():
    scene = Scene()
    animation = Transform(scene.frame, scene.frame.copy().shift((4., 0., 0.)), run_time=4 / 30, rate_func=linear)
    group = AnimationGroup(animation, run_time=4 / 30)
    group.anims_with_timings = [(animation, 2 / 30, 4 / 30)]
    group.max_end_time = 4 / 30
    samples = record_camera(scene)
    scene.play(group)
    np.testing.assert_allclose(np.array(samples)[:, 0], [0., 0., 2., 4.], atol=1e-8)


def camera_rate_curve_final_alpha_and_time_span_are_live():
    scene = Scene()
    samples = record_camera(scene)
    scene.play(Transform(scene.frame, scene.frame.copy().shift((4., 0., 0.)),
                         run_time=4 / 30, rate_func=lambda alpha: alpha * alpha,
                         final_alpha_value=.5))
    np.testing.assert_allclose(np.array(samples)[:, 0], [.25, 1., 2.25, 4.], atol=1e-8)
    np.testing.assert_allclose(scene.frame.get_center(), [1., 0., 0.], atol=1e-8)
    scene = Scene()
    samples = record_camera(scene)
    scene.play(Transform(scene.frame, scene.frame.copy().shift((4., 0., 0.)),
                         run_time=4 / 30, time_span=(2 / 30, 4 / 30), rate_func=linear))
    np.testing.assert_allclose(np.array(samples)[:, 0], [0., 0., 2., 4.], atol=1e-8)


def group_suspends_only_owned_live_camera_updaters():
    scene = Scene()
    ticks = []
    def record(frame, dt):
        if frame is scene.frame:
            ticks.append(dt)
    scene.frame.add_updater(record, call=False)
    animation = Transform(scene.frame, scene.frame.copy().shift((2., 0., 0.)), run_time=2 / 30, rate_func=linear)
    scene.play(AnimationGroup(animation, suspend_mobject_updating=True))
    assert ticks and all(dt == 0 for dt in ticks)
    assert not scene.frame._is_updating_suspended()
    ticks.clear()
    scene.frame.suspend_updating()
    scene.play(AnimationGroup(Transform(scene.frame, scene.frame.copy(), run_time=2 / 30),
                              suspend_mobject_updating=True))
    assert scene.frame._is_updating_suspended()
    assert ticks == []
    scene.frame.resume_updating()


def camera_failure_unwinds_and_allows_next_play():
    scene = Scene()
    square = Square()
    scene.add(square)
    failure = RuntimeError("camera-native-acceptance-failure")
    def path(a, b, alpha):
        if alpha > 0:
            raise failure
        return a
    try:
        scene.play(Transform(square, square.copy().shift((2., 0., 0.))),
                   Transform(scene.frame, scene.frame.copy(), path_func=path,
                             suspend_mobject_updating=True), run_time=2 / 30)
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError("authored camera failure was swallowed")
    assert not scene.frame._is_updating_suspended()
    assert not scene.frame._is_bound()
    assert scene.mobjects == [square]
    scene.play(scene.frame.animate.shift((2., 0., 0.)), run_time=2 / 30, rate_func=linear)
    np.testing.assert_allclose(scene.frame.get_center(), [2., 0., 0.], atol=1e-8)
    assert scene.mobjects == [square]


def nested_camera_changes_real_rendered_frames():
    import contextlib
    import io
    import json
    import pathlib
    import sys
    import tempfile
    import manimlib
    root = pathlib.Path(globals().get("_fmn_output_root") or tempfile.mkdtemp(prefix="fmn-camera-choreography-"))
    root.mkdir(parents=True, exist_ok=True)
    source, destination = root / "camera_choreography.py", root / "camera_choreography.y4m"
    source.write_text("""from manimlib import *
class NestedCamera(Scene):
    def construct(self):
        self.add(Square(side_length=1, fill_color=WHITE, fill_opacity=1, stroke_width=0))
        self.play(AnimationGroup(Succession(
            Transform(self.frame, self.frame.copy().shift(RIGHT), run_time=.25, rate_func=linear),
            Transform(self.frame, self.frame.copy().shift(2 * RIGHT), run_time=.25, rate_func=linear))))
""")
    previous = sys.argv
    stdout, stderr = io.StringIO(), io.StringIO()
    sys.argv = ["fmn-python", "--robot", str(source), "NestedCamera", "--format", "y4m",
                "--resolution", "96x54", "--fps", "8", "--threads", "1", "--video_dir", str(destination)]
    try:
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = manimlib._console_main()
    finally:
        sys.argv = previous
    report = json.loads(stdout.getvalue())
    assert code == 0 and report["frame_count"] == 4, report
    assert stderr.getvalue() == "", stderr.getvalue()
    header, payload = destination.read_bytes().split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2", header
    stride = 6 + 96 * 54 * 3 // 2
    assert len(payload) == 4 * stride
    centers = []
    for offset in range(0, len(payload), stride):
        assert payload[offset:offset + 6] == b"FRAME\n"
        luma = np.frombuffer(payload[offset + 6:offset + 6 + 96 * 54], dtype=np.uint8).reshape(54, 96)
        ys, xs = np.nonzero(luma > 200)
        assert len(xs) > 10
        assert abs(float(ys.mean()) - 26.5) < 1.
        centers.append(float(xs.mean()))
    # World geometry does not move. These independent pixel coordinates
    # therefore witness the real renderer consuming every camera pose.
    np.testing.assert_allclose(centers, [44.125, 40.75, 37.375, 34.], atol=.75, rtol=0)
    assert all(a - b > 2. for a, b in zip(centers, centers[1:])), centers


_CASES = [pose_interpolates_every_native_component, direct_transform_uses_pose_not_empty_records,
          native_camera_locks_and_failure_are_atomic, closed_camera_excursion_uses_reference_control_points,
          camera_builder_owns_runtime_and_renderer_identity, nested_camera_succession_uses_predecessor_pose,
          authored_camera_path_runs_during_native_drawable_coplay, camera_authored_lifecycle_keeps_source_identity,
          camera_play_keeps_present_roots_in_draw_order,
          authored_timing_table_controls_camera_delay, camera_rate_curve_final_alpha_and_time_span_are_live,
          group_suspends_only_owned_live_camera_updaters, camera_failure_unwinds_and_allows_next_play,
          nested_camera_changes_real_rendered_frames]
for _case in _CASES:
    _case()
print(f"camera animation acceptance: {len(_CASES)} cases passed")
