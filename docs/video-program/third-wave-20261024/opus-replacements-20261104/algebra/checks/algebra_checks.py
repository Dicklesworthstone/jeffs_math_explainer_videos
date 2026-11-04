"""Independent checks for the LavenderPike algebra candidate (bead 87do.64).

Every claim voiced or drawn by the candidate chapters 08, 09, 03 (equivalence
additions), 14, 16 and 17 is recomputed here from first principles:

* the square's symmetries are built as maps on the four corner coordinates,
  composed as functions (rightmost acts first), never from the r^t s^f formula
  the Rust module uses;
* subgroups are found by brute force over all 256 subsets;
* the Hamming code is built from its parity-check rule, not from a table.

Run: python3 -I -B checks/algebra_checks.py  (prints key<TAB>value lines and
asserts every claim; exit status 0 means every assertion held).
"""

from fractions import Fraction
from itertools import product

CORNERS = ((1, 1), (1, -1), (-1, -1), (-1, 1))  # corner 1..4, as in d4.rs


def turn(p):
    return (-p[1], p[0])


def flip(p):
    return (p[0], -p[1])


def as_map(f):
    """A symmetry recorded as the tuple of corner images."""
    return tuple(f(c) for c in CORNERS)


def compose(a, b):
    """a after b (rightmost first): first do b, then a."""
    lookup_a = dict(zip(CORNERS, a))
    return tuple(lookup_a[q] for q in b)


IDENT = as_map(lambda p: p)
R = as_map(turn)
S = as_map(flip)


def power(g, n):
    out = IDENT
    for _ in range(n):
        out = compose(g, out)
    return out


NAMES = {}
for t in range(4):
    NAMES[power(R, t)] = ["e", "r", "r²", "r³"][t]
    NAMES[compose(power(R, t), S)] = ["s", "rs", "r²s", "r³s"][t]
assert len(NAMES) == 8
RACK = [power(R, t) for t in range(4)] + [compose(power(R, t), S) for t in range(4)]
BY_NAME = {NAMES[g]: g for g in RACK}


def inverse(g):
    return next(h for h in RACK if compose(g, h) == IDENT)


def name_set(gs):
    return [NAMES[g] for g in RACK if g in gs]


out = []


def emit(key, value):
    out.append(f"{key}\t{value}")


# ---- 08: closure failures, the closure machine, all subgroups --------------
r, r2, r3, s = BY_NAME["r"], BY_NAME["r²"], BY_NAME["r³"], BY_NAME["s"]
assert compose(r2, r) == r3 and compose(r, r2) == r3  # 08_3: quarter then half
assert r3 not in {IDENT, r, r2}


def closure_rounds(generators):
    """Rounds of the closure machine: combine every ordered pair, keep new."""
    have = []
    for g in generators:
        if g not in have:
            have.append(g)
    rounds = [list(have)]
    while True:
        new = []
        for a in have:
            for b in have:
                c = compose(a, b)
                if c not in have and c not in new:
                    new.append(c)
        if not new:
            return rounds
        rounds.append(new)
        have += new


def flat(rounds):
    return {g for rnd in rounds for g in rnd}


rounds_s = closure_rounds([s])
assert name_set(flat(rounds_s)) == ["e", "s"]
assert [name_set(x) for x in rounds_s] == [["s"], ["e"]]
rounds_s_r2 = closure_rounds([s, r2])
assert name_set(flat(rounds_s_r2)) == ["e", "r²", "s", "r²s"]
assert [name_set(x) for x in rounds_s_r2] == [["r²", "s"], ["e", "r²s"]]
rounds_r_s = closure_rounds([r, s])
assert len(flat(rounds_r_s)) == 8
emit("closure.s", ",".join(name_set(flat(rounds_s))))
emit("closure.s_r2", ",".join(name_set(flat(rounds_s_r2))))
emit("closure.r_s.size", len(flat(rounds_r_s)))
emit("closure.r_s.rounds", len(rounds_r_s))
emit("closure.r_s.round_sizes", ",".join(str(len(x)) for x in rounds_r_s))
# r²s is the flip across the vertical midline: (x, y) -> (-x, y).
assert BY_NAME["r²s"] == as_map(lambda p: (-p[0], p[1]))


def is_subgroup(subset):
    return bool(subset) and all(compose(a, inverse(b)) in subset for a in subset for b in subset)


