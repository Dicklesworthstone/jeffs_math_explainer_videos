#!/usr/bin/env python3
"""Narration units for "A Resistor Network Is Secretly Answering a Probability Question".

The central synthesis reads narration_units.json (beside this file): the
lines of script.tsv grouped into units, each unit spoken as one coherent take
of the narration voice and then cut back into its lines, which the chapters
place one by one (`Kit::say`).

    python3 -I narration_units.py           write narration_units.json from script.tsv
    python3 -I narration_units.py --check   exit 1 unless the file on disk is valid and current

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

GROUPS is where the units are chosen: one complete thought each, broken where
the picture changes its subject, and a prediction question always ends its
unit, so the take ends on the question and the countdown follows it. The
long form's eleven chapters have 31 units. Chapter 01 has three: the line,
the walker's rule and its route (01u1); what a chance is, one step ahead,
the halfway rule and the prediction question (01u2); the answer and the
chance of walking for ever (01u3). Chapter 07 ends on two short units, its
question (07u3) and its answer (07u4). The checker in the p-values crate's
test file (test_narration_units.py) exercises these same rules; this copy
keeps only the grouping and the check.

The four-chapter review cut of 2026-10-20 was voiced from this file's
earlier groups (twelve units over 34 lines); those historical takes belong
to that script and its contexts. The long form's units here have not been
synthesized, and no earlier take is to be reused in a new unit's context on
the strength of the same words. The owner rejected the first-wave narration
on 2026-10-19 (AGENTS.md), and that method is not to be carried into new
synthesis; the grouping is a script structure for whatever qualified route
the central lane adopts.
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
    # 01: the line, the walker's rule, and one route
    ("01_1", "01_2", "01_3"),
    # what its chance is, one step ahead, halfway, and the prediction
    # question, ending its take
    ("01_4", "01_5", "01_6", "01_7"),
    # the answer, equal steps, and the chance of walking for ever
    ("01_8", "01_9"),
    # 02: the battery, one current, equal drops, and the question
    ("02_1", "02_2", "02_3", "02_4"),
    # the answer, the agreement, and why: the walker's rule
    ("02_5", "02_6", "02_7", "02_8"),
    # 03: the bridge network, its rule, and the left junction's check
    ("03_1", "03_2", "03_3"),
    # the school shortcuts, and the question
    ("03_4", "03_5", "03_6"),
    # none applies: the bridge
    ("03_7", "03_8"),
    # 04: the walk on the same wires, its rule, and the question
    ("04_1", "04_2", "04_3"),
    # the answer, why it must agree, and what the walker is not
    ("04_4", "04_5"),
    # 05: the left junction's drops, with the previewed numbers
    ("05_1", "05_2", "05_3"),
    # both voltages unknown, the left's balance, and the right's question
    ("05_4", "05_5", "05_6"),
    # the right's balance
    ("05_7",),
    # 06: the walker's first step: the prediction question, ending its take
    ("06_1",),
    # the answer, and the left's first-step equation
    ("06_2", "06_3"),
    # the right's, times four, the circuit's two balances
    ("06_4", "06_5", "06_6"),
    # 07: the solve
    ("07_1", "07_2", "07_3"),
    # the walker by its routes, and why that worked here
    ("07_4", "07_5", "07_6", "07_7"),
    # the bottom first: the prediction question, ending its take
    ("07_8",),
    # its answer
    ("07_9",),
    # 08: the guess, the rule for a round, and the first rounds
    ("08_1", "08_2", "08_3", "08_4"),
    # the gap's quarter, and the prediction question, ending its take
    ("08_5", "08_6"),
    # never exactly, the limit, and why the ends stay fixed
    ("08_7", "08_8", "08_9"),
    # 09: the supposed second solution, the difference, and its peak
    ("09_1", "09_2", "09_3"),
    # the peak spreads to the top, so one solution; the island's question,
    # ending its take
    ("09_4", "09_5", "09_6"),
    # the island's peak stops short, the hypothesis, and the walker there
    ("09_7", "09_8", "09_9"),
    # 10: the weighted rule, unequal wires, and the question
    ("10_1", "10_2", "10_3"),
    # the answer, the two equations, and their solution
    ("10_4", "10_5"),
    # 11: the method, the two runs, and when they meet
    ("11_1", "11_2", "11_3"),
    # beside the island: the prediction question, ending its take
    ("11_4",),
    # the answer, and the theorem with its hypotheses
    ("11_5", "11_6"),
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
