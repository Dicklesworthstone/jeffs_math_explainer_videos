#!/usr/bin/env python3
"""The worker's output checks on stage09's real files. Read-only; it hashes nothing large and runs no job.

The rehearsal's stand-ins can only show that the worker agrees with text written to fit it. This shows that the
parsers accept what the real kit, x264 and ffprobe wrote: stage09's master log, captions, one chapter and the master.
It proves the parsers only. The new film's bytes do not exist yet.

Usage, from the repository root:  python3.14 -I -B checks/stage09_parsers.py SCRATCH RESULTS
SCRATCH and RESULTS must not exist. SCRATCH holds a copy of the worker and a small config.
"""
import copy
import hashlib
import importlib.util
import json
import os
import re
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
REAL = HERE.parents[6]
ROOT_REL = "narration_lab/third-wave-20261024/root"
L = REAL / ROOT_REL / "five-film-release-20261027"
S = REAL / ROOT_REL / "elevenlabs-logarithm-rebuild-20261104/final-picture-repair01"
NARRATION = REAL / ROOT_REL / "elevenlabs-logarithm-rebuild-20261104/production-audio-stage09-01/prepared02/narration"
OLD = Path(re.search(r"^OLD = Path\('([^']+)'\)$", (S / "render-master01.py").read_text(), re.M).group(1))
WORKSPACE = OLD.parents[1]
WORK = L / "logarithm-stage09-render-work02"
TOPIC = "logarithm_mysteries"


def refuses(call):
    try:
        call()
    except SystemExit as stop:
        return stop.code == 1
    return False


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    scratch, out = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    if scratch.exists() or out.exists():
        raise SystemExit("SCRATCH and RESULTS must not exist")
    scratch.mkdir(parents=True)
    summary = json.loads((L / "logarithm-stage09-timeline02/timeline-summary.json").read_text())
    chapters = [x["chapter"] for x in summary["chapters"]]
    frames = {x["chapter"]: x["frames"] for x in summary["chapters"]}
    bound = json.loads((NARRATION / "report.json").read_text())["meta"]["direction_contract"]["binding"]["contract"]["map"]
    holds = [r for r in json.loads((WORKSPACE / bound["path"]).read_text())["visual_holds"] if r.get("video") == TOPIC and r.get("kind") == "prediction"]
    tools = {name: {"path": str(Path(shutil.which(name)).resolve()), "sha256": "unused"} for name in ("ffprobe",)}
    config = {"lane": str(scratch), "job": "stage09", "tools": tools, "topic": TOPIC, "cues": [c for x in summary["chapters"] for c in x["cues"]],
              "chapters": chapters, "frames": frames, "prediction_cues": [[r["cue"], r["next_cue"]] for r in holds], "sentences": 364}
    worker = scratch / "worker.py"
    shutil.copyfile(BUNDLE / "job_worker.py", worker)
    worker.with_suffix(".config.json").write_text(json.dumps(config))
    spec = importlib.util.spec_from_file_location("worker_copy", worker)
    W = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(W)
    env = dict(os.environ)
    lines = (L / "logarithm-stage09-master02/command.log").read_text().splitlines()
    master, srt, vtt = WORK / f"{TOPIC}_master.mp4", WORK / f"{TOPIC}.en.srt", WORK / f"{TOPIC}.en.vtt"
    plans = W.contracts(lines)
    found = W.holds_of(plans["master"])
    replays = W.replays_of(lines)
    report = W.mastered(lines, master)
    captions = W.captioned(lines, srt)
    timing = W.caption_files(srt.read_text(), vtt.read_text(), captions["captions"], found)
    chapter_media = W.movie_facts(WORK / f"{chapters[0]}.mp4", frames[chapters[0]], env)
    master_media = W.movie_facts(master, sum(frames.values()), env)
    fewer = copy.deepcopy(plans["master"])
    fewer["prediction_holds"].pop()
    no_replay = [x for x in lines if not x.startswith(chapters[5])]
    other_plan = [x.replace('"promotion":false', '"promotion":true').replace('"promotion": false', '"promotion": true') for x in lines]
    other_end = copy.deepcopy(found)
    other_end[3]["end_ms"] += 1
    controls = {"a lost hold refuses": refuses(lambda: W.holds_of(fewer)),
                "a lost replay line refuses": refuses(lambda: W.replays_of(no_replay)),
                "a promoted plan refuses": refuses(lambda: W.contracts(other_plan)),
                "a hold that left the captions refuses": refuses(lambda: W.caption_files(srt.read_text(), vtt.read_text(), captions["captions"], other_end)),
                "the wrong movie length refuses": refuses(lambda: W.movie_facts(master, sum(frames.values()) + 1, env))}
    data = {"worker_sha256": hashlib.sha256((BUNDLE / "job_worker.py").read_bytes()).hexdigest(), "python": sys.version.split()[0],
            "accepted": {"plans": sorted(plans), "prediction_holds": len(found), "replays": len(replays), "report": report, "captions": captions,
                         "timing": timing, "chapter": chapters[0], "chapter_media": chapter_media, "master_media": master_media},
            "negative_controls": controls,
            "scope": "Stage09's real files through the worker's parsers. These are not the repaired film's files; nothing was rendered, heard or viewed."}
    with out.open("x") as handle:
        handle.write(json.dumps(data, indent=2) + "\n")
    print(len(found), len(replays), captions, chapter_media["frames"], master_media["frames"], controls)
    return 0 if all(controls.values()) else 1


if __name__ == "__main__":
    sys.exit(main())
