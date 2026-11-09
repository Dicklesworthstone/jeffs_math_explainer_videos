"""Native-backed complex readouts, exact family replacement and failed updates."""
import copy
import importlib
from pathlib import Path
import pickle
import tempfile
from types import MethodType
import unittest

import manimlib as m
import numpy as np


def snapshot(mob):
    return (mob.number, mob.num_string, tuple(map(id, mob.get_family())),
            tuple(member.data.copy().tobytes() for member in mob.get_family()))


class DecimalAuthoringTests(unittest.TestCase):
    def test_complex_format_and_native_glyphs(self):
        for value, options, expected in (
            (1 + 2j, {}, "1.00+2.00i"),
            (-3 - 4j, {}, "–3.00–4.00i"),
            (2j, {}, "2.00i"),
            (-2j, {"include_sign": True}, "–2.00i"),
            (2j, {"include_sign": True}, "+2.00i"),
            (3 + 0j, {}, "3.00"),
            (0j, {}, "0.00"),
            (0j, {"hide_zero_components_on_complex": False}, "0.00+0.00i"),
            (2j, {"hide_zero_components_on_complex": False}, "0.00+2.00i"),
            (3 + 0j, {"hide_zero_components_on_complex": False}, "3.00+0.00i"),
            (1234.25 - 5678.5j, {}, "1,234.25–5,678.50i"),
            (1.9 - 2.9j, {"num_decimal_places": 0}, "1–2i"),
            (complex(-0.0001, -0.0001), {}, "0.00+0.00i"),
            (complex(-0.0, -0.0), {"hide_zero_components_on_complex": False}, "0.00+0.00i"),
            (np.complex64(1 + 2j), {}, "1.00+2.00i"),
            (np.complex128(1 + 2j), {}, "1.00+2.00i"),
        ):
            with self.subTest(value=value, options=options):
                value_mob = m.DecimalNumber(value, **options)
                self.assertEqual(value_mob.get_tex(), expected)
                self.assertEqual(value_mob.get_num_string(value), expected)
                self.assertEqual(value_mob.get_value(), value)
                self.assertEqual(len(value_mob), len(expected))
                self.assertTrue(value_mob.family_members_with_points())
                self.assertTrue(all(np.isfinite(member.get_points()).all()
                                    for member in value_mob.family_members_with_points()))

    def test_native_real_format_remains_unchanged(self):
        for value in (-123456.5, -0.001, -0.0, 0, 0.5, 2.5, 1e100):
            for precision in (0, 1, 2):
                for signed in (False, True):
                    real = m.DecimalNumber(value, num_decimal_places=precision,
                                           include_sign=signed, min_total_width=8)
                    reduced = m.DecimalNumber(complex(value, 0), num_decimal_places=precision,
                                              include_sign=signed, min_total_width=8)
                    self.assertEqual(real.get_tex(), reduced.get_tex())
                    self.assertEqual(len(real), len(reduced))
                    for a, b in zip(real.get_family(), reduced.get_family()):
                        np.testing.assert_array_equal(a.data, b.data)

    def test_switching_modes_keeps_anchor_style_and_identity(self):
        mob = m.DecimalNumber(1j, edge_to_fix=m.RIGHT, font_size=30, color=m.BLUE)
        mob.scale(1.5).shift(2 * m.RIGHT + m.UP)
        identity, edge, size = id(mob), mob.get_right().copy(), mob.get_font_size()
        for value, text in ((12345 + 6789j, "12,345.00+6,789.00i"), (2, "2.00"),
                            (-3j, "–3.00i"), (0j, "0.00"), (2 + 3j, "2.00+3.00i")):
            self.assertIs(mob.set_value(value), mob)
            self.assertEqual(id(mob), identity)
            self.assertEqual(mob.get_tex(), text)
            self.assertEqual(len(mob), len(text))
            self.assertEqual(mob.get_font_size(), size)
            np.testing.assert_allclose(mob.get_right(), edge, atol=2e-6)
            self.assertTrue(all(child.get_fill_color() == m.BLUE
                                for child in mob.family_members_with_points()))

    def test_complex_units_ellipsis_and_background(self):
        for unit in (None, "V", "^°", ""):
            for value in (1j, 1 + 2j, -1234.5 - 6.7j):
                with self.subTest(unit=unit, value=value):
                    mob = m.DecimalNumber(value, unit=unit, show_ellipsis=True,
                                          include_background_rectangle=True, color=m.GREEN)
                    self.assertEqual(len(mob), len(mob.num_string) + 1 + int(unit is not None))
                    self.assertGreater(mob.n_records(), 0)
                    for child in mob.submobjects:
                        self.assertGreaterEqual(child.get_left()[0] + 2e-6, mob.get_left()[0])
                        self.assertLessEqual(child.get_right()[0] - 2e-6, mob.get_right()[0])
                    for next_value in (8.25, 3j, 12 + 34j, 0):
                        mob.set_value(next_value)
                        self.assertEqual(len(mob), len(mob.num_string) + 1 + int(unit is not None))
                        self.assertGreater(mob.n_records(), 0)
                        np.testing.assert_allclose(mob.data["fill_rgba"][:, :3], 0)
                        self.assertTrue(all(child.get_fill_color() == m.GREEN
                                            for child in mob.submobjects if child.has_points()))

    def test_failed_values_leave_live_state_untouched(self):
        for bound in (False, True):
            mob = m.DecimalNumber(1 + 2j, include_background_rectangle=True).shift(m.UP)
            scene = m.Scene()
            if bound:
                scene.add(m.VGroup(mob))
            before = snapshot(mob)
            # A real NaN is now a display token (fm-5wq.30); complex
            # nonfinite components still refuse.
            for value in (complex(0, float("nan")),
                          complex(float("inf"), 0), complex(1, float("inf")), "seven", object()):
                with self.subTest(bound=bound, value=value):
                    with self.assertRaises((ValueError, TypeError)):
                        mob.set_value(value)
                    self.assertEqual(snapshot(mob), before)

    def test_combined_character_budget_and_native_failure_are_atomic(self):
        mob = m.DecimalNumber(1 + 2j)
        before = snapshot(mob)
        original = mob._decimal_params
        for index, bad in ((0, -1), (0, 4097), (1, 4097), (6, "x" * 4097),
                           (0, 2047), (1, 2048), (9, -1)):
            params = list(original)
            params[index] = bad
            mob._decimal_params = tuple(params)
            old_font = mob.font_size
            if index == 9:
                mob.font_size = bad
            try:
                with self.assertRaises((ValueError, RuntimeError, OverflowError)):
                    mob.set_value(2 + 3j)
                self.assertEqual(snapshot(mob), before)
            finally:
                mob._decimal_params = original
                mob.font_size = old_font
        # Native glyph admission fails before replacing an otherwise valid row.
        mob._decimal_params = (*original[:6], "\U0010ffff", *original[7:])
        try:
            with self.assertRaises((ValueError, RuntimeError)):
                mob.set_value(2 + 3j)
            self.assertEqual(snapshot(mob), before)
        finally:
            mob._decimal_params = original

    def test_copy_pickle_and_qualified_identity(self):
        self.assertIs(importlib.import_module("manimlib.mobject.numbers").DecimalNumber, m.DecimalNumber)
        mob = m.DecimalNumber(1 + 2j, unit="V", color=m.RED)
        for clone in (mob.copy(), copy.deepcopy(mob), pickle.loads(pickle.dumps(mob))):
            self.assertIs(type(clone), m.DecimalNumber)
            self.assertEqual(clone.get_value(), 1 + 2j)
            clone.set_value(9j)
            self.assertEqual(clone.get_tex(), "9.00i")
            self.assertEqual(mob.get_value(), 1 + 2j)

    def test_subclass_virtual_rebuild_receives_original_value(self):
        seen = []
        class Readout(m.DecimalNumber):
            def set_submobjects_from_number(self, value):
                seen.append(value)
                return super().set_submobjects_from_number(value)
        mob = Readout(1 + 2j)
        mob.set_value(3j)
        self.assertEqual(seen, [1 + 2j, 3j])
        self.assertEqual(mob.get_value(), 3j)

    def test_live_animation_of_nested_readout(self):
        mob = m.DecimalNumber(1 + 2j, include_background_rectangle=True)
        parent = m.VGroup(m.VGroup(mob))
        scene = m.Scene(camera_config=dict(resolution=(192, 108), fps=8))
        scene.add(parent)
        seen = []
        mob.add_updater(lambda readout: seen.append(readout.get_value()))
        scene.play(m.ChangeDecimalToValue(mob, -3j), run_time=0.5)
        self.assertEqual(mob.get_value(), -3j)
        self.assertGreater(len(set(seen)), 1)
        self.assertEqual(list(scene.mobjects), [parent])
        self.assertIs(parent[0][0], mob)
        self.assertEqual(len(mob), len(mob.num_string))
        scene.play(m.ChangingDecimal(mob, lambda alpha: 5 * alpha), run_time=0.25)
        self.assertEqual(mob.get_value(), 5)
        self.assertEqual(mob.get_tex(), "5.00")
        self.assertEqual(list(scene.mobjects), [parent])

    def test_authored_string_and_scalar_formatter_control_actual_glyphs(self):
        calls = []
        class Label(m.DecimalNumber):
            def get_num_string(self, value):
                calls.append(value)
                return "ready" if value == 7 else "go"
        label = Label(7)
        self.assertEqual((label.get_tex(), len(label)), ("ready", 5))
        label.set_value(8)
        self.assertEqual((label.get_tex(), len(label)), ("go", 2))
        self.assertEqual(calls, [7, 8])

        class Scientific(m.DecimalNumber):
            def get_formatter(self, **kwargs):
                return "{:.1e}"
        number = Scientific(1200)
        self.assertEqual(number.get_tex(), "1.2e+03")
        self.assertEqual(len(number), 7)
        for char, child in zip(number.get_tex(), number):
            np.testing.assert_allclose(child.copy().center().get_all_points(),
                                       m.Text(char).get_all_points(), atol=1e-6)

    def test_authored_complex_formatter_and_live_format_parameters(self):
        class Cartesian(m.DecimalNumber):
            def get_complex_formatter(self, **kwargs):
                return "{0.real:.1f}|{0.imag:.1f}"
        number = Cartesian(1 + 2j)
        self.assertEqual((number.get_tex(), len(number)), ("1.0|2.0", 7))
        number.set_value(2 - 3j)
        self.assertEqual(number.get_tex(), "2.0|–3.0")

        number = m.DecimalNumber(1234.5)
        self.assertEqual(number.num_decimal_places, 2)
        self.assertTrue(number.group_with_commas)
        number.num_decimal_places = 0
        number.group_with_commas = False
        number.include_sign = True
        number.min_total_width = 6
        number.show_ellipsis = True
        number.unit = "^°"
        number.include_background_rectangle = True
        number.set_value(12.9)
        self.assertEqual(number.get_tex(), "+00012")
        self.assertEqual(len(number), 8)
        self.assertTrue(number.has_points())
        self.assertEqual(number._formatter_config()["num_decimal_places"], 0)

    def test_glyph_hooks_copy_shared_and_scene_owned_templates(self):
        template = m.Square().shift(2 * m.UP)
        owner = m.Scene()
        owner.add(template)
        before = template.get_points().copy()
        calls = []
        class Shapes(m.DecimalNumber):
            def char_to_mob(self, char):
                calls.append(char)
                return template
        number = Shapes(12, font_size=24)
        self.assertEqual(calls, list("12.00"))
        self.assertEqual(len(set(map(id, number.submobjects))), 5)
        self.assertTrue(all(child is not template for child in number))
        self.assertTrue(all(abs(child.get_width() - 1) < 1e-6 for child in number))
        np.testing.assert_array_equal(template.get_points(), before)
        self.assertEqual(list(owner.mobjects), [template])
        number.set_value(3)
        self.assertEqual(calls, list("12.003.00"))
        np.testing.assert_array_equal(template.get_points(), before)
        destination = m.Scene()
        destination.add(number)
        self.assertEqual(list(destination.mobjects), [number])
        self.assertEqual(list(owner.mobjects), [template])

    def test_formatter_configuration_hook_controls_native_text(self):
        class Precision(m.DecimalNumber):
            def _formatter_config(self):
                return dict(super()._formatter_config(), num_decimal_places=3)
        number = Precision(1.25)
        self.assertEqual((number.get_tex(), len(number)), ("1.250", 5))
        np.testing.assert_allclose(number[-1].copy().center().get_all_points(),
                                   m.Text("0").get_all_points(), atol=1e-6)

    def test_instance_and_late_class_hooks_are_not_lowered_to_scalar(self):
        number = m.DecimalNumber(12)
        number.get_num_string = MethodType(lambda self, value: "changed", number)
        number.set_value(3)
        self.assertEqual((number.get_tex(), len(number)), ("changed", 7))
        original = m.DecimalNumber.char_to_mob
        seen = []
        try:
            def glyph(self, char):
                seen.append(char)
                return m.Triangle()
            m.DecimalNumber.char_to_mob = glyph
            stock = m.DecimalNumber(12)
            self.assertEqual(seen, list("12.00"))
            self.assertTrue(all(len(child.get_points()) == len(m.Triangle().get_points())
                                for child in stock))
        finally:
            m.DecimalNumber.char_to_mob = original

    def test_native_fonts_and_faces_are_available_to_live_counters(self):
        for value in (12.5, 1 + 2j, -3j):
            plain = m.DecimalNumber(value)
            styled = m.DecimalNumber(value, text_config={"font": "IBM Plex Sans", "weight": "BOLD"})
            self.assertEqual(styled.get_tex(), plain.get_tex())
            self.assertEqual(len(styled), len(plain))
            self.assertFalse(np.array_equal(styled.get_all_points(), plain.get_all_points()))
            for char, child in zip(styled.get_tex(), styled):
                expected = m.Text(char, font="IBM Plex Sans", weight="BOLD")
                np.testing.assert_allclose(child.copy().center().get_all_points(),
                                           expected.get_all_points(), atol=1e-6)

    def test_styled_rebuild_preserves_paint_anchor_copy_and_font_size(self):
        config = {"font": "CM Typewriter"}
        number = m.DecimalNumber(123 + 4j, text_config=config,
                                 include_background_rectangle=True, edge_to_fix=m.RIGHT)
        config["font"] = "not-a-font"
        number.set_color(m.BLUE).scale(1.5).shift(m.UP)
        edge, size = number.get_right().copy(), number.get_font_size()
        for value in (1j, 7, 8 + 9j):
            number.set_value(value)
            np.testing.assert_allclose(number.get_right(), edge, atol=2e-6)
            self.assertEqual(number.get_font_size(), size)
            self.assertTrue(all(part.get_fill_color() == m.BLUE
                                for child in number for part in child.family_members_with_points()))
            np.testing.assert_array_equal(number.data["fill_rgba"][:, :3], 0)
        for clone in (number.copy(), copy.deepcopy(number), pickle.loads(pickle.dumps(number))):
            clone.set_value(-2j)
            self.assertEqual(clone.get_tex(), "–2.00i")
            self.assertEqual(clone.text_config, {"font": "CM Typewriter"})
            self.assertEqual(number.get_value(), 8 + 9j)

    def test_authored_failure_and_resource_errors_do_not_publish_partial_digits(self):
        class Fragile(m.DecimalNumber):
            fail = False
            def char_to_mob(self, char):
                if self.fail and char == "9":
                    raise ValueError("authored missing glyph")
                return super().char_to_mob(char)
        number = Fragile(123)
        scene = m.Scene()
        parent = m.VGroup(number)
        scene.add(parent)
        before = snapshot(number)
        number.fail = True
        with self.assertRaisesRegex(ValueError, "authored missing glyph"):
            number.set_value(129)
        self.assertEqual(snapshot(number), before)
        self.assertEqual(list(scene.mobjects), [parent])
        for result in ("x" * 4097, 123):
            number.get_num_string = MethodType(lambda self, value, result=result: result, number)
            with self.assertRaises((ValueError, TypeError)):
                number.set_value(8)
            self.assertEqual(snapshot(number), before)
        del number.get_num_string
        number.char_to_mob = MethodType(lambda self, char: object(), number)
        with self.assertRaisesRegex(TypeError, "must return a VMobject"):
            number.set_value(8)
        self.assertEqual(snapshot(number), before)

    def test_text_configuration_rejects_unsupported_native_options(self):
        for config in ([], {"font_size": 24}, {"font": "not-a-font"},
                       {"weight": "HEAVY"}, {"global_config": {"letter_spacing": 10}}):
            with self.subTest(config=config), self.assertRaises((ValueError, TypeError, NotImplementedError)):
                m.DecimalNumber(1, text_config=config)

    def test_styled_counter_animation_uses_real_native_frames(self):
        class Counters(m.Scene):
            def construct(self):
                number = m.DecimalNumber(1 + 2j, text_config={"font": "IBM Plex Sans", "weight": "BOLD"})
                number.scale(3)
                self.add(number)
                self.play(m.ChangeDecimalToValue(number, 7 - 3j), run_time=.5)
                self.asserted_value = number.get_value()
        root = Path(tempfile.mkdtemp(prefix="fmn-styled-counters-"))
        sequences = []
        for threads in (1, 4):
            scene = Counters()
            receipt = scene.render(root / str(threads), resolution=(192, 108), fps=8, threads=threads)
            self.assertEqual(scene.asserted_value, 7 - 3j)
            self.assertEqual(receipt.frame_count, 4)
            frames = [path.read_bytes() for path in sorted(receipt.destination.glob("*.png"))]
            self.assertEqual(len(set(frames)), 4)
            sequences.append(frames)
        self.assertEqual(*sequences)


if __name__ in ("__main__", "<run_path>"):
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(DecimalAuthoringTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        raise AssertionError("native decimal authoring regressions failed")
