#!/usr/bin/env python3
"""Independent checks of root's stage06 reuse integration (BlueSalmon Mails 4620, 4622, 4629, 4631; bead 87do.39).

    python3 -I -B P/complex-frederick-stage06-reuse-root-review01/checks/root_reuse_review.py SCRATCH
        run from the repository root; SCRATCH must be a new or empty absolute folder outside the repository

Reviewed: P/complex-frederick-stage06-reuse-root01 (root's pinned copies of RubyRaven's unfinished stage06 intake and
bridge), P/complex-frederick-stage06-reuse-root-checks01 (root's regression replay), EC/stage06-runtime-transfer01
(transfer.json, report01/02, root-checks01.json) and EC/stage06-narration-transfer01 (retained-takes, request drafts).
Also read: the stage05 and stage06 crates, the stage05 take folder and lane, the guard jobs they name, and the real
stage06 take folder (status only, through the pinned intake, with its lane in SCRATCH).

Reads only, apart from SCRATCH: the bridge report and the regression replay write there. No transfer file, plan,
ingest, job, synthesis, recognition, decode or assembly is made. Nothing is heard. Exit 0 when every check passes.
"""
import ast
import contextlib
import hashlib
import importlib.util
import io
import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
ROOT01, CHECKS01, MINE = REPO / P / "complex-frederick-stage06-reuse-root01", \
    REPO / P / "complex-frederick-stage06-reuse-root-checks01", REPO / P / "complex-frederick-stage06-reuse01"
CAND = REPO / EC / "combined-repair-stage06-root01"
CRATE6, CRATE5 = CAND / "explainer_corpus/complex_analysis_visual", \
    REPO / EC / "combined-repair-stage05/explainer_corpus/complex_analysis_visual"
TAKES5, LANE5, TAKES6 = REPO / EC / "frederick-stage05-01", REPO / EC / "frederick-stage05-recognition01", \
    REPO / EC / "frederick-stage06-01"
RUNTIME, EVIDENCE = REPO / EC / "stage06-runtime-transfer01", REPO / EC / "stage06-narration-transfer01"
TRANSFER = RUNTIME / "transfer.json"
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
SEALED_BRIDGE5 = (f"{P}/complex-frederick-timing-bridge-repair01/frederick_timing_bridge.py",
                  "47069583af82e2aa28688f1ed4179f3b40694450f2bf282acff3549aaf406a39")
NAMED = {  # hashes root named in mail, or that root01's files bind
    "candidate SHA256SUMS (Mail 4620)": (CAND / "SHA256SUMS", "44cb2a3fc1aadc44fb8599a88b794368915c5fd64f588d34d81bd570be497001"),
    "stage06 script (Mail 4620)": (CRATE6 / "script.tsv", "0bae7acb15f2f0453c8a25ba7110fedd65cf9156eba932143e08db7fdc8bfeee"),
    "transfer.json (HANDOFF)": (TRANSFER, "60880cbee72f88375264717908d6c13e866d13a4636d213bdf4971ec69fb3f49"),
    "retained-takes.json (Mail 4622)": (EVIDENCE / "retained-takes.json",
                                        "c1a93026de8eb8fc4704ae5eb7dc681aef80a8847088b5156c5e6a53cb1bb608"),
    "remaining-request-drafts.json (Mail 4622)": (EVIDENCE / "remaining-request-drafts.json",
                                                  "a967aeceb53b9312a5bb45839ca4968372467ad09b6253e429763b56837ef301"),
    "root regression script (result01.json)": (CHECKS01 / "reuse_checks.py",
                                               "6da7f02ba78add962e5fd303df0f0b60c4dd1cfcce842eda7a0b10c2bb71af3b"),
    "sealed stage05 bridge": (REPO / SEALED_BRIDGE5[0], SEALED_BRIDGE5[1]),
}
SCRIPT5_SHA = "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3"
CHANGED = ["05_8", "10_3", "14_1", "14_3", "15_2"]
results = []


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def check(label, ok, detail=""):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {label}" + (f"\n     {detail}" if detail else ""))


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
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
    return str(text).replace(str(SCR.resolve()), "SCRATCH").replace(str(SCR), "SCRATCH").replace(str(REPO) + "/", "")


