"""Installed-extension cases for deferred and mixed persistent animations."""
from pathlib import Path
import tempfile
import numpy as np
from manimlib import (
    Animation, AnimationGroup, Succession, Scene, Square, Swap,
    LEFT, RIGHT, linear, turn_animation_into_updater, cycle_animation,
)


class OpacityRamp(Animation):
    def interpolate_submobject(self, current, start, alpha):
        current.set_opacity(alpha)


class ShiftOne(Animation):
    def interpolate_submobject(self, current, start, alpha):
        current.match_points(start)
        current.shift(alpha * RIGHT)


def detached_swap_wait():
    left, right = Square().shift(2 * LEFT), Square().shift(2 * RIGHT)
    animation = Swap(left, right, path_arc=0, rate_func=linear, run_time=.1)
    root = turn_animation_into_updater(animation)
    assert root is animation.mobject
    assert all(any(member is operand for member in root.get_family())
               for operand in (left, right))
    assert len(root.updaters) == 1
    assert animation.total_time == 0
    scene = Scene()
    scene.add(root)
    scene.wait(.3)
    np.testing.assert_allclose(left.get_center(), 2 * RIGHT, atol=1e-5)
    assert not root.updaters


def detached_mixed_group():
    left, right, faded = Square().shift(2 * LEFT), Square().shift(2 * RIGHT), Square(fill_opacity=1)
    group = AnimationGroup(
        Swap(left, right, path_arc=0, rate_func=linear, run_time=1),
        OpacityRamp(faded, rate_func=linear, run_time=1),
        run_time=1, rate_func=linear,
    )
    root = turn_animation_into_updater(group)
    scene = Scene()
    scene.add(root)
    root.update(.5)
    root.update(0.)
    np.testing.assert_allclose(left.get_center(), (0, 0, 0), atol=1e-5)
    np.testing.assert_allclose(faded.get_fill_opacity(), .5, atol=1e-5)
    root.update(.5)
    root.update(0.)
    assert not root.updaters
    assert any(mob is root for mob in scene.mobjects)


def deferred_same_object_succession():
    left, right = Square().shift(2 * LEFT), Square().shift(2 * RIGHT)
    group = Succession(
        Swap(left, right, path_arc=0, rate_func=linear, run_time=1),
        ShiftOne(left, rate_func=linear, run_time=1),
        run_time=2, rate_func=linear,
    )
    root = turn_animation_into_updater(group)
    scene = Scene()
    scene.add(root)
    root.update(1.5)
    root.update(0.)
    np.testing.assert_allclose(left.get_center(), 2.5 * RIGHT, atol=1e-5)
    root.clear_updaters()


def cancellation_before_adoption():
    left, right = Square().shift(2 * LEFT), Square().shift(2 * RIGHT)
    root = cycle_animation(Swap(left, right, path_arc=0, rate_func=linear))
    root.clear_updaters()
    assert not root.updaters
    scene = Scene()
    scene.add(root)
    scene.wait(.2)
    np.testing.assert_allclose(left.get_center(), 2 * LEFT, atol=1e-5)


def foreign_participant_after_adoption():
    left, right = Square(), Square()
    root = turn_animation_into_updater(Swap(left, right))
    Scene().add(left)
    Scene().add(right)
    try:
        root.update(0.)
    except (ValueError, RuntimeError) as error:
        assert 'multiple Scenes' in str(error)
    else:
        raise AssertionError('foreign participant must be refused after adoption')
    assert not root.updaters


def nested_group_context():
    events = []
    class Authored(AnimationGroup):
        def begin(self):
            events.append(self._composition_scene)
            super().begin()
    left, right = Square().shift(2 * LEFT), Square().shift(2 * RIGHT)
    inner = Authored(Swap(left, right, path_arc=0, rate_func=linear))
    outer = AnimationGroup(inner)
    root = turn_animation_into_updater(outer)
    scene = Scene()
    scene.add(root)
    root.update(.5)
    root.update(0.)
    assert events == [scene]
    assert '_composition_scene' not in inner.__dict__
    root.clear_updaters()


def read_luma(path):
    data = Path(path).read_bytes()
    header, separator, payload = data.partition(b'\n')
    if not separator or not header.startswith(b'YUV4MPEG2 '):
        raise ValueError('invalid Y4M header')
    tags = header.split()[1:]
    width = int(next(tag[1:] for tag in tags if tag.startswith(b'W')))
    height = int(next(tag[1:] for tag in tags if tag.startswith(b'H')))
    if width <= 0 or height <= 0 or width % 2 or height % 2:
        raise ValueError('expected positive even Y4M dimensions')
    if not any(tag.startswith(b'C420') for tag in tags):
        raise ValueError('expected planar 4:2:0')
    size, frames = width * height * 3 // 2, []
    while payload:
        marker, separator, payload = payload.partition(b'\n')
        if not separator or marker.split()[:1] != [b'FRAME'] or len(payload) < size:
            raise ValueError('truncated or malformed Y4M frame')
        frames.append(np.frombuffer(payload[:width * height], dtype=np.uint8).reshape(height, width).copy())
        payload = payload[size:]
    return frames


def rendered_background_motion():
    class Background(Scene):
        def construct(self):
            visible = Square(side_length=.7, fill_opacity=1, stroke_width=0, color='#FFFFFF').shift(2 * LEFT)
            invisible = Square(side_length=.3, fill_opacity=0, stroke_width=0).shift(2 * RIGHT)
            root = turn_animation_into_updater(Swap(visible, invisible, path_arc=0, rate_func=linear, run_time=.5))
            self.add(root)
            self.wait(.75)
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / 'background.y4m'
        Background().render(path, resolution=(128, 64), fps=8, threads=1)
        frames = read_luma(path)
        assert len(frames) == 6
        centers = []
        for frame in frames:
            _, xs = np.nonzero(frame > 200)
            assert len(xs) > 0
            centers.append(float(xs.mean()) + .5)
        # Analytic constant-speed swap under an affine camera: four equal
        # displacements, then the stable final pose. This is a pixel witness,
        # independent of the animated mobjects' position getters.
        assert centers[0] < 64 < centers[4]
        assert centers[4] - centers[0] > 10
        np.testing.assert_allclose(centers[:5], np.linspace(centers[0], centers[4], 5), atol=1.)
        np.testing.assert_allclose(centers[5], centers[4], atol=.1)


for case in (detached_swap_wait, detached_mixed_group, deferred_same_object_succession,
             cancellation_before_adoption, foreign_participant_after_adoption, nested_group_context,
             rendered_background_motion):
    case()
print('deferred/native composition acceptance: 7 cases passed')
