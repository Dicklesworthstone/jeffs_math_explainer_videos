#!/usr/bin/env python3
"""Facts oracle for "Two Bad Echoes Can Make One Perfect Measurement".

Run from this folder (`./x golay_echoes facts-check` does, with numpy through uv):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `golay_echoes facts`
prints (src/facts.rs), in the rows of docs/fact-sheet.md: G-F (computed), G-X
(the edges of the always/never sentences).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (which slides the
copy along the recording and adds the products that line up at each shift):
  * every score row as a polynomial product: each pair of pulses (i, j)
    contributes recording[i] * code[j] to shift i - j, accumulated over all
    pairs at once (a code against itself is A(z) * A(1/z));
  * the recording of a scene as the polynomial product code(z) * scene(z);
  * complementarity also through the chapter's equation 2,
    |A(z)|^2 + |B(z)|^2 = 2N on the unit circle (floating point, a check);
  * every complementary pair of up to eight pulses found by brute force, and
    the cancellation checked for each of them on every small scene;
  * numpy.correlate's "full" mode for every row, when numpy is installed.
"""
import cmath
import sys
from fractions import Fraction
from itertools import product

_EMITTED = {}
_FAILED = []
_CHECKS = [0]

CODE_A = (1, 1, 1, -1)
CODE_B = (1, 1, -1, 1)
STILL = (1,)
MOVED = (0, 1)
LOUD_AND_QUIET = (4, 0, 0, 1)


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"oracle: {key} emitted twice")
    _EMITTED[key] = str(value)
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    if not ok:
        _FAILED.append(name)
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)


def csv(values):
    return ",".join(str(v) for v in values)


# --------------------------------------------------------------- polynomials


def scores(recording, code):
    """Shift -> score: every pair (i, j) adds recording[i] * code[j] at i - j."""
    out = {}
    for i, r in enumerate(recording):
        for j, c in enumerate(code):
            out[i - j] = out.get(i - j, 0) + r * c
    return out


def echo(code, scene):
    """The polynomial product code(z) * scene(z), as a list."""
    out = [0] * (len(code) + len(scene) - 1)
    for i, c in enumerate(code):
        for d, h in enumerate(scene):
            out[i + d] += c * h
    return out


def add(*rows):
    out = {}
    for row in rows:
        for k, v in row.items():
            out[k] = out.get(k, 0) + v
    return out


def window(row, first, last):
    return [row.get(k, 0) for k in range(first, last + 1)]


def measurement(code, scene, noise=()):
    recording = echo(code, scene)
    recording += [0] * (len(noise) - len(recording))
    recording = [r + (noise[i] if i < len(noise) else 0) for i, r in enumerate(recording)]
    return scores(recording, code)


def ghosts(row, echoes):
    return sorted(k for k, v in row.items() if k not in echoes and v != 0)


def largest_ghost(row, echoes):
    return max((abs(v) for k, v in row.items() if k not in echoes), default=0)


def complementary(a, b):
    if len(a) != len(b) or not a:
        return False
    s = add(scores(a, a), scores(b, b))
    return all(v == (2 * len(a) if k == 0 else 0) for k, v in s.items())


# --------------------------------------------------------------------- facts


