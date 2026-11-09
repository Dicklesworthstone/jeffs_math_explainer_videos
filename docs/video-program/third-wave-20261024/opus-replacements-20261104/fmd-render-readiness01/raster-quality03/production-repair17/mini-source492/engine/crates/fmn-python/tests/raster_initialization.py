"""Native image admission on real custom record buffers, views and resources.

No mocked storage or rasterizer. Also executed by the embedded PyO3 harness.
"""
import unittest

import numpy as np
import manimlib as m


_PIXELS = np.array([[[240, 20, 10, 255], [10, 230, 50, 127]],
                    [[5, 30, 245, 255], [250, 220, 5, 191]]], dtype=np.uint8)


class ImageRecords(m.Mobject):
    data_dtype = m.ImageMobject.data_dtype + [("mass", 1)]


def raw(count=6):
    result = ImageRecords()
    result.resize(count)
    if count == 6:
        canonical = m.ImageMobject(_PIXELS, height=2, opacity=.75)
        for key in ("point", "im_coords", "opacity"):
            result.data[key] = canonical.data[key]
    result.data['mass'][:] = 13
    return result


def resource(pixels=_PIXELS):
    height, width = pixels.shape[:2]
    return m._RasterImage(width, height, pixels.tobytes())


def capture(obj):
    return m.Camera(resolution=(80, 64)).capture_snapshot(obj).png()


