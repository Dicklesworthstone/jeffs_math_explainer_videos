#!/usr/bin/env python3
"""Rehearsal of root's render-master01.py in throwaway repositories. Nothing here touches the real lane.

Real in each case: the bytes of render-master01.py (one line changes, the OLD path), the old stage09 worker it edits,
root's guard wrapper and guard source (so a private lock, lane and baseline), and the real 13 chapters, 80 cues and
script. Stand-ins: the binary (a small script), sbh, every prior job's receipt, tables, narration and QA.

Usage, from the repository root:  python3 -I -B checks/rehearse_render_master.py SCRATCH RESULTS
SCRATCH must not exist. RESULTS must not exist. No build, render, audio, real job or real manifest is involved.
"""
import ast
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import wave
from pathlib import Path

HERE = Path(__file__).resolve()
REAL = HERE.parents[6]
P_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
ROOT_REL = "narration_lab/third-wave-20261024/root"
S_REL = f"{ROOT_REL}/elevenlabs-logarithm-rebuild-20261104/final-picture-repair01"
B_REL = f"{ROOT_REL}/five-film-release-20261027"
GUARD_REL = f"{P_REL}/engine-secondary-validation01/guard_owner56g_target20g.py"
SOURCE_REL = ("narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/"
              "run_bounded_phase_oliver_owner48g_target14g_20261102.py")
DRIVER = "render-master01.py"
TIMELINE, SCORE = "logarithm-final-picture-timeline01", "logarithm-final-picture-score01"
BUILD, ASSEMBLE = "logarithm-final-picture-build01", "logarithm-final-picture-assemble01"
MASTER = "logarithm-final-picture-master01"
FRAMES = 6  # a stand-in chapter's length: 6 frames at 60 fps, 4,800 samples
UNUSED_ROW = "target/geometry-kit01/debug/logarithm_mysteries.d"  # a real row of build-inputs01.json


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def old_path():
    match = re.search(r"^OLD = Path\('([^']+)'\)$", (REAL / S_REL / DRIVER).read_text(), re.M)
    return match.group(1)


def config_of(path):
    tree = ast.parse(Path(path).read_text())
    node = next(n for n in tree.body if isinstance(n, ast.Assign)
                and any(isinstance(t, ast.Name) and t.id == "CONFIG" for t in n.targets))
    return json.loads(ast.literal_eval(node.value.args[0]))


REAL_CONFIG = config_of(old_path())
CHAPTERS, CUES = REAL_CONFIG["chapters"], REAL_CONFIG["cues"]

STUB = r'''
import json, os, sys, wave
from pathlib import Path
case = Path(__file__).resolve().parent
hooks = json.loads((case / "hooks.json").read_text())
argv = sys.argv[1:]
def flag(name):
    return argv[argv.index(name) + 1]
def fire(name):
    for action in hooks.get(name, []):
        target = Path(action["path"])
        if action["do"] == "append":
            with target.open("ab") as handle:
                handle.write(b"moved\n")
        elif action["do"] == "exit":
            raise SystemExit(action["code"])
if argv[0] == "chapters":
    print("\n".join(hooks["chapters"]))
elif argv[0] == "render":
    chapter, work = argv[1], Path(flag("--out"))
    lane = Path(flag("--score")).parents[2]
    tables = lane / hooks["timeline"] / "timeline-output"
    work.mkdir(exist_ok=True)
    fire("before:" + chapter)
    for ext in ("timeline.tsv", "cues.tsv"):
        (work / f"{chapter}.{ext}").write_bytes((tables / f"{chapter}.{ext}").read_bytes())
    (work / f"{chapter}.cuelog.tsv").write_text("# log\n")
    (work / f"{chapter}.mp4").write_bytes(b"picture " + chapter.encode())
    if chapter not in hooks.get("no_layout", []):
        (work / f"{chapter}.layout.tsv").write_text("")
    if chapter in hooks.get("changed_cues", []):
        with (work / f"{chapter}.cues.tsv").open("a") as handle:
            handle.write("extra\t0.000\t0.100\n")
    if chapter in hooks.get("changed_timeline", []):
        with (work / f"{chapter}.timeline.tsv").open("a") as handle:
            handle.write("mark\tpredict\t3\n")
    folder = work / "soundtrack" / chapter
    folder.mkdir(parents=True)
    samples = hooks["frames"] * 800 - (100 if chapter in hooks.get("short_track", []) else 0)
    with wave.open(str(folder / "soundtrack_0.wav"), "wb") as track:
        track.setnchannels(2); track.setsampwidth(2); track.setframerate(48000)
        track.writeframes(b"\0" * (samples * 4))
    fire("after:" + chapter)
elif argv[0] == "master":
    work = Path(flag("--out"))
    fire("master")
    (work / "master").mkdir()
    (work / "logarithm_mysteries_master.mp4").write_bytes(b"master")
elif argv[0] == "captions":
    work = Path(flag("--out"))
    fire("captions")
    (work / "logarithm_mysteries.en.srt").write_text("1\n")
    (work / "logarithm_mysteries.en.vtt").write_text("WEBVTT\n")
else:
    raise SystemExit("unknown stub command")
'''


