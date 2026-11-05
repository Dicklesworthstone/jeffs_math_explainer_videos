#!/usr/bin/env python3
"""Cheap checks of the stage07 binder and its proposed plan.

    python3 -I -B <this file> <fresh-scratch>

Run from the repo root, with a scratch path that does not exist yet. Writes only under that path.
Exit 0 only if every check passes. No Cargo, render, synthesis or ASR. The canonical crate and
stage07 are hashed before and after, so a concurrent edit by root shows up as a failure here.

Almost everything in sections B to G recomputes from bytes without importing the binder.
Sections H and I exercise the binder itself, in a throwaway mirror of its inputs.
"""
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
FOLDER = Path(__file__).resolve().parent.parent
REPO = Path.cwd()
BINDER = FOLDER / "bind_final80-stage07.py"
PLAN = FOLDER / "bound-plan"
REL = "explainer_corpus/logarithm_mysteries"
LANE = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
S6 = f"{LANE}/combined-repair-stage06"
S7 = f"{LANE}/combined-repair-stage07"
C7 = f"{S7}/candidate/{REL}"
C6 = f"{S6}/{REL}"
OPUS = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
PREFLIGHT = f"{OPUS}/logarithm-stage06-preflight01/preflight.py"
MAP = "narration-direction-map-oliver14.json"
SCRIPT = "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e"

results = []


def check(name, good, detail=""):
    results.append((name, bool(good)))
    print(f"{'ok  ' if good else 'FAIL'} {name}{'  ' + detail if detail else ''}")
    return good


def say(text):
    print(f"\n== {text}")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fsha(path):
    return sha(Path(path).read_bytes())


def tree(path):
    root = Path(path)
    return sha("\n".join(f"{p.relative_to(root)} {fsha(p)}" for p in sorted(root.rglob("*")) if p.is_file() and "target" not in p.relative_to(root).parts).encode())


def run(args, cwd=REPO):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True)


def scene(path):
    text = Path(path).read_text(encoding="utf-8")
    at = text.find("#[cfg(test)]")
    return text.split("\n"), (text if at < 0 else text[:at])


def script_rows(path):
    return dict(line.split("\t", 1) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip() and not line.startswith("id\t") and "\t" in line)


def registered(chapters):
    src = (Path(chapters) / "mod.rs").read_text(encoding="utf-8")
    return [f"{m}.rs" for m in re.findall(r"make: \|kit\| Box::new\((\w+)::", src)]


def says(chapters):
    rows = {}
    for name in registered(chapters):
        for n, line in enumerate(scene(Path(chapters) / name)[1].split("\n"), 1):
            if line.strip().startswith("//"):
                continue
            m = re.search(r'kit\.say\(stage, "(\d\d_\d+)"\)', line)
            if m:
                rows.setdefault(m.group(1), []).append((name, n))
    return rows


def load(rel):
    return json.loads((PLAN / rel).read_text(encoding="utf-8"))


