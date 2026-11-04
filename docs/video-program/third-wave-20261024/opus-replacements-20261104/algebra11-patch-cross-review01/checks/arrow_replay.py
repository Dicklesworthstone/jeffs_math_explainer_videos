#!/usr/bin/env python3
"""Run anywhere. Replays every cycle arrow the candidate draws, in Python, from fmn pin a399d626's formulas.

Arc law from crates/fmn-library/src/line.rs (Arrow::build_with_tip_index, key_dimensions) and the ring's
path from crates/fmn-anim/src/motion_path.rs (PathFunc::eval). Sampled polylines, not drawn pixels.
The candidate has not been rendered, so nothing here is evidence of how it looks.
"""
import math

SEP, LOOP_BEND, SWAP_BEND, WRONG_BEND, WALK_BEND = 0.16, 1.2, 1.0, 2.2, 1.0
BUFF, THICK, TIP_RATIO, TIP_ANGLE, MAX_TIP, MAX_W = 0.08, 3.0, 5.0, math.pi / 3, 0.5, 0.1


def rot(p, a):
    return (p[0] * math.cos(a) - p[1] * math.sin(a), p[0] * math.sin(a) + p[1] * math.cos(a))


def arrow(start, end, arc, samples=120):
    vx, vy = end[0] - start[0], end[1] - start[1]
    length = math.hypot(vx, vy)
    short = length - BUFF
    width = THICK * 0.015
    if MAX_W / (width / short) < 1:
        width *= MAX_W / (width / short)
    tip_w = TIP_RATIO * width
    tip_l = tip_w / (2 * math.tan(TIP_ANGLE / 2))
    if MAX_TIP / (tip_l / short) < 1:
        ratio = MAX_TIP / (tip_l / short)
        tip_l *= ratio
        tip_w *= ratio
    r = length / 2 / math.sin(arc / 2)
    mid = ((start[0] + end[0]) / 2, (start[1] + end[1]) / 2)
    half = rot((vx / 2, vy / 2), math.pi / 2)
    c = (mid[0] + half[0] / math.tan(arc / 2), mid[1] + half[1] / math.tan(arc / 2))
    s0 = rot((start[0] - c[0], start[1] - c[1]), BUFF / r)
    e0 = rot((end[0] - c[0], end[1] - c[1]), -BUFF / r)
    shaft_angle = arc - (2 * BUFF + tip_l) / r
    shaft = []
    for k in range(samples + 1):
        p = rot(s0, shaft_angle * k / samples)
        shaft.append((c[0] + p[0], c[1] + p[1]))
    base = shaft[-1]
    tip = (c[0] + e0[0], c[1] + e0[1])
    radial = (base[0] - c[0], base[1] - c[1])
    size = math.hypot(*radial)
    n = (radial[0] / size, radial[1] / size)
    corners = [(base[0] + n[0] * tip_w / 2, base[1] + n[1] * tip_w / 2), (base[0] - n[0] * tip_w / 2, base[1] - n[1] * tip_w / 2)]
    return {"shaft": shaft, "tip": tip, "corners": corners, "base": base, "mid": shaft[len(shaft) // 2], "from": start, "to": end}


def ring_mid(start, end, arc):
    half = ((end[0] - start[0]) / 2, (end[1] - start[1]) / 2)
    c = (start[0] + half[0] - half[1] / math.tan(arc / 2), start[1] + half[1] + half[0] / math.tan(arc / 2))
    p = rot((start[0] - c[0], start[1] - c[1]), arc / 2)
    return (c[0] + p[0], c[1] + p[1])


def seg_cross(a, b, c, d):
    def side(p, q, r):
        return (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])

    return side(a, b, c) * side(a, b, d) < 0 and side(c, d, a) * side(c, d, b) < 0


def crossings(p, q):
    return sum(seg_cross(p[i], p[i + 1], q[j], q[j + 1]) for i in range(len(p) - 1) for j in range(len(q) - 1))


def seg_dist(p, a, b):
    dx, dy = b[0] - a[0], b[1] - a[1]
    t = 0 if dx == dy == 0 else max(0, min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy)))
    return math.hypot(p[0] - a[0] - t * dx, p[1] - a[1] - t * dy)


