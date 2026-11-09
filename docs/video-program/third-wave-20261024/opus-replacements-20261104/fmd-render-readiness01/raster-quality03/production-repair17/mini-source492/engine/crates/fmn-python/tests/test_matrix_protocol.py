"""Python Matrix dispatch and aliasing tests; storage/typesetting are doubles.

The production adapter is imported unchanged. These tests are not native
geometry, font, render, wheel, or cross-platform certification evidence.
"""
from __future__ import annotations

import copy
import importlib.util
import math
from pathlib import Path
from types import ModuleType
import unittest

import numpy as np

MODULE_PATH = Path(__file__).resolve().parents[1] / "python/fmn_python/matrix.py"
_spec = importlib.util.spec_from_file_location("matrix_adapter_under_test", MODULE_PATH)
adapter = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(adapter)


def fixture():
    """Fresh module/classes per test, never an installed manimlib substitute."""
    native = ModuleType("matrix_fixture")
    native._np = np
    native.RIGHT = np.array([1., 0., 0.])
    native.LEFT = -native.RIGHT
    native.DOWN = np.array([0., -1., 0.])
    native.ORIGIN = np.zeros(3)
    native.DEG = math.pi / 180
    native.calls = []
    native.fast_calls = []
    native.fail_tex_at = None

    class VMobject:
        def __init__(self, width=None, height=None, **config):
            self.points = (np.empty((0, 3)) if width is None or height is None else
                           np.array([[-width / 2, -height / 2, 0.], [width / 2, height / 2, 0.]]))
            self.submobjects = []
            self.parents = []
            self.config = dict(config)
            self.color = config.get("color", "white")
            self.operations = []
            self._scene = None

        def __iter__(self):
            return iter(self.submobjects)

        def __len__(self):
            return len(self.submobjects)

        def __getitem__(self, index):
            value = self.submobjects[index]
            return VGroup(*value) if isinstance(index, slice) else value

        def get_family(self):
            found, seen, stack = [], set(), [self]
            while stack:
                item = stack.pop()
                if id(item) in seen:
                    continue
                seen.add(id(item))
                found.append(item)
                stack.extend(reversed(item.submobjects))
            return found

        def get_bounding_box(self):
            arrays = [mob.points for mob in self.get_family() if len(mob.points)]
            points = np.concatenate(arrays) if arrays else np.zeros((1, 3))
            lo, hi = points.min(axis=0), points.max(axis=0)
            return np.array([lo, (lo + hi) / 2, hi])

        def get_center(self):
            return self.get_bounding_box()[1]

        def length_over_dim(self, dim):
            box = self.get_bounding_box()
            return box[2, dim] - box[0, dim]

        def get_width(self):
            return self.length_over_dim(0)

        def get_height(self):
            return self.length_over_dim(1)

        def get_bounding_box_point(self, direction):
            lo, center, hi = self.get_bounding_box()
            return np.where(np.array(direction) > 0, hi, np.where(np.array(direction) < 0, lo, center))

        def shift(self, vector):
            for mob in self.get_family():
                mob.points += np.asarray(vector)
            return self

        def move_to(self, point, aligned_edge=None):
            target = point.get_center() if isinstance(point, VMobject) else np.asarray(point)
            edge = np.zeros(3) if aligned_edge is None else aligned_edge
            return self.shift(target - self.get_bounding_box_point(edge))

        def scale(self, factor, about_point=None):
            center = self.get_center() if about_point is None else np.asarray(about_point)
            for mob in self.get_family():
                mob.points[:] = center + (mob.points - center) * factor
            return self

        def set_height(self, height):
            if self.get_height():
                self.scale(height / self.get_height())
            return self

        def set_width(self, width):
            if self.get_width():
                self.scale(width / self.get_width())
            return self

        def next_to(self, other, direction, buff):
            self.operations.append(("next_to", other, tuple(direction), buff))
            return self.shift(other.get_bounding_box_point(direction) + np.asarray(direction) * buff
                              - self.get_bounding_box_point(-np.asarray(direction)))

        def become(self, other):
            self.operations.append(("become", getattr(other, "source", None)))
            self.points = other.points.copy()
            self.source = getattr(other, "source", None)
            self.color = other.color
            self.submobjects = list(other.submobjects)
            return self

        def rotate(self, angle):
            self.operations.append(("rotate", angle))
            return self

        def set_color(self, color):
            for mob in self.get_family():
                mob.color = color
            return self

        def add_background_rectangle(self):
            self.operations.append(("background",))
            return self

        def set_submobjects(self, members):
            for previous in self.submobjects:
                previous.parents = [parent for parent in previous.parents if parent is not self]
            self.submobjects = list(members)
            for member in self.submobjects:
                if not any(parent is self for parent in member.parents):
                    member.parents.append(self)
            return self

        def add(self, *members):
            return self.set_submobjects([*self.submobjects, *members])

        def center(self):
            return self.shift(-self.get_center())

        def copy(self, deep=False):
            return copy.deepcopy(self)

    class VGroup(VMobject):
        def __init__(self, *members):
            super().__init__()
            self.points = np.empty((0, 3))
            self.set_submobjects(members)

    class Tex(VMobject):
        def __init__(self, source, **config):
            native.calls.append(("tex", source, dict(config)))
            if native.fail_tex_at == len(native.calls):
                raise RuntimeError("native typesetting refusal fixture")
            super().__init__(1., 1., **config)
            self.source = source
            if source.startswith(r"\left["):
                self.points = np.empty((0, 3))
                self.set_submobjects([VMobject(.1, 1).shift([-1, 0, 0]), VMobject(.1, 1).shift([1, 0, 0])])

    class DecimalNumber(VMobject):
        def __init__(self, value, **config):
            native.calls.append(("number", value, dict(config)))
            super().__init__(1., 1., **config)
            self.value = value

    class Matrix(VMobject):
        def __init__(self, grid, **options):
            native.fast_calls.append((type(self), grid, options))
            grid = [[entry if isinstance(entry, VMobject) else Tex(str(entry)) for entry in row] for row in grid]
            VMobject.__init__(self)
            self.points = np.empty((0, 3))
            self.mob_matrix = grid
            self.elements = [entry for row in grid for entry in row]
            self.ellipses, self.brackets = [], []
            self.rows = VGroup(*(VGroup(*row) for row in grid))
            self.columns = VGroup(*(VGroup(*column) for column in zip(*grid)))
            self._matrix_shape = len(grid), len(grid[0])
            self.add(*self.elements)

        def get_rows(self):
            return self.rows

        def get_columns(self):
            return self.columns

        def get_entries(self):
            return VGroup(*self.elements)

        def copy(self, deep=False):
            result = super().copy(deep)
            result.brackets = list(result.brackets)
            return result

    fixture_init = Matrix.__init__

    class DecimalMatrix(Matrix):
        def __init__(self, matrix, **config):
            fixture_init(self, matrix, **config)
            self.float_matrix = matrix

    class IntegerMatrix(DecimalMatrix):
        pass

    class TexMatrix(Matrix):
        def __init__(self, matrix, **config):
            fixture_init(self, matrix, **config)

    class MobjectMatrix(Matrix):
        def __init__(self, group, n_rows, n_cols, **config):
            entries = list(group)[:n_rows * n_cols]
            fixture_init(self, [entries[start:start + n_cols] for start in range(0, len(entries), n_cols)], **config)
            self.group = group

        def element_to_mobject(self, element, **config):
            return element

    def make_existing(grid, cls=Matrix):
        instance = cls.__new__(cls)
        fixture_init(instance, grid)
        return instance
    native.make_existing = make_existing

    for cls in (VMobject, VGroup, Tex, DecimalNumber, Matrix, DecimalMatrix, IntegerMatrix, TexMatrix, MobjectMatrix):
        setattr(native, cls.__name__, cls)
    return native


