"""Live color controls over Atlas geometry and Marionette scalar trackers.

Atlas remains the authority for the initial layout and bank-level value
normalization. Real LinearNumberSlider children own subsequent edits; neither
input dispatch nor the frame loop is reimplemented here.
"""
from __future__ import annotations

from functools import wraps
import inspect
import math
from typing import Any


_CHANNEL_NAMES = ("r_slider", "g_slider", "b_slider", "a_slider")


def _channels(bank):
    return tuple(getattr(bank, name) for name in _CHANNEL_NAMES)


def _refresh_swatch(bank):
    # This callback uses its receiver, not a closed-over original bank. The
    # existing graph copier remaps the bank's direct Mobject attributes, so a
    # copied bank paints its own swatch from its own channel trackers.
    bank.selected_color_box.set_fill(
        color=bank.get_picked_color(), opacity=bank.get_picked_opacity(),
    )


def _channel_config(bank, index, defaults):
    config = dict(bank.sliders_kwargs)
    # Bank options are overrides of Atlas's slider defaults, not complete
    # primitive recipes. Fill absent keys from the public slider signature
    # before calling it: an empty Circle recipe otherwise means radius=1.
    for key in ("rounded_rect_kwargs", "circle_kwargs"):
        config[key] = dict(defaults.get(key, {}), **config.get(key, {}))
    config.setdefault("min_value", 0.0)
    config.setdefault("max_value", 1.0 if index == 3 else 255.0)
    step = config.get("step", 0.04 if index == 3 else 1.0)
    # Atlas ColorSliders::slider_overrides deliberately keeps alpha's usable
    # default when a shared RGB step exceeds the alpha span. This selects an
    # existing slider configuration; snapping itself stays in the native bank.
    if index == 3 and float(step) > float(config["max_value"]) - float(config["min_value"]):
        step = 0.04
    config["step"] = step
    config["value_type"] = bank._slider_value_type
    return config


def _normalize(bank, values):
    components = tuple(float(bank._slider_value_type(value)) for value in values)
    # Reuse the existing all-channel native validation, clamping and optional
    # shared-step snapping. Its temporary geometry is NOT installed into the
    # live bank; positioned controls and their listeners must remain intact.
    _, _, normalized = bank._native_color_slider_parts(components, apply_value=True)
    if len(normalized) != 4:
        raise RuntimeError("native ColorSliders must return four channel values")
    return tuple(normalized)


