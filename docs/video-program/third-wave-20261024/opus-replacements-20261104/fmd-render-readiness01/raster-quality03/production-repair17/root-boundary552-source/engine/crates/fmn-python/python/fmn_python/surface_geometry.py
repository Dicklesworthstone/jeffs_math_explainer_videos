"""Live UV-surface regeneration through Atlas, with native geometry publication.

Handles, child identities, materials, scene membership and updaters retain their
owners. Explicit set_resolution rebuilds the UV recipe while native regridding
preserves paint and texture fields; init_points retains its fixed-grid contract.
"""
from __future__ import annotations

import itertools
import math
import operator
from types import SimpleNamespace

from .surface_admission import GRID_BUDGET

_METADATA = ("u_range", "v_range", "epsilon", "normal_nudge", "preferred_creation_axis")


def _controls(surface):
    resolution = tuple(operator.index(v) for v in itertools.islice(iter(surface.resolution), 3))
    if len(resolution) != 2 or any(v < 2 for v in resolution) or math.prod(resolution) > GRID_BUDGET:
        raise ValueError(f"surface regeneration requires a UV grid of 2..{GRID_BUDGET} points with both axes >= 2")
    domains = []
    for name in ("u_range", "v_range"):
        pair = tuple(float(v) for v in itertools.islice(iter(getattr(surface, name)), 3))
        if len(pair) != 2 or not all(math.isfinite(v) for v in pair) or pair[0] == pair[1]:
            raise ValueError(name + " must contain two distinct finite bounds")
        domains.append(pair)
    epsilon, nudge = float(surface.epsilon), float(surface.normal_nudge)
    if not math.isfinite(epsilon) or epsilon <= 0 or not math.isfinite(nudge) or nudge < 0:
        raise ValueError("surface epsilon must be positive; normal_nudge must be nonnegative; both must be finite")
    if any(not math.isfinite(value + epsilon) for pair in domains for value in pair):
        raise ValueError("surface derivative samples must remain finite")
    return resolution, *domains, epsilon, nudge



def _solid_recipe(g, surface, controls):
    """Reuse specialized Atlas constructors, including analytic sphere normals.

    Unit UV functions do not encode cylinder radius/height/axis or disk/square
    dimensions. Sampling those functions alone would silently discard shape
    parameters. An authored UV override instead retains the generic callback path.
    """
    common = dict(resolution=controls[0], u_range=controls[1], v_range=controls[2],
                  epsilon=controls[3], normal_nudge=controls[4],
                  preferred_creation_axis=operator.index(surface.preferred_creation_axis))
    function = getattr(surface.uv_func, "__func__", None)
    for name in ("Sphere", "Torus", "Cone", "Cylinder", "Disk3D", "Square3D"):
        cls = g[name]
        if not isinstance(surface, cls) or function is not cls.uv_func:
            continue
        if name == "Sphere":
            values = dict(radius=float(surface.radius), true_normals=bool(surface.true_normals),
                          clockwise=bool(surface.clockwise))
        elif name == "Torus":
            values = dict(r1=float(surface.r1), r2=float(surface.r2))
        elif name in ("Cone", "Cylinder"):
            axis = tuple(float(value) for value in surface.axis)
            if len(axis) != 3 or not all(math.isfinite(value) for value in axis):
                raise ValueError("surface axis must contain three finite coordinates")
            values = dict(height=float(surface.height), radius=float(surface.radius), axis=axis)
        elif name == "Disk3D":
            values = dict(radius=float(surface.radius))
        else:
            values = dict(side_length=float(surface.side_length))
        if any(type(value) is float and not math.isfinite(value) for value in values.values()):
            raise ValueError("surface shape parameters must be finite")
        return cls, dict(common, **values)
    return None


