#!/usr/bin/env python3
"""Guarded 4K render and master of the repaired logarithm film on root's remote lane (bead 87do.63).

LavenderPike, 2026-11-05, for BlueSalmon. Replaces S/render-master01.py on mac-mini-max; that file stays as it is. It
fixes D1, D2, D4, D5 and D6 of logarithm-final-render-master-review01/REVIEW.md, and D3 by binding only what a render
reads. D7 stays: captions still follow the master, and a named master retry recovers a caption failure.

Run on mac-mini-max from the repository root, under the python3 in O/tools01.json:

    python3 -I -B render_master_remote.py check --seal H [options]    reads only; exit 0 ready, 1 refused, 3 not ready
    python3 -I -B render_master_remote.py run --seal H [options]      13 renders, then the master, one job at a time

    --seal H                       sha256 of this folder's SHA256SUMS
    --approved-schedule-sha256 H   needed when schedule-comparison01.json has rows in not_as_predicted
    --approved-score-sha256 H      needed when a stem changed in a chapter whose schedule did not change
    --retry NAME=rN                retry a failed attempt: NAME is a chapter or `master`, rN the next attempt, r2 to r9
    --wait-seconds S               run only: wait this long for unfinished prerequisites (default 0)

A chapter whose job passed and whose hashes all hold is skipped. A failed attempt is never retried by itself: `run`
stops at it, and only --retry goes on, after moving that attempt's outputs aside. Nothing is deleted.

Technical gates only: nothing here was heard, viewed or accepted.
"""
import argparse
import ast
import datetime
import fcntl
import hashlib
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import time
from pathlib import Path

HOST = "mac-mini-max"
ADMISSION_GIB = 10  # O/guard01.py asserts this value; checking first keeps a refused job from using up its name
HERE = Path(__file__).resolve()
BUNDLE = HERE.parent
WORKER = BUNDLE / "job_worker.py"
R = HERE.parents[5]
EL = R / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
S = EL / "final-picture-repair01"
O = EL / "remote-mac-mini-max01"
JOBS, TARGET = O / "jobs", O / "target"
SCRIPTS, PRIOR_SCRIPTS = O / "render-jobs01", O / "job-scripts"
GUARD, PINS, LANE_LOCK = O / "guard01.py", O / "guard-pins01.json", O / "expanded-native-lane.lock"
BIN = O / "logarithm_mysteries-release01"
BINDING, TOOLS = O / "remote-binding01.json", O / "tools01.json"
SCHEDULE, SCORED = O / "schedule-comparison01.json", O / "score-comparison01.json"
BUILD_INPUTS = O / "build-inputs01.json"
WORKSPACE = S / "workspace"
CRATE = WORKSPACE / "explainer_corpus/logarithm_mysteries"
KIT_TOOLS = [WORKSPACE / "tools" / name for name in ("coherent_narration.py", "narration_cache.py", "narration_delivery.py")]
BOUND_FILES = [S / name for name in ("integration.json", "holds-bound01.json", "spans01.json")]

BUILD, ASSEMBLE = "logarithm-remote-build01", "logarithm-remote-assemble01"
TIMELINE, SCORE = "logarithm-remote-timeline01", "logarithm-remote-score01"
MASTER = "logarithm-remote-master01"
# The rebind job is whichever one the assembly's manifest names: see rebind_job().
REBIND_RECEIPT = re.compile(re.escape(str(JOBS)) + r"/(logarithm-remote-rebind\d\d)/resource-receipt\.json")
N = JOBS / ASSEMBLE / "narration01"
WORK = JOBS / "logarithm-remote-render-work01"
ASIDE = JOBS / "logarithm-remote-render-aside01"

TOPIC = "logarithm_mysteries"
CHAPTERS = ["00_ratio_ruler", "01_building_ruler", "02_slide_rule", "03_folded_decade", "04_undoing_growth",
            "05_complex_turns", "06_lifting_loops", "07_branch_cut", "08_product_rule", "09_matrix_logs",
            "10_doing_nothing", "11_unreachable", "12_what_it_forgets"]
CUES, PREDICTIONS, SENTENCES = 80, 13, 364
BINDING_SCHEMA, TOOLS_SCHEMA = "logarithm-remote-binding-v1", "logarithm-remote-tools-v1"
TIMELINE_SUCCESS = "TIMELINE PASS: 13 chapters at 60 fps; 80 cue placements in script order."
SCORE_SUCCESS = "SCORE PASS: all 13 chapter score QA reports passed."
RENDER_SUCCESS = "4K60 chapter rendered; full timeline/cue tables match; picture review remains open."
MASTER_SUCCESS = ("MASTER WRITTEN: candidate master and captions; strict verify, pictures, ASR, caption screen, "
                  "loudness and listening remain.")