class RasterInitializationTests(unittest.TestCase):
    def test_preserves_declared_schema_and_live_same_size_views(self):
        obj = raw()
        data, old = obj.data, obj.data.copy()
        m._initialize_raster_image(obj, resource())
        np.testing.assert_array_equal(obj.data, old)
        self.assertEqual(obj.data.dtype.names, ('point', 'im_coords', 'opacity', 'mass'))
        data['opacity'][:] = .25
        np.testing.assert_array_equal(obj.data['opacity'], data['opacity'])
        obj.data['mass'][:] = 42
        np.testing.assert_array_equal(data['mass'], 42)
        self.assertEqual(m._read_raster_image(obj).pixels(), _PIXELS.tobytes())
        self.assertFalse(obj._is_bound())

    def test_published_image_renders_like_the_independent_native_builder(self):
        obj = raw()
        m._initialize_raster_image(obj, resource())
        expected = m.ImageMobject(_PIXELS, height=2, opacity=.75)
        self.assertEqual(capture(obj), capture(expected))
        obj.data['im_coords'][:, 0] = 1 - obj.data['im_coords'][:, 0]
        self.assertNotEqual(capture(obj), capture(expected))

    def test_placement_uniforms_children_and_updaters_survive(self):
        obj = raw().rotate(.3).shift((.2, .4, .1))
        decoration = m.Dot().shift(m.LEFT)
        obj.add(decoration)
        obj.fix_in_frame().set_z_index(7)
        calls = []
        updater = lambda mob, dt: calls.append(dt)
        obj.add_updater(updater)
        # add_updater's default call=True runs one dt=0 pass at registration
        # (BN-07 C-5); initialization itself must not run any updater.
        self.assertEqual(calls, [0.])
        before = obj.get_points().copy()
        m._initialize_raster_image(obj, resource())
        np.testing.assert_allclose(obj.get_points(), before, atol=1e-7)
        self.assertIs(obj.submobjects[0], decoration)
        self.assertIn(updater, obj.updaters)
        self.assertEqual(calls, [0.])
        snapshot = obj._engine_state()['snapshot']
        obj.set_z_index(7)
        self.assertEqual(obj._engine_state()['snapshot'], snapshot)
        obj.set_z_index(8)
        self.assertNotEqual(obj._engine_state()['snapshot'], snapshot)
        self.assertEqual(obj.uniforms['is_fixed_in_frame'], 1)

    def test_copy_and_old_capture_keep_the_original_pixels(self):
        obj = raw()
        m._initialize_raster_image(obj, resource())
        clone = obj.copy()
        frozen = m.Camera(resolution=(80, 64)).capture_snapshot(obj)
        before = capture(obj)
        m._initialize_raster_image(obj, resource(_PIXELS[:, ::-1].copy()))
        self.assertNotEqual(capture(obj), before)
        self.assertEqual(frozen.png(), before)
        for previous in (clone,):
            self.assertEqual(capture(previous), before)
            np.testing.assert_array_equal(previous.data['mass'], 13)

    def test_live_pixel_publication_and_transform_work_after_adoption(self):
        obj = raw()
        m._initialize_raster_image(obj, resource())
        scene = m.Scene()
        scene.add(obj)
        start = obj.get_points().copy()
        m._replace_raster_image(obj, resource(_PIXELS[:, ::-1].copy()))
        scene.play(obj.animate.shift(m.RIGHT), run_time=1/30, rate_func=m.linear)
        np.testing.assert_allclose(obj.get_points(), start + m.RIGHT, atol=1e-6)
        np.testing.assert_array_equal(obj.data['mass'], 13)
        self.assertEqual(m._read_raster_image(obj).pixels(), _PIXELS[:, ::-1].tobytes())

    def test_invalid_counts_are_rejected_without_a_partial_resource(self):
        for count in (0, 3, 5, 7):
            obj = raw(count)
            before = obj.data.copy()
            with self.subTest(count=count), self.assertRaisesRegex(ValueError, 'six records'):
                m._initialize_raster_image(obj, resource())
            np.testing.assert_array_equal(obj.data, before)
            with self.assertRaises(ValueError):
                m._read_raster_image(obj)

    def test_invalid_or_drifted_schemas_are_rejected_without_mutation(self):
        plain = m.Mobject()
        plain.resize(6)
        with self.assertRaisesRegex(ValueError, 'im_coords'):
            m._initialize_raster_image(plain, resource())
        obj = raw()
        obj.data_dtype = m.ImageMobject.data_dtype
        before = obj.data.copy()
        with self.assertRaisesRegex(ValueError, 'schema changed'):
            m._initialize_raster_image(obj, resource())
        np.testing.assert_array_equal(obj.data, before)

    def test_nonfinite_render_columns_refuse_and_custom_columns_are_not_reinterpreted(self):
        for key in ('point', 'im_coords', 'opacity'):
            obj = raw()
            obj.data[key][0, 0] = np.nan
            before = obj.data.copy()
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, 'finite'):
                m._initialize_raster_image(obj, resource())
            for field in before.dtype.names:
                np.testing.assert_array_equal(obj.data[field], before[field])
        obj = raw()
        obj.data['mass'][:] = np.nan
        m._initialize_raster_image(obj, resource())
        self.assertTrue(np.isnan(obj.data['mass']).all())

    def test_scene_owned_targets_refuse_and_keep_existing_material(self):
        obj = m.ImageMobject(_PIXELS)
        scene = m.Scene(); scene.add(obj)
        before = capture(obj)
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            m._initialize_raster_image(obj, resource(_PIXELS[:, ::-1].copy()))
        self.assertEqual(capture(obj), before)
        self.assertIs(obj._scene, scene)

    def test_schema_descriptor_cannot_change_ownership_before_publication(self):
        scene = m.Scene()
        class Authored(ImageRecords):
            @property
            def data_dtype(self):
                if getattr(self, 'adopt_on_schema', False):
                    self.adopt_on_schema = False
                    scene.add(self)
                return ImageRecords.data_dtype
        obj = Authored(); obj.resize(6)
        obj.adopt_on_schema = True
        with self.assertRaisesRegex(RuntimeError, 'ownership changed'):
            m._initialize_raster_image(obj, resource())
        self.assertIs(obj._scene, scene)  # Authored side effects are not rolled back.
        with self.assertRaises(ValueError):
            m._read_raster_image(obj)


result = unittest.TextTestRunner(verbosity=2).run(
    unittest.defaultTestLoader.loadTestsFromTestCase(RasterInitializationTests))
if not result.wasSuccessful():
    raise AssertionError('native image initialization acceptance failed')
