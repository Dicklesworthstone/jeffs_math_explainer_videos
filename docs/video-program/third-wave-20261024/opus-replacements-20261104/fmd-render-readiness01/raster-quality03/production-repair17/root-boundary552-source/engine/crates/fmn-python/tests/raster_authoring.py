"""Real native image resources, pixels, copies, views and scene snapshots."""
from __future__ import annotations

import pickle
import struct
import tempfile
from pathlib import Path
import unittest
import zlib

import numpy as np
import manimlib as m


def png(pixels):
    height, width, _ = pixels.shape
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
    rows = b''.join(b'\0' + row.tobytes() for row in pixels)
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b''))


def pixels(color=(255, 0, 0, 255), shape=(3, 5)):
    result = np.empty((*shape, 4), dtype=np.uint8)
    result[:] = color
    return result


def capture(image):
    camera = m.Camera(resolution=(80, 64))
    return camera.capture_snapshot(image)


class RasterAuthoringTests(unittest.TestCase):
    def test_array_and_file_constructors_produce_identical_native_records_and_pixels(self):
        data = pixels()
        data[0, 1] = (0, 255, 0, 91)
        data[-1, -1] = (0, 0, 255, 255)
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'image.png'
            path.write_bytes(png(data))
            old = m.ImageMobject(path, height=3, opacity=.6)
            for image in (m.ImageMobject(data, height=3, opacity=.6),
                          m.ImageMobject.from_pixel_array(data, height=3, opacity=.6),
                          m.ImageMobject.from_bytes(path.read_bytes(), height=3, opacity=.6)):
                np.testing.assert_array_equal(image.data, old.data)
                np.testing.assert_array_equal(image.get_pixel_array(), data)
                self.assertEqual(capture(image).png(), capture(old).png())
                self.assertEqual((image.pixel_width, image.pixel_height), (5, 3))
                self.assertAlmostEqual(image.get_width(), 5)
                self.assertFalse(image._is_bound())

    def test_grayscale_gray_alpha_rgb_and_reversed_strides_are_explicit(self):
        gray = np.arange(15, dtype=np.uint8).reshape(3, 5)
        for value in (gray, gray[..., None], np.stack((gray, gray+20), axis=-1),
                      np.stack((gray, gray+10, gray+20), axis=-1), pixels()[::-1, ::-1]):
            image = m.ImageMobject.from_pixel_array(value)
            array = image.get_pixel_array()
            channels = 1 if value.ndim == 2 else value.shape[-1]
            if channels < 3:
                for c in range(3):
                    np.testing.assert_array_equal(array[..., c], gray)
                np.testing.assert_array_equal(array[..., 3], gray+20 if channels == 2 else 255)
            else:
                np.testing.assert_array_equal(array[..., :3], value[..., :3])
                if channels == 4:
                    np.testing.assert_array_equal(array, value)

    def test_input_and_readback_arrays_cannot_change_native_pixels(self):
        data = pixels()
        image = m.ImageMobject(data)
        before = capture(image).png()
        data[:] = 0
        readback = image.get_pixel_array()
        self.assertTrue(readback.flags.writeable)
        readback[:] = 0
        self.assertEqual(capture(image).png(), before)
        np.testing.assert_array_equal(image.get_pixel_array(), pixels())

    def test_update_changes_only_image_resource_and_preserves_bound_geometry_views(self):
        image = m.ImageMobject(pixels(), height=2, opacity=.4)
        image.rotate(.3).shift((1, .5, 0))
        marker = m.Point((2, 2, 0))
        image.add(marker)
        image.note = object()
        scene = m.Scene()
        scene.add(image)
        live = image.data
        before, identity, roots = live.copy(), image.note, list(scene.mobjects)
        count = []
        image.add_updater(lambda mob, dt: count.append(dt))
        initial_ticks = list(count)
        updater = image.updaters[0]
        clock = scene.time
        old_png = capture(image).png()
        self.assertIs(image.set_pixel_array(pixels((0, 0, 255, 255), (7, 2))), image)
        np.testing.assert_array_equal(image.data, before)
        np.testing.assert_array_equal(live, before)
        self.assertIs(image.submobjects[0], marker)
        self.assertIs(image.note, identity)
        self.assertIs(image.updaters[0], updater)
        self.assertEqual(count, initial_ticks)
        self.assertEqual(scene.time, clock)
        self.assertEqual(scene.mobjects, roots)
        self.assertEqual((image.pixel_width, image.pixel_height), (2, 7))
        self.assertNotEqual(capture(image).png(), old_png)
        live['opacity'][:] = .8
        np.testing.assert_allclose(image.data['opacity'], .8)

    def test_native_image_copy_pickle_and_old_capture_retain_previous_resource(self):
        image = m.ImageMobject(pixels())
        image.shift((.25, 0, 0))
        clone = image.copy()
        serialized = pickle.dumps(image)
        snapshot = capture(image)
        before = snapshot.png()
        image.set_image(png(pixels((0, 255, 0, 255))))
        self.assertEqual(snapshot.png(), before)
        for old in (clone, pickle.loads(serialized)):
            np.testing.assert_array_equal(old.get_pixel_array(), pixels())
            self.assertEqual(capture(old).png(), before)
        self.assertNotEqual(capture(image).png(), before)

    def test_scene_checkpoint_restore_rewinds_native_image_content(self):
        image = m.ImageMobject(pixels())
        scene = m.Scene()
        scene.add(image)
        old = scene.get_state()
        expected = capture(image).png()
        image.set_pixel_array(pixels((0, 0, 255, 255), (1, 2)))
        self.assertNotEqual(capture(image).png(), expected)
        old.restore_scene(scene)
        np.testing.assert_array_equal(image.get_pixel_array(), pixels())
        self.assertEqual(capture(image).png(), expected)

    def test_save_restore_and_become_carry_pixels_not_only_point_records(self):
        image = m.ImageMobject(pixels())
        image.save_state()
        image.set_image(pixels((0, 255, 0, 255)))
        image.restore()
        np.testing.assert_array_equal(image.get_pixel_array(), pixels())
        other = m.ImageMobject(pixels((0, 0, 255, 255)))
        image.become(other)
        np.testing.assert_array_equal(image.get_pixel_array(), other.get_pixel_array())

    def test_invalid_inputs_leave_resource_records_and_metadata_untouched(self):
        image = m.ImageMobject(pixels())
        before = capture(image).png()
        attrs = (image.image_path, image.pixel_width, image.pixel_height)
        for bad in (np.ones((2, 2, 4)), np.zeros((2, 2, 3), dtype=np.int16),
                    np.zeros((0, 2, 4), dtype=np.uint8), np.zeros((2, 2, 5), dtype=np.uint8),
                    np.zeros((2, 3, 4, 5), dtype=np.uint8), b'not PNG or JPEG'):
            with self.subTest(shape=getattr(bad, 'shape', None)), self.assertRaises((TypeError, ValueError)):
                image.set_image(bad)
            self.assertEqual(capture(image).png(), before)
            self.assertEqual((image.image_path, image.pixel_width, image.pixel_height), attrs)
        huge = np.broadcast_to(np.zeros((1, 1, 4), dtype=np.uint8), (1, 16_777_217, 4))
        with self.assertRaisesRegex(ValueError, 'budget'):
            image.set_pixel_array(huge)
        with self.assertRaises(ValueError):
            m._RasterImage(2, 2, b'bad')
        with self.assertRaises(ValueError):
            m._RasterImage(0, 2, b'')
        with self.assertRaises(ValueError):
            m._RasterImage(16_777_217, 1, b'')

    def test_reinitializing_bound_image_refuses_before_array_evaluation(self):
        scene = m.Scene()
        image = m.ImageMobject(pixels())
        scene.add(image)
        class Explodes:
            def __array__(self, *args, **kwargs):
                raise AssertionError('unexpected conversion')
        with self.assertRaisesRegex(RuntimeError, 'set_image'):
            image.__init__(Explodes())
        np.testing.assert_array_equal(image.get_pixel_array(), pixels())

    def test_subclass_and_qualified_alias_identity_are_preserved(self):
        from manimlib.mobject.types.image_mobject import ImageMobject
        self.assertIs(ImageMobject, m.ImageMobject)
        class Annotated(m.ImageMobject):
            def __init__(self, source, **kwargs):
                super().__init__(source, **kwargs)
                self.note = 'authored'
        image = Annotated.from_pixel_array(pixels())
        self.assertIsInstance(image, Annotated)
        self.assertEqual(image.note, 'authored')

    def test_native_resource_dimensions_follow_restore_become_and_pickle(self):
        image = m.ImageMobject(pixels(shape=(2, 3)))
        image.save_state()
        image.set_pixel_array(pixels(shape=(7, 1)))
        self.assertEqual((image.pixel_width, image.pixel_height), (1, 7))
        image.restore()
        self.assertEqual((image.pixel_width, image.pixel_height), (3, 2))
        image.become(m.ImageMobject(pixels(shape=(3, 6))))
        self.assertEqual((image.pixel_width, image.pixel_height), (6, 3))
        # init_points must use the restored/current native aspect, not stale
        # constructor attributes. Other authored sizing controls stay unchanged.
        image.init_points()
        self.assertAlmostEqual(image.get_width() / image.get_height(), 2)
        for other in (image.copy(), pickle.loads(pickle.dumps(image))):
            self.assertEqual((other.pixel_width, other.pixel_height), (6, 3))
        with self.assertRaises(AttributeError):
            image.pixel_width = 99
        self.assertEqual(image.pixel_width, 6)

    def test_raster_builder_input_failure_preserves_pixels_without_advancing_scene(self):
        image = m.ImageMobject(pixels())
        scene = m.Scene()
        scene.add(image)
        before = scene.time, image.data.copy(), capture(image).png()
        failure = ValueError('authored raster input failed')
        class Explodes:
            def __array__(self, *args, **kwargs):
                raise failure
            def __fspath__(self):
                raise failure
        for method in ('set_image', 'set_pixel_array'):
            with self.assertRaises(ValueError) as caught:
                getattr(image.animate, method)(Explodes())
            self.assertIs(caught.exception, failure)
        self.assertEqual(scene.time, before[0])
        np.testing.assert_array_equal(image.data, before[1])
        self.assertEqual(capture(image).png(), before[2])
        # Ordinary record/placement animation is still available for images.
        center = image.get_center().copy()
        scene.play(image.animate.shift((1, 0, 0)), run_time=.125, rate_func=m.linear)
        np.testing.assert_allclose(image.get_center(), center + (1, 0, 0), atol=1e-6)
        np.testing.assert_array_equal(image.get_pixel_array(), pixels())

    def test_reentrant_array_conversion_cannot_publish_inner_or_outer_pixels(self):
        image = m.ImageMobject(pixels())
        class Recursive:
            def __array__(self, *args, **kwargs):
                image.set_pixel_array(pixels((0, 0, 255, 255)))
                return pixels((0, 255, 0, 255))
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            image.set_pixel_array(Recursive())
        np.testing.assert_array_equal(image.get_pixel_array(), pixels())
        image.set_pixel_array(pixels((0, 0, 255, 255)))
        np.testing.assert_array_equal(image.get_pixel_array(), pixels((0, 0, 255, 255)))


result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(RasterAuthoringTests))
if not result.wasSuccessful():
    raise AssertionError('native raster authoring acceptance failed')
