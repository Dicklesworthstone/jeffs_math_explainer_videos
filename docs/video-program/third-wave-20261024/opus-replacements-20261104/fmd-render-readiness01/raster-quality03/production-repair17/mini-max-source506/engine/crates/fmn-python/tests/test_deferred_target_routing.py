"""Engine-free routing tests; native geometry is covered by fmn-anim tests.

These exercise the production installer, not a replacement interpolation or
frame loop. The namespace records lowering decisions without executing a scene.
"""
from pathlib import Path
import sys
import types
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python.indication import install_indication


def namespace():
    native = types.SimpleNamespace()
    g = vars(native)

    def linear(alpha):
        return alpha

    class Mobject:
        def __init__(self, *children):
            self.submobjects = list(children)
            self.updaters = []

        def copy(self):
            raise AssertionError("routing must not copy geometry")

        def scale(self, factor):
            raise AssertionError("routing must not scale geometry")

    class Animation:
        def __init__(self, mobject=None, **kwargs):
            self.mobject = mobject if mobject is not None else Mobject()
            self.rate_func = None
            self.final_alpha_value = 1.0
            self.remover = False
            self.path_arc = 0.0
            self.__dict__.update(kwargs)

        def begin(self):
            raise AssertionError("routing must not begin an animation")

    class Transform(Animation):
        def create_target(self):
            raise AssertionError("routing must not run target factories")

        def create_starting_mobject(self):
            raise AssertionError("routing must not run starting factories")

    class GrowFromPoint(Transform):
        pass

    class GrowFromCenter(GrowFromPoint):
        pass

    class GrowFromEdge(GrowFromPoint):
        pass

    class GrowArrow(GrowFromPoint):
        pass

    class Indicate(Transform):
        pass

    class TurnInsideOut(Transform):
        pass

    class AnimationGroup(Animation):
        def __init__(self, *animations):
            super().__init__()
            self.animations = list(animations)

    def original_requires(animation):
        if isinstance(animation, AnimationGroup):
            return any(g["_requires_python_animation"](a) for a in animation.animations)
        return bool(getattr(animation, "previous_requires", False))

    class Scene:
        def play(self, *animations, **kwargs):
            decisions = [g["_requires_python_animation"](a) for a in animations]
            if kwargs.get("fail"):
                raise RuntimeError("original play failure")
            return decisions, kwargs, [a.rate_func for a in animations]

    g.update({name: value for name, value in locals().items() if isinstance(value, type)})
    g.update(_RATE_FUNC_NAMES={linear: "linear"}, _requires_python_animation=original_requires)
    install_indication(native)
    return native


