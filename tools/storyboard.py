"""Check a video's storyboard (gate G2) and write its script from it.

    python3 -I tools/storyboard.py [--check] [SLUG ...]   (default: every video with a storyboard)

The storyboard, explainer_corpus/<slug>/docs/storyboard.md, is the hand-edited
source of the video's words and pictures (from R1 on; plan §9.4, §10.4 and
§11.4 hold the revision-2 drafts it started from). Per chapter (a `## NN_key`
heading), one row per beat: `| id | words | picture | features | hold/listen |`.

Checks (exit 1 if any fails):
- every chapter's estimated length is at most 85 s (plan D-05);
- every beat has a picture (a card may stand in only if the features say so);
- every chapter has a prediction pause (⟨predict⟩), and a ⟨predict⟩ beat ends
  with its question (the answer is the next beat, so the picture can hold);
- ids are unique and sit under their own chapter.

It writes explainer_corpus/<slug>/script.tsv (id, spoken words) unless
--check, which instead fails when the file on disk differs from what the
storyboard says (a stale script). The Rust side parses the same format
(crates/explainer_kit/src/storyboard.rs); keep the two in step.

Length estimate per beat: the spoken words at 2.68 words a second (Hoeffding's
measured rate: 1,182 words took 440.9 s), plus a 0.6 s breath, plus 2 s for a
prediction pause, plus the hold (2.5 s) or listen window ("listen N s").
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CORPUS = ROOT / "explainer_corpus"
WPS, BREATH, PREDICT, HOLD = 2.68, 0.6, 2.0, 2.5
CAP = 85.0
MARK = "⟨predict⟩"

# the skill's showcase checklist, with the words that show a beat uses it
SHOWCASE = [
    ("TeX derivations (`write`, `TransformMatchingTex`: symbols → numbers → answer)", r"TransformMatchingTex|derivation|formula|equation|fraction"),
    ("value trackers driving live numbers and bars", r"live |count-up|counter|tracker|ticking"),
    ("morphs that keep identity (clouds, swarms, tables)", r"morph|swarm|reorganiz|untangl"),
    ("data windows (plots, histograms, number lines)", r"histogram|curve|plot|number line|axis|axes|bars?\b"),
    ("lagged group reveals", r"lagged|one by one|cascade|ripple"),
    ("highlights (`indicate`, surrounding boxes, flashes, glow)", r"indicate|box|flash|glow|highlight"),
    ("sound that teaches (listen windows, the score engine)", r"listen"),
    ("tables and grids built from primitives", r"table|grid|matrix|mosaic"),
    ("the camera route (3D, orbits)", r"\(camera\)|3D|orbit"),
]


def spoken(words):
    # an annotation runs from "*(" to the next ")*" (it may hold parentheses);
    # an unclosed one runs to the end; the same rule as storyboard.rs
    text = re.sub(r"\*\((?:.*?\)\*|.*$)", " ", words)
    text = text.replace(MARK, " ")
    return " ".join(text.split())


def hold_seconds(cell):
    m = re.search(r"listen\s+(\d+(?:\.\d+)?)\s*s", cell)
    if m:
        return float(m.group(1))
    return HOLD if re.search(r"\bhold\b", cell) else 0.0


def parse(path):
    chapters, cur, fence = [], None, False
    for n, line in enumerate(path.read_text().splitlines(), 1):
        if line.startswith("```"):
            fence = not fence
            continue
        if fence:
            continue
        h = re.match(r"## (\d\d_[^\s:(]+)[:\s]*(.*)", line)
        if h:
            cur = {"key": h.group(1), "title": h.group(2), "rows": []}
            chapters.append(cur)
            continue
        if re.match(r"\| \d\d_", line):
            if cur is None:
                raise SystemExit(f"{path}:{n}: a beat before any chapter heading")
            # cells split on every pipe not escaped as \| (an empty cell may be "| |")
            cells = re.split(r"(?<!\\)\|", line.strip())
            cols = [c.strip() for c in (cells[1:-1] if line.strip().endswith("|") else cells[1:])]
            cols += [""] * (5 - len(cols))
            cur["rows"].append({"n": n, "id": cols[0], "words": cols[1], "picture": cols[2],
                                "features": cols[3], "hold": cols[4]})
    if fence:
        raise SystemExit(f"{path}: an unclosed ``` block")
    return chapters


def check(slug, chapters):
    problems, seen, total = [], set(), 0.0
    showcase = {name: [] for name, _ in SHOWCASE}
    report = []
    for ch in chapters:
        est = 0.0
        for r in ch["rows"]:
            cid = r["id"]
            if cid in seen:
                problems.append(f"{slug} {cid}: id repeats")
            seen.add(cid)
            if not cid.startswith(ch["key"][:3]):
                problems.append(f"{slug} {cid}: sits under chapter {ch['key']}")
            words = spoken(r["words"])
            est += len(re.findall(r"[A-Za-z0-9][A-Za-z0-9'’,.\-]*", words)) / WPS + BREATH + hold_seconds(r["hold"])
            if MARK in r["words"]:
                est += PREDICT
                if not words.endswith("?"):
                    problems.append(f"{slug} {cid}: ⟨predict⟩ but the beat does not end with its question")
            if (not r["picture"] or r["picture"] in ("—", "-")) and not re.search(r"card", r["features"], re.I):
                problems.append(f"{slug} {cid}: no picture (G2)")
            for name, pat in SHOWCASE:
                if re.search(pat, r["picture"] + " " + r["features"] + " " + r["hold"], re.I):
                    showcase[name].append(cid)
        total += est
        if est > CAP:
            problems.append(f"{slug} {ch['key']}: estimated {est:.1f} s > {CAP:.0f} s")
        if not any(MARK in r["words"] for r in ch["rows"]):
            problems.append(f"{slug} {ch['key']}: no prediction pause")
        report.append(f"  {ch['key']:34} {est:5.1f} s  {len(ch['rows'])} beats")
    report.append(f"  total {total / 60:.1f} min over {len(chapters)} chapters")
    for name, _ in SHOWCASE:
        ids = showcase[name]
        report.append(f"  [{'x' if ids else ' '}] {name}" + (f": {', '.join(ids[:6])}{' …' if len(ids) > 6 else ''}" if ids else ""))
    return problems, report


def script_text(slug, chapters):
    lines = [f"# {slug}: generated from docs/storyboard.md by tools/storyboard.py; edit the storyboard, not this file"]
    for ch in chapters:
        for r in ch["rows"]:
            words = spoken(r["words"])
            if words:
                lines.append(f"{r['id']}\t{words}")
    return "\n".join(lines) + "\n"


def main(argv):
    check_only = "--check" in argv
    slugs = [a for a in argv if not a.startswith("--")] or sorted(
        p.parent.parent.name for p in CORPUS.glob("*/docs/storyboard.md"))
    failed = False
    for slug in slugs:
        board = CORPUS / slug / "docs/storyboard.md"
        chapters = parse(board)
        problems, report = check(slug, chapters)
        print(f"{slug}:")
        print("\n".join(report))
        script = CORPUS / slug / "script.tsv"
        text = script_text(slug, chapters)
        if check_only:
            if not script.is_file() or script.read_text() != text:
                problems.append(f"{slug}: script.tsv is stale (run tools/storyboard.py {slug})")
        else:
            script.write_text(text)
            print(f"  wrote {script.relative_to(ROOT)} ({text.count(chr(10)) - 1} lines)")
        for p in problems:
            print("  PROBLEM " + p)
        failed |= bool(problems)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
