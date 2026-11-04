#!/usr/bin/env python3
"""Validate the bound plan with the repository's coherent API in an isolated root.

    python3 -I -B validate_plan.py STAGED_TOPIC CANONICAL_TOPIC BUNDLE EMPTY_SCRATCH

Run from the repo root. Copies the script, unit plan and map into
EMPTY_SCRATCH/explainer_corpus/apportionment/, loads tools/coherent_narration.py
with its REPO pointed there, and runs read_units and direction_contract. It then
repeats explainer_kit::check_coherent_units in Python, checks every hold row against
the staged source, compares the map with oliver03 and the storyboard, and plants
defects to show the checks can fail.

Coverage only. It says nothing about takes, timing, listening or acceptance.
"""
import copy
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

TOPIC = "apportionment"
MAP = "narration-direction-map-oliver04.json"
PREV = "narration-direction-map-oliver03.json"
SAY = re.compile(r'\bsay\(&kit, stage, "(\d\d_\d+)"\)')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load_coherent(root):
    spec = importlib.util.spec_from_file_location("coherent_narration", "tools/coherent_narration.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.REPO = root.resolve()
    return module


def install(root, script, plan_text, map_text):
    topic = root / "explainer_corpus" / TOPIC
    (topic / "docs").mkdir(parents=True, exist_ok=True)
    (topic / "script.tsv").write_bytes(script)
    (topic / "narration_units.json").write_text(plan_text)
    (topic / "docs" / MAP).write_text(map_text)
    return topic


def contract_check(coherent, topic):
    """The repository's own read: raises on any broken binding."""
    script, units = coherent.read_units(topic / "script.tsv", topic / "narration_units.json", require_directions=True)
    plan = json.loads((topic / "narration_units.json").read_text())
    bound = coherent.direction_contract(topic / "script.tsv", plan)
    words = {c: coherent.direction_words(t) for c, t in script.items()}
    for r in bound["resolved"]:
        if r["end_word_index"] != len(words[r["cue"]]) - 1:
            raise ValueError(f"{r['cue']}: a direction does not end its cue")
    return script, units, bound


def kit_rules(script_bytes, plan, read_script, script_path):
    """explainer_kit::narration::check_coherent_units, transliterated, plus one cue per unit."""
    assert plan["schema_version"] == 1 and plan["script_sha256"] == sha(script_bytes), "plan does not bind the script"
    script = read_script(script_path)
    covered, ids = [], set()
    for unit in plan["units"]:
        assert unit["id"] and unit["id"] not in ids, unit["id"]
        ids.add(unit["id"])
        cues = unit["line_ids"]
        assert len(cues) == 1 and unit["id"] == f"oliver_{cues[0]}", f"{unit['id']} is not one cue"
        text = " ".join(script[c] for c in cues)
        assert unit["text"] == text and 1 <= len(text.split()) <= 100, f"{unit['id']} changes text or is too long"
        covered.extend(cues)
    assert covered == list(script), "units omit, repeat or reorder cues"
    return len(plan["units"])


def hold_problems(direction, chapters, cues):
    """Every way a hold row can fail to fit the given chapter files."""
    problems = []
    for r in direction["visual_holds"]:
        path = chapters / r["path"].rsplit("/", 1)[1]
        data = path.read_bytes()
        lines = data.decode().splitlines()
        tag = f"{r['cue']} {path.name}:{r['line']}"
        if sha(data) != r["sha256"]:
            problems.append(f"{tag}: hash differs")
        if not 1 <= r["line"] <= len(lines) or lines[r["line"] - 1].strip() != r["marker"]:
            problems.append(f"{tag}: marker is not on this line")
            continue
        before = [m.group(1) for line in lines[: r["line"] - 1] if (m := SAY.search(line))]
        after = [m.group(1) for line in lines[r["line"]:] if (m := SAY.search(line))]
        if not before or before[-1] != r["cue"] or not after or after[0] != r["next_cue"]:
            problems.append(f"{tag}: not between its two cues")
        i = cues.index(r["cue"])
        if i + 1 >= len(cues) or cues[i + 1] != r["next_cue"] or r["cue"][:2] != r["next_cue"][:2]:
            problems.append(f"{tag}: cues are not adjacent in one chapter")
        if "think" not in r["marker"] or r["kind"] != "prediction" or "range_s" in r or "word_index" in r:
            problems.append(f"{tag}: not a plain prediction row")
    return problems


def storyboard_rows(text):
    rows = {}
    for line in text.split("\n"):
        if re.match(r"\| \d\d_\d+ \|", line):
            cells = [c.strip() for c in re.split(r"(?<!\\)\|", line)]
            rows[cells[1]] = cells
    return rows


def main(argv):
    staged, canonical, bundle, scratch = (Path(a) for a in argv)
    if not scratch.is_dir() or any(scratch.iterdir()):
        raise SystemExit("scratch must be an existing empty directory")
    script = (staged / "script.tsv").read_bytes()
    plan_text = (bundle / "explainer_corpus" / TOPIC / "narration_units.json").read_text()
    map_text = (bundle / "explainer_corpus" / TOPIC / "docs" / MAP).read_text()
    plan, direction = json.loads(plan_text), json.loads(map_text)

    root = scratch / "isolated"
    topic = install(root, script, plan_text, map_text)
    coherent = load_coherent(root)
    parsed, units, bound = contract_check(coherent, topic)
    cues = list(parsed)
    print(f"coherent.read_units(require_directions=True): {len(units)} units over {len(cues)} cues")
    print(f"coherent.direction_contract: {len(bound['resolved'])} directions resolved, each ending its cue; {bound['claim']}")
    print(f"map path in contract: {plan['direction_contract']['map']['path']}")
    print(f"check_coherent_units rules, one cue per unit: {kit_rules(script, plan, coherent.read_script, topic / 'script.tsv')} units")
    words = {c: len(t.split()) for c, t in parsed.items()}
    print(f"words: total {sum(words.values())}, max {max(words.values())} ({[c for c, n in words.items() if n == max(words.values())]})")

    problems = hold_problems(direction, staged / "src" / "chapters", cues)
    assert not problems, problems
    print(f"{len(direction['visual_holds'])} hold rows fit the staged source (hash, marker at line, between adjacent cues)")
    kinds = {}
    for r in direction["visual_holds"]:
        kinds[r["kind"]] = kinds.get(r["kind"], 0) + 1
    assert kinds == {k: v for k, v in direction["hold_kinds"].items() if v}, kinds
    print(f"hold_kinds: {direction['hold_kinds']}")
    for h in direction["helpers"]:
        path = staged / "src" / "chapters" / "draw.rs"
        data = path.read_bytes()
        assert sha(data) == h["sha256"] and data.decode().splitlines()[h["line"] - 1].strip() == h["marker"], h
    print(f"{len(direction['helpers'])} helper rows fit draw.rs")

    prev = json.loads((canonical / "docs" / PREV).read_text())
    assert direction["overrides"] == prev["overrides"], "an authored direction changed"
    print(f"overrides: {len(direction['overrides'])}, identical to oliver03")
    changed = []
    assert [r["cue"] for r in prev["visual_holds"]] == [r["cue"] for r in direction["visual_holds"]]
    for a, b in zip(prev["visual_holds"], direction["visual_holds"]):
        assert a.keys() == b.keys(), a["cue"]
        delta = sorted(k for k in a if a[k] != b[k])
        if delta:
            changed.append(f"{a['cue']}: {', '.join(f'{k} {str(a[k])[:8]} -> {str(b[k])[:8]}' for k in delta)}")
    print(f"hold rows differing from oliver03: {changed}")

    against_installed = hold_problems(direction, canonical / "src" / "chapters", cues)
    print(f"against the installed canonical chapters, {len(against_installed)} problems, as expected until root installs:")
    for p in against_installed:
        print(f"  {p}")

    old = storyboard_rows((canonical / "docs" / "storyboard.md").read_text())
    new_text = (bundle / "explainer_corpus" / TOPIC / "docs" / "storyboard.md").read_text()
    new = storyboard_rows(new_text)
    assert list(old) == list(new) == cues, "storyboard rows are not the script's cues"
    for cue in cues:
        assert old[cue][2] == new[cue][2] and old[cue][4:] == new[cue][4:], cue
    differ = [c for c in cues if old[c][3] != new[c][3]]
    print(f"storyboard: words, features and hold cells identical in {len(cues)} rows; picture cell differs in {differ}")
    predict = [c for c in cues if "⟨predict⟩" in new[c][2]]
    listen = [c for c in cues if new[c][5] == "listen 4 s"]
    held = [r["cue"] for r in direction["visual_holds"]]
    assert predict == listen == held, (predict, listen, held)
    print(f"storyboard ⟨predict⟩ rows = listen 4 s rows = map rows: {len(held)}")

    # planted defects, each in its own copy: every one must be refused
    def planted(name, mutate_plan=None, mutate_map=None, raw_map=None):
        p, m = copy.deepcopy(plan), copy.deepcopy(direction)
        if mutate_map:
            mutate_map(m)
            text = json.dumps(m, indent=2, ensure_ascii=False) + "\n"
            p["direction_contract"]["map"]["sha256"] = sha(text.encode())
        else:
            text = raw_map or map_text
        if mutate_plan:
            mutate_plan(p)
        where = scratch / "planted" / name
        t = install(where, script, json.dumps(p, indent=2, ensure_ascii=False) + "\n", text)
        c = load_coherent(where)
        try:
            contract_check(c, t)
        except (ValueError, KeyError) as e:
            print(f"  refused {name}: {e}")
            return True
        print(f"  NOT REFUSED {name}")
        return False

    def edit_text(p):
        p["units"][3]["text"] += " extra"

    def drop_unit(p):
        del p["units"][5]

    def swap_units(p):
        p["units"][0], p["units"][1] = p["units"][1], p["units"][0]

    def foreign_phrase(m):
        m["overrides"][2]["after"] = "A phrase nobody says"

    def duplicate(m):
        m["overrides"].append(dict(m["overrides"][0]))

    def mid_cue(m):
        words_ = m["overrides"][0]["after"].split()
        m["overrides"][0]["after"] = " ".join(words_[:5])

    print("planted defects, coherent API:")
    results = [
        planted("unit-text-changed", mutate_plan=edit_text),
        planted("unit-dropped", mutate_plan=drop_unit),
        planted("units-reordered", mutate_plan=swap_units),
        planted("map-bytes-changed", raw_map=map_text.replace("15 authored", "16 authored", 1)),
        planted("phrase-not-in-cue", mutate_map=foreign_phrase),
        planted("direction-duplicated", mutate_map=duplicate),
        planted("direction-not-at-cue-end", mutate_map=mid_cue),
    ]
    print("planted defects, source binding:")
    for name, field, value in [("line-shifted", "line", 1), ("wrong-next-cue", "next_cue", "12_4")]:
        m = copy.deepcopy(direction)
        row = next(r for r in m["visual_holds"] if r["cue"] == "12_1")
        row[field] = row[field] + value if isinstance(value, int) else value
        found = hold_problems(m, staged / "src" / "chapters", cues)
        print(f"  {'refused' if found else 'NOT REFUSED'} {name}: {found[:1]}")
        results.append(bool(found))
    assert all(results), "a planted defect was not refused"
    print(f"{len(results)} planted defects refused")
    print("VALIDATION PASS: authored coverage only; no take, timing, listening or acceptance")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
