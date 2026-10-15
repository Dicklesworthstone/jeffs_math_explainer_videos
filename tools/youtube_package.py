#!/usr/bin/env python3
"""tools/youtube_package.py: the upload text of a video, checked, and its chapter times filled from a final.

    python3 -I tools/youtube_package.py check SLUG [--finals DIR] [--voice michael]
    python3 -I tools/youtube_package.py fill SLUG --finals DIR [--voice michael]

A video's package is `explainer_corpus/SLUG/publish/youtube.md` (what goes on the upload page, with the
description in its one ```text block) and `publish/VOICE/{title,description,tags}.txt` (the text the upload
lane pastes). `check` exits 1 on anything YouTube would refuse or that would publish wrong:

- title.txt: one line, at most 100 characters, no angle brackets; youtube.md's "- Title:" line starts with it.
- tags.txt: one line of comma-separated tags, at most 500 characters as YouTube counts them (a tag with a
  space counts its two quotes); youtube.md's "- Tags:" line holds the same tags.
- description.txt: at most 5000 characters, no angle brackets (YouTube refuses them), byte for byte the
  ```text block of youtube.md.
- the chapter list (the lines after a line "Chapters" up to a blank line): `M:SS Title` or `MM:SS Title`, the first at 0:00,
  at least three, rising, each at least 10 s after the one before; a time still written `?:??` means the final
  has not been cut yet and fails. With `--finals DIR`, one chapter per `DIR/*.timeline.tsv` and the times
  equal the ones `./x SLUG chapters` prints for that final (a start rounded down to the second).
- docs/protected.tsv, when the video has one: every phrase it requires in the description or the title stands
  in description.txt or title.txt, matched as the skill's protected_check.py matches (case, punctuation and
  line breaks ignored, whole words). A cut of docs/description.md that drops a credit fails here.
- the numbers, when the video binds its notes' numbers to its facts (docs/description_numbers.tsv, checked by
  tools/description_numbers.py): every number of description.txt and title.txt (outside the chapter times
  and the link) stands in docs/description.md or the script, so the upload text says only numbers the
  notes do (thousands separators and signs folded). A number typed into the upload text alone fails here.

`fill` writes those times into the `?:??` (or stale) chapter lines of both files, in chapter order, and then
runs `check` against the same finals. Existing padded minute fields stay padded. Nothing is uploaded and nothing is deleted.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
TITLE_MAX = 100
TAGS_MAX = 500
DESCRIPTION_MAX = 5000
CHAPTER_MIN_S = 10
CHAPTER = re.compile(r"^(\?:\?\?|(\d+):(\d\d)) (\S.*)$")
TEXT_BLOCK = re.compile(r"^```text\n(.*?)^```$", re.S | re.M)


def paths(slug: str, voice: str, root: Path = ROOT) -> dict[str, Path]:
    crate = root / "explainer_corpus" / slug
    pub = crate / "publish"
    return {
        "youtube": pub / "youtube.md",
        "title": pub / voice / "title.txt",
        "description": pub / voice / "description.txt",
        "tags": pub / voice / "tags.txt",
    }


def tags_length(tags: list[str]) -> int:
    """YouTube's count: the tags, a comma between each, and two quotes round a tag with a space."""
    return sum(len(t) + (2 if " " in t else 0) for t in tags) + max(0, len(tags) - 1)


def final_starts(finals: Path) -> list[tuple[str, int]]:
    """(chapter key, start in whole seconds) of each chapter of a final, as `./x SLUG chapters` computes it."""
    out, t = [], 0.0
    for tl in sorted(finals.glob("*.timeline.tsv")):
        fps = frames = None
        for row in tl.read_text(encoding="utf-8").splitlines():
            cells = row.split("\t")
            if cells[0] == "fps" and len(cells) > 1:
                fps = float(cells[1])
            elif cells[0] == "frames" and len(cells) > 1:
                frames = float(cells[1])
        if not fps or frames is None:
            raise ValueError(f"{tl}: no fps or frames line")
        out.append((tl.name[: -len(".timeline.tsv")], int(t)))
        t += frames / fps
    return out


def clock(s: int) -> str:
    return f"{s // 60}:{s % 60:02d}"


def chapter_lines(text: str) -> list[tuple[int, re.Match]]:
    """(line index, match) of each chapter line: the lines after a line "Chapters" up to the next blank."""
    lines = text.split("\n")
    out = []
    for i, line in enumerate(lines):
        if line.strip() == "Chapters":
            for j in range(i + 1, len(lines)):
                if not lines[j].strip():
                    break
                out.append((j, CHAPTER.match(lines[j])))
            break
    return out


