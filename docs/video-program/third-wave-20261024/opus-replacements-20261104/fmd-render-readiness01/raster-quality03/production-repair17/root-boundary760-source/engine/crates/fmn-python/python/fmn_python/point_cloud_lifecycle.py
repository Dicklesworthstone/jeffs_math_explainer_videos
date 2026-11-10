"""Point-cloud authoring through the shared native record initialization hooks.

The native record initializer selects the renderer primitive before any hook.
Authored schemas, rows, placement, children and updater ownership are retained;
there is no replacement object and no second point renderer.
"""
from __future__ import annotations

import math


def _finite(value, name):
    value = float(value)
    if not math.isfinite(value) or abs(value) > 3.4028234663852886e38:
        raise ValueError(name + " must be finite and f32-representable")
    return value


def install_point_cloud_lifecycle(native):
    g = vars(native)
    if g.get("_FMN_POINT_CLOUD_LIFECYCLE_INSTALLED", False):
        return
    initialize = g.get("_initialize_point_cloud")
    validate = g.get("_validate_point_cloud")
    if not callable(initialize) or not callable(validate):
        raise ImportError("native point-cloud initialization seam is missing")
    PMobject, PGroup, DotCloud, GlowDots = (
        g[name] for name in ("PMobject", "PGroup", "DotCloud", "GlowDots"))
    np = g["_np"]
    origin = g.get("NULL_POINTS", np.zeros((1, 3)))
    active = set()

    def engine_init(self):
        initialize(self)

    def pmobject(self, **kwargs):
        super(PMobject, self).__init__(**kwargs)
        # Base Mobject owns the native init_data/init_points/init_uniforms
        # dispatch. Like VMobject and Surface, this concrete family then runs
        # the public color hook, on the actual receiver and its own records.
        self.init_colors()

    def group(self, *pmobs, **kwargs):
        if not all(isinstance(mob, PMobject) for mob in pmobs):
            raise Exception("All submobjects must be of type PMobject")
        super(PGroup, self).__init__(**kwargs)
        self.add(*pmobs)

    def cloud(self, points=origin, color=g["GREY_C"], opacity=1.0,
              radius=.05, glow_factor=0.0, anti_alias_width=2.0, **kwargs):
        if self._is_bound():
            raise RuntimeError("point-cloud construction requires a detached target")
        if id(self) in active:
            raise RuntimeError("point-cloud construction is already in progress")
        values = tuple(_finite(value, name) for name, value in (
            ("radius", radius), ("glow_factor", glow_factor),
            ("anti_alias_width", anti_alias_width)))
        if min(values) < 0:
            raise ValueError("point-cloud radius, glow factor and anti-alias width must be non-negative")
        opacity = _finite(opacity, "point-cloud opacity")
        active.add(id(self))
        try:
            self.radius, self.glow_factor, self.anti_alias_width = values
            # The existing native cloud schema carries glow in a record lane
            # (pointcloud.rs). Extend the authored dtype, never replace it with
            # Atlas's stock schema. Native parsing still validates all fields.
            dtype = list(self.data_dtype)
            if not any(field[0] == "glow_factor" for field in dtype):
                self.data_dtype = [*dtype, ("glow_factor", np.float32, (1,))]
            super(DotCloud, self).__init__(color=color, opacity=opacity, **kwargs)
            self.set_radius(self.radius)
            if points is not None:
                # Reference order: supplied points replace hook-produced points
                # AFTER init_colors/set_radius. Explicit None retains them.
                self.set_points(points)
            glow = _finite(self.get_glow_factor(), "point-cloud glow factor")
            if glow < 0:
                raise ValueError("point-cloud glow factor must be non-negative")
            data = self.data if self.get_num_points() else self._style_data()
            data["glow_factor"][:] = glow
            validate(self)
        finally:
            active.discard(id(self))

    def glows(self, points=origin, color=g["YELLOW"], radius=.2,
              glow_factor=2.0, **kwargs):
        super(GlowDots, self).__init__(points, color=color, radius=radius,
                                     glow_factor=glow_factor, **kwargs)

    for cls, method in ((PMobject, pmobject), (PGroup, group),
                        (DotCloud, cloud), (GlowDots, glows)):
        method.__name__ = "__init__"
        method.__qualname__ = cls.__qualname__ + ".__init__"
        method.__module__ = cls.__module__
        cls.__init__ = method
    engine_init.__name__ = "_engine_init"
    engine_init.__qualname__ = DotCloud.__qualname__ + "._engine_init"
    engine_init.__module__ = DotCloud.__module__
    DotCloud._engine_init = engine_init
    g["_FMN_POINT_CLOUD_LIFECYCLE_INSTALLED"] = True
