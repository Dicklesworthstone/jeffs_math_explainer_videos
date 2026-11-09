"""Composite panel delegation; native layout and input acceptance is separate."""
import inspect
import importlib.util
import sys
from types import ModuleType
import unittest
from unittest.mock import patch

from test_color_sliders_protocol import environment, module


class ColorPanelTests(unittest.TestCase):
    def setUp(self):
        self.n = environment()
        Group, Mob = self.n.Group, self.n.Mobject
        Mob.add = lambda self, *children: (self.submobjects.extend(children), self)[1]
        calls = self.calls = []
        class ScalarControl(Mob):
            pass
        class Panel(Group):
            def __init__(self, *controls, panel_kwargs={"width": 3.},
                         opener_kwargs={"height": .5},
                         opener_text_kwargs={"text": "Control Panel", "font_size": 20}, **kwargs):
                if not all(isinstance(c, ScalarControl) for c in controls):
                    raise TypeError("old scalar-only panel")
                self.legacy_calls = 1
                self.panel_kwargs, self.opener_kwargs = dict(panel_kwargs), dict(opener_kwargs)
                self.opener_text_kwargs = dict(opener_text_kwargs)
                self.panel, self.panel_opener, self.controls = self._native_control_panel_parts(
                    "Control Panel", 20., controls, open=False,
                )
                super().__init__(self.panel, self.panel_opener, self.controls)
            def _native_control_panel_parts(self, text, size, controls, *, open):
                calls.append((text, size, tuple(controls), open))
                return Mob(), Group(Mob(), Mob()), Group(*controls)
            def move_panel_and_controls_to_panel_opener(self):
                self.relayouts = getattr(self, "relayouts", 0) + 1
            def add_controls(self, *controls):
                if not all(isinstance(c, ScalarControl) for c in controls):
                    raise TypeError("old scalar-only add")
                self.controls.add(*controls)
                self.move_panel_and_controls_to_panel_opener()
        self.original = Panel.__init__
        self.n.ControlPanel, self.n.ControlMobject = Panel, ScalarControl
        # The dependency is an explicitly doubled registrar. It verifies the
        # adapter delegates to the same control event binder; it is not an
        # alternative dispatcher or a native event receipt.
        package = ModuleType("color_panel_test_dependencies")
        package.__path__ = []
        events = ModuleType("color_panel_test_dependencies.control_events")
        def connect(g, owner, specifications):
            owner.connected = specifications
        events._connect = connect
        context = patch.dict(sys.modules, {
            package.__name__: package, events.__name__: events,
        })
        context.start()
        self.addCleanup(context.stop)
        test_spec = importlib.util.spec_from_file_location(package.__name__ + ".color_sliders", module.__file__)
        context = patch.object(module, "__spec__", test_spec)
        context.start()
        self.addCleanup(context.stop)
        context = patch.object(module, "__package__", package.__name__)
        context.start()
        self.addCleanup(context.stop)
        module._install_panel_content(vars(self.n))
    def test_bare_shapes_and_noncontrol_groups_keep_existing_refusal(self):
        panel = self.n.ControlPanel(self.n.ControlMobject())
        prior = list(panel.controls.submobjects)
        for shape in (self.n.Mobject(), self.n.Group(), self.n.Group(self.n.Mobject())):
            with self.subTest(shape=type(shape).__name__):
                with self.assertRaisesRegex(TypeError, "^ControlPanel controls must be ControlMobject instances$"):
                    self.n.ControlPanel(shape)
                with self.assertRaisesRegex(TypeError, "^ControlPanel controls must be ControlMobject instances$"):
                    panel.add_controls(shape)
        self.assertEqual(panel.controls.submobjects, prior)
        self.assertFalse(hasattr(panel, "relayouts"))

    def test_previous_constructor_rejects_color_bank(self):
        bank = self.n.ColorSliders()
        instance = self.n.ControlPanel.__new__(self.n.ControlPanel)
        with self.assertRaisesRegex(TypeError, "scalar-only"):
            self.original(instance, bank)
    def test_color_bank_and_grouped_controls_use_native_extent_path(self):
        bank, caption = self.n.ColorSliders(), self.n.Group(self.n.ControlMobject(), self.n.Mobject())
        panel = self.n.ControlPanel(bank, caption)
        self.assertEqual(self.calls[-1], ("Control Panel", 20., (bank, caption), False))
        self.assertEqual(panel.controls.submobjects, [bank, caption])
        self.assertIs(panel.panel_opener_rect, panel.panel_opener.submobjects[0])
        self.assertIs(panel.panel_info_text, panel.panel_opener.submobjects[1])
        self.assertTrue(panel.fixed and bank.fixed and caption.fixed)
        self.assertEqual([row[1] for row in panel.connected], ["MouseScrollEvent", "MouseDragEvent"])
    def test_scalar_constructor_keeps_original_path(self):
        control = self.n.ControlMobject()
        panel = self.n.ControlPanel(control)
        self.assertEqual(panel.legacy_calls, 1)
        self.assertEqual(len(self.calls), 1)
    def test_defaults_and_config_forwarding_without_aliasing_inputs(self):
        bank = self.n.ColorSliders()
        config = {"width": 4., "fill_color": "blue"}
        text = {"text": "Channels", "font_size": 18., "color": "white"}
        panel = self.n.ControlPanel(bank, panel_kwargs=config, opener_text_kwargs=text)
        self.assertEqual(self.calls[-1][:2], ("Channels", 18.))
        self.assertEqual(panel.panel_kwargs, config)
        self.assertIsNot(panel.panel_kwargs, config)
        self.assertEqual(panel.opener_kwargs, {"height": .5})
        self.assertEqual(str(inspect.signature(self.n.ControlPanel.__init__)), str(inspect.signature(self.original)))
    def test_invalid_options_and_content_fail_before_native_layout(self):
        bank = self.n.ColorSliders()
        for kwargs in ({"panel_kwargs": {"shader": "x"}},
                       {"opener_kwargs": {"unknown": 0}},
                       {"opener_text_kwargs": {"font": "external"}},
                       {"unsupported": True}):
            with self.assertRaises(TypeError):
                self.n.ControlPanel(bank, **kwargs)
        with self.assertRaises(TypeError):
            self.n.ControlPanel(bank, object())
        self.assertEqual(self.calls, [])
    def test_add_composites_preserves_existing_controls_and_root(self):
        scalar = self.n.ControlMobject()
        panel = self.n.ControlPanel(scalar)
        root = panel.controls
        bank = self.n.ColorSliders()
        caption = self.n.Group(self.n.ControlMobject(), self.n.Mobject())
        self.assertIsNone(panel.add_controls(bank, caption))
        self.assertIs(panel.controls, root)
        self.assertEqual(root.submobjects, [scalar, bank, caption])
        self.assertEqual(panel.relayouts, 1)
        bank.set_value(1., 2., 3., .4)
        self.assertEqual(root.submobjects[1].get_value()[0], 1/255.)
    def test_invalid_late_content_does_not_partially_attach(self):
        panel = self.n.ControlPanel(self.n.ControlMobject())
        prior = list(panel.controls.submobjects)
        with self.assertRaises(TypeError):
            panel.add_controls(self.n.ColorSliders(), None)
        self.assertEqual(panel.controls.submobjects, prior)
        self.assertFalse(hasattr(panel, "relayouts"))
    def test_authored_native_layout_and_relayout_hooks_remain_virtual(self):
        Base = self.n.ControlPanel
        class Authored(Base):
            def _native_control_panel_parts(self, *args, **kwargs):
                self.saw_layout = True
                return super()._native_control_panel_parts(*args, **kwargs)
            def move_panel_and_controls_to_panel_opener(self):
                self.saw_relayout = True
                return super().move_panel_and_controls_to_panel_opener()
        bank = self.n.ColorSliders()
        panel = Authored(bank)
        panel.add_controls(self.n.Group(self.n.ControlMobject()))
        self.assertTrue(panel.saw_layout and panel.saw_relayout)


if __name__ == "__main__":
    unittest.main()
