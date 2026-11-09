"""Installed-extension indication acceptance with real records and Reel output."""
import math
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m
from manimlib.animation.growing import GrowFromPoint as QualifiedGrow
from manimlib.animation.indication import ApplyWave as QualifiedWave
from manimlib.animation.indication import VShowPassingFlash as QualifiedFlash
from fmn_python import render_scene


def close(actual, expected):
    np.testing.assert_allclose(actual, expected, atol=3e-5, rtol=3e-5)


def wave_and_aliases():
    assert QualifiedWave is m.ApplyWave and issubclass(m.ApplyWave, m.Homotopy)
    assert QualifiedFlash is m.VShowPassingFlash and QualifiedGrow is m.GrowFromPoint
    line = m.Line(m.LEFT, m.RIGHT)
    before = line.get_points().copy()
    animation = m.ApplyWave(line, amplitude=.7, rate_func=m.linear, run_time=2.5)
    assert animation.run_time == 2.5
    animation.begin()
    animation.interpolate(.5)
    expected = before.copy()
    # For bounds [-1, 1], 2 * (x-proportion - .5) is exactly x.
    for index, point in enumerate(before):
        phase = .5 ** math.exp(float(point[0]))
        x = 2 * phase if phase < .5 else 2 * (1 - phase)
        expected[index, 1] += .7 * x ** 3 * (10 - 15 * x + 6 * x * x)
    close(line.get_points(), expected)
    animation.finish()
    close(line.get_points(), before)


def wave_construction_envelope_and_hooks():
    line, calls = m.Line(m.LEFT, m.RIGHT), []
    class Authored(m.ApplyWave):
        def function_at_time_t(self, t):
            calls.append(t)
            return super().function_at_time_t(t)
    animation = Authored(line, amplitude=.4, rate_func=m.linear)
    line.scale(2)
    animation.begin()
    animation.interpolate(.5)
    assert len(calls) >= 2
    close(animation.function_at_time_t(.5)(np.zeros(3)), .4 * m.UP)
    animation.finish()
    close(line.get_start(), 2 * m.LEFT)
    close(line.get_end(), 2 * m.RIGHT)


def wiggle_direct_and_pivots():
    line = m.Line(m.LEFT, m.RIGHT)
    before = line.get_points().copy()
    animation = m.WiggleOutThenIn(line, scale_value=2, rotation_angle=math.pi / 2,
                                 n_wiggles=1, rate_func=m.linear)
    animation.begin()
    animation.interpolate(.5)
    close(line.get_start(), 2 * m.DOWN)
    close(line.get_end(), 2 * m.UP)
    midpoint = line.get_points().copy()
    animation.interpolate(.5)
    close(line.get_points(), midpoint)
    animation.finish()
    close(line.get_points(), before)
    calls = []
    class Authored(m.WiggleOutThenIn):
        def get_scale_about_point(self):
            calls.append("scale")
            return m.RIGHT
        def get_rotate_about_point(self):
            calls.append("rotate")
            return m.ORIGIN
    scene = m.Scene(camera_config=dict(fps=8))
    scene.add(line)
    scene.play(Authored(line, run_time=.25, rate_func=m.linear))
    assert "scale" in calls and "rotate" in calls


def gaussian_widths_and_style_restore():
    line = m.Line(3 * m.LEFT, 3 * m.RIGHT)
    line.insert_n_curves(8)
    widths = np.linspace(2, 6, len(line.get_stroke_widths()))
    line.set_stroke(width=widths, recurse=False)
    original_color = line.get_stroke_color()
    animation = m.VShowPassingFlash(line, time_width=.6, taper_width=0,
                                    remover=False, rate_func=m.linear)
    animation.begin()
    animation.interpolate(.5)
    xs = np.linspace(0, 1, len(widths))
    expected = np.array([w * math.exp(-.5 * ((x - .5) / .1) ** 2)
                         if abs(x - .5) <= .3 else 0 for w, x in zip(widths, xs)])
    close(line.get_stroke_widths(), expected)
    line.set_stroke(color=m.RED)
    animation.finish()
    close(line.get_stroke_widths(), widths)
    assert line.get_stroke_color() == original_color


def width_family_and_authored_taper():
    calls = []
    class Authored(m.VShowPassingFlash):
        def taper_kernel(self, x):
            calls.append(x)
            return 2.0
    left, right = m.Line(m.LEFT, m.RIGHT), m.Line(m.LEFT, m.RIGHT).shift(m.UP)
    left.set_stroke(width=3)
    right.set_stroke(width=7)
    group = m.VGroup(left, right)
    scene = m.Scene(camera_config=dict(fps=8))
    scene.add(group)
    animation = Authored(group, run_time=.25, remover=False, rate_func=m.linear)
    scene.play(animation)
    assert calls and any(obj is group for obj in scene.mobjects)
    close(left.get_stroke_widths(), 3)
    close(right.get_stroke_widths(), 7)


def live_rate_without_speculative_sampling():
    line = m.Line(m.LEFT, m.RIGHT)
    scene = m.Scene(camera_config=dict(fps=8))
    scene.add(line)
    animation = m.WiggleOutThenIn(line, run_time=.25)
    class Rate:
        __hash__ = None
        calls = 0
        def __call__(self, t):
            assert hasattr(animation, "starting_mobject"), "authored rate was sampled before begin"
            self.calls += 1
            return t * t
    curve = Rate()
    scene.play(animation, rate_func=curve)
    assert curve.calls >= 3


