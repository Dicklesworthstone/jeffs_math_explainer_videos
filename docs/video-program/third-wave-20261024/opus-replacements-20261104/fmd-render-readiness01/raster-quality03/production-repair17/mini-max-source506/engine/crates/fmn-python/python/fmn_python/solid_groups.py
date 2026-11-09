"""Compound solids assembled through the existing native-backed group protocol.

Atlas owns each face's geometry. Public face factories, group initialization,
placement and style hooks act on the actual objects that enter the scene.
No replacement root, second geometry kernel or animation clock is introduced.
"""
from __future__ import annotations

from contextlib import contextmanager
from itertools import islice
import math
import sys

from .copying import FamilyRefs

_MAX_FAMILY = 65_536
_MAX_RECORDS = 16_777_216


def _finite(value, name):
    value = float(value)
    if not math.isfinite(value) or abs(value) > 3.4028234663852886e38:
        raise ValueError(name + " must be finite and f32-representable")
    return value


def _bind(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _faces(g, owner, values, root_type):
    """Freeze and admit a bounded, detached face DAG before group adoption."""
    faces = list(islice(iter(values), _MAX_FAMILY + 1))
    if len(faces) > _MAX_FAMILY:
        raise ValueError("solid face factory exceeds the 65536-member budget")
    if len({id(face) for face in faces}) != len(faces):
        raise ValueError("solid face factory returned duplicate roots")
    if any(not isinstance(face, root_type) for face in faces):
        raise TypeError("solid face factory returned an incompatible mobject")
    pending = [(face, False) for face in reversed(faces)]
    seen, visiting, members, records = set(), set(), [], 0
    while pending:
        member, leaving = pending.pop()
        marker = id(member)
        if leaving:
            visiting.remove(marker)
            continue
        if marker in visiting:
            raise ValueError("solid face factory returned a cyclic family")
        if marker in seen:
            continue
        if member is owner or not isinstance(member, g["Mobject"]):
            raise ValueError("solid face family contains its owner or an invalid member")
        if member._is_bound() or getattr(member, "_scene", None) is not None:
            raise ValueError("solid face factory must return detached objects; copy scene objects first")
        if len(seen) >= _MAX_FAMILY:
            raise ValueError("solid face family exceeds the 65536-member budget")
        seen.add(marker)
        visiting.add(marker)
        members.append(member)
        records += member.get_num_points()
        if records > _MAX_RECORDS:
            raise ValueError("solid face family exceeds the 16777216-record budget")
        if not g["_np"].isfinite(member.get_points()).all():
            raise ValueError("solid face geometry must be finite")
        children = list(islice(iter(member.submobjects), _MAX_FAMILY + 1))
        if len(children) > _MAX_FAMILY or len(pending) + len(children) > 2 * _MAX_FAMILY:
            raise ValueError("solid face family exceeds the 65536-member budget")
        pending.append((member, True))
        pending.extend((child, False) for child in reversed(children))
    # Authored point getters can execute Python; check ownership once more.
    if any(member._is_bound() or getattr(member, "_scene", None) is not None for member in members):
        raise ValueError("solid face ownership changed during admission")
    return faces


def install_solid_groups(native):
    g = vars(native)
    if g.get("_FMN_SOLID_GROUPS_INSTALLED", False):
        return
    Cube, Prism, Surface = (g[name] for name in ("Cube", "Prism", "Surface"))
    from .surface_admission import grid_shape

    active = set()

    @contextmanager
    def constructing(obj):
        if obj._is_bound():
            raise RuntimeError("solid construction requires a detached target")
        if id(obj) in active:
            raise RuntimeError("solid construction is already in progress")
        active.add(id(obj))
        try:
            yield
        finally:
            active.remove(id(obj))

    def symbol(name):
        # Like the Reference, resolve the helper in the class's defining module.
        # A later authored replacement is not captured away by installation.
        module = sys.modules.get(Cube.__module__)
        return getattr(module, name, g[name])

    def cube(self, color=None, opacity=1, shading=(0.1, 0.5, 0.1),
             square_resolution=(2, 2), side_length=2, **kwargs):
        depth_test = bool(kwargs.pop("depth_test", True))
        z_index = int(kwargs.pop("z_index", 0))
        g["_refuse_surface_keywords"](type(self).__name__ + "()", kwargs)
        shape = grid_shape(square_resolution, copies=6)
        side_length = _finite(side_length, "Cube side_length")
        opacity = _finite(1.0 if opacity is None else opacity, "Cube opacity")
        shading = tuple(_finite(v, "Cube shading") for v in islice(iter(
            (.1, .5, .1) if shading is None else shading), 4))
        if len(shading) != 3:
            raise ValueError("Cube shading must have three components")
        if not -(1 << 31) <= z_index < (1 << 31):
            raise ValueError("solid z_index must fit a signed 32-bit integer")
        with constructing(self):
            self.square_resolution = shape
            self.side_length = side_length
            face = symbol("Square3D")(side_length=side_length, resolution=shape,
                color=g["BLUE"] if color is None else color, opacity=opacity,
                shading=shading, depth_test=depth_test)
            _faces(g, self, [face], Surface)
            faces = _faces(g, self, symbol("square_to_cube_faces")(face), Surface)
            # SGroup initializes an empty native Surface root, then adds faces.
            # Run that same lifecycle, including any hook-owned decorations.
            super(Cube, self).__init__(color=g["WHITE"] if color is None else color,
                opacity=opacity, shading=shading, depth_test=depth_test, z_index=z_index)
            if self._is_bound():
                raise RuntimeError("solid ownership changed during initialization")
            faces = _faces(g, self, faces, Surface)
            self.add(*faces)
            # The legacy portal exposes the per-face shape on Cube.resolution.
            # Keep that metadata; the point-free root itself is not a UV face.
            self.resolution = shape
            # FamilyRefs, not an object ndarray: an ndarray is invisible to the
            # cycle collector, so Cube -> faces -> parents -> Cube never freed.
            self._solid_faces = FamilyRefs(faces)
            self._index_faces()

    def index_faces(self):
        for face in getattr(self, "_solid_faces", self.submobjects):
            shape = g["_surface_grid_resolution"](face)
            if shape is not None:
                face.resolution = shape
                face.compute_triangle_indices()

    def prism(self, width=3.0, height=2.0, depth=1.0, **kwargs):
        dimensions = tuple(_finite(value, "Prism " + name) for name, value in
                           (("width", width), ("height", height), ("depth", depth)))
        super(Prism, self).__init__(**kwargs)
        for dim, length in enumerate(dimensions):
            self.rescale_to_fit(length, dim, stretch=True)

    for cls, name, method in ((Cube, "__init__", cube), (Cube, "_index_faces", index_faces),
                              (Prism, "__init__", prism)):
        _bind(cls, name, method)
    g["_FMN_SOLID_GROUPS_INSTALLED"] = True
