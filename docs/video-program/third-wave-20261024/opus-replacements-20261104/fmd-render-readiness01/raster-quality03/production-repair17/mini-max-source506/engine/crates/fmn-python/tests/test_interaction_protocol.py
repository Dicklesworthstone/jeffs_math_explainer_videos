"""Input orchestration against explicit scene/geometry storage doubles.

The production adapter is imported unchanged. Native geometry, MRO bootstrap
and rendered interaction have separate installed-wheel acceptance tests.
"""
from __future__ import annotations

from enum import Enum
from pathlib import Path
import sys
import unittest
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python.interaction import install_interaction, _INPUT


def environment(install=True):
    class EventType(Enum):
        MouseMotionEvent = "mouse_motion_event"
        MouseDragEvent = "mouse_drag_event"
        MousePressEvent = "mouse_press_event"
        MouseReleaseEvent = "mouse_release_event"
        MouseScrollEvent = "mouse_scroll_event"
        KeyPressEvent = "key_press_event"
        KeyReleaseEvent = "key_release_event"

    class Mobject:
        def __init__(self, *children):
            self.submobjects = list(children)
            self.point = np.zeros(3)
            self.event_listners = []
            self.fixed = False
        def get_family(self):
            return [self, *(member for child in self.submobjects for member in child.get_family())]
        def move_to(self, point):
            self.point = np.asarray(point, dtype=float).copy()
            return self
        def is_point_touching(self, point):
            return np.linalg.norm(np.asarray(point) - self.point) < 0.5
        def is_fixed_in_frame(self):
            return self.fixed
        def add_event_listner(self, kind, callback):
            listener = EventListener(self, kind, callback)
            self.event_listners.append(listener)
            dispatcher.add_listner(listener)
            return self
        def remove_event_listner(self, kind, callback):
            for listener in tuple(self.event_listners):
                if listener.event_type == kind and listener.callback == callback:
                    self.event_listners.remove(listener)
                    dispatcher.remove_listner(listener)
            return self
        def get_event_listners(self):
            return self.event_listners

    class EventListener:
        def __init__(self, mobject, event_type, callback):
            self.mobject, self.event_type, self.callback = mobject, event_type, callback

    class EventDispatcher:
        def __init__(self):
            self.event_listners = {kind: [] for kind in EventType}
            self.mouse_point = np.zeros(3)
            self.mouse_drag_point = np.zeros(3)
            self.pressed_keys = set()
            self.draggable_object_listners = []
        def add_listner(self, listener):
            self.event_listners[listener.event_type].append(listener)
            return self
        def remove_listner(self, listener):
            bucket = self.event_listners[listener.event_type]
            bucket[:] = [item for item in bucket if item is not listener]
        def dispatch(self, kind, **data):
            # Pinned standalone dispatch semantics, over doubled hit testing.
            if kind == EventType.MouseMotionEvent:
                self.mouse_point = data["point"]
            elif kind == EventType.MouseDragEvent:
                self.mouse_drag_point = data["point"]
            elif kind == EventType.KeyPressEvent:
                self.pressed_keys.add(data["symbol"])
            elif kind == EventType.KeyReleaseEvent:
                self.pressed_keys.discard(data["symbol"])
            elif kind == EventType.MousePressEvent:
                self.draggable_object_listners = [
                    listener for listener in self.event_listners[EventType.MouseDragEvent]
                    if listener.mobject.is_point_touching(self.mouse_point)
                ]
            elif kind == EventType.MouseReleaseEvent:
                self.draggable_object_listners = []
            captured = kind == EventType.MouseDragEvent
            listeners = self.draggable_object_listners if captured else self.event_listners[kind]
            result = None
            for listener in listeners:
                if not captured and kind.value.startswith("mouse") and not listener.mobject.is_point_touching(self.mouse_point):
                    continue
                result = listener.callback(listener.mobject, data)
                if result is False:
                    return False
            return result
        __call__ = dispatch
        def get_mouse_point(self):
            return self.mouse_point
        def get_mouse_drag_point(self):
            return self.mouse_drag_point
        def is_key_pressed(self, symbol):
            return symbol in self.pressed_keys

    dispatcher = EventDispatcher()
    g = dict(EventType=EventType, EventListener=EventListener,
             EventDispatcher=EventDispatcher, Mobject=Mobject, _np=np)
    g["_event_dispatcher"] = lambda: dispatcher
    exec('''
def _scene_event_stopped(kind, **data):
    return _event_dispatcher().dispatch(kind, **data) is False

class Scene:
    def __init__(self):
        self.mobjects = []
        self.mouse_point = Mobject()
        self.mouse_drag_point = Mobject()
        self.events = []
        class Frame:
            def to_fixed_frame_point(self, point, relative=False):
                return _np.asarray(point).copy()
        self.frame = Frame()
    def add(self, *objects):
        self.mobjects.extend(objects)
        return self
    def remove(self, obj):
        self.mobjects[:] = [member for member in self.mobjects if member is not obj]
    def on_mouse_motion(self, point, d_point):
        self.mouse_point.move_to(point)
        if _scene_event_stopped(EventType.MouseMotionEvent, point=point, d_point=d_point):
            return
        self.events.append("motion-default")
    def on_mouse_press(self, point, button, mods):
        self.mouse_drag_point.move_to(point)
        _scene_event_stopped(EventType.MousePressEvent, point=point, button=button, mods=mods)
    def on_mouse_drag(self, point, d_point, buttons, modifiers):
        self.mouse_drag_point.move_to(point)
        self.events.append("camera-pan")
        if _scene_event_stopped(EventType.MouseDragEvent, point=point, d_point=d_point,
                                buttons=buttons, modifiers=modifiers):
            return
    def on_mouse_release(self, point, button, mods):
        _scene_event_stopped(EventType.MouseReleaseEvent, point=point, button=button, mods=mods)
    def on_mouse_scroll(self, point, offset, x_pixel_offset, y_pixel_offset):
        if _scene_event_stopped(EventType.MouseScrollEvent, point=point, offset=offset):
            return
        self.events.append("camera-zoom")
    def on_key_press(self, symbol, modifiers):
        if _scene_event_stopped(EventType.KeyPressEvent, symbol=symbol, modifiers=modifiers):
            return
        self.events.append("key-default")
    def on_key_release(self, symbol, modifiers):
        _scene_event_stopped(EventType.KeyReleaseEvent, symbol=symbol, modifiers=modifiers)

class InteractiveScene(Scene):
    def on_key_press(self, symbol, modifiers):
        super().on_key_press(symbol, modifiers)
        self.events.append("selection-default")
    def on_mouse_release(self, point, button, mods):
        super().on_mouse_release(point, button, mods)
        self.events.append("clear-selection")
''', g)
    class Module:
        @property
        def __dict__(self):
            return g
    if install:
        install_interaction(Module())
    return g, dispatcher


