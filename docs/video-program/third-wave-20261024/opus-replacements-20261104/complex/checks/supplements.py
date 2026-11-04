"""Resolve chapters::TEXT/TEX/MONO entries to their string values; report
duplicates within each list, unresolved names, and string constants of the
native chapters that no list names. Read-only. Usage: supplements.py <crate>"""
import re
import sys
from pathlib import Path

crate = Path(sys.argv[1])
chapters = crate / "src" / "chapters"


def literals(src, i):
    """Parse Rust string literals from src[i:] up to the closing ';'."""
    out = []
    depth = 0
    while i < len(src):
        c = src[i]
        if c == ";" and depth == 0:
            return out, i
        if c == "[":
            depth += 1
        elif c == "]":
            depth -= 1
        if c == "r" and src[i + 1] in '"#':
            j = i + 1
            hashes = 0
            while src[j] == "#":
                hashes += 1
                j += 1
            end = '"' + "#" * hashes
            k = src.index(end, j + 1)
            out.append(src[j + 1 : k])
            i = k + len(end)
            continue
        if c == '"':
            j = i + 1
            buf = []
            while src[j] != '"':
                if src[j] == "\\":
                    nxt = src[j + 1]
                    buf.append({"n": "\n", "t": "\t", "\\": "\\", '"': '"'}.get(nxt, nxt))
                    j += 2
                else:
                    buf.append(src[j])
                    j += 1
            out.append("".join(buf))
            i = j + 1
            continue
        i += 1
    raise SystemExit("unterminated const")


consts = {}
widths = {}
for f in chapters.glob("*.rs"):
    src = f.read_text()
    for m in re.finditer(r"pub const (\w+): (&str|\[&str; \d+\]|\[\[&str; (\d+)\]; \d+\]) =", src):
        vals, _ = literals(src, m.end())
        consts[(f.stem, m.group(1))] = vals
        if m.group(3):
            widths[(f.stem, m.group(1))] = int(m.group(3))

mod = (chapters / "mod.rs").read_text()
problems = 0
named = set()
for name in ["TEXT", "TEX", "MONO"]:
    m = re.search(rf"pub const {name}: &\[&str\] = &\[(.*?)\n\];", mod, re.S)
    body = m.group(1)
    seen = {}
    for line in body.splitlines():
        line = line.strip().rstrip(",")
        if not line or line.startswith("//"):
            continue
        r = re.fullmatch(r"(\w+)::(\w+)(?:\[(\d+)\])?(?:\[(\d+)\])?", line)
        if r:
            key = (r.group(1), r.group(2))
            named.add(key)
            if key not in consts:
                print(f"{name}: unresolved {line}")
                problems += 1
                continue
            vals = consts[key]
            idx = int(r.group(3)) if r.group(3) else 0
            if key in widths:
                if r.group(4) is None:
                    print(f"{name}: {line} is a 2-D array; index it twice")
                    problems += 1
                    continue
                idx = idx * widths[key] + int(r.group(4))
            if r.group(3) is None and len(vals) != 1:
                print(f"{name}: {line} is an array; index it")
                problems += 1
            if idx >= len(vals):
                print(f"{name}: {line} index out of range")
                problems += 1
                continue
            val = vals[idx]
        else:
            vals, _ = literals(line + ";", 0)
            if len(vals) != 1:
                print(f"{name}: cannot read {line!r}")
                problems += 1
                continue
            val = vals[0]
        if not val.strip():
            print(f"{name}: empty {line}")
            problems += 1
        if val in seen:
            print(f"{name}: duplicate {val!r}: {seen[val]} and {line}")
            problems += 1
        seen[val] = line
    print(f"{name}: {len(seen)} entries")

native = sorted(f.stem for f in chapters.glob("*.rs") if f.stem not in ("mod", "draw", "thumbnail"))
for (stem, cname), vals in sorted(consts.items()):
    if stem in native and (stem, cname) not in named:
        print(f"not listed: {stem}::{cname} = {vals!r}")
print(f"{problems} problems")
