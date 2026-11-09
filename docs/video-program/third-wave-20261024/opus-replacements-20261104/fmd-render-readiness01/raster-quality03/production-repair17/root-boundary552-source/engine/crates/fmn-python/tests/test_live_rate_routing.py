"""Source-level dispatch regressions; no native extension or renderer doubles.

These tests exercise the actual installer against minimal protocol objects.
The installed-extension geometry/clock witnesses live in live_fade_rates.py.
Run: python -m unittest discover -s crates/fmn-python/tests -p test_live_rate_routing.py -v
"""
from __future__ import annotations

import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest


_SOURCE = Path(__file__).resolve().parents[1] / "python" / "fmn_python" / "live_rates.py"
_spec = importlib.util.spec_from_file_location("fmn_test_live_rates", _SOURCE)
_adapter = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_adapter)


def protocol():
    def linear(t):
        return t

    class Animation:
        _native_kind = None

        def __init__(self, rate_func=None):
            self.rate_func = linear if rate_func is None else rate_func
            self.run_time = None
            self.time_span = None
            self.lag_ratio = 0.0

    class Transform(Animation):
        _native_kind = "transform"
        _target_attr = "target_mobject"

        def _ensure_runtime_defaults(self):
            pass

    class Fade(Transform):
        _native_kind = "fade"
        _target_attr = None

    class FadeIn(Fade):
        _native_kind = "fade_in"

    class FadeOut(Fade):
        _native_kind = "fade_out"

    class FadeInFromLarge(FadeIn):
        pass

    class FadeInFromPoint(FadeIn):
        pass

    class FadeOutToPoint(FadeOut):
        pass

    class NativeOnly(Animation):
        _native_kind = "native_only"

    class AnimationGroup(Animation):
        _native_kind = "animation_group"

        def __init__(self, *animations, rate_func=None):
            super().__init__(rate_func)
            self.animations = list(animations)
            self.max_end_time = 1.0
            self.aborts = 0

        def abort(self):
            self.aborts += 1

    class Builder:
        def __init__(self, result):
            self.result = result
            self.builds = 0

        def build(self):
            self.builds += 1
            return self.result

    class Scene:
        def play(self, *animations, **kwargs):
            self.received = animations, kwargs
            self.callbacks = tuple(native._requires_python_animation(a) for a in animations)
            if getattr(self, "failure", None) is not None:
                raise self.failure
            return "played"

    native = SimpleNamespace(
        Animation=Animation, Transform=Transform, Fade=Fade, FadeIn=FadeIn, FadeOut=FadeOut,
        FadeInFromLarge=FadeInFromLarge, FadeInFromPoint=FadeInFromPoint,
        FadeOutToPoint=FadeOutToPoint, NativeOnly=NativeOnly, AnimationGroup=AnimationGroup,
        _AnimationBuilder=Builder, Scene=Scene, _RATE_FUNC_NAMES={linear: "linear"}, linear=linear,
        _requires_python_animation=lambda animation: getattr(animation, "authored", False),
        prepare_animation=lambda builder: builder.build(),
        _fmn_ensure_composition_root=lambda group: None,
    )
    _adapter.install_live_rates(native)
    return native


class Curve:
    __hash__ = None

    def __bool__(self):
        raise AssertionError("rate truth value was inspected")

    def __eq__(self, other):
        raise AssertionError("rate equality was inspected")

    def __call__(self, t):
        raise AssertionError("dispatch must not speculatively evaluate easing")


