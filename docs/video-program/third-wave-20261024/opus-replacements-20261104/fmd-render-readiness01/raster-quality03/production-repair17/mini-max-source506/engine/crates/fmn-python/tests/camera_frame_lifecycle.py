"""Actual native camera-pose initialization and public placement-hook dispatch."""
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


class CameraFrameLifecycleTests(unittest.TestCase):
    def test_pose_exists_during_hooks_and_finalizers_run_in_reference_order(self):
        class Authored(m.CameraFrame):
            def init_data(self):
                self.events = ['data']
                self.seen_core = self._core
                np.testing.assert_array_equal(self.get_center(), m.ORIGIN)
                super().init_data()
            def init_points(self):
                self.events.append('points')
                self.shift(3*m.RIGHT)
                self.reorient(10., 20., 30.)
                super().init_points()
            def init_uniforms(self):
                self.events.append('uniforms')
                super().init_uniforms()
            def set_width(self, value, **kwargs):
                self.events.append(('width', value, kwargs))
                return super().set_width(value, **kwargs)
            def set_height(self, value, **kwargs):
                self.events.append(('height', value, kwargs))
                return super().set_height(value, **kwargs)
            def move_to(self, point, **kwargs):
                self.events.append(('move', tuple(point)))
                return super().move_to(point, **kwargs)
        frame = Authored(frame_shape=(8., 5.), center_point=(2., 1., 0.), fovy=.8)
        self.assertEqual(frame.events, ['data', 'points', 'uniforms',
            ('width', 8., {'stretch': True}), ('height', 5., {'stretch': True}),
            ('move', (2., 1., 0.))])
        self.assertIs(frame.seen_core, frame._core)
        np.testing.assert_array_equal(frame.get_center(), (2., 1., 0.))
        np.testing.assert_array_equal(frame.get_orientation(), (0., 0., 0., 1.))
        self.assertEqual(frame.get_shape(), (8., 5.))
        self.assertAlmostEqual(frame.get_field_of_view(), .8)

    def test_public_sizing_and_placement_overrides_control_the_native_pose(self):
        class Authored(m.CameraFrame):
            def set_width(self, value, **kwargs):
                return super().set_width(value/2, **kwargs)
            def set_height(self, value, **kwargs):
                return super().set_height(value/2, **kwargs)
            def move_to(self, point, **kwargs):
                return super().move_to(np.array(point)+m.RIGHT, **kwargs)
        frame = Authored(frame_shape=(8., 4.), center_point=(2., 0., 0.))
        self.assertEqual(frame.get_shape(), (4., 2.))
        np.testing.assert_array_equal(frame.get_center(), (3., 0., 0.))
        self.assertEqual(tuple(frame._core.shape()), (4., 2.))

    def test_stock_constructor_retains_native_pose_values(self):
        for shape, center, fovy, axes in [
            ((14.222222222222221, 8.), (0., 0., 0.), np.pi/4, 'zxz'),
            ((7.3, 2.1), (3.2, -1.7, .8), .5, 'xyz'),
            ((1e-100, 2e-100), (0., 0., 0.), 1.1, 'zyx'),
        ]:
            with self.subTest(shape=shape):
                expected = m._CameraFrameCore(shape, center, fovy, axes)
                actual = m.CameraFrame(frame_shape=shape, center_point=center, fovy=fovy, euler_axes=axes)
                self.assertEqual(actual._core.shape(), expected.shape())
                self.assertEqual(actual._core.center(), expected.center())
                self.assertEqual(actual._core.field_of_view(), expected.field_of_view())
                self.assertEqual(actual._core.euler_axes(), expected.euler_axes())
                np.testing.assert_array_equal(actual.get_view_matrix(), expected.view_matrix())

    def test_records_decorations_and_updaters_survive_finalization(self):
        updater = lambda mob, dt: None
        class Authored(m.CameraFrame):
            data_dtype = m.CameraFrame.data_dtype + [('mass', 1)]
            def init_points(self):
                self.set_points([[1., 2., 0.]])
                self.data['mass'][:] = 7
                self.marker = m.Dot()
                self.add(self.marker)
                self.add_updater(updater)
                self.shift(m.RIGHT)
        frame = Authored(center_point=(4., 0., 0.))
        self.assertIn(frame.marker, frame.submobjects)
        self.assertIn(updater, frame.updaters)
        np.testing.assert_array_equal(frame.data['mass'], [[7.]])
        np.testing.assert_array_equal(frame.get_center(), (4., 0., 0.))

    def test_copy_taken_during_initialization_has_independent_native_pose(self):
        class Authored(m.CameraFrame):
            def init_points(self):
                self.early_copy = self.copy()
                self.early_copy.shift(m.LEFT)
        frame = Authored(frame_shape=(8., 4.), center_point=(2., 0., 0.))
        self.assertIsNot(frame._core, frame.early_copy._core)
        np.testing.assert_array_equal(frame.early_copy.get_center(), (-1., 0., 0.))
        self.assertEqual(frame.early_copy.get_shape(), (2., 2.))
        np.testing.assert_array_equal(frame.get_center(), (2., 0., 0.))

    def test_copy_and_deepcopy_do_not_repeat_finalization_hooks(self):
        calls = []
        class Authored(m.CameraFrame):
            def set_width(self, value, **kwargs):
                calls.append(value)
                return super().set_width(value/2, **kwargs)
        original = Authored(frame_shape=(8., 4.))
        for deep in (False, True):
            copied = original.copy(deep=deep)
            self.assertEqual(copied.get_width(), 4.)
            copied.shift(m.RIGHT)
            np.testing.assert_array_equal(original.get_center(), m.ORIGIN)
            self.assertIsNot(copied._core, original._core)
        self.assertEqual(calls, [8.])

    def test_hook_failure_is_not_retried_and_no_positioning_follows(self):
        calls, failure = [], RuntimeError('authored camera points')
        class Authored(m.CameraFrame):
            def init_points(self):
                calls.append('points')
                self.shift(m.RIGHT)
                raise failure
            def set_width(self, value, **kwargs):
                calls.append('width')
                return super().set_width(value, **kwargs)
        with self.assertRaises(RuntimeError) as caught:
            Authored()
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, ['points'])
        self.assertEqual(m.CameraFrame().get_height(), 8.)

    def test_invalid_constructor_inputs_refuse_before_hooks(self):
        calls = []
        class Authored(m.CameraFrame):
            def init_points(self):
                calls.append('points')
        for options in [dict(frame_shape=(0., 4.)), dict(center_point=(float('nan'), 0., 0.)),
                        dict(fovy=0), dict(euler_axes='bad')]:
            with self.subTest(options=options), self.assertRaises(ValueError):
                Authored(**options)
        self.assertEqual(calls, [])

    def test_finalizer_failure_does_not_continue_or_hide_the_error(self):
        calls, failure = [], LookupError('authored camera width')
        class Authored(m.CameraFrame):
            def set_width(self, value, **kwargs):
                calls.append('width')
                raise failure
            def set_height(self, value, **kwargs):
                calls.append('height')
                return super().set_height(value, **kwargs)
        with self.assertRaises(LookupError) as caught:
            Authored()
        self.assertIs(caught.exception, failure)
        self.assertEqual(calls, ['width'])

    def test_authored_finalizer_orientation_and_default_reset(self):
        class Authored(m.CameraFrame):
            def move_to(self, point, **kwargs):
                super().move_to(point, **kwargs)
                self.reorient(20., 35., 10.)
                return self
        frame = Authored()
        expected = m.CameraFrame().reorient(20., 35., 10.)
        np.testing.assert_array_equal(frame.get_orientation(), expected.get_orientation())
        frame.make_orientation_default()
        frame.reorient(0, 0, 0).to_default_state()
        np.testing.assert_array_equal(frame.get_orientation(), expected.get_orientation())

    def test_authored_frame_capture_matches_independent_camera(self):
        class Authored(m.CameraFrame):
            def set_width(self, value, **kwargs):
                return super().set_width(value/2, **kwargs)
            def move_to(self, point, **kwargs):
                return super().move_to(np.array(point)+2*m.RIGHT, **kwargs)
        class AuthoredCamera(m.Camera):
            def init_frame(self, **config):
                self.frame = Authored(**config)
        options = dict(resolution=(96, 54), samples=1)
        camera = AuthoredCamera(frame_config=dict(frame_shape=(8., 4.)), **options)
        control = m.Camera(frame_config=dict(frame_shape=(4., 4.), center_point=(2., 0., 0.)), **options)
        shape = m.Triangle(fill_color=m.RED, fill_opacity=1).move_to((2., 0., 0.))
        self.assertEqual(camera.capture_snapshot(shape).png(), control.capture_snapshot(shape).png())
        self.assertNotEqual(camera.get_png(), m.Camera(**options).capture_snapshot(shape).png())

    def test_scene_camera_tracks_authored_pose_with_identical_one_four_thread_frames(self):
        class Authored(m.CameraFrame):
            def set_width(self, value, **kwargs):
                return super().set_width(value/2, **kwargs)
            def move_to(self, point, **kwargs):
                return super().move_to(np.array(point)+2*m.RIGHT, **kwargs)
        def render(path, threads, authored):
            scene = m.Scene(camera_config=dict(resolution=(96, 54), fps=4, samples=1))
            scene.frame = (Authored(frame_shape=(8., 4.)) if authored else
                           m.CameraFrame(frame_shape=(4., 4.), center_point=(2., 0., 0.)))
            with scene.render_session(path, format='png_sequence', resolution=(96, 54), fps=4, threads=threads):
                shape = m.Triangle(fill_color=m.RED, fill_opacity=1).move_to((2., 0., 0.))
                scene.add(shape)
                scene.play(scene.frame.animate.shift(.5*m.RIGHT), run_time=.75, rate_func=m.linear)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        with tempfile.TemporaryDirectory(prefix='fmn-frame-init-') as temp:
            root = Path(temp)
            actual = render(root/'one', 1, True)
            self.assertEqual(len(set(actual)), 3)
            self.assertEqual(actual, render(root/'four', 4, True))
            self.assertEqual(actual, render(root/'control', 1, False))


if __name__ == '__main__':
    unittest.main()
