"""Live Matrix authoring over the existing native mobjects and Scribe glyphs.

No geometry, font parser or animation interpolation lives here. These are
Python object-model operations: overridable entry conversion, grouping, and
identity-preserving replacement of already-built native entries.
"""
from __future__ import annotations

import itertools
import math
import operator
from typing import Any

_MAX_ENTRIES = 4096


def _finite(value: Any, name: str, *, nonnegative: bool = False) -> float:
    value = float(value)
    if not math.isfinite(value) or (nonnegative and value < 0):
        qualifier = "finite and nonnegative" if nonnegative else "finite"
        raise ValueError(f"{name} must be {qualifier}")
    return value


def _index(value: Any, length: int) -> int | None:
    if value is None:
        return None
    value = operator.index(value)
    return value % length if -length <= value < length else None


def _entry_config(matrix: Any, config: dict, *, decimal: bool) -> dict:
    result = dict(getattr(matrix, "_matrix_entry_style", {}))
    result["font_size"] = getattr(matrix, "_matrix_entry_font_size", 48.0)
    if decimal:
        result["num_decimal_places"] = getattr(matrix, "_matrix_decimal_places", 2)
    result.update(config)
    if not decimal:
        result.pop("num_decimal_places", None)
    return result


def _bind(cls: type, name: str, method: Any) -> None:
    # Do not use wraps on schema stubs: that would copy placeholder provenance
    # onto a real implementation. Class identity and every module alias stay.
    method.__name__ = name
    method.__qualname__ = cls.__qualname__ + "." + name
    method.__module__ = cls.__module__
    setattr(cls, name, method)


