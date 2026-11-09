"""Indication orchestration tests with storage doubles, not native-render proof."""
from __future__ import annotations

import copy
import math
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python.indication import install_indication


def native_table():
    def linear(t):
        return t

    def there_and_back(t):
        x = 2 * t if t < .5 else 2 * (1 - t)
        return x ** 3 * (10 - 15 * x + 6 * x * x)

    def wiggle(t, n=2):
        return there_and_back(t) * math.sin(n * math.pi * t)

    class Mobject:
        def __init__(self, points=(), widths=None, children=()):
            self.points = np.array(points, dtype=float).reshape(-1, 3)
            self.widths = np.full(len(self.points), 4.0) if widths is None else np.array(widths, dtype=float)
            self.submobjects = list(children)
            self.suspended = self.animating = False
            self.ticks = 0
            self.color = "white"

        def get_family(self):
            result, seen, stack = [], set(), [self]
            while stack:
                obj = stack.pop()
                if id(obj) not in seen:
                    seen.add(id(obj))
                    result.append(obj)
                    stack.extend(reversed(obj.submobjects))
            return result

        def copy(self):
            return copy.deepcopy(self)

        def get_center(self):
            points = [obj.points for obj in self.get_family() if len(obj.points)]
            if not points:
                return np.zeros(3)
            points = np.concatenate(points)
            return (points.min(axis=0) + points.max(axis=0)) / 2

        def match_points(self, other):
            self.points = other.points.copy()
            return self

        def scale(self, factor, about_point=None):
            point = self.get_center() if about_point is None else np.array(about_point)
            for obj in self.get_family():
                obj.points = point + factor * (obj.points - point)
            return self

        def rotate(self, angle, about_point=None):
            c, s = math.cos(angle), math.sin(angle)
            matrix = np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])
            point = self.get_center() if about_point is None else np.array(about_point)
            for obj in self.get_family():
                obj.points = point + (obj.points - point) @ matrix.T
            return self

        def get_stroke_widths(self):
            return self.widths

        def set_stroke(self, *, width, recurse=True):
            self.widths = np.asarray(width, dtype=float).reshape(-1).copy()
            if recurse:
                for obj in self.submobjects:
                    obj.set_stroke(width=width)
            return self

        def match_style(self, other):
            self.widths, self.color = other.widths.copy(), other.color
            for obj, target in zip(self.submobjects, other.submobjects):
                obj.match_style(target)
            return self

        def set_animating_status(self, value):
            for obj in self.get_family():
                obj.animating = value

        def _is_updating_suspended(self):
            return self.suspended

        def suspend_updating(self, recurse=True):
            for obj in self.get_family() if recurse else [self]:
                obj.suspended = True

        def resume_updating(self, recurse=True, call_updater=True):
            for obj in self.get_family() if recurse else [self]:
                obj.suspended = False
            if call_updater:
                self.update(0)

        def update(self, dt):
            if not self.suspended:
                self.ticks += 1
                for obj in self.submobjects:
                    obj.update(dt)

    class VMobject(Mobject):
        pass

    class Animation:
        _native_kind = None

        def __init__(self, mobject, run_time=1, rate_func=None, lag_ratio=0,
                     time_span=None, remover=False, final_alpha_value=1,
                     suspend_mobject_updating=False, **kwargs):
            if not isinstance(mobject, Mobject):
                raise TypeError("Animation needs a Mobject")
            self.mobject, self.run_time = mobject, run_time
            self.rate_func = linear if rate_func is None else rate_func
            self.lag_ratio, self.time_span = lag_ratio, time_span
            self.remover, self.final_alpha_value = remover, final_alpha_value
            self.suspend_mobject_updating = suspend_mobject_updating
            self.__dict__.update(kwargs)

        def _ensure_runtime_defaults(self):
            self.rate_func = linear if self.rate_func is None else self.rate_func

        def create_starting_mobject(self):
            return self.mobject.copy()

        def get_all_families_zipped(self):
            return zip(self.mobject.get_family(), self.starting_mobject.get_family())

        def begin(self):
            self._ensure_runtime_defaults()
            self.mobject.set_animating_status(True)
            self.starting_mobject = self.create_starting_mobject()
            self.mobject_was_updating = not self.mobject._is_updating_suspended()
            if self.suspend_mobject_updating:
                self.mobject.suspend_updating()
            self.families = list(self.get_all_families_zipped())
            self.interpolate(0)

        def time_spanned_alpha(self, alpha):
            if self.time_span is None:
                return alpha
            start, end = self.time_span
            return np.clip(alpha * self.run_time - start, 0, end - start) / (end - start)

        def get_sub_alpha(self, alpha, index, count):
            return self.rate_func(np.clip(alpha * ((count - 1) * self.lag_ratio + 1)
                                          - index * self.lag_ratio, 0, 1))

        def interpolate(self, alpha):
            return self.interpolate_mobject(alpha)

        def interpolate_mobject(self, alpha):
            for i, pair in enumerate(self.families):
                self.interpolate_submobject(*pair, self.get_sub_alpha(
                    self.time_spanned_alpha(alpha), i, len(self.families)))

        def interpolate_submobject(self, current, start, alpha):
            pass

        def update_mobjects(self, dt):
            self.starting_mobject.update(dt)

        def finish(self):
            self.interpolate(self.final_alpha_value)
            self.mobject.set_animating_status(False)
            if self.suspend_mobject_updating and self.mobject_was_updating:
                self.mobject.resume_updating()

        def clean_up_from_scene(self, scene):
            if self.remover:
                scene.remove(self.mobject)

    class WiggleOutThenIn(Animation):
        _native_kind = "wiggle_out_then_in"

        def __init__(self, mobject, scale_value=1.1, rotation_angle=.02 * math.pi,
                     n_wiggles=6, scale_about_point=None, rotate_about_point=None,
                     run_time=2, **kwargs):
            super().__init__(mobject, run_time=run_time, **kwargs)
            self.scale_value, self.rotation_angle, self.n_wiggles = scale_value, rotation_angle, n_wiggles
            self.scale_about_point, self.rotate_about_point = scale_about_point, rotate_about_point

        def get_scale_about_point(self):
            return self.mobject.get_center() if self.scale_about_point is None else self.scale_about_point

        def get_rotate_about_point(self):
            return self.mobject.get_center() if self.rotate_about_point is None else self.rotate_about_point

    class VShowPassingFlash(Animation):
        _native_kind = "v_show_passing_flash"

        def taper_kernel(self, x):
            if x < self.taper_width:
                return x
            if x > 1 - self.taper_width:
                return 1 - x
            return 1.0

    class FlashAround(VShowPassingFlash):
        pass

    class AnimationGroup(Animation):
        _native_kind = "animation_group"

        def __init__(self, *children):
            super().__init__(VMobject())
            self.animations = list(children)

    class Scene:
        def __init__(self):
            self.routes, self.removed = [], []
            self.failure = None

        def remove(self, obj):
            self.removed.append(obj)

        def play(self, *animations, **kwargs):
            def leaves(animation):
                if isinstance(animation, AnimationGroup):
                    for child in animation.animations:
                        yield from leaves(child)
                else:
                    yield animation
            for animation in (leaf for root in animations for leaf in leaves(root)):
                callback = native._requires_python_animation(animation)
                self.routes.append(callback)
                if callback:
                    if kwargs.get("rate_func") is not None:
                        animation.rate_func = kwargs["rate_func"]
                    animation.begin()
                    animation.interpolate(.5)
                    if self.failure is not None:
                        raise self.failure
                    animation.finish()
                    animation.clean_up_from_scene(self)
            return self

    class Builder:
        def __init__(self, result):
            self.result, self.calls = result, 0

        def build(self):
            self.calls += 1
            return self.result

    native = SimpleNamespace(
        Animation=Animation, AnimationGroup=AnimationGroup, Mobject=Mobject,
        VMobject=VMobject, Scene=Scene, WiggleOutThenIn=WiggleOutThenIn,
        VShowPassingFlash=VShowPassingFlash, FlashAround=FlashAround,
        # The stock rates fmn-core implements cross as names, as in the
        # bootstrap's native catalog; wiggle is not one of them.
        _np=np, _linear_rate=linear,
        _RATE_FUNC_NAMES={linear: "linear", there_and_back: "there_and_back"},
        _requires_python_animation=lambda a: not a._native_kind,
        _AnimationBuilder=Builder, prepare_animation=lambda b: b.build(),
        there_and_back=there_and_back, wiggle=wiggle, linear=linear,
    )
    return native


