"""Stair-step candidates (04-A2, S2) on real film pictures: a desk model of a 4K crop against a supersampled truth.

    python3 -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-color-repair03/checks/stair_model.py [OUT_DIR]

Not a render. It uses engine_mirror's row weights (aa 0, guard 0.0001) and models a row's colour along x two ways:
  steps  the kit's RowStrips: a colour jump at each cell edge, point-sampled at the pixel centre.
  blend  the blended strip this bundle adds: linear in linear light between cell centres, flat for the half cell at each
         end. Mid-cell records hold the mean of the two encoded colours; that is modelled too.
Truth: the portrait at 16 samples a pixel, box-filtered in linear light. Zero and pole cells stay the background.

Numbers: rms is encoded 0..255 against truth, over the crop, at 4K and after a 2x2 box to 1080p. riser is the largest
jump between neighbouring pixels along the worst row or column, as a share of truth's own largest jump.
These rank candidates. They do not say what a 4K frame looks like: that is for root's harness run and a person's eye.
With OUT_DIR, writes each candidate's crop as a PNG: a simulation, never to be read as a render.
"""
import pathlib
import sys

import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import engine_mirror as em  # noqa: E402

H = 2160
SCALE = H / 8.0
GUARD = 1e-4


TILE_SEED = complex(0.55, 0.95)
TILE_STEPS = 16


def tile_centre():
    """gallery.rs tile() and tile_centre(), with its seed and step count: the zoom plate's real centre."""
    step = 2.0 * np.pi / em.PHASE_LINES
    f0 = TILE_SEED**3 - 1.0
    g = 3.0 * TILE_SEED**2 / f0
    w0 = complex(np.log(abs(f0)), np.angle(f0))

    def below(x):
        return step * (np.floor(x / step + em.BAND_OFFSET) - em.BAND_OFFSET)

    corner = complex(below(w0.real), below(w0.imag))
    points = []
    for a, b in ((0j, 1 + 0j), (1 + 0j, 1 + 1j), (1 + 1j, 1j), (1j, 0j)):
        for k in range(TILE_STEPS):
            w = corner + (a + (b - a) * (k / TILE_STEPS)) * step
            z = TILE_SEED + (w - w0) / g
            target = np.exp(w)
            for _ in range(50):
                move = (z**3 - 1.0 - target) / (3.0 * z**2)
                z = z - move
                if abs(move) < 1e-15:
                    break
            points.append(z)
    return sum(points) / len(points)


def tile_zoom():
    c = tile_centre()
    return (2.6, -0.3), 5.0, (c.real - 0.65, c.real + 0.65), (c.imag - 0.65, c.imag + 0.65)


SUBJECTS = [
    ("z^2, reading plate", ((-2.6, -0.25), 6.4, (-2.0, 2.0), (-2.0, 2.0)), "z^2", 1.0, (500, 1100, 300, 900)),
    ("z^3-1, zoom plate", tile_zoom(), "z^3-1", 1.0, (2120, 2720, 900, 1500)),
    ("tan, trig plate", ((-2.4, -0.1), 8.4, (-6.6, 6.6), (-3.3, 3.3)), "tan", 1.0, (900, 1500, 900, 1400)),
    # Mail 2542: chapter 10's plates. Windows sit on each plate's busiest region.
    ("sin(1/z), rigidity plate", ((-2.8, 0.0), 5.6, (-0.42, 0.42), (-0.42, 0.42)), "sin(1/z)", 1.0, (860, 1460, 480, 1040)),
    ("sin z, rigidity plate", ((-2.8, -0.3), 6.0, (-4.0, 4.0), (-3.5, 3.5)), "sin", 1.0, (860, 1460, 860, 1460)),
    # Mail 2591 and 2614: chapter 12's smallest plate, and chapter 14's opening plate.
    ("1/z, residues small plate", ((1.5, -1.0), 2.8, (-1.2, 1.2), (-1.2, 1.2)), "1/z", 1.0, (2100, 2550, 1130, 1580)),
    ("1/(1+z^2), continuation plate", ((-3.3, -0.35), 5.6, (-2.5, 2.5), (-2.5, 2.5)), "bump", 1.0, (700, 1300, 550, 1150)),
]

