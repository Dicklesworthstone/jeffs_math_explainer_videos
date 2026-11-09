"""Analytic tests of the shipped curved-arrow endpoint *recipe*.

These tests execute the production installer against a small NumPy affine
object, not a substitute renderer or a native-portal acceptance claim. The
independent point oracle checks placement, curve shape and public hook order.
Run with: python -m unittest discover -s crates/fmn-python/tests -p test_curved_endpoint_recipe.py
"""
from __future__ import annotations

import importlib.util
import math
from pathlib import Path
import unittest

import numpy as np


_SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/arrow_geometry.py"
_SPEC = importlib.util.spec_from_file_location("_fmn_arrow_recipe_under_test", _SOURCE)
_RECIPE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_RECIPE)


class AffineCurve:
    """Independent affine oracle; no native class or renderer is mocked."""

    def __init__(self, points):
        self.points = np.array(points, dtype=float)
        self.calls = []

    def throw_error_if_no_points(self):
        if not len(self.points):
            raise ValueError("empty curve")

    def get_points(self):
        return self.points

    def get_start(self):
        return self.points[0].copy()

    def scale(self, factor, *, about_point):
        self.calls.append(("scale", factor))
        self.points = about_point + factor * (self.points - about_point)
        return self

    def rotate(self, angle, axis=(0.0, 0.0, 1.0)):
        axis = np.array(axis, dtype=float)
        norm = math.hypot(*axis)
        if norm == 0 or not math.isfinite(norm):
            raise ValueError("a rotation needs a finite nonzero axis")
        axis /= norm
        self.calls.append(("rotate", axis.copy()))
        center = (self.points.min(axis=0) + self.points.max(axis=0)) / 2
        vectors = self.points - center
        # Rodrigues' formula, independently applied to every point.
        self.points = center + (
            vectors * math.cos(angle)
            + np.cross(axis, vectors) * math.sin(angle)
            + np.outer(vectors @ axis, axis) * (1 - math.cos(angle))
        )
        return self

    def shift(self, vector):
        self.calls.append(("shift", np.array(vector, copy=True)))
        self.points += vector
        return self


_RECIPE._install_curved_tip_fitting({
    "CurvedArrow": AffineCurve, "_np": np, "_math": math,
    "_vec3": lambda point: tuple(float(point[i]) for i in range(3)),
})


class CurvedEndpointRecipeTests(unittest.TestCase):
    def test_all_axis_directions_and_oblique_chords_reach_the_requested_endpoints(self):
        directions = [*np.eye(3), *-np.eye(3), np.array([2.0, -3.0, 4.0])]
        for source in directions:
            for destination in directions:
                with self.subTest(source=source, destination=destination):
                    first = np.array([1.0, -2.0, 0.5])
                    points = [first, first + .3 * source + [.1, .2, .3], first + source]
                    curve = AffineCurve(points)
                    start = np.array([-4.0, 3.0, 2.0])
                    end = start + 2.5 * destination
                    self.assertIs(curve.put_start_and_end_on(start, end), curve)
                    np.testing.assert_allclose(curve.points[[0, -1]], [start, end], atol=1e-12)
                    self.assertEqual([name for name, _ in curve.calls], ["scale", "rotate", "rotate", "shift"])
                    # Fitting remains a similarity, not a flattened/rebuilt arc.
                    ratio = np.linalg.norm(end - start) / np.linalg.norm(source)
                    np.testing.assert_allclose(
                        np.linalg.norm(np.diff(curve.points, axis=0), axis=1),
                        ratio * np.linalg.norm(np.diff(points, axis=0), axis=1), atol=1e-12,
                    )

    def test_poles_have_a_stable_curve_plane_for_signed_zero(self):
        for sign in (-1.0, 1.0):
            for x in (0.0, -0.0):
                for y in (0.0, -0.0):
                    with self.subTest(sign=sign, x=x, y=y):
                        curve = AffineCurve([[0, 0, 0], [1, 1, 0], [2, 0, 0]])
                        curve.put_start_and_end_on([0, 0, 0], [x, y, sign * 4])
                        np.testing.assert_allclose(curve.points, [[0, 0, 0], [0, 2, sign * 2], [0, 0, sign * 4]], atol=1e-12)

    def test_nearly_vertical_chord_never_passes_a_tiny_axis_to_native_rotation(self):
        curve = AffineCurve([[0, 0, 0], [1, 1, 0], [2, 0, 0]])
        curve.put_start_and_end_on([0, 0, 0], [1e-300, -1e-300, 3])
        np.testing.assert_allclose(curve.points[-1], [0, 0, 3], atol=1e-12)
        self.assertAlmostEqual(float(np.linalg.norm(curve.calls[2][1])), 1.0)
        self.assertGreater(float(np.linalg.norm(curve.calls[2][1].astype(np.float32))), 0.99)

    def test_zero_length_destination_collapses_without_an_invalid_rotation(self):
        curve = AffineCurve([[0, 0, 0], [1, 1, 0], [2, 0, 0]])
        curve.put_start_and_end_on([1, 2, 3], [1, 2, 3])
        np.testing.assert_allclose(curve.points, [[1, 2, 3]] * 3, atol=1e-12)

    def test_invalid_endpoints_and_closed_shafts_refuse_before_transforming(self):
        for end in ([np.nan, 0, 0], [0, np.inf, 0], [0, 0, 1e100]):
            with self.subTest(end=end):
                curve = AffineCurve([[0, 0, 0], [1, 1, 0], [2, 0, 0]])
                before = curve.points.copy()
                with self.assertRaisesRegex(ValueError, "finite and f32-representable"):
                    curve.put_start_and_end_on([0, 0, 0], end)
                np.testing.assert_array_equal(curve.points, before)
                self.assertEqual(curve.calls, [])
        curve = AffineCurve([[0, 0, 0], [1, 1, 0], [0, 0, 0]])
        with self.assertRaisesRegex(Exception, "closed loop"):
            curve.put_start_and_end_on([0, 0, 0], [0, 0, 2])
        self.assertEqual(curve.calls, [])

    def test_endpoint_accessors_return_independent_shaft_points_not_tip_positions(self):
        class TippedCurve(AffineCurve):
            def get_start(self):
                return np.array([-1.0, 0.0, 0.0])
        curve = TippedCurve([[0, 0, 0], [1, 1, 0], [2, 0, 0]])
        start, end = curve.get_start_and_end()
        np.testing.assert_array_equal(start, [0, 0, 0])
        np.testing.assert_array_equal(end, [2, 0, 0])
        start[:] = 10
        end[:] = -10
        np.testing.assert_array_equal(curve.points[[0, -1]], [[0, 0, 0], [2, 0, 0]])


if __name__ == "__main__":
    unittest.main()
