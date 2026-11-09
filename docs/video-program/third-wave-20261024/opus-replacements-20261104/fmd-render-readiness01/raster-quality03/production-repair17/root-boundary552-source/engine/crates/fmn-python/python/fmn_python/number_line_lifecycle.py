"""NumberLine authoring through Line's native-backed initialization protocol.

The host assembles the public tick/label factories; native Lines, ArrowTips and
DecimalNumbers retain geometry ownership. No detached native tree replaces an
initialized subclass or discards its custom records and children.
"""
from __future__ import annotations

from bisect import bisect_left
from itertools import islice
import math

from .coordinate_lifecycle import _bind, _config, _length, _range

_MAX_TICKS = 65_536


def _finite(value, name, *, nonnegative=False):
    value = float(value)
    if not math.isfinite(value) or (nonnegative and value < 0):
        raise ValueError(name + " must be finite" + (" and nonnegative" if nonnegative else ""))
    return value


def _values(values, name):
    values = tuple(islice(iter(values), _MAX_TICKS + 1))
    if len(values) > _MAX_TICKS:
        raise ValueError(name + " exceeds the 65536-tick budget")
    return tuple(_finite(value, name) for value in values)


def _count(start, end, step, name):
    span = (end - start) / step
    if not math.isfinite(span) or span > _MAX_TICKS - 1:
        raise ValueError(name + " exceeds the 65536-tick budget")