SCOPE = ("Repaired logarithm with retained ElevenLabs Oliver recordings, on mac-mini-max. Technical evidence only; no "
         "picture, listening, fine-sync or independent AV acceptance.")
CONFIG_LINE = re.compile(r"^CONFIG = json\.loads\((.*)\)$", re.M)
CONFIG_SIDECAR = 'pathlib.Path(__file__).with_suffix(".config.json").read_text()'
SAME_RENDER_KEYS = ("job", "step", "chapter", "lane", "target", "binary", "binary_sha256", "narration", "render_work",
                    "timeline_job", "argv", "tools_sha256", "binding_sha256")
SAME_MASTER_KEYS = SAME_RENDER_KEYS + ("captions_argv", "render_jobs", "soundtracks", "renders")
FAILED_STATES = ("failed", "not-admitted", "no-receipt", "crashed", "unverified")


class Refused(Exception):
    """A definite no. Nothing downstream may run."""


class Pending(Exception):
    """Not finished yet: a prerequisite is absent or running."""


def need(ok, message):
    if not ok:
        raise Refused(message)


def read(path):
    return json.loads(Path(path).read_text())


HASHES = {}


def sha(path):
    """A file's hash, remembered for as long as its size and time stand: resumes read the same inputs many times."""
    path = Path(path)
    stat = path.stat()
    key = (str(path), stat.st_size, stat.st_mtime_ns)
    if key not in HASHES:
        with path.open("rb") as handle:
            HASHES[key] = hashlib.file_digest(handle, "sha256").hexdigest()
    return HASHES[key]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def same(a, b):
    return Path(a).resolve() == Path(b).resolve()


def rows(path):
    return [line.split("\t") for line in Path(path).read_text().splitlines() if line and not line.startswith("#")]


def first(table, key):
    return next(r[1] for r in table if r[0] == key)


def last_line(path):
    lines = [x for x in Path(path).read_text(errors="replace").splitlines() if x.strip()] if Path(path).is_file() else []
    return lines[-1] if lines else None


def stamp(receipt, key):
    return datetime.datetime.fromisoformat(receipt[key])


def tree(folder):
    return [p for p in sorted(Path(folder).rglob("*")) if p.is_file()]


def read_config(script):
    match = CONFIG_LINE.search(Path(script).read_text())
    need(match is not None, f"{script} has no CONFIG line")
    if match[1] == CONFIG_SIDECAR:
        return read(Path(script).with_suffix(".config.json"))
    return json.loads(ast.literal_eval(match[1]))


# ---------------------------------------------------------------- the host and its tools

def check_seal(expected):
    sums = BUNDLE / "SHA256SUMS"
    need(sums.is_file() and sha(sums) == expected, f"this folder's SHA256SUMS is not the seal {expected[:12]}")
    for line in sums.read_text().splitlines():
        want, name = line.split(maxsplit=1)
        need((BUNDLE / name).is_file() and sha(BUNDLE / name) == want, f"{name} differs from its seal")


def tool_path(tools):
    folders = [str(Path(tools[name]["path"]).parent) for name in ("python3", "ffmpeg", "ffprobe")]
    return os.pathsep.join(dict.fromkeys(folders + ["/usr/bin", "/bin"]))


def tool_facts():
    """O/tools01.json, measured on this host. The kit finds all three tools by name, so PATH must find these files."""
    if not TOOLS.is_file():
        raise Pending(f"{TOOLS.name} has not been measured on this host")
    tools = read(TOOLS)
    need(tools.get("schema") == TOOLS_SCHEMA and tools.get("host") == HOST, f"{TOOLS.name} is not a {TOOLS_SCHEMA} for {HOST}")
    path = tool_path(tools)
    for name in ("python3", "ffmpeg", "ffprobe"):
        row = tools[name]
        need(Path(row["path"]).is_absolute() and Path(row["path"]).is_file() and sha(row["path"]) == row["sha256"],
             f"this host's {name} is not {row['path']} {row['sha256'][:12]}")
        found = shutil.which(name, path=path)
        need(found is not None and same(found, row["path"]), f"PATH {path} finds {name} at {found}, not {row['path']}")
    need(same(sys.executable, tools["python3"]["path"]), f"this python3 is {sys.executable}, not {tools['python3']['path']}")
    done = subprocess.run([tools["ffmpeg"]["path"], "-hide_banner", "-encoders"], capture_output=True, text=True, timeout=60,
                          env=dict(os.environ, PATH=path))
    need(done.returncode == 0 and re.search(r"\blibx264\b", done.stdout) is not None, "the pinned ffmpeg has no libx264")
    return tools


