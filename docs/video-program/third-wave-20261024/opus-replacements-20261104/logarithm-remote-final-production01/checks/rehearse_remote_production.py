#!/usr/bin/env python3
"""Rehearsal of render_master_remote.py and job_worker.py in throwaway repositories. Nothing touches any real lane.

Real in every case: both helpers' bytes (see LIMITS for the three literals that change), root's guard01.py, guard wrapper
and guard source (so a private lock, lane and baseline), the real script's 80 cues, and the 13 prediction cues of
stage09's map. Stand-ins: the binary, sbh, ffmpeg, ffprobe, and every prior job's receipt, table, narration and QA.
Fixtures are a few hundred kilobytes each, because root barred large fixture writes on the short local disk.

Usage, from the repository root:  python3.14 -I -B checks/rehearse_remote_production.py SCRATCH RESULTS
SCRATCH and RESULTS must not exist. Use the remote's Python 3.14. No build, render, audio or real job is involved.
"""
import datetime
import fcntl
import hashlib
import importlib.util
import json
import os
import re
import shutil
import socket
import stat
import subprocess
import sys
import threading
import time
import types
from pathlib import Path

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
REAL = HERE.parents[6]
P_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
EL_REL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
O_REL, S_REL = EL_REL + "/remote-mac-mini-max01", EL_REL + "/final-picture-repair01"
BUNDLE_REL = P_REL + "/logarithm-remote-final-production01"
GUARD_REL = P_REL + "/engine-secondary-validation01/guard_owner56g_target20g.py"
SOURCE_REL = ("narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/"
              "run_bounded_phase_oliver_owner48g_target14g_20261102.py")
CRATE_REL = S_REL + "/workspace/explainer_corpus/logarithm_mysteries"
BRONZE = REAL / P_REL / "logarithm-remote-final-verifier02/verify_remote_final.py"
HOSTNAME = socket.gethostname().split(".")[0].lower()
PYTHON = str(Path(sys.executable).resolve())
FRAMES = 6  # a stand-in chapter: 6 frames at 60 fps, 4,800 samples
HOLD_CUES = ["00_4", "01_3", "02_2", "04_6", "05_1", "05_5", "06_2", "06_4", "07_3", "08_1", "08_4", "11_3", "12_5"]
MAP_REL = "docs/narration-direction-map-oliver17.json"
LIMITS = ["The binary, sbh, ffmpeg, ffprobe and all prior jobs are stand-ins, so no real kit, x264, null test, caption or file format ran.",
          "Three literals change in the fixture copies: the host name in the driver and in guard01.py, and the 10 GiB admission floor and 8 GiB stop in the driver and guard, because the local disk is short.",
          "Timings, resource stops at real sizes and the remote host itself were not exercised."]
SEI = (b"\x00x264 - core 165 r3222 b35605a - H.264/MPEG-4 AVC codec - Copyleft 2003-2025 - http://www.videolan.org/x264.html - "
       b"options: cabac=1 ref=1 rc=crf mbtree=0 crf=16.0 qcomp=0.60\x00")

spec = importlib.util.spec_from_file_location("driver_for_names", BUNDLE / "render_master_remote.py")
M = importlib.util.module_from_spec(spec)
spec.loader.exec_module(M)
CHAPTERS = M.CHAPTERS
BUILD, ASSEMBLE, TIMELINE, SCORE, MASTER = M.BUILD, M.ASSEMBLE, M.TIMELINE, M.SCORE, M.MASTER
REBIND_FAILED, REBIND = "logarithm-remote-rebind01", "logarithm-remote-rebind02"
CUES = [x.split("\t", 1)[0] for x in (REAL / CRATE_REL / "script.tsv").read_text().splitlines() if x and not x.startswith("#")]
TOPIC = M.TOPIC

STUB = r'''
import json, shutil, sys, wave
from pathlib import Path
case = Path(__file__).resolve().parent
hooks = json.loads((case / "hooks.json").read_text())
argv = sys.argv[1:]
chapters, frames = hooks["chapters"], hooks["frames"]
SEI = bytes.fromhex(hooks["sei"])
def flag(name):
    return argv[argv.index(name) + 1]
def fire(name):
    for action in hooks.get(name, []):
        if action["do"] == "append":
            with open(action["path"], "ab") as handle:
                handle.write(b"moved\n")
        elif action["do"] == "exit":
            raise SystemExit(action["code"])
def plan(consumer, holds):
    row = {"chapter_scope": None, "consumer": consumer, "directed_caption_segments": [], "listening_acceptance": False,
           "plan_mode": "canonical", "prediction_holds": holds, "promotion": hooks.get("promotion", False),
           "selected_cues": 80, "topic": "logarithm_mysteries", "verified_cues": 80}
    print(json.dumps(row), flush=True)
if argv[0] == "chapters":
    print("\n".join(chapters))
elif argv[0] == "render":
    chapter, work = argv[1], Path(flag("--out"))
    tables = Path(hooks["timeline_output"])
    work.mkdir(exist_ok=True)
    fire("before:" + chapter)
    (case / "path-seen.json").write_text(json.dumps({n: shutil.which(n) for n in ("python3", "ffmpeg", "ffprobe")}))
    for ext in ("timeline.tsv", "cues.tsv"):
        (work / f"{chapter}.{ext}").write_bytes((tables / f"{chapter}.{ext}").read_bytes())
    (work / f"{chapter}.cuelog.tsv").write_text("# log\n")
    sei = {"crf23": SEI.replace(b"crf=16.0", b"crf=23.0"), "none": b"no header"}.get(hooks.get("sei_kind", {}).get(chapter), SEI)
    (work / f"{chapter}.mp4").write_bytes(b"ftyp" + sei + chapter.encode())
    if chapter not in hooks.get("no_layout", []):
        (work / f"{chapter}.layout.tsv").write_text("")
    if chapter in hooks.get("changed_cues", []):
        with (work / f"{chapter}.cues.tsv").open("a") as handle:
            handle.write("extra\t0.000\t0.100\n")
    if chapter in hooks.get("changed_timeline", []):
        with (work / f"{chapter}.timeline.tsv").open("a") as handle:
            handle.write("mark\tpredict\t3\n")
    if chapter not in hooks.get("no_track", []):
        folder = work / "soundtrack" / chapter
        folder.mkdir(parents=True)
        samples = frames * 800 - (100 if chapter in hooks.get("short_track", []) else 0)
        with wave.open(str(folder / "soundtrack_0.wav"), "wb") as track:
            track.setnchannels(2); track.setsampwidth(2); track.setframerate(48000)
            track.writeframes(b"\0" * (samples * 4))
    fire("after:" + chapter)
elif argv[0] == "master":
    work = Path(flag("--out"))
    fire("master")
    total = frames * len(chapters)
    holds = [{"cue": c, "end_ms": 50 * (i + 1), "next_cue": n} for i, (c, n) in enumerate(hooks["holds"])]
    if hooks.get("drop_hold"):
        holds = holds[:-1]
    if hooks.get("wrong_hold"):
        holds[0]["cue"] = "00_1"
    plan("master", holds)
    for chapter in chapters:
        if chapter != hooks.get("no_replay"):
            print(f"{chapter} replay = fmn's soundtrack, bit for bit (7 cues, {frames * 800} samples)", flush=True)
    (work / "master").mkdir()
    path = work / "logarithm_mysteries_master.mp4"
    path.write_bytes(b"ftyp" + SEI + b"master")
    told = total + hooks.get("master_frames_delta", 0)
    print(f"master -> {path}: gain -1.28 dB, -20.12 LUFS, -1.50 dBTP; {told} frames (chapters {told}); audio 1.300 s (picture 1.300 s)", flush=True)
    fire("after-master")
elif argv[0] == "captions":
    work = Path(flag("--out"))
    fire("captions")
    plan("captions", [])
    ends = [50 * (i + 1) for i in range(len(hooks["holds"]))] + [700]
    if hooks.get("lost_stamp"):
        ends[3] += 1
    def stamp(ms, mark):
        return "00:00:%02d%s%03d" % (ms // 1000, mark, ms % 1000)
    def body(mark):
        return "".join(f"{i + 1}\n{stamp(a, mark)} --> {stamp(b, mark)}\nwords\n\n" for i, (a, b) in enumerate(zip([0] + ends, ends)))
    srt = work / "logarithm_mysteries.en.srt"
    srt.write_text(body(","))
    vtt = "WEBVTT\n\n" + body(".")
    if hooks.get("vtt_differs"):
        vtt = vtt.replace("00:00:00.050", "00:00:00.051")
    (work / "logarithm_mysteries.en.vtt").write_text(vtt)
    print(f"captions: {hooks.get('sentences', 364)} sentences as {len(ends)} captions -> {srt}", flush=True)
else:
    raise SystemExit("unknown stub command")
'''

