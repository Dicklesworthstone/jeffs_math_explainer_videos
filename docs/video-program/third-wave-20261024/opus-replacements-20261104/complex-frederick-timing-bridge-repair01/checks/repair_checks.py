"""Checks of complex-frederick-timing-bridge-repair01, the bridge candidate for root Mail 4190.

From the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-timing-bridge-repair01/checks/repair_checks.py NEW_SCRATCH_DIR

The candidate is the reviewed patch (complex-frederick-timing-independent01, fe5c3096) with one repair: the PCM
re-derivation no longer uses tempfile.TemporaryDirectory(), which removed its conversions on exit. Prepare now
converts into OUT/decode-verification/, a new folder created exclusively under its own --out, keeps every file and
records each hash in decode-verification.json, whose hash the prepare receipt carries; timings refuses if a kept
file is missing or changed.

Writes only under NEW_SCRATCH_DIR, which must not exist. Root's lane, take folder and job folders are only read:
plan-01..05 and ingest-01..05 are copied into scratch at pinned hashes for the report comparison. ffmpeg runs only on
the author's self-test noise fixtures. No production audio is decoded and nothing is heard.

Exit 0 PASS, 1 FAIL, 2 usage, 3 a pinned input moved.
"""
import ast
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import wave

sys.dont_write_bytecode = True
REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
BUNDLE = HERE.parent
CANDIDATE = BUNDLE / "frederick_timing_bridge.py"
CANDIDATE_SHA = "47069583af82e2aa28688f1ed4179f3b40694450f2bf282acff3549aaf406a39"
FULL_DIFF = BUNDLE / "patch/bridge-repair01.diff"
DELTA_DIFF = BUNDLE / "patch/delta-from-review01.diff"
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
BRIDGE_DIR = f"{P}/complex-frederick-timing-bridge01"
BRIDGE = f"{BRIDGE_DIR}/frederick_timing_bridge.py"
SELFTEST = f"{BRIDGE_DIR}/checks/bridge_selftest.py"
REVIEW_DIR = f"{P}/complex-frederick-timing-independent01"
REVIEW_PATCH = f"{REVIEW_DIR}/patch/bridge-fixes.diff"
REVIEWED_SHA = "7d689793daa9ad7520b6c456acba791a1b84fb8fb12e0529489679baacfa247b"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
LANE = f"{EC}/frederick-stage05-recognition01"
WORK = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
SUPP = (f"{WORK}/complex-frederick-targeted-small01", f"{EC}/frederick-targeted-small01/inputs.json")
PINS = {
    f"{BRIDGE_DIR}/SHA256SUMS": "f31fd5c5708fc19db33c6a16cf2e15302141a39277732290872f964d2acb9b8f",
    BRIDGE: "c820376e6bd1ce4cbb1ecee099563621b0aeea035a5ec77e6d83c3ad901ade5a",
    SELFTEST: "ab7f53ff83a4c9172b4949c7dce15741bcbb9b99df5743927636f07100220843",
    f"{REVIEW_DIR}/SHA256SUMS": "48e585685d981cb6054592523f49bfc7245d241fdb1727156cca4f2cdc914448",
    REVIEW_PATCH: "fe5c3096c76d690bd621df70845637ed8c554fcc7dfe446b9f679e6fff2a4902",
    "tools/coherent_narration.py": "95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5",
    "tools/narration_cache.py": "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0",
    f"{P}/complex-frederick-audio-preparation01/frederick_intake.py":
        "edc154c6c4f7277715981678465be0189e58d6f36b17822c23a608defd7a75d0",
}
LANE_PINS = {  # the lane as reviewed in complex-frederick-timing-independent01; later plans are additive, not read
    "plan-01/plan.json": "cb9ae156d97567bc2eb4cf871dcceb9c9eed0ac35df634aa9223233aacea3419",
    "plan-01/frozen-inputs.json": "54acd3096bc0c08cc54df0a7d24db5bda9a9b9e228593258de96673e09b7c473",
    "ingest-01/ingest.json": "b3f24232e71b6c3e6d6d03147aa46317e6d3ee7f8d305a734860cf6311b4a84a",
    "ingest-01/transcripts.json": "db2d0b13e716004b65e3bba2f16210cc3fcdf75cf3def6cb4433344530dae436",
    "plan-02/plan.json": "8d63900896b4b5737b903fb2cad2c37cfa070c7113d7f0d0d7d889066c91475b",
    "plan-02/frozen-inputs.json": "aaeba9acf529152342c5942b23eb55beae26a20ff326f1aba96d256cb0c29cb4",
    "ingest-02/ingest.json": "22b4ebb2e08c1bceb7a09be64246b4ba3747bec74c728629ebb529082213edee",
    "ingest-02/transcripts.json": "6637cf5de0dd6d9a61eb348c7d7c30175bc4ce276692d600fb87d2b15b8540b1",
    "plan-03/plan.json": "bb32e783855475fdcd1a93d55a9bf5a63d87314d7fad3adeab3f621969e7d20e",
    "plan-03/frozen-inputs.json": "327456005cb394af01f4cf35acb1c484124090509c9d3b073983eff01bada38b",
    "ingest-03/ingest.json": "9463a6825fe8e023b53c5747f78dabffa2fc7812d0ff2e2f1a67faa8f7e4d11d",
    "ingest-03/transcripts.json": "3b0ff70ffc8b6cadf7b2697f2641a1f221a93524236128671834160d5df1cf71",
    "plan-04/plan.json": "5382ffe6f8cc633899e42d5e09dd86f8cf0ccb593f27bbb28a5925e6c10408d2",
    "plan-04/frozen-inputs.json": "f5a8c19588e9f1af3b96bb7f8b5e54a26eab57b45e1f1cb8052f80a49f9b3a62",
    "ingest-04/ingest.json": "9eb272028a29b1cfb9b039e35132a550721aa3779d926747b3ba2c54964c8f36",
    "ingest-04/transcripts.json": "b1e0c9001daa7e920fa252c8c71a8704db9b39829b9c38f2dad3f478feb41a58",
    "plan-05/plan.json": "02d0b3443ffa01f4884bebafa191ade7f57f76bd76eee2c899920dae53ddac54",
    "plan-05/frozen-inputs.json": "c7e8fafc82795a094c6afaf689da122e7edf92444dcece26682a7db451180ec5",
    "ingest-05/ingest.json": "33a2d387ab4cf21bf573d21e5af8e8ab0aac307489fc9d7723328f4787f0d8af",
    "ingest-05/transcripts.json": "a1636c9c3b808c512d319de4bbf65201b63a30edaa54c5b003086a8d4fce59f1",
}
BY = {"by": "candidate check", "basis": "planted case in scratch; not a selection or answer of root's"}
DELETERS = re.compile(r"\b(tempfile|TemporaryDirectory|NamedTemporaryFile|mkdtemp|unlink|rmtree|rmdir|remove|"  # hygiene-pattern
                      r"removedirs)\b")  # hygiene-pattern
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


