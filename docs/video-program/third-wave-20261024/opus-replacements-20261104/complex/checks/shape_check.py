"""Desk check, not a compile. Three shapes a parser accepts and rustc rejects:
  1. format-family macros whose {} count differs from their positional arguments;
  2. `const X: [T; N] = [...]` whose element count is not N;
  3. a literal or obviously-shaped argument against a known parameter type
     (integer literal for f64, float literal for an integer, format!/vec!/array
     without & for a reference, & on a by-value parameter).
Text only. Usage (from the crate): python3 -I -B shape_check.py KIT_SRC FMN_CRATES
"""
import glob
import os
import re
import sys

SCRATCH = os.path.dirname(os.path.abspath(sys.argv[0]))
source = open(os.path.join(SCRATCH, "arity_check.py"), encoding="utf8").read()
exec(source[: source.index("def main():")])

kit_src, fmn_crates = sys.argv[1], sys.argv[2]
crate_files = [p for p in sorted(glob.glob("src/**/*.rs", recursive=True)) if not p.endswith(("thumbnail.rs", "score.rs"))]
problems = 0


def report(text):
    global problems
    print(text)
    problems += 1


# ---------------------------------------------------------------- 1. format strings
MACROS = re.compile(r"\b(format|println|eprintln|print|panic|assert|assert_eq|assert_ne|write|writeln)!\(")
fmt_checked = 0
for path in crate_files:
    raw = open(path, encoding="utf8").read()
    t = blank(raw)
    for m in MACROS.finditer(t):
        open_at = m.end() - 1
        end = close_of(t, open_at)
        if end < 0:
            continue
        pieces = []
        depth, start, i = 0, open_at + 1, open_at + 1
        while i < end:
            c = t[i]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
            elif c == "|" and depth == 0 and t[start:i].strip() in ("", "move"):
                j = t.find("|", i + 1)
                i = j if j > 0 else i
            elif c == "," and depth == 0:
                pieces.append((start, i))
                start = i + 1
            i += 1
        if t[start:end].strip():
            pieces.append((start, end))
        macro = m.group(1)
        skip = {"assert": 1, "assert_eq": 2, "assert_ne": 2, "write": 1, "writeln": 1}.get(macro, 0)
        if len(pieces) <= skip:
            continue
        a, b = pieces[skip]
        literal = raw[a:b].strip()
        got = re.match(r'r?(#*)"(.*)"\1$', literal, re.S)
        if not got:
            continue
        fmt = got.group(2).replace("{{", "").replace("}}", "")
        holes = re.findall(r"\{([^{}]*)\}", fmt)
        positional = sum(1 for h in holes if h == "" or h.startswith(":"))
        indexed = [int(h.split(":")[0]) for h in holes if h.split(":")[0].isdigit()]
        args = pieces[skip + 1 :]
        named = [t[x:y] for x, y in args if re.match(r"\s*\w+\s*=[^=]", t[x:y])]
        plain = len(args) - len(named)
        fmt_checked += 1
        want = max(positional, max(indexed) + 1 if indexed else 0)
        if want != plain:
            report(f"FORMAT {path}:{t[:m.start()].count(chr(10)) + 1} {macro}! has {want} positional holes, {plain} arguments")
print(f"format-family macros checked: {fmt_checked}")

# ---------------------------------------------------------------- 2. array lengths
arrays = 0
DECL = re.compile(r"\b(?:const|static)\s+(\w+)\s*:\s*\[([^\]]*?);\s*(\d+)\]\s*=\s*\[")
for path in crate_files:
    t = blank(open(path, encoding="utf8").read())
    for m in DECL.finditer(t):
        open_at = m.end() - 1
        end = close_of(t, open_at)
        inner = t[open_at + 1 : end]
        if re.match(r"\s*[^,\[\(]+;\s*\w+\s*$", inner):
            continue  # [x; N]
        count = len(top_commas(inner))
        arrays += 1
        if count != int(m.group(3)):
            report(f"ARRAY LENGTH {path}:{t[:m.start()].count(chr(10)) + 1} {m.group(1)} declares {m.group(3)}, has {count}")