def head_points(a):
    t, c0, c1 = a["tip"], a["corners"][0], a["corners"][1]
    return [t, c0, c1, ((t[0] + c0[0]) / 2, (t[1] + c0[1]) / 2), ((t[0] + c1[0]) / 2, (t[1] + c1[1]) / 2), a["base"]]


def clear(a, b):
    """Distance from a's head to all of b."""
    outline = b["shaft"] + [b["corners"][0], b["tip"], b["corners"][1]]
    return min(seg_dist(p, outline[i], outline[i + 1]) for p in head_points(a) for i in range(len(outline) - 1))


def link_offsets(cycle):
    n = len(cycle)
    if n < 3:
        return [(0.0, 0.0)] * n

    def tail_right(i):
        at, frm, to = cycle[i], cycle[(i + n - 1) % n], cycle[(i + 1) % n]
        return 1 / (to - at) > 1 / (frm - at)

    return [(SEP if tail_right(i) else -SEP, -SEP if tail_right((i + 1) % n) else SEP) for i in range(n)]


def build(cycle, slot_x, y, above, bend, parted=True):
    n = len(cycle)
    off = link_offsets(cycle) if parted else [(0.0, 0.0)] * n
    row = y + 0.66 if above else y - 0.42
    out = []
    for i, label in enumerate(cycle):
        nxt = cycle[(i + 1) % n]
        arc = -bend if above == (nxt > label) else bend
        out.append(((label, nxt), arc, arrow((slot_x(label) + off[i][0], row), (slot_x(nxt) + off[i][1], row), arc)))
    return out


PICTURES = [
    ("11_12 (1 3 5) above row", [1, 3, 5], lambda s: -3.4 + 1.5 * (s - 1), 1.0, True, LOOP_BEND, ("title", 3.05, 32)),
    ("11_15 (1 2 3 4) above row", [1, 2, 3, 4], lambda s: -3.4 + 1.5 * (s - 1), 1.0, True, LOOP_BEND, ("title", 3.05, 32)),
    ("11_22 (2 3 5 9 8 6) above deck", [2, 3, 5, 9, 8, 6], lambda s: -5.4 + 1.2 * (s - 1), 1.0, True, WALK_BEND, ("title", 3.05, 33)),
    ("11_4 (1 3 5) above top row", [1, 3, 5], lambda s: -3.4 + 1.5 * (s - 1), 0.75, True, LOOP_BEND, ("equation", 3.0, 34)),
]

for name, cycle, slot_x, y, above, bend, words in PICTURES:
    print(f"== {name}")
    for parted in (False, True):
        links = build(cycle, slot_x, y, above, bend, parted)
        tag = "candidate" if parted else "baseline "
        cross = sum(crossings(links[i][2]["shaft"], links[j][2]["shaft"]) for i in range(len(links)) for j in range(i + 1, len(links)))
        pairs = []
        for k, ((a, b), arc, ar) in enumerate(links):
            nxt = links[(k + 1) % len(links)][2]
            pairs.append(clear(ar, nxt))
        others = []
        for k, (_, _, ar) in enumerate(links):
            for m, (_, _, other) in enumerate(links):
                if m != k:
                    others.append(clear(ar, other))
        top = max(p[1] for _, _, ar in links for p in ar["shaft"])
        limit = words[1] - words[2] * 0.01 * 0.6
        label_gap = min(
            min(seg_dist(p, (slot_x(s) - 0.09, y + 0.46 + 0.1), (slot_x(s) + 0.09, y + 0.46 + 0.1)) for p in ar["shaft"] + [ar["tip"]] + ar["corners"])
            for s in (range(1, 11) if len(cycle) == 6 else range(1, 7))
            for _, _, ar in links
        )
        print(f"  {tag}: shaft crossings {cross}; head to the next link at its slot, min {min(pairs):.3f}; head to any other link, min {min(others):.3f}; top {top:.2f} vs {words[0]} {limit:.2f}; nearest slot number {label_gap:.3f}")
    if True:
        for (a, b), arc, ar in build(cycle, slot_x, y, above, bend, True):
            dx = ar["tip"][0] - slot_x(b)
            bulge = ar["mid"][1] - (y + 0.66)
            hop = ring_mid((slot_x(a), y), (slot_x(b), y), arc)
            print(f"    {a}->{b}: arc {arc:+.1f}; tip {dx:+.3f} from slot {b}'s centre; shaft bulge {bulge:+.2f}; ring at mid-hop {hop[1] - y:+.2f} from the row; same side: {(bulge > 0) == (hop[1] > y)}; travels {'right' if slot_x(b) > slot_x(a) else 'left'} and tip is {'right' if ar['tip'][0] > ar['base'][0] else 'left'} of its base")