def admission():
    """The guard's own admission test, run first: a job the guard refuses keeps its folder, so its name is used up."""
    need(shutil.which("sbh") is not None, "sbh is not on PATH; the guard needs it")
    gate = subprocess.run(["sbh", "check", "--target-free", "0.2", "--need", f"{ADMISSION_GIB}G", "--json"], cwd=R, text=True,
                          capture_output=True, timeout=45)
    need(gate.returncode == 0, f"sbh refuses admission: {gate.stdout[-200:]}{gate.stderr[-200:]}")
    disk = os.statvfs(R)
    need(disk.f_bavail * disk.f_frsize >= ADMISSION_GIB * 1024 ** 3, f"under {ADMISSION_GIB} GiB free; the guard would refuse")


# ---------------------------------------------------------------- job states

def lane_busy():
    if not LANE_LOCK.is_file():
        return False
    with LANE_LOCK.open("r") as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return True
        fcntl.flock(handle, fcntl.LOCK_UN)
    return False


def job_state(job):
    """absent, running, passed, or one of FAILED_STATES except unverified, which comes from reading the outputs."""
    folder = JOBS / job
    if not folder.exists():
        return "absent"
    path = folder / "resource-receipt.json"
    if not path.is_file():
        return "running" if lane_busy() else "no-receipt"  # the guard saves its first receipt after admission
    try:
        r = read(path)
    except json.JSONDecodeError:
        return "running"  # the guard rewrites its live receipt in place
    if not r.get("finished"):
        return "running" if lane_busy() else "crashed"
    if r.get("admitted") is not True:
        return "not-admitted"
    clean = (r.get("exit") == 0 and not r.get("error") and not r.get("stopped_for_resource_limit")
             and r.get("inputs_sha256") and r.get("inputs_sha256") == r.get("final_inputs_sha256"))
    return "passed" if clean else "failed"


def settled(job):
    state = job_state(job)
    if state in ("absent", "running"):
        raise Pending(f"{job} is {state}")
    need(state == "passed", f"{job} is {state}: nothing runs on it")
    return read(JOBS / job / "resource-receipt.json")


def comparison(path):
    try:
        data = read(path)
    except (FileNotFoundError, json.JSONDecodeError):
        raise Pending(f"{path.name} is not written yet") from None
    need(data.get("hard_failures") == [], f"{path.name} records hard failures: {data.get('hard_failures')}")
    return data


def rebind_job(binding, assemble_receipt):
    """The rebind that fed this assembly, with no job or manifest name assumed. The binding holds the manifest whose
    hash the assembly's guard froze, and that manifest holds the one rebind receipt the assembly read. A failed earlier
    attempt can stay on the lane: no assembly names it. The rule is the one in Bronze's verifier."""
    manifests = [p for p, h in binding["files"].items() if h == assemble_receipt.get("inputs_sha256")]
    need(len(manifests) == 1, f"the binding holds {len(manifests)} files with the hash {ASSEMBLE}'s guard froze, not one")
    manifest = Path(manifests[0])
    frozen_rows = read(manifest)["files"]
    need(any(row["path"] == str(BIN) and row["sha256"] == sha(BIN) for row in frozen_rows), f"{manifest.name} froze another binary than {BIN.name}")
    named = [(m[1], row["sha256"]) for row in frozen_rows for m in [REBIND_RECEIPT.fullmatch(row["path"])] if m]
    need(len(named) == 1, f"{manifest.name} names {len(named)} rebind jobs, not one")
    job, frozen = named[0]
    path = JOBS / job / "resource-receipt.json"
    need(path.is_file() and sha(path) == frozen, f"{job}: its receipt is not the one the assembly froze")
    r = settled(job)
    command = r.get("command")
    need(r.get("phase") == "native" and isinstance(command, list) and len(command) >= 2
         and Path(command[-2]).name == "rebind_audio.py" and command[-1] == "--apply", f"{job} did not run rebind_audio.py --apply: {command}")
    return job, manifest


def prior_config(job, line):
    """The timeline or score really ran this binary on this narration under this binding, under its own guard."""
    script, inputs = PRIOR_SCRIPTS / f"{job}.py", PRIOR_SCRIPTS / f"{job}-inputs.json"
    r = read(JOBS / job / "resource-receipt.json")
    need(r.get("phase") == "native" and r.get("command") == ["python3", "-I", "-B", str(script)], f"{job}: the guard ran {r.get('command')}")
    need(inputs.is_file() and sha(inputs) == r["inputs_sha256"], f"{job}: {inputs.name} is not the manifest its guard froze")
    need(last_line(JOBS / job / "command.log") == line, f"{job}: the log does not end with its success line")
    cfg = read_config(script)
    for key, want in (("job", job), ("lane", str(JOBS)), ("target", str(TARGET)), ("binary", str(BIN)),
                      ("binary_sha256", sha(BIN)), ("narration", str(N))):
        need(cfg.get(key) == want, f"{job}: its {key} is {cfg.get(key)}, not {want}")
    frozen = {row["path"]: row["sha256"] for row in read(inputs)["files"]}
    for path in (BIN, BINDING):
        need(frozen.get(str(path)) == sha(path), f"{job}: its guard did not freeze this {path.name}")
    return cfg


