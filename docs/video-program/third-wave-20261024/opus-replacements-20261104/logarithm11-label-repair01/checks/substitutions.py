"""Apply the three whole-line replacements to a copy of the chapter. Never writes under the repo.

python3 -I -B substitutions.py SOURCE_FILE OUTPUT_FILE

Each replacement names its line number and the exact old line, and stops if either has moved. The line count is
asserted, so every other line number in the file, and every source-line reference to it, stays valid.
"""
import sys
from pathlib import Path

CHANGES = [
    (
        1309,
        r'''            r"e^{tp} > 0 \text{ for all real } t \text{ and } p\text{: no axis reverses}",''',
        r'''            r"e^{tp} > 0 \text{ for real } t \text{ and } p\text{: } e^{tX} \text{ reverses no axis}",''',
    ),
    (
        1662,
        r'''            r"\operatorname{det} A = 0 \quad\text{but}\quad \operatorname{det} e^{X} = e^{\operatorname{tr} X} \neq 0",''',
        r'''            r"\text{singular } S\text{: } \operatorname{det} S = 0 \quad\text{but}\quad \operatorname{det} e^{X} = e^{\operatorname{tr} X} \neq 0",''',
    ),
    (
        1910,
        r'''            ("etp > 0 for all real t and p: no axis reverses", 22.0, 2.2),''',
        r'''            ("etp > 0 for real t and p: etX reverses no axis", 22.0, 2.2),''',
    ),
]
LINES = 2305


def main():
    source, output = Path(sys.argv[1]), Path(sys.argv[2])
    text = source.read_bytes().decode("utf-8")
    lines = text.split("\n")
    if lines[-1] != "" or len(lines) - 1 != LINES:
        raise SystemExit(f"expected {LINES} newline-ended lines, found {len(lines) - 1}")
    for at, old, new in CHANGES:
        if lines[at - 1] != old:
            raise SystemExit(f"line {at} is not the expected old line:\n{lines[at - 1]}")
        if text.count(old) != 1:
            raise SystemExit(f"old line {at} occurs {text.count(old)} times")
        lines[at - 1] = new
    output.write_bytes("\n".join(lines).encode("utf-8"))
    print(f"wrote {len(CHANGES)} replaced lines, {LINES} lines in all")


if __name__ == "__main__":
    main()