def facts():
    n = len(CODE_A)
    ra, rb = scores(CODE_A, CODE_A), scores(CODE_B, CODE_B)
    first, last = -(n - 1), n - 1

    emit("G-F1.pulses", n)
    emit("G-F1.code_a", csv(CODE_A))
    emit("G-F1.code_b", csv(CODE_B))
    emit("G-F1.complementary", str(complementary(CODE_A, CODE_B)).lower())
    # 01_3: the key's pairs are A's first two pulses and its last two
    emit("G-F1.key.agree", CODE_A[0] * CODE_A[1])
    emit("G-F1.key.disagree", CODE_A[2] * CODE_A[3])

    # 01_4 to 01_6: code A on its own
    emit("G-F2.a.scores", csv(window(ra, first, last)))
    emit("G-F2.a.peak", ra[0])
    one_off = [CODE_A[i] * CODE_A[i - 1] for i in range(1, n)]
    emit("G-F2.a.one_off.products", csv(one_off))
    emit("G-F2.a.one_off.pairs", len(one_off))
    emit("G-F2.a.one_off.agree", sum(1 for p in one_off if p == 1))
    emit("G-F2.a.one_off.disagree", sum(1 for p in one_off if p == -1))
    emit("G-F2.a.one_off.score", ra[1])
    emit("G-F2.ghost_fraction", Fraction(largest_ghost(ra, {0}), ra[0]))
    emit("G-F2.a.ghost_shifts", csv(ghosts(ra, {0})))
    emit("G-F2.a.far_shift_score", CODE_A[0] * CODE_A[-1])

    # 01_8: code B on its own
    emit("G-F2.b.scores", csv(window(rb, first, last)))
    emit("G-F2.b.ghost_shifts", csv(ghosts(rb, {0})))
    emit("G-F2.b.largest_ghost", largest_ghost(rb, {0}))

    # 01_10: the two rows added
    both = add(measurement(CODE_A, STILL), measurement(CODE_B, STILL))
    emit("G-F3.sum.scores", csv(window(both, first, last)))
    emit("G-F3.sum.peak", both[0])
    emit("G-F3.sum.largest_ghost", largest_ghost(both, {0}))
    emit("G-F3.peak_ratio", Fraction(both[0], ra[0]))

    # 01_12: the scene moved one step between the sends
    moved = add(measurement(CODE_A, STILL), measurement(CODE_B, MOVED))
    m_first, m_last = min(moved), max(moved)
    emit("G-F4.moved.step", MOVED.index(1) - STILL.index(1))
    emit("G-F4.moved.first_shift", m_first)
    emit("G-F4.moved.scores", csv(window(moved, m_first, m_last)))
    echoes = {STILL.index(1), MOVED.index(1)}
    emit("G-F4.moved.largest_ghost", largest_ghost(moved, echoes))
    emit("G-F4.moved.ghost_shifts", csv(ghosts(moved, echoes)))

    # the edge of "no single code is free of ghosts": every code of 2 to 10
    # pulses scores +1 or -1 at its far shift
    far = set()
    for length in range(2, 11):
        for code in product((1, -1), repeat=length):
            far.add(abs(scores(code, code)[length - 1]))
    emit("G-X1.far_shift_sizes", csv(sorted(far)))
    chapter_02()
    chapter_03()
    return ra, rb, both, moved


def pairs_at(x, y, k):
    """(value, inside?) of every lined-up pair of `x then y` at shift k >= 0,
    one pulse at i and the other at i + k, from the definition."""
    whole = list(x) + list(y)
    n = len(x)
    return [(whole[i] * whole[i + k], (i < n) == (i + k < n)) for i in range(len(whole) - k)]


