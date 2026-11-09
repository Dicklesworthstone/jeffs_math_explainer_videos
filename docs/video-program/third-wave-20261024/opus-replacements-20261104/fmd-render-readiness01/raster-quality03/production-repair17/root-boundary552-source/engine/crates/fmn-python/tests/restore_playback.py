"""Installed-wheel restoration over real native objects, cameras and playback."""
import numpy as np
import manimlib as m
from manimlib.animation.transform import Restore as QualifiedRestore


def close(actual, expected):
    assert np.allclose(actual, expected, atol=2e-5), (actual, expected)


def saved_target_identity():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.save_state()
    saved = square.saved_state
    square.shift(2 * m.RIGHT)
    animation = m.Restore(square, run_time=.125, rate_func=m.linear)
    square.save_state()
    square.shift(m.RIGHT)
    assert animation.target_mobject is saved
    scene.play(animation)
    close(square.get_center(), m.ORIGIN)
    assert square in scene.mobjects
    assert QualifiedRestore is m.Restore and isinstance(animation, m.Transform)


def authored_path():
    scene, square, samples, calls = m.Scene(), m.Square(), [], []
    scene.add(square)
    square.save_state()
    square.shift(2 * m.RIGHT)
    square.add_updater(lambda obj: samples.append(tuple(obj.get_center())), call=False)
    def path(start, end, alpha):
        calls.append(alpha)
        return (1 - alpha) * start + alpha * end + 4 * alpha * (1 - alpha) * m.UP
    scene.play(m.Restore(square, run_time=.25, rate_func=m.linear, path_func=path))
    close(square.get_center(), m.ORIGIN)
    assert calls and max(point[1] for point in samples) > .5


def authored_transform_hooks():
    scene, square, calls = m.Scene(), m.Square(), []
    scene.add(square)
    square.save_state()
    square.shift(2 * m.RIGHT)
    class Restore(m.Restore):
        def create_target(self):
            calls.append("target")
            return super().create_target()
        def interpolate_submobject(self, *args):
            calls.append("interpolate")
            return super().interpolate_submobject(*args)
    scene.play(Restore(square, run_time=.125, rate_func=m.linear))
    close(square.get_center(), m.ORIGIN)
    assert "target" in calls and calls.count("interpolate") > 1


def nested_succession():
    scene, square, positions = m.Scene(), m.Square(), []
    scene.add(square)
    square.save_state()
    square.add_updater(lambda obj: positions.append(obj.get_x()), call=False)
    scene.play(m.Succession(
        square.animate(run_time=.125, rate_func=m.linear).shift(2 * m.RIGHT),
        m.Restore(square, run_time=.125, rate_func=m.linear),
    ))
    close(square.get_center(), m.ORIGIN)
    assert max(positions) > 1


def mixed_camera_restore():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    frame, core = scene.frame, scene.frame._core
    initial = (frame.get_center().copy(), frame.get_shape(), frame.get_field_of_view(),
               tuple(core.orientation()))
    frame.save_state()
    frame.shift(2 * m.RIGHT).scale(.5).set_theta(.5).set_field_of_view(.7)
    scene.play(m.AnimationGroup(
        m.Restore(frame, run_time=.125, rate_func=m.linear),
        square.animate(run_time=.125, rate_func=m.linear).shift(m.UP),
    ))
    assert scene.frame is frame and frame._core is core
    for actual, expected in zip((frame.get_center(), frame.get_shape(), frame.get_field_of_view(),
                                 tuple(core.orientation())), initial):
        close(actual, expected)
    close(square.get_center(), m.UP)
    assert frame not in scene.mobjects


def partial_restoration():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.save_state()
    square.shift(4 * m.RIGHT)
    scene.play(m.Restore(square, run_time=.125, rate_func=m.linear, final_alpha_value=.25))
    close(square.get_center(), 3 * m.RIGHT)


