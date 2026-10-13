#!/usr/bin/env python3
"""Facts oracle for Video E, "How to Make a Fair Coin Out of an Unfair One".

Run from this folder (`./x fair_coin facts-check` does, with numpy through uv):

    uv run --no-project --with numpy python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `coin facts` prints
(src/facts.rs), in the rows of docs/fact-sheet.md: E-F (computed), E-X (the
edges of the always/never sentences), E-S (seeded runs) and E-C (cited).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model:
  * chances and costs with `fractions.Fraction` (exact), each also checked by
    listing every outcome where that is possible;
  * the coin with memory by solving its absorbing chains exactly; Blum's two
    algorithms by an exact linear solve over every reachable memory state
    (the Rust side follows histories forward in floating point instead);
  * rates by the papers' formulas (Peres 1992, Proposition 2; Elias 1972),
    and the procedures themselves written a second way (iteratively here);
  * the seeded runs replayed with NumPy's own PCG64DXSM;
  * CITED literals from the sources listed in docs/fact-sheet.md.
"""
import math
import sys
from fractions import Fraction as F
from itertools import combinations, permutations, product

import numpy as np

_EMITTED = {}
_FAILED = []
_CHECKS = [0]


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


def dec(x, places):
    """The decimal with `places` digits, rounded half away from zero, in integers."""
    x = F(x)
    scaled = abs(x) * 10**places
    q = (2 * scaled.numerator + scaled.denominator) // (2 * scaled.denominator)
    sign = "-" if x < 0 and q != 0 else ""
    if places == 0:
        return f"{sign}{q}"
    return f"{sign}{q // 10**places}.{q % 10**places:0{places}d}"


def exact(x):
    x = F(x)
    return str(x.numerator) if x.denominator == 1 else f"{x.numerator}/{x.denominator}"


def pct(x, places=0):
    return dec(F(x) * 100, places)


P = F(4, 5)  # the video's coin
Q_ = 1 - P
STOPS = [F(1, 2), F(3, 5), F(7, 10), F(4, 5), F(9, 10), F(99, 100)]
LOADED = [1, 1, 2, 2, 4, 10]  # twentieths
SEED = 1951


def stop_name(p):
    return f"p{pct(p)}"


# --------------------------------------------------------------------------
# E-F1 ... E-F4: two tosses of one coin
# --------------------------------------------------------------------------
def pair_chances(first, after_h, after_t):
    """{'HH': ..., 'HT': ..., 'TH': ..., 'TT': ...} for two tosses in a row."""
    out = {}
    for a, b in product("HT", repeat=2):
        pa = first if a == "H" else 1 - first
        second = after_h if a == "H" else after_t
        out[a + b] = pa * (second if b == "H" else 1 - second)
    return out


def coin_facts():
    sq = pair_chances(P, P, P)
    check("the four areas add to one", sum(sq.values()) == 1)
    for k in ("HH", "HT", "TH", "TT"):
        emit(f"E-F1.square.{k.lower()}", dec(sq[k], 2))
        emit(f"E-F1.square.{k.lower()}.exact", exact(sq[k]))
    emit("E-F1.heads_percent", pct(P))
    emit("E-F1.tails_percent", pct(Q_))
    decides = sq["HT"] + sq["TH"]
    emit("E-F2.decides", dec(decides, 2))
    emit("E-F2.decides_percent", pct(decides))
    emit("E-F2.thrown_away", dec(1 - decides, 2))
    emit("E-F2.thrown_away_percent", pct(1 - decides))
    emit("E-F2.heads_first_share.exact", exact(sq["HT"] / decides))
    emit("E-F2.heads_first_share_percent", pct(sq["HT"] / decides))
    # the cost: a geometric number of pairs, two tosses each
    pairs = 1 / decides
    emit("E-F3.expected_pairs", dec(pairs, 3))
    emit("E-F3.expected_tosses", dec(2 * pairs, 2))
    emit("E-F3.expected_tosses.exact", exact(2 * pairs))
    partial = sum(2 * r * decides * (1 - decides) ** (r - 1) for r in range(1, 400))
    check("the expected tosses are the sum of the geometric series", abs(float(partial) - float(2 * pairs)) < 1e-12)
    for r in (1, 2, 3, 5, 10):
        emit(f"E-F3.undecided_after.{r}_pairs_percent", pct((1 - decides) ** r, 1))
    for p in STOPS:
        s = pair_chances(p, p, p)
        check(f"{stop_name(p)}: heads-tails equals tails-heads", s["HT"] == s["TH"])
        emit(f"E-F4.{stop_name(p)}.ht", dec(s["HT"], 4))
        emit(f"E-F4.{stop_name(p)}.decides_percent", pct(s["HT"] + s["TH"], 1))
        emit(f"E-F4.{stop_name(p)}.expected_tosses", dec(1 / (p * (1 - p)), 2))
    emit("E-F4.p99.expected_tosses_rounded", dec(1 / (F(99, 100) * F(1, 100)), 0))
    emit("E-F4.p50.expected_tosses.exact", exact(1 / (F(1, 2) * F(1, 2))))


