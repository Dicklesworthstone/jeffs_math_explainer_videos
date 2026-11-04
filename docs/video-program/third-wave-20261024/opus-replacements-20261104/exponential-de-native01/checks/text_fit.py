"""Static extents and same-function overlaps for the new scenes' text, from the pinned CMU font tables.

Usage from the repo root:  python3 -I -B text_fit.py
Plain text: advance in ems x size x 0.01 (fmn's own layout), from cmunrm.ttf at franken_markdown 126a530.
TeX: my own walk of the source, deliberately generous; only the glyph probe and layout lint can measure it.
Calls are read from the source by regular expression, so an expression I cannot evaluate is printed as UNRESOLVED and
falls to the lint. Overlaps are per function, a superset of what shares the screen: anything printed needs a person's
read, and the read is recorded in HANDOFF.md. No pixels are involved in any of this.
"""
import os
import re
import struct

BASE = os.path.expanduser("~/.cargo/git/checkouts/franken_markdown-2f7478214ec48ef9/126a530/fmd-font/fonts/")
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCENES = {"tiny_multipliers": HERE + "/candidate/limit01/tiny_multipliers.rs", "only_solution": HERE + "/candidate/only01/only_solution.rs"}
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


def width(text, size):
    total = 0
    for ch in text:
        gid = chars.get(ord(ch))
        total += upem * 0.7 if gid is None else advances[min(gid, len(advances) - 1)]
    return total / upem * size * 0.01


SYMBOL = {
    "cdot": 0.78, "cdots": 1.67, "leq": 1.33, "geq": 1.33, "approx": 1.33, "infty": 1.0, "quad": 1.0, "qquad": 2.0,
    "sum": 1.45, "prod": 1.45, "left": 0.0, "right": 0.0, "neq": 1.33, "ne": 1.33, "iff": 2.2, "Rightarrow": 1.6,
    "Leftrightarrow": 1.6, "to": 1.2, "lim": 1.45, "times": 0.78, "ldots": 1.2, "big": 0.0,
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
            if name in ("frac", "tfrac", "dfrac", "binom"):
                scale = 0.75 if name == "tfrac" else 1.0
                if tex[i : i + 1] == "{":
                    a, i = group(i)
                    b, i = group(i)
                    total += (max(estimate(a), estimate(b)) + 0.25) * scale
                else:
                    total += 0.75 * scale
                    i += 2
            elif name in ("text", "mathrm"):
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


def tex_height(tex, size):
    tall = 2.1 if re.search(r"\\(frac|binom|sum|prod|lim)", tex) else 1.15
    return tall * size * 0.01


CALL = re.compile(
    r"k\.(text|tex)\(\s*stage,\s*(.+?),\s*([\d.]+),\s*(?:Palette::\w+\(\)|color),\s*(?:&\[\],\s*)?v\(\s*(.+?),\s*([^()]+?(?:\([^()]*\)[^()]*?)*),?\s*\),?\s*\)\?",
    re.S,
)


def constants(code):
    out = {"RIGHT_X": None}
    for name, value in re.findall(r"^const (\w+): f64 = ([^;]+);", code, re.M):
        try:
            out[name] = float(eval(value, {"__builtins__": {}}, {k: v for k, v in out.items() if v is not None}))
        except Exception:
            out[name] = None
    return out


def strings(code):
    out = {}
    for name, raw in re.findall(r'^const (\w+): &str =\s*r#?"(.*?)"#?;', code, re.M | re.S):
        out[name] = raw
    for name, raw in re.findall(r'^const (\w+): &str =\s*"(.*?)";', code, re.M | re.S):
        out[name] = raw
    return out


def number(expr, env):
    expr = expr.strip()
    try:
        return float(eval(expr, {"__builtins__": {}}, {k: v for k, v in env.items() if v is not None}))
    except Exception:
        return None


# Printed values the sources build with format!, so the widths below are the strings a viewer gets.
DYNAMIC = [
    ("tiny_multipliers", "tail after 4 ≤ 0.04: too big", 23.0, 3.75, 0.45),
    ("tiny_multipliers", "1024 cells: head ≤ 0.00492", 23.0, 3.75, -0.67),
    ("tiny_multipliers", "total ≤ 0.00514 < 0.01", 25.0, 3.75, -1.15),
    ("tiny_multipliers", "1024 cells, total ≤ 0.00514", 23.0, 3.75, -0.1),
    ("tiny_multipliers", "column 100: 0.00000", 23.0, 4.6, 2.25),
]
print("coverage of the non-ASCII characters these scenes send to Text (fallback chain CM, Noto Sans Math, Plex):")
for ch in "≤×′":
    faces = [("CM", chars), ("math", math_chars), ("Plex", plex_chars)]
    print(f"  {ch!r} U+{ord(ch):04X}: served by {next((n for n, t in faces if ord(ch) in t), 'NONE')}")

for scene, path in SCENES.items():
    code = open(path, encoding="utf-8").read().split("#[cfg(test)]")[0]
    env = constants(code)
    words = strings(code)
    print(f"\n=== {scene}")
    functions = [(m.group(1), m.start()) for m in re.finditer(r"^fn (\w+)\(", code, re.M)]
    functions.append(("<end>", len(code)))
    unresolved = 0
    boxes = {}
    for (fn, at), (_, end) in zip(functions, functions[1:]):
        body = code[at:end]
        for m in CALL.finditer(body):
            kind, source, size, x_expr, y_expr = m.groups()
            size = float(size)
            source = source.strip()
            literal = re.fullmatch(r'r?#?"(.*)"#?', source, re.S)
            text = words.get(source) if source in words else (literal.group(1) if literal else None)
            x, y = number(x_expr, env), number(y_expr, env)
            if text is None or x is None or y is None:
                unresolved += 1
                print(f"  UNRESOLVED {fn}: {source[:40]!r} at ({' '.join(x_expr.split())[:30]}, {' '.join(y_expr.split())[:30]})")
                continue
            w = estimate(text) * size * 0.01 if kind == "tex" else width(text, size)
            h = tex_height(text, size) if kind == "tex" else 1.2 * size * 0.01
            left, right = x - w / 2, x + w / 2
            off = left < -LIMIT or right > LIMIT or y + h / 2 > 4 - 0.25 or y - h / 2 < -4 + 0.15
            fails += off
            print(f"  {'OFF ' if off else 'ok  '}{fn}: {kind} {w:5.2f} wide, {left:6.2f} to {right:5.2f}, y {y:5.2f}  {text[:58]}")
            boxes.setdefault(fn, []).append((left, right, y - h / 2, y + h / 2, text[:34]))
    print("  extra: strings built with format!, at their printed values")
    for sc, text, size, x, y in DYNAMIC:
        if sc == scene:
            w = width(text, size)
            off = x - w / 2 < -LIMIT or x + w / 2 > LIMIT
            fails += off
            print(f"  {'OFF ' if off else 'ok  '}{w:5.2f} wide, {x - w / 2:6.2f} to {x + w / 2:5.2f}  {text}")
    print(f"  unresolved calls, left to the layout lint: {unresolved}")
    print("  overlaps among resolved text in one function (a superset of what shares the screen):")
    count = 0
    for fn, rows in boxes.items():
        for i, a in enumerate(rows):
            for b in rows[i + 1 :]:
                if a[0] < b[1] and b[0] < a[1] and a[2] < b[3] and b[2] < a[3]:
                    count += 1
                    print(f"    {fn}: {a[4]!r} / {b[4]!r}")
    print(f"  {count} pairs")
print("RESULT", "PASS" if not fails else f"{fails} FLAGGED")
raise SystemExit(1 if fails else 0)
