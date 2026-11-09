"""Source-level dispatch tests with fixture storage, not native acceptance."""
import ast
import importlib.util
from pathlib import Path
from types import MethodType
import unittest

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "python" / "manimlib" / "_animation_semantics.py"
BOOTSTRAP = ROOT / "python" / "manimlib_bootstrap.py"
spec = importlib.util.spec_from_file_location("composition_semantics_under_test", SOURCE)
semantics = importlib.util.module_from_spec(spec)
spec.loader.exec_module(semantics)


def environment():
    class Mobject:
        def __init__(self, *children):
            self.children = list(children)
            self._scene = None
            self.animating = False
            self.suspended = False
            self.x = 0.
        def get_family(self):
            result = [self]
            for child in self.children:
                for member in child.get_family():
                    if member not in result:
                        result.append(member)
            return result
        def _is_bound(self):
            return self._scene is not None
        def set_animating_status(self, value):
            self.animating = value
        def _is_updating_suspended(self):
            return self.suspended
        def suspend_updating(self):
            self.suspended = True
        def resume_updating(self):
            self.suspended = False
        def unlock_data(self):
            self.unlocked = True
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
        def __init__(self, mobject=None, run_time=1., rate_func=None, lag_ratio=0., **kwargs):
            self.mobject, self.run_time, self.rate_func, self.lag_ratio = mobject, run_time, rate_func, lag_ratio
            self.time_span = kwargs.get("time_span")
            self.final_alpha_value = kwargs.get("final_alpha_value", 1.)
            self.suspend_mobject_updating = kwargs.get("suspend_mobject_updating", False)
            self.remover = kwargs.get("remover", False)
            self.events = []
        def get_run_time(self):
            duration = 1. if self.run_time is None else self.run_time
            return max(duration, self.time_span[1]) if self.time_span else duration
        def _ensure_runtime_defaults(self):
            if self.run_time is None:
                self.run_time = 1.
            if self.rate_func is None:
                self.rate_func = lambda t: t*t
            if self.lag_ratio is None:
                self.lag_ratio = 0.
        def _native_params(self):
            return {}
        def _native_target(self):
            return getattr(self, "target_mobject", None)
        def begin(self):
            self.events.append(("begin", self.mobject.x))
            self.start = self.mobject.x
            self.mobject.set_animating_status(True)
        def interpolate(self, alpha):
            self.events.append(("interpolate", alpha))
            self.mobject.x = self.start + alpha
        def interpolate_mobject(self, alpha):
            pass
        def interpolate_submobject(self, *args):
            pass
        def update_mobjects(self, dt):
            self.events.append(("update", dt))
        def finish(self):
            self.events.append(("finish",))
            self.interpolate(self.final_alpha_value)
            self.mobject.set_animating_status(False)
        def clean_up_from_scene(self, scene):
            self.events.append(("cleanup",))
            if self.remover:
                scene.remove(self.mobject)
    class NativeAnimation(Animation):
        pass
    class Transform(NativeAnimation):
        _native_kind = "transform"
        _target_attr = "target_mobject"
        def __init__(self, mobject, target=None, **kwargs):
            super().__init__(mobject, **kwargs)
            self.target_mobject = target or Mobject()
    class TransformMatchingParts(NativeAnimation):
        pass
    class TransformMatchingShapes(TransformMatchingParts):
        pass
    class _AnimationBuilder:
        def __init__(self, animation):
            self.animation = animation
        def build(self):
            return self.animation
    def prepare_animation(animation):
        if isinstance(animation, _AnimationBuilder):
            return animation.build()
        if isinstance(animation, Animation):
            return animation
        raise TypeError("Expected Animation")
    def intervals(durations, lag):
        rows, start = [], 0.
        for duration in durations:
            rows.append((start, start + duration))
            start += duration * lag
        return rows
    class Core:
        def __init__(self, spec):
            self.spec, self.events = spec, []
            self.begun = self.finished = False
        def get_run_time(self):
            return 1. if self.spec[3] is None else self.spec[3]
        def begin(self):
            self.begun = True
            self.start = self.spec[1].x
            self.events.append(("begin", self.start))
            self.spec[1].set_animating_status(True)
        def update_mobjects(self, dt):
            self.events.append(("update", dt))
        def interpolate(self, alpha):
            self.events.append(("interpolate", alpha))
            self.spec[1].x = self.start + alpha
        def finish(self, scene):
            self.interpolate(1.)
            self.finished = True
            self.events.append(("finish",))
            self.spec[1].set_animating_status(False)
        def clean_up_from_scene(self):
            self.events.append(("cleanup",))
        def abort(self):
            if self.begun and not self.finished:
                self.events.append(("abort",))
                self.spec[1].set_animating_status(False)
                self.begun = False
    class SceneCore:
        def __init__(self):
            self.cores, self.roots, self.calls = [], [], []
        def _adopt(self, obj):
            for member in obj.get_family():
                if member._scene is not None and member._scene is not self:
                    raise ValueError("foreign stage")
                member._scene = self
        def add(self, *objects):
            for obj in objects:
                self._adopt(obj)
                if obj not in self.roots:
                    self.roots.append(obj)
        def remove(self, *objects):
            self.roots[:] = [obj for obj in self.roots if obj not in objects]
        def _native_animation_driver(self, spec):
            core = Core(spec)
            self.cores.append(core)
            return core
        def _play_animations(self, specs, callbacks, camera_pair, run_time, rate, lag):
            self.calls.append((specs, callbacks))
            for callback in callbacks:
                if callback is not None:
                    callback.begin()
            for alpha in (.25, .5, .75, 1.):
                for callback in callbacks:
                    if callback is not None:
                        callback.update_mobjects(.25)
                        callback.interpolate(alpha)
                if getattr(self, "fail_updater", False):
                    raise ValueError("updater failed")
            for callback in callbacks:
                if callback is not None:
                    callback.finish()
                    callback.clean_up_from_scene(self)
    import collections.abc
    from types import SimpleNamespace
    g = dict(locals())
    g.update(_NativeAnimation=NativeAnimation, _SceneCore=SceneCore,
             _BridgeMobject=Mobject, _collections_abc=collections.abc,
             _FMN_ROOT=SimpleNamespace(_composition_intervals=intervals),
             _linear_rate=lambda t: t, _RATE_FUNC_NAMES={},
             _native_scene_shell_factory=None,
             _refuse_unrouted=lambda where, keys: (_ for _ in ()).throw(TypeError(where))
                 if any(value for _, value in keys) else None)
    tree = ast.parse(BOOTSTRAP.read_text())
    names = {"AnimationGroup", "LaggedStart", "Succession", "_requires_python_animation",
             "_python_composition_members", "_composition_member_run_time", "_composition_timings",
             "_composition_timeline_position", "_NativeCompositionLeaf", "_CompositionCallbackDriver"}
    nodes = [node for node in tree.body if isinstance(node, (ast.FunctionDef, ast.ClassDef)) and node.name in names]
    if {node.name for node in nodes} != names:
        raise RuntimeError("Production composition definitions are missing")
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(BOOTSTRAP), "exec"), g)
    scene_class = next(node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == "Scene")
    play = next(node for node in scene_class.body if isinstance(node, ast.FunctionDef) and node.name == "play")
    exec(compile(ast.Module(body=[play], type_ignores=[]), str(BOOTSTRAP), "exec"), g)
    Scene = type("Scene", (SceneCore,), {"play": g.pop("play")})
    g["Scene"] = Scene
    semantics._install_matching_parts(g)
    return g


class CompositionLifecycleTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        self.Group = self.g["AnimationGroup"]
        self.Scene = self.g["Scene"]
        self.Mob = self.g["Mobject"]
        self.Animation = self.g["Animation"]
        self.Transform = self.g["Transform"]
        semantics._install_composition_lifecycle(self.g)
    def authored(self, base=None):
        parent = base or self.Group
        class Authored(parent):
            def begin(self):
                self.seen = ["begin"]
                super().begin()
            def interpolate(self, alpha):
                self.seen.append(alpha)
                super().interpolate(alpha)
            def finish(self):
                super().finish()
                self.seen.append("finish")
        return Authored
    def test_installer_registered(self):
        install = next(n for n in ast.parse(SOURCE.read_text()).body if isinstance(n, ast.FunctionDef) and n.name == "install")
        self.assertTrue(any(isinstance(n, ast.Call) and isinstance(n.func, ast.Name)
                            and n.func.id == "_install_composition_lifecycle" for n in ast.walk(install)))
    def test_unmodified_group_keeps_native_route_and_lazy_root(self):
        group = self.Group(self.Transform(self.Mob()))
        self.assertIsNone(group.mobject)
        self.assertFalse(self.g["_requires_python_animation"](group))
        scene = self.Scene()
        scene.play(group)
        self.assertEqual(scene.calls[0][0][0][0], "animation_group")
        self.assertEqual(scene.calls[0][1], [None])
    def test_authored_group_executes_native_leaves_and_hooks(self):
        mob = self.Mob()
        group = self.authored()(self.Transform(mob))
        scene = self.Scene()
        scene.play(group)
        self.assertEqual(group.seen, ["begin", 0., .25, .5, .75, 1., "finish"])
        self.assertEqual(mob.x, 1.)
        self.assertEqual(len(scene.cores), 1)
        self.assertEqual(scene.cores[0].events[0], ("begin", 0.))
        self.assertEqual(scene.cores[0].events[-1], ("cleanup",))
        self.assertIsNone(group._composition_driver)
        self.assertIsNone(group._composition_scene)
    def test_callback_defaults_linear_not_smooth(self):
        leaf = self.Animation(self.Mob())
        group = self.authored()(leaf)
        self.Scene().play(group)
        self.assertIn(("interpolate", .25), leaf.events)
    def test_native_and_python_members_share_unequal_windows(self):
        native = self.Transform(self.Mob(), run_time=2.)
        python = self.Animation(self.Mob(), run_time=1.)
        group = self.authored()(native, python, run_time=8.)
        scene = self.Scene()
        scene.play(group)
        self.assertIn(("interpolate", .25), scene.cores[0].events)
        self.assertIn(("interpolate", .5), python.events)
    def test_succession_begins_from_predecessor_final_state(self):
        mob = self.Mob()
        first, second = self.Animation(mob), self.Transform(mob)
        group = self.authored(self.g["Succession"])(first, second)
        scene = self.Scene()
        scene.play(group)
        self.assertEqual(scene.cores[0].events[0], ("begin", 1.))
        self.assertEqual(mob.x, 2.)
    def test_nested_custom_group_hooks_not_flattened(self):
        child = self.authored()(self.Transform(self.Mob()))
        outer = self.Group(child, self.Transform(self.Mob()))
        scene = self.Scene()
        scene.play(outer)
        self.assertEqual(child.seen[0], "begin")
        self.assertEqual(child.seen[-1], "finish")
        self.assertIsNone(child._composition_driver)
    def test_late_instance_hook_is_detected(self):
        group = self.Group(self.Transform(self.Mob()))
        seen = []
        def begin(instance):
            seen.append("called")
            self.Group.begin(instance)
        group.begin = MethodType(begin, group)
        self.assertTrue(self.g["_requires_python_animation"](group))
        self.Scene().play(group)
        self.assertEqual(seen, ["called"])
    def test_late_class_hook_is_detected(self):
        seen = []
        original = self.Group.begin
        def begin(group):
            seen.append(group)
            original(group)
        self.Group.begin = begin
        group = self.Group(self.Transform(self.Mob()))
        self.Scene().play(group)
        self.assertEqual(seen, [group])
    def test_direct_bound_group_lifecycle(self):
        mob = self.Mob()
        scene = self.Scene()
        scene.add(mob)
        group = self.Group(self.Transform(mob))
        group.begin()
        group.interpolate(.5)
        self.assertEqual(mob.x, .5)
        group.finish()
        group.clean_up_from_scene(scene)
        self.assertEqual(mob.x, 1.)
    def test_unbound_direct_begin_names_requirement(self):
        group = self.Group(self.Transform(self.Mob()))
        with self.assertRaisesRegex(RuntimeError, "scene-bound"):
            group.begin()
    def test_presuspended_root_stays_suspended(self):
        mob = self.Mob()
        group = self.authored()(self.Animation(mob), suspend_mobject_updating=True)
        root = group.get_all_mobjects()
        root.suspend_updating()
        self.Scene().play(group)
        self.assertTrue(root.suspended)
    def test_group_suspension_is_released(self):
        group = self.authored()(self.Animation(self.Mob()), suspend_mobject_updating=True)
        self.Scene().play(group)
        self.assertFalse(group.mobject.suspended)
    def test_updater_failure_unwinds_native_and_python_leaves(self):
        a, b = self.Mob(), self.Mob()
        group = self.authored()(self.Transform(a), self.Animation(b), suspend_mobject_updating=True)
        scene = self.Scene()
        scene.fail_updater = True
        with self.assertRaisesRegex(ValueError, "updater failed"):
            scene.play(group)
        self.assertIn(("abort",), scene.cores[0].events)
        self.assertFalse(a.animating or b.animating or group.mobject.animating or group.mobject.suspended)
        self.assertIsNone(group._composition_driver)
        self.assertIsNone(group._composition_scene)
    def test_authored_error_after_super_begin_unwinds(self):
        parent = self.Group
        class Broken(parent):
            def begin(group):
                super().begin()
                raise ValueError("outside super")
        mob = self.Mob()
        group = Broken(self.Transform(mob))
        scene = self.Scene()
        with self.assertRaisesRegex(ValueError, "outside super"):
            scene.play(group)
        self.assertFalse(mob.animating)
        self.assertIn(("abort",), scene.cores[0].events)
    def test_abort_is_idempotent_and_rebegin_restarts_driver(self):
        group = self.authored()(self.Transform(self.Mob()))
        scene = self.Scene()
        scene.add(group.get_all_mobjects())
        group.begin()
        group.begin()
        self.assertEqual(len(scene.cores), 2)
        self.assertIn(("abort",), scene.cores[0].events)
        group.abort()
        group.abort()
        self.assertEqual(scene.cores[1].events.count(("abort",)), 1)
    def test_cycle_is_rejected_before_scene_mutation(self):
        group = self.Group(self.Transform(self.Mob()))
        group.animations.append(group)
        scene = self.Scene()
        with self.assertRaisesRegex(ValueError, "cycle"):
            scene.play(group)
        self.assertEqual(scene.roots, [])
    def test_root_identity_deduplication(self):
        mob = self.Mob()
        group = self.authored()(self.Transform(mob), self.Animation(mob))
        root = group.get_all_mobjects()
        self.assertEqual(root.children, [mob])
        self.assertIs(root, group.get_all_mobjects())
    def test_child_final_alpha_is_preserved(self):
        leaf = self.Animation(self.Mob(), final_alpha_value=.3)
        self.Scene().play(self.authored()(leaf, final_alpha_value=.7))
        self.assertEqual(leaf.mobject.x, .3)


if __name__ == "__main__":
    unittest.main()
