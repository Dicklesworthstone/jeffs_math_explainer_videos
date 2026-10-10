#!/usr/bin/env python3
"""Count complete schedules (interleavings) of small ONE-SHOT programs: every process
runs its entry protocol, critical section and exit exactly once.

Busy-wait loops are modelled as blocking 'await' steps: a spin iteration only re-reads
and never writes, so for safety this is equivalent, and it makes the schedule tree
finite.  Counting uses memoised DP over (pcs, shared, locals, in-CS flags, seen-bad),
so it counts root-to-leaf paths of the full schedule tree without enumerating them."""
from functools import lru_cache


def make_bakery(N, with_choosing=True, cmp="lex"):
    """Return per-process op lists.  Ops:
       ("w", var, f(sh, m, i))        write shared var
       ("rmax", j)                    m := max(m, number[j])   (one read)
       ("await", pred(sh, i), enter)  blocking guard; enter=True marks CS entry
       ("exit", var)                  leave CS, write var := 0"""
    def less(a, b):
        if cmp == "lex":
            return a < b
        if cmp == "strict":
            return a[0] < b[0]
        return a[0] <= b[0]          # "le"

    progs = []
    for i in range(N):
        js = [j for j in range(N) if j != i]
        p = []
        if with_choosing:
            p.append(("w", f"c{i}", lambda sh, m, i: 1))
        for j in js:
            p.append(("rmax", f"n{j}"))
        p.append(("w", f"n{i}", lambda sh, m, i: m + 1))
        if with_choosing:
            p.append(("w", f"c{i}", lambda sh, m, i: 0))
        for k, j in enumerate(js):
            if with_choosing:
                p.append(("await", (lambda j: lambda sh, i: sh[f"c{j}"] == 0)(j), False))
            p.append(("await", (lambda j: lambda sh, i:
                       sh[f"n{j}"] == 0 or not less((sh[f"n{j}"], j), (sh[f"n{i}"], i)))(j),
                      k == len(js) - 1))
        p.append(("exit", f"n{i}"))
        progs.append(tuple(p))
    names = sorted({f"c{i}" for i in range(N)} | {f"n{i}" for i in range(N)})
    return progs, names


def count(progs, names):
    n = len(progs)
    idx = {v: k for k, v in enumerate(names)}

    @lru_cache(maxsize=None)
    def go(pcs, sh, ms, incs, bad):
        shd = {v: sh[idx[v]] for v in names}
        total = nbad = ndead = 0
        moved = False
        for i in range(n):
            if pcs[i] >= len(progs[i]):
                continue
            op = progs[i][pcs[i]]
            sh2, ms2, inc2 = list(sh), list(ms), list(incs)
            if op[0] == "await":
                if not op[1](shd, i):
                    continue
                if op[2]:
                    inc2[i] = 1
            elif op[0] == "w":
                sh2[idx[op[1]]] = op[2](shd, ms[i], i)
                if len(op) > 3 and op[3]:
                    inc2[i] = 1
            elif op[0] == "rmax":
                ms2[i] = max(ms[i], shd[op[1]])
            elif op[0] == "exit":
                inc2[i] = 0
                sh2[idx[op[1]]] = 0
            moved = True
            p2 = list(pcs); p2[i] += 1
            t, b, d = go(tuple(p2), tuple(sh2), tuple(ms2), tuple(inc2), bad or sum(inc2) >= 2)
            total += t; nbad += b; ndead += d
        if not moved:
            done = all(pcs[i] >= len(progs[i]) for i in range(n))
            return 1, int(bad), int(not done)
        return total, nbad, ndead

    return go(tuple([0] * n), tuple([0] * len(names)), tuple([0] * n), tuple([0] * n), False)


def make_naive_flag(N=2):
    """'Check a shared flag, then set it' lock:  await flag == 0; flag := 1; CS; flag := 0.
    The await (a read) and the write are separate steps."""
    progs = []
    for i in range(N):
        progs.append((("await", lambda sh, i: sh["f"] == 0, False),   # read flag, see 0
                      ("w", "f", lambda sh, m, i: 1, True),         # flag := 1 and walk into CS
                      ("exit", "f")))
    return progs, ["f"]


def lost_update():
    # x = 0; each of 2 threads: t := x ; x := t + 1
    from itertools import permutations
    outcomes = {}
    seqs = set(permutations("AABB"))
    for s in seqs:
        x, t, k = 0, {}, {"A": 0, "B": 0}
        for th in s:
            if k[th] == 0:
                t[th] = x
            else:
                x = t[th] + 1
            k[th] += 1
        outcomes[x] = outcomes.get(x, 0) + 1
    return len(seqs), outcomes


if __name__ == "__main__":
    total, outc = lost_update()
    print(f"lost update: {total} schedules; final x -> count: {outc}")
    p, nm = make_naive_flag(2)
    t, b, d = count(p, nm)
    print(f"naive flag lock N=2: {t} complete schedules | {b} put 2 in CS | {d} deadlock")
    for N in (2, 3):
        for wc in (False, True):
            for cmp in ("lex", "strict", "le"):
                progs, names = make_bakery(N, wc, cmp)
                t, b, d = count(progs, names)
                print(f"one-shot N={N} choosing={'yes' if wc else 'no '} compare={cmp:6s}: "
                      f"{t:>14,} complete schedules | {b:>12,} put 2+ in CS | {d:>10,} end in deadlock")
