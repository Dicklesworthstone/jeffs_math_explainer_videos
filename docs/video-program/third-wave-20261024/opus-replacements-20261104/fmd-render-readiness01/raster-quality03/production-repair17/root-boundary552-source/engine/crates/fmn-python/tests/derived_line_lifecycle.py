"""Actual native derived-line geometry and authored constructor/sample hooks."""
from __future__ import annotations

import copy
import math
from pathlib import Path
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m


class DerivedLineLifecycleTests(unittest.TestCase):
    def test_dashed_initialization_hooks_and_custom_records(self):
        class Custom(m.DashedLine):
            data_dtype = m.VMobject.data_dtype + [('weight', 1)]
            def init_data(self):
                super().init_data(); self.events = ['data']
            def init_points(self):
                super().init_points(); self.events.append('points')
                self.data['weight'][:] = 7
                self.shift(m.UP)
            def init_uniforms(self):
                super().init_uniforms(); self.events.append('uniforms')
            def init_colors(self):
                super().init_colors(); self.events.append('colors')
                self.set_color(m.RED)
            def calculate_num_dashes(self, length, ratio):
                self.events.append('count'); return 2
        line = Custom(m.LEFT, m.RIGHT)
        self.assertEqual(line.events, ['data', 'points', 'uniforms', 'colors', 'count'])
        self.assertEqual(len(line), 2)
        self.assertFalse(line.has_points())
        for dash in line:
            np.testing.assert_array_equal(dash.data['weight'], 7)
            np.testing.assert_allclose(dash.get_points()[:,1], 1)
            self.assertEqual(dash.get_stroke_color(), m.RED)

    def test_public_slice_overrides_execute_on_original_and_control_geometry(self):
        calls = []
        class Raised(m.DashedLine):
            def calculate_num_dashes(self, *args): return 3
            def get_subcurve(self, a, b):
                calls.append((self, a, b))
                return super().get_subcurve(a, b).shift(m.UP)
        line = Raised(m.LEFT * 2, m.RIGHT * 2)
        self.assertEqual(len(calls), 3)
        self.assertTrue(all(source is line for source, _, _ in calls))
        self.assertEqual(len(line), 3)
        np.testing.assert_allclose([part.get_center()[1] for part in line], 1)

    def test_arclength_count_uses_actual_bent_contour(self):
        class Bent(m.DashedLine):
            def init_points(self):
                self.set_points_as_corners([[0,0,0],[0,3,0],[1,3,0]])
        line = Bent(dash_length=1, positive_space_ratio=.5)
        self.assertEqual(len(line), 2)
        np.testing.assert_allclose(line[0].get_start(), [0,0,0])
        np.testing.assert_allclose(line[0].get_end(), [0,1,0], atol=1e-6)
        self.assertGreater(line[1].get_end()[0], .5)

    def test_invalid_or_excessive_dashes_do_not_run_slice_callbacks(self):
        calls = []
        class TooMany(m.DashedLine):
            def calculate_num_dashes(self, *args): return 4097
            def get_subcurve(self, *args): calls.append('slice')
        with self.assertRaisesRegex(ValueError, '4096 cap'): TooMany()
        self.assertEqual(calls, [])
        for options in ({'dash_length':0}, {'positive_space_ratio':0}, {'dash_length':math.nan}):
            class Invalid(m.DashedLine):
                def init_data(self): calls.append('data'); super().init_data()
            with self.assertRaises(ValueError): Invalid(**options)
            self.assertEqual(calls, [])

    def test_unknown_dash_option_still_refuses_before_installation(self):
        obj = m.DashedLine.__new__(m.DashedLine)
        with self.assertRaisesRegex(NotImplementedError, 'leftover_dash_option'):
            obj.__init__(leftover_dash_option=True)
        self.assertNotIn('submobjects', vars(obj))

    def test_failed_slicing_preserves_unsliced_source(self):
        failure = RuntimeError('authored dash slice')
        objects, calls = [], []
        class Broken(m.DashedLine):
            def calculate_num_dashes(self, *args): return 2
            def get_subcurve(self, a, b):
                objects.append(self); calls.append(a)
                if len(calls) == 2: raise failure
                return super().get_subcurve(a, b)
        with self.assertRaises(RuntimeError) as caught: Broken(m.LEFT, m.RIGHT)
        self.assertIs(caught.exception, failure)
        self.assertEqual(len(calls), 2)
        self.assertTrue(objects[0].has_points())
        self.assertEqual(list(objects[0].submobjects), [])

    def test_foreign_or_self_slices_are_not_adopted(self):
        foreign = m.Line(); scene = m.Scene(); scene.add(foreign)
        for mode in ('foreign', 'self'):
            made = []
            class Broken(m.DashedLine):
                def calculate_num_dashes(self, *args): return 1
                def get_subcurve(self, *args):
                    made.append(self)
                    return foreign if mode == 'foreign' else self
            with self.subTest(mode=mode), self.assertRaises((ValueError, RuntimeError)):
                Broken()
            self.assertTrue(made[0].has_points())
        self.assertIn(foreign, scene.mobjects)

    def test_dashed_copy_pickle_and_live_redraw_keep_native_family(self):
        original = m.DashedLine(m.LEFT, m.RIGHT, dash_length=.25)
        for cloned in (original.copy(), copy.deepcopy(original), pickle.loads(pickle.dumps(original))):
            self.assertEqual(len(cloned), len(original))
            for left, right in zip(original, cloned):
                np.testing.assert_array_equal(left.get_points(), right.get_points())
        # Each dash records its owner (FamilyRefs). Unpickling restores that
        # back-edge to the unpickled owner itself; it used to recurse without
        # end (fm-5wq.42).
        dashed = m.DashedVMobject(m.Circle())
        restored = pickle.loads(pickle.dumps(dashed))
        self.assertEqual(len(restored), len(dashed))
        for dash in restored:
            self.assertIs(vars(dash)["_fmn_derived_owner"][0], restored)
        scene = m.Scene(); scene.add(original)
        target = m.DashedLine(m.LEFT * 2, m.RIGHT * 2, dash_length=.1)
        original.become(target)
        self.assertEqual(len(original), len(target))
        scene.play(original.animate.shift(m.UP), run_time=1/30, rate_func=m.linear)
        np.testing.assert_allclose(original.get_start(), target.get_start() + m.UP)

    def test_tangent_hooks_control_real_record_geometry(self):
        class Custom(m.TangentLine):
            data_dtype = m.VMobject.data_dtype + [('weight', 1)]
            def init_data(self):
                super().init_data(); self.events = ['data']
            def init_points(self):
                super().init_points(); self.events.append('points')
                self.data['weight'][:] = 3
                self.shift(m.RIGHT)
            def init_uniforms(self):
                super().init_uniforms(); self.events.append('uniforms')
            def init_colors(self):
                super().init_colors(); self.events.append('colors')
                self.set_color(m.RED)
        source = m.Line([0,0,0], [0,4,0])
        tangent = Custom(source, .5, d_alpha=.125, length=2)
        self.assertEqual(tangent.events, ['data', 'points', 'uniforms', 'colors'])
        np.testing.assert_allclose(tangent.get_center(), [1,2,0])
        self.assertAlmostEqual(tangent.get_length(), 2)
        np.testing.assert_array_equal(tangent.data['weight'], 3)
        self.assertEqual(tangent.get_stroke_color(), m.RED)

    def test_tangent_calls_public_sampler_at_two_clipped_points(self):
        calls = []
        class Curve(m.Line):
            def pfp(self, a): calls.append(a); return [1,2*a,0]
        tangent = m.TangentLine(Curve(), .05, d_alpha=.1, length=3)
        np.testing.assert_allclose(calls, [0, .15])
        np.testing.assert_allclose(tangent.get_center(), [1,.15,0], atol=1e-6)
        self.assertAlmostEqual(tangent.get_length(), 3)
        np.testing.assert_allclose(tangent.get_unit_vector(), m.UP)

    def test_tangent_uses_live_arclength_not_curve_index(self):
        path = m.VMobject().set_points_as_corners([[0,0,0],[1,0,0],[1,3,0]])
        scene = m.Scene(); scene.add(path); path.shift([3,-2,0])
        tangent = m.TangentLine(path, .5, length=3, d_alpha=1e-4)
        np.testing.assert_allclose(tangent.get_center(), [4,-1,0], atol=1e-6)
        np.testing.assert_allclose(tangent.get_unit_vector(), m.UP)
        self.assertAlmostEqual(tangent.get_length(), 3)

    def test_sampler_failure_is_original_and_precedes_tangent_hooks(self):
        failure, calls = RuntimeError('sampler'), []
        class Curve(m.Line):
            def pfp(self, a): calls.append(a); raise failure
        class Tangent(m.TangentLine):
            def init_points(self): calls.append('points'); super().init_points()
        with self.assertRaises(RuntimeError) as caught: Tangent(Curve(), .5)
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, [.5-1e-6])

    def test_tangent_invalid_samples_and_parameters_are_refused(self):
        calls = []
        class Curve(m.Line):
            def pfp(self, a): calls.append(a); return [math.nan,0,0]
        with self.assertRaises(ValueError): m.TangentLine(Curve(), .5)
        self.assertEqual(len(calls), 1)
        calls.clear()
        with self.assertRaises(ValueError): m.TangentLine(Curve(), math.nan)
        self.assertEqual(calls, [])

    def test_empty_zero_length_and_zero_delta_stay_finite(self):
        self.assertFalse(m.TangentLine(m.VMobject(), .5).has_points())
        source = m.Line(m.LEFT, m.RIGHT)
        for options in ({'length':0}, {'d_alpha':0}, {'alpha':2}):
            tangent = m.TangentLine(source, **dict(alpha=.5, **options)) if 'alpha' not in options else m.TangentLine(source, **options)
            self.assertTrue(np.isfinite(tangent.get_points()).all())
            self.assertAlmostEqual(tangent.get_length(), 0)

    def test_derived_line_frames_match_independent_literal_geometry(self):
        class Dashes(m.DashedLine):
            def calculate_num_dashes(self, *args): return 2
            def get_subcurve(self, a, b): return super().get_subcurve(a, b).shift(m.UP)
        class Curve(m.Line):
            def pfp(self, a): return [1,2*a,0]
        class Tangent(m.TangentLine):
            def init_points(self): super().init_points(); self.shift(m.RIGHT)
        def render(path, authored, threads):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96,54), fps=4, threads=threads):
                if authored:
                    obj = m.VGroup(Dashes([0,0,0],[4,0,0],color=m.RED,stroke_width=2),
                                   Tangent(Curve(),.5,d_alpha=.125,length=2,color=m.BLUE,stroke_width=2))
                else:
                    obj = m.VGroup(m.Line([0,1,0],[1,1,0],color=m.RED,stroke_width=2),
                                   m.Line([8/3,1,0],[11/3,1,0],color=m.RED,stroke_width=2),
                                   m.Line([2,0,0],[2,2,0],color=m.BLUE,stroke_width=2))
                scene.add(obj); scene.play(obj.animate.shift(m.LEFT), run_time=1, rate_func=m.linear)
            return [file.read_bytes() for file in sorted(path.glob('*.png'))]
        root = Path(tempfile.mkdtemp(prefix='fmn-derived-lines-'))
        frames = render(root/'one', True, 1)
        self.assertEqual(frames, render(root/'four', True, 4))
        self.assertEqual(frames, render(root/'control', False, 1))
        self.assertEqual(len(frames), 4)
        self.assertGreater(len(set(frames)), 1)


if __name__ == '__main__':
    unittest.main()
