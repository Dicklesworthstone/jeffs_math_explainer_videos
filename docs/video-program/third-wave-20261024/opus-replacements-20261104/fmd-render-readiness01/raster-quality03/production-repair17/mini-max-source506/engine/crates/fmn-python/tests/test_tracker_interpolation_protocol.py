"""Production tracker definitions/interpolation with substitute native storage.

This suite is not native acceptance; tracker_interpolation.py exercises the
installed extension without replacing its engine or its Scene driver.
"""
import ast
import copy
import importlib.util
import math
from pathlib import Path
from types import ModuleType
import unittest

import numpy as np

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("tracker_interpolation_under_test", ROOT / "python/fmn_python/trackers.py")
trackers = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(trackers)


def environment(install=True):
    native = ModuleType("manimlib.tracker_fixture")
    g = vars(native)
    class Mobject:
        def __init__(self):
            self.data = np.zeros(0, dtype=[("point", "f4", (3,)), ("rgba", "f4", (4,))])
            self.uniforms = {"opacity": 1.0}
            self.locked_data_keys, self.locked_uniform_keys, self.const_data_keys = set(), set(), set()
            self.pointlike_data_keys, self.dirty = ("point",), 0
            self.setter_calls = []
        def init_uniforms(self):
            pass
        def _init_value_tracker(self, kind, value, im):
            self.native_kind = kind
            self.lanes = [math.log(value) if kind == 1 else value, im]
        def _tracker_value(self):
            if self.native_kind == 2:
                raise RuntimeError("not a scalar tracker")
            return math.exp(self.lanes[0]) if self.native_kind == 1 else self.lanes[0]
        def _tracker_complex_value(self):
            if self.native_kind != 2:
                raise RuntimeError("not a complex tracker")
            return tuple(self.lanes)
        def _set_tracker_value(self, value):
            self.lanes[0] = math.log(value) if self.native_kind == 1 else value
            self.setter_calls.append(value)
        def _set_tracker_complex_value(self, re, im):
            self.lanes[:] = [re, im]
            self.setter_calls.append(complex(re, im))
        def note_changed_data(self):
            self.dirty += 1
        def copy(self):
            return copy.deepcopy(self)
    g.update(Mobject=Mobject, _np=np, np=np, _LiveUniforms=dict,
             _install_live_state=Mobject.__init__,
             interpolate_value=lambda a, b, t: (1 - t) * a + t * b,
             straight_path=lambda a, b, t: (1 - t) * a + t * b)
    g["g"] = g
    bootstrap = ROOT / "python/manimlib_bootstrap.py"
    tree = ast.parse(bootstrap.read_text())
    names = {"ValueTracker", "ExponentialValueTracker", "ComplexValueTracker"}
    nodes = [node for node in tree.body if isinstance(node, ast.ClassDef) and node.name in names]
    assert {node.name for node in nodes} == names
    exec(compile(ast.Module(nodes, []), str(bootstrap), "exec"), g)
    source = ROOT / "python/manimlib/_animation_semantics.py"
    installer = next(node for node in ast.parse(source.read_text()).body if isinstance(node, ast.FunctionDef) and node.name == "install")
    names = {"interpolate_uniform", "mobject_interpolate"}
    nodes = [node for node in installer.body if isinstance(node, ast.FunctionDef) and node.name in names]
    assert {node.name for node in nodes} == names
    # mobject_interpolate closes over install()'s shipped_straight_path capture.
    captures = [node for node in installer.body if isinstance(node, ast.Assign) and any(
        isinstance(target, ast.Name) and target.id == "shipped_straight_path" for target in node.targets)]
    assert len(captures) == 1
    exec(compile(ast.Module(captures + nodes, []), str(source), "exec"), g)
    Mobject.interpolate = g["mobject_interpolate"]
    if install:
        trackers.install_tracker_interpolation(native)
    return native


