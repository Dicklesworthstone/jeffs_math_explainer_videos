#!/usr/bin/env python3
"""Final technical verification of the repaired logarithm master on the remote worker (bead 87do.63).

This is verify_final_picture.py (sealed deb12d24, BronzeJaguar, itself verify_stage09_final.py version 02, sealed
e8e19275) moved to root's remote lane on mac-mini-max, by BronzeJaguar, 2026-11-05. docs/final-picture-to-remote.diff
is the whole change. Every check of that helper stays. The change is:

  names     the remote jobs, lane, guard, binding and bound files
  host      every command refuses off mac-mini-max: the master, tools and job records exist only there
  tools     python3, ffmpeg and ffprobe come from a tools file measured on the worker, by path and hash. PATH is never
            inherited: the strict check runs with the pinned folders first, and refuses if another ffmpeg would answer
  sheets    Pillow is proved at check time, because a contact sheet is the job's last step
  record    verification.json gains `worker` and `tools_sha256`; every earlier key stays

The gates, from the earlier helper:

  G1  a fresh master: not the retired master or binary; build, assembly, timeline, score, 13 renders and master are
      all successful, stable guard receipts, in that order in time.
  G2  the repaired source: integration.json's changed rows, each with its stage09 hash before, a new hash after, on
      disk, bound by the master and compiled by the build; the binding names this binary, narration and map.
  G3  the new metadata: the direction map the narration binds, its 13 + 2 rows against the current source, the unit
      plan and script hashes in report.json, and audio byte-identical to the retained takes.
  G4  every hold's real window, read from the timeline's own rows, and the closing hold and fade after 12_21.
  G5  anchors.json: every earlier flagged item, all 13 pause lines, the 04 answer reveal, the chapter 11 glosses,
      sample X and one-logarithm formula, and the closing. Each resolves on the NEW schedule as a base plus a frame
      offset. A base that does not resolve refuses. Old film frames are never read here.

On the worker, from the repository root (every path is an argument; absolute paths):

  tools       --out T --ffmpeg F --ffprobe P
              measures this host's python3, Pillow, ffmpeg and ffprobe and writes a new tools file
  check       --lane L --jobs J --crate C --narration N --series SS --integration I --binding B --tools T
              [--master-job logarithm-remote-master01-rN]
              read only: all gates above, the master's command binding, the 13-chapter schedule, exact captions and
              the picture plan
  write-job   ... --verify NN --job-dir D
              writes D/logarithm-remote-verifyNN.py and its guard inputs, then prints the guard line
  verify-job  ... --verify NN --job-dir D
              after root's guarded run: its receipt, log, measurements, the extracted pictures and the anchor ledger

L is the remote guard's job folder (O/jobs). HANDOFF.md names the others. Nothing here launches a job, connects to a
host, views a picture, listens, or accepts the film. Root launches the printed guard line, fetches the sheets, views
every one, and owns the final review.
"""
import argparse
import ast
from collections import Counter
from datetime import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import sys
import wave

HERE = Path(__file__).resolve()
REPO = HERE.parents[5]
ANCHORS = HERE.parent / "anchors.json"
STAGE = "logarithm-remote"
HOST = "mac-mini-max"  # root's guard01.py refuses any other host, and so does this helper
TOOLS_SCHEMA = "logarithm-remote-tools-v1"
COMPARISONS = {"schedule": "schedule-comparison01.json", "score": "score-comparison01.json"}  # beside the binding: RubyRaven's O files
RENDER_ATTEMPT = r"(-r[2-9])?"  # LavenderPike's contract: render_jobs names the attempt that succeeded
REBIND_JOB = re.escape(STAGE) + r"-rebind\d\d"  # which one is read from the binding, never assumed: see check_rebind
REBIND_TOOL = "rebind_audio.py"
MIN_PILLOW = (10, 1)  # make_sheets: ImageDraw.text(font_size=...)
TOPIC = "logarithm_mysteries"
FPS = 60
CHAPTERS = ["00_ratio_ruler", "01_building_ruler", "02_slide_rule", "03_folded_decade", "04_undoing_growth",
            "05_complex_turns", "06_lifting_loops", "07_branch_cut", "08_product_rule", "09_matrix_logs",
            "10_doing_nothing", "11_unreachable", "12_what_it_forgets"]
CUES = 80
DIRECTION_MAP = "docs/narration-direction-map-oliver17.json"
HOLD_KINDS = {"prediction": 13, "reading": 2}
PREDICT_FADE_FRAMES = 15  # draw.rs predict: fade_out_all(&[bar, probe], 0.25) between the think and the next cue
BINDING_SCHEMA = "logarithm-remote-binding-v1"
# The retired stage09 film: a fresh master can be neither, and the script and the audio stay.
RETIRED_MASTER = "00236d1a6915ff968235a30ab59769b02125ece0fee482e4fa3154c265ac7c24"
RETIRED_BINARY = "e88e58e03b69b18f9e7c77dbeb4d810bc1b660637074b86746a6b98e497cda08"
SCRIPT_SHA256 = "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a"
# The six rendered stage09 sources the repairs replaced. Their new hashes come from integration.json, never from here.
STAGE09_SOURCES = {
    "undoing_growth.rs": "4dc4c555122d9cd5423d7094336c1d5e6f7f9afb77d6850a38bc0148c20180ef",
    "unreachable.rs": "c822e8b46796e79e2f8461872fe538de711a4fde755913ec57696710d0cc28a7",
    "lifting_loops.rs": "01885b9f558c36bc8af425bfdf427e4b30d74d575819121e52988b5d06db7346",
    "complex_turns.rs": "93f663d372ab79a77139287dc38b94c148909f7ea4a6ba549884c91f342eb19a",
    "draw.rs": "2a4e84504bab636433cd419b2c9990986d7c549bf05aa3432d33bd5a9b60fe28",
    "what_it_forgets.rs": "6951d3b2f157387683efaa3ec7c9d9d12a23c8448df247531cd5f0081bab1e28",
}
# The route's exact success lines: prepare_stage09_render.py SUCCESS, except the render line, which root's
# render-master01.py sets for these jobs. The master keeps stage09's line (it inherits the old config's).
TIMELINE_SUCCESS = "TIMELINE PASS: 13 chapters at 60 fps; 80 cue placements in script order."
SCORE_SUCCESS = "SCORE PASS: all 13 chapter score QA reports passed."
RENDER_SUCCESS = "4K60 chapter rendered; full timeline/cue tables match; picture review remains open."
MASTER_SUCCESS = ("MASTER WRITTEN: candidate master and captions; strict verify, pictures, ASR, caption screen, "
                  "loudness and listening remain.")
VERIFY_SUCCESS = ("FINAL VERIFY PASS: strict video check, frame count, A/V length and exact captions passed; "
                  "every sheet still needs root's viewing.")
# render-master01.py worker_text: the job's CONFIG line when the config sits beside the script
CONFIG_SIDECAR = 'pathlib.Path(__file__).with_suffix(".config.json").read_text()'
P_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
M_REL = "narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103"
STRICT_REL = f"{M_REL}/verify_video_preserve_logs01.sh"
SCREEN_REL = f"{M_REL}/caption-interval-audit01.py"
O_REL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01"
REMOTE_GUARD_REL = f"{O_REL}/guard01.py"
REMOTE_GUARD_PINS_REL = f"{O_REL}/guard-pins01.json"
GUARD_REL = f"{P_REL}/engine-secondary-validation01/guard_owner56g_target20g.py"
GUARD_SOURCE_REL = ("narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/"
                    "run_bounded_phase_oliver_owner48g_target14g_20261102.py")
PINS = {
    STRICT_REL: "3f4c68dbdcb02c1c90ff03cb4f20afc36792b7f20c6cc111a5eef21617a11410",
    SCREEN_REL: "3cc2643615ce5263b01d429ae6f72326c95e777af3e7f29ef9266a05155fa0c4",
    GUARD_REL: "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097",
    GUARD_SOURCE_REL: "819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da",
    REMOTE_GUARD_REL: "cc57457f1ca26f1d4ddb5780e2f7cf4160860bbd456057d380fcb6c028c27291",
    REMOTE_GUARD_PINS_REL: "0501d7b831717ac6f9be30eb0d7bcbdaccfcbcde238ba52f9f8b7bded606bbb6",
}
EXPECT = {"EXPECT_RES": "3840x2160", "EXPECT_FPS": "60", "EXPECT_BT709": "1", "EXPECT_LUFS": "-16", "EXPECT_TP_MAX": "-1.0"}
OUTPUTS = ("strict-video-verification.log", "decode-logs", "encoded-loudness.log", "caption-screen.json", "verification.json")
SCOPE = ("Technical checks and extracted pictures of the repaired logarithm master only, on mac-mini-max (retained "
         "ElevenLabs Oliver recordings). No listening, picture viewing, motion review, fine sync, ASR, independent review or publication "
         "acceptance is claimed.")
STAMP = r"(\d\d):([0-5]\d):([0-5]\d)[{d}](\d{{3}})"
# verify_video_preserve_logs01.sh lines 173 and 175: the software decode always runs first, with -xerror
SOFTWARE_DECODE = "full decode (video + audio, software, stops at the first error): 0 error lines, ffmpeg exit 0, "


class Refused(Exception):
    pass


def need(ok, message):
    if not ok:
        raise Refused(message)


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def load(path):
    need(Path(path).is_file(), f"{path} is missing")
    return json.loads(Path(path).read_text(encoding="utf-8"))


def on_host():
    """The worker's records, master and tools are real only on the worker."""
    name = socket.gethostname().split(".")[0].lower()
    need(name == HOST, f"this is {name}: the remote verifier runs only on {HOST}")


def pillow_version():
    try:
        import PIL
    except ImportError:
        return None
    return PIL.__version__


def tool_row(path):
    done = subprocess.run([path, "-version"], check=True, capture_output=True, text=True, timeout=60)
    return {"path": str(Path(path).resolve()), "sha256": sha(path), "version": done.stdout.splitlines()[0]}


