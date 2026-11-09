"""Native rich typography without SVG round-trips or rewritten source spans."""
from __future__ import annotations

from collections.abc import Mapping
from functools import wraps
import inspect
import itertools
import math

_MAX_BYTES = 262_144
_MAX_ITEMS = 4096


def _number(value, name, *, positive=False):
    result = float(value)
    if not math.isfinite(result) or (positive and result <= 0) or abs(result) > 3.4028234663852886e38:
        raise ValueError(name + " must be finite, f32-representable" + (" and positive" if positive else ""))
    return result


def _style_flag(value, name, positive):
    if not isinstance(value, str):
        raise TypeError(name + " must be a style name")
    normalized = value.upper()
    if normalized not in ("NORMAL", positive):
        raise ValueError(name + " supports NORMAL or " + positive)
    return normalized == positive


def _map(long, short, name):
    selected = long or short
    if not isinstance(selected, Mapping):
        raise TypeError(name + " must be a mapping")
    if len(selected) > _MAX_ITEMS:
        raise ValueError(name + " exceeds the 4096-entry budget")
    items = list(itertools.islice(selected.items(), _MAX_ITEMS + 1))
    if len(items) > _MAX_ITEMS:
        raise ValueError(name + " exceeds the 4096-entry budget")
    return dict(items)


def _stops(g, values, name):
    if isinstance(values, (str, bytes)):
        raise TypeError(name + " must be an iterable of colors, not one color string")
    stops = tuple(itertools.islice(iter(values), _MAX_ITEMS + 1))
    if len(stops) > _MAX_ITEMS:
        raise ValueError(name + " exceeds the 4096-stop budget")
    # Freeze colors too; caller-owned arrays cannot change during construction.
    result = tuple(tuple(float(v) for v in g["color_to_rgb"](stop)) for stop in stops)
    if any(len(row) != 3 or not all(math.isfinite(v) and 0 <= v <= 1 for v in row) for row in result):
        raise ValueError(name + " colors must have finite RGB components in [0, 1]")
    return result


def install_text_authoring(native):
    g = vars(native)
    if g.get("_FMN_TEXT_AUTHORING_INSTALLED", False):
        return
    Markup = g["MarkupText"]
    from .string_lifecycle import initialize_string, explicit_channel_colors
    previous = Markup.__init__
    signature = inspect.signature(previous)

    @wraps(previous)
    def initialize(self, *args, **kwargs):
        bound = signature.bind(self, *args, **kwargs)
        bound.apply_defaults()
        p = dict(bound.arguments)
        p.pop("self")
        style = explicit_channel_colors(p.pop("kwargs"))
        source = str(p["text"])
        if len(source.encode("utf-8")) > _MAX_BYTES:
            raise ValueError("Text source exceeds 262144 bytes")
        if p["global_config"] or p["local_configs"]:
            raise NotImplementedError("Text global_config/local_configs require unsupported Pango attributes; use native font/style maps")
        font = p["font"]
        if not isinstance(font, str):
            raise TypeError("Text font must be a family name")
        alignment = p["alignment"].upper() if isinstance(p["alignment"], str) else None
        if alignment not in ("", "LEFT", "CENTER", "RIGHT"):
            raise ValueError("Text alignment must be LEFT, CENTER or RIGHT")
        spacing = p["line_spacing_height"] if p["line_spacing_height"] is not None else p["lsh"]
        spacing = 1. if spacing is None else _number(spacing, "Text line spacing", positive=True)
        maps = {name: _map(p[long], p[name], name) for name, long in (
            ("t2c", "text2color"), ("t2f", "text2font"), ("t2s", "text2slant"),
            ("t2w", "text2weight"), ("t2g", "text2gradient"),
        )}
        for name in ("t2f", "t2s", "t2w", "t2g"):
            if any(not isinstance(key, str) for key in maps[name]):
                raise TypeError(name + " currently requires literal source-string keys")
        if any(not isinstance(value, str) for value in maps["t2f"].values()):
            raise TypeError("t2f values must be font family names")
        gradients = [(key, _stops(g, value, "t2g")) for key, value in maps["t2g"].items()]
        if sum(len(stops) for _, stops in gradients) > _MAX_ITEMS:
            raise ValueError("Text gradients exceed 4096 total stops")
        gradient = None if p["gradient"] is None else _stops(g, p["gradient"], "gradient")
        options = dict(font=font, font_size=_number(p["font_size"], "Text font_size", positive=True),
                       bold=_style_flag(p["weight"], "Text weight", "BOLD"),
                       italic=_style_flag(p["slant"], "Text slant", "ITALIC"),
                       alignment=alignment or "LEFT", line_spacing=spacing,
                       justify=bool(p["justify"]), indent=_number(p["indent"], "Text indent"),
                       disable_ligatures=bool(p["disable_ligatures"]),
                       t2f=list(maps["t2f"].items()),
                       t2s=[(key, _style_flag(value, "t2s", "ITALIC")) for key, value in maps["t2s"].items()],
                       t2w=[(key, _style_flag(value, "t2w", "BOLD")) for key, value in maps["t2w"].items()],
                       t2g=gradients)
        code_options = vars(self).get("_fmn_code_layout")
        if isinstance(self, g["Code"]) and code_options is not None:
            options["code_language"], options["code_style"] = code_options
        if p["line_width"] is not None:
            options["line_width"] = _number(p["line_width"], "Text line_width", positive=True)
        height = None if p["height"] is None else _number(p["height"], "Text height", positive=True)
        labelled = bool(style.pop("use_labelled_svg", False))
        centered = style.pop("should_center", True)
        style.pop("path_string_config", None)
        base_color = style.pop("base_color", "#FFFFFF")
        protect = style.pop("protect", ())
        g["_preflight_vmobject_style_kwargs"](style)
        self.text = self.string = source
        self.font_size, self.font = options["font_size"], font
        self.weight, self.slant = p["weight"], p["slant"]
        self.alignment, self.line_width = p["alignment"], p["line_width"]
        self.justify, self.indent, self.lsh = options["justify"], options["indent"], spacing
        self.global_config, self.local_configs = {}, {}
        self.disable_ligatures, self.isolate = options["disable_ligatures"], p["isolate"]
        self.use_labelled_svg, self.base_color, self.protect = labelled, base_color, protect
        for name, value in maps.items():
            setattr(self, name, value)
        self._fmn_text_options = options
        initialize_string(g, self, style)
        if gradient:
            self.set_color_by_gradient(*gradient)
        # Existing selectors retain regex/Unicode/byte-span behavior and win
        # over a whole-object gradient, as on the public authoring surface.
        self.set_color_by_text_to_color_map(self.t2c)
        if centered:
            self.center()
        if height is not None:
            self.set_height(height)

    Markup.__init__ = initialize

    def get_part_by_text(self, selector, *args, **kwargs):
        # This is an alias of select_part, not a second selector. Forward the
        # occurrence index positionally or by keyword and retain authored
        # select_part overrides, including their optional extension keywords.
        return self.select_part(selector, *args, **kwargs)

    get_part_by_text.__qualname__ = Markup.__qualname__ + ".get_part_by_text"
    get_part_by_text.__module__ = Markup.__module__
    Markup.get_part_by_text = get_part_by_text
    _install_code(g)
    g["_FMN_TEXT_AUTHORING_INSTALLED"] = True


