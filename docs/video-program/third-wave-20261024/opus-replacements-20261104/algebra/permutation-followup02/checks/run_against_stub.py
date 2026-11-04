#!/usr/bin/env python3
"""Type-check and run the real candidate permutations.rs against stand-in crates.

Bare rustc, no Cargo, no fmn, no Kit. checks/stub holds hand-written stand-ins for
the calls the chapter makes. This finds type, borrow, panic and bookkeeping errors
in the scene code. It cannot find a wrong real signature, a glyph, or a bad frame.

Prints rustc's warnings, the scene file's tests, then the scene run at three
assumed speaking rates: the clock is an estimate, because no take exists.

Usage: python3 -I -B checks/run_against_stub.py SCRATCH_DIR
"""

import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
STUB = HERE / "stub"
RATES = ("2.68", "2.4", "3.0")


def rustc(*arguments):
    done = subprocess.run(["rustc", "--edition", "2024", *arguments], capture_output=True, text=True)
    print(done.stderr.strip() or "(rustc printed nothing)")
    if done.returncode:
        sys.exit(f"rustc failed: {done.returncode}")


def main():
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    fmn = out / "libfmn.rlib"
    kit = out / "libexplainer_kit.rlib"
    rustc("--crate-type", "rlib", "--crate-name", "fmn", "-o", str(fmn), str(STUB / "fmn_stub.rs"))
    rustc(
        "--crate-type", "rlib", "--crate-name", "explainer_kit",
        "--extern", f"fmn={fmn}", "-o", str(kit), str(STUB / "kit_stub.rs"),
    )
    links = ["-L", str(out), "--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}"]
    tests = out / "chapter_tests"
    print("-- rustc --test, candidate permutations.rs")
    rustc(*links, "--crate-name", "chapter", "--test", "-o", str(tests), str(STUB / "crate_root.rs"))
    print(subprocess.run([str(tests), "--test-threads=1"], capture_output=True, text=True, check=True).stdout.strip())
    scene = out / "chapter_scene"
    print("-- rustc, candidate permutations.rs")
    rustc(*links, "--crate-name", "chapter", "-o", str(scene), str(STUB / "crate_root.rs"))
    print("-- clippy-driver -D warnings, candidate permutations.rs, with and without tests")
    for extra in (["--test"], []):
        done = subprocess.run(
            ["clippy-driver", "--edition", "2024", *links, "--crate-name", "chapter", *extra,
             "-o", str(out / "clippy_output"), str(STUB / "crate_root.rs"), "-D", "warnings"],
            capture_output=True, text=True,
        )
        print(done.stderr.strip() or "(clippy printed nothing)")
        if done.returncode:
            sys.exit(f"clippy failed: {done.returncode}")
    for rate in RATES:
        print(f"-- scene run at {rate} words per second")
        print(subprocess.run([str(scene), rate], capture_output=True, text=True, check=True).stdout.strip())


main()
