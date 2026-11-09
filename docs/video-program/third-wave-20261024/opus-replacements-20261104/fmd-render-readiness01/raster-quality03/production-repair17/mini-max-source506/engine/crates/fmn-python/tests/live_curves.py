"""Live sampled paths, native ownership, refusal recovery and real frame output."""
import copy
from pathlib import Path
import tempfile
import unittest

import manimlib as m
import numpy as np


def curve(function=None, **kwargs):
    return m.ParametricCurve(function or (lambda t: (t, t, 0)),
                             t_range=(-1, 1, .25), use_smoothing=False, **kwargs)


def analytic_points(level):
    ts = np.linspace(-1., 1., 9)
    samples = np.column_stack((ts, level * ts, np.zeros(len(ts))))
    result = np.empty((17, 3))
    result[::2] = samples
    result[1::2] = (samples[:-1] + samples[1:]) / 2
    return result


class LiveCurveTests(unittest.TestCase):
    def test_current_recipe_updates_in_place_without_appending(self):
        level = [1.]
        function = lambda t: (t, level[0] * t, 0)
        obj = curve(function)
        level[0] = 2.
        self.assertIs(obj.init_points(), obj)
        np.testing.assert_allclose(obj.get_points(), analytic_points(2.), atol=1e-7)
        first = obj.get_points().copy()
        obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), first)
        self.assertIs(obj.get_t_func(), function)
        from manimlib.mobject.functions import ParametricCurve
        self.assertIs(ParametricCurve, m.ParametricCurve)

    def test_scalar_graph_refresh_uses_its_parameter_recipe(self):
        level = [1.]
        function = lambda x: level[0] * x
        obj = m.FunctionGraph(function, x_range=(-1, 1, .25), use_smoothing=False)
        t_func = obj.t_func
        level[0] = 3.
        obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), analytic_points(3.))
        self.assertIs(obj.get_function(), function)
        self.assertIs(obj.t_func, t_func)
        obj.t_func = lambda t: (t, t*t, t)
        obj.t_range = (-.5, .5, .25)
        obj.init_points()
        expected = m.ParametricCurve(obj.t_func, t_range=obj.t_range, use_smoothing=False)
        np.testing.assert_array_equal(obj.get_points(), expected.get_points())

    def test_live_range_step_discontinuities_and_smoothing_match_native_builder(self):
        obj = curve(lambda t: (t, t*t, t*t*t))
        for domain, jumps, smooth in [((-2, 3, .5), (), False),
                                     ((-2, 3, .125), (-.5, 1.), True),
                                     ((2, -1, -.25), (), False),
                                     ((0, 0, .25), (), False)]:
            with self.subTest(domain=domain, smooth=smooth):
                obj.t_range, obj.discontinuities = domain, jumps
                obj.epsilon, obj.use_smoothing = .05, smooth
                obj.init_points()
                expected = m.ParametricCurve(obj.t_func, t_range=domain, epsilon=.05,
                                             discontinuities=jumps, use_smoothing=smooth)
                np.testing.assert_array_equal(obj.get_points(), expected.get_points())
                self.assertEqual(len(obj.get_subpaths()), len(expected.get_subpaths()))

    def test_scene_owner_annotations_updaters_style_and_clock_survive(self):
        obj = curve(color=m.BLUE, stroke_width=5)
        child = m.Dot()
        calls = []
        callback = lambda current, dt: calls.append(dt)
        obj.add(child).add_updater(callback, call=False)
        scene = m.Scene()
        scene.add(obj)
        initial_time = scene.get_time()
        obj.t_func = lambda t: (t, 3*t, 1.)
        obj.init_points()
        self.assertIs(obj._scene, scene)
        self.assertTrue(obj._is_bound())
        self.assertEqual(tuple(scene.mobjects), (obj,))
        self.assertIs(obj.submobjects[0], child)
        self.assertIs(obj.updaters[0], callback)
        self.assertEqual(scene.get_time(), initial_time)
        self.assertEqual(calls, [])
        np.testing.assert_allclose(obj.get_stroke_widths(), 5)

    def test_same_size_live_views_and_varying_record_styles_survive(self):
        obj = curve()
        obj.data['stroke_rgba'][:, 0] = np.linspace(0, 1, obj.get_num_points())
        styles = obj.data['stroke_rgba'].copy()
        view = obj.get_points()
        obj.t_func = lambda t: (t, 4*t, 0.)
        obj.init_points()
        np.testing.assert_array_equal(view, analytic_points(4.))
        np.testing.assert_array_equal(obj.data['stroke_rgba'], styles)

    def test_resized_views_detach_and_saved_state_stays_unchanged(self):
        obj = curve()
        view, original = obj.get_points(), obj.get_points().copy()
        obj.save_state()
        obj.t_range = (-1., 1., .125)
        obj.init_points()
        self.assertGreater(obj.get_num_points(), len(view))
        np.testing.assert_array_equal(view, original)
        np.testing.assert_array_equal(obj.saved_state.get_points(), original)
        expected = obj.get_points().copy()
        view[:] = 100
        np.testing.assert_array_equal(obj.get_points(), expected)

    def test_copy_and_deepcopy_share_callable_but_not_refreshed_geometry(self):
        level = [1.]
        obj = curve(lambda t: (t, level[0]*t, 0.))
        original = obj.get_points().copy()
        for cloned in (obj.copy(), copy.copy(obj), copy.deepcopy(obj)):
            level[0] = 2.
            cloned.init_points()
            np.testing.assert_array_equal(cloned.get_points(), analytic_points(2.))
            np.testing.assert_array_equal(obj.get_points(), original)
            self.assertIs(cloned.t_func, obj.t_func)

    def test_bound_method_recipe_and_subclass_dispatch_remain_real(self):
        class Wave:
            level = 1.
            def sample(self, t):
                return t, self.level*t, 0.
        class Custom(m.ParametricCurve):
            def init_points(self):
                self.refreshes = getattr(self, 'refreshes', 0) + 1
                return super().init_points()
        wave = Wave()
        obj = Custom(wave.sample, t_range=(-1, 1, .25), use_smoothing=False)
        # Construction now dispatches the same authored hook as an explicit
        # refresh. Assert both calls separately rather than hiding a skipped
        # constructor behind the old refresh-only count.
        self.assertEqual(obj.refreshes, 1)
        np.testing.assert_array_equal(obj.get_points(), analytic_points(1.))
        wave.level = 5.
        obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), analytic_points(5.))
        self.assertEqual(obj.refreshes, 2)

    def test_errors_and_cancellation_stop_once_preserve_records_and_allow_retry(self):
        for error in (LookupError('curve'), KeyboardInterrupt('cancel'), SystemExit('stop')):
            obj = curve()
            before = obj.data.copy()
            calls = []
            def fail(t):
                calls.append(t)
                raise error
            obj.t_func = fail
            with self.assertRaises(type(error)) as caught:
                obj.init_points()
            self.assertIs(caught.exception, error)
            self.assertEqual(len(calls), 1)
            np.testing.assert_array_equal(obj.data, before)
            obj.t_func = lambda t: (t, 2*t, 0.)
            obj.init_points()
            np.testing.assert_array_equal(obj.get_points(), analytic_points(2.))

    def test_invalid_numeric_sample_never_publishes(self):
        for result in ((0, np.nan, 0), (0, 1e100, 0), (0, 0), (0, 0, 0, 0)):
            obj = curve()
            before = obj.data.copy()
            calls = []
            obj.t_func = lambda t: calls.append(t) or result
            with self.assertRaises((ValueError, TypeError)):
                obj.init_points()
            self.assertEqual(len(calls), 1)
            np.testing.assert_array_equal(obj.data, before)

    def test_invalid_and_unbounded_controls_refuse_before_authored_sampling(self):
        import itertools
        for key, value in [('t_range', (0, 1, 1e-30)), ('t_range', itertools.repeat(1.)),
                           ('epsilon', np.nan), ('discontinuities', itertools.repeat(0.))]:
            obj = curve()
            before = obj.data.copy()
            calls = []
            obj.t_func = lambda t: calls.append(t) or (t, t, 0.)
            setattr(obj, key, value)
            with self.assertRaises((ValueError, TypeError)):
                obj.init_points()
            self.assertEqual(calls, [])
            np.testing.assert_array_equal(obj.data, before)

    def test_reentrancy_refuses_without_losing_existing_geometry(self):
        obj = curve()
        before = obj.data.copy()
        obj.t_func = lambda t: obj.init_points()
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            obj.init_points()
        np.testing.assert_array_equal(obj.data, before)

    def test_callback_record_edits_are_preserved_instead_of_overwritten(self):
        obj = curve()
        obj.t_func = lambda t: (obj.shift((0, .01, 0)), (t, t*t, 0))[1]
        with self.assertRaisesRegex(RuntimeError, 'changed during'):
            obj.init_points()
        self.assertGreater(obj.get_points()[0, 1], -1.)
        self.assertNotIn('_fmn_function_graph_busy', vars(obj))

    def test_callback_recipe_family_and_owner_changes_are_detected(self):
        for kind in ('recipe', 'family', 'owner', 'schema'):
            with self.subTest(kind=kind):
                obj = curve()
                before = obj.get_points().copy()
                scene, child = m.Scene(), m.Dot()
                changed = [False]
                def sample(t):
                    if not changed[0]:
                        changed[0] = True
                        if kind == 'recipe': obj.t_range = (0, 2, .1)
                        elif kind == 'family': obj.add(child)
                        elif kind == 'owner': scene.add(obj)
                        else: obj.pointlike_data_keys = ['point', 'other']
                    return t, t*t, 0.
                obj.t_func = sample
                with self.assertRaisesRegex(RuntimeError, 'changed during'):
                    obj.init_points()
                np.testing.assert_array_equal(obj.get_points(), before)
                if kind == 'family': self.assertIs(obj[0], child)
                if kind == 'owner': self.assertIs(obj._scene, scene)

    def test_direct_animation_and_live_function_binding_have_one_geometry_owner(self):
        obj = curve()
        animation = m.Transform(obj, obj.copy().shift(m.UP))
        animation.begin()
        with self.assertRaisesRegex(RuntimeError, 'active animation'):
            obj.init_points()
        animation.finish()
        obj.init_points()
        axes = m.Axes(x_range=(-1, 1), y_range=(-1, 1))
        graph = axes.get_graph(lambda x: x, bind=True, use_smoothing=False)
        with self.assertRaisesRegex(RuntimeError, 'unbind'):
            graph.init_points()
        axes.unbind_graph_from_func(graph)
        graph.init_points()

    def test_native_rendered_refresh_matches_independent_corner_path_at_all_threads(self):
        class Live(m.Scene):
            def construct(self):
                level = m.ValueTracker(.25)
                obj = curve(lambda t: (t, level.get_value()*t, 0.), color=m.BLUE, stroke_width=4)
                obj.add_updater(lambda current: current.init_points(), call=False)
                self.add(obj)
                self.play(level.animate.set_value(1.), run_time=.5, rate_func=m.linear)
        class Reference(m.Scene):
            def construct(self):
                level = m.ValueTracker(.25)
                obj = m.VMobject(color=m.BLUE, stroke_width=4)
                obj.set_points(analytic_points(.25))
                obj.add_updater(lambda current: current.set_points(analytic_points(level.get_value())), call=False)
                self.add(obj)
                self.play(level.animate.set_value(1.), run_time=.5, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-live-curves-') as directory:
            outputs = []
            for cls in (Live, Reference):
                for threads in (1, 4):
                    path = Path(directory) / f'{cls.__name__}-{threads}.y4m'
                    receipt = cls().render(path, format='y4m', resolution=(96, 54), fps=8, threads=threads)
                    self.assertEqual(receipt.frame_count, 4)
                    outputs.append(path.read_bytes())
            self.assertTrue(all(output == outputs[0] for output in outputs))
            body = outputs[0].split(b'\n', 1)[1]
            stride = 6 + 96*54*3//2
            self.assertNotEqual(body[:stride], body[-stride:])


if __name__ == '__main__':
    unittest.main()
