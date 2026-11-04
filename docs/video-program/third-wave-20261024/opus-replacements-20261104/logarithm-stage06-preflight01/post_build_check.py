#!/usr/bin/env python3
"""After cargo build: the real plan's counts, Rust's facts against the oracle, and the chapter list.

    python3 -I -B post_build_check.py --binary TARGET/debug/logarithm_mysteries [--repo ROOT]

Run from the repo root, after integration. A pass is a source and facts check. It says nothing about
pictures, takes or listening. The author has run this only against a stand-in binary in a temp copy.
"""
import argparse
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
CHAPTERS = ["00_ratio_ruler", "01_building_ruler", "02_slide_rule", "03_folded_decade", "04_undoing_growth", "05_complex_turns", "06_lifting_loops", "07_branch_cut", "08_product_rule", "09_matrix_logs", "10_doing_nothing", "11_unreachable", "12_what_it_forgets"]


def rows(text):
    pairs = [line.split("\t", 1) for line in text.splitlines() if line.strip()]
    result = dict(pairs)
    if len(result) != len(pairs):
        raise SystemExit("REFUSED duplicate fact keys")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--repo", default=".")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    crate = repo / "explainer_corpus/logarithm_mysteries"
    spec = importlib.util.spec_from_file_location("post_build_coherent", repo / "tools/coherent_narration.py")
    coherent = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = coherent
    spec.loader.exec_module(coherent)
    script, units = coherent.read_units(crate / "script.tsv", crate / "narration_units.json", require_directions=True)
    directions = coherent.direction_contract(crate / "script.tsv", json.loads((crate / "narration_units.json").read_text()))
    if not len(script) == len(units) == 80 or len(directions["resolved"]) != 15:
        raise SystemExit(f"REFUSED {len(script)} cues, {len(units)} units, {len(directions['resolved'])} directions; expected 80, 80, 15")
    print(f"Plan: {len(units)} units, {len(directions['resolved'])} directions. Source coverage only.")
    rust = rows(subprocess.check_output([args.binary, "facts"], text=True))
    oracle = rows(subprocess.check_output([sys.executable, "-I", "-B", str(crate / "oracle.py")], text=True))
    if rust != oracle:
        only = sorted(k for k in set(rust) ^ set(oracle))
        differ = sorted(k for k in set(rust) & set(oracle) if rust[k] != oracle[k])
        raise SystemExit(f"REFUSED facts differ. One side only: {only}. Different values: {differ}")
    print(f"Rust and the Python oracle agree on {len(rust)} facts.")
    listed = re.findall(r"\d\d_[a-z_]+", subprocess.check_output([args.binary, "chapters"], text=True))
    if listed != CHAPTERS:
        raise SystemExit(f"REFUSED the binary lists {listed}")
    print(f"The binary lists the {len(listed)} chapters, in order.")
    print("Still pending: glyph pages, new pictures, Oliver takes, ASR, listening, fine sync, final AV.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
