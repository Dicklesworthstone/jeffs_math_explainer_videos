#!/usr/bin/env python3
"""
bakery_mc.py -- exhaustive interleaving explorer for Lamport's Bakery algorithm
and broken variants.  Pure Python 3, no dependencies.

Process ids are 1..N (as in Lamport 1974); internally 0..N-1.
Each shared register choosing[i], number[i] is written only by process i.

Program for process i (Lamport, CACM 17(8):453-455, 1974):
  L1: choosing[i] := 1
      number[i]   := 1 + max(number[1..N])    -- each number[k] read separately,
                                                 in any order
      choosing[i] := 0
      for j := 1..N (j != i):
        L2: while choosing[j] != 0: skip
        L3: while number[j] != 0 and (number[j], j) < (number[i], i): skip
      CRITICAL SECTION
      number[i] := 0
      (back to non-critical section)

Register semantics
  atomic : every read and every write is one indivisible step.
  safe   : every WRITE is two steps (begin, end).  A read performed while the
           register is being written may return ANY value of the register's
           domain (number: 0..K, choosing: {0,1}).  A read that does not overlap a
           write returns the last committed value.  Modelling reads as points and
           writes as intervals is equivalent to Lamport's safe-register definition
           for single-writer registers: a read interval overlaps a write iff some
           point inside it does, and the reader does nothing else during its read.

Bounding: tickets are capped at K.  A transition that would write a ticket > K
is PRUNED (and counted).  "Verified" therefore means: for every execution in
which no ticket ever exceeds K.

Variants
  correct      : Lamport's algorithm
  no_choosing  : delete choosing[] and the L2 wait
  strict_nums  : L3 compares numbers only: wait while number[j]!=0 and number[j] <  number[i]
  le_nums      : L3 compares numbers only: wait while number[j]!=0 and number[j] <= number[i]
  double_fetch : correct, except max is computed as
                     t := 0; for k: if number[k] > t then t := number[k]
                 where the two occurrences of number[k] are SEPARATE reads.
                 (Lamport, "Reasoning About Nonatomic Operations", POPL 1983:
                  "not a valid implementation" for n > 2.)
"""
from collections import deque

NCS, RD, RD2, WN, WC0, L2, L3, CS = range(8)
PCNAME = {NCS: "ncs", RD: "max-read", RD2: "max-refetch", WN: "write-num",
          WC0: "choose:=0", L2: "L2", L3: "L3", CS: "CS"}


