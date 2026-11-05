"""Rebind the logarithm audio metadata to root's final integrated source.

Run from the repository root:

    python3 -I -B <this file> --check    read-only: full dry run, prints the predicted hashes
    python3 -I -B <this file> --apply    root only: writes the outputs below
    python3 -I -B <this file> --verify   read-only: run after --apply, before freezing inputs

--apply refuses unless <stage>/integration.json approves the exact bytes in the workspace. It writes
docs/narration-direction-map-oliver17.json, the unit plan's map binding, holds-bound01.json,
spans01.json, narration_units-before-rebind01.json and rebind-report01.json. It never writes a WAV,
script.tsv or map16, and it never assembles, builds, renders or synthesizes.

The copied coherent_narration.py does the resolving and the unit, direction and disposition checks.
It hashes the map but not the source the map names, so a source edit after --apply passes assemble;
--verify is what catches that.

Exit codes: 0 pass; 1 refused; 2 usage; 3 originals valid, but integration.json is absent.
"""
import argparse
import copy
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

TOPIC = "logarithm_mysteries"
REL = f"explainer_corpus/{TOPIC}"
EL = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
STAGE = EL / "final-picture-repair01"
PREPARED = EL / "production-audio-stage09-01/prepared02"
OLD_AUDIT = EL / "combined-repair-stage09/bound-plan/hold-audit.json"
SCRIPT = f"{REL}/script.tsv"
UNITS = f"{REL}/narration_units.json"
MAP16 = f"{REL}/docs/narration-direction-map-oliver16.json"
MAP17 = f"{REL}/docs/narration-direction-map-oliver17.json"
TOOL = "tools/coherent_narration.py"
CACHE = "tools/narration_cache.py"
AUDIT_NAME = "holds-bound01.json"
SPANS_NAME = "spans01.json"
BEFORE_UNITS_NAME = "narration_units-before-rebind01.json"
REPORT_NAME = "rebind-report01.json"
PINS = {
    "initial_copy": "870dd265ba76b34cc79b9f98c26cc6c963a11b9d33074ca8bf3fdb2b029ba19c",
    "script": "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a",
    "units": "1132a5a452b339dd4d6fb340668c525b2e10dfe892d27035ce11957e80cafce9",
    "map16": "14e87893ea389ef2cdf827f0a7e59a8e2b65ac537b726d796ffceff5117e4787",
    "spans": "d45894df2d1137a8dc8caaa1b56cdb664db3005beba769a3200bdffe4bafaa6e",
    "audit": "f6f315d628d4f862ae9b372c0c44aea67dd4ff2e71f5c26c6b2513c9193cb1ac",
    "tool": "95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5",
}
COUNTS = {"files": 51, "cues": 80, "units": 80, "prediction": 13, "reading": 2, "helpers": 15, "finishes": 13}
READING_MARKERS = {
    "10_4": "kit.music_breath(stage, FINE_PRINT_S - read_for)?;",
    "11_20": "kit.music_breath(stage, rule_read + row_reads.iter().sum::<f64>())?;",
}
BAR_FADE = re.compile(r"^fade_out_all\(stage, &\[[^\]]*\bbar\b[^\]]*\bprobe\b[^\]]*\], 0\.25\)$")
SAY = re.compile(r'\bkit\.say\(stage, "(\d\d_\d+)"\)')
ANY_SAY = re.compile(r"\.say\(")
TIMING = re.compile(
    r"\b(predict_pause|predict|kit\.hold|kit\.music_breath|kit\.listen|kit\.sonify|kit\.mark|kit\.finish|"
    r"stage\.wait|wait_frames|wait_until)\("
)
RS_LINE = re.compile(r"\b([a-z_]+\.rs):(\d+)\b")
LIMITS = [
    "Source text and hashes only. Nothing was compiled, built, rendered, assembled, synthesized or timed by this helper.",
    "A marker in source is not evidence that a hold plays as intended; native timing stays open.",
    "No listening, fine-sync, visual or independent acceptance is claimed.",
]
MAP_SCOPE = (
    "Final-picture map for final-picture-repair01: 13 prediction holds and 2 reading holds, each bound to source "
    "bytes root approved in integration.json. Successor to oliver16: the same overrides, cues, kinds and order; "
    "hashes, lines, markers and intervals follow the integrated source. The editorial closing hold after 12_21 is "
    "recorded in holds-bound01.json and is not a row here. Authored coverage only: no silence is added to any take, "
    "and no compile, render, measured timing, listening or independent acceptance is claimed. ElevenLabs Oliver."
)
AUDIT_CLAIM = (
    "Where each hold's call sits in root's approved source, from source text and hashes only. No row adds silence "
    "to a take, and no WAV, script line or sentence sample changes."
)


class Refused(Exception):
    pass


