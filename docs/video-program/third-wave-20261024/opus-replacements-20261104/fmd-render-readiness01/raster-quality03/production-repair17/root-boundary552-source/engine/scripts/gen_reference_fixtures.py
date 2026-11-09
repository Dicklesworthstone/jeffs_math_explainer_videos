#!/usr/bin/env python3
"""Generate fmn-core parity fixtures from the pinned Reference checkout.

Reference: 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13.
Reproduces the Reference's own arithmetic (constants.py over
default_config.yml; utils/rate_functions.py with its Bernstein bezier();
utils/color.py's sqrt-lerp gradient formulas) in pure Python doubles, and
writes fixtures the fmn-core parity tests compare against.

Outputs (committed under crates/fmn-core/fixtures/):
  constants.txt       NAME<TAB>VALUE lines (floats via repr, colors as hex)
  rate_functions.txt  manifest: name<TAB>sample_count per function, in order
  rate_functions.bin  f64 LE samples, 10001 per function (t = i/10000)
  color_ops.txt       interpolate/average/gradient samples as rgb repr triples
"""

import math
import os
import struct

import yaml

REF = os.path.join(os.path.dirname(os.path.abspath(__file__)), "manim_ref")
OUT = "/data/projects/franken_manim/crates/fmn-core/fixtures"
N = 10_000  # samples are t = i/N for i in 0..=N (10001 values)

with open(os.path.join(REF, "manimlib", "default_config.yml")) as f:
    CFG = yaml.safe_load(f)


def r(x: float) -> str:
    return repr(float(x))


# ---------------------------------------------------------------- constants
def gen_constants() -> None:
    lines: list[tuple[str, str]] = []

    def put(name, value):
        lines.append((name, value))

    def putf(name, value):
        put(name, r(value))

    def putv(name, v):
        put(name, "\t".join(r(c) for c in v))

    res = CFG["camera"]["resolution"]
    pw, ph = (int(x) for x in res.strip("()").split(","))
    putf("DEFAULT_PIXEL_WIDTH", pw)
    putf("DEFAULT_PIXEL_HEIGHT", ph)
    aspect = pw / ph
    putf("ASPECT_RATIO", aspect)
    frame_height = float(CFG["sizes"]["frame_height"])
    frame_width = frame_height * aspect
    putf("FRAME_HEIGHT", frame_height)
    putf("FRAME_WIDTH", frame_width)
    putf("FRAME_Y_RADIUS", frame_height / 2)
    putf("FRAME_X_RADIUS", frame_width / 2)

    s = CFG["sizes"]
    putf("SMALL_BUFF", s["small_buff"])
    putf("MED_SMALL_BUFF", s["med_small_buff"])
    putf("MED_LARGE_BUFF", s["med_large_buff"])
    putf("LARGE_BUFF", s["large_buff"])
    putf("DEFAULT_MOBJECT_TO_EDGE_BUFF", s["default_mobject_to_edge_buff"])
    putf("DEFAULT_MOBJECT_TO_MOBJECT_BUFF", s["default_mobject_to_mobject_buff"])

    dirs = {
        "ORIGIN": (0.0, 0.0, 0.0),
        "UP": (0.0, 1.0, 0.0),
        "DOWN": (0.0, -1.0, 0.0),
        "RIGHT": (1.0, 0.0, 0.0),
        "LEFT": (-1.0, 0.0, 0.0),
        "IN": (0.0, 0.0, -1.0),
        "OUT": (0.0, 0.0, 1.0),
        "X_AXIS": (1.0, 0.0, 0.0),
        "Y_AXIS": (0.0, 1.0, 0.0),
        "Z_AXIS": (0.0, 0.0, 1.0),
        "UL": (-1.0, 1.0, 0.0),
        "UR": (1.0, 1.0, 0.0),
        "DL": (-1.0, -1.0, 0.0),
        "DR": (1.0, -1.0, 0.0),
    }
    for name, v in dirs.items():
        putv(name, v)
    fy, fx = frame_height / 2, frame_width / 2
    putv("TOP", (0.0 * fy, 1.0 * fy, 0.0 * fy))
    putv("BOTTOM", (0.0 * fy, -1.0 * fy, 0.0 * fy))
    putv("LEFT_SIDE", (-1.0 * fx, 0.0 * fx, 0.0 * fx))
    putv("RIGHT_SIDE", (1.0 * fx, 0.0 * fx, 0.0 * fx))

    putf("PI", math.pi)
    putf("TAU", 2 * math.pi)
    putf("DEG", 2 * math.pi / 360)
    putf("RADIANS", 1.0)

    putf("DEFAULT_STROKE_WIDTH", CFG["vmobject"]["default_stroke_width"])
    putf("STROKE_WIDTH_CONVERSION", 0.01)  # shaders/quadratic_bezier/stroke/vert.glsl
    putf("DEFAULT_FPS", CFG["camera"]["fps"])
    put("DEFAULT_BACKGROUND_COLOR", CFG["camera"]["background_color"])

    for key, name in [("low", "RESOLUTION_LOW"), ("med", "RESOLUTION_MED"),
                      ("high", "RESOLUTION_HIGH"), ("4k", "RESOLUTION_4K")]:
        w, h = (int(x) for x in CFG["resolution_options"][key].strip("()").split(","))
        put(name, f"{w}\t{h}")

    for key, hexval in CFG["colors"].items():
        put(key.upper(), hexval)
    # Median-color aliases and semantic defaults, as constants.py binds them
    for alias, target in [("BLUE", "blue_c"), ("TEAL", "teal_c"), ("GREEN", "green_c"),
                          ("YELLOW", "yellow_c"), ("GOLD", "gold_c"), ("RED", "red_c"),
                          ("MAROON", "maroon_c"), ("PURPLE", "purple_c"), ("GREY", "grey_c")]:
        put(alias, CFG["colors"][target])
    put("DEFAULT_MOBJECT_COLOR", CFG["mobject"]["default_mobject_color"])
    put("DEFAULT_LIGHT_COLOR", CFG["mobject"]["default_light_color"])
    put("DEFAULT_VMOBJECT_STROKE_COLOR", CFG["vmobject"]["default_stroke_color"])
    put("DEFAULT_VMOBJECT_FILL_COLOR", CFG["vmobject"]["default_fill_color"])

    with open(os.path.join(OUT, "constants.txt"), "w") as f:
        f.write("# fmn-core constants parity fixture\n")
        f.write("# generated from 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13\n")
        f.write("# by scripts/gen_reference_fixtures.py — regenerate, never hand-edit\n")
        for name, value in lines:
            f.write(f"{name}\t{value}\n")
    print(f"constants.txt: {len(lines)} entries")