class Bakery:
    def __init__(self, N, K, variant="correct", safe=False, any_order=True, fcfs=False):
        assert variant in ("correct", "no_choosing", "strict_nums", "le_nums", "double_fetch")
        self.N, self.K, self.variant, self.safe = N, K, variant, safe
        self.any_order, self.fcfs = any_order, fcfs
        self.use_choosing = variant != "no_choosing"
        self.pruned_states = set()

    # ---- state layout -------------------------------------------------------
    # local[i] = (pc, a, m, wp, rest)
    #   RD : a = bitmask of indices still to read, m = max so far
    #   RD2: a = index being re-fetched, m = t so far, rest = mask still to read
    #   WN : m = max read
    #   L2/L3: a = current j
    #   wp = 1 iff a (safe-mode) write has begun but not ended
    # global = (locals, choosing, number, ahead)   ahead = FCFS ghost bits
    def initial(self):
        N = self.N
        return (tuple((NCS, 0, 0, 0, 0) for _ in range(N)), (0,) * N, (0,) * N, 0)

    def others(self, i):
        return [j for j in range(self.N) if j != i]

    def _writing_choosing(self, st, j):
        pc, a, m, wp, r = st[0][j]
        return wp == 1 and pc in (NCS, WC0)

    def _writing_number(self, st, j):
        pc, a, m, wp, r = st[0][j]
        return wp == 1 and pc in (WN, CS)

    def read_choosing(self, st, j):
        if self.safe and self._writing_choosing(st, j):
            return [0, 1]
        return [st[1][j]]

    def read_number(self, st, j):
        if self.safe and self._writing_number(st, j):
            return list(range(self.K + 1))
        return [st[2][j]]

    def must_wait(self, nj, j, ni, i):
        if nj == 0:
            return False
        if self.variant == "strict_nums":
            return nj < ni
        if self.variant == "le_nums":
            return nj <= ni
        return (nj, j) < (ni, i)

    def _first_wait(self, i):
        js = self.others(i)
        return (L2 if self.use_choosing else L3, js[0]) if js else (CS, 0)

    def _next_wait(self, i, j):
        js = self.others(i)
        k = js.index(j)
        if k + 1 < len(js):
            return (L2 if self.use_choosing else L3, js[k + 1])
        return (CS, 0)

    def _mark_ahead(self, st, i, ahead):
        """i begins its doorway: every process already in the bakery is ahead of i."""
        if not self.fcfs:
            return ahead
        N = self.N
        for k in self.others(i):
            if st[0][k][0] in (L2, L3):
                ahead |= 1 << (k * N + i)
        return ahead

    def successors(self, st):
        loc, ch, nu, ahead = st
        N, K = self.N, self.K
        out = []

        def mk(i, newl, ch2=ch, nu2=nu, ah2=ahead):
            l2 = list(loc)
            l2[i] = newl
            return (tuple(l2), ch2, nu2, ah2)

        def setv(t, i, v):
            t2 = list(t)
            t2[i] = v
            return tuple(t2)

        for i in range(N):
            pc, a, m, wp, rest = loc[i]
            allmask = sum(1 << j for j in self.others(i))
            if pc == NCS:
                if self.use_choosing:
                    if self.safe and not wp:
                        out.append((i, "begin choosing:=1",
                                    mk(i, (NCS, 0, 0, 1, 0), ah2=self._mark_ahead(st, i, ahead))))
                    else:
                        ah = ahead if self.safe else self._mark_ahead(st, i, ahead)
                        lab = "end choosing:=1" if self.safe else "choosing:=1"
                        out.append((i, lab, mk(i, (RD, allmask, 0, 0, 0), ch2=setv(ch, i, 1), ah2=ah)))
                else:
                    out.append((i, "start", mk(i, (RD, allmask, 0, 0, 0),
                                                 ah2=self._mark_ahead(st, i, ahead))))
            elif pc == RD:
                if a == 0:
                    out.append((i, f"max={m}", mk(i, (WN, 0, m, 0, 0))))
                    continue
                idxs = [j for j in range(N) if a >> j & 1]
                if not self.any_order:
                    idxs = idxs[:1]
                for j in idxs:
                    for v in self.read_number(st, j):
                        if self.variant == "double_fetch" and v > m:
                            out.append((i, f"read number[{j+1}]={v} (> t: re-fetch)",
                                        mk(i, (RD2, j, m, 0, a & ~(1 << j)))))
                        else:
                            rem = a & ~(1 << j)
                            nm = max(m, v)
                            # last read of the max loop goes straight to "write number"
                            nl = (RD, rem, nm, 0, 0) if rem else (WN, 0, nm, 0, 0)
                            out.append((i, f"read number[{j+1}]={v}", mk(i, nl)))
            elif pc == RD2:
                for v in self.read_number(st, a):
                    nl = (RD, rest, v, 0, 0) if rest else (WN, 0, v, 0, 0)
                    out.append((i, f"re-fetch number[{a+1}]={v} -> t", mk(i, nl)))
            elif pc == WN:
                t = m + 1
                if t > K:
                    self.pruned_states.add(st)
                    continue
                if self.safe and not wp:
                    out.append((i, f"begin number[{i+1}]:={t}", mk(i, (WN, 0, m, 1, 0))))
                else:
                    lab = f"{'end ' if self.safe else ''}number[{i+1}]:={t}"
                    nu2 = setv(nu, i, t)
                    if self.use_choosing:
                        out.append((i, lab, mk(i, (WC0, 0, 0, 0, 0), nu2=nu2)))
                    else:
                        p2, j2 = self._first_wait(i)
                        out.append((i, lab, mk(i, (p2, j2, 0, 0, 0), nu2=nu2)))
            elif pc == WC0:
                if self.safe and not wp:
                    out.append((i, "begin choosing:=0", mk(i, (WC0, 0, 0, 1, 0))))
                else:
                    p2, j2 = self._first_wait(i)
                    lab = "end choosing:=0" if self.safe else "choosing:=0"
                    out.append((i, lab, mk(i, (p2, j2, 0, 0, 0), ch2=setv(ch, i, 0))))
            elif pc == L2:
                j = a
                for v in sorted(set(self.read_choosing(st, j))):
                    if v != 0:
                        out.append((i, f"L2: choosing[{j+1}]={v}, wait", st))
                    else:
                        out.append((i, f"L2: choosing[{j+1}]=0, pass", mk(i, (L3, j, 0, 0, 0))))
            elif pc == L3:
                j = a
                ni = nu[i]
                for v in sorted(set(self.read_number(st, j))):
                    if self.must_wait(v, j, ni, i):
                        out.append((i, f"L3: number[{j+1}]={v}, wait", st))
                    else:
                        p2, j2 = self._next_wait(i, j)
                        lab = f"L3: number[{j+1}]={v}, pass"
                        ah = ahead
                        if p2 == CS:
                            lab += " -> ENTER CS"
                        out.append((i, lab, mk(i, (p2, j2, 0, 0, 0), ah2=ah)))
            elif pc == CS:
                if self.safe and not wp:
                    out.append((i, f"begin number[{i+1}]:=0 (exit)", mk(i, (CS, 0, 0, 1, 0))))
                else:
                    lab = f"{'end ' if self.safe else ''}number[{i+1}]:=0 (exit)"
                    out.append((i, lab, mk(i, (NCS, 0, 0, 0, 0), nu2=setv(nu, i, 0))))
        return out

    def in_cs(self, st):
        return [i for i in range(self.N) if st[0][i][0] == CS and st[0][i][3] == 0]

    def fcfs_check(self, prev, st, i, lab):
        """Returns (violation or None, new ahead bits) when process i enters CS."""
        N = self.N
        ah = st[3]
        if "ENTER CS" not in lab:
            return None, ah
        viol = None
        for k in range(N):
            if k != i and (ah >> (k * N + i)) & 1:
                viol = (k, i)
        for k in range(N):           # i is no longer ahead of anybody
            ah &= ~(1 << (i * N + k))
        return viol, ah