def top_level(path):
    """Top-level statements by name, as AST dumps (comments and formatting ignored)."""
    named = {}
    for node in ast.parse(path.read_text(encoding="utf-8")).body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            name = node.name
        elif isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name):
            name = node.targets[0].id
        elif isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant):
            name = "<docstring>"
        else:
            name = ast.dump(node)
        named.setdefault(name, []).append(ast.dump(node))
    return named


def differing(a, b):
    x, y = top_level(a), top_level(b)
    return sorted(n for n in set(x) | set(y) if x.get(n) != y.get(n))


def snapshot():
    roots = [ROOT01, CHECKS01, MINE, CAND, TAKES5, LANE5, TAKES6, RUNTIME, EVIDENCE]
    files = [p for root in roots for p in root.rglob("*") if p.is_file()]
    files += [p for p in CRATE5.rglob("*") if p.is_file()] + [REPO / SEALED_BRIDGE5[0]]
    for ingest in LANE5.glob("ingest-[0-9][0-9]/ingest.json"):
        files += [p for p in (WORK / json.loads(ingest.read_text())["job"]).glob("*") if p.is_file()]
    return {str(p.relative_to(REPO)): (p.stat().st_size, p.stat().st_mtime_ns, sha(p)) for p in sorted(set(files))}


if len(sys.argv) != 2:
    sys.exit(__doc__)
SCR = Path(sys.argv[1])
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
if not (REPO / "tools/coherent_narration.py").is_file():
    sys.exit("run from the repository root")
SCR.mkdir(parents=True, exist_ok=True)
before = snapshot()

# ----------------------------------------------------------------------------- R0: named hashes and seals

for label, (path, pin) in NAMED.items():
    check(f"R0 {label} is {pin[:8]}", sha(path) == pin)
for folder in (ROOT01, CAND):
    rows = [line.split("  ", 1) for line in (folder / "SHA256SUMS").read_text().splitlines() if line]
    listed = {name for _, name in rows} | {"SHA256SUMS"}
    extra = sorted(str(p.relative_to(folder)) for p in folder.rglob("*") if p.is_file() and
                   str(p.relative_to(folder)) not in listed)
    check(f"R0 {folder.name}: all {len(rows)} SHA256SUMS rows verify", all(sha(folder / n) == d for d, n in rows),
          f"SHA256SUMS {sha(folder / 'SHA256SUMS')[:8]}; files present but not listed: {extra or 'none'}")
result = json.loads((CHECKS01 / "result01.json").read_text())
check("R0 root-checks01 result01.json binds the regression script it ran and preserves the failed first copy",
      result["test_script_sha256"] == sha(CHECKS01 / "reuse_checks.py") and result["exit"] == 0
      and (result["checks"], result["passed"]) == (37, 37)
      and (CHECKS01 / "before-fixture-hold-rebind01-reuse_checks.py").is_file())

# ----------------------------------------------------------------------------- R1: root's edits are narrow

edits = {"intake": differing(MINE / "frederick_intake_stage06.py", ROOT01 / "frederick_intake_stage06.py"),
         "bridge (pins)": differing(MINE / "frederick_timing_bridge_stage06.py",
                                    ROOT01 / "before-report-summary01-frederick_timing_bridge_stage06.py"),
         "bridge (print fix)": differing(ROOT01 / "before-report-summary01-frederick_timing_bridge_stage06.py",
                                         ROOT01 / "frederick_timing_bridge_stage06.py")}
check("R1 against my unsealed staging, root changed only the stage06 pins, the map name and the report print",
      edits == {"intake": ["CRATE", "MAP_REL", "PINS", "cmd_plan"],
                "bridge (pins)": ["CHAPTER_SOURCES_SHA256", "DEPENDENCIES"], "bridge (print fix)": ["cmd_report"]},
      "; ".join(f"{k}: {', '.join(v)}" for k, v in edits.items()))

# ----------------------------------------------------------------------------- R2: the pins at today's bytes