# --------------------------------------------------------------------------
# E-F5, E-F6: where it breaks
# --------------------------------------------------------------------------
def solve(a, b):
    """Exact Gauss-Jordan elimination on Fractions; None when singular."""
    n = len(b)
    a = [row[:] + [b[i]] for i, row in enumerate(a)]
    for col in range(n):
        pivot = next((r for r in range(col, n) if a[r][col] != 0), None)
        if pivot is None:
            return None
        a[col], a[pivot] = a[pivot], a[col]
        lead = a[col][col]
        a[col] = [v / lead for v in a[col]]
        for r in range(n):
            if r != col and a[r][col] != 0:
                f = a[r][col]
                a[r] = [v - f * w for v, w in zip(a[r], a[col])]
    return [a[i][n] for i in range(n)]


def plain_rule(after_h, after_t):
    """P(heads-first wins | the toss before the first pair), by the absorbing chain."""
    h, t = pair_chances(after_h, after_h, after_t), pair_chances(after_t, after_h, after_t)
    x = solve([[1 - h["HH"], -h["TT"]], [-t["HH"], 1 - t["TT"]]], [h["HT"], t["HT"]])
    return None if x is None else {"H": x[0], "T": x[1]}


def after_only_rule(after_h, after_t, after):
    """Samuelson's rule on the tosses that follow `after`: P(heads then tails decides first)."""
    heads = {"H": after_h, "T": after_t}
    states = [(cur, op) for cur in "HT" for op in (None, "H", "T")]
    idx = {s: i for i, s in enumerate(states)}
    a = [[F(0)] * 6 for _ in range(6)]
    b = [F(0)] * 6
    for cur, op in states:
        r = idx[(cur, op)]
        a[r][r] += 1
        for x in "HT":
            px = heads[cur] if x == "H" else 1 - heads[cur]
            if cur != after:
                a[r][idx[(x, op)]] -= px
            elif op is None:
                a[r][idx[(x, x)]] -= px
            elif op == x:
                a[r][idx[(x, None)]] -= px
            elif op == "H":
                b[r] += px
    sol = solve(a, b)
    return None if sol is None else {"H": sol[idx[("H", None)]], "T": sol[idx[("T", None)]]}


def blum_first_two(stay, on_return):
    """The chances that the first two results Blum's algorithm announces are HH, HT, TH, TT,
    on his Theorem A chain with two states (exit bit 0 = stay, with chance `stay`; exit bit 1
    = go to the other state). Exact: one linear system over every reachable memory state."""
    start = (0, "-", "-", None)
    seen, order, edges = {start: 0}, [start], {}
    absorb = {}
    i = 0
    while i < len(order):
        state = order[i]
        cur, s0, s1, first = state
        slots = [s0, s1]
        pre = []
        if slots[cur] in ("WH", "WT"):  # algorithm B's step 1
            pre.append(slots[cur][1])
            slots[cur] = "-"
        for bit, p in ((0, stay), (1, 1 - stay)):
            sl, out = slots[:], pre[:]
            if sl[cur] == "-":
                sl[cur] = str(bit)
            elif sl[cur] == str(bit):
                sl[cur] = "-"
            else:
                result = "H" if sl[cur] == "1" else "T"  # exits 1 then 0: H; 0 then 1: T
                if on_return:
                    sl[cur] = "W" + result
                else:
                    out.append(result)
                    sl[cur] = "-"
            f, done = first, None
            for r in out:
                if f is None:
                    f = r
                else:
                    done = f + r
                    break
            to = 1 - cur if bit else cur
            if done:
                absorb.setdefault(i, {}).setdefault(done, F(0))
                absorb[i][done] += p
            else:
                nxt = (to, sl[0], sl[1], f)
                if nxt not in seen:
                    seen[nxt] = len(order)
                    order.append(nxt)
                edges.setdefault(i, {}).setdefault(seen[nxt], F(0))
                edges[i][seen[nxt]] += p
        i += 1
    n = len(order)
    result = {}
    for target in ("HH", "HT", "TH", "TT"):
        a = [[F(0)] * n for _ in range(n)]
        b = [F(0)] * n
        for r in range(n):
            a[r][r] += 1
            for c, p in edges.get(r, {}).items():
                a[r][c] -= p
            b[r] = absorb.get(r, {}).get(target, F(0))
        result[target] = solve(a, b)[0]
    return result, n


def blum_first_n(k, eps, on_return, n, tol=1e-13):
    """The chances of the first n announced results on Blum's ring of k states (exit 0 stays, with
    chance 1 - eps; exit 1 moves to the next state), following every history until `tol` is left."""
    live = {(0, ("-",) * k, ""): 1.0}
    done = {}
    left = 1.0
    while left > tol:
        nxt = {}
        for (cur, slots, said), mass in live.items():
            slots, pre = list(slots), ""
            if slots[cur] in ("WH", "WT"):
                pre, slots[cur] = slots[cur][1], "-"
            for bit, p in ((0, 1 - eps), (1, eps)):
                sl, out = slots[:], pre
                if sl[cur] == "-":
                    sl[cur] = str(bit)
                elif sl[cur] == str(bit):
                    sl[cur] = "-"
                else:
                    result = "H" if sl[cur] == "1" else "T"
                    if on_return:
                        sl[cur] = "W" + result
                    else:
                        out, sl[cur] = out + result, "-"
                seq = said + out
                if len(seq) >= n:
                    done[seq[:n]] = done.get(seq[:n], 0.0) + mass * p
                else:
                    key = ((cur + 1) % k if bit else cur, tuple(sl), seq)
                    nxt[key] = nxt.get(key, 0.0) + mass * p
        live, left = nxt, sum(nxt.values())
    return done