def write_tools(a):
    """Measures this host: what a job will run, by real path and hash, never a local assumption."""
    on_host()
    for name in ("out", "ffmpeg", "ffprobe"):
        value = getattr(a, name)
        need(value and Path(value).is_absolute(), f"tools needs an absolute --{name}")
    out = Path(a.out)
    need(not out.exists(), f"{out} exists: tools never replaces a file")
    # the interpreter as launched, not its Cellar target: launching that target directly is a different command line
    body = {"schema": TOOLS_SCHEMA, "host": HOST,
            "python3": {"path": sys.executable, "sha256": sha(sys.executable), "version": sys.version.split()[0], "pillow": pillow_version()},
            "ffmpeg": tool_row(a.ffmpeg), "ffprobe": tool_row(a.ffprobe)}
    check_pillow(body)
    with open(out, "x") as handle:
        handle.write(json.dumps(body, indent=2) + "\n")
    return {"tools": str(out), "tools_sha256": sha(out), **body}


def check_pillow(tools):
    live = pillow_version()
    parts = tuple(int(x) for x in re.findall(r"\d+", live or "")[:2])
    need(live is not None and parts >= MIN_PILLOW, f"this python3 has Pillow {live}: the contact sheets need {MIN_PILLOW[0]}.{MIN_PILLOW[1]} or later")
    need(tools["python3"]["pillow"] == live, f"the tools file says Pillow {tools['python3']['pillow']}, this python3 has {live}")


def check_tools(tools, label):
    """This host, this interpreter and these exact files, or a refusal."""
    need(tools.get("schema") == TOOLS_SCHEMA and tools.get("host") == HOST, f"{label}: not a {TOOLS_SCHEMA} for {HOST}")
    for name in ("python3", "ffmpeg", "ffprobe"):
        row = tools[name]
        need(Path(row["path"]).is_absolute() and Path(row["path"]).is_file() and sha(row["path"]) == row["sha256"],
             f"{label}: this host's {name} is not {row['path']} {row['sha256'][:12]}")
    need(same(sys.executable, tools["python3"]["path"]), f"{label}: this interpreter is {sys.executable}, not {tools['python3']['path']}")
    check_pillow(tools)


def tool_path(tools):
    """PATH for root's strict check, which finds both tools by name: pinned folders first, then the system's only."""
    return os.pathsep.join(dict.fromkeys([str(Path(tools[name]["path"]).parent) for name in ("ffmpeg", "ffprobe")] + ["/usr/bin", "/bin"]))


def check_tool_path(tools):
    path = tool_path(tools)
    for name in ("ffmpeg", "ffprobe"):
        found = shutil.which(name, path=path)
        need(found is not None and same(found, tools[name]["path"]), f"PATH {path} finds {name} at {found}, not the pinned {tools[name]['path']}")
    return path


def receipt_ok(r):
    return (r.get("admitted") is True and r.get("exit") == 0 and bool(r.get("finished"))
            and r.get("inputs_sha256") == r.get("final_inputs_sha256")
            and not r.get("stopped_for_resource_limit") and "error" not in r)


def ran(receipt, script):
    """The guard ran exactly this script. Root's render-master01.py launches with sys.executable, the route's own jobs
    with `python3`, so the interpreter may be either form of a Python 3; nothing else may differ."""
    command = receipt.get("command")
    return (isinstance(command, list) and len(command) == 4 and command[1:] == ["-I", "-B", str(script)]
            and re.fullmatch(r"python3(\.\d+)?", Path(command[0]).name) is not None)


def stamp(receipt, key):
    return datetime.fromisoformat(receipt[key])


def last_line(path):
    lines = [x for x in Path(path).read_text(errors="replace").splitlines() if x.strip()] if Path(path).is_file() else []
    return lines[-1] if lines else None


def norm(text):
    return " ".join(text.split())


def decode_lines(text):
    """Root's strict check prints one `full decode (...)` line per decode: the software one, then the hardware second
    opinion when it runs. It keeps a single decode log, which the second decode rewrites, so these lines are the
    record of both."""
    return [row for row in text.splitlines() if row.startswith("full decode (")]


def software_decode_clean(decodes):
    return isinstance(decodes, list) and bool(decodes) and isinstance(decodes[0], str) and decodes[0].startswith(SOFTWARE_DECODE)


# ---------------------------------------------------------------- parsers of the kit's own files

def read_script(path):
    """coherent_narration.read_script's rule: blank and '#' rows skipped, then exactly `cue<TAB>text`."""
    rows = {}
    for number, row in enumerate(Path(path).read_text(encoding="utf-8").splitlines(), 1):
        if not row.strip() or row.startswith("#"):
            continue
        cells = row.split("\t")
        need(len(cells) == 2 and re.fullmatch(r"\d\d_\d+", cells[0]) and cells[1].strip(),
             f"script row {number} is not cue<TAB>text")
        need(cells[0] not in rows, f"script cue {cells[0]} repeats")
        rows[cells[0]] = cells[1]
    need(rows, "the script is empty")
    return rows


def registry(crate):
    """The crate's chapter registry, in order (`key: "..."` in src/chapters/mod.rs)."""
    return re.findall(r'^\s*key: "([^"]+)",$', (Path(crate) / "src/chapters/mod.rs").read_text(), re.M)


def parse_timeline(text, name):
    """explainer_score::timeline::ChapterTimeline::parse, record for record."""
    t = {"chapter": None, "fps": 0, "frames": 0, "lines": [], "marks": [], "breaths": [], "listens": []}

    def integer(f, k, n):
        need(k < len(f) and re.fullmatch(r"\d+", f[k]), f"{name} line {n}: bad integer")
        return int(f[k])

    def number(f, k, n):
        try:
            return float(f[k])
        except (IndexError, ValueError):
            raise Refused(f"{name} line {n}: bad number") from None

    for n, raw in enumerate(text.split("\n"), 1):
        line = raw.rstrip("\r")
        if not line or line.startswith("#"):
            continue
        f = line.split("\t")
        if f[0] == "chapter":
            need(len(f) > 1, f"{name} line {n}: missing name")
            t["chapter"] = f[1]
        elif f[0] == "fps":
            t["fps"] = integer(f, 1, n)
        elif f[0] == "frames":
            t["frames"] = integer(f, 1, n)
        elif f[0] == "line":
            need(len(f) == 5, f"{name} line {n}: line needs id, start_frame, gain_db, path")
            t["lines"].append({"id": f[1], "start": integer(f, 2, n), "gain_db": number(f, 3, n), "wav": f[4]})
        elif f[0] == "mark":
            t["marks"].append({"label": f[1] if len(f) > 1 else "", "frame": integer(f, 2, n)})
        elif f[0] == "breath":
            t["breaths"].append({"frame": integer(f, 1, n), "seconds": number(f, 2, n)})
        elif f[0] == "listen":
            need(len(f) <= 5, f"{name} line {n}: listen needs frame, seconds, label and at most a path")
            t["listens"].append({"frame": integer(f, 1, n), "seconds": number(f, 2, n)})
        else:
            raise Refused(f"{name} line {n}: unknown record {f[0]!r}")
    need(t["fps"] != 0, f"{name}: missing fps")
    ids = [x["id"] for x in t["lines"]]
    need(len(set(ids)) == len(ids), f"{name}: a line id repeats")
    return t


def schedule(t):
    return (t["chapter"], t["fps"], t["frames"], [(x["id"], x["start"], x["gain_db"], x["wav"]) for x in t["lines"]])


def layout_anchors(text, name):
    """Layout rows are `checkpoint<TAB>kind<TAB>detail`; both of the kit's checkpoint forms end ` @<t.t>s`."""
    found = []
    for row in text.split("\n"):
        if not row.strip():
            continue
        m = re.fullmatch(r".* @([0-9]+\.[0-9])s", row.split("\t")[0])
        need(m is not None, f"{name}: unrecognized layout finding: {row[:90]}")
        found.append((round(float(m.group(1)) * FPS), row))
    return found


def parse_captions(text, vtt):
    """The kit's exact writers (captions.rs to_srt / to_vtt): `k\\nS --> E\\nline[\\nline]\\n\\n`, VTT without k."""
    name = "VTT" if vtt else "SRT"
    if vtt:
        need(text.startswith("WEBVTT\n\n"), "VTT: no WEBVTT header")
        text = text[len("WEBVTT\n\n"):]
    need(text.endswith("\n\n"), f"{name}: does not end with an empty line")
    stamp = STAMP.format(d="." if vtt else ",")
    out = []
    for k, block in enumerate(text[:-2].split("\n\n"), 1):
        lines = block.split("\n")
        if not vtt:
            need(lines[0] == str(k), f"SRT caption {k}: numbered {lines[0]!r}")
            lines = lines[1:]
        m = re.fullmatch(f"{stamp} --> {stamp}", lines[0])
        need(m is not None, f"{name} caption {k}: malformed time line {lines[0]!r}")
        g = [int(x) for x in m.groups()]
        start = ((g[0] * 60 + g[1]) * 60 + g[2]) * 1000 + g[3]
        end = ((g[4] * 60 + g[5]) * 60 + g[6]) * 1000 + g[7]
        need(1 <= len(lines) - 1 <= 2 and all(x.strip() for x in lines[1:]), f"{name} caption {k}: needs one or two text lines")
        out.append((start, end, tuple(lines[1:])))
    return out


def sentence_onsets(report):
    """explainer_kit narration::report_starts: running out_samples + gap_after_samples, over meta.rate."""
    rate = report.get("meta", {}).get("rate")
    need(isinstance(rate, (int, float)) and rate > 0, "report.json: no positive meta.rate")
    out = {}
    for cue, rows in report["lines"].items():
        t, starts = 0, []
        for k, s in enumerate(rows):
            need(isinstance(s.get("out_samples"), int) and isinstance(s.get("gap_after_samples"), int),
                 f"report.json: {cue} sentence {k} has no sample counts")
            starts.append(t / rate)
            t += s["out_samples"] + s["gap_after_samples"]
        out[cue] = starts
    return out


