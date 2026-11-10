"""Public scene update phases with real native updater slots and safe admission."""
import gc
import unittest
import weakref

import manimlib as m
import numpy as np


class SceneUpdatePhaseTests(unittest.TestCase):
    def test_native_slots_observe_all_root_host_changes(self):
        first, second = m.Square(), m.Square()
        scene = m.Scene().add(first, second)
        probe = scene._record_field_probe(first, 'point', 0)
        before = first.get_points()[0, 0]
        def move_first(obj, dt):
            first.data['point'][:] += dt * m.RIGHT
        second.add_updater(move_first, call=False)
        scene.update_mobjects(.25)
        self.assertEqual(probe.values(), [before + .25])

    def test_later_root_failure_discards_the_entire_native_phase(self):
        first, second = m.Square(), m.Square()
        scene = m.Scene().add(first, second)
        probe = scene._record_field_probe(first, 'point', 0)
        error = RuntimeError('second root failed')
        def fail(obj, dt): raise error
        second.add_updater(fail, call=False)
        with self.assertRaises(RuntimeError) as caught:
            scene.update_mobjects(.25)
        self.assertIs(caught.exception, error)
        self.assertEqual(probe.values(), [])
        second.remove_updater(fail)
        scene.update_mobjects(.5)
        self.assertEqual(len(probe.values()), 1)

    def test_root_omitting_super_suppresses_native_subtree(self):
        class Root(m.Group):
            def update(self, dt):
                self.shift(dt * m.RIGHT)
                return self
        leaf = m.Square(); root = Root(leaf)
        scene = m.Scene().add(root)
        probe = scene._record_field_probe(leaf, 'point', 0)
        scene.update_mobjects(.25)
        self.assertEqual(probe.values(), [])
        np.testing.assert_allclose(leaf.get_center(), [.25, 0, 0])

    def test_selected_root_dt_and_recursion_reach_native_phase_once(self):
        class Root(m.Group):
            def update(self, dt):
                return super().update(dt * 2, recurse=False)
        leaf = m.Square(); root = Root(leaf)
        other = m.Square(); scene = m.Scene().add(root, other)
        probe = scene._record_field_probe(leaf, 'point', 0)
        live = scene._record_field_probe(other, 'point', 0)
        scene.update_mobjects(.25)
        self.assertEqual(probe.values(), [])
        self.assertEqual(len(live.values()), 1)

    def test_stock_detection_does_not_run_callbacks(self):
        seen = []
        scene = m.Scene().add(m.Group(m.Square(), m.Circle()))
        scene.frame.add_updater(lambda frame, dt: seen.append(dt), call=False)
        self.assertFalse(scene._fmn_requires_public_scene_update())
        self.assertFalse(scene._fmn_dispatch_public_scene_update(.25))
        self.assertEqual(seen, [])

    def test_deep_dt_only_override_runs_on_exact_receiver(self):
        seen = []
        class Leaf(m.Square):
            def update(self, dt):
                seen.append((self, dt)); return super().update(dt)
        # The Reference reaches a nested override only through a family that
        # has updaters (mobject.py:826); the leaf's own updater provides one.
        leaf = Leaf(); leaf.add_updater(lambda obj, dt: None, call=False)
        scene = m.Scene().add(m.Group(m.Group(leaf)))
        self.assertTrue(scene._fmn_requires_public_scene_update())
        self.assertEqual(seen, [])
        self.assertTrue(scene._fmn_dispatch_public_scene_update(.125))
        self.assertEqual(seen, [(leaf, .125)])

    def test_custom_scene_controls_its_entire_update_phase(self):
        seen = []
        class Scene(m.Scene):
            def update_mobjects(self, dt):
                seen.append(dt)
        leaf = m.Square(); scene = Scene().add(leaf)
        probe = scene._record_field_probe(leaf, 'point', 0)
        self.assertTrue(scene._fmn_dispatch_public_scene_update(.2))
        self.assertEqual(seen, [.2]); self.assertEqual(probe.values(), [])

    def test_descriptor_is_classified_without_evaluation(self):
        seen = []
        class Leaf(m.Square):
            @property
            def update(self):
                seen.append('resolve')
                return lambda dt: seen.append(dt)
        scene = m.Scene().add(Leaf())
        self.assertTrue(scene._fmn_requires_public_scene_update())
        self.assertEqual(seen, [])
        self.assertTrue(scene._fmn_dispatch_public_scene_update(.25))
        self.assertEqual(seen, ['resolve', .25])

    def test_late_instance_and_base_patches_are_not_recaptured(self):
        leaf = m.Square(); scene = m.Scene().add(leaf); seen = []
        leaf.update = lambda dt: seen.append(dt)
        self.assertTrue(scene._fmn_requires_public_scene_update())
        del leaf.update
        original = m.Mobject.update
        try:
            m.Mobject.update = lambda self, dt=0: seen.append(dt)
            self.assertTrue(scene._fmn_requires_public_scene_update())
            # Repeated import initialization must not legitimize authored edits.
            from fmn_python.initialization import initialize
            initialize(getattr(m, "_native", m))
            self.assertTrue(scene._fmn_requires_public_scene_update())
        finally:
            m.Mobject.update = original
        self.assertFalse(scene._fmn_requires_public_scene_update())
        self.assertEqual(seen, [])

    def test_shared_descendant_gets_each_path_and_no_extra_native_visit(self):
        class Leaf(m.Square):
            def update(self, dt):
                self.shift(dt * m.RIGHT); return super().update(dt)
        leaf = Leaf(); scene = m.Scene().add(m.Group(leaf), m.Group(leaf))
        probe = scene._record_field_probe(leaf, 'point', 0)
        self.assertTrue(scene._fmn_dispatch_public_scene_update(.125))
        self.assertEqual(len(probe.values()), 2)
        np.testing.assert_allclose(leaf.get_center(), [.25, 0, 0])

    def test_scene_failure_does_not_retain_roots(self):
        def make():
            class Leaf(m.Square):
                def update(self, dt):
                    super().update(dt); raise RuntimeError('fail')
            root = Leaf(); scene = m.Scene().add(root)
            with self.assertRaises(RuntimeError):
                scene._fmn_dispatch_public_scene_update(.25)
            return weakref.ref(root), weakref.ref(scene)
        refs = make(); gc.collect()
        self.assertTrue(all(ref() is None for ref in refs))


if __name__ == '__main__':
    unittest.main()
