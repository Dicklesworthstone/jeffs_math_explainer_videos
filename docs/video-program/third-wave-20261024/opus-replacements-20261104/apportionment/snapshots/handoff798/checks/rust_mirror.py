"""Line-by-line Python transliteration of the candidate's model.rs + facts.rs
(same integer algorithms, same formatting), diffed in-process against
oracle.py's output. Checks logic and formats, not Rust types or syntax.

    python3 -I -B rust_mirror.py <path to oracle.py>
"""
import functools
import subprocess
import sys

KEYS = ["ash", "birch", "cedar"]
POP = [4000, 9000, 26000]
POP2 = [3900, 9500, 26100]
POP_GROWN = [POP[0], POP2[1], POP2[2]]
POP_PUZZLE = [4000, 9000, 30000]
POP4 = [2000, 5000, 8000, 27000]
KEYS4 = ["a", "b", "c", "d"]
GPP = {"A": [824, 44, 44, 44, 44], "B": [824, 44, 44, 44, 222], "C": [824, 1, 1, 44, 44]}


class R:
    def __init__(s, n, d):
        s.num, s.den = n, d

    def floor(s):
        return s.num // s.den

    def ceil(s):
        return -(-s.num // s.den)

    def tail(s):
        return R(s.num % s.den, s.den)

    def whole(s):
        return s.num % s.den == 0


def cmp(a, b):
    x, y = a.num * b.den, b.num * a.den
    return (x > y) - (x < y)


def total(p):
    return sum(p)


def share(p, i, h):
    return R(h * p[i], total(p))


def within(seats, p, h):
    return all(share(p, i, h).floor() <= seats[i] <= share(p, i, h).ceil() for i in range(len(p)))


def hamilton(p, h):
    n = len(p)
    seats = [share(p, i, h).floor() for i in range(n)]
    left = h - sum(seats)
    tail = lambda i: share(p, i, h).tail().num
    order = sorted(range(n), key=lambda i: (-tail(i), i))
    if 0 < left < n and tail(order[left - 1]) == tail(order[left]):
        raise ValueError("tie")
    for i in order[:left]:
        seats[i] += 1
    return seats


def naive(p, h):
    return [(2 * share(p, i, h).num + share(p, i, h).den) // (2 * share(p, i, h).den) for i in range(len(p))]


def signpost(rule, p, k):
    if rule == "jefferson":
        return R(p, k)
    if rule == "adams":
        return R(p, k - 1)
    if rule == "webster":
        return R(2 * p, 2 * k - 1)
    return R(p * p, k * (k - 1))


def rust_max_by(items, key_cmp):
    """Rust's Iterator::max_by returns the LAST maximum; the Rust code adds
    .then(b.cmp(&a)) so that, among equals, the lower index compares greater."""
    best = None
    for x in items:
        if best is None:
            best = x
            continue
        # max_by keeps x when cmp(x, best) is not Less (i.e. x >= best)
        if key_cmp(x, best) >= 0:
            best = x
    return best


def divisor(rule, p, h):
    n = len(p)
    seats = [0] * n
    for step in range(1, h + 1):
        nxt = lambda i: signpost(rule, p[i], seats[i] + 1)
        best = rust_max_by(range(n), lambda a, b: cmp(nxt(a), nxt(b)) or ((b > a) - (b < a)))
        tied = sum(1 for i in range(n) if cmp(nxt(i), nxt(best)) == 0) > 1
        if tied and step == h:
            raise ValueError("tie")
        seats[best] += 1
    return seats


def signposts(rule, p, count):
    allp = [(i, k, signpost(rule, p[i], k)) for i in range(len(p)) for k in range(1, count + 2)]
    allp.sort(key=functools.cmp_to_key(lambda a, b: -cmp(a[2], b[2]) or (a[0] - b[0])))
    return allp[:count]


def window(rule, p, h):
    lst = signposts(rule, p, h + 1)
    hi, lo = lst[h - 1][2], lst[h][2]
    assert cmp(hi, lo) != 0
    return lo, hi


def quota_rule(p, h):
    n = len(p)
    seats = [0] * n
    hist = [list(seats)]
    for hh in range(1, h + 1):
        el = [i for i in range(n) if seats[i] * total(p) < hh * p[i]]
        pri = lambda i: signpost("jefferson", p[i], seats[i] + 1)
        best = rust_max_by(el, lambda a, b: cmp(pri(a), pri(b)) or ((b > a) - (b < a)))
        if sum(1 for i in el if cmp(pri(i), pri(best)) == 0) > 1:
            raise ValueError("tie")
        seats[best] += 1
        hist.append(list(seats))
    return hist


def decimal(r, places):
    scale = 10**places
    n = (2 * r.num * scale + r.den) // (2 * r.den)
    if places == 0:
        return str(n)
    s = str(n).rjust(places + 1, "0")
    return s[:-places] + "." + s[-places:]


def grouped(n):
    return f"{n:,}"


def number(r):
    if r.whole():
        return grouped(r.floor())
    w, f = decimal(r, 1).split(".")
    return grouped(int(w)) + "." + f


def people(r):
    return grouped((2 * r.num + r.den) // (2 * r.den))


def facts():
    out = []
    add = lambda k, v: out.append((k, str(v)))
    T = total(POP)
    for i, k in enumerate(KEYS):
        add(f"pop.{k}", grouped(POP[i]))
    add("pop.total", grouped(T))
    for h in [14, 15, 16, 17]:
        w = 0
        for i, k in enumerate(KEYS):
            q = share(POP, i, h)
            add(f"share.{h}.{k}", decimal(q, 2))
            add(f"whole.{h}.{k}", q.floor())
            add(f"tail39.{h}.{k}", q.tail().num * 39 // q.den)
            add(f"tail.{h}.{k}", decimal(q.tail(), 2))
            w += q.floor()
        add(f"leftover.{h}", h - w)
    for i, k in [(0, "ash"), (1, "birch")]:
        t = share(POP, i, 15).tail()
        add(f"tail13.15.{k}", t.num * 13 // t.den)
    add("std_divisor.15", grouped(T // 15))
    for i, k in enumerate(KEYS):
        sp = R(POP[i], T)
        add(f"speed39.{k}", sp.num * 39 // sp.den)
        add(f"speed.{k}", decimal(sp, 2))
    for h in [14, 15, 16]:
        r = naive(POP, h)
        for i, k in enumerate(KEYS):
            add(f"naive.{h}.{k}", r[i])
        add(f"naive.{h}.total", sum(r))
    for h in [14, 15, 16, 17]:
        for i, k in enumerate(KEYS):
            add(f"hamilton.{h}.{k}", hamilton(POP, h)[i])
    ev = []
    for h in range(1, 100):
        a, b = hamilton(POP, h), hamilton(POP, h + 1)
        lo = [i for i in range(3) if b[i] < a[i]]
        if lo:
            ev.append((h, lo))
    add("alabama.additions", 99)
    add("alabama.count", len(ev))
    add("alabama.loser", ",".join(sorted({KEYS[i] for e in ev for i in e[1]})))
    add("alabama.houses", ",".join(f"{e[0]}to{e[0] + 1}" for e in ev))
    add("hamilton.within_quota_1_100", "yes" if all(within(hamilton(POP, h), POP, h) for h in range(1, 101)) else "no")
    lo, hi = window("jefferson", POP, 15)
    add("jefferson.15.window_low", number(lo))
    add("jefferson.15.window_high", number(hi))
    add("jefferson.example_divisor", grouped(2300))
    d = 2300
    s = 0
    for i, k in enumerate(KEYS):
        x = R(POP[i], d)
        add(f"jefferson.at2300.{k}", decimal(x, 2))
        s += x.floor()
    add("jefferson.at2300.total", s)
    s = 0
    for i, k in enumerate(KEYS):
        q = share(POP, i, 15)
        add(f"at2600.{k}", decimal(q, 2))
        s += q.floor()
    add("at2600.whole_total", s)
    for i, k in [(0, "ash"), (2, "cedar")]:
        p = POP[i]
        add(f"jefferson.drop_pct.{k}", decimal(R(100 * (p % d), p), 1))
    for h in [14, 15, 16, 17]:
        for i, k in enumerate(KEYS):
            add(f"jefferson.{h}.{k}", divisor("jefferson", POP, h)[i])
    lst = signposts("jefferson", POP, 19)
    for r, (i, k, v) in enumerate(lst):
        add(f"opening.{r + 1}", f"{KEYS[i]}{k}@{people(v)}")
    tie = next(r for r in range(1, len(lst)) if cmp(lst[r][2], lst[r - 1][2]) == 0)
    add("opening.first_tie_rank", tie)
    add("opening.first_tie_value", number(lst[tie][2]))
    for i, k in enumerate(KEYS):
        add(f"adams.15.{k}", divisor("adams", POP, 15)[i])
    for rule in ["webster", "hill"]:
        for h in [15, 16, 17]:
            for i, k in enumerate(KEYS):
                add(f"{rule}.{h}.{k}", divisor(rule, POP, h)[i])
    for rule in ["jefferson", "adams", "webster", "hill"]:
        bad = above = below = 0
        ties, bh = [], []
        for h in range(1, 101):
            try:
                s = divisor(rule, POP, h)
            except ValueError:
                ties.append(str(h))
                continue
            qs = [share(POP, i, h) for i in range(3)]
            bad += not within(s, POP, h)
            above += any(s[i] > qs[i].ceil() for i in range(3))
            low = any(s[i] < qs[i].floor() for i in range(3))
            below += low
            if low:
                bh.append(str(h))
        add(f"{rule}.quota_breaks_1_100", bad)
        add(f"{rule}.above_quota_1_100", above)
        add(f"{rule}.below_quota_1_100", below)
        add(f"{rule}.tie_houses_1_100", ",".join(ties) or "none")
        if rule == "hill":
            add("hill.below_quota_houses_1_100", ",".join(bh))
            for i, k in enumerate(KEYS):
                add(f"hill.3.{k}", divisor("hill", POP, 3)[i])
            add("share.3.cedar", decimal(share(POP, 2, 3), 2))
    for i, k in enumerate(KEYS4):
        add(f"w4.pop.{k}", grouped(POP4[i]))
    add("w4.pop.total", grouped(total(POP4)))
    for i, k in enumerate(KEYS4):
        add(f"w4.share.14.{k}", decimal(share(POP4, i, 14), 2))
    w4 = divisor("webster", POP4, 14)
    for i, k in enumerate(KEYS4):
        add(f"w4.webster.14.{k}", w4[i])
    add("w4.webster.14.within_quota", "yes" if within(w4, POP4, 14) else "no")
    lo, hi = window("webster", POP4, 14)
    add("w4.window_low", number(lo))
    add("w4.window_high", number(hi))
    add("webster.three_town_quota_breaks_1_100", sum(not within(divisor("webster", POP, h), POP, h) for h in range(1, 101)))
    last = 100
    while True:
        try:
            hist = quota_rule(POP, last)
            break
        except ValueError:
            last -= 1
    for h in [14, 15, 16, 17]:
        for i, k in enumerate(KEYS):
            add(f"quotamethod.{h}.{k}", hist[h][i])
    add("quotamethod.first_tie_house", str(last + 1) if last < 100 else "none")
    add("quotamethod.checked_through", last)
    add("quotamethod.within_quota_checked", "yes" if all(within(hist[h], POP, h) for h in range(1, last + 1)) else "no")
    add("quotamethod.never_takes_back_checked",
        "yes" if all(hist[h + 1][i] >= hist[h][i] for h in range(last) for i in range(3)) else "no")
    add("quotamethod.15.cedar_ceiling", share(POP, 2, 15).ceil())
    add("quotamethod.15.birch_priority", people(signpost("jefferson", POP[1], 4)))
    add("quotamethod.15.ash_priority", people(signpost("jefferson", POP[0], 2)))
    for i, k in enumerate(KEYS):
        add(f"census2.pop.{k}", grouped(POP2[i]))
    add("census2.pop.total", grouped(total(POP2)))
    for i, k in enumerate(KEYS):
        add(f"census2.share.16.{k}", decimal(share(POP2, i, 16), 2))
    for i, k in enumerate(KEYS):
        add(f"census2.hamilton.16.{k}", hamilton(POP2, 16)[i])
    for rule in ["jefferson", "webster"]:
        s1, s2 = divisor(rule, POP, 16), divisor(rule, POP2, 16)
        for i, k in enumerate(KEYS):
            add(f"census2.{rule}.16.{k}", s2[i])
        add(f"census2.{rule}.paradox", "yes" if s2[2] < s1[2] and s2[0] > s1[0] else "no")
    for name, rule, pops, count in [
        ("census2.grown", "jefferson", POP_GROWN, 18),
        ("census2", "jefferson", POP2, 18),
        ("webster", "webster", POP, 17),
        ("puzzle.webster", "webster", POP_PUZZLE, 17),
    ]:
        for r, (i, k, v) in enumerate(signposts(rule, pops, count)):
            add(f"{name}.opening.{r + 1}", f"{KEYS[i]}{k}@{people(v)}")
    add("puzzle.pop.cedar", grouped(POP_PUZZLE[2]))
    t = share(POP, 0, 15).tail()
    add("lottery.15.ash.two_pct", decimal(R(100 * t.num, t.den), 0))
    add("lottery.15.ash.one_pct", decimal(R(100 * (t.den - t.num), t.den), 0))
    for i, k in enumerate(KEYS):
        add(f"puzzle.webster.15.{k}", divisor("webster", POP_PUZZLE, 15)[i])
    for name, pops in GPP.items():
        add(f"gpp.{name}.total", total(pops))
        for i in range(5):
            q = share(pops, i, 10)
            n = i + 1
            add(f"gpp.{name}.pop.{n}", pops[i])
            add(f"gpp.{name}.share.{n}", decimal(q, 2))
            add(f"gpp.{name}.floor.{n}", q.floor())
            add(f"gpp.{name}.ceil.{n}", q.ceil())
    for k, v in [("seaton.house_before", 299), ("seaton.house_after", 300), ("seaton.alabama_before", 8),
                 ("seaton.alabama_after", 7), ("seaton.range_low", 275), ("seaton.range_high", 350),
                 ("ny1830.quota", "38.593"), ("ny1830.seats", 40), ("hill_law_year", 1941),
                 ("quota_method_year", 1975), ("by_impossibility_year", 1982), ("gpp_year", 2024)]:
        add("history." + k, v)
    return out


def main():
    oracle = subprocess.run([sys.executable, "-I", "-B", sys.argv[1]], capture_output=True, text=True, check=True).stdout
    theirs = {}
    for line in oracle.splitlines():
        if line.strip():
            k, _, v = line.partition("\t")
            theirs[k] = v
    ours = facts()
    keys = [k for k, _ in ours]
    assert len(keys) == len(set(keys)), "repeated key in mirror"
    only = [k for k, _ in ours if k not in theirs]
    differ = [(k, v, theirs[k]) for k, v in ours if k in theirs and theirs[k] != v]
    print(f"mirror {len(ours)} keys, oracle {len(theirs)} keys")
    print("only in mirror:", only)
    print("oracle keys the Rust side does not print:", [k for k in theirs if k not in dict(ours)])
    print("differ:", differ)
    return 1 if only or differ else 0


if __name__ == "__main__":
    sys.exit(main())
