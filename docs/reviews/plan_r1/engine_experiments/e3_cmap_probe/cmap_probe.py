"""E3 of the engine-feasibility review: which bundled faces map the code points we plan to show.

A minimal TrueType cmap reader (formats 4 and 12), std library only.

Usage:
    python3 -I cmap_probe.py FONT.ttf [FONT.ttf ...]

The faces franken_manim bundles live in franken_markdown at the pinned rev, under
fmd-font/fonts/ (computer-modern/cmun*.ttf, ibm-plex-sans/IBMPlexSans-*.ttf,
noto-sans-math/NotoSansMathSymbols.ttf). A code point that no face maps cannot be set
by Text; in Tex, fmd-math falls back along its face chain, so a symbol found only in
NotoSansMathSymbols renders in that face's heavier sans style.
"""
import struct
import sys

PROBE = {
    0x2295: "⊕ oplus",
    0x227A: "≺ prec",
    0x2AAF: "⪯ preceq",
    0x230A: "⌊ lfloor",
    0x230B: "⌋ rfloor",
    0x2309: "⌉ rceil",
    0x03B1: "α alpha",
    0x03B8: "θ theta",
    0x2717: "✗ ballot x",
    0x2713: "✓ check",
    0x00D7: "× times",
    0x2248: "≈ approx",
    0x21D4: "⇔ Leftrightarrow",
    0x221E: "∞ infty",
    0x2261: "≡ equiv",
    0x22C5: "⋅ cdot",
    0x2192: "→ to",
    0x2208: "∈ in",
    0x2211: "∑ sum",
    0x220F: "∏ prod",
    0x221A: "√ sqrt",
    0x2260: "≠ neq",
    0x2264: "≤ leq",
    0x2026: "… ldots",
    0x2013: "– en dash",
    0x0151: "ő o double acute",
    0x2115: "ℕ naturals",
    0x0302: "◌̂ combining hat",
    0x207B: "⁻ superscript minus",
    0x00B9: "¹ superscript one",
}


def cmap_codepoints(path):
    with open(path, "rb") as f:
        data = f.read()
    num_tables = struct.unpack(">H", data[4:6])[0]
    tables = {}
    for i in range(num_tables):
        tag, _, off, length = struct.unpack(">4sIII", data[12 + 16 * i: 28 + 16 * i])
        tables[tag.decode("latin1")] = (off, length)
    off, _ = tables["cmap"]
    n = struct.unpack(">H", data[off + 2: off + 4])[0]
    cps = set()
    for i in range(n):
        _, _, sub = struct.unpack(">HHI", data[off + 4 + 8 * i: off + 12 + 8 * i])
        so = off + sub
        fmt = struct.unpack(">H", data[so: so + 2])[0]
        if fmt == 4:
            segx2 = struct.unpack(">H", data[so + 6: so + 8])[0]
            seg = segx2 // 2
            ends = struct.unpack(">%dH" % seg, data[so + 14: so + 14 + segx2])
            starts = struct.unpack(">%dH" % seg, data[so + 16 + segx2: so + 16 + 2 * segx2])
            deltas = struct.unpack(">%dh" % seg, data[so + 16 + 2 * segx2: so + 16 + 3 * segx2])
            ro_at = so + 16 + 3 * segx2
            ros = struct.unpack(">%dH" % seg, data[ro_at: ro_at + segx2])
            for k in range(seg):
                for c in range(starts[k], ends[k] + 1):
                    if c == 0xFFFF:
                        continue
                    if ros[k] == 0:
                        g = (c + deltas[k]) & 0xFFFF
                    else:
                        gi = ro_at + 2 * k + ros[k] + 2 * (c - starts[k])
                        g = struct.unpack(">H", data[gi: gi + 2])[0]
                        if g:
                            g = (g + deltas[k]) & 0xFFFF
                    if g:
                        cps.add(c)
        elif fmt == 12:
            ngroups = struct.unpack(">I", data[so + 12: so + 16])[0]
            for k in range(ngroups):
                s, e, _ = struct.unpack(">III", data[so + 16 + 12 * k: so + 28 + 12 * k])
                cps.update(range(s, e + 1))
    return cps


def main(paths):
    if not paths:
        sys.exit("usage: python3 -I cmap_probe.py FONT.ttf [FONT.ttf ...]")
    fonts = {p.replace("\\", "/").rsplit("/", 1)[-1]: cmap_codepoints(p) for p in paths}
    names = list(fonts)
    print("codepoint".ljust(28) + " ".join(n[:16].ljust(16) for n in names))
    for cp, label in PROBE.items():
        cells = " ".join(("yes" if cp in fonts[n] else "-").ljust(16) for n in names)
        print(f"U+{cp:04X} {label}".ljust(28) + cells)


if __name__ == "__main__":
    main(sys.argv[1:])
