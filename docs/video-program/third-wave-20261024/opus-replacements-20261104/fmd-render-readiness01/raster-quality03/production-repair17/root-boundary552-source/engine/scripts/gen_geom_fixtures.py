#!/usr/bin/env python3
"""Generate fmn-geom parity fixtures from the pinned Reference checkout.

Reference: 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13, expected at
scripts/manim_ref (gitignored). Unlike gen_reference_fixtures.py (which
reproduces fmn-core's arithmetic in pure Python), this script imports the
Reference's own modules — utils/bezier.py, utils/space_ops.py, and the real
VMobject — and records what they actually produce.

Environment: needs numpy, scipy, fontTools, and VMobject's import closure
(colour, screeninfo, moderngl, PyOpenGL, pillow, appdirs, matplotlib, yaml,
mapbox_earcut, tqdm, rich, validators, addict). A uv venv works:
  uv venv .venv && uv pip install numpy scipy fonttools pyyaml colour \
    screeninfo moderngl PyOpenGL pillow appdirs matplotlib mapbox_earcut \
    tqdm rich validators addict

Outputs (committed under crates/fmn-geom/fixtures/):
  smooth_cubic_handles.txt   anchors -> h1/h2 from the scipy spline solves (f64)
  approx_smooth_handles.txt  anchors -> local handle rule output (f64)
  quad_approx_cubic.txt      cubic control points -> two-quad split (f64)
  arc_points.txt             (angle, n_components) -> arc point runs (f64)
  quadpath_cases.txt         named VMobject op sequences -> points, subpath
                             ends, closure flag, joint angles (f32 storage;
                             the Rust tests mirror the op sequences by name
                             and compare with loose f32 tolerances)

Note: no fixture covers change_anchor_mode("true_smooth") end-to-end — that
routes through fontTools cu2qu, which is fm-6cf's error-bounded converter.
The spline *solve* underneath it is covered exactly here.
"""

import os
import sys
import types

HERE = os.path.dirname(os.path.abspath(__file__))
REF = os.path.join(HERE, "manim_ref")
OUT_DIR = os.path.join(HERE, "..", "crates", "fmn-geom", "fixtures")

# Import Reference submodules without executing manimlib/__init__.py
# (which drags in the GL window stack).
sys.path.insert(0, REF)
pkg = types.ModuleType("manimlib")
pkg.__path__ = [os.path.join(REF, "manimlib")]
pkg.__file__ = os.path.join(REF, "manimlib", "__init__.py")
sys.modules["manimlib"] = pkg

import numpy as np  # noqa: E402

import manimlib.utils.bezier as bz  # noqa: E402
from manimlib.mobject.types.vectorized_mobject import VMobject  # noqa: E402

HEADER = [
    "# fmn-geom parity fixture",
    "# generated from 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13",
    "# by scripts/gen_geom_fixtures.py — regenerate, never hand-edit",
]

TAU = 2 * np.pi


def rows(arr):
    arr = np.asarray(arr, dtype=np.float64)
    if arr.ndim == 1:
        arr = arr.reshape(1, -1)
    return ["\t".join(repr(float(x)) for x in row) for row in arr]


def write(path, lines):
    with open(path, "w") as f:
        for line in HEADER:
            f.write(line + "\n")
        for line in lines:
            f.write(line + "\n")
    print(f"wrote {path}")


# ------------------------------------------------------- pure-f64 fixtures

ANCHOR_SETS = {
    "open3": [[0, 0, 0], [1, 1, 0], [2, 0, 0]],
    "open5": [[0, 0, 0], [1, 2, 0], [3, 3, 0], [4, 1, 0], [6, 0, 0]],
    "open2": [[0, 0, 0], [2, 1, 0]],
    "collinear4": [[0, 0, 0], [1, 0, 0], [2, 0, 0], [3, 0, 0]],
    "closed5": [[1, 0, 0], [0, 1, 0], [-1, 0, 0], [0, -1, 0], [1, 0, 0]],
    "spatial4": [[0, 0, 0], [1, 1, 1], [2, 0, 2], [3, 1, 0]],
}


