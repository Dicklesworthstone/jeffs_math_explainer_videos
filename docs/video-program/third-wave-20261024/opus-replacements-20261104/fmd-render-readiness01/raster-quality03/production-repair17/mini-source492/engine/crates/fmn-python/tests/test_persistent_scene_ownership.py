"""Persistent ownership admission using production lifecycle and storage doubles.

The shared Animation functions and _Execution are real; this suite checks the
Python/Scene boundary, not native geometry or rendering.
"""
import unittest

from test_animation_updater_protocol import adapter, environment


class PersistentSceneOwnership(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        class ForeignStageError(ValueError):
            pass
        self.native._ForeignStageError = ForeignStageError
        adapter.install_animation_updaters(self.native)
        self.owner, self.foreign = object(), object()
        self.left = self.native.Mobject(-2.)
        self.right = self.native.Mobject(2.)
        self.root = self.native.Mobject(0., self.left, self.right)
        self.animation = self.native.PerMember(self.root)

    def attach(self, cycle=False):
        return self.native.turn_animation_into_updater(self.animation, cycle=cycle)

    def assert_rejected_before_begin(self):
        with self.assertRaisesRegex(self.native._ForeignStageError, "multiple Scenes"):
            self.attach()
        self.assertEqual(self.animation.events, [])
        self.assertFalse(self.root.animating)
        self.assertFalse(hasattr(self.animation, "starting_mobject"))
        self.assertEqual(self.root.updaters, [])
        self.assertEqual((self.left.value, self.right.value), (-2., 2.))

    def test_callback_group_operands_cannot_span_scenes(self):
        self.left._scene, self.right._scene = self.owner, self.foreign
        self.assert_rejected_before_begin()

    def test_root_owner_and_descendant_owner_must_agree(self):
        self.root._scene, self.right._scene = self.owner, self.foreign
        self.assert_rejected_before_begin()

    def test_target_family_is_checked_without_creating_a_target(self):
        self.root._scene = self.owner
        self.animation._target_attr = "target_mobject"
        target_child = self.native.Mobject()
        target_child._scene = self.foreign
        self.animation.target_mobject = self.native.Mobject(0., target_child)
        self.animation.create_target = lambda: self.fail("admission must not create targets")
        self.assert_rejected_before_begin()

    def test_native_extra_operands_are_checked_for_callback_leaves(self):
        self.root._scene = self.owner
        extra = self.native.Mobject()
        extra._scene = self.foreign
        self.animation._native_extra_mobjects = (extra,)
        self.assert_rejected_before_begin()

    def test_detached_callbacks_still_run_without_allocating_a_scene(self):
        self.assertIs(self.attach(), self.root)
        self.root.update(.5).update(0.)
        self.assertEqual((self.left.value, self.right.value), (-1.5, 2.5))
        self.assertIsNone(self.root._scene)
        self.assertEqual(self.animation.total_time, .5)

    def test_single_owner_with_detached_root_preserves_root_identity(self):
        self.left._scene = self.right._scene = self.owner
        self.assertIs(self.attach(), self.root)
        self.assertIsNone(self.root._scene, "callback admission must not adopt or add draw roots")
        self.root.update(1.).update(0.)
        self.assertEqual((self.left.value, self.right.value), (-1., 3.))
        self.assertEqual(self.root.updaters, [])

    def test_same_scene_shared_descendants_are_allowed(self):
        self.root.submobjects.append(self.left)
        self.left._scene = self.right._scene = self.owner
        self.attach()
        self.root.update(.5).update(0.)
        self.assertEqual(self.left.value, -1.5)

    def test_late_split_adoption_aborts_before_advancing_callbacks(self):
        self.attach()
        self.root.update(.25)
        self.left._scene, self.right._scene = self.owner, self.foreign
        events, elapsed = list(self.animation.events), self.animation.total_time
        with self.assertRaisesRegex(self.native._ForeignStageError, "multiple Scenes"):
            self.root.update(.5)
        self.assertEqual(self.animation.events, events)
        self.assertEqual(self.animation.total_time, elapsed)
        self.assertEqual(self.root.updaters, [])
        self.assertFalse(self.root.animating)

    def test_late_single_scene_adoption_remains_live(self):
        self.attach()
        self.left._scene = self.right._scene = self.owner
        self.root.update(.5).update(0.)
        self.assertEqual(self.left.value, -1.5)
        self.assertEqual(self.animation.total_time, .5)

    def test_bound_execution_cannot_silently_switch_to_a_new_scene(self):
        self.root._scene = self.owner
        self.attach()
        self.root._scene = self.foreign
        with self.assertRaisesRegex(self.native._ForeignStageError, "multiple Scenes"):
            self.root.update(.5)
        self.assertEqual(self.animation.total_time, 0.)
        self.assertFalse(self.root.animating)

    def test_foreign_adoption_at_completion_cannot_run_finish(self):
        self.attach()
        self.root.update(1.)
        self.left._scene, self.right._scene = self.owner, self.foreign
        with self.assertRaises(self.native._ForeignStageError):
            self.root.update(0.)
        self.assertNotIn(("finish",), self.animation.events)
        self.assertEqual(self.root.updaters, [])

    def test_cycles_enforce_ownership_without_advancing_their_clock(self):
        self.attach(cycle=True)
        self.root.update(2.25)
        self.left._scene, self.right._scene = self.owner, self.foreign
        with self.assertRaises(self.native._ForeignStageError):
            self.root.update(.5)
        self.assertEqual(self.animation.total_time, 2.25)
        self.assertNotIn(("finish",), self.animation.events)

    def test_admission_failure_preserves_an_unrelated_updater_and_locks(self):
        other = lambda mob, dt: None
        self.root.add_updater(other, call=False)
        locks = self.root.locked_data_keys = {"existing"}
        self.left._scene, self.right._scene = self.owner, self.foreign
        with self.assertRaises(self.native._ForeignStageError):
            self.attach()
        self.assertEqual(self.root.updaters, [other])
        self.assertIs(self.root.locked_data_keys, locks)
        self.assertEqual(locks, {"existing"})

    def test_late_failure_restores_prior_locks_without_rewinding_geometry(self):
        locks = self.root.locked_data_keys = {"existing"}
        self.attach()
        self.root.update(.25).update(0.)
        self.root.locked_data_keys = {"temporary"}
        self.left._scene, self.right._scene = self.owner, self.foreign
        with self.assertRaises(self.native._ForeignStageError):
            self.root.update(.5)
        self.assertEqual(self.left.value, -1.75)
        self.assertIs(self.root.locked_data_keys, locks)
        self.assertEqual(locks, {"existing"})
        self.assertFalse(self.root.animating)

    def test_failed_abort_does_not_mask_the_ownership_failure(self):
        self.attach()
        def abort():
            raise RuntimeError("secondary cleanup failure")
        self.animation.abort = abort
        self.left._scene, self.right._scene = self.owner, self.foreign
        with self.assertRaises(self.native._ForeignStageError) as caught:
            self.root.update(0.)
        self.assertTrue(caught.exception.__notes__)
        self.assertEqual(self.root.updaters, [])
        self.assertFalse(self.root.animating)

    def test_begin_that_changes_ownership_is_unwound_before_registration(self):
        begin = self.animation.begin
        def conflicting_begin():
            begin()
            self.left._scene, self.right._scene = self.owner, self.foreign
        self.animation.begin = conflicting_begin
        with self.assertRaises(self.native._ForeignStageError):
            self.attach()
        self.assertFalse(self.root.animating)
        self.assertEqual(self.root.updaters, [])


class PersistentPhaseOwnership(unittest.TestCase):
    setUp = PersistentSceneOwnership.setUp
    attach = PersistentSceneOwnership.attach

    def split_owners(self):
        self.left._scene, self.right._scene = self.owner, self.foreign

    def test_interpolation_cannot_pass_foreign_operands_to_helper_updates(self):
        self.attach()
        interpolate = self.animation.interpolate
        def change_owner(alpha):
            interpolate(alpha)
            self.split_owners()
        self.animation.interpolate = change_owner
        self.animation.events.clear()
        with self.assertRaises(self.native._ForeignStageError):
            self.root.update(.5)
        self.assertFalse(any(event[0] == "helpers" for event in self.animation.events))
        self.assertEqual(self.animation.total_time, 0.)
        self.assertEqual(self.root.updaters, [])
        self.assertFalse(self.root.animating)

    def test_helper_updates_cannot_commit_a_foreign_owned_tick(self):
        self.attach()
        self.animation.update_mobjects = lambda dt: self.split_owners()
        with self.assertRaises(self.native._ForeignStageError):
            self.root.update(.5)
        self.assertEqual(self.animation.total_time, 0.)
        self.assertEqual(self.root.updaters, [])
        self.assertFalse(self.root.animating)

    def test_duration_hook_cannot_change_ownership_before_interpolation(self):
        self.attach()
        def duration():
            self.split_owners()
            return 1.
        self.animation.get_run_time = duration
        self.animation.events.clear()
        with self.assertRaises(self.native._ForeignStageError):
            self.root.update(.5)
        self.assertEqual(self.animation.events, [])
        self.assertEqual(self.animation.total_time, 0.)

    def test_duration_hook_cannot_change_ownership_before_finish(self):
        self.attach()
        self.root.update(1.)
        def duration():
            self.split_owners()
            return 1.
        self.animation.get_run_time = duration
        self.animation.events.clear()
        with self.assertRaises(self.native._ForeignStageError):
            self.root.update(0.)
        self.assertEqual(self.animation.events, [])
        self.assertEqual(self.root.updaters, [])

    def test_finish_owner_change_restores_transients_without_repeating_finish(self):
        locks = self.root.locked_data_keys = {"existing"}
        self.attach()
        self.root.update(1.)
        finish = self.animation.finish
        def change_owner():
            finish()
            self.root.locked_data_keys = {"temporary"}
            self.split_owners()
        self.animation.finish = change_owner
        with self.assertRaises(self.native._ForeignStageError):
            self.root.update(0.)
        self.root.update(0.)
        self.assertEqual(self.animation.events.count(("finish",)), 1)
        self.assertIs(self.root.locked_data_keys, locks)
        self.assertEqual(locks, {"existing"})
        self.assertEqual(self.root.updaters, [])

    def test_authored_failure_keeps_precedence_over_new_ownership_conflict(self):
        self.attach()
        failure = RuntimeError("authored interpolation failed")
        def fail(alpha):
            self.split_owners()
            raise failure
        self.animation.interpolate = fail
        with self.assertRaises(RuntimeError) as caught:
            self.root.update(.5)
        self.assertIs(caught.exception, failure)
        self.assertEqual(self.animation.total_time, 0.)
        self.assertEqual(self.root.updaters, [])

    def test_callback_cancellation_remains_boundary_safe(self):
        self.attach()
        updater, = self.root.updaters
        def cancel(alpha):
            self.root.remove_updater(updater)
        self.animation.interpolate = cancel
        self.root.update(.5)
        controller = updater._fmn_persistent_controller
        self.assertTrue(controller.closed)
        self.assertEqual(controller.nodes, ())
        self.assertEqual(controller.groups, ())
        self.assertEqual(self.animation.total_time, 0.)
        self.assertFalse(self.root.animating)

    def test_completion_releases_frozen_participant_references(self):
        self.attach()
        controller = self.root.updaters[0]._fmn_persistent_controller
        self.assertEqual(controller.nodes, (self.animation,))
        self.root.update(1.).update(0.)
        self.assertEqual(controller.nodes, ())
        self.assertEqual(controller.groups, ())
        self.assertIsNone(controller.driver)

    def group(self):
        # The existing fixture's group is an authored callback container, not
        # a replacement native timeline. Only ownership traversal is tested.
        self.native.Mobject._is_bound = lambda mob: mob._scene is not None
        self.native._fmn_ensure_composition_root = lambda animation: animation.mobject
        self.root._scene = self.owner
        group = self.native.AnimationGroup(self.root)
        group.animations = [self.animation]
        self.native.turn_animation_into_updater(group)
        return group

    def test_ownership_follows_frozen_children_not_edited_composition_list(self):
        group = self.group()
        group.animations[:] = [group]
        self.root.update(.5)
        self.animation._target_attr = "target_mobject"
        target = self.native.Mobject()
        target._scene = self.foreign
        self.animation.target_mobject = target
        with self.assertRaisesRegex(self.native._ForeignStageError, "multiple Scenes"):
            self.root.update(0.)
        self.assertNotIn("_composition_scene", group.__dict__)
        self.assertEqual(self.root.updaters, [])

    def test_new_unused_composition_child_does_not_change_frozen_execution(self):
        group = self.group()
        other = self.native.Mobject()
        other._scene = self.foreign
        group.animations[:] = [self.native.PerMember(other)]
        self.root.update(.5).update(0.)
        self.assertEqual(group.total_time, .5)
        self.assertEqual(other.value, 0.)
        self.assertNotIn("_composition_scene", group.__dict__)

    def test_each_already_visited_family_is_only_walked_once_per_check(self):
        self.root._scene = self.owner
        self.animation._native_extra_mobjects = (self.left, self.left)
        self.animation._target_attr = "target_mobject"
        self.animation.target_mobject = self.left
        visits = []
        get_family = self.left.get_family
        def observe():
            visits.append(self.left)
            return get_family()
        self.left.get_family = observe
        self.assertIs(adapter._scene_for(vars(self.native), self.animation, self.root), self.owner)
        self.assertEqual(visits, [self.left])

    def test_native_factory_owner_change_is_rejected_before_driver_begin(self):
        self.native.Mobject._is_bound = lambda mob: mob._scene is not None
        self.root._scene = self.owner
        events = []
        class Driver:
            def get_run_time(self):
                return 1.
            def begin(self):
                events.append("begin")
        def factory(scene):
            self.assertIs(scene, self.owner)
            self.split_owners()
            return Driver()
        controller = adapter._PersistentAnimation(
            vars(self.native), self.animation, self.root, False, factory)
        with self.assertRaises(self.native._ForeignStageError):
            controller.start()
        self.assertEqual(events, [])
        self.assertEqual(controller.nodes, ())
        self.assertEqual(self.root.updaters, [])
        self.assertFalse(self.root.animating)


if __name__ == "__main__":
    unittest.main()
