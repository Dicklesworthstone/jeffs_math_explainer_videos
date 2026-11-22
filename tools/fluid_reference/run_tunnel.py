"""Run declared tunnel experiments serially and retain each result and timing."""

import argparse
import csv
import json
import math
from pathlib import Path
import subprocess


def rows(path):
    with path.open(newline="") as handle:
        result = [{key: float(value) for key, value in row.items()}
                  for row in csv.DictReader(handle)]
    if any(not math.isfinite(value) for row in result for value in row.values()):
        raise ValueError(f"Nonfinite diagnostic in {path}")
    return result


def run_case(binary, output, settings, defaults, budget):
    path = output / settings["name"]
    path.mkdir()
    eta = settings.get("eta", defaults["eta"])
    interval = settings.get("regrid_steps", defaults["regrid_steps"])
    tol = settings.get("budget", budget)
    arguments = [str(binary), settings["flow"], settings["mode"], settings["level"],
                 settings["duration"], settings["dt"], tol, eta, interval, settings["sample_dt"],
                 path]
    if "capture_after" in settings:
        arguments += [settings["capture_after"], settings["capture_every"]]
    command = ["/usr/bin/time", "-l", *map(str, arguments)]
    with (path / "settings.json").open("x") as file:
        json.dump({"case": settings, "command": command}, file, indent=2)
        file.write("\n")
    print(f"Running {settings['name']}", flush=True)
    with (path / "solver.log").open("x") as summary, (path / "timing.log").open("x") as log:
        process = subprocess.run(command, cwd=path, stdout=summary, stderr=log, timeout=7200)
    if process.returncode:
        raise RuntimeError(f"{settings['name']} exited {process.returncode}: {path / 'timing.log'}")
    payload, = [line for line in (path / "solver.log").read_text().splitlines() if line.startswith("{")]
    result = json.loads(payload)
    with (path / "summary.json").open("x") as output:
        output.write(payload + "\n")
    times = rows(path / "series.csv")
    transfers = rows(path / "transfers.csv")
    if (len(times) < 2 or not math.isclose(times[-1]["t"], settings["duration"], abs_tol=1e-10)
            or any(a["t"] >= b["t"] for a, b in zip(times, times[1:]))
            or any(a["i"] >= b["i"] for a, b in zip(times, times[1:]))
            or int(times[-1]["i"]) != result["steps"] or len(transfers) != result["regrids"]):
        raise ValueError(f"Incomplete clock or event log for {path}")
    checks = {
        "pressure_solved_to_budget": result["max_scaled_pressure_residual"] <= tol * 1.01,
        "weighted_flux_defect_within_budget": result["max_weighted_volume_defect"] <= tol * 1.01,
        "face_cfl_within_budget": result["max_observed_face_cfl"] <= .600001,
        "no_solver_caps": result["failed_solve_caps"] == 0,
    }
    if settings["mode"] == "adaptive":
        checks["actual_tree_changes"] = len(transfers) > 0
        checks["correction_decreases_largest_transfer_defect"] = (
            bool(transfers)
            and max(row["raw_div_corrected"] for row in transfers)
            < max(row["raw_div_after"] for row in transfers)
        )
    retained = {"settings": settings, "summary": result, "checks": checks,
                "observations": len(times), "final_observation": times[-1],
                "snapshots": len(list(path.glob("frame-*.flow")))}
    print(json.dumps(retained), flush=True)
    return retained


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("plan", type=Path)
    parser.add_argument("group")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    plan = json.loads(args.plan.read_text())
    results = []
    for settings in plan[args.group]:
        result = run_case(args.binary.resolve(), args.output.resolve(), settings, plan["defaults"],
                          plan["projection_budget"])
        if settings["flow"] == "channel":
            limits = plan["channel_acceptance"]
            end = result["final_observation"]
            pressure = 8 * 0.001 * 0.3 * (1.8 - 0.4) / 0.41**2
            result["channel"] = {"expected_pressure_drop": pressure, "relative_pressure_error":
                                 abs(end["pressure_difference"] - pressure) / pressure}
            result["checks"]["analytical_profile"] = (
                end["relative_profile_l2"] <= limits["final_relative_profile_l2_max"]
            )
            result["checks"]["analytical_pressure_drop"] = (
                result["channel"]["relative_pressure_error"] <= limits["interior_pressure_drop_relative_error_max"]
            )
        results.append(result)
        with (args.output / f"{settings['name']}-verification.json").open("x") as file:
            json.dump(result, file, indent=2)
            file.write("\n")
        if not all(result["checks"].values()):
            raise ValueError(f"{settings['name']} failed an explicit commissioning check: {result['checks']}")
    with (args.output / "verification.json").open("x") as file:
        json.dump(results, file, indent=2)
        file.write("\n")


if __name__ == "__main__":
    main()
