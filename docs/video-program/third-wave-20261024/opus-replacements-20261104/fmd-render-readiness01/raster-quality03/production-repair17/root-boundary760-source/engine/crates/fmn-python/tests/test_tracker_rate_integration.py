"""Both production installers plus production frontend, with modeled storage/clock."""
import ast
import importlib.util
from pathlib import Path
import sys
import unittest

import numpy as np

from test_tracker_interpolation_protocol import environment as tracker_environment
from test_live_rates_protocol import environment as rate_environment

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))
from fmn_python.playback import install_scene_playback


def environment():
    native, storage = rate_environment(False), tracker_environment(False)
    native.Mobject = storage.Mobject
    native._BridgeMobject = (native._BridgeMobject, storage.Mobject)
    storage.Mobject._is_bound = lambda self:getattr(self, "bound", False)
    native.ValueTracker, native.ComplexValueTracker = storage.ValueTracker, storage.ComplexValueTracker
    native.ExponentialValueTracker = storage.ExponentialValueTracker
    native._np, native._OUT = np, (0., 0., 1.)
    class Restore(native.Animation):
        pass
    native.Restore = Restore
    install_scene_playback(native)
    return native


def tracker_transform(n, start, end, rate=None):
    animation = n.Transform(rate_func=rate)
    animation.mobject, animation.target_mobject = n.ValueTracker(start), n.ValueTracker(end)
    def begin():
        animation._ensure_runtime_defaults()
        animation.families = [(animation.mobject, animation.mobject.copy(), animation.target_mobject)]
        animation.interpolate(0.)
    animation.begin = begin
    def interpolate(current, first, last, alpha):
        current.interpolate(first, last, alpha)
        animation.values.append(float(current.get_value()))
    animation.interpolate_submobject = interpolate
    return animation


class TrackerRateIntegrationTests(unittest.TestCase):
    def test_playback_installs_both_features_and_uses_live_rate_to_drive_native_value(self):
        n = environment()
        self.assertTrue(n._FMN_TRACKER_INTERPOLATION_INSTALLED)
        self.assertTrue(n._FMN_LIVE_RATES_INSTALLED)
        animation = tracker_transform(n, 0, 1, lambda t:t*t)
        scene = n.Scene()
        scene.play(animation)
        self.assertAlmostEqual(animation.values[1], (1/60)**2)
        self.assertEqual(animation.mobject.get_value(), 1)
        self.assertEqual(scene.executions[0][0][0][0], "python_callback")

    def test_global_curve_and_final_alpha_reach_native_tracker(self):
        n = environment()
        animation = tracker_transform(n, 2, 10)
        animation.final_alpha_value = .25
        scene = n.Scene()
        scene.play(animation, rate_func=lambda t:t*t)
        self.assertEqual(animation.mobject.get_value(), 2.5)
        self.assertIsNone(scene.executions[0][4])

    def test_catalog_default_does_not_force_native_tracker_into_callback(self):
        n = environment()
        animation = n.Transform(rate_func=n.linear)
        animation.mobject, animation.target_mobject = n.ValueTracker(0), n.ValueTracker(1)
        scene = n.Scene()
        scene.lower_only = True
        scene.play(animation)
        self.assertEqual(scene.executions[0][0][0][0], "transform")


if __name__ == "__main__":
    unittest.main()