def timeline_facts(cues):
    summary = read(JOBS / TIMELINE / "timeline-summary.json")
    need([x["chapter"] for x in summary["chapters"]] == CHAPTERS, "the timeline's chapters differ")
    need([cue for x in summary["chapters"] for cue in x["cues"]] == cues, "the timeline's cues are not the script's")
    frames = {x["chapter"]: x["frames"] for x in summary["chapters"]}
    need(sum(frames.values()) == summary["total_frames"], "the timeline's total is not the sum of its chapters")
    output = JOBS / TIMELINE / "timeline-output"
    for item in summary["chapters"]:
        chapter, path = item["chapter"], output / f"{item['chapter']}.timeline.tsv"
        need(path.is_file() and sha(path) == item["timeline_sha256"], f"{chapter}: the timeline table moved")
        need((output / f"{chapter}.cues.tsv").is_file(), f"{chapter}: the timeline job has no cue table")
        table = rows(path)
        need(first(table, "chapter") == chapter and first(table, "fps") == "60" and int(first(table, "frames")) == item["frames"],
             f"{chapter}: the table is not this chapter at 60 fps and {item['frames']} frames")
        for row in table:
            if row[0] == "line":
                need(same(row[4], N / f"{row[1]}.wav"), f"{row[1]}: the timeline plays {row[4]}, outside the bound narration")
    return frames


def score_facts(frames):
    stems = JOBS / SCORE / "scored/stems"
    qa_folder = stems / "qa"
    need({p.stem for p in qa_folder.glob("*.json")} == set(CHAPTERS), "the score's QA reports are not the 13 chapters")
    for chapter, count in frames.items():
        need((stems / f"{chapter}.music.f32.wav").is_file() or (stems / f"{chapter}.music.f32.parts.tsv").is_file(),
             f"{chapter}: no score stem")
        qa = read(qa_folder / f"{chapter}.json")
        failed = [m["name"] for m in qa.get("metrics", []) if m.get("pass") not in (True, None)]
        need(qa.get("chapter") == chapter and qa.get("passed") is True and qa.get("metrics") and not failed, f"{chapter}: QA failed {failed}")
        lengths = [m["value"] for m in qa["metrics"] if m["name"] == "length"]
        need(lengths == [count * 800], f"{chapter}: QA measured length {lengths}, the chapter needs {count * 800}")


def approved(path, flag, value, surprises):
    """Reported deltas stop the chain until root names the exact comparison it read. Nothing passes by being ignored."""
    if surprises or value:
        need(value == sha(path), f"{path.name} reports {surprises}: root must read it and pass {flag} {sha(path)}")


