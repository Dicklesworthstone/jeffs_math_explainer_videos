"""Checks of complex-frederick-intake-resolution01, the intake candidate for root Mail 4190 (05_1 supersession).

From the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-intake-resolution01/checks/resolution_checks.py NEW_SCRATCH_DIR

The candidate is the sealed frederick_intake.py (edc154c6) plus one rule, check_resolution: when two or more
submission receipts bind a downloads receipt's request, a hash-bound CUE-submission-resolutionNN.json may select
one of them. Without a resolution the sealed one-submission rule and its message stand.

Writes only under NEW_SCRATCH_DIR, which must not exist. Fixture WAVs are the sealed self-test's seeded noise; fixture
receipts and resolutions are synthetic and say so. The real-data sections only read root's take folder and lane:
section 5 runs `status` (which writes nothing) on the real take folder, and section 6 copies the real 05_1 receipts
and takes into scratch, re-points the copied downloads receipt's two take paths at the copies (its only edit), and
adds a fixture resolution there. Root authors the real resolution; nothing here writes into narration_lab, runs fw
or the guard's launch path, synthesizes, builds or listens. The guard wrapper is imported only for check_inputs.

Exit 0 PASS, 1 FAIL, 2 usage, 3 a pinned input moved.
"""
import ast
from concurrent.futures import ThreadPoolExecutor
import difflib
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
BUNDLE = HERE.parent
CANDIDATE = BUNDLE / "frederick_intake.py"
CANDIDATE_SHA = "8760be91d1df2207faddb4bacca173d7351c14ebdd50e86e4ac295eff7fe4d32"
DIFF = BUNDLE / "patch/intake-resolution01.diff"
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
SEALED_DIR = f"{P}/complex-frederick-audio-preparation01"
ORIGINAL = f"{SEALED_DIR}/frederick_intake.py"
SELFTEST = f"{SEALED_DIR}/checks/intake_selftest.py"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
TAKES = f"{EC}/frederick-stage05-01"
LANE = f"{EC}/frederick-stage05-recognition01"
PINS = {
    f"{SEALED_DIR}/SHA256SUMS": "38a0cc638c39576c988dc5f37279fa77189cc09e4b80bbdd96c926c0c40497d5",
    ORIGINAL: "edc154c6c4f7277715981678465be0189e58d6f36b17822c23a608defd7a75d0",
    SELFTEST: "0fca8e1649dab182974b64e20f8248a90e2330ab4ae3a07b26e9fad7adc610f5",
}
REAL_051 = {  # root's retained 05_1 files, read only
    "05_1-request01.json": "cf1623eda56ddab6c227eb2e2c77f885bb0807fa8ea1b8aaa3f0afd70b90911d",
    "05_1-submission01.json": "77859fad33bcf30036d8af74daa95d709f8c90b7f838f30ebfd1cff47e9418ed",
    "05_1-submission02.json": "4c88d6d918d0e9ee2fda8dc914f8a99acc53e087e296fa3d8e3d31c9fc159dbf",
    "05_1-submission-observation01.json": "138bc84b1802281c567034864735d4359d3ccc64d20c370789db82bdc49d8106",
    "05_1-downloads01.json": "87191b79effc8c012c292477428b8af8d3d992b2507555a253f105447e3f231e",
    "05_1-download01.wav": "a76aa56c0440c2d8e656c3d1315fe3e0700cad53243995b157b420e81b0bd8ad",
    "05_1-download02.wav": "cf33a5328e7402fb04906af512ef7a749f6740d6fa2a33e961875ad7bed863f6",
}
TODAY = "2 submission receipts bind '05_1-request01.json'; expected one"
T = {k: f"2026-11-05T00:0{k}:00+00:00" for k in range(1, 6)}  # fixture times: attempt, evidence, retry, downloads, resolution
SCRATCH = None
FAILS = []


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha_bytes(Path(path).read_bytes())


def say(text=""):
    text = str(text)
    for path, name in ((SCRATCH.resolve(), "SCRATCH"), (SCRATCH, "SCRATCH"), (REPO.resolve(), "REPO"), (REPO, "REPO")):
        text = text.replace(str(path), name)
    print(text, flush=True)