MEDIA = r'''
import json, sys
from pathlib import Path
case = Path(__file__).resolve().parent
hooks = json.loads((case / "hooks.json").read_text())
tool, argv = sys.argv[1], sys.argv[2:]
if tool == "ffmpeg":
    print(" V..... libx264 stand-in" if not hooks.get("no_x264") else " V..... other stand-in")
else:
    path = Path(argv[-1])
    total = hooks["frames"] * len(hooks["chapters"])
    frames = total if path.stem.endswith("_master") else hooks["frames"]
    change = hooks.get("probe", {}).get(path.stem, {})
    video = {"codec_type": "video", "codec_name": "h264", "profile": "High", "width": 3840, "height": 2160, "r_frame_rate": "60/1",
             "pix_fmt": "yuv420p", "color_space": "bt709", "nb_frames": str(frames)}
    audio = {"codec_type": "audio", "codec_name": "aac", "sample_rate": "48000", "channels": 2}
    video.update(change.get("video", {}))
    audio.update(change.get("audio", {}))
    streams = [video] if change.get("drop_audio") else [video, audio]
    print(json.dumps({"streams": streams, "format": {"duration": "0.1"}}))
'''


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def executable(path, text):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(text)
    Path(path).chmod(Path(path).stat().st_mode | stat.S_IEXEC)


def when(minutes):
    return (datetime.datetime(2026, 1, 1, tzinfo=datetime.timezone.utc) + datetime.timedelta(minutes=minutes)).isoformat()