def gather(a):
    """Every gate before a job, cheapest first. Pending means wait, Refused means stop. Writes nothing."""
    tools = tool_facts()
    build_receipt = settled(BUILD)
    assemble_receipt = settled(ASSEMBLE)
    if not BINDING.is_file():
        raise Pending(f"{BINDING.name} is not written yet")
    timeline_receipt = settled(TIMELINE)
    schedule = comparison(SCHEDULE)
    score_receipt = settled(SCORE)
    scored = comparison(SCORED)
    b = read(BINDING)
    binary = sha(BIN)
    need(b.get("schema") == BINDING_SCHEMA, f"{BINDING.name} is not a {BINDING_SCHEMA}")
    need(b["build"]["job"] == BUILD and b["build"]["binary"] == str(BIN), f"the binding names the binary {b['build']['binary']}")
    need(b["build"]["binary_sha256"] == binary, f"the binary is {binary[:12]}, the binding says {b['build']['binary_sha256'][:12]}")
    need(b["audio"]["narration"] == str(N), f"the binding names the narration {b['audio']['narration']}")
    need(b["cues"] == CUES and b["chapters"] == len(CHAPTERS), f"the binding has {b['chapters']} chapters and {b['cues']} cues")
    moved = [p for p, h in b["files"].items() if not Path(p).is_file() or sha(p) != h]
    moved += [rel for rel, h in b["crate"].items() if not (CRATE / rel).is_file() or sha(CRATE / rel) != h]
    need(not moved, f"{len(moved)} bound files moved since the binding, first {moved[:2]}")
    rebind, assemble_inputs = rebind_job(b, assemble_receipt)
    prior_config(TIMELINE, TIMELINE_SUCCESS)
    score_config = prior_config(SCORE, SCORE_SUCCESS)
    need(str(JOBS / TIMELINE / "timeline-output") in score_config["argv"], "the score did not read this timeline")
    order = [build_receipt, read(JOBS / rebind / "resource-receipt.json"), assemble_receipt, timeline_receipt, score_receipt]
    need(all(stamp(x, "finished") <= stamp(y, "started") for x, y in zip(order, order[1:])),
         "build, rebind, assembly, timeline and score did not run in that order")
    listed = subprocess.run([str(BIN), "chapters"], capture_output=True, text=True, timeout=60)
    need(listed.returncode == 0 and listed.stdout.splitlines() == CHAPTERS, "the binary's chapter registry differs")
    cues = [line.split("\t", 1)[0] for line in (CRATE / "script.tsv").read_text().splitlines() if line and not line.startswith("#")]
    need(len(cues) == len(set(cues)) == CUES, f"the script has {len(cues)} cues")
    frames = timeline_facts(cues)
    score_facts(frames)
    bound_map = read(N / "report.json")["meta"]["direction_contract"]["binding"]["contract"]["map"]
    map_path = WORKSPACE / bound_map["path"]
    need(map_path.is_file() and sha(map_path) == bound_map["sha256"] == b["direction"]["map_sha256"], "the narration's map is not the bound one")
    holds = [[r["cue"], r["next_cue"]] for r in read(map_path)["visual_holds"] if r.get("video") == TOPIC and r.get("kind") == "prediction"]
    need(len(holds) == PREDICTIONS, f"the bound map has {len(holds)} prediction holds")
    approved(SCHEDULE, "--approved-schedule-sha256", a.approved_schedule_sha256, schedule["not_as_predicted"])
    unexplained = sorted(set(scored["stems_changed"]) - set(schedule["chapters_changed"]))
    approved(SCORED, "--approved-score-sha256", a.approved_score_sha256, unexplained)
    return {"tools": tools, "binding": b, "binary_sha256": binary, "cues": cues, "frames": frames, "prediction": holds,
            "prior_jobs": [BUILD, rebind, ASSEMBLE, TIMELINE, SCORE], "map": map_path, "assemble_inputs": assemble_inputs,
            "comparisons": {"not_as_predicted": schedule["not_as_predicted"], "chapters_changed": schedule["chapters_changed"],
                            "stems_changed": scored["stems_changed"], "film": schedule.get("film")}}


# ---------------------------------------------------------------- jobs

def job_files(job):
    return SCRIPTS / f"{job}.py", SCRIPTS / f"{job}.config.json", SCRIPTS / f"{job}-inputs.json"


def config_for(step, job, f, chapter=None, renders=None):
    script = job_files(job)[0]
    n, work, binary = str(N), str(WORK), str(BIN)
    stems = str(JOBS / SCORE / "scored/stems")
    cfg = {"job": job, "step": step, "script": str(script), "lane": str(JOBS), "target": str(TARGET), "host": HOST,
           "binary": binary, "binary_sha256": f["binary_sha256"], "topic": TOPIC, "narration": n, "chapters": CHAPTERS,
           "cues": f["cues"], "chapter": chapter, "frames": f["frames"], "timeline_job": TIMELINE, "score_job": SCORE,
           "render_work": work, "binding": str(BINDING), "binding_sha256": sha(BINDING), "comparisons": [str(SCHEDULE), str(SCORED)],
           "tools": f["tools"], "tools_sha256": sha(TOOLS), "prediction_cues": f["prediction"], "sentences": SENTENCES,
           "after": list(f["prior_jobs"]), "render_jobs": {}, "soundtracks": {}, "renders": {}, "scope": SCOPE,
           "helper_sha256": sha(HERE), "worker_sha256": sha(WORKER)}
    if step == "render":
        cfg["argv"] = [binary, "render", chapter, "--res", "3840x2160", "--fps", "60", "--narration", n, "--score", stems,
                       "--out", work, "--lint", "--crf", "16", "--preset", "veryfast"]
        cfg["success"] = RENDER_SUCCESS
    else:
        cfg["render_jobs"] = {ch: renders[ch]["job"] for ch in CHAPTERS}
        cfg["after"] += list(cfg["render_jobs"].values())
        cfg["soundtracks"] = {ch: {k: renders[ch]["summary"]["soundtrack"][k] for k in ("path", "sha256")} for ch in CHAPTERS}
        cfg["renders"] = {ch: render_files(ch, renders[ch]["summary"]) for ch in CHAPTERS}
        cfg["argv"] = [binary, "master", "--narration", n, "--score", stems, "--out", work]
        cfg["captions_argv"] = [binary, "captions", "all", "--narration", n, "--out", work]
        cfg["success"] = MASTER_SUCCESS
    return cfg


def render_files(chapter, s):
    return {f"{chapter}.mp4": s["mp4_sha256"], f"{chapter}.timeline.tsv": s["timeline_sha256"],
            f"{chapter}.cues.tsv": s["cues_sha256"], f"{chapter}.layout.tsv": s["layout_sha256"]}


