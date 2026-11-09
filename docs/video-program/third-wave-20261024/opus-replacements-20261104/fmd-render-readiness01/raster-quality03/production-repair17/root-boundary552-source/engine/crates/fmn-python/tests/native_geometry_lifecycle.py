"""Native-backed geometry must honor ordinary Python initialization and MRO.

Run against the real installed wheel: python native_geometry_lifecycle.py.
These tests do not replace, emulate, or mock the native geometry/scene engine.
"""
import copy
import unittest

import numpy as np
import manimlib as m


class NativeGeometryLifecycle(unittest.TestCase):
    def test_constructor_hooks_are_once_and_in_order(self):
        for base in (m.Arc, m.Circle, m.Dot, m.SmallDot, m.Ellipse):
            with self.subTest(base=base.__name__):
                class Authored(base):
                    def __init__(self):
                        self.events = []
                        super().__init__()
                        self.events.append("constructor")

                    def init_data(self):
                        self.events.append("data")
                        super().init_data()

                    def init_points(self):
                        self.events.append("points")
                        self.asserted_angle = self.angle
                        super().init_points()
                        self.shift(m.UP)

                    def init_uniforms(self):
                        self.events.append("uniforms")
                        super().init_uniforms()
                        self.uniforms["author_uniform"] = 7.0

                    def init_colors(self):
                        self.events.append("colors")
                        super().init_colors()
                        self.set_color(m.GREEN)

                obj = Authored()
                self.assertEqual(obj.events, ["data", "points", "uniforms", "colors", "constructor"])
                self.assertEqual(obj.uniforms["author_uniform"], 7.0)
                self.assertEqual(obj.get_color(), m.GREEN)
                self.assertGreater(obj.get_center()[1], 0.99)
                self.assertGreater(obj.get_num_points(), 0)

    def test_mixin_and_multilevel_super_dispatch(self):
        events = []

        class ShiftMixin:
            def init_points(self):
                events.append("mixin")
                super().init_points()
                self.shift(2 * m.UP)

        class Parent(m.Circle):
            def init_points(self):
                events.append("parent")
                super().init_points()
                self.shift(m.RIGHT)

        class Child(ShiftMixin, Parent):
            pass

        obj = Child(radius=2)
        self.assertEqual(events, ["mixin", "parent"])
        np.testing.assert_allclose(obj.get_center(), [1, 2, 0], atol=1e-6)
        self.assertAlmostEqual(obj.get_width(), 4.0, places=5)

    def test_override_can_replace_geometry_without_calling_super(self):
        points = np.array([[0., 0., 0.], [1., 1., 0.], [2., 0., 0.]])
        for base in (m.Arc, m.Circle, m.Dot, m.Ellipse):
            with self.subTest(base=base.__name__):
                class Authored(base):
                    def init_points(self):
                        self.points_on_entry = self.get_num_points()
                        self.set_points(points)

                obj = Authored()
                self.assertEqual(obj.points_on_entry, 0)
                np.testing.assert_array_equal(obj.get_points(), points)

    def test_constructor_recipe_is_visible_before_geometry_hook(self):
        class Authored(m.Arc):
            def init_points(self):
                self.recipe = (self.radius, self.angle, self.start_angle, self.n_components)
                super().init_points()

        obj = Authored(radius=3, angle=1.2, start_angle=0.4, n_components=7)
        self.assertEqual(obj.recipe, (3.0, 1.2, 0.4, 7))
        self.assertGreater(obj.get_num_points(), 0)

    def test_recipe_arrays_do_not_alias_caller_or_other_instances(self):
        point = np.array([1., 2., 0.])
        a = m.Circle(arc_center=point)
        b = m.Circle(arc_center=point)
        point[0] = 90
        a.arc_center[1] = 50
        np.testing.assert_array_equal(b.arc_center, [1, 2, 0])
        np.testing.assert_allclose(b.get_center(), [1, 2, 0], atol=1e-6)
        first, second = m.Dot(), m.Dot()
        first.arc_center[0] = 30
        np.testing.assert_array_equal(second.arc_center, [0, 0, 0])
        np.testing.assert_array_equal(m.ORIGIN, [0, 0, 0])

    def test_extra_record_fields_survive_native_geometry(self):
        class Authored(m.Circle):
            data_dtype = m.VMobject.data_dtype + [("wobble", 2)]

            def init_data(self):
                self.resize(1)
                self.set_field("wobble", 0, [0.25, -0.5])

            def init_points(self):
                super().init_points()
                self.set_field("wobble", self.get_num_points() - 1, [3, 4])

        obj = Authored(radius=2)
        self.assertIn("wobble", obj.data.dtype.names)
        np.testing.assert_allclose(obj.get_field("wobble", 0), [0.25, -0.5])
        np.testing.assert_allclose(obj.get_field("wobble", obj.get_num_points() - 1), [3, 4])
        self.assertAlmostEqual(obj.get_width(), 4, places=5)
        scene = m.Scene()
        scene.add(obj)
        self.assertIn("wobble", obj.data.dtype.names)

    def test_children_created_by_init_data_remain_attached(self):
        class Authored(m.Circle):
            def init_data(self):
                self.label = m.Dot(point=3 * m.RIGHT)
                self.add(self.label)

        obj = Authored()
        self.assertIs(obj[0], obj.label)
        scene = m.Scene()
        scene.add(obj)
        obj.shift(m.UP)
        np.testing.assert_allclose(obj.label.get_center(), [3, 1, 0], atol=1e-6)
        self.assertIs(obj[0], obj.label)

    def test_hook_errors_propagate_without_retry(self):
        for hook in ("init_data", "init_points", "init_uniforms", "init_colors"):
            with self.subTest(hook=hook):
                failure = RuntimeError("authored " + hook)
                calls = []

                def fail(self):
                    calls.append(hook)
                    raise failure

                cls = type("Broken", (m.Circle,), {hook: fail})
                with self.assertRaises(RuntimeError) as caught:
                    cls()
                self.assertIs(caught.exception, failure)
                self.assertEqual(calls, [hook])
                self.assertGreater(m.Circle().get_num_points(), 0)

    def test_copy_does_not_reinitialize_subclass(self):
        calls = []

        class Authored(m.Circle):
            def init_points(self):
                calls.append("points")
                super().init_points()
                self.shift(m.UP)

        original = Authored(radius=2)
        cloned = original.copy()
        deep = copy.deepcopy(original)
        self.assertEqual(calls, ["points"])
        self.assertIs(type(cloned), Authored)
        self.assertIs(type(deep), Authored)
        np.testing.assert_array_equal(cloned.get_points(), original.get_points())
        np.testing.assert_array_equal(deep.get_points(), original.get_points())

    def test_bound_regeneration_preserves_identity_family_style_and_updaters(self):
        obj = m.Circle(radius=1, color=m.BLUE, stroke_width=7)
        child = m.Dot(point=3 * m.RIGHT)
        obj.add(child)
        events = []

        def updater(mob, dt):
            events.append(dt)

        obj.add_updater(updater)
        scene = m.Scene()
        scene.add(obj)
        before_parent = obj.parents.copy()
        obj.radius = 2
        self.assertIs(obj.init_points(), obj)
        self.assertIs(scene.mobjects[0], obj)
        self.assertIs(obj[0], child)
        self.assertEqual(obj.parents, before_parent)
        self.assertIs(obj.updaters[0], updater)
        self.assertEqual(obj.get_stroke_color(), m.BLUE)
        self.assertAlmostEqual(obj.get_stroke_width(), 7)
        self.assertGreater(obj.get_num_points(), 0)
        np.testing.assert_allclose(obj.get_points()[:, 0].min(), -2, atol=1e-6)
        scene.wait(1 / 30)
        self.assertGreater(len(events), 0)

    def test_native_primitive_dimensions_and_default_styles(self):
        circle = m.Circle(radius=2, arc_center=[1, 2, 0])
        np.testing.assert_allclose(circle.get_center(), [1, 2, 0], atol=1e-6)
        self.assertAlmostEqual(circle.get_width(), 4, places=5)
        self.assertEqual(circle.get_stroke_color(), m.RED)
        self.assertEqual(circle.get_fill_opacity(), 0)
        dot = m.Dot(point=[2, 1, 0], radius=0.25)
        np.testing.assert_allclose(dot.get_center(), [2, 1, 0], atol=1e-6)
        self.assertAlmostEqual(dot.get_width(), 0.5, places=5)
        self.assertEqual(dot.get_fill_color(), m.WHITE)
        self.assertEqual(dot.get_fill_opacity(), 1)
        self.assertEqual(dot.get_stroke_width(), 0)
        ellipse = m.Ellipse(width=4, height=2)
        self.assertAlmostEqual(ellipse.get_width(), 4, places=5)
        self.assertAlmostEqual(ellipse.get_height(), 2, places=5)
        self.assertEqual(ellipse.get_stroke_color(), m.RED)

    def test_style_shorthand_and_explicit_native_flags(self):
        # fm-qead (BN-07 C-19): explicit channels beat color=, as the
        # Reference's `fill_color or color` does; color= fills only the rest.
        obj = m.Dot(color=m.GREEN, fill_color=m.BLUE, stroke_color=m.RED,
                    stroke_width=3, is_fixed_in_frame=True)
        self.assertEqual(obj.get_fill_color(), m.BLUE)
        self.assertEqual(obj.get_stroke_color(), m.RED)
        self.assertEqual(m.Dot(color=m.GREEN).get_fill_color(), m.GREEN)
        self.assertAlmostEqual(obj.get_stroke_width(), 3)
        obj = m.Circle(opacity=0.25)
        self.assertAlmostEqual(obj.get_fill_opacity(), 0.25)
        self.assertAlmostEqual(obj.get_stroke_opacity(), 0.25)

    def test_vmobject_init_colors_override_is_not_bypassed(self):
        class Authored(m.VMobject):
            def init_points(self):
                self.set_points([[0, 0, 0], [0.5, 0.5, 0], [1, 0, 0]])

            def init_colors(self):
                self.color_calls = getattr(self, "color_calls", 0) + 1
                super().init_colors()
                self.set_stroke(color=m.GREEN, width=11)

        obj = Authored(stroke_color=m.RED)
        self.assertEqual(obj.color_calls, 1)
        self.assertEqual(obj.get_stroke_color(), m.GREEN)
        self.assertAlmostEqual(obj.get_stroke_width(), 11)


if __name__ == "__main__":
    unittest.main()
