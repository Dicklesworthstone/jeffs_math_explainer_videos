"""Exhaustive checks of the GF(256) facts used in the memo (sections 2.1 and 2.4).

Usage:  python3 -I gf256_facts.py [--rfc path/to/rfc6330.txt]
With --rfc, also checks that RFC 6330's OCT_EXP / OCT_LOG tables are exactly the powers of x
modulo 0x11D and that table multiplication equals polynomial multiplication for all 65,536 pairs.
"""
import argparse, math, os, sys
from collections import Counter
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

ap = argparse.ArgumentParser()
ap.add_argument('--rfc', help='path to rfc6330.txt (optional)')
args = ap.parse_args()


def clmul(a, b):
    r = 0
    while b:
        if b & 1:
            r ^= a
        a <<= 1
        b >>= 1
    return r


def pmod(a, p):
    dp = p.bit_length() - 1
    while a and a.bit_length() - 1 >= dp:
        a ^= p << (a.bit_length() - 1 - dp)
    return a


def mul(a, b, p=0x11D):
    return pmod(clmul(a, b), p)


def poly(a):
    t = [(f"x^{i}" if i > 1 else ("x" if i == 1 else "1")) for i in range(a.bit_length() - 1, -1, -1) if a >> i & 1]
    return " + ".join(t) if t else "0"


# powers of alpha = 2 under 0x11D
x = 1
EXP = []
for i in range(510):
    EXP.append(x)
    x = mul(x, 2)
LOGD = {EXP[i]: i for i in range(255)}
print("OCT_EXP[0..12] (computed):", EXP[:13])
print("distinct values among first 255 powers of 2:", len(set(EXP[:255])))

if args.rfc:
    import rfc6330_data
    d = rfc6330_data.load(args.rfc)
    print("RFC OCT_EXP matches 2^i mod 0x11D for all 510 entries:", d['exp'] == EXP)
    print("RFC OCT_LOG matches discrete log:", all(d['log'][u - 1] == LOGD[u] for u in range(1, 256)))
    R_EXP, R_LOG = d['exp'], d['log']

    def tmul(u, v):
        return 0 if u == 0 or v == 0 else R_EXP[R_LOG[u - 1] + R_LOG[v - 1]]
    print("RFC table mul == poly mul for all 65536 pairs:",
          all(tmul(u, v) == mul(u, v) for u in range(256) for v in range(256)))
else:
    print("(no --rfc given: skipping comparison with RFC 6330 tables)")

# worked examples
ex = [(2, 128), (3, 7), (0x57, 0x83), (0x53, 0xCA), (2, 142), (16, 16), (255, 255), (5, 9), (200, 100), (7, 7), (128, 3), (150, 3)]
for a, b in ex:
    print(f"{a}*{b}: 0x11D -> {mul(a, b, 0x11D)} (0x{mul(a, b, 0x11D):02X}); 0x11B(AES) -> {mul(a, b, 0x11B)} "
          f"(0x{mul(a, b, 0x11B):02X}); carryless={clmul(a, b)}; int mod 256={(a * b) % 256}")
print("poly 0x57:", poly(0x57), " 0x83:", poly(0x83), " clmul:", poly(clmul(0x57, 0x83)), "=0x%X" % clmul(0x57, 0x83))
print("0x31:", poly(0x31), " 0xC1:", poly(0xC1))
print("29:", poly(29), " 0x11D:", poly(0x11D), " 0x11B:", poly(0x11B))


def inv(a, p=0x11D):
    for b in range(1, 256):
        if mul(a, b, p) == 1:
            return b


print("inverse of 2 (0x11D):", inv(2), "check", mul(2, inv(2)), " via OCT_EXP[255-log 2]:", EXP[255 - LOGD[2]])
print("inverse of 3 (0x11D):", inv(3), " inverse of 2 (AES 0x11B):", inv(2, 0x11B))
print("inverse of 0x53 under 0x11D:", hex(inv(0x53)), " under 0x11B:", hex(inv(0x53, 0x11B)))
print("logs: log 2 =", LOGD[2], " log 3 =", LOGD[3], " log 29 =", LOGD[29], " log 128 =", LOGD[128],
      " log 150 =", LOGD[150], " OCT_EXP[205] =", EXP[205])

