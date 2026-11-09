"""Keep the public glow uniform connected to the native point-cloud material.

Lumen stores glow in a per-record lane. Writing the public scalar uniform must
broadcast into that lane (or the empty cloud's retained default), not only into
Python's extras dictionary. Direct per-record writes remain per-record writes.
"""
from __future__ import annotations

from collections.abc import Mapping
from functools import wraps
import math


def _glow(value):
    value = float(value)
    if not math.isfinite(value) or not 0 <= value <= 3.4028234663852886e38:
        raise ValueError("glow_factor must be finite, non-negative and f32-representable")
    return value


class _UniformSource(Mapping):
    """Hide only the redundant scalar projection from the shared record lerp."""
    def __init__(self, source):
        self.source = source

    def __getitem__(self, key):
        if key == "glow_factor":
            raise KeyError(key)
        return self.source.uniforms[key]

    def __contains__(self, key):
        return key != "glow_factor" and key in self.source.uniforms

    def __iter__(self):
        return (key for key in self.source.uniforms if key != "glow_factor")

    def __len__(self):
        return sum(1 for _ in self)


class _RecordSource:
    """Read-through adapter, used only inside the stock record interpolator.

    Authored methods still receive the original objects. Properties execute on
    their original receiver; callbacks see the live, unmodified uniform maps.
    No data, identity graph or animation state is copied or retained here.
    """
    def __init__(self, source):
        self.source = source
        self.uniforms = _UniformSource(source)

    def __getattr__(self, name):
        return getattr(self.source, name)


def install_point_cloud_materials(native):
    g = vars(native)
    if g.get("_FMN_POINT_CLOUD_MATERIALS_INSTALLED", False):
        return
    if g.get("_FMN_RASTER_TRANSFORM_INSTALLED", False):
        raise ImportError("point-cloud materials must initialize before raster Transform adapters")
    DotCloud, Uniforms, np = g["DotCloud"], g["_LiveUniforms"], g["_np"]
    previous = Uniforms.__setitem__
    Mobject = g["Mobject"]
    previous_interpolate = Mobject.interpolate
    missing = object()
    float32 = np.dtype(np.float32)

    @wraps(previous)
    def setitem(self, key, value):
        owner = self._owner()
        if key != "glow_factor" or not isinstance(owner, DotCloud):
            return previous(self, key, value)
        value = _glow(value)
        # Resolve the actual record field before changing the mapping. The
        # native view protocol owns revisions and CoW; no geometry is rebuilt.
        records = owner._style_data()
        if "glow_factor" not in (records.dtype.names or ()):
            raise ValueError("point-cloud records have no native glow_factor field")
        column = records["glow_factor"]
        if column.dtype != float32 or column.shape != (len(records), 1):
            raise ValueError("native glow_factor requires one float32 lane per point")
        if not column.flags.writeable:
            raise ValueError("native glow_factor field is not writable")
        prior = self._extras.get(key, missing)
        previous(self, key, value)
        try:
            column[:] = value
        except BaseException:
            # In particular, an expired/locked native view must not leave a
            # success-shaped Python uniform after refusing the native write.
            if prior is missing:
                self._extras.pop(key, None)
            else:
                self._extras[key] = prior
            raise

    def set_glow_factor(self, glow_factor):
        value = _glow(glow_factor)
        self.uniforms["glow_factor"] = value
        self.glow_factor = value
        return self

    @wraps(previous_interpolate)
    def interpolate(self, mobject1, mobject2, alpha, path_func=None):
        if not isinstance(self, DotCloud):
            return previous_interpolate(self, mobject1, mobject2, alpha, path_func)
        # The one shared interpolator already blends every native glow lane.
        # Reapplying its scalar uniform projection would destroy heterogeneous
        # per-point glow. Mask only that redundant endpoint key for this call,
        # without changing the real endpoints, locks, maps or authored paths.
        result = previous_interpolate(self, _RecordSource(mobject1),
                                      _RecordSource(mobject2), alpha, path_func)
        if ("glow_factor" not in getattr(self, "locked_uniform_keys", ())
                and all("glow_factor" in obj.uniforms for obj in (self, mobject1, mobject2))):
            self.uniforms._extras["glow_factor"] = g["_interpolate"](
                mobject1.uniforms["glow_factor"], mobject2.uniforms["glow_factor"], float(alpha))
        return result

    Mobject.interpolate = interpolate
    set_glow_factor.__name__ = "set_glow_factor"
    set_glow_factor.__qualname__ = DotCloud.__qualname__ + ".set_glow_factor"
    set_glow_factor.__module__ = DotCloud.__module__
    Uniforms.__setitem__ = setitem
    DotCloud.set_glow_factor = set_glow_factor
    g["_FMN_POINT_CLOUD_MATERIALS_INSTALLED"] = True
