"""Production Scene lowering and family-alpha methods with modeled native execution.

The local geometry, lifecycle setup, and clock below are explicit test doubles.
Installed-extension acceptance is separate; no pixel/native proof is implied.
"""
import ast
import importlib.util
import math
from pathlib import Path
from types import ModuleType
import unittest

import numpy as np

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("live_rates_under_test", ROOT / "python/fmn_python/live_rates.py")
live_rates = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(live_rates)


def environment(install=True):
    native = ModuleType("manimlib.live_rates_fixture")
    g = vars(native)
    def linear(t):
        return t
    def smooth(t):
        return t*t*(3-2*t)
    class Mobject:
        def __init__(self):
            self.bound = False
        def _is_bound(self):
            return self.bound
    class CameraFrame(Mobject):
        pass
    class Animation:
        _native_kind = None
        def __init__(self, rate_func=None, run_time=1., time_span=None, lag_ratio=0., family_size=1):
            self.rate_func, self.run_time, self.time_span, self.lag_ratio = rate_func, run_time, time_span, lag_ratio
            self.mobject = Mobject()
            self.remover, self.suspend_mobject_updating, self.final_alpha_value = False, False, 1.
            self.family_size, self.values = family_size, []
        def _ensure_runtime_defaults(self):
            if self.rate_func is None:
                self.rate_func = linear
        def begin(self):
            self._ensure_runtime_defaults()
            self.families = [(index,) for index in range(self.family_size)]
            self.interpolate(0)
        def interpolate(self, alpha):
            self.current_alpha = alpha
            self.interpolate_mobject(alpha)
        def interpolate_submobject(self, index, alpha):
            self.values.append((self.current_alpha, index, float(alpha)))
        def update_mobjects(self, dt):
            pass
        def finish(self):
            self.interpolate(self.final_alpha_value)
        def clean_up_from_scene(self, scene):
            pass
    class Transform(Animation):
        _native_kind, _target_attr = "transform", "target_mobject"
        def __init__(self, *args, **kwargs):
            super().__init__(*args, **kwargs)
            self.target_mobject = Mobject()
        def _native_target(self):
            return self.target_mobject
        def _native_params(self):
            return {}
    class Unsupported(Transform):
        _native_kind, _target_attr = "cyclic_replace", None
    class AnimationGroup(Animation):
        _native_kind, _default_lag_ratio = "animation_group", 0.
        def __init__(self, *children, **kwargs):
            super().__init__(**kwargs)
            self.animations = list(children)
            self.abort_count = 0
        def abort(self):
            self.abort_count += 1
        def _native_params(self):
            return {}
    class Builder:
        def __init__(self, animation):
            self.overridden_animation, self.calls = animation, 0
        def build(self):
            self.calls += 1
            return self.overridden_animation
    class Scene:
        def __init__(self, fps=60):
            self.fps, self.playing, self.executions = fps, False, []
            self.lower_only = False
        def add(self, mob):
            mob.bound = True
        _adopt = add
        def _play_animations(self, specs, callbacks, camera, run_time, rate, lag):
            self.executions.append((specs, callbacks, camera, run_time, rate, lag))
            if self.lower_only:
                return self.executions[-1]
            self.playing = True
            try:
                for callback in callbacks:
                    if callback is not None:
                        callback.begin()
                duration = run_time or max((spec[3] or 1 for spec in specs), default=1)
                for frame in range(1, math.ceil(duration*self.fps)+1):
                    for callback in callbacks:
                        if callback is not None:
                            callback.update_mobjects(1/self.fps)
                            callback.interpolate(min(frame / self.fps / duration, 1))
                for callback in callbacks:
                    if callback is not None:
                        callback.finish()
            finally:
                self.playing = False
            return self.executions[-1]
    class _CompositionCallbackDriver:
        pass
    def ensure_root(group):
        group.root_prepared = True
        if group.mobject is None:
            group.mobject = Mobject()
        return group.mobject
    def requires(animation):
        return not getattr(animation, "_native_kind", None) or bool(getattr(animation, "authored", False))
    g.update(Animation=Animation, Transform=Transform, Unsupported=Unsupported,
             AnimationGroup=AnimationGroup, Scene=Scene, Mobject=Mobject,
             _BridgeMobject=Mobject, CameraFrame=CameraFrame, _AnimationBuilder=Builder,
             _CompositionCallbackDriver=_CompositionCallbackDriver,
             _fmn_ensure_composition_root=ensure_root,
             _requires_python_animation=requires, _python_composition_members=lambda group: [],
             _RATE_FUNC_NAMES={linear:"linear", smooth:"smooth"}, linear=linear, smooth=smooth,
             prepare_animation=lambda builder:builder.build(), np=np, _vec3=tuple)
    bootstrap = ROOT / "python/manimlib_bootstrap.py"
    scene_node = next(node for node in ast.parse(bootstrap.read_text()).body if isinstance(node, ast.ClassDef) and node.name == "Scene")
    method = next(node for node in scene_node.body if isinstance(node, ast.FunctionDef) and node.name == "play")
    exec(compile(ast.Module([method], []), str(bootstrap), "exec"), g)
    Scene.play = g["play"]
    source = ROOT / "python/manimlib/_animation_semantics.py"
    installer = next(node for node in ast.parse(source.read_text()).body if isinstance(node, ast.FunctionDef) and node.name == "install")
    names = {"get_sub_alpha", "time_spanned_alpha", "interpolate_mobject"}
    nodes = [node for node in installer.body if isinstance(node, ast.FunctionDef) and node.name in names]
    assert {node.name for node in nodes} == names
    exec(compile(ast.Module(nodes, []), str(source), "exec"), g)
    for name in names:
        setattr(Animation, name, g[name])
    if install:
        live_rates.install_live_rates(native)
    return native