def grow_after_sequential_mutation():
    scene, square = m.Scene(camera_config=dict(fps=8)), m.Square()
    scene.add(square)
    grow = m.GrowFromCenter(square, run_time=.25, rate_func=m.linear)
    scene.play(m.Succession(
        square.animate(run_time=.25, rate_func=m.linear).shift(2 * m.RIGHT).scale(2),
        grow,
    ))
    close(square.get_center(), 2 * m.RIGHT)
    close(square.get_width(), 4)
    close(grow.point, m.ORIGIN)


def grow_partial_endpoint_and_removal():
    scene, square = m.Scene(camera_config=dict(fps=8)), m.Square()
    scene.add(square)
    scene.play(m.GrowFromPoint(square, 2 * m.LEFT, run_time=.25,
                              rate_func=m.linear, final_alpha_value=.5))
    close(square.get_center(), m.LEFT)
    close(square.get_width(), 1)
    scene.play(m.GrowFromCenter(square, run_time=.25, remover=True, rate_func=m.linear))
    assert all(obj is not square for obj in scene.mobjects)


def inside_out_after_sequential_mutation():
    scene = m.Scene(camera_config=dict(fps=8))
    line = m.VMobject().set_points_as_corners([m.LEFT, m.UP, m.RIGHT])
    before = line.get_points().copy()
    scene.add(line)
    scene.play(m.Succession(
        line.animate(run_time=.25, rate_func=m.linear).shift(m.RIGHT),
        m.TurnInsideOut(line, run_time=.25, rate_func=m.linear, path_arc=0),
    ))
    close(line.get_points(), (before + m.RIGHT)[::-1])


def persistent_effect_updaters():
    line = m.Line(m.LEFT, m.RIGHT)
    before = line.get_points().copy()
    animation = m.WiggleOutThenIn(line, run_time=1, scale_value=2,
                                 rotation_angle=0, rate_func=m.linear)
    m.turn_animation_into_updater(animation)
    line.update(.5)
    line.update(0)
    close(line.get_width(), 4)
    line.update(.75)
    line.update(0)
    close(line.get_points(), before)
    assert not line.updaters
    wave = m.ApplyWave(line, run_time=1, rate_func=m.linear)
    m.cycle_animation(wave)
    line.update(2.5)
    line.update(0)
    assert np.max(line.get_points()[:, 1]) > .1
    line.clear_updaters()
    wave.abort()


def abort_keeps_style_and_suspension_ownership():
    class Bad(m.VShowPassingFlash):
        def interpolate_submobject(self, current, start, alpha):
            super().interpolate_submobject(current, start, alpha)
            if alpha > 0:
                raise RuntimeError("indication-mid-frame-failure")
    line, scene = m.Line(m.LEFT, m.RIGHT), m.Scene(camera_config=dict(fps=8))
    widths = line.get_stroke_widths().copy()
    scene.add(line)
    try:
        scene.play(Bad(line, run_time=.25, rate_func=m.linear, suspend_mobject_updating=True))
    except RuntimeError as error:
        assert "indication-mid-frame-failure" in str(error)
    else:
        raise AssertionError("the authored failure must propagate")
    close(line.get_stroke_widths(), widths)
    assert not line._is_updating_suspended()
    child = m.Line(m.LEFT, m.RIGHT)
    child.suspend_updating()
    group = m.VGroup(child)
    animation = m.WiggleOutThenIn(group, suspend_mobject_updating=True, rate_func=m.linear)
    animation.begin()
    animation.finish()
    assert child._is_updating_suspended() and not group._is_updating_suspended()


def rendered_effects_are_not_noops_and_repeat_across_threads():
    root = Path(tempfile.mkdtemp(prefix="fmn-indication-output-"))
    for name in ("wave", "wiggle", "flash"):
        class EffectScene(m.Scene):
            default_camera_config = dict(resolution=(96, 54), fps=8)
            def construct(self):
                square = m.Square(side_length=2, fill_opacity=0, stroke_color=m.WHITE, stroke_width=8)
                square.insert_n_curves(12)
                self.add(square)
                self.wait(.125)
                if name == "wave":
                    animation = m.ApplyWave(square, amplitude=.8, run_time=.5, rate_func=m.linear)
                elif name == "wiggle":
                    animation = m.WiggleOutThenIn(square, scale_value=1.5, rotation_angle=.2,
                                                  run_time=.5, rate_func=lambda t: t)
                else:
                    animation = m.VShowPassingFlash(square, time_width=.6, taper_width=0,
                                                    remover=False, run_time=.5, rate_func=m.linear)
                self.play(animation)
        one = render_scene(EffectScene, root / (name + "-one.y4m"), threads=1)
        four = render_scene(EffectScene, root / (name + "-four.y4m"), threads=4)
        assert one.frame_count == four.frame_count == 5
        data = one.destination.read_bytes()
        assert data == four.destination.read_bytes()
        header, payload = data.split(b"\n", 1)
        assert header == b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2"
        frame_size = 96 * 54 * 3 // 2
        assert len(payload) == 5 * (frame_size + 6)
        frames = []
        for offset in range(0, len(payload), frame_size + 6):
            assert payload[offset:offset + 6] == b"FRAME\n"
            frames.append(payload[offset + 6:offset + 6 + frame_size])
        assert len(set(frames)) >= 3, (name, "effect rendered as static frames")
    print("retained indication output:", root)


for case in (
    wave_and_aliases, wave_construction_envelope_and_hooks, wiggle_direct_and_pivots,
    gaussian_widths_and_style_restore, width_family_and_authored_taper,
    live_rate_without_speculative_sampling, grow_after_sequential_mutation,
    grow_partial_endpoint_and_removal, inside_out_after_sequential_mutation,
    persistent_effect_updaters, abort_keeps_style_and_suspension_ownership,
    rendered_effects_are_not_noops_and_repeat_across_threads,
):
    case()
    print("indication acceptance:", case.__name__)
