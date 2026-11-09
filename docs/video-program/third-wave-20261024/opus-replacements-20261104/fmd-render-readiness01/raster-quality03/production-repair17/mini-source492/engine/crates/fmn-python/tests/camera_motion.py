"""Spatial CameraFrame animations on the real native clock and renderer.

Run against an installed portal with ``python -I camera_motion.py``. These
checks use native camera/geometry values and scene playback, never mocks.
"""
import math
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m


def close(actual, expected):
    np.testing.assert_allclose(actual, expected, atol=2e-9, rtol=2e-9)


def camera_samples(scene):
    samples = []

    def observe(frame, dt):
        if frame is scene.frame and dt > 0:
            samples.append((frame.get_center().copy(), frame.get_orientation().copy()))

    scene.frame.add_updater(observe, call=False)
    return samples


def rotation_is_absolute_at_every_native_sample():
    for cls in (m.Rotate, m.Rotating):
        for frames in (2, 4, 7):
            for angle in (-1.3, math.tau * 2.5):
                scene = m.Scene()
                frame = scene.frame
                frame.shift((2, -1, 0.5)).scale(0.75).rotate(0.6, axis=m.RIGHT)
                frame.set_field_of_view(0.9)
                start, core = frame.copy(), frame._core
                samples = camera_samples(scene)
                scene.play(cls(frame, angle=angle, axis=(0, 2, 1),
                               run_time=frames / 30, rate_func=m.linear))
                for index, (center, quaternion) in enumerate(samples, 1):
                    # BN-02 preserves the nominal overshoot sample when the
                    # supplied binary-float duration lies above a frame tick.
                    alpha = index / frames
                    expected = start.copy().rotate(angle * alpha, axis=(0, 2, 1))
                    close(quaternion, expected.get_orientation())
                    close(center, start.get_center())
                close(frame.get_orientation(), start.copy().rotate(angle, axis=(0, 2, 1)).get_orientation())
                close(frame.get_shape(), start.get_shape())
                assert frame.get_field_of_view() == 0.9
                assert frame._core is core and scene.mobjects == [] and not frame._is_bound()


def rotation_obeys_live_rates_windows_and_final_alpha():
    class Rate:
        __hash__ = None

        def __call__(self, alpha):
            return alpha * alpha

    scene = m.Scene()
    start = scene.frame.copy()
    samples = camera_samples(scene)
    scene.play(m.Rotate(scene.frame, angle=1.2, run_time=4 / 30,
                        rate_func=Rate(), final_alpha_value=0.5))
    for index, (_, quaternion) in enumerate(samples, 1):
        close(quaternion, start.copy().rotate(1.2 * (index / 4) ** 2).get_orientation())
    close(scene.frame.get_orientation(), start.copy().rotate(0.3).get_orientation())
    scene = m.Scene()
    start = scene.frame.copy()
    samples = camera_samples(scene)
    scene.play(m.Rotate(scene.frame, angle=1.2, run_time=4 / 30,
                        time_span=(2 / 30, 4 / 30), rate_func="linear"))
    for (_, quaternion), alpha in zip(samples, (0, 0, .5, 1)):
        close(quaternion, start.copy().rotate(1.2 * alpha).get_orientation())


def succession_captures_predecessor_camera_orientation():
    scene = m.Scene()
    frame = scene.frame
    first = m.Rotate(frame, angle=0.7, axis=m.RIGHT, run_time=2 / 30, rate_func=m.linear)
    second = m.Rotate(frame, angle=-1.3, axis=m.UP, run_time=2 / 30, rate_func=m.linear)
    initial = frame.copy()
    scene.play(m.AnimationGroup(m.Succession(first, second)))
    expected = initial.copy().rotate(0.7, axis=m.RIGHT)
    close(second.starting_mobject.get_orientation(), expected.get_orientation())
    close(frame.get_orientation(), expected.rotate(-1.3, axis=m.UP).get_orientation())
    assert not frame._is_bound() and scene.mobjects == []


