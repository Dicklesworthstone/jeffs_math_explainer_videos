#!/usr/bin/env python3
"""Generate fmn-mobject positional-parity fixtures (fm-jru, §8.4, §16.4).

The Reference's positional API (mobject.py, 3b1b/manim @ 6199a00d) is pure
bounding-box arithmetic over point arrays — no OpenGL, no fonts. Rather than
stand up the Reference's full GL/config import closure just to call these
methods, this script reproduces the exact formulas from mobject.py in pure
NumPy (the same approach gen_reference_fixtures.py takes for fmn-core), and
records inputs + expected outputs. The Rust parity test (tests/positional.rs)
rebuilds each shape, applies the same operation through the Stage, and compares
at loose f32 tolerance (§16.4) — our engine computes in f64, the records are f32.

Every formula below is transcribed straight from mobject.py; NumPy performs the
min/max reductions independently of the hand-rolled Rust accumulator, so a
shared transcription bug is very unlikely.

Output (committed): crates/fmn-mobject/fixtures/positional.txt
"""

import os

import numpy as np

# --- constants, mirrored from fmn-core (which mirrors the Reference) ---
ASPECT_RATIO = 1920.0 / 1080.0
FRAME_HEIGHT = 8.0
FRAME_WIDTH = FRAME_HEIGHT * ASPECT_RATIO
FRAME_X_RADIUS = FRAME_WIDTH / 2.0
FRAME_Y_RADIUS = FRAME_HEIGHT / 2.0

ORIGIN = np.array([0.0, 0.0, 0.0])
UP = np.array([0.0, 1.0, 0.0])
DOWN = np.array([0.0, -1.0, 0.0])
RIGHT = np.array([1.0, 0.0, 0.0])
LEFT = np.array([-1.0, 0.0, 0.0])
UR = UP + RIGHT
DL = DOWN + LEFT

OUT_PATH = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "..",
    "crates",
    "fmn-mobject",
    "fixtures",
    "positional.txt",
)


