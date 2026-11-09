"""Real native tracker construction, schema ownership and callback acceptance."""
import copy
import importlib
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m


class TrackerLifecycleTests(unittest.TestCase):
    def test_cooperative_hooks_run_once_with_values_available_to_points(self):
        for base, value in ((m.ValueTracker, 3.), (m.ValueTracker, [2., 4.]),
                            (m.ExponentialValueTracker, 4.),
                            (m.ComplexValueTracker, 2+3j)):
            with self.subTest(base=base.__name__, value=value):
                events = []
                class Hooks:
                    def init_data(self):
                        events.append("data")
                        super().init_data()
                    def init_uniforms(self):
                        events.append("uniforms")
                        super().init_uniforms()
                    def init_updaters(self):
                        events.append("updaters")
                        return super().init_updaters()
                    def init_event_listners(self):
                        events.append("events")
                        return super().init_event_listners()
                    def init_points(self):
                        events.append("points")
                        self.observed = np.array(self.get_value(), copy=True)
                        super().init_points()
                        self.child = m.Dot()
                        self.add(self.child)
                    def init_colors(self):
                        events.append("colors")
                        super().init_colors()
                        self.set_color(m.BLUE)
                class Authored(Hooks, base):
                    pass
                tracker = Authored(value, color=m.RED)
                self.assertEqual(events, ["data", "uniforms", "updaters", "events", "points", "colors"])
                np.testing.assert_allclose(tracker.observed, value, atol=1e-14)
                self.assertIs(tracker[0], tracker.child)
                self.assertEqual(tracker.child.get_color(), m.BLUE)

    def test_custom_schema_views_children_and_native_value_survive_adoption(self):
        class Authored(m.ValueTracker):
            data_dtype = [*m.ValueTracker.data_dtype, ("tag", np.float32, (2,))]
            def init_data(self):
                super().init_data()
                self.resize(2)
                self.data["tag"][:] = [[7., 8.], [9., 10.]]
                self.view = self.data
                self.child = m.Dot()
                self.add(self.child)
            def init_points(self):
                self.set_points([[0., 0., 0.], [1., 0., 0.]])
        tracker = Authored(1 + 2**-40)
        child, view = tracker.child, tracker.view
        np.testing.assert_array_equal(view["tag"], [[7., 8.], [9., 10.]])
        scene = m.Scene()
        scene.add(tracker)
        self.assertIs(tracker[0], child)
        self.assertEqual(tracker.get_value(), 1 + 2**-40)
        tracker.data["tag"][0] = [11., 12.]
        np.testing.assert_array_equal(view["tag"][0], [11., 12.])
        for duplicate in (tracker.copy(), copy.deepcopy(tracker)):
            self.assertIsInstance(duplicate, Authored)
            self.assertIsNot(duplicate.child, child)
            self.assertIs(duplicate[0], duplicate.child)
            self.assertEqual(duplicate.get_value(), tracker.get_value())
            np.testing.assert_array_equal(duplicate.data, tracker.data)

    def test_init_data_can_change_constructor_seed_without_overwriting_uniform_hook(self):
        class Authored(m.ValueTracker):
            def init_data(self):
                super().init_data()
                self.value = [3., 5.]
            def init_uniforms(self):
                super().init_uniforms()
                self.set_value([7., 9.])
        tracker = Authored([0., 1.])
        np.testing.assert_array_equal(tracker.get_value(), [7., 9.])
        self.assertEqual(tracker._tracker_value(), 7.)

    def test_explicit_uniform_reinitialization_keeps_family_and_uses_seed(self):
        for base, value, changed in ((m.ValueTracker, [2., 4.], [3., 5.]),
                                    (m.ComplexValueTracker, 2+3j, 4+5j),
                                    (m.ExponentialValueTracker, 4., 9.)):
            with self.subTest(base=base.__name__):
                tracker = base(value)
                child = m.Dot()
                tracker.add(child)
                tracker.set_value(changed)
                tracker.init_uniforms()
                np.testing.assert_allclose(tracker.get_value(), value, atol=1e-14)
                self.assertIs(tracker[0], child)

    def test_constructor_style_and_authored_shading_are_not_overwritten(self):
        class Authored(m.ValueTracker):
            def init_points(self):
                self.set_points([[0., 0., 0.]])
                self.add(m.Dot())
            def init_uniforms(self):
                super().init_uniforms()
                self.set_shading(.4, .5, .6)
        tracker = Authored(2., color=m.BLUE, opacity=.5, shading=(.1, .2, .3),
                           z_index=7, depth_test=True, is_fixed_in_frame=True)
        self.assertEqual(tracker.z_index, 7)
        np.testing.assert_allclose(tracker.get_shading(), [.4, .5, .6])
        self.assertEqual(tracker[0].get_color(), m.BLUE)
        self.assertEqual(tracker.get_opacity(), .5)
        self.assertTrue(tracker.is_fixed_in_frame())
        self.assertTrue(tracker.depth_test)
        self.assertTrue(tracker.uniforms["depth_test"])
        self.assertTrue(tracker[0].uniforms["depth_test"])

    def test_hook_exceptions_propagate_without_running_later_hooks(self):
        for failing in ("init_data", "init_uniforms", "init_updaters", "init_event_listners",
                        "init_points", "init_colors"):
            with self.subTest(hook=failing):
                error = RuntimeError("authored tracker hook")
                def fail(self):
                    raise error
                cls = type("BrokenTracker", (m.ValueTracker,), {failing: fail})
                with self.assertRaises(RuntimeError) as caught:
                    cls(3.)
                self.assertIs(caught.exception, error)
                self.assertEqual(m.ValueTracker(5.).get_value(), 5.)

    def test_bad_values_and_keywords_fail_without_clipping_or_complex_truncation(self):
        for factory, value, error in ((m.ValueTracker, [], ValueError),
                                      (m.ValueTracker, [1+2j], TypeError),
                                      (m.ExponentialValueTracker, [2., 4.], TypeError),
                                      (m.ComplexValueTracker, object(), TypeError)):
            with self.subTest(factory=factory.__name__):
                with self.assertRaises(error):
                    factory(value)
        with self.assertRaisesRegex(TypeError, "unexpected keyword"):
            m.ValueTracker(2., unknown_tracker_option=True)

    def test_native_allocator_refuses_reinitialization_without_mutating_state(self):
        allocate = getattr(m, "_native", m)._portal_allocate_tracker
        tracker = m.ValueTracker(2.)
        tracker.add(m.Dot())
        for bound in (False, True):
            if bound:
                scene = m.Scene()
                scene.add(tracker)
            before = tracker._engine_state()["snapshot"]
            child = tracker[0]
            with self.assertRaisesRegex(RuntimeError, "only once"):
                allocate(tracker, 0)
            self.assertEqual(tracker._engine_state()["snapshot"], before)
            self.assertIs(tracker[0], child)
            self.assertEqual(tracker.get_value(), 2.)

    def test_controls_dispatch_hooks_without_resetting_native_widget_children(self):
        for base, value in ((m.ControlMobject, 2.), (m.Checkbox, True),
                            (m.EnableDisableButton, False), (m.LinearNumberSlider, 2.)):
            with self.subTest(base=base.__name__):
                calls = []
                class Authored(base):
                    def init_points(self):
                        calls.append(float(self.get_value()))
                        self.child = m.Dot()
                        self.add(self.child)
                tracker = Authored(value)
                self.assertEqual(calls, [float(value)])
                self.assertIn(tracker.child, tracker.submobjects)
                self.assertTrue(tracker.is_fixed_in_frame())
                self.assertTrue(tracker.has_updaters())
                if base is m.Checkbox:
                    child = tracker.child
                    box = tracker.box
                    tracker.toggle_value()
                    self.assertFalse(tracker.get_value())
                    self.assertIs(tracker.box, box)
                    self.assertIn(child, tracker.submobjects)

    def test_invalid_schema_and_encoding_do_not_publish_partial_native_state(self):
        allocate = getattr(m, "_native", m)._portal_allocate_tracker
        class Invalid(m.ValueTracker):
            data_dtype = [("point", np.float64, (3,)), ("rgba", 4)]
        shell = Invalid.__new__(Invalid)
        before = shell._engine_state()["snapshot"]
        with self.assertRaisesRegex(TypeError, "float32"):
            allocate(shell, 0)
        self.assertEqual(shell._engine_state()["snapshot"], before)
        shell = m.ValueTracker.__new__(m.ValueTracker)
        before = shell._engine_state()["snapshot"]
        with self.assertRaisesRegex(ValueError, "encoding"):
            allocate(shell, 3)
        self.assertEqual(shell._engine_state()["snapshot"], before)

    def test_schema_reentrancy_cannot_replace_an_initialized_root(self):
        allocate = getattr(m, "_native", m)._portal_allocate_tracker
        class Reentrant(m.ValueTracker):
            @property
            def data_dtype(self):
                self._init_value_tracker(0, 19., 0.)
                return m.ValueTracker.data_dtype
        shell = Reentrant.__new__(Reentrant)
        with self.assertRaisesRegex(RuntimeError, "only once"):
            allocate(shell, 0)
        self.assertEqual(shell._tracker_value(), 19.)

    def test_qualified_classes_and_repeated_installer_keep_identity(self):
        from fmn_python.tracker_lifecycle import install_tracker_lifecycle
        module = importlib.import_module("manimlib.mobject.value_tracker")
        self.assertIs(module.ValueTracker, m.ValueTracker)
        self.assertIs(module.ComplexValueTracker, m.ComplexValueTracker)
        before = (m.ValueTracker.__init__, m.ComplexValueTracker.__init__)
        install_tracker_lifecycle(getattr(m, "_native", m))
        self.assertEqual(before, (m.ValueTracker.__init__, m.ComplexValueTracker.__init__))

    def test_authored_tracker_callbacks_drive_native_rendered_frames(self):
        from fmn_python import render_scene
        root = Path(tempfile.mkdtemp(prefix="fmn-tracker-lifecycle-"))
        output = []
        for threads in (1, 4):
            for authored in (False, True):
                class Tracker(m.ValueTracker):
                    def init_points(self):
                        self.add(m.Square(side_length=.6, color=m.WHITE,
                                          fill_opacity=1, stroke_width=0))
                    def interpolate(self, start, end, alpha, path_func=None):
                        super().interpolate(start, end, alpha, path_func)
                        self.set_value(self.get_value() + alpha*(1-alpha))
                        return self
                class Render(m.Scene):
                    default_camera_config = dict(resolution=(96, 54), fps=8)
                    def construct(self):
                        tracker = Tracker(-2.) if authored else m.ValueTracker(-2.)
                        square = tracker[0] if authored else m.Square(
                            side_length=.6, color=m.WHITE, fill_opacity=1, stroke_width=0)
                        square.add_updater(lambda mob: mob.set_x(float(tracker.get_value())))
                        self.add(tracker)
                        if not authored:
                            self.add(square)
                        if authored:
                            self.play(tracker.animate.set_value(2.), run_time=.5, rate_func=m.linear)
                        else:
                            def control(current, alpha):
                                current.set_value(-2.+4.*alpha+alpha*(1-alpha))
                            self.play(m.UpdateFromAlphaFunc(tracker, control), run_time=.5,
                                      rate_func=m.linear)
                result = render_scene(Render, root / f"{threads}-{authored}.y4m", threads=threads)
                self.assertEqual(result.frame_count, 4)
                output.append(result.destination.read_bytes())
        self.assertTrue(all(value == output[0] for value in output))
        header, payload = output[0].split(b"\n", 1)
        self.assertEqual(header, b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2")
        size = 96 * 54 * 3 // 2
        self.assertEqual(len(payload), 4 * (6 + size))
        frames = [payload[i+6:i+6+size] for i in range(0, len(payload), 6+size)]
        self.assertGreater(len(set(frames)), 1, "tracker callback never reached rendered geometry")
        self.assertTrue(all(max(frame[:96*54]) > 100 for frame in frames), "blank render control")



    # fm-c1up/fm-5wq.13: typed scalar-control children are public API, not
    # anonymous records. These tests use the actual native geometry owners.
    def test_scalar_controls_expose_public_primitive_classes(self):
        slider, toggle = m.LinearNumberSlider(), m.EnableDisableButton()
        self.assertIs(type(slider.bar), m.RoundedRectangle)
        self.assertIs(type(slider.slider), m.Circle)
        self.assertIs(type(slider.slider_axis), m.Line)
        self.assertIs(type(toggle.box), m.Rectangle)
        self.assertEqual(toggle.box.get_vertices().shape, (4, 3))
        self.assertEqual(slider.bar.get_vertices().shape[1], 3)
        self.assertIs(m.LinearNumberSlider,
                      importlib.import_module('manimlib.mobject.interactive').LinearNumberSlider)

    def test_scalar_control_hooks_see_named_children_in_every_phase(self):
        for base, value in ((m.LinearNumberSlider, 3.), (m.EnableDisableButton, True)):
            events = []
            def check(obj, phase):
                names = ('bar', 'slider', 'slider_axis') if base is m.LinearNumberSlider else ('box',)
                events.append((phase, tuple(type(getattr(obj, name)) for name in names)))
            class Authored(base):
                def init_data(self):
                    check(self, 'data'); super().init_data()
                def init_uniforms(self):
                    check(self, 'uniforms'); super().init_uniforms()
                def init_points(self):
                    check(self, 'points'); super().init_points()
                    self.marker = m.Dot().shift(m.UP)
                    self.add(self.marker)
                def init_colors(self):
                    check(self, 'colors'); super().init_colors()
            with self.subTest(base=base.__name__):
                control = Authored(value)
                self.assertEqual([phase for phase, _ in events], ['data', 'uniforms', 'points', 'colors'])
                expected = ((m.RoundedRectangle, m.Circle, m.Line)
                            if base is m.LinearNumberSlider else (m.Rectangle,))
                self.assertTrue(all(classes == expected for _, classes in events))
                self.assertIn(control.marker, control.submobjects)
                self.assertEqual(control.get_value(), value)

    def test_control_children_honor_public_primitive_constructor_hooks(self):
        original = m.Circle.init_points
        calls = []
        def points(obj):
            calls.append(obj)
            original(obj)
            obj.stretch(1.5, 1, about_point=m.ORIGIN)
        m.Circle.init_points = points
        try:
            slider = m.LinearNumberSlider()
        finally:
            m.Circle.init_points = original
        self.assertEqual(calls, [slider.slider])
        self.assertAlmostEqual(slider.slider.get_height(), .3, places=6)
        self.assertAlmostEqual(slider.slider.get_width(), .2, places=6)

    def test_typed_slider_shapes_match_independent_public_construction(self):
        configs = ((dict(width=2., height=.075, corner_radius=.0375),
                    dict(radius=.1, stroke_color=m.GREY_A, fill_color=m.GREY_A, fill_opacity=1.)),
                   (dict(width=3., height=.2, corner_radius=.04, stroke_color=m.BLUE),
                    dict(radius=.2, fill_color=m.GREEN, stroke_color=m.RED, fill_opacity=.4)))
        for rect, circle in configs:
            slider = m.LinearNumberSlider(3., min_value=0., max_value=10.,
                                          rounded_rect_kwargs=rect, circle_kwargs=circle)
            bar, handle = m.RoundedRectangle(**rect), m.Circle(**circle)
            axis = m.Line(bar.get_bounding_box_point(m.LEFT), bar.get_bounding_box_point(m.RIGHT))
            axis.set_opacity(0.); handle.move_to(axis)
            for actual, expected in zip(slider.submobjects, (bar, handle, axis)):
                np.testing.assert_array_equal(actual.data, expected.data)
            self.assertEqual(slider.get_value(), 3.)
            np.testing.assert_array_equal(slider.slider.get_center(), m.ORIGIN)

    def test_typed_toggle_keeps_native_white_default_and_custom_colors(self):
        for value in (True, False):
            toggle = m.EnableDisableButton(value)
            self.assertEqual(toggle.box.get_fill_color(), m.WHITE)
            expected = m.Rectangle(width=.5, height=.5)
            np.testing.assert_array_equal(toggle.box.get_points(), expected.get_points())
            custom = m.EnableDisableButton(value, enable_color=m.BLUE, disable_color=m.YELLOW)
            self.assertEqual(custom.box.get_fill_color(), m.BLUE if value else m.YELLOW)
            custom.set_value(not value)
            self.assertEqual(custom.box.get_fill_color(), m.YELLOW if value else m.BLUE)

    def test_native_control_record_schema_survives_hooks_and_adoption(self):
        for base, value in ((m.LinearNumberSlider, 3.), (m.EnableDisableButton, True)):
            class Authored(base):
                data_dtype = [*m.ValueTracker.data_dtype, ('weight', np.float32, (1,))]
                def init_points(self):
                    super().init_points()
                    self.set_points([[0., 0., 0.]])
                    self.data['weight'][:] = 7.
                    self.view = self.data
            with self.subTest(base=base.__name__):
                control = Authored(value)
                children, view = tuple(control.submobjects), control.view
                scene = m.Scene().add(control)
                self.assertEqual(tuple(control.submobjects), children)
                self.assertIn(control, scene.mobjects)
                control.data['weight'][:] = 9.
                np.testing.assert_array_equal(view['weight'], [[9.]])
                control.set_value(4. if base is m.LinearNumberSlider else False)
                self.assertEqual(tuple(control.submobjects), children)
                np.testing.assert_array_equal(view['weight'], [[9.]])

    def test_control_copies_keep_types_aliases_and_independent_native_records(self):
        import pickle
        for original in (m.LinearNumberSlider(), m.EnableDisableButton()):
            names = ('bar', 'slider', 'slider_axis') if isinstance(original, m.LinearNumberSlider) else ('box',)
            for duplicate in (original.copy(), copy.deepcopy(original), pickle.loads(pickle.dumps(original))):
                with self.subTest(control=type(original).__name__):
                    self.assertIs(type(duplicate), type(original))
                    for name in names:
                        source, target = getattr(original, name), getattr(duplicate, name)
                        self.assertIs(type(target), type(source))
                        self.assertIsNot(source, target)
                        self.assertIn(target, duplicate.submobjects)
                        before = source.get_points().copy()
                        target.shift(m.UP)
                        np.testing.assert_array_equal(source.get_points(), before)

    def test_slider_live_drag_keeps_typed_children_and_axis_geometry(self):
        slider = m.LinearNumberSlider(0., min_value=-2., max_value=2., step=.5)
        slider.rotate(.4).shift(m.RIGHT)
        scene = m.Scene().add(slider)
        children = tuple(slider.submobjects)
        axis = slider.slider_axis.get_points().copy()
        point = slider.slider_axis.point_from_proportion(.75)
        self.assertFalse(slider.slider_on_mouse_drag(slider.slider, {'point': point}))
        self.assertEqual(slider.get_value(), 1.)
        self.assertEqual(tuple(slider.submobjects), children)
        np.testing.assert_array_equal(slider.slider_axis.get_points(), axis)
        np.testing.assert_allclose(slider.slider.get_center(), point, atol=2e-6)
        self.assertIn(slider, scene.mobjects)

    def test_slider_and_toggle_initialization_forward_base_options(self):
        for control in (m.LinearNumberSlider(z_index=7, name='author label'),
                        m.EnableDisableButton(z_index=7)):
            self.assertEqual(control.z_index, 7)
            self.assertTrue(control.is_fixed_in_frame())
            self.assertTrue(all(child.is_fixed_in_frame() for child in control.submobjects))
        self.assertEqual(m.LinearNumberSlider(name='author label').name, 'author label')

    def test_control_bad_parameters_never_start_tracker_hooks(self):
        calls = []
        class Slider(m.LinearNumberSlider):
            def init_data(self):
                calls.append('data'); super().init_data()
        for options in ({'min_value': 4., 'max_value': 2.}, {'step': 0.},
                        {'step': float('nan')}, {'value': float('inf')},
                        {'value': 11.}, {'rounded_rect_kwargs': None},
                        {'step': 5e-324}, {'rounded_rect_kwargs': {'width': 0.}}):
            with self.subTest(options=options), self.assertRaises((TypeError, ValueError)):
                Slider(**options)
        self.assertEqual(calls, [])

    def test_control_config_mappings_are_not_modified(self):
        rect = dict(width=3., height=.2, corner_radius=.04)
        circle = dict(radius=.2, fill_color=m.GREEN, stroke_color=m.RED, fill_opacity=.4)
        saved = copy.deepcopy((rect, circle))
        slider = m.LinearNumberSlider(rounded_rect_kwargs=rect, circle_kwargs=circle)
        self.assertEqual((rect, circle), saved)
        slider.rounded_rect_kwargs['width'] = 9.
        self.assertEqual((rect, circle), saved)

    def test_control_primitive_hook_failure_propagates_before_tracker_initialization(self):
        original = m.Circle.init_points
        calls, error = [], LookupError('authored knob')
        def points(obj):
            calls.append(obj)
            raise error
        m.Circle.init_points = points
        try:
            with self.assertRaises(LookupError) as caught:
                m.LinearNumberSlider()
        finally:
            m.Circle.init_points = original
        self.assertIs(caught.exception, error)
        self.assertEqual(len(calls), 1)

    def test_slider_accepts_ranges_wholly_outside_the_default_interval(self):
        for low, high, value in ((-40., -20., -30.), (20., 40., 30.)):
            slider = m.LinearNumberSlider(value, min_value=low, max_value=high, step=2.)
            self.assertEqual(slider.get_value(), value)
            slider.set_value(high)
            np.testing.assert_allclose(slider.slider.get_center(), slider.slider_axis.get_end(), atol=2e-6)

    def test_color_bank_keeps_compact_native_defaults_with_typed_channels(self):
        bank = m.ColorSliders()
        for channel in (bank.r_slider, bank.g_slider, bank.b_slider, bank.a_slider):
            self.assertIs(type(channel.bar), m.RoundedRectangle)
            self.assertIs(type(channel.slider), m.Circle)
            self.assertIs(type(channel.slider_axis), m.Line)
            self.assertAlmostEqual(channel.bar.get_width(), 2.)
            self.assertAlmostEqual(channel.bar.get_height(), .075)
            self.assertAlmostEqual(channel.slider.get_width(), .2)

    def test_typed_slider_animation_reaches_value_and_child_geometry(self):
        slider = m.LinearNumberSlider()
        scene = m.Scene().add(slider)
        scene.play(slider.animate.set_value(5.), run_time=.125)
        self.assertEqual(slider.get_value(), 5.)
        self.assertIs(type(slider.slider), m.Circle)
        np.testing.assert_allclose(slider.slider.get_center(),
                                   slider.slider_axis.point_from_proportion(.75), atol=2e-6)

    def test_typed_slider_render_matches_independent_native_shapes(self):
        def render(path, actual, workers):
            scene = m.Scene()
            with scene.render_session(path, format='png_sequence', resolution=(96,54), fps=8, threads=workers):
                if actual:
                    slider = m.LinearNumberSlider()
                    root, handle, axis = slider, slider.slider, slider.slider_axis
                else:
                    bar = m.RoundedRectangle(height=.075, width=2., corner_radius=.0375)
                    handle = m.Circle(radius=.1, stroke_color=m.GREY_A,
                                      fill_color=m.GREY_A, fill_opacity=1.)
                    axis = m.Line(m.LEFT, m.RIGHT).set_opacity(0.)
                    root = m.Group(bar, handle, axis).fix_in_frame()
                root.scale(2.).shift(m.UP)
                scene.add(root)
                for value in (-5., 0., 5.):
                    if actual:
                        slider.set_value(value)
                    else:
                        handle.move_to(axis.point_from_proportion((value+10.)/20.))
                    scene.wait(.125)
            return [p.read_bytes() for p in sorted(path.glob('*.png'))]
        root = Path(tempfile.mkdtemp(prefix='fmn-typed-control-'))
        expected = render(root/'control', False, 1)
        self.assertEqual(len(expected), 3)
        self.assertEqual(len(set(expected)), 3)
        for workers in (1,4,16):
            self.assertEqual(render(root/str(workers), True, workers), expected)


def run_tracker_lifecycle():
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(TrackerLifecycleTests))
    if not result.wasSuccessful():
        raise AssertionError("native tracker lifecycle acceptance failed")


if __name__ == "__main__":
    run_tracker_lifecycle()
