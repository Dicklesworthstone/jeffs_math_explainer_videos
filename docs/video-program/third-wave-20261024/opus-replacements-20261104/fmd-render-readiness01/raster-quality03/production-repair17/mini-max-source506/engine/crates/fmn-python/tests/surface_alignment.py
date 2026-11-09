"""Native UV topology, field interpolation, live views and actual rendered morphs.

Run against the installed portal or the embedded test initializer. There is no
stand-in UV resampler, rendering mock, or missing-extension skip.
"""
import io
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

import numpy as np
import manimlib as m


def surface(shape=(2, 2), height=0., **kwargs):
    return m.ParametricSurface(lambda u, v: (u, v, height + u * v),
                              u_range=(-1, 1), v_range=(-1, 1),
                              resolution=shape, shading=(0, 0, 0), **kwargs)


def grid(shape, height=0.):
    u, v = np.meshgrid(np.linspace(-1, 1, shape[0]), np.linspace(-1, 1, shape[1]), indexing='ij')
    return np.stack((u, v, height + u * v), axis=-1).reshape(-1, 3)


def texture(path):
    def chunk(kind, body):
        return struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind + body))
    raw = b'\0' + bytes((255, 0, 0, 255, 0, 255, 0, 255))
    raw += b'\0' + bytes((0, 0, 255, 255, 255, 255, 255, 255))
    path.write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 2, 2, 8, 6, 0, 0, 0))
                     + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b''))
    return path