class TrackerInterpolationTests(unittest.TestCase):
    def setUp(self):
        self.n = environment()

    def test_negative_control_only_updates_mirror(self):
        n = environment(False)
        current, start, end = n.ValueTracker(0), n.ValueTracker(0), n.ValueTracker(10)
        current.interpolate(start, end, .4)
        self.assertEqual(current.uniforms["value"][0], 4)
        self.assertEqual(current.get_value(), 0)

    def test_scalar_interpolation_updates_native_and_mirror(self):
        current, start, end = self.n.ValueTracker(0), self.n.ValueTracker(0), self.n.ValueTracker(10)
        self.assertIs(current.interpolate(start, end, .4), current)
        self.assertEqual(current.get_value(), 4)
        self.assertEqual(current.uniforms["value"][0], 4)
        self.assertEqual(current.setter_calls, [4])

    def test_complex_both_lanes_are_updated(self):
        current = self.n.ComplexValueTracker()
        current.interpolate(self.n.ComplexValueTracker(1 + 3j), self.n.ComplexValueTracker(9 - 5j), .25)
        self.assertEqual(current.get_value(), 3 + 1j)
        self.assertEqual(current.uniforms["value"][0], 3 + 1j)
        self.assertEqual(current.uniforms["value"].dtype, np.complex128)

    def test_exponential_interpolation_is_geometric_not_arithmetic(self):
        current = self.n.ExponentialValueTracker(1)
        current.interpolate(self.n.ExponentialValueTracker(1), self.n.ExponentialValueTracker(81), .5)
        self.assertAlmostEqual(current.get_value(), 9, places=13)

    def test_exponential_log_domain_handles_wide_dynamic_range(self):
        current = self.n.ExponentialValueTracker(1)
        current.interpolate(self.n.ExponentialValueTracker(1e-250), self.n.ExponentialValueTracker(1e250), .5)
        self.assertAlmostEqual(current.get_value(), 1, places=13)

    def test_native_endpoint_updates_override_stale_mirrors(self):
        start, end, current = self.n.ValueTracker(2), self.n.ValueTracker(4), self.n.ValueTracker()
        start._set_tracker_value(8)
        end._set_tracker_value(16)
        current.interpolate(start, end, .25)
        self.assertEqual(current.get_value(), 10)
        self.assertEqual(start.uniforms["value"][0], 2)
        self.assertEqual(end.uniforms["value"][0], 4)

    def test_native_complex_endpoint_updates_override_stale_mirrors(self):
        start, end, current = self.n.ComplexValueTracker(), self.n.ComplexValueTracker(), self.n.ComplexValueTracker()
        start._set_tracker_complex_value(2, 4)
        end._set_tracker_complex_value(6, 8)
        current.interpolate(start, end, .5)
        self.assertEqual(current.get_value(), 4 + 6j)

    def test_f64_values_do_not_pass_through_record_precision(self):
        start = 1 + 2**-40
        end = 1 + 2**-38
        current = self.n.ValueTracker()
        current.interpolate(self.n.ValueTracker(start), self.n.ValueTracker(end), .5)
        self.assertEqual(current.get_value(), (start + end) / 2)
        self.assertNotEqual(current.get_value(), float(np.float32(current.get_value())))

    def test_value_lock_preserves_native_value_and_other_interpolation(self):
        current, start, end = self.n.ValueTracker(19), self.n.ValueTracker(0), self.n.ValueTracker(8)
        current.locked_uniform_keys.add("value")
        start.uniforms["opacity"], end.uniforms["opacity"] = 0., 1.
        current.interpolate(start, end, .25)
        self.assertEqual(current.get_value(), 19)
        self.assertEqual(current.uniforms["value"][0], 19)
        self.assertEqual(current.uniforms["opacity"], .25)
        self.assertEqual(current.setter_calls, [])

    def test_point_lock_does_not_lock_tracker(self):
        current = self.n.ValueTracker()
        current.locked_data_keys.add("point")
        current.interpolate(self.n.ValueTracker(2), self.n.ValueTracker(6), .5)
        self.assertEqual(current.get_value(), 4)

    def test_self_as_start_is_read_before_mirror_write(self):
        current = self.n.ValueTracker(2)
        current.interpolate(current, self.n.ValueTracker(10), .25)
        self.assertEqual(current.get_value(), 4)

    def test_self_as_target_is_read_before_write(self):
        current = self.n.ComplexValueTracker(6 + 8j)
        current.interpolate(self.n.ComplexValueTracker(2 + 4j), current, .5)
        self.assertEqual(current.get_value(), 4 + 6j)

    def test_repeated_nonmonotonic_samples_use_endpoints(self):
        start, end, current = self.n.ValueTracker(2), self.n.ValueTracker(10), self.n.ValueTracker()
        for alpha in (.75, .25, 1., 0., .5):
            current.interpolate(start, end, alpha)
            self.assertEqual(current.get_value(), 2 + 8 * alpha)

    def test_scalar_and_complex_extrapolation(self):
        for cls, start, end in ((self.n.ValueTracker, 2, 6), (self.n.ComplexValueTracker, 2j, 6j)):
            current = cls()
            current.interpolate(cls(start), cls(end), 1.5)
            self.assertEqual(current.get_value(), -0.5 * start + 1.5 * end)

    def test_endpoint_kind_mismatch_fails_before_any_write(self):
        current = self.n.ValueTracker(3)
        for start, end in ((self.n.Mobject(), self.n.ValueTracker(5)),
                           (self.n.ValueTracker(5), self.n.ExponentialValueTracker(2))):
            with self.assertRaisesRegex(TypeError, "matching tracker encodings"):
                current.interpolate(start, end, .5)
            self.assertEqual(current.get_value(), 3)
            self.assertEqual(current.setter_calls, [])

    def test_unknown_kind_refuses(self):
        current = self.n.ValueTracker(2)
        current._tracker_kind = 99
        with self.assertRaisesRegex(ValueError, "Unknown"):
            current.interpolate(current, current, .5)

    def test_no_authored_setter_invocation(self):
        class Control(self.n.ValueTracker):
            def set_value(self, value):
                raise AssertionError("interpolation must not rebuild a discrete control")
        current = Control(1)
        current.interpolate(Control(1), Control(9), .5)
        self.assertEqual(current.get_value(), 5)

    def test_control_without_value_mirror_still_updates_native_state(self):
        current, start, end = self.n.ValueTracker(), self.n.ValueTracker(1), self.n.ValueTracker(9)
        for item in (current, start, end):
            del item.uniforms["value"]
        current.interpolate(start, end, .5)
        self.assertEqual(current.get_value(), 5)

    def test_authored_interpolate_super_sees_updated_value(self):
        seen = []
        class Tracker(self.n.ValueTracker):
            def interpolate(self, *args, **kwargs):
                result = super().interpolate(*args, **kwargs)
                seen.append(self.get_value())
                return result
        current = Tracker(0)
        current.interpolate(Tracker(0), Tracker(4), .5)
        self.assertEqual(seen, [2])

    def test_nontracker_keeps_original_protocol(self):
        current, start, end = (self.n.Mobject() for _ in range(3))
        start.uniforms["opacity"], end.uniforms["opacity"] = 0, 1
        self.assertIs(current.interpolate(start, end, .3), current)
        self.assertEqual(current.uniforms["opacity"], .3)

    def test_record_path_and_style_work_still_execute(self):
        current, start, end = self.n.ValueTracker(0), self.n.ValueTracker(0), self.n.ValueTracker(8)
        for item in (current, start, end):
            item.data = np.zeros(2, dtype=item.data.dtype)
        end.data["point"][:] = 4
        end.data["rgba"][:] = 1
        seen = []
        def path(a, b, t):
            seen.append(t)
            return (1-t) * a + t * b + 1
        current.interpolate(start, end, .25, path)
        np.testing.assert_array_equal(current.data["point"], np.full((2, 3), 2))
        np.testing.assert_array_equal(current.data["rgba"], np.full((2, 4), .25))
        self.assertEqual(current.get_value(), 2)
        self.assertEqual(seen, [.25])

    def test_original_path_error_does_not_publish_tracker_value(self):
        current, start, end = self.n.ValueTracker(2), self.n.ValueTracker(2), self.n.ValueTracker(8)
        for item in (current, start, end):
            item.data = np.zeros(1, dtype=item.data.dtype)
        error = ValueError("authored path")
        def path(*args):
            raise error
        with self.assertRaises(ValueError) as raised:
            current.interpolate(start, end, .5, path)
        self.assertIs(raised.exception, error)
        self.assertEqual(current.get_value(), 2)

    def test_exponential_nonrepresentable_extrapolation_refuses_before_write(self):
        current = self.n.ExponentialValueTracker(2)
        for start, end in ((1., 1e250), (1., 1e-250)):
            with self.assertRaisesRegex(ValueError, "nonrepresentable decoded value"):
                current.interpolate(self.n.ExponentialValueTracker(start),
                                    self.n.ExponentialValueTracker(end), 2.)
            self.assertEqual(current.get_value(), 2)
            self.assertEqual(current.setter_calls, [])

    def test_subclass_identity_and_reinstallation_are_preserved(self):
        cls, function = self.n.ValueTracker, self.n.Mobject.interpolate
        replacement = lambda *args: None
        self.n.Mobject.interpolate = replacement
        trackers.install_tracker_interpolation(self.n)
        self.assertIs(self.n.ValueTracker, cls)
        self.assertIs(self.n.Mobject.interpolate, replacement)
        self.assertEqual(function.__name__, "mobject_interpolate")

    def test_endpoint_values_select_infinite_scalar_without_nan(self):
        current = self.n.ValueTracker()
        start, end = self.n.ValueTracker(2), self.n.ValueTracker(float("inf"))
        with np.errstate(invalid="ignore"):
            current.interpolate(start, end, 0)
        self.assertEqual(current.get_value(), 2)
        current.interpolate(start, end, 1)
        self.assertTrue(math.isinf(current.get_value()))

    def test_exponential_unrepresentable_decoding_refuses_before_write(self):
        current, start, end = (self.n.ExponentialValueTracker(1) for _ in range(3))
        start.lanes[0] = -1000.
        for alpha in (0., .5, 1.):
            with self.assertRaisesRegex(ValueError, "positive finite decoded endpoints"):
                current.interpolate(start, end, alpha)
            self.assertEqual(current.get_value(), 1)
            self.assertEqual(current.setter_calls, [])


if __name__ == "__main__":
    unittest.main()
