#!/usr/bin/env python3
"""Validate the final 80-cue plan with the repository's coherent API, on a staged root.

    python3 -I -B <this file> EMPTY_SCRATCH_DIR

Run from the repo root. Reads the bundle, stage04, the kit and the tools. Writes only inside
EMPTY_SCRATCH_DIR, which must exist and be empty: the rehearsal root and the planted defects.

The hold rows, anchors and template are rechecked from raw bytes, without the binder's helpers.
The contract is the actual tools/coherent_narration.py with its REPO pointed at a staged root.
tools/narration_delivery.py's prediction_end runs on synthetic numbers, so it shows only how
the map's declarations behave. Each planted defect has to be refused, to show the checks can fail.

Coverage only. No take, measured timing, picture, listening or acceptance. Nothing compiled.
"""
import copy
import hashlib
import importlib.util
import json
import re
import shutil
import sys
from collections import Counter
from pathlib import Path

BUNDLE_ABS = Path(__file__).resolve().parent.parent
BUNDLE = BUNDLE_ABS.relative_to(Path.cwd())
TOPIC = "logarithm_mysteries"
REL = f"explainer_corpus/{TOPIC}"
MAP = "narration-direction-map-oliver12.json"
LANE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
STAGE = LANE / "combined-repair-stage04" / REL
CH = STAGE / "src/chapters"
REFERENCE = LANE / "narration-binding-stage01/candidate" / REL / "docs/narration-direction-map-expanded01.json"
OBSOLETE = {("00_1", "00_2"), ("01_1", "01_2"), ("03_1", "03_2"), ("04_1", "04_2")}
PREDICTIONS = {"00_4", "01_3", "02_2", "04_6", "05_1", "05_5", "06_2", "06_4", "07_3", "08_1", "08_4", "11_3", "12_5"}
READINGS = {"10_4", "11_20"}
SAY = re.compile(r'\bkit\.say\(stage, "(\d\d_\d+)"\)')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def coherent_at(root):
    module = load("coherent_narration", "tools/coherent_narration.py")
    module.REPO = Path(root).resolve()
    return module


def install(root, script, plan_text, map_text):
    topic = Path(root) / REL
    (topic / "docs").mkdir(parents=True, exist_ok=True)
    (topic / "script.tsv").write_bytes(script)
    (topic / "narration_units.json").write_text(plan_text, encoding="utf-8")
    (topic / "docs" / MAP).write_text(map_text, encoding="utf-8")
    return topic


def contract_check(coherent, script_path, plan_path):
    """The repository's own read, and the rule assembly applies to a native_visual_hold."""
    script, units = coherent.read_units(script_path, plan_path, require_directions=True)
    plan = json.loads(Path(plan_path).read_text(encoding="utf-8"))
    bound = coherent.direction_contract(script_path, plan)
    words = {c: coherent.direction_words(t) for c, t in script.items()}
    for r in bound["resolved"]:
        if r["end_word_index"] != len(words[r["cue"]]) - 1:
            raise ValueError(f"{r['cue']}: a direction does not end its cue")
    return script, units, bound


def kit_rules(script_bytes, plan, parsed):
    """explainer_kit::narration::check_coherent_units, transliterated, plus one cue per unit."""
    assert plan["schema_version"] == 1 and plan["script_sha256"] == sha(script_bytes), "plan does not bind the script"
    covered, ids = [], set()
    for unit in plan["units"]:
        assert unit["id"] and unit["id"] not in ids, unit["id"]
        ids.add(unit["id"])
        cues = unit["line_ids"]
        assert len(cues) == 1 and unit["id"] == f"oliver_{cues[0]}", f"{unit['id']} is not one cue"
        text = " ".join(parsed[c] for c in cues)
        assert unit["text"] == text and 1 <= len(text.split()) <= 100, f"{unit['id']} changes text or is too long"
        covered.extend(cues)
    assert covered == list(parsed), "units omit, repeat or reorder cues"
    return len(plan["units"])


