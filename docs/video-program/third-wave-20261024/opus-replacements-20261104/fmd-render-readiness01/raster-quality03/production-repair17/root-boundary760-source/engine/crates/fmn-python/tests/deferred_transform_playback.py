"""Installed-wheel deferred targets and complex arcs on actual native geometry."""
import math
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m
from manimlib.animation.transform import ApplyComplexFunction as QualifiedComplex


def close(actual, expected):
    assert np.allclose(actual, expected, atol=2e-5), (actual, expected)


def square_scene():
    scene, square = m.Scene(), m.Square().scale(.25)
    scene.add(square)
    return scene, square


def successive_methods_accumulate():
    scene, square = square_scene()
    scene.play(m.Succession(
        m.ApplyMethod(square.shift, m.RIGHT, run_time=.125, rate_func=m.linear),
        m.ApplyMethod(square.shift, m.RIGHT, run_time=.125, rate_func=m.linear),
    ))
    close(square.get_center(), 2 * m.RIGHT)


def authored_function_observes_predecessor():
    scene, square = square_scene()
    calls = []

    def function(target):
        calls.append(target.get_center().copy())
        assert target is not square
        return target.shift(m.UP)

    animation = m.ApplyFunction(function, square, run_time=.125, rate_func=m.linear)
    sequence = m.Succession(
        square.animate(run_time=.125, rate_func=m.linear).shift(2 * m.RIGHT),
        animation,
    )
    assert calls == []
    scene.play(sequence)
    assert len(calls) == 1
    close(calls[0], 2 * m.RIGHT)
    close(square.get_center(), 2 * m.RIGHT + m.UP)


def center_function_observes_predecessor():
    scene, square = square_scene()
    scene.play(m.Succession(
        m.ApplyMethod(square.shift, 3 * m.RIGHT, run_time=.125, rate_func=m.linear),
        m.ApplyPointwiseFunctionToCenter(lambda point: 2 * point, square,
                                         run_time=.125, rate_func=m.linear),
    ))
    close(square.get_center(), 6 * m.RIGHT)
    close(square.get_width(), .5)


def matrix_uses_translated_geometry():
    scene, square = square_scene()
    scene.play(m.Succession(
        m.ApplyMethod(square.shift, 2 * m.RIGHT, run_time=.125, rate_func=m.linear),
        m.ApplyMatrix([[0, -1], [1, 0]], square, run_time=.125, rate_func=m.linear),
    ))
    close(square.get_center(), 2 * m.UP)


def complex_arc_midpoint_and_endpoint():
    scene, line = m.Scene(), m.Line(m.RIGHT, 2 * m.RIGHT, buff=0)
    scene.add(line)
    animation = m.ApplyComplexFunction(lambda z: 1j * z, line, rate_func=m.linear)
    assert QualifiedComplex is m.ApplyComplexFunction
    animation.begin()
    animation.interpolate(.5)
    diagonal = (m.RIGHT + m.UP) / math.sqrt(2)
    close(line.get_start(), diagonal)
    close(line.get_end(), 2 * diagonal)
    animation.finish()
    animation.clean_up_from_scene(scene)
    close(line.get_start(), m.UP)
    close(line.get_end(), 2 * m.UP)


def explicit_complex_path_is_not_replaced():
    scene, line = m.Scene(), m.Line(m.RIGHT, 2 * m.RIGHT, buff=0)
    scene.add(line)
    calls = []

    def path(start, end, alpha):
        calls.append(alpha)
        return (1 - alpha) * start + alpha * end

    animation = m.ApplyComplexFunction(lambda z: 1j * z, line,
                                       path_func=path, rate_func=m.linear)
    animation.begin()
    animation.interpolate(.5)
    assert animation.path_func is path and len(calls) > 1
    close(line.get_start(), .5 * (m.RIGHT + m.UP))
    animation.finish()
    animation.clean_up_from_scene(scene)


def complex_play_uses_live_closure_and_hooks():
    scene, line = m.Scene(), m.Line(m.RIGHT, 2 * m.RIGHT, buff=0)
    scene.add(line)
    factor, calls = [1], []

    class Complex(m.ApplyComplexFunction):
        def interpolate_submobject(self, *args):
            calls.append(args[-1])
            return super().interpolate_submobject(*args)

    animation = Complex(lambda z: factor[0] * z, line, run_time=.25, rate_func=m.linear)
    factor[0] = 1j
    scene.play(animation)
    assert any(0 < alpha < 1 for alpha in calls)
    close(animation.path_arc, math.pi / 2)
    close(line.get_start(), m.UP)
    close(line.get_end(), 2 * m.UP)


def complex_animation_updater_uses_the_same_arc():
    scene, line = m.Scene(), m.Line(m.RIGHT, 2 * m.RIGHT, buff=0)
    scene.add(line)
    animation = m.ApplyComplexFunction(lambda z: 1j * z, line, run_time=1, rate_func=m.linear)
    assert m.turn_animation_into_updater(animation) is line
    line.update(.5)
    line.update(0)
    close(line.get_start(), (m.RIGHT + m.UP) / math.sqrt(2))
    line.update(.5)
    line.update(0)
    close(line.get_start(), m.UP)
    assert not line.updaters


def target_failure_preserves_predecessor_result():
    scene, square = square_scene()
    failure = RuntimeError("deferred-target-failed")
    calls = []

    def function(target):
        calls.append(target.get_center().copy())
        target.shift(m.UP)
        raise failure

    try:
        scene.play(m.Succession(
            m.ApplyMethod(square.shift, 2 * m.RIGHT, run_time=.125, rate_func=m.linear),
            m.ApplyFunction(function, square, run_time=.125, rate_func=m.linear),
        ))
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError("the target function's error must propagate")
    assert len(calls) == 1
    close(calls[0], 2 * m.RIGHT)
    close(square.get_center(), 2 * m.RIGHT)
    assert not square._is_updating_suspended()


