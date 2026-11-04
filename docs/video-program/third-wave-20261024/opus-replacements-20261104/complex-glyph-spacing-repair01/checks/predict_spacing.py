"""What the pinned layout rules should do to these strings, computed from the bundled fonts. Not a render.

python3 -I -B predict_spacing.py FONTS_CM FONTS_NOTO ROWS_TSV   (stdlib only)

FONTS_CM   fmd-font/fonts/computer-modern in the franken_markdown checkout at rev 126a530
FONTS_NOTO fmd-font/fonts/noto-sans-math in the same checkout
ROWS_TSV   coverage/rows.tsv of the frozen complex-glyphs02-review01 (ink extents measured on the glyphs02 probe pages)

Every number is arithmetic on font tables and pinned rule values. A positive bounding-box gap means the two inks
cannot touch horizontally. A negative one only says they may: glyph shapes and heights decide that, which is why
the probe must still be inspected.
"""
import csv
import struct
import sys
from pathlib import Path

THIN, MED, THICK, NEG_THIN = 3, 4, 5, -3  # node.rs SpaceKind::mu
SCRIPT = 0.7  # style.rs size_factor
PX_PER_EM_PER_SIZE = 1.35  # 135 px per unit at 1080p, 1 em = size * 0.01 units
PROBE_TEX_SIZE = 36.0  # crates/explainer_kit/src/glyphs.rs font_size(36.0)


class Face:
    def __init__(self, path):
        self.data = Path(path).read_bytes()
        d = self.data
        count = struct.unpack_from(">H", d, 4)[0]
        self.tables = {}
        for i in range(count):
            tag, _, off, length = struct.unpack_from(">4sIII", d, 12 + 16 * i)
            self.tables[tag.decode("latin-1")] = (off, length)
        head = self.tables["head"][0]
        self.upm = struct.unpack_from(">H", d, head + 18)[0]
        self.long_loca = struct.unpack_from(">h", d, head + 50)[0] == 1
        self.n_h = struct.unpack_from(">H", d, self.tables["hhea"][0] + 34)[0]
        self.cmap = self._cmap()

    def _cmap(self):
        d = self.data
        base = self.tables["cmap"][0]
        mapped = {}
        for i in range(struct.unpack_from(">H", d, base + 2)[0]):
            pid, _, off = struct.unpack_from(">HHI", d, base + 4 + 8 * i)
            at = base + off
            fmt = struct.unpack_from(">H", d, at)[0]
            if fmt == 4 and pid in (0, 3):
                segs = struct.unpack_from(">H", d, at + 6)[0] // 2
                end, start = at + 14, at + 16 + 2 * segs
                delta, ranges = start + 2 * segs, start + 4 * segs
                for s in range(segs):
                    e = struct.unpack_from(">H", d, end + 2 * s)[0]
                    b = struct.unpack_from(">H", d, start + 2 * s)[0]
                    dl = struct.unpack_from(">h", d, delta + 2 * s)[0]
                    r = struct.unpack_from(">H", d, ranges + 2 * s)[0]
                    for c in range(b, e + 1):
                        if c == 0xFFFF:
                            continue
                        if r == 0:
                            g = (c + dl) & 0xFFFF
                        else:
                            g = struct.unpack_from(">H", d, ranges + 2 * s + r + 2 * (c - b))[0]
                            g = (g + dl) & 0xFFFF if g else 0
                        if g:
                            mapped[c] = g
            elif fmt == 12:
                for g in range(struct.unpack_from(">I", d, at + 12)[0]):
                    b, e, first = struct.unpack_from(">III", d, at + 16 + 12 * g)
                    for c in range(b, e + 1):
                        mapped[c] = first + c - b
        return mapped

    def metrics(self, ch):
        """advance, ink left, ink right, all in em."""
        d, gid = self.data, self.cmap[ord(ch)]
        hmtx = self.tables["hmtx"][0]
        adv = struct.unpack_from(">H", d, hmtx + 4 * min(gid, self.n_h - 1))[0]
        loca, glyf = self.tables["loca"][0], self.tables["glyf"][0]
        if self.long_loca:
            at = struct.unpack_from(">I", d, loca + 4 * gid)[0]
        else:
            at = struct.unpack_from(">H", d, loca + 2 * gid)[0] * 2
        x_min, _, x_max, _ = struct.unpack_from(">hhhh", d, glyf + at + 2)
        return adv / self.upm, x_min / self.upm, x_max / self.upm


def px(em, size):
    return em * size * PX_PER_EM_PER_SIZE


