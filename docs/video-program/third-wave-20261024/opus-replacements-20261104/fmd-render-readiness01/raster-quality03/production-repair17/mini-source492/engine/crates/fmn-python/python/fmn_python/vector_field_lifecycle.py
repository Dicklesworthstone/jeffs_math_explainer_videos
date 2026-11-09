"""Authored vector-field construction over VMobject and native field updates.

Preparation and initial paint remain observable through the public protocol.
Atlas owns all vector geometry; this module only sequences receiver hooks.
"""
from __future__ import annotations

import math
import operator
from typing import Any

from .invocation import InvocationGuard
from .vector_fields import _bind, _finite_nonnegative, _rows


def install_vector_field_lifecycle(native: Any) -> None:
    """Construct authored fields through VMobject, then populate live vectors.

    The unchanged concrete class retains its one-sample construction.
    Authored subclasses use the Reference lifecycle: preparation, base hooks,
    final stroke configuration, then the public update_vectors operation.
    """
    g = vars(native)
    if g.get("_FMN_VECTOR_FIELD_LIFECYCLE_INSTALLED", False):
        return
    if not g.get("_FMN_VECTOR_FIELDS_INSTALLED", False):
        raise ImportError("vector field construction requires live field operations")
    from .movement import _changed, _protocols

    Field, np = g["VectorField"], g["_np"]
    constructing = InvocationGuard()
    hooks = ("init_data", "init_points", "init_uniforms", "init_colors",
             "update_sample_points", "init_base_stroke_width_array", "update_vectors",
             "_evaluate_outputs", "_geometry_inputs", "_build_geometry", "_apply_callback_style",
             "set_points", "set_stroke", "set_stroke_width", "get_family",
             "__getattribute__", "__getattr__")
    protocols = _protocols(g, g["VMobject"], hooks)

    def initialize(self, func, coordinate_system, sample_coords=None, density=2.0,
                   magnitude_range=None, color=None, color_map_name="3b1b_colormap",
                   color_map=None, stroke_opacity=1.0, stroke_width=3.0,
                   tip_width_ratio=4.0, tip_len_to_width=0.01, max_vect_len=None,
                   max_vect_len_to_step_size=0.8, flat_stroke=False,
                   norm_to_opacity_func=None, **kwargs):
        if self._is_bound():
            raise RuntimeError("a field constructor requires a detached target; use update_vectors")
        with constructing.hold(self, message="cannot reenter vector-field construction"):
            stock = type(self) is Field and not _changed(self, protocols)
            if not callable(func):
                raise TypeError("VectorField func must be callable")
            if color is None and color_map is not None and not callable(color_map):
                raise TypeError("VectorField color_map must be callable")
            if norm_to_opacity_func is not None and not callable(norm_to_opacity_func):
                raise TypeError("VectorField norm_to_opacity_func must be callable")
            if color is None and color_map is None and color_map_name not in (None, "3b1b_colormap"):
                raise NotImplementedError(
                    "VectorField color_map_name requires a non-bundled matplotlib "
                    f"map: {color_map_name!r}; pass color=, color_map=, or "
                    "color_map_name=None"
                )
            g["_preflight_vmobject_style_kwargs"](kwargs)
            stroke_width = _finite_nonnegative(stroke_width, "stroke_width")
            stroke_opacity = _finite_nonnegative(stroke_opacity, "stroke_opacity")
            tip_width_ratio = _finite_nonnegative(tip_width_ratio, "tip_width_ratio")
            tip_len_to_width = _finite_nonnegative(tip_len_to_width, "tip_len_to_width")
            coordinates = (g["_vector_field_sample_coords"](coordinate_system, density)
                           if sample_coords is None else sample_coords)
            coordinates = _rows(np, coordinates, "VectorField sample_coords")
            if len(coordinates) < 2:
                raise ValueError("VectorField needs at least two sample points")
            # Preparation is observable to public sample-point and width hooks,
            # before VMobject allocates this receiver's declared native schema.
            self.func, self.coordinate_system, self.sample_coords = func, coordinate_system, coordinates
            self.stroke_width, self.stroke_opacity = stroke_width, stroke_opacity
            self.tip_width_ratio, self.tip_len_to_width = tip_width_ratio, tip_len_to_width
            self.flat_stroke, self.color = bool(flat_stroke), color
            self.norm_to_opacity_func = norm_to_opacity_func
            self._native_default_color_map = (color is None and color_map is None
                                              and color_map_name == "3b1b_colormap")
            self.color_map = (None if color is not None else color_map if color_map is not None
                              else g["_vector_field_default_color_map"]
                              if self._native_default_color_map else None)
            self.update_sample_points()
            points = _rows(np, self.sample_points, "VectorField sample points", columns=3,
                           count=len(coordinates))
            if max_vect_len is None:
                length = _finite_nonnegative(max_vect_len_to_step_size, "max_vect_len_to_step_size")
                length *= float(np.linalg.norm(points[1] - points[0]))
            else:
                dimension = operator.index(getattr(coordinate_system, "dimension", 2))
                if not 1 <= dimension <= 3:
                    raise ValueError("VectorField coordinate-system dimension must be in 1..3")
                origin = np.asarray(coordinate_system.c2p(*([0.] * dimension)), dtype=float)
                unit = np.asarray(coordinate_system.c2p(1., *([0.] * (dimension - 1))), dtype=float)
                if origin.shape != (3,) or unit.shape != (3,):
                    raise ValueError("VectorField coordinate conversion must return a 3-vector")
                length = float(max_vect_len) * float(np.linalg.norm(unit - origin))
            if math.isnan(length) or length <= 0:
                raise ValueError("VectorField max displayed length must be positive and non-NaN")
            self.max_displayed_vect_len = length
            outputs = None
            if stock or magnitude_range is None:
                outputs = _rows(np, self._evaluate_outputs(), "VectorField callback output",
                                count=len(coordinates))
                if magnitude_range is None:
                    norms = np.linalg.norm(outputs, axis=1)
                    magnitude_range = (0., float(norms.max(initial=0.)))
            low, high = (float(value) for value in magnitude_range)
            if not math.isfinite(low) or not math.isfinite(high) or high < low:
                raise ValueError("VectorField magnitude_range must be finite and ordered")
            self.magnitude_range = (low, high)
            self.init_base_stroke_width_array(len(coordinates))
            kwargs.setdefault("joint_type", "no_joint")
            super(Field, self).__init__(stroke_opacity=stroke_opacity, stroke_width=stroke_width,
                                        flat_stroke=flat_stroke, **kwargs)
            # Native live-state installation seeds color; restore the field's
            # declared material choice before its geometry builder consumes it.
            self.color = color
            self.set_stroke(color=color, width=stroke_width, opacity=stroke_opacity, flat=flat_stroke)
            # Ordinary construction already sampled to infer its color range.
            # Publish those exact samples through the same live-update owner:
            # no second callback, and no set_stroke pass erasing attenuation.
            # Authored hooks always dispatch their public final operation.
            if stock and not _changed(self, protocols):
                g["_fmn_update_field_from_samples"](self, outputs)
            else:
                self.update_vectors()

    _bind(Field, "__init__", initialize)
    g["_FMN_VECTOR_FIELD_LIFECYCLE_INSTALLED"] = True
