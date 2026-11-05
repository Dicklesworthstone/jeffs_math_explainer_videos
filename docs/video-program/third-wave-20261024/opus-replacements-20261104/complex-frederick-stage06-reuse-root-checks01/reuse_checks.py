#!/usr/bin/env python3
"""Root replay of RubyRaven's stage06 regression against the pinned root integration (bead 87do.39).

    python3 -I -B P/complex-frederick-stage06-reuse-root-checks01/reuse_checks.py SCRATCH
        run from the repository root; SCRATCH must be a new or empty absolute folder outside the repository

The real stage05 side (crate, take folder, recognition lane, guard jobs) is only read; S1 proves it unchanged by size,
mtime and sha256. Mutation tests use a FIXTURE stage06 crate: the real stage05 crate with one stand-in sentence
added to each of the five actually changed cues, its units and map rebound. The root tools' real stage06 pins are
replaced in-process by fixture pins only for these mutation checks. Fixture WAVs are seeded noise. Actual stage06
report equality was checked separately in stage06-runtime-transfer01/root-checks01.json. Nothing is synthesized,
recognized, decoded, launched or heard here. This is a root replay, not independent review.
Exit 0 when every check passes, 1 otherwise.
"""
import array
import contextlib
import difflib
import hashlib
import importlib.util
import io
import json
import random
import shutil
import sys
import wave
from pathlib import Path

sys.dont_write_bytecode = True
REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = REPO / P / "complex-frederick-stage06-reuse-root01"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
TOPIC = "complex_analysis_visual"
CRATE5 = REPO / EC / "combined-repair-stage05/explainer_corpus" / TOPIC
TAKES5 = REPO / EC / "frederick-stage05-01"
LANE5 = REPO / EC / "frederick-stage05-recognition01"
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
SEALED = {
    "intake": (f"{P}/complex-frederick-intake-resolution01/frederick_intake.py",
               "8760be91d1df2207faddb4bacca173d7351c14ebdd50e86e4ac295eff7fe4d32"),
    "bridge": (f"{P}/complex-frederick-timing-bridge-repair01/frederick_timing_bridge.py",
               "47069583af82e2aa28688f1ed4179f3b40694450f2bf282acff3549aaf406a39"),
}
CHANGED = ("05_8", "10_3", "14_1", "14_3", "15_2")
STAND_IN = " This sentence is a stage06 fixture."
MAP_REL = "docs/narration-direction-map-oliver12.json"
results = []


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def check(label, ok, detail=""):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {label}" + (f": {detail}" if detail else ""))


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, REPO / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run(module, argv):
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        try:
            code = module.main(argv)
        except SystemExit as stop:
            code = stop.code
    return code, out.getvalue(), err.getvalue()


def clean(text):
    return str(text).replace(str(SCR.resolve()), "SCRATCH").replace(str(SCR), "SCRATCH").replace(str(REPO), "REPO")


def snapshot():
    """Size, mtime and sha256 of every file on the real stage05 side this bundle reads."""
    files = [p for root in (TAKES5, LANE5) for p in sorted(root.rglob("*")) if p.is_file()]
    files += [CRATE5 / "script.tsv", CRATE5 / "narration_units.json", CRATE5 / MAP_REL,
              *sorted((CRATE5 / "src/chapters").glob("*.rs"))]
    files += [REPO / path for path, _ in SEALED.values()]
    return {str(p.relative_to(REPO)): (p.stat().st_size, p.stat().st_mtime_ns, sha(p)) for p in files}


