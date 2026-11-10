"""Authored tracker construction over Marionette's typed scalar/complex state.

Allocate the native record schema once, then dispatch the public Mobject hooks.
The uniform initializer publishes values through native setters, never through
ControlMobject.set_value or a float32 point-field encoding.
"""
from __future__ import annotations


def _keep_control_dynamic(mobject):
    """Keep controls active during waits without an unpickleable closure."""
    # The dynamic marker is the Reference control contract. A module-level
    # callable also survives the normal graph copier and pickle protocol.
    return None


def install_tracker_lifecycle(native):
    g = vars(native)
    if g.get("_FMN_TRACKER_LIFECYCLE_INSTALLED", False):
        return
    ValueTracker, Complex = g["ValueTracker"], g["ComplexValueTracker"]
    Control = g["ControlMobject"]
    np = g["_np"]
    allocate = g["_portal_allocate_tracker"]

    def init_uniforms(self):
        # Follow the Reference's mutable constructor seed. An init_data hook
        # may replace self.value before this hook, and explicit later calls
        # reset the uniform from that seed through the same typed authority.
        super(ValueTracker, self).init_uniforms()
        self.uniforms["shading"] = self.shading
        kind = self._tracker_kind
        if kind == 2:
            value = complex(self.value)
            components = np.array([value], dtype=self.value_type)
            self._set_tracker_complex_value(value.real, value.imag)
        else:
            if np.iscomplexobj(self.value):
                raise TypeError("a scalar ValueTracker cannot store complex values")
            components = np.array(self.value, dtype=self.value_type).reshape(-1)
            if not len(components):
                raise ValueError(type(self).__name__ + " needs at least one value")
            if len(components) > 1 and kind != 0:
                raise TypeError("a vector value needs ValueTracker")
            self._set_tracker_value(float(components[0]))
            if len(components) == 1:
                # Retain ExponentialValueTracker's existing decoded-uniform
                # convention, including the native logarithm/exp round trip.
                components[0] = self._tracker_value()
        self.uniforms["value"] = components

    def initialize(self, value=0, **kwargs):
        allowed = {"color", "opacity", "shading", "texture_paths",
                   "is_fixed_in_frame", "depth_test", "z_index"}
        unknown = kwargs.keys() - allowed
        if unknown:
            raise TypeError("unexpected keyword arguments: " + ", ".join(sorted(unknown)))
        g["_install_live_state"](self)
        self.value = value
        self.color = kwargs.get("color", g["_WHITE"])
        self.opacity = kwargs.get("opacity", 1.0)
        self.shading = kwargs.get("shading", (0.0, 0.0, 0.0))
        self.texture_paths = kwargs.get("texture_paths")
        self.depth_test = kwargs.get("depth_test", False)
        allocate(self, self._tracker_kind)
        self.z_index = kwargs.get("z_index", 0)
        # Uniforms precede points so scalar, complex and vector values are
        # available to authored geometry hooks. No native builder follows
        # these hooks and therefore none can discard their work.
        self.init_data()
        self.init_uniforms()
        self.init_updaters()
        self.init_event_listners()
        self.init_points()
        self.init_colors()
        if self.depth_test:
            self.apply_depth_test()
        if kwargs.get("is_fixed_in_frame", False):
            self.fix_in_frame()

    def complex_initialize(self, value=0, **kwargs):
        super(Complex, self).__init__(value, **kwargs)

    def complex_uniforms(self):
        super(Complex, self).init_uniforms()

    def control_initialize(self, value, *mobjects, **kwargs):
        if not all(isinstance(mobject, g["Mobject"]) for mobject in mobjects):
            raise TypeError("ControlMobject children must be Mobject instances")
        # Native widget factories create independent child shells before this
        # call, not the tracker root. Attach them only after authored hooks;
        # neither their colors nor an authored child should be replaced.
        super(Control, self).__init__(value, **kwargs)
        self.add(*mobjects)
        self.add_updater(_keep_control_dynamic)
        self.fix_in_frame()

    for cls, methods in (
        (ValueTracker, (("__init__", initialize), ("init_uniforms", init_uniforms))),
        (Complex, (("__init__", complex_initialize), ("init_uniforms", complex_uniforms))),
        (Control, (("__init__", control_initialize),)),
    ):
        for name, method in methods:
            method.__name__ = name
            method.__qualname__ = cls.__qualname__ + "." + name
            method.__module__ = cls.__module__
            setattr(cls, name, method)
    g["_FMN_TRACKER_LIFECYCLE_INSTALLED"] = True
