"""Do the rendered colour pictures show the functions the voice names? Frame pixels against an independent evaluation.

    python3 -I -B colour_vs_function.py      (from the repo root)

Functions are written here again with cmath, plus my own Lanczos gamma and Euler-Maclaurin zeta. They are not the
film's code. WHEEL (model.rs:489) is copied, because the wheel is what is being read. Pixels are divided by their
largest channel, which removes the brightness bands, so this tests hue only. Grid of 61 x 61 points inside each
picture; points on marks, cards and dark ground are skipped. The control evaluates the conjugate output, so the
check can fail. Reads native 720p PNGs; writes nothing.
"""
import cmath
import math
import pathlib
import statistics

from PIL import Image

ROOT = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-frames01")
WHEEL = [(0xED, 0x5B, 0x62), (0xF0, 0x8A, 0x51), (0xE7, 0xB4, 0x4E), (0xB7, 0xC7, 0x5B), (0x71, 0xC9, 0x79), (0x4D, 0xC3, 0xA5),
         (0x4A, 0xBA, 0xCD), (0x57, 0x9B, 0xE2), (0x7A, 0x83, 0xE7), (0xA3, 0x73, 0xD8), (0xC6, 0x6D, 0xB3), (0xE3, 0x65, 0x8C)]
LANCZOS = [0.99999999999980993, 676.5203681218851, -1259.1392167224028, 771.32342877765313, -176.61502916214059,
           12.507343278686905, -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7]
BERNOULLI = [1 / 6, -1 / 30, 1 / 42, -1 / 30, 5 / 66, -691 / 2730, 7 / 6, -3617 / 510]


def gamma(z):
    if z.real < 0.5:
        return math.pi / (cmath.sin(math.pi * z) * gamma(1 - z))
    z -= 1
    x = LANCZOS[0] + sum(c / (z + i + 1) for i, c in enumerate(LANCZOS[1:]))
    t = z + 7.5
    return math.sqrt(2 * math.pi) * t ** (z + 0.5) * cmath.exp(-t) * x


def zeta(s, n=40):
    total = sum(k ** -s for k in range(1, n))
    total += n ** (1 - s) / (s - 1) + 0.5 * n ** -s
    rising, fact = s, 2.0
    for j, b in enumerate(BERNOULLI):
        total += b / fact * rising * n ** (-s - 2 * j - 1)
        rising *= (s + 2 * j + 1) * (s + 2 * j + 2)
        fact *= (2 * j + 3) * (2 * j + 4)
    return total


def hue(angle):
    t = (angle % math.tau) / math.tau * 12
    k = min(int(t), 11)
    f = t - k
    a, b = WHEEL[k], WHEEL[(k + 1) % 12]
    c = [a[i] + (b[i] - a[i]) * f for i in range(3)]
    m = max(c)
    return [x / m for x in c]


def plane(cx, cy, w, x, y):
    return (cx, cy, w, w * (y[1] - y[0]) / (x[1] - x[0]), x, y)


Z04 = plane(-2.6, -0.25, 6.4, (-2, 2), (-2, 2))
SQ = plane(-3.4, -0.2, 5.6, (-2, 2), (-2, 2))
EX = plane(-2.6, -0.25, 3.2, (-3, 3), (-4, 8))
TR = plane(-2.4, -0.1, 8.4, (-math.tau, math.tau), (-3.3, 3.3))
GA = plane(-2.4, -0.3, 8.4, (-4.5, 4.5), (-2.5, 2.5))
ZE = plane(-3.4, 0.2, 6.4, (-7, 3), (-3, 3))
ST = (1.6, 0.35, 2.0, 5.9, (-1, 2), (0, 32))
CASES = [
    ("04_reading_color/frame-0020.png", "z", Z04, lambda z: z),
    ("04_reading_color/frame-0140.png", "z^2", Z04, lambda z: z * z),
    ("04_reading_color/frame-0200.png", "1/z", Z04, lambda z: 1 / z),
    ("04_reading_color/frame-0300.png", "1/(1+z^2)", Z04, lambda z: 1 / (1 + z * z)),
    ("05_gallery/frame-0030.png", "z^3-1", SQ, lambda z: z ** 3 - 1),
    ("05_gallery/frame-0082.png", "(z-1)/(z+1)^2", SQ, lambda z: (z - 1) / (z + 1) ** 2),
    ("05_gallery/frame-0092.png", "e^z", EX, cmath.exp),
    ("05_gallery/frame-0166.png", "sin z", TR, cmath.sin),
    ("05_gallery/frame-0200.png", "tan z", TR, cmath.tan),
    ("05_gallery/frame-0280.png", "gamma", GA, gamma),
    ("05_gallery/frame-0436.png", "zeta", ZE, zeta),
    ("05_gallery/frame-0436.png", "zeta strip", ST, zeta),
]