def run(argv, cwd=None, env=None):
    return subprocess.run([str(a) for a in argv], cwd=cwd or REPO, capture_output=True, text=True, env=env)


def tool(path, *args, env=None):
    return run([sys.executable, "-I", "-B", path, *args], env=env)


def last(text, n=1):
    lines = [x for x in str(text).strip().splitlines() if x.strip()]
    return " | ".join(lines[-n:]) if lines else "(nothing)"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def dump(path, data):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(json.dumps(data, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def pcm(path):
    with wave.open(str(path)) as w:
        return w.readframes(w.getnframes())


def listing(folder):
    return {str(p.relative_to(folder)): sha_file(p) for p in sorted(Path(folder).rglob("*")) if p.is_file()}


def defs(source):
    """Top-level functions and classes by name, as source text; the other top-level statements as ast dumps."""
    tree = ast.parse(source)
    named, rest, imports = {}, [], set()
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            named[node.name] = ast.get_source_segment(source, node)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            imports |= {a.name for a in node.names}
        elif not (isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant) and node is tree.body[0]):
            rest.append(ast.dump(node))
    return named, rest, imports


# ----------------------------------------------------------------------------- sections

def section_pins():
    say("== 1. Pins")
    moved = [p for p, h in PINS.items() if not (REPO / p).is_file() or sha_file(REPO / p) != h]
    moved += [f"{LANE}/{p}" for p, h in LANE_PINS.items()
              if not (REPO / LANE / p).is_file() or sha_file(REPO / LANE / p) != h]
    for path, pin in ((CANDIDATE, CANDIDATE_SHA), (HERE / "bridge_selftest.py", PINS[SELFTEST])):
        if not path.is_file() or sha_file(path) != pin:
            moved.append(str(path.relative_to(BUNDLE)))
    moved += [str(p.relative_to(BUNDLE)) for p in (FULL_DIFF, DELTA_DIFF) if not p.is_file()]
    if moved:
        say("PIN MOVED: " + ", ".join(moved))
        sys.exit(3)
    seals = []
    for folder, count in ((BRIDGE_DIR, 12), (REVIEW_DIR, 4)):
        sums = (REPO / folder / "SHA256SUMS").read_text().splitlines()
        seals.append(len(sums) == count and all(sha_file(REPO / folder / x.split("  ", 1)[1]) == x.split("  ", 1)[0]
                                                for x in sums))
    check("p1-pins", all(seals), f"{len(PINS)} sources plus {len(LANE_PINS)} lane files at their pins; the candidate is "
                                 f"{CANDIDATE_SHA[:8]} and its self-test copy is the author's {PINS[SELFTEST][:8]}; the "
                                 f"bridge seal f31fd5c5 and the review seal 48e58568 both verify")


def section_diffs():
    say("== 2. The candidate as diffs (apply them to copies, never to the sealed folder)")
    full, delta = SCRATCH / "tree-full", SCRATCH / "tree-delta"
    for tree in (full, delta):
        (tree / BRIDGE).parent.mkdir(parents=True)
        shutil.copyfile(REPO / BRIDGE, tree / BRIDGE)
        run(["git", "init", "-q"], cwd=tree)
    one = run(["git", "apply", FULL_DIFF], cwd=full)
    in_place = run(["git", "apply", "--check", FULL_DIFF])
    review = run(["git", "apply", REPO / REVIEW_PATCH], cwd=delta)
    reviewed = sha_file(delta / BRIDGE)
    two = run(["git", "apply", DELTA_DIFF], cwd=delta)
    counts = []
    for diff in (FULL_DIFF, DELTA_DIFF):
        lines = diff.read_text().splitlines()
        counts.append((sum(x.startswith("+") and not x.startswith("+++") for x in lines),
                       sum(x.startswith("-") and not x.startswith("---") for x in lines)))
    check("d1-diffs-apply", one.returncode == 0 and sha_file(full / BRIDGE) == CANDIDATE_SHA and in_place.returncode == 0
          and review.returncode == 0 and reviewed == REVIEWED_SHA and two.returncode == 0
          and sha_file(delta / BRIDGE) == CANDIDATE_SHA,
          f"bridge-repair01.diff (+{counts[0][0]} -{counts[0][1]}) turns the sealed bridge into the candidate "
          f"{CANDIDATE_SHA[:8]} and passes `git apply --check` on the sealed file in place; the review's patch gives "
          f"the reviewed {reviewed[:8]}, and delta-from-review01.diff (+{counts[1][0]} -{counts[1][1]}) turns that into "
          f"the candidate")
    reviewed_tree = SCRATCH / "tree-reviewed"  # tree-delta now holds the candidate; keep the reviewed file apart
    (reviewed_tree / BRIDGE).parent.mkdir(parents=True)
    shutil.copyfile(REPO / BRIDGE, reviewed_tree / BRIDGE)
    run(["git", "init", "-q"], cwd=reviewed_tree)
    run(["git", "apply", REPO / REVIEW_PATCH], cwd=reviewed_tree)
    sealed_src, reviewed_src, cand_src = ((REPO / BRIDGE).read_text(), (reviewed_tree / BRIDGE).read_text(),
                                          CANDIDATE.read_text())
    (s_named, _s_rest, _s_imp), (r_named, r_rest, r_imp), (c_named, c_rest, c_imp) = map(
        defs, (sealed_src, reviewed_src, cand_src))
    changed = sorted(n for n in set(r_named) | set(c_named) if r_named.get(n) != c_named.get(n))
    added = sorted(set(c_named) - set(r_named))
    check("d2-delta-scope", changed == ["check_decoded", "cmd_prepare", "cmd_timings", "retain_conversions"]
          and added == ["retain_conversions"] and c_named["check_decoded"] == s_named["check_decoded"]
          and r_imp - c_imp == {"tempfile"} and not c_imp - r_imp and r_rest == c_rest,
          f"against the reviewed file only {changed} differ ({added} is new); check_decoded is again byte-identical "
          f"to the sealed bridge's; the only import removed is tempfile; every other function, class and constant is "
          f"unchanged, so the review's F1, F2 and F4 code is the code reviewed")
    hits = sorted({m.group(1) for m in DELETERS.finditer(cand_src)})
    sealed_hits = sorted({m.group(1) for m in DELETERS.finditer(sealed_src)})
    reviewed_hits = sorted({m.group(1) for m in DELETERS.finditer(reviewed_src)})
    check("d3-nothing-removes-files", not hits and not sealed_hits and reviewed_hits == ["TemporaryDirectory", "tempfile"],
          f"the candidate names no temporary-folder or file-removal call ({hits or 'none'}); the sealed bridge names "
          f"none either; the reviewed patch named {reviewed_hits}")
    bundle_b = SCRATCH / "bundle-b"
    bundle_b.mkdir()
    shutil.copyfile(reviewed_tree / BRIDGE, bundle_b / "frederick_timing_bridge.py")
    return bundle_b / "frederick_timing_bridge.py"


def section_selftests():
    say("== 3. The author's 37 checks on the sealed bridge (a) and the candidate (c)")
    (SCRATCH / "bundle-a/checks").mkdir(parents=True)
    shutil.copyfile(REPO / BRIDGE, SCRATCH / "bundle-a/frederick_timing_bridge.py")
    shutil.copyfile(REPO / SELFTEST, SCRATCH / "bundle-a/checks/bridge_selftest.py")
    paths = {"a": SCRATCH / "bundle-a/checks/bridge_selftest.py", "c": HERE / "bridge_selftest.py"}
    with ThreadPoolExecutor(2) as pool:
        out = dict(zip("ac", pool.map(lambda t: tool(paths[t], SCRATCH / f"st-{t}"), "ac")))
    a, c = (out[t].stdout.replace(str(SCRATCH / f"st-{t}"), "ST") for t in "ac")
    kept = sorted(p.name for p in (SCRATCH / "st-c/prepared/decode-verification").iterdir())
    check("s1-selftest-both", all(out[t].returncode == 0 and "SELFTEST: PASS (37/37)" in out[t].stdout for t in "ac")
          and a == c and not (SCRATCH / "st-a/prepared/decode-verification").exists() and len(kept) == 6,
          f"both pass 37/37 with identical transcripts; in the candidate's world, prepare kept {len(kept)} conversions "
          f"({kept[0]}..{kept[-1]}) and the author's timings run accepted them")


def section_report(tools):
    say("== 4. The report on a frozen copy of root's lane (plans and ingests 01-05)")
    lane = SCRATCH / "lane"
    for rel_path in LANE_PINS:
        (lane / rel_path).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPO / LANE / rel_path, lane / rel_path)
    common = ["report", "--lane", lane, "--supplement", *SUPP]
    with ThreadPoolExecutor(2) as pool:
        res = list(pool.map(lambda t: tool(tools[t], *common, "--out", SCRATCH / f"report-{t}"), "ac"))
    names = ("source.json", "takes.json", "questions.json", "answers-template.json", "coverage.json")
    same = all(r.returncode == 0 for r in res) and all(
        (SCRATCH / "report-a" / n).read_bytes() == (SCRATCH / "report-c" / n).read_bytes() for n in names)
    c = read(SCRATCH / "report-a/coverage.json")["counts"]
    check("r1-report-unchanged", same and c["takes"] == 62,
          f"sealed and candidate exit 0 with byte-identical outputs ({c['takes']} takes of "
          f"{c['cues_with_current_takes']} cues, {c['questions']} questions)")


