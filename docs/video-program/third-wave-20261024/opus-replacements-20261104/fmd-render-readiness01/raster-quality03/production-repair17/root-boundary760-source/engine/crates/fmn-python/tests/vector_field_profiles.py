"""Native short-vector widths and authored taper profiles, not geometry doubles."""
from pathlib import Path
import copy
import tempfile
import unittest
from unittest.mock import patch

import manimlib as m
import numpy as np


def chart():
    return m.Axes(x_range=(-2, 2, 1), y_range=(-2, 2, 1), width=4, height=4)


def make(cls=m.VectorField, function=None, **kwargs):
    options = dict(sample_coords=[[-1., 0.], [1., 0.]], max_vect_len=float('inf'),
                   stroke_width=4., tip_width_ratio=4., tip_len_to_width=.03125,
                   color=m.WHITE, magnitude_range=(0., 2.))
    options.update(kwargs)
    if function is None:
        function = lambda rows: np.tile([1., 0.], (len(rows), 1))
    return cls(function, chart(), **options)


def native_widths(field):
    # Independent Atlas preparation: not the updater's profile adapter.
    import sys
    g = vars(sys.modules['manimlib.manimlib'])
    scratch = m.VMobject.__new__(m.VMobject)
    g['_install_live_state'](scratch)
    outputs = field.func(np.array(field.sample_coords, copy=True))
    field._build_geometry(g['_native_shell_factory'], outputs, target=scratch)
    return np.array(scratch.data['stroke_width'], copy=True)


class Profiled(m.VectorField):
    gain = 2.

    def init_base_stroke_width_array(self, count):
        super().init_base_stroke_width_array(count)
        self.base_stroke_width_array *= self.gain