class Mob:
    """A minimal mobject: own points plus submobjects, exactly the surface the
    positional formulas touch."""

    def __init__(self, points):
        self.points = np.array(points, dtype=float).reshape(-1, 3)
        self.subs = []

    def family(self):
        out = [self]
        for s in self.subs:
            out.extend(s.family())
        return out

    # --- bounding box (mobject.py:338-360, 1512-1518) ---
    def bounding_box(self):
        all_pts = np.vstack([m.points for m in self.family() if len(m.points)])
        if len(all_pts) == 0:
            return np.zeros((3, 3))
        mins = all_pts.min(0)
        maxs = all_pts.max(0)
        mids = (mins + maxs) / 2
        return np.array([mins, mids, maxs])

    def bbox_point(self, direction):
        bb = self.bounding_box()
        idx = (np.sign(direction) + 1).astype(int)
        return np.array([bb[idx[i]][i] for i in range(3)])

    def center(self):
        return self.bounding_box()[1]

    def length_over_dim(self, dim):
        bb = self.bounding_box()
        return abs((bb[2] - bb[0])[dim])

    def get_width(self):
        return self.length_over_dim(0)

    def get_height(self):
        return self.length_over_dim(1)

    def get_coord(self, dim, direction=ORIGIN):
        return self.bbox_point(direction)[dim]

    # --- transforms (mobject.py:282-308, 919-967) ---
    def _apply(self, func, about_point=None, about_edge=ORIGIN):
        if about_point is None and about_edge is not None:
            about_point = self.bbox_point(about_edge)
        for m in self.family():
            if about_point is None:
                m.points = func(m.points)
            else:
                m.points = func(m.points - about_point) + about_point
        return self

    def shift(self, vec):
        return self._apply(lambda p: p + vec, about_edge=None)

    def scale(self, factor, about_point=None, about_edge=ORIGIN):
        factor = max(factor, 1e-8)
        return self._apply(lambda p: factor * p, about_point, about_edge)

    def stretch(self, factor, dim, about_edge=ORIGIN):
        def func(p):
            p = p.copy()
            p[:, dim] *= factor
            return p

        return self._apply(func, about_edge=about_edge)

    def center_it(self):
        return self.shift(-self.center())

    def align_on_border(self, direction, buff):
        target = np.sign(direction) * np.array([FRAME_X_RADIUS, FRAME_Y_RADIUS, 0])
        pta = self.bbox_point(direction)
        shift_val = target - pta - buff * np.array(direction)
        shift_val = shift_val * abs(np.sign(direction))
        return self.shift(shift_val)

    def next_to(self, target, direction=RIGHT, buff=0.25, aligned_edge=ORIGIN):
        if isinstance(target, Mob):
            target_point = target.bbox_point(aligned_edge + direction)
        else:
            target_point = np.array(target)
        pta = self.bbox_point(aligned_edge - direction)
        return self.shift(target_point - pta + buff * direction)

    def move_to(self, target, aligned_edge=ORIGIN):
        if isinstance(target, Mob):
            t = target.bbox_point(aligned_edge)
        else:
            t = np.array(target)
        pta = self.bbox_point(aligned_edge)
        return self.shift(t - pta)

    def align_to(self, target, direction):
        if isinstance(target, Mob):
            point = target.bbox_point(direction)
        else:
            point = np.array(target)
        for dim in range(3):
            if direction[dim] != 0:
                self.set_coord(point[dim], dim, direction)
        return self

    def set_coord(self, value, dim, direction=ORIGIN):
        curr = self.get_coord(dim, direction)
        shift_vect = np.zeros(3)
        shift_vect[dim] = value - curr
        return self.shift(shift_vect)

    def set_x(self, x):
        return self.set_coord(x, 0)

    def set_y(self, y):
        return self.set_coord(y, 1)

    def rescale_to_fit(self, length, dim, stretch):
        old = self.length_over_dim(dim)
        if old == 0:
            return self
        if stretch:
            return self.stretch(length / old, dim)
        return self.scale(length / old)

    def set_width(self, width, stretch=False):
        return self.rescale_to_fit(width, 0, stretch)

    def set_height(self, height, stretch=False):
        return self.rescale_to_fit(height, 1, stretch)

    def match_width(self, other):
        return self.rescale_to_fit(other.length_over_dim(0), 0, False)

    def match_x(self, other):
        return self.set_coord(other.get_coord(0), 0)

    def arrange(self, direction, buff, center):
        for m1, m2 in zip(self.subs, self.subs[1:]):
            m2.next_to(m1, direction, buff)
        if center:
            self.center_it()
        return self

    def arrange_in_grid(self, n_cols, h_buff, v_buff, aligned_edge=ORIGIN):
        subs = self.subs
        x_unit = h_buff + max(s.get_width() for s in subs)
        y_unit = v_buff + max(s.get_height() for s in subs)
        for index, sm in enumerate(subs):
            x, y = index % n_cols, index // n_cols
            sm.move_to(ORIGIN, aligned_edge)
            sm.shift(x * x_unit * RIGHT + y * y_unit * DOWN)
        self.center_it()
        return self


# A few reusable shapes (point sets), chosen to be asymmetric so min/mid/max and
# every direction are distinguishable.
def tri():
    return [[-1.0, -0.5, 0.0], [1.5, 0.25, 0.0], [0.5, 2.0, 0.0]]


def quad():
    return [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 1.0, 0.0], [0.0, 1.0, 0.0]]


def dot(at):
    return [list(at)]


# Each scenario: name, a builder returning (root, all_nodes_in_index_order),
# and an op applied to node 0 (using node 1+ as references where needed).
def build_single(points):
    m = Mob(points)
    return m, [m]


def build_parent(child_specs):
    parent = Mob(np.zeros((0, 3)))
    nodes = [parent]
    for spec in child_specs:
        c = Mob(spec)
        parent.subs.append(c)
        nodes.append(c)
    return parent, nodes


def build_two(a_pts, b_pts):
    a, b = Mob(a_pts), Mob(b_pts)
    return a, [a, b]


SCENARIOS = []


def scenario(name, builder, op):
    SCENARIOS.append((name, builder, op))


