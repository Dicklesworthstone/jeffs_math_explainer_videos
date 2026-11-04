"""Independent arithmetic checks for the current two-scene opening draft."""

modulus = 12
start = 11
advance = 5
total = start + advance
position = total % modulus
assert position == 4
assert {x % modulus for x in (4, 16, 28, -8)} == {4}
assert {x % modulus for x in (5, 17)} == {5}
assert 4 - 4 == 0
assert 16 - 4 == 12 and 4 - 16 == -12
assert (28 - 16) + (16 - 4) == 28 - 4 == 24

print(f"clock.modulus\t{modulus}")
print(f"clock.start\t{start}")
print(f"clock.advance\t{advance}")
print(f"clock.sum\t{total}")
print(f"clock.position\t{position}")
for index, value in enumerate((4, 16, 28, -8)):
    print(f"lane.four.{index}\t{value}")
for index, value in enumerate((5, 17)):
    print(f"lane.five.{index}\t{value}")
for index, value in enumerate((0, 1, 2)):
    print(f"close.example.{index}\t{value}")
points = ((1, 1), (1, -1), (-1, -1), (-1, 1))
rotated = points
for _ in range(4):
    rotated = tuple((-y, x) for x, y in rotated)
assert rotated == points
print("square.rotation_order\t4")
def transform(point, turns, reflected):
    x, y = point
    if reflected:
        y = -y
    for _ in range(turns):
        x, y = -y, x
    return x, y
square_maps = {tuple(transform(point, turns, reflected) for point in points) for reflected in (False, True) for turns in range(4)}
assert len(square_maps) == 8
print(f"square.symmetry_count\t{len(square_maps)}")
turn = lambda point: (-point[1], point[0])
flip = lambda point: (point[0], -point[1])
left = flip(turn((1, 1)))
right = turn(flip((1, 1)))
assert left == (-1, -1) and right == (1, 1)
print(f"order.turn_then_flip.1.x\t{left[0]}")
print(f"order.turn_then_flip.1.y\t{left[1]}")
print(f"order.flip_then_turn.1.x\t{right[0]}")
print(f"order.flip_then_turn.1.y\t{right[1]}")

# ---- The rebuilt chapters 03, 06, 08, 09, 10, 13, 14, 16 and 17 ---------------
# Recomputed from first principles, never from the r^t s^f formula the Rust
# d4_group module uses: a symmetry is the tuple of its four corner images,
# products compose those maps (rightmost acts first), subgroups come from all
# 256 subsets, and the Hamming code comes from its parity-check rule.
from fractions import Fraction
from itertools import product


def emit(key, value):
    if isinstance(value, bool):
        value = "true" if value else "false"
    print(f"{key}\t{value}")


def joined(values):
    return ",".join(str(v) for v in values)


def as_map(f):
    return tuple(f(c) for c in points)


def compose(a, b):
    """a after b: first do b, then a."""
    image_of = dict(zip(points, a))
    return tuple(image_of[q] for q in b)


IDENT = as_map(lambda p: p)
TURN = as_map(turn)
FLIP = as_map(flip)


def power(g, n):
    out = IDENT
    for _ in range(n):
        out = compose(g, out)
    return out


# Rack order: rotations e, r, r^2, r^3, then r^t after the flip.
RACK = [power(TURN, t) for t in range(4)] + [compose(power(TURN, t), FLIP) for t in range(4)]
TEX = dict(zip(RACK, ["e", "r", "r^2", "r^3", "s", "rs", "r^2s", "r^3s"]))
assert len(TEX) == 8
e_, r_, r2_, s_ = RACK[0], RACK[1], RACK[2], RACK[4]


def inverse(g):
    return next(h for h in RACK if compose(g, h) == IDENT)


def names(moves):
    return joined(TEX[g] for g in RACK if g in moves)


# 03: closeness on the drawn line -4..6; fractions by cross-multiplication.
near = lambda a, b: abs(a - b) <= 1
near_one = [n for n in range(-4, 7) if near(n, 1)]
near_zero = [n for n in range(-4, 7) if near(n, 0)]
assert near_one == [0, 1, 2] and near_zero == [-1, 0, 1]
emit("close.near_one", joined(near_one))
emit("close.near_zero", joined(near_zero))
emit("close.shared", len(set(near_one) & set(near_zero)))
assert set(near_one) != set(near_zero)
emit("fraction.half_meets_two_quarters", Fraction(1, 2) == Fraction(2, 4) and 1 * 4 == 2 * 2)


# 06: clock moves by walking a hand round a twelve-mark dial one hour at a
# time and counting full laps (never a remainder); tile motions as corner maps
# performed in time order, where undoing a move is the map found to compose
# with it to the identity.
def walk(*advances):
    hand, laps = 0, 0
    for hours in advances:
        for _ in range(hours):
            hand += 1
            if hand == 12:
                hand, laps = 0, laps + 1
    return hand, laps


