#!/usr/bin/env python3
"""Settled-wake history analyzer for the DFG 2D-2 cylinder test. Standard library only.

RubyRaven, 2026-11-21, Agent Mail 36195 (root BlueSalmon). Written against
tools/fluid_reference/dfg_tunnel.c and run_tunnel.py, and executes the phase
procedure of docs/.../dfg-phase-independent01 (tools/dfg_phase03.py, REPORT.md).

Subcommands
  run RUN_DIR --out OUT.json
      One solver run directory as run_tunnel.py leaves it: series.csv,
      settings.json, summary.json, and optionally timing.log and transfers.csv.
  featflow LOGDIR --level L --dt K --out OUT.json
      FEATFLOW 2014 2D-2 reference files bdforces_q2_lvL_dtK and
      pointvalues_q2_lvL_dtK, read through the same analysis.
  compare MANIFEST --out OUT.json
      Grid and step rows (analysis JSON files from `run`) against a reference
      analysis, with tolerances that root declares before the runs.

Definitions
  ST96 = Schaefer and Turek 1996, NNFM 48, section 2: St = D f / U with U the
  mean inflow speed 2 U(0, H/2) / 3; Delta P = P(0.15, 0.2) - P(0.25, 0.2); for 2D-2,
  t0 is the state with maximum lift and Delta P is read at t0 + 1/(2f).
  FF = FEATFLOW 2D-2 page: a cycle runs from one lift minimum to the next;
  mean = (max + min)/2 and amp = max - min over that cycle.

A trace yields measurements only when every gate passes:
  complete  real, finished observations on the declared clock (run only);
  case      flow "wake" (Re 100, U = 1 m/s) for the production solver;
  enough    a minimum lift swing, enough complete lift cycles delimited by
            lift minima, a settled trailing window of such cycles, enough
            samples per period, and a Delta P reading half a period after max lift.
Otherwise the status is INCOMPLETE, NOT_2D2 or INSUFFICIENT, and no period,
frequency or Strouhal number is reported, not even as a diagnostic.

Every output file is created exclusively; nothing is overwritten.
"""

import argparse
import bisect
import csv
import hashlib
import json
import math
import os
import statistics
import sys

TOOL = "wake_history.py"
SERIES_HEADER = [
    "t", "i", "wet_leaves", "allocated_cells", "drag", "lift", "pressure_difference",
    "relative_profile_l2", "weighted_div_max", "full_cell_div_max", "wet_div_l2", "net_flux",
    "energy", "wet_area", "pressure_residual", "pressure_cycles", "predictor_cycles",
    "viscous_cycles",
]
DIAMETER = 0.1
# dfg_tunnel.c: umax = wake ? 1.5 : .3; force scale uses U = 2 umax / 3.
UMAX = {"wake": 1.5, "steady": 0.3, "channel": 0.3}
DEFAULTS = {
    "min_cycles": 3,
    "rel_period_tol": 1e-3,
    "force_tol": 1e-3,
    "min_samples_per_period": 20,
    "min_lift_swing": 0.1,
    "hysteresis_fraction": 0.25,
    "edge_samples": 3,
    "clock_tol": 1e-9,
}
DEFAULT_NOTES = {
    "min_cycles": "default, root may declare another value",
    "rel_period_tol": "default; consecutive and last-cycle agreement of the min-to-min period",
    "force_tol": "default; absolute agreement of cL max/min, cD max/min and ST96 Delta P",
    "min_samples_per_period": "default; below it the peak and phase bias grows (see bias)",
    "min_lift_swing": "default; a shedding 2D-2 wake swings cL by about 2 (FEATFLOW 2014 lv6)",
    "hysteresis_fraction": "band = fraction x trailing-half lift swing, about the trailing midline",
    "edge_samples": "default; an excursion cut by the start or end of the data counts only with this many "
                    "monotone samples on each side of its extremum",
    "clock_tol": "allowed |t - k sample_dt| on the observation clock",
}


# ----------------------------------------------------------------- helpers

