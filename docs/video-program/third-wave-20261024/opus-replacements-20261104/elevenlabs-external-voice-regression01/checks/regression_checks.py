"""Replay for P/elevenlabs-external-voice-regression01 (WhiteSwan, root Mail 3917 and 3920).

Usage, from the repository root:
    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/elevenlabs-external-voice-regression01/checks/regression_checks.py NEW_SCRATCH_DIR

Writes only under NEW_SCRATCH_DIR, which must not exist. It reads, and edits none of:
- canonical tools/ (the installed voice repair and the two test files);
- root's preserved pre-install tool bytes (external-voice-integration01);
- the reviewed repair bundle, the review's proposed fix, and the sealed logarithm-clarity-integration01.

Every WAV the tests write is synthetic. No production audio is decoded; nothing is compiled, rendered,
synthesized, transcribed or heard. A PASS here says nothing about whose voice any real take holds.

Sections:
1. pins: exit 3 when a pinned input moved;
2. the diff rebuilds the patched test files from the canonical bases, and removes no line;
3. the suites on the installed tools: 44 original tests, then 44 + 8 new;
4. controls: the pre-install tools, the reviewed repair without the F2 fix, and 18 weakened copies of the
   installed tools; the patched suite must fail on each, and every copy the original 44 tests miss must
   fail a new test;
5. Mail 3920: the old tool pins of logarithm-clarity-integration01, qualified against the installed bytes.

The canonical test files may be at their base (diff not yet applied) or at the patched hashes (applied).
Exit 0 when every check holds, 1 otherwise, 2 for usage, 3 when a pinned input differs.
"""
import ast
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

REPO = Path.cwd()
THIS = Path(__file__).resolve()
HERE = THIS.parents[1]
P = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
DIFF = HERE / "patch/regression-tests.diff"
TESTS = ("test_coherent_narration.py", "test_narration_delivery.py")
TOOLS3 = ("coherent_narration.py", "narration_cache.py", "narration_delivery.py")
BASE_TESTS = {
    "test_coherent_narration.py": "074bf1b2e5ce007e702bff3166eb3e6aea006f2f73a9b865740d74a833dfba01",
    "test_narration_delivery.py": "bd45cb3d0285f287e6acaf71b0dda54cea6974b3db245c03a7bcba0830baf3fe",
}
PATCHED_TESTS = {
    "test_coherent_narration.py": "19487bd0ed812af7a2100b77e325af4cd901217d22da9e8e3075329a4372bc3e",
    "test_narration_delivery.py": "cd4d8dfa6947a5e2ea94d4d5ac93ae62cf12ec8401a98819a371330735ef887f",
}
INSTALLED = {  # root Mail 3920 and receipt01.json
    "coherent_narration.py": "95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5",
    "narration_cache.py": "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0",
    "narration_delivery.py": "961f17cc85dd30eadca73f950ec0bab6e52eb47ad43a7384c3710c97962b1878",
}
PRESERVED = Path("narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/external-voice-integration01")
BEFORE = {  # root's preserved pre-install bytes
    "coherent_narration.py": "e0aa81f352cb2c847f53dacf1d37b7d1b6a44526102df7f79329278c278c48d0",
    "narration_delivery.py": "c91914b89bed9e7aca71a649d403fa591bf40ec135f9c3d30f9d5e45c88422ba",
}
ROOT_RECEIPT = "fb72d39f7c62b91fa5c69066f06e3b68c5a3fd3ad60e46023426ddaaed053457"
REPAIR = P / "elevenlabs-external-voice-repair01"
REPAIR_SUMS = "3df152e499bd6cdc75144495b3d4733168b14bdd14aab56b8eda01d38c45c0b3"
REPAIR_DIFFS = ("coherent_narration.diff", "narration_delivery.diff")
REPAIR_COHERENT = "18c4e75e740943463a7ccecd22f7aaf01e08e19b45c30d0960d6a5e12490ba7d"
REVIEW = P / "elevenlabs-external-voice-review01"
REVIEW_SUMS = "0da3fa5095e0b60d48f1272fee772a11f96b7ab3d14cbfc9af974abc72df4068"
FIX = REVIEW / "proposed/legacy-voice-contradiction.diff"
CLARITY = P / "logarithm-clarity-integration01"
CLARITY_SUMS = "5fcb5ea628feb09888f5576cc78c4a05cb745180f68f7361cc6f601173cd2a7d"
STAGE08 = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage08/candidate")
REL = "explainer_corpus/logarithm_mysteries"

