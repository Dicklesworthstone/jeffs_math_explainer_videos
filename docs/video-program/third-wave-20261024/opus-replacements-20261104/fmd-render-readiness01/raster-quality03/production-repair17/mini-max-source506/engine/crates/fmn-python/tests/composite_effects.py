"""Installed-extension composite effects: live geometry, hooks and real output."""
import hashlib
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m
from fmn_python import render_scene
from manimlib.animation.indication import Flash as QualifiedFlash
from manimlib.animation.composition import LaggedStartMap as QualifiedMap


def close(actual, expected):
    np.testing.assert_allclose(actual, expected, rtol=0, atol=3e-5)


def members(scene):
    return {id(member) for root in scene.mobjects for member in root.get_family()}


def numeric_flash_direct_lifecycle():
    scene = m.Scene()
    flash = m.Flash(m.ORIGIN, num_lines=4, line_length=2, flash_radius=3)
    assert flash.group is flash.mobject is flash.lines
    assert QualifiedFlash is m.Flash
    scene.add(flash.lines)
    for child in flash.animations:
        child.rate_func = m.linear
    flash.begin()
    flash.update_mobjects(.1)
    flash.interpolate(.1)
    for line in flash.lines:
        close(line.get_arc_length(), .6)
    flash.interpolate(.5)
    for line in flash.lines:
        close(line.get_arc_length(), 2)
    line_ids = {id(line) for line in flash.lines}
    flash.finish()
    flash.clean_up_from_scene(scene)
    assert not line_ids.intersection(members(scene))


def following_flash_executes_authored_children():
    events, samples = [], []
    class Child(m.Animation):
        def update_mobjects(self, dt):
            events.append((id(self), "helpers", dt))
            super().update_mobjects(dt)
        def interpolate_submobject(self, current, start, alpha):
            events.append((id(self), "alpha", alpha))
            current.set_stroke(width=2 + alpha)
    class Flash(m.Flash):
        def create_line_anims(self):
            return [Child(line, rate_func=m.linear) for line in self.lines]
    scene = m.Scene(camera_config=dict(fps=8))
    target = m.Dot(m.LEFT)
    scene.add(target)
    target.add_updater(lambda obj, dt: obj.shift(dt * m.RIGHT), call=False)
    flash = Flash(target, num_lines=4, run_time=.5)
    flash.lines.add_updater(
        lambda group: samples.append((group.get_center().copy(), target.get_center().copy())),
        call=False,
    )
    scene.play(flash)
    assert samples and any(point[0] > -.9 for point, _ in samples)
    for point, expected in samples:
        close(point, expected)
    for child in flash.animations:
        calls = [(kind, value) for identity, kind, value in events if identity == id(child)]
        assert any(kind == "helpers" and value > 0 for kind, value in calls)
        assert any(kind == "alpha" and 0 < value < 1 for kind, value in calls)


def flash_tracks_mutated_point_array():
    point = np.array([0., 0., 0.])
    flash = m.Flash(point, num_lines=4)
    assert flash.point is point
    point[:] = (2, -1, 0)
    flash.lines.update(0)
    close(flash.lines.get_center(), point)


def mapped_group_keeps_identity_and_updaters():
    scene = m.Scene(camera_config=dict(fps=8))
    root = m.VGroup(m.Square().shift(2 * m.LEFT), m.Square().shift(2 * m.RIGHT))
    children = tuple(root)
    observed = []
    root.add_updater(lambda group: observed.append((id(group), group.get_center().copy())), call=False)
    effect = m.LaggedStartMap(
        lambda child: m.ApplyMethod(child.shift, m.RIGHT, rate_func=m.linear),
        root, run_time=.5, lag_ratio=.5,
    )
    assert QualifiedMap is m.LaggedStartMap
    assert effect.mobject is effect.group is root
    scene.play(effect)
    assert observed and all(identity == id(root) for identity, _ in observed)
    close(children[0].get_center(), m.LEFT)
    close(children[1].get_center(), 3 * m.RIGHT)
    assert id(root) in members(scene)


def clock_keeps_face_and_rotates_native_hands():
    scene = m.Scene(camera_config=dict(fps=8))
    clock = m.Clock()
    center = clock.get_center().copy()
    start = clock.hour_hand.get_end().copy() - center
    face_points = clock.get_points().copy()
    effect = m.ClockPassesTime(clock, run_time=.25, hours_passed=3)
    assert effect.mobject is effect.group is clock
    scene.play(effect)
    assert id(clock) in members(scene)
    np.testing.assert_array_equal(clock.get_points(), face_points)
    close(clock.hour_hand.get_end(), center + np.array([start[1], -start[0], start[2]]))


def broadcast_uses_ring_group_and_staggered_restore():
    scene = m.Scene()
    effect = m.Broadcast(m.ORIGIN, small_radius=.1, big_radius=2, n_circles=2,
                         run_time=1.5, lag_ratio=.5, remover=False)
    assert effect.mobject is effect.group is effect.circles
    rings = tuple(effect.circles)
    scene.add(effect.circles)
    for child in effect.animations:
        child.rate_func = m.linear
    effect.begin()
    effect.interpolate(1 / 3)
    close(rings[0].get_width(), 2.1)
    close(rings[1].get_width(), .2)
    effect.finish()
    effect.clean_up_from_scene(scene)
    for ring in rings:
        close(ring.get_width(), 4)
        assert id(ring) in members(scene)


