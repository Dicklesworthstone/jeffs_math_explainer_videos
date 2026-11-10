"""Installed-wheel execution acceptance with actual native objects and output.

The protocol fixtures do not establish these assertions. This suite requires
the extension and exercises real callback release windows, suspension, native
clock ticks, public Scene hooks and Lumen/Reel output.
"""
from __future__ import annotations

import tempfile
from pathlib import Path

import numpy as np
import manimlib
from manimlib import Animation, EndScene, Group, RIGHT, Scene, Square, linear

if "_expected_package_version" in globals():
    assert manimlib.__version__ == _expected_package_version


class Probe(Animation):
    def __init__(self, mob, phase=None, error=None, **kwargs):
        self.phase = phase
        self.error = error if error is not None else LookupError("authored execution failure")
        self.events = []
        self.after_interpolate = None
        self.abort_error = None
        super().__init__(mob, run_time=.25, rate_func=linear,
                         suspend_mobject_updating=True, **kwargs)

    def begin(self):
        self.events.append("begin")
        super().begin()
        self.mobject.locked_data_keys = {"point"}
        self.mobject.const_data_keys.add("point")
        if self.phase == "begin":
            raise self.error

    def update_mobjects(self, dt):
        if self.phase == "helpers":
            raise self.error
        super().update_mobjects(dt)

    def interpolate_mobject(self, alpha):
        if alpha > 0:
            self.events.append("interpolate")
            if self.phase == "interpolate":
                raise self.error
            if self.after_interpolate is not None:
                self.after_interpolate()

    def finish(self):
        self.events.append("finish")
        if self.phase == "finish":
            raise self.error
        super().finish()

    def clean_up_from_scene(self, scene):
        self.events.append("cleanup")
        if self.phase == "cleanup":
            raise self.error
        super().clean_up_from_scene(scene)

    def abort(self):
        self.events.append("abort")
        if self.abort_error is not None:
            raise self.abort_error
        # Deliberately do not repair storage here: the shared owner must also
        # recover custom Animation subclasses with no usable abort method.


def capture_error(operation, expected):
    try:
        operation()
    except BaseException as error:
        assert error is expected, (type(error), type(expected))
    else:
        raise AssertionError("authored failure did not propagate")


def assert_released(mob, locks):
    assert not mob._is_updating_suspended()
    assert not getattr(mob, "_is_animating", False)
    assert mob.locked_data_keys is locks
    assert mob.locked_data_keys == {"retained"}
    assert not mob.const_data_keys


def prepared_scene():
    mob = Square(fill_opacity=1).shift(RIGHT)
    mob.locked_data_keys.add("retained")
    locks = mob.locked_data_keys
    scene = Scene().add(mob)
    return scene, mob, locks


def partial_begin_recovers_and_subsequent_wait_runs_live_updaters():
    scene, mob, locks = prepared_scene()
    ticks = []
    def record(current, dt):
        if current is mob:
            ticks.append(dt)
    mob.add_updater(record, call=False)
    anim = Probe(mob, "begin")
    capture_error(lambda: scene.play(anim), anim.error)
    assert_released(mob, locks)
    assert ticks == [], "recovery ran an extra authored updater pass"
    assert "finish" not in anim.events and "cleanup" not in anim.events
    assert scene.num_plays == 0
    scene.wait(.1)
    assert any(dt > 0 for dt in ticks)
    assert scene.num_plays == 1


def later_begin_failure_recovers_prior_callback_and_leaves_future_unbegun():
    scene, mob, locks = prepared_scene()
    other, future = Square(), Square()
    scene.add(other, future)
    first, second, third = Probe(mob), Probe(other, "begin"), Probe(future)
    capture_error(lambda: scene.play(first, second, third), second.error)
    assert_released(mob, locks)
    assert not other._is_updating_suspended()
    assert not future._is_updating_suspended()
    assert third.events == []
    assert "finish" not in first.events and "cleanup" not in first.events


def helper_interpolation_and_finish_failures_leave_the_scene_usable():
    for phase in ("helpers", "interpolate", "finish"):
        scene, mob, locks = prepared_scene()
        anim = Probe(mob, phase, remover=True)
        capture_error(lambda: scene.play(anim), anim.error)
        assert_released(mob, locks)
        assert mob in scene.mobjects, "failed animation performed remover cleanup"
        assert "cleanup" not in anim.events
        assert anim.events.count("finish") == (1 if phase == "finish" else 0)
        scene.wait(.1)
        assert scene.num_plays == 1


