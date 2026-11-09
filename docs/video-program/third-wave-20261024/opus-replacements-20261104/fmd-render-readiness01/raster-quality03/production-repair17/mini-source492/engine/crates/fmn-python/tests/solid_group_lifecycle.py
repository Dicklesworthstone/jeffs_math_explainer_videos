"""Actual native surface-group hooks, face topology, records and rendered frames."""
import copy
import importlib
from pathlib import Path
import pickle
import tempfile
import unittest
from unittest.mock import patch

import numpy as np
import manimlib as m

three = importlib.import_module('manimlib.mobject.three_dimensions')


def native_group(kind='cube', **options):
    """Independent Atlas group builder, not the public face assembly route."""
    obj = m.Cube.__new__(m.Cube)
    m._install_live_state(obj)
    if kind == 'cube':
        shape = options.get('square_resolution', (2, 2))
        specs = obj._build_cube(m._native_surface_shell_factory, options.get('side_length', 2.), shape, 0)
    else:
        shape = (2, 2)
        specs = obj._build_prism(m._native_surface_shell_factory,
            options.get('width', 3.), options.get('height', 2.), options.get('depth', 1.), 0)
    m._hang_native_children(obj, specs)
    obj.resolution = shape
    for child in obj:
        child.resolution = shape
        child.compute_triangle_indices()
    obj._apply_surface_style(None, 1., (.1, .5, .1), True)
    return obj


def png_frames(path, obj, threads):
    scene = m.Scene()
    with scene.render_session(path, format='png_sequence', resolution=(96, 64), fps=4, threads=threads):
        scene.add(obj)
        scene.play(obj.animate.shift(m.RIGHT * .6), run_time=.5, rate_func=m.linear)
    return [file.read_bytes() for file in sorted(path.glob('*.png'))]


