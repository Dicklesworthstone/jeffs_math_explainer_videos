#!/usr/bin/env python3
"""The candidate checker on stage04's actual bytes, and on planted true scene defects.

    python3 -I -B <this file> EMPTY_SCRATCH_DIR

Run from the repo root. Writes only inside EMPTY_SCRATCH_DIR. Each plant goes into two copies:

  new: the whole of stage04's src/chapters, retired files, .orig and test modules included,
       with the candidate checker. This is the setting it has to work in.
  old: registered chapters only, cut at their tests module, with stage04's own checker. That
       is the only setting where the old checker passes, so it's the fair comparison.

Gates: stage04 as it stands passes the new checker. Every plant that must refuse does, for its
named reason, including all of those the old checker refuses. Every plant that must pass does.
The output lists both directions of disagreement: what only the new checker refuses, and what
only the old one refuses. The second list is the only place the new checker accepts something
the old one didn't, so a reviewer should read it. Source text only; nothing is compiled, and
no scene is run.
"""

import re
import shutil
import subprocess
import sys
from pathlib import Path

REL = "explainer_corpus/logarithm_mysteries"
STAGE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage04") / REL
HERE = Path(__file__).resolve().parent.parent
CANDIDATE = HERE / REL / "docs/check_cues.py"
OLD_CHECKER = STAGE / "docs/check_cues.py"
CANONICAL = Path(REL)
CH = "src/chapters"
SCRIPT = "script.tsv"
TESTS = "#[cfg(test)]\nmod tests"


def say(cue):
    return f'kit.say(stage, "{cue}")?;'


def until(cue, j):
    return f'kit.until_sentence(stage, "{cue}", {j})?;'


def registered(mod_text):
    return [f"{m}.rs" for m in re.findall(r"make: \|kit\| Box::new\((\w+)::", mod_text)]


def files_of(root):
    out = {}
    for p in sorted((root / CH).iterdir()):
        if p.is_file():
            out[f"{CH}/{p.name}"] = p.read_bytes().decode("utf-8")
    out[SCRIPT] = (root / SCRIPT).read_text(encoding="utf-8")
    return out


def scene_only(files):
    names = registered(files[f"{CH}/mod.rs"])
    out = {SCRIPT: files[SCRIPT], f"{CH}/mod.rs": files[f"{CH}/mod.rs"]}
    for name in names:
        text = files[f"{CH}/{name}"]
        at = text.find(TESTS)
        out[f"{CH}/{name}"] = text if at < 0 else text[:at]
    return out


def write(crate, files, checker):
    for rel, body in files.items():
        path = crate / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body, encoding="utf-8")
    (crate / "docs").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(checker, crate / "docs/check_cues.py")


def run(crate):
    done = subprocess.run([sys.executable, "-I", "-B", str(crate / "docs/check_cues.py")], capture_output=True, text=True)
    lines = (done.stdout + done.stderr).splitlines()
    failures = [f.strip() for f in lines[lines.index("FAIL") + 1 :]] if "FAIL" in lines else []
    return done.returncode, lines, failures


def first(files, rel, old, new):
    """Replace the first occurrence, which must be scene code and not the tests module."""
    if rel not in files:
        raise AssertionError(f"{rel} absent")
    text = files[rel]
    at = text.find(old)
    if at < 0:
        raise AssertionError(f"{rel}: {old!r} is not there")
    cut = text.find(TESTS)
    if cut >= 0 and at > cut:
        raise AssertionError(f"{rel}: first {old!r} is in the tests module")
    files[rel] = text[:at] + new + text[at + len(old) :]


def scene(files, *edits):
    for rel, old, new in edits:
        first(files, f"{CH}/{rel}", old, new)


def no_registry(files):
    rel = f"{CH}/mod.rs"
    new, n = re.subn(r"(pub fn registry\(\) -> Vec<Chapter> \{\n).*?(\n\}\n)", r"\1    vec![]\2", files[rel], count=1, flags=re.S)
    assert n == 1
    files[rel] = new


def swapped_registry(files):
    rel = f"{CH}/mod.rs"
    entries = re.findall(r"        Chapter \{\n.*?\n        \},\n", files[rel], flags=re.S)
    assert len(entries) >= 2
    files[rel] = files[rel].replace(entries[0] + entries[1], entries[1] + entries[0])


def remove(files, rel):
    del files[rel]