print("== 11_4 wrong arrow: slot 5 to slot 3, ends on the head of 1->3")
x = lambda s: -3.4 + 1.5 * (s - 1)
row = 0.75 + 0.66
wrong = arrow((x(5) + SEP, row), (x(3) - SEP, row), WRONG_BEND)
one_three = arrow((x(1) + SEP, row), (x(3) - SEP, row), -LOOP_BEND)
three_five = arrow((x(3) + SEP, row), (x(5) - SEP, row), -LOOP_BEND)
print(f"  distance between the two tips {math.hypot(wrong['tip'][0] - one_three['tip'][0], wrong['tip'][1] - one_three['tip'][1]):.3f} (the same anchor; each trimmed {BUFF} along its own arc)")
print(f"  wrong crosses 3->5: {crossings(wrong['shaft'], three_five['shaft'])}; wrong crosses 1->3: {crossings(wrong['shaft'], one_three['shaft'])}; wrong top {max(p[1] for p in wrong['shaft']):.2f} vs equation 2.80")

print("== swaps keep one double-headed arc: how far one shaft strays from the other, over the shared middle")


def stray(p, q):
    middle = p["shaft"][20:-20]
    return max(min(math.hypot(a[0] - b[0], a[1] - b[1]) for b in q["shaft"]) for a in middle)


for name, cycle, slot_x, y, bend in [
    ("11_12 (2 6)", [2, 6], lambda s: -3.4 + 1.5 * (s - 1), 1.0, LOOP_BEND),
    ("11_15 (5 6)", [5, 6], lambda s: -3.4 + 1.5 * (s - 1), 1.0, LOOP_BEND),
    ("11_22 (4 7)", [4, 7], lambda s: -5.4 + 1.2 * (s - 1), 1.0, WALK_BEND),
]:
    links = build(cycle, slot_x, y, False, bend)
    print(f"  {name}: stray {stray(links[0][2], links[1][2]):.4f}; lowest point {min(p[1] for p in links[0][2]['shaft']):.2f}")

print("== at each slot of the candidate: arriving head against leaving tail, in units and in 4K pixels (270 a unit)")
for name, cycle, slot_x, y, above, bend, _ in PICTURES[:3]:
    links = build(cycle, slot_x, y, above, bend, True)
    row = []
    for k, ((a, b), _, ar) in enumerate(links):
        gap = clear(ar, links[(k + 1) % len(links)][2])
        row.append(f"slot {b}: {gap:.3f} ({gap * 270:.0f} px)")
    print(f"  {name}: " + "; ".join(row))

print("== the same minimum if LINK_SEP changed; nothing here proposes changing it before a picture is seen")
for sep in (0.16, 0.20, 0.24):
    SEP = sep
    out = []
    for name, cycle, slot_x, y, above, bend, words in PICTURES[:3]:
        links = build(cycle, slot_x, y, above, bend, True)
        gap = min(clear(ar, links[(k + 1) % len(links)][2]) for k, (_, _, ar) in enumerate(links))
        top = max(p[1] for _, _, ar in links for p in ar["shaft"])
        out.append(f"{name.split()[0]} gap {gap:.3f} top {top:.2f}")
    print(f"  {sep:.2f}: " + "; ".join(out))
