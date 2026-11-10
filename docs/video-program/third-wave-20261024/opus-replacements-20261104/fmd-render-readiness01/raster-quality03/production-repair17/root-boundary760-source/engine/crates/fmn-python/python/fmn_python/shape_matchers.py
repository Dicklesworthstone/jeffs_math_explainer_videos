"""Shape-matcher lifecycle over the existing native-backed Rectangle protocol.

A surrounding box is an authored Rectangle that is resized and positioned,
not a disposable native tree. Keep its record schema, children, styles and
public hooks when following a moving target or changing its padding.
"""
from __future__ import annotations

import math
from itertools import islice
from numbers import Real


def install_shape_matchers(native):
    g = vars(native)
    if g.get("_FMN_SHAPE_MATCHERS_INSTALLED", False):
        return
    Surrounding = g["SurroundingRectangle"]
    Mobject = g["_BridgeMobject"]

    def require_target(mobject, operation):
        if not isinstance(mobject, Mobject):
            raise TypeError(operation + " expects a Mobject")

    def padding(value):
        value = float(value)
        if not math.isfinite(value):
            raise ValueError("surrounding rectangle buff must be finite")
        return value

    def surrounding_init(self, mobject, buff=0.1, color=g["_YELLOW"], **kwargs):
        require_target(mobject, "SurroundingRectangle")
        buff = padding(buff)
        # Preserve the portal's established fallback policy: explicit channel
        # colors win over the surrounding box's constructor color.
        if kwargs.get("fill_color") is None:
            kwargs["fill_color"] = color
        if kwargs.get("stroke_color") is None:
            kwargs["stroke_color"] = color
        # Rectangle owns the native geometry recipe and the single ordinary
        # init_data -> init_points -> init_uniforms -> init_colors dispatch.
        super(Surrounding, self).__init__(**kwargs)
        self.buff = buff
        self.surround(mobject)
        if mobject.is_fixed_in_frame():
            self.fix_in_frame()

    def surround(self, mobject, buff=None):
        require_target(mobject, "SurroundingRectangle.surround")
        buff = padding(self.buff if buff is None else buff)
        self.mobject = mobject
        self.buff = buff
        if not mobject.family_members_with_points():
            # Native annotations draw nothing for an empty target. Retain the
            # actual authored root records so a later target can restore them.
            if self.has_points():
                self._annotation_empty_data = self.data.copy()
                self.clear_points()
            return self
        saved = vars(self).pop("_annotation_empty_data", None)
        if saved is not None and not self.has_points():
            self.set_data(saved)
        # This is Rectangle.surround's native set_shape/move_to operation on
        # the existing family. Rebuilding a fresh rectangle here destroys a
        # hook-authored outline or gradient. Reapplying get_style() also calls
        # BackgroundRectangle.set_style, which deliberately forces black; a
        # size change must not accidentally become that explicit style edit.
        super(Surrounding, self).surround(mobject, self.buff)
        return self

    def set_buff(self, buff):
        self.buff = padding(buff)
        # Reference set_buff calls the public hook with just the mobject. A
        # subclass accepting only that positional argument remains legal.
        self.surround(self.mobject)
        return self

    for name, function in (("__init__", surrounding_init), ("surround", surround),
                           ("set_buff", set_buff)):
        function.__name__ = name
        function.__qualname__ = Surrounding.__qualname__ + "." + name
        function.__module__ = Surrounding.__module__
        setattr(Surrounding, name, function)
    g["_FMN_SHAPE_MATCHERS_INSTALLED"] = True


def _finite(value, name):
    value = float(value)
    if not math.isfinite(value) or abs(value) > 3.4028234663852886e38:
        raise ValueError(name + " must be finite and f32-representable")
    return value


def _width(value):
    if isinstance(value, Real):
        return _finite(value, "annotation stroke_width")
    widths = tuple(islice(iter(value), 4097))
    if not widths or len(widths) > 4096:
        raise ValueError("annotation stroke_width needs 1..4096 samples")
    return tuple(_finite(v, "annotation stroke_width") for v in widths)


def _bind(cls, name, method):
    method.__name__ = name
    method.__qualname__ = cls.__qualname__ + "." + name
    method.__module__ = cls.__module__
    setattr(cls, name, method)


def install_annotation_marks(native):
    """Use authored Line/VGroup geometry for crosses and underlines."""
    g = vars(native)
    if g.get("_FMN_ANNOTATION_MARKS_INSTALLED", False):
        return
    Cross, Underline, Line = (g[name] for name in ("Cross", "Underline", "Line"))

    def target(obj, name):
        if not isinstance(obj, g["_BridgeMobject"]):
            raise TypeError(name + " expects a Mobject")

    def cross_init(self, mobject, stroke_color=g["RED"], stroke_width=[0, 6, 0], **kwargs):
        target(mobject, "Cross")
        widths = _width(stroke_width)
        g["_preflight_vmobject_style_kwargs"](kwargs)
        arms = (Line(g["UL"], g["DR"]), Line(g["UR"], g["DL"])) if mobject.family_members_with_points() else ()
        super(Cross, self).__init__(*arms, **kwargs)
        self.insert_n_curves(20)
        self.replace(mobject, stretch=True)
        self.set_stroke(stroke_color, width=widths)

    def underline_init(self, mobject, buff=g["SMALL_BUFF"], stroke_color=g["WHITE"],
                       stroke_width=[0, 3, 3, 0], stretch_factor=1.2, **kwargs):
        target(mobject, "Underline")
        buff = _finite(buff, "Underline buff")
        stretch_factor = _finite(stretch_factor, "Underline stretch_factor")
        widths = _width(stroke_width)
        super(Underline, self).__init__(g["LEFT"], g["RIGHT"], **kwargs)
        if not isinstance(widths, Real):
            self.insert_n_curves(max(0, len(widths) - 2))
        self.set_stroke(stroke_color, widths)
        self.set_width(_finite(mobject.get_width() * stretch_factor, "Underline width"))
        self.next_to(mobject, g["DOWN"], buff=buff)
        if not mobject.family_members_with_points():
            self.clear_points()

    _bind(Cross, "__init__", cross_init)
    _bind(Underline, "__init__", underline_init)
    g["_FMN_ANNOTATION_MARKS_INSTALLED"] = True
