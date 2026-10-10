#!/usr/bin/env python3
"""Bakery algorithm under an x86-TSO-style memory model (Owens/Sarkar/Sewell):
each process has a FIFO store buffer; a store is appended to the buffer; a load
returns the newest buffered store to that address in the process's OWN buffer if
any, else main memory; at any time a buffer may flush its oldest entry to memory;
MFENCE blocks until the process's own buffer is empty.

fences: subset of {"F1","F2"}:
  F1 = MFENCE right after  choosing[i] := 1   (before reading the other numbers)
  F2 = MFENCE right after  choosing[i] := 0   (before the L2/L3 wait loops)
Tickets capped at K (transitions writing a ticket > K are pruned); store buffers hold
at most B entries (a store waits while the buffer is full).  Both are
under-approximations: a violation found is real; "OK" means OK within the bounds."""
from collections import deque

NCS, F1, RD, WN, WC0, F2, L2, L3, CS = range(9)
NAMES = {NCS: "ncs", F1: "mfence#1", RD: "max-read", WN: "write-num", WC0: "choose:=0",
         F2: "mfence#2", L2: "L2", L3: "L3", CS: "CS"}


class TSOBakery:
    def __init__(self, N, K, fences=(), B=4):
        self.N, self.K, self.fences, self.B = N, K, set(fences), B   # B = store-buffer capacity
        self.pruned = 0

    def initial(self):
        N = self.N
        return (tuple((NCS, 0, 0) for _ in range(N)), (0,) * N, (0,) * N, tuple(() for _ in range(N)))

    def load(self, st, i, kind, j):
        for (k2, j2, v) in reversed(st[3][i]):      # store forwarding from own buffer
            if k2 == kind and j2 == j:
                return v
        return (st[1] if kind == "c" else st[2])[j]

    def successors(self, st):
        loc, ch, nu, buf = st
        N, K = self.N, self.K
        out = []

        def mk(i, newl, newbuf=None):
            l2 = list(loc); l2[i] = newl
            b2 = buf if newbuf is None else tuple(newbuf if k == i else buf[k] for k in range(N))
            return (tuple(l2), ch, nu, b2)

        for i in range(N):
            pc, a, m = loc[i]
            others = [j for j in range(N) if j != i]
            # 1) flush oldest buffered store of process i
            if buf[i]:
                (kind, j, v) = buf[i][0]
                ch2, nu2 = list(ch), list(nu)
                (ch2 if kind == "c" else nu2)[j] = v
                b2 = tuple(buf[i][1:] if k == i else buf[k] for k in range(N))
                out.append((i, f"flush {'choosing' if kind == 'c' else 'number'}[{j+1}]={v} to memory",
                            (loc, tuple(ch2), tuple(nu2), b2)))
            # 2) program step (a store is disabled while the buffer is full: bounded-buffer model)
            full = len(buf[i]) >= self.B
            if full and pc in (NCS, WN, WC0, CS):
                continue
            if pc == NCS:
                mask = sum(1 << j for j in others)
                out.append((i, "store choosing:=1 (buffered)",
                            mk(i, (F1 if "F1" in self.fences else RD, mask, 0), buf[i] + (("c", i, 1),))))
            elif pc in (F1, F2):
                if not buf[i]:
                    nxt = (RD, a, 0) if pc == F1 else (L2, others[0], 0)
                    out.append((i, "mfence", mk(i, nxt)))
            elif pc == RD:
                j = min(k for k in range(N) if a >> k & 1)
                v = self.load(st, i, "n", j)
                rem = a & ~(1 << j)
                nm = max(m, v)
                out.append((i, f"load number[{j+1}]={v}", mk(i, (RD, rem, nm) if rem else (WN, 0, nm))))
            elif pc == WN:
                t = m + 1
                if t > K:
                    self.pruned += 1
                    continue
                out.append((i, f"store number:={t} (buffered)", mk(i, (WC0, 0, 0), buf[i] + (("n", i, t),))))
            elif pc == WC0:
                nxt = (F2, 0, 0) if "F2" in self.fences else (L2, others[0], 0)
                out.append((i, "store choosing:=0 (buffered)", mk(i, nxt, buf[i] + (("c", i, 0),))))
            elif pc == L2:
                j = a
                v = self.load(st, i, "c", j)
                if v:
                    out.append((i, f"L2 load choosing[{j+1}]=1, wait", st))
                else:
                    out.append((i, f"L2 load choosing[{j+1}]=0, pass", mk(i, (L3, j, 0))))
            elif pc == L3:
                j = a
                nj = self.load(st, i, "n", j)
                ni = self.load(st, i, "n", i)
                if nj != 0 and (nj, j) < (ni, i):
                    out.append((i, f"L3 load number[{j+1}]={nj}, wait", st))
                else:
                    k = others.index(j)
                    if k + 1 < len(others):
                        out.append((i, f"L3 load number[{j+1}]={nj}, pass", mk(i, (L2, others[k + 1], 0))))
                    else:
                        out.append((i, f"L3 load number[{j+1}]={nj}, pass -> ENTER CS", mk(i, (CS, 0, 0))))
            elif pc == CS:
                out.append((i, "store number:=0 (exit, buffered)", mk(i, (NCS, 0, 0), buf[i] + (("n", i, 0),))))
        return out

    def in_cs(self, st):
        return [i for i in range(self.N) if st[0][i][0] == CS]


def explore_tso(model):
    init = model.initial()
    seen = {init: None}
    q = deque([init])
    bad = None
    while q:
        st = q.popleft()
        for (i, lab, ns) in model.successors(st):
            if ns not in seen:
                seen[ns] = (st, i, lab)
                q.append(ns)
                if bad is None and len(model.in_cs(ns)) >= 2:
                    bad = ns
    tr = []
    if bad is not None:
        s = bad
        while seen[s] is not None:
            p, i, lab = seen[s]
            tr.append((f"P{i+1}: {lab}", s))
            s = p
        tr.reverse()
    return len(seen), bad, tr


def fmt(model, st):
    loc, ch, nu, buf = st
    ps = " | ".join(f"P{i+1}:{NAMES[loc[i][0]]}" + (f"(j={loc[i][1]+1})" if loc[i][0] in (L2, L3) else "")
                    for i in range(model.N))
    bs = " ".join(f"buf{i+1}=[" + ",".join(f"{'c' if k == 'c' else 'n'}{j+1}={v}" for (k, j, v) in buf[i]) + "]"
                  for i in range(model.N))
    return f"{ps} || mem choosing={list(ch)} number={list(nu)} || {bs}"


if __name__ == "__main__":
    import sys
    # Default: N=2 only (seconds).  "--n3" adds the unfenced N=3 runs, which did NOT
    # finish within 10 minutes in pure Python; tso3.py covers the fenced N=3 case.
    for N, K in ((2, 4), (3, 3)) if "--n3" in sys.argv[1:] else ((2, 4),):
        for fences in ((), ("F1",), ("F2",), ("F1", "F2")):
            m = TSOBakery(N, K, fences)
            n, bad, tr = explore_tso(m)
            print(f"TSO N={N} K={K} fences={'+'.join(fences) or 'none':6s}: states={n:>9,} "
                  f"-> {'MUTUAL EXCLUSION VIOLATED (shortest trace %d steps)' % len(tr) if bad else 'OK'}",
                  flush=True)
            if bad and N == 2 and fences in ((), ("F1",), ("F2",)):
                for lab, s in tr:
                    print("      ", lab.ljust(42), fmt(m, s))