def read(path, box, f, conj=False):
    cx, cy, w, h, xr, yr = box
    im = Image.open(ROOT / path).convert("RGB")
    left, top = 640 + 90 * (cx - w / 2), 360 - 90 * (cy + h / 2)
    errs = []
    for i in range(61):
        for j in range(61):
            u, t = (i + 0.5) / 61, (j + 0.5) / 61
            z = complex(xr[0] + (xr[1] - xr[0]) * u, yr[1] - (yr[1] - yr[0]) * t)
            px = im.getpixel((int(left + u * w * 90), int(top + t * h * 90)))
            m = max(px)
            if m < 70 or min(px) > 200:
                continue
            try:
                out = f(z)
                if conj:
                    out = out.conjugate()
                want = hue(cmath.phase(out))
            except (ZeroDivisionError, OverflowError, ValueError):
                continue
            got = [c / m for c in px]
            errs.append(max(abs(a - b) for a, b in zip(want, got)))
    return errs


print("function, points read, median and 90th percentile hue error (0 to 1), control's median")
for path, name, box, f in CASES:
    e = read(path, box, f)
    c = read(path, box, f, conj=True)
    e.sort()
    print(f"{name:16s} {path:34s} n={len(e):4d} median {statistics.median(e):.3f}  p90 {e[int(0.9 * len(e))]:.3f}   conjugate control {statistics.median(c):.3f}")

print()
print("Numbers on screen and in the voice, recomputed")
step = math.tau / 12
print(f"band ratio e^(tau/12) = {math.exp(step):.4f}   card 1.69, voice about 1.7")
print(f"sin(i) = {cmath.sin(1j)}   card 1.18 i")
print(f"sin(3i) = {cmath.sin(3j)}   card 10.0 i")
print(f"max |sin x| over the plane's real line = {max(abs(math.sin(-math.tau + k * 2 * math.tau / 4000)) for k in range(4001)):.6f}")
print(f"tan(4i), tan(-4i) = {cmath.tan(4j):.6f}, {cmath.tan(-4j):.6f}")
print(f"gamma(4), gamma(5) = {gamma(4 + 0j).real:.6f}, {gamma(5 + 0j).real:.6f}")
print(f"z^3 - 1 at its three zeros: {[round(abs(cmath.exp(2j * math.pi * k / 3) ** 3 - 1), 12) for k in range(3)]}")
print(f"1/(1+z^2) denominators at i and -i: {abs(1 + 1j ** 2):.1f}, {abs(1 + (-1j) ** 2):.1f}; |i| = 1")
for t in (14.1347251, 21.0220396, 25.0108576, 30.4248761):
    print(f"|zeta(1/2 + {t}i)| = {abs(zeta(complex(0.5, t))):.2e}   card {t:.2f}i")
for s in (-2, -4, -6):
    print(f"|zeta({s})| = {abs(zeta(complex(s, 0))):.2e}")
print("exp tile: both sides tau/12 =", round(step, 6), "in the plane's own units, so exact squares")


def turns(f, c, r, n=4000):
    tot, prev = 0.0, cmath.phase(f(c + r))
    for k in range(1, n + 1):
        cur = cmath.phase(f(c + r * cmath.exp(1j * math.tau * k / n)))
        d = (cur - prev + math.pi) % math.tau - math.pi
        tot, prev = tot + d, cur
    return round(tot / math.tau, 6)


print("turns by winding, independent:",
      "z^2 at 0:", turns(lambda z: z * z, 0, 0.9), " 1/z at 0:", turns(lambda z: 1 / z, 0, 0.9),
      " bump at i:", turns(lambda z: 1 / (1 + z * z), 1j, 0.5), " bump at -i:", turns(lambda z: 1 / (1 + z * z), -1j, 0.5),
      " z^3-1 big loop:", turns(lambda z: z ** 3 - 1, 0, 1.8), " (z-1)/(z+1)^2 at -1:", turns(lambda z: (z - 1) / (z + 1) ** 2, -1, 0.3))