def check(name, ok, detail):
    if not ok:
        FAILS.append(name)
    say(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def dump(path, data):
    Path(path).write_text(json.dumps(data, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")


def listing(folder):
    return {p.name: sha_file(p) for p in sorted(Path(folder).iterdir()) if p.is_file()}


def intake(tool, *args, lane, takes):
    r = subprocess.run([sys.executable, "-I", "-B", str(tool), *map(str, args), "--lane", str(lane), "--takes",
                        str(takes)], cwd=REPO, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr


def refused_of(out, cue="05_1"):
    return [r for row in json.loads(out)["refused"] if row["cue"] == cue for r in row["reasons"]]


# ----------------------------------------------------------------------------- fixture worlds

ST = None  # the sealed self-test, copied unchanged into this bundle; its TOOL is the candidate


def at(fx, name):
    return {"name": name, "sha256": sha_bytes((fx.takes / name).read_bytes())}


def shape(fx, observation=True):
    """The 05_1 shape, synthetic: request01; submission01, a timed-out attempt in root's hash layout
    (submitted_once true); its observation; submission02, a retry that records no submitted state; and the retry's
    downloads receipt with its two takes."""
    fx.add("05_1", (51, 52), layout="hash")
    t = fx.takes
    ST.edit(t / "05_1-submission01.json", lambda s: s.update(at=T[1]))
    if observation:
        ST.dump(t / "05_1-submission-observation01.json", {
            "at": T[2], "supersedes": "05_1-submission01.json completion implication",
            "state": "unconfirmed submission; do not count as generated", "listening": False,
            "synthetic_fixture": True})
    ST.dump(t / "05_1-submission02.json", {"at": T[3], "cue": "05_1", "request_sha256": at(fx, "05_1-request01.json")[
        "sha256"], "attempt": 2, "basis": "synthetic retry", "listening": False})
    ST.edit(t / "05_1-downloads01.json", lambda d: d.update(at=T[4]))


def resolution(fx, name="05_1-submission-resolution01.json", **over):
    data = {"schema": ST.fi.RESOLUTION_SCHEMA, "cue": "05_1", "request": at(fx, "05_1-request01.json"),
            "downloads": at(fx, "05_1-downloads01.json"),
            "selected": dict(at(fx, "05_1-submission02.json"), submitted=True, generation_requests=1),
            "superseded": [dict(at(fx, "05_1-submission01.json"),
                                evidence=at(fx, "05_1-submission-observation01.json"))],
            "by": "fixture root", "basis": "synthetic test of the resolution rule", "at": T[5], "listening": False}
    data.update(over)
    ST.dump(fx.takes / name, data)
    return data


def world(name, resolved=True, observation=True):
    fx = ST.Fixture(SCRATCH / "t" / name)
    fx.add("01_1", (11, 12), layout="name")
    fx.add("01_2", (21, 22), layout="hash")
    shape(fx, observation)
    if resolved:
        resolution(fx)
    return fx


def case(name, plant, needle, resolved=True):
    """A planted defect: 05_1 is refused with the needle, and the other two cues are still planned."""
    fx = world(name, resolved)
    plant(fx)
    code, out, err = fx.run("plan")
    reasons = refused_of(out) if code == 0 else [err.strip()]
    hit = next((r for r in reasons if needle in r), None)
    planned = sorted({n[:4] for n in fx.inputs("plan-01")})
    check(name, code == 0 and hit and planned == ["01_1", "01_2"] and len(fx.inputs("plan-01")) == 4,
          f"05_1 refused: {hit or reasons}; planned {planned}")


def rewrite(fx, change):
    """Re-author the fixture resolution after a planted change, so its hashes are current."""
    data = resolution(fx)
    change(data)
    ST.dump(fx.takes / "05_1-submission-resolution01.json", data)


# ----------------------------------------------------------------------------- sections

def section_pins():
    say("== 1. Pins")
    moved = [p for p, h in PINS.items() if not (REPO / p).is_file() or sha_file(REPO / p) != h]
    moved += [f"{TAKES}/{n}" for n, h in REAL_051.items() if sha_file(REPO / TAKES / n) != h]
    for path, pin in ((CANDIDATE, CANDIDATE_SHA), (HERE / "intake_selftest.py", PINS[SELFTEST])):
        if not path.is_file() or sha_file(path) != pin:
            moved.append(str(path.relative_to(BUNDLE)))
    if not DIFF.is_file():
        moved.append(str(DIFF.relative_to(BUNDLE)))
    resolutions = [n for n in sorted(p.name for p in (REPO / TAKES).iterdir()) if "resolution" in n]
    if resolutions:  # root has authored the real one: sections 5 and 6 assume the folder before it
        moved.append(f"{TAKES} now holds {', '.join(resolutions)}")
    if moved:
        say("PIN MOVED: " + ", ".join(moved))
        sys.exit(3)
    sums = (REPO / SEALED_DIR / "SHA256SUMS").read_text().splitlines()
    sealed = all(sha_file(REPO / SEALED_DIR / x.split("  ", 1)[1]) == x.split("  ", 1)[0] for x in sums)
    check("p1-pins", sealed,
          f"the sealed intake {PINS[ORIGINAL][:8]}, its self-test {PINS[SELFTEST][:8]} (copied unchanged here) and "
          f"its seal 38a0cc63 ({len(sums)} files) verify; the candidate is {CANDIDATE_SHA[:8]}; root's 7 retained 05_1 "
          f"files are at their pins, and the take folder holds no resolution file")


def section_diff():
    say("== 2. The candidate as a diff of the sealed intake")
    tree = SCRATCH / "tree"
    (tree / ORIGINAL).parent.mkdir(parents=True)
    shutil.copyfile(REPO / ORIGINAL, tree / ORIGINAL)
    subprocess.run(["git", "init", "-q"], cwd=tree, check=True)
    applied = subprocess.run(["git", "apply", str(DIFF)], cwd=tree, capture_output=True)
    in_place = subprocess.run(["git", "apply", "--check", str(DIFF)], cwd=REPO, capture_output=True)
    lines = DIFF.read_text().splitlines()
    plus = sum(x.startswith("+") and not x.startswith("+++") for x in lines)
    minus = sum(x.startswith("-") and not x.startswith("---") for x in lines)
    check("d1-diff-applies", applied.returncode == 0 and sha_file(tree / ORIGINAL) == CANDIDATE_SHA
          and in_place.returncode == 0,
          f"intake-resolution01.diff (+{plus} -{minus}) turns a copy of the sealed intake into the candidate "
          f"{CANDIDATE_SHA[:8]}, and `git apply --check` passes on the sealed file in place")

    def parts(source):
        tree_ = ast.parse(source)
        named = {n.name: ast.get_source_segment(source, n) for n in tree_.body
                 if isinstance(n, (ast.FunctionDef, ast.ClassDef))}
        consts = {t.id: ast.dump(n.value) for n in tree_.body if isinstance(n, ast.Assign) for t in n.targets
                  if isinstance(t, ast.Name)}
        return named, consts
    (o_named, o_consts), (c_named, c_consts) = parts((REPO / ORIGINAL).read_text()), parts(CANDIDATE.read_text())
    changed = sorted(n for n in set(o_named) | set(c_named) if o_named.get(n) != c_named.get(n))
    new_consts = sorted(set(c_consts) - set(o_consts))
    moved_consts = sorted(n for n in o_consts if c_consts.get(n) != o_consts[n])
    delta = list(difflib.ndiff(o_named["check_receipt"].splitlines(), c_named["check_receipt"].splitlines()))
    removed = [x[2:] for x in delta if x.startswith("- ")]
    added = [x[2:] for x in delta if x.startswith("+ ")]
    check("d2-scope", changed == ["check_receipt", "check_resolution", "instant"] and new_consts ==
          ["RESOLUTION_KEYS", "RESOLUTION_SCHEMA"] and moved_consts == ["NAMES"]
          and removed == ["    if len(submissions) != 1:"] and added[-1] == "    elif len(submissions) != 1:",
          f"only {changed} differ; check_receipt keeps every sealed line except `if len(submissions) != 1:`, which "
          f"becomes `elif` after the {len(added) - 1} added lines that call check_resolution; NAMES gains the "
          f"resolution and observation patterns; {new_consts} are new; every other function and constant, including "
          f"the frozen-inputs writer, is the sealed code")


def section_selftest():
    say("== 3. The sealed intake's 54 checks, on the sealed intake (o) and on the candidate (c)")
    (SCRATCH / "bundle-o/checks").mkdir(parents=True)
    shutil.copyfile(REPO / ORIGINAL, SCRATCH / "bundle-o/frederick_intake.py")
    shutil.copyfile(REPO / SELFTEST, SCRATCH / "bundle-o/checks/intake_selftest.py")
    paths = {"o": SCRATCH / "bundle-o/checks/intake_selftest.py", "c": HERE / "intake_selftest.py"}

    def go(t):
        return subprocess.run([sys.executable, "-I", "-B", str(paths[t]), str(SCRATCH / f"st-{t}")], cwd=REPO,
                              capture_output=True, text=True)
    with ThreadPoolExecutor(2) as pool:
        out = dict(zip("oc", pool.map(go, "oc")))
    o, c = (out[t].stdout.replace(str(SCRATCH / f"st-{t}"), "ST") for t in "oc")
    check("s1-sealed-selftest-both", all(out[t].returncode == 0 and "SELFTEST: PASS (54/54)" in out[t].stdout
                                         for t in "oc") and o == c,
          "both pass 54/54 with identical transcripts: no-resolution behaviour, messages, plans, frozen inputs and "
          "ingests are unchanged")


def section_tests():
    say("== 4. The resolution contract on synthetic fixtures (REVIEW T1-T13, plus T14 for the time order)")
    fx = world("t01-two-binders-no-resolution", resolved=False, observation=False)
    code, out, err = fx.run("plan")
    lane_o = fx.root / "orig" / "lane"
    lane_o.parent.mkdir()
    code_o, out_o, _ = intake(REPO / ORIGINAL, "plan", lane=lane_o, takes=fx.takes)
    check("t01-two-binders-no-resolution", code == 0 and refused_of(out) == [TODAY] and code_o == 0
          and refused_of(out_o) == [TODAY] and len(fx.inputs("plan-01")) == 4,
          f"ambiguous success is refused with the sealed message, exactly as the sealed intake refuses it: "
          f"{refused_of(out)}")

    fx = world("t02-valid-resolution")
    before = listing(fx.takes)
    code, out, err = fx.run("plan")
    plan = fx.plan_json("plan-01") if code == 0 else {"takes": []}
    rows = [t for t in plan["takes"] if t["cue"] == "05_1"]
    kinds = {k: v["name"] for k, v in rows[0]["receipts"].items()} if rows else {}
    frozen = read(fx.lane / "plan-01/frozen-inputs.json")["files"] if code == 0 else []
    held = {(Path(f["path"]).name, f["sha256"]) for f in frozen if f["role"] == "receipt"}
    every = all((v["name"], v["sha256"]) in held and Path(v["name"]).name == v["name"]
                for t in plan["takes"] for v in t["receipts"].values())
    guard = load("guard_wrapper", REPO / ST.fi.GUARD).load().check_inputs(fx.lane / "plan-01/frozen-inputs.json") \
        if code == 0 else None
    check("t02-valid-resolution", code == 0 and sorted(fx.inputs("plan-01")) == sorted(
        ["01_1-download01.wav", "01_1-download02.wav", "01_2-download01.wav", "01_2-download02.wav",
         "05_1-download01.wav", "05_1-download02.wav"]) and not refused_of(out) and kinds == {
        "downloads": "05_1-downloads01.json", "request": "05_1-request01.json",
        "resolution": "05_1-submission-resolution01.json", "submission": "05_1-submission02.json",
        "superseded01": "05_1-submission01.json", "superseded01_evidence": "05_1-submission-observation01.json"}
          and every and guard == sha_bytes((fx.lane / "plan-01/frozen-inputs.json").read_bytes())
          and listing(fx.takes) == before,
          f"05_1's two takes are planned; each rests on {sorted(kinds)}, so only submission01 is set aside, as "
          f"superseded01 with its evidence; every receipt of every take is a bare name frozen at its plan hash "
          f"(the bridge candidate's rule), root's guard check_inputs accepts the manifest, and the take folder is "
          f"unchanged ({len(before)} files)")

    def stated(key, field=None):
        def plant(fx):
            data = read(fx.takes / "05_1-submission-resolution01.json")
            target = data[key][0] if key == "superseded" else data[key]
            (target[field] if field else target)["sha256"] = "0" * 64
            ST.dump(fx.takes / "05_1-submission-resolution01.json", data)
        return plant
    case("t03a-request-hash", stated("request"), "names 05_1-request01.json at 000000000000")
    case("t03b-downloads-hash", stated("downloads"), "names the downloads receipt 05_1-downloads01.json at 000000000000")
    case("t03c-selected-hash", stated("selected"), "05_1-submission02.json is ")
    case("t03d-superseded-hash", stated("superseded"), "05_1-submission01.json is ")
    case("t03e-evidence-hash", stated("superseded", "evidence"),
         "the evidence for 05_1-submission01.json must be a 05_1-submission-observationNN.json")
    case("t03f-evidence-edited-after", lambda fx: ST.edit(fx.takes / "05_1-submission-observation01.json",
                                                         lambda o: o.update(state="edited after the resolution")),
         "the evidence for 05_1-submission01.json must be a 05_1-submission-observationNN.json")
    case("t04-unnamed-third-binder", lambda fx: ST.dump(fx.takes / "05_1-submission03.json", {
        "at": T[3], "cue": "05_1", "request_sha256": at(fx, "05_1-request01.json")["sha256"], "attempt": 3}),
         "are ['05_1-submission01.json', '05_1-submission02.json', '05_1-submission03.json'], but it names")
    case("t05-two-resolutions", lambda fx: shutil.copyfile(fx.takes / "05_1-submission-resolution01.json",
                                                           fx.takes / "05_1-submission-resolution02.json"),
         "2 submission resolutions for 05_1")

    def one_binder(fx):
        ST.edit(fx.takes / "05_1-submission02.json", lambda s: s.update(request_sha256="f" * 64))
        resolution(fx)
    case("t06-only-one-binder", one_binder, "only 1 submission receipt binds 05_1-request01.json")
    case("t07a-selected-also-superseded", lambda fx: rewrite(fx, lambda d: d.update(superseded=[
        dict(at(fx, "05_1-submission02.json"), evidence=d["superseded"][0]["evidence"])])),
         "names one submission receipt twice")
    case("t07b-duplicate-superseded", lambda fx: rewrite(fx, lambda d: d.update(
        superseded=d["superseded"] * 2)), "names one submission receipt twice")
    case("t07c-other-cue-submission", lambda fx: rewrite(fx, lambda d: d.update(superseded=[
        dict(at(fx, "01_1-submission01.json"), evidence=d["superseded"][0]["evidence"])])),
         "the submission receipts binding 05_1-request01.json are")

    def other_request(fx):
        ST.dump(fx.takes / "05_1-submission03.json", {"at": T[3], "cue": "05_1", "request_sha256": "f" * 64})
        rewrite(fx, lambda d: d.update(superseded=[dict(at(fx, "05_1-submission03.json"),
                                                        evidence=d["superseded"][0]["evidence"])]))
    case("t07d-other-request-submission", other_request, "the submission receipts binding 05_1-request01.json are")
    case("t08a-missing-evidence", lambda fx: rewrite(fx, lambda d: d["superseded"][0].update(evidence={
        "name": "05_1-submission-observation02.json", "sha256": "0" * 64})), "'05_1-submission-observation02.json' is not")

    def evidence_says(text):
        def plant(fx):
            ST.edit(fx.takes / "05_1-submission-observation01.json", lambda o: o.update(supersedes=text))
            resolution(fx)
        return plant
    case("t08b-evidence-supersedes-another", evidence_says("05_1-submission02.json retry"),
         "supersedes '05_1-submission02.json retry', not 05_1-submission01.json")
    case("t08c-evidence-name-prefix", evidence_says("05_1-submission01.json.old"),
         "supersedes '05_1-submission01.json.old', not 05_1-submission01.json")
    case("t08d-evidence-not-an-observation", lambda fx: rewrite(fx, lambda d: d["superseded"][0].update(
        evidence=at(fx, "05_1-request01.json"))), "'05_1-request01.json' is not")
    case("t09a-state-not-submitted", lambda fx: rewrite(fx, lambda d: d["selected"].update(submitted=False)),
         "the selected state must be submitted true")
    case("t09b-state-count-bool", lambda fx: rewrite(fx, lambda d: d["selected"].update(generation_requests=True)),
         "the selected state must be submitted true with a positive integer generation_requests")

    def contrary(fx):
        ST.edit(fx.takes / "05_1-submission02.json", lambda s: s.update(submitted=False))
        resolution(fx)
    case("t09c-state-contradicts-receipt", contrary, "contradicts 05_1-submission02.json's own submitted")
    case("t09d-take-count", lambda fx: rewrite(fx, lambda d: d["selected"].update(generation_requests=2)),
         "incomplete take set: 2 takes for 2 generation request(s); expected 4")
    case("t10-another-downloads-receipt", lambda fx: rewrite(fx, lambda d: d["downloads"].update(
        name="05_1-downloads02.json")), "names the downloads receipt 05_1-downloads02.json")
    fx = world("t11-observation-alone", resolved=False)
    code, out, err = fx.run("plan")
    check("t11-observation-alone", code == 0 and refused_of(out) == [TODAY] and len(fx.inputs("plan-01")) == 4,
          f"an observation that supersedes submission01 changes nothing without a resolution: {refused_of(out)}")

    def others(lane):
        rows = [t for t in read(lane / "plan-01/plan.json")["takes"] if t["cue"] != "05_1"]
        frozen = [f for f in read(lane / "plan-01/frozen-inputs.json")["files"]
                  if f["role"] in ("receipt", "wav") and not Path(f["path"]).name.startswith("05_1-")]
        inputs = (lane / "plan-01/inputs.txt").read_text(encoding="utf-8").splitlines()
        return json.dumps([rows, frozen]), len(inputs)
    fx = world("t12-other-cues")  # one take folder, planned into two lanes: resolution file moved aside, then back
    aside = fx.root / "moved-aside-05_1-submission-resolution01.json"
    (fx.takes / "05_1-submission-resolution01.json").rename(aside)
    lanes = {k: fx.root / k / "lane" for k in ("without", "with")}
    for k in lanes:
        lanes[k].parent.mkdir()
    ra = intake(CANDIDATE, "plan", lane=lanes["without"], takes=fx.takes)
    aside.rename(fx.takes / "05_1-submission-resolution01.json")
    rb = intake(CANDIDATE, "plan", lane=lanes["with"], takes=fx.takes)
    (a, na), (b, nb) = others(lanes["without"]), others(lanes["with"])
    check("t12-other-cues-unchanged", ra[0] == rb[0] == 0 and a == b and (na, nb) == (4, 6),
          "on one take folder, 01_1 and 01_2 get byte-identical plan rows and frozen entries with and without 05_1's "
          "resolution file; only 05_1's two takes are added")

    def own_state(layout):
        def plant(fx):
            if layout == "name":
                ST.dump(fx.takes / "05_1-submission02.json", {"at": T[3], "cue": "05_1",
                                                              "request": "05_1-request01.json", "submitted": True,
                                                              "generation_requests": 1})
            else:
                ST.edit(fx.takes / "05_1-submission02.json", lambda s: s.update(submitted_once=True))
            resolution(fx)
        return plant
    ready = []
    for layout in ("name", "hash"):
        fx = world(f"t13-selected-own-state-{layout}")
        own_state(layout)(fx)
        code, out, err = fx.run("plan")
        ready.append(code == 0 and not refused_of(out) and len(fx.inputs("plan-01")) == 6)
    check("t13-selected-with-its-own-state", all(ready),
          "a retry receipt that states its own submission (by name with submitted and generation_requests, or by "
          "hash with submitted_once) and an agreeing resolution: 05_1 is planned in both layouts")
    case("t14a-resolution-before-downloads", lambda fx: rewrite(fx, lambda d: d.update(at=T[3])),
         "the resolution is earlier than 05_1-downloads01.json")
    case("t14b-evidence-before-attempt", lambda fx: (
        ST.edit(fx.takes / "05_1-submission-observation01.json", lambda o: o.update(at="2026-11-04T23:59:00+00:00")),
        resolution(fx)), "must come after 05_1-submission01.json and before the resolution")
    case("t14c-downloads-without-time", lambda fx: (ST.edit(fx.takes / "05_1-downloads01.json", lambda d: d.pop("at")),
                                                    resolution(fx)), "downloads has no ISO time with its zone")
    case("t14d-selected-after-downloads", lambda fx: (ST.edit(fx.takes / "05_1-submission02.json",
                                                              lambda s: s.update(at=T[5])), resolution(fx)),
         "05_1-submission02.json is later than 05_1-downloads01.json")
    case("t15-form", lambda fx: rewrite(fx, lambda d: d.update(note="an extra key")),
         "not a frederick-submission-resolution-v1 for 05_1 with exactly the keys")


def real_lane_copy(dest):
    """plan.json, ingest.json and release files of root's live lane: what lane_state reads."""
    names = []
    for path in sorted((REPO / LANE).iterdir()):
        files = ([path / "plan.json"] if path.name.startswith("plan-") else [path / "ingest.json"]
                 if path.name.startswith("ingest-") else [path] if path.name.startswith("release-") else [])
        for f in files:
            if f.is_file():
                target = dest / f.relative_to(REPO / LANE)
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(f, target)
                names.append(str(f.relative_to(REPO / LANE)))
    return names


def section_real_status():
    say("== 5. Real take folder, read only: `status` (writes nothing) with the sealed intake and the candidate")
    lane = SCRATCH / "real-status" / "lane"
    copied = real_lane_copy(lane)
    before = listing(REPO / TAKES)
    o = intake(REPO / ORIGINAL, "status", lane=lane, takes=REPO / TAKES)
    c = intake(CANDIDATE, "status", lane=lane, takes=REPO / TAKES)
    plans = sum(n.startswith("plan-") for n in copied)
    check("r1-real-status-unchanged", o[0] == c[0] == 0 and o[1] == c[1] and refused_of(c[1]) == [TODAY]
          and listing(REPO / TAKES) == before and plans >= 5,
          "on root's live take folder and a copy of the live lane's plans and closers (both read at run time, so "
          "their counts are not printed), the sealed intake and the candidate print the same status; 05_1 stays "
          "refused with the sealed message until a resolution exists; the take folder is unchanged")


def section_real_staging():
    say("== 6. Real 05_1 files, staged in scratch with a fixture resolution (root authors the real one)")
    stage = SCRATCH / "real-051"
    takes, lane = stage / "takes", stage / "lane"
    takes.mkdir(parents=True)
    for n in REAL_051:
        if n != "05_1-downloads01.json":
            shutil.copyfile(REPO / TAKES / n, takes / n)
    real = read(REPO / TAKES / "05_1-downloads01.json")
    staged = json.loads(json.dumps(real))
    for take in staged["takes"]:
        take["path"] = str(takes / Path(take["path"]).name)
    (takes / "05_1-downloads01.json").write_text(json.dumps(staged, indent=2, ensure_ascii=False) + "\n",
                                                 encoding="utf-8")
    same = all(sha_file(takes / n) == h for n, h in REAL_051.items() if n != "05_1-downloads01.json")
    only_paths = [{k: v for k, v in t.items() if k != "path"} for t in staged["takes"]] == [
        {k: v for k, v in t.items() if k != "path"} for t in real["takes"]] and {
        k: v for k, v in staged.items() if k != "takes"} == {k: v for k, v in real.items() if k != "takes"}
    o0 = intake(REPO / ORIGINAL, "plan", lane=lane, takes=takes)
    c0 = intake(CANDIDATE, "plan", lane=lane, takes=takes)
    fixture = {
        "schema": "frederick-submission-resolution-v1", "cue": "05_1",
        "request": {"name": "05_1-request01.json", "sha256": sha_file(takes / "05_1-request01.json")},
        "downloads": {"name": "05_1-downloads01.json", "sha256": sha_file(takes / "05_1-downloads01.json")},
        "selected": {"name": "05_1-submission02.json", "sha256": sha_file(takes / "05_1-submission02.json"),
                     "submitted": True, "generation_requests": 1},
        "superseded": [{"name": "05_1-submission01.json", "sha256": sha_file(takes / "05_1-submission01.json"),
                        "evidence": {"name": "05_1-submission-observation01.json",
                                     "sha256": sha_file(takes / "05_1-submission-observation01.json")}}],
        "by": "fixture (resolution_checks.py); not root's resolution",
        "basis": "staging test of the contract on copies of the real 05_1 receipts", "at": "2026-11-05T06:00:00+00:00",
        "listening": False}
    dump(takes / "05_1-submission-resolution01.json", fixture)
    o1 = intake(REPO / ORIGINAL, "plan", lane=lane, takes=takes)
    c1 = intake(CANDIDATE, "plan", lane=lane, takes=takes)
    plan = read(lane / "plan-01/plan.json") if (lane / "plan-01/plan.json").is_file() else {"takes": []}
    rows = plan["takes"]
    guard = load("guard_wrapper_real", REPO / ST.fi.GUARD).load().check_inputs(lane / "plan-01/frozen-inputs.json") \
        if rows else None
    check("g1-real-051-staged", same and only_paths and o0[0] == c0[0] == 0 and refused_of(o0[1]) == [TODAY]
          and refused_of(c0[1]) == [TODAY] and o1[0] == 0 and refused_of(o1[1]) == [TODAY] and c1[0] == 0
          and not refused_of(c1[1]) and [t["name"] for t in rows] == ["05_1-download01.wav", "05_1-download02.wav"]
          and all(t["receipts"]["submission"]["name"] == "05_1-submission02.json"
                  and t["receipts"]["superseded01"]["name"] == "05_1-submission01.json" for t in rows)
          and [t["sha256"] for t in rows] == [REAL_051["05_1-download01.wav"], REAL_051["05_1-download02.wav"]]
          and guard == sha_file(lane / "plan-01/frozen-inputs.json"),
          f"the copies keep the real hashes (the downloads copy differs only in its two take paths); without a "
          f"resolution both intakes refuse 05_1 ({TODAY}); with the fixture resolution the sealed intake still "
          f"refuses it, and the candidate plans both real takes ({rows[0]['sha256'][:8] if rows else '-'}, "
          f"{rows[1]['sha256'][:8] if len(rows) > 1 else '-'}) on submission02, with submission01 set aside as "
          f"superseded01; root's guard check_inputs accepts the manifest")
    check("g2-real-folder-untouched", all(sha_file(REPO / TAKES / n) == h for n, h in REAL_051.items())
          and not any("resolution" in p.name for p in (REPO / TAKES).iterdir()),
          "root's 7 retained 05_1 files are still at their pins and the take folder holds no resolution file")


def section_hygiene():
    if BUNDLE.name != "complex-frederick-intake-resolution01":
        say("== 7. Hygiene: skipped (not run from the bundle)")
        return
    say("== 7. Hygiene of this bundle")
    machine = re.compile(r"/Users/|/home/|/var/folders|jemanuel|scratchpad|claude-501|@gmail")  # hygiene-pattern
    pronoun = re.compile(r"\b(he|she|his|her|him|hers|himself|herself)\b", re.I)  # hygiene-pattern
    files = sorted(p for p in BUNDLE.rglob("*") if p.is_file() and p.name != "SHA256SUMS"
                   and "__pycache__" not in p.relative_to(BUNDLE).parts)
    names = [str(p.relative_to(BUNDLE)) for p in files]
    want = ["HANDOFF.md", "checks/intake_selftest.py", "checks/outputs/resolution_checks.txt",
            "checks/resolution_checks.py", "frederick_intake.py", "patch/intake-resolution01.diff"]
    dirty = [n for n, p in zip(names, files) if any(
        (machine.search(x) or pronoun.search(x)) and not x.endswith("# hygiene-pattern")
        for x in p.read_text(encoding="utf-8", errors="replace").splitlines())]
    check("h1-hygiene", names == want and not dirty,
          f"exactly the {len(want)} bundle files, with no machine path, user name, address or gendered pronoun"
          f"{(' ' + str(dirty)) if dirty else ''}{(' ' + str(names)) if names != want else ''}")


def main():
    global SCRATCH, ST
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    SCRATCH = Path(sys.argv[1]).absolute()
    if SCRATCH.exists() or not (REPO / "tools/coherent_narration.py").is_file():
        print("give a NEW scratch folder and run from the repository root")
        return 2
    SCRATCH.mkdir(parents=True)
    section_pins()
    ST = load("intake_selftest_copy", HERE / "intake_selftest.py")
    section_diff()
    section_selftest()
    section_tests()
    section_real_status()
    section_real_staging()
    section_hygiene()
    say(f"RESULT {'PASS' if not FAILS else 'FAIL'}, {len(FAILS)} failed{(': ' + ', '.join(FAILS)) if FAILS else ''}")
    return 0 if not FAILS else 1


if __name__ == "__main__":
    sys.exit(main())
