"""Native transition acceptance with independent color and geometry witnesses."""
from concurrent.futures import ThreadPoolExecutor
import struct
import unittest
import zlib

import numpy as np
import manimlib as m


def pixels(color, shape=(1, 1)):
    return np.full((*shape, 4), color, dtype=np.uint8)


def resource(array):
    h, w, _ = array.shape
    return m._RasterImage(w, h, array.tobytes())


def array(image):
    w, h = image.size
    return np.frombuffer(image.pixels(), dtype=np.uint8).reshape(h, w, 4)


def gamma_png(color):
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 1, 1, 8, 6, 0, 0, 0))
            + chunk(b'gAMA', struct.pack('>I', 100000))
            + chunk(b'IDAT', zlib.compress(b'\0' + bytes(color))) + chunk(b'IEND', b''))


class RasterTransitionKernelTests(unittest.TestCase):
    def test_linear_light_midpoint_not_encoded_rgb_midpoint(self):
        start = m.ImageMobject(pixels((0, 0, 0, 255)))
        end = resource(pixels((255, 255, 255, 255)))
        plan = m._RasterTransition(start, end)
        np.testing.assert_array_equal(array(plan.sample(.5)), pixels((188, 188, 188, 255)))
        np.testing.assert_array_equal(start.get_pixel_array(), pixels((0, 0, 0, 255)))

    def test_premultiplied_alpha_has_no_transparent_red_halo(self):
        start = m.ImageMobject(pixels((255, 0, 0, 0)))
        plan = m._RasterTransition(start, resource(pixels((0, 0, 255, 255))))
        np.testing.assert_array_equal(array(plan.sample(.5)), pixels((0, 0, 255, 128)))
        np.testing.assert_array_equal(array(plan.sample(0)), pixels((255, 0, 0, 0)))

    def test_mixed_input_transfer_is_decoded_before_blending_and_exact_at_endpoints(self):
        start = m.ImageMobject.from_bytes(gamma_png((128, 128, 128, 255)))
        saved = start.copy()
        self.assertFalse(m._raster_images_equal(start, m.ImageMobject(pixels((128, 128, 128, 255)))))
        camera = m.Camera(resolution=(24, 16))
        rendered = np.frombuffer(camera.capture_snapshot(start).pixels(), dtype=np.uint8).reshape(16,24,4)
        np.testing.assert_array_equal(rendered[8,12], [188,188,188,255])
        end = resource(pixels((0, 0, 0, 255)))
        plan = m._RasterTransition(start, end)
        # Linear 128/255 at half strength => sRGB 137, not encoded-space 64.
        np.testing.assert_array_equal(array(plan.sample(.5)), pixels((137, 137, 137, 255)))
        plan.apply(start, .5)
        plan.apply(start, 0)
        self.assertTrue(m._raster_images_equal(start, saved))
        plan.apply(start, 1)
        self.assertTrue(m._raster_images_equal(start, m.ImageMobject(end)))

    def test_differing_sizes_use_common_uv_lattice_without_resizing_geometry(self):
        start = m.ImageMobject(pixels((255, 0, 0, 255), (1, 3))).rotate(.2)
        original = start.data.copy()
        end = pixels((0, 0, 255, 255), (2, 1))
        plan = m._RasterTransition(start, resource(end))
        middle = plan.sample(.5)
        self.assertEqual(middle.size, (3, 2))
        np.testing.assert_array_equal(array(middle), pixels((188, 0, 188, 255), (2, 3)))
        plan.apply(start, .5)
        np.testing.assert_array_equal(start.data, original)
        plan.apply(start, 1)
        self.assertEqual((start.pixel_width, start.pixel_height), (1, 2))
        np.testing.assert_array_equal(start.get_pixel_array(), end)
        np.testing.assert_array_equal(start.data, original)

    def test_resampling_uses_bilinear_premultiplied_native_texture_sampler(self):
        first = pixels((0, 0, 0, 255), (1, 2))
        first[0, 1, :3] = 255
        obj = m.ImageMobject(first)
        plan = m._RasterTransition(obj, resource(pixels((0, 0, 0, 255), (1, 3))))
        # Center UV samples half black/white then half black => linear .25.
        np.testing.assert_array_equal(array(plan.sample(.5))[0, 1], [137, 137, 137, 255])

    def test_light_dark_pair_changes_as_one_resource_and_missing_side_falls_back_to_light(self):
        surface = m.Surface(resolution=(2, 2))
        obj = m.TexturedSurface(surface, pixels((255, 0, 0, 255)), pixels((0, 255, 0, 255)))
        start = obj.copy()
        plan = m._RasterTransition(obj, resource(pixels((0, 0, 255, 255))))
        plan.apply(obj, .5)
        np.testing.assert_array_equal(obj.get_pixel_array(), pixels((188, 0, 188, 255)))
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True), pixels((0, 188, 188, 255)))
        plan.apply(obj, 1)
        with self.assertRaises(ValueError):
            obj.get_pixel_array(dark=True)
        plan.apply(obj, 0)
        self.assertTrue(m._raster_images_equal(obj, start))
        # The reverse operation introduces a dark pair, not a sudden mid-frame switch.
        plain = m.TexturedSurface(surface, pixels((0, 0, 255, 255)))
        reverse = m._RasterTransition(plain, resource(pixels((255, 0, 0, 255))),
                                      resource(pixels((0, 255, 0, 255))))
        reverse.apply(plain, .5)
        self.assertTrue(m._raster_images_equal(plain, m.TexturedSurface(
            surface, pixels((188, 0, 188, 255)), pixels((0, 188, 188, 255)))))

    def test_bound_publication_preserves_views_clock_updaters_and_frozen_snapshots(self):
        obj = m.ImageMobject(pixels((255, 0, 0, 255)))
        scene = m.Scene(); scene.add(obj)
        scene.camera.reset_pixel_shape(32, 24)
        callbacks = []
        obj.add_updater(lambda mob, dt: callbacks.append(dt))
        before = len(callbacks)
        view, records, clock = obj.data, obj.data.copy(), scene.time
        frozen = scene.camera.capture_snapshot(obj); png = frozen.png()
        plan = m._RasterTransition(obj, resource(pixels((0, 0, 255, 255))))
        plan.apply(obj, .5)
        self.assertEqual(scene.time, clock)
        self.assertEqual(len(callbacks), before)
        np.testing.assert_array_equal(view, records)
        self.assertEqual(frozen.png(), png)
        self.assertNotEqual(scene.camera.capture_snapshot(obj).png(), png)

    def test_plan_is_immutable_reusable_and_independent_of_later_target_edits(self):
        obj = m.ImageMobject(pixels((255, 0, 0, 255)))
        plan = m._RasterTransition(obj, resource(pixels((0, 0, 255, 255))))
        obj.set_pixel_array(pixels((0, 255, 0, 255)))
        expected = plan.sample(.5).pixels()
        with ThreadPoolExecutor(max_workers=4) as pool:
            actual = list(pool.map(lambda _: plan.sample(.5).pixels(), range(12)))
        self.assertEqual(actual, [expected] * 12)
        plan.apply(obj, .5)
        self.assertEqual(obj.get_pixel_array().tobytes(), expected)

    def test_nonfinite_alpha_and_incompatible_targets_refuse_without_publication(self):
        obj = m.ImageMobject(pixels((255, 0, 0, 255)))
        end = resource(pixels((0, 0, 255, 255)))
        with self.assertRaises(ValueError): m._RasterTransition(m.Square(), end)
        with self.assertRaises(ValueError): m._RasterTransition(obj, end, end)
        plan = m._RasterTransition(obj, end)
        saved = obj.copy()
        for alpha in (float('nan'), float('inf'), -float('inf')):
            with self.assertRaises(ValueError): plan.apply(obj, alpha)
            self.assertTrue(m._raster_images_equal(obj, saved))
        self.assertEqual(plan.sample(-1).pixels(), saved.get_pixel_array().tobytes())
        self.assertEqual(plan.sample(2).pixels(), end.pixels())

    def test_crossed_aspect_ratios_refuse_before_allocating_large_output(self):
        obj = m.ImageMobject(pixels((255, 0, 0, 255), (1, 4097)))
        end = resource(pixels((0, 0, 255, 255), (4097, 1)))
        with self.assertRaisesRegex(ValueError, 'output.*budget'):
            m._RasterTransition(obj, end)
        self.assertEqual(obj.pixel_width, 4097)


result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(RasterTransitionKernelTests))
if not result.wasSuccessful():
    raise AssertionError('native raster transition acceptance failed')
