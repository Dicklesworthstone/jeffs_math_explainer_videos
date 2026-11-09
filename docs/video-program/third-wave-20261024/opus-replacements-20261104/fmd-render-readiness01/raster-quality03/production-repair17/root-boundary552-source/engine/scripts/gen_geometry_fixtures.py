#!/usr/bin/env python3
"""Generate fmn-library geometry parity fixtures (fm-oab, §12.1, §16.4).

The Reference's geometry constructors (`manimlib/mobject/geometry.py`,
3b1b/manim @ 6199a00d) are pure point arithmetic over `quadratic_bezier_
points_for_arc` and `set_points_as_corners` — but importing them means
importing VMobject, which means moderngl, a window, and a config file. So,
as `gen_positional_fixtures.py` does for the positional API, this script
reproduces the constructor formulas in pure NumPy, transcribed straight
from geometry.py, and records the point arrays they produce.

WHERE OUR RULE DIFFERS ON PURPOSE the fixture records OUR value and says
so: `density` cases carry the component count both engines would choose,
and the Rust test asserts the Reference's number only where the two rules
agree (BN-09 documents the rest).

Output (committed): crates/fmn-library/fixtures/geometry.txt
"""

import math
import os

import numpy as np

OUT_PATH = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "..",
    "crates",
    "fmn-library",
    "fixtures",
    "geometry.txt",
)

HEADER = [
    "# fmn-library geometry parity fixture",
    "# formulas transcribed from 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13",
    "# by scripts/gen_geometry_fixtures.py — regenerate, never hand-edit",
]

TAU = 2 * math.pi
PI = math.pi
DEG = TAU / 360
ORIGIN = np.zeros(3)
RIGHT = np.array([1.0, 0.0, 0.0])
UP = np.array([0.0, 1.0, 0.0])
UR = UP + RIGHT
UL = UP - RIGHT
DL = -UP - RIGHT
DR = -UP + RIGHT

LINES = []


# --- Reference helpers, transcribed ---------------------------------------


def quadratic_bezier_points_for_arc(angle, n_components=8):
    """utils/bezier.py: 2n+1 points tracing the unit arc from 0 to angle."""
    n_points = 2 * n_components + 1
    angles = np.linspace(0, angle, n_points)
    points = np.array([[math.cos(a), math.sin(a), 0.0] for a in angles])
    theta = angle / n_components
    points[1::2] /= math.cos(theta / 2)
    return points


def reference_arc_n_components(angle):
    """geometry.py Arc.__init__: int(15 * |angle| / TAU) + 1."""
    return int(15 * (abs(angle) / TAU)) + 1


def ours_arc_n_components(angle):
    """BN-09: max(1, ceil(16 * |angle| / TAU))."""
    return max(1, math.ceil(16 * abs(angle) / TAU))


def rotation_matrix_z(angle):
    c, s = math.cos(angle), math.sin(angle)
    return np.array([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])


def arc_points(start_angle, angle, radius, arc_center, n_components):
    """Arc.__init__: set_points(arc) -> rotate -> scale -> shift."""
    points = quadratic_bezier_points_for_arc(angle, n_components)
    points = points @ rotation_matrix_z(start_angle).T
    points = points * radius
    return points + np.asarray(arc_center, dtype=float)


def set_points_as_corners(corners):
    """VMobject.set_points_as_corners: anchors with midpoint handles."""
    corners = np.asarray(corners, dtype=float)
    if len(corners) < 2:
        return corners.copy()
    out = np.zeros((2 * (len(corners) - 1) + 1, 3))
    out[0::2] = corners
    out[1::2] = 0.5 * (corners[:-1] + corners[1:])
    return out


def compass_directions(n, start_vect):
    angle = TAU / n
    return np.array(
        [start_vect @ rotation_matrix_z(k * angle).T for k in range(n)]
    )


def stretch_to(points, dim, length):
    """Mobject.rescale_to_fit(stretch=True) about the box centre."""
    points = points.copy()
    lo, hi = points[:, dim].min(), points[:, dim].max()
    old = hi - lo
    if old == 0:
        return points
    center = 0.5 * (lo + hi)
    points[:, dim] = center + (points[:, dim] - center) * (length / old)
    return points


