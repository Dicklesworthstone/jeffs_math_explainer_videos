"""Check the inspection narration plan with the repo's own readers, plus an
independent source scan and negative controls. Read-only; run from the
repo root (the director's evidence paths are repo-relative):

    python3 -I -B explainer_corpus/inspection_paradox/docs/narration-prep-20261024/check_plan.py [--script SCRIPT_TSV]

SCRIPT_TSV defaults to the projected-script.tsv beside this file. After root
integrates the 10/11 and 12 patches and regenerates script.tsv, pass the
live explainer_corpus/inspection_paradox/script.tsv: the plan binds it only
if the bytes are the same.

Checks: script binding; tools/coherent_narration.py read_units (schema,
exact text, at most 100 words, one chapter, coverage in order); the same
coverage counted independently; every seam inside a unit re-scanned in the
chapter source for a think, hold, wait or authored window between the two
says; tools/narration_director.py validate_map and directions (every phrase
names one boundary, no duplicate boundary, evidence hashes, adjacent hold
cues); each visual hold's evidence lines hold the call it names; every
anchor unique. Then negative controls, each of which must be refused.
Exit 1 on any failure. Not a compile, synthesis, timing or listening check.
"""
import argparse
import collections
import copy
import hashlib
import importlib.util
import json
import pathlib
import re
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
REPO = pathlib.Path.cwd()
VIDEO = "inspection_paradox"
parser = argparse.ArgumentParser()
parser.add_argument("--script", default=str(HERE / "projected-script.tsv"))
args = parser.parse_args()
script_path = pathlib.Path(args.script)


def rel(p):
    p = pathlib.Path(p).resolve()
    return str(p.relative_to(REPO.resolve())) if p.is_relative_to(REPO.resolve()) else p.name


