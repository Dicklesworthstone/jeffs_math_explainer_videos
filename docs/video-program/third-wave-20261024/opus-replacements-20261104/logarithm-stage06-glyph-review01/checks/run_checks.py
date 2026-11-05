"""Run every check in this bundle and save its output beside it. No Cargo, no render, no job. Reads the frozen job's files only.

    python3 -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-glyph-review01/checks/run_checks.py

From the repo root. Plain python3 -B, because the picture checks need numpy and PIL. Saved outputs have the home path replaced.
page_map.py rewrites coverage/page-map.tsv and coverage/row-map.tsv from the pages and registry.
"""
import pathlib
import subprocess
import sys

CHECKS = pathlib.Path(__file__).resolve().parent
BUNDLE = CHECKS.parent
PY = [sys.executable, "-B"]
RUNS = [
    ("registry_coverage", PY + ["-I", str(CHECKS / "registry_coverage.py")]),
    ("page_map", PY + [str(CHECKS / "page_map.py"), str(BUNDLE / "coverage/page-map.tsv"), str(BUNDLE / "coverage/row-map.tsv")]),
    ("font_faces", PY + ["-I", str(CHECKS / "font_faces.py")]),
    ("tex_faces", PY + [str(CHECKS / "tex_faces.py"), str(BUNDLE / "evidence/ell-faces.png")]),
    ("source_scan", PY + ["-I", str(CHECKS / "source_scan.py")]),
    ("gaps", PY + [str(CHECKS / "gaps.py")]),
    ("registry_delta", PY + ["-I", str(CHECKS / "registry_delta.py")]),
    (
        "touch_test",
        PY
        + [str(CHECKS / "touch_test.py")]
        + [
            "47:300,865,340,920",
            "47:400,865,620,925",
            "41:50,400,115,455",
            "47:760,385,815,440",
            "48:50,850,110,912",
            "49:100,225,165,272",
            "41:185,765,260,812",
            "42:155,620,210,672",
            "48:565,855,635,910",
        ],
    ),
    ("page_table", PY + ["-I", str(CHECKS / "page_table.py")]),
]
# The enlargements in evidence/. They change nothing and are for looking only.
CROPS = {
    "delta-implication-ell.png": ["2", "47:45,860,1010,930", "41:45,398,125,460"],
    "delta-singular-S.png": ["2", "47:45,380,1060,445"],
    "delta-real-positive.png": ["2", "48:45,850,1000,915"],
    "delta-short-strings.png": ["3", "49:45,715,180,760", "35:45,272,385,322"],
    "minus-en-dash-vs-noto.png": ["3", "39:45,470,200,520", "39:45,925,200,970", "40:45,130,200,175"],
    "superscripts-tight.png": ["3", "42:45,525,220,600", "49:45,215,175,280", "44:45,215,175,285", "44:525,205,610,275", "41:50,760,265,820"],
    "unary-minus-spacing.png": ["3", "47:45,465,330,515", "41:45,478,250,525", "47:45,295,570,362"],
}
failed = []
for name, spec in CROPS.items():
    r = subprocess.run(PY + [str(CHECKS / "crop.py"), str(BUNDLE / "evidence" / name)] + spec, capture_output=True, text=True)
    if r.returncode:
        failed.append(name)
for name, command in RUNS:
    r = subprocess.run(command, capture_output=True, text=True)
    body = (r.stdout + r.stderr).replace(str(pathlib.Path.home()), "<home>").replace(str(pathlib.Path.cwd()), "<repo>")
    (CHECKS / f"{name}.out").write_text(body)
    print(f"{'ok  ' if r.returncode == 0 else 'FAIL'} {name} (exit {r.returncode})")
    if r.returncode:
        failed.append(name)
print("ALL OK" if not failed else f"FAILED: {failed}")
sys.exit(1 if failed else 0)
