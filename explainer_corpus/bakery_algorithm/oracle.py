#!/usr/bin/env python3
"""Facts oracle for Video C, "The Algorithm Lamport Is Proudest Of" (bead mx-e-c-7v0.2).

Run from the repository root (`./x bakery_algorithm facts-check` runs it from
this folder):

    python3 -I explainer_corpus/bakery_algorithm/oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, covering every row C-F1 ... C-F14 of
the facts ledger in docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md
section 11.5, plus supplementary rows C-X1 ... C-X3 for narrated claims the
ledger does not list. Integers are printed without separators; lists are
comma-separated strings; booleans are 1/0.

stderr: provenance and cross-checks (lines starting with "#"), then a summary.
The exit status is 1 if any cross-check failed. Values are never forced to
agree with the plan; plan values are compared in fact-sheet.md, not here.

How values are obtained:
  * computed here, from scratch (interleaving counts by explicit path
    enumeration for two customers and by an avoidance DP for three; slates
    example; arithmetic; code transliterations), and cross-checked against the
    research scripts;
  * computed by reusing the research scripts' side-effect-free modules
    (bakery_mc.py, tree_count.py, bakery_tso.py) on the same schedules and
    bounds the scripts use; the schedules of scenarios.py, leapfrog.py and
    df_trace.py are copied here because those files run on import;
  * RECORDED: values read from a cited output file in
    docs/research/bakery_algorithm/scripts/outputs/ (keys ending in
    ".recorded"); each is also recomputed live and compared;
  * CITED: dates, citations and verbatim-quote numbers (row C-F14), pinned
    from primary sources listed in fact-sheet.md.

Needs only the Python standard library. Sets sys.dont_write_bytecode so that
importing the research scripts leaves no __pycache__ behind.
"""
import sys

sys.dont_write_bytecode = True  # importing the research scripts must not write into their folder

import math
import re
import time
from collections import deque
from itertools import permutations, product
from pathlib import Path

T0 = time.time()
HERE = Path(__file__).resolve().parent
REPO = next(d for d in HERE.parents if (d / "docs" / "research").is_dir())  # explainer_corpus/<slug>/ -> root
SCRIPTS = REPO / "docs" / "research" / "bakery_algorithm" / "scripts"
OUTPUTS = SCRIPTS / "outputs"
sys.path.insert(0, str(SCRIPTS))

import bakery_mc  # noqa: E402  (library: classes and functions only)
import bakery_tso  # noqa: E402  (main code guarded by __name__ == "__main__")
import tree_count  # noqa: E402  (main code guarded by __name__ == "__main__")

from bakery_mc import Bakery, explore  # noqa: E402

# --------------------------------------------------------------------------
# output helpers
# --------------------------------------------------------------------------
_EMITTED = set()
_FAILED = []
_PASSED = [0]


def emit(key, value):
    """Print one fact. Keys have no whitespace; values have no tab/newline."""
    if key in _EMITTED or not key or any(c.isspace() for c in key):
        raise SystemExit(f"oracle bug: bad or duplicate key {key!r}")
    if isinstance(value, bool):
        value = int(value)
    value = str(value)
    if "\t" in value or "\n" in value:
        raise SystemExit(f"oracle bug: value of {key} holds a tab or newline")
    _EMITTED.add(key)
    print(f"{key}\t{value}", flush=True)


def note(msg):
    print(f"# {msg}", file=sys.stderr, flush=True)


def check(label, ok, detail=""):
    """A cross-check. Failures are reported at the end; they never alter a value."""
    if ok:
        _PASSED[0] += 1
        note(f"check ok   {label}" + (f"  [{detail}]" if detail else ""))
    else:
        _FAILED.append(label)
        note(f"CHECK FAIL {label}" + (f"  [{detail}]" if detail else ""))


def recorded(fname, pattern):
    """Find `pattern` in a recorded output file; return (match, 'file:line')."""
    path = OUTPUTS / fname
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        m = re.search(pattern, line)
        if m:
            return m, f"outputs/{fname}:{lineno}"
    raise SystemExit(f"oracle: pattern {pattern!r} not found in {path}")


def num(s):
    return int(s.replace(",", ""))


def timed(fn, *a, **kw):
    t = time.time()
    r = fn(*a, **kw)
    return r, time.time() - t


# --------------------------------------------------------------------------
# Independent interleaving engine (written for this oracle; shares no code
# with tree_count.py). Semantics, as in plan 02 lower-third / memo 3.7: each
# read, each write and each successful wait is one step; a wait can be taken
# only when it would pass; the max reads the other customers' tickets one at
# a time in index order; each customer runs entry, room and exit once.
# Memory is a tuple: c_0..c_{N-1} (flags) then n_0..n_{N-1} (tickets).
# A customer is in the room when its next op is its exit write.
# --------------------------------------------------------------------------

def bakery_ops(N, i, flag, cmp):
    others = [j for j in range(N) if j != i]
    ops = []
    if flag:
        ops.append(("set_flag", 1))
    ops += [("read_max", j) for j in others]
    ops.append(("write_ticket",))
    if flag:
        ops.append(("set_flag", 0))
    for j in others:
        if flag:
            ops.append(("await_flag_down", j))
        ops.append(("await_turn", j, cmp))
    ops.append(("exit",))
    return ops


def naive_sign_ops():
    # "check that it says vacant" (a successful wait), "flip it to occupied and go in", "leave"
    return [("await_vacant",), ("flip_occupied",), ("exit_sign",)]


def apply_op(op, i, N, mem, m):
    """Return (mem', m') or None when the op is a wait that cannot pass."""
    k = op[0]
    if k == "set_flag":
        return mem[:i] + (op[1],) + mem[i + 1:], m
    if k == "read_max":
        return mem, max(m, mem[N + op[1]])
    if k == "write_ticket":
        return mem[:N + i] + (m + 1,) + mem[N + i + 1:], m
    if k == "await_flag_down":
        return (mem, m) if mem[op[1]] == 0 else None
    if k == "await_turn":
        j, cmp = op[1], op[2]
        nj, ni = mem[N + j], mem[N + i]
        if cmp == "lex":
            blocked = nj != 0 and (nj < ni or (nj == ni and j < i))
        elif cmp == "strict":
            blocked = nj != 0 and nj < ni
        elif cmp == "le":
            blocked = nj != 0 and nj <= ni
        else:
            raise ValueError(cmp)
        return None if blocked else (mem, m)
    if k == "exit":
        return mem[:N + i] + (0,) + mem[N + i + 1:], m
    # naive sign: memory is the single sign (0 vacant, 1 occupied)
    if k == "await_vacant":
        return (mem, m) if mem[0] == 0 else None
    if k == "flip_occupied":
        return (1,), m
    if k == "exit_sign":
        return (0,), m
    raise ValueError(k)


def in_room(progs, pcs):
    return sum(1 for p, pc in zip(progs, pcs) if pc == len(p) - 1)


