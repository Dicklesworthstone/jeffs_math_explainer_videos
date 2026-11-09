"""Exercise installed history behavior against the snapshot protocol backend."""
import unittest

from test_scene_state_protocol import Mob, environment


class SceneHistoryProtocol(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        self.mob = Mob(x=1.)
        self.scene = self.native.Scene(self.mob)

    def x(self): return float(self.mob.data["point"][0, 0])
    def move(self, value): self.mob.data["point"][0, 0] = value

    def test_duplicate_saves_are_not_added(self):
        self.assertIs(self.scene.save_state(), self.scene)
        self.scene.save_state()
        self.assertEqual(len(self.scene.undo_stack), 1)

    def test_new_branch_discards_redo(self):
        self.scene.save_state()
        self.move(2.)
        self.scene.undo()
        self.assertEqual(self.x(), 1.)
        self.assertEqual(len(self.scene.redo_stack), 1)
        self.move(3.)
        self.scene.save_state()
        self.assertEqual(self.scene.redo_stack, [])
        self.scene.redo()
        self.assertEqual(self.x(), 3.)

    def test_multiple_undo_redo_roundtrips(self):
        for value in (1., 2., 3.):
            self.move(value)
            self.scene.save_state()
        self.move(4.)
        for value in (3., 2., 1.):
            self.scene.undo()
            self.assertEqual(self.x(), value)
        for value in (2., 3., 4.):
            self.scene.redo()
            self.assertEqual(self.x(), value)

    def test_failed_undo_does_not_move_either_history_stack(self):
        self.scene.save_state()
        self.move(2.)
        undo, redo = tuple(self.scene.undo_stack), tuple(self.scene.redo_stack)
        self.scene.refuse_restore = True
        with self.assertRaisesRegex(RuntimeError, "backend rejected"):
            self.scene.undo()
        self.assertEqual(tuple(self.scene.undo_stack), undo)
        self.assertEqual(tuple(self.scene.redo_stack), redo)
        self.assertEqual(self.x(), 2.)

    def test_failed_redo_does_not_move_either_history_stack(self):
        self.scene.save_state()
        self.move(2.)
        self.scene.undo()
        undo, redo = tuple(self.scene.undo_stack), tuple(self.scene.redo_stack)
        self.scene.refuse_restore = True
        with self.assertRaisesRegex(RuntimeError, "backend rejected"):
            self.scene.redo()
        self.assertEqual(tuple(self.scene.undo_stack), undo)
        self.assertEqual(tuple(self.scene.redo_stack), redo)
        self.assertEqual(self.x(), 1.)

    def test_changed_history_limit_bounds_both_stacks(self):
        for value in range(5):
            self.move(float(value))
            self.scene.save_state()
        self.assertEqual(len(self.scene.undo_stack), 3)
        self.scene.max_num_saved_states = 1
        self.scene.undo()
        self.scene.undo()
        self.assertLessEqual(len(self.scene.redo_stack), 1)
        self.scene.redo()
        self.assertLessEqual(len(self.scene.undo_stack), 1)

    def test_zero_limit_disables_snapshot_allocation(self):
        self.scene.save_state()
        self.scene.max_num_saved_states = 0
        def forbidden(): raise AssertionError("must not allocate a disabled snapshot")
        self.scene.get_state = forbidden
        self.assertIs(self.scene.save_state(), self.scene)
        self.assertEqual(self.scene.undo_stack, [])
        self.assertEqual(self.scene.redo_stack, [])

    def test_invalid_limit_rejected_before_state_capture(self):
        for value, error in ((-1, ValueError), (.5, TypeError), (True, TypeError)):
            self.scene.max_num_saved_states = value
            with self.assertRaises(error):
                self.scene.save_state()
            self.assertEqual(self.scene.undo_stack, [])

    def test_camera_only_change_is_saved_and_reversible(self):
        self.scene.save_state()
        frame, core = self.scene.frame, self.scene.frame._core
        core.set_center((2., 3., 4.))
        self.scene.save_state()
        self.assertEqual(len(self.scene.undo_stack), 2)
        core.set_center((8., 9., 10.))
        self.scene.undo()
        self.assertEqual(core.center(), (2., 3., 4.))
        self.scene.undo()
        self.assertEqual(core.center(), (0., 0., 0.))
        self.assertIs(frame._core, core)

    def test_authored_get_and_restore_hooks_are_used(self):
        scene_class = self.native.Scene
        events = []
        class Authored(scene_class):
            def get_state(self):
                events.append("get")
                return super().get_state()
            def restore_state(self, state):
                events.append("restore")
                return super().restore_state(state)
        scene = Authored(self.mob)
        scene.save_state()
        self.move(2.)
        scene.undo()
        scene.redo()
        self.assertEqual(events, ["get", "get", "restore", "get", "restore"])

    def test_empty_history_is_noop(self):
        self.assertIsNone(self.scene.undo())
        self.assertIsNone(self.scene.redo())
        self.assertEqual(self.scene.restore_calls, 0)


if __name__ == "__main__":
    unittest.main()
