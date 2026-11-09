#!/usr/bin/env python3
"""Capture topology fixtures from the pinned Reference's skia-pathops route.

Run from the repository root with the Reference environment:

    scripts/manim_ref/.venv/bin/python scripts/gen_path_boolean_fixtures.py

The output is committed as ``crates/fmn-geom/fixtures/path_booleans.txt``.
The fixture intentionally records topology (area, contour count, point-in-fill
grid) and retains the emitted Skia command stream only as audit evidence; Rust
acceptance never requires point-for-point output identity.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import subprocess

import pathops


REFERENCE_COMMIT = "6199a00d4c1b1127ebe45cb629c3f22538b10e13"
SKIA_PATHOPS_VERSION = "0.9.2"
GRID_WIDTH = 19
GRID_HEIGHT = 17

Point = tuple[float, float]
Contours = tuple[tuple[Point, ...], ...]


@dataclass(frozen=True)
class Case:
    name: str
    operation: str
    subject: Contours
    clip: Contours


def rect(x0: float, y0: float, x1: float, y1: float) -> Contours:
    return (((x0, y0), (x1, y0), (x1, y1), (x0, y1)),)


CASES = (
    Case(
        "overlap_union",
        "union",
        rect(0.0, 0.0, 2.0, 2.0),
        rect(1.0, 1.0, 3.0, 3.0),
    ),
    Case(
        "overlap_intersection",
        "intersection",
        rect(0.0, 0.0, 2.0, 2.0),
        rect(1.0, 1.0, 3.0, 3.0),
    ),
    Case(
        "overlap_difference",
        "difference",
        rect(0.0, 0.0, 2.0, 2.0),
        rect(1.0, 1.0, 3.0, 3.0),
    ),
    Case(
        "overlap_exclusion",
        "exclusion",
        rect(0.0, 0.0, 2.0, 2.0),
        rect(1.0, 1.0, 3.0, 3.0),
    ),
    Case(
        "shared_edge_union",
        "union",
        rect(-2.0, -1.0, 0.0, 1.0),
        rect(0.0, -1.0, 2.0, 1.0),
    ),
    Case(
        "partial_overlap_exclusion",
        "exclusion",
        rect(-2.0, -1.0, 1.0, 1.0),
        rect(0.0, -1.0, 2.0, 1.0),
    ),
    Case(
        "corner_tangent_union",
        "union",
        rect(-2.0, -2.0, 0.0, 0.0),
        rect(0.0, 0.0, 2.0, 2.0),
    ),
    Case(
        "donut_clip",
        "intersection",
        (
            ((-3.0, -3.0), (3.0, -3.0), (3.0, 3.0), (-3.0, 3.0)),
            ((-1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (1.0, -1.0)),
        ),
        rect(-2.0, -4.0, 2.0, 4.0),
    ),
    Case(
        "self_crossing_union",
        "union",
        (((-2.5, -1.5), (2.5, 1.5), (-2.5, 1.5), (2.5, -1.5)),),
        rect(3.0, 3.0, 3.5, 3.5),
    ),
    Case(
        "disjoint_difference",
        "difference",
        rect(-3.0, -1.0, -1.0, 1.0),
        rect(1.0, -1.0, 3.0, 1.0),
    ),
)


def reference_head() -> str:
    return subprocess.check_output(
        ["git", "-C", "scripts/manim_ref", "rev-parse", "HEAD"],
        text=True,
        timeout=10,
    ).strip()


def make_path(contours: Contours) -> pathops.Path:
    path = pathops.Path()
    for contour in contours:
        if not contour:
            continue
        path.moveTo(*contour[0])
        for point in contour[1:]:
            path.lineTo(*point)
        path.close()
    return path


def apply(case: Case) -> pathops.Path:
    operations = {
        "union": pathops.PathOp.UNION,
        "intersection": pathops.PathOp.INTERSECTION,
        "difference": pathops.PathOp.DIFFERENCE,
        "exclusion": pathops.PathOp.XOR,
    }
    return pathops.op(
        make_path(case.subject),
        make_path(case.clip),
        operations[case.operation],
    )


def encode_contours(contours: Contours) -> str:
    if not contours:
        return "-"
    return ";".join(
        "|".join(f"{x:.17g},{y:.17g}" for x, y in contour)
        for contour in contours
    )


def sample_point(x: int, y: int) -> Point:
    # Irrational-looking offsets avoid the integer and half-integer fixture
    # boundaries. The formula is mirrored in the Rust acceptance test.
    return (-3.137 + 0.371 * x, -3.083 + 0.409 * y)


def encode_fill_grid(path: pathops.Path) -> str:
    return "".join(
        "1" if path.contains(sample_point(x, y)) else "0"
        for y in range(GRID_HEIGHT)
        for x in range(GRID_WIDTH)
    )


def encode_commands(path: pathops.Path) -> str:
    names = {
        pathops.PathVerb.MOVE: "M",
        pathops.PathVerb.LINE: "L",
        pathops.PathVerb.QUAD: "Q",
        pathops.PathVerb.CUBIC: "C",
        pathops.PathVerb.CLOSE: "Z",
    }
    commands = []
    for verb, points in path:
        payload = ";".join(f"{x:.17g},{y:.17g}" for x, y in points)
        commands.append(f"{names[verb]}:{payload}")
    return "|".join(commands)


def main() -> None:
    head = reference_head()
    if head != REFERENCE_COMMIT:
        raise SystemExit(
            f"Reference checkout is {head}; expected pinned {REFERENCE_COMMIT}"
        )
    if pathops.__version__ != SKIA_PATHOPS_VERSION:
        raise SystemExit(
            f"skia-pathops is {pathops.__version__}; expected {SKIA_PATHOPS_VERSION}"
        )
    print(
        "# fmn-geom path boolean topology fixtures; generated by "
        "scripts/gen_path_boolean_fixtures.py"
    )
    print(f"# reference={REFERENCE_COMMIT} skia-pathops={pathops.__version__}")
    print(f"# grid={GRID_WIDTH}x{GRID_HEIGHT}")
    print(
        "# name\\top\\tsubject\\tclip\\tsk_area\\tsk_contours\\tsk_fill_bits"
        "\\tsk_commands"
    )
    for case in CASES:
        result = apply(case)
        fields = (
            case.name,
            case.operation,
            encode_contours(case.subject),
            encode_contours(case.clip),
            f"{result.area:.17g}",
            str(len(tuple(result.contours))),
            encode_fill_grid(result),
            encode_commands(result),
        )
        print("\t".join(fields))


if __name__ == "__main__":
    main()
