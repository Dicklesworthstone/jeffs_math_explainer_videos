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
order; each text exactly its lines joined; at most 100 words a unit. The
shared synthesis adapter may check the same contract; this copy goes when it
does.
"""
import hashlib
import json
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
SCRIPT = HERE / "script.tsv"
UNITS = HERE / "narration_units.json"
MAX_WORDS = 100


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


def build(groups, data):
    """The units file for `groups` (lists of line ids) and script bytes."""
    text = dict(parse_script(data))
    units, ordinal, last = [], 0, None
    for ids in groups:
        here = chapter(ids[0]) if ids else ""
        ordinal = ordinal + 1 if here == last else 1
        last = here
        units.append(
            {
                "id": f"{here}u{ordinal}",
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
    counts = {}
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
        here = chapter(ids[0])
        counts[here] = counts.get(here, 0) + 1
        if unit["id"] != f"{here}u{counts[here]}":
            errors.append(f"unit {n} is {unit['id']!r}, not {here}u{counts[here]}")
        joined = " ".join(text[i] for i in ids)
        if unit["text"] != joined:
            errors.append(f"{unit['id']}'s text is not its lines joined: {unit['text']!r}")
        words = len(unit["text"].split())
        if words > MAX_WORDS:
            errors.append(f"{unit['id']} has {words} words, over {MAX_WORDS}")
        said.extend(ids)
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
    if build(groups_of(doc), data) != doc:
        failed.append("--write would change the committed file")
    first, second = groups_of(doc)[0], groups_of(doc)[1]
    expect("a line left out", regrouped(lambda g: g[0].remove(first[1])), f"{first[1]} is in no unit")
    expect("a line said twice", regrouped(lambda g: g[1].insert(0, first[-1])), f"{first[-1]} is in 2 units")
    expect(
        "two lines swapped",
        regrouped(lambda g: g[0].__setitem__(slice(0, 2), [first[1], first[0]])),
        "do not follow the script's order",
    )
    cross = next(k for k, g in enumerate(groups_of(doc)) if chapter(g[0]) != chapter(first[0]))
    expect(
        "a unit across a chapter boundary",
        regrouped(lambda g: g[cross - 1].append(g[cross].pop(0))),
        "joins chapters",
    )
    expect("a unit of a whole chapter", regrouped(lambda g: g[0].extend(g.pop(1))), "over 100")
    paraphrased = json.loads(json.dumps(doc))
    paraphrased["units"][0]["text"] = paraphrased["units"][0]["text"].replace("Up, up", "Up up", 1)
    expect("a paraphrased text", paraphrased, "text is not its lines joined")
    expect("an edited script", doc, "the units are stale", script=data.replace(b"four pulses", b"4 pulses", 1))
    renamed = json.loads(json.dumps(doc))
    ch = chapter(second[0])
    renamed["units"][1]["id"] = f"{ch}u3"
    expect("a skipped unit number", renamed, f"is '{ch}u3', not {ch}u2")
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
        doc = build(groups_of(doc), data)
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