def words(text: str) -> str:
    """protected_check.py's form: lower case, every run of non-alphanumerics one space, padded for whole words."""
    return " " + re.sub(r"[^0-9a-z]+", " ", text.lower()).strip() + " "


def protected_missing(protected: Path, texts: dict[str, str], names: dict[str, str] | None = None) -> list[str]:
    """The rows of docs/protected.tsv whose <where> names a kind in `texts` (description, title, captions) and
    whose phrase that text lacks, one line each, each named by `names[kind]` (default `<kind>.txt`); no file,
    no rows."""
    if not protected.is_file():
        return []
    have = {k: words(t) for k, t in texts.items()}
    names = names or {}
    out = []
    for line in protected.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        cols = line.split("\t")
        where = [w.strip() for w in (cols[1] if len(cols) > 1 else "").split(",")]
        why = cols[2].strip() if len(cols) > 2 else ""
        for kind in where:
            if kind in have and words(cols[0]) not in have[kind]:
                name = names.get(kind, f"{kind}.txt")
                out.append(f"{name}: the protected phrase {cols[0].strip()!r} is missing (docs/protected.tsv: {why})")
    return out


NUMBER = re.compile(r"(?<![\w.])[-−]?\d+(?:,\d{3})*(?:\.\d+)?")


def numbers(text: str) -> list[tuple[str, int]]:
    """(number, offset) of each number in `text` outside links, separators and sign folded: '350,757' -> '350757'."""
    text = re.sub(r"https?://\S+", lambda m: " " * len(m.group(0)), text)
    return [(m.group(0).lstrip("-−").replace(",", ""), m.start()) for m in NUMBER.finditer(text)]


def unbacked_numbers(crate: Path, texts: dict[str, str]) -> list[str]:
    """The numbers of the upload text that docs/description.md and the script do not say, one line each; none
    when the video does not bind its notes' numbers (no docs/description_numbers.tsv)."""
    notes = crate / "docs" / "description.md"
    if not (crate / "docs" / "description_numbers.tsv").is_file() or not notes.is_file():
        return []
    said = {n for n, _ in numbers(notes.read_text(encoding="utf-8"))}
    script = crate / "script.tsv"
    if script.is_file():
        said |= {n for n, _ in numbers(script.read_text(encoding="utf-8"))}
    out = []
    for name, text in texts.items():
        chapter_rows = {i for i, _ in chapter_lines(text)}
        for i, line in enumerate(text.split("\n")):
            if i in chapter_rows:
                continue
            for n, at in numbers(line):
                if n not in said:
                    where = line[max(0, at - 30): at + 30].strip()
                    out.append(f"{name}: the number {n} ('...{where}...') stands nowhere in docs/description.md "
                               "or the script")
    return out


