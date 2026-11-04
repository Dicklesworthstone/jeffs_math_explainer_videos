#!/usr/bin/env python3
"""Can the stand-in run fail? Break copies of the scene file and see.

Each mutant changes one thing in a scratch copy of candidate/ and checks/stub/.
Each must be caught by rustc, a test, the object ledger or the overlap lint.
A mutant that survives means the clean result of run_against_stub.py is empty.

Usage: python3 -I -B checks/stub_sensitivity.py SCRATCH_DIR
"""

import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RELATIVE = "candidate/explainer_corpus/abstract_algebra_expanded/src/permutations.rs"
LOOPS_CLEAR = "fn loops_run(stage: &mut Stage<'_>, k: &Kit, board: &mut LoopBoard) -> fmn::Result<()> {\n    clear_notes(stage, &mut board.notes)?;\n"
LOOPS_KEEP = "fn loops_run(stage: &mut Stage<'_>, k: &Kit, board: &mut LoopBoard) -> fmn::Result<()> {\n"

MUTANTS = [
    ("old prediction caption left up", LOOPS_CLEAR, LOOPS_KEEP, "overlap"),
    (
        "fades out a caption already gone",
        "    leaving.retain(|&mob| mob != scene.notes[1]);\n",
        "",
        "ledger",
    ),
    (
        "moves a card before it appears",
        "    fade(stage, &showing, true)?;\n\n    k.until_sentence(stage, \"11_18\", 1)?;\n",
        "    k.until_sentence(stage, \"11_18\", 1)?;\n",
        "ledger",
    ),
    (
        "waits on a sentence the cue lacks",
        'k.until_sentence(stage, "11_19", 2)?;',
        'k.until_sentence(stage, "11_19", 3)?;',
        "scene",
    ),
    (
        "speaks two cues out of script order",
        'k.say(stage, "11_18")?;',
        'k.say(stage, "11_19")?;',
        "scene",
    ),
    (
        "uses a board after moving it away",
        "let wires = loop_and_wires(stage, &k, full)?;",
        "let wires = loop_and_wires(stage, &k, full)?;\n        let _ = full.mobs();",
        "rustc",
    ),
    (
        "the wrong shuffle for the deck",
        "let p = perfect_shuffle(DECK_CARDS, true);\n    cut(",
        "let p = perfect_shuffle(DECK_CARDS, false);\n    cut(",
        "scene",
    ),
    (
        "a slip that does not change parity",
        "            *place = slot + 1;\n",
        "            *place = slot;\n",
        "test",
    ),
]


def attempt(copy, out):
    """Return what caught the mutant: rustc, test, scene, ledger, overlap, or None."""
    fmn = out / "libfmn.rlib"
    kit = out / "libexplainer_kit.rlib"
    links = ["-L", str(out), "--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}"]
    root = copy / "checks/stub/crate_root.rs"
    base = ["rustc", "--edition", "2024", *links, "--crate-name", "chapter"]
    tests = out / "mutant_tests"
    if subprocess.run([*base, "--test", "-o", str(tests), str(root)], capture_output=True).returncode:
        return "rustc"
    if subprocess.run([str(tests)], capture_output=True).returncode:
        return "test"
    scene = out / "mutant_scene"
    subprocess.run([*base, "-o", str(scene), str(root)], capture_output=True, check=True)
    done = subprocess.run([str(scene)], capture_output=True, text=True)
    if done.returncode:
        return "scene"
    if "\n0 object-ledger problems" not in done.stdout:
        return "ledger"
    # One box overlap is the baseline: slot label 3 and the ring of 11_4, by 0.014,
    # on an em box. The digit's ink is shorter than that box and clears the ring.
    if "\n1 estimated overlaps" not in done.stdout:
        return "overlap"
    return None


def main():
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    copy = out / "copy"
    fmn = out / "libfmn.rlib"
    kit = out / "libexplainer_kit.rlib"
    stub = ROOT / "checks/stub"
    subprocess.run(
        ["rustc", "--edition", "2024", "--crate-type", "rlib", "--crate-name", "fmn", "-o", str(fmn),
         str(stub / "fmn_stub.rs")], check=True, capture_output=True)
    subprocess.run(
        ["rustc", "--edition", "2024", "--crate-type", "rlib", "--crate-name", "explainer_kit",
         "--extern", f"fmn={fmn}", "-o", str(kit), str(stub / "kit_stub.rs")], check=True, capture_output=True)
    shutil.copytree(ROOT / "candidate", copy / "candidate", dirs_exist_ok=True)
    shutil.copytree(stub, copy / "checks/stub", dirs_exist_ok=True)
    original = (ROOT / RELATIVE).read_text()
    target = copy / RELATIVE

    target.write_text(original)
    baseline = attempt(copy, out)
    print(f"unchanged copy: caught by {baseline}")
    survivors = 1 if baseline else 0
    for name, before, after, expected in MUTANTS:
        if original.count(before) != 1:
            sys.exit(f"mutant {name!r}: pattern occurs {original.count(before)} times")
        target.write_text(original.replace(before, after))
        caught = attempt(copy, out)
        print(f"{name}: caught by {caught} (expected {expected})")
        if caught != expected:
            survivors += 1
    print(f"{survivors} surprises")
    sys.exit(1 if survivors else 0)


main()
