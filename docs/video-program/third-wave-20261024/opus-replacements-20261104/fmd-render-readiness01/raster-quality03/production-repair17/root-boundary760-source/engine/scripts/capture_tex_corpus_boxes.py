#!/usr/bin/env python3
"""Capture real TeX's box dimensions for a stratified corpus sample
(fm-tex-layout-oracle-bkbc).

TeX itself reports each formula's box: width, height and depth, from \\wd,
\\ht and \\dp of a saved box, in display style. The preamble is the pinned
Reference's default template (scripts/manim_ref/manimlib/tex_templates.yml,
packages and \\minus), and the document class is 10 pt, so 1 em = 10 pt.
Each string is stripped, as the Reference strips it. A single line is braced
as amsmath braces an align* cell; multi-line strings (`\\\\` or `&`) are
boxed as a top-aligned `aligned`, the inner form of the Reference's align*
wrapper, without its invisible row struts and trailing column glue. No SVG
conversion or Reference SVG cache is involved, so the Reference's cache race
(Appendix C-22, fm-0v8k) cannot reach these boxes.

The corpus is a private fixture (plan §15.3): its strings never ship. The
committed fixture `crates/fmn-conformance/fixtures/tex_corpus_boxes.v1.tsv`
carries only each string's public digest sha256(mode + NUL + string), the
box in ems, its occurrence count and its construct classes. The oracle test
(`crates/fmn-conformance/tests/tex_corpus_boxes.rs`) finds the strings again
through the private corpus where it is present.

Sample: corpus math strings, up to PER_CLASS per construct class (by
occurrence, digest tie-break), then the most frequent remaining strings until
TARGET candidates. Strings TeX cannot typeset alone (fragments of
multi-argument Tex calls) are left out and counted.

Usage:
  capture_tex_corpus_boxes.py --corpus corpus/tex_corpus.jsonl [--jobs 8]
  (needs `latex` from the TeX installation the Reference uses)
"""
from __future__ import annotations

import argparse
import concurrent.futures
import json
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "crates" / "fmn-conformance" / "fixtures" / "tex_corpus_boxes.v1.tsv"
TEMPLATES = ROOT / "scripts" / "manim_ref" / "manimlib" / "tex_templates.yml"
PER_CLASS = 4
TARGET = 640
BOX = re.compile(r"FMNBOX ([0-9.]+)pt ([0-9.]+)pt ([0-9.]+)pt")


def sample(corpus: pathlib.Path) -> list[dict]:
    records = [json.loads(line) for line in corpus.open(encoding="utf-8")]
    math = [r for r in records if r["mode"] == "math" and r["text"].strip()]
    ranked = sorted(math, key=lambda r: (-r["count"], r["sha256"]))
    classes: dict[str, list[dict]] = {}
    for r in ranked:
        for c in r["constructs"] or ["(plain)"]:
            classes.setdefault(c, []).append(r)
    chosen: dict[str, dict] = {}
    for c in sorted(classes):
        for r in classes[c][:PER_CLASS]:
            chosen.setdefault(r["sha256"], r)
    for r in ranked:
        if len(chosen) >= TARGET:
            break
        chosen.setdefault(r["sha256"], r)
    return sorted(chosen.values(), key=lambda r: (-r["count"], r["sha256"]))


def reference_preamble() -> str:
    """The `default:` template's preamble block, verbatim."""
    text = TEMPLATES.read_text(encoding="utf-8")
    block = text[text.index("\ndefault:"):].split("preamble: |-\n", 1)[1]
    lines = []
    for line in block.splitlines():
        if line and not line.startswith("    "):
            break
        lines.append(line[4:])
    return "\n".join(lines).strip()


