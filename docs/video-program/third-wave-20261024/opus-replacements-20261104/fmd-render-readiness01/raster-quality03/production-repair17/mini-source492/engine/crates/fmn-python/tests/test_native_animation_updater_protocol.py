"""Production native-driver lowering and updater ownership, with a core double.

This suite does not execute Rust, render frames, or prove native arithmetic.
"""
import ast
import types
import unittest
from pathlib import Path

from test_animation_updater_protocol import adapter, environment

ROOT = Path(__file__).resolve().parents[1]


def native_environment():
    native = environment()
    g = vars(native)
    class NativeOnly(native.Animation):
        _native_kind = "v_show_passing_flash"
        _target_attr = "path"
        def _native_target(self):
            return getattr(self, "path", None)
        def _native_params(self):
            return {"time_width": .3}
    class CameraFrame(native.Mobject):
        pass
    native.NativeOnly, native.CameraFrame = NativeOnly, CameraFrame
    native.Mobject._is_bound = lambda self:self._scene is not None
    def clear(self, recurse=True):
        for member in self.get_family() if recurse else [self]:
            member.updaters.clear()
        return self
    native.Mobject.clear_updaters = clear
    class Core:
        def __init__(self, spec):
            self.spec = spec
            self.anchor = spec[1]
            self.events = []
            self.initial = self.anchor.value
            self.duration = max(spec[3], spec[6].get("time_span", (0., 0.))[1])
            self.fail = None
        def event(self, name, *values):
            self.events.append((name, *values))
            if self.fail == name:
                raise RuntimeError("native " + name)
        def get_run_time(self):
            return self.duration
        def begin(self):
            self.initial = self.anchor.value
            self.anchor.animating = True
            self.event("begin")
            self.interpolate(0.)
        def interpolate(self, alpha):
            self.event("interpolate", alpha)
            self.anchor.value = self.initial + 4 * alpha
        def update_mobjects(self, dt):
            self.event("helpers", dt)
        def finish(self, scene):
            assert scene is self.anchor._scene
            self.event("finish")
            self.interpolate(1.)
            self.anchor.animating = False
        def clean_up_from_scene(self):
            raise AssertionError("persistent animation must not clean up scene membership")
        def abort(self):
            self.anchor.animating = False
            self.event("abort")
    class Scene:
        def __init__(self):
            self.roots, self.cores, self.adoptions = [], [], []
            self.fail = None
        def _adopt(self, mob):
            self.adoptions.append(mob)
            for member in mob.get_family():
                if member._scene is not None and member._scene is not self:
                    raise ValueError("foreign scene")
                member._scene = self
        def add(self, mob):
            self._adopt(mob)
            self.roots.append(mob)
        def _native_animation_driver(self, spec):
            if self.fail == "lower":
                raise RuntimeError("native lower")
            core = Core(spec)
            core.fail = self.fail
            self.cores.append(core)
            return core
    native.Scene = Scene
    path = ROOT / "python/manimlib_bootstrap.py"
    tree = ast.parse(path.read_text())
    leaf = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == "_NativeCompositionLeaf")
    exec(compile(ast.Module([leaf], []), str(path), "exec"), g)
    g["NativeLeaf"] = g["_NativeCompositionLeaf"]
    path = ROOT / "python/manimlib/_animation_semantics.py"
    tree = ast.parse(path.read_text())
    installer = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "_install_matching_parts")
    for name in ("native_rate", "PythonLeaf", "make_driver"):
        node = next(n for n in installer.body if isinstance(n, (ast.ClassDef, ast.FunctionDef)) and n.name == name)
        exec(compile(ast.Module([node], []), str(path), "exec"), g)
    g["_RATE_FUNC_NAMES"] = {g["smooth"]:"linear"}
    g["_composition_member_run_time"] = lambda a:a.get_run_time()
    g["_requires_python_animation"] = lambda a:not isinstance(a, NativeOnly)
    g["_fmn_make_animation_driver"] = g["make_driver"]
    adapter.install_animation_updaters(native)
    return native