NEW = [
    "test_coherent_narration.CoherentTests.test_frederick_receipt_binds_provider_voice_and_request",
    "test_coherent_narration.CoherentTests.test_frederick_ui_takes_assemble_and_verify_sample_for_sample",
    "test_coherent_narration.CoherentTests.test_legacy_and_v2_oliver_receipts_assemble_as_one_voice",
    "test_coherent_narration.CoherentTests.test_legacy_oliver_receipt_cannot_state_another_voice_or_request",
    "test_coherent_narration.CoherentTests.test_one_assembly_refuses_two_narrator_voices",
    "test_coherent_narration.CoherentTests.test_verify_refuses_a_voice_swapped_in_after_assembly",
    "test_narration_delivery.DeliveryTests.test_frederick_and_legacy_oliver_ui_assemblies_reach_render",
    "test_narration_delivery.DeliveryTests.test_render_refuses_swapped_voice_and_legacy_contradiction_cleanly",
]
WITHOUT_F2 = [  # the new tests that need the F2 fix
    "test_coherent_narration.CoherentTests.test_legacy_oliver_receipt_cannot_state_another_voice_or_request",
    "test_narration_delivery.DeliveryTests.test_render_refuses_swapped_voice_and_legacy_contradiction_cleanly",
]
# Weakened copies of the installed tools: (label, file, old text, new text). Old text must occur once.
# None means: take that file from root's preserved pre-install bytes.
MUTANTS = [
    ("M1 no one-voice guard in verify", "coherent_narration.py",
     "        assembly_voice(records)\n        contract = direction_contract(", "        contract = direction_contract("),
    ("M2 no one-voice guard in assemble", "coherent_narration.py",
     "    assembly_voice(records)\n    directions=", "    directions="),
    ("M3 guard folds every UI voice into one", "coherent_narration.py",
     "voices.add(NATIVE_VOICE if voice is None else voice)", "voices.add(voice is None)"),
    ("M4 Frederick label missing from the table", "coherent_narration.py",
     "    'ElevenLabs UI, Frederick Surrey': 'Frederick Surrey - Smooth and Velvety',\n", ""),
    ("M5 provider not checked", "coherent_narration.py", "receipt.get('provider') != 'ElevenLabs' or ", ""),
    ("M6 record voice not required", "coherent_narration.py",
     "any(not isinstance(row, dict) or row.get('voice') != voice for row in records)",
     "any(not isinstance(row, dict) for row in records)"),
    ("M7 v2 record keys not required", "coherent_narration.py",
     "EXTERNAL_RECORD_KEYS + (EXTERNAL_V2_RECORD_KEYS if v2 else ())", "EXTERNAL_RECORD_KEYS"),
    ("M8 request digest not checked", "coherent_narration.py",
     "            if cache.digest(row['request']) != row['request_sha256']:\n", "            if False:\n"),
    ("M9 request cue not compared", "coherent_narration.py", "request.get('cue') != row['cue']", "False"),
    ("M10 request text not compared", "coherent_narration.py", "or request.get('text') != row['text'] ", ""),
    ("M11 request voice not compared", "coherent_narration.py", " or request.get('voice') != voice", ""),
    ("M12 native-key check removed", "coherent_narration.py",
     "        if any(key not in receipt for key in NATIVE_RECEIPT_KEYS):\n"
     "            raise ValueError('generation receipt is neither native FrankenTTS nor an identified external UI receipt')\n", ""),
    ("M13 legacy check ignores record voices", "coherent_narration.py",
     "[receipt.get('voice')] + [row.get('voice') for row in rows]", "[receipt.get('voice')]"),
    ("M14 legacy check allows a request", "coherent_narration.py",
     "\n                or any('request' in row or 'request_sha256' in row for row in rows))", ")"),
    ("M15 legacy check refuses the short name Oliver", "coherent_narration.py",
     "LEGACY_OLIVER_VOICES = ('Oliver', 'Oliver - Clean, British and Steady')",
     "LEGACY_OLIVER_VOICES = ('Oliver - Clean, British and Steady',)"),
    ("M16 delivery accepts any receipt", "narration_delivery.py",
     "require(coherent.external_ui_voice(receipt) is not None,", "require(True,"),
    ("M17 delivery diff not applied", "narration_delivery.py", None, None),
    ("M18 only the delivery diff applied", "coherent_narration.py", None, None),
]

