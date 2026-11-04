"""A desk mirror of the pinned engine's coverage rules for a stack of horizontal strips. Not a render.

What it copies, with the file it reads from at fmn a399d626 (crates/fmn-render/src):
  - aa_coverage (stroke.rs:121): t = clamp(1/2 - excess/aa, 0, 1), then t^2 (3 - 2t); aa <= 0 becomes 1e-8.
  - excess = distance to the strip's centreline - half width (stroke.rs:17), a straight horizontal strip here.
  - source-over in linear light, with row alpha times coverage (engine.rs:3274). Rows paint in order, row 0 first.
  - the adaptive classifier (engine.rs:2463-2552, 2213-2236): a row counts one boundary sheet in a pixel when its centre
    coverage is strictly between 0 and 1, or a 2x probe (y + 0.25, y + 0.75) is, or a probe crosses from the centre's
    saturated value. Two sheets gives 2x2 samples, four gives 4x4. A saturated centre skips the probes only when the
    AA band exceeds 1/sqrt(2) px. Every strip here is wide enough to count: full width >= 2 AA bands (engine.rs:2716).
  - f32 storage of each strip's y and of its stroke width (record buffer lanes are f32).

What it leaves out: x-direction detail inside a pixel (a colour step or a plate's left and right edge), the encoder, and
anything drawn over the picture. It is good for the row structure and nothing else.
"""
import math

import numpy as np

FRAME_HEIGHT = 8.0
PHASE_LINES = 12
BAND_OFFSET = 0.5
WHEEL = (
    (0xED, 0x5B, 0x62), (0xF0, 0x8A, 0x51), (0xE7, 0xB4, 0x4E), (0xB7, 0xC7, 0x5B), (0x71, 0xC9, 0x79), (0x4D, 0xC3, 0xA5),
    (0x4A, 0xBA, 0xCD), (0x57, 0x9B, 0xE2), (0x7A, 0x83, 0xE7), (0xA3, 0x73, 0xD8), (0xC6, 0x6D, 0xB3), (0xE3, 0x65, 0x8C),
)
BACKGROUND = (0x0E / 255.0, 0x11 / 255.0, 0x16 / 255.0)
SQRT_HALF = math.sqrt(0.5)


def decode(c):
    c = np.asarray(c, dtype=float)
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def encode(c):
    c = np.clip(np.asarray(c, dtype=float), 0.0, 1.0)
    return np.where(c <= 0.0031308, c * 12.92, 1.055 * c ** (1.0 / 2.4) - 0.055)


def luma(rgb):
    return 0.2126 * rgb[..., 0] + 0.7152 * rgb[..., 1] + 0.0722 * rgb[..., 2]


def aa_coverage(excess, aa):
    band = aa if aa > 0.0 else 1e-8
    t = min(1.0, max(0.0, 0.5 - excess / band))
    return t * t * (3.0 - 2.0 * t)


class Stack:
    """Strips of a plate at one resolution. `overlap_units` widens every stroke past the cell height."""

    def __init__(self, height_px, top, height, rows, aa, overlap_units=0.0):
        self.scale = height_px / FRAME_HEIGHT
        self.origin = height_px / 2.0
        self.rows = rows
        self.aa = aa
        self.cell = height / rows
        self.top_px = self.origin - top * self.scale
        ys = np.array([top - self.cell * (i + 0.5) for i in range(rows)], dtype=np.float32).astype(float)
        self.centres = self.origin - ys * self.scale
        width_units = float(np.float32((self.cell + overlap_units) / 0.01))
        self.half = 0.5 * width_units * 0.01 * self.scale
        self.band = aa if aa > 0.0 else 1e-8
        self.reach = self.half + max(aa, 0.0)

    def near(self, y):
        nominal = (y - self.top_px) * self.rows / (self.rows * self.cell * self.scale)
        lo = max(0, int(nominal) - 3)
        hi = min(self.rows, int(nominal) + 4)
        return [i for i in range(lo, hi) if abs(y - self.centres[i]) <= self.reach + 1.0]

    def cover(self, i, y):
        return aa_coverage(abs(y - self.centres[i]) - self.half, self.aa)

    def sheets(self, py):
        """Boundary sheets in pixel row py, each row at most one (stroke_has_eligible_subcell_edge)."""
        n = 0
        for i in self.near(py + 0.5):
            c0 = self.cover(i, py + 0.5)
            if 0.0 < c0 < 1.0:
                n += 1
                continue
            if self.band > SQRT_HALF:
                continue
            for dy in (0.25, 0.75):
                c = self.cover(i, py + dy)
                if 0.0 < c < 1.0 or (c0 <= 0.0 and c > 0.0) or (c0 >= 1.0 and c < 1.0):
                    n += 1
                    break
        return n

    def sample(self, y, opacity):
        """Weights at one sample: {row: weight}, background weight. Painter order, source-over."""
        weights = {}
        ground = 1.0
        for i in self.near(y):
            a = opacity * self.cover(i, y)
            if a <= 0.0:
                continue
            for k in weights:
                weights[k] *= 1.0 - a
            ground *= 1.0 - a
            weights[i] = weights.get(i, 0.0) + a
        return weights, ground

    def pixel_row(self, py, opacity=1.0, policy="adaptive"):
        """One pixel row's weights, resolved as the engine would: {row: weight}, background weight, samples per axis."""
        n = self.sheets(py) if policy == "adaptive" else 0
        samples = 4 if n >= 4 else 2 if n >= 2 else 1
        total = {}
        ground = 0.0
        for k in range(samples):
            w, g = self.sample(py + (k + 0.5) / samples, opacity)
            ground += g / samples
            for i, x in w.items():
                total[i] = total.get(i, 0.0) + x / samples
        return total, ground, samples