def wav_seconds(path):
    with wave.open(str(path)) as w:
        return w.getnframes() / w.getframerate()


# ---------------------------------------------------------------- the static checks

class Ctx:
    def __init__(self, a):
        for key in ("lane", "jobs", "crate", "narration", "integration", "binding", "tools"):
            value = Path(getattr(a, key))
            need(value.is_absolute(), f"--{key} must be an absolute path")
            setattr(self, key, value)
        need(re.fullmatch(r"\d\d", a.series or "") is not None, "--series must be two digits")
        self.series = a.series
        base = f"{STAGE}-master{a.series}"
        self.master_job = getattr(a, "master_job", None) or base  # LavenderPike's driver can name a retry, as for a render
        need(re.fullmatch(re.escape(base) + RENDER_ATTEMPT, self.master_job) is not None, f"--master-job {self.master_job}: only {base} or its -r2 to -r9 attempts")
        self.timeline_job = f"{STAGE}-timeline{a.series}"
        self.score_job = f"{STAGE}-score{a.series}"
        self.build_job = f"{STAGE}-build{a.series}"
        self.assemble_job = f"{STAGE}-assemble{a.series}"
        self.work = self.lane / f"{STAGE}-render-work{a.series}"
        self.verify_job = None
        if getattr(a, "verify", None):
            need(re.fullmatch(r"\d\d", a.verify) is not None, "--verify must be two digits")
            self.verify_job = f"{STAGE}-verify{a.verify}"
        self.job_dir = Path(a.job_dir) if getattr(a, "job_dir", None) else None
        need(self.job_dir is None or self.job_dir.is_absolute(), "--job-dir must be an absolute path")


def read_config(script):
    match = re.search(r"^CONFIG = json\.loads\((.*)\)$", Path(script).read_text(), re.M)
    need(match is not None, f"{script} has no CONFIG line")
    if match.group(1) == CONFIG_SIDECAR:
        return load(Path(script).with_suffix(".config.json"))
    return json.loads(ast.literal_eval(match.group(1)))


def same(a, b):
    return Path(a).resolve() == Path(b).resolve()


def bind_master(c, cues):
    """The master job ran the bound command through the guard, and nothing it bound has moved since."""
    script, inputs = c.jobs / f"{c.master_job}.py", c.jobs / f"{c.master_job}-inputs.json"
    folder = c.lane / c.master_job
    r = load(folder / "resource-receipt.json")
    need(r.get("phase") == "native" and ran(r, script), f"{c.master_job}: the guard's receipt ran {r.get('command')}, not {script}")
    need(receipt_ok(r), f"{c.master_job} did not pass: exit {r.get('exit')}, error {r.get('error')}")
    need(inputs.is_file() and r["inputs_sha256"] == sha(inputs), f"{c.master_job}: the guard's input hash is not {inputs.name}")
    rows = json.loads(inputs.read_text())["files"]
    # render-master01.py freeze() sorts its rows by path, so the script is found by name, not by being first
    own = [row for row in rows if row["path"] == str(script)]
    need(len(own) == 1 and own[0]["sha256"] == sha(script), f"{c.master_job}: the script moved")
    moved = [row["path"] for row in rows if not Path(row["path"]).is_file() or sha(row["path"]) != row["sha256"]]
    need(not moved, f"{c.master_job}: {len(moved)} bound inputs moved since it ran, first {moved[:2]}")
    bound = {Path(row["path"]).resolve() for row in rows}
    need(script.with_suffix(".config.json").resolve() in bound, f"{c.master_job}: the guard did not bind the script's config")
    cfg = read_config(script)
    need(cfg.get("step") == "master" and cfg.get("job") == c.master_job and same(cfg.get("lane"), c.lane),
         f"{c.master_job}: the script is another step, job or lane")
    need(same(cfg["narration"], c.narration), f"{c.master_job}: the master used narration {cfg['narration']}")
    need(same(cfg["render_work"], c.work), f"{c.master_job}: the master wrote {cfg['render_work']}, not {c.work}")
    need(cfg["chapters"] == CHAPTERS and cfg["cues"] == cues, f"{c.master_job}: the script's chapters or cues differ")
    need(cfg["timeline_job"] == c.timeline_job and list(cfg["render_jobs"]) == CHAPTERS,
         f"{c.master_job}: the script names another timeline or chapter set")
    binary = cfg["binary"]
    need(Path(binary).resolve() in bound and sha(binary) == cfg["binary_sha256"], f"{c.master_job}: the binary is not the bound one")
    n, work = cfg["narration"], cfg["render_work"]
    stems = str(Path(cfg["lane"]) / c.score_job / "scored/stems")
    need(cfg["argv"] == [binary, "master", "--narration", n, "--score", stems, "--out", work],
         f"{c.master_job}: the master command is {cfg['argv']}")
    need(cfg["captions_argv"] == [binary, "captions", "all", "--narration", n, "--out", work],
         f"{c.master_job}: the captions command is {cfg['captions_argv']}")
    need(last_line(folder / "command.log") == MASTER_SUCCESS, f"{c.master_job}: the log does not end with the master's success line")
    s = load(folder / "master-summary.json")
    outputs = {"master": c.work / f"{TOPIC}_master.mp4", "srt": c.work / f"{TOPIC}.en.srt", "vtt": c.work / f"{TOPIC}.en.vtt"}
    for key, path in outputs.items():
        need(same(s[key]["path"], path), f"{c.master_job}: master-summary names {s[key]['path']} for {key}")
        need(path.is_file() and sha(path) == s[key]["sha256"], f"{c.master_job}: the {key} changed since the master job")
    need(s["master"]["sha256"] != RETIRED_MASTER, f"{c.master_job}: this is the retired stage09 master")
    need(cfg["binary_sha256"] != RETIRED_BINARY, f"{c.master_job}: this ran the retired stage09 binary")
    renders, render_receipts = {}, {}
    for ch in CHAPTERS:
        job = cfg["render_jobs"][ch]
        need(re.fullmatch(re.escape(f"{STAGE}-render-{ch}-{c.series}") + RENDER_ATTEMPT, job) is not None, f"{ch}: render job {job}")
        rr = load(c.lane / job / "resource-receipt.json")
        need(receipt_ok(rr) and ran(rr, c.jobs / f"{job}.py"), f"{job} did not pass")
        need(last_line(c.lane / job / "command.log") == RENDER_SUCCESS, f"{job}: the log does not end with the render's success line")
        rs = load(c.lane / job / "render-summary.json")
        need(rs["chapter"] == ch and sha(c.work / f"{ch}.mp4") == rs["mp4_sha256"], f"{job}: {ch}.mp4 is not the file it rendered")
        for ext in ("mp4", "timeline.tsv", "layout.tsv"):
            need((c.work / f"{ch}.{ext}").resolve() in bound, f"{c.master_job} did not bind {ch}.{ext}")
        renders[ch] = {"job": job, "mp4_sha256": rs["mp4_sha256"]}
        render_receipts[job] = rr
    tr = load(c.lane / c.timeline_job / "resource-receipt.json")
    need(receipt_ok(tr) and last_line(c.lane / c.timeline_job / "command.log") == TIMELINE_SUCCESS, f"{c.timeline_job} did not pass")
    sr = load(c.lane / c.score_job / "resource-receipt.json")
    need(receipt_ok(sr) and last_line(c.lane / c.score_job / "command.log") == SCORE_SUCCESS, f"{c.score_job} did not pass")
    br, ar = load(c.lane / c.build_job / "resource-receipt.json"), load(c.lane / c.assemble_job / "resource-receipt.json")
    need(receipt_ok(br), f"{c.build_job} did not pass")
    need(receipt_ok(ar), f"{c.assemble_job} did not pass")
    # The rebind between the build and the assembly is check_rebind's: only the binding says which job it was.
    order = [(c.build_job, br, c.assemble_job, ar), (c.assemble_job, ar, c.timeline_job, tr), (c.timeline_job, tr, c.score_job, sr)]
    order += [(c.score_job, sr, job, rr) for job, rr in render_receipts.items()]
    order += [(job, rr, c.master_job, r) for job, rr in render_receipts.items()]
    for early, done, late, began in order:
        need(stamp(done, "finished") <= stamp(began, "started"), f"{late} started before {early} finished")
    return {"cfg": cfg, "bound": bound, "outputs": outputs, "renders": renders, "build_receipt": br, "assemble_receipt": ar,
            "master_job": {"script_sha256": sha(script), "inputs_sha256": r["inputs_sha256"], "binary_sha256": cfg["binary_sha256"]}}


def check_binding(c, m):
    """The binding root's timeline and score ran under names this binary, narration, map and integration record, the
    crate has not moved since it was bound, and the build's own manifest is found by the hash in the build's receipt."""
    b = load(c.binding)
    cfg = m["cfg"]
    need(b.get("schema") == BINDING_SCHEMA, f"{c.binding.name}: not a {BINDING_SCHEMA}")
    need(b["approved"]["integration_sha256"] == sha(c.integration), f"{c.binding.name}: root approved another integration.json")
    need(b["build"]["job"] == c.build_job and same(b["build"]["binary"], cfg["binary"]) and b["build"]["binary_sha256"] == cfg["binary_sha256"],
         f"{c.binding.name}: the bound binary is not the one the master ran")
    need(same(b["audio"]["narration"], c.narration), f"{c.binding.name}: the bound narration is {b['audio']['narration']}")
    need(b["cues"] == CUES and b["chapters"] == len(CHAPTERS), f"{c.binding.name}: {b['chapters']} chapters and {b['cues']} cues")
    moved = [rel for rel, digest in b["crate"].items() if not (c.crate / rel).is_file() or sha(c.crate / rel) != digest]
    need(not moved, f"{len(moved)} crate files moved since the binding, first {moved[:2]}")
    manifests = [path for path, digest in b["files"].items() if digest == m["build_receipt"]["inputs_sha256"]]
    need(len(manifests) == 1, f"{c.binding.name}: no one bound file is the build's guard manifest")
    return b, {row["path"]: row["sha256"] for row in load(manifests[0])["files"]}


