#!/usr/bin/env python3
"""Show that the binder and validator refuse what they claim to refuse.

    python3 -I -B planted_defects.py REPO CRATE PLAN_DIR EMPTY_SCRATCH

Copies the crate into the scratch, plants one defect at a time in the copy, and
expects a refusal naming it. Starts with two controls: an unchanged copy must bind
to the same bytes as PLAN_DIR and must validate. Never touches CRATE or PLAN_DIR.
"""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BIND = str(HERE / "bind_oliver_plan.py")
VALIDATE = str(HERE / "validate_oliver_plan.py")
TOPIC_REL = "explainer_corpus/complex_analysis_visual"
MAP = "docs/narration-direction-map-oliver08.json"
WINDOW = "reading_window(&kit, stage, &line)?;"


def run(script, *args):
    done = subprocess.run([sys.executable, "-I", "-B", script, *args], capture_output=True, text=True)
    return done.returncode, done.stdout + done.stderr


def edit(root, rel, old, new, count=1):
    path = Path(root) / rel
    text = path.read_text(encoding="utf8")
    if text.count(old) < 1:
        raise SystemExit(f"planted defect does not apply: {rel}: {old!r}")
    path.write_text(text.replace(old, new, count), encoding="utf8")


# (name, [(file in the crate, old, new), ...], what the refusal must mention)
SEVERAL = [
    (
        "cues truly swapped in a chapter",
        [
            ("src/chapters/turning.rs", '"02_1")?;', '"02_X")?;'),
            ("src/chapters/turning.rs", '"02_2")?;', '"02_1")?;'),
            ("src/chapters/turning.rs", '"02_X")?;', '"02_2")?;'),
        ],
        "out of script order",
    ),
    (
        "prediction no longer a question, storyboard agreeing",
        [
            ("script.tsv", "So what stops the recipe at exactly one?", "So what stops the recipe at exactly one."),
            ("docs/storyboard.md", "So what stops the recipe at exactly one?", "So what stops the recipe at exactly one."),
        ],
        "does not end in a question",
    ),
    (
        "prediction on a chapter's last cue, which delivery would not hold",
        [
            ("script.tsv", "First, we need a way to see complex functions at all.", "First, do we need a way to see complex functions at all?"),
            ("docs/storyboard.md", "First, we need a way to see complex functions at all.", "First, do we need a way to see complex functions at all?"),
            ("docs/storyboard.md", "| 01_6 | So points", "| 01_6 | ⟨predict⟩ So points"),
            ("docs/storyboard.md", "| arrows | hold |", "| arrows | listen 5 s |"),
            ("src/chapters/mod.rs", '    ("01_4", 5.0),\n', '    ("01_4", 5.0),\n    ("01_6", 5.0),\n'),
            ("src/chapters/the_bump.rs", "kit.music_breath(stage, 2.0)?;", "let _ = predict(&kit, stage, &line, &ASK, v(4.9, 3.0))?;"),
        ],
        "no next cue in its chapter",
    ),
    (
        "a window with too few seconds, storyboard agreeing",
        [
            ("src/chapters/mod.rs", '("10_5", 3.0)', '("10_5", 0.5)'),
            ("docs/storyboard.md", "highlighted on the theorem card. | | read 3 s |", "highlighted on the theorem card. | | read 0.5 s |"),
        ],
        "is outside",
    ),
    (
        "a window so long the gap passes 4 s, storyboard agreeing",
        [
            ("src/chapters/mod.rs", '("10_5", 3.0)', '("10_5", 9.0)'),
            ("docs/storyboard.md", "highlighted on the theorem card. | | read 3 s |", "highlighted on the theorem card. | | read 9 s |"),
        ],
        "is outside",
    ),
    (
        "a window on a prediction: the delay-the-answer rule itself",
        [
            ("src/chapters/mod.rs", '    ("10_5", 3.0),\n', '    ("10_4", 3.0),\n    ("10_5", 3.0),\n'),
            ("src/chapters/rigidity.rs", 'let line = say(&kit, stage, "10_5")?;', WINDOW + '\n        let line = say(&kit, stage, "10_5")?;'),
        ],
        "it would delay the answer",
    ),
    (
        "a window on a chapter's last cue, table and call agreeing",
        [
            ("src/chapters/mod.rs", '    ("14_3", 2.5),\n', '    ("13_11", 2.5),\n    ("14_3", 2.5),\n'),
            ("src/chapters/real_integral.rs", "kit.music_breath(stage, 2.5)?;\n        let mut all = vec![", WINDOW + "\n        let mut all = vec!["),
        ],
        "no next cue in its chapter",
    ),
    (
        "a window on a cue with no card: the no-card rule itself",
        [
            ("src/chapters/mod.rs", '    ("04_4", 2.5),\n', '    ("04_3", 2.5),\n    ("04_4", 2.5),\n'),
            ("src/chapters/reading_color.rs", 'let line = say(&kit, stage, "04_4")?;', WINDOW + '\n        let line = say(&kit, stage, "04_4")?;'),
        ],
        "no precisely card",
    ),
]