def cleanup_failure_restores_a_previously_suspended_descendant():
    child = Square()
    root = Group(child)
    child.suspend_updating()
    scene = Scene().add(root)
    anim = Probe(root, "cleanup")
    capture_error(lambda: scene.play(anim), anim.error)
    assert not root._is_updating_suspended()
    assert child._is_updating_suspended()
    assert anim.events.count("finish") == 1
    assert "abort" not in anim.events, "completed callback was aborted twice"


def shared_descendants_use_pre_batch_suspension_state():
    child = Square()
    left, right = Group(child), Group(child)
    scene = Scene().add(left, right)
    first, second = Probe(left), Probe(right, "begin")
    capture_error(lambda: scene.play(first, second), second.error)
    for mob in (child, left, right):
        assert not mob._is_updating_suspended()
        assert not getattr(mob, "_is_animating", False)
    assert left[0] is child and right[0] is child


def abort_failure_preserves_primary_exception_and_still_recovers():
    scene, mob, locks = prepared_scene()
    anim = Probe(mob, "interpolate")
    anim.abort_error = RuntimeError("abort failed too")
    capture_error(lambda: scene.play(anim), anim.error)
    assert_released(mob, locks)
    assert any("RuntimeError" in note for note in anim.error.__notes__)


def interrupts_preserve_identity_and_release_suspension():
    for error in (KeyboardInterrupt(), SystemExit(19)):
        scene, mob, locks = prepared_scene()
        capture_error(lambda: scene.play(Probe(mob, "begin", error)), error)
        assert_released(mob, locks)
        assert "_fmn_scene_execution" not in vars(scene)


def native_play_wait_and_writer_hooks_have_one_ordered_public_lifecycle():
    events = []
    class Hooks(Scene):
        def pre_play(self):
            events.append(("pre", self.num_plays))
            super().pre_play()
        def post_play(self):
            super().post_play()
            events.append(("post", self.num_plays))
    scene = Hooks()
    scene.file_writer.begin_animation = lambda: events.append("writer-begin")
    scene.file_writer.end_animation = lambda: events.append("writer-end")
    mob = Square()
    scene.add(mob)
    scene.play(mob.animate.shift(RIGHT), run_time=.1, rate_func=linear)
    scene.wait(.1, stop_condition=lambda: True)
    assert scene.num_plays == 2
    assert events == [("pre", 0), "writer-begin", "writer-end", ("post", 1),
                      ("pre", 1), "writer-begin", "writer-end", ("post", 2)]
    np.testing.assert_allclose(mob.get_center(), RIGHT, atol=1e-6)
    before = list(events)
    scene.play()
    assert events == before


def invalid_input_is_refused_before_a_public_pre_hook():
    calls = []
    scene = Scene()
    scene.pre_play = lambda: calls.append("pre")
    for operation in (lambda: scene.play(object()), lambda: scene.wait(-1),
                      lambda: scene.wait(stop_condition=4)):
        try:
            operation()
        except (TypeError, ValueError):
            pass
        else:
            raise AssertionError("invalid input was accepted")
    assert calls == [] and scene.num_plays == 0


def callback_cannot_reenter_the_same_scene_segment():
    scene, mob, locks = prepared_scene()
    anim = Probe(mob)
    anim.after_interpolate = lambda: scene.wait(.1)
    try:
        scene.play(anim)
    except RuntimeError as error:
        assert "another play/wait" in str(error)
    else:
        raise AssertionError("reentrant native wait was admitted")
    assert_released(mob, locks)
    assert scene.num_plays == 0
    scene.wait(.1)
    assert scene.num_plays == 1


def y4m_luma(path, width=128, height=72):
    payload = path.read_bytes()
    header, rest = payload.split(b"\n", 1)
    assert header.startswith(b"YUV4MPEG2 ")
    assert f"W{width}".encode() in header.split()
    assert f"H{height}".encode() in header.split()
    assert b"F8:1" in header.split()
    assert any(token.startswith(b"C420") for token in header.split())
    size = width * height * 3 // 2
    result = []
    while rest:
        marker, rest = rest.split(b"\n", 1)
        assert marker == b"FRAME"
        assert len(rest) >= size
        result.append(np.frombuffer(rest[:width * height], dtype=np.uint8).reshape(height, width).copy())
        rest = rest[size:]
    return result


