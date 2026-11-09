"""Constructor-dispatch unit tests; the native Arc and tip operations are doubled.

This suite checks the production adapter's cooperative Python call graph, not
geometry, live record storage, or rendering. native_arrow_lifecycle.py covers
those separately against the actual installed/embedded engine.
"""
import importlib.util
import math
from pathlib import Path
import unittest

_SPEC = importlib.util.spec_from_file_location(
    "curved_constructor_adapter",
    Path(__file__).parents[1] / "python/fmn_python/arrow_geometry.py",
)
_ADAPTER = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_ADAPTER)


def namespace():
    class Arc:
        def __init__(self, **kwargs):
            self.events = [("arc", kwargs)]
            self.recipe = kwargs
            self.init_points()
            self.init_colors()
        def init_points(self):
            self.events.append("points")
        def init_colors(self):
            self.events.append("colors")
        def set_points_as_corners(self, points):
            self.events.append(("corners", points))
        def put_start_and_end_on(self, start, end):
            self.events.append(("fit", start, end))
        def add_tip(self, at_start=False):
            self.events.append(("tip", at_start))
    class ArcBetweenPoints(Arc):
        # A detector for accidentally retaining the old constructor route.
        def __init__(self, *args, **kwargs):
            self.events = ["detached native tree"]
    class CurvedArrow(ArcBetweenPoints):
        pass
    class CurvedDoubleArrow(CurvedArrow):
        pass
    g = dict(Arc=Arc, ArcBetweenPoints=ArcBetweenPoints, CurvedArrow=CurvedArrow,
             CurvedDoubleArrow=CurvedDoubleArrow, _math=math, _LEFT=(-1, 0, 0),
             _RIGHT=(1, 0, 0))
    _ADAPTER._install_curved_arrows(g)
    return g


class CurvedConstructorProtocol(unittest.TestCase):
    def test_fit_follows_all_initialization_phases(self):
        g = namespace()
        obj = g["ArcBetweenPoints"]("start", "end", radius=3, arc_center=(4, 5, 6),
                                    start_angle=.4, n_components=9)
        self.assertEqual(obj.events[0], ("arc", dict(angle=math.tau/4, radius=3,
                         arc_center=(4, 5, 6), start_angle=.4, n_components=9)))
        self.assertEqual(obj.events[1:], ["points", "colors", ("fit", "start", "end")])

    def test_double_arrow_keeps_one_arc_and_two_ordered_virtual_tips(self):
        g = namespace()
        obj = g["CurvedDoubleArrow"]("start", "end", color="blue", angle=-.5)
        self.assertEqual(obj.events, [("arc", dict(angle=-.5, color="blue")),
                         "points", "colors", ("fit", "start", "end"),
                         ("tip", False), ("tip", True)])

    def test_zero_angle_replaces_degenerate_arc_before_fitting(self):
        g = namespace()
        obj = g["CurvedArrow"]("a", "b", angle=0)
        self.assertEqual(obj.events[1:], ["points", "colors",
                         ("corners", [g["_LEFT"], g["_RIGHT"]]),
                         ("fit", "a", "b"), ("tip", False)])

    def test_nonzero_angles_do_not_replace_authored_arc(self):
        g = namespace()
        for angle in (-1, 1e-30, 2):
            obj = g["ArcBetweenPoints"](1, 2, angle=angle)
            self.assertFalse(any(isinstance(e, tuple) and e[0] == "corners" for e in obj.events))

    def test_authored_initialization_is_dispatched_through_a_mixin(self):
        g = namespace()
        class Mixin:
            def init_points(self):
                self.events.append("mixin")
                self.assert_recipe = self.recipe["radius"]
                super().init_points()
        class Derived(Mixin, g["CurvedDoubleArrow"]):
            pass
        obj = Derived(1, 2, radius=7)
        self.assertEqual(obj.events[1:4], ["mixin", "points", "colors"])
        self.assertEqual(obj.assert_recipe, 7)
        self.assertEqual(obj.events[-3:], [("fit", 1, 2), ("tip", False), ("tip", True)])

    def test_overridden_fit_and_tip_hooks_are_not_bypassed(self):
        g = namespace()
        class Derived(g["CurvedArrow"]):
            def put_start_and_end_on(self, *ends):
                self.events.append(("authored fit", ends))
            def add_tip(self, **kwargs):
                self.events.append("authored tip")
        obj = Derived(1, 2)
        self.assertEqual(obj.events[-2:], [("authored fit", (1, 2)), "authored tip"])

    def test_late_parent_constructor_patch_stays_cooperative(self):
        g = namespace()
        original = g["Arc"].__init__
        def patched(self, **kwargs):
            original(self, **kwargs)
            self.events.append("patched arc")
        g["Arc"].__init__ = patched
        obj = g["CurvedDoubleArrow"](1, 2)
        self.assertEqual(obj.events[-4:], ["patched arc", ("fit", 1, 2),
                                          ("tip", False), ("tip", True)])

    def test_callback_exception_identity_and_no_later_phase(self):
        g = namespace()
        calls, error = [], RuntimeError("authored failure")
        class Derived(g["CurvedDoubleArrow"]):
            def init_colors(self):
                calls.append("colors")
                raise error
            def put_start_and_end_on(self, *args): calls.append("fit")
            def add_tip(self, **kwargs): calls.append("tip")
        with self.assertRaises(RuntimeError) as caught:
            Derived(1, 2)
        self.assertIs(caught.exception, error)
        self.assertEqual(calls, ["colors"])

    def test_tip_failure_prevents_second_tip(self):
        g = namespace()
        calls, error = [], ValueError("tip failure")
        class Derived(g["CurvedDoubleArrow"]):
            def add_tip(self, at_start=False):
                calls.append(at_start)
                raise error
        with self.assertRaises(ValueError) as caught:
            Derived(1, 2)
        self.assertIs(caught.exception, error)
        self.assertEqual(calls, [False])

    def test_class_identity_and_public_method_metadata(self):
        g = namespace()
        for name in ("ArcBetweenPoints", "CurvedArrow", "CurvedDoubleArrow"):
            cls = g[name]
            self.assertEqual(cls.__init__.__name__, "__init__")
            self.assertEqual(cls.__init__.__qualname__, cls.__qualname__ + ".__init__")
            self.assertEqual(cls.__init__.__module__, cls.__module__)
        self.assertIs(g["CurvedArrow"].__bases__[0], g["ArcBetweenPoints"])
        self.assertIs(g["CurvedDoubleArrow"].__bases__[0], g["CurvedArrow"])


if __name__ == "__main__":
    unittest.main()
