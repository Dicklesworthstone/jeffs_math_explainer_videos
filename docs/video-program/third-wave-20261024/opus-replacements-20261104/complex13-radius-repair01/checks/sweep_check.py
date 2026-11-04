"""Chapter 13's radius sweep and pole-name sweep: where do things touch, before and after the repair?

Geometry and z-order only, read from the two source files. No render, no Cargo. The half sizes are
the ones measured on the existing 720p frames (see frame_calibration.py). "Touch" means two inked
shapes overlap, whichever is on top. "Struck" means the line is drawn over the lettering or ring.

Usage: python3 -I -B sweep_check.py CANONICAL_RS STAGED_RS
Exit 0 if the staged source has no struck contact and every clearance holds; else exit 1.
Run with --plant to confirm that each planted fault is caught.
"""

import math
import re
import sys
from pathlib import Path

UNIT = 5.0 / 4.8  # scene units per plane unit: Plane::new(.., 5.0, (-2.4, 2.4), ..)
R = 2.0
HALF = {"R": (0.08, 0.085), "i": (0.021, 0.08), "-i": (0.085, 0.08)}
RING = 0.107
RING_BACK = 0.13 / UNIT
LINE = {"spoke": 2.5, "segment": 5.0, "lit": 8.0, "axis": 2.0}
ARROW = {"x": (0.2, 0.8), "head_from": 0.55, "half": 0.145}
SCRATCH_RATE = 55.6  # degrees per second on preview02, the SCRATCH voice: frames 90-95, measured


def half_line(name):
    return LINE[name] * 0.01 / 2.0 / UNIT


def read(path):
    text = Path(path).read_text(encoding="utf8")
    code = text[: text.index("#[cfg(test)]\nmod tests")]
    m = re.search(r"C::polar\(r \* ([\d.]+), th\) \+ C::polar\(([\d.]+), th - PI / 2\.0\)", code)
    numbers = {"along": float(m.group(1)), "off": float(m.group(2))}

    def const(name, default):
        c = re.search(rf"const {name}: f64 = (-?[\d.]+);", code)
        return float(c.group(1)) if c else default

    shown = re.search(r"const R_NAME_SHOWN: \[f64; 2\] = \[([\d.]+), ([\d.]+)\];", code)
    numbers["shown"] = (float(shown.group(1)), float(shown.group(2))) if shown else None
    numbers["r_back"] = const("R_BACK", 0.0) / UNIT
    numbers["i_back"] = const("I_BACK", 0.0) / UNIT
    numbers["i_x"] = const("POLE_NAME_X", -0.28)
    numbers["mi_x"] = const("MINUS_POLE_NAME_X", 0.36)
    # z-order: is the spoke rooted before the marks are re-added, and what comes back over it?
    three = code[code.index('say(&kit, stage, "13_3")') : code.index('say(&kit, stage, "13_4")')]
    readd = re.search(r"stage\.add_to_scene\(spoke\)\?;\s*for m in \[([^\]]*(?:\[\d\][^\]]*)*)\] \{\s*stage\.add_to_scene\(m\)\?;", three)
    fade = three.find("fade_in_all(stage, &[zdot, spoke")
    numbers["over_spoke"] = readd.group(1) if readd and readd.start() < fade else ""
    numbers["waits"] = "a.set_opacity(r_name, shown, true);" in three and "r_name, zsq" not in three
    numbers["backs_gone"] = "fade_out_all(stage, &[zdot, spoke, r_name, r_back, i_back], 0.3)?;" in three
    return numbers


def r_name(n, th):
    t = math.radians(th)
    a = n["along"] * R
    return (a * math.cos(t) + n["off"] * math.sin(t), a * math.sin(t) - n["off"] * math.cos(t))


def shown(n, th):
    if not n["shown"] or not n["waits"]:
        return 1.0
    lo, hi = n["shown"]
    return min(max((math.radians(th) - lo) / (hi - lo), 0.0), 1.0)


def spoke_x_at(th, y):
    return y * math.cos(math.radians(th)) / math.sin(math.radians(th))


def windows(hits):
    out, start, last = [], None, None
    for th, hit in hits:
        if hit and start is None:
            start = th
        if not hit and start is not None:
            out.append((start, last))
            start = None
        last = th
    if start is not None:
        out.append((start, last))
    return out


