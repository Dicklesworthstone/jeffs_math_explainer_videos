"""Fresh-wheel InteractiveScene editing over real native records and workers.

Use the installed interpreter with -I. No source-tree native imports or window
emulation: the same public class receives the same admitted Studio events.
"""
from __future__ import annotations

import json
from pathlib import Path
import runpy
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python.interaction import dispatch_live_input
from fmn_python.studio import Studio

_helpers = runpy.run_path(str(Path(__file__).with_name("studio_preview.py")))
Preview, rgba_png = _helpers["Preview"], _helpers["rgba_png"]


class NativeEditingTests(unittest.TestCase):
    def setUp(self):
        self.s = m.InteractiveScene()
        self.s.setup()
        self.box = m.Square(side_length=2, fill_opacity=1)
        self.s.add(self.box)
        self.s.add_to_selection(self.box)

    def tearDown(self):
        self.s.clear_selection()
        self.s.undo_stack.clear()
        self.s.redo_stack.clear()
        self.s.clear()

    def motion(self, point, modifiers=0):
        return dispatch_live_input(self.s, "on_mouse_motion", (np.array(point, dtype=float), np.zeros(3)), modifiers)

    def key(self, char, modifiers=0, release=False):
        method = "on_key_release" if release else "on_key_press"
        symbol = ord(char) if isinstance(char, str) else char
        return dispatch_live_input(self.s, method, (symbol, modifiers), modifiers)

    def test_axis_grabs_repeat_release_and_native_undo_redo(self):
        self.motion([0, 0, 0])
        self.key("h")
        self.assertEqual(len(self.s.undo_stack), 0, "cancelled gestures must not allocate history")
        self.motion([2, 3, 4])
        np.testing.assert_allclose(self.box.get_center(), [2, 0, 0])
        self.key("h")  # Browser auto-repeat must not rebase the mouse offset.
        self.motion([3, 5, 9])
        np.testing.assert_allclose(self.box.get_center(), [3, 0, 0])
        self.assertEqual(len(self.s.undo_stack), 1)
        self.key("h", release=True)
        self.motion([4, 5, 9])
        np.testing.assert_allclose(self.box.get_center(), [3, 0, 0])
        self.key("z", 2)
        np.testing.assert_allclose(self.box.get_center(), [0, 0, 0])
        self.assertEqual(len(self.s.undo_stack), 0)
        self.assertEqual(len(self.s.redo_stack), 1)
        self.key("z", 3)
        np.testing.assert_allclose(self.box.get_center(), [3, 0, 0])
        self.assertEqual(len(self.s.redo_stack), 0)

    def test_y_and_z_grabs_only_change_the_selected_world_axis(self):
        for char, axis in (("v", 1), ("z", 2)):
            self.motion([0, 0, 0])
            old = self.box.get_center().copy()
            self.key(char)
            self.motion([2, 3, 4])
            target = old.copy()
            target[axis] += [2, 3, 4][axis]
            np.testing.assert_allclose(self.box.get_center(), target)
            self.key(char, release=True)

    def test_uniform_resize_is_cumulative_and_recovers_from_the_pivot(self):
        self.motion([1, 1, 0])
        self.key("t")  # Previously only shift-t initialized the reference.
        for point, size in (([2, 2, 0], 4), ([3, 3, 0], 6), ([1, 1, 0], 2)):
            self.motion(point)
            np.testing.assert_allclose([self.box.get_width(), self.box.get_height()], [size, size])
        self.motion([0, 0, 0])
        self.assertGreater(self.box.get_width(), 0)
        self.motion([1, 1, 0])
        np.testing.assert_allclose(self.box.get_width(), 2, atol=1e-5)
        self.assertEqual(len(self.s.undo_stack), 1)
        self.key("t", release=True)
        self.motion([2, 2, 0])
        np.testing.assert_allclose(self.box.get_width(), 2, atol=1e-5)

    def test_ctrl_stretch_reflection_and_return_to_uniform_are_finite(self):
        self.motion([1, 1, 0])
        self.key("t")
        self.motion([2, 0.5, 0], 2)
        np.testing.assert_allclose([self.box.get_width(), self.box.get_height()], [4, 1])
        original = self.box.get_points().copy()
        self.motion([-2, 0.5, 0], 2)
        np.testing.assert_allclose(self.box.get_points()[:, 0], -original[:, 0])
        self.motion([1, 1, 0], 0)
        np.testing.assert_allclose([self.box.get_width(), self.box.get_height()], [2, 2])
        self.motion([0, 1, 0], 2)
        self.assertTrue(np.isfinite(self.box.get_points()).all())
        self.motion([1, 1, 0], 2)
        np.testing.assert_allclose([self.box.get_width(), self.box.get_height()], [2, 2], atol=1e-5)

    def test_corner_resize_preserves_its_native_pivot(self):
        self.motion([1, 1, 0])
        pivot = self.box.get_corner(m.DL).copy()
        self.key("t", 1)
        self.motion([3, 3, 0], 1)
        np.testing.assert_allclose(self.box.get_corner(m.DL), pivot)
        np.testing.assert_allclose([self.box.get_width(), self.box.get_height()], [4, 4])
        self.key("t", release=True)

    def test_empty_degenerate_and_changed_selection_do_not_corrupt_geometry(self):
        self.s.clear_selection()
        self.key("g")
        self.key("t")
        self.motion([0, 0, 0], 2)
        self.assertEqual(len(self.s.undo_stack), 0)
        self.s.add_to_selection(self.box)
        self.key("t")  # mouse exactly at pivot: undefined radial reference
        self.motion([1, 1, 0])
        np.testing.assert_allclose(self.box.get_width(), 2)
        self.key("t", release=True)
        self.key("g")
        self.s.clear_selection()
        self.motion([2, 2, 0])
        np.testing.assert_allclose(self.box.get_center(), [0, 0, 0])
        self.assertFalse(self.s.is_grabbing)

    def test_consumed_motion_never_applies_the_selection_gesture(self):
        self.motion([0, 0, 0])
        self.key("h")
        seen = []
        self.box.add_mouse_motion_listner(lambda mob, event: seen.append(event["point"].copy()) or False)
        self.motion([0.5, 0, 0])
        self.assertEqual(len(seen), 1)
        np.testing.assert_allclose(self.box.get_center(), [0, 0, 0])
        self.assertEqual(len(self.s.undo_stack), 0)
        self.box.clear_event_listners()

    def test_drag_owned_by_gesture_does_not_redispatch_motion_or_pan_camera(self):
        self.motion([0, 0, 0])
        seen = []
        self.box.add_mouse_motion_listner(lambda mob, event: seen.append("motion"))
        camera = self.s.frame.get_center().copy()
        self.key("h")
        dispatch_live_input(self.s, "on_mouse_drag", (np.array([0.5, 0.5, 0]),
                            np.array([0.5, 0.5, 0]), 1, 0), 0)
        self.assertEqual(seen, [])
        np.testing.assert_allclose(self.box.get_center(), [0.5, 0, 0])
        np.testing.assert_allclose(self.s.frame.get_center(), camera)
        self.box.clear_event_listners()

    def test_removed_selection_cannot_be_moved_by_a_stale_gesture(self):
        self.motion([0, 0, 0])
        self.key("g")
        self.s.remove(self.box)
        self.motion([1, 2, 3])
        np.testing.assert_allclose(self.box.get_center(), [0, 0, 0])
        self.assertFalse(self.s.is_grabbing)
        self.assertEqual(len(self.s.undo_stack), 0)

    def test_resize_admission_refuses_extreme_scale_before_geometry_or_history_changes(self):
        self.motion([1, 1, 0])
        self.key("t")
        points = self.box.get_points().copy()
        with self.assertRaisesRegex(ValueError, "resize scale"):
            self.motion([1e10, 1e10, 0])
        np.testing.assert_array_equal(points, self.box.get_points())
        self.assertEqual(len(self.s.undo_stack), 0)

    def test_fixed_frame_grab_and_resize_use_frame_not_world_coordinates(self):
        self.box.fix_in_frame()
        self.s.frame.shift(m.RIGHT + 2 * m.UP).scale(2).rotate(m.PI / 2)
        to_world = self.s.frame.from_fixed_frame_point
        self.motion(to_world([0, 0, 0]))
        self.key("h")
        self.motion(to_world([1, 2, 0]))
        np.testing.assert_allclose(self.box.get_center(), [1, 0, 0], atol=1e-6)
        self.key("h", release=True)
        self.motion(to_world([2, 1, 0]))
        self.key("t")
        self.motion(to_world([3, 2, 0]))
        np.testing.assert_allclose([self.box.get_width(), self.box.get_height()], [4, 4], atol=1e-5)

    def test_mixed_coordinate_space_selection_is_refused_before_mutation(self):
        fixed = m.Square().shift(3 * m.RIGHT).fix_in_frame()
        self.s.add(fixed)
        self.s.add_to_selection(fixed)
        self.motion([0, 0, 0])
        with self.assertRaisesRegex(ValueError, "world-space and fixed-frame"):
            self.key("g")
        self.assertEqual(len(self.s.undo_stack), 0)
        np.testing.assert_allclose(self.box.get_center(), [0, 0, 0])
        np.testing.assert_allclose(fixed.get_center(), [3, 0, 0])

    def test_rotated_camera_marquee_and_topmost_tap_project_all_bounds_corners(self):
        top = m.Square(side_length=1)
        self.s.add(top)
        self.s.clear_selection()
        self.s.frame.shift(m.RIGHT).scale(1.5).rotate(m.PI / 2)
        self.motion([0, 0, 0])
        self.key("s")
        self.key("s", release=True)
        self.assertEqual(tuple(self.s.selection), (top,))
        self.s.clear_selection()
        center = self.s.frame.to_fixed_frame_point([0, 0, 0])
        self.motion(self.s.frame.from_fixed_frame_point(center - np.array([1.5, 1.5, 0])))
        self.key("s")
        self.motion(self.s.frame.from_fixed_frame_point(center + np.array([1.5, 1.5, 0])))
        self.key("s", release=True)
        self.assertEqual(set(self.s.selection), {self.box, top})
        self.assertEqual(len(self.s.undo_stack), 0, "selection alone is not a geometry edit")

    def test_shift_sweep_is_additive_and_release_does_not_toggle_collected_objects(self):
        second = m.Square(side_length=1).shift(3 * m.RIGHT)
        self.s.add(second)
        self.s.clear_selection()
        self.motion([-3, 0, 0])
        self.key("s", 1)
        self.motion([0, 0, 0], 1)
        self.motion([3, 0, 0], 1)
        self.motion([0, 0, 0], 1)  # revisiting a hit does not deselect it
        self.key("s", 1, release=True)
        self.assertEqual(set(self.s.selection), {self.box, second})
        self.assertFalse(self.s.is_selecting)
        self.assertNotIn(self.s.selection_rectangle, self.s.mobjects)

    def test_fixed_frame_tap_under_moved_camera_selects_the_visible_object(self):
        self.box.fix_in_frame().shift(m.RIGHT)
        self.s.clear_selection()
        self.s.frame.shift(4 * m.LEFT + m.UP).rotate(m.PI / 2).scale(2)
        self.motion(self.s.frame.from_fixed_frame_point([1, 0, 0]))
        self.key("s")
        self.key("s", release=True)
        self.assertEqual(tuple(self.s.selection), (self.box,))

    def test_pointful_scope_excludes_disabled_descendants_and_deduplicates_aliases(self):
        child = m.Square().shift(3 * m.RIGHT)
        group = m.Group(self.box, child)
        self.s.add(group)
        self.s.disable_interaction(child)
        self.s.select_top_level_mobs = False
        self.s.regenerate_selection_search_set()
        candidates = self.s.get_selection_search_set()
        self.assertIn(self.box, candidates)
        self.assertNotIn(child, candidates)
        self.assertEqual(len(candidates), len({id(mob) for mob in candidates}))
        self.s.enable_interaction(child)
        self.assertIn(child, self.s.get_selection_search_set())

    def test_group_ungroup_delete_and_nudge_keep_native_history_and_clear_redo_branch(self):
        other = m.Square().shift(3 * m.RIGHT)
        self.s.add(other)
        self.s.add_to_selection(other)
        self.key("g", 2)
        self.assertEqual(len(self.s.selection), 1)
        group = self.s.selection[0]
        self.assertEqual(set(group), {self.box, other})
        self.key("g", 3)
        self.assertEqual(set(self.s.selection), {self.box, other})
        self.key(0xff08)  # Reference BACKSPACE
        self.assertNotIn(self.box, self.s.mobjects)
        self.key("z", 2)
        self.assertIn(self.box, self.s.mobjects)
        self.assertTrue(self.s.redo_stack)
        self.s.add_to_selection(self.box)
        self.key(0xff53, 1)  # Reference RIGHT, with shift = 10x nudge
        np.testing.assert_allclose(self.box.get_center(), [0.5, 0, 0])
        self.assertFalse(self.s.redo_stack)

    def test_palette_swatch_is_pickable_without_becoming_selectable_or_history_content(self):
        original = self.box.get_color()
        self.s.frame.shift(m.RIGHT).rotate(m.PI / 2).scale(1.5)
        self.key("c")
        self.assertIn(self.s.color_palette, self.s.mobjects)
        self.assertNotIn(self.s.color_palette, self.s.get_selection_search_set())
        self.assertEqual(len(self.s.undo_stack), 0)
        swatch = next(s for s in self.s.color_palette if s.get_color() != original)
        expected = swatch.get_color()
        world = self.s.frame.from_fixed_frame_point(swatch.get_center())
        self.s.choose_color(world)
        self.assertEqual(self.box.get_color(), expected)
        self.assertNotIn(self.s.color_palette, self.s.mobjects)
        self.key("z", 2)
        self.assertEqual(self.box.get_color(), original)
        self.assertNotIn(self.s.color_palette, self.s.mobjects)

    def test_qualified_class_alias_and_keyword_restore_keep_the_public_contract(self):
        from manimlib.scene.interactive_scene import InteractiveScene
        self.assertIs(InteractiveScene, m.InteractiveScene)
        checkpoint = self.s.get_state()
        self.box.shift(2 * m.RIGHT)
        self.s.restore_state(scene_state=checkpoint)
        np.testing.assert_allclose(self.box.get_center(), [0, 0, 0])
        self.assertFalse(self.s.is_grabbing)
        self.assertEqual(len(self.s.selection), 0)


