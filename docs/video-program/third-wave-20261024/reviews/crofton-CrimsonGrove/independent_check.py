#!/usr/bin/env python3
"""Independent recomputation of the crofton film's numeric claims.

Reviewer: CrimsonGrove, for Bead mx-video-program-program-87do.22.2 (review
of the tend-0610 crofton intake). Consumer: root's verdict on that bead and
the lane's repairs. Retire it once the claims it checks are pinned in the
lane's own oracle and native facts.

Standard library only (no numpy, so no shared random generator). The inputs
are only the film's data: the coast's pieces, the card, the sticks and the
gap, transcribed from docs/storyboard-draft.md and src/model.rs. No code is
imported from, or copied out of, the lane's oracle.py. Its methods differ:

  * a crossing is a meeting point found by intersecting each line with each
    piece directly (a line against an arc by acos, not floor arithmetic on
    one-way pieces). A tangent line meets an arc at one point, and a point
    on a joint is counted once;
  * an average over slides is exact: for one direction, the count is
    constant between the heights where it can change (joints, ends, arc
    extremes), so each interval is weighted by its length;
  * the dividers walk by scanning arc length and bisecting for the first
    point exactly one step away, not by chord walking or circle meetings;
  * random lines through pins use Python's own generator.

    python3 -I -B independent_check.py   # report on stdout, exit 1 on a DIFF
"""

import math
import random
import sys
from fractions import Fraction as Fr

GAP = 2.0
CARD_W, CARD_H = 16.0, 9.0
START = (-7.3, -0.5)
HEADING = -75
# ("S", length) a straight run; ("A", radius, turn degrees, left positive);
# ("T", degrees) a corner turned on the spot (the rippled beach)
COAST = [
    ("A", Fr(2), 75),
    ("S", Fr(7, 2)),
    ("A", Fr(5, 4), 105),
    ("S", Fr(3, 2)),
    ("A", Fr(3, 4), -195),
    ("S", Fr(3, 2)),
    ("A", Fr(7, 4), 165),
    ("A", Fr(1), -120),
    ("S", Fr(1)),
    ("A", Fr(3, 5), 180),
    ("A", Fr(1), -105),
]
NAMES = ["cove", "beach", "climb", "rise", "headland", "fall", "bay", "point", "run", "hook", "end"]

diffs = []


def report(what, got, claimed, ok):
    flag = "ok  " if ok else "DIFF"
    print(f"{flag} {what}: computed {got}; film says {claimed}")
    if not ok:
        diffs.append(what)


def note(what):
    print(f"note {what}")


def r(x, places):
    """Round half away from zero (as the film's facts do)."""
    q = Fr(x).limit_denominator(10**12) if isinstance(x, float) else Fr(x)
    s = 1 if q >= 0 else -1
    n = abs(q) * 10**places
    whole = int(n) + (1 if n - int(n) >= Fr(1, 2) else 0)
    return Fr(s * whole, 10**places)


# --------------------------------------------------------------- geometry


def build(pieces, start=START, heading=HEADING):
    x, y = start
    h = math.radians(heading)
    out = []
    for p in pieces:
        if p[0] == "T":
            h += math.radians(p[1])
        elif p[0] == "S":
            L = float(p[1])
            q = (x + L * math.cos(h), y + L * math.sin(h))
            out.append(("S", (x, y), q))
            x, y = q
        else:
            rad, turn = float(p[1]), math.radians(p[2])
            side = 1 if turn > 0 else -1
            c = (x - side * rad * math.sin(h), y + side * rad * math.cos(h))
            a0 = math.atan2(y - c[1], x - c[0])
            out.append(("A", c, rad, a0, a0 + turn))
            x, y = c[0] + rad * math.cos(a0 + turn), c[1] + rad * math.sin(a0 + turn)
            h += turn
    return out


def length_of(g):
    return math.dist(g[1], g[2]) if g[0] == "S" else g[2] * abs(g[4] - g[3])


def point(g, u):
    if g[0] == "S":
        return (g[1][0] + u * (g[2][0] - g[1][0]), g[1][1] + u * (g[2][1] - g[1][1]))
    a = g[3] + u * (g[4] - g[3])
    return (g[1][0] + g[2] * math.cos(a), g[1][1] + g[2] * math.sin(a))


class Curve:
    def __init__(self, geom):
        self.geom = geom
        self.starts = [0.0]
        for g in geom:
            self.starts.append(self.starts[-1] + length_of(g))
        self.length = self.starts[-1]

    def at(self, s):
        s = min(max(s, 0.0), self.length)
        for i, g in enumerate(self.geom):
            if s <= self.starts[i + 1] or i == len(self.geom) - 1:
                L = length_of(g)
                return point(g, 0.0 if L == 0 else min(1.0, (s - self.starts[i]) / L))
        raise AssertionError

    def polyline(self, per_cm=200):
        pts = [point(self.geom[0], 0.0)]
        for g in self.geom:
            n = 1 if g[0] == "S" else max(2, int(length_of(g) * per_cm))
            pts.extend(point(g, k / n) for k in range(1, n + 1))
        return pts


def unit_normal(beta_deg):
    b = math.radians(beta_deg)
    return (-math.sin(b), math.cos(b))