print(f"sized const arrays checked: {arrays}")


# ---------------------------------------------------------------- 3. argument shapes
def signatures(paths):
    table = {}
    for path in paths:
        t = blank(open(path, encoding="utf8").read())
        for m in FN.finditer(t):
            open_at = m.end() - 1
            end = close_of(t, open_at)
            if end < 0:
                continue
            params = param_split(t[open_at + 1 : end])
            if params and re.match(r"\s*(mut\s+)?&?\s*('\w+\s+)?(mut\s+)?self\b", params[0]):
                params = params[1:]
            types = []
            for p in params:
                head, _, ty = p.partition(":")
                types.append(re.sub(r"\s+", " ", ty).strip())
            table.setdefault(m.group(1), []).append(types)
    return table


crate = signatures(crate_files)
kit = signatures(sorted(glob.glob(os.path.join(kit_src, "*.rs"))))
engine = signatures(
    [os.path.join(fmn_crates, "fmn/src/lib.rs")]
    + sorted(glob.glob(os.path.join(fmn_crates, "fmn-mobject/src/**/*.rs"), recursive=True))
    + sorted(glob.glob(os.path.join(fmn_crates, "fmn-library/src/**/*.rs"), recursive=True))
)
BY_VALUE = {"f64", "usize", "u32", "i32", "i64", "bool", "Mob", "Srgb", "Vec3", "C", "Plane", "Frame2", "Func"}
INTS = {"usize", "u32", "i32", "i64", "u64", "u8"}
call = re.compile(r"(?P<recv>\bkit\.|\bstage\.arena_mut\(\)\.|\bstage\.|(?<![\w.:]))(?P<name>[a-z_][a-z_0-9]*)\s*\(")
shapes = 0
for path in crate_files:
    t = blank(open(path, encoding="utf8").read())
    closures = set(re.findall(r"let\s+(?:mut\s+)?(\w+)\s*=\s*(?:move\s+)?\|", t))
    for m in call.finditer(t):
        recv, name = m.group("recv"), m.group("name")
        if t[max(0, m.start() - 3) : m.start()] == "fn " or (not recv and name in closures):
            continue
        table = kit if recv == "kit." else engine if recv.startswith("stage.") else crate
        options = table.get(name)
        if not options:
            continue
        open_at = m.end() - 1
        end = close_of(t, open_at)
        if end < 0:
            continue
        args = [a.strip() for a in top_commas(t[open_at + 1 : end])]
        fits = [o for o in options if len(o) == len(args)]
        if len(fits) != 1:
            continue  # ambiguous by name: nothing to compare against
        for arg, ty in zip(args, fits[0]):
            shapes += 1
            line = t[: m.start()].count("\n") + 1
            where = f"{path}:{line} {recv}{name}"
            if ty == "f64" and re.fullmatch(r"-?\d+", arg):
                report(f"INT FOR f64 {where}: {arg}")
            if ty in INTS and re.fullmatch(r"-?\d+\.\d*", arg):
                report(f"FLOAT FOR {ty} {where}: {arg}")
            if ty.startswith("&") and re.match(r"(format!|vec!|\[)", arg):
                report(f"MISSING & {where}: {arg[:40]} for {ty}")
            if ty in BY_VALUE and arg.startswith("&"):
                report(f"EXTRA & {where}: {arg[:40]} for {ty}")
            if ty == "Vec3" and arg.startswith("("):
                report(f"TUPLE FOR Vec3 {where}: {arg[:40]}")
            if ty == "bool" and re.fullmatch(r"-?[\d.]+", arg):
                report(f"NUMBER FOR bool {where}: {arg}")
            if ty.startswith("Option<") and re.fullmatch(r"-?[\d.]+", arg):
                report(f"BARE VALUE FOR {ty} {where}: {arg}")
print(f"argument shapes compared: {shapes}")
print(f"shape problems: {problems}")