def add_row(files, row):
    files[SCRIPT] += row


def replace_row(files, cue, new):
    rows = files[SCRIPT].split("\n")
    hits = [k for k, r in enumerate(rows) if r.startswith(cue + "\t")]
    assert len(hits) == 1
    rows[hits[0]] = new
    files[SCRIPT] = "\n".join(rows)


def drop_row(files, cue):
    rows = [r for r in files[SCRIPT].split("\n") if not r.startswith(cue + "\t")]
    files[SCRIPT] = "\n".join(rows)


UG = "undoing_growth.rs"
PR = "product_rule.rs"
LOOP = "for (k, &m) in steps.iter().enumerate() {"
STEP = 'r"= \\sqrt{1} = 1",'

# (name, edit, required failure text or None for must-pass, old copy applies?)
PLANTS = [
    ("real say removed; the test string that names it stays", lambda f: scene(f, (UG, say("04_5"), "")), "04_5: said 0 times", True),
    ("real say replaced by a comment", lambda f: scene(f, (UG, say("04_5"), "// " + say("04_5"))), "04_5: said 0 times", True),
    ("real say replaced by a block comment", lambda f: scene(f, (UG, say("04_5"), "/* " + say("04_5") + " */")), "04_5: said 0 times", True),
    ("real say replaced by a string", lambda f: scene(f, (UG, say("04_5"), 'let _s = "kit.say(stage, \\"04_5\\")";')), "04_5: said 0 times", True),
    ("real say replaced by a raw string", lambda f: scene(f, (UG, say("04_5"), 'let _s = r#"kit.say(stage, "04_5")"#;')), "04_5: said 0 times", True),
    ("real say moved into a cfg(test) helper", lambda f: (scene(f, (UG, say("04_5"), "")), f.__setitem__(f"{CH}/{UG}", f[f"{CH}/{UG}"] + '#[cfg(test)]\nfn probe() {\n    kit.say(stage, "04_5");\n}\n')), "04_5: said 0 times", True),
    ("real say moved to a retired file", lambda f: (scene(f, (UG, say("04_5"), "")), f.__setitem__(f"{CH}/bookends.rs", f.get(f"{CH}/bookends.rs", "") + 'fn old() { kit.say(stage, "04_5"); }\n')), "04_5: said 0 times", True),
    ("real hook past the end of its cue", lambda f: scene(f, (PR, until("08_4", 5), until("08_4", 99))), "until_sentence(08_4, 99) but 08_4 has 7 sentences", True),
    ("real hook at index zero", lambda f: scene(f, (PR, until("08_4", 5), until("08_4", 0))), "until_sentence(08_4, 0) but 08_4 has 7 sentences", True),
    ("real hook going backwards", lambda f: scene(f, (PR, until("08_4", 6), until("08_4", 2))), "until_sentence(08_4, 2) goes backwards", True),
    ("real hook on a cue that isn't the current line", lambda f: scene(f, (PR, until("08_4", 6), until("08_3", 1))), "while 08_4 is the current line", True),
    ("real hook on an unknown cue", lambda f: scene(f, (PR, until("08_4", 6), until("08_99", 1))), "unknown cue 08_99", True),
    ("a real say duplicated", lambda f: scene(f, (PR, say("08_5"), say("08_5") + "\n        " + say("08_5"))), "08_5: said 2 times", True),
    ("a real say out of order", lambda f: scene(f, (PR, say("08_5"), "@@"), (PR, say("08_4"), say("08_5")), (PR, "@@", say("08_4"))), "08_4 said after 08_5", True),
    ("a real say on an unknown cue", lambda f: scene(f, (PR, say("08_5"), say("08_99"))), "unknown cue 08_99", True),
    ("a real say in the wrong chapter", lambda f: scene(f, ("ratio_ruler.rs", say("00_5"), ""), ("building_ruler.rs", say("01_1"), say("00_5") + "\n        " + say("01_1"))), "00_5 said in chapter 01_building_ruler", True),
    ("a say with a computed cue", lambda f: scene(f, (PR, say("08_5"), 'let id = "08_5"; kit.say(stage, id)?;')), "does not name a literal cue", True),
    ("the computed hook pushed past its cue", lambda f: scene(f, (PR, 'kit.until_sentence(stage, "08_4", k + 1)?;', 'kit.until_sentence(stage, "08_4", k + 9)?;')), "has a computed index", True),
    ("the computed hook's loop reversed", lambda f: scene(f, (PR, LOOP, "for (k, &m) in steps.iter().rev().enumerate() {")), "reviewed loop", True),
    ("the computed hook's array grown past the cue", lambda f: scene(f, (PR, STEP, STEP + ' r"a", r"b", r"c", r"d",')), "until_sentence(08_4, k + 1) but 08_4 has 7 sentences", True),
    ("a say in an unmarked module called tests", lambda f: f.__setitem__(f"{CH}/{PR}", f[f"{CH}/{PR}"] + 'mod tests_unmarked {\n    fn t() {\n        kit.say(stage, "08_5");\n    }\n}\n'), "08_5: said 2 times", True),
    ("a cfg(not(test)) in a chapter", lambda f: f.__setitem__(f"{CH}/{PR}", f[f"{CH}/{PR}"] + "#[cfg(not(test))]\nfn f() {}\n"), "a cfg this check can't evaluate", True),
    ("a say in a helper module", lambda f: f.__setitem__(f"{CH}/draw.rs", f[f"{CH}/draw.rs"] + 'pub fn extra() {\n    kit.say(stage, "08_5");\n}\n'), "say 08_5 in a module the registry does not run", False),
    ("the retired bookends.rs declared again", lambda f: first(f, f"{CH}/mod.rs", "mod branch_cut;", "mod bookends;\nmod branch_cut;"), "bookends.rs:", False),
    ("a registered file missing", lambda f: remove(f, f"{CH}/matrix_logs.rs"), "matrix_logs.rs is missing", True),
    ("an empty registry", no_registry, "mod.rs registry is empty", True),
    ("two chapters swapped in the registry", swapped_registry, "mod.rs registry out of order", True),
    ("a registered module not declared", lambda f: first(f, f"{CH}/mod.rs", "mod matrix_logs;\n", ""), "matrix_logs, which mod.rs does not declare", True),
    ("the registry pointed at a retired module", lambda f: first(f, f"{CH}/mod.rs", "Box::new(matrix_logs::", "Box::new(matrix_log::"), "names matrix_log, which mod.rs does not declare", False),
    ("a chapter key renumbered", lambda f: first(f, f"{CH}/mod.rs", 'key: "09_', 'key: "19_'), "does not cover cue prefixes", True),
    ("a cue row duplicated in the script", lambda f: add_row(f, "08_5\tSpoken twice.\n"), "duplicate cue ids", True),
    ("digits in spoken text", lambda f: replace_row(f, "01_1", "01_1\tNow 3 words."), "01_1: digits in spoken text", True),
    ("a script row with no tab", lambda f: add_row(f, "13_1 no tab\n"), "no tab between cue id and text", True),
    ("a said cue dropped from the script", lambda f: drop_row(f, "08_5"), "unknown cue 08_5", True),
    ("a new cue nobody says", lambda f: add_row(f, "12_99\tNobody says this.\n"), "12_99: said 0 times", True),
    # these must not stop the film
    ("a comment that repeats a real say", lambda f: scene(f, (PR, say("08_5"), "// " + say("08_5") + "\n        " + say("08_5"))), None, True),
    ("a char literal holding a quote before a real say", lambda f: scene(f, (PR, say("08_5"), "let _q = '\"';\n        " + say("08_5"))), None, True),
    ("a string with hook text in it, the real say kept", lambda f: scene(f, (PR, say("08_5"), 'let _s = r#"kit.until_sentence(stage, "08_5", 99)"#;\n        ' + say("08_5"))), None, True),
    ("another test fixture naming a bad hook", lambda f: f.__setitem__(f"{CH}/{PR}", f[f"{CH}/{PR}"] + '#[cfg(test)]\nmod more_tests {\n    #[test]\n    fn t() {\n        kit.until_sentence(stage, "08_5", 99);\n    }\n}\n'), None, True),
]


