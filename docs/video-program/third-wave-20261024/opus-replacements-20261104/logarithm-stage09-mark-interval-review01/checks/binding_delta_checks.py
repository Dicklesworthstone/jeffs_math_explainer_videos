#!/usr/bin/env python3
"""Independent checks of Lavender's plan-binding delta on top of the L1 fix (Mail 4094 / 4111, bead .63).

Run from the repository root, with a scratch folder that does not exist yet:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-mark-interval-review01/checks/binding_delta_checks.py NEW_SCRATCH

Read-only on the repository. It writes only small fixture folders under NEW_SCRATCH and leaves them
there. It imports the L1 staged helper and the combined (L1 + binding) staged helper side by side:
  B01  seals and pins;
  B02  the delta is exactly diff(L1 staged, combined staged), only in cmd_prepare and cmd_timings, and
       every L1 function reviewed in review_checks.py is byte-identical in the combined helper;
  B03  resolved_points of both helpers agree on all 162 real recordings (recognizer values and the
       timing inspection's leave_24k answers);
  B04  prepare writes the plan before the receipt and the receipt hashes that file (static);
  B05  cmd_timings in process on fixture prepared folders: which refusal comes first for a good plan,
       a missing plan, a receipt without the key, a plan changed by one mark within its sentence,
       a wrong binding and a changed spans file; and timings writes nothing in those folders.
The fixtures are labelled "review fixture"; nothing here is production, an answer or listening.
"""

import argparse
import difflib
import hashlib
import importlib.util
import json
import os
import re
import sys

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
L1 = f"{P}/logarithm-stage09-mark-interval-fix01"
DELTA = f"{P}/logarithm-stage09-plan-binding-fix01"
DELTA_SUMS = "5d2523af08db080e00baed5643a79cd25e8f7b7576878b284590080044e676b4"
L1_SUMS = "60f5f63ad958a1e41c3d6b668f1166e9599dc76977480616eb08c50ae5b115a7"
BASE_DIR = f"{P}/logarithm-stage09-audio-preparation01"
INSPECTION = f"{P}/logarithm-stage09-timing-inspection01/outputs/timing-inspection.json"
CRATE = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09/candidate/explainer_corpus/logarithm_mysteries"
PINS = {
    f"{L1}/staged/prepare_audio.py": "2f080ab7492638320697882bc6cc596e6fdb1330164df9cf980ba89ec9e2e73c",
    f"{DELTA}/staged/prepare_audio.py": "08a269279c47f3a3639ac2458b378135ecca620af9fd246d6b9291eaabd28b14",
    f"{DELTA}/prepare_audio.patch": "637788fb80f573685e744f3ad0c2053a4cda3ffbf592950706a0fb4c335f10d0",
    f"{L1}/SHA256SUMS": L1_SUMS,
    f"{DELTA}/SHA256SUMS": DELTA_SUMS,
    INSPECTION: "7d19519935c529227b42d2baf94f5614c841061526e4be7b2b9f8896b86a81de",
}
L1_FUNCTIONS = ["kit_sentences", "sentence_opens", "mark_outside_sentence", "resolved_points", "kit_mirror"]
FIXTURE = {"by": "review fixture", "basis": "review fixture"}
results = {"pass": 0, "fail": 0}


