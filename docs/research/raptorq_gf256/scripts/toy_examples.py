"""Toy examples for the video: the article's 4-packet system, the 5-unknown peeling cascade,
inactivation of C, the 5-cycle (singular over GF(2), fixed by one GF(256) "twist"), and the
Z/256 row-collapse facts (memo sections 2.3-2.5).

Usage:  python3 -I toy_examples.py      (needs numpy; < 10 s)
"""
import os, random, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gf256np import rank256, rank2, MUL, INV


def mul(a, b):
    return int(MUL[a, b])


def div(a, b):
    return int(MUL[a, INV[b]])


A, B, C, D, E = 5, 3, 7, 2, 12
print("values A..E:", A, B, C, D, E, [format(v, '04b') for v in (A, B, C, D, E)])
print("article: A^C=", A ^ C, " B^C^D=", B ^ C ^ D, " A^B=", A ^ B, " A^B^C^D=", A ^ B ^ C ^ D)
M = np.array([[1, 0, 1, 0], [0, 1, 1, 1], [1, 1, 0, 0], [1, 1, 1, 1]])
print("article 4 packets rank over GF(2):", rank2(M))

pk = {'y1': (('C',), C), 'y2': (('A', 'C'), A ^ C), 'y3': (('B', 'C', 'D'), B ^ C ^ D),
      'y4': (('A', 'B'), A ^ B), 'y5': (('D', 'E'), D ^ E), 'y6': (('B', 'E'), B ^ E)}
for k, (s, v) in pk.items():
    print(k, '=', ' xor '.join(s), '=', v)


def peel(pk):
    eqs = {k: [set(s), v] for k, (s, v) in pk.items()}
    known = {}
    log = []
    while True:
        ripple = [k for k, (s, v) in eqs.items() if len(s) == 1]
        if not ripple:
            break
        k = sorted(ripple)[0]
        (sym,), v = eqs.pop(k)
        known[sym] = v
        log.append((k, sym, v, len(ripple)))
        for kk, e in eqs.items():
            if sym in e[0]:
                e[0].remove(sym)
                e[1] ^= v
        eqs = {kk: e for kk, e in eqs.items() if e[0]}
    return known, log, eqs


known, log, rest = peel(pk)
print("peel order (packet, symbol, value, ripple size):", log)
print("recovered:", known)
known, log, rest = peel({k: v for k, v in pk.items() if k != 'y1'})
print("without y1 -> peel log:", log, " stuck eqs:", {k: (sorted(s), v) for k, (s, v) in rest.items()})
# inactivation: treat C as unknown c. A = y2+c; B = y4+A = y4+y2+c; D = y3+B+c = y3+y4+y2; E = y5+D; y6 = B+E -> c
y2, y3, y4, y5, y6 = A ^ C, B ^ C ^ D, A ^ B, D ^ E, B ^ E
c = y6 ^ y5 ^ y3
print("inactivation: c = y6^y5^y3 =", y6, '^', y5, '^', y3, '=', c, " (C =", C, ")")
Ar = y2 ^ c; Br = y4 ^ Ar; Dr = y3 ^ y4 ^ y2; Er = y5 ^ Dr
print("back-substituted A,B,D,E:", Ar, Br, Dr, Er)

cyc = np.zeros((5, 5), dtype=np.uint8)
for i in range(5):
    cyc[i, i] = 1
    cyc[i, (i + 1) % 5] = 1
print("5-cycle (A+B,B+C,C+D,D+E,E+A) rank over GF(2):", rank2(cyc), " over GF(256):", rank256(cyc))
cyc2 = cyc.copy(); cyc2[4, 0] = 2
print("5-cycle with E+2A rank over GF(256):", rank256(cyc2))
y = [A ^ B, B ^ C, C ^ D, D ^ E, E ^ mul(2, A)]
print("cycle eq values (A+B,B+C,C+D,D+E,E+2A):", y, " 2*A =", mul(2, A))
S = y[0] ^ y[1] ^ y[2] ^ y[3]
print("S=y1+y2+y3+y4 =", S, " y5+S =", y[4] ^ S, " 1+2 =", 1 ^ 2, " A=(y5+S)/3 =", div(y[4] ^ S, 3))
random.seed(1); Ntr = 200000; cnt = 0
for _ in range(Ntr):
    pr = 1
    for i in range(5):
        pr = mul(pr, random.randint(1, 255))
    cnt += (pr == 1)
print("MC P(product of 5 random nonzero octets = 1) =", cnt / Ntr, " vs 1/255 =", 1 / 255)
tri = np.array([[1, 1, 0], [0, 1, 1], [1, 0, 1]], dtype=np.uint8)
print("triangle A+B,B+C,A+C rank GF(2):", rank2(tri), " GF(256):", rank256(tri))
print("Z/256 distinct products per row: a=1:", len({b % 256 for b in range(256)}), " a=2:",
      len({2 * b % 256 for b in range(256)}), " a=128:", len({128 * b % 256 for b in range(256)}),
      " a=3:", len({3 * b % 256 for b in range(256)}))
print("GF(256) nonzero rows all permutations:", all(len({int(MUL[a, b]) for b in range(256)}) == 256 for a in range(1, 256)))
print("sum 1/(2^i-1) =", sum(1 / (2 ** i - 1) for i in range(1, 80)), " sum 1/(256^i-1) =",
      sum(1 / (256 ** i - 1) for i in range(1, 20)))