class Case:
    def __init__(self, base, name, hooks=None, sbh_exit=0):
        self.dir = base / name
        self.repo = self.dir / "repo"
        self.s = self.repo / S_REL
        self.b = self.repo / B_REL
        self.hooks = dict(hooks or {})
        self.build()
        self.set_sbh(sbh_exit)

    def set_sbh(self, code):
        folder = self.dir / "bin"
        folder.mkdir(exist_ok=True)
        tool = folder / "sbh"
        tool.write_text(f"#!/bin/bash\necho '{{}}'\nexit {code}\n")
        tool.chmod(tool.stat().st_mode | stat.S_IEXEC)

    def write_json(self, path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(data, indent=2))

    def done(self, job):
        self.write_json(self.b / job / "resource-receipt.json",
                        {"admitted": True, "exit": 0, "finished": "t", "inputs_sha256": "a", "final_inputs_sha256": "a"})

    def build(self):
        s, b, repo = self.s, self.b, self.repo
        s.mkdir(parents=True)
        b.mkdir(parents=True)
        (repo / "target").mkdir()
        for rel in (GUARD_REL, SOURCE_REL):
            (repo / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REAL / rel, repo / rel)
        old = self.dir / "old-worker.py"
        old.write_text(Path(old_path()).read_text().replace(str(REAL), str(repo)))
        driver = (REAL / S_REL / DRIVER).read_text()
        assert driver.count(old_path()) == 1
        (s / DRIVER).write_text(driver.replace(old_path(), str(old)))
        crate = s / "workspace/explainer_corpus/logarithm_mysteries"
        crate.mkdir(parents=True)
        shutil.copyfile(REAL / crate.relative_to(repo) / "script.tsv", crate / "script.tsv")
        narration = s / "narration01"
        narration.mkdir()
        for cue in CUES:
            (narration / f"{cue}.wav").write_bytes(b"cue " + cue.encode())
        (narration / "report.json").write_text("{}\n")
        tables = b / TIMELINE / "timeline-output"
        tables.mkdir(parents=True)
        summary = []
        for chapter in CHAPTERS:
            mine = [c for c in CUES if c.startswith(chapter.split("_")[0] + "_")]
            rows = [f"chapter\t{chapter}", "fps\t60", f"frames\t{FRAMES}"]
            rows += [f"line\t{c}\t0\t0\t{narration / (c + '.wav')}" for c in mine]
            (tables / f"{chapter}.timeline.tsv").write_text("\n".join(rows) + "\n")
            (tables / f"{chapter}.cues.tsv").write_text("".join(f"{c}\t0.000\t0.100\n" for c in mine))
            summary.append({"chapter": chapter, "cues": mine})
        self.write_json(b / TIMELINE / "timeline-summary.json", {"chapters": summary})
        qa = b / SCORE / "scored/stems/qa"
        for chapter in CHAPTERS:
            self.write_json(qa / f"{chapter}.json", {"chapter": chapter, "passed": True,
                                                     "metrics": [{"name": "length", "value": 1, "pass": True}]})
        for job in (BUILD, ASSEMBLE, TIMELINE, SCORE):
            self.done(job)
        unused = repo / UNUSED_ROW
        unused.parent.mkdir(parents=True)
        unused.write_text("dependency list of a debug build no render reads\n")
        frozen = s / "frozen-source.rs"
        frozen.write_text("// source\n")
        self.write_json(s / "build-inputs01.json", {"files": [{"path": str(p), "sha256": sha(p)} for p in (unused, frozen)]})
        self.write_json(s / "assemble-inputs01.json", {"files": [{"path": str(frozen), "sha256": sha(frozen)}]})
        stub = self.dir / "stub.py"
        stub.write_text(STUB)
        binary = s / "logarithm_mysteries-release01"
        binary.write_text(f'#!/bin/bash\nexec "{sys.executable}" -I -B "{stub}" "$@"\n')
        binary.chmod(binary.stat().st_mode | stat.S_IEXEC)
        self.write_hooks()

    def write_hooks(self):
        data = dict(self.hooks, chapters=CHAPTERS, timeline=TIMELINE, frames=FRAMES)
        (self.dir / "hooks.json").write_text(json.dumps(data))

    def run(self, *args):
        env = dict(os.environ, PATH=f"{self.dir / 'bin'}:{os.environ['PATH']}")
        done = subprocess.run([sys.executable, "-I", "-B", str(self.s / DRIVER), *args], cwd=self.repo, env=env,
                              capture_output=True, text=True, timeout=600)
        return done

    def receipt(self, job):
        path = self.b / job / "resource-receipt.json"
        return json.loads(path.read_text()) if path.is_file() else None

    def rendered(self):
        return [c for c in CHAPTERS if (self.b / f"logarithm-final-picture-render-{c}-01/render-summary.json").is_file()]

    def job(self, chapter):
        return f"logarithm-final-picture-render-{chapter}-01"

    def log(self, job):
        path = self.b / job / "command.log"
        return path.read_text() if path.is_file() else ""