def breaks_facts():
    two = pair_chances(P, F(1, 2), F(1, 2))
    emit("E-F5.two_coins.second_percent", pct(F(1, 2)))
    emit("E-F5.two_coins.ht", dec(two["HT"], 2))
    emit("E-F5.two_coins.th", dec(two["TH"], 2))
    emit("E-F5.two_coins.heads_first_percent", pct(two["HT"] / (two["HT"] + two["TH"])))
    emit("E-F5.two_coins.ratio", exact(two["HT"] / two["TH"]))
    # a coin that repeats its last face four times in five
    s = P
    after_hh = pair_chances(s, s, 1 - s)  # the toss before was a head
    after_tt = pair_chances(1 - s, s, 1 - s)
    emit("E-F6.sticky.repeat_percent", pct(s))
    emit("E-F6.sticky.after_heads.ht", dec(after_hh["HT"], 2))
    emit("E-F6.sticky.after_heads.th", dec(after_hh["TH"], 2))
    emit("E-F6.sticky.after_heads.ratio", exact(after_hh["HT"] / after_hh["TH"]))
    emit("E-F6.sticky.after_heads.heads_first_percent", pct(after_hh["HT"] / (after_hh["HT"] + after_hh["TH"])))
    emit("E-F6.sticky.after_tails.heads_first_percent", pct(after_tt["HT"] / (after_tt["HT"] + after_tt["TH"])))
    # the long-run share of heads solves h = h*after_h + (1 - h)*after_t
    long_h = (1 - s) / ((1 - s) + (1 - s))
    first = pair_chances(long_h, s, 1 - s)
    check("sticky: the first pair of a long run is fair", first["HT"] == first["TH"])
    emit("E-F6.sticky.long_run_first_pair.heads_first.exact", exact(first["HT"] / (first["HT"] + first["TH"])))
    plain = plain_rule(s, 1 - s)
    emit("E-F6.sticky.plain_rule.after_heads_percent", pct(plain["H"], 1))
    emit("E-F6.sticky.plain_rule.after_tails_percent", pct(plain["T"], 1))
    emit("E-F6.sticky.plain_rule.long_run.exact", exact(long_h * plain["H"] + (1 - long_h) * plain["T"]))
    sam = after_only_rule(s, 1 - s, "H")
    emit("E-F6.sticky.after_heads_only.from_heads.exact", exact(sam["H"]))
    emit("E-F6.sticky.after_heads_only.from_tails.exact", exact(sam["T"]))
    # a lopsided memory (after a head 9/10, after a tail 1/2): unfair even from a long-run start
    a, b = F(9, 10), F(1, 2)
    lop = plain_rule(a, b)
    lop_h = b / ((1 - a) + b)
    emit("E-F6.lopsided.plain_rule.long_run.exact", exact(lop_h * lop["H"] + (1 - lop_h) * lop["T"]))
    lsam = after_only_rule(a, b, "H")
    emit("E-F6.lopsided.after_heads_only.exact", exact(lsam["H"]))
    check("Samuelson's rule is fair from either start", lsam["H"] == lsam["T"] == F(1, 2))
    # Blum 1986 (FOCS 1984), Theorems A and B, on his chain with eps = 1/5
    eager, n_a = blum_first_two(s, on_return=False)
    patient, n_b = blum_first_two(s, on_return=True)
    check("Blum: each algorithm's four chances add to one", sum(eager.values()) == 1 == sum(patient.values()))
    check("Blum B (announce on return) is exactly fair and independent", all(v == F(1, 4) for v in patient.values()))
    check("Blum A (announce at once) is not", any(v != F(1, 4) for v in eager.values()))
    for k in ("HH", "HT", "TH", "TT"):
        emit(f"E-F7.blum.at_once.{k.lower()}", dec(eager[k], 4))
        emit(f"E-F7.blum.on_return.{k.lower()}", dec(patient[k], 4))
    print(f"# Blum: {n_a} memory states at once, {n_b} on return; at once exact: "
          + ", ".join(f"{k} {exact(v)}" for k, v in eager.items()), file=sys.stderr)
    # his "alternatively eps = 1/2": even when every exit is a fair toss, A favours a T first
    fair_a, _ = blum_first_two(F(1, 2), on_return=False)
    check("Blum A at eps = 1/2: the first result is T with chance 5/8", fair_a["TH"] + fair_a["TT"] == F(5, 8))
    for stay in (F(1, 2), F(9, 10), F(3, 10), F(99, 100)):
        b_, _ = blum_first_two(stay, on_return=True)
        check(f"Blum B is fair at stay = {exact(stay)}", all(v == F(1, 4) for v in b_.values()))
    # Theorem A part 2 (k = 2): after two Hs, two Ts almost never follow as eps -> 0 ("it should be 1/4"),
    # and Theorem B on a longer ring, by following every history forward in floats
    a4 = blum_first_n(2, 0.01, on_return=False, n=4)
    hh = sum(v for s, v in a4.items() if s.startswith("HH"))
    check("Blum A, eps = 1/100: P(TT next | HH) is under 0.01", a4.get("HHTT", 0.0) / hh < 0.01)
    check("Blum A, eps = 1/100: the quadruple HHTT has chance under 0.001 (fair: 1/16)", a4.get("HHTT", 0.0) < 0.001)
    for k, eps in ((2, 0.01), (4, 0.01), (6, 0.5)):
        b4 = blum_first_n(k, eps, on_return=True, n=4)
        check(f"Blum B, k = {k}, eps = {eps}: every quadruple has chance 1/16",
              len(b4) == 16 and all(abs(v - 1 / 16) < 1e-9 for v in b4.values()))
    # part 1 as we compute it: from fresh states the first results are Ts (chance (1-eps)/(2-eps) at
    # each first departure), so about k/2 of them as eps -> 0 and k/3 at eps = 1/2 come before any H
    a_first = blum_first_n(2, 0.01, on_return=False, n=1)
    check("Blum A, eps = 1/100: the first result is T about three times in four", abs(a_first["T"] - 0.75) < 0.01)


