"""Installed-wheel acceptance for authored Rotating/Rotate playback."""
import numpy as np
from manimlib import Dot, Group, RIGHT, UP, ORIGIN, Rotate, Rotating, Scene, linear


def stock_rotation():
    scene = Scene()
    dot = Dot(RIGHT)
    scene.add(dot)
    scene.play(Rotate(dot, angle=np.pi / 2, about_point=ORIGIN, rate_func=linear, run_time=1 / 30))
    np.testing.assert_allclose(dot.get_center()[:2], UP[:2], atol=1e-5)


def authored_rate_and_final_alpha():
    scene = Scene()
    dot = Dot(RIGHT)
    seen = []
    rate = lambda t: seen.append(float(t)) or t * t
    animation = Rotate(
        dot, angle=np.pi, about_point=ORIGIN, rate_func=rate,
        final_alpha_value=.5, run_time=2 / 30,
    )
    scene.add(dot)
    scene.play(animation)
    # final alpha .5 passes through the authored t^2 curve -> pi/4.
    root = np.sqrt(.5)
    np.testing.assert_allclose(dot.get_center()[:2], [root, root], atol=2e-4)
    assert seen and any(0 < value < 1 for value in seen)


def subclass_hook_dispatch():
    events = []
    class CustomRotate(Rotate):
        def interpolate_mobject(self, alpha):
            events.append(float(alpha))
            return super().interpolate_mobject(alpha)
    scene = Scene()
    dot = Dot(RIGHT)
    scene.add(dot)
    scene.play(CustomRotate(dot, angle=np.pi / 2, about_point=ORIGIN, rate_func=linear, run_time=1 / 30))
    assert events
    np.testing.assert_allclose(dot.get_center()[:2], UP[:2], atol=1e-5)


def mobject_hook_dispatch():
    calls = []
    class CustomDot(Dot):
        def rotate(self, angle, *args, **kwargs):
            calls.append(float(angle))
            return super().rotate(angle, *args, **kwargs)
    scene = Scene()
    dot = CustomDot(RIGHT)
    scene.add(dot)
    scene.play(Rotate(dot, angle=np.pi / 2, about_point=ORIGIN, rate_func=linear, run_time=1 / 30))
    assert calls
    np.testing.assert_allclose(dot.get_center()[:2], UP[:2], atol=1e-5)


def nested_rotation_callback():
    events = []
    class CustomRotating(Rotating):
        def interpolate_mobject(self, alpha):
            events.append(float(alpha))
            return super().interpolate_mobject(alpha)
    scene = Scene()
    a, b = Dot(RIGHT), Dot(2 * RIGHT)
    scene.add(a, b)
    scene.play(
        Group(
            a,
            b,
        ).animate.shift(0 * RIGHT),
        CustomRotating(b, angle=np.pi / 2, about_point=ORIGIN, rate_func=linear, run_time=1 / 30),
        run_time=1 / 30,
        rate_func=linear,
    )
    assert events


for case in (
    stock_rotation,
    authored_rate_and_final_alpha,
    subclass_hook_dispatch,
    mobject_hook_dispatch,
    nested_rotation_callback,
):
    case()
print("rotation playback acceptance: 5 cases passed")

# Object protocols must be honored even when the Animation class is stock.
from pathlib import Path as _Path
import tempfile as _tempfile
import unittest as _unittest
from unittest.mock import patch as _patch
import manimlib as _m


