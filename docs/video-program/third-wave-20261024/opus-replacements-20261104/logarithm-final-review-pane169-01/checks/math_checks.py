#!/usr/bin/env python3
"""Recompute the on-screen numbers of sheets 226-339 from first principles. Standard library only.

This is the reviewer's own arithmetic. It does not read the film's facts or oracle.
X0 below is inferred from pixels (sheets 285-289); it is not printed on the card.
"""

import cmath
import math

FPS = 60
checks = 0


def near(name, got, want, tol=5e-4):
    global checks
    checks += 1
    assert abs(got - want) <= tol, f"{name}: got {got}, want {want}"
    print(f"PASS {name}: {got:.4f}")


def mul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(2)) for j in range(2)] for i in range(2)]


def act(a, v):
    return [a[0][0] * v[0] + a[0][1] * v[1], a[1][0] * v[0] + a[1][1] * v[1]]


def expm(a, terms=40):
    total = [[1.0, 0.0], [0.0, 1.0]]
    term = [[1.0, 0.0], [0.0, 1.0]]
    for n in range(1, terms):
        term = [[x / n for x in row] for row in mul(term, a)]
        total = [[total[i][j] + term[i][j] for j in range(2)] for i in range(2)]
    return total


SAMPLE = [[0.3, -1.0], [0.8, -0.1]]
U = [1.0, 1.0]

# sheet 234 and 281: the sample flow
E = expm(SAMPLE)
for name, got, want in zip(("e^X 11", "e^X 12", "e^X 21", "e^X 22"), sum(E, []), (0.905, -0.970, 0.776, 0.517)):
    near(name, got, want, 6e-4)
near("Xu x", act(SAMPLE, U)[0], -0.7)
near("e^X u y", act(E, U)[1], 1.293, 6e-4)
near("X e^X u = e^X X u, x", act(SAMPLE, act(E, U))[0], act(E, act(SAMPLE, U))[0], 1e-9)
near("X e^X u = e^X X u, y", act(SAMPLE, act(E, U))[1], act(E, act(SAMPLE, U))[1], 1e-9)

# sheets 268-270: signed area under the sample flow, trace 0.2
for t, shown in ((1, 1.221), (3, 1.822), (-2, 0.670)):
    near(f"det e^(tX) t={t}", math.exp(0.2 * t), shown, 6e-4)

# sheet 250: turning rate of the sample flow
trace = SAMPLE[0][0] + SAMPLE[1][1]
det = SAMPLE[0][0] * SAMPLE[1][1] - SAMPLE[0][1] * SAMPLE[1][0]
omega = math.sqrt(det - (trace / 2) ** 2)
near("turning rate", omega, 0.872, 3e-4)
near("turning rate / pi", omega / math.pi, 0.28, 3e-3)

# sheets 276, 285-289: A and the concrete X the picture uses
A = [[-2.0, 0.0], [0.0, -0.5]]
near("det A", A[0][0] * A[1][1], 1.0)
X0 = [[0.4, 0.6], [-0.5, -0.3]]
au, xu = act(A, U), act(X0, U)
first_a, first_x = act(X0, au), act(A, xu)
near("Au x", au[0], -2.0)
near("Au y", au[1], -0.5)
near("Xu x (pixels about 1.0)", xu[0], 1.0)
near("Xu y (pixels about -0.8)", xu[1], -0.8)
near("gap", math.dist(first_a, first_x), 1.17, 2e-3)
XA, AX = mul(X0, A), mul(A, X0)
near("XA-AX 12 = 1.5 b", XA[0][1] - AX[0][1], 1.5 * 0.6)
near("XA-AX 21 = -1.5 c", XA[1][0] - AX[1][0], -1.5 * -0.5)
near("XA-AX 11 = 0", XA[0][0] - AX[0][0], 0.0)
near("1.5 = (-1/2) - (-2)", -0.5 - -2.0, 1.5)
diagonal = [[0.4, 0.0], [0.0, -0.3]]
near("meeting point x (pixels about -0.81)", act(diagonal, au)[0], -0.8)
near("meeting point y (pixels about 0.15)", act(diagonal, au)[1], 0.15)

# sheet 295: e^(pi J) = -I
J = [[0.0, -1.0], [1.0, 0.0]]
half = expm([[math.pi * x for x in row] for row in J])
near("e^(pi J) 11", half[0][0], -1.0, 1e-9)
near("e^(pi J) 21", half[1][0], 0.0, 1e-9)

# sheet 309: the complex logarithm of A
near("e^(ln 2 + i pi)", cmath.exp(math.log(2) + 1j * math.pi).real, -2.0, 1e-9)
near("e^(-ln 2 + i pi)", cmath.exp(-math.log(2) + 1j * math.pi).real, -0.5, 1e-9)
near("ln 2", math.log(2), 0.693, 2e-4)

# sheets 313-316: log ruler and dial, measured against drawn pixels
per_doubling = 61
for value, pixel in ((2, 297), (3, 333), (4, 358), (8, 419)):
    near(f"ruler tick {value}", 236 + per_doubling * math.log2(value), pixel, 1.5)
for value, degrees in ((2, 108), (3, 171), (4, 216), (2.4, 136)):
    near(f"dial angle {value}", 360 * math.log10(value), degrees, 1.5)

# sheets 324-333: laps = rate x time, against the two printed decimals
for rate, time, laps in ((0.5, 0.97, 0.48), (1.5, 0.61, 0.92), (1.5, 0.92, 1.38), (2.5, 0.17, 0.43),
                         (2.5, 0.47, 1.16), (-0.5, 0.45, -0.22)):
    near(f"laps at rate {rate} time {time}", rate * time, laps, 0.0151)

# sheet 330-335: every odd half lap about the rod lands on the same half turn
for laps in (0.5, 1.5, 2.5, -0.5, -1.5):
    angle = 2 * math.pi * laps
    near(f"cos at {laps} laps", math.cos(angle), -1.0, 1e-9)
    near(f"sin at {laps} laps", math.sin(angle), 0.0, 1e-9)

# sheet 334: half turn about the top-to-front-right edge axis moves the rod's top
n = (1 / math.sqrt(2), 0.0, 1 / math.sqrt(2))
top = (0.0, 0.0, 1.0)
dot = sum(a * b for a, b in zip(n, top))
image = tuple(2 * dot * a - b for a, b in zip(n, top))
near("top image x", image[0], 1.0, 1e-9)
near("top image z", image[2], 0.0, 1e-9)

# prediction timing from frame numbers
holds = {
    "11_3": {"cue_end": 95106, "mark": 95154, "bar_last": 95397, "next_cue": 95409},
    "12_5": {"cue_end": 112347, "mark": 112396, "bar_last": 112639, "next_cue": 112651},
}
for cue, h in holds.items():
    print(
        f"HOLD {cue}: silent {(h['next_cue'] - h['cue_end']) / FPS:.2f} s; bar seen "
        f"{(h['bar_last'] - h['mark']) / FPS:.2f} s; bar starts "
        f"{(h['mark'] - h['cue_end']) / FPS:.2f} s after the cue"
    )
print(f"HOLD 11_20: {(105085 - 103972) / FPS:.2f} s; FINAL after last word: {(116244 - 116197) / FPS:.2f} s")
print(f"ALL PASS: {checks} checks")
