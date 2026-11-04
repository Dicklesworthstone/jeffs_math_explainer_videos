#!/usr/bin/env python3
"""Can these checks fail? Break copies of the repaired chapters and see.

Each mutant undoes or breaks one thing in a scratch copy of candidate/. Every layer
that can see it is run and named, so the report shows which layer bites on what:

  rustc    the chapter (with its const asserts) no longer compiles
  test     a chapter test fails
  scene    the scene errors at a speaking rate
  ledger   the stand-in's object ledger reports a problem (a double fade, say)
  overlap  the estimated overlap report gains a line the unchanged copy lacks
  claim    a claim in checks/state_checks.py fails

A mutant no layer catches means the clean result says nothing about that repair. So
does an unchanged copy that anything catches. Exit 1 on either.

Usage: python3 -I -B checks/stub_sensitivity.py SCRATCH_DIR
Writes only under SCRATCH_DIR.
"""

import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STUB = ROOT / "checks/stub"
PREFIX = "explainer_corpus/logarithm_mysteries"
CHAPTERS = ("building_ruler", "slide_rule", "folded_decade", "undoing_growth")
RATES = ("2.68", "2.3", "3.1")

MUTANTS = [
    ("lengths inset by half a stroke, as the review first said", "building_ruler",
     "const INSET: f64 = 0.06;", "const INSET: f64 = 0.03;"),
    ("base-two words stay up for the base-ten sentence", "building_ruler",
     "fade_out_all(stage, &retired, 0.5)?;", "let _ = &retired;"),
    ("keyboard line arrives over the power row", "building_ruler",
     "swap(stage, &[threes, twos, gap], &[keys], 0.6)?;", "fade_in_all(stage, &[keys], 0.6)?;"),
    ("hairline back at 0.62", "slide_rule",
     "const HAIR_REACH: f64 = 0.42;", "const HAIR_REACH: f64 = 0.62;"),
    ("end tick and its words left out of the fade", "slide_rule",
     "&[six, index, off_line, off, ghost, beyond, end, stops, c_name],",
     "&[six, index, off_line, off, ghost, beyond, c_name],"),
    ("the slide waits on a phrase the cue lacks", "slide_rule",
     'const SLIDE_PHRASE: &str = "slide the moving";', 'const SLIDE_PHRASE: &str = "slide the movng";'),
    ("the landing tick lit before the turn: an answer leak", "folded_decade",
     "let lit_tick = stage.add_value_tracker(0.0);", "let lit_tick = stage.add_value_tracker(1.0);"),
    ("readings back on the helix", "folded_decade",
     "const READ_DX: f64 = 0.65;", "const READ_DX: f64 = 0.0;"),
    ("the old title stays under the new one", "folded_decade",
     "swap(stage, &[heading2], &[heading3, forgets], 0.5)?;", "fade_in_all(stage, &[heading3, forgets], 0.5)?;"),
    ("ln 2's label back on the tick labels", "undoing_growth",
     "p(1.1, -0.75),\n        )?;\n        fade_in_lagged(stage, &[across, down, ln2_word], 1.4, 0.5)?;",
     "p(1.1, -0.4),\n        )?;\n        fade_in_lagged(stage, &[across, down, ln2_word], 1.4, 0.5)?;"),
    ("the name y = e^x left out of the flip's fade", "undoing_growth",
     "fade_out_all(stage, &[name, run0, rise0, floor, nowhere], 0.25)?;",
     "fade_out_all(stage, &[run0, rise0, floor, nowhere], 0.25)?;"),
    ("the still floor faded twice", "undoing_growth",
     "fade_out_all(stage, &[name, run0, rise0, floor, nowhere], 0.25)?;",
     "fade_out_all(stage, &[name, run0, rise0, floor, nowhere], 0.25)?;\n"
     "        fade_out_all(stage, &[floor], 0.2)?;"),
    ("a sentence the cue lacks", "undoing_growth",
     'kit.until_sentence(stage, "04_4", 4)?;', 'kit.until_sentence(stage, "04_4", 9)?;'),
    ("the name back at 1.9, on the mirror", "undoing_growth",
     "const EXP_NAME_X: f64 = 1.68;", "const EXP_NAME_X: f64 = 1.9;"),
    ("the red bar's far end stays behind in the flip", "undoing_growth",
     "let (a1, b1) = flipped(0.0, LO, m);", "let (a1, b1) = flipped(0.0, LO, 0.0);"),
    ("the teal bar never slides, so nothing lands on 8", "undoing_growth",
     "shift_all(stage, &[bar_b], v(at(2.0) - at(1.0), 0.0), 1.2)?;", "let _ = (&bar_b, at(2.0));"),
    ("closing rows back on the dashed line", "undoing_growth",
     "const TEN_ROWS_X: f64 = 4.4;", "const TEN_ROWS_X: f64 = 2.6;"),
]


def crate_root(out, name, chapter_dir):
    support = ROOT / "base" / PREFIX / "src"
    lines = [
        "#![allow(dead_code, clippy::should_implement_trait)]",
        f'#[path = "{support}/facts.rs"]',
        "pub mod facts;",
        f'#[path = "{support}/model.rs"]',
        "pub mod model;",
        "pub mod chapters {",
        f'    #[path = "{support}/chapters/draw.rs"]',
        "    pub mod draw;",
    ]
    for chapter in CHAPTERS:
        lines += [f'    #[path = "{chapter_dir}/{chapter}.rs"]', f"    pub mod {chapter};"]
    lines += ["}", f'include!("{STUB}/harness_main.rs");']
    path = out / f"{name}_root.rs"
    path.write_text("\n".join(lines) + "\n")
    return path