def remover_restoration():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.save_state()
    square.shift(m.RIGHT)
    scene.play(m.Restore(square, run_time=.125, rate_func=m.linear, remover=True))
    close(square.get_center(), m.ORIGIN)
    assert square not in scene.mobjects


def direct_transform_lifecycle():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.save_state()
    square.shift(4 * m.RIGHT)
    animation = m.Restore(square, rate_func=m.linear)
    animation.begin()
    animation.interpolate(.5)
    close(square.get_center(), 2 * m.RIGHT)
    animation.finish()
    animation.clean_up_from_scene(scene)
    close(square.get_center(), m.ORIGIN)


def camera_authored_path_retains_pose_identity_and_partial_endpoint():
    scene = m.Scene()
    frame, core, alphas = scene.frame, scene.frame._core, []
    frame.save_state()
    frame.shift(4 * m.RIGHT)
    def path(start, end, alpha):
        alphas.append(alpha)
        return (1 - alpha) * start + alpha * end + 4 * alpha * (1 - alpha) * m.UP
    animation = m.Restore(frame, path_func=path, rate_func=m.linear,
                          suspend_mobject_updating=True, final_alpha_value=.5, run_time=.125)
    scene.play(m.AnimationGroup(animation))
    close(frame.get_center(), [2, 1, 0])
    assert frame is scene.frame and frame._core is core and not frame._is_bound()
    assert not frame._is_updating_suspended()
    assert animation.path_func is path and any(0 < a < 1 for a in alphas)


def persistent_restore_uses_the_same_authored_path():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.save_state()
    square.shift(4 * m.RIGHT)
    def path(start, end, alpha):
        return (1 - alpha) * start + alpha * end + 4 * alpha * (1 - alpha) * m.UP
    animation = m.Restore(square, path_func=path, rate_func=m.linear, run_time=1)
    assert m.turn_animation_into_updater(animation) is square
    square.update(.5)
    square.update(0)
    close(square.get_center(), [2, 1, 0])
    square.update(.5)
    square.update(0)
    close(square.get_center(), m.ORIGIN)
    assert not square.updaters


def invalid_path_is_rejected_without_touching_saved_or_live_geometry():
    square = m.Square()
    square.save_state()
    saved = square.saved_state
    original = saved.get_points().copy()
    square.shift(m.RIGHT)
    current = square.get_points().copy()
    try:
        m.Restore(square, path_func=object())
    except TypeError as error:
        assert "path_func must be callable" in str(error)
    else:
        raise AssertionError("noncallable restoration path was accepted")
    np.testing.assert_array_equal(square.get_points(), current)
    np.testing.assert_array_equal(saved.get_points(), original)
    assert square.saved_state is saved


def failing_restore_preserves_error_and_recovers_for_the_next_segment():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.save_state()
    square.shift(2 * m.RIGHT)
    failure = LookupError("restoration path failed")
    def path(start, end, alpha):
        if alpha > 0:
            raise failure
        return start
    try:
        scene.play(m.Restore(square, path_func=path, suspend_mobject_updating=True,
                             run_time=.125))
    except LookupError as error:
        assert error is failure
    else:
        raise AssertionError("restoration swallowed the authored exception")
    assert not square._is_updating_suspended()
    assert square in scene.mobjects
    assert "_fmn_scene_execution" not in vars(scene)
    scene.play(m.Restore(square, run_time=.125))
    close(square.get_center(), m.ORIGIN)


for case in (saved_target_identity, authored_path, authored_transform_hooks, nested_succession,
             mixed_camera_restore, partial_restoration, remover_restoration, direct_transform_lifecycle,
             camera_authored_path_retains_pose_identity_and_partial_endpoint,
             persistent_restore_uses_the_same_authored_path,
             invalid_path_is_rejected_without_touching_saved_or_live_geometry,
             failing_restore_preserves_error_and_recovers_for_the_next_segment):
    case()
    print("restore playback acceptance:", case.__name__)
