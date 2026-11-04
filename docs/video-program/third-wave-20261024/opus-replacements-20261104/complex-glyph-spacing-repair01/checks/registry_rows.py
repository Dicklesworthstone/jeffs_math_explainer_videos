"""Where the changed strings should land in a new glyph registry. Not a probe run.

python3 -I -B registry_rows.py REGISTRY_TSV   (from any directory; reads only)

The probe keys rows by (TextKind, source) in a BTreeMap, so a row's number is its place in byte order within its kind
(crates/explainer_kit/src/glyphs.rs). This re-sorts the glyphs02 registry with the new strings. The control re-sorts
the old strings first and must reproduce the file's order, which proves the row decoding. The prediction holds only
if no other string has changed since glyphs02.
"""
import re
import sys
from pathlib import Path

CHANGES = {
    r"\frac{f'}{f} \approx \frac{m}{z - z_0}": r"\frac{f'}{f} \approx \frac{m}{z - z_{\,0}}",
    r"\frac{f(z_0 + h) - f(z_0)}{h}": r"\frac{f(z_{\,0} + h) - f(z_{\,0})}{h}",
    r"\lim_{h \to 0}\frac{f(z_0 + h) - f(z_0)}{h}": r"\lim_{h \to 0}\frac{f(z_{\,0} + h) - f(z_{\,0})}{h}",
    r"\oint (a + b\,(z - z_0))\,dz = 0": r"\oint (a + b\,(z - z_{\,0}))\,dz = 0",
    r"\times\, f'(z_0)": r"\times\, f'(z_{\,0})",
    r"\to\, f'(z_0)": r"\to\, f'(z_{\,0})",
    r"f(z) \approx a + b\,(z - z_0)": r"f(z) \approx a + b\,(z - z_{\,0})",
    r"z_0 = 1 + i": r"z_{\,0} = 1 + i",
    r"z_0^2 = 2i": r"z_{\,0}^2 = 2i",
    r"|z^5 + 1| \le 2 < 3 = |-3z|": r"|z^5 + 1| \le 2 < 3 = |{-3z}|",
}


def decode(quoted):
    """Rust's {:?} form of a str, back to the str."""
    body = quoted[1:-1]

    def one(m):
        tail = m.group(1)
        if tail.startswith("u{"):
            return chr(int(tail[2:-1], 16))
        return {"n": "\n", "t": "\t", "r": "\r", "\\": "\\", '"': '"', "'": "'", "0": "\0"}[tail]

    return re.sub(r"\\(u\{[0-9a-fA-F]+\}|.)", one, body)


def order(rows):
    kinds = []
    for kind, _ in rows:
        if kind not in kinds:
            kinds.append(kind)
    return sorted(rows, key=lambda r: (kinds.index(r[0]), r[1].encode("utf-8")))


def main():
    rows = []
    for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
        kind, _, quoted = line.split("\t", 2)
        rows.append((kind, decode(quoted)))
    print("rows", len(rows), "control: re-sorting the old strings reproduces the file:", order(rows) == rows)
    missing = [old for old in CHANGES if ("Tex", old) not in rows]
    print("old strings missing from this registry:", missing if missing else "none")
    stray = [s for k, s in rows if k == "Tex" and s not in CHANGES and ("z_0" in s or re.search(r"\|\s*[-\u2212]", s))]
    print("other Tex rows holding z_0 or a bar then a minus:", stray if stray else "none")
    new_rows = order([(k, CHANGES.get(s, s) if k == "Tex" else s) for k, s in rows])
    print("distinct after the change:", len(set(new_rows)) == len(new_rows))
    place = {row: i for i, row in enumerate(new_rows, 1)}
    moved = sum(1 for i, row in enumerate(rows, 1) if place[(row[0], CHANGES.get(row[1], row[1]) if row[0] == "Tex" else row[1])] != i)
    print("rows whose number changes, changed strings included:", moved)
    print("old_row\tnew_row\tnew_source")
    for i, (kind, source) in enumerate(rows, 1):
        if kind == "Tex" and source in CHANGES:
            print(f"{i}\t{place[(kind, CHANGES[source])]}\t{CHANGES[source]}")


if __name__ == "__main__":
    main()
