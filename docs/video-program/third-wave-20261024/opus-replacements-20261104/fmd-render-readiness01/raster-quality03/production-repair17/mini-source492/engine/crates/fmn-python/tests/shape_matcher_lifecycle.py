"""Native annotation hooks, preserved record state and real frame witnesses."""
from __future__ import annotations

import copy
import inspect
import math
from pathlib import Path
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m


class ShapeMatcherLifecycleTests(unittest.TestCase):
    def test_each_annotation_runs_cooperative_hooks_once(self):
        for base in (m.SurroundingRectangle, m.BackgroundRectangle, m.Cross, m.Underline):
            with self.subTest(base=base.__name__):
                class Custom(base):
                    data_dtype = m.VMobject.data_dtype + [('mass', 1)]
                    def init_data(self):
                        super().init_data()
                        self.events = ['data']
                    def init_points(self):
                        super().init_points()
                        self.events.append('points')
                        self.data['mass'][:] = 7
                    def init_uniforms(self):
                        super().init_uniforms()
                        self.events.append('uniforms')
                        self.uniforms['fixed_in_frame'] = 1
                    def init_colors(self):
                        self.events.append('colors')
                        return super().init_colors()
                obj = Custom(m.Square())
                self.assertEqual(obj.events, ['data', 'points', 'uniforms', 'colors'])
                self.assertIn('mass', obj.data.dtype.names)
                np.testing.assert_array_equal(obj.data['mass'], 7)
                self.assertEqual(obj.uniforms["fixed_in_frame"], 1)

    def test_authored_outline_survives_retargeting_and_live_scene(self):
        class Triangle(m.SurroundingRectangle):
            data_dtype = m.VMobject.data_dtype + [('mass', 1)]
            def init_points(self):
                self.set_points_as_corners([[-1,-1,0], [0,1,0], [1,-1,0], [-1,-1,0]])
                self.data['mass'][:] = 5
        target = m.Square()
        box = Triangle(target)
        self.assertEqual(box.get_num_points(), 7)
        before = box.get_points().copy()
        box.surround(m.Rectangle(width=4, height=1).shift(m.RIGHT), buff=.3)
        self.assertEqual(box.get_num_points(), 7)
        np.testing.assert_array_equal(box.data['mass'], 5)
        self.assertAlmostEqual(box.get_width(), 4.6, places=6)
        self.assertAlmostEqual(box.get_height(), 1.6, places=6)
        scene = m.Scene(); scene.add(box)
        box.surround(target, buff=.1)
        self.assertEqual(box.get_num_points(), 7)
        np.testing.assert_allclose(box.get_points(), before, atol=1e-6)

    def test_decorations_and_updaters_survive_retargeting(self):
        class Decorated(m.SurroundingRectangle):
            def init_data(self):
                super().init_data()
                self.marker = m.Dot(radius=.03)
                self.add(self.marker)
        obj = Decorated(m.Square())
        updater = lambda mob, dt: None
        obj.add_updater(updater)
        obj.save_state()
        saved = obj.saved_state
        marker = obj.marker
        for target in (m.Circle(), m.Rectangle(width=4, height=3)):
            obj.surround(target, .2)
            self.assertIs(obj.marker, marker)
            self.assertIn(marker, obj.submobjects)
            self.assertIs(obj.saved_state, saved)
            self.assertIn(updater, obj.get_updaters())

    def test_constructor_uses_public_surround_shape_and_move_hooks(self):
        calls = []
        class Custom(m.SurroundingRectangle):
            def surround(self, target, buff=None):
                calls.append(('surround', target, buff))
                return super().surround(target, buff)
            def set_shape(self, *args, **kwargs):
                calls.append(('shape',))
                return super().set_shape(*args, **kwargs)
            def move_to(self, *args, **kwargs):
                calls.append(('move',))
                return super().move_to(*args, **kwargs)
        target = m.Square()
        obj = Custom(target)
        self.assertEqual([call[0] for call in calls], ['surround', 'shape', 'move'])
        obj.set_buff(.4)
        self.assertEqual([call[0] for call in calls], ['surround', 'shape', 'move'] * 2)
        self.assertAlmostEqual(obj.get_width(), 2.8)

    def test_target_width_height_overrides_are_not_bypassed(self):
        class Target(m.Square):
            def get_width(self): return 4.0
            def get_height(self): return 3.0
        obj = m.SurroundingRectangle(Target(), buff=.2)
        self.assertAlmostEqual(obj.get_width(), 4.4)
        self.assertAlmostEqual(obj.get_height(), 3.4)
        line = m.Underline(Target(), stretch_factor=1.5)
        self.assertAlmostEqual(line.get_width(), 6)

    def test_cross_public_replace_controls_assembled_arms(self):
        calls = []
        class Raised(m.Cross):
            def replace(self, target, **kwargs):
                calls.append(target)
                return super().replace(target, **kwargs).shift(m.UP)
        target = m.Square().shift(m.RIGHT)
        cross = Raised(target)
        self.assertEqual(calls, [target])
        np.testing.assert_allclose(cross.get_center(), m.RIGHT + m.UP, atol=1e-6)
        self.assertEqual(len(cross), 2)
        for arm in cross:
            self.assertGreater(len(arm.get_stroke_widths()), 3)
            self.assertAlmostEqual(float(arm.get_stroke_widths().max()), 6)

    def test_underline_custom_points_and_placement_survive(self):
        calls = []
        class VRule(m.Underline):
            data_dtype = m.VMobject.data_dtype + [('mass', 1)]
            def init_points(self):
                self.set_points_as_corners([[-1,0,0], [0,-.5,0], [1,0,0]])
                self.data['mass'][:] = 3
            def next_to(self, target, *args, **kwargs):
                calls.append(target)
                return super().next_to(target, *args, **kwargs).shift(m.RIGHT)
        target = m.Square()
        rule = VRule(target, buff=.3, stroke_width=2)
        self.assertEqual(calls, [target])
        self.assertEqual(rule.get_num_points(), 5)
        self.assertGreater(rule.get_height(), .4)
        self.assertAlmostEqual(rule.get_top()[1], -1.3)
        self.assertAlmostEqual(rule.get_center()[0], 1)
        np.testing.assert_array_equal(rule.data['mass'], 3)

    def test_falsey_explicit_channels_and_fixed_flag_remain_supported(self):
        target = m.Square().fix_in_frame()
        a = m.SurroundingRectangle(target, color=m.BLUE, stroke_color=m.RED,
                                   fill_color=m.GREEN, fill_opacity=.3, stroke_width=0)
        self.assertEqual(a.get_stroke_color(), m.RED)
        self.assertEqual(a.get_fill_color(), m.GREEN)
        self.assertEqual(a.get_stroke_width(), 0)
        self.assertTrue(a.is_fixed_in_frame())
        b = m.SurroundingRectangle(target, color=m.BLUE, stroke_color=None, fill_color=None)
        self.assertEqual(b.get_stroke_color(), m.BLUE)
        self.assertEqual(b.get_fill_color(), m.BLUE)

    def test_empty_target_and_later_regrowth_keep_authored_records(self):
        class Custom(m.SurroundingRectangle):
            data_dtype = m.VMobject.data_dtype + [('mass', 1)]
            def init_points(self):
                super().init_points()
                self.data['mass'][:] = 9
        a = Custom(m.Mobject())
        self.assertFalse(a.has_points())
        scene = m.Scene(); scene.add(a)
        a.surround(m.Square(), buff=.25)
        self.assertAlmostEqual(a.get_width(), 2.5)
        np.testing.assert_array_equal(a.data['mass'], 9)
        for cls in (m.Cross, m.Underline):
            self.assertFalse(cls(m.Mobject()).family_members_with_points())

    def test_failure_stops_initialization_and_preserves_authored_exception(self):
        failure = RuntimeError('authored annotation geometry')
        for base in (m.SurroundingRectangle, m.BackgroundRectangle, m.Cross, m.Underline):
            with self.subTest(base=base.__name__):
                calls = []
                class Broken(base):
                    def init_points(self): calls.append('points'); raise failure
                    def init_uniforms(self): calls.append('uniforms')
                with self.assertRaises(RuntimeError) as caught:
                    Broken(m.Square())
                self.assertIs(caught.exception, failure)
                self.assertEqual(calls, ['points'])

    def test_parameter_admission_precedes_hooks(self):
        for base, config in ((m.SurroundingRectangle, {'buff':math.nan}),
                             (m.Cross, {'stroke_width':[1, math.inf]}),
                             (m.Underline, {'stretch_factor':math.inf}),
                             (m.Underline, {'stroke_width':range(5000)})):
            calls = []
            class Custom(base):
                def init_points(self): calls.append('points'); super().init_points()
            with self.subTest(config=config), self.assertRaises((TypeError, ValueError)):
                Custom(m.Square(), **config)
            self.assertEqual(calls, [])

    def test_same_size_record_views_survive_surround(self):
        a = m.SurroundingRectangle(m.Square())
        points = a.get_points()
        fills = a.data['fill_rgba']
        a.surround(m.Rectangle(width=4, height=3), .25)
        np.testing.assert_allclose(points, a.get_points())
        self.assertTrue(np.shares_memory(points, a.get_points()))
        fills[:,3] = .4
        np.testing.assert_allclose(a.data['fill_rgba'][:,3], .4)

    def test_copies_pickle_and_builder_targets_do_not_rerun_constructors(self):
        original = m.SurroundingRectangle(m.Square())
        original.surround(m.Mobject())
        for clone in (original.copy(), copy.deepcopy(original), pickle.loads(pickle.dumps(original))):
            clone.surround(m.Square().shift(m.RIGHT), .25)
            self.assertAlmostEqual(clone.get_width(), 2.5)
            self.assertFalse(original.has_points())
        scene = m.Scene(); source = m.SurroundingRectangle(m.Square())
        scene.add(source)
        scene.play(source.animate.surround(m.Rectangle(width=4,height=3), .3), run_time=1/30)
        self.assertAlmostEqual(source.get_width(), 4.6, places=5)
        self.assertAlmostEqual(source.get_height(), 3.6, places=5)

    def test_background_post_construction_style_and_partial_reveal(self):
        class Custom(m.BackgroundRectangle):
            def init_colors(self):
                super().init_colors()
                self.set_fill(opacity=.4)
        plate = Custom(m.Square(), color='#123456', fill_opacity=.6)
        self.assertAlmostEqual(plate.get_fill_opacity(), .4)
        self.assertEqual(m.VMobject.get_fill_color(plate), '#123456')
        plate.surround(m.Rectangle(width=3,height=2), .2)
        self.assertEqual(m.VMobject.get_fill_color(plate), '#123456')
        plate.pointwise_become_partial(plate, 0, .5)
        self.assertAlmostEqual(plate.get_fill_opacity(), .3)

    def test_signatures_and_aliases_stay_published(self):
        from manimlib.mobject.shape_matchers import SurroundingRectangle, Cross, Underline
        self.assertIs(SurroundingRectangle, m.SurroundingRectangle)
        self.assertIs(Cross, m.Cross); self.assertIs(Underline, m.Underline)
        self.assertEqual(str(inspect.signature(Cross)),
                         "(mobject, stroke_color='#FC6255', stroke_width=[0, 6, 0], **kwargs)")

    def test_actual_frames_match_independently_positioned_triangle(self):
        class Triangle(m.SurroundingRectangle):
            def init_points(self):
                self.set_points_as_corners([[-1,-1,0],[0,1,0],[1,-1,0],[-1,-1,0]])
        def render(path, authored, threads):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96,54), fps=4, threads=threads):
                if authored:
                    shape = Triangle(m.Square(), color=m.RED, buff=.25, fill_opacity=.4, stroke_width=1)
                else:
                    shape = m.VMobject(color=m.RED, fill_opacity=.4, stroke_width=1)
                    shape.set_points_as_corners([[-1.25,-1.25,0],[0,1.25,0],
                                                [1.25,-1.25,0],[-1.25,-1.25,0]])
                scene.add(shape)
                scene.play(shape.animate.shift(m.RIGHT), run_time=1, rate_func=m.linear)
            return [x.read_bytes() for x in sorted(path.glob('*.png'))]
        root = Path(tempfile.mkdtemp(prefix='fmn-annotations-'))
        result = render(root/'one', True, 1)
        self.assertEqual(result, render(root/'four', True, 4))
        self.assertEqual(result, render(root/'control', False, 1))
        self.assertEqual(len(result), 4)
        self.assertGreater(len(set(result)), 1)


if __name__ == '__main__':
    unittest.main()
