"""Scratch: what do the direction maps bind in the three source files? Read-only."""
import json
import os
import sys

WS = sys.argv[1]
pins = ("2a4e84504bab", "93f663d372ab", "01885b9f558c")
names = ("draw.rs", "complex_turns.rs", "lifting_loops.rs")


def walk(x, path, out):
    if isinstance(x, dict):
        for k, v in x.items():
            walk(v, path + [str(k)], out)
    elif isinstance(x, list):
        for i, v in enumerate(x):
            walk(v, path + [str(i)], out)
    elif isinstance(x, str):
        if any(p in x for p in pins) or any(n in x for n in names):
            out.append(("/".join(path), x))


for n in ("15", "16"):
    d = json.load(open(os.path.join(WS, "docs/narration-direction-map-oliver%s.json" % n)))
    print("== oliver%s top keys:" % n, list(d)[:30])
    out = []
    walk(d, [], out)
    print("hits", len(out))
    for p, v in out[:14]:
        print("  ", p[:90], "=>", v[-90:])
    if len(out) > 14:
        print("   ...")
        for p, v in out[-4:]:
            print("  ", p[:90], "=>", v[-90:])
    # the parent of the first hit, whole
    if out:
        parts = out[0][0].split("/")
        node = d
        for q in parts[:-1]:
            node = node[int(q)] if isinstance(node, list) else node[q]
        print("first hit parent:", json.dumps(node, ensure_ascii=False)[:900])