class SurfaceAlignmentTests(unittest.TestCase):
    def assert_topology(self, obj, shape):
        self.assertEqual(tuple(obj.resolution), shape)
        self.assertEqual(m._surface_grid_resolution(obj), shape)
        self.assertEqual(obj.get_num_points(), shape[0] * shape[1])
        triangles = obj.get_triangle_indices()
        self.assertEqual(len(triangles), 6 * (shape[0] - 1) * (shape[1] - 1))
        self.assertEqual(int(np.min(triangles)), 0)
        self.assertEqual(int(np.max(triangles)), obj.get_num_points() - 1)
        self.assertEqual(obj.get_uv_grid().shape, (*shape, 2))

    def test_surface_group_empty_roots_align_their_drawable_leaves(self):
        for left_type, right_type in ((m.SGroup, m.SGroup), (m.SGroup, m.Group),
                                      (m.Group, m.SGroup)):
            with self.subTest(left=left_type.__name__, right=right_type.__name__):
                a, b = surface((2, 3)), surface((3, 2), 2)
                left, right = left_type(a), right_type(b)
                target = b.data.copy()
                self.assertFalse(left.is_aligned_with(right))
                scene = m.Scene()
                scene.add(left)
                scene.play(m.Transform(left, right), run_time=.125, rate_func=m.linear)
                self.assertEqual(left.get_num_points(), 0)
                self.assertIsNone(m._surface_grid_resolution(left))
                self.assertIs(left.submobjects[0], a)
                self.assert_topology(a, (3, 3))
                np.testing.assert_allclose(a.get_points(), grid((3, 3), 2), atol=2e-6)
                np.testing.assert_array_equal(b.data, target)

    def test_empty_surface_group_can_align_and_transform_without_a_fake_grid(self):
        left, right = m.SGroup(), m.SGroup()
        self.assertTrue(left.is_aligned_with(right))
        self.assertIs(left.align_points(right), left)
        scene = m.Scene()
        scene.add(left)
        scene.play(m.Transform(left, right), run_time=.125)
        self.assertEqual(left.get_num_points(), 0)
        self.assertIsNone(m._surface_grid_resolution(left))

    def test_surface_group_builder_and_saved_state_remain_usable(self):
        obj = surface((2, 3))
        group = m.SGroup(obj)
        group.save_state()
        scene = m.Scene()
        scene.add(group)
        scene.play(group.animate.shift((0, 0, 2)), run_time=.125, rate_func=m.linear)
        np.testing.assert_allclose(obj.get_points(), grid((2, 3), 2), atol=2e-6)
        scene.play(m.Restore(group), run_time=.125, rate_func=m.linear)
        np.testing.assert_allclose(obj.get_points(), grid((2, 3)), atol=2e-6)
        self.assert_topology(obj, (2, 3))

    def test_direct_alignment_uses_both_uv_dimensions(self):
        left, right = surface((2, 3)), surface((4, 2), 2)
        self.assertIs(left.align_points(right), left)
        for obj, height in ((left, 0), (right, 2)):
            self.assert_topology(obj, (4, 3))
            np.testing.assert_allclose(obj.get_points(), grid((4, 3), height), atol=2e-6)
            np.testing.assert_allclose(obj.uv_to_point(.2, -.3), (.2, -.3, height - .06), atol=2e-6)

    def test_equal_record_counts_are_not_equal_topology(self):
        left, right = surface((2, 3)), surface((3, 2), 2)
        self.assertFalse(left.is_aligned_with(right))
        before = right.data.copy()
        animation = m.Transform(left, right, rate_func=m.linear)
        animation.begin()
        self.assertIsNot(animation.target_copy, right)
        animation.interpolate(.5)
        self.assert_topology(left, (3, 3))
        np.testing.assert_allclose(left.get_points(), grid((3, 3), 1), atol=2e-6)
        animation.finish()
        self.assert_topology(right, (3, 2))
        np.testing.assert_array_equal(right.data, before)

    def test_scene_play_updates_queries_and_preserves_target(self):
        left, right = surface(), surface((3, 4), 2)
        before = right.data.copy()
        scene = m.Scene()
        scene.add(left)
        scene.play(m.Transform(left, right), run_time=.25, rate_func=m.linear)
        self.assert_topology(left, (3, 4))
        np.testing.assert_allclose(left.get_points(), grid((3, 4), 2), atol=2e-6)
        np.testing.assert_array_equal(right.data, before)
        np.testing.assert_allclose(left.uv_to_point(.25, .5), (.25, .5, 2.125), atol=2e-6)
        self.assertIs(left._scene, scene)
        self.assertEqual(tuple(scene.mobjects), (left,))

    def test_every_normal_and_color_lane_is_aligned(self):
        left, right = surface(), surface((3, 4), 2)
        for obj in (left, right):
            p = np.asarray(obj.get_points()).copy()
            obj.data['d_normal_point'][:] = p + (0, 0, .25)
            obj.data['rgba'][:] = np.column_stack(((p[:, 0] + 1) / 2, (p[:, 1] + 1) / 2,
                                                   np.full(len(p), .25), np.ones(len(p))))
        animation = m.Transform(left, right, rate_func=m.linear)
        animation.begin()
        for alpha in (0, .25, .75, 1, .5):
            animation.interpolate(alpha)
            p = grid((3, 4), 2 * alpha)
            np.testing.assert_allclose(left.get_points(), p, atol=2e-6)
            np.testing.assert_allclose(left.data['d_normal_point'], p + (0, 0, .25), atol=2e-6)
            np.testing.assert_allclose(left.data['rgba'][:, 0], (p[:, 0] + 1) / 2, atol=2e-6)
            np.testing.assert_allclose(left.data['rgba'][:, 1], (p[:, 1] + 1) / 2, atol=2e-6)
        animation.finish()

    def test_resized_views_detach_but_new_views_follow_playback(self):
        left, right = surface(), surface((3, 4), 2)
        old_view = left.get_points()
        before = old_view.copy()
        left.save_state()
        animation = m.Transform(left, right, rate_func=m.linear)
        animation.begin()
        live = left.get_points()
        animation.interpolate(.5)
        np.testing.assert_array_equal(old_view, before)
        np.testing.assert_allclose(live, grid((3, 4), 1), atol=2e-6)
        old_view[0] = (20, 20, 20)
        np.testing.assert_allclose(left.get_points(), grid((3, 4), 1), atol=2e-6)
        self.assert_topology(left.saved_state, (2, 2))
        animation.finish()

    def test_equal_shape_keeps_live_record_generation(self):
        left, right = surface((3, 4)), surface((3, 4), 2)
        live = left.get_points()
        indices = left.get_triangle_indices()
        indices[:] = indices[::-1].copy()
        before_indices = indices.copy()
        left.align_points(right)
        self.assertIs(left.get_triangle_indices(), indices)
        np.testing.assert_array_equal(left.get_triangle_indices(), before_indices)
        left.shift((0, 0, 3))
        np.testing.assert_allclose(live, grid((3, 4), 3), atol=2e-6)

    def test_alignment_never_calls_uv_recipes(self):
        left, right = surface(), surface((3, 4), 2)
        def refuse(*args):
            raise AssertionError('a frozen UV grid must not be regenerated')
        left.passed_uv_func = right.passed_uv_func = refuse
        animation = m.Transform(left, right, rate_func=m.linear)
        animation.begin()
        animation.finish()
        np.testing.assert_allclose(left.get_points(), grid((3, 4), 2), atol=2e-6)

    def test_bound_and_detached_owners_both_directions(self):
        for bound_left in (True, False):
            with self.subTest(bound_left=bound_left):
                left, right = surface((2, 3)), surface((4, 2), 2)
                scene = m.Scene()
                scene.add(left if bound_left else right)
                left.align_points(right)
                self.assert_topology(left, (4, 3))
                self.assert_topology(right, (4, 3))
                self.assertEqual(left._is_bound(), bound_left)
                self.assertEqual(right._is_bound(), not bound_left)

    def test_same_scene_alignment_preserves_children_and_updaters(self):
        left, right, child = surface((2, 3)), surface((4, 2), 2), m.Dot()
        left.add(child)
        calls = []
        callback = lambda obj, dt: calls.append(dt)
        left.add_updater(callback, call=False)
        scene = m.Scene()
        scene.add(left, right)
        left.align_points(right)
        self.assertEqual(tuple(scene.mobjects), (left, right))
        self.assertIs(left.submobjects[0], child)
        self.assertIs(left.updaters[0], callback)
        self.assertEqual(calls, [])
        self.assertEqual(scene.get_time(), 0)

    def test_nested_group_transforms_publish_surface_metadata(self):
        source, target = surface(), surface((3, 4), 2)
        left, right = m.Group(m.Group(source)), m.Group(m.Group(target))
        scene = m.Scene()
        scene.add(left)
        scene.play(m.Transform(left, right), run_time=.25, rate_func=m.linear)
        self.assertIs(left[0][0], source)
        self.assert_topology(source, (3, 4))
        np.testing.assert_allclose(source.get_points(), grid((3, 4), 2), atol=2e-6)

    def test_succession_and_animate_use_current_grid(self):
        obj = surface()
        scene = m.Scene()
        scene.add(obj)
        scene.play(m.Succession(m.Transform(obj, surface((3, 4), 1), run_time=.25),
                                m.Transform(obj, surface((4, 3), 2), run_time=.25)))
        self.assert_topology(obj, (4, 4))
        np.testing.assert_allclose(obj.get_points(), grid((4, 4), 2), atol=2e-6)
        scene.play(obj.animate.shift((0, 0, 1)), run_time=.25, rate_func=m.linear)
        self.assert_topology(obj, (4, 4))
        np.testing.assert_allclose(obj.get_points(), grid((4, 4), 3), atol=2e-6)

    def test_replacement_keeps_authored_target_topology(self):
        source, target = surface((2, 3)), surface((3, 2), 2)
        before = target.data.copy()
        scene = m.Scene()
        scene.add(source)
        scene.play(m.ReplacementTransform(source, target), run_time=.25)
        self.assertEqual(tuple(scene.mobjects), (target,))
        self.assert_topology(target, (3, 2))
        np.testing.assert_array_equal(target.data, before)

    def test_authored_path_applies_to_pointlikes_not_colors(self):
        source, target = surface(), surface((3, 4), 2)
        source.data['d_normal_point'][:] = source.get_points() + (0, 0, .25)
        target.data['d_normal_point'][:] = target.get_points() + (0, 0, .25)
        source.set_color(m.RED)
        target.set_color(m.BLUE)
        rgba = (source.data['rgba'][0].copy() + target.data['rgba'][0].copy()) / 2
        calls = []
        def path(start, end, alpha):
            calls.append(start.shape)
            return (1 - alpha) * start + alpha * end + np.array((0, 0, 4 * alpha * (1 - alpha)))
        animation = m.Transform(source, target, path_func=path, rate_func=m.linear)
        animation.begin()
        animation.interpolate(.5)
        self.assertTrue(all(shape[-1] == 3 for shape in calls))
        np.testing.assert_allclose(source.get_points(), grid((3, 4), 2), atol=2e-6)
        np.testing.assert_allclose(source.data['d_normal_point'], grid((3, 4), 2.25), atol=2e-6)
        np.testing.assert_allclose(source.data['rgba'], np.tile(rgba, (12, 1)), atol=2e-6)
        animation.finish()

    def test_invalid_peer_leaves_both_records_and_metadata_unchanged(self):
        for bad in ('count', 'numeric', 'budget'):
            with self.subTest(bad=bad):
                left, right = surface(), surface((3, 4), 2)
                if bad == 'count':
                    right.resize_points(11)
                elif bad == 'numeric':
                    right.data['rgba'][0, 0] = np.nan
                else:
                    left, right = surface((513, 2)), surface((2, 513))
                before = [obj.data.copy() for obj in (left, right)]
                shapes = [obj.resolution for obj in (left, right)]
                with self.assertRaises((TypeError, ValueError)):
                    left.align_points(right)
                for obj, records, shape in zip((left, right), before, shapes):
                    for key in records.dtype.names:
                        np.testing.assert_array_equal(obj.data[key], records[key])
                    self.assertEqual(obj.resolution, shape)

    def test_unstructured_peer_refuses_without_flattening(self):
        left, right = surface(), m.Mobject()
        for a, b in ((left, right), (right, left)):
            self.assertFalse(a.is_aligned_with(b))
            with self.assertRaises(TypeError):
                a.align_points(b)
        self.assert_topology(left, (2, 2))
        self.assertEqual(right.get_num_points(), 0)

    def test_textured_uv_fields_and_decoded_material_survive(self):
        with tempfile.TemporaryDirectory(prefix='fmn-uv-texture-') as directory:
            path = texture(Path(directory) / 'texture.png')
            left = m.TexturedSurface(surface((2, 3)), path)
            right = m.TexturedSurface(surface((3, 2), 2), path)
            expected = [(u, v) for u in np.linspace(0, 1, 3) for v in np.linspace(0, 1, 3)]
            # An authored texture map is data, not something to regenerate from
            # a filename or from the new resolution during a morph.
            for obj in (left, right):
                points = np.asarray(obj.get_points())
                obj.data['im_coords'][:] = (points[:, :2] + 1) / 2
            scene = m.Scene()
            scene.add(left)
            scene.play(m.Transform(left, right), run_time=.25)
            self.assert_topology(left, (3, 3))
            np.testing.assert_allclose(left.data['im_coords'], expected, atol=2e-6)
            self.assertEqual(left.image_file, str(path.resolve()))
            self.assertEqual(left.num_textures, 1)

    def test_rendered_morph_has_valid_uv_queries_and_identical_thread_bytes(self):
        samples = []
        class Morph(m.Scene):
            def construct(self):
                obj = surface((2, 3), color=m.RED)
                target = surface((3, 2), 1, color=m.BLUE)
                self.add(obj)
                obj.add_updater(lambda s, dt: samples.append((s.resolution, s.uv_to_point(0, 0).copy())), call=False)
                self.play(m.Transform(obj, target), run_time=.5, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-uv-morph-') as directory:
            results = []
            for threads in (1, 4):
                path = Path(directory) / f'morph-{threads}.y4m'
                receipt = Morph().render(path, format='y4m', resolution=(96, 54), fps=8, threads=threads)
                self.assertEqual(receipt.frame_count, 4)
                results.append(path.read_bytes())
            self.assertEqual(results[0], results[1])
            header, payload = results[0].split(b'\n', 1)
            self.assertTrue(header.startswith(b'YUV4MPEG2'))
            frame_size = 96 * 54 * 3 // 2
            frames = [payload[i + 6:i + 6 + frame_size] for i in range(0, len(payload), frame_size + 6)]
            self.assertEqual(len(frames), 4)
            self.assertNotEqual(frames[0], frames[-1])
        self.assertTrue(samples)
        self.assertTrue(all(tuple(shape) == (3, 3) for shape, point in samples))
        self.assertTrue(all(np.isfinite(point).all() for shape, point in samples))


def run_surface_alignment_acceptance():
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(SurfaceAlignmentTests))
    if not result.wasSuccessful():
        raise AssertionError(stream.getvalue())
    print(stream.getvalue())


if __name__ == '__main__':
    run_surface_alignment_acceptance()