failures = []
SCRATCH = None


def say(line):
    print(line.replace(str(SCRATCH), "SCRATCH").replace(str(REPO), "REPO"), flush=True)


def check(ok, what):
    if not ok:
        failures.append(what)
    say(("PASS " if ok else "FAIL ") + what)


def section(title):
    say(f"== {title}")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, cwd, tmp, timeout=1800):
    tmp.mkdir(parents=True, exist_ok=True)
    return subprocess.run(cmd, cwd=cwd, env=dict(os.environ, TMPDIR=str(tmp)), capture_output=True, text=True, timeout=timeout)


def suite_mode(tools, out):
    """Child process: run every test_*.py in tools and write {test id: ok|fail|error|skip}."""
    import unittest
    outcome = {}

    class Collect(unittest.TestResult):
        def addSuccess(self, test):
            outcome.setdefault(test.id(), "ok")

        def addFailure(self, test, err):
            outcome[test.id()] = "fail"

        def addError(self, test, err):
            outcome[test.id()] = "error"

        def addSubTest(self, test, subtest, err):
            if err is not None and outcome.get(test.id()) != "error":
                outcome[test.id()] = "fail" if issubclass(err[0], test.failureException) else "error"

        def addSkip(self, test, reason):
            outcome[test.id()] = "skip"

    suite = unittest.defaultTestLoader.discover(str(tools), pattern="test_*.py", top_level_dir=str(tools))
    suite.run(Collect())
    Path(out).write_text(json.dumps(outcome, indent=1, sort_keys=True))
    return 0


def suite(label, tools, tests):
    """Copy tools + tests into SCRATCH/suites/label/tools, run them in a child, return outcomes."""
    folder = SCRATCH / "suites" / label / "tools"
    folder.mkdir(parents=True)
    for name, path in list(tools.items()) + list(tests.items()):
        shutil.copyfile(path, folder / name)
    out = folder.parent / "outcomes.json"
    result = run([sys.executable, "-I", "-B", str(THIS), "--suite", str(folder), str(out)], folder.parent, folder.parent / "tmp")
    if result.returncode != 0 or not out.exists():
        return {"<runner>": "error: " + (result.stderr.strip().splitlines() or ["no output"])[-1]}
    return json.loads(out.read_text())


def not_ok(outcomes, names=None):
    return sorted(n for n, v in outcomes.items() if v != "ok" and (names is None or n in names))


def short(names):
    return ", ".join(n.rsplit(".", 1)[-1] for n in names) or "none"


def git_apply(diffs, folder, reverse=False):
    for diff in diffs:
        result = subprocess.run(["git", "apply", *(["-R"] if reverse else []), str(REPO / diff)],
                                cwd=folder, capture_output=True, text=True)
        if result.returncode:
            return f"{Path(diff).name}: {result.stderr.strip()}"
    return None


