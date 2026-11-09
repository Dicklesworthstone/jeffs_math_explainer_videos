"""Real native OBJ/MTL imports against independent mesh and pixel witnesses.

Only fixture PNG bytes are encoded here (stdlib); both imported and manually
assembled models use the real Atlas decoder, Marionette records and Lumen.
"""
from __future__ import annotations

import inspect
from pathlib import Path
import pickle
import struct
import tempfile
from types import SimpleNamespace
import unittest
import zlib

import numpy as np
import manimlib as m


def png(rgba):
    """Independent, original tiny test raster; no production codec substitution."""
    height, width, _ = rgba.shape
    def chunk(kind, data):
        return (struct.pack('>I', len(data)) + kind + data
                + struct.pack('>I', zlib.crc32(kind + data)))
    rows = b''.join(b'\0' + row.tobytes() for row in rgba)
    return (b'\x89PNG\r\n\x1a\n'
            + chunk(b'IHDR', struct.pack('>2I5B', width, height, 8, 6, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b''))


VERTICES = np.array([[-2., -1., 0.], [0., -1., 0.], [0., 1., 0.],
                     [-2., 1., 0.], [2., -1., 0.], [2., 1., 0.]])
GEOMETRY = ''.join('v %s %s %s\n' % tuple(point) for point in VERTICES)
GEOMETRY += 'vt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nvn 0 0 1\n'
LEFT = 'f 1/1/1 2/2/1 3/3/1 4/4/1\n'
RIGHT = 'f 2/1/1 5/2/1 6/3/1 3/4/1\n'


class ObjMaterialTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='fmn-obj-material-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.colors = np.array([[[255, 0, 0, 255], [0, 255, 0, 255]],
                                [[0, 0, 255, 255], [255, 255, 0, 128]]], dtype=np.uint8)
        self.write('materials/maps/tiles.png', png(self.colors))
        self.write('materials/scene.mtl', 'newmtl painted\nmap_Kd maps/tiles.png\n'
                   'newmtl plain\nKd 0.1 0.7 0.2\nd 0.8\n')
        self.path = self.write('model.obj', 'mtllib materials/scene.mtl\n' + GEOMETRY
                               + 'usemtl painted\n' + LEFT + 'usemtl plain\n' + RIGHT)

    def write(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data.encode() if isinstance(data, str) else data)
        return path

    def capture(self, *objects, threads=1):
        camera = m.Camera(resolution=(128, 80), background_opacity=0)
        camera.capture_threads = threads
        return camera.capture_snapshot(*objects)

    def test_constructor_identity_signature_material_parts_and_shared_normalization(self):
        from manimlib.mobject.types.surface import ThreeDModel
        self.assertIs(ThreeDModel, m.ThreeDModel)
        self.assertEqual(str(inspect.signature(ThreeDModel)), '(obj_file: str, height=3)')
        self.assertEqual(ThreeDModel.__bases__, (m.Group,))
        model = ThreeDModel(str(self.path), height=2)
        self.assertEqual(model.obj_file, str(self.path))
        self.assertEqual(model.height, 2)
        self.assertEqual([child.material_name for child in model], ['painted', 'plain'])
        self.assertIsInstance(model[0], m.TexturedGeometry)
        self.assertIsInstance(model[1], m.Surface)
        self.assertEqual(model[0].get_num_points(), 6)
        self.assertEqual(model[1].get_num_points(), 6)
        self.assertAlmostEqual(model[0].get_center()[0], -1)
        self.assertAlmostEqual(model[1].get_center()[0], 1)
        np.testing.assert_allclose(model.get_center(), (0, 0, 0), atol=1e-6)
        self.assertEqual(model.get_num_points(), 0)
        # Input-closure identities carry canonical paths (macOS /var ->
        # /private/var), one spelling per file.
        self.assertIn(str((self.root / 'materials/maps/tiles.png').resolve()), model.asset_paths)

    def test_texture_pixels_uvs_and_normals_match_independent_manual_mesh(self):
        model = m.ThreeDModel(self.path, height=2)
        geometry = SimpleNamespace(vertices=VERTICES[:4],
                                   faces=np.array([[0, 1, 2], [0, 2, 3]]),
                                   visual=SimpleNamespace(uv=np.array([[0., 0.], [1., 0.], [1., 1.], [0., 1.]])))
        expected = m.TexturedGeometry(geometry, str(self.root / 'materials/maps/tiles.png'),
                                      shading=(.3, .2, .4))
        np.testing.assert_array_equal(model[0].get_pixel_array(), self.colors)
        np.testing.assert_array_equal(model[0].data, expected.data)
        self.assertEqual(self.capture(model[0]).png(), self.capture(expected).png())
        self.assertGreater(len(set(self.capture(model[0]).pixels())), 3)

    def test_plain_material_dissolve_is_rendered_not_just_metadata(self):
        model = m.ThreeDModel(self.path, height=2)
        rgba = np.asarray(model[1].data['rgba'])
        np.testing.assert_allclose(rgba, np.tile([.1, .7, .2, .8], (6, 1)))
        before = self.capture(model[1]).png()
        model[1].set_color(m.RED)
        self.assertNotEqual(before, self.capture(model[1]).png())

    def test_repeated_material_runs_and_per_corner_uv_seams_are_retained(self):
        path = self.write('seams.obj', 'mtllib materials/scene.mtl\n' + GEOMETRY
                          + 'usemtl painted\nf 1/1/1 2/2/1 3/3/1\n'
                          + 'usemtl plain\n' + RIGHT
                          + 'usemtl painted\nf 1/4/1 3/3/1 4/2/1\n')
        model = m.ThreeDModel(path, height=2)
        self.assertEqual([child.material_name for child in model], ['painted', 'plain', 'painted'])
        np.testing.assert_array_equal(model[0].get_points()[0], model[2].get_points()[0])
        np.testing.assert_array_equal(model[0].data['im_coords'][0], [0, 1])
        np.testing.assert_array_equal(model[2].data['im_coords'][0], [0, 0])

    def test_untextured_obj_matches_existing_native_geometry_builder(self):
        path = self.write('plain.obj', GEOMETRY + LEFT + RIGHT)
        model = m.ThreeDModel(path, height=2)
        expected = m.Group()
        specs = expected._build_three_d_model(m._native_surface_shell_factory, path.read_bytes(), 2)
        m._hang_native_children(expected, specs)
        self.assertEqual(len(model), 1)
        np.testing.assert_array_equal(model[0].data, expected[0].data)
        self.assertEqual(self.capture(model).png(), self.capture(expected).png())

    def test_missing_required_inputs_and_uvs_preserve_existing_detached_model(self):
        model = m.ThreeDModel(self.path, height=2)
        child = model[0]
        pixels, records = self.capture(model).png(), child.data.copy()
        bad = self.write('missing.obj', 'mtllib missing.mtl\n' + GEOMETRY + LEFT)
        with self.assertRaises(FileNotFoundError):
            m.ThreeDModel.__init__(model, bad)
        self.assertIs(model[0], child)
        np.testing.assert_array_equal(child.data, records)
        self.assertEqual(self.capture(model).png(), pixels)
        self.write('missing.obj', 'mtllib materials/scene.mtl\n' + GEOMETRY + 'usemtl painted\nf 1 2 3\n')
        with self.assertRaisesRegex(ValueError, 'without UV'):
            m.ThreeDModel.__init__(model, bad)
        self.assertIs(model[0], child)
        self.assertEqual(self.capture(model).png(), pixels)

    def test_bound_reinitialization_and_reentrant_path_conversion_refuse(self):
        model = m.ThreeDModel(self.path, height=2)
        class Reentrant:
            def __fspath__(path_self):
                m.ThreeDModel.__init__(model, self.path)
                return str(self.path)
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            m.ThreeDModel.__init__(model, Reentrant())
        scene = m.Scene(); scene.add(model)
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            m.ThreeDModel.__init__(model, Reentrant())
        self.assertTrue(model._is_bound())

    def test_copies_pickle_and_native_snapshots_do_not_reopen_model_assets(self):
        model = m.ThreeDModel(self.path, height=2)
        expected = self.capture(model).png()
        copies = [model.copy(), pickle.loads(pickle.dumps(model))]
        # Moving the asset directory makes the original paths unavailable.
        (self.root / 'materials').rename(self.root / 'moved-materials')
        for copied in copies:
            self.assertEqual(self.capture(copied).png(), expected)
            np.testing.assert_array_equal(copied[0].get_pixel_array(), self.colors)
        model[0].set_pixel_array(np.full((3, 2, 4), 255, dtype=np.uint8))
        self.assertNotEqual(self.capture(model).png(), expected)
        self.assertEqual(self.capture(copies[0]).png(), expected)

    def test_group_placement_animation_and_scene_restore_keep_native_materials(self):
        model = m.ThreeDModel(self.path, height=2)
        scene = m.Scene(); scene.add(model)
        image = model[0].get_pixel_array().copy()
        checkpoint = scene.get_state()
        scene.play(model.animate.shift((1, 0, 0)), run_time=.1, rate_func=m.linear)
        np.testing.assert_array_equal(model[0].get_pixel_array(), image)
        self.assertAlmostEqual(model.get_center()[0], 1, places=5)
        scene.restore_state(checkpoint)
        self.assertAlmostEqual(model.get_center()[0], 0, places=5)
        np.testing.assert_array_equal(model[0].get_pixel_array(), image)

    def test_captures_match_across_thread_counts_and_do_not_tick_scene(self):
        model = m.ThreeDModel(self.path, height=2)
        scene = m.Scene(); scene.add(model)
        ticks = []; model.add_updater(lambda mob, dt: ticks.append(dt))
        count, clock = len(ticks), scene.time
        images = [self.capture(model, threads=n).png() for n in (1, 4, 16)]
        self.assertTrue(all(image == images[0] for image in images))
        self.assertEqual(scene.time, clock)
        self.assertEqual(len(ticks), count)

    def test_unsupported_map_transform_is_an_explicit_import_error(self):
        self.write('materials/scene.mtl', 'newmtl painted\nmap_Kd -s 2 2 1 maps/tiles.png\n')
        with self.assertRaisesRegex(ValueError, 'map options'):
            m.ThreeDModel(self.path)


result = unittest.TextTestRunner(verbosity=2).run(
    unittest.defaultTestLoader.loadTestsFromTestCase(ObjMaterialTests))
if not result.wasSuccessful():
    raise AssertionError('native OBJ material acceptance failed')