def explore(model, check_assert2=False, liveness=False):
    init = model.initial()
    index = {init: 0}
    states = [init]
    parent = [None]
    edges = [] if liveness else None
    entries = set()
    q = deque([0])
    bad = bad_kind = None
    a2_checked = 0
    while q:
        sid = q.popleft()
        st = states[sid]
        succ_ids = []
        for (i, lab, ns) in model.successors(st):
            if model.fcfs:
                viol, ah = model.fcfs_check(st, ns, i, lab)
                ns = ns[:3] + (ah,)
            else:
                viol = None
            new = ns not in index
            if new:
                index[ns] = len(states)
                states.append(ns)
                parent.append((sid, i, lab))
                q.append(index[ns])
            nid = index[ns]
            succ_ids.append(nid)
            if "ENTER CS" in lab:
                entries.add(sid)
            if bad is None:
                cs = model.in_cs(ns)
                if len(cs) >= 2:
                    bad, bad_kind = nid, "MUTUAL EXCLUSION VIOLATED"
                elif viol:
                    bad, bad_kind = nid, f"FCFS VIOLATED (P{viol[0]+1} was ahead of P{viol[1]+1})"
                elif check_assert2 and new and cs:
                    a2_checked += 1
                    c = cs[0]
                    for k in model.others(c):
                        if ns[0][k][0] in (L2, L3) and not ((ns[2][c], c) < (ns[2][k], k)):
                            bad, bad_kind = nid, "ASSERTION 2 VIOLATED"
        if liveness:
            edges.append(succ_ids)
    res = {"states": len(states), "bad": bad, "bad_kind": bad_kind,
           "pruned": len(model.pruned_states), "a2_checked": a2_checked}
    if bad is not None:
        res["trace"] = _trace(parent, states, bad, model)
    if liveness:
        n = len(states)
        rev = [[] for _ in range(n)]
        for s in range(n):
            for t in edges[s]:
                rev[t].append(s)
        good = [False] * n
        dq = deque()
        for s in range(n):
            if s in entries or states[s] in model.pruned_states:
                good[s] = True
                dq.append(s)
        while dq:
            t = dq.popleft()
            for s in rev[t]:
                if not good[s]:
                    good[s] = True
                    dq.append(s)
        stuck = None
        for s in range(n):
            st = states[s]
            trying = any(st[0][i][0] not in (NCS, CS) for i in range(model.N))
            if trying and not model.in_cs(st) and not good[s]:
                stuck = s
                break
        res["stuck"] = stuck
        if stuck is not None:
            res["stuck_trace"] = _trace(parent, states, stuck, model)
    return res


def fmt_state(model, st):
    parts = []
    for i in range(model.N):
        pc, a, m, wp, rest = st[0][i]
        s = PCNAME[pc]
        if pc in (L2, L3):
            s += f"(j={a+1})"
        if pc in (RD, WN, RD2):
            s += f"(m={m})"
        if wp:
            s += "[writing]"
        parts.append(f"P{i+1}:{s}")
    return " | ".join(parts) + f" || choosing={list(st[1])} number={list(st[2])}"


def _trace(parent, states, sid, model):
    steps = []
    while parent[sid] is not None:
        p, i, lab = parent[sid]
        steps.append((i, lab, states[sid]))
        sid = p
    steps.reverse()
    return [(f"P{i+1}: {lab}", fmt_state(model, st)) for (i, lab, st) in steps]


def replay(model, schedule, verbose=True):
    """schedule: list of (proc_1based, label_substring); each step must match exactly
    one enabled transition of that process.  Raises if the step is illegal."""
    st = model.initial()
    rows = []
    for (p, sub) in schedule:
        i = p - 1
        en = [(lab, ns) for (pi, lab, ns) in model.successors(st) if pi == i]
        cands = [(lab, ns) for (lab, ns) in en if sub in lab]
        if len(cands) != 1:
            raise RuntimeError(f"step P{p} '{sub}': {len(cands)} matches; enabled={[l for l, _ in en]}")
        lab, st = cands[0]
        rows.append((f"P{p}: {lab}", fmt_state(model, st), model.in_cs(st)))
    if verbose:
        for k, (a, b, cs) in enumerate(rows, 1):
            flag = "   <== TWO PROCESSES IN CS" if len(cs) >= 2 else ""
            print(f"  {k:2d}. {a:40s} {b}{flag}")
    return st, rows