def closed(subset):
    return bool(subset) and all(compose(a, b) in subset for a in subset for b in subset)


subgroups = []
for mask in range(1, 256):
    subset = {RACK[i] for i in range(8) if mask >> i & 1}
    full = IDENT in subset and closed(subset) and all(inverse(g) in subset for g in subset)
    # One-step test equals the full definition on every nonempty subset.
    assert is_subgroup(subset) == full
    # Finite subgroup test: closure alone suffices for a nonempty finite subset.
    assert closed(subset) == full
    if full:
        subgroups.append(subset)
sizes = sorted(len(h) for h in subgroups)
assert len(subgroups) == 10 and sizes == [1, 2, 2, 2, 2, 2, 4, 4, 4, 8]
assert all(8 % n == 0 for n in sizes)
emit("subgroups.count", len(subgroups))
emit("subgroups.sizes", ",".join(map(str, sizes)))
emit("subgroups.order2", sizes.count(2))
emit("subgroups.order4", sizes.count(4))
# The machine, fed every nonempty collection, produces exactly those ten.
results = set()
for mask in range(1, 256):
    results.add(frozenset(flat(closure_rounds([RACK[i] for i in range(8) if mask >> i & 1]))))
assert results == {frozenset(h) for h in subgroups}
# Test machine (08_5, 08_12, 08_13) on K = {e, r², s, r²s}.
K = flat(rounds_s_r2)
for h in K:
    assert compose(h, inverse(h)) == IDENT  # feed (h, h)
    assert compose(IDENT, inverse(h)) == inverse(h)  # feed (e, h)
    for a in K:
        assert compose(a, inverse(inverse(h))) == compose(a, h)  # feed (a, h^-1)
# The machine rejects {e, r, r²}: feed (e, r) gives r³.
assert compose(IDENT, inverse(r)) == r3
emit("test.reject.e_r", NAMES[compose(IDENT, inverse(r))])

# ---- 09: centralizer of s, mirror lines, center -----------------------------
centralizer = {g for g in RACK if compose(g, s) == compose(s, g)}
assert name_set(centralizer) == ["e", "r²", "s", "r²s"]
assert centralizer == K  # same subgroup the closure machine grew
emit("centralizer.s", ",".join(name_set(centralizer)))
# The opening experiment: turn-then-flip versus flip-then-turn, blue corner 1.
blue = CORNERS[0]
assert dict(zip(CORNERS, compose(s, r)))[blue] == (-1, -1)
assert dict(zip(CORNERS, compose(r, s)))[blue] == (1, 1)
# Each reflection fixes a line through the centre pointwise; record its angle
# in eighths of a turn (0 horizontal, 1 diagonal y=x, 2 vertical, 3 y=-x).
DIRS = {0: (1, 0), 1: (1, 1), 2: (0, 1), 3: (-1, 1)}


def act_vec(g, vec):
    """Extend the corner map linearly: corners span the plane."""
    x, y = vec
    # (1,0) = ((1,1)+(1,-1))/2 and (0,1) = ((1,1)-(1,-1))/2.
    img = dict(zip(CORNERS, g))
    a, b = img[(1, 1)], img[(1, -1)]
    e1 = (Fraction(a[0] + b[0], 2), Fraction(a[1] + b[1], 2))
    e2 = (Fraction(a[0] - b[0], 2), Fraction(a[1] - b[1], 2))
    return (x * e1[0] + y * e2[0], x * e1[1] + y * e2[1])


def line_of(vec):
    x, y = vec
    for k, (dx, dy) in DIRS.items():
        if x * dy - y * dx == 0:
            return k
    raise AssertionError(vec)


def mirror(g):
    fixed = [k for k, d in DIRS.items() if act_vec(g, d) == d]
    return fixed[0] if len(fixed) == 1 else None


mirrors = {NAMES[g]: mirror(g) for g in RACK}
assert mirrors == {"e": None, "r": None, "r²": None, "r³": None, "s": 0, "rs": 1, "r²s": 2, "r³s": 3}
for g in RACK:
    moved = line_of(act_vec(g, DIRS[0]))
    assert (moved == 0) == (g in centralizer)  # the mirror-line criterion
    conj = compose(compose(g, s), inverse(g))  # undo g, flip, redo g
    assert mirror(conj) == moved  # the flip across the moved line