def chapter_03():
    """The doubling: concatenation written out, products classified by
    index, rows from the polynomial products above."""
    seed = ((1,), (1,))
    two = (seed[0] + seed[1], seed[0] + tuple(-v for v in seed[1]))
    four = (two[0] + two[1], two[0] + tuple(-v for v in two[1]))
    pair = lambda p: f"{csv(p[0])};{csv(p[1])}"
    emit("G-F10.seed", pair(seed))
    emit("G-F10.seed_length", len(seed[0]))
    emit("G-F10.length_two", pair(two))
    emit("G-F10.length_four", pair(four))

    x, y = two
    shift = 1
    a = pairs_at(x, y, shift)
    b = pairs_at(x, tuple(-v for v in y), shift)
    emit("G-F10.shift", shift)
    emit("G-F10.a.inside", csv([v for v, inside in a if inside]))
    emit("G-F10.a.across", csv([v for v, inside in a if not inside]))
    emit("G-F10.b.inside", csv([v for v, inside in b if inside]))
    emit("G-F10.b.across", csv([v for v, inside in b if not inside]))
    whole_a, whole_b = four
    n = len(x)
    flips = sorted({sum(whole_a[j] != whole_b[j] for j in (i, i + shift))
                    for i in range(len(whole_a) - shift) if (i < n) != (i + shift < n)})
    emit("G-F10.flipped_in_crossing", csv(flips))
    sx, sy = scores(x, x), scores(y, y)
    emit("G-F10.old_rows_at_shift", csv([sx.get(shift, 0), sy.get(shift, 0)]))
    emit("G-F10.inside_total", sum(v for v, inside in a + b if inside))
    emit("G-F10.total", sum(v for v, _ in a + b))
    new = add(scores(whole_a, whole_a), scores(whole_b, whole_b))
    emit("G-F10.factor", Fraction(new[0], sx[0] + sy[0]))
    emit("G-F10.length_factor", Fraction(len(whole_a), len(x)))

    lengths, p = [], seed
    for _ in range(7):
        if not complementary(*p):
            raise SystemExit(f"oracle: the doubling of length {len(p[0])} does not cancel")
        lengths.append(len(p[0]))
        p = (p[0] + p[1], p[0] + tuple(-v for v in p[1]))
    emit("G-F11.lengths", csv(lengths))
    g10 = ((1, 1, -1, 1, -1, 1, -1, -1, 1, 1), (1, 1, -1, 1, 1, 1, 1, 1, -1, -1))
    emit("G-F11.length10", len(g10[0]))
    emit("G-F11.length10_complementary", str(complementary(*g10)).lower())
    emit("G-F11.length10_power_of_two", str(len(g10[0]) & (len(g10[0]) - 1) == 0).lower())

    no_flip = {k: 2 * v for k, v in scores(whole_a, whole_a).items()}
    far = 2 * n - 1
    emit("G-F12.no_flip.scores", csv(window(no_flip, -(len(whole_a) - 1), len(whole_a) - 1)))
    emit("G-F12.no_flip.far_products", csv([v for v, _ in pairs_at(x, y, far)] * 2))
    emit("G-F12.no_flip.far", no_flip[far])
    emit("G-X3.crossing_counts", csv([sum(1 for _, inside in pairs_at(x, y, k) if not inside) for k in range(2 * n)]))


def lone_echo(scene, step):
    """The scene with only its echo at `step` left."""
    return tuple(h if d == step else 0 for d, h in enumerate(scene))


def chapter_02():
    """A loud and a quiet echo. Each echo's part is measured on its own
    (the pattern sent into a scene holding only that echo), not scaled from
    the pattern's row as the Rust side does."""
    scene = LOUD_AND_QUIET
    steps = [d for d, h in enumerate(scene) if h != 0]
    loud, quiet = sorted(steps, key=lambda d: -scene[d])
    first, last = -(len(CODE_A) - 1), len(scene) + len(CODE_A) - 2

    emit("G-F5.scene", csv(scene))
    emit("G-F5.echoes", len(steps))
    emit("G-F5.loud", scene[loud])
    emit("G-F5.quiet", scene[quiet])
    emit("G-F5.quiet_step", quiet - loud)
    emit("G-F5.recording_a", csv(echo(CODE_A, scene)))

    ra = measurement(CODE_A, scene)
    loud_a = measurement(CODE_A, lone_echo(scene, loud))
    quiet_a = measurement(CODE_A, lone_echo(scene, quiet))
    emit("G-F6.a.scores", csv(window(ra, first, last)))
    emit("G-F6.a.at_quiet", ra.get(quiet, 0))
    emit("G-F6.a.loud_part_at_quiet", loud_a.get(quiet, 0))
    emit("G-F6.a.quiet_part_at_quiet", quiet_a.get(quiet, 0))
    emit("G-F6.a.loud_ghosts_one_step", csv([loud_a.get(loud - 1, 0), loud_a.get(loud + 1, 0)]))
    emit("G-F6.a.zero_at_echo", csv([k for k in range(first, last + 1) if k in steps and ra.get(k, 0) == 0]))
    emit("G-F6.a.score_without_echo", csv([k for k in range(first, last + 1) if k not in steps and ra.get(k, 0) != 0]))

    rb = measurement(CODE_B, scene)
    loud_b = measurement(CODE_B, lone_echo(scene, loud))
    emit("G-F7.b.scores", csv(window(rb, first, last)))
    emit("G-F7.b.at_quiet", rb.get(quiet, 0))
    emit("G-F7.b.loud_part_at_quiet", loud_b.get(quiet, 0))
    emit("G-F7.b.over_quiet", Fraction(rb.get(quiet, 0), quiet_a.get(quiet, 0)))

    total = add(ra, rb)
    emit("G-F8.sum.scores", csv(window(total, first, last)))
    emit("G-F8.sum.at_loud", total.get(loud, 0))
    emit("G-F8.sum.at_quiet", total.get(quiet, 0))
    emit("G-F8.sum.largest_ghost", largest_ghost(total, set(steps)))
    emit("G-F8.magnification", Fraction(total.get(quiet, 0), scene[quiet]))

    twice = add(measurement(CODE_A, scene), measurement(CODE_A, scene))
    emit("G-F9.a_twice.sends", 2)
    emit("G-F9.a_twice.scores", csv(window(twice, first, last)))
    emit("G-F9.a_twice.at_quiet", twice.get(quiet, 0))
    emit("G-F9.a_twice.at_one_step", csv([twice.get(loud - 1, 0), twice.get(loud + 1, 0)]))

    codes = list(product((1, -1), repeat=len(CODE_A)))
    times = 2 * len(CODE_A)
    giving_back = giving_back_complementary = complementary_pairs = 0
    for a in codes:
        for b in codes:
            s = add(measurement(a, scene), measurement(b, scene))
            back = all(v == times * (scene[k] if 0 <= k < len(scene) else 0) for k, v in s.items())
            comp = complementary(a, b)
            giving_back += back
            giving_back_complementary += back and comp
            complementary_pairs += comp
    emit("G-X2.pairs", len(codes) ** 2)
    emit("G-X2.giving_back", giving_back)
    emit("G-X2.giving_back_complementary", giving_back_complementary)
    emit("G-X2.complementary", complementary_pairs)


