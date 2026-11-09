"""Installed-wheel cyclic transforms on real native objects and Choreo frames."""
import math
import pathlib
import tempfile

import numpy as np
import manimlib as m
from manimlib.animation.transform import CyclicReplace as QualifiedCyclicReplace
from manimlib.animation.transform import Swap as QualifiedSwap


def close(actual, expected):
    assert np.allclose(actual, expected, atol=2e-5), (actual, expected)


def pair():
    scene = m.Scene()
    left = m.Square().scale(.25).shift(m.LEFT)
    right = m.Circle().scale(.25).shift(m.RIGHT)
    scene.add(left, right)
    return scene, left, right


def group_identity_and_live_targets():
    scene, left, right = pair()
    third = m.Triangle().scale(.25).shift(3 * m.RIGHT)
    scene.add(third)
    objects = (left, right, third)
    colors = [obj.get_color() for obj in objects]
    animation = m.CyclicReplace(*objects, path_arc=0, run_time=.125, rate_func=m.linear)
    assert QualifiedCyclicReplace is m.CyclicReplace and QualifiedSwap is m.Swap
    assert isinstance(animation.mobject, m.Group)
    assert all(child is source for child, source in zip(animation.mobject, objects))
    assert animation.target_mobject is None
    right.shift(m.UP)  # Must be sampled at begin, not construction.
    positions = [obj.get_center().copy() for obj in objects]
    scene.play(animation)
    for index, obj in enumerate(objects):
        close(obj.get_center(), positions[(index + 1) % len(objects)])
        assert obj.get_color() == colors[index]
    assert all(child is source for child, source in zip(animation.mobject, objects))


def direct_arc_midpoint():
    scene, left, right = pair()
    animation = m.Swap(left, right, rate_func=m.linear)
    animation.begin()
    animation.interpolate(.5)
    close(left.get_center(), (0, 1 - math.sqrt(2), 0))
    close(right.get_center(), (0, math.sqrt(2) - 1, 0))
    animation.finish()
    animation.clean_up_from_scene(scene)
    close(left.get_center(), m.RIGHT)
    close(right.get_center(), m.LEFT)


def authored_path_and_helpers():
    scene, left, right = pair()
    samples, calls = [], []
    left.add_updater(lambda obj: samples.append(obj.get_center().copy()), call=False)

    def path(start, end, alpha):
        calls.append(alpha)
        return (1 - alpha) * start + alpha * end + 4 * alpha * (1 - alpha) * m.UP

    scene.play(m.Swap(left, right, path_func=path, rate_func=m.linear, run_time=.25))
    close(left.get_center(), m.RIGHT)
    close(right.get_center(), m.LEFT)
    assert len(calls) > 2 and max(point[1] for point in samples) > .5


def authored_target_and_interpolation():
    scene, left, right = pair()
    calls = []

    class LiftedSwap(m.Swap):
        def create_target(self):
            calls.append("target")
            return super().create_target().shift(m.UP)

        def interpolate_submobject(self, *args):
            calls.append("interpolate")
            return super().interpolate_submobject(*args)

    scene.play(LiftedSwap(left, right, path_arc=0, run_time=.125, rate_func=m.linear))
    close(left.get_center(), m.RIGHT + m.UP)
    close(right.get_center(), m.LEFT + m.UP)
    assert calls.count("target") == 1 and calls.count("interpolate") > 2


def succession_samples_predecessor_final_state():
    scene, left, right = pair()
    swap = m.Swap(left, right, path_arc=0, run_time=.125, rate_func=m.linear)
    scene.play(m.Succession(
        left.animate(run_time=.125, rate_func=m.linear).shift(m.UP),
        swap,
    ))
    close(left.get_center(), m.RIGHT)
    close(right.get_center(), m.LEFT + m.UP)
    assert swap.target_mobject is not None