class FadeRateRouting(unittest.TestCase):
    def setUp(self):
        self.native = protocol()

    def families(self):
        return tuple(getattr(self.native, name) for name in (
            "FadeIn", "FadeOut", "FadeInFromLarge", "FadeInFromPoint", "FadeOutToPoint"))

    def test_leaf_custom_rates_select_callbacks_without_sampling(self):
        for cls in self.families():
            with self.subTest(kind=cls.__name__):
                self.assertTrue(self.native._requires_python_animation(cls(Curve())))

    def test_catalog_functions_and_spellings_remain_native(self):
        for cls in self.families():
            for rate in (self.native.linear, "linear", None):
                with self.subTest(kind=cls.__name__, rate=rate):
                    self.assertFalse(self.native._requires_python_animation(cls(rate)))

    def test_mixed_fade_transform_uses_live_play_override(self):
        for cls in self.families():
            with self.subTest(kind=cls.__name__):
                scene, rate = self.native.Scene(), Curve()
                fade, transform = cls(), self.native.Transform()
                self.assertEqual(scene.play(fade, transform, rate_func=rate), "played")
                self.assertEqual(scene.callbacks, (True, True))
                self.assertIsNone(scene.received[1]["rate_func"])
                self.assertIs(fade.rate_func, rate)
                self.assertIs(transform.rate_func, rate)

    def test_unsupported_native_sibling_preserves_global_lowering(self):
        fade, sibling = self.native.FadeIn(), self.native.NativeOnly()
        scene, rate = self.native.Scene(), Curve()
        scene.play(fade, sibling, rate_func=rate)
        self.assertIs(scene.received[1]["rate_func"], rate)
        self.assertIs(fade.rate_func, self.native.linear)
        self.assertEqual(scene.callbacks, (False, False))

    def test_bare_fade_is_not_promoted_without_a_target(self):
        self.assertFalse(self.native._requires_python_animation(self.native.Fade(Curve())))

    def test_builder_returned_fade_is_built_once(self):
        fade, scene, rate = self.native.FadeOut(), self.native.Scene(), Curve()
        builder = self.native._AnimationBuilder(fade)
        scene.play(builder, rate_func=rate)
        self.assertEqual(builder.builds, 1)
        self.assertEqual(scene.callbacks, (True,))
        self.assertIs(scene.received[0][0], fade)

    def test_group_override_does_not_replace_child_easing(self):
        child_rate, group_rate = Curve(), Curve()
        child = self.native.FadeIn(child_rate)
        group = self.native.AnimationGroup(child)
        scene = self.native.Scene()
        scene.play(group, rate_func=group_rate)
        self.assertIs(group.rate_func, group_rate)
        self.assertIs(child.rate_func, child_rate)
        self.assertNotIn("_composition_scene", vars(group))

    def test_callback_defaults_resolve_catalog_strings_for_fades(self):
        for cls in self.families():
            fade = cls("linear")
            fade._ensure_runtime_defaults()
            self.assertIs(fade.rate_func, self.native.linear)

    def test_previous_authored_classifier_is_preserved(self):
        animation = self.native.NativeOnly()
        animation.authored = True
        self.assertTrue(self.native._requires_python_animation(animation))

    def test_installation_is_idempotent(self):
        play = self.native.Scene.play
        _adapter.install_live_rates(self.native)
        self.assertIs(self.native.Scene.play, play)


