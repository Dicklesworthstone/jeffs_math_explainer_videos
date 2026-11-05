"""Check 15_4's growing circle and the poles' marks from source text, on canonical and on the repair, and plant faults.

Geometry: which labels the circle crosses, at which radii, with half sizes measured on preview02's frames.
Draw order: walks the statements with the engine's rule as read at the pin: a fade roots a mob only if it is
not on stage yet, and on top, while add_to_scene removes and re-appends. Also mirrors the new Rust tests.
It does not draw, and a real fade could still surprise it.

Usage: python3 -I -B sweep_check.py CANONICAL_RS STAGED_RS
"""

import math
import re
import sys
from pathlib import Path

OLD, NEW = (Path(a).read_text(encoding="utf8") for a in sys.argv[1:3])
A, R_END, R_START, GROW_S, G0_PREVIEW = 3.0, 3.162, 0.02, 1.4, 64.665
UNIT, STROKE_HALF = 0.95, 0.025 / 0.95
# half sizes in plane units, measured on the frame before the ring starts
SIZES = {"sqrt10": (0.64, 0.12), "x=3": (0.32, 0.10), "i": (0.05, 0.12), "-i": (0.13, 0.12)}
# CMR10 advance widths in ems, for the caption fit; width = ems x size x 0.01
EMS = dict(zip("abcdefghijklmnopqrstuvwxyz", [0.5, 0.556, 0.444, 0.556, 0.444, 0.306, 0.5, 0.556, 0.278, 0.306, 0.528, 0.278, 0.833, 0.556, 0.5, 0.556, 0.528, 0.392, 0.394, 0.389, 0.556, 0.528, 0.722, 0.528, 0.528, 0.444]))
EMS.update({" ": 0.333, ";": 0.278, ":": 0.278, "−": 0.778, ",": 0.278, "'": 0.278})
ALTERNATIVE = ["a loop's output turns", "count zeros − poles"]
PANEL_W, SPACING = 2.3, 2.65
MEASURED_LONGEST = 1.78  # "at most one extension;" on preview02's frames


def num(pattern, text, default=None):
    m = re.search(pattern, text)
    return float(m.group(1)) if m else default


def layout(source):
    names = re.search(r"pub const PLANE_NAME_X: \[f64; 2\] = \[(-?[\d.]+), (-?[\d.]+)\];", source)
    if names:
        xs = float(names.group(1)), float(names.group(2))
    else:
        found = re.findall(r"cp\.p\(C::new\((-?[\d.]+), poles\[\d\] \* POLE_NAME_OUT\)\)", source)
        xs = float(found[0]), float(found[1])
    out = num(r"POLE_NAME_OUT: f64 = ([\d.]+);", source)
    y = num(r"CENTRE_NAME_Y: f64 = (-?[\d.]+);", source, -0.45)
    pad = num(r"BACK_PAD: f64 = ([\d.]+);", source)
    # radius_label_at(3, 1): the spoke's middle, 0.55 off it, away from the axis
    length = math.hypot(A, 1.0)
    says = (A / 2 + 0.55 * 1.0 / length, 0.5 + 0.55 * A / length)
    return {"sqrt10": says, "x=3": (A, y), "i": (xs[0], out), "-i": (xs[1], -out)}, pad


def band(at, half):
    """The radii at which the circle's stroke touches this box, or None if it stays clear."""
    w, h = half
    far = max(math.hypot(at[0] + a * w - A, at[1] + b * h) for a in (-1, 1) for b in (-1, 1))
    dx, dy = max(abs(at[0] - A) - w, 0.0), max(abs(at[1]) - h, 0.0)
    near = math.hypot(dx, dy)
    lo, hi = near - STROKE_HALF, far + STROKE_HALF
    return None if lo > R_END else (lo, min(hi, R_END))


def preview_time(r):
    return G0_PREVIEW + GROW_S * (r - R_START) / (R_END - R_START)


def distance_to_spokes(z):
    best = 9.0
    for im in (1.0, -1.0):
        d, e = (-A, im), (z[0] - A, z[1])
        t = max(0.0, min(1.0, (e[0] * d[0] + e[1] * d[1]) / (d[0] * d[0] + d[1] * d[1])))
        best = min(best, math.hypot(e[0] - t * d[0], e[1] - t * d[1]))
    return best


