#!/usr/bin/env python3
"""Facts oracle for "A Resistor Network Is Secretly Answering a Probability Question".

Run from this folder (`./x resistor_random_walk facts-check` does, with numpy
through uv):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `resistor facts`
prints (src/facts.rs), in the rows of docs/fact-sheet.md: R-F (what the
opening shows and says), R-X (the edges of its always/never sentences), R-P
(the primary source's own worked examples).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (which
eliminates the current balance, and inverts I - Q of the walker's absorbing
chain):
  * every voltage by Kirchhoff's spanning-forest count: over the spanning
    forests of two trees, one holding the top and one the bottom, weighted
    by the product of their links' conductances, a junction's voltage is the
    weight of the forests that put it in the top's tree over the weight of
    them all (checked here against Cramer's rule on random networks);
  * every chance of the walk by Cramer's rule on its first-step equations,
    h = P h with h = 1 at the top and 0 at the bottom, each determinant a
    sum over permutations;
  * the determinant of the current balance by permutations too, checked to
    equal the forests' total weight, which is zero exactly when a junction
    is cut off from both ends;
  * the chance that the walk is still going after n steps, as the product
    of the step chances along the one route that stays in the middle
    (checked against the closed form 1/4**n);
  * for the opening network, the walk's chance once more as a sum over all
    routes (a geometric series), a seeded simulation, and numpy's float
    solve when numpy is installed.
"""
import random
import sys
from fractions import Fraction
from itertools import combinations, permutations, product

_EMITTED = {}
_FAILED = []
_CHECKS = [0]

TOP, LEFT, RIGHT, BOTTOM = 0, 1, 2, 3
NAMES = ("top", "left", "right", "bottom")
# (one junction, the other, how many identical wires side by side)
LINKS = ((TOP, LEFT, 2), (TOP, RIGHT, 1), (LEFT, RIGHT, 1), (LEFT, BOTTOM, 1), (RIGHT, BOTTOM, 2))
ROUTE = (LEFT, RIGHT, LEFT, RIGHT, TOP)
STILL_STEPS = 3


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


def flag(b):
    return "true" if b else "false"


class Net:
    """`size` junctions, links (a, b, conductance), the top and the bottom."""

    def __init__(self, size, links, top, bottom):
        self.size, self.top, self.bottom = size, top, bottom
        self.links = [(a, b, Fraction(c)) for a, b, c in links]
        self.middle = [x for x in range(size) if x not in (top, bottom)]

    def c(self, x, y):
        return sum((c for a, b, c in self.links if {a, b} == {x, y}), Fraction(0))

    def at(self, x):
        return sum((c for a, b, c in self.links if x in (a, b)), Fraction(0))


def det(m):
    """Leibniz: the sum over permutations of signed products."""
    n = len(m)
    total = Fraction(0)
    for perm in permutations(range(n)):
        inversions = sum(1 for i in range(n) for j in range(i + 1, n) if perm[i] > perm[j])
        term = Fraction(-1 if inversions % 2 else 1)
        for i in range(n):
            term *= m[i][perm[i]]
        total += term
    return total


def cramer(m, rhs):
    d = det(m)
    if d == 0:
        return None
    out = []
    for k in range(len(m)):
        mk = [row[:k] + [rhs[i]] + row[k + 1:] for i, row in enumerate(m)]
        out.append(det(mk) / d)
    return out


def components(size, links):
    parent = list(range(size))

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for a, b, _ in links:
        ra, rb = find(a), find(b)
        if ra == rb:
            return None  # a cycle
        parent[ra] = rb
    return find


def forests(net):
    """Every spanning forest of two trees, the top in one and the bottom in
    the other: (its weight, the junctions in the top's tree)."""
    out = []
    for chosen in combinations(net.links, net.size - 2):
        find = components(net.size, chosen)
        if find is None or find(net.top) == find(net.bottom):
            continue
        roots = {find(net.top), find(net.bottom)}
        if any(find(x) not in roots for x in range(net.size)):
            continue
        weight = Fraction(1)
        for _, _, c in chosen:
            weight *= c
        out.append((weight, {x for x in range(net.size) if find(x) == find(net.top)}))
    return out


def forest_voltages(net):
    """Voltages with the top at 1 and the bottom at 0, by the forest count;
    None when no such forest exists (a junction is cut off)."""
    fs = forests(net)
    total = sum((w for w, _ in fs), Fraction(0))
    if total == 0:
        return None
    return [sum((w for w, side in fs if x in side), Fraction(0)) / total for x in range(net.size)]


def balance(net):
    """The current balance over the middle junctions, as a matrix."""
    return [[net.at(x) if x == y else -net.c(x, y) for y in net.middle] for x in net.middle]


