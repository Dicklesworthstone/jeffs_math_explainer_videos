"""Real-native mathematical Markdown scenes; no host parser/layout doubles."""
from __future__ import annotations

import copy
from pathlib import Path
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python.markdown import MarkdownMobject


def geometry(obj):
    return [np.asarray(member.get_points()).copy()
            for member in obj.family_members_with_points()]


def shape(obj):
    runs = geometry(obj)
    points = np.concatenate(runs) if runs else np.empty((0, 3))
    return points - points[0] if len(points) else points


def same_geometry(case, actual, expected):
    a, b = geometry(actual), geometry(expected)
    case.assertEqual(len(a), len(b))
    for left, right in zip(a, b):
        np.testing.assert_allclose(left, right, atol=6e-6)


def content(document, block):
    return document.get_block(block)._markdown_content


class MathematicalDocumentTests(unittest.TestCase):
    def doc(self, source='# Result\n\nBefore $x_i+y_j$ after.\n', **options):
        return MarkdownMobject(source, font_size=32, **options)

    def test_default_formula_is_native_and_source_selection_uses_original_text(self):
        source = '# Ω\n\nBefore $x_i+y_j$ after.\n'
        doc = self.doc(source)
        self.assertTrue(doc.math_mode)
        self.assertIs(doc.select_text('x_i+y_j')[0], doc.get_block(1))
        self.assertIn('Ω', doc.get_block_source(0))
        self.assertIn('$x_i+y_j$', doc.get_block_source(1))
        np.testing.assert_allclose(shape(content(doc, 1)[1]),
                                   shape(m.Tex('x_i+y_j', font_size=32)), atol=3e-6)
        # math_mode=False is the explicit opt-out that keeps dollars literal.
        literal = MarkdownMobject('$x_i+y_j$', math_mode=False, font_size=32)
        self.assertFalse(literal.math_mode)
        self.assertNotEqual(shape(content(literal, 0)).shape,
                            shape(content(self.doc('$x_i+y_j$'), 0)).shape)

    def test_prose_dollars_stay_literal_under_fmds_delimiter_rule(self):
        for source in ('It costs $5 and $10 today.', 'a $ x$ b', 'a $x $ b'):
            with self.subTest(source=source):
                np.testing.assert_allclose(shape(self.doc(source)),
                                           shape(self.doc(source.replace('$', r'\$'))), atol=0)

    def test_math_options_survive_live_edits_and_preserve_unchanged_block_identity(self):
        doc = self.doc(line_width=2.5, body_color=m.GREEN)
        scene = m.Scene(); scene.add(doc)
        title = doc.get_block(0)
        glyph = content(doc, 0).family_members_with_points()[0]
        view = glyph.get_points()
        old = np.asarray(view).copy()
        doc.set_source('# Result\n\nThe value is $\\frac{1}{x}$ now.\n')
        self.assertIs(doc.get_block(0), title)
        np.testing.assert_array_equal(view, old)
        self.assertEqual(doc.line_width, 2.5)
        self.assertEqual(doc.body_color, tuple(m.color_to_rgb(m.GREEN)))
        # `$…$` is inline (text-style) math; Tex itself is display style like
        # the Reference's align*, so the oracle names the inline style.
        np.testing.assert_allclose(shape(content(doc, 1)[3]),
                                   shape(m.Tex(r'\textstyle\frac{1}{x}', font_size=32)), atol=4e-6)
        for glyph in content(doc, 1)[3].family_members_with_points():
            np.testing.assert_allclose(glyph.data['fill_rgba'][:, :3],
                                       np.tile(m.color_to_rgb(m.GREEN), (len(glyph.data), 1)), atol=1e-6)
        self.assertEqual(scene.get_time(), 0)

    def test_copy_pickle_and_checkpoint_keep_math_configuration_and_independent_owners(self):
        doc = self.doc(line_width=3, body_color=m.YELLOW)
        scene = m.Scene(); scene.add(doc)
        state = scene.get_state()
        for clone in (doc.copy(), copy.deepcopy(doc), pickle.loads(pickle.dumps(doc))):
            clone.set_source('# Result\n\nNow $z^2$ is shown.\n')
            self.assertTrue(clone.math_mode)
            self.assertEqual(clone.line_width, 3)
            self.assertIsNot(clone.get_block(1), doc.get_block(1))
            self.assertIn('x_i+y_j', doc.source)
        doc.set_source('# Result\n\nNow $z^2$ is shown.\n')
        scene.restore_state(state)
        self.assertIn('x_i+y_j', doc.source)
        doc.set_source('# Result\n\nRestored $a^2$ document.\n')
        self.assertTrue(doc.math_mode)
        self.assertEqual(doc.line_width, 3)

    def test_invalid_math_and_assets_refuse_before_any_live_geometry_or_catalog_change(self):
        doc = self.doc(); scene = m.Scene(); scene.add(doc)
        before = geometry(doc); source = doc.source
        ranges, owners = doc.block_ranges, tuple(doc.get_blocks())
        for target in [r'$\unknownmathematicaldocumentcommand{x}$',
                       '![external](https://example.invalid/asset.png)',
                       '| x |\n|---|\n| $x_i$ |\n', 'x' * 32769]:
            with self.subTest(target=target[:60]):
                with self.assertRaises((ValueError, RuntimeError)):
                    doc.set_source(target)
                self.assertEqual(doc.source, source)
                self.assertEqual(doc.block_ranges, ranges)
                self.assertEqual(tuple(doc.get_blocks()), owners)
                self.assertEqual(len(geometry(doc)), len(before))
                for a, b in zip(geometry(doc), before):
                    np.testing.assert_array_equal(a, b)
        doc.set_source('# Result\n\nRecovered $x^2$.\n')
        self.assertIn('Recovered', doc.source)
        self.assertEqual(scene.get_time(), 0)

    def test_math_reflow_commutes_with_an_independent_affine_placement(self):
        a, b = self.doc(line_width=2.5), self.doc(line_width=2.5)
        matrix = np.array([[-1, .4, .2], [.2, 1.3, -.1], [.4, .1, 1.]])
        displacement = np.array([2., -1., .4])
        updated = '# Result\n\nLonger $\\frac{1}{x^2}$ mathematical content.\n'
        a.apply_matrix(matrix, about_point=m.ORIGIN).shift(displacement)
        a.set_source(updated)
        b.set_source(updated)
        b.apply_matrix(matrix, about_point=m.ORIGIN).shift(displacement)
        same_geometry(self, a, b)

    def test_native_wrap_separates_lines_without_changing_formula_or_glyph_shapes(self):
        source = r'alpha $\frac{1}{x}$ beta $\sqrt{x}$ gamma'
        wide = self.doc(source)
        narrow = self.doc(source, line_width=1.5)
        self.assertGreater(narrow.get_height(), wide.get_height())
        self.assertEqual(len(content(wide, 0)), len(content(narrow, 0)))
        for a, b in zip(content(wide, 0), content(narrow, 0)):
            np.testing.assert_allclose(shape(a), shape(b), atol=3e-6)
        # At least one wrapped adjacent pair has distinct, nonoverlapping ink.
        pairs = list(zip(content(narrow, 0), content(narrow, 0).submobjects[1:]))
        self.assertTrue(any(a.get_bottom()[1] > b.get_top()[1] for a, b in pairs))

    def test_source_morph_commits_exact_math_endpoint_and_retains_block_wrappers(self):
        doc = self.doc(); scene = m.Scene(); scene.add(doc)
        target = '# Result\n\nBefore $\\frac{1}{x^2}$ after.\n'
        expected = doc.copy(); expected.set_source(target)
        block = doc.get_block(1)
        scene.play(doc.animate_source(target, rate_func=m.linear, path_arc=.3), run_time=.25)
        self.assertEqual(doc.source, target)
        self.assertTrue(doc.math_mode)
        self.assertIs(block, doc.get_block(1))
        same_geometry(self, doc, expected)
        doc.set_source('# Result\n\nFinal $x^3$ value.\n')

    def test_returning_and_aborted_morphs_restore_native_formula_and_source(self):
        doc = self.doc(line_width=3); initial = doc.copy()
        target = '# Result\n\nBefore $\\sqrt{x}$ after.\n'
        anim = doc.animate_source(target, rate_func=m.there_and_back)
        anim.begin(); anim.interpolate(.5); anim.finish()
        self.assertEqual(doc.source, initial.source)
        same_geometry(self, doc, initial)
        anim = doc.animate_source(target, rate_func=m.linear)
        anim.begin(); anim.interpolate(.4); anim.abort()
        self.assertEqual(doc.source, initial.source)
        same_geometry(self, doc, initial)
        doc.set_source(target)
        self.assertTrue(doc.math_mode)

    def test_partial_endpoint_remains_visual_and_checkpoint_restoration_recovers(self):
        doc = self.doc(); scene = m.Scene(); scene.add(doc)
        state = scene.get_state(); before = doc.source
        anim = doc.animate_source('# Result\n\nAfter $x^2$ change.\n',
                                  rate_func=m.linear, final_alpha_value=.5)
        anim.begin(); anim.finish()
        self.assertEqual(doc.source, before)
        with self.assertRaisesRegex(RuntimeError, 'partial'):
            doc.set_source('# New\n\n$x$\n')
        scene.restore_state(state)
        doc.set_source('# New\n\n$x$\n')
        self.assertTrue(doc.math_mode)

    def test_succession_targets_are_resolved_at_begin_and_empty_document_can_gain_math(self):
        doc = self.doc(); scene = m.Scene(); scene.add(doc)
        first = '# Result\n\nFirst $x^2$ state.\n'
        last = '# Result\n\nLast $\\frac{1}{x}$ state.\n'
        scene.play(m.Succession(doc.animate_source(first), doc.animate_source(last)), run_time=.5)
        self.assertEqual(doc.source, last)
        self.assertTrue(doc.math_mode)
        doc.set_source('')
        self.assertEqual(doc.block_ranges, ())
        doc.set_source('```math\nx^2\n```')
        self.assertEqual(doc.block_kinds, ('math',))

    def test_math_mode_admission_is_explicit_and_owned(self):
        for options, error in [({'math_mode': 1}, TypeError),
                               ({'math_mode': False, 'line_width': 2}, ValueError),
                               ({'math_mode': False, 'body_color': m.GREEN}, ValueError),
                               ({'line_width': 0}, ValueError),
                               ({'line_width': float('nan')}, ValueError),
                               ({'body_color': (1, 0, float('nan'))}, ValueError)]:
            with self.subTest(options=options), self.assertRaises(error):
                MarkdownMobject('text', **options)
        doc = self.doc()
        with self.assertRaises(m._CapabilityError):
            doc.animate.set_source('$x$')

    def test_default_math_pixels_match_independent_tex_at_one_and_four_threads(self):
        # The expected scene uses only Tex and explicit placement. It does not
        # call either document builder or inspect the actual document geometry.
        # Fenced math is display style; inline `$…$` is text style.
        for source, tex in [('```math\n\\frac{1}{x^2}\n```', r'\displaystyle\frac{1}{x^2}'),
                            (r'$\frac{1}{x^2}$', r'\textstyle\frac{1}{x^2}')]:
            class Actual(m.Scene):
                def construct(self):
                    self.add(MarkdownMobject(source, font_size=48))
                    self.wait(.25)
            class Expected(m.Scene):
                def construct(self):
                    self.add(m.Tex(tex, font_size=48).center())
                    self.wait(.25)
            with self.subTest(source=source), tempfile.TemporaryDirectory() as tmp:
                previous = None
                for threads in (1, 4):
                    frames = []
                    for cls in (Actual, Expected):
                        path = Path(tmp) / f'{cls.__name__}-{threads}.y4m'
                        receipt = cls().render(path, format='y4m', resolution=(128, 72), fps=8,
                                               threads=threads)
                        self.assertEqual(receipt.frame_count, 2)
                        frames.append(path.read_bytes())
                    self.assertEqual(frames[0], frames[1])
                    if previous is not None:
                        self.assertEqual(frames[0], previous)
                    previous = frames[0]


if __name__ == '__main__':
    unittest.main(verbosity=2)