class MatrixEditingTests(unittest.TestCase):
    def setUp(self):
        self.native = fixture()
        adapter.install_matrix(self.native)
        self.grid = [[self.native.VMobject(1., 1.).shift([3 * column, -2 * row, 0])
                      for column in range(3)] for row in range(2)]
        self.matrix = self.native.make_existing(self.grid)

    def test_installer_is_idempotent_and_keeps_class_identity(self):
        cls = self.native.Matrix
        method = cls.swap_entries_for_ellipses
        adapter.install_matrix(self.native)
        self.assertIs(self.native.Matrix, cls)
        self.assertIs(cls.swap_entries_for_ellipses, method)
        self.assertFalse(getattr(method, "_fmn_schema_placeholder", False))

    def test_conversion_uses_native_types_and_explicit_config(self):
        for value in (1.25, complex(0, 2), np.float32(3), np.complex128(2j)):
            result = self.matrix.element_to_mobject(value, font_size=72, color="blue")
            self.assertIsInstance(result, self.native.DecimalNumber)
            self.assertEqual(result.config["font_size"], 72)
        for value in (4, "x", np.int64(5)):
            result = self.matrix.element_to_mobject(value)
            self.assertIsInstance(result, self.native.Tex)
            self.assertEqual(result.source, str(value))

    def test_caller_owned_mobject_is_not_restyled_or_copied(self):
        entry = self.grid[0][0]
        self.assertIs(self.matrix.element_to_mobject(entry, color="blue"), entry)
        self.assertEqual(entry.color, "white")

    def test_decimal_helper_preserves_configured_defaults(self):
        matrix = self.native.make_existing(self.grid, self.native.DecimalMatrix)
        matrix._matrix_entry_font_size = 64
        matrix._matrix_decimal_places = 3
        matrix._matrix_entry_style = {"color": "red"}
        number = matrix.element_to_mobject(1, num_decimal_places=5)
        self.assertIsInstance(number, self.native.DecimalNumber)
        self.assertEqual(number.config, {"font_size": 64, "num_decimal_places": 5, "color": "red"})

    def test_single_replacement_keeps_live_aliases_and_returns_none(self):
        entry = self.grid[0][0]
        center = entry.get_center().copy()
        result = self.matrix.swap_entry_for_dots(entry, self.native.Tex(r"\vdots"))
        self.assertIsNone(result)
        self.assertIs(self.matrix.rows[0][0], entry)
        self.assertIs(self.matrix.columns[0][0], entry)
        self.assertIs(self.matrix.submobjects[0], entry)
        np.testing.assert_array_equal(entry.get_center(), center)
        self.assertNotIn(entry, self.matrix.elements)
        self.assertEqual(self.matrix.ellipses, [entry])

    def test_row_and_column_keep_order_identity_and_intersection_angle(self):
        before = [entry.get_center().copy() for row in self.grid for entry in row]
        result = self.matrix.swap_entries_for_ellipses(1, 2)
        self.assertIs(result, self.matrix)
        self.assertEqual(len(self.matrix.elements), 2)
        self.assertEqual(len(self.matrix.ellipses), 4)
        self.assertEqual([call[1] for call in self.native.calls], [r"\vdots"] * 3 + [r"\hdots"] * 2)
        for entry, center in zip(self.matrix.submobjects, before):
            np.testing.assert_allclose(entry.get_center(), center)
        self.assertEqual(self.grid[1][2].operations[-1], ("rotate", -math.pi / 4))
        self.assertEqual(self.grid[1][2].source, r"\hdots")
        self.assertAlmostEqual(self.grid[1][0].get_height(), .65 * 3 / 2)
        self.assertAlmostEqual(self.grid[0][2].get_width(), .4 * 7 / 3)

    def test_negative_and_numpy_indices(self):
        self.matrix.swap_entries_for_ellipses(np.int64(-1), -1)
        self.assertEqual(self.grid[1][2].source, r"\hdots")
        self.assertFalse(hasattr(self.grid[0][0], "source"))

    def test_out_of_range_is_noop(self):
        self.assertIs(self.matrix.swap_entries_for_ellipses(200, -100), self.matrix)
        self.assertEqual(self.native.calls, [])
        self.assertEqual(len(self.matrix.elements), 6)

    def test_bad_indices_and_ratios_refuse_before_mutation(self):
        for kwargs in ({"row_index": 1.5}, {"col_index": "1"},
                       {"row_index": 0, "height_ratio": float("nan")},
                       {"col_index": 0, "width_ratio": float("inf")},
                       {"row_index": 0, "height_ratio": -1}):
            with self.subTest(kwargs=kwargs), self.assertRaises((TypeError, ValueError)):
                self.matrix.swap_entries_for_ellipses(**kwargs)
        self.assertEqual(self.native.calls, [])
        self.assertEqual(len(self.matrix.elements), 6)

    def test_typesetting_failure_does_not_partly_replace_grid(self):
        self.native.fail_tex_at = 4
        with self.assertRaisesRegex(RuntimeError, "typesetting refusal"):
            self.matrix.swap_entries_for_ellipses(0, 1)
        self.assertEqual(self.matrix.ellipses, [])
        self.assertEqual(len(self.matrix.elements), 6)
        self.assertTrue(all(not entry.operations for row in self.grid for entry in row))

    def test_replacement_override_is_actually_called(self):
        calls = []
        class Custom(self.native.Matrix):
            def swap_entry_for_dots(self, entry, dots):
                calls.append(entry)
                return super().swap_entry_for_dots(entry, dots)
        matrix = self.native.make_existing(self.grid, Custom)
        matrix.swap_entries_for_ellipses(row_index=0)
        self.assertEqual(calls, self.grid[0])

    def test_callback_failure_retains_original_exception_and_applied_prefix(self):
        failure = LookupError("authored hook")
        calls = []
        class Custom(self.native.Matrix):
            def swap_entry_for_dots(self, entry, dots):
                calls.append(entry)
                if len(calls) == 2:
                    raise failure
                return super().swap_entry_for_dots(entry, dots)
        matrix = self.native.make_existing(self.grid, Custom)
        with self.assertRaises(LookupError) as raised:
            matrix.swap_entries_for_ellipses(row_index=0)
        self.assertIs(raised.exception, failure)
        self.assertEqual(matrix.ellipses, [self.grid[0][0]])

    def test_repeated_replacement_does_not_duplicate_ellipsis_alias(self):
        self.matrix.swap_entries_for_ellipses(row_index=0)
        self.matrix.swap_entries_for_ellipses(row_index=0)
        self.assertEqual(len(self.matrix.ellipses), 3)
        self.assertEqual(len(self.matrix.elements), 3)

    def test_native_bracket_request_and_placement(self):
        result = self.matrix.create_brackets(self.matrix.rows, .25, .2)
        self.assertEqual(len(result), 2)
        self.assertEqual(self.native.calls[0][1],
                         r"\left[\begin{array}{c}\quad \\\quad \\\end{array}\right]")
        self.assertAlmostEqual(result.get_height(), self.matrix.rows.get_height() + .25)
        self.assertEqual(result[0].operations[-1][-1], .2)

    def test_color_and_background_helpers_use_public_getters(self):
        entry = self.grid[0][0]
        native = self.native
        class Custom(native.Matrix):
            def get_columns(self):
                return native.VGroup(native.VGroup(entry))
            def get_entries(self):
                return native.VGroup(entry)
        matrix = self.native.make_existing(self.grid, Custom)
        self.assertIs(matrix.set_column_colors("blue", "green"), matrix)
        self.assertEqual(entry.color, "blue")
        self.assertEqual(self.grid[0][1].color, "white")
        self.assertIs(matrix.add_background_to_entries(), matrix)
        self.assertEqual(entry.operations, [("background",)])

    def test_invalid_entry_type_never_mutates_lists(self):
        with self.assertRaises(TypeError):
            self.matrix.swap_entry_for_dots(self.grid[0][0], object())
        self.assertEqual(len(self.matrix.elements), 6)