def check_rebind(c, m, binding):
    """The rebind this assembly used, by hash: the binding holds the assembly's guard manifest, that manifest holds one
    rebind receipt, and the master froze that same receipt. No job name is assumed, so a failed attempt can stay on the
    lane, unread. Every receipt gate applies to the job named, and its receipt must show the rebind helper's --apply."""
    ar, br = m["assemble_receipt"], m["build_receipt"]
    manifests = [path for path, digest in binding["files"].items() if digest == ar["inputs_sha256"]]
    need(len(manifests) == 1, f"{c.binding.name}: no one bound file is the assembly's guard manifest")
    manifest = Path(manifests[0])
    need(manifest.is_file() and sha(manifest) == ar["inputs_sha256"], f"{manifest.name} is not the manifest {c.assemble_job}'s guard froze")
    need(manifest.resolve() in m["bound"], f"{c.master_job}: the guard did not bind {manifest.name}")
    rows = [(Path(row["path"]), row["sha256"]) for row in load(manifest)["files"]
            if Path(row["path"]).name == "resource-receipt.json" and re.fullmatch(REBIND_JOB, Path(row["path"]).parent.name) is not None]
    need(len(rows) == 1, f"{manifest.name} names {len(rows)} rebind receipts, not one")
    path, digest = rows[0]
    job = path.parent.name
    need(same(path.parent.parent, c.lane), f"{job}: its receipt is not on this lane: {path}")
    need(path.is_file() and sha(path) == digest, f"{job}: its receipt is not the one {c.assemble_job} froze")
    r = load(path)
    need(receipt_ok(r), f"{job} did not pass: exit {r.get('exit')}, error {r.get('error')}")
    command = r.get("command")
    need(r.get("phase") == "native" and isinstance(command, list) and len(command) == 5 and command[1:3] == ["-I", "-B"]
         and re.fullmatch(r"python3(\.\d+)?", Path(command[0]).name) is not None and Path(command[3]).name == REBIND_TOOL and command[4] == "--apply",
         f"{job}: the guard's receipt ran {command}, not {REBIND_TOOL} --apply")
    for early, done, late, began in ((c.build_job, br, job, r), (job, r, c.assemble_job, ar)):
        need(stamp(done, "finished") <= stamp(began, "started"), f"{late} started before {early} finished")
    need(path.resolve() in m["bound"], f"{c.master_job}: the guard did not bind {job}'s receipt")
    return {"job": job, "receipt": path, "receipt_sha256": digest, "manifest": manifest}


def check_comparisons(c):
    """The stage09 comparisons, a second lock after the master's: a hard failure refuses here too. A move other than
    the predicted one is reported for root, never judged here."""
    found = {}
    for step, name in COMPARISONS.items():
        path, doc = c.binding.parent / name, load(c.binding.parent / name)
        need(doc.get("hard_failures") == [], f"{name}: hard failures {doc.get('hard_failures')}")
        found[step] = {"sha256": sha(path), "not_as_predicted": doc.get("not_as_predicted", [])}
    return found


def check_sources(c, m, built):
    """integration.json's changed rows are the repaired film: each leaves its stage09 hash, is on disk at its new hash,
    is bound by the master and was compiled by the build. Extra changed rows are allowed and reported: a compile repair
    can add one. New hashes are read from the record, so a compile repair needs no change here."""
    doc = load(c.integration)
    need(doc.get("approved_for_guarded_build") is True and doc.get("audio_changed") is False,
         "integration.json does not approve this source with unchanged audio")
    digest = sha(c.crate / "script.tsv")
    need(digest == SCRIPT_SHA256, f"script.tsv is {digest[:12]}, not the recorded narration's script")
    need(doc.get("script_sha256") == digest, "integration.json names another script")
    workspace = Path(doc["source_workspace"])
    rows = {row["relative_path"]: row for row in doc["files"]}
    changed = doc["changed_files"]
    need(len(set(changed)) == len(changed) and all(rel in rows for rel in changed), "integration.json's changed rows are not its files")
    absent = sorted(set(STAGE09_SOURCES) - {Path(rel).name for rel in changed})
    need(not absent, f"integration.json does not change {absent}")
    sources = []
    for rel in changed:
        row, path = rows[rel], workspace / rel
        need(row["sha256"] != row["before_sha256"], f"{rel}: integration.json records no change")
        need(STAGE09_SOURCES.get(path.name, row["before_sha256"]) == row["before_sha256"], f"{rel}: not changed from the rendered stage09 source")
        need(path.is_file() and sha(path) == row["sha256"], f"{rel}: not the integrated file")
        need(path.resolve() in m["bound"], f"{c.master_job}: the guard did not bind {rel}")
        need(built.get(str(path)) == row["sha256"], f"{c.build_job}: the build did not compile {rel} at its integrated hash")
        sources.append({"path": rel, "before_sha256": row["before_sha256"], "sha256": row["sha256"]})
    return doc, sources


def check_audio(c, m, script):
    """The assembly's own record: the retained recordings, byte for byte, in the bound narration."""
    path = c.lane / c.assemble_job / "audio-comparison.json"
    need(path.resolve() in m["bound"], f"{c.master_job}: the guard did not bind {path.name}")
    a = load(path)
    need(a.get("cues") == CUES and a.get("all_full_wavs_byte_identical") is True and a.get("all_phrase_timings_byte_identical") is True,
         f"{c.assemble_job} does not record byte-identical audio")
    differ = [cue for cue in script if a["wav_sha256"].get(cue) != sha(c.narration / f"{cue}.wav")]
    need(not differ, f"{len(differ)} WAVs differ from the assembly's record, first {differ[:3]}")
    need(a.get("phrase_timings_sha256") == sha(c.narration / "phrase-timings.json"), "phrase-timings.json differs from the assembly's record")


def check_map(c, script, report, binding):
    """The direction map the narration binds, and all its rows against the current source. The master need not bind
    the map itself: it binds report.json, and report.json binds the map by hash."""
    path, workspace = c.crate / DIRECTION_MAP, c.crate.parents[1]
    need(path.is_file(), f"{DIRECTION_MAP} is missing")
    digest = sha(path)
    meta = report["meta"]
    contract = meta["direction_contract"]["binding"]["contract"]["map"]
    need(contract["sha256"] == digest and Path(contract["path"]).name == path.name,
         f"report.json binds the map {contract['path']} {contract['sha256'][:12]}, not {path.name} {digest[:12]}")
    need(binding["direction"]["map_sha256"] == digest and binding["crate"].get(DIRECTION_MAP) == digest, f"{c.binding.name}: another direction map")
    need(meta["script_sha256"] == sha(c.crate / "script.tsv"), "report.json was assembled from another script")
    need(meta["units_sha256"] == sha(c.crate / "narration_units.json"), "report.json was assembled from another unit plan")
    holds = {}
    for h in load(path)["visual_holds"]:
        need(h["cue"] in script and h["cue"] not in holds, f"hold map: {h['cue']}")
        holds[h["cue"]] = h
    kinds = dict(Counter(h["kind"] for h in holds.values()))
    need(kinds == HOLD_KINDS, f"hold map: {kinds}, not {HOLD_KINDS}")
    stale = [h["cue"] for h in holds.values() if not (workspace / h["path"]).is_file() or sha(workspace / h["path"]) != h["sha256"]]
    need(not stale, f"hold map: stale source hash for {stale}")
    return holds, digest


def frames_covering(seconds):
    """narration.rs frames_covering, at 60 fps: the kit's own rounding of a hold's length."""
    return max(0, math.ceil(seconds * FPS - 1e-6))


def hold_windows(chapters, placements, timelines, holds):
    """Where each hold really fell, from the timelines' rows and nothing assumed. Every hold has one breath between its
    cue and the next. A prediction also has its `predict` mark on that frame, where draw.rs adds the bar and the pause
    line, then the think, then a fade of the bar and line, then the next cue. A reading hold runs straight into it."""
    first = {x["chapter"]: x for x in chapters}
    sites, used = {}, set()
    for i, p in enumerate(placements):
        cue, ch = p["cue"], p["chapter"]
        if cue not in holds:
            continue
        t, base, kind = timelines[ch], first[ch]["start_frame"], holds[cue]["kind"]
        start = p["start_frame"] - base
        nxt = placements[i + 1] if i + 1 < len(placements) else None
        until = (nxt["start_frame"] - base) if nxt and nxt["chapter"] == ch else t["frames"]
        breaths = [b for b in t["breaths"] if start < b["frame"] < until]
        marks = [m for m in t["marks"] if start <= m["frame"] < until]
        need(len(breaths) == 1, f"{cue}: {len(breaths)} breath rows between the cue and the next")
        b = breaths[0]
        think_end = b["frame"] + frames_covering(b["seconds"])
        if kind == "prediction":
            need([m["label"] for m in marks] == ["predict"] and marks[0]["frame"] == b["frame"],
                 f"{cue}: the predict mark is not one mark on the hold's breath: {marks}")
            need(until - think_end == PREDICT_FADE_FRAMES, f"{cue}: {until - think_end} frames from the think's end to the next cue")
        else:
            need(not marks and until == think_end, f"{cue}: the reading hold has marks {marks} or ends {until - think_end} frames early")
        used.add((ch, b["frame"]))
        sites[cue] = {"cue": cue, "kind": kind, "chapter": ch, "next_cue": holds[cue]["next_cue"], "seconds": b["seconds"],
                      "start_frame": base + b["frame"], "think_end_frame": base + think_end, "next_frame": base + until}
    stray = [(ch, b["frame"]) for ch, t in timelines.items() for b in t["breaths"] if (ch, b["frame"]) not in used]
    return sites, stray


