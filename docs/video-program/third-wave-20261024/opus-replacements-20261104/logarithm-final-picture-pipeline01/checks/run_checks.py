#!/usr/bin/env python3
"""Checks for prepare_final_picture_pipeline.py and its two controllers. Real folders are only read.

    python3 -I -B checks/run_checks.py SCRATCH        SCRATCH must not exist; it needs about 200 MB

A reads real outputs: stage09's tables and stems, the repaired sources, root's frozen build manifest.
B builds a scratch repository with the same layout and runs the two real controllers end to end. Real there: repaired
source, Lavender's rebind_audio.py --apply, root's assemble01.py, the guard's code (with its own lock, lane and baseline),
coherent_narration, narration_delivery, oracle.py, version 4's worker and root's render-master01.py prerequisites.
Stand-ins there: the compiled kit (a script that retimes stage09's tables and writes sparse stems) and build01.sh.
C changes one thing at a time in that repository and requires a refusal.

Nothing here compiles, renders, scores, synthesizes, or listens. The kit's real timing is what the first run will show.
"""
import contextlib
import hashlib
import importlib.util
import json
import os
import shutil
import struct
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
REAL = HERE.parents[6]
SCRATCH = Path(sys.argv[1]).absolute()
SCRATCH.mkdir(parents=True)
SCRATCH = SCRATCH.resolve()
HOME = str(Path.home())


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


M = load("pipeline", BUNDLE / "prepare_final_picture_pipeline.py")  # the real paths; only read below
H, N = M.H, M.N
CHAPTERS = M.CHAPTERS
PRIOR_T, PRIOR_S = M.LANE / M.PRIOR["timeline"], M.LANE / M.PRIOR["score"]
INITIAL = json.loads((M.S / "initial-copy.json").read_text())
STAGE09_WS = Path(INITIAL["source"])
REBIND = REAL / M.P_REL / "logarithm-final-audio-rebind01/rebind_audio.py"
F = SCRATCH / "repo"
RESULTS = []


def at(path):
    return F / Path(path).relative_to(REAL)


F_S, F_LANE, F_EL = at(M.S), at(M.LANE), at(M.EL)
F_BUNDLE, F_HELPER = at(BUNDLE), at(BUNDLE / "prepare_final_picture_pipeline.py")
F_WS, F_CRATE = F_S / "workspace", F_S / "workspace" / H.CRATE
GUARD = at(REAL / N.GUARD_REL)
TL_JOB, SC_JOB = "logarithm-final-picture-timeline01", "logarithm-final-picture-score01"


def clean(text):
    return str(text).replace(str(SCRATCH), "SCRATCH").replace(str(REAL), "REPO").replace(HOME, "HOME")


def record(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}" + (f"  [{clean(detail)}]" if detail else ""), flush=True)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def helper(*args):
    return subprocess.run(["python3", "-I", "-B", str(F_HELPER), *args], cwd=F, capture_output=True, text=True, timeout=900)


def shell(script, *args):
    return subprocess.run(["/bin/bash", str(script), *args], cwd=F, capture_output=True, text=True, timeout=1800)


def guard(job, inputs, *command):
    return subprocess.run([sys.executable, "-I", "-B", str(GUARD), "--phase", "native", "--job", job, "--inputs", str(inputs),
                           "--", *command], cwd=F, capture_output=True, text=True, timeout=900)


def refused(name, args, fragment):
    done = helper(*args)
    ok = done.returncode == 1 and fragment in done.stdout + done.stderr
    tail = (done.stdout + done.stderr).strip().splitlines()
    record(f"refuses: {name}", ok, clean(tail[-1] if tail else "")[:150] + ("" if ok else f" exit {done.returncode}"))


@contextlib.contextmanager
def changed(path, edit=None):
    """Rewrites a file, or moves it aside when edit is None, and always restores it."""
    path, before = Path(path), Path(path).read_bytes()
    stat = path.stat()
    aside = path.with_name(path.name + ".aside")
    if edit is None:
        path.rename(aside)
    else:
        path.write_bytes(edit(before))
    try:
        yield
    finally:
        if edit is None:
            aside.rename(path)
        else:
            path.write_bytes(before)
        os.utime(path, ns=(stat.st_atime_ns, stat.st_mtime_ns))


def fields(path, **updates):
    def edit(raw):
        data = json.loads(raw)
        for key, value in updates.items():
            if value is ...:
                data.pop(key, None)
            else:
                data[key] = value
        return json.dumps(data).encode()
    return changed(path, edit)


def replace(path, old, new):
    return changed(path, lambda raw: raw.replace(old.encode(), new.encode(), 1))


def manifest(path, paths):
    Path(path).write_text(json.dumps({"files": [{"path": str(p), "sha256": sha(p)} for p in paths]}, indent=2) + "\n")