def world_args(st):
    w = st / "world"
    return ["--takes", w / "takes", "--lane", w / "lane", "--jobs", w / "work",
            "--supplement", w / "work/complex-fixture-targeted-small01", w / "targeted/inputs.json"]


def section_retention(tools):
    say("== 5. Retained conversions, in the candidate's self-test world")
    st, v = SCRATCH / "st-c", SCRATCH / "v"
    v.mkdir()
    req = st / "world/takes/01_2-request01.json"  # the self-test's n22 appends one space here and leaves it
    pin = next(x for x in read(st / "world/lane/plan-01/plan.json")["takes"]
               if x["name"] == "01_2-download01.wav")["receipts"]["request"]["sha256"]
    text = req.read_text(encoding="utf-8")
    if text.endswith(" ") and sha_bytes(text[:-1].encode("utf-8")) == pin:
        req.write_text(text[:-1], encoding="utf-8")
    check("w0-world-restored", sha_file(req) == pin, "n22's one-space edit of 01_2-request01.json removed; the world "
                                                     "again has the plan's request hash")
    base = ["prepare", *world_args(st), "--chapter", "01", "--selection", st / "selection.json",
            "--reviewer", "candidate check", "--answers", st / "answers.json"]
    cues = list(read(st / "selection.json")["takes"])
    rb = tool(tools["b"], *base, "--decoded", st / "decoded", "--out", v / "prepared-b")
    rc = tool(tools["c"], *base, "--decoded", st / "decoded", "--out", v / "prepared-c")
    pc = v / "prepared-c"
    ver = read(pc / "decode-verification.json") if rc.returncode == 0 else {"files": []}
    held = read(pc / "prepare-receipt.json").get("decode_verification", {}) if rc.returncode == 0 else {}
    decoded = {r["cue"]: Path(r["wav"]) for r in read(st / "decoded/decode-receipt.json")["records"]}
    on_disk = sorted(p.name for p in (pc / "decode-verification").iterdir()) if rc.returncode == 0 else []
    rows_ok = [f["cue"] for f in ver["files"] if f["matches_decoded"]
               and sha_file(pc / "decode-verification" / f"{f['cue']}.wav") == f["sha256"]
               and pcm(pc / "decode-verification" / f"{f['cue']}.wav") == pcm(decoded[f["cue"]])]
    identical = sum(sha_file(pc / "decode-verification" / f"{c}.wav") == sha_file(decoded[c]) for c in rows_ok)
    same_out = rb.returncode == 0 and rc.returncode == 0 and all(
        (v / "prepared-b" / n).read_bytes() == (pc / n).read_bytes() for n in ("spans.json", "phrase-marks-plan.json"))
    check("v1-prepare-keeps-conversions", same_out and on_disk == [f"{c}.wav" for c in cues] and rows_ok == cues
          and held.get("sha256") == sha_file(pc / "decode-verification.json") and held.get("files") == len(cues)
          and not (v / "prepared-b/decode-verification").exists(),
          f"the candidate's prepare exits 0 and keeps {len(on_disk)} conversions in its --out/decode-verification/, "
          f"each at the hash in decode-verification.json, whose hash the prepare receipt carries; each has the decoded "
          f"PCM ({identical}/{len(cues)} byte-identical files); spans.json and phrase-marks-plan.json equal the reviewed "
          f"patch's, which keeps nothing")
    # The reviewed patch's TemporaryDirectory against the candidate's kept folder, on a decoded file changed by one
    # sample whose receipt row was re-hashed (the review's w2).
    d = v / "decoded-swap"
    shutil.copytree(st / "decoded", d)
    receipt = read(d / "decode-receipt.json")
    for r in receipt["records"]:
        r["wav"] = str(d / "decoded" / Path(r["wav"]).name)
    data = bytearray((d / "decoded/01_4.wav").read_bytes())
    data[-2] ^= 1
    (d / "decoded/01_4.wav").write_bytes(bytes(data))
    next(r for r in receipt["records"] if r["cue"] == "01_4")["wav_sha256"] = sha_file(d / "decoded/01_4.wav")
    dump(d / "decode-receipt.json", receipt)
    res = {}
    for t in "bc":
        (v / f"tmp-{t}").mkdir()
        res[t] = tool(tools[t], *base, "--decoded", d, "--out", v / f"swap-{t}",
                      env=dict(os.environ, TMPDIR=str(v / f"tmp-{t}")))
    b, c = res["b"], res["c"]
    tmp_b, tmp_c = list((v / "tmp-b").iterdir()), list((v / "tmp-c").iterdir())
    check("v2-reviewed-patch-removed-its-conversions", b.returncode == 2 and "01_4: the decoded PCM is not this take's "
          "conversion" in b.stderr and not tmp_b and not (v / "swap-b").exists(),
          f"the reviewed patch refuses ({last(b.stderr)}), so it converted the takes and compared them, yet its TMPDIR "
          f"holds {len(tmp_b)} entries afterwards and it wrote no --out: TemporaryDirectory removed every conversion "
          f"(root's preservation finding)")
    sc = v / "swap-c"
    kept = read(sc / "decode-verification.json") if (sc / "decode-verification.json").is_file() else {"files": []}
    flags = {f["cue"]: f["matches_decoded"] for f in kept["files"]}
    evidence = sc / "decode-verification/01_4.wav"
    check("v3-candidate-keeps-the-evidence", c.returncode == 2 and "01_4: the decoded PCM is not this take's "
          "conversion" in c.stderr and flags == {cue: cue != "01_4" for cue in cues}
          and sorted(p.name for p in (sc / "decode-verification").iterdir()) == [f"{x}.wav" for x in cues]
          and pcm(evidence) == pcm(st / "decoded/decoded/01_4.wav") and pcm(evidence) != pcm(d / "decoded/01_4.wav")
          and "01_4" in (sc / "REFUSED.txt").read_text() and not (sc / "spans.json").exists() and not tmp_c,
          f"the candidate refuses with the same message, keeps all {len(flags)} conversions in its --out (01_4 marked "
          f"not matching; its kept file is the true conversion, not the changed decode), writes REFUSED.txt and no "
          f"spans; its TMPDIR stays empty ({len(tmp_c)} entries)")
    cand = load("bridge_candidate", tools["c"])
    ex = v / "exclusive"
    (ex / "decode-verification").mkdir(parents=True)
    (ex / "decode-verification/sentinel.txt").write_text("planted before the call\n", encoding="utf-8")
    try:
        cand.retain_conversions(ex, {"records": []}, {}, "ffmpeg")
        raised = None
    except FileExistsError as error:
        raised = type(error).__name__
    before = listing(pc)
    again = tool(tools["c"], *base, "--decoded", st / "decoded", "--out", pc)
    check("v4-exclusive-folders", raised == "FileExistsError" and listing(ex) == {
        "decode-verification/sentinel.txt": sha_bytes(b"planted before the call\n")} and again.returncode == 2
          and "exists; give a new folder" in again.stderr and listing(pc) == before,
          f"retain_conversions refuses an existing decode-verification/ ({raised}) and leaves the planted file alone; "
          f"prepare into an existing --out is refused ({last(again.stderr)}) and its {len(before)} files are unchanged")
    # timings: a copy of the author's own prepared folder (made by the candidate), positive and with planted changes.
    out = {}
    for kind in ("copy", "changed", "moved", "record"):
        p = v / f"prepared-{kind}"
        shutil.copytree(st / "prepared", p)
        if kind == "changed":
            data = bytearray((p / "decode-verification/01_3.wav").read_bytes())
            data[-2] ^= 1
            (p / "decode-verification/01_3.wav").write_bytes(bytes(data))
        elif kind == "moved":
            (p / "decode-verification/01_3.wav").rename(p / "moved-aside-01_3.wav")
        elif kind == "record":
            record = read(p / "decode-verification.json")
            record["kept"] = "edited after prepare"
            dump(p / "decode-verification.json", record)
        out[kind] = tool(tools["c"], "timings", "--prepared", p, "--narration", st / "narration",
                         "--out", v / f"timings-{kind}")
    wants = {"changed": "01_3: the retained conversion", "moved": "01_3: the retained conversion",
             "record": "decode-verification.json is missing or changed since prepare"}
    refused = {k: out[k].returncode == 2 and wants[k] in out[k].stderr and not (v / f"timings-{k}").exists()
               for k in wants}
    check("v5-timings-binds-the-kept-files", out["copy"].returncode == 0 and all(refused.values()),
          f"timings accepts a copy of the author's prepared folder ({last(out['copy'].stdout)}); it refuses, writing "
          f"nothing, when 01_3's kept conversion is changed in one byte ({last(out['changed'].stderr)}), moved aside, "
          f"or decode-verification.json is edited ({last(out['record'].stderr)})")


