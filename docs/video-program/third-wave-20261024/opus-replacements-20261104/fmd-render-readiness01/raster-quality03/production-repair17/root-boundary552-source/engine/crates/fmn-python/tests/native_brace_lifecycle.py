"""Brace construction and regeneration on the real Atlas/Marionette/Lumen path."""
import copy
import gc
from pathlib import Path
import tempfile
import unittest
import weakref

import numpy as np
import manimlib as m
from fmn_python import render_session


def kernel(target, direction=m.DOWN, buff=.2, line=False):
    """An independent native brace; no public Brace constructor or hooks."""
    obj = m._native_shell_factory()
    if line:
        specs, tip = obj._build_line_brace(m._native_shell_factory,
            target.get_start(), target.get_end(), direction, buff)
    else:
        specs, tip = obj._build_brace(m._native_shell_factory, target, direction, buff)
    assert not specs
    return obj, tip


def render(path, obj, workers):
    scene = m.Scene()
    with render_session(scene, path, resolution=(80, 48), fps=24, threads=workers) as session:
        scene.add(obj)
        scene.play(m.Transform(obj, obj.copy().shift(.5 * m.UP)), run_time=.125,
                   rate_func=m.linear)
    assert session.result.frame_count == 3
    return path.read_bytes()


class BraceLifecycle(unittest.TestCase):
    def test_native_geometry_tip_and_paints_are_preserved(self):
        for line in (False, True):
            target = m.Line([-2, -.5, 0], [1, 1.5, 0]) if line else m.Rectangle(width=3, height=1).shift(m.RIGHT)
            for direction in (m.DOWN, m.UP, m.RIGHT, np.array([1., -2, 0])):
                with self.subTest(line=line, direction=direction):
                    cls = m.LineBrace if line else m.Brace
                    actual = cls(target, direction, buff=.4)
                    expected, tip = kernel(target, direction, .4, line)
                    np.testing.assert_array_equal(actual.get_points(), expected.get_points())
                    np.testing.assert_array_equal(actual.get_tip(), expected.get_points()[tip])
                    for name in ('fill_rgba', 'stroke_rgba', 'stroke_width'):
                        np.testing.assert_array_equal(actual.data[name], expected.data[name])

    def test_all_hooks_and_cooperative_mro_run_once(self):
        for base in (m.Brace, m.LineBrace):
            events = []
            class Mixin:
                def init_points(self):
                    events.append('mixin')
                    super().init_points()
                    self.shift(m.RIGHT)
            class Authored(Mixin, base):
                def init_data(self):
                    events.append('data')
                    super().init_data()
                def init_uniforms(self):
                    events.append('uniforms')
                    super().init_uniforms()
                    self.uniforms['authored'] = 7.
                def init_colors(self):
                    events.append('colors')
                    super().init_colors()
                    self.set_color(m.BLUE)
            target = m.Line(m.LEFT, m.RIGHT)
            obj = Authored(target, m.DOWN, color=m.RED)
            expected, _ = kernel(target, line=base is m.LineBrace)
            expected.shift(m.RIGHT)
            self.assertEqual(events, ['data', 'mixin', 'uniforms', 'colors'])
            np.testing.assert_array_equal(obj.get_points(), expected.get_points())
            self.assertEqual(obj.get_color(), m.BLUE)
            self.assertEqual(obj.uniforms['authored'], 7.)

    def test_replacement_geometry_and_custom_records_survive(self):
        points = [[-1., 0, 0], [0, -.75, 0], [1, 0, 0]]
        child = m.Circle(radius=.2)
        class Authored(m.Brace):
            data_dtype = [*m.VMobject.data_dtype, ('weight', 1)]
            def init_data(self):
                super().init_data()
                self.add(child)
            def init_points(self):
                self.set_points(points)
                self.data['weight'][:] = 11.
                self.tip_point_index = 1
        obj = Authored(m.Square())
        np.testing.assert_array_equal(obj.get_points(), points)
        np.testing.assert_array_equal(obj.get_tip(), points[1])
        np.testing.assert_array_equal(obj.data['weight'], 11.)
        self.assertIs(obj.submobjects[0], child)
        self.assertIsInstance(obj, m.Tex)
        self.assertIsInstance(obj.copy(), Authored)

    def test_replacement_tip_follows_declared_direction(self):
        class Authored(m.Brace):
            def init_points(self):
                self.set_points([[-1, 0, 0], [0, -.75, 0], [1, 0, 0]])
        obj = Authored(m.Square(), m.DOWN)
        np.testing.assert_array_equal(obj.get_tip(), [0, -.75, 0])
        obj.rotate(.7, about_point=m.ORIGIN).shift(m.RIGHT)
        np.testing.assert_array_equal(obj.get_tip(), obj.get_points()[1])

    def test_explicit_style_channels_and_inherited_flags(self):
        obj = m.Brace(m.Square(), fill_color=m.BLUE, stroke_color=m.RED,
                      opacity=.3, stroke_width=2., depth_test=True, is_fixed_in_frame=True)
        self.assertEqual(obj.get_fill_color(), m.BLUE)
        self.assertEqual(obj.get_stroke_color(), m.RED)
        self.assertAlmostEqual(obj.get_fill_opacity(), .3, places=6)
        self.assertAlmostEqual(obj.get_stroke_opacity(), .3, places=6)
        self.assertTrue(obj.is_fixed_in_frame())
        self.assertTrue(obj.uniforms['depth_test'])

    def test_live_rebuild_retains_identity_children_custom_data_and_style(self):
        child = m.Circle(radius=.1)
        class Authored(m.Brace):
            data_dtype = [*m.VMobject.data_dtype, ('weight', 1)]
            def init_data(self):
                super().init_data()
                self.add(child)
        target = m.Square()
        obj = Authored(target, color=m.GREEN)
        obj.data['weight'][:] = 19.
        scene = m.Scene()
        scene.add(target, obj)
        obj.save_state()
        saved = obj.saved_state
        seen = []
        obj.add_updater(lambda mob: seen.append(mob), call=False)
        target.stretch(2, 0).shift(m.UP)
        self.assertIs(obj.init_points(), obj)
        expected, tip = kernel(target)
        np.testing.assert_array_equal(obj.get_points(), expected.get_points())
        np.testing.assert_array_equal(obj.get_tip(), expected.get_points()[tip])
        np.testing.assert_array_equal(obj.data['weight'], 19.)
        self.assertIs(obj.submobjects[0], child)
        self.assertIs(obj.saved_state, saved)
        self.assertIs(obj._scene, scene)
        self.assertEqual(obj.get_color(), m.GREEN)
        obj.update(0)
        self.assertEqual(seen, [obj])

    def test_line_brace_rebuild_reads_current_endpoints_without_moving_source(self):
        line = m.Line([-1, -.5, 0], [1, .5, 0])
        brace = m.LineBrace(line)
        scene = m.Scene()
        scene.add(line, brace)
        line.rotate(.6).shift(m.UP)
        before = line.get_points().copy()
        brace.init_points()
        expected, tip = kernel(line, m.UP, line=True)
        np.testing.assert_array_equal(line.get_points(), before)
        np.testing.assert_array_equal(brace.get_points(), expected.get_points())
        np.testing.assert_array_equal(brace.get_tip(), expected.get_points()[tip])

    def test_endpoint_errors_are_not_retried_or_wrapped(self):
        error = RuntimeError('endpoint author failure')
        class Target(m.Line):
            fail = False
            def get_start(self):
                if self.fail:
                    raise error
                return super().get_start()
        target = Target(m.LEFT, m.RIGHT)
        brace = m.LineBrace(target)
        before = brace.get_points().copy()
        target.fail = True
        with self.assertRaises(RuntimeError) as caught:
            brace.init_points()
        self.assertIs(caught.exception, error)
        np.testing.assert_array_equal(brace.get_points(), before)
        target.fail = False
        brace.init_points()

    def test_validation_refuses_before_hooks_and_failed_rebuild_is_atomic(self):
        hits = []
        class Authored(m.Brace):
            def init_data(self):
                hits.append('data')
                super().init_data()
        for options in ({'buff': float('nan')}, {'direction': [1, 2]},
                        {'direction': [0, float('inf'), 0]}, {'font_size': float('inf')},
                        {'unrouted_option': 1}):
            with self.subTest(options=options), self.assertRaises((TypeError, ValueError)):
                Authored(m.Square(), **options)
        self.assertEqual(hits, [])
        obj = m.Brace(m.Square())
        before, tip = obj.get_points().copy(), obj.tip_point_index
        obj.buff = float('nan')
        with self.assertRaises(ValueError):
            obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), before)
        self.assertEqual(obj.tip_point_index, tip)

    def test_reentrant_geometry_refuses_and_releases_invocation_state(self):
        class Target(m.Line):
            brace = None
            def get_start(self):
                if self.brace is not None:
                    self.brace.init_points()
                return super().get_start()
        target = Target(m.LEFT, m.RIGHT)
        obj = m.LineBrace(target)
        before = obj.get_points().copy()
        target.brace = obj
        with self.assertRaisesRegex(RuntimeError, 'already in progress'):
            obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), before)
        target.brace = None
        obj.init_points()

    def test_copy_and_collection_preserve_owned_children_without_hidden_roots(self):
        class Authored(m.Brace):
            def init_data(self):
                super().init_data()
                self.mark = m.Circle(radius=.1)
                self.add(self.mark)
        obj = Authored(m.Square())
        dup = copy.deepcopy(obj)
        self.assertIs(dup.mark, dup.submobjects[0])
        self.assertIsNot(dup.mark, obj.mark)
        refs = [weakref.ref(obj), weakref.ref(obj.mark)]
        del obj
        gc.collect()
        self.assertTrue(all(ref() is None for ref in refs))

    def test_empty_and_invalid_tip_are_named_not_index_errors(self):
        class Empty(m.Brace):
            def init_points(self):
                pass
        with self.assertRaisesRegex(ValueError, 'empty brace'):
            Empty(m.Square()).get_tip()
        obj = m.Brace(m.Square())
        obj.tip_point_index = 100000
        with self.assertRaisesRegex(ValueError, 'outside'):
            obj.get_tip()

    def test_authored_interpolation_executes_during_native_clock(self):
        samples = []
        class Authored(m.Brace):
            def interpolate(self, start, end, alpha, path_func=None):
                samples.append(alpha)
                return super().interpolate(start, end, alpha, path_func)
        obj = Authored(m.Square())
        scene = m.Scene()
        scene.add(obj)
        scene.play(m.Transform(obj, obj.copy().shift(m.RIGHT)), run_time=.1)
        self.assertGreater(len(samples), 1)
        self.assertEqual(samples[-1], 1.)

    def test_rendered_hook_geometry_matches_literal_native_control(self):
        class Authored(m.Brace):
            def init_points(self):
                super().init_points()
                self.shift(.8 * m.RIGHT)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected, _ = kernel(m.Square())
            expected.shift(.8 * m.RIGHT)
            pixels = render(root/'literal.y4m', expected, 1)
            for workers in (1, 4, 16):
                self.assertEqual(render(root/f'authored-{workers}.y4m', Authored(m.Square()), workers), pixels)
            ordinary, _ = kernel(m.Square())
            self.assertNotEqual(render(root/'ordinary.y4m', ordinary, 1), pixels)

    def test_translucency_reaches_rendered_frames_and_explicit_channels_win(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            control, _ = kernel(m.Square())
            control.set_opacity(.25)
            pixels = render(root/'literal.y4m', control, 1)
            for workers in (1, 4, 16):
                self.assertEqual(render(root/f'alpha-{workers}.y4m',
                    m.Brace(m.Square(), opacity=.25), workers), pixels)
            opaque, _ = kernel(m.Square())
            self.assertNotEqual(render(root/'opaque.y4m', opaque, 1), pixels)
        obj = m.Brace(m.Square(), opacity=.25, fill_opacity=.75, stroke_opacity=.5)
        self.assertEqual(obj.get_fill_opacity(), .75)
        self.assertEqual(obj.get_stroke_opacity(), .5)

    def test_authored_late_hooks_cannot_publish_nonfinite_or_scene_owned_braces(self):
        class NonFinite(m.Brace):
            def init_colors(self):
                super().init_colors()
                self.data['point'][0, 0] = float('nan')
        with self.assertRaisesRegex(ValueError, 'geometry must be finite'):
            NonFinite(m.Square())
        scene = m.Scene()
        class Bound(m.Brace):
            def init_colors(self):
                super().init_colors()
                scene.add(self)
        with self.assertRaisesRegex(RuntimeError, 'ownership changed'):
            Bound(m.Square())


def run_native_braces():
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(BraceLifecycle))
    if not result.wasSuccessful():
        raise AssertionError('native brace lifecycle acceptance failed')


if __name__ == '__main__':
    run_native_braces()
