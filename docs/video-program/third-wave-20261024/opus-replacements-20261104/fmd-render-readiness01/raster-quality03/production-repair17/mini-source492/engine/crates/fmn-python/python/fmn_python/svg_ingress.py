"""Native SVG paint preparation under the existing SVGMobject front door.

The host only installs prepared families and preserves their paint roles during
ordinary styling. XML, inherited styles, filled-set normalization, subdivision,
and arclength all remain native. No source rewriting or second path model.
"""
from __future__ import annotations

from functools import wraps
import inspect
from itertools import islice
import math

from .invocation import InvocationGuard

_ROLE = "_fmn_svg_paint_role"
_MAX_PARTS = 65_536
_MAX_POINTS = 1_048_576  # Atlas's MAX_SVG_PAINT_POINTS, including paint layers.


def _dimension(value, name):
    if value is None:
        return None
    value = float(value)
    if not math.isfinite(value) or value <= 0 or value > 3.4028234663852886e38:
        raise ValueError("SVG " + name + " must be positive, finite and f32-representable")
    return value


def _authored_parts(g, receiver, values):
    """Freeze a whole factory result before adopting or transforming any part."""
    parts = list(islice(iter(values), _MAX_PARTS + 1))
    if len(parts) > _MAX_PARTS:
        raise ValueError("SVG factory exceeds the 65536-part budget")
    if len({id(part) for part in parts}) != len(parts):
        raise ValueError("SVG factory returned the same root more than once")
    retained = {id(member) for member in g["_family_preorder"](receiver)}
    seen, admitted, count = set(), [], 0
    for part in parts:
        if not isinstance(part, g["VMobject"]):
            raise TypeError("mobjects_from_svg_string must return VMobjects")
        for member in g["_family_preorder"](part):
            marker = id(member)
            if marker in retained:
                raise ValueError("SVG factory cannot return the receiver or its existing family")
            if marker in seen:
                continue  # Shared descendants remain a shared DAG, not copies.
            seen.add(marker)
            admitted.append(member)
            if len(seen) > _MAX_PARTS:
                raise ValueError("SVG factory exceeds the 65536-member budget")
            if not isinstance(member, g["VMobject"]):
                raise TypeError("SVG factory families must contain only VMobjects")
            if getattr(member, "_scene", None) is not None or member._is_bound():
                raise ValueError("SVG factory must return detached geometry")
            count += member.get_num_points()
            if count > _MAX_POINTS:
                raise ValueError("SVG factory exceeds the 1048576-point budget")
            if not g["_np"].isfinite(member.get_points()).all():
                raise ValueError("SVG factory geometry must be finite")
    # A later authored geometry getter may adopt an earlier part. Nothing has
    # been installed yet; recheck native ownership after all such callbacks.
    if any(g["_BridgeMobject"]._is_bound(member) or vars(member).get("_scene") is not None
           for member in admitted):
        raise ValueError("SVG factory ownership changed during preparation")
    return parts


