"""Live color-bank orchestration with explicit geometry/tracker doubles.

These fixtures are not native geometry, value-normalization or render proof.
The installed-wheel color_sliders suite tests those separate boundaries.
"""
from __future__ import annotations

import ast
import copy
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest

import numpy as np

SOURCE = Path(__file__).resolve().parents[1] / "python" / "fmn_python" / "color_sliders.py"
spec = importlib.util.spec_from_file_location("color_sliders_under_test", SOURCE)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def environment(install=True):
    listeners, native_calls, created = [], [], []
    class Mob:
        def __init__(self, *children):
            self.submobjects = list(children)
            self.point = np.zeros(3)
            self.style = {}
            self.updaters, self.event_listners = [], []
            self.fixed = self.suspended = False
        def get_center(self):
            return self.point.copy()
        def get_family(self):
            return [self, *(m for child in self.submobjects for m in child.get_family())]
        def shift(self, delta):
            for member in self.get_family():
                member.point += delta
            return self
        def move_to(self, other):
            return self.shift((other.point if isinstance(other, Mob) else other) - self.point)
        def set_submobjects(self, children):
            self.submobjects[:] = children
            return self
        def match_style(self, other):
            self.style = dict(other.style)
            return self
        def set_fill(self, color, opacity):
            self.style.update(color=color, opacity=opacity)
            return self
        def fix_in_frame(self):
            for member in self.get_family():
                member.fixed = True
            return self
        def add_updater(self, updater):
            self.updaters.append(updater)
            self.update(0.)
            return self
        def update(self, dt=0.):
            if self.suspended:
                return self
            for child in self.submobjects:
                child.update(dt)
            for updater in tuple(self.updaters):
                updater(self)
            return self
        def get_event_listners(self):
            return self.event_listners
        def remove_event_listner(self, kind, callback):
            for listener in tuple(self.event_listners):
                if listener.event_type == kind and listener.callback is callback:
                    self.event_listners.remove(listener)
                    listeners.remove(listener)
    class Group(Mob):
        pass
    class VectorShell(Mob):
        def set_submobjects(self, children):
            if any(isinstance(child, Slider) for child in children):
                raise TypeError("scalar controls cannot inhabit a vector-only group")
            return super().set_submobjects(children)
    class Slider(Mob):
        def __init__(self, value, **config):
            self.config = config
            self.min_value, self.max_value = config["min_value"], config["max_value"]
            self.step, self.value_type = config["step"], config["value_type"]
            self.bar, self.slider, self.slider_axis = Mob(), Mob(), Mob()
            self.value = value
            super().__init__(self.bar, self.slider, self.slider_axis)
            listener = SimpleNamespace(event_type="drag", callback=self.set_value)
            self.slider.event_listners.append(listener)
            listeners.append(listener)
            created.append(self)
        def assert_value(self, value):
            if not self.min_value <= value <= self.max_value:
                raise ValueError("channel outside range")
        def set_value(self, value):
            self.assert_value(value)
            self.value = value
            alpha = (value - self.min_value) / (self.max_value - self.min_value)
            self.slider.move_to(self.slider_axis.point + np.array([2 * alpha - 1, 0, 0]))
            return self
        def get_value(self):
            return self.value_type(self.value)
    class Bank(Group):
        def __init__(self, sliders_kwargs=None, default_rgb_value=255., default_a_value=1., **kwargs):
            if kwargs:
                raise TypeError("unexpected keyword")
            self.sliders_kwargs = dict(sliders_kwargs or {})
            self._slider_value_type = np.dtype(self.sliders_kwargs.get("value_type", float)).type
            self._color_slider_components = (default_rgb_value,) * 3 + (default_a_value,)
            self.swatch, self.sliders, values = self._native_color_slider_parts(
                self._color_slider_components, apply_value=False,
            )
            self._color_slider_components = values
            self.background, self.selected_color_box = self.swatch.submobjects
            self.r_slider, self.g_slider, self.b_slider, self.a_slider = self.sliders.submobjects
            super().__init__(self.swatch, self.sliders)
        def _native_color_slider_parts(self, values, *, apply_value):
            native_calls.append((tuple(values), apply_value))
            if apply_value:
                values = native.normalizer(tuple(values))
            box = Mob().shift(np.array([0., 3., 0.]))
            background = Mob().move_to(box)
            background.native_grid = True
            rows = []
            for index in range(4):
                parts = [Mob(), Mob(), Mob()]
                parts[1].style = {"native_palette": index}
                row = VectorShell(*parts).shift(np.array([1., 2. - index, 0.]))
                rows.append(row)
            return VectorShell(background, box), VectorShell(*rows), tuple(values)
        def get_value(self):
            r, g, b, a = self._color_slider_components
            return np.array([r / 255., g / 255., b / 255., a])
        def set_value(self, *values):
            self._color_slider_components = values
        def get_background(self):
            raise NotImplementedError("unbound semantic method")
        def get_picked_color(self):
            return tuple(self.get_value()[:3])
        def get_picked_opacity(self):
            return self.get_value()[3]
    native = SimpleNamespace(ColorSliders=Bank, Group=Group, LinearNumberSlider=Slider,
                             Mobject=Mob, _np=np, normalizer=lambda values: values,
                             listeners=listeners, native_calls=native_calls, created=created)
    if install:
        module.install_color_sliders(native)
    return native


