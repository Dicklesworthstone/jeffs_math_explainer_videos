"""Solid constructor hooks over Surface's native sampling/publication lifecycle.

The stock UV function selects Atlas's specialized solid builder, not a Python
sampler. An authored UV function instead uses Surface's bounded callback path.
In either case only pointlike columns are published into the actual subclass's
records, so custom schemas, children, and cooperative hooks keep their owners.
"""
from __future__ import annotations

import itertools
import math

from .surface_admission import _finite


def install_solid_lifecycle(native):
    g = vars(native)
    if g.get("_FMN_SOLID_LIFECYCLE_INSTALLED", False):
        return
    if not g.get("_FMN_SURFACE_LIFECYCLE_INSTALLED", False):
        raise ImportError("solid construction requires the shared Surface lifecycle")
    Surface, Torus, Sphere, np = (g[name] for name in ("Surface", "Torus", "Sphere", "_np"))
    # Capture identities now. Later class/instance monkey patches are authored
    # functions, even if they retain a built-in method's name or signature.
    torus_uv, sphere_uv = Torus.uv_func, Sphere.uv_func
    Cylinder, Cone, Line3D = (g.get(name) for name in ("Cylinder", "Cone", "Line3D"))
    axial_types = () if Cylinder is None else (Cylinder,)
    cylinder_uv = None if Cylinder is None else Cylinder.uv_func
    cone_uv = None if Cone is None else Cone.uv_func
    Disk, Square = (g.get(name) for name in ("Disk3D", "Square3D"))
    flat_types = tuple(cls for cls in (Disk, Square) if cls is not None)
    flat_protocols = {cls: {name: getattr(cls, name, None) for name in (
        "init_data", "init_points", "init_uniforms", "init_colors", "uv_func",
        "scale", "get_center", "get_bounding_box", "get_family",
        "apply_points_function", "apply_points_function_about_point")}
        for cls in flat_types}
    placement_hooks = ("scale", "set_depth", "apply_matrix", "rescale_to_fit",
                       "get_depth", "get_center", "get_bounding_box", "get_family",
                       "apply_points_function", "apply_points_function_about_point")
    placement_protocol = {} if Cylinder is None else {
        name: getattr(Cylinder, name, None) for name in placement_hooks}
    # Invocation state, not copied/pickled attributes. A retained sphere
    # candidate exists only until the constructor finishes (also on failure).
    constructing = {}

    def admit(self):
        if self._is_bound():
            raise RuntimeError("a surface constructor requires a detached target; use init_points or become")
        if id(self) in constructing:
            raise RuntimeError("solid initialization is already in progress")

    def dimensions(self):
        if isinstance(self, axial_types):
            return (_finite(self.height, "surface height", record=True),
                    _finite(self.radius, "surface radius", record=True),
                    vector(self.axis, "surface axis"))
        keys = (("r1", "r2") if isinstance(self, Torus) else
                (("side_length",) if Square is not None and isinstance(self, Square) else ("radius",)))
        values = tuple(_finite(getattr(self, key), "surface " + key, record=True) for key in keys)
        if isinstance(self, Sphere):
            values += (bool(self.true_normals), bool(self.clockwise))
        return values

    def vector(value, name):
        values = tuple(itertools.islice(iter(value), 4))
        if len(values) != 3:
            raise ValueError(name + " must contain three coordinates")
        return tuple(_finite(v, name, record=True) for v in values)

    def cylinder(self, u_range=(0, math.tau), v_range=(-1, 1),
                 resolution=(101, 11), height=2, radius=1, axis=(0, 0, 1), **kwargs):
        admit(self)
        height = _finite(height, "surface height", record=True)
        radius = _finite(radius, "surface radius", record=True)
        axis = vector(axis, "surface axis")
        constructing[id(self)] = {}
        try:
            self.height, self.radius, self.axis = height, radius, np.array(axis)
            super(Cylinder, self).__init__(u_range=u_range, v_range=v_range,
                                          resolution=resolution, **kwargs)
            kind = "cone" if Cone is not None and isinstance(self, Cone) else "cylinder"
            self._solid_params = (kind, *dimensions(self))
            self._solid_native_height = self.get_height()
        finally:
            constructing.pop(id(self), None)

    def cone(self, u_range=(0, math.tau), v_range=(0, 1), *args, **kwargs):
        # Keep the actual Cylinder MRO and its duplicate-argument semantics.
        super(Cone, self).__init__(*args, u_range=u_range, v_range=v_range, **kwargs)

    def place_axial(target, values):
        height, radius, axis = values
        target.scale(radius)
        target.set_depth(height, stretch=True)
        target.apply_matrix(g["z_to_vector"](axis))

    def cylinder_points(self):
        context = constructing.get(id(self))
        if context is not None:
            context["axial_placed"] = False
        result = super(Cylinder, self).init_points()
        if context is not None and not context["axial_placed"]:
            # An authored UV map, transform hook or existing child family needs
            # the actual receiver's Reference placement protocol. Native stock
            # samples have already applied it in f64 and must not be scaled twice.
            place_axial(self, dimensions(self))
        return result

    def plain_axial_placement(self):
        if (type(self).__getattribute__ is not Cylinder.__getattribute__
                or getattr(type(self), "__getattr__", None) is not getattr(Cylinder, "__getattr__", None)):
            return False
        if self.submobjects:
            return False
        for name, expected in placement_protocol.items():
            method = getattr(self, name, None)
            if getattr(method, "__func__", method) is not expected:
                return False
        return True

    def line3d(self, start, end, width=.05, resolution=(21, 25), **kwargs):
        admit(self)
        start, end = (np.array(vector(v, "Line3D endpoint")) for v in (start, end))
        width = _finite(width, "Line3D width", record=True)
        axis = end - start
        super(Line3D, self).__init__(height=g["get_norm"](axis), radius=width / 2,
                                    axis=axis, resolution=resolution, **kwargs)
        # Placement belongs AFTER every construction hook, as in the Reference.
        # In particular, init_colors observes the centered cylinder and a custom
        # point-hook replacement is translated too. Do not reconstruct the root.
        self.shift((start + end) / 2)

    def flat(self, cls, key, value, u_range, v_range, resolution, kwargs):
        admit(self)
        value = _finite(value, "surface " + key, record=True)
        # Only fully unchanged protocols may use Atlas's pre-scaled f64 grid.
        # Any authored hook must see the unit grid and receive the final scale
        # AFTER init_colors; scaling before the hooks loses replacement geometry
        # and leaves children added by those hooks unscaled.
        optimized = (type(self).__getattribute__ is cls.__getattribute__
                     and getattr(type(self), "__getattr__", None) is getattr(cls, "__getattr__", None))
        if optimized:
            for name, expected in flat_protocols[cls].items():
                method = getattr(self, name, None)
                if getattr(method, "__func__", method) is not expected:
                    optimized = False
        constructing[id(self)] = dict(flat_optimized=optimized)
        try:
            setattr(self, key, value)
            super(cls, self).__init__(u_range=u_range, v_range=v_range,
                                      resolution=resolution, **kwargs)
            if not optimized:
                self.scale(value / 2 if cls is Square else value)
            self._solid_params = ("square" if cls is Square else "disk", getattr(self, key))
            self._solid_native_height = self.get_height()
        finally:
            constructing.pop(id(self), None)

    def disk(self, radius=1, u_range=(0, 1), v_range=(0, math.tau),
             resolution=(2, 100), **kwargs):
        flat(self, Disk, "radius", radius, u_range, v_range, resolution, kwargs)

    def square(self, side_length=2., u_range=(-1, 1), v_range=(-1, 1),
               resolution=(2, 2), **kwargs):
        flat(self, Square, "side_length", side_length, u_range, v_range, resolution, kwargs)

    def torus(self, u_range=(0, math.tau), v_range=(0, math.tau),
              r1=3.0, r2=1.0, **kwargs):
        admit(self)
        radii = tuple(_finite(value, "surface " + key, record=True)
                      for key, value in (("r1", r1), ("r2", r2)))
        constructing[id(self)] = {}
        try:
            self.r1, self.r2 = radii
            super(Torus, self).__init__(u_range=u_range, v_range=v_range, **kwargs)
            self._solid_params = ("torus", self.r1, self.r2)
            self._solid_native_height = self.get_height()
        finally:
            constructing.pop(id(self), None)

    def radial_normals(self, radius, nudge, preserved=None):
        if radius == 0:
            # Atlas's existing degenerate-sphere rule avoids division by zero.
            return
        nudge = _finite(nudge, "surface normal_nudge", record=True)
        if nudge < 0:
            raise ValueError("surface normal_nudge must be nonnegative")
        if preserved is not None:
            # Atlas computes radial normals before narrowing its sampled f64
            # points. Do not recompute untouched stock columns from f32 records:
            # that double rounding can change an otherwise identical frame.
            # World-space publication normalizes -0 to +0. That is not an
            # authored geometry edit: byte comparison here used to discard the
            # f64-built normals and double-round hundreds of nonzero values.
            if all(np.array_equal(self.data[key], preserved.data[key])
                   for key in ("point", "d_normal_point")):
                return
        if vars(self).get("_is_animating", False) or getattr(self, "locked_data_keys", ()):
            raise RuntimeError("release the sphere's active animation before correcting its normals")
        factor = (radius + nudge) / radius
        with np.errstate(over="ignore", invalid="ignore"):
            normals = np.asarray(self.data['point'], dtype=np.float64) * factor
        if normals.shape != self.data['d_normal_point'].shape:
            raise ValueError("sphere normal points must match the native record shape")
        if (not np.isfinite(normals).all()
                or np.any(np.abs(normals) > np.finfo(np.float32).max)):
            raise ValueError("sphere normal points must remain finite and f32-representable")
        # Validate the complete column before replacing any of it. As in the
        # Reference this final correction also applies to an authored UV map or
        # a point-hook replacement, and happens after init_colors has run.
        self.data['d_normal_point'][:] = normals

    def sphere(self, u_range=(0, math.tau), v_range=(0, math.pi),
               resolution=(101, 51), radius=1.0, true_normals=True,
               clockwise=False, **kwargs):
        admit(self)
        radius = _finite(radius, "surface radius", record=True)
        true_normals, clockwise = bool(true_normals), bool(clockwise)
        context = dict(radius=radius, true_normals=true_normals)
        constructing[id(self)] = context
        try:
            self.radius, self.true_normals, self.clockwise = radius, true_normals, clockwise
            super(Sphere, self).__init__(u_range=u_range, v_range=v_range,
                                        resolution=resolution, **kwargs)
            if true_normals:
                preserved = context.get("candidate")
                if (context.get("sampled_radius") != radius
                        or context.get("sampled_nudge") != self.normal_nudge):
                    preserved = None
                radial_normals(self, radius, self.normal_nudge, preserved)
            self._solid_params = ("sphere", self.radius)
        finally:
            constructing.pop(id(self), None)

    def build_candidate(self, options, sample):
        if not isinstance(self, (Torus, Sphere, *axial_types, *flat_types)):
            return sample()
        values = dimensions(self)
        function = getattr(self.uv_func, "__func__", None)
        is_torus = isinstance(self, Torus)
        is_axial = isinstance(self, axial_types)
        is_cone = is_axial and Cone is not None and isinstance(self, Cone)
        is_flat = isinstance(self, flat_types)
        is_square = is_flat and Square is not None and isinstance(self, Square)
        expected = (flat_protocols[Square if is_square else Disk]["uv_func"] if is_flat else
                    ((cone_uv if is_cone else cylinder_uv) if is_axial else
                     (torus_uv if is_torus else sphere_uv)))
        stock = function is expected
        context = constructing.get(id(self))
        if is_axial and context is not None and not plain_axial_placement(self):
            stock = False
        if stock:
            candidate = Surface.__new__(Surface)
            g["_install_live_state"](candidate)
            if is_flat:
                size = values[0]
                if context is not None and not context["flat_optimized"]:
                    size = 2. if is_square else 1.
                builder = candidate._build_square3d if is_square else candidate._build_disk3d
                specs = builder(g["_native_surface_shell_factory"], size,
                    options["u_range"], options["v_range"], options["resolution"],
                    options["preferred_creation_axis"], options["epsilon"],
                    options["normal_nudge"], 0)
            elif is_axial:
                builder = candidate._build_cone if is_cone else candidate._build_cylinder
                specs = builder(g["_native_surface_shell_factory"], *values,
                    options["u_range"], options["v_range"], options["resolution"],
                    options["preferred_creation_axis"], options["epsilon"],
                    options["normal_nudge"], 0)
            elif is_torus:
                specs = candidate._build_torus(
                    g["_native_surface_shell_factory"], *values,
                    options["u_range"], options["v_range"], options["resolution"],
                    options["preferred_creation_axis"], options["epsilon"],
                    options["normal_nudge"], 0)
            else:
                radius, true_normals, clockwise = values
                if context is not None:
                    # The constructor's true_normals/radius arguments govern
                    # its final correction, even if init_data edits the recipe.
                    true_normals = (context["true_normals"]
                                    and radius == context["radius"])
                specs = candidate._build_sphere(
                    g["_native_surface_shell_factory"], radius,
                    options["u_range"], options["v_range"], options["resolution"],
                    true_normals, clockwise, options["preferred_creation_axis"],
                    options["epsilon"], options["normal_nudge"])
            if specs:
                raise RuntimeError("a solid sampler returned unexpected children")
        else:
            candidate = sample()
            if is_axial and context is None:
                # The native UV sampler supplies the authored object-space map.
                # Complete Cylinder's recipe through Marionette's existing
                # pointlike transforms, before any live geometry is published.
                # This also serves set_resolution and explicit regeneration.
                place_axial(candidate, values)
            elif is_flat and context is None:
                candidate.scale(values[0] / 2 if is_square else values[0])
        if dimensions(self) != values:
            raise RuntimeError("solid shape parameters changed during sampling; geometry was not published")
        if is_axial and context is not None:
            context["axial_placed"] = stock
        if isinstance(self, Sphere):
            if context is not None:
                context.update(candidate=candidate if stock else None,
                               sampled_radius=values[0], sampled_nudge=options["normal_nudge"])
            elif values[1] and not stock:
                # A live authored-UV rebuild has no constructor epilogue. Apply
                # the same correction to its detached candidate before publish.
                radial_normals(candidate, values[0], options["normal_nudge"])
        return candidate

    constructors = [(Torus, torus), (Sphere, sphere)]
    constructors.extend((cls, method) for cls, method in (
        (Cylinder, cylinder), (Cone, cone), (Line3D, line3d),
        (Disk, disk), (Square, square)) if cls is not None)
    if Cylinder is not None:
        # surface_geometry replaced the old Cylinder no-op before the shared
        # construction protocol was installed. Follow the current Surface hook,
        # which distinguishes initial publication from live regeneration.
        cylinder_points.__name__, cylinder_points.__qualname__, cylinder_points.__module__ = (
            "init_points", Cylinder.__qualname__ + ".init_points", Cylinder.__module__)
        Cylinder.init_points = cylinder_points
    for cls, constructor in constructors:
        constructor.__name__, constructor.__qualname__, constructor.__module__ = (
            "__init__", cls.__qualname__ + ".__init__", cls.__module__)
        cls.__init__ = constructor
    g["_fmn_build_solid_candidate"] = build_candidate
    g["_FMN_SOLID_LIFECYCLE_INSTALLED"] = True