def named_files(value):
    """Every file a JSON value names, as the narration tools open them: relative to the repository, the guard's cwd."""
    if isinstance(value, dict):
        value = list(value.values())
    if isinstance(value, list):
        return [p for item in value for p in named_files(item)]
    if isinstance(value, str) and "/" in value and len(value) < 1024 and (R / value).is_file():
        return [R / value]
    return []


def narration_reads():
    """The retained takes, receipts and hold evidence that coherent verify reopens at every render."""
    assembly = read(N / "assembly.json")
    found = {*named_files(assembly), *named_files(read(N / "report.json"))}
    for receipt in sorted({R / unit["generation_receipt"] for unit in assembly["manifest"]["units"]}):
        need(receipt.is_file(), f"the assembly names a receipt that is not here: {receipt}")
        found.update(named_files(read(receipt)))
    return sorted(found)


def input_rows(f, extra=()):
    """What a render or master reads, and nothing it doesn't: no build tree, no engine source, no target folder."""
    paths = {BIN, BINDING, TOOLS, SCHEDULE, SCORED, GUARD, PINS, BUILD_INPUTS, f["assemble_inputs"], *BOUND_FILES, *KIT_TOOLS, f["map"]}
    paths.update(Path(row["path"]) for row in read(PINS)["files"])
    paths.update(Path(f["tools"][name]["path"]) for name in ("python3", "ffmpeg", "ffprobe"))
    paths.update(CRATE / rel for rel in f["binding"]["crate"])
    for job in (BUILD, ASSEMBLE, TIMELINE, SCORE, f["prior_jobs"][1]):
        paths.update(tree(JOBS / job))
    paths.update(narration_reads())
    paths.update(extra)
    missing = sorted(str(p) for p in paths if not p.is_file())
    need(not missing, f"{len(missing)} inputs are missing, first {missing[:2]}")
    return {str(p): sha(p) for p in paths}


def write_once(path, data):
    """Write a new file whole, or accept the identical file an interrupted launch left. Anything else refuses."""
    if path.exists():
        need(path.read_bytes() == data, f"{path} exists with other bytes: move it aside before launching")
        return
    scratch = path.with_name(path.name + ".writing")
    with scratch.open("xb") as handle:
        handle.write(data)
        handle.flush()
        os.fsync(handle.fileno())
    scratch.rename(path)


def launch(job, cfg, f, extra=()):
    """Everything is checked and hashed first, so a refused freeze leaves no files behind, then one guarded job."""
    script, config, manifest = job_files(job)
    script_bytes = WORKER.read_bytes()
    config_bytes = (json.dumps(cfg, indent=2) + "\n").encode()
    files = input_rows(f, extra)
    files[str(script)], files[str(config)] = digest(script_bytes), digest(config_bytes)
    manifest_bytes = (json.dumps({"files": [{"path": p, "sha256": h} for p, h in sorted(files.items())]}, indent=2) + "\n").encode()
    admission()
    write_once(script, script_bytes)
    write_once(config, config_bytes)
    write_once(manifest, manifest_bytes)
    python = f["tools"]["python3"]["path"]
    print(f"LAUNCH {job}: {len(files)} frozen inputs", flush=True)
    done = subprocess.run([python, "-I", "-B", str(GUARD), "--phase", "native", "--job", job, "--inputs", str(manifest), "--",
                           python, "-I", "-B", str(script)], cwd=R, check=False)
    state = job_state(job)
    need(done.returncode == 0 and state == "passed", f"{job} did not pass: guard exit {done.returncode}, state {state}. "
         f"Read {JOBS / job}/command.log; it is not retried by itself")


def job_record(job, f, want, keys, line):
    """The receipt ran this script, and every file its guard froze still has its frozen bytes."""
    script, _, manifest = job_files(job)
    r = read(JOBS / job / "resource-receipt.json")
    command = r.get("command")
    need(r.get("phase") == "native" and isinstance(command, list) and len(command) == 4 and command[1:] == ["-I", "-B", str(script)]
         and same(command[0], f["tools"]["python3"]["path"]), f"{job}: the guard ran {command}")
    need(manifest.is_file() and sha(manifest) == r["inputs_sha256"], f"{job}: {manifest.name} is not the manifest its guard froze")
    moved = [row["path"] for row in read(manifest)["files"] if not Path(row["path"]).is_file() or sha(row["path"]) != row["sha256"]]
    need(not moved, f"{job}: {len(moved)} frozen inputs moved, first {moved[:2]}")
    cfg = read_config(script)
    for key in keys:
        need(cfg.get(key) == want.get(key), f"{job}: its {key} differs from this film's")
    need(last_line(JOBS / job / "command.log") == line, f"{job}: the log does not end with its success line")