def path_uses_true_length_and_preserves_optical_pose():
    scene = m.Scene()
    path = m.VMobject().set_points_as_corners([(0, 0, 0), (1, 0, 0), (1, 3, 0)])
    scene.frame.rotate(0.7, axis=m.RIGHT).scale(0.5).set_field_of_view(0.8)
    orientation, shape = scene.frame.get_orientation(), scene.frame.get_shape()
    samples = camera_samples(scene)
    scene.play(m.MoveAlongPath(scene.frame, path, run_time=4 / 30,
                               rate_func=m.linear, suspend_mobject_updating=False))
    close([sample[0] for sample in samples], [(1, 0, 0), (1, 1, 0), (1, 2, 0), (1, 3, 0)])
    for _, quaternion in samples:
        close(quaternion, orientation)
    close(scene.frame.get_shape(), shape)
    assert scene.frame.get_field_of_view() == 0.8
    assert not path._is_bound() and scene.mobjects == []


def simultaneous_path_and_rotation_keep_native_play_order():
    scene = m.Scene()
    start = scene.frame.copy()
    path = m.Line(m.ORIGIN, 4 * m.RIGHT)
    square = m.Square()
    scene.add(square)
    samples = camera_samples(scene)
    scene.play(m.Rotate(scene.frame, angle=1.2, rate_func=m.linear),
               m.MoveAlongPath(scene.frame, path, rate_func=m.linear,
                               suspend_mobject_updating=False),
               square.animate(rate_func=m.linear).shift(2 * m.UP), run_time=4 / 30)
    for index, (center, quaternion) in enumerate(samples, 1):
        close(center, (index, 0, 0))
        close(quaternion, start.copy().rotate(1.2 * index / 4).get_orientation())
    close(square.get_center(), 2 * m.UP)
    assert scene.mobjects == [square] and not scene.frame._is_bound()


def tracking_uses_construction_offset_and_current_sibling_state():
    scene = m.Scene()
    dot = m.Dot().shift(m.RIGHT)
    scene.add(dot)
    scene.frame.shift(3 * m.RIGHT)
    follower = m.MaintainPositionRelativeTo(scene.frame, dot,
                                           suspend_mobject_updating=False)
    dot.shift(m.UP)
    scene.frame.shift(7 * m.LEFT)
    samples = camera_samples(scene)
    scene.play(dot.animate(rate_func=m.linear).shift(4 * m.RIGHT), follower, run_time=4 / 30)
    close([sample[0] for sample in samples], [(4, 1, 0), (5, 1, 0), (6, 1, 0), (7, 1, 0)])
    assert scene.mobjects == [dot] and not scene.frame._is_bound()


def tracking_samples_authored_centers_exactly_once_at_construction():
    scene = m.Scene()
    dot = m.Dot()
    calls = []
    frame_center, dot_center = scene.frame.get_center, dot.get_center

    def camera_center():
        calls.append("camera")
        return frame_center()

    def target_center():
        calls.append("target")
        return dot_center()

    scene.frame.get_center, dot.get_center = camera_center, target_center
    animation = m.MaintainPositionRelativeTo(scene.frame, dot)
    assert calls == ["camera", "target"], calls
    close(animation.diff, m.ORIGIN)


def foreign_helpers_and_destructive_camera_operations_are_refused():
    scene, other = m.Scene(), m.Scene()
    path, dot = m.Line(m.ORIGIN, m.RIGHT), m.Dot()
    other.add(path, dot)
    for animation in (m.MoveAlongPath(scene.frame, path),
                      m.MaintainPositionRelativeTo(scene.frame, dot),
                      m.Rotate(scene.frame, remover=True),
                      m.Rotate(scene.frame.copy())):
        before = scene.frame.copy()
        try:
            scene.play(m.AnimationGroup(animation), run_time=2 / 30)
        except (ValueError, NotImplementedError, m._ForeignStageError):
            pass
        else:
            raise AssertionError("invalid camera motion was accepted")
        close(scene.frame.get_center(), before.get_center())
        close(scene.frame.get_orientation(), before.get_orientation())
        assert scene.time == 0 and scene.mobjects == []
        assert not scene.frame._is_updating_suspended() and not scene.frame._is_bound()
    assert other.mobjects == [path, dot]


def authored_rotation_dispatch_preserves_live_receiver():
    scene = m.Scene()
    frame = scene.frame
    calls, rotate = [], frame.rotate

    def authored_rotate(angle, axis=m.OUT, **kwargs):
        calls.append((angle, axis))
        return rotate(angle, axis, **kwargs)

    frame.rotate = authored_rotate
    scene.play(m.Rotate(frame, angle=0.8, run_time=2 / 30, rate_func=m.linear))
    close([entry[0] for entry in calls], [0., 0.4, 0.8, 0.8])
    close(frame.get_orientation(), m.CameraFrame().rotate(0.8).get_orientation())