def sweep(n):
    """Rows of (what, windows in degrees, drawn on top of it?)."""
    angles = [k / 10 for k in range(0, 1801)]
    hx, hy = HALF["R"]
    seg, axis, spoke = half_line("segment"), half_line("axis"), half_line("spoke")
    rows = {}

    def add(name, test, struck):
        rows[name] = (windows([(a, test(a)) for a in angles]), struck)

    over = n["over_spoke"]
    add("R across the gold segment", lambda a: shown(n, a) > 0 and abs(r_name(n, a)[1]) < hy + seg, "R on top; line through the letter")
    r_axis_struck = n["r_back"] < math.hypot(hx, hy)
    add(
        "R on the imaginary axis",
        lambda a: shown(n, a) > 0 and abs(r_name(n, a)[0]) < hx + axis,
        "axis shows through the letter" if r_axis_struck else "behind R's backing",
    )
    ring_struck = "inside[1]" not in over
    add(
        "spoke over the pole ring",
        lambda a: 1.0 < a < 179.0 and abs(spoke_x_at(a, 1.0)) < RING + spoke,
        "spoke through the ring" if ring_struck else "behind the ring's backing",
    )
    name_struck = "pole_names[0]" not in over or "i_back" not in over
    add(
        "spoke over the name i",
        lambda a: 1.0 < a < 179.0 and abs(spoke_x_at(a, 1.0) - n["i_x"]) < HALF["i"][0] + spoke,
        "spoke through the letter" if name_struck else "behind i's backing",
    )
    arrow_struck = "arrows[0]" not in over
    add(
        "spoke over the segment's arrow head",
        lambda a: 0.0 < a < 90.0 and ARROW["head_from"] * math.tan(math.radians(a)) < ARROW["half"],
        "spoke over the head" if arrow_struck else "behind the head",
    )
    return rows, dict(r_axis=r_axis_struck, ring=ring_struck, name=name_struck, arrow=arrow_struck)


def clearances(n):
    """Smallest gaps in plane units over the whole walk while R shows, and over the poles' growth."""
    out = {}
    hx, hy = HALF["R"]
    back = max(n["r_back"], 0.0)
    shown_angles = [k / 10 for k in range(0, 1801) if shown(n, k / 10) > 0]
    pts = [(a, r_name(n, a)) for a in shown_angles]
    ink_r = math.hypot(hx, hy)
    out["R to spoke"] = min(z[0] * math.sin(math.radians(a)) - z[1] * math.cos(math.radians(a)) for a, z in pts) - ink_r - half_line("spoke")
    if back:
        out["R backing to spoke"] = min(z[0] * math.sin(math.radians(a)) - z[1] * math.cos(math.radians(a)) for a, z in pts) - back - half_line("spoke")
        out["R backing to segment"] = min(z[1] for a, z in pts) - back - half_line("segment")
        out["R backing to pole backing"] = min(math.hypot(z[0], z[1] - 1) for a, z in pts) - back - RING_BACK
        out["R backing to i backing"] = min(math.hypot(z[0] - n["i_x"], z[1] - 1) for a, z in pts) - back - n["i_back"]
        out["R backing to half circle arrow"] = R - ARROW["half"] - max(math.hypot(*z) for a, z in pts) - back
        out["R backing past the letter"] = back - ink_r
    if n["i_back"]:
        out["i backing to axis"] = -n["i_x"] - n["i_back"] - half_line("axis")
        out["i backing to ring ink"] = -n["i_x"] - n["i_back"] - RING
        out["i backing past the letter"] = n["i_back"] - math.hypot(*HALF["i"])
    k = [2.0 / (2.0 + 8.0 * j / 400) for j in range(401)]
    lit = half_line("lit")
    out["growth: i to lit segment"] = min(k) - HALF["i"][1] - lit
    out["growth: ring to lit segment"] = min(k) - RING - lit
    out["growth: -i to lit segment"] = min(k) - HALF["-i"][1] - lit
    out["growth: -i to arrow head"] = ARROW["head_from"] - n["mi_x"] - HALF["-i"][0]
    out["growth: i to ring"] = -n["i_x"] - HALF["i"][0] - RING
    out["growth: -i to ring"] = n["mi_x"] - HALF["-i"][0] - RING
    return out


