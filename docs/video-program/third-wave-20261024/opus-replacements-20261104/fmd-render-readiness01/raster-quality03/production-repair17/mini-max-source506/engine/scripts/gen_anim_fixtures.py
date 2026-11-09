#!/usr/bin/env python3
"""Generate fmn-anim normalized-alpha-pipeline fixtures from the pinned Reference.

Reference: 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13.
Reproduces the Reference's own arithmetic in pure Python doubles:
`Animation.get_sub_alpha` and `Animation.time_spanned_alpha`
(manimlib/animation/animation.py), `clip` (utils/simple_functions.py),
and the `smooth`/`linear` rate functions (utils/rate_functions.py).

Outputs (committed under crates/fmn-anim/fixtures/):
  sub_alpha.txt     rate<TAB>lag_ratio<TAB>num<TAB>index<TAB>alpha<TAB>expected
                    across lag_ratio values (incl. >1) and family sizes
  time_spanned.txt  run_time<TAB>start<TAB>end<TAB>alpha<TAB>expected, with
                    run_time pre-widened to max(run_time, end) as begin() does
"""

import os

OUT = "/data/projects/franken_manim/crates/fmn-anim/fixtures"


def r(x: float) -> str:
    return repr(float(x))


# --- the pinned formulas, line for line -----------------------------------
def clip(value: float, lower: float, upper: float) -> float:
    if value < lower:
        return lower
    if value > upper:
        return upper
    return value


def linear(t: float) -> float:
    return t


def smooth(t: float) -> float:
    s = 1 - t
    return (t**3) * (10 * s * s + 5 * s * t + t * t)


def get_sub_alpha(alpha: float, index: int, num: int, lag_ratio: float, rate) -> float:
    full_length = (num - 1) * lag_ratio + 1
    value = alpha * full_length
    lower = index * lag_ratio
    raw_sub_alpha = clip((value - lower), 0, 1)
    return rate(raw_sub_alpha)


def time_spanned_alpha(alpha: float, run_time: float, start: float, end: float) -> float:
    return clip(alpha * run_time - start, 0, end - start) / (end - start)


# --- corpora ---------------------------------------------------------------
RATES = [("linear", linear), ("smooth", smooth)]
LAG_RATIOS = [0.0, 0.05, 0.25, 0.5, 0.9, 1.0, 2.0]
FAMILY_SIZES = [1, 2, 3, 7, 25, 100]
ALPHAS = [i / 20 for i in range(21)]

SPANS = [(0.0, 1.0), (0.0, 0.5), (0.2, 0.8), (0.5, 2.0), (1.0, 3.0)]
RUN_TIMES = [1.0, 2.0, 2.5, 3.0]


def gen_sub_alpha() -> None:
    lines = []
    for rate_name, rate in RATES:
        for lag in LAG_RATIOS:
            for num in FAMILY_SIZES:
                indices = sorted({0, 1, num // 2, num - 1} & set(range(num)))
                for index in indices:
                    for alpha in ALPHAS:
                        value = get_sub_alpha(alpha, index, num, lag, rate)
                        lines.append(
                            "\t".join(
                                [rate_name, r(lag), str(num), str(index), r(alpha), r(value)]
                            )
                        )
    with open(os.path.join(OUT, "sub_alpha.txt"), "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"sub_alpha.txt: {len(lines)} cases")


def gen_time_spanned() -> None:
    lines = []
    for start, end in SPANS:
        for run_time in RUN_TIMES:
            widened = max(run_time, end)  # begin(): run_time = max(end, run_time)
            for alpha in ALPHAS:
                value = time_spanned_alpha(alpha, widened, start, end)
                lines.append("\t".join([r(widened), r(start), r(end), r(alpha), r(value)]))
    with open(os.path.join(OUT, "time_spanned.txt"), "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"time_spanned.txt: {len(lines)} cases")


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    gen_sub_alpha()
    gen_time_spanned()