def wheel(angle):
    t = np.mod(angle, math.tau) / math.tau * 12.0
    k = np.minimum(np.floor(t).astype(int), 11)
    f = (t - k)[..., None]
    table = np.array(WHEEL, dtype=float)
    return (table[k] + (table[(k + 1) % 12] - table[k]) * f) / 255.0


def shade(w):
    step = math.tau / PHASE_LINES
    size = np.abs(w)
    size_band = np.mod(np.log(size) / step + BAND_OFFSET, 1.0)
    phase_band = np.mod(np.mod(np.angle(w), math.tau) / step + BAND_OFFSET, 1.0)
    return (0.58 + 0.42 * size_band) * (0.80 + 0.20 * phase_band)


def portrait(f, z, enhance):
    """Encoded colours for inputs z. Zero, pole and non-finite cells stay the background, as draw.rs paints them."""
    with np.errstate(all="ignore"):
        w = f(z)
        good = np.isfinite(w) & (np.abs(w) > 0.0)
        safe = np.where(good, w, 1.0 + 0j)
        out = wheel(np.angle(safe)) * (1.0 + (shade(safe) - 1.0) * enhance)[..., None]
    return np.where(good[..., None], out, np.array(BACKGROUND))


FUNCS = {
    "z": lambda z: z,
    "z^2": lambda z: z * z,
    "1/z": lambda z: 1.0 / z,
    "bump": lambda z: 1.0 / (1.0 + z * z),
    "tan": lambda z: np.tan(z),
    "z^3-1": lambda z: z * z * z - 1.0,
    "sin": lambda z: np.sin(z),
    "sin(1/z)": lambda z: np.sin(1.0 / z),
}


class Plate:
    """A plate's cell grid as the film builds it: picture_cells, then cell_input at cell centres."""

    def __init__(self, center, width, x, y, cell=0.035):
        self.center, self.width, self.x, self.y = center, width, x, y
        self.height = width * (y[1] - y[0]) / (x[1] - x[0])
        self.left = center[0] - width / 2.0
        self.top = center[1] + self.height / 2.0
        self.cols = max(int(round(width / cell)), 8)
        self.rows = max(int(round(self.height / cell)), 8)

    def inputs(self):
        u = (np.arange(self.cols) + 0.5) / self.cols
        v = (np.arange(self.rows) + 0.5) / self.rows
        re = self.x[0] + (self.x[1] - self.x[0]) * u
        im = self.y[1] - (self.y[1] - self.y[0]) * v
        return re[None, :] + 1j * im[:, None]

    def colours(self, f, enhance):
        return portrait(f, self.inputs(), enhance)

    def input_at(self, xs_px, ys_px, height_px):
        scale = height_px / FRAME_HEIGHT
        origin_x = height_px * 16.0 / 9.0 / 2.0
        origin_y = height_px / 2.0
        sx = (xs_px - origin_x) / scale
        sy = (origin_y - ys_px) / scale
        re = self.x[0] + (sx - self.left) / self.width * (self.x[1] - self.x[0])
        im = self.y[1] - (self.top - sy) / self.height * (self.y[1] - self.y[0])
        return re[None, :] + 1j * im[:, None]

    def stack(self, height_px, aa, overlap_units=0.0):
        return Stack(height_px, self.top, self.height, self.rows, aa, overlap_units)

    def col_of(self, xs_px, height_px):
        scale = height_px / FRAME_HEIGHT
        origin_x = height_px * 16.0 / 9.0 / 2.0
        sx = (xs_px + 0.5 - origin_x) / scale
        return np.clip(((sx - self.left) / self.width * self.cols).astype(int), 0, self.cols - 1)


def composite(plate, stack, colours, x0, x1, y0, y1, height_px, opacity=1.0, policy="adaptive"):
    """Encoded RGB for a pixel window, plus the sample count per pixel row."""
    xs = np.arange(x0, x1, dtype=float)
    cols = plate.col_of(xs, height_px)
    linear = decode(colours)
    ground = decode(np.array(BACKGROUND))
    out = np.zeros((y1 - y0, x1 - x0, 3))
    samples = []
    for row, py in enumerate(range(y0, y1)):
        weights, g, s = stack.pixel_row(py, opacity, policy)
        samples.append(s)
        acc = np.tile(ground * g, (x1 - x0, 1))
        for i, w in weights.items():
            acc = acc + w * linear[i, cols]
        out[row] = acc
    return encode(out), samples


def residual(series):
    series = np.asarray(series, dtype=float)
    kernel = np.ones(9) / 9.0
    return series[4:-4] - np.convolve(series, kernel, mode="valid")


def correlation(a, b):
    a, b = np.asarray(a) - np.mean(a), np.asarray(b) - np.mean(b)
    return float(np.sum(a * b) / math.sqrt(float(np.sum(a * a) * np.sum(b * b))))


def stats(series):
    """Mean, rms, rms %, strongest period, as stripes02.py computes them."""
    r = residual(series)
    mean = float(np.mean(series))
    rms = float(np.sqrt(np.mean(r * r)))
    n = len(r)
    spectrum = np.abs(np.fft.fft(r))
    k = int(np.argmax(spectrum[2 : n // 2])) + 2
    return mean, rms, 100.0 * rms / mean, n / k