def closing_window(chapters, placements, timelines, doc, stray):
    """The editorial closing after the last word: the one breath no hold row claims, at the length integration.json
    records, followed by exactly the recorded fade. No map row, and native timing was never measured before this."""
    last, info = placements[-1], chapters[-1]
    ch = info["chapter"]
    need(doc.get("cue") == last["cue"] and last["chapter"] == ch, f"editorial_closing_hold names {doc.get('cue')}, not the film's last cue")
    need(doc.get("after_complete_recording") is True and doc.get("prediction_or_reading_direction_row") is False,
         "editorial_closing_hold is not the closing the map leaves out")
    need(len(stray) == 1 and stray[0][0] == ch, f"expected one closing breath in {ch}, found {stray}")
    t = timelines[ch]
    b = next(x for x in t["breaths"] if x["frame"] == stray[0][1])
    voice_end = last["start_frame"] - info["start_frame"] + round(last["duration_s"] * FPS)
    need(b["frame"] >= voice_end, f"the closing breath at {b['frame']} starts before the last word ends at {voice_end}")
    need(abs(b["seconds"] - doc["hold_seconds"]) < 1e-6, f"the closing hold is {b['seconds']} s, integration.json says {doc['hold_seconds']} s")
    hold_end = b["frame"] + frames_covering(b["seconds"])
    fade = t["frames"] - hold_end
    need(fade == frames_covering(doc["fade_seconds"]), f"{fade} frames follow the closing hold, integration.json says {doc['fade_seconds']} s")
    base = info["start_frame"]
    return {"cue": last["cue"], "chapter": ch, "voice_end_frame": base + voice_end, "hold_start_frame": base + b["frame"],
            "hold_end_frame": base + hold_end, "end_frame": base + t["frames"], "hold_seconds": b["seconds"], "fade_seconds": doc["fade_seconds"],
            "hold_start_seconds": (base + b["frame"]) / FPS, "end_seconds": (base + t["frames"]) / FPS}


def anchor_places(chapters, placements, script, onsets, marks, sites, closing):
    """base -> (film frame, lowest, one past the highest) on this schedule. A negative offset may reach back to the cue
    before; a positive one may not pass the next cue. Everything comes from the arguments; nothing old is read."""
    info = {x["chapter"]: x for x in chapters}
    cues = {}
    for i, p in enumerate(placements):
        first, end = info[p["chapter"]]["start_frame"], info[p["chapter"]]["start_frame"] + info[p["chapter"]]["frames"]
        before = placements[i - 1] if i else None
        after = placements[i + 1] if i + 1 < len(placements) else None
        cues[p["cue"]] = {"start": p["start_frame"], "seconds": p["duration_s"],
                          "low": before["start_frame"] if before and before["chapter"] == p["chapter"] else first,
                          "high": after["start_frame"] if after and after["chapter"] == p["chapter"] else end}

    def cue_of(name, base):
        need(name in cues, f"anchor base {base}: no such cue")
        return cues[name]

    def place(base):
        kind, _, rest = base.partition(":")
        if kind in ("chapter_start", "chapter_end"):
            need(rest in info, f"anchor base {base}: no such chapter")
            first, end = info[rest]["start_frame"], info[rest]["start_frame"] + info[rest]["frames"]
            openers = [p["start_frame"] for p in placements if p["chapter"] == rest]
            need(openers, f"anchor base {base}: the chapter has no cues")
            return (first, first, openers[0]) if kind == "chapter_start" else (end, openers[-1], end)
        if kind == "closing":
            need(closing is not None and rest in ("hold_start", "hold_end", "end"), f"anchor base {base}: no such closing point")
            return closing[f"{rest}_frame"], closing["hold_start_frame"], closing["end_frame"]
        if kind == "hold":
            name, _, point = rest.partition(":")
            need(name in sites and point in ("start", "think_end", "next"), f"anchor base {base}: no such hold point")
            s = sites[name]
            low = cue_of(name, base)["start"]
            return ((s["next_frame"], low, s["next_frame"] + 1) if point == "next" else (s[f"{point}_frame"], low, s["next_frame"]))
        if kind == "cue":
            c = cue_of(rest, base)
            return c["start"], c["low"], c["high"]
        if kind == "voice_end":
            c = cue_of(rest, base)
            return c["start"] + round(c["seconds"] * FPS), c["low"], c["high"]
        if kind == "sentence":
            name, _, number = rest.rpartition(":")
            c = cue_of(name, base)
            need(number.isdigit() and 1 <= int(number) <= len(onsets[name]), f"anchor base {base}: {name} has {len(onsets[name])} sentences")
            return c["start"] + round(onsets[name][int(number) - 1] * FPS), c["low"], c["high"]
        if kind == "phrase":
            name, _, words = rest.partition(":")
            c, text = cue_of(name, base), script.get(name, "").encode()
            need(words and text.count(words.encode()) == 1, f"anchor base {base}: the words are not once in the script")
            byte = text.index(words.encode())
            hits = [m for m in marks.get(name, []) if m["byte"] == byte]
            need(len(hits) == 1, f"anchor base {base}: no phrase mark at byte {byte}")
            return c["start"] + round(hits[0]["seconds"] * FPS), c["low"], c["high"]
        raise Refused(f"anchor base {base}: unknown kind")

    return place


def resolve_anchors(rows, place, total_frames):
    """Every anchor on this schedule, or a refusal. The `was` field of a row is never read: it is for people and tests."""
    anchors = []
    for row in rows:
        at, low, high = place(row["base"])
        for offset in row["offsets"]:
            frame = at + offset
            need(0 <= frame < total_frames, f"anchor {row['id']} {offset:+d}: frame {frame} is outside the film")
            need(frame >= low if offset < 0 else frame < high, f"anchor {row['id']} {offset:+d}: frame {frame} left its cue window {low}..{high}")
            anchors.append({"id": row["id"], "finding": row["finding"], "base": row["base"], "offset": offset, "frame": frame,
                            "seconds": frame / FPS, "note": row["note"], "reason": f"{row['id']} {offset:+d} {row['note']}"})
    return anchors


def anchor_ledger(samples, anchors):
    """Where each anchor landed in the plan: the picture number and the sheet that shows it."""
    index = {s["frame"]: k for k, s in enumerate(samples, 1)}
    return [{key: a[key] for key in ("id", "finding", "base", "offset", "frame", "seconds", "note")}
            | {"picture": index[a["frame"]], "sheet": math.ceil(index[a["frame"]] / 4)} for a in anchors]


def read_schedule(c, script, summary, bound):
    need(summary.get("fps") == FPS and [x["chapter"] for x in summary["chapters"]] == CHAPTERS and summary["cues"] == len(script),
         f"{c.timeline_job}: the summary is not 13 chapters at 60 fps with {len(script)} cues")
    offset, chapters, placements, timelines = 0, [], [], {}
    for item in summary["chapters"]:
        ch = item["chapter"]
        path, reference = c.work / f"{ch}.timeline.tsv", c.lane / c.timeline_job / "timeline-output" / f"{ch}.timeline.tsv"
        need(reference.resolve() in bound, f"the master did not bind the timeline job's {ch}.timeline.tsv")
        t = parse_timeline(path.read_text(), path.name)
        need(schedule(t) == schedule(parse_timeline(reference.read_text(), reference.name)),
             f"{ch}: the 4K render's schedule differs from the 60 fps timeline")
        need(t["chapter"] == ch and t["fps"] == FPS and t["frames"] == item["frames"] and [x["id"] for x in t["lines"]] == item["cues"],
             f"{ch}: the timeline differs from the timeline summary")
        # root's asr01.py compares these hashes byte for byte: a difference has to stop here, not after this job
        need(sha(path) == sha(reference) == item["timeline_sha256"], f"{ch}: the render's, timeline job's and summary's timeline bytes differ")
        for line in t["lines"]:
            wav = c.narration / f"{line['id']}.wav"
            need(same(line["wav"], wav), f"{line['id']}: the line plays {line['wav']}, not the bound narration")
            seconds = wav_seconds(wav)
            need(line["start"] / FPS + seconds <= t["frames"] / FPS + 0.001, f"{line['id']}: runs past the end of {ch}")
            placements.append({"cue": line["id"], "chapter": ch, "start_frame": offset + line["start"], "duration_s": seconds})
        chapters.append({"chapter": ch, "start_frame": offset, "frames": t["frames"], "timeline_sha256": sha(path)})
        timelines[ch] = t
        offset += t["frames"]
    need(offset == summary["total_frames"], f"the chapters sum to {offset} frames, the timeline summary says {summary['total_frames']}")
    need([p["cue"] for p in placements] == list(script), "the cue placements differ from the script order")
    return chapters, placements, timelines, offset


def check_captions(srt_text, vtt_text, script, report, placements, total_frames):
    """SRT = VTT exactly; in order, inside the video; the text equals the script, as a whole and cue by cue."""
    srt, vtt = parse_captions(srt_text, False), parse_captions(vtt_text, True)
    if srt != vtt:
        k = next((i for i, (x, y) in enumerate(zip(srt, vtt)) if x != y), min(len(srt), len(vtt)))
        raise Refused(f"SRT and VTT differ at caption {k + 1} ({len(srt)} SRT, {len(vtt)} VTT captions)")
    limit = round(total_frames * 1000 / FPS) + 1
    previous = 0
    for k, (start, end, _lines) in enumerate(srt, 1):
        need(0 <= start < end <= limit, f"caption {k}: {start}-{end} ms is empty or outside the video's {limit} ms")
        need(start >= previous, f"caption {k} starts at {start} ms, before caption {k - 1} ends at {previous} ms")
        previous = end
    need(norm(" ".join(" ".join(x[2]) for x in srt)) == norm(" ".join(script.values())), "the captions' text differs from the script")
    starts = [(p["cue"], p["start_frame"] * 1000 / FPS) for p in placements]
    by_cue = {cue: [] for cue in script}
    for start, _end, lines in srt:
        earlier = [cue for cue, at in starts if at <= start + 1.0]  # the kit rounds caption times to the millisecond
        need(earlier, f"a caption at {start} ms starts before the first cue")
        by_cue[earlier[-1]].append(" ".join(lines))
    for cue, text in script.items():
        got = norm(" ".join(by_cue[cue]))
        need(got == norm(text), f"{cue}: the captions shown during this cue read {got[:70]!r}, the script {norm(text)[:70]!r}")
        need(norm(" ".join(s["sentence"] for s in report["lines"][cue])) == norm(text), f"{cue}: the narration report's sentences differ from the script")
    return {"caption_blocks": len(srt), "srt_equals_vtt": True, "text_equals_script": "whole and per cue",
            "last_caption_end_ms": previous, "video_ms": limit - 1}


