"""Source-level planner/driver protocol tests, not native rendering acceptance."""
import ast
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest

SOURCE = Path(__file__).resolve().parents[1] / "python" / "manimlib" / "_animation_semantics.py"
spec = importlib.util.spec_from_file_location("matching_semantics_under_test", SOURCE)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def environment():
    class Mobject:
        def __init__(self, *children, shape=None, center=0):
            self.children, self.shape, self.center = list(children), shape, center
            self._scene, self.animating = None, False
        def get_family(self):
            result = [self]
            for child in self.children:
                for member in child.get_family():
                    if all(member is not old for old in result):
                        result.append(member)
            return result
        def family_members_with_points(self):
            return [obj for obj in self.get_family() if obj.shape is not None]
        def has_same_shape_as(self, other):
            return [m.shape for m in self.family_members_with_points()] == [m.shape for m in other.family_members_with_points()]
        def get_center(self):
            return self.center
        def _is_bound(self):
            return self._scene is not None
        def unlock_data(self):
            self.unlocked = True
        def resume_updating(self):
            self.resumed = True
        def set_animating_status(self, value):
            self.animating = value
    class VMobject(Mobject):
        pass
    class Group(Mobject):
        pass
    class VGroup(VMobject):
        pass
    class CameraFrame(Mobject):
        pass
    class Animation:
        _native_kind = None
        def __init__(self, mobject=None, run_time=None, rate_func=None, lag_ratio=None, **kwargs):
            self.mobject, self.run_time, self.rate_func, self.lag_ratio = mobject, run_time, rate_func, lag_ratio
            self.time_span = kwargs.get("time_span")
            self.suspend_mobject_updating = kwargs.get("suspend_mobject_updating", False)
            self.final_alpha_value = kwargs.get("final_alpha_value", 1.)
            self.remover = kwargs.get("remover", False)
            self.kwargs, self.events = kwargs, []
        def _ensure_runtime_defaults(self):
            if self.run_time is None:
                self.run_time = 1.
            if self.rate_func is None:
                self.rate_func = lambda t: t*t
            if self.lag_ratio is None:
                self.lag_ratio = 0.
        def get_run_time(self):
            return 1. if self.run_time is None else self.run_time
        def _native_target(self):
            return getattr(self, "target_mobject", None)
        def _native_params(self):
            return {}
        def begin(self):
            self.events.append(("begin",))
        def update_mobjects(self, dt):
            self.events.append(("update", dt))
        def interpolate(self, alpha):
            self.events.append(("interpolate", alpha))
        def finish(self):
            self.events.append(("finish",))
        def clean_up_from_scene(self, scene):
            self.events.append(("cleanup",))
    class NativeAnimation(Animation):
        pass
    class Transform(NativeAnimation):
        _native_kind = "transform"
        def __init__(self, source, target, **kwargs):
            super().__init__(source, **kwargs)
            self.target_mobject = target
    class FadeInFromPoint(NativeAnimation):
        _native_kind = "fade_in_from_point"
        def __init__(self, source, point, **kwargs):
            super().__init__(source, **kwargs)
            self.point = point
        def _native_params(self):
            return {"point": self.point}
    class FadeOutToPoint(FadeInFromPoint):
        _native_kind = "fade_out_to_point"
    class AnimationGroup(NativeAnimation):
        _native_kind = "animation_group"
        def __init__(self, *animations, run_time=None, lag_ratio=0, **kwargs):
            super().__init__(run_time=run_time, lag_ratio=lag_ratio, **kwargs)
            self.animations = list(animations)
    class TransformMatchingParts(NativeAnimation):
        _native_kind = "transform_matching_parts"
    class TransformMatchingShapes(TransformMatchingParts):
        _native_kind = "transform_matching_shapes"
    class Core:
        def __init__(self, spec):
            self.spec, self.events = spec, []
        def get_run_time(self):
            return 1. if self.spec[3] is None else self.spec[3]
        def begin(self):
            self.events.append(("begin",))
        def update_mobjects(self, dt):
            self.events.append(("update", dt))
        def interpolate(self, alpha):
            self.events.append(("interpolate", alpha))
        def finish(self, scene):
            self.events.append(("finish",))
        def clean_up_from_scene(self):
            self.events.append(("cleanup",))
        def abort(self):
            self.events.append(("abort",))
    class NativeLeaf:
        def __init__(self, scene, core):
            self.scene, self.core = scene, core
        def begin(self):
            self.core.begin()
        def update_mobjects(self, dt):
            self.core.update_mobjects(dt)
        def interpolate(self, alpha):
            self.core.interpolate(alpha)
        def finish(self):
            self.core.finish(self.scene)
        def clean_up_from_scene(self, scene):
            self.core.clean_up_from_scene()
        def abort(self):
            self.core.abort()
    class CallbackDriver:
        def __init__(self, group, children):
            self.group, self.children = group, children
        def begin(self):
            for child in self.children:
                child.begin()
        def update_mobjects(self, dt):
            for child in self.children:
                child.update_mobjects(dt)
        def interpolate(self, alpha):
            for child in self.children:
                child.interpolate(self.group.rate_func(alpha) if self.group.rate_func else alpha)
        def finish(self):
            for child in self.children:
                child.finish()
        def clean_up_from_scene(self, scene):
            for child in self.children:
                child.clean_up_from_scene(scene)
        def abort(self):
            for child in self.children:
                if isinstance(child, (NativeLeaf, CallbackDriver)):
                    child.abort()
    class Scene:
        def __init__(self):
            self.roots, self.cores = [], []
        def _adopt(self, obj):
            for member in obj.get_family():
                member._scene = self
        def add(self, *objects):
            for obj in objects:
                self._adopt(obj)
                if all(obj is not old for old in self.roots):
                    self.roots.append(obj)
        def remove(self, *objects):
            ids = {id(obj) for obj in objects}
            self.roots[:] = [obj for obj in self.roots if id(obj) not in ids]
        def _native_animation_driver(self, spec):
            core = Core(spec)
            self.cores.append(core)
            return core
        def play(self, *animations, run_time=None, rate_func=None, lag_ratio=None):
            for animation in animations:
                self.add(animation.mobject)
                animation.begin()
            if getattr(self, "fail_updater", False):
                raise ValueError("scene updater failure")
            for animation in animations:
                animation.interpolate(.5)
                animation.finish()
                animation.clean_up_from_scene(self)
    g = dict(locals())
    g.update(_requires_python_animation=lambda a: not a._native_kind,
             _CompositionCallbackDriver=CallbackDriver, _NativeCompositionLeaf=NativeLeaf,
             _RATE_FUNC_NAMES={}, _linear_rate=lambda t: t,
             _composition_member_run_time=lambda a: a.get_run_time())
    return g


class MatchingPartsProtocolTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        self.original = self.g["TransformMatchingParts"]
        module._install_matching_parts(self.g)
        self.Parts = self.g["TransformMatchingParts"]
        self.square = lambda center=0: self.g["VMobject"](shape="square", center=center)
        self.circle = lambda center=0: self.g["VMobject"](shape="circle", center=center)
        self.group = self.g["VGroup"]

    def test_shared_initializer_calls_matching_installer(self):
        tree = ast.parse(SOURCE.read_text())
        install = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "install")
        self.assertTrue(any(isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                            and node.func.id == "_install_matching_parts" for node in ast.walk(install)))

    def test_class_identity_and_reference_mro(self):
        self.assertIs(self.Parts, self.original)
        self.assertTrue(issubclass(self.Parts, self.g["AnimationGroup"]))
        self.assertTrue(issubclass(self.g["TransformMatchingShapes"], self.Parts))
        self.assertIsNone(self.g["TransformMatchingShapes"]._native_kind)

    def test_explicit_pair_precedes_shape_matching(self):
        a, b, x, y = [self.square() for _ in range(4)]
        animation = self.Parts(self.group(a, b), self.group(x, y), matched_pairs=[(a, y)])
        self.assertEqual([(p.mobject, p.target_mobject) for p in animation.animations], [(a, y), (b, x)])

    def test_overrides_determine_actual_plan(self):
        calls = []
        class Custom(self.Parts):
            def find_pairs_with_matching_shapes(self, sources, targets):
                calls.append((tuple(sources), tuple(targets)))
                return [(sources[-1], targets[0])]
            def add_transform(self, source, target):
                calls.append((source, target))
                return super().add_transform(source, target)
        a, b, x = self.square(), self.square(), self.square()
        animation = Custom(self.group(a, b), x)
        self.assertIs(animation.animations[0].mobject, b)
        self.assertIs(animation.animations[1].mobject, a)
        self.assertEqual(len(calls), 2)

    def test_custom_match_and_mismatch_factories(self):
        class Match(self.g["Transform"]):
            pass
        class Mismatch(self.g["Transform"]):
            pass
        a, b, x, y = self.square(), self.square(), self.square(), self.circle()
        animation = self.Parts(self.group(a, b), self.group(x, y), matched_pairs=[(b, y)],
                               match_animation=Match, mismatch_animation=Mismatch, path_arc=.5)
        self.assertIs(type(animation.animations[0]), Mismatch)
        self.assertIs(type(animation.animations[1]), Match)
        self.assertEqual(animation.animations[0].kwargs["path_arc"], .5)

    def test_group_claim_consumes_descendants_once(self):
        a, b, x, y = [self.square() for _ in range(4)]
        source, target = self.group(a, b), self.group(x, y)
        animation = self.Parts(source, target, matched_pairs=[(source, target), (a, x)])
        self.assertEqual(len(animation.animations), 1)
        self.assertEqual(animation.source_pieces, [])
        self.assertEqual(animation.target_pieces, [])

    def test_unmatched_directional_fades(self):
        source, target = self.square(-3), self.circle(4)
        animation = self.Parts(source, target)
        self.assertEqual([a._native_kind for a in animation.animations], ["fade_out_to_point", "fade_in_from_point"])
        self.assertEqual([a.point for a in animation.animations], [4, -3])

    def test_empty_families_and_one_sided_fade(self):
        self.assertEqual(self.Parts(self.group(), self.group()).animations, [])
        animation = self.Parts(self.group(), self.square())
        self.assertEqual(len(animation.animations), 1)
        self.assertEqual(animation.animations[0]._native_kind, "fade_in_from_point")

    def test_native_leaf_lowering_and_cleanup(self):
        source, target, unrelated = self.square(), self.square(2), self.circle(8)
        animation = self.Parts(source, target)
        scene = self.g["Scene"]()
        scene.add(source, unrelated, animation.mobject)
        animation.begin()
        animation.update_mobjects(.1)
        animation.interpolate(.5)
        animation.finish()
        animation.clean_up_from_scene(scene)
        self.assertEqual(scene.roots, [unrelated, target])
        self.assertIs(scene.cores[0].spec[1], source)
        self.assertIs(scene.cores[0].spec[2], target)
        self.assertEqual(scene.cores[0].events, [("begin",), ("interpolate", 0.), ("update", .1), ("interpolate", .5), ("finish",), ("cleanup",)])
        self.assertFalse(animation.mobject.animating)

    def test_python_leaf_hooks_not_lowered_to_native(self):
        class Authored(self.g["Transform"]):
            _native_kind = None
        animation = self.Parts(self.square(), self.square(), match_animation=Authored)
        scene = self.g["Scene"]()
        scene.add(animation.mobject)
        animation.begin()
        animation.interpolate(.5)
        animation.finish()
        self.assertEqual(scene.cores, [])
        self.assertIn(("interpolate", .5), animation.animations[0].events)

    def test_nested_factory_composition(self):
        def factory(source, target, **kwargs):
            return self.g["AnimationGroup"](self.g["Transform"](source, target, **kwargs))
        animation = self.Parts(self.square(), self.square(), match_animation=factory)
        scene = self.g["Scene"]()
        scene.add(animation.mobject)
        animation.begin()
        animation.interpolate(.25)
        animation.finish()
        self.assertEqual(len(scene.cores), 1)
        self.assertIn(("interpolate", .25), scene.cores[0].events)

    def test_callback_failure_aborts_native_sibling(self):
        class Broken(self.g["Transform"]):
            _native_kind = None
            def interpolate(self, alpha):
                if alpha > 0:
                    raise ValueError("authored failure")
        a, b, x, y = self.square(), self.circle(), self.square(), self.square()
        animation = self.Parts(self.group(a, b), self.group(x, y), matched_pairs=[(b, y)], mismatch_animation=Broken)
        scene = self.g["Scene"]()
        scene.add(animation.mobject)
        animation.begin()
        with self.assertRaisesRegex(ValueError, "authored failure"):
            animation.interpolate(.5)
        self.assertIn(("abort",), scene.cores[0].events)
        self.assertFalse(animation.mobject.animating)
        self.assertNotIn(animation.target, scene.roots)

    def test_scene_updater_failure_aborts_matching_leaves(self):
        animation = self.Parts(self.square(), self.square())
        scene = self.g["Scene"]()
        scene.fail_updater = True
        with self.assertRaisesRegex(ValueError, "scene updater failure"):
            scene.play(animation)
        self.assertIn(("abort",), scene.cores[0].events)
        self.assertIsNone(animation._matching_scene)
        self.assertNotIn(animation.target, scene.roots)

    def test_cleanup_releases_runtime_drivers(self):
        animation = self.Parts(self.square(), self.square())
        scene = self.g["Scene"]()
        scene.play(animation)
        self.assertIsNone(animation._matching_driver)
        self.assertIsNone(animation._matching_scene)
        self.assertTrue(animation.remover)

    def test_runtime_lag_and_no_double_easing(self):
        animation = self.Parts(self.square(), self.square(), run_time=3, lag_ratio=.2)
        animation._ensure_runtime_defaults()
        self.assertEqual((animation.run_time, animation.lag_ratio), (3, .2))
        self.assertEqual(animation.rate_func(.25), .25)

    def test_custom_rate_and_native_params_are_preserved(self):
        animation = self.Parts(self.square(), self.circle(), rate_func=lambda t: t*t, time_span=(.2, .8), suspend_mobject_updating=True)
        scene = self.g["Scene"]()
        scene.add(animation.mobject)
        animation.begin()
        spec = scene.cores[0].spec
        self.assertEqual(spec[4][15], .25)
        self.assertEqual(spec[6]["time_span"], (.2, .8))
        self.assertTrue(spec[6]["suspend_mobject_updating"])

    def test_invalid_factories_and_pairs_fail(self):
        with self.assertRaises(TypeError):
            self.Parts(self.square(), self.square(), match_animation=0)
        with self.assertRaises(TypeError):
            self.Parts(self.square(), self.square(), matched_pairs=[(1, 2)])
        with self.assertRaisesRegex(TypeError, "return an Animation"):
            self.Parts(self.square(), self.square(), match_animation=lambda *a, **k: None)

    def test_begin_requires_real_scene_binding(self):
        with self.assertRaisesRegex(RuntimeError, "scene-bound"):
            self.Parts(self.square(), self.square()).begin()


if __name__ == "__main__":
    unittest.main(verbosity=2)
