"""Actual native axes, public factories, constructor hooks and rendered output.

Run against the installed portal; no geometry, scene, renderer or sink doubles.
"""
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


class CoordinateLifecycleTests(unittest.TestCase):
    def test_group_hooks_precede_factories_and_see_ranges(self):
        for base, count in ((m.Axes, 2), (m.ThreeDAxes, 3)):
            with self.subTest(base=base.__name__):
                class Authored(base):
                    def init_data(self):
                        self.events = ["data"]
                        self.range_at_init = tuple(self.x_range)
                        super().init_data()
                    def init_points(self):
                        self.events.append("points")
                        super().init_points()
                    def init_uniforms(self):
                        self.events.append("uniforms")
                        super().init_uniforms()
                    def init_colors(self):
                        self.events.append("colors")
                        super().init_colors()
                    def create_axis(self, *args, **kwargs):
                        self.events.append("axis")
                        return super().create_axis(*args, **kwargs)
                chart = Authored(x_range=(-2, 3, .5), axis_config={"include_ticks": False})
                self.assertEqual(chart.events, ["data", "points", "uniforms", "colors"] + ["axis"] * count)
                self.assertEqual(chart.range_at_init, (-2., 3., .5))
                self.assertEqual(len(chart.get_axes()), count)
                self.assertEqual(chart.dimension, count)

    def test_class_config_defaults_change_real_axis_paint(self):
        class Authored(m.Axes):
            default_axis_config = {"color": m.RED, "stroke_width": 7}
            default_y_axis_config = {"color": m.GREEN}
        chart = Authored(axis_config={"include_ticks": False}, x_axis_config={"stroke_width": 3})
        self.assertEqual(chart.x_axis.get_stroke_color(), m.RED)
        self.assertEqual(chart.y_axis.get_stroke_color(), m.GREEN)
        self.assertAlmostEqual(chart.x_axis.get_stroke_width(), 3)
        self.assertAlmostEqual(chart.y_axis.get_stroke_width(), 7)

    def test_factory_results_are_the_live_chart_axes(self):
        made = []
        class Authored(m.Axes):
            def create_axis(self, *args, **kwargs):
                axis = super().create_axis(*args, **kwargs)
                axis.stretch(2 if not made else 3, 0, about_point=m.ORIGIN)
                made.append(axis)
                return axis
        chart = Authored(x_range=(-1, 1), y_range=(-1, 1), axis_config={"include_ticks": False})
        self.assertIs(chart.x_axis, made[0])
        self.assertIs(chart.y_axis, made[1])
        np.testing.assert_allclose(chart.c2p(1, 1), [2, 3, 0], atol=1e-6)
        scene = m.Scene()
        scene.add(chart)
        chart.shift(m.RIGHT)
        np.testing.assert_allclose(chart.c2p(1, 1), [3, 3, 0], atol=1e-6)
        self.assertIs(chart.x_axis, made[0])

    def test_group_schema_and_authored_children_survive_construction(self):
        class Authored(m.Axes):
            data_dtype = m.VMobject.data_dtype + [("weight", 1)]
            def init_data(self):
                self.resize(1)
                self.set_field("weight", 0, [17])
                self.marker = m.Line([-1, -1, 0], [1, 1, 0])
                self.add(self.marker)
        chart = Authored(axis_config={"include_ticks": False})
        self.assertIn("weight", chart.data.dtype.names)
        self.assertEqual(float(chart.get_field("weight", 0)[0]), 17)
        self.assertIs(chart.submobjects[0], chart.marker)
        self.assertIn(chart.x_axis, chart.submobjects)
        self.assertIn(chart.y_axis, chart.submobjects)
        m.Scene().add(chart)
        self.assertIn("weight", chart.data.dtype.names)

    def test_cooperative_factory_mixin_and_multilevel_hooks(self):
        calls = []
        class FactoryMixin:
            def create_axis(self, *args, **kwargs):
                calls.append("mixin")
                return super().create_axis(*args, **kwargs)
        class Parent(m.Axes):
            def init_points(self):
                calls.append("parent")
                super().init_points()
        class Child(FactoryMixin, Parent):
            pass
        Child(axis_config={"include_ticks": False})
        self.assertEqual(calls, ["parent", "mixin", "mixin"])

    def test_config_and_nested_defaults_do_not_alias_factories(self):
        config = {"decimal_number_config": {"num_decimal_places": 2}}
        class Authored(m.Axes):
            default_axis_config = {"decimal_number_config": {"font_size": 24}}
            def create_axis(self, range_terms, axis_config, length):
                axis_config["decimal_number_config"]["font_size"] = 15
                return super().create_axis(range_terms, axis_config, length)
        chart = Authored(axis_config=config)
        self.assertEqual(config, {"decimal_number_config": {"num_decimal_places": 2}})
        self.assertEqual(Authored.default_axis_config["decimal_number_config"]["font_size"], 24)
        self.assertEqual(chart._axes_params[2], config)

    def test_three_d_z_factory_and_scale_convention(self):
        calls = []
        class Authored(m.ThreeDAxes):
            def create_axis(self, range_terms, axis_config, length):
                calls.append((tuple(range_terms), dict(axis_config), length))
                return super().create_axis(range_terms, axis_config, length)
        chart = Authored(x_range=(-1, 1), y_range=(-1, 1), z_range=(-2, 2),
                         unit_size=2, axis_config={"include_ticks": False}, z_axis_config={"unit_size": 3})
        self.assertEqual(len(calls), 3)
        self.assertEqual(calls[-1][1]["unit_size"], 3)
        np.testing.assert_allclose(chart.c2p(1, 1, 1), [2, 2, 3], atol=1e-6)
        plain = m.ThreeDAxes(unit_size=2, axis_config={"include_ticks": False})
        np.testing.assert_allclose(plain.c2p(1, 1, 1), [2, 2, 1], atol=1e-6)

    def test_sizes_and_asymmetric_origins_are_live(self):
        chart = m.Axes(x_range=(-1, 3), y_range=(-2, 4), width=8, height=6,
                       axis_config={"include_ticks": False})
        np.testing.assert_allclose(chart.x_axis.get_start(), [-4, -1, 0], atol=1e-6)
        np.testing.assert_allclose(chart.y_axis.get_end(), [-2, 3, 0], atol=1e-6)
        np.testing.assert_allclose(chart.c2p(2, 2), [2, 1, 0], atol=1e-6)

    def test_invalid_ranges_fail_before_authored_hooks(self):
        calls = []
        class Authored(m.Axes):
            def init_points(self): calls.append("init")
        for options in ({"y_range": (1, 0)}, {"x_range": (-1, 1, 0)},
                        {"width": float("inf")}, {"unit_size": 0}, {"x_range": (0, 1, 1, 2)}):
            with self.subTest(options=options), self.assertRaises((ValueError, TypeError)):
                Authored(**options)
        self.assertEqual(calls, [])

    def test_bad_factory_and_foreign_scene_refuse_before_rotation(self):
        class Bad(m.Axes):
            def create_axis(self, *args, **kwargs): return m.Square()
        with self.assertRaisesRegex(TypeError, "NumberLine"):
            Bad()
        foreign = m.NumberLine((-1, 1), include_ticks=False)
        scene = m.Scene()
        scene.add(foreign)
        before = foreign.get_points().copy()
        class Foreign(m.Axes):
            def create_axis(self, *args, **kwargs): return foreign
        with self.assertRaisesRegex(ValueError, "detached"):
            Foreign()
        np.testing.assert_array_equal(foreign.get_points(), before)

    def test_factories_cannot_share_or_adopt_the_first_axis(self):
        class Shared(m.Axes):
            def create_axis(self, *args, **kwargs):
                if hasattr(self, "x_axis"): return self.x_axis
                return super().create_axis(*args, **kwargs)
        with self.assertRaisesRegex(ValueError, "independent"):
            Shared()
        scene = m.Scene()
        class Adopting(m.Axes):
            def create_axis(self, *args, **kwargs):
                if hasattr(self, "x_axis"): scene.add(self.x_axis)
                return super().create_axis(*args, **kwargs)
        with self.assertRaisesRegex(ValueError, "detached"):
            Adopting()

    def test_factory_exception_is_not_retried(self):
        failure, calls = RuntimeError("authored axis"), []
        class Broken(m.Axes):
            def create_axis(self, *args, **kwargs):
                calls.append("axis")
                raise failure
        with self.assertRaises(RuntimeError) as caught:
            Broken()
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, ["axis"])
        self.assertEqual(len(m.Axes().get_axes()), 2)

    def test_all_rendered_frames_follow_authored_axis_geometry(self):
        def render(path, threads, authored):
            scene = m.Scene()
            with scene.render_session(path, format="png_sequence", resolution=(128, 72), fps=4, threads=threads):
                if authored:
                    class Wide(m.Axes):
                        def create_axis(self, *args, **kwargs):
                            axis = super().create_axis(*args, **kwargs)
                            axis.stretch(2, 0, about_point=m.ORIGIN)
                            return axis
                    chart = Wide(x_range=(-1, 1), y_range=(-1, 1), axis_config={"include_ticks": False})
                else:
                    # Independent geometry: literal expected endpoints, not
                    # another Axes or a call to its patched factory.
                    chart = m.VGroup(m.NumberLine((-2, 2), include_ticks=False),
                                     m.NumberLine((-2, 2), include_ticks=False).rotate(m.PI / 2))
                scene.add(chart)
                scene.play(m.Transform(chart, chart.copy().shift(m.RIGHT), run_time=1, rate_func=m.linear))
            return [p.read_bytes() for p in sorted(Path(path).glob('*.png'))]
        root = Path(tempfile.mkdtemp(prefix="fmn-coordinate-native-"))
        frames = render(root/'one', 1, True)
        self.assertEqual(len(frames), 4)
        self.assertGreater(len(set(frames)), 1)
        self.assertEqual(frames, render(root/'four', 4, True))
        self.assertEqual(frames, render(root/'expected', 1, False))
        print('retained coordinate-native frames:', root)


if __name__ == '__main__':
    unittest.main()
