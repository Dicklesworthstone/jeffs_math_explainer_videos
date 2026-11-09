"""Actual-native graph construction, first-error isolation and output equivalence."""
import functools
import inspect
import math
from pathlib import Path
import tempfile
import unittest
import weakref
import gc

import numpy as np
import manimlib as m
from fmn_python.graph_admission import install_graph_admission


_REQUESTS = (
    (m.ParametricCurve, dict(t_range=(0, 1, .2), use_smoothing=False)),
    (m.FunctionGraph, dict(x_range=(0, 1, .2), use_smoothing=False)),
    (m.ImplicitFunction, dict(x_range=(-1, 1), y_range=(-1, 1), min_depth=1, max_quads=16)),
)


def _raw_curve(function, *, t_range=(0, 1, .2), epsilon=1e-8, discontinuities=(), smooth=False):
    obj = m.VMobject.__new__(m.VMobject)
    m._install_live_state(obj)
    specs = m._FMN_ADMITTED_ORIGINAL_build_parametric_curve(obj, m._native_shell_factory, function,
                    t_range, epsilon, list(discontinuities), smooth)
    m._hang_native_children(obj, specs)
    return obj


def _raw_graph(function, *, x_range=(0, 1, .2), epsilon=1e-8, discontinuities=(), smooth=False):
    obj = m.VMobject.__new__(m.VMobject)
    m._install_live_state(obj)
    specs = m._FMN_ADMITTED_ORIGINAL_build_function_graph(obj, m._native_shell_factory, function,
                    x_range, epsilon, list(discontinuities), smooth)
    m._hang_native_children(obj, specs)
    return obj


def _raw_implicit(function, *, x_range=(-1, 1), y_range=(-1, 1), min_depth=1, max_quads=16, smooth=False):
    obj = m.VMobject.__new__(m.VMobject)
    m._install_live_state(obj)
    specs = m._FMN_ADMITTED_ORIGINAL_build_implicit_function(obj, m._native_shell_factory, function,
                    x_range, y_range, min_depth, max_quads, smooth)
    m._hang_native_children(obj, specs)
    obj.set_joint_type('no_joint')
    return obj