# --------------------------------------------------------------------------
# E-F8, E-F9: same counts, same chance
# --------------------------------------------------------------------------
def esym(w, n):
    """The elementary symmetric polynomial e_n, by listing the subsets."""
    return sum((math.prod(c) for c in combinations(w, n)), F(0))


def pattern_facts():
    for n in (2, 3, 4):
        emit(f"E-F8.pascal.row{n}", ",".join(str(math.comb(n, k)) for k in range(n + 1)))
    seqs = ["".join(s) for s in product("HT", repeat=4)]
    chance = {s: P ** s.count("H") * Q_ ** s.count("T") for s in seqs}
    two = sorted(s for s in seqs if s.count("H") == 2)
    check("four tosses, two heads: six patterns, one chance", len(two) == 6 and len({chance[s] for s in two}) == 1)
    emit("E-F8.four_two.patterns", len(two))
    emit("E-F8.four_two.list", ",".join(two))
    emit("E-F8.four_two.each", dec(chance[two[0]], 4))
    emit("E-F8.four_two.each.exact", exact(chance[two[0]]))
    hit = sum(chance[s] for s in two)
    emit("E-F8.four_two.works_percent", pct(hit, 1))
    emit("E-F8.four_two.expected_tosses", dec(4 / hit, 1))
    fair_hit = F(math.comb(4, 2), 16)
    emit("E-F8.four_two.fair_coin.works_percent", pct(fair_hit, 1))
    emit("E-F8.four_two.fair_coin.expected_tosses", dec(4 / fair_hit, 1))
    for k in range(5):
        emit(f"E-F8.row4.heads{k}_percent", pct(sum(chance[s] for s in seqs if s.count("H") == k), 2))
    # three tosses: two fair three-way choices (one head, or two)
    three = sum(P ** s.count("H") * Q_ ** s.count("T") for s in map("".join, product("HT", repeat=3))
                if s.count("H") in (1, 2))
    emit("E-F8.three.works_percent", pct(three))
    emit("E-F8.three_plus_coin.expected_tosses", dec(3 / three + 1 / (P * Q_), 1))
    # every count class for n up to 6, at several chances: one chance per class
    ok = True
    for p in STOPS + [F(1, 3), F(1, 1000)]:
        for n in range(1, 7):
            for k in range(n + 1):
                cls = {math.prod(p if c == "H" else 1 - p for c in s) for s in product("HT", repeat=n)
                       if s.count("H") == k}
                ok &= len(cls) == 1
    check("every pattern with the same count of heads has the same chance (n <= 6)", ok)


