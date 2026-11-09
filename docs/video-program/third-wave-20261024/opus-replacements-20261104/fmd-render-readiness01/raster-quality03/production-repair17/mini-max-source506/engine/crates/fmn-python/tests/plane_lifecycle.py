"""Native NumberPlane/ComplexPlane authoring, live grid factories and pixels."""
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


class PlaneLifecycleTests(unittest.TestCase):
    def test_constructor_dispatches_all_public_factories(self):
        for base in (m.NumberPlane, m.ComplexPlane):
            calls = []
            class Authored(base):
                def init_points(self):
                    calls.append("points")
                    super().init_points()
                def create_axis(self, *args, **kwargs):
                    calls.append("axis")
                    return super().create_axis(*args, **kwargs)
                def init_background_lines(self):
                    calls.append("background")
                    super().init_background_lines()
                def get_lines(self):
                    calls.append("lines")
                    return super().get_lines()
                def get_lines_parallel_to_axis(self, *args):
                    calls.append("parallel")
                    return super().get_lines_parallel_to_axis(*args)
            plane = Authored(x_range=(-1, 1), y_range=(-1, 1), faded_line_ratio=1)
            self.assertEqual(calls, ["points", "axis", "axis", "background", "lines", "parallel", "parallel"])
            self.assertIs(plane.submobjects[0], plane.faded_lines)
            self.assertIs(plane.submobjects[1], plane.background_lines)
            self.assertIs(plane.axes[0], plane.x_axis)

    def test_grid_follows_the_authored_axes_not_a_second_native_chart(self):
        class Authored(m.NumberPlane):
            def create_axis(self, *args, **kwargs):
                line = super().create_axis(*args, **kwargs)
                line.stretch(2, 0, about_point=m.ORIGIN)
                return line
        plane = Authored(x_range=(-1, 1), y_range=(-1, 1), faded_line_ratio=0)
        np.testing.assert_allclose(plane.c2p(1, 1), [2, 2, 0], atol=1e-6)
        self.assertEqual(len(plane.background_lines), 4)
        horizontal = plane.background_lines[0]
        np.testing.assert_allclose(horizontal.get_start(), [-2, -2, 0], atol=1e-6)
        np.testing.assert_allclose(horizontal.get_end(), [2, -2, 0], atol=1e-6)

    def test_custom_grid_result_is_adopted_without_replanning(self):
        made = []
        class Authored(m.NumberPlane):
            def get_lines(self):
                result = (m.VGroup(m.Line([-1, -1, 0], [1, 1, 0])),
                          m.VGroup(m.Line([-1, 1, 0], [1, -1, 0])))
                made.append(result)
                return result
        plane = Authored(background_line_style={"stroke_color": m.RED, "stroke_width": 5},
                         faded_line_style={"stroke_opacity": .4})
        self.assertEqual(len(made), 1)
        self.assertIs(plane.background_lines, made[0][0])
        self.assertIs(plane.faded_lines, made[0][1])
        self.assertEqual(plane.background_lines[0].get_stroke_color(), m.RED)
        self.assertEqual(plane.faded_lines[0].get_stroke_color(), m.RED)
        self.assertAlmostEqual(plane.background_lines[0].get_stroke_width(), 5)
        self.assertAlmostEqual(plane.faded_lines[0].get_stroke_opacity(), .4, places=6)

    def test_live_ratio_controls_subsequent_get_lines(self):
        plane = m.NumberPlane(x_range=(-1, 1), y_range=(-1, 1), faded_line_ratio=1)
        children = tuple(plane.submobjects)
        self.assertEqual((len(plane.background_lines), len(plane.faded_lines)), (4, 4))
        plane.faded_line_ratio = 3
        major, minor = plane.get_lines()
        self.assertEqual((len(major), len(minor)), (4, 12))
        self.assertEqual(tuple(plane.submobjects), children)
        self.assertEqual(plane._plane_params[7], 1)

    def test_live_affine_axes_determine_grid_endpoints(self):
        plane = m.NumberPlane(x_range=(-1, 1), y_range=(-1, 1), faded_line_ratio=0)
        plane.apply_matrix([[1, .5, 0], [0, 2, 0], [0, 0, 1]])
        plane.shift(m.RIGHT)
        major, minor = plane.get_lines()
        self.assertEqual(len(minor), 0)
        np.testing.assert_allclose(major[0].get_start(), [-.5, -2, 0], atol=1e-6)
        np.testing.assert_allclose(major[0].get_end(), [1.5, -2, 0], atol=1e-6)

    def test_grid_respects_public_number_mapping_overrides(self):
        plane = m.NumberPlane(x_range=(-1, 1), y_range=(-1, 1), faded_line_ratio=0)
        original = plane.y_axis.n2p
        seen = []
        def mapping(x):
            seen.append(float(x))
            return original(x) + np.array([0, 0, float(x)])
        plane.y_axis.n2p = mapping
        major, minor = plane.get_lines_parallel_to_axis(plane.x_axis, plane.y_axis)
        self.assertEqual(seen, [0., -1., 1.])
        np.testing.assert_allclose(major[0].get_start(), [-1, -1, -1], atol=1e-6)

    def test_style_defaults_and_input_dicts_are_independent(self):
        background, faded = {"stroke_width": 3}, {"stroke_opacity": .1}
        first = m.NumberPlane(background_line_style=background, faded_line_style=faded)
        second = m.NumberPlane()
        self.assertEqual(background, {"stroke_width": 3})
        self.assertEqual(faded, {"stroke_opacity": .1})
        first.background_line_style["stroke_width"] = 20
        self.assertEqual(second.background_line_style["stroke_width"], 2)
        self.assertEqual(first._plane_params[5]["stroke_width"], 3)
        self.assertEqual(first.faded_line_style["stroke_color"], m.BLUE_D)

    def test_authored_children_and_schema_survive_plane_construction(self):
        class Authored(m.NumberPlane):
            data_dtype = m.VMobject.data_dtype + [("weight", 1)]
            def init_data(self):
                self.resize(1)
                self.set_field("weight", 0, [9])
                self.marker = m.Line([-1, -1, 0], [1, 1, 0])
                self.add(self.marker)
        plane = Authored(x_range=(-1, 1), y_range=(-1, 1))
        self.assertIn(plane.marker, plane.submobjects)
        self.assertIn("weight", plane.data.dtype.names)
        self.assertEqual(float(plane.get_field("weight", 0)[0]), 9)
        self.assertEqual(len(plane.axes), 2)

    def test_complex_mapping_uses_the_authored_chart(self):
        class Authored(m.ComplexPlane):
            def create_axis(self, *args, **kwargs):
                line = super().create_axis(*args, **kwargs)
                line.scale(2, about_point=m.ORIGIN)
                return line
        plane = Authored(x_range=(-1, 1), y_range=(-1, 1))
        np.testing.assert_allclose(plane.n2p(.5+.25j), [1, .5, 0], atol=1e-6)
        self.assertAlmostEqual(plane.p2n([1, .5, 0]), .5+.25j)

    def test_invalid_grid_controls_refuse_before_hooks_or_grid_allocation(self):
        calls = []
        class Authored(m.NumberPlane):
            def init_points(self): calls.append("init")
        for kwargs in ({"faded_line_ratio": -1}, {"faded_line_ratio": True},
                       {"faded_line_ratio": 1.5}, {"x_range": (-1, 1, 1e-12)},
                       {"y_range": (0, float('inf'), 1)},
                       {"x_range": (-1, 1, .0001), "y_range": (-1, 1, .0001), "faded_line_ratio": 1}):
            with self.subTest(kwargs=kwargs), self.assertRaises((ValueError, TypeError)):
                Authored(**kwargs)
        self.assertEqual(calls, [])

    def test_bad_or_shared_factory_groups_are_not_published(self):
        for result in (None, (), (m.VGroup(),), (m.Line(), m.VGroup())):
            class Authored(m.NumberPlane):
                def get_lines(self): return result
            obj = Authored.__new__(Authored)
            with self.assertRaises((TypeError, ValueError)):
                obj.__init__(x_range=(-1, 1), y_range=(-1, 1))
            self.assertEqual(tuple(obj.submobjects), (obj.x_axis, obj.y_axis))
        group = m.VGroup(m.Line())
        class Shared(m.NumberPlane):
            def get_lines(self): return group, group
        with self.assertRaisesRegex(ValueError, "independent"):
            Shared()

    def test_foreign_factory_geometry_is_not_restyled(self):
        scene = m.Scene()
        group = m.VGroup(m.Line(color=m.RED))
        scene.add(group)
        class Foreign(m.NumberPlane):
            def get_lines(self): return group, m.VGroup()
        with self.assertRaisesRegex(ValueError, "detached"):
            Foreign(background_line_style={"stroke_color": m.GREEN})
        self.assertEqual(group[0].get_stroke_color(), m.RED)

    def test_factory_failure_does_not_retry_or_publish_a_half_grid(self):
        failure, calls = RuntimeError('grid failure'), []
        class Broken(m.NumberPlane):
            def get_lines(self):
                calls.append('get_lines')
                raise failure
        obj = Broken.__new__(Broken)
        with self.assertRaises(RuntimeError) as caught:
            obj.__init__()
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, ['get_lines'])
        self.assertEqual(tuple(obj.submobjects), (obj.x_axis, obj.y_axis))

    def test_real_frames_match_native_grid_builder_at_one_four_threads(self):
        def render(path, threads, native):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(128, 72), fps=4, threads=threads):
                if native:
                    g = vars(m._native)
                    plane = g['_native_shell_factory']()
                    children = plane._build_number_plane(g['_native_shell_factory'],
                        (-2., 2., 1.), (-1., 1., 1.), {}, {}, {}, None, None, 2, None, None, 1.)
                    g['_hang_native_children'](plane, children)
                else:
                    plane = m.NumberPlane(x_range=(-2, 2, 1), y_range=(-1, 1, 1), faded_line_ratio=2)
                scene.add(plane)
                scene.play(m.Transform(plane, plane.copy().shift(m.RIGHT), run_time=1, rate_func=m.linear))
            return [p.read_bytes() for p in sorted(Path(path).glob('*.png'))]
        root = Path(tempfile.mkdtemp(prefix='fmn-plane-native-'))
        frames = render(root/'one', 1, False)
        self.assertEqual(len(frames), 4)
        self.assertGreater(len(set(frames)), 1)
        self.assertEqual(frames, render(root/'four', 4, False))
        self.assertEqual(frames, render(root/'native', 1, True))
        print('retained plane-native frames:', root)


if __name__ == '__main__':
    unittest.main()