class Case:
    def __init__(self, base, name, hooks=None, driver_host=HOSTNAME):
        self.dir = base / name
        self.repo = self.dir / "repo"
        self.o, self.s = self.repo / O_REL, self.repo / S_REL
        self.jobs, self.scripts, self.prior = self.o / "jobs", self.o / "render-jobs01", self.o / "job-scripts"
        self.crate = self.repo / CRATE_REL
        self.n = self.jobs / ASSEMBLE / "narration01"
        self.work = self.jobs / "logarithm-remote-render-work01"
        self.aside = self.jobs / "logarithm-remote-render-aside01"
        self.bin = self.o / "logarithm_mysteries-release01"
        self.binding_path, self.tools_path = self.o / "remote-binding01.json", self.o / "tools01.json"
        self.schedule, self.scored = self.o / "schedule-comparison01.json", self.o / "score-comparison01.json"
        self.assemble_inputs = self.o / "assemble-inputs01.json"
        self.hooks = dict(hooks or {})
        self.driver_host = driver_host
        self.clock = 0
        self.build()

    def tick(self):
        self.clock += 1
        return when(self.clock)

    def write_json(self, path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(data, indent=2))

    def edit_json(self, path, change):
        data = json.loads(path.read_text())
        change(data)
        self.write_json(path, data)

    def sbh(self, exits=""):
        (self.dir / "sbh-exits").write_text(exits + "\n")

    def receipt(self, job, command, inputs, exit_code=0):
        started, finished = self.tick(), self.tick()
        self.write_json(self.jobs / job / "resource-receipt.json",
                        {"started": started, "phase": "native", "command": command, "admitted": True, "inputs_sha256": inputs,
                         "final_inputs_sha256": inputs, "exit": exit_code, "finished": finished, "samples": []})

    def prior_job(self, job, step, **changes):
        script, inputs = self.prior / f"{job}.py", self.prior / f"{job}-inputs.json"
        cfg = {"job": job, "step": step, "script": str(script), "lane": str(self.jobs), "target": str(self.o / "target"),
               "binary": str(self.bin), "binary_sha256": sha(self.bin), "narration": str(self.n),
               "argv": [str(self.bin), step, "--narration", str(self.n), "--timelines", str(self.jobs / TIMELINE / "timeline-output")]}
        cfg.update(changes)
        script.parent.mkdir(exist_ok=True)
        script.write_text("CONFIG = json.loads(%r)\n" % json.dumps(cfg, indent=1))
        self.write_json(inputs, {"files": [{"path": str(p), "sha256": sha(p)} for p in (script, self.bin, self.binding_path)]})

    def finish_prior(self, job, line):
        script, inputs = self.prior / f"{job}.py", self.prior / f"{job}-inputs.json"
        self.receipt(job, ["python3", "-I", "-B", str(script)], sha(inputs))
        (self.jobs / job / "command.log").write_text("RUN stand-in\n" + line + "\n")

    def rewrite_prior(self, job, step, line, **changes):
        keep = json.loads((self.jobs / job / "resource-receipt.json").read_text())
        self.prior_job(job, step, **changes)
        keep["inputs_sha256"] = keep["final_inputs_sha256"] = sha(self.prior / f"{job}-inputs.json")
        self.write_json(self.jobs / job / "resource-receipt.json", keep)

    def build(self):
        o, s, crate, n, repo = self.o, self.s, self.crate, self.n, self.repo
        self.dir.mkdir()
        for folder in (s, crate / "docs", crate / "src/chapters", s / "workspace/tools", self.jobs, o / "target"):
            folder.mkdir(parents=True, exist_ok=True)
        # the helpers, with this host's name and no disk floor
        bundle = repo / BUNDLE_REL
        bundle.mkdir(parents=True)
        driver = (BUNDLE / "render_master_remote.py").read_text()
        for old, new in (('HOST = "mac-mini-max"', f'HOST = "{self.driver_host}"'), ("ADMISSION_GIB = 10 ", "ADMISSION_GIB = 0 ")):
            assert driver.count(old) == 1, old
            driver = driver.replace(old, new)
        (bundle / "render_master_remote.py").write_text(driver)
        shutil.copyfile(BUNDLE / "job_worker.py", bundle / "job_worker.py")
        (bundle / "SHA256SUMS").write_text("".join(f"{sha(bundle / x)}  {x}\n" for x in ("job_worker.py", "render_master_remote.py")))
        self.seal = sha(bundle / "SHA256SUMS")
        # root's guard, its pins and its source
        for rel in (GUARD_REL, SOURCE_REL):
            (repo / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REAL / rel, repo / rel)
        guard = (REAL / O_REL / "guard01.py").read_text()
        assert guard.count("'mac-mini-max'") == 1 and guard.count("    return guard.main()\n") == 1
        guard = guard.replace("'mac-mini-max'", repr(HOSTNAME))
        guard = guard.replace("    return guard.main()\n", "    guard.ADMISSION_GIB['native'] = 0\n    guard.STOP_FREE_GIB = 0\n    return guard.main()\n")
        (o / "guard01.py").write_text(guard)
        self.pins = o / "guard-pins01.json"
        self.write_json(self.pins, {"files": [{"path": str(repo / rel), "sha256": sha(repo / rel)} for rel in (GUARD_REL, SOURCE_REL)]})
        # stand-in tools
        executable(self.dir / "bin/sbh", f'#!/bin/bash\necho \'{{}}\'\nread -r first rest < "{self.dir}/sbh-exits"\n'
                                          f'echo "$rest" > "{self.dir}/sbh-exits"\nexit "${{first:-0}}"\n')
        self.sbh()
        (self.dir / "stub.py").write_text(STUB)
        (self.dir / "media.py").write_text(MEDIA)
        executable(self.bin, f'#!/bin/bash\nexec "{PYTHON}" -I -B "{self.dir / "stub.py"}" "$@"\n')
        self.ffmpeg, self.ffprobe = self.dir / "media/ffmpeg", self.dir / "media/ffprobe"
        for tool in (self.ffmpeg, self.ffprobe):
            executable(tool, f'#!/bin/bash\nexec "{PYTHON}" -I -B "{self.dir / "media.py"}" {tool.name} "$@"\n')
        self.write_tools()
        self.write_hooks()
        # the source, map and rebind
        shutil.copyfile(REAL / CRATE_REL / "script.tsv", crate / "script.tsv")
        (crate / "narration_units.json").write_text("{}\n")
        (crate / "src/chapters/mod.rs").write_text("// stand-in\n")
        (crate / "src/chapters/draw.rs").write_text("// stand-in\n")
        following = dict(zip(CUES, CUES[1:]))
        holds = [{"video": TOPIC, "cue": c, "next_cue": following[c], "kind": "prediction"} for c in HOLD_CUES]
        holds += [{"video": TOPIC, "cue": "03_1", "next_cue": "03_2", "kind": "reading"}, {"video": "other", "cue": "00_1", "next_cue": "00_2", "kind": "prediction"}]
        self.write_json(crate / MAP_REL, {"visual_holds": holds})
        self.hooks["holds"] = [[c, following[c]] for c in HOLD_CUES]
        for name in ("coherent_narration.py", "narration_cache.py", "narration_delivery.py"):
            (s / "workspace/tools" / name).write_text("# stand-in\n")
        for name in ("integration.json", "holds-bound01.json", "spans01.json"):
            (s / name).write_text("{}\n")
        (o / "build-inputs01.json").write_text('{"files": []}\n')
        self.receipt(BUILD, ["/bin/bash", "build01.sh"], sha(o / "build-inputs01.json"))
        (self.jobs / BUILD / "chapters.txt").write_text("\n".join(CHAPTERS) + "\n")
        rebind = ["python3", "-I", "-B", str(repo / P_REL / "logarithm-final-audio-rebind01/rebind_audio.py"), "--apply"]
        self.receipt(REBIND_FAILED, rebind, "a", exit_code=1)
        self.receipt(REBIND, rebind, "b")
        # the assembly: narration, retained takes and their receipt
        n.mkdir(parents=True)
        for cue in CUES:
            (n / f"{cue}.wav").write_bytes(b"cue " + cue.encode())
        takes = repo / EL_REL / "production-audio-stage09-01/decoded02"
        (takes / "decoded").mkdir(parents=True)
        for cue in CUES:
            (takes / "decoded" / f"{cue}.wav").write_bytes(b"take " + cue.encode())
            (takes / f"{cue}.source").write_bytes(b"source")
        self.write_json(takes / "decode-receipt.json", {"records": [{"cue": c, "source": str((takes / f"{c}.source").relative_to(repo))} for c in CUES]})
        (self.takes) = takes
        self.write_json(n / "assembly.json", {"manifest": {"units": [
            {"wav": str((takes / "decoded" / f"{c}.wav").relative_to(repo)), "generation_receipt": str((takes / "decode-receipt.json").relative_to(repo))} for c in CUES]}})
        map_rel = "explainer_corpus/logarithm_mysteries/" + MAP_REL
        self.write_json(n / "report.json", {"meta": {"direction_contract": {"binding": {"contract": {"map": {"path": map_rel, "sha256": sha(crate / MAP_REL)}}}}}})
        (n / "phrase-timings.json").write_text("{}\n")
        self.write_json(self.jobs / ASSEMBLE / "audio-comparison.json", {"cues": 80})
        rebind_receipt = self.jobs / REBIND / "resource-receipt.json"
        self.write_json(self.assemble_inputs, {"files": [{"path": str(rebind_receipt), "sha256": sha(rebind_receipt)}]})
        self.receipt(ASSEMBLE, ["python3", "assemble"], sha(self.assemble_inputs))
        self.write_binding()
        # timeline and score
        self.prior_job(TIMELINE, "timeline")
        output = self.jobs / TIMELINE / "timeline-output"
        output.mkdir(parents=True)
        chapters = []
        for chapter in CHAPTERS:
            mine = [c for c in CUES if c.split("_")[0] == chapter.split("_")[0]]
            table = [f"chapter\t{chapter}", "fps\t60", f"frames\t{FRAMES}"] + [f"line\t{c}\t0\t0\t{n / (c + '.wav')}" for c in mine]
            (output / f"{chapter}.timeline.tsv").write_text("\n".join(table) + "\n")
            (output / f"{chapter}.cues.tsv").write_text("".join(f"{c}\t0.000\t0.050\n" for c in mine))
            (output / f"{chapter}.cuelog.tsv").write_text("# log\n")
            chapters.append({"chapter": chapter, "frames": FRAMES, "cues": mine, "timeline_sha256": sha(output / f"{chapter}.timeline.tsv")})
        self.write_json(self.jobs / TIMELINE / "timeline-summary.json", {"fps": 60, "chapters": chapters, "cues": 80, "total_frames": FRAMES * len(CHAPTERS)})
        self.finish_prior(TIMELINE, M.TIMELINE_SUCCESS)
        self.write_json(self.schedule, {"not_as_predicted": [], "hard_failures": [], "chapters_changed": [CHAPTERS[4], CHAPTERS[12]], "film": {"delta_frames": 444}})
        self.prior_job(SCORE, "score")
        stems = self.jobs / SCORE / "scored/stems"
        for chapter in CHAPTERS:
            (stems / "qa").mkdir(parents=True, exist_ok=True)
            (stems / f"{chapter}.music.f32.wav").write_bytes(b"stem")
            self.write_json(stems / "qa" / f"{chapter}.json", {"chapter": chapter, "passed": True, "metrics": [
                {"name": "length", "value": FRAMES * 800, "pass": True}, {"name": "tempo", "value": 1, "pass": None}]})
        self.finish_prior(SCORE, M.SCORE_SUCCESS)
        self.write_json(self.scored, {"hard_failures": [], "stems_changed": [CHAPTERS[4], CHAPTERS[12]]})
        self.write_hooks()

    def write_tools(self, **changes):
        row = lambda path: {"path": str(Path(path).resolve()), "sha256": sha(path), "version": "stand-in"}
        data = {"schema": M.TOOLS_SCHEMA, "host": HOSTNAME, "python3": dict(row(PYTHON), pillow="12.1.1"),
                "ffmpeg": row(self.ffmpeg), "ffprobe": row(self.ffprobe)}
        data.update(changes)
        self.write_json(self.tools_path, data)

    def write_binding(self):
        n = self.n
        crate = {str(p.relative_to(self.crate)): sha(p) for p in sorted(self.crate.rglob("*")) if p.is_file()}
        files = {str(p): sha(p) for p in sorted(n.rglob("*")) if p.is_file()}
        for path in (self.assemble_inputs, self.o / "build-inputs01.json"):
            files[str(path)] = sha(path)
        self.write_json(self.binding_path, {
            "schema": M.BINDING_SCHEMA, "build": {"job": BUILD, "binary": str(self.bin), "binary_sha256": sha(self.bin)},
            "audio": {"narration": str(n)}, "cues": 80, "chapters": 13, "crate": crate, "files": files,
            "direction": {"map_sha256": sha(self.crate / MAP_REL), "holds": 15}, "approved": {"integration_sha256": sha(self.s / "integration.json")}})

    def write_hooks(self):
        data = dict(self.hooks, chapters=CHAPTERS, frames=FRAMES, sei=SEI.hex(), timeline_output=str(self.jobs / TIMELINE / "timeline-output"))
        (self.dir / "hooks.json").write_text(json.dumps(data))

    def run(self, command, *args, seal=None):
        env = dict(os.environ, PATH=f"{self.dir / 'bin'}:/usr/bin:/bin")
        return subprocess.run([sys.executable, "-I", "-B", str(self.repo / BUNDLE_REL / "render_master_remote.py"), command,
                               "--seal", seal or self.seal, *args], cwd=self.repo, env=env, capture_output=True, text=True, timeout=300)

    def job(self, chapter, attempt=""):
        return f"logarithm-remote-render-{chapter}-01{attempt}"

    def state(self, job):
        path = self.jobs / job / "resource-receipt.json"
        return json.loads(path.read_text()) if path.is_file() else None

    def log(self, job):
        path = self.jobs / job / "command.log"
        return path.read_text() if path.is_file() else ""

    def rendered(self):
        return [c for c in CHAPTERS if (self.jobs / self.job(c) / "render-summary.json").is_file()]

    def job_folders(self):
        return sorted(p.name for p in self.jobs.iterdir() if p.is_dir())

    def snapshot(self):
        return {str(p): (p.stat().st_size, p.stat().st_mtime_ns) for p in sorted(self.repo.rglob("*")) if p.is_file()}