scenario("shift", lambda: build_single(tri()), lambda n: n[0].shift([1.0, -2.0, 0.5]))
scenario("scale_center", lambda: build_single(tri()), lambda n: n[0].scale(2.0))
scenario(
    "scale_about_edge",
    lambda: build_single(quad()),
    lambda n: n[0].scale(0.5, about_edge=DL),
)
scenario(
    "scale_about_point",
    lambda: build_single(quad()),
    lambda n: n[0].scale(3.0, about_point=np.array([1.0, 1.0, 0.0])),
)
scenario("stretch_x", lambda: build_single(quad()), lambda n: n[0].stretch(2.5, 0))
scenario("stretch_y", lambda: build_single(quad()), lambda n: n[0].stretch(0.5, 1))
scenario("center", lambda: build_single(tri()), lambda n: n[0].center_it())
scenario("to_edge_up", lambda: build_single(tri()), lambda n: n[0].align_on_border(UP, 0.5))
scenario("to_edge_left", lambda: build_single(tri()), lambda n: n[0].align_on_border(LEFT, 0.5))
scenario("to_corner_ur", lambda: build_single(tri()), lambda n: n[0].align_on_border(UR, 0.25))
scenario(
    "next_to_right",
    lambda: build_two(quad(), dot([3.0, 0.0, 0.0])),
    lambda n: n[0].next_to(n[1], RIGHT, 0.25),
)
scenario(
    "next_to_up_aligned",
    lambda: build_two(tri(), quad()),
    lambda n: n[0].next_to(n[1], UP, 0.5, aligned_edge=LEFT),
)
scenario(
    "next_to_point",
    lambda: build_single(tri()),
    lambda n: n[0].next_to([2.0, 2.0, 0.0], DOWN, 0.1),
)
scenario(
    "move_to_point",
    lambda: build_single(tri()),
    lambda n: n[0].move_to([1.0, 1.0, 0.0]),
)
scenario(
    "move_to_mob_edge",
    lambda: build_two(tri(), quad()),
    lambda n: n[0].move_to(n[1], aligned_edge=UP),
)
scenario(
    "align_to_mob",
    lambda: build_two(tri(), quad()),
    lambda n: n[0].align_to(n[1], UP),
)
scenario("set_x", lambda: build_single(tri()), lambda n: n[0].set_x(-3.0))
scenario("set_y", lambda: build_single(tri()), lambda n: n[0].set_y(2.5))
scenario("set_width_scale", lambda: build_single(quad()), lambda n: n[0].set_width(4.0))
scenario("set_height_stretch", lambda: build_single(quad()), lambda n: n[0].set_height(3.0, stretch=True))
scenario(
    "match_width",
    lambda: build_two(quad(), tri()),
    lambda n: n[0].match_width(n[1]),
)
scenario(
    "match_x",
    lambda: build_two(tri(), dot([5.0, 0.0, 0.0])),
    lambda n: n[0].match_x(n[1]),
)
scenario(
    "nested_bbox_shift",
    lambda: build_parent([quad(), dot([5.0, 5.0, 0.0])]),
    lambda n: n[0].shift([1.0, 1.0, 0.0]),
)
scenario(
    "arrange_right",
    lambda: build_parent([quad(), quad(), tri()]),
    lambda n: n[0].arrange(RIGHT, 0.5, True),
)
scenario(
    "arrange_down",
    lambda: build_parent([quad(), tri()]),
    lambda n: n[0].arrange(DOWN, 0.25, True),
)
scenario(
    "arrange_in_grid",
    lambda: build_parent([quad(), quad(), quad(), tri()]),
    lambda n: n[0].arrange_in_grid(2, 0.3, 0.3),
)


def fmt(x):
    return repr(float(x))


def main():
    lines = [
        "# positional parity fixtures (fm-jru) — generated by gen_positional_fixtures.py",
        "# Reference: 3b1b/manim @ 6199a00d, formulas from mobject.py, computed in NumPy f64.",
    ]
    for name, builder, op in SCENARIOS:
        root, nodes = builder()
        # Record inputs (own points + parent linkage) before mutating.
        parent_of = {}
        for pi, node in enumerate(nodes):
            for c in node.subs:
                parent_of[id(c)] = pi
        in_specs = []
        for i, node in enumerate(nodes):
            parent = parent_of.get(id(node), -1)
            in_specs.append((i, parent, node.points.copy()))
        op(nodes)
        lines.append(f"SCENARIO {name}")
        lines.append(f"NODES {len(nodes)}")
        for i, parent, pts in in_specs:
            flat = " ".join(fmt(v) for v in pts.reshape(-1))
            lines.append(f"IN {i} {parent} {len(pts)} {flat}")
        for i, node in enumerate(nodes):
            flat = " ".join(fmt(v) for v in node.points.reshape(-1))
            lines.append(f"OUT {i} {len(node.points)} {flat}")
        bb = root.bounding_box()
        lines.append("BBOX " + " ".join(fmt(v) for v in bb.reshape(-1)))
        lines.append("END")

    os.makedirs(os.path.dirname(OUT_PATH), exist_ok=True)
    with open(OUT_PATH, "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"wrote {len(SCENARIOS)} scenarios to {OUT_PATH}")


if __name__ == "__main__":
    main()