def hold_problems(direction, cues):
    """Every way a hold row can fail to fit stage04's bytes, read fresh."""
    problems = []
    if {(r["cue"], r["next_cue"]) for r in direction["visual_holds"]} & OBSOLETE:
        problems.append("an obsolete pair is back")
    if {r["cue"] for r in direction["visual_holds"] if r["kind"] == "prediction"} != PREDICTIONS:
        problems.append("the prediction rows are not the 13 actual calls")
    if {r["cue"] for r in direction["visual_holds"] if r["kind"] == "reading"} != READINGS:
        problems.append("the reading rows are not 10_4 and 11_20")
    if [o["cue"] for o in direction["overrides"]] != [r["cue"] for r in direction["visual_holds"]]:
        problems.append("overrides and hold rows differ")
    for r in direction["visual_holds"]:
        path = CH / r["path"].rsplit("/", 1)[1]
        data = path.read_bytes()
        lines = data.decode("utf-8").split("\n")
        tag = f"{r['cue']} {path.name}:{r['line']}"
        if sha(data) != r["sha256"] or r["evidence"]["sha256"] != r["sha256"] or r["evidence"]["path"] != r["path"]:
            problems.append(f"{tag}: hash differs")
        if not 1 <= r["line"] <= len(lines) or lines[r["line"] - 1].strip() != r["marker"]:
            problems.append(f"{tag}: marker is not on this line")
            continue
        before = [(n, m.group(1)) for n, line in enumerate(lines[: r["line"] - 1], 1) if (m := SAY.search(line))]
        after = [m.group(1) for line in lines[r["line"] :] if (m := SAY.search(line))]
        if not before or before[-1][1] != r["cue"]:
            problems.append(f"{tag}: not after its cue")
            continue
        i = cues.index(r["cue"])
        if i + 1 >= len(cues) or cues[i + 1] != r["next_cue"]:
            problems.append(f"{tag}: cues are not adjacent")
        crosses = r["cue"][:2] != r["next_cue"][:2]
        if crosses != (r["cue"] == "10_4") or (not crosses and (not after or after[0] != r["next_cue"])):
            problems.append(f"{tag}: not before its next cue")
        if crosses and after:
            problems.append(f"{tag}: a say after the chapter's last hold")
        interval = "\n".join(lines[before[-1][0] - 1 : r["line"]])
        if sha(interval.encode("utf-8")) != r["interval_sha256"]:
            problems.append(f"{tag}: interval changed")
        want = "predict(" if r["kind"] == "prediction" else "kit.music_breath(stage,"
        if not r["marker"].startswith(want):
            problems.append(f"{tag}: the marker is not this kind's call")
    return problems


def anchor_problems(doc, template, script):
    """Every anchor from raw script and chapter bytes; no seconds anywhere."""
    problems = []
    for r in doc["anchors"]:
        raw, wanted = script[r["cue"]].encode("utf-8"), r["phrase"].encode("utf-8")
        tag = f"{r['cue']}@{r['byte']}"
        if raw[r["byte"] : r["byte_end"]] != wanted or raw.count(wanted) != 1 or raw.find(wanted) != r["byte"]:
            problems.append(f"{tag}: not where text.find lands")
        if r["byte"] and raw[r["byte"] - 1] != 0x20:
            problems.append(f"{tag}: not at a word start")
        if r["measured_seconds"] is not None:
            problems.append(f"{tag}: a seconds value")
        for site in r["hook_sites"]:
            data = (CH / site["file"]).read_bytes()
            if sha(data) != r["source_sha256"]:
                problems.append(f"{tag}: source hash differs")
            line = data.decode("utf-8").split("\n")[site["line"] - 1]
            token = r["const"] or r["phrase"]
            if site["call"] in ("until_phrase", "phrase_time") and not (f'"{r["cue"]}"' in line and token in line):
                problems.append(f"{tag}: {site['file']}:{site['line']} is not this hook")
    expected = {}
    for r in doc["anchors"]:
        expected.setdefault(r["cue"], []).append(r["byte"])
    if list(template["cues"]) != list(script):
        problems.append("template cues are not the script's")
    for cue, entry in template["cues"].items():
        if entry["text_sha256"] != sha(script[cue].encode("utf-8")) or entry["wav_sha256"] is not None:
            problems.append(f"{cue}: template binding")
        if [m["byte"] for m in entry["marks"]] != expected.get(cue, []) or any(m["seconds"] is not None for m in entry["marks"]):
            problems.append(f"{cue}: template marks")
        bytes_ = [m["byte"] for m in entry["marks"]]
        if bytes_ != sorted(set(bytes_)):
            problems.append(f"{cue}: template marks are not strictly increasing")
    return problems