def geometry(source):
    faults, rows = [], []
    where, pad = layout(source)
    for name, at in where.items():
        b = band(at, SIZES[name])
        if b:
            rows.append(f"{name}: circle crosses it at r {b[0]:.3f}-{b[1]:.3f}, {GROW_S * (b[1] - b[0]) / (R_END - R_START):.3f} s, preview02 {preview_time(b[0]):.2f}-{preview_time(b[1]):.2f} s")
        else:
            rows.append(f"{name}: circle never reaches it; nearest {math.hypot(max(abs(at[0] - A) - SIZES[name][0], 0.0), max(abs(at[1]) - SIZES[name][1], 0.0)) - R_END:.3f} past the finished rim")
    if pad is None:
        for name in ("i", "-i"):
            if band(where[name], SIZES[name]):
                faults.append(f"circle crosses the name {name}")
    else:
        for name in ("i", "-i"):
            if band(where[name], SIZES[name]):
                faults.append(f"circle crosses the name {name}")
            x, y = where[name]
            w, h = SIZES[name]
            if x + w > -0.2 or min(math.hypot(x + a * w, y + b * h) for a in (-1, 1) for b in (-1, 1)) < 1.1:
                faults.append(f"the name {name} is too near the axis or the radius 1 ring")
        p = pad / UNIT
        for name in ("sqrt10", "x=3"):
            x, y = where[name]
            w, h = SIZES[name]
            for a in (-1, 1):
                for b in (-1, 1):
                    z = (x + a * (w + p), y + b * (h + p))
                    if math.hypot(z[0] - A, z[1]) > R_END - STROKE_HALF - 0.2:
                        faults.append(f"backing of {name} reaches the finished rim")
                    if abs(z[1]) < 0.2:
                        faults.append(f"backing of {name} covers the axis")
                    if abs(math.hypot(*z) - 1.0) < 0.1:
                        faults.append(f"backing of {name} covers the radius 1 ring")
                    if distance_to_spokes(z) < 0.1:
                        faults.append(f"backing of {name} covers a spoke")
    return sorted(set(faults)), rows


def captions(source):
    block = source[source.index("pub const CAPTIONS") : source.index("pub const ASK_PREFIX")]
    rows, faults = [], []
    texts = re.findall(r'"([^"]+)"', block)
    longest_est = max(sum(EMS[c] for c in t) * 18 * 0.01 for t in texts)
    scale = MEASURED_LONGEST / (sum(EMS[c] for c in "at most one extension;") * 18 * 0.01)
    for t in texts + ALTERNATIVE:
        est = sum(EMS[c] for c in t) * 18 * 0.01 * scale
        rows.append(f"{t!r}: about {est:.2f} scene units")
        if est > PANEL_W - 0.3:
            faults.append(f"caption {t!r} is wider than its panel allows")
    if longest_est * scale * 2 > 2 * SPACING - 0.3:
        faults.append("captions run into each other")
    return faults, rows


MARKS = ["m_back", "m_ring"]