def section_reviewed_fixes(tools):
    say("== 6. The review's F1, F2 and F4 plants, on the candidate")
    st, v = SCRATCH / "st-c", SCRATCH / "v"
    rows = {(x["cue"], x["file"]): x for x in read(st / "report-a/takes.json")["takes"]}
    m = next(x for x in rows[("01_2", "01_2-download01.wav")]["anchors"] if x["byte"] == 65)
    ans = read(st / "answers.json")
    ans["answers"][f"onset:01_2:s{m['sentence']}:01_2-download01.wav"] = dict(BY, decision="sample", sample_24k=120000)
    dump(v / "answers-onset.json", ans)
    w1 = tool(tools["c"], "prepare", *world_args(st), "--chapter", "01", "--selection", st / "selection.json",
              "--reviewer", "candidate check", "--decoded", st / "decoded", "--answers", v / "answers-onset.json",
              "--out", v / "prepared-onset")
    res = {}
    for kind, sample in (("moved", 112800), ("outside", 70000)):
        p = v / f"prepared-{kind}-marks"
        shutil.copytree(st / "prepared", p)
        plan = read(p / "phrase-marks-plan.json")
        for x in plan["cues"]["01_2"]["marks"]:
            if x["byte"] == 65:
                x["sample_24k"] = sample
        dump(p / "phrase-marks-plan.json", plan)
        if kind == "outside":
            receipt = read(p / "prepare-receipt.json")
            receipt["phrase_marks_plan_sha256"] = sha_file(p / "phrase-marks-plan.json")
            dump(p / "prepare-receipt.json", receipt)
        res[kind] = tool(tools["c"], "timings", "--prepared", p, "--narration", st / "narration",
                         "--out", v / f"timings-{kind}-marks")
    dl = st / "world/takes/01_3-downloads01.json"  # last: this edits the world
    data = read(dl)
    data["request"] = "01_3-request02.json"
    dump(dl, data)
    w5 = tool(tools["c"], "decode", *world_args(st), "--chapter", "01", "--selection", st / "selection.json",
              "--out", v / "decoded-after-downloads-edit")
    check("f1-f4-still-refused", w1.returncode == 2 and "outside its sentence 1" in w1.stderr
          and not (v / "prepared-onset").exists()
          and res["moved"].returncode == 2 and "phrase-marks-plan.json changed since prepare" in res["moved"].stderr
          and res["outside"].returncode == 2 and "outside its sentence 1" in res["outside"].stderr
          and w5.returncode == 2 and "01_3: the downloads receipt 01_3-downloads01.json changed" in w5.stderr,
          f"F1 prepare (onset answered past the byte-65 mark): {last(w1.stderr)}, and no --out is written, since "
          f"intervals are checked before any conversion; F4: {last(res['moved'].stderr)}; F1 timings (hash forged): "
          f"{last(res['outside'].stderr)}; F2 decode: {last(w5.stderr)}")


