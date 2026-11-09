#!/usr/bin/env python3
"""Generate fmn-anim composition (§9.4) fixtures from the pinned Reference.

Reference: 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13.
Reproduces the Reference's own arithmetic in pure Python doubles:
`AnimationGroup.build_animations_with_timings` and `AnimationGroup.interpolate`
(manimlib/animation/composition.py), `interpolate` / `integer_interpolate`
(utils/bezier.py), and `clip` (utils/simple_functions.py).

Outputs (committed under crates/fmn-anim/fixtures/):
  composition_timings.txt  lag<TAB>run_times<TAB>starts<TAB>ends<TAB>max_end_time
                           the interval table, member for member
  composition_alpha.txt    lag<TAB>run_times<TAB>alpha<TAB>sub_alphas
                           AnimationGroup.interpolate's member sub-alphas
  succession_active.txt    run_times<TAB>alpha<TAB>ref_index<TAB>ref_sub_alpha
                                     <TAB>our_index<TAB>our_sub_alpha
                           BOTH answers per case: the Reference's equal-share
                           `integer_interpolate` pick and ours off the same
                           interval table the group uses. Where they differ,
                           the divergence is BN-11's, asserted deliberately —
                           never quietly absorbed into a tolerance.
"""

import os

OUT = "/data/projects/franken_manim/crates/fmn-anim/fixtures"


def r(x: float) -> str:
    return repr(float(x))


def joinf(xs) -> str:
    return ",".join(r(x) for x in xs)


# --- the pinned formulas, line for line -----------------------------------
def clip(value: float, lower: float, upper: float) -> float:
    if value < lower:
        return lower
    if value > upper:
        return upper
    return value


def interpolate(start: float, end: float, alpha: float) -> float:
    return (1 - alpha) * start + alpha * end


def integer_interpolate(start: int, end: int, alpha: float):
    if alpha >= 1:
        return (end - 1, 1.0)
    if alpha <= 0:
        return (start, 0)
    value = int(interpolate(start, end, alpha))
    residue = ((end - start) * alpha) % 1
    return (value, residue)


def build_animations_with_timings(run_times, lag_ratio: float):
    """AnimationGroup.build_animations_with_timings (composition.py:90)."""
    out = []
    curr_time = 0.0
    for run_time in run_times:
        start_time = curr_time
        end_time = start_time + run_time
        out.append((start_time, end_time))
        curr_time = interpolate(start_time, end_time, lag_ratio)
    return out


def group_sub_alphas(run_times, lag_ratio: float, alpha: float):
    """AnimationGroup.interpolate (composition.py:108), member for member."""
    timings = build_animations_with_timings(run_times, lag_ratio)
    max_end_time = max((end for _, end in timings), default=0.0)
    time = alpha * max_end_time
    out = []
    for start_time, end_time in timings:
        anim_time = end_time - start_time
        if anim_time == 0:
            out.append(0.0)
        else:
            out.append(clip((time - start_time) / anim_time, 0, 1))
    return out


def succession_ours(run_times, alpha: float):
    """Our Succession: the SAME interval table, at lag_ratio = 1."""
    timings = build_animations_with_timings(run_times, 1.0)
    max_end_time = max((end for _, end in timings), default=0.0)
    time = alpha * max_end_time
    index = 0
    for i, (start_time, _) in enumerate(timings):
        if time >= start_time:
            index = i
    start_time, end_time = timings[index]
    anim_time = end_time - start_time
    sub = 0.0 if anim_time == 0 else clip((time - start_time) / anim_time, 0, 1)
    return index, sub


# --- corpora ---------------------------------------------------------------
LAG_RATIOS = [0.0, 0.05, 0.25, 0.5, 1.0, 2.0]
RUN_TIME_SETS = [
    [1.0],
    [1.0, 1.0],
    [1.0, 1.0, 1.0],
    [3.0, 1.0],          # unequal: the Succession divergence's headline case
    [0.5, 2.0, 1.25],
    [2.0, 0.0, 1.0],     # a zero-length member (the anim_time == 0 branch)
    [1.0, 1.0, 1.0, 1.0, 1.0],
    [0.25, 0.5, 0.75, 1.0, 1.25, 1.5],
]
ALPHAS = [i / 20 for i in range(21)]


def gen_timings() -> None:
    lines = []
    for lag in LAG_RATIOS:
        for run_times in RUN_TIME_SETS:
            timings = build_animations_with_timings(run_times, lag)
            starts = [s for s, _ in timings]
            ends = [e for _, e in timings]
            max_end_time = max(ends, default=0.0)
            lines.append(
                "\t".join(
                    [r(lag), joinf(run_times), joinf(starts), joinf(ends), r(max_end_time)]
                )
            )
    with open(os.path.join(OUT, "composition_timings.txt"), "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"composition_timings.txt: {len(lines)} cases")


def gen_alpha() -> None:
    lines = []
    for lag in LAG_RATIOS:
        for run_times in RUN_TIME_SETS:
            for alpha in ALPHAS:
                subs = group_sub_alphas(run_times, lag, alpha)
                lines.append("\t".join([r(lag), joinf(run_times), r(alpha), joinf(subs)]))
    with open(os.path.join(OUT, "composition_alpha.txt"), "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"composition_alpha.txt: {len(lines)} cases")


def gen_succession() -> None:
    lines = []
    for run_times in RUN_TIME_SETS:
        n = len(run_times)
        for alpha in ALPHAS:
            ref_index, ref_sub = integer_interpolate(0, n, alpha)
            our_index, our_sub = succession_ours(run_times, alpha)
            lines.append(
                "\t".join(
                    [
                        joinf(run_times),
                        r(alpha),
                        str(ref_index),
                        r(ref_sub),
                        str(our_index),
                        r(our_sub),
                    ]
                )
            )
    with open(os.path.join(OUT, "succession_active.txt"), "w") as f:
        f.write("\n".join(lines) + "\n")
    differing = sum(1 for line in lines if line.split("\t")[2] != line.split("\t")[4])
    print(f"succession_active.txt: {len(lines)} cases, {differing} where BN-11 diverges")


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    gen_timings()
    gen_alpha()
    gen_succession()