def listing(lines):
    out = set()
    for line in lines:
        m = re.fullmatch(r'  ([a-z_0-9]+\.rs):([0-9]+): "(.*)"', line)
        if m:
            out.add((m.group(1), int(m.group(2)), m.group(3)))
    return out


def main(argv):
    scratch = Path(argv[0])
    if not scratch.is_dir() or any(scratch.iterdir()):
        raise SystemExit("scratch must be an existing empty directory")
    whole = files_of(STAGE)
    cut = scene_only(whole)
    wrong = 0

    ug = whole[f"{CH}/{UG}"]
    if ug.find(say("04_5")) > ug.find(TESTS) or 'kit.say(stage, "04_5")' not in ug[ug.find(TESTS) :]:
        raise AssertionError("the 04_5 plants need a real say and a test string that names it")
    print("Baselines, stage04 bytes:")
    write(scratch / "base-new", whole, CANDIDATE)
    code, lines, failures = run(scratch / "base-new")
    print(f"  new checker, stage04 as it stands: exit {code}; {lines[0]}; {len(failures)} failures; last line {lines[-1]!r}")
    for line in lines[1:]:
        if line.startswith(("registered modules", "other declared", "not declared", "other cue helpers")) or " until_sentence(" in line:
            print(f"    {line.strip()}")
    if code or lines[0] != "80 cues, 364 sentences, 80 said" or lines[-1] != "OK":
        wrong += 1
        print("  WRONG: stage04 must pass")
    new_listed = listing(lines)

    write(scratch / "base-old-whole", whole, OLD_CHECKER)
    code, lines, failures = run(scratch / "base-old-whole")
    print(f"  old checker, stage04 as it stands: exit {code}; {lines[0]}; {len(failures)} failures")
    write(scratch / "base-old-cut", cut, OLD_CHECKER)
    code, lines, failures = run(scratch / "base-old-cut")
    print(f"  old checker, registered scene code only: exit {code}; {lines[0]}; {len(failures)} failures; last line {lines[-1]!r}")
    if code:
        wrong += 1
        print("  WRONG: the old checker must pass on scene code, or the comparison isn't fair")
    old_listed = listing(lines)

    print()
    print("The informational digit listing (old on scene code, new on stage04):")
    print(f"  old lists {len(old_listed)}, new lists {len(new_listed)}")
    lost = sorted(old_listed - new_listed)
    print(f"  in old's and not new's: {len(lost)}")
    for name, n, lit in lost[:20]:
        print(f"    {name}:{n}: {lit[:100]}")

    print()
    print("Plants. old = old checker on scene code; new = candidate on all of stage04.")
    print(" old new  plant")
    old_missed, only_old = [], []
    for n, (name, edit, reason, old_applies) in enumerate(PLANTS, 1):
        planted = dict(whole)
        edit(planted)
        write(scratch / f"p{n:02d}-new", planted, CANDIDATE)
        new_code, _, new_failures = run(scratch / f"p{n:02d}-new")
        old_code = None
        if old_applies:
            cut_planted = dict(cut)
            edit(cut_planted)
            write(scratch / f"p{n:02d}-old", cut_planted, OLD_CHECKER)
            old_code = run(scratch / f"p{n:02d}-old")[0]
        print(f"  {'-' if old_code is None else old_code}   {new_code}   {name}")
        if reason is None:
            if new_code != 0:
                wrong += 1
                print(f"      WRONG: must pass, got {new_failures[:4]}")
            if old_code == 1:
                only_old.append(name)
            continue
        hits = [f for f in new_failures if reason in f]
        if new_code != 1 or not hits:
            wrong += 1
            print(f"      WRONG: must refuse for {reason!r}, got exit {new_code}, {new_failures[:4]}")
        else:
            print(f"      {hits[0][:150]}")
        if old_code == 1 and new_code != 1:
            wrong += 1
            print("      WRONG: the old checker refused this and the new one didn't")
        if old_code == 0:
            old_missed.append(name)

    must_refuse = sum(p[2] is not None for p in PLANTS)
    print()
    print(f"{len(PLANTS)} plants: {must_refuse} must refuse, {len(PLANTS) - must_refuse} must pass, {wrong} wrong")
    print(f"Refused by new and passed by old ({len(old_missed)}):")
    for name in old_missed:
        print(f"  {name}")
    print(f"Passed by new and refused by old ({len(only_old)}). Each is text that is not a compiled call:")
    for name in only_old:
        print(f"  {name}")

    print()
    print("Canonical crate, for information only. No gate reads this.")
    if (CANONICAL / CH / "mod.rs").is_file():
        write(scratch / "canon-new", files_of(CANONICAL), CANDIDATE)
        code, lines, failures = run(scratch / "canon-new")
        print(f"  new checker: exit {code}; {lines[0]}; {len(failures)} failures")
        for f in failures[:12]:
            print(f"    {f[:170]}")
    return 1 if wrong else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