def dice_facts():
    w = [F(x, sum(LOADED)) for x in LOADED]
    fair = [F(1, 6)] * 6
    emit("E-F9.loaded.percents", ",".join(pct(x) for x in w))
    emit("E-F9.loaded.six_percent", pct(w[5]))
    two = {"first": F(0), "second": F(0), "tie": F(0)}
    for a, b in product(range(6), repeat=2):
        two["first" if a > b else "second" if b > a else "tie"] += w[a] * w[b]
    check("two rolls: first bigger and second bigger are equally likely", two["first"] == two["second"])
    check("two rolls: the tie chance is the sum of squares", two["tie"] == sum(x * x for x in w))
    emit("E-F9.two_rolls.tie_percent", pct(two["tie"], 1))
    emit("E-F9.two_rolls.first_bigger_percent", pct(two["first"], 2))
    emit("E-F9.two_rolls.decides_percent", pct(1 - two["tie"], 1))
    emit("E-F9.two_rolls.expected_rolls", dec(2 / (1 - two["tie"]), 1))
    fair_tie = sum(x * x for x in fair)
    emit("E-F9.two_rolls.fair_die.expected_rolls", dec(2 / (1 - fair_tie), 1))
    # three rolls, all different: 3! orders, each e_3(w)
    by_order = {}
    distinct = F(0)
    for rolls in product(range(6), repeat=3):
        if len(set(rolls)) == 3:
            pattern = tuple(sorted(rolls).index(r) for r in rolls)
            by_order[pattern] = by_order.get(pattern, F(0)) + math.prod(w[r] for r in rolls)
            distinct += math.prod(w[r] for r in rolls)
    check("three different rolls: six orders, one chance", len(by_order) == 6 and len(set(by_order.values())) == 1)
    check("each order's chance is e_3 of the weights", next(iter(by_order.values())) == esym(w, 3))
    emit("E-F9.three_rolls.orders", len(by_order))
    emit("E-F9.three_rolls.each_order_percent", pct(esym(w, 3), 2))
    emit("E-F9.three_rolls.works_percent", pct(distinct, 1))
    emit("E-F9.three_rolls.expected_rolls", dec(3 / distinct, 1))
    fair_distinct = 6 * esym(fair, 3)
    emit("E-F9.three_rolls.fair_die.works.exact", exact(fair_distinct))
    emit("E-F9.three_rolls.fair_die.expected_rolls", dec(3 / fair_distinct, 1))
    emit("E-F9.three_rolls.trios", math.comb(6, 3))
    emit("E-F9.three_rolls.boxes_all_different", sum(1 for r in product(range(6), repeat=3) if len(set(r)) == 3))
    emit("E-F9.three_rolls.boxes", 6**3)
    # the brick the cube picture follows: faces four, five and six
    emit("E-F9.brick.sides_percent", ",".join(pct(w[i]) for i in (3, 4, 5)))
    emit("E-F9.brick.volume_percent", pct(w[3] * w[4] * w[5], 1))
    for n in range(2, 7):
        emit(f"E-F9.orders.{n}", math.factorial(n))
    # n different values give n! equally likely orders, for other dice too
    ok = True
    for weights in ([1, 2, 3, 4, 5, 6], [1, 1, 1, 1, 1, 1], LOADED):
        ww = [F(x, sum(weights)) for x in weights]
        for n in (2, 3, 4):
            seen = {}
            for rolls in product(range(6), repeat=n):
                if len(set(rolls)) == n:
                    pattern = tuple(sorted(rolls).index(r) for r in rolls)
                    seen[pattern] = seen.get(pattern, F(0)) + math.prod(ww[r] for r in rolls)
            ok &= len(seen) == math.factorial(n) and len(set(seen.values())) == 1
    check("n different rolls: n! orders, equally likely (n = 2, 3, 4; three dice)", ok)


# --------------------------------------------------------------------------
# E-F10: what the trick throws away
# --------------------------------------------------------------------------
def entropy(p):
    return sum(-x * math.log2(x) for x in (p, 1 - p) if x > 0)


def peres_rate(depth, p):
    """Peres 1992, Proposition 2: r_1 = pq, r_v(p) = pq + r_{v-1}(p^2+q^2)/2 + (p^2+q^2) r_{v-1}(p^2/(p^2+q^2))/2."""
    q = 1 - p
    if depth == 0 or p <= 0 or q <= 0:
        return 0.0
    same = p * p + q * q
    return p * q + peres_rate(depth - 1, same) / 2 + same * peres_rate(depth - 1, p * p / same) / 2


def elias_bits(m):
    """Elias 1972: a fair choice among m, read off through the binary digits of m."""
    return sum((1 << j) * j for j in range(m.bit_length()) if m >> j & 1) / m


def elias_rate(n, p):
    return sum(math.comb(n, k) * p**k * (1 - p) ** (n - k) * elias_bits(math.comb(n, k)) for k in range(n + 1)) / n


PERES_DEPTHS = (1, 2, 3, 4, 6, 8)
ELIAS_BLOCKS = (2, 4, 8, 16, 32, 64)


