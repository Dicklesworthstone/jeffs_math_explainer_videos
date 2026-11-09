"""Test the shipped installer against real bootstrap classes without CPython FFI.

The bootstrap's animation definitions and timing adapters are compiled unchanged.
Rust storage is replaced by a small NumPy-backed fixture, and the interval builder
by explicit input/output cases that check delegation. These are Python contract
tests, not evidence of Rust timing, native rendering or installed-wheel behavior.
"""
from __future__ import annotations

import abc
import ast
import collections.abc
import copy
import importlib.util
import math
import types
import unittest
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[1]
PYTHON = ROOT / "crates/fmn-python/python"
BOOTSTRAP = PYTHON / "manimlib_bootstrap.py"
INSTALLER = PYTHON / "manimlib/_animation_semantics.py"
ANIMATION_CLASSES = {
    "Animation", "_NativeAnimation", "Transform", "ReplacementTransform",
    "TransformFromCopy", "CyclicReplace", "Swap", "AnimationGroup",
    "LaggedStart", "Succession", "LaggedStartMap",
    "DrawBorderThenFill", "FadeTransform", "FadeTransformPieces",
    "TransformMatchingParts", "TransformMatchingShapes",
    "TransformMatchingStrings", "TransformMatchingTex",
    "ShowPartial", "ShowCreation", "Uncreate", "ShowPassingFlash", "Write",
    "_AnimationBuilder", "_NativeCompositionLeaf", "_CompositionCallbackDriver",
    "Group", "VGroup",
}
ANIMATION_FUNCTIONS = {
    "prepare_animation", "_composition_member_run_time", "_composition_timings",
    "_requires_python_animation", "_linear_rate", "_composition_timeline_position",
    "_vec3", "_refuse_unrouted", "_interpolate", "_smooth_rate",
    "_there_and_back_rate", "_install_rate_functions", "_install_path_functions",
}


def make_native():
    class Mobject:
        pointlike_data_keys = ("point",)
        const_data_keys = ()

        def __init__(self, *children, points=((0.0, 0.0, 0.0),)):
            self.submobjects = list(children)
            self.data = np.zeros(len(points), dtype=[("point", "f8", 3), ("rgba", "f8", 4)])
            self.data["point"] = np.asarray(points).reshape((-1, 3))
            self.uniforms = {"shading": (0.0, 0.0, 0.0)}
            self.locked_data_keys = set()
            self.locked_uniform_keys = set()
            self.animating = False
            self.suspended = False
            self.updaters = []
            self.changes = 0

        def __iter__(self):
            return iter(self.submobjects)

        def copy(self):
            return copy.deepcopy(self)

        def get_family(self):
            return [self, *(descendant for child in self for descendant in child.get_family())]

        def family_members_with_points(self):
            return [member for member in self.get_family() if len(member.data)]

        def set_animating_status(self, value):
            for member in self.get_family():
                member.animating = value

        def _is_updating_suspended(self):
            return self.suspended

        def suspend_updating(self):
            self.suspended = True

        def resume_updating(self):
            self.suspended = False

        def is_aligned_with(self, other):
            return [len(m.data) for m in self.get_family()] == [len(m.data) for m in other.get_family()]

        def align_data_and_family(self, other):
            if not self.is_aligned_with(other):
                raise AssertionError("fixture deliberately does not simulate Rust alignment")

        def has_updaters(self):
            return any(m.updaters for m in self.get_family())

        def lock_matching_data(self, start, target):
            for live, left, right in zip(self.get_family(), start.get_family(), target.get_family()):
                live.locked_data_keys = {
                    key for key in live.data.dtype.names
                    if np.array_equal(left.data[key], right.data[key])
                }

        def unlock_data(self):
            for member in self.get_family():
                member.locked_data_keys.clear()
                member.locked_uniform_keys.clear()

        def note_changed_data(self):
            self.changes += 1

        def update(self, dt):
            if not self.suspended:
                for updater in self.updaters:
                    updater(self, dt)

    def unsupported_native(*args, **kwargs):
        raise AssertionError("fixture does not implement native scene, camera, or geometry storage")

    # These Rust-backed surfaces are needed for installation/type checks,
    # not exercised by this record-interpolation fixture. Never pretend to
    # construct native state or successfully execute a scene/updater.
    class VMobject(Mobject):
        __init__ = unsupported_native
        pointwise_become_partial = unsupported_native

    class Surface(Mobject):
        __init__ = unsupported_native

    class StringMobject(VMobject):
        __init__ = unsupported_native

    class CameraFrame(Mobject):
        __init__ = unsupported_native
        _dispatch_updater = unsupported_native

    class Scene:
        __init__ = unsupported_native
        play = unsupported_native

    modules = {}

    def ensure_module(name):
        # Real utility installers publish into isolated modules rather than
        # polluting sys.modules or a separately installed manimlib package.
        if name not in modules:
            modules[name] = types.ModuleType(name)
        return modules[name]

    native = types.ModuleType("manimlib")
    # Fixed boundary responses, not a second implementation of Choreo's
    # recurrence. Unexpected inputs fail; the native/wheel suites execute
    # the actual interval builder with independent analytic expectations.
    interval_cases = {
        ((1.0, 1.0), 0.0): [(0.0, 1.0), (0.0, 1.0)],
        ((1.0, 1.0), 0.05): [(0.0, 1.0), (0.05, 1.05)],
        ((1.0, 1.0), 1.0): [(0.0, 1.0), (1.0, 2.0)],
        ((5.0, 1.05), 1.0): [(0.0, 5.0), (5.0, 6.05)],
    }
    native.interval_calls = []

    def interval_fixture(durations, lag):
        request = (tuple(durations), lag)
        native.interval_calls.append(request)
        if request not in interval_cases:
            raise AssertionError(f"unregistered interval fixture request: {request}")
        return list(interval_cases[request])

    native._composition_intervals = interval_fixture
    g = vars(native)
    g.update({
        "_np": np, "_copy": copy, "_math": math,
        "_abc": abc, "_collections_abc": collections.abc,
        "_OUT": np.array([0.0, 0.0, 1.0]), "_RATE_FUNC_NAMES": {},
        "Mobject": Mobject, "VMobject": VMobject, "Surface": Surface,
        "StringMobject": StringMobject, "CameraFrame": CameraFrame,
        "Scene": Scene, "_FMN_ROOT": native, "_FMN_MODULE": native,
        "_ensure_module": ensure_module,
    })
    source = ast.parse(BOOTSTRAP.read_text(encoding="utf-8"), filename=str(BOOTSTRAP))
    selected = [
        node for node in source.body
        if (isinstance(node, ast.ClassDef) and node.name in ANIMATION_CLASSES)
        or (isinstance(node, ast.FunctionDef) and node.name in ANIMATION_FUNCTIONS)
    ]
    if {node.name for node in selected} != ANIMATION_CLASSES | ANIMATION_FUNCTIONS:
        raise AssertionError("bootstrap animation definition inventory changed")
    exec(compile(ast.Module(body=selected, type_ignores=[]), str(BOOTSTRAP), "exec"), g)  # ubs:ignore — explicit definition inventory from the checked-in bootstrap, not external input.
    native._install_rate_functions()
    native._install_path_functions()
    spec = importlib.util.spec_from_file_location("fmn_installer_under_test", INSTALLER)
    installer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(installer)
    installer.install(native)
    return native


