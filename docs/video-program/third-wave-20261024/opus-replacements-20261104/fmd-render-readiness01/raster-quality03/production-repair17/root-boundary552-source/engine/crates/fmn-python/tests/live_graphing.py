"""Actual installed-portal graph sampling, updater, geometry and output acceptance.

No native substitute and no missing-extension skip. Run this against a wheel
built from the matching source, or through scripts/check_portal_runtime.sh.
"""
from __future__ import annotations

import math
from pathlib import Path
import tempfile
import unittest

import manimlib as m
import numpy as np

from fmn_python import render_scene


def y4m_frames(data):
    header, separator, tail = data.partition(b'\n')
    if not separator or not header.startswith(b'YUV4MPEG2 '):
        raise AssertionError('expected native Y4M output')
    fields = header.split()[1:]
    width = int(next(value[1:] for value in fields if value.startswith(b'W')))
    height = int(next(value[1:] for value in fields if value.startswith(b'H')))
    chroma = next(value for value in fields if value.startswith(b'C'))
    if chroma not in {b'C420', b'C420jpeg', b'C420mpeg2'} or width % 2 or height % 2:
        raise AssertionError('expected 8-bit 4:2:0 output with even dimensions')
    size, position, frames = width * height * 3 // 2, 0, []
    while position < len(tail):
        if tail[position:position + 6] != b'FRAME\n':
            raise AssertionError('bad native frame marker')
        start = position + 6
        frame = tail[start:start + size]
        if len(frame) != size:
            raise AssertionError('truncated native frame')
        frames.append(frame)
        position = start + size
    return width, height, frames


class RecordComparison(unittest.TestCase):
    def test_record_comparison_is_array_equal(self):
        # fm-5wq.31: the per-frame "changed during sampling" check compares
        # bytes first; np.array_equal still decides every other case.
        from fmn_python.graphing import _records_equal
        graph = m.Square()
        before = graph.data.copy()
        nan = before.copy()
        nan["point"][0, 0] = np.nan
        signed = before.copy()
        signed["point"][0, 2] = -0.0
        zero = before.copy()
        zero["point"][0, 2] = 0.0
        changed = before.copy()
        changed["point"][1, 0] += 1
        renamed = before.copy()
        renamed.dtype.names = tuple(f"renamed_{index}" for index in range(len(before.dtype.names)))
        cases = {
            "identical live view": (graph.data, before),
            "identical copies": (before.copy(), before),
            "one value changed": (changed, before),
            "identical NaN records": (nan, nan.copy()),
            "-0.0 against 0.0": (signed, zero),
            "shorter": (before[:-1], before),
            "renamed fields": (renamed, before),
            "plain arrays": (graph.get_points(), graph.get_points().copy()),
            "plain arrays with NaN": (np.array([np.nan]), np.array([np.nan])),
        }
        for name, (current, previous) in cases.items():
            with self.subTest(name):
                try:
                    expected = bool(np.array_equal(current, previous))
                except Exception as error:
                    with self.assertRaises(type(error)):
                        _records_equal(np, current, previous)
                    continue
                self.assertIs(_records_equal(np, current, previous), expected)
        self.assertIs(_records_equal(np, nan, nan.copy()), False)
        self.assertIs(_records_equal(np, signed, zero), True)


