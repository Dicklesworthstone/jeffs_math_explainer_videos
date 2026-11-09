"""Real control binding code over explicit geometry/value/event fixtures."""
from __future__ import annotations

import unittest
import numpy as np
from test_interaction_protocol import environment as input_environment
from fmn_python.control_events import install_control_events, _connect
from fmn_python.interaction import install_interaction


def environment():
    g, dispatcher = input_environment(False)
    Mob, Event = g["Mobject"], g["EventType"]
    old_move = Mob.move_to
    def move(self, point):
        value = point.point if isinstance(point, Mob) else np.asarray(point)
        delta = value - self.point
        for child in self.submobjects:
            child.move_to(child.point + delta)
        return old_move(self, value)
    Mob.move_to = move
    Mob.get_center = lambda self: self.point.copy()
    Mob.get_width = lambda self: getattr(self, "width", 1.)
    Mob.get_height = lambda self: getattr(self, "height", 1.)
    Mob.stretch_to_fit_width = lambda self, value: (setattr(self, "width", value), self)[1]
    Mob.stretch_to_fit_height = lambda self, value: (setattr(self, "height", value), self)[1]
    def scale(self, value):
        self.width, self.height = self.get_width() * value, self.get_height() * value
        return self
    Mob.scale = scale
    def become(self, other):
        self.move_to(other)
        self.width, self.height = other.get_width(), other.get_height()
        return self
    Mob.become = become
    Mob.set_fill = lambda self, color: (setattr(self, "fill", color), self)[1]
    Mob.set_stroke = lambda self, color: (setattr(self, "stroke", color), self)[1]
    def fixed(self):
        for member in self.get_family():
            member.fixed = True
        return self
    Mob.fix_in_frame = fixed
    def register(kind):
        return lambda self, callback: self.add_event_listner(kind, callback)
    for name, kind in (("add_mouse_press_listner", Event.MousePressEvent),
                       ("add_mouse_drag_listner", Event.MouseDragEvent),
                       ("add_mouse_scroll_listner", Event.MouseScrollEvent),
                       ("add_key_press_listner", Event.KeyPressEvent)):
        setattr(Mob, name, register(kind))

    class MotionMobject(Mob):
        def __init__(self, mobject):
            super().__init__(mobject)
            self.mobject = mobject
        def mob_on_mouse_drag(self, mob, data):
            mob.move_to(data["point"])
            return False
    class Button(Mob):
        def __init__(self, mobject, on_click):
            super().__init__(mobject)
            self.mobject, self.on_click = mobject, on_click
        def mob_on_mouse_press(self, mob, data):
            self.on_click(mob)
            return False
    class Control(Mob):
        def __init__(self, value, *children):
            super().__init__(*children)
            self.value = value
        def get_value(self):
            return self.value
        def set_value(self, value):
            self.set_value_anim(value)
            self.value = value
            return self
        def toggle_value(self):
            self.set_value(not self.value)
        def on_mouse_press(self, mob, data):
            mob.toggle_value()
            return False
    class Checkbox(Control):
        def __init__(self, value=True):
            self.box, self.box_content = Mob(), Mob()
            super().__init__(value, self.box, self.box_content)
        def get_checkmark(self):
            return Mob()
        def get_cross(self):
            return Mob()
        def set_value_anim(self, value):
            self.box_content.become(self.get_checkmark() if value else self.get_cross())
    class EnableDisableButton(Control):
        def __init__(self, value=True):
            self.box = Mob()
            self.enable_color, self.disable_color = "green", "red"
            super().__init__(value, self.box)
        def set_value_anim(self, value):
            self.box.become(Mob())
    class LinearNumberSlider(Control):
        def __init__(self, value=0., min_value=-10., max_value=10., step=1., **kwargs):
            self.bar, self.slider, self.slider_axis = Mob(), Mob(), Mob()
            self.min_value, self.max_value, self.step = min_value, max_value, step
            super().__init__(value, self.bar, self.slider, self.slider_axis)
        def set_value_anim(self, value):
            self.slider.move_to([value, 0, 0])
        def get_value_from_point(self, point):
            return min(self.max_value, max(self.min_value, point[0]))
        def slider_on_mouse_drag(self, mob, data):
            self.set_value(self.get_value_from_point(data["point"]))
            return False
    class Textbox(Control):
        def __init__(self, value="", isInitiallyActive=False):
            self.isActive, self.active_color, self.deactive_color = isInitiallyActive, "blue", "red"
            self.box, self.text = self._native_textbox_parts(value, None)
            super().__init__(value, self.box, self.text)
        def _native_textbox_parts(self, current, replacement):
            text = Mob()
            text.value = current if replacement is None else replacement
            return [Mob(), text]
        def active_anim(self, flag):
            self.box.become(Mob())
        def box_on_mouse_press(self, mob, data):
            self.isActive = not self.isActive
            self.active_anim(self.isActive)
            return False
        def set_value(self, value):
            parts = self._native_textbox_parts(self.value, value)
            self.text.become(parts[1])
            self.value = value
            return self
        def on_key_press(self, mob, data):
            if self.isActive:
                self.set_value(self.value + chr(data["symbol"]))
                return False
    class ControlPanel(Mob):
        def __init__(self, *controls):
            self.panel, self.panel_opener, self.controls = Mob(), Mob(Mob(), Mob()), Mob(*controls)
            super().__init__(self.panel, self.panel_opener, self.controls)
            self.events = []
        def panel_on_mouse_scroll(self, mob, data):
            self.events.append(("scroll", data["offset"]))
            return False
        def panel_opener_on_mouse_drag(self, mob, data):
            self.events.append(("drag", data["point"]))
            return False
    for cls in (MotionMobject, Button, Checkbox, EnableDisableButton, LinearNumberSlider, Textbox, ControlPanel):
        g[cls.__name__] = cls
    class Module:
        @property
        def __dict__(self):
            return g
    install_interaction(Module())
    install_control_events(Module())
    return g, dispatcher