emit("axioms.closure.moves", joined((3, 4)))
emit("axioms.closure.result", walk(3, 4)[0])
emit("axioms.wrap.moves", joined((8, 7)))
emit("axioms.wrap.total", 8 + 7)
emit("axioms.wrap.position", walk(8, 7)[0])
assert walk(8, 7)[1] == 1, "the wrap example passes twelve"
emit("axioms.grouped.moves", joined((3, 4, 5)))
emit("axioms.grouped.first_two", walk(3, 4)[0])
emit("axioms.grouped.last_two", walk(4, 5)[0])
emit("axioms.grouped.total", 3 + 4 + 5)
emit("axioms.grouped.position", walk(walk(3, 4)[0], 5)[0])
emit("axioms.grouped.position_regrouped", walk(3, walk(4, 5)[0])[0])
emit("axioms.laps.moves", joined((8, 7, 9)))
emit("axioms.laps.total", 8 + 7 + 9)
emit("axioms.laps.full_laps", walk(8, 7, 9)[1])
emit("axioms.laps.first_two_dropped", walk(8, 7)[0])
emit("axioms.laps.then_last", walk(8, 7)[0] + 9)
emit("axioms.laps.position", walk(8, 7, 9)[0])
emit("axioms.identity.start", 3)
emit("axioms.identity.result", walk(3, 0)[0])
undo_three = [k for k in range(12) if walk(3, k)[0] == 0]
assert len(undo_three) == 1
emit("axioms.inverse.of_start", undo_three[0])
emit("axioms.inverse.result", walk(3, undo_three[0])[0])
# "Taking a remainder before or after adding gives the same clock position":
# every triple of moves, walked as one advance or dropped to the dial between.
for x, y, z in product(range(12), repeat=3):
    assert walk(x + y + z)[0] == walk(walk(x, y)[0], z)[0] == walk(x, walk(y, z)[0])[0]


def performed(*moves):
    """The one move equal to doing `moves` in time order."""
    out = IDENT
    for g in moves:
        out = compose(g, out)
    return out


a_, b_ = r_, s_  # 06_7 and 06_11: a is the quarter-turn, b the flip
assert compose(compose(s_, r_), s_) == compose(s_, compose(r_, s_))
emit("axioms.regroup.product", TEX[performed(s_, r_, s_)])
emit("axioms.swapped.product", TEX[performed(s_, s_, r_)])
emit("axioms.solve.a", TEX[a_])
emit("axioms.solve.b", TEX[b_])
emit("axioms.solve.x", TEX[performed(b_, inverse(a_))])
emit("axioms.solve.check", TEX[performed(b_, inverse(a_), a_)])
emit("axioms.wrong_side.candidate", TEX[performed(inverse(a_), b_)])
emit("axioms.wrong_side.check", TEX[performed(inverse(a_), b_, a_)])
emit("axioms.undo.a", TEX[a_])
emit("axioms.undo.b", TEX[b_])
emit("axioms.undo.reverse_order", TEX[performed(b_, a_, inverse(a_), inverse(b_))])
emit("axioms.undo.same_order", TEX[performed(b_, a_, inverse(b_), inverse(a_))])
# The deductions, edge-tested over the whole square group: one identity; one
# left and one right inverse, equal; cancellation on both sides; and the
# reverse-order rule for every pair.
assert [f for f in RACK if all(compose(f, g) == g == compose(g, f) for g in RACK)] == [e_]
for g in RACK:
    assert [h for h in RACK if compose(h, g) == e_] == [h for h in RACK if compose(g, h) == e_] == [inverse(g)]
    for h in RACK:
        assert inverse(compose(g, h)) == compose(inverse(h), inverse(g))
        for k in RACK:
            assert (compose(g, h) == compose(g, k)) == (h == k) == (compose(h, g) == compose(k, g))


# 08: closure-machine passes, from generators to a full pass with nothing new.
def closure_rounds(generators):
    have = list(dict.fromkeys(generators))
    rounds = [list(have)]
    while True:
        fresh = []
        for a in have:
            for b in have:
                c = compose(a, b)
                if c not in have and c not in fresh:
                    fresh.append(c)
        if not fresh:
            return rounds
        rounds.append(fresh)
        have += fresh


for name, generators in (("flip", [s_]), ("flip_half_turn", [s_, r2_]), ("quarter_turn_flip", [r_, s_])):
    rounds = closure_rounds(generators)
    emit(f"closure.{name}.new_per_pass", joined(len(fresh) for fresh in rounds[1:]))
    emit(f"closure.{name}.size", sum(len(x) for x in rounds))
found = []
for mask in range(1, 256):
    subset = {RACK[i] for i in range(8) if mask >> i & 1}
    if IDENT in subset and all(compose(a, b) in subset and inverse(a) in subset for a in subset for b in subset):
        found.append(subset)
emit("subgroups.count", len(found))
for size in range(1, 9):
    emit(f"subgroups.of_size.{size}", sum(len(h) == size for h in found))


# 09: commuting by corner maps; mirror lines from the linear extension.
def act(g, vec):
    """Extend the corner map linearly ((1,0) and (0,1) from two corners)."""
    image_of = dict(zip(points, g))
    a, b = image_of[(1, 1)], image_of[(1, -1)]
    e1 = (Fraction(a[0] + b[0], 2), Fraction(a[1] + b[1], 2))
    e2 = (Fraction(a[0] - b[0], 2), Fraction(a[1] - b[1], 2))
    return (vec[0] * e1[0] + vec[1] * e2[0], vec[0] * e1[1] + vec[1] * e2[1])


DIRECTIONS = {0: (1, 0), 1: (1, 1), 2: (0, 1), 3: (-1, 1)}


def line_of(vec):
    return next(k for k, (dx, dy) in DIRECTIONS.items() if vec[0] * dy - vec[1] * dx == 0)


centralizer = {g for g in RACK if compose(g, s_) == compose(s_, g)}
emit("centralizer.s.members", names(centralizer))
emit("centralizer.s.checks", len(centralizer))
emit("centralizer.s.crosses", 8 - len(centralizer))
for g in RACK:
    emit(f"mirror.carry.{TEX[g]}", line_of(act(g, DIRECTIONS[0])))
returned = {g for g in RACK if line_of(act(g, DIRECTIONS[0])) == 0}
assert returned == centralizer
emit("mirror.returned_by", names(returned))
center = {g for g in RACK if all(compose(g, h) == compose(h, g) for h in RACK)}
emit("center.members", names(center))

