"""Validate the candidate's Oliver plan with the repository's own code,
without touching explainer_corpus/apportionment.

    python3 -I -B validate_oliver_plan.py <repo root> <candidate topic dir> <empty scratch dir>

Copies the candidate's script.tsv, narration_units.json and direction map into
<scratch>/explainer_corpus/apportionment/, loads tools/coherent_narration.py
from the repo with its REPO pointed at <scratch>, and runs its
direction_contract on the plan. Then repeats explainer_kit's
check_coherent_units rules (narration.rs) in Python. Coverage only: this says
nothing about takes, timing or listening.
"""

import hashlib
import importlib.util
import json
import shutil
import sys
from pathlib import Path

repo, cand, scratch = (Path(a).resolve() for a in sys.argv[1:4])
topic = scratch / "explainer_corpus" / "apportionment"
if topic.exists():
    raise SystemExit(f"{topic} exists; give an empty scratch dir")
(topic / "docs").mkdir(parents=True)
for rel in ("script.tsv", "narration_units.json", "docs/narration-direction-map-oliver03.json"):
    shutil.copy2(cand / rel, topic / rel)

spec = importlib.util.spec_from_file_location("coherent", repo / "tools" / "coherent_narration.py")
coherent = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coherent)
coherent.REPO = scratch

plan = json.loads((topic / "narration_units.json").read_text())
bound = coherent.direction_contract(topic / "script.tsv", plan)
print(f"direction_contract: {len(bound['resolved'])} directions resolved; {bound['claim']}")
ends = {(r["cue"], r["end_word_index"]) for r in bound["resolved"]}

# explainer_kit::narration::check_coherent_units, transliterated
script_source = (topic / "script.tsv").read_text()
assert plan["schema_version"] == 1, "schema"
assert plan["script_sha256"] == hashlib.sha256(script_source.encode()).hexdigest(), "plan does not bind the script"
script = coherent.read_script(topic / "script.tsv")
covered, ids = [], set()
for unit in plan["units"]:
    assert unit["id"] and unit["id"] not in ids, unit["id"]
    ids.add(unit["id"])
    cues = unit["line_ids"]
    assert cues and len({c.split("_", 1)[0] for c in cues}) == 1, f"{unit['id']} crosses chapters"
    text = " ".join(script[c] for c in cues)
    assert unit["text"] == text and 1 <= len(text.split()) <= 100, f"{unit['id']} changes text or is too long"
    covered.extend(cues)
assert covered == list(script), "units omit, repeat or reorder cues"
print(f"check_coherent_units rules: {len(plan['units'])} units cover {len(covered)} cues in order")

# each prediction override ends its cue, and each visual hold's marker sits
# at its recorded line in the candidate chapter file with the recorded hash
words = {c: coherent.direction_words(t) for c, t in script.items()}
assert all(i == len(words[c]) - 1 for c, i in ends), "a prediction override does not end its cue"
direction_map = json.loads((topic / "docs" / "narration-direction-map-oliver03.json").read_text())
for hold in direction_map["visual_holds"]:
    src = cand / "src" / "chapters" / Path(hold["path"]).name
    lines = src.read_text().splitlines()
    assert hashlib.sha256(src.read_bytes()).hexdigest() == hold["sha256"], f"{src.name} changed"
    assert lines[hold["line"] - 1].strip() == hold["marker"], f"{src.name}:{hold['line']} marker moved"
print(f"visual holds: {len(direction_map['visual_holds'])} markers at their lines, hashes current")