def fixture_crate(root, extra_edit=None):
    """The real stage05 crate with the stand-in edits, its units and map rebound; returns (crate, pins, script)."""
    crate = root / "explainer_corpus" / TOPIC
    (crate / "docs").mkdir(parents=True)
    shutil.copytree(CRATE5 / "src/chapters", crate / "src/chapters")
    rows, script = [], {}
    for line in (CRATE5 / "script.tsv").read_text(encoding="utf-8").splitlines(keepends=True):
        cells = line.rstrip("\n").split("\t")
        if len(cells) == 2 and (cells[0] in CHANGED or cells[0] == extra_edit):
            line = f"{cells[0]}\t{cells[1]}{STAND_IN}\n"
        rows.append(line)
        cells = line.rstrip("\n").split("\t")
        if len(cells) == 2 and not line.startswith("#"):
            script[cells[0]] = cells[1]
    (crate / "script.tsv").write_text("".join(rows), encoding="utf-8")
    script_sha = sha(crate / "script.tsv")
    units = json.loads((CRATE5 / "narration_units.json").read_text(encoding="utf-8"))
    units["script_sha256"] = units["direction_contract"]["script_sha256"] = script_sha
    for unit in units["units"]:
        unit["text"] = script[unit["line_ids"][0]]
    direction = json.loads((CRATE5 / MAP_REL).read_text(encoding="utf-8"))
    direction["scripts"][TOPIC] = script_sha
    # The fixture appended a sentence: keep a complete-cue reading hold at the new cue end.
    # Leaving its old `after` text would intentionally make it an interior boundary instead.
    for override in direction["overrides"]:
        cue = override.get("cue")
        if cue in script and script[cue] == override.get("after", "") + STAND_IN:
            override["after"] = script[cue]
    (crate / MAP_REL).write_text(json.dumps(direction, indent=2) + "\n", encoding="utf-8")
    units["direction_contract"]["map"]["sha256"] = sha(crate / MAP_REL)
    (crate / "narration_units.json").write_text(json.dumps(units, indent=2) + "\n", encoding="utf-8")
    return crate, {"script": script_sha, "units": sha(crate / "narration_units.json"), "map": sha(crate / MAP_REL)}, script


def noise(seed):
    """One second of seeded noise as a 48 kHz mono 16-bit WAV, in memory."""
    draw = random.Random(seed)
    samples = array.array("h", (draw.randint(-3000, 3000) for _ in range(48000)))
    buffer = io.BytesIO()
    with wave.open(buffer, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(48000)
        w.writeframes(samples.tobytes())
    return buffer.getvalue()


def take_folder(folder, cue, text, script_sha, wavs):
    """A stage06-style request, submission (layout A) and downloads receipt for one cue; wavs are (name, bytes)."""
    request = {"cue": cue, "text": text, "text_sha256": hashlib.sha256(text.encode()).hexdigest(),
               "script_sha256": script_sha, "voice": i6.VOICE, **i6.SETTINGS, "submitted": False}
    (folder / f"{cue}-request01.json").write_text(json.dumps(request), encoding="utf-8")
    (folder / f"{cue}-submission01.json").write_text(json.dumps(
        {"cue": cue, "request": f"{cue}-request01.json", "submitted": True, "generation_requests": 1}), encoding="utf-8")
    takes = []
    for number, (name, data) in enumerate(wavs, 1):
        (folder / name).write_bytes(data)
        info = i6.probe_wav(data)
        takes.append({"path": str((folder / name).resolve()), "sha256": hashlib.sha256(data).hexdigest(),
                      "source": f"ElevenLabs_fixture_{i6.SOURCE_TAG} ({number}).wav", "pcm_sha256": info["pcm_sha256"],
                      "frames": info["frames"], "seconds": info["seconds"]})
    (folder / f"{cue}-downloads01.json").write_text(json.dumps(
        {"cue": cue, "request": f"{cue}-request01.json", "takes": takes}), encoding="utf-8")


def edited(source, target, change):
    data = json.loads(Path(source).read_text(encoding="utf-8"))
    change(data)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(data, indent=1) + "\n", encoding="utf-8")
    return target


def refused_by(call, words):
    try:
        call()
    except Exception as error:  # the intake's Refused or PinMoved
        return words in str(error), clean(str(error))[:160]
    return False, "not refused"


# ----------------------------------------------------------------------------- setup

if len(sys.argv) != 2:
    sys.exit(__doc__)
