#!/usr/bin/env python3
"""Generate .npy interchange fixtures from the pinned Reference (fm-xb3).

Reference: 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13, expected at
scripts/manim_ref (gitignored). This script imports the Reference's own
utils/bezier.py and saves what it actually produces as NumPy `.npy` files —
the cross-engine fixture format the Gauntlet reads with its owned reader
(crates/fmn-conformance/src/npy.rs). Structural fixtures where the formulas
intentionally coincide, compared at loose f32 tolerance per §16.4.

Environment: numpy, scipy, fonttools, rich, mapbox_earcut, pyyaml, colour,
screeninfo, appdirs, tqdm, validators, addict (utils/bezier.py's transitive
import closure — no GL needed). A uv venv works:
  uv venv .venv && uv pip install numpy scipy fonttools rich mapbox_earcut \
    pyyaml colour screeninfo appdirs tqdm validators addict

Outputs (committed under crates/fmn-conformance/fixtures/npy/):
  arc_quarter_n4.npy    quadratic_bezier_points_for_arc(TAU/4, 4)   (9, 3) f64
  arc_full_n8.npy       quadratic_bezier_points_for_arc(TAU, 8)     (17, 3) f64
  arc_neg_third_n2.npy  quadratic_bezier_points_for_arc(-TAU/3, 2)  (5, 3) f64
  arc_half_n8.npy       quadratic_bezier_points_for_arc(TAU/2, 8)   (17, 3) f64
  partial_quad.npy      partial_quadratic_bezier_points(q, .25, .75) (3, 3) f64
  partial_quad_first_half.npy   partial_...(q, 0.0, 0.5)            (3, 3) f64
  partial_quad_second_half.npy  partial_...(q, 0.5, 1.0)            (3, 3) f64
  quad_eval.npy         bezier(q)(t) at t = 0, 1/8, ..., 1          (9, 3) f64
  integer_interpolate.npy  integer_interpolate(0, 10, a) pairs      (5, 2) f64
  MANIFEST.tsv          name, dtype, shape, sha256 + provenance header

The quad_eval/partial_quad/integer_interpolate rows are the §16.3
"positional API results" (fm-t1v): pure functions of the curve
parameters whose formulas intentionally coincide with ours.

The Rust side (tests/npy_interchange.rs) hardcodes the same case parameters,
recomputes with fmn-geom, and compares point-for-point; the manifest hashes
guard fixture integrity in between.
"""

import hashlib
import os
import subprocess
import sys
import types

HERE = os.path.dirname(os.path.abspath(__file__))
REF = os.path.join(HERE, "manim_ref")
OUT_DIR = os.path.join(HERE, "..", "crates", "fmn-conformance", "fixtures", "npy")

# Import Reference submodules without executing manimlib/__init__.py
# (which drags in the GL window stack).
sys.path.insert(0, REF)
pkg = types.ModuleType("manimlib")
pkg.__path__ = [os.path.join(REF, "manimlib")]
pkg.__file__ = os.path.join(REF, "manimlib", "__init__.py")
sys.modules["manimlib"] = pkg

import numpy as np  # noqa: E402

import manimlib.utils.bezier as bz  # noqa: E402

TAU = 2.0 * np.pi

# One quadratic (start, handle, end) for the partial-curve case; asymmetric
# and off-axis so nothing cancels.
PARTIAL_QUAD = np.array(
    [[-1.0, 0.5, 0.25], [0.75, 2.0, -0.5], [2.0, -1.0, 1.0]], dtype=np.float64
)


