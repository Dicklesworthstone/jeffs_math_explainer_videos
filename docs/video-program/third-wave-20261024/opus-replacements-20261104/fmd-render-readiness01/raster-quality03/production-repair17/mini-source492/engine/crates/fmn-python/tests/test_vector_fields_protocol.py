"""Vector-field Python dispatch with explicit native-record/builder doubles.

The adapter is production code. Geometry is deliberately synthetic: these
cases test the native call payload and publication boundary, not arrow math,
rendering, Rust storage, installed wheels or certified output.
"""
from __future__ import annotations

import copy
import importlib.util
import itertools
from pathlib import Path
import sys
from types import ModuleType
import unittest
from unittest.mock import patch

import numpy as np

PATH = Path(__file__).resolve().parents[1] / "python/fmn_python/vector_fields.py"
# The adapter uses package-relative imports, so load it as a submodule of
# fmn_python: the importable one, else this source tree's.
try:
    importlib.import_module("fmn_python")
except ImportError:
    sys.path.insert(0, str(PATH.parents[1]))
SPEC = importlib.util.spec_from_file_location("fmn_python.vector_fields_under_test", PATH)
adapter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(adapter)


def fixture():
    native = ModuleType("explicit_field_protocol_fixture")
    native._np = np
    native.calls = []
    native.fail_build = None
    native.children = []
    dtype = np.dtype([("point", "f4", (3,)), ("stroke_width", "f4", (1,)),
                      ("stroke_rgba", "f4", (4,)), ("custom", "f4", (1,))])

    def resize(array, count):
        if not len(array):
            return np.zeros((count, *array.shape[1:]), dtype=array.dtype)
        return array[np.arange(count) * len(array) // count] if count else array[:0]
    native.resize_preserving_order = resize

    class VMobject:
        pointlike_data_keys = ["point"]

        def __init__(self):
            self.data = np.zeros(0, dtype=dtype)
            self.defaults = np.ones(1, dtype=dtype)
            self.defaults["stroke_rgba"] = [0.2, 0.4, 0.8, 0.7]
            self.defaults["custom"] = 9
            self.submobjects, self.updaters = [], []
            self.uniforms = {"flat_stroke": False}

        def _style_data(self):
            return self.data if len(self.data) else self.defaults

        def get_points(self):
            return self.data["point"]

        # The adapter compares family identity and binding around user
        # callbacks; this double is a detached leaf.
        def get_family(self, recurse=True):
            return [self]

        def _is_bound(self):
            return False

        def get_num_points(self):
            return len(self.data)

        def set_points(self, points):
            source = self._style_data().copy()
            if not len(points) and len(self.data):
                self.defaults = source[:1].copy()
            self.data = resize(source, len(points)).copy()
            self.data["point"] = points
            return self

        def match_points(self, other):
            return self.set_points(other.get_points())

        def get_stroke_opacities(self):
            return self.data["stroke_rgba"][:, 3]

        def get_stroke_widths(self):
            return self.data["stroke_width"][:, 0]

        def _build_vector_field_samples(self, factory, bases, vectors, norms, *config):
            if native.fail_build:
                raise native.fail_build
            native.calls.append(dict(bases=np.array(bases), vectors=np.array(vectors),
                                     norms=np.array(norms), config=config))
            if len(bases) < 2:
                raise ValueError("VectorField needs at least two sample points")
            count = max(0, 8 * len(bases) - 1)
            # NOT a renderer/arrow implementation. Distinct synthetic rows
            # make stale geometry and incorrect publication observable.
            self.set_points(np.repeat(np.array(bases).reshape(-1, 3), 8, axis=0)[:count])
            self.data["stroke_width"] = 2
            return native.children

    class Axes:
        dimension = 2
        def __init__(self):
            self.matrix = np.eye(3)
            self.offset = np.zeros(3)
        def c2p(self, *coords):
            coords = np.broadcast_arrays(*[np.asarray(x, dtype=float) for x in coords])
            points = np.stack([*coords, *[np.zeros_like(coords[0]) for _ in range(3 - len(coords))]], axis=-1)
            return points @ self.matrix.T + self.offset

    class VectorField(VMobject):
        def __init__(self, func=None, coords=None, axes=None):
            super().__init__()
            self.sample_coords = np.array([[0., 0.], [1., 0.]] if coords is None else coords)
            self.func = func if func is not None else lambda xs: np.ones_like(xs)
            self.coordinate_system = axes or Axes()
            self.color_map = self.norm_to_opacity_func = None
            self.magnitude_range = (0., 2.)
            self.max_displayed_vect_len = 0.8
            self.stroke_width, self.stroke_opacity = 3., 1.
            self.tip_width_ratio, self.tip_len_to_width = 4., 0.01
            self.flat_stroke, self._native_default_color_map, self.color = False, False, None
            self.sample_points = self.coordinate_system.c2p(*self.sample_coords.T)
            self.base_stroke_width_array = np.ones(15)
            self.set_points(np.zeros((15, 3)))

        # The existing bootstrap's native delegation contract, unmodified.
        def _build_geometry(self, factory, outputs, target=None):
            out_vects, output_norms = self._geometry_inputs(outputs)
            if target is None:
                target = self
            return target._build_vector_field_samples(
                factory, [tuple(row) for row in self.sample_points],
                [tuple(row) for row in out_vects], [float(v) for v in output_norms],
                self.max_displayed_vect_len, self.stroke_width, self.stroke_opacity,
                self.tip_width_ratio, self.tip_len_to_width, self.flat_stroke,
                self._native_default_color_map, self.color, self.magnitude_range,
            )

        def set_stroke_width(self, width):
            self.get_stroke_widths()[:] = width * self.base_stroke_width_array
            self.stroke_width = width
            return self

    native.VMobject, native.VectorField, native.Axes = VMobject, VectorField, Axes
    native._install_live_state = VMobject.__init__
    native._native_shell_factory = VMobject
    return native


class FieldProtocolTests(unittest.TestCase):
    def setUp(self):
        self.native = fixture()
        adapter.install_vector_fields(self.native)
        self.field = self.native.VectorField()

    def test_idempotence_and_class_identity(self):
        cls, method = self.native.VectorField, self.native.VectorField.update_vectors
        adapter.install_vector_fields(self.native)
        self.assertIs(self.native.VectorField, cls)
        self.assertIs(cls.update_vectors, method)

    def test_resampling_rebuilds_bases_width_profile_and_record_count(self):
        self.field.set_sample_coords([[2, 3], [4, 5], [6, 7]])
        self.assertIs(self.field.update_vectors(), self.field)
        np.testing.assert_array_equal(self.native.calls[-1]["bases"], [[2, 3, 0], [4, 5, 0], [6, 7, 0]])
        self.assertEqual(len(self.field.data), 23)
        self.assertEqual(len(self.field.base_stroke_width_array), 23)
        self.field.set_stroke_width(5)
        np.testing.assert_array_equal(self.field.get_stroke_widths()[4::8], [20, 20, 20])

    def test_moving_rotating_axes_refreshes_positions_and_vector_differences(self):
        axes = self.field.coordinate_system
        axes.matrix = np.array([[0., -2., 0.], [3., 0., 0.], [0., 0., 1.]])
        axes.offset = np.array([7., 8., 9.])
        self.field.func = lambda xs: np.tile([1., 2.], (len(xs), 1))
        self.field.update_vectors()
        np.testing.assert_array_equal(self.native.calls[-1]["bases"], [[7, 8, 9], [7, 11, 9]])
        np.testing.assert_array_equal(self.native.calls[-1]["vectors"], [[-4, 3, 0], [-4, 3, 0]])

    def test_three_dimensional_payload(self):
        self.field.coordinate_system.dimension = 3
        self.field.set_sample_coords([[1, 2, 3], [4, 5, 6]])
        self.field.update_vectors()
        np.testing.assert_array_equal(self.native.calls[-1]["bases"], [[1, 2, 3], [4, 5, 6]])
        np.testing.assert_array_equal(self.native.calls[-1]["vectors"], np.ones((2, 3)))

    def test_shrinking_samples_retains_live_paint_and_custom_lanes(self):
        color = self.field.data["stroke_rgba"][0].copy()
        self.field.set_sample_coords([[0, 0], [1, 0], [2, 0]]).update_vectors()
        self.field.set_sample_coords([[5, 6], [7, 8]]).update_vectors()
        self.assertEqual(len(self.field.data), 15)
        np.testing.assert_allclose(self.field.data["stroke_rgba"], np.tile(color, (15, 1)))
        self.assertTrue(np.all(self.field.data["custom"] == 9))

    def test_small_sample_tables_keep_native_capability_boundary(self):
        original = self.field.sample_coords
        for table in ([], [[0, 0]]):
            with self.assertRaisesRegex(ValueError, "at least two"):
                self.field.set_sample_coords(table)
            self.assertIs(self.field.sample_coords, original)
        self.field.sample_coords = np.empty((0, 2))
        with self.assertRaisesRegex(ValueError, "at least two"):
            self.field.update_vectors()
        self.assertEqual(self.native.calls, [])

    def test_callback_failure_preserves_geometry_and_metadata_identity(self):
        points, widths = self.field.sample_points, self.field.base_stroke_width_array
        original = self.field.data.copy()
        self.field.set_sample_coords([[10, 20], [30, 40], [50, 60]])
        failure = LookupError("authored vector failure")
        def bad(xs):
            raise failure
        self.field.func = bad
        with self.assertRaises(LookupError) as raised:
            self.field.update_vectors()
        self.assertIs(raised.exception, failure)
        self.assertIs(self.field.sample_points, points)
        self.assertIs(self.field.base_stroke_width_array, widths)
        np.testing.assert_array_equal(self.field.data, original)
        self.assertFalse(adapter._FIELD_UPDATES.busy(self.field))

    def test_native_failure_preserves_last_field_and_next_update_works(self):
        self.native.fail_build = RuntimeError("native budget")
        original = self.field.data.copy()
        with self.assertRaisesRegex(RuntimeError, "native budget"):
            self.field.update_vectors()
        np.testing.assert_array_equal(self.field.data, original)
        self.native.fail_build = None
        self.field.update_vectors()
        self.assertEqual(len(self.native.calls), 1)

    def test_callback_inputs_are_detached_and_function_executes_once(self):
        before = self.field.sample_coords.copy()
        seen = []
        def func(xs):
            seen.append(xs)
            xs[:] = 99
            return np.zeros_like(xs)
        self.field.func = func
        self.field.update_vectors()
        self.assertEqual(len(seen), 1)
        np.testing.assert_array_equal(self.field.sample_coords, before)
        np.testing.assert_array_equal(self.native.calls[-1]["bases"][:, :2], before)

    def test_bad_outputs_refuse_before_native_publication(self):
        original = self.field.data.copy()
        for output in (np.ones((1, 2)), np.ones(2), np.ones((2, 4)),
                       [[np.nan, 1], [2, 3]], [[np.inf, 1], [2, 3]],
                       np.ones((2, 2), dtype=complex)):
            self.field.func = lambda xs, output=output: output
            with self.subTest(output=output), self.assertRaises((TypeError, ValueError)):
                self.field.update_vectors()
            np.testing.assert_array_equal(self.field.data, original)
        self.assertEqual(self.native.calls, [])

    def test_zero_magnitude_range_has_finite_color_alphas(self):
        self.field.func = lambda xs: np.zeros_like(xs)
        self.field.magnitude_range = (0, 0)
        seen = []
        def color(alphas):
            seen.append(alphas.copy())
            return np.column_stack([alphas, alphas, np.ones_like(alphas)])
        self.field.color_map = color
        with np.errstate(all="raise"):
            self.field.update_vectors()
        np.testing.assert_array_equal(seen, np.zeros((1, 15)))
        np.testing.assert_array_equal(self.field.data["stroke_rgba"][:, :3], np.tile([0, 0, 1], (15, 1)))

    def test_color_and_opacity_callbacks_precede_live_geometry_and_accept_columns(self):
        original = self.field.data.copy()
        def color(alphas):
            np.testing.assert_array_equal(self.field.data, original)
            return np.tile([0.8, 0.2, 0.3, 0.9], (len(alphas), 1))
        def opacity(norms):
            np.testing.assert_array_equal(self.field.data, original)
            return np.full((len(norms), 1), .25)
        self.field.color_map, self.field.norm_to_opacity_func = color, opacity
        self.field.update_vectors()
        np.testing.assert_allclose(self.field.data["stroke_rgba"], np.tile([.8, .2, .3, .25], (15, 1)))

    def test_opacity_failure_does_not_leave_partial_color_or_geometry(self):
        original = self.field.data.copy()
        self.field.color_map = lambda xs: np.ones((len(xs), 4))
        self.field.norm_to_opacity_func = lambda xs: np.full(len(xs), np.nan)
        with self.assertRaises(ValueError):
            self.field.update_vectors()
        np.testing.assert_array_equal(self.field.data, original)
        self.assertNotIn(id(self.field), adapter._STYLE_TARGETS)

    def test_scalar_opacity_and_rgb_preserve_unowned_lanes(self):
        self.field.norm_to_opacity_func = lambda xs: .4
        self.field.update_vectors()
        np.testing.assert_allclose(self.field.get_stroke_opacities(), .4)
        np.testing.assert_allclose(self.field.data["stroke_rgba"][:, :3], np.tile([.2, .4, .8], (15, 1)))
        self.assertTrue(np.all(self.field.data["custom"] == 9))

    def test_invalid_color_opacity_shapes_and_values(self):
        cases = [("color_map", lambda xs: np.ones((len(xs), 2))),
                 ("color_map", lambda xs: np.full((len(xs), 3), np.inf)),
                 ("color_map", lambda xs: np.full((len(xs), 3), 1e100)),
                 ("norm_to_opacity_func", lambda xs: np.ones((len(xs), 2))),
                 ("norm_to_opacity_func", lambda xs: np.ones(2))]
        for name, callback in cases:
            field = self.native.VectorField()
            setattr(field, name, callback)
            with self.subTest(name=name), self.assertRaises(ValueError):
                field.update_vectors()

    def test_reentrant_update_is_rejected_without_leaking_busy_state(self):
        self.field.func = lambda xs: self.field.update_vectors()
        with self.assertRaisesRegex(RuntimeError, "reenter"):
            self.field.update_vectors()
        self.assertFalse(adapter._FIELD_UPDATES.busy(self.field))

    def test_resampling_from_callback_is_rejected(self):
        self.field.func = lambda xs: self.field.set_sample_coords([[7, 8]])
        with self.assertRaisesRegex(RuntimeError, "resample"):
            self.field.update_vectors()
        self.assertEqual(len(self.field.sample_coords), 2)

    def test_public_update_and_width_helpers_keep_authored_receiver(self):
        seen = []
        class Custom(self.native.VectorField):
            def update_sample_points(self):
                seen.append(("points", self))
                return super().update_sample_points()
            def init_base_stroke_width_array(self, count):
                seen.append(("widths", self, count))
                return super().init_base_stroke_width_array(count)
        field = Custom()
        field.update_vectors()
        self.assertEqual(seen, [("points", field), ("widths", field, 2)])

    def test_field_children_updaters_and_uniforms_are_not_replaced(self):
        child, callback = object(), lambda m: None
        self.field.submobjects.append(child)
        self.field.updaters.append(callback)
        uniforms = self.field.uniforms
        self.field.update_vectors()
        self.assertEqual(self.field.submobjects, [child])
        self.assertEqual(self.field.updaters, [callback])
        self.assertIs(self.field.uniforms, uniforms)

    def test_budget_and_invalid_coordinates_fail_without_rebinding(self):
        original = self.field.sample_coords
        with patch.object(adapter, "_MAX_SAMPLES", 3):
            with self.assertRaisesRegex(ValueError, "budget"):
                self.field.set_sample_coords(itertools.repeat([1, 2]))
        self.assertIs(self.field.sample_coords, original)
        for data in ([[np.nan, 0]], [[1, 2, 3, 4]], [1, 2]):
            with self.assertRaises(ValueError):
                self.field.set_sample_coords(data)
        self.assertIs(self.field.sample_coords, original)

    def test_coordinate_shape_and_overflow_are_not_silently_reshaped(self):
        original = self.field.data.copy()
        for conversion in (lambda *xs: np.ones(6), lambda *xs: np.full((2, 3), 1e100)):
            self.field.coordinate_system.c2p = conversion
            with self.assertRaises(ValueError):
                self.field.update_vectors()
            np.testing.assert_array_equal(self.field.data, original)

    def test_native_family_expansion_is_not_discarded(self):
        self.native.children = [object()]
        with self.assertRaisesRegex(RuntimeError, "returned children"):
            self.field.update_vectors()


if __name__ == "__main__":
    unittest.main()
