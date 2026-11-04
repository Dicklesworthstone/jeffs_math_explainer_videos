#!/usr/bin/env python3
"""Compile and run the chapter's pure arithmetic with bare rustc, no Cargo.

Only the marked pure section of candidate permutations.rs, the helpers it needs,
`listed` from main.rs and `mod repetition_tests` are extracted, byte for byte.
Scene code, fmn and the Kit are not compiled here, so a pass says nothing about
them. The printed facts are then compared with the independent oracle.

Usage: python3 -I checks/pure_rust_arithmetic.py SCRATCH_DIR
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
K = ROOT / "candidate/explainer_corpus/abstract_algebra_expanded"
HELPERS = [
    "cycle_images",
    "then",
    "cards_by_slot",
    "swaps_needed",
    "parity_word",
    "cycle_tex",
    "arrangements",
    "one_based",
    "cell_text",
    "way_home",
]


def braced(source, start):
    """The text from `start` through the brace that closes its first brace."""
    depth, at = 0, source.index("{", start)
    while True:
        if source[at] == "{":
            depth += 1
        elif source[at] == "}":
            depth -= 1
            if depth == 0:
                return source[start : at + 1]
        at += 1


def named_fn(source, name):
    found = re.search(rf"^(?:pub\(crate\) )?fn {name}\b", source, re.MULTILINE)
    if not found:
        sys.exit(f"missing fn {name}")
    return braced(source, found.start())


def run(command):
    return subprocess.run(command, check=True, capture_output=True, text=True).stdout


def main():
    scratch = Path(sys.argv[1])
    scratch.mkdir(parents=True, exist_ok=True)
    source = (K / "src/permutations.rs").read_text()
    main_rs = (K / "src/main.rs").read_text()

    begin = source.index("// ---- repetition, parity and shuffles: pure arithmetic")
    end = source.index("// ---- end of pure arithmetic")
    tests = braced(source, source.index("mod repetition_tests"))
    parts = [
        "#![allow(dead_code)]",
        named_fn(main_rs, "listed"),
        *(named_fn(source, name) for name in HELPERS),
        source[begin:end],
    ]

    test_file = scratch / "pure_tests.rs"
    test_file.write_text("\n\n".join(parts + ["#[cfg(test)]\n" + tests]) + "\n")
    test_bin = scratch / "pure_tests"
    run(["rustc", "--edition", "2024", "--test", str(test_file), "-o", str(test_bin)])
    report = run([str(test_bin), "--test-threads=1"])
    print(report.strip())

    facts_file = scratch / "pure_facts.rs"
    printer = 'fn main() {\n    for (key, value) in repetition_facts() {\n        println!("{key}\\t{value}");\n    }\n}'
    facts_file.write_text("\n\n".join(parts + [printer]) + "\n")
    facts_bin = scratch / "pure_facts"
    run(["rustc", "--edition", "2024", str(facts_file), "-o", str(facts_bin)])
    rust = dict(line.split("\t", 1) for line in run([str(facts_bin)]).splitlines())

    lines = run([sys.executable, "-I", "-B", str(K / "oracle.py")]).splitlines()
    oracle = dict(line.split("\t", 1) for line in lines)
    if len(oracle) != len(lines):
        sys.exit("the oracle prints a key twice")

    wrong = [key for key, value in rust.items() if oracle.get(key) != value]
    for key in wrong:
        print(f"MISMATCH {key}: rust {rust[key]!r} oracle {oracle.get(key)!r}")
    print(f"{len(rust)} Rust repetition facts; {len(lines)} oracle lines; {len(wrong)} disagree")
    if wrong:
        sys.exit(1)
    for key, value in rust.items():
        print(f"{key}\t{value}")


main()