# ------------------------------------------------------------ rate functions
def choose(n: int, k: int) -> int:
    return math.comb(n, k)


def bezier(points):
    # Mirrors manimlib.utils.bezier.bezier: Bernstein sum in index order.
    n = len(points) - 1

    def result(t):
        return sum(((1 - t) ** (n - k)) * (t ** k) * choose(n, k) * p
                   for k, p in enumerate(points))
    return result


def smooth(t):
    s = 1 - t
    return (t ** 3) * (10 * s * s + 5 * s * t + t * t)


def rush_into(t):
    return 2 * smooth(0.5 * t)


def rush_from(t):
    return 2 * smooth(0.5 * (t + 1)) - 1


def slow_into(t):
    return math.sqrt(1 - (1 - t) * (1 - t))


def double_smooth(t):
    if t < 0.5:
        return 0.5 * smooth(2 * t)
    return 0.5 * (1 + smooth(2 * t - 1))


def there_and_back(t):
    new_t = 2 * t if t < 0.5 else 2 * (1 - t)
    return smooth(new_t)


def there_and_back_with_pause(t, pause_ratio=1.0 / 3):
    a = 2.0 / (1.0 - pause_ratio)
    if t < 0.5 - pause_ratio / 2:
        return smooth(a * t)
    if t < 0.5 + pause_ratio / 2:
        return 1
    return smooth(a - a * t)


def running_start(t, pull_factor=-0.5):
    return bezier([0, 0, pull_factor, pull_factor, 1, 1, 1])(t)


def overshoot(t, pull_factor=1.5):
    return bezier([0, 0, pull_factor, pull_factor, 1, 1])(t)


def wiggle(t, wiggles=2):
    return there_and_back(t) * math.sin(wiggles * math.pi * t)


def squish_rate_func(func, a=0.4, b=0.6):
    def result(t):
        if a == b:
            return a
        if t < a:
            return func(0)
        if t > b:
            return func(1)
        return func((t - a) / (b - a))
    return result


def lingering(t):
    return squish_rate_func(lambda t: t, 0, 0.8)(t)


def exponential_decay(t, half_life=0.1):
    return 1 - math.exp(-t / half_life)


def not_quite_there(func=smooth, proportion=0.7):
    def result(t):
        return proportion * func(t)
    return result