SCR = Path(sys.argv[1])
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
if not (REPO / "tools/coherent_narration.py").is_file():
    sys.exit("run from the repository root")
SCR.mkdir(parents=True, exist_ok=True)
before = snapshot()
for role, (path, pin) in SEALED.items():
    check(f"P0 sealed stage05 {role} unchanged", sha(REPO / path) == pin, f"{path.rsplit('/', 2)[-2]} {pin[:8]}")
i6 = load(f"{P}/complex-frederick-stage06-reuse-root01/frederick_intake_stage06.py", "intake_stage06_under_check")
b6 = load(f"{P}/complex-frederick-stage06-reuse-root01/frederick_timing_bridge_stage06.py", "bridge_stage06_under_check")
b5 = load(SEALED["bridge"][0], "bridge_stage05_sealed")

# ----------------------------------------------------------------------------- P1: wrong pins refuse

real_pins = i6.PINS
i6.PINS = {key: "0" * 64 for key in real_pins}
code, _, err = run(i6, ["status", "--lane", str(SCR / "lane-unset"), "--transfer", str(SCR / "none.json")])
check("P1a stage06 intake with planted wrong pins exits 3", code == 3, clean(err.strip())[:120])
i6.PINS = real_pins
real_dependency = b6.DEPENDENCIES["intake"]
b6.DEPENDENCIES["intake"] = (real_dependency[0], "0" * 64)
code, _, err = run(b6, ["report", "--out", str(SCR / "report-unset")])
check("P1b stage06 bridge with planted wrong intake pin exits 3", code == 3, clean(err.strip())[:120])
b6.DEPENDENCIES["intake"] = real_dependency

# ----------------------------------------------------------------------------- T: the transfer on the real stage05 side

crate6, pins6, script6 = fixture_crate(SCR / "fixture-stage06")
i6.PINS = pins6
i6.MAP_REL = MAP_REL  # Fixture preserves the old map filename; actual candidate uses Frederick stage06.
script5 = i6.read_script(CRATE5 / "script.tsv")
code, out, err = run(i6, ["transfer", "--crate", str(crate6), "--out", str(SCR / "t1"), "--by", "fixture check",
                          "--basis", "reuse checks on the real stage05 side"])
check("T1 transfer from the real stage05 side exits 0", code == 0, clean(err.strip())[:200])
t1 = SCR / "t1/transfer.json"
data = json.loads(t1.read_text(encoding="utf-8"))
gone = data["not_transferred"]
cues = {c["cue"]: c for c in data["cues"]}
takes = [t for c in data["cues"] for t in c["takes"]]
check("T2 55 cues and 110 takes transfer, none refused",
      (len(cues), len(takes), gone["takes_refused"]) == (55, 110, {}), f"{len(cues)} cues, {len(takes)} takes")
check("T3 every transferred cue has the same text in both crates and two takes",
      all(script5[c] == script6[c] and len(row["takes"]) == 2 for c, row in cues.items()))
check("T4 the five edited cues are the changed ones, have no takes, and need stage06 requests",
      [c["cue"] for c in gone["changed_text"]] == list(CHANGED) and not set(CHANGED) & set(cues)
      and set(CHANGED) <= set(gone["need_stage06_requests"]), str([c["cue"] for c in gone["changed_text"]]))
check("T5 the other 50 cues need stage06 requests; 105 ids in the same order",
      len(gone["need_stage06_requests"]) == 50 and data["same_cue_ids_and_order"]
      and set(gone["need_stage06_requests"]) | set(cues) == set(script6))
plans = json.loads((LANE5 / "plan-10/plan.json").read_text(encoding="utf-8"))
held51 = next(t["receipts"] for t in plans["takes"] if t["cue"] == "05_1")
check("T6 05_1 rests on root's resolution exactly as plan-10 froze it",
      cues["05_1"]["receipts"] == held51 and "resolution" in held51, ", ".join(sorted(held51)))
