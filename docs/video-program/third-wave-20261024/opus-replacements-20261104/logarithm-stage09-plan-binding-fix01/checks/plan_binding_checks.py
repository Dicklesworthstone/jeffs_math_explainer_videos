"""Checks for the phrase-plan hash binding (O3; root's Mail 4094, bead 87do.63). From the repository root:

    python3 -I -B .../logarithm-stage09-plan-binding-fix01/checks/plan_binding_checks.py FRESH_SCRATCH

LavenderPike. Read-only on the repository; writes only under FRESH_SCRATCH. The L1 helper (sealed in
logarithm-stage09-mark-interval-fix01) and this bundle's staged helper are imported side by side, and cmd_prepare and
cmd_timings run in process on the real stage09 binding for chapter 08. The selection and decode rows are TEST FIXTURES,
and the decoded WAVs are zero-filled stand-ins of the decoded length, never a decode of a recording. The assembly is of
those stand-ins, by prepare's own command, with the pinned assembler's workspace copy. No ffmpeg, recognition,
production decode or production assembly; no take is chosen, and nothing here is a listening check. P06 reruns
BronzeJaguar's sealed suite unchanged through the sealed L1 wrapper, with HELPER pointed at the staged file.
"""
import ast
import contextlib
import difflib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import types
import wave

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
UNDER = f"{P}/logarithm-stage09-audio-preparation01"
UNDER_SEAL = "2bb8e2800fd4d403ebd7bc73f8f6515b53a227125dc6a418fdd74e5e4c40bb13"
L1 = f"{P}/logarithm-stage09-mark-interval-fix01"
L1_SEAL = "60f5f63ad958a1e41c3d6b668f1166e9599dc76977480616eb08c50ae5b115a7"
L1_HELPER_SHA = "2f080ab7492638320697882bc6cc596e6fdb1330164df9cf980ba89ec9e2e73c"
HERE = f"{P}/logarithm-stage09-plan-binding-fix01"
STAGED = f"{HERE}/staged/prepare_audio.py"
PATCH = f"{HERE}/prepare_audio.patch"
BINDING = f"{UNDER}/bindings/logarithm-stage09-oliver.json"
CHANGED = ("04_4", "08_5")
FIXTURE = {"by": "LavenderPike TEST FIXTURE (not root)", "basis": "TEST FIXTURE ONLY: first recording of each cue; nobody chose or heard anything"}
BINDING_REFUSAL = "phrase-marks-plan.json is not the plan this prepare receipt binds"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha(Path(path).read_bytes())


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_sums(folder):
    sums = (REPO / folder / "SHA256SUMS").read_bytes()
    rows = [line.split("  ", 1) for line in sums.decode().splitlines()]
    return sha(sums), len(rows), [n for h, n in rows if sha_file(REPO / folder / n) != h]


def top_level(text):
    tree = ast.parse(text)
    defs = {n.name: ast.get_source_segment(text, n) for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))}
    return defs, [ast.dump(n) for n in tree.body if not isinstance(n, (ast.FunctionDef, ast.ClassDef))]


@contextlib.contextmanager
def scratch_out(module):
    """make_out without its repository-path guard, so outputs land in scratch; restored afterwards."""
    original, module.make_out = module.make_out, lambda b_, o: (os.makedirs(o), o)[1]
    try:
        yield
    finally:
        module.make_out = original


