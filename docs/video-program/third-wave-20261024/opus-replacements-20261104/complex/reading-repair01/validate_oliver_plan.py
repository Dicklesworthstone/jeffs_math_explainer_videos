#!/usr/bin/env python3
"""Validate the Oliver plan with the repository's own code, touching nothing canonical.

    python3 -I -B validate_oliver_plan.py REPO CRATE PLAN_DIR EMPTY_SCRATCH [--map-version oliver08]

Copies the crate's script.tsv and the plan's units and map into
<scratch>/explainer_corpus/complex_analysis_visual/, loads tools/coherent_narration.py
from REPO with its REPO pointed at the scratch, and runs its read_units with
require_directions and its direction_contract. Then repeats, independently of the
binder and with plain line scans:

  explainer_kit::narration::check_coherent_units (schema, hash, one chapter per unit,
    exact text, 1 to 100 words, every cue once, in order);
  narration_delivery external_plan (exactly one cue per unit);
  narration_delivery prediction_end (each prediction pair declared once, next cue in the
    same chapter, so the hold can be found);
  coherent_narration check_dispositions for native_visual_hold (the override ends its cue);
  narration_director validate_map's adjacency of (cue, next_cue);
  each hold's file hash and marker, and the say before and after its marker line;
  a reading row exists exactly where READING_WINDOWS and a reading_window call do, with the
    card up before it, the next cue straight after it, and the card cleared by that cue.

Only chapter files registered in mod.rs are scanned. Coverage only: it says nothing about
takes, timing or listening, and a reading row's seconds are a proposal, not a measurement.
"""

import argparse
import collections
import glob
import hashlib
import importlib.util
import json
import os
import re
import shutil
import sys
from pathlib import Path

TOPIC = "complex_analysis_visual"
TOPIC_REL = f"explainer_corpus/{TOPIC}"
SAY = re.compile(r'\bsay\(&kit, stage, "(\d\d_\d+)"\)')
WINDOW_MARKER = "reading_window(&kit, stage, &line)?;"


def require(condition, text):
    if not condition:
        raise SystemExit("REFUSED: " + text)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def code_lines(path):
    """Physical lines before the file's test module, with line comments dropped."""
    lines = Path(path).read_text(encoding="utf8").split("\n")
    out = []
    for i, line in enumerate(lines):
        if line.startswith("#[cfg(test)]") and i + 1 < len(lines) and lines[i + 1].startswith("mod "):
            break
        out.append(re.sub(r"^\s*//.*$", "", line))
    return out


