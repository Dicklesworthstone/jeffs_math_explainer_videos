"""Desk stand-in for two things the build would check in the glyph registry of chapters/mod.rs.

1. `no_supplement_is_empty_or_duplicated`: values within TEXT, TEX and MONO stay unique.
2. The new entries use only characters and TeX commands the registry already has, or are listed.

Run from the repository root:  python3 -I -B <this file> [folder of a stacked crate's src/chapters]
Five chapters, facts and mod.rs resolve in the staged candidate; the others in the canonical crate.
With a folder, the candidate side is that folder instead: replay_patches.py uses this for all lanes stacked.
Nothing here compiles anything, so it is not evidence that the glyph probes pass.
"""
import os
import re
import sys

ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-picture-repair-11-15"
CANON = "explainer_corpus/complex_analysis_visual/src/chapters"
SIDES = {
    "baseline": ROOT + "/baseline/explainer_corpus/complex_analysis_visual/src/chapters",
    "candidate": ROOT + "/candidate/explainer_corpus/complex_analysis_visual/src/chapters",
}

if len(sys.argv) > 1:
    SIDES["candidate"] = sys.argv[1]


def literals_and_brackets(body):
    """Tokens of a Rust initialiser: strings, and [ ] ; stops at the top-level semicolon."""
    tokens, i = [], 0
    while i < len(body):
        c = body[i]
        if c == "/" and body[i : i + 2] == "//":
            i = body.index("\n", i)
            continue
        if c == ";":
            break
        if c in "[]":
            tokens.append(c)
            i += 1
            continue
        raw = re.match(r'r(#*)"', body[i:])
        if raw:
            fence = '"' + raw.group(1)
            start = i + raw.end()
            end = body.index(fence, start)
            tokens.append(("s", body[start:end]))
            i = end + len(fence)
            continue
        if c == '"':
            j, out = i + 1, []
            while body[j] != '"':
                if body[j] == "\\":
                    if body[j + 1] == "\n":
                        j += 2
                        while body[j].isspace():
                            j += 1
                        continue
                    out.append({"n": "\n", "t": "\t", '"': '"', "\\": "\\"}[body[j + 1]])
                    j += 2
                else:
                    out.append(body[j])
                    j += 1
            tokens.append(("s", "".join(out)))
            i = j + 1
            continue
        i += 1
    return tokens


def tree(tokens):
    stack = [[]]
    for t in tokens:
        if t == "[":
            stack.append([])
        elif t == "]":
            done = stack.pop()
            stack[-1].append(done)
        else:
            stack[-1].append(t[1])
    return stack[0][0] if len(stack[0]) == 1 else stack[0]


def const_value(source, name):
    m = re.search(r"pub const " + re.escape(name) + r"\s*:[^=]*=", source)
    if not m:
        return None
    return tree(literals_and_brackets(source[m.end() :]))


def module_source(side, module, cache={}):
    key = (side, module)
    if key not in cache:
        path = SIDES[side] + "/" + module + ".rs"
        if not os.path.exists(path):
            path = CANON + "/" + module + ".rs"
        cache[key] = open(path, encoding="utf-8").read()
    return cache[key]


def registry(side, list_name):
    """The (entry, value) pairs of TEXT, TEX or MONO on one side."""
    source = module_source(side, "mod")
    start = source.index("pub const " + list_name + ": &[&str] = &[")
    end = source.index("\n];", start)
    pairs = []
    for raw in source[start:end].splitlines()[1:]:
        line = raw.split("//")[0].strip().rstrip(",") if not raw.strip().startswith("//") else ""
        if not line:
            continue
        if line.startswith('"') or line.startswith('r"') or line.startswith('r#'):
            pairs.append((line, tree(literals_and_brackets(line + ";"))))
            continue
        m = re.fullmatch(r"(\w+)::(\w+)((?:\[\d+\])*)", line)
        if not m:
            pairs.append((line, None))
            continue
        value = const_value(module_source(side, m.group(1)), m.group(2))
        for index in re.findall(r"\d+", m.group(3)):
            value = value[int(index)] if value is not None else None
        pairs.append((line, value))
    return pairs


def tex_commands(s):
    return set(re.findall(r"\\[a-zA-Z]+", s))


failed = False
for name in ("TEXT", "TEX", "MONO"):
    base = registry("baseline", name)
    cand = registry("candidate", name)
    unresolved = [e for e, v in cand if not isinstance(v, str)]
    values = [v for _, v in cand if isinstance(v, str)]
    dupes = sorted({v for v in values if values.count(v) > 1})
    added = [(e, v) for e, v in cand if e not in {x for x, _ in base}]
    removed = [e for e, _ in base if e not in {x for x, _ in cand}]
    print(f"{name}: baseline {len(base)} entries, candidate {len(cand)}; "
          f"unresolved {len(unresolved)}; duplicate values {len(dupes)}; added {len(added)}; removed {len(removed)}")
    for e in unresolved:
        print("  UNRESOLVED", e)
    for v in dupes:
        print("  DUPLICATE", repr(v))
        failed = True
    known_chars = {c for _, v in base if isinstance(v, str) for c in v}
    known_cmds = {c for _, v in base if isinstance(v, str) for c in tex_commands(v)}
    for e, v in added:
        new_chars = sorted({c for c in v if c not in known_chars}) if name != "TEX" else []
        new_cmds = sorted(tex_commands(v) - known_cmds) if name == "TEX" else []
        note = ""
        if new_chars:
            note += f"  NEW CHARACTERS {new_chars}"
        if new_cmds:
            note += f"  COMMANDS NOT YET IN THIS REGISTRY {new_cmds}"
        print(f"  + {e} = {v!r}{note}")
        if not v.strip():
            print("  EMPTY", e)
            failed = True
    for e in removed:
        print("  - ", e)

print("RESULT", "FAIL" if failed else "PASS: no empty or duplicate values")
sys.exit(1 if failed else 0)
