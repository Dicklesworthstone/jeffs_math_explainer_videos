#!/usr/bin/env python3
"""Narration units for "Why P-Values Confuse Even Doctors" (p_values_medical).

The central synthesis reads narration_units.json (beside this file): the
lines of script.tsv grouped into units, each unit spoken as one coherent take
of the narration voice and then cut back into its lines, which the chapters
place one by one (`Kit::say`).

    python3 -I narration_units.py           write narration_units.json from script.tsv
    python3 -I narration_units.py --check   exit 1 unless the file on disk is valid and current
    python3 -I test_narration_units.py      the rules below, each shown to fail when broken

Schema 1:
    {"schema_version": 1,
     "script_sha256": the SHA-256 of script.tsv's bytes,
     "units": [{"id": "<chapter>u<ordinal>", "line_ids": [line ids in order],
                "text": the lines' exact text joined by one space}]}

The rules (`problems` checks each one):
  * every line of script.tsv is in exactly one unit, and the units read the
    lines in script order: none left out, repeated or moved;
  * a unit holds the lines of one chapter, and its id names that chapter and
    its place in it (01u1, 01u2, ...); a unit split into separate requests
    keeps its place and letters its parts in order (04u1a, 04u1b, ...), two or
    more of them;
  * a unit's text is its lines exactly, joined by one space, no stage
    direction or paraphrase; it ends a sentence; it has at most 100 words
    (split on whitespace, so "seventy-seven" is one);
  * script_sha256 is the hash of script.tsv as it is on disk.

GROUPS is where the units are chosen: complete thoughts of 55 to 90 words
where the script allows, broken where the picture changes its subject. Four
units are shorter because their thought is: the survey before chapter 01's
title card (01u1), the positives gathered after the count (01u3), the
imagined 14 between the wrong readings and the made-up days (05u2), and the
survey's question before its answer (06u1).

A list of groups in GROUPS is one unit split into separate requests. 04u1
and 05u2 were split where a take ran two lines together with no silence
between them, so no cut there could keep both words whole: "first. | For
nineteen" and "Fourteen. | Fourteen or more" (root, Mail 3384). 04u1 was
split again after its question, "fifteen? | Fifteen itself", where the take
left only shallow quiet before the answer (Mail 3420). So 04u1a ends on the
question and 05u2a is one line. 02u1 is split at its prediction hold, after
02_3's question: with its counts said in fuller steps (the measured redo,
Mail 4297) it is 103 words, over the cap, and its parts, 51 and 52 words,
are each about the length of the owner-accepted opening, 01u1's 51. 05u1
is split at its prediction hold for the reason 04u1a was: its take left
0.10 s between "did nothing?" and "No.", where the scene then held five
seconds, and 0.09 s at "true. | Nor" (the v4 cut's measured boundaries).
"""
import hashlib
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = HERE / "script.tsv"
UNITS = HERE / "narration_units.json"

SCHEMA_VERSION = 1
MAX_WORDS = 100
UNIT_ID = re.compile(r"(\d\d)u([1-9]\d*)([a-z]?)")
KEYS = {"schema_version", "script_sha256", "units"}
UNIT_KEYS = {"id", "line_ids", "text"}

GROUPS = [
    # 01: the survey, before the title card
    ("01_1", "01_2"),
    # the positive result, the tempting ninety-five percent, the count
    ("01_4", "01_5", "01_6", "01_7", "01_8"),
    # the positives gathered, and the group each percent is a share of
    ("01_9", "01_10", "01_11"),
    # 02: the same test in clinic B and its question | the count that
    # answers it (split at the prediction hold)
    [("02_1", "02_2", "02_3"), ("02_4", "02_5", "02_6")],
    # what it means, why, the two rates apart, the study's imaginary world
    ("02_7", "02_8", "02_9", "02_10"),
    # 03: the pairs, the rule decided in advance, the sharp null, the fifteen
    ("03_1", "03_2", "03_3", "03_4"),
    # the coins tossed again, every assignment, the question asked in that world
    ("03_5", "03_6", "03_7", "03_8", "03_9"),
    # 04: the study's fifteen and which bars count | fifteen and up, and the
    # one way to get twenty | the other counts up to 21,700 (split after the
    # question, and where the take ran "first" into "For")
    [("04_1", "04_2"), ("04_3", "04_4"), ("04_5", "04_6")],
    # the share and its name, the wrong tails, the world it lives in
    ("04_7", "04_8", "04_9", "04_10"),
    # 05: two percent, and the question asked of it | what two percent is not
    # (split at the prediction hold)
    [("05_1", "05_2"), ("05_3", "05_4")],
    # an imagined fourteen | its tail and the five percent convention (split
    # where the take ran "Fourteen." into "Fourteen or more")
    [("05_5",), ("05_6", "05_7")],
    # the two made-up versions of the recovery days
    ("05_8", "05_9", "05_10"),
    # 06: the survey's keyed answer, and the question for our study
    ("06_1", "06_2", "06_3"),
    # the answer, its name, what it leaves open, what to ask
    ("06_4", "06_5", "06_6", "06_7"),
]


