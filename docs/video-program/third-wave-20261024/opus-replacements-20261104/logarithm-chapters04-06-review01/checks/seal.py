"""Write and verify base.sha256 and bundle.sha256. Run from the repo root."""

import hashlib
from pathlib import Path

HERE = Path(__file__).resolve().parent
C = HERE.parent.relative_to(Path.cwd())
CR = Path("explainer_corpus/logarithm_mysteries")
CH = CR / "src/chapters"
F = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-combined-repair-frames01")
R = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-combined-repair-preview01/render")
CHAPTERS = ("04_undoing_growth", "05_complex_turns", "06_lifting_loops")

inputs = [CH / n for n in ("undoing_growth.rs", "complex_turns.rs", "lifting_loops.rs", "draw.rs")]
inputs += [CR / "script.tsv", CR / "oracle.py", CR / "docs/check_cues.py"]
inputs += [Path("crates/explainer_kit/src") / n for n in ("kit.rs", "play.rs", "narration.rs")]
for name in CHAPTERS:
    inputs.append(F / name / "sampling.json")
    inputs += [R / f"{name}.mp4", R / f"{name}.cues.tsv", R / f"{name}.layout.tsv"]


def line(path):
    return f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path}\n"


(C / "base.sha256").write_text("".join(line(p) for p in inputs))
files = sorted(p for p in C.rglob("*") if p.is_file() and p.name != "bundle.sha256")
(C / "bundle.sha256").write_text("".join(line(p) for p in files))
print(len(inputs), "inputs;", len(files), "bundle files")

bad = 0
for name in ("base.sha256", "bundle.sha256"):
    n = 0
    for row in (C / name).read_text().splitlines():
        digest, path = row.split("  ", 1)
        n += 1
        if hashlib.sha256(Path(path).read_bytes()).hexdigest() != digest:
            bad += 1
            print("BAD", path)
    print(name, n, "lines")
print("mismatches", bad)
home = ("/" + "Users/").encode()
for path in sorted(p for p in C.rglob("*") if p.is_file()):
    if home in path.read_bytes():
        print("HOME PATH", path)
for name in inputs[:4]:
    print(hashlib.sha256(name.read_bytes()).hexdigest(), name.name)
print("bundle", hashlib.sha256((C / "bundle.sha256").read_bytes()).hexdigest())
