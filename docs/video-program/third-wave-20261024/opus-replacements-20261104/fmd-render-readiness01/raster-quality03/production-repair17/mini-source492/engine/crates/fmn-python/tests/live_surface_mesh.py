"""Live SurfaceMesh acceptance on actual native records, callbacks and rendered frames."""
import copy
import io
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


def plane(shape=(5, 7), height=0):
    return m.ParametricSurface(lambda u, v: (u, v, height),
                              resolution=shape, u_range=(-2, 2), v_range=(-1, 1))


def records(mesh):
    return [wire.data.copy() for wire in mesh.submobjects]


class SurfaceMeshTests(unittest.TestCase):
    def assert_records(self, mesh, before):
        self.assertEqual(len(mesh.submobjects), len(before))
        for wire, expected in zip(mesh.submobjects, before):
            np.testing.assert_array_equal(wire.data, expected)

    def assert_planar_wires(self, mesh, density, height):
        wires = {vars(w)["_fmn_surface_wire"]: w for w in mesh.submobjects
                 if "_fmn_surface_wire" in vars(w)}
        self.assertEqual(len(wires), sum(density))
        for axis in range(2):
            for index, station in enumerate(np.linspace((-2, -1)[axis], (2, 1)[axis], density[axis])):
                points = wires[axis, index].get_points()
                np.testing.assert_allclose(points[:, axis], station, atol=2e-6)
                np.testing.assert_allclose(points[:, 2], height, atol=2e-6)
                np.testing.assert_allclose(points[0, 1-axis], (-1, -2)[axis], atol=2e-6)
                np.testing.assert_allclose(points[-1, 1-axis], (1, 2)[axis], atol=2e-6)

    def test_constructor_retains_recipe_controls_and_style_seed(self):
        mesh = m.SurfaceMesh(plane(), resolution=(3, 4), normal_nudge=.125,
                             stroke_color=m.RED, stroke_width=2, depth_test=False)
        self.assertEqual(mesh.resolution, (3, 4))
        self.assertEqual(mesh.normal_nudge, .125)
        self.assertEqual(mesh.get_stroke_color(), m.RED)
        self.assertEqual(mesh.get_stroke_width(), 2)
        self.assertFalse(mesh.uniforms['depth_test'])
        self.assert_planar_wires(mesh, (3, 4), .125)
        # fm-78b6: the point-less root's fill is VMobject's default, as the
        # pinned Reference reports it (#888888 at opacity 0).
        self.assertEqual((mesh.get_fill_color(), mesh.get_fill_opacity()), ('#888888', 0.0))

    def test_rebuild_follows_current_native_points_without_uv_evaluation(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(3, 4), normal_nudge=.125)
        old = tuple(mesh.submobjects)
        source.passed_uv_func = lambda *args: (_ for _ in ()).throw(AssertionError('UV must not execute'))
        for z in (1, 3, -2):
            source.get_points()[:, 2] = z
            source.data['d_normal_point'][:] = source.get_points() + (0, 0, .001)
            self.assertIsNone(mesh.init_points())
            self.assertEqual(tuple(mesh.submobjects), old)
            self.assert_planar_wires(mesh, (3, 4), z + .125)

    def test_normal_override_and_nonuniform_placement_are_observed(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(3, 4), normal_nudge=.25)
        source.stretch(2, 0).rotate(m.PI/4, axis=m.RIGHT).shift((1, 2, 3))
        calls = []
        def normals():
            calls.append(True)
            return np.tile((0, 0, -1), (source.get_num_points(), 1))
        source.get_unit_normals = normals
        mesh.init_points()
        self.assertEqual(calls, [True])
        expected = m.SurfaceMesh(source, resolution=(3, 4), normal_nudge=.25)
        for a, b in zip(mesh, expected):
            np.testing.assert_array_equal(a.get_points(), b.get_points())

    def test_bound_mesh_keeps_live_views_updaters_annotations_and_styles(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(3, 4))
        annotation = m.Dot()
        mesh.add(annotation)
        scene = m.Scene()
        scene.add(source, mesh)
        wires = tuple(mesh.submobjects[:-1])
        views = [wire.get_points() for wire in wires]
        wires[1].set_stroke(m.GREEN, width=3, opacity=.4)
        style = wires[1].data['stroke_rgba'].copy()
        widths = wires[1].data['stroke_width'].copy()
        calls = []
        callback = lambda obj, dt: calls.append(dt)
        wires[1].add_updater(callback, call=False)
        source.shift((0, 0, 2))
        mesh.init_points()
        self.assertEqual(tuple(scene.mobjects), (source, mesh))
        self.assertEqual(tuple(mesh.submobjects), (*wires, annotation))
        self.assertEqual(calls, [])
        self.assertIs(wires[1].updaters[0], callback)
        np.testing.assert_array_equal(wires[1].data['stroke_rgba'], style)
        np.testing.assert_array_equal(wires[1].data['stroke_width'], widths)
        for wire, view in zip(wires, views):
            self.assertIs(wire._scene, scene)
            np.testing.assert_array_equal(view, wire.get_points())
            np.testing.assert_allclose(view[:, 2], 2.01, atol=2e-6)

    def test_density_changes_reuse_survivors_and_detach_removed_lines(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(3, 4), stroke_color=m.RED, stroke_width=2,
                             depth_test=False, joint_type='bevel')
        annotation = m.Dot()
        mesh.add(annotation)
        scene = m.Scene()
        scene.add(mesh)
        old = {vars(w)['_fmn_surface_wire']: w for w in mesh.submobjects[:-1]}
        mesh.resolution = (5, 6)
        mesh.init_points()
        current = {vars(w)['_fmn_surface_wire']: w for w in mesh.submobjects if w is not annotation}
        for key, wire in old.items():
            self.assertIs(current[key], wire)
        for wire in current.values():
            self.assertEqual(wire.get_stroke_color(), m.RED)
            self.assertEqual(wire.get_stroke_width(), 2)
            self.assertEqual(wire.get_joint_type(), m.VMobject.joint_type_map['bevel'])
            self.assertFalse(wire.uniforms['depth_test'])
        self.assertIn(annotation, mesh.submobjects)
        mesh.resolution = (2, 1)
        mesh.init_points()
        self.assertEqual(len(mesh.submobjects), 4)
        self.assert_planar_wires(mesh, (2, 1), .01)
        self.assertNotIn(mesh, old[(0, 2)].parents)
        self.assertTrue(np.isfinite(old[(0, 2)].get_points()).all())
        self.assertEqual(tuple(scene.mobjects), (mesh,))

    def test_singleton_strip_and_empty_sources_keep_existing_native_semantics(self):
        for shape in ((0, 0), (0, 3), (3, 0), (1, 1), (1, 7), (5, 1)):
            with self.subTest(shape=shape):
                source = m.Surface(resolution=shape)
                mesh = m.SurfaceMesh(source, resolution=(3, 4))
                count = 7 if shape[0] * shape[1] else 0
                self.assertEqual(len(mesh.submobjects), count)
                before = tuple(mesh.submobjects)
                source.shift((0, 0, 2))
                mesh.init_points()
                self.assertEqual(tuple(mesh.submobjects), before)
                fresh = m.SurfaceMesh(source, resolution=(3, 4))
                for a, b in zip(mesh, fresh):
                    np.testing.assert_array_equal(a.get_points(), b.get_points())

    def test_zero_density_can_be_populated_later_using_the_root_style(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(0, 0), stroke_color=m.RED,
                             stroke_width=2, depth_test=False)
        self.assertEqual(len(mesh.submobjects), 0)
        mesh.resolution = (0, 3)
        mesh.init_points()
        self.assertEqual(len(mesh.submobjects), 3)
        for line in mesh:
            self.assertEqual(line.get_stroke_color(), m.RED)
            self.assertEqual(line.get_stroke_width(), 2)
            self.assertFalse(line.uniforms['depth_test'])

    def test_source_topology_changes_resize_wires_under_the_existing_view_protocol(self):
        source = plane((3, 4))
        mesh = m.SurfaceMesh(source, resolution=(2, 3))
        old = tuple(mesh.submobjects)
        views = [w.get_points() for w in old]
        before = [v.copy() for v in views]
        source.become(plane((7, 9), 2))
        mesh.init_points()
        self.assertEqual(tuple(mesh.submobjects), old)
        self.assert_planar_wires(mesh, (2, 3), 2.01)
        for view, data, wire in zip(views, before, old):
            np.testing.assert_array_equal(view, data)
            self.assertGreater(wire.get_num_points(), len(view))
        views[0][:] = 99
        self.assert_planar_wires(mesh, (2, 3), 2.01)

    def test_copy_keeps_external_surface_but_refreshes_only_its_own_wires(self):
        source = plane()
        original = m.SurfaceMesh(source, resolution=(3, 4))
        for copier in (lambda w: w.copy(), copy.copy, copy.deepcopy):
            with self.subTest(copier=copier):
                duplicate = copier(original)
                self.assertTrue(all(a is not b for a, b in zip(original, duplicate)))
                before = records(original)
                duplicate.uv_surface = plane(height=2)
                duplicate.init_points()
                self.assert_records(original, before)
                self.assert_planar_wires(duplicate, (3, 4), 2.01)

    def test_bound_and_detached_sources_can_refresh_without_adopting_each_other(self):
        for scene_source, scene_mesh in ((True, False), (False, True), (True, True)):
            with self.subTest(source=scene_source, mesh=scene_mesh):
                source = plane()
                mesh = m.SurfaceMesh(source, resolution=(3, 4))
                a, b = m.Scene(), m.Scene()
                if scene_source:
                    a.add(source)
                if scene_mesh:
                    b.add(mesh)
                source.shift((0, 0, 2))
                mesh.init_points()
                self.assertEqual(source._is_bound(), scene_source)
                self.assertEqual(mesh._is_bound(), scene_mesh)
                self.assert_planar_wires(mesh, (3, 4), 2.01)

    def test_invalid_controls_refuse_before_source_callbacks(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(3, 4))
        before = records(mesh)
        calls = []
        original = source.get_points
        source.get_points = lambda: calls.append(True) or original()
        for shape in ((-1, 2), (1.5, 2), (2, 3, 4), (65536, 1), (12000, 12000)):
            mesh.resolution = shape
            with self.subTest(shape=shape), self.assertRaises((ValueError, TypeError)):
                mesh.init_points()
            self.assert_records(mesh, before)
        mesh.resolution = (3, 4)
        mesh.normal_nudge = float('nan')
        with self.assertRaises(ValueError):
            mesh.init_points()
        self.assertEqual(calls, [])

    def test_bad_samples_and_callback_errors_publish_nothing_and_allow_retry(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(3, 4))
        before = records(mesh)
        original = source.get_unit_normals
        error = LookupError('authored normals')
        def broken():
            raise error
        source.get_unit_normals = broken
        with self.assertRaises(LookupError) as caught:
            mesh.init_points()
        self.assertIs(caught.exception, error)
        self.assert_records(mesh, before)
        source.get_unit_normals = lambda: np.full((source.n_records(), 3), float('nan'))
        with self.assertRaises(ValueError):
            mesh.init_points()
        self.assert_records(mesh, before)
        source.get_unit_normals = original
        source.shift((0, 0, 2))
        mesh.init_points()
        self.assert_planar_wires(mesh, (3, 4), 2.01)

    def test_reentrant_and_mutating_callbacks_do_not_overwrite_authored_effects(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(3, 4))
        original = source.get_unit_normals
        before = records(mesh)
        source.get_unit_normals = lambda: mesh.init_points() or original()
        with self.assertRaisesRegex(RuntimeError, 'already in progress'):
            mesh.init_points()
        self.assert_records(mesh, before)
        def mutate():
            mesh[0].shift((0, 0, 2))
            return original()
        source.get_unit_normals = mutate
        with self.assertRaisesRegex(RuntimeError, 'changed during regeneration'):
            mesh.init_points()
        np.testing.assert_allclose(mesh[0].get_points()[:, 2], 2.01, atol=2e-6)
        for wire, data in zip(mesh.submobjects[1:], before[1:]):
            np.testing.assert_array_equal(wire.data, data)

    def test_callback_source_changes_and_adoption_are_detected(self):
        for what in ('source', 'adopt', 'density'):
            with self.subTest(what=what):
                source = plane()
                mesh = m.SurfaceMesh(source, resolution=(3, 4))
                before = records(mesh)
                original = source.get_unit_normals
                scene = m.Scene()
                def change():
                    if what == 'source':
                        source.shift((0, 0, 2))
                    elif what == 'adopt':
                        scene.add(mesh)
                    else:
                        mesh.resolution = (4, 5)
                    return original()
                source.get_unit_normals = change
                with self.assertRaises(RuntimeError):
                    mesh.init_points()
                self.assert_records(mesh, before)

    def test_active_animation_and_locked_wire_refuse_regeneration(self):
        source = plane()
        mesh = m.SurfaceMesh(source, resolution=(3, 4))
        before = records(mesh)
        mesh[0].lock_data(['point'])
        with self.assertRaisesRegex(RuntimeError, 'active animation'):
            mesh.init_points()
        self.assert_records(mesh, before)
        mesh[0].unlock_data()
        mesh._is_animating = True
        with self.assertRaises(RuntimeError):
            mesh.init_points()
        mesh._is_animating = False
        mesh.init_points()

    def test_real_tracker_driven_mesh_has_moving_pixels_and_thread_invariant_output(self):
        histories = []
        class LiveMesh(m.Scene):
            def construct(self):
                tracker = m.ValueTracker(0)
                source = m.ParametricSurface(lambda u, v: (u, v + tracker.get_value()*(u*u-1), 0),
                            resolution=(7, 9), u_range=(-2, 2), v_range=(-1, 1))
                mesh = m.SurfaceMesh(source, resolution=(4, 5), normal_nudge=0,
                                     stroke_color=m.BLUE, stroke_width=2)
                identities = tuple(mesh.submobjects)
                def refresh(w):
                    source.init_points()
                    w.init_points()
                    histories.append((len(w.submobjects), identities == tuple(w.submobjects)))
                mesh.add_updater(refresh, call=False)
                self.add(mesh)
                self.play(tracker.animate.set_value(.8), run_time=.5, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-live-wireframe-') as directory:
            outputs = []
            for threads in (1, 4):
                destination = Path(directory)/f'mesh-{threads}.y4m'
                receipt = LiveMesh().render(destination, format='y4m', resolution=(96, 54), fps=8, threads=threads)
                self.assertEqual(receipt.frame_count, 4)
                outputs.append(destination.read_bytes())
            self.assertEqual(outputs[0], outputs[1])
            _, raw = outputs[0].split(b'\n', 1)
            size = 96*54*3//2
            frames = []
            for offset in range(0, len(raw), size+6):
                self.assertEqual(raw[offset:offset+6], b'FRAME\n')
                frames.append(raw[offset+6:offset+6+size])
            self.assertEqual(len(frames), 4)
            self.assertNotEqual(frames[0], frames[-1])
            self.assertGreater(len(set(frames[0][:96*54])), 1)
        self.assertTrue(histories)
        self.assertTrue(all(count == 9 and stable for count, stable in histories))


def run_live_surface_mesh_acceptance():
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(SurfaceMeshTests))
    if not result.wasSuccessful():
        raise AssertionError(stream.getvalue())
    print(stream.getvalue())
    return result.testsRun


if __name__ == '__main__':
    run_live_surface_mesh_acceptance()
