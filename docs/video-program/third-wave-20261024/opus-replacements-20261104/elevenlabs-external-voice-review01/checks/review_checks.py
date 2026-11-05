"""Independent review checks for P/elevenlabs-external-voice-repair01 (WhiteSwan, answering root Mail 3827).

Usage, from the repository root:
    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/elevenlabs-external-voice-review01/checks/review_checks.py NEW_SCRATCH_DIR

Writes only under NEW_SCRATCH_DIR, which must not exist. It reads:
- the sealed repair bundle;
- canonical tools/;
- retained receipt and span-manifest JSON.

No audio file is opened: the retained scan reads JSON only, and every WAV in the scenarios is synthetic.
The transcript shows the scratch root as SCRATCH.

Three tool variants run the same checks:
    base     canonical tools (pinned hashes)
    applied  base + the bundle's two diffs, applied here with git apply (the bundle used patch(1))
    fixed    applied + proposed/legacy-voice-contradiction.diff (this review's narrow fix for finding F2)

Lines tagged FINDING assert that the sealed repair still shows the reported defect, so PASS on them confirms the
finding. Exit 0 when every expectation holds, 1 otherwise, 2 for usage, 3 when a pinned input differs.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

REPO = Path.cwd()
HERE = Path(__file__).resolve().parents[1]
P = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
BUNDLE = P / "elevenlabs-external-voice-repair01"
BUNDLE_SUMS = "3df152e499bd6cdc75144495b3d4733168b14bdd14aab56b8eda01d38c45c0b3"
BASE_PINS = {  # recorded from canonical tools/ at the start of this review
    "coherent_narration.py": "e0aa81f352cb2c847f53dacf1d37b7d1b6a44526102df7f79329278c278c48d0",
    "narration_delivery.py": "c91914b89bed9e7aca71a649d403fa591bf40ec135f9c3d30f9d5e45c88422ba",
    "narration_cache.py": "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0",
    "test_coherent_narration.py": "074bf1b2e5ce007e702bff3166eb3e6aea006f2f73a9b865740d74a833dfba01",
    "test_narration_delivery.py": "bd45cb3d0285f287e6acaf71b0dda54cea6974b3db245c03a7bcba0830baf3fe",
}
PATCHED_CLAIMS = {"coherent_narration.py": "18c4e75e740943463a7ccecd22f7aaf01e08e19b45c30d0960d6a5e12490ba7d",
                  "narration_delivery.py": "961f17cc85dd30eadca73f950ec0bab6e52eb47ad43a7384c3710c97962b1878"}
DIFFS = ("coherent_narration.diff", "narration_delivery.diff")
FIX = HERE / "proposed/legacy-voice-contradiction.diff"
TOOLS3 = ("coherent_narration.py", "narration_cache.py", "narration_delivery.py")
OLIVER_LABEL = "ElevenLabs UI, Oliver"

ONE_VOICE = "refused ValueError: one assembly must use one narrator voice, not: "
FRED_OLIVER = "Frederick Surrey - Smooth and Velvety; Oliver - Clean, British and Steady"
REQUEST = "refused ValueError: external request does not name this cue, text and voice exactly$"
IDENTITY = "refused ValueError: external UI receipt provider or voice differs from its exact source label$"
RECORD_VOICE = "refused ValueError: every external UI record must state the receipt voice exactly$"
LABEL = "refused ValueError: external UI receipt names no exact known source label$"
UNKNOWN = "refused ValueError: unrecognised external UI receipt; a non-legacy label needs schema elevenlabs-ui-receipt-v2$"
CHANGED = "refused ValueError: external source, text or decoded audio changed$"
LEGACY_FIX = "refused ValueError: legacy Oliver receipt states another voice or a request; use schema elevenlabs-ui-receipt-v2$"
OK_PCM = r"ok pcm=exact$"
# Every scenario in scenarios.py, with what the applied repair must do. A third element names a finding.
APPLIED = {
    "fred_shared": (OK_PCM,), "fred_per_cue": (OK_PCM,), "oliver_legacy": (r"ok pcm=exact digest=[0-9a-f]{16}$",),
    "oliver_v2_with_legacy": (OK_PCM,), "native_only": (OK_PCM,), "extra_keys_ignored": (r"ok$",),
    "mix_fred_oliver": (re.escape(ONE_VOICE + FRED_OLIVER) + r" \[out absent\]$",),
    "mix_external_native": (re.escape(ONE_VOICE + "Oliver - Clean, British and Steady; native FrankenTTS receipt") + r" \[out absent\]$",),
    "swap_after_assembly": (r"assemble=ok; unit_spans_alone=ok; verify=" + re.escape(ONE_VOICE + FRED_OLIVER) + "$",),
    **{name: (REQUEST,) for name in ("request_voice_oliver", "request_voice_short", "request_text_neighbour",
                                     "request_text_one_word", "request_cue_neighbour", "request_of_neighbour_file",
                                     "request_voice_missing", "request_not_object")},
    "request_bytes_changed": (r"refused ValueError: external request file changed$",),
    "request_bare_name": (r"refused FileNotFoundError: ", "F3"),
    **{name: (IDENTITY,) for name in ("receipt_voice_short", "receipt_voice_oliver_on_fred_label", "provider_wrong", "provider_missing")},
    **{name: (RECORD_VOICE,) for name in ("record_voice_missing", "record_voice_case", "records_not_list")},
    **{name: (LABEL,) for name in ("label_near_miss", "label_trailing_space", "label_lowercase_v2", "label_as_list")},
    "label_lowercase_unversioned": (r"refused ValueError: generation receipt is neither native FrankenTTS nor an identified external UI receipt$",),
    **{name: (UNKNOWN,) for name in ("schema_unknown", "schema_unknown_on_oliver_label", "unversioned_frederick")},
    "receipt_is_list": (r"refused ValueError: generation receipt must be a JSON object$",),
    "record_missing_request_sha256": (r"refused ValueError: external receipt record lacks request_sha256$",),
    "record_missing_samples": (r"refused ValueError: external receipt record lacks samples$",),
    **{name: (CHANGED,) for name in ("wav_sha256_wrong", "samples_off_by_one", "rate_48000", "source_sha256_wrong",
                                     "source_changed", "wav_copy_at_other_path", "record_text_neighbour")},
    **{name: (r"refused ValueError: external receipt must identify exactly one source per cue$",)
       for name in ("record_duplicated", "record_for_other_cue_only")},
    "context_crop": (r"refused ValueError: external UI takes require one complete cue without context crop$",),
    "decode_shape_oliver": (r"ok$",),
    "decode_shape_frederick": (UNKNOWN, "F1"),
    "decode_shape_frederick_plus_v2_fields": (r"ok$",),
    "decode_shape_misbound": (r"ok$", "F2"),
    "misbound_mixed_with_oliver": (OK_PCM, "F2"),
    "v2_schema_stripped_under_oliver_label": (r"ok$", "F2"),
    "delivery_frederick": (r"exit 0: plan_mode=external-ui$",),
    "delivery_oliver_legacy": (r"exit 0: plan_mode=external-ui$",),
    "delivery_native_under_external_plan": (r"exit 1: narration delivery refused: external plan requires an explicitly identified UI source$",),
    "delivery_swapped_mix": (r"exit 1: narration delivery refused: one assembly must use one narrator voice",),
}
FIXED = dict(APPLIED, **{name: (LEGACY_FIX,) for name in
                         ("decode_shape_misbound", "misbound_mixed_with_oliver", "v2_schema_stripped_under_oliver_label")})
BASE = {  # what the canonical tools do, for contrast
    **{name: (r"refused KeyError: 'wav'$",) for name in ("fred_shared", "fred_per_cue", "decode_shape_frederick",
                                                         "decode_shape_frederick_plus_v2_fields", "unversioned_frederick")},
    "delivery_frederick": (r"assembly refused KeyError: 'wav'$",),
    "mix_external_native": (r"ok \[out exists\]$",),
    "receipt_is_list": (r"crash AttributeError: 'list' object has no attribute 'get'$",),
    "schema_unknown_on_oliver_label": (r"ok$",),
    **{name: APPLIED[name] for name in ("oliver_v2_with_legacy", "native_only", "decode_shape_oliver", "decode_shape_misbound",
                                        "misbound_mixed_with_oliver", "v2_schema_stripped_under_oliver_label",
                                        "delivery_oliver_legacy", "delivery_native_under_external_plan")},
}
# Weakened copies of the applied repair. Each must change at least one scenario outcome.
MUTANTS = [
    ("X1 no one-voice guard in verify", "coherent_narration.py",
     "        assembly_voice(records)\n        contract = direction_contract(", "        contract = direction_contract("),
    ("X2 no one-voice guard in assemble", "coherent_narration.py",
     "    assembly_voice(records)\n    directions=", "    directions="),
    ("X3 request digest not checked", "coherent_narration.py",
     "            if cache.digest(row['request']) != row['request_sha256']:\n", "            if False:\n"),
    ("X4 request text not compared", "coherent_narration.py", "or request.get('text') != row['text'] ", ""),
    ("X5 request cue not compared", "coherent_narration.py", "request.get('cue') != row['cue']", "False"),
    ("X6 provider not checked", "coherent_narration.py", "receipt.get('provider') != 'ElevenLabs' or ", ""),
    ("X7 request voice not compared", "coherent_narration.py", " or request.get('voice') != voice", ""),
    ("X8 native-key check removed", "coherent_narration.py",
     "        if any(key not in receipt for key in NATIVE_RECEIPT_KEYS):\n"
     "            raise ValueError('generation receipt is neither native FrankenTTS nor an identified external UI receipt')\n", ""),
    ("X9 delivery accepts any receipt", "narration_delivery.py",
     "require(coherent.external_ui_voice(receipt) is not None,", "require(True,"),
    ("X10 delivery diff not applied", "narration_delivery.py", None, "base"),
    ("X11 only the delivery diff applied", "coherent_narration.py", None, "base"),
]

failures = []
SCRATCH = None


def say(line):
    print(line.replace(str(SCRATCH), "SCRATCH"), flush=True)


def check(ok, what, tag=None):
    if not ok:
        failures.append(what)
    say(("PASS " if ok else "FAIL ") + (f"FINDING {tag}: " if tag else "") + what)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, cwd, timeout=900):
    env = dict(os.environ, TMPDIR=str(SCRATCH / "tmp"))
    return subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def section(title):
    say(f"== {title}")


def main():
    global SCRATCH
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    SCRATCH = Path(sys.argv[1]).resolve()
    if SCRATCH.exists():
        print("usage: give a new scratch directory", file=sys.stderr)
        return 2
    if not (REPO / BUNDLE).is_dir():
        print("usage: run from the repository root", file=sys.stderr)
        return 2

    section("seal and pins")
    sums = REPO / BUNDLE / "SHA256SUMS"
    listed = [line.split("  ", 1) for line in sums.read_text().splitlines()]
    stale = [name for digest, name in listed if sha(REPO / BUNDLE / name) != digest]
    actual = sorted(str(p.relative_to(REPO / BUNDLE)) for p in (REPO / BUNDLE).rglob("*") if p.is_file())
    check(sha(sums) == BUNDLE_SUMS and not stale and sorted([n for _, n in listed] + ["SHA256SUMS"]) == actual,
          f"repair bundle matches its seal {BUNDLE_SUMS[:8]}: {len(listed)} files listed, none changed, none unlisted")
    drift = [n for n, h in BASE_PINS.items() if sha(REPO / "tools" / n) != h]
    check(not drift, "canonical tools/ still has the five pinned base hashes" + (f" (differs: {drift})" if drift else ""))
    if stale or drift or sha(sums) != BUNDLE_SUMS:
        return 3

    section("apply with git apply (independent of the bundle's patch(1) route)")
    SCRATCH.mkdir(parents=True)
    (SCRATCH / "tmp").mkdir()
    mirrors = {}
    for label in ("base", "applied", "fixed"):
        (SCRATCH / label / "tools").mkdir(parents=True)
        for name in BASE_PINS:
            shutil.copyfile(REPO / "tools" / name, SCRATCH / label / "tools" / name)
        mirrors[label] = SCRATCH / label / "tools"
    for label, diffs in (("applied", [REPO / BUNDLE / "patch" / d for d in DIFFS]),
                         ("fixed", [REPO / BUNDLE / "patch" / d for d in DIFFS] + [FIX])):
        for diff in diffs:
            result = run(["git", "apply", "--whitespace=nowarn", str(diff)], SCRATCH / label)
            check(result.returncode == 0, f"{label}: git apply {diff.name}" + (f" ({result.stderr.strip()})" if result.returncode else ""))
    for name, claim in PATCHED_CLAIMS.items():
        got = mirrors["applied"] / name
        check(sha(got) == claim and got.read_bytes() == (REPO / BUNDLE / "patched/tools" / name).read_bytes(),
              f"base + diff = patched/tools/{name} byte for byte ({claim[:8]})")
    for diff, removed in (("coherent_narration.diff", 2), ("narration_delivery.diff", 1)):
        lines = (REPO / BUNDLE / "patch" / diff).read_text().splitlines()
        minus = [l for l in lines if l.startswith("-") and not l.startswith("---")]
        check(len(minus) == removed, f"{diff} removes exactly {removed} line(s)")
    fix_minus = [l for l in FIX.read_text().splitlines() if l.startswith("-") and not l.startswith("---")]
    check(not fix_minus, f"the proposed fix only adds lines ({sha(FIX)[:8]})")

    section("canonical tools/test_*.py suites")
    counts = {}
    for label, tools in mirrors.items():
        result = run([sys.executable, "-I", "-B", "-m", "unittest", "discover", "-s", str(tools), "-p", "test_*.py"], tools.parent)
        ran = re.search(r"^Ran (\d+) tests?", result.stderr, re.M)
        counts[label] = int(ran.group(1)) if ran else -1
        check(result.returncode == 0 and counts[label] > 0, f"{label}: {counts[label]} canonical tests pass")
    check(len(set(counts.values())) == 1, "the same canonical test count on all three variants")

    section("independent synthetic scenarios (checks/scenarios.py)")
    outcomes = {}

    def scenarios(label, files):
        repo = SCRATCH / f"scenarios-{label}"
        (repo / "tools").mkdir(parents=True)
        for name, source in files.items():
            shutil.copyfile(source, repo / "tools" / name)
        result = run([sys.executable, "-I", "-B", str(HERE / "checks/scenarios.py"), str(repo)], SCRATCH)
        if result.returncode != 0:
            say(f"FAIL scenarios.py on {label}: {result.stderr.strip().splitlines()[-1:]}")
            failures.append(f"scenarios {label}")
            return {}
        (SCRATCH / f"outcomes-{label}.json").write_text(result.stdout)
        return json.loads(result.stdout)

    for label in ("base", "applied", "fixed"):
        outcomes[label] = scenarios(label, {n: mirrors[label] / n for n in TOOLS3})
    for label, table in (("applied", APPLIED), ("fixed", FIXED)):
        got = outcomes[label]
        check(set(got) == set(table), f"{label}: every one of the {len(table)} scenarios has an expectation")
        for name in sorted(table):
            pattern, *tag = table[name]
            ok = name in got and re.match(pattern, got[name]) is not None
            if not ok or tag:
                check(ok, f"{label} {name}: {got.get(name)}", tag[0] if tag else None)
        plain = [n for n in table if len(table[n]) == 1]
        say(f"     {label}: {sum(re.match(table[n][0], got.get(n, '')) is not None for n in plain)} of {len(plain)} "
            "other expectations hold (listed only when they fail)")
    for name in sorted(BASE):
        got = outcomes["base"].get(name)
        check(got is not None and re.match(BASE[name][0], got) is not None, f"base {name}: {got}")
    legacy = {label: outcomes[label].get("oliver_legacy") for label in outcomes}
    check(len(set(legacy.values())) == 1, f"legacy Oliver assembly: base, applied and fixed outputs are identical apart from paths ({legacy['base']})")
    keyerror = sum(v == "refused KeyError: 'wav'" for v in outcomes["base"].values())
    say(f"     base: {keyerror} of {len(outcomes['base'])} scenarios end in KeyError 'wav' (the Frederick label falls into the native branch)")

    section("weakened copies of the repair (this review's mutants), against both suites")
    bundle_suite = REPO / BUNDLE / "tests/test_external_voice.py"
    applied_text = {n: (mirrors["applied"] / n).read_text() for n in TOOLS3}
    for label, name, old, new in MUTANTS:
        files = {n: mirrors["applied"] / n for n in TOOLS3}
        tools = SCRATCH / "mutants" / re.sub(r"\W+", "-", label) / "tools"
        tools.mkdir(parents=True)
        if old is None:
            files[name] = mirrors["base"] / name
        else:
            text = applied_text[name]
            check(text.count(old) == 1, f"{label}: the mutated site occurs once")
            (tools / name).write_text(text.replace(old, new))
            files[name] = tools / name
        for n in TOOLS3:
            if not (tools / n).exists():
                shutil.copyfile(files[n], tools / n)
        got = scenarios(re.sub(r"\W+", "-", label), {n: tools / n for n in TOOLS3})
        changed = sorted(n for n in got if got[n] != outcomes["applied"].get(n))
        theirs = run([sys.executable, "-I", "-B", str(REPO / bundle_suite), str(mirrors["base"]), str(tools)], SCRATCH)
        failed = sorted(set(re.findall(r"^(?:FAIL|ERROR): (\w+)", theirs.stderr, re.M)))
        check(bool(changed), f"{label}: killed by {len(changed)} scenarios ({', '.join(changed[:3])}{', ...' if len(changed) > 3 else ''}); "
              f"bundle suite: {'killed by ' + str(len(failed)) + ' tests' if theirs.returncode else 'SURVIVES'}")
        if label.startswith("X11"):
            say(f"     X11 delivery_oliver_legacy: {got.get('delivery_oliver_legacy')}")

    section("the bundle's own synthetic checks, replayed (frederick_rehearsal.py and retained_compat.py open production audio and are not run)")
    result = run([sys.executable, "-I", "-B", str(REPO / BUNDLE / "tests/run_tests.py"), "--work", str(SCRATCH / "bundle-run-tests")], REPO)
    check(result.returncode == 0 and result.stdout.strip().endswith("ALL PASS"), "tests/run_tests.py: " + result.stdout.strip().splitlines()[-1])
    result = run([sys.executable, "-I", "-B", str(REPO / BUNDLE / "checks/mutation_check.py"), "--work", str(SCRATCH / "bundle-mutation")], REPO)
    check(result.returncode == 0 and result.stdout.strip().endswith("ALL MUTANTS KILLED"), "checks/mutation_check.py: " + result.stdout.strip().splitlines()[-1])
    result = run([sys.executable, "-I", "-B", str(REPO / bundle_suite), str(mirrors["base"]), str(mirrors["fixed"])], SCRATCH)
    ran = re.search(r"^Ran (\d+) tests?", result.stderr, re.M)
    check(result.returncode == 0, f"the bundle's {ran.group(1) if ran else '?'} synthetic tests also pass with the proposed fix")

    section("retained receipts and span manifests (JSON metadata only)")
    variants = {label: load(mirrors[label] / "coherent_narration.py", f"coherent_{label}") for label in ("applied", "fixed")}

    def classify(module, receipt):
        try:
            voice = module.external_ui_voice(receipt)
        except ValueError:
            return "refused"
        return "native" if voice is None else voice

    def base_class(receipt):
        return "Oliver - Clean, British and Steady" if isinstance(receipt, dict) and receipt.get("source") == OLIVER_LABEL else "native"

    def locate(text):
        path = Path(text)
        for candidate in (path, REPO / path) + tuple(REPO / text[text.index(top):] for top in ("narration_lab/", "docs/") if top in text):
            if candidate.is_file():
                return candidate
        return None

    listing = run(["rg", "-l", "--glob", "*.json", '"generation_receipt"', "narration_lab", str(P)], REPO).stdout.split()
    manifests, receipts, missing = 0, {}, 0
    mixed = {"base": 0, "applied": 0, "fixed": 0}
    for name in sorted(listing):
        try:
            data = json.loads((REPO / name).read_text())
        except ValueError:
            continue
        units = data.get("units") if isinstance(data, dict) else None
        if not isinstance(units, list):
            units = (data.get("manifest") or {}).get("units") if isinstance(data, dict) else None
        refs = [u["generation_receipt"] for u in units or [] if isinstance(u, dict) and isinstance(u.get("generation_receipt"), str)]
        if not refs:
            continue
        manifests += 1
        classes = {"base": set(), "applied": set(), "fixed": set()}
        for ref in refs:
            path = locate(ref)
            if path is None:
                missing += 1
                continue
            key = str(path.resolve())
            if key not in receipts:
                try:
                    receipt = json.loads(path.read_text())
                except ValueError:
                    missing += 1
                    continue
                receipts[key] = {"base": base_class(receipt), **{l: classify(m, receipt) for l, m in variants.items()}}
            for label in classes:
                classes[label].add(receipts[key][label])
        for label in classes:
            mixed[label] += len(classes[label]) > 1
    agree = {label: sum(r[label] == r["base"] for r in receipts.values()) for label in ("applied", "fixed")}
    oliver = sum(r["base"] != "native" for r in receipts.values())
    say(f"     {manifests} span manifests name {len(receipts)} distinct receipts ({oliver} Oliver UI, "
        f"{len(receipts) - oliver} native); {missing} references no longer resolve")
    for label in ("applied", "fixed"):
        check(agree[label] == len(receipts), f"{label}: every retained receipt is classified as the canonical tool reads it "
              f"({agree[label]} of {len(receipts)})")
    check(mixed == {"base": 0, "applied": 0, "fixed": 0}, f"no retained manifest mixes voices or receipt kinds ({mixed})")
    census = run(["rg", "-l", "--glob", "*.json", '"source": *"ElevenLabs UI, Oliver"', "narration_lab", str(P)], REPO).stdout.split()
    legacy_receipts = [json.loads((REPO / n).read_text()) for n in sorted(census)]
    legacy_receipts = [r for r in legacy_receipts if isinstance(r, dict) and r.get("source") == OLIVER_LABEL]
    for label, module in variants.items():
        counted = sum(classify(module, r) == "Oliver - Clean, British and Steady" for r in legacy_receipts)
        check(counted == len(legacy_receipts), f"{label}: all {len(legacy_receipts)} retained receipts with the Oliver label read as Oliver")

    section("hygiene of this review's files")
    authored = [HERE / n for n in ("REVIEW.md", "checks/review_checks.py", "checks/scenarios.py", "proposed/legacy-voice-contradiction.diff")]
    check(all(p.is_file() for p in authored), "REVIEW.md, both check scripts and the proposed diff are present")
    authored = [p for p in authored if p.is_file()]
    machine = re.compile(r"/Users/|/private/|/home/|jemanuel|scratchpad|claude-501|@gmail")  # hygiene-pattern
    pronoun = re.compile(r"\b(he|she|his|her|him|hers|himself|herself)\b", re.I)  # hygiene-pattern
    bad = []
    for path in authored:
        lines = [l for l in path.read_text().splitlines() if not l.endswith("# hygiene-pattern")]
        if any(machine.search(l) or pronoun.search(l) for l in lines):
            bad.append(path.name)
    check(not bad, f"no machine paths, user names, addresses or gendered pronouns in {len(authored)} files" + (f" ({bad})" if bad else ""))

    say(f"RESULT {'PASS' if not failures else 'FAIL'}, {len(failures)} failed")
    return 0 if not failures else 1


if __name__ == "__main__":
    sys.exit(main())