def replay_samples_new_state_instead_of_cached_target():
    scene, square = square_scene()
    calls = []

    def function(target):
        calls.append(target.get_center().copy())
        return target.shift(m.RIGHT)

    animation = m.ApplyFunction(function, square, run_time=.125, rate_func=m.linear)
    scene.play(animation)
    first_target = animation.target_mobject
    square.shift(3 * m.RIGHT)
    scene.play(animation)
    close(calls, [[0, 0, 0], [4, 0, 0]])
    close(square.get_center(), 5 * m.RIGHT)
    assert animation.target_mobject is not first_target


def method_keywords_and_pointwise_functions_observe_predecessor():
    scene, square = square_scene()
    options = {"about_point": m.ORIGIN}
    scene.play(m.Succession(
        m.ApplyMethod(square.shift, m.RIGHT, run_time=.125, rate_func=m.linear),
        m.ApplyMethod(square.scale, 2, options, run_time=.125, rate_func=m.linear),
        m.ApplyPointwiseFunction(lambda point: point + m.UP, square,
                                run_time=.125, rate_func=m.linear),
    ))
    close(square.get_center(), 2 * m.RIGHT + m.UP)
    close(square.get_width(), 1)
    assert len(options) == 1 and options["about_point"] is m.ORIGIN


def deferred_transforms_share_persistent_composition_lifecycle():
    scene, square = square_scene()
    calls = []

    def function(target):
        calls.append(target.get_center().copy())
        return target.shift(m.UP)

    sequence = m.Succession(
        m.ApplyMethod(square.shift, 2 * m.RIGHT, run_time=.5, rate_func=m.linear),
        m.ApplyFunction(function, square, run_time=.5, rate_func=m.linear),
        rate_func=m.linear,
    )
    root = m.turn_animation_into_updater(sequence)
    scene.add(root)
    assert calls == []
    root.update(.75)
    root.update(0)
    assert len(calls) == 1
    close(calls[0], 2 * m.RIGHT)
    close(square.get_center(), 2 * m.RIGHT + .5 * m.UP)
    root.update(.25)
    root.update(0)
    close(square.get_center(), 2 * m.RIGHT + m.UP)
    assert not root.updaters


def later_target_error_does_not_execute_unstarted_targets():
    scene, square = square_scene()
    calls, failure = [], LookupError("second target failed")

    def fail(target):
        calls.append(target.get_center().copy())
        raise failure

    def never(target):
        raise AssertionError("a future target ran while planning the composition")

    try:
        scene.play(m.Succession(
            m.ApplyMethod(square.shift, m.RIGHT, run_time=.125, rate_func=m.linear),
            m.ApplyFunction(fail, square, run_time=.125),
            m.ApplyFunction(never, square, run_time=.125),
        ))
    except LookupError as error:
        assert error is failure
    else:
        raise AssertionError("the target failure was swallowed")
    close(calls, [[1, 0, 0]])
    close(square.get_center(), m.RIGHT)
    assert not square._is_updating_suspended()
    assert "_fmn_scene_execution" not in vars(scene)
    scene.play(m.ApplyMethod(square.shift, m.UP, run_time=.125))
    close(square.get_center(), m.RIGHT + m.UP)


def deferred_sequence_frames_match_independently_positioned_native_geometry():
    def render(destination, threads, stationary=None):
        scene, square = m.Scene(), m.Square(fill_color=m.BLUE, fill_opacity=1, stroke_width=0)
        square.scale(.5)
        with scene.render_session(destination, format="png_sequence", resolution=(96, 64),
                                  fps=4, threads=threads):
            scene.add(square)
            if stationary is None:
                scene.play(m.Succession(
                    m.ApplyMethod(square.shift, m.RIGHT, run_time=.5, rate_func=m.linear),
                    m.ApplyMethod(square.shift, m.RIGHT, run_time=.5, rate_func=m.linear),
                    rate_func=m.linear,
                ))
            else:
                square.move_to(stationary * m.RIGHT)
                scene.wait(.25)
        return [file.read_bytes() for file in sorted(Path(destination).glob("*.png"))]

    with tempfile.TemporaryDirectory(prefix="fmn-deferred-targets-") as directory:
        root = Path(directory)
        frames = render(root / "one", 1)
        assert len(frames) == 4 and len(set(frames)) == 4
        assert frames == render(root / "four", 4), "thread count changed sequential transforms"
        for index, center in enumerate((.5, 1., 1.5, 2.)):
            expected, = render(root / f"reference-{index}", 1, center)
            assert frames[index] == expected, (index, center)


for case in (successive_methods_accumulate, authored_function_observes_predecessor,
             center_function_observes_predecessor, matrix_uses_translated_geometry,
             complex_arc_midpoint_and_endpoint, explicit_complex_path_is_not_replaced,
             complex_play_uses_live_closure_and_hooks, complex_animation_updater_uses_the_same_arc,
             target_failure_preserves_predecessor_result,
             replay_samples_new_state_instead_of_cached_target,
             method_keywords_and_pointwise_functions_observe_predecessor,
             deferred_transforms_share_persistent_composition_lifecycle,
             later_target_error_does_not_execute_unstarted_targets,
             deferred_sequence_frames_match_independently_positioned_native_geometry):
    case()
    print("deferred transform acceptance:", case.__name__)
