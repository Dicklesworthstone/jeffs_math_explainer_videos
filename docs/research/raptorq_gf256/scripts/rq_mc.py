"""Monte Carlo decode-failure rate of a real RaptorQ decoder (cberner/raptorq, PyPI `raptorq` 2.0.0)
when it receives exactly K, K+1, K+2 random encoding symbols drawn from a pool of K source + R repair symbols.

Usage:  python -I rq_mc.py K trials R        e.g.  python -I rq_mc.py 10 1000000 20000
Use K equal to a K' value of RFC 6330 Table 2 (10, 101, 1002, ...) so no padding is involved.
The memo's K=10 run (10^6 trials per h) took ~15 min on a loaded machine; 20000 trials takes seconds.
"""
import os, random, sys, time
import raptorq

K = int(sys.argv[1]); trials = int(sys.argv[2]); R = int(sys.argv[3]); T = 8
random.seed(2026 + K)
data = os.urandom(K * T)
pool = raptorq.Encoder.with_defaults(data, T).get_encoded_packets(R)
t0 = time.time()
for h in (0, 1, 2):
    fails = 0
    for t in range(trials):
        sub = random.sample(pool, K + h)
        dec = raptorq.Decoder.with_defaults(len(data), T)
        r = None
        for p in sub:
            r = dec.decode(p)
            if r is not None:
                break
        if r != data:
            fails += 1
    print(f"K={K} recv=K+{h} trials={trials}: failures={fails} rate={fails/trials:.3e}", flush=True)
print("time", time.time() - t0)