def heights_where_count_changes(geom, beta):
    """Joints, ends and arc extremes, as heights across lines of direction beta."""
    n = unit_normal(beta)
    psi = math.atan2(n[1], n[0])
    hs = []
    for g in geom:
        for u in (0.0, 1.0):
            p = point(g, u)
            hs.append(p[0] * n[0] + p[1] * n[1])
        if g[0] == "A":
            lo, hi = sorted((g[3], g[4]))
            hc = g[1][0] * n[0] + g[1][1] * n[1]
            k = math.ceil((lo - psi) / math.pi)
            while psi + k * math.pi <= hi:
                hs.append(hc + g[2] * math.cos(k * math.pi))
                k += 1
    return hs


def meets_one_line(geom, n, h, closed=False):
    """Meeting points of the curve with the line {x : n.x = h}. Each piece
    owns its start and not its end (the last piece of an open curve owns
    both), so a joint on the line counts once and a tangent point once.
    None if a straight run lies along the line."""
    psi = math.atan2(n[1], n[0])
    count = 0
    last = len(geom) - 1
    for i, g in enumerate(geom):
        own_end = i == last and not closed
        if g[0] == "S":
            ha = g[1][0] * n[0] + g[1][1] * n[1]
            hb = g[2][0] * n[0] + g[2][1] * n[1]
            if ha == hb:
                if ha == h:
                    return None
                continue
            t = (h - ha) / (hb - ha)
            count += 0 <= t < 1 or (own_end and t == 1)
        else:
            c, rad, a0, a1 = g[1], g[2], g[3], g[4]
            x = (h - (c[0] * n[0] + c[1] * n[1])) / rad
            if abs(x) > 1:
                continue
            base = math.acos(max(-1.0, min(1.0, x)))
            sols = {round((psi + base) % (2 * math.pi), 12), round((psi - base) % (2 * math.pi), 12)}
            for th in sols:
                for k in range(-2, 3):
                    t = (th + 2 * math.pi * k - a0) / (a1 - a0)
                    if 0 <= t < 1 or (own_end and abs(t - 1) < 1e-12):
                        count += 1
                        break
    return count


def meets(geom, beta, slide, gap=GAP, closed=False):
    """Meeting points with the sheet's lines n.x = slide + k gap."""
    n = unit_normal(beta)
    hs = heights_where_count_changes(geom, beta)
    lo, hi = min(hs), max(hs)
    total = 0
    for k in range(math.floor((lo - slide) / gap) - 1, math.ceil((hi - slide) / gap) + 2):
        m = meets_one_line(geom, n, slide + k * gap, closed)
        if m is None:
            return None
        total += m
    return total


def slide_distribution(geom, beta, gap=GAP, closed=False):
    """{count: share of slides in [0, gap)}, exact for this direction."""
    cuts = sorted({h % gap for h in heights_where_count_changes(geom, beta)} | {0.0, gap})
    dist = {}
    for a, b in zip(cuts, cuts[1:]):
        if b - a < 1e-13:
            continue
        c = meets(geom, beta, (a + b) / 2, gap, closed)
        dist[c] = dist.get(c, 0.0) + (b - a) / gap
    return dist


def clearance(geom, beta, slide, gap=GAP):
    worst = math.inf
    for h in heights_where_count_changes(geom, beta):
        t = (h - slide) / gap
        worst = min(worst, abs(t - round(t)) * gap)
    return worst


def travel(geom, beta, per_cm=2000):
    n = unit_normal(beta)
    tot = 0.0
    for g in geom:
        k = 1 if g[0] == "S" else max(2, int(length_of(g) * per_cm))
        hs = [point(g, j / k)[0] * n[0] + point(g, j / k)[1] * n[1] for j in range(k + 1)]
        tot += sum(abs(b - a) for a, b in zip(hs, hs[1:]))
    return tot


def walk(curve, step, h=0.002):
    """Dividers: from each landing, the first point further along exactly
    `step` away in a straight line, by scanning and bisection."""
    s0, here = 0.0, curve.at(0.0)
    landings = [(here, 0.0)]
    while True:
        s = s0
        found = None
        while s < curve.length:
            s_next = min(s + h, curve.length)
            if math.dist(here, curve.at(s_next)) >= step:
                a, b = s, s_next
                for _ in range(60):
                    m = (a + b) / 2
                    if math.dist(here, curve.at(m)) >= step:
                        b = m
                    else:
                        a = m
                found = b
                break
            s = s_next
        if found is None:
            return landings, math.dist(here, curve.at(curve.length))
        s0, here = found, curve.at(found)
        landings.append((here, s0))


def segments_cross(p, q, a, b):
    def side(u, v, w):
        return (v[0] - u[0]) * (w[1] - u[1]) - (v[1] - u[1]) * (w[0] - u[0])
    return side(p, q, a) * side(p, q, b) < 0 and side(a, b, p) * side(a, b, q) < 0


# --------------------------------------------------------------- the checks


