"""GF(256) arithmetic with numpy, in RaptorQ's representation (x^8+x^4+x^3+x^2+1 = 0x11D, alpha = 2).

The exp/log tables are computed from the polynomial; gf256_facts.py --rfc checks that they equal
RFC 6330's OCT_EXP / OCT_LOG tables entry for entry.
"""
import numpy as np


def _tables(poly=0x11D):
    exp = [0] * 510
    x = 1
    for i in range(510):
        exp[i] = x
        x <<= 1
        if x & 0x100:
            x ^= poly
    log = [0] * 256
    for i in range(255):
        log[exp[i]] = i
    return exp, log


_exp, _log = _tables()
EXP = np.array(_exp + _exp[:20], dtype=np.int32)   # padded so EXP[LOG[u]+LOG[v]] never overflows
LOG = np.array(_log, dtype=np.int32)                # LOG[0] is unused (log of 0 is undefined)
MUL = np.zeros((256, 256), dtype=np.uint8)          # full multiplication table
for _u in range(1, 256):
    MUL[_u, 1:] = EXP[LOG[_u] + LOG[1:]]
INV = np.zeros(256, dtype=np.uint8)
for _u in range(1, 256):
    INV[_u] = EXP[255 - LOG[_u]]


def rank256(M):
    """Rank of a matrix of octets over GF(256) (a 0/1 matrix gives its GF(2) rank)."""
    A = np.array(M, dtype=np.uint8).copy()
    m, n = A.shape
    r = 0
    for c in range(n):
        if r == m:
            break
        nz = np.nonzero(A[r:, c])[0]
        if nz.size == 0:
            continue
        p = r + nz[0]
        if p != r:
            A[[r, p]] = A[[p, r]]
        A[r] = MUL[INV[A[r, c]]][A[r]]
        rows = np.nonzero(A[:, c])[0]
        rows = rows[rows != r]
        if rows.size:
            f = A[rows, c]
            A[rows] ^= MUL[f[:, None], A[r][None, :]]
        r += 1
    return r


def rank2(M):
    """Rank over GF(2) of a 0/1 matrix."""
    A = (np.array(M, dtype=np.uint8) & 1).copy()
    m, n = A.shape
    r = 0
    for c in range(n):
        if r == m:
            break
        nz = np.nonzero(A[r:, c])[0]
        if nz.size == 0:
            continue
        p = r + nz[0]
        if p != r:
            A[[r, p]] = A[[p, r]]
        rows = np.nonzero(A[:, c])[0]
        rows = rows[rows != r]
        if rows.size:
            A[rows] ^= A[r][None, :]
        r += 1
    return r
