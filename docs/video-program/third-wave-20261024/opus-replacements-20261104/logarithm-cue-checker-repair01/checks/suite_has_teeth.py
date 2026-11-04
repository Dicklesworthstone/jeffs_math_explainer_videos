#!/usr/bin/env python3
"""Does docs/test_check_cues.py notice a broken checker?

    python3 -I -B <this file> EMPTY_SCRATCH_DIR

Run from the repo root. Writes only inside EMPTY_SCRATCH_DIR. It runs the regression file against
stage04's old checker, and against copies of the candidate with one rule switched off. Each must
make at least one case wrong. It shows the suite can fail. It doesn't show that the suite is
complete.
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
SUITE = HERE / REL / "docs/test_check_cues.py"

SABOTAGE = [
    ("line comments read as code", 'elif src.startswith("//", i):', "elif False:"),
    ("block comments read as code", 'elif src.startswith("/*", i):', "elif False:"),
    ("raw strings not recognised", "elif RAW_STRING.match(src, i):", "elif False:"),
    ("char literals not recognised", """elif src.startswith("'", i + 2):""", "elif False:"),
    ("cfg attributes ignored, so test modules read and other cfgs allowed", 'if head in ("cfg", "cfg_attr"):', "if False:"),
    ("#[test] items kept", 'if words == ["test"] and not inner:', "if False:"),
    ("any item accepted after a test attribute", 'if i >= len(tokens) or not (tokens[i].kind == "id" and tokens[i].text in ITEM_WORDS or at(tokens, i, "{")):', "if False:"),
    ("#[path] allowed", 'if head == "path":', "if False:"),
    ("include! allowed", 'if at(tokens, i, "include ! ("):', "if False:"),
    ("an empty registry allowed", "if not chapters:\n                failures.append", "if False:\n                failures.append"),
    ("a missing file skipped in silence", """failures.append(f"mod.rs {'registry names' if module in modules else 'declares'} {module}, but {module}.rs is missing")""", "pass"),
    ("an undeclared registered module allowed", "if module in modules and module not in declared:", "if False:"),
    ("helper modules not read for hooks", "if name[:-3] in modules or not comparable:", "if True:"),
    ("a said count of other than one allowed", "if len(who) != 1:", "if False:"),
    ("order not checked", "if last_anywhere is not None and order[cid] <= order[last_anywhere]:", "if False:"),
    ("chapter of a cue not checked", "if cid[:2] != key[:2]:", "if False:"),
    ("unknown cues skipped in silence", 'failures.append(f"{where}: unknown cue {cid}")', "pass"),
    ("the current line not checked", "if cid != last_said:", "if False:"),
    ("index bounds not checked", "if not (0 < hook.low and hook.high < count[cid]):", "if False:"),
    ("backwards indices allowed", "if hook.low < last_j.get(cid, 0):", "if False:"),
    ("any receiver accepted", 'if not called or not at(tokens, i - 2, "kit ."):', "if not called:"),
    ("a computed cue skipped in silence", """failures.append(f"{where}: {t.text}({shown}) does not name a literal cue")""", "pass"),
    ("any computed index accepted", 'raise Refusal("nobody has reviewed this computed index")', "return 1, 1"),
    ("the reviewed loop not checked", """if not at(tokens, kit_at - len(row["loop"].split()), row["loop"]):""", "if False:"),
    ("the reviewed array taken as four", 'length = array_len(tokens, row["array"])', "length = 4"),
    ("digits in spoken text allowed", 'if re.search(r"[0-9]", text):', "if False:"),
    ("duplicate script ids allowed", "if len(order) != len(cues):", "if False:"),
    ("malformed rows skipped in silence", """failures.append(f"script.tsv:{n}: no tab between cue id and text")""", "pass"),
]


def run_suite(checker_text, folder):
    docs = folder / "docs"
    docs.mkdir(parents=True)
    (docs / "check_cues.py").write_text(checker_text, encoding="utf-8")
    shutil.copyfile(SUITE, docs / "test_check_cues.py")
    empty = folder / "fixtures"
    empty.mkdir()
    done = subprocess.run([sys.executable, "-I", "-B", str(docs / "test_check_cues.py"), str(empty)], capture_output=True, text=True)
    last = (done.stdout + done.stderr).strip().splitlines()[-1]
    m = re.search(r"([0-9]+) wrong$", last)
    return int(m.group(1)) if m else None, last


def main(argv):
    scratch = Path(argv[0])
    if not scratch.is_dir() or any(scratch.iterdir()):
        raise SystemExit("scratch must be an existing empty directory")
    source = CANDIDATE.read_text(encoding="utf-8")
    silent = 0

    wrong, last = run_suite(source, scratch / "candidate")
    print(f"candidate as it stands: {wrong} wrong")
    if wrong != 0:
        silent += 1
    wrong, last = run_suite((STAGE / "docs/check_cues.py").read_text(encoding="utf-8"), scratch / "old")
    print(f"stage04's old checker: {wrong} wrong")
    if not wrong:
        silent += 1

    for n, (name, old, new) in enumerate(SABOTAGE, 1):
        if source.count(old) != 1:
            raise SystemExit(f"{name}: {old!r} occurs {source.count(old)} times")
        wrong, last = run_suite(source.replace(old, new), scratch / f"s{n:02d}")
        caught = wrong is None or wrong > 0
        print(f"{'caught' if caught else 'MISSED'}: {name}: {wrong if wrong is not None else 'suite crashed, ' + last[:80]} wrong")
        if not caught:
            silent += 1
    print(f"{len(SABOTAGE)} rules switched off one at a time; {silent} went unnoticed")
    return 1 if silent else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