def walk_order(source):
    """Return the draw-order faults of 15_1 and of 15_3 to 15_4."""
    helper = re.search(r"fn root_under.*?\n\}\n", source, flags=re.S)
    reversed_helper = bool(helper and "on_top.iter().chain(mobs)" in helper.group(0))
    faults = []

    def members(expr, groups):
        expr = expr.strip()
        if expr.startswith("&"):
            expr = expr[1:].strip()
        if expr.startswith("["):
            out = []
            for n in expr[1:-1].split(","):
                if n.strip():
                    out += members(n, groups)
            return out
        return groups.get(expr, [expr])

    def apply(block, groups, label):
        scene = []
        statement = re.compile(
            r"root_under\(stage, (?P<rm>&\[(?:[^\[\]]|\[\d\])*\]|&\w+), &(?P<rt>\w+)\)\?;"
            r"|fade_in_all\(stage, (?P<fi>&\[(?:[^\[\]]|\[\d\])*\]|&\w+), [\d.]+\)\?;"
            r"|stage\.add_to_scene\((?P<add>\w+)\)\?;"
            r"|let mut over_ring = vec!\[(?P<over>[^\]]*)\];"
            r"|over_ring\.extend\(&(?P<ox>\w+)\);"
            r"|stage\.play\(tween_linear\(grown"
            r"|fade_out_all\(stage, &(?P<fo>all), [\d.]+\)\?;"
            r"|let mut all = vec!\[(?P<all>[^\]]*)\];"
            r"|all\.extend\((?P<ax>\w+)\);"
        )

        def root(m):
            if m in scene:
                scene.remove(m)
            scene.append(m)

        def order_check():
            top = max((scene.index(m) for m in MARKS if m in scene), default=None)
            if top is None:
                return
            for thing in ("old", "spokes", "ring"):
                if thing in scene and scene.index(thing) > top:
                    faults.append(f"{label}: {thing} is above the poles' marks")

        for s in statement.finditer(block):
            g = s.groupdict()
            if g["rm"]:
                mobs, top = members(g["rm"], groups), members(g["rt"], groups)
                for m in top + mobs if reversed_helper else mobs + top:
                    root(m)
            elif g["fi"]:
                for m in members(g["fi"], groups):
                    if m not in scene:
                        scene.append(m)
                order_check()
            elif g["add"]:
                for m in members(g["add"], groups):
                    root(m)
            elif g["over"] is not None:
                groups["over_ring"] = members("[" + g["over"] + "]", groups)
            elif g["ox"]:
                groups["over_ring"] = groups.get("over_ring", []) + members(g["ox"], groups)
            elif s.group(0).startswith("stage.play"):
                order_check()
                ring = scene.index("ring") if "ring" in scene else -1
                for label_name, back in (("says", "backs[0]"), ("centre_name", "backs[1]")):
                    if scene.index(label_name) < ring:
                        faults.append(f"{label}: ring is above {label_name}")
                    elif back not in scene or not ring < scene.index(back) < scene.index(label_name):
                        faults.append(f"{label}: {label_name} has no backing between it and the ring")
            elif g["all"] is not None:
                groups["all"] = members("[" + g["all"] + "]", groups)
            elif g["ax"]:
                groups["all"] = groups.get("all", []) + members(g["ax"], groups)
            elif g["fo"]:
                for m in groups.get("all", []):
                    if m in scene:
                        scene.remove(m)
                for m in scene:
                    if m.startswith("backs") or m in MARKS or m in ("ring", "old", "spokes"):
                        faults.append(f"{label}: {m} is left on stage after the fade out")

    code = source[: source.index("#[cfg(test)]\nmod tests")]
    if "let backs = [" in code:
        backs = {"backs": ["backs[0]", "backs[1]"]}
    else:
        backs = {}
    first = code[code.index('say(&kit, stage, "15_1")') : code.index('say(&kit, stage, "15_2")')]
    apply(first.replace("fade_in_all(stage, &pole_mobs, 0.5)?;", "fade_in_all(stage, &[m_back, m_ring], 0.5)?;"), {"marks": list(MARKS), "guides": ["guides"]}, "15_1")
    last = code[code.index('say(&kit, stage, "15_3")') : code.index("let title = kit.text(")]
    apply(last, {"marks_here": list(MARKS), "poles_here": list(MARKS) + ["names"], **backs}, "15_3-15_4")
    return sorted(set(faults))


def mirror_of_rust_tests(source):
    start = source.find('say(&kit, stage, "15_1")')
    scene = source[start : source.find("let title = kit.text(")]
    problems = []
    for root, shown in (
        ("root_under(stage, &[ring], &marks)?;", "fade_in_all(stage, &[ring], 0.5)?;"),
        ("root_under(stage, &[old], &marks_here)?;", "fade_in_all(stage, &[old], 0.4)?;"),
        ("root_under(stage, &spokes, &marks_here)?;", "fade_in_all(stage, &spokes, 0.5)?;"),
        ("root_under(stage, &[ring, backs[0], backs[1]], &over_ring)?;", "stage.play(tween_linear(grown, r3, 1.4)?)?;"),
    ):
        at, then = scene.find(root), scene.find(shown)
        if at < 0 or then < 0 or at > then:
            problems.append(root)
    if "stage.add_to_scene(ring)" in scene:
        problems.append("ring still rooted on its own")
    if "over_ring.extend(&marks_here);" not in scene or "all.extend(backs);" not in scene:
        problems.append("marks over the ring, backs fading out")
    return problems