check("T7 every transferred take is the processed take of its plan, at its plan hash",
      all(any(t["sha256"] == p["sha256"] and t["name"] == p["name"]
              for p in json.loads((LANE5 / t["plan"] / "plan.json").read_text(encoding="utf-8"))["takes"])
          for t in takes))
check("T8 the stage05 scan's open item is reported: 09_5 submitted, not downloaded",
      gone["stage05_scan"]["awaiting_download"] == ["09_5"] and gone["stage05_scan"]["refused"] == [],
      str(gone["stage05_scan"]["awaiting_download"]))
check("T9 no absolute path in the transfer", str(REPO) not in t1.read_text(encoding="utf-8") and "/Users/" not in
      t1.read_text(encoding="utf-8"))

# ----------------------------------------------------------------------------- V: verifying a transfer later

ok = i6.check_transfer(t1, script6)
check("V1 check_transfer accepts the transfer as written", len(ok["cues"]) == 55 and len(ok["audio"]) == 220)
plants = {
    "V2 a take hash that is not today's": (lambda d: d["cues"][0]["takes"][0].update(sha256="0" * 64), "changed, moved"),
    "V3 a receipt hash that is not today's": (lambda d: d["cues"][3]["receipts"]["request"].update(sha256="0" * 64),
                                             "changed or is missing"),
    "V4 no basis": (lambda d: d.update(basis=" "), "by and basis"),
    "V5 other stage06 pins": (lambda d: d["stage06"]["crate_pins"].update(script="0" * 64), "other stage06 pins"),
    "V6 a lane file hash that is not today's": (lambda d: d["stage05"]["lane_files"].update({"plan-01/plan.json": "0"}),
                                               "stage05 lane"),
    "V7 another stage05 intake": (lambda d: d["stage05"]["intake"].update(sha256="0" * 64), "sealed stage05 intake"),
}
for label, (change, words) in plants.items():
    planted = edited(t1, SCR / "plants" / (label.split()[0] + ".json"), change)
    hit, why = refused_by(lambda: i6.check_transfer(planted, script6), words)
    check(f"{label} is refused", hit, why)

crate6b, pins6b, script6b = fixture_crate(SCR / "fixture-stage06-01_1-edited", extra_edit="01_1")
i6.PINS = pins6b
hit, why = refused_by(lambda: i6.check_transfer(t1, script6b), "other stage06 pins")
check("V8 a crate whose saved cue 01_1 changed refuses the earlier transfer", hit, why)
code, _, err = run(i6, ["transfer", "--crate", str(crate6b), "--out", str(SCR / "t-01_1"), "--by", "fixture check",
                        "--basis", "01_1 edited"])
tb = json.loads((SCR / "t-01_1/transfer.json").read_text(encoding="utf-8"))
check("V9 a new transfer then leaves 01_1 out: its stage06 row is not the stage05 text",
      code == 0 and len(tb["cues"]) == 54 and "01_1" not in {c["cue"] for c in tb["cues"]}
      and any("not the stage05 text" in r for r in tb["not_transferred"]["takes_refused"].get("01_1", [])),
      f"{len(tb['cues'])} cues; 01_1: {tb['not_transferred']['takes_refused'].get('01_1', ['?'])[0][:60]}")
i6.PINS = pins6

lane_copy = SCR / "stage05-lane-copy"
shutil.copytree(LANE5, lane_copy)
code, _, _ = run(i6, ["transfer", "--crate", str(crate6), "--out", str(SCR / "t-copy"), "--by", "fixture check",
                      "--basis", "lane copy", "--stage05-lane", str(lane_copy)])
copy_transfer = SCR / "t-copy/transfer.json"
(lane_copy / "plan-12").mkdir()
shutil.copyfile(LANE5 / "plan-11/plan.json", lane_copy / "plan-12/plan.json")
hit, why = refused_by(lambda: i6.check_transfer(copy_transfer, script6), "gained, lost or changed")
check("V10 a stage05 plan added after the transfer refuses it", code == 0 and hit, why)
hit, why = refused_by(lambda: i6.derive_transfer(script6, i6.stage05_intake(), CRATE5, TAKES5, lane_copy,
                                                 [REPO / f for f in i6.stage05_intake().OTHER_VOICE]), "open plans")
