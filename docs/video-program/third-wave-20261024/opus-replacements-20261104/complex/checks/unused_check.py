"""Desk check, not a compile: `let` bindings never used again before their
function ends or the name is rebound, and `let mut` bindings never visibly
mutated. Comments do not count as uses; format strings do ({name} captures).
Usage (from the crate): python3 -I -B unused_check.py
"""
import glob
import os
import re
import sys

SCRATCH = os.path.dirname(os.path.abspath(sys.argv[0]))
source = open(os.path.join(SCRATCH, "arity_check.py"), encoding="utf8").read()
exec(source[: source.index("def main():")])


def strip_comments(src):
    """Blank comments only, keeping strings and every offset."""
    out = list(src)
    i, n = 0, len(src)
    in_str = False
    while i < n:
        c = src[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif src[i : i + 2] == "//":
            j = src.find("\n", i)
            j = n if j < 0 else j
            out[i:j] = " " * (j - i)
            i = j
            continue
        elif c == '"':
            in_str = True
        elif c == "'" and re.match(r"'(\\.|[^\\'])'", src[i : i + 4]):
            i += re.match(r"'(\\.|[^\\'])'", src[i : i + 4]).end()
            continue
        i += 1
    return "".join(out)


FN_HEAD = re.compile(r"\bfn\s+\w+")
LET = re.compile(r"\blet\s+(mut\s+)?(?:(\w+)|([\(\[][^=;]*?[\)\]]))\s*(?::[^=;]+)?=")
problems = 0
checked = 0
for path in sorted(glob.glob("src/**/*.rs", recursive=True)):
    if path.endswith(("thumbnail.rs", "score.rs")):
        continue
    raw = open(path, encoding="utf8").read()
    t = blank(raw)
    text = strip_comments(raw)
    for head in FN_HEAD.finditer(t):
        paren = t.find("(", head.end() - 1)
        close = close_of(t, paren)
        if close < 0:
            continue
        semi, brace = t.find(";", close), t.find("{", close)
        if brace < 0 or (0 <= semi < brace):
            continue
        end = close_of(t, brace)
        body_start = brace
        for m in LET.finditer(t, body_start, end):
            mut, simple, pattern = m.group(1), m.group(2), m.group(3)
            names = [simple] if simple else re.findall(r"\b([a-z_]\w*)\b", pattern or "")
            for name in names:
                if name.startswith("_") or name in ("mut", "ref", "true", "false"):
                    continue
                checked += 1
                after = m.end()
                rebound = re.compile(r"\blet\s+(?:mut\s+)?[\(\[]?[^=;]*?\b" + re.escape(name) + r"\b[^=;]*?=")
                stop = end
                nxt = rebound.search(t, after, end)
                # the initializer of a rebinding may itself use the old value, so search through its `=` and on to `;`
                if nxt:
                    semi2 = t.find(";", nxt.end())
                    stop = semi2 if 0 <= semi2 < end else end
                use = re.compile(r"\b" + re.escape(name) + r"\b")
                span = text[after:stop]
                line = t[: m.start()].count("\n") + 1
                if not use.search(span):
                    print(f"UNUSED {path}:{line} {name}")
                    problems += 1
                    continue
                if mut and simple:
                    tspan = t[after:stop]
                    mutated = re.search(
                        r"(\b" + re.escape(name) + r"\s*(\[[^\]]*\]\s*)?(=[^=]|[-+*/]=))"
                        r"|(&mut\s+\*?" + re.escape(name) + r"\b)"
                        r"|(\b" + re.escape(name) + r"\.\w+\s*\()"
                        r"|(\bfor\b[^{]*\bin\s+&mut\s+" + re.escape(name) + r"\b)",
                        tspan,
                    )
                    if not mutated:
                        print(f"NEEDLESS MUT {path}:{line} {name}")
                        problems += 1
print(f"bindings checked: {checked}; problems: {problems}")
