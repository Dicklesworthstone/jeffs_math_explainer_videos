"""item-reach01.py: do the bundle03 source changes reach any of master993's kept chapters? (CC0, independent)

Text reads only. For the four changed chapter modules (the_bump, loop_sums, boundary, counting) it splits the 760 base
and the bundle03 replacement into top-level items and lists the items whose text differs. It then takes every
reference from an unchanged source file into those modules, follows it through the module's own items (and the
use-imports between changed modules) to a closure, and reports whether the closure holds any changed item.
mod.rs is the fifth changed row; its diff is printed whole.

usage: python3 -I -B item-reach01.py BASE_SRC BUNDLE03_DIR OUT
  BASE_SRC     the 760 tree's complex_analysis_visual/src
  BUNDLE03_DIR the folder holding csd-counting02-<module>.rs copies
"""
import difflib
import hashlib
import re
import sys
from pathlib import Path

BASE, NEW, OUT = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
CHANGED = ("the_bump", "loop_sums", "boundary", "counting")
lines = []
say = lines.append
START = re.compile(r"^(pub(\([^)]*\))?\s+)?(const|static|fn|struct|enum|type|trait|mod|impl|use|macro_rules!)\b|^impl\b")
NAME = re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?(const|static|fn|struct|enum|type|trait|mod)\s+(\w+)")
WORD = re.compile(r"\b[A-Za-z_]\w*\b")


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def items(text):
    """name -> item text (with the attributes and comments just above it)"""
    out, cur, name, pending = {}, [], None, []

    def closed():
        last = next((r for r in reversed(cur) if r.strip()), "")
        return last.startswith("}") or last.startswith("]") or (last.endswith(";") and not last[:1].isspace())

    for row in text.splitlines():
        if START.match(row):
            if name is not None:
                out[name] = "\n".join(cur).rstrip()
            cur, pending = pending + [row], []
            m = NAME.match(row)
            if m:
                name = "%s %s" % (m.group(1), m.group(2))
            elif row.startswith("use") or row.startswith("pub use"):
                name = "use " + row.strip()
            else:
                name = row.split("{")[0].strip()
            n, k = name, 2
            while n in out:
                n, k = "%s #%d" % (name, k), k + 1
            name = n
        elif (row.startswith("#[") or row.startswith("//")) and (name is None or closed()):
            pending.append(row)
        elif pending and not row.strip():
            pending.append(row)
        elif name is not None:
            cur += pending + [row]
            pending = []
    if name is not None:
        out[name] = "\n".join(cur).rstrip()
    return out


def short(name):
    return name.split()[-1] if name.split()[0] in ("const", "static", "fn", "struct", "enum", "type", "trait", "mod") else name


mods = {}
for m in CHANGED:
    b, n = (BASE / "chapters" / ("%s.rs" % m)).read_text(), (NEW / ("csd-counting02-%s.rs" % m)).read_text()
    ib, inew = items(b), items(n)
    changed = sorted(k for k in set(ib) | set(inew) if ib.get(k) != inew.get(k))
    mods[m] = {"base": ib, "new": inew, "changed": set(changed)}
    say("== %s: base %s, bundle03 %s; %d items, %d differ" % (m, sha(BASE / "chapters" / ("%s.rs" % m))[:8],
                                                            sha(NEW / ("csd-counting02-%s.rs" % m))[:8], len(inew), len(changed)))
    for k in changed:
        tag = "added" if k not in ib else "removed" if k not in inew else "changed"
        say("   %-8s %s" % (tag, k[:100]))

by_short = {m: {} for m in CHANGED}
for m in CHANGED:
    for k in set(mods[m]["base"]) | set(mods[m]["new"]):
        if not k.startswith("use "):
            by_short[m].setdefault(short(k), []).append(k)
    for k in set(mods[m]["new"]):
        if k.startswith("impl"):
            for w in WORD.findall(k):
                if w not in ("impl", "for", "SceneConstruct", "mut"):
                    by_short[m].setdefault(w, []).append(k)


def imports(m):
    """name -> (module, name) for use-imports from other changed modules"""
    out = {}
    for k in mods[m]["new"]:
        if not k.startswith("use "):
            continue
        g = re.match(r"use super::(\w+)::(\{([^}]*)\}|(\w+));", k[4:])
        if g and g.group(1) in CHANGED:
            for nm in (g.group(3) or g.group(4)).split(","):
                if nm.strip():
                    out[nm.strip()] = (g.group(1), nm.strip())
    return out


IMP = {m: imports(m) for m in CHANGED}


def closure(m, name):
    seen, todo = set(), [(m, k) for k in by_short[m].get(name, [])]
    while todo:
        mk = todo.pop()
        if mk in seen:
            continue
        seen.add(mk)
        mm, k = mk
        body = mods[mm]["new"].get(k) or mods[mm]["base"].get(k, "")
        for w in set(WORD.findall(body)):
            for k2 in by_short[mm].get(w, []):
                todo.append((mm, k2))
            if w in IMP[mm]:
                m2, n2 = IMP[mm][w]
                todo += [(m2, k2) for k2 in by_short[m2].get(n2, [])]
    return seen


say("== references from unchanged files into the changed modules, and their closures")
refs = []
for p in sorted(BASE.rglob("*.rs")):
    rel = p.relative_to(BASE).as_posix()
    if rel in ("chapters/%s.rs" % m for m in CHANGED) or rel == "chapters/mod.rs":
        continue
    text = p.read_text()
    for g in re.finditer(r"use super::(\w+)::(\{([^}]*)\}|(\w+));", text):
        if g.group(1) in CHANGED:
            for nm in (g.group(3) or g.group(4)).split(","):
                if nm.strip():
                    refs.append((rel, g.group(1), nm.strip()))
    for g in re.finditer(r"\b(the_bump|loop_sums|boundary|counting)::(\w+)", text):
        if not text[max(0, g.start() - 11):g.start()].endswith("use super::"):
            refs.append((rel, g.group(1), g.group(2)))
reached_changed = []
for rel, m, nm in sorted(set(refs)):
    cl = closure(m, nm)
    hit = sorted("%s::%s" % (mm, short(k)) for mm, k in cl if k in mods[mm]["changed"])
    say("%-28s -> %s::%-14s closure %3d items; changed in closure: %s" % (rel, m, nm, len(cl), hit or "none"))
    if hit:
        reached_changed.append((rel, m, nm, hit))
    if not cl:
        say("   WARNING: %s::%s not found as a top-level item" % (m, nm))

say("== mod.rs (760 e1f4d1b8 -> bundle03 4de1e4a5), whole diff")
a, b = (BASE / "chapters/mod.rs").read_text().splitlines(), (NEW / "csd-counting02-mod.rs").read_text().splitlines()
say("mod.rs base %s, bundle03 %s" % (sha(BASE / "chapters/mod.rs")[:8], sha(NEW / "csd-counting02-mod.rs")[:8]))
for row in difflib.unified_diff(a, b, "base", "bundle03", n=0, lineterm=""):
    say("   " + row)
im = items((NEW / "csd-counting02-mod.rs").read_text())
ib = items((BASE / "chapters/mod.rs").read_text())
say("mod.rs items that differ: %s" % sorted(k for k in set(im) | set(ib) if im.get(k) != ib.get(k)))
say("RESULT: %s" % ("no reference from an unchanged file reaches a changed item" if not reached_changed
                    else "%d references reach changed items" % len(reached_changed)))
with open(OUT, "x", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")
print("\n".join(lines))