def check(slug: str, voice: str = "michael", finals: Path | None = None, root: Path = ROOT) -> list[str]:
    """Every problem with the package, one line each (empty when it is ready to upload)."""
    p = paths(slug, voice, root)
    bad = []
    for k, f in p.items():
        if not f.is_file():
            bad.append(f"{f.relative_to(root)}: missing")
    if bad:
        return bad
    yt = p["youtube"].read_text(encoding="utf-8")
    title = p["title"].read_text(encoding="utf-8").rstrip("\n")
    if "\n" in title or not title.strip():
        bad.append("title.txt: must be one non-empty line")
    if len(title) > TITLE_MAX:
        bad.append(f"title.txt: {len(title)} characters (YouTube takes {TITLE_MAX})")
    if "<" in title or ">" in title:
        bad.append("title.txt: angle brackets (YouTube refuses them)")
    m = re.search(r"^- Title: (.*)$", yt, re.M)
    if not m or not m.group(1).startswith(title):
        bad.append("youtube.md: its '- Title:' line does not start with title.txt")
    tags_raw = p["tags"].read_text(encoding="utf-8").rstrip("\n")
    tags = [t.strip() for t in tags_raw.split(",") if t.strip()]
    if "\n" in tags_raw:
        bad.append("tags.txt: must be one line")
    if tags_length(tags) > TAGS_MAX:
        bad.append(f"tags.txt: {tags_length(tags)} characters as YouTube counts them (it takes {TAGS_MAX})")
    m = re.search(r"^- Tags: (.*)$", yt, re.M)
    if not m or [t.strip() for t in m.group(1).split(",")] != tags:
        bad.append("youtube.md: its '- Tags:' line is not tags.txt")
    desc = p["description"].read_text(encoding="utf-8")
    if len(desc) > DESCRIPTION_MAX:
        bad.append(f"description.txt: {len(desc)} characters (YouTube takes {DESCRIPTION_MAX})")
    if "<" in desc or ">" in desc:
        bad.append("description.txt: angle brackets (YouTube refuses them)")
    blocks = TEXT_BLOCK.findall(yt)
    if len(blocks) != 1:
        bad.append(f"youtube.md: {len(blocks)} ```text blocks (the description is the one block)")
    elif blocks[0] != desc:
        bad.append("youtube.md: the ```text block is not description.txt byte for byte")
    protected = root / "explainer_corpus" / slug / "docs" / "protected.tsv"
    bad.extend(protected_missing(protected, {"description": desc, "title": title}))
    crate = root / "explainer_corpus" / slug
    bad.extend(unbacked_numbers(crate, {"description.txt": desc, "title.txt": title}))
    chapters = chapter_lines(desc)
    if len(chapters) < 3:
        bad.append(f"description.txt: {len(chapters)} chapter lines after 'Chapters' (YouTube needs 3)")
    prev = None
    for n, (i, mt) in enumerate(chapters):
        if mt is None:
            bad.append(f"description.txt line {i + 1}: not 'M:SS Title'")
            continue
        if mt.group(1) == "?:??":
            bad.append(f"description.txt line {i + 1}: time not filled (run fill against the final)")
            prev = None
            continue
        s = int(mt.group(2)) * 60 + int(mt.group(3))
        if n == 0 and s != 0:
            bad.append("description.txt: the first chapter must start at 0:00")
        if prev is not None and s - prev < CHAPTER_MIN_S:
            bad.append(f"description.txt line {i + 1}: {s - prev} s after the one before (YouTube needs {CHAPTER_MIN_S})")
        prev = s
    if finals is not None:
        starts = final_starts(finals)
        if len(starts) != len(chapters):
            bad.append(f"description.txt: {len(chapters)} chapters, the final has {len(starts)}")
        else:
            for (key, s), (i, mt) in zip(starts, chapters):
                if (mt is not None and mt.group(1) != "?:??"
                        and f"{int(mt.group(2))}:{mt.group(3)}" != clock(s)):
                    bad.append(f"description.txt line {i + 1}: {mt.group(1)}, the final's {key} starts at {clock(s)}")
    return bad


def fill_text(text: str, starts: list[tuple[str, int]]) -> str:
    chapters = chapter_lines(text)
    if len(chapters) != len(starts):
        raise ValueError(f"{len(chapters)} chapter lines, the final has {len(starts)} chapters")
    lines = text.split("\n")
    for (i, mt), (_, s) in zip(chapters, starts):
        if mt is None:
            raise ValueError(f"line {i + 1} is not a chapter line: {lines[i]!r}")
        timestamp = clock(s)
        if mt.group(2) is not None and len(mt.group(2)) >= 2:
            timestamp = f"{s // 60:02d}:{s % 60:02d}"
        lines[i] = f"{timestamp} {mt.group(4)}"
    return "\n".join(lines)


def fill(slug: str, finals: Path, voice: str = "michael", root: Path = ROOT) -> list[str]:
    p = paths(slug, voice, root)
    starts = final_starts(finals)
    desc = p["description"].read_text(encoding="utf-8")
    new = fill_text(desc, starts)
    yt = p["youtube"].read_text(encoding="utf-8")
    blocks = TEXT_BLOCK.findall(yt)
    if len(blocks) != 1 or blocks[0] != desc:
        return ["youtube.md: its ```text block is not description.txt; mend that before filling"]
    p["description"].write_text(new, encoding="utf-8")
    p["youtube"].write_text(yt.replace(desc, new, 1), encoding="utf-8")
    return check(slug, voice, finals, root)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("action", choices=["check", "fill"])
    ap.add_argument("slug")
    ap.add_argument("--finals", type=Path, help="a final's renders/final directory (its *.timeline.tsv)")
    ap.add_argument("--voice", default="michael")
    args = ap.parse_args(argv)
    if args.action == "fill" and args.finals is None:
        ap.error("fill needs --finals")
    if args.finals is not None and not list(args.finals.glob("*.timeline.tsv")):
        print(f"youtube_package: no *.timeline.tsv in {args.finals}", file=sys.stderr)
        return 2
    bad = check(args.slug, args.voice, args.finals) if args.action == "check" else fill(args.slug, args.finals, args.voice)
    for b in bad:
        print(f"FAIL {args.slug}: {b}")
    if not bad:
        print(f"PASS {args.slug}: the upload text is ready")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
