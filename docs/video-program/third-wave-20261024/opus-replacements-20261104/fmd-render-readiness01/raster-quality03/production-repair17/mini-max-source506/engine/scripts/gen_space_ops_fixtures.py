#!/usr/bin/env python3
"""Generate fmn-geom space_ops parity fixtures (fm-ngx, §7.5, §16.4).

Reference: 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13, expected at
scripts/manim_ref (gitignored). This script imports the Reference's *own*
`manimlib/utils/space_ops.py` — stubbing only the two module-level imports
that pull in native/UI packages (`mapbox_earcut`, used solely by
`earclip_triangulation`, which is bead fm-81u, and `tqdm`, its progress
display) — so every recorded value is what the Reference actually computes,
including everything routed through scipy's `Rotation`.

The rotation conventions (quaternion sign and element order, composition
order, `as_euler` branch/range choices, gimbal-lock degeneracy handling) are
scipy's, and this program fixes them as documented FrankenManim semantics
(§7.5, §2.2) — so they are recorded from scipy here and locked by
crates/fmn-geom/tests/space_ops_parity.rs.

Environment: numpy + scipy (the Reference's own pins are not needed for these
pure-math paths).

Output (committed): crates/fmn-geom/fixtures/space_ops.txt
"""

import os
import sys
import types

HERE = os.path.dirname(os.path.abspath(__file__))
REF = os.path.join(HERE, "manim_ref")
OUT_PATH = os.path.join(
    HERE, "..", "crates", "fmn-geom", "fixtures", "space_ops.txt"
)

# Stub the two imports space_ops.py makes that we neither have nor need: the
# earcut binding (fm-81u owns triangulation) and tqdm's progress display,
# both used only inside earclip_triangulation.
earcut_stub = types.ModuleType("mapbox_earcut")
earcut_stub.triangulate_float32 = None
sys.modules["mapbox_earcut"] = earcut_stub
tqdm_stub = types.ModuleType("tqdm")
tqdm_auto = types.ModuleType("tqdm.auto")
tqdm_auto.tqdm = None
tqdm_stub.auto = tqdm_auto
sys.modules["tqdm"] = tqdm_stub
sys.modules["tqdm.auto"] = tqdm_auto
# manimlib.utils.iterables imports colour.Color for a type annotation only.
colour_stub = types.ModuleType("colour")


class _Color:  # noqa: D101 - annotation placeholder
    pass


colour_stub.Color = _Color
sys.modules["colour"] = colour_stub

# Import Reference submodules without executing manimlib/__init__.py.
sys.path.insert(0, REF)
pkg = types.ModuleType("manimlib")
pkg.__path__ = [os.path.join(REF, "manimlib")]
pkg.__file__ = os.path.join(REF, "manimlib", "__init__.py")
sys.modules["manimlib"] = pkg

import numpy as np  # noqa: E402

# manimlib.constants pulls the whole config/colour stack in through
# manimlib.config; space_ops needs only these six names, whose values are
# verbatim from constants.py (and mirrored, fixture-locked, in fmn-core).
constants_stub = types.ModuleType("manimlib.constants")
constants_stub.DOWN = np.array([0.0, -1.0, 0.0])
constants_stub.OUT = np.array([0.0, 0.0, 1.0])
constants_stub.RIGHT = np.array([1.0, 0.0, 0.0])
constants_stub.UP = np.array([0.0, 1.0, 0.0])
constants_stub.PI = np.pi
constants_stub.TAU = 2 * np.pi
sys.modules["manimlib.constants"] = constants_stub
from scipy.spatial.transform import Rotation  # noqa: E402

import manimlib.utils.space_ops as so  # noqa: E402

HEADER = [
    "# fmn-geom space_ops parity fixture",
    "# generated from 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13",
    "# (and, for the rotation conventions, scipy's Rotation, which the",
    "#  Reference delegates to and which §7.5 fixes as our semantics)",
    "# by scripts/gen_space_ops_fixtures.py — regenerate, never hand-edit",
]

PI = np.pi
TAU = 2 * PI

LINES = []


def row(vals):
    return "\t".join(repr(float(x)) for x in np.asarray(vals, dtype=np.float64).ravel())