# Z/256 facts
print("Z/256: 2*128 mod 256 =", (2 * 128) % 256, "; # units:", sum(1 for a in range(256) if math.gcd(a, 256) == 1),
      "; # nonzero non-units:", sum(1 for a in range(1, 256) if math.gcd(a, 256) != 1))
print("does 2 have an inverse mod 256?", any((2 * b) % 256 == 1 for b in range(256)))
print("Z/256 distinct products per row: a=1:", len({b % 256 for b in range(256)}), " a=2:",
      len({2 * b % 256 for b in range(256)}), " a=128:", len({128 * b % 256 for b in range(256)}))
print("GF(256) nonzero rows are all permutations:", all(len({mul(a, b) for b in range(256)}) == 256 for a in range(1, 256)))


# irreducible & primitive counts
def is_irred(p):
    d = p.bit_length() - 1
    for q in range(2, 1 << (d // 2 + 1)):
        if 1 <= q.bit_length() - 1 <= d // 2 and pmod(p, q) == 0:
            return False
    return True


def order_of(g, p):
    x = g
    k = 1
    while x != 1:
        x = mul(x, g, p)
        k += 1
        if k > 300:
            return None
    return k


irr = [p for p in range(256, 512) if is_irred(p)]
prim = [p for p in irr if order_of(2, p) == 255]
print("# irreducible deg-8 over GF(2):", len(irr), "; # primitive (x has order 255):", len(prim))
print("0x11D irreducible?", 0x11D in irr, "primitive?", 0x11D in prim, "; 0x11B irreducible?", 0x11B in irr,
      "primitive?", 0x11B in prim)
print("order of 2 under 0x11B:", order_of(2, 0x11B), "; order of 3 under 0x11B:", order_of(3, 0x11B),
      "; order of 2 under 0x11D:", order_of(2, 0x11D))
print("# primitive elements in GF(256):", sum(1 for g in range(1, 256) if order_of(g, 0x11D) == 255),
      " phi(255) =", sum(1 for k in range(1, 256) if math.gcd(k, 255) == 1))
print("orders histogram (0x11D):", sorted(Counter(order_of(g, 0x11D) for g in range(1, 256)).items()))


def pw(a, n, p=0x11D):
    r = 1
    for _ in range(n):
        r = mul(r, a, p)
    return r


print("GF(16) subfield:", [a for a in range(256) if pw(a, 16) == a])
print("GF(4) subfield:", [a for a in range(256) if pw(a, 4) == a])


def xtime(a):
    return ((a << 1) & 0xFF) ^ (0x1D if a & 0x80 else 0)


print("xtime == multiply by 2 for all bytes:", all(xtime(a) == mul(a, 2) for a in range(256)))
print("log-ordered table is Hankel (entry depends on i+j only):",
      all(mul(EXP[i], EXP[j]) == EXP[(i + j) % 255] for i in range(255) for j in range(255)))
print("a^255 == 1 for all nonzero a:", all(pw(a, 255) == 1 for a in range(1, 256)))
print("freshman's dream (a+b)^2 = a^2+b^2 for all pairs:",
      all(mul(a ^ b, a ^ b) == mul(a, a) ^ mul(b, b) for a in range(256) for b in range(256)))

for P, name in ((0x11D, '0x11D'), (0x11B, '0x11B')):
    c = clmul(0x57, 0x83)
    print(f"reduction steps for 0x57*0x83 mod {name}:")
    while c.bit_length() - 1 >= 8:
        sh = c.bit_length() - 1 - 8
        print("   ", poly(c), " XOR x^%d*p(x) = %s" % (sh, poly(P << sh)))
        c ^= P << sh
    print("    result:", poly(c), "=", c, hex(c))
