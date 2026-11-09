"""Admit point tables and preserve authored transform dispatch over native records.

Native RecordBuffer resize, view lifetimes, paint lanes and path metadata remain
owned by the existing Mobject/VMobject protocol. Empty Rust vector returns are
valid empty point tables, and authored point maps use the shared native matrices.
"""
from __future__ import annotations

# The largest finite float32, exactly as `np.finfo(np.float32).max` compares
# against float64 coordinates.
_F32_MAX = 3.4028234663852886e38


def _point_table(np, points):
    values = np.asarray(points)
    if values.ndim == 1 and values.shape == (0,):
        values = values.reshape((0, 3))
    if values.ndim != 2 or values.shape[1] != 3:
        raise ValueError("points must be an (N, 3) array of real coordinates")
    if values.dtype.kind == "c":
        raise TypeError("points must be real coordinates")
    # Own the input before resizing or invoking authored resize hooks. A view
    # can alias the live destination, including a reversed/strided field view.
    values = np.array(values, dtype=np.float64, copy=True)
    # One reduction proves both checks below, since NaN fails the comparison;
    # only a table that fails it takes them, in order, for the exact error
    # (fm-ztjj: five NumPy calls per one-point table otherwise).
    if values.size and np.abs(values).max() <= _F32_MAX:
        return values
    if not np.isfinite(values).all():
        raise ValueError("points must be finite")
    if np.any(np.abs(values) > np.finfo(np.float32).max):
        raise ValueError("points must be f32-representable")
    return values


def install_point_editing(native):
    """Keep native classes and VMobject's odd-count/path-metadata dispatch."""
    g = vars(native)
    if g.get("_FMN_POINT_EDITING_INSTALLED", False):
        return
    Mobject, np = g["Mobject"], g["_np"]
    from functools import wraps
    from .movement import _changed, _protocols

    original_rotate = Mobject.rotate
    rotation_mapping = _protocols(g, Mobject, (
        "apply_points_function", "get_family", "__getattribute__", "__getattr__",
    ))

    @wraps(original_rotate)
    def rotate(self, angle, axis=g["_OUT"], about_point=None, **kwargs):
        # The placement shortcut is valid only for the shipping point mapper
        # and family walk. Authored implementations must receive the actual
        # native rotation map, just as they do for apply_matrix. Keep Lumen's
        # f64 placement path unchanged for ordinary objects and surfaces.
        if _changed(self, rotation_mapping):
            matrix = np.asarray(g["rotation_matrix_transpose"](float(angle), axis))
            self.apply_points_function(lambda points: np.dot(points, matrix),
                                       about_point, **kwargs)
            return self
        return original_rotate(self, angle, axis=axis, about_point=about_point, **kwargs)

    Mobject.rotate = rotate

    def set_points(self, points):
        values = _point_table(np, points)
        self.resize_points(len(values), resize_func=g["resize_preserving_order"])
        self.data["point"][:] = values
        return self

    def append_points(self, new_points):
        values = _point_table(np, new_points)
        if not len(values):
            return self
        count = self.get_num_points()
        self.resize_points(count + len(values))
        data = self.data
        # Appended records inherit the last existing record's non-point lanes.
        # An empty source instead uses resize_points' retained style defaults.
        if count:
            data[count:] = data[count - 1]
        data["point"][count:] = values
        return self

    def get_shader_data(self):
        # This historical name is a record export, not a GLSL operation.
        # Surface supplies a NumPy triangle-index array. Its truth value is
        # undefined for multiple elements (and [0] is false), so None alone
        # denotes an unindexed stream. Empty topology must export no rows,
        # including a nonempty one-row/one-column Surface grid.
        indices = self.get_shader_vert_indices()
        data = self.data
        return data if indices is None else data[indices]

    for name, method in (("set_points", set_points), ("append_points", append_points),
                         ("get_shader_data", get_shader_data)):
        method.__name__ = name
        method.__qualname__ = Mobject.__qualname__ + "." + name
        method.__module__ = Mobject.__module__
        setattr(Mobject, name, method)
    g["_FMN_POINT_EDITING_INSTALLED"] = True