def parse_script(text):
    """The (id, text) lines of script.tsv, by the kit's rules (explainer_kit
    `Script::parse`): `#` and blank lines skipped, one tab between id and
    text, both trimmed and non-empty, no id twice."""
    lines, seen = [], set()
    for n, raw in enumerate(text.split("\n"), 1):
        line = raw.removesuffix("\r")
        if not line.strip() or line.startswith("#"):
            continue
        if "\t" not in line:
            raise ValueError(f"script.tsv line {n}: no tab between id and text")
        line_id, rest = line.split("\t", 1)
        if "\t" in rest:
            raise ValueError(f"script.tsv line {n}: a stray tab inside the text")
        line_id, rest = line_id.strip(), rest.strip()
        if not line_id or not rest:
            raise ValueError(f"script.tsv line {n}: empty id or text")
        if line_id in seen:
            raise ValueError(f"script.tsv line {n}: id {line_id!r} repeats")
        seen.add(line_id)
        lines.append((line_id, rest))
    return lines


def chapter_of(line_id):
    """A line's chapter: the two digits before its underscore ("01" of "01_10")."""
    return line_id.split("_", 1)[0]


def words(text):
    return len(text.split())


def problems(doc, lines, script_bytes):
    """Every way `doc` breaks the rules for the script `lines` (whose file
    has the bytes `script_bytes`); empty when it is valid."""
    if not isinstance(doc, dict) or set(doc) != KEYS:
        return [f"the document must be an object with exactly the keys {sorted(KEYS)}"]
    out = []
    version = doc["schema_version"]
    if type(version) is not int or version != SCHEMA_VERSION:
        out.append(f"schema_version is {version!r}, not {SCHEMA_VERSION}")
    want = hashlib.sha256(script_bytes).hexdigest()
    if doc["script_sha256"] != want:
        out.append(f"script_sha256 {doc['script_sha256']!r} is not script.tsv's {want}")
    units = doc["units"]
    if not isinstance(units, list) or not units:
        return [*out, "units must be a non-empty list"]
    text_of = dict(lines)
    said, place, parts = [], {}, {}
    for k, unit in enumerate(units, 1):
        if not isinstance(unit, dict) or set(unit) != UNIT_KEYS:
            out.append(f"unit {k}: must be an object with exactly the keys {sorted(UNIT_KEYS)}")
            continue
        uid, ids, text = unit["id"], unit["line_ids"], unit["text"]
        name = f"unit {k} ({uid!r})"
        if not isinstance(ids, list) or not ids or not all(isinstance(i, str) for i in ids):
            out.append(f"{name}: line_ids must be a non-empty list of line ids")
            continue
        said.extend(ids)
        unknown = [i for i in ids if i not in text_of]
        if unknown:
            out.append(f"{name}: {unknown} are not lines of script.tsv")
        chapters = sorted({chapter_of(i) for i in ids})
        if len(chapters) > 1:
            out.append(f"{name}: holds lines of chapters {chapters}; a unit stays in one chapter")
        else:
            c = chapters[0]
            n, letter = place.get(c, (0, ""))
            allowed = [f"{c}u{n + 1}", f"{c}u{n + 1}a"] + ([f"{c}u{n}{chr(ord(letter) + 1)}"] if letter else [])
            m = UNIT_ID.fullmatch(uid) if isinstance(uid, str) else None
            if m is None or uid not in allowed:
                out.append(f"{name}: the id must be {allowed[0]!r}, its chapter and its place in it "
                           f"(or {', '.join(map(repr, allowed[1:]))} for a split unit's part)")
                place[c] = (n + 1, "")
            else:
                place[c] = (int(m.group(2)), m.group(3))
                if m.group(3):
                    parts.setdefault(uid[:-1], []).append(m.group(3))
        if not isinstance(text, str):
            out.append(f"{name}: text must be a string")
            continue
        if not unknown:
            exact = " ".join(text_of[i] for i in ids)
            if text != exact:
                at = next((j for j, (a, b) in enumerate(zip(text, exact)) if a != b), min(len(text), len(exact)))
                out.append(f"{name}: text differs from its lines joined by one space at character {at}: "
                           f"{text[at:at + 30]!r} where the script has {exact[at:at + 30]!r}")
        if not text.rstrip().endswith((".", "?", "!")):
            out.append(f"{name}: text must end a sentence")
        if words(text) > MAX_WORDS:
            out.append(f"{name}: {words(text)} words, more than {MAX_WORDS}")
    for base, letters in parts.items():
        if len(letters) < 2:
            out.append(f"split unit {base} has only part {letters[0]!r}: a split unit has two parts or more")
    order = [i for i, _ in lines]
    missing = [i for i in order if i not in said]
    if missing:
        out.append(f"lines in no unit: {missing}")
    repeated = sorted({i for i in said if said.count(i) > 1})
    if repeated:
        out.append(f"lines in more than one place: {repeated}")
    known = [i for i in said if i in text_of]
    if not missing and not repeated and known != order:
        j = next(j for j, (a, b) in enumerate(zip(known, order)) if a != b)
        out.append(f"the units leave script order at {known[j]!r}, where the script has {order[j]!r}")
    return out


