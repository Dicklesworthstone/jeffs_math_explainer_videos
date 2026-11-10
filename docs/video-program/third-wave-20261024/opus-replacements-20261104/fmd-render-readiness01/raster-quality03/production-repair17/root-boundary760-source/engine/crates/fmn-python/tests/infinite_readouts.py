"""Real infinity and NaN display over finite native glyph records (fm-5wq.30).

No extended-real interpolation or NaN-geometry policy is introduced here.
"""
import copy
import json
import math
from pathlib import Path
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m


class DecoratedNumber(m.DecimalNumber):
    data_dtype = [*m.DecimalNumber.data_dtype, ('tag', np.float32, (1,))]

    def init_points(self):
        super().init_points()
        self.set_points_as_corners([[-1, -1, 0], [1, -1, 0]])
        self.data['tag'][:] = 7
        self.marker = m.Dot().shift(2*m.UP)
        self.add(self.marker)


def snapshot(number):
    return (number.number, number.num_string, tuple(id(child) for child in number.get_family()),
            tuple(child.data.tobytes() for child in number.get_family()))


class InfiniteReadoutTests(unittest.TestCase):
    def assert_finite_geometry(self, number):
        ink = number.family_members_with_points()
        self.assertTrue(ink)
        for child in ink:
            self.assertTrue(np.isfinite(child.data.view(np.float32)).all())

    def test_positive_and_negative_real_infinity_are_display_tokens(self):
        for value, expected in ((math.inf, 'inf'), (-math.inf, '–inf'),
                                (np.float32('inf'), 'inf'), (np.float64('-inf'), '–inf')):
            number = m.DecimalNumber(value)
            self.assertEqual(number.get_tex(), expected)
            self.assertEqual(number.get_value(), value)
            self.assert_finite_geometry(number)
        print(json.dumps({"bead": "fm-5wq.30", "construct": "DecimalNumber(float('inf'))",
                          "observed_text": "inf", "expected_text": "inf",
                          "reference": "6199a00:manimlib/mobject/numbers.py:get_num_string",
                          "finite_records": True, "verdict": "pass"}, sort_keys=True))

    def test_format_precision_padding_sign_and_commas_match_public_formatter(self):
        for precision in (1, 2, 8):
            for value in (math.inf, -math.inf):
                number = m.DecimalNumber(value, num_decimal_places=precision,
                                         include_sign=True, min_total_width=9, group_with_commas=True)
                expected = ('{:+09,.'+str(precision)+'f}').format(value).replace('-', '–')
                self.assertEqual(number.get_tex(), expected)
                self.assert_finite_geometry(number)

    def test_live_transitions_preserve_anchor_color_font_and_scene_identity(self):
        for bound in (False, True):
            number = m.DecimalNumber(12.5, edge_to_fix=m.RIGHT, color=m.BLUE, font_size=32)
            number.shift(2*m.RIGHT + m.UP)
            scene = m.Scene()
            if bound: scene.add(number)
            edge = number.get_right().copy()
            for value, text in ((math.inf, 'inf'), (-math.inf, '–inf'), (2.5, '2.50'),
                                (3+4j, '3.00+4.00i'), (math.inf, 'inf')):
                self.assertIs(number.set_value(value), number)
                self.assertEqual(number.get_tex(), text)
                np.testing.assert_allclose(number.get_right(), edge, atol=2e-6)
                self.assertEqual(number.get_font_size(), 32)
                self.assert_finite_geometry(number)
                self.assertTrue(all(child.get_fill_color() == m.BLUE
                                    for child in number.family_members_with_points()))
                if bound: self.assertIs(scene.mobjects[0], number)

    def test_generated_glyphs_are_replaced_without_losing_custom_records_or_children(self):
        number = DecoratedNumber(2.5)
        marker, data = number.marker, number.data.copy()
        for value in (math.inf, -math.inf, 3.5, math.inf):
            number.set_value(value)
            self.assertIs(number.marker, marker)
            self.assertIn(marker, number.submobjects)
            np.testing.assert_array_equal(number.data['tag'], data['tag'])
            self.assertEqual(len(number), len(number.get_tex()) + 1)
        self.assert_finite_geometry(number)

    def test_units_ellipsis_and_background_use_existing_native_glyph_composition(self):
        for unit in ('m', '^2'):
            number = m.DecimalNumber(math.inf, unit=unit, show_ellipsis=True,
                                     include_background_rectangle=True, color=m.RED)
            self.assertEqual(number.get_tex(), 'inf')
            self.assertEqual(len(number), 5)
            self.assertTrue(number.has_points())
            self.assert_finite_geometry(number)
            number.set_value(-math.inf)
            self.assertEqual(len(number), 6)
            np.testing.assert_array_equal(number.data['fill_rgba'][:, :3], 0)

    def test_copy_deepcopy_and_pickle_keep_independent_values(self):
        number = m.DecimalNumber(math.inf, color=m.GREEN)
        before = snapshot(number)
        for duplicate in (number.copy(), copy.deepcopy(number), pickle.loads(pickle.dumps(number))):
            self.assertEqual(duplicate.get_value(), math.inf)
            duplicate.set_value(123.5)
            self.assertEqual(duplicate.get_tex(), '123.50')
            self.assertEqual(snapshot(number), before)
            self.assertFalse({id(child) for child in duplicate.get_family()} &
                             {id(child) for child in number.get_family()})

    def test_custom_formatter_and_glyph_converter_are_not_bypassed(self):
        calls = []
        class Authored(m.DecimalNumber):
            def get_num_string(self, number):
                calls.append(('format', number))
                return 'limit' if math.isinf(number) else super().get_num_string(number)
            def char_to_mob(self, char):
                calls.append(('glyph', char))
                return super().char_to_mob(char)
        number = Authored(math.inf)
        self.assertEqual(calls, [('format', math.inf), *[('glyph', char) for char in 'limit']])
        self.assertEqual(number.get_tex(), 'limit')
        self.assert_finite_geometry(number)

    def test_failed_special_glyph_leaves_previous_live_readout_intact(self):
        error = RuntimeError('authored glyph failure')
        class Authored(m.DecimalNumber):
            def char_to_mob(self, char):
                if char == 'f': raise error
                return super().char_to_mob(char)
        number = Authored(2.5)
        scene = m.Scene()
        scene.add(number)
        before = snapshot(number)
        with self.assertRaises(RuntimeError) as caught: number.set_value(math.inf)
        self.assertIs(caught.exception, error)
        self.assertEqual(snapshot(number), before)
        number.set_value(3.5)
        self.assertEqual(number.get_tex(), '3.50')

    def test_real_nan_is_a_display_token_like_infinity(self):
        # The pinned Reference formats NaN through the public formatter
        # (`nan`, `+nan`), as _2023/clt/wordy_scenes.py AverageDiceValues shows.
        for kwargs, expected, glyphs in ((dict(), 'nan', 3), (dict(include_sign=True), '+nan', 4),
                                         (dict(unit='m'), 'nan', 4)):
            with self.subTest(**kwargs):
                number = m.DecimalNumber(math.nan, **kwargs)
                self.assertEqual(number.get_tex(), expected)
                self.assertTrue(math.isnan(number.get_value()))
                self.assertEqual(len(number.family_members_with_points()), glyphs)
                self.assert_finite_geometry(number)
        number = m.DecimalNumber(1.5)
        number.set_value(math.nan)
        self.assertEqual(number.get_tex(), 'nan')
        number.set_value(2.25)
        self.assertEqual(number.get_tex(), '2.25')

    def test_complex_nonfinite_values_still_refuse_before_publication(self):
        number = m.DecimalNumber(math.inf)
        before = snapshot(number)
        for value in (complex(math.inf, 0), complex(0, math.inf), complex(0, math.nan)):
            with self.subTest(value=value):
                with self.assertRaises(ValueError): number.set_value(value)
                self.assertEqual(snapshot(number), before)

    def test_integer_format_retains_its_explicit_conversion_failure(self):
        number = m.DecimalNumber(2, num_decimal_places=0)
        before = snapshot(number)
        with self.assertRaises(OverflowError): number.set_value(math.inf)
        self.assertEqual(snapshot(number), before)
        with self.assertRaises(OverflowError): m.Integer(math.inf)
        # The Reference's int(nan): ValueError, as int(inf) is OverflowError.
        with self.assertRaises(ValueError): number.set_value(math.nan)
        self.assertEqual(snapshot(number), before)
        with self.assertRaises(ValueError): m.Integer(math.nan)

    def test_nonfinite_geometry_is_not_admitted_as_a_side_effect(self):
        obj = m.VMobject().set_points_as_corners([[0, 0, 0], [1, 1, 0]])
        before = obj.data.copy()
        for value in (math.inf, -math.inf, math.nan):
            with self.assertRaises(ValueError): obj.set_points([[value, 0, 0]])
            np.testing.assert_array_equal(obj.data, before)

    def test_updater_can_show_a_divergent_limit_and_recover(self):
        number = m.DecimalNumber(1)
        states = iter((math.inf, -math.inf, 2.5))
        seen = []
        def update(obj, dt):
            if dt:
                obj.set_value(next(states))
                seen.append(obj.get_tex())
                self.assert_finite_geometry(obj)
        number.add_updater(update)
        scene = m.Scene()
        with tempfile.TemporaryDirectory() as directory:
            with scene.render_session(Path(directory)/'frames', format='png_sequence',
                                      resolution=(96, 64), fps=3):
                scene.add(number)
                scene.wait(1)
        self.assertEqual(seen, ['inf', '–inf', '2.50'])

    def test_png_sequences_match_independent_literal_glyphs_at_one_and_four_threads(self):
        def render(path, number, threads):
            number.move_to(m.ORIGIN)
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96, 64),
                                      fps=3, threads=threads):
                scene.add(number)
                scene.play(number.animate.shift(2*m.RIGHT), run_time=1, rate_func=m.linear)
            frames = [file.read_bytes() for file in sorted(path.glob('*.png'))]
            self.assertEqual(len(frames), 3)
            self.assertEqual(len(set(frames)), 3)
            return frames
        def literal(text):
            children = [m.Text(char).scale(1.5) for char in text]
            result = m.VGroup(*children).arrange(m.RIGHT, buff=.072, aligned_edge=m.DOWN)
            if text.startswith('–'):
                children[0].align_to(children[1], m.UP)
                children[0].shift(children[1].get_height()*m.DOWN/2)
            result.set_fill(m.BLUE, opacity=1, border_width=.5).set_stroke(m.BLUE, width=0)
            return result
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for sign, value, text in (('positive', math.inf, 'inf'), ('negative', -math.inf, '–inf')):
                expected = render(root/(sign+'-literal'), literal(text), 1)
                for threads in (1, 4):
                    number = m.DecimalNumber(value, color=m.BLUE, font_size=72)
                    self.assertEqual(render(root/f'{sign}-{threads}', number, threads), expected)
                self.assertNotEqual(render(root/(sign+'-wrong'), literal('nan'), 1), expected)


if __name__ == '__main__':
    unittest.main()
