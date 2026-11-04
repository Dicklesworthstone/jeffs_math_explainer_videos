"""Desk stand-in for `dead_code` and `unused_imports`, which `clippy -D warnings` would turn into failures.

In a binary crate an unused `pub` item still warns. A private item is counted in its own file only; a `pub` one
in its own file, or where another file names it through its module or imports it from there. Test modules do not
count. Five chapters, draw, mod, facts and model resolve in the staged candidate; the rest in canonical.
It works on names, so it can miss a shadowed name; a clean result is not a Clippy pass.
Run from the repo root: python3 -I -B <this file>
"""
import glob
import os
import re
import sys

STAGE = (
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-picture-repair-11-15"
    "/candidate/explainer_corpus/complex_analysis_visual/src/"
)
CANON = "explainer_corpus/complex_analysis_visual/src/"
CHANGED = ["chapters/counting.rs", "chapters/residues.rs", "chapters/real_integral.rs", "chapters/continuing.rs", "chapters/closing.rs", "facts.rs"]


def live(text):
    """The source without its test module and comments; cfg(test) items are cut too."""
    text = text.split("#[cfg(test)]\nmod tests")[0]
    text = re.sub(r"#\[cfg\(test\)\]\n(?:\s*///[^\n]*\n)*\s*(?:pub )?(?:fn|const) \w+[^\n]*\{.*?\n\}\n", "", text, flags=re.S)
    return "\n".join(line.split("//")[0] if '"' not in line else line for line in text.splitlines())


sources = {}
for path in glob.glob(CANON + "**/*.rs", recursive=True):
    rel = os.path.relpath(path, CANON)
    staged = STAGE + rel
    sources[rel] = open(staged if os.path.exists(staged) else path, encoding="utf-8").read()

failed = False
checked = 0
for rel in CHANGED:
    text = live(sources[rel])
    module = os.path.basename(rel)[:-3]
    others = "\n".join(live(s) for r, s in sources.items() if r != rel)
    imports = " ".join(re.findall(r"use (?:super|crate(?:::chapters)?)::" + module + r"::([^;]+);", others, flags=re.S))
    for vis, name in re.findall(r"^\s*(pub(?:\(crate\))? )?(?:fn|const) (\w+)", text, flags=re.M):
        checked += 1
        if name in ("name", "construct", "main"):
            continue
        own = len(re.findall(r"\b" + re.escape(name) + r"\b", text))
        outside = bool(vis) and (
            re.search(r"\b" + module + r"::" + re.escape(name) + r"\b", others)
            or re.search(r"\b" + re.escape(name) + r"\b", imports)
        )
        if own < 2 and not outside:
            print(f"UNUSED outside tests: {rel}: {name}")
            failed = True
    for block in re.findall(r"^use ([^;]+);", text, flags=re.M | re.S):
        tail = block.split("{")[-1].rstrip("}") if "{" in block else block.split("::")[-1]
        for name in re.findall(r"[A-Za-z_]\w*", tail):
            if name in ("self", "prelude") or block.strip().endswith("*"):
                continue
            checked += 1
            body = re.sub(r"^use [^;]+;", "", text, flags=re.M | re.S)
            if not re.search(r"\b" + re.escape(name) + r"\b", body):
                print(f"UNUSED import: {rel}: {name}")
                failed = True
print(f"names checked: {checked}")
print("RESULT", "FAIL" if failed else "PASS")
sys.exit(1 if failed else 0)