class ControlTests(unittest.TestCase):
    def setUp(self):
        self.g, self.dispatcher = environment()
        self.Scene, self.Mob, self.Event = self.g["Scene"], self.g["Mobject"], self.g["EventType"]
    def test_button_click_reaches_authored_callback(self):
        seen, shape = [], self.Mob().move_to([3, 0, 0])
        button = self.g["Button"](shape, seen.append)
        self.Scene().add(button).on_mouse_press([3, 0, 0], 1, 0)
        self.assertEqual(seen, [shape])
    def test_motion_wrapper_drag_moves_original_object_not_camera(self):
        shape = self.Mob()
        wrapper = self.g["MotionMobject"](shape)
        scene = self.Scene().add(wrapper)
        scene.on_mouse_press([0, 0, 0], 1, 0)
        scene.on_mouse_drag([4, 0, 0], [4, 0, 0], 1, 0)
        np.testing.assert_equal(shape.point, [4, 0, 0])
        self.assertEqual(scene.events, [])
    def test_checkbox_click_changes_value_and_mark_at_live_box(self):
        box = self.g["Checkbox"](True)
        box.move_to([3, 1, 0]); box.box.width, box.box.height = 4., 2.
        self.Scene().add(box).on_mouse_press([3, 1, 0], 1, 0)
        self.assertIs(box.get_value(), False)
        np.testing.assert_equal(box.box_content.point, [3, 1, 0])
        self.assertEqual((box.box_content.width, box.box_content.height), (2., 1.))
    def test_toggle_colors_without_replacing_live_box_geometry(self):
        button = self.g["EnableDisableButton"](True)
        button.move_to([3, 0, 0]); button.box.width = 7.
        self.Scene().add(button).on_mouse_press([3, 0, 0], 1, 0)
        self.assertIs(button.get_value(), False)
        self.assertEqual(button.box.fill, "red")
        self.assertEqual(button.box.width, 7.)
        np.testing.assert_equal(button.box.point, [3, 0, 0])
    def test_slider_drag_runs_value_setter_and_clamps(self):
        slider = self.g["LinearNumberSlider"]()
        scene = self.Scene().add(slider)
        scene.on_mouse_press([0, 0, 0], 1, 0)
        scene.on_mouse_drag([99, 0, 0], [99, 0, 0], 1, 0)
        self.assertEqual(slider.get_value(), 10.)
        self.assertEqual(scene.events, [])
    def test_textbox_focus_typing_and_backspace_use_registered_handlers(self):
        box = self.g["Textbox"]("hi")
        box.move_to([3, 0, 0])
        scene = self.Scene().add(box)
        scene.on_mouse_press([3, 0, 0], 1, 0)
        self.assertTrue(box.isActive)
        scene.on_key_press(ord("a"), 1)
        self.assertEqual(box.get_value(), "hiA")
        scene.on_key_press(0xFF08, 0)
        self.assertEqual(box.get_value(), "hi")
        np.testing.assert_equal(box.text.point, [3, 0, 0])
        np.testing.assert_equal(box.box.point, [3, 0, 0])
        self.assertEqual(scene.events, [])
    def test_inactive_textbox_does_not_consume_shortcuts(self):
        box = self.g["Textbox"]("x")
        scene = self.Scene().add(box)
        scene.on_key_press(ord("a"), 0)
        self.assertEqual(box.get_value(), "x")
        self.assertEqual(scene.events, ["key-default"])
    def test_navigation_and_control_keysyms_are_not_inserted_as_letters(self):
        box = self.g["Textbox"]("x", isInitiallyActive=True)
        scene = self.Scene().add(box)
        for symbol, modifier in ((0xFF51, 0), (ord("c"), 2), (0xFFE1, 0), (ord("v"), 64), (-1, 0)):
            scene.on_key_press(symbol, modifier)
        self.assertEqual(box.get_value(), "x")
        self.assertEqual(scene.events, [])
    def test_panel_registers_drag_and_scroll_and_preserves_aliases(self):
        panel = self.g["ControlPanel"]()
        self.assertIs(panel.panel_opener_rect, panel.panel_opener.submobjects[0])
        self.assertIs(panel.panel_info_text, panel.panel_opener.submobjects[1])
        self.assertTrue(panel.fixed)
        scene = self.Scene().add(panel)
        scene.on_mouse_scroll([0, 0, 0], [0, 1], 0, 1)
        scene.on_mouse_press([0, 0, 0], 1, 0)
        scene.on_mouse_drag([0, 2, 0], [0, 2, 0], 1, 0)
        self.assertEqual([entry[0] for entry in panel.events], ["scroll", "drag"])
        self.assertEqual(scene.events, [])
    def test_authored_handler_override_is_registered(self):
        Base = self.g["Button"]
        class Authored(Base):
            def mob_on_mouse_press(self, mob, data):
                self.seen = mob
                return False
        shape = self.Mob()
        button = Authored(shape, lambda m: self.fail("stock handler invoked"))
        self.Scene().add(button).on_mouse_press([0, 0, 0], 1, 0)
        self.assertIs(button.seen, shape)
    def test_duplicate_connection_does_not_register_twice(self):
        button = self.g["Button"](self.Mob(), lambda m: None)
        _connect(self.g, button, (("mobject", "MousePressEvent", "add_mouse_press_listner", "mob_on_mouse_press"),))
        self.assertEqual(len(button.mobject.event_listners), 1)
    def test_failed_later_registration_unwinds_earlier_listener(self):
        box = self.g["Textbox"]()
        # Remove its existing bindings to exercise a two-registration attempt.
        for target in (box, box.box):
            for listener in tuple(target.event_listners):
                target.remove_event_listner(listener.event_type, listener.callback)
        def fail(callback):
            raise LookupError("registrar refused")
        box.add_key_press_listner = fail
        with self.assertRaisesRegex(LookupError, "registrar refused"):
            _connect(self.g, box, (
                ("box", "MousePressEvent", "add_mouse_press_listner", "box_on_mouse_press"),
                ("", "KeyPressEvent", "add_key_press_listner", "on_key_press"),
            ))
        self.assertEqual(box.box.event_listners, [])
        self.assertFalse(any(self.dispatcher.event_listners.values()))
    def test_noncallable_button_action_is_rejected_without_listener(self):
        shape = self.Mob()
        with self.assertRaises(TypeError):
            self.g["Button"](shape, None)
        self.assertEqual(shape.event_listners, [])
    def test_removed_controls_and_other_scene_are_inactive(self):
        first, second = self.g["Checkbox"](), self.g["Checkbox"]()
        one, two = self.Scene().add(first), self.Scene().add(second)
        one.remove(first)
        one.on_mouse_press([0, 0, 0], 1, 0)
        self.assertTrue(first.get_value())
        self.assertTrue(second.get_value())
        two.on_mouse_press([0, 0, 0], 1, 0)
        self.assertFalse(second.get_value())


if __name__ == "__main__":
    unittest.main()
