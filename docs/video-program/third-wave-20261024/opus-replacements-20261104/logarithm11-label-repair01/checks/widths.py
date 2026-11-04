"""Measure the candidate strings with the chapter's own width model. Reads only.

python3 -I -B widths.py CHAPTER_RS

The table comes from the match arms of em() in the test module of the file given, so it cannot drift from the source.
Typeset lines are flattened the way the test's mirrors flatten them, with exponents at full size.
"""
import re
import sys
from pathlib import Path


def em_table(rs):
    body = rs[rs.index("fn em(c: char) -> f64 {"):]
    body = body[:body.index("\n    }\n")]
    table = {}
    for line in body.splitlines():
        m = re.match(r"\s*(.*?) => ([0-9.]+),$", line)
        if not m or m.group(1) == "_":
            continue
        pattern, value = m.group(1), float(m.group(2))
        if "'0'..='9'" in pattern:
            for d in "0123456789":
                table[d] = value
            pattern = pattern.replace("'0'..='9'", "")
        for c in re.findall(r"'(.)'", pattern):
            table[c] = value
    return table


def main():
    table = em_table(Path(sys.argv[1]).read_text(encoding="utf-8"))

    def width(s, size):
        return sum(table.get(c, 0.75) for c in s) * size * 0.01

    limit = 7.11 - 0.25
    print("Left column, x = 2.2, size 22; the test's assertion: x + width * 1.03 < 6.86")
    for label, s in [
        ("baseline positive", "etp > 0 for all real t and p: no axis reverses"),
        ("neighbour axes_word", "etX stretches the axes by etp and etq"),
        ("neighbour stage_word", "a half turn, then a stretch: two flows in turn"),
        ("CHOSEN 'for real'", "etp > 0 for real t and p: etX reverses no axis"),
        ("rejected 'for all real'", "etp > 0 for all real t and p: etX reverses no axis"),
        ("rejected 'always'", "etp > 0 always, so etX reverses no axis"),
        ("rejected 'never reverses'", "etp > 0 for real t and p: etX never reverses an axis"),
    ]:
        w = width(s, 22.0)
        end = 2.2 + w * 1.03
        print(f"  {label:<26} {len(s):>3} chars  width {w:5.3f}  checked end {end:5.3f}  {'pass' if end < limit else 'FAIL'}  {s}")

    print()
    print("Closing, centred at x = 0, size 28; frame half width 7.11; read with like_zero at 15 characters a second")
    like_zero = "flattening matrices, like zero, have no logarithm"
    for label, s in [
        ("baseline", "det A = 0 but det eX = etr X ≠ 0"),
        ("CHOSEN singular S", "singular S: det S = 0 but det eX = etr X ≠ 0"),
        ("fallback letter only", "det S = 0 but det eX = etr X ≠ 0"),
        ("rejected conditional", "if det S = 0, S is no eX: det eX = etr X ≠ 0"),
        ("rejected in words", "a flattening matrix: det = 0 but det eX = etr X ≠ 0"),
    ]:
        w = width(s, 28.0)
        chars = len(s) + len(like_zero)
        print(f"  {label:<22} {len(s):>3} chars  width {w:5.3f}  half {w / 2:5.3f}  with like_zero {chars:>3} chars = {chars / 15:4.2f} s  {s}")
    print(f"  like_zero alone, size 22: {len(like_zero)} chars, width {width(like_zero, 22.0):5.3f}")


if __name__ == "__main__":
    main()
