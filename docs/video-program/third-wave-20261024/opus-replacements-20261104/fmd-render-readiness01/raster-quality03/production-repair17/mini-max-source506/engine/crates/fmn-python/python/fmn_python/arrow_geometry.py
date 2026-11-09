"""Line and arrow authoring over native geometry and the VMobject lifecycle.

Python owns the recipe and public hook dispatch. Atlas still constructs the
filled/curved outline and tip; Marionette owns point publication, schemas and
live views. Rebuilding an arrow must not replace its record schema or family.
"""
from __future__ import annotations


def install_arrow_geometry(native):
    g = vars(native)
    if g.get("_FMN_ARROW_GEOMETRY_INSTALLED", False):
        return
    _install_line_regeneration(g)
    _install_stroke_arrow(g)
    Arrow, np = g["Arrow"], g["_np"]

    def arrow_init(
        self, start=g["_LEFT"], end=g["_LEFT"], buff=0.25, path_arc=0.0,
        fill_color=g["_StyleDefault"](g["_DEFAULT_LIGHT_COLOR"]), fill_opacity=1.0, stroke_width=0.0,
        thickness=3.0, tip_width_ratio=5, tip_angle=g["_math"].pi / 3,
        max_tip_length_to_length_ratio=0.5, max_width_to_length_ratio=0.1,
        **kwargs,
    ):
        self.thickness, self.tip_width_ratio = float(thickness), float(tip_width_ratio)
        self.tip_angle = float(tip_angle)
        self.max_tip_length_to_length_ratio = float(max_tip_length_to_length_ratio)
        self.max_width_to_length_ratio = float(max_width_to_length_ratio)
        # Line installs the endpoint recipe, then the native base dispatches
        # init_data/init_points/init_uniforms and VMobject runs init_colors.
        # Preserve cooperative constructor dispatch instead of running a
        # second hook pass over an already constructed native arrow.
        super(Arrow, self).__init__(
            start, end, buff=buff, path_arc=path_arc, fill_color=fill_color,
            fill_opacity=fill_opacity, stroke_width=stroke_width, **kwargs,
        )

    def init_points(self):
        # Unlike inherited Line.init_points, this cannot become a plain line.
        # Keep the public endpoint hook virtual for authored arrow subclasses.
        return self.set_points_by_ends(self.start, self.end, self.buff, self.path_arc)

    def set_points_by_ends(self, start, end, buff=0, path_arc=0):
        start, end = g["_vec3"](start), g["_vec3"](end)
        # Stage the entire native outline before touching live records. The
        # original detached builder installed a new tree, dropping custom
        # dtype lanes and init_data-owned children on every scale/rebuild.
        candidate = g["_native_shell_factory"]()
        specs, tip_index = candidate._build_arrow(
            g["_native_shell_factory"], start, end, float(buff), float(path_arc),
            self.thickness, self.tip_width_ratio, self.tip_angle,
            self.max_tip_length_to_length_ratio, self.max_width_to_length_ratio,
        )
        if specs:
            raise RuntimeError("a point-only native Arrow builder returned children")
        self.set_points(candidate.get_points())
        self.tip_index = tip_index
        self.start, self.end = np.array(start), np.array(end)
        return self

    for name, function in (("__init__", arrow_init), ("init_points", init_points),
                           ("set_points_by_ends", set_points_by_ends)):
        function.__name__ = name
        function.__qualname__ = Arrow.__qualname__ + "." + name
        function.__module__ = Arrow.__module__
        setattr(Arrow, name, function)
    _install_curved_arrows(g)
    _install_curved_tip_fitting(g)
    g["_FMN_ARROW_GEOMETRY_INSTALLED"] = True


