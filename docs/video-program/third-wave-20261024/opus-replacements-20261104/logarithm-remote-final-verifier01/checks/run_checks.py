"""Checks for verify_remote_final.py. From the repository root, where FRESH_SCRATCH does not exist:

    python3 -I -B .../logarithm-remote-final-verifier01/checks/run_checks.py FRESH_SCRATCH

Read-only on the repository; writes only under FRESH_SCRATCH. No SSH, no guard, no real ffmpeg or ffprobe, no strict
decode, no render, no synthesis, no listening and no viewing. It runs on the local host, so every worker fact is a
STAND-IN: the helper's HOST is patched in this process for the fixture, after the real value is shown to refuse here.
The stage09 records are real and validate the helper's behaviour only: they accept nothing about a new master.

  G1  the helper against the sealed final-picture verifier, definition by definition; anchors.json byte-identical
  G2  real stage09 data, no anchors: the plan equals the sealed 1356 pictures, frames, seconds and reasons
  G3  anchors on the real stage09 schedule: every old frame comes back; the required sites are all present
  G4  the plan with anchors keeps all old frames and reasons, and names the anchor first
  G5  a SYNTHETIC schedule (real timelines with chapter 04 +24 and chapter 12 +420 and a closing breath): anchors move
      with it, and `was` changes nothing
  G6  planted refusals of the anchor resolver
  G7  the 15 real holds and the closing arithmetic, with planted faults
  G8  a whole STAND-IN fixture in the remote layout: passes, and refuses each planted fault, the tool pins included
  G9  the command line and run_job with STAND-IN ffmpeg, ffprobe and strict check: no media is read or written
  G10 no home or scratch path in the delivered files
  G11 root's real remote records: build manifest, guard pins, transfer lists and the two files still to transfer
"""
import argparse
import ast
from collections import Counter
import contextlib
import copy
import difflib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import socket
import stat
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import stage09_data as D  # noqa: E402

REPO = Path.cwd()
FOLDER = (REPO / D.HELPER).parent
HELPER = REPO / D.HELPER
BASE_DIR = REPO / D.P / "logarithm-final-picture-verifier01"
BASE = BASE_DIR / "verify_final_picture.py"
REMOTE = REPO / D.EL / "remote-mac-mini-max01"
RENDER_MASTER = REPO / D.S / "render-master01.py"
SHIFT = {"04_undoing_growth": 24, "12_what_it_forgets": 420}
CLOSING_LOCAL = 9837


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def whole_diff():
    """docs/final-picture-to-remote.diff, recomputed, so that the delivered diff cannot drift from the helper."""
    return "".join(difflib.unified_diff(BASE.read_text().splitlines(True), HELPER.read_text().splitlines(True), n=3,
                                        fromfile="logarithm-final-picture-verifier01/verify_final_picture.py",
                                        tofile="logarithm-remote-final-verifier01/verify_remote_final.py"))


def definitions(path):
    found = {}
    for node in ast.parse(Path(path).read_text()).body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            found[node.name] = ast.dump(node)
        elif isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name):
            found[node.targets[0].id] = ast.dump(node)
    return found


