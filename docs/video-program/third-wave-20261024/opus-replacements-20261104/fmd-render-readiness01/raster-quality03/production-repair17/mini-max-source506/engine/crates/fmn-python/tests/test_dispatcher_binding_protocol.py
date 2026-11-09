"""Install input adapters against the exact, qualified-only Reference export."""
import sys
from types import ModuleType
import unittest
from unittest.mock import patch

from fmn_python.interaction import install_interaction
from test_interaction_protocol import environment


class DispatcherInstallationTests(unittest.TestCase):
    def test_qualified_only_dispatcher_keeps_identity_exports_and_single_delivery(self):
        g, dispatcher = environment(install=False)
        original = g.pop("EventDispatcher")
        qualified = ModuleType("manimlib.event_handler.event_dispatcher")
        qualified.EventDispatcher = original

        class Module:
            @property
            def __dict__(self):
                return g

        with patch.dict(sys.modules, {qualified.__name__: qualified}):
            install_interaction(Module())
            dispatch = original.dispatch
            install_interaction(Module())
            self.assertIs(original.dispatch, dispatch)
            self.assertIs(type(dispatcher), original)
            self.assertIs(qualified.EventDispatcher, original)
            self.assertNotIn("EventDispatcher", g)
            obj, seen = g["Mobject"](), []
            obj.add_event_listner(g["EventType"].MousePressEvent,
                                 lambda obj, event: seen.append(event["point"]))
            scene = g["Scene"]().add(obj)
            scene.on_mouse_press([0, 0, 0], 1, 0)
            self.assertEqual(len(seen), 1)
            independent = qualified.EventDispatcher()
            independent.dispatch(g["EventType"].KeyPressEvent, symbol=7, modifiers=0)
            self.assertTrue(independent.is_key_pressed(7))
            self.assertFalse(dispatcher.is_key_pressed(7))

    def test_missing_dispatcher_fails_before_patching_scene_or_marking_installed(self):
        g, _ = environment(install=False)
        g.pop("EventDispatcher")
        before = g["Scene"].on_mouse_press

        class Module:
            @property
            def __dict__(self):
                return g

        with patch.dict(sys.modules, {"manimlib.event_handler.event_dispatcher": None}):
            with self.assertRaisesRegex(ImportError, "canonical module"):
                install_interaction(Module())
        self.assertNotIn("_FMN_INTERACTION_INSTALLED", g)
        self.assertIs(g["Scene"].on_mouse_press, before)


if __name__ == "__main__":
    unittest.main()
