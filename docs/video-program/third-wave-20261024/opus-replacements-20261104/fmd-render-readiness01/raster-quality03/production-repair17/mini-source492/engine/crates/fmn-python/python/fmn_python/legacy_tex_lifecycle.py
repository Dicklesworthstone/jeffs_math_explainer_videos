"""Legacy single-string math on Scribe and the shared glyph publisher.

The public expression normalizer remains authored Python. Native typesetting
owns outlines and UTF-8 spans; VMobject owns records and lifecycle dispatch.
No SVG/LaTeX round trip, alternate font engine or new animation clock is used.
"""
from __future__ import annotations

from functools import wraps
import math

from .invocation import InvocationGuard
from .string_lifecycle import initialize_string, _publish

_MAX_BYTES = 262_144


def install_legacy_tex_lifecycle(native):
    g = vars(native)
    if g.get("_FMN_LEGACY_TEX_LIFECYCLE_INSTALLED", False):
        return
    # This legacy class is qualified-only in the pinned export schema.
    # Bootstrap has already populated the per-interpreter module tree.
    module = g["_sys"].modules["manimlib.mobject.svg.old_tex_mobject"]
    Single, VMobject = module.SingleStringTex, g["VMobject"]
    np = g["_np"]
    constructors, rebuilds = InvocationGuard(), InvocationGuard()
    previous = Single.__init__

    def scalar(value, name, *, positive=False):
        value = float(value)
        if (not math.isfinite(value) or abs(value) > np.finfo(np.float32).max
                or (positive and value <= 0)):
            raise ValueError(name + " must be finite and f32-representable" +
                             (" and positive" if positive else ""))
        return value

    def source_text(value):
        if not isinstance(value, str):
            raise TypeError("SingleStringTex expression must be a string")
        if len(value.encode("utf-8")) > _MAX_BYTES:
            raise ValueError("SingleStringTex expression exceeds 262144 bytes")
        return value

    def idle(self):
        if vars(self).get("_is_animating", False) or getattr(self, "locked_data_keys", ()):
            raise RuntimeError("release the text's active animation before regenerating its glyphs")

    def recipe(self):
        text = source_text(self.tex_string)
        size = scalar(self.font_size, "SingleStringTex font_size", positive=True)
        mode = bool(self.math_mode)
        g["_validate_tex_options"](self.template, self.additional_preamble, not mode)
        alignment = g["_tex_line_align"](self.alignment, not mode)
        return text, size, mode, self.template, self.additional_preamble, alignment

    @wraps(previous)
    def initialize(self, tex_string, height=None, fill_color=g["_StyleDefault"](g["_WHITE"]), fill_opacity=1.0,
                   stroke_width=0, svg_default={"fill_color": g["_WHITE"]},
                   path_string_config={}, font_size=48, alignment=r"\centering",
                   math_mode=True, organize_left_to_right=False, template="",
                   additional_preamble="", **kwargs):
        if self._is_bound():
            raise RuntimeError("a text constructor requires a detached target; use init_points")
        idle(self)
        text = source_text(str(tex_string))
        size = scalar(font_size, "SingleStringTex font_size", positive=True)
        selected_height = None if height is None else scalar(height, "SingleStringTex height")
        mode = bool(math_mode)
        g["_validate_tex_options"](template, additional_preamble, not mode)
        g["_tex_line_align"](alignment, not mode)
        style = dict(kwargs, fill_color=fill_color,
                     fill_opacity=(None if fill_opacity is None else
                                   scalar(fill_opacity, "SingleStringTex fill_opacity")),
                     stroke_width=(None if stroke_width is None else
                                   scalar(stroke_width, "SingleStringTex stroke_width")))
        # Scribe glyphs use a white stroke and a half-pixel fill border;
        # VMobject's generic grey/zero defaults must not change legacy ink.
        # The colors are constructor defaults: a caller's color= beats them
        # (BN-07 C-19, fm-qead).
        white = g["_StyleDefault"](g["_WHITE"])
        for key, fallback in (("fill_color", white), ("stroke_color", white),
                              ("fill_border_width", 0.5), ("stroke_width", 0.0)):
            if style.get(key) is None:
                style[key] = fallback
        for key in ("fill_opacity", "stroke_opacity"):
            if style.get(key) is None:
                opacity = style.get("opacity")
                style[key] = 1.0 if opacity is None else scalar(opacity, "SingleStringTex opacity")
        g["_preflight_vmobject_style_kwargs"](style)
        svg_default, path_string_config = dict(svg_default), dict(path_string_config)
        with constructors.hold(self, message="legacy text initialization is already in progress"):
            self.tex_string = text
            self.svg_default, self.path_string_config = svg_default, path_string_config
            self.font_size, self.alignment, self.math_mode = size, alignment, mode
            self.organize_left_to_right = bool(organize_left_to_right)
            self.template, self.additional_preamble = template, additional_preamble
            self.base_color = g["_WHITE"]
            initialize_string(g, self, style)
            if selected_height is not None:
                self.set_height(selected_height)
            if self.organize_left_to_right:
                self.organize_submobjects_left_to_right()

    def init_points(self):
        with rebuilds.hold(self, message="legacy text regeneration is already in progress"):
            idle(self)
            selected = recipe(self)
            normalizer = self.get_modified_expression
            identity = (getattr(normalizer, "__func__", normalizer),
                        getattr(normalizer, "__self__", None))
            owner, bound = vars(self).get("_scene"), self._is_bound()
            before, children = self.data.copy(), tuple(self.submobjects)
            # Read the existing style before calling authored expression code;
            # subsequent regeneration retains explicit live whole-object edits.
            style = None if constructors.busy(self) else self.get_style()
            source = source_text(normalizer(selected[0]))
            candidate = g["_native_shell_factory"]()
            specs = candidate._build_tex(
                g["_native_shell_factory"], [source], "", not selected[2],
                selected[1], None, False, selected[3], selected[4], selected[5])
            if style is not None:
                # Stage the complete styled family before writing the receiver.
                g["_hang_native_children"](candidate, specs)
                candidate.set_style(**style)
                # This candidate is already assembled; the shared publisher
                # must not attach these same children a second time.
                specs = ()
            idle(self)
            current = self.get_modified_expression
            if (vars(self).get("_scene") is not owner or self._is_bound() != bound
                    or tuple(self.submobjects) != children
                    or self.data.dtype != before.dtype or self.data.tobytes() != before.tobytes()
                    or recipe(self) != selected
                    or getattr(current, "__func__", current) is not identity[0]
                    or getattr(current, "__self__", None) is not identity[1]):
                raise RuntimeError("legacy text changed during typesetting; glyphs were not published")
            _publish(g, self, candidate, specs)
        return self

    def init_colors(self):
        # SingleStringTex is a VMobject, not modern Tex: the explicit legacy
        # constructor fill/stroke settings apply to the completed glyph family.
        return VMobject.init_colors(self)

    for name, function in (("__init__", initialize), ("init_points", init_points),
                           ("init_colors", init_colors)):
        function.__name__ = name
        function.__qualname__ = Single.__qualname__ + "." + name
        function.__module__ = Single.__module__
        setattr(Single, name, function)
    g["_FMN_LEGACY_TEX_LIFECYCLE_INSTALLED"] = True
