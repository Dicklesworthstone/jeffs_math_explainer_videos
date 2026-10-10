"""Independent Python implementation of the RFC 6330 constraint matrix A (Sections 5.3.3-5.3.5).

Call load(path_to_rfc6330_txt) once before use (it supplies Table 2 and the V0..V3 random tables).
Cross-validated byte-exact against the cberner/raptorq encoder by rq_validate.py.

precode_rows(..., hdpc_mode=...) supports the ablation in rq_ablate.py:
  'gf256'        RFC HDPC rows G_HDPC = MT * GAMMA (the real code)
  'gf2random'    the H HDPC rows replaced by dense uniformly random 0/1 rows
  'gf256random'  the H HDPC rows replaced by dense uniformly random octets
  'gf2bits'      every nonzero octet of G_HDPC replaced by 1
"""
import os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gf256np import MUL, EXP
import rfc6330_data

V = None
TAB = None


def load(rfc_path):
    global V, TAB
    d = rfc6330_data.load(rfc_path)
    V = d['V']
    TAB = {r[0]: r for r in d['table2']}


# RFC 6330 Section 5.3.5.2, Table 1
F = [0, 5243, 529531, 704294, 791675, 844104, 879057, 904023, 922747, 937311, 948962, 958494, 966438, 973160, 978921,
     983914, 988283, 992138, 995565, 998631, 1001391, 1003887, 1006157, 1008229, 1010129, 1011876, 1013490, 1014983,
     1016370, 1017662, 1048576]


def is_prime(n):
    if n < 2:
        return False
    i = 2
    while i * i <= n:
        if n % i == 0:
            return False
        i += 1
    return True


def rand(y, i, m):  # Section 5.3.5.1
    x0 = (y + i) % 256
    x1 = ((y >> 8) + i) % 256
    x2 = ((y >> 16) + i) % 256
    x3 = ((y >> 24) + i) % 256
    return (V[0][x0] ^ V[1][x1] ^ V[2][x2] ^ V[3][x3]) % m


class Params:  # Section 5.3.3.3
    def __init__(s, Kp):
        if TAB is None:
            raise RuntimeError("call rq6330.load(path_to_rfc6330_txt) first")
        _, J, S, H, W = TAB[Kp]
        s.Kp = Kp; s.J = J; s.S = S; s.H = H; s.W = W
        s.L = Kp + S + H; s.P = s.L - W; s.U = s.P - H; s.B = W - S
        p = s.P
        while not is_prime(p):
            p += 1
        s.P1 = p


def deg(v, W):  # Section 5.3.5.2
    for d in range(1, 31):
        if F[d - 1] <= v < F[d]:
            return min(d, W - 2)


def tuple_(pr, X):  # Section 5.3.5.4
    A = 53591 + pr.J * 997
    if A % 2 == 0:
        A += 1
    B = 10267 * (pr.J + 1)
    y = (B + X * A) % (1 << 32)
    v = rand(y, 0, 1 << 20)
    d = deg(v, pr.W)
    a = 1 + rand(y, 1, pr.W - 1)
    b = rand(y, 2, pr.W)
    d1 = 2 + rand(X, 3, 2) if d < 4 else 2
    a1 = 1 + rand(X, 4, pr.P1 - 1)
    b1 = rand(X, 5, pr.P1)
    return d, a, b, d1, a1, b1


def enc_indices(pr, X):  # Section 5.3.5.3: indices of intermediate symbols XORed into encoding symbol X
    d, a, b, d1, a1, b1 = tuple_(pr, X)
    idx = [b]
    for j in range(1, d):
        b = (b + a) % pr.W
        idx.append(b)
    while b1 >= pr.P:
        b1 = (b1 + a1) % pr.P1
    idx.append(pr.W + b1)
    for j in range(1, d1):
        b1 = (b1 + a1) % pr.P1
        while b1 >= pr.P:
            b1 = (b1 + a1) % pr.P1
        idx.append(pr.W + b1)
    return idx


def lt_row(pr, X):
    r = np.zeros(pr.L, dtype=np.uint8)
    for i in enc_indices(pr, X):
        r[i] ^= 1
    return r


def precode_rows(pr, hdpc_mode='gf256', rng=None):
    S, H, B, W, P, L, Kp = pr.S, pr.H, pr.B, pr.W, pr.P, pr.L, pr.Kp
    A = np.zeros((S + H, L), dtype=np.uint8)
    # LDPC rows (Section 5.3.3.3)
    for i in range(B):
        a = 1 + i // S
        b = i % S
        A[b, i] ^= 1
        b = (b + a) % S
        A[b, i] ^= 1
        b = (b + a) % S
        A[b, i] ^= 1
    for i in range(S):
        A[i, B + i] ^= 1
        a = i % P
        b = (i + 1) % P
        A[i, W + a] ^= 1
        A[i, W + b] ^= 1
    # HDPC rows: MT (H x (K'+S)) * GAMMA ((K'+S) x (K'+S))
    n = Kp + S
    MT = np.zeros((H, n), dtype=np.uint8)
    for j in range(n - 1):
        r1 = rand(j + 1, 6, H)
        r2 = (rand(j + 1, 6, H) + rand(j + 1, 7, H - 1) + 1) % H
        MT[r1, j] = 1
        MT[r2, j] = 1
    for i in range(H):
        MT[i, n - 1] = EXP[i]
    # G = MT*GAMMA via the recurrence G[:,j] = MT[:,j] + alpha * G[:,j+1]
    G = np.zeros((H, n), dtype=np.uint8)
    acc = np.zeros(H, dtype=np.uint8)
    for j in range(n - 1, -1, -1):
        acc = MUL[2][acc] ^ MT[:, j]
        G[:, j] = acc
    if hdpc_mode == 'gf2random':
        G = rng.integers(0, 2, size=G.shape).astype(np.uint8)
    elif hdpc_mode == 'gf256random':
        G = rng.integers(0, 256, size=G.shape).astype(np.uint8)
    elif hdpc_mode == 'gf2bits':
        G = (G != 0).astype(np.uint8)
    A[S:S + H, :n] = G
    for i in range(H):
        A[S + i, n + i] = 1
    return A


def build_A(pr, isis, **kw):
    pre = precode_rows(pr, **kw)
    rows = [lt_row(pr, X) for X in isis]
    return np.vstack([pre] + rows) if rows else pre
