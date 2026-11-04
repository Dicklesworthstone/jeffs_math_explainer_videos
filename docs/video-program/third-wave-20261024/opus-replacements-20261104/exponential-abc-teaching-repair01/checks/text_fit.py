"""Plain-text extents from the pinned CMU font tables, and a rough upper estimate for TeX.

Usage from the repo root:  python3 -I -B text_fit.py
Text: advance in ems x size x 0.01 (fmn's own layout), from cmunrm.ttf at franken_markdown 126a530.
TeX: my own walk of the source, deliberately generous. It is an estimate; only the glyph probe and layout lint can
measure TeX. Frame is x +/-7.11, text margin 0.25.
"""
import os
import re
import struct

BASE = os.path.expanduser("~/.cargo/git/checkouts/franken_markdown-2f7478214ec48ef9/126a530/fmd-font/fonts/")
LIMIT = 7.11 - 0.25
fails = 0


def tables(data):
    count = struct.unpack(">H", data[4:6])[0]
    out = {}
    for i in range(count):
        tag, _, offset, length = struct.unpack(">4sIII", data[12 + 16 * i : 28 + 16 * i])
        out[tag.decode()] = (offset, length)
    return out


def cmap(data, table):
    offset = table["cmap"][0]
    _, count = struct.unpack(">HH", data[offset : offset + 4])
    best = None
    for i in range(count):
        _, _, sub = struct.unpack(">HHI", data[offset + 4 + 8 * i : offset + 12 + 8 * i])
        fmt = struct.unpack(">H", data[offset + sub : offset + sub + 2])[0]
        if fmt == 12:
            best = (12, offset + sub)
            break
        if fmt == 4 and best is None:
            best = (4, offset + sub)
    fmt, at = best
    mapping = {}
    if fmt == 12:
        groups = struct.unpack(">I", data[at + 12 : at + 16])[0]
        for g in range(groups):
            start, end, gid = struct.unpack(">III", data[at + 16 + 12 * g : at + 28 + 12 * g])
            for c in range(start, end + 1):
                mapping[c] = gid + c - start
    else:
        seg = struct.unpack(">H", data[at + 6 : at + 8])[0] // 2
        ends = struct.unpack(f">{seg}H", data[at + 14 : at + 14 + 2 * seg])
        starts = struct.unpack(f">{seg}H", data[at + 16 + 2 * seg : at + 16 + 4 * seg])
        deltas = struct.unpack(f">{seg}h", data[at + 16 + 4 * seg : at + 16 + 6 * seg])
        ranges_at = at + 16 + 6 * seg
        ranges = struct.unpack(f">{seg}H", data[ranges_at : ranges_at + 2 * seg])
        for i in range(seg):
            for c in range(starts[i], ends[i] + 1):
                if c == 0xFFFF:
                    continue
                if ranges[i] == 0:
                    gid = (c + deltas[i]) & 0xFFFF
                else:
                    where = ranges_at + 2 * i + ranges[i] + 2 * (c - starts[i])
                    gid = struct.unpack(">H", data[where : where + 2])[0]
                    gid = (gid + deltas[i]) & 0xFFFF if gid else 0
                if gid:
                    mapping[c] = gid
    return mapping


def font(path):
    data = open(path, "rb").read()
    table = tables(data)
    upem = struct.unpack(">H", data[table["head"][0] + 18 : table["head"][0] + 20])[0]
    metrics = struct.unpack(">H", data[table["hhea"][0] + 34 : table["hhea"][0] + 36])[0]
    at = table["hmtx"][0]
    advances = [struct.unpack(">H", data[at + 4 * i : at + 4 * i + 2])[0] for i in range(metrics)]
    return cmap(data, table), advances, upem


chars, advances, upem = font(BASE + "computer-modern/cmunrm.ttf")
math_chars, _, _ = font(BASE + "noto-sans-math/NotoSansMathSymbols.ttf")
plex_chars, _, _ = font(BASE + "ibm-plex-sans/IBMPlexSans-Regular.ttf")
print("coverage of the non-ASCII characters these scenes send to Text (fallback chain CM, Noto Sans Math, Plex):")
for ch in "≤×":
    faces = [("CM", chars), ("math", math_chars), ("Plex", plex_chars)]
    print(f"  {ch!r} U+{ord(ch):04X}: served by {next((n for n, t in faces if ord(ch) in t), 'NONE')}")