def main():
    if len(sys.argv) != 2 or Path(sys.argv[1]).exists():
        sys.exit("usage: run_checks.py <scratch path that does not exist yet>")
    scratch = Path(sys.argv[1]).resolve()
    scratch.mkdir(parents=True)
    before = {p: tree(p) for p in (REL, S7, f"{S6}/bound-plan", f"{LANE}/narration-request-rounding01", f"{OPUS}/logarithm-opening-timing-repair01", f"{OPUS}/logarithm12-readability-repair01")}

    # ------------------------------------------------------------------ A
    say("A. The binder writes the same bytes twice, and the delivered plan is those bytes")
    one, two = scratch / "out1", scratch / "out2"
    a = run([sys.executable, "-I", "-B", str(BINDER), "--out", str(one)])
    b = run([sys.executable, "-I", "-B", str(BINDER), "--out", str(two)])
    check("binder exits 0 twice", a.returncode == 0 and b.returncode == 0, (a.stderr + b.stderr).strip()[:300])
    names = sorted(p.relative_to(one).as_posix() for p in one.rglob("*") if p.is_file())
    check("nine outputs", len(names) == 9, f"{len(names)}")
    check("deterministic", all(fsha(one / n) == fsha(two / n) for n in names) and a.stdout == b.stdout)
    check("delivered plan equals a fresh run", all(fsha(one / n) == fsha(PLAN / n) for n in names) and sorted(p.relative_to(PLAN).as_posix() for p in PLAN.rglob("*") if p.is_file()) == names)
    c = run([sys.executable, "-I", "-B", str(BINDER), "--check", "--out", str(PLAN)])
    check("--check passes on the delivered plan", c.returncode == 0, c.stderr.strip()[:200])
    c = run([sys.executable, "-I", "-B", str(BINDER), "--print-pins"])
    table = re.search(r"# PINS-BEGIN\n(.*?)# PINS-END\n", BINDER.read_text(encoding="utf-8"), re.S).group(1)
    check("--print-pins reproduces the binder's literal table", c.stdout == table, f"{table.count(': ')} pins")

    # ------------------------------------------------------------------ B
    say("B. Inputs: the exact bytes, against root's receipts")
    check("script is b89eab60", fsha(f"{C7}/script.tsv") == SCRIPT)
    check("before script is f201d2dc", fsha(f"{S7}/before-rounding-wording01/script.tsv").startswith("f201d2dc"))
    receipts = {
        "integration-partial01.json": {"matrix_logs.rs": "7df6f8681099c76dcd1f32494e2edcbb07b427c54178d5c557d5801052f481b0", "doing_nothing.rs": "7f445be212c586093d2092113b086d809dcc189619985baf1c8f0c407c709b47"},
        "integration-partial03.json": {"undoing_growth.rs": "4dc4c555122d9cd5423d7094336c1d5e6f7f9afb77d6850a38bc0148c20180ef", "unreachable.rs": "c822e8b46796e79e2f8461872fe538de711a4fde755913ec57696710d0cc28a7"},
        "integration-partial04.json": {"ratio_ruler.rs": "198d4a0586369258d845a21ee857eeb51e34357b61fe40398e5789379a6b5fd1", "folded_decade.rs": "c4f023ce5284dd768767b28c9d68a4da1e05c40385eb750dc9d4fbe583b6347e", "slide_rule.rs": "68d4b011fb4d90033ef1ae69aedef1639e12c11ce3624dbb5c378152d39b1ecd"},
        "integration-partial05.json": {"what_it_forgets.rs": "6951d3b2f157387683efaa3ec7c9d9d12a23c8448df247531cd5f0081bab1e28"},
    }
    for receipt, files in receipts.items():
        body = (Path(S7) / receipt).read_text(encoding="utf-8")
        for name, want in files.items():
            check(f"{name} is the bytes {receipt} records", fsha(f"{C7}/src/chapters/{name}") == want and want in body, want[:12])
    body = (Path(S7) / "integration-partial02.json").read_text(encoding="utf-8")
    check("integration-partial02 records the script and the 01_5 request", SCRIPT in body and "f0a6d133ce8d07e4fcb9242c0e7a70966611ce1baca1501d3b0242c58b9e9b92" in body)
    rows = [line.split("  ", 1) for line in (PLAN / "inputs.sha256").read_text().splitlines()]
    moved = [p for h, p in rows if not Path(p).is_file() or fsha(p) != h]
    check("every line of inputs.sha256 still holds", not moved, f"{len(rows)} files; moved: {moved[:4]}")
    differ = sorted(p.relative_to(C7).as_posix() for p in Path(C7).rglob("*") if p.is_file() and fsha(p) != fsha(Path(C6) / p.relative_to(C7)))
    want = sorted(["script.tsv"] + [f"src/chapters/{n}.rs" for n in ("doing_nothing", "folded_decade", "matrix_logs", "ratio_ruler", "slide_rule", "undoing_growth", "unreachable", "what_it_forgets")])
    check("candidate differs from stage06's crate only in the nine integrated files", differ == want, f"{differ}")
    check("oracle.py and facts.rs equal stage06's, so its oracle table serves", fsha(f"{C7}/oracle.py") == fsha(f"{C6}/oracle.py") and fsha(f"{C7}/src/facts.rs") == fsha(f"{C6}/src/facts.rs"))
    o = run([sys.executable, "-I", "-B", f"{C7}/oracle.py"])
    check("candidate's oracle.py reproduces stage06's oracle04.tsv", o.returncode == 0 and o.stdout == Path(f"{S6}/oracle04.tsv").read_text(encoding="utf-8"), f"{len(o.stdout.splitlines())} rows")

    # ------------------------------------------------------------------ C
    say("C. All 80 cues")
    texts = script_rows(f"{C7}/script.tsv")
    old_texts = script_rows(f"{C6}/script.tsv")
    cues = list(texts)
    check("80 cues", len(cues) == 80, f"{len(cues)}; {sum(len(t.split()) for t in texts.values())} words")
    check("exactly one cue changed since stage06, and it is 01_5", [c for c in cues if texts[c] != old_texts.get(c)] == ["01_5"] and list(old_texts) == cues)
    check("the change is exactly the reviewed phrase", old_texts["01_5"].replace("its true position is one point", "its position is about one point") == texts["01_5"])
    check("01_5's request file is its exact text", Path(f"{LANE}/narration-request-rounding01/01_5.txt").read_bytes() == texts["01_5"].encode() and sha(texts["01_5"].encode()).startswith("f0a6d133"))
    said = says(f"{C7}/src/chapters")
    check("each cue said exactly once, in script order", list(said) == cues and all(len(v) == 1 for v in said.values()))
    units = load(f"{REL}/narration_units.json")
    check("units: one per cue, in order, exact text", [u["line_ids"] for u in units["units"]] == [[c] for c in cues] and all(u["text"] == texts[u["line_ids"][0]] for u in units["units"]) and [u["id"] for u in units["units"]] == [f"oliver_{c}" for c in cues])
    check("units bind the script", units["script_sha256"] == SCRIPT and units["direction_contract"]["script_sha256"] == SCRIPT)
    template = load("phrase-timings-unmeasured-template.json")
    check("template: all 80 cues with their text hashes, all unmeasured", list(template["cues"]) == cues and all(template["cues"][c]["text_sha256"] == sha(texts[c].encode()) and template["cues"][c]["wav_sha256"] is None for c in cues) and all(m["seconds"] is None for c in cues for m in template["cues"][c]["marks"]))
    delta = load("cue-delta.json")
    check("cue delta: all 80, each current request file holds the exact bytes", [d["cue"] for d in delta["cues"]] == cues and all(Path(d["current_request"]["path"]).read_bytes() == texts[d["cue"]].encode() for d in delta["cues"]), f"{delta['counts']}")
    check("cue delta: one rewording, nine splits", delta["counts"]["reworded_since_stage06"] == 1 and delta["counts"]["split_cues"] == 9 and [d["cue"] for d in delta["cues"] if d["reworded_since_stage06"]] == ["01_5"])

    # ------------------------------------------------------------------ D
    say("D. All 15 holds, recomputed from the candidate's bytes")
    m = load(f"{REL}/docs/{MAP}")
    check("the map binds the script", m["scripts"]["logarithm_mysteries"] == SCRIPT)
    check("13 predictions and 2 readings", m["hold_kinds"] == {"prediction": 13, "reading": 2} and len(m["visual_holds"]) == 15 and len(m["overrides"]) == 15)
    check("contract names the map's exact bytes", units["direction_contract"]["map"] == {"path": f"{REL}/docs/{MAP}", "sha256": fsha(PLAN / REL / "docs" / MAP)})
    bad = []
    for o, v in zip(m["overrides"], m["visual_holds"]):
        name = v["path"].rsplit("/", 1)[1]
        path = Path(C7) / "src/chapters" / name
        lines = path.read_text(encoding="utf-8").split("\n")
        at = cues.index(v["cue"])
        if v["sha256"] != fsha(path) or v["evidence"] != {"path": v["path"], "sha256": v["sha256"]}:
            bad.append((v["cue"], "file hash"))
        if lines[v["line"] - 1].strip() != v["marker"]:
            bad.append((v["cue"], "marker"))
        if said[v["cue"]] != [(name, said[v["cue"]][0][1])] or not said[v["cue"]][0][1] < v["line"]:
            bad.append((v["cue"], "say"))
        if sha("\n".join(lines[said[v["cue"]][0][1] - 1 : v["line"]]).encode()) != v["interval_sha256"]:
            bad.append((v["cue"], "interval"))
        if cues[at + 1] != v["next_cue"]:
            bad.append((v["cue"], "next cue"))
        if o["cue"] != v["cue"] or o["after"] != texts[v["cue"]]:
            bad.append((v["cue"], "override"))
    check("hash, marker, say, interval, next cue and override, on all 15", not bad, f"{bad}")
    # Stage06's own values, by the same formula, on stage06's bytes: the positive control.
    m6 = json.loads(Path(f"{S6}/bound-plan/{REL}/docs/narration-direction-map-oliver13.json").read_text(encoding="utf-8"))
    said6 = says(f"{C6}/src/chapters")
    bad = []
    for v in m6["visual_holds"]:
        lines = (Path(C6) / "src/chapters" / v["path"].rsplit("/", 1)[1]).read_text(encoding="utf-8").split("\n")
        if sha("\n".join(lines[said6[v["cue"]][0][1] - 1 : v["line"]]).encode()) != v["interval_sha256"]:
            bad.append(v["cue"])
    check("the same formula reproduces all 15 of stage06's intervals on stage06's bytes", not bad and len(m6["visual_holds"]) == 15, f"{bad}")
    by = {v["cue"]: v for v in m["visual_holds"]}
    # Values stated in the repair handoffs, before this binder existed.
    expected = {
        "00_4": ("ratio_ruler.rs", 371, "86bf44bb70227da9e2f416862ba78a056faaaa05b3b7955e8bc009d769001515", "opening handoff"),
        "02_2": ("slide_rule.rs", 204, "20094536ae2191f558b6ca76bc94b089e10f8fbe2bf78647b08036b306a75215", "opening handoff"),
        "10_4": ("doing_nothing.rs", 602, "9fe875942e620939da7839720e40e029c830cfe152247ae59de1db3c4c2d8927", "chapter 10 handoff"),
        "11_3": ("unreachable.rs", 1070, "e34e65ad69a71865c956f2cf6e3d5270433370e8b6e79e5f8defc221555771db", "chapter 11 handoff"),
        "11_20": ("unreachable.rs", 1590, "4a4e6c72d1f92e5e88e7430c03e0f0f062e3e7972184736bfc911a4313fae5e0", "chapter 11 handoff"),
        "12_5": ("what_it_forgets.rs", 1123, "b238a92968284627ce972479bb1c5a93ac5f5df33efee0b17ddea5d362c8387f", "chapter 12 handoff"),
    }
    for cue, (name, line, interval, source) in expected.items():
        v = by[cue]
        check(f"{cue} agrees with the {source}", v["path"].endswith("/" + name) and v["line"] == line and v["interval_sha256"] == interval, f"{name}:{line} {interval[:12]}")
    cmp = load("stage07-delta.json")
    s = cmp["hold_summary"]
    check("no hold's marker line, say line or marker text changed", not s["marker_line_moved"] and not s["say_line_moved"] and not s["marker_text_changed"])
    check("three intervals changed: 02_2, 10_4, 11_20", s["interval_changed"] == ["02_2", "10_4", "11_20"])
    check("seven file hashes changed, eight holds untouched", s["file_hash_changed"] == ["00_4", "02_2", "04_6", "10_4", "11_3", "11_20", "12_5"] and len(s["unchanged_entirely"]) == 8)
    check("no held cue's text changed", not [h["cue"] for h in cmp["holds"] if h["cue_text_changed"]])
    check("01_5 is not a held cue", not cmp["wording"]["is_a_held_cue"] and "01_5" not in by)

    audit = load("hold-audit.json")
    reading = {r["cue"]: r for r in audit["reading_holds"]}
    lines10 = scene(f"{C7}/src/chapters/doing_nothing.rs")[1]
    textbook = re.search(r'^const TEXTBOOK: &str = "([^"]*)";', lines10, re.M).group(1)
    cited = re.search(r'^const SOURCE: &str = "([^"]*)";', lines10, re.M).group(1)
    check("10_4's reading counts the prefix once: 197 characters, as stage06 did", reading["10_4"]["fine_print_chars"] == 197 == len(textbook) + len(cited) and cited.startswith("precisely: "), f"{len(textbook)} + {len(cited)}; double counting would say 208")
    check("10_4's floor rate is unchanged", reading["10_4"]["characters_per_second_at_floor"] == 14.924)
    check("10_4 stays conditional, with no window claimed", reading["10_4"]["conditional"] is True and reading["10_4"]["window_seconds"] is None)
    check("11_20's window is unchanged: 159 + 107 characters, 17.7333 s", reading["11_20"]["rule_chars"] == 159 and reading["11_20"]["row_chars"] == 107 and reading["11_20"]["window_seconds"] == 17.7333)
    ln = Path(f"{C7}/src/chapters/doing_nothing.rs").read_text(encoding="utf-8").split("\n")
    check("doing_nothing.rs:598 is the reading's kit.hold", ln[597].strip() == "kit.hold(stage)?;")
    ln = Path(f"{C7}/src/chapters/unreachable.rs").read_text(encoding="utf-8").split("\n")
    check("unreachable.rs:1555 is the reading's kit.hold", ln[1554].strip() == "kit.hold(stage)?;")

    # ------------------------------------------------------------------ E
    say("E. SITES: every wait, against the opening handoff's explicit list")
    waits = {(w["file"], w["line"]): w for w in audit["other_waits"]}
    rr = Path(f"{C7}/src/chapters/ratio_ruler.rs").read_text(encoding="utf-8").split("\n")
    check("ratio_ruler.rs:181 is gone and :210 is the dots, picture pacing, under 00_2", ("ratio_ruler.rs", 181) not in waits and waits[("ratio_ruler.rs", 210)]["class"] == "picture_pacing" and waits[("ratio_ruler.rs", 210)]["cue"] == "00_2" and rr[209].strip() == "stage.wait(0.18)?;")
    check("ratio_ruler.rs:555 is read_out's wait, word sync, census under 00_6", waits[("ratio_ruler.rs", 555)]["class"] == "word_sync_helper" and waits[("ratio_ruler.rs", 555)]["cue"] == "00_6" and rr[554].strip() == "super::draw::wait_until(stage, at)?;")
    check("ratio_ruler.rs:509 is still the helper tail", waits[("ratio_ruler.rs", 509)]["class"] == "helper_tail" and rr[508].strip() == "kit.hold(stage)")
    check("13 other waits: 12 + 1 added, none removed", len(waits) == 13 and [(r["file"], r["line"]) for r in cmp["other_waits"]["added"]] == [("ratio_ruler.rs", 555)] and not cmp["other_waits"]["removed"])
    check("only the dots moved or changed cue", [(r["file"], r["line"]["stage06"], r["line"]["stage07"]) for r in cmp["other_waits"]["moved_or_recued"]] == [("ratio_ruler.rs", 181, 210)])
    timing = re.compile(r"\b(predict_pause|kit\.listen|kit\.sonify|stage\.wait|wait_frames|wait_until|kit\.hold)\(")
    found, predicts, breaths = [], 0, 0
    for name in registered(f"{C7}/src/chapters"):
        for n, line in enumerate(scene(Path(C7) / "src/chapters" / name)[1].split("\n"), 1):
            if line.strip().startswith("//") or re.search(r"\bfn (predict|wait_until)\(", line):
                continue
            if timing.search(line):
                found.append((name, n))
            predicts += bool(re.search(r"\bpredict\(", line))
            breaths += "kit.music_breath(" in line
    check("own scan: every wait in the registered scene code is classified", sorted(found) == sorted(waits), f"{len(found)}; only in source {sorted(set(found) - set(waits))[:3]}; only in plan {sorted(set(waits) - set(found))[:3]}")
    check("own scan: 13 predict calls, 2 reading windows", predicts == 13 and breaths == 2, f"{predicts}, {breaths}")
    check("13 chapter finishes; only folded_decade's moved", len(audit["chapter_finishes"]) == 13 and [(r["file"], r["line"]["stage06"], r["line"]["stage07"]) for r in cmp["chapter_finishes_moved"]] == [("folded_decade.rs", 441, 448)])
    check("stale pair 00_1 -> 00_2 lists no wait in its final interval", next(r for r in cmp["stale_pairs"]["stage07"] if r["cue"] == "00_1")["other_waits_in_final_interval"] == [])
    check("no stale pair became a hold", not ({("00_1", "00_2"), ("01_1", "01_2"), ("03_1", "03_2"), ("04_1", "04_2")} & {(v["cue"], v["next_cue"]) for v in m["visual_holds"]}))
    check("no helper or consumer row changed since stage06", not cmp["helper_and_consumer_rows"]["changed_since_stage06"], f"{cmp['helper_and_consumer_rows']['count']} rows")

    # ------------------------------------------------------------------ F
    say("F. Phrase and sentence hooks")
    anchors = load("phrase-anchors.json")
    a6 = json.loads(Path(f"{S6}/bound-plan/phrase-anchors.json").read_text(encoding="utf-8"))
    key = lambda r: (r["cue"], r["phrase"], r["byte"], r["byte_end"], r["sentence_index"], r["word_index"], r["const"])
    check("24 anchors, the same cue, phrase, byte, sentence and word as stage06", sorted(map(key, anchors["anchors"])) == sorted(map(key, a6["anchors"])) and len(anchors["anchors"]) == 24)
    check("every anchor's bytes land on its phrase in the current text", all(texts[r["cue"]].encode()[r["byte"] : r["byte_end"]] == r["phrase"].encode() for r in anchors["anchors"]))
    check("no anchor's hook line moved", not cmp["phrase_anchors"]["hook_lines_moved"] and not cmp["phrase_anchors"]["byte_or_text_position_changed"])
    check("all measured seconds are null", anchors["counts"]["measured"] == 0 and all(r["measured_seconds"] is None for r in anchors["anchors"]))
    check("no anchor on the opening's 00_1 or on 01_5", not [r for r in anchors["anchors"] if r["cue"] in ("00_1", "01_5")] and not template["cues"]["00_1"]["marks"] and not template["cues"]["01_5"]["marks"])
    check("ratio_ruler.rs has no until_phrase, phrase_time or phrase_marks", not re.search(r"until_phrase|phrase_time|phrase_marks", scene(f"{C7}/src/chapters/ratio_ruler.rs")[1]))
    check("folded_decade.rs has no until_phrase, phrase_time or phrase_marks", not re.search(r"until_phrase|phrase_time|phrase_marks", scene(f"{C7}/src/chapters/folded_decade.rs")[1]))
    for name in ("slide_rule.rs", "undoing_growth.rs", "doing_nothing.rs", "unreachable.rs", "what_it_forgets.rs"):
        check(f"{name} has as many phrase_marks uses as stage06", scene(f"{C7}/src/chapters/{name}")[1].count("phrase_marks(") == scene(f"{C6}/src/chapters/{name}")[1].count("phrase_marks("))
    h = cmp["sentence_hooks"]
    check("literal sentence hooks 282 -> 285", h["literal"] == {"stage06": 282, "stage07": 285} and audit["counts"]["literal_sentence_hooks"] == 285)
    check("the three added are the opening handoff's, and none removed", [(r["file"], r["hook"], r["cue"], r["sentence"], r["lines"]) for r in h["added"]] == [("folded_decade.rs", "sentence_at", "03_3", 0, [344]), ("folded_decade.rs", "sentence_at", "03_3", 1, [344]), ("ratio_ruler.rs", "sentence_at", "00_1", 1, [541])] and not h["removed"])
    check("cue_differs_from_current gains ratio_ruler.rs:541", {"file": "ratio_ruler.rs", "line": 541, "hook": "sentence_at", "cue": "00_1", "current": "00_6"} in audit["sentence_hooks"]["cue_differs_from_current"] and len(audit["sentence_hooks"]["cue_differs_from_current"]) == 3)
    check("the one computed sentence hook is unchanged", [r["line"] for r in audit["sentence_hooks"]["non_literal"]] == [339])
    check("01_5 keeps 4 sentences, and its three hooks stay in range", cmp["wording"]["sentences"] == {"stage06": 4, "stage07": 4} and [r["sentence"] for r in cmp["wording"]["sentence_reads_on_cue"]] == [1, 2, 3])
    check("01_5 carries no anchor and building_ruler.rs is unchanged", cmp["wording"]["phrase_anchors_on_cue"] == 0 and fsha(f"{C7}/src/chapters/building_ruler.rs") == fsha(f"{C6}/src/chapters/building_ruler.rs"))
    check("the picture already says about: pos(3) \\approx", r"\mathrm{{pos}}(3) \approx {}" in scene(f"{C7}/src/chapters/building_ruler.rs")[1])
    est = anchors["opening_read_out_estimates"]
    source = scene(f"{C7}/src/chapters/ratio_ruler.rs")[1]
    phrases = re.findall(r'"([^"]*)"', re.search(r"const READ_PHRASES: \[&str; 6\] = \[\n(.*?)\];", source, re.S).group(1))
    rest = texts["00_1"].split(". ", 1)[1]
    check("00_1's six estimates reproduce read_shares from source and script", [r["phrase"] for r in est["rows"]] == phrases and all(abs(r["share_of_rest_of_cue"] - rest.index(r["phrase"]) / len(rest)) < 1e-4 for r in est["rows"]), f"{[r['share_of_rest_of_cue'] for r in est['rows']]}")
    check("they are marked current baseline, estimates, unmeasured", est["status"].startswith("CURRENT BASELINE") and all(r["measured_seconds"] is None for r in est["rows"]))
    share = anchors["sentence_share_hooks"]["rows"]
    check("the 3 sentence-share hooks are unchanged", [(r["cue"], r["const"], r["line"]) for r in share] == [("12_20", "AXIS_RESET", 1199), ("12_20", "AXIS_CHIP", 1202), ("12_21", "TIE_AT", 1280)])

    # ------------------------------------------------------------------ G
    say("G. The rest of the plan's bindings, and the map's name")
    disp = load("direction-dispositions-proposed.json")
    check("dispositions bind the map, all 15 native, evidence is hold-audit's bytes", disp["map_sha256"] == fsha(PLAN / REL / "docs" / MAP) and [r["cue"] for r in disp["direction_dispositions"]] == [v["cue"] for v in m["visual_holds"]] and all(r["kind"] == "native_visual_hold" and r["evidence"] == {"path": f"{OPUS}/logarithm-stage07-binder01/bound-plan/hold-audit.json", "sha256": fsha(PLAN / "hold-audit.json")} for r in disp["direction_dispositions"]))
    alt = scratch / "out-final"
    a = run([sys.executable, "-I", "-B", str(BINDER), "--out", str(alt), "--final-out", f"{S7}/bound-plan"])
    differ = sorted(n for n in names if fsha(alt / n) != fsha(PLAN / n))
    check("--final-out changes only the dispositions file", a.returncode == 0 and differ == ["direction-dispositions-proposed.json"] and f"{S7}/bound-plan/hold-audit.json" in (alt / differ[0]).read_text(encoding="utf-8"), f"{differ}")
    check("candidate holds no oliver14; canonical holds no oliver14 either", not (Path(C7) / "docs" / MAP).exists() and not (Path(REL) / "docs" / MAP).exists())
    canon = json.loads(Path(f"{REL}/docs/narration-direction-map-oliver13.json").read_text(encoding="utf-8"))
    check("canonical's oliver13 binds the old script, so oliver13 cannot be reused", canon["scripts"]["logarithm_mysteries"].startswith("f201d2dc"))
    check("candidate's own narration_units.json is stale, and the plan's is not it", json.loads(Path(f"{C7}/narration_units.json").read_text(encoding="utf-8"))["script_sha256"] != SCRIPT and fsha(f"{C7}/narration_units.json") != fsha(PLAN / REL / "narration_units.json"))
    text = {n: (PLAN / n).read_text(encoding="utf-8") for n in names}
    needles = ["/" + "Users/", "/" + "private/", "/" + "tmp/"]
    check("no absolute home path and no scratch path in the plan", not [n for n, t in text.items() if any(x in t for x in needles)])

    # ------------------------------------------------------------------ H
    say("H. Stale input: refused before anything is computed, in a mirror of the inputs")
    mirror = scratch / "mirror"
    pins = {}
    exec(table, {}, pins)
    pins = pins["PINS"]
    def place(rel):
        target = mirror / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(rel, target)
    for rel in set(pins) | {p for _, p in rows}:
        place(rel)
    # The crates, the tools and the plan-level files the consumers read.
    for p in list(Path("tools").glob("*.py")) + list(Path("crates/explainer_kit/src").glob("*.rs")) + [Path("crates/explainer_score/src/score.rs")]:
        place(p.as_posix())
    for p in Path(C7).rglob("*"):
        if p.is_file():
            place(p.as_posix())
    for p in Path(REL).rglob("*"):
        if p.is_file() and "target" not in p.relative_to(REL).parts:
            place(p.as_posix())
    for p in Path(f"{LANE}/narration-request-final80-01").iterdir():
        if p.is_file():
            place(p.as_posix())
    preview = f"{LANE.rsplit('/', 1)[0]}/five-film-release-20261027/logarithm-combined-repair-preview01/resource-receipt.json"
    place(preview)
    (mirror / "mirror-tree.marker").write_text("a throwaway mirror\n")
    def bind(*args):
        return run([sys.executable, "-I", "-B", str(BINDER), *args], cwd=mirror)
    pos = scratch / "out-mirror"
    r = bind("--out", str(pos))
    check("positive control: the unchanged mirror binds, with identical bytes", r.returncode == 0 and all(fsha(pos / n) == fsha(PLAN / n) for n in names), r.stderr.strip()[:300])
    CH = f"{C7}/src/chapters"
    plants = {
        "the script": f"{C7}/script.tsv",
        "a held chapter": f"{CH}/what_it_forgets.rs",
        "the opening": f"{CH}/ratio_ruler.rs",
        "an unchanged chapter": f"{CH}/building_ruler.rs",
        "an unregistered file": f"{CH}/doing_nothing.rs.orig",
        "draw.rs": f"{CH}/draw.rs",
        "mod.rs": f"{CH}/mod.rs",
        "main.rs": f"{C7}/src/main.rs",
        "facts.rs": f"{C7}/src/facts.rs",
        "oracle.py": f"{C7}/oracle.py",
        "the storyboard": f"{C7}/docs/storyboard.md",
        "the stale map": f"{C7}/docs/narration-direction-map-oliver11.json",
        "the candidate's units": f"{C7}/narration_units.json",
        "the kit": "crates/explainer_kit/src/narration.rs",
        "the score": "crates/explainer_score/src/score.rs",
        "a tool": "tools/coherent_narration.py",
        "the inventory": f"{LANE}/narration-request-inventory01.json",
        "the request plan": f"{LANE}/narration-request-plan01.json",
        "the final-80 list": f"{LANE}/narration-request-final80-01/requests.json",
        "the rounding request": f"{LANE}/narration-request-rounding01/01_5.txt",
        "the rounding review": f"{LANE}/rounding-wording-repair01/review.json",
        "the before script": f"{S7}/before-rounding-wording01/script.tsv",
        "an integration receipt": f"{S7}/integration-partial05.json",
        "stage06's oracle table": f"{S6}/oracle04.tsv",
        "stage06's plan": f"{S6}/bound-plan/hold-audit.json",
        "stage06's chapter": f"{C6}/src/chapters/doing_nothing.rs",
    }
    for label, rel in plants.items():
        path = mirror / rel
        was = path.read_bytes()
        path.write_bytes(was + b"\n")
        out = scratch / "must-not-exist"
        r = bind("--out", str(out))
        path.write_bytes(was)
        check(f"one byte more in {label}: refused", r.returncode == 3 and "STALE INPUT" in r.stderr and rel in r.stderr and not out.exists(), f"exit {r.returncode}")
    real = {
        "stage06's script": (f"{C7}/script.tsv", f"{C6}/script.tsv"),
        "the what_it_forgets.rs before chapter 12's repair": (f"{CH}/what_it_forgets.rs", f"{S7}/before-chapter12-integration01/what_it_forgets.rs"),
        "the ratio_ruler.rs before the opening repair": (f"{CH}/ratio_ruler.rs", f"{S7}/before-opening-integration01/ratio_ruler.rs"),
    }
    for label, (rel, older) in real.items():
        path = mirror / rel
        was = path.read_bytes()
        path.write_bytes(Path(older).read_bytes())
        out = scratch / "must-not-exist"
        r = bind("--out", str(out))
        path.write_bytes(was)
        check(f"{label}: refused", r.returncode == 3 and rel in r.stderr and not out.exists(), f"exit {r.returncode}")
    path = mirror / f"{CH}/doing_nothing.rs"
    was = path.read_bytes()
    path.unlink()
    out = scratch / "must-not-exist"
    r = bind("--out", str(out))
    path.write_bytes(was)
    check("a missing chapter: refused", r.returncode == 3 and "MISSING" in r.stderr and not out.exists())
    stray = mirror / CH / "new_chapter.rs"
    stray.write_text("// stray\n")
    r = bind("--out", str(out))
    stray.unlink()
    check("a stray file in the chapter folder: refused", r.returncode == 3 and "LISTING" in r.stderr and "new_chapter.rs" in r.stderr and not out.exists())
    for label, target in (("canonical", REL), ("canonical's docs", f"{REL}/docs"), ("the candidate", f"{S7}/candidate"), ("the candidate's crate", C7)):
        r = bind("--out", target)
        check(f"--out into {label}: refused", r.returncode == 1 and "refused" in (r.stderr + r.stdout))
    check("those refusals wrote nothing", not (mirror / REL / "docs" / MAP).exists() and not (mirror / REL / "hold-audit.json").exists() and not (mirror / C7 / "hold-audit.json").exists())
    r = bind("--final-out", "/somewhere/absolute", "--out", str(scratch / "must-not-exist"))
    check("an absolute --final-out: refused", r.returncode == 1 and not (scratch / "must-not-exist").exists())
    held = scratch / "out-held"
    r = bind("--out", str(held))
    victim = held / "hold-audit.json"
    victim.write_text("someone else's bytes\n")
    r = bind("--out", str(held))
    check("an existing output that differs is not overwritten", r.returncode == 1 and "DIFFERS, not overwritten: hold-audit.json" in r.stdout and victim.read_text() == "someone else's bytes\n")
    r = bind("--check", "--out", str(scratch / "out-empty"))
    check("--check writes nothing, and fails on a missing plan", r.returncode == 1 and not (scratch / "out-empty").exists())
    r = bind("--print-pins", "--out", str(scratch / "x"))
    check("--print-pins with other arguments is refused", r.returncode == 1)

    # ------------------------------------------------------------------ I
    say("I. Semantic stops, with the pins bypassed in the mirror so a planted change reaches them")
    driver = FOLDER / "checks/build_unpinned.py"
    def plant(label, rel, old, new, expect):
        path = mirror / rel
        was = path.read_bytes()
        text = was.decode("utf-8")
        if old not in text:
            check(f"plant {label}", False, f"the text to plant on is gone: {old[:50]!r}")
            return
        path.write_text(text.replace(old, new, 1), encoding="utf-8")
        r = run([sys.executable, "-I", "-B", str(driver), str(BINDER)], cwd=mirror)
        path.write_bytes(was)
        check(f"plant {label}: stops", r.stdout.startswith("STOP:") and expect in r.stdout, r.stdout.strip()[:150] or r.stderr.strip()[:150])
    r = run([sys.executable, "-I", "-B", str(driver), str(BINDER)], cwd=mirror)
    check("control: with nothing planted and pins bypassed, it builds", r.stdout.strip() == "BUILT", r.stdout.strip()[:200] + r.stderr.strip()[:200])
    plant("an unclassified wait", f"{CH}/building_ruler.rs", 'kit.until_sentence(stage, "01_5", 1)?;', 'stage.wait(1.0)?;\n        kit.until_sentence(stage, "01_5", 1)?;', "not classified")
    plant("the dots' wait removed", f"{CH}/ratio_ruler.rs", "stage.wait(0.18)?;", "", "classified waits that are not in the source")
    plant("a line moving the dots' wait", f"{CH}/ratio_ruler.rs", 'kit.say(stage, "00_2")?;', 'kit.say(stage, "00_2")?;\n        let _x = 0;', "not classified")
    plant("a prediction removed", f"{CH}/building_ruler.rs", "predict(&kit,", "not_predict(&kit,", "predict calls")
    plant("a line above 10_4's kit.hold", f"{CH}/doing_nothing.rs", "let shown_from = stage.scene().time().to_f64();", "let shown_from = stage.scene().time().to_f64();\n        let _x = 0;", "not classified")
    plant("a third reading window", f"{CH}/building_ruler.rs", 'kit.say(stage, "01_5")?;', 'kit.say(stage, "01_5")?;\n        kit.music_breath(stage, 1.0)?;', "reading windows changed")
    plant("a longer theorem", f"{CH}/unreachable.rs", "const READING_RATE: f64 = 15.0;", "const READING_RATE: f64 = 14.0;", "159 + 107")
    plant("SOURCE without its prefix", f"{CH}/doing_nothing.rs", 'const SOURCE: &str = "precisely: ', 'const SOURCE: &str = "', "prefix")
    plant("a sentence hook out of range", f"{CH}/building_ruler.rs", 'kit.until_sentence(stage, "01_5", 3)?;', 'kit.until_sentence(stage, "01_5", 9)?;', "does not exist")
    plant("a phrase hook it cannot read", f"{CH}/building_ruler.rs", 'kit.until_sentence(stage, "01_5", 1)?;', 'until_phrase(stage, kit, x, y)?;\n        kit.until_sentence(stage, "01_5", 1)?;', "cannot read")
    plant("a read phrase the cue does not say", f"{CH}/ratio_ruler.rs", '"seventeen sixty",', '"seventeen seventy",', "00_1")
    plant("a cue said out of order", f"{CH}/building_ruler.rs", 'kit.say(stage, "01_5")?;', 'kit.say(stage, "01_6")?;', "do not say every script cue")
    plant("another word in 01_5", f"{C7}/script.tsv", "its position is about one point", "its position is roughly one point", "not the reviewed rounding wording")
    plant("another cue reworded", f"{C7}/script.tsv", "Here are seven notes", "Here are eight notes", "more than the 01_5")
    plant("a chapter with no receipt", f"{CH}/building_ruler.rs", "// 01_5: the keyboard callback; then the true position", "// 01_5: the keyboard callback", "not the integration receipts' files")
    plant("a chapter no receipt names", f"{CH}/ratio_ruler.rs", "// 00_1: the title alone", "// 00_1: the title", "does not name the current hash")
    plant("facts.rs differing", f"{C7}/src/facts.rs", "fn ", "fn  ", "oracle04.tsv no longer serves")

    # ------------------------------------------------------------------ J
    say("J. A second census: RubyRaven's preflight, unmodified, on this plan, in the mirror")
    for n in names:
        for base in (f"{S7}/candidate/bound-plan", f"{OPUS}/logarithm-stage07-binder01/bound-plan"):
            target = mirror / base / n
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(PLAN / n, target)
    shutil.copyfile(f"{S6}/oracle04.tsv", mirror / S7 / "candidate/oracle04.tsv")
    # tools/narration_director.py needs numpy, which /usr/bin/python3 lacks here. Use the first
    # python3 on PATH that has it, so the director's real validate_map runs rather than a WARN.
    interpreter = sys.executable
    for candidate in filter(None, [shutil.which("python3"), sys.executable]):
        if run([candidate, "-I", "-B", "-c", "import numpy"]).returncode == 0:
            interpreter = candidate
            break
    has_numpy = run([interpreter, "-I", "-B", "-c", "import numpy"]).returncode == 0
    print(f"numpy available to the interpreter used: {has_numpy}")
    r = run([interpreter, "-I", "-B", str(FOLDER / "checks/preflight_view.py"), str(REPO / PREFLIGHT)], cwd=mirror)
    print(r.stdout.rstrip())
    if r.stderr.strip():
        print(r.stderr.strip()[:600])
    check("no FAIL beyond the listed expected ones", r.returncode == 0, f"exit {r.returncode}")
    check("narration_director's validate_map ran, not a WARN", not has_numpy or "CON.director_not_run" not in r.stdout, "needs numpy")
    r = run([interpreter, "-I", "-B", str(FOLDER / "checks/preflight_view.py"), str(REPO / PREFLIGHT), "--derived-requests"], cwd=mirror)
    print(r.stdout.rstrip())
    if r.stderr.strip():
        print(r.stderr.strip()[:600])
    check("control: with 01_5's request swapped in the mirror, all six request FAILs go away", r.returncode == 0, f"exit {r.returncode}")

    # ------------------------------------------------------------------ K
    say("K. The crate's own cue checker, on a scratch copy")
    copy = scratch / "cues"
    for rel in ("script.tsv", "docs/check_cues.py"):
        (copy / rel).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(Path(C7) / rel, copy / rel)
    shutil.copytree(Path(C7) / "src", copy / "src")
    r = run([sys.executable, "-I", "-B", "docs/check_cues.py"], cwd=copy)
    check("docs/check_cues.py passes", r.returncode == 0 and r.stdout.strip().splitlines()[-1] == "OK", r.stderr.strip()[:200])
    path = copy / "src/chapters/building_ruler.rs"
    data = path.read_text(encoding="utf-8")
    path.write_text(data.replace('kit.say(stage, "01_5")?;', ""), encoding="utf-8")
    r = run([sys.executable, "-I", "-B", "docs/check_cues.py"], cwd=copy)
    check("control: it refuses a removed say", r.returncode != 0)

    # ------------------------------------------------------------------ Z
    say("Z. Nothing it read has changed")
    for p, h in before.items():
        check(f"unchanged: {p}", tree(p) == h)

    failed = [n for n, good in results if not good]
    print(f"\n{len(results) - len(failed)} of {len(results)} passed. Source and plan text only: nothing compiled, rendered, synthesized, measured or heard.")
    if failed:
        print("FAILED:", failed)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
