"""Independent checks of the sealed complex-frederick-timing-bridge01 (SHA256SUMS f31fd5c5), for root Mails 4036 and 4042.

From the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-timing-independent01/checks/independent_checks.py NEW_SCRATCH_DIR

Writes only under NEW_SCRATCH_DIR, which must not exist. Root's lane, take folder and job folders are only read:
plan-01..05 and ingest-01..05 of the lane are copied into scratch at pinned hashes, and every planted case edits
those copies or the author's own self-test world. No production audio is decoded. ffmpeg runs only on synthetic WAVs
and the self-test's noise fixtures. The one real-lane decode uses a stub that writes synthetic PCM, and its receipt
names the stub. The bridge's own report hashes the real takes, as it always does. Nothing is heard.

Exit 0 PASS, 1 FAIL, 2 usage, 3 a pinned input moved.
"""
from array import array
from concurrent.futures import ThreadPoolExecutor
import contextlib
import hashlib
import importlib.util
import io
import json
import math
from pathlib import Path
import re
import shutil
import subprocess
import sys
import types
import wave

sys.dont_write_bytecode = True
REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
BUNDLE = HERE.parent
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
BRIDGE_DIR = f"{P}/complex-frederick-timing-bridge01"
BRIDGE = f"{BRIDGE_DIR}/frederick_timing_bridge.py"
SELFTEST = f"{BRIDGE_DIR}/checks/bridge_selftest.py"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
LANE = f"{EC}/frederick-stage05-recognition01"
TAKES = f"{EC}/frederick-stage05-01"
WORK = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
SUPP = (f"{WORK}/complex-frederick-targeted-small01", f"{EC}/frederick-targeted-small01/inputs.json")
CRATE = f"{EC}/combined-repair-stage05/explainer_corpus/complex_analysis_visual"
PATCH = BUNDLE / "patch/bridge-fixes.diff"
PATCHED_SHA = "7d689793daa9ad7520b6c456acba791a1b84fb8fb12e0529489679baacfa247b"
PINS = {
    f"{BRIDGE_DIR}/SHA256SUMS": "f31fd5c5708fc19db33c6a16cf2e15302141a39277732290872f964d2acb9b8f",
    BRIDGE: "c820376e6bd1ce4cbb1ecee099563621b0aeea035a5ec77e6d83c3ad901ade5a",
    SELFTEST: "ab7f53ff83a4c9172b4949c7dce15741bcbb9b99df5743927636f07100220843",
    "tools/coherent_narration.py": "95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5",
    "tools/narration_cache.py": "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0",
    "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
    f"{CRATE}/src/chapters/draw.rs": "3f1c581dbc028a4d1a50a46d15028a0e005e0a76cf58709c6fbbbce2d30ea2db",
    f"{P}/complex-frederick-audio-preparation01/frederick_intake.py":
        "edc154c6c4f7277715981678465be0189e58d6f36b17822c23a608defd7a75d0",
    f"{P}/logarithm-stage08-audio-preparation01/prepare_audio.py":
        "3083d5cdfc98f18c404278155b9d432a0530e7b0eae0b0c09afca02e27c2d540",
}
LANE_PINS = {  # the lane at this review: plans and ingests 01-05; later ones are additive and not read here
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
BY = {"by": "independent check", "basis": "planted case in scratch; not a selection or answer of root's"}
SCRATCH = None
LINES, FAILS = [], []


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha_bytes(Path(path).read_bytes())


def say(text=""):
    text = str(text)
    for path, name in ((SCRATCH.resolve(), "SCRATCH"), (SCRATCH, "SCRATCH"), (REPO.resolve(), "REPO"), (REPO, "REPO")):
        text = text.replace(str(path), name)
    LINES.append(text)
    print(text, flush=True)


def check(name, ok, detail):
    if not ok:
        FAILS.append(name)
    say(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")


def run(argv, cwd=None):
    return subprocess.run([str(a) for a in argv], cwd=cwd or REPO, capture_output=True, text=True)


def tool(path, *args):
    return run([sys.executable, "-I", "-B", path, *args])


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


def write_wav(path, rate, samples):
    data = array("h", samples)
    if sys.byteorder != "little":
        data.byteswap()
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(data.tobytes())


def pcm_samples(pcm):
    data = array("h")
    data.frombytes(pcm)
    if sys.byteorder != "little":
        data.byteswap()
    return data


def refusal(fn, *args):
    """None if fn accepts, else the refusal text (any exception named Refused)."""
    try:
        fn(*args)
        return None
    except Exception as error:  # the modules' own Refused classes
        if type(error).__name__ != "Refused":
            raise
        return str(error)


def kit_accepts(text, rows, length):
    """narration.rs Narrator::load, phrase-timings branch (lines 924-949), re-ported: byte in range and on a char
    boundary, time finite, non-decreasing from 0 and before the WAV's end, bytes strictly increasing, no repeat."""
    raw, previous, previous_byte, seen = text.encode("utf-8"), 0.0, None, set()
    for at, time in rows:
        boundary = at == 0 or (at < len(raw) and raw[at] & 0xC0 != 0x80)
        if at >= len(raw) or not boundary or not math.isfinite(time) or time < previous or time >= length \
                or (previous_byte is not None and at <= previous_byte) or at in seen:
            return False
        seen.add(at)
        previous, previous_byte = time, at
    return True


def film_offsets(text):
    """draw.rs sentence_offsets (lines 82-91), re-ported: char offsets after '.', '?' or '!' and a space."""
    chars = list(text)
    return [0] + [i + 2 for i in range(max(len(chars) - 2, 0)) if chars[i] in ".?!" and chars[i + 1] == " "]


# ----------------------------------------------------------------------------- sections

def section_pins():
    say("== 1. Pins")
    moved = [p for p, h in PINS.items() if not (REPO / p).is_file() or sha_file(REPO / p) != h]
    moved += [f"{LANE}/{p}" for p, h in LANE_PINS.items()
              if not (REPO / LANE / p).is_file() or sha_file(REPO / LANE / p) != h]
    if not PATCH.is_file():
        moved.append(str(PATCH))
    if moved:
        say("PIN MOVED: " + ", ".join(moved))
        sys.exit(3)
    sums = (REPO / BRIDGE_DIR / "SHA256SUMS").read_text().splitlines()
    bad = [line for line in sums if sha_file(REPO / BRIDGE_DIR / line.split("  ", 1)[1]) != line.split("  ", 1)[0]]
    check("p1-pins", not bad and len(sums) == 12,
          f"{len(PINS)} tools and sources plus {len(LANE_PINS)} lane files (plan/ingest 01-05) at their pinned hashes; "
          f"the bridge seal f31fd5c5 verifies {len(sums) - len(bad)}/{len(sums)}")


def section_patch():
    say("== 2. The proposed patch (unapplied; apply it to a copy, never to the sealed folder)")
    tree = SCRATCH / "tree"
    (tree / BRIDGE).parent.mkdir(parents=True)
    shutil.copyfile(REPO / BRIDGE, tree / BRIDGE)
    init = run(["git", "init", "-q"], cwd=tree)
    applied = run(["git", "apply", PATCH], cwd=tree)
    got = sha_file(tree / BRIDGE) if applied.returncode == 0 else None
    lines = PATCH.read_text().splitlines()
    plus = sum(x.startswith("+") and not x.startswith("+++") for x in lines)
    minus = sum(x.startswith("-") and not x.startswith("---") for x in lines)
    on_repo = run(["git", "apply", "--check", PATCH])
    check("p2-patch-applies", init.returncode == 0 and got == PATCHED_SHA and on_repo.returncode == 0,
          f"git apply on a scratch copy of the sealed bridge gives {str(got)[:8]} (pinned {PATCHED_SHA[:8]}); "
          f"+{plus} -{minus} lines in 1 file; `git apply --check` also passes against the sealed file in place")
    for tag, src in (("a", REPO / BRIDGE), ("b", tree / BRIDGE)):
        (SCRATCH / f"bundle-{tag}/checks").mkdir(parents=True)
        shutil.copyfile(src, SCRATCH / f"bundle-{tag}/frederick_timing_bridge.py")
        shutil.copyfile(REPO / SELFTEST, SCRATCH / f"bundle-{tag}/checks/bridge_selftest.py")
    return SCRATCH / "bundle-a/frederick_timing_bridge.py", SCRATCH / "bundle-b/frederick_timing_bridge.py"


def section_selftests():
    say("== 3. The author's 37 checks, sealed (a) and patched (b)")
    with ThreadPoolExecutor(2) as pool:
        out = dict(zip("ab", pool.map(lambda t: tool(SCRATCH / f"bundle-{t}/checks/bridge_selftest.py",
                                                     SCRATCH / f"st-{t}"), "ab")))
    a, b = (out[t].stdout.replace(str(SCRATCH / f"st-{t}"), "ST") for t in "ab")
    check("s1-selftest-both", all(out[t].returncode == 0 and "SELFTEST: PASS (37/37)" in out[t].stdout for t in "ab")
          and a == b, "the sealed and the patched bridge each pass 37/37 with identical transcripts, so the patch's "
                      "new checks accept the author's synthetic world (plan binding, receipts, PCM re-derivation, "
                      "intervals, marks-plan hash)")


def section_report(tools):
    say("== 4. The bridge's report on a frozen copy of root's lane (plans and ingests 01-05)")
    lane = SCRATCH / "lane"
    for rel_path in LANE_PINS:
        (lane / rel_path).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPO / LANE / rel_path, lane / rel_path)
    common = ["report", "--lane", lane, "--supplement", *SUPP]
    with ThreadPoolExecutor(2) as pool:
        res = list(pool.map(lambda t: tool(tools[t], *common, "--out", SCRATCH / f"report-{t}"), "ab"))
    names = ("source.json", "takes.json", "questions.json", "answers-template.json", "coverage.json")
    same = all(r.returncode == 0 for r in res) and all(
        (SCRATCH / "report-a" / n).read_bytes() == (SCRATCH / "report-b" / n).read_bytes() for n in names)
    c = read(SCRATCH / "report-a/coverage.json")["counts"]
    check("r1-report-unchanged-by-patch", same and c["takes"] == 62,
          f"both exit 0 with byte-identical outputs: {c['takes']} takes of {c['cues_with_current_takes']} cues; "
          f"sentence onsets {c['sentence_onsets_supported']}/{c['sentence_onsets']} supported; anchor marks "
          f"{c['anchor_marks_supported']}/{c['anchor_marks']} supported (+{c['anchor_marks_at_sentence_onsets']} at "
          f"sentence onsets); {c['questions']} questions, all blocking")
    return lane


def section_interval(mods):
    say("== 5. F1: a mark is never checked against its own sentence (real rows; resolve in-process)")
    A, B = mods
    rows = read(SCRATCH / "report-a/takes.json")["takes"]
    take = {(t["cue"], t["file"]): t for t in rows}
    base = {f"onset:05_4:s3:05_4-download0{n}.wav": dict(BY, decision="use_candidate") for n in (1, 2)}
    same, refused = 0, []
    for t in rows:
        f24 = (t["frames"] + 1) // 2
        ra, rb = refusal(A.resolve, t, base, f24), refusal(B.resolve, t, base, f24)
        if ra or rb:
            refused.append((t["file"], ra, rb))
            continue
        sa, ea, ma = A.resolve(t, base, f24)
        sb, eb, mb = B.resolve(t, base, f24)
        same += (sa, ea, ma) == (sb, eb, [{k: v for k, v in m.items() if k != "sentence"} for m in mb])
    check("i1-real-takes-resolve-unchanged", same == len(rows) == 62 and not refused,
          f"all {len(rows)} real takes resolve identically under the sealed and the patched resolve (the only input "
          f"is use_candidate for 05_4 s3 in both takes); the patch adds the sentence index to each mark")
    margins, outside = [], 0
    for t in rows:
        f24 = (t["frames"] + 1) // 2
        _s, ends, _m = A.resolve(t, base, f24)
        bounds = [0] + ends
        for m in t["anchors"]:
            if m["status"] == "supported":
                k = m["sentence"]
                outside += not bounds[k] <= m["sample_24k"] < bounds[k + 1]
                margins.append((m["sample_24k"] - bounds[k], bounds[k + 1] - m["sample_24k"], t["cue"], t["file"],
                                m["byte"], k))
    low, high = min(margins), min(margins, key=lambda x: x[1])
    check("i2-real-margins", outside == 0 and len(margins) == 50,
          f"today none of the {len(margins)} supported mid-sentence marks lies outside its sentence; the closest to "
          f"its own onset is {low[2]} {low[3]} byte {low[4]}, {low[0] / 24000:.3f} s after sentence {low[5]}; the "
          f"closest to the next onset is {high[2]} {high[3]} byte {high[4]}, {high[1] / 24000:.3f} s before it")

    def plant(name, cue, file, byte, make):
        t = take[(cue, file)]
        f24 = (t["frames"] + 1) // 2
        m = next(x for x in t["anchors"] if x["byte"] == byte)
        _s, ends, _m = A.resolve(t, base, f24)
        bounds = [0] + ends
        pid, sample = make(t, m, bounds)
        answers = dict(base, **{pid: dict(BY, decision="sample", sample_24k=sample)})
        ra = refusal(A.resolve, t, answers, f24)
        rb = refusal(B.resolve, t, answers, f24)
        detail = f"answer {pid} sample {sample}: "
        if ra is None:
            _s2, ends2, marks2 = A.resolve(t, answers, f24)
            b2 = [0] + ends2
            got = next(x for x in marks2 if x["byte"] == byte)["sample_24k"]
            k = m["sentence"]
            inside = b2[k] <= got < b2[k + 1]
            text = SCRIPT[cue]
            kit = kit_accepts(text, [(x["byte"], x["sample_24k"] / 24000) for x in marks2], f24 / 24000)
            detail += (f"sealed resolve accepts, mark at {got} against sentence {k} = [{b2[k]}, {b2[k + 1]}); the kit's "
                       f"loader would {'accept' if kit else 'refuse'} these marks; patched: {last(rb)}")
            ok = not inside and kit and rb is not None and f"outside its sentence {k}" in rb
        else:
            detail += f"sealed resolve refused: {ra}"
            ok = False
        check(name, ok, detail)

    plant("i3-onset-moved-past-mark", "02_6", "02_6-download02.wav", 139,
          lambda t, m, b: (f"onset:02_6:s{m['sentence']}:02_6-download02.wav", m["sample_24k"] + 240))
    plant("i4-mark-after-next-onset", "02_1", "02_1-download02.wav", 91,
          lambda t, m, b: (m["id"], b[m["sentence"] + 1] + 240))
    plant("i5-mark-before-own-onset", "02_6", "02_6-download02.wav", 139,
          lambda t, m, b: (m["id"], b[m["sentence"]] - 240))
    return take


def stub_subprocess(frames):
    """Stands in for ffmpeg in one in-process cmd_decode: writes synthetic 24 kHz PCM of the expected length."""

    def run_stub(argv, **_kw):
        if argv[1:] == ["-version"]:
            return subprocess.CompletedProcess(argv, 0, stdout="ffmpeg STUB: synthetic PCM, no audio decoded\n",
                                               stderr="")
        n, seed = frames[argv[argv.index("-i") + 1]], len(argv[-1])
        write_wav(argv[-1], 24000, [((i * 7919 + seed) % 4001) - 2000 for i in range(n)])
        return subprocess.CompletedProcess(argv, 0)

    return types.SimpleNamespace(run=run_stub, CompletedProcess=subprocess.CompletedProcess)


def section_binding(tools, mods, lane):
    say("== 6. F2: plan.json, which names each take's request, is never re-bound to the frozen inputs")
    A, B = mods
    # As the bridge's own dry run does: byte copies of the pinned assembler, its cache and the stage05 map.
    _script, files = A.dep("intake").check_binding(REPO / CRATE)
    for src, dst, pin in A.workspace_copies(files):
        (SCRATCH / "workspace" / dst).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, SCRATCH / "workspace" / dst)
        assert sha_file(SCRATCH / "workspace" / dst) == pin
    coherent = load("workspace_coherent", SCRATCH / "workspace/tools/coherent_narration.py")
    real = read(lane / "plan-01/plan.json")
    t0 = next(t for t in real["takes"] if t["name"] == "01_1-download01.wav")
    req = read(Path(t0["path"]).parent / t0["receipts"]["request"]["name"])
    planted = SCRATCH / "elsewhere/01_1-request01.json"
    dump(planted, dict(req, planted="outside the take folder; never frozen by the recognition guard"))
    planted_sha = sha_file(planted)
    lane_a = SCRATCH / "lane-planted-a"
    shutil.copytree(lane, lane_a)
    plan = read(lane_a / "plan-01/plan.json")
    for t in plan["takes"]:
        if t["name"] == "01_1-download01.wav":
            t["receipts"]["request"] = {"name": str(planted), "sha256": planted_sha}
    dump(lane_a / "plan-01/plan.json", plan)
    lane_b = SCRATCH / "lane-planted-b"
    shutil.copytree(lane_a, lane_b)
    frozen = read(lane_b / "plan-01/frozen-inputs.json")
    for f in frozen["files"]:
        if f["role"] == "plan":
            f["sha256"] = sha_file(lane_b / "plan-01/plan.json")
    dump(lane_b / "plan-01/frozen-inputs.json", frozen)
    ingest = read(lane_b / "ingest-01/ingest.json")
    ingest["frozen_inputs_sha256"] = sha_file(lane_b / "plan-01/frozen-inputs.json")
    dump(lane_b / "ingest-01/ingest.json", ingest)
    results = {}
    for name, where in (("a", lane_a), ("b", lane_b)):
        for t in "ab":
            results[(name, t)] = tool(tools[t], "report", "--lane", where, "--supplement", *SUPP,
                                      "--out", SCRATCH / f"report-planted-{name}-{t}")
    rec = {}
    for name in "ab":
        r = results[(name, "a")]
        rows = read(SCRATCH / f"report-planted-{name}-a/takes.json")["takes"] if r.returncode == 0 else []
        rec[name] = next((x["request"] for x in rows if x["file"] == "01_1-download01.wav"), {})
    check("b1-plan-rebound-request", results[("a", "a")].returncode == 0
          and rec["a"].get("path") == str(planted.resolve()) and results[("a", "b")].returncode == 2
          and "plan-01/plan.json is not the plan its guard froze" in results[("a", "b")].stderr,
          f"plan.json alone edited: the sealed report binds 01_1-download01 to {rec['a'].get('path')} "
          f"({str(rec['a'].get('sha256'))[:8]}); patched: {last(results[('a', 'b')].stderr)}")
    check("b2-consistent-forgery", results[("b", "a")].returncode == 0 and rec["b"].get("path") == str(planted.resolve())
          and results[("b", "b")].returncode == 2 and "is not a bare name in the take folder that the guard froze"
          in results[("b", "b")].stderr,
          f"plan.json, its frozen-inputs entry and the ingest hash all rewritten to agree: the sealed report still binds "
          f"the planted request; patched: {last(results[('b', 'b')].stderr)}")

    # The sealed decode writes that binding into a v2 receipt, and the canonical assembler accepts it.
    script = SCRIPT
    sel ={"schema": A.SELECTION_SCHEMA, "source_sha256": read(SCRATCH / "report-a/source.json")["sha256"],
           "script_sha256": read(SCRATCH / "report-a/source.json")["pins"]["script_sha256"], "chapter": "01",
           "takes": {c: f"{c}-download01.wav" for c in script if c.startswith("01_")}, **BY}
    dump(SCRATCH / "selection-01.json", sel)
    frames = {t["path"]: (t["frames"] + 1) // 2 for t in read(SCRATCH / "report-a/takes.json")["takes"]}
    script_rows, units = coherent.read_units(REPO / CRATE / "script.tsv", REPO / CRATE / "narration_units.json")
    unit = next(u for u in units if u["line_ids"] == ["01_1"])

    def decode(mod, where, out):
        args = types.SimpleNamespace(crate=str(REPO / CRATE), lane=str(where), jobs=str(REPO / WORK),
                                     takes=str(REPO / TAKES), supplement=[list(SUPP)],
                                     selection=str(SCRATCH / "selection-01.json"), chapter="01", out=str(out),
                                     ffmpeg="ffmpeg")
        real_sub, mod.subprocess = mod.subprocess, stub_subprocess(frames)
        try:
            with contextlib.redirect_stdout(io.StringIO()):  # its success line names absolute scratch paths
                return refusal(mod.cmd_decode, args)
        finally:
            mod.subprocess = real_sub

    def assembler_accepts(out):
        receipt = read(out / "decode-receipt.json")
        row = next(r for r in receipt["records"] if r["cue"] == "01_1")
        t = next(x for x in read(SCRATCH / "report-a/takes.json")["takes"] if x["file"] == "01_1-download01.wav")
        _s, ends, _m = A.resolve(t, {}, row["samples"])
        record = {"id": unit["id"], "wav": row["wav"], "sha256": row["wav_sha256"],
                  "generation_receipt": str(out / "decode-receipt.json"), "sentence_ends": ends,
                  "gap_after_samples": [0] * len(ends)}
        try:
            spans = coherent.unit_spans(unit, script_rows, record)
            return row, f"accepted ({len(spans)} sentence spans, voice {coherent.external_ui_voice(receipt)!r})"
        except Exception as error:
            return row, f"refused: {type(error).__name__}: {error}"

    bad = decode(A, lane_a, SCRATCH / "decode-planted-a")
    row, verdict = assembler_accepts(SCRATCH / "decode-planted-a") if bad is None else ({}, "no receipt")
    check("b3-v2-receipt-carries-it", bad is None and row.get("request") == str(planted.resolve())
          and verdict.startswith("accepted"),
          f"the sealed decode (ffmpeg stubbed, synthetic PCM) writes request {row.get('request')} into the v2 receipt; "
          f"canonical coherent_narration.unit_spans: {verdict}")
    good = decode(B, lane, SCRATCH / "decode-real-b")
    row, verdict = assembler_accepts(SCRATCH / "decode-real-b") if good is None else ({}, "no receipt")
    blocked = decode(B, lane_a, SCRATCH / "decode-planted-b")
    check("b4-patched-decode", good is None and row.get("request") == f"{TAKES}/01_1-request01.json"
          and verdict.startswith("accepted") and blocked is not None and "is not the plan its guard froze" in blocked,
          f"positive control on the unplanted lane copy: the patched decode re-checks all 6 downloads receipts, records "
          f"{row.get('request')}, and unit_spans is {verdict.split(' (')[0]}; on the planted copy it refuses: "
          f"{last(blocked)}")


def world_args(st):
    w = st / "world"
    return (["--takes", w / "takes", "--lane", w / "lane", "--jobs", w / "work",
             "--supplement", w / "work/complex-fixture-targeted-small01", w / "targeted/inputs.json"])


def section_world(tools):
    say("== 7. Planted cases in the author's self-test world (prepare and timings, sealed a / patched b)")
    restored = []
    for t in "ab":  # the self-test's last case (n22) appends one space to 01_2-request01.json and leaves it there
        req = SCRATCH / f"st-{t}/world/takes/01_2-request01.json"
        pin = next(x for x in read(SCRATCH / f"st-{t}/world/lane/plan-01/plan.json")["takes"]
                   if x["name"] == "01_2-download01.wav")["receipts"]["request"]["sha256"]
        text = req.read_text(encoding="utf-8")
        if text.endswith(" ") and sha_bytes(text[:-1].encode("utf-8")) == pin:
            req.write_text(text[:-1], encoding="utf-8")
        restored.append(sha_file(req) == pin)
    check("w0-world-restored", all(restored), "the self-test leaves its world with n22's one-space edit of "
                                              "01_2-request01.json; removing that space gives the plan's request hash "
                                              "again in both worlds, so the planted cases below start from a clean world")
    res = {}
    for t in "ab":
        st = SCRATCH / f"st-{t}"
        rows = {(x["cue"], x["file"]): x for x in read(st / "report-a/takes.json")["takes"]}
        m = next(x for x in rows[("01_2", "01_2-download01.wav")]["anchors"] if x["byte"] == 65)
        prep = ["prepare", *world_args(st), "--chapter", "01", "--selection", st / "selection.json",
                "--reviewer", "independent check"]
        # F1 through the whole prepare: a sentence onset answered past the mark that sits inside that sentence.
        ans = read(st / "answers.json")
        ans["answers"][f"onset:01_2:s{m['sentence']}:01_2-download01.wav"] = dict(BY, decision="sample",
                                                                               sample_24k=120000)
        dump(SCRATCH / f"w{t}/answers-onset.json", ans)
        res[("w1", t)] = tool(tools[t], *prep, "--decoded", st / "decoded", "--answers", SCRATCH / f"w{t}/answers-onset.json",
                              "--out", SCRATCH / f"w{t}/prepared-onset")
        # F3: one decoded file changed by one sample and its receipt row re-hashed.
        for kind in ("same", "swap"):
            d = SCRATCH / f"w{t}/decoded-{kind}"
            shutil.copytree(st / "decoded", d)
            receipt = read(d / "decode-receipt.json")
            for r in receipt["records"]:
                r["wav"] = str(d / "decoded" / Path(r["wav"]).name)
            if kind == "swap":
                data = bytearray((d / "decoded/01_4.wav").read_bytes())
                data[-2] ^= 1
                (d / "decoded/01_4.wav").write_bytes(bytes(data))
            receipt["records"][3]["wav_sha256"] = sha_file(d / "decoded/01_4.wav")
            dump(d / "decode-receipt.json", receipt)
            res[(f"w2{kind}", t)] = tool(tools[t], *prep, "--decoded", d, "--answers", st / "answers.json",
                                         "--out", SCRATCH / f"w{t}/prepared-{kind}")
        # F4: the marks plan edited after prepare; then (consistent forgery) also its hash in the prepare receipt.
        for kind, sample in (("moved", 112800), ("outside", 70000)):
            p = SCRATCH / f"w{t}/prepared-{kind}-marks"
            shutil.copytree(st / "prepared", p)
            plan = read(p / "phrase-marks-plan.json")
            for x in plan["cues"]["01_2"]["marks"]:
                if x["byte"] == 65:
                    x["sample_24k"] = sample
            dump(p / "phrase-marks-plan.json", plan)
            if kind == "outside" and t == "b":
                receipt = read(p / "prepare-receipt.json")
                receipt["phrase_marks_plan_sha256"] = sha_file(p / "phrase-marks-plan.json")
                dump(p / "prepare-receipt.json", receipt)
            res[(f"w3{kind}", t)] = tool(tools[t], "timings", "--prepared", p, "--narration", st / "narration",
                                         "--out", SCRATCH / f"w{t}/timings-{kind}")
        # F2b: a downloads receipt that no longer names the request its take came from.
        dl = st / "world/takes/01_3-downloads01.json"
        data = read(dl)
        data["request"] = "01_3-request02.json"
        dump(dl, data)
        res[("w4", t)] = tool(tools[t], "decode", *world_args(st), "--chapter", "01", "--selection",
                              st / "selection.json", "--out", SCRATCH / f"w{t}/decoded-after-downloads-edit")
    a, b = res[("w1", "a")], res[("w1", "b")]
    plan = read(SCRATCH / "wa/prepared-onset/phrase-marks-plan.json")["cues"]["01_2"] if a.returncode == 0 else {}
    starts = [s["sample_24k"] for s in plan.get("sentence_starts_24k", [])]
    mark = next((x["sample_24k"] for x in plan.get("marks", []) if x["byte"] == 65), None)
    check("w1-prepare-onset-past-mark", a.returncode == 0 and mark is not None and starts and mark < starts[1]
          and b.returncode == 2 and "outside its sentence 1" in b.stderr,
          f"sealed prepare (its dry run through the pinned assembler included) exits 0 with 01_2 sentence starts "
          f"{starts} and the byte-65 mark at {mark}, before its own sentence; patched: {last(b.stderr)}")
    sa, sb = res[("w2same", "a")], res[("w2same", "b")]
    wa, wb = res[("w2swap", "a")], res[("w2swap", "b")]
    spans = read(SCRATCH / "wa/prepared-swap/spans.json")["units"] if wa.returncode == 0 else []
    swapped = sha_file(SCRATCH / "wa/decoded-swap/decoded/01_4.wav")
    check("w2-decoded-pcm-not-rederived", sa.returncode == 0 and sb.returncode == 0 and wa.returncode == 0
          and any(u["sha256"] == swapped for u in spans) and wb.returncode == 2
          and "01_4: the decoded PCM is not this take's conversion" in wb.stderr,
          f"01_4's decoded file changed in its last sample and its receipt row re-hashed: the sealed prepare exits 0 and "
          f"its spans carry the changed file ({swapped[:8]}); patched: {last(wb.stderr)}; the unchanged copy passes "
          f"both")
    ma, mb = res[("w3moved", "a")], res[("w3moved", "b")]
    frag = read(SCRATCH / "wa/timings-moved/phrase-timings-fragment-01.json") if ma.returncode == 0 else {"cues": {}}
    moved = [x["seconds"] for x in frag["cues"].get("01_2", {}).get("marks", []) if x["byte"] == 65]
    check("w3-marks-plan-unbound", ma.returncode == 0 and moved == [112800 / 24000] and mb.returncode == 2
          and "phrase-marks-plan.json changed since prepare" in mb.stderr,
          f"01_2's byte-65 mark edited in phrase-marks-plan.json after prepare: the sealed timings writes it at "
          f"{moved} s; patched: {last(mb.stderr)}")
    oa, ob = res[("w3outside", "a")], res[("w3outside", "b")]
    frag = read(SCRATCH / "wa/timings-outside/phrase-timings-fragment-01.json") if oa.returncode == 0 else {"cues": {}}
    early = [x["seconds"] for x in frag["cues"].get("01_2", {}).get("marks", []) if x["byte"] == 65]
    check("w4-timings-interval", oa.returncode == 0 and early == [70000 / 24000] and ob.returncode == 2
          and "outside its sentence 1" in ob.stderr,
          f"the same mark moved to 70000, before 01_2 sentence 1 (79200): the sealed timings writes {early} s; patched, "
          f"with the receipt hash forged to match: {last(ob.stderr)}")
    da, db = res[("w4", "a")], res[("w4", "b")]
    check("w5-downloads-not-reread", da.returncode == 0 and db.returncode == 2
          and "01_3: the downloads receipt 01_3-downloads01.json changed or does not list" in db.stderr,
          f"01_3's downloads receipt rewritten to name another request: the sealed decode exits 0 ({last(da.stdout)}); "
          f"patched: {last(db.stderr)}")


def section_ffmpeg(A, coherent):
    say("== 8. The decode contract on synthetic WAVs (root's ffmpeg arguments)")
    d = SCRATCH / "ffmpeg"
    d.mkdir()
    version = run(["ffmpeg", "-version"]).stdout.splitlines()[0].split(" Copyright")[0]
    got, short, ok = [], [], True
    for n in (1, 7, 63, 64, 65, 129, 4097, 47999, 48001, 794881):
        src, dst = d / f"n{n}.wav", d / f"n{n}-24k.wav"
        write_wav(src, 48000, [((i * 7919) % 4001) - 2000 for i in range(n)])
        r = run(["ffmpeg", *A.FFMPEG_ARGS, "-i", src, *A.FFMPEG_OUT, dst])
        params, frames, pcm = A.wav_info(dst)
        if n < 64:  # below the rule's domain ffmpeg returns fewer frames; the bridge's frame check refuses those
            short.append(f"{n}->{frames}")
            ok = ok and r.returncode == 0 and frames < (n + 1) // 2
            continue
        _info, pcm2 = coherent.wav_pcm(dst)
        got.append(f"{n}->{frames}")
        ok = ok and r.returncode == 0 and params == (1, 2, 24000) and frames == (n + 1) // 2 and pcm2 == pcm
    shortest = min(t["frames"] for t in read(SCRATCH / "report-a/takes.json")["takes"])
    check("f1-frame-count", ok and shortest >= 64,
          f"{version}: frames are ceil(n/2) for {', '.join(got)}, and the bridge's wav_info and the assembler's wav_pcm "
          f"read the same samples; below 64 input samples ffmpeg returns fewer ({', '.join(short)}), which "
          f"expected_frames would refuse (fail-closed); the shortest real take has {shortest} frames")
    spikes = (24000, 24001, 48007, 72000, 72001)
    src, dst = d / "impulses.wav", d / "impulses-24k.wav"
    samples = [0] * 96000
    for i in spikes:
        samples[i] = 20000
    write_wav(src, 48000, samples)
    r = run(["ffmpeg", *A.FFMPEG_ARGS, "-i", src, *A.FFMPEG_OUT, dst])
    out = pcm_samples(A.wav_info(dst)[2])
    peaks = []
    for i in spikes:
        lo = i // 2 - 40
        window = [abs(x) for x in out[lo:lo + 81]]
        peaks.append(lo + window.index(max(window)))
    offsets = [p - i / 2 for p, i in zip(peaks, spikes)]
    check("f2-no-shift", r.returncode == 0 and all(abs(x) <= 0.5 for x in offsets),
          f"single-sample impulses at 48 kHz samples {list(spikes)} peak at 24 kHz samples {peaks} (offsets "
          f"{offsets} samples): no resampler delay, so sample_24k = round(t * 24000) addresses the decoded file")
    before = dst.read_bytes()
    again = run(["ffmpeg", *A.FFMPEG_ARGS, "-i", src, *A.FFMPEG_OUT, dst])
    check("f3-no-overwrite-exit-0", again.returncode == 0 and "already exists" in again.stderr
          and dst.read_bytes() == before,
          f"`-n` leaves an existing target's bytes unchanged but exits {again.returncode}, so check=True cannot see a "
          "skipped write; the bridge never meets it, since decode writes each cue to a new file in a new folder")


def section_chain(A, lane):
    say("== 9. The real producer chain behind every ingested take (read-only)")
    intake = A.dep("intake")
    script = SCRIPT
    plans = bound = takes = 0
    bad, cues = [], set()
    folder_names = {}
    for n in range(1, 6):
        plan = read(lane / f"plan-0{n}/plan.json")
        frozen = read(lane / f"plan-0{n}/frozen-inputs.json")["files"]
        plans += 1
        pin = [f for f in frozen if f["role"] == "plan"]
        bound += len(pin) == 1 and pin[0]["sha256"] == sha_file(lane / f"plan-0{n}/plan.json")
        held = {(Path(f["path"]).parent.name, Path(f["path"]).name): f["sha256"] for f in frozen if f["role"] == "receipt"}
        for t in plan["takes"]:
            takes += 1
            folder = Path(t["path"]).parent
            cue = t["cue"]
            cues.add(cue)
            why = []
            for kind, r in t["receipts"].items():
                p = folder / r["name"]
                if Path(r["name"]).name != r["name"] or not p.is_file() or sha_file(p) != r["sha256"] \
                        or held.get((folder.name, r["name"])) != r["sha256"]:
                    why.append(f"{kind} receipt not bare, current and frozen")
            q = read(folder / t["receipts"]["request"]["name"])
            if (q.get("cue"), q.get("text"), q.get("voice"), q.get("script_sha256"), q.get("submitted")) != (
                    cue, script[cue], intake.VOICE, intake.PINS["script"], False) \
                    or q.get("text_sha256") != sha_bytes(q["text"].encode("utf-8")):
                why.append("request is not the prepared, unsubmitted current text and Frederick voice")
            dl = read(folder / t["receipts"]["downloads"]["name"])
            if dl.get("cue") != cue or dl.get("request") != t["receipts"]["request"]["name"] or not any(
                    Path(x["path"]).name == t["name"] and (x["sha256"], x["pcm_sha256"], x["frames"]) ==
                    (t["sha256"], t["pcm_sha256"], t["frames"]) for x in dl["takes"]):
                why.append("downloads receipt does not list this take from its request")
            if folder not in folder_names:
                folder_names[folder] = sorted(p.name for p in folder.iterdir() if p.is_file())
            binders = [name for name in folder_names[folder]
                       if intake.NAMES["submission"].fullmatch(name) and name.startswith(cue + "-")
                       and intake.binds(read(folder / name), t["receipts"]["request"]["name"],
                                        t["receipts"]["request"]["sha256"])]
            sub = read(folder / t["receipts"]["submission"]["name"])
            count = sub.get("generation_requests", 1 if sub.get("submitted_once") is True else None)
            if binders != [t["receipts"]["submission"]["name"]] or not (
                    sub.get("submitted") is True or sub.get("submitted_once") is True) or type(count) is not int \
                    or len(dl["takes"]) != 2 * count:
                why.append(f"submission binding: {binders}")
            if why:
                bad.append((t["name"], why))
    check("c1-real-chain", plans == bound == 5 and takes == 62 and not bad,
          f"{bound}/{plans} plan.json files equal their frozen entry; for all {takes} takes of {len(cues)} cues every "
          f"receipt is a bare name in the take folder, current and frozen at the plan's hash; each request is the "
          f"current text and voice with `submitted: false` (prepared only); each downloads receipt lists its take from "
          f"that request; exactly one submission binds it{(' ' + str(bad[:2])) if bad else ''}")
    return folder_names


def section_source(A):
    say("== 10. Source side: sentence starts and anchor bytes, against my own re-ports of the film's rules")
    bron = A.dep("log_preparation")
    source = read(SCRATCH / "report-a/source.json")
    differ = [c for c, x in SCRIPT.items() if [a for a, _ in bron.kit_sentences(x)] != film_offsets(x)]
    rust = (REPO / CRATE / "src/chapters/draw.rs").read_text(encoding="utf-8").splitlines()[81:91]
    rule = "\n".join(rust)
    check("x1-sentence-starts", not differ and "if matches!(chars[i], '.' | '?' | '!') && chars[i + 1] == ' ' {" in rule
          and "for i in 0..chars.len().saturating_sub(2) {" in rule and source["film_ignores_measured_sentence_starts"]
          == [], f"for all {len(SCRIPT)} cues the kit-rule sentence start characters equal draw.rs sentence_offsets "
                 "(lists, not only counts; draw.rs:82-91 pinned), so the film uses the measured starts for every cue")
    bad, repeated, onset = [], [], 0
    for a in source["anchors"]:
        text = SCRIPT[a["cue"]]
        raw = text.encode("utf-8")
        offs = film_offsets(text)
        k = max(j for j, at in enumerate(offs) if at <= a["char"])
        if a["byte"] != raw.find(a["phrase"].encode("utf-8")) or a["char"] != len(raw[:a["byte"]].decode("utf-8")) \
                or a["sentence"] != k or a["at_sentence_start"] != (a["char"] in offs) \
                or a["byte_end"] != a["byte"] + len(a["phrase"].encode("utf-8")):
            bad.append(a["cue"] + f":b{a['byte']}")
        if a["occurrences_in_cue"] > 1:
            repeated.append(f"{a['cue']} {a['phrase']!r}")
        onset += a["at_sentence_start"] and a["sentence"] > 0
    check("x2-anchor-bytes", not bad and len(source["anchors"]) == 93,
          f"all {len(source['anchors'])} anchors: byte = the first find() of the phrase (what Spoken::at looks up), "
          f"char and sentence agree with draw.rs, {onset} start a sentence k >= 1; phrases occurring more than once in "
          f"their cue: {repeated or 'none'}")


def section_051(A, folder_names):
    say("== 11. 05_1 (Mail 4042): what a supersession record would have to bind (read-only facts)")
    intake = A.dep("intake")
    folder = REPO / TAKES
    names = folder_names.get(folder.resolve()) or sorted(p.name for p in folder.iterdir() if p.is_file())
    request = "05_1-request01.json"
    req_sha = sha_file(folder / request)
    binders = [n for n in names if intake.NAMES["submission"].fullmatch(n) and n.startswith("05_1-")
               and intake.binds(read(folder / n), request, req_sha)]
    s2 = read(folder / "05_1-submission02.json")
    obs = read(folder / "05_1-submission-observation01.json")
    dl = read(folder / "05_1-downloads01.json")
    plan5 = read(SCRATCH / "lane/plan-05/plan.json")
    refused = [r["reasons"] for r in plan5["refused"] if r["cue"] == "05_1"]
    k = SCRATCH / "k051"
    k.mkdir()
    for n in (request, "05_1-downloads01.json", "05_1-submission02.json", "05_1-submission-observation01.json"):
        shutil.copyfile(folder / n, k / n)
    groups, _loose = intake.name_groups(k)
    reasons, _t, _r = intake.check_receipt(k, "05_1-downloads01.json", "05_1", SCRIPT, set(), groups)
    alone = [r for r in reasons if "submi" in r or "generation" in r]
    check("k1-051-facts", binders == ["05_1-submission01.json", "05_1-submission02.json"]
          and refused == [["2 submission receipts bind '05_1-request01.json'; expected one"]]
          and not {"submitted", "submitted_once", "generation_requests"} & set(s2)
          and str(obs.get("supersedes", "")).startswith("05_1-submission01.json") and len(dl["takes"]) == 2
          and any("not marked submitted" in r for r in alone),
          f"both {binders} bind {request} ({req_sha[:8]}) by request_sha256, so plans 04 and 05 refuse 05_1; "
          f"submission02 ({sha_file(folder / '05_1-submission02.json')[:8]}) has no submitted, submitted_once or "
          f"generation_requests, so even alone the sealed intake refuses it: {alone}; the observation "
          f"({sha_file(folder / '05_1-submission-observation01.json')[:8]}) names submission01 in `supersedes` but no "
          f"rule reads it; downloads01 ({sha_file(folder / '05_1-downloads01.json')[:8]}) lists 2 takes")


def section_hygiene():
    if BUNDLE.name != "complex-frederick-timing-independent01":
        say("== 12. Hygiene: skipped (not run from the bundle)")
        return
    say("== 12. Hygiene of this bundle")
    machine = re.compile(r"/Users/|/home/|/var/folders|jemanuel|scratchpad|claude-501|@gmail")  # hygiene-pattern
    pronoun = re.compile(r"\b(he|she|his|her|him|hers|himself|herself)\b", re.I)  # hygiene-pattern
    # Gitignored __pycache__ folders (written by editor hooks, never by these -B runs) are not part of the bundle.
    files = sorted(p for p in BUNDLE.rglob("*") if p.is_file() and p.name != "SHA256SUMS"
                   and "__pycache__" not in p.relative_to(BUNDLE).parts)
    names = [str(p.relative_to(BUNDLE)) for p in files]
    want = ["REVIEW.md", "checks/independent_checks.py", "checks/outputs/independent_checks.txt",
            "patch/bridge-fixes.diff"]
    dirty = [n for n, p in zip(names, files) if any(
        (machine.search(x) or pronoun.search(x)) and not x.endswith("# hygiene-pattern")
        for x in p.read_text(encoding="utf-8", errors="replace").splitlines())]
    check("h1-hygiene", names == want and not dirty,
          f"exactly the {len(want)} bundle files, with no machine path, user name, address or gendered pronoun"
          f"{(' ' + str(dirty)) if dirty else ''}{(' ' + str(names)) if names != want else ''}")


def main():
    global SCRATCH, SCRIPT
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    SCRATCH = Path(sys.argv[1]).absolute()
    if SCRATCH.exists() or not (REPO / "tools/coherent_narration.py").is_file():
        print("give a NEW scratch folder and run from the repository root")
        return 2
    SCRATCH.mkdir(parents=True)
    section_pins()
    tools = dict(zip("ab", section_patch()))
    section_selftests()
    lane = section_report(tools)
    A, B = load("bridge_sealed", tools["a"]), load("bridge_patched", tools["b"])
    SCRIPT = A.dep("intake").read_script(REPO / CRATE / "script.tsv")
    section_interval((A, B))
    section_binding(tools, (A, B), lane)
    section_world(tools)
    section_ffmpeg(A, load("canonical_coherent_ffmpeg", REPO / "tools/coherent_narration.py"))
    names = section_chain(A, lane)
    section_source(A)
    section_051(A, names)
    section_hygiene()
    say(f"RESULT {'PASS' if not FAILS else 'FAIL'}, {len(FAILS)} failed{(': ' + ', '.join(FAILS)) if FAILS else ''}")
    return 0 if not FAILS else 1


SCRIPT = {}
if __name__ == "__main__":
    sys.exit(main())
