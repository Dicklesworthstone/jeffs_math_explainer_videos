"""Source-compatible complex matrix construction and native live-cell updates."""
import copy
from pathlib import Path
import tempfile
import unittest

import manimlib as m
import numpy as np


class ComplexMatrixTests(unittest.TestCase):
    def test_bound_cells_become_one_live_matrix_without_duplicate_roots(self):
        scene = m.Scene(camera_config=dict(resolution=(96, 54), fps=4))
        circle, square = m.Circle(radius=.2), m.Square(side_length=.3)
        scene.add(circle, square)
        matrix = m.Matrix([[circle, square]])
        self.assertIs(matrix.elements[0], circle)
        self.assertIs(matrix.elements[1], square)
        scene.add(matrix)
        self.assertEqual(list(scene.mobjects), [matrix])
        before = circle.get_center().copy()
        scene.play(circle.animate.shift(m.UP), run_time=.25)
        np.testing.assert_allclose(circle.get_center(), before + m.UP, atol=1e-6)
        self.assertEqual(list(scene.mobjects), [matrix])
        self.assertIs(matrix.get_row(0)[0], circle)
        self.assertIs(matrix.get_column(0)[0], circle)

    def test_default_matrix_preserves_authored_native_cells_and_grid_aliases(self):
        for container in ("list", "array", "iterators"):
            with self.subTest(container=container):
                cells = [m.Circle(radius=.2), m.Square(side_length=.5),
                         m.Tex("x"), m.DecimalNumber(1.25)]
                rows = [cells[:2], cells[2:]]
                values = rows
                if container == "array":
                    values = np.empty((2, 2), dtype=object)
                    for index, cell in enumerate(cells):
                        values[index // 2, index % 2] = cell
                elif container == "iterators":
                    values = (iter(row) for row in rows)
                widths = [cell.get_width() for cell in cells]
                matrix = m.Matrix(values)
                for index, cell in enumerate(cells):
                    self.assertIs(matrix.elements[index], cell)
                    self.assertIs(matrix.mob_matrix[index // 2][index % 2], cell)
                    self.assertIs(matrix.get_row(index // 2)[index % 2], cell)
                    self.assertIs(matrix.get_column(index % 2)[index // 2], cell)
                    self.assertAlmostEqual(cell.get_width(), widths[index], places=5)
                clone = matrix.copy()
                self.assertIs(clone.elements[0], clone.get_row(0)[0])
                self.assertIsNot(clone.elements[0], cells[0])
                before = cells[0].get_points().copy()
                clone.elements[0].shift(m.UP)
                np.testing.assert_array_equal(cells[0].get_points(), before)

    def test_default_matrix_mixes_geometry_text_and_real_numbers(self):
        shape, label = m.Circle(color=m.GREEN), m.Tex("x")
        matrix = m.Matrix([[shape, 1.25], [label, "y"]])
        self.assertIs(matrix.elements[0], shape)
        self.assertIs(matrix.elements[2], label)
        self.assertIsInstance(matrix.elements[1], m.DecimalNumber)
        self.assertEqual(matrix.elements[1].get_value(), 1.25)
        self.assertIsInstance(matrix.elements[3], m.Tex)
        self.assertLess(matrix.brackets[0].get_right()[0], matrix.get_entries().get_left()[0])
        self.assertGreater(matrix.brackets[1].get_left()[0], matrix.get_entries().get_right()[0])
        # A real NaN cell is a DecimalNumber reading `nan`, as in the pinned
        # Reference (fm-5wq.30). Conversion still completes before moving a
        # caller-owned object when a cell refuses (a complex NaN component).
        self.assertEqual(m.Matrix([[m.Square(), float("nan")]]).elements[1].get_tex(), "nan")
        before = shape.get_points().copy()
        with self.assertRaises(ValueError):
            m.Matrix([[shape, complex(float("nan"), 0)]])
        np.testing.assert_array_equal(shape.get_points(), before)

    def test_geometry_matrix_motion_renders_without_duplicate_scene_roots(self):
        class GeometryMatrix(m.Scene):
            def construct(self):
                shape = m.Circle(radius=.3, color=m.BLUE)
                matrix = m.Matrix([[shape, m.Tex("x")], [m.Square(side_length=.5), 1.25]])
                self.add(matrix)
                self.play(shape.animate.shift(.5 * m.UP), run_time=.5)
                assert list(self.mobjects) == [matrix]
                assert matrix.get_row(0)[0] is shape
                assert matrix.get_column(0)[0] is shape
        root = Path(tempfile.mkdtemp(prefix="fmn-geometry-matrix-"))
        sequences = []
        for threads in (1, 4):
            receipt = GeometryMatrix().render(root / str(threads), resolution=(192, 108),
                                               fps=8, threads=threads)
            self.assertEqual(receipt.frame_count, 4)
            frames = [path.read_bytes() for path in sorted(receipt.destination.glob("*.png"))]
            self.assertEqual(len(set(frames)), 4)
            sequences.append(frames)
        self.assertEqual(*sequences)

    def test_builtin_matrix_types_route_complex_values_to_native_readouts(self):
        values = [[1 + 2j, -3j], [complex(4, 0), 5.25 - 6.75j]]
        for constructor in (m.Matrix, m.DecimalMatrix, m.IntegerMatrix, m.TexMatrix):
            for data in (values, np.asarray(values), (iter(row) for row in values)):
                with self.subTest(constructor=constructor.__name__, container=type(data).__name__):
                    matrix = constructor(data)
                    for index, entry in enumerate(matrix.elements):
                        self.assertIsInstance(entry, m.DecimalNumber)
                        self.assertEqual(entry.get_value(), values[index // 2][index % 2])
                        self.assertEqual(len(entry), len(entry.num_string))
                        self.assertTrue(entry.family_members_with_points())
                    self.assertEqual(matrix.elements[0].get_tex(),
                                     "1+2i" if constructor is m.IntegerMatrix else "1.00+2.00i")
                    self.assertEqual(len(matrix.brackets), 2)
                    self.assertLess(matrix.brackets[0].get_right()[0], matrix.get_entries().get_left()[0])
                    self.assertGreater(matrix.brackets[1].get_left()[0], matrix.get_entries().get_right()[0])

    def test_mixed_native_entries_are_not_replaced_or_stringified(self):
        existing = m.Circle(color=m.GREEN)
        matrix = m.Matrix([[existing, 1 + 2j], ["x", 3.5]])
        self.assertIs(matrix.mob_matrix[0][0], existing)
        self.assertIsInstance(matrix.mob_matrix[0][1], m.DecimalNumber)
        self.assertIsInstance(matrix.mob_matrix[1][0], m.Tex)
        self.assertIsInstance(matrix.mob_matrix[1][1], m.DecimalNumber)
        self.assertEqual(matrix.mob_matrix[0][1].get_value(), 1 + 2j)

    def test_precision_style_and_numpy_scalar_types_survive(self):
        values = np.array([[1 + 2j, 3 - 4j]], dtype=np.complex64)
        matrix = m.DecimalMatrix(values, num_decimal_places=3,
                                 decimal_config=dict(color=m.BLUE, include_sign=True, font_size=32))
        self.assertIs(matrix.float_matrix, values)
        for entry in matrix.elements:
            self.assertEqual(entry.get_font_size(), 32)
            self.assertTrue(all(member.get_fill_color() == m.BLUE
                                for member in entry.family_members_with_points()))
        self.assertEqual(matrix.elements[0].get_tex(), "+1.000+2.000i")
        self.assertEqual(matrix.elements[1].get_tex(), "+3.000–4.000i")

    def test_invalid_complex_cell_does_not_move_existing_native_input(self):
        for bad in (complex(1, float("inf")), complex(float("nan"), 2)):
            existing = m.Circle().shift(2 * m.RIGHT)
            before = existing.get_points().copy()
            with self.assertRaises(ValueError):
                m.Matrix([[existing, bad]])
            np.testing.assert_array_equal(existing.get_points(), before)

    def test_nested_live_cells_keep_matrix_aliases_and_scene_roots(self):
        matrix = m.DecimalMatrix([[1 + 2j, 3j]])
        entries, brackets = list(matrix.elements), list(matrix.brackets)
        scene = m.Scene(camera_config=dict(resolution=(192, 108), fps=8))
        parent = m.VGroup(matrix)
        scene.add(parent)
        scene.play(m.ChangeDecimalToValue(entries[0], -2j),
                   m.ChangeDecimalToValue(entries[1], 7), run_time=0.5)
        self.assertEqual(list(scene.mobjects), [parent])
        self.assertEqual(list(matrix.elements), entries)
        self.assertEqual(list(matrix.brackets), brackets)
        for index, expected in enumerate((-2j, 7)):
            entry = entries[index]
            self.assertIs(matrix.get_row(0)[index], entry)
            self.assertIs(matrix.get_column(index)[0], entry)
            self.assertEqual(entry.get_value(), expected)
            self.assertEqual(len(entry), len(entry.num_string))

    def test_complex_matrix_copy_owns_independent_numeric_cells(self):
        matrix = m.DecimalMatrix([[1 + 2j, 3j]])
        for clone in (matrix.copy(), copy.deepcopy(matrix)):
            clone.elements[0].set_value(8 - 9j)
            self.assertEqual(clone.elements[0].get_value(), 8 - 9j)
            self.assertEqual(matrix.elements[0].get_value(), 1 + 2j)
            self.assertIs(clone.mob_matrix[0][0], clone.elements[0])
            self.assertIsNot(clone.elements[0], matrix.elements[0])


if __name__ in ("__main__", "<run_path>"):
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(ComplexMatrixTests))
    if not result.wasSuccessful():
        raise AssertionError("complex matrix acceptance failed")