def verify_render(chapter, job, f):
    """D1 and D5: a chapter counts only if its job, its frozen inputs and its files all still hold."""
    job_record(job, f, config_for("render", job, f, chapter), SAME_RENDER_KEYS, RENDER_SUCCESS)
    s = read(JOBS / job / "render-summary.json")
    need(s.get("chapter") == chapter and s.get("binary_sha256") == f["binary_sha256"], f"{job}: the summary is for another chapter or binary")
    for name, recorded in render_files(chapter, s).items():
        need((WORK / name).is_file() and sha(WORK / name) == recorded, f"{name} is not the file {job} recorded")
    track = s["soundtrack"]
    need(Path(track["path"]).is_file() and sha(track["path"]) == track["sha256"], f"{chapter}: the soundtrack capture moved")
    output = JOBS / TIMELINE / "timeline-output"
    for ext in ("timeline.tsv", "cues.tsv"):
        need(rows(output / f"{chapter}.{ext}") == rows(WORK / f"{chapter}.{ext}"), f"{chapter}: the {ext} differs from the timeline job's")
    need(s["media"]["frames"] == f["frames"][chapter], f"{job}: the movie's frames are not the timeline's")
    return s


def verify_master(job, f, renders):
    job_record(job, f, config_for("master", job, f, None, renders), SAME_MASTER_KEYS, MASTER_SUCCESS)
    s = read(JOBS / job / "master-summary.json")
    for key in ("master", "srt", "vtt"):
        need(Path(s[key]["path"]).is_file() and sha(s[key]["path"]) == s[key]["sha256"], f"the {key} is not the file {job} recorded")
    need(len(s["prediction_holds"]) == PREDICTIONS and s["captions"]["sentences"] == SENTENCES, f"{job}: the summary's holds or sentences differ")
    return s


def attempts(base):
    return [base] + [f"{base}-r{n}" for n in range(2, 10)]


def status_of(name, base, retry, verify):
    """One chapter or the master: done, todo, retry or blocked. Only a retry root named goes past a failed attempt."""
    names = attempts(base)
    used = [job for job in names if (JOBS / job).exists()]
    need(used == names[:len(used)], f"{name}: the attempts have a gap: {used}")
    if not used:
        need(retry is None, f"{name} has no attempt to retry")
        return {"state": "todo", "next": base}
    job, state, why = used[-1], job_state(used[-1]), None
    if state == "passed":
        try:
            found = verify(job)
        except (Refused, OSError, KeyError, ValueError) as error:
            state, why = "unverified", f"{type(error).__name__}: {error}"
        else:
            need(retry is None, f"{job} passed and its hashes hold: there is nothing to retry")
            return {"state": "done", "job": job, "summary": found}
    if state == "running":
        need(retry is None, f"{job} is running")
        return {"state": "blocked", "job": job, "why": "running"}
    following = names[len(used)] if len(used) < len(names) else None
    if retry is None:
        return {"state": "blocked", "job": job, "why": why or state, "root_may_pass": f"--retry {name}=r{len(used) + 1}" if following else None}
    need(following is not None and retry == following.rsplit("-", 1)[1], f"{name}: the next attempt is {following}, not {retry}")
    return {"state": "retry", "next": following, "failed": job, "failed_as": why or state, "aside": [str(p) for p in leftovers(name)]}


def leftovers(name):
    if name == "master":
        paths = [WORK / f"{TOPIC}_master.mp4", WORK / f"{TOPIC}.en.srt", WORK / f"{TOPIC}.en.vtt", WORK / "master"]
    else:
        paths = sorted(WORK.glob(f"{name}.*")) + [WORK / "soundtrack" / name]
    return [p for p in paths if p.exists()]


def set_aside(job, paths):
    """A named retry's only move: the failed attempt's outputs, kept whole under ASIDE."""
    if not paths:
        return
    folder = ASIDE / job
    folder.mkdir(parents=True)
    with (folder / "moved.json").open("x") as handle:
        json.dump([{"from": str(p), "sha256": sha(p) if p.is_file() else None} for p in paths], handle, indent=2)
    for path in paths:
        path.rename(folder / path.name)
    print(f"ASIDE {job}: {len(paths)} paths in {folder}", flush=True)


def plan(f, retries):
    chapters = {}
    for chapter in CHAPTERS:
        base = f"logarithm-remote-render-{chapter}-01"
        chapters[chapter] = status_of(chapter, base, retries.get(chapter), lambda job, c=chapter: verify_render(c, job, f))
    if all(s["state"] == "done" for s in chapters.values()):
        renders = {c: {"job": s["job"], "summary": s["summary"]} for c, s in chapters.items()}
        master = status_of("master", MASTER, retries.get("master"), lambda job: verify_master(job, f, renders))
    else:
        need("master" not in retries, "--retry master needs all 13 chapters done first")
        master, renders = {"state": "waiting"}, None
    return chapters, master, renders


