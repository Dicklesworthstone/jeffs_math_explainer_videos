"""Write the coverage table from what was viewed, then base.sha256 and bundle.sha256. Run from the repo root."""

import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
C = HERE.parent.relative_to(Path.cwd())
O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
A = Path("explainer_corpus/abstract_algebra_expanded")
ROOT = Path("narration_lab/third-wave-20261024/root")
LAB = ROOT / "five-film-release-20261027"
TAKES = ROOT / "elevenlabs-expanded-algebra-20261103"
T = TAKES / "applications-repair-stage01"
STAGE02 = TAKES / "combined-repair-stage02"
F = LAB / "algebra-combined-repair-frames01/17_applications"
R = LAB / "algebra-combined-repair-preview01/render"
AFM = Path("narration_lab/model-sidecar-20261017/.venv-cosyvoice3/lib/python3.10/site-packages/matplotlib/mpl-data/fonts/afm")
times = json.loads((F / "sampling.json").read_text())["times"]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


# (sheet, header times read from the image, what was seen). All baseline: no picture of candidate02 exists.
SHEETS = [
    (205, ["361.000", "361.290", "361.390", "361.500"], "17_11's page just before 17_12: star of four triangles, three rows; all of it goes when 17_12 starts"),
    (208, ["364.500", "365.000", "365.500", "366.000"], "baseline prompts: all three already up at 364.5, centred; prompt 2 still says equal; prompt 3 as today; no answer on screen"),
    (211, ["370.000", "370.500", "371.000", "371.500"], "same three prompts held, nothing else on screen"),
    (223, ["389.000", "389.500", "390.000", "390.500"], "answers title and dim clock with 0 and 6 in hero; ring travels 0, 6, 0; right half empty"),
    (225, ["393.000", "393.500", "393.700", "394.000"], "answer page: two rows at the right, hH=H (h in H) in green at y -1, Closure line at y -2.5; clear band between them, below the clock"),
    (230, ["402.000", "402.500", "403.000", "403.500"], "same page held; frame 920 is the one measured; nothing moves"),
    (236, ["412.918", "413.000", "413.018", "413.218"], "same page to the chapter's last samples; the last sentence has no picture of its own"),
]
rows = ["sheet\tpath\tsha256\tpictures\ttimes_read_from_image\tseen"]
for n, read, seen in SHEETS:
    path = F / f"sheet-{n:03d}.png"
    frames = list(range(4 * n, 4 * n + 4))
    for k, r in zip(frames, read):
        if abs(times[k] - float(r)) > 0.0015:
            raise SystemExit(f"sheet {n}: read {r}, sampling {times[k]}")
    rows.append(f"sheet-{n:03d}\t{path}\t{digest(path)}\t{frames[0]}-{frames[-1]}\t{' '.join(read)}\t{seen}")
(C / "coverage").mkdir(exist_ok=True)
(C / "coverage" / "sheets-viewed.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")

inputs = [
    A / "src/applications.rs",
    A / "src/cosets.rs",
    A / "src/axioms.rs",
    A / "src/main.rs",
    A / "script.tsv",
    T / "baseline/applications.rs",
    T / "baseline/script.tsv",
    T / "baseline/narration-direction-map-oliver24.json",
    T / "candidate/applications.rs",
    T / "candidate02/applications.rs",
    T / "candidate02/script.tsv",
    T / "candidate02/narration_units.json",
    T / "candidate02/narration-direction-map-applications-proposed.json",
    T / "coset-answer-followup.patch",
    T / "coset-orientation-check01.json",
    T / "assembly01.json",
    STAGE02 / "assembly01.json",
    STAGE02 / "explainer_corpus/abstract_algebra_expanded/src/applications.rs",
    STAGE02 / "explainer_corpus/abstract_algebra_expanded/script.tsv",
    STAGE02 / "explainer_corpus/abstract_algebra_expanded/docs/narration-direction-map-oliver26.json",
    TAKES / "current-retained-coverage40.json",
    O / "algebra17-patch-cross-review01/candidate-C1C2/applications.rs",
    O / "algebra17-patch-cross-review01/REVIEW.md",
    O / "algebra-picture-review-17_applications/REVIEW.md",
    Path("crates/explainer_kit/src/narration.rs"),
    Path("crates/explainer_kit/src/kit.rs"),
    AFM / "cmr10.afm",
    AFM / "cmmi10.afm",
    AFM / "cmsy10.afm",
    F / "sampling.json",
    F / "frame-0836.png",
    F / "frame-0920.png",
    R / "17_applications.cues.tsv",
    R / "17_applications.mp4",
]


def line(path):
    return f"{digest(path)}  {path}\n"


(C / "base.sha256").write_text("".join(line(p) for p in inputs))
files = sorted(p for p in C.rglob("*") if p.is_file() and p.name != "bundle.sha256")
(C / "bundle.sha256").write_text("".join(line(p) for p in files))
print(len(inputs), "inputs;", len(files), "bundle files")

bad = 0
for name in ("base.sha256", "bundle.sha256"):
    for row in (C / name).read_text().splitlines():
        want, path = row.split("  ", 1)
        if digest(Path(path)) != want:
            bad += 1
            print("BAD", path)
print("mismatches", bad)
home = ("/" + "Users/").encode()
for path in sorted(p for p in C.rglob("*") if p.is_file()):
    if home in path.read_bytes():
        print("HOME PATH", path)
print("canonical", digest(A / "src/applications.rs"))
print("bundle", digest(C / "bundle.sha256"))
