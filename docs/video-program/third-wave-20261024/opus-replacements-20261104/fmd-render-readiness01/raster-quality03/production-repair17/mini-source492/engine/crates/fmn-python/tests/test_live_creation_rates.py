"""Creation-rate dispatch through production Scene.play and family-alpha code.

Geometry and the clock are explicit fixtures from test_live_rates_protocol;
these tests do not claim compiled-extension or rendered-pixel acceptance.
"""
import math
import unittest

from test_live_rates_protocol import environment, live_rates


def creation_environment(install=True):
    native = environment(False)

    class ShowPartial(native.Animation):
        _native_kind = "show_creation"

        def _native_params(self):
            return {}

        def _native_target(self):
            return None

    class ShowCreation(ShowPartial):
        pass

    class Uncreate(ShowPartial):
        _native_kind = "uncreate"

    class ShowPassingFlash(ShowPartial):
        _native_kind = "show_passing_flash"

    # The actual border/write family is not a ShowPartial subclass.
    class DrawBorderThenFill(native.Animation):
        _native_kind = "draw_border_then_fill"
        _native_params = ShowPartial._native_params
        _native_target = ShowPartial._native_target

    class Write(DrawBorderThenFill):
        _native_kind = "write"

    for cls in (ShowPartial, ShowCreation, Uncreate, ShowPassingFlash,
                DrawBorderThenFill, Write):
        setattr(native, cls.__name__, cls)
    if install:
        live_rates.install_live_rates(native)
    return native


class LiveCreationRateTests(unittest.TestCase):
    names = ("ShowCreation", "Uncreate", "ShowPassingFlash", "DrawBorderThenFill", "Write")

    def test_negative_control_precomputes_creation_rate_before_clock(self):
        native = creation_environment(False)
        scene, phases = native.Scene(), []
        scene.play(native.Write(rate_func=lambda t: phases.append(scene.playing) or t))
        self.assertEqual(phases, [False] * 31)
        self.assertEqual(scene.executions[0][0][0][0], "write")

    def test_all_creation_families_run_authored_curves_on_callback_clock(self):
        for name in self.names:
            with self.subTest(name=name):
                native = creation_environment()
                scene, phases = native.Scene(fps=120), []
                animation = getattr(native, name)(
                    rate_func=lambda t: phases.append(scene.playing) or t*t)
                scene.play(animation)
                self.assertEqual(len(phases), 122)
                self.assertTrue(all(phases))
                self.assertEqual(scene.executions[0][0][0][0], "python_callback")
                self.assertAlmostEqual(animation.values[1][2], (1/120)**2)

    def test_mixed_transform_write_play_does_not_sample_global_rate(self):
        native = creation_environment()
        scene, phases = native.Scene(), []
        first, second = native.Transform(), native.Write()
        scene.play(first, second, rate_func=lambda t: phases.append(scene.playing) or t**3)
        self.assertTrue(all(phases))
        self.assertEqual(first.values, second.values)
        self.assertIsNone(scene.executions[0][4])
        self.assertEqual([spec[0] for spec in scene.executions[0][0]],
                         ["python_callback", "python_callback"])

    def test_high_frequency_easing_is_not_a_thirty_hz_interpolant(self):
        native = creation_environment()
        animation = native.ShowCreation(rate_func=lambda t: t + .02*math.sin(30*math.pi*t))
        native.Scene().play(animation)
        self.assertAlmostEqual(animation.values[1][2], 1/60 + .02)

    def test_family_lag_and_time_window_reach_live_creation_curve(self):
        native = creation_environment()
        animation = native.Write(rate_func=lambda t: t*t, time_span=(.2, .8),
                                 lag_ratio=.4, family_size=2)
        native.Scene().play(animation)
        midpoint = [row[2] for row in animation.values if abs(row[0] - .5) < 1e-12]
        self.assertEqual(len(midpoint), 2)
        for actual, expected in zip(midpoint, [.7**2, .3**2]):
            self.assertAlmostEqual(actual, expected)

    def test_final_alpha_and_live_closure_replacement_are_observable(self):
        native = creation_environment()
        state = {"power": 2}
        animation = native.Uncreate(rate_func=lambda t: t**state["power"])
        animation.final_alpha_value = .5
        state["power"] = 3
        native.Scene().play(animation)
        self.assertEqual(animation.values[-1][2], .125)

    def test_catalog_creation_curves_stay_native(self):
        native = creation_environment()
        for name in self.names:
            for rate in (None, native.linear, native.smooth, "linear"):
                with self.subTest(name=name, rate=rate):
                    scene = native.Scene()
                    animation = getattr(native, name)(rate_func=rate)
                    scene.play(animation)
                    self.assertEqual(scene.executions[0][0][0][0], animation._native_kind)

    def test_prior_authored_callback_can_receive_global_live_rate(self):
        native = creation_environment()
        animation = native.Unsupported()
        animation.authored = True
        scene, phases = native.Scene(), []
        scene.play(animation, rate_func=lambda t: phases.append(scene.playing) or t*t)
        self.assertTrue(all(phases))
        self.assertIsNone(scene.executions[0][4])
        self.assertAlmostEqual(animation.values[1][2], (1/60)**2)

    def test_unhashable_curve_never_runs_hash_or_equality(self):
        native = creation_environment()
        class Rate:
            __hash__ = None
            def __eq__(self, other):
                raise AssertionError("easing equality was probed")
            def __call__(self, t):
                return t*t
        animation = native.Write(rate_func=Rate())
        native.Scene().play(animation)
        self.assertAlmostEqual(animation.values[1][2], (1/60)**2)

    def test_authored_exception_is_raised_during_play_not_spec_construction(self):
        native = creation_environment()
        scene, error = native.Scene(), LookupError("authored rate")
        def rate(t):
            self.assertTrue(scene.playing)
            raise error
        with self.assertRaises(LookupError) as raised:
            scene.play(native.ShowPassingFlash(rate_func=rate))
        self.assertIs(raised.exception, error)


if __name__ == "__main__":
    unittest.main()