def wav_head(samples):
    return (b"RIFF" + struct.pack("<I", 36 + samples * 8) + b"WAVEfmt " + struct.pack("<IHHIIHH", 16, 3, 2, 48000, 384000, 8, 32)
            + b"data" + struct.pack("<I", samples * 8))


def sparse_stem(path, samples):
    with open(path, "wb") as handle:
        handle.write(wav_head(samples))
        handle.truncate(44 + samples * 8)


STAND_IN = r'''#!/usr/bin/env python3
# Stand-in for the compiled kit, which names @@CRATE@@/src/main.rs. Not a build of anything.
import json, shutil, struct, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
LANE = HERE.parents[1] / "five-film-release-20261027"
TL, QA = LANE / "logarithm-stage09-timeline02/timeline-output", LANE / "logarithm-stage09-score02/scored/stems"
CHAPTERS = @@CHAPTERS@@
# Stage09's tables with the two predicted changes: 04_7 and everything after it +24, chapter 12's closing +420.
def opt(name):
    return sys.argv[sys.argv.index(name) + 1]
def timeline(chapter, narration, out):
    rows = [r.split("\t") for r in (TL / (chapter + ".timeline.tsv")).read_text().splitlines()]
    start = next((int(r[2]) for r in rows if r[0] == "line" and r[1] == "04_7"), 10 ** 9)
    move = lambda f: f + 24 if f >= start else f
    extra = {"04_undoing_growth": 24, "12_what_it_forgets": 420}.get(chapter, 0)
    text, frames = [], {}
    for r in rows:
        if r[0] == "frames":
            r[1] = str(int(r[1]) + extra)
        elif r[0] == "line":
            r[2] = str(move(int(r[2])))
            frames[r[1]] = int(r[2])
            r[4] = str(Path(narration) / (r[1] + ".wav"))
        elif r[0] == "mark":
            r[2] = str(move(int(r[2])))
        elif r[0] == "breath":
            r[1] = str(move(int(r[1])))
        text.append("\t".join(r))
    (out / (chapter + ".timeline.tsv")).write_text("\n".join(text) + "\n")
    cues = [r.split("\t") for r in (TL / (chapter + ".cues.tsv")).read_text().splitlines()]
    for r in cues:
        r[1] = f"{frames[r[0]] / 60:.3f}"
    (out / (chapter + ".cues.tsv")).write_text("\n".join("\t".join(r) for r in cues) + "\n")
    log = []
    for line in (TL / (chapter + ".cuelog.tsv")).read_text().splitlines():
        r = line.split("\t")
        if not line.startswith("#"):
            cue = r[0].rsplit("/", 1)[1][:-4]
            r[0], r[3] = str(Path(narration) / (cue + ".wav")), str(frames[cue])
        log.append("\t".join(r))
    (out / (chapter + ".cuelog.tsv")).write_text("\n".join(log) + "\n")
def score(timelines, out):
    stems = out / "stems"
    (stems / "qa").mkdir(parents=True)
    for chapter in CHAPTERS:
        rows = (timelines / (chapter + ".timeline.tsv")).read_text().splitlines()
        samples = int(next(r.split("\t")[1] for r in rows if r.startswith("frames\t"))) * 800
        head = (b"RIFF" + struct.pack("<I", 36 + samples * 8) + b"WAVEfmt " + struct.pack("<IHHIIHH", 16, 3, 2, 48000, 384000, 8, 32)
                + b"data" + struct.pack("<I", samples * 8))
        with (stems / (chapter + ".music.f32.wav")).open("wb") as handle:
            handle.write(head)
            handle.truncate(44 + samples * 8)
        qa = json.loads((QA / "qa" / (chapter + ".json")).read_text())
        for metric in qa["metrics"]:
            if metric["name"] == "length":
                metric["value"] = samples
        (stems / "qa" / (chapter + ".json")).write_text(json.dumps(qa))
    for name in ("score.fingerprint", "voice_profile.tsv"):
        shutil.copy2(QA / name, stems / name)
if sys.argv[1] == "chapters":
    print("\n".join(CHAPTERS))
elif sys.argv[1] == "facts":
    sys.stdout.write((HERE / "stand-in-facts.tsv").read_text())
elif sys.argv[1] == "render":
    out = Path(opt("--out"))
    out.mkdir(parents=True)
    for chapter in CHAPTERS:
        timeline(chapter, opt("--narration"), out)
elif sys.argv[1] == "score":
    score(Path(opt("--timelines")), Path(opt("--out")))
else:
    sys.exit(f"stand-in: {sys.argv[1:]}")
'''