def main():
    geom = build(COAST)
    coast = Curve(geom)
    straight = sum((p[1] for p in COAST if p[0] == "S"), Fr(0))
    pi_part = sum((p[1] * Fr(abs(p[2]), 180) for p in COAST if p[0] == "A"), Fr(0))
    L = float(straight) + float(pi_part) * math.pi
    print("== the coast")
    report("exact length", f"{straight} + {pi_part} pi = {L:.6f}", "15/2 + 1399/240 pi = 25.81", (straight, pi_part) == (Fr(15, 2), Fr(1399, 240)) and r(L, 2) == Fr(2581, 100))
    report("arc-length sum of placed pieces", f"{coast.length:.9f}", "the exact length", abs(coast.length - L) < 1e-9)
    # smooth joins: each piece leaves heading the way the last arrived
    def heading_at(g, u):
        if g[0] == "S":
            return math.atan2(g[2][1] - g[1][1], g[2][0] - g[1][0])
        a = g[3] + u * (g[4] - g[3])
        return a + (math.pi / 2 if g[4] > g[3] else -math.pi / 2)
    bends = [abs((heading_at(b, 0) - heading_at(a, 1) + math.pi) % (2 * math.pi) - math.pi) for a, b in zip(geom, geom[1:])]
    gaps_ = [math.dist(point(a, 1), point(b, 0)) for a, b in zip(geom, geom[1:])]
    report("pieces join without gaps or corners", f"max gap {max(gaps_):.1e} cm, max bend {max(bends):.1e} rad", "joined smoothly", max(gaps_) < 1e-12 and max(bends) < 1e-9)
    pts = coast.polyline(400)
    rx, ry = max(abs(p[0]) for p in pts), max(abs(p[1]) for p in pts)
    report("reach on the card", f"|x| {rx:.3f}, |y| {ry:.3f}", "|x| 7.30, |y| 2.02, a quarter cm inside 16 x 9", rx < 8 - 0.25 and ry < 4.5 - 0.25 and r(rx, 2) == Fr(730, 100) and r(ry, 2) == Fr(202, 100))
    chords = list(zip(pts, pts[1:]))
    boxes = [(min(p[0], q[0]), max(p[0], q[0]), min(p[1], q[1]), max(p[1], q[1])) for p, q in chords]
    hits = 0
    for i in range(len(chords)):
        for j in range(i + 2, len(chords)):
            bi, bj = boxes[i], boxes[j]
            if bi[1] < bj[0] or bj[1] < bi[0] or bi[3] < bj[2] or bj[3] < bi[2]:
                continue
            hits += segments_cross(*chords[i], *chords[j])
    report("the coast never crosses itself", f"{hits} crossings among {len(chords)} chords", "never", hits == 0)
    ends = math.dist(coast.at(0), coast.at(coast.length))
    report("ends apart", f"{ends:.4f}", "14.7 (14.73)", r(ends, 1) == Fr(147, 10))
    # the beach and the headland (01_3)
    beach = geom[1]
    report("beach level and 3.5 cm", f"y {beach[1][1]:.6f} to {beach[2][1]:.6f}, {length_of(beach):.6f} cm", "a straight 3.5 cm run at -1.98", abs(beach[1][1] - beach[2][1]) < 1e-12 and abs(length_of(beach) - 3.5) < 1e-12 and r(beach[1][1], 2) == Fr(-198, 100))
    head = geom[4]
    mid = (head[3] + head[4]) / 2
    out = (math.cos(mid), math.sin(mid))
    apex = (head[1][0] + head[2] * out[0], head[1][1] + head[2] * out[1])
    above = max((p[0] - apex[0]) * out[0] + (p[1] - apex[1]) * out[1] for p in pts)
    tilt = math.degrees(math.atan2(out[1], out[0])) - 90
    report("headland tangent at its middle has the coast on one side", f"highest point {above:.2e} above it; tilt {tilt:.4f} degrees", "touches at one point; ruler tilts 7.5 degrees", above < 1e-9 and abs(tilt - 7.5) < 1e-9)

    print("\n== crossings at the placements shown (meeting points, direct intersection)")
    shown = {0: (0.45, 7), 45: (1.20, 8), 90: (0.30, 7), 135: (1.35, 8)}
    for beta, (slide, claimed) in shown.items():
        c = meets(geom, beta, slide)
        cl = clearance(geom, beta, slide)
        best = max(range(40), key=lambda k: (round(clearance(geom, beta, k * 0.05), 9), -k)) * 0.05
        report(f"direction {beta}, slide {slide:.2f}", f"{c} crossings, clearance {cl:.3f} cm, clearest 0.05 grid slide {best:.2f}", f"{claimed} crossings, clearance >= 0.15, slide {slide:.2f}", c == claimed and cl >= 0.15 and abs(best - slide) < 1e-9)
    # which line and which piece, walking the coast (02_4, 02_6)
    # sort crossings by arc length: find each meeting's position by scanning
    walk_order = []
    prev = None
    samples = [coast.at(s) for s in [j * coast.length / 200000 for j in range(200001)]]
    for j, p in enumerate(samples):
        side = math.floor((p[1] - 0.45) / GAP)
        if prev is not None and side != prev:
            s = j * coast.length / 200000
            idx = max(i for i in range(len(geom)) if coast.starts[i] <= s)
            walk_order.append((NAMES[idx], max(side, prev)))
        prev = side
    pieces_seen = ",".join(w[0] for w in walk_order)
    lines = sorted({w[1] for w in walk_order})
    per_line = [sum(1 for w in walk_order if w[1] == k) for k in lines]
    report("first placement in walking order", f"{pieces_seen}; per line bottom to top {per_line}", "cove,climb,rise,fall,bay,bay,end; lower line 4, upper 3", pieces_seen == "cove,climb,rise,fall,bay,bay,end" and per_line == [4, 3])
    lower = [w[0] for w in walk_order if w[1] == lines[0]]
    upper = [w[0] for w in walk_order if w[1] == lines[1]]
    note(f"02_6 says lower: cove, climb, twice round the bay -> {lower}; upper: either side of the headland and near the end -> {upper}")
    # the live slide of 04_1: from slide 0.45 up 1.5 cm, and the whole gap
    def runs(beta, s_from, s_to, steps):
        out_ = []
        for i in range(steps + 1):
            c = meets(geom, beta, s_from + (s_to - s_from) * i / steps)
            if out_ and out_[-1][0] == c:
                out_[-1][1] += 1
            else:
                out_.append([c, 1])
        return [(c, n * (s_to - s_from) / steps) for c, n in out_]
    shown_runs = runs(0, 0.45, 1.95, 15000)
    seq = ",".join(str(c) for c, _ in shown_runs)
    short = min(w for _, w in shown_runs)
    report("04_1 slide sequence over 1.5 cm", f"{seq}, shortest run {short:.3f} cm", "7,9,8,7, shortest 0.056", seq == "7,9,8,7" and abs(short - 0.056) < 0.002)
    # exact runs over the whole gap: the count is constant between change heights
    cuts = sorted({(h - 0.45) % GAP for h in heights_where_count_changes(geom, 0)} | {0.0, GAP})
    exact_runs = []
    for a, b in zip(cuts, cuts[1:]):
        if b - a < 1e-13:
            continue
        c = meets(geom, 0, 0.45 + (a + b) / 2)
        if exact_runs and exact_runs[-1][0] == c:
            exact_runs[-1][2] = b
        else:
            exact_runs.append([c, a, b])
    note("direction 0, the whole gap from the first placement (count, from, to): " + ", ".join(f"{c} {a:.4f}..{b:.4f}" for c, a, b in exact_runs))
    brief = [(c, a, b) for c, a, b in exact_runs if b - a < 0.02]
    report("the tangent flicker the slide stops short of", "; ".join(f"{c} crossings from {a:.4f} to {b:.4f} cm past the start" for c, a, b in brief) or "none",
           "1.568 to 1.573, 0.005 wide, past the 1.5 cm shown", len(brief) == 1 and brief[0][1] > 1.5 and abs(brief[0][1] - 1.568) < 0.001 and abs(brief[0][2] - 1.573) < 0.001)

    print("\n== travel, averages and the identity")
    claims = {0: ("15.26", "7.63"), 45: ("17.49", "8.74"), 90: ("16.45", "8.23"), 135: ("15.82", "7.91")}
    means = []
    for beta, (tr_c, mean_c) in claims.items():
        t = travel(geom, beta)
        d = slide_distribution(geom, beta)
        exact_mean = sum(c * w for c, w in d.items())
        means.append(exact_mean)
        counts = sorted(d)
        report(f"direction {beta} travel and slide mean", f"travel {t:.4f}, exact slide mean {exact_mean:.4f}, counts {counts[0]}..{counts[-1]}", f"{tr_c}, {mean_c}", f"{r(t, 2):.2f}" == tr_c and f"{r(exact_mean, 2):.2f}" == mean_c and abs(exact_mean - t / GAP) < 1e-5)
    four = sum(means) / 4
    report("four directions", f"mean {four:.5f} crossings, x pi {four * math.pi:.4f}, error {(four * math.pi / L - 1) * 100:.3f}%", "8.13, 25.53, -1.1%", f"{r(four, 2):.2f}" == "8.13" and f"{r(four * math.pi, 2):.2f}" == "25.53")
    note(f"said-versus-shown: 8.13 x pi = {8.13 * math.pi:.4f} (reads 25.54), the film shows 25.53 beside 8.13 (11_4, 23_2, 28_3)")
    dirs = 3600
    mean_travel = sum(travel(geom, 180 * (i + 0.5) / dirs, 400) for i in range(dirs)) / dirs
    report("average travel over every direction", f"{mean_travel:.4f}, x pi/2 = {mean_travel * math.pi / 2:.4f}", "16.43, the length 25.81", f"{r(mean_travel, 2):.2f}" == "16.43" and abs(mean_travel * math.pi / 2 - L) < 2e-3)
    note(f"said-versus-shown: 8.22 x pi = {8.22 * math.pi:.4f} (11_5 handles it in small print); 2 x 25.81 = 51.62 while 19_4 says 51.63 (2L = {2 * L:.4f})")
    note(f"said-versus-shown: 17.49 / 2 = 8.745 on the 11_1 table row that shows 8.74 (exact 45-degree travel {travel(geom, 45):.4f})")
    half_turn_only = sum(travel(geom, 90 * (i + 0.5) / 900, 400) for i in range(900)) / 900
    report("negative control: directions over a quarter turn only", f"{half_turn_only * math.pi / 2:.4f} cm", "differs from the length (the half-turn range matters)", abs(half_turn_only * math.pi / 2 - L) > 0.05)

    skew = sum(2 * math.sin(math.radians(180 * (i + 0.5) / 900)) ** 2 * travel(geom, 180 * (i + 0.5) / 900, 400) for i in range(900)) / 900
    report("negative control: directions weighted 2 sin^2 instead of evenly", f"{skew * math.pi / 2:.4f} cm", "differs from the length (even turning matters)", abs(skew * math.pi / 2 - L) > 0.05)
    report("negative control: worth pi x gap, the half forgotten", f"{mean_travel / GAP * math.pi * GAP:.4f} cm", "twice the length, so the worth's half matters", abs(mean_travel * math.pi - 2 * L) < 2e-3)
    bent = [p if i != 4 else ("A", Fr(4, 5), -195) for i, p in enumerate(COAST)]
    bg = build(bent)
    report("negative control: headland radius 4/5 instead of 3/4", f"length {Curve(bg).length:.4f}; first placement {meets(bg, 0, 0.45)} crossings", "the checks see a changed coast (length not 25.81)", f"{r(Curve(bg).length, 2):.2f}" != "25.81")

    print("\n== one drop: the exact distribution of a count over slide and direction")
    def drop_distribution(g, gap, ndir=3600, closed=False):
        tot = {}
        for i in range(ndir):
            for c, w in slide_distribution(g, 180 * (i + 0.5) / ndir, gap, closed).items():
                tot[c] = tot.get(c, 0.0) + w / ndir
        return tot
    dist = drop_distribution(geom, GAP)
    mean = sum(c * w for c, w in dist.items())
    var = sum(c * c * w for c, w in dist.items()) - mean * mean
    worth = math.pi * GAP / 2
    sd1 = math.sqrt(var) * worth
    report("mean count", f"{mean:.5f}", "8.22 = 2L/(pi gap)", abs(mean - 2 * L / (math.pi * GAP)) < 2e-4)
    note(f"exact distribution of one drop's count: { {k: round(v, 5) for k, v in sorted(dist.items())} }")
    note(f"21_4 'in a thousand drops, single counts ran anywhere from five to twelve' is the seeded sample's range; the exact range is {min(dist)}..{max(dist)}")
    # where a drop crosses most often: the widest such window, recounted on a dense polyline
    top = max(dist)
    windows = []
    for i in range(3600):
        beta = 180 * (i + 0.5) / 3600
        cuts_ = sorted({h % GAP for h in heights_where_count_changes(geom, beta)} | {0.0, GAP})
        windows += [(b_ - a_, beta, (a_ + b_) / 2) for a_, b_ in zip(cuts_, cuts_[1:]) if b_ - a_ > 1e-12 and meets(geom, beta, (a_ + b_) / 2) == top]
    w_, beta_, mid_ = max(windows)
    n_ = unit_normal(beta_)
    sides = [math.floor(((q[0] * n_[0] + q[1] * n_[1]) - mid_) / GAP) for q in coast.polyline(20000)]
    poly = sum(1 for u_, v_ in zip(sides, sides[1:]) if u_ != v_)
    report(f"drops meeting the sheet {top} times exist", f"chance {dist[top] * 100:.3f}%; widest window direction {beta_:.3f}, slides {mid_ - w_ / 2:.4f}..{mid_ + w_ / 2:.4f}: direct {top}, polyline (20,000 per cm) {poly}", "the seeded thousand ran 5 to 12 (a sample statement)", poly == top)
    report("one drop's spread, cm", f"exact {sd1:.4f}, rounds {r(sd1, 1):.1f}", "4.3 (21_4, 25_3; measured over the seeded 1,000 drops)", f"{r(sd1, 1):.1f}" == "4.3")
    report("ten drops' spread, cm", f"{sd1 / math.sqrt(10):.3f}", "1.4", f"{r(sd1 / math.sqrt(10), 1):.1f}" == "1.4")
    # the exact distribution of the sum of ten independent drops
    pmf = {0: 1.0}
    for _ in range(10):
        nxt = {}
        for a_, pa in pmf.items():
            for b_, pb in dist.items():
                nxt[a_ + b_] = nxt.get(a_ + b_, 0.0) + pa * pb
        pmf = nxt
    far = sum(p_ for s_, p_ in pmf.items() if abs(s_ / 10 * worth - L) > 1.0)
    report("ten drops off by over 1 cm", f"{far * 100:.2f}% exactly (the film's 1,000 repeats gave 43%)", "43%, more than four times in ten", f"{r(far * 100, 0):.0f}" == "43")
    need = math.ceil((sd1 / 0.1) ** 2)
    report("drops for a spread of 0.1 cm", f"exact {need}, to the nearest hundred {round(need, -2)}", "1,868 on screen, 'about nineteen hundred' (22_3)", round(need, -2) == 1900)
    report("a thousand drops' spread, cm", f"{sd1 / math.sqrt(1000):.4f}", "0.14 (22_1)", f"{r(sd1 / math.sqrt(1000), 2):.2f}" == "0.14")
    for gap, claimed in ((4.0, "8.2"), (8.0, "15.9")):
        d_ = drop_distribution(geom, gap, 1800)
        m_ = sum(c * w for c, w in d_.items())
        s_ = math.sqrt(sum(c * c * w for c, w in d_.items()) - m_ * m_) * math.pi * gap / 2
        report(f"gap {gap:g}: mean x worth", f"{m_ * math.pi * gap / 2:.4f}", "25.8 on average (25_2)", abs(m_ * math.pi * gap / 2 - L) < 0.01)
        report(f"gap {gap:g}: one drop's spread, cm", f"exact {s_:.4f}, rounds {r(s_, 1):.1f}", f"{claimed} (25_3; the seeded 1,000 drops)", f"{r(s_, 1):.1f}" == claimed)
    # the exact spreads, cross-checked by Python's own random drops
    mc = random.Random(1868)
    for gap in (2.0, 8.0):
        xs = [meets(geom, 180 * mc.random(), gap * mc.random(), gap) for _ in range(20000)]
        m_ = sum(xs) / len(xs)
        sd_ = math.sqrt(sum((x - m_) ** 2 for x in xs) / (len(xs) - 1)) * math.pi * gap / 2
        note(f"gap {gap:g}: 20,000 random drops (Python random, seed 1868) give mean x worth {m_ * math.pi * gap / 2:.3f} and one drop's spread {sd_:.3f} cm; counts {min(xs)}..{max(xs)}")

    print("\n== the chart: lines one at a time (19_1)")
    # each line's own count, weighted by its patch of chart (distance in cm,
    # direction as a share of the half turn); slivers under 1e-9 cm are tangencies
    area, widest = {}, {}
    for i in range(3600):
        beta = 180 * (i + 0.5) / 3600
        hs = sorted(set(heights_where_count_changes(geom, beta)))
        n = unit_normal(beta)
        for a, b in zip(hs, hs[1:]):
            if b - a < 1e-9:
                continue
            m = meets_one_line(geom, n, (a + b) / 2)
            area[m] = area.get(m, 0.0) + (b - a) / 3600
            if m not in widest or b - a > widest[m][0]:
                widest[m] = (b - a, beta, (a + b) / 2)
    most = max(area)
    w_, beta_, mid_ = widest[most]
    n_ = unit_normal(beta_)
    sides = [math.copysign(1, (q[0] * n_[0] + q[1] * n_[1]) - mid_) for q in coast.polyline(20000)]
    poly = sum(1 for u_, v_ in zip(sides, sides[1:]) if u_ != v_)
    note("chart area by a line's own count (cm x share of the half turn): " + ", ".join(f"{k}: {v:.4f}" for k, v in sorted(area.items())))
    report("the chart's patches add up to the mean travel", f"sum of count x area {sum(k * v for k, v in area.items()):.4f}", "16.43 (19_4's 51.63 / pi)", f"{r(sum(k * v for k, v in area.items()), 2):.2f}" == "16.43")
    def hull_perimeter(points):
        pts_ = sorted(set(points))
        def cross_(o, a, b):
            return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
        lower, upper = [], []
        for q in pts_:
            while len(lower) >= 2 and cross_(lower[-2], lower[-1], q) <= 0:
                lower.pop()
            lower.append(q)
        for q in reversed(pts_):
            while len(upper) >= 2 and cross_(upper[-2], upper[-1], q) <= 0:
                upper.pop()
            upper.append(q)
        h_ = lower[:-1] + upper[:-1]
        return sum(math.dist(a, b) for a, b in zip(h_, h_[1:] + h_[:1]))
    hit = sum(v for k, v in area.items() if k >= 1) * math.pi
    card_lines = 2 * (CARD_W + CARD_H)
    report("lines meeting the coast, as chart area", f"{hit:.4f} cm rad; the coast's convex hull perimeter {hull_perimeter(coast.polyline(400)):.4f}", "Cauchy: the hull's perimeter", abs(hit - hull_perimeter(coast.polyline(400))) < 0.01)
    report("19_1 'Most lines miss it entirely'", f"{hit / card_lines * 100:.1f}% of the lines meeting the card ({card_lines:g} cm rad, 17_6's region) cross the coast; {hit / (2 * 9.2 * math.pi) * 100:.1f}% of the whole 18.4 cm x half-turn chart", "most miss", hit / card_lines < 0.5)
    report("19_1 'Some cross once'", f"{area[1] * math.pi / hit * 100:.1f}% of the lines that meet the coast cross it once", "some", True)
    report("most crossings by a single line", f"{most} (widest window {w_:.4f} cm at direction {beta_:.3f}, distance {mid_:.4f}; polyline recount {poly})", "19_1's shading runs 1 to 12", most == 12 and poly == most)

    print("\n== sticks, the bent stick, the square, the coin")
    stick30 = build([("S", Fr(6))], (0.0, 0.0), 30)
    d30 = slide_distribution(stick30, 0)
    report("6 cm stick at 30 degrees, over slides", f"{ {k: round(v, 6) for k, v in d30.items()} }", "1 or 2, half the time each, mean 1.5", set(d30) == {1, 2} and abs(d30[2] - 0.5) < 1e-9)
    def range_over(g, closed=False, ndir=720):
        seen = set()
        for i in range(ndir):
            seen |= set(slide_distribution(g, 180 * (i + 0.5) / ndir, GAP, closed))
        return sorted(seen)
    hook = build([("S", Fr(3)), ("T", 90), ("S", Fr(3))], (0.0, 0.0), 0)
    straight6 = build([("S", Fr(6))], (0.0, 0.0), 0)
    report("bent and straight sticks' single counts", f"L {range_over(hook)}; straight {range_over(straight6)}", "1 to 4 against 0 to 3", range_over(hook) == [1, 2, 3, 4] and range_over(straight6) == [0, 1, 2, 3])
    square = build([("S", Fr(2)), ("T", 90), ("S", Fr(2)), ("T", 90), ("S", Fr(2)), ("T", 90), ("S", Fr(2))], (-1.0, -1.0), 0)
    sq_counts = range_over(square, closed=True)
    four = sum(slide_distribution(square, 180 * (i + 0.5) / 3600, GAP, True).get(4, 0.0) for i in range(3600)) / 3600
    report("square of side 2: counts and share of fours", f"{sq_counts}, {four:.6f}", "2 or 4; 4/pi - 1 = 0.273240", sq_counts == [2, 4] and abs(four - (4 / math.pi - 1)) < 1e-4)
    s2 = math.sqrt(2)
    tilted = [("S", (s2, s2), (0.0, 2 * s2)), ("S", (0.0, 2 * s2), (-s2, s2)), ("S", (-s2, s2), (0.0, 0.0)), ("S", (0.0, 0.0), (s2, s2))]
    note(f"28_5 edge: the square turned 45 degrees with a corner on a line meets the sheet {meets(tilted, 0, 0.0, GAP, True)} times (chance zero; the draft's note names only a side lying along a line)")
    coin = [("A", (0.0, 1.0), 1.0, -math.pi / 2, 3 * math.pi / 2)]
    report("05_4: a 2 cm coin grazing two lines", f"{meets(coin, 0, 0.0, GAP, True)} meeting points", "two, one on each line", meets(coin, 0, 0.0, GAP, True) == 2)

    print("\n== fixed directions (23, 24) and the six-direction discrepancy")
    for m, lo_c, hi_c in ((1, "-100", "57"), (2, "-21.5", "11.1"), (4, "-5.2", "2.6"), (6, "-2.3", "1.2")):
        t = math.pi / (2 * m)
        lo = t / math.tan(t) if m > 1 else 0.0
        hi = t / math.sin(t)
        places = 0 if m == 1 else 1
        got_lo, got_hi = (lo - 1) * 100, (hi - 1) * 100
        fmt = (lambda x: f"{r(x, places):.{places}f}")
        report(f"{m} directions: stick band", f"{got_lo:.4f}% .. {got_hi:+.4f}%", f"{lo_c}% .. +{hi_c}%", fmt(got_lo) == lo_c and fmt(got_hi) == hi_c)
    low6 = (2 + math.sqrt(3)) * math.pi / 12 - 1
    high6 = (math.pi / 12) / math.sin(math.pi / 12) - 1
    note(f"six directions in closed form: (2 + sqrt 3) pi / 12 - 1 = {low6 * 100:.4f}% (a stick along a family), (pi/12) / sin(pi/12) - 1 = {high6 * 100:+.4f}% (midway)")
    # what would give the paper's -2.26% with its +1.15%?
    for name, value in (("pi as 22/7", (2 + math.sqrt(3)) * (22 / 7) / 12 - 1), ("error relative to the estimate", 1 - 1 / ((2 + math.sqrt(3)) * math.pi / 12))):
        note(f"six directions, {name}: {value * 100:+.4f}%")
    kink = math.pi / 12  # the band's lower end is a kink: slope pi/12 per radian off the family
    note(f"to read -2.26% the stick must sit {((0.0226 - abs(low6)) / -kink if abs(low6) > 0.0226 else 0) and math.degrees((abs(low6) - 0.0226) / kink):.3f} degrees off a family (the lower end is a V-shaped minimum)")
    est6 = sum(travel(geom, 30 * k) for k in range(6)) / 6 / GAP * worth
    report("coast, six slid directions", f"{est6:.4f} ({(est6 / L - 1) * 100:+.3f}%)", "25.73, inside the band", f"{r(est6, 2):.2f}" == "25.73" and low6 <= est6 / L - 1 <= high6)
    report("longimeter spacing for one crossing per mm", f"{12 / math.pi:.4f} mm", "12/pi = 3.82 mm", f"{r(12 / math.pi, 2):.2f}" == "3.82")

    print("\n== the dividers (01, 25, 26, 27)")
    claims_d = {4.0: (4, "18.7"), 2.0: (11, "22.6"), 1.0: (25, "25.1"), 0.5: (51, "25.6")}
    totals = {}
    for step, (steps_c, len_c) in claims_d.items():
        lands, last = walk(coast, step)
        total = (len(lands) - 1) * step + last
        totals[step] = total
        report(f"dividers {step:g} cm", f"{len(lands) - 1} steps, last {last:.4f}, total {total:.4f}", f"{steps_c} steps, {len_c}", len(lands) - 1 == steps_c and f"{r(total, 1):.1f}" == len_c)
        if step == 2.0:
            hs, he = coast.starts[4], coast.starts[5]
            k = next(k for k in range(1, len(lands)) if hs < lands[k - 1][1] < he)
            report("the step that leaves the headland", f"step {k}, across {lands[k][1] - lands[k - 1][1]:.4f} cm of coast", "step 6, 3.08 cm", k == 6 and f"{r(lands[k][1] - lands[k - 1][1], 2):.2f}" == "3.08")
            report("the short last bit", f"{last:.4f}", "0.64", f"{r(last, 2):.2f}" == "0.64")
    report("halving the step from 1 to 0.5", f"+{(totals[0.5] / totals[1.0] - 1) * 100:.3f}%", "+2%", f"{r((totals[0.5] / totals[1.0] - 1) * 100, 0):.0f}" == "2")
    sweep = [round(0.25 + 0.05 * i, 2) for i in range(76)]
    lens = []
    for st in sweep:
        lands, last = walk(coast, st, 0.004)
        lens.append((len(lands) - 1) * st + last)
    rises = [(a, b) for (a, la), (b, lb) in zip(zip(sweep, lens), zip(sweep[1:], lens[1:])) if lb > la + 1e-6]
    note(f"01_7 'a smaller step cuts less off', swept 0.25 to 4.00 cm by 0.05: {len(rises)} places where a larger step walks further: {rises[:8]}")
    sq_open = build([("S", Fr(2)), ("T", 90), ("S", Fr(2)), ("T", 90), ("S", Fr(2)), ("T", 90), ("S", Fr(2))], (0.0, 0.0), 0)
    sqc = Curve(sq_open)
    l2, last2 = walk(sqc, 2.0)
    l19, last19 = walk(sqc, 1.9)
    t2, t19 = (len(l2) - 1) * 2.0 + last2, (len(l19) - 1) * 1.9 + last19
    report("negative control: a smaller step is not always longer", f"square outline of side 2: step 2 walks {t2:.4f}, step 1.9 walks {t19:.4f}", "1.9 shorter than 2 on this outline", t19 < t2)
    # the rippled beach (26)
    rr = Fr(7, 2) / 14
    rippled = COAST[:1] + [("T", 90), ("A", rr, -180), ("T", 90)] * 7 + COAST[2:]
    rg = build(rippled)
    rc = Curve(rg)
    r_len = float(sum((p[1] for p in rippled if p[0] == "S"), Fr(0))) + float(sum((p[1] * Fr(abs(p[2]), 180) for p in rippled if p[0] == "A"), Fr(0))) * math.pi
    report("rippled drawing's length", f"{r_len:.4f} ({rc.length:.4f} placed)", "27.8", f"{r(r_len, 1):.1f}" == "27.8" and abs(rc.length - r_len) < 1e-9)
    r_mean = sum(travel(rg, 180 * (i + 0.5) / 1800, 400) for i in range(1800)) / 1800 / GAP
    report("rippled drawing's mean count", f"{r_mean:.4f}", "8.85", f"{r(r_mean, 2):.2f}" == "8.85")
    lands, last = walk(rc, 2.0)
    rt = (len(lands) - 1) * 2.0 + last
    report("2 cm dividers on the rippled drawing", f"{rt:.4f}", "22.6", f"{r(rt, 1):.1f}" == "22.6")
    report("Richardson's halving for D = 1.25", f"{(2 ** 0.25 - 1) * 100:.3f}%", "19%", f"{r((2 ** 0.25 - 1) * 100, 0):.0f}" == "19")

    print("\n== random lines: pins against the sheet's way (16, 17, 18)")
    rng = random.Random(20261024)
    def pin_line():
        return ((rng.uniform(-8, 8), rng.uniform(-4.5, 4.5)), (rng.uniform(-8, 8), rng.uniform(-4.5, 4.5)))
    def crosses(line, stick):
        (p, q), (a, b) = line, stick
        def side(w):
            return (q[0] - p[0]) * (w[1] - p[1]) - (q[1] - p[1]) * (w[0] - p[0])
        return side(a) * side(b) < 0
    sticks = {
        "middle, upright": ((0.0, -1.0), (0.0, 1.0)),
        "corner, upright (the film's)": ((-7.0, 2.0), (-7.0, 4.0)),
        "middle, lying": ((-1.0, 0.0), (1.0, 0.0)),
        "bottom edge, upright": ((0.0, -4.4), (0.0, -2.4)),
        "bottom edge, lying": ((-1.0, -4.3), (1.0, -4.3)),
        "left edge, lying": ((-7.9, 0.0), (-5.9, 0.0)),
        "left edge, upright": ((-7.9, -1.0), (-7.9, 1.0)),
    }
    N = 300000
    hits = dict.fromkeys(sticks, 0)
    for _ in range(N):
        line = pin_line()
        for k, s in sticks.items():
            hits[k] += crosses(line, s)
    rates = {k: v / N for k, v in hits.items()}
    se = math.sqrt(0.22 * 0.78 / N)
    report("pins, middle stick", f"{rates['middle, upright'] * 100:.2f}% (+/- {200 * se:.2f})", "21.9%", abs(rates["middle, upright"] - 0.219) < 3 * se + 0.0005)
    report("pins, corner stick", f"{rates['corner, upright (the film\'s)'] * 100:.2f}%", "12.4%", abs(rates["corner, upright (the film's)"] - 0.124) < 3 * se + 0.0005)
    report("the sheet's way, any 2 cm stick", f"2 x 2 / 50 = {4 / 50 * 100:.1f}%", "8%", True)
    note("18_4 'a stick near the edge is always shortchanged' by the pins, against the same stick in the middle: " + "; ".join(f"{k} {v * 100:.2f}%" for k, v in rates.items()))
    note("the pins cross the film's corner stick 12.4% of the time, more than the even way's 8%: shortchanged only relative to the middle stick under the same pins")

    print(f"\n# {len(diffs)} differences: {diffs}")
    return 1 if diffs else 0


if __name__ == "__main__":
    sys.exit(main())
