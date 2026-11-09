"""Native coordinate annotations: actual glyphs, ownership and rendered placement."""
from __future__ import annotations

import copy
import itertools
from pathlib import Path
import tempfile
import unittest
from types import MethodType

import numpy as np
import manimlib as m


class NumberLineLabelsTests(unittest.TestCase):
    def assert_points(self, left, right):
        a, b = left.family_members_with_points(), right.family_members_with_points()
        self.assertEqual(len(a), len(b))
        for actual, expected in zip(a, b):
            np.testing.assert_allclose(actual.get_points(), expected.get_points(), atol=2e-6)

    def test_vertical_line_labels_have_actual_numeric_spacing(self):
        line = m.NumberLine((-2, 2, 1)).rotate(m.PI / 2)
        labels = line.add_numbers([-1, 0, 1], direction=m.RIGHT, buff=.2)
        self.assertIs(line.numbers, labels)
        self.assertIsInstance(labels, m.VGroup)
        np.testing.assert_allclose([x.get_center()[1] for x in labels], [-1, 0, 1], atol=1e-6)
        for label in labels:
            self.assertIsInstance(label, m.DecimalNumber)
            self.assertAlmostEqual(label.get_left()[0], .2, places=6)

    def test_reflected_sheared_tilted_line_matches_independent_coordinates(self):
        matrix = np.array([[-1.5, .5, 0], [.75, 2, 0], [.5, -.25, 1.]])
        offset = np.array([.2, -.4, 1.])
        line = m.NumberLine((-2, 3, 1))
        old_origin = line.n2p(0).copy()
        line.apply_matrix(matrix).shift(offset)
        labels = line.add_numbers([-1, 0, 2], direction=m.RIGHT, buff=.3)
        for value, label in zip((-1, 0, 2), labels):
            anchor = matrix @ (old_origin + [value, 0, 0]) + offset
            expected = m.DecimalNumber(value, num_decimal_places=0, font_size=24)
            expected.next_to(anchor, m.RIGHT, buff=.3)
            self.assert_points(label, expected)

    def test_existing_children_views_and_scene_are_not_rebuilt(self):
        line = m.NumberLine((-2, 2, 1)).rotate(.4)
        marker = m.Dot((3, 1, 0)); line.add(marker)
        ticks = line.ticks
        calls = []; updater = lambda mob, dt: calls.append(dt)
        line.add_updater(updater, call=False)
        scene = m.Scene(); scene.add(line)
        line.save_state(); saved = line.saved_state
        # Nursery-to-scene binding is allowed to change record ownership.
        # The operation under test starts after that transfer, not before it.
        records = line.data.copy(); live = line.get_points()
        children = tuple(line.submobjects)
        labels = line.add_numbers([1, 2])
        self.assertEqual(tuple(line.submobjects[:-1]), children)
        self.assertIs(line.ticks, ticks)
        self.assertIs(line.saved_state, saved)
        self.assertEqual(tuple(scene.mobjects), (line,))
        self.assertEqual(calls, [])
        self.assertEqual(scene.get_time(), 0)
        np.testing.assert_array_equal(line.data, records)
        line.shift(m.UP)
        np.testing.assert_allclose(live, records['point'] + m.UP, atol=1e-6)
        self.assertIs(labels._scene, scene)
        self.assertNotIn(labels, saved.submobjects)

    def test_repeated_calls_keep_old_groups_and_append_one_new_group(self):
        line = m.NumberLine((0, 2, 1))
        first = line.add_numbers([0]); before = first.get_center().copy()
        second = line.add_numbers([1, 2])
        self.assertIn(first, line.submobjects)
        self.assertIs(line.numbers, second)
        np.testing.assert_array_equal(first.get_center(), before)
        self.assertEqual([n.get_value() for n in second], [1., 2.])

    def test_default_ticks_exclusions_and_duplicate_values(self):
        line = m.NumberLine((-1, 2, 1), numbers_to_exclude=[0])
        self.assertEqual([x.get_value() for x in line.add_numbers()], [-1, 1, 2])
        self.assertEqual([x.get_value() for x in line.add_numbers([1, 0, 1], excluding=[])], [1, 0, 1])
        tipped = m.NumberLine((0, 2, 1), include_tip=True)
        self.assertEqual([x.get_value() for x in tipped.add_numbers()], [0, 1])

    def test_units_precision_and_signs_use_public_number_builder(self):
        line = m.NumberLine((-4, 4, 1)).rotate(.7)
        kwargs = dict(unit=2, unit_tex='m', num_decimal_places=0, include_sign=True, color=m.RED)
        labels = line.add_numbers([-2, 2, 4], **kwargs)
        for value, label in zip((-2, 2, 4), labels):
            self.assert_points(label, line.get_number_mobject(value, font_size=24, **kwargs))
            self.assertEqual(label.get_value(), value / 2)

    def test_authored_formatter_receives_keywords_and_returns_actual_objects(self):
        calls, returned = [], []
        class Custom(m.NumberLine):
            def get_number_mobject(self, value, *, word, **kwargs):
                calls.append((value, word, kwargs['font_size']))
                label = m.Text(word).move_to(self.n2p(value))
                returned.append(label)
                return label
        line = Custom((0, 1, 1))
        labels = line.add_numbers([0, 1], word='x', font_size=18)
        self.assertEqual(calls, [(0., 'x', 18.), (1., 'x', 18.)])
        self.assertEqual(tuple(labels), tuple(returned))

    def test_authored_tick_range_and_number_mapping_dispatch(self):
        class Custom(m.NumberLine):
            def get_tick_range(self): return iter((.25, .75))
            def number_to_point(self, value): return np.array([value, value * value, 1.])
        line = Custom((0, 1, 1))
        labels = line.add_numbers(direction=m.RIGHT)
        np.testing.assert_allclose([x.get_center()[1] for x in labels], [.0625, .5625], atol=1e-6)
        np.testing.assert_allclose([x.get_center()[2] for x in labels], [1, 1], atol=1e-6)

    def test_later_formatter_failure_never_publishes_partial_labels(self):
        for error in (ValueError('label'), KeyboardInterrupt('cancel')):
            class Custom(m.NumberLine):
                def get_number_mobject(self, value, **kwargs):
                    if value == 1: raise error
                    return super().get_number_mobject(value, **kwargs)
            line = Custom((0, 2, 1)); before = tuple(line.submobjects)
            with self.assertRaises(type(error)) as raised: line.add_numbers([0, 1, 2])
            self.assertIs(raised.exception, error)
            self.assertEqual(tuple(line.submobjects), before)
            self.assertNotIn('numbers', vars(line))
            self.assertEqual(len(line.add_numbers([0])), 1)

    def test_recursive_labeling_refuses_and_next_call_recovers(self):
        class Custom(m.NumberLine):
            recursive = True
            def get_number_mobject(self, value, **kwargs):
                if self.recursive: self.add_numbers([0])
                return super().get_number_mobject(value, **kwargs)
        line = Custom((0, 1, 1))
        with self.assertRaisesRegex(RuntimeError, 'reenter'): line.add_numbers([0])
        line.recursive = False
        self.assertEqual(len(line.add_numbers([0])), 1)

    def test_copy_during_formatter_does_not_inherit_permanent_busy_state(self):
        copies = []
        class Custom(m.NumberLine):
            def get_number_mobject(self, value, **kwargs):
                if not copies:
                    copies.extend((self.copy(), copy.deepcopy(self)))
                    self.save_state(); copies.append(self.saved_state)
                return super().get_number_mobject(value, **kwargs)
        line = Custom((0, 1, 1)); line.add_numbers([0])
        for item in copies: self.assertEqual(len(item.add_numbers([1])), 1)

    def test_generator_or_formatter_mutations_are_not_overwritten(self):
        line = m.NumberLine((0, 1, 1)); before = tuple(line.submobjects)
        def values():
            yield 0
            line.shift(m.UP)
            yield 1
        with self.assertRaisesRegex(RuntimeError, 'changed'): line.add_numbers(values())
        self.assertEqual(tuple(line.submobjects), before)
        self.assertAlmostEqual(line.n2p(0)[1], 1.)
        self.assertEqual(len(line.add_numbers([0])), 1)

    def test_bounded_iterables_and_nonfinite_values_fail_before_formatter(self):
        calls = []
        class Custom(m.NumberLine):
            def get_number_mobject(self, value, **kwargs):
                calls.append(value)
                return super().get_number_mobject(value, **kwargs)
        line = Custom((0, 1, 1))
        for values in (itertools.repeat(1), [0, float('nan')], [float('inf')]):
            with self.assertRaises(ValueError): line.add_numbers(values)
        line.x_step = 1e-50
        with self.assertRaisesRegex(ValueError, 'budget'): line.add_numbers()
        self.assertEqual(calls, [])

    def test_unknown_keyword_keeps_existing_refusal_contract(self):
        line = m.NumberLine((0, 1, 1)); before = tuple(line.submobjects)
        with self.assertRaises(NotImplementedError) as raised: line.add_numbers([0], bogus=True)
        self.assertEqual(str(raised.exception), 'NumberLine.add_numbers() keyword(s) not yet routed to the native builder: bogus')
        self.assertEqual(tuple(line.submobjects), before)

    def test_render_matches_independent_rotated_number_line_at_one_and_four_threads(self):
        angle = .8
        class Labeled(m.Scene):
            manual = False
            def construct(self):
                line = m.NumberLine((-2, 2, 1)).rotate(angle).shift((.5, .3, 0))
                if self.manual:
                    labels = m.VGroup()
                    for value in (-1, 1, 2):
                        anchor = np.array([value*np.cos(angle)+.5, value*np.sin(angle)+.3, 0])
                        label = m.DecimalNumber(value, num_decimal_places=0, font_size=24)
                        label.next_to(anchor, m.RIGHT, buff=.2)
                        labels.add(label)
                    line.add(labels)
                else: line.add_numbers([-1, 1, 2], direction=m.RIGHT, buff=.2)
                self.add(line)
                self.play(line.animate.shift((.4, .2, 0)), run_time=.5, rate_func=m.linear)
        class Reference(Labeled): manual = True
        with tempfile.TemporaryDirectory() as d:
            outputs = []
            for cls, threads in ((Labeled, 1), (Labeled, 4), (Reference, 1), (Reference, 4)):
                path = Path(d) / f'{cls.__name__}-{threads}.y4m'
                result = cls().render(path, format='y4m', resolution=(96, 64), fps=8, threads=threads)
                self.assertEqual(result.frame_count, 4)
                outputs.append(path.read_bytes())
            self.assertTrue(all(data == outputs[0] for data in outputs))