def gen_smooth_cubic_handles():
    lines = []
    for name, anchors in ANCHOR_SETS.items():
        anchors = np.array(anchors, dtype=np.float64)
        h1, h2 = bz.get_smooth_cubic_bezier_handle_points(anchors)
        lines.append(f"case {name}")
        lines.append(f"anchors {len(anchors)}")
        lines.extend(rows(anchors))
        lines.append(f"h1 {len(h1)}")
        lines.extend(rows(h1))
        lines.append(f"h2 {len(h2)}")
        lines.extend(rows(h2))
        lines.append("end")
    write(os.path.join(OUT_DIR, "smooth_cubic_handles.txt"), lines)


def gen_approx_smooth_handles():
    lines = []
    for name in ("open3", "open5", "closed5", "open2"):
        anchors = np.array(ANCHOR_SETS[name], dtype=np.float64)
        handles = bz.approx_smooth_quadratic_bezier_handles(anchors)
        handles = np.atleast_2d(handles)
        lines.append(f"case {name}")
        lines.append(f"anchors {len(anchors)}")
        lines.extend(rows(anchors))
        lines.append(f"handles {len(handles)}")
        lines.extend(rows(handles))
        lines.append("end")
    write(os.path.join(OUT_DIR, "approx_smooth_handles.txt"), lines)


CUBICS = {
    "s_curve_inflection": [[0, 0, 0], [1, 2, 0], [2, -2, 0], [3, 0, 0]],
    "c_curve": [[0, 0, 0], [0, 1, 0], [1, 2, 0], [2, 2, 0]],
    "collinear": [[0, 0, 0], [1, 0, 0], [2, 0, 0], [3, 0, 0]],
    "cusp_like": [[0, 0, 0], [2, 2, 0], [-1, 2, 0], [1, 0, 0]],
}


def gen_quad_approx_cubic():
    lines = []
    for name, pts in CUBICS.items():
        a0, h0, h1, a1 = (np.array([p], dtype=np.float64) for p in pts)
        out = bz.get_quadratic_approximation_of_cubic(a0, h0, h1, a1)
        lines.append(f"case {name}")
        lines.append(f"cubic 4")
        lines.extend(rows(np.vstack([a0, h0, h1, a1])))
        lines.append(f"quads {len(out)}")
        lines.extend(rows(out))
        lines.append("end")
    write(os.path.join(OUT_DIR, "quad_approx_cubic.txt"), lines)


ARCS = [
    ("quarter_n4", TAU / 4, 4),
    ("quarter_n2", TAU / 4, 2),
    ("full_n16", TAU, 16),
    ("neg_half_n8", -TAU / 2, 8),
    ("one_rad_n3", 1.0, 3),
]


def gen_arc_points():
    lines = []
    for name, angle, n in ARCS:
        pts = bz.quadratic_bezier_points_for_arc(angle, n)
        lines.append(f"case {name}")
        lines.append(f"angle {angle!r}")
        lines.append(f"n_components {n}")
        lines.append(f"points {len(pts)}")
        lines.extend(rows(pts))
        lines.append("end")
    write(os.path.join(OUT_DIR, "arc_points.txt"), lines)


# ---------------------------------------------------- VMobject op sequences
# Each op sequence here is mirrored by name in
# crates/fmn-geom/tests/reference_parity.rs; keep the two lists in sync.

P = lambda x, y, z=0.0: np.array([float(x), float(y), float(z)])


def case_line_quad_multi(vm):
    vm.start_new_path(P(0, 0))
    vm.add_line_to(P(1, 0))
    vm.add_quadratic_bezier_curve_to(P(1.5, 1), P(2, 0))
    vm.start_new_path(P(3, 0))
    vm.add_line_to(P(4, 1))


def case_corners_closed_square(vm):
    vm.set_points_as_corners([P(0, 0), P(2, 0), P(2, 2), P(0, 2), P(0, 0)])


def case_cubic_curve(vm):
    vm.start_new_path(P(0, 0))
    vm.add_cubic_bezier_curve_to(P(0, 1), P(1, 2), P(2, 2))


def case_cubic_simple_approx(vm):
    vm.use_simple_quadratic_approx = True
    vm.start_new_path(P(0, 0))
    vm.add_cubic_bezier_curve_to(P(1, 0.25), P(2, 0.5), P(3, 1))


