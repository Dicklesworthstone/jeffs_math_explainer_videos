"""Checkpoint lifecycle with explicit native storage doubles, not pixel proof."""
from types import SimpleNamespace
import unittest

import numpy as np
from test_scene_state_protocol import environment, Mob
from fmn_python.interactive_editing import install_interactive_editing


def fixture():
    native = environment()
    class InteractiveScene(native.Scene):
        select_top_level_mobs = True

        def setup(self):
            raise AssertionError("checkpointing must never run authored setup")

        def get_state(self):
            # The bootstrap path whose pre-setup failure the real native test
            # reproduced. Keep it as a negative control before installation.
            return native.SceneState(self, ignore=[self.selection_highlight,
                                                    self.selection_rectangle, self.crosshair])

        def clear_selection(self):
            self.selection = []

        def add(self, *objects):
            super().add(*(obj for obj in objects if obj not in self.roots))

        def bring_to_back(self, *objects):
            self.roots = [*objects, *(obj for obj in self.roots if obj not in objects)]

    native.InteractiveScene = InteractiveScene
    native._PYGLET_MOD_CTRL, native._PYGLET_MOD_COMMAND, native._PYGLET_MOD_SHIFT = 2, 64, 1
    return native


class InteractiveCheckpointProtocol(unittest.TestCase):
    def setUp(self):
        self.native = fixture()
        install_interactive_editing(self.native)
        self.cell = Mob(x=1)
        self.scene = self.native.InteractiveScene(self.cell)

    def test_bootstrap_negative_control_requires_widgets(self):
        native = fixture()
        with self.assertRaises(AttributeError):
            native.InteractiveScene(Mob(x=1)).get_state()

    def test_fresh_snapshot_does_not_create_widgets_or_run_setup(self):
        state = self.scene.get_state()
        self.assertIsNotNone(state._checkpoint)
        self.assertEqual(list(state.mobjects_to_copies), [self.cell])
        for name in ("selection_highlight", "selection_rectangle", "crosshair"):
            self.assertFalse(hasattr(self.scene, name))
        self.assertEqual(self.scene.mobjects, [self.cell])

    def test_restore_before_setup_keeps_native_identity_clock_and_search(self):
        state = self.scene.get_state()
        self.cell.data["point"] += 8
        self.scene._time, self.scene.num_plays = 2, 7
        self.assertIsNone(self.scene.restore_state(state))
        np.testing.assert_array_equal(self.cell.get_points(), [[1, 0, 0]])
        self.assertIs(self.scene.mobjects[0], self.cell)
        self.assertEqual((self.scene.get_time(), self.scene.num_plays), (0, 0))
        self.assertEqual(self.scene.selection_search_set, [self.cell])
        self.assertEqual(self.scene.selection, [])
        self.assertFalse(hasattr(self.scene, "selection_highlight"))

    def test_partial_setup_ignores_only_existing_widgets(self):
        self.scene.crosshair = Mob(x=5)
        self.scene.add(self.scene.crosshair)
        state = self.scene.get_state()
        self.assertEqual(list(state.mobjects_to_copies), [self.cell])
        self.assertFalse(hasattr(self.scene, "selection_highlight"))
        self.scene.restore_state(state)
        self.assertFalse(hasattr(self.scene, "selection_highlight"))
        self.assertEqual(self.scene.selection_search_set, [self.cell])

    def test_live_highlight_reinserted_once_in_front_of_root_order(self):
        self.scene.selection_highlight = Mob()
        self.scene.selection_rectangle, self.scene.crosshair = Mob(), Mob()
        self.scene.unselectables = [self.scene.selection_highlight, self.scene.selection_rectangle,
                                   self.scene.crosshair]
        self.scene.add(self.scene.selection_highlight)
        state = self.scene.get_state()
        self.assertNotIn(self.scene.selection_highlight, state.mobjects_to_copies)
        for _ in range(3):
            self.scene.restore_state(state)
            self.assertIs(self.scene.mobjects[0], self.scene.selection_highlight)
            self.assertEqual(self.scene.mobjects.count(self.scene.selection_highlight), 1)
            self.assertEqual(self.scene.selection_search_set, [self.cell])

    def test_failed_restore_does_not_cancel_gesture_or_clear_selection(self):
        foreign = self.native.InteractiveScene(Mob(x=5)).get_state()
        self.scene._fmn_edit_gesture = gesture = object()
        self.scene.selection = [self.cell]
        self.scene.is_grabbing = True
        with self.assertRaises(ValueError):
            self.scene.restore_state(foreign)
        self.assertIs(self.scene._fmn_edit_gesture, gesture)
        self.assertTrue(self.scene.is_grabbing)
        self.assertEqual(self.scene.selection, [self.cell])

    def test_successful_restore_cancels_stale_gesture(self):
        state = self.scene.get_state()
        self.scene._fmn_edit_gesture = object()
        self.scene._fmn_selection_swept = True
        self.scene.is_grabbing, self.scene.is_selecting = True, True
        self.scene.restore_state(state)
        self.assertNotIn("_fmn_edit_gesture", vars(self.scene))
        self.assertNotIn("_fmn_selection_swept", vars(self.scene))
        self.assertFalse(self.scene.is_grabbing or self.scene.is_selecting)

    def test_undo_redo_work_before_widgets_are_initialized(self):
        self.scene.save_state()
        self.cell.data["point"] += 3
        self.scene._time = 8
        self.scene.undo()
        np.testing.assert_array_equal(self.cell.get_points(), [[1, 0, 0]])
        self.assertEqual(self.scene.get_time(), 0)
        self.scene.redo()
        np.testing.assert_array_equal(self.cell.get_points(), [[4, 3, 3]])
        self.assertEqual(self.scene.get_time(), 8)
        self.assertFalse(hasattr(self.scene, "selection_highlight"))

    def test_existing_widget_identity_survives_capture(self):
        self.scene.selection_highlight = empty_group = Mob()
        self.scene.get_state()
        self.assertIs(self.scene.selection_highlight, empty_group)
        self.assertNotIn(empty_group, self.scene.mobjects)

    def test_installer_is_idempotent_and_preserves_subclass_dispatch(self):
        cls = self.native.InteractiveScene
        original = cls.get_state
        install_interactive_editing(self.native)
        self.assertIs(cls.get_state, original)
        seen = []
        class Custom(cls):
            def get_state(self):
                seen.append(self)
                return super().get_state()
        custom = Custom(self.cell)
        custom.save_state()
        self.assertEqual(seen, [custom])


if __name__ == '__main__':
    unittest.main()