def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for block in iter(lambda: fh.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def vertex3(xs, ys):
    """Vertex of the parabola through three points with any spacing.

    Returns (x, y, ok); ok is False when the parabola is flat or its vertex
    lies outside [x0, x2], in which case the middle sample is returned."""
    x0, x1, x2 = xs
    y0, y1, y2 = ys
    d0, d2 = x0 - x1, x2 - x1
    r0, r2 = (y0 - y1) / d0, (y2 - y1) / d2
    a = (r2 - r0) / (d2 - d0)
    b = r0 - a * d0
    if a == 0.0:
        return x1, y1, False
    off = -b / (2.0 * a)
    if not d0 <= off <= d2:
        return x1, y1, False
    return x1 + off, y1 - b * b / (4.0 * a), True


def lsq_vertex(xs, ys):
    """Vertex of the least-squares parabola through the given points."""
    xm = xs[len(xs) // 2]
    u = [x - xm for x in xs]
    s = [sum(v ** k for v in u) for k in range(5)]
    r = [sum(y * v ** k for v, y in zip(u, ys)) for k in range(3)]
    m = [[s[0], s[1], s[2], r[0]], [s[1], s[2], s[3], r[1]], [s[2], s[3], s[4], r[2]]]
    for col in range(3):
        piv = max(range(col, 3), key=lambda k: abs(m[k][col]))
        m[col], m[piv] = m[piv], m[col]
        if m[col][col] == 0.0:
            return None
        for k in range(3):
            if k != col:
                fac = m[k][col] / m[col][col]
                m[k] = [p - fac * q for p, q in zip(m[k], m[col])]
    c0, c1, c2 = (m[k][3] / m[k][k] for k in range(3))
    if c2 == 0.0:
        return None
    xv = -c1 / (2.0 * c2)
    return xm + xv, c0 + c1 * xv + c2 * xv * xv


def bracket(ts, t):
    """Index i with ts[i] <= t <= ts[i+1], or None outside the samples."""
    if t < ts[0] or t > ts[-1]:
        return None
    i = bisect.bisect_right(ts, t) - 1
    return min(max(i, 0), len(ts) - 2)


def interp_linear(ts, ys, t):
    i = bracket(ts, t)
    if i is None:
        return None
    w = (t - ts[i]) / (ts[i + 1] - ts[i])
    return ys[i] + w * (ys[i + 1] - ys[i])


def interp_cubic(ts, ys, t):
    """Four-point Lagrange interpolation on samples i-1 .. i+2 around t."""
    i = bracket(ts, t)
    if i is None or i - 1 < 0 or i + 2 > len(ts) - 1:
        return None
    js = (i - 1, i, i + 1, i + 2)
    total = 0.0
    for j in js:
        w = 1.0
        for k in js:
            if k != j:
                w *= (t - ts[k]) / (ts[j] - ts[k])
        total += w * ys[j]
    return total


def nearest(ts, ys, t):
    i = bracket(ts, t)
    if i is None:
        return None
    j = i if abs(ts[i] - t) <= abs(ts[i + 1] - t) else i + 1
    return ys[j]


def time_average(ts, ys, a, b):
    """Trapezoidal mean of the sampled curve over [a, b], ends by linear interpolation."""
    pts = [(a, interp_linear(ts, ys, a))]
    lo, hi = bisect.bisect_right(ts, a), bisect.bisect_left(ts, b)
    pts += [(ts[k], ys[k]) for k in range(lo, hi)]
    pts.append((b, interp_linear(ts, ys, b)))
    area = sum(0.5 * (y0 + y1) * (x1 - x0) for (x0, y0), (x1, y1) in zip(pts, pts[1:]))
    return area / (b - a)


def refine(ts, ys, k):
    """Sample extremum k refined by a 3-point parabola (and a 5-point fit)."""
    out = {"sample_t": ts[k], "sample_value": ys[k]}
    if 0 < k < len(ts) - 1:
        tv, vv, ok = vertex3(ts[k - 1:k + 2], ys[k - 1:k + 2])
        out.update({"t": tv, "value": vv, "parabola_ok": ok})
    else:
        out.update({"t": ts[k], "value": ys[k], "parabola_ok": False})
    if 2 <= k < len(ts) - 2:
        fit = lsq_vertex(ts[k - 2:k + 3], ys[k - 2:k + 3])
        if fit is not None:
            out.update({"lsq5_t": fit[0], "lsq5_value": fit[1]})
    return out


def local_extrema(ys, kind):
    idx = []
    for k in range(1, len(ys) - 1):
        if kind == "max" and ys[k] > ys[k - 1] and ys[k] >= ys[k + 1]:
            idx.append(k)
        if kind == "min" and ys[k] < ys[k - 1] and ys[k] <= ys[k + 1]:
            idx.append(k)
    return idx


def excursions(ys, mid, band):
    """Partition samples into alternating low/high excursions with hysteresis.

    An excursion starts when the value leaves the band [mid - band, mid + band]
    on one side and lasts until it leaves on the other side. It is complete
    when it began and ended with such a transition."""
    segs, state, start, by_transition = [], None, None, False
    for k, v in enumerate(ys):
        side = "low" if v < mid - band else "high" if v > mid + band else None
        if side is None or side == state:
            continue
        if state is not None:
            segs.append({"kind": state, "start": start, "end": k - 1,
                         "complete": by_transition})
        state, start, by_transition = side, k, state is not None
    if state is not None:
        segs.append({"kind": state, "start": start, "end": len(ys) - 1, "complete": False})
    return segs


def sinusoid_parabola_bias(h, period, steps=2000):
    """Worst errors of the 3-point vertex on cos(2 pi t / period) sampled every h.

    Returns (value error in units of the amplitude, time error in seconds,
    largest-sample miss in units of the amplitude). A model for a single
    harmonic; it says nothing about harmonics, noise or regrid jumps."""
    w = 2.0 * math.pi / period
    worst_v = worst_t = 0.0
    for j in range(-steps, steps + 1):
        d = 0.5 * h * j / steps
        xs = [d - h, d, d + h]
        tv, vv, _ = vertex3(xs, [math.cos(w * x) for x in xs])
        worst_v, worst_t = max(worst_v, abs(vv - 1.0)), max(worst_t, abs(tv))
    return worst_v, worst_t, 1.0 - math.cos(math.pi * h / period)


# ------------------------------------------------------------ the analysis

def analyse_history(ts, cd, cl, dp, umean, params):
    """Cycles, settled window, gates and (when every gate passes) measurements."""
    n = len(ts)
    gates, flags = [], []

    def gate(name, ok, detail):
        gates.append({"category": "enough", "gate": name, "ok": bool(ok), "detail": detail})

    res = {"samples": n, "gates": gates, "flags": flags}
    if n < 7:
        gate("samples", False, "%d samples; at least 7 are needed" % n)
        return res
    half = (ts[0] + ts[-1]) / 2.0
    tail = [y for t, y in zip(ts, cl) if t >= half]
    swing = max(tail) - min(tail)
    mid = 0.5 * (max(tail) + min(tail))
    band = params["hysteresis_fraction"] * swing
    res["trailing_half_lift"] = {"from_t": half, "sample_min": min(tail), "sample_max": max(tail),
                                 "swing": swing}
    gate("lift swing", swing >= params["min_lift_swing"],
         "trailing-half sample swing %.6g against minimum %.6g" % (swing, params["min_lift_swing"]))
    if swing < params["min_lift_swing"]:
        gate("complete lift cycles", False, "no lift excursions evaluated: swing below minimum")
        return res

    segs = excursions(cl, mid, band)
    cl_ext = {"min": local_extrema(cl, "min"), "max": local_extrema(cl, "max")}
    for s in segs:
        low = s["kind"] == "low"
        k = (min if low else max)(range(s["start"], s["end"] + 1), key=lambda j: cl[j])
        s["extremum"] = refine(ts, cl, k)
        s["index"] = k
        s["sample_extrema"] = sum(1 for j in cl_ext["min" if low else "max"] if s["start"] <= j <= s["end"])
        # A complete excursion confirms its extremum. An excursion cut by the
        # start or end of the data counts only with edge_samples samples moving
        # away monotonically on each side, inside the excursion.
        sign, e = (1.0 if low else -1.0), params["edge_samples"]
        shoulders = (e >= 1 and k - e >= s["start"] and k + e <= s["end"] and
                     all(sign * (cl[j - 1] - cl[j]) > 0 for j in range(k - e + 1, k + 1)) and
                     all(sign * (cl[j + 1] - cl[j]) > 0 for j in range(k, k + e)))
        s["confirmed"] = "complete excursion" if s["complete"] else (
            "%d monotone samples each side" % e if shoulders else None)
    lows = [s for s in segs if s["kind"] == "low" and s["confirmed"]]
    highs = [s for s in segs if s["kind"] == "high" and s["confirmed"]]

    cd_max = [refine(ts, cd, k) for k in local_extrema(cd, "max")]
    cd_min = [refine(ts, cd, k) for k in local_extrema(cd, "min")]
    dp_max = [refine(ts, dp, k) for k in local_extrema(dp, "max")]
    dp_min = [refine(ts, dp, k) for k in local_extrema(dp, "min")]

    def inside(points, a, b):
        return [p for p in points if a <= p["t"] < b]

    spacing = statistics.median(b - a for a, b in zip(ts, ts[1:]))
    cycles = []
    for lo0, lo1 in zip(lows, lows[1:]):
        hi = [s for s in highs if lo0["index"] < s["index"] < lo1["index"]]
        if len(hi) != 1:
            flags.append("lift minima at %.9g and %.9g bracket %d complete maxima" %
                         (lo0["extremum"]["t"], lo1["extremum"]["t"], len(hi)))
            continue
        hi = hi[0]
        t0, t1 = lo0["extremum"]["t"], lo1["extremum"]["t"]
        period = t1 - t0
        cyc = {"t0": t0, "t1": t1, "period_min_to_min": period,
               "lift_min_start": lo0["extremum"], "lift_min_end": lo1["extremum"],
               "lift_max": hi["extremum"],
               "single_peaked": lo0["sample_extrema"] == 1 and lo1["sample_extrema"] == 1
               and hi["sample_extrema"] == 1}
        cl_min = min(lo0["extremum"]["value"], lo1["extremum"]["value"])
        cl_max = hi["extremum"]["value"]
        cyc["lift"] = {"max": cl_max, "min": cl_min, "mean_ff": 0.5 * (cl_max + cl_min),
                       "amp_ff": cl_max - cl_min, "time_average": time_average(ts, cl, t0, t1),
                       "max_offset_s": hi["extremum"]["t"] - t0,
                       "max_offset_fraction": (hi["extremum"]["t"] - t0) / period}
        for name, ys, mx, mn in (("drag", cd, cd_max, cd_min), ("pressure_difference", dp, dp_max, dp_min)):
            ins_max, ins_min = inside(mx, t0, t1), inside(mn, t0, t1)
            if not ins_max or not ins_min:
                cyc[name] = None
                flags.append("%s has no local maximum or minimum in the cycle from %.9g" % (name, t0))
                continue
            top = max(ins_max, key=lambda p: p["value"])
            bot = min(ins_min, key=lambda p: p["value"])
            cyc[name] = {"max": top["value"], "max_t": top["t"], "max_detail": top,
                         "min": bot["value"], "min_t": bot["t"],
                         "mean_ff": 0.5 * (top["value"] + bot["value"]),
                         "amp_ff": top["value"] - bot["value"],
                         "time_average": time_average(ts, ys, t0, t1),
                         "local_maxima": len(ins_max),
                         "max_offset_s": top["t"] - t0, "max_offset_fraction": (top["t"] - t0) / period}
        # ST96: Delta P half a period after the lift maximum, the period from
        # this maximum to the next one (FEATFLOW cycles do not define it).
        later = [s for s in highs if s["index"] > hi["index"]]
        if later:
            t_period, source = later[0]["extremum"]["t"] - hi["extremum"]["t"], "max_to_max"
        else:
            t_period, source = period, "min_to_min_fallback"
        t_eval = hi["extremum"]["t"] + 0.5 * t_period
        cub = interp_cubic(ts, dp, t_eval)
        if cub is None:
            cyc["st96_delta_p"] = None
        else:
            h = spacing / 10.0
            up, dn = interp_cubic(ts, dp, t_eval + h), interp_cubic(ts, dp, t_eval - h)
            cyc["st96_delta_p"] = {
                "t_max_lift": hi["extremum"]["t"], "period": t_period, "period_source": source,
                "t_eval": t_eval, "cubic": cub, "linear": interp_linear(ts, dp, t_eval),
                "nearest": nearest(ts, dp, t_eval),
                "slope_per_s": None if up is None or dn is None else (up - dn) / (2 * h)}
        cycles.append(cyc)
    res["complete_cycles"] = len(cycles)
    gate("complete lift cycles", len(cycles) >= params["min_cycles"],
         "%d complete cycles between lift minima; at least %d required" % (len(cycles), params["min_cycles"]))

    def agree(a, b):
        why = []
        if abs(a["period_min_to_min"] - b["period_min_to_min"]) > params["rel_period_tol"] * b["period_min_to_min"]:
            why.append("period")
        pairs = [("lift max", a["lift"]["max"], b["lift"]["max"]), ("lift min", a["lift"]["min"], b["lift"]["min"])]
        for name in ("drag",):
            if a[name] is None or b[name] is None:
                why.append(name + " extrema missing")
            else:
                pairs += [(name + " max", a[name]["max"], b[name]["max"]), (name + " min", a[name]["min"], b[name]["min"])]
        if a["st96_delta_p"] is not None and b["st96_delta_p"] is not None:
            pairs.append(("ST96 Delta P", a["st96_delta_p"]["cubic"], b["st96_delta_p"]["cubic"]))
        why += [nm for nm, x, y in pairs if abs(x - y) > params["force_tol"]]
        return why

    settled = []
    if cycles:
        settled = [len(cycles) - 1]
        for k in range(len(cycles) - 2, -1, -1):
            if agree(cycles[k], cycles[settled[0]]) or agree(cycles[k], cycles[-1]):
                break
            settled.insert(0, k)
    res["settled_cycles"] = len(settled)
    gate("settled window", len(settled) >= params["min_cycles"],
         "%d trailing cycles agree with their successor and with the last cycle; at least %d required"
         % (len(settled), params["min_cycles"]))
    if len(settled) < params["min_cycles"]:
        return res
    win = [cycles[k] for k in settled]
    t_lo, t_hi = win[0]["t0"], win[-1]["t1"]
    spacings = [b - a for a, b in zip(ts, ts[1:]) if t_lo <= a and b <= t_hi]
    h = statistics.median(spacings)
    mean_period = sum(c["period_min_to_min"] for c in win) / len(win)
    spp = mean_period / h
    gate("samples per period", spp >= params["min_samples_per_period"],
         "%.2f samples per period in the settled window; at least %g required" % (spp, params["min_samples_per_period"]))
    multi = sum(1 for c in win if not c["single_peaked"])
    if multi:
        flags.append("%d settled cycles have more than one sample-level lift extremum in an excursion "
                     "(noise or regrid jumps; see bias.observed)" % multi)
    gate("drag and pressure extrema", all(c["drag"] and c["pressure_difference"] for c in win),
         "every settled cycle has local drag and pressure-difference extrema")
    with_dp = [c for c in win if c["st96_delta_p"] is not None]
    gate("ST96 Delta P reading", bool(with_dp),
         "%d settled cycles have Delta P at max lift + T/2 inside the samples" % len(with_dp))
    if not all(g["ok"] for g in gates):
        return res

    full = [c for c in with_dp if c["st96_delta_p"]["period_source"] == "max_to_max"]
    rep = (full or with_dp)[-1]
    res["settled_window"] = {"t_from": t_lo, "t_to": t_hi, "cycles": len(win), "multi_extremum_cycles": multi,
                             "median_sample_spacing": h, "samples_per_period": spp,
                             "spacing_min": min(spacings), "spacing_max": max(spacings)}
    res["cycles"] = cycles
    res["settled_indices"] = settled
    res["representative_cycle"] = cycles.index(rep)

    def spread(get):
        vals = [get(c) for c in win if get(c) is not None]
        return {"min": min(vals), "max": max(vals), "range": max(vals) - min(vals), "n": len(vals)}

    m = {
        "strouhal": DIAMETER / (rep["period_min_to_min"] * umean),
        "period": rep["period_min_to_min"],
        "frequency": 1.0 / rep["period_min_to_min"],
        "strouhal_settled_mean": DIAMETER / (mean_period * umean),
        "drag": {k: rep["drag"][k] for k in ("max", "min", "mean_ff", "amp_ff", "time_average")},
        "lift": {k: rep["lift"][k] for k in ("max", "min", "mean_ff", "amp_ff", "time_average")},
        "pressure_difference_curve": {k: rep["pressure_difference"][k] for k in ("max", "min", "mean_ff", "amp_ff", "time_average")},
        "st96_delta_p": rep["st96_delta_p"]["cubic"],
        "st96_delta_p_detail": rep["st96_delta_p"],
        "phase": {"cycle_start": "lift minimum (FEATFLOW)",
                  "drag_max_after_start_s": rep["drag"]["max_offset_s"],
                  "drag_max_after_start_fraction": rep["drag"]["max_offset_fraction"],
                  "lift_max_after_start_s": rep["lift"]["max_offset_s"],
                  "lift_max_after_start_fraction": rep["lift"]["max_offset_fraction"],
                  "drag_local_maxima_per_cycle": rep["drag"]["local_maxima"]},
        "window_maxima": {"drag_max": max(c["drag"]["max"] for c in win),
                          "lift_max": max(c["lift"]["max"] for c in win)},
        "settled_spread": {
            "period": spread(lambda c: c["period_min_to_min"]),
            "drag_max": spread(lambda c: c["drag"]["max"]),
            "drag_min": spread(lambda c: c["drag"]["min"]),
            "lift_max": spread(lambda c: c["lift"]["max"]),
            "lift_min": spread(lambda c: c["lift"]["min"]),
            "st96_delta_p": spread(lambda c: c["st96_delta_p"]["cubic"] if c["st96_delta_p"] else None),
        },
    }
    res["measurements"] = m

    # Sampling and interpolation bias: observed estimator spread, plus a
    # single-harmonic model at this spacing (drag's dominant harmonic is 2f).
    lv, lt, lmiss = sinusoid_parabola_bias(h, rep["period_min_to_min"])
    dv, dt_, dmiss = sinusoid_parabola_bias(h, rep["period_min_to_min"] / 2.0)
    a_l, a_d = rep["lift"]["amp_ff"] / 2.0, rep["drag"]["amp_ff"] / 2.0
    lm, dm = rep["lift_max"], rep["drag"]["max_detail"]
    st = rep["st96_delta_p"]
    slope = st["slope_per_s"] or 0.0
    res["bias"] = {
        "observed": {
            "lift_max_parabola_minus_largest_sample": lm["value"] - lm["sample_value"],
            "lift_max_parabola_minus_lsq5": (lm["value"] - lm["lsq5_value"]) if "lsq5_value" in lm else None,
            "drag_max_parabola_minus_largest_sample": dm["value"] - dm["sample_value"],
            "drag_max_parabola_minus_lsq5": (dm["value"] - dm["lsq5_value"]) if "lsq5_value" in dm else None,
            "delta_p_cubic_minus_linear": st["cubic"] - st["linear"],
            "delta_p_cubic_minus_nearest": st["cubic"] - st["nearest"],
            "delta_p_slope_per_s": slope,
        },
        "single_harmonic_model": {
            "note": "model, not a measurement: one cosine at the cycle period (lift) or half of it (drag), "
                    "amplitude = amp_ff/2, sampled every median spacing",
            "lift_largest_sample_miss_max": a_l * lmiss,
            "lift_parabola_value_error_max": a_l * lv,
            "lift_parabola_time_error_max_s": lt,
            "drag_largest_sample_miss_max": a_d * dmiss,
            "drag_parabola_value_error_max": a_d * dv,
            "drag_parabola_time_error_max_s": dt_,
            "delta_p_from_peak_time_error_max": abs(slope) * lt,
        },
    }
    res["representative_samples"] = [
        [t - rep["t0"], d, c, p] for t, d, c, p in zip(ts, cd, cl, dp)
        if rep["t0"] - 2 * h <= t <= rep["t1"] + 2 * h]
    return res


# --------------------------------------------------------------- loaders

def read_series(path):
    with open(path, newline="") as fh:
        reader = csv.reader(fh)
        header = next(reader, None)
        rows = list(reader)
    return header, rows


def parse_timing(path):
    out = {}
    with open(path) as fh:
        for line in fh:
            f = line.split()
            if len(f) == 6 and f[1] == "real" and f[3] == "user":
                out["real_s"] = float(f[0])
            elif line.strip().endswith("maximum resident set size"):
                out["max_rss_bytes"] = int(f[0])
            elif line.strip().endswith("peak memory footprint"):
                out["peak_memory_footprint_bytes"] = int(f[0])
    return out


def load_run(run_dir, params):
    gates = []

    def gate(cat, name, ok, detail):
        gates.append({"category": cat, "gate": name, "ok": bool(ok), "detail": detail})

    paths = {k: os.path.join(run_dir, k) for k in
             ("series.csv", "settings.json", "summary.json", "timing.log", "transfers.csv")}
    inputs = {k: {"path": p, "sha256": sha256(p)} for k, p in paths.items() if os.path.isfile(p)}
    out = {"source": "production run", "run_dir": run_dir, "inputs": inputs, "gates": gates}
    for k in ("series.csv", "settings.json", "summary.json"):
        gate("complete", k + " present", k in inputs,
             "written by run_tunnel.py; summary.json only after the solver exits successfully")
    case = None
    if "settings.json" in inputs:
        with open(paths["settings.json"]) as fh:
            case = json.load(fh).get("case")
        out["settings"] = case
    data = None
    if "series.csv" in inputs:
        header, rows = read_series(paths["series.csv"])
        gate("complete", "series.csv header", header == SERIES_HEADER,
             "exact dfg_tunnel.c column list" if header == SERIES_HEADER else "header differs: %r" % header)
        try:
            vals = [[float(x) for x in r] for r in rows]
            ok = all(len(r) == len(SERIES_HEADER) for r in vals) and all(math.isfinite(x) for r in vals for x in r)
        except ValueError:
            vals, ok = [], False
        gate("complete", "numeric finite rows", ok and len(vals) >= 2, "%d data rows" % len(rows))
        if ok and len(vals) >= 2 and header == SERIES_HEADER:
            data = {name: [r[k] for r in vals] for k, name in enumerate(SERIES_HEADER)}
    if data:
        ts, steps = data["t"], data["i"]
        gate("complete", "clock increasing", all(a < b for a, b in zip(ts, ts[1:])) and
             all(a < b for a, b in zip(steps, steps[1:])), "t and step index strictly increase")
        if case:
            sdt, dur = float(case["sample_dt"]), float(case["duration"])
            expected = [k * sdt for k in range(int(math.floor(dur / sdt + 1e-9)) + 1)]
            if dur - expected[-1] > 1e-11:
                expected.append(dur)
            dev = max((abs(a - b) / max(1.0, abs(b)) for a, b in zip(ts, expected)), default=float("inf"))
            gate("complete", "observation clock", len(ts) == len(expected) and dev <= params["clock_tol"],
                 "%d observations of %d expected at k x %g s up to %g s; largest deviation %.3g"
                 % (len(ts), len(expected), sdt, dur, dev))
        if "summary.json" in inputs:
            with open(paths["summary.json"]) as fh:
                summary = json.loads(fh.read().strip().splitlines()[-1])
            out["summary"] = summary
            gate("complete", "summary matches last observation",
                 abs(summary["t"] - ts[-1]) <= 1e-10 and int(summary["steps"]) == int(steps[-1]),
                 "summary t %.17g steps %s; last row t %.17g step %d"
                 % (summary["t"], summary["steps"], ts[-1], steps[-1]))
            gate("complete", "no capped solves", summary.get("failed_solve_caps") == 0,
                 "failed_solve_caps = %r" % summary.get("failed_solve_caps"))
    flow = case.get("flow") if case else None
    gate("case", "DFG 2D-2 wake", flow == "wake",
         "flow %r: %s" % (flow, {"wake": "2D-2, Re 100, U = 1 m/s", "steady": "ST96 2D-1, Re 20 (steady test)",
                                 "channel": "empty channel, no cylinder"}.get(flow, "unknown")))
    if "timing.log" in inputs:
        out["timing"] = parse_timing(paths["timing.log"])
    if "transfers.csv" in inputs:
        _, trows = read_series(paths["transfers.csv"])
        out["transfer_rows"] = len(trows)
    return out, data, flow


def cost_columns(out, data, analysis):
    """Cells, steps, solver step and time over the settled window (W1 cost columns)."""
    win = analysis.get("settled_window")
    if not win or not data:
        return None
    ts, st = data["t"], data["i"]
    idx = [k for k, t in enumerate(ts) if win["t_from"] <= t <= win["t_to"]]
    wet = [data["wet_leaves"][k] for k in idx]
    alloc = [data["allocated_cells"][k] for k in idx]
    per_obs = {}
    for a, b in zip(idx, idx[1:]):
        key = str(int(st[b] - st[a]))
        per_obs[key] = per_obs.get(key, 0) + 1
    cols = {
        "wet_leaves": {"min": min(wet), "max": max(wet), "mean_over_observations": sum(wet) / len(wet)},
        "allocated_cells": {"min": min(alloc), "max": max(alloc)},
        "steps_in_window": int(st[idx[-1]] - st[idx[0]]),
        "mean_solver_step_in_window": (ts[idx[-1]] - ts[idx[0]]) / (st[idx[-1]] - st[idx[0]]),
        "steps_per_observation_interval": per_obs,
        "nominal_dt": (out.get("settings") or {}).get("dt"),
        "sample_dt": (out.get("settings") or {}).get("sample_dt"),
    }
    s = out.get("summary") or {}
    for k in ("steps", "elapsed_solver_s", "min_dt", "max_dt", "regrids", "wet_leaf_steps", "allocated_cell_steps"):
        if k in s:
            cols["run_" + k] = s[k]
    cols.update({"run_" + k: v for k, v in (out.get("timing") or {}).items()})
    trp = os.path.join(out["run_dir"], "transfers.csv")
    if os.path.isfile(trp):
        header, rows = read_series(trp)
        col = {name: k for k, name in enumerate(header or [])}
        jumps = [(abs(float(r[col["drag_after"]]) - float(r[col["drag_before"]])),
                  abs(float(r[col["lift_after"]]) - float(r[col["lift_before"]])))
                 for r in rows if win["t_from"] <= float(r[col["t"]]) <= win["t_to"]]
        cols["regrids_in_window"] = len(jumps)
        if jumps:
            cols["largest_regrid_force_jump"] = {"drag": max(j[0] for j in jumps), "lift": max(j[1] for j in jumps)}
    return cols


def load_featflow(logdir, level, dt):
    """FEATFLOW 2014 2D-2 files: identical stamps, probes (0.15, 0.2) and (0.25, 0.2)."""
    fp = os.path.join(logdir, "bdforces_q2_lv%d_dt%d" % (level, dt))
    pp = os.path.join(logdir, "pointvalues_q2_lv%d_dt%d" % (level, dt))
    tf, cd, cl, tp, dp = [], [], [], [], []
    with open(fp) as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            f = line.split()
            if f[2] != "2":
                raise ValueError("unexpected boundary component %r" % f[2])
            tf.append(float(f[1]))
            cd.append(float(f[3]))
            cl.append(float(f[4]))
    with open(pp) as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            f = line.split()
            a = (float(f[2]), float(f[3]), f[4], f[5], float(f[6]))
            b = (float(f[7]), float(f[8]), f[9], f[10], float(f[11]))
            if not (abs(a[0] - 0.15) < 1e-9 and abs(a[1] - 0.2) < 2e-6 and a[2:4] == ("3", "0")
                    and abs(b[0] - 0.25) < 1e-9 and abs(b[1] - 0.2) < 2e-6 and b[2:4] == ("3", "0")):
                raise ValueError("unexpected probe columns: %r" % line)
            tp.append(float(f[1]))
            dp.append(a[4] - b[4])
    if tf != tp:
        raise ValueError("force and point files have different time stamps")
    inputs = {os.path.basename(p): {"path": p, "sha256": sha256(p)} for p in (fp, pp)}
    return tf, cd, cl, dp, inputs


# ------------------------------------------------------------ commands

def status_of(gates):
    for cat, status in (("complete", "INCOMPLETE"), ("case", "NOT_2D2"), ("enough", "INSUFFICIENT")):
        if any(g["category"] == cat and not g["ok"] for g in gates):
            return status
    return "OK"


def finish(result, path):
    gates = result["gates"]
    result["status"] = status_of(gates)
    result["reasons"] = ["%s: %s" % (g["gate"], g["detail"]) for g in gates if not g["ok"]]
    if result["status"] != "OK":
        a = result.get("analysis") or {}
        result["analysis"] = {k: a[k] for k in ("samples", "trailing_half_lift", "complete_cycles",
                                                 "settled_cycles") if k in a}
        result["analysis"]["structure_flags"] = len(a.get("flags", []))
        result["analysis"]["refused"] = ("no extremum time, period, frequency or Strouhal number "
                                         "is reported for this trace")
        result.pop("table_row", None)
    with open(path, "x") as fh:
        json.dump(result, fh, indent=1, sort_keys=True)
        fh.write("\n")
    line = "%s %s" % (result["status"], result.get("label", ""))
    if result["status"] == "OK":
        m = result["analysis"]["measurements"]
        line += " St %.7f cDmax %.7f cLmax %.7f dP(ST96) %.7f" % (
            m["strouhal"], m["drag"]["max"], m["lift"]["max"], m["st96_delta_p"])
    else:
        line += " | " + " | ".join(result["reasons"])
    print(line)
    return result


def table_row(result, costs):
    m = result["analysis"]["measurements"]
    row = {"strouhal": m["strouhal"],
           "drag_peak": m["drag"]["max"], "drag_middle_ff": m["drag"]["mean_ff"], "drag_swing": m["drag"]["amp_ff"],
           "lift_peak": m["lift"]["max"], "lift_middle_ff": m["lift"]["mean_ff"], "lift_swing": m["lift"]["amp_ff"],
           "drag_time_average": m["drag"]["time_average"], "lift_time_average": m["lift"]["time_average"],
           "delta_p_st96": m["st96_delta_p"],
           "curve": "representative_samples (t - t0, cD, cL, Delta P), t0 = lift minimum"}
    if costs:
        row.update({"cells_wet_leaves_max": costs["wet_leaves"]["max"],
                    "cells_wet_leaves_mean": costs["wet_leaves"]["mean_over_observations"],
                    "steps_run": costs.get("run_steps"), "time_on_clock_s": costs.get("run_elapsed_solver_s"),
                    "memory_max_rss_bytes": costs.get("run_max_rss_bytes")})
    return row


def cmd_run(args, params):
    out, data, flow = load_run(args.run_dir, params)
    out.update({"tool": TOOL, "label": args.label or args.run_dir, "params": params, "param_notes": DEFAULT_NOTES})
    if data:
        umean = 2.0 * UMAX.get(flow, 1.5) / 3.0
        out["constants"] = {"diameter": DIAMETER, "mean_inflow_speed": umean,
                            "lift_sign": "force of the fluid on the cylinder, +y towards the top wall "
                                         "(Basilisk embed_force; ST96 F_L)"}
        an = analyse_history(data["t"], data["drag"], data["lift"], data["pressure_difference"], umean, params)
        out["gates"] += an.pop("gates")
        out["analysis"] = an
        costs = cost_columns(out, data, an)
        if costs:
            out["costs"] = costs
        if status_of(out["gates"]) == "OK":
            out["table_row"] = table_row(out, costs)
    return finish(out, args.out)


def cmd_featflow(args, params):
    ts, cd, cl, dp, inputs = load_featflow(args.logdir, args.level, args.dt)
    out = {"tool": TOOL, "source": "FEATFLOW 2014 2D-2 reference (Q2/P1disc), not our solver",
           "label": args.label or "featflow lv%d dt%d" % (args.level, args.dt),
           "inputs": inputs, "params": params, "param_notes": DEFAULT_NOTES,
           "constants": {"diameter": DIAMETER, "mean_inflow_speed": 1.0},
           "gates": [{"category": "complete", "gate": "reference files", "ok": True,
                      "detail": "identical force and point stamps; probes (0.15, 0.2) and (0.25, 0.2)"}]}
    an = analyse_history(ts, cd, cl, dp, 1.0, params)
    out["gates"] += an.pop("gates")
    out["analysis"] = an
    if status_of(out["gates"]) == "OK":
        out["table_row"] = table_row(out, None)
    return finish(out, args.out)


def curve_difference(a_samples, b_samples, period):
    """Max and RMS difference of cD and cL over one swing in physical time from t0."""
    n = 400
    ta, tb = [r[0] for r in a_samples], [r[0] for r in b_samples]
    cols = {name: ([r[k] for r in a_samples], [r[k] for r in b_samples]) for name, k in (("drag", 1), ("lift", 2))}
    diffs = {"drag": [], "lift": []}
    for j in range(n + 1):
        tau = period * j / n
        for name in ("drag", "lift"):
            ya, yb = cols[name]
            va, vb = interp_cubic(ta, ya, tau), interp_cubic(tb, yb, tau)
            if va is None or vb is None:
                return None
            diffs[name].append(va - vb)
    return {name: {"max_abs": max(abs(d) for d in v), "rms": math.sqrt(sum(d * d for d in v) / len(v))}
            for name, v in diffs.items()}


def cmd_compare(args, params):
    with open(args.manifest) as fh:
        man = json.load(fh)
    refusals = []

    def load(p):
        with open(p) as fh:
            return json.load(fh)
    ref = load(man["reference"])
    rows = [(r["role"], r["analysis"], load(r["analysis"])) for r in man["rows"]]
    if ref.get("status") != "OK":
        refusals.append("reference status %s" % ref.get("status"))
    for role, path, a in rows:
        if a.get("status") != "OK":
            refusals.append("%s (%s): status %s" % (role, path, a.get("status")))
    roles = [r[0] for r in rows]
    if roles != ["grid1", "grid2", "grid3", "step2", "step3"]:
        refusals.append("rows must be grid1, grid2, grid3 (smallest step), step2, step3 (finest grid); got %r" % roles)
    out = {"tool": TOOL, "manifest": args.manifest, "close_enough": man.get("close_enough"), "refusals": refusals}
    if not refusals:
        st = {role: a["settings"] for role, _, a in rows}
        if len({json.dumps(a["params"], sort_keys=True) for _, _, a in rows}) != 1:
            refusals.append("analysis parameters differ between rows")
        if len({st[r]["sample_dt"] for r in st}) != 1:
            refusals.append("sample_dt differs between rows")
        if not (st["grid1"]["level"] < st["grid2"]["level"] < st["grid3"]["level"] == st["step2"]["level"] == st["step3"]["level"]):
            refusals.append("levels must rise over grids 1-3 and stay at grid 3 for steps 2-3")
        if not (st["grid1"]["dt"] == st["grid2"]["dt"] == st["grid3"]["dt"] < st["step2"]["dt"] < st["step3"]["dt"]):
            refusals.append("nominal dt must be equal over grids 1-3 and grow for steps 2-3")
    if refusals:
        out["slots"] = None
        with open(args.out, "x") as fh:
            json.dump(out, fh, indent=1, sort_keys=True)
            fh.write("\n")
        print("REFUSED | " + " | ".join(refusals))
        return out
    keys = ["strouhal", "drag_peak", "drag_middle_ff", "drag_swing", "lift_peak", "lift_middle_ff",
            "lift_swing", "delta_p_st96"]
    table = {role: a["table_row"] for role, _, a in rows}
    table["reference"] = ref["table_row"]
    changes = {}
    for a, b in (("grid1", "grid2"), ("grid2", "grid3"), ("grid3", "step2"), ("step2", "step3")):
        changes["%s_to_%s" % (a, b)] = {k: table[b][k] - table[a][k] for k in keys}
    against_ref = {role: {k: table[role][k] - table["reference"][k] for k in keys} for role in roles}
    curves = {}
    for role, _, a in rows:
        p = min(a["analysis"]["measurements"]["period"], ref["analysis"]["measurements"]["period"])
        curves[role] = curve_difference(a["analysis"]["representative_samples"],
                                        ref["analysis"]["representative_samples"], p)
    out.update({"table": table, "changes": changes, "against_reference": against_ref,
                "curve_against_reference": curves})
    tol = man.get("close_enough")
    verdict = None
    unavailable = []
    if tol:
        errors = {k: abs(v) for k, v in against_ref["grid3"].items()}
        errors.update({"curve_" + name: metric["max_abs"]
                       for name, metric in (curves["grid3"] or {}).items()})
        unavailable = sorted(tol.keys() - errors.keys())
        verdict = {name: name in errors and type(limit) in (int, float)
                   and 0 <= errors[name] <= limit < math.inf
                   for name, limit in tol.items()}
    out["meets_declared_tolerance"] = verdict
    if verdict and all(verdict.values()):
        g3 = rows[2][2]
        out["slots"] = {"20_v1.St": table["grid3"]["strouhal"],
                        "20_v1.d1": changes["grid1_to_grid2"]["strouhal"],
                        "20_v1.d2": changes["grid2_to_grid3"]["strouhal"],
                        "20_v1.cells": table["grid3"].get("cells_wet_leaves_max"),
                        "20_v1.steps": table["grid3"].get("steps_run"),
                        "20_v1.time_s": table["grid3"].get("time_on_clock_s"),
                        "source": g3["label"]}
    else:
        out["slots"] = None
        if unavailable:
            out["slots_withheld"] = "missing measured values for " + ", ".join(unavailable)
        else:
            out["slots_withheld"] = "no declared tolerance" if not tol else "a declared tolerance is not met"
    with open(args.out, "x") as fh:
        json.dump(out, fh, indent=1, sort_keys=True)
        fh.write("\n")
    print("COMPARED", "slots filled" if out["slots"] else "slots withheld: " + out["slots_withheld"])
    return out


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)

    def common(sp):
        sp.add_argument("--out", required=True, help="output JSON, created exclusively")
        sp.add_argument("--label")
        for k, v in DEFAULTS.items():
            sp.add_argument("--" + k.replace("_", "-"), type=type(v), default=v)
    r = sub.add_parser("run")
    r.add_argument("run_dir")
    common(r)
    f = sub.add_parser("featflow")
    f.add_argument("logdir")
    f.add_argument("--level", type=int, required=True)
    f.add_argument("--dt", type=int, required=True)
    common(f)
    c = sub.add_parser("compare")
    c.add_argument("manifest")
    c.add_argument("--out", required=True)
    args = p.parse_args(argv)
    params = {k: getattr(args, k) for k in DEFAULTS if hasattr(args, k)}
    return {"run": cmd_run, "featflow": cmd_featflow, "compare": cmd_compare}[args.cmd](args, params)


if __name__ == "__main__":
    main()
