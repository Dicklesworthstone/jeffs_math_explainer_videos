"""List every chapter 05-08 sheet with its hash, frames and times, and the cue each frame falls in.

Reads only. Standard library. Writes the two tables this review's coverage and anchors rest on:

    manifest/sheets.tsv    chapter, sheet, sha256, bytes, first frame, last frame, first time, last time
    manifest/cues.tsv      chapter, cue, start, end, words of the cue as the stage's inputs hashed them

    python3 -I -B build_manifest.py
"""
import hashlib
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE.parent
REPO = HERE.parents[5]
B = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
FRAMES = B / "logarithm-stage06-frames01"
RENDER = B / "logarithm-stage06-preview01/render"
INPUTS = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/stage06-preview01/inputs.json"
LOG = REPO / "explainer_corpus/logarithm_mysteries"
CHAPTERS = ["05_complex_turns", "06_lifting_loops", "07_branch_cut", "08_product_rule"]
SOURCES = ["script.tsv", "docs/storyboard.md", "src/model.rs", "src/chapters/draw.rs", "src/chapters/complex_turns.rs", "src/chapters/lifting_loops.rs", "src/chapters/branch_cut.rs", "src/chapters/product_rule.rs"]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    (OUT / "manifest").mkdir(exist_ok=True)
    script = {}
    for line in (LOG / "script.tsv").read_text(encoding="utf-8").splitlines():
        if "\t" in line:
            key, text = line.split("\t", 1)
            script[key] = text

    sheets = ["chapter\tsheet\tsha256\tbytes\tfirst_frame\tlast_frame\tfirst_s\tlast_s"]
    cues = ["chapter\tcue\tstart_s\tend_s\ttext"]
    for chapter in CHAPTERS:
        folder = FRAMES / chapter
        sampling = json.loads((folder / "sampling.json").read_text(encoding="utf-8"))
        times = sampling["times"]
        found = sorted(folder.glob("sheet-*.png"))
        frames = sorted(folder.glob("frame-*.png"))
        assert len(frames) == len(times), chapter
        assert len(found) == (len(times) + 3) // 4, chapter
        for n, path in enumerate(found):
            assert path.name == f"sheet-{n:03}.png", path
            first = n * 4
            last = min(first + 3, len(times) - 1)
            sheets.append(f"{chapter}\t{n:03}\t{sha(path)}\t{path.stat().st_size}\t{first}\t{last}\t{times[first]:.3f}\t{times[last]:.3f}")
        rows = []
        for line in (RENDER / f"{chapter}.cues.tsv").read_text(encoding="utf-8").splitlines():
            key, start, length = line.split("\t")
            rows.append((float(start), float(length), key))
        for start, length, key in sorted(rows):
            cues.append(f"{chapter}\t{key}\t{start:.3f}\t{start + length:.3f}\t{script.get(key, 'NOT IN CURRENT SCRIPT')}")
        print(f"{chapter}: {len(found)} sheets, {len(frames)} pictures, duration {sampling['duration']:.3f} s, cues {len(rows)}, lint anchors {len(sampling['layout_findings'])}")
    (OUT / "manifest/sheets.tsv").write_text("\n".join(sheets) + "\n", encoding="utf-8")
    (OUT / "manifest/cues.tsv").write_text("\n".join(cues) + "\n", encoding="utf-8")
    print(f"total sheets {len(sheets) - 1}")

    hashed = {f["path"]: f["sha256"] for f in json.loads(INPUTS.read_text(encoding="utf-8"))["files"]}
    print("source now against what the preview's inputs hashed:")
    for rel in SOURCES:
        path = LOG / rel
        was = hashed.get(str(path))
        print(f"  {rel}: {'same' if was == sha(path) else 'CHANGED since preview' if was else 'not in inputs'}  {sha(path)[:12]}")


if __name__ == "__main__":
    main()
