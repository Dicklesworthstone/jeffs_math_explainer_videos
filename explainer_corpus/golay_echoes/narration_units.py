#!/usr/bin/env python3
"""The narration units of "Two Bad Echoes Can Make One Perfect Measurement".

Run from this folder:

    python3 -I -B narration_units.py              check narration_units.json
    python3 -I -B narration_units.py --write      rebuild its texts and script hash
    python3 -I -B narration_units.py --self-test  the check refuses broken files

narration_units.json is what the central synthesis reads: each unit is one
coherent Michael take of whole script lines, cut back into the lines' cues by
the take's measured sentence timings. Its line ids are the hand-made part (one
complete thought per unit); `--write` fills in each unit's text, its lines
joined by one space, and the SHA-256 of script.tsv's bytes, so after a script
edit only the grouping needs a look.

The check (exit 1 on any failure, each one printed): schema_version 1; the
hash of the script.tsv on disk; every line of the script in exactly one unit,
in script order; no unit across two chapters; unit ids <chapter>u1, u2, ... in
order, a unit split into consecutive takes keeping its number with letters
a, b, ... (at least two parts: 01u1a, 01u1b); each text exactly its lines
joined; at most 100 words a unit. The
shared synthesis adapter may check the same contract; this copy goes when it
does.
"""
import hashlib
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
SCRIPT = HERE / "script.tsv"
UNITS = HERE / "narration_units.json"
MAX_WORDS = 100
UNIT_ID = re.compile(r"(\d\d)u([1-9][0-9]*)([a-z]?)")


def parse_script(data):
    """The script's lines, (id, text) in order, from script.tsv's bytes."""
    lines = []
    for raw in data.decode("utf-8").splitlines():
        if not raw.strip() or raw.startswith("#"):
            continue
        line_id, text = raw.split("\t", 1)
        lines.append((line_id, text))
    return lines


def chapter(line_id):
    return line_id.split("_", 1)[0]


def build(groups, data, unit_ids=None):
    """The units file for `groups` (lists of line ids) and script bytes, the
    units named `unit_ids` when given, else numbered in order."""
    text = dict(parse_script(data))
    units, ordinal, last = [], 0, None
    for k, ids in enumerate(groups):
        here = chapter(ids[0]) if ids else ""
        ordinal = ordinal + 1 if here == last else 1
        last = here
        units.append(
            {
                "id": unit_ids[k] if unit_ids else f"{here}u{ordinal}",
                "line_ids": list(ids),
                "text": " ".join(text.get(i, "") for i in ids),
            }
        )
    return {
        "schema_version": 1,
        "script_sha256": hashlib.sha256(data).hexdigest(),
        "units": units,
    }