class NativePersistentTests(unittest.TestCase):
    def setUp(self):
        self.native = native_environment()
        self.scene = self.native.Scene()
        self.mob = self.native.Mobject(2.)
        self.scene.add(self.mob)
    def attach(self, cycle=False, **kwargs):
        animation = self.native.NativeOnly(self.mob, **kwargs)
        result = self.native.turn_animation_into_updater(animation, cycle=cycle)
        self.assertIs(result, self.mob)
        return animation, self.scene.cores[-1]
    def test_uses_actual_native_factory_and_leaf_wrapper(self):
        animation, core = self.attach()
        self.assertEqual(core.spec[0], "v_show_passing_flash")
        self.assertIs(core.spec[1], self.mob)
        self.mob.update(.5).update(0.)
        self.assertEqual(self.mob.value, 4.)
        self.assertEqual(animation.total_time, .5)
        self.assertEqual(len(self.scene.cores), 1)
    def test_begin_and_registration_order(self):
        _, core = self.attach()
        self.assertEqual(core.events, [("begin",), ("interpolate", 0.), ("interpolate", 0.), ("helpers", 0.)])
    def test_native_parameters_and_effective_duration(self):
        _, core = self.attach(run_time=2., time_span=(1., 3.), lag_ratio=.4, rate_func="linear", suspend_mobject_updating=True)
        self.assertEqual(core.spec[3:6], (2., "linear", .4))
        self.assertFalse(core.spec[6]["suspend_mobject_updating"])
        self.assertEqual(core.spec[6]["time_span"], (1., 3.))
        self.mob.update(2.5).update(0.)
        self.assertTrue(self.mob.updaters)
        self.assertAlmostEqual(core.events[-2][1], 2.5 / 3.)
    def test_finish_once_without_scene_cleanup(self):
        _, core = self.attach(remover=True)
        roots = list(self.scene.roots)
        self.mob.update(1.).update(0.).update(100.)
        self.assertEqual(core.events.count(("finish",)), 1)
        self.assertEqual(self.scene.roots, roots)
        self.assertEqual(self.mob.value, 6.)
        self.assertFalse(self.mob.updaters)
    def test_finish_drops_native_driver_reference(self):
        self.attach()
        owner = self.mob.updaters[0]._fmn_persistent_controller
        self.mob.update(1.).update(0.)
        self.assertTrue(owner.closed)
        self.assertIsNone(owner.driver)
    def test_cycle_wrap_uses_one_retained_driver(self):
        _, core = self.attach(cycle=True)
        self.mob.update(2.25).update(0.)
        self.assertEqual(self.mob.value, 3.)
        self.assertEqual(core.events.count(("begin",)), 1)
        self.assertNotIn(("finish",), core.events)
    def test_negative_cycle_wrap(self):
        self.attach(cycle=True)
        self.mob.update(-.25).update(0.)
        self.assertEqual(self.mob.value, 5.)
    def test_native_lower_failure_has_no_registration(self):
        self.scene.fail = "lower"
        with self.assertRaisesRegex(RuntimeError, "lower"):
            self.attach()
        self.assertFalse(self.mob.updaters)
        self.assertFalse(self.mob.animating)
    def test_native_begin_failure_aborts(self):
        self.scene.fail = "begin"
        with self.assertRaisesRegex(RuntimeError, "begin"):
            self.attach()
        self.assertEqual(self.scene.cores[-1].events.count(("abort",)), 1)
        self.assertFalse(self.mob.animating)
    def test_native_interpolation_failure_aborts_and_detaches(self):
        _, core = self.attach()
        core.fail = "interpolate"
        with self.assertRaisesRegex(RuntimeError, "interpolate"):
            self.mob.update(.2)
        self.assertEqual(core.events.count(("abort",)), 1)
        self.assertFalse(self.mob.updaters)
    def test_native_helper_failure_aborts(self):
        _, core = self.attach()
        core.fail = "helpers"
        with self.assertRaisesRegex(RuntimeError, "helpers"):
            self.mob.update(.2)
        self.assertEqual(core.events.count(("abort",)), 1)
    def test_native_finish_failure_aborts(self):
        _, core = self.attach()
        self.mob.update(1.)
        core.fail = "finish"
        with self.assertRaisesRegex(RuntimeError, "finish"):
            self.mob.update(0.)
        self.assertFalse(self.mob.updaters)
        self.assertFalse(self.mob.animating)
    def test_explicit_remove_aborts_without_final_jump(self):
        _, core = self.attach()
        self.mob.update(.25).update(0.)
        updater, = self.mob.updaters
        self.mob.remove_updater(updater)
        self.assertEqual(self.mob.value, 3.)
        self.assertEqual(core.events.count(("abort",)), 1)
        self.assertNotIn(("finish",), core.events)
        self.assertIsNone(updater._fmn_persistent_controller.driver)
    def test_clear_releases_native_driver(self):
        _, core = self.attach()
        self.mob.clear_updaters()
        self.assertEqual(core.events.count(("abort",)), 1)
        self.assertFalse(self.mob.updaters)
    def test_clear_copy_does_not_abort_original(self):
        animation, core = self.attach()
        clone = self.mob.copy()
        clone.clear_updaters()
        self.assertNotIn(("abort",), core.events)
        self.mob.update(.4)
        self.assertEqual(animation.total_time, .4)
    def test_native_copy_does_not_advance_original(self):
        animation, core = self.attach()
        before = list(core.events)
        self.mob.copy().update(.5)
        self.assertEqual(core.events, before)
        self.assertEqual(animation.total_time, 0.)
    def test_duplicate_registration_is_rejected(self):
        animation, _ = self.attach()
        with self.assertRaisesRegex(RuntimeError, "already"):
            self.native.turn_animation_into_updater(animation)
        self.assertEqual(len(self.scene.cores), 1)
        self.assertEqual(len(self.mob.updaters), 1)
    def test_foreign_target_rejected_before_lowering(self):
        target = self.native.Mobject()
        self.native.Scene().add(target)
        animation = self.native.NativeOnly(self.mob, path=target)
        with self.assertRaisesRegex(ValueError, "multiple Scenes"):
            self.native.turn_animation_into_updater(animation)
        self.assertEqual(self.scene.cores, [])
    def test_foreign_extra_rejected_before_adoption(self):
        target = self.native.Mobject()
        self.native.Scene().add(target)
        animation = self.native.NativeOnly(self.mob, _native_extra_mobjects=(target,))
        with self.assertRaisesRegex(ValueError, "multiple Scenes"):
            self.native.turn_animation_into_updater(animation)
        self.assertEqual(self.scene.cores, [])
    def test_detached_targets_adopted_without_becoming_draw_roots(self):
        target = self.native.Mobject()
        animation = self.native.NativeOnly(self.mob, path=target)
        self.native.turn_animation_into_updater(animation)
        self.assertIs(target._scene, self.scene)
        self.assertEqual(self.scene.roots, [self.mob])
    def test_authored_begin_is_not_silently_discarded(self):
        animation = self.native.NativeOnly(self.mob)
        animation.begin = lambda:None
        with self.assertRaisesRegex(NotImplementedError, "authored lifecycle"):
            self.native.turn_animation_into_updater(animation)
        self.assertEqual(self.scene.cores, [])
    def test_native_nondefault_endpoint_not_silently_discarded(self):
        with self.assertRaisesRegex(NotImplementedError, "final_alpha"):
            self.attach(final_alpha_value=.3)
        self.assertEqual(self.scene.cores, [])
    def test_callbacks_retain_existing_live_lifecycle(self):
        animation = self.native.PerMember(self.mob, final_alpha_value=.3)
        self.native.turn_animation_into_updater(animation)
        self.mob.update(1.).update(0.)
        self.assertEqual(self.mob.value, 2.3)
        self.assertEqual(self.scene.cores, [])
    def test_cancellation_during_interpolation_stops_helper_phase(self):
        _, core = self.attach()
        updater, = self.mob.updaters
        core.interpolate = lambda alpha:self.mob.remove_updater(updater)
        before = len([e for e in core.events if e[0] == "helpers"])
        self.mob.update(.2)
        self.assertEqual(len([e for e in core.events if e[0] == "helpers"]), before)
    def test_preexisting_suspension_survives_abort(self):
        child = self.native.Mobject()
        child.suspended = True
        self.mob.submobjects.append(child)
        self.attach()
        self.mob.clear_updaters()
        self.assertTrue(child.suspended)
    def test_cleanup_error_keeps_original_failure(self):
        _, core = self.attach()
        original = RuntimeError("original")
        core.interpolate = lambda alpha:(_ for _ in ()).throw(original)
        core.abort = lambda:(_ for _ in ()).throw(ValueError("cleanup"))
        with self.assertRaises(RuntimeError) as caught:
            self.mob.update(.1)
        self.assertIs(caught.exception, original)
        self.assertTrue(original.__notes__)
        self.assertFalse(self.mob.updaters)
    def test_manual_cleanup_error_still_detaches(self):
        _, core = self.attach()
        core.abort = lambda:(_ for _ in ()).throw(ValueError("cleanup"))
        with self.assertRaisesRegex(ValueError, "cleanup"):
            self.mob.clear_updaters()
        self.assertFalse(self.mob.updaters)


if __name__ == "__main__":
    unittest.main()