results = []


def record(name, ok, detail=""):
    results.append({"name": name, "pass": bool(ok), "detail": detail})
    print(("PASS " if ok else "FAIL ") + name + ": " + str(detail)[:150], flush=True)


def said(done, phrase):
    return done.returncode != 0 and phrase in done.stdout + done.stderr


def contract_tests(base):
    c = Case(base, "chain")
    before = c.snapshot()
    done = c.run("check")
    report = json.loads(done.stdout) if done.returncode == 0 else {}
    record("check: ready, 13 chapters to do, the rebind read from the assembly", done.returncode == 0 and report["ready"] is True and
           all(s["state"] == "todo" for s in report["chapters"].values()) and report["rebind_job"] == REBIND, report.get("rebind_job"))
    record("check: changes no file", c.snapshot() == before, len(before))
    done = c.run("run")
    jobs = [c.job(ch) for ch in CHAPTERS] + [MASTER]
    receipts = {job: c.state(job) for job in jobs}
    ok = all(r and r["exit"] == 0 and r["admitted"] is True and not r.get("error") and r["inputs_sha256"] == r["final_inputs_sha256"] for r in receipts.values())
    record("chain: 13 renders and the master pass root's real guard", done.returncode == 0 and ok, {"exit": done.returncode, "jobs": len(jobs)})
    record("chain: every job ends with Bronze's agreed success line", all(c.log(j).strip().splitlines()[-1] == M.RENDER_SUCCESS for j in jobs[:-1])
           and c.log(MASTER).strip().splitlines()[-1] == M.MASTER_SUCCESS, "14 logs")
    worker = (BUNDLE / "job_worker.py").read_bytes()
    record("chain: each job script is the worker byte for byte", all((c.scripts / f"{j}.py").read_bytes() == worker for j in jobs), 14)
    line = re.search(r"^CONFIG = json\.loads\((.*)\)$", worker.decode(), re.M)
    record("chain: the config line is the agreed sidecar form", line[1] == 'pathlib.Path(__file__).with_suffix(".config.json").read_text()', "exact")
    record("chain: the guard ran the pinned python3", all(Path(r["command"][0]).resolve() == Path(PYTHON) and Path(r["command"][0]).name.startswith("python3") for r in receipts.values()), "resolved")
    seen = json.loads((c.dir / "path-seen.json").read_text())
    record("chain: the kit's PATH finds exactly the pinned tools", same(seen["ffmpeg"], c.ffmpeg) and same(seen["ffprobe"], c.ffprobe) and same(seen["python3"], PYTHON), "3 tools")
    rows = {j: json.loads((c.scripts / f"{j}-inputs.json").read_text())["files"] for j in jobs}
    paths = {j: {r["path"] for r in v} for j, v in rows.items()}
    record("D3: no manifest holds a target or engine row", not any("/target/" in p or "/engine/" in p for v in paths.values() for p in v), "14 manifests")
    record("rebind: the manifests hold the job the assembly named, not the failed one",
           all(str(c.jobs / REBIND / "resource-receipt.json") in v and not any(f"/{REBIND_FAILED}/" in p for p in v) for v in paths.values()), REBIND)
    wants = [c.scripts / f"{MASTER}.py", c.scripts / f"{MASTER}.config.json", c.crate / "script.tsv", c.crate / "src/chapters/mod.rs", c.crate / "src/chapters/draw.rs",
             c.n / "report.json", c.n / "phrase-timings.json", c.jobs / TIMELINE / "timeline-summary.json", c.jobs / ASSEMBLE / "audio-comparison.json",
             c.s / "integration.json", c.crate / MAP_REL, c.crate / "narration_units.json", c.tools_path, c.binding_path, c.schedule, c.scored]
    wants += [c.jobs / TIMELINE / "timeline-output" / f"{ch}.{ext}" for ch in CHAPTERS for ext in ("timeline.tsv", "cues.tsv", "cuelog.tsv")]
    wants += [c.work / f"{ch}.{ext}" for ch in CHAPTERS for ext in ("mp4", "timeline.tsv", "layout.tsv")]
    record("master manifest: binds every file on Bronze's list and the agreed additions", all(str(p) in paths[MASTER] for p in wants), len(wants))
    record("manifests: the takes and receipts coherent verify reopens are bound", all(str(c.takes / "decode-receipt.json") in v and str(c.takes / "decoded/00_1.wav") in v and str(c.takes / "00_1.source") in v for v in paths.values()), "3 samples")
    summary = json.loads((c.jobs / MASTER / "master-summary.json").read_text())
    render = json.loads((c.jobs / c.job(CHAPTERS[0]) / "render-summary.json").read_text())
    record("summaries keep the agreed keys and add the evidence", all(k in summary for k in ("master", "srt", "vtt", "worker", "tools_sha256", "prediction_holds", "captions", "media"))
           and all(k in render for k in ("chapter", "mp4_sha256", "timeline_sha256", "soundtrack", "layout_findings", "media", "worker")), sorted(summary)[:4])
    record("D6: the summary holds the 13 holds, 364 sentences and 13 replays", len(summary["prediction_holds"]) == 13 and summary["captions"]["sentences"] == 364 and len(summary["replays"]) == 13, "13/364/13")
    cfg = json.loads((c.scripts / f"{MASTER}.config.json").read_text())
    record("master config: the agreed argv and captions_argv", cfg["argv"] == [str(c.bin), "master", "--narration", str(c.n), "--score", str(c.jobs / SCORE / "scored/stems"), "--out", str(c.work)]
           and cfg["captions_argv"] == [str(c.bin), "captions", "all", "--narration", str(c.n), "--out", str(c.work)] and list(cfg["render_jobs"]) == CHAPTERS, "exact")
    render_cfg = json.loads((c.scripts / f"{c.job(CHAPTERS[0])}.config.json").read_text())
    record("render config: 4K60, CRF 16, lint, veryfast, this narration and these stems", render_cfg["argv"] == [str(c.bin), "render", CHAPTERS[0], "--res", "3840x2160", "--fps", "60", "--narration", str(c.n),
           "--score", str(c.jobs / SCORE / "scored/stems"), "--out", str(c.work), "--lint", "--crf", "16", "--preset", "veryfast"], "exact")
    if BRONZE.is_file():
        spec = importlib.util.spec_from_file_location("bronze_verifier", BRONZE)
        B = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(B)
        a = types.SimpleNamespace(lane=str(c.jobs), jobs=str(c.scripts), crate=str(c.crate), narration=str(c.n), integration=str(c.s / "integration.json"),
                                  binding=str(c.binding_path), tools=str(c.tools_path), series="01", master_job=MASTER, verify=None, job_dir=None)
        try:
            found = B.bind_master(B.Ctx(a), CUES)
            record("Bronze's own bind_master accepts this master job", found["cfg"]["job"] == MASTER and len(found["renders"]) == 13, sha(BRONZE)[:12])
        except (B.Refused, KeyError) as error:
            record("Bronze's own bind_master accepts this master job", False, str(error).replace(str(base), "CASE")[:150])
    snapshot = {j: (c.jobs / j / "resource-receipt.json").stat().st_mtime_ns for j in jobs}
    folders = c.job_folders()
    done = c.run("run")
    record("resume: a second run launches nothing and exits 0", done.returncode == 0 and done.stdout.count("not launched") == 14 and c.job_folders() == folders
           and {j: (c.jobs / j / "resource-receipt.json").stat().st_mtime_ns for j in jobs} == snapshot, done.stdout.count("not launched"))
    script = c.scripts / f"{c.job(CHAPTERS[0])}.py"
    done = subprocess.run([sys.executable, "-I", "-B", str(script)], cwd=c.repo, capture_output=True, text=True, timeout=60)
    record("worker: a script run again by hand refuses", said(done, "does not show this script running"), "guard's finished receipt")