def case_smooth_chain(vm):
    vm.start_new_path(P(0, 0))
    vm.add_line_to(P(1, 1))
    vm.add_smooth_curve_to(P(2, 0))
    vm.add_smooth_curve_to(P(3, 1))


def case_smooth_cubic_chain(vm):
    vm.start_new_path(P(0, 0))
    vm.add_smooth_cubic_curve_to(P(0.5, 1), P(1, 1))
    vm.add_smooth_cubic_curve_to(P(2, 0), P(2.5, 0.5))


def case_arc_to_quarter(vm):
    vm.start_new_path(P(1, 0))
    vm.add_arc_to(P(0, 1), TAU / 4, n_components=4)


def case_close_path_line(vm):
    vm.set_points_as_corners([P(0, 0), P(2, 0), P(1, 2)])
    vm.close_path()


def case_close_path_smooth(vm):
    vm.set_points_as_corners([P(0, 0), P(2, 0), P(1, 2)])
    vm.close_path(smooth=True)


def case_start_new_path_singleton(vm):
    vm.start_new_path(P(0, 0))
    vm.start_new_path(P(1, 1))


def case_single_point(vm):
    vm.start_new_path(P(1, 2))


def case_reverse_multi(vm):
    case_line_quad_multi(vm)
    vm.reverse_points()


def case_make_jagged_arc(vm):
    vm.set_points(bz.quadratic_bezier_points_for_arc(TAU / 4, 4))
    vm.change_anchor_mode("jagged")


def case_approx_smooth_zigzag(vm):
    vm.set_points_as_corners([P(0, 0), P(1, 1), P(2, 0), P(3, 1), P(4, 0)])
    vm.make_approximately_smooth()


def case_subdivide_sharp_arc(vm):
    vm.set_points(bz.quadratic_bezier_points_for_arc(TAU / 4, 1))
    vm.subdivide_sharp_curves()


def case_insert_curves(vm):
    vm.set_points_as_corners([P(0, 0), P(1, 0), P(4, 0)])
    vm.insert_n_curves(3)


def case_null_line(vm):
    vm.start_new_path(P(0, 0))
    vm.add_line_to(P(0, 0))
    vm.add_line_to(P(1, 0))


def case_two_subpaths_last_closed(vm):
    vm.set_points_as_corners([P(5, 5), P(6, 5)])
    vm.add_subpath(
        VMobject()
        .set_points_as_corners([P(0, 0), P(2, 0), P(2, 2), P(0, 0)])
        .get_points()
    )


CASES = [
    case_line_quad_multi,
    case_corners_closed_square,
    case_cubic_curve,
    case_cubic_simple_approx,
    case_smooth_chain,
    case_smooth_cubic_chain,
    case_arc_to_quarter,
    case_close_path_line,
    case_close_path_smooth,
    case_start_new_path_singleton,
    case_single_point,
    case_reverse_multi,
    case_make_jagged_arc,
    case_approx_smooth_zigzag,
    case_subdivide_sharp_arc,
    case_insert_curves,
    case_null_line,
    case_two_subpaths_last_closed,
]


def gen_quadpath_cases():
    lines = []
    for fn in CASES:
        name = fn.__name__.removeprefix("case_")
        vm = VMobject()
        fn(vm)
        points = vm.get_points()
        ends = vm.get_subpath_end_indices()
        angles = vm.get_joint_angles()
        lines.append(f"case {name}")
        lines.append(f"points {len(points)}")
        lines.extend(rows(points))
        lines.append("ends " + "\t".join(str(int(e)) for e in ends))
        lines.append(f"closed {int(vm.is_closed())}")
        lines.append("joint_angles " + "\t".join(repr(float(a)) for a in angles))
        lines.append("end")
    write(os.path.join(OUT_DIR, "quadpath_cases.txt"), lines)


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    gen_smooth_cubic_handles()
    gen_approx_smooth_handles()
    gen_quad_approx_cubic()
    gen_arc_points()
    gen_quadpath_cases()


if __name__ == "__main__":
    main()
