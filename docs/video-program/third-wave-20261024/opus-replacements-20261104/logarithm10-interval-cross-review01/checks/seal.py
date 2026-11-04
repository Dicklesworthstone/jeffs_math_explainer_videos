"""Write coverage tables from what was viewed, then base.sha256 and bundle.sha256. Run from the repo root."""

import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
C = HERE.parent.relative_to(Path.cwd())
O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
CAND = O / "logarithm10-interval-repair01"
CR = Path("explainer_corpus/logarithm_mysteries")
F = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-combined-repair-frames01/10_doing_nothing")
R = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-combined-repair-preview01/render")
PLAN = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage05/bound-plan")
times = json.loads((F / "sampling.json").read_text())["times"]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


# (sheet, header times read from the image, what was seen)
SHEETS = [
    (35, ["62.500", "63.000", "63.500", "64.000"], "lap rings and ellipse flows: ±2π absent at 62.5 and 63.0, full by 63.5; closed green dot at π under a flow's turning; page then holds"),
    (37, ["66.400", "66.500", "66.700", "67.000"], "hold; no rule; π closed"),
    (38, ["67.100", "67.300", "67.500", "67.600"], "rule barely begun at 67.1, part faded at 67.3, full at 67.5; π filled throughout"),
    (39, ["68.000", "68.500", "69.000", "69.500"], "rule full; π still filled at all four: the contradiction root reports"),
    (40, ["70.000", "70.500", "71.000", "71.500"], "70.0 mid-swap, ring over a fading dot; crosses on ±2π by 70.5; rule full; both ends rings"),
    (41, ["72.000", "72.500", "73.000", "73.500"], "flow words leaving at 72.5; condition lines by 73.0; rule stays; ends stay open"),
    (50, ["90.000", "90.500", "91.000", "91.500"], "rule still up; ends open; dim fine print at the bottom, including −π < Im < π, which agrees with the open ends; page holds"),
]
rows = ["sheet\tpath\tsha256\tpictures\ttimes_read_from_image\tseen"]
for n, read, seen in SHEETS:
    path = F / f"sheet-{n:03d}.png"
    frames = list(range(4 * n, 4 * n + 4))
    for k, r in zip(frames, read):
        if abs(times[k] - float(r)) > 0.0015:
            raise SystemExit(f"sheet {n}: read {r}, sampling {times[k]}")
    rows.append(f"sheet-{n:03d}\t{path}\t{digest(path)}\t{frames[0]}-{frames[-1]}\t{' '.join(read)}\t{seen}")
(C / "coverage" / "sheets-viewed.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")

FRAMES = [
    (159, "π filled green, −π ring, rule full: the disagreement at full size"),
    (163, "both ends rings, crosses on ±2π; the 6 pt green window line runs to each ring's centre, so the ring is not fully hollow, though it stays distinct from frame 159's filled dot"),
]
rows = ["image\ttime_s\tsha256\tseen"]
for n, seen in FRAMES:
    path = F / f"frame-{n:04d}.png"
    rows.append(f"{path}\t{times[n]:.3f}\t{digest(path)}\t{seen}")
(C / "coverage" / "viewed-frames.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")

inputs = [
    CAND / "REVIEW.md",
    CAND / "before-doing_nothing.rs",
    CAND / "doing_nothing.rs",
    CAND / "interval.patch",
    CAND / "receipt.json",
    CR / "src/chapters/doing_nothing.rs",
    CR / "src/chapters/draw.rs",
    CR / "script.tsv",
    CR / "docs/check_cues.py",
    CR / "docs/storyboard.md",
    Path("crates/explainer_kit/src/narration.rs"),
    F / "sampling.json",
    R / "10_doing_nothing.mp4",
    R / "10_doing_nothing.cues.tsv",
    PLAN / "phrase-anchors.json",
    PLAN / "hold-audit.json",
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
print("canonical", digest(CR / "src/chapters/doing_nothing.rs"))
print("bundle", digest(C / "bundle.sha256"))