def _disconnect(controls, original):
    for control in reversed(controls):
        for member in control.get_family():
            for listener in tuple(member.get_event_listners()):
                try:
                    member.remove_event_listner(listener.event_type, listener.callback)
                except BaseException as error:
                    try:
                        original.add_note("color slider listener cleanup failed: " + type(error).__name__)
                    except BaseException:
                        pass


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def install_color_sliders(native: Any) -> None:
    """Bind independent channels without replacing any published class object.

    Install after install_control_events so each newly constructed channel
    uses the same registered drag handler as an ordinary LinearNumberSlider.
    Direct extension embeddings can use the same installers explicitly.
    """
    g = vars(native)
    if g.get("_FMN_COLOR_SLIDERS_INSTALLED", False):
        return
    Bank, Group, Slider = g["ColorSliders"], g["Group"], g["LinearNumberSlider"]
    slider_signature = inspect.signature(Slider)
    defaults = {key: dict(parameter.default)
                for key, parameter in slider_signature.parameters.items()
                if key in ("rounded_rect_kwargs", "circle_kwargs")}
    original_init = Bank.__init__
    original_get_value = Bank.get_value
    np = g["_np"]
    typed_shapes = all(name in g for name in ("Rectangle", "Square", "VGroup", "_ColorValue"))

    @wraps(original_init)
    def initialize(self, *args, **kwargs):
        original_init(self, *args, **kwargs)
        templates = tuple(self.sliders.submobjects)
        if len(templates) != 4 or any(len(template.submobjects) != 3 for template in templates):
            raise RuntimeError("native ColorSliders needs four bar/handle/axis compositions")
        values = _normalize(self, self._color_slider_components)
        created = []
        try:
            for index, (template, value) in enumerate(zip(templates, values)):
                channel = Slider(value=value, **_channel_config(self, index, defaults))
                created.append(channel)
                # Atlas's bank builder owns layout and the RGB/alpha handle
                # palette. Match axis centers, not bounding-box centers that
                # move when the handle leaves the constructor's midpoint.
                channel.slider.match_style(template.submobjects[1])
                channel.shift(template.submobjects[2].get_center() - channel.slider_axis.get_center())
                # Correct the Reference's parked-at-midpoint constructor wart:
                # the visible handle must show the channel's actual value.
                channel.set_value(value)
            # Scalar controls are Mobjects, not VMobjects. A real Group is
            # required rather than stuffing them into the native vector shell.
            self.sliders = Group(*created)
            for name, channel in zip(_CHANNEL_NAMES, created):
                setattr(self, name, channel)
            self._color_slider_components = values
            self._fmn_color_channels_ready = True
            if typed_shapes:
                # Public constructors retain their normal hooks and methods;
                # the native bank supplies placement, not anonymous children.
                selection = g["Rectangle"](
                    stroke_color=g["WHITE"], **self.rect_kwargs,
                )
                selection.move_to(self.selected_color_box)
                self.selected_color_box = selection
            background = self.get_background()
            if not isinstance(background, g["Mobject"]):
                raise TypeError("ColorSliders.get_background must return a Mobject")
            self.background = background
            self.swatch = Group(background, self.selected_color_box)
            self.set_submobjects([self.swatch, self.sliders])
            self.fix_in_frame()
            # Parent updater order observes every channel's completed update,
            # including .animate and authored child updaters in Scene.wait.
            self.add_updater(_refresh_swatch)
        except BaseException as error:
            _disconnect(created, error)
            raise

    def get_value(self):
        if not getattr(self, "_fmn_color_channels_ready", False):
            # Cooperative constructors may inspect the initial native bank
            # before its live scalar children have been attached.
            return original_get_value(self)
        red, green, blue, alpha = (channel.get_value() for channel in _channels(self))
        return np.array([red / 255.0, green / 255.0, blue / 255.0, alpha])

    def set_value(self, r, g, b, a):
        values = _normalize(self, (r, g, b, a))
        channels = _channels(self)
        # Do not partially apply an ordinary invalid bank update. Authored
        # validators run before the first setter; arbitrary user code that
        # mutates and then raises cannot be transactionally rolled back here.
        for channel, value in zip(channels, values):
            channel.assert_value(value)
        for channel, value in zip(channels, values):
            channel.set_value(value)
        self._color_slider_components = values
        _refresh_swatch(self)
        # The Reference's aggregate setter has no fluent return.

    def get_background(self):
        if typed_shapes:
            length = float(self.background_grid_kwargs["single_square_len"])
            width, height = float(self.rect_kwargs["width"]), float(self.rect_kwargs["height"])
            if not all(math.isfinite(value) and value > 0 for value in (length, width, height)):
                raise ValueError("ColorSliders grid dimensions must be finite and positive")
            counts = height / length, width / length
            if not all(math.isfinite(value) and value <= 65536 for value in counts):
                raise ValueError("ColorSliders background exceeds 65536 squares")
            rows, columns = (max(1, int(value)) for value in counts)
            if columns % 2 == 0:
                columns += 1
            if rows * columns > 65536:
                raise ValueError("ColorSliders background exceeds 65536 squares")
            # The bank stores each validated grid color as an RGB triple. A
            # bare triple passed to set_fill is a three-color gradient of
            # scalars, so each one becomes an exact single color value without
            # quantizing through hex.
            colors = tuple(g["_ColorValue"](rgb) for rgb in self._background_grid_colors)
            if not colors:
                raise ValueError("ColorSliders background colors must not be empty")
            background = g["Square"](length).get_grid(n_rows=rows, n_cols=columns, buff=0.)
            background.stretch_to_fit_width(width)
            background.stretch_to_fit_height(height)
            for index, square in enumerate(background):
                square.set_stroke(width=0., opacity=0.)
                square.set_fill(colors[index % len(colors)], opacity=1.)
        else:
            # Reduced orchestration embeddings provide their own geometry.
            values = tuple(channel.get_value() for channel in _channels(self))
            swatch, _, _ = self._native_color_slider_parts(values, apply_value=True)
            background = swatch.submobjects[0]
        background.move_to(self.selected_color_box)
        background.fix_in_frame()
        return background

    for name, function in {
        "__init__": initialize, "get_value": get_value,
        "set_value": set_value, "get_background": get_background,
    }.items():
        _method(Bank, name, function)
    _install_panel_content(g)
    g["_FMN_COLOR_SLIDERS_INSTALLED"] = True