class GraphAdmissionTests(unittest.TestCase):
    def test_all_three_samplers_stop_at_first_callback_error(self):
        for cls, options in _REQUESTS:
            for stop in (1, 2, 5):
                for error_type in (LookupError, KeyboardInterrupt, SystemExit):
                    with self.subTest(cls=cls, stop=stop, error_type=error_type):
                        calls = []
                        error = error_type('original authored failure')
                        def sample(*args):
                            calls.append(args)
                            if len(calls) == stop:
                                raise error
                            return (args[0], 0, 0) if cls is m.ParametricCurve else args[0]
                        with self.assertRaises(error_type) as caught:
                            cls(sample, **options)
                        self.assertIs(caught.exception, error)
                        self.assertEqual(len(calls), stop)

    def test_result_conversion_failure_has_original_identity(self):
        for cls, options in _REQUESTS:
            with self.subTest(cls=cls):
                calls = []
                error = RuntimeError('conversion')
                class Value:
                    def __float__(self):
                        calls.append('float')
                        raise error
                    def __array__(self, *args, **kwargs):
                        calls.append('array')
                        raise error
                def sample(*args):
                    calls.append('sample')
                    return Value()
                with self.assertRaises(RuntimeError) as caught:
                    cls(sample, **options)
                self.assertIs(caught.exception, error)
                self.assertEqual(calls, ['sample', 'array' if cls is m.ParametricCurve else 'float'])

    def test_parametric_values_require_exact_real_finite_vec3(self):
        for value in ((0, 1), (0, 1, 2, 3), (0, float('nan'), 0), (0, 1e99, 0),
                      np.array([1, 2, 3], dtype=complex), [[1, 2, 3]]):
            with self.subTest(value=value):
                calls = []
                with self.assertRaises(ValueError):
                    m.ParametricCurve(lambda t: calls.append(t) or value, t_range=(0, 1, .25))
                self.assertEqual(calls, [0.])

    def test_function_graph_nonfinite_values_fail_instead_of_poisoning_geometry(self):
        for value in (float('nan'), float('inf'), 1e99):
            calls = []
            with self.assertRaisesRegex(ValueError, 'declare discontinuities'):
                m.FunctionGraph(lambda t: calls.append(t) or value, x_range=(0, 1, .25))
            self.assertEqual(calls, [0.])

    def test_scalar_samplers_do_not_coerce_text_or_complex_samples(self):
        for cls, options in _REQUESTS[1:]:
            for value in ('2', b'2', 2j, np.complex64(2j), np.complex128(0j)):
                with self.subTest(cls=cls, value=value):
                    calls = []
                    with self.assertRaises(TypeError):
                        cls(lambda *args: calls.append(args) or value, **options)
                    self.assertEqual(len(calls), 1)

    def test_user_float_protocol_still_converts_exactly_once_per_scalar_sample(self):
        for cls, options in _REQUESTS[1:]:
            with self.subTest(cls=cls):
                calls, conversions = [], []
                class Value:
                    def __init__(self, x): self.x = x
                    def __float__(self):
                        conversions.append(self.x)
                        return float(self.x)
                def sample(*args):
                    calls.append(args[0])
                    return Value(args[0])
                obj = cls(sample, **options)
                self.assertTrue(calls)
                self.assertEqual(calls, conversions)
                self.assertTrue(np.isfinite(obj.get_points()).all())

    def test_implicit_undefined_regions_retain_native_sample_order_and_geometry(self):
        guarded_calls, raw_calls = [], []
        def field(calls, x, y):
            calls.append((x, y))
            return float('nan') if x < -.25 else x*x + y*y - .5
        options = dict(x_range=(-1, 1), y_range=(-1, 1), min_depth=2, max_quads=64)
        obj = m.ImplicitFunction(functools.partial(field, guarded_calls), **options)
        expected = _raw_implicit(functools.partial(field, raw_calls), **options)
        self.assertEqual(guarded_calls, raw_calls)
        self.assertGreater(obj.get_num_points(), 0)
        np.testing.assert_array_equal(obj.data, expected.data)
        self.assertTrue(np.isfinite(obj.get_points()).all())
        for value in (float('nan'), float('inf'), float('-inf')):
            empty = m.ImplicitFunction(lambda x, y: value, **_REQUESTS[2][1])
            self.assertEqual(empty.get_num_points(), 0)

    def test_nonpositive_step_and_reversed_range_keep_native_endpoint_policy(self):
        for step in (0, -1, -.125):
            calls = []
            obj = m.ParametricCurve(lambda t: calls.append(t) or (t, 0, 0), t_range=(3, -2, step))
            self.assertEqual(calls, [3.])
            np.testing.assert_array_equal(obj.get_points(), [[3, 0, 0]])
        expected = _raw_curve(lambda t: (t, t*t, 0), t_range=(3, -2, .25))
        obj = m.ParametricCurve(lambda t: (t, t*t, 0), t_range=(3, -2, .25), use_smoothing=False)
        np.testing.assert_array_equal(obj.data, expected.data)

    def test_declared_discontinuities_are_still_excluded_by_the_native_sampler(self):
        calls = []
        def sample(x):
            calls.append(x)
            return 1 / x
        obj = m.FunctionGraph(sample, x_range=(-1, 1, .1), discontinuities=(0,), epsilon=.01,
                              use_smoothing=False)
        self.assertEqual(len(obj.get_subpaths()), 2)
        self.assertNotIn(0, calls)
        expected = _raw_graph(lambda x: 1/x, x_range=(-1, 1, .1), discontinuities=(0,), epsilon=.01)
        np.testing.assert_array_equal(obj.get_points(), expected.get_points())

    def test_range_iterables_are_bounded_before_bootstrap_materialization(self):
        for cls, options in _REQUESTS:
            for name in (('t_range',) if cls is m.ParametricCurve else
                         ('x_range', 'y_range') if cls is m.ImplicitFunction else ('x_range',)):
                with self.subTest(cls=cls, name=name):
                    reads, calls = [], []
                    def endless():
                        while True:
                            reads.append(1)
                            yield 1
                    with self.assertRaises(ValueError):
                        cls(lambda *args: calls.append(args) or 0, **dict(options, **{name: endless()}))
                    self.assertEqual(len(reads), 3 if cls is m.ImplicitFunction else 4)
                    self.assertEqual(calls, [])

    def test_discontinuity_iterables_have_bounded_host_materialization(self):
        for cls, options in _REQUESTS[:2]:
            reads, calls = [], []
            def endless():
                while True:
                    reads.append(1)
                    yield 5
            with self.assertRaisesRegex(ValueError, 'discontinuities'):
                cls(lambda *args: calls.append(args) or 0, discontinuities=endless(), **options)
            self.assertEqual(len(reads), 65_537)
            self.assertEqual(calls, [])

    def test_native_sample_and_contour_budgets_still_precede_callbacks(self):
        requests = ((m.ParametricCurve, dict(t_range=(0, 1, 1e-12))),
                    (m.FunctionGraph, dict(x_range=(0, 1, 1e-12))),
                    (m.ImplicitFunction, dict(min_depth=30)),
                    (m.ImplicitFunction, dict(max_quads=65_537)))
        for cls, options in requests:
            calls = []
            with self.assertRaises((ValueError, OverflowError)):
                cls(lambda *args: calls.append(args) or 0, **options)
            self.assertEqual(calls, [])

    def test_nonfinite_controls_precede_callbacks(self):
        for cls, options in _REQUESTS:
            key = 't_range' if cls is m.ParametricCurve else 'x_range'
            for value in ((0, float('inf')), (float('nan'), 1)):
                calls = []
                with self.assertRaises(ValueError):
                    cls(lambda *args: calls.append(args) or 0, **dict(options, **{key: value}))
                self.assertEqual(calls, [])

    def test_iterator_errors_propagate_once_without_running_the_function(self):
        error = RuntimeError('iterator')
        reads, calls = [], []
        def broken():
            reads.append(1)
            yield 0
            raise error
        with self.assertRaises(RuntimeError) as caught:
            m.FunctionGraph(lambda x: calls.append(x) or x, discontinuities=broken())
        self.assertIs(caught.exception, error)
        self.assertEqual(reads, [1])
        self.assertEqual(calls, [])

    def test_metadata_keeps_authored_function_identity_and_ordinary_queries(self):
        f = lambda x: x*x
        v = lambda t: (t, t*t, 0)
        field = lambda x, y: x+y
        curve, graph, implicit = (m.ParametricCurve(v), m.FunctionGraph(f),
                                 m.ImplicitFunction(field, **_REQUESTS[2][1]))
        self.assertIs(curve.t_func, v)
        self.assertIs(graph.function, f)
        self.assertIs(implicit.func, field)
        np.testing.assert_array_equal(curve.get_point_from_function(.5), (.5, .25, 0))
        np.testing.assert_array_equal(graph.get_point_from_function(.5), (.5, .25, 0))

    def test_bound_public_and_private_reconstruction_preserves_live_scene(self):
        for cls, options in _REQUESTS:
            function = (lambda t: (t, 0, 0)) if cls is m.ParametricCurve else (lambda *args: args[0])
            obj = cls(function, **options)
            scene = m.Scene()
            scene.add(obj)
            before, live = obj.data.copy(), obj.get_points()
            calls = []
            with self.assertRaisesRegex(RuntimeError, 'detached target'):
                cls.__init__(obj, lambda *args: calls.append(args) or 0, **options)
            method = obj._build_parametric_curve if cls is m.ParametricCurve else obj._build_function_graph if cls is m.FunctionGraph else obj._build_implicit_function
            with self.assertRaisesRegex(RuntimeError, 'detached target'):
                method()  # Refuse ownership before even converting a request.
            self.assertEqual(calls, [])
            self.assertIs(obj._scene, scene)
            np.testing.assert_array_equal(obj.data, before)
            obj.shift((1, 0, 0))
            np.testing.assert_allclose(live, before['point'] + (1, 0, 0), atol=1e-6)

    def test_direct_seam_failure_leaves_native_records_and_live_views_unchanged(self):
        for name, args in (('_build_parametric_curve', ((0, 1, .2), 1e-8, (), False)),
                           ('_build_function_graph', ((0, 1, .2), 1e-8, (), False)),
                           ('_build_implicit_function', ((-1, 1), (-1, 1), 1, 16, False))):
            obj = m.Line((0, 0, 0), (1, 1, 0))
            before, live = obj.data.copy(), obj.get_points()
            calls, error = [], RuntimeError('direct')
            def bad(*args):
                calls.append(args)
                raise error
            with self.assertRaises(RuntimeError) as caught:
                getattr(obj, name)(m._native_shell_factory, bad, *args)
            self.assertIs(caught.exception, error)
            self.assertEqual(len(calls), 1)
            np.testing.assert_array_equal(obj.data, before)
            np.testing.assert_array_equal(live, before['point'])

    def test_recursive_sampler_failure_is_local_to_each_construction(self):
        calls = []
        def sample(t):
            calls.append(t)
            try:
                m.ImplicitFunction(lambda x, y: (_ for _ in ()).throw(ValueError('inner')), **_REQUESTS[2][1])
            except ValueError:
                pass
            return t, t*t, 0
        obj = m.ParametricCurve(sample, **_REQUESTS[0][1])
        self.assertEqual(len(calls), 6)
        self.assertGreater(obj.get_num_points(), 1)

    def test_callback_objects_are_not_retained_after_their_graph_dies(self):
        class Function:
            def __call__(self, t): return t*t
        function = Function()
        ref = weakref.ref(function)
        obj = m.FunctionGraph(function, x_range=(0, 1, .25))
        del obj, function
        gc.collect()
        self.assertIsNone(ref())

    def test_idempotent_installation_preserves_signatures(self):
        method = m.Mobject._build_function_graph
        constructor = m.FunctionGraph.__init__
        signature = inspect.signature(constructor)
        install_graph_admission(m._FMN_ROOT)
        self.assertIs(m.Mobject._build_function_graph, method)
        self.assertIs(m.FunctionGraph.__init__, constructor)
        self.assertEqual(inspect.signature(constructor), signature)

    def test_safe_native_records_and_rendered_animation_are_unchanged(self):
        curve_f, scalar_f = (lambda t: (t, math.sin(t), 0)), math.sin
        field_f = lambda x, y: x*x+y*y-.5
        guarded = (m.ParametricCurve(curve_f, t_range=(-2, 2, .2)),
                   m.FunctionGraph(scalar_f, x_range=(-2, 2, .2)),
                   m.ImplicitFunction(field_f, **_REQUESTS[2][1]))
        raw = (_raw_curve(curve_f, t_range=(-2, 2, .2), smooth=True),
               _raw_graph(scalar_f, x_range=(-2, 2, .2), smooth=True),
               _raw_implicit(field_f))
        for obj, original in zip(guarded, raw):
            np.testing.assert_array_equal(obj.get_points(), original.get_points())
            obj.set_color(m.BLUE)
            original.set_color(m.BLUE)
        outputs = []
        with tempfile.TemporaryDirectory(prefix='fmn-graph-admitted-') as directory:
            for objects in (guarded, raw):
                for threads in (1, 4):
                    class Render(m.Scene):
                        def construct(self):
                            group = m.VGroup(*(obj.copy() for obj in objects))
                            self.add(group)
                            self.play(group.animate.shift((.5, .5, 0)), run_time=.25, rate_func=m.linear)
                    path = Path(directory) / f'{len(outputs)}.y4m'
                    receipt = Render().render(path, format='y4m', resolution=(64, 36), fps=8, threads=threads)
                    self.assertEqual(receipt.frame_count, 2)
                    outputs.append(path.read_bytes())
        self.assertEqual(len(set(outputs)), 1)


def run_graph_admission_acceptance():
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(GraphAdmissionTests))
    if not result.wasSuccessful():
        raise AssertionError('native graph admission acceptance failed')


if __name__ in ('__main__', '<run_path>'):
    run_graph_admission_acceptance()
