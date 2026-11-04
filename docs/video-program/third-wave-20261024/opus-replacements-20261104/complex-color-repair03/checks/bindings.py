"""Two string checks that need no engine: what the other complex lanes' patches touch, and the source scans in the unrun tests.

    python3 -I -B .../complex-color-repair03/checks/bindings.py

1. Each staged patch from complex02-05-repair03 and complex06-09-repair03, with its hash: does it touch draw.rs or main.rs,
   and does it move a plate literal the lab pins?
2. The literal counts in draw.rs's every_colour_picture_is_crisp, and every `contains` in plate_lab.rs's
   the_plates_are_the_films, evaluated here on the candidate's text. Those two Rust tests have still not run. This only says
   their strings hold today.
Exit 1 on any conflict. Reads only.
"""
import hashlib
import pathlib
import re
import sys

BUNDLE = pathlib.Path(__file__).resolve().parent.parent
LANES = BUNDLE.parent
CRATE = BUNDLE / "candidate/explainer_corpus/complex_analysis_visual"
failed = False


def report(ok, text):
    global failed
    failed = failed or not ok
    print(f"{'PASS' if ok else 'FAIL'}  {text}")


lab = (CRATE / "examples/plate_lab.rs").read_text()
start = lab.index("fn the_plates_are_the_films(")
body = lab[start : lab.index("\n    }\n", start)]
files = dict(re.findall(r'let (\w+) = include_str!\("\.\./src/chapters/([\w.]+)"\);', body))
literals = []
for name, escaped in re.findall(r'assert!\(\s*(\w+)\s*\.contains\(\s*"((?:[^"\\]|\\.)*)"\s*\)', body):
    literals.append((name, escaped.replace('\\"', '"')))
report(len(files) >= 5 and len(literals) >= 7, f"the_plates_are_the_films parsed: {len(files)} files, {len(literals)} literals")

print("\nOther lanes' staged patches")
patches = sorted(list(LANES.glob("complex02-05-repair03/*.patch")) + list(LANES.glob("complex06-09-repair03/patches/*.patch")))
report(len(patches) == 5, f"{len(patches)} patches found")
for path in patches:
    text = path.read_text()
    touched = sorted({line[6:] for line in text.splitlines() if line.startswith(("--- a/", "+++ b/"))})
    ours = [t for t in touched if t.endswith(("chapters/draw.rs", "src/main.rs"))]
    changed = [line[1:] for line in text.splitlines() if line[:1] in "+-" and not line.startswith(("---", "+++"))]
    moved = sorted({lit for _, lit in literals if any(lit in line for line in changed)})
    sha = hashlib.sha256(path.read_bytes()).hexdigest()
    print(f"  {path.parent.name}/{path.name} {sha}")
    print(f"    touches: {', '.join(pathlib.PurePosixPath(t).name for t in touched)}")
    report(not ours and not moved, f"{path.name}: touches draw.rs or main.rs: {ours or 'no'}; moves a pinned literal: {moved or 'no'}")

print("\nThe source scans, on the candidate's text")
draw = (CRATE / "src/chapters/draw.rs").read_text()
code = draw[: draw.index("#[cfg(test)]\nmod tests")]
for needle, want in [
    ("RowStrips::build(stage", 1),
    ("blended_rows(stage", 1),
    ("Picture::build(", 2),
    ("crisp(stage, picture.rows(), cell_h, style);", 1),
]:
    seen = code.count(needle)
    report(seen == want, f"every_colour_picture_is_crisp: {needle!r} appears {seen}, wants {want}")
for name, lit in literals:
    source = (CRATE / "src/chapters" / files[name]).read_text() if (CRATE / "src/chapters" / files[name]).exists() else None
    if source is None:
        source = (BUNDLE.parents[4] / "explainer_corpus/complex_analysis_visual/src/chapters" / files[name]).read_text()
    report(lit in source, f"the_plates_are_the_films: {files[name]} contains {lit}")

print("\nRESULT", "FAILED" if failed else "OK")
sys.exit(1 if failed else 0)
