"""Time-windowed traces using the existing native smooth-path constructor.

Only temporal sample bookkeeping lives here. Actual frame times arrive from
Mobject's updater; curve construction and stroke interpolation remain Chisel
and Marionette operations. Unobserved motion is not integrated or guessed:
anchor points linearly interpolate consecutive observed source positions.
"""
from __future__ import annotations

from bisect import bisect_right
import math
from typing import Any

# A malformed spacing must fail before an enormous Python allocation. This
# is a resource ceiling, not adaptive sampling or silent history truncation.
_MAX_ANCHORS = 100_000


def _parameters(window, spacing):
    window = math.inf if window is None else float(window)
    spacing = float(spacing)
    if math.isnan(window) or window < 0:
        raise ValueError("TracedPath time_traced must be nonnegative or infinite")
    if not math.isfinite(spacing) or spacing <= 0:
        raise ValueError("TracedPath time_per_anchor must be finite and positive")
    if math.isfinite(window) and window / spacing > _MAX_ANCHORS - 3:
        raise ValueError("TracedPath time window exceeds the anchor budget; increase time_per_anchor")
    return window, spacing


def _point(np, value):
    point = np.asarray(value, dtype=float)
    if point.shape != (3,) or not np.isfinite(point).all():
        raise ValueError("TracedPath source must return a finite 3D point")
    return tuple(float(component) for component in point)


def _lerp(start, end, alpha):
    # A convex blend avoids overflowing end-start for opposite large inputs.
    alpha = min(1.0, max(0.0, alpha))
    return tuple((1.0 - alpha) * a + alpha * b for a, b in zip(start, end))


def _visible(anchors, now, point, window):
    rows = list(anchors)
    if not rows or rows[-1][0] != now:
        rows.append((now, point))
    else:
        rows[-1] = (now, point)
    cutoff = now - window
    index = bisect_right([time for time, _ in rows], cutoff)
    if index == 0:
        return rows
    if index == len(rows):
        return [(now, point)]
    before, after = rows[index - 1], rows[index]
    boundary = _lerp(before[1], after[1], (cutoff - before[0]) / (after[0] - before[0]))
    return [(cutoff, boundary), *rows[index:]]