def install_matrix(native: Any) -> None:
    if vars(native).get("_FMN_MATRIX_INSTALLED", False):
        return
    Matrix, DecimalMatrix = native.Matrix, native.DecimalMatrix
    VMobject, VGroup = native.VMobject, native.VGroup
    Tex, DecimalNumber = native.Tex, native.DecimalNumber
    np = vars(native)["_np"]

    def element_to_mobject(self, element, **config):
        if not isinstance(self, Matrix):
            raise TypeError("Matrix.element_to_mobject requires a Matrix instance")
        if isinstance(element, VMobject):
            return element
        if isinstance(element, (float, complex, np.floating, np.complexfloating)):
            return DecimalNumber(element, **_entry_config(self, config, decimal=True))
        return Tex(str(element), **_entry_config(self, config, decimal=False))

    def decimal_element_to_mobject(self, element, **config):
        if not isinstance(self, DecimalMatrix):
            raise TypeError("DecimalMatrix.element_to_mobject requires a DecimalMatrix instance")
        return DecimalNumber(element, **_entry_config(self, config, decimal=True))

    def create_brackets(self, rows, v_buff, h_buff):
        count = len(rows)
        if not 0 < count <= _MAX_ENTRIES:
            raise ValueError("matrix brackets require 1..4096 rows")
        v_buff = _finite(v_buff, "bracket_v_buff")
        h_buff = _finite(h_buff, "bracket_h_buff")
        height = _finite(rows.get_height() + v_buff, "bracket height", nonnegative=True)
        source = r"\left[\begin{array}{c}" + count * r"\quad \\" + r"\end{array}\right]"
        brackets = Tex(source)
        if len(brackets) < 2:
            raise RuntimeError("native matrix delimiter typesetting did not produce a bracket pair")
        brackets.set_height(height)
        middle = len(brackets) // 2
        left, right = brackets[:middle], brackets[middle:]
        left.next_to(rows, native.LEFT, h_buff)
        right.next_to(rows, native.RIGHT, h_buff)
        return VGroup(left, right)

    def swap_entry_for_dots(self, entry, dots):
        if not isinstance(entry, VMobject) or not isinstance(dots, VMobject):
            raise TypeError("matrix ellipsis replacement requires VMobject entries and dots")
        dots.move_to(entry)
        # Native become keeps the live entry handle; replacing the Python list
        # slot here would strand animations, aliases and external references.
        entry.become(dots)
        for index, element in enumerate(self.elements):
            if element is entry:
                del self.elements[index]
                break
        if not any(element is entry for element in self.ellipses):
            self.ellipses.append(entry)
        # The Reference helper returns None; the batch operation returns self.

    def swap_entries_for_ellipses(
        self, row_index=None, col_index=None, height_ratio=0.65, width_ratio=0.4,
    ):
        rows, columns = self.get_rows(), self.get_columns()
        n_rows, n_cols = len(rows), len(columns)
        if not n_rows or not n_cols or n_rows * n_cols > _MAX_ENTRIES:
            raise ValueError("matrix ellipses require a nonempty grid of at most 4096 entries")
        row_index, col_index = _index(row_index, n_rows), _index(col_index, n_cols)
        if row_index is None and col_index is None:
            return self
        height_ratio = _finite(height_ratio, "height_ratio", nonnegative=True)
        width_ratio = _finite(width_ratio, "width_ratio", nonnegative=True)
        height = _finite(height_ratio * rows.get_height() / n_rows, "ellipsis height", nonnegative=True)
        width = _finite(width_ratio * columns.get_width() / n_cols, "ellipsis width", nonnegative=True)
        # Complete native typesetting and scaling before touching live cells.
        # A font/capability/budget error cannot leave half of a row replaced.
        replacements = []
        if row_index is not None:
            for column in columns:
                dots = Tex(r"\vdots")
                dots.set_height(height)
                replacements.append((column[row_index], dots))
        if col_index is not None:
            for row in rows:
                dots = Tex(r"\hdots")
                dots.set_width(width)
                replacements.append((row[col_index], dots))
        for entry, dots in replacements:
            self.swap_entry_for_dots(entry, dots)
        if row_index is not None and col_index is not None:
            entry = rows[row_index][col_index]
            center = entry.get_center().copy()
            entry.rotate(-45 * native.DEG)
            # Native glyph contours need not have rotationally symmetric
            # bounds. Keep the matrix cell's layout anchor after rotation;
            # rotating about that anchor alone does not preserve its bbox.
            entry.move_to(center)
        return self

    def set_column_colors(self, *colors):
        for column, color in zip(self.get_columns(), colors):
            column.set_color(color)
        return self

    def add_background_to_entries(self):
        for entry in self.get_entries():
            entry.add_background_rectangle()
        return self

    for name, method in (
        ("element_to_mobject", element_to_mobject),
        ("create_brackets", create_brackets),
        ("swap_entry_for_dots", swap_entry_for_dots),
        ("swap_entries_for_ellipses", swap_entries_for_ellipses),
        ("set_column_colors", set_column_colors),
        ("add_background_to_entries", add_background_to_entries),
    ):
        _bind(Matrix, name, method)
    _bind(DecimalMatrix, "element_to_mobject", decimal_element_to_mobject)
    _install_construction(native)
    native._FMN_MATRIX_INSTALLED = True


def _rows(matrix: Any) -> list[list[Any]]:
    """Bound and materialize input with Atlas's shape/entry budget before conversion."""
    rows, total = [], 0
    try:
        outer = iter(matrix)
    except TypeError as error:
        raise TypeError("matrix must be an iterable of row iterables") from error
    for row in itertools.islice(outer, _MAX_ENTRIES + 1):
        if len(rows) == _MAX_ENTRIES:
            raise ValueError("matrix rows exceed the declared limit of 4096")
        try:
            values = list(itertools.islice(iter(row), _MAX_ENTRIES + 1))
        except TypeError as error:
            raise TypeError("matrix rows must be iterable") from error
        total += len(values)
        if total > _MAX_ENTRIES:
            raise ValueError("matrix entries exceed the declared limit of 4096")
        rows.append(values)
    if not rows or not rows[0]:
        raise ValueError("a matrix needs at least one row and one column")
    width = len(rows[0])
    if any(len(row) != width for row in rows):
        raise ValueError("ragged matrix: all rows must have the same number of entries")
    return rows


