"""Width test mirror, TeX commands, glyph coverage and predicted registry rows. Reads only. Not a build or a probe.

python3 -I -B mirror_and_glyphs.py BASELINE_RS PATCHED_RS REGISTRY_TSV FONTS_DIR

FONTS_DIR is fmd-font's fonts/computer-modern at franken_markdown rev 126a530. The registry is the root's
logarithm-combined-repair-glyphs01. Row numbers follow crates/explainer_kit/src/glyphs.rs: a BTreeMap keyed by
(TextKind, source), so a row's number is its place in byte order within its kind.
"""
import re
import struct
import sys
from pathlib import Path

OLD = {
    r"\operatorname{det} A = 0 \quad\text{but}\quad \operatorname{det} e^{X} = e^{\operatorname{tr} X} \neq 0":
    r"\text{singular } S\text{: } \operatorname{det} S = 0 \quad\text{but}\quad \operatorname{det} e^{X} = e^{\operatorname{tr} X} \neq 0",
    r"e^{tp} > 0 \text{ for all real } t \text{ and } p\text{: no axis reverses}":
    r"e^{tp} > 0 \text{ for real } t \text{ and } p\text{: } e^{tX} \text{ reverses no axis}",
}
MIRROR_NEW = "etp > 0 for real t and p: etX reverses no axis"
TEXT_RIGHT = 7.11 - 0.25
ROOM = 1.03


def em_table(rs):
    """Read the match arms of the test module's em() from the source, so the table cannot drift from it."""
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


def width(table, s, size):
    return sum(table.get(c, 0.75) for c in s) * size * 0.01


def rows_of(rs):
    block = rs[rs.index("fn the_panels_and_text_column_fit()"):]
    block = block[:block.index("] {\n            assert!(x + width(s, size) * WIDTH_ROOM < TEXT_RIGHT")]
    return [(m[0], float(m[1]), m[2]) for m in re.findall(r'\("([^"]+)", ([0-9.]+), ([^)]+)\),', block)]


def check_mirrors(label, table, rs):
    worst = 0.0
    failed = []
    rows = rows_of(rs)
    for s, size, x_expr in rows:
        x = 2.2 + 0.85 if "KEY_WORD_X" in x_expr else float(x_expr)
        end = x + width(table, s, size) * ROOM
        worst = max(worst, end)
        if not end < TEXT_RIGHT:
            failed.append(s)
    print(f"{label}: {len(rows)} width rows, largest checked right edge {worst:.3f} against {TEXT_RIGHT:.2f}; failures: {failed if failed else 'none'}")
    return rows


def commands(s):
    return sorted(set(re.findall(r"\\[A-Za-z]+", s)))


def cmap_of(font):
    num = struct.unpack_from(">H", font, 4)[0]
    tables = {}
    for i in range(num):
        tag, _, off, length = struct.unpack_from(">4sIII", font, 12 + 16 * i)
        tables[tag.decode("latin-1")] = (off, length)
    base = tables["cmap"][0]
    count = struct.unpack_from(">H", font, base + 2)[0]
    mapped = set()
    for i in range(count):
        _, _, off = struct.unpack_from(">HHI", font, base + 4 + 8 * i)
        at = base + off
        fmt = struct.unpack_from(">H", font, at)[0]
        if fmt == 12:
            groups = struct.unpack_from(">I", font, at + 12)[0]
            for g in range(groups):
                b, e, _ = struct.unpack_from(">III", font, at + 16 + 12 * g)
                mapped.update(range(b, e + 1))
        elif fmt == 4:
            segs = struct.unpack_from(">H", font, at + 6)[0] // 2
            end = at + 14
            start = end + 2 * segs + 2
            for s in range(segs):
                e = struct.unpack_from(">H", font, end + 2 * s)[0]
                b = struct.unpack_from(">H", font, start + 2 * s)[0]
                mapped.update(c for c in range(b, e + 1) if c != 0xFFFF)
    return mapped


def decode(quoted):
    body = quoted[1:-1]

    def one(m):
        tail = m.group(1)
        if tail.startswith("u{"):
            return chr(int(tail[2:-1], 16))
        return {"n": "\n", "t": "\t", "r": "\r", "\\": "\\", '"': '"', "'": "'", "0": "\0"}[tail]

    return re.sub(r"\\(u\{[0-9a-fA-F]+\}|.)", one, body)


