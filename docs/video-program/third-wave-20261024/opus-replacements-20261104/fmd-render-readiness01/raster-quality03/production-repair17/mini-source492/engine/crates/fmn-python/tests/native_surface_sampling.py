"""Exercise the raw native sampler, bypassing the Python callback guard."""
import unittest
import numpy as np
import manimlib as m


def raw(target, function, shape=(2, 2), epsilon=.001, nudge=.001):
    return m._FMN_ADMITTED_ORIGINAL_build_parametric_surface(
        target, m._native_surface_shell_factory, function, (0., 1.), (0., 1.), shape, epsilon, nudge)


class NativeSurfaceSamplingTests(unittest.TestCase):
    def test_native_error_channel_stops_without_the_host_guard(self):
        for stop in (1, 2, 3, 7, 12):
            for kind in (RuntimeError, KeyboardInterrupt, SystemExit):
                with self.subTest(stop=stop, kind=kind):
                    obj = m.Surface(resolution=(2, 2))
                    before, view = obj.data.copy(), obj.get_points()
                    calls, error = [], kind('native cancellation')
                    def sample(u, v):
                        calls.append((u, v))
                        if len(calls) == stop:
                            raise error
                        return u, v, 0.
                    with self.assertRaises(kind) as caught:
                        raw(obj, sample)
                    self.assertIs(caught.exception, error)
                    self.assertEqual(len(calls), stop)
                    np.testing.assert_array_equal(obj.data, before)
                    np.testing.assert_array_equal(view, before['point'])

    def test_native_numeric_refusal_stops_at_each_probe(self):
        for stop in (1, 2, 3, 5):
            calls = []
            def sample(u, v):
                calls.append((u, v))
                return (float('nan'), v, 0.) if len(calls) == stop else (u, v, 0.)
            with self.assertRaises(ValueError):
                raw(m.Surface(resolution=(2, 2)), sample)
            self.assertEqual(len(calls), stop)

    def test_raw_native_admission_precedes_callbacks(self):
        obj = m.Surface(resolution=(2, 2))
        before = obj.data.copy()
        # (513, 512) is one row past the surface sampling budget, Marionette's
        # 262144-record ceiling since fm-1rb8 (0725c122) raised it from 65536.
        for options in ({'shape': (2**62, 2)}, {'shape': (0, 2**62)}, {'shape': (513, 512)},
                        {'epsilon': 0.}, {'nudge': -1.}):
            calls = []
            with self.assertRaises(ValueError):
                raw(obj, lambda u, v: calls.append((u, v)) or (u, v, 0.), **options)
            self.assertEqual(calls, [])
            np.testing.assert_array_equal(obj.data, before)

    def test_bound_raw_target_refuses_before_callback(self):
        obj = m.Surface(resolution=(2, 2))
        scene = m.Scene()
        scene.add(obj)
        calls = []
        with self.assertRaisesRegex(RuntimeError, 'detached'):
            raw(obj, lambda u, v: calls.append((u, v)) or (u, v, 0.))
        self.assertEqual(calls, [])
        self.assertIs(obj._scene, scene)

    def test_exact_three_components_are_required_by_native_extraction(self):
        obj = m.Surface(resolution=(2, 2))
        for value in ((1, 2), (1, 2, 3, 4)):
            calls = []
            with self.assertRaises(ValueError):
                raw(obj, lambda u, v: calls.append((u, v)) or value)
            self.assertEqual(len(calls), 1)


def run_native_surface_sampling():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeSurfaceSamplingTests))
    if not result.wasSuccessful():
        raise AssertionError('native fallible surface sampling failed')


if __name__ in ('__main__', '<run_path>'):
    run_native_surface_sampling()
