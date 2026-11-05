#!/usr/bin/env python3
"""Logarithm final-picture timeline and score, for the repaired workspace only (bead 87do.63).

A thin binding on the sealed stage09 helper, prepare_stage09_render.py version 4. Version 4's write_step, verify_step,
verify_job, job template, argv, timeouts and success lines run unchanged, loaded by hash. Two names change in memory
on that loaded copy, and no file of version 4 is edited:
- STAGE, so the jobs are logarithm-final-picture-timeline01 and -score01;
- load_binding, because version 4 refuses any workspace but its own stage09 one.

What this file adds is what version 4 could not know: the build and assembly receipts of the repaired workspace, root's
approval of integration.json and holds-bound01.json, the direction map checked against the repaired sources, the full
tables, the score stems' real length, and comparisons with the stage09 schedules. There is no render or master step.

Run from anywhere; the controllers run it. Root's two approvals are hashes root passes in, so nothing is inferred from
files whose schema this helper does not own.

    seal --sums SHA                                     this bundle against its SHA256SUMS
    poll --job JOB [--needs FILE]                       prints waiting, ready or failed
    ready --integration-sha256 H --holds-sha256 H       every gate; writes nothing
    bind  --integration-sha256 H --holds-sha256 H       every gate, then S/final-picture-binding01.json
    step --step timeline|score                          writes one guarded job, launches nothing
    verify-step --step timeline|score                   version 4's verify_job plus the gates here
    compare --step timeline|score                       actual deltas from stage09; writes one comparison file

Exit 0 pass, 1 refused, 2 usage. Technical gates only: nothing here was heard, viewed or accepted.
"""
import argparse
import hashlib
import importlib.util
import json
import struct
import subprocess
import sys
import types
from pathlib import Path

HERE = Path(__file__).resolve()
REPO = HERE.parents[5]
P_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
ROOT_REL = "narration_lab/third-wave-20261024/root"
V4_DIR = REPO / P_REL / "logarithm-stage09-render-preparation04"
V4_SHA = "debd6f757ba571e71ea1a4a306f81fd12cf00417dee0a9e627834c8a9ed9ee73"
V4_SUMS_SHA = "34b04c880117b80344686197688b2591b5b78d7e79f1e1294de653247b7fd409"  # root's score02 controller pinned it
EL = REPO / ROOT_REL / "elevenlabs-logarithm-rebuild-20261104"
LANE = REPO / ROOT_REL / "five-film-release-20261027"
S = EL / "final-picture-repair01"
WS = S / "workspace"
BINARY = S / "logarithm_mysteries-release01"
NARRATION = S / "narration01"
OLD_AUDIO = EL / "production-audio-stage09-01/prepared02/narration"
BUILD, ASSEMBLE = "logarithm-final-picture-build01", "logarithm-final-picture-assemble01"
PRIOR = {"timeline": "logarithm-stage09-timeline02", "score": "logarithm-stage09-score02"}
COMPARISON = {"timeline": S / "schedule-comparison01.json", "score": S / "score-comparison01.json"}
BINDING = S / "final-picture-binding01.json"
MM = SERIES = "01"
# The two repairs that were meant to move a chapter, in frames at 60 fps. A comparison reports every other change.
PREDICTED = {"04_undoing_growth": 24, "12_what_it_forgets": 420}


