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

# ---- The rebuilt chapters 03, 06, 08, 09, 14, 16 and 17 -----------------------
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
