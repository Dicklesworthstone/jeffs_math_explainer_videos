"""Write the coverage table from what was viewed, then base.sha256 and bundle.sha256. Run from the repo root."""

import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
C = HERE.parent.relative_to(Path.cwd())
O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
W = O / "algebra10-repair03-picture-review01"
A = Path("explainer_corpus/abstract_algebra_expanded")
LAB = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
F = LAB / "algebra-chapter10-repair03-frames01/10_cyclic"
R = LAB / "algebra-chapter10-repair03-preview01/render"
TAKES = Path("narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103")
STAGE02 = TAKES / "combined-repair-stage02/explainer_corpus/abstract_algebra_expanded"
AFM = Path("narration_lab/model-sidecar-20261017/.venv-cosyvoice3/lib/python3.10/site-packages/matplotlib/mpl-data/fonts/afm/cmr10.afm")
times = json.loads((F / "sampling.json").read_text())["times"]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


# (sheet, header times read from the image, what was seen)
SHEETS = [
    (3, ["4.500", "5.000", "5.500", "6.000"], "D: the question stands beside a bare clock, marker at 0, no dots, during the general sentences"),
    (25, ["45.500", "46.000", "46.500", "47.000"], "A: question, 0→8→4→0, ⟨8⟩={0,8,4}, ord(8)=3 at the right; nothing about 12; space under ord(8)=3 is empty"),
    (46, ["81.500", "82.000", "82.500", "83.000"], "F: d row arrives by 82.0; gold triangle draws from 82.5, blue and green begin by 83.0; the walk's three gold dots are still on the clock"),
    (47, ["83.500", "84.000", "84.500", "85.000"], "F: purple draws, star whole by 84.0; hours tinted by orbit at 84.5, each hour matching its triangle; `4 orbits, 3 positions each` full by 85.0"),
    (48, ["85.500", "86.000", "86.500", "87.000"], "F: star and tints still at 85.5; at 86.0 only the gold triangle is left and the hours are ink; the label stays; block arcs draw from 86.5"),
    (49, ["87.500", "88.000", "88.500", "89.000"], "F and C: label directly above `n = d n' = 4·3` from 88.0; three labelled blocks; the hop disc lights 0 at 88.0 and 4 by 88.5"),
    (50, ["89.500", "90.000", "90.500", "91.000"], "C: hop at 8 at 89.5 and 90.0 with no k row; k = d k' = 4·2 first at 90.5; gcd(n',k') by 91.0; label still up"),
    (51, ["91.500", "92.000", "92.500", "93.000"], "C and F: n and k rows pulse together at 91.5 with coprime in green below; panel then holds; label directly above n = d n' = 4·3"),
    (52, ["93.500", "94.000", "94.500", "95.000"], "F: page holds, six panel rows, one gold triangle, three labelled blocks; `4 orbits` stands beside three arcs and one triangle"),
    (53, ["95.025", "95.125", "95.325", "95.500"], "F: page still holds, label up"),
    (54, ["95.733", "95.933", "96.000", "96.233"], "F: label, n, d rows at 95.733; at 95.933 setting, d and label are fading; gone at 96.0; rows sliding at 96.233; the gold triangle stays throughout"),
    (71, ["125.500", "126.000", "126.500", "127.000"], "E: gcd row builds under the question title, from 125.5, complete by 126.5; no ord row yet; the ord rule stands above"),
    (80, ["143.500", "144.000", "144.500", "145.000"], "the clash: the ord(k) row reads 12 under 5, 7 and 11, each boxed in green, with ord(k)=12/gcd(k,12) above; +8's red triangle and `3 of 12 positions`"),
    (126, ["225.200", "225.500", "226.000", "226.500"], "B: the proof page fading at 225.2; table faint at 225.5; U(12), title and whole table full by 226.0; right half empty"),
    (140, ["253.000", "253.500", "254.000", "254.500"], "B: U(12), table, squares (hero yellow) and No element (green); right column has clear space at y −0.5 and below `No element`; the diagonal cells 1 are not marked"),
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
    A / "src/cyclic.rs",
    A / "src/axioms.rs",
    A / "src/d4_group.rs",
    A / "src/cosets.rs",
    A / "src/isomorphism.rs",
    A / "src/main.rs",
    A / "script.tsv",
    A / "oracle.py",
    A / "narration_units.json",
    A / "docs/narration-direction-map-oliver24.json",
    STAGE02 / "src/cyclic.rs",
    STAGE02 / "docs/narration-direction-map-oliver26.json",
    W / "REVIEW.md",
    W / "patches/cyclic-picture-followup.patch",
    Path("crates/explainer_kit/src/narration.rs"),
    Path("crates/explainer_kit/src/props.rs"),
    Path("crates/explainer_kit/src/palette.rs"),
    Path("tools/narration_delivery.py"),
    Path("tools/narration_director.py"),
    AFM,
    F / "sampling.json",
    R / "10_cyclic.cues.tsv",
    R / "10_cyclic.mp4",
]
for cue in ("10_1", "10_2", "10_4", "10_6", "10_10"):
    folder = LAB / f"expanded-algebra-{cue.replace('_', '-')}-asr01"
    for take in ("take01", "take02"):
        inputs.append(folder / f"{take}-large.json")
        inputs.append(TAKES / f"{cue}-{take}.wav")


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
print("canonical", digest(A / "src/cyclic.rs"))
print("bundle", digest(C / "bundle.sha256"))
