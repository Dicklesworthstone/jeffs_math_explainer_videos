"""RFC 6330 degree table vs the ideal soliton, Table 2 parameters (S, H, W, L, P), coupon-collector numbers,
and the small arithmetic used in the memo (memo sections 2.1, 2.2, 2.5, 2.6).

Usage:  python3 -I misc_facts.py [--rfc path/to/rfc6330.txt]     (stdlib only, < 1 s)
Without --rfc, the Table 2 part is skipped.
"""
import argparse, math, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

ap = argparse.ArgumentParser()
ap.add_argument('--rfc', help='path to rfc6330.txt (needed for the Table 2 part)')
args = ap.parse_args()

# RFC 6330 Section 5.3.5.2, Table 1
F = [0, 5243, 529531, 704294, 791675, 844104, 879057, 904023, 922747, 937311, 948962, 958494, 966438, 973160, 978921,
     983914, 988283, 992138, 995565, 998631, 1001391, 1003887, 1006157, 1008229, 1010129, 1011876, 1013490, 1014983,
     1016370, 1017662, 1048576]
N = 2 ** 20
p = {d: (F[d] - F[d - 1]) / N for d in range(1, 31)}
print("Table 1 vs ideal soliton 1/(d(d-1)):  d, mass (units of 2^-20), prob, ideal mass, ratio")
for d in range(1, 31):
    m = F[d] - F[d - 1]
    ideal = N / (d * (d - 1)) if d > 1 else None
    print(f"  {d:2d} {m:7d} {m/N:.6f} {('%.1f' % ideal) if ideal else '-':>9} {('%.4f' % (m/ideal)) if ideal else '-'}")
print("1/29 - P(1) =", 1 / 29 - p[1], " vs P(30) =", p[30])
print("degree probs d=1..6:", {d: round(p[d], 4) for d in range(1, 7)}, " P(d>=7)=", round(sum(p[d] for d in range(7, 31)), 4))
print("mean LT degree (W large):", sum(d * p[d] for d in p))
pd_lt4 = sum(p[d] for d in (1, 2, 3))   # d1 = 2 + Rand(X,3,2) if d < 4 else 2
print("P(d<4)=", pd_lt4, " E[d1]=", 2.5 * pd_lt4 + 2 * (1 - pd_lt4),
      " mean total symbols touched:", sum(d * p[d] for d in p) + 2.5 * pd_lt4 + 2 * (1 - pd_lt4))

if args.rfc:
    import rfc6330_data
    t2 = rfc6330_data.load(args.rfc)['table2']
    print("# K' values in Table 2:", len(t2))
    for Kp, J, S, H, W in t2:
        if Kp in (10, 26, 101, 1002, 10017, 56403):
            L = Kp + S + H; P = L - W
            print(f"K'={Kp}: S={S} H={H} W={W} L={L} P={P} U={P-H} sqrt(K')={math.sqrt(Kp):.1f}  "
                  f"(S+H)/K'={(S+H)/Kp*100:.2f}%  H/L={H/L*100:.3f}%")
    print("range of H over all K':", sorted(set(r[3] for r in t2)))
    for Kp, J, S, H, W in t2[::60] + [t2[-1]]:
        L = Kp + S + H
        print(f"  K'={Kp} (S+H)/K'={(S+H)/Kp*100:.2f}%  P={L-W}  sqrtK'={math.sqrt(Kp):.0f}")
else:
    print("(no --rfc given: skipping Table 2 parameters)")

# coupon collector
print("e^-5 =", math.exp(-5), " (1-1/K)^(5K) for K=10000:", (1 - 1 / 10000) ** 50000,
      " expected untouched of 10000:", 10000 * (1 - 1 / 10000) ** 50000)
print("e^-4.8166 (RFC mean degree, N=K) =", math.exp(-4.8166))
for K in (10, 100, 1000, 10000):
    HK = sum(1 / i for i in range(1, K + 1))
    print(f"random copies, K={K}: K*H_K = {K*HK:.1f} = {HK:.3f} x K")

# small numbers quoted in the memo
print("P(a packet reaches all 10^6 receivers) at 1% loss = exp(", 1e6 * math.log(0.99), ")")
print("50000^3 =", 50000 ** 3)
print("measured K=10 failure ratio K -> K+1 (6316/1e6 vs 24/1e6):", (6316 / 1e6) / (24 / 1e6))
print("binomial standard errors of the memo's Monte Carlo rates:")
for f, n in ((6316, 1e6), (24, 1e6), (1, 1e6), (1262, 2e5), (4, 2e5), (108, 2e4), (646, 1e5), (137, 2e4),
             (7066, 1e4), (702, 1e3)):
    q = f / n
    print(f"  {f}/{int(n)} = {q:.3e} +- {math.sqrt(q*(1-q)/n):.1e}")