def _owners(native: Any, entries: Any) -> None:
    """Reject cross-Scene references before layout moves any caller-owned cell."""
    owner, seen = None, set()
    for entry in entries:
        if not isinstance(entry, native.VMobject):
            continue
        for member in entry.get_family():
            if id(member) in seen:
                continue
            seen.add(id(member))
            scene = getattr(member, "_scene", None)
            if scene is None:
                continue
            if owner is not None and scene is not owner:
                error = getattr(native, "_ForeignStageError", ValueError)
                raise error("a matrix cannot contain entries from multiple Scenes; copy its mobjects")
            owner = scene


def _install_construction(native: Any) -> None:
    """Keep Atlas's scalar fast path, but never lower away authored hooks.

    A customized Matrix is an ordinary Python composition of native mobjects.
    Its row layout is the Reference's public move_to choreography, not a new
    geometry kernel, typesetter, or record-buffer implementation. In particular
    those calls must remain virtual: a native scalar constructor cannot execute
    user overrides of element conversion, movement, brackets, or grouping.
    """
    Matrix, DecimalMatrix = native.Matrix, native.DecimalMatrix
    IntegerMatrix, TexMatrix, MobjectMatrix = native.IntegerMatrix, native.TexMatrix, native.MobjectMatrix
    VMobject, VGroup = native.VMobject, native.VGroup
    np = vars(native)["_np"]
    original = {cls: cls.__init__ for cls in (Matrix, DecimalMatrix, IntegerMatrix, TexMatrix, MobjectMatrix)}
    original_copy = Matrix.copy

    def source_grid(matrix, rows):
        # Concrete authored containers remain observable to grid overrides;
        # one-shot outer/row iterators use the already materialized table.
        containers = (list, tuple, np.ndarray)
        if isinstance(matrix, containers) and all(isinstance(row, containers) for row in matrix):
            return matrix
        return rows

    def matrix_copy(self, deep=False):
        result = original_copy(self, deep)
        # The native Matrix copier already remaps entries/rows/columns and
        # bracket children. Keep a live bracket group when a Python constructor
        # supplied one instead of changing its public container type to list.
        if isinstance(self.brackets, VGroup) and not isinstance(result.brackets, VGroup):
            result.brackets = VGroup(*result.brackets)
        return result

    def create_mobject_matrix(self, matrix, v_buff, h_buff, aligned_corner, **element_config):
        rows = _rows(matrix)
        v_buff, h_buff = _finite(v_buff, "v_buff"), _finite(h_buff, "h_buff")
        corner = np.asarray(aligned_corner, dtype=float)
        if corner.shape != (3,) or not np.isfinite(corner).all():
            raise ValueError("element_alignment_corner must have three finite components")
        _owners(native, (entry for row in rows for entry in row))
        # Finish conversion/type/owner validation before native move_to writes.
        # Do not float-coerce input in typed subclasses before a Python hook
        # sees it: custom converters are entitled to the original value.
        converted = [[self.element_to_mobject(entry, **element_config) for entry in row] for row in rows]
        entries = [entry for row in converted for entry in row]
        if any(not isinstance(entry, VMobject) for entry in entries):
            raise TypeError("Matrix.element_to_mobject must return a VMobject for every entry")
        _owners(native, entries)
        widths = [_finite(entry.get_width(), "entry width", nonnegative=True) for entry in entries]
        heights = [_finite(entry.get_height(), "entry height", nonnegative=True) for entry in entries]
        x_step = _finite(max(widths) + h_buff, "matrix column step") * native.RIGHT
        y_step = _finite(max(heights) + v_buff, "matrix row step") * native.DOWN
        positions = [i * y_step + j * x_step for i, row in enumerate(converted) for j in range(len(row))]
        if not all(np.isfinite(position).all() for position in positions):
            raise ValueError("matrix layout positions must be finite")
        for entry, position in zip(entries, positions):
            entry.move_to(position, aligned_corner)
        return converted

    _bind(Matrix, "create_mobject_matrix", create_mobject_matrix)
    hooks = ("element_to_mobject", "create_mobject_matrix", "create_brackets",
             "swap_entries_for_ellipses", "swap_entry_for_dots", "get_rows",
             "get_columns", "add", "center", "init_points", "init_colors", "init_uniforms")
    baselines = {cls: {name: getattr(cls, name, None) for name in hooks} for cls in original}

    def unchanged(self, cls):
        # All subclasses use the virtual constructor even if they currently
        # inherit every hook: __getattribute__ and cooperative bases are Python.
        if type(self) is not cls:
            return False
        for name, method in baselines[cls].items():
            current = getattr(self, name, None)
            if getattr(current, "__func__", current) is not method:
                return False
        return True

    def matrix_init(
        self, matrix, v_buff=0.5, h_buff=0.5, bracket_h_buff=0.2,
        bracket_v_buff=0.25, height=None, element_config=dict(),
        element_alignment_corner=native.DOWN, ellipses_row=None, ellipses_col=None,
    ):
        rows = _rows(matrix)
        config = dict(element_config)
        common = dict(v_buff=_finite(v_buff, "v_buff"), h_buff=_finite(h_buff, "h_buff"),
                      bracket_h_buff=_finite(bracket_h_buff, "bracket_h_buff"),
                      bracket_v_buff=_finite(bracket_v_buff, "bracket_v_buff"),
                      height=None if height is None else _finite(height, "height", nonnegative=True),
                      element_alignment_corner=element_alignment_corner,
                      ellipses_row=ellipses_row, ellipses_col=ellipses_col)
        if height is not None:
            _finite(common["height"] - 2 * common["bracket_v_buff"], "matrix entry height", nonnegative=True)
        _owners(native, (entry for row in rows for entry in row))
        # Built-in ordinary scalar matrices keep their existing native layout,
        # glyph decoration, span maps and exact fast-path output. Already-built
        # mobjects must retain their handles, and complex entries use the
        # native-backed readout composition. Neither belongs in a scalar-only
        # constructor, regardless of whether element_config was supplied.
        needs_composition = any(isinstance(entry, (VMobject, complex, np.complexfloating))
                                for row in rows for entry in row)
        if unchanged(self, Matrix) and not config and not needs_composition:
            return original[Matrix](self, rows, element_config=config, **common)
        VMobject.__init__(self)
        self._matrix_entry_font_size = config.get("font_size", 48.0)
        self._matrix_decimal_places = config.get("num_decimal_places", 2)
        self._matrix_entry_style = {key: value for key, value in config.items()
                                   if key not in {"font_size", "num_decimal_places"}}
        made = self.create_mobject_matrix(source_grid(matrix, rows), common["v_buff"], common["h_buff"],
                                          element_alignment_corner, **config)
        checked = _rows(made)
        entries = [entry for row in checked for entry in row]
        if any(not isinstance(entry, VMobject) for entry in entries):
            raise TypeError("Matrix.create_mobject_matrix must return a rectangular VMobject grid")
        _owners(native, entries)
        # Retain an authored list/tuple grid and its row identities, not just
        # its cells. Iterators are necessarily materialized to own their data.
        self.mob_matrix = (made if isinstance(made, (list, tuple))
                           and all(isinstance(row, (list, tuple)) for row in made) else checked)
        n_rows, n_cols = len(checked), len(checked[0])
        self._matrix_shape = (n_rows, n_cols)
        self._matrix_kind = "mobject"
        self.elements = entries
        self.columns = VGroup(*(VGroup(*(row[index] for row in checked)) for index in range(n_cols)))
        self.rows = VGroup(*(VGroup(*row) for row in checked))
        if height is not None:
            self.rows.set_height(common["height"] - 2 * common["bracket_v_buff"])
        self.brackets = self.create_brackets(self.rows, common["bracket_v_buff"], common["bracket_h_buff"])
        self.ellipses = []
        self.add(*self.elements)
        self.add(*self.brackets)
        self.center()
        self.swap_entries_for_ellipses(ellipses_row, ellipses_col)

    def decimal_init(self, matrix, num_decimal_places=2, decimal_config=dict(), **config):
        rows = _rows(matrix)
        options = dict(decimal_config)
        if "num_decimal_places" in options:
            raise TypeError("num_decimal_places was supplied twice")
        if "element_config" in config:
            raise TypeError("element_config is specified by decimal_config")
        self.float_matrix = matrix
        has_complex = any(isinstance(entry, (complex, np.complexfloating))
                          for row in rows for entry in row)
        if unchanged(self, DecimalMatrix) and not options and not has_complex:
            result = original[DecimalMatrix](self, rows, num_decimal_places=num_decimal_places, **config)
            self.float_matrix = matrix
            return result
        Matrix.__init__(self, source_grid(matrix, rows),
                        element_config=dict(options, num_decimal_places=num_decimal_places), **config)

    def integer_init(self, matrix, num_decimal_places=0, decimal_config=dict(), **config):
        rows = _rows(matrix)
        options = dict(decimal_config)
        if "num_decimal_places" in options:
            raise TypeError("num_decimal_places was supplied twice")
        if "element_config" in config:
            raise TypeError("element_config is specified by decimal_config")
        self.float_matrix = matrix
        has_complex = any(isinstance(entry, (complex, np.complexfloating))
                          for row in rows for entry in row)
        if unchanged(self, IntegerMatrix) and not options and not has_complex:
            result = original[IntegerMatrix](self, rows, num_decimal_places=num_decimal_places, **config)
            self.float_matrix = matrix
            return result
        DecimalMatrix.__init__(self, source_grid(matrix, rows), num_decimal_places=num_decimal_places,
                               decimal_config=options, **config)
        self.float_matrix = matrix

    def tex_init(self, matrix, tex_config=dict(), **config):
        rows = _rows(matrix)
        options = dict(tex_config)
        if "element_config" in config:
            raise TypeError("element_config is specified by tex_config")
        if (unchanged(self, TexMatrix) and not options
                and not any(isinstance(entry, (VMobject, float, complex, np.floating, np.complexfloating))
                            for row in rows for entry in row)):
            return original[TexMatrix](self, rows, **config)
        Matrix.__init__(self, source_grid(matrix, rows), element_config=options, **config)

    def mobject_init(
        self, group, n_rows=None, n_cols=None, height=4.0,
        element_alignment_corner=native.ORIGIN, **config,
    ):
        if not isinstance(group, VGroup):
            raise TypeError("MobjectMatrix group must be a VGroup")
        count = len(group)
        if count == 0:
            raise ValueError("a matrix needs at least one row and one column")
        if n_rows is not None:
            n_rows = operator.index(n_rows)
        if n_cols is not None:
            n_cols = operator.index(n_cols)
        if (n_rows is not None and n_rows <= 0) or (n_cols is not None and n_cols <= 0):
            raise ValueError("MobjectMatrix dimensions must be positive")
        if n_rows is None:
            n_rows = math.isqrt(count) if n_cols is None else count // n_cols
        if n_cols is None:
            n_cols = count // n_rows
        if n_rows <= 0 or n_cols <= 0:
            raise ValueError("MobjectMatrix dimensions must be positive")
        required = n_rows * n_cols
        if required > _MAX_ENTRIES:
            raise ValueError("matrix entries exceed the declared limit of 4096")
        if count < required:
            raise ValueError("Input to MobjectMatrix must have at least n_rows * n_cols entries")
        entries = [group[index] for index in range(required)]
        _owners(native, entries)
        if unchanged(self, MobjectMatrix) and height is not None:
            return original[MobjectMatrix](self, group, n_rows=n_rows, n_cols=n_cols, height=height,
                                           element_alignment_corner=element_alignment_corner, **config)
        rows = [entries[start:start + n_cols] for start in range(0, required, n_cols)]
        Matrix.__init__(self, rows, height=height, element_alignment_corner=element_alignment_corner, **config)
        self.group = group

    for cls, method in ((Matrix, matrix_init), (DecimalMatrix, decimal_init),
                        (IntegerMatrix, integer_init), (TexMatrix, tex_init), (MobjectMatrix, mobject_init)):
        _bind(cls, "__init__", method)
    _bind(Matrix, "copy", matrix_copy)
