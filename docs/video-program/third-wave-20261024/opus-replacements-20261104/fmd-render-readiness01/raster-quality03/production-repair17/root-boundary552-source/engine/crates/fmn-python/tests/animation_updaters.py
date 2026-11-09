"""Installed-extension acceptance for persistent animation updaters."""
import numpy as np
from manimlib import (
    Animation, Line, Scene, ShowCreation, Square, RIGHT, ORIGIN,
    cycle_animation, linear, turn_animation_into_updater,
)


def drawing_protocol():
    line = Line(ORIGIN, 2 * RIGHT)
    turn_animation_into_updater(ShowCreation(line, run_time=1, rate_func=linear))
    line.update(.5)
    line.update(0)
    np.testing.assert_allclose(line.get_end(), RIGHT, atol=1e-5)


def helper_order_and_time():
    events = []
    class Fade(Animation):
        def interpolate_submobject(self, current, start, alpha):
            current.set_opacity(alpha)
            events.append("interpolate")
        def update_mobjects(self, dt):
            events.append(("helpers", dt))
            super().update_mobjects(dt)
    square = Square(fill_opacity=1)
    animation = Fade(square, rate_func=linear, time_span=(1., 3.))
    turn_animation_into_updater(animation)
    events.clear()
    square.update(2.)
    assert events[-1] == ("helpers", 2.)
    assert "interpolate" in events[:-1]
    square.update(0.)
    np.testing.assert_allclose(square.get_fill_opacity(), .5, atol=1e-6)
    assert square.updaters


def scene_wait_completion():
    class Fade(Animation):
        def interpolate_submobject(self, current, start, alpha):
            current.set_opacity(alpha)
    scene = Scene()
    square = Square(fill_opacity=1)
    scene.add(square)
    animation = Fade(square, run_time=.1, final_alpha_value=.3, rate_func=linear, remover=True)
    turn_animation_into_updater(animation)
    scene.wait(.3)
    np.testing.assert_allclose(square.get_fill_opacity(), .3, atol=1e-6)
    assert not square.updaters
    assert any(mob is square for mob in scene.mobjects)


def copy_does_not_drive_source():
    line = Line(ORIGIN, 2 * RIGHT)
    animation = ShowCreation(line, run_time=1, rate_func=linear)
    turn_animation_into_updater(animation)
    line.copy().update(.75)
    assert animation.total_time == 0
    line.update(.5)
    line.update(0)
    np.testing.assert_allclose(line.get_end(), RIGHT, atol=1e-5)


def cyclic_reveal():
    line = Line(ORIGIN, 2 * RIGHT)
    animation = ShowCreation(line, run_time=1, rate_func=linear)
    cycle_animation(animation)
    line.update(2.25)
    line.update(0)
    np.testing.assert_allclose(line.get_end(), .5 * RIGHT, atol=1e-5)
    assert line.updaters


def failure_detaches():
    class Bad(Animation):
        def interpolate_submobject(self, current, start, alpha):
            if alpha > 0:
                raise RuntimeError("persistent callback failed")
    square = Square()
    turn_animation_into_updater(Bad(square, rate_func=linear))
    square.update(.5)
    try:
        square.update(0.)
    except RuntimeError as error:
        assert "persistent callback failed" in str(error)
    else:
        raise AssertionError("callback error must propagate")
    assert not square.updaters
    square.update(.5)


for case in (drawing_protocol, helper_order_and_time, scene_wait_completion,
             copy_does_not_drive_source, cyclic_reveal, failure_detaches):
    case()
print("persistent Python animation acceptance: 6 cases passed")