def flashy_fade_runs_real_fade_and_outline_cleanup():
    scene = m.Scene()
    square = m.Square(fill_opacity=.8)
    effect = m.FlashyFadeIn(square, fade_lag=.5, rate_func=m.linear)
    assert effect.group is effect.mobject
    assert effect.animations[0].mobject is square
    assert effect.animations[1].mobject is effect.outline
    scene.add(effect.group)
    effect.begin()
    effect.interpolate(.25)
    close(square.get_fill_opacity(), 0)
    effect.interpolate(.75)
    close(square.get_fill_opacity(), .4)
    effect.finish()
    effect.clean_up_from_scene(scene)
    close(square.get_fill_opacity(), .8)
    assert id(square) in members(scene)
    assert id(effect.outline) not in members(scene)


def live_group_rate_is_not_probed_before_begin():
    calls = []
    class Rate:
        __hash__ = None
        started = False
        def __call__(self, alpha):
            assert self.started, "group rate evaluated speculatively before begin"
            calls.append(alpha)
            return alpha * alpha
    rate = Rate()
    class Flash(m.Flash):
        def begin(self):
            rate.started = True
            super().begin()
    scene = m.Scene(camera_config=dict(fps=8))
    scene.play(Flash(m.ORIGIN, num_lines=4, run_time=.5, rate_func=rate))
    assert any(0 < alpha < 1 for alpha in calls)


def persistent_numeric_flash_uses_same_children():
    scene = m.Scene(camera_config=dict(fps=8))
    effect = m.Flash(m.ORIGIN, num_lines=4, run_time=.25)
    lines = tuple(effect.lines)
    scene.add(effect.lines)
    result = m.turn_animation_into_updater(effect)
    assert result is effect.lines
    scene.wait(.75)
    assert all(not hasattr(updater, "_fmn_persistent_controller") for updater in effect.lines.updaters)
    for line in lines:
        close(line.get_arc_length(), .2)
    # Persistent animations intentionally do not call scene-removal cleanup.
    assert all(id(line) in members(scene) for line in lines)


def child_exception_cancels_composite_and_publication():
    root = Path(tempfile.mkdtemp(prefix="fmn-composite-failure-"))
    destination = root / "failed.y4m"
    error = RuntimeError("authored Flash child failed")
    class Broken(m.Animation):
        def interpolate_submobject(self, current, start, alpha):
            if alpha > 0:
                raise error
    class Flash(m.Flash):
        def create_line_anims(self):
            return [Broken(line, suspend_mobject_updating=True) for line in self.lines]
    class Failing(m.Scene):
        def construct(self):
            self.effect = Flash(m.ORIGIN, num_lines=4, run_time=.25)
            self.play(self.effect)
    scene = Failing()
    try:
        scene.render(destination, resolution=(96, 54), fps=8, threads=1)
    except RuntimeError as caught:
        assert caught is error
    else:
        raise AssertionError("authored child error must propagate")
    assert not destination.exists()
    assert scene.effect._composition_driver is None
    assert not any(member._is_updating_suspended() for member in scene.effect.lines.get_family())


def y4m_luma(path):
    header, payload = path.read_bytes().split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2"
    frame_size = 96 * 54 * 3 // 2
    assert len(payload) % (frame_size + 6) == 0
    frames = []
    for start in range(0, len(payload), frame_size + 6):
        assert payload[start:start + 6] == b"FRAME\n"
        frames.append(np.frombuffer(payload[start + 6:start + 6 + 96 * 54], np.uint8).copy())
    return frames


def actual_composite_frames_and_thread_repeatability():
    class Effects(m.Scene):
        def construct(self):
            self.wait(.125)
            self.play(m.Flash(m.ORIGIN, num_lines=8, flash_radius=1.5,
                              line_length=.6, line_stroke_width=30, run_time=.5))
    root = Path(tempfile.mkdtemp(prefix="fmn-composite-output-"))
    results = [render_scene(Effects, root / f"threads-{threads}.y4m",
                            resolution=(96, 54), fps=8, threads=threads) for threads in (1, 4)]
    for result in results:
        assert result.frame_count == 5
        payload = result.destination.read_bytes()
        assert hashlib.sha256(payload).hexdigest() == result.digest
        assert len(payload) == result.bytes
        frames = y4m_luma(result.destination)
        assert len(frames) == 5
        assert max(frame.max() for frame in frames[1:]) > 128
        assert len({frame.tobytes() for frame in frames}) > 2
    assert results[0].destination.read_bytes() == results[1].destination.read_bytes()


for case in (
    numeric_flash_direct_lifecycle, following_flash_executes_authored_children,
    flash_tracks_mutated_point_array, mapped_group_keeps_identity_and_updaters,
    clock_keeps_face_and_rotates_native_hands, broadcast_uses_ring_group_and_staggered_restore,
    flashy_fade_runs_real_fade_and_outline_cleanup, live_group_rate_is_not_probed_before_begin,
    persistent_numeric_flash_uses_same_children, child_exception_cancels_composite_and_publication,
    actual_composite_frames_and_thread_repeatability,
):
    case()
    print("composite effects acceptance:", case.__name__)
