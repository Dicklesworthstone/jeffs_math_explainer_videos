"""Real-extension acceptance for authored AnimationGroup lifecycles."""
import unittest
import numpy as np
from manimlib import AnimationGroup, Flash, Scene, Square, Succession, Transform, VGroup


class RecordingGroup(AnimationGroup):
    def begin(self):
        self.begin_count = getattr(self, "begin_count", 0) + 1
        self.positions = []
        super().begin()
    def interpolate(self, alpha):
        super().interpolate(alpha)
        self.positions.append((float(alpha), self.animations[0].mobject.get_center().copy()))
    def finish(self):
        super().finish()
        self.finish_count = getattr(self, "finish_count", 0) + 1


class CompositionLifecycleAcceptance(unittest.TestCase):
    def test_native_motion_runs_inside_authored_group(self):
        source = Square()
        target = source.copy().shift((2, 0, 0))
        group = RecordingGroup(Transform(source, target, rate_func=lambda a: a), run_time=2 / 30)
        scene = Scene()
        scene.add(source)
        scene.play(group)
        self.assertEqual(group.begin_count, 1)
        self.assertEqual(group.finish_count, 1)
        self.assertTrue(any(abs(alpha - .5) < 1e-7 and np.allclose(point, [1, 0, 0])
                            for alpha, point in group.positions))
        np.testing.assert_allclose(source.get_center(), [2, 0, 0], atol=1e-6)
        self.assertIsNone(group._composition_driver)
        self.assertIsNone(group._composition_scene)

    def test_nested_custom_group_keeps_hooks(self):
        source = Square()
        inner = RecordingGroup(Transform(source, source.copy().shift((2, 0, 0))))
        scene = Scene()
        scene.add(source)
        scene.play(AnimationGroup(inner), run_time=2 / 30)
        self.assertEqual(inner.begin_count, 1)
        self.assertEqual(inner.finish_count, 1)
        np.testing.assert_allclose(source.get_center(), [2, 0, 0], atol=1e-6)

    def test_mixed_succession_snapshots_previous_native_result(self):
        source = Square()
        start_centers = []
        class AuthoredTransform(Transform):
            def begin(self):
                start_centers.append(self.mobject.get_center().copy())
                super().begin()
        class AuthoredSuccession(Succession):
            def begin(self):
                self.called = True
                super().begin()
        sequence = AuthoredSuccession(
            Transform(source, source.copy().shift((1, 0, 0)), run_time=2 / 30),
            AuthoredTransform(source, source.copy().shift((2, 0, 0)), run_time=2 / 30),
        )
        scene = Scene()
        scene.add(source)
        scene.play(sequence)
        self.assertTrue(sequence.called)
        self.assertEqual(len(start_centers), 1)
        np.testing.assert_allclose(start_centers[0], [1, 0, 0], atol=1e-6)
        np.testing.assert_allclose(source.get_center(), [2, 0, 0], atol=1e-6)

    def test_direct_lifecycle_with_scene_bound_member(self):
        source = Square()
        scene = Scene()
        scene.add(source)
        group = AnimationGroup(Transform(source, source.copy().shift((2, 0, 0)), rate_func=lambda a: a))
        group.begin()
        group.interpolate(.5)
        np.testing.assert_allclose(source.get_center(), [1, 0, 0], atol=1e-6)
        group.finish()
        group.clean_up_from_scene(scene)
        np.testing.assert_allclose(source.get_center(), [2, 0, 0], atol=1e-6)

    def test_override_failure_releases_native_state(self):
        class BrokenGroup(AnimationGroup):
            def begin(self):
                super().begin()
                raise ValueError("authored group failure")
        source = Square()
        group = BrokenGroup(Transform(source, source.copy().shift((1, 0, 0))),
                            suspend_mobject_updating=True, run_time=2 / 30)
        scene = Scene()
        scene.add(source)
        with self.assertRaisesRegex(ValueError, "authored group failure"):
            scene.play(group)
        self.assertFalse(group.mobject._is_updating_suspended())
        self.assertFalse(source.is_changing())
        self.assertIsNone(group._composition_driver)
        self.assertIsNone(group._composition_scene)


    def test_explicit_root_is_the_played_root(self):
        source = Square()
        root = VGroup(source)
        group = AnimationGroup(Transform(source, source.copy().shift((2, 0, 0))),
                               group=root, run_time=2 / 30)
        self.assertIs(group.group, root)
        self.assertIs(group.mobject, root)
        scene = Scene()
        scene.play(group)
        self.assertTrue(any(mobject is root for mobject in scene.mobjects))
        np.testing.assert_allclose(source.get_center(), [2, 0, 0], atol=1e-6)

    def test_authored_group_factory_keeps_native_members(self):
        source = Square()
        roots = []
        def factory(*mobjects):
            root = VGroup(*mobjects)
            roots.append(root)
            return root
        group = AnimationGroup(source.animate.shift((2, 0, 0)), group_type=factory,
                               run_time=2 / 30)
        self.assertEqual(len(roots), 1)
        self.assertIs(group.mobject, roots[0])
        Scene().play(group)
        np.testing.assert_allclose(source.get_center(), [2, 0, 0], atol=1e-6)

    def test_authored_timing_table_delays_second_member(self):
        class DelayedGroup(AnimationGroup):
            def build_animations_with_timings(self, lag_ratio):
                self.anims_with_timings = [(member, index * 2., index * 2. + 1.)
                                           for index, member in enumerate(self.animations)]
            def interpolate(self, alpha):
                super().interpolate(alpha)
                self.samples.append((float(alpha), first.get_center().copy(), second.get_center().copy()))
        first, second = Square(), Square().shift((0, 3, 0))
        group = DelayedGroup(
            Transform(first, first.copy().shift((2, 0, 0)), rate_func=lambda a: a),
            Transform(second, second.copy().shift((2, 0, 0)), rate_func=lambda a: a),
            run_time=6 / 30,
        )
        group.samples = []
        Scene().play(group)
        self.assertTrue(any(abs(alpha - .5) < 1e-7
                            and np.allclose(a, [2, 0, 0]) and np.allclose(b, [0, 3, 0])
                            for alpha, a, b in group.samples))
        np.testing.assert_allclose(first.get_center(), [2, 0, 0], atol=1e-6)
        np.testing.assert_allclose(second.get_center(), [2, 3, 0], atol=1e-6)


    def test_following_flash_retains_its_shipped_leaf_lifecycle(self):
        source = Square().shift((2, 1, 0))
        flash = Flash(source, run_time=2 / 30)
        scene = Scene()
        scene.add(source)
        scene.play(flash)
        self.assertFalse(any(mobject is flash.lines for mobject in scene.mobjects))
        np.testing.assert_allclose(source.get_center(), [2, 1, 0], atol=1e-6)


suite = unittest.defaultTestLoader.loadTestsFromTestCase(CompositionLifecycleAcceptance)
result = unittest.TextTestRunner(verbosity=2).run(suite)
if result.testsRun != 9 or not result.wasSuccessful():
    raise AssertionError("Native composition lifecycle acceptance failed")
