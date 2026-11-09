"""Real adapter dispatch with doubled native Rectangle operations, not pixels."""
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest

_SPEC = importlib.util.spec_from_file_location(
    "shape_matcher_adapter",
    Path(__file__).parents[1] / "python/fmn_python/shape_matchers.py",
)
_ADAPTER = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_ADAPTER)


def namespace():
    class Mobject:
        def is_fixed_in_frame(self): return False
        def family_members_with_points(self): return [self]
    class Rectangle(Mobject):
        def __init__(self, **kwargs):
            self.events = [("rectangle", kwargs)]
            self.init_points()
            self.init_colors()
        def init_points(self): self.events.append("points")
        def init_colors(self): self.events.append("colors")
        def surround(self, target, buff): self.events.append(("shape", target, buff))
        def fix_in_frame(self): self.events.append("fixed")
    class SurroundingRectangle(Rectangle):
        pass
    class BackgroundRectangle(SurroundingRectangle):
        def set_style(self, **kwargs):
            raise AssertionError("resizing must not invoke the black style override")
    native = SimpleNamespace(_BridgeMobject=Mobject, Rectangle=Rectangle,
                             SurroundingRectangle=SurroundingRectangle,
                             BackgroundRectangle=BackgroundRectangle, _YELLOW="yellow")
    _ADAPTER.install_shape_matchers(native)
    return native


class ShapeMatcherProtocol(unittest.TestCase):
    def test_lifecycle_precedes_virtual_surround(self):
        n = namespace()
        target = n._BridgeMobject()
        obj = n.SurroundingRectangle(target, buff=.2)
        self.assertEqual(obj.events, [("rectangle", {"fill_color":"yellow",
                          "stroke_color":"yellow"}), "points", "colors", ("shape",target,.2)])
        self.assertIs(obj.mobject, target)

    def test_explicit_channel_colors_are_not_overwritten(self):
        n = namespace()
        obj = n.SurroundingRectangle(n._BridgeMobject(), color="blue", stroke_color="red")
        self.assertEqual(obj.events[0][1], {"fill_color":"blue", "stroke_color":"red"})

    def test_rectangle_configuration_reaches_authored_parent(self):
        n = namespace()
        obj = n.SurroundingRectangle(n._BridgeMobject(), width=7, height=3)
        self.assertEqual(obj.events[0][1]["width"], 7)
        self.assertEqual(obj.events[0][1]["height"], 3)

    def test_fixed_in_frame_follows_surround(self):
        n = namespace()
        class Target(n._BridgeMobject):
            def is_fixed_in_frame(self): return True
        obj = n.SurroundingRectangle(Target())
        self.assertEqual(obj.events[-1], "fixed")
        self.assertEqual(obj.events[-2][0], "shape")

    def test_set_buff_respects_a_single_argument_surround_override(self):
        n = namespace()
        class Authored(n.SurroundingRectangle):
            def surround(self, mobject):
                self.events.append("authored surround")
                return super().surround(mobject)
        target = n._BridgeMobject()
        obj = Authored(target)
        self.assertIs(obj.set_buff(.75), obj)
        self.assertEqual(obj.events[-2:], ["authored surround", ("shape",target,.75)])
        self.assertEqual(obj.events.count("points"), 1)

    def test_background_resizing_never_calls_its_style_override(self):
        n = namespace()
        target = n._BridgeMobject()
        obj = n.BackgroundRectangle(target, fill_color="green")
        self.assertIs(obj.surround(target, .4), obj)
        self.assertIs(obj.set_buff(.5), obj)
        self.assertEqual(obj.events[0][1]["fill_color"], "green")
        self.assertEqual(obj.events[-1], ("shape", target, .5))

    def test_authored_points_and_colors_are_dispatched_once(self):
        n = namespace()
        class Authored(n.SurroundingRectangle):
            def init_points(self):
                self.events.append("authored points")
            def init_colors(self):
                self.events.append("authored colors")
        obj = Authored(n._BridgeMobject())
        obj.set_buff(.3)
        self.assertEqual(obj.events[1:3], ["authored points", "authored colors"])
        self.assertEqual(obj.events.count("authored points"), 1)

    def test_invalid_target_is_refused_before_construction_or_mutation(self):
        n = namespace()
        with self.assertRaises(TypeError): n.SurroundingRectangle(object())
        target = n._BridgeMobject()
        obj = n.SurroundingRectangle(target)
        events = list(obj.events)
        with self.assertRaises(TypeError): obj.surround(object(), .5)
        self.assertEqual(obj.events, events)
        self.assertIs(obj.mobject, target)
        self.assertEqual(obj.buff, .1)

    def test_failure_propagates_without_later_surround(self):
        n = namespace()
        calls, error = [], RuntimeError("authored init failure")
        class Authored(n.SurroundingRectangle):
            def init_points(self):
                calls.append("points")
                raise error
            def surround(self, *args): calls.append("surround")
        with self.assertRaises(RuntimeError) as caught: Authored(n._BridgeMobject())
        self.assertIs(caught.exception, error)
        self.assertEqual(calls, ["points"])

    def test_later_rectangle_surround_patch_is_not_captured_away(self):
        n = namespace()
        target = n._BridgeMobject()
        obj = n.SurroundingRectangle(target)
        def patched(self, target, buff): self.events.append(("patched",target,buff))
        n.Rectangle.surround = patched
        obj.set_buff(.4)
        self.assertEqual(obj.events[-1], ("patched",target,.4))

    def test_nonfinite_padding_refuses_before_mutating_a_live_matcher(self):
        n = namespace()
        target = n._BridgeMobject()
        obj = n.SurroundingRectangle(target)
        before = list(obj.events)
        for value in (float("nan"), float("inf"), -float("inf")):
            with self.subTest(value=value):
                with self.assertRaises(ValueError): obj.set_buff(value)
                with self.assertRaises(ValueError): obj.surround(n._BridgeMobject(), value)
                self.assertEqual(obj.events, before)
                self.assertIs(obj.mobject, target)
                self.assertEqual(obj.buff, .1)

    def test_nonfinite_constructor_padding_refuses_before_authored_hooks(self):
        n = namespace()
        calls = []
        class Authored(n.SurroundingRectangle):
            def init_points(self): calls.append("points")
        with self.assertRaises(ValueError): Authored(n._BridgeMobject(), buff=float("nan"))
        self.assertEqual(calls, [])

    def test_installation_is_idempotent_and_keeps_class_identity(self):
        n = namespace()
        cls, init = n.SurroundingRectangle, n.SurroundingRectangle.__init__
        _ADAPTER.install_shape_matchers(n)
        self.assertIs(n.SurroundingRectangle, cls)
        self.assertIs(n.SurroundingRectangle.__init__, init)
        self.assertIs(n.BackgroundRectangle.__bases__[0], cls)
        self.assertEqual(cls.surround.__qualname__, cls.__qualname__ + ".surround")


if __name__ == "__main__":
    unittest.main()
