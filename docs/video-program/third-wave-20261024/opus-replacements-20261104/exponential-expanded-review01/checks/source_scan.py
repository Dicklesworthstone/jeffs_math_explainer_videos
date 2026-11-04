#!/usr/bin/env python3
"""Source-level scan of the candidates: TEXT indices, unused strings, prediction marks, dead code, bindings.

Reads the originals and writes nothing. Name-based, so not a Clippy pass. Run from the repo root.
"""
import hashlib
import json
import re

ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
SERIES = ROOT + "exponential-expanded-series01/"
PRODUCT = ROOT + "exponential-expanded-product01/"
RATE = ROOT + "exponential-expanded-rate02/"
FACTS = ROOT + "exponential-rate-facts01/"
PRIOR = ROOT + "exponential-expanded-rate01-independent-review/candidate/"


def read(path):
    return open(path, encoding="utf-8").read()


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def table(source, name):
    block = re.search(rf"pub const {name}: &\[&str\] = &\[(.*?)\n\];", source, re.S).group(1)
    return re.findall(r'r?"((?:[^"\\]|\\.)*)"', block)


print("== Independence")
for name in ("rate_law.rs", "rate_model.rs"):
    print(f"  rate02/{name} equals my earlier review candidate: {sha(RATE + name) == sha(PRIOR + name)}")

print("== factorials.rs TEXT use")
source = read(SERIES + "factorials.rs")
text = table(source, "TEXT")
for at, line in enumerate(source.splitlines(), 1):
    for index in re.findall(r"TEXT\[(\d+)\]", line):
        print(f"  line {at}: TEXT[{index}] = {text[int(index)]!r}")
used = {int(i) for i in re.findall(r"TEXT\[(\d+)\]", source)} | {3, 4, 5, 6, 7}
literals = set(re.findall(r'"([^"]*)"', source.split("impl SceneConstruct")[1]))
for i, entry in enumerate(text):
    if i not in used and entry not in literals:
        print(f"  never drawn: TEXT[{i}] = {entry!r}")

print("== product_grid.rs TEXT use")
source = read(PRODUCT + "product_grid.rs")
text = table(source, "TEXT")
used = {int(i) for i in re.findall(r"TEXT\[(\d+)\]", source)}
for i, entry in enumerate(text):
    print(f"  TEXT[{i}] {'drawn' if i in used else 'NEVER DRAWN'}: {entry!r}")

print("== prediction holds")
for label, path in (("rate_law", RATE + "rate_law.rs"), ("factorials", SERIES + "factorials.rs"), ("product_grid", PRODUCT + "product_grid.rs")):
    scene = read(path).split("#[cfg(test)]")[0]
    print(f"  {label}: predict mark {'predict' in scene}, music_breath {'music_breath' in scene}, bare wait(4.0) {'wait(4.0)' in scene}")

print("== names used only in tests (a binary crate warns on these under -D warnings)")
for path in (SERIES + "series_model.rs", PRODUCT + "product_model.rs", FACTS + "candidate/src/rate_model.rs"):
    source = read(path)
    body, _, tests = source.partition("#[cfg(test)]")
    scene = ""
    if "series_model" in path:
        scene = read(SERIES + "factorials.rs")
    if "product_model" in path:
        scene = read(PRODUCT + "product_grid.rs")
    for name in re.findall(r"pub (?:fn|const) (\w+)", body):
        inside = len(re.findall(rf"\b{name}\b", body)) - 1
        outside = len(re.findall(rf"\b{name}\b", scene))
        in_tests = len(re.findall(rf"\b{name}\b", tests))
        if inside + outside == 0:
            print(f"  {path.rsplit('/', 1)[1]}: {name} has no non-test caller (tests: {in_tests})")

print("== helpers the scenes call that a model file must export")
for label, path, model in (("factorials", SERIES + "factorials.rs", SERIES + "series_model.rs"), ("product_grid", PRODUCT + "product_grid.rs", PRODUCT + "product_model.rs")):
    imports = re.search(r"use crate::\w+::\{([^}]*)\}", read(path)).group(1)
    names = [n.strip() for n in imports.split(",") if n.strip()]
    exported = re.findall(r"pub (?:fn|const) (\w+)", read(model))
    print(f"  {label}: imports {names}; all exported: {all(n in exported for n in names)}")

print("== facts01: candidate rate_model is the reviewed model plus append_facts")
old = read(RATE + "rate_model.rs")
new = read(FACTS + "candidate/src/rate_model.rs")
added = re.search(r"\n/// Export the same checkpoints.*?\n}\n", new, re.S).group(0)
print("  equal after removing append_facts:", new.replace(added, "\n", 1).replace("\n\n\n", "\n\n") == old.replace("\n\n\n", "\n\n"))
bindings = json.load(open(FACTS + "bindings.json", encoding="utf-8"))
for change in bindings["changes"]:
    candidate = FACTS + "candidate/" + change["path"].split("exponential_magic/")[1]
    canonical = change["path"]
    before = change["before_sha256"]
    state = "new file" if before is None else ("baseline equals canonical" if sha(canonical) == before else "CANONICAL MOVED")
    print(f"  {change['path'].split('/')[-1]}: candidate hash ok {sha(candidate) == change['sha256']}; {state}")

print("== stale statements")
readme = read(RATE + "README.md")
words = sum(len(row.split("\t", 1)[1].split()) for row in read(RATE + "script-slice.tsv").splitlines())
print(f"  rate02 README says 244 words: {'244 words' in readme}; the slice has {words}")
tests = len(re.findall(r"#\[test\]", read(RATE + "rate_model.rs")))
print(f"  rate02 README says two Rust tests in rate_model.rs: {'two meaningful Rust tests' in readme}; it has {tests}")
print(f"  series README cites 324 tail samples; series oracle.py computes tail samples: {'324' in read(SERIES + 'oracle.py')}")
print(f"  series README says the bound shows upward-rounded: {'upward-rounded' in read(SERIES + 'README.md')}; series_model.rs rounds up: {'ceil' in read(SERIES + 'series_model.rs')}")
print(f"  product oracle emits fact rows: {'print(f' in read(PRODUCT + 'oracle.py')}; product_model has append_facts: {'append_facts' in read(PRODUCT + 'product_model.rs')}")
