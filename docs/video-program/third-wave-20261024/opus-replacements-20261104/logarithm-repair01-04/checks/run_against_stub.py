#!/usr/bin/env python3
"""Type-check and run chapters 01 to 04 against stand-in crates, for both trees.

Bare rustc and clippy-driver, no Cargo, no fmn, no real Kit. checks/stub holds
hand-written stand-ins for the calls the chapters make; facts.rs, model.rs and
draw.rs are the real baseline files, and the Kit's Facts is its real source file.

This finds type, borrow, panic and bookkeeping errors in the scene code, and it
measures overlaps on estimated text boxes. It cannot find a wrong real signature,
a TeX failure, a glyph or a bad frame.

Usage: python3 -I -B checks/run_against_stub.py SCRATCH_DIR [base|candidate ...]
Writes only under SCRATCH_DIR. Prints each tree's report and leaves its snapshot
rows in SCRATCH_DIR/<tree>-<rate>.tsv for checks/state_checks.py.
"""

import os
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
STUB = HERE / "stub"
PREFIX = "explainer_corpus/logarithm_mysteries"
RATES = ("2.68", "2.3", "3.1")
CHAPTERS = ("building_ruler", "slide_rule", "folded_decade", "undoing_growth")


def rustc(*arguments, tool="rustc", env=None):
    done = subprocess.run([tool, "--edition", "2024", *arguments], capture_output=True, text=True, env=env)
    if done.stderr.strip():
        print(done.stderr.strip())
    if done.returncode:
        sys.exit(f"{tool} failed: {done.returncode}")


def crate_root(out, tree):
    """A crate with the real baseline support files and this tree's four chapters."""
    support = ROOT / "base" / PREFIX / "src"
    chapter_dir = ROOT / tree / PREFIX / "src/chapters"
    lines = [
        # model.rs names methods add, sub and mul; that is baseline code, not repaired here.
        "#![allow(dead_code, clippy::should_implement_trait)]",
        f'#[path = "{support}/facts.rs"]',
        "pub mod facts;",
        f'#[path = "{support}/model.rs"]',
        "pub mod model;",
        "pub mod chapters {",
        f'    #[path = "{support}/chapters/draw.rs"]',
        "    pub mod draw;",
    ]
    for name in CHAPTERS:
        lines += [f'    #[path = "{chapter_dir}/{name}.rs"]', f"    pub mod {name};"]
    lines += ["}", f'include!("{STUB}/harness_main.rs");']
    path = out / f"{tree}_root.rs"
    path.write_text("\n".join(lines) + "\n")
    return path


def main():
    out = Path(sys.argv[1]).resolve()
    trees = sys.argv[2:] or ["base", "candidate"]
    out.mkdir(parents=True, exist_ok=True)
    fmn = out / "libfmn.rlib"
    kit = out / "libexplainer_kit.rlib"
    print("-- rustc, the stand-in crates")
    rustc("--crate-type", "rlib", "--crate-name", "fmn", "-o", str(fmn), str(STUB / "fmn_stub.rs"))
    rustc(
        "--crate-type", "rlib", "--crate-name", "explainer_kit",
        "--extern", f"fmn={fmn}", "-o", str(kit), str(STUB / "kit_stub.rs"),
    )
    links = ["-L", str(out), "--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}"]
    for tree in trees:
        env = dict(os.environ, TREE_ROOT=str(ROOT / tree / PREFIX))
        root = crate_root(out, tree)
        tests = out / f"{tree}_tests"
        print(f"\n######## {tree} ########")
        print("-- rustc --test")
        rustc(*links, "--crate-name", "chapter", "--test", "-o", str(tests), str(root), env=env)
        done = subprocess.run([str(tests), "--test-threads=1"], capture_output=True, text=True)
        print(done.stdout.strip())
        if done.returncode:
            print(f"TESTS FAILED in {tree}: {done.returncode}")
            if tree == "candidate":
                sys.exit(1)
        scene = out / f"{tree}_scene"
        print("-- rustc")
        rustc(*links, "--crate-name", "chapter", "-o", str(scene), str(root), env=env)
        if tree == "candidate":
            print("-- clippy-driver -D warnings, with and without tests")
            for extra in (["--test"], []):
                rustc(
                    *links, "--crate-name", "chapter", *extra, "-o", str(out / "clippy_output"), str(root),
                    "-D", "warnings", tool="clippy-driver", env=env,
                )
            print("(clippy clean)")
        for rate in RATES:
            print(f"-- scene run at {rate} words per second")
            run = subprocess.run(
                [str(scene), rate, str(out / f"{tree}-{rate}.tsv")], capture_output=True, text=True,
            )
            print(run.stdout.strip())
            if run.returncode:
                print(run.stderr.strip())
                sys.exit(f"{tree} scene failed at {rate}")


main()