class AxesLabelsTests(unittest.TestCase):
    assert_points = NumberLineLabelsTests.assert_points

    def expected(self, value, anchor, direction, buff=.25, font_size=24, **kwargs):
        label = m.DecimalNumber(value, num_decimal_places=kwargs.pop('num_decimal_places', 0),
                                font_size=font_size, **kwargs)
        label.next_to(anchor, direction, buff=buff)
        if value < 0 and direction[0] == 0:
            label.shift(label[0].get_width() * m.LEFT / 2)
        return label

    def test_labels_follow_reflected_sheared_and_tilted_axes(self):
        matrix = np.array([[-1.2, .5, 0], [.75, 1.5, 0], [.4, -.3, 1.]])
        offset = np.array([.25, -.5, 1.])
        axes = m.Axes((-2, 2, 1), (-2, 2, 1)).apply_matrix(matrix).shift(offset)
        # coordinate_systems.py:520 returns the labels, kept as
        # axes.coordinate_labels = VGroup(x numbers, y numbers).
        labels = axes.add_coordinate_labels([-1, 1], [-1, 1])
        self.assertIs(labels, axes.coordinate_labels)
        self.assertEqual(list(labels), [axes.x_axis.numbers, axes.y_axis.numbers])
        for dim, axis in enumerate(axes.get_axes()):
            direction = m.DOWN if dim == 0 else m.LEFT
            for value, label in zip((-1, 1), axis.numbers):
                coordinate = np.zeros(3); coordinate[dim] = value
                expected = self.expected(value, matrix @ coordinate + offset, direction)
                self.assert_points(label, expected)
                self.assertIsInstance(label, m.DecimalNumber)

    def test_independently_moved_axes_use_their_own_numeric_line(self):
        axes = m.Axes((-2, 2, 1), (-2, 2, 1))
        axes.x_axis.shift((2, 1, .5)); axes.y_axis.shift((-1, 2, 1))
        axes.add_coordinate_labels([1], [1])
        self.assert_points(axes.x_axis.numbers[0], self.expected(1, (3, 1, .5), m.DOWN))
        self.assert_points(axes.y_axis.numbers[0], self.expected(1, (-1, 3, 1), m.LEFT))

    def test_unrelated_annotations_and_prior_labels_do_not_change_layout(self):
        axes = m.Axes((-2, 2, 1), (-2, 2, 1)).rotate(.4)
        axes.add_coordinate_labels([1], [1]); first = (axes.x_axis.numbers, axes.y_axis.numbers)
        old = [[part.get_points().copy() for part in label.family_members_with_points()]
               for axis in axes.get_axes() for label in axis.numbers]
        marker = m.Dot((40, 35, 0)); axes.add(marker)
        axes.add_coordinate_labels([1], [1])
        for index, label in enumerate(label for axis in axes.get_axes() for label in axis.numbers):
            for actual, expected in zip(label.family_members_with_points(), old[index]):
                np.testing.assert_array_equal(actual.get_points(), expected)
        self.assertIn(first[0], axes.x_axis.submobjects)
        self.assertIn(first[1], axes.y_axis.submobjects)
        self.assertIn(marker, axes.submobjects)

    def test_plane_configs_merge_from_actual_axis_inputs_and_call_overrides(self):
        plane = m.NumberPlane((-2, 2, 1), (-2, 2, 1), faded_line_ratio=0,
                              axis_config={'decimal_number_config': {'num_decimal_places': 1}},
                              y_axis_config={'line_to_number_direction': m.RIGHT,
                                             'line_to_number_buff': .4,
                                             'decimal_number_config': {'num_decimal_places': 2}})
        plane.add_coordinate_labels([1], [1], font_size=18)
        self.assert_points(plane.x_axis.numbers[0],
                           self.expected(1, (1, 0, 0), m.DL, .1, 18, num_decimal_places=1))
        self.assert_points(plane.y_axis.numbers[0],
                           self.expected(1, (0, 1, 0), m.RIGHT, .4, 18,
                                         num_decimal_places=2))
        plane.add_coordinate_labels([1], [1], direction=m.UP, buff=.3,
                                    num_decimal_places=3, include_sign=True)
        for dim, axis in enumerate(plane.get_axes()):
            anchor = np.zeros(3); anchor[dim] = 1
            self.assert_points(axis.numbers[0], self.expected(1, anchor, m.UP, .3,
                               num_decimal_places=3, include_sign=True))

    def test_second_axis_failure_preserves_both_axis_families(self):
        for error in (ValueError('y format'), KeyboardInterrupt('cancel')):
            axes = m.Axes((-1, 1, 1), (-1, 1, 1))
            before = tuple(axes.x_axis.submobjects), tuple(axes.y_axis.submobjects)
            def fail(axis, value, **kwargs): raise error
            axes.y_axis.get_number_mobject = MethodType(fail, axes.y_axis)
            with self.assertRaises(type(error)) as raised:
                axes.add_coordinate_labels([1], [1])
            self.assertIs(raised.exception, error)
            self.assertEqual((tuple(axes.x_axis.submobjects), tuple(axes.y_axis.submobjects)), before)
            self.assertNotIn('numbers', vars(axes.x_axis))
            self.assertNotIn('coordinate_labels', vars(axes))
            del axes.y_axis.get_number_mobject
            axes.add_coordinate_labels([1], [1])
            self.assertEqual(len(axes.x_axis.numbers), 1)

    def test_formatter_cannot_overwrite_edits_to_the_other_axis(self):
        axes = m.Axes((-1, 1, 1), (-1, 1, 1))
        points = axes.x_axis.get_points().copy()
        def mutate(axis, value, **kwargs):
            axes.x_axis.shift(m.RIGHT)
            return m.DecimalNumber(value, num_decimal_places=0)
        axes.y_axis.get_number_mobject = MethodType(mutate, axes.y_axis)
        with self.assertRaisesRegex(RuntimeError, 'changed'):
            axes.add_coordinate_labels([1], [1])
        self.assertNotIn('numbers', vars(axes.x_axis))
        np.testing.assert_allclose(axes.x_axis.get_points(), points + m.RIGHT)
        del axes.y_axis.get_number_mobject
        axes.add_coordinate_labels([1], [1])

    def test_custom_axis_mapping_ticks_and_formatter_dispatch(self):
        calls, created = [], []
        class Axis(m.NumberLine):
            def get_tick_range(self): return iter((.25, .75))
            def get_number_mobject(self, value, **kwargs):
                calls.append((self, value))
                label = m.Text('t').move_to((value, value * value, 1))
                created.append(label)
                return label
        axes = m.Axes((0, 1, .5), (0, 1, .5))
        x, y = Axis((0, 1, .5)), Axis((0, 1, .5))
        axes.set_submobjects([x, y]); axes.axes = m.VGroup(x, y)
        axes.x_axis, axes.y_axis = x, y
        axes.add_coordinate_labels()
        self.assertEqual([(axis is x, value) for axis, value in calls],
                         [(True, .25), (True, .75), (False, .25), (False, .75)])
        self.assertEqual(tuple(x.numbers) + tuple(y.numbers), tuple(created))

    def test_chart_dispatch_reentry_refuses_and_recovers(self):
        class Recursive(m.Axes):
            recursive = False
            def get_axes(self):
                if self.recursive: self.add_coordinate_labels([1], [1])
                return super().get_axes()
        axes = Recursive((-1, 1, 1), (-1, 1, 1)); axes.recursive = True
        with self.assertRaisesRegex(RuntimeError, 'reenter'):
            axes.add_coordinate_labels([1], [1])
        axes.recursive = False
        self.assertIs(axes.add_coordinate_labels([1], [1]), axes.coordinate_labels)

    def test_shared_axis_reentry_cannot_publish_another_charts_batch(self):
        first = m.Axes((-1, 1, 1), (-1, 1, 1))
        second = m.Axes((-1, 1, 1), (-1, 1, 1))
        second.x_axis = first.x_axis
        second.set_submobjects([second.x_axis, second.y_axis])
        second.axes = m.VGroup(second.x_axis, second.y_axis)
        before = tuple(first.x_axis.submobjects), tuple(first.y_axis.submobjects)
        def recursive(axis, value, **kwargs):
            second.add_coordinate_labels([1], [1])
            return m.DecimalNumber(value)
        first.y_axis.get_number_mobject = MethodType(recursive, first.y_axis)
        with self.assertRaisesRegex(RuntimeError, 'shared axes'):
            first.add_coordinate_labels([1], [1])
        self.assertEqual((tuple(first.x_axis.submobjects), tuple(first.y_axis.submobjects)), before)
        self.assertNotIn('numbers', vars(second.y_axis))
        del first.y_axis.get_number_mobject
        self.assertIs(first.add_coordinate_labels([1], [1]), first.coordinate_labels)
        self.assertIs(second.add_coordinate_labels([1], [1]), second.coordinate_labels)

    def test_callback_copies_of_chart_remain_labelable(self):
        axes = m.Axes((-1, 1, 1), (-1, 1, 1)); copies = []
        def duplicate(axis, value, **kwargs):
            if not copies:
                copies.extend((axes.copy(), copy.deepcopy(axes)))
                axes.save_state(); copies.append(axes.saved_state)
            return m.DecimalNumber(value, num_decimal_places=0)
        axes.y_axis.get_number_mobject = MethodType(duplicate, axes.y_axis)
        axes.add_coordinate_labels([1], [1])
        for cloned in copies:
            self.assertIs(cloned.add_coordinate_labels([1], [1]), cloned.coordinate_labels)

    def test_bound_plane_grid_views_snapshot_and_animation_ownership(self):
        plane = m.NumberPlane((-1, 1, 1), (-1, 1, 1), faded_line_ratio=1)
        scene = m.Scene(); scene.add(plane)
        plane.save_state(); saved = plane.saved_state
        roots = tuple(plane.submobjects)
        axes = tuple(plane.get_axes()); views = [a.get_points() for a in axes]
        originals = [view.copy() for view in views]
        calls = []; updater = lambda obj, dt: calls.append(dt)
        plane.add_updater(updater, call=False)
        plane.add_coordinate_labels([1], [1])
        self.assertEqual(tuple(plane.submobjects), roots)
        self.assertEqual(tuple(plane.get_axes()), axes)
        self.assertEqual(tuple(scene.mobjects), (plane,))
        self.assertEqual(calls, [])
        self.assertEqual(scene.get_time(), 0)
        self.assertIs(plane.saved_state, saved)
        for axis in axes:
            self.assertIs(axis.numbers._scene, scene)
        scene.play(plane.animate.shift(m.RIGHT), run_time=.25, rate_func=m.linear)
        for live, old in zip(views, originals):
            np.testing.assert_allclose(live, old + m.RIGHT, atol=2e-6)
        self.assertNotIn('numbers', vars(saved.x_axis))
        cloned = plane.copy()
        self.assertIn(cloned.x_axis.numbers, cloned.x_axis.submobjects)
        self.assertIn(cloned.y_axis.numbers, cloned.y_axis.submobjects)
        self.assertIsNot(cloned.x_axis.numbers, plane.x_axis.numbers)
        self.assert_points(cloned.x_axis.numbers, plane.x_axis.numbers)

    def test_default_and_combined_label_budgets_fail_before_formatting(self):
        calls = []
        axes = m.Axes((-1, 1, 1), (-1, 1, 1))
        def format_label(axis, value, **kwargs):
            calls.append(value)
            return m.DecimalNumber(value)
        axes.x_axis.get_number_mobject = MethodType(format_label, axes.x_axis)
        with self.assertRaisesRegex(ValueError, 'budget'):
            axes.add_coordinate_labels(itertools.repeat(1), [])
        with self.assertRaisesRegex(ValueError, 'budget'):
            axes.add_coordinate_labels([1]*2050, [1]*2050)
        # Default labels come from each axis's own tick range
        # (coordinate_systems.py:529), so an explosive axis step is refused
        # by the label budget before any formatting.
        axes.x_axis.x_step = 1e-50
        with self.assertRaisesRegex(ValueError, 'budget'):
            axes.add_coordinate_labels()
        self.assertEqual(calls, [])

    def test_plane_default_exclusions_and_existing_bad_input_refusal(self):
        plane = m.NumberPlane((-1, 1, 1), (-1, 1, 1), faded_line_ratio=0)
        plane.add_coordinate_labels()
        for axis in plane.get_axes():
            self.assertEqual([label.get_value() for label in axis.numbers], [-1, 1])
        with self.assertRaises(TypeError) as raised:
            plane.add_coordinate_labels(excluding=object())
        self.assertEqual(str(raised.exception),
                         'NumberPlane.add_coordinate_labels excluding must be an iterable of real numbers')

    def test_complex_imaginary_units_do_not_leak_into_real_labels(self):
        plane = m.ComplexPlane((-2, 2, 1), (-2, 2, 1), faded_line_ratio=0)
        self.assertIs(plane.add_coordinate_labels([1j, 1, -1j, -1, 2j]), plane)
        labels = plane.coordinate_labels
        self.assertTrue(all(isinstance(label, m.DecimalNumber) for label in labels))
        self.assertEqual([label.get_value() for label in labels], [1, 1, -1, -1, 2])
        self.assertEqual([label.unit for label in labels], ['i', None, 'i', None, 'i'])
        self.assertLess(len(labels[0]), len(labels[-1]))  # i, not 1i.

    def test_complex_pairs_and_dominant_axis_follow_live_native_lines(self):
        matrix = np.array([[1.5, .5, 0], [-.25, 1, 0], [.4, -.2, 1.]])
        offset = np.array([.25, -.5, .75])
        plane = m.ComplexPlane((-2, 2, 1), (-2, 2, 1), faded_line_ratio=0)
        plane.apply_matrix(matrix).shift(offset)
        plane.add_coordinate_labels([(1, .5), (.5, 2), (1, 1)], direction=m.RIGHT, buff=.2)
        for label, coords in zip(plane.coordinate_labels, ((1, 0, 0), (0, 2, 0), (1, 0, 0))):
            expected = self.expected(label.get_value(), matrix @ coords + offset,
                                     m.RIGHT, .2, 36, unit=label.unit)
            self.assert_points(label, expected)

    def test_complex_defaults_skip_first_and_custom_value_provider(self):
        plane = m.ComplexPlane((-2, 2, 1), (-2, 2, 1), faded_line_ratio=0)
        plane.add_coordinate_labels(skip_first=False)
        self.assertEqual([v.get_value() for v in plane.coordinate_labels], [-2, -1, 0, 1, 2, -2, -1, 1, 2])
        plane.add_coordinate_labels(skip_first=True)
        self.assertEqual([v.get_value() for v in plane.coordinate_labels], [-1, 0, 1, 2, -1, 1, 2])
        calls = []
        class Custom(m.ComplexPlane):
            def get_default_coordinate_values(self, skip_first=True):
                calls.append(skip_first); return iter((1, 2j))
        custom = Custom((-2, 2, 1), (-2, 2, 1), faded_line_ratio=0)
        custom.add_coordinate_labels(skip_first=False)
        self.assertEqual(calls, [False])
        self.assertEqual([v.get_value() for v in custom.coordinate_labels], [1, 2])

    def test_three_dimensional_axes_keep_the_existing_xy_label_scope(self):
        axes = m.ThreeDAxes((-1, 1, 1), (-1, 1, 1), (-1, 1, 1)).rotate(.6, axis=m.RIGHT)
        before = tuple(axes.z_axis.submobjects)
        axes.add_coordinate_labels([1], [1])
        self.assertEqual(tuple(axes.z_axis.submobjects), before)
        self.assert_points(axes.y_axis.numbers[0],
                           self.expected(1, (0, np.cos(.6), np.sin(.6)), m.LEFT))

    def test_chart_render_matches_independent_geometry_at_one_and_four_threads(self):
        matrix = np.array([[1, .4, 0], [.5, 1, 0], [0, 0, 1.]])
        class Labeled(m.Scene):
            manual = False
            def construct(self):
                axes = m.Axes((-2, 2, 1), (-2, 2, 1)).apply_matrix(matrix)
                if self.manual:
                    for dim, axis in enumerate(axes.get_axes()):
                        numbers = m.VGroup()
                        for value in (1, 2):
                            coords = np.zeros(3); coords[dim] = value
                            label = m.DecimalNumber(value, num_decimal_places=0, font_size=24)
                            label.next_to(matrix @ coords, m.DOWN if dim == 0 else m.LEFT, buff=.25)
                            numbers.add(label)
                        axis.add(numbers)
                else:
                    axes.add_coordinate_labels([1, 2], [1, 2])
                self.add(axes)
                self.play(axes.animate.shift((.3, .2, 0)), run_time=.5, rate_func=m.linear)
        class Reference(Labeled): manual = True
        with tempfile.TemporaryDirectory() as d:
            outputs = []
            for cls, threads in ((Labeled, 1), (Labeled, 4), (Reference, 1), (Reference, 4)):
                path = Path(d) / f'{cls.__name__}-{threads}.y4m'
                result = cls().render(path, format='y4m', resolution=(96, 64), fps=8, threads=threads)
                self.assertEqual(result.frame_count, 4)
                outputs.append(path.read_bytes())
            self.assertTrue(all(data == outputs[0] for data in outputs))


if __name__ == '__main__':
    unittest.main()