class RotationObjectProtocols(_unittest.TestCase):
    def rotation(self, obj, cls=None, **kwargs):
        return (cls or _m.Rotate)(obj, angle=np.pi / 2, about_point=_m.ORIGIN,
                                  rate_func=_m.linear, run_time=.1, **kwargs)

    def test_plain_rotations_keep_native_admission(self):
        for cls in (_m.Rotate, _m.Rotating):
            for obj in (_m.Square(), _m.Circle(), _m.VGroup(_m.Square(), _m.Circle())):
                with self.subTest(cls=cls, obj=type(obj)):
                    self.assertFalse(_m._requires_python_animation(self.rotation(obj, cls)))
        class Plain(_m.Square):
            pass
        self.assertFalse(_m._requires_python_animation(self.rotation(Plain())))

    def test_authored_copy_changes_actual_start_geometry_and_frame_samples(self):
        for cls in (_m.Rotate, _m.Rotating):
            with self.subTest(cls=cls):
                calls, samples = [], []
                class Authored(_m.Square):
                    def copy(self):
                        calls.append(self)
                        return super().copy().shift(_m.UP)
                obj = Authored().shift(2 * _m.RIGHT)
                start = obj.get_points().copy() + _m.UP
                obj.add_updater(lambda mob, dt: samples.append(mob.get_points().copy())
                                if mob is obj and dt > 0 else None, call=False)
                animation = self.rotation(obj, cls)
                animation.run_time = .125
                self.assertTrue(_m._requires_python_animation(animation))
                scene = _m.Scene()
                with _tempfile.TemporaryDirectory() as directory:
                    with scene.render_session(_Path(directory) / 'samples.y4m',
                                              resolution=(48, 32), fps=24):
                        scene.play(animation)
                self.assertIn(obj, calls)
                self.assertEqual(len(samples), 3)
                for index, points in enumerate(samples, 1):
                    theta = index * np.pi / 6
                    matrix = np.array([[np.cos(theta), -np.sin(theta), 0],
                                       [np.sin(theta), np.cos(theta), 0], [0, 0, 1]])
                    np.testing.assert_allclose(points, start @ matrix.T, atol=3e-6)
                np.testing.assert_allclose(obj.get_center(), [-1, 2, 0], atol=3e-6)

    def test_snapshot_is_taken_at_nested_begin_not_spec_lowering(self):
        calls = []
        class Authored(_m.Square):
            def copy(self):
                calls.append(self.get_center().copy())
                return super().copy().shift(_m.UP)
        obj = Authored()
        rotation = self.rotation(obj)
        class Prefix(_m.Animation):
            def finish(self):
                super().finish()
                obj.shift(2 * _m.RIGHT)
        _m.Scene().play(_m.Succession(Prefix(_m.Mobject(), run_time=.1), rotation))
        self.assertTrue(calls)
        np.testing.assert_allclose(calls[-1], [2, 0, 0], atol=1e-6)
        np.testing.assert_allclose(obj.get_center(), [-1, 2, 0], atol=3e-6)

    def test_lower_level_point_transform_override_executes_on_live_receiver(self):
        calls = []
        class Authored(_m.Square):
            def apply_points_function(self, function, *args, **kwargs):
                if getattr(self, 'armed', False):
                    calls.append(self)
                    original = function
                    function = lambda points: original(points) + .25 * _m.OUT
                return super().apply_points_function(function, *args, **kwargs)
        obj = Authored().shift(2 * _m.RIGHT)
        obj.armed = True
        animation = self.rotation(obj)
        self.assertTrue(_m._requires_python_animation(animation))
        _m.Scene().play(animation)
        self.assertTrue(calls)
        self.assertTrue(all(receiver is obj for receiver in calls))
        np.testing.assert_allclose(obj.get_center(), [0, 2, .25], atol=3e-6)

    def test_selected_family_rotation_leaves_excluded_geometry_unchanged(self):
        class Selected(_m.VGroup):
            def get_family(self, recurse=True):
                return [self, *self.submobjects[:1]] if recurse else [self]
        first, excluded = _m.Dot(_m.RIGHT), _m.Dot(2 * _m.RIGHT)
        obj = Selected(first, excluded)
        excluded_points = excluded.get_points().copy()
        scene = _m.Scene()
        scene.add(obj)
        scene.play(self.rotation(obj))
        np.testing.assert_allclose(first.get_center(), [0, 1, 0], atol=3e-6)
        np.testing.assert_array_equal(excluded.get_points(), excluded_points)

    def test_direct_rotation_maps_custom_pointlike_fields_and_preserves_data(self):
        calls = []
        class Authored(_m.Mobject):
            data_dtype = [('point', 3), ('normal_point', 3), ('tag', 1)]
            pointlike_data_keys = ['point', 'normal_point']
            def init_points(self):
                self.set_points([[2, 0, 0], [3, 0, 0]])
                self.data['normal_point'][:] = [[2, 0, 1], [3, 0, 1]]
                self.data['tag'][:] = [[11], [29]]
            def apply_points_function(self, function, *args, **kwargs):
                calls.append(self)
                return super().apply_points_function(function, *args, **kwargs)
        for bound in (False, True):
            with self.subTest(bound=bound):
                obj = Authored()
                if bound:
                    scene = _m.Scene()
                    scene.add(obj)
                points, normals, tags = (obj.data[key].copy()
                                         for key in ('point', 'normal_point', 'tag'))
                obj.rotate(np.pi / 2, about_point=[1, 0, 0])
                for key, original in (('point', points), ('normal_point', normals)):
                    expected = np.column_stack((1 - original[:, 1], original[:, 0] - 1,
                                                original[:, 2]))
                    np.testing.assert_allclose(obj.data[key], expected, atol=3e-6)
                np.testing.assert_array_equal(obj.data['tag'], tags)
        self.assertEqual(len(calls), 2)

    def test_copy_descriptor_is_not_evaluated_during_admission(self):
        reads = []
        class Authored(_m.Square):
            @property
            def copy(self):
                reads.append(self)
                return lambda: _m.Square.copy(self).shift(_m.UP)
        obj = Authored()
        animation = self.rotation(obj)
        self.assertTrue(_m._requires_python_animation(animation))
        self.assertEqual(reads, [])
        _m.Scene().play(animation)
        self.assertTrue(reads)
        np.testing.assert_allclose(obj.get_center(), [-1, 0, 0], atol=3e-6)

    def test_animation_mobject_descriptor_is_not_evaluated_during_admission(self):
        reads = []
        class Authored(_m.Rotate):
            pass
        obj = _m.Square()
        animation = self.rotation(obj, Authored)
        def get(instance):
            reads.append(instance)
            return instance.__dict__['mobject']
        Authored.mobject = property(get, lambda obj, value: obj.__dict__.__setitem__('mobject', value))
        self.assertTrue(_m._requires_python_animation(animation))
        self.assertEqual(reads, [])

    def test_submobject_descriptor_does_not_run_during_family_admission(self):
        reads = []
        class Authored(_m.VGroup):
            pass
        obj = Authored(_m.Square())
        def get(instance):
            reads.append(instance)
            return instance.__dict__['submobjects']
        Authored.submobjects = property(get, lambda obj, value: obj.__dict__.__setitem__('submobjects', value))
        self.assertTrue(_m._requires_python_animation(self.rotation(obj)))
        self.assertEqual(reads, [])

    def test_descendant_copy_protocol_is_detected_without_family_callbacks(self):
        calls = []
        class Child(_m.Square):
            def copy(self):
                calls.append(self)
                return super().copy()
        child = Child()
        shared = _m.VGroup(_m.VGroup(child), _m.VGroup(child))
        self.assertTrue(_m._requires_python_animation(self.rotation(shared)))
        self.assertEqual(calls, [])

    def test_late_instance_and_shared_base_copy_hooks_are_not_frozen_away(self):
        obj = _m.Square()
        animation = self.rotation(obj)
        self.assertFalse(_m._requires_python_animation(animation))
        obj.copy = lambda: _m.Square.copy(obj).shift(_m.UP)
        self.assertTrue(_m._requires_python_animation(animation))
        _m.Scene().play(animation)
        np.testing.assert_allclose(obj.get_center(), [-1, 0, 0], atol=3e-6)
        original = _m.Mobject.copy
        def changed(self):
            return original(self).shift(_m.UP)
        with _patch.object(_m.Mobject, 'copy', changed):
            other = _m.Square()
            self.assertTrue(_m._requires_python_animation(self.rotation(other)))
            _m.Scene().play(self.rotation(other))
            np.testing.assert_allclose(other.get_center(), [-1, 0, 0], atol=3e-6)

    def test_authored_snapshot_failure_unwinds_and_scene_can_play_again(self):
        error, failing = RuntimeError('rotation snapshot failed'), [True]
        class Authored(_m.Square):
            def copy(self):
                if failing[0]:
                    raise error
                return super().copy().shift(_m.UP)
        obj, scene = Authored(), _m.Scene()
        animation = self.rotation(obj, suspend_mobject_updating=True)
        with self.assertRaises(RuntimeError) as caught:
            scene.play(animation)
        self.assertIs(caught.exception, error)
        self.assertFalse(obj._is_updating_suspended())
        failing[0] = False
        scene.play(animation)
        self.assertFalse(obj._is_updating_suspended())
        np.testing.assert_allclose(obj.get_center(), [-1, 0, 0], atol=3e-6)

    def test_authored_easing_observes_exact_48fps_rotation_samples(self):
        seen = []
        class Authored(_m.Square):
            def copy(self):
                return super().copy().shift(_m.UP)
        def rate(alpha):
            seen.append(alpha)
            return alpha * alpha
        obj = Authored()
        with _tempfile.TemporaryDirectory() as directory:
            scene = _m.Scene()
            with scene.render_session(_Path(directory) / 'motion.y4m', resolution=(48, 32), fps=48):
                scene.add(obj)
                scene.play(_m.Rotate(obj, angle=np.pi / 2, about_point=_m.ORIGIN,
                                     rate_func=rate, run_time=3 / 48))
        self.assertEqual(len(seen), 5)  # begin(0), three captures, finish(1)
        np.testing.assert_allclose(seen, [0, 1/3, 2/3, 1, 1], atol=1e-15)
        np.testing.assert_allclose(obj.get_center(), [-1, 0, 0], atol=3e-6)

    def test_rendered_snapshots_match_independent_native_controls(self):
        class Authored(_m.Square):
            def copy(self):
                return super().copy().shift(_m.UP)
        def render(path, kind, workers):
            scene = _m.Scene()
            with scene.render_session(path, resolution=(64, 40), fps=24, threads=workers):
                obj = (Authored() if kind == 'authored' else _m.Square()).scale(.5).shift(_m.RIGHT)
                if kind == 'expected':
                    obj.shift(_m.UP)
                scene.add(obj)
                scene.play(_m.Rotate(obj, angle=np.pi / 2, about_point=_m.ORIGIN,
                                     rate_func=_m.linear, run_time=.125))
            return path.read_bytes()
        with _tempfile.TemporaryDirectory() as directory:
            root = _Path(directory)
            expected = render(root / 'expected.y4m', 'expected', 1)
            self.assertNotEqual(expected, render(root / 'ignored.y4m', 'ignored', 1))
            for workers in (1, 4, 16):
                with self.subTest(workers=workers):
                    self.assertEqual(expected, render(root / f'{workers}.y4m', 'authored', workers))
            frames = expected.split(b'FRAME\n')[1:]
            self.assertEqual(len(frames), 3)
            self.assertEqual(len(set(frames)), 3)


_rotation_result = _unittest.TextTestRunner(verbosity=2).run(
    _unittest.defaultTestLoader.loadTestsFromTestCase(RotationObjectProtocols))
if not _rotation_result.wasSuccessful():
    raise AssertionError('rotation object protocols failed')