class IndicationProtocol(unittest.TestCase):
    def setUp(self):
        self.n = native_table()
        self.identities = self.n.WiggleOutThenIn, self.n.VShowPassingFlash, self.n.FlashAround
        install_indication(self.n)

    def line(self, widths=(1, 2, 4, 8, 16)):
        return self.n.VMobject([(x, 0, 0) for x in np.linspace(-1, 1, len(widths))], widths)

    def test_stock_classes_keep_native_dispatch(self):
        for animation in (self.n.WiggleOutThenIn(self.line()),
                          self.n.VShowPassingFlash(self.line()), self.n.FlashAround(self.line())):
            self.assertFalse(self.n._requires_python_animation(animation))

    def test_idempotency_class_identity_and_public_method_identity(self):
        before = self.n.Scene.play
        install_indication(self.n)
        self.assertIs(before, self.n.Scene.play)
        self.assertEqual(self.identities, (self.n.WiggleOutThenIn, self.n.VShowPassingFlash, self.n.FlashAround))
        for cls in self.identities[:2]:
            method = cls.interpolate_submobject
            self.assertEqual(method.__qualname__, cls.__qualname__ + ".interpolate_submobject")
            self.assertEqual(method.__module__, cls.__module__)

    def test_wiggle_midpoint_uses_native_owned_scale_and_rotation_operations(self):
        line = self.line((2, 2))
        initial = line.points.copy()
        animation = self.n.WiggleOutThenIn(line, scale_value=2, rotation_angle=math.pi / 2, n_wiggles=1)
        animation.begin()
        animation.interpolate(.5)
        np.testing.assert_allclose(line.points, [[0, -2, 0], [0, 2, 0]], atol=1e-12)
        animation.finish()
        np.testing.assert_allclose(line.points, initial, atol=1e-12)
        self.assertFalse(line.animating)

    def test_wiggle_is_absolute_not_cumulative(self):
        line = self.line()
        animation = self.n.WiggleOutThenIn(line, scale_value=1.5)
        animation.begin()
        animation.interpolate(.3)
        once = line.points.copy()
        animation.interpolate(.3)
        np.testing.assert_allclose(line.points, once, atol=1e-12)

    def test_pivot_getter_overrides_are_observed(self):
        calls = []
        class Authored(self.n.WiggleOutThenIn):
            def get_scale_about_point(self):
                calls.append("scale")
                return np.array([1., 0, 0])
            def get_rotate_about_point(self):
                calls.append("rotate")
                return np.zeros(3)
        line = self.n.VMobject([[0, 0, 0], [2, 0, 0]])
        animation = Authored(line, scale_value=2, rotation_angle=math.pi / 2, n_wiggles=1)
        self.assertTrue(self.n._requires_python_animation(animation))
        animation.begin()
        calls.clear()
        animation.interpolate(.5)
        self.assertEqual(calls, ["scale", "rotate"])
        np.testing.assert_allclose(line.points, [[0, -1, 0], [0, 3, 0]], atol=1e-12)

    def test_gaussian_matches_independent_scalar_formula(self):
        line = self.line()
        animation = self.n.VShowPassingFlash(line, time_width=1, taper_width=0)
        animation.begin()
        animation.interpolate(.5)
        expected = [w * math.exp(-.5 * ((x - .5) / (1 / 6)) ** 2)
                    for w, x in zip((1, 2, 4, 8, 16), (0, .25, .5, .75, 1))]
        np.testing.assert_allclose(line.widths, expected, atol=1e-12)
        self.assertTrue(self.n._requires_python_animation(animation))

    def test_compact_support_is_exactly_zero_outside_window(self):
        line = self.line()
        animation = self.n.VShowPassingFlash(line, time_width=.3, taper_width=0)
        animation.begin()
        animation.interpolate(.5)
        np.testing.assert_array_equal(line.widths, [0, 0, 4, 0, 0])

    def test_taper_hook_and_public_profile_map(self):
        class Authored(self.n.VShowPassingFlash):
            def taper_kernel(self, x):
                return 2
        line = self.line()
        animation = Authored(line)
        self.assertTrue(self.n._requires_python_animation(animation))
        animation.begin()
        np.testing.assert_array_equal(animation.submob_to_widths[hash(line)], [2, 4, 8, 16, 32])
        animation.interpolate(.5)
        self.assertEqual(line.widths[2], 8)

    def test_begin_samples_current_widths_and_repeated_interpolation_does_not_decay(self):
        line = self.line()
        animation = self.n.VShowPassingFlash(line, time_width=1, taper_width=0)
        line.widths *= 3
        original = line.widths.copy()
        animation.begin()
        animation.interpolate(.5)
        once = line.widths.copy()
        animation.interpolate(.5)
        np.testing.assert_array_equal(line.widths, once)
        animation.finish()
        np.testing.assert_array_equal(line.widths, original)
        line.widths *= 2
        animation.begin()
        animation.finish()
        np.testing.assert_array_equal(line.widths, original * 2)

    def test_point_free_root_and_descendants_keep_distinct_profiles(self):
        a, b = self.line(), self.line((6, 7, 8))
        group = self.n.VMobject(children=(a, b))
        animation = self.n.VShowPassingFlash(group, taper_width=0)
        animation.begin()
        self.assertEqual(len(animation.submob_to_widths[hash(group)]), 0)
        animation.interpolate(.5)
        self.assertEqual(a.widths[2], 4)
        self.assertEqual(b.widths[1], 7)
        animation.finish()
        np.testing.assert_array_equal(a.widths, [1, 2, 4, 8, 16])
        np.testing.assert_array_equal(b.widths, [6, 7, 8])

    def test_full_style_restoration_and_remover_false(self):
        line, scene = self.line(), self.n.Scene()
        animation = self.n.VShowPassingFlash(line, remover=False)
        self.assertTrue(self.n._requires_python_animation(animation))
        animation.begin()
        line.color = "red"
        animation.interpolate(.5)
        animation.finish()
        animation.clean_up_from_scene(scene)
        self.assertEqual(line.color, "white")
        self.assertFalse(scene.removed)
        np.testing.assert_array_equal(line.widths, [1, 2, 4, 8, 16])

    def test_cleanup_removes_only_once_after_success(self):
        line, scene = self.line(), self.n.Scene()
        animation = self.n.VShowPassingFlash(line)
        animation.begin()
        animation.finish()
        animation.finish()
        animation.clean_up_from_scene(scene)
        animation.clean_up_from_scene(scene)
        self.assertEqual(scene.removed, [line])

    def test_abort_restores_widths_and_owned_suspension_without_ticks(self):
        line = self.line()
        animation = self.n.VShowPassingFlash(line, suspend_mobject_updating=True)
        animation.begin()
        animation.interpolate(.5)
        animation.abort()
        np.testing.assert_array_equal(line.widths, [1, 2, 4, 8, 16])
        self.assertFalse(line.suspended)
        self.assertFalse(line.animating)
        self.assertEqual(line.ticks, 0)

    def test_preexisting_child_suspension_is_preserved(self):
        child = self.line()
        child.suspended = True
        root = self.n.VMobject(children=(child,))
        animation = self.n.WiggleOutThenIn(root, suspend_mobject_updating=True)
        animation.begin()
        animation.finish()
        self.assertFalse(root.suspended)
        self.assertTrue(child.suspended)
        self.assertEqual(child.ticks, 0)

    def test_nested_scene_failure_unwinds_and_preserves_original_exception(self):
        line, scene = self.line(), self.n.Scene()
        animation = self.n.VShowPassingFlash(line, rate_func=lambda x: x, suspend_mobject_updating=True)
        failure = RuntimeError("sibling failed")
        scene.failure = failure
        with self.assertRaises(RuntimeError) as caught:
            scene.play(self.n.AnimationGroup(animation))
        self.assertIs(caught.exception, failure)
        self.assertFalse(line.animating or line.suspended)
        self.assertFalse(scene.removed)
        np.testing.assert_array_equal(line.widths, [1, 2, 4, 8, 16])

    def test_authored_interpolation_failure_restores_flash_style(self):
        class Bad(self.n.VShowPassingFlash):
            def interpolate_submobject(self, current, start, alpha):
                super().interpolate_submobject(current, start, alpha)
                if alpha > 0:
                    raise RuntimeError("authored failure")
        line = self.line()
        animation = Bad(line, suspend_mobject_updating=True)
        animation.begin()
        with self.assertRaisesRegex(RuntimeError, "authored failure"):
            animation.interpolate(.5)
        self.assertFalse(line.animating or line.suspended)
        np.testing.assert_array_equal(line.widths, [1, 2, 4, 8, 16])

    def test_helper_update_failure_aborts(self):
        animation = self.n.VShowPassingFlash(self.line(), suspend_mobject_updating=True)
        animation.begin()
        def fail(dt):
            raise RuntimeError("helper")
        animation.starting_mobject.update = fail
        with self.assertRaisesRegex(RuntimeError, "helper"):
            animation.update_mobjects(.1)
        self.assertFalse(animation.mobject.animating or animation.mobject.suspended)

    def test_invalid_taper_fails_before_any_style_or_flag_mutation(self):
        class Bad(self.n.VShowPassingFlash):
            def taper_kernel(self, x):
                return float("nan")
        line = self.line()
        with self.assertRaisesRegex(ValueError, "taper_kernel"):
            Bad(line).begin()
        self.assertFalse(line.animating)
        np.testing.assert_array_equal(line.widths, [1, 2, 4, 8, 16])

    def test_invalid_window_and_wrong_object_fail_precisely(self):
        for value in (0, -1, float("nan"), float("inf"), 5e-324):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.n.VShowPassingFlash(self.line(), time_width=value)
        with self.assertRaises(TypeError):
            self.n.VShowPassingFlash(self.n.Mobject())

    def test_post_construction_invalid_parameters_fail_before_play(self):
        animation, scene = self.n.VShowPassingFlash(self.line()), self.n.Scene()
        animation.time_width = 0
        with self.assertRaises(ValueError):
            scene.play(animation)
        self.assertFalse(scene.routes)

    def test_invalid_wiggle_and_pivot_are_rejected(self):
        for options in ({"scale_value": float("nan")}, {"n_wiggles": float("inf")},
                        {"scale_about_point": [0, 1]}, {"rotate_about_point": [0, 0, float("nan")]}):
            with self.subTest(options=options), self.assertRaises(ValueError):
                self.n.WiggleOutThenIn(self.line(), **options).begin()

    def test_family_lag_and_time_span_are_applied_by_shared_animation(self):
        a, b = self.line(), self.line()
        root = self.n.VMobject(children=(a, b))
        animation = self.n.VShowPassingFlash(root, taper_width=0, lag_ratio=1,
                                            run_time=3, time_span=(1, 3))
        animation.begin()
        animation.interpolate(2 / 3)
        self.assertEqual(a.widths[2], 4)
        self.assertEqual(b.widths[2], 0)

    def test_partial_wiggle_endpoint_is_respected(self):
        line = self.line((2, 2))
        animation = self.n.WiggleOutThenIn(line, scale_value=2, rotation_angle=0, final_alpha_value=.25)
        self.assertTrue(self.n._requires_python_animation(animation))
        animation.begin()
        animation.finish()
        np.testing.assert_allclose(line.points[:, 0], [-1.5, 1.5], atol=1e-12)

    def test_live_unhashable_rate_and_top_level_override(self):
        class Curve:
            __hash__ = None
            def __call__(self, t):
                return t * t
        rate = Curve()
        animation = self.n.WiggleOutThenIn(self.line())
        scene = self.n.Scene()
        scene.play(animation, rate_func=rate)
        self.assertEqual(scene.routes, [True])
        self.assertIs(animation.rate_func, rate)
        self.assertNotIn("_indication_force_callback", vars(animation))

    def test_unknown_rate_refuses_before_flags(self):
        line = self.line()
        with self.assertRaisesRegex(ValueError, "unknown rate"):
            self.n.WiggleOutThenIn(line, rate_func="not-a-rate").begin()
        self.assertFalse(line.animating)

    def test_authored_object_methods_and_later_monkeypatches_select_callbacks(self):
        class Authored(self.n.VMobject):
            def set_stroke(self, **kwargs):
                return super().set_stroke(**kwargs)
        self.assertTrue(self.n._requires_python_animation(self.n.VShowPassingFlash(Authored())))
        self.n.Animation.begin = lambda self: None
        self.assertTrue(self.n._requires_python_animation(self.n.WiggleOutThenIn(self.line())))

    def test_builder_prepared_once_and_cycles_rejected(self):
        builder = self.n._AnimationBuilder(self.n.WiggleOutThenIn(self.line()))
        self.n.Scene().play(builder)
        self.assertEqual(builder.calls, 1)
        group = self.n.AnimationGroup()
        group.animations.append(group)
        with self.assertRaisesRegex(ValueError, "cycle"):
            self.n.Scene().play(group)

    def test_no_missing_exports_are_manufactured(self):
        minimal = SimpleNamespace()
        install_indication(minimal)
        self.assertEqual(vars(minimal), {})


if __name__ == "__main__":
    unittest.main()