def install_traced_path(native: Any) -> None:
    """Keep existing class identities while fixing temporal trace sampling."""
    g = vars(native)
    if g.get("_FMN_TRACED_PATH_INSTALLED", False):
        return
    Trace = g.get("TracedPath")
    if Trace is None:
        return
    np = g["_np"]

    def initialize(self, traced_point_func, time_traced=None, time_per_anchor=1.0 / 15,
                   stroke_color=None, stroke_width=2.0, stroke_opacity=1.0, **kwargs):
        if not callable(traced_point_func):
            raise TypeError(
                "TracedPath requires a callable returning the traced "
                "point; got " + type(traced_point_func).__name__
            )
        window, spacing = _parameters(time_traced, time_per_anchor)
        super(Trace, self).__init__(**kwargs)
        self.traced_point_func = traced_point_func
        self.time_traced, self.time_per_anchor = window, spacing
        self.stroke_config = dict(color=g["_WHITE"] if stroke_color is None else stroke_color,
                                  width=stroke_width, opacity=stroke_opacity)
        self.time = 0.0
        self.traced_points = []
        self._trace_anchors = ()
        self._trace_previous = None
        self._trace_origin, self._trace_next_index, self._trace_spacing = 0.0, 1, spacing
        # Use the supplied current object, not a captured self, so copied
        # traces evolve their own immutable history through the normal updater.
        self.add_updater(lambda current, dt: current.update_path(dt))

    def publish(self, rows):
        points = np.asarray([point for _, point in rows], dtype=float)
        self.set_points_smoothly(points)
        self.set_stroke(**self.stroke_config)
        # Never share writable caller arrays with the history or this mirror.
        return [point.copy() for point in points]

    def update_path(self, dt):
        delta = float(dt)
        if not math.isfinite(delta) or delta < 0:
            raise ValueError("TracedPath dt must be finite and nonnegative")
        if delta == 0:
            return self
        window, spacing = _parameters(self.time_traced, self.time_per_anchor)
        if self._trace_previous is not None and self.time != self._trace_previous[0]:
            raise ValueError("TracedPath time was changed outside its updater; construct a new trace to reset it")
        now = float(self.time) + delta
        if not math.isfinite(now) or now <= self.time:
            raise ValueError("TracedPath elapsed time cannot advance at this precision")
        # Sample once at the actual updater boundary, never at invented
        # intermediate times (source callbacks may have observable effects).
        point = _point(np, self.traced_point_func())
        previous = self._trace_previous
        if previous is None:
            anchors, origin, next_index = ((now, point),), now, 1
        else:
            origin, next_index = self._trace_origin, self._trace_next_index
            if spacing != self._trace_spacing:
                origin, next_index = previous[0], 1
            ratio = (now - origin) / spacing
            if not math.isfinite(ratio):
                raise ValueError("TracedPath anchor grid exceeds representable time")
            last_index = math.floor(math.nextafter(ratio, math.inf))
            cutoff = now - window
            first_index = next_index
            if math.isfinite(window):
                # Skip grid points already outside the trailing window, but
                # retain a predecessor to interpolate its moving left edge.
                first_index = max(first_index, math.ceil((cutoff - origin) / spacing) - 1)
            retained = list(self._trace_anchors)
            keep = max(0, bisect_right([time for time, _ in retained], cutoff) - 1)
            retained = retained[keep:]
            count = max(0, last_index - first_index + 1)
            if count + len(retained) + 1 > _MAX_ANCHORS:
                raise ValueError("TracedPath exceeds the anchor budget; use a finite window or larger spacing")
            for index in range(first_index, last_index + 1):
                time = min(now, origin + index * spacing)
                if retained and time <= retained[-1][0]:
                    continue
                alpha = (time - previous[0]) / (now - previous[0])
                retained.append((time, _lerp(previous[1], point, alpha)))
            anchors, next_index = tuple(retained), last_index + 1
        if window == 0:
            anchors, origin, next_index = ((now, point),), now, 1
        rows = _visible(anchors, now, point, window)
        points = publish(self, rows)
        # Temporal state commits only after native curve/style operations
        # succeed. Arbitrary authored geometry hooks are not transactional.
        self._trace_anchors, self._trace_previous = anchors, (now, point)
        self._trace_origin, self._trace_next_index, self._trace_spacing = origin, next_index, spacing
        self.traced_points, self.time = points, now
        return self

    def method(cls, name, function):
        function.__name__ = name
        function.__qualname__ = cls.__qualname__ + "." + name
        function.__module__ = cls.__module__
        setattr(cls, name, function)

    method(Trace, "__init__", initialize)
    method(Trace, "update_path", update_path)
    Tail = g.get("TracingTail")
    if Tail is not None:
        original_tail_init = getattr(Tail, "__init__", None)

        def tail_init(self, mobject_or_func, time_traced=1.0, stroke_color=None,
                      stroke_width=(0, 3), stroke_opacity=(0, 1), time_per_anchor=1.0 / 15, **kwargs):
            if not isinstance(mobject_or_func, g["Mobject"]) and not callable(mobject_or_func):
                raise TypeError(
                    "TracingTail traces a Mobject or a point-returning "
                    "callable; got " + type(mobject_or_func).__name__
                )
            anchor_dt = float(time_per_anchor)
            if not math.isfinite(anchor_dt) or anchor_dt <= 0:
                raise ValueError("TracingTail time_per_anchor must be finite and positive")
            # Validate before selecting the native route too: a bound source
            # must not bypass the finite history/allocation contract.
            window, spacing = _parameters(time_traced, anchor_dt)
            if not math.isfinite(window):
                raise ValueError("TracingTail requires a finite time_traced")
            if (
                isinstance(mobject_or_func, g["Mobject"])
                and hasattr(self, "_init_native_tracer")
                and mobject_or_func._is_bound()
                and original_tail_init is not None
            ):
                return original_tail_init(
                    self,
                    mobject_or_func,
                    time_traced=window,
                    stroke_color=stroke_color,
                    stroke_width=stroke_width,
                    stroke_opacity=stroke_opacity,
                    time_per_anchor=anchor_dt,
                    **kwargs,
                )
            # A detached source has no Stage handle for the optimized native
            # tracer. Normalize it to the already-supported live point callback;
            # do not adopt it early or capture a frozen construction-time center.
            source = mobject_or_func.get_center if isinstance(mobject_or_func, g["Mobject"]) else mobject_or_func
            super(Tail, self).__init__(source, time_traced=window, time_per_anchor=anchor_dt, stroke_color=stroke_color,
                                      stroke_width=stroke_width, stroke_opacity=stroke_opacity, **kwargs)
            point = _point(np, self.traced_point_func())
            count = math.ceil(window / spacing)
            rows = tuple((-index * spacing, point) for index in range(count, -1, -1))
            self._trace_anchors, self._trace_previous = rows, (0.0, point)
            # Reference changing.py seeds sample history, not path geometry.
            # Keep the native points and style untouched until a positive-dt
            # update publishes the first trace. The timestamped anchor grid
            # still retains the predecessor needed by the moving time window.
            self.traced_points = [np.array(point) for _ in range(int(window / spacing))]
        if not issubclass(Tail, Trace):
            Tail.__bases__ = (Trace,)
        method(Tail, "__init__", tail_init)
    g["_FMN_TRACED_PATH_INSTALLED"] = True
