"""Cross-validate rq6330.py against the independent cberner/raptorq encoder (PyPI `raptorq` 2.0.0),
and check that the systematic matrix A (ISIs 0..K'-1) has full rank for the first N table values of K'.

Usage:  python -I rq_validate.py --rfc path/to/rfc6330.txt [--kcount 120]
Needs numpy and raptorq==2.0.0. With --kcount 120 (checks K' up to 1002) it is slow (minutes);
--kcount 30 finishes quickly.
"""
import argparse, os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gf256np import MUL, INV, rank256
import rq6330
from rq6330 import Params, build_A, enc_indices
import raptorq

ap = argparse.ArgumentParser()
ap.add_argument('--rfc', required=True, help='path to rfc6330.txt')
ap.add_argument('--kcount', type=int, default=120, help="number of smallest K' values to rank-check")
args = ap.parse_args()
rq6330.load(args.rfc)


def solve(A, Dm):
    A = A.copy(); Dm = Dm.copy(); m, n = A.shape; r = 0
    for c in range(n):
        nz = np.nonzero(A[r:, c])[0]
        if nz.size == 0:
            raise Exception("singular")
        p = r + nz[0]
        if p != r:
            A[[r, p]] = A[[p, r]]; Dm[[r, p]] = Dm[[p, r]]
        iv = INV[A[r, c]]; A[r] = MUL[iv][A[r]]; Dm[r] = MUL[iv][Dm[r]]
        rows = np.nonzero(A[:, c])[0]; rows = rows[rows != r]
        if rows.size:
            f = A[rows, c]
            A[rows] ^= MUL[f[:, None], A[r][None, :]]
            Dm[rows] ^= MUL[f[:, None], Dm[r][None, :]]
        r += 1
    return Dm[:n]


T = 8
for Kp in (10, 26, 101):
    pr = Params(Kp)
    A = build_A(pr, range(Kp))
    print(f"K'={Kp}: S={pr.S} H={pr.H} W={pr.W} L={pr.L} P={pr.P} U={pr.U} P1={pr.P1}; systematic A rank={rank256(A)} of {pr.L}")
    data = os.urandom(Kp * T)
    src = np.frombuffer(data, dtype=np.uint8).reshape(Kp, T)
    Dm = np.vstack([np.zeros((pr.S + pr.H, T), dtype=np.uint8), src])
    C = solve(A, Dm)   # intermediate symbols
    pk = raptorq.Encoder.with_defaults(data, T).get_encoded_packets(50)
    ok = 0; tot = 0
    for p in pk:   # packet = 1-byte SBN, 3-byte ESI, then the symbol
        esi = int.from_bytes(p[1:4], 'big'); sym = np.frombuffer(p[4:], dtype=np.uint8)
        mine = np.zeros(T, dtype=np.uint8)
        for i in enc_indices(pr, esi):
            mine ^= C[i]
        tot += 1; ok += np.array_equal(mine, sym)
    print(f"   my encoder vs cberner/raptorq packets: {ok}/{tot} byte-identical")
bad = []
ks = sorted(rq6330.TAB)[:args.kcount]
for Kp in ks:
    pr = Params(Kp)
    if rank256(build_A(pr, range(Kp))) < pr.L:
        bad.append(Kp)
print(f"systematic A full rank for first {len(ks)} K' values; failures: {bad}; max K' checked {ks[-1]}")