def rate_facts():
    p = float(P)
    h = entropy(p)
    emit("E-F10.von_neumann.rate", dec(P * Q_, 2))
    emit("E-F10.von_neumann.per_hundred", pct(P * Q_))
    emit("E-F10.entropy", f"{h:.4f}")
    emit("E-F10.entropy.per_hundred", f"{100 * h:.0f}")
    emit("E-F10.von_neumann.share_of_entropy_percent", f"{100 * p * (1 - p) / h:.0f}")
    emit("E-F10.fair_coin.von_neumann.rate", dec(F(1, 4), 2))
    emit("E-F10.fair_coin.von_neumann.percent", pct(F(1, 4)))
    emit("E-F10.fair_coin.entropy", f"{entropy(0.5):.0f}")
    for d in PERES_DEPTHS:
        emit(f"E-F10.peres.depth{d}.rate", f"{peres_rate(d, p):.4f}")
    for n in ELIAS_BLOCKS:
        emit(f"E-F10.elias.block{n}.rate", f"{elias_rate(n, p):.4f}")
    check("Peres depth 1 and Elias block 2 are von Neumann's rule",
          abs(peres_rate(1, p) - 0.16) < 1e-15 and abs(elias_rate(2, p) - 0.16) < 1e-15)
    check("the rates climb and stay under the entropy",
          all(peres_rate(d, p) < peres_rate(d + 1, p) < h for d in range(1, 12))
          and all(elias_rate(n, p) < elias_rate(2 * n, p) < h for n in (2, 4, 8, 16, 32, 64, 128)))
    check("the entropy is a fixed point of Peres's recursion",
          abs(p * (1 - p) + entropy(p * p + (1 - p) ** 2) / 2
              + (p * p + (1 - p) ** 2) * entropy(p * p / (p * p + (1 - p) ** 2)) / 2 - h) < 1e-12)
    best = max(F(k, 1000) * (1 - F(k, 1000)) for k in range(1001))
    check("von Neumann's rule gives at most one fair toss per four (his 25 percent)", best == F(1, 4))


# --------------------------------------------------------------------------
# E-X: the edges of the always/never sentences that are counts
# --------------------------------------------------------------------------
def edge_facts():
    """No rule that reads a fixed number of tosses and then names a winner is fair for every coin.
    A rule is the set of patterns the first player wins on; her chance is a polynomial of degree n in
    p, so "one half for every p" is the same as "one half at n + 1 chances". Checked two ways: every
    rule as a set of patterns (n <= 4: 65,536 rules), and every list of how many winning patterns
    have each count of heads (n <= 5), which is how src/model.rs counts."""
    for n in range(1, 6):
        chances = [F(i, n + 2) for i in range(1, n + 2)]
        seqs = list(product((1, 0), repeat=n))  # 1 is a head
        if n <= 4:
            # integer weights: at p = i/(n+2) a pattern with k heads has chance i^k (n+2-i)^(n-k) / (n+2)^n
            fair_sets = 0
            sums = [[0] * (1 << len(seqs)) for _ in chances]
            for c, p in enumerate(chances):
                i = p.numerator * (n + 2) // p.denominator
                weight = [i ** sum(s) * (n + 2 - i) ** (n - sum(s)) for s in seqs]
                for rule in range(1, 1 << len(seqs)):
                    low = rule & -rule
                    sums[c][rule] = sums[c][rule ^ low] + weight[low.bit_length() - 1]
            for rule in range(1 << len(seqs)):
                fair_sets += all(2 * sums[c][rule] == (n + 2) ** n for c in range(len(chances)))
            check(f"n = {n}: none of the {1 << len(seqs)} fixed-length rules is fair for every coin", fair_sets == 0)
        row = [math.comb(n, k) for k in range(n + 1)]
        lists = fair = 0
        for wins in product(*(range(c + 1) for c in row)):
            lists += 1
            fair += all(sum(a * p**k * (1 - p) ** (n - k) for k, a in enumerate(wins)) == F(1, 2) for p in chances)
        emit(f"E-X1.fixed_rules.n{n}.count_lists", lists)
        emit(f"E-X1.fixed_rules.n{n}.fair_for_every_coin", fair)
    # the check can say yes: for a fair coin alone, "the first toss wins" is fair
    check("the fairness check accepts a rule that is fair at p = 1/2", F(1, 2) * 1 + F(1, 2) * 0 == F(1, 2))


# --------------------------------------------------------------------------
# E-S: the seeded runs, replayed with NumPy's PCG64DXSM
# --------------------------------------------------------------------------
class Stream:
    """Uniform draws in [0, 1), one at a time, from run `i`'s own generator."""

    def __init__(self, i):
        self.seed = SEED * 100 + i
        self.rng = np.random.Generator(np.random.PCG64DXSM(self.seed))
        self.buf, self.at = np.empty(0), 0

    def unit(self):
        if self.at == len(self.buf):
            self.buf, self.at = self.rng.random(65536), 0
        self.at += 1
        return float(self.buf[self.at - 1])

    def toss(self, p):
        return "H" if self.unit() < float(p) else "T"

    def tosses(self, n, p):
        return "".join(self.toss(p) for _ in range(n))

    def roll(self, weights):
        u, total, upto = self.unit(), sum(weights), 0
        for face, w in enumerate(weights, 1):
            upto += w
            if u < upto / total:
                return face
        return len(weights)


def play(stream, n, first, second):
    """n games of the trick: each a list of pairs, the last one deciding."""
    out = []
    for _ in range(n):
        game = []
        while True:
            pair = stream.toss(first) + stream.toss(second)
            game.append(pair)
            if pair[0] != pair[1]:
                break
        out.append(game)
    return out


def von_neumann(seq):
    return "".join(seq[i] for i in range(0, len(seq) - 1, 2) if seq[i] != seq[i + 1])