def partial_endpoint_and_removal():
    scene, left, right = pair()
    animation = m.Swap(left, right, path_arc=0, run_time=.125,
                       rate_func=m.linear, final_alpha_value=.25)
    scene.play(animation)
    close(left.get_center(), .5 * m.LEFT)
    close(right.get_center(), .5 * m.RIGHT)
    scene.play(m.Swap(left, right, path_arc=0, run_time=.125,
                      rate_func=m.linear, remover=True))
    visible = [member for root in scene.mobjects for member in root.get_family()]
    assert all(member is not left and member is not right for member in visible)


def callback_failure_releases_suspension():
    scene, left, right = pair()
    failure = RuntimeError("cyclic-path-failed")

    def path(start, end, alpha):
        if alpha > 0:
            raise failure
        return start.copy()

    animation = m.Swap(left, right, path_func=path, run_time=.125,
                       rate_func=m.linear, suspend_mobject_updating=True)
    try:
        scene.play(animation)
    except RuntimeError as error:
        assert error is failure
    else:
        raise AssertionError("the authored path error must escape playback")
    assert not animation.mobject._is_updating_suspended()
    assert not left._is_updating_suspended() and not right._is_updating_suspended()
    assert not left.locked_data_keys and not right.locked_data_keys


def animation_updater_drives_the_whole_group():
    scene, left, right = pair()
    animation = m.Swap(left, right, path_arc=0, run_time=1, rate_func=m.linear)
    group = m.turn_animation_into_updater(animation)
    assert group is animation.mobject
    scene.add(group)
    group.update(.5)
    group.update(0)
    close(left.get_center(), m.ORIGIN)
    close(right.get_center(), m.ORIGIN)
    group.update(.5)
    group.update(0)
    close(left.get_center(), m.RIGHT)
    close(right.get_center(), m.LEFT)


def rendered_motion():
    from fmn_python import render_scene

    class SwappingSquares(m.Scene):
        default_camera_config = dict(resolution=(96, 54), fps=8)

        def construct(self):
            left = m.Square(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)
            right = m.Square(side_length=1, fill_color=m.RED, fill_opacity=1, stroke_width=0)
            left.shift(2 * m.LEFT)
            right.shift(2 * m.RIGHT)
            self.add(left, right)
            self.wait(1 / 8)
            self.play(m.Swap(left, right, path_arc=math.pi, run_time=1 / 4, rate_func=m.linear))
            close(left.get_center(), 2 * m.RIGHT)
            close(right.get_center(), 2 * m.LEFT)

    root = pathlib.Path(tempfile.mkdtemp(prefix="fmn-cyclic-replace-"))
    first = render_scene(SwappingSquares, root / "swap.y4m", threads=1)
    second = render_scene(SwappingSquares, root / "repeat.y4m", threads=2)
    assert first.frame_count == second.frame_count == 3
    payload = first.destination.read_bytes()
    assert payload == second.destination.read_bytes()
    header, raw = payload.split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2"
    frame_size = 96 * 54 * 3 // 2
    assert len(raw) == 3 * (6 + frame_size)
    centers = []
    for offset in range(0, len(raw), 6 + frame_size):
        assert raw[offset:offset + 6] == b"FRAME\n"
        luma = np.frombuffer(raw[offset + 6:offset + 6 + 96 * 54], dtype=np.uint8).reshape(54, 96)
        ys, xs = np.nonzero(luma > 200)
        assert len(xs) > 5
        centers.append((float(xs.mean()), float(ys.mean())))
    assert centers[0][0] + 5 < centers[1][0] < centers[2][0] - 5
    assert centers[1][1] > centers[0][1] + 5


for case in (group_identity_and_live_targets, direct_arc_midpoint, authored_path_and_helpers,
             authored_target_and_interpolation, succession_samples_predecessor_final_state,
             partial_endpoint_and_removal, callback_failure_releases_suspension,
             animation_updater_drives_the_whole_group, rendered_motion):
    case()
    print("cyclic replacement acceptance:", case.__name__)