# name, row height, column width, samples per axis in a cell, blend along x
CANDIDATES = [
    ("S0 cells 0.035, steps (film now)", 0.035, 0.035, 1, False),
    ("S1 P3: four samples, steps", 0.035, 0.035, 2, False),
    ("S2 cells 0.0175, steps", 0.0175, 0.0175, 1, False),
    ("S3 cells 0.035, blend", 0.035, 0.035, 1, True),
    ("S4 rows 0.0175, cols 0.035, blend", 0.0175, 0.035, 1, True),
    ("S5 rows 0.0117, cols 0.035, blend", 0.035 / 3.0, 0.035, 1, True),
    ("S6 cells 0.0175, blend", 0.0175, 0.0175, 1, True),
    ("S7 rows 0.0175, cols 0.035, four samples, blend", 0.0175, 0.035, 2, True),
    ("S8 cells 0.0117, blend", 0.035 / 3.0, 0.035 / 3.0, 1, True),
    ("S9 cells 0.00875, blend", 0.035 / 4.0, 0.035 / 4.0, 1, True),
]


class Grid:
    def __init__(self, args, row, col):
        self.center, self.width, self.x, self.y = args
        self.height = self.width * (self.y[1] - self.y[0]) / (self.x[1] - self.x[0])
        self.left = self.center[0] - self.width / 2.0
        self.top = self.center[1] + self.height / 2.0
        self.cols = max(int(round(self.width / col)), 8)
        self.rows = max(int(round(self.height / row)), 8)


def cell_colours(grid, f, enhance, samples):
    """Encoded colours per cell. samples > 1 is P3's mean, over offsets at a quarter of a cell for samples = 2."""
    u = (np.arange(grid.cols) + 0.5) / grid.cols
    v = (np.arange(grid.rows) + 0.5) / grid.rows
    re = grid.x[0] + (grid.x[1] - grid.x[0]) * u
    im = grid.y[1] - (grid.y[1] - grid.y[0]) * v
    z = re[None, :] + 1j * im[:, None]
    if samples == 1:
        return em.portrait(f, z, enhance)
    dx = (grid.x[1] - grid.x[0]) / grid.cols / 4.0
    dy = (grid.y[1] - grid.y[0]) / grid.rows / 4.0
    out = 0.0
    for sx in (-1.0, 1.0):
        for sy in (-1.0, 1.0):
            out = out + em.portrait(f, z + sx * dx + 1j * sy * dy, enhance)
    return out / 4.0


def along_x(grid, colours_linear, encoded, xs_px, blend):
    """Linear colour at each pixel centre for every row: rows x pixels x 3."""
    origin_x = H * 16.0 / 9.0 / 2.0
    sx = (xs_px + 0.5 - origin_x) / SCALE
    t = (sx - grid.left) / grid.width * grid.cols
    if not blend:
        k = np.clip(t.astype(int), 0, grid.cols - 1)
        return colours_linear[:, k]
    # stations at cell centres; a mid-cell record holds the mean of two encoded colours, decoded.
    c = t - 0.5
    k = np.clip(np.floor(c).astype(int), 0, grid.cols - 2)
    f = np.clip(c - k, 0.0, 1.0)
    a = colours_linear[:, k]
    b = colours_linear[:, k + 1]
    mid = em.decode((encoded[:, k] + encoded[:, k + 1]) / 2.0)
    g = f[None, :, None]
    first = a + (mid - a) * (g * 2.0)
    second = mid + (b - mid) * (g * 2.0 - 1.0)
    return np.where(g < 0.5, first, second)


def model(args, fname, enhance, window, row, col, samples, blend):
    x0, x1, y0, y1 = window
    grid = Grid(args, row, col)
    encoded = cell_colours(grid, em.FUNCS[fname], enhance, samples)
    linear = em.decode(encoded)
    stack = em.Stack(H, grid.top, grid.height, grid.rows, 0.0, GUARD)
    xs = np.arange(x0, x1, dtype=float)
    per_row = along_x(grid, linear, encoded, xs, blend)
    ground = em.decode(np.array(em.BACKGROUND))
    out = np.zeros((y1 - y0, x1 - x0, 3))
    for i, py in enumerate(range(y0, y1)):
        weights, g, _ = stack.pixel_row(py)
        acc = ground * g * np.ones((x1 - x0, 3))
        for r, w in weights.items():
            acc = acc + w * per_row[r]
        out[i] = acc
    return em.encode(out)


def truth(args, fname, enhance, window, n=4):
    x0, x1, y0, y1 = window
    grid = Grid(args, 0.035, 0.035)
    origin_x = H * 16.0 / 9.0 / 2.0
    origin_y = H / 2.0
    offs = (np.arange(n) + 0.5) / n
    sx = (np.repeat(np.arange(x0, x1), n) + np.tile(offs, x1 - x0) - origin_x) / SCALE
    sy = (origin_y - (np.repeat(np.arange(y0, y1), n) + np.tile(offs, y1 - y0))) / SCALE
    re = grid.x[0] + (sx - grid.left) / grid.width * (grid.x[1] - grid.x[0])
    im = grid.y[1] - (grid.top - sy) / grid.height * (grid.y[1] - grid.y[0])
    z = re[None, :] + 1j * im[:, None]
    c = em.decode(em.portrait(em.FUNCS[fname], z, enhance))
    c = c.reshape(y1 - y0, n, x1 - x0, n, 3).mean(axis=(1, 3))
    return em.encode(c)


