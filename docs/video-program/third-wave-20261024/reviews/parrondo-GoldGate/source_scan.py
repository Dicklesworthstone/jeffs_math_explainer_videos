#!/usr/bin/env python3
"""GoldGate's static scan of the parrondo snapshot's Rust (no Cargo).

    python3 -I -B source_scan.py SNAPSHOT_DIR

What the non-test build would compile (every `#[cfg(test)]` item and the
trailing `mod tests` removed): imports never named outside their `use`, and
crate items (fn, const, struct, enum) never named outside their definition.
Each is a candidate `unused_imports` / `dead_code` warning, which
`cargo clippy --all-targets -- -D warnings` (AGENTS.md) turns into an error.
Candidates are hints for a reader to confirm by eye, not compiler output:
trait imports used only through method calls (`Timed::rt`) and items named
only by macros show up here too.
"""

import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1]) / "src"
files = sorted(root.rglob("*.rs"))


def non_test(text):
    cut = re.search(r"\n#\[cfg\(test\)\]\s*\nmod tests\b", text)
    if cut:
        text = text[: cut.start()]
    # a single #[cfg(test)] item: const/use/static up to ';', or fn/struct up to its closing brace
    out, i = [], 0
    for m in re.finditer(r"#\[cfg\(test\)\]\s*\n", text):
        if m.start() < i:
            continue
        out.append(text[i : m.start()])
        j = m.end()
        semi = text.find(";", j)
        brace = text.find("{", j)
        if brace == -1 or (semi != -1 and semi < brace):
            i = semi + 1
        else:
            depth, k = 0, brace
            while True:
                if text[k] == "{":
                    depth += 1
                elif text[k] == "}":
                    depth -= 1
                    if depth == 0:
                        break
                k += 1
            i = k + 1
    out.append(text[i:])
    return "".join(out)


def strip_comments(text):
    text = re.sub(r"//[^\n]*", "", text)
    # string literals, including `\` line continuations (hence re.S)
    return re.sub(r'"(?:\\.|[^"\\])*"', '""', text, flags=re.S)


bodies = {f: strip_comments(non_test(f.read_text(encoding="utf-8"))) for f in files}
everything = "\n".join(bodies.values())

print("== imports never named outside their `use` (non-test build)")
for f, body in bodies.items():
    uses = re.findall(r"\buse\s+([^;]+);", body)
    rest = re.sub(r"\buse\s+[^;]+;", "", body)
    for u in uses:
        names = []
        if "{" in u:
            inner = u[u.index("{") + 1 : u.rindex("}")]
            head = u[: u.index("{")].rstrip(":").split("::")[-1]
            for part in inner.split(","):
                part = part.strip()
                if not part or part == "*":
                    continue
                if part == "self":
                    names.append(head)
                else:
                    names.append(part.split(" as ")[-1].split("::")[-1].strip())
        elif not u.strip().endswith("*"):
            names.append(u.split(" as ")[-1].split("::")[-1].strip())
        for n in names:
            if not re.search(rf"\b{re.escape(n)}\b", rest):
                print(f"   {f.relative_to(root.parent)}: `{n}` (from `use {' '.join(u.split())}`)")

print("== crate items never named outside their definition (non-test build)")
for f, body in bodies.items():
    for m in re.finditer(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:fn|const|struct|enum|static)\s+([A-Za-z_][A-Za-z0-9_]*)", body, re.M):
        name = m.group(1)
        if name in ("main", "construct", "name"):
            continue
        hits = len(re.findall(rf"\b{re.escape(name)}\b", everything))
        if hits <= 1:
            print(f"   {f.relative_to(root.parent)}: `{name}`")