i6 = load(ROOT01 / "frederick_intake_stage06.py", "root01_intake_under_review")
b6 = load(ROOT01 / "frederick_timing_bridge_stage06.py", "root01_bridge_under_review")
b5 = load(REPO / SEALED_BRIDGE5[0], "sealed_stage05_bridge")
script6, files6 = i6.check_binding(CRATE6)
units = json.loads((CRATE6 / "narration_units.json").read_text())
check("R2 the stage06 intake's pins are the candidate's script, units and map; check_binding accepts the crate",
      all(sha(files6[k]) == i6.PINS[k] for k in ("script", "units", "map")) and len(script6) == 105
      and units["direction_contract"]["map"]["path"].endswith("/" + i6.MAP_REL),
      f"script {i6.PINS['script'][:8]}, units {i6.PINS['units'][:8]}, map {i6.PINS['map'][:8]} ({i6.MAP_REL})")
chapters = sorted((CRATE6 / "src/chapters").glob("*.rs"))
digest = hashlib.sha256("".join(f"{p.name} {sha(p)}\n" for p in chapters).encode()).hexdigest()
side, _script, _files = b6.source_side(CRATE6)
deps_ok = all(sha(REPO / path) == pin for path, pin in b6.DEPENDENCIES.values())
same = all(b6.DEPENDENCIES[k] == b5.DEPENDENCIES[k] for k in b6.DEPENDENCIES if k != "intake")
check("R2 the bridge pins the root01 intake at its bytes, the 25 stage06 chapter sources, and the sealed stage05 "
      "bridge's other dependencies unchanged",
      b6.DEPENDENCIES["intake"] == (str((ROOT01 / "frederick_intake_stage06.py").relative_to(REPO)),
                                    sha(ROOT01 / "frederick_intake_stage06.py"))
      and len(chapters) == 25 and digest == b6.CHAPTER_SOURCES_SHA256 and deps_ok and same,
      f"intake {b6.DEPENDENCIES['intake'][1][:8]}, chapters {digest[:8]}, other dependencies "
      f"{', '.join(k for k in b6.DEPENDENCIES if k != 'intake')}; source side {side['sha256'][:8]}")

# ----------------------------------------------------------------------------- R3: the script delta

script5 = i6.read_script(CRATE5 / "script.tsv")
changed = [c for c in script5 if script5[c] != script6.get(c)]
voiced = sorted({p.name.split("-")[0] for p in TAKES5.glob("*-download[0-9][0-9].wav")})
check("R3 105 cue ids in the same order; exactly 05_8, 10_3, 14_1, 14_3 and 15_2 changed; none of them was ever "
      "downloaded; all 55 downloaded cues keep their text",
      list(script5) == list(script6) and changed == CHANGED and not set(changed) & set(voiced) and len(voiced) == 55
      and all(script5[c] == script6[c] for c in voiced), f"changed {changed}; downloaded cues {len(voiced)}")

# ----------------------------------------------------------------------------- R4: the transfer

moved = i6.check_transfer(TRANSFER, script6)
data = moved["data"]
s5 = i6.stage05_intake()
body = i6.derive_transfer(script6, s5, CRATE5, TAKES5, LANE5, [REPO / f for f in s5.OTHER_VOICE])  # in memory only
keys = ("stage05", "stage06", "same_cue_ids_and_order", "cues", "not_transferred")
check("R4 check_transfer accepts root's transfer today, and re-deriving it in memory from the sealed stage05 side "
      "gives the same body field for field",
      len(moved["cues"]) == 55 and len(moved["audio"]) == 220 and all(body[k] == data[k] for k in keys)
      and data["tool"]["sha256"] == sha(ROOT01 / "frederick_intake_stage06.py"),
      f"{len(data['cues'])} cues, {sum(len(c['takes']) for c in data['cues'])} takes; written by "
      f"{data['tool']['sha256'][:8]}; by {data['by']}; listening {data['listening']}")
