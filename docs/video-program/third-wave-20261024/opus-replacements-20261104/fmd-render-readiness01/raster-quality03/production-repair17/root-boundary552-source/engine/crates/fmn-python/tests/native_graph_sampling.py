"""Real raw native graph builders: no FirstFailure wrapper or mocked sampler."""
from pathlib import Path
import tempfile
import unittest
import numpy as np
import manimlib as m


KINDS = ('parametric_curve', 'function_graph', 'implicit_function')


def options(kind):
    return ((-1., 1.), (-1., 1.), 1, 7, False) if kind == 'implicit_function' else ((0., 1., .25), 1e-8, [], False)


def sample(kind, *args):
    if kind == 'parametric_curve':
        return args[0], args[0] * args[0], 0.
    if kind == 'function_graph':
        return args[0] * args[0]
    x, y = args
    return x * x + y * y - .375


def raw(kind, obj, function, controls=None):
    method = getattr(m, '_FMN_ADMITTED_ORIGINAL_build_' + kind)
    return method(obj, m._native_shell_factory, function, *(options(kind) if controls is None else controls))


class NativeGraphSamplingTests(unittest.TestCase):
    def test_original_exceptions_terminate_raw_native_loops_at_every_stage(self):
        for kind in KINDS:
            trace = []
            raw(kind, m.VMobject(), lambda *a: trace.append(a) or sample(kind, *a))
            for stop in sorted({1, 2, len(trace) // 2, len(trace)}):
                for error_type in (RuntimeError, KeyboardInterrupt, SystemExit):
                    with self.subTest(kind=kind, stop=stop, error_type=error_type):
                        obj = m.Line((0, 0, 0), (1, 1, 0))
                        before, view = obj.data.copy(), obj.get_points()
                        calls, error = [], error_type('original native callback')
                        def function(*args):
                            calls.append(args)
                            if len(calls) == stop:
                                raise error
                            return sample(kind, *args)
                        with self.assertRaises(error_type) as caught:
                            raw(kind, obj, function)
                        self.assertIs(caught.exception, error)
                        self.assertEqual(calls, trace[:stop])
                        np.testing.assert_array_equal(obj.data, before)
                        np.testing.assert_array_equal(view, before['point'])

    def test_numeric_conversion_exception_is_not_masked_or_repeated(self):
        for kind in KINDS:
            calls, conversions = [], []
            error = LookupError('conversion')
            class Number:
                def __float__(self):
                    conversions.append(1)
                    raise error
            def function(*args):
                calls.append(args)
                return (0., Number(), 0.) if kind == 'parametric_curve' else Number()
            with self.assertRaises(LookupError) as caught:
                raw(kind, m.VMobject(), function)
            self.assertIs(caught.exception, error)
            self.assertEqual(len(calls), 1)
            self.assertEqual(conversions, [1])

    def test_raw_curve_requires_exactly_three_components(self):
        for point in ((1., 2.), (1., 2., 3., 4.)):
            calls = []
            with self.assertRaises(ValueError):
                raw('parametric_curve', m.VMobject(), lambda t: calls.append(t) or point)
            self.assertEqual(len(calls), 1)

    def test_raw_point_records_refuse_nonfinite_and_overflow_values_immediately(self):
        for kind in KINDS[:2]:
            for value in (float('nan'), float('inf'), -float('inf'), 1e100):
                obj = m.Line()
                before = obj.data.copy()
                calls = []
                def function(*args):
                    calls.append(args)
                    return (args[0], value, 0.) if kind == 'parametric_curve' else value
                with self.assertRaises(ValueError):
                    raw(kind, obj, function)
                self.assertEqual(len(calls), 1)
                np.testing.assert_array_equal(obj.data, before)

    def test_raw_constructor_ownership_refuses_before_authored_work(self):
        for kind in KINDS:
            obj, scene, calls = m.Line(), m.Scene(), []
            scene.add(obj)
            before, view = obj.data.copy(), obj.get_points()
            with self.assertRaisesRegex(RuntimeError, 'detached'):
                raw(kind, obj, lambda *a: calls.append(a) or sample(kind, *a))
            self.assertEqual(calls, [])
            self.assertIs(obj._scene, scene)
            np.testing.assert_array_equal(obj.data, before)
            np.testing.assert_array_equal(view, before['point'])

    def test_raw_admission_still_precedes_all_callbacks(self):
        for kind in KINDS:
            controls = list(options(kind))
            if kind == 'implicit_function':
                controls[2] = 32
            else:
                controls[0] = (0., 1., 1e-100)
            calls = []
            with self.assertRaises(ValueError):
                raw(kind, m.VMobject(), lambda *a: calls.append(a) or sample(kind, *a), controls)
            self.assertEqual(calls, [])

    def test_undefined_implicit_regions_are_not_exceptions(self):
        for undefined in (float('nan'), float('inf'), -float('inf')):
            obj = m.VMobject()
            raw('implicit_function', obj, lambda x, y: undefined if x < 0 else y - .125)
            points = obj.get_points()
            self.assertGreater(len(points), 0)
            self.assertTrue(np.isfinite(points).all())
            self.assertTrue((points[:, 0] >= 0).all())
            np.testing.assert_allclose(points[:, 1], .125, atol=1e-6)

    def test_nested_raw_failures_do_not_poison_outer_sampling(self):
        outer, inner = [], []
        error = ValueError('inner')
        def function(t):
            outer.append(t)
            def fail(*args):
                inner.append(args)
                raise error
            with self.assertRaises(ValueError) as caught:
                raw('implicit_function', m.VMobject(), fail)
            self.assertIs(caught.exception, error)
            return t, t, 0.
        obj = m.VMobject()
        raw('parametric_curve', obj, function)
        self.assertEqual(outer, [0., .25, .5, .75, 1.])
        self.assertEqual(len(inner), 5)
        np.testing.assert_allclose(obj.get_points()[:, 0], obj.get_points()[:, 1], atol=0)

    def test_raw_failure_preserves_authored_side_effects_without_repeating_them(self):
        for kind in KINDS:
            obj = m.Line()
            before, view = obj.get_points().copy(), obj.get_points()
            calls, error = [], RuntimeError('side effect')
            def function(*args):
                calls.append(args)
                obj.shift((1, 0, 0))
                raise error
            with self.assertRaises(RuntimeError) as caught:
                raw(kind, obj, function)
            self.assertIs(caught.exception, error)
            self.assertEqual(len(calls), 1)
            np.testing.assert_allclose(view, before + (1, 0, 0), atol=1e-6)

    def test_safe_raw_paths_render_like_an_independent_line_scene_at_one_four_threads(self):
        class Scene(m.Scene):
            direct = False
            def construct(self):
                if self.direct:
                    curve = m.Line((-1, -.5, 0), (1, .5, 0), buff=0)
                else:
                    curve = m.VMobject()
                    raw('parametric_curve', curve, lambda t: (2*t-1, t-.5, 0),
                        ((0., 1., 0.), 1e-8, [], False))
                    # Endpoint-only sampling cannot form a line; exercise its
                    # specified semantics above, and use two samples for this oracle.
                    raw('parametric_curve', curve, lambda t: (2*t-1, t-.5, 0),
                        ((0., 1., 1.), 1e-8, [], False))
                curve.set_stroke(m.BLUE, width=3)
                self.add(curve)
                self.play(curve.animate.shift((0, .5, 0)), run_time=.5, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-native-graphs-') as directory:
            results = []
            for direct, threads in ((True, 1), (False, 1), (False, 4)):
                scene = Scene()
                scene.direct = direct
                path = Path(directory) / f'{direct}-{threads}.y4m'
                receipt = scene.render(path, format='y4m', resolution=(96, 54), fps=8, threads=threads)
                self.assertEqual(receipt.frame_count, 4)
                results.append(path.read_bytes())
            self.assertEqual(results[0], results[1])
            self.assertEqual(results[1], results[2])


def run_native_graph_sampling():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeGraphSamplingTests))
    if not result.wasSuccessful():
        raise AssertionError('raw native fallible graph sampling failed')


if __name__ in ('__main__', '<run_path>'):
    run_native_graph_sampling()
