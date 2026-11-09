"""Native ruled tables for data-driven scenes, without a host pandas install.

CSV goes through Atlas/frankenpandas; literal string grids and CSV share the
same Scribe table builder. No parser, font engine or layout solver lives here.
"""
from __future__ import annotations

import itertools
import math
import operator

import manimlib as m
import numpy as np

from .invocation import InvocationGuard

_EDITS = InvocationGuard()

_MAX_CELLS = 4096
_MAX_BYTES = 262_144


def _items(values, limit, name):
    if isinstance(values, (str, bytes)):
        raise TypeError(name + " must be an iterable, not a string")
    result = list(itertools.islice(iter(values), limit + 1))
    if len(result) > limit:
        raise ValueError(name + " exceeds the 4096-cell table budget")
    return result


def _grid(headers, rows):
    names = _items(headers, _MAX_CELLS // 2, "headers")
    if not names:
        raise ValueError("table requires at least one header")
    body = _items(rows, _MAX_CELLS // len(names) - 1, "rows")
    if not body:
        raise ValueError("table requires at least one body row")
    body = [_items(row, len(names), "row") for row in body]
    if any(len(row) != len(names) for row in body):
        raise ValueError("table rows must match the header count")
    size = 0
    for row in [names, *body]:
        for value in row:
            if not isinstance(value, str):
                raise TypeError("literal table cells must be strings; CSV performs native type inference")
            size += len(value.encode("utf-8"))
            if size > _MAX_BYTES:
                raise ValueError("table exceeds 262144 UTF-8 bytes")
    return tuple(names), tuple(tuple(row) for row in body)


def _index(value, length):
    value = operator.index(value)
    if not -length <= value < length:
        raise IndexError("table index out of range")
    return value % length


def _csv_grid(text, separator):
    if not isinstance(text, str):
        raise TypeError("table CSV must be text; read files explicitly in the host")
    if len(text.encode("utf-8")) > _MAX_BYTES:
        raise ValueError("table CSV exceeds 262144 UTF-8 bytes")
    if not isinstance(separator, str) or len(separator) != 1:
        raise ValueError("table CSV separator must be one character")
    # Refuse the same unsupported syntax on older compatible native wheels
    # too; never let fp-frame silently split a quoted delimiter.
    if '"' in text:
        raise ValueError("quoted CSV is unavailable in the pinned native frame reader; supply a literal string grid instead")
    parser = getattr(m, "_table_from_csv", None)
    if not callable(parser):
        raise m._CapabilityError("this native wheel does not provide CSV tables")
    return parser(text, separator)


def _discrete(method):
    def refuse(*args, **kwargs):
        raise m._CapabilityError("table data edits are discrete; use table.animate_data(...) for a morph")
    method._override_animate = refuse
    return method


class TableMobject(m.VGroup):
    """A native table whose cells and rules are independently animatable.

    Body indices are zero-based (negative indices are supported). A column
    may also be named when its header is unique. Returned groups contain live
    native objects, while ``headers`` and ``values`` are immutable text tuples.
    """
    def __init__(self, headers, rows, *, font_size=24, color=m.WHITE,
                 header_color=m.BLUE_B, rule_color=m.GREY_B, **kwargs):
        names, body = _grid(headers, rows)
        size = float(font_size)
        if not math.isfinite(size) or not 0 < size <= 10000:
            raise ValueError("table font_size must be finite and in (0, 10000]")
        for value in (color, header_color, rule_color):
            rgb = m.color_to_rgb(value)
            if not all(math.isfinite(v) and 0 <= v <= 1 for v in rgb):
                raise ValueError("table colors must have finite RGB components in [0,1]")
        builder = getattr(m, "_build_table", None)
        if not callable(builder):
            raise m._CapabilityError("this native wheel does not provide table construction")
        raw = m.VMobject()
        specs = builder(raw, m._native_shell_factory, list(names), [list(row) for row in body])
        m._hang_native_children(raw, specs)
        count = len(names) + len(body)
        if len(raw) != count + len(names) * (len(body) + 1):
            raise RuntimeError("native table family does not match its declared grid")
        children = tuple(raw.submobjects)
        raw.remove(*children)
        rules = m.VGroup(*children[:count])
        cells = m.VGroup(*children[count:])
        # Derive cell centers from the native rules, never recompute text
        # metrics/padding in the host. Empty cells need a real placement anchor.
        low, high = rules[0].get_corner(m.DL), rules[0].get_corner(m.UR)
        xs = [low[0], *(line.get_start()[0] for line in rules.submobjects[1 + len(body):]), high[0]]
        ys = [high[1], *(line.get_start()[1] for line in rules.submobjects[1:1 + len(body)]), low[1]]
        for index, cell in enumerate(cells):
            row, col = divmod(index, len(names))
            glyphs = tuple(cell.submobjects)
            cell.remove(*glyphs)
            cell._table_content = m.VGroup(*glyphs)
            cell._table_anchor = m.VectorizedPoint(((xs[col] + xs[col+1]) / 2,
                                                   (ys[row] + ys[row+1]) / 2, 0))
            cell.add(cell._table_content, cell._table_anchor)
        # The native outline is the placement authority, including empty cells.
        # Three invisible points retain the authored affine chart through all
        # ordinary mobject transformations, copies and scene checkpoints.
        anchors = m.VGroup(*(m.VectorizedPoint(rules[0].get_corner(d)) for d in (m.DL, m.DR, m.UL)))
        super().__init__(rules, cells, anchors, **kwargs)
        self._table_rules, self._table_cells, self._table_anchors = rules, cells, anchors
        self._table_headers, self._table_values = names, body
        self._table_layout_size = (rules[0].get_width(), rules[0].get_height())
        self._table_font_size = size
        self._table_colors = (color, header_color, rule_color)
        cells.set_color(color)
        self.get_headers().set_color(header_color)
        rules.set_stroke(rule_color)
        self.scale(size / 48, about_point=m.ORIGIN)
        # Invisible chart anchors must not determine the visible table bounds.
        self.shift(-rules[0].get_center())

    @classmethod
    def from_csv(cls, text, *, separator=",", **kwargs):
        names, body = _csv_grid(text, separator)
        return cls(names, body, **kwargs)

    @property
    def headers(self):
        return self._table_headers

    @property
    def values(self):
        return self._table_values

    @property
    def shape(self):
        return len(self.values), len(self.headers)

    def _column(self, column):
        if isinstance(column, str):
            matches = [i for i, name in enumerate(self.headers) if name == column]
            if len(matches) != 1:
                raise KeyError("table column name is missing or ambiguous: " + column)
            return matches[0]
        return _index(column, len(self.headers))

    def get_cell(self, row, column):
        return self._table_cells[(_index(row, len(self.values)) + 1) * len(self.headers) + self._column(column)]

    def get_headers(self):
        return m.VGroup(*self._table_cells.submobjects[:len(self.headers)])

    def get_rows(self):
        return m.VGroup(*(m.VGroup(*(self.get_cell(row, col) for col in range(self.shape[1])))
                          for row in range(self.shape[0])))

    def get_columns(self):
        return m.VGroup(*(m.VGroup(*(self.get_cell(row, col) for row in range(self.shape[0])))
                          for col in range(self.shape[1])))

    def get_rules(self):
        return m.VGroup(*self._table_rules.submobjects)


    def _structure(self):
        rows, cols = len(self._table_values), len(self._table_headers)
        if (self._table_rules not in self.submobjects or self._table_cells not in self.submobjects
                or self._table_anchors not in self.submobjects
                or len(self._table_rules) != rows + cols
                or len(self._table_cells) != (rows + 1) * cols
                or len(self._table_anchors) != 3):
            raise RuntimeError("table-owned rule/cell structure was edited directly")
        for cell in self._table_cells:
            if (getattr(cell, '_table_content', None) not in cell.submobjects
                    or getattr(cell, '_table_anchor', None) not in cell.submobjects):
                raise RuntimeError("table cell content/anchor was removed")

    def _stamp(self):
        self._structure()
        family = self.get_family()
        for obj in family:
            obj.get_points()  # Bake retained placement before comparing records.
        return (self.headers, self.values, self._table_layout_size,
                tuple((id(obj), id(getattr(obj, '_scene', None)),
                       tuple(id(child) for child in obj.submobjects),
                       obj.data.dtype.descr, obj.data.tobytes()) for obj in family))

    def _chart(self):
        points = [anchor.get_center() for anchor in self._table_anchors]
        width, height = self._table_layout_size
        ex, ey = (points[1] - points[0]) / width, (points[2] - points[0]) / height
        normal = np.cross(ex, ey)
        area = np.linalg.norm(normal)
        scale = np.linalg.norm(ex) * np.linalg.norm(ey)
        if (not np.isfinite(points).all() or not np.isfinite(scale) or scale == 0
                or area <= 1e-7 * scale):
            raise ValueError("table reflow requires a finite, nondegenerate affine placement")
        return points[2], np.column_stack((ex, ey, normal / area))

    def _prepare(self, headers, rows):
        if vars(self).get('_is_animating', False):
            raise RuntimeError("cannot replace table data during an active table/family animation")
        if vars(self).get('_table_data_partial', False):
            raise RuntimeError("cannot reflow an unfinished table morph; finish it or restore a checkpoint")
        if any(getattr(obj, 'locked_data_keys', ()) for obj in self.get_family()):
            raise RuntimeError("release table record locks before replacing its data")
        before = self._stamp()
        names, body = _grid(headers, rows)
        if names == self.headers and body == self.values:
            if self._stamp() != before:
                raise RuntimeError("table changed during data preparation")
            return None
        upper_left, matrix = self._chart()
        candidate = TableMobject(names, body, font_size=48,
                                 color=self._table_colors[0], header_color=self._table_colors[1],
                                 rule_color=self._table_colors[2])
        width, height = candidate._table_layout_size
        candidate.apply_matrix(matrix, about_point=m.ORIGIN)
        candidate.shift(upper_left + matrix @ np.array([width / 2, -height / 2, 0]))
        # Match surviving positional cells, not flatten-order indices: adding
        # a column must never turn an old row's first cell into its neighbor.
        old_cols = len(self.headers)
        for index, cell in enumerate(candidate._table_cells):
            row, col = divmod(index, len(names))
            if row <= len(self.values) and col < old_cols:
                cell._table_content.match_style(self._table_cells[row * old_cols + col]._table_content)
        for key, rule in candidate._rules_by_key().items():
            old = self._rules_by_key().get(key)
            if old is not None:
                rule.match_style(old)
        if self._stamp() != before or vars(self).get('_is_animating', False):
            raise RuntimeError("table changed during data preparation")
        return candidate

    def _rules_by_key(self):
        rows, cols = self.shape
        keys = [('outline', 0), *(('row', i) for i in range(rows)),
                *(('column', i) for i in range(cols - 1))]
        return dict(zip(keys, self._table_rules))

    def _publish(self, candidate):
        """Install prepared native geometry while retaining surviving owners."""
        old_cols, old_rows = len(self.headers), len(self.values)
        cells, rules = [], []
        old_rules = self._rules_by_key()
        for key, new in candidate._rules_by_key().items():
            old = old_rules.get(key)
            if old is None:
                rules.append(new)
            else:
                old.set_points(new.get_points())
                rules.append(old)
        for index, new in enumerate(candidate._table_cells):
            row, col = divmod(index, len(candidate.headers))
            if row > old_rows or col >= old_cols:
                cells.append(new)
                continue
            old = self._table_cells[row * old_cols + col]
            old_anchor = old._table_anchor.get_center()
            delta = new._table_anchor.get_center() - old_anchor
            # Cell-owned annotations follow its new center, but are not text
            # glyphs and must not be dropped when the string becomes empty.
            for annotation in tuple(old.submobjects):
                if annotation is not old._table_content and annotation is not old._table_anchor:
                    annotation.shift(delta)
            old._table_anchor.set_points(new._table_anchor.get_points())
            old._table_content.set_submobjects(tuple(new._table_content.submobjects))
            cells.append(old)
        self._table_rules.set_submobjects(rules)
        self._table_cells.set_submobjects(cells)
        for old, new in zip(self._table_anchors, candidate._table_anchors):
            old.set_points(new.get_points())
        self._table_headers, self._table_values = candidate.headers, candidate.values
        self._table_layout_size = candidate._table_layout_size
        vars(self).pop('_table_data_partial', None)
        return self

    @_discrete
    def set_data(self, rows, *, headers=None):
        """Rebuild through native layout, preserving the current upper-left anchor.

        Surviving (row, column) cells, rules, updaters and annotations retain
        their identity. New strings reflow column widths and row heights.
        Input validation and native shaping finish before live geometry changes.
        """
        with _EDITS.hold(self, message="table data replacement cannot reenter the same table"):
            candidate = self._prepare(self.headers if headers is None else headers, rows)
            if candidate is not None:
                self._publish(candidate)
        return self

    @_discrete
    def set_cell(self, row, column, value):
        row, column = _index(row, len(self.values)), self._column(column)
        rows = [list(values) for values in self.values]
        rows[row][column] = value
        return self.set_data(rows)

    @_discrete
    def set_csv(self, text, *, separator=","):
        """Replace the displayed dataset through Atlas's native CSV parser."""
        with _EDITS.hold(self, message="table data replacement cannot reenter the same table"):
            before = self._stamp()
            names, body = _csv_grid(text, separator)
            if self._stamp() != before:
                raise RuntimeError("table changed during CSV ingestion")
            candidate = self._prepare(names, body)
            if candidate is not None:
                self._publish(candidate)
        return self

    def animate_data(self, rows, *, headers=None, **animation_config):
        """Morph an equal-shaped table using the normal Transform lifecycle.

        The target is laid out at begin (so Succession observes earlier edits).
        Values expose the last committed grid during a morph. Full endpoints
        commit exact glyph families; a partial final endpoint remains visual
        and cannot be reflowed until completed or a checkpoint is restored.
        Shape changes are discrete set_data operations.
        """
        names, body = _grid(self.headers if headers is None else headers, rows)
        if (len(body), len(names)) != self.shape:
            raise ValueError("table morphs require the same shape; use set_data for row/column changes")
        return TableDataAnimation(self, names, body, **animation_config)


class TableDataAnimation(m.Transform):
    """Native geometry morph with committed table-data endpoint semantics."""
    def __init__(self, table, headers, rows, **kwargs):
        if not isinstance(table, TableMobject):
            raise TypeError("table data animation requires a TableMobject")
        self._data_headers, self._data_rows = _grid(headers, rows)
        self._sampled_alphas = []
        self._table_initial = None
        super().__init__(table, None, **kwargs)

    def begin(self):
        self.mobject._structure()
        if (len(self._data_rows), len(self._data_headers)) != self.mobject.shape:
            raise ValueError("table shape changed before the data animation began")
        # Only semantic endpoint data is retained on the animation, never an
        # active-invocation lock on the copied mobject.
        self._table_initial = self.mobject.copy()
        target = self.mobject.copy()
        target.set_data(self._data_rows, headers=self._data_headers)
        self.target_mobject = target
        return super().begin()

    def interpolate_mobject(self, alpha):
        self._sampled_alphas = []
        return super().interpolate_mobject(alpha)

    def interpolate_submobject(self, submobject, start, target, alpha):
        self._sampled_alphas.append(float(alpha))
        return super().interpolate_submobject(submobject, start, target, alpha)

    def finish(self):
        super().finish()
        if self._sampled_alphas and all(value >= 1 for value in self._sampled_alphas):
            self.mobject._publish(self.target_mobject)
        elif self._sampled_alphas and all(value <= 0 for value in self._sampled_alphas):
            self.mobject._publish(self._table_initial)
        else:
            self.mobject._table_data_partial = True

    def abort(self):
        try:
            super().abort()
        finally:
            if self._table_initial is not None:
                self.mobject._publish(self._table_initial)
