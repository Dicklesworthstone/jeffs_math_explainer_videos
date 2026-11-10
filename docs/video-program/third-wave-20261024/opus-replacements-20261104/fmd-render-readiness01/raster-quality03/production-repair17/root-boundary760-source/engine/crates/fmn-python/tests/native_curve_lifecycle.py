"""Authored sampled curves on the real native object, clock and output paths."""
import copy
from pathlib import Path
import tempfile
import unittest

import manimlib as m
import numpy as np
from fmn_python import render_session


CASES = (
    (m.ParametricCurve, lambda t: (t, t * t, 0), {"t_range": (-1, 1, .25)}),
    (m.FunctionGraph, lambda t: t * t, {"x_range": (-1, 1, .25)}),
)


class NativeCurveLifecycle(unittest.TestCase):
    def test_hooks_run_once_in_order_with_recipe_ready(self):
        for base, function, options in CASES:
            with self.subTest(base=base.__name__):
                events = []

                class Authored(base):
                    def init_data(self):
                        events.append("data")
                        self.recipe_on_entry = self.t_func
                        super().init_data()

                    def init_points(self):
                        events.append("points")
                        super().init_points()
                        self.shift(m.UP)

                    def init_uniforms(self):
                        events.append("uniforms")
                        super().init_uniforms()
                        self.uniforms["curve_author"] = 17.0

                    def init_colors(self):
                        events.append("colors")
                        super().init_colors()
                        self.set_stroke(m.GREEN, width=6)

                obj = Authored(function, use_smoothing=False, **options)
                reference = base(function, use_smoothing=False, **options).shift(m.UP)
                self.assertEqual(events, ["data", "points", "uniforms", "colors"])
                self.assertIs(obj.recipe_on_entry, obj.t_func)
                self.assertEqual(obj.uniforms["curve_author"], 17)
                self.assertEqual(obj.get_stroke_color(), m.GREEN)
                self.assertEqual(obj.get_stroke_width(), 6)
                np.testing.assert_array_equal(obj.get_points(), reference.get_points())

    def test_initial_records_match_the_native_builder_with_discontinuities(self):
        # Independent native construction, not another public constructor.
        # In particular this catches normal placeholders and recomputation of
        # joint angles after float32 quantization at a subpath break.
        for smoothing in (False, True):
            for breaks in ((), (0,)):
                with self.subTest(smoothing=smoothing, breaks=breaks):
                    function = lambda t: (t, t*t, 0)
                    native = m.VMobject()
                    native._build_parametric_curve(
                        m._native_shell_factory, function, (-1, 1, .25),
                        .05, list(breaks), smoothing,
                    )
                    curve = m.ParametricCurve(
                        function, t_range=(-1, 1, .25), epsilon=.05,
                        discontinuities=breaks, use_smoothing=smoothing,
                    )
                    self.assertEqual(curve.data.tobytes(), native.data.tobytes())

    def test_override_can_replace_sampling_entirely(self):
        for base, _, options in CASES:
            with self.subTest(base=base.__name__):
                def forbidden(*args):
                    raise AssertionError("the overridden native sampler must not run")

                class Authored(base):
                    def init_points(self):
                        self.records_before = self.get_num_points()
                        self.set_points([[-1, 2, 0], [0, 2, 0], [1, 2, 0]])

                obj = Authored(forbidden, **options)
                self.assertEqual(obj.records_before, 0)
                np.testing.assert_array_equal(obj.get_points(), [[-1, 2, 0], [0, 2, 0], [1, 2, 0]])

    def test_mixin_super_chain_and_recipe_edits_drive_native_sampling(self):
        calls = []

        class Mixin:
            def init_points(self):
                calls.append("mixin")
                super().init_points()
                self.shift(m.UP)

        class Parent(m.ParametricCurve):
            def init_points(self):
                calls.append("parent")
                self.t_func = lambda t: (t, 2 * t, 1)
                self.t_range = (-.5, .5, .25)
                super().init_points()

        class Child(Mixin, Parent):
            pass

        obj = Child(lambda t: (t, 0, 0), use_smoothing=False)
        reference = m.ParametricCurve(lambda t: (t, 2*t, 1), t_range=(-.5, .5, .25),
                                     use_smoothing=False).shift(m.UP)
        self.assertEqual(calls, ["mixin", "parent"])
        np.testing.assert_array_equal(obj.get_points(), reference.get_points())

    def test_init_data_records_and_children_survive_sampling(self):
        for base, function, options in CASES:
            with self.subTest(base=base.__name__):
                class Authored(base):
                    data_dtype = m.VMobject.data_dtype + [("weight", 2)]

                    def init_data(self):
                        self.resize(1)
                        self.set_field("weight", 0, [3, 7])
                        self.label = m.Dot(point=3*m.RIGHT)
                        self.add(self.label)

                obj = Authored(function, **options)
                self.assertIs(obj[0], obj.label)
                np.testing.assert_array_equal(obj.get_field("weight", 0), [3, 7])
                scene = m.Scene()
                scene.add(obj)
                obj.init_points()
                self.assertIs(scene.mobjects[0], obj)
                self.assertIs(obj[0], obj.label)
                np.testing.assert_array_equal(obj.get_field("weight", 0), [3, 7])

    def test_copies_do_not_repeat_authored_construction(self):
        calls = []

        class Authored(m.FunctionGraph):
            def init_points(self):
                calls.append("points")
                super().init_points()
                self.shift(m.UP)

        obj = Authored(lambda t: t*t, x_range=(-1, 1, .25))
        for clone in (obj.copy(), copy.deepcopy(obj)):
            self.assertIs(type(clone), Authored)
            np.testing.assert_array_equal(obj.get_points(), clone.get_points())
        self.assertEqual(calls, ["points"])

    def test_hook_failures_propagate_once_and_stop_construction(self):
        for base, function, options in CASES:
            for name in ("init_data", "init_points", "init_uniforms", "init_colors"):
                with self.subTest(base=base.__name__, hook=name):
                    failure, calls = KeyboardInterrupt("authored " + name), []

                    def fail(self):
                        calls.append(name)
                        raise failure

                    cls = type("Broken", (base,), {name: fail})
                    with self.assertRaises(KeyboardInterrupt) as caught:
                        cls(function, **options)
                    self.assertIs(caught.exception, failure)
                    self.assertEqual(calls, [name])
                    self.assertGreater(base(function, **options).get_num_points(), 0)

    def test_sampling_failure_is_atomic_and_recoverable(self):
        class Authored(m.ParametricCurve):
            def init_points(self):
                super().init_points()
                self.shift(m.UP)

        obj = Authored(lambda t: (t, t, 0), t_range=(0, 1, .1), color=m.BLUE)
        scene = m.Scene()
        scene.add(obj)
        before = obj.data.copy()
        failure, seen = ValueError("curve sampling failure"), []

        def broken(t):
            seen.append(t)
            raise failure

        obj.t_func = broken
        with self.assertRaises(ValueError) as caught:
            obj.init_points()
        self.assertIs(caught.exception, failure)
        self.assertEqual(len(seen), 1)
        np.testing.assert_array_equal(obj.data, before)
        obj.t_func = lambda t: (t, 2*t, 0)
        obj.init_points()
        self.assertIs(obj._scene, scene)
        self.assertEqual(obj.get_stroke_color(), m.BLUE)
        self.assertGreater(obj.get_end()[1], 2.9)

    def test_callback_cannot_reenter_regeneration_or_overwrite_its_own_edits(self):
        obj = m.ParametricCurve(lambda t: (t, 0, 0), t_range=(0, 1, .25))
        before = obj.get_points().copy()

        def reenter(t):
            obj.init_points()
            return t, 0, 0

        obj.t_func = reenter
        with self.assertRaisesRegex(RuntimeError, "reenter"):
            obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), before)
        changed = [False]

        def mutate(t):
            if not changed[0]:
                changed[0] = True
                obj.shift(m.UP)
            return t, 3, 0

        obj.t_func = mutate
        with self.assertRaisesRegex(RuntimeError, "changed during sampling"):
            obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), before + m.UP)

    def test_graph_options_and_aliases_remain_live(self):
        function = lambda t: t*t
        graph = m.FunctionGraph(function, x_range=(-2, 2, .5), epsilon=.02,
                                discontinuities=[0], use_smoothing=False)
        self.assertIs(graph.get_function(), function)
        self.assertEqual(graph.get_x_range(), (-2, 2, .5))
        self.assertEqual(graph.get_stroke_color(), m.YELLOW)
        self.assertEqual(graph.get_fill_opacity(), 0)
        from manimlib.mobject.functions import FunctionGraph, ParametricCurve
        self.assertIs(FunctionGraph, m.FunctionGraph)
        self.assertIs(ParametricCurve, m.ParametricCurve)

    def test_reinitializing_bound_curve_refuses_without_damage(self):
        obj = m.FunctionGraph(lambda t: t, x_range=(0, 1, .25))
        scene = m.Scene()
        scene.add(obj)
        before = obj.data.copy()
        with self.assertRaisesRegex(RuntimeError, "detached"):
            obj.__init__(lambda t: 7, x_range=(0, 1, .25))
        np.testing.assert_array_equal(obj.data, before)
        self.assertIs(obj._scene, scene)

    def test_authored_curve_reaches_real_png_pixels(self):
        class Raised(m.FunctionGraph):
            def init_points(self):
                super().init_points()
                self.shift(2*m.UP)

        directory = Path(tempfile.mkdtemp(prefix="fmn-curve-lifecycle-"))
        pixels = []
        for index, obj in enumerate((
            Raised(lambda t: 0, x_range=(-2, 2, .5), stroke_width=8),
            m.FunctionGraph(lambda t: 0, x_range=(-2, 2, .5), stroke_width=8).shift(2*m.UP),
            m.FunctionGraph(lambda t: 0, x_range=(-2, 2, .5), stroke_width=8),
        )):
            scene = m.Scene()
            path = directory / f"curve-{index}.png"
            with render_session(scene, path, resolution=(64, 36), fps=24, threads=1):
                scene.add(obj)
                scene.wait(1/24)
            pixels.append(path.read_bytes())
        self.assertTrue(pixels[0].startswith(b"\x89PNG"))
        self.assertEqual(pixels[0], pixels[1])
        self.assertNotEqual(pixels[0], pixels[2])


if __name__ == "__main__":
    unittest.main()
