"""Copied native strings regenerate one glyph family with live source spans."""
from pathlib import Path
import copy
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m


class DecoratedText(m.Text):
    data_dtype = m.VMobject.data_dtype + [('weight', 1)]

    def init_data(self):
        super().init_data()
        self.decoration = m.Dot(2 * m.UP)
        self.add(self.decoration)


class DecoratedTex(m.Tex):
    def init_data(self):
        super().init_data()
        self.decoration = m.Dot(2 * m.UP)
        self.add(self.decoration)


def fingerprint(mob):
    return [(type(member).__name__, member.data.dtype.descr, member.data.tobytes())
            for member in mob.get_family()]


class CopiedStringRegenerationTests(unittest.TestCase):
    def test_shallow_copy_replaces_its_own_glyphs_across_text_families(self):
        for factory, text in ((m.Text, 'ab'), (m.MarkupText, '<b>ab</b>'),
                              (m.Code, 'a=1'), (m.Tex, 'x+y'), (m.TexText, 'abc')):
            with self.subTest(factory=factory.__name__):
                source = factory(text)
                before = fingerprint(source)
                count = len(source)
                clone = source.copy()
                for _ in range(3):
                    clone.init_points()
                    self.assertEqual(len(clone), count)
                    self.assertTrue(all(child in clone.submobjects for child in clone._fmn_string_children))
                    self.assertTrue(all(child not in source.submobjects for child in clone._fmn_string_children))
                self.assertEqual(fingerprint(source), before)
                # Constructors can perform a final centering step after points.
                # Compare to an independently regenerated source, not a copied one.
                expected = factory(text)
                expected.init_points()
                self.assertEqual(fingerprint(clone), fingerprint(expected))

    def test_shallow_deep_and_pickle_keep_decorations_without_double_glyphs(self):
        source = DecoratedText('abc')
        for clone in (source.copy(), copy.deepcopy(source), pickle.loads(pickle.dumps(source))):
            self.assertIsNot(clone.decoration, source.decoration)
            for _ in range(2):
                clone.init_points()
                self.assertEqual(len(clone), 4)
                self.assertIs(clone.submobjects[0], clone.decoration)
                self.assertIn('weight', clone.data.dtype.names)
            self.assertEqual(len(source), 4)

    def test_unicode_selectors_point_only_at_new_copied_glyphs(self):
        source = DecoratedText('é+λ')
        clone = source.copy()
        old = tuple(clone._fmn_string_children)
        clone.init_points()
        self.assertEqual(len(clone), 4)
        for text in ('é', '+', 'λ'):
            selected = clone.select_part(text)
            members = selected.family_members_with_points()
            self.assertTrue(members)
            self.assertTrue(all(member in clone.get_family() for member in members))
            self.assertTrue(all(member not in old for member in members))
            self.assertTrue(all(member not in source.get_family() for member in members))
        self.assertEqual(clone._string_sub_spans, source._string_sub_spans)

    def test_nested_tex_span_paths_remain_rebased_around_decorations(self):
        source = DecoratedTex('x', '+', 'y')
        clone = source.copy()
        paths = [list(path) for path in source._string_sub_paths]
        for _ in range(3):
            clone.init_points()
            self.assertEqual(len(clone), len(source))
            self.assertEqual(clone._string_sub_paths, paths)
            selected = clone.get_part_by_tex('x').family_members_with_points()
            self.assertTrue(selected)
            self.assertTrue(all(member in clone.get_family() for member in selected))
            self.assertNotIn(clone.decoration, selected)

    def test_copy_of_copy_and_shared_parent_identity(self):
        source = m.Text('ab')
        group = m.Group(m.Group(source), m.Group(source))
        clone = group.copy()
        left, right = clone[0][0], clone[1][0]
        self.assertIs(left, right)
        left.init_points()
        self.assertEqual(len(right), 2)
        next_copy = left.copy()
        next_copy.init_points()
        self.assertEqual(len(next_copy), 2)
        self.assertEqual(len(source), 2)

    def test_bound_source_copies_regenerate_independently(self):
        source = m.Text('ab')
        scene = m.Scene()
        scene.add(source)
        before = fingerprint(source)
        clone = source.copy()
        clone.init_points()
        self.assertEqual(len(clone), 2)
        self.assertEqual(fingerprint(source), before)
        self.assertEqual(scene.mobjects, [source])

    def test_typesetter_failure_does_not_change_copy_or_original(self):
        source = DecoratedText('ab')
        clone = source.copy()
        before = fingerprint(clone), tuple(clone.submobjects), tuple(clone._fmn_string_children)
        clone.font = 'not-a-native-font'
        with self.assertRaises((ValueError, RuntimeError)):
            clone.init_points()
        self.assertEqual(fingerprint(clone), before[0])
        self.assertEqual(tuple(clone.submobjects), before[1])
        self.assertEqual(tuple(clone._fmn_string_children), before[2])
        self.assertEqual(len(source), 3)

    def test_new_source_text_shrinks_without_stale_ink(self):
        source = m.Text('abcdef')
        clone = source.copy()
        clone.text = 'λ'
        clone.init_points()
        self.assertEqual(len(clone), 1)
        self.assertEqual(clone.get_string(), 'λ')
        self.assertEqual(len(clone.select_part('λ')), 1)
        self.assertEqual(source.get_string(), 'abcdef')
        self.assertEqual(len(source), 6)

    def test_animate_regeneration_does_not_inflate_the_target_family(self):
        source = m.Text('ab')
        scene = m.Scene()
        scene.add(source)
        animation = m.prepare_animation(source.animate.init_points().shift(m.RIGHT))
        self.assertEqual(len(animation.target_mobject), 2)
        scene.play(animation, run_time=2 / 30, rate_func=m.linear)
        self.assertEqual(len(source), 2)

    def test_translucent_copied_regeneration_matches_independent_native_pixels(self):
        root = Path(tempfile.mkdtemp(prefix='fmn-copy-string-'))
        for cls, text in ((m.Text, 'ab'), (m.Tex, 'x+y'), (m.Code, 'x=1')):
            def render(path, copied, threads):
                original = cls(text, fill_opacity=.4)
                obj = original.copy() if copied else original
                obj.init_points()
                scene = m.Scene()
                with scene.render_session(path, format='png_sequence', resolution=(128, 72), fps=4, threads=threads):
                    scene.add(obj)
                    scene.play(obj.animate.shift(m.RIGHT), run_time=.5, rate_func=m.linear)
                return [file.read_bytes() for file in sorted(path.glob('*.png'))]
            with self.subTest(cls=cls.__name__):
                first = render(root / (cls.__name__ + '-one'), True, 1)
                self.assertEqual(first, render(root / (cls.__name__ + '-four'), True, 4))
                self.assertEqual(first, render(root / (cls.__name__ + '-direct'), False, 1))
                self.assertEqual(len(first), 2)
                self.assertNotEqual(first[0], first[1])


if __name__ in ('__main__', '<run_path>'):
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(CopiedStringRegenerationTests))
    if not result.wasSuccessful():
        raise AssertionError('native copied-string regeneration regressions failed')