def enumerate_paths(progs, N, mem0):
    """Brute force for two customers: every maximal order, as its lattice path."""
    out = []

    def go(pcs, mem, ms, path):
        moved = False
        for i, p in enumerate(progs):
            if pcs[i] >= len(p):
                continue
            r = apply_op(p[pcs[i]], i, N, mem, ms[i])
            if r is None:
                continue
            moved = True
            p2 = list(pcs)
            p2[i] += 1
            ms2 = list(ms)
            ms2[i] = r[1]
            go(tuple(p2), r[0], tuple(ms2), path + [tuple(p2)])
        if not moved:
            done = all(pcs[i] >= len(p) for i, p in enumerate(progs))
            out.append((path, done))

    go(tuple(0 for _ in progs), mem0, tuple(0 for _ in progs), [tuple(0 for _ in progs)])
    return out


def path_stats(progs, paths):
    red = tuple(len(p) - 1 for p in progs)          # the cell where both are in the room
    total = len(paths)
    both_inside = sum(1 for path, _ in paths if any(in_room(progs, pt) >= 2 for pt in path))
    through_red = sum(1 for path, _ in paths if red in path)
    dead = sum(1 for _, done in paths if not done)
    end_both_inside = sum(1 for path, _ in paths if in_room(progs, path[-1]) >= 2)
    return total, both_inside, through_red, dead, end_both_inside


def count_dp(progs, N, mem0):
    """Leaf counts without enumeration. bad = all leaves - leaves reachable while
    never having two or more customers in the room (an avoidance count, a
    different method from tree_count.py's sticky flag)."""
    memo_all, memo_safe = {}, {}

    def succ(pcs, mem, ms):
        for i, p in enumerate(progs):
            if pcs[i] < len(p):
                r = apply_op(p[pcs[i]], i, N, mem, ms[i])
                if r is not None:
                    p2 = pcs[:i] + (pcs[i] + 1,) + pcs[i + 1:]
                    yield p2, r[0], ms[:i] + (r[1],) + ms[i + 1:]

    def all_leaves(s):
        if s in memo_all:
            return memo_all[s]
        total = dead = 0
        moved = False
        for t in succ(*s):
            moved = True
            a, d = all_leaves(t)
            total += a
            dead += d
        if not moved:
            pcs = s[0]
            total, dead = 1, int(not all(pcs[i] >= len(p) for i, p in enumerate(progs)))
        memo_all[s] = (total, dead)
        return total, dead

    def safe_leaves(s):
        if in_room(progs, s[0]) >= 2:
            return 0
        if s in memo_safe:
            return memo_safe[s]
        total = 0
        moved = False
        for t in succ(*s):
            moved = True
            total += safe_leaves(t)
        if not moved:
            total = 1
        memo_safe[s] = total
        return total

    s0 = (tuple(0 for _ in progs), mem0, tuple(0 for _ in progs))
    total, dead = all_leaves(s0)
    return total, total - safe_leaves(s0), dead


def bakery_progs(N, flag, cmp):
    return [bakery_ops(N, i, flag, cmp) for i in range(N)]


# --------------------------------------------------------------------------
# Replays against bakery_mc's transition relation (each step must match
# exactly one enabled transition, as in bakery_mc.replay), keeping raw states.
# --------------------------------------------------------------------------

def run_schedule(model, schedule):
    st = model.initial()
    rows = []
    for (p, sub) in schedule:
        i = p - 1
        cands = [(lab, ns) for (pi, lab, ns) in model.successors(st) if pi == i and sub in lab]
        if len(cands) != 1:
            raise SystemExit(f"oracle: replay step P{p} {sub!r} matched {len(cands)} transitions")
        lab, ns = cands[0]
        rows.append((p, lab, st, ns))
        st = ns
    return rows


def max_in_room(model, rows):
    return max(len(model.in_cs(ns)) for (_, _, _, ns) in rows)


# Schedules copied from docs/research/bakery_algorithm/scripts/scenarios.py
# (that file replays them at import time, so it is not imported).
SCHED_A = [(1, "start"), (1, "read number[2]=0"),
           (2, "start"), (2, "read number[1]=0"), (2, "number[2]:=1"),
           (2, "L3: number[1]=0, pass"),
           (1, "number[1]:=1"), (1, "L3: number[2]=1, pass")]