class LiveRateTests(unittest.TestCase):
    def setUp(self):
        self.n = environment()
        self.scene = self.n.Scene()

    def test_negative_control_samples_callable_before_native_play(self):
        n = environment(False)
        scene, phases = n.Scene(), []
        scene.play(n.Transform(rate_func=lambda t: phases.append(scene.playing) or t))
        self.assertEqual(phases, [False] * 31)
        self.assertEqual(scene.executions[0][0][0][0], "transform")

    def test_leaf_custom_rate_runs_only_at_lifecycle_samples(self):
        phases = []
        animation = self.n.Transform(rate_func=lambda t:phases.append(self.scene.playing) or t*t)
        self.scene.play(animation)
        self.assertEqual(len(phases), 62)
        self.assertTrue(all(phases))
        self.assertEqual(self.scene.executions[0][0][0][0], "python_callback")
        self.assertAlmostEqual(animation.values[1][2], (1/60)**2)

    def test_sixty_hz_curve_keeps_detail_between_old_thirty_hz_samples(self):
        curve = lambda t:t + .02*math.sin(30*math.pi*t)
        animation = self.n.Transform(rate_func=curve)
        self.scene.play(animation)
        self.assertAlmostEqual(animation.values[1][2], 1/60+.02)
        old_interpolant = (curve(0)+curve(1/30))/2
        self.assertGreater(abs(animation.values[1][2] - old_interpolant), .019)

    def test_global_custom_curve_does_not_create_a_native_table(self):
        phases = []
        rate = lambda t:phases.append(self.scene.playing) or t**3
        animation = self.n.Transform()
        self.scene.play(animation, rate_func=rate)
        self.assertTrue(all(phases))
        self.assertIsNone(self.scene.executions[0][4])
        self.assertIs(animation.rate_func, rate)

    def test_global_override_applies_to_all_supported_top_level_members(self):
        first, second = self.n.Transform(rate_func=lambda t:0), self.n.Animation(rate_func=lambda t:0)
        self.scene.play(first, second, rate_func=lambda t:t*t)
        self.assertAlmostEqual(first.values[1][2], (1/60)**2)
        self.assertEqual(first.values, second.values)
        self.assertIsNone(self.scene.executions[0][4])

    def test_group_curve_is_not_written_onto_children(self):
        child_rate, group_rate = lambda t:t*t, lambda t:t**3
        child = self.n.Transform(rate_func=child_rate)
        group = self.n.AnimationGroup(child)
        self.scene.lower_only = True
        self.scene.play(group, rate_func=group_rate)
        self.assertIs(group.rate_func, group_rate)
        self.assertIs(child.rate_func, child_rate)
        self.assertEqual(self.scene.executions[0][0][0][0], "python_callback")

    def test_own_group_curve_selects_callback_without_evaluation(self):
        def rate(t):
            raise AssertionError("route inspection evaluated the curve")
        group = self.n.AnimationGroup(self.n.Transform(), rate_func=rate)
        self.assertTrue(self.n._requires_python_animation(group))

    def test_catalog_curves_remain_native(self):
        for rate in (None, self.n.linear, self.n.smooth, "linear"):
            self.scene.play(self.n.Transform(rate_func=rate))
            self.assertEqual(self.scene.executions[-1][0][0][0], "transform")

    def test_catalog_string_global_override_works_on_callback(self):
        animation = self.n.Transform(rate_func=lambda t:t*t)
        self.scene.play(animation, rate_func="linear")
        self.assertAlmostEqual(animation.values[1][2], 1/60)
        self.assertEqual(self.scene.executions[0][4], "linear")

    def test_own_catalog_string_is_resolved_for_authored_callback(self):
        animation = self.n.Transform(rate_func="smooth")
        animation.authored = True
        self.scene.play(animation)
        self.assertAlmostEqual(animation.values[1][2], self.n.smooth(1/60))

    def test_invalid_catalog_raises_before_execution(self):
        with self.assertRaisesRegex(ValueError, "unknown rate function"):
            self.scene.play(self.n.Transform(), rate_func="missing")
        self.assertEqual(self.scene.executions, [])

    def test_unhashable_callable_runs_without_hash_or_equality_probes(self):
        class Curve:
            __hash__ = None
            def __eq__(self, other):
                raise AssertionError("equality probe")
            def __call__(self, t):
                return t*t
        animation = self.n.Transform(rate_func=Curve())
        self.scene.play(animation)
        self.assertAlmostEqual(animation.values[1][2], (1/60)**2)

    def test_builder_is_prepared_once_for_global_curve(self):
        builder = self.n._AnimationBuilder(self.n.Transform())
        self.scene.play(builder, rate_func=lambda t:t)
        self.assertEqual(builder.calls, 1)

    def test_bad_builder_product_never_enters_play(self):
        builder = self.n._AnimationBuilder(object())
        with self.assertRaisesRegex(TypeError, "must return an Animation"):
            self.scene.play(builder, rate_func=lambda t:t)
        self.assertEqual(self.scene.executions, [])

    def test_lag_and_time_span_evaluate_at_true_sub_alpha(self):
        animation = self.n.Transform(rate_func=lambda t:t*t, time_span=(.2, .8), lag_ratio=.4, family_size=2)
        self.scene.play(animation)
        frame = [value for value in animation.values if np.isclose(value[0], .5)]
        expected = [(.5*1.4)**2, (.5*1.4-.4)**2]
        np.testing.assert_allclose([row[2] for row in frame], expected, atol=1e-14)

    def test_final_alpha_still_runs_through_curve(self):
        animation = self.n.Transform(rate_func=lambda t:t*t)
        animation.final_alpha_value = .25
        self.scene.play(animation)
        self.assertEqual(animation.values[-1], (.25, 0, .0625))

    def test_live_rate_replacement_is_not_constructor_cached(self):
        animation = self.n.Transform(rate_func=self.n.linear)
        self.assertFalse(self.n._requires_python_animation(animation))
        animation.rate_func = lambda t:t*t
        self.assertTrue(self.n._requires_python_animation(animation))
        animation.rate_func = self.n.linear
        self.assertFalse(self.n._requires_python_animation(animation))

    def test_rate_error_retains_exception_identity(self):
        error = ValueError("rate failed")
        def rate(t):
            if t > 0:
                raise error
            return t
        with self.assertRaises(ValueError) as raised:
            self.scene.play(self.n.Transform(rate_func=rate))
        self.assertIs(raised.exception, error)

    def test_play_runtime_and_lag_overrides_are_not_lost(self):
        animation = self.n.Transform(rate_func=lambda t:t*t, family_size=2)
        self.scene.play(animation, run_time=.5, lag_ratio=.75)
        self.assertEqual(animation.run_time, .5)
        self.assertEqual(animation.lag_ratio, .75)
        self.assertEqual(self.scene.executions[0][3], .5)
        self.assertEqual(self.scene.executions[0][5], .75)

    def test_unsupported_native_kind_keeps_existing_global_easing(self):
        self.scene.lower_only = True
        seen = []
        rate = lambda t:seen.append(t) or t
        supported, unsupported = self.n.Transform(), self.n.Unsupported()
        self.scene.play(supported, unsupported, rate_func=rate)
        self.assertEqual(len(seen), 31)
        self.assertIsInstance(self.scene.executions[0][4], list)
        self.assertEqual([spec[0] for spec in self.scene.executions[0][0]], ["transform", "cyclic_replace"])

    def test_unknown_native_leaf_is_not_reinterpreted(self):
        animation = self.n.Unsupported(rate_func=lambda t:t)
        self.assertFalse(self.n._requires_python_animation(animation))

    def test_reinstallation_preserves_later_replacements(self):
        scene_type = self.n.Scene
        replacement = lambda *args:None
        self.n.Scene.play = replacement
        live_rates.install_live_rates(self.n)
        self.assertIs(self.n.Scene, scene_type)
        self.assertIs(self.n.Scene.play, replacement)

    def test_new_callback_group_gets_root_and_scene_context(self):
        group = self.n.AnimationGroup(self.n.Transform(), rate_func=lambda t:t)
        group.mobject = None
        original = self.scene._play_animations
        contexts = []
        def execute(*args):
            contexts.append(group._composition_scene)
            return original(*args)
        self.scene._play_animations = execute
        self.scene.lower_only = True
        self.scene.play(group)
        self.assertTrue(group.root_prepared)
        self.assertEqual(contexts, [self.scene])
        self.assertFalse(hasattr(group, "_composition_scene"))

    def test_nested_custom_group_is_prepared_and_prior_context_restored(self):
        child = self.n.AnimationGroup(rate_func=lambda t:t)
        child.mobject = None
        parent = self.n.AnimationGroup(child, rate_func=lambda t:t*t)
        marker = object()
        child._composition_scene = marker
        self.scene.lower_only = True
        self.scene.play(parent)
        self.assertTrue(child.root_prepared)
        self.assertTrue(parent.root_prepared)
        self.assertIs(child._composition_scene, marker)

    def test_builder_returned_group_is_prepared(self):
        group = self.n.AnimationGroup(rate_func=lambda t:t)
        group.mobject = None
        builder = self.n._AnimationBuilder(group)
        self.scene.lower_only = True
        self.scene.play(builder)
        self.assertEqual(builder.calls, 1)
        self.assertTrue(group.root_prepared)

    def test_group_failure_unwinds_and_restores_context(self):
        group = self.n.AnimationGroup(rate_func=lambda t:t)
        error = RuntimeError("native play failed")
        def fail(*args):
            raise error
        self.scene._play_animations = fail
        with self.assertRaises(RuntimeError) as raised:
            self.scene.play(group)
        self.assertIs(raised.exception, error)
        self.assertEqual(group.abort_count, 1)
        self.assertFalse(hasattr(group, "_composition_scene"))

    def test_group_cycle_fails_before_play(self):
        group = self.n.AnimationGroup(rate_func=lambda t:t)
        group.animations.append(group)
        with self.assertRaisesRegex(ValueError, "contains a cycle"):
            self.scene.play(group)
        self.assertEqual(self.scene.executions, [])

    def test_duplicate_group_identity_is_prepared_once(self):
        group = self.n.AnimationGroup(rate_func=lambda t:t)
        calls = []
        previous = self.n._fmn_ensure_composition_root
        def prepare(item):
            calls.append(item)
            return previous(item)
        self.n._fmn_ensure_composition_root = prepare
        self.scene.lower_only = True
        self.scene.play(group, group)
        self.assertEqual(calls, [group])


if __name__ == "__main__":
    unittest.main()
