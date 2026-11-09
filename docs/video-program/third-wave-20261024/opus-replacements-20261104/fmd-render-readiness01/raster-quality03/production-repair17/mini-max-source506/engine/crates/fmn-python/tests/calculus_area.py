"""Real-native integral regions: exact curves, disconnected topology and playback.

Run against the installed portal. No substitute for native geometry/rendering,
no missing-extension skips, and no reference-function resampling as an oracle.
"""
from __future__ import annotations

import itertools
import math
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import manimlib as m
import numpy as np

from fmn_python import render_scene


def frames(path):
    data = Path(path).read_bytes()
    header, _, payload = data.partition(b'\n')
    if not header.startswith(b'YUV4MPEG2 '):
        raise AssertionError('expected native Y4M')
    fields = header.split()
    width = int(next(x[1:] for x in fields if x.startswith(b'W')))
    height = int(next(x[1:] for x in fields if x.startswith(b'H')))
    if not any(x in (b'C420', b'C420jpeg', b'C420mpeg2') for x in fields):
        raise AssertionError('expected native 8-bit 4:2:0 output')
    size = width * height * 3 // 2
    result = []
    for start in range(0, len(payload), size + 6):
        if payload[start:start + 6] != b'FRAME\n':
            raise AssertionError('incorrect native frame boundary')
        frame = payload[start + 6:start + 6 + size]
        if len(frame) != size:
            raise AssertionError('truncated native frame')
        result.append(np.frombuffer(frame[:width * height], dtype=np.uint8).reshape(height, width))
    return data, result