SCHED_A2 = [(1, "choosing:=1"), (1, "read number[2]=0"),
            (2, "choosing:=1"), (2, "read number[1]=0"), (2, "number[2]:=1"), (2, "choosing:=0"),
            (2, "L2: choosing[1]=1, wait"),
            (1, "number[1]:=1"), (1, "choosing:=0"),
            (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, pass"),
            (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait"),
            (1, "number[1]:=0"), (2, "L3: number[1]=0, pass")]
SCHED_B = [(1, "choosing:=1"), (2, "choosing:=1"), (1, "read number[2]=0"), (2, "read number[1]=0"),
           (1, "number[1]:=1"), (2, "number[2]:=1"), (1, "choosing:=0"), (2, "choosing:=0"),
           (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, pass"),
           (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, pass")]
SCHED_B2 = SCHED_B[:8] + [(1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, wait"),
                          (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait"),
                          (1, "L3: number[2]=1, wait"), (2, "L3: number[1]=1, wait")]
SCHED_C = [(1, "choosing:=1"), (2, "choosing:=1"), (1, "read number[2]=0"), (2, "read number[1]=0"),
           (1, "number[1]:=1"), (2, "number[2]:=1")]
SCHED_C3 = [(3, "choosing:=1"), (3, "read number[1]=0"), (3, "read number[2]=0"), (3, "number[3]:=1"),
            (3, "choosing:=0"),
            (1, "choosing:=1"), (2, "choosing:=1"),
            (1, "read number[2]=0"), (2, "read number[1]=0"),
            (1, "read number[3]=1"), (2, "read number[3]=1"),
            (1, "number[1]:=2"), (2, "number[2]:=2")]
SCHED_D = [
    (3, "choosing:=1"), (2, "choosing:=1"),
    (3, "read number[1]=0"), (3, "read number[2]=0"), (3, "read number[4]=0"),
    (2, "read number[1]=0"), (2, "read number[3]=0"),
    (3, "number[3]:=1"), (3, "choosing:=0"),
    (3, "L2: choosing[1]=0, pass"), (3, "L3: number[1]=0, pass"),
    (3, "L2: choosing[2]=1, wait"),
    (2, "read number[4]=0"), (2, "number[2]:=1"), (2, "choosing:=0"),
    (4, "choosing:=1"), (4, "read number[1]=0"), (4, "read number[2]=1"), (4, "read number[3]=1"),
    (4, "number[4]:=2"), (4, "choosing:=0"),
    (3, "L2: choosing[2]=0, pass"), (3, "L3: number[2]=1, wait"),
    (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=0, pass"),
    (2, "L2: choosing[3]=0, pass"), (2, "L3: number[3]=1, pass"),
    (2, "L2: choosing[4]=0, pass"), (2, "L3: number[4]=2, pass"),
    (1, "choosing:=1"), (1, "read number[2]=1"), (1, "read number[3]=1"), (1, "read number[4]=2"),
    (1, "number[1]:=3"), (1, "choosing:=0"),
    (2, "number[2]:=0"),
    (3, "L3: number[2]=0, pass"), (3, "L2: choosing[4]=0, pass"),
    (3, "L3: number[4]=2, pass"),
    (4, "L2: choosing[1]=0, pass"), (4, "L3: number[1]=3, pass"),
    (4, "L2: choosing[2]=0, pass"), (4, "L3: number[2]=0, pass"),
    (4, "L2: choosing[3]=0, pass"), (4, "L3: number[3]=1, wait"),
    (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=0, pass"),
    (1, "L2: choosing[3]=0, pass"), (1, "L3: number[3]=1, wait"),
    (3, "number[3]:=0"),
    (4, "L3: number[3]=0, pass"),
    (1, "L3: number[3]=0, pass"), (1, "L2: choosing[4]=0, pass"), (1, "L3: number[4]=2, wait"),
    (2, "choosing:=1"), (2, "read number[1]=3"), (2, "read number[3]=0"), (2, "read number[4]=2"),
    (2, "number[2]:=4"), (2, "choosing:=0"),
    (4, "number[4]:=0"),
    (1, "L3: number[4]=0, pass"),
    (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=3, wait"),
    (1, "number[1]:=0"),
    (2, "L3: number[1]=0, pass"), (2, "L2: choosing[3]=0, pass"), (2, "L3: number[3]=0, pass"),
    (2, "L2: choosing[4]=0, pass"), (2, "L3: number[4]=0, pass"),
]
SCHED_E1 = [(1, "begin choosing:=1"), (1, "end choosing:=1"), (1, "read number[2]=0"),
            (1, "begin number[1]:=1"),
            (2, "begin choosing:=1"), (2, "end choosing:=1"),
            (2, "read number[1]=7"),
            (1, "end number[1]:=1"), (1, "begin choosing:=0"), (1, "end choosing:=0"),
            (2, "begin number[2]:=8"), (2, "end number[2]:=8"), (2, "begin choosing:=0"),
            (2, "end choosing:=0"),
            (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=8, pass"),
            (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait")]
SCHED_E3 = SCHED_E1 + [(1, "begin number[1]:=0"), (1, "end number[1]:=0"),
                       (1, "begin choosing:=1"), (1, "end choosing:=1"),
                       (1, "read number[2]=8"), (1, "begin number[1]:=9"),
                       (2, "L3: number[1]=0, pass"),
                       (1, "end number[1]:=9"), (1, "begin choosing:=0"), (1, "end choosing:=0"),
                       (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=8, wait")]


def leapfrog_schedule():
    """Copied from leapfrog.py (which runs on import)."""
    s = [(1, "choosing:=1"), (1, "read number[2]=0"), (1, "number[1]:=1"), (1, "choosing:=0"),
         (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=0, pass")]
    for r in range(5):
        a, b = (2, 1) if r % 2 == 0 else (1, 2)
        nb = 1 + r
        na = nb + 1
        s += [(a, "choosing:=1"), (a, f"read number[{b}]={nb}"), (a, f"number[{a}]:={na}"),
              (a, "choosing:=0"), (a, f"L2: choosing[{b}]=0, pass"), (a, f"L3: number[{b}]={nb}, wait"),
              (b, f"number[{b}]:=0"), (a, f"L3: number[{b}]=0, pass")]
    return s


TICKET_WRITE = re.compile(r"^(?:end )?number\[(\d+)\]:=(\d+)$")


def tickets_taken(rows):
    out = []
    for (p, lab, _, _) in rows:
        m = TICKET_WRITE.match(lab)
        if m and int(m.group(2)) > 0:
            out.append((int(m.group(1)), int(m.group(2))))
    return out


def fmt_list(xs):
    return ",".join(str(x) for x in xs)


# ==========================================================================
# C-F1  lost-update orders
# ==========================================================================
def row_cf1():
    start = 41
    orders = set()
    for perm in permutations([("P1", "R"), ("P1", "W"), ("P2", "R"), ("P2", "W")]):
        if perm.index(("P1", "R")) < perm.index(("P1", "W")) and perm.index(("P2", "R")) < perm.index(("P2", "W")):
            orders.add(perm)
    lose, middle, finals = set(), set(), {}
    for o in orders:
        x, t, pos, pts = start, {}, {"P1": 0, "P2": 0}, []
        for who, op in o:
            if op == "R":
                t[who] = x
            else:
                x = t[who] + 1
            pos[who] += 1
            pts.append((pos["P1"], pos["P2"]))
        finals[o] = x
        if x != start + 2:
            lose.add(o)
        if (1, 1) in pts:
            middle.add(o)
    emit("C-F1.score_start", start)
    emit("C-F1.orders", len(orders))
    emit("C-F1.orders_lose_a_point", len(lose))
    emit("C-F1.orders_through_middle", len(middle))
    emit("C-F1.losing_orders_are_exactly_the_middle_orders", lose == middle)
    emit("C-F1.score_end_when_a_point_is_lost", sorted({finals[o] for o in lose})[0])
    emit("C-F1.score_end_otherwise", sorted({finals[o] for o in orders - lose})[0])
    emit("C-F1.binomial_4_choose_2", math.comb(4, 2))
    tc_total, tc_out = tree_count.lost_update()
    check("C-F1 tree_count.lost_update agrees", (tc_total, tc_out.get(1)) == (len(orders), len(lose)),
          f"tree_count: {tc_total} orders, {tc_out}")
    m, where = recorded("tree_count_out.txt", r"lost update: (\d+) schedules; final x -> count: \{1: (\d+), 2: (\d+)\}")
    check("C-F1 matches recording", (int(m.group(1)), int(m.group(2))) == (len(orders), len(lose)), where)


# ==========================================================================
# C-F2  naive check-then-flip sign
# ==========================================================================
def row_cf2():
    progs = [naive_sign_ops(), naive_sign_ops()]
    paths = enumerate_paths(progs, 1, (0,))
    total, both, red, dead, end_both = path_stats(progs, paths)
    emit("C-F2.orders", total)
    emit("C-F2.orders_both_inside", both)
    emit("C-F2.orders_through_red_square", red)
    emit("C-F2.orders_deadlocked", dead)
    emit("C-F2.orders_ending_with_both_inside", end_both)
    tc = tree_count.count(*tree_count.make_naive_flag(2))
    check("C-F2 tree_count agrees", tc == (total, both, dead), f"tree_count {tc}")
    m, where = recorded("tree_count_out.txt", r"naive flag lock N=2: (\d+) complete schedules \| (\d+) put 2 in CS \| (\d+) deadlock")
    check("C-F2 matches recording", tuple(map(int, m.groups())) == (total, both, dead), where)


# ==========================================================================
# C-F3 / C-F5  two customers, one visit each
# ==========================================================================
def two_customer_counts(flag, cmp):
    progs = bakery_progs(2, flag, cmp)
    mem0 = (0,) * 4
    total, both, red, dead, _ = path_stats(progs, enumerate_paths(progs, 2, mem0))
    dp = count_dp(progs, 2, mem0)
    check(f"N=2 flag={flag} cmp={cmp}: path enumeration == avoidance DP", dp == (total, both, dead), f"DP {dp}")
    tc = tree_count.count(*tree_count.make_bakery(2, flag, cmp))
    check(f"N=2 flag={flag} cmp={cmp}: tree_count.py agrees", tc == (total, both, dead), f"tree_count {tc}")
    m, where = recorded("crosscheck_out.txt", rf"^{flag} {cmp} \[(\d+), (\d+), (\d+)\]")
    check(f"N=2 flag={flag} cmp={cmp}: matches crosscheck recording", tuple(map(int, m.groups())) == (total, both, dead), where)
    return total, both, red, dead


def row_cf3():
    t, b, r, d = two_customer_counts(False, "lex")
    emit("C-F3.no_flag.orders", t)
    emit("C-F3.no_flag.orders_both_inside", b)
    emit("C-F3.no_flag.orders_through_red_square", r)
    emit("C-F3.no_flag.orders_deadlocked", d)
    t, b, r, d = two_customer_counts(True, "lex")
    emit("C-F3.flag.orders", t)
    emit("C-F3.flag.orders_both_inside", b)
    emit("C-F3.flag.orders_through_red_square", r)
    emit("C-F3.flag.orders_deadlocked", d)


def row_cf4():
    for flag, tag in ((False, "no_flag"), (True, "flag")):
        (total, bad, dead), secs = timed(count_dp, bakery_progs(3, flag, "lex"), 3, (0,) * 6)
        emit(f"C-F4.{tag}.orders", total)
        emit(f"C-F4.{tag}.orders_two_or_more_inside", bad)
        emit(f"C-F4.{tag}.orders_deadlocked", dead)
        tc = tree_count.count(*tree_count.make_bakery(3, flag, "lex"))
        check(f"C-F4 N=3 {tag}: avoidance DP == tree_count.py", tc == (total, bad, dead), f"tree_count {tc}; DP {secs:.1f}s")
        m, where = recorded("tree_count_out.txt",
                            rf"one-shot N=3 choosing={'yes' if flag else 'no '} compare=lex\s*:\s*([\d,]+) complete schedules \|\s*([\d,]+) put 2\+ in CS \|\s*([\d,]+) end in deadlock")
        check(f"C-F4 N=3 {tag}: matches recording", tuple(num(g) for g in m.groups()) == (total, bad, dead), where)


def row_cf5():
    t, b, _, d = two_customer_counts(True, "strict")
    emit("C-F5.strict_lt_with_flag.orders", t)
    emit("C-F5.strict_lt_with_flag.orders_both_inside", b)
    emit("C-F5.strict_lt_with_flag.orders_deadlocked", d)
    t, b, _, d = two_customer_counts(True, "le")
    emit("C-F5.le_with_flag.orders", t)
    emit("C-F5.le_with_flag.orders_both_inside", b)
    emit("C-F5.le_with_flag.orders_deadlocked", d)


# ==========================================================================
# C-F6  replayed traces
# ==========================================================================
def scenario_steps_recorded(header_prefix):
    """Step count of a trace in outputs/scenarios_out.txt (lines '  k. ...' under its header)."""
    lines = (OUTPUTS / "scenarios_out.txt").read_text(encoding="utf-8").splitlines()
    n, inside = 0, False
    for ln in lines:
        if ln.startswith("("):
            if inside:
                break
            inside = ln.startswith(header_prefix)
            continue
        if inside and re.match(r"^\s+\d+\. ", ln):
            n += 1
    return n


def row_cf6():
    # (a) no flag: both in the room after 8 steps
    m = Bakery(2, 5, "no_choosing", any_order=False)
    rows = run_schedule(m, SCHED_A)
    emit("C-F6.a.steps", len(rows))
    emit("C-F6.a.in_room_at_end", len(m.in_cs(rows[-1][3])))
    check("C-F6 a steps match scenarios_out.txt", scenario_steps_recorded("(a) ") == len(rows))

    # (a') the same order with the flag
    m = Bakery(2, 5, "correct", any_order=False)
    rows = run_schedule(m, SCHED_A2)
    emit("C-F6.a_prime.steps", len(rows))
    emit("C-F6.a_prime.max_in_room", max_in_room(m, rows))
    wait_step = next(k for k, (p, lab, _, _) in enumerate(rows, 1) if p == 2 and "choosing[1]=1, wait" in lab)
    emit("C-F6.a_prime.customer2_stopped_by_flag_at_step", wait_step)
    check("C-F6 a' steps match scenarios_out.txt", scenario_steps_recorded("(a') ") == len(rows))

    # (b1) numbers only, strict <: both enter
    m = Bakery(2, 5, "strict_nums", any_order=False)
    rows = run_schedule(m, SCHED_B)
    emit("C-F6.b1.steps", len(rows))
    emit("C-F6.b1.in_room_at_end", len(m.in_cs(rows[-1][3])))
    check("C-F6 b1 steps match scenarios_out.txt", scenario_steps_recorded("(b1) ") == len(rows))

    # (b2) numbers only, <=: deadlock (every enabled move is a self-loop)
    m = Bakery(2, 5, "le_nums", any_order=False)
    rows = run_schedule(m, SCHED_B2)
    st = rows[-1][3]
    succ = m.successors(st)
    emit("C-F6.b2.steps", len(rows))
    emit("C-F6.b2.deadlocked", bool(succ) and all(ns == st for (_, _, ns) in succ) and not m.in_cs(st))
    check("C-F6 b2 steps match scenarios_out.txt", scenario_steps_recorded("(b2) ") == len(rows))

    # (c) equal tickets in the correct algorithm; (c') three customers, both take 2
    m = Bakery(2, 5, "correct", any_order=False)
    rows = run_schedule(m, SCHED_C)
    emit("C-F6.c.steps", len(rows))
    emit("C-F6.c.tickets", fmt_list(rows[-1][3][2]))
    check("C-F6 c steps match scenarios_out.txt", scenario_steps_recorded("(c) ") == len(rows))
    m = Bakery(3, 5, "correct", any_order=False)
    rows = run_schedule(m, SCHED_C3)
    emit("C-F6.c_three_customers.steps", len(rows))
    emit("C-F6.c_three_customers.tickets", fmt_list(rows[-1][3][2]))
    check("C-F6 c' steps match scenarios_out.txt", scenario_steps_recorded("(c') ") == len(rows))

    # (d) four customers, 70 steps
    m = Bakery(4, 6, "correct", any_order=True)
    rows = run_schedule(m, SCHED_D)
    entries = [(ns[2][p - 1], p) for (p, lab, _, ns) in rows if "ENTER CS" in lab]
    stop = next(k for k, (p, lab, _, _) in enumerate(rows, 1) if p == 3 and "L2: choosing[2]=1, wait" in lab)
    emit("C-F6.d.steps", len(rows))
    emit("C-F6.d.room_entries", len(entries))
    emit("C-F6.d.service_order", "".join(f"({t},{p})" for t, p in entries))
    emit("C-F6.d.service_order_is_lexicographic", entries == sorted(entries))
    emit("C-F6.d.tickets_taken", ",".join(f"P{p}:{t}" for p, t in tickets_taken(rows)))
    emit("C-F6.d.customer3_stopped_by_customer2_flag_at_step", stop)
    emit("C-F6.d.max_in_room", max_in_room(m, rows))
    # 07_5 (claim walk, G3): the (ticket, customer) pairs customer 1 waits behind, in order
    waits = []
    for (p, lab, _, _) in rows:
        w = re.match(r"L3: number\[(\d+)\]=(\d+), wait", lab)
        if p == 1 and w and (not waits or waits[-1] != (int(w.group(2)), int(w.group(1)))):
            waits.append((int(w.group(2)), int(w.group(1))))
    emit("C-F6.d.customer1_waits_behind", "".join(f"({t},{j})" for t, j in waits))
    check("C-F6 d steps match scenarios_out.txt", scenario_steps_recorded("(d) ") == len(rows))

    # (e1) safe registers: a read overlapping a write returns garbage 7, ticket 8
    m = Bakery(2, 9, "correct", safe=True, any_order=False)
    rows = run_schedule(m, SCHED_E1)
    k = next(k for k, (p, lab, _, _) in enumerate(rows) if p == 2 and lab.startswith("read number[1]="))
    overl = m._writing_number(rows[k][2], 0)
    garbage = int(rows[k][1].split("=")[1])
    committed = rows[k][2][2][0]
    emit("C-F6.e1.steps", len(rows))
    emit("C-F6.e1.overlapping_read_value", garbage)
    emit("C-F6.e1.ticket_taken_after_garbage", dict((p, t) for p, t in tickets_taken(rows))[2])
    emit("C-F6.e1.max_in_room", max_in_room(m, rows))
    check("C-F6 e1 read overlaps a write and returns neither old nor new value",
          overl and garbage not in (committed, 1), f"committed {committed}, being written 1, read {garbage}")
    check("C-F6 e1 steps match scenarios_out.txt", scenario_steps_recorded("(e1) ") == len(rows))

    # (e3) an overlapping read returns 0 and lets customer 2 in; customer 1 must wait
    rows = run_schedule(m, SCHED_E3)
    k = next(k for k, (p, lab, _, _) in enumerate(rows) if p == 2 and lab.startswith("L3: number[1]=") and "ENTER CS" in lab)
    overl = m._writing_number(rows[k][2], 0)
    committed = rows[k][2][2][0]
    value = int(re.match(r"L3: number\[1\]=(\d+),", rows[k][1]).group(1))
    final = rows[-1][3]
    emit("C-F6.e3.steps", len(rows))
    emit("C-F6.e3.overlapping_read_value", value)
    emit("C-F6.e3.overlapping_read_equals_pre_write_value", committed == value)
    emit("C-F6.e3.customer2_enters_on_that_read", "ENTER CS" in rows[k][1])
    emit("C-F6.e3.final_tickets", fmt_list(final[2]))
    emit("C-F6.e3.max_in_room", max_in_room(m, rows))
    check("C-F6 e3 read overlaps customer 1's write of 9", overl and rows[k][2][0][0][0] == bakery_mc.WN)
    check("C-F6 e3 steps match scenarios_out.txt", scenario_steps_recorded("(e3) ") == len(rows))

    # leapfrog: tickets climb while the bakery never empties
    m = Bakery(2, 7, "correct", any_order=False)
    rows = run_schedule(m, leapfrog_schedule())
    emit("C-F6.leapfrog.steps", len(rows))
    emit("C-F6.leapfrog.tickets", fmt_list(t for _, t in tickets_taken(rows)))
    emit("C-F6.leapfrog.final_tickets", fmt_list(rows[-1][3][2]))
    mm, where = recorded("leapfrog_out.txt", r"tickets taken in order: ([\d,]+)")
    check("C-F6 leapfrog tickets match recording", mm.group(1) == fmt_list(t for _, t in tickets_taken(rows)), where)

    # double fetch (max reads a slate twice): BFS shortest violation, N=3 safe
    r, secs = timed(explore, Bakery(3, 3, "double_fetch", safe=True, any_order=False))
    emit("C-F6.double_fetch.n3_k3_safe.shortest_violation_steps", len(r["trace"]))
    emit("C-F6.double_fetch.n3_k3_safe.states", r["states"])
    emit("C-F6.double_fetch.n3_k3_safe.violation_found", r["bad"] is not None)
    mm, where = recorded("df_trace_out.txt", r"double_fetch N=3 K=3 safe fixed-order: (\d+) (.*)$")
    trace_lines = sum(1 for ln in (OUTPUTS / "df_trace_out.txt").read_text().splitlines() if re.match(r"^\s{4}P\d: ", ln))
    check("C-F6 double-fetch matches recording", (num(mm.group(1)), trace_lines) == (r["states"], len(r["trace"])),
          f"{where}; {secs:.1f}s")
    for K in (6, 9):
        r2 = explore(Bakery(2, K, "double_fetch", safe=True))
        emit(f"C-F6.double_fetch.n2_k{K}_safe.states", r2["states"])
        emit(f"C-F6.double_fetch.n2_k{K}_safe.violation_found", r2["bad"] is not None)
        mm, where = recorded("df_trace_out.txt", rf"double_fetch N=2 K={K} safe: states=([\d,]+) result=(\w+)")
        check(f"C-F6 double-fetch N=2 K={K} matches recording",
              (num(mm.group(1)), mm.group(2) == "OK") == (r2["states"], r2["bad"] is None), where)


# ==========================================================================
# C-F7  slates 0,3,0,0,2: customer 4 (plan) or customer 1 (video) takes 4;
#       then customers 3 and 4 tie on 5 (04_2)
# ==========================================================================
def row_cf7():
    slates = [0, 3, 0, 0, 2]
    me = 4
    reads = [slates[k - 1] for k in range(1, 6) if k != me]
    ticket = 1 + max(reads)
    after = list(slates)
    after[me - 1] = ticket
    order = [k for (_, k) in sorted((t, k) for k, t in enumerate(after, 1) if t)]
    emit("C-F7.slates_before", fmt_list(slates))
    emit("C-F7.customer4_reads", fmt_list(reads))
    emit("C-F7.customer4_ticket", ticket)
    emit("C-F7.slates_after", fmt_list(after))
    emit("C-F7.service_order", fmt_list(order))
    # R1 (Maya M3): the video's reader is Blue, customer 1, so a ticket is
    # never confused with a customer number (4 reading and writing 4)
    me = 1
    reads1 = [slates[k - 1] for k in range(1, 6) if k != me]
    ticket1 = 1 + max(reads1)
    after1 = list(slates)
    after1[me - 1] = ticket1
    order1 = [k for (_, k) in sorted((t, k) for k, t in enumerate(after1, 1) if t)]
    emit("C-F7.customer1_reads", fmt_list(reads1))
    emit("C-F7.customer1_ticket", ticket1)
    emit("C-F7.slates_after_customer1", fmt_list(after1))
    emit("C-F7.service_order_customer1", fmt_list(order1))

    # Same example in the bakery_mc model: customers 2 and 5 already hold 3 and 2
    # (waiting at L2); customer 4 runs its doorway; then EVERY interleaving of
    # 2, 4, 5 (no new arrivals) is explored and the room-entry orders collected.
    m = Bakery(5, 6, "correct", any_order=False)
    loc = [(bakery_mc.NCS, 0, 0, 0, 0)] * 5
    loc[1] = (bakery_mc.L2, 0, 0, 0, 0)
    loc[4] = (bakery_mc.L2, 0, 0, 0, 0)
    seated = (tuple(loc), (0,) * 5, (0, 3, 0, 0, 2), 0)
    st, door_reads = model_steps(m, seated, [(4, sub) for sub in DOORWAY_4])
    orders = model_service_orders(m, st, {2, 4, 5})
    emit("C-F7.model.customer4_reads", fmt_list(door_reads[4]))
    emit("C-F7.model.customer4_ticket", st[2][3])
    emit("C-F7.model.distinct_service_orders", len(orders))
    emit("C-F7.model.service_order", fmt_orders(orders))

    # Claim walk (G3): the video's reader is customer 1 (03_4-03_5), so the
    # model check is repeated for her doorway from the same seated state.
    st1, door_reads = model_steps(m, seated, [(1, sub) for sub in DOORWAY_1])
    orders = model_service_orders(m, st1, {1, 2, 5})
    emit("C-F7.model.customer1_reads", fmt_list(door_reads[1]))
    emit("C-F7.model.customer1_ticket", st1[2][0])
    emit("C-F7.model.customer1_distinct_service_orders", len(orders))
    emit("C-F7.model.customer1_service_order", fmt_orders(orders))

    # 04_1-04_4: back at 03's slates 4,3,0,0,2 before anyone is served,
    # customers 3 and 4 run their doorways side by side (each read of one
    # right after the same read of the other), both write the same ticket,
    # then EVERY interleaving of all five (no new arrivals) is explored.
    lockstep = [(3, "choosing:=1"), (4, "choosing:=1")]
    for a, b in zip(DOORWAY_3[1:5], DOORWAY_4[1:5]):
        lockstep += [(3, a), (4, b)]
    lockstep += [(3, "number[3]:="), (4, "number[4]:="), (3, "choosing:=0"), (4, "choosing:=0")]
    st2, door_reads = model_steps(m, st1, lockstep)
    orders = model_service_orders(m, st2, {1, 2, 3, 4, 5})
    emit("C-F7.model.tie.customer3_reads", fmt_list(door_reads[3]))
    emit("C-F7.model.tie.customer4_reads", fmt_list(door_reads[4]))
    emit("C-F7.model.tie.slates_after", fmt_list(st2[2]))
    emit("C-F7.model.tie.distinct_service_orders", len(orders))
    emit("C-F7.model.tie.service_order", fmt_orders(orders))


# Doorways for N = 5 with the max read in index order (any_order=False).
DOORWAY_1 = ("choosing:=1", "read number[2]", "read number[3]", "read number[4]", "read number[5]",
             "number[1]:=", "choosing:=0")
DOORWAY_3 = ("choosing:=1", "read number[1]", "read number[2]", "read number[4]", "read number[5]",
             "number[3]:=", "choosing:=0")
DOORWAY_4 = ("choosing:=1", "read number[1]", "read number[2]", "read number[3]", "read number[5]",
             "number[4]:=", "choosing:=0")


def model_steps(m, st, steps):
    """Take each (customer, label prefix) step; exactly one transition must match.
    Returns the final state and the ticket values each customer read."""
    reads = {}
    for (p, sub) in steps:
        (lab, st), = [(lab, ns) for (i, lab, ns) in m.successors(st) if i == p - 1 and lab.startswith(sub)]
        if lab.startswith("read"):
            reads.setdefault(p, []).append(int(lab.split("=")[1]))
    return st, reads


def model_service_orders(m, st, movers):
    """Every interleaving of the movers (1-based; nobody leaves the
    non-critical section again): the set of room-entry orders at the leaves."""
    movers = {p - 1 for p in movers}
    seen = {(st, ())}
    q = deque([(st, ())])
    orders = set()
    while q:
        s, seq = q.popleft()
        moved = False
        for (i, lab, ns) in m.successors(s):
            if i not in movers or ns == s or s[0][i][0] == bakery_mc.NCS:
                continue
            moved = True
            seq2 = seq + ((i + 1,) if "ENTER CS" in lab else ())
            if (ns, seq2) not in seen:
                seen.add((ns, seq2))
                q.append((ns, seq2))
        if not moved:
            orders.add(seq)
    return orders


def fmt_orders(orders):
    return fmt_list(next(iter(orders))) if len(orders) == 1 else "|".join(map(fmt_list, sorted(orders)))


# ==========================================================================
# C-F8  model-checker state counts (recorded and recomputed)
# ==========================================================================
REC_N4K4 = 3027633   # RECORDED: outputs/sweep_big.txt ("N=4 K=4 correct atomic states=3,027,633 ... OK", 711.7 s)
REC_N3K6 = 269550    # RECORDED: outputs/sweep_big.txt ("N=3 K=6 correct safe states=269,550 ... OK", 34.9 s)


def row_cf8():
    for (N, K, safe, rec, tag) in ((4, 4, False, REC_N4K4, "n4_k4_atomic"), (3, 6, True, REC_N3K6, "n3_k6_safe")):
        mm, where = recorded("sweep_big.txt",
                             rf"N={N} K={K} correct\s+{'safe' if safe else 'atomic'}\s+states=\s*([\d,]+) pruned=\s*([\d,]+) result=(.*?)\s+\(")
        check(f"C-F8 {tag}: pinned constant is what {where} says", num(mm.group(1)) == rec and mm.group(3).startswith("OK"), where)
        emit(f"C-F8.{tag}.states.recorded", rec)
        r, secs = timed(explore, Bakery(N, K, "correct", safe=safe))
        emit(f"C-F8.{tag}.states", r["states"])
        emit(f"C-F8.{tag}.violation_found", r["bad"] is not None)
        check(f"C-F8 {tag}: live recomputation equals the recording", r["states"] == rec and r["pruned"] == num(mm.group(2)),
              f"live {r['states']:,} states, {r['pruned']:,} pruned, {secs:.1f}s")


# ==========================================================================
# C-F9  x86-TSO store-buffer model
# ==========================================================================
def row_cf9():
    for fences, tag in (((), "no_fence"), (("F1",), "f1_only"), (("F2",), "f2_only"), (("F1", "F2"), "f1_f2")):
        mdl = bakery_tso.TSOBakery(2, 4, fences)
        (n, bad, tr), secs = timed(bakery_tso.explore_tso, mdl)
        emit(f"C-F9.{tag}.states", n)
        emit(f"C-F9.{tag}.violation_found", bad is not None)
        if bad is not None:
            emit(f"C-F9.{tag}.shortest_violation_steps", len(tr))
        if tag == "no_fence":
            _, ch, nu, _ = bad
            emit("C-F9.no_fence.memory_choosing_at_violation", fmt_list(ch))
            emit("C-F9.no_fence.memory_tickets_at_violation", fmt_list(nu))
            emit("C-F9.no_fence.in_room_at_violation", len(mdl.in_cs(bad)))
        fname = "none" if not fences else "+".join(fences)
        mm, where = recorded("bakery_tso_out.txt", rf"TSO N=2 K=4 fences={re.escape(fname)}\s*: states=\s*([\d,]+) -> (.*)$")
        rec_steps = re.search(r"shortest trace (\d+) steps", mm.group(2))
        live = (n, len(tr) if bad is not None else None)
        recd = (num(mm.group(1)), int(rec_steps.group(1)) if rec_steps else None)
        check(f"C-F9 {tag} matches recording", live == recd, f"{where}; {secs:.1f}s")
    for (N, K, B) in ((2, 6, 4), (3, 3, 3)):
        (n, bad, _), secs = timed(bakery_tso.explore_tso, bakery_tso.TSOBakery(N, K, ("F1", "F2"), B=B))
        emit(f"C-F9.f1_f2.n{N}_k{K}_b{B}.states", n)
        emit(f"C-F9.f1_f2.n{N}_k{K}_b{B}.violation_found", bad is not None)
        mm, where = recorded("tso3.txt", rf"TSO N={N} K={K} B={B} fences=F1\+F2: states=([\d,]+) -> (\w+)")
        check(f"C-F9 fenced N={N} K={K} B={B} matches recording", (num(mm.group(1)), mm.group(2) == "OK") == (n, bad is None),
              f"{where}; {secs:.1f}s")


# ==========================================================================
# C-F10 / C-F11  arithmetic
# ==========================================================================
def row_cf10():
    ms_year = 365.25 * 86400 * 1000            # Julian year, as Lamport's "a year" at 1 entry per msec
    emit("C-F10.ms_per_year", int(ms_year))
    emit("C-F10.two_pow_35", 2 ** 35)
    emit("C-F10.ms_per_year_below_two_pow_35", ms_year < 2 ** 35)
    emit("C-F10.two_pow_34", 2 ** 34)
    emit("C-F10.ms_per_year_below_two_pow_34", ms_year < 2 ** 34)
    emit("C-F10.ms_per_365_day_year", 365 * 86400 * 1000)
    # R1 (Dev dev-02): a 32-bit ticket at one per millisecond wraps in days
    emit("C-F10.u32_wrap_days_at_1_per_ms_1dp", round(2**32 / (86400 * 1000), 1))
    # claim walk (G3): a signed 32-bit ticket (Java int, AtomicIntegerArray) overflows at 2^31
    emit("C-F10.i32_wrap_days_at_1_per_ms_1dp", round(2**31 / (86400 * 1000), 1))


def row_cf11():
    year_s = 365.25 * 86400
    years = 2 ** 64 / 1e9 / year_s
    emit("C-F11.u64_years_at_1e9_per_s", repr(years))
    emit("C-F11.u64_years_at_1e9_per_s_1dp", f"{years:.1f}")
    emit("C-F11.i64_years_at_1e9_per_s_1dp", f"{2 ** 63 / 1e9 / year_s:.1f}")
    emit("C-F11.u64_years_at_1e9_per_s_365_day_years_1dp", f"{2 ** 64 / 1e9 / (365 * 86400):.1f}")


# ==========================================================================
# C-F12  reads per uncontended entry
# ==========================================================================
class Slates:
    """A shared array that counts loads by whose slate is read."""

    def __init__(self, n, init, me):
        self.v, self.me = [init] * n, me
        self.other_loads = self.own_loads = 0

    def load(self, k):
        if k == self.me:
            self.own_loads += 1
        else:
            self.other_loads += 1
        return self.v[k]

    def store(self, k, x):
        self.v[k] = x


def rust_panel_lock(i, choosing, number):
    """Transliteration of the plan's Rust lock() (section 11.4, chapter 10)."""
    n = len(number.v)
    choosing.store(i, True)
    t = 1 + max(number.load(k) for k in range(n))           # each slate read once (all N, own included)
    number.store(i, t)
    choosing.store(i, False)
    for j in (j for j in range(n) if j != i):
        while choosing.load(j):
            pass
        while True:
            nj = number.load(j)
            if nj == 0 or (nj, j) > (t, i):
                break


def lamport_1974_lock(i, choosing, number):
    """Transliteration of the 1974 program: max over number[1..N]; for j = 1..N (j = i included);
    L3 reads number[i] as written in the paper."""
    n = len(number.v)
    choosing.store(i, 1)
    number.store(i, 1 + max(number.load(k) for k in range(n)))
    choosing.store(i, 0)
    for j in range(n):
        while choosing.load(j) != 0:
            pass
        while True:
            nj = number.load(j)
            if not (nj != 0 and (nj, j) < (number.load(i), i)):
                break


def model_reads_alone(N):
    m = Bakery(N, 2, "correct", any_order=False)
    st = m.initial()
    reads = 0
    while True:
        (lab, st), = [(lab, ns) for (i, lab, ns) in m.successors(st) if i == 0]
        if lab.startswith(("read number", "L2:", "L3:")):
            reads += 1
        if "(exit)" in lab:
            return reads


def row_cf12():
    for N in (2, 3, 4, 5):
        emit(f"C-F12.other_slate_reads_alone.model.n{N}", model_reads_alone(N))
    ok = True
    for N in range(2, 9):
        c1, n1 = Slates(N, False, 0), Slates(N, 0, 0)
        rust_panel_lock(0, c1, n1)
        c2, n2 = Slates(N, 0, 0), Slates(N, 0, 0)
        lamport_1974_lock(0, c2, n2)
        others = (model_reads_alone(N), c1.other_loads + n1.other_loads, c2.other_loads + n2.other_loads)
        ok &= all(x == 3 * (N - 1) for x in others)
        if N == 5:
            emit("C-F12.other_slate_reads_alone.rust_panel.n5", others[1])
            emit("C-F12.own_slate_loads_alone.rust_panel.n5", c1.own_loads + n1.own_loads)
            emit("C-F12.other_slate_reads_alone.lamport1974.n5", others[2])
    emit("C-F12.other_slate_reads_equal_3_n_minus_1_for_n_2_to_8", ok)


# ==========================================================================
# C-F13  the JavaScript comma operator
# ==========================================================================
def js_comma(*operands):
    """JavaScript's comma operator evaluates every operand and yields the last."""
    return operands[-1]


def row_cf13():
    t, i, j = [0, 5, 3], 2, 1                                 # the example in comma_operator_check.js
    as_written = js_comma(t[j], j) < js_comma(t[i], i)
    lexicographic = t[j] < t[i] or (t[j] == t[i] and j < i)
    emit("C-F13.example.as_written", as_written)
    emit("C-F13.example.j_lt_i", j < i)
    emit("C-F13.example.lexicographic", lexicographic)
    grid = [(tj, ti, jj, ii) for tj, ti in product(range(5), repeat=2) for jj, ii in product(range(4), repeat=2) if jj != ii]
    emit("C-F13.as_written_equals_j_lt_i_on_every_grid_case",
         all((js_comma(tj, jj) < js_comma(ti, ii)) == (jj < ii) for tj, ti, jj, ii in grid))
    # RECORDED tool output (not recomputable in Python): README.md of the scripts; re-run 2026-10-10
    # with tsc 5.9.3 and node v26.8.1, see fact-sheet.md.
    emit("C-F13.tsc_version.recorded", "5.9.3")
    emit("C-F13.tsc_strict_diagnostics_on_owner_snippet.recorded", 0)


# ==========================================================================
# C-F14  dates, citations, quoted numbers (CITED; sources in fact-sheet.md)
# ==========================================================================
def row_cf14():
    cited = [
        ("C-F14.dijkstra_problem.year", 1965),
        ("C-F14.dijkstra_problem.citation", "CACM 8(9):569, Sept 1965"),
        ("C-F14.knuth_eventual_entry.year", 1966),
        ("C-F14.knuth_eventual_entry.citation", "CACM 9(5):321-322, May 1966"),
        ("C-F14.bakery_paper.year", 1974),
        ("C-F14.bakery_paper.citation", "CACM 17(8):453-455, Aug 1974"),
        ("C-F14.bakery_paper.received", "September 1973"),
        ("C-F14.nonatomic_operations.year", 1983),
        ("C-F14.nonatomic_operations.citation", "POPL 10, Jan 1983, 28-37"),
        ("C-F14.interprocess_communication.year", 1986),
        ("C-F14.interprocess_communication.citation", "Distributed Computing 1(2):77-101, 1986"),
        ("C-F14.time_clocks.year", 1978),
        ("C-F14.time_clocks.citation", "CACM 21(7):558-565, July 1978"),
        ("C-F14.turing_lecture.year", 2015),
        ("C-F14.turing_lecture.citation", "CACM 58(6):71-76, June 2015"),
        ("C-F14.turing_lecture.delivered", "PODC, Paris, July 2014"),
        ("C-F14.turing_award.year", 2013),
        ("C-F14.considered_impossible.year", 1973),
        ("C-F14.considered_impossible.brinch_hansen_page", 88),
        ("C-F14.turing_lecture_garbage.from", 9),
        ("C-F14.turing_lecture_garbage.to", 10),
        ("C-F14.turing_lecture_garbage.value", 2496),
        ("C-F14.publications_page_garbage.from", 0),
        ("C-F14.publications_page_garbage.to", 1),
        ("C-F14.publications_page_garbage.value", 7456),
    ]
    for k, v in cited:
        emit(k, v)


# ==========================================================================
# Supplementary rows (narrated claims not in the ledger)
# ==========================================================================
def row_cx1():
    """08_1: rewriting 09 as 10 one digit at a time; what a glance in between reads."""
    old, new = "09", "10"
    between = set()
    for order in permutations(range(2)):
        cur = list(old)
        for pos in order[:-1]:
            cur[pos] = new[pos]
            between.add("".join(cur))
    emit("C-X1.rewrite_09_as_10.reads_in_between", fmt_list(sorted(between)))


def row_cx2():
    """09_1: FCFS (doorway order) ghost check and Lamport's Assertion 2, as run_sweep.py 'fcfs'."""
    for (N, K, safe) in ((2, 6, False), (2, 6, True), (3, 4, False), (3, 4, True)):
        tag = f"n{N}_k{K}_{'safe' if safe else 'atomic'}"
        r, secs = timed(explore, Bakery(N, K, "correct", safe=safe, fcfs=True), check_assert2=True)
        emit(f"C-X2.fcfs_and_assertion2.{tag}.states", r["states"])
        emit(f"C-X2.fcfs_and_assertion2.{tag}.violation_found", r["bad"] is not None)
        mm, where = recorded("sweep_fcfs.txt",
                             rf"N={N} K={K} correct\s+{'safe' if safe else 'atomic'}\s+states=\s*([\d,]+) pruned=\s*([\d,]+) result=(.*?) \| Assertion2 checked in ([\d,]+) CS states")
        check(f"C-X2 {tag} matches recording",
              (num(mm.group(1)), mm.group(3).startswith("OK"), num(mm.group(4))) == (r["states"], r["bad"] is None, r["a2_checked"]),
              f"{where}; {secs:.1f}s")


def row_cx3():
    """09_2 / 04_3: liveness (every trying state can reach a room entry), atomic, as run_sweep.py 'liveness'."""
    for (N, K) in ((2, 6), (3, 4)):
        for var, tag in (("correct", "correct"), ("le_nums", "le_numbers_only")):
            r, secs = timed(explore, Bakery(N, K, var), liveness=True)
            emit(f"C-X3.liveness.n{N}_k{K}.{tag}.no_progress_state_found", r["stuck"] is not None)
            mm, where = recorded("sweep_live.txt", rf"N={N} K={K} {var}\s+atomic states=\s*([\d,]+) .*liveness: (.*?)\s+\(")
            check(f"C-X3 n{N}_k{K} {var} matches recording",
                  (num(mm.group(1)), mm.group(2).startswith("NO-PROGRESS")) == (r["states"], r["stuck"] is not None),
                  f"{where}; {secs:.1f}s")


ROWS = [row_cf1, row_cf2, row_cf3, row_cf4, row_cf5, row_cf6, row_cf7, row_cf8, row_cf9,
        row_cf10, row_cf11, row_cf12, row_cf13, row_cf14, row_cx1, row_cx2, row_cx3]

if __name__ == "__main__":
    for row in ROWS:
        t = time.time()
        row()
        note(f"{row.__name__} done in {time.time() - t:.1f}s")
    note(f"{len(_EMITTED)} keys; {_PASSED[0]} cross-checks passed, {len(_FAILED)} failed; "
         f"{time.time() - T0:.1f}s total")
    if _FAILED:
        note("FAILED: " + "; ".join(_FAILED))
        sys.exit(1)