RATE_FUNCS = [
    ("linear", lambda t: t),
    ("smooth", smooth),
    ("rush_into", rush_into),
    ("rush_from", rush_from),
    ("slow_into", slow_into),
    ("double_smooth", double_smooth),
    ("there_and_back", there_and_back),
    ("there_and_back_with_pause", there_and_back_with_pause),
    ("running_start", running_start),
    ("overshoot", overshoot),
    ("wiggle", wiggle),
    ("lingering", lingering),
    ("exponential_decay", exponential_decay),
    ("not_quite_there_smooth_0.7", not_quite_there(smooth, 0.7)),
    ("squish_smooth_0.4_0.6", squish_rate_func(smooth, 0.4, 0.6)),
]


def gen_rate_functions() -> None:
    with open(os.path.join(OUT, "rate_functions.bin"), "wb") as fb, \
         open(os.path.join(OUT, "rate_functions.txt"), "w") as ft:
        ft.write("# fmn-core rate-function parity manifest (f64 LE in rate_functions.bin)\n")
        ft.write("# generated from 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13\n")
        for name, fn in RATE_FUNCS:
            samples = [float(fn(i / N)) for i in range(N + 1)]
            fb.write(struct.pack(f"<{len(samples)}d", *samples))
            ft.write(f"{name}\t{len(samples)}\n")
    total = (N + 1) * len(RATE_FUNCS)
    print(f"rate_functions.bin: {len(RATE_FUNCS)} functions, {total} samples")


# -------------------------------------------------------------------- color
def hex_to_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))


def interpolate(a, b, alpha):
    return (1 - alpha) * a + alpha * b


def interpolate_color_rgb(c1, c2, alpha):
    # utils/color.py: np.sqrt(interpolate(rgb1**2, rgb2**2, alpha))
    return tuple(math.sqrt(interpolate(x * x, y * y, alpha)) for x, y in zip(c1, c2))


def average_color_rgb(rgbs):
    # utils/color.py: np.sqrt((rgbs**2).mean(0))
    n = len(rgbs)
    return tuple(math.sqrt(sum(c[i] * c[i] for c in rgbs) / n) for i in range(3))


def gen_color_ops() -> None:
    palette = list(CFG["colors"].items())
    lines = []
    alphas = [0.0, 0.125, 0.3, 0.5, 0.7, 0.875, 1.0]
    # Interpolation: consecutive palette pairs (cycled) x alphas
    for i in range(len(palette)):
        (n1, h1), (n2, h2) = palette[i], palette[(i + 7) % len(palette)]
        c1, c2 = hex_to_rgb(h1), hex_to_rgb(h2)
        for a in alphas:
            out = interpolate_color_rgb(c1, c2, a)
            lines.append(f"interp\t{n1}\t{n2}\t{r(a)}\t" + "\t".join(map(r, out)))
    # Averages: full palette, and each 5-color family
    all_rgbs = [hex_to_rgb(h) for _, h in palette]
    lines.append("average\tALL\t" + "\t".join(map(r, average_color_rgb(all_rgbs))))
    for fam in ["blue", "teal", "green", "yellow", "gold", "red", "maroon", "purple", "grey"]:
        rgbs = [hex_to_rgb(h) for name, h in palette if name.startswith(fam + "_")]
        lines.append(f"average\t{fam}\t" + "\t".join(map(r, average_color_rgb(rgbs))))
    # color_gradient over COLORMAP_3B1B (blue_e, green_c, yellow_c, red_c), length 9
    refs = [hex_to_rgb(CFG["colors"][k]) for k in ("blue_e", "green_c", "yellow_c", "red_c")]
    length = 9
    n_ref = len(refs)
    for j in range(length):
        alpha = j * (n_ref - 1) / (length - 1)
        floor, am1 = int(alpha), alpha % 1
        if j == length - 1:
            floor, am1 = n_ref - 2, 1.0
        out = interpolate_color_rgb(refs[floor], refs[floor + 1], am1)
        lines.append(f"gradient3b1b\t{j}\t" + "\t".join(map(r, out)))

    with open(os.path.join(OUT, "color_ops.txt"), "w") as f:
        f.write("# fmn-core color-operation parity fixture\n")
        f.write("# generated from 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13\n")
        for line in lines:
            f.write(line + "\n")
    print(f"color_ops.txt: {len(lines)} entries")


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    gen_constants()
    gen_rate_functions()
    gen_color_ops()