def apply_into(label, sources, diffs, reverse=False):
    """Copy sources (name -> path) into SCRATCH/trees/label/tools and git apply the diffs there."""
    tree = SCRATCH / "trees" / label
    (tree / "tools").mkdir(parents=True)
    for name, path in sources.items():
        shutil.copyfile(path, tree / "tools" / name)
    subprocess.run(["git", "init", "-q", str(tree)], check=True)
    error = git_apply(diffs, tree, reverse)
    return tree / "tools", error


def removed_lines(diff):
    return sum(1 for line in Path(diff).read_text().splitlines() if line.startswith("-") and not line.startswith("---"))


def top_level(path):
    """Top-level definitions by name (functions, classes, assignments) plus one entry for every other statement."""
    defs, other = {}, []
    for node in ast.parse(Path(path).read_text()).body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            defs[node.name] = node
        elif isinstance(node, ast.Assign) and all(isinstance(t, ast.Name) for t in node.targets):
            for target in node.targets:
                defs[target.id] = node
        else:
            other.append(node)
    defs["<imports and other statements>"] = ast.Module(body=other, type_ignores=[])
    return defs


def changed_names(old, new):
    a, b = top_level(old), top_level(new)
    return sorted(n for n in set(a) | set(b) if (ast.dump(a[n]) if n in a else None) != (ast.dump(b[n]) if n in b else None))


def reach(defs, roots, module_alias=None):
    """Names reachable from roots through Name loads; with module_alias, also collect alias.attr uses."""
    seen, todo, attrs = set(), list(roots), set()
    while todo:
        name = todo.pop()
        if name in seen or name not in defs:
            continue
        seen.add(name)
        for node in ast.walk(defs[name]):
            if isinstance(node, ast.Name) and node.id in defs:
                todo.append(node.id)
            if (module_alias and isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name)
                    and node.value.id == module_alias):
                attrs.add(node.attr)
    return seen, attrs


def enclosing(path, line):
    for node in ast.parse(Path(path).read_text()).body:
        if node.lineno <= line <= node.end_lineno:
            return getattr(node, "name", type(node).__name__)
    return None


CONSUMER = r'''
import importlib.util, json, sys
from pathlib import Path
spec = importlib.util.spec_from_file_location("coherent_narration", sys.argv[1])
cn = importlib.util.module_from_spec(spec); spec.loader.exec_module(cn)
cn.REPO = Path(sys.argv[2]).resolve()
script, units_path, disp_path = sys.argv[3], sys.argv[4], sys.argv[5]
try:
    _, units = cn.read_units(script, units_path, require_directions=True)
    contract = cn.direction_contract(script, json.loads(Path(units_path).read_text()))
    cn.review_directions(contract, units, json.loads(Path(disp_path).read_text()))
    print("ACCEPTED", len(units), len(contract["resolved"]))
except Exception as e:
    print("REFUSED", type(e).__name__, e)
'''