BUILD_STAND_IN = r'''#!/bin/bash
# Stand-in for root's build01.sh: no compiler. The same outputs in the same places.
set -euo pipefail
S="@@S@@"
C="$S/workspace/explainer_corpus/logarithm_mysteries"
JOB="@@LANE@@/logarithm-final-picture-build01"
test -f "$S/integration.json"
test -f "$JOB/resource-receipt.json"
cp -p "$S/stand-in-binary" "$S/logarithm_mysteries-release01"
"$S/logarithm_mysteries-release01" facts > "$JOB/facts-native.tsv"
python3 -I -B "$C/oracle.py" > "$JOB/facts-oracle.tsv"
cmp "$JOB/facts-native.tsv" "$JOB/facts-oracle.tsv"
"$S/logarithm_mysteries-release01" chapters > "$JOB/chapters.txt"
'''


def real_controls():
    print("A. real outputs, read only", flush=True)
    frames = M.full_tables(PRIOR_T / "timeline-output", M.cues_of(), M.OLD_AUDIO)
    record("the table rules hold on stage09's real timeline02", len(frames) == 13 and sum(frames.values()) == 116245,
           f"{len(frames)} chapters, {sum(frames.values())} frames")
    stems = PRIOR_S / "scored/stems"
    record("stems and QA hold on stage09's real score02, with its 26 null metrics", M.score_gate(stems, frames) == 13)
    parts = sorted(p.name.split(".")[0] for p in stems.glob("*.parts.tsv"))
    record("that includes the split stems", len(parts) >= 4, " ".join(parts))
    same = M.compare_timeline(PRIOR_T / "timeline-output", PRIOR_T / "timeline-output")
    record("stage09 against itself reports zero deltas", same["film"]["delta_frames"] == 0 and not same["chapters_changed"]
           and not same["hard_failures"] and all(c["film_delta"] == 0 for c in same["cues"]))
    rows = M.tree(M.CRATE)
    frozen = {r["path"]: r["sha256"] for r in json.loads((M.S / "build-inputs01.json").read_text())["files"]}
    wanted = [M.S / "build01.sh", *M.build_sources(rows)]
    record("root's real build-inputs01.json binds every file the build gate requires", all(str(p) in frozen for p in wanted),
           f"{len(wanted)} required of {len(frozen)}")
    moved = [p for p, h in frozen.items() if sha(p) != h]
    record("and none of its rows has moved", not moved, " ".join(moved[:2]))
    binding = json.loads((STAGE09_WS / "narration-binding03.json").read_text())
    data = Path(binding["build"]["binary"]).read_bytes()
    record("the binary gate: stage09's real binary names its own crate, not the repaired one",
           H.mentions(data, STAGE09_WS / H.CRATE) > 0 and H.mentions(data, M.CRATE) == 0)
    map16 = json.loads((STAGE09_WS / H.CRATE / "docs/narration-direction-map-oliver16.json").read_text())["visual_holds"]
    changed_files = {p.rsplit("/", 1)[1] for p in json.loads((M.S / "integration.json").read_text())["changed_files"]}

    def stale(root):
        out = []
        for r in map16:
            try:
                M.hold_gate(root / r["path"], r["cue"], r["sha256"], r["line"], r["marker"], r["interval_sha256"], "hold")
            except H.Refused:
                out.append(r["cue"])
        return out
    record("the hold gate passes all 15 map16 holds on stage09's own source", len(map16) == 15 and not stale(STAGE09_WS))
    caught = stale(M.WS)
    record("and refuses exactly the holds in changed files on the repaired source",
           caught == [r["cue"] for r in map16 if r["path"].rsplit("/", 1)[1] in changed_files], " ".join(caught))


def compare_units():
    print("A. comparison rules on copies of stage09's real tables", flush=True)
    base = SCRATCH / "compare"
    shutil.copytree(PRIOR_T / "timeline-output", base)
    table = base / "07_branch_cut.timeline.tsv"
    with replace(table, "frames\t10182", "frames\t10242"):
        result = M.compare_timeline(PRIOR_T / "timeline-output", base)
    record("an unpredicted change is reported and refuses nothing", "07_branch_cut" in result["not_as_predicted"]
           and result["film"]["delta_frames"] == 60 and not result["hard_failures"], f"+{result['film']['delta_frames']} frames")
    swap = base / "09_matrix_logs.timeline.tsv"

    def swapped(raw):
        lines = raw.decode().split("\n")
        first = [n for n, line in enumerate(lines) if line.startswith("line\t")][:2]
        lines[first[0]], lines[first[1]] = lines[first[1]], lines[first[0]]
        return "\n".join(lines).encode()
    with changed(swap, swapped):
        record("a changed cue order is a hard failure", M.compare_timeline(PRIOR_T / "timeline-output", base)["hard_failures"])
    log = base / "09_matrix_logs.cuelog.tsv"
    wav = M.OLD_AUDIO / "09_1.wav"
    with replace(log, sha(wav), "0" * 64):
        record("changed cue audio is a hard failure", M.compare_timeline(PRIOR_T / "timeline-output", base)["hard_failures"])


