#!/usr/bin/env python3
"""Script, storyboard and literal-caption bindings for the follow-up candidate.

1. Rows outside chapter 11 equal the base script, in the same places.
2. Every chapter 11 cue is at most 100 words and sits in the kit's id form.
3. storyboard-chapter11.md rejoins to script.tsv byte for byte, cue by cue.
4. Every caption the scene code writes as a literal number agrees with the
   independent oracle, and appears in the source exactly once or more.
5. Words the prediction beats promise stay out of earlier captions: the answer
   of each prediction appears in the source only after its hold.

Usage: python3 -I -B checks/script_and_captions.py
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
K = ROOT / "candidate/explainer_corpus/abstract_algebra_expanded"


def rows_of(path):
    rows = [line.split("\t") for line in path.read_text().splitlines()]
    if any(len(row) != 2 for row in rows):
        sys.exit(f"{path.name}: a row without exactly one tab")
    return rows


def main():
    failures = []
    script = rows_of(K / "script.tsv")
    base = rows_of(ROOT / "base/script.tsv.base")
    source = (K / "src/permutations.rs").read_text()
    oracle_lines = subprocess.run(
        [sys.executable, "-I", "-B", str(K / "oracle.py")], capture_output=True, text=True, check=True
    ).stdout.splitlines()
    oracle = dict(line.split("\t", 1) for line in oracle_lines)
    fact = lambda key: oracle["permutations." + key]

    # 1. Outside the chapter, nothing moved.
    outside = [row for row in script if not row[0].startswith("11_")]
    base_outside = [row for row in base if not row[0].startswith("11_")]
    if outside != base_outside:
        failures.append("rows outside chapter 11 differ from the base")
    first = next(i for i, row in enumerate(script) if row[0].startswith("11_"))
    count = sum(row[0].startswith("11_") for row in script)
    if not all(row[0].startswith("11_") for row in script[first : first + count]):
        failures.append("chapter 11 rows are not contiguous")
    if first != next(i for i, row in enumerate(base) if row[0].startswith("11_")):
        failures.append("chapter 11 starts on a different row")

    # 2. The limit the assembler enforces.
    cues = {cue: text for cue, text in script if cue.startswith("11_")}
    too_long = {cue: len(text.split()) for cue, text in cues.items() if len(text.split()) > 100}
    if too_long:
        failures.append(f"over 100 words: {too_long}")

    # 3. Storyboard words, rejoined.
    board = (ROOT / "storyboard-chapter11.md").read_text()
    told = {}
    for match in re.finditer(r"^## (11_\d+)\n\n\| s \|.*\n\|---.*\n((?:\|.*\n?)+)", board, re.MULTILINE):
        cells = [line.split(" | ") for line in match.group(2).strip().splitlines()]
        told[match.group(1)] = " ".join(line[1] for line in cells)
        indexes = [int(line[0].lstrip("| ")) for line in cells]
        if indexes != list(range(len(cells))):
            failures.append(f"{match.group(1)}: sentence numbers are not 0..n")
    if list(told) != list(cues):
        failures.append("storyboard cues are not the script's cues in script order")
    for cue, text in cues.items():
        if told.get(cue) != text:
            failures.append(f"{cue}: storyboard words differ from script.tsv")

    # 4. Literal captions against the oracle.
    deck = fact("deck_images").split(",")
    out_lengths = fact("full_out_lengths").split(",")
    three = fact("loop_three_home").split(",")
    pair = fact("loop_pair_home").split(",")
    four = fact("second_four_home").split(",")
    second_pair = fact("second_pair_home").split(",")
    literal = [
        (
            "Top half lands on 1, 3, 5, 7, 9. Bottom half lands on 2, 4, 6, 8, 10.",
            deck[:5] == ["1", "3", "5", "7", "9"] and deck[5:] == ["2", "4", "6", "8", "10"],
        ),
        ("Positions 1 and 10 stay", deck[0] == "1" and deck[9] == "10"),
        (
            "2 fixed, 1 swap, 6 cycles of length 8",
            out_lengths.count("1") == 2 and out_lengths.count("2") == 1 and out_lengths.count("8") == 6,
        ),
        ("All ten home", fact("deck_order") == "6" and len(deck) == 10),
        ("4 swaps, 2 swaps: both even", fact("even_lists_final") == fact("short_route_final")),
        ("3 + 2 = 5: neither loop is home", "5" not in three and "5" not in pair),
        (
            "8 is a later return. 6 is no return.",
            "8" in four and "8" in second_pair and not ("6" in four and "6" in second_pair),
        ),
        ("Without switches, stage 4 repeats stage 1", fact("wire_order") == "3"),
    ]
    for caption, holds in literal:
        if caption not in source:
            failures.append(f"caption not in the source: {caption!r}")
        if not holds:
            failures.append(f"caption disagrees with the oracle: {caption!r}")

    # 5. No answer on screen before its hold, in source order inside each scene.
    before_hold = [
        ("fn loops_question", "fn loops_run", ["lcm", "order", "first_return"]),
        ("fn second_question", "fn second_run", ["lcm", "first_return", "order"]),
        ("fn shuffle_question", "fn walk_cycle", ["lcm", "first_return", "order_by_cycles", "moving_cycles"]),
    ]
    for start, end, words in before_hold:
        body = source[source.index(start) : source.index(end)]
        drawn = [line for line in body.splitlines() if not line.strip().startswith("//")]
        for word in words:
            # The word "clock" caption mentions order on purpose; it states no value.
            hits = [line for line in drawn if word in line and "first return = order" not in line]
            if hits:
                failures.append(f"{start}: {word!r} before the hold: {hits[0].strip()}")

    print(f"{len(cues)} chapter 11 cues, {sum(len(t.split()) for t in cues.values())} words, longest {max(len(t.split()) for t in cues.values())}")
    print(f"{len(outside)} rows outside chapter 11 equal the base")
    print(f"{len(told)} storyboard cues rejoin to script.tsv")
    print(f"{len(literal)} literal captions agree with the oracle")
    print(f"{len(before_hold)} prediction scenes draw no answer before the hold")
    for failure in failures:
        print("FAIL", failure)
    print(f"{len(failures)} failures")
    sys.exit(1 if failures else 0)


main()