check("V11 no transfer while a stage05 plan is open", hit, why)

# ----------------------------------------------------------------------------- I: the stage06 intake

takes6 = SCR / "frederick-stage06-fixture-takes"
takes6.mkdir()
real01 = (TAKES5 / "01_1-download01.wav").read_bytes()
SCRIPT5_SHA = "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3"
take_folder(takes6, "05_8", script6["05_8"], pins6["script"],  # a remaining (edited) cue: eligible
            [("05_8-download01.wav", noise(1)), ("05_8-download02.wav", noise(2))])
take_folder(takes6, "01_1", script6["01_1"], pins6["script"],  # a transferred cue: refused
            [("01_1-download01.wav", noise(3)), ("01_1-download02.wav", noise(4))])
take_folder(takes6, "05_9", script6["05_9"], pins6["script"],  # a byte copy of transferred audio: refused
            [("05_9-download01.wav", real01), ("05_9-download02.wav", noise(5))])
take_folder(takes6, "06_8", script6["06_8"], SCRIPT5_SHA,  # still bound to the stage05 script: refused
            [("06_8-download01.wav", noise(6)), ("06_8-download02.wav", noise(7))])
code, out, err = run(i6, ["status", "--lane", str(SCR / "lane06"), "--takes", str(takes6), "--crate", str(crate6),
                          "--transfer", str(t1)])
status = json.loads(out) if code == 0 else {}
reasons = {r["cue"]: " ".join(r["reasons"]) for r in status.get("refused", [])}
check("I1 status: only the remaining cue 05_8 is eligible",
      code == 0 and status["eligible_wavs"] == ["05_8 05_8-download01.wav", "05_8 05_8-download02.wav"],
      clean(err.strip())[:160] or str(status.get("eligible_wavs")))
check("I2 a stage06 request for transferred 01_1 is refused", "has transferred stage05 takes" in reasons.get("01_1", ""),
      reasons.get("01_1", "")[:90])
check("I3 05_9 audio equal to transferred 01_1-download01 is a duplicate",
      "same audio as transferred stage05 01_1 01_1-download01.wav" in reasons.get("05_9", ""), reasons.get("05_9", "")[:90])
check("I4 a remaining cue still bound to the stage05 script is refused as drift",
      "script drift" in reasons.get("06_8", ""), reasons.get("06_8", "")[:90])
check("I5 status wrote nothing", not (SCR / "lane06").exists())
code, out, err = run(i6, ["plan", "--lane", str(SCR / "lane06"), "--takes", str(takes6), "--crate", str(crate6),
                          "--transfer", str(t1)])
plan6 = json.loads((SCR / "lane06/plan-01/plan.json").read_text(encoding="utf-8")) if code == 0 else {}
frozen = json.loads((SCR / "lane06/plan-01/frozen-inputs.json").read_text(encoding="utf-8"))["files"] if code == 0 else []
check("I6 plan-01 holds the two 05_8 takes and binds the transfer, frozen at its hash",
      code == 0 and [t["name"] for t in plan6["takes"]] == ["05_8-download01.wav", "05_8-download02.wav"]
      and plan6["binding"]["transfer"]["sha256"] == sha(t1) and plan6["binding"]["script_sha256"] == pins6["script"]
      and any(f["role"] == "transfer" and f["sha256"] == sha(t1) for f in frozen), clean(err.strip())[:160])
code, out, err = run(i6, ["status", "--lane", str(SCR / "lane06-on-stage05-folder"), "--crate", str(crate6),
                          "--takes", str(TAKES5), "--transfer", str(t1)])
status = json.loads(out) if code == 0 else {}
transferred = [x for x in status.get("excluded_wavs", []) if x.endswith("transferred from stage05")]
check("I7 on the real stage05 take folder: 110 WAVs reported transferred, nothing eligible or refused",
      code == 0 and len(transferred) == 110 and not status["eligible_wavs"] and not status["refused"],
      clean(err.strip())[:160] or f"{len(transferred)} transferred, {len(status.get('refused', []))} refused")
