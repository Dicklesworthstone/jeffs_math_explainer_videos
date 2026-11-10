"""Exercise the production input adapter; geometry is explicitly doubled here."""
import unittest
from fmn_python.interaction import (
    dispatch_live_input, input_key_pressed, input_modifiers, _LIVE_EVENT,
)
from test_interaction_protocol import environment


class LiveInputContextTests(unittest.TestCase):
    def setUp(self):
        self.g, self.dispatcher = environment()
        self.g["Scene"].get_window = lambda s: getattr(s, "window", None)
        self.scene = self.g["Scene"]()

    def test_motion_and_scroll_keep_wire_modifiers_without_extra_method_arguments(self):
        seen = []
        scene = self.scene
        motion = scene.on_mouse_motion
        scroll = scene.on_mouse_scroll
        def on_motion(point, delta):
            seen.append(input_modifiers(scene))
            motion(point, delta)
        def on_scroll(point, offset, x, y):
            seen.append(input_modifiers(scene))
            scroll(point, offset, x, y)
        scene.on_mouse_motion, scene.on_mouse_scroll = on_motion, on_scroll
        dispatch_live_input(scene, "on_mouse_motion", ([3, 2, 1], [0, 0, 0]), 3)
        dispatch_live_input(scene, "on_mouse_scroll", ([4, 2, 1], [0, 1], 0, 1), 64)
        dispatch_live_input(scene, "on_mouse_motion", ([5, 2, 1], [0, 0, 0]), 0)
        self.assertEqual(seen, [3, 64, 0])
        self.assertEqual(input_modifiers(scene), 0)
        self.assertEqual(scene.mouse_point.point.tolist(), [5, 2, 1])
        self.assertIsNone(_LIVE_EVENT.get())

    def test_nested_other_scene_and_exception_restore_outer_modifier_owner(self):
        inner = self.g["Scene"]()
        seen = []
        def nested(point, delta):
            seen.append((input_modifiers(self.scene), input_modifiers(inner)))
            raise ValueError("authored failure")
        inner.on_mouse_motion = nested
        def outer(point, delta):
            seen.append((input_modifiers(self.scene), input_modifiers(inner)))
            with self.assertRaisesRegex(ValueError, "authored"):
                dispatch_live_input(inner, "on_mouse_motion", (point, delta), 4)
            seen.append((input_modifiers(self.scene), input_modifiers(inner)))
        self.scene.on_mouse_motion = outer
        dispatch_live_input(self.scene, "on_mouse_motion", ([0, 0, 0], [0, 0, 0]), 1)
        self.assertEqual(seen, [(1, 0), (0, 4), (1, 0)])
        self.assertIsNone(_LIVE_EVENT.get())
        self.assertEqual(input_modifiers(inner), 0)

    def test_key_state_is_scene_scoped_and_release_clears_it(self):
        other = self.g["Scene"]()
        dispatch_live_input(self.scene, "on_key_press", (ord("h"), 0), 0)
        self.assertTrue(input_key_pressed(self.scene, ord("h")))
        self.assertFalse(input_key_pressed(other, ord("h")))
        self.assertFalse(self.dispatcher.is_key_pressed(ord("h")))
        dispatch_live_input(self.scene, "on_key_release", (ord("h"), 0), 0)
        self.assertFalse(input_key_pressed(self.scene, ord("h")))

    def test_real_window_precedes_cached_dispatcher_state(self):
        from types import SimpleNamespace
        keys = {0xffe3, ord("v")}
        self.scene.window = SimpleNamespace(is_key_pressed=lambda k: k in keys)
        dispatch_live_input(self.scene, "on_key_press", (ord("h"), 1), 1)
        self.assertEqual(input_modifiers(self.scene), 2)
        self.assertFalse(input_key_pressed(self.scene, ord("h")))
        self.assertTrue(input_key_pressed(self.scene, ord("v")))

    def test_press_without_prior_hover_updates_scene_cursor(self):
        self.scene.on_mouse_press([4, 3, 2], 1, 0)
        self.assertEqual(self.scene.mouse_point.point.tolist(), [4, 3, 2])
        self.assertEqual(self.scene.mouse_drag_point.point.tolist(), [4, 3, 2])

    def test_refused_dispatch_never_calls_arbitrary_methods(self):
        for modifiers in (True, -1, 8, 128, None):
            with self.assertRaises(ValueError):
                dispatch_live_input(self.scene, "on_key_press", (ord("g"), 0), modifiers)
        with self.assertRaises(ValueError):
            dispatch_live_input(self.scene, "construct", (), 0)
        self.assertEqual(self.scene.events, [])
        self.assertIsNone(_LIVE_EVENT.get())

    def test_authored_override_can_dispatch_directly_before_calling_a_scene_gateway(self):
        seen = []
        def override(symbol, modifiers):
            seen.append(input_modifiers(self.scene))
            self.dispatcher.dispatch(self.g["EventType"].KeyPressEvent,
                                     symbol=symbol, modifiers=modifiers)
        self.scene.on_key_press = override
        dispatch_live_input(self.scene, "on_key_press", (ord("h"), 2), 2)
        self.assertEqual(seen, [2])
        self.assertTrue(self.dispatcher.is_key_pressed(ord("h")))
        self.assertFalse(input_key_pressed(self.scene, ord("h")))
        self.assertIsNone(_LIVE_EVENT.get())


if __name__ == "__main__":
    unittest.main()
