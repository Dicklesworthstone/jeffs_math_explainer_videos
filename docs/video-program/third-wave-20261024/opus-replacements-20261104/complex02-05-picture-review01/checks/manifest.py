"""Write the sheet manifest and cue map for complex chapters 02 to 05, and report what preview02 was built from.

python3 -I -B manifest.py OUT_DIR   (from the repo root)

Writes OUT_DIR/coverage/sheets.tsv and OUT_DIR/cues.txt. Reads only otherwise. A sheet in the manifest is a sheet that
exists. A row saying it was viewed is added by hand-run viewed.py after the Read tool has opened it.
"""
import hashlib
import json
import sys
from pathlib import Path

B = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
FRAMES = B / "complex-combined-picture-frames02"
RENDER = B / "complex-combined-picture-preview02/render"
STAGE = Path("narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-picture-stage01")
CHAPTERS = ["02_turning_numbers", "03_two_planes", "04_reading_color", "05_gallery"]
CRATE = "explainer_corpus/complex_analysis_visual"
FILES = {
    "02_turning_numbers": "turning.rs",
    "03_two_planes": "two_planes.rs",
    "04_reading_color": "reading_color.rs",
    "05_gallery": "gallery.rs",
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    out = Path(sys.argv[1])
    (out / "coverage").mkdir(parents=True, exist_ok=True)
    script = {}
    for line in Path(CRATE, "script.tsv").read_text(encoding="utf-8").splitlines():
        cells = line.split("\t")
        if len(cells) >= 2:
            script[cells[0]] = cells[-1]
    rows = ["chapter\tsheet\tsha256\tframes\ttimes"]
    cue_lines = []
    totals = {}
    for chapter in CHAPTERS:
        folder = FRAMES / chapter
        times = json.loads((folder / "sampling.json").read_text(encoding="utf-8"))
        cues = [ln.split("\t") for ln in (RENDER / f"{chapter}.cues.tsv").read_text(encoding="utf-8").splitlines()]
        sheets = sorted(folder.glob("sheet-*.png"))
        count = len(times["times"])
        if len(sheets) != (count + 3) // 4:
            raise SystemExit(f"{chapter}: {len(sheets)} sheets for {count} frames")
        for index, sheet in enumerate(sheets):
            frames = list(range(index * 4, min(index * 4 + 4, count)))
            rows.append("\t".join([
                chapter, sheet.name, sha(sheet), ",".join(map(str, frames)),
                ",".join(f"{times['times'][f]:.3f}" for f in frames),
            ]))
        totals[chapter] = (len(sheets), count, times["duration"])
        cue_lines.append(f"\n## {chapter}: {len(sheets)} sheets, {count} frames, duration {times['duration']:.3f}")
        for cue, start, length in cues:
            start, length = float(start), float(length)
            first = next((i for i, t in enumerate(times["times"]) if t >= start), None)
            cue_lines.append(f"{cue}  {start:8.3f} to {start + length:8.3f}  frame {first} sheet {first // 4 if first is not None else '-'}")
            cue_lines.append(f"    {script.get(cue, '??')}")
    (out / "coverage/sheets.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")
    (out / "cues.txt").write_text("\n".join(cue_lines) + "\n", encoding="utf-8")
    print("totals", totals, "sheets", sum(v[0] for v in totals.values()), "frames", sum(v[1] for v in totals.values()))

    inputs = json.loads((STAGE / "preview-inputs02.json").read_text(encoding="utf-8"))["files"]
    tail = {item["path"].split("jeffs_math_explainer_videos/", 1)[-1]: item["sha256"] for item in inputs}
    print("preview inputs", len(tail))
    candidate = STAGE / "candidate" / CRATE
    print("candidate exists", candidate.exists())
    for name in [*FILES.values(), "draw.rs", "mod.rs"]:
        rel = f"{CRATE}/src/chapters/{name}"
        rows = [k for k in tail if k.endswith(f"/chapters/{name}")]
        now = sha(rel) if Path(rel).exists() else "none"
        print(name, "canonical", now[:12], "bound", [(k.split('/')[-6:][0], tail[k][:12]) for k in rows][:3])
    review = json.loads((STAGE / "picture-review-progress01.json").read_text(encoding="utf-8"))
    print("root review", review.get("at"), review.get("reviewer"), {k: (str(v)[:400]) for k, v in review.items() if k not in ("sheets_actually_viewed",)})
    seen = {}
    for row in review["sheets_actually_viewed"]:
        seen.setdefault(row["path"].split("/")[-2], []).append(row["path"].split("/")[-1])
    print("root viewed", {k: len(v) for k, v in seen.items()})


if __name__ == "__main__":
    main()