def end_at_animation_terminates_before_the_excluded_segment_and_publishes():
    class Limited(Scene):
        def construct(self):
            self.box = Square(fill_opacity=1, stroke_width=0)
            self.add(self.box)
            self.play(self.box.animate.shift(RIGHT), run_time=.25, rate_func=linear)
            self.wait(.125)
            self.wait(1)
            raise AssertionError("execution continued past the exclusive end")
    scene = Limited(end_at_animation_number=2)
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "limited.y4m"
        result = scene.render(path, format="y4m", resolution=(128, 72), fps=8, threads=1)
        assert result.frame_count == 3
        assert len(y4m_luma(path)) == 3
    assert scene.num_plays == 2
    np.testing.assert_allclose(scene.box.get_center(), RIGHT, atol=1e-6)


def hooks_drive_real_frames_and_preserve_one_four_thread_output():
    class HookMotion(Scene):
        # Luma-only detection below needs a black field: the red fill set in
        # pre_play has nearly the default #333333 background's luma.
        default_camera_config = {"background_color": "#000000"}

        def pre_play(self):
            super().pre_play()
            self.hooks.append(("pre", self.num_plays))
            if self.num_plays == 1:
                self.box.set_fill("#FF0000")
        def post_play(self):
            super().post_play()
            self.hooks.append(("post", self.num_plays))
        def construct(self):
            self.hooks = []
            self.box = Square(side_length=1, fill_opacity=1, stroke_width=0)
            self.box.move_to([-2, .8, 0])
            self.add(self.box)
            self.play(self.box.animate.shift(RIGHT), run_time=.25, rate_func=linear)
            self.wait(.125)
            self.play(self.box.animate.shift(RIGHT), run_time=.25, rate_func=linear)
    with tempfile.TemporaryDirectory() as directory:
        outputs = []
        for threads in (1, 4):
            path = Path(directory) / f"hooks-{threads}.y4m"
            scene = HookMotion()
            result = scene.render(path, format="y4m", resolution=(128, 72), fps=8, threads=threads)
            assert result.frame_count == 5
            assert scene.hooks == [("pre", 0), ("post", 1), ("pre", 1),
                                   ("post", 2), ("pre", 2), ("post", 3)]
            frames = y4m_luma(path)
            assert len(frames) == 5
            assert len({frame.tobytes() for frame in frames}) >= 3
            centers = []
            for frame in frames:
                ys, xs = np.where(frame > 30)
                assert len(xs) >= 8, "rendered animation is blank"
                assert ys.mean() < 36, "above-origin geometry rendered upside down"
                centers.append(xs.mean())
            assert centers[-1] > centers[0] + 5, "hooked scene lost actual motion"
            outputs.append(path.read_bytes())
        assert outputs[0] == outputs[1]


def failed_render_does_not_publish_a_successful_artifact():
    failure = LookupError("render interrupted by authored animation")
    class Failed(Scene):
        def construct(self):
            self.box = Square()
            self.add(self.box)
            self.play(Probe(self.box, "interpolate", failure))
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "failed.y4m"
        scene = Failed()
        capture_error(lambda: scene.render(path, format="y4m", resolution=(128, 72), fps=8, threads=1), failure)
        assert not path.exists(), "failed generation was published"
        assert not scene.box._is_updating_suspended()
        assert scene.num_plays == 0
        assert "_fmn_owned_render_session" not in vars(scene)


