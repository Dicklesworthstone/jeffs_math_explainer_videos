"""Install production protocols with fixture storage, native kernels and clock."""
import ast
import collections.abc
import copy
import math
import os
from fractions import Fraction
from pathlib import Path
from types import SimpleNamespace
import unittest
import numpy as np
from test_camera_pose_protocol import Core, semantics, SOURCE

BOOTSTRAP = Path(os.environ.get("FMN_BOOTSTRAP_SOURCE", str(SOURCE.parents[1] / "manimlib_bootstrap.py")))
NAMES = {"AnimationGroup", "LaggedStart", "Succession", "_requires_python_animation",
         "_composition_member_run_time", "_composition_timings", "_composition_timeline_position",
         "_NativeCompositionLeaf", "_CompositionCallbackDriver"}


def environment():
    def linear(t):
        return t
    def smooth(t):
        return t * t * (3. - 2. * t)
    class Mobject:
        def __init__(self, *children):
            self.submobjects = list(children)
            self.data = np.zeros(1, dtype=[("point", float, (3,))])
            self.uniforms = {"opacity": 1.}
            self.locked_data_keys, self.locked_uniform_keys, self.const_data_keys = set(), set(), set()
            self.pointlike_data_keys = {"point"}
            self._scene, self.updaters = None, []
            self.animating, self.suspended = False, False
        def copy(self):
            result = type(self)()
            result.submobjects = [child.copy() for child in self.submobjects]
            result.data = self.data.copy()
            result.uniforms = dict(self.uniforms)
            result.updaters = list(self.updaters)
            if hasattr(self, "_core"):
                result._core = copy.copy(self._core)
            return result
        def get_family(self):
            result = [self]
            for child in self.submobjects:
                result.extend(member for member in child.get_family() if member not in result)
            return result
        def family_members_with_points(self):
            return [member for member in self.get_family() if len(member.data)]
        def has_same_shape_as(self, other):
            return self.data.shape == other.data.shape
        def get_center(self):
            return self.data["point"][0].copy() if len(self.data) else np.zeros(3)
        def shift(self, vector):
            self.data["point"] += vector
            return self
        def _is_bound(self):
            return self._scene is not None
        def note_changed_data(self):
            pass
        def set_animating_status(self, value):
            self.animating = value
        def is_aligned_with(self, other):
            return self.data.shape == other.data.shape
        def align_data_and_family(self, other):
            assert self.is_aligned_with(other)
        def lock_matching_data(self, start, end):
            if np.array_equal(start.data, end.data):
                self.locked_data_keys.add("point")
        def unlock_data(self):
            self.locked_data_keys.clear()
        def has_updaters(self):
            return any(member.updaters for member in self.get_family())
        def _is_updating_suspended(self):
            return self.suspended
        def suspend_updating(self):
            self.suspended = True
        def resume_updating(self):
            self.suspended = False
            self.update(0.)
        def update(self, dt):
            if not self.suspended:
                for child in self.submobjects:
                    child.update(dt)
                for updater in self.updaters[:]:
                    self._dispatch_updater(updater, dt)
        def _dispatch_updater(self, updater, dt):
            updater(self, dt)
    class VMobject(Mobject):
        pass
    class Group(Mobject):
        def __init__(self, *children):
            super().__init__(*children)
            self.data = self.data[:0]
    class VGroup(Group, VMobject):
        pass
    class CameraFrame(Mobject):
        def __init__(self):
            super().__init__()
            self.data = self.data[:0]
            self._core = Core()
        def shift(self, vector):
            self._core.set_center(np.array(self._core.center()) + vector)
            return self
        def get_center(self):
            return np.array(self._core.center())
    class Animation:
        _native_kind = None
    class NativeAnimation(Animation):
        _target_attr = None
        def _native_params(self):
            return {"final_alpha_value": self.final_alpha_value}
        def _native_target(self):
            return getattr(self, "target_mobject", None)
    class Transform(NativeAnimation):
        _native_kind = "transform"
        _target_attr = "target_mobject"
        def __init__(self, mobject, target_mobject=None, path_arc=0., path_arc_axis=(0, 0, 1), path_func=None, **kwargs):
            super().__init__(mobject, **kwargs)
            self.target_mobject, self.path_arc, self.path_arc_axis, self.path_func = target_mobject, path_arc, path_arc_axis, path_func
    class ReplacementTransform(Transform):
        pass
    class TransformFromCopy(Transform):
        pass
    class DrawBorderThenFill(NativeAnimation):
        pass
    class FadeTransform(Transform):
        pass
    class FadeTransformPieces(FadeTransform):
        pass
    class TransformMatchingParts(NativeAnimation):
        pass
    class TransformMatchingShapes(TransformMatchingParts):
        pass
    class TransformMatchingStrings(NativeAnimation):
        pass
    class TransformMatchingTex(TransformMatchingStrings):
        pass
    class StringMobject(VMobject):
        pass
    class Builder:
        def __init__(self, source, target, **kwargs):
            self.mobject, self.target, self.kwargs = source, target, kwargs
            self.overridden_animation = None
            self.calls = 0
        def build(self):
            self.calls += 1
            return self.overridden_animation if self.overridden_animation is not None else Transform(self.mobject, self.target, **self.kwargs)
    def prepare_animation(animation):
        if isinstance(animation, Builder):
            return animation.build()
        if not isinstance(animation, Animation):
            raise TypeError("not an Animation")
        return animation
    def refuse(where, entries):
        if any(active for _, active in entries):
            raise TypeError(where + " unsupported args")
    def intervals(durations, lag):
        output, time = [], 0.
        for duration in durations:
            output.append((time, time + duration))
            time += lag * duration
        return output
    class NativeCore:
        def __init__(self, scene, spec):
            self.scene, self.spec = scene, spec
            self.begun, self.finished = False, False
        def get_run_time(self):
            duration = 1. if self.spec[3] is None else self.spec[3]
            return max(duration, self.spec[6].get("time_span", (0., 0.))[1])
        def begin(self):
            self.begun = True
            self.start = self.spec[1].copy()
            self.scene.events.append(("native.begin", self.spec[1]))
        def update_mobjects(self, dt):
            self.scene.events.append(("native.update", dt))
        def interpolate(self, alpha):
            self.scene.events.append(("native.interpolate", alpha))
            start, target = self.start, self.spec[2]
            alpha = min(max(alpha, 0.), 1.)
            rate = self.spec[4]
            if rate == "smooth":
                alpha = smooth(alpha)
            if target is not None:
                self.spec[1].data["point"] = (1. - alpha) * start.data["point"] + alpha * target.data["point"]
        def finish(self, scene):
            if self.begun and not self.finished:
                self.interpolate(self.spec[6].get("final_alpha_value", 1.))
                self.finished = True
                self.scene.events.append(("native.finish", self.spec[1]))
        def clean_up_from_scene(self):
            self.scene.events.append(("native.cleanup", self.spec[1]))
        def abort(self):
            if self.begun and not self.finished:
                self.finished = True
                self.scene.events.append(("native.abort", self.spec[1]))
    class Scene:
        def __init__(self, fps=4):
            self.frame, self.roots, self.cores, self.events = CameraFrame(), [], [], []
            self.samples, self.calls, self.fps, self.elapsed = [], [], fps, 0.
            self.updater = None
        def _adopt(self, root):
            for member in root.get_family():
                if isinstance(member, CameraFrame):
                    raise AssertionError("camera adopted as drawable")
                if member._is_bound() and member._scene is not self:
                    raise ValueError("foreign scene")
                member._scene = self
        def add(self, *roots):
            for root in roots:
                self._adopt(root)
                if root not in self.roots:
                    self.roots.append(root)
        def remove(self, root, *more):
            for value in (root, *more):
                if value in self.roots:
                    self.roots.remove(value)
        def get_mobject_family_members(self):
            # Scene.get_mobject_family_members: scene_play keeps roots that
            # are already present in draw order (fm-kpvx).
            return [member for root in self.roots for member in root.get_family()]
        def _native_animation_driver(self, spec):
            core = NativeCore(self, spec)
            self.cores.append(core)
            return core
        def play(self, *args, **kwargs):
            self.calls.append(("original", args, kwargs))
            return "original"
        def _play_animations(self, specs, callbacks, camera, run_time, rate_func, lag_ratio):
            self.calls.append(("native_clock", specs, callbacks, camera, run_time, rate_func, lag_ratio))
            assert len(specs) == len(callbacks) == 1
            assert camera is None
            duration, callback = specs[0][3], callbacks[0]
            callback.begin()
            if getattr(self, "fail_after_begin", False):
                raise RuntimeError("native begin failed")
            self.add(specs[0][1])
            for frame in range(1, math.ceil(Fraction.from_float(duration) * self.fps) + 1):
                dt = 1. / self.fps
                callback.update_mobjects(dt)
                callback.interpolate((frame / self.fps) / duration)
                self.elapsed += dt
                for updater in self.frame.updaters[:]:
                    self.frame._dispatch_updater(updater, dt)
                if self.updater is not None:
                    self.updater(self, dt)
                self.samples.append((self.elapsed, self.frame.get_center().copy(), list(self.roots)))
            callback.finish()
            callback.clean_up_from_scene(self)
            for updater in self.frame.updaters[:]:
                self.frame._dispatch_updater(updater, 0.)
            return self.samples
    g = dict(locals())
    g.update(_NativeAnimation=NativeAnimation, _AnimationBuilder=Builder, _SceneCore=Scene, _np=np, _copy=copy,
             _OUT=np.array([0., 0., 1.]), _refuse_unrouted=refuse, _smooth_rate=smooth,
             _linear_rate=linear, _interpolate=lambda a, b, t: (1-t)*a+t*b,
             straight_path=lambda a, b, t: (1-t)*a+t*b, _collections_abc=collections.abc,
             _RATE_FUNC_NAMES={linear: "linear", smooth: "smooth"}, _TexError=ValueError,
             _FMN_ROOT=SimpleNamespace(_composition_intervals=intervals))
    tree = ast.parse(BOOTSTRAP.read_text())
    definitions = [node for node in tree.body if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name in NAMES]
    assert {node.name for node in definitions} == NAMES
    exec(compile(ast.Module(body=definitions, type_ignores=[]), str(BOOTSTRAP), "exec"), g)
    module = SimpleNamespace(**g)
    semantics.install(module)
    return vars(module)


class CameraChoreographyTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        self.scene = self.g["Scene"]()
        self.Frame = self.g["CameraFrame"]
        self.Transform = self.g["Transform"]
        self.Group = self.g["AnimationGroup"]
        self.Succession = self.g["Succession"]
    def camera_move(self, x, **kwargs):
        kwargs.setdefault("rate_func", self.g["linear"])
        return self.Transform(self.scene.frame, self.scene.frame.copy().shift((x, 0, 0)), **kwargs)
    def xs(self):
        return [float(sample[1][0]) for sample in self.scene.samples]
    def test_camera_free_play_is_untouched(self):
        obj = self.g["Mobject"]()
        animation = self.Transform(obj, obj.copy())
        self.assertEqual(self.scene.play(animation), "original")
        self.assertIs(self.scene.calls[0][1][0], animation)
    def test_top_level_camera_duration_uses_native_clock(self):
        animation = self.camera_move(4, run_time=2)
        core = self.scene.frame._core
        self.scene.play(animation)
        self.assertEqual(len(self.scene.calls), 1)
        self.assertEqual(self.scene.calls[0][0], "native_clock")
        self.assertEqual(self.scene.calls[0][1][0][3], 2)
        self.assertEqual(self.xs(), [.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.])
        self.assertIs(self.scene.frame._core, core)
        self.assertFalse(self.scene.frame._is_bound())
        self.assertEqual(self.scene.roots, [])
        self.assertTrue(all(not roots for _, _, roots in self.scene.samples))
    def test_binary_float_runtime_uses_exact_clock_coverage(self):
        self.scene = self.g["Scene"](fps=30)
        self.scene.play(self.camera_move(4, run_time=.1))
        self.assertEqual(len(self.scene.samples), 4)
        np.testing.assert_allclose(self.xs(), [4/3, 8/3, 4., 4.])
    def test_builder_is_prepared_once_with_its_own_options(self):
        builder = self.g["Builder"](self.scene.frame, self.scene.frame.copy().shift((2, 0, 0)), run_time=.5, rate_func=self.g["linear"])
        self.scene.play(builder)
        self.assertEqual(builder.calls, 1)
        self.assertEqual(self.xs(), [1., 2.])
    def test_authored_camera_path_receives_each_capture_alpha(self):
        alphas = []
        def path(a, b, alpha):
            alphas.append(alpha)
            return (1-alpha)*a+alpha*b+[0, 4*alpha*(1-alpha), 0]
        self.scene.play(self.camera_move(0, run_time=1, path_func=path))
        np.testing.assert_allclose([p[1][1] for p in self.scene.samples], [.75, 1., .75, 0.])
        self.assertEqual(alphas, [0., .25, .5, .75, 1., 1.])
    def test_leaf_rate_curve_and_final_alpha_are_not_overwritten(self):
        animation = self.camera_move(4, run_time=1, rate_func=lambda t: t*t, final_alpha_value=.5)
        self.scene.play(animation)
        self.assertEqual(self.xs(), [.25, 1., 2.25, 4.])
        np.testing.assert_allclose(self.scene.frame.get_center(), [1., 0., 0.])
    def test_unequal_camera_succession_begins_just_in_time(self):
        first = self.camera_move(1, run_time=.5)
        second = self.camera_move(3, run_time=1.)
        self.scene.play(self.Succession(first, second))
        np.testing.assert_allclose(self.xs(), [.5, 1., 1.5, 2., 2.5, 3.])
        np.testing.assert_allclose(second.starting_mobject.get_center(), [1., 0., 0.])
        self.assertEqual(self.scene.roots, [])
    def test_nested_groups_share_the_same_frame_grid(self):
        group = self.Group(self.Succession(self.camera_move(1, run_time=.5), self.camera_move(3, run_time=.5)), run_time=2)
        self.scene.play(group)
        np.testing.assert_allclose(self.xs(), [.25, .5, .75, 1., 1.5, 2., 2.5, 3.])
    def test_native_drawable_and_camera_preserve_frame_order(self):
        obj = self.g["Mobject"]()
        native = self.Transform(obj, obj.copy().shift((4, 0, 0)), run_time=1, rate_func=self.g["linear"])
        observed = []
        Base = self.Transform
        class WatchingCamera(Base):
            def update_mobjects(self, dt):
                observed.append(float(obj.get_center()[0]))
                super().update_mobjects(dt)
        camera = WatchingCamera(self.scene.frame, self.scene.frame.copy().shift((4, 0, 0)), run_time=1, rate_func=self.g["linear"])
        self.scene.play(native, camera)
        self.assertEqual(observed, [1., 2., 3., 4.])
        self.assertEqual(len(self.scene.cores), 1)
        self.assertEqual(self.scene.roots, [obj])
    def test_reversed_native_camera_order_observes_previous_state(self):
        obj = self.g["Mobject"]()
        observed = []
        def path(a, b, t):
            if t not in (0., 1.):
                observed.append(float(obj.get_center()[0]))
            return (1-t)*a+t*b
        self.scene.play(self.camera_move(4, run_time=1, path_func=path), self.Transform(obj, obj.copy().shift((4, 0, 0)), run_time=1))
        self.assertEqual(observed, [0., 1., 2.])
    def test_multiple_camera_animations_keep_argument_order(self):
        self.scene.play(self.camera_move(8, run_time=1), self.camera_move(4, run_time=1))
        self.assertEqual(self.xs(), [1., 2., 3., 4.])
    def test_short_camera_finishes_on_its_own_timeline(self):
        obj = self.g["Mobject"]()
        self.scene.play(self.camera_move(2, run_time=.5), self.Transform(obj, obj.copy().shift((4, 0, 0)), run_time=1))
        self.assertEqual(self.xs(), [1., 2., 2., 2.])
    def test_play_overrides_apply_to_top_level_camera(self):
        self.scene.play(self.camera_move(4, run_time=3), run_time=1, rate_func=lambda t: t*t)
        self.assertEqual(self.xs(), [.25, 1., 2.25, 4.])
    def test_time_span_is_applied_by_the_camera_animation(self):
        self.scene.play(self.camera_move(4, run_time=1, time_span=(.5, 1.)))
        self.assertEqual(self.xs(), [0., 0., 2., 4.])
    def test_suspended_camera_updaters_resume_only_owned_suspension(self):
        ticks = []
        self.scene.frame.updaters.append(lambda frame, dt: ticks.append(dt) if frame is self.scene.frame else None)
        self.scene.play(self.camera_move(4, run_time=1, suspend_mobject_updating=True))
        self.assertTrue(ticks)
        self.assertTrue(all(dt == 0 for dt in ticks))
        self.assertFalse(self.scene.frame.suspended)
    def test_group_suspension_includes_non_drawable_camera(self):
        ticks = []
        self.scene.frame.updaters.append(lambda frame, dt: ticks.append(dt) if frame is self.scene.frame else None)
        self.scene.play(self.Group(self.camera_move(4, run_time=1), suspend_mobject_updating=True))
        self.assertTrue(all(dt == 0 for dt in ticks))
        self.assertFalse(self.scene.frame.suspended)
    def test_preexisting_camera_suspension_is_retained(self):
        self.scene.frame.suspend_updating()
        ticks = []
        self.scene.frame.updaters.append(lambda frame, dt: ticks.append(dt) if frame is self.scene.frame else None)
        self.scene.play(self.Group(self.camera_move(4, run_time=1), suspend_mobject_updating=True))
        self.assertTrue(self.scene.frame.suspended)
        self.assertEqual(ticks, [])
    def test_authored_timing_rows_really_delay_camera_motion(self):
        group = self.Group(self.camera_move(4, run_time=1), run_time=1)
        group.anims_with_timings = [(group.animations[0], .5, 1.)]
        group.max_end_time = 1.
        self.scene.play(group)
        self.assertEqual(self.xs(), [0., 0., 2., 4.])
    def test_group_subclass_hooks_keep_original_camera_child_identity(self):
        Base = self.Group
        calls = []
        class Authored(Base):
            def begin(self):
                calls.append(self.animations[0])
                super().begin()
        animation = self.camera_move(4, run_time=1)
        group = Authored(animation)
        self.scene.play(group)
        self.assertEqual(calls, [animation])
        self.assertIs(group.animations[0].mobject, self.scene.frame)
    def test_camera_path_exception_aborts_native_sibling(self):
        obj = self.g["Mobject"]()
        failure = RuntimeError("path failed")
        def path(a, b, t):
            if t > 0:
                raise failure
            return a
        camera = self.camera_move(4, path_func=path, suspend_mobject_updating=True)
        with self.assertRaises(RuntimeError) as caught:
            self.scene.play(self.Transform(obj, obj.copy().shift((4, 0, 0))), camera)
        self.assertIs(caught.exception, failure)
        self.assertTrue(self.scene.cores[0].finished)
        self.assertFalse(self.scene.frame.animating)
        self.assertFalse(self.scene.frame.suspended)
        self.assertEqual(self.scene.roots, [obj])
    def test_scene_updater_exception_aborts_nested_camera_group(self):
        failure = RuntimeError("scene updater failed")
        self.scene.updater = lambda *args: (_ for _ in ()).throw(failure)
        group = self.Group(self.camera_move(4, suspend_mobject_updating=True))
        with self.assertRaises(RuntimeError) as caught:
            self.scene.play(group)
        self.assertIs(caught.exception, failure)
        self.assertFalse(self.scene.frame.suspended)
        self.assertFalse(group.mobject.animating)
        self.assertIsNone(group._composition_driver)
        self.assertIsNone(group._composition_scene)
    def test_native_prologue_failure_aborts_already_begun_camera(self):
        self.scene.fail_after_begin = True
        with self.assertRaisesRegex(RuntimeError, "native begin failed"):
            self.scene.play(self.camera_move(4, suspend_mobject_updating=True))
        self.assertFalse(self.scene.frame.suspended)
        self.assertFalse(self.scene.frame.animating)
        self.assertFalse(self.scene._fmn_camera_play_active)
    def test_wrong_scene_camera_is_rejected_before_mutation(self):
        other = self.Frame()
        with self.assertRaisesRegex(ValueError, "this Scene.frame"):
            self.scene.play(self.Transform(other, other.copy()))
        self.assertEqual(self.scene.roots, [])
        self.assertEqual(self.scene.calls, [])
    def test_camera_replacement_is_named_not_silently_ignored(self):
        with self.assertRaisesRegex(NotImplementedError, "remove or replace"):
            self.scene.play(self.g["ReplacementTransform"](self.scene.frame, self.scene.frame.copy()))
    def test_nonfinite_runtime_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "finite"):
            self.scene.play(self.camera_move(4), run_time=math.nan)
        self.assertEqual(self.scene.calls, [])
    def test_cycles_are_rejected_even_after_camera_member(self):
        group = self.Group(self.camera_move(4))
        group.animations.append(group)
        with self.assertRaisesRegex(ValueError, "cycle"):
            self.scene.play(group)
        self.assertEqual(self.scene.calls, [])
    def test_private_clock_cleanup_does_not_dispatch_public_remove_override(self):
        self.scene.remove = lambda *args: self.fail("private slot reached public Scene.remove")
        self.scene.play(self.camera_move(4, run_time=1))
        self.assertEqual(self.xs(), [1., 2., 3., 4.])
        self.assertEqual(self.scene.roots, [])
    def test_replay_after_failure_has_no_active_driver_or_clock_leak(self):
        self.scene.fail_after_begin = True
        with self.assertRaises(RuntimeError):
            self.scene.play(self.camera_move(4, suspend_mobject_updating=True))
        self.scene.fail_after_begin = False
        self.scene.play(self.camera_move(2, run_time=.5))
        self.assertEqual(self.xs(), [1., 2.])
        self.assertEqual(self.scene.roots, [])
    def test_stale_ownership_flag_cannot_resume_a_pre_suspended_camera(self):
        Base = self.Transform
        class BadCopy(Base):
            def create_starting_mobject(self):
                raise RuntimeError("copy failed")
        self.scene.frame.suspend_updating()
        animation = BadCopy(self.scene.frame, self.scene.frame.copy(), suspend_mobject_updating=True)
        animation.mobject_was_updating = True
        with self.assertRaisesRegex(RuntimeError, "copy failed"):
            self.scene.play(animation)
        self.assertTrue(self.scene.frame.suspended)
        self.assertFalse(self.scene.frame.animating)
    def test_explicit_drawable_root_is_preserved_with_camera_children(self):
        root = self.g["Mobject"]()
        group = self.Group(self.camera_move(4), group=root)
        self.scene.play(group)
        self.assertIs(group.mobject, root)
        self.assertEqual(self.scene.roots, [root])
        self.assertFalse(self.scene.frame._is_bound())
    def test_camera_in_explicit_drawable_root_is_rejected(self):
        root = self.g["Group"](self.scene.frame)
        group = self.Group(self.camera_move(4), group=root)
        with self.assertRaisesRegex(ValueError, "drawable group cannot contain"):
            self.scene.play(group)
        self.assertFalse(self.scene.frame._is_bound())
        self.assertFalse(self.scene._fmn_camera_play_active)
    def test_native_non_transform_camera_animation_is_named(self):
        Native = self.g["_NativeAnimation"]
        class Unsupported(Native):
            _native_kind = "rotating"
        with self.assertRaisesRegex(NotImplementedError, "no camera-pose animation protocol"):
            self.scene.play(Unsupported(self.scene.frame))
        self.assertEqual(self.scene.calls, [])
    def test_camera_as_native_extra_cannot_sneak_into_the_stage(self):
        obj = self.g["Mobject"]()
        animation = self.Transform(obj, obj.copy())
        animation._native_extra_mobjects = [self.scene.frame]
        with self.assertRaisesRegex(NotImplementedError, "extra drawable"):
            self.scene.play(animation)
        self.assertFalse(self.scene.frame._is_bound())
    def test_top_level_finish_cleanup_order_matches_choreo(self):
        events = []
        Base = self.Transform
        class Mark(Base):
            def finish(self):
                events.append((self.name, "finish"))
                super().finish()
            def clean_up_from_scene(self, scene):
                events.append((self.name, "cleanup"))
                super().clean_up_from_scene(scene)
        a = Mark(self.scene.frame, self.scene.frame.copy(), name="a")
        b = Mark(self.scene.frame, self.scene.frame.copy(), name="b")
        self.scene.play(a, b)
        self.assertEqual(events, [("a", "finish"), ("a", "cleanup"), ("b", "finish"), ("b", "cleanup")])
    def test_begin_and_helper_hooks_keep_native_camera_identity(self):
        seen = []
        Base = self.Transform
        class Authored(Base):
            def begin(self):
                seen.append(self.mobject)
                super().begin()
            def update_mobjects(self, dt):
                seen.append(self.mobject)
                super().update_mobjects(dt)
        frame = self.scene.frame
        self.scene.play(Authored(frame, frame.copy().shift((4, 0, 0)), run_time=1))
        self.assertEqual(seen, [frame] * 5)

    def test_reentrant_play_is_rejected_and_unwinds(self):
        def path(a, b, t):
            if t > 0:
                self.scene.play(self.camera_move(2))
            return a
        with self.assertRaisesRegex(RuntimeError, "Reentrant"):
            self.scene.play(self.camera_move(4, path_func=path, suspend_mobject_updating=True))
        self.assertFalse(self.scene._fmn_camera_play_active)
        self.assertFalse(self.scene.frame.suspended)


if __name__ == "__main__":
    unittest.main()