class LiveGraphingAcceptance(unittest.TestCase):
    def axes(self):
        return m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=8, height=4)

    def test_scalar_math_function_updates_on_the_native_scene_clock(self):
        axes = self.axes()
        amplitude = m.ValueTracker(.5)
        calls = []
        def function(x):
            self.assertIs(type(x), float)
            calls.append(x)
            return amplitude.get_value() * math.sin(x)
        graph = axes.get_graph(function, bind=True, color=m.BLUE)
        scene = m.Scene()
        scene.add(graph)
        calls.clear()
        before = graph.get_points().copy()
        scene.play(amplitude.animate.set_value(1.5), run_time=.25)
        self.assertTrue(calls)
        self.assertFalse(np.array_equal(graph.get_points(), before))
        np.testing.assert_allclose(axes.i2gp(.5, graph), axes.c2p(.5, 1.5 * math.sin(.5)), atol=1e-5)
        self.assertEqual(len(graph.updaters), 1)

    def test_binding_takes_the_function_shape_without_a_frame(self):
        # fm-skpp: the Reference binds through add_updater(call=True), so a
        # bound graph follows its function before any scene frame runs.
        axes = self.axes()
        graph = axes.get_graph(lambda x: 0.)
        self.assertAlmostEqual(graph.get_height(), 0.)
        axes.bind_graph_to_func(graph, lambda xs: np.exp(-xs ** 2))
        np.testing.assert_allclose(axes.i2gp(0., graph), axes.c2p(0., 1.), atol=1e-5)
        self.assertGreater(graph.get_height(), .9)
        self.assertEqual(len(graph.updaters), 1)

    def test_moving_discontinuity_keeps_full_domain_and_disconnected_native_paths(self):
        axes = self.axes()
        cut = m.ValueTracker(0)
        graph = axes.get_graph(lambda x: x, color=m.YELLOW, stroke_width=6)
        graph.epsilon = .01
        axes.bind_graph_to_func(graph, lambda xs: np.where(xs < cut.get_value(), -.75, .75),
                                jagged=False, get_discontinuities=lambda: [cut.get_value()])
        scene = m.Scene()
        scene.add(graph)
        for value in (0., .5, -.75, 1., 0., 1.25):
            cut.set_value(value)
            scene.wait(.125)
            paths = graph.get_subpaths()
            self.assertEqual(len(paths), 2)
            self.assertAlmostEqual(axes.p2c(paths[0][0])[0], -2., places=4)
            self.assertAlmostEqual(axes.p2c(paths[-1][-1])[0], 2., places=4)
            self.assertLess(axes.p2c(paths[0][-1])[0], value)
            self.assertGreater(axes.p2c(paths[-1][0])[0], value)
            self.assertTrue(np.isfinite(graph.get_points()).all())
            np.testing.assert_allclose(graph.get_stroke_widths(), 6)

    def test_disappearing_domain_preserves_empty_style_edits_on_recovery(self):
        axes = self.axes()
        hidden = [True]
        graph = axes.get_graph(lambda x: x, color=m.BLUE, stroke_width=5)
        graph.epsilon = 10
        axes.bind_graph_to_func(graph, lambda xs: xs,
                                get_discontinuities=lambda: [0.] if hidden[0] else [])
        graph.update(0)
        self.assertEqual(graph.get_num_points(), 0)
        graph.set_stroke(color=m.YELLOW, width=7, opacity=.6)
        hidden[0] = False
        graph.update(0)
        self.assertGreater(graph.get_num_points(), 2)
        np.testing.assert_allclose(graph.get_stroke_widths(), 7)
        np.testing.assert_allclose(graph.get_stroke_opacities(), .6)

    def test_geometric_queries_read_native_curves_and_skip_gaps(self):
        axes = self.axes()
        left = m.VMobject().set_points_as_corners([axes.c2p(-2, -1), axes.c2p(-.5, -1)])
        right = m.VMobject().set_points_as_corners([axes.c2p(.5, 1), axes.c2p(2, 1)])
        left.add_subpath(right.get_points())
        np.testing.assert_allclose(axes.i2gp(-1, left), axes.c2p(-1, -1), atol=1e-5)
        np.testing.assert_allclose(axes.i2gp(1, left), axes.c2p(1, 1), atol=1e-5)
        self.assertIsNone(axes.i2gp(0, left))
        self.assertIsNone(axes.i2gp(3, left))
        # The x data range need not contain any [0,1] curve parameter at all.
        positive = m.Axes(x_range=(10, 20, 1), y_range=(0, 10, 1))
        line = m.VMobject().set_points_as_corners([positive.c2p(10, 0), positive.c2p(20, 10)])
        np.testing.assert_allclose(positive.i2gp(12, line), positive.c2p(12, 2), atol=1e-5)

    def test_native_quadratic_and_reversed_geometric_lookup(self):
        axes = self.axes()
        graph = m.VMobject()
        graph.set_points([axes.c2p(0, 0), axes.c2p(.5, 0), axes.c2p(2, 2)])
        for x in (.1, .5, 1., 1.8):
            t = (-1 + math.sqrt(1 + 4 * x)) / 2
            np.testing.assert_allclose(axes.i2gp(x, graph), axes.c2p(x, 2 * t * t), atol=2e-5)
        graph.reverse_points()
        np.testing.assert_allclose(axes.i2gp(1, graph), axes.c2p(1, 2 * ((math.sqrt(5) - 1) / 2) ** 2), atol=2e-5)

    def test_native_callback_failure_preserves_last_graph_and_next_wait_recovers(self):
        axes = self.axes()
        should_fail = [False]
        failure = LookupError('live graph callback failed')
        def function(xs):
            if should_fail[0]:
                raise failure
            return np.sin(xs)
        graph = axes.get_graph(math.sin)
        axes.bind_graph_to_func(graph, function)
        scene = m.Scene()
        scene.add(graph)
        scene.wait(.125)
        before = graph.get_points().copy()
        should_fail[0] = True
        with self.assertRaises(LookupError) as raised:
            scene.wait(.125)
        self.assertIs(raised.exception, failure)
        np.testing.assert_array_equal(graph.get_points(), before)
        should_fail[0] = False
        scene.wait(.125)
        self.assertTrue(np.isfinite(graph.get_points()).all())

    def test_copy_and_unbind_keep_native_object_ownership(self):
        axes = self.axes()
        factor = [1.]
        graph = axes.get_graph(lambda x: factor[0] * x, bind=True)
        copied = graph.copy(deep=True)
        before = graph.get_points().copy()
        factor[0] = .5
        copied.update(0)
        np.testing.assert_array_equal(graph.get_points(), before)
        self.assertFalse(np.array_equal(copied.get_points(), before))
        axes.unbind_graph_from_func(copied)
        factor[0] = 2.
        copied.update(0)
        np.testing.assert_allclose(axes.i2gp(1, copied), axes.c2p(1, .5), atol=1e-5)
        self.assertEqual(len(graph.updaters), 1)
        self.assertEqual(len(copied.updaters), 0)

    def test_real_native_frames_change_and_match_across_thread_counts(self):
        class WaveScene(m.Scene):
            def construct(self):
                axes = m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=8, height=4)
                amplitude = m.ValueTracker(.25)
                graph = axes.get_graph(lambda x: amplitude.get_value() * math.sin(2 * x),
                                       bind=True, color=m.WHITE, stroke_width=8)
                self.add(graph)
                self.wait(.125)
                self.play(amplitude.animate.set_value(1.25), run_time=1, rate_func=m.linear)
        with tempfile.TemporaryDirectory(prefix='fmn-live-graphing-') as directory:
            artifacts = []
            for threads in (1, 4):
                path = Path(directory) / f'wave-{threads}.y4m'
                receipt = render_scene(WaveScene, path, format='y4m', resolution=(160, 90),
                                       fps=8, threads=threads)
                data = path.read_bytes()
                width, height, frames = y4m_frames(data)
                self.assertEqual((width, height), (160, 90))
                self.assertEqual(len(frames), receipt.frame_count)
                self.assertGreaterEqual(len(frames), 8)
                self.assertGreater(len(set(frames)), 4)
                for frame in frames:
                    luma = np.frombuffer(frame[:width * height], dtype=np.uint8)
                    self.assertGreater(int(luma.max()) - int(luma.min()), 20)
                artifacts.append(data)
            self.assertEqual(artifacts[0], artifacts[1])