emit("mirror.preserving", ",".join(NAMES[g] for g in RACK if line_of(act_vec(g, DIRS[0])) == 0))
center = {g for g in RACK if all(compose(g, h) == compose(h, g) for h in RACK)}
assert name_set(center) == ["e", "r²"]
all_lines = {g for g in RACK if all(line_of(act_vec(g, DIRS[k])) == k for k in range(4))}
assert all_lines == center
emit("center", ",".join(name_set(center)))
# Centralizer of every element is a subgroup (the theorem of 09_7).
for x in RACK:
    assert is_subgroup({g for g in RACK if compose(g, x) == compose(x, g)})

# ---- 03: equivalence classes ----------------------------------------------
close = lambda a, b: abs(a - b) <= 1
assert close(0, 1) and close(1, 2) and not close(0, 2)
near_one = [n for n in range(-5, 6) if close(n, 1)]
near_zero = [n for n in range(-5, 6) if close(n, 0)]
assert near_one == [0, 1, 2] and near_zero == [-1, 0, 1]
emit("close.near_one", ",".join(map(str, near_one)))
emit("close.near_zero", ",".join(map(str, near_zero)))
lane = [n for n in range(-12, 41) if (n - 4) % 12 == 0]
assert {4, 16, 28, -8} <= set(lane)
# Clock classes partition a window of integers: every integer in one lane.
window = range(-24, 48)
lanes = {k: [n for n in window if n % 12 == k] for k in range(12)}
assert sorted(n for k in lanes for n in lanes[k]) == list(window)
# Fractions: (a, b) ~ (c, d) iff a d = b c; classes lie on lines through 0.
pairs = [(a, b) for a in range(-4, 5) for b in range(-4, 5) if b != 0]
for (a, b), (c, d) in product(pairs, pairs):
    assert (a * d == b * c) == (Fraction(a, b) == Fraction(c, d))
half = [(a, b) for a, b in pairs if 2 * a == b]
assert half == [(-2, -4), (-1, -2), (1, 2), (2, 4)]
emit("fraction.half_class", ";".join(f"{a}/{b}" for a, b in half))
emit("fraction.classes_in_window", len({Fraction(a, b) for a, b in pairs}))

# ---- 14: cosets ---------------------------------------------------------------
H12 = {0, 4, 8}
cosets12 = {frozenset((g + h) % 12 for h in H12) for g in range(12)}
assert len(cosets12) == 4 and all(len(c) == 3 for c in cosets12)
assert {(4 + h) % 12 for h in H12} == H12
assert (9 - 1) % 12 in H12 and (7 - 1) % 12 not in H12
emit("clock.same_coset.1_9", (9 - 1) % 12)
emit("clock.other_coset.1_7", (7 - 1) % 12)
Hs = {IDENT, s}
left = [name_set({compose(g, h) for h in Hs}) for g in RACK[:4]]
right = [name_set({compose(h, g) for h in Hs}) for g in RACK[:4]]
assert left == [["e", "s"], ["r", "rs"], ["r²", "r²s"], ["r³", "r³s"]]
assert right == [["e", "s"], ["r", "r³s"], ["r²", "r²s"], ["r³", "rs"]]
assert compose(s, r) == BY_NAME["r³s"]
emit("coset.left.r", ",".join(left[1]))
emit("coset.right.r", ",".join(right[1]))
for g in RACK:  # the bijection h -> g h, and the coset criterion
    assert len({compose(g, h) for h in Hs}) == 2
    for k in RACK:
        same = {compose(g, h) for h in Hs} == {compose(k, h) for h in Hs}
        assert same == (compose(inverse(g), k) in Hs)
z12_orders = sorted({next(m for m in range(1, 13) if (m * k) % 12 == 0) for k in range(12)})
assert z12_orders == [1, 2, 3, 4, 6, 12] and all(12 % o == 0 for o in z12_orders)
emit("clock.element_orders", ",".join(map(str, z12_orders)))

# ---- 16: product grids and the collapsing table ------------------------------
def left_coset_index(g, subgroup):
    for i, rep in enumerate(RACK):
        if g in {compose(rep, h) for h in subgroup}:
            first = min(RACK.index(compose(rep, h)) for h in subgroup)
            return first
    raise AssertionError


