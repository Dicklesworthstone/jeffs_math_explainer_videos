"""In-place refresh of sampled 2D/3D paths through the native Atlas builder."""
from __future__ import annotations

from .graphing import (_BINDING, _GRAPH_UPDATES, _MAX_SAMPLES, _bind_method, _finite,
                       _records_equal)


def install_curve_regeneration(native):
    """Refresh sampled paths through Atlas without reconstructing their owner."""
    g = vars(native)
    if g.get("_FMN_CURVE_REGENERATION_INSTALLED", False):
        return
    Curve, np = g["ParametricCurve"], g["_np"]

    def controls(obj):
        from .graph_admission import _numbers
        return (_numbers(obj.t_range, "curve t_range", 3, 2),
                _finite(obj.epsilon, "curve epsilon"),
                _numbers(obj.discontinuities, "curve discontinuities", _MAX_SAMPLES),
                bool(obj.use_smoothing))

    def identity(function):
        return (getattr(function, "__func__", function),
                getattr(function, "__self__", None))

    def idle(obj):
        if vars(obj).get("_is_animating", False) or getattr(obj, "locked_data_keys", ()):
            raise RuntimeError("release the curve's active animation before regenerating it")
        if vars(obj).get(_BINDING) is not None:
            raise RuntimeError("unbind the live function graph before regenerating its parameter recipe")

    def init_points(self):
        """Replace geometry from the current t_func, range and discontinuities.

        The recipe is evaluated in scene coordinates, just as at construction.
        Earlier affine edits are not reapplied. Atlas owns all sampling and
        smoothing; publication uses the existing point-record/view protocol.
        """
        idle(self)
        if tuple(self.pointlike_data_keys) != ("point",):
            raise TypeError("curve regeneration requires the VMobject pointlike schema")
        with _GRAPH_UPDATES.hold(self, message="curve regeneration cannot reenter itself"):
            options = controls(self)
            function = self.t_func
            function_identity = identity(function)
            if not callable(function):
                raise TypeError("curve t_func must be callable")
            self.get_points()  # Observe/bake placement before taking the snapshot.
            before = self.data.copy()
            owner, bound = vars(self).get("_scene"), self._is_bound()
            family = tuple(self.get_family())
            prepare = g.get("_fmn_prepare_curve_function")
            sample, verify = (function, None) if prepare is None else prepare(function)
            # Do not recursively construct a public Curve: its init_points
            # now reaches this same protocol. Atlas still builds every sample
            # and smooth handle, on a detached candidate. Never replace self's
            # native owner (including during initial custom-dtype construction).
            candidate = g["_native_shell_factory"]()
            specs = candidate._build_parametric_curve(
                g["_native_shell_factory"], sample, options[0], options[1],
                list(options[2]), options[3],
            )
            if specs:
                raise RuntimeError("native curve sampling unexpectedly returned children")
            if verify is not None:
                verify()
            idle(self)
            current_identity = identity(self.t_func)
            if (current_identity[0] is not function_identity[0]
                    or current_identity[1] is not function_identity[1]
                    or controls(self) != options
                    or vars(self).get("_scene") is not owner or self._is_bound() != bound
                    or tuple(self.get_family()) != family
                    or tuple(self.pointlike_data_keys) != ("point",)
                    or not _records_equal(np, self.data, before)):
                raise RuntimeError("curve changed during sampling; candidate was not published")
            points = candidate.get_points()
            if not np.isfinite(points).all():
                raise ValueError("native curve sampling produced nonfinite records")
            self.set_points(points)
            if len(before) == 0:
                # Keep Atlas's initial normals and joints, including its
                # pre-quantization joint calculation at discontinuities. Later
                # refreshes retain the existing live point/paint protocol.
                for key in ("base_normal", "joint_angle"):
                    self.data[key][:] = candidate.data[key]
        return self

    _bind_method(Curve, "init_points", init_points)
    g["_FMN_CURVE_REGENERATION_INSTALLED"] = True