def width(text, size):
    total = 0
    for ch in text:
        gid = chars.get(ord(ch))
        total += upem * 0.7 if gid is None else advances[min(gid, len(advances) - 1)]
    return total / upem * size * 0.01


COLUMN = lambda k: -3.9 + 1.1 * k
TEXTS = [
    ("rate_law", "Next: construct an exact solution", 28.0, 0.0),
    ("factorials", "Where the factorials come from", 35.0, 0.0),
    ("factorials", "What must a be?", 27.0, COLUMN(2)),
    ("factorials", "function", 23.0, -5.35),
    ("factorials", "derivative", 23.0, -5.35),
    ("factorials", "denominator", 23.0, -5.35),
    ("factorials", "constant", 25.0, COLUMN(0)),
    ("factorials", "partial sums on fixed axes", 23.0, 0.0),
    ("factorials", "A finite polynomial loses its highest power", 25.0, 0.0),
    ("factorials", "The infinite tail needs a proof", 25.0, 0.0),
    ("factorials", "Does the sum exist?", 25.0, 0.0),
    ("factorials", "May we differentiate term by term?", 25.0, 0.0),
    ("factorials", "Uniform convergence on every bounded interval", 25.0, 0.0),
    ("factorials", "|x| ≤ 1.5, degree 4: tail ≤ 0.12657", 23.0, 3.9),
    ("factorials", "zoom ×16", 23.0, 5.4),
    ("factorials", "Why adding inputs multiplies outputs", 25.0, 0.0),
    ("factorials", "Why the stepped approximations reach E", 25.0, 0.0),
    ("factorials", "Why E is the only solution from height one", 25.0, 0.0),
    ("product_grid", "the inputs we have earned", 23.0, 0.0),
]
print("text extents (scene units):")
for scene, text, size, centre in TEXTS:
    w = width(text, size)
    left, right = centre - w / 2, centre + w / 2
    bad = left < -LIMIT or right > LIMIT
    fails += bad
    print(f"  {'OFF ' if bad else 'ok  '}{scene}: {w:5.2f} wide, {left:6.2f} to {right:5.2f}  {text!r}")

print("the two rows that sit on the table's 1.1-unit columns, neighbour by neighbour:")
for label, row, size in (
    ("equations", ["a = 1", "2b = 1", "3c = 1/2", "4d = 1/6"], 25.0),
    ("solved", ["a = 1", "b = 1/2", "c = 1/6", "d = 1/24"], 25.0),
):
    for a, b in zip(row, row[1:]):
        gap = 1.1 - width(a, size) / 2 - width(b, size) / 2
        bad = gap < 0.05
        fails += bad
        print(f"  {'TIGHT ' if bad else 'ok    '}{label}: {a!r} then {b!r}: gap {gap:.2f}")
for label, row, size, pitch in (
    ("ratio row", ["3/2", "3/4", "1/2", "3/8", "3/10", "1/4", "3/14", "3/16"], 23.0, 0.66),
    ("coefficients", ["1", "1", "1/2", "1/6", "1/24"], 25.0, 1.1),
):
    for a, b in zip(row, row[1:]):
        gap = pitch - width(a, size) / 2 - width(b, size) / 2
        bad = gap < 0.05
        fails += bad
        print(f"  {'TIGHT ' if bad else 'ok    '}{label}: {a!r} then {b!r}: gap {gap:.2f}")


SYMBOL = {
    "cdot": 0.78, "cdots": 1.67, "leq": 1.33, "geq": 1.33, "approx": 1.33, "infty": 1.0, "quad": 1.0, "div": 1.33,
    "sum": 1.45, "xrightarrow": 1.8, "left": 0.0, "right": 0.0,
}