def cramer_walk(net, end):
    """The chance from every junction of reaching `end` first, by Cramer's
    rule on h(x) = sum over y of P(x, y) h(y)."""
    if any(net.at(x) == 0 for x in net.middle):
        return None
    m = [[(1 if x == y else 0) - net.c(x, y) / net.at(x) for y in net.middle] for x in net.middle]
    rhs = [net.c(x, end) / net.at(x) for x in net.middle]
    inside = cramer(m, rhs)
    if inside is None:
        return None
    out = [Fraction(1 if x == end else 0) for x in range(net.size)]
    for x, h in zip(net.middle, inside):
        out[x] = h
    return out


def far_ends(x):
    """One far end per identical wire at x, in link order."""
    out = []
    for a, b, k in LINKS:
        if x in (a, b):
            out.extend([b if x == a else a] * k)
    return out


def still_walking(net, start, steps):
    """The chance of having reached neither end after 1..steps steps, by
    the one route that stays in the middle (the Rust model instead pushes
    the whole distribution forward a step at a time). In the opening network
    each middle junction has exactly one middle neighbour, so a walk still
    going after n steps has gone back and forth along that one link, and its
    chance is the product of those n step chances."""
    across = {}
    for x in net.middle:
        inside = [y for y in net.middle if net.c(x, y) > 0]
        if len(inside) != 1:
            raise SystemExit(f"oracle: junction {x} has middle neighbours {inside}, not one")
        across[x] = inside[0]
    out, chance, x = [], Fraction(1), start
    for _ in range(steps):
        chance *= net.c(x, across[x]) / net.at(x)
        x = across[x]
        out.append(chance)
    return out


def opening():
    return Net(4, LINKS, TOP, BOTTOM)


def chapter_02(net, v, h):
    """Chapter 02 at the left junction: the drops along its four wires from
    the forest voltages, the walker's first step by counting wires, its
    first-step terms from the Cramer chances, the conductance weights, and
    the pair counted once (its own forests and Cramer solve)."""
    ends = far_ends(LEFT)
    drops = [v[y] - v[LEFT] for y in ends]
    emit("R-F4.drops_into", csv(drops))
    emit("R-F4.flow", ",".join("in" if d > 0 else "out" for d in drops))
    inflow = [d for d in drops if d > 0]
    outflow = [-d for d in drops if d < 0]
    emit("R-F4.in", csv(inflow))
    emit("R-F4.out", csv(outflow))
    emit("R-F4.in_total", sum(inflow))
    emit("R-F4.out_total", sum(outflow))
    emit("R-F4.net", sum(drops))
    for y in (TOP, RIGHT, BOTTOM):
        emit(f"R-F4.drop.{NAMES[y]}", abs(v[y] - v[LEFT]))
    emit("R-F4.coefficient", len(ends))

    up = ends.count(TOP)
    emit("R-F5.top_wires", up)
    emit("R-F5.to_top", Fraction(up, len(ends)))
    emit("R-F5.neighbours", len(set(ends)))
    emit("R-F5.trap", Fraction(1, len(set(ends))))
    terms = [Fraction(1, len(ends)) * h[y] for y in ends]
    emit("R-F5.terms", csv(terms))
    emit("R-F5.sum", sum(terms))

    links = [net.c(LEFT, y) for y in (TOP, RIGHT, BOTTOM)]
    emit("R-F6.left.conductances", csv(links))
    emit("R-F6.left.total", net.at(LEFT))
    weights = [c / net.at(LEFT) for c in links]
    emit("R-F6.left.weights", csv(weights))
    counted = [Fraction(ends.count(y), len(ends)) for y in (TOP, RIGHT, BOTTOM)]
    emit("R-F6.pair_adds", flag(counted == weights))
    # the general rule holds at middle junctions only: the top's own
    # weighted average is not its value
    emit("R-X5.top_average", sum(net.c(TOP, y) * v[y] for y in range(net.size)) / net.at(TOP))

    single = Net(4, [(a, b, 1 if (a, b) == (TOP, LEFT) else k) for a, b, k in LINKS], TOP, BOTTOM)
    emit("R-X4.single.to_top", single.c(LEFT, TOP) / single.at(LEFT))
    emit("R-X4.single.voltages", csv(forest_voltages(single)))
    emit("R-X4.single.chances", csv(cramer_walk(single, TOP)))