def peres(seq, depth):
    """Peres's procedure, with a work list instead of recursion."""
    out, work = [], [(seq, depth)]
    while work:
        s, d = work.pop(0)
        if d == 0 or len(s) < 2:
            continue
        pairs = [s[i:i + 2] for i in range(0, len(s) - 1, 2)]
        out.append((d, "".join(a for a, b in pairs if a != b)))
        work.append(("".join("H" if a != b else "T" for a, b in pairs), d - 1))
        work.append(("".join(a for a, b in pairs if a == b), d - 1))
    return "".join(piece for _, piece in out)


def elias(seq, n):
    """Elias's rule on blocks of n: the block's place among the patterns with as many heads
    (found by listing them), read off through the binary digits of the class size."""
    places = {}
    out = []
    for i in range(0, len(seq) - n + 1, n):
        block = seq[i:i + n]
        k = block.count("H")
        if k not in places:
            places[k] = {"".join("H" if j in c else "T" for j in range(n)): r
                         for r, c in enumerate(combinations(range(n), k))}
        rank, m = places[k][block], math.comb(n, k)
        for j in reversed(range(m.bit_length())):
            if m >> j & 1:
                if rank < 1 << j:
                    out.append("".join("H" if (rank >> b) & 1 == 0 else "T" for b in reversed(range(j))))
                    break
                rank -= 1 << j
    return "".join(out)


def run_facts():
    emit("E-S0.seed", SEED)
    # S1: just call it
    s1 = Stream(1).tosses(1000, P)
    emit("E-S1.call_it.tosses", len(s1))
    emit("E-S1.call_it.heads", s1.count("H"))
    emit("E-S1.call_it.tails", s1.count("T"))
    # S2: the trick, 1,000 games
    g = play(Stream(2), 1000, P, P)
    wins = sum(1 for game in g if game[-1] == "HT")
    used = sum(2 * len(game) for game in g)
    emit("E-S2.trick.games", len(g))
    emit("E-S2.trick.heads_first", wins)
    emit("E-S2.trick.tails_first", len(g) - wins)
    emit("E-S2.trick.tosses", used)
    emit("E-S2.trick.mean_tosses", dec(F(used, len(g)), 2))
    emit("E-S2.trick.longest_pairs", max(len(game) for game in g))
    emit("E-S2.trick.first_games", " ".join("".join(game) for game in g[:8]))
    # S3: 1,000 games at each stop of the slider
    for i, p in enumerate(STOPS):
        gs = play(Stream(10 + i), 1000, p, p)
        emit(f"E-S3.{stop_name(p)}.heads_first", sum(1 for game in gs if game[-1] == "HT"))
        emit(f"E-S3.{stop_name(p)}.mean_tosses", dec(F(sum(2 * len(x) for x in gs), len(gs)), 2))
    # S4: the price, 10,000 games
    g4 = play(Stream(4), 10000, P, P)
    used4 = sum(2 * len(game) for game in g4)
    emit("E-S4.price.games", len(g4))
    emit("E-S4.price.mean_tosses", dec(F(used4, len(g4)), 2))
    emit("E-S4.price.first_pair_percent", pct(F(sum(1 for x in g4 if len(x) == 1), len(g4)), 1))
    hist = [sum(1 for x in g4 if len(x) == r) for r in range(1, 9)]
    emit("E-S4.price.pairs_1_to_8", ",".join(map(str, hist)))
    emit("E-S4.price.pairs_9_plus", len(g4) - sum(hist))
    emit("E-S4.price.longest_pairs", max(len(x) for x in g4))
    # S5: two different coins
    g5 = play(Stream(5), 1000, P, F(1, 2))
    emit("E-S5.two_coins.games", len(g5))
    emit("E-S5.two_coins.heads_first", sum(1 for game in g5 if game[-1] == "HT"))
    # S6: the sticky coin, 200,000 tosses after a head
    st, prev, seq = Stream(6), "H", []
    for _ in range(200000):
        prev = st.toss(P if prev == "H" else 1 - P)
        seq.append(prev)
    seq = "".join(seq)
    pairs = [seq[i:i + 2] for i in range(0, len(seq), 2)]
    counts = {("HH", "HT"): 0, ("HH", "TH"): 0, ("TT", "HT"): 0, ("TT", "TH"): 0}
    for before, now in zip(pairs, pairs[1:]):
        if (before, now) in counts:
            counts[(before, now)] += 1
    emit("E-S6.sticky.tosses", len(seq))
    emit("E-S6.sticky.heads_percent", pct(F(seq.count("H"), len(seq)), 1))
    emit("E-S6.sticky.after_hh.ht", counts[("HH", "HT")])
    emit("E-S6.sticky.after_hh.th", counts[("HH", "TH")])
    emit("E-S6.sticky.after_hh.heads_first_percent",
         pct(F(counts[("HH", "HT")], counts[("HH", "HT")] + counts[("HH", "TH")]), 1))
    emit("E-S6.sticky.after_tt.ht", counts[("TT", "HT")])
    emit("E-S6.sticky.after_tt.th", counts[("TT", "TH")])
    follows = "".join(x for before, x in zip("H" + seq, seq) if before == "H")
    sam = von_neumann(follows)
    emit("E-S6.sticky.after_heads_only.heads_first", sam.count("H"))
    emit("E-S6.sticky.after_heads_only.tails_first", sam.count("T"))
    emit("E-S6.sticky.after_heads_only.heads_first_percent", pct(F(sam.count("H"), len(sam)), 1))
    # S7: a die from the coin, 24,000 tosses in blocks of four
    s7 = Stream(7).tosses(24000, P)
    faces = sorted("".join(c) for c in product("HT", repeat=4) if c.count("H") == 2)
    blocks = [s7[i:i + 4] for i in range(0, len(s7), 4)]
    made = [faces.index(b) for b in blocks if b in faces]
    emit("E-S7.coin_die.tosses", len(s7))
    emit("E-S7.coin_die.dice", len(made))
    emit("E-S7.coin_die.faces", ",".join(str(made.count(i)) for i in range(6)))
    emit("E-S7.coin_die.tosses_per_die", dec(F(len(s7), len(made)), 1))
    # S8: the loaded die, 6,000 rolls
    st8 = Stream(8)
    r8 = [st8.roll(LOADED) for _ in range(6000)]
    emit("E-S8.loaded.rolls", len(r8))
    emit("E-S8.loaded.faces", ",".join(str(r8.count(f)) for f in range(1, 7)))
    orders = sorted(permutations(range(3)))
    made8 = []
    for i in range(0, len(r8), 3):
        t = r8[i:i + 3]
        if len(set(t)) == 3:
            made8.append(orders.index(tuple(sorted(t).index(x) for x in t)))
    emit("E-S8.three_rolls.dice", len(made8))
    emit("E-S8.three_rolls.orders", ",".join(str(made8.count(i)) for i in range(6)))
    emit("E-S8.three_rolls.rolls_per_die", dec(F(len(r8), len(made8)), 1))
    two = [r8[i:i + 2] for i in range(0, len(r8), 2)]
    emit("E-S8.two_rolls.first_bigger", sum(1 for a, b in two if a > b))
    emit("E-S8.two_rolls.second_bigger", sum(1 for a, b in two if b > a))
    emit("E-S8.two_rolls.ties", sum(1 for a, b in two if a == b))
    # S9: fair tosses squeezed from 100,000 unfair ones
    s9 = Stream(9).tosses(100000, P)
    emit("E-S9.squeeze.tosses", len(s9))
    emit("E-S9.squeeze.heads", s9.count("H"))
    for name, out in (("von_neumann", von_neumann(s9)), ("peres2", peres(s9, 2)), ("peres3", peres(s9, 3)),
                      ("peres_all", peres(s9, 40)), ("elias16", elias(s9, 16))):
        emit(f"E-S9.squeeze.{name}.fair_tosses", len(out))
        emit(f"E-S9.squeeze.{name}.heads", out.count("H"))
        emit(f"E-S9.squeeze.{name}.per_hundred", dec(F(100 * len(out), len(s9)), 1))
    check("the work-list Peres equals von Neumann's rule at depth 1", peres(s9[:2000], 1) == von_neumann(s9[:2000]))


