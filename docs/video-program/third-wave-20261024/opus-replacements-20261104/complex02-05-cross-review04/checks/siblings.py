"""Removed public items still named elsewhere; binder with all pending sibling patches; card widths."""
import importlib.util, json, re, shutil, subprocess, sys, tempfile
from pathlib import Path
sys.dont_write_bytecode = True
O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
X = O / "complex02-05-cross-review04"
R = O / "complex02-05-repair03"
REL = Path("explainer_corpus/complex_analysis_visual")
patch = (R / "complex02-05-repair03.patch").read_text(encoding="utf-8")
removed = set(re.findall(r"^-\s*pub (?:const|fn|struct) (\w+)", patch, re.M)) - set(re.findall(r"^\+\s*pub (?:const|fn|struct) (\w+)", patch, re.M))
print("public items removed by the patch:", sorted(removed))
staged = {p.name: p.read_text(encoding="utf-8") for p in (R / REL / "src/chapters").glob("*.rs")}
for p in sorted((REL / "src").rglob("*.rs")):
    text = staged.get(p.name, p.read_text(encoding="utf-8"))
    for name in removed:
        for m in re.finditer(rf"\b{name}\b", text):
            print("  still named:", p.name, text.count("\n", 0, m.start()) + 1, name)

# Binder on the staged tree plus every pending sibling patch that fits canonical
spec = importlib.util.spec_from_file_location("binder", X / "binder/bind_oliver_plan.py")
binder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(binder)
tmp = Path(tempfile.mkdtemp(prefix="cross04-all-"))
shutil.copytree(REL, tmp / REL, ignore=shutil.ignore_patterns("target"))
for f in ("turning", "two_planes", "reading_color", "gallery"):
    shutil.copyfile(R / REL / "src/chapters" / f"{f}.rs", tmp / REL / "src/chapters" / f"{f}.rs")
for rel in ("complex02-05-cross-review04/rebinding-tex-storyboard.patch", "complex-glyph-spacing-repair01/complex-glyph-spacing.patch", "complex11-counting-review-repair01/patches/01-reach-label.patch", "complex11-counting-review-repair01/patches/02-storyboard-11_7.patch", "complex13-radius-repair01/complex13-radius-repair01.patch"):
    done = subprocess.run(["git", "apply", "--whitespace=nowarn", str((O / rel).resolve())], cwd=tmp, capture_output=True, text=True)
    print("applied" if done.returncode == 0 else "REFUSED", rel, done.stderr[:180].strip())
plan, outputs, _a, _c, text = binder.build(".", str(tmp / REL), "oliver11")
print("binder problems with all siblings:", len(plan.problems), plan.problems[:6])
holds = json.loads(text)["visual_holds"]
print("hold rows", len(holds))

# binder CLI names
src = (X / "binder/bind_oliver_plan.py").read_text(encoding="utf-8")
print(re.findall(r'add_argument\("([^"]+)"', src))
print([l.strip()[:150] for l in src.split("\n") if "already exists in canonical" in l or "def main" in l])
for rel in ("complex11-counting-teaching02",):
    d = O / rel
    if d.exists():
        for p in sorted(d.rglob("*")):
            if p.suffix in (".patch", ".diff") or p.name == "counting.rs":
                t = p.read_text(encoding="utf-8", errors="replace")
                print(p.relative_to(O), "names:", [c for c in ("NEEDS_SAID[i]", "line.sentence(i + 3)", "line.wait_sentence(stage, i + 2)") if c in t])