# (name, file in the crate, old, new, what the refusal must mention)
BINDER = [
    ("say with a computed cue", "src/chapters/boundary.rs", 'let line = say(&kit, stage, "09_2")?;', 'let line = say(&kit, stage, cue)?;', "say is not"),
    ("say removed", "src/chapters/boundary.rs", 'let line = say(&kit, stage, "09_2")?;', 'let line = say(&kit, stage, "09_1")?;', "09_2 is said 0"),
    ("say inside a loop", "src/chapters/rigidity.rs", 'let line = say(&kit, stage, "10_2")?;', 'for _ in 0..1 { let line = say(&kit, stage, "10_2")?; }', "say is not"),
    ("say inside a loop on its own line", "src/chapters/rigidity.rs", '        let line = say(&kit, stage, "10_2")?;', '        for _ in 0..1 {\n        let line = say(&kit, stage, "10_2")?;\n        }', "may not run once"),
    ("kit.hold through another receiver", "src/chapters/turning.rs", 'kit.hold(stage)?;', 'self.kit.hold(stage)?;', "could not read"),
    ("a cue past an early #[cfg(test)] use goes missing", "src/chapters/one_derivative.rs", 'let line = say(&kit, stage, "06_3")?;', 'let line = say(&kit, stage, "06_2")?;', "06_3 is said 0"),
    ("predict call removed", "src/chapters/the_bump.rs", "let asked = predict(&kit, stage, &line, &ASK, v(4.9, 3.0))?;", "let asked = Vec::new();", "predict calls and PREDICTS differ"),
    ("PREDICTS row removed", "src/chapters/mod.rs", '    ("01_4", 5.0),\n', "", "predict calls and PREDICTS differ"),
    ("window differs from the storyboard", "src/chapters/mod.rs", '("03_3", 4.0)', '("03_3", 7.0)', "differs from PREDICTS"),
    ("predict given another line", "src/chapters/the_bump.rs", "predict(&kit, stage, &line, &ASK, v(4.9, 3.0))", "predict(&kit, stage, &first, &ASK, v(4.9, 3.0))", "not given the current line"),
    ("predict order changed in draw.rs", "src/chapters/draw.rs", '    kit.hold(stage)?;\n    kit.mark(stage, "predict");', '    kit.mark(stage, "predict");\n    kit.hold(stage)?;', "order changed"),
    ("a wait after the voice", "src/chapters/boundary.rs", '        kit.hold(stage)?;\n        let line = say(&kit, stage, "09_2")?;', '        kit.hold(stage)?;\n        pause(stage, 3.0)?;\n        let line = say(&kit, stage, "09_2")?;', "unclassified post-voice hold"),
    ("a stage.wait after the voice", "src/chapters/boundary.rs", '        kit.hold(stage)?;\n        let line = say(&kit, stage, "09_2")?;', '        kit.hold(stage)?;\n        stage.wait(3.0)?;\n        let line = say(&kit, stage, "09_2")?;', "unclassified post-voice hold"),
    ("a wait between predict and the next cue", "src/chapters/boundary.rs", 'let asked = predict(&kit, stage, &line, &ASK, v(right, -2.0))?;', 'let asked = predict(&kit, stage, &line, &ASK, v(right, -2.0))?;\n        pause(stage, 1.0)?;', "between predict"),
    ("a music breath inside a chapter", "src/chapters/boundary.rs", '        kit.hold(stage)?;\n        let line = say(&kit, stage, "09_2")?;', '        kit.music_breath(stage, 2.0)?;\n        let line = say(&kit, stage, "09_2")?;', "music breath inside"),
    ("a phrase not in its cue", "src/chapters/the_bump.rs", 'line.wait_for(stage, "At one point one")?;', 'line.wait_for(stage, "At one point two")?;', "is not in cue 01_3"),
    ("a sentence its cue lacks", "src/chapters/the_bump.rs", "line.wait_sentence(stage, 3)?;", "line.wait_sentence(stage, 30)?;", "does not exist in cue"),
    ("a new non-literal wait", "src/chapters/the_bump.rs", 'line.wait_for(stage, "At one point one")?;', "line.wait_for(stage, somewhere)?;", "nobody has resolved"),
    ("a changed non-literal wait", "src/chapters/counting.rs", "line.wait_sentence(stage, i + 2)?;", "line.wait_sentence(stage, i + 3)?;", "nobody has resolved"),
    ("a dynamic phrase's source changed", "src/chapters/gallery.rs", '"at minus one", "at minus two"];', '"at minus one", "at minus nine"];', "DYNAMIC's source line is gone"),
    ("a cue over 100 words", "script.tsv", "So what stops the recipe at exactly one?", "So what stops the recipe at exactly one?" + " word" * 101, "outside 1 to 100"),
    ("script word changed", "script.tsv", "So what stops the recipe at exactly one?", "So what halts the recipe at exactly one?", "differ from script.tsv"),
    ("an unclassified storyboard hold", "docs/storyboard.md", "| hold |", "| wait 4 s |", "unclassified"),
    ("READING_WINDOWS row removed", "src/chapters/mod.rs", '    ("05_8", 2.0),\n', "", "reading_window calls and READING_WINDOWS differ"),
    ("a reading_window call removed", "src/chapters/gallery.rs", WINDOW, "kit.hold(stage)?;", "reading_window calls and READING_WINDOWS differ"),
    ("table differs from the storyboard", "src/chapters/mod.rs", '("09_5", 2.0)', '("09_5", 2.5)', "differs from READING_WINDOWS"),
    ("storyboard's reading window gone", "docs/storyboard.md", "| live readout | read 2 s |", "| live readout | |", "READING_WINDOWS has 09_5"),
    ("window moved after the next say", "src/chapters/boundary.rs", WINDOW + '\n        let line = say(&kit, stage, "09_6")?;', 'kit.hold(stage)?;\n        let line = say(&kit, stage, "09_6")?;\n        ' + WINDOW, "READING_WINDOWS differ"),
    ("a fade between the window and the next say", "src/chapters/rigidity.rs", WINDOW + '\n        let line = say(&kit, stage, "10_6")?;', WINDOW + '\n        fade_out_all(stage, &[exact], 0.3)?;\n        let line = say(&kit, stage, "10_6")?;', "between the reading window"),
    ("the card cleared before the window", "src/chapters/rigidity.rs", "        " + WINDOW + '\n        let line = say(&kit, stage, "10_6")?;', "        fade_out_all(stage, &[exact], 0.3)?;\n        " + WINDOW + '\n        let line = say(&kit, stage, "10_6")?;', "can go before its window"),
    ("a pause added after the window", "src/chapters/continuing.rs", WINDOW + '\n        let line = say(&kit, stage, "14_4")?;', WINDOW + '\n        pause(stage, 1.0)?;\n        let line = say(&kit, stage, "14_4")?;', "unclassified post-voice hold"),
    ("a window given another line", "src/chapters/one_point.rs", WINDOW, "reading_window(&kit, stage, &first)?;", "not given the current line"),
    ("helper order swapped", "src/chapters/draw.rs", "    wait_until(stage, line.end())?;\n    kit.hold(stage)?;\n    pause(stage, seconds)\n", "    pause(stage, seconds)?;\n    wait_until(stage, line.end())?;\n    kit.hold(stage)\n", "reading_window's order changed"),
    ("helper waits before the voice ends", "src/chapters/draw.rs", "    wait_until(stage, line.end())?;\n    kit.hold(stage)?;\n    pause(stage, seconds)\n", "    kit.hold(stage)?;\n    wait_until(stage, line.end())?;\n    pause(stage, seconds)\n", "reading_window's order changed"),
    ("helper gains a fade", "src/chapters/draw.rs", "    kit.hold(stage)?;\n    pause(stage, seconds)\n", "    kit.hold(stage)?;\n    fade_in_all(stage, &[], 0.1)?;\n    pause(stage, seconds)\n", "clears or draws"),
    ("the card not cleared by the next cue", "src/chapters/rigidity.rs", "listed[1], not_in, exact]", "listed[1], not_in]", "is not cleared"),
    ("a registered chapter missing from disk", "src/chapters/mod.rs", "mod gallery;\n", "mod gallery;\nmod gone_missing;\n", "not on disk"),
]