walk = []
for c in data["cues"]:
    cue = c["cue"]
    req_path = TAKES5 / c["receipts"]["request"]["name"]
    req = json.loads(req_path.read_text())
    listing = json.loads((TAKES5 / c["receipts"]["downloads"]["name"]).read_text())
    listed = {Path(t["path"]).name: t for t in listing["takes"]}
    ok = (sha(req_path) == c["receipts"]["request"]["sha256"]
          and (req["cue"], req["text"], req["voice"], req["script_sha256"]) == (cue, script6[cue], i6.VOICE, SCRIPT5_SHA)
          and sha(TAKES5 / c["receipts"]["downloads"]["name"]) == c["receipts"]["downloads"]["sha256"]
          and sorted(listed) == sorted(t["name"] for t in c["takes"]))
    for t in c["takes"]:
        plan = json.loads((LANE5 / t["plan"] / "plan.json").read_text())
        ingest = json.loads((LANE5 / t["ingest"] / "ingest.json").read_text())
        ok &= (sha(REPO / t["path"]) == t["sha256"] == listed[t["name"]]["sha256"]
               and t["pcm_sha256"] == listed[t["name"]]["pcm_sha256"]
               and any((r["cue"], r["name"], r["sha256"]) == (cue, t["name"], t["sha256"]) for r in plan["takes"])
               and ingest["plan"] == t["plan"] and t["sha256"] in {p["sha256"] for p in ingest["processed"]})
    if not ok:
        walk.append(cue)
check("R4 walked without the tools: each transferred request still binds the stage05 script (12442480) and names "
      "the cue, the unchanged text and Frederick; each WAV is the receipt's, its plan's and its ingest's",
      not walk, f"mismatches: {walk or 'none'}")

# ----------------------------------------------------------------------------- R5/R6: the report and its print

r6 = b6.build_report(CRATE6, None, WORK, REPO / i6.TAKES, transfer=TRANSFER)
bytes_equal = sorted(name for name, text in r6["out"].items()
                     if (RUNTIME / "report02" / name).read_bytes() == (text if isinstance(text, bytes) else text.encode()))
r1_r2 = all((RUNTIME / "report01" / n).read_bytes() == (RUNTIME / "report02" / n).read_bytes() for n in r6["out"])
check("R5 my in-process report on the real inputs is byte-equal to root's report02 (and report01 equals report02)",
      bytes_equal == sorted(r6["out"]) and len(bytes_equal) == 5 and r1_r2, ", ".join(bytes_equal))
r5 = b5.build_report(CRATE5, LANE5, WORK, TAKES5)
equal = [k for k, e in r6["takes"].items() if {x: y for x, y in e.items() if x != "transfer"} == r5["takes"].get(k)]
ids5, ids6 = sorted(q["id"] for q in r5["questions"]), sorted(q["id"] for q in r6["questions"])
same_q = sum(1 for a, b in zip(sorted(r5["questions"], key=lambda q: q["id"]), sorted(r6["questions"], key=lambda q: q["id"]))
             if a == b)
check("R5 against the sealed stage05 bridge: 110 take entries equal except the transfer field; the same 29 question "
      "ids", len(equal) == 110 == len(r6["takes"]) == len(r5["takes"]) and ids5 == ids6 and len(ids6) == 29,
      f"{len(equal)}/110 entries; {len(ids6)} ids, {same_q} question objects identical")
code, out, err = run(b6, ["report", "--out", str(SCR / "cli-report"), "--transfer", str(TRANSFER.relative_to(REPO))])
files_equal = all((SCR / "cli-report" / n).read_bytes() == (RUNTIME / "report02" / n).read_bytes() for n in r6["out"])
code2, out2, err2 = run(b6, ["report", "--out", str(SCR / "cli-report-no-lane")])
check("R6 the fixed print: the report CLI exits 0 with the transfer (both lane states) and without any lane",
      code == 0 and "recognition stage06=no recognition lane, stage05=read:" in out and files_equal
      and code2 == 0 and "recognition stage06=no recognition lane: 0 takes" in out2,
      clean(out.strip())[:150] + "\n     " + clean(out2.strip())[:110] + (clean(err + err2)[:200]))

# ----------------------------------------------------------------------------- R7: the real stage06 take folder

code, out, err = run(i6, ["status", "--lane", str(SCR / "status-lane"), "--transfer", str(TRANSFER)])
status = json.loads(out) if code == 0 else {}
need = data["not_transferred"]["need_stage06_requests"]
by_cue = {}
for name in status.get("eligible_wavs", []):
    by_cue.setdefault(name.split()[0], []).append(name.split()[1])