def typed_background_environment():
    """Real scalar-color/fill methods over explicit record and grid doubles."""
    native = environment(False)
    source = SOURCE.parents[1] / "manimlib_bootstrap.py"
    tree = ast.parse(source.read_text())
    definitions = {node.name: node for node in tree.body
                   if isinstance(node, (ast.FunctionDef, ast.ClassDef))}
    selected = [definitions[name] for name in (
        "_listify", "_array_is_constant", "_resize_with_interpolation",
        "_ColorValue", "_color_to_rgb",
    )]
    for owner, names in (
        ("Mobject", ("_style_data", "set_rgba_array_by_color")),
        ("VMobject", ("set_fill",)),
    ):
        methods = {node.name: node for node in definitions[owner].body
                   if isinstance(node, ast.FunctionDef)}
        selected.extend(methods[name] for name in names)
    namespace = {"_np": np, "_family_preorder": lambda root: root.get_family()}
    exec(compile(ast.Module(selected, type_ignores=[]), str(source), "exec"), namespace)  # ubs:ignore -- fixed definitions from this repository's bootstrap; no external input

    class Records(native.Mobject):
        def __init__(self):
            super().__init__()
            self.rows = np.zeros(9, dtype=[("fill_rgba", float, (4,))])
        @property
        def data(self):
            return self.rows
        def n_records(self):
            return len(self.rows)
        def _fill_rgba_lanes(self, *args):
            # Storage is explicitly doubled, so use the shipping view-write
            # fallback rather than claiming native lane-fill acceptance.
            return False
    namespace["Mobject"] = Records
    Records._style_data = namespace["_style_data"]
    Records.set_rgba_array_by_color = namespace["set_rgba_array_by_color"]
    Records.set_fill = namespace["set_fill"]

    class Grid(native.Group):
        def __iter__(self):
            return iter(self.submobjects)
        def stretch_to_fit_width(self, width):
            self.width = width
            return self
        def stretch_to_fit_height(self, height):
            self.height = height
            return self
    class Square(Records):
        def __init__(self, length):
            super().__init__()
            self.length = length
        def get_grid(self, *, n_rows, n_cols, buff):
            return Grid(*(type(self)(self.length) for _ in range(n_rows * n_cols)))
        def set_stroke(self, **kwargs):
            self.style.update(kwargs)
            return self
    native.Square, native.Rectangle, native.VGroup = Square, Square, Grid
    native._ColorValue = namespace["_ColorValue"]
    module.install_color_sliders(native)
    return native