def build(lines, script_bytes, groups=GROUPS):
    """The document for `lines` grouped by `groups`; a ValueError if the
    result breaks a rule (a line the groups leave out, say)."""
    text_of = dict(lines)
    units, per_chapter = [], {}
    for group in groups:
        split = isinstance(group, list)
        members = group if split else [group]
        chapter = chapter_of(members[0][0])
        per_chapter[chapter] = per_chapter.get(chapter, 0) + 1
        for j, ids in enumerate(members):
            units.append({
                "id": f"{chapter}u{per_chapter[chapter]}" + (chr(ord("a") + j) if split else ""),
                "line_ids": list(ids),
                "text": " ".join(text_of.get(i, "") for i in ids),
            })
    doc = {
        "schema_version": SCHEMA_VERSION,
        "script_sha256": hashlib.sha256(script_bytes).hexdigest(),
        "units": units,
    }
    found = problems(doc, lines, script_bytes)
    if found:
        raise ValueError("; ".join(found))
    return doc


def dump(doc):
    return json.dumps(doc, indent=2, ensure_ascii=False) + "\n"


def main(argv):
    script_bytes = SCRIPT.read_bytes()
    lines = parse_script(script_bytes.decode("utf-8"))
    if argv == ["--check"]:
        try:
            doc = json.loads(UNITS.read_text(encoding="utf-8"))
        except (OSError, ValueError) as e:
            print(f"{UNITS.name}: {e}", file=sys.stderr)
            return 1
        found = problems(doc, lines, script_bytes)
        if not found and dump(doc) != dump(build(lines, script_bytes)):
            found = [f"{UNITS.name} is not what GROUPS and script.tsv give: rerun python3 -I narration_units.py"]
        for p in found:
            print(f"FAIL {p}", file=sys.stderr)
        if found:
            return 1
    elif argv:
        print(__doc__.split("\n\n")[2], file=sys.stderr)
        return 2
    else:
        doc = build(lines, script_bytes)
        UNITS.write_text(dump(doc), encoding="utf-8")
    for unit in doc["units"]:
        print(f"{unit['id']}\t{words(unit['text'])} words\t{' '.join(unit['line_ids'])}")
    print(f"{len(doc['units'])} units, {len(lines)} lines, script {doc['script_sha256'][:12]}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
