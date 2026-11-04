#!/usr/bin/env python3
"""Run from the repo root. Every mobject the bundle patch adds: where it is made, shown, removed or handed back.

Text search over the frozen candidate. Read together with the source; it does not run anything.
"""
import re

B = "docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-picture-review-11_permutations/"
lines = open(B + "candidate/permutations.rs", encoding="utf-8").read().splitlines()


def uses(name, first, last):
    pattern = re.compile(r"\b" + name + r"\b")
    return [(i + 1, row.strip()[:88]) for i, row in enumerate(lines) if first <= i + 1 <= last and pattern.search(row)]


def span(needle):
    start = next(i for i, row in enumerate(lines) if row.startswith(needle))
    end = next(i for i in range(start + 1, len(lines)) if lines[i] == "}")
    return start + 1, end + 1


for owner, needle, names in [
    ("11_20 even_subgroup", "fn even_subgroup", ["row"]),
    ("11_21 twelve_of_twenty_four", "fn twelve_of_twenty_four", ["even_key", "odd_key"]),
    ("11_22 read_the_cycles", "fn read_the_cycles", ["reading"]),
    ("11_26 loop_and_wires", "fn loop_and_wires", ["fourth"]),
]:
    first, last = span(needle)
    print(f"== {owner} ({first}-{last})")
    for name in names:
        for at, row in uses(name, first, last):
            print(f"  {name} :{at} {row}")

say5 = next(i for i, row in enumerate(lines) if 'k.say(stage, "11_5")' in row)
say6 = next(i for i, row in enumerate(lines) if 'k.say(stage, "11_6")' in row)
print(f"== 11_5 in construct ({say5 + 1}-{say6 + 1})")
for name in ("swap_name", "swap_cards"):
    for at, row in uses(name, say5 + 1, say6 + 1):
        print(f"  {name} :{at} {row}")

print("== callers that clear what the scenes hand back")
for needle in ("let test = even_subgroup", "fade(stage, &test, false)", "let half = twelve_of_twenty_four", "fade(stage, &half, false)", "let wires = loop_and_wires", "fade(stage, &wires, false)"):
    print(" ", needle, "->", [i + 1 for i, row in enumerate(lines) if needle in row])
print("== the cell colours are text, and recolor already ran on these same cells in baseline sentence 5")
print(" ", [i + 1 for i, row in enumerate(lines) if "recolor(stage, &" in row and any(w in row for w in ("left_cells", "right_cells", "nothing", "doubles"))])
print("== AD's two pulse targets in full_deck")
first, last = span("fn full_deck")
for at, row in uses("notes", first, last):
    print(f"  :{at} {row}")
