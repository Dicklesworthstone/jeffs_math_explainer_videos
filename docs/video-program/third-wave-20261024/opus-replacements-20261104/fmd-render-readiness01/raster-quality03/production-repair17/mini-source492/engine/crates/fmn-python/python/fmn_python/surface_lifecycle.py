"""Authored Surface initialization over Atlas sampling and native UV topology.

Initial construction and live regeneration have different admission contracts:
empty grids/strips are legal initially; live regridding keeps its existing rules.
No callback, geometry sampler, frame clock or material is emulated here.
"""
from __future__ import annotations

from .surface_admission import sampling_options


def install_surface_lifecycle(native):
    g = vars(native)
    if g.get("_FMN_SURFACE_LIFECYCLE_INSTALLED", False):
        return
    if not callable(g.get("_initialize_surface_grid")):
        raise ImportError("native surface initialization seam is missing")
    Surface, Parametric = (g[name] for name in ("Surface", "ParametricSurface"))
    regenerate = Surface.init_points
    # Invocation state, not copied/pickled attributes. A callback may create a
    # distinct nested surface, but cannot recursively initialize its own owner.
    constructing, sampling = set(), set()

    def idle(self):
        if vars(self).get("_is_animating", False) or getattr(self, "locked_data_keys", ()):
            raise RuntimeError("release the surface's active animation before initializing its geometry")

    def controls(self):
        return sampling_options({name: getattr(self, name) for name in
            ("resolution", "u_range", "v_range", "epsilon", "normal_nudge",
             "preferred_creation_axis")})

    def initialize(self, color=g["GREY"], shading=(.3, .2, .4), depth_test=True,
                   u_range=(0, 1), v_range=(0, 1), resolution=(101, 101),
                   preferred_creation_axis=1, epsilon=.001, normal_nudge=.001,
                   **kwargs):
        if self._is_bound():
            raise RuntimeError("a surface constructor requires a detached target; use init_points or become")
        if id(self) in constructing:
            raise RuntimeError("surface initialization is already in progress")
        opacity = kwargs.pop("opacity", None)
        z_index = int(kwargs.pop("z_index", 0))
        # A keyword outside the Reference's Surface/Mobject chain is its own
        # TypeError; an accepted keyword Atlas does not route yet is refused.
        g["_refuse_surface_keywords"](type(self).__name__ + "()", kwargs)
        options = sampling_options(dict(resolution=resolution, u_range=u_range,
            v_range=v_range, preferred_creation_axis=preferred_creation_axis,
            epsilon=epsilon, normal_nudge=normal_nudge))
        seeds = dict(color=g["GREY"] if color is None else color,
                     opacity=1.0 if opacity is None else opacity,
                     shading=(.3, .2, .4) if shading is None else shading,
                     depth_test=bool(depth_test), z_index=z_index)
        constructing.add(id(self))
        try:
            # The Reference's Surface keeps its sampling controls itself and
            # passes only Mobject's own keywords up (fm-5wq.20).
            # _install_live_state leaves these attributes alone.
            for name, value in options.items():
                setattr(self, name, value)
            # Mobject installs live fields, then the native engine dispatches
            # init_data, init_points and init_uniforms through the actual MRO.
            # Seeds follow _install_live_state so it cannot reset their values.
            super(Surface, self).__init__(**seeds)
            # An authored hook may replace sampling entirely. Its actual table
            # must still gain durable native topology before entering a Scene.
            shape = controls(self)["resolution"]
            idle(self)
            g["_initialize_surface_grid"](self, shape)
            self.compute_triangle_indices()
            self.set_z_index(self.z_index)
            self.init_colors()
        finally:
            constructing.discard(id(self))

    def parametric(self, uv_func, u_range=(0, 1), v_range=(0, 1), **kwargs):
        if self._is_bound():
            raise RuntimeError("a surface constructor requires a detached target; use init_points or become")
        if id(self) in constructing:
            raise RuntimeError("surface initialization is already in progress")
        if not callable(uv_func):
            raise TypeError("surface function must be callable")
        self.passed_uv_func = uv_func
        super(Parametric, self).__init__(u_range=u_range, v_range=v_range, **kwargs)

    def init_uniforms(self):
        super(Surface, self).init_uniforms()
        # This is the Surface hook, not a post-hook overwrite: overrides which
        # call super() can deliberately replace these values afterwards.
        self.set_shading(*self.shading)
        (self.apply_depth_test if self.depth_test else self.deactivate_depth_test)()

    def init_points(self):
        if id(self) not in constructing:
            return regenerate(self)
        if id(self) in sampling:
            raise RuntimeError("surface initialization sampling is already in progress")
        sampling.add(id(self))
        try:
            options = controls(self)
            idle(self)
            before = self.data.copy()
            owner, bound = vars(self).get("_scene"), self._is_bound()
            family = tuple(id(member) for member in self.get_family())
            function = self.uv_func
            identity = (getattr(function, "__func__", function),
                        getattr(function, "__self__", None))
            recipe = getattr(self, "passed_uv_func", None)
            if identity[0] is Parametric.uv_func:
                function = recipe
            if not callable(function):
                raise TypeError("surface uv_func must be callable")
            verify = None
            prepare = g.get("_fmn_prepare_surface_function")
            if prepare is not None:
                function, verify = prepare(function)
            def sample():
                candidate = Surface.__new__(Surface)
                g["_install_live_state"](candidate)
                specs = candidate._build_parametric_surface(
                    g["_native_surface_shell_factory"], function,
                    options["u_range"], options["v_range"], options["resolution"],
                    options["epsilon"], options["normal_nudge"])
                if specs:
                    raise RuntimeError("a UV surface sampler returned unexpected children")
                return candidate

            # Stock solids keep their specialized Atlas kernels; authored UV
            # overrides use this same bounded sampler and publication contract.
            build_solid = g.get("_fmn_build_solid_candidate")
            candidate = sample() if build_solid is None else build_solid(self, options, sample)
            if verify is not None:
                verify()
            idle(self)
            current = self.uv_func
            if (vars(self).get("_scene") is not owner or self._is_bound() != bound
                    or tuple(id(member) for member in self.get_family()) != family
                    or controls(self) != options
                    or getattr(self, "passed_uv_func", None) is not recipe
                    or getattr(current, "__func__", current) is not identity[0]
                    or getattr(current, "__self__", None) is not identity[1]
                    or self.data.dtype != before.dtype
                    or self.data.tobytes() != before.tobytes()):
                raise RuntimeError("surface changed during initialization; sampled geometry was not published")
            g["_initialize_surface_grid"](self, options["resolution"], candidate)
        finally:
            sampling.discard(id(self))
        return None

    for cls, name, method in ((Surface, "__init__", initialize),
                              (Parametric, "__init__", parametric),
                              (Surface, "init_uniforms", init_uniforms),
                              (Surface, "init_points", init_points)):
        method.__name__, method.__qualname__, method.__module__ = (
            name, cls.__qualname__ + "." + name, cls.__module__)
        setattr(cls, name, method)
    g["_FMN_SURFACE_LIFECYCLE_INSTALLED"] = True