class GraphAreaAcceptance(unittest.TestCase):
    def axes(self):
        return m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=8, height=4)

    def line(self, axes, points):
        return m.VMobject().set_points_as_corners([axes.c2p(x, y) for x, y in points])

    def coordinates(self, axes, points):
        return np.array([axes.p2c(p) for p in points])

    def test_nonuniform_samples_clip_by_x_not_curve_index(self):
        axes = self.axes()
        graph = self.line(axes, [(0, 1), (.1, 1), (2, 1)])
        graph.x_range = (0, 2)
        area = axes.get_area_under_graph(graph, (.5, 1.5))
        np.testing.assert_allclose(self.coordinates(axes, area.get_points()[::2]),
                                   [[.5, 1], [1.5, 1], [1.5, 0], [.5, 0], [.5, 1]], atol=2e-6)

    def test_plain_geometric_graph_needs_no_analytic_metadata(self):
        axes = self.axes()
        graph = self.line(axes, [(-1, 1), (2, 1)])
        area = axes.get_area_under_graph(graph)
        self.assertIsInstance(area, m.VMobject)
        self.assertTrue(area.is_closed())
        np.testing.assert_allclose(self.coordinates(axes, area.get_points()[::2]),
                                   [[-1, 1], [2, 1], [2, 0], [-1, 0], [-1, 1]], atol=2e-6)

    def test_quadratic_handles_are_exact_after_nonuniform_parameter_inversion(self):
        axes = self.axes()
        graph = m.VMobject().set_points([axes.c2p(0, 0), axes.c2p(.5, 0), axes.c2p(2, 2)])
        # x(t)=t+t^2, y(t)=2t^2, with analytically known parameter roots.
        t0, t1 = ((math.sqrt(1 + 4 * x) - 1) / 2 for x in (.5, 1.5))
        dt = t1 - t0
        expected = [[.5, 2 * t0 * t0], [.5 + dt * (.5 + t0), 2 * t0 * t0 + 2 * dt * t0],
                    [1.5, 2 * t1 * t1]]
        area = axes.get_area_under_graph(graph, (.5, 1.5))
        np.testing.assert_allclose(self.coordinates(axes, area.get_points()[:3]), expected, atol=2e-6)
        self.assertEqual(area.get_num_points(), 9)

    def test_each_disconnected_component_closes_at_its_own_baseline(self):
        axes = self.axes()
        graph = self.line(axes, [(-2, 1), (-.5, 1)])
        graph.add_subpath(self.line(axes, [(.5, -1), (2, -1)]).get_points())
        area = axes.get_area_under_graph(graph, (-.75, .75))
        self.assertEqual(len(area.get_subpaths()), 2)
        for path, expected in zip(area.get_subpaths(), [(-.75, -.5), (.5, .75)]):
            coords = self.coordinates(axes, path)
            np.testing.assert_allclose([coords[:, 0].min(), coords[:, 0].max()], expected, atol=2e-6)
            np.testing.assert_array_equal(path[0], path[-1])
        self.assertEqual(axes.get_area_under_graph(graph, (-.4, .4)).get_num_points(), 0)

    def test_reversed_path_has_the_same_geometric_region(self):
        axes = self.axes()
        graph = self.line(axes, [(-2, .5), (-.7, 1.2), (.2, .4), (2, 1)])
        forward = axes.get_area_under_graph(graph, (-1, 1))
        graph.reverse_points()
        backward = axes.get_area_under_graph(graph, (-1, 1))
        np.testing.assert_array_equal(forward.get_points(), backward.get_points())

    def test_analytic_discontinuities_keep_the_drawn_exclusion_interval(self):
        axes = self.axes()
        graph = axes.get_graph(lambda x: 1 / x, (-1, 1, .2), discontinuities=(0,),
                               epsilon=.05, use_smoothing=False)
        original = graph.get_subpaths()
        self.assertEqual(len(original), 2)
        area = axes.get_area_under_graph(graph, (-.75, .75))
        paths = area.get_subpaths()
        self.assertEqual(len(paths), 2)
        for source, region in zip(original, paths):
            source_x = self.coordinates(axes, source)[:, 0]
            region_x = self.coordinates(axes, region)[:, 0]
            self.assertGreaterEqual(region_x.min() + 1e-6, max(-.75, source_x.min()))
            self.assertLessEqual(region_x.max() - 1e-6, min(.75, source_x.max()))

    def test_rotated_scaled_translated_and_tilted_axes_preserve_region(self):
        for angle, direction in ((math.pi / 2, m.OUT), (math.pi, m.OUT), (.6, m.RIGHT)):
            with self.subTest(angle=angle):
                axes = self.axes().stretch(1.3, 0).rotate(angle, direction).shift(m.UP)
                graph = self.line(axes, [(-1, -1), (1, -1)])
                area = axes.get_area_under_graph(graph, (-.5, .5))
                np.testing.assert_allclose(area.get_points()[::2], [axes.c2p(x, y) for x, y in
                    [(-.5, -1), (.5, -1), (.5, 0), (-.5, 0), (-.5, -1)]], atol=2e-6)

    def test_source_live_views_callbacks_and_scene_membership_remain_owned(self):
        axes = self.axes()
        graph = axes.get_graph(lambda x: 1)
        scene = m.Scene()
        scene.add(graph)
        view = graph.get_points()
        before = view.copy()
        failure = AssertionError('area extraction must not run source callbacks')
        def forbidden(*args):
            raise failure
        graph.underlying_function = forbidden
        graph.add_updater(forbidden, call=False)
        area = axes.get_area_under_graph(graph, (-1, 1))
        np.testing.assert_array_equal(view, before)
        np.testing.assert_array_equal(graph.get_points(), before)
        self.assertIs(graph.underlying_function, forbidden)
        self.assertEqual(graph.updaters, [forbidden])
        self.assertEqual(area.updaters, [])
        self.assertFalse(hasattr(area, 'underlying_function'))
        self.assertIn(graph, scene.mobjects)
        self.assertNotIn(area, scene.mobjects)
        other = m.Scene()
        other.add(area)
        other.wait(.125)
        np.testing.assert_array_equal(graph.get_points(), before)

    def test_area_is_frozen_and_follows_drawn_geometry_not_stale_range(self):
        axes = self.axes()
        graph = self.line(axes, [(-1, 1), (1, 1)])
        graph.x_range = (50, 100)
        graph.underlying_function = lambda x: 100
        area = axes.get_area_under_graph(graph, (-.5, .5))
        before = area.get_points().copy()
        self.assertAlmostEqual(axes.p2c(area.get_start())[1], 1., places=5)
        graph.shift(m.UP)
        np.testing.assert_array_equal(area.get_points(), before)

    def test_fill_and_projection_style_survive_without_sharing_uniform_lists(self):
        axes = self.axes()
        graph = self.line(axes, [(-1, 1), (1, 1)]).fix_in_frame()
        graph.set_shading(.2, .3, .4)
        graph.set_clip_plane(m.RIGHT, -.7)
        area = axes.get_area_under_graph(graph, (-.5, .5), fill_color=m.YELLOW, fill_opacity=.3)
        np.testing.assert_allclose(area.get_fill_opacities(), .3)
        np.testing.assert_allclose(area.get_stroke_widths(), 0)
        np.testing.assert_allclose(m.color_to_rgb(area.get_fill_color()), m.color_to_rgb(m.YELLOW))
        self.assertEqual(area.uniforms['is_fixed_in_frame'], 1.)
        self.assertEqual(area.uniforms['shading'], graph.uniforms['shading'])
        self.assertEqual(area.uniforms['clip_planes'], graph.uniforms['clip_planes'])
        self.assertIsNot(area.uniforms['clip_planes'], graph.uniforms['clip_planes'])

    def test_empty_zero_width_and_disjoint_requests_are_empty_native_paths(self):
        axes = self.axes()
        graph = self.line(axes, [(-1, 1), (1, 1)])
        for source, bounds in ((m.VMobject(), None), (graph, (0, 0)), (graph, (3, 4)),
                               (m.VMobject().set_points([axes.c2p(0, 1)]), None)):
            area = axes.get_area_under_graph(source, bounds)
            self.assertEqual(area.get_num_points(), 0)
            self.assertEqual(area.updaters, [])

    def test_numpy_and_iterator_bounds_are_not_modified(self):
        axes = self.axes()
        graph = self.line(axes, [(-1, 1), (1, 1)])
        bounds = np.array([-.5, .5])
        expected = axes.get_area_under_graph(graph, tuple(bounds)).get_points().copy()
        for value in (bounds, iter(bounds)):
            np.testing.assert_array_equal(axes.get_area_under_graph(graph, value).get_points(), expected)
        np.testing.assert_array_equal(bounds, [-.5, .5])

    def test_invalid_requests_and_foldbacks_fail_without_source_changes(self):
        axes = self.axes()
        graph = self.line(axes, [(-1, 1), (1, 1)])
        before = graph.get_points().copy()
        for bounds in ([], [0], [0, 1, 2], [1, 0], [0, np.nan], [0, np.inf], itertools.repeat(1)):
            with self.subTest(bounds=bounds), self.assertRaises((ValueError, TypeError)):
                axes.get_area_under_graph(graph, bounds)
        for opacity in (-1, 2, np.nan):
            with self.assertRaises(ValueError):
                axes.get_area_under_graph(graph, fill_opacity=opacity)
        with self.assertRaisesRegex(ValueError, 'folds back'):
            axes.get_area_under_graph(self.line(axes, [(-1, 0), (1, 1), (0, 2)]))
        with self.assertRaises(TypeError):
            axes.get_area_under_graph(object())
        np.testing.assert_array_equal(graph.get_points(), before)

    def test_input_and_output_record_budgets_fail_before_publishing_geometry(self):
        axes = self.axes()
        graph = self.line(axes, [(-1, 1), (1, 1)])
        before = graph.get_points().copy()
        with patch('fmn_python.calculus._MAX_RECORDS', 2), self.assertRaisesRegex(ValueError, 'bounded'):
            axes.get_area_under_graph(graph)
        with patch('fmn_python.calculus._MAX_RECORDS', 5), self.assertRaisesRegex(ValueError, 'record budget'):
            axes.get_area_under_graph(graph)
        np.testing.assert_array_equal(graph.get_points(), before)

    def test_rendered_disconnected_area_matches_independent_polygons(self):
        def scene_class(helper):
            class AreaScene(m.Scene):
                def construct(self):
                    axes = m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=8, height=4)
                    if helper:
                        graph = m.VMobject().set_points_as_corners([axes.c2p(-2, 1), axes.c2p(-.5, 1)])
                        graph.add_subpath(m.VMobject().set_points_as_corners([
                            axes.c2p(.5, 1), axes.c2p(2, 1)]).get_points())
                        area = axes.get_area_under_graph(graph, (-1.5, 1.5), m.WHITE, 1)
                    else:
                        area = m.VGroup(*(m.Polygon(*[axes.c2p(x, y) for x, y in
                            [(a, 1), (b, 1), (b, 0), (a, 0)]], fill_color=m.WHITE,
                            fill_opacity=1, stroke_width=0) for a, b in [(-1.5, -.5), (.5, 1.5)]))
                    self.add(area)
                    self.wait(.25)
            return AreaScene
        with tempfile.TemporaryDirectory(prefix='fmn-graph-area-') as directory:
            outputs = []
            for helper, threads in ((False, 1), (True, 1), (True, 4)):
                path = Path(directory) / f'area-{helper}-{threads}.y4m'
                render_scene(scene_class(helper), path, format='y4m', resolution=(160, 90), fps=8, threads=threads)
                data, images = frames(path)
                self.assertEqual(len(images), 2)
                self.assertGreater(np.count_nonzero(images[0] > 100), 200)
                # No fill should be visible in the missing middle interval:
                # nothing there may rise more than 24 luma codes above the
                # background (16 on black, ~60 on the default #333333).
                self.assertLess(int(images[0][:, 75:85].max()), int(images[0][0, 0]) + 24)
                outputs.append(data)
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual(outputs[1], outputs[2])

    def test_tracker_driven_integral_updates_through_real_playback(self):
        class IntegralScene(m.Scene):
            def construct(self):
                axes = m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=8, height=4)
                graph = m.VMobject().set_points_as_corners([axes.c2p(-1, 1), axes.c2p(1, 1)])
                stop = m.ValueTracker(-.75)
                area = m.always_redraw(lambda: axes.get_area_under_graph(graph, (-1, stop.get_value()), m.WHITE, 1))
                self.add(area)
                self.play(stop.animate.set_value(1.), run_time=1, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-integral-playback-') as directory:
            outputs = []
            for threads in (1, 4):
                path = Path(directory) / f'integral-{threads}.y4m'
                receipt = render_scene(IntegralScene, path, format='y4m', resolution=(160, 90), fps=8, threads=threads)
                data, images = frames(path)
                self.assertEqual(len(images), receipt.frame_count)
                self.assertGreaterEqual(len(images), 8)
                amounts = [np.count_nonzero(image > 100) for image in images]
                self.assertTrue(all(a < b for a, b in zip(amounts, amounts[1:])), amounts)
                outputs.append(data)
            self.assertEqual(outputs[0], outputs[1])


suite = unittest.defaultTestLoader.loadTestsFromTestCase(GraphAreaAcceptance)
if suite.countTestCases() != 16:
    raise AssertionError('graph area acceptance inventory drift')
result = unittest.TextTestRunner(verbosity=2).run(suite)
if not result.wasSuccessful():
    raise AssertionError('graph area native acceptance failed')