class SurfaceGroupLifecycleTests(unittest.TestCase):
    def test_each_group_runs_public_hooks_once_and_keeps_aliases(self):
        for base in (m.Cube, m.Prism):
            events = []
            class Authored(base):
                def init_data(self):
                    events.append(('data', self.square_resolution, self.side_length))
                    super().init_data()
                def init_points(self):
                    events.append('points')
                    super().init_points()
                def init_uniforms(self):
                    events.append('uniforms')
                    super().init_uniforms()
                def init_colors(self):
                    events.append('colors')
                    super().init_colors()
            with self.subTest(base=base):
                obj = Authored()
                self.assertEqual(events, [('data', (2, 2), 2.), 'points', 'uniforms', 'colors'])
                self.assertEqual(len(obj), 6)
        self.assertIs(three.Cube, m.Cube)
        self.assertIs(three.Prism, m.Prism)
        self.assertIs(m.Cube.__bases__[0], m.SGroup)
        self.assertIs(m.Prism.__bases__[0], m.Cube)

    def test_custom_root_schema_and_hook_children_survive(self):
        class Authored(m.Cube):
            data_dtype = m.Surface.data_dtype + [('mass', np.float32, (1,))]
            def init_data(self):
                super().init_data()
                self._style_data()['mass'][:] = 7
            def init_points(self):
                self.decoration = m.Square3D(side_length=.2, resolution=(3, 3))
                self.add(self.decoration)
        obj = Authored(square_resolution=(3, 4))
        self.assertIn('mass', obj.data.dtype.names)
        np.testing.assert_array_equal(obj._style_data()['mass'], 7)
        self.assertIs(obj[0], obj.decoration)
        self.assertEqual(obj.decoration.resolution, (3, 3))
        self.assertEqual(len(obj), 7)
        self.assertEqual([x.n_records() for x in obj._solid_faces], [12] * 6)
        scene = m.Scene(); scene.add(obj)
        scene.play(obj.animate.shift(m.RIGHT), run_time=1/30, rate_func=m.linear)
        self.assertIn('mass', obj.data.dtype.names)
        self.assertIs(obj[0], obj.decoration)

    def test_face_factory_is_called_once_and_returned_identities_are_kept(self):
        original, made, calls = three.square_to_cube_faces, [], []
        def factory(face):
            calls.append(face)
            for part in original(face):
                made.append(part)
                yield part.shift(m.UP).set_color(m.RED)
        with patch.object(three, 'square_to_cube_faces', factory):
            obj = m.Cube(side_length=.7)
        self.assertEqual(len(calls), 1)
        self.assertEqual(list(obj), made)
        self.assertTrue(all(part.get_color() == m.RED for part in obj))
        np.testing.assert_allclose(obj.get_center(), m.UP, atol=1e-6)

    def test_square_constructor_override_and_normals_reach_actual_faces(self):
        calls = []
        original = three.Square3D
        class AuthoredFace(original):
            def __init__(self, **kwargs):
                calls.append(kwargs)
                super().__init__(**kwargs)
                self.stretch(1.5, 0)
        with patch.object(three, 'Square3D', AuthoredFace):
            obj = m.Cube(side_length=1, square_resolution=(3, 4))
        self.assertEqual(len(calls), 1)
        self.assertTrue(all(isinstance(face, AuthoredFace) for face in obj))
        self.assertAlmostEqual(obj[0].get_width(), 1.5, places=6)
        self.assertTrue(all(face.resolution == (3, 4) for face in obj))
        self.assertTrue(all(len(face.get_triangle_indices()) == 36 for face in obj))

    def test_independent_native_builder_agrees_at_subpixel_precision(self):
        for side, shape in ((2., (2, 2)), (.7, (3, 4)), (0., (2, 2)), (-1., (2, 2))):
            with self.subTest(side=side, shape=shape):
                actual = m.Cube(side_length=side, square_resolution=shape)
                expected = native_group(side_length=side, square_resolution=shape)
                for a, b in zip(actual, expected):
                    for field in ('point', 'd_normal_point', 'rgba'):
                        # Public family rotations observe f32 records; the monolithic
                        # Atlas builder rotates before narrowing. Near-zero cancellation
                        # is not bit-identical, but stays far below a pixel or f32 ulp
                        # at unit scene scale. Existing golden assertions are unchanged.
                        np.testing.assert_allclose(a.data[field], b.data[field], rtol=0, atol=1e-12)

    def test_prism_dimensions_call_live_public_rescaling(self):
        class Authored(m.Prism):
            def init_data(self):
                self.calls = []
                super().init_data()
            def rescale_to_fit(self, length, dim, stretch=False, **kwargs):
                self.calls.append((length, dim, stretch))
                return super().rescale_to_fit(length, dim, stretch=stretch, **kwargs)
        obj = Authored(width=4, height=3, depth=2)
        self.assertEqual(obj.calls, [(4., 0, True), (3., 1, True), (2., 2, True)])
        np.testing.assert_allclose([obj.get_width(), obj.get_height(), obj.get_depth()], [4, 3, 2])
        for a, b in zip(obj, native_group('prism', width=4, height=3, depth=2)):
            np.testing.assert_array_equal(a.get_points(), b.get_points())

    def test_prism_forwards_face_resolution_and_preserves_uniform_hooks(self):
        class Authored(m.Prism):
            def init_uniforms(self):
                super().init_uniforms()
                self.uniforms['is_fixed_in_frame'] = 1
        obj = Authored(square_resolution=(3, 4), side_length=1, color=m.GREEN,
                       opacity=.4, shading=(.2, .3, .4), depth_test=False, z_index=7)
        self.assertEqual(obj.resolution, (3, 4))
        self.assertTrue(obj.is_fixed_in_frame())
        self.assertEqual(obj.z_index, 7)
        for face in obj:
            self.assertEqual(face.get_color(), m.GREEN)
            self.assertAlmostEqual(face.get_opacity(), .4, places=6)
            np.testing.assert_allclose(face.get_shading(), (.2, .3, .4))
            self.assertFalse(face.uniforms['depth_test'])
            self.assertEqual(face.n_records(), 12)

    def test_copy_deepcopy_and_pickle_remap_generated_face_references(self):
        obj = m.Cube()
        for copied in (obj.copy(), copy.deepcopy(obj), pickle.loads(pickle.dumps(obj))):
            with self.subTest(copy=copied):
                self.assertEqual(list(copied._solid_faces), list(copied))
                self.assertTrue(all(a is not b for a, b in zip(obj, copied)))
                copied._index_faces()
                np.testing.assert_array_equal(obj[0].get_points(), copied[0].get_points())

    def test_failing_factory_never_initializes_or_adopts_partial_output(self):
        calls, error = [], RuntimeError('authored face failure')
        class Authored(m.Cube):
            def init_data(self): calls.append('data'); super().init_data()
        face = m.Square3D()
        def factory(_):
            yield face
            raise error
        with patch.object(three, 'square_to_cube_faces', factory):
            with self.assertRaises(RuntimeError) as caught:
                Authored()
        self.assertIs(caught.exception, error)
        self.assertEqual(calls, [])
        self.assertFalse(face._is_bound())
        self.assertEqual(len(m.Cube()), 6)

    def test_foreign_face_is_not_moved_by_the_face_assembler(self):
        face = m.Square3D(); scene = m.Scene(); scene.add(face)
        before = face.get_points().copy()
        with patch.object(three, 'Square3D', lambda **kwargs: face):
            with self.assertRaisesRegex(ValueError, 'detached'):
                m.Cube()
        np.testing.assert_array_equal(face.get_points(), before)

    def test_invalid_duplicate_and_foreign_factory_output_refuses(self):
        face = m.Square3D()
        foreign = m.Square3D(); scene = m.Scene(); scene.add(foreign)
        for values in ((None,), (face, face), (foreign,)):
            with self.subTest(values=values), patch.object(three, 'square_to_cube_faces', lambda _: iter(values)):
                with self.assertRaises((ValueError, TypeError)):
                    m.Cube()

    def test_nonfinite_controls_and_aggregate_budget_precede_hooks(self):
        calls = []
        class Authored(m.Cube):
            def init_data(self): calls.append('data'); super().init_data()
        for options in ({'side_length': float('nan')}, {'shading': (1, 2)},
                        {'opacity': float('inf')}, {'z_index': 1 << 35},
                        {'square_resolution': (1000, 1000)}, {'square_resolution': (2.5, 2)}):
            with self.subTest(options=options), self.assertRaises((ValueError, TypeError)):
                Authored(**options)
        self.assertEqual(calls, [])

    def test_hook_failure_stops_once_and_constructor_reentry_refuses(self):
        calls, error = [], RuntimeError('group hook')
        class Fails(m.Cube):
            def init_points(self): calls.append('points'); raise error
            def init_uniforms(self): calls.append('uniforms')
        with self.assertRaises(RuntimeError) as caught: Fails()
        self.assertIs(caught.exception, error)
        self.assertEqual(calls, ['points'])
        class Reentrant(m.Cube):
            def init_points(self): m.Cube.__init__(self)
        with self.assertRaisesRegex(RuntimeError, 'progress'): Reentrant()
        self.assertEqual(len(m.Cube()), 6)

    def test_scene_bound_reconstruction_is_refused(self):
        obj = m.Cube(); scene = m.Scene(); scene.add(obj)
        old = list(obj)
        with self.assertRaisesRegex(RuntimeError, 'detached'): m.Cube.__init__(obj)
        self.assertEqual(list(obj), old)

    def test_factory_geometry_renders_like_independent_native_controls(self):
        original = three.square_to_cube_faces
        def factory(face):
            return [part.shift(.4 * m.UP) for part in original(face)]
        root = Path(tempfile.mkdtemp(prefix='fmn-surface-groups-'))
        def authored():
            with patch.object(three, 'square_to_cube_faces', factory): return m.Cube(side_length=.8)
        frames = png_frames(root/'one', authored(), 1)
        self.assertEqual(frames, png_frames(root/'four', authored(), 4))
        self.assertEqual(frames, png_frames(root/'expected', native_group(side_length=.8).shift(.4*m.UP), 1))
        self.assertEqual(len(frames), 2)
        self.assertNotEqual(*frames)

    def test_prism_render_matches_independent_native_group(self):
        root = Path(tempfile.mkdtemp(prefix='fmn-prism-groups-'))
        options = dict(width=.7, height=1.1, depth=.4)
        frames = png_frames(root/'authored', m.Prism(**options), 1)
        self.assertEqual(frames, png_frames(root/'expected', native_group('prism', **options), 1))


if __name__ == '__main__':
    unittest.main()