def install_svg_ingress(native):
    g = vars(native)
    if g.get("_FMN_SVG_INGRESS_INSTALLED", False):
        return
    Svg, VM, Mob, np = g["SVGMobject"], g["VMobject"], g["Mobject"], g["_np"]
    builder = g.get("_build_svg_paints")
    if not callable(builder):
        raise ImportError("native SVG paint preparation is missing from this wheel")

    def members(obj, recurse):
        return tuple(g["_family_preorder"](obj)) if recurse else (obj,)

    def disallowed(obj, field):
        role = vars(obj).get(_ROLE)
        return ((role == "fill" and field == "stroke_rgba")
                or (role == "stroke" and field == "fill_rgba"))

    old_color = Mob.set_rgba_array_by_color
    @wraps(old_color)
    def by_color(self, color=None, opacity=None, name="rgba", recurse=True):
        family = members(self, recurse)
        if not any(_ROLE in vars(obj) for obj in family):
            return old_color(self, color, opacity, name, recurse)
        for obj in family:
            old_color(obj, color, 0.0 if disallowed(obj, name) else opacity, name, False)
        return self

    def masked(value):
        rgba = np.array(value, dtype=float, copy=True)
        if rgba.ndim == 0 or rgba.shape[-1] != 4:
            raise ValueError("SVG paint RGBA arrays must have four channels")
        rgba[..., 3] = 0
        return rgba

    old_array = Mob.set_rgba_array
    @wraps(old_array)
    def array(self, rgba_array, name="rgba", recurse=False):
        family = members(self, recurse)
        if not any(disallowed(obj, name) for obj in family):
            return old_array(self, rgba_array, name, recurse)
        inactive = masked(rgba_array)
        for obj in family:
            old_array(obj, inactive if disallowed(obj, name) else rgba_array, name, False)
        return self

    old_style = VM.set_style
    style_signature = inspect.signature(old_style)
    @wraps(old_style)
    def style(self, *args, **kwargs):
        bound = style_signature.bind(self, *args, **kwargs)
        bound.apply_defaults()
        family = members(self, bound.arguments["recurse"])
        if not any(_ROLE in vars(obj) for obj in family):
            return old_style(self, *args, **kwargs)
        values = dict(bound.arguments)
        values.pop("self")
        values["recurse"] = False
        # Explicit RGBA style arrays bypass the ordinary color setter in the
        # base class. Keep those aliases, and match_style, paint-role aware too.
        for obj in family:
            current = dict(values)
            for field in ("fill_rgba", "stroke_rgba"):
                if disallowed(obj, field):
                    current[field.removesuffix("rgba") + "opacity"] = 0.0
                    if current[field] is not None:
                        current[field] = masked(current[field])
            old_style(obj, **current)
        return self

    def tag(specs):
        # The native checked builder emits either an unchanged flat shape or
        # a recordless shape containing one fill layer followed by strokes.
        # Even empty/hidden roles are retained, so future styling stays correct.
        for _, children in specs:
            for index, (layer, _) in enumerate(children):
                vars(layer)[_ROLE] = "fill" if index == 0 else "stroke"
        return specs

    constructors = InvocationGuard()

    def build(self, factory, file_name, svg_string):
        if svg_string is not None:
            source = svg_string
        elif constructors.busy(self):
            # The unchanged constructor has already called the authored
            # resolver after installing live state and saved these exact bytes.
            if "svg_string" not in vars(self):
                raise RuntimeError("SVG source requested before its resolver completed")
            source = vars(self)["svg_string"]
        else:
            source = self.file_name_to_svg_string(file_name)
        return tag(builder(self, factory, source, {}))

    old_init = Svg.__init__
    init_signature = inspect.signature(old_init)
    @wraps(old_init)
    def initialize(self, *args, **kwargs):
        with constructors.hold(self, message="SVG construction cannot reenter the same object"):
            bound = init_signature.bind(self, *args, **kwargs)
            bound.apply_defaults()
            p = bound.arguments
            options = dict(p["kwargs"])
            g["_preflight_vmobject_style_kwargs"](options)
            g["_refuse_unrouted"](type(self).__name__ + "()", [
                ("svg_default", bool(p["svg_default"])
                 and any(value is not None for value in p["svg_default"].values())),
                ("path_string_config", bool(p["path_string_config"])),
            ])
            height = _dimension(p["height"] if p["height"] is not None else type(self).height, "height")
            width = _dimension(p["width"] if p["width"] is not None else type(self).width, "width")
            if self._is_bound():
                raise RuntimeError("SVG construction requires a detached receiver")
            # Preserve the resolver's established access to live proxy state
            # and the exact input recipes; initialize the record schema only
            # once, through VMobject, after source resolution succeeds.
            g["_install_live_state"](self)
            self.svg_default = dict(p["svg_default"]) if p["svg_default"] else dict.fromkeys((
                "color", "opacity", "fill_color", "fill_opacity", "stroke_width",
                "stroke_color", "stroke_opacity",
            ))
            self.path_string_config = dict(p["path_string_config"] or {})
            if p["svg_string"]:
                self.svg_string = p["svg_string"]
            else:
                name = p["file_name"] or type(self).file_name
                if not name:
                    raise Exception("Must specify either a file_name or svg_string SVGMobject")
                self.svg_string = self.file_name_to_svg_string(name)
            if not isinstance(self.svg_string, str):
                raise TypeError("SVG source resolver must return str")
            if self._is_bound():
                raise RuntimeError("SVG receiver was adopted during source resolution")
            # Do not install a replacement tree over init_data-owned columns
            # or children. Both public assembly hooks produce the real family.
            g["_init_native_vmobject"](self, options)
            self.init_svg_mobject()
            self.flip(g["_RIGHT"])
            self.set_style(
                fill_color=p["color"] or p["fill_color"], fill_opacity=p["fill_opacity"],
                stroke_color=p["color"] or p["stroke_color"], stroke_width=p["stroke_width"],
                stroke_opacity=p["stroke_opacity"],
            )
            g["_apply_vmobject_style_kwargs"](self, dict(options))
            if p["should_center"]:
                self.center()
            if height is not None:
                self.set_height(height)
            if width is not None:
                self.set_width(width)

    def parts(self, source):
        # This API returns detached shapes; it must not replace the receiver's
        # nursery as the former native-builder call on self did.
        root = VM()
        specs = tag(builder(root, g["_native_shell_factory"], source, {}))
        g["_hang_native_children"](root, specs)
        result = list(root.submobjects)
        root.remove(*result)
        return result

    rebuilds = InvocationGuard()

    def rebuild(self):
        # Reference svg_mobject.py:123 (Ledger row `same`): add a freshly
        # built family, so a second call appends a second one, and return
        # None. Preparing first keeps a refused source from publishing.
        with rebuilds.hold(self, message="SVG rebuilding cannot reenter the same object"):
            children = _authored_parts(g, self, self.mobjects_from_svg_string(self.svg_string))
            self.add(*children)

    Mob.set_rgba_array_by_color = by_color
    Mob.set_rgba_array = array
    VM.set_style = style
    Svg.__init__ = initialize
    for name, function in (("_build_svg_mobject", build),
                           ("mobjects_from_svg_string", parts),
                           ("init_svg_mobject", rebuild)):
        function.__name__ = name
        function.__qualname__ = Svg.__qualname__ + "." + name
        function.__module__ = Svg.__module__
        setattr(Svg, name, function)
    g["_FMN_SVG_INGRESS_INSTALLED"] = True