def _bind(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _install_line_regeneration(g):
    def replace(self, start, end, buff=0.0, path_arc=0.0):
        start, end = g["_vec3"](start), g["_vec3"](end)
        style, uniforms = self.get_style(), self.uniforms.copy()
        # Detached objects also own real Marionette records. Replacing their
        # entire nursery drops custom schemas, views and hook-owned children.
        # Use the same native point writer on either side of Scene adoption.
        self._rebuild_line(start, end, float(buff), float(path_arc))
        self.set_style(**style, recurse=False)
        self.uniforms.update(uniforms)
        return self

    _bind(g["Line"], "_replace_line_geometry", replace)


def _install_stroke_arrow(g):
    Stroke = g["StrokeArrow"]

    def initialize(
        self, start, end, stroke_color=g["_StyleDefault"](g["_DEFAULT_LIGHT_COLOR"]), stroke_width=5,
        buff=0.25, tip_width_ratio=5, tip_len_to_width=0.0075,
        max_tip_length_to_length_ratio=0.3, max_width_to_length_ratio=8.0,
        **kwargs,
    ):
        self.tip_width_ratio = float(tip_width_ratio)
        self.tip_len_to_width = float(tip_len_to_width)
        self.max_tip_length_to_length_ratio = float(max_tip_length_to_length_ratio)
        self.max_width_to_length_ratio = float(max_width_to_length_ratio)
        self.n_tip_points = 3
        self.original_stroke_width = float(stroke_width)
        # Line resolves endpoint objects and runs the cooperative data/points/
        # uniforms/colors protocol. The existing tip reset keeps the profile
        # synchronized when init_colors dispatches the public set_stroke.
        super(Stroke, self).__init__(
            start, end, buff=buff, stroke_color=stroke_color,
            stroke_width=stroke_width, **kwargs,
        )

    def points(self):
        # Inherited Line.init_points would build a plain line. Atlas already
        # provides a schema-preserving stroke-arrow writer for live or nursery
        # objects, including exact taper widths and true-arclength trimming.
        return self.set_points_by_ends(self.start, self.end, self.buff, self.path_arc)

    _bind(Stroke, "__init__", initialize)
    _bind(Stroke, "init_points", points)


def _install_curved_arrows(g):
    """Keep Arc construction, endpoint fitting and tip creation cooperative.

    The Reference fits the *hook-authored* Arc only after all initialization
    phases, then adds tips through the public TipableVMobject protocol. A
    detached _build_curved_arrow tree skips those hooks and discards custom
    record schemas and children. Reuse the existing native-backed Arc and
    endpoint/tip operations instead; no path or tip geometry is computed here.
    """
    Between = g["ArcBetweenPoints"]
    Curved, Double = g["CurvedArrow"], g["CurvedDoubleArrow"]

    def between_init(self, start, end, angle=g["_math"].tau / 4, **kwargs):
        # Do not discard start_angle/radius/arc_center: although fitting
        # cancels them for an ordinary Arc, authored init_points can read
        # or change them. super() also preserves later base monkeypatches.
        super(Between, self).__init__(angle=angle, **kwargs)
        if angle == 0:
            self.set_points_as_corners([g["_LEFT"], g["_RIGHT"]])
        self.put_start_and_end_on(start, end)

    def curved_init(self, start_point, end_point, **kwargs):
        super(Curved, self).__init__(start_point, end_point, **kwargs)
        self.add_tip()

    def double_init(self, start_point, end_point, **kwargs):
        super(Double, self).__init__(start_point, end_point, **kwargs)
        self.add_tip(at_start=True)

    for cls, function in ((Between, between_init), (Curved, curved_init),
                          (Double, double_init)):
        function.__name__ = "__init__"
        function.__qualname__ = cls.__qualname__ + ".__init__"
        function.__module__ = cls.__module__
        cls.__init__ = function


def _install_curved_tip_fitting(g):
    """Fit explicit curved arrows through their public endpoint protocol.

    Generic Line and Arc retain their existing native true-arclength trimming.
    CurvedArrow/CurvedDoubleArrow use the Reference's whole-shaft fitting,
    including its distinction between raw shaft endpoints and attached tips.
    """
    Curved = g["CurvedArrow"]

    def endpoints(self):
        # Reference Mobject.get_start_and_end reads the shaft, whereas
        # get_start/get_end individually may name attached tips. Conflating
        # those contracts scales a double arrow against the wrong chord.
        self.throw_error_if_no_points()
        points = self.get_points()
        return points[0].copy(), points[-1].copy()

    def reset_endpoints(self, tip, at_start):
        if self.get_length() == 0:
            return self
        start = tip.get_base() if at_start else self.get_start()
        end = self.get_end() if at_start else tip.get_base()
        # Curved arrows fit the entire shaft to the tip base, rather than
        # slicing off a true-arclength interval. Leave the generic Tipable
        # native trimming policy unchanged for Line and ordinary Arc users.
        self.put_start_and_end_on(start, end)
        return self

    def fit_endpoints(self, start, end):
        # The native Stage shortcut fits raw arena endpoints and cannot see
        # Python tip accessors or authored scale/rotate/shift hooks. Compose
        # the same public operations in both nursery and scene-owned states.
        # Chisel/Marionette still own every point transform and live buffer.
        np, math = g["_np"], g["_math"]
        start, end = (np.array(g["_vec3"](point), dtype=float) for point in (start, end))
        if (not np.isfinite([start, end]).all()
                or np.any(np.abs([start, end]) > np.finfo(np.float32).max)):
            raise ValueError("curved-arrow endpoints must be finite and f32-representable")
        current_start, current_end = (np.array(point, dtype=float, copy=True)
                                      for point in self.get_start_and_end())
        current = current_end - current_start
        if np.all(current == 0):
            raise Exception("Cannot position endpoints of closed loop")
        target = end - start
        horizontal = math.hypot(target[0], target[1])
        # At either pole the azimuth is undefined. Choose zero consistently,
        # including signed-zero coordinates, and keep a real pitch axis. The
        # old cross-with-OUT axis was zero for every vertical destination.
        azimuth = math.atan2(target[1], target[0]) if horizontal else 0.0
        pitch_axis = np.array(
            [-target[1] / horizontal, target[0] / horizontal, 0.0]
            if horizontal else [0.0, 1.0, 0.0]
        )
        # Normalize in f64 before crossing the native seam: an almost vertical
        # chord must not underflow its axis when native geometry stores f32.
        # Retain the azimuth-then-pitch convention, rather than a shortest-arc
        # rotation which would change the authored curve's plane and tip roll.
        self.scale(np.linalg.norm(target) / np.linalg.norm(current), about_point=current_start)
        self.rotate(azimuth - math.atan2(current[1], current[0]))
        self.rotate(
            math.atan2(current[2], math.hypot(current[0], current[1]))
            - math.atan2(target[2], horizontal),
            axis=pitch_axis,
        )
        self.shift(start - self.get_start())
        return self

    _bind(Curved, "get_start_and_end", endpoints)
    _bind(Curved, "put_start_and_end_on", fit_endpoints)
    _bind(Curved, "reset_endpoints_based_on_tip", reset_endpoints)
