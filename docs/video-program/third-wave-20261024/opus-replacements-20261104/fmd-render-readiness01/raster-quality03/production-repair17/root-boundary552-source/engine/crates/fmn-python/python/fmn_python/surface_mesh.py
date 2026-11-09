"""Live wireframes over Atlas's sampled-surface mesh constructor.

The existing native builder owns normals, UV interpolation and smooth paths.
This adapter refreshes the generated wire family without replacing its scene
identity, discarding styles, or accumulating duplicate lines on every tick.
"""
from __future__ import annotations

import copy
import itertools
import math
import operator

from .surface_admission import GRID_BUDGET

_KEY = "_fmn_surface_wire"
_BUSY = "_fmn_mesh_regenerating"
# Atlas's wireframe output budget (lines and wire samples); the source grid
# is a dense UV chart bounded by GRID_BUDGET instead.
_BUDGET = 65_536


def _resolution(values, bound=_BUDGET):
    shape = tuple(operator.index(v) for v in itertools.islice(iter(values), 3))
    if len(shape) != 2 or any(v < 0 for v in shape) or any(v > bound for v in shape):
        raise ValueError("SurfaceMesh resolution requires two nonnegative bounded line counts")
    return shape


def install_surface_mesh(native):
    g = vars(native)
    if g.get("_FMN_SURFACE_MESH_INSTALLED", False):
        return
    Mesh, Surface, Mobject, VMobject, np = (g[name] for name in
        ("SurfaceMesh", "Surface", "Mobject", "VMobject", "_np"))
    original_init = Mesh.__init__

    def controls(source, shape, nudge):
        shape, nudge = _resolution(shape), float(nudge)
        if sum(shape) > _BUDGET:
            raise ValueError("SurfaceMesh line counts exceed the native budget")
        if not math.isfinite(nudge):
            raise ValueError("SurfaceMesh normal_nudge must be finite")
        if not isinstance(source, Surface):
            raise TypeError("SurfaceMesh source must be a Surface")
        source_shape = _resolution(source.resolution, GRID_BUDGET)
        count = source_shape[0] * source_shape[1]
        if (count > GRID_BUDGET or count != source.n_records()
                or shape[0] * source_shape[1] + shape[1] * source_shape[0] > _BUDGET):
            raise ValueError("SurfaceMesh source topology or wire samples exceed the native budget")
        # The triangle-grid seam deliberately refuses singleton/strip grids;
        # the existing wireframe sampler supports these degenerate charts.
        if min(source_shape) >= 2 and tuple(g["_surface_grid_resolution"](source) or ()) != source_shape:
            raise ValueError("SurfaceMesh requires matching native UV topology")
        return shape, nudge, source_shape

    def tag(wires, shape):
        keys = [(axis, index) for axis in range(2) for index in range(shape[axis])]
        if wires and len(wires) != len(keys):
            raise RuntimeError("native SurfaceMesh produced an inconsistent wire family")
        for wire, key in zip(wires, keys):
            vars(wire)[_KEY] = key

    def init(self, uv_surface, resolution=(21, 11), stroke_width=1,
             stroke_color=None, normal_nudge=.01, depth_test=True,
             joint_type="no_joint", **kwargs):
        shape, nudge, _ = controls(uv_surface, resolution, normal_nudge)
        original_init(self, uv_surface, resolution=shape, stroke_width=stroke_width,
                      stroke_color=stroke_color, normal_nudge=nudge,
                      depth_test=depth_test, joint_type=joint_type, **kwargs)
        self.resolution, self.normal_nudge = shape, nudge
        tag(self.submobjects, shape)
        # Keep a native style seed even when the requested density is empty.
        # Newly added wires inherit current per-axis styles, or this root seed.
        VMobject.set_stroke(self, color=g["GREY_A"] if stroke_color is None else stroke_color,
                           width=stroke_width, opacity=1, recurse=False)
        # The root fill is VMobject's default, as in the Reference (fm-78b6);
        # fill_color= and color= are refused before this point.
        VMobject.set_fill(self, color=g["_pinned_manim_config"]().vmobject.default_fill_color,
                          opacity=0, recurse=False)

    def idle(members):
        if any(vars(member).get("_is_animating", False) or
               getattr(member, "locked_data_keys", ()) for member in members):
            raise RuntimeError("release the wireframe's active animation before regenerating geometry")

    def init_points(self):
        if vars(self).get(_BUSY, False):
            raise RuntimeError("wireframe regeneration is already in progress")
        source = self.uv_surface
        shape, nudge, source_shape = controls(source, self.resolution, self.normal_nudge)
        children = tuple(self.submobjects)
        owned = {}
        for child in children:
            key = vars(child).get(_KEY)
            if key is not None:
                if (not isinstance(child, VMobject) or not isinstance(key, tuple)
                        or len(key) != 2 or any(type(v) is not int for v in key)
                        or key[0] not in (0, 1) or key[1] < 0 or key in owned):
                    raise ValueError("wireframe generated-line identities are inconsistent")
                owned[key] = child
        idle((self, *owned.values()))
        owner, bound = vars(self).get("_scene"), self._is_bound()
        source_owner, source_bound = vars(source).get("_scene"), source._is_bound()
        family = tuple(self.get_family())
        # Own snapshots before invoking authored get_points/get_unit_normals.
        # The native builder executes those callbacks with no Stage borrow.
        before = [(wire, wire.data.copy()) for wire in owned.values()]
        source_before = source.data.copy()
        vars(self)[_BUSY] = True
        try:
            candidate = Mesh(source, resolution=shape, normal_nudge=nudge)
            generated = tuple(candidate.submobjects)
            desired = {vars(wire)[_KEY]: wire for wire in generated}
            for key, wire in owned.items():
                if key in desired and wire.data.dtype != desired[key].data.dtype:
                    raise ValueError("wireframe regeneration requires compatible path records")
            for wire in generated:
                points = np.asarray(wire.get_points())
                if (points.ndim != 2 or points.shape[1] != 3 or not np.isfinite(points).all()
                        or np.any(np.abs(points) > np.finfo(np.float32).max)):
                    raise ValueError("wireframe geometry must be finite and f32-representable")
            if (self.uv_surface is not source or self._is_bound() != bound
                    or vars(self).get("_scene") is not owner
                    or tuple(self.get_family()) != family or tuple(self.submobjects) != children
                    or controls(source, self.resolution, self.normal_nudge) != (shape, nudge, source_shape)
                    or source._is_bound() != source_bound
                    or vars(source).get("_scene") is not source_owner
                    or not np.array_equal(source.data, source_before)
                    or any(not np.array_equal(wire.data, data) for wire, data in before)):
                raise RuntimeError("surface or wireframe changed during regeneration; no mesh was published")
            idle((self, *owned.values()))
            # Prepare styles for newly added wires before touching the live
            # family. Existing wires retain their own full native style records.
            for key, wire in desired.items():
                if key not in owned:
                    template = next((old for old_key, old in owned.items() if old_key[0] == key[0]),
                                    next(iter(owned.values()), self))
                    VMobject.match_style(wire, template, recurse=False)
                    wire.uniforms.update(copy.deepcopy(dict(template.uniforms)))
            for key, wire in owned.items():
                if key in desired:
                    Mobject.match_points(wire, desired[key])
                    wire._refresh_vmobject_path_metadata()
                    wire.needs_new_unit_normal = True
            # Preserve authored children and the order of all surviving wires.
            # Removed wires detach; objects held by callers are never deleted.
            survivors = [child for child in children
                         if _KEY not in vars(child) or vars(child)[_KEY] in desired]
            additions = [wire for key, wire in desired.items() if key not in owned]
            if additions or len(survivors) != len(children):
                candidate.set_submobjects([])
                self.set_submobjects([*survivors, *additions])
            self.resolution, self.normal_nudge = shape, nudge
        finally:
            vars(self).pop(_BUSY, None)
        return None

    for name, method in (("__init__", init), ("init_points", init_points)):
        method.__name__ = name
        method.__qualname__ = Mesh.__qualname__ + "." + name
        method.__module__ = Mesh.__module__
        setattr(Mesh, name, method)
    g["_FMN_SURFACE_MESH_INSTALLED"] = True
