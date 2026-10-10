"""Ablation: rank failure of RaptorQ's matrix A with N = K'+h received symbols (uniformly random 24-bit ESIs),
with the H HDPC rows as specified (gf256) or replaced by random dense GF(2) / GF(256) rows.

Usage:  python3 -I rq_ablate.py --rfc path/to/rfc6330.txt K' trials mode h
        mode in {gf256, gf2random, gf256random, gf2bits}
        e.g.  python3 -I rq_ablate.py --rfc rfc6330.txt 10 10000 gf2random 0
Needs numpy. Roughly 3-10 ms per trial at K'=10 and ~0.1 s at K'=101 on an unloaded machine.
"""
import argparse, os, sys, time
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gf256np import rank256
import rq6330
from rq6330 import Params, precode_rows, lt_row

ap = argparse.ArgumentParser()
ap.add_argument('--rfc', required=True, help='path to rfc6330.txt')
ap.add_argument('Kp', type=int)
ap.add_argument('trials', type=int)
ap.add_argument('mode', choices=['gf256', 'gf2random', 'gf256random', 'gf2bits'])
ap.add_argument('h', type=int)
args = ap.parse_args()
rq6330.load(args.rfc)
Kp, trials, mode, h = args.Kp, args.trials, args.mode, args.h
pr = Params(Kp)
rng = np.random.default_rng(1000 * Kp + 17 * h + len(mode))
pre_fixed = precode_rows(pr)
fails = 0; t0 = time.time()
for t in range(trials):
    esis = rng.choice(1 << 24, size=Kp + h, replace=False)
    pre = pre_fixed if mode == 'gf256' else precode_rows(pr, hdpc_mode=mode, rng=rng)
    A = np.vstack([pre] + [lt_row(pr, int(X)) for X in esis])
    if rank256(A) < pr.L:
        fails += 1
print(f"K'={Kp} L={pr.L} H={pr.H} mode={mode} N=K'+{h} trials={trials}: fails={fails} rate={fails/trials:.3e} "
      f"time={time.time()-t0:.0f}s", flush=True)
