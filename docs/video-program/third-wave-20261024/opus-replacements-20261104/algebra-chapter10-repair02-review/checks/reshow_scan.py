"""Text scan for a handle hidden and later shown again in the corpus's scene source.

Run from the repo root: python3 -I -B <this file>
Reads only. It's a regex over lines, not a parser: it sees `hide(stage, &[a, b]`
and `show(stage, &[a, b]` or `&list` written on one line, and treats a new `let`
or assignment of the name as a different object. A hit needs reading. No hits
doesn't prove there are no cases, for example a call split across lines.
"""
import re
from pathlib import Path

CALL = r"\b{}\(\s*stage,\s*&\[?([A-Za-z_][\w\.]*(?:,\s*[A-Za-z_][\w\.]*)*)\]?"
files, hits = 0, 0
for path in sorted(Path("explainer_corpus").glob("*/src/**/*.rs")):
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except UnicodeDecodeError:
        continue
    files += 1
    hidden = {}
    for n, line in enumerate(lines, 1):
        for name in list(hidden):
            if re.search(rf"\b(let (mut )?{re.escape(name)}\b|{re.escape(name)} = )", line):
                hidden.pop(name)
        m = re.search(CALL.format("hide"), line)
        if m:
            for name in re.split(r"\s*,\s*", m.group(1)):
                hidden[name] = n
            continue
        m = re.search(CALL.format("show"), line)
        if m:
            for name in re.split(r"\s*,\s*", m.group(1)):
                if name in hidden and n - hidden[name] < 300:
                    print(f"{path}: {name}: hidden at {hidden[name]}, shown at {n}")
                    hits += 1
                    hidden.pop(name)
print(f"{files} files, {hits} hits")