class NativeCompositionContractTests(unittest.TestCase):
    def setUp(self):
        self.native = make_native()

    def test_compositions_preserve_native_timing_and_deferred_root(self):
        n = self.native
        first, second = n.Animation(n.Mobject()), n.Animation(n.Mobject())
        for cls, lag, duration in (
            (n.AnimationGroup, 0.0, 1.0),
            (n.LaggedStart, 0.05, 1.05),
            (n.Succession, 1.0, 2.0),
        ):
            with self.subTest(cls=cls.__name__):
                group = cls(first, second)
                self.assertIsNone(group.mobject)
                self.assertEqual(n.interval_calls[-1], ((1.0, 1.0), lag))
                self.assertEqual(group.run_time, duration)
                self.assertEqual(group.max_end_time, duration)
                self.assertIsNone(group.rate_func)
                self.assertEqual(group.lag_ratio, lag)
                self.assertEqual(group.animations, [first, second])
                self.assertEqual(str(group), cls.__name__)

    def test_iterable_and_nested_compositions_remain_native_specs(self):
        n = self.native
        children = [n.Animation(n.Mobject()), n.Animation(n.Mobject())]
        nested = n.AnimationGroup((child for child in children), run_time=5.0)
        outer = n.Succession(nested, n.LaggedStart(*children))
        self.assertEqual(nested.animations, children)
        self.assertEqual(nested.run_time, 5.0)
        self.assertIs(outer.animations[0], nested)

    def test_lagged_start_map_can_construct_its_native_group(self):
        n = self.native
        members = n.Mobject(n.Mobject(), n.Mobject())
        mapped = n.LaggedStartMap(n.Animation, members)
        self.assertEqual([child.mobject for child in mapped.animations], list(members))
        self.assertEqual(mapped.run_time, 2.0)
        self.assertEqual(mapped.lag_ratio, 0.05)

    def test_only_real_composition_types_accept_a_missing_root(self):
        n = self.native
        class PretendGroup(n._NativeAnimation):
            _native_kind = "animation_group"
        for cls in (n.Animation, n._NativeAnimation, n.Transform, PretendGroup):
            with self.subTest(cls=cls.__name__), self.assertRaises(TypeError):
                cls(None)
        with self.assertRaises(TypeError):
            n._NativeAnimation(object())

    def test_targetless_transforms_do_not_invent_a_target(self):
        n = self.native
        for cls in (n.CyclicReplace, n.Swap):
            with self.subTest(cls=cls.__name__):
                animation = cls(n.Mobject(), n.Mobject())
                self.assertIsNone(animation._native_target())
                self.assertFalse(hasattr(animation, "target_mobject"))

    def test_ordinary_transforms_still_resolve_and_validate_targets(self):
        n = self.native
        target = n.Mobject()
        self.assertIs(n.Transform(n.Mobject(), target)._native_target(), target)
        with self.assertRaises(TypeError):
            n.Transform(n.Mobject(), object())._native_target()