def table_rows(raw, table):
    """The cues of `pub const <table>`, in order, from the file's text."""
    block = re.search(rf"^pub const {table}\b.*?^\];", raw, re.M | re.S)
    require(block, f"mod.rs has no {table}")
    return re.findall(r'^\s*\("(\d\d_\d+)",\s*[\d.]+\),\s*$', block.group(0), re.M)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("repo")
    parser.add_argument("crate")
    parser.add_argument("plan")
    parser.add_argument("scratch")
    parser.add_argument("--map-version", default="oliver08")
    args = parser.parse_args()
    repo, crate, plan_dir, scratch = (Path(p).resolve() for p in (args.repo, args.crate, args.plan, args.scratch))
    map_rel = f"docs/narration-direction-map-{args.map_version}.json"
    topic = scratch / TOPIC_REL
    require(not topic.exists(), f"{topic} exists; give an empty scratch dir")
    (topic / "docs").mkdir(parents=True)
    shutil.copy2(crate / "script.tsv", topic / "script.tsv")
    shutil.copy2(plan_dir / TOPIC_REL / "narration_units.json", topic / "narration_units.json")
    shutil.copy2(plan_dir / TOPIC_REL / map_rel, topic / map_rel)

    spec = importlib.util.spec_from_file_location("coherent", repo / "tools" / "coherent_narration.py")
    coherent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(coherent)
    coherent.REPO = scratch

    plan = json.loads((topic / "narration_units.json").read_text(encoding="utf8"))
    direction = json.loads((topic / map_rel).read_text(encoding="utf8"))

    # the repository's own code, on the plan as it would be integrated
    try:
        script, units = coherent.read_units(topic / "script.tsv", topic / "narration_units.json", require_directions=True)
        bound = coherent.direction_contract(topic / "script.tsv", plan)
    except (ValueError, OSError, KeyError, TypeError) as error:
        raise SystemExit(f"REFUSED: the repository's contract: {error}")
    print(f"coherent.read_units(require_directions=True): {len(units)} units, {len(script)} cues")
    print(f"coherent.direction_contract: {len(bound['resolved'])} directions resolved; {bound['claim']}")

    # check_coherent_units, transliterated from crates/explainer_kit/src/narration.rs
    source = (topic / "script.tsv").read_bytes()
    require(plan["schema_version"] == 1, "schema")
    require(plan["script_sha256"] == hashlib.sha256(source).hexdigest(), "plan does not bind the script")
    covered, ids = [], set()
    for unit in plan["units"]:
        require(unit["id"] and unit["id"] not in ids, f"repeated unit {unit['id']}")
        ids.add(unit["id"])
        cues = unit["line_ids"]
        require(cues and len({c.split("_", 1)[0] for c in cues}) == 1, f"{unit['id']} crosses chapters")
        text = " ".join(script[c] for c in cues)
        require(unit["text"] == text, f"{unit['id']} changes the text")
        require(1 <= len(text.split()) <= 100, f"{unit['id']} is outside 1 to 100 words")
        # narration_delivery external_plan, and this plan's naming
        require(len(cues) == 1, f"{unit['id']} is not exactly one cue")
        require(unit["id"] == f"oliver_{cues[0]}", f"{unit['id']} is misnamed")
        covered.extend(cues)
    cues = list(script)
    require(covered == cues, "units omit, repeat or reorder cues")
    print(f"check_coherent_units rules: {len(plan['units'])} units cover {len(covered)} cues in order, one cue each")

    # the map binds this script and asks for no timing
    contract = plan["direction_contract"]
    require(contract["topic"] == TOPIC, "contract names another topic")
    require(contract["map"]["path"] == f"{TOPIC_REL}/{map_rel}", "contract names another map")
    require(direction["scripts"] == {TOPIC: plan["script_sha256"]}, "map binds another script")
    for item in direction["overrides"]:
        require("range_s" not in item and "word_index" not in item, f"override on {item['cue']} asks for timing")
    for item in direction["visual_holds"]:
        require("range_s" not in item and "word_index" not in item, f"hold on {item['cue']} invents a timing")

    # every say in the source, with a plain scan the binder does not share
    chapters = crate / "src" / "chapters"
    said = collections.Counter()
    mod_text = (chapters / "mod.rs").read_text(encoding="utf8")
    registered = {"mod.rs"} | {f"{m}.rs" for m in re.findall(r"^(?:pub )?mod (\w+);", mod_text, re.M)}
    for path in sorted(glob.glob(str(chapters / "*.rs"))):
        if os.path.basename(path) not in registered:
            continue
        for line in code_lines(path):
            if re.search(r"(?<![\w.])say\(", line) and "pub fn say" not in line:
                require(SAY.search(line), f"{os.path.basename(path)}: non-literal say: {line.strip()}")
                said[SAY.search(line).group(1)] += 1
    require(sum(said.values()) == len(cues), f"{sum(said.values())} say calls against {len(cues)} cues")
    require(set(said) == set(cues) and set(said.values()) == {1}, "a cue is not said exactly once")
    print(f"source: {sum(said.values())} literal say calls, each cue once")

    # overrides and holds pair up, and each override ends its cue
    resolved = {r["cue"]: r for r in bound["resolved"]}
    holds = direction["visual_holds"]
    require(len(resolved) == len(bound["resolved"]), "two directions on one cue")
    require(list(resolved) == [h["cue"] for h in holds], "overrides and holds differ")
    for cue, row in resolved.items():
        require(row["end_word_index"] == len(coherent.direction_words(script[cue])) - 1, f"{cue}: override does not end its cue")
    print(f"overrides: {len(resolved)}, each ends its cue, so native_visual_hold can dispose of it")

    # the holds' cue pairs, and their evidence in the source
    pairs = set(zip(cues, cues[1:]))
    declared = collections.Counter((h["cue"], h["next_cue"]) for h in holds)
    predict_rows = table_rows(mod_text, "PREDICTS")
    reading_rows = table_rows(mod_text, "READING_WINDOWS")
    require([h["cue"] for h in holds if h["kind"] == "prediction"] == predict_rows, "holds differ from PREDICTS")
    require([h["cue"] for h in holds if h["kind"] == "reading"] == reading_rows, "reading holds differ from READING_WINDOWS")
    require(not set(predict_rows) & set(reading_rows), "a cue is both a prediction and a reading window")
    require({h["kind"] for h in holds} <= {"prediction", "reading"}, "a hold of an unknown kind")
    calls = []
    for path in sorted(glob.glob(str(chapters / "*.rs"))):
        if os.path.basename(path) not in registered:
            continue
        lines = code_lines(path)
        for number, line in enumerate(lines):
            if line.strip() == WINDOW_MARKER:
                before = [SAY.search(x).group(1) for x in lines[:number] if SAY.search(x)]
                calls.append(before[-1] if before else None)
            elif "reading_window(" in line and "pub fn reading_window" not in line:
                require(False, f"{os.path.basename(path)}: a reading_window call that is not the marker: {line.strip()}")
    require(sorted(calls) == sorted(reading_rows), f"reading_window calls {sorted(calls)} differ from READING_WINDOWS")
    draw_lines = [x.strip() for x in code_lines(chapters / "draw.rs")]
    head = next((i for i, x in enumerate(draw_lines) if x.startswith("pub fn reading_window(")), None)
    require(head is not None, "draw.rs has no reading_window")
    tail = draw_lines[head : draw_lines.index("}", head)]
    voice, breath, wait = (tail.index(x) if x in tail else -1 for x in ("wait_until(stage, line.end())?;", "kit.hold(stage)?;", "pause(stage, seconds)"))
    require(0 <= voice < breath < wait and wait == len(tail) - 1, "draw::reading_window's order changed")
    require(not re.search(r"fade|remove_from_scene|clear_updaters|\.play\(", "\n".join(tail)), "draw::reading_window clears or draws")
    for hold in holds:
        cue, following = hold["cue"], hold["next_cue"]
        require((cue, following) in pairs, f"{cue}: next cue {following} is not adjacent")
        require(cue[:2] == following[:2], f"{cue}: next cue {following} is in another chapter")
        require(declared[(cue, following)] == 1, f"{cue}: declared {declared[(cue, following)]} times")
        if hold["kind"] == "prediction":
            require(script[cue].rstrip().endswith("?"), f"{cue} does not end in a question")
            require("predict(&kit, stage, &line," in hold["marker"], f"{cue}: marker is no predict call")
        else:
            require(hold["marker"] == WINDOW_MARKER, f"{cue}: marker is no reading_window call")
        require(hold["path"].startswith(f"{TOPIC_REL}/src/chapters/"), f"{cue}: path does not mirror the canonical tree")
        path = chapters / Path(hold["path"]).name
        require(digest(path) == hold["sha256"], f"{path.name} changed")
        lines = code_lines(path)
        require(lines[hold["line"] - 1].strip() == hold["marker"], f"{path.name}:{hold['line']} marker moved")
        before = [SAY.search(x).group(1) for x in lines[: hold["line"] - 1] if SAY.search(x)]
        after = [SAY.search(x).group(1) for x in lines[hold["line"] :] if SAY.search(x)]
        require(before and before[-1] == cue, f"{path.name}:{hold['line']} is not owned by {cue}")
        require(after and after[0] == following, f"{path.name}:{hold['line']} is not followed by {following}")
        if hold["kind"] == "reading":
            says = [i for i, x in enumerate(lines) if SAY.search(x)]
            here = max(i for i in says if i < hold["line"] - 1)
            then = min(i for i in says if i > hold["line"] - 1)
            rest = [x.strip() for x in lines[hold["line"] : then] if x.strip()]
            require(not rest, f"{path.name}:{hold['line']} something waits between the window and {following}: {rest[:1]}")
            cards = [i for i in range(here, hold["line"] - 1) if re.search(r"let \w+ = precisely_now\(", lines[i])]
            require(len(cards) == 1, f"{cue}: {len(cards)} precisely cards before its window")
            card = re.search(r"let (\w+) = precisely_now\(", lines[cards[0]]).group(1)
            kept = "\n".join(lines[cards[0] : hold["line"] - 1])
            require(not re.search(r"fade_out|remove_from_scene", kept), f"{cue}: the {card} card can go before its window")
            later = [i for i in says if i > then]
            cleared = "\n".join(lines[then : later[0] if later else len(lines)])
            require("fade_out_all" in cleared and re.search(rf"\b{card}\b", cleared), f"{cue}: {following} does not clear {card}")
    for helper in direction["helpers"]:
        path = chapters / Path(helper["path"]).name
        require(digest(path) == helper["sha256"], f"{path.name} changed")
        require(path.read_text(encoding="utf8").split("\n")[helper["line"] - 1].strip() == helper["marker"], f"{path.name}: marker moved")
    kinds = direction["hold_kinds"]
    require(kinds["prediction"] == len(predict_rows) and kinds["reading"] == len(reading_rows), "hold_kinds disagrees with the holds")
    print(f"visual holds: {kinds['prediction']} predictions, {kinds['reading']} reading; adjacent, same chapter, hashes and markers current")
    print(f"reading rows: exactly the {len(reading_rows)} cues with a reading_window call and a READING_WINDOWS row; no range_s on any row")
    print("Coverage only. No take, timing, listening or acceptance.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