# -------------------------------------------------------------- cross-checks


def cross_checks(ra, rb, both, moved):
    n = len(CODE_A)
    # the chapter's equation 2 on the unit circle
    worst = 0.0
    for t in range(360):
        z = cmath.exp(2j * cmath.pi * t / 360)
        pa = sum(c * z ** (-i) for i, c in enumerate(CODE_A))
        pb = sum(c * z ** (-i) for i, c in enumerate(CODE_B))
        worst = max(worst, abs(abs(pa) ** 2 + abs(pb) ** 2 - 2 * n))
    check(f"|A(z)|^2 + |B(z)|^2 = {2 * n} on the unit circle (worst {worst:.1e})", worst < 1e-9)
    check("code B's ghosts are minus code A's", all(ra[k] == -rb[k] for k in ra if k != 0))
    check("the sum is 2N at shift 0 and zero elsewhere", both == {k: (2 * n if k == 0 else 0) for k in both})

    # exhaustive: complementary pairs by length, and the cancellation on
    # every scene of three steps with strengths -2..2, for each pair
    counts = []
    for length in range(1, 9):
        codes = list(product((1, -1), repeat=length))
        rows = {c: scores(c, c) for c in codes}
        pairs = [
            (a, b)
            for a in codes
            for b in codes
            if all(rows[a][k] + rows[b][k] == (2 * length if k == 0 else 0) for k in rows[a])
        ]
        counts.append(len(pairs))
        bad = 0
        for a, b in pairs:
            for scene in product(range(-2, 3), repeat=3):
                s = add(measurement(a, scene), measurement(b, scene))
                want = {d: 2 * length * h for d, h in enumerate(scene)}
                if any(v != want.get(k, 0) for k, v in s.items()):
                    bad += 1
        check(f"length {length}: {len(pairs)} pairs, every same-scene sum is the scene times {2 * length}", bad == 0)
    check(f"complementary pairs by length 1..8: {counts}", counts == [4, 8, 0, 32, 0, 0, 0, 192])

    # the changed scene, two ways it can change
    flipped = add(measurement(CODE_A, STILL), measurement(CODE_B, (-1,)))
    check("echo upside down the second time: the echo cancels, the ghosts double",
          window(flipped, -3, 3) == [-2, 0, 2, 0, 2, 0, -2])
    check("moved one step: neither echo reaches the full 8", (moved[0], moved[1]) == (3, 5))

    # noise is scored, not cancelled
    noise = (0, 0, 0, 0, 0, 1)
    noisy = add(measurement(CODE_A, STILL, noise), measurement(CODE_B, STILL))
    check("noise at the receiver leaves its own scores", ghosts(noisy, {0}) == [2, 3, 4, 5])

    # the doubling construction from one pulse gives the chapter's pair
    a, b = (1,), (1,)
    for _ in range(2):
        a, b = a + b, a + tuple(-x for x in b)
    check("doubling (1),(1) twice gives the chapter's pair", (a, b) == (CODE_A, CODE_B))

    # chapter 02's wrong-pair theorem, through its proof: for every ordered
    # pair of four-pulse patterns and every scene with an echo, the sum minus
    # 8 times the scene is the scene convolved with g = C_a + C_b - 8 delta,
    # and its last nonzero score is (last echo) x (last entry of g), so it is
    # zero exactly when g is: when the pair is complementary
    scenes = [s for s in product(range(-2, 3), repeat=3) if any(s)] + [LOUD_AND_QUIET]
    codes = list(product((1, -1), repeat=4))
    rows = {c: scores(c, c) for c in codes}
    bad = 0
    for a in codes:
        for b in codes:
            g = add(rows[a], rows[b], {0: -8})
            g = {k: v for k, v in g.items() if v}
            for scene in scenes:
                residual = add(measurement(a, scene), measurement(b, scene), {d: -8 * h for d, h in enumerate(scene)})
                residual = {k: v for k, v in residual.items() if v}
                conv = {}
                for d, h in enumerate(scene):
                    for k, v in g.items():
                        conv[d + k] = conv.get(d + k, 0) + h * v
                conv = {k: v for k, v in conv.items() if v}
                lead_scene = max(d for d, h in enumerate(scene) if h)
                lead = (lead_scene + max(g), scene[lead_scene] * g[max(g)]) if g else None
                got = (max(residual), residual[max(residual)]) if residual else None
                if residual != conv or got != lead or (not residual) != complementary(a, b):
                    bad += 1
    check(f"wrong pairs: residual = scene * g and its last score is the product, {len(codes) ** 2} pairs x {len(scenes)} scenes", bad == 0)

    # chapter 03's identity on every pair of equal-length up/down halves of 1
    # to 5 pulses, from the polynomial products: S(x+y) + S(x-y) = 2 (S(x) + S(y))
    bad = 0
    for length in range(1, 6):
        halves = list(product((1, -1), repeat=length))
        rows = {h: scores(h, h) for h in halves}
        for hx in halves:
            for hy in halves:
                left = add(scores(hx + hy, hx + hy), scores(hx + tuple(-v for v in hy), hx + tuple(-v for v in hy)))
                right = {k: 2 * v for k, v in add(rows[hx], rows[hy]).items()}
                if any(left.get(k, 0) != right.get(k, 0) for k in set(left) | set(right)):
                    bad += 1
    check("doubling identity on every pair of up/down halves of 1 to 5 pulses", bad == 0)

    # another implementation: numpy.correlate, "full" mode
    try:
        import numpy as np
    except ImportError:
        print("# skip numpy.correlate parity (numpy not installed)", file=sys.stderr)
        return
    for name, rec, code, row in [
        ("A", CODE_A, CODE_A, ra),
        ("B", CODE_B, CODE_B, rb),
        ("B moved", echo(CODE_B, MOVED), CODE_B, add(measurement(CODE_B, MOVED))),
    ]:
        full = np.correlate(np.array(rec), np.array(code), "full").tolist()
        first = -(len(code) - 1)
        check(f"numpy.correlate agrees for {name}", full == window(row, first, first + len(full) - 1))


def main():
    rows = facts()
    cross_checks(*rows)
    print(f"# {_CHECKS[0] - len(_FAILED)}/{_CHECKS[0]} cross-checks passed", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