class RiemannAcceptance(unittest.TestCase):
    def axes(self):
        return m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=8, height=4)

    def test_bins_follow_rotated_stretched_and_three_dimensional_axes(self):
        for angle, axis in ((math.pi / 2, m.OUT), (math.pi, m.OUT), (.7, m.RIGHT)):
            for height in (1., -1.):
                with self.subTest(angle=angle, height=height):
                    axes = self.axes().stretch(1.3, 0).rotate(angle, axis).shift(m.UP)
                    graph = axes.get_graph(lambda x: height)
                    rects = axes.get_riemann_rectangles(graph, (0, 1), dx=.4)
                    self.assertEqual(len(rects), 3)
                    for rect, (left, right) in zip(rects, ((0, .4), (.4, .8), (.8, 1))):
                        self.assertIsInstance(rect, m.Rectangle)
                        np.testing.assert_allclose(rect.get_points()[::2], [
                            axes.c2p(left, 0), axes.c2p(right, 0), axes.c2p(right, height),
                            axes.c2p(left, height), axes.c2p(left, 0)], atol=2e-6)
                        self.assertEqual(rect.positive, height > 0)

    def test_partial_last_bin_and_original_range_are_preserved(self):
        axes = self.axes()
        graph = axes.get_graph(lambda x: 1)
        for domain in ([0., 1.], np.array([0., 1.])):
            original = np.array(domain, copy=True)
            rects = axes.get_riemann_rectangles(graph, domain, dx=.3)
            self.assertEqual(len(rects), 4)
            last = np.array([axes.p2c(p) for p in rects[-1].get_points()[::2]])
            np.testing.assert_allclose(last[:, 0], [.9, 1, 1, .9, .9], atol=1e-6)
            np.testing.assert_array_equal(domain, original)
        exact = axes.get_riemann_rectangles(graph, (0, .1 + .2), dx=.1)
        self.assertEqual(len(exact), 3)

    def test_three_value_domain_controls_step_and_final_sample_is_clipped(self):
        axes = self.axes()
        samples = []
        graph = axes.get_graph(lambda x: x)
        graph.underlying_function = lambda x: samples.append(x) or 1.
        rects = axes.get_riemann_rectangles(graph, (0, 1, .4), dx=.2, input_sample_type='right')
        self.assertEqual(len(rects), 3)
        np.testing.assert_allclose(samples, [.4, .8, 1.])

    def test_sampling_modes_use_each_actual_bin(self):
        axes = self.axes()
        graph = axes.get_graph(lambda x: x)
        for mode, samples in (('left', [0., .4, .8]), ('right', [.4, .8, 1.]),
                              ('center', [.2, .6, .9])):
            rects = axes.get_riemann_rectangles(graph, (0, 1), dx=.4, input_sample_type=mode)
            for rect, expected in zip(rects, samples):
                self.assertAlmostEqual(axes.p2c(rect.get_points()[4])[1], expected, places=5)

    def test_unsigned_area_keeps_negative_geometry_and_gradient(self):
        axes = self.axes().rotate(math.pi)
        graph = axes.get_graph(lambda x: -1)
        for signed, color in ((True, m.RED), (False, m.BLUE)):
            rects = axes.get_riemann_rectangles(graph, (0, 1), dx=.5,
                                               colors=(m.BLUE,), show_signed_area=signed)
            for rect in rects:
                self.assertFalse(rect.positive)
                np.testing.assert_allclose(m.color_to_rgb(rect.get_fill_color()), m.color_to_rgb(color))
                self.assertAlmostEqual(axes.p2c(rect.get_points()[4])[1], -1., places=5)

    def test_styles_are_native_and_zero_height_is_not_negative(self):
        axes = self.axes()
        graph = axes.get_graph(lambda x: 0)
        rect = axes.get_riemann_rectangles(graph, (0, 1), dx=1., colors=(m.BLUE,),
                                          fill_opacity=.3, stroke_color=m.YELLOW,
                                          stroke_width=7, stroke_background=False)[0]
        self.assertTrue(rect.positive)
        np.testing.assert_allclose(rect.get_fill_opacities(), .3)
        np.testing.assert_allclose(rect.get_stroke_widths(), 7)
        np.testing.assert_allclose(m.color_to_rgb(rect.get_stroke_color()), m.color_to_rgb(m.YELLOW))
        self.assertFalse(rect.stroke_behind)

    def test_default_colors_are_the_references(self):
        # Pinned Reference: colors=(BLUE, GREEN) for the bins and
        # fill_color=BLUE for get_area_under_graph.
        axes = self.axes()
        graph = axes.get_graph(lambda x: 1)
        rects = axes.get_riemann_rectangles(graph, (0, 1), dx=.25)
        np.testing.assert_allclose(
            m.color_to_rgb(rects[0].get_fill_color()), m.color_to_rgb(m.BLUE), atol=1e-6)
        np.testing.assert_allclose(
            m.color_to_rgb(rects[-1].get_fill_color()), m.color_to_rgb(m.GREEN), atol=1e-6)
        area = axes.get_area_under_graph(graph, (0, 1))
        np.testing.assert_allclose(
            m.color_to_rgb(area.get_fill_color()), m.color_to_rgb(m.BLUE), atol=1e-6)

    def test_invalid_requests_are_rejected_before_authored_callbacks(self):
        axes = self.axes()
        graph = axes.get_graph(lambda x: 1)
        called = []
        graph.underlying_function = lambda x: called.append(x) or 1
        requests = [dict(dx=0), dict(dx=-1), dict(dx=float('nan')), dict(dx=1e-10),
                    dict(x_range=(1, 0)), dict(x_range=(0, float('inf'))),
                    dict(x_range=(0, 1, 2, 3)), dict(input_sample_type='middle'),
                    dict(fill_opacity=float('nan')), dict(stroke_width=-1), dict(colors=())]
        for options in requests:
            with self.subTest(options=options), self.assertRaises((TypeError, ValueError)):
                axes.get_riemann_rectangles(graph, **options)
        self.assertEqual(called, [])
        self.assertEqual(len(axes.get_riemann_rectangles(graph, (1, 1))), 0)
        self.assertEqual(called, [])

    def test_geometric_graph_and_discontinuity_fail_without_editing_source(self):
        axes = self.axes()
        graph = m.VMobject().set_points_as_corners([axes.c2p(0, 1), axes.c2p(.4, 1)])
        right = m.VMobject().set_points_as_corners([axes.c2p(.6, -1), axes.c2p(1, -1)])
        graph.add_subpath(right.get_points())
        before = graph.get_points().copy()
        rects = axes.get_riemann_rectangles(graph, (0, .4), dx=.2)
        self.assertEqual(len(rects), 2)
        with self.assertRaisesRegex(ValueError, 'discontinuity'):
            axes.get_riemann_rectangles(graph, (0, 1), dx=1, input_sample_type='center')
        np.testing.assert_array_equal(graph.get_points(), before)

    def test_native_render_matches_explicit_chart_polygons_at_each_thread_count(self):
        def scene_class(use_helper):
            class AreaScene(m.Scene):
                def construct(self):
                    axes = m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=8, height=4)
                    axes.rotate(math.pi / 2)
                    if use_helper:
                        graph = axes.get_graph(lambda x: 1)
                        objects = axes.get_riemann_rectangles(graph, (0, 1), dx=.4,
                                                              colors=(m.WHITE,), stroke_width=0)
                    else:
                        objects = m.VGroup(*(m.Polygon(*[axes.c2p(x, y) for x, y in
                            ((a, 0), (b, 0), (b, 1), (a, 1))], fill_color=m.WHITE,
                            fill_opacity=1, stroke_width=0) for a, b in ((0, .4), (.4, .8), (.8, 1))))
                    self.add(objects)
                    self.wait(.25)
            return AreaScene
        with tempfile.TemporaryDirectory(prefix='fmn-riemann-') as directory:
            outputs = []
            for helper, threads in ((False, 1), (True, 1), (True, 4)):
                path = Path(directory) / f'area-{helper}-{threads}.y4m'
                render_scene(scene_class(helper), path, format='y4m', resolution=(160, 90), fps=8, threads=threads)
                data = path.read_bytes()
                width, height, frames = y4m_frames(data)
                self.assertEqual(len(frames), 2)
                luma = np.frombuffer(frames[0][:width * height], dtype=np.uint8)
                self.assertGreater(np.count_nonzero(luma > 100), 200)
                outputs.append(data)
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual(outputs[1], outputs[2])


suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromTestCase(cls)
                           for cls in (RecordComparison, LiveGraphingAcceptance, RiemannAcceptance))
if suite.countTestCases() != 20:
    raise AssertionError('live graph native acceptance inventory drift')
result = unittest.TextTestRunner(verbosity=2).run(suite)
if not result.wasSuccessful():
    raise AssertionError('live graph native acceptance failed')