def main():
    global SCRATCH
    if len(sys.argv) == 4 and sys.argv[1] == "--suite":
        return suite_mode(Path(sys.argv[2]), Path(sys.argv[3]))
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    SCRATCH = Path(sys.argv[1]).resolve()
    if SCRATCH.exists() or not (REPO / "tools/coherent_narration.py").is_file() or not (REPO / P).is_dir():
        print("usage: run from the repository root and give a new scratch directory", file=sys.stderr)
        return 2

    # ------------------------------------------------------------------ 1
    section("pins")
    tools = REPO / "tools"
    moved = [n for n, h in INSTALLED.items() if sha(tools / n) != h]
    check(not moved, "canonical tools/ holds the installed repair: coherent 95615cfa, delivery 961f17cc, cache 47331efd"
          + (f" (differs: {moved})" if moved else ""))
    state = ("base" if all(sha(tools / n) == h for n, h in BASE_TESTS.items()) else
             "patched" if all(sha(tools / n) == h for n, h in PATCHED_TESTS.items()) else None)
    check(state is not None, f"canonical test files are at {state or 'neither'}: "
          + ", ".join(f"{n} {sha(tools / n)[:8]}" for n in TESTS))
    before = {n: REPO / PRESERVED / f"before-{n}" for n in BEFORE}
    stale = [n for n, h in BEFORE.items() if sha(before[n]) != h]
    receipt_ok = sha(REPO / PRESERVED / "receipt01.json") == ROOT_RECEIPT
    check(not stale and receipt_ok, "root's preserved pre-install bytes: coherent e0aa81f3, delivery c91914b8; receipt01.json fb72d39f")
    seals = [(REPAIR, REPAIR_SUMS), (REVIEW, REVIEW_SUMS), (CLARITY, CLARITY_SUMS)]
    bad_seals = []
    for folder, digest in seals:
        listed = [line.split("  ", 1) for line in (REPO / folder / "SHA256SUMS").read_text().splitlines()]
        if sha(REPO / folder / "SHA256SUMS") != digest or any(sha(REPO / folder / n) != h for h, n in listed):
            bad_seals.append(folder.name)
    check(not bad_seals, "sealed inputs verify: repair01 3df152e4, review01 0da3fa50, clarity-integration01 5fcb5ea6"
          + (f" (differs: {bad_seals})" if bad_seals else ""))
    if moved or state is None or stale or not receipt_ok or bad_seals:
        return 3
    SCRATCH.mkdir(parents=True)

    # ------------------------------------------------------------------ 2
    section("the diff")
    if state == "base":
        base_tests = {n: tools / n for n in TESTS}
    else:
        rebuilt, error = apply_into("base-tests", {n: tools / n for n in TESTS}, [DIFF.relative_to(REPO)], reverse=True)
        check(error is None, f"canonical tests are patched; git apply -R rebuilds the bases ({error or 'ok'})")
        base_tests = {n: rebuilt / n for n in TESTS}
    check(all(sha(base_tests[n]) == BASE_TESTS[n] for n in TESTS), "base test files: 074bf1b2 and bd45cb3d")
    patched, error = apply_into("patched-tests", base_tests, [DIFF.relative_to(REPO)])
    check(error is None, f"git apply regression-tests.diff onto the bases ({error or 'ok'})")
    check(all(sha(patched / n) == PATCHED_TESTS[n] for n in TESTS),
          "patched test files: " + ", ".join(f"{n} {sha(patched / n)[:8]}" for n in TESTS))
    check(removed_lines(DIFF) == 0, f"the diff removes no line, so all original tests are kept as they are ({removed_lines(DIFF)} removed)")
    live = subprocess.run(["git", "apply", "--check", *(["-R"] if state == "patched" else []), str(DIFF)], cwd=REPO,
                          capture_output=True, text=True)
    check(live.returncode == 0, f"git apply --check{' -R' if state == 'patched' else ''} against canonical tools/ (check only, nothing written)")
    for n in TESTS:
        try:
            compile((patched / n).read_text(), n, "exec")
            check(True, f"{n} (patched) compiles")
        except SyntaxError as error:
            check(False, f"{n} (patched) compiles: {error}")
    patched_tests = {n: patched / n for n in TESTS}

    # ------------------------------------------------------------------ 3
    section("suites on the installed tools")
    installed = {n: tools / n for n in TOOLS3}
    original_run = suite("installed-original", installed, base_tests)
    patched_run = suite("installed-patched", installed, patched_tests)
    original = sorted(original_run)
    check(len(original) == 44 and not not_ok(original_run), f"original suite: {len(original)} tests, not ok: {short(not_ok(original_run))}")
    check(sorted(set(patched_run) - set(original)) == NEW and set(original) <= set(patched_run),
          f"patched suite = the 44 original tests + these 8 new tests: {short(sorted(set(patched_run) - set(original)))}")
    check(len(patched_run) == 52 and not not_ok(patched_run), f"patched suite: {len(patched_run)} tests, not ok: {short(not_ok(patched_run))}")

    # ------------------------------------------------------------------ 4
    section("controls: the new tests must fail on the tools they guard against")
    applied_tools, error = apply_into("repair-without-f2", {"coherent_narration.py": before["coherent_narration.py"],
                                                           "narration_delivery.py": before["narration_delivery.py"],
                                                           "narration_cache.py": tools / "narration_cache.py"},
                                      [REPAIR / "patch" / d for d in REPAIR_DIFFS])
    check(error is None and sha(applied_tools / "coherent_narration.py") == REPAIR_COHERENT
          and sha(applied_tools / "narration_delivery.py") == INSTALLED["narration_delivery.py"],
          f"pre-install bytes + the repair's two diffs = the reviewed repair (coherent 18c4e75e, delivery 961f17cc) ({error or 'ok'})")
    rebuilt_tools, error = apply_into("repair-with-f2", {n: applied_tools / n for n in TOOLS3}, [FIX])
    check(error is None and all(sha(rebuilt_tools / n) == INSTALLED[n] for n in TOOLS3),
          f"... + the review's F2 fix = the installed tools byte for byte ({error or 'ok'})")
    controls = [("pre-install", {"coherent_narration.py": before["coherent_narration.py"],
                                 "narration_delivery.py": before["narration_delivery.py"],
                                 "narration_cache.py": tools / "narration_cache.py"}),
                ("repair-without-F2", {n: applied_tools / n for n in TOOLS3})]
    mutant_dir = SCRATCH / "mutants"
    for label, name, old, new in MUTANTS:
        variant = dict(installed)
        slug = re.sub(r"\W+", "-", label.split(" ", 1)[0])
        if old is None:
            variant[name] = before[name]
        else:
            text = (tools / name).read_text()
            check(text.count(old) == 1, f"{label}: the weakened text occurs once in the installed {name}")
            (mutant_dir / slug).mkdir(parents=True)
            (mutant_dir / slug / name).write_text(text.replace(old, new))
            variant[name] = mutant_dir / slug / name
        controls.append((label, variant))
    jobs = [(label, kind, variant, base_tests if kind == "original" else patched_tests)
            for label, variant in controls for kind in ("original", "patched")]
    with ThreadPoolExecutor(max_workers=max(1, min(6, (os.cpu_count() or 2) // 2))) as pool:
        results = list(pool.map(lambda job: suite(re.sub(r"\W+", "-", job[0]) + "-" + job[1], job[2], job[3]), jobs))
    outcomes = {(job[0], job[1]): result for job, result in zip(jobs, results)}
    pre_orig, pre_new = outcomes[("pre-install", "original")], outcomes[("pre-install", "patched")]
    check(not not_ok(pre_orig) and len(pre_orig) == 44, f"pre-install tools: the 44 original tests pass ({short(not_ok(pre_orig))} not ok)")
    check(not_ok(pre_new) == NEW, f"pre-install tools: all 8 new tests fail, no original test does ({len(not_ok(pre_new))} not ok)")
    rep_orig, rep_new = outcomes[("repair-without-F2", "original")], outcomes[("repair-without-F2", "patched")]
    check(not not_ok(rep_orig) and len(rep_orig) == 44, f"repair without the F2 fix: the 44 original tests pass ({short(not_ok(rep_orig))} not ok)")
    check(not_ok(rep_new) == WITHOUT_F2, f"repair without the F2 fix: exactly the 2 legacy-contradiction tests fail: {short(not_ok(rep_new))}")
    survivors = []
    for label, _, _, _ in MUTANTS:
        patched_failed = not_ok(outcomes[(label, "patched")])
        new_failed = not_ok(outcomes[(label, "patched")], NEW)
        orig_failed = not_ok(outcomes[(label, "original")])
        if not orig_failed:
            survivors.append(label.split(" ", 1)[0])
        # The patched suite is what canonical runs; it must refuse every weakened copy.
        check(bool(patched_failed) and (new_failed or orig_failed),
              f"{label}: patched suite fails {len(patched_failed)}; new tests failing: {short(new_failed)}; "
              f"original suite alone: {'fails ' + str(len(orig_failed)) if orig_failed else 'SURVIVES'}")
    check(all(not_ok(outcomes[(label, "patched")], NEW) for label, *_ in MUTANTS if label.split(" ", 1)[0] in survivors),
          f"every weakened copy the 44 original tests miss ({len(survivors)} of {len(MUTANTS)}: {', '.join(survivors)}) "
          "fails a new test")

    # ------------------------------------------------------------------ 5
    section("Mail 3920: qualify the old tool pins of logarithm-clarity-integration01 (sealed, not edited)")
    qualify_start = len(failures)
    pins = (REPO / CLARITY / "pins.sha256").read_text().splitlines()
    inputs = (REPO / CLARITY / "stage09/bound-plan/inputs.sha256").read_text().splitlines()
    rows = [line.split("  ", 1) for line in pins]
    drifted = sorted(n for h, n in rows if not (REPO / n).is_file() or sha(REPO / n) != h)
    check(pins == inputs, "pins.sha256 and stage09/bound-plan/inputs.sha256 are the same list")
    check(drifted == ["tools/coherent_narration.py"],
          f"of its {len(rows)} pinned files, only tools/coherent_narration.py moved: {drifted}")
    check(dict((n, h) for h, n in rows).get("tools/coherent_narration.py") == BEFORE["coherent_narration.py"],
          "the moved pin is root's preserved pre-install coherent e0aa81f3")
    old_c, new_c = before["coherent_narration.py"], tools / "coherent_narration.py"
    old_d, new_d = before["narration_delivery.py"], tools / "narration_delivery.py"
    changed_c, changed_d = changed_names(old_c, new_c), changed_names(old_d, new_d)
    say(f"     top-level names whose code differs, coherent: {', '.join(changed_c)}")
    say(f"     top-level names whose code differs, delivery: {', '.join(changed_d)}")
    helper = ast.parse((REPO / CLARITY / "integrate_clarity.py").read_text())
    calls = sorted({n.attr for n in ast.walk(helper) if isinstance(n, ast.Attribute)
                    and isinstance(n.value, ast.Name) and n.value.id == "coherent"})
    consumer = sorted(set(re.findall(r"\bcn\.(\w+)\(", (REPO / CLARITY / "checks/run_checks.py").read_text())))
    defs_c, defs_d = top_level(new_c), top_level(new_d)
    for what, roots in (("integrate_clarity.py calls coherent." + ", coherent.".join(calls), calls),
                        ("its run_checks consumer calls " + ", ".join(consumer), consumer)):
        seen, _ = reach(defs_c, roots)
        seen_old, _ = reach(top_level(old_c), roots)
        hit = sorted((seen | seen_old) & set(changed_c))
        check(seen == seen_old and not hit, f"{what}: they reach {len(seen)} coherent names, none changed ({short(hit)})")
    audit = json.loads((REPO / CLARITY / "stage09/bound-plan/hold-audit.json").read_text())["consumers"]
    for entry in (e for e in audit if e["path"] in ("tools/coherent_narration.py", "tools/narration_delivery.py")):
        name = Path(entry["path"]).name
        old_path, new_path = before[name], tools / name
        check(entry["sha256"] == BEFORE[name], f"hold-audit anchor {name}:{entry['line']} pins the pre-install bytes")
        old_lines = old_path.read_text().splitlines()
        new_lines = new_path.read_text().splitlines()
        hits = [i + 1 for i, line in enumerate(new_lines) if entry["marker"] in line]
        old_hits = [i + 1 for i, line in enumerate(old_lines) if entry["marker"] in line]
        func = enclosing(new_path, hits[0]) if len(hits) == 1 else None
        same = func is not None and func not in (changed_c if name.startswith("coherent") else changed_d)
        check(old_hits == [entry["line"]] and len(hits) == 1 and same,
              f"anchor {entry['marker'][:48]!r}: once in the installed {name}, line {entry['line']} -> {hits}, inside {func}, unchanged")
        if name == "narration_delivery.py":
            seen, attrs = reach(defs_d, [func], "coherent")
            reach_c, _ = reach(defs_c, sorted(attrs & set(defs_c)))
            say(f"     {func} reaches changed code: delivery {short(sorted(seen & set(changed_d)))}; "
                f"coherent {short(sorted(reach_c & set(changed_c)))}")
    # As in its run_checks.py: the sealed stage09 copied to root's install path; evidence paths are relative to it.
    inst = SCRATCH / "consumer"
    install = inst / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09"
    for part in ("candidate", "bound-plan"):
        shutil.copytree(REPO / CLARITY / "stage09" / part, install / part)
    lane9, disp = install / "candidate", install / "bound-plan/direction-dispositions-proposed.json"
    versions = {}
    for label, path in (("pre-install", old_c), ("installed", new_c)):
        # coherent loads narration_cache.py from its own folder.
        (inst / label).mkdir()
        shutil.copyfile(path, inst / label / "coherent_narration.py")
        shutil.copyfile(tools / "narration_cache.py", inst / label / "narration_cache.py")
        versions[label] = inst / label / "coherent_narration.py"
    sealed_out = (REPO / CLARITY / "checks/run_checks.out").read_text()
    for units, expect in ((lane9 / REL / "narration_units.json", "ACCEPTED 80 15"),
                          (REPO / STAGE08 / REL / "narration_units.json", "REFUSED")):
        got = []
        for coherent in versions.values():
            result = run([sys.executable, "-I", "-B", "-c", CONSUMER, str(coherent), str(lane9), str(lane9 / REL / "script.tsv"),
                          str(units), str(disp)], inst, SCRATCH / "consumer-tmp")
            got.append((result.stdout + result.stderr).strip())
        check(got[0] == got[1] and got[0].startswith(expect) and got[0] in sealed_out,
              f"consumer replay on the sealed stage09 ({'stage09' if expect.startswith('ACCEPTED') else 'stage08'} units): "
              f"pre-install and installed coherent print the same line as the sealed transcript: {got[1][:90]}")
    say("     conclusion: " + ("rebinding coherent e0aa81f3 -> 95615cfa changes no output of the helper or of its consumer check"
                              if len(failures) == qualify_start else "NOT qualified; see the FAIL lines above"))

    # ------------------------------------------------------------------ 6
    section("hygiene")
    authored = [p for p in sorted(HERE.rglob("*")) if p.is_file() and p.name != "SHA256SUMS" and not p.name.endswith(".out")]
    machine = re.compile(r"/Users/|/private/|/home/|/var/folders|jemanuel|scratchpad|claude-501|@gmail")  # hygiene-pattern
    pronoun = re.compile(r"\b(he|she|his|her|him|hers|himself|herself)\b", re.I)  # hygiene-pattern
    dirty = [str(p.relative_to(HERE)) for p in authored
             if any((machine.search(l) or pronoun.search(l)) and not l.endswith("# hygiene-pattern")
                    for l in p.read_text().splitlines())]
    check(not dirty, f"no machine path, user name, address or gendered pronoun in {len(authored)} authored files {dirty}")

    say(f"RESULT {'PASS' if not failures else 'FAIL'}, {len(failures)} failed")
    return 0 if not failures else 1


if __name__ == "__main__":
    sys.exit(main())