def validator_cases():
    def unit_text(units, direction):
        units["units"][3]["text"] += " Extra."

    def reordered(units, direction):
        units["units"][0], units["units"][1] = units["units"][1], units["units"][0]

    def merged(units, direction):
        units["units"][0]["line_ids"] = ["01_1", "01_2"]

    def wrong_map_path(units, direction):
        units["direction_contract"]["map"]["path"] = TOPIC_REL + "/docs/narration-direction-map-oliver06.json"

    def wrong_next(units, direction):
        direction["visual_holds"][0]["next_cue"] = "01_6"

    def moved_marker(units, direction):
        direction["visual_holds"][0]["line"] += 1

    def wrong_hash(units, direction):
        direction["visual_holds"][0]["sha256"] = "0" * 64

    def timing(units, direction):
        direction["overrides"][0]["range_s"] = [1.0, 2.0]

    def mid_cue(units, direction):
        direction["overrides"][0]["after"] = direction["overrides"][0]["after"].split(". ")[0]

    def dropped_hold(units, direction):
        direction["visual_holds"].pop()
        direction["overrides"].pop()
        direction["hold_kinds"]["prediction"] -= 1

    def reading_range(units, direction):
        hold = next(h for h in direction["visual_holds"] if h["kind"] == "reading")
        hold["range_s"] = [1.0, 2.0]

    def invented_reading(units, direction):
        hold = next(h for h in direction["visual_holds"] if h["kind"] == "reading")
        extra = dict(hold, cue="01_1", next_cue="01_2", marker="kit.hold(stage)?;")
        direction["visual_holds"].insert(0, extra)
        over = next(o for o in direction["overrides"] if o["cue"] == hold["cue"])
        direction["overrides"].insert(0, dict(over, cue="01_1", after=units["units"][0]["text"]))
        direction["hold_kinds"]["reading"] += 1

    def dropped_reading(units, direction):
        cue = next(h["cue"] for h in direction["visual_holds"] if h["kind"] == "reading")
        direction["visual_holds"] = [h for h in direction["visual_holds"] if h["cue"] != cue]
        direction["overrides"] = [o for o in direction["overrides"] if o["cue"] != cue]
        direction["hold_kinds"]["reading"] -= 1

    def retyped_reading(units, direction):
        next(h for h in direction["visual_holds"] if h["kind"] == "reading")["kind"] = "prediction"
        direction["hold_kinds"] = {"prediction": 14, "reading": 6}

    def reading_count(units, direction):
        direction["hold_kinds"]["reading"] = 0

    def reading_marker(units, direction):
        next(h for h in direction["visual_holds"] if h["kind"] == "reading")["marker"] = "kit.hold(stage)?;"

    def reading_next(units, direction):
        hold = next(h for h in direction["visual_holds"] if h["kind"] == "reading")
        hold["next_cue"] = hold["next_cue"][:3] + "99"

    def reading_override(units, direction):
        cue = next(h["cue"] for h in direction["visual_holds"] if h["kind"] == "reading")
        direction["overrides"] = [o for o in direction["overrides"] if o["cue"] != cue]

    return [
        ("a reading hold carries an invented range", reading_range),
        ("a reading row with no added wait", invented_reading),
        ("a reading row removed", dropped_reading),
        ("a reading row retyped as a prediction", retyped_reading),
        ("hold_kinds says no reading", reading_count),
        ("a reading row's marker changed", reading_marker),
        ("a reading row's next cue changed", reading_next),
        ("a reading row's override dropped", reading_override),
        ("unit text changed", unit_text),
        ("units reordered", reordered),
        ("two cues in one unit", merged),
        ("contract names the old map", wrong_map_path),
        ("hold names the wrong next cue", wrong_next),
        ("hold marker line moved", moved_marker),
        ("hold hash stale", wrong_hash),
        ("override carries an invented range", timing),
        ("override stops mid-cue", mid_cue),
        ("a prediction hold dropped", dropped_hold),
    ]


