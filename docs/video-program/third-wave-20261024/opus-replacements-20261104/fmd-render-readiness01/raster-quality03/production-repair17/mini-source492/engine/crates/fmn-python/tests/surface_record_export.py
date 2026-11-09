"""Native indexed-record exports used by corpus surface-to-triangle scenes.

Reference: 3b1b/manim 6199a00, mobject/types/surface.py
compute_triangle_indices/get_shader_vert_indices. No GLSL context is needed.
"""
import copy
import json
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


def triangle_indices(nu, nv):
    # Independent cell enumeration: do not call the implementation's indexer.
    return np.array([index for u in range(max(0, nu - 1))
                     for v in range(max(0, nv - 1))
                     for index in (u*nv+v, (u+1)*nv+v, u*nv+v+1,
                                   u*nv+v+1, (u+1)*nv+v, (u+1)*nv+v+1)], dtype=int)


class SurfaceRecordExportTests(unittest.TestCase):
    def test_triangle_stream_matches_independent_cell_order(self):
        for nu, nv in ((2, 2), (2, 5), (5, 2), (4, 7)):
            surface = m.ParametricSurface(lambda u, v: (u, v, u*v),
                                          resolution=(nu, nv))
            expected = surface.data.copy()[triangle_indices(nu, nv)]
            stream = surface.get_shader_data()
            self.assertEqual(len(stream), 6*(nu-1)*(nv-1))
            np.testing.assert_array_equal(stream, expected)
        print(json.dumps({"bead": "fm-5wq.29", "construct": "Surface(resolution=(4,7)).get_shader_data()",
                          "observed_records": len(stream), "expected_records": 108,
                          "reference": "6199a00:manimlib/mobject/types/surface.py:compute_triangle_indices",
                          "verdict": "pass"}, sort_keys=True))

    def test_empty_topology_does_not_export_strip_control_points(self):
        for shape in ((0, 0), (0, 4), (4, 0), (1, 1), (1, 5), (5, 1)):
            with self.subTest(shape=shape):
                surface = m.Surface(resolution=shape)
                self.assertEqual(surface.get_shader_data().shape, (0,))
                self.assertEqual(surface.get_shader_data().dtype, surface.data.dtype)
        self.assertEqual(m.SGroup(m.Surface(resolution=(2, 2))).get_shader_data().shape, (0,))

    def test_all_native_and_authored_record_fields_are_gathered(self):
        class Tagged(m.Surface):
            data_dtype = [*m.Surface.data_dtype, ("tag", np.float32, (2,))]
        surface = Tagged(resolution=(3, 4))
        surface.data['tag'][:] = np.arange(24).reshape(12, 2)
        surface.data['rgba'][:] = np.arange(48).reshape(12, 4)/48
        surface.shift(m.RIGHT).rotate(.31, axis=m.UP, about_point=m.ORIGIN)
        records = surface.data.copy()
        actual = surface.get_shader_data()
        self.assertEqual(actual.dtype, records.dtype)
        for key in records.dtype.names:
            np.testing.assert_array_equal(actual[key], records[key][triangle_indices(3, 4)])
        np.testing.assert_array_equal(surface.data, records)

    def test_single_zero_index_is_not_treated_as_absent(self):
        for indices in ([0], np.array([0]), np.array([0], dtype=np.uint32)):
            with self.subTest(indices=indices):
                class Indexed(m.Mobject):
                    def get_shader_vert_indices(self): return indices
                obj = Indexed().set_points([[1, 2, 3], [4, 5, 6]])
                self.assertEqual(obj.get_shader_data().shape, (1,))
                np.testing.assert_array_equal(obj.get_shader_data()['point'], [[1, 2, 3]])

    def test_provider_is_called_once_without_truth_testing_or_coercion(self):
        class Indices(np.ndarray):
            def __bool__(self): raise AssertionError("index array was truth-tested")
        calls = []
        indices = np.array([2, 0, 2, 1]).view(Indices)
        class Indexed(m.Mobject):
            def get_shader_vert_indices(self):
                calls.append(self)
                return indices
        obj = Indexed().set_points(np.eye(3))
        actual = obj.get_shader_data()
        self.assertEqual(calls, [obj])
        np.testing.assert_array_equal(actual['point'], np.eye(3)[[2, 0, 2, 1]])

    def test_unindexed_mobject_and_vmobject_keep_live_record_views(self):
        for obj in (m.Mobject().set_points([[0, 0, 0]]), m.Square()):
            with self.subTest(kind=type(obj).__name__):
                actual = obj.get_shader_data()
                self.assertTrue(np.shares_memory(actual, obj.data))
                actual['point'][:, 0] += 1
                np.testing.assert_array_equal(actual['point'], obj.get_points())

    def test_empty_authored_index_array_or_list_exports_no_rows(self):
        for indices in ([], np.array([], dtype=int)):
            class Indexed(m.Mobject):
                def get_shader_vert_indices(self): return indices
            obj = Indexed().set_points(np.eye(3))
            self.assertEqual(obj.get_shader_data().shape, (0,))

    def test_indexed_stream_is_owned_and_fresh_after_placement(self):
        surface = m.Surface(resolution=(2, 3))
        first = surface.get_shader_data()
        frozen = first.copy()
        self.assertFalse(np.shares_memory(first, surface.data))
        scene = m.Scene()
        scene.add(surface)
        surface.shift(2*m.RIGHT)
        np.testing.assert_array_equal(first, frozen)
        np.testing.assert_array_equal(surface.get_shader_data()['point'], frozen['point'] + 2*m.RIGHT)
        first['point'][:] = 0
        self.assertTrue(np.any(surface.get_points()))

    def test_live_resampling_uses_the_new_native_topology(self):
        surface = m.ParametricSurface(lambda u, v: (u, v, 0), resolution=(2, 3))
        scene = m.Scene()
        scene.add(surface)
        surface.set_resolution((4, 5))
        actual = surface.get_shader_data()
        self.assertEqual(len(actual), 72)
        np.testing.assert_array_equal(actual, surface.data.copy()[triangle_indices(4, 5)])

    def test_sorted_face_order_and_copy_are_observed(self):
        surface = m.ParametricSurface(lambda u, v: (u, v, u+v), resolution=(3, 4))
        indices = triangle_indices(3, 4).reshape(-1, 3)
        dots = surface.get_points()[indices[:, 0]] @ np.array([0, 0, -1])
        expected = surface.data.copy()[indices[np.argsort(dots)].flatten()]
        surface.sort_faces_back_to_front(m.IN)
        for obj in (surface, surface.copy(), copy.deepcopy(surface)):
            np.testing.assert_array_equal(obj.get_shader_data(), expected)

    def test_bad_indices_raise_without_rewriting_source_records(self):
        surface = m.Surface(resolution=(2, 3))
        before = surface.data.copy()
        for indices in (np.array([100]), np.array([0.5]), np.array([True, False])):
            with self.subTest(indices=indices):
                surface.triangle_indices = indices
                with self.assertRaises((IndexError, TypeError)):
                    surface.get_shader_data()
                np.testing.assert_array_equal(surface.data, before)

    def test_authored_provider_exception_is_not_retried_or_wrapped(self):
        error, calls = RuntimeError("authored index failure"), []
        class Indexed(m.Mobject):
            def get_shader_vert_indices(self):
                calls.append(self)
                raise error
        obj = Indexed().set_points(np.eye(3))
        with self.assertRaises(RuntimeError) as caught: obj.get_shader_data()
        self.assertIs(caught.exception, error)
        self.assertEqual(calls, [obj])
        np.testing.assert_array_equal(obj.get_points(), np.eye(3))

    def test_triangle_reconstruction_renders_like_literal_grid_at_one_and_four_threads(self):
        def render(path, use_export, threads, missing_triangle=False):
            # The corpus idiom reshapes the stream into triangle vertices.
            surface = m.Surface(u_range=(-1, 1), v_range=(-1, 1), resolution=(3, 4))
            if use_export:
                triangles = surface.get_shader_data()['point'].reshape(-1, 3, 3)
            else:
                points = np.array([(u, v, 0) for u in (-1, 0, 1)
                                   for v in np.linspace(-1, 1, 4)], dtype=np.float32)
                triangles = points[triangle_indices(3, 4)].reshape(-1, 3, 3)
            if missing_triangle: triangles = triangles[1:]
            group = m.VGroup(*(m.Polygon(*points, fill_color=m.BLUE,
                                         fill_opacity=.8, stroke_width=0)
                               for points in triangles))
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96, 64),
                                      fps=3, threads=threads):
                scene.add(group)
                scene.play(group.animate.shift(2*m.RIGHT), run_time=1, rate_func=m.linear)
            frames = [p.read_bytes() for p in sorted(path.glob('*.png'))]
            self.assertEqual(len(frames), 3)
            self.assertEqual(len(set(frames)), 3)
            self.assertTrue(all(frame.startswith(b'\x89PNG\r\n\x1a\n') for frame in frames))
            return frames
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = render(root/'literal', False, 1)
            for threads in (1, 4):
                self.assertEqual(render(root/f'export-{threads}', True, threads), expected)
            self.assertNotEqual(render(root/'missing', False, 1, True), expected)


if __name__ == '__main__':
    unittest.main()