def authored_rates_observe_restored_pose_and_may_edit_it():
    scene = m.Scene()
    frame = scene.frame.shift((1, 2, 0)).rotate(0.3)
    start, seen = frame.copy(), []

    def rate(alpha):
        seen.append((frame.get_center().copy(), frame.get_orientation().copy()))
        # The rate's authored edits survive until the rotation hook, exactly
        # as they do after a drawable's match_points restoration.
        frame.shift(alpha * m.UP)
        return alpha

    scene.play(m.Rotate(frame, angle=0.8, run_time=4 / 30, rate_func=rate))
    assert len(seen) == 6
    for center, orientation in seen:
        close(center, start.get_center())
        close(orientation, start.get_orientation())
    close(frame.get_center(), start.get_center() + m.UP)
    close(frame.get_orientation(), start.copy().rotate(0.8).get_orientation())


def authored_rotation_owns_axis_interpretation():
    scene = m.Scene()
    frame, calls = scene.frame, []
    native_rotate = frame.rotate

    def authored_rotate(angle, axis=m.OUT, **kwargs):
        calls.append(tuple(axis))
        return native_rotate(angle, axis=m.OUT if not any(axis) else axis, **kwargs)

    frame.rotate = authored_rotate
    scene.play(m.Rotate(frame, angle=0.8, axis=m.ORIGIN,
                        run_time=2 / 30, rate_func=m.linear))
    assert calls == [(0., 0., 0.)] * 4
    close(frame.get_orientation(), m.CameraFrame().rotate(0.8).get_orientation())
    # Without an override, native validation still refuses a zero axis.
    ordinary = m.Scene()
    try:
        ordinary.play(m.Rotate(ordinary.frame, axis=m.ORIGIN), run_time=1 / 30)
    except ValueError:
        pass
    else:
        raise AssertionError("stock native rotation accepted a zero axis")


def final_camera_motion_changes_real_native_pixels():
    class Image(m.Scene):
        reference = False

        def construct(self):
            self.add(m.Rectangle(width=1, height=2, fill_color=m.RED,
                                 fill_opacity=1, stroke_width=0).shift(m.RIGHT))
            if self.reference:
                self.frame.rotate(0.8, axis=m.OUT).move_to((0.3, 0.5, 0))
            else:
                self.play(m.Rotate(self.frame, angle=0.8, rate_func=m.linear),
                          m.MoveAlongPath(self.frame, m.Line(m.ORIGIN, (0.3, 0.5, 0)),
                                          rate_func=m.linear), run_time=2 / 30)

    images = []
    with tempfile.TemporaryDirectory(prefix="fmn-camera-motion-") as directory:
        for name, reference, threads in (("motion-1", False, 1), ("motion-4", False, 4),
                                         ("reference", True, 1)):
            scene = Image()
            scene.reference = reference
            output = Path(directory) / (name + ".png")
            scene._begin_png(str(output), 96, 64, 30, threads, 0)
            try:
                scene.run()
                scene._finish_render(scene.frame._core, scene.camera.light_source.get_center())
            except BaseException:
                scene._abort_render()
                raise
            images.append(output.read_bytes())
    assert images[0] == images[1] == images[2], "camera motion disagrees with a directly authored native pose"
    assert images[0][:8] == b"\x89PNG\r\n\x1a\n"


_CASES = (rotation_is_absolute_at_every_native_sample,
          rotation_obeys_live_rates_windows_and_final_alpha,
          succession_captures_predecessor_camera_orientation,
          path_uses_true_length_and_preserves_optical_pose,
          simultaneous_path_and_rotation_keep_native_play_order,
          tracking_uses_construction_offset_and_current_sibling_state,
          tracking_samples_authored_centers_exactly_once_at_construction,
          foreign_helpers_and_destructive_camera_operations_are_refused,
          authored_rotation_dispatch_preserves_live_receiver,
          authored_rates_observe_restored_pose_and_may_edit_it,
          authored_rotation_owns_axis_interpretation,
          final_camera_motion_changes_real_native_pixels)
for _case in _CASES:
    _case()
print(f"camera motion acceptance: {len(_CASES)} cases passed")