class DeferredTargetRoutingTests(unittest.TestCase):
    def setUp(self):
        self.n = namespace()
        self.requires = self.n._requires_python_animation

    def test_stock_growth_and_indication_remain_native(self):
        for name in ("GrowFromPoint", "GrowFromCenter", "GrowFromEdge", "GrowArrow",
                     "Indicate", "TurnInsideOut"):
            for rate in (None, "linear", next(iter(self.n._RATE_FUNC_NAMES))):
                with self.subTest(name=name, rate=rate):
                    effect = getattr(self.n, name)(rate_func=rate)
                    self.assertFalse(self.requires(effect))
                    self.assertFalse(hasattr(effect, "target_mobject"))

    def test_replay_products_and_host_updaters_do_not_force_callbacks(self):
        effect = self.n.GrowFromCenter()
        calls = []
        updater = lambda *args: calls.append(args)
        effect.mobject.updaters.append(updater)
        effect.target_mobject = self.n.Mobject()
        effect.starting_mobject = self.n.Mobject()
        self.assertFalse(self.requires(effect))
        self.assertEqual(calls, [])
        self.assertIs(effect.mobject.updaters[0], updater)

    def test_authored_animation_factories_and_instance_hooks_use_callbacks(self):
        class Custom(self.n.GrowFromPoint):
            def create_target(self):
                raise AssertionError("must be deferred until begin")
        self.assertTrue(self.requires(Custom()))
        effect = self.n.Indicate()
        effect.create_starting_mobject = lambda: None
        self.assertTrue(self.requires(effect))

    def test_changed_shared_base_hook_is_not_bypassed(self):
        self.n.Animation.begin = lambda self: None
        self.assertTrue(self.requires(self.n.GrowFromCenter()))

    def test_authored_geometry_hooks_in_shared_children_use_callbacks(self):
        class Custom(self.n.Mobject):
            def copy(self):
                raise AssertionError("no speculative factory execution")
        child = Custom()
        root = self.n.Mobject(self.n.Mobject(child), self.n.Mobject(child))
        self.assertTrue(self.requires(self.n.Indicate(root)))
        self.assertFalse(self.requires(self.n.Indicate(self.n.Mobject(self.n.Mobject()))))

    def test_custom_unhashable_rates_are_not_probed(self):
        class Curve:
            __hash__ = None
            def __call__(self, alpha):
                raise AssertionError("no sampled rate table during routing")
        self.assertTrue(self.requires(self.n.GrowFromPoint(rate_func=Curve())))

    def test_partial_endpoints_removers_and_indicate_arcs_use_callbacks(self):
        for name in ("GrowFromPoint", "Indicate", "TurnInsideOut"):
            cls = getattr(self.n, name)
            self.assertTrue(self.requires(cls(final_alpha_value=0.5)))
            self.assertTrue(self.requires(cls(remover=True)))
        self.assertTrue(self.requires(self.n.Indicate(path_arc=0.5)))
        self.assertFalse(self.requires(self.n.GrowFromPoint(path_arc=0.5)))

    def test_existing_subsystem_decisions_are_preserved(self):
        self.assertTrue(self.requires(self.n.GrowFromPoint(previous_requires=True)))
        self.assertTrue(self.requires(self.n.Animation(previous_requires=True)))
        self.assertFalse(self.requires(self.n.Animation()))

    def test_custom_global_rate_remains_live_and_temporary_flag_is_removed(self):
        curve = lambda alpha: alpha * alpha
        effect = self.n.GrowFromCenter()
        decisions, kwargs, rates = self.n.Scene().play(effect, rate_func=curve)
        self.assertEqual(decisions, [True])
        self.assertIsNone(kwargs["rate_func"])
        self.assertIs(rates[0], curve)
        self.assertNotIn("_indication_force_callback", vars(effect))
        effect.rate_func = "linear"
        self.assertFalse(self.requires(effect))

    def test_custom_global_rate_reaches_deferred_leaves_in_nested_groups(self):
        curve = lambda alpha: alpha * alpha
        leaf = self.n.GrowFromCenter()
        root = self.n.AnimationGroup(self.n.AnimationGroup(leaf), self.n.Indicate())
        decisions, kwargs, rates = self.n.Scene().play(root, rate_func=curve)
        self.assertEqual(decisions, [True])
        self.assertIsNone(kwargs["rate_func"])
        self.assertIs(rates[0], curve)
        self.assertNotIn("_indication_force_callback", vars(leaf))
        root.rate_func = "linear"
        self.assertFalse(self.requires(root))

    def test_failure_restores_existing_dispatch_flag(self):
        effect = self.n.GrowFromPoint(_indication_force_callback=False)
        with self.assertRaisesRegex(RuntimeError, "original play failure"):
            self.n.Scene().play(effect, rate_func=lambda a: a, fail=True)
        self.assertIs(effect._indication_force_callback, False)

    def test_mixed_native_roots_keep_existing_global_override_payload(self):
        curve = lambda alpha: alpha * alpha
        effect = self.n.GrowFromPoint()
        decisions, kwargs, _ = self.n.Scene().play(effect, self.n.Animation(), rate_func=curve)
        self.assertEqual(decisions, [True, False])
        self.assertIs(kwargs["rate_func"], curve)
        self.assertNotIn("_indication_force_callback", vars(effect))

    def test_installer_is_idempotent_in_a_reduced_embedding(self):
        requires, play = self.n._requires_python_animation, self.n.Scene.play
        install_indication(self.n)
        self.assertIs(self.n._requires_python_animation, requires)
        self.assertIs(self.n.Scene.play, play)
        empty = types.SimpleNamespace()
        install_indication(empty)
        self.assertEqual(vars(empty), {})


if __name__ == "__main__":
    unittest.main()
