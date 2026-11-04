"""Static cross-check of the native scenes against script.tsv and facts.rs.

For each native chapter file: the say() ids must be exactly the chapter's
cues in script order; every wait_sentence index must exist (sentences split
as draw.rs sentence_offsets does: '.', '?' or '!' followed by a space);
every phrase passed to wait_for/until/at must occur in the cue; every fact
key must be added or pinned in facts.rs; every predict() must be on a cue in
chapters::PREDICTS whose text ends with '?'. Read-only; prints a report.
"""

import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
script = {}
order = []
for raw in (root / "script.tsv").read_text(encoding="utf-8").splitlines():
    if not raw.strip() or raw.startswith("#"):
        continue
    cue, text = raw.split("\t", 1)
    script[cue] = text
    order.append(cue)

facts_src = (root / "src/facts.rs").read_text(encoding="utf-8")
keys = set(re.findall(r'f\.(?:add|pin)\(\s*"([^"]+)"', facts_src))

mod_src = (root / "src/chapters/mod.rs").read_text(encoding="utf-8")
predicts = dict(re.findall(r'\("(\d\d_\d+)", ([0-9.]+)\)', mod_src))


def sentences(text):
    n = 1
    for i in range(len(text) - 2):
        if text[i] in ".?!" and text[i + 1] == " ":
            n += 1
    return n


problems = 0
files = sorted((root / "src/chapters").glob("*.rs"))
for f in files:
    src = f.read_text(encoding="utf-8")
    ids = re.findall(r'say\(&kit, stage, "(\d\d_\d+)"\)', src)
    if not ids:
        continue
    chapter = ids[0][:2]
    want = [c for c in order if c.startswith(chapter + "_")]
    status = "ok" if ids == want else f"MISMATCH want {want}"
    if ids != want:
        problems += 1
    print(f"{f.name}: says {len(ids)} cues {ids[0]}..{ids[-1]} {status}")
    current = None
    for n, line in enumerate(src.splitlines(), 1):
        m = re.search(r'say\(&kit, stage, "(\d\d_\d+)"\)', line)
        if m:
            current = m.group(1)
        for k in re.findall(r'wait_sentence\(stage, (\d+)\)', line):
            have = sentences(script[current])
            if int(k) >= have:
                problems += 1
                print(f"  {f.name}:{n} {current} sentence {k} but only {have}")
        for p in re.findall(r'(?:wait_for\(stage, |until\(stage, |\.at\()"([^"]+)"', line):
            if p not in script[current]:
                problems += 1
                print(f"  {f.name}:{n} {current} has no phrase {p!r}")
        for key in re.findall(r'fact(?:_f64|_list)?\("([^"]+)"\)', line):
            if key not in keys:
                problems += 1
                print(f"  {f.name}:{n} fact {key!r} is not in facts.rs")
        if re.search(r'\bpredict\(&kit, stage, &line', line):
            if current not in predicts:
                problems += 1
                print(f"  {f.name}:{n} predict on {current}, not in PREDICTS")
            elif not script[current].rstrip().endswith("?"):
                problems += 1
                print(f"  {f.name}:{n} predict on {current}, which does not end with '?'")
print(f"fact keys in facts.rs: {len(keys)}; predicts in mod.rs: {len(predicts)}")
print("problems:", problems)
sys.exit(1 if problems else 0)