def emit(name, op, fields):
    """One fixture case: `case <name>`, `op <op>`, then `key value…` lines."""
    LINES.append(f"case {name}")
    LINES.append(f"op {op}")
    for key, value in fields:
        if isinstance(value, str):
            LINES.append(f"{key} {value}")
        elif np.isscalar(value):
            LINES.append(f"{key} {repr(float(value))}")
        else:
            arr = np.asarray(value, dtype=np.float64)
            if arr.ndim == 1:
                LINES.append(f"{key} {row(arr)}")
            else:
                # A 2-D block is announced as `key rows <n>`, so the reader
                # never has to guess whether a lone number is a count.
                LINES.append(f"{key} rows {len(arr)}")
                for r in arr:
                    LINES.append(row(r))
    LINES.append("end")


# --------------------------------------------------------------- inventories

# Axes chosen to cover: cardinal axes, a general axis, a non-unit axis (the
# Reference normalizes internally), and near-degenerate magnitudes.
AXES = {
    "z": [0.0, 0.0, 1.0],
    "x": [1.0, 0.0, 0.0],
    "y": [0.0, 1.0, 0.0],
    "diag": [1.0, 1.0, 1.0],
    "long": [0.0, 0.0, 7.5],
    "skew": [0.3, -1.7, 2.2],
}

ANGLES = {
    "zero": 0.0,
    "eps": 1e-12,
    "third": PI / 3,
    "half": PI / 2,
    "near_pi": PI - 1e-9,
    "pi": PI,
    "past_pi": PI + 0.4,
    "neg_half": -PI / 2,
    "big": 3 * TAU + 1.1,
}

VECTORS = {
    "right": [1.0, 0.0, 0.0],
    "up": [0.0, 1.0, 0.0],
    "out": [0.0, 0.0, 1.0],
    "zero": [0.0, 0.0, 0.0],
    "general": [1.0, -2.0, 3.0],
    "tiny": [1e-14, 0.0, 0.0],
    "planar": [-3.0, 4.0, 0.0],
}


def gen_rotation_matrices():
    for aname, angle in ANGLES.items():
        for xname, axis in AXES.items():
            name = f"rotmat_{aname}_{xname}"
            emit(
                name,
                "rotation_matrix",
                [
                    ("angle", angle),
                    ("axis", axis),
                    ("matrix", so.rotation_matrix(angle, np.array(axis, dtype=float))),
                ],
            )
    for aname, angle in ANGLES.items():
        emit(
            f"rotz_{aname}",
            "rotation_about_z",
            [("angle", angle), ("matrix", so.rotation_about_z(angle))],
        )
    for aname, angle in list(ANGLES.items())[:4]:
        emit(
            f"rotmatT_{aname}",
            "rotation_matrix_transpose",
            [
                ("angle", angle),
                ("axis", AXES["skew"]),
                (
                    "matrix",
                    so.rotation_matrix_transpose(
                        angle, np.array(AXES["skew"], dtype=float)
                    ),
                ),
            ],
        )


def gen_rotation_between():
    pairs = {
        # ordinary
        "out_to_x": ("out", "right"),
        "x_to_up": ("right", "up"),
        "general": ("general", "planar"),
        # identical (atol early return)
        "same": ("out", "out"),
        # antiparallel: the cross product degenerates, RIGHT fallback fires
        "anti_z": ("out", [0.0, 0.0, -1.0]),
        # antiparallel along RIGHT: the RIGHT fallback ALSO degenerates,
        # so the UP fallback fires
        "anti_x": ("right", [-1.0, 0.0, 0.0]),
        "zero_src": ("zero", "out"),
    }
    for name, (a, b) in pairs.items():
        v1 = np.array(VECTORS[a] if isinstance(a, str) else a, dtype=float)
        v2 = np.array(VECTORS[b] if isinstance(b, str) else b, dtype=float)
        emit(
            f"rotbetween_{name}",
            "rotation_between_vectors",
            [("v1", v1), ("v2", v2), ("matrix", so.rotation_between_vectors(v1, v2))],
        )
    for name in ("right", "up", "out", "general", "planar"):
        v = np.array(VECTORS[name], dtype=float)
        emit(
            f"z_to_vector_{name}",
            "z_to_vector",
            [("v", v), ("matrix", so.z_to_vector(v))],
        )