class InputTests(unittest.TestCase):
    def setUp(self):
        self.g, self.dispatcher = environment()
        self.Event = self.g["EventType"]
        self.Mob, self.Scene = self.g["Mobject"], self.g["Scene"]

    def listen(self, obj, kind, callback):
        obj.add_event_listner(kind, callback)
        return obj.event_listners[-1]

    def test_scene_press_uses_its_own_point(self):
        obj, seen = self.Mob().move_to([4, 0, 0]), []
        self.listen(obj, self.Event.MousePressEvent, lambda m, e: seen.append(e["point"]))
        self.Scene().add(obj).on_mouse_press([4, 0, 0], 1, 0)
        self.assertEqual(len(seen), 1)
        np.testing.assert_equal(seen[0], [4, 0, 0])

    def test_standalone_hover_capture_and_pointer_identity_are_preserved(self):
        obj, seen = self.Mob(), []
        self.listen(obj, self.Event.MouseDragEvent, lambda m, e: seen.append(m))
        hover, point = np.zeros(3), np.array([4., 0, 0])
        self.dispatcher(self.Event.MouseMotionEvent, point=hover)
        self.dispatcher(self.Event.MousePressEvent, point=point)
        self.dispatcher(self.Event.MouseDragEvent, point=point)
        self.assertEqual(seen, [obj])
        self.assertIs(self.dispatcher.get_mouse_point(), hover)
        self.assertIs(self.dispatcher.get_mouse_drag_point(), point)

    def test_explicit_detached_global_interceptor_remains_supported(self):
        obj = self.Mob()
        listener = self.g["EventListener"](obj, self.Event.MouseScrollEvent, lambda m, e: False)
        self.dispatcher.add_listner(listener)
        scene = self.Scene()
        self.assertIs(scene.on_mouse_scroll([0, 0, 0], [0, 1], 0, 1), False)
        self.assertEqual(scene.events, [])

    def test_old_stale_hover_control_reproduces_missing_click(self):
        g, dispatcher = environment(False)
        mob, seen = g["Mobject"]().move_to([4, 0, 0]), []
        mob.add_event_listner(g["EventType"].MousePressEvent, lambda m, e: seen.append(1))
        dispatcher.dispatch(g["EventType"].MousePressEvent, point=[4, 0, 0])
        self.assertEqual(seen, [])

    def test_scene_input_ignores_other_scenes_and_detached_objects(self):
        a, b, c, seen = self.Mob(), self.Mob(), self.Mob(), []
        for name, mob in zip("abc", (a, b, c)):
            self.listen(mob, self.Event.MousePressEvent, lambda m, e, name=name: seen.append(name))
        self.Scene().add(a).on_mouse_press([0, 0, 0], 1, 0)
        self.assertEqual(seen, ["a"])
        self.Scene().add(b).on_mouse_press([0, 0, 0], 1, 0)
        self.assertEqual(seen, ["a", "b"])

    def test_descendant_control_is_admitted(self):
        child, seen = self.Mob(), []
        self.listen(child, self.Event.MousePressEvent, lambda m, e: seen.append(m))
        self.Scene().add(self.Mob(child)).on_mouse_press([0, 0, 0], 1, 0)
        self.assertEqual(seen, [child])

    def test_consumed_drag_never_pans_camera(self):
        mob = self.Mob()
        self.listen(mob, self.Event.MouseDragEvent, lambda m, e: (m.move_to(e["point"]), False)[1])
        scene = self.Scene().add(mob)
        scene.on_mouse_press([0, 0, 0], 1, 0)
        self.assertIs(scene.on_mouse_drag([8, 0, 0], [8, 0, 0], 1, 0), False)
        self.assertNotIn("camera-pan", scene.events)
        np.testing.assert_equal(mob.point, [8, 0, 0])

    def test_consumed_key_stops_interactive_default_and_dispatches_once(self):
        obj, seen = self.Mob(), []
        self.listen(obj, self.Event.KeyPressEvent, lambda m, e: (seen.append(1), False)[1])
        scene = self.g["InteractiveScene"]().add(obj)
        self.assertIs(scene.on_key_press(65, 0), False)
        self.assertEqual(seen, [1])
        self.assertEqual(scene.events, [])

    def test_unconsumed_super_chain_dispatches_once(self):
        obj, seen = self.Mob(), []
        self.listen(obj, self.Event.KeyPressEvent, lambda m, e: seen.append(1))
        scene = self.g["InteractiveScene"]().add(obj)
        scene.on_key_press(65, 0)
        self.assertEqual(seen, [1])
        self.assertEqual(scene.events, ["key-default", "selection-default"])

    def test_release_clears_capture_and_can_stop_selection_cleanup(self):
        obj, seen = self.Mob(), []
        self.listen(obj, self.Event.MouseDragEvent, lambda m, e: seen.append(1))
        self.listen(obj, self.Event.MouseReleaseEvent, lambda m, e: False)
        scene = self.g["InteractiveScene"]().add(obj)
        scene.on_mouse_press([0, 0, 0], 1, 0)
        scene.on_mouse_release([0, 0, 0], 1, 0)
        self.assertNotIn("clear-selection", scene.events)
        scene.on_mouse_drag([0, 0, 0], [1, 0, 0], 1, 0)
        self.assertEqual(seen, [])

    def test_drag_capture_isolated_per_scene(self):
        a, b, seen = self.Mob(), self.Mob(), []
        self.listen(a, self.Event.MouseDragEvent, lambda m, e: seen.append("a"))
        self.listen(b, self.Event.MouseDragEvent, lambda m, e: seen.append("b"))
        first, second = self.Scene().add(a), self.Scene().add(b)
        first.on_mouse_press([0, 0, 0], 1, 0)
        second.on_mouse_drag([0, 0, 0], [1, 0, 0], 1, 0)
        self.assertEqual(seen, [])
        second.on_mouse_press([0, 0, 0], 1, 0)
        first.on_mouse_drag([3, 0, 0], [3, 0, 0], 1, 0)
        second.on_mouse_drag([3, 0, 0], [3, 0, 0], 1, 0)
        self.assertEqual(seen, ["a", "b"])

    def test_removed_scene_members_do_not_receive_captured_drag(self):
        obj, seen = self.Mob(), []
        self.listen(obj, self.Event.MouseDragEvent, lambda m, e: seen.append(1))
        scene = self.Scene().add(obj)
        scene.on_mouse_press([0, 0, 0], 1, 0)
        scene.remove(obj)
        scene.on_mouse_drag([0, 0, 0], [1, 0, 0], 1, 0)
        self.assertEqual(seen, [])

    def test_removing_later_listener_takes_effect_during_delivery(self):
        first, second, seen = self.Mob(), self.Mob(), []
        later = self.listen(second, self.Event.KeyPressEvent, lambda m, e: seen.append(2))
        early = self.listen(first, self.Event.KeyPressEvent,
                            lambda m, e: self.dispatcher.remove_listner(later))
        self.dispatcher.event_listners[self.Event.KeyPressEvent][:] = [early, later]
        self.Scene().add(first, second).on_key_press(1, 0)
        self.assertEqual(seen, [])

    def test_added_listener_waits_until_next_event(self):
        obj, seen = self.Mob(), []
        def callback(m, e):
            self.listen(m, self.Event.KeyPressEvent, lambda m, e: seen.append(2))
        self.listen(obj, self.Event.KeyPressEvent, callback)
        scene = self.Scene().add(obj)
        scene.on_key_press(1, 0)
        self.assertEqual(seen, [])
        scene.on_key_press(1, 0)
        self.assertEqual(seen, [2])

    def test_key_state_isolated_and_visible_inside_callbacks(self):
        a, b, seen = self.Mob(), self.Mob(), []
        for obj in (a, b):
            self.listen(obj, self.Event.KeyPressEvent,
                        lambda m, e: seen.append(self.dispatcher.is_key_pressed(65)))
        one, two = self.Scene().add(a), self.Scene().add(b)
        one.on_key_press(65, 0)
        two.on_key_press(66, 0)
        self.assertEqual(seen, [True, False])
        self.assertFalse(self.dispatcher.is_key_pressed(65))
        one.on_key_release(65, 0)
        one.on_key_press(66, 0)
        self.assertEqual(seen[-1], False)

    def test_nested_other_scene_restores_outer_context(self):
        a, b, seen = self.Mob(), self.Mob(), []
        one, two = self.Scene().add(a), self.Scene().add(b)
        self.listen(b, self.Event.KeyPressEvent, lambda m, e: seen.append("inner"))
        def outer(m, e):
            two.on_key_press(66, 0)
            seen.append(self.dispatcher.is_key_pressed(65))
            return False
        self.listen(a, self.Event.KeyPressEvent, outer)
        one.on_key_press(65, 0)
        self.assertEqual(seen, ["inner", True])
        self.assertIsNone(_INPUT.get())

    def test_nested_same_scene_event_is_not_suppressed_during_dispatch(self):
        obj, seen = self.Mob(), []
        scene = self.Scene().add(obj)
        def callback(m, e):
            seen.append(e["symbol"])
            if e["symbol"] == 1:
                scene.on_key_press(2, 0)
        self.listen(obj, self.Event.KeyPressEvent, callback)
        scene.on_key_press(1, 0)
        self.assertEqual(seen, [1, 2])

    def test_exception_clears_capture_and_context(self):
        obj = self.Mob()
        def fail(m, e):
            raise LookupError("authored failure")
        listener = self.listen(obj, self.Event.MouseDragEvent, fail)
        scene = self.Scene().add(obj)
        scene.on_mouse_press([0, 0, 0], 1, 0)
        with self.assertRaisesRegex(LookupError, "authored failure"):
            scene.on_mouse_drag([0, 0, 0], [1, 0, 0], 1, 0)
        self.assertIsNone(_INPUT.get())
        seen = []
        listener.callback = lambda m, e: seen.append(1)
        scene.on_mouse_drag([0, 0, 0], [1, 0, 0], 1, 0)
        self.assertEqual(seen, [])

    def test_invalid_mouse_point_precedes_scene_mutation(self):
        scene = self.Scene()
        for point in ([float("nan"), 0, 0], [1, 2], [1, 2, float("inf")]):
            with self.assertRaises(ValueError):
                scene.on_mouse_drag(point, [0, 0, 0], 1, 0)
        np.testing.assert_equal(scene.mouse_drag_point.point, [0, 0, 0])
        self.assertEqual(scene.events, [])

    def test_scroll_is_hit_tested_at_current_point(self):
        obj = self.Mob().move_to([4, 0, 0])
        self.listen(obj, self.Event.MouseScrollEvent, lambda m, e: False)
        scene = self.Scene().add(obj)
        scene.on_mouse_scroll([4, 0, 0], [0, 1], 0, 1)
        self.assertEqual(scene.events, [])

    def test_custom_scene_super_and_keywords_remain_valid(self):
        Base = self.Scene
        class Authored(Base):
            def on_key_press(self, symbol, modifiers):
                self.events.append("authored")
                return super().on_key_press(symbol, modifiers)
        obj, seen = self.Mob(), []
        self.listen(obj, self.Event.KeyPressEvent, lambda m, e: (seen.append(1), False)[1])
        scene = Authored().add(obj)
        self.assertIs(scene.on_key_press(symbol=1, modifiers=0), False)
        self.assertEqual(scene.events, ["authored"])
        self.assertEqual(seen, [1])

    def test_fixed_controls_use_camera_projection_for_hit_and_delivery(self):
        obj, seen = self.Mob(), []
        obj.fixed = True
        self.listen(obj, self.Event.MouseDragEvent, lambda m, e: (seen.append(e), False)[1])
        scene = self.Scene().add(obj)
        class Frame:
            def to_fixed_frame_point(self, point, relative=False):
                return np.asarray(point) / 2 if relative else (np.asarray(point) - [10, 0, 0]) / 2
        scene.frame = Frame()
        scene.on_mouse_press([10, 0, 0], 1, 0)
        original = np.array([14., 0, 0])
        scene.on_mouse_drag(original, [4, 0, 0], 1, 0)
        self.assertEqual(len(seen), 1)
        np.testing.assert_equal(seen[0]["point"], [2, 0, 0])
        np.testing.assert_equal(seen[0]["d_point"], [2, 0, 0])
        np.testing.assert_equal(original, [14, 0, 0])
        self.assertEqual(scene.events, [])

    def test_world_controls_are_not_camera_projected(self):
        obj, seen = self.Mob().move_to([10, 0, 0]), []
        self.listen(obj, self.Event.MousePressEvent, lambda m, e: seen.append(e["point"]))
        scene = self.Scene().add(obj)
        class Frame:
            def to_fixed_frame_point(self, *args, **kwargs):
                raise AssertionError("world control projected")
        scene.frame = Frame()
        scene.on_mouse_press([10, 0, 0], 1, 0)
        self.assertEqual(seen, [[10, 0, 0]])

    def test_nested_input_does_not_retarget_outer_delivery(self):
        a, b, seen = self.Mob(), self.Mob(), []
        scene = self.Scene().add(a, b)
        self.listen(a, self.Event.MousePressEvent,
                    lambda m, e: scene.on_mouse_motion([9, 0, 0], [9, 0, 0]))
        self.listen(b, self.Event.MousePressEvent, lambda m, e: seen.append("outer"))
        scene.on_mouse_press([0, 0, 0], 1, 0)
        self.assertEqual(seen, ["outer"])

    def test_reinstall_keeps_classes_and_wrappers_identical(self):
        method = self.Scene.on_key_press
        class Module:
            @property
            def __dict__(self):
                return g
        g = self.g
        install_interaction(Module())
        self.assertIs(self.Scene.on_key_press, method)
        self.assertIs(g["Scene"], self.Scene)


if __name__ == "__main__":
    unittest.main()
