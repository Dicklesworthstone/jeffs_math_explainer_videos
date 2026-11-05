#!/usr/bin/env python3
"""Read-only facts from the stage09 run and root's current manifests, for REVIEW.md.

Reads small tables, receipts, logs and file sizes. It hashes nothing large and runs no job. Live disk readings are left
out so that two runs compare byte for byte; REVIEW.md gives those as dated readings.

Usage, from the repository root:  python3 -I -B checks/stage09_evidence.py RESULTS   (RESULTS must not exist)
"""
import ast
import datetime
import filecmp
import json
import re
import sys
from pathlib import Path

REAL = Path(__file__).resolve().parents[6]
ROOT_REL = "narration_lab/third-wave-20261024/root"
S = REAL / ROOT_REL / "elevenlabs-logarithm-rebuild-20261104/final-picture-repair01"
L = REAL / ROOT_REL / "five-film-release-20261027"
OLD = Path(re.search(r"^OLD = Path\('([^']+)'\)$", (S / "render-master01.py").read_text(), re.M).group(1))
JOBS = OLD.parent
W = JOBS.parents[1]
TIMELINE = L / "logarithm-stage09-timeline02/timeline-output"
WORK = L / "logarithm-stage09-render-work02"
GIB = 1024 ** 3


def config_of(path):
    tree = ast.parse(path.read_text())
    node = next(n for n in tree.body if isinstance(n, ast.Assign)
                and any(isinstance(t, ast.Name) and t.id == "CONFIG" for t in n.targets))
    return json.loads(ast.literal_eval(node.value.args[0]))


def rows(path):
    return [line.split("\t") for line in path.read_text().splitlines() if line and not line.startswith("#")]


def receipt(job):
    return json.loads((L / job / "resource-receipt.json").read_text())


def stamp(job, key):
    return datetime.datetime.fromisoformat(receipt(job)[key])


def seconds(job):
    return (stamp(job, "finished") - stamp(job, "started")).total_seconds()


def growth(job):
    r = receipt(job)
    before, samples = r["admission"], r["samples"]
    work = max(s["work_bytes"] for s in samples) - before["work_bytes"]
    target = max(s["target_bytes"] for s in samples) - before["target_bytes"]
    return round((max(0, work) + max(0, target)) / GIB, 2)


def main():
    if len(sys.argv) != 2 or Path(sys.argv[1]).exists():
        raise SystemExit(__doc__)
    master = config_of(OLD)
    chapters = master["chapters"]
    tables = {}
    for chapter in chapters:
        tables[chapter] = {
            "timeline_equal": rows(TIMELINE / f"{chapter}.timeline.tsv") == rows(WORK / f"{chapter}.timeline.tsv"),
            "cues_equal": rows(TIMELINE / f"{chapter}.cues.tsv") == rows(WORK / f"{chapter}.cues.tsv"),
            "layout_file": (WORK / f"{chapter}.layout.tsv").is_file(),
            "cuelog_rows": [len(rows(TIMELINE / f"{chapter}.cuelog.tsv")), len(rows(WORK / f"{chapter}.cuelog.tsv"))]}
    qa = sorted((L / "logarithm-stage09-score02/scored/stems/qa").glob("*.json"))
    first_qa = json.loads(qa[0].read_text())
    old_render = config_of(JOBS / f"logarithm-stage09-render-{chapters[-1]}-02.py")
    flags = [a for a in old_render["argv"][1:] if not a.startswith("/")]
    log = (L / "logarithm-stage09-master02/command.log").read_text().splitlines()
    contract = next(json.loads(x) for x in log if x.startswith("{") and '"consumer": "master"' in x)
    caption_line = next(x for x in log if x.startswith("captions: "))
    srt = (WORK / "logarithm_mysteries.en.srt").read_text()
    summary = json.loads((L / "logarithm-stage09-timeline02/timeline-summary.json").read_text())
    frames = {x["chapter"]: x["frames"] for x in summary["chapters"]}
    render_jobs = [f"logarithm-stage09-render-{c}-02" for c in chapters]
    inputs = json.loads((S / "build-inputs01.json").read_text())["files"]
    target_rows = [r["path"] for r in inputs if Path(r["path"]).is_relative_to(REAL / "target")]
    engine_rows = [r["path"] for r in inputs if Path(r["path"]).is_relative_to(W / "engine")]
    size = sum(Path(r["path"]).stat().st_size for r in inputs)
    old_rows = json.loads((JOBS / f"logarithm-stage09-render-{chapters[4]}-02-inputs.json").read_text())["files"]
    master_rows = json.loads((JOBS / "logarithm-stage09-master02-inputs.json").read_text())["files"]
    kit_here, kit_there = REAL / "crates/explainer_kit/src", W / "repo-view/crates/explainer_kit/src"
    kit = {p.name: filecmp.cmp(p, kit_there / p.name, shallow=False) for p in sorted(kit_here.glob("*.rs"))}
    data = {
        "chapters": len(chapters), "tables": tables,
        "tables_all_equal": all(t["timeline_equal"] and t["cues_equal"] for t in tables.values()),
        "layout_all_present": all(t["layout_file"] for t in tables.values()),
        "qa": {"files": len(qa), "keys": sorted(first_qa), "null_pass_metrics": sum(m["pass"] is None for m in first_qa["metrics"])},
        "render_flags": flags,
        "frames": {"film": summary["total_frames"], "longest": max(frames.values()), "chapter_04": frames["04_undoing_growth"], "chapter_12": frames["12_what_it_forgets"]},
        "master_log": {"prediction_holds": len(contract["prediction_holds"]), "verified_cues": contract["verified_cues"], "captions": caption_line.split(" -> ")[0]},
        "srt_cues": len(re.findall(r" --> ", srt)),
        "growth_gib": {"timeline": growth("logarithm-stage09-timeline02"), "score": growth("logarithm-stage09-score02"),
                       "largest_render": max(growth(j) for j in render_jobs), "master": growth("logarithm-stage09-master02"),
                       "fresh_debug_build": growth("logarithm-stage09-ws-build02")},
        "renders_lane_seconds": round(sum(seconds(j) for j in render_jobs)),
        "chain_wall_hours": round((stamp("logarithm-stage09-master02", "finished") - stamp(render_jobs[0], "started")).total_seconds() / 3600, 1),
        "build_inputs": {"rows": len(inputs), "megabytes": round(size / 1e6), "under_target": len(target_rows),
                         "under_engine": len(engine_rows), "target_folder": sorted({"/".join(Path(p).relative_to(REAL).parts[:3]) for p in target_rows})},
        "old_render_manifest_rows": len(old_rows),
        "old_master_manifest_work_rows": sum("/logarithm-stage09-render-work02/" in r["path"] for r in master_rows),
        "kit_files_differing": sorted(name for name, same in kit.items() if not same),
        "kit_files_identical": sum(kit.values()),
    }
    with open(sys.argv[1], "x") as handle:
        handle.write(json.dumps(data, indent=2) + "\n")
    print(data["tables_all_equal"], data["renders_lane_seconds"], data["chain_wall_hours"])


if __name__ == "__main__":
    main()
