"""Native SVG subclass construction, factory identity and rendered witnesses."""
from __future__ import annotations

import copy
import importlib
import inspect
from pathlib import Path
import pickle
import tempfile
import unittest
from unittest.mock import patch

import numpy as np
import manimlib as m

SOURCE = '<svg><path d="M0 0H2V1H0Z" fill="red"/></svg>'
DASHED = '<svg><path d="M0 0H2V2H0Z" fill="blue" stroke="red" stroke-width="6" stroke-dasharray=".5 .5"/></svg>'


def pixels(obj):
    return m.Camera(resolution=(96, 54)).capture_snapshot(obj).pixels()


class SvgLifecycleTests(unittest.TestCase):
    def test_hook_order_and_source_recipe(self):
        events = []
        class Custom(m.SVGMobject):
            def init_data(self):
                events.append(('data', self.svg_string))
                super().init_data()
            def init_points(self):
                events.append(('points', self.get_num_points()))
                super().init_points()
            def init_uniforms(self):
                events.append(('uniforms',))
                super().init_uniforms()
            def init_colors(self):
                events.append(('colors', len(self)))
                super().init_colors()
            def init_svg_mobject(self):
                events.append(('svg',))
                super().init_svg_mobject()
            def mobjects_from_svg_string(self, source):
                events.append(('parts', source))
                return super().mobjects_from_svg_string(source)
        obj = Custom(svg_string=SOURCE)
        self.assertEqual(events, [('data', SOURCE), ('points', 0), ('uniforms',),
                                  ('colors', 0), ('svg',), ('parts', SOURCE)])
        self.assertEqual(len(obj), 1)

    def test_custom_columns_root_geometry_and_decorations_survive(self):
        class Custom(m.SVGMobject):
            data_dtype = m.VMobject.data_dtype + [('weight', 1)]
            def init_data(self):
                super().init_data()
                self.decoration = m.Dot(m.RIGHT)
                self.add(self.decoration)
            def init_points(self):
                self.set_points_as_corners([[-1, 0, 0], [0, 1, 0], [1, 0, 0]])
                self.data['weight'][:] = 7
                self.weights = self.data['weight']
            def init_uniforms(self):
                super().init_uniforms()
                self.uniforms['fixed_in_frame'] = .75
        obj = Custom(svg_string=SOURCE, height=2)
        self.assertEqual(obj.get_num_points(), 5)
        self.assertIs(obj[0], obj.decoration)
        np.testing.assert_array_equal(obj.data['weight'], 7)
        self.assertEqual(obj.uniforms['fixed_in_frame'], .75)
        obj.weights[:] = 9
        np.testing.assert_array_equal(obj.data['weight'], 9)
        scene = m.Scene(); scene.add(obj)
        scene.play(obj.animate.shift(m.RIGHT), run_time=1/30, rate_func=m.linear)
        np.testing.assert_array_equal(obj.data['weight'], 9)

    def test_returned_parts_are_actual_children_not_reconstructed(self):
        made, calls = [], []
        class Custom(m.SVGMobject):
            def mobjects_from_svg_string(self, source):
                calls.append(source)
                for point in (m.LEFT, m.RIGHT):
                    obj = m.Square(side_length=.5).shift(point).set_fill(m.GREEN, 1)
                    made.append(obj)
                    yield obj
        obj = Custom(svg_string='authored source', height=1)
        self.assertEqual(calls, ['authored source'])
        self.assertEqual(list(obj.submobjects), made)
        self.assertTrue(all(child.get_fill_color() == m.GREEN for child in made))

    def test_init_svg_override_can_supply_the_complete_family(self):
        class Custom(m.SVGMobject):
            def init_svg_mobject(self):
                self.authored = m.Triangle().set_fill(m.YELLOW, 1)
                self.add(self.authored)
        obj = Custom(svg_string='no XML parsing requested by this subclass', height=2)
        self.assertEqual(list(obj), [obj.authored])
        self.assertEqual(obj.authored.get_fill_color(), m.YELLOW)

    def test_factory_edits_native_paint_layers_without_filling_dashes(self):
        class Custom(m.SVGMobject):
            def mobjects_from_svg_string(self, source):
                parts = super().mobjects_from_svg_string(source)
                for part in parts:
                    part.stretch(1.5, 0)
                return parts
        obj = Custom(svg_string=DASHED, height=2, stroke_width=None)
        expected = m.SVGMobject(svg_string=DASHED, height=2, stroke_width=None)
        expected.stretch(1.5, 0)
        self.assertEqual(pixels(obj), pixels(expected))
        obj.set_opacity(.4)
        self.assertTrue(all(layer.get_fill_opacity() == 0 for layer in obj[0].submobjects[1:]))

    def test_live_rebuild_invokes_public_factory_once_and_appends(self):
        obj = m.SVGMobject(svg_string=SOURCE)
        scene = m.Scene(); scene.add(obj)
        old = list(obj.submobjects)
        made = m.Circle(radius=.25)
        calls = []
        obj.mobjects_from_svg_string = lambda source: calls.append(source) or [made]
        self.assertIsNone(obj.init_svg_mobject())
        self.assertEqual(calls, [SOURCE])
        self.assertEqual(list(obj.submobjects), [*old, made])
        self.assertIs(made._scene, scene)
        self.assertEqual(scene.get_time(), 0)

    def test_generator_failure_does_not_install_half_a_family(self):
        obj = m.SVGMobject(svg_string=SOURCE)
        before = list(obj.submobjects); frame = pixels(obj)
        failure = RuntimeError('authored SVG factory')
        def factory(source):
            yield m.Square()
            raise failure
        obj.mobjects_from_svg_string = factory
        with self.assertRaises(RuntimeError) as caught:
            obj.init_svg_mobject()
        self.assertIs(caught.exception, failure)
        self.assertEqual(list(obj.submobjects), before)
        self.assertEqual(pixels(obj), frame)

    def test_foreign_reused_and_invalid_parts_are_refused_without_mutation(self):
        obj = m.SVGMobject(svg_string=SOURCE)
        foreign = m.Square(); scene = m.Scene(); scene.add(foreign)
        before = foreign.get_points().copy(); children = list(obj.submobjects)
        repeated = m.Square()
        bad = m.VMobject().set_points_as_corners([[0, 0, 0], [1, 0, 0]])
        bad.data['point'][0, 0] = float('nan')
        for result in ([foreign], [obj], [obj[0]], [repeated, repeated], [None], [bad]):
            with self.subTest(result=result):
                obj.mobjects_from_svg_string = lambda source: iter(result)
                with self.assertRaises((TypeError, ValueError)):
                    obj.init_svg_mobject()
                self.assertEqual(list(obj.submobjects), children)
                np.testing.assert_array_equal(foreign.get_points(), before)

    def test_shared_descendants_keep_identity(self):
        shared = m.Dot()
        left, right = m.VGroup(shared), m.VGroup(shared)
        class Custom(m.SVGMobject):
            def mobjects_from_svg_string(self, source):
                return [left, right]
        obj = Custom(svg_string=SOURCE)
        self.assertIs(obj[0][0], obj[1][0])
        clone = obj.copy()
        self.assertIs(clone[0][0], clone[1][0])
        self.assertIsNot(clone[0][0], shared)

    def test_later_getter_cannot_adopt_an_earlier_part_unnoticed(self):
        obj = m.SVGMobject(svg_string=SOURCE); previous = list(obj.submobjects)
        first, second, scene = m.Square(), m.Circle(), m.Scene()
        original = second.get_points
        def points():
            scene.add(first)
            return original()
        second.get_points = points
        obj.mobjects_from_svg_string = lambda source: [first, second]
        with self.assertRaisesRegex(ValueError, 'ownership changed'):
            obj.init_svg_mobject()
        self.assertEqual(list(obj.submobjects), previous)
        self.assertIs(first._scene, scene)

    def test_failed_hook_stops_later_phases_without_retry(self):
        events = []; failure = ValueError('authored points')
        class Custom(m.SVGMobject):
            def init_points(self):
                events.append('points')
                raise failure
            def init_uniforms(self):
                events.append('uniforms')
            def init_svg_mobject(self):
                events.append('svg')
        with self.assertRaises(ValueError) as caught:
            Custom(svg_string=SOURCE)
        self.assertIs(caught.exception, failure)
        self.assertEqual(events, ['points'])

    def test_reentrant_factory_unwinds_and_allows_next_rebuild(self):
        obj = m.SVGMobject(svg_string=SOURCE)
        previous = list(obj.submobjects)
        default = obj.mobjects_from_svg_string
        obj.mobjects_from_svg_string = lambda source: obj.init_svg_mobject()
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            obj.init_svg_mobject()
        self.assertEqual(list(obj.submobjects), previous)
        obj.mobjects_from_svg_string = default
        obj.init_svg_mobject()
        self.assertEqual(len(obj), len(previous) * 2)

    def test_bounds_reject_before_adoption(self):
        import fmn_python.svg_ingress as ingress
        obj = m.SVGMobject(svg_string=SOURCE); previous = list(obj.submobjects)
        obj.mobjects_from_svg_string = lambda source: (m.Square() for _ in range(4))
        with patch.object(ingress, '_MAX_PARTS', 3), self.assertRaisesRegex(ValueError, 'budget'):
            obj.init_svg_mobject()
        self.assertEqual(list(obj.submobjects), previous)

    def test_copy_and_pickle_keep_svg_family_independent(self):
        source = m.SVGMobject(svg_string=DASHED, stroke_width=None)
        before = pixels(source)
        for clone in (source.copy(), copy.deepcopy(source), pickle.loads(pickle.dumps(source))):
            clone.svg_string = SOURCE
            old = list(clone.submobjects)
            clone.init_svg_mobject()
            self.assertEqual(len(clone), len(old) + 1)
            self.assertIsNot(clone[0], source[0])
            self.assertEqual(pixels(source), before)

    def test_frames_match_literal_geometry_at_one_and_four_threads(self):
        class Custom(m.SVGMobject):
            def mobjects_from_svg_string(self, source):
                return [part.stretch(1.5, 0) for part in super().mobjects_from_svg_string(source)]
        def render(path, factory, threads):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96, 54), fps=8, threads=threads):
                obj = factory(); scene.add(obj)
                scene.play(obj.animate.shift(m.RIGHT), run_time=.5, rate_func=m.linear)
            return [file.read_bytes() for file in sorted(path.glob('*.png'))]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            authored = lambda: Custom(svg_string=SOURCE, height=1)
            literal = lambda: m.Rectangle(width=3, height=1).set_fill('#ff0000', 1).set_stroke(width=0)
            frames = render(root/'one', authored, 1)
            self.assertEqual(frames, render(root/'four', authored, 4))
            self.assertEqual(frames, render(root/'literal', literal, 1))
            self.assertEqual(len(frames), 4)
            self.assertGreater(len(set(frames)), 1)

    def test_aliases_signatures_and_missing_source(self):
        exported = importlib.import_module('manimlib.mobject.svg.svg_mobject')
        self.assertIs(exported.SVGMobject, m.SVGMobject)
        self.assertIn('svg_default', inspect.signature(m.SVGMobject).parameters)
        with self.assertRaisesRegex(Exception, 'Must specify either'):
            m.SVGMobject()


if __name__ == '__main__':
    unittest.main(verbosity=2)