def overlap_lines(stdout):
    """The overlap report's lines, without the seconds, which move with any timing."""
    found = set()
    for line in stdout.splitlines():
        if line.startswith("  (in motion)"):
            found.add(line.strip())
        else:
            match = re.match(r"\s+\d+\.\d\s+(\S.*)$", line)
            if match and ('" meets' in line or "safe frame" in line):
                found.add(match.group(1))
    return found


class Trial:
    def __init__(self, out):
        self.out = out
        self.check = ROOT / "checks/state_checks.py"
        fmn, kit = out / "libfmn.rlib", out / "libexplainer_kit.rlib"
        self.links = ["-L", str(out), "--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}"]
        self.rustc = ["rustc", "--edition", "2024", *self.links, "--crate-name", "chapter"]
        self.baseline = set()

    def run(self, root, tree_root):
        """Every layer's verdict on one tree, and what the claims named."""
        env = dict(os.environ, TREE_ROOT=str(tree_root))
        out = self.out
        tests = out / "trial_tests"
        if subprocess.run([*self.rustc, "--test", "-o", str(tests), str(root)], capture_output=True, env=env).returncode:
            return ["rustc"], []
        caught = []
        if subprocess.run([str(tests), "--test-threads=1"], capture_output=True).returncode:
            caught.append("test")
        scene = out / "trial_scene"
        subprocess.run([*self.rustc, "-o", str(scene), str(root)], capture_output=True, check=True, env=env)
        overlaps, ledger = set(), False
        for rate in RATES:
            done = subprocess.run(
                [str(scene), rate, str(out / f"candidate-{rate}.tsv")], capture_output=True, text=True,
            )
            if done.returncode:
                return [*caught, "scene"], []
            ledger = ledger or any(
                int(n) for n in re.findall(r"^(\d+) object-ledger problems", done.stdout, re.M)
            )
            overlaps |= overlap_lines(done.stdout)
        if ledger:
            caught.append("ledger")
        if overlaps - self.baseline:
            caught.append("overlap")
        self.seen_overlaps = overlaps
        judged = subprocess.run(
            [sys.executable, "-I", "-B", str(self.check), str(out)], capture_output=True, text=True,
        )
        failed = sorted({line.split("\t")[0] for line in judged.stdout.splitlines() if "FAIL" in line})
        if judged.returncode:
            caught.append("claim")
        return caught, failed


def main():
    out = Path(sys.argv[1]).resolve()
    out.mkdir(parents=True, exist_ok=True)
    fmn, kit = out / "libfmn.rlib", out / "libexplainer_kit.rlib"
    subprocess.run(
        ["rustc", "--edition", "2024", "--crate-type", "rlib", "--crate-name", "fmn", "-o", str(fmn),
         str(STUB / "fmn_stub.rs")], check=True, capture_output=True)
    subprocess.run(
        ["rustc", "--edition", "2024", "--crate-type", "rlib", "--crate-name", "explainer_kit",
         "--extern", f"fmn={fmn}", "-o", str(kit), str(STUB / "kit_stub.rs")], check=True, capture_output=True)
    trial = Trial(out)

    # The base's snapshots, which state_checks.py needs for its sensitivity half.
    base_tree = ROOT / "base" / PREFIX
    base_root = crate_root(out, "base", base_tree / "src/chapters")
    base_scene = out / "base_scene"
    subprocess.run(
        [*trial.rustc, "-o", str(base_scene), str(base_root)],
        check=True, capture_output=True, env=dict(os.environ, TREE_ROOT=str(base_tree)),
    )
    subprocess.run([str(base_scene), "2.68", str(out / "base-2.68.tsv")], check=True, capture_output=True)

    copy = out / "copy" / PREFIX
    shutil.copytree(ROOT / "candidate" / PREFIX, copy, dirs_exist_ok=True)
    chapter_dir = copy / "src/chapters"
    root = crate_root(out, "mutant", chapter_dir)
    originals = {c: (ROOT / "candidate" / PREFIX / f"src/chapters/{c}.rs").read_text() for c in CHAPTERS}

    # The unchanged copy sets the overlap baseline, and must itself be clean.
    caught, _ = trial.run(root, copy)
    trial.baseline = trial.seen_overlaps
    caught = [c for c in caught if c != "overlap"]
    print(f"unchanged copy: caught by {', '.join(caught) or 'nothing'}")
    print(f"{len(trial.baseline)} overlap lines in the baseline")
    surprises = len(caught)

    for name, chapter, before, after in MUTANTS:
        original = originals[chapter]
        if original.count(before) != 1:
            sys.exit(f"mutant {name!r}: pattern occurs {original.count(before)} times")
        (chapter_dir / f"{chapter}.rs").write_text(original.replace(before, after))
        caught, failed = trial.run(root, copy)
        (chapter_dir / f"{chapter}.rs").write_text(original)
        named = f" (claims on {', '.join(failed)})" if failed else ""
        print(f"{name}: caught by {', '.join(caught) or 'NOTHING'}{named}")
        if not caught:
            surprises += 1
    print(f"{len(MUTANTS)} mutants, {surprises} surprises")
    sys.exit(1 if surprises else 0)


main()