def check(doc, data):
    """Every way `doc` fails to be the units file of the script `data`."""
    errors = []
    if not isinstance(doc, dict) or set(doc) != {"schema_version", "script_sha256", "units"}:
        return ["the file is not {schema_version, script_sha256, units}"]
    if doc["schema_version"] != 1 or isinstance(doc["schema_version"], bool):
        errors.append(f"schema_version is {doc['schema_version']!r}, not 1")
    if doc["script_sha256"] != hashlib.sha256(data).hexdigest():
        errors.append("script_sha256 is not the hash of script.tsv: the units are stale")
    lines = parse_script(data)
    text = dict(lines)
    units = doc["units"]
    if not isinstance(units, list) or not units:
        return errors + ["there are no units"]
    said = []
    # per chapter, the last unit's number and letter ("" when not split)
    named = {}

    def name_unit(n, unit_id, here):
        m = UNIT_ID.fullmatch(str(unit_id))
        last_n, last_letter = named.get(here, (0, ""))
        if last_letter == "a" and not (m and int(m.group(2)) == last_n and m.group(3) == "b"):
            errors.append(f"{here}u{last_n}a is split into one part")
        if not m or m.group(1) != here:
            errors.append(f"unit {n} is {unit_id!r}, not a unit id of chapter {here}")
            return
        number, letter = int(m.group(2)), m.group(3)
        if letter in ("", "a"):
            if number != last_n + 1:
                errors.append(f"unit {n} is {unit_id!r}, not {here}u{last_n + 1}{letter}")
        elif not last_letter or number != last_n or ord(letter) != ord(last_letter) + 1:
            wanted = f"{here}u{last_n}{chr(ord(last_letter) + 1)}" if last_letter else f"{here}u{last_n + 1}"
            errors.append(f"unit {n} is {unit_id!r}, not {wanted}")
        named[here] = (number, letter)

    for n, unit in enumerate(units):
        if not isinstance(unit, dict) or set(unit) != {"id", "line_ids", "text"}:
            errors.append(f"unit {n} is not {{id, line_ids, text}}")
            continue
        ids = unit["line_ids"]
        if not isinstance(ids, list) or not ids:
            errors.append(f"{unit['id']} has no lines")
            continue
        unknown = [i for i in ids if i not in text]
        if unknown:
            errors.append(f"{unit['id']} has lines not in the script: {', '.join(map(str, unknown))}")
            continue
        chapters = sorted({chapter(i) for i in ids})
        if len(chapters) > 1:
            errors.append(f"{unit['id']} joins chapters {' and '.join(chapters)}")
        name_unit(n, unit["id"], chapter(ids[0]))
        joined = " ".join(text[i] for i in ids)
        if unit["text"] != joined:
            errors.append(f"{unit['id']}'s text is not its lines joined: {unit['text']!r}")
        words = len(unit["text"].split())
        if words > MAX_WORDS:
            errors.append(f"{unit['id']} has {words} words, over {MAX_WORDS}")
        said.extend(ids)
    errors.extend(f"{here}u{n}a is split into one part" for here, (n, letter) in named.items() if letter == "a")
    order = [i for i, _ in lines]
    for i in order:
        times = said.count(i)
        if times == 0:
            errors.append(f"{i} is in no unit")
        elif times > 1:
            errors.append(f"{i} is in {times} units")
    if not errors and said != order:
        errors.append("the units do not follow the script's order")
    return errors


def groups_of(doc):
    return [unit["line_ids"] for unit in doc["units"]]


def ids_of(doc):
    return [unit["id"] for unit in doc["units"]]


