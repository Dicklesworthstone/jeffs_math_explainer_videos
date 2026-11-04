#!/usr/bin/env python3
"""Run anywhere. The arithmetic behind the patched pictures, from definitions, not from the film's helpers."""
from itertools import permutations
from math import gcd


def lcm(*xs):
    out = 1
    for x in xs:
        out = out * x // gcd(out, x)
    return out


def dest_from_deal(n, keep_top):
    """Deal two halves alternately into a new stack; return card position -> new position, 1-based."""
    top, bottom = list(range(1, n // 2 + 1)), list(range(n // 2 + 1, n + 1))
    first, second = (top, bottom) if keep_top else (bottom, top)
    stack = [c for pair in zip(first, second) for c in pair]
    return {c: i + 1 for i, c in enumerate(stack)}


def cycles(p):
    seen, out = set(), []
    for s in sorted(p):
        if s not in seen:
            c = [s]
            seen.add(s)
            while p[c[-1]] != s:
                c.append(p[c[-1]])
                seen.add(c[-1])
            out.append(c)
    return out


def compose(*ps):
    """Rightmost acts first; each p is a dict with missing keys fixed."""
    def run(x):
        for p in reversed(ps):
            x = p.get(x, x)
        return x
    return run


def cyc(*labels):
    return {a: labels[(i + 1) % len(labels)] for i, a in enumerate(labels)}


print("== 11_4: inverse and commuting")
f, g = cyc(1, 3, 4), cyc(1, 4, 3)
print("(1 3 4)^-1 = (1 4 3):", all(compose(g, f)(x) == x for x in range(1, 5)))
a, b = cyc(1, 3, 5), cyc(2, 6)
print("(1 3 5)(2 6) = (2 6)(1 3 5):", all(compose(a, b)(x) == compose(b, a)(x) for x in range(1, 7)))
print("(2 6)(3 5 1) is the same map:", all(compose(a, b)(x) == compose(cyc(2, 6), cyc(3, 5, 1))(x) for x in range(1, 7)), "| 4 fixed:", compose(a, b)(4) == 4)
slot = {c: c for c in range(1, 5)}
after = {c: f.get(slot[c], slot[c]) for c in slot}
print("Frame 0186, cards after the forward run: card -> slot", after, "| the frame shows card 1 in slot 3, 3 in 4, 4 in 1, 2 in 2")

print("\n== 11_5: the swap shown is the example's first swap")
print("(1 3 4) = (1 4)(1 3):", all(compose(cyc(1, 4), cyc(1, 3))(x) == f.get(x, x) for x in range(1, 5)))
print("(1 3 4) = (3 4)(1 4):", all(compose(cyc(3, 4), cyc(1, 4))(x) == f.get(x, x) for x in range(1, 5)))
for m in range(2, 9):
    full = cyc(*range(1, m + 1))
    swaps = [cyc(1, k) for k in range(m, 1, -1)]
    print(f"  length {m}: {m - 1} swaps rebuild it:", all(compose(*swaps)(x) == full.get(x, x) for x in range(1, m + 1)))

print("\n== 11_20: the patched trades compose and undo")
row = list(range(1, 5))
for trade in [(0, 1), (1, 2)]:
    row[trade[0]], row[trade[1]] = row[trade[1]], row[trade[0]]
print("after two trades:", row, "(a three-cycle, so two swaps compose into something new)")
for trade in [(1, 2), (0, 1)]:
    row[trade[0]], row[trade[1]] = row[trade[1]], row[trade[0]]
print("after undoing them in reverse:", row, "| home:", row == [1, 2, 3, 4])

print("\n== 11_21: the key against the cells")


def sign(p):
    return sum(p[i] > p[j] for i in range(4) for j in range(i + 1, 4)) % 2


even = [p for p in permutations(range(4)) if sign(p) == 0]
print("even", len(even), "odd", 24 - len(even), "| swapping the first two sends every even cell to an odd cell:", all(sign((p[1], p[0], p[2], p[3])) == 1 for p in even))
kinds = {0: 0, 3: 0, 4: 0}
for p in even:
    kinds[sum(p[i] != i for i in range(4))] += 1
print("moved 0:", kinds[0], "moved 4:", kinds[4], "moved 3:", kinds[3])
print(f"Sentence 5 repaints {kinds[0]} green and {kinds[4]} hero. Gold cells left: {kinds[3]} of {len(even)}. 'Gold: even' then covers two thirds of the even cells.")

print("\n== 11_22: ten cards")
p = dest_from_deal(10, True)
print("deal:", p)
moving = [c for c in cycles(p) if len(c) > 1]
print("moving cycles:", moving, "| fixed:", [c[0] for c in cycles(p) if len(c) == 1], "| lcm:", lcm(*[len(c) for c in moving]))
at_slot = {p[c]: c for c in p}
print("card now in each slot after one shuffle:", [at_slot[s] for s in range(1, 11)], "| frame 1276 reads 1 6 2 7 3 8 4 9 5 10")
print("every arrow from slot N lands on card N:", all(at_slot[p[n]] == n for n in p))
print("ring at rest shows a different card number from its slot, per slot of the six-cycle:", {s: at_slot[s] for s in moving[0]})
print("'Card 2 went from slot 2 to slot 3':", p[2] == 3)

print("\n== 11_24 to 11_26: fifty-two cards and eight wires")
out, inn = dest_from_deal(52, True), dest_from_deal(52, False)
lengths = sorted(len(c) for c in cycles(out))
print("out-shuffle lengths:", {x: lengths.count(x) for x in sorted(set(lengths))}, "| lcm(1,2,8) =", lcm(1, 2, 8), "| swap:", [c for c in cycles(out) if len(c) == 2], "| fixed:", [c[0] for c in cycles(out) if len(c) == 1])
print("'By size' is a statement about the strip's order: cycle lengths sorted, true positions interleaved:", lengths[:3], "first long cycle:", [c for c in cycles(out) if len(c) == 8][0])
print("in-shuffle lengths:", sorted(len(c) for c in cycles(inn)), "| 52 / 8 =", 52 / 8)
print("out-shuffle keeps the top card on top:", out[1] == 1, "| in-shuffle sends it to second place:", inn[1] == 2)
state = {x: x for x in inn}
for _ in range(8):
    state = {x: inn[state[x]] for x in state}
print("after 8 in-shuffles, cards home:", sum(state[x] == x for x in state), "(the patch only delays this caption; the number comes from out_of_place)")
w = dest_from_deal(8, True)
print("eight wires:", [c for c in cycles(w)], "| order", lcm(*[len(c) for c in cycles(w)]))
state = {x: x for x in w}
after = {}
for stage in range(1, 5):
    state = {x: w[state[x]] for x in state}
    after[stage] = dict(state)
    print(f"  stage {stage}: every token home: {all(state[x] == x for x in state)}")
print("stage 4 leaves every token where stage 1 did:", after[4] == after[1])