def build_fixture():
    print("B. scratch repository", flush=True)
    F_S.mkdir(parents=True)
    F_LANE.mkdir(parents=True)
    (F / "target").mkdir()
    (F_BUNDLE / "checks").mkdir(parents=True)
    names = ["prepare_final_picture_pipeline.py", "timeline-after-final-picture-assemble01.sh", "score-after-final-picture-timeline01.sh"]
    others = [M.V4_DIR / "prepare_stage09_render.py", M.V4_DIR / "SHA256SUMS", REAL / H.NATIVE_REL, REAL / N.GUARD_REL, REAL / N.GUARD_SOURCE_REL]
    for path in [BUNDLE / name for name in names] + others:
        at(path).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, at(path))
    for sibling in M.EL.iterdir():  # takes, receipts and old audit, read through links; only the repair folder is copied
        if sibling.name != M.S.name:
            (F_EL / sibling.name).symlink_to(sibling)
    shutil.copy2(M.S / "initial-copy.json", F_S / "initial-copy.json")
    for row in INITIAL["files"]:  # the 51 approved files; the unit plan as it was before any rebind
        target = F_WS / row["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2((STAGE09_WS if row["path"].endswith("narration_units.json") else M.WS) / row["path"], target)
    data = json.loads((M.S / "integration.json").read_text())
    data["source_workspace"] = str(F_WS)
    for row in data["files"] + data["all_pre_rebind_files"]:
        row["path"] = str(F_WS / row["relative_path"])
    (F_S / "integration.json").write_text(json.dumps(data, indent=2) + "\n")
    verify = [sys.executable, "-I", "-B", str(F_WS / "tools/coherent_narration.py"), "verify", str(at(M.OLD_AUDIO)), "--script",
              str(F_CRATE / "script.tsv"), "--units", str(F_CRATE / "narration_units.json"), "--require-directions"]
    blind = subprocess.run(verify, cwd=F, capture_output=True, text=True, timeout=900)
    record("why the hold gate exists: coherent verify passes the repaired source with the stale map", blind.returncode == 0)
    done = subprocess.run([sys.executable, "-I", "-B", str(REBIND), "--apply", "--stage", str(F_S)], cwd=F, capture_output=True, text=True, timeout=900)
    record("Lavender's real rebind --apply on the scratch stage", done.returncode == 0, f"rebind_audio.py {sha(REBIND)[:12]}, exit {done.returncode} {done.stderr[-200:]}")
    oracle = subprocess.run(["python3", "-I", "-B", str(F_CRATE / "oracle.py")], cwd=F, capture_output=True, timeout=300, check=True)
    (F_S / "stand-in-facts.tsv").write_bytes(oracle.stdout)
    binary = F_S / "stand-in-binary"
    binary.write_text(STAND_IN.replace("@@CRATE@@", str(F_CRATE)).replace("@@CHAPTERS@@", json.dumps(CHAPTERS)))
    binary.chmod(0o755)
    build = F_S / "build01.sh"
    build.write_text(BUILD_STAND_IN.replace("@@S@@", str(F_S)).replace("@@LANE@@", str(F_LANE)))
    sources = [F_CRATE / rel for rel in sorted(H.tree(F_CRATE))
               if rel.startswith("src/") or rel in ("Cargo.toml", "Cargo.lock", "script.tsv", "oracle.py")]
    manifest(F_S / "build-inputs01.json", [build, binary, F_S / "integration.json", *sources])
    done = guard(M.BUILD, F_S / "build-inputs01.json", "/bin/bash", str(build))
    record("stand-in build through the real guard", done.returncode == 0, done.stderr[-200:])
    shutil.copy2(M.S / "assemble01.py", F_S / "assemble01.py")
    assemble = F_S / "assemble01.py"
    map17 = F_CRATE / "docs/narration-direction-map-oliver17.json"
    manifest(F_S / "assemble-inputs01.json", [assemble, F_CRATE / "script.tsv", F_CRATE / "narration_units.json", map17, F_S / "spans01.json",
                                              F_S / "holds-bound01.json", F_S / "integration.json", *sorted((F_WS / "tools").iterdir())])
    done = guard(M.ASSEMBLE, F_S / "assemble-inputs01.json", sys.executable, "-I", "-B", str(assemble))
    record("root's real assemble01.py through the real guard", done.returncode == 0, (done.stdout + done.stderr)[-300:] if done.returncode else "")
    for job, step in ((PRIOR_T, "timeline"), (PRIOR_S, "score")):
        (F_LANE / job.name).mkdir()
        shutil.copy2(job / "resource-receipt.json", F_LANE / job.name / "resource-receipt.json")
    shutil.copytree(PRIOR_T / "timeline-output", F_LANE / PRIOR_T.name / "timeline-output")
    stems = F_LANE / PRIOR_S.name / "scored/stems"
    (stems / "qa").mkdir(parents=True)
    for chapter in CHAPTERS:
        table = H.timeline_rows(PRIOR_T / "timeline-output" / f"{chapter}.timeline.tsv")
        sparse_stem(stems / f"{chapter}.music.f32.wav", int(H.first(table, "frames")) * 800)
        shutil.copy2(PRIOR_S / "scored/stems/qa" / f"{chapter}.json", stems / "qa" / f"{chapter}.json")
    for name in ("score.fingerprint", "voice_profile.tsv"):
        shutil.copy2(PRIOR_S / "scored/stems" / name, stems / name)
    shutil.copy2(M.S / "render-master01.py", F_S / "render-master01.py")
    (F_BUNDLE / "SHA256SUMS").write_text("".join(f"{sha(F_BUNDLE / name)}  {name}\n" for name in names))


def before_bind(integration, holds, ready):
    print("C. refusals before anything is bound", flush=True)
    build, assemble = F_LANE / M.BUILD / "resource-receipt.json", F_LANE / M.ASSEMBLE / "resource-receipt.json"
    done = helper(*ready)
    facts = json.loads(done.stdout) if done.returncode == 0 else {}
    record("ready passes on real-format receipts, rebind and assembly", facts.get("cues") == 80 and facts.get("chapters") == 13
           and facts.get("direction", {}).get("holds") == 15 and facts.get("direction", {}).get("closing_hold_cue") == "12_21",
           f"{facts.get('facts_rows')} fact rows, map {facts.get('direction', {}).get('map', '').rsplit('-', 1)[-1]}" if facts else done.stdout[-300:])
    report = json.loads((F_S / "narration01/report.json").read_text())
    bound = report["meta"]["direction_contract"]["binding"]["contract"]["map"]["path"]
    record("the assembled narration binds map17", bound.endswith("oliver17.json"), bound.rsplit("/", 1)[1])
    record("ready wrote nothing", not M_BINDING.exists() and not (F_S / "jobs").exists())
    for name, update in (("exit 1", {"exit": 1}), ("not admitted", {"admitted": False}), ("resource stop", {"stopped_for_resource_limit": True}),
                         ("an error", {"error": "x"}), ("inputs changed during the run", {"final_inputs_sha256": "0" * 64}), ("no finish", {"finished": ...})):
        with fields(build, **update):
            refused(f"build receipt with {name}", ready, "not a successful stable run")
    with fields(assemble, exit=1):
        refused("assembly receipt with exit 1", ready, "not a successful stable run")
    with fields(build, command=["/bin/bash", "other.sh"]):
        refused("a receipt for another script", ready, "ran")
    with changed(build, None):
        refused("no build receipt", ready, "no guard receipt")
    with fields(build, finished=...):
        record("poll: unfinished is waiting", helper("poll", "--job", M.BUILD).stdout.strip() == "waiting")
    with changed(build, lambda raw: raw[:20]):
        record("poll: a half-written receipt is waiting", helper("poll", "--job", M.BUILD).stdout.strip() == "waiting")
    with fields(build, exit=2):
        record("poll: a failed job is failed", helper("poll", "--job", M.BUILD).stdout.strip() == "failed")
    record("poll: a good job is ready", helper("poll", "--job", M.BUILD).stdout.strip() == "ready")
    refused("the wrong integration hash", ["ready", "--integration-sha256", "0" * 64, "--holds-sha256", holds], "root approved")
    refused("the wrong holds hash", ["ready", "--integration-sha256", integration, "--holds-sha256", "0" * 64], "root approved")
    with changed(F_S / "holds-bound01.json", None):
        refused("no holds-bound01.json, which the assembly froze", ready, "frozen inputs moved")
    with fields(F_S / "integration.json", approved_for_guarded_build=False):
        refused("an integration that does not approve", ["ready", "--integration-sha256", sha(F_S / "integration.json"), "--holds-sha256", holds], "does not approve")
    units_before = F_S / "narration_units-before-rebind01.json"
    with changed(units_before, lambda raw: raw + b" "):
        refused("a preserved unit plan that is not the integrated one", ready, "is not the integrated file")
    source = F_CRATE / "src/chapters/what_it_forgets.rs"
    with changed(source, lambda raw: raw + b"\n"):
        refused("a source edited after integration", ready, "is not the integrated file")
    with changed(F_S / "build-inputs01.json", lambda raw: raw + b" "):
        refused("a manifest that is not the frozen one", ready, "not the manifest its guard froze")
    narrow = json.loads((F_S / "build-inputs01.json").read_text())
    narrow["files"] = [r for r in narrow["files"] if not r["path"].endswith("oracle.py")]
    text = json.dumps(narrow).encode()
    digest = hashlib.sha256(text).hexdigest()
    with changed(F_S / "build-inputs01.json", lambda raw: text), fields(build, inputs_sha256=digest, final_inputs_sha256=digest):
        refused("a frozen manifest that does not bind oracle.py", ready, "does not bind")
    with changed(F_S / "stand-in-binary", lambda raw: raw + b"\n"):
        refused("a build input moved after the build", ready, "frozen inputs moved")
    with changed(M_BINARY, None):
        refused("no binary", ready, "no binary")
    with changed(M_BINARY, lambda raw: raw.replace(str(F_CRATE).encode(), b"elsewhere")):
        refused("a binary compiled for another crate", ready, "not compiled for the repaired")
    wav = F_S / "narration01/09_1.wav"

    def flip(raw):
        return raw[:2000] + bytes([raw[2000] ^ 1]) + raw[2001:]
    with changed(wav, flip):
        refused("a changed WAV", ready, "differs from stage09")
    with changed(F_S / "narration01/phrase-timings.json", lambda raw: raw + b" "):
        refused("changed phrase timings", ready, "phrase-timings.json differs")
    with changed(F_LANE / M.ASSEMBLE / "audio-comparison.json", lambda raw: raw.replace(b"true", b"false")):
        refused("an assembly that does not record identity", ready, "byte-identical")
    with changed(F_LANE / M.BUILD / "facts-oracle.tsv", lambda raw: raw + b"x\t1\n"):
        refused("oracle facts that differ", ready, "native and oracle facts differ")
    with changed(F_S / "stand-in-facts.tsv", lambda raw: raw + b"x\t1\n"):
        refused("a binary whose facts differ from its record", ready, "the binary's facts")
    with changed(F_LANE / M.BUILD / "chapters.txt", lambda raw: raw.split(b"\n", 1)[1]):
        refused("twelve chapters", ready, "not the 13 chapters")
    with changed(F_WS / "tools/narration_cache.py", lambda raw: raw + b"\n"):
        refused("a changed tool", ready, "narration_cache.py")
    audit, frozen = F_S / "holds-bound01.json", F_S / "assemble-inputs01.json"

    @contextlib.contextmanager
    def audited(edit):
        """A tampered audit that root approved and the assembly froze, so holds_gate alone must refuse it."""
        data = json.loads(audit.read_text())
        edit(data)
        text = json.dumps(data).encode()
        digest = hashlib.sha256(text).hexdigest()
        rows = json.loads(frozen.read_text())
        for row in rows["files"]:
            if row["path"] == str(audit):
                row["sha256"] = digest
        bound = json.dumps(rows).encode()
        inputs = hashlib.sha256(bound).hexdigest()
        with changed(audit, lambda raw: text), changed(frozen, lambda raw: bound), fields(assemble, inputs_sha256=inputs, final_inputs_sha256=inputs):
            yield ["ready", "--integration-sha256", integration, "--holds-sha256", digest]

    def tamper_hold(d):
        d["holds"][3]["after"]["interval_sha256"] = "0" * 64
    def tamper_closing(d):
        d["editorial_closing_hold"]["marker"] = "kit.music_breath(stage, ELSEWHERE)?;"
    def tamper_map(d):
        d["map"] = d["predecessor_map"]
    def tamper_list(d):
        d["holds"] = d["holds"][:-1]
    for name, edit, fragment in (("an audit whose hold differs from the map", tamper_hold, "the map and audit differ"),
                                 ("an audit whose closing marker left its line", tamper_closing, "the marker left"),
                                 ("an audit naming the old map", tamper_map, "bind different maps"),
                                 ("an audit missing a hold", tamper_list, "different holds"),
                                 ("an audit of another integration", lambda d: d.update(integration_sha256="0" * 64), "audits another integration")):
        with audited(edit) as args:
            refused(name, args, fragment)
    with changed(F_S / "narration01/report.json", lambda raw: raw + b"\n"):
        refused("the real coherent verify rejecting a changed report", ready, "coherent verify refused")
    refused("a step before the binding exists", ["step", "--step", "timeline"], "no binding")


def controllers_refuse(seal, integration, holds):
    print("C. controllers launch nothing when they must not", flush=True)
    timeline = F_BUNDLE / "timeline-after-final-picture-assemble01.sh"
    build = F_LANE / M.BUILD / "resource-receipt.json"
    untouched = lambda: not M_BINDING.exists() and not (F_S / "jobs").exists() and not (F_LANE / TL_JOB).exists()
    done = shell(timeline, "0" * 64, integration, holds)
    record("a wrong seal launches nothing", done.returncode == 1 and untouched())
    with fields(build, exit=1):
        done = shell(timeline, seal, integration, holds)
    record("a failed build launches nothing", done.returncode == 1 and "failed; no timeline launched" in done.stdout and untouched())
    done = shell(timeline, seal, integration, "0" * 64)
    record("unapproved holds launch nothing", done.returncode == 1 and untouched())
    hurry = F_BUNDLE / "hurry.sh"
    hurry.write_text(timeline.read_text().replace("attempt<2160", "attempt<1").replace("sleep 20", "sleep 0"))
    with fields(build, finished=...):
        done = shell(hurry, seal, integration, holds)
    record("a deadline launches nothing", done.returncode == 2 and "wait deadline" in done.stdout and untouched())
    hurry.unlink()


def end_to_end(seal, integration, holds):
    print("B. both real controllers, end to end", flush=True)
    done = shell(F_BUNDLE / "timeline-after-final-picture-assemble01.sh", seal, integration, holds)
    last = done.stdout.strip().splitlines()[-1] if done.stdout.strip() else ""
    record("the timeline controller finishes", done.returncode == 0 and last == "Timeline finished, verified and compared; score not launched",
           last if done.returncode == 0 else (done.stdout + done.stderr)[-500:])
    receipt = json.loads((F_LANE / TL_JOB / "resource-receipt.json").read_text())
    script = (F_S / "jobs" / f"{TL_JOB}.py").read_text()
    record("the job ran under the real guard, from version 4's unchanged template", H.receipt_ok(receipt)
           and receipt["command"] == ["python3", "-I", "-B", str(F_S / "jobs" / f"{TL_JOB}.py")]
           and f"Written by prepare_stage09_render.py (sha256 {M.V4_SHA})" in script)
    rows = json.loads((F_S / "jobs" / f"{TL_JOB}-inputs.json").read_text())["files"]
    named = {r["path"] for r in rows}
    record("its frozen inputs bind the binary, narration, holds, map, integration and both prior receipts",
           {str(M_BINARY), str(F_S / "narration01/01_1.wav"), str(F_S / "holds-bound01.json"), str(F_S / "integration.json"),
            str(F_CRATE / "docs/narration-direction-map-oliver17.json"), str(F_LANE / M.BUILD / "resource-receipt.json"),
            str(F_LANE / M.ASSEMBLE / "resource-receipt.json")} <= named, f"{len(rows)} rows")
    c = json.loads((F_S / "schedule-comparison01.json").read_text())
    by = {x["chapter"]: x for x in c["chapters"]}
    cues = {x["cue"]: x for x in c["cues"]}
    record("the comparison reports the stand-in's actual deltas, not equality",
           c["film"]["delta_frames"] == 444 and c["chapters_changed"] == ["04_undoing_growth", "12_what_it_forgets"]
           and not c["not_as_predicted"] and not c["hard_failures"] and by["12_what_it_forgets"]["frames_after"] == 10257
           and by["04_undoing_growth"]["cues_moved_locally"] == 1 and cues["04_7"]["local_delta"] == 24 and cues["04_6"]["film_delta"] == 0
           and cues["05_1"]["film_delta"] == 24 and cues["05_1"]["local_delta"] == 0 and cues["12_21"]["film_delta"] == 24,
           f"film +{c['film']['delta_frames']} frames, +{c['film']['delta_seconds']} s")
    print("C. tables after the timeline", flush=True)
    output = F_LANE / TL_JOB / "timeline-output"
    verify = ["verify-step", "--step", "timeline"]
    record("verify-step passes the 13 full tables", helper(*verify).returncode == 0)
    table = output / "09_matrix_logs"
    log, listed = table.with_suffix(".cuelog.tsv"), table.with_suffix(".cues.tsv")

    def rotate(raw):
        lines = raw.decode().splitlines()
        return "\n".join(lines[1:] + lines[:1]).encode() + b"\n"
    wav = F_S / "narration01/09_1.wav"
    with changed(listed, None):
        refused("a missing table", verify, "missing table")
    with changed(listed, rotate):
        refused("cues out of script order", verify, "not the script's")
    with replace(log, str(F_S / "narration01/09_1.wav"), str(at(M.OLD_AUDIO) / "09_1.wav")):
        refused("a cue playing old narration", verify, "plays outside the narration")
    with replace(log, sha(wav), "0" * 64):
        refused("a cue log hash that is not the WAV's", verify, "not the WAV's")
    with replace(listed, "0.600", "0.700"):
        refused("cues.tsv disagreeing with the timeline", verify, "disagree")
    with replace(listed, "23.840", "9999.000"):
        refused("a cue running past the end", verify, "runs past")
    with changed(table.with_suffix(".timeline.tsv"), lambda raw: raw + b"\n"):
        refused("a timeline that moved after its summary", verify, "")
    refused("a second timeline job", ["step", "--step", "timeline"], "exists")
    refused("a second comparison", ["compare", "--step", "timeline"], "preserve it")
    refused("a second binding", ["bind", "--integration-sha256", integration, "--holds-sha256", holds], "preserve it")
    score = F_BUNDLE / "score-after-final-picture-timeline01.sh"
    with fields(F_LANE / TL_JOB / "resource-receipt.json", exit=1):
        done = shell(score, seal)
    record("a failed timeline launches no score", done.returncode == 1 and "no score launched" in done.stdout and not (F_LANE / SC_JOB).exists())
    done = shell(score, seal)
    last = done.stdout.strip().splitlines()[-1] if done.stdout.strip() else ""
    record("the score controller finishes", done.returncode == 0 and last == "Score finished, verified and compared; render not launched",
           last if done.returncode == 0 else (done.stdout + done.stderr)[-500:])
    c = json.loads((F_S / "score-comparison01.json").read_text())
    by = {x["chapter"]: x for x in c["chapters"]}
    record("the score comparison reports the changed stems and the QA change",
           c["stems_changed"] == ["04_undoing_growth", "12_what_it_forgets"] and by["12_what_it_forgets"]["qa_changed"].get("length") == [9837 * 800, 10257 * 800],
           " ".join(c["stems_changed"]))
    print("C. stems and QA after the score", flush=True)
    verify = ["verify-step", "--step", "score"]
    record("verify-step passes 13 stems and QA reports, with their null metrics", helper(*verify).returncode == 0)
    stems = F_LANE / SC_JOB / "scored/stems"
    summary = F_LANE / SC_JOB / "score-summary.json"
    report = stems / "qa/09_matrix_logs.json"

    def qa(edit, name, fragment):
        data = json.loads(report.read_text())
        edit(data)
        text = json.dumps(data).encode()
        hashes = json.loads(summary.read_text())["qa_sha256"]
        with changed(report, lambda raw: text), fields(summary, qa_sha256={**hashes, "09_matrix_logs": hashlib.sha256(text).hexdigest()}):
            refused(name, verify, fragment)

    def fail_metric(d):
        d["metrics"][0]["pass"] = False
    def wrong_length(d):
        next(m for m in d["metrics"] if m["name"] == "length")["value"] = 1
    qa(fail_metric, "a failed QA metric under a passed report", "QA failed")
    qa(wrong_length, "a QA length that is not the chapter's", "QA measured length")
    stem = stems / "09_matrix_logs.music.f32.wav"
    size = stem.stat().st_size
    os.truncate(stem, size - 800)
    refused("a stem shorter than its header", verify, "shorter than its header")
    os.truncate(stem, size)
    samples = (size - 44) // 8
    head = wav_head(samples - 800)
    with open(stem, "r+b") as handle:
        handle.write(head)
    refused("a stem one chapter frame short", verify, "the chapter needs")
    with open(stem, "r+b") as handle:
        handle.write(wav_head(samples))
    with changed(report, None):
        refused("a missing QA report", verify, "")
    probe = ("import importlib.util, sys\n"
             f"spec = importlib.util.spec_from_file_location('render_master', {str(F_S / 'render-master01.py')!r})\n"
             "m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)\n"
             "m.prerequisites(); print('prerequisites hold')\n")
    done = subprocess.run([sys.executable, "-I", "-B", "-c", probe], cwd=F, capture_output=True, text=True, timeout=300)
    record("root's real render-master01.py prerequisites hold on these outputs", done.returncode == 0 and "prerequisites hold" in done.stdout,
           f"render-master01.py {sha(M.S / 'render-master01.py')[:12]} " + done.stderr[-200:])


def main():
    print(f"helper {sha(BUNDLE / 'prepare_final_picture_pipeline.py')[:12]}; version 4 {M.V4_SHA[:12]}; integration {sha(M.S / 'integration.json')[:12]}")
    real_controls()
    compare_units()
    build_fixture()
    integration, holds = sha(F_S / "integration.json"), sha(F_S / "holds-bound01.json")
    seal = sha(F_BUNDLE / "SHA256SUMS")
    ready = ["ready", "--integration-sha256", integration, "--holds-sha256", holds]
    before_bind(integration, holds, ready)
    record("the seal command", helper("seal", "--sums", seal).returncode == 0)
    with changed(F_HELPER, lambda raw: raw + b"\n"):
        record("the seal command refuses a changed bundle file", helper("seal", "--sums", seal).returncode == 1)
    controllers_refuse(seal, integration, holds)
    end_to_end(seal, integration, holds)
    wrote = [p for p in (M.BINDING, M.S / "jobs", *M.COMPARISON.values(), M.LANE / TL_JOB, M.LANE / SC_JOB) if p.exists()]
    record("the real repair folder and lane got none of this run's outputs", not wrote, " ".join(map(str, wrote)))
    print(f"{'ALL ' if all(RESULTS) else 'FAILED: '}{sum(RESULTS)}/{len(RESULTS)} PASS. Stand-in kit and build; no real build, assembly, timeline or score ran.")
    return 0 if all(RESULTS) else 1


M_BINARY, M_BINDING = F_S / "logarithm_mysteries-release01", F_S / "final-picture-binding01.json"
if __name__ == "__main__":
    sys.exit(main())