def main():
    cm, noto, rows_tsv = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
    italic, roman = Face(cm / "cmunti.ttf"), Face(cm / "cmunrm.ttf")
    symbols = Face(noto / "NotoSansMathSymbols.ttf")

    print("F1. z then subscript 0: base ink past its advance, against the digit's left bearing")
    z_adv, _, z_right = italic.metrics("z")
    over = z_right - z_adv
    _, zero_left, _ = roman.metrics("0")
    print(f"  z advance {z_adv:.4f} em, ink right {z_right:.4f} em, overhang {over:.4f} em (cmunti.ttf)")
    print(f"  0 left bearing {zero_left:.4f} em, at script size {zero_left * SCRIPT:.4f} em (cmunrm.ttf)")
    for label, mu in (("z_0 as it stands", 0), ("z_{\\,0} 3 mu, the patch", THIN), ("z_{\\;\\!0} 2 mu, not patched", THICK + NEG_THIN)):
        gap = mu / 18 * SCRIPT + zero_left * SCRIPT - over
        cells = "  ".join(f"size {s:g}: {px(gap, s):+.1f} px" for s in (28, 30, 32, 36))
        print(f"  {label:<30} added {mu / 18 * SCRIPT:.4f} em, box gap {gap:+.4f} em; at 1080p {cells}")

    print("\nCalibration: box gap against the gap measured on the glyphs02 probe (size 36; frozen touch_test.txt)")
    for label, base, face, sub, measured in (
        ("p_1 row 608", "p", roman, "1", "4 px apart (x469, x473), two components"),
        ("c_0 row 644", "c", roman, "0", "1 px apart (x73, x74), two components"),
        ("z_0 row 716", "z", roman, "0", "one component at levels 0.25, 0.5 and 0.75"),
    ):
        adv, _, right = italic.metrics(base)
        _, left, _ = face.metrics(sub)
        gap = left * SCRIPT - (right - adv)
        print(f"  {label}: box gap {gap:+.4f} em = {px(gap, PROBE_TEX_SIZE):+.1f} px; measured {measured}")

    print("\nF3. bar, minus, 3: atom glue, from Ord Bin Ord to Ord only (atom.rs classify_list)")
    bar_adv, _, bar_right = roman.metrics("|")
    face = "cmunrm.ttf" if 0x2212 in roman.cmap else "NotoSansMathSymbols.ttf"
    minus = (roman if 0x2212 in roman.cmap else symbols).metrics("−")
    _, three_left, _ = roman.metrics("3")
    print(f"  minus comes from {face}: advance {minus[0]:.4f}, ink {minus[1]:.4f} to {minus[2]:.4f} em")
    size = 34.0
    for label, mu in (("|-3z| as it stands", MED), ("|{-3z}| the patch", 0)):
        left = bar_adv - bar_right + mu / 18 + minus[1]
        right = minus[0] - minus[2] + mu / 18 + three_left
        print(f"  {label:<22} bar to minus {left:+.4f} em = {px(left, size):+.1f} px; "
              f"minus to 3 {right:+.4f} em = {px(right, size):+.1f} px (size 34, 1080p)")
    z_end = 2 * MED / 18
    print(f"  string narrows by {z_end:.4f} em = {z_end * size * 0.01:.4f} units at size 34")

    print("\nWidths and edges in the film. Ink width from the probe row, scaled to the call's size; placements are centred")
    rows = {}
    with open(rows_tsv, encoding="utf-8", newline="") as f:
        for r in csv.DictReader(f, delimiter="\t", quoting=csv.QUOTE_NONE):
            rows[int(r["registry_row"])] = int(r["right"]) - int(r["left"])
    # (name, registry row, film size, centre x or None when plane-relative, count of z_0, width change in mu per em)
    sites = [
        ("one_derivative QUOTIENT_TEX", 597, 36, 0.0, 2),
        ("one_derivative SETTLES_TEX", 638, 32, 2.75, 1),
        ("one_derivative TIMES_TEX", 637, 30, 3.3, 1),
        ("one_derivative z_0 = 1 + i", 716, 28, None, 1),
        ("one_derivative z_0^2 = 2i", 717, 28, None, 1),
        ("closing LIMIT_TEX", 605, 32, -2.4, 2),
        ("loop_sums LINEAR_TEX", 676, 28, 3.6, 1),
        ("loop_sums EACH_TEX", 612, 28, 3.6, 1),
        ("residues ZERO_TEX", 594, 32, -3.3, 1),
        ("counting LEASH_TEX", 723, 34, -3.45, 0),
    ]
    edges = {}
    for name, row, size, centre, count in sites:
        old = rows[row] / (PROBE_TEX_SIZE * PX_PER_EM_PER_SIZE) * size * 0.01
        delta = count * THIN / 18 * SCRIPT * size * 0.01 if count else -2 * MED / 18 * size * 0.01
        new = old + delta
        where = "plane-relative" if centre is None else f"edges {centre - new / 2:+.3f} to {centre + new / 2:+.3f}"
        if centre is not None:
            edges[name] = (centre - old / 2, centre + old / 2, centre - new / 2, centre + new / 2)
        print(f"  {name:<30} row {row} size {size}: {old:.3f} -> {new:.3f} units ({delta:+.3f}); {where}")
    a, b = edges["one_derivative QUOTIENT_TEX"], edges["one_derivative SETTLES_TEX"]
    print(f"  on one line at y=3.1, QUOTIENT_TEX to SETTLES_TEX gap {b[0] - a[1]:.3f} -> {b[2] - a[3]:.3f} units")
    print("  the frame's half width is 7.11 units; no edge above comes near it")


if __name__ == "__main__":
    main()