# --------------------------------------------------------------------------
# E-C: cited literals (sources and pages: docs/fact-sheet.md)
# --------------------------------------------------------------------------
def cited_facts():
    emit("E-C1.von_neumann.talk_year", 1949)
    emit("E-C1.von_neumann.printed_year", 1951)
    emit("E-C1.von_neumann.pages", "36-38")
    emit("E-C1.von_neumann.series_number", 12)
    emit("E-C1.von_neumann.efficiency_percent", 25)
    emit("E-C2.samuelson.year", 1968)
    emit("E-C2.hoeffding_simons.year", 1970)
    emit("E-C2.elias.year", 1972)
    emit("E-C2.blum.year", 1986)
    emit("E-C2.dijkstra.year", 1990)
    emit("E-C2.dijkstra.outcomes", 37)
    emit("E-C2.peres.year", 1992)
    emit("E-C2.juels.year", 2000)
    emit("E-C3.gelman_nolan.year", 2002)
    emit("E-C3.diaconis.year", 2007)
    emit("E-C3.diaconis.same_side_percent", 51)
    emit("E-C3.bartos.year", 2025)
    emit("E-C3.bartos.flips", 350757)
    emit("E-C3.bartos.same_side", "0.508")
    emit("E-C3.pennies.spins", 5520)
    emit("E-C3.pennies.heads", 1878)
    emit("E-C3.pennies.heads_percent", pct(F(1878, 5520)))
    emit("E-C4.intel.year", 1999)
    emit("E-C4.intel.raw_samples_per_bit", 6)
    # Dijkstra's roulette: 37 tosses, not all alike; the 37 rotations are different because 37 is prime
    emit("E-C2.dijkstra.patterns_per_outcome", (2**37 - 2) // 37)
    check("2^37 - 2 is a multiple of 37", (2**37 - 2) % 37 == 0)


def main():
    coin_facts()
    breaks_facts()
    pattern_facts()
    dice_facts()
    rate_facts()
    edge_facts()
    run_facts()
    cited_facts()
    print(f"# {len(_EMITTED)} facts, {_CHECKS[0]} cross-checks, {len(_FAILED)} failed", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
