"""The actual native Markdown parser/layout/record boundary; no parser doubles."""
from __future__ import annotations

import io
import unittest
import numpy as np
import manimlib as m

SOURCE = '# Heading\n\nA **bold** word & `Vec<T>`.\n\n```rust\nfn main() {}\n```\n\n- outer\n  - inner\n'


def build(source=SOURCE, **kwargs):
    obj = m.VMobject()
    specs, ranges, kinds = m._build_markdown(obj, m._native_shell_factory, source, **kwargs)
    m._hang_native_children(obj, specs)
    return obj, tuple(map(tuple, ranges)), tuple(kinds)


class NativeMarkdownTests(unittest.TestCase):
    def test_structured_blocks_and_source_byte_ranges(self):
        obj, spans, kinds = build()
        self.assertEqual(kinds, ('heading', 'paragraph', 'code', 'list'))
        self.assertEqual(len(obj), len(spans))
        for (start, end), child in zip(spans, obj):
            self.assertTrue(SOURCE.encode()[start:end].decode())
            self.assertTrue(child.family_members_with_points())
        self.assertEqual(SOURCE.encode()[spans[0][0]:spans[0][1]].decode().strip(), '# Heading')

    def test_visible_blocks_are_left_aligned_and_do_not_overlap(self):
        obj, _, _ = build()
        self.assertAlmostEqual(obj[0].get_top()[1], 0, places=5)
        for child in obj:
            self.assertAlmostEqual(child.get_left()[0], 0, places=5)
        for first, second in zip(obj.submobjects, obj.submobjects[1:]):
            self.assertAlmostEqual(first.get_bottom()[1] - second.get_top()[1], .45, places=5)

    def test_unicode_source_ranges_and_empty_document(self):
        source = '# αβ\n\nΩ & λ\n'
        obj, spans, kinds = build(source)
        self.assertEqual(kinds, ('heading', 'paragraph'))
        self.assertIn('Ω', source.encode()[spans[1][0]:spans[1][1]].decode())
        self.assertEqual(len(build('')[0]), 0)

    def test_inline_code_angle_brackets_render_as_literal_monospace(self):
        obj, _, _ = build('`<b>x & y</b>`')
        reference = m.MarkupText('&lt;b&gt;x &amp; y&lt;/b&gt;', font='CM Typewriter', font_size=48)
        reference.shift(-reference.get_corner(m.UL))
        actual = m.Camera(resolution=(144, 64)).capture_snapshot(obj).png()
        expected = m.Camera(resolution=(144, 64)).capture_snapshot(reference).png()
        self.assertEqual(actual, expected)

    def test_fences_preserve_native_syntax_colors_and_tables_have_rules(self):
        code, _, _ = build('```rust\nfn main() { let x = 2; }\n```')
        colors = set()
        for member in code.family_members_with_points():
            colors.update(tuple(row) for row in member.data['fill_rgba'])
        self.assertGreater(len(colors), 1)
        table, _, kinds = build('| a | b |\n| --- | --- |\n| x | 2 |\n')
        self.assertEqual(kinds, ('table',))
        self.assertGreater(len(table[0]), 4)
        self.assertGreater(table[0][0].get_num_points(), 0)

    def test_refusals_leave_existing_receiver_records_unchanged(self):
        obj = m.Square()
        before = obj.data.copy()
        for source, options in [('x'*32769, {}), ('x\n\n'*257, {}), ('x', {'font_size': float('nan')}),
                                ('x', {'block_gap': -1}), ('x', {'theme': 'not-a-theme'})]:
            with self.assertRaises((ValueError, m._CapabilityError)):
                m._build_markdown(obj, m._native_shell_factory, source, **options)
            np.testing.assert_array_equal(obj.data, before)
        scene=m.Scene(); scene.add(obj)
        with self.assertRaisesRegex(ValueError, 'detached'):
            m._build_markdown(obj, m._native_shell_factory, 'ok')
        self.assertEqual(tuple(scene.mobjects), (obj,))
        np.testing.assert_array_equal(obj.data, before)


def run_native_markdown_acceptance():
    stream=io.StringIO()
    result=unittest.TextTestRunner(stream=stream,verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeMarkdownTests))
    print(stream.getvalue())
    if not result.wasSuccessful():
        raise AssertionError(stream.getvalue())


if __name__ == '__main__':
    run_native_markdown_acceptance()