def gen_quaternions():
    for aname, angle in ANGLES.items():
        for xname in ("z", "x", "diag", "skew"):
            axis = np.array(AXES[xname], dtype=float)
            q = so.quaternion_from_angle_axis(angle, axis)
            emit(
                f"quat_from_aa_{aname}_{xname}",
                "quaternion_from_angle_axis",
                [("angle", angle), ("axis", axis), ("quat", q)],
            )

    # angle_axis_from_quaternion: undefined at identity (0/0) — the Reference
    # divides by the rotvec norm, so we record only non-degenerate inputs and
    # the Rust side documents the identity refusal.
    for aname in ("third", "half", "near_pi", "past_pi", "neg_half"):
        for xname in ("z", "diag", "skew"):
            angle = ANGLES[aname]
            axis = np.array(AXES[xname], dtype=float)
            q = so.quaternion_from_angle_axis(angle, axis)
            ang, ax = so.angle_axis_from_quaternion(q)
            emit(
                f"aa_from_quat_{aname}_{xname}",
                "angle_axis_from_quaternion",
                [("quat", q), ("angle", ang), ("axis", ax)],
            )

    quats = [
        so.quaternion_from_angle_axis(ANGLES["third"], np.array(AXES["z"], dtype=float)),
        so.quaternion_from_angle_axis(ANGLES["half"], np.array(AXES["x"], dtype=float)),
        so.quaternion_from_angle_axis(
            ANGLES["past_pi"], np.array(AXES["skew"], dtype=float)
        ),
    ]
    emit(
        "quat_mult_pair",
        "quaternion_mult",
        [("quats", np.array(quats[:2])), ("quat", so.quaternion_mult(*quats[:2]))],
    )
    emit(
        "quat_mult_triple",
        "quaternion_mult",
        [("quats", np.array(quats)), ("quat", so.quaternion_mult(*quats))],
    )
    emit(
        "quat_mult_single",
        "quaternion_mult",
        [("quats", np.array(quats[:1])), ("quat", so.quaternion_mult(*quats[:1]))],
    )
    for i, q in enumerate(quats):
        emit(
            f"quat_conj_{i}",
            "quaternion_conjugate",
            [("quat", q), ("out", so.quaternion_conjugate(q))],
        )
        emit(
            f"quat_to_matrix_{i}",
            "rotation_matrix_from_quaternion",
            [("quat", q), ("matrix", so.rotation_matrix_from_quaternion(q))],
        )
        emit(
            f"quat_to_matrixT_{i}",
            "rotation_matrix_transpose_from_quaternion",
            [
                ("quat", q),
                ("matrix", so.rotation_matrix_transpose_from_quaternion(q)),
            ],
        )


# The Euler sequences the Reference actually uses: CameraFrame's default
# "zxz" and the "zxy" alternative it supports (camera_frame.py:30, :162).
# "xyz" (asymmetric, extrinsic) and "ZXZ"/"ZXY" (intrinsic) are recorded too
# so the general algorithm is locked, not just the two hot paths.
EULER_SEQS = ("zxz", "zxy", "xyz", "ZXZ", "ZXY", "XYZ")

EULER_ROTATIONS = {
    "identity": Rotation.identity(),
    "pure_z": Rotation.from_rotvec([0.0, 0.0, 1.0]),
    "pure_x": Rotation.from_rotvec([0.7, 0.0, 0.0]),
    "pure_y": Rotation.from_rotvec([0.0, -1.3, 0.0]),
    "general": Rotation.from_rotvec(0.7 * np.array([1.0, 2.0, 3.0]) / np.sqrt(14)),
    "general2": Rotation.from_rotvec(2.9 * np.array([-1.0, 0.4, 0.2]) / np.linalg.norm([-1.0, 0.4, 0.2])),
    # gimbal lock for the symmetric zxz family: second angle at 0 …
    "gimbal_zero": Rotation.from_euler("zxz", [0.4, 0.0, 0.9]),
    # … and at pi
    "gimbal_pi": Rotation.from_euler("zxz", [0.4, PI, 0.9]),
    # near-lock (inside scipy's 1e-7 window is a lock; just outside is not)
    "near_lock": Rotation.from_euler("zxz", [0.4, 1e-9, 0.9]),
    "off_lock": Rotation.from_euler("zxz", [0.4, 1e-5, 0.9]),
    # gimbal lock for the asymmetric (Tait–Bryan) family: middle angle ±pi/2
    "tb_lock": Rotation.from_euler("zxy", [0.4, PI / 2, 0.9]),
    "tb_lock_neg": Rotation.from_euler("zxy", [0.4, -PI / 2, 0.9]),
    "half_turn": Rotation.from_rotvec([0.0, 0.0, PI]),
}