rH = [r, compose(r, s)]
grid = [[name_set({compose(compose(a, b), h) for h in Hs}) for b in rH] for a in rH]
products = [[NAMES[compose(a, b)] for b in rH] for a in rH]
assert products == [["r²", "r²s"], ["s", "e"]]
assert grid == [[["r²", "r²s"], ["r²", "r²s"]], [["e", "s"], ["e", "s"]]]
emit("grid.rH_rH.products", ";".join(",".join(row) for row in products))
rotations = {power(r, t) for t in range(4)}
# Normal: every block of the 8x8 table (rows/cols by coset of R) is one colour.
for a_block in (rotations, set(RACK) - rotations):
    for b_block in (rotations, set(RACK) - rotations):
        colours = {compose(a, b) in rotations for a in a_block for b in b_block}
        assert len(colours) == 1
quotient = {
    (ra, rb): (ra == rb)  # rotation iff both or neither are rotations
    for ra in (True, False)
    for rb in (True, False)
}
for a in RACK:
    for b in RACK:
        assert (compose(a, b) in rotations) == quotient[(a in rotations, b in rotations)]
# Not normal: blocks by left cosets of {e, s} mix colours.
order_h = []
for g in RACK[:4]:
    order_h += [g, compose(g, s)]
assert [NAMES[g] for g in order_h] == ["e", "s", "r", "rs", "r²", "r²s", "r³", "r³s"]
coset_of = {g: left_coset_index(g, Hs) for g in RACK}
mixed = 0
for i in range(4):
    for j in range(4):
        block = {coset_of[compose(a, b)] for a in order_h[2 * i : 2 * i + 2] for b in order_h[2 * j : 2 * j + 2]}
        mixed += len(block) > 1
assert mixed == 8
emit("table.nonnormal.mixed_blocks", mixed)
emit("table.nonnormal.blocks", 16)
# Conjugating s by the quarter-turn moves its mirror (16_4 callback).
assert mirror(compose(compose(r, s), inverse(r))) == 2

# ---- 17: ISBN-10 (unchanged facts) and Hamming(7,4) ----------------------------
isbn = [0, 3, 0, 6, 4, 0, 6, 1, 5, 2]
weighted = sum((10 - i) * d for i, d in enumerate(isbn))
assert weighted == 132 == 12 * 11
emit("isbn.total", weighted)


def syndrome(word):
    """XOR of the (1-based) positions that hold a one."""
    acc = 0
    for position, bit in enumerate(word, start=1):
        if bit:
            acc ^= position
    return acc


words = list(product((0, 1), repeat=7))
code = [w for w in words if syndrome(w) == 0]
assert len(words) == 128 and len(code) == 16
xor = lambda a, b: tuple(x ^ y for x, y in zip(a, b))
assert all(xor(a, b) in code for a in code for b in code)  # a subgroup
cosets7 = {}
for w in words:
    cosets7.setdefault(syndrome(w), []).append(w)
assert len(cosets7) == 8 and all(len(v) == 16 for v in cosets7.values())
assert 128 // 16 == 8
unit = lambda j: tuple(1 if i == j else 0 for i in range(1, 8))
errors = [tuple([0] * 7)] + [unit(j) for j in range(1, 8)]
assert sorted(syndrome(e) for e in errors) == list(range(8))  # one per coset
sent = (0, 1, 1, 0, 0, 1, 1)
assert syndrome(sent) == 0
one_error = xor(sent, unit(5))
assert syndrome(one_error) == 5 and xor(one_error, unit(syndrome(one_error))) == sent
two_errors = xor(xor(sent, unit(2)), unit(5))
label = syndrome(two_errors)
miscorrected = xor(two_errors, unit(label))
assert label == 7 and miscorrected in code and miscorrected != sent
emit("hamming.words", len(words))
emit("hamming.codewords", len(code))
emit("hamming.cosets", len(cosets7))
emit("hamming.sent", "".join(map(str, sent)))
emit("hamming.one_error.position", 5)
emit("hamming.one_error.received", "".join(map(str, one_error)))
emit("hamming.one_error.syndrome", format(syndrome(one_error), "03b"))
emit("hamming.two_errors.received", "".join(map(str, two_errors)))
emit("hamming.two_errors.syndrome", format(label, "03b"))
emit("hamming.two_errors.decoded", "".join(map(str, miscorrected)))
# Every pair of distinct single errors is miscorrected to a third position.
for i in range(1, 8):
    for j in range(i + 1, 8):
        k = i ^ j
        assert k not in (0, i, j)

print("\n".join(out))
print(f"ok\t{len(out)} values; all assertions held")