class WorkerEditingTests(unittest.TestCase):
    def test_unedited_interactive_scene_uses_real_modifiers_native_history_and_pixels(self):
        with tempfile.TemporaryDirectory(prefix="fmn edit scene ") as directory:
            root = Path(directory)
            path, facts = root / "edit.py", root / "facts.json"
            path.write_text("from manimlib import *\nimport json\n" + f'''
class Edit(InteractiveScene):
    def construct(self):
        self.box = Square(side_length=2, fill_opacity=1, fill_color=RED, stroke_width=0)
        self.add(self.box)
        self.add_to_selection(self.box)
        self.wait(0.25)
    def on_key_release(self, symbol, modifiers):
        super().on_key_release(symbol, modifiers)
        with open({str(facts)!r}, 'w') as output:
            json.dump(dict(center=self.box.get_center().tolist(), width=self.box.get_width(),
                           height=self.box.get_height(), undo=len(self.undo_stack),
                           redo=len(self.redo_stack)), output)
''')
            with Studio(path, "Edit", interactive=True, resolution=(96, 54), fps=8,
                        threads=1, max_frames=2, max_bytes=4 * 1024 * 1024) as host:
                api = Preview(host)
                history = api.frame()
                api.json("/api/scrub", {"frame": 1})
                initial = api.frame()
                def event(kind, **fields):
                    with api.request("/api/inspect") as response:
                        generation = int(response.headers["X-FMN-Worker-Generation"])
                        view = json.load(response)["view"]
                    result = api.json("/api/event", dict(type=kind, worker_generation=generation,
                        frame=view["frame_index"], revision=view["input_revision"], **fields))
                    api.expected_digest = result["sha256"]
                    return result
                event("mouse_motion", x=0, y=0, dx=0, dy=0, modifiers=0)
                event("key_press", key="h", modifiers=0)
                event("mouse_motion", x=1, y=-2, dx=1, dy=-2, modifiers=0)
                event("key_release", key="h", modifiers=0)
                state = json.loads(facts.read_text())
                np.testing.assert_allclose(state["center"], [1, 0, 0])
                moved = api.frame()
                self.assertNotEqual(initial, moved)
                # Native geometry and separately decoded PNGs must agree.
                width, height, pixels = rgba_png(moved)
                image = np.frombuffer(pixels, np.uint8).reshape(height, width, 4)
                red = (image[:, :, 0] > 150) & (image[:, :, 1] < 100) & (image[:, :, 2] < 100)
                self.assertGreater(np.nonzero(red)[1].mean(), width / 2 + 4)
                event("mouse_motion", x=2, y=-1, dx=1, dy=1, modifiers=0)
                event("key_press", key="t", modifiers=0)
                event("mouse_motion", x=3, y=-0.5, dx=1, dy=0.5, modifiers=2)
                event("key_release", key="t", modifiers=0)
                state = json.loads(facts.read_text())
                np.testing.assert_allclose([state["width"], state["height"]], [4, 1])
                event("key_press", key="z", modifiers=2)
                event("key_release", key="z", modifiers=2)
                state = json.loads(facts.read_text())
                np.testing.assert_allclose([state["width"], state["height"]], [2, 2])
                # Browser Shift-modified letter keys are uppercase, unlike
                # the Reference's unshifted integer key symbols.
                event("key_press", key="Z", modifiers=3)
                event("key_release", key="Z", modifiers=3)
                state = json.loads(facts.read_text())
                np.testing.assert_allclose([state["width"], state["height"]], [4, 1])
                event("key_press", key="a", modifiers=2)
                event("key_release", key="a", modifiers=2)
                event("mouse_motion", x=3, y=-0.5, dx=0, dy=0, modifiers=0)
                event("key_press", key="T", modifiers=1)
                event("mouse_motion", x=7, y=-1.5, dx=4, dy=-1, modifiers=1)
                event("key_release", key="T", modifiers=1)
                state = json.loads(facts.read_text())
                np.testing.assert_allclose([state["width"], state["height"]], [8, 2])
                np.testing.assert_allclose(state["center"], [3, 0.5, 0])
                api.json("/api/scrub", {"frame": 0})
                self.assertEqual(history, api.frame(), "editing rewrote historical captures")
                self.assertTrue(host.alive)


if __name__ == "__main__":
    unittest.main()
