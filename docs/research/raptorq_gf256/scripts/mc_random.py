"""Monte Carlo sanity check of rank_probs.py: failure rate of random (k+h) x k matrices over GF(2) and GF(256).

Usage:  python3 -I mc_random.py [k] [trials]      (defaults: k=20, trials=20000; needs numpy)
The memo's MC (k=20, 20000 trials) took ~1 min on an unloaded machine.
"""
import os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gf256np import rank256, rank2

k = int(sys.argv[1]) if len(sys.argv) > 1 else 20
trials = int(sys.argv[2]) if len(sys.argv) > 2 else 20000
rng = np.random.default_rng(12345)
for h in range(0, 4):
    f2 = 0
    f256 = 0
    for t in range(trials):
        M2 = rng.integers(0, 2, size=(k + h, k), dtype=np.uint8)
        if rank2(M2) < k:
            f2 += 1
        M = rng.integers(0, 256, size=(k + h, k), dtype=np.uint8)
        if rank256(M) < k:
            f256 += 1
    print(f"k={k} h={h} trials={trials}: GF(2) fail rate={f2/trials:.4f}   GF(256) fail rate={f256/trials:.6f} ({f256} fails)")