def fixtures():
    yield (
        "arc_quarter_n4",
        "quadratic_bezier_points_for_arc(TAU/4, n_components=4)",
        np.asarray(bz.quadratic_bezier_points_for_arc(TAU / 4.0, 4), dtype=np.float64),
    )
    yield (
        "arc_full_n8",
        "quadratic_bezier_points_for_arc(TAU, n_components=8)",
        np.asarray(bz.quadratic_bezier_points_for_arc(TAU, 8), dtype=np.float64),
    )
    yield (
        "arc_neg_third_n2",
        "quadratic_bezier_points_for_arc(-TAU/3, n_components=2)",
        np.asarray(bz.quadratic_bezier_points_for_arc(-TAU / 3.0, 2), dtype=np.float64),
    )
    yield (
        "arc_half_n8",
        "quadratic_bezier_points_for_arc(TAU/2, n_components=8)",
        np.asarray(bz.quadratic_bezier_points_for_arc(TAU / 2.0, 8), dtype=np.float64),
    )
    yield (
        "partial_quad",
        "partial_quadratic_bezier_points(PARTIAL_QUAD, 0.25, 0.75)",
        np.asarray(
            bz.partial_quadratic_bezier_points(PARTIAL_QUAD, 0.25, 0.75),
            dtype=np.float64,
        ),
    )
    yield (
        "partial_quad_first_half",
        "partial_quadratic_bezier_points(PARTIAL_QUAD, 0.0, 0.5)",
        np.asarray(
            bz.partial_quadratic_bezier_points(PARTIAL_QUAD, 0.0, 0.5),
            dtype=np.float64,
        ),
    )
    yield (
        "partial_quad_second_half",
        "partial_quadratic_bezier_points(PARTIAL_QUAD, 0.5, 1.0)",
        np.asarray(
            bz.partial_quadratic_bezier_points(PARTIAL_QUAD, 0.5, 1.0),
            dtype=np.float64,
        ),
    )
    # Positional API results (§16.3): evaluate one quadratic along its
    # parameter, and the integer/residue split the Reference uses to
    # walk a curve list.
    quad_fn = bz.bezier(PARTIAL_QUAD)
    ts = np.arange(9, dtype=np.float64) / 8.0
    yield (
        "quad_eval",
        "bezier(PARTIAL_QUAD)(t) for t in {0, 1/8, ..., 1}",
        np.asarray([quad_fn(t) for t in ts], dtype=np.float64),
    )
    alphas = [0.05, 0.25, 0.5, 0.75, 0.95]
    yield (
        "integer_interpolate",
        "integer_interpolate(0, 10, a) for a in {0.05, 0.25, 0.5, 0.75, 0.95}",
        np.asarray(
            [bz.integer_interpolate(0, 10, a) for a in alphas], dtype=np.float64
        ),
    )


def ref_commit() -> str:
    try:
        return (
            subprocess.run(
                ["git", "-C", REF, "rev-parse", "HEAD"],
                capture_output=True,
                text=True,
                check=True,
                timeout=15,
            ).stdout.strip()
        )
    except Exception:
        return "unknown"


def main() -> None:
    os.makedirs(OUT_DIR, exist_ok=True)
    rows = []
    for name, formula, arr in fixtures():
        assert (
            arr.dtype == np.float64 and arr.ndim == 2 and arr.shape[1] in (2, 3)
        ), name
        # The Reference sometimes hands back F-contiguous views; the
        # interchange subset is C order only (crates/fmn-conformance/src/npy.rs).
        arr = np.ascontiguousarray(arr)
        path = os.path.join(OUT_DIR, f"{name}.npy")
        np.save(path, arr)
        with open(path, "rb") as f:
            digest = hashlib.sha256(f.read()).hexdigest()
        shape = "x".join(str(d) for d in arr.shape)
        rows.append((f"{name}.npy", "<f8", shape, digest, formula))
        print(f"  {name}.npy  {shape}  {digest[:12]}…")

    manifest = os.path.join(OUT_DIR, "MANIFEST.tsv")
    with open(manifest, "w") as f:
        f.write("# fmn npy fixture manifest v1\n")
        f.write(f"# reference: 3b1b/manim @ {ref_commit()}\n")
        f.write(f"# generator: scripts/gen_npy_fixtures.py (numpy {np.__version__})\n")
        f.write("# columns: file\tdtype\tshape\tsha256\tformula\n")
        for row in rows:
            f.write("\t".join(row) + "\n")
    print(f"wrote {manifest}")


if __name__ == "__main__":
    main()
