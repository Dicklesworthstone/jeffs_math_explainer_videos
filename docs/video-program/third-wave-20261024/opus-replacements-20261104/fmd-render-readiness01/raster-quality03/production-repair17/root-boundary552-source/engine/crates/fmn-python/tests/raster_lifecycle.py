"""Image subclass hooks on native records, resources, captures and frame output.

No replacement image renderer or simulated material satisfies this suite.
"""
import copy
import pickle
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


_PIXELS = np.array([[[240, 20, 10, 255], [10, 230, 50, 127], [70, 180, 240, 255]],
                    [[5, 30, 245, 255], [250, 220, 5, 191], [200, 20, 210, 255]]], dtype=np.uint8)


def resource(pixels=_PIXELS):
    height, width = pixels.shape[:2]
    return m._RasterImage(width, height, pixels.tobytes())


def direct_image(pixels=_PIXELS, height=2, opacity=.6):
    """Independent Atlas builder, not the ImageMobject constructor under test."""
    obj = m._native_shell_factory()
    specs = m._build_raster_image(obj, resource(pixels), m._native_shell_factory, height, opacity, 0)
    if specs:
        raise AssertionError('native image quad unexpectedly has children')
    return obj


def capture(obj):
    return m.Camera(resolution=(96, 64)).capture_snapshot(obj).png()


