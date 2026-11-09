"""Coordinate-aware calculus visuals over existing native path operations.

Python owns graph queries and coordinate dispatch. Chisel and Marionette own
curve construction, clipping, record buffers, placement and rendered geometry.
"""
from __future__ import annotations

import copy
import itertools
import math

from fmn_python.graphing import _MAX_RECORDS, _MAX_SAMPLES, _bind_method, _finite


def _riemann_grid(values, default_step):
    """Bound the request before evaluating authored functions or allocating paths."""
    values = tuple(itertools.islice(iter(values), 4))
    if len(values) not in (2, 3):
        raise ValueError("Riemann x_range requires two or three values")
    start, stop = (_finite(value, "Riemann range endpoint") for value in values[:2])
    step = _finite(values[2] if len(values) == 3 else default_step, "Riemann dx")
    if stop < start or step <= 0:
        raise ValueError("Riemann range must not decrease and dx must be positive")
    if start == stop:
        return ()
    ratio = (stop - start) / step
    if not math.isfinite(ratio) or ratio > _MAX_SAMPLES:
        raise ValueError("Riemann rectangles exceed their 65536-item budget")
    # An exact number of steps may round one ulp above an integer. Do not add
    # an almost-zero last rectangle, but always retain the authored endpoint.
    count = max(1, math.ceil(math.nextafter(ratio, -math.inf)))
    boundaries = [start + index * step for index in range(count)] + [stop]
    if any(right <= left for left, right in zip(boundaries, boundaries[1:])):
        raise ValueError("Riemann dx is too small to advance the range")
    return tuple(zip(boundaries, boundaries[1:]))


def install_graph_calculus(native):
    g = vars(native)
    if g.get("_FMN_GRAPH_CALCULUS_INSTALLED"):
        return
    Coordinates, np = g["CoordinateSystem"], g["_np"]

    def point(value):
        result = np.asarray(value)
        if (result.shape != (3,) or result.dtype.kind not in "biuf"
                or not np.isfinite(result).all()
                or np.any(np.abs(result) > np.finfo(np.float32).max)):
            raise ValueError("calculus geometry requires three finite float32-representable coordinates")
        return result.astype(float, copy=True)

    def get_riemann_rectangles(
        self, graph, x_range=None, dx=None, input_sample_type="left",
        stroke_width=1, stroke_color=None, fill_opacity=1, colors=None,
        negative_color=None, stroke_background=True, show_signed_area=True,
    ):
        if input_sample_type not in ("left", "right", "center"):
            raise ValueError("input_sample_type must be 'left', 'right', or 'center'")
        step = self.x_range[2] if dx is None else dx
        grid = _riemann_grid(self.x_range[:2] if x_range is None else x_range, step)
        width = _finite(stroke_width, "Riemann stroke_width")
        opacity = _finite(fill_opacity, "Riemann fill_opacity")
        if not 0 <= width <= np.finfo(np.float32).max or not 0 <= opacity <= 1:
            raise ValueError("Riemann stroke_width must be nonnegative and fill_opacity in [0,1]")
        # The pinned Reference's default gradient is (BLUE, GREEN).
        palette = tuple(itertools.islice(
            iter((g["BLUE"], g["GREEN"]) if colors is None else colors),
            _MAX_SAMPLES + 1))
        if not palette or len(palette) > _MAX_SAMPLES:
            raise ValueError("Riemann colors must be a nonempty bounded iterable")
        stroke_color = g["BLACK"] if stroke_color is None else stroke_color
        negative_color = g["RED"] if negative_color is None else negative_color
        # Validate colors before any graph callback; keep the actual palette
        # for the existing linear-light native style/gradient owner.
        for color in (*palette, stroke_color, negative_color):
            rgba = np.asarray(g["color_to_rgb"](color))
            if not np.isfinite(rgba).all():
                raise ValueError("Riemann colors must be finite")

        rectangles = []
        for left, right in grid:
            sample = (left if input_sample_type == "left" else right
                      if input_sample_type == "right" else left + (right - left) / 2)
            queried = self.i2gp(sample, graph)
            if queried is None:
                raise ValueError("Riemann sample lies outside the graph or in a discontinuity")
            coordinates = np.asarray(self.p2c(point(queried)))
            if (coordinates.ndim != 1 or len(coordinates) < 2
                    or coordinates.dtype.kind not in "biuf" or not np.isfinite(coordinates).all()):
                raise ValueError("Riemann graph inverse coordinates must contain finite x and y")
            # Exact baseline queries stay zero even when the inverse of a
            # rotated chart has a small floating-point residual at y=0.
            height = (0. if np.array_equal(queried, self.c2p(sample, 0))
                      else float(coordinates[1]))
            corners = [point(self.c2p(x, y)) for x, y in (
                (left, 0), (right, 0), (right, height), (left, height), (left, 0))]
            # Preserve Rectangle identity, but build its path in the axes'
            # chart, not a screen-aligned bounding box. Chisel still owns
            # corner conversion, records, placement and rasterization.
            rectangle = g["Rectangle"]()
            rectangle.set_points_as_corners(corners)
            rectangle.positive = height >= 0
            rectangles.append(rectangle)
        result = g["VGroup"](*rectangles)
        result.set_submobject_colors_by_gradient(*palette)
        result.set_style(stroke_width=width, stroke_color=stroke_color,
                         fill_opacity=opacity, stroke_behind=stroke_background)
        if show_signed_area:
            for rectangle in rectangles:
                if not rectangle.positive:
                    rectangle.set_fill(negative_color)
        return result

    _bind_method(Coordinates, "get_riemann_rectangles", get_riemann_rectangles)
    _install_graph_areas(g, point)
    g["_FMN_GRAPH_CALCULUS_INSTALLED"] = True