def facts():
    net = opening()
    v = forest_voltages(net)
    h = cramer_walk(net, TOP)
    hb = cramer_walk(net, BOTTOM)

    # 01_1: the network
    emit("R-F1.junctions", net.size)
    emit("R-F1.middle", len(net.middle))
    emit("R-F1.links", len(LINKS))
    emit("R-F1.wires", sum(k for _, _, k in LINKS))
    doubled = [k for _, _, k in LINKS if k > 1]
    emit("R-F1.doubled", len(doubled))
    assert len(set(doubled)) == 1
    emit("R-F1.side_by_side", doubled[0])
    counts = {len(far_ends(x)) for x in net.middle}
    assert len(counts) == 1
    emit("R-F1.wires_at_middle", counts.pop())
    emit("R-F1.top_volts", v[TOP])
    emit("R-F1.bottom_volts", v[BOTTOM])

    # 01_2, 01_3: the electrical picture
    emit("R-F2.voltages", csv(v))
    emit("R-F2.left", v[LEFT])
    emit("R-F2.right", v[RIGHT])
    for name, x in (("left", LEFT), ("right", RIGHT)):
        ends = [v[y] for y in far_ends(x)]
        emit(f"R-F2.{name}.far_ends", csv(ends))
        emit(f"R-F2.{name}.average", sum(ends) / len(ends))
    currents = [(v[a] - v[b]) * k for a, b, k in LINKS]
    emit("R-F2.currents", csv(currents))
    out_of = {x: sum(i for (a, b, _), i in zip(LINKS, currents) if a == x)
              - sum(i for (a, b, _), i in zip(LINKS, currents) if b == x) for x in range(4)}
    emit("R-F2.balance", flag(all(out_of[x] == 0 for x in net.middle)))

    # 01_4 to 01_7: the walk
    emit("R-F3.chances", csv(h))
    emit("R-F3.left", h[LEFT])
    emit("R-F3.right", h[RIGHT])
    emit("R-F3.top_chance", h[TOP])
    emit("R-F3.bottom_chance", h[BOTTOM])
    emit("R-F3.wire_chance", Fraction(1, len(far_ends(LEFT))))
    ends = [h[y] for y in far_ends(LEFT)]
    emit("R-F3.left.far_ends", csv(ends))
    emit("R-F3.left.average", sum(ends) / len(ends))
    emit("R-F3.stops", csv(h[x] + hb[x] for x in net.middle))
    emit("R-F3.still_walking", csv(still_walking(net, LEFT, STILL_STEPS)))
    steps_ok = all(net.c(a, b) > 0 for a, b in zip(ROUTE, ROUTE[1:]))
    ends_ok = ROUTE[-1] in (TOP, BOTTOM) and all(x in net.middle for x in ROUTE[:-1])
    assert steps_ok and ends_ok, "the drawn route is not a walk"
    emit("R-F3.route", ",".join(NAMES[x] for x in ROUTE))
    emit("R-F3.same", flag(h == v))

    chapter_02(net, v, h)

    # 01_7: one solution, and its edge
    d = det(balance(net))
    emit("R-X1.determinant", d)
    emit("R-X1.solutions", 1 if d != 0 else "many")
    # both ends at zero: Cramer's rule on the balance with nothing pushed in
    inside = cramer(balance(net), [Fraction(0)] * len(net.middle))
    zero_ends = [Fraction(0)] * net.size
    for x, value in zip(net.middle, inside):
        zero_ends[x] = value
    emit("R-X1.zero_ends", csv(zero_ends))
    island = Net(6, LINKS + ((4, 5, 1),), TOP, BOTTOM)
    reach = {TOP, BOTTOM}
    for _ in range(island.size):
        reach |= {b for a, b, _ in island.links if a in reach} | {a for a, b, _ in island.links if b in reach}
    emit("R-X2.island.cut_off", csv(x for x in island.middle if x not in reach))
    emit("R-X2.island.determinant", det(balance(island)))
    emit("R-X2.island.refused", flag(forest_voltages(island) is None and cramer_walk(island, TOP) is None))

    # the family: every network on four junctions, six links each absent,
    # one wire or two
    pairs = list(combinations(range(4), 2))
    tally = dict(networks=0, determined=0, cut_off=0, singular=0, agree=0, between_ends=0)
    for ks in product(range(3), repeat=len(pairs)):
        fam = Net(4, [(a, b, k) for (a, b), k in zip(pairs, ks) if k], TOP, BOTTOM)
        tally["networks"] += 1
        fv = forest_voltages(fam)
        if det(balance(fam)) == 0:
            tally["singular"] += 1
        if fv is None:
            tally["cut_off"] += 1
            continue
        tally["determined"] += 1
        fh = cramer_walk(fam, TOP)
        tally["agree"] += fh == fv
        tally["between_ends"] += all(0 <= fv[x] <= 1 for x in fam.middle)
    for key in ("networks", "determined", "cut_off", "singular", "agree", "between_ends"):
        emit(f"R-X3.family.{key}", tally[key])

    # the primary source's worked examples
    ds = Net(4, ((0, 2, 1), (0, 3, 1), (1, 2, 1), (1, 3, 2), (2, 3, 2)), 0, 1)
    dv = forest_voltages(ds)
    emit("R-P1.voltages", csv(dv[2:]))
    emit("R-P1.chances", csv(cramer_walk(ds, 0)[2:]))
    emit("R-P1.current_cd", (dv[2] - dv[3]) * 2)
    emit("R-P1.steps_from_c", csv(ds.c(2, y) / ds.at(2) for y in (0, 1, 3)))
    ln = Net(6, [(x, x + 1, 1) for x in range(5)], 5, 0)
    emit("R-P2.line5", csv(forest_voltages(ln)))
    return net, v, h


