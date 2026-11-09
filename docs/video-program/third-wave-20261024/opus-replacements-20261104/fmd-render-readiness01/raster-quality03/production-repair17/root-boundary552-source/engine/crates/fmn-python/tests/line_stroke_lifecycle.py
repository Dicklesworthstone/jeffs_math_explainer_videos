"""Actual native line regeneration and StrokeArrow subclass lifecycle."""
from __future__ import annotations

import copy
import inspect
from pathlib import Path
import pickle
import tempfile
import unittest
from unittest.mock import patch

import numpy as np
import manimlib as m


class AnnotatedLine(m.Line):
    data_dtype = m.VMobject.data_dtype + [('mass', 1)]


class AnnotatedStroke(m.StrokeArrow):
    data_dtype = m.VMobject.data_dtype + [('mass', 1)]


class LineStrokeLifecycleTests(unittest.TestCase):
    def test_detached_and_bound_line_rebuild_preserve_schema_family_and_state(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                obj = AnnotatedLine(m.LEFT, m.RIGHT, color=m.BLUE, stroke_width=3)
                obj.data['mass'][:] = 7
                child = m.Dot(); obj.add(child)
                obj.uniforms['author_uniform'] = 4
                updater = lambda obj, dt: None
                obj.add_updater(updater)
                obj.save_state()
                saved = obj.saved_state
                scene = m.Scene()
                if bound:
                    scene.add(obj)
                identity = hash(obj)
                self.assertIs(obj.set_points_by_ends(m.DOWN, m.UP, path_arc=.6), obj)
                self.assertEqual(hash(obj), identity)
                self.assertIs(obj[0], child)
                self.assertIs(obj.saved_state, saved)
                self.assertIs(obj.updaters[0], updater)
                self.assertEqual(obj.uniforms['author_uniform'], 4)
                np.testing.assert_array_equal(obj.data['mass'], 7)
                np.testing.assert_allclose(obj.get_start(), m.DOWN, atol=1e-6)
                np.testing.assert_allclose(obj.get_end(), m.UP, atol=1e-6)
                self.assertEqual(obj.get_stroke_color(), m.BLUE)
                self.assertEqual(obj.get_stroke_width(), 3)
                if bound:
                    self.assertIs(obj._scene, scene)

    def test_line_views_live_until_topology_resize(self):
        obj = AnnotatedLine()
        mass, points = obj.data['mass'], obj.get_points()
        mass[:] = 2
        obj.set_points_by_ends(m.DOWN, m.UP)
        np.testing.assert_array_equal(points, obj.get_points())
        mass[:] = 9
        np.testing.assert_array_equal(obj.data['mass'], 9)
        before = points.copy()
        obj.set_path_arc(1.5)
        self.assertGreater(obj.get_num_points(), len(points))
        np.testing.assert_array_equal(points, before)
        points[:] = 500
        mass[:] = 500
        self.assertLess(float(np.max(obj.get_points())), 10)
        np.testing.assert_array_equal(obj.data['mass'], 9)

    def test_line_rebuild_uses_world_endpoints_and_keeps_child_placement(self):
        obj = AnnotatedLine().shift([2, 3, 0]).scale(2)
        child = m.Dot([5, 2, 0]); obj.add(child)
        before = child.get_points().copy()
        obj.set_points_by_ends([-3, 1, 0], [4, -2, 0])
        np.testing.assert_allclose(obj.get_start(), [-3, 1, 0], atol=1e-6)
        np.testing.assert_allclose(obj.get_end(), [4, -2, 0], atol=1e-6)
        np.testing.assert_array_equal(child.get_points(), before)

    def test_failed_line_recipe_keeps_values_views_and_path_arc(self):
        obj = AnnotatedLine(); obj.data['mass'][:] = 3
        children = [m.Dot()]; obj.add(*children)
        points, before, data = obj.get_points(), obj.get_points().copy(), obj.data.copy()
        with self.assertRaises(ValueError):
            obj.set_path_arc(float('nan'))
        np.testing.assert_array_equal(obj.data, data)
        np.testing.assert_array_equal(points, before)
        self.assertEqual(obj.path_arc, 0)
        self.assertEqual(list(obj), children)

    def test_line_public_style_dispatch_survives_regeneration(self):
        obj = m.Line(); calls = []
        original = obj.set_style
        def style(**kwargs):
            calls.append(kwargs['recurse'])
            return original(**kwargs)
        obj.set_style = style
        obj.set_points_by_ends(m.DOWN, m.UP)
        self.assertEqual(calls, [False])

    def test_stroke_hooks_run_once_with_recipes_visible(self):
        events = []
        class Custom(m.StrokeArrow):
            def init_data(self):
                events.append(('data', self.tip_width_ratio, self.original_stroke_width, self.buff))
                super().init_data()
            def init_points(self):
                events.append(('points',))
                super().init_points()
                self.shift(m.UP)
            def init_uniforms(self):
                events.append(('uniforms',))
                super().init_uniforms()
                self.uniforms['author_uniform'] = 7
            def init_colors(self):
                events.append(('colors',))
                super().init_colors()
                self.set_color(m.GREEN)
        obj = Custom(m.LEFT, m.RIGHT, buff=.1, stroke_width=2, tip_width_ratio=3)
        self.assertEqual(events, [('data', 3., 2., .1), ('points',), ('uniforms',), ('colors',)])
        np.testing.assert_allclose(obj.get_start(), [-.9, 1, 0], atol=1e-6)
        np.testing.assert_allclose(obj.get_end(), [.9, 1, 0], atol=1e-6)
        np.testing.assert_allclose(obj.get_stroke_widths()[-3:], [6, 3, 0])
        self.assertEqual(obj.get_stroke_color(), m.GREEN)
        self.assertEqual(obj.uniforms['author_uniform'], 7)

    def test_stroke_cooperative_mro_and_virtual_endpoints(self):
        calls = []
        class Parent(m.StrokeArrow):
            def init_points(self):
                calls.append('parent')
                super().init_points()
            def set_points_by_ends(self, *args, **kwargs):
                calls.append('ends')
                return super().set_points_by_ends(*args, **kwargs)
        class Mixin:
            def init_points(self):
                calls.append('mixin')
                super().init_points()
                self.shift(2*m.UP)
        class Custom(Mixin, Parent):
            pass
        original = m.Line.__init__
        def line_init(self, *args, **kwargs):
            calls.append('line')
            original(self, *args, **kwargs)
        with patch.object(m.Line, '__init__', line_init):
            obj = Custom(m.LEFT, m.RIGHT, buff=0)
        # One point initialization and one endpoint reset from init_colors.
        self.assertEqual(calls, ['line', 'mixin', 'parent', 'ends', 'ends'])
        np.testing.assert_allclose(obj.get_start(), [-1, 2, 0], atol=1e-6)
        self.assertEqual(obj.get_num_points(), 7)

    def test_stroke_custom_fields_views_and_children_survive_tip_resets(self):
        class Custom(AnnotatedStroke):
            def init_data(self):
                super().init_data()
                self.decoration = m.Dot(2*m.UP)
                self.add(self.decoration)
            def init_points(self):
                super().init_points()
                self.data['mass'][:] = 7
                self.masses = self.data['mass']
        obj = Custom(m.LEFT, m.RIGHT, buff=0)
        obj.masses[:] = 9
        np.testing.assert_array_equal(obj.data['mass'], 9)
        scene = m.Scene(); scene.add(obj)
        # Nursery-to-Scene adoption moves the records (Stage::move_into,
        # c03206ca): a view exported before scene.add stays attached and live
        # (plan section 8.2).
        nursery_view = obj.masses
        obj.masses = obj.data['mass']
        self.assertTrue(np.shares_memory(nursery_view, obj.masses))
        nursery_view[:] = 10
        np.testing.assert_array_equal(obj.data['mass'], 10)
        nursery_view[:] = 9
        for action in (lambda: obj.set_stroke(width=3), lambda: obj.scale(2),
                       lambda: obj.set_points_by_ends(m.DOWN, m.UP)):
            action()
            self.assertIs(obj[0], obj.decoration)
            np.testing.assert_array_equal(obj.data['mass'], 9)
            obj.masses[:] = 11
            np.testing.assert_array_equal(obj.data['mass'], 11)
            obj.masses[:] = 9
            self.assertIs(obj._scene, scene)

    def test_stroke_init_points_restores_taper_not_plain_line(self):
        obj = m.StrokeArrow(m.LEFT, m.RIGHT, buff=.1, path_arc=.5, stroke_width=3)
        expected = m._native_shell_factory()
        expected._build_stroke_arrow(m._native_shell_factory, tuple(m.LEFT), tuple(m.RIGHT),
                                     3., .1, .5, 5., .0075, .3, 8.)
        obj.shift(2*m.UP)
        self.assertIs(obj.init_points(), obj)
        np.testing.assert_allclose(obj.get_points(), expected.get_points(), rtol=1e-6, atol=1e-6)
        np.testing.assert_array_equal(obj.get_stroke_widths(), expected.get_stroke_widths())
        self.assertGreater(obj.get_num_points(), 7)

    def test_stroke_failed_hooks_stop_later_phases(self):
        for name in ('init_data', 'init_points', 'init_uniforms', 'init_colors'):
            with self.subTest(name=name):
                calls = []; failure = ValueError(name)
                def fail(self):
                    calls.append(name)
                    raise failure
                cls = type('Broken', (m.StrokeArrow,), {name: fail})
                with self.assertRaises(ValueError) as caught:
                    cls(m.LEFT, m.RIGHT)
                self.assertIs(caught.exception, failure)
                self.assertEqual(calls, [name])
                self.assertEqual(m.StrokeArrow(m.LEFT, m.RIGHT).get_num_points(), 7)

    def test_stroke_source_mobjects_resolve_boundaries_without_adoption(self):
        left, right = m.Square().shift(2*m.LEFT), m.Square().shift(2*m.RIGHT)
        scene = m.Scene(); scene.add(left, right)
        obj = m.StrokeArrow(left, right, buff=.1)
        np.testing.assert_allclose(obj.get_start(), [-.9, 0, 0], atol=1e-6)
        np.testing.assert_allclose(obj.get_end(), [.9, 0, 0], atol=1e-6)
        self.assertEqual(list(obj), [])
        self.assertIs(left._scene, scene)
        self.assertFalse(obj._is_bound())

    def test_stroke_style_configuration_and_signature(self):
        obj = m.StrokeArrow(m.LEFT, m.RIGHT, color=m.RED, stroke_color=m.BLUE,
                            stroke_width=3, opacity=.4, flat_stroke=True,
                            scale_stroke_with_zoom=True, joint_type='bevel',
                            anti_alias_width=2, depth_test=True, is_fixed_in_frame=True)
        # fm-qead (BN-07 C-19): the explicit stroke_color beats color=.
        self.assertEqual(obj.get_stroke_color(), m.BLUE)
        self.assertEqual(m.StrokeArrow(m.LEFT, m.RIGHT, color=m.RED).get_stroke_color(), m.RED)
        self.assertAlmostEqual(obj.get_stroke_opacity(), .4, places=6)
        self.assertTrue(obj.get_flat_stroke())
        self.assertTrue(obj.get_scale_stroke_with_zoom())
        self.assertEqual(obj.uniforms['anti_alias_width'], 2)
        self.assertTrue(obj.uniforms['depth_test'])
        self.assertEqual(obj.uniforms['is_fixed_in_frame'], 1)
        self.assertEqual(list(inspect.signature(m.StrokeArrow).parameters), [
            'start', 'end', 'stroke_color', 'stroke_width', 'buff', 'tip_width_ratio',
            'tip_len_to_width', 'max_tip_length_to_length_ratio',
            'max_width_to_length_ratio', 'kwargs',
        ])
        from manimlib.mobject.geometry import StrokeArrow
        self.assertIs(StrokeArrow, m.StrokeArrow)

    def test_stroke_copies_do_not_repeat_initialization(self):
        calls = []
        class Custom(AnnotatedStroke):
            def init_points(self):
                calls.append('points')
                super().init_points()
                self.data['mass'][:] = 5
        obj = Custom(m.LEFT, m.RIGHT)
        before = obj.get_points().copy()
        for clone in (obj.copy(), copy.deepcopy(obj)):
            self.assertIs(type(clone), Custom)
            clone.set_points_by_ends(m.DOWN, m.UP)
            np.testing.assert_array_equal(clone.data['mass'], 5)
            np.testing.assert_array_equal(obj.get_points(), before)
        self.assertEqual(calls, ['points'])

    def test_pickled_line_and_stroke_regenerate_independently(self):
        for obj in (m.Line(), m.StrokeArrow(m.LEFT, m.RIGHT)):
            with self.subTest(obj=type(obj).__name__):
                before = obj.get_points().copy()
                clone = pickle.loads(pickle.dumps(obj))
                clone.set_points_by_ends(m.DOWN, m.UP)
                np.testing.assert_allclose(clone.get_start(), m.DOWN, atol=1e-6)
                np.testing.assert_array_equal(obj.get_points(), before)

    def test_line_builder_endpoint_changes_keep_custom_columns_and_children(self):
        obj = AnnotatedLine(); obj.data['mass'][:] = 8
        child = m.Dot(2*m.UP); obj.add(child)
        animation = obj.animate.set_points_by_ends(m.DOWN, m.UP).build()
        scene = m.Scene(); scene.add(obj)
        scene.play(animation, run_time=1/30, rate_func=m.linear)
        np.testing.assert_array_equal(obj.data['mass'], 8)
        self.assertIs(obj[0], child)
        np.testing.assert_allclose(obj.get_start(), m.DOWN, atol=1e-6)

    def test_stroke_taper_and_all_frames_match_independent_native_builder(self):
        def render(path, authored, threads):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96, 54),
                                      fps=8, threads=threads):
                objects = []
                for arc, y in ((-.8, -2), (0, 0), (.8, 2)):
                    if authored:
                        class Raised(m.StrokeArrow):
                            def init_points(self):
                                super().init_points()
                                self.shift(.25*m.UP)
                        obj = Raised(2*m.LEFT, 2*m.RIGHT, buff=.1, path_arc=arc,
                                     stroke_width=3, stroke_color=m.RED)
                    else:
                        obj = m._native_shell_factory()
                        specs = obj._build_stroke_arrow(m._native_shell_factory,
                            (-2.,0.,0.),(2.,0.,0.),3.,.1,arc,5.,.0075,.3,8.)
                        self.assertFalse(specs)
                        # StrokeArrow's existing set_stroke/init_colors resets
                        # from trimmed endpoints with zero further buffer.
                        # Exercise that documented two-stage native recipe
                        # without the Python StrokeArrow constructor.
                        obj._rebuild_stroke_arrow(tuple(obj.get_start()),tuple(obj.get_end()),
                            3.,0.,arc,5.,.0075,.3,8.)
                        obj.set_stroke(color=m.RED)
                        obj.shift(.25*m.UP)
                    obj.shift(y*m.UP); objects.append(obj)
                group=m.VGroup(*objects); scene.add(group)
                scene.play(group.animate.shift(.5*m.RIGHT), run_time=.5, rate_func=m.linear)
            return [file.read_bytes() for file in sorted(path.glob('*.png'))]
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            frames=render(root/'one',True,1)
            self.assertEqual(frames,render(root/'four',True,4))
            self.assertEqual(frames,render(root/'native',False,1))
            self.assertEqual(len(frames),4)
            self.assertGreater(len(set(frames)),1)


if __name__ == '__main__':
    unittest.main(verbosity=2)