def load_v4():
    path = V4_DIR / "prepare_stage09_render.py"
    for file, digest in ((path, V4_SHA), (V4_DIR / "SHA256SUMS", V4_SUMS_SHA)):
        found = hashlib.sha256(file.read_bytes()).hexdigest()
        if found != digest:
            raise SystemExit(f"REFUSED: {file.name} is {found[:12]}, expected {digest[:12]}")
    spec = importlib.util.spec_from_file_location("prepare_stage09_render", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


H = load_v4()
N = H.N
Refused, need, sha, tree, pinned = H.Refused, H.need, H.sha, H.tree, H.pinned
CHAPTERS = H.CHAPTERS
CRATE = WS / H.CRATE
TOOLS = WS / "tools"
H.STAGE = "logarithm-final-picture"
C = types.SimpleNamespace(lane=LANE, jobs=S / "jobs", crate=CRATE, tools=TOOLS, target=REPO / "target",
                          ws_binding=S / "integration.json", stage_rows=lambda: tree(CRATE))


def load_binding(c, mm):
    """Version 4's load_binding for this workspace: the binding is this helper's, and nothing bound has moved."""
    need(BINDING.is_file(), f"no binding {BINDING}: bind needs the receipts and approvals first")
    b = json.loads(BINDING.read_text())
    need(b.get("helper_sha256") == sha(HERE) and b.get("render_helper_sha256") == V4_SHA,
         "the binding is another helper's")
    need(sha(b["build"]["binary"]) == b["build"]["binary_sha256"], "the bound binary moved")
    need(tree(CRATE) == b["crate"] and tree(TOOLS) == H.TOOLS, "the workspace moved since it was bound")
    moved = [p for p, h in b["files"].items() if not Path(p).is_file() or sha(p) != h]
    need(not moved, f"bound files moved: {moved[:3]}")
    return b, BINDING


H.load_binding = load_binding


# ---------------------------------------------------------------- gates

def cues_of():
    cues = H.script_cues(CRATE / "script.tsv")
    need(len(CHAPTERS) == 13 and len(cues) == len(set(cues)) == 80,
         f"expected 13 chapters and 80 cues, found {len(CHAPTERS)} and {len(cues)}")
    return cues


def finished(job, script):
    """A successful, stable guarded run of exactly this script."""
    path = LANE / job / "resource-receipt.json"
    need(path.is_file(), f"no guard receipt for {job}")
    r = json.loads(path.read_text())
    need(H.receipt_ok(r) and r.get("phase") == "native",
         f"{job} is not a successful stable run: admitted {r.get('admitted')}, exit {r.get('exit')}, "
         f"finished {bool(r.get('finished'))}, inputs stable {r.get('inputs_sha256') == r.get('final_inputs_sha256')}, "
         f"resource stop {r.get('stopped_for_resource_limit', False)}, error {r.get('error')}")
    need((r.get("command") or [""])[-1] == str(script), f"{job} ran {r.get('command')}, not {script}")
    return r


def build_sources(crate):
    """The crate files build01.sh compiles, tests or compares: root's manifest binds these, not the map or units."""
    return [CRATE / rel for rel in crate if (rel.startswith("src/") and rel.endswith(".rs"))
            or rel in ("Cargo.toml", "Cargo.lock", "script.tsv", "oracle.py")]


def frozen(job, manifest, receipt, required):
    """The manifest the guard froze for this job: still true, and it binds the files that job depends on."""
    need(manifest.is_file(), f"{job}: no input manifest {manifest}")
    need(sha(manifest) == receipt["inputs_sha256"], f"{job}: {manifest.name} is not the manifest its guard froze")
    rows = {row["path"]: row["sha256"] for row in json.loads(manifest.read_text())["files"]}
    moved = [p for p, h in rows.items() if not Path(p).is_file() or sha(p) != h]
    need(not moved, f"{job}: its frozen inputs moved after it ran: {moved[:3]}")
    absent = [str(p) for p in required if str(p) not in rows]
    need(not absent, f"{job}: {manifest.name} does not bind {absent[:3]}")


def integration_gate(digest):
    """Root's approval is the hash root passes in. What the document approves must still be on disk.

    The one row allowed to move is the unit plan: the rebind rewrites it and keeps the approved bytes beside it."""
    path = S / "integration.json"
    need(path.is_file(), f"missing {path}")
    need(sha(path) == digest, f"integration.json is {sha(path)[:12]}, root approved {digest[:12]}")
    data = json.loads(path.read_text())
    need(data.get("approved_for_guarded_build") is True and data.get("audio_changed") is False,
         "integration.json does not approve this source with unchanged audio")
    need(data.get("script_sha256") == sha(CRATE / "script.tsv"), "integration.json names another script")
    need(Path(data["source_workspace"]).resolve() == WS.resolve(), "integration.json names another workspace")
    units = (CRATE / "narration_units.json").resolve()
    for row in data["all_pre_rebind_files"]:
        file = Path(row["path"]).resolve()
        file = S / "narration_units-before-rebind01.json" if file == units else file
        need(file.is_file() and sha(file) == row["sha256"], f"{row['relative_path']} is not the integrated file")
    return len(data["all_pre_rebind_files"])


def hold_gate(file, cue, digest, line, marker, interval, what):
    """One hold against the source itself: file hash, marker on its line, and the say-to-hold interval."""
    name = f"{what} {cue} in {file.name}"
    need(file.is_file() and sha(file) == digest, f"{name}: the file hash is stale")
    text = file.read_text()
    lines = text.split("\n")
    need(0 < line <= len(lines) and marker in lines[line - 1], f"{name}: the marker left line {line}")
    film = text.split("\n#[cfg(test)]\nmod tests {")[0].split("\n")  # source tests quote these lines; only the film counts
    says = [n for n, code in enumerate(film, 1) if f'kit.say(stage, "{cue}")' in code]
    need(len(says) == 1 and says[0] <= line, f"{name}: expected one say above the hold, found {says}")
    found = hashlib.sha256("\n".join(lines[says[0] - 1:line]).encode()).hexdigest()
    need(found == interval, f"{name}: the interval differs")


def holds_gate(digest, integration_digest, narration):
    """holds-bound01.json, the map the narration binds, and both against the repaired sources.

    coherent verify does not do this: it passes the repaired workspace with the stale stage09 map."""
    path = S / "holds-bound01.json"
    need(path.is_file(), f"missing {path}")
    need(sha(path) == digest, f"holds-bound01.json is {sha(path)[:12]}, root approved {digest[:12]}")
    audit = json.loads(path.read_text())
    need(audit.get("integration_sha256") == integration_digest, "holds-bound01.json audits another integration")
    need(audit.get("script_sha256") == sha(CRATE / "script.tsv"), "holds-bound01.json audits another script")
    contract = json.loads((narration / "report.json").read_text())["meta"]["direction_contract"]["binding"]["contract"]
    need(contract["map"] == audit.get("map"), "the narration and holds-bound01.json bind different maps")
    map_file = WS / contract["map"]["path"]
    need(map_file.is_file() and sha(map_file) == contract["map"]["sha256"], f"{map_file.name} is not the bound map")
    rows = [r for r in json.loads(map_file.read_text())["visual_holds"] if r.get("video") == H.TOPIC]
    after = {h["cue"]: h["after"] for h in audit["holds"]}
    need(rows and [r["cue"] for r in rows] == [h["cue"] for h in audit["holds"]], "the map and the audit list different holds")
    for r in rows:
        a = after[r["cue"]]
        need((r["sha256"], r["line"], r["marker"], r["interval_sha256"])
             == (a["source_sha256"], a["marker_line"], a["marker"], a["interval_sha256"]), f"hold {r['cue']}: the map and audit differ")
        hold_gate(WS / r["path"], r["cue"], r["sha256"], r["line"], r["marker"], r["interval_sha256"], "hold")
    closing = audit["editorial_closing_hold"]
    hold_gate(CRATE / "src/chapters" / closing["file"], closing["cue"], closing["source_sha256"], closing["marker_line"],
              closing["marker"], closing["interval_sha256"], "closing hold")
    return {"map": contract["map"]["path"], "map_sha256": contract["map"]["sha256"], "holds": len(rows),
            "closing_hold_cue": closing["cue"]}


def live(argv, recorded, what):
    done = subprocess.run(argv, cwd=REPO, capture_output=True, timeout=300)
    need(done.returncode == 0, f"{what} exited {done.returncode}: {done.stderr.decode(errors='replace')[-300:]}")
    need(done.stdout == recorded, f"{what} differs from its build record")


def facts_gate(folder):
    native, oracle = folder / "facts-native.tsv", folder / "facts-oracle.tsv"
    need(native.is_file() and oracle.is_file() and native.stat().st_size > 0, "the build left no facts tables")
    need(native.read_bytes() == oracle.read_bytes(), "native and oracle facts differ")
    live([str(BINARY), "facts"], native.read_bytes(), "the binary's facts")
    live(["python3", "-I", "-B", str(CRATE / "oracle.py")], oracle.read_bytes(), "oracle.py")
    return len([line for line in native.read_text().splitlines() if line.strip()])


def audio_gate(cues):
    """Byte identity with the stage09 narration, from the assembly's record and from the files themselves."""
    record = LANE / ASSEMBLE / "audio-comparison.json"
    need(record.is_file(), "the assembly left no audio-comparison.json")
    a = json.loads(record.read_text())
    need(a.get("cues") == 80 and a.get("all_full_wavs_byte_identical") is True
         and a.get("all_phrase_timings_byte_identical") is True, "the assembly does not record byte-identical audio")
    need({p.stem for p in NARRATION.glob("*.wav")} == set(cues), "narration01 does not hold exactly the script's cues")
    for cue in cues:
        digest = sha(NARRATION / f"{cue}.wav")
        need(digest == a["wav_sha256"].get(cue) == sha(OLD_AUDIO / f"{cue}.wav"), f"{cue}.wav differs from stage09")
    timings = sha(NARRATION / "phrase-timings.json")
    need(timings == a.get("phrase_timings_sha256") == sha(OLD_AUDIO / "phrase-timings.json"),
         "phrase-timings.json differs from stage09")
    return {"wavs_byte_identical_to_stage09": len(cues), "phrase_timings_sha256": timings}


def tool_checks():
    """Version 4's two audio checks, on the workspace's tools and narration01."""
    verify = ["python3", "-I", "-B", str(TOOLS / "coherent_narration.py"), "verify", str(NARRATION), "--script",
              str(CRATE / "script.tsv"), "--units", str(CRATE / "narration_units.json"), "--require-directions"]
    done = H.run_tool(verify)
    need(done.returncode == 0, f"coherent verify refused: {(done.stderr or done.stdout).strip()[-500:]}")
    contract = ["python3", "-I", "-B", str(TOOLS / "narration_delivery.py"), "--topic", str(CRATE), "--narration",
                str(NARRATION), "--consumer", "render", "--gain-db", H.GAIN_DB]
    for key in CHAPTERS:
        contract += ["--chapter-key", key]
    done = H.run_tool(contract)
    need(done.returncode == 0, f"the delivery contract refused: {(done.stderr or done.stdout).strip()[-500:]}")
    result = json.loads(done.stdout)
    need(result.get("verified_cues") == result.get("selected_cues") == 80 and result.get("topic") == H.TOPIC
         and result.get("chapter_scope") is None and result.get("promotion") is False, f"contract result {result}")
    return {"coherent_verify": verify, "delivery_contract": contract}


def gates(a):
    """Everything before freezing inputs, cheapest first. Returns the binding's body and writes nothing."""
    for name, digest in H.TOOLS.items():
        pinned(TOOLS / name, digest, f"workspace tools/{name}")
    pinned(REPO / N.GUARD_REL, N.TOOL_PIN[N.GUARD_REL], "the resource guard")
    pinned(REPO / N.GUARD_SOURCE_REL, N.TOOL_PIN[N.GUARD_SOURCE_REL], "the guard's source")
    build_script, assemble_script = S / "build01.sh", S / "assemble01.py"
    build_receipt = finished(BUILD, build_script)
    assemble_receipt = finished(ASSEMBLE, assemble_script)
    integrated = integration_gate(a.integration_sha256)
    cues = cues_of()
    crate = tree(CRATE)
    build_inputs, assemble_inputs = S / "build-inputs01.json", S / "assemble-inputs01.json"
    bound, spans = S / "holds-bound01.json", S / "spans01.json"
    frozen(BUILD, build_inputs, build_receipt, [build_script, *build_sources(crate)])
    frozen(ASSEMBLE, assemble_inputs, assemble_receipt,
           [assemble_script, CRATE / "script.tsv", CRATE / "narration_units.json", spans, TOOLS / "coherent_narration.py"])
    folder = LANE / BUILD
    need(BINARY.is_file(), f"no binary at {BINARY}")
    need(H.mentions(BINARY.read_bytes(), CRATE) > 0, "the binary was not compiled for the repaired workspace's crate")
    listed = (folder / "chapters.txt").read_text().splitlines() if (folder / "chapters.txt").is_file() else []
    need(listed == CHAPTERS, f"the build's chapter registry is not the 13 chapters: {listed}")
    live([str(BINARY), "chapters"], (folder / "chapters.txt").read_bytes(), "the binary's chapters")
    audio = audio_gate(cues)
    hold_facts = holds_gate(a.holds_sha256, a.integration_sha256, NARRATION)
    facts = facts_gate(folder)
    checks = tool_checks()
    for step, job in PRIOR.items():
        need(H.receipt_ok(json.loads((LANE / job / "resource-receipt.json").read_text())), f"the prior {step} {job} did not pass")
    files = {str(p): sha(p) for p in (spans, bound, build_inputs, assemble_inputs, build_script, assemble_script, HERE,
                                      HERE.parent / "SHA256SUMS")}
    for job in (BUILD, ASSEMBLE):
        files.update({str(LANE / job / rel): h for rel, h in tree(LANE / job).items()})
    files.update({str(NARRATION / rel): h for rel, h in tree(NARRATION).items()})
    return {"schema": "logarithm-final-picture-binding-v1", "helper_sha256": sha(HERE), "render_helper_sha256": V4_SHA,
            "ws": str(WS), "crate": crate, "approved": {"integration_sha256": a.integration_sha256,
                                                       "holds_bound_sha256": a.holds_sha256, "integrated_rows": integrated},
            "build": {"job": BUILD, "binary": str(BINARY), "binary_sha256": sha(BINARY)},
            "audio": {"narration": str(NARRATION), **audio}, "voice": H.VOICE, "cues": len(cues), "chapters": len(CHAPTERS),
            "facts_rows": facts, "direction": hold_facts, "checks": checks, "files": files,
            "claim": "Technical binding only. Nothing here was heard, rendered or accepted."}


def ready(a):
    body = gates(a)
    return {k: body[k] for k in ("cues", "chapters", "facts_rows", "direction")} | {
        "binary_sha256": body["build"]["binary_sha256"], "bound_files": len(body["files"]), "wrote": "nothing"}


def bind(a):
    need(not BINDING.exists(), f"{BINDING} exists: preserve it")
    H.write_new(BINDING, json.dumps(gates(a), indent=2) + "\n")
    return {"binding": str(BINDING), "binding_sha256": sha(BINDING)}


# ---------------------------------------------------------------- tables, stems and QA

def table(path):
    need(path.is_file() and path.stat().st_size > 0, f"missing table {path.name}")
    return [line.split("\t") for line in path.read_text().splitlines() if line and not line.startswith("#")]


def full_tables(output, cues, narration=NARRATION):
    """All three tables of all 13 chapters, each cue placed once, and the tables agreeing with each other and with
    the WAVs. Every rule here holds on stage09's real timeline02."""
    frames = {}
    for chapter in CHAPTERS:
        want = [c for c in cues if c.startswith(chapter.split("_")[0] + "_")]
        rows = table(output / f"{chapter}.timeline.tsv")
        lines = [r for r in rows if r[0] == "line"]
        listed, logged = table(output / f"{chapter}.cues.tsv"), table(output / f"{chapter}.cuelog.tsv")
        fps, total = int(H.first(rows, "fps")), int(H.first(rows, "frames"))
        need(H.first(rows, "chapter") == chapter and fps == 60, f"{chapter}: not this chapter at 60 fps")
        need([r[1] for r in lines] == [r[0] for r in listed] == want, f"{chapter}: the tables' cues are not the script's")
        need(len(logged) == len(want), f"{chapter}: the cue log has {len(logged)} rows for {len(want)} cues")
        starts = [int(r[2]) for r in lines]
        need(starts == sorted(starts), f"{chapter}: the cues are out of order")
        for line, entry, log in zip(lines, listed, logged):
            cue, wav = line[1], narration / f"{line[1]}.wav"
            need(abs(float(entry[1]) - int(line[2]) / fps) < 0.00051, f"{cue}: cues.tsv and the timeline disagree")
            need(Path(line[4]).resolve() == wav.resolve() == Path(log[0]).resolve(), f"{cue}: plays outside the narration")
            need(log[1] == sha(wav), f"{cue}: the cue log's hash is not the WAV's")
            need(int(log[3]) == int(line[2]) and int(log[4]) == fps, f"{cue}: the cue log's frame differs")
            need(int(line[2]) + float(entry[2]) * fps <= total, f"{cue}: runs past the chapter's end")
        frames[chapter] = total
    return frames


def wav_frames(path):
    """Stereo frames in a 48 kHz 32-bit float WAV, refusing a header that promises more than the file holds."""
    need(path.is_file(), f"missing stem {path.name}")
    size = path.stat().st_size
    with path.open("rb") as handle:
        head = handle.read(12)
        need(head[:4] == b"RIFF" and head[8:12] == b"WAVE", f"{path.name}: not a WAV")
        form = None
        while True:
            chunk = handle.read(8)
            need(len(chunk) == 8, f"{path.name}: no data chunk")
            name, length = chunk[:4], struct.unpack("<I", chunk[4:])[0]
            if name == b"data":
                need(form == (3, 2, 48000, 32), f"{path.name}: format {form}")
                need(handle.tell() + length <= size, f"{path.name}: the data is shorter than its header says")
                return length // 8
            if name == b"fmt ":
                body = handle.read(length)
                fields = struct.unpack("<HHIIHH", body[:16])
                # Stage09's real stems are WAVE_FORMAT_EXTENSIBLE: the format code opens the subformat GUID.
                tag = struct.unpack("<H", body[24:26])[0] if fields[0] == 0xFFFE else fields[0]
                form = (tag, fields[1], fields[2], fields[5])
                handle.seek(length & 1, 1)
            else:
                handle.seek(length + (length & 1), 1)


def stem_frames(stems, chapter):
    parts = stems / f"{chapter}.music.f32.parts.tsv"
    if not parts.is_file():
        return wav_frames(stems / f"{chapter}.music.f32.wav")
    rows = [line.split("\t") for line in parts.read_text().splitlines()]
    need(rows[0] == ["score-parts", "1"], f"{parts.name}: unknown format")
    at = 0
    for start, length, _, digest, name in rows[1:]:
        need(int(start) == at and sha(stems / name) == digest and wav_frames(stems / name) == int(length),
             f"{parts.name}: the part {name} is not what it lists")
        at += int(length)
    return at


def score_gate(stems, frames):
    """The stem each render will require (Kit::finish), and every QA metric of all 13 reports."""
    for chapter, count in frames.items():
        want = count * 800
        found = stem_frames(stems, chapter)
        need(found == want, f"{chapter}: the stem holds {found} samples; the chapter needs {want}")
        qa = json.loads((stems / "qa" / f"{chapter}.json").read_text())
        # null is a real value: two informational metrics in every stage09 report, and render-master01.py accepts it
        failed = [m["name"] for m in qa.get("metrics", []) if m.get("pass") not in (True, None)]
        need(qa.get("passed") is True and qa.get("metrics") and not failed, f"{chapter}: QA failed {failed}")
        length = [m["value"] for m in qa["metrics"] if m["name"] == "length"]
        need(length == [want], f"{chapter}: QA measured length {length}, not {want}")
    return len(frames)


def job_output(step):
    return LANE / H.job_name(step, SERIES)


def verify(step):
    result = H.verify_step(C, MM, SERIES, step)  # version 4, unchanged: receipt, script, log, summary, QA
    cues = cues_of()
    output = job_output("timeline") / "timeline-output"
    summary = json.loads((job_output("timeline") / "timeline-summary.json").read_text())
    for item in summary["chapters"]:
        need(sha(output / f"{item['chapter']}.timeline.tsv") == item["timeline_sha256"], f"{item['chapter']}: timeline moved")
    frames = full_tables(output, cues)
    result["tables"] = {"chapters": len(frames), "cues": len(cues), "total_frames": sum(frames.values())}
    if step == "score":
        result["score_qa_and_stems"] = score_gate(job_output("score") / "scored/stems", frames)
    return result


# ---------------------------------------------------------------- comparisons with stage09

def schedule(output):
    found = {}
    for chapter in CHAPTERS:
        rows = table(output / f"{chapter}.timeline.tsv")
        found[chapter] = {"frames": int(H.first(rows, "frames")), "lines": [(r[1], int(r[2])) for r in rows if r[0] == "line"],
                          "other": ["\t".join(r) for r in rows if r[0] not in ("chapter", "fps", "frames", "line")],
                          "wavs": {r[0].rsplit("/", 1)[1][:-4]: r[1] for r in table(output / f"{chapter}.cuelog.tsv")}}
    return found


def compare_timeline(before_dir, after_dir):
    """Actual deltas. Only a changed cue list or changed audio is a failure; moved frames are what this reports."""
    before, after = schedule(before_dir), schedule(after_dir)
    chapters, cues, hard = [], [], []
    start_before = start_after = 0
    for chapter in CHAPTERS:
        old, new = before[chapter], after[chapter]
        if [c for c, _ in old["lines"]] != [c for c, _ in new["lines"]]:
            hard.append(f"{chapter}: the cue list changed")
        if old["wavs"] != new["wavs"]:
            hard.append(f"{chapter}: cue audio changed")
        moved = 0
        for (cue, at_old), (_, at_new) in zip(old["lines"], new["lines"]):
            moved += at_old != at_new
            cues.append({"cue": cue, "chapter": chapter, "local_before": at_old, "local_after": at_new,
                         "local_delta": at_new - at_old, "film_before": start_before + at_old,
                         "film_after": start_after + at_new, "film_delta": start_after + at_new - start_before - at_old})
        delta = new["frames"] - old["frames"]
        chapters.append({"chapter": chapter, "frames_before": old["frames"], "frames_after": new["frames"], "delta_frames": delta,
                         "delta_seconds": round(delta / 60, 3), "predicted_delta": PREDICTED.get(chapter, 0),
                         "cues_moved_locally": moved,
                         "other_rows_removed": [r for r in old["other"] if r not in new["other"]],
                         "other_rows_added": [r for r in new["other"] if r not in old["other"]]})
        start_before += old["frames"]
        start_after += new["frames"]
    surprises = [c["chapter"] for c in chapters if c["delta_frames"] != c["predicted_delta"] or
                 (c["cues_moved_locally"] and c["chapter"] not in PREDICTED)]
    return {"prior": PRIOR["timeline"], "film": {"frames_before": start_before, "frames_after": start_after,
                                                 "delta_frames": start_after - start_before,
                                                 "delta_seconds": round((start_after - start_before) / 60, 3)},
            "chapters_changed": [c["chapter"] for c in chapters if c["delta_frames"] or c["cues_moved_locally"]],
            "not_as_predicted": surprises, "hard_failures": hard, "chapters": chapters, "cues": cues}


def stem_rows(stems, chapter):
    return {p.name: sha(p) for p in sorted(stems.glob(f"{chapter}.music.*"))}


def compare_score(before_stems, after_stems):
    chapters = []
    for chapter in CHAPTERS:
        old = json.loads((before_stems / "qa" / f"{chapter}.json").read_text())
        new = json.loads((after_stems / "qa" / f"{chapter}.json").read_text())
        values = {m["name"]: m["value"] for m in old["metrics"]}
        chapters.append({"chapter": chapter, "stem_byte_identical": stem_rows(before_stems, chapter) == stem_rows(after_stems, chapter),
                         "qa_changed": {m["name"]: [values.get(m["name"]), m["value"]] for m in new["metrics"]
                                        if values.get(m["name"]) != m["value"]}})
    read = lambda stems: (stems / "score.fingerprint").read_text().strip()
    return {"prior": PRIOR["score"], "hard_failures": [], "fingerprint_before": read(before_stems), "fingerprint_after": read(after_stems),
            "stems_changed": [c["chapter"] for c in chapters if not c["stem_byte_identical"]], "chapters": chapters}


def compare(step):
    need(not COMPARISON[step].exists(), f"{COMPARISON[step]} exists: preserve it")
    verify(step)
    if step == "timeline":
        result = compare_timeline(LANE / PRIOR[step] / "timeline-output", job_output(step) / "timeline-output")
    else:
        result = compare_score(LANE / PRIOR[step] / "scored/stems", job_output(step) / "scored/stems")
    result["scope"] = "Technical comparison of tables and files. No listening or viewing."
    H.write_new(COMPARISON[step], json.dumps(result, indent=2) + "\n")
    brief = {k: v for k, v in result.items() if k not in ("chapters", "cues", "scope")}
    if step == "timeline":
        brief["changed"] = [{k: c[k] for k in ("chapter", "frames_before", "frames_after", "delta_frames", "delta_seconds", "cues_moved_locally")}
                            for c in result["chapters"] if c["delta_frames"] or c["cues_moved_locally"]]
    need(not result["hard_failures"], f"the comparison found {result['hard_failures']}; see {COMPARISON[step]}")
    return {"comparison": str(COMPARISON[step]), "comparison_sha256": sha(COMPARISON[step]), **brief}


# ---------------------------------------------------------------- controllers

def seal(digest):
    sums = HERE.parent / "SHA256SUMS"
    need(sums.is_file() and sha(sums) == digest, f"this bundle's SHA256SUMS is not the seal {digest[:12]}")
    for line in sums.read_text().splitlines():
        expected, name = line.split(maxsplit=1)
        need(sha(HERE.parent / name.lstrip("*")) == expected, f"{name} differs from the seal")
    return {"seal": "PASS"}


def poll(job, needs):
    try:
        r = json.loads((LANE / job / "resource-receipt.json").read_text())
    except (FileNotFoundError, json.JSONDecodeError):  # the guard rewrites the receipt while a job runs
        return "waiting"
    if not r.get("finished"):
        return "waiting"
    if not H.receipt_ok(r):
        return "failed"
    if needs:
        try:
            if json.loads(Path(needs).read_text()).get("hard_failures"):
                return "failed"
        except (FileNotFoundError, json.JSONDecodeError):
            return "waiting"
    return "ready"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["seal", "poll", "ready", "bind", "step", "verify-step", "compare"])
    parser.add_argument("--sums")
    parser.add_argument("--job")
    parser.add_argument("--needs")
    parser.add_argument("--integration-sha256")
    parser.add_argument("--holds-sha256")
    parser.add_argument("--step", choices=["timeline", "score"])  # render and master are root's render-master01.py
    a = parser.parse_args(argv)
    if a.command in ("ready", "bind") and not (a.integration_sha256 and a.holds_sha256):
        parser.error(f"{a.command} needs root's approved --integration-sha256 and --holds-sha256")
    if a.command in ("step", "verify-step", "compare") and not a.step:
        parser.error(f"{a.command} needs --step")
    if a.command == "seal" and not a.sums or a.command == "poll" and not a.job:
        parser.error(f"{a.command} is missing its argument")
    if a.command == "poll":
        print(poll(a.job, a.needs))
        return 0
    try:
        if a.command == "seal":
            result = seal(a.sums)
        elif a.command == "ready":
            result = {"ready": "PASS", **ready(a)}
        elif a.command == "bind":
            result = {"bind": "WRITTEN", **bind(a)}
        elif a.command == "step":
            result = {"step": "WRITTEN", **H.write_step(C, MM, SERIES, a.step)}
        elif a.command == "verify-step":
            result = {"verify_step": "PASS", **verify(a.step)}
        else:
            result = {"compare": "WRITTEN", **compare(a.step)}
    except Refused as refusal:
        print(f"REFUSED: {refusal}")
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