def main(argv):
    scratch = Path(argv[0])
    if not scratch.is_dir() or any(scratch.iterdir()):
        raise SystemExit("scratch must be an existing empty directory")
    script_path = STAGE / "script.tsv"
    script_bytes = script_path.read_bytes()
    plan_path = BUNDLE / REL / "narration_units.json"
    map_path = BUNDLE / REL / "docs" / MAP
    plan_text, map_text = plan_path.read_text(encoding="utf-8"), map_path.read_text(encoding="utf-8")
    plan, direction = json.loads(plan_text), json.loads(map_text)
    audit = json.loads((BUNDLE / "hold-audit.json").read_text(encoding="utf-8"))
    anchors = json.loads((BUNDLE / "phrase-anchors.json").read_text(encoding="utf-8"))
    template = json.loads((BUNDLE / "phrase-timings-unmeasured-template.json").read_text(encoding="utf-8"))
    proposed = json.loads((BUNDLE / "direction-dispositions-proposed.json").read_text(encoding="utf-8"))
    delta = json.loads((BUNDLE / "cue-delta.json").read_text(encoding="utf-8"))

    # ---------------------------------------------------------------- reproducible
    binder = load("bind_final80", BUNDLE / "bind_final80.py")
    built = binder.build()[0]
    for rel, text in built.items():
        assert (BUNDLE / rel).read_text(encoding="utf-8") == text, f"{rel} differs from a fresh build"
    print(f"binder: a fresh build equals all {len(built)} generated files on disk, byte for byte")

    # ---------------------------------------------------------------- the contract, on staged roots
    coherent = coherent_at(BUNDLE)
    parsed, units, bound = contract_check(coherent, script_path, plan_path)
    cues = list(parsed)
    assert len(units) == 80 and len(cues) == 80 and len(bound["resolved"]) == 15
    print(f"staged root = this bundle, script = stage04: coherent.read_units(require_directions=True) -> {len(units)} units over {len(cues)} cues")
    print(f"coherent.direction_contract -> {len(bound['resolved'])} directions resolved, each ending its cue; claim: {bound['claim']}")
    rehearsal = install(scratch / "rehearsal", script_bytes, plan_text, map_text)
    again = coherent_at(scratch / "rehearsal")
    _, _, bound2 = contract_check(again, rehearsal / "script.tsv", rehearsal / "narration_units.json")
    assert [(r["cue"], r["end_word_index"]) for r in bound2["resolved"]] == [(r["cue"], r["end_word_index"]) for r in bound["resolved"]]
    print("rehearsal root with the canonical layout (script, plan and map copied): same 15 resolutions")
    print(f"map path in contract: {plan['direction_contract']['map']['path']}, sha256 {plan['direction_contract']['map']['sha256']}")
    print(f"check_coherent_units rules, one cue per unit: {kit_rules(script_bytes, plan, parsed)} units, in script order")
    assert all(len(u["line_ids"]) == 1 for u in units)
    print("tools/narration_delivery.py external_plan's rule holds: one cue per unit, so an ElevenLabs UI plan can carry this contract unchanged")
    words = {c: len(t.split()) for c, t in parsed.items()}
    assert sum(words.values()) == 4592 and max(words.values()) == 95 and sha(script_bytes) == plan["script_sha256"]
    print(f"words: total {sum(words.values())}, max {max(words.values())} ({[c for c, n in words.items() if n == 95]}); script sha256 {sha(script_bytes)}")
    try:
        coherent.read_units(script_path, STAGE / "narration_units.json")
        raise AssertionError("stage04's stale plan was accepted")
    except ValueError as error:
        print(f"stage04's copied 14-unit plan, against the final script: refused ({error})")

    # ---------------------------------------------------------------- holds
    problems = hold_problems(direction, cues)
    assert not problems, problems
    print(f"{len(direction['visual_holds'])} hold rows fit stage04 (file hash, marker at line, between adjacent cues, interval hash): {direction['hold_kinds']}")
    pairs = {(r["cue"], r["next_cue"]) for r in direction["visual_holds"]}
    assert not pairs & OBSOLETE
    print(f"the four obsolete pairs are absent: {sorted(a + '->' + b for a, b in OBSOLETE)}")
    for row in audit["removed_pairs"]:
        assert not row["old_after_is_final_cue_text"] and not row["old_after_in_final_cue"] and not row["windows_in_final_interval"]
        assert not row["old_file_registered"] or not row["old_marker_still_at_old_line"], row["cue"]
    print("each names a retired file, or a line that no longer holds that marker; its old words are not in the final cue; its final interval has no window")
    for r in audit["prediction_holds"] + audit["reading_holds"]:
        assert r["cue_text"] == parsed[r["cue"]] and r["cue_text_sha256"] == sha(parsed[r["cue"]].encode("utf-8"))
    print("every audited hold carries its cue's exact text and text hash")
    for h in audit["helpers"] + audit["consumers"]:
        path = CH / "draw.rs" if h["path"].endswith("chapters/draw.rs") else STAGE / "src/main.rs" if h["path"].endswith("/src/main.rs") else Path(h["path"])
        data = path.read_bytes()
        assert sha(data) == h["sha256"] and data.decode("utf-8").split("\n")[h["line"] - 1].strip() == h["marker"], h
    print(f"{len(audit['helpers'])} helper rows and {len(audit['consumers'])} consumer rows: hash and marker at line hold")
    assert {r["path"] for r in direction["shared_helpers"]} == {h["path"] for h in audit["helpers"]}
    for r in audit["reading_holds"]:
        if r["cue"] == "11_20":
            assert sum(len(x) for x in r["rule_lines"]) == 159 and sum(len(x) for x in r["row_verdicts"]) == 107
            assert abs(266 / 15 - r["window_seconds"]) < 1e-4
            print(f"11_20: {r['rule_chars']} + {r['row_chars']} characters at {r['characters_per_second']} per second = {r['window_seconds']} s of window, from the source's literals; source floor {r['source_floor_after_voice_s']} s after the voice")
        else:
            assert r["window_seconds"] is None and r["conditional"]
            print(f"10_4: conditional window, no number claimed; {r['fine_print_chars']} characters at the {r['floor_seconds_from_shown']} s floor = {r['characters_per_second_at_floor']} per second")

    # ---------------------------------------------------------------- the actual consumer, on synthetic numbers
    delivery = load("narration_delivery", "tools/narration_delivery.py")
    declared = Counter((h.get("cue"), h.get("next_cue")) for h in direction.get("visual_holds", []) if h.get("video") == TOPIC and h.get("kind") == "prediction")
    assert len(declared) == 13 and set(declared.values()) == {1}
    fps = 60
    outputs = {c: {"frames": 24000, "sample_rate": 24000} for c in cues}
    for cue, nxt in declared:
        line, following = ("line", cue, "600"), ("line", nxt, "1500")
        assert delivery.prediction_end(line, following, fps, [12.0], outputs, declared, 5.0) == 30.0, cue
        assert delivery.prediction_end(line, following, fps, [], outputs, declared, 5.0) is None, cue
    line, following = ("line", "11_20", "600"), ("line", "11_8", "1500")
    assert delivery.prediction_end(line, following, fps, [12.0], outputs, declared, 5.0) is None
    print("narration_delivery.prediction_end, synthetic numbers: 13 declared pairs extend with a mark and do not without one; reading pair 11_20->11_8 is not a prediction and never extends")

    # ---------------------------------------------------------------- dispositions: no invented silence
    rows = proposed["direction_dispositions"]
    assert {r["kind"] for r in rows} == {"native_visual_hold"} and len(rows) == 15
    assert proposed["map_sha256"] == sha(map_text.encode("utf-8"))
    reviewed = coherent.review_directions(bound, units, {"direction_dispositions": rows})
    lines = {c: [{"sentence": s, "gap_after_samples": 0} for s in coherent.sentences(t)] for c, t in parsed.items()}
    coherent.check_dispositions(reviewed, lines)
    print("coherent.review_directions and check_dispositions accept 15 native_visual_hold rows with zero inserted samples (structure only; there is no take)")

    # ---------------------------------------------------------------- phrase anchors
    problems = anchor_problems(anchors, template, parsed)
    assert not problems, problems
    hook = re.compile(r'(?:until_phrase\((?:&kit, stage|stage, &kit), |phrase_time\(&kit, )"\d\d_\d+"')
    found = 0
    for name in {r["file"] for r in audit["registered_chapters"]}:
        scene = (CH / name).read_text(encoding="utf-8").split("#[cfg(test)]\nmod tests")[0]
        found += sum(1 for line in scene.split("\n") if hook.search(line) and not line.strip().startswith("//"))
    sites = sum(1 for r in anchors["anchors"] for s in r["hook_sites"] if s["call"] in ("until_phrase", "phrase_time"))
    assert found == sites, (found, sites)
    c = anchors["counts"]
    assert c["anchors"] == len(anchors["anchors"]) == 24 and c["measured"] == 0
    print(f"{c['anchors']} phrase anchors in {c['cues_with_anchors']} cues, each exactly where text.find lands, once, at a word start; a separate scan counts the same {found} until_phrase and phrase_time calls")
    print(f"affected by a split: {c['affected_by_cue_split']}; with a new cue or byte: {c['byte_or_cue_moved']}; measured: {c['measured']}")
    print(f"template: {len(template['cues'])} cues, every wav_sha256 and seconds null")
    assert all(s["cue"] in parsed and s["sentence_index"] < len(coherent.sentences(parsed[s["cue"]])) for s in anchors["sentence_share_hooks"]["rows"])
    print(f"{len(anchors['sentence_share_hooks']['rows'])} sentence share hooks, listed apart: not byte anchors")

    # ---------------------------------------------------------------- cue delta
    joined = {}
    for d in delta["cues"]:
        joined.setdefault(d["baseline_cue"], []).append(parsed[d["cue"]])
        assert d["text_sha256"] == sha(parsed[d["cue"]].encode("utf-8"))
    inventory = json.loads((LANE / "narration-request-inventory01.json").read_text(encoding="utf-8"))
    for row in inventory["cues"]:
        assert " ".join(joined[row["cue"]]).encode("utf-8") == Path(row["text_path"]).read_bytes(), row["cue"]
    print(f"cue delta: the 80 cues rejoin to root's 75 request texts byte for byte; {delta['counts']}")

    # ---------------------------------------------------------------- reference only
    ref = json.loads(REFERENCE.read_text(encoding="utf-8"))
    ref_holds = {r["cue"]: r for r in ref["visual_holds"]}
    new_holds = {r["cue"]: r for r in direction["visual_holds"]}
    assert set(new_holds) - set(ref_holds) == {"11_20"} and not set(ref_holds) - set(new_holds)
    same_interval = [c for c in ref_holds if ref_holds[c]["interval_sha256"] == new_holds[c]["interval_sha256"]]
    moved = [f"{c} {ref_holds[c]['line']}->{new_holds[c]['line']}" for c in ref_holds if ref_holds[c]["line"] != new_holds[c]["line"]]
    print(f"reference only, narration-binding-stage01 expanded01: 14 of its rows carry over, 11_20 is new; same interval hash on {len(same_interval)}; line moves: {moved}")
    print(f"reference only, 10_4's marker: expanded01 {ref_holds['10_4']['marker']!r}; this map {new_holds['10_4']['marker']!r}")

    # ---------------------------------------------------------------- at this run: what is installed
    print("proposed versus installed canonical, at this run:")
    for rel in ["script.tsv", "narration_units.json", "src/chapters/draw.rs"] + [f"src/chapters/{r['file']}" for r in audit["registered_chapters"]]:
        new = BUNDLE / REL / rel if rel == "narration_units.json" else STAGE / rel
        old = Path(REL) / rel
        state = "same" if old.is_file() and old.read_bytes() == new.read_bytes() else "not installed" if old.is_file() else "no such canonical file"
        print(f"  {rel}: {state}")
    print(f"  docs/{MAP}: {'present' if (Path(REL) / 'docs' / MAP).exists() else 'not installed'}")

    # ---------------------------------------------------------------- portable paths
    leak = re.compile("/" + "Users/|/" + "home/|/" + "private/tmp|~" + "/")
    for path in sorted(BUNDLE_ABS.rglob("*")):
        if path.is_file() and leak.search(path.read_text(encoding="utf-8", errors="replace")):
            raise AssertionError(f"an absolute home path in {path.name}")
    print("no absolute home or scratch path in any bundle file")

    # ---------------------------------------------------------------- planted defects
    results = []

    def planted(name, mutate_plan=None, mutate_map=None, raw_map=None):
        p, m = copy.deepcopy(plan), copy.deepcopy(direction)
        text = raw_map or map_text
        if mutate_map:
            mutate_map(m)
            text = json.dumps(m, indent=2, ensure_ascii=False) + "\n"
            p["direction_contract"]["map"]["sha256"] = sha(text.encode("utf-8"))
        if mutate_plan:
            mutate_plan(p)
        where = scratch / "planted" / name
        topic = install(where, script_bytes, json.dumps(p, indent=2, ensure_ascii=False) + "\n", text)
        try:
            contract_check(coherent_at(where), topic / "script.tsv", topic / "narration_units.json")
        except (ValueError, KeyError) as error:
            print(f"  refused {name}: {error}")
            results.append(True)
            return
        print(f"  NOT REFUSED {name}")
        results.append(False)

    def edit_text(p):
        p["units"][3]["text"] += " extra"

    def drop_unit(p):
        del p["units"][5]

    def swap_units(p):
        p["units"][0], p["units"][1] = p["units"][1], p["units"][0]

    def too_long(p):
        at = next(i for i, u in enumerate(p["units"]) if u["id"] == "oliver_11_7")
        u, v = p["units"][at], p["units"][at + 1]
        p["units"][at : at + 2] = [{"id": u["id"], "line_ids": u["line_ids"] + v["line_ids"], "text": u["text"] + " " + v["text"]}]

    def foreign_phrase(m):
        m["overrides"][2]["after"] = "A phrase nobody says"

    def duplicate(m):
        m["overrides"].append(dict(m["overrides"][0]))

    def mid_cue(m):
        m["overrides"][0]["after"] = " ".join(m["overrides"][0]["after"].split()[:5])

    def old_pair(m):
        stale = json.loads((STAGE / "docs/narration-direction-map-oliver11.json").read_text(encoding="utf-8"))
        m["overrides"].append(dict(stale["overrides"][0]))

    print("planted defects, coherent API:")
    planted("unit-text-changed", mutate_plan=edit_text)
    planted("unit-dropped", mutate_plan=drop_unit)
    planted("units-reordered", mutate_plan=swap_units)
    planted("unit-over-100-words", mutate_plan=too_long)
    planted("map-bytes-changed", raw_map=map_text.replace("13 prediction holds", "14 prediction holds", 1))
    planted("phrase-not-in-cue", mutate_map=foreign_phrase)
    planted("direction-duplicated", mutate_map=duplicate)
    planted("direction-not-at-cue-end", mutate_map=mid_cue)
    planted("oliver11-direction-back", mutate_map=old_pair)

    print("planted defects, source binding:")

    def hold_case(name, mutate):
        m = copy.deepcopy(direction)
        mutate(m)
        found_ = hold_problems(m, cues)
        print(f"  {'refused' if found_ else 'NOT REFUSED'} {name}: {found_[:1]}")
        results.append(bool(found_))

    def row(m, cue):
        return next(r for r in m["visual_holds"] if r["cue"] == cue)

    hold_case("line-shifted", lambda m: row(m, "12_5").update(line=row(m, "12_5")["line"] + 1))
    hold_case("wrong-next-cue", lambda m: row(m, "05_1").update(next_cue="05_20"))
    hold_case("file-hash-changed", lambda m: row(m, "00_4").update(sha256="0" * 64))
    hold_case("interval-changed", lambda m: row(m, "08_4").update(interval_sha256="0" * 64))
    hold_case("obsolete-pair-back", lambda m: m["visual_holds"].append({**row(m, "00_4"), "cue": "00_1", "next_cue": "00_2"}))
    hold_case("reading-row-dropped", lambda m: m["visual_holds"].remove(row(m, "11_20")))

    print("planted defects, phrase anchors:")

    def anchor_case(name, mutate):
        a, t = copy.deepcopy(anchors), copy.deepcopy(template)
        mutate(a, t)
        found_ = anchor_problems(a, t, parsed)
        print(f"  {'refused' if found_ else 'NOT REFUSED'} {name}: {found_[:1]}")
        results.append(bool(found_))

    anchor_case("byte-shifted", lambda a, t: a["anchors"][17].update(byte=a["anchors"][17]["byte"] + 1))
    anchor_case("invented-seconds", lambda a, t: a["anchors"][0].update(measured_seconds=1.25))
    anchor_case("invented-template-seconds", lambda a, t: t["cues"]["11_20"]["marks"][0].update(seconds=3.5))
    anchor_case("template-bound-to-a-wav", lambda a, t: t["cues"]["11_20"].update(wav_sha256="0" * 64))

    print("planted defects, dispositions:")

    def disposition_case(name, mutate):
        r = copy.deepcopy(rows)
        mutate(r)
        try:
            checked = coherent.review_directions(bound, units, {"direction_dispositions": r})
            coherent.check_dispositions(checked, lines)
        except (ValueError, KeyError) as error:
            print(f"  refused {name}: {error}")
            results.append(True)
            return
        print(f"  NOT REFUSED {name}")
        results.append(False)

    disposition_case("invented-silence-with-no-gap", lambda r: r[0].update(kind="authored_silence"))
    disposition_case("a-direction-with-no-disposition", lambda r: r.pop())
    disposition_case("evidence-hash-wrong", lambda r: r[1]["evidence"].update(sha256="0" * 64))

    assert all(results), "a planted defect was not refused"
    print(f"{len(results)} planted defects refused")
    print("VALIDATION PASS: authored coverage and source binding only; no take, timing, picture, listening or acceptance")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
