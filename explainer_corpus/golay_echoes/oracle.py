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
    return ra, rb, both, moved


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