def check_phrases(phrases, script, narration):
    """The installed phrase-timings.json (prepare_audio.py cmd_timings), checked by the kit's own load rule."""
    marks = {}
    for cue, entry in phrases["cues"].items():
        need(cue in script, f"phrase-timings.json names unknown cue {cue}")
        wav = narration / f"{cue}.wav"
        need(entry["wav_sha256"] == sha(wav) and entry["text_sha256"] == hashlib.sha256(script[cue].encode()).hexdigest(),
             f"{cue}: phrase timings were made for another WAV or text")
        seconds, prev_t, prev_b = wav_seconds(wav), 0.0, -1
        for m in entry["marks"]:
            need(0 <= m["seconds"] < seconds and m["seconds"] >= prev_t and m["byte"] > prev_b,
                 f"{cue}: the mark at byte {m['byte']} breaks the kit's order or range rule")
            prev_t, prev_b = m["seconds"], m["byte"]
        if entry["marks"]:
            marks[cue] = entry["marks"]
    return marks


def plan_samples(chapters, placements, timelines, script, onsets, holds, marks, layouts, anchors=()):
    """Root's sampling (opening, middle, closing, every 10 s, cue start/middle/end, sentences and half a second after,
    post-cue holds, layout findings) plus each hold's last frame, the timelines' marks and every phrase mark. Anchors
    go in first, so a sheet's caption names the anchor; every other reason at that frame stays after it."""
    samples = {}
    for a in anchors:
        samples.setdefault(a["frame"], []).append(a["reason"])
    by_chapter = {x["chapter"]: x for x in chapters}

    def add(ch, local, reason):
        info = by_chapter[ch]
        if 0 <= local < info["frames"]:
            samples.setdefault(info["start_frame"] + local, []).append(reason)

    order = [p["cue"] for p in placements]
    for x in chapters:
        ch, frames = x["chapter"], x["frames"]
        for local, label in ((min(180, frames - 1), "opening"), (frames // 2, "middle"), (max(0, frames - 120), "closing")):
            add(ch, local, f"{ch} {label}")
        for local in range(0, frames, 600):
            add(ch, local, f"{ch} periodic 10s")
        for m in timelines[ch]["marks"]:
            add(ch, m["frame"], f"{ch} mark {m['label']}")
            add(ch, m["frame"] + 30, f"{ch} mark {m['label']} plus half second")
        anchors = layouts.get(ch, [])
        for anchor, row in anchors:
            for delta in (-12, 0, 12, 60, 120):
                add(ch, anchor + delta, f"{ch} layout {row}")
        if anchors:
            for local in range(min(a for a, _ in anchors) + 300, frames, 300):
                add(ch, local, f"{ch} layout later sample")
    for i, p in enumerate(placements):
        ch, cue = p["chapter"], p["cue"]
        start = p["start_frame"] - by_chapter[ch]["start_frame"]
        length = round(p["duration_s"] * FPS)
        for delta, label in ((0, "cue start"), (round(p["duration_s"] * 30), "cue middle"), (max(0, length - 12), "cue end")):
            add(ch, start + delta, f"{cue} {label}")
        for k, at in enumerate(onsets[cue], 1):
            add(ch, start + round(at * FPS), f"{cue} sentence {k}")
            add(ch, start + round(at * FPS) + 30, f"{cue} sentence {k} plus half second")
        for m in marks.get(cue, []):
            words = " ".join(script[cue].encode()[m["byte"]:].decode(errors="replace").split()[:4])
            for delta, label in ((0, "at"), (30, "plus 0.5 s"), (60, "plus 1 s"), (120, "plus 2 s")):
                add(ch, start + round(m["seconds"] * FPS) + delta, f"{cue} phrase byte {m['byte']} '{words}' {label}")
        if cue in holds:
            hold = holds[cue]
            nxt = placements[i + 1] if i + 1 < len(placements) else None
            need(hold["next_cue"] == (nxt["cue"] if nxt else None), f"{cue}: the hold map's next cue {hold['next_cue']} is not the film's")
            until = (nxt["start_frame"] - by_chapter[ch]["start_frame"]) if nxt and nxt["chapter"] == ch else by_chapter[ch]["frames"]
            for delta in (12, 60, 120):
                if start + length + delta < until:
                    add(ch, start + length + delta, f"{cue} post-cue {hold['kind']} hold")
            add(ch, until - 12, f"{cue} {hold['kind']} hold, last moment before {hold['next_cue']}")
    need(order == list(script), "placements are out of script order")
    return [{"frame": f, "seconds": f / FPS, "reasons": r} for f, r in sorted(samples.items())]


def gather(c):
    on_host()
    tools = load(c.tools)
    check_tools(tools, c.tools.name)
    check_tool_path(tools)
    for rel, digest in PINS.items():
        need(sha(REPO / rel) == digest, f"root's {rel} is not the pinned {digest[:12]}")
    need(registry(c.crate) == CHAPTERS, "the crate's chapter registry is not the 13 chapters")
    script = read_script(c.crate / "script.tsv")
    need(len(script) == CUES, f"the script has {len(script)} cues, not {CUES}")
    m = bind_master(c, list(script))
    for rel in ("script.tsv", "src/chapters/mod.rs", "narration_units.json", DIRECTION_MAP):
        need((c.crate / rel).resolve() in m["bound"], f"the master did not bind the crate's {rel}")
    need(c.integration.resolve() in m["bound"], "the master did not bind integration.json")
    need(m["cfg"].get("tools_sha256") == sha(c.tools), f"{c.master_job} ran under another tools file: the master and this check must name one ffmpeg")
    for name in ("report.json", "phrase-timings.json"):
        need((c.narration / name).resolve() in m["bound"], f"the master did not bind narration {name}")
    binding, built = check_binding(c, m)
    rebind = check_rebind(c, m, binding)
    integration, sources = check_sources(c, m, built)
    check_audio(c, m, script)
    comparisons = check_comparisons(c)
    summary_path = c.lane / c.timeline_job / "timeline-summary.json"
    need(summary_path.resolve() in m["bound"], "the master did not bind the timeline summary")
    chapters, placements, timelines, total = read_schedule(c, script, load(summary_path), m["bound"])
    report = load(c.narration / "report.json")
    # The canonical assembler writes report.json with sorted keys (narration_cache.encoded), and the kit reads its
    # lines as a map: compare the cue set. The order is checked on the timelines' placements (read_schedule).
    need(sorted(report["lines"]) == sorted(script), "report.json's cues differ from the script")
    captions = check_captions(m["outputs"]["srt"].read_text(encoding="utf-8"), m["outputs"]["vtt"].read_text(encoding="utf-8"),
                              script, report, placements, total)
    holds, map_sha256 = check_map(c, script, report, binding)
    marks = check_phrases(load(c.narration / "phrase-timings.json"), script, c.narration)
    layouts = {ch: layout_anchors((c.work / f"{ch}.layout.tsv").read_text(), f"{ch}.layout.tsv") for ch in CHAPTERS}
    onsets = sentence_onsets(report)
    sites, stray = hold_windows(chapters, placements, timelines, holds)
    closing = closing_window(chapters, placements, timelines, integration.get("editorial_closing_hold", {}), stray)
    place = anchor_places(chapters, placements, script, onsets, marks, sites, closing)
    anchors = resolve_anchors(load(ANCHORS)["rows"], place, total)
    samples = plan_samples(chapters, placements, timelines, script, onsets, holds, marks, layouts, anchors)
    return {"script": script, "master": m, "chapters": chapters, "placements": placements, "total_frames": total,
            "captions": captions, "holds": len(holds), "phrase_marks": sum(len(v) for v in marks.values()),
            "layout_findings": {ch: len(v) for ch, v in layouts.items()}, "samples": samples,
            "anchors": anchor_ledger(samples, anchors), "pause_sites": list(sites.values()), "closing": closing, "sources": sources,
            "integration_sha256": sha(c.integration), "binding_sha256": sha(c.binding), "map_sha256": map_sha256, "tools": tools,
            "tools_sha256": sha(c.tools), "comparisons": comparisons, "rebind": rebind}


def check(c):
    f = gather(c)
    return {"master": str(f["master"]["outputs"]["master"]), "master_sha256": sha(f["master"]["outputs"]["master"]),
            "master_job": f["master"]["master_job"], "rebind_job": f["rebind"]["job"], "chapters": len(f["chapters"]), "cues": len(f["placements"]),
            "total_frames": f["total_frames"], "seconds": round(f["total_frames"] / FPS, 3), "captions": f["captions"],
            "holds": f["holds"], "phrase_marks": f["phrase_marks"], "layout_findings": f["layout_findings"],
            "source_changes": f["sources"], "integration_sha256": f["integration_sha256"], "binding_sha256": f["binding_sha256"],
            "direction_map_sha256": f["map_sha256"], "pause_sites": f["pause_sites"], "closing": f["closing"], "anchors": len(f["anchors"]),
            "anchor_rows": len({a["id"] for a in f["anchors"]}), "pictures_planned": len(f["samples"]),
            "sheets_planned": math.ceil(len(f["samples"]) / 4), "tools": f["tools"], "tools_sha256": f["tools_sha256"], "comparisons": f["comparisons"], "not_claimed": SCOPE}


# ---------------------------------------------------------------- the guarded job

JOB_TEMPLATE = r'''#!/usr/bin/env python3
"""@@JOB@@: final technical verification of the repaired logarithm master. Written by verify_remote_final.py
(sha256 @@HELPER@@). Launch only through root's guard from the repository root; the guard binds every input.
Technical checks and extracted pictures only: no listening, picture viewing, fine sync or acceptance is claimed.
"""
import hashlib
import importlib.util
import json
import sys

CONFIG = json.loads(@@CONFIG@@)


def main():
    with open(CONFIG["helper"], "rb") as handle:
        if hashlib.file_digest(handle, "sha256").hexdigest() != CONFIG["helper_sha256"]:
            print("REFUSED: the verification helper moved", flush=True)
            return 1
    spec = importlib.util.spec_from_file_location("verify_remote_final", CONFIG["helper"])
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    return helper.run_job(CONFIG)


if __name__ == "__main__":
    sys.exit(main())
'''


def write_job(c):
    need(c.verify_job and c.job_dir, "write-job needs --verify NN and --job-dir D")
    f = gather(c)
    job = c.verify_job
    script, inputs = c.job_dir / f"{job}.py", c.job_dir / f"{job}-inputs.json"
    for path in (script, inputs, c.lane / job):
        need(not path.exists(), f"{path} exists: use a new --verify number")
    out = f["master"]["outputs"]
    cfg = {"job": job, "lane": str(c.lane), "script": str(script), "helper": str(HERE), "helper_sha256": sha(HERE),
           "master": {"path": str(out["master"]), "sha256": sha(out["master"])},
           "srt": {"path": str(out["srt"]), "sha256": sha(out["srt"])}, "vtt": {"path": str(out["vtt"]), "sha256": sha(out["vtt"])},
           "master_job": c.master_job, "series": c.series, "total_frames": f["total_frames"], "fps": FPS,
           "chapters": f["chapters"], "placements": f["placements"], "captions": f["captions"],
           "strict": str(REPO / STRICT_REL), "screen": str(REPO / SCREEN_REL), "ffmpeg": f["tools"]["ffmpeg"]["path"], "ffprobe": f["tools"]["ffprobe"]["path"],
           "expect": EXPECT, "samples": f["samples"], "success": VERIFY_SUCCESS, "scope": SCOPE,
           "anchors": f["anchors"], "pause_sites": f["pause_sites"], "closing": f["closing"], "source_changes": f["sources"],
           "integration_sha256": f["integration_sha256"], "binding_sha256": f["binding_sha256"], "tools": f["tools"],
           "tools_sha256": f["tools_sha256"], "comparisons": f["comparisons"],
           "rebind_job": f["rebind"]["job"], "rebind_receipt_sha256": f["rebind"]["receipt_sha256"]}
    c.job_dir.mkdir(parents=True, exist_ok=True)
    text = JOB_TEMPLATE.replace("@@JOB@@", job).replace("@@HELPER@@", cfg["helper_sha256"])
    with open(script, "x") as handle:
        handle.write(text.replace("@@CONFIG@@", repr(json.dumps(cfg, indent=1))))
    paths = [script, HERE] + [REPO / rel for rel in PINS] + [Path(cfg["ffmpeg"]), Path(cfg["ffprobe"]), Path(f["tools"]["python3"]["path"]), c.tools]
    paths += [out["master"], out["srt"], out["vtt"], c.jobs / f"{c.master_job}.py", c.jobs / f"{c.master_job}-inputs.json"]
    paths += [c.lane / c.master_job / name for name in ("resource-receipt.json", "command.log", "master-summary.json")]
    paths += [c.lane / c.timeline_job / "timeline-summary.json"]
    paths += [c.work / f"{ch}.{ext}" for ch in CHAPTERS for ext in ("timeline.tsv", "layout.tsv")]
    paths += [c.lane / c.timeline_job / "timeline-output" / f"{ch}.timeline.tsv" for ch in CHAPTERS]
    paths += [c.narration / name for name in ("report.json", "phrase-timings.json")] + [c.narration / f"{cue}.wav" for cue in f["script"]]
    paths += [c.crate / rel for rel in ("script.tsv", DIRECTION_MAP, "src/chapters/mod.rs", "narration_units.json")]
    paths += [c.jobs / f"{c.master_job}.config.json", c.integration, c.binding, ANCHORS]
    paths += [c.lane / c.assemble_job / "audio-comparison.json"] + [c.binding.parent / name for name in COMPARISONS.values()]
    paths += [f["rebind"]["manifest"], f["rebind"]["receipt"]]
    have = {p.resolve() for p in paths}
    paths += [c.crate.parents[1] / row["path"] for row in f["sources"] if (c.crate.parents[1] / row["path"]).resolve() not in have]
    need(len({p.resolve() for p in paths}) == len(paths), "an input row repeats")
    with open(inputs, "x") as handle:
        handle.write(json.dumps({"files": [{"path": str(p), "sha256": sha(p)} for p in paths]}, indent=2) + "\n")
    python = f["tools"]["python3"]["path"]
    return {"job": job, "script": str(script), "script_sha256": sha(script), "inputs": str(inputs), "inputs_sha256": sha(inputs),
            "input_rows": len(paths), "pictures_planned": len(f["samples"]), "sheets_planned": math.ceil(len(f["samples"]) / 4),
            "launch_from_repo_root_on_worker": f"{python} -I -B {REMOTE_GUARD_REL} --phase native --job {job} --inputs {inputs} -- {python} -I -B {script}"}


def media_facts(probe, video_packets, total_frames):
    """The master's streams, its frame count against the chapters' timelines, and the kit's A/V length gate."""
    video = [s for s in probe["streams"] if s["codec_type"] == "video" and not s.get("disposition", {}).get("attached_pic")]
    audio = [s for s in probe["streams"] if s["codec_type"] == "audio"]
    need(len(video) == 1 and len(audio) == 1 and len(probe["streams"]) == 2, "the master is not one video and one audio stream")
    v, a = video[0], audio[0]
    need((v["codec_name"], v["width"], v["height"], v["r_frame_rate"]) == ("h264", 3840, 2160, "60/1"),
         f"video is {v['codec_name']} {v['width']}x{v['height']} at {v['r_frame_rate']}")
    need((a["codec_name"], int(a["sample_rate"]), a["channels"]) == ("aac", 48000, 2),
         f"audio is {a['codec_name']} {a['sample_rate']} Hz, {a['channels']} channels")
    need(video_packets == total_frames, f"the master has {video_packets} video frames; the chapters' timelines sum to {total_frames}")
    picture = total_frames / FPS
    need(abs(float(v["duration"]) - picture) <= 1 / FPS, f"the video lasts {v['duration']} s, the timelines {picture:.6f} s")
    need(abs(float(a["duration"]) - picture) <= 1024 / 48000 + 1e-3,
         f"the audio lasts {a['duration']} s, the picture {picture:.6f} s (the kit allows one AAC frame)")
    return {"video": "h264 3840x2160 60/1", "audio": "aac 48000 Hz stereo", "video_frames": video_packets,
            "video_seconds": float(v["duration"]), "audio_seconds": float(a["duration"]), "picture_seconds": picture}


def loudness_summary(text):
    tail = text[text.rfind("Summary:"):] if "Summary:" in text else ""
    found = {key: re.search(rf"{key}:\s+(-?[0-9.]+|-inf)", tail) for key in ("I", "LRA", "Peak")}
    need(all(found.values()), "the ebur128 log has no Summary with I, LRA and true peak")
    return {"integrated_lufs": found["I"].group(1), "lra_lu": found["LRA"].group(1), "true_peak_dbtp": found["Peak"].group(1)}


def make_sheets(pictures, job):
    from PIL import Image, ImageDraw
    sheets = []
    for start in range(0, len(pictures), 4):
        batch = pictures[start:start + 4]
        sheet = Image.new("RGB", (1920, 580 * math.ceil(len(batch) / 2)), "#111827")
        draw = ImageDraw.Draw(sheet)
        for i, p in enumerate(batch):
            x, y = (i % 2) * 960, (i // 2) * 580
            with Image.open(p["path"]) as img:
                sheet.paste(img, (x, y))
            more = f" (+{len(p['reasons']) - 1})" if len(p["reasons"]) > 1 else ""
            draw.text((x + 10, y + 548), f"{p['seconds']:.2f}s f{p['frame']} | {p['reasons'][0][:78]}{more}", fill="white", font_size=17)
        path = job / f"sheet-{start // 4 + 1:03d}.jpg"
        with path.open("xb") as handle:  # never replaces a sheet
            sheet.save(handle, format="JPEG", quality=92)
        sheets.append(str(path))
    return sheets


def run_job(cfg):
    job = Path(cfg["lane"]) / cfg["job"]
    try:
        receipt = job / "resource-receipt.json"
        need(receipt.is_file(), f"launch only through the guard: {receipt} is missing")
        started = json.loads(receipt.read_text())
        need(ran(started, cfg["script"]) and "exit" not in started and started.get("admitted") is True,
             "the guard's receipt for this job does not show this script running")
        on_host()
        check_tools(cfg["tools"], "the job's tools")
        tool_dirs = check_tool_path(cfg["tools"])
        master = cfg["master"]["path"]
        for key in ("master", "srt", "vtt"):
            need(sha(cfg[key]["path"]) == cfg[key]["sha256"], f"the {key} moved since the job was written")
        for name in OUTPUTS:
            need(not (job / name).exists(), f"{name} exists already")
        for old in sorted(job.glob("frame-*")) + sorted(job.glob("sheet-*")):
            need(False, f"{old.name} exists already")  # ffmpeg -n would keep it and still exit 0
        (job / "decode-logs").mkdir()
        env = dict(os.environ, **cfg["expect"], PATH=tool_dirs, TMPDIR=str(job / "decode-logs"))  # root's strict check keeps its decode logs here
        with (job / "strict-video-verification.log").open("x") as log:
            done = subprocess.run(["bash", cfg["strict"], master], stdout=log, stderr=subprocess.STDOUT, env=env, timeout=14400)
        need(done.returncode == 0 and last_line(job / "strict-video-verification.log") == "VERIFY PASS", "root's strict video check failed")
        strict_text = (job / "strict-video-verification.log").read_text(errors="replace")
        notes = [row for row in strict_text.splitlines() if row.startswith("note:")]
        # root's script keeps one decode log, rewritten by its second decode: its summary lines are the record of both
        decodes = decode_lines(strict_text)
        need(software_decode_clean(decodes), "the strict log does not record a clean software decode")
        probe = json.loads(subprocess.run([cfg["ffprobe"], "-v", "error", "-show_streams", "-show_format", "-of", "json", master],
                                          check=True, capture_output=True, text=True, timeout=600).stdout)
        packets = subprocess.run([cfg["ffprobe"], "-v", "error", "-select_streams", "V:0", "-count_packets", "-show_entries",
                                  "stream=nb_read_packets", "-of", "csv=p=0", master], check=True, capture_output=True, text=True, timeout=3600)
        media = media_facts(probe, int(packets.stdout.strip()), cfg["total_frames"])
        with (job / "encoded-loudness.log").open("x") as log:
            subprocess.run([cfg["ffmpeg"], "-nostdin", "-hide_banner", "-nostats", "-i", master, "-vn", "-af", "ebur128=peak=true",
                            "-f", "null", "-"], stdout=log, stderr=subprocess.STDOUT, check=True, timeout=14400)
        loud = loudness_summary((job / "encoded-loudness.log").read_text(errors="replace"))
        subprocess.run([sys.executable, "-I", "-B", cfg["screen"], cfg["srt"]["path"], str(cfg["total_frames"]), str(FPS),
                        str(job / "caption-screen.json")], check=True, capture_output=True, text=True, timeout=600)
        screen = json.loads((job / "caption-screen.json").read_text())
        pictures = []
        for i, s in enumerate(cfg["samples"], 1):
            path = job / f"frame-{i:04d}.png"
            need(not path.exists(), f"{path.name} exists already")
            subprocess.run([cfg["ffmpeg"], "-nostdin", "-n", "-v", "error", "-xerror", "-ss", f"{s['frame'] / FPS:.9f}", "-i", master,
                            "-frames:v", "1", "-vf", "scale=960:540", str(path)], check=True, timeout=180)
            need(path.is_file() and path.stat().st_size > 0, f"ffmpeg wrote no picture for frame {s['frame']}")
            pictures.append({**s, "path": str(path), "sha256": sha(path)})
        sheets = make_sheets(pictures, job)
        sheet_sha256 = [sha(path) for path in sheets]
        with (job / "verification.json").open("x") as handle:
            handle.write(json.dumps({
                "video": master, "sha256": cfg["master"]["sha256"], "srt_sha256": cfg["srt"]["sha256"], "vtt_sha256": cfg["vtt"]["sha256"],
                "master_job": cfg["master_job"], "frames": cfg["total_frames"], "fps": FPS, "chapters": cfg["chapters"],
                "placements": cfg["placements"], "captions": cfg["captions"], "media": media, "strict_video_check": "passed",
                "strict_video_notes": notes, "strict_video_decodes": decodes,
                "encoded_loudness": loud, "caption_screen": {"caption_count": screen["caption_count"],
                "over_20_characters_per_second": len(screen["over_20_characters_per_second"]),
                "lines_over_42_characters": len(screen["lines_over_42_characters"])},
                "pictures": pictures, "sheets": sheets, "sheet_sha256": sheet_sha256, "pictures_viewed": False,
                "anchors": cfg["anchors"], "pause_sites": cfg["pause_sites"], "closing": cfg["closing"],
                "source_changes": cfg["source_changes"], "integration_sha256": cfg["integration_sha256"],
                "binding_sha256": cfg["binding_sha256"], "tools_sha256": cfg["tools_sha256"], "comparisons": cfg["comparisons"],
                "rebind_job": cfg["rebind_job"], "rebind_receipt_sha256": cfg["rebind_receipt_sha256"],
                "worker": {"host": HOST, "python3": cfg["tools"]["python3"], "ffmpeg": cfg["tools"]["ffmpeg"], "ffprobe": cfg["tools"]["ffprobe"]},
                "scope": cfg["scope"]}, indent=2) + "\n")
        for row in decodes:
            print(row, flush=True)
        print(f"{len(pictures)} pictures in {len(sheets)} sheets await root's viewing.", flush=True)
        print(cfg["success"], flush=True)
        return 0
    except Refused as refusal:
        print(f"REFUSED: {refusal}", flush=True)
        return 1


def verify_job(c):
    on_host()
    need(c.verify_job and c.job_dir, "verify-job needs --verify NN and --job-dir D")
    job, folder = c.verify_job, c.lane / c.verify_job
    script, inputs = c.job_dir / f"{job}.py", c.job_dir / f"{job}-inputs.json"
    r = load(folder / "resource-receipt.json")
    need(r.get("phase") == "native" and ran(r, script), f"{job}: the receipt ran {r.get('command')}")
    need(receipt_ok(r), f"{job} did not pass: exit {r.get('exit')}, error {r.get('error')}")
    need(r["inputs_sha256"] == sha(inputs) and json.loads(inputs.read_text())["files"][0]["sha256"] == sha(script), f"{job}: inputs or script moved")
    need(last_line(folder / "command.log") == VERIFY_SUCCESS, f"{job}: the log does not end with the success line")
    cfg = read_config(script)
    need(cfg["job"] == job and same(cfg["lane"], c.lane), f"{job}: the script names another job or lane")
    v = load(folder / "verification.json")
    need(v["sha256"] == cfg["master"]["sha256"] == sha(cfg["master"]["path"]) and v["frames"] == cfg["total_frames"], f"{job}: master or frames")
    need([p["frame"] for p in v["pictures"]] == [s["frame"] for s in cfg["samples"]], f"{job}: the pictures are not the planned ones")
    need(all(Path(p["path"]).is_file() and sha(p["path"]) == p.get("sha256") for p in v["pictures"]), f"{job}: a picture is missing or changed")
    sheet_hashes = v.get("sheet_sha256", [])
    need(len(v["sheets"]) == math.ceil(len(v["pictures"]) / 4) == len(sheet_hashes)
         and all(Path(s).is_file() and sha(s) == h for s, h in zip(v["sheets"], sheet_hashes)), f"{job}: a sheet is missing or changed")
    for key in ("srt", "vtt"):
        need(sha(cfg[key]["path"]) == cfg[key]["sha256"], f"{job}: the {key} changed since the job was written")
        need(v.get(f"{key}_sha256") == cfg[key]["sha256"], f"{job}: verification.json names another {key}")
    need(software_decode_clean(v.get("strict_video_decodes")), f"{job}: verification.json does not record root's clean software decode")
    need(v["strict_video_decodes"] == decode_lines((folder / "strict-video-verification.log").read_text(errors="replace")),
         f"{job}: verification.json's decode lines differ from the strict log's")
    need(v["pictures_viewed"] is False, f"{job}: verification.json claims the pictures were viewed; only root views them")
    need(last_line(folder / "strict-video-verification.log") == "VERIFY PASS", f"{job}: the strict log does not end with VERIFY PASS")
    need(any((folder / "decode-logs").glob("verify_decode.*")), f"{job}: the strict check's decode logs are missing")
    need(load(folder / "caption-screen.json")["srt_sha256"] == cfg["srt"]["sha256"], f"{job}: the caption screen read another SRT")
    for key in ("anchors", "pause_sites", "closing", "source_changes", "integration_sha256", "binding_sha256", "tools_sha256", "comparisons",
                "rebind_job", "rebind_receipt_sha256"):
        need(v.get(key) == cfg[key], f"{job}: verification.json's {key} differ from the job's")
    frames = [p["frame"] for p in v["pictures"]]
    lost = [a["id"] for a in cfg["anchors"] if frames[a["picture"] - 1] != a["frame"]]
    need(not lost, f"{job}: {len(lost)} anchors are not on their pictures, first {lost[:2]}")
    need(v.get("worker", {}).get("host") == HOST and all(v["worker"].get(k) == cfg["tools"][k] for k in ("python3", "ffmpeg", "ffprobe")),
         f"{job}: verification.json names other worker tools")
    return {"job": job, "verification_sha256": sha(folder / "verification.json"), "master_sha256": v["sha256"], "frames": v["frames"], "media": v["media"],
            "strict_video_notes": v["strict_video_notes"],
            "strict_video_decodes": v["strict_video_decodes"],
            "encoded_loudness": v["encoded_loudness"],
            "caption_screen": v["caption_screen"], "pictures": len(v["pictures"]), "sheets": len(v["sheets"]), "sheets_folder": str(folder),
            "anchors": len(v["anchors"]), "pause_sites": v["pause_sites"], "closing": v["closing"], "source_changes": v["source_changes"],
            "pictures_viewed": False, "open": "root views every sheet; listening, fine sync, final ASR and acceptance remain root's"}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["tools", "check", "write-job", "verify-job"])
    for name in ("--lane", "--jobs", "--crate", "--narration", "--series", "--integration", "--binding", "--tools"):
        parser.add_argument(name)
    parser.add_argument("--verify")
    parser.add_argument("--master-job")
    parser.add_argument("--job-dir")
    for name in ("--out", "--ffmpeg", "--ffprobe"):
        parser.add_argument(name)
    a = parser.parse_args(argv)
    try:
        if a.command == "tools":
            print(json.dumps({"tools_written": "WRITTEN", **write_tools(a)}, indent=2))
            return 0
        for name in ("lane", "jobs", "crate", "narration", "series", "integration", "binding", "tools"):
            need(getattr(a, name), f"{a.command} needs --{name}")
        c = Ctx(a)
        if a.command == "check":
            result = {"check": "PASS", **check(c)}
        elif a.command == "write-job":
            result = {"write_job": "WRITTEN", **write_job(c)}
        else:
            result = {"verify_job": "PASS", **verify_job(c)}
    except Refused as refusal:
        print(f"REFUSED: {refusal}")
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