def gen_euler():
    import warnings

    for rname, rot in EULER_ROTATIONS.items():
        quat = rot.as_quat()
        for seq in EULER_SEQS:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore", UserWarning)
                angles = rot.as_euler(seq)
            emit(
                f"as_euler_{seq}_{rname}",
                "as_euler",
                [("seq", seq), ("quat", quat), ("angles", angles)],
            )

    angle_sets = {
        "small": [0.3, 1.1, -0.4],
        "zeros": [0.0, 0.0, 0.0],
        "locked": [0.4, 0.0, 0.9],
        "locked_pi": [0.4, PI, 0.9],
        "tb_locked": [0.4, PI / 2, 0.9],
        "wide": [3.0, 2.5, -3.1],
        "negative": [-2.2, -0.6, 1.9],
    }
    for aname, angles in angle_sets.items():
        for seq in EULER_SEQS:
            rot = Rotation.from_euler(seq, angles)
            emit(
                f"from_euler_{seq}_{aname}",
                "from_euler",
                [
                    ("seq", seq),
                    ("angles", angles),
                    ("quat", rot.as_quat()),
                    ("matrix", rot.as_matrix()),
                ],
            )


def gen_vector_ops():
    for vname, v in VECTORS.items():
        for aname in ("third", "half", "pi", "neg_half", "big"):
            for xname in ("z", "diag"):
                angle = ANGLES[aname]
                axis = np.array(AXES[xname], dtype=float)
                emit(
                    f"rotate_vector_{vname}_{aname}_{xname}",
                    "rotate_vector",
                    [
                        ("v", v),
                        ("angle", angle),
                        ("axis", axis),
                        ("out", so.rotate_vector(np.array(v, dtype=float), angle, axis)),
                    ],
                )
    for vname in ("right", "up", "general", "planar"):
        v2 = np.array(VECTORS[vname][:2], dtype=float)
        for aname in ("third", "half", "big"):
            angle = ANGLES[aname]
            emit(
                f"rotate_vector_2d_{vname}_{aname}",
                "rotate_vector_2d",
                [("v", v2), ("angle", angle), ("out", so.rotate_vector_2d(v2, angle))],
            )

    for vname, v in VECTORS.items():
        arr = np.array(v, dtype=float)
        emit(
            f"angle_of_vector_{vname}",
            "angle_of_vector",
            [("v", arr), ("angle", so.angle_of_vector(arr))],
        )
        emit(
            f"norm_{vname}",
            "get_norm",
            [("v", arr), ("norm", so.get_norm(arr))],
        )
        emit(
            f"normalize_{vname}",
            "normalize",
            [("v", arr), ("out", so.normalize(arr))],
        )
    emit(
        "normalize_zero_fallback",
        "normalize_fallback",
        [
            ("v", VECTORS["zero"]),
            ("fallback", VECTORS["out"]),
            (
                "out",
                so.normalize(
                    np.array(VECTORS["zero"], dtype=float),
                    np.array(VECTORS["out"], dtype=float),
                ),
            ),
        ],
    )

    pairs = [
        ("right", "up"),
        ("right", "right"),
        ("right", [-1.0, 0.0, 0.0]),
        ("general", "planar"),
        ("zero", "up"),
        ("tiny", "up"),
    ]
    for i, (a, b) in enumerate(pairs):
        v1 = np.array(VECTORS[a] if isinstance(a, str) else a, dtype=float)
        v2 = np.array(VECTORS[b] if isinstance(b, str) else b, dtype=float)
        emit(
            f"angle_between_{i}",
            "angle_between_vectors",
            [("v1", v1), ("v2", v2), ("angle", so.angle_between_vectors(v1, v2))],
        )
        emit(
            f"unit_normal_{i}",
            "get_unit_normal",
            [("v1", v1), ("v2", v2), ("out", so.get_unit_normal(v1, v2))],
        )
        emit(
            f"cross_{i}",
            "cross",
            [("v1", v1), ("v2", v2), ("out", so.cross(v1, v2))],
        )
        emit(
            f"cross2d_{i}",
            "cross2d",
            [("v1", v1), ("v2", v2), ("value", so.cross2d(v1, v2))],
        )
        emit(
            f"dist_{i}",
            "get_dist",
            [("v1", v1), ("v2", v2), ("value", so.get_dist(v1, v2))],
        )
        emit(
            f"midpoint_{i}",
            "midpoint",
            [("v1", v1), ("v2", v2), ("out", so.midpoint(v1, v2))],
        )
    emit(
        "project_along_vector",
        "project_along_vector",
        [
            ("point", VECTORS["general"]),
            ("v", so.normalize(np.array(VECTORS["planar"], dtype=float))),
            (
                "out",
                so.project_along_vector(
                    np.array(VECTORS["general"], dtype=float),
                    so.normalize(np.array(VECTORS["planar"], dtype=float)),
                ),
            ),
        ],
    )
    for vname in ("general", "planar", "zero"):
        arr = np.array(VECTORS[vname], dtype=float)
        emit(
            f"norm_squared_{vname}",
            "norm_squared",
            [("v", arr), ("value", so.norm_squared(arr))],
        )