class ColorSliderTests(unittest.TestCase):
    def setUp(self):
        self.n = environment()
    def test_package_activates_after_control_event_installation(self):
        from fmn_python.initialization import _STEPS
        step_names = [step[1] for step in _STEPS]
        self.assertLess(step_names.index("install_control_events"), step_names.index("install_color_sliders"))
    def test_old_bank_is_not_independently_editable(self):
        old = environment(False)
        bank = old.ColorSliders()
        self.assertFalse(isinstance(bank.r_slider, old.LinearNumberSlider))
        with self.assertRaises(NotImplementedError):
            bank.get_background()
    def test_real_channels_and_nonvector_groups(self):
        bank = self.n.ColorSliders()
        self.assertIs(type(bank.sliders), self.n.Group)
        self.assertIs(type(bank.swatch), self.n.Group)
        self.assertEqual(bank.sliders.submobjects, list(module._channels(bank)))
        self.assertTrue(all(isinstance(c, self.n.LinearNumberSlider) for c in module._channels(bank)))
        self.assertEqual(len(self.n.listeners), 4)
    def test_channels_reflect_native_normalization_and_initial_positions(self):
        self.n.normalizer = lambda values: (20., 40., 60., .2)
        bank = self.n.ColorSliders()
        np.testing.assert_allclose(bank.get_value(), [20 / 255., 40 / 255., 60 / 255., .2])
        for i, c in enumerate(module._channels(bank)):
            np.testing.assert_allclose(c.slider_axis.point, [1., 2. - i, 0.])
            self.assertEqual(c.slider.style, {"native_palette": i})
            self.assertTrue(c.fixed)
        self.assertNotEqual(bank.r_slider.slider.point[0], bank.r_slider.slider_axis.point[0])
    def test_default_ranges_and_alpha_shared_step_exception(self):
        bank = self.n.ColorSliders(sliders_kwargs={"step": 5.})
        self.assertEqual([c.step for c in module._channels(bank)], [5., 5., 5., .04])
        expanded = self.n.ColorSliders(sliders_kwargs={"max_value": 300., "step": 5.})
        self.assertEqual(expanded.a_slider.step, 5.)
    def test_configuration_inputs_and_nested_dicts_not_mutated(self):
        config = {"rounded_rect_kwargs": {"width": 4.}, "circle_kwargs": {"radius": .2}}
        before = copy.deepcopy(config)
        bank = self.n.ColorSliders(sliders_kwargs=config)
        bank.r_slider.config["rounded_rect_kwargs"]["width"] = 99
        self.assertEqual(config, before)
        self.assertEqual(bank.g_slider.config["rounded_rect_kwargs"]["width"], 4.)
    def test_each_channel_drives_readout_instead_of_stale_tuple(self):
        bank = self.n.ColorSliders()
        bank.r_slider.set_value(11.); bank.a_slider.set_value(.3)
        bank._color_slider_components = (-1.,) * 4
        np.testing.assert_allclose(bank.get_value(), [11 / 255., 1., 1., .3])
    def test_whole_bank_set_preserves_position_and_registered_identities(self):
        bank = self.n.ColorSliders().shift(np.array([3., -2., 0.]))
        objects = tuple(bank.get_family())
        listeners = tuple(self.n.listeners)
        centers = [c.slider_axis.point.copy() for c in module._channels(bank)]
        self.assertIsNone(bank.set_value(10., 20., 30., .4))
        self.assertEqual(tuple(bank.get_family()), objects)
        self.assertEqual(tuple(self.n.listeners), listeners)
        for c, point in zip(module._channels(bank), centers):
            np.testing.assert_equal(c.slider_axis.point, point)
        np.testing.assert_allclose(bank.selected_color_box.style["color"], [10/255., 20/255., 30/255.])
        self.assertEqual(bank.selected_color_box.style["opacity"], .4)
    def test_set_delegates_normalization_and_uses_its_result(self):
        bank = self.n.ColorSliders()
        self.n.normalizer = lambda values: (12., 34., 56., .78)
        bank.set_value(-3., 900., 21., .1)
        self.assertIn(((-3., 900., 21., .1), True), self.n.native_calls)
        np.testing.assert_allclose(bank.get_value(), [12/255., 34/255., 56/255., .78])
    def test_native_rejection_does_not_partially_set_channels(self):
        bank = self.n.ColorSliders()
        before = bank.get_value().copy()
        def reject(values):
            raise ValueError("native refused")
        self.n.normalizer = reject
        with self.assertRaisesRegex(ValueError, "native refused"):
            bank.set_value(1., 2., 3., float("nan"))
        np.testing.assert_equal(bank.get_value(), before)
    def test_all_authored_validators_precede_first_setter(self):
        bank = self.n.ColorSliders()
        before = bank.get_value().copy()
        def reject(value):
            raise LookupError("last validator")
        bank.a_slider.assert_value = reject
        with self.assertRaisesRegex(LookupError, "last validator"):
            bank.set_value(1., 2., 3., .5)
        np.testing.assert_equal(bank.get_value(), before)
    def test_parent_paint_observes_child_updater_same_tick(self):
        bank = self.n.ColorSliders()
        bank.r_slider.updaters.append(lambda channel: channel.set_value(31.))
        bank.update(.1)
        self.assertEqual(bank.selected_color_box.style["color"][0], 31/255.)
    def test_suspend_then_resume_preserves_live_paint(self):
        bank = self.n.ColorSliders()
        bank.suspended = True
        bank.r_slider.set_value(0.)
        bank.update(.1)
        self.assertEqual(bank.selected_color_box.style["color"][0], 1.)
        bank.suspended = False
        bank.update(0.)
        self.assertEqual(bank.selected_color_box.style["color"][0], 0.)
    def test_paint_calls_public_overrides(self):
        Base = self.n.ColorSliders
        class Authored(Base):
            def get_picked_color(self):
                return "authored-color"
            def get_picked_opacity(self):
                return .125
        bank = Authored()
        self.assertEqual(bank.selected_color_box.style, {"color": "authored-color", "opacity": .125})
    def test_copied_paint_uses_its_receiver(self):
        bank = self.n.ColorSliders()
        other = copy.deepcopy(bank)
        other.r_slider.set_value(0.)
        other.update(0.)
        self.assertEqual(other.selected_color_box.style["color"][0], 0.)
        self.assertEqual(bank.selected_color_box.style["color"][0], 1.)
    def test_background_is_fresh_native_geometry_at_live_swatch(self):
        bank = self.n.ColorSliders().shift(np.array([2., 4., 0.]))
        first, second = bank.get_background(), bank.get_background()
        self.assertIsNot(first, second)
        self.assertIsNot(first, bank.background)
        self.assertTrue(first.native_grid)
        self.assertTrue(first.fixed)
        np.testing.assert_equal(first.point, bank.selected_color_box.point)
    def test_typed_background_preserves_scalar_rgb_precision_and_opacity(self):
        native = typed_background_environment()
        bank = native.ColorSliders.__new__(native.ColorSliders)
        native.Mobject.__init__(bank)
        bank.rect_kwargs = {"width": 1., "height": .5}
        bank.background_grid_kwargs = {"single_square_len": .25}
        colors = np.array([[.123456789, .345678912, .789123456],
                           [.876543219, .654321987, .210987654]])
        bank._background_grid_colors = tuple(tuple(color) for color in colors)
        bank.selected_color_box = native.Mobject().shift(np.array([2., 3., 0.]))
        first, second = bank.get_background(), bank.get_background()
        self.assertIsNot(first, second)
        for background in (first, second):
            self.assertIs(type(background), native.VGroup)
            self.assertEqual(len(background.submobjects), 10)
            np.testing.assert_array_equal(background.point, bank.selected_color_box.point)
            for index, square in enumerate(background):
                self.assertIs(type(square), native.Square)
                rgba = square.data["fill_rgba"]
                np.testing.assert_allclose(rgba[:, :3],
                                           np.tile(colors[index % 2], (len(rgba), 1)),
                                           rtol=0, atol=1e-12)
                np.testing.assert_array_equal(rgba[:, 3], np.ones(len(rgba)))
                self.assertTrue(square.fixed)
    def test_background_override_controls_constructor_product(self):
        Base = self.n.ColorSliders
        chosen = self.n.Mobject()
        class Authored(Base):
            def get_background(self):
                return chosen
        bank = Authored()
        self.assertIs(bank.background, chosen)
        self.assertIs(bank.swatch.submobjects[0], chosen)
    def test_failure_after_channel_construction_disconnects_new_listeners(self):
        Base = self.n.ColorSliders
        unrelated = SimpleNamespace(event_type="other", callback=lambda: None)
        self.n.listeners.append(unrelated)
        class Broken(Base):
            def get_background(self):
                raise ValueError("background failed")
        with self.assertRaisesRegex(ValueError, "background failed"):
            Broken()
        self.assertEqual(self.n.listeners, [unrelated])
        self.assertTrue(all(not c.slider.event_listners for c in self.n.created))
    def test_invalid_background_is_named_and_disconnects(self):
        Base = self.n.ColorSliders
        class Broken(Base):
            def get_background(self):
                return None
        with self.assertRaisesRegex(TypeError, "get_background"):
            Broken()
        self.assertEqual(self.n.listeners, [])
    def test_installer_is_idempotent_preserves_class_and_public_metadata(self):
        cls, init = self.n.ColorSliders, self.n.ColorSliders.__init__
        module.install_color_sliders(self.n)
        self.assertIs(self.n.ColorSliders, cls)
        self.assertIs(self.n.ColorSliders.__init__, init)
        self.assertEqual(cls.get_value.__name__, "get_value")
        self.assertFalse(getattr(cls.get_background, "_fmn_schema_placeholder", False))


if __name__ == "__main__":
    unittest.main()
