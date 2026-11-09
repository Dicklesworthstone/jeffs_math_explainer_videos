"""Real native array textures and atomic light/dark material replacement."""
from __future__ import annotations

from pathlib import Path
import pickle
import tempfile
from types import SimpleNamespace
import unittest
import numpy as np
import manimlib as m


def pixels(color, shape=(3, 5)):
    return np.broadcast_to(np.array(color, dtype=np.uint8), (*shape, 4)).copy()


def surface():
    return m.Surface(u_range=(-2, 2), v_range=(-2, 2), resolution=(3, 4))


def geometry():
    return SimpleNamespace(vertices=np.array([[-2,-2,0],[2,-2,0],[2,2,0],[-2,2,0]], dtype=float),
        faces=np.array([[0,1,2],[0,2,3]]),
        visual=SimpleNamespace(uv=np.array([[0,1],[1,1],[1,0],[0,0]], dtype=float)))


def capture(obj):
    camera = m.Camera(resolution=(80,64))
    return camera.capture_snapshot(obj)


class RasterTextureTests(unittest.TestCase):
    def setUp(self):
        self.red = pixels((255,0,0,255))
        self.blue = pixels((0,0,255,255), (2,7))
        self.green = pixels((0,255,0,255), (6,1))

    def test_in_memory_surface_pair_renders_both_sides_and_preserves_source(self):
        src = surface()
        before = src.data.copy()
        obj = m.TexturedSurface(src, self.red, self.blue, shading=(0,0,0))
        self.assertEqual(obj.num_textures, 2)
        self.assertIsNone(obj.image_file)
        np.testing.assert_array_equal(src.data, before)
        np.testing.assert_array_equal(obj.get_pixel_array(), self.red)
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True), self.blue)
        frame = np.frombuffer(capture(obj).pixels(), dtype=np.uint8).reshape(64,80,4)
        self.assertGreater(frame[32,40,0], 240)
        obj.rotate(m.PI, axis=m.UP)
        frame = np.frombuffer(capture(obj).pixels(), dtype=np.uint8).reshape(64,80,4)
        self.assertGreater(frame[32,40,2], 240)
        self.assertFalse(src._is_bound())

    def test_surface_live_pair_replacement_preserves_all_records_and_snapshots(self):
        obj = m.TexturedSurface(surface(), self.red, self.blue, shading=(0,0,0), opacity=.7)
        obj.rotate(.25, axis=m.RIGHT).shift((.2,.1,0))
        scene = m.Scene(); scene.add(obj)
        live, saved = obj.data, obj.data.copy()
        old = capture(obj)
        old_png = old.png()
        copied, serialized = obj.copy(), pickle.dumps(obj)
        ticks=[]
        obj.add_updater(lambda mob,dt: ticks.append(dt))
        initial_ticks=list(ticks)
        clock=scene.time
        self.assertIs(obj.set_pixel_array(self.green, dark_pixels=self.red), obj)
        np.testing.assert_array_equal(obj.data, saved)
        np.testing.assert_array_equal(live, saved)
        self.assertEqual(ticks, initial_ticks)
        self.assertEqual(scene.time, clock)
        self.assertEqual(obj.num_textures, 2)
        np.testing.assert_array_equal(obj.get_pixel_array(), self.green)
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True), self.red)
        self.assertNotEqual(capture(obj).png(), old_png)
        self.assertEqual(old.png(), old_png)
        for prior in (copied, pickle.loads(serialized)):
            np.testing.assert_array_equal(prior.get_pixel_array(dark=True), self.blue)
            self.assertEqual(capture(prior).png(), old_png)
        live['opacity'][:]=.5
        np.testing.assert_allclose(obj.data['opacity'], .5)

    def test_invalid_dark_input_never_publishes_a_half_updated_pair(self):
        obj = m.TexturedSurface(surface(), self.red, self.blue)
        before = capture(obj).png()
        attrs = (obj.image_file, obj.dark_image_file, obj.num_textures)
        for dark in (b'bad png', np.zeros((1,1,5),dtype=np.uint8), np.zeros((2,2,4))):
            with self.assertRaises((TypeError,ValueError)):
                obj.set_textures(self.green, dark)
            np.testing.assert_array_equal(obj.get_pixel_array(), self.red)
            np.testing.assert_array_equal(obj.get_pixel_array(dark=True), self.blue)
            self.assertEqual(capture(obj).png(), before)
            self.assertEqual((obj.image_file,obj.dark_image_file,obj.num_textures),attrs)
        self.assertIs(obj.set_textures(self.green),obj)
        self.assertEqual(obj.num_textures,1)
        with self.assertRaisesRegex(ValueError,'dark'):
            obj.get_pixel_array(dark=True)

    def test_geometry_construction_and_live_pair_use_native_mesh_normals_and_uvs(self):
        obj = m.TexturedGeometry(geometry(),self.red,shading=(0,0,0))
        records=obj.data.copy()
        self.assertEqual(obj.n_records(),6)
        obj.set_textures(self.green,self.blue)
        np.testing.assert_array_equal(obj.data,records)
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True),self.blue)
        self.assertEqual(obj.num_textures,2)
        obj.rotate(m.PI,axis=m.UP)
        frame=np.frombuffer(capture(obj).pixels(),dtype=np.uint8).reshape(64,80,4)
        self.assertGreater(frame[32,40,2],240)
        self.assertIsNone(obj.texture_file)

    def test_encoded_bytes_and_files_share_the_same_constructor_and_material_path(self):
        # A native image capture supplies actual encoded input without a host codec.
        encoded = capture(m.ImageMobject(self.red)).png()
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'image.png'; path.write_bytes(encoded)
            from_file=m.TexturedSurface(surface(),path,path,shading=(0,0,0))
            from_bytes=m.TexturedSurface(surface(),encoded,shading=(0,0,0))
            self.assertEqual(from_file.num_textures,1)
            self.assertEqual(from_file.image_file,str(path.resolve()))
            self.assertEqual(capture(from_file).png(),capture(from_bytes).png())
            from_bytes.set_textures(path,path)
            self.assertEqual(from_bytes.num_textures,1)
            self.assertEqual(capture(from_file).png(),capture(from_bytes).png())

    def test_failed_constructor_decode_preserves_existing_object_and_source(self):
        src=surface()
        obj=m.TexturedSurface(src,self.red,self.blue)
        source_before=src.data.copy(); object_before=obj.data.copy()
        with self.assertRaises(ValueError):
            obj.__init__(src,self.green,b'bad PNG')
        np.testing.assert_array_equal(src.data,source_before)
        np.testing.assert_array_equal(obj.data,object_before)
        np.testing.assert_array_equal(obj.get_pixel_array(),self.red)
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True),self.blue)

    def test_existing_file_backed_objects_accept_live_arrays_without_reinitializing(self):
        encoded = capture(m.ImageMobject(self.red)).png()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'old-image.png'
            path.write_bytes(encoded)
            objects = (m.TexturedSurface(surface(), path, shading=(0, 0, 0)),
                       m.TexturedGeometry(geometry(), path, shading=(0, 0, 0)))
        for obj in objects:
            original = capture(obj)
            frozen = original.png()
            records = obj.data.copy()
            view = obj.data
            # The original file no longer exists: replacement/readback/copy
            # must use retained native resources, never reopen source paths.
            obj.set_pixel_array(self.green, dark_pixels=self.blue)
            np.testing.assert_array_equal(obj.data, records)
            np.testing.assert_array_equal(view, records)
            np.testing.assert_array_equal(obj.get_pixel_array(), self.green)
            np.testing.assert_array_equal(obj.get_pixel_array(dark=True), self.blue)
            self.assertEqual(obj.num_textures, 2)
            self.assertEqual(original.png(), frozen)
            self.assertNotEqual(capture(obj).png(), frozen)
            obj.rotate(m.PI, axis=m.UP)
            frame = np.frombuffer(capture(obj).pixels(), dtype=np.uint8).reshape(64, 80, 4)
            self.assertGreater(frame[32, 40, 2], 240)
            records = obj.data.copy()
            for method in ('set_textures', 'set_pixel_array'):
                obj.set_pixel_array(self.green, dark_pixels=self.blue)
                animation = getattr(obj.animate(rate_func=m.linear), method)(self.red).build()
                animation.begin(); animation.interpolate(.5)
                expected = pixels((188, 188, 0, 255), (6,5))
                np.testing.assert_array_equal(obj.get_pixel_array(), expected)
                animation.finish()
                np.testing.assert_array_equal(obj.get_pixel_array(), self.red)
                np.testing.assert_array_equal(obj.data, records)

    def test_material_builder_input_failure_preserves_both_material_sides(self):
        for obj in (m.TexturedSurface(surface(), self.red, self.blue),
                    m.TexturedGeometry(geometry(), self.red)):
            before = obj.data.copy(), obj.get_pixel_array().copy()
            failure = ValueError('material conversion failed')
            class Explodes:
                def __array__(self, *args, **kwargs):
                    raise failure
            for method in ('set_textures', 'set_pixel_array'):
                with self.assertRaises(ValueError) as caught:
                    getattr(obj.animate, method)(Explodes())
                self.assertIs(caught.exception, failure)
            np.testing.assert_array_equal(obj.data, before[0])
            np.testing.assert_array_equal(obj.get_pixel_array(), before[1])

    def test_reentrant_material_conversion_cannot_replace_either_side(self):
        obj=m.TexturedSurface(surface(),self.red,self.blue)
        class Reentrant:
            def __array__(_, *args, **kwargs):
                obj.set_pixel_array(self.green)
                return self.red
        with self.assertRaisesRegex(RuntimeError,'reenter'):
            obj.set_pixel_array(self.green,Reentrant())
        np.testing.assert_array_equal(obj.get_pixel_array(),self.red)
        np.testing.assert_array_equal(obj.get_pixel_array(dark=True),self.blue)


result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(RasterTextureTests))
if not result.wasSuccessful():
    raise AssertionError('native raster texture acceptance failed')
