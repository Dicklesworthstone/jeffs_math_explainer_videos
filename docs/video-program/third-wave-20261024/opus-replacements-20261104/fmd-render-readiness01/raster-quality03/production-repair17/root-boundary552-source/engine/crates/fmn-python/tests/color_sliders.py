"""Installed-wheel color-control acceptance with real Atlas/Marionette state.

The companion protocol suite uses storage doubles. This suite requires native
controls, actual scene input/playback, and decoded Lumen/Reel output.
"""
import hashlib
from pathlib import Path
import tempfile
import unittest

import numpy as np
import manimlib as m
from fmn_python import render_scene
from manimlib.mobject.interactive import ColorSliders as QualifiedColorSliders


def channels(bank):
    return bank.r_slider, bank.g_slider, bank.b_slider, bank.a_slider


def unregister(root):
    for member in root.get_family():
        for listener in tuple(member.get_event_listners()):
            member.remove_event_listner(listener.event_type, listener.callback)


def drag(scene, channel, proportion):
    frame = scene.frame
    start = frame.from_fixed_frame_point(channel.slider.get_center())
    target = frame.from_fixed_frame_point(channel.slider_axis.point_from_proportion(proportion))
    # No preceding hover: Scene input uses the event's explicit point.
    scene.on_mouse_press(start, 1, 0)
    scene.on_mouse_drag(target, target - start, 1, 0)
    scene.on_mouse_release(target, 1, 0)


def swatch_matches(bank):
    np.testing.assert_allclose(
        m.color_to_rgb(bank.selected_color_box.get_fill_color()),
        m.color_to_rgb(bank.get_picked_color()), atol=2e-6, rtol=0,
    )
    assert abs(bank.selected_color_box.get_fill_opacity() - bank.get_picked_opacity()) < 2e-6


def y4m_frames(path):
    data = path.read_bytes()
    header, rest = data.split(b"\n", 1)
    assert header.startswith(b"YUV4MPEG2 "), header
    fields = header.split()
    width = int(next(field[1:] for field in fields if field.startswith(b"W")))
    height = int(next(field[1:] for field in fields if field.startswith(b"H")))
    assert any(field.startswith(b"C420") for field in fields), header
    size = width * height * 3 // 2
    frames = []
    while rest:
        marker, rest = rest.split(b"\n", 1)
        assert marker == b"FRAME" or marker.startswith(b"FRAME "), marker
        assert len(rest) >= size
        frames.append(rest[:size])
        rest = rest[size:]
    return width, height, frames