def run(source):
    geo, rows = geometry(source)
    cap, cap_rows = captions(source)
    return geo + cap + walk_order(source), rows, cap_rows


def once(text, a, b):
    if text.count(a) != 1:
        raise SystemExit(f"plant does not apply, {text.count(a)} matches: {a!r}")
    return text.replace(a, b)


old_faults, old_rows, _ = run(OLD)
new_faults, new_rows, cap_rows = run(NEW)
print(f"canonical: {len(old_faults)} faults")
for f in old_faults:
    print("  ", f)
print("  sweep on canonical:")
for r in old_rows:
    print("    ", r)
print(f"repair: {len(new_faults)} faults")
for f in new_faults:
    print("  ", f)
print("  sweep on the repair (crossings are now behind a backing):")
for r in new_rows:
    print("    ", r)
print("captions on the repair, widths scaled to the measured longest:")
for r in cap_rows:
    print("  ", r)

PLANTS = {
    "no re-add in 15_1": lambda t: once(t, "        root_under(stage, &[ring], &marks)?;\n", ""),
    "no re-add at the old ring": lambda t: once(t, "        root_under(stage, &[old], &marks_here)?;\n", ""),
    "no re-add at the spokes": lambda t: once(t, "        root_under(stage, &spokes, &marks_here)?;\n", ""),
    "ring rooted on its own": lambda t: once(t, "        root_under(stage, &[ring, backs[0], backs[1]], &over_ring)?;\n", "        stage.add_to_scene(ring)?;\n"),
    "marks left out over the ring": lambda t: once(t, "        over_ring.extend(&marks_here);\n", ""),
    "backs never fade out": lambda t: once(t, "        all.extend(backs);\n", ""),
    "helper puts the top first": lambda t: once(t, "for &m in mobs.iter().chain(on_top) {", "for &m in on_top.iter().chain(mobs) {"),
    "i back on the right of the axis": lambda t: once(t, "PLANE_NAME_X: [f64; 2] = [-0.42, -0.5];", "PLANE_NAME_X: [f64; 2] = [0.42, -0.5];"),
    "-i back on the right of the axis": lambda t: once(t, "PLANE_NAME_X: [f64; 2] = [-0.42, -0.5];", "PLANE_NAME_X: [f64; 2] = [-0.42, 0.5];"),
    "backing far too big": lambda t: once(t, "BACK_PAD: f64 = 0.05;", "BACK_PAD: f64 = 0.6;"),
    "x = 3 up on the axis": lambda t: once(t, "CENTRE_NAME_Y: f64 = -0.45;", "CENTRE_NAME_Y: f64 = -0.2;"),
    "caption too long": lambda t: once(t, '"on an open region:"', '"on an open region of the plane:"'),
}
caught = 0
for name, plant in PLANTS.items():
    faults, _rows, _caps = run(plant(NEW))
    mirror = mirror_of_rust_tests(plant(NEW))
    caught += bool(faults)
    print(f"plant: {name}: {'CAUGHT' if faults else 'MISSED'} ({len(faults)}); Rust-test text mirror {'fails' if mirror else 'passes'}")
print(f"plants caught: {caught}/{len(PLANTS)}")

mirror_new, mirror_old = mirror_of_rust_tests(NEW), mirror_of_rust_tests(OLD)
print(f"Rust-test text mirror on the repair: {mirror_new or 'all hold'}")
print(f"Rust-test text mirror on canonical (must fail): {len(mirror_old)} problems")

FORMULA = ['line.wait_for(stage, "the square root of ten")?;', "fade_in_all(stage, &[says], 0.4)?;", "stage.play(tween_linear(grown, r3, 1.4)?)?;", "line.wait_sentence(stage, 1)?;", "fade_in_all(stage, &[reach], 0.5)?;", "fade_in_all(stage, &end_marks, 0.4)?;"]
missing = [s for s in FORMULA if s not in NEW]
print(f"anchor formula's constants in the staged file: {'all there' if not missing else missing}")
ok = old_faults and not new_faults and caught == len(PLANTS) and not mirror_new and mirror_old and not missing
print("RESULT", "pass" if ok else "FAIL")
sys.exit(0 if ok else 1)