def resume_tests(base):
    c = Case(base, "render-failure", {"after:" + CHAPTERS[2]: [{"do": "exit", "code": 1}]})
    first = c.run("run")
    record("D1: a failure stops the chain; earlier chapters stand", first.returncode != 0 and c.rendered() == CHAPTERS[:2] and c.state(c.job(CHAPTERS[2]))["exit"] == 1 and c.state(c.job(CHAPTERS[3])) is None, "2 done")
    c.hooks.pop("after:" + CHAPTERS[2])
    c.write_hooks()
    folders, mtimes = c.job_folders(), (c.jobs / c.job(CHAPTERS[0]) / "resource-receipt.json").stat().st_mtime_ns
    second = c.run("run")
    record("D1: a rerun never retries by itself, even when the cause has cleared", said(second, "stopped before launching anything") and c.job_folders() == folders, "no new job")
    look = c.run("check")
    report = json.loads(look.stdout)
    record("D1: check names the blocked chapter and the retry root may pass", look.returncode == 1 and report["blocked"] == [CHAPTERS[2]] and report["chapters"][CHAPTERS[2]]["root_may_pass"] == f"--retry {CHAPTERS[2]}=r2"
           and report["chapters"][CHAPTERS[0]]["state"] == "done", report["blocked"])
    record("D1: a retry names the next attempt, not any attempt", said(c.run("run", "--retry", f"{CHAPTERS[2]}=r3"), "the next attempt is"), "r3 refused")
    record("D1: a chapter that passed cannot be retried", said(c.run("run", "--retry", f"{CHAPTERS[0]}=r2"), "nothing to retry"), "r2 refused")
    record("D1: a chapter with no attempt cannot be retried", said(c.run("run", "--retry", f"{CHAPTERS[5]}=r2"), "no attempt to retry"), "r2 refused")
    record("D1: --retry takes known names only", said(c.run("run", "--retry", "99_x=r2"), "expected one NAME=rN"), "99_x refused")
    failed_movie = sha(c.work / f"{CHAPTERS[2]}.mp4")
    done = c.run("run", "--retry", f"{CHAPTERS[2]}=r2")
    cfg = json.loads((c.scripts / f"{MASTER}.config.json").read_text()) if (c.scripts / f"{MASTER}.config.json").is_file() else {}
    record("D1: the named retry finishes the film from there", done.returncode == 0 and cfg["render_jobs"][CHAPTERS[2]] == c.job(CHAPTERS[2], "-r2") and c.state(MASTER)["exit"] == 0, "-r2 used")
    kept = c.aside / c.job(CHAPTERS[2])
    record("D1: the failed attempt's outputs are moved aside whole, never deleted", sha(kept / f"{CHAPTERS[2]}.mp4") == failed_movie and (kept / CHAPTERS[2] / "soundtrack_0.wav").is_file() and (kept / "moved.json").is_file(), "aside")
    record("D1: earlier chapters were not launched again", (c.jobs / c.job(CHAPTERS[0]) / "resource-receipt.json").stat().st_mtime_ns == mtimes, "same receipt")

    c = Case(base, "preflight")
    c.sbh("1")
    first = c.run("run")
    record("D1: a host that would be refused uses up no job name", said(first, "sbh refuses admission") and not (c.jobs / c.job(CHAPTERS[0])).exists(), "no folder")
    second = c.run("run")
    record("D1: the same name then runs", second.returncode == 0 and c.state(MASTER)["exit"] == 0, "base names")

    c = Case(base, "non-admission")
    c.sbh("0 1")
    first = c.run("run")
    r = c.state(c.job(CHAPTERS[0]))
    second = c.run("run")
    third = c.run("run", "--retry", f"{CHAPTERS[0]}=r2")
    record("D1: a guard non-admission stops the chain, stays stopped, and a named retry recovers",
           first.returncode != 0 and r["admitted"] is False and said(second, "stopped before launching anything") and third.returncode == 0 and c.state(MASTER)["exit"] == 0, "-r2")

    c = Case(base, "resource-stop", {"before:" + CHAPTERS[1]: [{"do": "exit", "code": 1}]})
    c.run("run")
    c.edit_json(c.jobs / c.job(CHAPTERS[0]) / "resource-receipt.json", lambda r: r.update(stopped_for_resource_limit=True))
    report = json.loads(c.run("check").stdout)
    record("D1: a resource stop is a failed attempt", report["chapters"][CHAPTERS[0]]["state"] == "blocked" and report["chapters"][CHAPTERS[0]]["why"] == "failed", report["chapters"][CHAPTERS[0]]["why"])

    c = Case(base, "moved-movie", {"before:" + CHAPTERS[1]: [{"do": "exit", "code": 1}]})
    c.run("run")
    movie = c.work / f"{CHAPTERS[0]}.mp4"
    with movie.open("ab") as handle:
        handle.write(b"x")
    changed = sha(movie)
    done = c.run("run")
    record("D1: a passed job whose movie changed is not skipped", said(done, "not the file") and c.state(c.job(CHAPTERS[0], "-r2")) is None, "unverified")
    c.hooks.pop("before:" + CHAPTERS[1])
    c.write_hooks()
    done = c.run("run", "--retry", f"{CHAPTERS[0]}=r2", "--retry", f"{CHAPTERS[1]}=r2")
    record("D1: root can retry it, and the changed bytes are kept", done.returncode == 0 and sha(c.aside / c.job(CHAPTERS[0]) / movie.name) == changed, "aside")

    c = Case(base, "crashed", {"before:" + CHAPTERS[1]: [{"do": "exit", "code": 1}]})
    c.run("run")
    c.edit_json(c.jobs / c.job(CHAPTERS[1]) / "resource-receipt.json", lambda r: r.pop("finished"))
    free = json.loads(c.run("check").stdout)["chapters"][CHAPTERS[1]]
    with (c.o / "expanded-native-lane.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        busy = json.loads(c.run("check").stdout)["chapters"][CHAPTERS[1]]
        fcntl.flock(lock, fcntl.LOCK_UN)
    record("D1: an unfinished receipt is running while the lane is held, crashed when it is free", free["why"] == "crashed" and busy["why"] == "running", [free["why"], busy["why"]])

    c = Case(base, "partial-launch")
    good = c.pins.read_text()
    c.pins.write_text(good.replace('"sha256": "', '"sha256": "0', 1))
    first = c.run("run")
    left = sorted(p.name for p in c.scripts.iterdir() if p.name != ".driver.lock")
    c.pins.write_text(good)
    second = c.run("run")
    record("D1: a launch the guard refused leaves no job, and the rerun takes its identical files", first.returncode != 0 and len(left) == 3 and not (c.jobs / c.job(CHAPTERS[0])).exists() and second.returncode == 0, len(left))

    c = Case(base, "refused-freeze")
    tool = c.s / "workspace/tools/narration_cache.py"
    tool.rename(c.dir / "narration_cache.py")
    done = c.run("run")
    record("D1: a refused freeze writes no script, config or manifest", said(done, "inputs are missing") and [p.name for p in c.scripts.iterdir()] == [".driver.lock"], "folder empty")

    c = Case(base, "second-driver")
    c.scripts.mkdir()
    with (c.scripts / ".driver.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        done = c.run("run")
        fcntl.flock(lock, fcntl.LOCK_UN)
    record("D1: a second driver refuses", said(done, "another render_master_remote.py run"), "lock held")

    c = Case(base, "master-retry", {"captions": [{"do": "exit", "code": 1}]})
    first = c.run("run")
    stranded = sha(c.work / f"{TOPIC}_master.mp4")
    c.hooks.pop("captions")
    c.write_hooks()
    second = c.run("run")
    third = c.run("run", "--retry", "master=r2")
    record("D7 stays, recoverable: a caption refusal stops the job, and a named master retry finishes", first.returncode != 0 and said(second, "stopped before launching anything") and third.returncode == 0 and c.state(MASTER + "-r2")["exit"] == 0
           and sha(c.aside / MASTER / f"{TOPIC}_master.mp4") == stranded, "-r2")


def comparison_tests(base):
    c = Case(base, "no-comparison")
    c.scored.rename(c.dir / "held.json")
    done = c.run("check")
    record("D2: an unwritten comparison is pending, exit 3", done.returncode == 3 and json.loads(done.stdout)["ready"] is False, "exit 3")
    done = c.run("run")
    record("D2: run without a wait refuses and launches nothing", said(done, "not ready") and not (c.jobs / c.job(CHAPTERS[0])).exists(), "no job")
    timer = threading.Timer(1.0, lambda: (c.dir / "held.json").rename(c.scored))
    timer.start()
    done = c.run("run", "--wait-seconds", "3")
    timer.join()
    record("D2: run waits for the comparison, then goes on", done.returncode == 0 and "WAIT" in done.stdout and c.state(MASTER)["exit"] == 0, "waited")

    c = Case(base, "hard-failure")
    c.edit_json(c.scored, lambda d: d.update(hard_failures=["a stem moved"]))
    record("D2: a hard failure refuses at once, even with a wait", said(c.run("run", "--wait-seconds", "60"), "records hard failures"), "no wait")
    c = Case(base, "half-written")
    c.schedule.write_text('{"hard_fail')
    record("D2: a half-written comparison is pending, not passed", c.run("check").returncode == 3, "exit 3")
    c = Case(base, "no-key")
    c.edit_json(c.schedule, lambda d: d.pop("hard_failures"))
    record("D2: a comparison without the key refuses", said(c.run("check"), "records hard failures"), "None")

    c = Case(base, "surprise")
    c.edit_json(c.schedule, lambda d: d.update(not_as_predicted=[CHAPTERS[6]]))
    record("D2: an unpredicted move stops until root names the file it read", said(c.run("check"), "--approved-schedule-sha256"), "no flag")
    record("D2: the wrong hash stops too", said(c.run("check", "--approved-schedule-sha256", "0" * 64), "--approved-schedule-sha256"), "wrong")
    record("D2: root's exact hash passes", c.run("check", "--approved-schedule-sha256", sha(c.schedule)).returncode == 0, "exact")
    c = Case(base, "stems")
    c.edit_json(c.scored, lambda d: d.update(stems_changed=[CHAPTERS[4], CHAPTERS[6]]))
    record("D2: a stem changed in an unmoved chapter stops until named", said(c.run("check"), "--approved-score-sha256") and c.run("check", "--approved-score-sha256", sha(c.scored)).returncode == 0, CHAPTERS[6])

    c = Case(base, "failed-score")
    c.edit_json(c.jobs / SCORE / "resource-receipt.json", lambda r: r.update(exit=1))
    record("D2: a failed score refuses at once, even with a wait", said(c.run("run", "--wait-seconds", "60"), f"{SCORE} is failed"), "no wait")
    c = Case(base, "absent-score")
    (c.jobs / SCORE).rename(c.dir / "score")
    record("D2: an absent prior job is pending", c.run("check").returncode == 3, "exit 3")
    c = Case(base, "no-stem")
    (c.jobs / SCORE / f"scored/stems/{CHAPTERS[3]}.music.f32.wav").rename(c.dir / "stem")
    record("score: a missing stem refuses", said(c.run("check"), "no score stem"), CHAPTERS[3])
    c = Case(base, "qa-length")
    c.edit_json(c.jobs / SCORE / f"scored/stems/qa/{CHAPTERS[3]}.json", lambda d: d["metrics"][0].update(value=1))
    record("score: a QA length that is not frames x 800 refuses", said(c.run("check"), "QA measured length"), CHAPTERS[3])
    c = Case(base, "qa-fail")
    c.edit_json(c.jobs / SCORE / f"scored/stems/qa/{CHAPTERS[3]}.json", lambda d: d.update(passed=False))
    record("score: a failed QA report refuses", said(c.run("check"), "QA failed"), CHAPTERS[3])


def binding_tests(base):
    def refused(name, phrase, change):
        c = Case(base, "bind-" + name.replace(" ", "-"))
        change(c)
        record(f"D4: {name} refuses", said(c.run("check"), phrase), phrase)
    refused("binding names another binary hash", "the binding says", lambda c: c.edit_json(c.binding_path, lambda d: d["build"].update(binary_sha256="0" * 64)))
    refused("binding names another binary path", "the binding names the binary", lambda c: c.edit_json(c.binding_path, lambda d: d["build"].update(binary="/other/binary")))
    refused("binding names another narration", "the binding names the narration", lambda c: c.edit_json(c.binding_path, lambda d: d["audio"].update(narration=str(c.s / "narration01"))))
    refused("binary changed after the build", "the binding says", lambda c: c.bin.open("a").write("# moved\n"))
    refused("timeline ran another binary", "its binary is", lambda c: c.rewrite_prior(TIMELINE, "timeline", M.TIMELINE_SUCCESS, binary="/other/binary"))
    refused("timeline ran another narration", "its narration is", lambda c: c.rewrite_prior(TIMELINE, "timeline", M.TIMELINE_SUCCESS, narration=str(c.s / "narration01")))
    refused("score ran another binary hash", "its binary_sha256 is", lambda c: c.rewrite_prior(SCORE, "score", M.SCORE_SUCCESS, binary_sha256="0" * 64))
    refused("score read another timeline", "did not read this timeline", lambda c: c.rewrite_prior(SCORE, "score", M.SCORE_SUCCESS, argv=["x"]))
    refused("timeline row plays outside the narration", "outside the bound narration", lambda c: timeline_elsewhere(c))
    refused("crate file moved after the binding", "moved since the binding", lambda c: (c.crate / "src/chapters/draw.rs").write_text("// moved\n"))
    refused("bound narration file moved", "moved since the binding", lambda c: (c.n / "00_1.wav").write_bytes(b"moved"))
    refused("timeline script is not what its guard ran", "guard ran", lambda c: c.edit_json(c.jobs / TIMELINE / "resource-receipt.json", lambda r: r.update(command=["python3", "other.py"])))
    refused("rebind row absent", "names 0 rebind jobs", lambda c: rebind_rows(c, []))
    refused("two rebind rows", "names 2 rebind jobs", lambda c: rebind_rows(c, [REBIND_FAILED, REBIND]))
    refused("assembly names the failed rebind", f"{REBIND_FAILED} is failed", lambda c: rebind_rows(c, [REBIND_FAILED]))
    refused("rebind receipt changed since the assembly", "not the one the assembly froze", lambda c: c.edit_json(c.jobs / REBIND / "resource-receipt.json", lambda r: r.update(extra=1)))
    refused("binding does not hold the assembly's manifest", "files with the hash", lambda c: c.edit_json(c.jobs / ASSEMBLE / "resource-receipt.json", lambda r: r.update(inputs_sha256="0" * 64, final_inputs_sha256="0" * 64)))
    refused("rebind receipt shows another command", "did not run rebind_audio.py --apply", lambda c: rebind_command(c, ["python3", "other.py"]))
    refused("rebind receipt lost its phase", "did not run rebind_audio.py --apply", lambda c: rebind_command(c, None))
    refused("jobs out of order", "did not run in that order", lambda c: c.edit_json(c.jobs / ASSEMBLE / "resource-receipt.json", lambda r: r.update(finished=when(9000))))
    refused("map holds lost a prediction", "prediction holds", lambda c: map_short(c))


def timeline_elsewhere(c):
    path = c.jobs / TIMELINE / "timeline-output" / f"{CHAPTERS[3]}.timeline.tsv"
    path.write_text(path.read_text().replace(str(c.n), str(c.s / "narration01")))
    c.edit_json(c.jobs / TIMELINE / "timeline-summary.json", lambda d: d["chapters"][3].update(timeline_sha256=sha(path)))


def rebind_rows(c, jobs):
    rows = [{"path": str(c.jobs / j / "resource-receipt.json"), "sha256": sha(c.jobs / j / "resource-receipt.json")} for j in jobs]
    c.write_json(c.assemble_inputs, {"files": rows})
    digest = sha(c.assemble_inputs)
    c.edit_json(c.jobs / ASSEMBLE / "resource-receipt.json", lambda r: r.update(inputs_sha256=digest, final_inputs_sha256=digest))
    c.edit_json(c.binding_path, lambda d: d["files"].update({str(c.assemble_inputs): digest}))


def rebind_command(c, command):
    """Changes the good rebind's receipt, then refreezes it where the assembly froze it, so only this gate can refuse."""
    def change(r):
        if command is None:
            r.pop("phase")
        else:
            r["command"] = command
    c.edit_json(c.jobs / REBIND / "resource-receipt.json", change)
    rebind_rows(c, [REBIND])


def map_short(c):
    c.edit_json(c.crate / MAP_REL, lambda d: d["visual_holds"].pop(0))
    digest = sha(c.crate / MAP_REL)
    c.edit_json(c.n / "report.json", lambda d: d["meta"]["direction_contract"]["binding"]["contract"]["map"].update(sha256=digest))
    c.write_binding()
    c.rewrite_prior(TIMELINE, "timeline", M.TIMELINE_SUCCESS)
    c.rewrite_prior(SCORE, "score", M.SCORE_SUCCESS)


def picture_tests(base):
    last = CHAPTERS[12]

    def moved(name, target, phrase):
        c = Case(base, "moved-" + name)
        c.hooks["after:" + last] = [{"do": "append", "path": str(target(c))}]
        c.write_hooks()
        done = c.run("run")
        record(f"D5: {name} changed after its render stops the chain before the master", said(done, phrase) and c.state(MASTER) is None, phrase)
    moved("movie", lambda c: c.work / f"{CHAPTERS[0]}.mp4", "not the file")
    moved("timeline-table", lambda c: c.work / f"{CHAPTERS[0]}.timeline.tsv", "not the file")
    moved("cue-table", lambda c: c.work / f"{CHAPTERS[0]}.cues.tsv", "not the file")
    moved("lint-table", lambda c: c.work / f"{CHAPTERS[0]}.layout.tsv", "not the file")
    moved("soundtrack", lambda c: c.work / "soundtrack" / CHAPTERS[0] / "soundtrack_0.wav", "soundtrack capture moved")

    c = Case(base, "moved-during-master")
    c.hooks["master"] = [{"do": "append", "path": str(c.work / f"{CHAPTERS[0]}.mp4")}]
    c.write_hooks()
    done = c.run("run")
    record("D5: a movie changed while the master ran fails the job, with no summary", done.returncode != 0 and "no longer matches its render summary" in c.log(MASTER) and not (c.jobs / MASTER / "master-summary.json").exists(), "after the master")

    def master_refused(name, hooks, phrase):
        c = Case(base, "master-" + name.replace(" ", "-"), hooks)
        done = c.run("run")
        record(f"D6: {name} fails the master, with no summary", done.returncode != 0 and phrase in c.log(MASTER) and not (c.jobs / MASTER / "master-summary.json").exists(), phrase)
    master_refused("a lost prediction hold", {"drop_hold": True}, "planned 12 prediction holds")
    master_refused("a different hold", {"wrong_hold": True}, "planned 13 prediction holds")
    master_refused("a hold missing from the captions", {"lost_stamp": True}, "the captions lost the prediction holds")
    master_refused("363 sentences", {"sentences": 363}, "report 363 sentences")
    master_refused("a missing replay line", {"no_replay": CHAPTERS[7]}, "replay lines")
    master_refused("the wrong frame count", {"master_frames_delta": 1}, "the film has 78")
    master_refused("a promoted plan", {"promotion": True}, "plan line differs")
    master_refused("captions that disagree", {"vtt_differs": True}, "the SRT has")
    master_refused("a master that is not 4K", {"probe": {f"{TOPIC}_master": {"video": {"width": 1920, "height": 1080}}}}, "not the 4K60 picture")
    master_refused("a master short of frames", {"probe": {f"{TOPIC}_master": {"video": {"nb_frames": "77"}}}}, "holds 77 frames")

    first = CHAPTERS[0]

    def render_refused(name, hooks, phrase):
        c = Case(base, "render-" + name.replace(" ", "-"), hooks)
        done = c.run("run")
        record(f"output: {name} fails that chapter, with no summary", done.returncode != 0 and phrase in c.log(c.job(first)) and not (c.jobs / c.job(first) / "render-summary.json").exists() and c.state(c.job(CHAPTERS[1])) is None, phrase)
    render_refused("1080p", {"probe": {first: {"video": {"width": 1920, "height": 1080}}}}, "not the 4K60 picture")
    render_refused("30 fps", {"probe": {first: {"video": {"r_frame_rate": "30/1"}}}}, "not the 4K60 picture")
    render_refused("hevc", {"probe": {first: {"video": {"codec_name": "hevc"}}}}, "not the 4K60 picture")
    render_refused("a missing frame", {"probe": {first: {"video": {"nb_frames": "5"}}}}, "holds 5 frames")
    render_refused("no audio", {"probe": {first: {"drop_audio": True}}}, "1 video and 0 audio")
    render_refused("mono audio", {"probe": {first: {"audio": {"channels": 1}}}}, "not 48 kHz stereo AAC")
    render_refused("CRF 23", {"sei_kind": {first: "crf23"}}, "x264's own record of CRF 16")
    render_refused("no x264 header", {"sei_kind": {first: "none"}}, "x264's own record of CRF 16")
    render_refused("a changed cue table", {"changed_cues": [first]}, "full cue table differs")
    render_refused("a changed timeline row", {"changed_timeline": [first]}, "differs from the 60 fps timeline")
    render_refused("missing lint", {"no_layout": [first]}, "rendered lint is missing")
    render_refused("a short soundtrack", {"short_track": [first]}, "not the whole soundtrack")
    render_refused("no soundtrack", {"no_track": [first]}, "no soundtrack capture")


def host_tests(base):
    def refused(name, phrase, change, code=None):
        c = Case(base, "host-" + name.replace(" ", "-"))
        change(c)
        done = c.run("check")
        record(f"tools: {name} refuses", (done.returncode == code if code else said(done, phrase)), phrase)
    refused("ffmpeg changed after it was measured", "this host's ffmpeg", lambda c: c.ffmpeg.open("a").write("# moved\n"))
    refused("no libx264", "no libx264", lambda c: (c.hooks.update(no_x264=True), c.write_hooks()))
    refused("tools file for another host", "is not a", lambda c: c.write_tools(host="other"))
    refused("another python3", "this python3 is", lambda c: other_python(c))
    refused("another ffprobe earlier on PATH", "PATH", lambda c: path_shadow(c))
    refused("unmeasured tools", "", lambda c: c.tools_path.rename(c.dir / "tools.json"), code=3)
    c = Case(base, "seal")
    record("seal: a wrong seal refuses", said(c.run("check", seal="0" * 64), "is not the seal"), "SHA256SUMS")
    (c.repo / BUNDLE_REL / "job_worker.py").open("a").write("# moved\n")
    record("seal: a changed worker refuses", said(c.run("check"), "differs from its seal"), "job_worker.py")
    c = Case(base, "off-host", driver_host="not-this-host")
    record("host: the driver refuses off its host", said(c.run("check"), "runs only on not-this-host"), "host")


def other_python(c):
    twin = c.dir / "python-twin/python3"
    executable(twin, "#!/bin/bash\n")
    c.write_tools(python3={"path": str(twin.resolve()), "sha256": sha(twin), "version": "x", "pillow": "12.1.1"})


def path_shadow(c):
    moved = c.dir / "media2/ffprobe"
    moved.parent.mkdir()
    c.ffprobe.rename(moved)
    executable(c.ffprobe, "#!/bin/bash\n")
    c.write_tools(ffprobe={"path": str(moved.resolve()), "sha256": sha(moved), "version": "x"})


def same(a, b):
    return Path(a).resolve() == Path(b).resolve()


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    base, out = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    if base.exists() or out.exists():
        raise SystemExit("SCRATCH and RESULTS must not exist")
    base.mkdir(parents=True)
    os.chdir(REAL)
    for group in (contract_tests, resume_tests, comparison_tests, binding_tests, picture_tests, host_tests):
        group(base)
    counts = {"tests": len(results), "passed": sum(r["pass"] for r in results), "failed": sum(not r["pass"] for r in results)}
    data = {"driver_sha256": sha(BUNDLE / "render_master_remote.py"), "worker_sha256": sha(BUNDLE / "job_worker.py"),
            "guard01_sha256": sha(REAL / O_REL / "guard01.py"), "guard_sha256": sha(REAL / GUARD_REL), "guard_source_sha256": sha(REAL / SOURCE_REL),
            "bronze_verifier_sha256": sha(BRONZE) if BRONZE.is_file() else None, "python": sys.version.split()[0],
            "counts": counts, "results": results, "limits": LIMITS,
            "scope": "Throwaway repositories with a stand-in binary. No real job, build, render, audio or lane was involved."}
    with out.open("x") as handle:
        handle.write(json.dumps(data, indent=2) + "\n")
    print(counts)
    return 0 if not counts["failed"] else 1


if __name__ == "__main__":
    sys.exit(main())
