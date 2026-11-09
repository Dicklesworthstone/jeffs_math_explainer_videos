"""Native glyph, style, span and pixel witnesses for rich text construction."""
import gc
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from manimlib.mobject.svg.text_mobject import Text as QualifiedText


def points(obj):
    return np.array(obj.get_all_points(), copy=True)


def records(obj):
    return [member.data.copy() for member in obj.family_members_with_points()]


class TextAuthoringTests(unittest.TestCase):
    def test_bundled_fonts_select_different_native_glyph_geometry(self):
        serif = m.Text("Typography", font="Computer Modern")
        sans = m.Text("Typography", font="IBM Plex Sans")
        mono = m.Text("Typography", font="CM Typewriter")
        self.assertIs(QualifiedText, m.Text)
        self.assertFalse(np.array_equal(points(serif), points(sans)))
        self.assertFalse(np.array_equal(points(sans), points(mono)))
        self.assertEqual(serif.get_string(), "Typography")

    def test_bold_and_italic_equal_native_markup_faces(self):
        for options, content in (({"weight": "BOLD"}, "<b>A</b>"),
                                 ({"slant": "ITALIC"}, "<i>A</i>"),
                                 ({"slant": "ITALIC", "weight": "BOLD"}, "<b><i>A</i></b>")):
            np.testing.assert_array_equal(points(m.Text("A", **options)), points(m.MarkupText(content)))

    def test_markup_overrides_inherited_font_and_face_without_span_rewriting(self):
        source = 'é<span weight="normal" font_family="Computer Modern">B</span>C'
        text = m.MarkupText(source, font="IBM Plex Sans", weight="BOLD")
        self.assertEqual(text.get_string(), source)
        self.assertEqual(text._string_sub_spans[0], (0, 2))
        part = text.get_part_by_text("B")
        expected = m.Text("B")
        np.testing.assert_allclose(points(part.copy().center()), points(expected), atol=1e-6)

    def test_local_face_maps_and_long_aliases_keep_unicode_selectors(self):
        text = m.Text("éABA", text2font={"B": "IBM Plex Sans"}, t2w={"A": "BOLD"}, t2s={"B": "ITALIC"})
        for substring, options in (("A", {"weight": "BOLD"}), ("B", {"font": "IBM Plex Sans", "slant": "ITALIC"})):
            for part in text.get_parts_by_text(substring):
                np.testing.assert_allclose(points(part.copy().center()), points(m.Text(substring, **options)), atol=1e-6)
        self.assertEqual(text._string_sub_spans[0], (0, 2))
        self.assertEqual(len(text.get_parts_by_text("A")), 2)

    def test_global_and_local_gradients_and_final_color_map(self):
        local = m.Text("ABC", t2g={"ABC": [m.RED, m.BLUE]})
        colors = [part.get_fill_color() for part in local]
        self.assertEqual(colors[0], m.RED)
        self.assertEqual(colors[-1], m.BLUE)
        self.assertNotEqual(colors[1], colors[0])
        text = m.Text("ABC", gradient=(c for c in [m.RED, m.BLUE]), t2c={"B": m.GREEN})
        self.assertEqual(text.get_part_by_text("B")[0].get_fill_color(), m.GREEN)

    def test_alignment_changes_lines_not_glyph_shapes(self):
        source = "AAAA\nA"
        left, right = m.Text(source, alignment="LEFT"), m.Text(source, alignment="RIGHT")
        self.assertLess(left[-1].get_x(), right[-1].get_x())
        center = m.Text(source, alignment="CENTER")
        self.assertLess(left[-1].get_x(), center[-1].get_x())
        self.assertLess(center[-1].get_x(), right[-1].get_x())
        self.assertAlmostEqual(left[-1].get_width(), right[-1].get_width(), places=6)

    def test_line_spacing_changes_baselines_without_scaling_ink(self):
        first, second = m.Text("A\nA", lsh=1), m.Text("A\nA", line_spacing_height=2)
        self.assertAlmostEqual(second[0].get_y() - second[1].get_y(),
                               2 * (first[0].get_y() - first[1].get_y()), places=5)
        self.assertAlmostEqual(first[0].get_height(), second[0].get_height(), places=6)

    def test_literal_text_is_not_interpreted_as_markup(self):
        text = m.Text("<b>A</b>", weight="BOLD")
        self.assertEqual(len(text._string_sub_spans), 8)
        self.assertEqual(text.get_string(), "<b>A</b>")

    def test_unknown_fonts_and_unimplemented_styles_refuse(self):
        for options in ({"font": "not-a-font"}, {"t2f": {"A": "not-a-font"}},
                        {"weight": "HEAVY"}, {"slant": "OBLIQUE"},
                        {"alignment": "diagonal"}, {"lsh": 0}, {"line_width": float("nan")},
                        {"t2w": {"A": "BOLDER"}}, {"t2f": {("A", "B"): "Computer Modern"}}):
            with self.subTest(options=options), self.assertRaises((ValueError, TypeError)):
                m.Text("A", **options)
        with self.assertRaises(ValueError):
            m.Text("", font="not-a-font")

    def test_copy_animation_and_matching_preserve_native_spans(self):
        source = m.Text("Move", font="IBM Plex Sans", weight="BOLD", t2c={"M": m.RED})
        duplicate = source.copy()
        scene = m.Scene()
        scene.add(source)
        scene.play(source.animate.shift(m.RIGHT), run_time=.125)
        self.assertFalse(np.array_equal(points(source), points(duplicate)))
        self.assertEqual(source._string_sub_spans, duplicate._string_sub_spans)
        target = m.Text("Move", font="CM Typewriter").shift(m.UP)
        scene.play(m.TransformMatchingStrings(source, target, run_time=.125))
        self.assertIn(target, scene.mobjects)
        self.assertNotIn(source, scene.mobjects)

    def test_constructor_hooks_and_qualified_aliases_are_preserved(self):
        calls = []
        class Authored(m.Text):
            def set_color_by_text_to_color_map(self, mapping):
                calls.append(self)
                return super().set_color_by_text_to_color_map(mapping)
        text = Authored("A", font="IBM Plex Sans", t2w={"A": "BOLD"})
        self.assertEqual(calls, [text])
        self.assertIsInstance(text, QualifiedText)

    def test_inner_monospace_tag_overrides_inherited_family_without_rewriting_source(self):
        source = "<tt>A</tt>"
        actual = m.MarkupText(source, font="IBM Plex Sans")
        expected = m.Text("A", font="CM Typewriter")
        np.testing.assert_array_equal(points(actual), points(expected))
        self.assertEqual(actual._string_sub_spans, [(4, 5)])
        self.assertEqual(actual.get_string(), source)

    def test_resource_bounds_and_nonfinite_paint_are_refused(self):
        import itertools
        cases = (("A" * 262_145, {}),
                 ("A", {"t2w": {str(i): "BOLD" for i in range(4097)}}),
                 ("A", {"gradient": itertools.repeat(m.RED)}),
                 ("A", {"gradient": [[float("nan"), 0, 0]]}),
                 ("A", {"gradient": m.RED}),
                 ("A", {"font_size": float("inf")}),
                 ("A", {"height": 0}))
        for source, options in cases:
            with self.subTest(options=tuple(options)), self.assertRaises((ValueError, TypeError)):
                m.Text(source, **options)
        with self.assertRaisesRegex(NotImplementedError, "Pango attributes"):
            m.Text("A", global_config={"letter_spacing": 10})

    def test_native_frames_match_thread_counts_and_differ_from_plain_text(self):
        def render(output, threads, styled):
            scene = m.Scene()
            with scene.render_session(output, format="png_sequence", resolution=(160, 90), fps=4, threads=threads):
                text = m.Text("Native", font="IBM Plex Sans" if styled else "", weight="BOLD" if styled else "NORMAL")
                scene.add(text)
                scene.play(text.animate(rate_func=m.linear).shift(m.RIGHT), run_time=.5)
            return [path.read_bytes() for path in sorted(Path(output).glob("*.png"))]
        with tempfile.TemporaryDirectory() as root:
            path = Path(root)
            frames = render(path/"one", 1, True)
            self.assertEqual(len(frames), 2)
            self.assertEqual(frames, render(path/"four", 4, True))
            self.assertNotEqual(frames, render(path/"plain", 1, False))
            self.assertNotEqual(frames[0], frames[1])


suite = unittest.defaultTestLoader.loadTestsFromTestCase(TextAuthoringTests)
assert suite.countTestCases() == 14
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError("native text authoring acceptance failed")