class RasterLifecycleTests(unittest.TestCase):
    def test_recipe_and_native_pixels_are_available_to_hooks_in_order(self):
        class Authored(m.ImageMobject):
            def init_data(self):
                self.events = ['data']
                self.seeds = (self.height, self.opacity, self.pixel_width, self.pixel_height)
                np.testing.assert_array_equal(self.get_pixel_array(), _PIXELS)
                super().init_data()
            def init_points(self):
                self.events.append('points')
                super().init_points()
                self.sample = self.point_to_rgb(self.get_center())
            def init_uniforms(self):
                self.events.append('uniforms')
                super().init_uniforms()
            def init_colors(self):
                self.events.append('colors')
                super().init_colors()
        image = Authored(_PIXELS, height=2, opacity=.6)
        self.assertEqual(image.events, ['data', 'points', 'uniforms', 'colors'])
        self.assertEqual(image.seeds, (2, .6, 3, 2))
        self.assertEqual(image.sample.shape, (3,))
        self.assertNotIn('_fmn_image_authoring_busy', vars(image))
        self.assertFalse(any(isinstance(v, m._RasterImage) for v in vars(image).values()))

    def test_custom_schema_children_and_exported_views_survive_admission(self):
        class Authored(m.ImageMobject):
            data_dtype = m.ImageMobject.data_dtype + [('mass', 1)]
            def init_data(self):
                super().init_data()
                self.data['mass'][:] = 13
                self.data_view = self.data
                self.decoration = m.Dot(radius=.05)
                self.add(self.decoration)
            def init_points(self):
                super().init_points()
                self.shift(m.RIGHT)
        image = Authored(_PIXELS, height=2)
        self.assertIn(image.decoration, image.submobjects)
        self.assertEqual(image.data.dtype.names, ('point', 'im_coords', 'opacity', 'mass'))
        np.testing.assert_array_equal(image.data_view['mass'], 13)
        image.data_view['mass'][:] = 7
        np.testing.assert_array_equal(image.data['mass'], 7)
        np.testing.assert_allclose(image.get_center(), m.RIGHT, atol=1e-6)
        self.assertEqual(m._read_raster_image(image).pixels(), _PIXELS.tobytes())

    def test_authored_uv_pose_and_alpha_hooks_control_rendered_pixels(self):
        class Authored(m.ImageMobject):
            def init_points(self):
                super().init_points()
                self.data['im_coords'][:, 0] = 1 - self.data['im_coords'][:, 0]
                self.rotate(.2).shift(.3 * m.RIGHT)
            def init_colors(self):
                self.set_opacity(.4)
        actual = Authored(_PIXELS, height=2)
        expected = direct_image(opacity=.4)
        expected.data['im_coords'][:, 0] = 1 - expected.data['im_coords'][:, 0]
        expected.rotate(.2).shift(.3 * m.RIGHT)
        self.assertEqual(capture(actual), capture(expected))
        self.assertNotEqual(capture(actual), capture(direct_image(opacity=.4)))

    def test_custom_init_data_may_supply_the_whole_quad_without_super(self):
        points = np.array([[-1,1,0], [-1,-1,0], [1,1,0], [1,-1,0], [1,1,0], [-1,-1,0]])
        uv = np.array([[0,0], [0,1], [1,0], [1,1], [1,0], [0,1]])
        class Authored(m.ImageMobject):
            def init_data(self):
                self.resize(6)
                self.data['point'][:] = points
                self.data['im_coords'][:] = uv
                self.data['opacity'][:] = .6
            def init_points(self):
                self.shift(.25 * m.UP)
        image = Authored(_PIXELS)
        np.testing.assert_array_equal(image.get_points(), points + .25 * m.UP)
        np.testing.assert_array_equal(image.get_pixel_array(), _PIXELS)
        expected = direct_image()
        expected.set_width(2, stretch=True).shift(.25 * m.UP)
        self.assertEqual(capture(image), capture(expected))

    def test_uniform_override_runs_after_constructor_flags(self):
        class Authored(m.ImageMobject):
            def init_uniforms(self):
                super().init_uniforms()
                self.fixed_seen = self.is_fixed_in_frame()
                self.unfix_from_frame()
                self.set_z_index(12)
        image = Authored(_PIXELS, is_fixed_in_frame=True, z_index=7)
        self.assertTrue(image.fixed_seen)
        self.assertFalse(image.is_fixed_in_frame())
        snapshot = image._engine_state()['snapshot']
        image.set_z_index(12)
        self.assertEqual(image._engine_state()['snapshot'], snapshot)
        image.set_z_index(7)
        self.assertNotEqual(image._engine_state()['snapshot'], snapshot)

    def test_classmethod_preserves_subclass_and_native_pixels(self):
        class Authored(m.ImageMobject):
            def init_points(self):
                super().init_points()
                self.shift(m.UP)
        obj = Authored.from_pixel_array(_PIXELS, height=2)
        self.assertIsInstance(obj, Authored)
        np.testing.assert_allclose(obj.get_center(), m.UP)
        np.testing.assert_array_equal(obj.get_pixel_array(), _PIXELS)

    def test_constructor_freezes_input_before_authored_hooks(self):
        supplied = _PIXELS.copy()
        class Authored(m.ImageMobject):
            def init_data(self):
                supplied[:] = 0
                super().init_data()
        image = Authored(supplied)
        np.testing.assert_array_equal(image.get_pixel_array(), _PIXELS)
        self.assertFalse(supplied.any())

    def test_decode_and_configuration_fail_before_initialization_hooks(self):
        calls = []
        class Authored(m.ImageMobject):
            def init_data(self):
                calls.append('data')
                super().init_data()
        for source, options in ((b'not a PNG', {}), (_PIXELS, {'height': -1}),
                                (_PIXELS, {'opacity': float('nan')}), (_PIXELS, {'z_index': 2**40})):
            with self.subTest(options=options), self.assertRaises((TypeError, ValueError)):
                Authored(source, **options)
        self.assertEqual(calls, [])

    def test_failed_hook_is_not_retried_and_releases_invocation_guard(self):
        failure = RuntimeError('authored image initialization')
        calls, retained = [], []
        class Authored(m.ImageMobject):
            def init_points(self):
                calls.append('points')
                retained.append(self)
                raise failure
            def init_uniforms(self):
                calls.append('uniforms')
        with self.assertRaises(RuntimeError) as caught:
            Authored(_PIXELS)
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, ['points'])
        self.assertNotIn('_fmn_image_authoring_busy', vars(retained[0]))
        # The guard is gone and the already initialized resource is usable.
        retained[0].set_pixel_array(_PIXELS[:, ::-1])
        np.testing.assert_array_equal(retained[0].get_pixel_array(), _PIXELS[:, ::-1])

    def test_hook_cannot_replace_constructor_resource_reentrantly(self):
        class Authored(m.ImageMobject):
            def init_points(self):
                self.set_pixel_array(_PIXELS[:, ::-1])
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            Authored(_PIXELS)

    def test_invalid_authored_rows_or_columns_refuse_at_final_admission(self):
        for kind in ('count', 'point', 'im_coords', 'opacity'):
            class Authored(m.ImageMobject):
                def init_colors(self):
                    if kind == 'count':
                        self.resize(5)
                    else:
                        self.data[kind][0, 0] = np.nan
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                Authored(_PIXELS)

    def test_scene_adoption_in_a_hook_cannot_bypass_detached_admission(self):
        scene, retained = m.Scene(), []
        class Authored(m.ImageMobject):
            def init_colors(self):
                retained.append(self)
                scene.add(self)
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            Authored(_PIXELS)
        self.assertIs(retained[0]._scene, scene)  # Authored effects are not rolled back.

    def test_live_edits_preserve_custom_fields_and_render_through_scene(self):
        class Authored(m.ImageMobject):
            data_dtype = m.ImageMobject.data_dtype + [('mass', 1)]
            def init_data(self):
                super().init_data()
                self.data['mass'][:] = 9
        image = Authored(_PIXELS, height=2)
        scene = m.Scene(); scene.add(image)
        before = image.get_points().copy()
        view = image.data
        image.set_pixel_array(_PIXELS[:, ::-1])
        scene.play(image.animate.shift(m.RIGHT), run_time=1/30, rate_func=m.linear)
        np.testing.assert_allclose(image.get_points(), before + m.RIGHT, atol=1e-6)
        np.testing.assert_array_equal(image.data['mass'], 9)
        view['opacity'][:] = .3
        np.testing.assert_allclose(image.data['opacity'], .3)

    def test_copies_pickle_and_regeneration_use_current_native_resource(self):
        image = m.ImageMobject(_PIXELS, height=2)
        for clone in (image.copy(), copy.deepcopy(image), pickle.loads(pickle.dumps(image))):
            self.assertFalse(any(isinstance(v, m._RasterImage) for v in vars(clone).values()))
            np.testing.assert_array_equal(clone.get_pixel_array(), _PIXELS)
            clone.height = 3
            clone.init_points()
            self.assertAlmostEqual(clone.get_height(), 3)
            self.assertAlmostEqual(clone.get_width(), 4.5)
            np.testing.assert_array_equal(image.get_pixel_array(), _PIXELS)
        image.set_pixel_array(_PIXELS.transpose(1, 0, 2))
        image.init_points()
        self.assertEqual((image.pixel_width, image.pixel_height), (2, 3))
        self.assertAlmostEqual(image.get_width(), 4/3, places=6)

    def test_ordinary_output_matches_the_independent_native_builder(self):
        for shape, height, opacity in (((2,3), 2, .6), ((3,7), 1.75, .25), ((7,2), .7, 1)):
            pixels = np.resize(_PIXELS, (*shape, 4))
            image = m.ImageMobject(pixels, height=height, opacity=opacity)
            expected = direct_image(pixels, height, opacity)
            with self.subTest(shape=shape):
                np.testing.assert_array_equal(image.data, expected.data)
                self.assertEqual(capture(image), capture(expected))

    def test_animation_frames_match_literal_pose_and_worker_count(self):
        class Authored(m.ImageMobject):
            def init_points(self):
                super().init_points()
                self.data['im_coords'][:, 0] = 1 - self.data['im_coords'][:, 0]
                self.shift(.5 * m.LEFT)
        def render(destination, authored, threads):
            scene = m.Scene()
            with scene.render_session(destination, format='png_sequence', resolution=(96,64), fps=4, threads=threads):
                if authored:
                    obj = Authored(_PIXELS, height=2, opacity=.6)
                else:
                    obj = direct_image()
                    obj.data['im_coords'][:, 0] = 1 - obj.data['im_coords'][:, 0]
                    obj.shift(.5 * m.LEFT)
                scene.add(obj)
                scene.play(obj.animate.shift(m.RIGHT), run_time=.5, rate_func=m.linear)
            return [p.read_bytes() for p in sorted(destination.glob('*.png'))]
        root = Path(tempfile.mkdtemp(prefix='fmn-image-lifecycle-'))
        frames = render(root/'one', True, 1)
        self.assertEqual(frames, render(root/'four', True, 4))
        self.assertEqual(frames, render(root/'expected', False, 1))
        self.assertEqual(len(frames), 2)
        self.assertNotEqual(frames[0], frames[1])


def run_raster_lifecycle():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(RasterLifecycleTests))
    if not result.wasSuccessful():
        raise AssertionError('native image lifecycle acceptance failed')


if __name__ == '__main__':
    run_raster_lifecycle()