def _install_code(g):
    Code = g["Code"]
    previous = Code.__init__
    missing = object()

    @wraps(previous)
    def initialize(self, code, font="Consolas", font_size=24, lsh=1.0,
                   fill_color=None, stroke_color=None, language="python",
                   code_style="monokai", **kwargs):
        if not isinstance(font, str):
            raise TypeError("Code font must be a family name")
        if not isinstance(language, str) or not isinstance(code_style, str):
            raise TypeError("Code language and code_style must be strings")
        if len(language.encode("utf-8")) > 1024 or len(code_style.encode("utf-8")) > 1024:
            raise ValueError("Code language/style names exceed 1024 bytes")
        # The legacy default name is an explicit Code-only compatibility
        # policy, not OS font discovery. Record the real bundled family.
        native_font = "CM Typewriter" if font in ("", "Consolas") else font
        source = str(code)
        if fill_color is not None:
            kwargs["fill_color"] = fill_color
        if stroke_color is not None:
            kwargs["stroke_color"] = stroke_color
        attrs = vars(self)
        old = attrs.get("_fmn_code_layout", missing)
        attrs["_fmn_code_layout"] = (language, code_style)
        try:
            super(Code, self).__init__(source, font=native_font, font_size=font_size,
                                      lsh=lsh, **kwargs)
        finally:
            if old is missing:
                attrs.pop("_fmn_code_layout", None)
            else:
                attrs["_fmn_code_layout"] = old
        self.code, self.font, self.native_font = source, font, native_font
        self.language, self.code_style = language, code_style

    # Code is a StringMobject with one live family of glyphs. The earlier
    # scaffold duplicated all child records into the parent, double-rendering
    # translucent ink and leaving stale copies after selector-based edits.
    Code._hoist_descendant_records = False
    Code.__init__ = initialize
    Code.__doc__ = (
        "Syntax-highlighted literal source over native Scribe glyphs. "
        "The existing native highlighter supplies positional token colors; "
        "Code's legacy Consolas default selects the bundled CM Typewriter. "
        "native_font records that actual choice. Unknown languages use plain "
        "native text, and unsupported theme/font names refuse."
    )
