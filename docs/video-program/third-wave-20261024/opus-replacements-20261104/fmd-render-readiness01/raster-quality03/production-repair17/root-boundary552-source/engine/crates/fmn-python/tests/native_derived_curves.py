"""Derived paths through actual native geometry, records, clocks and output."""
import copy
import gc
import importlib
from pathlib import Path
import tempfile
import unittest
import weakref
from unittest.mock import patch

import numpy as np
import manimlib as m
from fmn_python import render_session


def literal(source, dashed=False, count=4):
    if dashed:
        return m.VGroup(*(source.get_subcurve(a, b) for a, b in
                          source._dash_curve_intervals(count, .5)))
    obj = m._native_shell_factory()
    specs = obj._build_curves_as_submobjects(m._native_shell_factory, source)
    m._hang_native_children(obj, specs)
    for part in obj:
        part.match_style(source)
    return obj


class DerivedCurvesTests(unittest.TestCase):
    def test_stock_native_segments_and_styles_are_unchanged(self):
        for source in (m.Circle(stroke_width=3, color=m.BLUE), m.VMobject(),
                       m.VMobject().set_points_as_corners([[0, 0, 0], [1, 0, 0], [1, 2, 0]])):
            obj = m.CurvesAsSubmobjects(source)
            control = literal(source)
            self.assertEqual(len(obj), len(control))
            for part, expected in zip(obj, control):
                np.testing.assert_array_equal(part.data, expected.data)
                self.assertEqual(part.parents, [obj])

    def test_curves_honor_all_receiver_hooks_and_custom_root_records(self):
        calls = []
        class Authored(m.CurvesAsSubmobjects):
            data_dtype = [*m.VMobject.data_dtype, ('weight', 1)]
            def init_data(self):
                calls.append('data'); super().init_data()
                self.marker = m.Circle(radius=.1); self.add(self.marker)
            def init_points(self):
                calls.append('points'); super().init_points()
                self.set_points_as_corners([[0, 0, 0], [0, 1, 0]])
                self.data['weight'][:] = 17.
            def init_uniforms(self):
                calls.append('uniforms'); super().init_uniforms()
            def init_colors(self):
                calls.append('colors'); super().init_colors(); self.set_color(m.RED)
        source = m.Line(m.LEFT, m.RIGHT, color=m.BLUE)
        obj = Authored(source)
        self.assertEqual(calls, ['data', 'points', 'uniforms', 'colors'])
        self.assertEqual(len(obj), 2)
        self.assertIs(obj[0], obj.marker)
        self.assertEqual(obj.get_stroke_color(), m.RED)
        self.assertEqual(obj[-1].get_stroke_color(), m.BLUE)
        np.testing.assert_array_equal(obj.data['weight'], 17.)

    def test_authored_curve_tuples_are_used_and_yields_are_snapshotted(self):
        visits = []
        class Authored(m.VMobject):
            def get_bezier_tuples(self):
                points = np.array([[-1., 0, 0], [0, 1, 0], [1, 0, 0]])
                for offset in (0., 2., 5.):
                    points[:, 2] = offset
                    visits.append(offset)
                    yield points
        obj = m.CurvesAsSubmobjects(Authored())
        self.assertEqual(visits, [0., 2., 5.])
        self.assertEqual(len(obj), 3)
        for part, offset in zip(obj, visits):
            np.testing.assert_array_equal(part.get_points()[:, 2], offset)

    def test_public_source_point_getter_is_not_bypassed(self):
        calls = []
        source = m.Square()
        original = source.get_points
        def points():
            calls.append(1)
            return original() + m.UP
        source.get_points = points
        obj = m.CurvesAsSubmobjects(source)
        self.assertEqual(len(calls), 1)
        np.testing.assert_array_equal(obj[0].get_points(), original()[:3] + m.UP)

    def test_qualified_curve_factory_retains_its_subclass_fields_and_identity(self):
        made = []
        class Part(m.VMobject):
            data_dtype = [*m.VMobject.data_dtype, ('weight', 1)]
            def set_points(self, points):
                super().set_points(points); self.data['weight'][:] = 23.; return self
        def factory():
            obj = Part(); made.append(obj); return obj
        module = importlib.import_module(m.CurvesAsSubmobjects.__module__)
        with patch.object(module, 'VMobject', factory):
            obj = m.CurvesAsSubmobjects(m.Square())
        self.assertEqual(len(made), 4)
        self.assertTrue(all(a is b for a, b in zip(obj, made)))
        np.testing.assert_array_equal(obj[0].data['weight'], 23.)

    def test_invalid_factory_must_not_mutate_a_scene_owned_object(self):
        victim = m.Line(m.UP, m.DOWN)
        scene = m.Scene(); scene.add(victim)
        before = victim.data.copy()
        module = importlib.import_module(m.CurvesAsSubmobjects.__module__)
        with patch.object(module, 'VMobject', lambda: victim):
            with self.assertRaisesRegex(ValueError, 'detached'):
                m.CurvesAsSubmobjects(m.Square())
        np.testing.assert_array_equal(victim.data, before)

    def test_factory_cannot_restyle_source_descendants(self):
        child = m.Line(m.LEFT, m.RIGHT, color=m.BLUE)
        source = m.VGroup(child)
        module = importlib.import_module(m.CurvesAsSubmobjects.__module__)
        before = child.data.copy()
        with patch.object(module, 'VMobject', lambda: m.VGroup(child)):
            source.get_bezier_tuples = lambda: iter([np.zeros((3, 3))])
            with self.assertRaisesRegex(ValueError, 'independent'):
                m.CurvesAsSubmobjects(source)
        np.testing.assert_array_equal(child.data, before)

    def test_live_curve_rebuild_replaces_only_generated_children(self):
        source = m.Square(color=m.BLUE)
        obj = m.CurvesAsSubmobjects(source)
        before, after = m.Dot(m.LEFT), m.Dot(m.RIGHT)
        obj.add_to_back(before); obj.add(after)
        previous = tuple(obj._derived_parts)
        scene = m.Scene(); scene.add(obj)
        source.shift(m.UP)
        self.assertIs(obj.init_points(), obj)
        self.assertIs(obj[0], before); self.assertIs(obj[-1], after)
        self.assertEqual(len(obj), 6)
        self.assertTrue(all(part not in obj.submobjects for part in previous))
        self.assertIs(obj._scene, scene)
        for part, control in zip(obj._derived_parts, literal(source)):
            np.testing.assert_array_equal(part.get_points(), control.get_points())
        obj.init_points(); self.assertEqual(len(obj), 6)

    def test_live_dashes_follow_geometry_count_and_keep_source_record_fields(self):
        class Source(m.VMobject):
            data_dtype = [*m.VMobject.data_dtype, ('weight', 1)]
        source = Source(stroke_color=m.BLUE).set_points_as_corners([[0, 0, 0], [.1, 0, 0], [6, 0, 0]])
        source.data['weight'][:] = 13.
        obj = m.DashedVMobject(source, num_dashes=4)
        lengths = [part.get_arc_length() for part in obj]
        np.testing.assert_allclose(lengths, lengths[0], atol=1e-6, rtol=0)
        source.shift(m.UP); obj.num_dashes = 6; obj.init_points()
        self.assertEqual(len(obj), 6)
        for part, expected in zip(obj, literal(source, True, 6)):
            np.testing.assert_array_equal(part.get_points(), expected.get_points())
            np.testing.assert_array_equal(part.data['weight'], 13.)
        obj.num_dashes = 0; obj.init_points(); self.assertEqual(len(obj), 0)
        obj.num_dashes = 3; obj.init_points(); self.assertEqual(len(obj), 3)

    def test_dash_slicer_results_keep_authored_identities(self):
        parts, visits = [], []
        class Source(m.Line):
            def get_subcurve(self, a, b):
                visits.append((a, b))
                obj = super().get_subcurve(a, b)
                obj.mark = len(visits)
                parts.append(obj)
                return obj
        obj = m.DashedVMobject(Source(m.LEFT, m.RIGHT), num_dashes=4)
        self.assertEqual(len(visits), 4)
        self.assertTrue(all(a is b for a, b in zip(obj, parts)))
        self.assertEqual([part.mark for part in obj], [1, 2, 3, 4])

    def test_failed_dash_rebuild_does_not_publish_a_partial_batch(self):
        source = m.Line(m.LEFT, m.RIGHT)
        obj = m.DashedVMobject(source, num_dashes=4)
        before = tuple(obj.submobjects)
        original = source.get_subcurve
        error = RuntimeError('second slice failed'); visits = []
        def part(a, b):
            visits.append(a)
            if len(visits) == 2:
                raise error
            return original(a, b)
        source.get_subcurve = part
        with self.assertRaises(RuntimeError) as caught:
            obj.init_points()
        self.assertIs(caught.exception, error)
        self.assertEqual(tuple(obj.submobjects), before)
        self.assertEqual(len(visits), 2)
        source.get_subcurve = original; obj.init_points()

    def test_failed_curve_generator_and_invalid_triples_leave_previous_children(self):
        source = m.Square()
        obj = m.CurvesAsSubmobjects(source)
        before = tuple(obj.submobjects)
        def values():
            yield np.zeros((3, 3))
            yield np.zeros((2, 3))
        source.get_bezier_tuples = values
        with self.assertRaisesRegex(ValueError, 'three finite'):
            obj.init_points()
        self.assertEqual(tuple(obj.submobjects), before)

    def test_locked_and_reentrant_regeneration_refuse(self):
        source = m.Line(m.LEFT, m.RIGHT)
        obj = m.DashedVMobject(source, num_dashes=2)
        before = tuple(obj.submobjects)
        obj.locked_data_keys.add('point')
        with self.assertRaisesRegex(RuntimeError, 'active animation'):
            obj.init_points()
        obj.locked_data_keys.clear()
        source.get_subcurve = lambda a, b: obj.init_points()
        with self.assertRaisesRegex(RuntimeError, 'already in progress'):
            obj.init_points()
        self.assertEqual(tuple(obj.submobjects), before)

    def test_callback_cannot_overwrite_receiver_edits(self):
        source = m.Line(m.LEFT, m.RIGHT)
        obj = m.DashedVMobject(source, num_dashes=2)
        marker = m.Dot()
        original = source.get_subcurve
        def slice(a, b):
            obj.add(marker)
            return original(a, b)
        source.get_subcurve = slice
        old = tuple(obj._derived_parts)
        with self.assertRaisesRegex(RuntimeError, 'changed during sampling'):
            obj.init_points()
        self.assertIs(obj[-1], marker)
        self.assertEqual(tuple(obj._derived_parts), old)

    def test_copy_deepcopy_and_collection_remap_generated_references(self):
        for cls in (m.CurvesAsSubmobjects, m.DashedVMobject):
            with self.subTest(cls=cls):
                obj = cls(m.Square()); marker = m.Dot(); obj.add(marker)
                for duplicate in (obj.copy(), copy.deepcopy(obj)):
                    self.assertTrue(all(a is b for a, b in zip(duplicate._derived_parts, duplicate.submobjects)))
                    duplicate.init_points()
                    self.assertEqual(len(duplicate), len(obj))
                    self.assertIsNot(duplicate[0], obj[0])
                refs = [weakref.ref(obj), weakref.ref(obj[0])]
                del obj, marker
                gc.collect()
                self.assertTrue(all(ref() is None for ref in refs))

    def test_count_and_ratio_refusals_keep_existing_children(self):
        obj = m.DashedVMobject(m.Square(), num_dashes=4)
        before = tuple(obj.submobjects)
        for count, ratio in ((4097, .5), (3, 0), (2, float('nan'))):
            obj.num_dashes, obj.positive_space_ratio = count, ratio
            with self.assertRaises(ValueError):
                obj.init_points()
            self.assertEqual(tuple(obj.submobjects), before)

    def test_authored_root_hooks_remain_before_dash_sampling_and_style_precedence(self):
        visits = []
        class Source(m.Line):
            def get_subcurve(self, a, b):
                visits.append('slice'); return super().get_subcurve(a, b)
        class Authored(m.DashedVMobject):
            def init_points(self):
                visits.append('points'); super().init_points()
            def init_colors(self):
                visits.append('colors'); super().init_colors()
        obj = Authored(Source(m.LEFT, m.RIGHT, color=m.BLUE), num_dashes=2,
                       stroke_color=m.RED, stroke_behind=True, flat_stroke=True)
        self.assertEqual(visits, ['points', 'colors', 'slice', 'slice'])
        self.assertEqual(obj.get_stroke_color(), m.BLUE)
        self.assertTrue(obj.stroke_behind); self.assertTrue(obj.flat_stroke)

    def test_animation_builder_regeneration_keeps_decoration_identity(self):
        for cls in (m.CurvesAsSubmobjects, m.DashedVMobject):
            source = m.Square()
            obj = cls(source); marker = m.Circle(radius=.1); obj.add(marker)
            scene = m.Scene(); scene.add(obj)
            source.shift(m.UP)
            scene.play(obj.animate.init_points(), run_time=.125, rate_func=m.linear)
            self.assertIs(obj[-1], marker)
            expected = literal(source, cls is m.DashedVMobject, 15)
            for actual, control in zip(obj[:-1], expected):
                np.testing.assert_array_equal(actual.get_points(), control.get_points())

    def test_empty_and_nonpositive_patterns_keep_hook_decorations(self):
        class Authored(m.DashedVMobject):
            def init_data(self):
                super().init_data(); self.marker = m.Circle(); self.add(self.marker)
        for count in (0, -3):
            obj = Authored(m.Square(), num_dashes=count, positive_space_ratio=object())
            self.assertEqual(len(obj), 1); self.assertIs(obj[0], obj.marker)
            obj.init_points(); self.assertEqual(len(obj), 1)
        self.assertEqual(len(m.CurvesAsSubmobjects(m.VMobject())), 0)

    def test_bounded_authored_generator_fails_without_partial_publication(self):
        adapter = importlib.import_module('fmn_python.derived_curves')
        source = m.Line(m.LEFT, m.RIGHT); obj = m.CurvesAsSubmobjects(source)
        previous = tuple(obj.submobjects); visits = []
        def tuples():
            for index in range(4):
                visits.append(index)
                yield np.zeros((3, 3))
        source.get_bezier_tuples = tuples
        # Exercise the exact production bound at a small deterministic seam,
        # rather than creating thousands of proxies in an allocation test.
        with patch.object(adapter, '_MAX_PARTS', 2):
            with self.assertRaisesRegex(ValueError, 'part budget'):
                obj.init_points()
        self.assertEqual(visits, [0, 1, 2])
        self.assertEqual(tuple(obj.submobjects), previous)
        obj.init_points(); self.assertEqual(len(obj), 4)

    def test_bound_and_reentrant_constructors_do_not_replace_live_objects(self):
        source = m.Square()
        for cls in (m.CurvesAsSubmobjects, m.DashedVMobject):
            obj = cls(source); scene = m.Scene(); scene.add(obj)
            before = tuple(obj.submobjects)
            with self.assertRaisesRegex(RuntimeError, 'detached target'):
                obj.__init__(source)
            self.assertEqual(tuple(obj.submobjects), before)
            class Reentrant(cls):
                def init_points(self):
                    self.__init__(source)
            with self.assertRaisesRegex(RuntimeError, 'already in progress'):
                Reentrant(source)

    def test_alignment_copies_remain_generated_after_become_and_restore(self):
        for count in (0, 3):
            obj = m.DashedVMobject(m.Square(), num_dashes=count)
            target = m.DashedVMobject(m.Square().shift(m.UP), num_dashes=5)
            obj.become(target)
            # become changes geometry, not the receiver's construction recipe.
            self.assertEqual(obj.num_dashes, count)
            obj.init_points()
            self.assertEqual(len(obj), count)
            obj.save_state()
            obj.add_n_more_submobjects(4)
            obj.restore(); obj.init_points()
            self.assertEqual(len(obj), count)

    def test_rebuilt_frames_match_independent_native_paths(self):
        def render(path, kind, native, workers):
            source = m.Square(stroke_color=m.BLUE, fill_opacity=0)
            cls = m.DashedVMobject if kind else m.CurvesAsSubmobjects
            obj = literal(source, bool(kind)) if native else (cls(source, num_dashes=4) if kind else cls(source))
            scene = m.Scene()
            with render_session(scene, path, resolution=(80, 48), fps=8, threads=workers) as session:
                scene.add(obj); scene.wait(.125)
                source.shift(.75 * m.RIGHT)
                if native:
                    parts = list(literal(source, bool(kind)))
                    obj.set_submobjects(parts)
                else:
                    obj.init_points()
                scene.wait(.125)
                source.set_color(m.RED)
                if native:
                    parts = list(literal(source, bool(kind)))
                    obj.set_submobjects(parts)
                else:
                    obj.init_points()
                scene.wait(.125)
            self.assertEqual(session.result.frame_count, 3)
            return path.read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for kind in (0, 1):
                expected = render(root/f'control-{kind}.y4m', kind, True, 1)
                for workers in (1, 4, 16):
                    self.assertEqual(render(root/f'actual-{kind}-{workers}.y4m', kind, False, workers), expected)
                # Geometry and color edits really change successive frame payloads.
                frames = expected.split(b'FRAME\n')[1:]
                self.assertEqual(len(frames), 3)
                self.assertNotEqual(frames[0], frames[1]); self.assertNotEqual(frames[1], frames[2])


def run_native_derived_curves():
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(DerivedCurvesTests))
    if not result.wasSuccessful():
        raise AssertionError('native derived-curve acceptance failed')


if __name__ == '__main__':
    run_native_derived_curves()