# --- emission -------------------------------------------------------------


def rows(arr):
    arr = np.asarray(arr, dtype=np.float64)
    return ["\t".join(repr(float(x)) for x in row) for row in arr]


def emit(name, kind, params, points, note=""):
    LINES.append(f"case {name}")
    LINES.append(f"kind {kind}")
    for key, value in params.items():
        if isinstance(value, str):
            LINES.append(f"p {key} {value}")
        elif np.isscalar(value):
            LINES.append(f"p {key} {repr(float(value))}")
        else:
            joined = "\t".join(repr(float(x)) for x in np.asarray(value).ravel())
            LINES.append(f"p {key} {joined}")
    if note:
        LINES.append(f"note {note}")
    LINES.append(f"points {len(points)}")
    LINES.extend(rows(points))
    LINES.append("end")


def gen_arcs():
    # Angles where the Reference's density rule and ours agree, so the
    # whole point array is comparable.
    cases = [
        ("quarter", 0.0, TAU / 4, 1.0, ORIGIN),
        ("half", 0.0, PI, 1.0, ORIGIN),
        ("full", 0.0, TAU, 1.0, ORIGIN),
        ("offset", 0.3, 1.7, 2.5, np.array([1.0, -2.0, 0.0])),
        ("negative", TAU / 3, -TAU / 4, 1.5, ORIGIN),
        ("tiny", 0.0, 0.05, 1.0, ORIGIN),
    ]
    for name, start, angle, radius, center in cases:
        ours = ours_arc_n_components(angle)
        theirs = reference_arc_n_components(angle)
        pts = arc_points(start, angle, radius, center, ours)
        emit(
            f"arc_{name}",
            "arc",
            {
                "start_angle": start,
                "angle": angle,
                "radius": radius,
                "center": center,
                "n_components": float(ours),
                "reference_n_components": float(theirs),
            },
            pts,
        )

    # Circle: the Reference's Circle is Arc(angle=TAU) with 16 components,
    # which is exactly our rule's answer.
    emit(
        "circle_r3",
        "circle",
        {"radius": 3.0, "center": ORIGIN},
        arc_points(0.0, TAU, 3.0, ORIGIN, 16),
    )
    emit(
        "circle_offset",
        "circle",
        {"radius": 1.25, "center": np.array([-1.0, 0.5, 0.0])},
        arc_points(0.0, TAU, 1.25, np.array([-1.0, 0.5, 0.0]), 16),
    )
    # Dot: Circle of DEFAULT_DOT_RADIUS at a point.
    emit(
        "dot_default",
        "dot",
        {"radius": 0.08, "center": np.array([1.0, 2.0, 0.0])},
        arc_points(0.0, TAU, 0.08, np.array([1.0, 2.0, 0.0]), 16),
    )
    emit(
        "small_dot",
        "dot",
        {"radius": 0.04, "center": ORIGIN},
        arc_points(0.0, TAU, 0.04, ORIGIN, 16),
    )
    # Ellipse: Circle stretched in both axes about its centre.
    pts = arc_points(0.0, TAU, 1.0, ORIGIN, 16)
    pts = stretch_to(stretch_to(pts, 0, 4.0), 1, 1.0)
    emit("ellipse_4x1", "ellipse", {"width": 4.0, "height": 1.0}, pts)