def order(rows):
    kinds = []
    for kind, _, _ in rows:
        if kind not in kinds:
            kinds.append(kind)
    return sorted(rows, key=lambda r: (kinds.index(r[0]), r[2].encode("utf-8")))


def main():
    base_rs = Path(sys.argv[1]).read_text(encoding="utf-8")
    new_rs = Path(sys.argv[2]).read_text(encoding="utf-8")
    registry = Path(sys.argv[3])
    fonts = Path(sys.argv[4])

    print("--- width assertion of the_panels_and_text_column_fit, mirrored")
    table = em_table(new_rs)
    print("em table read from the source:", len(table), "characters;", "same in both files:", em_table(base_rs) == table)
    check_mirrors("baseline", table, base_rs)
    rows = check_mirrors("patched", table, new_rs)
    print("new mirror row present:", any(s == MIRROR_NEW for s, _, _ in rows))
    print("old mirror row gone:", not any("no axis reverses" in s for s, _, _ in rows))
    for label, s in [("new positive", MIRROR_NEW), ("rejected, keeping 'all real'", "etp > 0 for all real t and p: etX reverses no axis")]:
        end = 2.2 + width(table, s, 22.0) * ROOM
        print(f"  {label}: checked right edge {end:.3f}, {'passes' if end < TEXT_RIGHT else 'fails'}")

    print("--- TeX commands, old against new string")
    for old, new in OLD.items():
        print(f"  old {commands(old)}")
        print(f"  new {commands(new)}")
        print("  new command introduced:", sorted(set(commands(new)) - set(commands(old))) or "none")

    print("--- glyph coverage, pinned faces")
    italic = cmap_of((fonts / "cmunti.ttf").read_bytes())
    roman = cmap_of((fonts / "cmunrm.ttf").read_bytes())
    print("  math letter S in cmunti.ttf (the face FACE_ITALIC math letters use):", ord("S") in italic)
    print("  math letters of the new strings in cmunti.ttf:", {c: ord(c) in italic for c in "SXept"})
    for words in ("singular :", " for real  and : reverses no axis"):
        print(f"  text {words!r} in cmunrm.ttf, all present:", all(ord(c) in roman for c in words))

    print("--- registry rows")
    rows = []
    lines = registry.read_text(encoding="utf-8").splitlines()
    for line in lines:
        kind, chapter, quoted = line.split("\t", 2)
        rows.append((kind, chapter, decode(quoted)))
    print("rows", len(rows), "; control, re-sorting the old strings reproduces the file:", order(rows) == rows)
    stripped = [re.sub(r"\\[A-Za-z]+", "", s) if k == "Tex" else s for k, _, s in rows]
    print("rows in the whole film using S as a standalone letter:", sum(1 for s in stripped if re.search(r"(?<![A-Za-z])S(?![A-Za-z])", s)),
          "; control, rows using X the same way:", sum(1 for s in stripped if re.search(r"(?<![A-Za-z])X(?![A-Za-z])", s)))
    for old in OLD:
        at = [i + 1 for i, r in enumerate(rows) if r[2] == old]
        print(f"  old row {at}, chapter {[rows[i - 1][1] for i in at]}: {old[:60]}")
    new_rows = order([(k, c, OLD.get(s, s)) for k, c, s in rows])
    print("rows after the change", len(new_rows), "; distinct:", len({(k, s) for k, _, s in new_rows}) == len(new_rows))
    for new in OLD.values():
        at = [i + 1 for i, r in enumerate(new_rows) if r[2] == new]
        print(f"  new row {at}: {new[:70]}")
    moved = [(i + 1, a[2][:50]) for i, (a, b) in enumerate(zip(rows, new_rows)) if a[2] != b[2]]
    print("row numbers whose string differs from the old registry:", len(moved), "; first", moved[0][0], "; last", moved[-1][0])
    unchanged_text = [r for r in rows if r[0] == "Text"] == [r for r in new_rows if r[0] == "Text"]
    print("Text rows unchanged:", unchanged_text)


if __name__ == "__main__":
    main()
