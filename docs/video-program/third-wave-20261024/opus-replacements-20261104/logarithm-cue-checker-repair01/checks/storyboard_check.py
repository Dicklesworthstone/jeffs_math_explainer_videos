#!/usr/bin/env python3
"""The candidate storyboard against stage04's script and source, and against the final80 binder.

    python3 -I -B <this file> EMPTY_SCRATCH_DIR

Run from the repo root. Writes only inside EMPTY_SCRATCH_DIR. It loads the frozen binder from
logarithm-final80-bound-plan01 and points it at a scratch copy of stage04 that differs only in
docs/storyboard.md, so what reads the tables here is the binder's actual parser. Nothing in the
frozen bundle or in stage04 is changed. Text only: no listening, no build.
"""

import difflib
import hashlib
import importlib.util
import json
import shutil
import sys
from pathlib import Path

REL = "explainer_corpus/logarithm_mysteries"
STAGE_ROOT = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage04")
STAGE = STAGE_ROOT / REL
HERE = Path(__file__).resolve().parent.parent
CANDIDATE = HERE / REL / "docs/storyboard.md"
FINAL80 = HERE.parent / "logarithm-final80-bound-plan01"
SCRIPT_SHA = "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84"
BASE_SHA = "b1f9f3dd35e6a5074e895c73f7849565f15a109e4d213276e41f03f131aaa7cb"
ENDS_ON_QUESTION = ["01_3", "02_2", "04_6", "05_1", "05_5", "06_2", "06_4", "07_3", "08_1", "08_4", "11_3"]
DOES_NOT = ["00_4", "12_5"]
PREDICTION_HEADING = "## Prediction holds"
READING_HEADING = "## Authored reading holds"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load_binder():
    spec = importlib.util.spec_from_file_location("bind_final80", FINAL80 / "bind_final80.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def changed_lines(base, new):
    """Base line ranges the candidate changes, 0-based, half open."""
    a, b = base.split("\n"), new.split("\n")
    return [(i1, i2) for tag, i1, i2, _, _ in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes() if tag != "equal"]


def main(argv):
    scratch = Path(argv[0])
    if not scratch.is_dir() or any(scratch.iterdir()):
        raise SystemExit("scratch must be an existing empty directory")
    problems = []

    def need(ok, what):
        print(f"{'ok ' if ok else 'BAD'} {what}")
        if not ok:
            problems.append(what)

    base_bytes = (STAGE / "docs/storyboard.md").read_bytes()
    base, new = base_bytes.decode("utf-8"), CANDIDATE.read_text(encoding="utf-8")
    need(sha(base_bytes) == BASE_SHA, f"the base is stage04's storyboard, {BASE_SHA[:16]}")
    script_bytes = (STAGE / "script.tsv").read_bytes()
    need(sha(script_bytes) == SCRIPT_SHA, f"stage04's script is the final 80, {SCRIPT_SHA[:16]}; this repair has no script edit")

    # Only the two hold sections change.
    lines = base.split("\n")
    pred = lines.index(next(l for l in lines if l.startswith(PREDICTION_HEADING)))
    read = lines.index(READING_HEADING)
    after = next(i for i in range(read + 1, len(lines)) if lines[i].startswith("## "))
    changes = changed_lines(base, new)
    need(all(pred < i1 and i2 < after for i1, i2 in changes), f"all {len(changes)} changed runs fall inside the two hold sections (base lines {pred + 1} to {after})")

    # The binder's own parser, on a copy of stage04 with only the storyboard replaced.
    copied = scratch / "stage" / REL
    shutil.copytree(STAGE_ROOT, scratch / "stage")
    shutil.copyfile(CANDIDATE, copied / "docs/storyboard.md")

    binder = load_binder()
    stock = binder.storyboard_tables()
    stock_answered = {r[0]: r[2] for r in stock[0]}
    binder.STAGE_ROOT, binder.STAGE, binder.CH = scratch / "stage", copied, copied / "src/chapters"
    predictions, readings, text = binder.storyboard_tables()
    answered = {r[0]: r[2] for r in predictions}
    need(len(predictions) == 13, "the binder's parser reads 13 prediction rows")
    need(answered == stock_answered, "every answered-in cell is where the binder reads it, unchanged")
    need([r[0] for r in readings] == ["10_4", "11_20"], f"the binder's parser reads the reading rows {[r[0] for r in readings]}")
    for needle in ("159 characters", "107 characters", "17.73 s"):
        need(needle in text, f"the binder's needle {needle!r} is still there")
    row_11_20 = next(l for l in base.split("\n") if l.startswith("| 11_20 |"))
    need(row_11_20 in new.split("\n"), "the 11_20 row is byte for byte the same")

    # The question column against the script.
    cues = dict(line.split("\t", 1) for line in script_bytes.decode("utf-8").splitlines() if line and not line.startswith("#"))
    coherent = binder.load_coherent()
    print()
    print("The last column against the script's sentences:")
    for row in predictions:
        cue, last_cell = row[0], row[3]
        said = coherent.sentences(cues[cue])
        ends = said[-1].endswith("?")
        asks = [s for s in said if s.endswith("?")]
        need(bool(asks), f"{cue} has a question: {asks[-1] if asks else None!r}")
        need(last_cell.startswith("yes" if ends else "no"), f"{cue}: the table says {last_cell!r}; the cue ends {said[-1]!r}")
    need(sorted(r[0] for r in predictions if r[3].startswith("yes")) == ENDS_ON_QUESTION, "yes rows are the 11 expected")
    need(sorted(r[0] for r in predictions if r[3].startswith("no")) == DOES_NOT, "no rows are 00_4 and 12_5")
    print()

    # The 10_4 row against the source.
    detail = binder.reading_10_4()
    row_10_4 = next(l for l in new.split("\n") if l.startswith("| 10_4 |"))
    scene = (STAGE / "src/chapters/doing_nothing.rs").read_text(encoding="utf-8")
    need(detail["fine_print_chars"] == 197 and f"{detail['fine_print_chars']} characters" in row_10_4, "197 characters matches TEXTBOOK + \"precisely: \" + SOURCE")
    need(detail["floor_seconds_from_shown"] == 13.2 and "13.2 s" in row_10_4, "13.2 s matches FINE_PRINT_S")
    need("if read_for < FINE_PRINT_S {\n            kit.music_breath(stage, FINE_PRINT_S - read_for)?;" in scene, "the window in source is conditional, as the row says")
    need("can be zero" in row_10_4, "the row says the window can be zero, and claims no length")

    # The whole binder, on that copy. Only what reads the storyboard may differ.
    outputs, audit, _, _ = binder.build()
    print()
    print("The final80 binder rerun on the scratch copy, against the frozen bundle:")
    differing = {}
    for rel, body in sorted(outputs.items()):
        frozen_body = (FINAL80 / rel).read_text(encoding="utf-8")
        if rel == "inputs.sha256":
            body = body.replace(str(binder.STAGE_ROOT), str(STAGE_ROOT))
        if body != frozen_body:
            differing[rel] = (frozen_body, body)
    same = sorted(set(outputs) - set(differing))
    print(f"  {len(outputs)} outputs; byte for byte the same: {same}")
    print(f"  differing: {sorted(differing)}")
    for rel in ("hold-audit.json", "inputs.sha256"):
        if rel in differing:
            old_lines, new_lines = (text.split("\n") for text in differing[rel])
            for line in difflib.unified_diff(old_lines, new_lines, lineterm="", n=0):
                if not line.startswith(("---", "+++", "@@")):
                    print(f"    {rel}: {' '.join(line.split())[:150]}")
    need(sorted(differing) == ["direction-dispositions-proposed.json", "hold-audit.json", "inputs.sha256"], "only the audit, the rows that bind the audit, and the input hashes differ")
    need({"narration_units.json", "docs/narration-direction-map-oliver12.json"} <= {r.split(f"{REL}/")[-1] for r in same}, "the plan and the map are byte for byte the same")
    old_audit, new_audit = differing["hold-audit.json"]
    old_rows, new_rows = differing["direction-dispositions-proposed.json"]
    need(new_rows.replace(sha(new_audit.encode("utf-8")), sha(old_audit.encode("utf-8"))) == old_rows, "the dispositions change only in the hold-audit hash they bind")
    print(f"    hold-audit hash in the 15 rows: {sha(old_audit.encode('utf-8'))[:16]} becomes {sha(new_audit.encode('utf-8'))[:16]}")
    old_inputs, new_inputs = (set(text.split("\n")) for text in differing["inputs.sha256"])
    need(len(old_inputs - new_inputs) == 1 and len(new_inputs - old_inputs) == 1 and f"{sha(new.encode('utf-8'))}  " in next(iter(new_inputs - old_inputs)), "one input hash differs, the storyboard's")
    need(audit["storyboard"]["reading_windows_missing_from_table"] == [], "the binder now finds no reading window missing from the table")
    frozen = json.loads((FINAL80 / "hold-audit.json").read_text(encoding="utf-8"))
    need(frozen["storyboard"]["reading_windows_missing_from_table"] == ["10_4"], "the frozen audit recorded 10_4 as missing, which this repairs")

    print()
    print(f"{len(problems)} problems")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
