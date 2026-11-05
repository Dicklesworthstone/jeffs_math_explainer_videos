"""The repair's binding facts: hold rows, hook anchors, wait sites and the rows the binder would need.

    python3 -I -B bindings.py <baseline-chapters> <staged-chapters> <script.tsv> <stage06-hold-audit.json> <out-dir>

Writes hold-table.json, phrase-anchors-proposed.json and binder-rows.json in <out-dir>. The interval
formula is the binder's: lines[say_line - 1 : marker_line] joined by newlines, with the text split on
newlines. measured_seconds stays null: no mark is invented here. Standard library.
"""
import hashlib
import json
import pathlib
import re
import sys

BASE, STAGED, SCRIPT, AUDIT, OUT = (pathlib.Path(a) for a in sys.argv[1:6])
FILES = ("complex_turns.rs", "lifting_loops.rs", "branch_cut.rs", "product_rule.rs")
HOOK_FILES = ("complex_turns.rs", "lifting_loops.rs", "product_rule.rs")
script = dict(line.rstrip("\n").split("\t", 1) for line in SCRIPT.read_text(encoding="utf-8").splitlines() if "\t" in line and not line.startswith("#"))


def sha(data):
    return hashlib.sha256(data if isinstance(data, bytes) else data.encode("utf-8")).hexdigest()


def one_line(lines, needle):
    rows = [i for i, line in enumerate(lines, 1) if line.strip() == needle]
    assert len(rows) == 1, (needle, rows)
    return rows[0]


def scene_lines(text):
    return text.split("#[cfg(test)]\nmod tests")[0].split("\n")


audit = json.loads(AUDIT.read_text(encoding="utf-8"))
holds = []
for row in audit["prediction_holds"]:
    if row["file"] not in FILES:
        continue
    name, cue, marker = row["file"], row["cue"], row["marker"]
    base_text = (BASE / name).read_text(encoding="utf-8")
    new_text = (STAGED / name).read_text(encoding="utf-8")
    assert sha(base_text) == row["source_sha256"], name
    entry = {"cue": cue, "next_cue": row["next_cue"], "file": name, "marker": marker}
    for label, text in (("old", base_text), ("new", new_text)):
        lines = text.split("\n")
        scene = scene_lines(text)
        say = one_line(scene, f'kit.say(stage, "{cue}")?;')
        mark = one_line(scene, marker)
        nxt = one_line(scene, f'kit.say(stage, "{row["next_cue"]}")?;')
        entry[label] = {
            "sha256": sha(text),
            "say_line": say,
            "marker_line": mark,
            "next_say_line": nxt,
            "interval_sha256": sha("\n".join(lines[say - 1 : mark])),
        }
    assert entry["old"]["say_line"] == row["say_line"] and entry["old"]["marker_line"] == row["marker_line"], cue
    assert entry["old"]["interval_sha256"] == row["interval_sha256"], (cue, "the formula does not reproduce the audit")
    entry["lines_moved"] = entry["new"]["say_line"] - entry["old"]["say_line"]
    entry["interval_changed"] = entry["new"]["interval_sha256"] != entry["old"]["interval_sha256"]
    holds.append(entry)

finishes = {}
for name in FILES:
    a, b = scene_lines((BASE / name).read_text(encoding="utf-8")), scene_lines((STAGED / name).read_text(encoding="utf-8"))
    finishes[name] = {"old_line": one_line(a, "kit.finish(stage)"), "new_line": one_line(b, "kit.finish(stage)")}

# the binder asks an integration receipt to name each changed file's hash. Root writes that file when it
# integrates, and may name it differently; the row's value is the part that holds.
RECEIPT = "integration-teaching-repair01.json"
# what a later binder update counts per file: calls in the film's code, old and new
per_file = {}
for name in FILES:
    counts = {}
    for label, root in (("old", BASE), ("new", STAGED)):
        scene = "\n".join(scene_lines((root / name).read_text(encoding="utf-8")))
        counts[label] = {
            "say": len(re.findall(r'kit\.say\(stage, "', scene)),
            "until_sentence": len(re.findall(r"kit\.until_sentence\(stage, ", scene)),
            "until_phrase_calls": len(re.findall(r'until_phrase\(&kit, stage, "', scene)),
            "phrase_marks_uses": scene.count("phrase_marks("),
            "wait_frames_calls": len(re.findall(r"\bwait_frames\(", scene)),
            "lines": len((root / name).read_text(encoding="utf-8").split("\n")) - 1,
        }
    counts["sha256"] = {"old": sha((BASE / name).read_text(encoding="utf-8")), "new": sha((STAGED / name).read_text(encoding="utf-8"))}
    per_file[name] = counts