class MatrixConstructionTests(unittest.TestCase):
    def setUp(self):
        self.native = fixture()
        adapter.install_matrix(self.native)

    def cell(self, width=1., height=1., at=(0., 0., 0.)):
        return self.native.VMobject(width, height).move_to(at)

    def test_mixed_entries_keep_caller_objects_and_use_native_conversion(self):
        square = self.cell(2., 1., (9, 3, 0)).set_color("blue")
        tall = self.cell(.5, 2., (-4, 5, 0))
        matrix = self.native.Matrix([[square, 1.25], ["x", tall]])
        self.assertIs(matrix.mob_matrix[0][0], square)
        self.assertIs(matrix.rows[0][0], square)
        self.assertIs(matrix.columns[0][0], square)
        self.assertIs(matrix.submobjects[0], square)
        self.assertIs(matrix.mob_matrix[1][1], tall)
        self.assertIsInstance(matrix.mob_matrix[0][1], self.native.DecimalNumber)
        self.assertIsInstance(matrix.mob_matrix[1][0], self.native.Tex)
        self.assertEqual(square.color, "blue")
        self.assertAlmostEqual(square.get_width(), 2.)
        self.assertAlmostEqual(tall.get_height(), 2.)
        self.assertEqual(self.native.fast_calls, [])

    def test_all_constructor_hooks_dispatch_in_reference_order(self):
        calls = []
        class Custom(self.native.Matrix):
            def create_mobject_matrix(self, matrix, *args, **kwargs):
                calls.append("layout-begin")
                result = super().create_mobject_matrix(matrix, *args, **kwargs)
                calls.append("layout-end")
                return result
            def element_to_mobject(self, element, **config):
                calls.append(("entry", element, config["font_size"]))
                return super().element_to_mobject(element, **config)
            def create_brackets(self, *args):
                calls.append("brackets")
                return super().create_brackets(*args)
            def add(self, *members):
                calls.append(("add", len(members)))
                return super().add(*members)
            def center(self):
                calls.append("center")
                return super().center()
            def swap_entries_for_ellipses(self, row, col):
                calls.append(("ellipses", row, col))
                return super().swap_entries_for_ellipses(row, col)
        matrix = Custom([[1, 2]], element_config={"font_size": 30}, ellipses_col=1)
        self.assertEqual(calls, ["layout-begin", ("entry", 1, 30), ("entry", 2, 30), "layout-end",
                                 "brackets", ("add", 2), ("add", 2), "center", ("ellipses", None, 1)])
        self.assertEqual(len(matrix.ellipses), 1)
        self.assertEqual(self.native.fast_calls, [])

    def test_builtin_scalar_constructors_keep_their_existing_fast_paths(self):
        self.native.Matrix([[1]])
        self.native.DecimalMatrix([[1.2]])
        self.native.IntegerMatrix([[2]])
        self.native.TexMatrix([["x"]])
        group = self.native.VGroup(self.cell(), self.cell())
        self.native.MobjectMatrix(group, n_rows=1)
        self.assertEqual([call[0] for call in self.native.fast_calls], [
            self.native.Matrix, self.native.DecimalMatrix, self.native.IntegerMatrix,
            self.native.TexMatrix, self.native.MobjectMatrix,
        ])

    def test_base_class_monkeypatch_is_not_silently_lowered(self):
        called = []
        original = self.native.Matrix.element_to_mobject
        def custom(self, entry, **config):
            called.append(entry)
            return original(self, entry, **config)
        self.native.Matrix.element_to_mobject = custom
        self.native.Matrix([[1, 2]])
        self.assertEqual(called, [1, 2])
        self.assertEqual(self.native.fast_calls, [])

    def test_decimal_subclass_receives_raw_values_and_full_configuration(self):
        token = object()
        called = []
        native = self.native
        class Custom(native.DecimalMatrix):
            def element_to_mobject(self, value, **config):
                called.append((value, dict(config)))
                return super().element_to_mobject(2.5, **config)
        data = [[token]]
        options = {"include_sign": True, "unit": "m", "font_size": 36}
        matrix = Custom(data, num_decimal_places=4, decimal_config=options)
        self.assertIs(matrix.float_matrix, data)
        self.assertIs(called[0][0], token)
        self.assertEqual(called[0][1], dict(options, num_decimal_places=4))
        self.assertEqual(matrix.elements[0].config, dict(options, num_decimal_places=4))
        self.assertEqual(options, {"include_sign": True, "unit": "m", "font_size": 36})

    def test_integer_subclass_keeps_zero_place_default_and_virtual_conversion(self):
        values = []
        class Custom(self.native.IntegerMatrix):
            def element_to_mobject(self, value, **config):
                values.append((value, config["num_decimal_places"]))
                return super().element_to_mobject(value, **config)
        matrix = Custom([[3.5]])
        self.assertEqual(values, [(3.5, 0)])
        self.assertEqual(matrix.elements[0].config["num_decimal_places"], 0)
        self.assertEqual(self.native.fast_calls, [])

    def test_tex_subclass_and_mobject_entries_do_not_stringify_caller_geometry(self):
        source = self.cell(2, 1)
        matrix = self.native.TexMatrix([[source]])
        self.assertIs(matrix.elements[0], source)
        values = []
        class Custom(self.native.TexMatrix):
            def element_to_mobject(self, value, **config):
                values.append(value)
                return source
        Custom([[object()]])
        self.assertEqual(len(values), 1)
        self.assertEqual(self.native.fast_calls, [])

    def test_native_numeric_metadata_retains_caller_input_identity(self):
        for cls in (self.native.DecimalMatrix, self.native.IntegerMatrix):
            data = [[1.5]]
            matrix = cls(data)
            self.assertIs(matrix.float_matrix, data)

    def test_generators_are_consumed_once_before_conversion(self):
        visits = []
        def rows():
            for row in range(2):
                visits.append(row)
                yield (self.cell() for _ in range(2))
        matrix = self.native.Matrix(rows())
        self.assertEqual(visits, [0, 1])
        self.assertEqual(matrix._matrix_shape, (2, 2))

    def test_empty_and_ragged_grids_fail_before_mutating_existing_cells(self):
        entry = self.cell(at=(7, 3, 0))
        points = entry.points.copy()
        for grid in ([], [[]], [[entry], []], [[entry], [entry, entry]]):
            with self.subTest(grid=grid), self.assertRaises(ValueError):
                self.native.Matrix(grid)
        np.testing.assert_array_equal(entry.points, points)
        self.assertEqual(self.native.calls, [])

    def test_entry_budget_bounds_infinite_outer_and_inner_iterators(self):
        import itertools
        from unittest.mock import patch
        with patch.object(adapter, "_MAX_ENTRIES", 4):
            for grid in (itertools.repeat([1]), [itertools.repeat(1)], [[1, 2, 3], [4, 5, 6]]):
                with self.assertRaisesRegex(ValueError, "limit"):
                    self.native.Matrix(grid)
            self.assertEqual(adapter._rows([[1, 2], [3, 4]]), [[1, 2], [3, 4]])
        self.assertEqual(self.native.calls, [])

    def test_conversion_failure_is_original_and_precedes_caller_movement(self):
        entry = self.cell(at=(8, 6, 0))
        points = entry.points.copy()
        failure = LookupError("authored conversion")
        class Custom(self.native.Matrix):
            def element_to_mobject(self, value, **config):
                if value is entry:
                    return entry
                raise failure
        with self.assertRaises(LookupError) as raised:
            Custom([[entry, "fail"]])
        self.assertIs(raised.exception, failure)
        np.testing.assert_array_equal(entry.points, points)

    def test_bad_converter_result_is_rejected_before_native_moves(self):
        entry = self.cell(at=(8, 6, 0))
        points = entry.points.copy()
        class Custom(self.native.Matrix):
            def element_to_mobject(self, value, **config):
                return entry if value == 0 else object()
        with self.assertRaisesRegex(TypeError, "return a VMobject"):
            Custom([[0, 1]])
        np.testing.assert_array_equal(entry.points, points)

    def test_multiple_scene_owners_refuse_before_conversion_or_layout(self):
        a, b = self.cell(at=(4, 0, 0)), self.cell(at=(8, 0, 0))
        a._scene, b._scene = object(), object()
        before = [a.points.copy(), b.points.copy()]
        with self.assertRaisesRegex(ValueError, "multiple Scenes"):
            self.native.Matrix([[a, b]])
        np.testing.assert_array_equal(a.points, before[0])
        np.testing.assert_array_equal(b.points, before[1])
        self.assertEqual(self.native.calls, [])

    def test_conversion_cannot_introduce_a_foreign_scene(self):
        a, b = self.cell(at=(4, 0, 0)), self.cell(at=(8, 0, 0))
        a._scene, b._scene = object(), object()
        before = a.points.copy()
        class Custom(self.native.Matrix):
            def element_to_mobject(self, value, **config):
                return b if value == "foreign" else a
        with self.assertRaisesRegex(ValueError, "multiple Scenes"):
            Custom([[a, "foreign"]])
        np.testing.assert_array_equal(a.points, before)

    def test_custom_grid_hook_retains_the_authored_container_and_row_identity(self):
        authored = [[self.cell(), self.cell()]]
        class Custom(self.native.Matrix):
            def create_mobject_matrix(self, *args, **kwargs):
                return authored
            def element_to_mobject(self, *args, **kwargs):
                raise AssertionError("the overridden grid does not request conversion")
        matrix = Custom([[1]])
        self.assertIs(matrix.mob_matrix, authored)
        self.assertIs(matrix.mob_matrix[0], authored[0])
        self.assertEqual(matrix._matrix_shape, (1, 2))
        self.assertIs(matrix.columns[1][0], authored[0][1])

    def test_custom_grid_hook_must_return_a_rectangular_mobject_grid(self):
        for returned, error in (([[object()]], TypeError), ([[self.cell()], []], ValueError), ([], ValueError)):
            class Custom(self.native.Matrix):
                def create_mobject_matrix(self, *args, **kwargs):
                    return returned
            with self.subTest(returned=returned), self.assertRaises(error):
                Custom([[1]])

    def test_per_entry_movement_remains_virtual(self):
        calls = []
        class AuthoredCell(self.native.VMobject):
            def move_to(self, point, aligned_edge=None):
                calls.append((np.array(point), np.array(aligned_edge)))
                return super().move_to(point, aligned_edge)
        entry = AuthoredCell(1, 2)
        self.native.Matrix([[entry]])
        self.assertEqual(len(calls), 1)
        np.testing.assert_array_equal(calls[0][0], np.zeros(3))
        np.testing.assert_array_equal(calls[0][1], self.native.DOWN)

    def test_natural_layout_alignment_and_authored_height(self):
        for corner in (self.native.DOWN, self.native.ORIGIN, -self.native.DOWN):
            cells = [[self.cell(2, 1), self.cell(1, 3)], [self.cell(.5, 2), self.cell(2, 1)]]
            matrix = self.native.Matrix(cells, h_buff=.75, v_buff=.25, element_alignment_corner=corner)
            anchor = matrix.mob_matrix[0][0].get_bounding_box_point(corner)
            for row in range(2):
                for column in range(2):
                    point = matrix.mob_matrix[row][column].get_bounding_box_point(corner)
                    np.testing.assert_allclose(point - anchor, [column * 2.75, -row * 3.25, 0], atol=1e-12)
        matrix = self.native.Matrix([[self.cell(), self.cell()]], height=4, bracket_v_buff=.25)
        self.assertAlmostEqual(matrix.rows.get_height(), 3.5)
        self.assertAlmostEqual(matrix.brackets.get_height(), 3.75)

    def test_invalid_spacing_corner_and_height_fail_before_moving_cells(self):
        entry = self.cell(at=(2, 3, 0))
        before = entry.points.copy()
        for kwargs in ({"h_buff": math.inf}, {"v_buff": math.nan}, {"height": -1},
                       {"height": .1, "bracket_v_buff": 1}, {"element_alignment_corner": [1, 2]},
                       {"element_alignment_corner": [0, math.nan, 0]}):
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                self.native.Matrix([[entry]], **kwargs)
            np.testing.assert_array_equal(entry.points, before)

    def test_mobject_subclass_partitions_without_touching_unused_cells(self):
        cells = [self.cell(at=(3 * i, 4, 0)) for i in range(5)]
        leftover = cells[-1].points.copy()
        group = self.native.VGroup(*cells)
        called = []
        class Custom(self.native.MobjectMatrix):
            def element_to_mobject(self, element, **config):
                called.append(element)
                return element
        matrix = Custom(group)
        self.assertEqual(matrix._matrix_shape, (2, 2))
        self.assertEqual(called, cells[:4])
        self.assertIs(matrix.group, group)
        self.assertTrue(all(a is b for a, b in zip(matrix.elements, cells[:4])))
        np.testing.assert_array_equal(cells[-1].points, leftover)
        self.assertEqual(self.native.fast_calls, [])

    def test_mobject_matrix_accepts_none_height_for_natural_layout(self):
        cells = [self.cell(2, 1), self.cell(1, 3)]
        matrix = self.native.MobjectMatrix(self.native.VGroup(*cells), n_rows=1, height=None)
        self.assertIs(matrix.elements[0], cells[0])
        self.assertAlmostEqual(cells[0].get_width(), 2)
        self.assertAlmostEqual(cells[1].get_height(), 3)

    def test_mobject_dimensions_refuse_before_division_or_mutation(self):
        group = self.native.VGroup(self.cell(at=(2, 0, 0)))
        before = group[0].points.copy()
        for options in ({"n_rows": 0}, {"n_cols": 0}, {"n_rows": -1}, {"n_rows": 1 << 63},
                        {"n_rows": 3, "n_cols": 4}, {"n_cols": 4}, {"n_rows": 1.5}):
            with self.subTest(options=options), self.assertRaises((ValueError, TypeError)):
                self.native.MobjectMatrix(group, **options)
            np.testing.assert_array_equal(group[0].points, before)
        with self.assertRaises(ValueError):
            self.native.MobjectMatrix(self.native.VGroup())

    def test_duplicate_typed_entry_options_are_not_silently_ignored(self):
        for cls in (self.native.DecimalMatrix, self.native.IntegerMatrix):
            with self.assertRaisesRegex(TypeError, "supplied twice"):
                cls([[1]], decimal_config={"num_decimal_places": 3})
            with self.assertRaisesRegex(TypeError, "element_config"):
                cls([[1]], element_config={})
        with self.assertRaisesRegex(TypeError, "element_config"):
            self.native.TexMatrix([["x"]], element_config={})


    def test_grid_overrides_receive_original_concrete_input_containers(self):
        for base in (self.native.Matrix, self.native.DecimalMatrix, self.native.IntegerMatrix, self.native.TexMatrix):
            seen = []
            class Custom(base):
                def create_mobject_matrix(self, matrix, *args, **kwargs):
                    seen.append(matrix)
                    return super().create_mobject_matrix(matrix, *args, **kwargs)
            source = np.array([[1, 2]])
            Custom(source)
            self.assertIs(seen[0], source)

    def test_dynamic_copy_keeps_live_brackets_and_independent_cells(self):
        matrix = self.native.Matrix([[self.cell(), self.cell()]])
        copied = matrix.copy()
        self.assertIsInstance(copied.brackets, self.native.VGroup)
        self.assertIs(copied.rows[0][0], copied.elements[0])
        self.assertIs(copied.columns[0][0], copied.elements[0])
        self.assertIsNot(copied.elements[0], matrix.elements[0])
        before = matrix.elements[0].points.copy()
        copied.elements[0].shift([4, 0, 0])
        np.testing.assert_array_equal(matrix.elements[0].points, before)


    def test_varied_cell_sizes_and_corners_preserve_reference_spacing(self):
        rng = np.random.default_rng(20260919)
        corners = (np.zeros(3), np.array([0., -1., 0.]), np.array([0., 1., 0.]),
                   np.array([-1., 1., 0.]), np.array([1., -1., 0.]))
        for case in range(160):
            rows, columns = int(rng.integers(1, 5)), int(rng.integers(1, 5))
            dimensions = rng.uniform(.3, 2.5, (rows, columns, 2))
            cells = [[self.cell(*dimensions[row, col], at=rng.normal(size=3))
                      for col in range(columns)] for row in range(rows)]
            v_buff, h_buff = rng.uniform(-.2, .8, 2)
            corner = corners[case % len(corners)]
            matrix = self.native.Matrix(cells, v_buff=v_buff, h_buff=h_buff,
                                        element_alignment_corner=corner)
            first = cells[0][0].get_bounding_box_point(corner)
            for row in range(rows):
                for column in range(columns):
                    expected = [column * (dimensions[:, :, 0].max() + h_buff),
                                -row * (dimensions[:, :, 1].max() + v_buff), 0.]
                    actual = cells[row][column].get_bounding_box_point(corner) - first
                    np.testing.assert_allclose(actual, expected, atol=1e-12,
                                               err_msg=f"case {case}, cell {row},{column}")
                    self.assertIs(matrix.mob_matrix[row][column], cells[row][column])


if __name__ == "__main__":
    unittest.main()