def install_surface_geometry(native):
    g = vars(native)
    if g.get("_FMN_SURFACE_GEOMETRY_INSTALLED", False):
        return
    for name in ("_surface_grid_resolution", "_copy_surface_geometry", "_regrid_surface_geometry"):
        if not callable(g.get(name)):
            raise ImportError("native surface geometry seam is missing: " + name)
    Surface, Textured, Geometry, Mobject, np = (g[name] for name in
        ("Surface", "TexturedSurface", "TexturedGeometry", "Mobject", "_np"))
    original_become = Mobject.become
    triangles = Surface.compute_triangle_indices
    # An active operation belongs to this invocation, not serializable scene
    # state. Copying a surface during a callback must not copy a permanent lock.
    # IDs cannot be reused while the corresponding call retains its self.
    active = set()

    def resolution(self):
        result = g["_surface_grid_resolution"](self)
        if result is None:
            raise TypeError("surface regeneration requires an actual native UV grid")
        return result

    def idle(self):
        if vars(self).get("_is_animating", False) or getattr(self, "locked_data_keys", ()):
            raise RuntimeError("release the surface's active animation before regenerating its geometry")

    def regenerate(self, requested=None):
        """Resample the current UV recipe, preserving native identity and style.

        Designed for explicit rebuilds and ValueTracker-driven updaters. Like
        construction, sampling evaluates the current UV function in its own
        coordinates; previous affine geometry edits are not reapplied. Grid
        changes are admitted only by set_resolution. Errors publish no geometry;
        arbitrary side effects made by the callback itself are not rolled back.
        """
        if id(self) in active:
            raise RuntimeError("surface regeneration is already in progress")
        if isinstance(self, Geometry):
            raise TypeError("indexed TexturedGeometry is not a UV-grid surface")
        shape = tuple(operator.index(v) for v in itertools.islice(iter(self.resolution), 3))
        if requested is None and len(shape) == 2 and 0 in shape and all(0 <= v <= GRID_BUDGET for v in shape):
            # Empty Surface construction calls this hook through _engine_init.
            # There are no samples to regenerate, and no triangle grid to copy.
            if self.n_records():
                raise ValueError("surface resolution no longer matches its native records")
            return None
        idle(self)
        original_controls = _controls(self)
        if original_controls[0] != resolution(self):
            raise ValueError("surface resolution changed; use set_resolution() instead of assigning topology")
        controls = original_controls if requested is None else (requested, *original_controls[1:])
        topology = None
        if controls[0] != original_controls[0]:
            topology = dict(resolution=controls[0], triangle_indices=triangles(SimpleNamespace(resolution=controls[0])))
        active.add(id(self))
        try:
            # Source records are retained before callbacks; a callback that edits
            # the destination must not have its changes overwritten by this build.
            before = self.data.copy()
            owner = vars(self).get("_scene")
            bound = self._is_bound()
            family = tuple(self.get_family())
            uv_method = self.uv_func
            uv_identity = (getattr(uv_method, "__func__", uv_method),
                           getattr(uv_method, "__self__", None))
            solid = None if isinstance(self, Textured) else _solid_recipe(g, self, controls)
            recipe = getattr(self, "passed_uv_func", None)
            verify_sampling = None
            if solid is not None:
                cls, options = solid
                candidate = cls(**options)
            elif isinstance(self, Textured):
                candidate = self.uv_surface
                if not isinstance(candidate, Surface) or candidate is self:
                    raise TypeError("TexturedSurface requires a distinct source Surface")
                if resolution(candidate) != controls[0]:
                    raise ValueError("textured surface and source require matching native UV topology")
            else:
                function = self.uv_func
                recipe = getattr(self, "passed_uv_func", None)
                if getattr(function, "__func__", None) is g["ParametricSurface"].uv_func:
                    function = recipe
                if not callable(function):
                    raise TypeError("surface uv_func must be callable")
                prepare = g.get("_fmn_prepare_surface_function")
                if prepare is not None:
                    function, verify_sampling = prepare(function)
                failure = None
                def sample(u, v):
                    nonlocal failure
                    if failure is not None:
                        raise failure
                    try:
                        value = np.asarray(function(u, v))
                        if value.shape != (3,) or value.dtype.kind not in "fiu":
                            raise ValueError("surface uv_func must return three real coordinates")
                        value = np.asarray(value, dtype=float)
                        if not np.isfinite(value).all() or np.any(np.abs(value) > np.finfo(np.float32).max):
                            raise ValueError("surface uv_func must return finite f32-representable coordinates")
                        return tuple(value)
                    except BaseException as error:
                        failure = error
                        raise
                def sample_candidate():
                    # Bypass Python constructors, not Atlas: its native sampler
                    # owns the finite-difference normal convention and grid.
                    result = Surface.__new__(Surface)
                    g["_install_live_state"](result)
                    shape, u_range, v_range, epsilon, nudge = controls
                    specs = result._build_parametric_surface(g["_native_surface_shell_factory"],
                                sample, u_range, v_range, shape, epsilon, nudge)
                    g["_hang_native_children"](result, specs)
                    return result
                build_solid = g.get("_fmn_build_solid_candidate")
                sample_options = dict(resolution=controls[0], u_range=controls[1],
                    v_range=controls[2], epsilon=controls[3], normal_nudge=controls[4],
                    preferred_creation_axis=self.preferred_creation_axis)
                candidate = (sample_candidate() if build_solid is None else
                             build_solid(self, sample_options, sample_candidate))
            if verify_sampling is not None:
                verify_sampling()
            idle(self)
            if solid is not None and _solid_recipe(g, self, controls) != solid:
                raise RuntimeError("surface shape parameters changed during regeneration")
            if not isinstance(self, Textured) and getattr(self, "passed_uv_func", None) is not recipe:
                raise RuntimeError("surface UV function changed during regeneration")
            current_uv = self.uv_func
            if (getattr(current_uv, "__func__", current_uv) is not uv_identity[0]
                    or getattr(current_uv, "__self__", None) is not uv_identity[1]):
                raise RuntimeError("surface UV function changed during regeneration")
            # Detached proxies have no _scene attribute. Check native admission
            # as well as its host mirror: a callback may bind this very object.
            if (vars(self).get("_scene") is not owner or self._is_bound() != bound
                    or tuple(self.get_family()) != family
                    or _controls(self) != original_controls or resolution(self) != original_controls[0]
                    or not np.array_equal(self.data, before)):
                raise RuntimeError("surface changed during regeneration; sampled geometry was not published")
            g["_copy_surface_geometry" if topology is None else "_regrid_surface_geometry"](self, candidate)
            if topology is not None:
                vars(self).update(topology)
            if solid is not None:
                for key in ("_solid_params", "_solid_native_height"):
                    if key in vars(candidate):
                        vars(self)[key] = vars(candidate)[key]
        finally:
            active.discard(id(self))
        return None

    def init_points(self):
        """Regenerate the current UV recipe without changing its grid shape."""
        return regenerate(self)

    def set_resolution(self, resolution):
        """Resample the current recipe at a new UV resolution, retaining identity.

        Paint, opacity, UVs and custom records are interpolated from the old
        normalized UV chart. Geometry and normals are freshly sampled. A
        TexturedSurface reads its uv_surface, which must already have the new
        native shape; it never mutates or evaluates that source implicitly.
        """
        if id(self) in active:
            raise RuntimeError("surface regeneration is already in progress")
        # Bound even an arbitrary iterable before allocation or UV evaluation.
        from .surface_admission import grid_shape
        shape = grid_shape(resolution)
        if min(shape) < 2:
            raise ValueError("live surface regridding requires both UV dimensions >= 2")
        regenerate(self, shape)
        return self

    def resolution_animation(self, *args, **kwargs):
        raise g["_CapabilityError"](
            "UV resolution changes are discrete; use set_resolution() in a scene updater, "
            "or Transform to a separately sampled surface for geometric interpolation"
        )

    set_resolution._override_animate = resolution_animation

    def become(self, mobject, match_updaters=False):
        other = mobject
        # Marionette already transfers durable primitive topology. Publish its
        # Python grid projection as well: UV queries and triangle-index helpers
        # must not reshape the new native records using the old dimensions.
        planned = {}
        if isinstance(other, Mobject):
            for source in other.get_family():
                if isinstance(source, Surface):
                    shape = g["_surface_grid_resolution"](source)
                    if shape is not None:
                        metadata = {key: getattr(source, key) for key in _METADATA if hasattr(source, key)}
                        for key in ("u_range", "v_range"):
                            if key in metadata:
                                metadata[key] = tuple(metadata[key])
                        metadata["resolution"] = tuple(shape)
                        holder = SimpleNamespace(resolution=shape)
                        metadata["triangle_indices"] = triangles(holder)
                        planned[id(source)] = metadata
        result = original_become(self, other, match_updaters=match_updaters)
        for receiver, source in zip(self.get_family(), other.get_family()):
            metadata = planned.get(id(source))
            if isinstance(receiver, Surface) and metadata is not None:
                vars(receiver).update(metadata)
        return result

    for cls, name, method in ((Surface, "init_points", init_points),
                               (Surface, "set_resolution", set_resolution),
                               (Mobject, "become", become)):
        method.__name__, method.__qualname__, method.__module__ = name, cls.__qualname__ + "." + name, cls.__module__
        setattr(cls, name, method)
    # Cylinder historically overrides the Surface initializer with a no-op.
    # Cone and Line3D inherit it; route the same method without another sampler.
    g["Cylinder"].init_points = Surface.init_points
    g["_FMN_SURFACE_GEOMETRY_INSTALLED"] = True
