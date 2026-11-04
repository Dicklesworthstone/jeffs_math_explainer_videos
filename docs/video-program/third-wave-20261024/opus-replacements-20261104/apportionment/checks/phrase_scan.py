"""Check every chapter's draft cues against script.tsv: each
`say(&kit, stage, "ID")` opens a line; each later `wait_for(stage, "PHRASE")`
must occur in that line's text, after the previous phrase of the same line
(wait_for never waits backwards, so an out-of-order phrase is a silent
mistiming). Also: every script id is said exactly once, in order, and the
phrase loops over literal lists are reported for a manual look.
argv: candidate dir."""

import re
import sys
from pathlib import Path

cand = Path(sys.argv[1])
script = {}
order = []
for line in (cand / "script.tsv").read_text().splitlines():
    if line.startswith("#") or not line.strip():
        continue
    cid, text = line.split("\t", 1)
    script[cid] = text
    order.append(cid)

token = re.compile(r'say\(\s*&kit,\s*stage,\s*"(\d\d_\d+)"\s*\)|wait_for\(\s*stage,\s*"((?:[^"\\]|\\.)*)"\s*\)', re.S)
problems, per_file, checked = [], [], 0
for path in sorted((cand / "src" / "chapters").glob("*.rs")):
    src = path.read_text()
    said = []
    per_file.append(said)
    cur, pos = None, -1
    for m in token.finditer(src):
        if m.group(1):
            cur, pos = m.group(1), -1
            said.append(cur)
            if cur not in script:
                problems.append(f"{path.name}: says {cur}, not in script.tsv")
            continue
        phrase = m.group(2)
        checked += 1
        if cur is None:
            problems.append(f"{path.name}: wait_for {phrase!r} before any say")
            continue
        text = script.get(cur, "")
        at = text.find(phrase)
        if at < 0:
            problems.append(f"{path.name} {cur}: phrase {phrase!r} not in the line")
        elif at < pos:
            problems.append(f"{path.name} {cur}: phrase {phrase!r} comes before the previous cue")
        else:
            pos = at
said = [c for f in sorted((f for f in per_file if f), key=lambda f: f[0]) for c in f]
if said != order:
    missing = [c for c in order if c not in said]
    extra = [c for c in said if c not in order]
    problems.append(f"said order differs: missing {missing}, extra {extra}, first mismatch "
                    f"{next(((a, b) for a, b in zip(said, order) if a != b), None)}")
print(f"{len(said)} lines said, {checked} literal phrases checked")
for p in problems:
    print("PROBLEM", p)
print("problems:", len(problems))