anchors, rows = [], {"phrase_mark_uses": {}, "fallback": {}, "sites": [], "receipts": {RECEIPT: [f"src/chapters/{name}" for name in FILES]}}
HOOK = re.compile(r'until_phrase\(&kit, stage, "(\d\d_\d+)", ([A-Z_]+)\)')
for name in HOOK_FILES:
    text = (STAGED / name).read_text(encoding="utf-8")
    scene = scene_lines(text)
    consts = dict(re.findall(r'const ([A-Z_]+): &str =\s*"([^"\\]*)";', "\n".join(scene)))
    for number, line in enumerate(scene, 1):
        found = HOOK.search(line)
        if not found:
            continue
        cue, const = found.groups()
        phrase = consts[const]
        raw = script[cue].encode("utf-8")
        byte = raw.find(phrase.encode("utf-8"))
        assert raw.count(phrase.encode("utf-8")) == 1 and raw[byte - 1] == 0x20, phrase
        opens = [0] + [i + 2 for i in range(len(raw) - 1) if raw[i] in b".?!" and raw[i + 1] == 0x20]
        anchors.append({
            "cue": cue,
            "byte": byte,
            "byte_end": byte + len(phrase.encode("utf-8")),
            "phrase": phrase,
            "const": const,
            "sentence_index": max(i for i, o in enumerate(opens) if o <= byte),
            "word_index": len(raw[:byte].decode("utf-8").split()),
            "hook_sites": [{"file": name, "line": number, "call": "until_phrase"}],
            "source_sha256": sha(text),
            "cue_text_sha256": sha(script[cue]),
            "fallback_without_mark": f"Without a mark at this byte: the phrase's character share of its measured sentence ({name} phrase_seconds). Provisional.",
            "measured_seconds": None,
        })
    assert sum(line.count("phrase_marks(") for line in scene) == 1
    wait = [i for i, line in enumerate(scene, 1) if re.search(r"\bwait_frames\(", line)]
    assert len(wait) == 1, (name, wait)
    rows["phrase_mark_uses"][name] = 1
    rows["fallback"][name] = f"Without a mark at this byte: the phrase's character share of its measured sentence ({name} phrase_seconds). Provisional."
    rows["sites"].append([name, wait[0], "word_sync_helper", "Inside until_phrase: waits to a phrase's time, inside the voice."])

rows["added_anchors"] = [[a["cue"], a["phrase"]] for a in anchors]
(OUT / "hold-table.json").write_text(json.dumps({"holds": holds, "finishes": finishes, "per_file": per_file}, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
(OUT / "phrase-anchors-proposed.json").write_text(json.dumps({"anchors": anchors, "note": "New hooks only. measured_seconds is null: no ASR mark was invented."}, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
(OUT / "binder-rows.json").write_text(json.dumps(rows, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

print("holds, say and marker lines old -> new, interval changed?")
for h in holds:
    print(f"  {h['cue']} {h['file']} {h['old']['say_line']}-{h['old']['marker_line']} -> {h['new']['say_line']}-{h['new']['marker_line']} next say {h['old']['next_say_line']} -> {h['new']['next_say_line']} changed {h['interval_changed']} {h['new']['interval_sha256'][:16]}")
print("finishes", finishes)
for name, counts in per_file.items():
    print(f"  {name}: " + ", ".join(f"{k} {counts['old'][k]} -> {counts['new'][k]}" for k in counts["old"]))
print("anchors")
for a in anchors:
    print(f"  {a['cue']} byte {a['byte']}-{a['byte_end']} sentence {a['sentence_index']} word {a['word_index']} {a['const']} {a['hook_sites'][0]['file']}:{a['hook_sites'][0]['line']} {a['phrase']!r}")
print("sites", rows["sites"])
print("receipts", rows["receipts"])
print("added anchors", len(rows["added_anchors"]))
assert len(holds) == 7 and not any(h["interval_changed"] for h in holds)
assert all(a["measured_seconds"] is None for a in anchors)
print("OK: 7 holds, no interval text changed")