def cross_checks(net, v, h):
    # the forest count against Cramer's rule on the current balance, and the
    # balance's determinant against the forests' total weight, on random
    # networks of rational conductances
    rng = random.Random(20261019)
    bad_v = bad_det = 0
    for trial in range(300):
        size = rng.randint(3, 6)
        links = [(a, b, Fraction(rng.randint(1, 5), rng.randint(1, 3)))
                 for a, b in combinations(range(size), 2) if rng.random() < 0.55]
        t = Net(size, links, 0, size - 1)
        fs = forests(t)
        if det(balance(t)) != sum((w for w, _ in fs), Fraction(0)):
            bad_det += 1
        fv = forest_voltages(t)
        rhs = [t.c(x, t.top) for x in t.middle]
        cv = cramer(balance(t), rhs)
        if (fv is None) != (cv is None):
            bad_v += 1
        elif fv is not None and [fv[x] for x in t.middle] != cv:
            bad_v += 1
    check("the forest count equals Cramer's rule on the current balance, 300 random networks", bad_v == 0)
    check("the balance's determinant equals the forests' total weight, 300 random networks", bad_det == 0)

    # the opening's walk as a sum over every route: from the left, each
    # round trip left -> right -> left has chance 1/4 * 1/4, and each trip
    # ends at the top by the direct pair of wires (2/4) or by right then
    # top (1/4 * 1/4)
    p = lambda x, y: net.c(x, y) / net.at(x)
    trip = p(LEFT, RIGHT) * p(RIGHT, LEFT)
    exit_top = p(LEFT, TOP) + p(LEFT, RIGHT) * p(RIGHT, TOP)
    check("every route summed (a geometric series) gives the left's chance", exit_top / (1 - trip) == h[LEFT])
    check("each step from the left goes to the top with chance 1/2", p(LEFT, TOP) == Fraction(1, 2))
    # the still-walking chances against their closed form: each step that
    # stays in the middle has chance 1/4, so n of them have 1/4**n
    sw = still_walking(net, LEFT, 8)
    check("still walking after n = 1..8 steps is 1/4**n", sw == [Fraction(1, 4**n) for n in range(1, 9)])

    # chapter 02: the drops balance, and counting wires agrees with the
    # conductance weights the walk uses
    drops = [v[y] - v[LEFT] for y in far_ends(LEFT)]
    check("the four drops into the left junction add to zero", sum(drops) == 0)
    check("two of four wires up is the conductance weight 2/4", Fraction(far_ends(LEFT).count(TOP), 4) == p(LEFT, TOP))
    # the derivation's equation: 4v = 1 + 1 + 2/5 + 0 at v = the left voltage
    check("4 times the left voltage is the far ends' sum", 4 * v[LEFT] == sum(v[y] for y in far_ends(LEFT)))

    # the averaging rule, each middle junction against its neighbours
    for x in net.middle:
        avg = sum(net.c(x, y) * v[y] for y in range(4)) / net.at(x)
        check(f"the {NAMES[x]} junction's voltage is its conductance-weighted average", avg == v[x])

    # a seeded simulation of 200 000 walkers from the left (a float check)
    sim = random.Random(7)
    hits = 0
    walkers = 200_000
    for _ in range(walkers):
        x = LEFT
        while x not in (TOP, BOTTOM):
            x = sim.choice(far_ends(x))
        hits += x == TOP
    rate = hits / walkers
    check(f"simulated walkers reach the top {rate:.4f} of the time, within 0.005 of 3/5", abs(rate - 0.6) < 0.005)

    # another implementation: numpy's float solve of the current balance
    try:
        import numpy as np
    except ImportError:
        print("# skip numpy parity (numpy not installed)", file=sys.stderr)
        return
    a = np.array([[float(c) for c in row] for row in balance(net)])
    b = np.array([float(net.c(x, TOP)) for x in net.middle])
    got = np.linalg.solve(a, b)
    check("numpy.linalg.solve agrees with the forest count", np.allclose(got, [float(v[x]) for x in net.middle]))


def main():
    rows = facts()
    cross_checks(*rows)
    print(f"# {_CHECKS[0] - len(_FAILED)}/{_CHECKS[0]} cross-checks passed", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