def main():
    if len(sys.argv) != 2 or os.path.exists(sys.argv[1]):
        print("usage: from the repository root, plan_binding_checks.py FRESH_SCRATCH (must not exist)", file=sys.stderr)
        return 2
    scratch = Path(sys.argv[1])
    scratch.mkdir(parents=True)
    results = []

    def check(name, ok, detail=""):
        results.append(bool(ok))
        for path in sorted({str(scratch.resolve()), str(scratch)}, key=len, reverse=True):
            detail = detail.replace(path, "SCRATCH")
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {detail}" if detail else ""))

    # ------------------------------------------------------------------ P01 sealed inputs
    u_sum, u_n, u_bad = verify_sums(UNDER)
    l_sum, l_n, l_bad = verify_sums(L1)
    check("P01-sealed-inputs", u_sum == UNDER_SEAL and u_n == 14 and not u_bad and l_sum == L1_SEAL and l_n == 7 and not l_bad
          and sha_file(REPO / L1 / "staged/prepare_audio.py") == L1_HELPER_SHA,
          f"Bronze's stage09 bundle {u_sum[:12]} ({u_n} files verify, differing {u_bad}); the L1 fix bundle {l_sum[:12]} "
          f"({l_n} files verify, differing {l_bad}); this delta's base is the L1 staged helper {L1_HELPER_SHA[:12]}")

    # ------------------------------------------------------------------ P02 the delta is narrow and chains onto L1
    l1_text = (REPO / L1 / "staged/prepare_audio.py").read_text(encoding="utf-8")
    staged_text = (REPO / STAGED).read_text(encoding="utf-8")
    remade = "".join(difflib.unified_diff(l1_text.splitlines(keepends=True), staged_text.splitlines(keepends=True),
                                          "a/prepare_audio.py", "b/prepare_audio.py"))
    chain = scratch / "chain"
    chain.mkdir()
    shutil.copyfile(REPO / UNDER / "prepare_audio.py", chain / "prepare_audio.py")
    codes = [subprocess.run(["patch", "-p1", "-d", str(chain), "-i", str(REPO / p)], capture_output=True, text=True).returncode
             for p in (f"{L1}/prepare_audio.patch", PATCH)]
    ldefs, lrest = top_level(l1_text)
    sdefs, srest = top_level(staged_text)
    changed = sorted(n for n in ldefs if sdefs.get(n) != ldefs[n])
    per_fn = {}
    for n in changed:
        lines = [x for x in difflib.unified_diff(ldefs[n].splitlines(), sdefs[n].splitlines(), lineterm="", n=0)
                 if x[:1] in "+-" and not x.startswith(("+++", "---"))]
        per_fn[n] = f"+{sum(x[0] == '+' for x in lines)}/-{sum(x[0] == '-' for x in lines)}"
    check("P02-narrow-delta", remade == (REPO / PATCH).read_text(encoding="utf-8") and codes == [0, 0]
          and sha_file(chain / "prepare_audio.py") == sha_file(REPO / STAGED) and list(sdefs) == list(ldefs)
          and changed == ["cmd_prepare", "cmd_timings"] and per_fn["cmd_prepare"] == "+1/-1" and lrest == srest,
          f"prepare_audio.patch is the diff L1 staged -> staged; `patch -p1` of the L1 patch then this one turns Bronze's "
          f"helper into the staged file ({sha_file(REPO / STAGED)[:12]}); changed {per_fn}; no function added or removed, "
          f"all other top-level code unchanged")

    l1 = load("prepare_audio_l1", REPO / L1 / "staged/prepare_audio.py")
    staged = load("prepare_audio_plan_binding", REPO / STAGED)
    b = staged.load_binding(BINDING)
    script = staged.read_script(b["script"])

    # ------------------------------------------------------------------ P03 positive: the real prepare -> timings shape
    _files, questions, recordings, inputs = staged.build_report(b)
    clar_dir = b["inventories"][1]["takes_dir"]
    first = {}
    for e in recordings:
        first.setdefault(e["cue"], e)
    ch08 = [c for c in script if c.startswith("08_")]
    takes = {c: (f"{clar_dir}/{c}-download01.wav" if c in CHANGED else first[c]["file"]) for c in ch08}
    selection = scratch / "selection-08.json"
    selection.write_text(json.dumps({"schema": "audio-preparation-selection-v1", "binding_sha256": b["sha256"],
                                     "script_sha256": b["pins"][b["script"]], "takes": takes, **FIXTURE}), encoding="utf-8")
    _sel, chosen = staged.load_selection(b, str(selection), inputs, "08")
    dec = scratch / "decoded"
    dec.mkdir()
    rows = []
    for cue, t in chosen.items():
        n = (t["frames"] + 1) // 2
        with wave.open(str(dec / f"{cue}.wav"), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(24000)
            w.writeframes(b"\0\0" * n)
        rows.append({"cue": cue, "text": script[cue], "source": t["rel"], "source_sha256": t["sha256"], "source_pcm_sha256": t["pcm"],
                     "source_frames": t["frames"], "source_rate": t["rate"], "downloads_receipt": t.get("receipt"),
                     "wav": str(dec / f"{cue}.wav"), "wav_sha256": sha_file(dec / f"{cue}.wav"), "samples": n, "rate": 24000})
    receipt = {"source": b["voice"]["receipt_source"], "voice": b["voice"]["name"], "provider": b["voice"]["provider"],
               "production": True, "binding": staged.binding_summary(b), "tool": "TEST FIXTURE (no decode ran)",
               "conversion": "TEST FIXTURE: zero-filled stand-ins of the decoded length", "ffmpeg": {"version": "none: TEST FIXTURE",
               "arguments": []}, "selection_sha256": sha_file(selection), "chapter": "08", "records": rows,
               "claim": "TEST FIXTURE: zero-filled stand-ins, not a decode; LavenderPike plan-binding check"}
    (dec / "decode-receipt.json").write_text(json.dumps(receipt, indent=1), encoding="utf-8")
    decode_fn = next(n for n in ast.parse(staged_text).body if isinstance(n, ast.FunctionDef) and n.name == "cmd_decode")
    keysets = [{k.value for k in d.keys if isinstance(k, ast.Constant)} for d in ast.walk(decode_fn) if isinstance(d, ast.Dict)]
    shape_ok = set(rows[0]) == next(k for k in keysets if "wav_sha256" in k) and set(receipt) == next(k for k in keysets if "records" in k)
    prepared = scratch / "prepared"
    with scratch_out(staged), contextlib.redirect_stdout(io.StringIO()):
        code = staged.cmd_prepare(b, types.SimpleNamespace(selection=str(selection), answers=None, decoded=str(dec),
                                                           reviewer="LavenderPike TEST", out=str(prepared), chapter="08", test_only=False))
    pr = json.loads((prepared / "prepare-receipt.json").read_text())
    plan_doc = json.loads((prepared / "phrase-marks-plan.json").read_text())
    argv = json.loads((prepared / "assemble-command.json").read_text())["argv"]
    r = subprocess.run([sys.executable, *argv[1:]], cwd=REPO, capture_output=True, text=True)
    narration = prepared / "narration"

    def timings(module, folder, tag):
        out = scratch / f"timings-{tag}"
        try:
            with scratch_out(module), contextlib.redirect_stdout(io.StringIO()):
                module.cmd_timings(b, types.SimpleNamespace(prepared=str(folder), narration=str(narration), out=str(out), test_only=False))
            written = json.loads((out / "phrase-timings-fragment-08.json").read_text())
            return "written " + json.dumps({c: [(m["byte"], m["seconds"]) for m in e["marks"]] for c, e in written["cues"].items() if e["marks"]})
        except module.Refused as error:
            return "refused: " + str(error) + ("" if not out.exists() else " (OUT created)")
        except OSError as error:
            return f"raised {type(error).__name__}"

    plan_marks = {c: [(m["byte"], m["seconds"]) for m in ms] for c, ms in plan_doc["cues"].items() if ms}
    good = timings(staged, prepared, "staged-good")
    check("P03-positive-real-shape", shape_ok and code == 0 and r.returncode == 0
          and pr["phrase_marks_plan_sha256"] == sha_file(prepared / "phrase-marks-plan.json")
          and good == "written " + json.dumps(plan_marks) and plan_marks == {"08_5": [(267, 22.42), (306, 25.379999)]},
          f"staged cmd_prepare on the real binding for chapter 08 (fixture selection, 08_5 by its clarity path, recognizer "
          f"values, decode rows with exactly cmd_decode's keys) writes phrase_marks_plan_sha256 = the plan file's hash "
          f"({pr['phrase_marks_plan_sha256'][:12]}); prepare's own assemble command (pinned assembler copy, "
          f"--require-directions) exits {r.returncode}; staged timings then accepts the bound plan and writes {good[8:]}"
          + ("" if r.returncode == 0 else f"; assemble: {(r.stderr or r.stdout).strip()[-300:]}"))

    def variant(tag, plan=None, drop_plan=False, receipt_edit=None):
        folder = scratch / f"prepared-{tag}"
        shutil.copytree(prepared, folder, ignore=shutil.ignore_patterns("narration", *(["phrase-marks-plan.json"] if drop_plan else [])))
        if plan is not None:
            (folder / "phrase-marks-plan.json").write_bytes(plan)
        if receipt_edit is not None:
            doc = json.loads((folder / "prepare-receipt.json").read_text())
            receipt_edit(doc, folder)
            (folder / "prepare-receipt.json").write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
        return folder

    def moved_plan(delta):
        doc = json.loads((prepared / "phrase-marks-plan.json").read_text())
        doc["cues"]["08_5"][0]["seconds"] = round(doc["cues"]["08_5"][0]["seconds"] + delta, 6)
        return (json.dumps(doc, indent=1, ensure_ascii=False) + "\n").encode()

    # ------------------------------------------------------------------ P04 a mark moved within its own sentence
    ends = json.loads((prepared / "spans.json").read_text())["units"][ch08.index("08_5")]["sentence_ends"]
    onsets = [x / 24000 for x in [0] + ends[:-1]]
    inside = staged.mark_outside_sentence(script["08_5"], onsets, ends[-1] / 24000, 267, round(22.42 + 0.05, 6)) is None
    within = variant("within", plan=moved_plan(0.05))
    l1_within, staged_within = timings(l1, within, "l1-within"), timings(staged, within, "staged-within")
    check("P04-within-sentence-mutation", inside and l1_within.startswith("written ") and '[267, 22.47]' in l1_within
          and staged_within.startswith("refused: " + BINDING_REFUSAL) and "(OUT created)" not in staged_within,
          f"08_5's 'leaves a half turn' mark moved +0.05 s in the plan, still inside sentence 4 (the L1 interval check "
          f"passes it): the L1 helper writes it ({l1_within[8:]}); the staged helper refuses before writing anything: "
          f"{staged_within[9:]!r}")

    # ------------------------------------------------------------------ P05 missing plan, unbound receipt, any changed byte
    missing = variant("missing", drop_plan=True)
    unbound = variant("unbound", receipt_edit=lambda doc, _f: doc.pop("phrase_marks_plan_sha256"))
    spaced = variant("spaced", plan=(prepared / "phrase-marks-plan.json").read_bytes() + b" ")
    outside_plan = moved_plan(-(22.42 - onsets[4]) - 0.1)
    rehashed = variant("rehashed", plan=outside_plan,
                       receipt_edit=lambda doc, _f: doc.update(phrase_marks_plan_sha256=sha(outside_plan)))
    got = {"missing (staged)": timings(staged, missing, "staged-missing"), "missing (L1)": timings(l1, missing, "l1-missing"),
           "receipt without the hash": timings(staged, unbound, "staged-unbound"),
           "plan with one added space, same marks": timings(staged, spaced, "staged-spaced"),
           "mark moved out of its sentence, receipt re-hashed": timings(staged, rehashed, "staged-rehashed")}
    check("P05-missing-and-mismatch", got["missing (staged)"] == "refused: phrase-marks-plan.json is missing from the prepared folder"
          and got["missing (L1)"] == "raised FileNotFoundError"
          and got["receipt without the hash"].startswith("refused: " + BINDING_REFUSAL)
          and got["plan with one added space, same marks"].startswith("refused: " + BINDING_REFUSAL)
          and got["mark moved out of its sentence, receipt re-hashed"].startswith("refused: 08_5: the mark at byte 267")
          and not any("(OUT created)" in v for v in got.values()),
          "; ".join(f"{k} -> {v if len(v) < 90 else v[:90] + '...'}" for k, v in got.items())
          + "; the last shows the L1 interval check still stands behind the binding")

    # ------------------------------------------------------------------ P06 Bronze's sealed suite, unchanged, on the staged helper
    wrapper = load("l1_bronze_wrapper", REPO / L1 / "checks/bronze_suite_on_staged.py")
    wrapper.STAGED = STAGED
    argv_saved, sys.argv = sys.argv, ["bronze_suite_on_staged.py", str(scratch / "bronze-suite")]
    buf = io.StringIO()
    try:
        with contextlib.redirect_stdout(buf):
            bronze_code = wrapper.main()
    finally:
        sys.argv = argv_saved
    lines = buf.getvalue().splitlines()
    sealed = (REPO / UNDER / "checks/outputs/run_checks-01.txt").read_text(encoding="utf-8").splitlines()
    check("P06-bronze-suite", bronze_code == 0 and lines[1] == f"prepare_audio.py {sha_file(REPO / STAGED)}" and lines[2:] == sealed[1:]
          and sealed[-1] == "79 PASS, 0 FAIL",
          f"Bronze's sealed run_checks.py through the sealed L1 wrapper, HELPER -> the staged file: {lines[-1]}; the "
          f"transcript equals Bronze's sealed run_checks-01.txt after the helper hash line (its full-film prepare with "
          f"answers then timings, chapters 01 and 04, and every plant now pass through the plan binding)")

    print(f"PLAN-BINDING: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