def measure_one(text: str, preamble: str) -> tuple[float, float, float] | None:
    # The Reference strips the joined Tex string; a leading newline would
    # otherwise put a blank line (\par) inside the math.
    text = text.strip()
    multiline = "\\\\" in text or "&" in text
    # A single line is braced as amsmath braces an align* cell
    # ($\m@th\displaystyle{##}$): a top-level \over then forms a display
    # fraction instead of swallowing \displaystyle into its numerator. As in
    # the Reference, a newline follows the text, so a trailing `\` is a
    # control space.
    # Multi-line strings are set as `aligned`, top-aligned ([t]) so the box
    # keeps its first row's baseline, as the Reference's align* rows do; a
    # centred `aligned` clips a short row's negative depth.
    body = (
        f"\\begin{{aligned}}[t]\n{text}\n\\end{{aligned}}" if multiline else f"{{{text}\n}}"
    )
    # amsmath's alignment rows carry an invisible strut (\strut@) that pads
    # the first row's height and the last row's depth. The Reference measures
    # ink, so that padding never reaches the screen; drop it. Row pitch
    # (\baselineskip + \jot) is unaffected. Array struts stay: they are an
    # array's visible row spacing. `aligned` also ends a one-row string's
    # last column with \minalignsep (10 pt) of glue that is no ink; zero it.
    # It is the space between column pairs too, and no sampled row has two
    # top-level pairs.
    unpad = (
        "\\makeatletter\\def\\strut@{}\\makeatother\\renewcommand{\\minalignsep}{0pt}\n"
        if multiline
        else ""
    )
    document = (
        "\\documentclass{article}\n" + preamble + "\n" + unpad
        + "\\newsavebox\\fmnbox\n\\begin{document}\n"
        "\\sbox\\fmnbox{$\\displaystyle\n" + body + "\n$}\n"
        "\\typeout{FMNBOX \\the\\wd\\fmnbox\\space\\the\\ht\\fmnbox\\space\\the\\dp\\fmnbox}\n"
        "\\end{document}\n"
    )
    with tempfile.TemporaryDirectory(prefix="tex-corpus-box-") as work:
        (pathlib.Path(work) / "f.tex").write_text(document, encoding="utf-8")
        try:
            run = subprocess.run(["latex", "-interaction=batchmode", "-halt-on-error", "f.tex"],
                                 cwd=work, capture_output=True, timeout=60, check=False)
        except subprocess.TimeoutExpired:
            return None
        log = (pathlib.Path(work) / "f.log").read_text(encoding="latin-1", errors="replace")
    match = BOX.search(log)
    if run.returncode != 0 or match is None:
        return None
    return tuple(float(v) / 10.0 for v in match.groups())  # pt -> em at 10 pt


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--corpus", type=pathlib.Path, required=True)
    parser.add_argument("--jobs", type=int, default=8)
    args = parser.parse_args()
    items = sample(args.corpus)
    preamble = reference_preamble()
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        boxes = list(pool.map(lambda item: measure_one(item["text"], preamble), items))
    rows, failed = [], 0
    for item, box in zip(items, boxes):
        if box is None or box[0] <= 0.0 or box[1] + box[2] <= 0.0:
            failed += 1
            continue
        # "|" appears in no construct name ("\\," and "\\ " rule out "," and " ").
        classes = "|".join(sorted(item["constructs"])) or "(plain)"
        width, height, depth = box
        rows.append(f"{item['sha256']}\t{width:.5f}\t{height:.5f}\t{depth:.5f}\t{item['count']}\t{classes}")
    header = [
        "# Real TeX boxes for a stratified corpus sample (fm-tex-layout-oracle-bkbc).",
        "# Strings are private fixtures (plan §15.3): each row names its corpus string by",
        "# sha256(mode + NUL + string). Width, height and depth are TeX's \\wd, \\ht and \\dp",
        "# of the stripped string boxed in display style, in ems at 10 pt: braced like an align*",
        "# cell, or for multi-line strings as `aligned`[t] without its invisible struts and glue.",
        "# Preamble: the pinned Reference's default template (3b1b/manim 6199a00d); TeX Live",
        "# 2025 on the dev host.",
        f"# Sample: up to {PER_CLASS} strings per construct class by",
        f"# occurrence, then the most frequent remaining strings, {len(items)} candidates;",
        f"# {failed} TeX could not typeset alone (fragments of multi-argument Tex calls) are",
        "# omitted. Regenerate only with scripts/capture_tex_corpus_boxes.py; never edit by hand.",
        "# sha256\twidth\theight\tdepth\tcount\tclasses (|-separated construct names)",
    ]
    FIXTURE.write_text("\n".join(header + rows) + "\n", encoding="utf-8")
    print(f"{len(rows)} rows, {failed} omitted -> {FIXTURE.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
