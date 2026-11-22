"""Measure a fixed and adaptive quadtree against the same exact fluid solution."""

import csv
import io
import json
import math
from pathlib import Path
import re
import statistics
import subprocess
import sys


def run_case(binary, case, out):
    root = out / case["id"]
    root.mkdir()
    args = ["/usr/bin/time", "-l", str(binary), case["mesh"], str(case["dt_cap"]),
            str(case["pressure_budget"])]
    print(f"Running {case['id']}", flush=True)
    with (root / "solver.log").open("x") as data, (root / "time.log").open("x") as timing:
        process = subprocess.run(args, cwd=root, stdout=data, stderr=timing, timeout=1200)
    if process.returncode:
        raise RuntimeError(f"{case['id']} exited {process.returncode}")
    content = (root / "solver.log").read_text()
    lines = content.splitlines()
    report, = (json.loads(line) for line in lines if line.startswith("{"))
    series = list(csv.DictReader(io.StringIO("\n".join(line for line in lines if "," in line
                                                      and not line.startswith(("{", "#"))))))
    times = [float(row["time"]) for row in series]
    if len(times) != 17 or any(not math.isclose(t, .025 * i, abs_tol=1e-10)
                               for i, t in enumerate(times)):
        raise ValueError(f"Observation clock for {case['id']} is incomplete")
    if not math.isclose(max(float(row["relative_l2"]) for row in series), report["peak_rel_l2"],
                        rel_tol=1e-12):
        raise ValueError(f"Error summary differs from recorded samples for {case['id']}")
    raw_timing = (root / "time.log").read_text()
    report["process_rss_bytes"] = int(re.search(r"^\s*(\d+)\s+maximum resident set size",
                                                 raw_timing, re.MULTILINE).group(1))
    report["process_wall_seconds"] = float(re.search(r"^\s*([\d.]+)\s+real", raw_timing,
                                                      re.MULTILINE).group(1))
    report["configuration"] = case
    report["observations"] = len(times)
    print(json.dumps(report), flush=True)
    return report


def main():
    binary, plan_file, out = (Path(p).resolve() for p in sys.argv[1:])
    plan = json.loads(plan_file.read_text())
    target = plan["target"]
    reports = {}
    for case in plan["cases"]:
        report = run_case(binary, case, out)
        report["within_error_target"] = (
            report["peak_rel_l2"] <= target["max_sampled_area_weighted_velocity_relative_l2"]
            and report["peak_volume_defect"] <= 1.01 * case["pressure_budget"]
            and report["peak_scaled_pressure_residual"] <= 1.01 * case["pressure_budget"]
            and report["peak_cfl"] <= .600001
        )
        reports[case["id"]] = report
    fine = [reports[f"fine_{suffix}"] for suffix in ("a", "b")]
    adaptive = [reports[f"adaptive_{suffix}"] for suffix in ("a", "b")]
    for group in (fine, adaptive):
        for key in ("steps", "leaf_steps", "cells", "regrids", "peak_rel_l2", "peak_volume_defect"):
            if group[0][key] != group[1][key]:
                raise ValueError(f"Repeatability check failed for {key}")
    ratio = lambda key: statistics.median(row[key] for row in adaptive) / statistics.median(
        row[key] for row in fine)
    comparison = {
        "target": target,
        "runs": reports,
        "both_meet_the_accuracy_target": all(row["within_error_target"] for row in fine + adaptive),
        "adaptive_to_fixed_leaf_step_ratio": ratio("leaf_steps"),
        "adaptive_to_fixed_allocated_step_ratio": ratio("allocated_cell_steps"),
        "adaptive_to_fixed_solver_elapsed_ratio": ratio("solver_seconds"),
        "adaptive_to_fixed_process_wall_ratio": ratio("process_wall_seconds"),
        "adaptive_to_fixed_process_rss_ratio": ratio("process_rss_bytes"),
        "halving_dt_peak_error_delta": reports["adaptive_timestep"]["peak_rel_l2"]
                                       - adaptive[0]["peak_rel_l2"],
        "pressure_100x_tighter_peak_error_delta": reports["adaptive_pressure"]["peak_rel_l2"]
                                                 - adaptive[0]["peak_rel_l2"],
        "scope": "Periodic exact shear case. Measured timing on this host with this instrumentation."
    }
    with (out / "comparison.json").open("x") as handle:
        json.dump(comparison, handle, indent=2)
        handle.write("\n")
    print(json.dumps({key: value for key, value in comparison.items() if key != "runs"}))


if __name__ == "__main__":
    main()
