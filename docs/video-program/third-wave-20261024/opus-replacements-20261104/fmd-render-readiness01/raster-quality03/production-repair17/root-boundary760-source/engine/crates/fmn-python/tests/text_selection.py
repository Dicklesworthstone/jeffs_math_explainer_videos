"""Occurrence selection stays a live native glyph view across text front doors."""
import copy
import re
import unittest

import manimlib as m
import numpy as np


class TextSelectionTests(unittest.TestCase):
    def test_positional_and_keyword_occurrences_select_the_same_live_glyphs(self):
        for cls in (m.Text, m.MarkupText, m.Code):
            with self.subTest(cls=cls.__name__):
                text = cls('café café café')
                parts = text.get_parts_by_text('café')
                self.assertEqual(len(parts), 3)
                for index in (0, 1, 2, -1, np.int64(1)):
                    selected = text.get_part_by_text('café', index)
                    expected = parts[index]
                    self.assertEqual(tuple(map(id, selected)), tuple(map(id, expected)))
                    keyword = text.get_part_by_text('café', index=index)
                    self.assertEqual(tuple(map(id, selected)), tuple(map(id, keyword)))
                text.get_part_by_text('café', 1).set_color(m.RED)
                self.assertTrue(all(glyph.get_fill_color() == m.RED for glyph in parts[1]))
                self.assertTrue(all(glyph.get_fill_color() != m.RED for glyph in parts[0]))

    def test_regex_and_unicode_keep_native_source_mapping(self):
        text = m.MarkupText('<b>café</b> + <i>café</i>')
        literal = text.get_part_by_text('café', 1)
        pattern = text.get_part_by_text(re.compile(r'caf.'), 1)
        self.assertEqual(tuple(map(id, literal)), tuple(map(id, pattern)))
        self.assertGreater(len(literal), 0)
        self.assertEqual(tuple(map(id, literal)), tuple(map(id, text.select_part('café', 1))))

    def test_authored_selector_dispatch_preserves_argument_forms(self):
        calls = []
        class Authored(m.Text):
            def select_part(self, selector, *args, **kwargs):
                calls.append((self, selector, args, dict(kwargs)))
                kwargs.pop('custom', None)
                return super().select_part(selector, *args, **kwargs)
        text = Authored('A A')
        text.get_part_by_text('A')
        text.get_part_by_text('A', 1, custom=True)
        text.get_part_by_text('A', index=-1)
        self.assertEqual(calls, [(text, 'A', (), {}), (text, 'A', (1,), {'custom': True}),
                                 (text, 'A', (), {'index': -1})])

    def test_errors_match_the_underlying_selector_without_geometry_mutation(self):
        text = m.Text('A A')
        before = text.get_all_points().copy()
        for args, kwargs in (((3,), {}), ((1.2,), {}), ((1,), {'index': 0}),
                             ((0, 1), {}), ((), {'unknown': True})):
            with self.subTest(args=args, kwargs=kwargs):
                with self.assertRaises((TypeError, IndexError)) as expected:
                    text.select_part('A', *args, **kwargs)
                with self.assertRaises(type(expected.exception)):
                    text.get_part_by_text('A', *args, **kwargs)
                np.testing.assert_array_equal(text.get_all_points(), before)

    def test_copy_selection_changes_only_owned_glyphs_and_native_pixels(self):
        text = m.Code('let let', language='rust')
        duplicate = copy.deepcopy(text)
        text.get_part_by_text('let', -1).set_color(m.BLUE)
        camera = m.Camera()
        camera.reset_pixel_shape(160, 90)
        changed = camera.capture_snapshot(text).pixels()
        self.assertNotEqual(changed, camera.capture_snapshot(duplicate).pixels())
        duplicate.select_part('let', -1).set_color(m.BLUE)
        self.assertEqual(changed, camera.capture_snapshot(duplicate).pixels())


if __name__ == '__main__':
    unittest.main(verbosity=2)