def brief(status):
    return {key: value for key, value in status.items() if key != "summary"}


def check(a, retries):
    try:
        f = gather(a)
    except Pending as pending:
        print(json.dumps({"ready": False, "pending": str(pending)}, indent=2))
        return 3
    chapters, master, _ = plan(f, retries)
    blocked = [c for c, s in [*chapters.items(), ("master", master)] if s["state"] == "blocked"]
    print(json.dumps({"ready": not blocked, "blocked": blocked, "rebind_job": f["prior_jobs"][1], "comparisons": f["comparisons"],
                      "frames": sum(f["frames"].values()), "tools": {k: f["tools"][k]["path"] for k in ("python3", "ffmpeg", "ffprobe")},
                      "chapters": {c: brief(s) for c, s in chapters.items()}, "master": brief(master)}, indent=2))
    return 1 if blocked else 0


def ready(a):
    deadline = time.monotonic() + a.wait_seconds
    while True:
        try:
            return gather(a)
        except Pending as pending:
            left = deadline - time.monotonic()
            need(left > 0, f"not ready: {pending}")
            print(f"WAIT {pending}", flush=True)
            time.sleep(min(20, left))


def stop_if_blocked(chapters, master):
    blocked = {c: s for c, s in [*chapters.items(), ("master", master)] if s["state"] == "blocked"}
    need(not blocked, "stopped before launching anything: " + json.dumps({c: brief(s) for c, s in blocked.items()}))


def run(a, retries):
    SCRIPTS.mkdir(exist_ok=True)
    with (SCRIPTS / ".driver.lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise Refused("another render_master_remote.py run holds this lane") from None
        f = ready(a)
        stop_if_blocked(*plan(f, retries)[:2])
        for chapter in CHAPTERS:
            f = gather(a)  # every input again, as of this launch
            base = f"logarithm-remote-render-{chapter}-01"
            status = status_of(chapter, base, retries.get(chapter), lambda job, c=chapter: verify_render(c, job, f))
            if status["state"] == "done":
                print(f"DONE {status['job']}: passed, hashes hold, not launched", flush=True)
                continue
            need(status["state"] in ("todo", "retry"), f"{chapter}: {brief(status)}")
            if status["state"] == "retry":
                set_aside(status["failed"], leftovers(chapter))
                retries.pop(chapter)
            job = status["next"]
            launch(job, config_for("render", job, f, chapter), f)
            verify_render(chapter, job, f)
        f = gather(a)
        chapters, status, renders = plan(f, retries)
        stop_if_blocked(chapters, status)
        need(renders is not None, "not every chapter is done")
        if status["state"] == "done":
            print(f"DONE {status['job']}: passed, hashes hold, not launched", flush=True)
        else:
            if status["state"] == "retry":
                set_aside(status["failed"], leftovers("master"))
            job = status["next"]
            extra = [p for c in CHAPTERS for p in tree(JOBS / renders[c]["job"])] + tree(WORK)
            launch(job, config_for("master", job, f, None, renders), f, extra)
            status = {"job": job, "summary": verify_master(job, f, renders)}
        s = status["summary"]
        print(json.dumps({"master_job": status["job"], "master_sha256": s["master"]["sha256"], "srt_sha256": s["srt"]["sha256"],
                          "vtt_sha256": s["vtt"]["sha256"], "not_claimed": SCOPE}, indent=2), flush=True)
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=("check", "run"))
    parser.add_argument("--seal", required=True)
    parser.add_argument("--approved-schedule-sha256")
    parser.add_argument("--approved-score-sha256")
    parser.add_argument("--retry", action="append", default=[])
    parser.add_argument("--wait-seconds", type=int, default=0)
    a = parser.parse_args(argv)
    try:
        need(socket.gethostname().split(".")[0].lower() == HOST, f"this helper runs only on {HOST}")
        need(Path.cwd().resolve() == R, f"run from the repository root, {R}")
        need(a.command == "run" or not a.wait_seconds, "--wait-seconds goes with run")
        retries = {}
        for value in a.retry:
            name, _, attempt = value.partition("=")
            need((name in CHAPTERS or name == "master") and re.fullmatch(r"r[2-9]", attempt) and name not in retries,
                 f"--retry {value}: expected one NAME=rN per chapter or master, r2 to r9")
            retries[name] = attempt
        check_seal(a.seal)
        return {"check": check, "run": run}[a.command](a, retries)
    except Refused as refused:
        print(f"REFUSED: {refused}", file=sys.stderr, flush=True)
        return 1


if __name__ == "__main__":
    sys.exit(main())
