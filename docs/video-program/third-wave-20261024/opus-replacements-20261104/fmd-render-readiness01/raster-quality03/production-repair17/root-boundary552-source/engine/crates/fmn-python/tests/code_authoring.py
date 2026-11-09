"""Native token styling and live Code glyphs, without a Python lexer or renderer."""
import gc
import inspect
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from manimlib.mobject.svg.text_mobject import Code as QualifiedCode


def colors(part):
    return [glyph.get_fill_color() for glyph in part.family_members_with_points()]


class CodeAuthoringTests(unittest.TestCase):
    def test_rust_tokens_do_not_bleed_into_identifier_substrings(self):
        source = "fn fnord() { let outlet = 42; }"
        code = m.Code(source, language="rust")
        self.assertEqual(set(colors(code.get_part_by_text("fn", 0))), {"#68C246"})
        self.assertNotIn("#68C246", colors(code.get_part_by_text("fnord")))
        self.assertEqual(set(colors(code.get_part_by_text("42"))), {"#F7830E"})
        self.assertEqual(code.get_string(), source)
        self.assertEqual(code.get_num_points(), 0, "glyphs must not be duplicated into the root")

    def test_python_strings_and_comments_are_styled_positionally(self):
        source = 'if gift == "if": # if\n    return 12'
        code = m.Code(source)
        occurrences = code.get_parts_by_text("if")
        self.assertEqual(len(occurrences), 4)
        # 'gift' contains 'if', but its identifier must not inherit the keyword fill.
        self.assertNotEqual(colors(occurrences[0]), colors(occurrences[1]))
        self.assertEqual(set(colors(occurrences[2])), {"#F2BD45"})
        self.assertEqual(set(colors(occurrences[3])), {"#717C68"})

    def test_theme_names_select_owned_native_palettes(self):
        light = m.Code("fn", language="rust", code_style="default")
        dark = m.Code("fn", language="rust", code_style="monokai")
        alias = m.Code("fn", language="rust", code_style="dracula")
        self.assertEqual(set(colors(light)), {"#02008B"})
        self.assertEqual(set(colors(dark)), {"#68C246"})
        self.assertEqual(colors(dark), colors(alias))
        with self.assertRaisesRegex(ValueError, "unsupported native Code theme"):
            m.Code("fn", code_style="not-a-theme")

    def test_unknown_language_uses_declared_plain_native_fallback(self):
        code = m.Code('fn "text" 42', language="not-a-language")
        self.assertEqual(set(colors(code)), {m.WHITE})
        self.assertEqual(code.language, "not-a-language")

    def test_code_font_policy_and_explicit_bundled_fonts(self):
        code = m.Code("ABC")
        self.assertEqual(code.font, "Consolas")
        self.assertEqual(code.native_font, "CM Typewriter")
        reference = m.Text("ABC", font="CM Typewriter", font_size=24)
        np.testing.assert_array_equal(code.get_all_points(), reference.get_all_points())
        explicit = m.Code("ABC", font="IBM Plex Sans")
        self.assertEqual(explicit.native_font, "IBM Plex Sans")
        self.assertFalse(np.array_equal(code.get_all_points(), explicit.get_all_points()))
        with self.assertRaises(ValueError):
            m.Code("ABC", font="not-installed")

    def test_unicode_source_and_markup_characters_keep_original_byte_spans(self):
        source = 'let café = "<b>&é</b>";'
        code = m.Code(source, language="rust")
        self.assertEqual(code.code, source)
        self.assertEqual(code.text, source)
        encoded = source.encode("utf-8")
        for start, end in code._string_sub_spans:
            self.assertTrue(encoded[start:end].decode("utf-8"))
        self.assertEqual(len(code.get_parts_by_text("é")), 2)
        self.assertEqual(len(code.get_part_by_text("<b>")), 3)

    def test_line_spacing_and_local_face_options_share_text_layout(self):
        code = m.Code("A\nA", lsh=2, font="Computer Modern", t2w={"A": "BOLD"})
        text = m.Text("A\nA", lsh=2, font="Computer Modern", weight="BOLD", font_size=24)
        np.testing.assert_array_equal(code.get_all_points(), text.get_all_points())
        normal = m.Code("A\nA", lsh=1, font="Computer Modern", t2w={"A": "BOLD"})
        self.assertAlmostEqual(code[0].get_y() - code[1].get_y(),
                               2 * (normal[0].get_y() - normal[1].get_y()), places=6)

    def test_uniform_and_selector_colors_override_highlighting(self):
        code = m.Code("fn main", language="rust", fill_color=m.GREEN)
        self.assertEqual(set(colors(code)), {m.GREEN})
        code = m.Code("fn main", language="rust", gradient=[m.RED, m.BLUE], t2c={"fn": m.GREEN})
        self.assertEqual(set(colors(code.get_part_by_text("fn"))), {m.GREEN})
        code.get_part_by_text("main").set_color(m.YELLOW)
        self.assertEqual(set(colors(code.get_part_by_text("main"))), {m.YELLOW})

    def test_selector_edits_and_translucent_ink_equal_independent_text_pixels(self):
        code = m.Code("fn main", language="rust", fill_opacity=.5)
        text = m.Text("fn main", font="CM Typewriter", font_size=24, fill_opacity=.5,
                      t2c={"fn": "#68C246"})
        camera = m.Camera()
        camera.reset_pixel_shape(160, 90)
        self.assertEqual(camera.capture_snapshot(code).pixels(), camera.capture_snapshot(text).pixels())
        code.get_part_by_text("fn").set_color(m.BLUE)
        text.get_part_by_text("fn").set_color(m.BLUE)
        self.assertEqual(camera.capture_snapshot(code).pixels(), camera.capture_snapshot(text).pixels())

    def test_copy_and_matching_keep_token_colors_and_live_source_handles(self):
        code = m.Code("fn main", language="rust")
        duplicate = code.copy()
        code.get_part_by_text("fn").set_color(m.RED)
        self.assertEqual(set(colors(duplicate.get_part_by_text("fn"))), {"#68C246"})
        scene = m.Scene().add(code)
        target = m.Code("fn next", language="rust").shift(m.RIGHT)
        scene.play(m.TransformMatchingStrings(code, target, run_time=.125))
        self.assertIn(target, scene.mobjects)
        self.assertNotIn(code, scene.mobjects)
        self.assertEqual(target.get_string(), "fn next")
        self.assertNotIn("_fmn_code_layout", vars(target))

    def test_alias_signature_and_authored_receiver_survive_installation(self):
        calls = []
        class Authored(m.Code):
            def set_color_by_text_to_color_map(self, mapping):
                calls.append(self)
                return super().set_color_by_text_to_color_map(mapping)
        code = Authored("fn", language="rust")
        self.assertIs(QualifiedCode, m.Code)
        self.assertEqual(calls, [code])
        self.assertEqual(m.Code.__bases__, (m.MarkupText,))
        self.assertEqual(str(inspect.signature(m.Code)),
            "(code, font='Consolas', font_size=24, lsh=1.0, fill_color=None, "
            "stroke_color=None, language='python', code_style='monokai', **kwargs)")
        failed = m.Code.__new__(m.Code)
        with self.assertRaises(ValueError):
            m.Code.__init__(failed, "x", code_style="missing")
        self.assertNotIn("_fmn_code_layout", vars(failed))

    def test_real_code_frames_are_repeatable_and_differ_from_plain_tokens(self):
        def render(output, threads, language):
            scene = m.Scene()
            with scene.render_session(output, format="png_sequence", resolution=(160, 90), fps=4, threads=threads):
                code = m.Code("fn main() { 42 }", language=language, font_size=48)
                scene.add(code)
                scene.play(code.animate(rate_func=m.linear).shift(m.RIGHT), run_time=.5)
            return [file.read_bytes() for file in sorted(Path(output).glob("*.png"))]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            frames = render(root / "one", 1, "rust")
            self.assertEqual(len(frames), 2)
            self.assertEqual(frames, render(root / "four", 4, "rust"))
            self.assertNotEqual(frames, render(root / "plain", 1, "no-lexer"))
            self.assertNotEqual(frames[0], frames[1])


suite = unittest.defaultTestLoader.loadTestsFromTestCase(CodeAuthoringTests)
assert suite.countTestCases() == 12
result = unittest.TextTestRunner(verbosity=2).run(suite)
gc.collect()
if not result.wasSuccessful():
    raise AssertionError("native Code authoring acceptance failed")