def install_number_line_lifecycle(native):
    g = vars(native)
    if g.get("_FMN_NUMBER_LINE_LIFECYCLE_INSTALLED", False):
        return
    NumberLine, Line, Group, np = g["NumberLine"], g["Line"], g["VGroup"], g["_np"]

    def initialize(self, x_range=(-8, 8, 1), **kwargs):
        terms = _range(g, x_range, "NumberLine.x_range")
        config = dict(kwargs)
        options = dict(kwargs)
        color = options.pop("color", g["_DEFAULT_LIGHT_COLOR"])
        stroke_width = options.pop("stroke_width", 2.0)
        unit_size = _length(options.pop("unit_size", 1.0), "NumberLine.unit_size")
        width = _length(options.pop("width", None), "NumberLine.width")
        include_ticks = bool(options.pop("include_ticks", True))
        include_numbers = bool(options.pop("include_numbers", False))
        include_tip = bool(options.pop("include_tip", False))
        tick_size = _finite(options.pop("tick_size", .1), "NumberLine.tick_size", nonnegative=True)
        multiple = _finite(options.pop("longer_tick_multiple", 1.5), "NumberLine.longer_tick_multiple", nonnegative=True)
        offset = _finite(options.pop("tick_offset", 0), "NumberLine.tick_offset")
        spacing = options.pop("big_tick_spacing", None)
        big_values = options.pop("big_tick_numbers", ())
        big = _values(() if big_values is None else big_values, "NumberLine.big_tick_numbers")
        if include_ticks or include_numbers:
            _count(*terms, "NumberLine ticks")
        if spacing is not None:
            spacing = _length(spacing, "NumberLine.big_tick_spacing")
            _count(terms[0], terms[1], spacing, "NumberLine big ticks")
            big = tuple(np.arange(terms[0], terms[1] + spacing, spacing))
        direction = np.array(g["_vec3"](options.pop("line_to_number_direction", g["_DOWN"])), dtype=float)
        if not np.isfinite(direction).all():
            raise ValueError("NumberLine label direction must be finite")
        buff = _finite(options.pop("line_to_number_buff", g["_MED_SMALL_BUFF"]), "NumberLine label buff")
        tip = _config(options.pop("tip_config", {"width": .25, "length": .25}), "NumberLine.tip_config")
        decimal = _config(options.pop("decimal_number_config", {"num_decimal_places": 0, "font_size": 36}), "NumberLine.decimal_number_config")
        excluded = options.pop("numbers_to_exclude", None)
        excluded = None if excluded is None else _values(excluded, "NumberLine.numbers_to_exclude")
        self.x_range = terms
        self.x_min, self.x_max, self.x_step = terms
        self.tick_size, self.longer_tick_multiple, self.tick_offset = tick_size, multiple, offset
        self.include_ticks, self.include_numbers, self.include_tip = include_ticks, include_numbers, include_tip
        self.big_tick_numbers = list(big)
        self.line_to_number_direction, self.line_to_number_buff = direction, buff
        self.tip_config, self.decimal_number_config, self.numbers_to_exclude = tip, decimal, excluded
        # Retain inspection metadata consumed by older bound helpers. Live
        # geometry and the public fields above, not this inventory, drive ticks.
        self._number_line_params = (terms, config)
        # Line's primitive helper treats color as an overriding shorthand.
        # NumberLine's explicit stroke_color must win over its color default,
        # and an explicit None defers to it: VMobject.__init__ resolves
        # `stroke_color or color` (subsets passes stroke_color=None).
        for key in ("fill_color", "stroke_color"):
            if options.get(key) is None:
                options[key] = color
        super(NumberLine, self).__init__(
            terms[0] * g["_RIGHT"], terms[1] * g["_RIGHT"],
            stroke_width=stroke_width, **options,
        )
        if width is not None:
            self.set_width(width)
        else:
            self.scale(unit_size)
        self.center()
        if include_tip:
            self.add_tip()
            self.tip.set_stroke(self.get_stroke_color(), self.get_stroke_width())
        self.ticks = Group()
        if include_ticks:
            self.add_ticks()
        if include_numbers:
            self.add_numbers(excluding=self.numbers_to_exclude)

    def get_tick_range(self):
        # Public mutable fields must work after construction too. Check before
        # arange allocates, including changes to x_step or include_tip.
        lo, hi, step = _range(g, (self.x_min, self.x_max, self.x_step), "NumberLine tick range")
        _count(lo, hi, step, "NumberLine ticks")
        stop = hi if self.include_tip else hi + step
        if not math.isfinite(stop):
            raise ValueError("NumberLine tick endpoint must be finite")
        values = np.arange(lo, stop, step)
        return values[values <= hi]

    def add_ticks(self):
        positions = _values(self.get_tick_range(), "NumberLine tick range")
        big = sorted(_values(self.big_tick_numbers, "NumberLine.big_tick_numbers"))
        size = _finite(self.tick_size, "NumberLine.tick_size", nonnegative=True)
        multiple = _finite(self.longer_tick_multiple, "NumberLine.longer_tick_multiple", nonnegative=True)
        claimed = {id(member) for member in self.get_family()}
        ticks = []
        for x in positions:
            index = bisect_left(big, x)
            nearby = big[max(0, index - 1):index + 1]
            length = size * multiple if np.isclose(nearby, x).any() else size
            tick = self.get_tick(x, length)
            if not isinstance(tick, Line):
                raise TypeError("NumberLine.get_tick must return a Line")
            members = tuple(islice(iter(tick.get_family()), _MAX_TICKS + 1))
            if len(members) > _MAX_TICKS or any(id(member) in claimed for member in members):
                raise ValueError("NumberLine.get_tick must return independent bounded families")
            for member in members:
                if member._is_bound() or getattr(member, "_scene", None) is not None:
                    raise ValueError("NumberLine.get_tick must return detached geometry")
                if not np.isfinite(member.get_points()).all():
                    raise ValueError("NumberLine tick points must be finite")
            claimed.update(map(id, members))
            if len(claimed) > _MAX_TICKS * 4:
                raise ValueError("NumberLine tick families exceed the aggregate budget")
            ticks.append(tick)
        # No partial new tick group is published when an authored factory fails.
        group = Group(*ticks)
        self.add(group)
        self.ticks = group
        return self

    for name, function in (("__init__", initialize), ("get_tick_range", get_tick_range), ("add_ticks", add_ticks)):
        _bind(NumberLine, name, function)
    g["_FMN_NUMBER_LINE_LIFECYCLE_INSTALLED"] = True