FLOORS = {
    "R to spoke": 0.05,
    "R backing to spoke": 0.05,
    "R backing to segment": 0.001,
    "R backing to pole backing": 0.1,
    "R backing to i backing": 0.1,
    "R backing to half circle arrow": 0.1,
    "i backing to axis": 0.1,
    "i backing to ring ink": 0.05,
    "i backing past the letter": 0.005,
    "R backing past the letter": 0.005,
    "growth: i to lit segment": 0.05,
    "growth: ring to lit segment": 0.04,
    "growth: -i to lit segment": 0.05,
    "growth: -i to arrow head": 0.05,
    "growth: i to ring": 0.05,
    "growth: -i to ring": 0.05,
}


def verdict(n):
    rows, struck = sweep(n)
    faults = [name for name, (w, _) in rows.items() if name == "R across the gold segment" and w]
    faults += [k for k, v in struck.items() if v]
    gaps = clearances(n)
    faults += [f"{k} {v:.3f} under {FLOORS[k]}" for k, v in gaps.items() if v < FLOORS.get(k, 0.0)]
    if not n["backs_gone"]:
        faults.append("backings not faded with the spoke")
    return faults


def report(label, n):
    rows, _ = sweep(n)
    print(f"--- {label}")
    print("  contacts over the walk, walker angle in degrees; seconds at the scratch rate of 55.6 deg/s")
    for name, (w, how) in rows.items():
        if not w:
            print(f"    {name:38} none")
        for a, b in w:
            print(f"    {name:38} {a:6.1f} to {b:6.1f}  {(b - a) / SCRATCH_RATE:5.2f} s  {how}")
    if n["i_back"] and "pole_names[0]" in n["over_spoke"]:
        # the cost of the backings: a stretch of the spoke is hidden for longer than the letter alone would hide it
        hidden = [(k / 10, abs(spoke_x_at(k / 10, 1.0)) < RING_BACK or abs(spoke_x_at(k / 10, 1.0) - n["i_x"]) < n["i_back"]) for k in range(10, 1790)]
        for a, b in windows(hidden):
            print(f"    {'(cost) spoke hidden at height 1':38} {a:6.1f} to {b:6.1f}  {(b - a) / SCRATCH_RATE:5.2f} s  the rest of the spoke stays drawn")
    print("  smallest clearances, plane units (1 unit = 93.75 px at 720p)")
    for k, v in clearances(n).items():
        print(f"    {k:38} {v:+.3f}")
    print("  frame 93's pose, walker 98.75 deg:")
    x = spoke_x_at(98.75, 1.0)
    print(f"    spoke to ring ink {abs(x) - RING - half_line('spoke'):+.3f}; spoke to i letter {x - n['i_x'] - HALF['i'][0] - half_line('spoke'):+.3f}; R centre x {r_name(n, 98.75)[0]:+.3f}")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    canon, staged = read(args[0]), read(args[1])
    report("canonical", canon)
    report("with this repair", staged)
    faults_canon, faults = verdict(canon), verdict(staged)
    print("--- canonical faults:", faults_canon)
    print("--- repair faults:", faults)
    if "--plant" in sys.argv:
        print("--- planted faults, each must be caught")
        plants = {
            "R shown from the start": {"shown": None},
            "R comes in too early": {"shown": (0.10, 0.20)},
            "R backing too small": {"r_back": 0.09},
            "R backing too big": {"r_back": 0.26},
            "i backing too small": {"i_back": 0.05},
            "i backing into the ring": {"i_back": 0.2},
            "i name moved onto the axis": {"i_x": -0.15},
            "-i name onto the arrow": {"mi_x": 0.5},
            "R pushed toward the spoke": {"off": 0.16},
            "R pushed onto the pole": {"along": 0.55},
            "nothing re-added over the spoke": {"over_spoke": ""},
            "ring not re-added": {"over_spoke": "arrows[0], i_back, inside[0], pole_names[0]"},
            "name's backing not re-added": {"over_spoke": "arrows[0], inside[0], inside[1], pole_names[0]"},
            "arrow not re-added": {"over_spoke": "i_back, inside[0], inside[1], pole_names[0]"},
            "backings left behind": {"backs_gone": False},
            "still fades R in with the rest": {"waits": False},
        }
        missed = []
        for name, change in plants.items():
            caught = verdict({**staged, **change})
            print(f"    {'caught' if caught else 'MISSED'}  {name}: {caught[:2]}")
            if not caught:
                missed.append(name)
        print("missed:", missed)
        if missed:
            sys.exit(1)
    sys.exit(1 if faults else 0)