def synthetic_timeline(text, chapter, narration=None):
    """A real stage09 timeline with the pipeline's predicted growth. Synthetic: no such timeline exists yet."""
    out = []
    for raw in text.split("\n"):
        f = raw.split("\t")
        if f[0] == "frames":
            f[1] = str(int(f[1]) + SHIFT.get(chapter, 0))
        if f[0] == "line" and narration is not None:
            f[4] = str(Path(narration) / f"{f[1]}.wav")
        out.append("\t".join(f))
    body = "\n".join(out)
    if chapter == "12_what_it_forgets":
        body = body.rstrip("\n") + f"\nbreath\t{CLOSING_LOCAL}\t5.5\n"
    return body


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("scratch")
    a = parser.parse_args()
    if os.path.exists(a.scratch):
        print("FRESH_SCRATCH must not exist", file=sys.stderr)
        return 2
    scratch = Path(a.scratch)
    scratch.mkdir(parents=True)
    V = D.helper()
    data = D.load(V)
    results = []

    def clean(text):
        for path, label in ((str(scratch.resolve()), "SCRATCH"), (str(scratch), "SCRATCH"), (str(REPO), "REPO")):
            text = text.replace(path, label)
        return text

    def check(name, ok, detail=""):
        results.append(bool(ok))
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {clean(detail)}" if detail else ""), flush=True)

    def refusal(fn, *args):
        try:
            fn(*args)
        except V.Refused as error:
            return clean(str(error))
        return None

    sums = {line.split()[1].lstrip("*"): line.split()[0] for line in (BASE_DIR / "SHA256SUMS").read_text().splitlines()}
    print(f"helper sha256 {sha(HELPER)}; anchors sha256 {sha(FOLDER / 'anchors.json')}")
    print(f"base helper sha256 {sha(BASE)}; root's render-master01.py as read sha256 {sha(RENDER_MASTER)}")
    sealed_ok = all(sha(BASE_DIR / rel) == digest for rel, digest in sums.items()) and sha(BASE_DIR / "SHA256SUMS").startswith("deb12d24")

    # ------------------------------------------------------------------ G1
    old, new = definitions(BASE), definitions(HELPER)
    same = sorted(k for k in old if k in new and old[k] == new[k])
    changed = sorted(k for k in old if k in new and old[k] != new[k])
    added = sorted(k for k in new if k not in old)
    removed = sorted(k for k in old if k not in new)
    want_changed = ["BINDING_SCHEMA", "Ctx", "JOB_TEMPLATE", "PINS", "SCOPE", "STAGE", "bind_master", "check", "gather", "main", "read_schedule", "run_job", "verify_job", "write_job"]
    want_added = ["HOST", "MIN_PILLOW", "O_REL", "REMOTE_GUARD_PINS_REL", "REMOTE_GUARD_REL", "RENDER_ATTEMPT", "TOOLS_SCHEMA", "check_pillow", "check_tool_path", "check_tools",
                  "on_host", "pillow_version", "tool_path", "tool_row", "write_tools"]
    must_keep = ["parse_timeline", "schedule", "layout_anchors", "parse_captions", "sentence_onsets", "wav_seconds", "read_script", "registry",
                 "check_captions", "check_phrases", "media_facts", "loudness_summary", "make_sheets", "decode_lines",
                 "software_decode_clean", "receipt_ok", "ran", "last_line", "EXPECT", "OUTPUTS", "CHAPTERS", "CUES", "TIMELINE_SUCCESS", "SCORE_SUCCESS",
                 "RENDER_SUCCESS", "MASTER_SUCCESS", "VERIFY_SUCCESS", "SOFTWARE_DECODE", "check_binding", "check_sources", "check_audio",
                 "check_map", "hold_windows", "closing_window", "anchor_places", "resolve_anchors", "anchor_ledger", "plan_samples", "frames_covering",
                 "read_config", "RETIRED_MASTER", "RETIRED_BINARY", "SCRIPT_SHA256", "STAGE09_SOURCES", "HOLD_KINDS", "DIRECTION_MAP"]
    delivered_diff = (FOLDER / "docs/final-picture-to-remote.diff").read_text()
    frozen_length = "116" + "245"  # the retired film's length, spelled so that this file is not a false hit of the check below
    base_pins = module("base_helper_for_pins", BASE).PINS
    check("G1-definitions", changed == want_changed and added == want_added and removed == ["tool"] and all(k in same for k in must_keep)
          and delivered_diff == whole_diff() and sealed_ok and sha(FOLDER / "anchors.json") == sums["anchors.json"]
          and base_pins.items() <= V.PINS.items() and frozen_length not in HELPER.read_text() and frozen_length not in (FOLDER / "anchors.json").read_text(),
          f"the sealed verifier still verifies ({len(sums)} files); anchors.json is byte-identical to it; {len(same)} definitions are identical to it, all parsers, "
          f"measurements, sheets, gates, plan and resolver among them; {len(changed)} changed: {changed}; {len(added)} added: {added}; removed: {removed} (PATH lookup, "
          f"replaced by pins); every earlier pin stays and {len(V.PINS) - len(base_pins)} are added; docs/final-picture-to-remote.diff is this change exactly; the retired "
          f"film's frame count is nowhere in the helper or the anchors")

    # ------------------------------------------------------------------ G2
    args = (data["chapters"], data["placements"], data["timelines"], data["script"], data["onsets"], data["holds"], data["marks"], data["layouts"])
    plan = V.plan_samples(*args)
    sealed = [{k: p[k] for k in ("frame", "seconds", "reasons")} for p in data["v"]["pictures"]]
    check("G2-real-plan-replay", plan == sealed and len(plan) == 1356,
          f"on the real stage09 timelines, narration report, phrase marks, oliver16 and layouts, with no anchors, the plan equals all "
          f"{len(sealed)} sealed pictures (frame, seconds, every reason, in order)")

    # ------------------------------------------------------------------ G3
    doc = json.loads((FOLDER / "anchors.json").read_text())
    rows = doc["rows"]
    total = sum(x["frames"] for x in data["chapters"])
    sites, stray = V.hold_windows(data["chapters"], data["placements"], data["timelines"], data["holds"])
    place = V.anchor_places(data["chapters"], data["placements"], data["script"], data["onsets"], data["marks"], sites, None)
    flagged = [r for r in rows if r["was"]]
    needs_tail = lambda r: r["base"].startswith("closing:") or r["base"] == "voice_end:12_21"  # stage09 stopped at the voice: these need the new tail
    fresh = [r for r in rows if not r["was"] and not needs_tail(r)]
    closing_rows = [r for r in rows if not r["was"] and needs_tail(r)]
    back = V.resolve_anchors(flagged, place, total)
    fresh_back = V.resolve_anchors(fresh, place, total)
    expect = [f for r in flagged for f in r["was"]]
    every = back + fresh_back
    names = Counter(r["finding"] for r in rows)
    pause = sorted(k for k in names if k.startswith("pause-"))
    joins = sorted(k for k in names if k.startswith("join-") and k[5:7].isdigit())
    needed = {"p167-O1", "p168-O2", "p168-O3", "p168-U1", "p168-U7", "p169-F1", "p169-F2", "p169-F3", "p169-M1", "p169-H1", "join-J1", "join-J2",
              "join-J3", "join-J4", "root-seen", "rep-04-reveal", "rep-11-gloss", "rep-11-sample", "rep-11-formula", "rep-12-closing"}
    closed = {r["offsets"][0]: r["base"] for r in closing_rows}
    readable = all(a["reason"].isascii() and "\t" not in a["reason"] and len(a["reason"]) <= 78 for a in every)
    no_tail = [refusal(V.resolve_anchors, [r], place, total) for r in closing_rows]
    check("G3-anchors-on-stage09", [x["frame"] for x in back] == expect and len(pause) == 13 and len(joins) == 12 and needed <= set(names) and readable
          and all(no_tail) and not stray,
          f"{len(rows)} rows, {sum(len(r['offsets']) for r in rows)} anchors: all {len(expect)} flagged old frames come back exactly from their "
          f"schedule bases; {len(fresh_back)} new anchors resolve; the {len(closing_rows)} rows that need the new tail are refused on "
          f"stage09, which stopped at the voice ({no_tail[0]!r}; {no_tail[1]!r}); {len(pause)} pause sites, {len(joins)} chapter "
          f"joins and every required finding are present; captions are ASCII, no tabs, at most 78 characters (longest "
          f"{max(len(a['reason']) for a in every)}); closing bases {sorted(set(closed.values()))}")

    # ------------------------------------------------------------------ G4
    full = V.plan_samples(*args, every)
    by_frame = {s["frame"]: s for s in full}
    kept = all(o["frame"] in by_frame and by_frame[o["frame"]]["reasons"][-len(o["reasons"]):] == o["reasons"] for o in sealed)
    reasons = {x["reason"] for x in every}
    first = all(by_frame[x["frame"]]["reasons"][0] in reasons for x in every)
    only = sorted(set(by_frame) - {o["frame"] for o in sealed})
    ledger = V.anchor_ledger(full, every)
    check("G4-superset", kept and first and all(full[x["picture"] - 1]["frame"] == x["frame"] and x["sheet"] == (x["picture"] + 3) // 4 for x in ledger),
          f"with anchors the plan has {len(full)} pictures in {(len(full) + 3) // 4} sheets on stage09: all {len(sealed)} old frames with all their old "
          f"reasons in order, {len(only)} frames only the anchors add, an anchor's reason first at each anchored frame, and the ledger's "
          f"picture and sheet numbers point at the anchored frames")

    # ------------------------------------------------------------------ G5
    real = {x["chapter"]: (REPO / D.WORK / f"{x['chapter']}.timeline.tsv").read_text() for x in data["chapters"]}
    moved_t = {ch: V.parse_timeline(synthetic_timeline(text, ch), ch) for ch, text in real.items()}
    durations = {p["cue"]: p["duration_s"] for p in data["placements"]}

    def schedule_of(timelines):
        offset, chapters, placements = 0, [], []
        for ch in V.CHAPTERS:
            for line in timelines[ch]["lines"]:
                placements.append({"cue": line["id"], "chapter": ch, "start_frame": offset + line["start"], "duration_s": durations[line["id"]]})
            chapters.append({"chapter": ch, "start_frame": offset, "frames": timelines[ch]["frames"]})
            offset += timelines[ch]["frames"]
        return chapters, placements, offset

    chapters2, placements2, total2 = schedule_of(moved_t)
    sites2, stray2 = V.hold_windows(chapters2, placements2, moved_t, data["holds"])
    close_doc = {"cue": "12_21", "hold_seconds": 5.5, "fade_seconds": 1.5, "after_complete_recording": True, "prediction_or_reading_direction_row": False}
    closing2 = V.closing_window(chapters2, placements2, moved_t, close_doc, stray2)
    place2 = V.anchor_places(chapters2, placements2, data["script"], data["onsets"], data["marks"], sites2, closing2)
    moved = V.resolve_anchors(rows, place2, total2)
    boundary = next(x["start_frame"] for x in data["chapters"] if x["chapter"] == "05_complex_turns")

    def expected(row, was):
        if row["base"] == "chapter_end:04_undoing_growth":
            return was + 24
        return was + 24 if was >= boundary else was

    wanted = [expected(r, f) for r in flagged for f in r["was"]]
    got = V.resolve_anchors(flagged, place2, total2)
    mangled = copy.deepcopy(rows)
    for r in mangled:
        r["was"] = [0] * len(r["offsets"]) if r["was"] else [999999]
    missing = [{k: v for k, v in r.items() if k != "was"} for r in rows]
    twelve = next(x for x in chapters2 if x["chapter"] == "12_what_it_forgets")["start_frame"]
    arithmetic = (closing2["hold_start_frame"] - twelve, closing2["hold_end_frame"] - twelve, closing2["end_frame"] - twelve)
    pauses = sorted(a["frame"] for a in moved if a["finding"].startswith("pause-"))
    check("G5-synthetic-schedule", [x["frame"] for x in got] == wanted and arithmetic == (9837, 10167, 10257) and closing2["end_frame"] == total2
          and V.resolve_anchors(mangled, place2, total2) == moved == V.resolve_anchors(missing, place2, total2)
          and all(x["frame"] < total2 for x in moved),
          f"SYNTHETIC: real timelines, chapter 04 +24, chapter 12 +420, closing breath at local {CLOSING_LOCAL}, {total2} frames. All {len(wanted)} flagged "
          f"anchors land at the old frame + 24 from chapter 05 on and unchanged before it; the closing resolves to local {arithmetic}; "
          f"{len(pauses)} pause anchors resolve from the moved holds; replacing or deleting every `was` changes no resolved frame")

    # ------------------------------------------------------------------ G6
    def one(base, offsets=(0,)):
        return [{"id": "t", "finding": "t", "base": base, "offsets": list(offsets), "note": "n"}]

    bad = {
        "missing cue": (one("cue:99_1"), "no such cue"), "missing chapter": (one("chapter_start:99_x"), "no such chapter"),
        "sentence past the end": (one("sentence:04_7:99"), "04_7 has"), "sentence zero": (one("sentence:04_7:0"), "04_7 has"),
        "words not in the script": (one("phrase:11_5:no such words"), "the words are not once"),
        "words without a phrase mark": (one("phrase:11_5:the"), "the words are not once"),
        "hold not in the map": (one("hold:04_7:start"), "no such hold point"), "unknown hold point": (one("hold:00_4:middle"), "no such hold point"),
        "closing point unknown": (one("closing:middle"), "no such closing point"), "unknown kind": (one("frame:123"), "unknown kind"),
        "offset past the next cue": (one("cue:04_6", (9000,)), "left its cue window"),
        "offset before the cue before": (one("cue:05_2", (-9000,)), "left its cue window"),
        "past the film": (one("closing:end", (0,)), "outside the film"), "chapter end itself": (one("chapter_end:11_unreachable", (0,)), "left its cue window"),
    }
    outcome = {name: refusal(V.resolve_anchors, r, place2, total2) for name, (r, _w) in bad.items()}
    no_mark = "phrase:11_5:" + next(w for w in data["script"]["11_5"].split() if data["script"]["11_5"].split().count(w) == 1 and data["script"]["11_5"].count(w) == 1)
    outcome["once in the script, no mark"] = refusal(V.resolve_anchors, one(no_mark), place2, total2)
    check("G6-planted-resolver", all(outcome[n] and bad[n][1] in outcome[n] for n in bad) and "no phrase mark" in (outcome["once in the script, no mark"] or ""),
          "; ".join(f"{n} -> {outcome[n]!r}" for n in outcome))

    # ------------------------------------------------------------------ G7
    order = [h["cue"] for h in json.loads((REPO / D.OLD_MAP).read_text())["visual_holds"]]
    starts = {x["chapter"]: x["start_frame"] for x in data["chapters"]}
    known = {"00_4": (4339, 4594), "08_4": (6503, 6758), "10_4": (6091, 6466), "11_20": (14163, 15227), "12_5": (5988, 6243)}
    table = all((sites[c]["start_frame"] - starts[sites[c]["chapter"]], sites[c]["next_frame"] - starts[sites[c]["chapter"]]) == v for c, v in known.items())

    def hold_plant(change, schedule=None):
        t = copy.deepcopy(data["timelines"])
        p = copy.deepcopy(data["placements"])
        change(t, p)
        return refusal(V.hold_windows, data["chapters"], p, t, data["holds"])

    def nudge_next(_t, p):
        next(x for x in p if x["cue"] == "00_5")["start_frame"] += 1

    def add_mark(t, _p):
        t["00_ratio_ruler"]["marks"].append({"label": "predict", "frame": 4400})

    def mark_reading(t, _p):
        t["10_doing_nothing"]["marks"].append({"label": "x", "frame": 6100})

    def move_mark(t, _p):
        t["00_ratio_ruler"]["marks"][0]["frame"] += 1

    def rename_mark(t, _p):
        t["00_ratio_ruler"]["marks"][0]["label"] = "other"

    def drop_mark(t, _p):
        t["00_ratio_ruler"]["marks"].pop(0)

    def drop_breath(t, _p):
        t["00_ratio_ruler"]["breaths"].pop(0)

    def add_breath(t, _p):
        t["00_ratio_ruler"]["breaths"].append({"frame": 4400, "seconds": 1.0})

    def short_think(t, _p):
        t["00_ratio_ruler"]["breaths"][0]["seconds"] = 3.0

    holds_bad = {"no predict mark": drop_mark, "two marks": add_mark, "mark off the breath": move_mark, "another label": rename_mark, "no breath": drop_breath,
                 "two breaths": add_breath, "think shortened": short_think, "next cue a frame late": nudge_next, "reading hold with a mark": mark_reading}
    hold_out = {n: hold_plant(fn) for n, fn in holds_bad.items()}

    def close_plant(doc_change=None, t_change=None, stray_now=None):
        t, d = copy.deepcopy(moved_t), dict(close_doc)
        if doc_change:
            d.update(doc_change)
        if t_change:
            t_change(t)
        return refusal(V.closing_window, chapters2, placements2, t, d, stray2 if stray_now is None else stray_now)

    def fade_short(t):
        t["12_what_it_forgets"]["frames"] -= 1

    def early(t):
        t["12_what_it_forgets"]["breaths"][-1]["frame"] -= 200

    ch12 = "12_what_it_forgets"
    close_out = {"another cue": close_plant({"cue": "12_20"}), "hold 5 s in the record": close_plant({"hold_seconds": 5.0}), "fade 1 s in the record": close_plant({"fade_seconds": 1.0}),
                 "a frame short": close_plant(None, fade_short), "no closing breath": close_plant(stray_now=[]), "two closing breaths": close_plant(stray_now=stray2 + stray2),
                 "breath in another chapter": close_plant(stray_now=[("11_unreachable", 100)]), "claims a map row": close_plant({"prediction_or_reading_direction_row": True}),
                 "breath before the last word ends": refusal(V.closing_window, chapters2, placements2, (lambda t: (early(t), t)[1])(copy.deepcopy(moved_t)), close_doc, [(ch12, CLOSING_LOCAL - 200)])}
    check("G7-holds-and-closing", list(sites) == order and table and Counter(s["kind"] for s in sites.values()) == Counter(V.HOLD_KINDS)
          and sum(len(t["marks"]) for t in data["timelines"].values()) == 13 and sum(len(t["breaths"]) for t in data["timelines"].values()) == 15
          and all(hold_out.values()) and all(close_out.values()),
          f"on the real stage09 rows: all {len(sites)} holds in map order, 13 predict marks on their breaths, every prediction ends 15 frames before its next cue and "
          f"every reading hold on it, by the kit's rounding; planted holds: " + "; ".join(f"{n} -> {v!r}" for n, v in hold_out.items())
          + "; planted closing: " + "; ".join(f"{n} -> {v!r}" for n, v in close_out.items()))

    # ------------------------------------------------------------------ G8 the stand-in fixture
    R = module("render_master01_as_read", RENDER_MASTER)
    real_host = refusal(V.gather, V.Ctx(argparse.Namespace(lane="/x", jobs="/x", crate="/x", narration="/x", integration="/x", binding="/x", tools="/x", series="01")))
    real_tools = refusal(V.write_tools, argparse.Namespace(out=str(scratch / "t.json"), ffmpeg="/x", ffprobe="/x"))
    V.HOST = socket.gethostname().split(".")[0].lower()  # this process only: every worker fact below is a stand-in
    S, O = scratch / "S", scratch / "O"
    lane, jobs, narration, ws = O / "jobs", O / "render-jobs01", O / "narration01", S / "workspace"
    crate = ws / "explainer_corpus/logarithm_mysteries"
    work = lane / f"{V.STAGE}-render-work01"
    for folder in (lane, jobs, narration, crate / "docs", crate / "src/chapters", work, O / "audio-jobs01"):
        folder.mkdir(parents=True)
    source = REPO / D.CRATE
    for rel in ["script.tsv", "narration_units.json"] + [f"src/chapters/{p.name}" for p in sorted((source / "src/chapters").glob("*.rs"))]:
        (crate / rel).write_bytes((source / rel).read_bytes())
    tools_dir, other_tools = scratch / "stand-in-tools", scratch / "stand-in-tools-2"
    tools_dir.mkdir()
    other_tools.mkdir()

    def tool_script(name, body):
        path = tools_dir / name
        path.write_text("#!/bin/sh\n# STAND-IN for checks only. Reads no media and decodes nothing.\n" + body)
        path.chmod(path.stat().st_mode | stat.S_IXUSR)
        return str(path)

    fake_ffprobe = tool_script("ffprobe", f'case "$*" in *-version*) echo "ffprobe version STAND-IN";; *count_packets*) cat "{tools_dir}/count.txt";; *) cat "{tools_dir}/probe.json";; esac\n')
    fake_ffmpeg = tool_script("ffmpeg", 'for a; do last=$a; done\n'
                              f'case "$*" in *-version*) echo "ffmpeg version STAND-IN";; *ebur128*) printf "Summary:\\n  I: -16.0 LUFS\\n  LRA: 5.0 LU\\n  Peak: -1.5 dBFS\\n";; *) cp "{tools_dir}/picture.png" "$last";; esac\n')
    (other_tools / "ffprobe").write_text("#!/bin/sh\n# STAND-IN, another copy\n")
    (other_tools / "ffprobe").chmod(0o755)
    tools_json = O / "tools01.json"
    O.mkdir(exist_ok=True)
    with contextlib.redirect_stdout(io.StringIO()):
        wrote_tools = V.main(["tools", "--out", str(tools_json), "--ffmpeg", fake_ffmpeg, "--ffprobe", fake_ffprobe])
    tools_again = refusal(V.write_tools, argparse.Namespace(out=str(tools_json), ffmpeg=fake_ffmpeg, ffprobe=fake_ffprobe))
    tools_relative = refusal(V.write_tools, argparse.Namespace(out="tools01.json", ffmpeg=fake_ffmpeg, ffprobe=fake_ffprobe))
    binary = O / "logarithm_mysteries-release01"
    binary.write_bytes(b"STAND-IN binary: never run\n")
    integration, binding, build_inputs, assemble_inputs = S / "integration.json", O / "remote-binding01.json", O / "build-inputs01.json", O / "assemble-inputs01.json"
    record = json.loads((REPO / D.S / "integration.json").read_text())
    record["source_workspace"] = str(ws)
    integration.write_text(json.dumps(record, indent=1))
    map_old = json.loads((REPO / D.OLD_MAP).read_text())
    for h in map_old["visual_holds"]:  # what the rebind does to the hashes; a stand-in for root's real oliver17, which does not exist yet
        h["sha256"] = sha(ws / h["path"])
    map17 = crate / V.DIRECTION_MAP
    map17.write_text(json.dumps(map_old, indent=1))
    report = json.loads((REPO / D.NARRATION / "report.json").read_text())
    report["meta"]["direction_contract"]["binding"]["contract"]["map"] = {"path": f"explainer_corpus/logarithm_mysteries/{V.DIRECTION_MAP}", "sha256": sha(map17)}
    (narration / "report.json").write_text(json.dumps(report, sort_keys=True))
    (narration / "phrase-timings.json").write_bytes((REPO / D.NARRATION / "phrase-timings.json").read_bytes())
    cues = list(data["script"])
    for cue in cues:
        (narration / f"{cue}.wav").symlink_to(REPO / D.NARRATION / f"{cue}.wav")  # real recordings, read only, never copied or changed
    clock = iter(range(10, 59))

    def tick():
        return f"2026-11-06T01:{next(clock):02d}:00.000000+00:00"

    def receipt(command, inputs, start):
        return {"started": start, "phase": "native", "command": command, "admitted": True, "inputs_sha256": sha(inputs), "final_inputs_sha256": sha(inputs),
                "exit": 0, "finished": tick()}

    def job_folder(name, files, line, command, inputs):
        folder = lane / name
        folder.mkdir()
        start = tick()
        for rel, body in files.items():
            (folder / rel).parent.mkdir(parents=True, exist_ok=True)
            (folder / rel).write_text(body)
        (folder / "command.log").write_text(f"RUN fixture\n{line}\n")
        (folder / "resource-receipt.json").write_text(json.dumps(receipt(command, inputs, start)))
        return folder

    def tree_rows(folder):
        return [p for p in sorted(Path(folder).rglob("*")) if p.is_file()]

    crate_rows = tree_rows(crate)
    build_inputs.write_text(json.dumps({"files": [{"path": str(p), "sha256": sha(p)} for p in crate_rows]}, indent=2))
    assemble_inputs.write_text(json.dumps({"files": [{"path": str(p), "sha256": sha(p)} for p in (crate / "script.tsv", crate / "narration_units.json", map17)]}, indent=2))
    py = sys.executable
    build_job, assemble_job, rebind_job = f"{V.STAGE}-build01", f"{V.STAGE}-assemble01", f"{V.STAGE}-rebind01"
    tl, sc, mjob = f"{V.STAGE}-timeline01", f"{V.STAGE}-score01", f"{V.STAGE}-master01"
    job_folder(build_job, {"chapters.txt": "\n".join(V.CHAPTERS) + "\n"}, "BUILD fixture", ["/bin/bash", str(O / "build01.sh")], build_inputs)
    job_folder(rebind_job, {"rebind-receipt.json": "{}"}, "REBIND fixture", [py, "-I", "-B", str(O / "rebind01.py")], build_inputs)
    wavs = {cue: sha(narration / f"{cue}.wav") for cue in cues}
    job_folder(assemble_job, {"audio-comparison.json": json.dumps({"cues": 80, "wav_sha256": wavs, "phrase_timings_sha256": sha(narration / "phrase-timings.json"),
                                                                   "all_full_wavs_byte_identical": True, "all_phrase_timings_byte_identical": True})},
               "ASSEMBLE fixture", [py, "-I", "-B", str(O / "assemble01.py")], assemble_inputs)
    offset, summary_rows, captions, texts = 0, [], [], {}

    def stamp_of(ms, mark):
        return f"{ms // 3600000:02d}:{ms // 60000 % 60:02d}:{ms // 1000 % 60:02d}{mark}{ms % 1000:03d}"

    for ch in V.CHAPTERS:
        text = synthetic_timeline(real[ch], ch, narration)
        texts[ch] = text
        t = V.parse_timeline(text, ch)
        (work / f"{ch}.timeline.tsv").write_text(text)
        (work / f"{ch}.layout.tsv").write_text((REPO / D.WORK / f"{ch}.layout.tsv").read_text())
        (work / f"{ch}.mp4").write_bytes(f"STAND-IN chapter {ch}\n".encode())
        for line in t["lines"]:
            at = round((offset + line["start"]) * 1000 / 60)
            captions.append((at, at + 1000, data["script"][line["id"]]))
        summary_rows.append({"chapter": ch, "frames": t["frames"], "cues": [x["id"] for x in t["lines"]], "timeline_sha256": sha(work / f"{ch}.timeline.tsv")})
        offset += t["frames"]
    total_fixture = offset
    timeline_inputs = O / "audio-jobs01" / f"{tl}-inputs.json"
    timeline_inputs.write_text(json.dumps({"files": [{"path": str(crate / "script.tsv"), "sha256": sha(crate / "script.tsv")}]}))
    job_folder(tl, {"timeline-summary.json": json.dumps({"fps": 60, "chapters": summary_rows, "cues": 80, "total_frames": total_fixture}),
                    **{f"timeline-output/{ch}.timeline.tsv": texts[ch] for ch in V.CHAPTERS}},
               V.TIMELINE_SUCCESS, ["python3", "-I", "-B", str(O / "audio-jobs01" / f"{tl}.py")], timeline_inputs)
    job_folder(sc, {"scored/stems/qa/stand-in.json": "{}"}, V.SCORE_SUCCESS, ["python3", "-I", "-B", str(O / "audio-jobs01" / f"{sc}.py")], timeline_inputs)
    render_jobs = {ch: f"{V.STAGE}-render-{ch}-01" for ch in V.CHAPTERS}
    soundtracks = {}
    for ch, job in render_jobs.items():
        track = work / "soundtrack" / ch / "soundtrack_0.wav"
        track.parent.mkdir(parents=True)
        track.write_bytes(b"STAND-IN capture\n")
        soundtracks[ch] = {"path": str(track), "sha256": sha(track)}
        (jobs / f"{job}-inputs.json").write_text(json.dumps({"files": [{"path": str(work / f"{ch}.mp4"), "sha256": sha(work / f"{ch}.mp4")}]}))
        job_folder(job, {"render-summary.json": json.dumps({"chapter": ch, "mp4_sha256": sha(work / f"{ch}.mp4"), "soundtrack": soundtracks[ch]})},
                   V.RENDER_SUCCESS, [py, "-I", "-B", str(jobs / f"{job}.py")], jobs / f"{job}-inputs.json")
    master, srt, vtt = work / f"{V.TOPIC}_master.mp4", work / f"{V.TOPIC}.en.srt", work / f"{V.TOPIC}.en.vtt"
    master.write_bytes(b"STAND-IN master: not a video\n")
    srt.write_text("".join(f"{k}\n{stamp_of(s, ',')} --> {stamp_of(e, ',')}\n{x}\n\n" for k, (s, e, x) in enumerate(captions, 1)))
    vtt.write_text("WEBVTT\n\n" + "".join(f"{stamp_of(s, '.')} --> {stamp_of(e, '.')}\n{x}\n\n" for s, e, x in captions))
    script_path, config_path, inputs_path = jobs / f"{mjob}.py", jobs / f"{mjob}.config.json", jobs / f"{mjob}-inputs.json"
    script_path.write_text(R.worker_text())  # root's real worker text, as render-master01.py writes it
    cfg = R.old_config()  # the same statements as root's launch(), on scratch paths
    stems = lane / sc / "scored/stems"
    cfg.update(job=mjob, step="master", script=str(script_path), binary=str(binary), binary_sha256=sha(binary), narration=str(narration), chapter=None,
               timeline_job=tl, render_work=str(work), after=[tl, sc] + list(render_jobs.values()), soundtracks=soundtracks, render_jobs=render_jobs,
               lane=str(lane), target=str(scratch / "target"))
    cfg["argv"] = [str(binary), "master", "--narration", str(narration), "--score", str(stems), "--out", str(work)]
    cfg["captions_argv"] = [str(binary), "captions", "all", "--narration", str(narration), "--out", str(work)]
    cfg["scope"] = "STAND-IN fixture"
    cfg.update(tools=json.loads(tools_json.read_text()), tools_sha256=sha(tools_json), binding=str(binding))  # Lavender's added keys, Mail 5296
    config_path.write_text(json.dumps(cfg, indent=2))
    bound_paths = sorted({*(tree_rows(narration)), *crate_rows, *tree_rows(lane / tl), *tree_rows(lane / sc), *tree_rows(lane / build_job), *tree_rows(lane / rebind_job), *tree_rows(lane / assemble_job),
                          *[p for job in render_jobs.values() for p in tree_rows(lane / job)], *[p for p in tree_rows(work) if p not in (master, srt, vtt)],
                          binary, build_inputs, assemble_inputs, script_path, config_path, integration}, key=str)
    binding_body = {"schema": V.BINDING_SCHEMA, "approved": {"integration_sha256": sha(integration)}, "build": {"job": build_job, "binary": str(binary), "binary_sha256": sha(binary)},
                    "audio": {"narration": str(narration)}, "voice": "STAND-IN", "cues": 80, "chapters": 13, "crate": {str(p.relative_to(crate)): sha(p) for p in crate_rows},
                    "direction": {"map": f"explainer_corpus/logarithm_mysteries/{V.DIRECTION_MAP}", "map_sha256": sha(map17), "holds": 15, "closing_hold_cue": "12_21"},
                    "files": {str(build_inputs): sha(build_inputs)}}
    binding.write_text(json.dumps(binding_body, indent=1))
    inputs_path.write_text(json.dumps({"files": [{"path": str(p), "sha256": sha(p)} for p in bound_paths]}, indent=2))
    (lane / mjob).mkdir()
    (lane / mjob / "master-summary.json").write_text(json.dumps({k: {"path": str(p), "sha256": sha(p)} for k, p in (("master", master), ("srt", srt), ("vtt", vtt))}))
    (lane / mjob / "command.log").write_text(f"RUN fixture\n{V.MASTER_SUCCESS}\n")
    master_start = tick()
    mreceipt = lane / mjob / "resource-receipt.json"
    mreceipt.write_text(json.dumps(receipt([py, "-I", "-B", str(script_path)], inputs_path, master_start)))
    breceipt, rreceipt = lane / build_job / "resource-receipt.json", lane / rebind_job / "resource-receipt.json"

    def namespace(**extra):
        return argparse.Namespace(lane=str(lane), jobs=str(jobs), crate=str(crate), narration=str(narration), integration=str(integration), binding=str(binding), tools=str(tools_json), series="01", **extra)

    c = V.Ctx(namespace())
    result = V.gather(c)

    def js(path, change):
        def edit(raw):
            value = json.loads(raw)
            changed_value = change(value)
            return json.dumps(value if changed_value is None else changed_value, indent=1).encode()
        return path, edit

    def field(path, **fields):
        return js(path, lambda d: d.update(fields))

    def reseal(follow_map=True):
        """A consistent forgery: everything downstream of the plant re-hashed, so that only the planted fact can decide."""
        if follow_map:
            r = json.loads((narration / "report.json").read_text())
            r["meta"]["direction_contract"]["binding"]["contract"]["map"]["sha256"] = sha(map17)
            (narration / "report.json").write_text(json.dumps(r, sort_keys=True))
        for holder in (breceipt, rreceipt):
            b = json.loads(holder.read_text())
            b["inputs_sha256"] = b["final_inputs_sha256"] = sha(build_inputs)
            holder.write_text(json.dumps(b))
        d = json.loads(binding.read_text())
        d["approved"]["integration_sha256"] = sha(integration)
        d["crate"] = {str(p.relative_to(crate)): sha(p) for p in tree_rows(crate)}
        d["direction"]["map_sha256"] = sha(map17)
        d["files"] = {str(build_inputs): sha(build_inputs)}
        binding.write_text(json.dumps(d, indent=1))
        rows_now = json.loads(inputs_path.read_text())
        for row in rows_now["files"]:
            row["sha256"] = sha(row["path"])
        inputs_path.write_text(json.dumps(rows_now, indent=2))
        m = json.loads(mreceipt.read_text())
        m["inputs_sha256"] = m["final_inputs_sha256"] = sha(inputs_path)
        mreceipt.write_text(json.dumps(m))

    protected = [narration / "report.json", breceipt, rreceipt, binding, inputs_path, mreceipt]

    @contextlib.contextmanager
    def planted(edits, seal=False, after=(), follow_map=True):
        keep = {p: p.read_bytes() for p, _ in [*edits, *after]}
        if seal:
            keep.update({p: p.read_bytes() for p in protected if p not in keep})
        try:
            for p, fn in edits:
                p.write_bytes(fn(p.read_bytes()))
            if seal:
                reseal(follow_map)
            for p, fn in after:
                p.write_bytes(fn(p.read_bytes()))
            yield
        finally:
            for p, raw in keep.items():
                p.write_bytes(raw)

    def row_of(rel):
        return next(r for r in json.loads(integration.read_text())["files"] if r["relative_path"] == rel)

    src = "explainer_corpus/logarithm_mysteries/src/chapters/"

    def integration_row(rel, **fields):
        def change(d):
            next(r for r in d["files"] if r["relative_path"] == rel).update(fields)
        return js(integration, change)

    def without_source(d):
        d["changed_files"] = [x for x in d["changed_files"] if not x.endswith("/draw.rs")]

    def drop_manifest_row(path_end):
        def change(d):
            d["files"] = [r for r in d["files"] if not r["path"].endswith(path_end)]
        return js(inputs_path, change)

    def build_row_other(d):
        next(r for r in d["files"] if r["path"].endswith("/unreachable.rs"))["sha256"] = "0" * 64

    def strip_breath(raw):
        return b"\n".join(x for x in raw.split(b"\n") if not x.startswith(b"breath\t9837")) + b""

    def fewer_holds(d):
        d["visual_holds"].pop()

    def stale_hold(d):
        d["visual_holds"][0]["sha256"] = "0" * 64

    def report_map(**fields):
        return js(narration / "report.json", lambda d: d["meta"]["direction_contract"]["binding"]["contract"]["map"].update(fields))

    def report_meta(**fields):
        return js(narration / "report.json", lambda d: d["meta"].update(fields))

    def set_cfg(key, value):
        return field(config_path, **{key: value})

    first_render = lane / render_jobs[V.CHAPTERS[0]]
    other_log = (lambda _raw: b"RENDER PASS: 4K60 chapter; frame and cue schedule equals the 60 fps timeline.\n")
    cases = {
        "the master exited 1": ([field(mreceipt, exit=1)], {}, "did not pass: exit 1"),
        "the master stopped for a resource limit": ([field(mreceipt, stopped_for_resource_limit=True)], {}, "did not pass"),
        "the master's inputs changed during the run": ([field(mreceipt, final_inputs_sha256="0" * 64)], {}, "did not pass"),
        "the master never finished": ([js(mreceipt, lambda d: d.pop("finished") and None)], {}, "did not pass"),
        "the master not admitted": ([field(mreceipt, admitted=False)], {}, "did not pass"),
        "bash ran the master": ([field(mreceipt, command=["/bin/bash", str(script_path)])], {}, "the guard's receipt ran"),
        "a python2 ran the master": ([field(mreceipt, command=["python2.7", "-I", "-B", str(script_path)])], {}, "the guard's receipt ran"),
        "another script ran": ([field(mreceipt, command=[py, "-I", "-B", str(jobs / "other.py")])], {}, "the guard's receipt ran"),
        "the manifest was edited": ([(inputs_path, lambda d: d + b" ")], {}, "the guard's input hash is not"),
        "a chapter mp4 moved": ([(work / "05_complex_turns.mp4", lambda d: d + b"x")], {}, "bound inputs moved since it ran"),
        "a repaired source moved": ([(crate / "src/chapters/unreachable.rs", lambda d: d + b"\n")], {}, "bound inputs moved since it ran"),
        "a WAV row moved": ([(narration / "phrase-timings.json", lambda d: d + b"\n")], {}, "bound inputs moved since it ran"),
        "the master changed after its job": ([(master, lambda d: d + b"x")], {}, "the master changed since the master job"),
        "the SRT changed after its job": ([(srt, lambda d: d + b"\n")], {}, "the srt changed since the master job"),
        "the log lacks the master's line": ([(lane / mjob / "command.log", lambda d: d + b"Traceback\n")], {}, "does not end with the master's success line"),
        "a render's log has stage09's line": ([(first_render / "command.log", other_log)], {"seal": True}, "does not end with the render's success line"),
        "a render exited 1": ([field(first_render / "resource-receipt.json", exit=1)], {"seal": True}, "did not pass"),
        "score failed": ([field(lane / sc / "resource-receipt.json", exit=1)], {"seal": True}, f"{sc} did not pass"),
        "score log lacks its line": ([(lane / sc / "command.log", lambda d: d + b"x\n")], {"seal": True}, f"{sc} did not pass"),
        "build failed": ([field(breceipt, exit=101)], {"seal": True}, f"{build_job} did not pass"),
        "assembly failed": ([field(lane / assemble_job / "resource-receipt.json", exit=1)], {"seal": True}, f"{assemble_job} did not pass"),
        "rebind failed": ([field(rreceipt, exit=1)], {"seal": True}, f"{rebind_job} did not pass"),
        "the rebind started before the build ended": ([field(rreceipt, started="2026-11-06T00:00:00.000000+00:00")], {"seal": True}, "started before"),
        "the assembly started before the rebind ended": ([field(lane / assemble_job / "resource-receipt.json", started="2026-11-06T00:00:00.000000+00:00")], {"seal": True}, "started before"),
        "the master ran under other tools": ([set_cfg("tools_sha256", "0" * 64)], {"seal": True}, "ran under another tools file"),
        "a render attempt -r1": ([set_cfg("render_jobs", {**render_jobs, V.CHAPTERS[0]: render_jobs[V.CHAPTERS[0]] + "-r1"})], {"seal": True}, "render job"),
        "the master did not bind integration.json": ([drop_manifest_row("/integration.json")], {"seal": True}, "did not bind integration.json"),
        "the master did not bind the map": ([drop_manifest_row(V.DIRECTION_MAP)], {"seal": True}, "did not bind the crate's"),
        "the master did not bind the unit plan": ([drop_manifest_row("/narration_units.json")], {"seal": True}, "did not bind the crate's"),
        "a timeline with a comment added": ([(work / "05_complex_turns.timeline.tsv", lambda d: d + b"# same schedule, other bytes\n")], {"seal": True}, "timeline bytes differ"),
        "a summary with another timeline hash": ([js(lane / tl / "timeline-summary.json", lambda d: d["chapters"][3].update(timeline_sha256="0" * 64))], {"seal": True}, "timeline bytes differ"),
        "timeline failed": ([field(lane / tl / "resource-receipt.json", exit=1)], {"seal": True}, f"{tl} did not pass"),
        "the master started before a render ended": ([field(mreceipt, started="2026-11-06T01:11:00.000000+00:00")], {}, "started before"),
        "the timeline started before the build ended": ([field(lane / tl / "resource-receipt.json", started="2026-11-06T00:00:00.000000+00:00")], {"seal": True}, "started before"),
        "the config is a render step": ([set_cfg("step", "render")], {"seal": True}, "another step, job or lane"),
        "the config has 12 chapters": ([set_cfg("chapters", V.CHAPTERS[:-1])], {"seal": True}, "the script's chapters or cues differ"),
        "the config has 79 cues": ([set_cfg("cues", cues[:-1])], {"seal": True}, "the script's chapters or cues differ"),
        "the config names another narration": ([set_cfg("narration", str(scratch / "other"))], {"seal": True}, "the master used narration"),
        "the config's command writes elsewhere": ([set_cfg("argv", cfg["argv"][:-1] + [str(scratch / "elsewhere")])], {"seal": True}, "the master command is"),
        "the guard did not bind the config": ([drop_manifest_row(".config.json")], {"seal": True}, "did not bind the script's config"),
        "the guard did not bind the script": ([drop_manifest_row(f"{mjob}.py")], {"seal": True}, "the script moved"),
        "not approved": ([field(integration, approved_for_guarded_build=False)], {"seal": True}, "does not approve"),
        "audio changed": ([field(integration, audio_changed=True)], {"seal": True}, "does not approve"),
        "integration names another script": ([field(integration, script_sha256="0" * 64)], {"seal": True}, "names another script"),
        "draw.rs left out": ([js(integration, without_source)], {"seal": True}, "does not change ['draw.rs']"),
        "not changed from stage09": ([integration_row(src + "unreachable.rs", before_sha256="0" * 64)], {"seal": True}, "not changed from the rendered stage09 source"),
        "a changed row with no change": ([integration_row(src + "unreachable.rs", sha256=row_of(src + "unreachable.rs")["before_sha256"])], {"seal": True}, "records no change"),
        "a new hash not on disk": ([integration_row(src + "unreachable.rs", sha256="0" * 64)], {"seal": True}, "not the integrated file"),
        "the guard did not bind a source": ([drop_manifest_row("/unreachable.rs")], {"seal": True}, "did not bind"),
        "the build compiled other bytes": ([js(build_inputs, build_row_other)], {"seal": True}, "did not compile"),
        "binding schema": ([], {"seal": True, "after": [field(binding, schema="other")]}, f"not a {V.BINDING_SCHEMA}"),
        "binding approves another integration": ([], {"seal": True, "after": [js(binding, lambda d: d["approved"].update(integration_sha256="0" * 64))]}, "root approved another"),
        "binding names another binary": ([], {"seal": True, "after": [js(binding, lambda d: d["build"].update(binary_sha256="0" * 64))]}, "the bound binary is not"),
        "binding names another build job": ([], {"seal": True, "after": [js(binding, lambda d: d["build"].update(job="other"))]}, "the bound binary is not"),
        "binding names another narration": ([], {"seal": True, "after": [js(binding, lambda d: d["audio"].update(narration=str(scratch)))]}, "the bound narration is"),
        "binding has 79 cues": ([], {"seal": True, "after": [field(binding, cues=79)]}, "79 cues"),
        "a crate file left its bound hash": ([], {"seal": True, "after": [js(binding, lambda d: d["crate"].update({"script.tsv": "0" * 64}))]}, "crate files moved"),
        "the binding lost the build manifest": ([], {"seal": True, "after": [field(binding, files={})]}, "no one bound file"),
        "report binds another map hash": ([report_map(sha256="0" * 64)], {"seal": True, "follow_map": False}, "report.json binds the map"),
        "report binds oliver16": ([report_map(path="explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver16.json")], {"seal": True}, "report.json binds the map"),
        "binding has another map": ([], {"seal": True, "after": [js(binding, lambda d: d["direction"].update(map_sha256="0" * 64))]}, "another direction map"),
        "report from another script": ([report_meta(script_sha256="0" * 64)], {"seal": True}, "assembled from another script"),
        "report from another unit plan": ([report_meta(units_sha256="0" * 64)], {"seal": True}, "assembled from another unit plan"),
        "14 hold rows": ([js(map17, fewer_holds)], {"seal": True}, "hold map:"),
        "a stale hold row": ([js(map17, stale_hold)], {"seal": True}, "stale source hash"),
        "audio record not identical": ([field(lane / assemble_job / "audio-comparison.json", all_full_wavs_byte_identical=False)], {"seal": True}, "does not record byte-identical audio"),
        "a WAV off the audio record": ([js(lane / assemble_job / "audio-comparison.json", lambda d: d["wav_sha256"].update({"05_1": "0" * 64}))], {"seal": True}, "WAVs differ from the assembly's record"),
        "the closing breath is gone": ([(work / "12_what_it_forgets.timeline.tsv", strip_breath)], {"seal": True}, "one closing breath"),
        "the closing record is gone": ([js(integration, lambda d: d.pop("editorial_closing_hold") and None)], {"seal": True}, "editorial_closing_hold names None"),
        "a predict mark is gone": ([(work / "00_ratio_ruler.timeline.tsv", lambda d: b"\n".join(x for x in d.split(b"\n") if not x.startswith(b"mark\tpredict")))], {"seal": True}, "predict mark"),
        "a registry without a chapter": ([(crate / "src/chapters/mod.rs", lambda d: d.replace(b'key: "12_what_it_forgets",', b'key_gone: "12",'))], {}, "registry is not the 13"),
        "a script of 79 cues": ([(crate / "script.tsv", lambda d: b"\n".join(x for x in d.split(b"\n") if not x.startswith(b"12_21\t")))], {}, "the script has 79 cues"),
        "a script of 81 cues": ([(crate / "script.tsv", lambda d: d + b"12_22\tAn extra cue.\n")], {}, "the script has 81 cues"),
        "a repeated cue": ([(crate / "script.tsv", lambda d: d + b"12_21\tAgain.\n")], {}, "repeats"),
        "tools file of another schema": ([field(tools_json, schema="other")], {}, f"not a {V.TOOLS_SCHEMA} for"),
        "tools file of another host": ([field(tools_json, host="another-host")], {}, f"not a {V.TOOLS_SCHEMA} for"),
        "ffmpeg not the pinned bytes": ([js(tools_json, lambda d: d["ffmpeg"].update(sha256="0" * 64))], {}, "this host's ffmpeg is not"),
        "ffprobe path missing here": ([js(tools_json, lambda d: d["ffprobe"].update(path=str(scratch / "gone")))], {}, "this host's ffprobe is not"),
        "a relative tool path": ([js(tools_json, lambda d: d["ffmpeg"].update(path="ffmpeg"))], {}, "this host's ffmpeg is not"),
        "another interpreter": ([js(tools_json, lambda d: d["python3"].update(path=fake_ffmpeg, sha256=sha(fake_ffmpeg)))], {}, "this interpreter is"),
        "another Pillow on file": ([js(tools_json, lambda d: d["python3"].update(pillow="1.0"))], {}, "the tools file says Pillow"),
        "another ffprobe would answer on PATH": ([js(tools_json, lambda d: d["ffprobe"].update(path=str(other_tools / "ffprobe"), sha256=sha(other_tools / "ffprobe")))], {}, "finds ffprobe at"),
    }
    out = {}
    for name, (edits, options, _want) in cases.items():
        with planted(edits, **options):
            out[name] = refusal(V.gather, c)
    retired = {}
    for constant, digest in (("RETIRED_MASTER", sha(master)), ("RETIRED_BINARY", sha(binary))):
        saved = getattr(V, constant)
        setattr(V, constant, digest)
        retired[constant] = refusal(V.gather, c)
        setattr(V, constant, saved)
    saved_pillow = V.MIN_PILLOW
    V.MIN_PILLOW = (99, 0)
    old_pillow = refusal(V.gather, c)
    V.MIN_PILLOW = saved_pillow
    seventh = "explainer_corpus/logarithm_mysteries/src/facts.rs"
    extra = [js(integration, lambda d: d["changed_files"].append(seventh)), integration_row(seventh, before_sha256="1" * 64)]
    seventh_file = crate / "src/facts.rs"
    seventh_file.write_bytes((source / "src/facts.rs").read_bytes())  # present in the record; added to the crate for this one case
    with planted(extra + [js(inputs_path, lambda d: d["files"].append({"path": str(seventh_file), "sha256": sha(seventh_file)})),
                          js(build_inputs, lambda d: d["files"].append({"path": str(seventh_file), "sha256": sha(seventh_file)}))], seal=True):
        seven = V.gather(c)
    os.replace(seventh_file, scratch / "moved-aside-facts.rs")
    bad_anchor = scratch / "anchors-bad.json"
    bad_anchor.write_text(json.dumps({"rows": one("cue:99_1")}))
    saved = V.ANCHORS
    V.ANCHORS = bad_anchor
    anchor_refused = refusal(V.gather, c)
    V.ANCHORS = scratch / "no-such-anchors.json"
    anchors_gone = refusal(V.gather, c)
    V.ANCHORS = saved
    wrong = [n for n in cases if not (out[n] and cases[n][2] in out[n])]
    relative = refusal(V.Ctx, argparse.Namespace(**{**vars(namespace()), "integration": "integration.json"}))
    again = refusal(V.gather, c)
    check("G8-fixture-gates", not wrong and result["total_frames"] == 116689 and len(result["placements"]) == 80 and len(result["chapters"]) == 13
          and all(retired.values()) and "retired stage09 master" in retired["RETIRED_MASTER"] and "retired stage09 binary" in retired["RETIRED_BINARY"]
          and len(seven["sources"]) == 7 and "no such cue" in anchor_refused and "is missing" in anchors_gone and "must be an absolute path" in relative and again is None
          and wrote_tools == 0 and "the contact sheets need 99.0" in (old_pillow or "") and "exists: tools never replaces" in (tools_again or "") and "absolute --out" in (tools_relative or "")
          and "runs only on mac-mini-max" in (real_host or "") and "runs only on mac-mini-max" in (real_tools or ""),
          f"The real HOST refuses here ({real_host!r}), for `tools` as well. Then, with HOST patched in this process, a STAND-IN fixture in the remote layout: root's real worker text and old config shape, its sorted manifest, sys.executable receipts, the real integration record's rows, real WAVs "
          f"by read-only link, synthetic timelines, stand-in media, a stand-in oliver17, and a tools file written by the real `tools` command from stand-in tools. gather passes with 13 chapters, 80 cues, {result['total_frames']} frames, "
          f"{len(result['sources'])} source changes, {len(result['pause_sites'])} holds, {len(result['anchors'])} anchors and {len(result['samples'])} pictures. "
          f"{len(cases)} planted faults refused, each by its expected gate"
          + (f"; WRONG: {[(n, out[n]) for n in wrong]}" if wrong else "") + f"; a seventh changed source passes and is reported ({len(seven['sources'])} rows); "
          f"8 more refused: retired master, retired binary, a relative path, an unresolved anchors file, a missing anchors file, too old a Pillow, a tools file that exists, a relative tools path; after all of it the fixture passes again")
    print("  G8 detail: " + clean("; ".join(f"{n} -> {out[n]}" for n in cases)), flush=True)

    # ------------------------------------------------------------------ G9 the command line and the job, with stand-in tools
    common = ["--lane", str(lane), "--jobs", str(jobs), "--crate", str(crate), "--narration", str(narration), "--series", "01",
              "--integration", str(integration), "--binding", str(binding), "--tools", str(tools_json)]

    def cli(*extra_args):
        """main itself, in this process: HOST stays patched without any override in the helper."""
        with contextlib.redirect_stdout(io.StringIO()) as text:
            code = V.main([*extra_args, *common])
        return code, clean(text.getvalue().strip())

    vdir = O / "verify-jobs01"
    code_c, out_c = cli("check")
    facts = json.loads(out_c) if code_c == 0 else {}
    code_w, out_w = cli("write-job", "--verify", "01", "--job-dir", str(vdir))
    written = json.loads(out_w) if code_w == 0 else {}
    vjob = f"{V.STAGE}-verify01"
    vscript, vinputs = vdir / f"{vjob}.py", vdir / f"{vjob}-inputs.json"
    vrows = json.loads(vinputs.read_text())["files"] if vinputs.is_file() else []
    current = bool(vrows) and vrows[0]["path"] == str(vscript) and all(Path(r["path"]).is_file() and sha(r["path"]) == r["sha256"] for r in vrows)
    named = {Path(r["path"]).name for r in vrows}
    bound_ok = {f"{mjob}.config.json", "integration.json", binding.name, "anchors.json", "unreachable.rs", "what_it_forgets.rs", "draw.rs", "audio-comparison.json",
                Path(V.DIRECTION_MAP).name, "tools01.json", "ffmpeg", "ffprobe", Path(sys.executable).name, "guard01.py", "guard-pins01.json"} <= named
    job_cfg = V.read_config(vscript) if vscript.is_file() else {}
    twice = cli("write-job", "--verify", "01", "--job-dir", str(vdir))
    early = cli("verify-job", "--verify", "01", "--job-dir", str(vdir))
    check("G9a-command-line", code_c == 0 and facts.get("chapters") == 13 and facts.get("cues") == 80 and facts.get("total_frames") == 116689 and code_w == 0 and current and bound_ok
          and written.get("job") == "logarithm-remote-verify01" and written.get("input_rows") == len(vrows)
          and written.get("launch_from_repo_root_on_worker", "").startswith(f"{py} -I -B {V.REMOTE_GUARD_REL} --phase native --job logarithm-remote-verify01 --inputs ")
          and job_cfg.get("ffmpeg") == fake_ffmpeg and job_cfg.get("ffprobe") == fake_ffprobe and job_cfg["tools"] == json.loads(tools_json.read_text())
          and len(job_cfg.get("samples", [])) == facts.get("pictures_planned") and twice[0] == 1 and "exists: use a new --verify number" in twice[1]
          and early[0] == 1 and "resource-receipt.json is missing" in early[1],
          f"`check` passes ({facts.get('total_frames')} frames from the timelines, {facts.get('anchors')} anchors in {facts.get('anchor_rows')} rows, {facts.get('pictures_planned')} pictures, "
          f"{facts.get('sheets_planned')} sheets). `write-job` writes job {written.get('job')} with {len(vrows)} current input rows, the script first, binding the config, integration, binding, "
          f"anchors, map, audio record, changed sources, tools file, tools and interpreter, and prints root's remote guard line, launched by nobody here. Writing it again is refused ({twice[1]!r}); `verify-job` before any "
          f"launch is refused ({early[1]!r})")

    from PIL import Image
    Image.new("RGB", (960, 540), "#101820").save(tools_dir / "picture.png")
    probe = {"streams": [{"codec_type": "video", "codec_name": "h264", "width": 3840, "height": 2160, "r_frame_rate": "60/1", "duration": f"{total_fixture / 60:.6f}"},
                         {"codec_type": "audio", "codec_name": "aac", "sample_rate": "48000", "channels": 2, "duration": f"{total_fixture / 60:.6f}"}], "format": {}}
    (tools_dir / "probe.json").write_text(json.dumps(probe))
    (tools_dir / "count.txt").write_text(f"{total_fixture}\n")
    strict_dir = scratch / "stand-in-strict"
    strict_dir.mkdir()
    fake_strict = strict_dir / "strict.sh"
    fake_strict.write_text('#!/bin/sh\n# STAND-IN for checks only. Decodes nothing. It reports which tools PATH would give root\'s real script.\n'
                           'echo "note: PATH gives $(command -v ffmpeg) and $(command -v ffprobe)"\n'
                           'echo "full decode (video + audio, software, stops at the first error): 0 error lines, ffmpeg exit 0, 1 s"\n'
                           ': > "$TMPDIR/verify_decode.standin"\necho "VERIFY PASS"\n')
    fake_strict = str(fake_strict)
    anchors_in_job = job_cfg["anchors"]
    keep_ids = {"pause-00_4.1", "rep-04-reveal.2", "rep-12-closing.2", "rep-12-closing.3"}
    taken = [x for x in anchors_in_job if x["id"] in keep_ids]
    frames_taken = {x["frame"] for x in taken} | {job_cfg["samples"][k]["frame"] for k in (0, 500, 1000)}
    sub_samples = [s for s in job_cfg["samples"] if s["frame"] in frames_taken]
    job2 = f"{V.STAGE}-verify02"
    script2, inputs2 = vdir / f"{job2}.py", vdir / f"{job2}-inputs.json"
    cfg2 = dict(job_cfg, job=job2, script=str(script2), samples=sub_samples, strict=fake_strict,
                anchors=V.anchor_ledger(sub_samples, [dict(x) for x in taken]))
    body = V.JOB_TEMPLATE.replace("@@JOB@@", job2).replace("@@HELPER@@", sha(HELPER))
    script2.write_text(body.replace("@@CONFIG@@", repr(json.dumps(cfg2, indent=1))))
    inputs2.write_text(json.dumps({"files": [{"path": str(script2), "sha256": sha(script2)}]}) + "\n")
    folder2 = lane / job2
    folder2.mkdir()
    started = {"started": tick(), "admitted": True, "phase": "native", "command": [sys.executable, "-I", "-B", str(script2)], "inputs_sha256": sha(inputs2)}
    (folder2 / "resource-receipt.json").write_text(json.dumps(started))
    log = io.StringIO()
    with contextlib.redirect_stdout(log):
        code = V.run_job(cfg2)
    (folder2 / "command.log").write_text(log.getvalue())
    started.update(exit=code, finished=tick(), final_inputs_sha256=started["inputs_sha256"])
    (folder2 / "resource-receipt.json").write_text(json.dumps(started))
    ctx2 = V.Ctx(namespace(verify="02", job_dir=str(vdir)))
    passed = V.verify_job(ctx2)
    vjson = folder2 / "verification.json"
    made = json.loads(vjson.read_text())
    old_keys = list(data["v"])
    row_shape = all(set(made[k][0]) == set(data["v"][k][0]) for k in ("placements", "chapters"))

    def vedit(**fields):
        return [field(vjson, **fields)]

    tampered = {"anchor ledger edited": (vedit(anchors=[]), "verification.json's anchors differ"),
                "tools hash edited": (vedit(tools_sha256="0" * 64), "verification.json's tools_sha256 differ"),
                "worker edited": (vedit(worker={"host": "elsewhere"}), "names other worker tools"),
                "closing edited": (vedit(closing={}), "verification.json's closing differ"),
                "pause sites edited": (vedit(pause_sites=[]), "verification.json's pause_sites differ"),
                "source changes edited": (vedit(source_changes=[]), "verification.json's source_changes differ"),
                "integration hash edited": (vedit(integration_sha256="0" * 64), "verification.json's integration_sha256 differ"),
                "pictures claimed viewed": (vedit(pictures_viewed=True), "claims the pictures were viewed"),
                "a picture changed": ([(Path(made["pictures"][0]["path"]), lambda d: d + b"x")], "a picture is missing or changed"),
                "a sheet changed": ([(Path(made["sheets"][0]), lambda d: d + b"x")], "a sheet is missing or changed")}
    outcome2 = {}
    for name, (edits, _want) in tampered.items():
        with planted(edits):
            outcome2[name] = refusal(V.verify_job, ctx2)

    wrong_cfg = dict(cfg2, anchors=[dict(cfg2["anchors"][0], picture=len(sub_samples))] + cfg2["anchors"][1:])
    keep_script = script2.read_bytes()
    keep_inputs, keep_receipt, keep_json = inputs2.read_bytes(), (folder2 / "resource-receipt.json").read_bytes(), vjson.read_bytes()
    script2.write_text(body.replace("@@CONFIG@@", repr(json.dumps(wrong_cfg, indent=1))))
    inputs2.write_text(json.dumps({"files": [{"path": str(script2), "sha256": sha(script2)}]}) + "\n")
    d2 = json.loads(keep_receipt)
    d2["inputs_sha256"] = d2["final_inputs_sha256"] = sha(inputs2)
    (folder2 / "resource-receipt.json").write_text(json.dumps(d2))
    vjson.write_text(json.dumps(dict(made, anchors=wrong_cfg["anchors"]), indent=2))
    off_picture = refusal(V.verify_job, ctx2)
    script2.write_bytes(keep_script)
    inputs2.write_bytes(keep_inputs)
    (folder2 / "resource-receipt.json").write_bytes(keep_receipt)
    vjson.write_bytes(keep_json)

    asr = module("root_asr_as_read", REMOTE / "asr01.py")
    asr.B, asr.VERIFY, asr.TIMELINE = lane, lane / job2, lane / tl / "timeline-summary.json"
    asr_proof = asr.bound_proof()
    attempts = {name: V.re.fullmatch(V.RENDER_ATTEMPT, name) is not None for name in ("", "-r2", "-r9", "-r1", "-r10")}
    forms = {"python3": True, "/opt/homebrew/opt/python@3.15/bin/python3.15": True, "python3.15": True, "bash": False, "python2.7": False, "python3x": False}
    ran_ok = all(V.ran({"command": [k, "-I", "-B", "s.py"]}, "s.py") is want for k, want in forms.items())
    ran_ok = ran_ok and not V.ran({"command": ["python3", "-B", "s.py"]}, "s.py") and not V.ran({"command": ["python3", "-I", "-B", "s.py", "x"]}, "s.py") and not V.ran({}, "s.py")
    admitted = lambda folder: (folder.mkdir(parents=True), (folder / "resource-receipt.json").write_text(json.dumps(
        {"admitted": True, "command": [sys.executable, "-I", "-B", str(script2)]})))
    stale_lane, off_lane = scratch / "stale-lane", scratch / "off-lane"
    admitted(stale_lane / job2)
    admitted(off_lane / job2)
    stale_tools = copy.deepcopy(cfg2["tools"])
    stale_tools["ffmpeg"]["sha256"] = "0" * 64
    with contextlib.redirect_stdout(io.StringIO()) as stale_text:
        stale_code = V.run_job(dict(cfg2, lane=str(stale_lane), tools=stale_tools))
    patched_host, V.HOST = V.HOST, "mac-mini-max"
    with contextlib.redirect_stdout(io.StringIO()) as off_text:
        off_code = V.run_job(dict(cfg2, lane=str(off_lane)))
    off_job = refusal(V.verify_job, ctx2)
    V.HOST = patched_host
    written_nothing = all(sorted(p.name for p in (x / job2).iterdir()) == ["resource-receipt.json"] for x in (stale_lane, off_lane))
    guarded_lane = scratch / "guard-lane"
    (guarded_lane / job2).mkdir(parents=True)
    (guarded_lane / job2 / "resource-receipt.json").write_text(json.dumps({"admitted": True, "command": ["/bin/bash", str(script2)]}))
    with contextlib.redirect_stdout(io.StringIO()) as other_log_text:
        other_code = V.run_job(dict(cfg2, lane=str(guarded_lane)))
    refused_run = other_log_text.getvalue().strip()
    untouched = sorted(p.name for p in (guarded_lane / job2).iterdir()) == ["resource-receipt.json"]
    gave = [n for n in made["strict_video_notes"] if n.startswith("note: PATH gives ")]
    check("G9b-job-with-stand-in-tools", code == 0 and passed["frames"] == 116689 and all(k in made for k in old_keys) and asr_proof["sha256"] == made["sha256"]
          and attempts == {"": True, "-r2": True, "-r9": True, "-r1": False, "-r10": False}
          and gave == [f"note: PATH gives {fake_ffmpeg} and {fake_ffprobe}"] and made["worker"]["ffmpeg"]["path"] == fake_ffmpeg and passed.get("verification_sha256") == sha(vjson)
          and stale_code == 1 and off_code == 1 and written_nothing and "this host's ffmpeg is not" in stale_text.getvalue()
          and "runs only on mac-mini-max" in off_text.getvalue() and "runs only on mac-mini-max" in (off_job or "") and made["video"] == str(master) and made["sha256"] == sha(master)
          and made["fps"] == 60 and len(made["placements"]) == 80 and row_shape and made["pictures_viewed"] is False and len(made["anchors"]) == len(taken)
          and log.getvalue().strip().splitlines()[-1] == V.VERIFY_SUCCESS and all(outcome2[n] and tampered[n][1] in outcome2[n] for n in tampered)
          and "not on their pictures" in (off_picture or "") and ran_ok and other_code == 1 and "does not show this script running" in refused_run and untouched and refusal(V.verify_job, ctx2) is None,
          f"STAND-IN ffmpeg, ffprobe and strict check; no media read or written; real caption screen on the fixture SRT; {len(sub_samples)} of the {len(job_cfg['samples'])} planned pictures, "
          f"all one blank test image. run_job's control flow completes under a receipt recording an absolute interpreter, and verify-job passes. verification.json keeps every "
          f"version 02 key ({old_keys[:5]} and the rest), with frames {made['frames']} taken from the fixture's timelines, placement and chapter rows of the same keys as the sealed record, and adds anchors, pause_sites, closing, source_changes, "
          f"integration_sha256, binding_sha256, tools_sha256 and worker. The strict stand-in reports that PATH gives exactly the pinned tools. Root's real asr01.py bound_proof accepts this verification.json as written (read-only import, paths pointed at the fixture, no ASR). A tool moved after write-job and a run off the worker each refuse before anything is written. Refused after the run: " + "; ".join(f"{n} -> {outcome2[n]!r}" for n in tampered)
          + f"; an anchor off its picture -> {off_picture!r}. `ran` accepts python3 and an absolute python3.NN only; a job under a bash receipt -> {refused_run!r}, nothing written; restored, verify-job passes")

    # ------------------------------------------------------------------ G10 delivered files carry no local path
    needles = ["/Us" + "ers/", "/priv" + "ate/tmp", "/ho" + "me/", "claude" + "-501", "@gm" + "ail", "jema" + "nuel"]
    delivered = [p for p in sorted(FOLDER.rglob("*")) if p.is_file() and "__pycache__" not in p.parts and p.name != "run_checks.txt"]
    hits = [(p.name, n) for p in delivered for n in needles if n in p.read_text(errors="replace")]
    check("G10-no-local-paths", not hits, f"{len(delivered)} delivered files carry none of the {len(needles)} local-path needles; {hits[:4]}")

    # ------------------------------------------------------------------ G11 root's real records for the remote lane
    real_integration = json.loads((REPO / D.S / "integration.json").read_text())
    real_build = {row["path"]: row["sha256"] for row in json.loads((REMOTE / "build-inputs01.json").read_text())["files"]}
    real_rows = {row["relative_path"]: row for row in real_integration["files"]}
    workspace = real_integration["source_workspace"].rstrip("/")
    compiled = [Path(rel).name for rel in real_integration["changed_files"] if real_build.get(f"{workspace}/{rel}") == real_rows[rel]["sha256"]]
    script_row = real_build.get(f"{workspace}/explainer_corpus/logarithm_mysteries/script.tsv")

    def listed(name):
        return {row["path"]: row["sha256"] for row in json.loads((REMOTE / name).read_text())["files"]}

    sent = {**listed("source-transfer01.json"), **listed("audio-transfer01.json"), **listed("support-transfer01.json"), **real_build}
    extra = json.loads((FOLDER / "transfer-extra01.json").read_text())
    pins_abs = {str(REPO / rel): digest for rel, digest in V.PINS.items()}
    on_remote = sorted(p for p, digest in pins_abs.items() if sent.get(p) == digest)
    to_send = sorted(rel for rel, digest in V.PINS.items() if str(REPO / rel) not in sent)
    guard_pinned = {str(REPO / V.GUARD_REL), str(REPO / V.GUARD_SOURCE_REL)} <= {row["path"] for row in json.loads((REMOTE / "guard-pins01.json").read_text())["files"]}
    check("G11-real-remote-records", sorted(compiled) == sorted(V.STAGE09_SOURCES) and script_row == V.SCRIPT_SHA256 and all(sha(REPO / rel) == digest for rel, digest in V.PINS.items())
          and guard_pinned and len(on_remote) == 4 and to_send == sorted(row["path"] for row in extra["files"])
          and all(V.PINS[row["path"]] == row["sha256"] == sha(REPO / row["path"]) for row in extra["files"]),
          f"root's real build-inputs01.json ({len(real_build)} rows) pins all {len(compiled)} repaired sources at their integrated hashes and the script at its pin; all {len(V.PINS)} helper pins "
          f"equal the files here; {len(on_remote)} are in root's transfer or launch records at the pinned hash; the {len(to_send)} that are not are exactly transfer-extra01.json: "
          f"{[Path(x).name for x in to_send]}. These are local copies of root's records: nothing here reached the worker")

    print(f"REMOTE VERIFIER CHECKS: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