def section_hygiene():
    if BUNDLE.name != "complex-frederick-timing-bridge-repair01":
        say("== 7. Hygiene: skipped (not run from the bundle)")
        return
    say("== 7. Hygiene of this bundle")
    machine = re.compile(r"/Users/|/home/|/var/folders|jemanuel|scratchpad|claude-501|@gmail")  # hygiene-pattern
    pronoun = re.compile(r"\b(he|she|his|her|him|hers|himself|herself)\b", re.I)  # hygiene-pattern
    files = sorted(p for p in BUNDLE.rglob("*") if p.is_file() and p.name != "SHA256SUMS"
                   and "__pycache__" not in p.relative_to(BUNDLE).parts)
    names = [str(p.relative_to(BUNDLE)) for p in files]
    want = ["HANDOFF.md", "checks/bridge_selftest.py", "checks/outputs/repair_checks.txt", "checks/repair_checks.py",
            "frederick_timing_bridge.py", "patch/bridge-repair01.diff", "patch/delta-from-review01.diff"]
    dirty = [n for n, p in zip(names, files) if any(
        (machine.search(x) or pronoun.search(x)) and not x.endswith("# hygiene-pattern")
        for x in p.read_text(encoding="utf-8", errors="replace").splitlines())]
    check("h1-hygiene", names == want and not dirty,
          f"exactly the {len(want)} bundle files, with no machine path, user name, address or gendered pronoun"
          f"{(' ' + str(dirty)) if dirty else ''}{(' ' + str(names)) if names != want else ''}")


def main():
    global SCRATCH
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    SCRATCH = Path(sys.argv[1]).absolute()
    if SCRATCH.exists() or not (REPO / "tools/coherent_narration.py").is_file():
        print("give a NEW scratch folder and run from the repository root")
        return 2
    SCRATCH.mkdir(parents=True)
    section_pins()
    reviewed = section_diffs()
    section_selftests()
    tools = {"a": SCRATCH / "bundle-a/frederick_timing_bridge.py", "b": reviewed, "c": CANDIDATE}
    section_report(tools)
    section_retention(tools)
    section_reviewed_fixes(tools)
    section_hygiene()
    say(f"RESULT {'PASS' if not FAILS else 'FAIL'}, {len(FAILS)} failed{(': ' + ', '.join(FAILS)) if FAILS else ''}")
    return 0 if not FAILS else 1


if __name__ == "__main__":
    sys.exit(main())
