"""Native play/wait must dispatch public scene and drawable update protocols."""
from fractions import Fraction
import math
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


# BN-02: count the exact binary64 duration, not its rounded product with fps.
# In particular .1 is strictly above 3/30, while nextafter(.1, 0) is below.
def frame_count(duration, fps=30):
    return max(0, math.ceil(Fraction.from_float(duration) * fps))


class Moving(m.Square):
    def update(self, dt):
        if not self._is_updating_suspended():
            self.shift(dt * m.RIGHT)
        return super().update(dt)


class NativeSceneUpdateTests(unittest.TestCase):
    def test_wait_detects_override_without_an_updater_list(self):
        seen = []
        class Leaf(m.Square):
            def update(self, dt):
                seen.append((self, dt, self._scene.time))
                return super().update(dt)
        leaf = Leaf(); scene = m.Scene().add(leaf)
        self.assertEqual(list(leaf.updaters), [])
        scene.wait(.1)
        self.assertEqual([who for who, _, _ in seen], [leaf] * 5)
        np.testing.assert_allclose([dt for _, dt, _ in seen], [0, 1/30, 1/30, 1/30, 1/30])
        np.testing.assert_allclose([time for _, _, time in seen], [0, 1/30, 2/30, 3/30, 4/30])

    def test_wait_reaches_nested_dt_only_children(self):
        # The Reference reaches a nested override only through a family that
        # has updaters (mobject.py:826); the leaf's own updater provides one.
        leaf = Moving(); leaf.add_updater(lambda obj, dt: None, call=False)
        scene = m.Scene().add(m.Group(m.Group(leaf)))
        scene.wait(.1)
        np.testing.assert_allclose(leaf.get_center(), [4/30, 0, 0], atol=1e-6)

    def test_play_updates_unanimated_override_and_runs_native_slots_once(self):
        leaf = Moving(); animated = m.Square().shift(m.UP)
        scene = m.Scene().add(leaf, animated)
        probe = scene._record_field_probe(leaf, 'point', 0)
        scene.play(animated.animate.shift(m.UP), run_time=.1, rate_func=m.linear)
        np.testing.assert_allclose(leaf.get_center(), [4/30, 0, 0], atol=1e-6)
        np.testing.assert_allclose(animated.get_center(), [0, 2, 0], atol=1e-6)
        self.assertEqual(len(probe.values()), 5)  # four frames and final zero-dt

    def test_wait_initial_zero_and_per_frame_slots_are_not_duplicated(self):
        leaf = Moving(); scene = m.Scene().add(leaf)
        probe = scene._record_field_probe(leaf, 'point', 0)
        scene.wait(.1)
        self.assertEqual(len(probe.values()), 5)
        np.testing.assert_allclose(leaf.get_center(), [4/30, 0, 0], atol=1e-6)

    def test_root_omitting_super_suppresses_default_native_work(self):
        class Root(m.Group):
            def update(self, dt):
                self.shift(dt * m.RIGHT)
                return self
        leaf = m.Square(); root = Root(leaf); scene = m.Scene().add(root)
        probe = scene._record_field_probe(leaf, 'point', 0)
        scene.wait(.1)
        self.assertEqual(probe.values(), [])
        np.testing.assert_allclose(leaf.get_center(), [4/30, 0, 0], atol=1e-6)

    def test_root_recursion_choice_and_dt_are_respected(self):
        seen = []
        class Root(m.Group):
            def update(self, dt):
                return super().update(dt * 2, recurse=False)
        leaf = Moving(); root = Root(leaf); scene = m.Scene().add(root)
        root.add_updater(lambda _, dt: seen.append(dt), call=False)
        probe = scene._record_field_probe(leaf, 'point', 0)
        scene.wait(2/30)
        np.testing.assert_allclose(seen, [0, 2/30, 2/30])
        np.testing.assert_array_equal(leaf.get_center(), [0, 0, 0])
        self.assertEqual(probe.values(), [])

    def test_custom_scene_owns_the_whole_phase(self):
        seen = []
        class Scene(m.Scene):
            def update_mobjects(self, dt):
                seen.append((dt, self.time))
        leaf = m.Square(); scene = Scene().add(leaf)
        probe = scene._record_field_probe(leaf, 'point', 0)
        scene.wait(2/30)
        np.testing.assert_allclose(seen, [[0,0], [1/30,1/30], [1/30,2/30]])
        self.assertEqual(probe.values(), [])

    def test_scene_override_super_keeps_all_host_callbacks_before_native(self):
        class Scene(m.Scene):
            def update_mobjects(self, dt):
                return super().update_mobjects(dt)
        first, second = m.Square(), m.Square()
        scene = Scene().add(first, second)
        probe = scene._record_field_probe(first, 'point', 0)
        before = first.get_points()[0,0]
        def move(_, dt): first.data['point'][:] += dt * m.RIGHT
        second.add_updater(move, call=False)
        scene.wait(2/30)
        np.testing.assert_allclose(probe.values(), [before, before+1/30, before+2/30], atol=1e-6)

    def test_camera_override_runs_before_drawables(self):
        seen = []
        leaf = Moving(); scene = m.Scene().add(leaf)
        original = scene.frame.update
        def update(dt):
            seen.append((dt, float(leaf.get_center()[0])))
            return original(dt)
        scene.frame.update = update
        scene.wait(2/30)
        np.testing.assert_allclose(seen, [[0,0], [1/30,0], [1/30,1/30]], atol=1e-6)

    def test_late_update_patch_is_seen_on_the_next_wait(self):
        leaf = m.Square(); scene = m.Scene().add(leaf)
        scene.wait(1/30)
        original = leaf.update
        def update(dt):
            leaf.shift(dt * m.RIGHT); return original(dt)
        leaf.update = update
        scene.wait(2/30)
        np.testing.assert_allclose(leaf.get_center(), [2/30,0,0], atol=1e-6)

    def test_descriptor_is_not_evaluated_by_route_inspection(self):
        seen = []
        class Leaf(m.Square):
            @property
            def update(self):
                seen.append('resolve')
                return lambda dt: seen.append(dt)
        scene = m.Scene().add(Leaf())
        self.assertEqual(seen, [])
        scene.wait(2/30)
        self.assertEqual(seen, ['resolve',0.,'resolve',1/30,'resolve',1/30])

    def test_shared_child_and_its_native_slot_run_once_per_path(self):
        leaf = Moving(); scene = m.Scene().add(m.Group(leaf), m.Group(leaf))
        probe = scene._record_field_probe(leaf, 'point', 0)
        scene.wait(2/30)
        self.assertEqual(len(probe.values()), 6)
        np.testing.assert_allclose(leaf.get_center(), [4/30,0,0], atol=1e-6)

    def test_positive_dt_failure_preserves_exception_and_discards_native_work(self):
        error = LookupError('authored scene update failed')
        class Broken(m.Square):
            def update(self, dt):
                if dt > 0: raise error
                return super().update(dt)
        first, broken = m.Square(), Broken(); scene = m.Scene().add(first, broken)
        probe = scene._record_field_probe(first, 'point', 0)
        with self.assertRaises(LookupError) as caught: scene.wait(.1)
        self.assertIs(caught.exception, error)
        self.assertEqual(len(probe.values()), 1)  # initial zero only
        self.assertAlmostEqual(scene.time, 1/30)
        broken.update = lambda dt: m.Mobject.update(broken, dt)
        scene.wait(1/30)
        self.assertEqual(len(probe.values()), 3)
        self.assertAlmostEqual(scene.time, 2/30)

    def test_stop_condition_observes_completed_authored_updates(self):
        leaf = Moving(); scene = m.Scene().add(leaf)
        scene.wait(1., stop_condition=lambda: leaf.get_center()[0] > .05)
        self.assertAlmostEqual(scene.time, 2/30)
        np.testing.assert_allclose(leaf.get_center(), [2/30,0,0], atol=1e-6)

    def test_skip_mode_runs_the_same_authored_grid_transitions(self):
        outputs = []
        for skip in (False, True):
            leaf = Moving(); scene = m.Scene(skip_animations=skip).add(leaf)
            scene.wait(.1)
            outputs.append((leaf.get_points().copy(), scene.time))
        np.testing.assert_array_equal(outputs[0][0], outputs[1][0])
        self.assertEqual(outputs[0][1], outputs[1][1])
        np.testing.assert_allclose(outputs[0][0].mean(axis=0)[0] - m.Square().get_points().mean(axis=0)[0], 4/30, atol=1e-6)

    def test_camera_only_play_uses_the_same_complete_phase(self):
        leaf = Moving(); scene = m.Scene().add(leaf)
        before = scene.frame.get_center().copy()
        probe = scene._record_field_probe(leaf, 'point', 0)
        scene.play(scene.frame.animate.shift(m.UP), run_time=.1, rate_func=m.linear)
        np.testing.assert_allclose(leaf.get_center(), [4/30,0,0], atol=1e-6)
        np.testing.assert_allclose(scene.frame.get_center(), before+m.UP, atol=1e-6)
        self.assertEqual(len(probe.values()), 5)  # public camera play: 4 frames, final zero-dt

    def test_stop_callback_can_install_an_override_for_the_next_frame(self):
        leaf = m.Square(); scene = m.Scene().add(leaf); calls = []
        original = leaf.update
        def update(dt):
            calls.append(dt); leaf.shift(dt*m.RIGHT); return original(dt)
        def condition():
            leaf.update = update
            return False
        scene.wait(.1, stop_condition=condition)
        np.testing.assert_allclose(calls, [1/30,1/30,1/30])
        np.testing.assert_allclose(leaf.get_center(), [3/30,0,0], atol=1e-6)

    def test_binary_duration_edges_preserve_stock_clock_and_exact_phase_counts(self):
        self.assertEqual(frame_count(.1), 4)
        self.assertEqual(frame_count(math.nextafter(.1, 0.)), 3)
        self.assertEqual(frame_count(.125), 4)
        for duration in (.1, math.nextafter(.1, 0.), .125, 1/30):
            n = frame_count(duration)
            for mode in ('wait', 'play', 'camera'):
                expected_dt = ([0.] if mode == 'wait' else []) + [1/30] * n
                if mode != 'wait':
                    expected_dt.append(0.)
                outcomes = []
                for authored in (False, True):
                    with self.subTest(duration=duration, mode=mode, authored=authored):
                        seen = []
                        class Leaf(m.Square):
                            def update(self, dt):
                                seen.append(dt)
                                self.shift(dt * m.RIGHT)
                                return super().update(dt)
                        leaf = Leaf() if authored else m.Square()
                        if not authored:
                            def update(obj, dt):
                                seen.append(dt)
                                obj.shift(dt * m.RIGHT)
                            leaf.add_updater(update, call=False)
                        scene = m.Scene().add(leaf)
                        if mode == 'wait':
                            scene.wait(duration)
                        elif mode == 'camera':
                            scene.play(scene.frame.animate.shift(m.UP), run_time=duration,
                                       rate_func=m.linear)
                        else:
                            animated = m.Square().shift(m.UP)
                            scene.add(animated)
                            scene.play(animated.animate.shift(m.UP), run_time=duration,
                                       rate_func=m.linear)
                        outcomes.append(leaf.get_points().copy())
                        self.assertEqual(seen, expected_dt)
                        self.assertEqual(scene.time, n/30)
                        np.testing.assert_allclose(leaf.get_center(), [n/30,0,0], atol=1e-6)
                np.testing.assert_array_equal(outcomes[0], outcomes[1])

    def test_failed_play_releases_animation_state_for_reuse(self):
        error = LookupError('observer failed during play')
        class Broken(m.Square):
            def update(self, dt):
                if dt > 0: raise error
                return super().update(dt)
        broken, animated = Broken(), m.Square()
        scene = m.Scene().add(broken, animated)
        with self.assertRaises(LookupError) as caught:
            scene.play(animated.animate.shift(m.UP), run_time=.1)
        self.assertIs(caught.exception, error)
        self.assertFalse(animated._is_updating_suspended())
        self.assertFalse(animated.locked_data_keys)
        broken.update = lambda dt: m.Mobject.update(broken, dt)
        scene.play(animated.animate.shift(m.RIGHT), run_time=1/30)
        self.assertFalse(animated._is_updating_suspended())

    def test_wait_and_play_pixels_match_independent_updater_controls(self):
        def render(path, custom, threads):
            leaf = Moving() if custom else m.Square()
            if not custom:
                leaf.add_updater(lambda obj, dt: obj.shift(dt * m.RIGHT), call=False)
            animated = m.Square().shift(2*m.UP)
            scene = m.Scene()
            with scene.render_session(path, resolution=(80,48), fps=24, threads=threads):
                scene.add(leaf, animated)
                scene.wait(.125)
                scene.play(animated.animate.shift(m.LEFT), run_time=.125, rate_func=m.linear)
            return path.read_bytes(), leaf.get_points()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected, points = render(root/'control.y4m', False, 1)
            for threads in (1,4,16):
                actual, final = render(root/f'actual-{threads}.y4m', True, threads)
                self.assertEqual(actual, expected)
                np.testing.assert_array_equal(final, points)
            np.testing.assert_allclose(final.mean(axis=0)[0]-m.Square().get_points().mean(axis=0)[0], .25, atol=1e-6)


def run_native_scene_update_dispatch():
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(NativeSceneUpdateTests)
    result = unittest.TextTestRunner().run(suite)
    if not result.wasSuccessful():
        raise AssertionError("native scene update dispatch acceptance failed")


if __name__ == '__main__':
    run_native_scene_update_dispatch()
