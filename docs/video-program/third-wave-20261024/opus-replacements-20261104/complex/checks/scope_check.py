"""Desk check, not a compile: names used in a file without being in scope there.

For each file, and for its test module separately, a name counts as in scope if
the file declares it, imports it, or the engine could supply it through
`fmn::prelude::*`. A name reached by a path (`draw::x`, `Palette::x`) is judged
only by its first segment. Text only: it cannot see block scopes or macros.

Usage (from the crate): python3 -I -B scope_check.py KIT_SRC FMN_CRATES
"""
import glob
import os
import re
import sys

SCRATCH = os.path.dirname(os.path.abspath(sys.argv[0]))
source = open(os.path.join(SCRATCH, "arity_check.py"), encoding="utf8").read()
exec(source[: source.index("def main():")])  # blank(), close_of(), FN

kit_src, fmn_crates = sys.argv[1], sys.argv[2]

ITEM = re.compile(r"\b(?:fn|const|static|struct|enum|type|trait|mod)\s+([A-Za-z_]\w*)")
USE = re.compile(r"^\s*(?:pub(?:\(\w+\))?\s+)?use\s+([^;]+);", re.M)


def imported(text):
    names, globs = set(), []
    for m in USE.finditer(text):
        body = re.sub(r"\s+", " ", m.group(1))
        if body.endswith("*"):
            globs.append(body)
        for piece in re.findall(r"([A-Za-z_]\w*)\s*(?=[,}]|$)", body):
            names.add(piece)
        for piece in re.findall(r"\bas\s+([A-Za-z_]\w*)", body):
            names.add(piece)
        if "self" in body:
            head = re.match(r"([\w:]+)::\{", body)
            if head:
                names.add(head.group(1).split("::")[-1])
    return names, globs


# what the engine and kit could supply, by name
engine_names = set()
for path in glob.glob(os.path.join(fmn_crates, "**/*.rs"), recursive=True):
    if "/fmn-python/" in path or "/fmn-wasm/" in path:
        continue
    text = open(path, encoding="utf8", errors="replace").read()
    engine_names.update(re.findall(r"\bpub\s+(?:const\s+fn|fn|const|static|struct|enum|type|trait|mod)\s+([A-Za-z_]\w*)", text))
    engine_names.update(re.findall(r"\bpub\s+use\s+[^;]*?([A-Z]\w*)\s*;", text))
    engine_names.update(re.findall(r"\bconst\s+([A-Z][A-Z0-9_]+)", text))

kit_names = set()
for path in glob.glob(os.path.join(kit_src, "*.rs")):
    text = open(path, encoding="utf8").read()
    kit_names.update(re.findall(r"\bpub\s+(?:const\s+fn|fn|const|static|struct|enum|type|trait)\s+([A-Za-z_]\w*)", text))

crate_files = sorted(glob.glob("src/**/*.rs", recursive=True))
owners = {}
for path in crate_files:
    t = blank(open(path, encoding="utf8").read())
    for name in ITEM.findall(t):
        owners.setdefault(name, set()).add(path)

STD_TYPES = set(
    """Some None Ok Err Self Vec String Box Option Result Rc RefCell Cell OnceLock Fn FnMut FnOnce Copy Clone Debug
    Default Display Send Sync Sized Iterator IntoIterator PartialEq PartialOrd Eq Ord Hash From Into AsRef
    Add Sub Mul Div Neg ToString""".split()
)
problems = 0
for path in crate_files:
    if path.endswith(("thumbnail.rs", "score.rs")):
        continue
    raw = blank(open(path, encoding="utf8").read())
    split = raw.split("#[cfg(test)]\nmod tests")
    prod = split[0]
    test = split[1] if len(split) > 1 else ""
    prod_names, prod_globs = imported(prod)
    prod_items = set(ITEM.findall(prod))
    test_names, _ = imported(test)
    test_items = set(ITEM.findall(test))
    has_prelude = any("prelude" in g for g in prod_globs)
    for label, body, scope in (
        ("prod", prod, prod_names | prod_items),
        ("test", test, prod_names | prod_items | test_names | test_items),
    ):
        if not body:
            continue
        # the use lines themselves are not uses
        body_nouse = USE.sub(lambda m: "\n" * m.group(0).count("\n"), body)
        seen = set()
        # type-like and constant-like names, not reached through a path or a field
        for m in re.finditer(r"(?<![\w.:])([A-Z]\w*)", body_nouse):
            name = m.group(1)
            if name in scope or name in STD_TYPES or name in seen:
                continue
            # generic parameters and associated names declared in place
            if re.search(r"<[^<>]*\b" + name + r"\b[^<>]*>\s*\(", body_nouse) and len(name) == 1:
                continue
            seen.add(name)
            if has_prelude and name in engine_names and name not in kit_names:
                continue
            line = body_nouse[: m.start()].count("\n") + 1
            where = "kit" if name in kit_names else ("crate:" + ",".join(sorted(os.path.basename(p) for p in owners[name])) if name in owners else "unknown")
            print(f"NOT IN SCOPE ({label}) {path}:{line} {name} [{where}]")
            problems += 1
        # free calls to a function another file of this crate or the kit defines
        for m in re.finditer(r"(?<![\w.:])([a-z_]\w*)\s*(?:::<[^(]*>)?\(", body_nouse):
            name = m.group(1)
            if name in scope or name in seen:
                continue
            if name in ("if", "for", "while", "match", "return", "move", "in", "as", "let", "mut", "fn", "else", "loop", "pub", "assert", "vec", "format", "panic", "write", "println"):
                continue
            if re.search(r"\blet\s+(?:mut\s+)?" + re.escape(name) + r"\b", body_nouse):
                continue
            if re.search(r"\b" + re.escape(name) + r"\s*:\s*(?:&\s*)?(?:dyn|impl|fn)\b", body_nouse):
                continue
            if re.search(r"\|[^|\n]*\b" + re.escape(name) + r"\b[^|\n]*\|", body_nouse):
                continue
            elsewhere = name in owners or name in kit_names
            in_engine = name in engine_names
            if has_prelude and in_engine and name not in owners and name not in kit_names:
                continue
            if elsewhere or not in_engine:
                seen.add(name)
                line = body_nouse[: m.start()].count("\n") + 1
                where = "kit" if name in kit_names else ("crate:" + ",".join(sorted(os.path.basename(p) for p in owners.get(name, []))) if name in owners else "unknown")
                print(f"NOT IN SCOPE ({label}) {path}:{line} {name}() [{where}]")
                problems += 1
print(f"scope problems: {problems}")