class VectorFieldProfileTests(unittest.TestCase):
    def test_stock_constructor_keeps_native_short_vector_attenuation(self):
        field = make(function=lambda rows: np.array([[.125, 0.], [1., 0.]]))
        expected = native_widths(field)
        self.assertEqual(float(expected[0, 0]), 1.)
        self.assertEqual(float(expected[8, 0]), 4.)
        np.testing.assert_array_equal(field.data['stroke_width'], expected)
        field.update_vectors()
        np.testing.assert_array_equal(field.data['stroke_width'], expected)

    def test_zero_field_has_zero_width_at_construction_and_update(self):
        for cls in (m.VectorField, Profiled):
            for tip in (0., .03125):
                field = make(cls, function=lambda rows: np.zeros_like(rows), tip_len_to_width=tip)
                with self.subTest(cls=cls, tip=tip):
                    np.testing.assert_array_equal(field.data['stroke_width'], 0.)
                    field.update_vectors()
                    np.testing.assert_array_equal(field.data['stroke_width'], 0.)
                    self.assertTrue(np.isfinite(field.get_points()).all())

    def test_stock_constructor_evaluates_once_even_with_inferred_range(self):
        for inferred in (False, True):
            calls = []
            def function(rows):
                calls.append(rows.copy())
                return np.tile([len(calls) / 8., 0.], (len(rows), 1))
            field = make(function=function, magnitude_range=None if inferred else (0, 1))
            self.assertEqual(len(calls), 1)
            np.testing.assert_array_equal(field.data['stroke_width'][0::8], 1.)
            field.update_vectors()
            self.assertEqual(len(calls), 2)
            np.testing.assert_array_equal(field.data['stroke_width'][0::8], 2.)

    def test_authored_profile_is_used_during_construction_and_each_update(self):
        field = make(Profiled)
        np.testing.assert_array_equal(field.data['stroke_width'], native_widths(field) * 2)
        field.gain = 3.
        field.update_vectors()
        np.testing.assert_array_equal(field.data['stroke_width'], native_widths(field) * 3)

    def test_native_attenuation_scales_all_custom_lanes_independently(self):
        class Custom(m.VectorField):
            def init_base_stroke_width_array(self, count):
                self.base_stroke_width_array = np.tile([0., 1.5, 2.5, 1., 6., 3., 0., 0.], count)[:-1]
        field = make(Custom, function=lambda rows: np.array([[.125, 0.], [1., 0.]]))
        scales = np.repeat([1., 4.], 8)[:-1]
        np.testing.assert_array_equal(field.data['stroke_width'][:, 0],
                                      scales * field.base_stroke_width_array)
        self.assertEqual(float(field.data['stroke_width'][0, 0]), 0.)
        self.assertEqual(float(field.data['stroke_width'][4, 0]), 6.)

    def test_zero_tip_and_nonzero_vectors_use_native_full_width(self):
        field = make(Profiled, function=lambda rows: np.tile([.125, 0.], (len(rows), 1)),
                     tip_len_to_width=0.)
        np.testing.assert_array_equal(field.data['stroke_width'], native_widths(field) * 2)
        self.assertEqual(float(field.data['stroke_width'][0, 0]), 8.)

    def test_default_profile_keeps_exact_native_head_rounding(self):
        class Forwarding(m.VectorField):
            def init_base_stroke_width_array(self, count):
                super().init_base_stroke_width_array(count)
        for cls in (m.VectorField, Forwarding):
            field = make(cls, function=lambda rows: np.array([[.01, .023], [.6, .41]]),
                         stroke_width=3.7, tip_width_ratio=3.1, max_vect_len=.8)
            field.update_vectors()
            self.assertEqual(field.data['stroke_width'].tobytes(), native_widths(field).tobytes())

    def test_profiles_survive_copy_resampling_and_custom_record_columns(self):
        class Tagged(Profiled):
            data_dtype = m.VectorField.data_dtype + [('tag', 1)]
            def init_points(self):
                super().init_points()
                self.data['tag'][:] = 9.
        field = make(Tagged)
        scene = m.Scene()
        scene.add(field)
        for clone in (field.copy(), copy.deepcopy(field)):
            clone.gain = 3.
            clone.set_sample_coords([[-1, 1], [0, 1], [1, 1]]).update_vectors()
            self.assertEqual(clone.get_num_points(), 23)
            np.testing.assert_array_equal(clone.data['stroke_width'], native_widths(clone) * 3)
            np.testing.assert_array_equal(clone.data['tag'], 9.)
            self.assertEqual(field.get_num_points(), 15)
        field.update_vectors()
        np.testing.assert_array_equal(field.data['stroke_width'], native_widths(field) * 2)

    def test_same_size_width_views_remain_live(self):
        field = make(Profiled)
        view = field.data['stroke_width']
        field.gain = 4.
        field.update_vectors()
        np.testing.assert_array_equal(view, field.data['stroke_width'])
        view[0] = 2.
        self.assertEqual(float(field.data['stroke_width'][0, 0]), 2.)

    def test_bad_profiles_fail_before_native_build_and_preserve_current_records(self):
        field = make(Profiled)
        before, previous = field.data.copy(), field.base_stroke_width_array
        bad_values = [np.ones(14), np.ones((15, 1)), np.full(15, np.nan),
                      np.full(15, -1.), np.full(15, 1j), np.full(15, 1e100)]
        for bad in bad_values:
            def invalid(count):
                field.base_stroke_width_array = bad
            with self.subTest(shape=bad.shape, first=bad.flat[0]):
                with patch.object(field, 'init_base_stroke_width_array', invalid), \
                     patch.object(field, '_build_geometry', side_effect=AssertionError('built invalid profile')):
                    with self.assertRaises((ValueError, TypeError)):
                        field.update_vectors()
                np.testing.assert_array_equal(field.data, before)
                self.assertIs(field.base_stroke_width_array, previous)
        field.update_vectors()

    def test_profile_product_overflow_does_not_partially_publish(self):
        field = make(Profiled)
        before = field.data.copy()
        def too_wide(count):
            field.base_stroke_width_array = np.full(8 * count - 1, np.finfo(np.float32).max)
        with patch.object(field, 'init_base_stroke_width_array', too_wide):
            with self.assertRaisesRegex(ValueError, 'profiled stroke widths'):
                field.update_vectors()
        np.testing.assert_array_equal(field.data, before)

    def test_paint_callback_cannot_silently_replace_a_prepared_profile(self):
        field = make(Profiled)
        before = field.data.copy()
        def mutate(norms):
            field.base_stroke_width_array = np.zeros_like(field.base_stroke_width_array)
            return .5
        field.norm_to_opacity_func = mutate
        with self.assertRaisesRegex(RuntimeError, 'changed during sampling'):
            field.update_vectors()
        np.testing.assert_array_equal(field.data, before)
        field.norm_to_opacity_func = None
        field.update_vectors()

    def test_paint_callback_failure_keeps_geometry_widths_and_custom_lanes(self):
        field = make(Profiled)
        before = field.data.copy()
        error = RuntimeError('authored opacity')
        def broken(norms):
            raise error
        field.norm_to_opacity_func = broken
        field.gain = 3.
        with self.assertRaises(RuntimeError) as caught:
            field.update_vectors()
        self.assertIs(caught.exception, error)
        np.testing.assert_array_equal(field.data, before)
        field.norm_to_opacity_func = None
        field.update_vectors()
        np.testing.assert_array_equal(field.data['stroke_width'], native_widths(field) * 3)

    def test_animated_profiles_match_literal_native_records_and_threads(self):
        def render(path, threads, authored, unchanged=False):
            phase = [0.]
            def function(rows):
                return np.tile([1., phase[0]], (len(rows), 1))
            field = make(Profiled if authored else m.VectorField, function=function,
                         stroke_width=6., max_vect_len=1.)
            def update(obj, dt):
                if not dt:
                    return
                phase[0] += dt
                if authored:
                    obj.gain = 1. + phase[0] * 4
                    obj.update_vectors()
                else:
                    obj.update_vectors()
                    if not unchanged:
                        obj.data['stroke_width'][:] *= 1. + phase[0] * 4
            field.add_updater(update)
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(160, 90),
                                      fps=4, threads=threads) as output:
                scene.add(field)
                scene.wait(.75)
            self.assertEqual(output.result.frame_count, 3)
            frames = [p.read_bytes() for p in sorted(path.glob('*.png'))]
            self.assertEqual(len(set(frames)), 3)
            return frames
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = render(root/'literal', 1, False)
            self.assertNotEqual(expected, render(root/'unchanged', 1, False, True))
            for workers in (1, 4):
                self.assertEqual(render(root/f'authored-{workers}', workers, True), expected)


if __name__ == '__main__':
    unittest.main()
