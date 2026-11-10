"""Actual native vector-group lineage, ingestion and scene-code regressions.

Reference: 3b1b/manim@6199a00d, types/vectorized_mobject.py VGroup and
mobject.py Group. Literal geometry controls do not construct the tested groups.
"""
from __future__ import annotations

import copy
import importlib
import json
import pickle
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import numpy as np
import manimlib as m


class VectorGroupProtocol(unittest.TestCase):
    def test_public_lineage_and_existing_library_subclasses(self):
        self.assertEqual(m.VGroup.__bases__, (m.Group, m.VMobject))
        self.assertEqual(m.VGroup.__mro__[:4], (m.VGroup, m.Group, m.VMobject, m.Mobject))
        for mob in (m.VGroup(), m.VGroup3D(), m.Axes(), m.NumberPlane(),
                    m.VCube(), m.VGroup3D(m.Square())):
            with self.subTest(cls=type(mob).__name__):
                self.assertIsInstance(mob, m.Group)
                self.assertIsInstance(mob, m.VMobject)
        self.assertNotIsInstance(m.Square(), m.Group)
        self.assertIs(importlib.import_module('manimlib.mobject.types.vectorized_mobject').VGroup,
                      m.VGroup)

    def test_group_ingestion_is_virtual_and_uses_public_add(self):
        calls = []
        class Authored(m.VGroup):
            def _ingest_args(self, *args):
                calls.append(('ingest', len(args)))
                return super()._ingest_args(*args)
            def add(self, *args):
                calls.append(('add', len(args)))
                return super().add(*args)
        square = m.Square()
        group = Authored(square, square)
        self.assertEqual(calls, [('ingest', 0), ('ingest', 2), ('add', 2)])
        self.assertEqual(list(group), [square])

    def test_late_group_patch_reaches_vector_groups(self):
        calls = []
        original = m.Group._ingest_args
        def recording(self, *objects):
            calls.append((self, objects))
            return original(self, *objects)
        square = m.Square()
        with patch.object(m.Group, '_ingest_args', recording):
            group = m.VGroup(square)
        self.assertEqual(calls, [(group, ()), (group, (square,))])

    def test_generator_is_consumed_once_before_native_adoption(self):
        square, circle, visits = m.Square(), m.Circle(), []
        def children():
            for child in (square, circle, square):
                visits.append(child)
                yield child
        group = m.VGroup(children())
        self.assertEqual(visits, [square, circle, square])
        self.assertEqual(list(group), [square, circle])

    def test_generator_failure_does_not_adopt_a_prefix(self):
        captured, failure = [], RuntimeError('authored generator')
        class Capture(m.VGroup):
            def init_data(self):
                captured.append(self)
                super().init_data()
        square = m.Square()
        def broken():
            yield square
            raise failure
        with self.assertRaises(RuntimeError) as caught:
            Capture(broken())
        self.assertIs(caught.exception, failure)
        self.assertEqual(list(captured[0]), [])
        self.assertNotIn(captured[0], square.parents)

    def test_native_initialization_hooks_run_once_with_custom_records(self):
        class Authored(m.VGroup):
            data_dtype = m.VMobject.data_dtype + [('mass', 1)]
            def init_data(self):
                self.events = ['data']
                super().init_data()
            def init_points(self):
                self.events.append('points')
                self.set_points([[0, 0, 0], [.5, 1, 0], [1, 0, 0]])
                self.data['mass'][:] = 7
                self.decoration = m.Dot(m.UP)
                self.add(self.decoration)
            def init_uniforms(self):
                self.events.append('uniforms')
                super().init_uniforms()
            def init_colors(self):
                self.events.append('colors')
                return super().init_colors()
        square = m.Square()
        group = Authored(square)
        self.assertEqual(group.events, ['data', 'points', 'uniforms', 'colors'])
        np.testing.assert_array_equal(group.data['mass'], 7)
        self.assertEqual(list(group), [group.decoration, square])

    def test_first_child_uniforms_follow_reference_post_ingestion_precedence(self):
        first, second = m.Square(), m.Circle()
        first.set_shading(.7, .4, .2)
        first.fix_in_frame()
        first.uniforms['authored'] = 17
        second.set_shading(.1, .2, .3)
        before_second = dict(second.uniforms)
        group = m.VGroup(first, second, shading=(.9, .8, .7))
        for key, value in first.uniforms.items():
            np.testing.assert_array_equal(group.uniforms[key], value, err_msg=key)
        group.uniforms['authored'] = 23
        self.assertEqual(first.uniforms['authored'], 17)
        self.assertIsNot(group.uniforms, first.uniforms)
        for key, value in before_second.items():
            np.testing.assert_array_equal(second.uniforms[key], value)

    def test_hook_created_first_child_controls_uniforms(self):
        class Decorated(m.VGroup):
            def init_points(self):
                self.decoration = m.Square()
                self.decoration.uniforms['authored'] = 41
                self.add(self.decoration)
        other = m.Circle()
        other.uniforms['authored'] = 99
        group = Decorated(other)
        self.assertEqual(group.uniforms['authored'], 41)
        self.assertEqual(other.uniforms['authored'], 99)

    def test_empty_group_keeps_its_own_uniforms(self):
        class Empty(m.VGroup):
            def init_uniforms(self):
                super().init_uniforms()
                self.uniforms['authored'] = 71
        group = Empty(flat_stroke=True)
        self.assertEqual(group.uniforms['authored'], 71)
        self.assertTrue(group.get_flat_stroke())

    def test_addition_mutates_same_group_and_preserves_actual_members(self):
        first, second, third = m.Square(), m.Circle(), m.Triangle()
        group = m.VGroup(first)
        result = group + second + third + second
        self.assertIs(result, group)
        self.assertEqual(list(group), [first, second, third])
        alias = group
        group += m.VGroup()
        self.assertIs(group, alias)
        self.assertEqual(len(group), 4)

    def test_addition_calls_authored_add_and_preserves_exception(self):
        failure, calls = RuntimeError('authored add'), []
        class Authored(m.VGroup):
            def add(self, *objects):
                calls.extend(objects)
                if getattr(self, 'fail', False):
                    raise failure
                return super().add(*objects)
        group, square = Authored(), m.Square()
        group.fail = True
        with self.assertRaises(RuntimeError) as caught:
            group + square
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, [square])
        self.assertEqual(list(group), [])

    def test_invalid_addition_and_cycle_refuse_without_mutation(self):
        group = m.VGroup(m.Square())
        original = list(group)
        for other in (m.Point(), m.Group(), 1, [m.Circle()]):
            with self.subTest(other=type(other).__name__), self.assertRaises(AssertionError):
                group + other
            self.assertEqual(list(group), original)
        with self.assertRaises(Exception):
            group + group
        self.assertEqual(list(group), original)

    def test_copy_deepcopy_and_pickle_keep_group_lineage_and_independent_members(self):
        source = m.VGroup(m.Square(), m.Circle())
        for clone in (source.copy(), copy.copy(source), copy.deepcopy(source),
                      pickle.loads(pickle.dumps(source))):
            self.assertIsInstance(clone, m.Group)
            self.assertEqual([type(x) for x in clone], [m.Square, m.Circle])
            self.assertIsNot(source[0], clone[0])
            self.assertIs(clone + m.Triangle(), clone)
            self.assertEqual(len(source), 2)
            self.assertEqual(len(clone), 3)

    def test_scene_owned_addition_and_animation_keep_one_root(self):
        scene, square = m.Scene(), m.Square()
        group = m.VGroup(square)
        scene.add(group)
        circle = m.Circle().shift(m.RIGHT)
        self.assertIs(group + circle, group)
        self.assertEqual(list(scene.mobjects), [group])
        self.assertIs(group[1], circle)
        scene.play(group.animate.shift(m.UP), run_time=2/30, rate_func=m.linear)
        np.testing.assert_allclose(square.get_center(), m.UP, atol=1e-6)
        np.testing.assert_allclose(circle.get_center(), m.RIGHT+m.UP, atol=1e-6)

    def test_constructor_and_operator_share_foreign_scene_refusal(self):
        owner, foreign = m.Scene(), m.Scene()
        group, square = m.VGroup(m.Square()), m.Square()
        owner.add(group)
        foreign.add(square)
        before = list(group)
        with self.assertRaises(Exception):
            group + square
        self.assertEqual(list(group), before)
        self.assertEqual(list(foreign.mobjects), [square])

    def test_installation_is_idempotent(self):
        from fmn_python.grouping import install_grouping
        native = getattr(m, '_native', m)
        before = (m.VGroup, m.VGroup.__bases__, m.VGroup.__init__, m.VGroup.__add__)
        install_grouping(native)
        self.assertEqual(before, (m.VGroup, m.VGroup.__bases__, m.VGroup.__init__, m.VGroup.__add__))

    def test_group_dispatched_animation_matches_independent_literal_frames(self):
        def render(path, workers, grouped):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', fps=3,
                                      resolution=(96, 54), threads=workers):
                square = m.Square(side_length=.7, fill_opacity=1, color=m.RED).shift(m.LEFT)
                circle = m.Circle(radius=.35, fill_opacity=1, color=m.BLUE).shift(m.RIGHT)
                if grouped:
                    root = m.VGroup(square) + circle
                    scene.add(root)
                    # Real source patterns dispatch collections differently
                    # from leaves. Before the lineage fix this branch is lost.
                    targets = [mob for mob in scene.mobjects if isinstance(mob, m.Group)]
                else:
                    scene.add(square, circle)
                    targets = [square, circle]
                if targets:
                    scene.play(*(mob.animate.shift(m.UP) for mob in targets),
                               run_time=1, rate_func=m.linear)
                else:
                    scene.wait(1)
            return [p.read_bytes() for p in sorted(Path(path).glob('*.png'))]
        with tempfile.TemporaryDirectory() as directory:
            p = Path(directory)
            first = render(p/'group1', 1, True)
            self.assertEqual(len(first), 3)
            self.assertGreater(len(set(first)), 1)
            self.assertEqual(first, render(p/'group4', 4, True))
            self.assertEqual(first, render(p/'literal', 1, False))
        print(json.dumps({'bead':'fm-16u6','construct':'VGroup(Square()) + Circle()',
            'mro':[cls.__name__ for cls in m.VGroup.__mro__[:4]],
            'reference':'6199a00d:vectorized_mobject.py:VGroup',
            'frames':3,'workers':[1,4],'verdict':'pass'}, sort_keys=True))


if __name__ == '__main__':
    unittest.main()