refused = [(r["cue"], r["receipt"], r["reasons"]) for r in status.get("refused", [])]
check("R7 the pinned intake on the real stage06 take folder (status only): all 50 remaining cues have two eligible "
      "takes each; the one refusal is 02_5, a transferred cue",
      code == 0 and sorted(by_cue) == sorted(need) and all(len(v) == 2 for v in by_cue.values())
      and not status["excluded_wavs"] and not status["unreceipted_wavs"] and not status["awaiting_download"]
      and refused == [("02_5", "02_5-downloads01.json", ["cue 02_5 has transferred stage05 takes; new stage06 requests "
                                                         "are for the remaining cues only"])],
      f"{status.get('downloads_receipts')} downloads receipts, {len(status.get('eligible_wavs', []))} eligible WAVs; "
      f"refused: {[r[:2] for r in refused]}" + clean(err)[:200])
drafts = {d["cue"]: d for d in json.loads((EVIDENCE / "remaining-request-drafts.json").read_text())["requests"]}
fields = ("cue", "text", "text_sha256", "script_sha256", "voice", "model", "stability", "similarity", "clone_strength",
          "output")
differ = []
for cue in need:
    listing = json.loads(next(TAKES6.glob(f"{cue}-downloads[0-9][0-9].json")).read_text())
    request = json.loads((TAKES6 / listing["request"]).read_text())
    if [request.get(k) for k in fields] != [drafts[cue].get(k) for k in fields]:
        differ.append(cue)
check("R7 root's 50 request drafts are the remaining cues in script order, and each real stage06 request the intake "
      "accepted equals its draft in every provider field",
      list(drafts) == need and not differ, f"{len(drafts)} drafts; differing: {differ or 'none'}")
new25 = json.loads((TAKES6 / "02_5-downloads01.json").read_text())
old25 = next(c for c in data["cues"] if c["cue"] == "02_5")
names_new = sorted(Path(t["path"]).name for t in new25["takes"])
check("R7 the 02_5 replacement pair reuses the transferred pair's file names with other audio",
      names_new == sorted(t["name"] for t in old25["takes"])
      and not {t["sha256"] for t in new25["takes"]} & {t["sha256"] for t in old25["takes"]},
      f"stage06 {names_new}; the bridge keys takes by (cue, file) and refuses a cue with takes in both lanes")

# ----------------------------------------------------------------------------- R8: root's retained-take evidence

records = json.loads((EVIDENCE / "retained-takes.json").read_text())
ev = {r["cue"]: r for r in records["records"]}
consistent = sorted(ev) == sorted(moved["cues"]) and all(
    ev[c]["text"] == script6[c] and ev[c]["original_request"]["sha256"] == moved["cues"][c]["receipts"]["request"]["sha256"]
    and ev[c]["original_download_receipt"]["sha256"] == moved["cues"][c]["receipts"]["downloads"]["sha256"]
    and sorted(t["sha256"] for t in ev[c]["takes"]) == sorted(t["sha256"] for t in moved["cues"][c]["takes"])
    for c in ev)
check("R8 retained-takes.json names the same 55 cues, texts, request and downloads receipts and 110 WAVs as the "
      "transfer", consistent and records["new_script"]["sha256"] == i6.PINS["script"]
      and records["old_script"]["sha256"] == SCRIPT5_SHA)

# ----------------------------------------------------------------------------- R9: root's regression, replayed

replay = subprocess.run([sys.executable, "-I", "-B", str(CHECKS01 / "reuse_checks.py"), str(SCR / "root-regression")],
                        capture_output=True, timeout=1800)
log = replay.stdout + replay.stderr
check("R9 root's 37-check regression replays 37/37, and its output hashes to the log root recorded",
      replay.returncode == 0 and b"37/37 PASS" in log and hashlib.sha256(log).hexdigest() == result["log_sha256"],
      f"exit {replay.returncode}; output {hashlib.sha256(log).hexdigest()[:8]}, root's log {result['log_sha256'][:8]}")

after = snapshot()
check("S1 every real file read is unchanged (size, mtime, sha256)", before == after, f"{len(after)} files")
print(f"\n{sum(results)}/{len(results)} PASS")
sys.exit(0 if all(results) else 1)