def sha_file(p):
    with open(p, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def report(name, ok, detail):
    results["pass" if ok else "fail"] += 1
    print(f"{name} {'PASS' if ok else 'FAIL'}: {detail}")


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def function_source(src, name):
    start = src.index(f"\ndef {name}(")
    end = src.find("\ndef ", start + 1)
    return src[start:end]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("scratch")
    scratch = ap.parse_args().scratch
    if os.path.exists(scratch):
        print("refused: the scratch folder must not exist yet")
        return 2

    # B01
    stale = [p for p, h in PINS.items() if sha_file(p) != h]
    for d in (L1, DELTA):
        listed = dict(reversed(ln.split("  ", 1)) for ln in open(f"{d}/SHA256SUMS", encoding="utf8").read().splitlines())
        found = {os.path.relpath(os.path.join(r, f), d) for r, _x, fs in os.walk(d) for f in fs} - {"SHA256SUMS"}
        stale += [f"{d}/{rel}" for rel, want in listed.items() if not os.path.isfile(f"{d}/{rel}") or sha_file(f"{d}/{rel}") != want]
        stale += [f"{d}/{rel} (unlisted)" for rel in sorted(found - set(listed))]
    if stale:
        print("STALE:\n" + "\n".join(stale))
        return 3
    report("B01", True, f"L1 {L1_SUMS[:8]} and binding delta {DELTA_SUMS[:8]} seals verify with no unlisted file; {len(PINS)} pins hold")

    l1 = load("review_l1_prepare_audio", f"{L1}/staged/prepare_audio.py")
    both = load("review_combined_prepare_audio", f"{DELTA}/staged/prepare_audio.py")
    src_l1 = open(f"{L1}/staged/prepare_audio.py", encoding="utf8").read()
    src_both = open(f"{DELTA}/staged/prepare_audio.py", encoding="utf8").read()

    # B02
    a, b = src_l1.splitlines(keepends=True), src_both.splitlines(keepends=True)
    diff = "".join(difflib.unified_diff(a, b, "a/prepare_audio.py", "b/prepare_audio.py", n=3))
    patch = open(f"{DELTA}/prepare_audio.patch", encoding="utf8").read()
    plus = sum(1 for ln in diff.splitlines() if ln.startswith("+") and not ln.startswith("+++"))
    minus = sum(1 for ln in diff.splitlines() if ln.startswith("-") and not ln.startswith("---"))
    hunks = []
    for h in re.finditer(r"^@@ -\d+(?:,\d+)? \+(\d+)", diff, re.M):
        line = int(h.group(1)) + 3
        hunks.append(next(re.match(r"def (\w+)", b[k]).group(1) for k in range(line, -1, -1) if b[k].startswith("def ")))
    same = {f: function_source(src_l1, f) == function_source(src_both, f) for f in L1_FUNCTIONS}
    report("B02", diff == patch and (plus, minus) == (8, 3) and hunks == ["cmd_prepare", "cmd_timings"] and all(same.values()),
           f"diff(L1 staged, combined) equals the delta patch: {diff == patch}; +{plus}/-{minus} (HANDOFF line 21 says +7/-3, its "
           f"P02 row +1/-1 and +7/-2); hunks in {hunks}; "
           f"L1 functions byte-identical: {same}")

    # B03
    script = l1.read_script(f"{CRATE}/script.tsv")
    entries = json.load(open(f"{BASE_DIR}/report/anchors.json"))["recordings"]
    insp = {r["id"]: r for r in json.load(open(INSPECTION))["rows"]}
    agree, problems = 0, []
    for view in ("recognizer", "leave_24k answers"):
        for e in entries:
            applied = {}
            if view != "recognizer":
                applied = {p["id"]: {"decision": "sample", **FIXTURE, "sample_24k": insp[p["id"]]["leave_24k"]}
                           for p in e["sentences"][1:] + e["phrase_anchors"] if insp.get(p.get("id"), {}).get("class") == "recognizer_early"}
            frames24 = (e["frames"] + 1) // 2
            try:
                x = l1.resolved_points(e, applied, frames24, script[e["cue"]])
                y = both.resolved_points(e, applied, frames24, script[e["cue"]])
            except (l1.Refused, both.Refused) as err:
                problems.append(f"{view} {e['label']}: {err}")
                continue
            agree += x == y
    report("B03", agree == 2 * len(entries) == 324 and not problems,
           f"{len(entries)} recordings x 2 views: combined resolved_points equals L1's in {agree}; refused {problems[:1]}")

    # B04
    prep = function_source(src_both, "cmd_prepare")
    i_plan = prep.index('write_new(f"{out}/phrase-marks-plan.json"')
    i_receipt = prep.index('write_new(f"{out}/prepare-receipt.json"')
    hashed = '"phrase_marks_plan_sha256": sha_file(f"{out}/phrase-marks-plan.json")' in prep[i_receipt:]
    pinned = '"prepare_receipt_sha256": sha_file(f"{prepared}/prepare-receipt.json")' in function_source(src_both, "cmd_timings")
    report("B04", i_plan < i_receipt and hashed and pinned,
           f"cmd_prepare writes phrase-marks-plan.json before prepare-receipt.json ({i_plan < i_receipt}) and the receipt hashes that "
           f"written file ({hashed}); timings output still pins the receipt by prepare_receipt_sha256 ({pinned})")

    # B05
    binding = {"use": "historical-test", "sha256": "f" * 64, "path": "review-fixture-binding.json", "voice": {"name": "review fixture"},
               "assembler": "A", "assembler_cache": "C", "map": "M", "map_in_tree": "map.json", "pins": {"A": "0" * 64, "C": "0" * 64, "M": "0" * 64}}
    spans = b'{"units": [], "note": "review fixture"}\n'
    plan = {"cues": {"08_5": [{"byte": 267, "seconds": 22.42, "basis": "review fixture"}, {"byte": 306, "seconds": 25.379999, "basis": "review fixture"}]},
            "note": "review fixture"}
    plan_bytes = (json.dumps(plan, indent=1) + "\n").encode()
    within = json.loads(plan_bytes)
    within["cues"]["08_5"][0]["seconds"] = 22.47
    cases = {
        "good plan": {},
        "plan missing": {"plan": None},
        "receipt without the plan key": {"drop_key": True},
        "plan mark moved 0.05 s within its sentence": {"plan": (json.dumps(within, indent=1) + "\n").encode()},
        "wrong binding and plan missing": {"binding": "e" * 64, "plan": None},
        "spans changed and plan missing": {"spans_after": spans + b" ", "plan": None},
    }

    def build(folder, c):
        os.makedirs(f"{folder}/workspace/tools")
        with open(f"{folder}/spans.json", "wb") as f:
            f.write(c.get("spans_after", spans))
        if c.get("plan", plan_bytes) is not None:
            with open(f"{folder}/phrase-marks-plan.json", "wb") as f:
                f.write(c.get("plan", plan_bytes))
        receipt = {"binding": {"sha256": c.get("binding", binding["sha256"])}, "production": False,
                   "spans_sha256": hashlib.sha256(spans).hexdigest(), "phrase_marks_plan_sha256": hashlib.sha256(plan_bytes).hexdigest(),
                   "claim": "review fixture"}
        if c.get("drop_key"):
            del receipt["phrase_marks_plan_sha256"]
        with open(f"{folder}/prepare-receipt.json", "w", encoding="utf8") as f:
            json.dump(receipt, f, indent=1)
        with open(f"{folder}/workspace/tools/coherent_narration.py", "w", encoding="utf8") as f:
            f.write("# review fixture, not the assembler\n")

    def listing(folder):
        return sorted((os.path.relpath(os.path.join(r, f), folder), sha_file(os.path.join(r, f))) for r, _x, fs in os.walk(folder) for f in fs)

    def outcome(mod, folder):
        args = argparse.Namespace(prepared=folder, narration=f"{folder}/narration", test_only=True)
        try:
            mod.cmd_timings(binding, args)
            return "ran"
        except mod.Refused as e:
            return "Refused: " + str(e).split(":")[0].split(";")[0]
        except Exception as e:  # noqa: BLE001 - the type is the result here
            return f"raised {type(e).__name__}"

    want = {
        "good plan": ("Refused: workspace copy tools/coherent_narration.py changed", "Refused: workspace copy tools/coherent_narration.py changed"),
        "plan missing": ("raised FileNotFoundError", "Refused: phrase-marks-plan.json is missing from the prepared folder"),
        "receipt without the plan key": ("Refused: workspace copy tools/coherent_narration.py changed",
                                         "Refused: phrase-marks-plan.json is not the plan this prepare receipt binds"),
        "plan mark moved 0.05 s within its sentence": ("Refused: workspace copy tools/coherent_narration.py changed",
                                                       "Refused: phrase-marks-plan.json is not the plan this prepare receipt binds"),
        "wrong binding and plan missing": ("raised FileNotFoundError", "Refused: the prepared folder was made from another binding"),
        "spans changed and plan missing": ("raised FileNotFoundError", "Refused: spans.json changed since prepare"),
    }
    lines, ok, untouched = [], True, True
    for k, (label, c) in enumerate(cases.items()):
        got = []
        for tag, mod in (("l1", l1), ("combined", both)):
            folder = os.path.join(scratch, f"case{k}-{tag}")
            build(folder, c)
            before = listing(folder)
            got.append(outcome(mod, folder))
            untouched &= listing(folder) == before
        ok &= tuple(got) == want[label]
        lines.append(f"[{label}: L1 {got[0]} | combined {got[1]}]")
    report("B05", ok and untouched, " ".join(lines) + f"; no file written or changed in any fixture folder: {untouched}")

    print(f"RESULT {'PASS' if results['fail'] == 0 else 'FAIL'}: {results['pass']} pass, {results['fail']} fail")
    return 0 if results["fail"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
