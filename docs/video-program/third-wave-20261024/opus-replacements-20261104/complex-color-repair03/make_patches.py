"""Build this bundle's patches from candidate/ against the crate's current bytes. Read-only outside the bundle.

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-color-repair03/make_patches.py

Writes a new patches/ directory and refuses one that exists. Paths in the patches are repo-root-relative, for
patch -p1 --fuzz=0 or git apply from the repo root. New files carry /dev/null as their old side.
"""
import difflib
import pathlib

ROOT = pathlib.Path.cwd()
BUNDLE = pathlib.Path(__file__).resolve().parent
CANDIDATE = BUNDLE / "candidate/explainer_corpus/complex_analysis_visual"
CRATE = "explainer_corpus/complex_analysis_visual"
OUT = BUNDLE / "patches"


def diff(rel):
    old = ROOT / CRATE / rel
    new = CANDIDATE / rel
    a = old.read_text().splitlines(True) if old.exists() else []
    b = new.read_text().splitlines(True)
    src = f"a/{CRATE}/{rel}" if old.exists() else "/dev/null"
    return "".join(difflib.unified_diff(a, b, src, f"b/{CRATE}/{rel}", n=3))


def select(name, rung, budget, note):
    """A patch against candidate draw.rs flipping the film's choice, and the tests that hold it."""
    rel = "src/chapters/draw.rs"
    a = (CANDIDATE / rel).read_text()
    b = a
    for old, new in [
        ("pub const PLATE: PlateStyle = PLATE_NOW;\n", f"pub const PLATE: PlateStyle = {rung};\n"),
        ("assert_eq!(PLATE, PLATE_NOW);\n", f"assert_eq!(PLATE, {rung});\n"),
        (f"#[allow(dead_code)] // for examples/plate_lab.rs, until root chooses\npub const {rung}:", f"pub const {rung}:"),
    ]:
        assert b.count(old) == 1, old
        b = b.replace(old, new)
    if budget is not None:
        old = 'assert!(rows <= 200 && cols <= 200, "{cols} × {rows}");\n'
        new = f'assert!(rows <= {budget} && cols <= {budget}, "{{cols}} × {{rows}}");\n'
        assert b.count(old) == 1, old
        b = b.replace(old, new)
    text = "".join(difflib.unified_diff(a.splitlines(True), b.splitlines(True), f"a/{CRATE}/{rel}", f"b/{CRATE}/{rel}", n=3))
    (OUT / name).write_text(text)
    print(name, note)


OUT.mkdir(exist_ok=False)
(OUT / "A-plate-core.patch").write_text(diff("src/plate_layout.rs") + diff("src/main.rs") + diff("src/chapters/draw.rs"))
(OUT / "B-plate-lab.patch").write_text(diff("examples/plate_lab.rs"))
select("select-tile.patch", "PLATE_TILE", None, "after A: tiling rows only; removes the stripes, leaves the steps")
select("select-blend.patch", "PLATE_BLEND", None, "after A: tiling rows, colours run along each row")
select("select-rows2.patch", "PLATE_ROWS2", 400, "after A: and rows half as high; raises the strip budget test to 400")
select("select-cells2.patch", "PLATE_CELLS2", 400, "after A: and cells half the size; raises the strip budget test to 400")