class PlayRateRecovery(unittest.TestCase):
    def setUp(self):
        self.native = protocol()

    def test_cycle_refuses_before_any_root_rate_is_overwritten(self):
        scene, original, replacement = self.native.Scene(), Curve(), Curve()
        leaf = self.native.FadeIn(original)
        group = self.native.AnimationGroup(rate_func=original)
        group.animations.append(group)
        with self.assertRaisesRegex(ValueError, "cycle"):
            scene.play(leaf, group, rate_func=replacement)
        self.assertIs(leaf.rate_func, original)
        self.assertIs(group.rate_func, original)
        self.assertFalse(hasattr(scene, "received"))

    def test_play_failure_and_cancellation_restore_original_rates(self):
        for failure in (ValueError("begin failed"), KeyboardInterrupt(), SystemExit(3)):
            with self.subTest(kind=type(failure).__name__):
                scene = self.native.Scene()
                scene.failure = failure
                original, replacement = Curve(), Curve()
                fade = self.native.FadeOut(original)
                moving = self.native.Transform("linear")
                with self.assertRaises(type(failure)) as caught:
                    scene.play(fade, moving, rate_func=replacement)
                self.assertIs(caught.exception, failure)
                self.assertIs(fade.rate_func, original)
                self.assertEqual(moving.rate_func, "linear")

    def test_failed_animation_can_be_retried_with_its_original_rate(self):
        scene, original = self.native.Scene(), Curve()
        animation = self.native.FadeIn(original)
        scene.failure = ValueError("transient failure")
        with self.assertRaises(ValueError):
            scene.play(animation, rate_func=Curve())
        scene.failure = None
        scene.play(animation)
        self.assertIs(animation.rate_func, original)
        self.assertEqual(scene.callbacks, (True,))

    def test_success_retains_the_applied_override(self):
        scene, replacement = self.native.Scene(), Curve()
        animation = self.native.FadeIn()
        scene.play(animation, rate_func=replacement)
        self.assertIs(animation.rate_func, replacement)

    def test_root_preparation_failure_restores_rates(self):
        failure = RuntimeError("cannot bind root")
        def refuse(group):
            raise failure
        self.native._fmn_ensure_composition_root = refuse
        original = Curve()
        group = self.native.AnimationGroup(self.native.FadeIn(), rate_func=original)
        with self.assertRaises(RuntimeError) as caught:
            self.native.Scene().play(group, rate_func=Curve())
        self.assertIs(caught.exception, failure)
        self.assertIs(group.rate_func, original)
        self.assertNotIn("_composition_scene", vars(group))

    def test_nested_contexts_and_child_rates_survive_failure(self):
        original, child_rate, replacement, old_scene = Curve(), Curve(), Curve(), object()
        leaf = self.native.FadeIn(child_rate)
        inner = self.native.AnimationGroup(leaf, rate_func=child_rate)
        outer = self.native.AnimationGroup(inner, rate_func=original)
        inner._composition_scene = old_scene
        scene = self.native.Scene()
        scene.failure = RuntimeError("play failed")
        with self.assertRaises(RuntimeError):
            scene.play(outer, rate_func=replacement)
        self.assertIs(outer.rate_func, original)
        self.assertIs(inner.rate_func, child_rate)
        self.assertIs(leaf.rate_func, child_rate)
        self.assertIs(inner._composition_scene, old_scene)
        self.assertNotIn("_composition_scene", vars(outer))
        self.assertEqual((inner.aborts, outer.aborts), (1, 1))

    def test_shared_group_is_bound_and_aborted_once(self):
        original = Curve()
        shared = self.native.AnimationGroup(self.native.FadeIn(), rate_func=original)
        outer = self.native.AnimationGroup(shared, shared)
        scene = self.native.Scene()
        scene.failure = RuntimeError("failed")
        with self.assertRaises(RuntimeError):
            scene.play(outer, shared, rate_func=Curve())
        self.assertEqual(shared.aborts, 1)
        self.assertIs(shared.rate_func, original)

    def test_duplicate_top_level_animation_keeps_one_original_snapshot(self):
        original = Curve()
        animation = self.native.FadeIn(original)
        scene = self.native.Scene()
        scene.failure = RuntimeError("duplicate rejected downstream")
        with self.assertRaises(RuntimeError):
            scene.play(animation, animation, rate_func=Curve())
        self.assertIs(animation.rate_func, original)

    def test_rate_assignment_failure_unwinds_earlier_assignments(self):
        original, replacement = Curve(), Curve()
        failure = ValueError("rate descriptor refused")
        class RefusingFade(self.native.FadeIn):
            @property
            def rate_func(self):
                return self._rate
            @rate_func.setter
            def rate_func(self, value):
                if value is replacement:
                    raise failure
                self._rate = value
        first, second = self.native.FadeIn(original), RefusingFade(original)
        with self.assertRaises(ValueError) as caught:
            self.native.Scene().play(first, second, rate_func=replacement)
        self.assertIs(caught.exception, failure)
        self.assertIs(first.rate_func, original)
        self.assertIs(second.rate_func, original)

    def test_cleanup_failure_does_not_replace_primary_error(self):
        original, replacement = Curve(), Curve()
        failure = ValueError("primary play failure")
        class RefusingRestore(self.native.FadeIn):
            @property
            def rate_func(self):
                return self._rate
            @rate_func.setter
            def rate_func(self, value):
                if value is original and getattr(self, "_rate", None) is replacement:
                    raise RuntimeError("restore failed")
                self._rate = value
        animation = RefusingRestore(original)
        scene = self.native.Scene()
        scene.failure = failure
        with self.assertRaises(ValueError) as caught:
            scene.play(animation, rate_func=replacement)
        self.assertIs(caught.exception, failure)
        self.assertIn("animation easing restoration failed", failure.__notes__)

    def test_catalog_play_override_still_uses_native_catalog(self):
        animation, scene = self.native.FadeIn(), self.native.Scene()
        scene.play(animation, rate_func="linear")
        self.assertIs(scene.received[1]["rate_func"], self.native.linear)
        self.assertEqual(scene.callbacks, (False,))

    def test_invalid_catalog_refuses_before_builder_execution(self):
        builder = self.native._AnimationBuilder(self.native.FadeIn())
        with self.assertRaisesRegex(ValueError, "unknown rate function"):
            self.native.Scene().play(builder, rate_func="not_a_curve")
        self.assertEqual(builder.builds, 0)


if __name__ == "__main__":
    unittest.main()