def self_test(doc, data):
    """Each broken file is refused for its own reason; the real one passes."""
    failed = []

    def expect(name, broken, reason, script=data):
        found = check(broken, script)
        if not any(reason in e for e in found):
            failed.append(f"{name}: expected {reason!r}, got {found}")

    def regrouped(change):
        groups = [list(g) for g in groups_of(doc)]
        change(groups)
        return build(groups, data)

    if check(doc, data):
        failed.append(f"the committed file fails: {check(doc, data)}")
    if build(groups_of(doc), data, ids_of(doc)) != doc:
        failed.append("--write would change the committed file")
    # the first unit of two lines or more, and the unit after it
    f = next(k for k, g in enumerate(groups_of(doc)) if len(g) >= 2)
    first, second = groups_of(doc)[f], groups_of(doc)[f + 1]
    expect("a line left out", regrouped(lambda g: g[f].remove(first[1])), f"{first[1]} is in no unit")
    expect("a line said twice", regrouped(lambda g: g[f + 1].insert(0, first[-1])), f"{first[-1]} is in 2 units")
    expect(
        "two lines swapped",
        regrouped(lambda g: g[f].__setitem__(slice(0, 2), [first[1], first[0]])),
        "do not follow the script's order",
    )
    cross = next(k for k, g in enumerate(groups_of(doc)) if chapter(g[0]) != chapter(groups_of(doc)[0][0]))

    def whole_chapter(g):
        g[: cross] = [[i for group in g[: cross] for i in group]]

    expect(
        "a unit across a chapter boundary",
        regrouped(lambda g: g[cross - 1].append(g[cross].pop(0))),
        "joins chapters",
    )
    expect("a unit of a whole chapter", regrouped(whole_chapter), "over 100")
    paraphrased = json.loads(json.dumps(doc))
    paraphrased["units"][0]["text"] = paraphrased["units"][0]["text"].replace("Up, up", "Up up", 1)
    expect("a paraphrased text", paraphrased, "text is not its lines joined")
    expect("an edited script", doc, "the units are stale", script=data.replace(b"four pulses", b"4 pulses", 1))
    # unit names, on the file regrouped into numbered units (01u1, 01u2, ...)
    plain = build(groups_of(doc), data)
    ch = chapter(plain["units"][0]["line_ids"][0])
    renamed = json.loads(json.dumps(plain))
    renamed["units"][1]["id"] = f"{ch}u3"
    expect("a skipped unit number", renamed, f"is '{ch}u3', not {ch}u2")
    split = json.loads(json.dumps(plain))
    split["units"][0]["id"], split["units"][1]["id"] = f"{ch}u1a", f"{ch}u1b"
    for later in split["units"][2:]:
        if chapter(later["line_ids"][0]) == ch:
            later["id"] = f"{ch}u{int(UNIT_ID.fullmatch(later['id']).group(2)) - 1}"
    if check(split, data):
        failed.append(f"a unit split into {ch}u1a and {ch}u1b is refused: {check(split, data)}")
    skipped = json.loads(json.dumps(split))
    skipped["units"][1]["id"] = f"{ch}u1c"
    expect("a split part's letter skipped", skipped, f"is '{ch}u1c', not {ch}u1b")
    lone = json.loads(json.dumps(split))
    lone["units"][1]["id"] = f"{ch}u2"
    expect("a split into one part", lone, f"{ch}u1a is split into one part")
    backwards = json.loads(json.dumps(split))
    backwards["units"][0]["id"], backwards["units"][1]["id"] = f"{ch}u1b", f"{ch}u1a"
    expect("split parts out of order", backwards, f"is '{ch}u1b', not {ch}u1")
    extra = json.loads(json.dumps(doc))
    extra["units"][0]["voice"] = "michael"
    expect("a field the schema does not have", extra, "is not {id, line_ids, text}")
    future = json.loads(json.dumps(doc))
    future["schema_version"] = 2
    expect("another schema", future, "schema_version is 2")
    unknown = json.loads(json.dumps(doc))
    unknown["units"][-1]["line_ids"].append("99_1")
    expect("a line the script lacks", unknown, "not in the script: 99_1")

    # the word limit at its edge, on a two-line script of one chapter
    def script_of(words):
        return ("01_1\t" + "a " * (words - 1) + "end.\n01_2\tLast.\n").encode()

    at_limit = script_of(MAX_WORDS - 1)
    if check(build([["01_1", "01_2"]], at_limit), at_limit):
        failed.append(f"a unit of exactly {MAX_WORDS} words is refused")
    over = script_of(MAX_WORDS)
    expect(f"a unit of {MAX_WORDS + 1} words", build([["01_1", "01_2"]], over), "over 100", script=over)
    return failed


def main(argv):
    data = SCRIPT.read_bytes()
    doc = json.loads(UNITS.read_text(encoding="utf-8"))
    if "--write" in argv:
        doc = build(groups_of(doc), data, ids_of(doc))
        UNITS.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    if "--self-test" in argv:
        failed = self_test(doc, data)
        for f in failed:
            print(f"FAIL {f}", file=sys.stderr)
        print(f"self-test: {'FAIL' if failed else 'PASS'}", file=sys.stderr)
        return 1 if failed else 0
    errors = check(doc, data)
    for e in errors:
        print(f"FAIL {e}", file=sys.stderr)
    for unit in doc.get("units", []):
        words = len(str(unit.get("text", "")).split())
        print(f"{unit.get('id')}\t{words} words\t{' '.join(map(str, unit.get('line_ids', [])))}")
    print(f"narration units: {'FAIL' if errors else 'PASS'}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