def estimate(tex):
    """Ems, by a small recursive walk. Generous: fractions at full size, spaced relations."""
    i, total = 0, 0.0

    def group(j):
        depth = 0
        for k in range(j, len(tex)):
            depth += tex[k] == "{"
            depth -= tex[k] == "}"
            if depth == 0:
                return tex[j + 1 : k], k + 1
        raise ValueError(tex)

    while i < len(tex):
        ch = tex[i]
        if ch == "\\":
            m = re.match(r"\\([A-Za-z]+)", tex[i:])
            if not m:
                total += 0.3 if tex[i + 1 : i + 2] in (" ", ",") else 0.6
                i += 2
                continue
            name = m.group(1)
            i += len(m.group(0))
            if name in ("frac", "tfrac"):
                scale = 1.0 if name == "frac" else 0.75
                if tex[i : i + 1] == "{":
                    a, i = group(i)
                    b, i = group(i)
                    total += (max(estimate(a), estimate(b)) + 0.25) * scale
                else:
                    total += 0.75 * scale
                    i += 2
            elif name == "text":
                a, i = group(i)
                total += 0.5 * len(a)
            else:
                total += SYMBOL.get(name, 0.8)
        elif ch in "^_":
            i += 1
            if tex[i : i + 1] == "{":
                a, i = group(i)
                total += 0.75 * estimate(a)
            else:
                total += 0.75 * 0.55
                i += 1
        elif ch in "=+-<>":
            total += 1.33
            i += 1
        elif ch in "()|,./!":
            total += 0.45
            i += 1
        elif ch in "{} ":
            i += 1
        else:
            total += 0.58
            i += 1
    return total


TEXES = [
    ("rate_law", r"y'=y,\quad y(0)=1", 34.0, 2.75),
    ("factorials", r"y'=y,\quad y(0)=1", 30.0, 3.9),
    ("factorials", r"u_{k+1}=\frac{u_k}{k+1},\quad u_0=1", 28.0, 3.9),
    ("factorials", r"\frac{|x|}{k+1}\leq\frac{R}{k+1}\leq\frac12\quad(k+1\geq 2R)", 28.0, 3.9),
    ("factorials", r"\sum_{k=N+1}^{\infty}\frac{|x|^k}{k!}\leq\frac{R^{N+1}}{(N+1)!}\left(1+\frac12+\frac14+\cdots\right)", 25.0, 3.9),
    ("factorials", r"=\frac{2R^{N+1}}{(N+1)!}\quad(N+2\geq 2R)", 25.0, 3.9),
    ("factorials", r"\frac{d}{dx}\frac{x^k}{k!}=\frac{x^{k-1}}{(k-1)!}", 26.0, -3.0),
    ("factorials", r"E(x)=\sum_{k=0}^{\infty}\frac{x^k}{k!},\quad E'=E,\quad E(0)=1", 28.0, 0.0),
    ("factorials", r"y=E\ \text{solves}\ y'=y,\ y(0)=1", 26.0, 0.0),
    ("product_grid", r"e=E(1)=1+1+\frac12+\frac16+\cdots\approx 2.71828", 29.0, 0.0),
    ("product_grid", r"E(2)=e\cdot e,\quad E(3)=e^3", 28.0, 0.0),
    ("product_grid", r"E\left(\tfrac12\right)^2=e\ \text{and}\ E>0,\ \text{so}\ E\left(\tfrac12\right)=e^{1/2}", 28.0, 0.0),
    ("product_grid", r"E(-x)=\frac{1}{E(x)}", 28.0, 0.0),
    ("product_grid", r"E(x)=e^x", 42.0, 0.0),
]
print("TeX, ESTIMATED extents (a rough upper estimate, not a measurement):")
for scene, tex, size, centre in TEXES:
    w = estimate(tex) * size * 0.01
    left, right = centre - w / 2, centre + w / 2
    bad = left < -LIMIT or right > LIMIT
    fails += bad
    print(f"  {'OFF ' if bad else 'ok  '}{scene}: {w:5.2f} wide, {left:6.2f} to {right:5.2f}  {tex[:60]}")
print("bars and neighbours: the 02_5 right panel starts at about x =", round(3.9 - estimate(TEXES[4][1]) * 25 * 0.01 / 2, 2), "; the last main bar ends at -0.20")
print("RESULT", "PASS" if not fails else f"{fails} FLAGGED")
raise SystemExit(1 if fails else 0)