def gen_polys():
    triangle = [[-3.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]]
    emit(
        "polygon_triangle",
        "polygon",
        {"vertices": np.array(triangle)},
        set_points_as_corners([*triangle, triangle[0]]),
    )
    quad = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 2.0, 0.0], [0.0, 2.0, 0.0]]
    emit(
        "polygon_quad",
        "polygon",
        {"vertices": np.array(quad)},
        set_points_as_corners([*quad, quad[0]]),
    )
    emit(
        "polyline_open",
        "polyline",
        {"vertices": np.array(quad[:3])},
        set_points_as_corners(quad[:3]),
    )

    for n in (3, 4, 5, 6, 8):
        start_angle = (n % 2) * 90 * DEG
        start_vect = RIGHT @ rotation_matrix_z(start_angle).T
        verts = compass_directions(n, start_vect)
        emit(
            f"regular_polygon_{n}",
            "regular_polygon",
            {"n": float(n), "radius": 1.0},
            set_points_as_corners([*verts, verts[0]]),
        )
    # Explicit radius and start angle.
    start_vect = (2.0 * RIGHT) @ rotation_matrix_z(0.4).T
    verts = compass_directions(5, start_vect)
    emit(
        "regular_polygon_5_turned",
        "regular_polygon",
        {"n": 5.0, "radius": 2.0, "start_angle": 0.4},
        set_points_as_corners([*verts, verts[0]]),
    )

    for name, w, h in (("rect_4x2", 4.0, 2.0), ("rect_1x5", 1.0, 5.0)):
        pts = set_points_as_corners([UR, UL, DL, DR, UR])
        pts = stretch_to(stretch_to(pts, 0, w), 1, h)
        emit(name, "rectangle", {"width": w, "height": h}, pts)
    pts = set_points_as_corners([UR, UL, DL, DR, UR])
    pts = stretch_to(stretch_to(pts, 0, 2.0), 1, 2.0)
    emit("square_2", "square", {"side_length": 2.0}, pts)


def gen_lines():
    for name, start, end in (
        ("line_lr", [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        ("line_diag", [0.0, 0.0, 0.0], [3.0, 4.0, 0.0]),
        ("line_3d", [1.0, 1.0, 1.0], [-2.0, 0.5, 2.0]),
    ):
        emit(
            name,
            "line",
            {"start": np.array(start), "end": np.array(end), "buff": 0.0},
            set_points_as_corners([start, end]),
        )

    # A buffered straight line: the Reference trims by alpha = buff/length
    # in curve-index space, which for one straight curve is the same cut
    # ours makes by true length — so this case is comparable.
    start, end = np.array([0.0, 0.0, 0.0]), np.array([10.0, 0.0, 0.0])
    buff = 2.0
    alpha = min(buff / 10.0, 0.5)
    a = start + alpha * (end - start)
    b = start + (1 - alpha) * (end - start)
    emit(
        "line_buffed",
        "line",
        {"start": start, "end": end, "buff": buff},
        set_points_as_corners([a, b]),
    )

    # Elbow: corners UP, UR, RIGHT, sized and rotated about the ORIGIN.
    for name, width, angle in (("elbow", 0.2, 0.0), ("elbow_turned", 0.5, TAU / 16)):
        pts = set_points_as_corners([UP, UR, RIGHT])
        lo, hi = pts[:, 0].min(), pts[:, 0].max()
        pts = pts * (width / (hi - lo))
        pts = pts @ rotation_matrix_z(angle).T
        emit(name, "elbow", {"width": width, "angle": angle}, pts)


def gen_cubic():
    a0 = np.array([0.0, 0.0, 0.0])
    h0 = np.array([1.0, 2.0, 0.0])
    h1 = np.array([3.0, 2.0, 0.0])
    a1 = np.array([4.0, 0.0, 0.0])
    # The Reference's two-quad split (utils/bezier.py), which our
    # QuadPath::add_cubic_bezier_curve_to also uses until fm-6cf lands.
    t = 0.5
    mid = (
        a0 * (1 - t) ** 3
        + 3 * h0 * t * (1 - t) ** 2
        + 3 * h1 * t**2 * (1 - t)
        + a1 * t**3
    )
    emit(
        "cubic_bezier",
        "cubic",
        {"a0": a0, "h0": h0, "h1": h1, "a1": a1, "midpoint": mid},
        np.array([a0, a1]),  # endpoints only: the split is fm-6cf's to pin
        note="endpoints_only",
    )


def main():
    gen_arcs()
    gen_polys()
    gen_lines()
    gen_cubic()
    out = os.path.abspath(OUT_PATH)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w") as f:
        for line in HEADER:
            f.write(line + "\n")
        for line in LINES:
            f.write(line + "\n")
    n = sum(1 for line in LINES if line.startswith("case "))
    print(f"wrote {out} ({n} cases)")


if __name__ == "__main__":
    main()