def _install_graph_areas(g, point):
    Coordinates, VMobject, np = g["CoordinateSystem"], g["VMobject"], g["_np"]

    def coordinate(axes, p):
        coords = np.asarray(axes.p2c(p.copy()))
        if (coords.ndim != 1 or len(coords) < 2 or coords.dtype.kind not in "biuf"
                or not np.isfinite(coords).all()):
            raise ValueError("graph area inverse coordinates must contain finite x and y")
        return float(coords[0])

    def parameter(axes, path, xs, index, x):
        """Invert data x, using the shipped evaluator rather than another curve kernel."""
        start, stop = xs[2 * index], xs[2 * index + 2]
        if x <= start:
            return 0.
        if x >= stop:
            return 1.
        curve = g["bezier"](path[2 * index:2 * index + 3])
        low, high = 0., 1.
        best, error = 0., abs(start - x)
        for _ in range(64):
            middle = (low + high) / 2
            if middle == low or middle == high:
                break
            value = coordinate(axes, curve(middle))
            if abs(value - x) < error:
                best, error = middle, abs(value - x)
            if value == x:
                return middle
            if value < x:
                low = middle
            else:
                high = middle
        if error > 8 * np.finfo(np.float32).eps * max(1., abs(x), abs(start), abs(stop)):
            raise ValueError("graph area boundary could not be resolved within float32 precision")
        return best

    def get_area_under_graph(self, graph, x_range=None, fill_color=None, fill_opacity=.5):
        if not isinstance(graph, VMobject):
            raise TypeError("get_area_under_graph requires a VMobject graph")
        bounds = None
        if x_range is not None:
            values = tuple(itertools.islice(iter(x_range), 3))
            if len(values) != 2:
                raise ValueError("graph area x_range requires exactly two endpoints")
            bounds = tuple(_finite(value, "graph area endpoint") for value in values)
            if bounds[1] < bounds[0]:
                raise ValueError("graph area x_range must not decrease")
        opacity = _finite(fill_opacity, "graph area fill_opacity")
        if not 0 <= opacity <= 1:
            raise ValueError("graph area fill_opacity must be in [0,1]")
        color = g["BLUE"] if fill_color is None else fill_color  # Reference fill_color=BLUE
        if not np.isfinite(np.asarray(g["color_to_rgb"](color))).all():
            raise ValueError("graph area fill_color must be finite")
        size = graph.get_num_points()
        if size > _MAX_RECORDS or size < 0 or (size and size % 2 != 1):
            raise ValueError("graph area requires a bounded shared-anchor path")
        points = np.asarray(graph.get_points())
        if (points.shape != (size, 3) or points.dtype.kind not in "biuf"
                or not np.isfinite(points).all()
                or np.any(np.abs(points) > np.finfo(np.float32).max)):
            raise ValueError("graph area requires finite real path coordinates")
        # Freeze the current drawn geometry, not an analytic function which
        # may have changed since the last scene tick. No graph updater or
        # function is run, and no source live view is written or resized.
        points = points.astype(float, copy=True)
        result = VMobject()
        result.uniforms.update(copy.deepcopy(dict(graph.uniforms)))
        paths = VMobject.get_subpaths_from_points(result, points)
        total = 0
        for path in paths:
            if len(path) < 3 or (bounds is not None and bounds[0] == bounds[1]):
                continue
            xs = np.array([coordinate(self, p) for p in path])
            if xs[0] > xs[-1]:
                path, xs = path[::-1].copy(), xs[::-1].copy()
            tolerance = 8 * np.finfo(np.float32).eps * max(1., float(np.max(np.abs(xs))))
            # For the affine axes chart, monotone quadratic control x values
            # certify a function graph. Refuse foldbacks rather than silently
            # choosing a branch and painting invented area.
            ordered = np.maximum.accumulate(xs)
            if np.any(xs < ordered - tolerance):
                raise ValueError("graph area requires x-monotone subpaths; the path folds back")
            xs = ordered
            left = xs[0] if bounds is None else max(xs[0], bounds[0])
            right = xs[-1] if bounds is None else min(xs[-1], bounds[1])
            if right <= left:
                continue
            anchors, curves = xs[::2], len(path) // 2
            first = min(curves - 1, max(0, int(np.searchsorted(anchors, left, side="right")) - 1))
            last = min(curves - 1, max(0, int(np.searchsorted(anchors, right, side="left")) - 1))
            a = parameter(self, path, xs, first, left)
            b = parameter(self, path, xs, last, right)
            # The native partial-reveal owner clips exact quadratic handles.
            # Compact only its collapsed flanks; never resample the curve.
            total += 2 * (last - first + 1) + 7 + (2 if total else 0)
            if total > _MAX_RECORDS:
                raise ValueError("graph area exceeds its native path record budget")
            source = VMobject().set_points(path)
            part = VMobject().pointwise_become_partial(source, (first + a) / curves, (last + b) / curves)
            part.set_points(part.get_points()[2 * first:2 * last + 3].copy())
            part.add_line_to(point(self.c2p(right, 0)))
            part.add_line_to(point(self.c2p(left, 0)))
            part.close_path()
            # Close each continuous component independently at the baseline.
            # A native null-curve join must not turn a pole into a filled bridge.
            result.add_subpath(part.get_points())
        result.match_style(graph, recurse=False)
        result.set_stroke(width=0)
        result.set_fill(color, opacity)
        return result

    _bind_method(Coordinates, "get_area_under_graph", get_area_under_graph)