sha = lambda p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def load_tool(name):
    spec = importlib.util.spec_from_file_location(name, REPO / "tools" / f"{name}.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


cn = load_tool("coherent_narration")
nd = load_tool("narration_director")
units_path, map_path, seams_path = HERE / "narration_units.json", HERE / "director-map.json", HERE / "seams.json"
plan = json.loads(units_path.read_text())
direction = json.loads(map_path.read_text())
seams = {r["seam"]: r for r in json.loads(seams_path.read_text())["seams"]}
failures, counted = [], 0


def ok(cond, what):
    global counted
    counted += 1
    if not cond:
        failures.append(what)
    return cond


print(f"script {rel(script_path)} sha256 {sha(script_path)}")
# A. binding
ok(plan["script_sha256"] == sha(script_path), "units do not bind this script")
ok(direction["scripts"].get(VIDEO) == sha(script_path), "map does not bind this script")
print(f"A. units and map bind this script: {plan['script_sha256'] == direction['scripts'].get(VIDEO) == sha(script_path)}")

# B. the repo's own unit reader
script, units = cn.read_units(script_path, units_path)
print(f"B. coherent_narration.read_units: {len(script)} cues, {len(units)} units")

# C. coverage counted independently
order = [c for u in plan["units"] for c in u["line_ids"]]
counts = collections.Counter(order)
ok(order == list(script), "unit order is not script order")
ok(all(counts[c] == 1 for c in script) and set(counts) == set(script), "a cue is missing or repeated")
for u in plan["units"]:
    ok(u["text"] == " ".join(script[c] for c in u["line_ids"]), f"{u['id']} text is not verbatim")
    ok(set(u) == {"id", "line_ids", "text"}, f"{u['id']} has keys beyond the schema")
words = [len(u["text"].split()) for u in plan["units"]]
per_chapter = collections.Counter(u["line_ids"][0][:2] for u in plan["units"])
print(f"C. every cue once, in order: {order == list(script) and max(counts.values()) == 1}; "
      f"{len(order)} cues; words per unit {min(words)}-{max(words)}; units per chapter "
      + " ".join(f"{k}:{v}" for k, v in sorted(per_chapter.items())))

# D. seams inside units, re-scanned in the source
SAY = 'say(&kit, stage, "{}")'
HOLD = re.compile(r"\bthink\(|\bkit\.hold\(|\bstand\(|\bwait_until\(|\bpause\(|\bkit\.(?:music_breath|listen|sonify)\(")
files = {}
for r in seams.values():
    files[r["seam"].split("|")[0][:2]] = r["file"]


def held_between(a, b):
    lines = (REPO / files[a[:2]]).read_text().splitlines()
    i = next(n for n, l in enumerate(lines) if SAY.format(a) in l)
    j = next(n for n, l in enumerate(lines) if SAY.format(b) in l)
    return [(n + 1, lines[n].strip()) for n in range(i + 1, j)
            if not lines[n].strip().startswith("//") and HOLD.search(lines[n])]


inside = [(a, b) for u in plan["units"] for a, b in zip(u["line_ids"], u["line_ids"][1:])]
for a, b in inside:
    found = held_between(a, b)
    ok(not found, f"{a}|{b} is inside a unit but the source holds there: {found}")
    s = seams.get(f"{a}|{b}")
    ok(s is not None and s["decision"] == "coherent_eligible" and s["slack_s"] >= 0.5,
       f"{a}|{b} is inside a unit but not a coherent-eligible seam")
ok(("10_3", "10_4") not in inside, "10_3|10_4 is inside a unit")
boundary_holds = [s for s, r in seams.items() if r["decision"] == "boundary" and "events" in r]
rescan = [s for s in boundary_holds if held_between(*s.split("|"))]
ok(len(rescan) == len(boundary_holds), "a source-proven hold seam does not re-scan as held")
print(f"D. {len(inside)} seams inside units: none holds in the source, all coherent-eligible (slack >= 0.5 s); "
      f"10_3|10_4 inside a unit: {('10_3', '10_4') in inside}; {len(rescan)}/{len(boundary_holds)} hold seams re-scan as held")

# E. the director's own validation and splicing
manifest = {"videos": [{"id": VIDEO, "script": str(script_path)}]}
nd.validate_map(manifest, direction)
rows = nd.directions(VIDEO, nd.qa.read_tsv(script_path), direction)
authored = {k: v for k, v in rows.items() if v["basis"] == "authored"}
ok(len(authored) == len(direction["overrides"]), "an override did not land on its own boundary")
toks = {c: nd.tokens(t) for c, t in script.items()}
cue_end = [(c, i) for (c, i) in authored if i == len(toks[c]) - 1]
ok(not cue_end, f"overrides at a cue end (those boundaries belong to the holds): {cue_end}")
roles = collections.Counter(v["role"] for v in authored.values())
print(f"E. narration_director.validate_map passes; directions places {len(authored)}/{len(direction['overrides'])} "
      f"overrides on distinct boundaries, none at a cue end; roles " + ", ".join(f"{k} {v}" for k, v in sorted(roles.items())))

# F. visual holds and their source lines
CALL = {"prediction": r"\bthink\(", "reading_hold": r"\bkit\.hold\(|\bwait_until\(|\bpause\(",
        "authored_window": r"\bkit\.music_breath\(", "deliberate_overrun": r"\.wait_for\(",
        "chapter_end": r"\bstand\(|\bkit\.hold\("}  # 10 and 11 end on kit.hold + wait_until, not stand
held = {(h["cue"], h["next_cue"]) for h in direction["visual_holds"]}
for h in direction["visual_holds"]:
    a, b = h["evidence"]["lines"]
    body = "\n".join((REPO / h["evidence"]["path"]).read_text().splitlines()[a - 1 : b])
    ok(re.search(CALL[h["kind"]], body) is not None, f"{h['cue']}|{h['next_cue']}: lines {a}-{b} lack its {h['kind']} call")
expected = {tuple(s.split("|")) for s, r in seams.items()
            if r["decision"] == "boundary" and r["kind"] in ("prediction", "reading_hold", "authored_window", "deliberate_overrun")}
chapter_ends = {(a, b) for a, b in zip(list(script), list(script)[1:]) if a[:2] != b[:2]}
ok(held == expected | chapter_ends, "visual holds are not exactly the source holds, deliberate overruns and chapter ends")
ok(not held & set(inside), "a visual hold sits inside a unit")
print(f"F. {len(held)} visual holds = {len(expected)} source holds/deliberate overruns + {len(chapter_ends)} chapter ends; "
      f"each evidence span holds its call; none inside a unit")

# G. anchors
bad = [s for s, r in seams.items() if r.get("anchor") and script[s.split("|")[0]].count(r["anchor"]) != 1]
ok(not bad, f"seam anchors not unique: {bad}")
print(f"G. override phrases unique (directions raises otherwise); {sum(1 for r in seams.values() if r.get('anchor'))} seam anchors unique: {not bad}")

# H. negative controls: each must be refused
scratch = tempfile.TemporaryDirectory()  # control variants live outside the repo
tmp = pathlib.Path(scratch.name) / "units.json"


def refused(label, fn):
    try:
        fn()
    except (ValueError, KeyError, StopIteration) as e:
        print(f"   refused as expected: {label}: {str(e)[:110]}")
        return True
    failures.append(f"control not refused: {label}")
    print(f"   NOT REFUSED: {label}")
    return False


def units_variant(mutate):
    p = copy.deepcopy(plan)
    mutate(p)
    return lambda: cn.read_units(script_path, write(p))


def write(obj):
    tmp.write_text(json.dumps(obj))
    return tmp


def drop(p):
    p["units"][0]["line_ids"].pop()
    p["units"][0]["text"] = " ".join(script[c] for c in p["units"][0]["line_ids"])


def dup(p):
    p["units"].insert(1, dict(p["units"][1], id="dup"))


def swap(p):
    p["units"][1], p["units"][2] = p["units"][2], p["units"][1]


def retext(p):
    p["units"][3]["text"] = p["units"][3]["text"][:-1] + "!"


def cross(p):
    i = next(k for k, u in enumerate(p["units"]) if u["line_ids"][-1] == "01_7")
    a, b = p["units"][i], p["units"].pop(i + 1)
    a["line_ids"] += b["line_ids"]
    a["text"] += " " + b["text"]


def long(p):
    i = next(k for k, u in enumerate(p["units"]) if u["id"] == "01u1")
    a = p["units"][i]
    for _ in range(2):
        b = p["units"].pop(i + 1)
        a["line_ids"] += b["line_ids"]
        a["text"] += " " + b["text"]


def rehash(p):
    p["script_sha256"] = "0" * 64


def merged_10_3_10_4():
    p = copy.deepcopy(plan)
    i = next(k for k, u in enumerate(p["units"]) if u["line_ids"] == ["10_3"])
    b = p["units"].pop(i + 1)
    p["units"][i]["line_ids"] += b["line_ids"]
    p["units"][i]["text"] += " " + b["text"]
    cn.read_units(script_path, write(p))  # the schema reader accepts it ...
    for u in p["units"]:
        for a, c in zip(u["line_ids"], u["line_ids"][1:]):
            if held_between(a, c) or seams[f"{a}|{c}"]["decision"] != "coherent_eligible":
                raise ValueError(f"{a}|{c} is a {seams[f'{a}|{c}']['kind']} seam")  # ... the seam check does not


def map_variant(mutate):
    d = copy.deepcopy(direction)
    mutate(d)
    return lambda: (nd.validate_map(manifest, d), nd.directions(VIDEO, nd.qa.read_tsv(script_path), d))


print("H. negative controls")
try:
    refused("units omit a cue", units_variant(drop))
    refused("units repeat a cue", units_variant(dup))
    refused("units reorder two units", units_variant(swap))
    refused("unit text changed by one character", units_variant(retext))
    refused("a unit crosses chapters 01|02", units_variant(cross))
    refused("a unit over 100 words", units_variant(long))
    refused("units bound to another script", units_variant(rehash))
    refused("10_3 and 10_4 merged (read_units alone accepts it; the source seam check refuses)", merged_10_3_10_4)
    refused("override phrase 'three' (twice in 12_4)",
            map_variant(lambda d: d["overrides"].append(dict(d["overrides"][-1], cue="12_4", after="three"))))
    refused("override phrase not in its cue",
            map_variant(lambda d: d["overrides"].append(dict(d["overrides"][-1], cue="12_4", after="half of four"))))
    refused("two overrides on one boundary", map_variant(lambda d: d["overrides"].append(dict(d["overrides"][-1]))))
    refused("visual hold evidence changed",
            map_variant(lambda d: d["visual_holds"][0]["evidence"].update(sha256="0" * 64)))
    refused("visual hold names non-adjacent cues", map_variant(lambda d: d["visual_holds"][0].update(next_cue="01_7")))
    refused("map bound to another script", map_variant(lambda d: d["scripts"].update({VIDEO: "0" * 64})))
finally:
    scratch.cleanup()
print(f"{counted} checks; {len(failures)} failure(s)")
for f in failures:
    print("  FAIL " + f)
sys.exit(1 if failures else 0)