class TransformDispatchTests(unittest.TestCase):
    def setUp(self):
        self.native = make_native()

    def family_transform(self, **kwargs):
        n = self.native
        source = n.Mobject(n.Mobject(), n.Mobject())
        target = source.copy()
        for member in target.get_family():
            member.data["point"][:, 0] = 2.0
        return n.Transform(source, target, **kwargs)

    def test_lag_ratio_and_rate_are_applied_per_family_member(self):
        transform = self.family_transform(lag_ratio=0.5, rate_func=lambda alpha: alpha * alpha)
        transform.begin()
        transform.interpolate(0.5)
        positions = [float(member.data["point"][0, 0]) for member in transform.mobject.get_family()]
        self.assertEqual(positions, [2.0, 0.5, 0.0])
        transform.finish()
        self.assertFalse(transform.mobject.animating)
        self.assertTrue(all(not member.locked_data_keys for member in transform.mobject.get_family()))

    def test_time_span_is_remapped_before_lag_and_rate(self):
        transform = self.family_transform(
            run_time=1.0, time_span=(0.25, 0.75), lag_ratio=0.5,
            rate_func=lambda alpha: alpha * alpha,
        )
        transform.begin()
        transform.update(0.375)
        positions = [float(member.data["point"][0, 0]) for member in transform.mobject.get_family()]
        self.assertEqual(positions, [0.5, 0.0, 0.0])

    def test_subclass_submobject_hook_is_not_bypassed(self):
        n = self.native
        class Probe(n.Transform):
            def interpolate_submobject(self, live, start, target, alpha):
                self.calls.append((live, start, target, alpha))
        source, target = n.Mobject(), n.Mobject()
        transform = Probe(source, target, rate_func=lambda alpha: alpha)
        transform.calls = []
        transform.begin()
        transform.interpolate(0.5)
        self.assertEqual([call[3] for call in transform.calls], [0.0, 0.5])
        self.assertIs(transform.calls[-1][0], source)
        self.assertIs(transform.calls[-1][1], transform.starting_mobject)
        self.assertIs(transform.calls[-1][2], target)

    def test_subclass_whole_mobject_override_remains_authoritative(self):
        n = self.native
        class Probe(n.Transform):
            def interpolate_mobject(self, alpha):
                self.calls.append(alpha)
        transform = Probe(n.Mobject(), n.Mobject())
        transform.calls = []
        transform.begin()
        transform.interpolate(0.25)
        self.assertEqual(transform.calls, [0.0, 0.25])

    def test_curved_path_only_receives_pointlike_fields(self):
        n = self.native
        calls = []
        def curve(start, end, alpha):
            calls.append((start.shape, end.shape, alpha))
            value = n.straight_path(start, end, alpha)
            value[..., 1] += math.sin(math.pi * alpha)
            return value
        curve._fmn_path_arc = math.pi
        source = n.Mobject(points=((1.0, 0.0, 0.0),))
        target = n.Mobject(points=((-1.0, 0.0, 0.0),))
        target.data["rgba"][:] = 1.0
        target.uniforms["shading"] = (1.0, 0.5, 0.25)
        transform = n.Transform(source, target, path_func=curve, rate_func=lambda alpha: alpha)
        transform.begin()
        transform.interpolate(0.5)
        np.testing.assert_allclose(source.data["point"][0], [0.0, 1.0, 0.0])
        np.testing.assert_allclose(source.data["rgba"][0], [0.5] * 4)
        np.testing.assert_allclose(source.uniforms["shading"], [0.5, 0.25, 0.125])
        self.assertEqual(calls, [((1, 3), (1, 3), 0.0), ((1, 3), (1, 3), 0.5)])

    def test_empty_constant_records_still_interpolate_uniforms(self):
        n = self.native
        source, target = n.Mobject(points=()), n.Mobject(points=())
        source.const_data_keys = ("rgba",)
        target.uniforms["shading"] = (1.0, 0.5, 0.25)
        self.assertIs(source.interpolate(source.copy(), target, 0.5), source)
        self.assertEqual(source.changes, 0)
        np.testing.assert_allclose(source.uniforms["shading"], [0.5, 0.25, 0.125])

    def test_locked_fields_and_uniforms_are_preserved(self):
        n = self.native
        source, target = n.Mobject(), n.Mobject(points=((2.0, 0.0, 0.0),))
        source.locked_data_keys = {"rgba"}
        source.locked_uniform_keys = {"shading"}
        target.data["rgba"][:] = 1.0
        target.uniforms["shading"] = (1.0, 0.5, 0.25)
        source.interpolate(source.copy(), target, 0.5)
        np.testing.assert_allclose(source.data["point"][0], [1.0, 0.0, 0.0])
        np.testing.assert_array_equal(source.data["rgba"], np.zeros((1, 4)))
        self.assertEqual(source.uniforms["shading"], (0.0, 0.0, 0.0))


if __name__ == "__main__":
    unittest.main()