def planar_content_takes_the_retained_route_and_matches_the_camera_route():
    # fm-sq8.11: planar vector content under the default camera frame renders
    # on Lumen's retained 2D route; anything the camera must project keeps the
    # camera route. Both draw the same picture within an AA budget.
    from manimlib import ParametricSurface, VMobject

    class Planar(Scene):
        def construct(self):
            box = Square(side_length=1.5, fill_opacity=1, stroke_width=4).move_to([-1, .5, 0])
            self.add(box)
            self.play(box.animate.shift(RIGHT), run_time=.25, rate_func=linear)

    class Lifted(Planar):
        # The same picture plus an invisible point off the plane, which makes
        # the stage non-planar and so forces the camera route.
        def construct(self):
            marker = VMobject().set_points_as_corners([[5., 3., 1.], [5.01, 3., 1.]])
            self.add(marker.set_stroke(opacity=0).set_fill(opacity=0))
            super().construct()

    class Fixed(Scene):
        # Fixed-in-frame content is drawn through the camera route.
        def construct(self):
            self.add(Square(side_length=1.5, fill_opacity=1).fix_in_frame())
            self.wait(.25)

    class Moving(Planar):
        def construct(self):
            super().construct()
            self.play(self.frame.animate.shift(.5 * RIGHT), run_time=.25, rate_func=linear)

    class Curved(Scene):
        def construct(self):
            self.add(ParametricSurface(lambda u, v: np.array([u, v, u * v]),
                                       u_range=(-1, 1), v_range=(-1, 1)))
            self.wait(.25)

    def render(scene_class, name, threads=1):
        scene = scene_class()
        path = Path(directory) / name
        result = scene.render(path, format="y4m", resolution=(128, 72), fps=8, threads=threads)
        return scene._render_routes, y4m_luma(path), result.frame_count, path.read_bytes()

    with tempfile.TemporaryDirectory() as directory:
        routes, planar, frames, one = render(Planar, "planar-1.y4m")
        assert routes == (frames, 0, None), routes
        for threads in (4, 16):
            _, _, _, other = render(Planar, f"planar-{threads}.y4m", threads=threads)
            assert one == other, f"the planar route differs at {threads} threads"
        routes, lifted, lifted_frames, _ = render(Lifted, "lifted.y4m")
        assert routes[:2] == (0, lifted_frames) and lifted_frames == frames, routes
        assert "nonplanar" in routes[2], routes
        difference = np.abs(np.stack(planar).astype(np.int16) - np.stack(lifted).astype(np.int16))
        print("planar vs camera route luma: mean", float(difference.mean()),
              "p99", float(np.percentile(difference, 99)), "max", int(difference.max()))
        # Measured 2026-10-04: mean 0.0017, p99 0, max 8 luma codes.
        assert (difference.mean() < 0.05 and np.percentile(difference, 99) < 1
                and difference.max() <= 16), difference.max()
        routes, _, moving_frames, _ = render(Moving, "moving.y4m")
        assert routes[0] > 0 and routes[1] > 0 and sum(routes[:2]) == moving_frames, routes
        assert routes[2] == "camera frame differs from the default frame", routes
        routes, _, fixed_frames, _ = render(Fixed, "fixed.y4m")
        assert routes[:2] == (0, fixed_frames) and "fixed-in-frame" in routes[2], routes
        routes, _, curved_frames, _ = render(Curved, "curved.y4m")
        assert routes[:2] == (0, curved_frames) and routes[2], routes

        # Certified provenance journals the route (closure item 7).
        def certified_manifest(scene_class, name):
            scene = scene_class()
            result = scene.render(Path(directory) / name, format="png_sequence",
                                  resolution=(128, 72), fps=8, threads=1, reproducible=True,
                                  sources={"scene_execution.py": b"planar route provenance"})
            return scene._render_routes, (Path(result.manifest).parent / "manifest.txt").read_text()

        routes, text = certified_manifest(Planar, "planar-certified")
        assert "cpu-certified-lumen:retained-2d, %d retained-2d and 0 camera frames" % routes[0] in text, text
        routes, text = certified_manifest(Lifted, "lifted-certified")
        assert "cpu-certified-lumen:camera, 0 retained-2d" in text and "first camera frame: nonplanar" in text, text


CASES = (
    partial_begin_recovers_and_subsequent_wait_runs_live_updaters,
    later_begin_failure_recovers_prior_callback_and_leaves_future_unbegun,
    helper_interpolation_and_finish_failures_leave_the_scene_usable,
    cleanup_failure_restores_a_previously_suspended_descendant,
    shared_descendants_use_pre_batch_suspension_state,
    abort_failure_preserves_primary_exception_and_still_recovers,
    interrupts_preserve_identity_and_release_suspension,
    native_play_wait_and_writer_hooks_have_one_ordered_public_lifecycle,
    invalid_input_is_refused_before_a_public_pre_hook,
    callback_cannot_reenter_the_same_scene_segment,
    end_at_animation_terminates_before_the_excluded_segment_and_publishes,
    hooks_drive_real_frames_and_preserve_one_four_thread_output,
    failed_render_does_not_publish_a_successful_artifact,
    planar_content_takes_the_retained_route_and_matches_the_camera_route,
)
for case in CASES:
    case()
    print("native scene execution passed:", case.__name__)