class ColorSliderAcceptance(unittest.TestCase):
    def setUp(self):
        self.owned = []
    def bank(self, **kwargs):
        result = m.ColorSliders(**kwargs)
        self.owned.append(result)
        return result
    def tearDown(self):
        for root in reversed(self.owned):
            unregister(root)
    def test_qualified_identity_real_scalar_children_and_constructor_handles(self):
        self.assertIs(QualifiedColorSliders, m.ColorSliders)
        bank = self.bank(default_rgb_value=64, default_a_value=.6)
        self.assertIs(type(bank.sliders), m.Group)
        self.assertEqual(list(bank.sliders), list(channels(bank)))
        for control in channels(bank):
            self.assertIsInstance(control, m.LinearNumberSlider)
            self.assertIsInstance(control, m.ValueTracker)
            fraction = (control.get_value() - control.min_value) / (control.max_value - control.min_value)
            np.testing.assert_allclose(control.slider.get_center(),
                                       control.slider_axis.point_from_proportion(fraction), atol=3e-6)
            self.assertTrue(control.slider.get_event_listners())
            self.assertTrue(control.slider.is_fixed_in_frame())
        swatch_matches(bank)
    def test_native_layout_and_custom_control_geometry_are_preserved(self):
        bank = self.bank(
            sliders_kwargs={"rounded_rect_kwargs": {"width": 3., "height": .12, "corner_radius": .04},
                            "circle_kwargs": {"radius": .16, "fill_opacity": .7}},
            rect_kwargs={"width": 3.5, "height": .7, "stroke_opacity": .4},
            background_grid_kwargs={"colors": [m.BLUE, m.WHITE], "single_square_len": .2},
            sliders_buff=.3,
        )
        # The constructor's native layout: rows arranged with each handle at its
        # axis midpoint, as the pinned Reference arranges them (every axis at
        # x=0 here). apply_value=True rows are validation-only geometry; the
        # handle at its value widens each row box and shifts it by half a radius.
        _, template, _ = bank._native_color_slider_parts((255., 255., 255., 1.), apply_value=False)
        for control, native_row in zip(channels(bank), template):
            self.assertAlmostEqual(float(control.slider_axis.get_center()[0]), 0., places=5)
            np.testing.assert_allclose(control.slider_axis.get_center(), native_row[2].get_center(), atol=3e-6)
            self.assertAlmostEqual(control.bar.get_width(), native_row[0].get_width(), places=5)
            self.assertAlmostEqual(control.slider.get_width(), native_row[1].get_width(), places=5)
        self.assertAlmostEqual(bank.selected_color_box.get_width(), 3.5, places=5)
        self.assertAlmostEqual(bank.selected_color_box.get_stroke_opacity(), .4, places=5)
    def test_drag_updates_independent_channels_swatch_and_not_camera(self):
        bank = self.bank()
        scene = m.Scene(drag_to_pan=True)
        scene.add(bank)
        center = scene.frame.get_center().copy()
        drag(scene, bank.r_slider, 0.)
        drag(scene, bank.a_slider, 0.)
        scene.update(0.)
        np.testing.assert_allclose(bank.get_value(), [0., 1., 1., 0.], atol=1e-7)
        np.testing.assert_equal(scene.frame.get_center(), center)
        swatch_matches(bank)
    def test_fixed_controls_accept_world_input_after_camera_transform(self):
        bank = self.bank()
        bank.scale(.7).shift(2 * m.RIGHT)
        scene = m.Scene(drag_to_pan=False)
        scene.add(bank)
        scene.frame.scale(1.4).shift(m.LEFT + .5 * m.UP)
        drag(scene, bank.b_slider, 0.)
        np.testing.assert_allclose(bank.get_value(), [1., 1., 0., 1.], atol=1e-7)
    def test_aggregate_set_keeps_transformed_geometry_and_listeners(self):
        bank = self.bank()
        bank.scale(.8).rotate(.3).shift(m.RIGHT)
        family = tuple(bank.get_family())
        axes = [control.slider_axis.get_points().copy() for control in channels(bank)]
        registrations = [tuple(control.slider.get_event_listners()) for control in channels(bank)]
        self.assertIsNone(bank.set_value(10., 20., 30., .4))
        self.assertEqual(tuple(bank.get_family()), family)
        for control, points, registered in zip(channels(bank), axes, registrations):
            np.testing.assert_array_equal(control.slider_axis.get_points(), points)
            self.assertEqual(tuple(control.slider.get_event_listners()), registered)
        np.testing.assert_allclose(bank.get_value(), [10/255., 20/255., 30/255., .4], atol=1e-7)
        swatch_matches(bank)
    def test_shared_step_clamping_and_invalid_values_keep_native_contract(self):
        bank = self.bank(sliders_kwargs={"step": 5.})
        requested = (-100., 27., 900., .3)
        _, _, expected = bank._native_color_slider_parts(requested, apply_value=True)
        bank.set_value(*requested)
        np.testing.assert_allclose([c.get_value() for c in channels(bank)], expected)
        self.assertEqual(bank.a_slider.step, .04)
        prior = [c.get_value() for c in channels(bank)]
        arrays = [c.get_points().copy() for c in bank.family_members_with_points()]
        with self.assertRaises((ValueError, RuntimeError)):
            bank.set_value(1., 2., 3., float("nan"))
        np.testing.assert_array_equal([c.get_value() for c in channels(bank)], prior)
        for member, points in zip(bank.family_members_with_points(), arrays):
            np.testing.assert_array_equal(member.get_points(), points)
    def test_tracker_animation_paints_intermediate_values_on_the_scene_clock(self):
        bank = self.bank()
        scene = m.Scene()
        scene.add(bank)
        seen = []
        def observe(current):
            swatch_matches(current)
            seen.append(float(current.r_slider.get_value()))
        bank.add_updater(observe)
        scene.play(bank.r_slider.animate.set_value(0.), run_time=.4, rate_func=m.linear)
        self.assertTrue(any(0. < value < 255. for value in seen), seen)
        self.assertAlmostEqual(bank.r_slider.get_value(), 0.)
        scene.play(bank.animate.set_value(10., 20., 30., .4), run_time=.2, rate_func=m.linear)
        np.testing.assert_allclose(bank.get_value(), [10/255., 20/255., 30/255., .4], atol=1e-6)
        swatch_matches(bank)
    def test_suspension_then_resumption_and_receiver_correct_copy(self):
        bank = self.bank()
        initial = m.color_to_rgb(bank.selected_color_box.get_fill_color())
        bank.suspend_updating()
        bank.r_slider.set_value(0.)
        bank.update(.1)
        np.testing.assert_array_equal(m.color_to_rgb(bank.selected_color_box.get_fill_color()), initial)
        bank.resume_updating()
        swatch_matches(bank)
        duplicate = bank.copy()
        self.owned.append(duplicate)
        self.assertIsNot(duplicate.r_slider, bank.r_slider)
        duplicate.g_slider.set_value(0.)
        duplicate.update(0.)
        swatch_matches(duplicate)
        self.assertAlmostEqual(bank.g_slider.get_value(), 255.)
        self.assertAlmostEqual(duplicate.g_slider.get_value(), 0.)
    def test_removed_bank_does_not_receive_another_scenes_input(self):
        first, second = self.bank(), self.bank()
        one, two = m.Scene(), m.Scene()
        one.add(first); two.add(second)
        one.remove(first)
        drag(one, first.r_slider, 0.)
        self.assertAlmostEqual(first.r_slider.get_value(), 255.)
        self.assertAlmostEqual(second.r_slider.get_value(), 255.)
        drag(two, second.r_slider, 0.)
        self.assertAlmostEqual(second.r_slider.get_value(), 0.)
        self.assertAlmostEqual(first.r_slider.get_value(), 255.)
    def test_background_hook_is_live_and_default_getter_returns_fresh_geometry(self):
        class Authored(m.ColorSliders):
            def get_background(self):
                self.background_calls = getattr(self, "background_calls", 0) + 1
                result = super().get_background()
                result.set_fill(m.BLUE, opacity=1)
                return result
        bank = Authored()
        self.owned.append(bank)
        self.assertEqual(bank.background_calls, 1)
        bank.shift(m.UP)
        background = bank.get_background()
        self.assertIsNot(background, bank.background)
        np.testing.assert_allclose(background.get_center(), bank.selected_color_box.get_center(), atol=2e-6)
        self.assertGreater(len(background.get_family()), 2)
    def test_swatch_and_checkerboard_use_public_rectangle_and_square_classes(self):
        bank = self.bank(background_grid_kwargs={"colors": [m.RED, m.BLUE], "single_square_len": .2})
        self.assertIs(type(bank.selected_color_box), m.Rectangle)
        self.assertEqual(len(bank.selected_color_box.get_vertices()), 4)
        self.assertIs(type(bank.background), m.VGroup)
        self.assertFalse(bank.background.has_points())
        self.assertEqual(len(bank.background), 22)
        for index, square in enumerate(bank.background):
            self.assertIs(type(square), m.Square)
            self.assertEqual(len(square.get_vertices()), 4)
            self.assertEqual(square.get_fill_color(), (m.RED, m.BLUE)[index % 2])
            self.assertEqual(square.get_stroke_width(), 0.)
        native, _, _ = bank._native_color_slider_parts((255.,255.,255.,1.), apply_value=False)
        for square, template in zip(bank.background, native[0]):
            np.testing.assert_allclose(square.get_bounding_box(), template.get_bounding_box(), atol=3e-6)
    def test_custom_rgb_checkerboard_keeps_precision_when_regenerated(self):
        colors = ((.123456789, .345678912, .789123456),
                  (.876543219, .654321987, .210987654))
        bank = self.bank(
            rect_kwargs={"width": 1., "height": .5},
            background_grid_kwargs={"colors": colors, "single_square_len": .25},
        )
        bank.shift(m.RIGHT + m.UP)
        family = tuple(bank.get_family())
        before = [square.data["fill_rgba"].copy() for square in bank.background]
        regenerated = bank.get_background()
        self.assertIsNot(regenerated, bank.background)
        self.assertEqual(tuple(bank.get_family()), family)
        np.testing.assert_allclose(regenerated.get_center(),
                                   bank.selected_color_box.get_center(), atol=3e-6)
        for background in (bank.background, regenerated):
            self.assertIs(type(background), m.VGroup)
            self.assertEqual(len(background), 10)
            for index, square in enumerate(background):
                self.assertIs(type(square), m.Square)
                self.assertEqual(len(square.get_vertices()), 4)
                rgba = np.asarray(square.data["fill_rgba"])
                self.assertGreater(len(rgba), 0)
                np.testing.assert_allclose(rgba[:, :3],
                                           np.tile(colors[index % 2], (len(rgba), 1)),
                                           rtol=0, atol=1e-7)
                np.testing.assert_array_equal(rgba[:, 3], np.ones(len(rgba)))
                self.assertTrue(square.is_fixed_in_frame())
        for square, expected in zip(bank.background, before):
            np.testing.assert_array_equal(square.data["fill_rgba"], expected)

    def test_default_checkerboard_matches_the_reference_grid(self):
        # The Reference's ColorSliders() (class-sweep facts at 6199a00d):
        # 105 Square members in a 5x21 grid whose fills alternate GREY_A and
        # GREY_C (53 and 52) behind zero-width, transparent strokes.
        bank = self.bank()
        self.assertEqual(len(bank.background), 105)
        for index, square in enumerate(bank.background):
            self.assertIs(type(square), m.Square)
            self.assertEqual(square.get_fill_color(), (m.GREY_A, m.GREY_C)[index % 2])
            self.assertEqual(square.get_fill_opacity(), 1.)
            self.assertEqual(square.get_stroke_width(), 0.)
            self.assertEqual(square.get_stroke_opacity(), 0.)
        fills = [square.get_fill_color() for square in bank.background]
        self.assertEqual((fills.count(m.GREY_A), fills.count(m.GREY_C)), (53, 52))
        # A regenerated grid paints from the same validated authored colors.
        fresh = bank.get_background()
        self.assertEqual([square.get_fill_color() for square in fresh], fills)
    def test_background_uses_public_square_hooks_without_recasting_authored_children(self):
        original, calls = m.Square.init_points, []
        def points(square):
            original(square)
            calls.append(square)
            square.authored_marker = "public square"
        m.Square.init_points = points
        try:
            bank = self.bank()
        finally:
            m.Square.init_points = original
        self.assertEqual(len(calls), 1)
        self.assertTrue(all(square.authored_marker == "public square" for square in bank.background))
        self.assertIs(type(bank.get_background()[0]), m.Square)
    def test_typed_color_family_survives_copy_binding_and_channel_edits(self):
        bank = self.bank()
        duplicate = bank.copy()
        self.owned.append(duplicate)
        m.Scene().add(duplicate)
        family = tuple(duplicate.get_family())
        duplicate.set_value(20.,40.,60.,.5)
        self.assertEqual(tuple(duplicate.get_family()), family)
        self.assertIs(type(duplicate.selected_color_box), m.Rectangle)
        self.assertTrue(all(type(square) is m.Square for square in duplicate.background))
        self.assertAlmostEqual(bank.r_slider.get_value(), 255.)
        swatch_matches(duplicate)
    def test_panel_uses_typed_labels_rectangles_and_pointless_group_roots(self):
        for content in (m.Checkbox(), self.bank()):
            panel = m.ControlPanel(content, opener_text_kwargs={"text": "Settings", "font_size": 24})
            self.owned.append(panel)
            self.assertIs(type(panel.panel), m.Rectangle)
            self.assertIs(type(panel.panel_opener_rect), m.Rectangle)
            self.assertEqual(panel.panel_opener_rect.get_stroke_color(), m.GREY_A)
            self.assertIs(type(panel.panel_opener), m.Group)
            self.assertIs(type(panel.controls), m.Group)
            self.assertIs(type(panel.panel_info_text), m.Text)
            self.assertEqual(panel.panel_info_text.string, "Settings")
            self.assertFalse(panel.panel_opener.has_points())
            self.assertFalse(panel.controls.has_points())
            self.assertFalse(panel.panel_info_text.has_points())
            self.assertIs(panel.controls[0], content)
            identities = panel.panel, panel.panel_opener, panel.panel_opener_rect, panel.panel_info_text, panel.controls
            for operation in (panel.open_panel, panel.close_panel, panel.open_panel):
                operation()
                self.assertEqual((panel.panel, panel.panel_opener, panel.panel_opener_rect,
                                  panel.panel_info_text, panel.controls), identities)
                self.assertEqual(panel.panel_info_text.string, "Settings")
                self.assertEqual(len(panel.panel.get_vertices()), 4)
                np.testing.assert_allclose(panel.panel_info_text.get_center(),
                                           panel.panel_opener_rect.get_center(), atol=3e-6)
    def test_panel_label_runs_public_text_hooks_once_across_open_close(self):
        original, calls = m.Text.init_points, []
        def points(label):
            original(label)
            calls.append(label)
        m.Text.init_points = points
        try:
            panel = m.ControlPanel(m.Checkbox())
            self.owned.append(panel)
            panel.open_panel().close_panel()
        finally:
            m.Text.init_points = original
        self.assertEqual(calls, [panel.panel_info_text])
    def test_panel_contains_live_color_bank_and_accepts_late_composite_content(self):
        bank = self.bank()
        panel = m.ControlPanel(bank)
        self.owned.append(panel)
        self.assertIs(panel.controls.submobjects[0], bank)
        panel.open_panel()
        scene = m.Scene(drag_to_pan=False)
        scene.add(panel)
        drag(scene, bank.r_slider, 0.)
        scene.update(0.)
        self.assertAlmostEqual(bank.r_slider.get_value(), 0.)
        swatch_matches(bank)
        content = m.Group(m.Checkbox(), m.LinearNumberSlider())
        self.owned.append(content)
        panel.add_controls(content)
        self.assertIs(panel.controls.submobjects[-1], content)
        panel.remove_controls(content)
        self.assertNotIn(content, panel.controls.submobjects)
    def test_bare_shapes_keep_the_existing_panel_refusal(self):
        bank = self.bank()
        panel = m.ControlPanel(bank)
        self.owned.append(panel)
        prior = tuple(panel.controls.submobjects)
        for shape in (m.Circle(), m.Group(), m.Group(m.Square())):
            with self.subTest(shape=type(shape).__name__):
                with self.assertRaisesRegex(TypeError, "^ControlPanel controls must be ControlMobject instances$"):
                    m.ControlPanel(shape)
                with self.assertRaisesRegex(TypeError, "^ControlPanel controls must be ControlMobject instances$"):
                    panel.add_controls(shape)
        self.assertEqual(tuple(panel.controls.submobjects), prior)

    def test_real_output_changes_with_drag_and_is_thread_repeatable(self):
        class PickColors(m.Scene):
            def construct(self):
                bank = m.ColorSliders()
                self.add(bank)
                try:
                    self.wait(.25)
                    drag(self, bank.g_slider, 0.)
                    drag(self, bank.b_slider, 0.)
                    self.wait(.25)
                    drag(self, bank.a_slider, 0.)
                    self.wait(.25)
                finally:
                    unregister(bank)
        with tempfile.TemporaryDirectory(prefix="fmn-color-controls-") as directory:
            outputs = []
            for threads in (1, 4):
                path = Path(directory) / f"color-{threads}.y4m"
                receipt = render_scene(PickColors, path, format="y4m", resolution=(160, 96), fps=8, threads=threads)
                self.assertEqual(receipt.frame_count, 6)
                width, height, frames = y4m_frames(path)
                self.assertEqual((width, height, len(frames)), (160, 96, 6))
                self.assertEqual(frames[0], frames[1])
                self.assertEqual(frames[2], frames[3])
                self.assertEqual(frames[4], frames[5])
                for left, right in ((0, 2), (2, 4)):
                    changed = sum(a != b for a, b in zip(frames[left][:width*height], frames[right][:width*height]))
                    self.assertGreater(changed, 30)
                outputs.append(path.read_bytes())
            self.assertEqual(hashlib.sha256(outputs[0]).digest(), hashlib.sha256(outputs[1]).digest())


result = unittest.TextTestRunner(verbosity=2).run(
    unittest.defaultTestLoader.loadTestsFromTestCase(ColorSliderAcceptance)
)
assert result.wasSuccessful(), "native color-control acceptance failed"