def main():
    repo, crate, plan_dir, scratch = (Path(p).resolve() for p in sys.argv[1:5])
    scratch.mkdir(parents=True, exist_ok=True)
    if any(scratch.iterdir()):
        raise SystemExit(f"{scratch} is not empty")
    failed = 0

    # controls: an unchanged copy binds to PLAN_DIR's bytes and validates
    control = scratch / "control"
    shutil.copytree(crate, control / "crate")
    code, text = run(BIND, str(repo), str(control / "crate"), str(control / "out"))
    same = code == 0 and all(
        (control / "out" / rel).read_bytes() == (plan_dir / rel).read_bytes()
        for rel in (f"{TOPIC_REL}/narration_units.json", f"{TOPIC_REL}/{MAP}", "source-audit.json")
    )
    print(f"{'ok  ' if same else 'FAIL'} control: unchanged crate binds to the same bytes")
    failed += not same
    # an old file left on disk says a cue, but mod.rs does not register it, so the map cannot change
    old = control / "crate/src/chapters/left_over.rs"
    old.write_text('fn construct() {\n    let line = say(&kit, stage, "01_1")?;\n}\n', encoding="utf8")
    code, text = run(BIND, str(repo), str(control / "crate"), str(control / "out-old"))
    map_same = code == 0 and (control / "out-old" / TOPIC_REL / MAP).read_bytes() == (plan_dir / TOPIC_REL / MAP).read_bytes()
    print(f"{'ok  ' if map_same else 'FAIL'} control: an unregistered old file with a say is ignored; the map's bytes are unchanged")
    failed += not map_same
    old.unlink()
    code, text = run(VALIDATE, str(repo), str(crate), str(plan_dir), str(control / "validate"))
    print(f"{'ok  ' if code == 0 else 'FAIL'} control: the plan validates")
    failed += code != 0

    cases = [(name, [(rel, old, new)], expected) for name, rel, old, new, expected in BINDER] + SEVERAL
    for i, (name, edits, expected) in enumerate(cases):
        case = scratch / f"bind{i:02d}"
        shutil.copytree(crate, case / "crate")
        for rel, old, new in edits:
            edit(case / "crate", rel, old, new)
        code, text = run(BIND, str(repo), str(case / "crate"), str(case / "out"))
        good = code != 0 and expected in text and not (case / "out").exists()
        print(f"{'ok  ' if good else 'FAIL'} binder refuses: {name}")
        if not good:
            failed += 1
            print(f"     exit {code}, expected {expected!r}; output: {text[:500]}")
        shutil.rmtree(case)

    for i, (name, plant) in enumerate(validator_cases()):
        case = scratch / f"validate{i:02d}"
        shutil.copytree(plan_dir / TOPIC_REL, case / TOPIC_REL)
        units_path, map_path = case / TOPIC_REL / "narration_units.json", case / TOPIC_REL / MAP
        units = json.loads(units_path.read_text(encoding="utf8"))
        direction = json.loads(map_path.read_text(encoding="utf8"))
        plant(units, direction)
        map_text = json.dumps(direction, indent=2, ensure_ascii=False) + "\n"
        map_path.write_text(map_text, encoding="utf8")
        # keep the contract's map hash current, so only the planted defect can refuse
        import hashlib

        units["direction_contract"]["map"]["sha256"] = hashlib.sha256(map_text.encode()).hexdigest()
        units_path.write_text(json.dumps(units, indent=2, ensure_ascii=False) + "\n", encoding="utf8")
        code, text = run(VALIDATE, str(repo), str(crate), str(case), str(case / "scratch"))
        print(f"{'ok  ' if code != 0 else 'FAIL'} validator refuses: {name}")
        if code == 0:
            failed += 1
        elif "REFUSED" not in text and "ValueError" not in text:
            print(f"     refused by a crash, not a check: {text[-300:]}")
            failed += 1
        shutil.rmtree(case)

    print(f"{len(cases)} binder defects, {len(validator_cases())} validator defects, 3 controls; {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