code, _, err = run(i6, ["status", "--lane", str(LANE5), "--crate", str(crate6), "--transfer", str(t1)])
check("I8 the sealed stage05 lane is refused as a stage06 lane", code == 2 and "sealed stage05 lane" in err,
      clean(err.strip())[:100])

# ----------------------------------------------------------------------------- B: the stage06 bridge

b6._loaded["intake"] = i6  # the fixture-pinned intake stands in for the unset DEPENDENCIES pin
b6.CHAPTER_SOURCES_SHA256 = hashlib.sha256("".join(
    f"{p.name} {sha(p)}\n" for p in sorted((crate6 / "src/chapters").glob("*.rs"))).encode()).hexdigest()
r6 = b6.build_report(crate6, None, WORK, TAKES5, transfer=t1)
r5 = b5.build_report(CRATE5, LANE5, WORK, TAKES5)
info = r6["coverage"]["recognition"]["transfer"]
check("B1 the stage06 bridge reads the 110 transferred takes through the transfer, all current",
      len(r6["takes"]) == 110 and all(e["text_current"] and e["transfer"]["sha256"] == sha(t1)
                                      for e in r6["takes"].values()) and info["stage05_takes_not_transferred"] == [],
      f"{len(r6['takes'])} takes")
same = [k for k, e in r6["takes"].items() if {x: y for x, y in e.items() if x != "transfer"} == r5["takes"][k]]
check("B2 every transferred take's report entry equals the sealed stage05 bridge's, field for field",
      len(same) == 110 == len(r5["takes"]), f"{len(same)}/110 equal")
q5 = sorted(q["id"] for q in r5["questions"])
q6 = sorted(q["id"] for q in r6["questions"])
check("B3 the same question ids, so root's answers carry over by id (re-signed for the new source side)",
      q5 == q6 and r6["side"]["sha256"] != r5["side"]["sha256"], f"{len(q6)} questions")
i6.PINS = pins6b  # the chapter sources are byte copies in both fixtures, so the chapter pin holds
r6b = b6.build_report(crate6b, None, WORK, TAKES5, transfer=SCR / "t-01_1/transfer.json")
left = r6b["coverage"]["recognition"]["transfer"]["stage05_takes_not_transferred"]
check("B4 with 01_1 edited, its two stage05 takes stay out of the stage06 report",
      len(r6b["takes"]) == 108 and left == ["01_1 01_1-download01.wav", "01_1 01_1-download02.wav"], str(left))
i6.PINS = pins6

# ----------------------------------------------------------------------------- S: nothing real was written

after = snapshot()
check("S1 the real stage05 crate, take folder, lane and sealed tools are unchanged (size, mtime, sha256)",
      before == after, f"{len(after)} files")
patch = SCR / "patch"
patch.mkdir()
for role, new in (("intake", "frederick_intake_stage06.py"), ("bridge", "frederick_timing_bridge_stage06.py")):
    old = SEALED[role][0]
    diff = "".join(difflib.unified_diff(
        (REPO / old).read_text(encoding="utf-8").splitlines(keepends=True),
        (HERE / new).read_text(encoding="utf-8").splitlines(keepends=True),
        fromfile=f"a/{old}", tofile=f"b/{P}/complex-frederick-stage06-reuse01/{new}"))
    (patch / f"{role}.diff").write_text(diff, encoding="utf-8")
    plus = sum(1 for line in diff.splitlines() if line.startswith("+") and not line.startswith("+++"))
    minus = sum(1 for line in diff.splitlines() if line.startswith("-") and not line.startswith("---"))
    print(f"NOTE {role}: {new} is {SEALED[role][1][:8]} plus {plus}/-{minus} lines (SCRATCH/patch/{role}.diff)")
print(f"\n{sum(results)}/{len(results)} PASS")
sys.exit(0 if all(results) else 1)