# 10: the twelve-clock by multiples (8m modulo 12 for m = 0, 1, ...), never
# by stepping a walk; orders from n / gcd (Euclid, not a divisor search); H by
# closing {8} under addition modulo 12; exponents as plain integers.
from math import gcd

n10, k10 = 12, 8
first_home = min(m for m in range(1, n10 + 1) if (k10 * m) % n10 == 0)
assert first_home == n10 // gcd(n10, k10)
emit("cyclic.walk", joined((k10 * m) % n10 for m in range(first_home)))
emit("cyclic.order", first_home)
emit("cyclic.table.totals", joined(k10 * m for m in range(1, first_home + 1)))
emit("cyclic.table.positions", joined((k10 * m) % n10 for m in range(1, first_home + 1)))
d10 = gcd(n10, k10)
emit("cyclic.gcd", d10)
starts = range(d10)  # the orbit of s is s + 8m; residues modulo d never meet
orbit_lists = [[(s + k10 * m) % n10 for m in range(first_home)] for s in starts]
assert sorted(sum(orbit_lists, [])) == list(range(n10))
emit("cyclic.orbits.count", len(orbit_lists))
emit("cyclic.orbits", ";".join(joined(o) for o in orbit_lists))
emit("cyclic.blocks", n10 // d10)
emit("cyclic.block_move", k10 // d10)
emit("cyclic.blocks.gcd", gcd(n10 // d10, k10 // d10))
emit("cyclic.blocks.visited", joined((k10 * m) % n10 // d10 for m in range(first_home)))
emit("cyclic.blocks.order", (n10 // d10) // gcd(n10 // d10, k10 // d10))
emit("cyclic.gcd_row", joined(gcd(k, n10) for k in range(1, n10)))
emit("cyclic.order_row", joined(n10 // gcd(k, n10) for k in range(1, n10)))
emit("cyclic.generators", joined(k for k in range(1, n10) if gcd(k, n10) == 1))
for step in (5, 7):
    emit(f"cyclic.tour.{step}", joined((step * m) % n10 for m in range(n10)))
h10 = {0}
while True:
    grown = h10 | {(a + b) % n10 for a in h10 | {k10} for b in h10 | {k10}}
    if grown == h10:
        break
    h10 = grown
emit("cyclic.h", joined(sorted(h10)))
line10 = range(-8, 13)
emit("cyclic.line.range", joined([line10[0], line10[-1]]))
emit("cyclic.line.members", joined(m for m in line10 if m % n10 in h10))
least10 = min(j for j in range(1, n10 + 1) if j % n10 in h10)
emit("cyclic.least_step", least10)
emit("cyclic.line.scanned_out", joined(j for j in range(1, least10) if j % n10 not in h10))
emit("cyclic.negative", -8)
emit("cyclic.negative.inverse_in_h", 8 % n10 in h10)
for name, m in (("member", 8), ("ghost", 10)):
    q, r = divmod(m, least10)
    emit(f"cyclic.{name}", m)
    if name == "ghost":
        emit("cyclic.ghost.in_h", m % n10 in h10)
        emit("cyclic.ghost.r_in_h", r % n10 in h10)
    emit(f"cyclic.{name}.q", q)
    emit(f"cyclic.{name}.r", r)
assert all(m % least10 == 0 for m in line10 if m % n10 in h10)
emit("cyclic.generated", joined((least10 * m) % n10 for m in range(n10 // least10)))

# 13_9 to 13_12: words as composed corner maps (written left to right, the
# rightmost acting first); machines and the number line by plain integers.
def word(*moves):
    out = IDENT
    for g in moves:
        out = compose(out, g)
    return out


assert compose(s_, r_) != compose(r_, s_)
emit("arithmetic.swap.ab", TEX[compose(s_, r_)])
emit("arithmetic.swap.ba", TEX[compose(r_, s_)])
emit("arithmetic.word", joined(TEX[g] for g in (s_, r_, s_)))
emit("arithmetic.word.left_packaged", TEX[compose(compose(s_, r_), s_)])
emit("arithmetic.word.right_packaged", TEX[compose(s_, compose(r_, s_))])
assert all(compose(compose(a, b), c) == compose(a, compose(b, c)) for a, b, c in product(RACK, repeat=3))
add_two, triple, less_one = (lambda x: x + 2), (lambda x: 3 * x), (lambda x: x - 1)
values = [1]
for machine in (add_two, triple, less_one):
    values.append(machine(values[-1]))
emit("arithmetic.machines.trail", joined(values))
emit("arithmetic.machines.first_two_packaged", less_one((lambda y: triple(add_two(y)))(1)))
emit("arithmetic.machines.last_two_packaged", (lambda y: less_one(triple(y)))(add_two(1)))
assert triple(add_two(1)) != add_two(triple(1))
drawn = range(-5, 6)
emit("arithmetic.line.range", joined([drawn[0], drawn[-1]]))
emit("arithmetic.line.start", 2)
emit("arithmetic.line.there", 2 + 3)
emit("arithmetic.line.back", 2 + 3 + (-3))
emit("arithmetic.line.missing_inverse", -3)
assert -3 in drawn and -3 < 1 and 0 < 1  # the inverse and zero lie outside 1, 2, 3, ...
emit("arithmetic.closure.product", TEX[compose(r_, s_)])
emit("arithmetic.long_word", joined(TEX[g] for g in (s_, r_, s_, r2_, s_)))
emit("arithmetic.long_word.first_three", TEX[word(s_, r_, s_)])
assert word(s_, r_, s_) == inverse(r_)
emit("arithmetic.long_word.product", TEX[word(s_, r_, s_, r2_, s_)])
emit("arithmetic.identity.left", TEX[compose(e_, r_)])
emit("arithmetic.identity.right", TEX[compose(r_, e_)])
solved = [g for g in RACK if compose(r_, g) == s_]
assert len(solved) == 1
emit("arithmetic.solve.x", TEX[solved[0]])
emit("arithmetic.solve.check", TEX[compose(r_, solved[0])])

# 14: clock representatives and the cosets of H = {e, s}.
clock_h = {0, 4, 8}
for name, (g, k) in (("same", (1, 9)), ("other", (1, 7))):
    same = {(g + h) % 12 for h in clock_h} == {(k + h) % 12 for h in clock_h}
    assert same == ((k - g) % 12 in clock_h)
    emit(f"coset.clock.{name}.step", (k - g) % 12)
    emit(f"coset.clock.{name}.in_h", same)
clock_cosets = {frozenset((n + h) % 12 for h in clock_h) for n in range(12)}
emit("coset.clock.count", len(clock_cosets))
emit("coset.clock.size", len(clock_h))
flip_h = {e_, s_}
left_cosets = {frozenset(compose(g, h) for h in flip_h) for g in RACK}
right_cosets = {frozenset(compose(h, g) for h in flip_h) for g in RACK}
emit("coset.flip.left_count", len(left_cosets))
emit("coset.flip.right_count", len(right_cosets))
emit("coset.flip.r_left", names({compose(r_, h) for h in flip_h}))
emit("coset.flip.r_right", names({compose(h, r_) for h in flip_h}))
emit("coset.flip.cuts_agree", left_cosets == right_cosets)

# 11: permutations from cycle notation, the square's corners from coordinates
# and adjacency, the witness from the definition of the product. Nothing here
# uses the Rust image arrays, its chip tables or d4.apply.
from itertools import combinations, permutations as arrangements_of


def perm_cycles(size, *cycles):
    image = {label: label for label in range(1, size + 1)}
    for cycle in cycles:
        for a, b in zip(cycle, cycle[1:] + cycle[:1]):
            image[a] = b
    return image


def perm_then(first, second):
    return {label: second[first[label]] for label in first}


def perm_trails(image):
    """Each label's walk until it first repeats, and the label it repeats."""
    walks = {}
    for start in image:
        seen, at = [], start
        while at not in seen:
            seen.append(at)
            at = image[at]
        walks[start] = (seen, at)
    return walks


def perm_swap_count(image):
    orbits = {frozenset(walk) for walk, _ in perm_trails(image).values()}
    return len(image) - len(orbits)


def perm_listed(image):
    return joined(image[label] for label in sorted(image))


def perm_swap(values, pair):
    values = list(values)
    values[pair[0]], values[pair[1]] = values[pair[1]], values[pair[0]]
    return tuple(values)


def perm_differences(values):
    return [values[j] - values[i] for i, j in combinations(range(len(values)), 2)]


def perm_delta(values):
    total = 1
    for difference in perm_differences(values):
        total *= difference
    return total


# 11_1: the drawn bad map, 1 and 2 into slot 1, 3 into slot 3.
perm_bad = {1: 1, 2: 1, 3: 3}
emit("permutations.bad_map_arrivals", joined(list(perm_bad.values()).count(s) for s in (1, 2, 3)))

# 11_2: a cycle has no first label; reversing usually changes it, but not a swap.
assert perm_cycles(4, (1, 3, 4)) == perm_cycles(4, (3, 4, 1)) != perm_cycles(4, (1, 4, 3))
for perm_m in range(2, 8):
    perm_loop = tuple(range(1, perm_m + 1))
    perm_same = perm_cycles(perm_m, perm_loop) == perm_cycles(perm_m, perm_loop[::-1])
    assert perm_same == (perm_m == 2)

# 11_3: the overlapping pair does not commute.
assert perm_then(perm_cycles(3, (1, 2)), perm_cycles(3, (1, 2, 3))) == perm_cycles(3, (1, 3))
assert perm_then(perm_cycles(3, (1, 2, 3)), perm_cycles(3, (1, 2))) == perm_cycles(3, (2, 3))

# 11_4: undo, disjoint loops, and the trail claim on every permutation up to
# six labels: each walk first repeats at its own start.
perm_forward = perm_cycles(4, (1, 3, 4))
perm_inverse = perm_cycles(4, (1, 4, 3))
assert perm_then(perm_forward, perm_inverse) == perm_cycles(4)
emit("permutations.inverse_images", perm_listed(perm_inverse))
perm_tri = perm_cycles(6, (1, 3, 5))
perm_pair = perm_cycles(6, (2, 6))
perm_both = perm_then(perm_tri, perm_pair)
assert perm_both == perm_then(perm_pair, perm_tri) == perm_cycles(6, (2, 6), (3, 5, 1))
assert perm_both[4] == 4
perm_row = {perm_both[card]: card for card in perm_both}
emit("permutations.disjoint_final", perm_listed(perm_row))
for perm_size in range(1, 7):
    for perm_word in arrangements_of(range(1, perm_size + 1)):
        perm_map = dict(zip(range(1, perm_size + 1), perm_word))
        assert all(repeat == start for start, (_, repeat) in perm_trails(perm_map).items())

# 11_5 and 11_6: a length-m cycle from m - 1 swaps, and two descriptions of one.
assert perm_then(perm_cycles(4, (1, 3)), perm_cycles(4, (1, 4))) == perm_forward
assert perm_then(perm_cycles(4, (1, 4)), perm_cycles(4, (3, 4))) == perm_forward
for perm_m in range(2, 8):
    perm_built = perm_cycles(perm_m)
    for perm_label in range(2, perm_m + 1):
        perm_built = perm_then(perm_built, perm_cycles(perm_m, (1, perm_label)))
    assert perm_built == perm_cycles(perm_m, tuple(range(1, perm_m + 1)))

# 11_7: the witness, and the proof's claims for every swap at sizes 2 to 6.
perm_witness = (1, 2, 4, 8)
perm_swapped = perm_swap(perm_witness, (0, 2))
perm_chips = perm_differences(perm_witness)
perm_after = perm_differences(perm_swapped)
emit("permutations.witness", joined(perm_witness))
emit("permutations.witness_swapped", joined(perm_swapped))
emit("permutations.chips_start", joined(perm_chips))
emit("permutations.chips_swapped", joined(perm_after))
assert len({abs(d) for d in perm_chips}) == len(perm_chips)
emit(
    "permutations.chip_destinations",
    joined(next(k + 1 for k, new in enumerate(perm_after) if abs(new) == abs(old)) for old in perm_chips),
)
emit("permutations.delta_start", perm_delta(perm_witness))
emit("permutations.delta_swapped", perm_delta(perm_swapped))
for perm_size in range(2, 7):
    perm_x = tuple(2**k for k in range(perm_size))
    perm_positions = list(combinations(range(perm_size), 2))
    for perm_a, perm_b in perm_positions:
        perm_y = perm_swap(perm_x, (perm_a, perm_b))
        assert perm_delta(perm_y) == -perm_delta(perm_x)
        perm_old = dict(zip(perm_positions, perm_differences(perm_x)))
        perm_new = dict(zip(perm_positions, perm_differences(perm_y)))
        assert perm_new[(perm_a, perm_b)] == -perm_old[(perm_a, perm_b)]
        for perm_pos in perm_positions:
            if not set(perm_pos) & {perm_a, perm_b}:
                assert perm_new[perm_pos] == perm_old[perm_pos]
        for perm_third in set(range(perm_size)) - {perm_a, perm_b}:
            perm_keys = [tuple(sorted((perm_third, s))) for s in (perm_a, perm_b)]
            perm_kept = perm_new[perm_keys[0]] * perm_new[perm_keys[1]]
            assert perm_kept == perm_old[perm_keys[0]] * perm_old[perm_keys[1]]
            # perm_x increases, so every old difference is positive: negative means reversed.
            perm_flips = sum(perm_new[key] < 0 for key in perm_keys)
            assert (perm_flips == 2) == (perm_a < perm_third < perm_b)

# 11_8: the two routes, and every swap word up to six on the witness.
perm_route = [perm_witness]
for perm_step in ((0, 1), (1, 2), (0, 1)):
    perm_route.append(perm_swap(perm_route[-1], perm_step))
assert perm_route[-1] == perm_swapped
emit("permutations.route_three_arrangements", ";".join(joined(x) for x in perm_route[1:]))
emit("permutations.route_three_deltas", joined(perm_delta(x) for x in perm_route))
perm_words = {perm_witness}
for perm_length in range(1, 7):
    perm_words = {perm_swap(x, step) for x in perm_words for step in combinations(range(4), 2)}
    assert {perm_delta(x) for x in perm_words} == {1008 * (-1) ** perm_length}

# 11_9: corners by coordinates; a symmetry is an arrangement keeping neighbours.
perm_corners = ((1, 1), (1, -1), (-1, -1), (-1, 1))
perm_quarter = {k + 1: perm_corners.index((-y, x)) + 1 for k, (x, y) in enumerate(perm_corners)}
perm_flip = {k + 1: perm_corners.index((x, -y)) + 1 for k, (x, y) in enumerate(perm_corners)}


def perm_neighbours(a, b):
    return sum(p != q for p, q in zip(perm_corners[a - 1], perm_corners[b - 1])) == 1


def perm_torn(place):
    """Which edges of the labelled square land on a diagonal, given each card's corner."""
    edges = ((1, 2), (2, 3), (3, 4), (4, 1))
    return [(a, b) for a, b in edges if not perm_neighbours(place[a], place[b])]


perm_all = list(arrangements_of(range(1, 5)))
perm_kept_cells = [
    "".join(map(str, cell)) for cell in perm_all if not perm_torn(dict(zip(range(1, 5), cell)))
]
emit("permutations.corner_count", len(perm_all))
emit("permutations.edge_keeping", len(perm_kept_cells))
emit("permutations.edge_keeping_arrangements", joined(perm_kept_cells))
emit(
    "permutations.trade_broken_edges",
    joined(f"{a}-{b}" for a, b in perm_torn(perm_cycles(4, (1, 2)))),
)
perm_half = perm_then(perm_quarter, perm_quarter)
perm_rs = perm_then(perm_flip, perm_quarter)
perm_sr = perm_then(perm_quarter, perm_flip)
assert perm_quarter == perm_cycles(4, (1, 4, 3, 2))
assert perm_flip == perm_cycles(4, (1, 2), (3, 4))
assert perm_half == perm_cycles(4, (1, 3), (2, 4))
assert perm_rs == perm_cycles(4, (2, 4)) and perm_sr == perm_cycles(4, (1, 3))
assert perm_then(perm_then(perm_cycles(4, (1, 4)), perm_cycles(4, (1, 3))), perm_cycles(4, (1, 2))) == perm_quarter
emit("permutations.quarter_images", perm_listed(perm_quarter))
emit("permutations.flip_images", perm_listed(perm_flip))
emit("permutations.half_images", perm_listed(perm_half))
emit("permutations.rs_images", perm_listed(perm_rs))
emit("permutations.sr_images", perm_listed(perm_sr))
emit("permutations.quarter_swaps", perm_swap_count(perm_quarter))
emit("permutations.half_swaps", perm_swap_count(perm_half))
# "All twenty-four permutations form a group": closed, with identity and inverses.
perm_group = [dict(zip(range(1, 5), cell)) for cell in perm_all]
assert all(perm_then(a, b) in perm_group for a in perm_group for b in perm_group)
assert all(any(perm_then(a, b) == perm_cycles(4) for b in perm_group) for a in perm_group)
# "Sit inside it as a subgroup": the kept eight are closed and hold the identity.
perm_kept_maps = [perm for perm in perm_group if not perm_torn(perm)]
assert all(perm_then(a, b) in perm_kept_maps for a in perm_kept_maps for b in perm_kept_maps)
assert perm_cycles(4) in perm_kept_maps

# 11_12 to 11_27: repeating a permutation, parity, and shuffles. Orders come
# from running the map, parity from counting inversions, and shuffles from
# dealing an actual list of cards, none of which the Rust side does.
from math import factorial, gcd


def perm_identity(image):
    return {label: label for label in image}


def perm_power(image, runs):
    power = perm_identity(image)
    for _ in range(runs):
        power = perm_then(power, image)
    return power


def perm_home(image, labels, runs):
    power = perm_power(image, runs)
    return all(power[label] == label for label in labels)


def perm_home_runs(image, labels, limit):
    return [runs for runs in range(1, limit + 1) if perm_home(image, labels, runs)]


def perm_order(image):
    runs, power = 1, dict(image)
    while power != perm_identity(image):
        runs, power = runs + 1, perm_then(power, image)
    return runs


def perm_orbits(image):
    """Disjoint cycles, each from its smallest label, by that label."""
    seen, orbits = set(), []
    for start in sorted(image):
        if start not in seen:
            orbit, at = [], start
            while at not in orbit:
                orbit.append(at)
                at = image[at]
            seen.update(orbit)
            orbits.append(orbit)
    return orbits


def perm_lengths(image):
    return sorted(len(orbit) for orbit in perm_orbits(image))


def perm_lcm(numbers):
    total = 1
    for number in numbers:
        total = total * number // gcd(total, number)
    return total


def perm_inversions(values):
    return sum(values[i] > values[j] for i, j in combinations(range(len(values)), 2))


def perm_sign(image):
    return 1 - 2 * (perm_inversions([image[label] for label in sorted(image)]) % 2)


def perm_inverse(image):
    return {slot: label for label, slot in image.items()}


def perm_row_after(size, word):
    """The row of cards after trading the contents of one-based slots, in order."""
    row = list(range(1, size + 1))
    for a, b in word:
        row[a - 1], row[b - 1] = row[b - 1], row[a - 1]
    return row


def perm_of_row(row):
    return {card: slot for slot, card in enumerate(row, 1)}


def perm_all_maps(size):
    return [dict(zip(range(1, size + 1), cell)) for cell in arrangements_of(range(1, size + 1))]


def perm_deal(row, keep_top):
    half = len(row) // 2
    first, second = (row[:half], row[half:]) if keep_top else (row[half:], row[:half])
    return [card for pair in zip(first, second) for card in pair]


def perm_shuffle(cards, keep_top):
    return perm_of_row(perm_deal(list(range(1, cards + 1)), keep_top))


def perm_written_cycles(image):
    return ";".join(" ".join(map(str, orbit)) for orbit in perm_orbits(image) if len(orbit) > 1)


perm_first = perm_cycles(6, [1, 3, 5], [2, 6])
perm_second = perm_cycles(6, [1, 2, 3, 4], [5, 6])
perm_overlap = perm_then(perm_cycles(3, [1, 2]), perm_cycles(3, [1, 2, 3]))
emit("permutations.loop_three_home", joined(perm_home_runs(perm_first, [1, 3, 5], 12)))
emit("permutations.loop_pair_home", joined(perm_home_runs(perm_first, [2, 6], 12)))
emit("permutations.loops_order", perm_order(perm_first))
emit("permutations.second_four_home", joined(perm_home_runs(perm_second, [1, 2, 3, 4], 12)))
emit("permutations.second_pair_home", joined(perm_home_runs(perm_second, [5, 6], 12)))
emit("permutations.second_order", perm_order(perm_second))
emit("permutations.overlap_images", perm_listed(perm_overlap))
emit("permutations.overlap_order", perm_order(perm_overlap))
# 11_13, run by run: two runs home the pair, three the triple, five neither.
assert perm_home(perm_first, [2, 6], 2) and not perm_home(perm_first, [1, 3, 5], 2)
assert perm_home(perm_first, [1, 3, 5], 3) and not perm_home(perm_first, [2, 6], 3)
assert perm_home(perm_first, [2, 6], 4) and not perm_home(perm_first, [1, 3, 5], 4)
assert not perm_home(perm_first, [2, 6], 5) and not perm_home(perm_first, [1, 3, 5], 5)
assert perm_first[4] == 4
# 11_16: eight is a later return of the second row, six is no return.
assert perm_home(perm_second, list(perm_second), 8) and not perm_home(perm_second, list(perm_second), 6)
assert 4 * 2 == 8 and 4 + 2 == 6 and perm_order(perm_second) == perm_lcm([4, 2]) == 4
# 11_17: overlap breaks the rule on the film's own pair, while the rule holds
# for the disjoint cycles of every permutation up to seven labels.
assert perm_overlap == perm_cycles(3, [1, 3]) and perm_lcm([3, 2]) == 6 and perm_order(perm_overlap) == 2
for perm_size in range(1, 8):
    for perm in perm_all_maps(perm_size):
        assert perm_order(perm) == perm_lcm(perm_lengths(perm))

perm_even_lists = [(1, 2), (3, 4), (1, 3), (2, 4)]
perm_short_route = [(1, 4), (2, 3)]
emit("permutations.even_lists_final", joined(perm_row_after(4, perm_even_lists)))
emit("permutations.short_route_final", joined(perm_row_after(4, perm_short_route)))
emit("permutations.one_more_final", joined(perm_row_after(4, perm_even_lists + [(1, 2)])))
emit("permutations.two_odd_final", joined(perm_row_after(4, [(1, 2), (2, 3)])))
assert perm_row_after(4, perm_even_lists) == perm_row_after(4, perm_short_route)
assert perm_of_row(perm_row_after(4, perm_short_route)) == perm_cycles(4, [1, 4], [2, 3])
assert perm_of_row(perm_row_after(4, [(1, 2), (2, 3)])) == perm_cycles(4, [1, 3, 2])
# 11_19: the signs multiply on every pair of five-label permutations.
perm_s5 = perm_all_maps(5)
assert all(perm_sign(perm_then(a, b)) == perm_sign(a) * perm_sign(b) for a in perm_s5 for b in perm_s5)
# 11_20: the evens pass the film's test, a b^-1 by undoing b then doing a, for
# three to six labels; the odds have no identity and are not closed. Running a
# swap word backwards is its inverse, on the words the picture uses.
for perm_size in range(3, 7):
    perm_evens = [perm for perm in perm_all_maps(perm_size) if perm_sign(perm) == 1]
    perm_odds = [perm for perm in perm_all_maps(perm_size) if perm_sign(perm) == -1]
    assert perm_identity(perm_evens[0]) in perm_evens and perm_identity(perm_odds[0]) not in perm_odds
    assert all(perm_then(perm_inverse(b), a) in perm_evens for a in perm_evens for b in perm_evens)
    assert any(perm_then(a, b) not in perm_odds for a in perm_odds for b in perm_odds)
perm_word_b = [(1, 3), (2, 4)]
assert perm_of_row(perm_row_after(4, perm_word_b[::-1])) == perm_inverse(perm_of_row(perm_row_after(4, perm_word_b)))
# 11_21: the first-two swap pairs evens with odds at every size from two up.
for perm_size in range(2, 7):
    perm_words = list(arrangements_of(range(1, perm_size + 1)))
    perm_even_words = [word for word in perm_words if perm_inversions(word) % 2 == 0]
    perm_paired = {(word[1], word[0]) + word[2:] for word in perm_even_words}
    assert perm_paired == {word for word in perm_words if perm_inversions(word) % 2 == 1}
    assert len(perm_even_words) * 2 == factorial(perm_size)
perm_even_cells = [
    word for word in arrangements_of(range(1, 5)) if perm_inversions(word) % 2 == 0
]
perm_cell_kinds = [perm_lengths(perm_of_row(list(word))) for word in perm_even_cells]
emit("permutations.even_on_four", len(perm_even_cells))
emit("permutations.odd_on_four", factorial(4) - len(perm_even_cells))
emit(
    "permutations.even_kinds",
    joined(perm_cell_kinds.count(kind) for kind in ([1, 1, 1, 1], [2, 2], [1, 3])),
)
emit(
    "permutations.even_partners",
    ",".join(
        "".join(map(str, word)) + ">" + "".join(map(str, (word[1], word[0]) + word[2:]))
        for word in perm_even_cells
    ),
)

perm_deck = perm_shuffle(10, True)
perm_out = perm_shuffle(52, True)
perm_in = perm_shuffle(52, False)
perm_wires = perm_shuffle(8, True)
# The same three maps by modular arithmetic, from the positions alone.
for perm_size in (8, 10, 52):
    assert perm_shuffle(perm_size, True) == {
        slot: (2 * (slot - 1)) % (perm_size - 1) + 1 if slot < perm_size else perm_size
        for slot in range(1, perm_size + 1)
    }
    assert perm_shuffle(perm_size, False) == {
        slot: (2 * slot) % (perm_size + 1) for slot in range(1, perm_size + 1)
    }
emit("permutations.deck_images", perm_listed(perm_deck))
emit("permutations.deck_cycles", perm_written_cycles(perm_deck))
emit("permutations.deck_order", perm_order(perm_deck))
perm_deck_two = perm_power(perm_deck, 2)
emit("permutations.deck_away_after_two", sum(perm_deck_two[card] != card for card in perm_deck_two))
# "Most of the deck is still out of place"; the swapped pair is home.
assert sum(perm_deck_two[card] != card for card in perm_deck_two) > 10 / 2
assert perm_deck_two[4] == 4 and perm_deck_two[7] == 7
emit("permutations.full_out_lengths", joined(perm_lengths(perm_out)))
emit("permutations.full_out_order", perm_order(perm_out))
emit("permutations.full_in_lengths", joined(perm_lengths(perm_in)))
emit("permutations.full_in_order", perm_order(perm_in))
perm_in_eight = perm_power(perm_in, perm_order(perm_out))
emit("permutations.full_in_home_after_eight", sum(perm_in_eight[card] == card for card in perm_in_eight))
perm_52_digits = str(factorial(52))
emit("permutations.full_arrangements_scientific", f"{perm_52_digits[0]},{len(perm_52_digits) - 1}")
# "Six and a half times as long", and the top card's place in each shuffle.
assert perm_order(perm_in) / perm_order(perm_out) == 6.5
assert perm_out[1] == 1 and perm_in[1] == 2
emit("permutations.wire_images", perm_listed(perm_wires))
emit("permutations.wire_cycles", perm_written_cycles(perm_wires))
emit("permutations.wire_order", perm_order(perm_wires))
assert perm_power(perm_wires, 4) == perm_wires


def perm_dealt_with_slip(cards, rounds, round_of_slip, slot):
    """Deal `rounds` out-shuffles; after round `round_of_slip`, slots trade."""
    row = list(range(1, cards + 1))
    for round_number in range(1, rounds + 1):
        row = perm_deal(row, True)
        if round_number == round_of_slip:
            row[slot - 1], row[slot] = row[slot], row[slot - 1]
    return row


# The slip of 11_27 happens after the first shuffle, in slots 21 and 22.
perm_slipped = perm_dealt_with_slip(52, 8, 1, 21)
perm_slipped_cards = [card for slot, card in enumerate(perm_slipped, 1) if card != slot]
emit("permutations.slip_cards", joined(sorted(perm_slipped_cards)))
emit("permutations.slip_parity", "odd" if perm_inversions(perm_slipped) % 2 else "even")
# "The deck cannot be home": any neighbouring slip, in any of the eight rounds.
for perm_round in range(1, 9):
    for perm_slot in range(1, 52):
        perm_row = perm_dealt_with_slip(52, 8, perm_round, perm_slot)
        assert perm_inversions(perm_row) % 2 == 1
        assert sum(card != slot for slot, card in enumerate(perm_row, 1)) == 2
emit("permutations.undo_by_more", perm_order(perm_out) - 1)
assert perm_then(perm_out, perm_power(perm_out, 7)) == perm_identity(perm_out)
emit("permutations.full_home_labels", len(perm_home_runs(perm_out, list(perm_out), 8)))

# 16: rH times rH, then the sorted 8 x 8 tables.
rh = [compose(r_, h) for h in flip_h]
products = [compose(a, b) for a in rh for b in rh]
emit("quotient.rh_products.count", len(products))
emit("quotient.rh_products.in_r2h", sum(p in {compose(r2_, h) for h in flip_h} for p in products))
emit("quotient.rh_products.in_h", sum(p in flip_h for p in products))
emit("quotient.table.entries", len(RACK) ** 2)
rotations = set(RACK[:4])


def mixed(blocks, class_of):
    return sum(len({class_of(compose(a, b)) for a in rows for b in cols}) > 1 for rows in blocks for cols in blocks)


by_rotation = [RACK[:4], RACK[4:]]
emit("quotient.rotations.solid_blocks", len(by_rotation) ** 2 - mixed(by_rotation, lambda g: g in rotations))
by_left_coset = [[g, compose(g, s_)] for g in RACK[:4]]
coset_of = {g: next(c for c in left_cosets if g in c) for g in RACK}
emit("quotient.flip_h.blocks", len(by_left_coset) ** 2)
emit("quotient.flip_h.mixed_blocks", mixed(by_left_coset, lambda g: coset_of[g]))


# 17: Hamming(7,4) from its parity checks; a word lists positions 1..7.
def label_of(word):
    out = 0
    for position, bit in enumerate(word, start=1):
        if bit:
            out ^= position
    return out


def flipped(word, position):
    return tuple(b ^ (i == position) for i, b in enumerate(word, start=1))


text = lambda word: "".join(map(str, word))
words = list(product((0, 1), repeat=7))
code = [w for w in words if label_of(w) == 0]
assert all(tuple(x ^ y for x, y in zip(a, b)) in code for a in code for b in code)
check_bits = [p for p in range(1, 8) if p & (p - 1) == 0]
by_label = {}
for w in words:
    by_label.setdefault(label_of(w), []).append(w)
sent = (0, 1, 1, 0, 0, 1, 1)
assert sent in code
situations = [sent] + [flipped(sent, p) for p in range(1, 8)]
emit("hamming.check_bits", len(check_bits))
emit("hamming.length", 7)
emit("hamming.data_bits", 7 - len(check_bits))
emit("hamming.words", len(words))
emit("hamming.codewords", len(code))
emit("hamming.cosets", len(by_label))
emit("hamming.coset_sizes", joined(sorted({len(v) for v in by_label.values()})))
emit("hamming.situations", len(situations))
emit("hamming.distinct_labels", len({label_of(w) for w in situations}))
emit("hamming.sent", text(sent))
received = flipped(sent, 5)
emit("hamming.single.position", 5)
emit("hamming.single.received", text(received))
emit("hamming.single.syndrome", format(label_of(received), "03b"))
emit("hamming.single.syndrome_value", label_of(received))
emit("hamming.single.repaired", text(flipped(received, label_of(received))))
garbled = flipped(flipped(sent, 2), 5)
decoded = flipped(garbled, label_of(garbled))
emit("hamming.double.positions", "2,5")
emit("hamming.double.received", text(garbled))
emit("hamming.double.syndrome", format(label_of(garbled), "03b"))
emit("hamming.double.syndrome_value", label_of(garbled))
emit("hamming.double.decoded", text(decoded))
emit("hamming.double.decoded_is_codeword", decoded in code)
emit("hamming.double.decoded_is_sent", decoded == sent)


# 17_16: a three-bit label read by place value; 17_18: the extended code
# (codeword plus an overall parity bit that makes the count of ones even),
# decoded for every single and double flip of every codeword.
def worth(label):
    return [int(digit) * 2 ** (2 - i) for i, digit in enumerate(format(label, "03b"))]


assert all(sum(worth(n)) == n for n in range(8))
emit("hamming.place_values", joined(2 ** (2 - i) for i in range(3)))
emit("hamming.single.worths", joined(worth(label_of(received))))
emit("hamming.double.worths", joined(worth(label_of(garbled))))


def flip8(word, position):
    return tuple(b ^ (i == position) for i, b in enumerate(word, start=1))


corrected = reported = 0
for c in code:
    c8 = c + (sum(c) % 2,)
    for a in range(1, 9):
        once = flip8(c8, a)
        if sum(once) % 2 == 1:
            label = label_of(once[:7])
            corrected += (once[:7] if label == 0 else flipped(once[:7], label)) == c
        for b in range(a + 1, 9):
            twice = flip8(once, b)
            reported += sum(twice) % 2 == 0 and label_of(twice[:7]) != 0
emit("hamming.extended.single_flips_corrected", corrected)
emit("hamming.extended.double_flips_reported", reported)
