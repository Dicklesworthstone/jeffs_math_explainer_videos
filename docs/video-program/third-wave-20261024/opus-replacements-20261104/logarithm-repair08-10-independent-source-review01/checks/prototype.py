"""Scratch prototype of the review's proposed real-flow border test, so its tolerance is
known before it is proposed. A Python port of model::mexp and draw::inside_clip; not the crate."""
import math


def mmul(a, b):
    return [[a[i][0] * b[0][j] + a[i][1] * b[1][j] for j in range(2)] for i in range(2)]


def madd(a, b):
    return [[a[i][j] + b[i][j] for j in range(2)] for i in range(2)]


def mscale(a, k):
    return [[a[i][j] * k for j in range(2)] for i in range(2)]


I2 = [[1.0, 0.0], [0.0, 1.0]]
J = [[0.0, -1.0], [1.0, 0.0]]


def mexp(x):
    norm = max(abs(v) for r in x for v in r)
    s = 0
    while norm / 2.0 ** s > 0.25:
        s += 1
    small = mscale(x, 1.0 / 2.0 ** s)
    total, term = I2, I2
    for k in range(1, 31):
        term = mscale(mmul(term, small), 1.0 / k)
        total = madd(total, term)
    for _ in range(s):
        total = mmul(total, total)
    return total


def apply(a, p):
    return [a[0][0] * p[0] + a[0][1] * p[1], a[1][0] * p[0] + a[1][1] * p[1]]


def rotation(t):
    return [[math.cos(t), -math.sin(t)], [math.sin(t), math.cos(t)]]


def inverse(a):
    d = a[0][0] * a[1][1] - a[0][1] * a[1][0]
    return [[a[1][1] / d, -a[0][1] / d], [-a[1][0] / d, a[0][0] / d]]


def ellipse(a, b, tilt):
    p = mmul(rotation(tilt), [[a, 0.0], [0.0, b]])
    return mscale(mmul(mmul(p, J), inverse(p)), 2 * math.pi)


def inside(q, clip, slack):
    return abs(q[0]) <= clip + slack and abs(q[1]) <= clip + slack


flows = {"2piJ": mscale(J, 2 * math.pi), "ellipse 2,1,0": ellipse(2, 1, 0), "ellipse 1.6,.7,30": ellipse(1.6, 0.7, math.radians(30))}
for name, x in flows.items():
    home = mexp(x)
    worst = max(abs(home[i][j] - I2[i][j]) for i in range(2) for j in range(2))
    border = [[s * 3.0, -3.0 + j / 12.0] for s in (1, -1) for j in range(73)] + [[-3.0 + j / 12.0, s * 3.0] for s in (1, -1) for j in range(73)]
    print(f"{name:20s} error {worst:.2e}  kept with slack {sum(inside(apply(home, p), 3.0, 1e-9) for p in border)}/{len(border)}"
          f"  kept without {sum(inside(apply(home, p), 3.0, 0.0) for p in border)}/{len(border)}")
stretch = [[math.log(1.5), 0.0], [0.0, math.log(0.6)]]
home = mexp(stretch)
line = [[s * 2.0, -4.0 + j / 12.0] for s in (1, -1) for j in range(97)]
print(f"stretch x=+-2: lands at {apply(home, [2.0, 0.0])[0]!r}; kept with slack {sum(inside(apply(home, p), 3.0, 1e-9) for p in line)}/{len(line)}"
      f"; without {sum(inside(apply(home, p), 3.0, 0.0) for p in line)}/{len(line)}")
