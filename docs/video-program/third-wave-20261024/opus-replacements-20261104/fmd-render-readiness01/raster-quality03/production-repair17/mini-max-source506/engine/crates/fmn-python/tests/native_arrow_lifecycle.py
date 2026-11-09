"""Arrow/Vector public lifecycle and regeneration against the actual native kernel.

No fake records, geometry, frame clock or renderer satisfy these tests.
"""
import copy
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from manimlib.mobject.geometry import Arrow as QualifiedArrow


class NativeArrowLifecycle(unittest.TestCase):
    def assert_points(self, actual, expected):
        np.testing.assert_allclose(actual, expected, rtol=2e-5, atol=2e-5)

    def test_constructor_hooks_once_in_order_for_arrows_and_vectors(self):
        for base, args in ((m.Arrow, (m.LEFT, m.RIGHT)), (m.Vector, ([2, 1],))):
            with self.subTest(base=base.__name__):
                events = []
                class Authored(base):
                    def init_data(self):
                        events.append("data")
                        super().init_data()
                    def init_points(self):
                        events.append("points")
                        self.seen_recipe = (self.thickness, self.tip_angle, self.buff)
                        super().init_points()
                        self.shift(m.UP)
                    def init_uniforms(self):
                        events.append("uniforms")
                        super().init_uniforms()
                        self.uniforms["author_uniform"] = 7.0
                    def init_colors(self):
                        events.append("colors")
                        super().init_colors()
                        self.set_color(m.GREEN)
                expected = base(*args, thickness=5, buff=.1).shift(m.UP)
                obj = Authored(*args, thickness=5, buff=.1)
                self.assertEqual(events, ["data", "points", "uniforms", "colors"])
                self.assertEqual(obj.seen_recipe, (5.0, np.pi / 3, .1))
                self.assertEqual(obj.uniforms["author_uniform"], 7.0)
                self.assertEqual(obj.get_fill_color(), m.GREEN)
                self.assert_points(obj.get_points(), expected.get_points())
                self.assertIs(QualifiedArrow, m.Arrow)

    def test_multilevel_mixin_and_endpoint_hook_dispatch(self):
        events = []
        class Parent(m.Arrow):
            def init_points(self):
                events.append("parent")
                super().init_points()
            def set_points_by_ends(self, *args, **kwargs):
                events.append("ends")
                return super().set_points_by_ends(*args, **kwargs)
        class Mixin:
            def init_points(self):
                events.append("mixin")
                super().init_points()
                self.shift(2 * m.UP)
        class Child(Mixin, Parent):
            pass
        obj = Child(m.LEFT, m.RIGHT, buff=0)
        self.assertEqual(events, ["mixin", "parent", "ends"])
        self.assert_points(obj.get_start(), m.LEFT + 2 * m.UP)
        self.assert_points(obj.get_end(), m.RIGHT + 2 * m.UP)

    def test_override_can_replace_geometry_without_super(self):
        points = np.array([[0., 0., 0.], [1., 1., 0.], [2., 0., 0.]])
        class Custom(m.Vector):
            def init_points(self):
                self.points_on_entry = self.get_num_points()
                self.set_points(points)
                self.tip_index = 2
        obj = Custom([3, 2])
        self.assertEqual(obj.points_on_entry, 0)
        np.testing.assert_array_equal(obj.get_points(), points)
        self.assert_points(obj.get_end(), points[2])
        self.assertEqual(obj.get_fill_opacity(), 1.0)

    def test_custom_dtype_and_init_data_children_survive_construction_and_rebuild(self):
        class Custom(m.Arrow):
            data_dtype = m.VMobject.data_dtype + [("wobble", 2)]
            def init_data(self):
                self.resize(1)
                self.set_field("wobble", 0, [0.25, -0.5])
                self.label = m.Dot(point=3*m.RIGHT)
                self.add(self.label)
        obj = Custom(m.LEFT, m.RIGHT, buff=0, color=m.BLUE)
        self.assertIn("wobble", obj.data.dtype.names)
        self.assert_points(obj.get_field("wobble", 0), [.25, -.5])
        self.assertIs(obj[0], obj.label)
        obj.data["wobble"][:] = [7, 9]
        obj.put_start_and_end_on(m.DOWN, m.UP)
        self.assert_points(obj.data["wobble"], np.tile([7,9], (obj.get_num_points(),1)))
        self.assertIs(obj[0], obj.label)
        self.assertEqual(obj.get_fill_color(), m.BLUE)
        obj.path_arc = .75
        obj.init_points()
        self.assert_points(obj.data["wobble"], np.tile([7,9], (obj.get_num_points(),1)))
        self.assertIs(obj[0], obj.label)

    def test_regeneration_keeps_live_identity_styles_uniforms_and_updaters(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                obj = m.Arrow(m.LEFT, m.RIGHT, buff=0, color=m.BLUE,
                              fill_opacity=.4, stroke_width=3)
                child = m.Dot(point=3*m.RIGHT)
                obj.add(child)
                ticks = []
                def updater(current, dt): ticks.append(dt)
                obj.add_updater(updater)
                obj.uniforms["author_uniform"] = 4.0
                scene = m.Scene()
                if bound: scene.add(obj)
                identity, parents = hash(obj), list(obj.parents)
                self.assertIs(obj.set_points_by_ends(m.DOWN, m.UP, path_arc=.5), obj)
                self.assertEqual(hash(obj), identity)
                self.assertEqual(obj.parents, parents)
                self.assertIs(obj[0], child)
                self.assertIs(obj.updaters[0], updater)
                self.assertEqual(obj.get_fill_color(), m.BLUE)
                self.assertAlmostEqual(obj.get_fill_opacity(), .4, places=6)
                self.assertAlmostEqual(obj.get_stroke_width(), 3, places=6)
                self.assertEqual(obj.uniforms["author_uniform"], 4.0)
                if bound:
                    self.assertIs(scene.mobjects[0], obj)
                    scene.wait(1/30)
                    self.assertTrue(ticks)

    def test_same_count_views_remain_live_and_resized_views_detach(self):
        obj = m.Arrow(m.LEFT, m.RIGHT, buff=0)
        points = obj.get_points()
        obj.set_points_by_ends(m.DOWN, m.UP)
        np.testing.assert_array_equal(points, obj.get_points())
        points[0] += m.RIGHT
        np.testing.assert_array_equal(points, obj.get_points())
        old = points.copy()
        obj.set_points_by_ends(m.DOWN, m.UP, path_arc=1.5)
        self.assertNotEqual(len(points), obj.get_num_points())
        np.testing.assert_array_equal(points, old)
        points[:] = 40
        self.assertLess(float(np.max(obj.get_points())), 5)

    def test_failed_native_recipe_preserves_live_records_and_endpoint_metadata(self):
        obj = m.Arrow(m.LEFT, m.RIGHT, buff=0)
        scene = m.Scene()
        scene.add(obj)
        for start, end, options in (([np.nan,0,0], m.RIGHT, {}),
                                    (m.LEFT, m.RIGHT, {"path_arc": np.nan}),
                                    (m.LEFT, [np.inf,0,0], {})):
            with self.subTest(start=start, end=end, options=options):
                data, old_start, old_end = obj.data.copy(), obj.start.copy(), obj.end.copy()
                index = obj.tip_index
                with self.assertRaises(Exception):
                    obj.set_points_by_ends(start, end, **options)
                np.testing.assert_array_equal(obj.data, data)
                np.testing.assert_array_equal(obj.start, old_start)
                np.testing.assert_array_equal(obj.end, old_end)
                self.assertEqual(obj.tip_index, index)

    def test_hook_failures_propagate_once_without_following_hooks(self):
        for hook in ("init_data", "init_points", "init_uniforms", "init_colors"):
            with self.subTest(hook=hook):
                calls, failure = [], RuntimeError("authored " + hook)
                def fail(self):
                    calls.append(hook)
                    raise failure
                cls = type("Broken", (m.Arrow,), {hook: fail})
                with self.assertRaises(RuntimeError) as caught:
                    cls(m.LEFT,m.RIGHT)
                self.assertIs(caught.exception, failure)
                self.assertEqual(calls, [hook])
                self.assertGreater(m.Arrow(m.LEFT,m.RIGHT).get_num_points(), 0)

    def test_copy_and_deepcopy_do_not_reinitialize_recipe(self):
        calls = []
        class Authored(m.Arrow):
            def init_points(self):
                calls.append("points")
                super().init_points()
                self.shift(m.UP)
        obj = Authored(m.LEFT,m.RIGHT, buff=0)
        for clone in (obj.copy(), copy.deepcopy(obj)):
            self.assertIs(type(clone), Authored)
            np.testing.assert_array_equal(clone.get_points(), obj.get_points())
            self.assertEqual(clone.tip_index, obj.tip_index)
            clone.put_start_and_end_on(m.LEFT,2*m.RIGHT)
            self.assert_points(obj.get_start(), m.LEFT+m.UP)
        self.assertEqual(calls, ["points"])

    def test_init_points_rebuilds_an_arrow_not_an_inherited_plain_line(self):
        obj = m.Vector([3, 1], thickness=5)
        points, tip = obj.get_points().copy(), obj.tip_index
        self.assertIs(obj.init_points(), obj)
        np.testing.assert_array_equal(obj.get_points(), points)
        self.assertEqual(obj.tip_index, tip)
        self.assertGreater(obj.get_num_points(), 3)

    def test_live_scaling_preserves_custom_lanes_and_native_dimensions(self):
        class Custom(m.Vector):
            data_dtype = m.VMobject.data_dtype + [("heat", 1)]
        obj = Custom([2,1], thickness=4)
        obj.data["heat"][:] = .75
        for bound in (False, True):
            if bound:
                scene = m.Scene()
                scene.add(obj)
            start, end = obj.get_start().copy(), obj.get_end().copy()
            obj.scale(2, about_point=m.ORIGIN)
            self.assert_points(obj.get_start(), 2*start)
            self.assert_points(obj.get_end(), 2*end)
            np.testing.assert_array_equal(obj.data["heat"], np.full((obj.get_num_points(),1),.75))
            self.assertEqual(obj.get_stroke_width(), 0)

    def test_line_constructor_remains_in_the_cooperative_mro(self):
        calls, original = [], m.Line.__init__
        def line_init(obj, *args, **kwargs):
            calls.append(obj)
            return original(obj, *args, **kwargs)
        m.Line.__init__ = line_init
        try:
            arrow = m.Arrow(m.LEFT,m.RIGHT)
            vector = m.Vector([2,1])
        finally:
            m.Line.__init__ = original
        self.assertEqual(calls, [arrow,vector])

    def test_stock_pixels_match_native_builder_without_arrow_constructor(self):
        root = Path(tempfile.mkdtemp(prefix="fmn-arrow-native-oracle-"))
        def render(path, native):
            scene = m.Scene()
            with scene.render_session(path, format="png", resolution=(96,54), threads=1):
                for arc, y in ((0,-2),(.8,0),(-.8,2)):
                    if native:
                        obj = m._native_shell_factory()
                        children, tip = obj._build_arrow(m._native_shell_factory,
                            tuple(2*m.LEFT), tuple(2*m.RIGHT), .1, arc, 8, 5, np.pi/3, .5, .1)
                        self.assertFalse(children)
                        obj.set_fill(m.BLUE, opacity=1)
                        obj.set_stroke(m.BLUE, width=0)
                    else:
                        obj = m.Arrow(2*m.LEFT,2*m.RIGHT,buff=.1,path_arc=arc,
                                      thickness=8,color=m.BLUE)
                    obj.shift(y*m.UP)
                    scene.add(obj)
            return path.read_bytes()
        self.assertEqual(render(root/"public.png",False),render(root/"native.png",True))
        print("retained native-builder arrow comparison:",root)

    def test_native_outline_and_rendered_authored_geometry_at_one_four_threads(self):
        root = Path(tempfile.mkdtemp(prefix="fmn-arrow-lifecycle-"))
        def render(destination, threads, authored):
            scene = m.Scene()
            with scene.render_session(destination, format="png_sequence", fps=4,
                                      resolution=(96,54), threads=threads):
                class Shifted(m.Arrow):
                    def init_points(self):
                        super().init_points()
                        self.shift(m.UP)
                cls = Shifted if authored else m.Arrow
                obj = cls(2*m.LEFT,2*m.RIGHT, buff=.1, path_arc=.8, thickness=8, color=m.BLUE)
                if not authored: obj.shift(m.UP)
                scene.add(obj)
                scene.play(m.GrowArrow(obj, run_time=1, rate_func=m.linear))
            return [p.read_bytes() for p in sorted(destination.glob("*.png"))]
        frames = render(root/"one",1,True)
        self.assertEqual(frames, render(root/"four",4,True))
        self.assertEqual(frames, render(root/"expected",1,False))
        self.assertEqual(len(frames),4)
        self.assertGreater(len(set(frames)),2)
        # A fresh Atlas shell remains an independent construction oracle.
        for arc in (0,.8,-.8):
            obj = m.Arrow(m.LEFT,m.RIGHT,buff=.1,path_arc=arc,thickness=5)
            candidate = m._native_shell_factory()
            children,tip = candidate._build_arrow(m._native_shell_factory, tuple(m.LEFT),tuple(m.RIGHT),
                .1,arc,5,5,np.pi/3,.5,.1)
            self.assertFalse(children)
            self.assertEqual(obj.tip_index, tip)
            np.testing.assert_array_equal(obj.get_points(),candidate.get_points())
        print("retained native arrow frames:",root)


if __name__ == "__main__":
    unittest.main()
else:
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeArrowLifecycle))
    if not result.wasSuccessful():
        raise AssertionError("native arrow lifecycle acceptance failed")