def gen_geometry_helpers():
    intersections = {
        "axes_2d": ([0, 0, 0], [1, 0, 0], [1, -1, 0], [0, 1, 0], 1e-5),
        "parallel_2d": ([0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 0, 0], 1e-5),
        "skew_3d": ([0, 0, 0], [1, 0, 0], [1, 1, 1], [0, 0, 1], 1e-5),
        "meeting_3d": ([0, 0, 1], [1, 0, 0], [2, 0, 1], [0, 1, 0], 1e-5),
        "below_threshold": ([0, 0, 0], [1, 0, 0], [0, 1, 0], [1e-9, 0, 0], 1e-5),
    }
    for name, (p0, v0, p1, v1, thr) in intersections.items():
        args = [np.array(a, dtype=float) for a in (p0, v0, p1, v1)]
        emit(
            f"find_intersection_{name}",
            "find_intersection",
            [
                ("p0", args[0]),
                ("v0", args[1]),
                ("p1", args[2]),
                ("v1", args[3]),
                ("threshold", thr),
                ("out", so.find_intersection(*args, threshold=thr)),
            ],
        )

    lines = {
        "cross": (([0, 0, 0], [2, 2, 0]), ([0, 2, 0], [2, 0, 0])),
        "shifted": (([-1, 1, 0], [3, 1, 0]), ([1, -4, 0], [1, 5, 0])),
    }
    for name, (l1, l2) in lines.items():
        a = [np.array(p, dtype=float) for p in l1]
        b = [np.array(p, dtype=float) for p in l2]
        emit(
            f"line_intersection_{name}",
            "line_intersection",
            [
                ("l1a", a[0]),
                ("l1b", a[1]),
                ("l2a", b[0]),
                ("l2b", b[1]),
                ("out", so.line_intersection(a, b)),
            ],
        )

    closest = {
        "interior": ([0, 0, 0], [4, 0, 0], [2, 3, 0]),
        "before": ([0, 0, 0], [4, 0, 0], [-5, 1, 0]),
        "after": ([0, 0, 0], [4, 0, 0], [9, 1, 0]),
        "spatial": ([0, 0, 0], [1, 1, 1], [1, 0, 0]),
    }
    for name, (a, b, p) in closest.items():
        args = [np.array(x, dtype=float) for x in (a, b, p)]
        emit(
            f"closest_point_{name}",
            "get_closest_point_on_line",
            [
                ("a", args[0]),
                ("b", args[1]),
                ("p", args[2]),
                ("out", so.get_closest_point_on_line(*args)),
            ],
        )

    windings = {
        "unit_square_ccw": [[1, 1, 0], [-1, 1, 0], [-1, -1, 0], [1, -1, 0], [1, 1, 0]],
        "unit_square_cw": [[1, 1, 0], [1, -1, 0], [-1, -1, 0], [-1, 1, 0], [1, 1, 0]],
        "outside": [[3, 1, 0], [4, 1, 0], [4, 2, 0], [3, 2, 0], [3, 1, 0]],
        "double": [
            [1, 0, 0], [0, 1, 0], [-1, 0, 0], [0, -1, 0],
            [1, 0, 0], [0, 1, 0], [-1, 0, 0], [0, -1, 0], [1, 0, 0],
        ],
    }
    for name, pts in windings.items():
        arr = np.array(pts, dtype=float)
        emit(
            f"winding_{name}",
            "get_winding_number",
            [("points", arr), ("value", so.get_winding_number(arr))],
        )

    for n in (3, 4, 5, 6, 8):
        for sname in ("right", "up"):
            start = np.array(VECTORS[sname], dtype=float)
            emit(
                f"compass_{n}_{sname}",
                "compass_directions",
                [
                    ("n", float(n)),
                    ("start", start),
                    ("points", so.compass_directions(n, start)),
                ],
            )
    start = np.array([2.0, 0.0, 0.0])
    emit(
        "compass_5_scaled",
        "compass_directions",
        [("n", 5.0), ("start", start), ("points", so.compass_directions(5, start))],
    )

    triangles = {
        "unit": ([0, 0], [1, 0], [0, 1]),
        "degenerate": ([0, 0], [1, 1], [2, 2]),
        "big": ([-3, -1], [4, 0], [1, 5]),
    }
    for name, (a, b, c) in triangles.items():
        args = [np.array(x, dtype=float) for x in (a, b, c)]
        emit(
            f"tri_area_{name}",
            "tri_area",
            [("a", args[0]), ("b", args[1]), ("c", args[2]), ("value", so.tri_area(*args))],
        )

    inside = {
        "center": ([0.25, 0.25], [0, 0], [1, 0], [0, 1]),
        "outside": ([2.0, 2.0], [0, 0], [1, 0], [0, 1]),
        "edge": ([0.5, 0.5], [0, 0], [1, 0], [0, 1]),
        "cw_winding": ([0.25, 0.25], [0, 0], [0, 1], [1, 0]),
    }
    for name, (p, a, b, c) in inside.items():
        args = [np.array(x, dtype=float) for x in (p, a, b, c)]
        emit(
            f"inside_triangle_{name}",
            "is_inside_triangle",
            [
                ("p", args[0]),
                ("a", args[1]),
                ("b", args[2]),
                ("c", args[3]),
                ("value", 1.0 if so.is_inside_triangle(*args) else 0.0),
            ],
        )

    paths = {
        "hit": ([-2, 0, 0], [2, 0, 0], [[0, -1, 0], [0, 1, 0], [1, 1, 0]]),
        "miss": ([-2, 5, 0], [2, 5, 0], [[0, -1, 0], [0, 1, 0], [1, 1, 0]]),
        "touch": ([0, -1, 0], [0, 1, 0], [[0, -1, 0], [0, 1, 0], [1, 1, 0]]),
    }
    for name, (s, e, path) in paths.items():
        arr = np.array(path, dtype=float)
        emit(
            f"line_intersects_path_{name}",
            "line_intersects_path",
            [
                ("start", np.array(s, dtype=float)),
                ("end", np.array(e, dtype=float)),
                ("path", arr),
                (
                    "value",
                    1.0
                    if so.line_intersects_path(
                        np.array(s, dtype=float), np.array(e, dtype=float), arr
                    )
                    else 0.0,
                ),
            ],
        )

    polylines = {
        "square": [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]],
        "spatial": [[0, 0, 0], [1, 1, 1], [2, 0, 2]],
        "single": [[3, 3, 3]],
    }
    for name, pts in polylines.items():
        arr = np.array(pts, dtype=float)
        emit(
            f"polyline_{name}",
            "poly_line_length",
            [("points", arr), ("value", so.poly_line_length(arr))],
        )
        emit(
            f"center_of_mass_{name}",
            "center_of_mass",
            [("points", arr), ("out", so.center_of_mass(arr))],
        )

    complexes = {
        "unit": (1.0, 0.0),
        "i": (0.0, 1.0),
        "general": (-2.5, 0.75),
    }
    for name, (re, im) in complexes.items():
        z = complex(re, im)
        emit(
            f"complex_to_r3_{name}",
            "complex_to_R3",
            [("z", [re, im]), ("out", so.complex_to_R3(z))],
        )
        p = np.array([re, im, 7.0])
        back = so.R3_to_complex(p)
        emit(
            f"r3_to_complex_{name}",
            "R3_to_complex",
            [("point", p), ("z", [back.real, back.imag])],
        )

    for dim, thickness in ((4, 2), (5, 1), (5, 3)):
        emit(
            f"thick_diagonal_{dim}_{thickness}",
            "thick_diagonal",
            [
                ("dim", float(dim)),
                ("thickness", float(thickness)),
                ("matrix", so.thick_diagonal(dim, thickness).astype(np.float64)),
            ],
        )


def main():
    if not os.path.isdir(REF):
        raise SystemExit(f"Reference checkout missing at {REF}")
    gen_rotation_matrices()
    gen_rotation_between()
    gen_quaternions()
    gen_euler()
    gen_vector_ops()
    gen_geometry_helpers()

    out = os.path.abspath(OUT_PATH)
    with open(out, "w") as f:
        for line in HEADER:
            f.write(line + "\n")
        for line in LINES:
            f.write(line + "\n")
    n_cases = sum(1 for line in LINES if line.startswith("case "))
    print(f"wrote {out} ({n_cases} cases)")


if __name__ == "__main__":
    main()
