"""Native numeric readout lifecycle, record ownership, regeneration and pixels."""
from pathlib import Path
import copy
import pickle
import tempfile
import unittest

import numpy as np
import manimlib as m


class DecoratedNumber(m.DecimalNumber):
    data_dtype = m.VMobject.data_dtype + [('mass', 1)]

    def init_data(self):
        super().init_data()
        self.decoration = m.Dot(2 * m.UP)
        self.add(self.decoration)

    def init_points(self):
        super().init_points()
        self.set_points_as_corners([[-1, -1, 0], [1, -1, 0]])
        self.data['mass'][:] = 7


class DecimalLifecycleTests(unittest.TestCase):
    def test_reference_hook_order_and_recipe_visibility(self):
        class Number(m.DecimalNumber):
            def init_data(self):
                self.events = ['data']
                self.recipe = (self.num_decimal_places, self.unit, self.font_size)
                super().init_data()
            def init_points(self):
                self.events.append('points')
                super().init_points()
            def init_uniforms(self):
                self.events.append('uniforms')
                super().init_uniforms()
            def init_colors(self):
                self.events.append(('colors', bool(self.submobjects)))
                return super().init_colors()
            def set_submobjects_from_number(self, value):
                self.events.append(('digits', value))
                return super().set_submobjects_from_number(value)
        number = Number(12.5, num_decimal_places=1, unit='V', font_size=30)
        self.assertEqual(number.recipe, (1, 'V', 30))
        self.assertEqual(number.events, ['data', 'points', 'uniforms', ('colors', False),
                                        ('digits', 12.5), ('colors', True)])

    def test_root_records_custom_lanes_and_decorations_survive_value_changes(self):
        number = DecoratedNumber(123, num_decimal_places=0)
        points = number.get_points().copy()
        for value in [1, 12345, -8, 2 + 3j]:
            number.set_submobjects_from_number(value)
            np.testing.assert_array_equal(number.get_points(), points)
            np.testing.assert_array_equal(number.data['mass'], 7)
            self.assertIn(number.decoration, number.submobjects)
            self.assertEqual(len(number), 1 + len(number.num_string))

    def test_color_hooks_reach_real_glyphs_and_preserve_update_style(self):
        class Green(m.DecimalNumber):
            def init_colors(self):
                super().init_colors()
                self.set_color(m.GREEN)
        number = Green(12)
        for value in (12, 1 + 2j, -3):
            number.set_value(value)
            self.assertTrue(all(leaf.get_fill_color() == m.GREEN
                                for leaf in number.family_members_with_points()))

    def test_background_keeps_root_schema_and_same_size_views_live(self):
        class Number(m.DecimalNumber):
            data_dtype = m.VMobject.data_dtype + [('mass', 1)]
        number = Number(1, include_background_rectangle=True)
        number.data['mass'][:] = 9
        view = number.data
        count = len(view)
        number.set_value(100)
        self.assertEqual(len(number.data), count)
        np.testing.assert_array_equal(number.data['mass'], 9)
        view['mass'][:] = 11
        np.testing.assert_array_equal(number.data['mass'], 11)
        np.testing.assert_array_equal(number.data['fill_rgba'][:, :3], 0)
        number.include_background_rectangle = False
        number.set_value(1)
        self.assertEqual(len(number.data), 0)
        self.assertIn('mass', number.data.dtype.names)
        np.testing.assert_array_equal(view['mass'], 11)

    def test_decorated_root_and_background_coexist_without_duplicate_children(self):
        number = DecoratedNumber(1, include_background_rectangle=True)
        points = number.get_points().copy()
        for value in (2, 20, 200, 3):
            number.set_submobjects_from_number(value)
            np.testing.assert_array_equal(number.get_points(), points)
            np.testing.assert_array_equal(number.data['mass'], 7)
            self.assertEqual(len(number), len(number.num_string) + 2)
            background = number._fmn_decimal_background_child
            self.assertIsNotNone(background)
            self.assertEqual(background.get_fill_color(), m.BLACK)
        number.include_background_rectangle = False
        number.set_submobjects_from_number(1)
        self.assertIsNone(number._fmn_decimal_background_child)
        self.assertEqual(len(number), len(number.num_string) + 1)
        np.testing.assert_array_equal(number.get_points(), points)

    def test_glyph_style_donor_is_not_a_decoration(self):
        number = DecoratedNumber(1, color=m.GREEN)
        number.decoration.set_color(m.RED)
        number.set_value(2)
        self.assertEqual(number.decoration.get_fill_color(), m.RED)
        self.assertTrue(all(leaf.get_fill_color() == m.GREEN
                            for child in number._fmn_decimal_children
                            for leaf in child.family_members_with_points()))

    def test_failures_keep_the_bound_generation_unchanged(self):
        number = DecoratedNumber(1, include_background_rectangle=True)
        scene = m.Scene()
        scene.add(number)
        error = RuntimeError('authored formatter')
        def fail(value):
            raise error
        number.get_num_string = fail
        before = (tuple(map(id, number.get_family())), number.data.copy(), number.number,
                  number.num_string, tuple(number._fmn_decimal_children))
        with self.assertRaises(RuntimeError) as caught:
            number.set_value(2)
        self.assertIs(caught.exception, error)
        self.assertEqual(tuple(map(id, number.get_family())), before[0])
        np.testing.assert_array_equal(number.data, before[1])
        self.assertEqual((number.number, number.num_string, tuple(number._fmn_decimal_children)), before[2:])

    def test_invalid_options_fail_before_hooks(self):
        calls = []
        class Number(m.DecimalNumber):
            def init_data(self):
                calls.append('data')
                return super().init_data()
        # A real NaN is a display token (`nan`, fm-5wq.30); a complex one refuses.
        for options in ({'number': complex(float('nan'), 0)}, {'num_decimal_places': 4097},
                        {'font_size': 0}, {'edge_to_fix': [float('inf'), 0, 0]},
                        {'stroke_width': float('nan')}):
            with self.subTest(options=options), self.assertRaises((ValueError, TypeError)):
                Number(**options)
        self.assertEqual(calls, [])

    def test_init_colors_exception_propagates_without_retry(self):
        calls, failure = [], RuntimeError('authored colors')
        class Number(m.DecimalNumber):
            def init_colors(self):
                calls.append('colors')
                raise failure
        with self.assertRaises(RuntimeError) as caught:
            Number(1)
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, ['colors'])

    def test_copy_deepcopy_and_pickle_remap_generated_children(self):
        number = DecoratedNumber(123, include_background_rectangle=True)
        for clone in (number.copy(), copy.deepcopy(number), pickle.loads(pickle.dumps(number))):
            self.assertIsNot(clone.decoration, number.decoration)
            self.assertIn(clone.decoration, clone.submobjects)
            self.assertTrue(all(child in clone.submobjects for child in clone._fmn_decimal_children))
            clone.set_value(9)
            self.assertEqual(len(clone), len(clone.num_string) + 2)
            self.assertEqual(number.get_value(), 123)
            np.testing.assert_array_equal(clone.data['mass'], 7)

    def test_integer_uses_same_schema_and_hook_protocol(self):
        class Integer(m.Integer):
            data_dtype = m.VMobject.data_dtype + [('mass', 1)]
            def init_points(self):
                self.called = True
                return super().init_points()
        integer = Integer(8)
        self.assertTrue(integer.called)
        integer.set_value(-15)
        self.assertEqual(integer.get_value(), -15)
        self.assertIn('mass', integer.data.dtype.names)

    def test_existing_matrix_cells_remain_updatable(self):
        matrix = m.DecimalMatrix([[1.5, 2.5]])
        cell = matrix.get_entries()[0]
        cell.set_value(13.5)
        cell.init_colors()
        self.assertEqual(cell.get_value(), 13.5)
        self.assertEqual(len(cell), len(cell.num_string))

    def test_animated_readout_preserves_native_root_and_hook_geometry(self):
        number = DecoratedNumber(1, num_decimal_places=0)
        scene = m.Scene()
        scene.add(number)
        points, decoration = number.get_points().copy(), number.decoration
        scene.play(m.ChangeDecimalToValue(number, 12), run_time=2 / 30, rate_func=m.linear)
        self.assertEqual(number.get_value(), 12)
        self.assertIn(decoration, number.submobjects)
        # set_value re-seats the full family's fixed edge. Compare shape, not its
        # translation, so the test does not assume a different anchoring rule.
        np.testing.assert_allclose(np.diff(number.get_points(), axis=0), np.diff(points, axis=0), atol=1e-6)
        np.testing.assert_array_equal(number.data['mass'], 7)

    def test_color_hook_pngs_match_independent_numbers_at_each_sample(self):
        root = Path(tempfile.mkdtemp(prefix='fmn-decimal-lifecycle-'))
        def render(path, threads, authored):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(128, 72), fps=4, threads=threads):
                if authored:
                    class Green(m.DecimalNumber):
                        def init_colors(self):
                            super().init_colors()
                            self.set_color(m.GREEN)
                    number = Green(1, num_decimal_places=0).scale(2)
                    scene.add(number)
                    scene.play(m.ChangeDecimalToValue(number, 9), run_time=1, rate_func=m.linear)
                else:
                    number = m.DecimalNumber(1, num_decimal_places=0, color=m.GREEN).scale(2)
                    edge = number.get_left().copy()
                    # Choreo emits the interval's right-end samples: .25,.5,.75,1.
                    for value in (3, 5, 7, 9):
                        literal = m.DecimalNumber(value, num_decimal_places=0, color=m.GREEN).scale(2)
                        literal.move_to(edge, m.LEFT)
                        scene.clear()
                        scene.add(literal)
                        scene.wait(.25)
            return [file.read_bytes() for file in sorted(path.glob('*.png'))]
        first = render(root / 'one', 1, True)
        self.assertEqual(first, render(root / 'four', 4, True))
        self.assertEqual(first, render(root / 'independent', 1, False))
        self.assertEqual(len(first), 4)
        self.assertGreater(len(set(first)), 1)

    def test_compatible_native_digit_slots_keep_identity_and_authored_attributes(self):
        for bound in (False, True):
            number = m.DecimalNumber(12, num_decimal_places=0, color=m.GREEN)
            if bound:
                scene = m.Scene().add(number)
            digits = tuple(number.submobjects)
            for i, digit in enumerate(digits):
                digit.user_label = ('slot', i)
            edge = number.get_left().copy()
            for value in (21, 98, 11):
                number.set_value(value)
                self.assertEqual(tuple(number.submobjects), digits)
                self.assertEqual(tuple(number._fmn_decimal_children), digits)
                expected = m.DecimalNumber(value, num_decimal_places=0, color=m.GREEN)
                expected.move_to(edge, m.LEFT)
                for i, (actual, literal) in enumerate(zip(digits, expected.submobjects)):
                    self.assertEqual(actual.user_label, ('slot', i))
                    np.testing.assert_array_equal(actual.data, literal.data)

    def test_same_record_count_keeps_live_numpy_views(self):
        for bound in (False, True):
            number = m.DecimalNumber(11, num_decimal_places=0)
            if bound:
                scene = m.Scene().add(number)
            digit = number[0]
            view = digit.get_points()
            before = view.copy()
            number.font_size *= 1.5
            number.set_value(11)
            self.assertIs(number[0], digit)
            self.assertFalse(np.array_equal(view, before))
            np.testing.assert_array_equal(view, digit.get_points())
            view[:] += m.UP
            np.testing.assert_array_equal(view, digit.get_points())

    def test_changed_glyph_record_count_preserves_proxy_but_detaches_old_view(self):
        number = m.DecimalNumber(12, num_decimal_places=0)
        scene = m.Scene().add(number)
        digit = number[0]
        view = digit.get_points()
        before = view.copy()
        number.set_value(21)
        self.assertIs(number[0], digit)
        self.assertNotEqual(len(view), len(digit.data))
        np.testing.assert_array_equal(view, before)
        current = digit.get_points().copy()
        view[:] += m.RIGHT
        np.testing.assert_array_equal(digit.get_points(), current)

    def test_growing_and_shrinking_rows_use_exact_replacement_not_padding(self):
        number = m.DecimalNumber(12, num_decimal_places=0, group_with_commas=False)
        scene = m.Scene().add(number)
        for value in (1234, 1, 23, 0):
            old = tuple(number.submobjects)
            number.set_value(value)
            self.assertEqual(len(number), len(str(value)))
            self.assertFalse(any(child in number.submobjects for child in old))
            self.assertEqual(tuple(number._fmn_decimal_children), tuple(number.submobjects))
            self.assertEqual(len(number.family_members_with_points()), len(str(value)))

    def test_native_complex_digits_ellipsis_and_units_retain_compatible_slots(self):
        for start, end, options in (
            (1+2j, 3+4j, {}), (1200, 3400, {'show_ellipsis': True}),
            (12, 34, {'unit': 'Hz'}), (-12, -34, {'unit': '^V'}),
        ):
            number = m.DecimalNumber(start, num_decimal_places=0, **options)
            old = tuple(number.submobjects)
            edge = number.get_left().copy()
            number.set_value(end)
            literal = m.DecimalNumber(end, num_decimal_places=0, **options).move_to(edge, m.LEFT)
            self.assertEqual(len(old), len(number))
            for previous, actual, expected in zip(old, number.submobjects, literal.submobjects):
                if not previous.submobjects:
                    self.assertIs(actual, previous)
                else:
                    self.assertIsNot(actual, previous)  # Unit group metadata/topology owns replacement.
                for a, b in zip(actual.get_family(), expected.get_family()):
                    np.testing.assert_array_equal(a.data, b.data)

    def test_decorated_background_slot_and_root_records_survive_recycling(self):
        number = DecoratedNumber(12, num_decimal_places=0, include_background_rectangle=True)
        scene = m.Scene().add(number)
        previous = tuple(number._fmn_decimal_children)
        background = number._fmn_decimal_background_child
        points = number.get_points().copy()
        number.set_submobjects_from_number(21)
        self.assertIs(number._fmn_decimal_background_child, background)
        self.assertEqual(tuple(number._fmn_decimal_children), previous)
        self.assertIn(number.decoration, number.submobjects)
        np.testing.assert_array_equal(number.data['mass'], 7)
        np.testing.assert_array_equal(number.get_points(), points)
        self.assertEqual(background.get_fill_color(), m.BLACK)

    def test_generated_copies_recycle_their_own_digits_only(self):
        source = DecoratedNumber(12, num_decimal_places=0)
        originals = tuple(source._fmn_decimal_children)
        source_points = [x.data.copy() for x in originals]
        for clone in (source.copy(), copy.deepcopy(source), pickle.loads(pickle.dumps(source))):
            digits = tuple(clone._fmn_decimal_children)
            self.assertFalse(any(x in originals for x in digits))
            clone.set_value(21)
            self.assertEqual(tuple(clone._fmn_decimal_children), digits)
            self.assertEqual(source.get_value(), 12)
            for old, points in zip(originals, source_points):
                np.testing.assert_array_equal(old.data, points)

    def test_matrix_cell_digits_and_attached_updaters_remain_live(self):
        matrix = m.DecimalMatrix([[12, 13]], num_decimal_places=0)
        cell = matrix.get_entries()[0]
        scene = m.Scene().add(matrix)
        digit = cell[0]
        calls = []
        def tick(obj, dt):
            calls.append(obj)
        digit.add_updater(tick, call=False)
        cell.set_value(21)
        self.assertIs(cell[0], digit)
        self.assertIn(tick, digit.updaters)
        calls.clear()
        scene.wait(.125)
        self.assertTrue(calls)
        self.assertTrue(all(obj is digit for obj in calls))

    def test_failed_digit_writer_rolls_back_all_preceding_native_writes(self):
        number = DecoratedNumber(12, num_decimal_places=0, include_background_rectangle=True)
        scene = m.Scene().add(number)
        members = tuple(number.get_family())
        before = [member.data.copy() for member in members]
        generated = tuple(number._fmn_decimal_children)
        failure = RuntimeError('digit writer committed then failed')
        digit = generated[-1]
        original = digit.set_data
        first = [True]
        def fail_once(data):
            original(data)
            if first[0]:
                first[0] = False
                raise failure
            return digit
        digit.set_data = fail_once
        with self.assertRaises(RuntimeError) as caught:
            number.set_submobjects_from_number(21)
        self.assertIs(caught.exception, failure)
        self.assertEqual(number.get_value(), 12)
        self.assertEqual(number.num_string, '12')
        self.assertEqual(tuple(number.get_family()), members)
        self.assertEqual(tuple(number._fmn_decimal_children), generated)
        for member, data in zip(members, before):
            np.testing.assert_array_equal(member.data, data)
        number.set_value(21)
        self.assertEqual(tuple(number._fmn_decimal_children), generated)

    def test_failed_parent_splice_restores_recycled_digits_and_metadata(self):
        number = m.DecimalNumber(12, num_decimal_places=0, include_background_rectangle=True)
        scene = m.Scene().add(number)
        members = tuple(number.get_family())
        before = [member.data.copy() for member in members]
        original, first = number.set_submobjects, [True]
        failure = RuntimeError('parent splice committed then failed')
        def fail_once(children):
            result = original(children)
            if first[0]:
                first[0] = False
                raise failure
            return result
        number.set_submobjects = fail_once
        with self.assertRaises(RuntimeError) as caught:
            number.set_value(21)
        self.assertIs(caught.exception, failure)
        self.assertEqual(number.get_value(), 12)
        self.assertEqual(tuple(number.get_family()), members)
        for member, data in zip(members, before):
            np.testing.assert_array_equal(member.data, data)
        number.set_value(21)
        self.assertEqual(number.get_value(), 21)

    def test_authored_text_glyphs_keep_their_source_metadata_replacement(self):
        number = m.DecimalNumber(12, num_decimal_places=0, text_config={'font': 'IBM Plex Sans'})
        old = tuple(number.submobjects)
        number.set_value(21)
        self.assertEqual([child.get_string() for child in number.submobjects], ['2', '1'])
        self.assertTrue(all(isinstance(child, m.Text) for child in number.submobjects))
        self.assertFalse(any(child in number.submobjects for child in old))
        self.assertEqual([child.get_string() for child in old], ['1', '2'])

    def test_removed_digit_reference_is_not_mutated_or_reinserted(self):
        number = m.DecimalNumber(12, num_decimal_places=0)
        removed, retained = number[0], number[1]
        number.remove(removed)
        before = removed.data.copy()
        number.set_submobjects_from_number(21)
        self.assertIsNot(number[0], removed)
        self.assertIs(number[1], retained)
        np.testing.assert_array_equal(removed.data, before)
        self.assertEqual(len(number), 2)

    def test_recycled_animation_keeps_digit_references_and_matches_native_literals(self):
        def render(path, threads, animated):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(128, 72), fps=4, threads=threads):
                number = m.DecimalNumber(10, num_decimal_places=0, color=m.BLUE).scale(2)
                edge, digits = number.get_left().copy(), tuple(number.submobjects)
                if animated:
                    scene.add(number)
                    seen = []
                    number.add_updater(lambda obj: seen.append(tuple(obj.submobjects)) if obj is number else None, call=False)
                    scene.play(m.ChangeDecimalToValue(number, 19), run_time=1, rate_func=m.linear)
                    self.assertTrue(seen)
                    self.assertTrue(all(row == digits for row in seen))
                else:
                    for value in (12, 14, 16, 19):
                        literal = m.DecimalNumber(value, num_decimal_places=0, color=m.BLUE).scale(2)
                        literal.move_to(edge, m.LEFT)
                        scene.clear().add(literal)
                        scene.wait(.25)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        root = Path(tempfile.mkdtemp(prefix='fmn-recycled-decimal-'))
        control = render(root/'literal', 1, False)
        self.assertEqual(len(control), 4)
        self.assertGreater(len(set(control)), 1)
        for threads in (1, 4, 16):
            self.assertEqual(render(root/str(threads), threads, True), control)


if __name__ in ('__main__', '<run_path>'):
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(DecimalLifecycleTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        raise AssertionError('native decimal lifecycle regressions failed')