def last_line(text):
    lines = [x for x in text.strip().splitlines() if x.strip()]
    return lines[-1].replace("\\", "/") if lines else ""


results = []


def record(name, ok, detail):
    results.append({"name": name, "pass": bool(ok), "detail": detail})
    print(("PASS " if ok else "FAIL ") + name + ": " + str(detail)[:160], flush=True)


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    base, out = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    if base.exists() or out.exists():
        raise SystemExit("SCRATCH and RESULTS must not exist")
    base.mkdir(parents=True)
    os.chdir(REAL)

    # ---- launch contract
    c = Case(base, "template")
    done = c.run("--check-template")
    record("template: the real driver's edits apply to the real old worker", done.returncode == 0, last_line(done.stdout))

    c = Case(base, "chain")
    done = c.run()
    receipts = {c.job(ch): c.receipt(c.job(ch)) for ch in CHAPTERS}
    receipts[MASTER] = c.receipt(MASTER)
    ok = all(r and r["exit"] == 0 and r["admitted"] is True and not r.get("error")
             and r["inputs_sha256"] == r["final_inputs_sha256"] for r in receipts.values())
    record("chain: 13 renders and the master pass the real guard and the edited worker", done.returncode == 0 and ok and len(c.rendered()) == 13,
           {"exit": done.returncode, "jobs": len(receipts)})
    sample = c.receipt(c.job(CHAPTERS[0]))
    record("chain: the guard's command equals the worker's sys.executable check", sample["command"][0] == sys.executable, "same interpreter path")
    configs = sorted(p.name for p in (c.s / "render-jobs01").glob("*.config.json"))
    record("chain: one script, config and manifest per job", len(configs) == 14 and len(list((c.s / "render-jobs01").glob("*-inputs.json"))) == 14, len(configs))
    summary = json.loads((c.b / MASTER / "master-summary.json").read_text())
    record("chain: the master summary names master, srt, vtt and 13 soundtracks", sorted(summary) == ["master", "scope", "soundtracks", "srt", "vtt"] and len(summary["soundtracks"]) == 13, sorted(summary))
    manifest = json.loads((c.s / "render-jobs01" / f"{c.job(CHAPTERS[0])}-inputs.json").read_text())["files"]
    record("chain: a render manifest carries the build's rows", any(r["path"].endswith(UNUSED_ROW) for r in manifest), UNUSED_ROW)
    master_rows = json.loads((c.s / "render-jobs01" / f"{MASTER}-inputs.json").read_text())["files"]
    work = str(c.b / "logarithm-final-picture-render-work01")
    record("chain: the master manifest freezes the render work folder", any(r["path"].startswith(work) for r in master_rows),
           sum(r["path"].startswith(work) for r in master_rows))

    # ---- root's added gates are live
    for name, hooks, want in (("changed cue table", {"changed_cues": [CHAPTERS[3]]}, "full cue table differs"),
                              ("changed timeline row", {"changed_timeline": [CHAPTERS[3]]}, "differs from the 60 fps timeline"),
                              ("missing lint", {"no_layout": [CHAPTERS[3]]}, "rendered lint is missing"),
                              ("short soundtrack", {"short_track": [CHAPTERS[3]]}, "not the whole soundtrack")):
        c = Case(base, "gate-" + name.replace(" ", "-"), hooks)
        done = c.run()
        record(f"control: {name} stops the chain at that chapter",
               done.returncode != 0 and want in c.log(c.job(CHAPTERS[3])) and len(c.rendered()) == 3 and c.receipt(MASTER) is None, want)

    # ---- findings
    c = Case(base, "ignored-evidence")
    c.write_json(c.s / "final-picture-binding01.json", {"build": {"binary_sha256": "0" * 64}})
    c.write_json(c.s / "score-comparison01.json", {"hard_failures": ["stand-in hard failure"]})
    done = c.run()
    stems = [p for p in (c.b / SCORE / "scored/stems").iterdir() if p.is_file()]
    record("finding 2: the chain runs with a failed comparison on disk and with no stem files at all",
           done.returncode == 0 and c.receipt(MASTER)["exit"] == 0 and not stems, {"stem_files": len(stems), "comparison": "hard_failures present"})
    config = json.loads((c.s / "render-jobs01" / f"{c.job(CHAPTERS[0])}.config.json").read_text())
    record("finding 4: the job's binary hash ignores the binding's", config["binary_sha256"] != "0" * 64 and done.returncode == 0,
           "config takes the file's current hash")

    c = Case(base, "non-admission", sbh_exit=1)
    first = c.run()
    r = c.receipt(c.job(CHAPTERS[0]))
    c.set_sbh(0)
    second = c.run()
    record("finding 1: one non-admission fails the name; a rerun with a healthy host still refuses",
           first.returncode != 0 and r["admitted"] is False and second.returncode != 0 and "AssertionError" in second.stderr and c.receipt(MASTER) is None and c.rendered() == [],
           {"first": last_line(first.stderr)[:70], "second": last_line(second.stderr)[:70]})

    c = Case(base, "render-failure", {"before:" + CHAPTERS[2]: [{"do": "exit", "code": 1}]})
    first = c.run()
    finished = c.rendered()
    second = c.run()
    record("finding 1: a failure at chapter 02 leaves 00 and 01 done; a rerun refuses at chapter 00",
           first.returncode != 0 and finished == CHAPTERS[:2] and second.returncode != 0 and "AssertionError" in second.stderr and CHAPTERS[0] in second.stderr,
           {"finished": len(finished)})

    c = Case(base, "partial-launch")
    rows = json.loads((c.s / "build-inputs01.json").read_text())
    (c.repo / UNUSED_ROW).write_text("rebuilt\n")
    first = c.run()
    folder = c.s / "render-jobs01"
    left = sorted(p.name for p in folder.iterdir())
    sizes = {p.name: p.stat().st_size for p in folder.iterdir()}
    (c.repo / UNUSED_ROW).write_text("dependency list of a debug build no render reads\n")
    second = c.run()
    record("finding 1: a refused freeze leaves script, config and an empty manifest; the restored rerun still refuses",
           first.returncode != 0 and len(left) == 3 and sizes[f"{c.job(CHAPTERS[0])}-inputs.json"] < 40 and second.returncode != 0 and not (c.b / c.job(CHAPTERS[0])).exists(),
           {"left": len(left), "rows": len(rows["files"])})

    c = Case(base, "unused-row-moves")
    c.hooks = {"after:" + CHAPTERS[5]: [{"do": "append", "path": str(c.repo / UNUSED_ROW)}]}
    c.write_hooks()
    done = c.run()
    r = c.receipt(c.job(CHAPTERS[5]))
    work = c.b / "logarithm-final-picture-render-work01"
    record("finding 3: a build row no render reads moves during chapter 05: the finished chapter's job fails and the chain stops",
           done.returncode != 0 and r["exit"] == 0 and "reviewed input changed" in r.get("error", "") and (work / f"{CHAPTERS[5]}.mp4").is_file()
           and c.receipt(c.job(CHAPTERS[6])) is None, {"worker_exit": r["exit"], "error": r.get("error", "")[:40]})

    c = Case(base, "picture-swap")
    work = c.b / "logarithm-final-picture-render-work01"
    c.hooks = {"after:" + CHAPTERS[12]: [{"do": "append", "path": str(work / f"{CHAPTERS[0]}.mp4")}]}
    c.write_hooks()
    done = c.run()
    recorded = json.loads((c.b / c.job(CHAPTERS[0]) / "render-summary.json").read_text())["mp4_sha256"]
    frozen = {r["path"]: r["sha256"] for r in json.loads((c.s / "render-jobs01" / f"{MASTER}-inputs.json").read_text())["files"]}
    record("finding 5: a chapter movie changed after its render passes the master's freeze and worker",
           done.returncode == 0 and frozen[str(work / f"{CHAPTERS[0]}.mp4")] != recorded and c.receipt(MASTER)["exit"] == 0, "the manifest froze the changed bytes")

    c = Case(base, "caption-refusal", {"captions": [{"do": "exit", "code": 1}]})
    work = c.b / "logarithm-final-picture-render-work01"
    done = c.run()
    record("finding 6: a caption refusal fails the job after the master is written, with no summary",
           done.returncode != 0 and (work / "logarithm_mysteries_master.mp4").is_file() and not (c.b / MASTER / "master-summary.json").exists(), "master kept, job failed")

    counts = {"tests": len(results), "passed": sum(r["pass"] for r in results), "failed": sum(not r["pass"] for r in results)}
    data = {"driver_sha256": sha(REAL / S_REL / DRIVER), "old_worker_sha256": sha(old_path()), "guard_sha256": sha(REAL / GUARD_REL),
            "guard_source_sha256": sha(REAL / SOURCE_REL), "counts": counts, "results": results,
            "scope": "Throwaway repositories with a stand-in binary. No real job, build, render, audio or lane was involved."}
    with out.open("x") as handle:
        handle.write(json.dumps(data, indent=2) + "\n")
    print(counts)
    return 0 if not counts["failed"] else 1


if __name__ == "__main__":
    sys.exit(main())