def _install_panel_content(g):
    """Allow composite controls on the existing native panel layout path."""
    import inspect

    Panel = g.get("ControlPanel")
    if Panel is None:
        return
    original = Panel.__init__
    signature = inspect.signature(original)
    Group, ScalarControl, Bank = g["Group"], g["ControlMobject"], g["ColorSliders"]
    rectangle_keys = {"width", "height", "color", "fill_color", "fill_opacity",
                      "stroke_color", "stroke_width", "stroke_opacity"}

    def config(value, name, allowed):
        result = dict(value)
        unknown = sorted(set(result) - allowed)
        if unknown:
            raise TypeError("unexpected keyword arguments: " + ", ".join(name + "." + key for key in unknown))
        return result

    def is_control(control):
        if isinstance(control, (ScalarControl, Bank)):
            return True
        # A composite can include labels/decorations alongside its controls.
        # A bare shape or an empty/decorative group is still not a control;
        # keep the pre-existing refusal and its public diagnostic intact.
        return isinstance(control, Group) and any(
            isinstance(member, (ScalarControl, Bank))
            for member in control.get_family()[1:]
        )

    def validate(controls):
        if not all(is_control(control) for control in controls):
            raise TypeError("ControlPanel controls must be ControlMobject instances")

    if all(name in g for name in ("Rectangle", "Text", "_NATIVE_VMOBJECT_STYLE_KEYS")):
        original_parts = Panel._native_control_panel_parts
        from .control_events import _shape

        @wraps(original_parts)
        def typed_parts(self, opener_text, opener_font_size, controls, *, open):
            targets = original_parts(self, opener_text, opener_font_size, controls, open=open)
            if getattr(self, "panel", None) is not None:
                # Open/close only consumes placement targets. Preserve the
                # already-authored panel, label, and their event listeners.
                return targets
            panel_target, opener_target, _ = targets
            panel_style = dict(width=g["_FRAME_SHAPE"][0] / 4,
                               height=g["_MED_SMALL_BUFF"] + g["_FRAME_HEIGHT"],
                               fill_opacity=1., stroke_width=0.)
            opener_style = dict(width=g["_FRAME_SHAPE"][0] / 8, height=.5, fill_opacity=1.)
            for recipe, values in ((panel_style, self.panel_kwargs),
                                   (opener_style, self.opener_kwargs)):
                recipe.update(values)
                recipe.setdefault("fill_color", recipe.get("color", g["_GREY_C"]))
                recipe.setdefault("stroke_color", recipe.get("color", g["_GREY_A"]))
            panel = _shape(g, "Rectangle", panel_style, "panel_kwargs")
            opener_rect = _shape(g, "Rectangle", opener_style, "opener_kwargs")
            text_style = dict(self.opener_text_kwargs)
            text_style.pop("text", None)
            text_style["font_size"] = opener_font_size
            label = g["Text"](opener_text, **text_style)
            panel.move_to(panel_target)
            opener_rect.move_to(opener_target[0])
            label.move_to(opener_rect)
            return panel, Group(opener_rect, label), Group(*controls)

        _method(Panel, "_native_control_panel_parts", typed_parts)

    @wraps(original)
    def initialize(self, *controls, **kwargs):
        # Preserve the established scalar-only path and its authored hooks.
        if all(isinstance(control, ScalarControl) for control in controls):
            return original(self, *controls, **kwargs)
        validate(controls)
        bound = signature.bind(self, *controls, **kwargs)
        bound.apply_defaults()
        args = bound.arguments
        extra = args.get("kwargs", {})
        if extra:
            raise TypeError("unexpected keyword arguments: " + ", ".join(sorted(extra)))
        self.panel_kwargs = config(args["panel_kwargs"], "panel_kwargs", rectangle_keys)
        self.opener_kwargs = config(args["opener_kwargs"], "opener_kwargs", rectangle_keys)
        self.opener_text_kwargs = config(args["opener_text_kwargs"], "opener_text_kwargs",
                                         {"text", "font_size", "color", "fill_color"})
        self._control_panel_open = False
        text = self.opener_text_kwargs
        panel, opener, content = self._native_control_panel_parts(
            str(text.get("text", "Control Panel")), float(text.get("font_size", 20)),
            controls, open=False,
        )
        # This helper computes native extents/layout and grafts the original
        # controls; no scalar proxy/wrapper or replacement layout is needed.
        super(Panel, self).__init__(panel, opener, content)
        self.panel, self.panel_opener, self.controls = panel, opener, content
        self.panel_opener_rect, self.panel_info_text = opener.submobjects
        self.fix_in_frame()
        from .control_events import _connect

        _connect(g, self, (
            ("panel", "MouseScrollEvent", "add_mouse_scroll_listner", "panel_on_mouse_scroll"),
            ("panel_opener", "MouseDragEvent", "add_mouse_drag_listner", "panel_opener_on_mouse_drag"),
        ))

    def add_controls(self, *new_controls):
        validate(new_controls)
        self.controls.add(*new_controls)
        self.move_panel_and_controls_to_panel_opener()

    _method(Panel, "__init__", initialize)
    _method(Panel, "add_controls", add_controls)