def down2(a):
    h, w = a.shape[0] // 2 * 2, a.shape[1] // 2 * 2
    lin = em.decode(a[:h, :w])
    return em.encode((lin[::2, ::2] + lin[1::2, ::2] + lin[::2, 1::2] + lin[1::2, 1::2]) / 4.0)


def rms(a, b):
    return float(np.sqrt(np.mean(((a - b) * 255.0) ** 2)))


def riser(a):
    y = em.luma(a * 255.0)
    return float(max(np.abs(np.diff(y, axis=0)).max(), np.abs(np.diff(y, axis=1)).max()))


def lattice_power(a, t, period):
    """Row-mean energy of (candidate - truth) at the lattice's own frequency, in luma: how much of a repeating step shows."""
    d = em.luma((a - t) * 255.0).mean(axis=1)
    d = d - d.mean()
    n = len(d)
    k = np.arange(n)
    return float(2.0 * abs(np.sum(d * np.exp(-2j * np.pi * k / period))) / n)


PLATES = {
    # plate: how many times the film builds it, from the Painter::build sites; used for cost shares only
    "6.4 x 6.4": (6.4, 6.4),
    "8.4 x 4.2": (8.4, 4.2),
    "5.6 x 5.6": (5.6, 5.6),
    "1.5 x 1.5": (1.5, 1.5),
}


def cost(row, col, samples, blend):
    w, h = PLATES["6.4 x 6.4"]
    rows, cols = max(int(round(h / row)), 8), max(int(round(w / col)), 8)
    drawn = cols + 2 if not blend else cols + 3
    records = (3 + 4 * cols + 4) if not blend else (2 * cols + 11)
    return rows, cols, rows * cols * samples * samples, drawn, records, rows * records


out_dir = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else None
try:
    from PIL import Image
except ImportError:
    Image = None

print(f"zoom plate centre, from gallery.rs tile(): {tile_centre():.6f}")
base = cost(0.035, 0.035, 1, False)
print("Cost on the 6.4 x 6.4 plate, from the layouts, as a multiple of the film now. Not timed.")
print(f"{'candidate':50s} {'rows':>5s} {'cols':>5s} {'evals':>7s} {'x now':>6s} {'curves/row':>10s} {'x now':>6s} {'records':>8s} {'x now':>6s}")
for name, row, col, samples, blend in CANDIDATES:
    c = cost(row, col, samples, blend)
    print(f"{name:50s} {c[0]:5d} {c[1]:5d} {c[2]:7d} {c[2] / base[2]:6.1f} {c[3]:10d} {c[3] / base[3]:6.2f} {c[5]:8d} {c[5] / base[5]:6.2f}")
print("evals: model evaluations per repaint (twice during a crossfade). curves/row: what each pixel's nearest-segment loop")
print("walks, so raster work per pixel scales with it. records: colour records rewritten per frame of a fade or repaint.")

for title, args, fname, enhance, window in SUBJECTS:
    t = truth(args, fname, enhance, window)
    t2 = down2(t)
    print(f"\n{title}, window {window}, truth's largest neighbour jump {riser(t):.0f} luma")
    print(f"{'candidate':50s} {'rms 4K':>7s} {'rms 1080':>9s} {'row period px':>14s} {'lattice luma':>13s}")
    for name, row, col, samples, blend in CANDIDATES:
        a = model(args, fname, enhance, window, row, col, samples, blend)
        grid = Grid(args, row, col)
        period = grid.height / grid.rows * SCALE
        print(f"{name:50s} {rms(a, t):7.2f} {rms(down2(a), t2):9.2f} {period:14.2f} {lattice_power(a, t, period):13.2f}")
        if out_dir and Image:
            out_dir.mkdir(parents=True, exist_ok=True)
            tag = name.split()[0]
            label = title.split(",")[0].replace("^", "").replace("/", "")
            Image.fromarray((a * 255.0 + 0.5).astype(np.uint8)).save(out_dir / f"{label}-{tag}.png")
    if out_dir and Image:
        Image.fromarray((t * 255.0 + 0.5).astype(np.uint8)).save(out_dir / f"{title.split(',')[0].replace('^', '').replace('/', '')}-truth.png")
