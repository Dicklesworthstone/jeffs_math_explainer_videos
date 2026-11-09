"""Implicit subclasses on native contour, ownership and rendered-frame paths."""
import copy
from pathlib import Path
import tempfile
import unittest

import manimlib as m
import numpy as np


DOMAIN = dict(x_range=(-1, 1), y_range=(-1, 1), min_depth=2, max_quads=64)


class ImplicitLifecycle(unittest.TestCase):
    def test_hooks_and_cooperative_mro_observe_recipe_before_sampling(self):
        events = []

        class Mixin:
            def init_points(self):
                events.append("mixin")
                super().init_points()
                self.shift(m.UP)

        class Parent(m.ImplicitFunction):
            def init_data(self):
                events.append("data")
                self.initial_recipe = self.func
                self.initial_controls = self.x_range, self.y_range, self.min_depth, self.max_quads
                super().init_data()

            def init_points(self):
                events.append("points")
                self.func = lambda x, y: y-.25
                super().init_points()

            def init_uniforms(self):
                events.append("uniforms")
                super().init_uniforms()
                self.uniforms["contour_author"] = 23.0

            def init_colors(self):
                events.append("colors")
                super().init_colors()
                self.set_stroke(m.GREEN, width=6)

        class Authored(Mixin, Parent):
            pass

        original = lambda x, y: y+.25
        obj = Authored(original, **DOMAIN)
        self.assertEqual(events, ["data", "mixin", "points", "uniforms", "colors"])
        self.assertIs(obj.initial_recipe, original)
        self.assertEqual(obj.initial_controls, ((-1, 1), (-1, 1), 2, 64))
        self.assertEqual(obj.uniforms["contour_author"], 23)
        self.assertEqual(obj.get_stroke_color(), m.GREEN)
        self.assertEqual(obj.get_stroke_width(), 6)
        self.assertEqual(obj.joint_type, "no_joint")
        self.assertGreater(obj.get_num_points(), 0)
        np.testing.assert_array_equal(obj.get_points()[:, 1], 1.25)

    def test_override_can_replace_contouring_without_calling_the_field(self):
        def forbidden(x, y):
            raise AssertionError("replaced contour hook must not call the field")

        class Authored(m.ImplicitFunction):
            def init_points(self):
                self.set_points([[-1, 2, 0], [0, 2, 0], [1, 2, 0]])

        obj = Authored(forbidden, **DOMAIN)
        np.testing.assert_array_equal(obj.get_points(), [[-1, 2, 0], [0, 2, 0], [1, 2, 0]])

    def test_custom_records_children_and_scene_owner_survive_refresh(self):
        class Authored(m.ImplicitFunction):
            data_dtype = m.VMobject.data_dtype + [("weight", 2)]

            def init_data(self):
                self.resize(1)
                self.set_field("weight", 0, [3, 7])
                self.label = m.Dot(point=3*m.RIGHT)
                self.add(self.label)

        obj = Authored(lambda x, y: y-.25, color=m.BLUE, **DOMAIN)
        np.testing.assert_array_equal(obj.get_field("weight", 0), [3, 7])
        self.assertIs(obj[0], obj.label)
        scene = m.Scene()
        scene.add(obj)
        obj.func = lambda x, y: y-.5
        obj.init_points()
        self.assertIs(scene.mobjects[0], obj)
        self.assertIs(obj._scene, scene)
        self.assertIs(obj[0], obj.label)
        self.assertEqual(obj.get_stroke_color(), m.BLUE)
        np.testing.assert_array_equal(obj.get_field("weight", 0), [3, 7])
        np.testing.assert_array_equal(obj.get_points()[:, 1], .5)

    def test_initial_records_and_missing_regions_match_independent_native_builder(self):
        for smooth in (False, True):
            for field in (lambda x, y: x*x+y*y-.31,
                          lambda x, y: np.nan if x < 0 else y-.25,
                          lambda x, y: 1):
                with self.subTest(smooth=smooth, field=field):
                    native = m.VMobject()
                    native._build_implicit_function(m._native_shell_factory, field,
                                                   (-1, 1), (-1, 1), 2, 64, smooth)
                    native.set_joint_type("no_joint")
                    obj = m.ImplicitFunction(field, use_smoothing=smooth, **DOMAIN)
                    self.assertEqual(obj.data.tobytes(), native.data.tobytes())
                    self.assertEqual(dict(obj.uniforms), dict(native.uniforms))

    def test_hook_errors_are_not_swallowed_repeated_or_wrapped(self):
        for name in ("init_data", "init_points", "init_uniforms", "init_colors"):
            for error in (LookupError(name), KeyboardInterrupt(name), SystemExit(name)):
                with self.subTest(name=name, error=type(error)):
                    calls = []

                    def fail(self):
                        calls.append(name)
                        raise error

                    cls = type("Broken", (m.ImplicitFunction,), {name: fail})
                    with self.assertRaises(type(error)) as caught:
                        cls(lambda x, y: y-.25, **DOMAIN)
                    self.assertIs(caught.exception, error)
                    self.assertEqual(calls, [name])
                    self.assertGreater(m.ImplicitFunction(lambda x, y: y-.25, **DOMAIN).get_num_points(), 0)

    def test_copies_keep_authored_class_without_repeating_initialization(self):
        calls = []

        class Authored(m.ImplicitFunction):
            def init_points(self):
                calls.append("points")
                super().init_points()
                self.shift(m.UP)

        obj = Authored(lambda x, y: y-.25, **DOMAIN)
        before = obj.get_points().copy()
        for clone in (obj.copy(), copy.copy(obj), copy.deepcopy(obj)):
            self.assertIs(type(clone), Authored)
            np.testing.assert_array_equal(clone.get_points(), before)
        self.assertEqual(calls, ["points"])
        clone.func = lambda x, y: y-.5
        clone.init_points()
        np.testing.assert_array_equal(clone.get_points()[:, 1], 1.5)
        np.testing.assert_array_equal(obj.get_points(), before)

    def test_callback_point_schema_change_is_not_overwritten(self):
        obj = m.ImplicitFunction(lambda x, y: y-.25, **DOMAIN)
        before = obj.get_points().copy()

        def change_schema(x, y):
            obj.pointlike_data_keys = ["point", "other"]
            return y-.5

        obj.func = change_schema
        with self.assertRaisesRegex(RuntimeError, "changed during sampling"):
            obj.init_points()
        np.testing.assert_array_equal(obj.get_points(), before)
        self.assertEqual(obj.pointlike_data_keys, ["point", "other"])
        obj.pointlike_data_keys = ["point"]
        obj.func = lambda x, y: y-.5
        obj.init_points()
        np.testing.assert_array_equal(obj.get_points()[:, 1], .5)

    def test_bound_reinitialization_refuses_before_editing_live_state(self):
        obj = m.ImplicitFunction(lambda x, y: y-.25, **DOMAIN)
        scene = m.Scene()
        scene.add(obj)
        before, field = obj.data.copy(), obj.func
        with self.assertRaisesRegex(RuntimeError, "detached"):
            obj.__init__(lambda x, y: y-.5, **DOMAIN)
        self.assertIs(obj.func, field)
        self.assertIs(obj._scene, scene)
        np.testing.assert_array_equal(obj.data, before)

    def test_authored_updating_contours_match_independent_line_frames(self):
        class Raised(m.ImplicitFunction):
            def init_points(self):
                super().init_points()
                self.shift(.5*m.UP)

        class Scene(m.Scene):
            reference = False

            def construct(self):
                level = m.ValueTracker(-.25)
                if self.reference:
                    obj = m.Line((-1, .25, 0), (1, .25, 0), buff=0)
                    obj.add_updater(lambda current: current.put_start_and_end_on(
                        (-1, level.get_value()+.5, 0), (1, level.get_value()+.5, 0)), call=False)
                else:
                    obj = Raised(lambda x, y: y-level.get_value(), **DOMAIN)
                    obj.add_updater(lambda current: current.init_points(), call=False)
                obj.set_stroke(m.BLUE, width=4, opacity=1).set_fill(opacity=0)
                obj.set_joint_type("no_joint")
                self.add(obj)
                self.play(level.animate.set_value(.25), run_time=.5, rate_func=m.linear)

        directory = Path(tempfile.mkdtemp(prefix="fmn-implicit-lifecycle-"))
        outputs = []
        for reference, threads in ((True, 1), (False, 1), (False, 4)):
            scene = Scene()
            scene.reference = reference
            path = directory / f"{reference}-{threads}.y4m"
            receipt = scene.render(path, format="y4m", resolution=(64, 36), fps=8, threads=threads)
            self.assertEqual(receipt.frame_count, 4)
            outputs.append(path.read_bytes())
        self.assertEqual(outputs[0], outputs[1])
        self.assertEqual(outputs[1], outputs[2])
        body = outputs[0].split(b"\n", 1)[1]
        stride = 6 + 64*36*3//2
        self.assertNotEqual(body[:stride], body[-stride:])


if __name__ == "__main__":
    unittest.main()