def need(ok, message):
    if not ok:
        raise Refused(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fsha(path):
    return sha(Path(path).read_bytes())


def dump(obj):
    return (json.dumps(obj, indent=2) + "\n").encode()


def load_tool(root):
    """The copied tool, by file, so REPO is that tree and -I stays safe."""
    name = "coherent_narration_" + sha(str(root).encode())[:12]
    spec = importlib.util.spec_from_file_location(name, Path(root) / TOOL)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def tree_files(root):
    found = set()
    for folder, _, names in os.walk(root):
        for name in names:
            found.add(Path(folder, name).relative_to(root).as_posix())
    return found


def frozen(path):
    name = Path(path).name
    return (
        path.startswith("tools/")
        or name in ("script.tsv", "narration_units.json", "Cargo.toml", "Cargo.lock")
        or name.startswith("narration-direction-map-")
    )


def film_code(text):
    """(lines, scene): scene stops at the tests module and keeps the file's line numbers."""
    cut = text.find("\n#[cfg(test)]\nmod tests {")
    return text.split("\n"), text if cut < 0 else text[:cut]


def registered(chapters):
    src = (chapters / "mod.rs").read_text(encoding="utf-8")
    pairs = re.findall(r'key: "(\d\d_\w+)",\s*make: \|kit\| Box::new\((\w+)::', src)
    declared = set(re.findall(r"^(?:pub )?mod (\w+);", src, re.M))
    need(all(m in declared for _, m in pairs) and len({m for _, m in pairs}) == len(pairs), "the registry does not match the declared modules")
    need(
        sorted(m for _, m in pairs) == sorted(d for d in declared if d not in ("draw", "thumbnail")),
        "mod.rs declares a chapter the registry does not use",
    )
    return [f"{module}.rs" for _, module in pairs]


def census(root, cues):
    """Every say and timing call in the registered chapters' film code, by file and line."""
    chapters = Path(root) / REL / "src/chapters"
    says, sites, files = [], [], {}
    for name in registered(chapters):
        data = (chapters / name).read_bytes()
        lines, scene = film_code(data.decode("utf-8"))
        files[name] = {"sha256": sha(data), "lines": lines, "scene": scene}
        current = None
        for n, line in enumerate(scene.split("\n"), 1):
            if line.strip().startswith("//"):
                continue
            hit = SAY.search(line)
            if hit:
                current = hit.group(1)
                says.append({"file": name, "line": n, "cue": current})
            elif ANY_SAY.search(line):
                raise Refused(f"{name}:{n}: a say this helper cannot read: {line.strip()}")
            timing = TIMING.search(line)
            if timing and not re.search(r"\bfn (predict|wait_until)\(", line):
                sites.append({"file": name, "line": n, "call": timing.group(1), "marker": line.strip(), "cue": current})
    need([s["cue"] for s in says] == cues, "registered chapters do not say every script cue once, in order")
    return {"says": says, "sites": sites, "files": files}


def bound(found, site, cue, nxt, script, tool):
    """One hold's source facts; the interval rule is the binder's: say line through marker line."""
    name, line = site["file"], site["line"]
    marker = site["marker"]
    need(marker.endswith(";") and marker.count("(") == marker.count(")"), f"{cue}: {name}:{line} is not a whole one-line call: {marker}")
    say_at = {s["cue"]: s for s in found["says"]}
    start = say_at[cue]
    need(start["file"] == name and start["line"] < line, f"{name}:{line}: the hold is not after {cue}'s say")
    same_chapter = nxt is not None and nxt[:2] == cue[:2]
    next_say = None
    if same_chapter:
        next_say = say_at[nxt]["line"]
        need(say_at[nxt]["file"] == name and line < next_say, f"{name}:{line}: the hold is not before {nxt}'s say")
    lines = found["files"][name]["lines"]
    interval = "\n".join(lines[start["line"] - 1 : line])
    between = []
    if next_say is not None:
        for n in range(line + 1, next_say):
            code = lines[n - 1].strip()
            if code and not code.startswith("//"):
                between.append({"line": n, "code": code})
    return {
        "cue": cue,
        "next_cue": nxt,
        "next_cue_crosses_chapter": nxt is not None and not same_chapter,
        "file": name,
        "source_sha256": found["files"][name]["sha256"],
        "say_line": start["line"],
        "marker_line": line,
        "marker": marker,
        "next_say_line": next_say,
        "interval_sha256": sha(interval.encode("utf-8")),
        "cue_text_sha256": sha(script[cue].encode("utf-8")),
        "final_sentence": tool.sentences(script[cue])[-1],
        "code_between_marker_and_next_say": between,
    }


def derive_holds(found, rows, script, tool, closing_cue):
    """The 15 map holds as this source has them, plus any breath in the last cue's tail."""
    predicts = [s for s in found["sites"] if s["call"] == "predict"]
    breaths = [s for s in found["sites"] if s["call"] == "kit.music_breath"]
    want_predict = sorted(r["cue"] for r in rows if r["kind"] == "prediction")
    want_reading = sorted(r["cue"] for r in rows if r["kind"] == "reading")
    need(len(want_predict) == COUNTS["prediction"] and len(want_reading) == COUNTS["reading"], "map rows are not 13 prediction and 2 reading")
    got = sorted(s["cue"] for s in predicts)
    need(got == want_predict, f"predict calls moved, went or arrived: missing {sorted(set(want_predict) - set(got))}, extra {sorted(set(got) - set(want_predict))}, {len(got)} calls")
    reading = [s for s in breaths if s["cue"] in want_reading]
    closing = [s for s in breaths if s["cue"] == closing_cue]
    stray = [s for s in breaths if s not in reading and s not in closing]
    need(not stray, f"undocumented kit.music_breath at {[(s['file'], s['line'], s['cue']) for s in stray]}")
    need(sorted(s["cue"] for s in reading) == want_reading, "the reading holds are not one breath per reading cue")
    holds = {}
    for row in rows:
        site = next(s for s in predicts + reading if s["cue"] == row["cue"])
        need(Path(row["path"]).name == site["file"], f"{row['cue']}: the hold moved to {site['file']}")
        hold = bound(found, site, row["cue"], row["next_cue"], script, tool)
        if row["kind"] == "prediction":
            need(not hold["next_cue_crosses_chapter"], f"{row['cue']}: a prediction at a chapter's end")
            need(not hold["code_between_marker_and_next_say"], f"{row['cue']}: code between predict and the next say: {hold['code_between_marker_and_next_say']}")
        else:
            need(hold["marker"] == READING_MARKERS[row["cue"]], f"{row['cue']}: the reading marker changed: {hold['marker']}")
        holds[row["cue"]] = {"kind": row["kind"], **hold}
    return holds, closing


def constant_text(scene, pattern):
    hits = re.findall(pattern, scene, re.M | re.S)
    need(len(hits) == 1, f"pattern {pattern!r} matched {len(hits)} times")
    return hits[0]


def reading_inputs(found):
    """The literals the two reading rows' basis text rests on. Equal literals keep the basis true."""
    un = found["files"]["unreachable.rs"]["scene"]
    dn = found["files"]["doing_nothing.rs"]["scene"]
    return {
        "11_20": [
            constant_text(un, r"^const THEOREM: \[&str; 4\] = \[\n(.*?)^\];"),
            constant_text(un, r"^const READING_RATE: f64 = ([0-9.]+);"),
            constant_text(un, r"^(fn block_examples\(\).*?^})"),
        ],
        "10_4": [
            constant_text(dn, r'^const TEXTBOOK: &str = "([^"\\]*)";'),
            constant_text(dn, r'^const SOURCE: &str = "([^"\\]*)";'),
            constant_text(dn, r"^const FINE_PRINT_S: f64 = ([0-9.]+);"),
        ],
    }


def helper_rows(workspace, kit_repo, audit_rows):
    """The 15 helper markers of the old audit, found afresh; predict's body still runs in order."""
    rows = []
    for old in audit_rows:
        base = kit_repo if old["path"].startswith("crates/") else workspace
        data = (Path(base) / old["path"]).read_bytes()
        lines = data.decode("utf-8").split("\n")
        if old["marker"].startswith("fade_out_all(stage, &[bar, probe], "):
            hits = [n for n, line in enumerate(lines, 1) if BAR_FADE.match(line.strip())]
        else:
            hits = [n for n, line in enumerate(lines, 1) if line.strip() == old["marker"]]
        need(len(hits) == 1, f"{old['path']}: helper marker {old['marker']!r} is on {len(hits)} lines")
        rows.append({"path": old["path"], "sha256": sha(data), "line": hits[0], "marker": lines[hits[0] - 1].strip()})
    draw = next(r for r in rows if r["marker"].startswith("pub fn predict(kit: &Kit"))
    text = (Path(workspace) / draw["path"]).read_text(encoding="utf-8").split("\n")
    end = next(n for n in range(draw["line"], len(text)) if text[n] == "}")
    body = [line.strip() for line in text[draw["line"] : end]]
    order = [next((i for i, line in enumerate(body) if test(line)), None) for test in (
        lambda s: s == "kit.hold(stage)?;",
        lambda s: s == 'kit.mark(stage, "predict");',
        lambda s: s == "kit.music_breath(stage, THINK)?;",
        lambda s: BAR_FADE.match(s) is not None,
    )]
    need(None not in order and order == sorted(order), "draw::predict no longer holds, marks, breathes and fades in that order")
    return rows


def kit_repo_of(root):
    """The checkout the crate compiles the kit from, as Cargo.toml names it."""
    text = (Path(root) / REL / "Cargo.toml").read_text(encoding="utf-8")
    hit = re.search(r'explainer_kit\s*=\s*\{[^}]*path\s*=\s*"([^"]+)"', text)
    need(hit is not None, "Cargo.toml has no path dependency on explainer_kit")
    return Path(hit.group(1)).parents[1]


def load_originals(stage, original):
    raw = (stage / "initial-copy.json").read_bytes()
    need(sha(raw) == PINS["initial_copy"], "initial-copy.json changed")
    initial = json.loads(raw)
    rows = {r["path"]: r["sha256"] for r in initial["files"]}
    need(len(rows) == COUNTS["files"], "initial-copy.json does not list 51 files")
    ws = Path(original or initial["source"])
    for path, digest in rows.items():
        need(fsha(ws / path) == digest, f"original {path} does not match initial-copy.json")
    for key, path in (("script", SCRIPT), ("units", UNITS), ("map16", MAP16), ("tool", TOOL)):
        need(fsha(ws / path) == PINS[key], f"original {path} is not the pinned file")
    map16 = json.loads((ws / MAP16).read_text(encoding="utf-8"))
    units = json.loads((ws / UNITS).read_text(encoding="utf-8"))
    need(dump(map16) == (ws / MAP16).read_bytes() and dump(units) == (ws / UNITS).read_bytes(), "map16 or units would not round-trip byte for byte")
    spans_raw = (PREPARED / "spans.json").read_bytes()
    need(sha(spans_raw) == PINS["spans"], "prepared02/spans.json is not the pinned file")
    spans = json.loads(spans_raw)
    need(dump(spans) == spans_raw, "prepared02/spans.json would not round-trip byte for byte")
    reference = json.loads((PREPARED / "narration/report.json").read_text(encoding="utf-8"))
    need(reference["meta"]["spans_sha256"] == PINS["spans"] and reference["meta"]["units_sha256"] == PINS["units"], "prepared02's report does not bind the pinned spans and units")
    audit_raw = OLD_AUDIT.read_bytes()
    need(sha(audit_raw) == PINS["audit"], "the stage09 hold audit is not the pinned file")
    return {"initial": rows, "ws": ws, "map16": map16, "units": units, "spans": spans, "reference": reference, "audit": json.loads(audit_raw)}


def check_originals(o):
    """Every map16 hold and helper hash against the original workspace, through the copied tool."""
    tool = load_tool(o["ws"])
    script_path = o["ws"] / SCRIPT
    script, units = tool.read_units(script_path, o["ws"] / UNITS, True)
    cues = list(script)
    need(len(cues) == COUNTS["cues"] and len(units) == COUNTS["units"], "the original plan is not 80 units over 80 cues")
    m = o["map16"]
    rows = m["visual_holds"]
    need(m["hold_kinds"] == {"prediction": COUNTS["prediction"], "reading": COUNTS["reading"]}, "map16's hold kinds changed")
    need(len(rows) == len(m["overrides"]) == 15 and [r["cue"] for r in rows] == [r["cue"] for r in m["overrides"]], "map16's holds and overrides are not one to one")
    for row in rows:
        need(row["next_cue"] == cues[cues.index(row["cue"]) + 1], f"{row['cue']}: next cue differs from the script")
    resolved = tool.direction_contract(script_path, o["units"])["resolved"]
    need(len(resolved) == 15, "the copied tool did not resolve 15 directions")
    found = census(o["ws"], cues)
    holds, closing = derive_holds(found, rows, script, tool, cues[-1])
    need(not closing, "the original source has a closing breath")
    for row in rows:
        hold = holds[row["cue"]]
        need(row["path"] == f"{REL}/src/chapters/{hold['file']}" and row["evidence"] == {"path": row["path"], "sha256": row["sha256"]}, f"{row['cue']}: map16 row path or evidence differs")
        need((row["sha256"], row["line"], row["marker"], row["interval_sha256"]) == (hold["source_sha256"], hold["marker_line"], hold["marker"], hold["interval_sha256"]), f"{row['cue']}: map16 does not match the original source")
    audit = o["audit"]
    need({r["cue"]: r["interval_sha256"] for r in audit["prediction_holds"] + audit["reading_holds"]} == {r["cue"]: r["interval_sha256"] for r in rows}, "the stage09 audit and map16 disagree on intervals")
    kit_repo = kit_repo_of(o["ws"])
    helpers = helper_rows(o["ws"], kit_repo, audit["helpers"])
    need(len(helpers) == COUNTS["helpers"] and helpers == [{k: r[k] for k in ("path", "sha256", "line", "marker")} for r in audit["helpers"]], "the original helpers differ from the stage09 audit")
    for row in m["shared_helpers"]:
        base = kit_repo if row["path"].startswith("crates/") else o["ws"]
        need(fsha(Path(base) / row["path"]) == row["sha256"], f"shared helper {row['path']} changed")
    finishes = [s for s in found["sites"] if s["call"] == "kit.finish"]
    need(len(finishes) == COUNTS["finishes"], "the original film does not finish 13 chapters")
    return {"tool": tool, "cues": cues, "script": script, "found": found, "holds": holds, "kit_repo": kit_repo, "helpers": helpers}


def scan(workspace, initial, mode):
    """Current hash of every file, refusing a file the initial copy does not list."""
    allowed = set(initial) | ({MAP17} if mode == "verify" else set())
    extra = sorted(tree_files(workspace) - allowed)
    need(not extra, f"files outside the 51 copied: {extra[:5]}")
    missing = sorted(p for p in initial if not (workspace / p).is_file())
    need(not missing, f"copied files are missing: {missing[:5]}")
    return {p: fsha(workspace / p) for p in initial}


def gate(stage, workspace, initial, current):
    """Root's integration.json must approve exactly the bytes present. Returns (data, sha, changed rows)."""
    path = stage / "integration.json"
    need(path.is_file(), "integration.json is absent: nothing may be written")
    raw = path.read_bytes()
    data = json.loads(raw)
    need(data.get("approved_for_guarded_build") is True, "integration.json does not approve this source")
    need(data.get("script_sha256") == PINS["script"], "integration.json names a different script")
    need(data.get("audio_changed") is False, "integration.json does not say the audio is unchanged")
    need(Path(data.get("source_workspace", "")).resolve() == workspace.resolve(), "integration.json names another workspace")
    rows = data.get("all_pre_rebind_files")
    need(isinstance(rows, list) and len(rows) == COUNTS["files"], "all_pre_rebind_files is not 51 rows")
    by = {r["relative_path"]: r for r in rows}
    need(len(by) == len(rows) and set(by) == set(initial), "all_pre_rebind_files is not the 51 copied files")
    for rel, row in by.items():
        need(Path(row["path"]).resolve() == (workspace / rel).resolve(), f"{rel}: the row names another path")
        need(row["before_sha256"] == initial[rel], f"{rel}: approved before-hash differs from the initial copy")
        need(row["sha256"] == current[rel], f"{rel}: the workspace differs from the approved hash")
    actual = sorted(p for p in initial if current[p] != initial[p])
    need(sorted(data.get("changed_files", [])) == actual, f"changed_files differs from the workspace: {actual}")
    bad = [p for p in actual if frozen(p)]
    need(not bad, f"changed files that must not change: {bad}")
    need(all(r == by.get(r["relative_path"]) for r in data.get("files", [])), "a files row differs from all_pre_rebind_files")
    rust = {p for p in by if p.endswith(".rs")}
    need({r["relative_path"] for r in data.get("files", [])} == rust, "files is not every Rust file")
    return data, sha(raw), [{"path": p, "before_sha256": initial[p], "sha256": current[p]} for p in actual]


def check_closing(doc, sites, found, last_cue):
    """The editorial closing hold: documented exactly, at the film's end, and never a map row."""
    need(doc is not None or not sites, "kit.music_breath after the last cue is not documented")
    if doc is None:
        return None
    need(len(sites) == 1, f"editorial_closing_hold needs one breath in {last_cue}, found {len(sites)}")
    site = sites[0]
    need(doc.get("cue") == last_cue == site["cue"], "editorial_closing_hold names the wrong cue")
    need(doc.get("source") == f"{REL}/src/chapters/{site['file']}", "editorial_closing_hold names the wrong file")
    need(doc.get("marker") == site["marker"], "editorial_closing_hold names a different marker")
    need(doc.get("after_complete_recording") is True, "editorial_closing_hold is not after the complete recording")
    need(doc.get("prediction_or_reading_direction_row") is False, "editorial_closing_hold claims a map row")
    need(doc.get("native_timing_verified") is False, "editorial_closing_hold claims timing no build has measured")
    entry = found["files"][site["file"]]
    lines, scene = entry["lines"], entry["scene"]
    hold_name = re.fullmatch(r"kit\.music_breath\(stage, ([A-Z_]+)\)\?;", site["marker"])
    need(hold_name is not None, "the closing marker does not name a constant")
    say = next(s for s in found["says"] if s["cue"] == last_cue)
    need(say["file"] == site["file"] and say["line"] < site["line"], "the closing breath is not after the last say")
    finish = [s for s in found["sites"] if s["call"] == "kit.finish" and s["file"] == site["file"]]
    need(len(finish) == 1 and site["line"] < finish[0]["line"], "the closing breath is not before the chapter's finish")
    tail = [line.strip() for line in lines[site["line"] : finish[0]["line"] - 1] if line.strip() and not line.strip().startswith("//")]
    fade_name = re.fullmatch(r"fade_all\(stage, ([A-Z_]+)\)\?;", tail[0]) if len(tail) == 1 else None
    need(fade_name is not None, f"between the closing breath and the finish, only one fade is allowed: {tail}")
    hold_s = float(constant_text(scene, rf"^const {hold_name.group(1)}: f64 = ([0-9.]+);"))
    fade_s = float(constant_text(scene, rf"^const {fade_name.group(1)}: f64 = ([0-9.]+);"))
    need(doc.get("hold_seconds") == hold_s, f"hold_seconds differs from the source's {hold_s}")
    need(doc.get("fade_seconds") == fade_s, f"fade_seconds differs from the source's {fade_s}")
    interval = "\n".join(lines[say["line"] - 1 : site["line"]])
    return {
        "cue": last_cue,
        "file": site["file"],
        "source_sha256": entry["sha256"],
        "say_line": say["line"],
        "marker_line": site["line"],
        "marker": site["marker"],
        "finish_line": finish[0]["line"],
        "interval_sha256": sha(interval.encode("utf-8")),
        "hold_seconds": hold_s,
        "fade_seconds": fade_s,
        "fade_marker": tail[0],
        "after_complete_recording": True,
        "prediction_or_reading_direction_row": False,
        "native_timing_verified": False,
        "note": "Separate from the 15 map holds: it has no next cue, so it can be neither a map row nor a disposition.",
    }


def other_wait_diff(before, after):
    """Non-hold waits added or removed, by file and marker. Reported, not judged."""
    def bag(found):
        counts = {}
        for s in found["sites"]:
            if s["call"] not in ("predict", "kit.music_breath", "kit.finish"):
                key = f"{s['file']}: {s['marker']}"
                counts[key] = counts.get(key, 0) + 1
        return counts
    a, b = bag(before), bag(after)
    return {
        "added": sorted(k for k in b if b[k] > a.get(k, 0)),
        "removed": sorted(k for k in a if a[k] > b.get(k, 0)),
    }


def build(o, base, stage, workspace, data, integration_sha, changed):
    """Every output as bytes, from the originals and the approved source. Writes nothing."""
    tool, cues, script = base["tool"], base["cues"], base["script"]
    found = census(workspace, cues)
    m16 = o["map16"]
    rows = m16["visual_holds"]
    need("editorial_closing_hold" in data, "integration.json has no editorial_closing_hold key")
    holds, closing_sites = derive_holds(found, rows, script, tool, cues[-1])
    closing = check_closing(data["editorial_closing_hold"], closing_sites, found, cues[-1])
    need(len([s for s in found["sites"] if s["call"] == "kit.finish"]) == COUNTS["finishes"], "the film no longer finishes 13 chapters")
    before_reading, after_reading = reading_inputs(base["found"]), reading_inputs(found)
    for cue in READING_MARKERS:
        need(before_reading[cue] == after_reading[cue], f"{cue}: the literals behind the reading basis changed; map17's basis needs root's review")
    helpers = helper_rows(workspace, base["kit_repo"], o["audit"]["helpers"])

    map17 = copy.deepcopy(m16)
    for row in map17["visual_holds"]:
        hold = holds[row["cue"]]
        row.update(sha256=hold["source_sha256"], line=hold["marker_line"], marker=hold["marker"], interval_sha256=hold["interval_sha256"])
        row["evidence"] = {"path": row["path"], "sha256": hold["source_sha256"]}
    for row in map17["shared_helpers"]:
        source = base["kit_repo"] if row["path"].startswith("crates/") else workspace
        row["sha256"] = fsha(Path(source) / row["path"])
    map17["scope"] = MAP_SCOPE
    map17["predecessor"] = {"path": MAP16, "sha256": PINS["map16"]}
    map_bytes = dump(map17)
    need(map17["overrides"] == m16["overrides"] and map17["scripts"] == m16["scripts"] and map17["hold_kinds"] == m16["hold_kinds"], "the map's direction content changed")

    units = copy.deepcopy(o["units"])
    units["direction_contract"]["map"] = {"path": MAP17, "sha256": sha(map_bytes)}
    units_bytes = dump(units)

    audit_holds = []
    for row in rows:
        hold = holds[row["cue"]]
        before = base["holds"][row["cue"]]
        keys = ("source_sha256", "say_line", "marker_line", "marker", "next_say_line", "interval_sha256")
        audit_holds.append({
            "cue": row["cue"], "next_cue": row["next_cue"], "kind": row["kind"], "file": hold["file"],
            "next_cue_crosses_chapter": hold["next_cue_crosses_chapter"],
            "cue_text_sha256": hold["cue_text_sha256"], "final_sentence": hold["final_sentence"],
            "before": {k: before[k] for k in keys}, "after": {k: hold[k] for k in keys},
            "changed": [k for k in keys if before[k] != hold[k]],
            "code_between_marker_and_next_say": hold["code_between_marker_and_next_say"],
        })
    dispositions = copy.deepcopy(o["spans"]["direction_dispositions"])
    need([d["cue"] for d in dispositions] == [r["cue"] for r in rows], "dispositions and holds are not in the same order")
    for row in dispositions:
        hold = holds[row["cue"]]
        cited = RS_LINE.findall(row["reason"])
        need(len(cited) == 1 and cited[0][0] == hold["file"], f"{row['cue']}: the disposition does not cite its hold once")
        row["reason"] = RS_LINE.sub(f"{hold['file']}:{hold['marker_line']}", row["reason"])
    audit = {
        "schema_version": 1, "topic": TOPIC, "claim": AUDIT_CLAIM, "limits": LIMITS,
        "script_sha256": PINS["script"],
        "integration_sha256": integration_sha,
        "map": {"path": MAP17, "sha256": sha(map_bytes)},
        "predecessor_map": {"path": MAP16, "sha256": PINS["map16"]},
        "predecessor_audit_sha256": PINS["audit"],
        "kit": o["audit"]["kit"],
        "counts": {
            "cues": len(cues), "prediction_holds": COUNTS["prediction"], "reading_holds": COUNTS["reading"],
            "holds_with_a_changed_marker": sum("marker" in h["changed"] for h in audit_holds),
            "holds_with_a_changed_interval": sum("interval_sha256" in h["changed"] for h in audit_holds),
            "holds_with_a_changed_line": sum("marker_line" in h["changed"] for h in audit_holds),
        },
        "holds": audit_holds,
        "editorial_closing_hold": closing,
        "other_waits_diff": other_wait_diff(base["found"], found),
        "helpers": helpers,
        "changed_files": changed,
        "dispositions": [{k: d[k] for k in ("override_index", "cue", "kind", "reason")} for d in dispositions],
    }
    audit_bytes = dump(audit)

    evidence_path = os.path.relpath(stage / AUDIT_NAME)
    for row in dispositions:
        row["evidence"] = {"path": evidence_path, "sha256": sha(audit_bytes)}
    spans = {**o["spans"], "units_sha256": sha(units_bytes), "direction_dispositions": dispositions}
    spans_bytes = dump(spans)
    return {"map": map_bytes, "units": units_bytes, "audit": audit_bytes, "spans": spans_bytes, "audit_obj": audit}


def tool_verify(tool, script_path, units_path, spans, o):
    """All 80 units and 15 directions through the copied tool, then sample equality with prepared02."""
    script, units = tool.read_units(script_path, units_path, True)
    need(len(units) == COUNTS["units"], "the plan is not 80 units")
    contract = tool.direction_contract(script_path, json.loads(Path(units_path).read_text(encoding="utf-8")))
    directions = tool.review_directions(contract, units, spans)
    records = spans["units"]
    need([r["id"] for r in records] == [u["id"] for u in units], "spans omit, repeat or reorder units")
    need(records == o["spans"]["units"] and spans["boundary_review"] == o["spans"]["boundary_review"], "spans changed outside metadata")
    tool.assembly_voice(records)
    tool.check_boundary_review(spans["boundary_review"])
    lines, audio = {}, {}
    for unit, record in zip(units, records):
        for cue, row, pcm in tool.unit_spans(unit, script, record):
            lines.setdefault(cue, []).append(row)
            audio[cue] = audio.get(cue, b"") + pcm
    tool.check_dispositions(directions, lines)
    need(lines == o["reference"]["lines"], "sentence rows differ from prepared02's report")
    for cue, pcm in audio.items():
        need(tool.wav_pcm(PREPARED / f"narration/{cue}.wav")[1] == pcm, f"{cue}: samples differ from prepared02")
    return {
        "units": len(units), "cues": len(audio), "directions_resolved": len(contract["resolved"]),
        "dispositions": len(directions["dispositions"]), "sentence_rows": sum(len(v) for v in lines.values()),
        "cue_rows_equal_prepared02": len(lines), "cue_samples_equal_prepared02": len(audio),
    }


def dry_run(stage, workspace, outputs, o):
    """Run the copied tool on a throwaway tree, so a refusal comes before anything is written."""
    base = Path(tempfile.mkdtemp(prefix="logarithm-rebind-"))
    for name in (TOOL, CACHE):
        (base / name).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(workspace / name, base / name)
    (base / MAP17).parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(workspace / SCRIPT, base / SCRIPT)
    (base / MAP17).write_bytes(outputs["map"])
    (base / UNITS).write_bytes(outputs["units"])
    (base / AUDIT_NAME).write_bytes(outputs["audit"])
    spans = json.loads(outputs["spans"])
    for row in spans["direction_dispositions"]:
        row["evidence"]["path"] = str(base / AUDIT_NAME)
    return tool_verify(load_tool(base), base / SCRIPT, base / UNITS, spans, o)


def tool_check_cli(workspace):
    done = subprocess.run(
        [sys.executable, "-I", "-B", str(workspace / TOOL), "check", "--script", str(workspace / SCRIPT), "--units", str(workspace / UNITS), "--require-directions"],
        capture_output=True, text=True,
    )
    need(done.returncode == 0, f"the copied tool's check failed: {done.stderr.strip()}")
    return json.loads(done.stdout)


def report(mode, integration_sha, changed, outputs, verification, cli):
    return {
        "schema_version": 1, "topic": TOPIC, "mode": mode, "limits": LIMITS,
        "integration_sha256": integration_sha,
        "approved_changed_files": changed,
        "counts": outputs["audit_obj"]["counts"],
        "editorial_closing_hold": outputs["audit_obj"]["editorial_closing_hold"],
        "other_waits_diff": outputs["audit_obj"]["other_waits_diff"],
        "verification": verification,
        "tool_check_cli": cli,
        "outputs_sha256": {
            MAP17: sha(outputs["map"]), UNITS: sha(outputs["units"]),
            AUDIT_NAME: sha(outputs["audit"]), SPANS_NAME: sha(outputs["spans"]),
        },
    }


def write_new(path, data):
    with open(path, "xb") as handle:
        handle.write(data)


def apply(stage, workspace, outputs, o, integration_sha, changed):
    targets = [workspace / MAP17, stage / AUDIT_NAME, stage / SPANS_NAME, stage / BEFORE_UNITS_NAME, stage / REPORT_NAME]
    present = [p.name for p in targets if p.exists()]
    need(not present, f"already present, preserved: {present}")
    dry_run(stage, workspace, outputs, o)
    # the approved bytes again, immediately before the first write
    current = scan(workspace, o["initial"], "apply")
    gate(stage, workspace, o["initial"], current)
    write_new(workspace / MAP17, outputs["map"])
    write_new(stage / BEFORE_UNITS_NAME, (workspace / UNITS).read_bytes())
    temporary = workspace / REL / "narration_units.json.rebind"
    write_new(temporary, outputs["units"])
    os.replace(temporary, workspace / UNITS)
    write_new(stage / AUDIT_NAME, outputs["audit"])
    write_new(stage / SPANS_NAME, outputs["spans"])
    verification = tool_verify(load_tool(workspace), workspace / SCRIPT, workspace / UNITS, json.loads(outputs["spans"]), o)
    done = report("apply", integration_sha, changed, outputs, verification, tool_check_cli(workspace))
    write_new(stage / REPORT_NAME, dump(done))
    return done


def verify(stage, workspace, outputs, o, integration_sha, changed):
    expected = {
        workspace / MAP17: outputs["map"], workspace / UNITS: outputs["units"], stage / AUDIT_NAME: outputs["audit"],
        stage / SPANS_NAME: outputs["spans"],
    }
    for path, data in expected.items():
        need(path.is_file() and path.read_bytes() == data, f"{path.name} differs from what the approved source gives")
    need(fsha(stage / BEFORE_UNITS_NAME) == PINS["units"], "the preserved unit plan changed")
    verification = tool_verify(load_tool(workspace), workspace / SCRIPT, workspace / UNITS, json.loads(outputs["spans"]), o)
    return report("verify", integration_sha, changed, outputs, verification, tool_check_cli(workspace))


def run(args):
    stage = Path(args.stage)
    workspace = stage / "workspace"
    need((Path.cwd() / PREPARED).is_dir(), "run from the repository root")
    o = load_originals(stage, args.original)
    base = check_originals(o)
    mode = "check" if args.check else "apply" if args.apply else "verify"
    current = scan(workspace, o["initial"], mode)
    if mode == "verify":
        need((stage / BEFORE_UNITS_NAME).is_file(), "--verify needs a finished --apply")
        current[UNITS] = PINS["units"]
    elif mode == "check" and not (stage / "integration.json").is_file():
        changed = sorted(p for p in o["initial"] if current[p] != o["initial"][p])
        print(json.dumps({"state": "originals valid; integration.json absent", "changed_files": changed}, indent=2))
        return 3
    data, integration_sha, changed = gate(stage, workspace, o["initial"], current)
    outputs = build(o, base, stage, workspace, data, integration_sha, changed)
    if mode == "check":
        done = report("check", integration_sha, changed, outputs, dry_run(stage, workspace, outputs, o), None)
    elif mode == "apply":
        done = apply(stage, workspace, outputs, o, integration_sha, changed)
    else:
        done = verify(stage, workspace, outputs, o, integration_sha, changed)
    print(json.dumps(done, indent=2))
    return 0


def main():
    parser = argparse.ArgumentParser(description="Rebind the logarithm audio metadata to the final source.")
    how = parser.add_mutually_exclusive_group(required=True)
    how.add_argument("--check", action="store_true", help="read-only full dry run")
    how.add_argument("--apply", action="store_true", help="root only: write the outputs")
    how.add_argument("--verify", action="store_true", help="read-only: outputs equal what the approved source gives")
    parser.add_argument("--stage", default=str(STAGE), help="stage folder (tests pass a scratch copy)")
    parser.add_argument("--original", help="tests only: a copy of the original workspace")
    args = parser.parse_args()
    try:
        return run(args)
    except (Refused, OSError, ValueError, KeyError, TypeError) as error:
        print(f"rebind_audio: refused: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
