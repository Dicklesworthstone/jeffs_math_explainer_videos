#!/usr/bin/env python3
"""Does the preflight catch what it claims to catch? Run from the repo root:

    python3 -I -B test_preflight.py

Each case changes one thing in a disposable copy, rebinds the hashes so that a semantic check
has to do the catching, and runs the real preflight CLI with pins off. The real tree is read only.
A case passes only if every named check FAILs. The decoy cases must leave those checks passing.
"""
import importlib.util
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("stage06_sandbox", HERE / "sandbox.py")
sandbox = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = sandbox
spec.loader.exec_module(sandbox)
pf, Box, sha = sandbox.pf, sandbox.Box, sandbox.sha
ZERO = "0" * 64
SAY_03_2 = 'kit.say(stage, "03_2")?;'
CASES = []


def case(*must_fail):
    def register(fn):
        CASES.append((fn.__name__, fn, must_fail))
        return fn
    return register


def rows(key):
    return lambda d: d[key]


# ------------------------------------------------------------------ exact script and unit text
@case("TXT.unit_text_exact", "TXT.request_files_exact", "group_consumers.completed")
def script_character_changed(b):
    b.sub(b.crate / "script.tsv", "Here are seven notes", "Here are sevan notes")


@case("PHR.10_4@39.exact_byte", "TXT.unit_text_exact")
def script_rewords_an_anchor(b):
    b.sub(b.crate / "script.tsv", "keep the turning strictly between minus pi", "keep the turning strictly within minus pi")


@case("TXT.unit_text_exact", "group_consumers.completed")
def unit_text_differs(b):
    b.edit_json(b.units, lambda d: d["units"][0].__setitem__("text", d["units"][0]["text"] + " Extra."))


@case("TXT.units_one_per_cue_in_order", "group_consumers.completed")
def unit_removed(b):
    b.edit_json(b.units, lambda d: d["units"].pop(10))


@case("TXT.units_one_per_cue_in_order", "group_consumers.completed")
def units_reordered(b):
    def change(d):
        d["units"][3], d["units"][4] = d["units"][4], d["units"][3]
    b.edit_json(b.units, change)


@case("TXT.contract_map_hash", "group_consumers.completed")
def contract_map_hash_wrong(b):
    b.edit_json(b.units, lambda d: d["direction_contract"]["map"].__setitem__("sha256", ZERO))


@case("TXT.contract_names_map13")
def units_bind_oliver12(b):
    def change(d):
        d["direction_contract"]["map"]["path"] = d["direction_contract"]["map"]["path"].replace("oliver13", "oliver12")
    b.edit_json(b.units, change)


@case("TXT.request_files_exact", "CON.request_text")
def request_text_differs(b):
    path = b.root / pf.LANE / "narration-request-final80-01/05_20.txt"
    b.write(path, path.read_bytes() + b" Extra.")


@case("TXT.requests_bind_script")
def requests_bind_another_script(b):
    b.edit_json(b.requests, lambda d: d.__setitem__("script_sha256", ZERO))


@case("TXT.older_request_bytes_same")
def older_request_text_differs(b):
    path = b.root / pf.LANE / "narration-request-text01/00_1.txt"
    b.write(path, path.read_bytes() + b" Extra.")


@case("TXT.delta_binds")
def cue_delta_text_hash_wrong(b):
    b.edit_json(b.plan / "cue-delta.json", lambda d: d["cues"][0].__setitem__("text_sha256", ZERO))


# ------------------------------------------------------------------ registry and cue coverage
@case("REG.thirteen", "REG.every_cue_said_once_in_order")
def chapter_removed_from_registry(b):
    path = b.chapters / "mod.rs"
    text = path.read_text(encoding="utf-8")
    new, n = re.subn(r'\s*Chapter \{\s*key: "03_folded_decade",.*?\},', "", text, count=1, flags=re.S)
    assert n == 1
    b.write(path, new.encode("utf-8"))


@case("REG.in_cue_order", "REG.scene_name_is_key")
def registry_keys_swapped(b):
    path = b.chapters / "mod.rs"
    text = path.read_text(encoding="utf-8").replace('"00_ratio_ruler"', "@@").replace('"01_building_ruler"', '"00_ratio_ruler"').replace("@@", '"01_building_ruler"')
    b.write(path, text.encode("utf-8"))


@case("REG.scene_name_is_key")
def scene_name_differs(b):
    b.chapter("folded_decade.rs", 'fn name(&self) -> &str {\n        "03_folded_decade"', 'fn name(&self) -> &str {\n        "03_folded_decad"', canon=True)


@case("REG.every_cue_said_once_in_order")
def cue_said_twice(b):
    b.chapter("folded_decade.rs", SAY_03_2, SAY_03_2 + " " + SAY_03_2, canon=True)


@case("REG.every_cue_said_once_in_order")
def cue_not_said(b):
    b.chapter("folded_decade.rs", SAY_03_2, "/* gone */", canon=True)


@case("REG.nothing_unreadable")
def say_without_a_literal(b):
    b.chapter("folded_decade.rs", SAY_03_2, "kit.say(stage, WHICH)?;", canon=True)


@case("REG.sentence_hooks_in_range")
def sentence_hook_out_of_range(b):
    b.chapter("undoing_growth.rs", 'kit.until_sentence(stage, "04_1", 1)?;', 'kit.until_sentence(stage, "04_1", 99)?;')


@case("REG.sentence_hooks_on_current_cue")
def sentence_hook_on_another_cue(b):
    b.chapter("undoing_growth.rs", 'kit.until_sentence(stage, "04_1", 1)?;', 'kit.until_sentence(stage, "04_3", 1)?;')


@case("REG.oracle_reproduces")
def oracle_changed(b):
    path = b.stage / "oracle04.tsv"
    b.write(path, path.read_bytes() + b"made.up\t1\n", rebind=False)


# ------------------------------------------------------------------ holds
@case("HOLD.00_4.marker_line", "HOLD.census_equals_map")
def hold_line_moved(b):
    b.edit_json(b.map, lambda d: d["visual_holds"][0].__setitem__("line", d["visual_holds"][0]["line"] + 1))


@case("HOLD.04_6.source_hash")
def hold_source_hash_stale(b):
    def change(d):
        row = next(v for v in d["visual_holds"] if v["cue"] == "04_6")
        row["sha256"] = ZERO
    b.edit_json(b.map, change)


@case("HOLD.00_4.interval_hash")
def source_edited_inside_a_hold_interval(b):
    b.chapter("ratio_ruler.rs", "fade_in_all(stage, &[a4_ring, a4_word], 0.5)?;", "fade_in_all(stage, &[a4_ring, a4_word], 0.6)?;", canon=True)


@case("HOLD.census_equals_map")
def extra_predict(b):
    b.chapter("folded_decade.rs", SAY_03_2, SAY_03_2 + " predict(&kit, stage, v(0.0, 0.0), 1.0)?;", canon=True)


@case("HOLD.census_equals_map")
def extra_predict_across_lines(b):
    b.chapter("folded_decade.rs", SAY_03_2, SAY_03_2 + " predict(\n&kit,\nstage,\nv(0.0, 0.0),\n1.0,\n)?;", canon=True)


@case("HOLD.census_equals_map")
def second_predict_on_a_line_already_listed(b):
    b.chapter("ratio_ruler.rs", "predict(&kit, stage, v(3.6, -2.75), 3.0)?;", "predict(&kit, stage, v(3.6, -2.75), 3.0)?; predict(&kit, stage, v(0.0, 0.0), 1.0)?;", canon=True)


@case("HOLD.census_equals_map")
def extra_music_breath(b):
    b.chapter("folded_decade.rs", SAY_03_2, SAY_03_2 + " kit.music_breath(stage, 1.0)?;", canon=True)


@case("HOLD.every_other_wait_classified")
def extra_wait(b):
    b.chapter("folded_decade.rs", SAY_03_2, SAY_03_2 + " stage.wait(1.0)?;", canon=True)


@case("HOLD.no_scene_mark")
def scene_emits_its_own_mark(b):
    b.chapter("folded_decade.rs", SAY_03_2, SAY_03_2 + ' kit.mark(stage, "predict");', canon=True)


@case("HOLD.after_is_whole_cue", "CON.direction_contract")
def direction_after_shortened(b):
    b.edit_json(b.map, lambda d: d["overrides"][0].__setitem__("after", d["overrides"][0]["after"].rsplit(" ", 1)[0]))


@case("HOLD.census_equals_map", "HOLD.11_20.marker_line", "CON.declared_pairs")
def reading_hold_flipped_to_prediction(b):
    def change(d):
        next(v for v in d["visual_holds"] if v["cue"] == "11_20")["kind"] = "prediction"
    b.edit_json(b.map, change)


@case("HOLD.00_4.next_cue")
def hold_names_a_nonadjacent_cue(b):
    b.edit_json(b.map, lambda d: d["visual_holds"][0].__setitem__("next_cue", "00_6"))


@case("HOLD.stale_pairs_gone", "HOLD.census_equals_map")
def retired_pair_restored(b):
    def change(d):
        d["visual_holds"][0].update({"cue": "00_1", "next_cue": "00_2"})
    b.edit_json(b.map, change)


@case("group_consumers.completed", "STALE.every_named_path_and_hash")
def disposition_evidence_hash_wrong(b):
    path = b.plan / "direction-dispositions-proposed.json"
    b.edit_json(path, lambda d: d["direction_dispositions"][0]["evidence"].__setitem__("sha256", ZERO))


@case("CON.dispositions_all_native")
def disposition_proposes_inserted_silence(b):
    path = b.plan / "direction-dispositions-proposed.json"
    b.edit_json(path, lambda d: d["direction_dispositions"][0].__setitem__("kind", "authored_silence"))


@case("HOLD.predict_helper_order", "HOLD.helpers_and_consumers_unmoved")
def predict_helper_loses_its_mark(b):
    b.chapter("draw.rs", 'kit.mark(stage, "predict");', "/* gone */", canon=True)


@case("HOLD.11_20_rule_lines", "HOLD.11_20_window")
def theorem_text_shortened(b):
    b.sub(b.chapters / "unreachable.rs", "occur an even number of times.", "occur evenly.", count=2)


@case("HOLD.11_20_window")
def reading_rate_changed(b):
    text = (b.chapters / "unreachable.rs").read_text(encoding="utf-8")
    found = re.search(r"const READING_RATE: f64 = [0-9.]+;", text).group(0)
    b.chapter("unreachable.rs", found, "const READING_RATE: f64 = 30.0;")


# ------------------------------------------------------------------ phrase anchors
@case("PHR.11_1@320.destination")
def phrase_constant_edited(b):
    b.chapter("unreachable.rs", 'const HALFWAY: &str = "a halfway point";', 'const HALFWAY: &str = "a midway point";')


@case("PHR.census_equals_plan", "PHR.every_hook_on_the_cue_being_said")
def hook_moved_to_another_cue(b):
    b.chapter("doing_nothing.rs", 'until_phrase(&kit, stage, "10_4", "strictly between")?;', 'until_phrase(&kit, stage, "10_3", "strictly between")?;')


@case("PHR.census_equals_plan")
def hook_added(b):
    b.chapter("doing_nothing.rs", 'until_phrase(&kit, stage, "10_4", "strictly between")?;', 'until_phrase(&kit, stage, "10_4", "strictly between")?; until_phrase(&kit, stage, "10_4", "same rule")?;')


@case("REG.nothing_unreadable")
def hook_phrase_not_readable(b):
    b.chapter("doing_nothing.rs", 'until_phrase(&kit, stage, "10_4", "strictly between")?;', 'until_phrase(&kit, stage, "10_4", computed_phrase)?;')


@case("PHR.11_1@321.exact_byte", "PHR.11_1@321.template")
def anchor_byte_off_by_one(b):
    def change(d):
        row = next(r for r in d["anchors"] if r["cue"] == "11_1" and r["byte"] == 320)
        row["byte"] = 321
    b.edit_json(b.plan / "phrase-anchors.json", change)


@case("PHR.11_5@72.sentence_index")
def anchor_sentence_index_wrong(b):
    def change(d):
        next(r for r in d["anchors"] if r["cue"] == "11_5" and r["byte"] == 72)["sentence_index"] = 0
    b.edit_json(b.plan / "phrase-anchors.json", change)


@case("PHR.02_1@48.template")
def template_byte_differs(b):
    path = b.plan / "phrase-timings-unmeasured-template.json"
    b.edit_json(path, lambda d: d["cues"]["02_1"]["marks"][0].__setitem__("byte", 49))


@case("PHR.template_covers_all_cues", "PHR.template_text_hashes")
def template_loses_a_cue(b):
    path = b.plan / "phrase-timings-unmeasured-template.json"
    b.edit_json(path, lambda d: d["cues"].pop("07_3"))


@case("PHR.none_measured")
def anchor_claims_a_measurement(b):
    b.edit_json(b.plan / "phrase-anchors.json", lambda d: d["anchors"][0].__setitem__("measured_seconds", 1.5))


# ------------------------------------------------------------------ assembly and stale bindings
@case("ASM.retained.branch_cut.rs")
def retained_chapter_back_to_stage05(b):
    older = b.stage / "before-rebinding-source" / pf.REL / "src/chapters/branch_cut.rs"
    b.write(b.chapters / "branch_cut.rs", older.read_bytes(), rebind=False)


@case("ASM.replay.complex_turns.rs")
def candidate_is_not_the_patch(b):
    b.append("complex_turns.rs", "// a byte the reviewed patch does not hold\n")


@case("ASM.patch_is_the_reviewed_patch.complex_turns.rs")
def stage_patch_differs_from_reviewed(b):
    path = b.stage / "patches/05-complex_turns.patch"
    b.write(path, path.read_bytes() + b"\n", rebind=False)


@case("ASM.delta_is_exactly_the_repairs")
def undeclared_file_differs(b):
    path = b.crate / "src/facts.rs"
    b.write(path, path.read_bytes() + b"// undeclared\n")


@case("STALE.no_absolute_homes")
def plan_gains_an_absolute_home(b):
    b.edit_json(b.plan / "cue-delta.json", lambda d: d.__setitem__("claim", d["claim"] + " /" + "Users/someone/x"))


@case("STALE.no_older_stage_names")
def plan_names_an_older_stage(b):
    b.edit_json(b.plan / "hold-audit.json", lambda d: d.__setitem__("claim", d["claim"] + " combined-repair-stage05"))


@case("STALE.no_source_names_a_map")
def source_names_a_map(b):
    path = b.crate / "src/main.rs"
    b.write(path, path.read_bytes() + b"// narration-direction-map-oliver12.json\n")
    (b.canon / "src/main.rs").write_bytes(path.read_bytes())


@case("STALE.map13_not_in_canonical", "ASM.only_in_canonical")
def map13_half_installed(b):
    shutil.copyfile(b.map, b.canon / "docs" / pf.MAP_NAME)


@case("STALE.canonical_map12_matches_canonical")
def canonical_source_moved_under_map12(b):
    path = b.canon / "src/chapters/branch_cut.rs"
    path.write_bytes(path.read_bytes() + b"// moved\n")


DECOY = r'''
// kit.say(stage, "03_2")?; predict(&kit, stage, v(0.0, 0.0), 1.0)?; kit.music_breath(stage, 1.0)?; stage.wait(1.0)?;
/* kit.mark(stage, "predict"); until_phrase(&kit, stage, "03_1", "nothing")?; /* nested */ stage.wait(2.0)?; */
#[allow(dead_code)]
const DECOY: &str = "kit.say(stage, \"03_2\")?; predict(&kit, stage, v(0.0, 0.0), 1.0)?;";
#[allow(dead_code)]
const DECOY_RAW: &str = r#"stage.wait(1.0)?; kit.music_breath(stage, 1.0)?; kit.mark(stage, "predict");"#;
#[cfg(test)]
mod decoy_tests {
    #[test]
    fn decoy() {
        kit.say(stage, "03_2").unwrap();
        predict(&kit, stage, v(0.0, 0.0), 1.0).unwrap();
        stage.wait(1.0).unwrap();
        kit.music_breath(stage, 1.0).unwrap();
        kit.mark(stage, "predict");
        until_phrase(&kit, stage, "03_1", "nothing").unwrap();
    }
}
'''
DECOY_MUST_PASS = ["REG.every_cue_said_once_in_order", "REG.nothing_unreadable", "HOLD.census_equals_map", "HOLD.every_other_wait_classified", "HOLD.no_scene_mark", "PHR.census_equals_plan"]


def run_cli(root, *extra):
    run = subprocess.run([sys.executable, "-I", "-B", str(HERE / "preflight.py"), "--repo", str(root), "--quiet", *extra], capture_output=True, text=True)
    return run.returncode, re.findall(r"^FAIL (\S+)", run.stdout, re.M), run


def gate_files(box, tmp):
    """A narration folder and a filled timing file. Stand-in values: they exercise the rules, nothing else."""
    template = json.loads((box.plan / "phrase-timings-unmeasured-template.json").read_bytes())
    script = pf.read_script(box.crate / "script.tsv")
    timing = pf.fill_template(template, script)
    narration = tmp / "narration"
    narration.mkdir()
    outputs = {c: {"sha256": timing["cues"][c]["wav_sha256"], "frames": 24000 * 60, "sample_rate": 24000} for c in script}
    (narration / "assembly.json").write_text(json.dumps({"outputs": outputs}))
    return timing, narration


def main():
    failures, total = [], 0
    tmp = Path(tempfile.mkdtemp(prefix="stage06-tests-"))
    try:
        base = tmp / "base"
        count = sandbox.build_base(Path.cwd(), base)
        print(f"sandbox: {count} files copied; the real tree is only read")

        code, fails, run = run_cli(base)
        good = code == 0 and not fails
        total += 1
        print(f"{'pass' if good else 'MISS'} positive control, pins on: exit {code}; {run.stdout.strip().splitlines()[-1]}")
        if not good:
            failures.append("positive control, pins on")
            print(run.stdout[-2000:])

        code, fails, run = run_cli(base, "--no-pins")
        good = code == 0 and not fails
        total += 1
        print(f"{'pass' if good else 'MISS'} positive control, pins off: exit {code}")
        if not good:
            failures.append("positive control, pins off")

        for name, mutate, must_fail in CASES:
            total += 1
            box = Box(base, tmp / name)
            mutate(box)
            code, fails, run = run_cli(box.root, "--no-pins")
            missing = [m for m in must_fail if m not in fails]
            good = code == 1 and not missing
            print(f"{'pass' if good else 'MISS'} {name}: {len(fails)} FAIL; named {list(must_fail)}" + (f"; MISSING {missing}; got {fails[:10]}; {run.stderr[-300:]}" if not good else ""))
            if not good:
                failures.append(name)
            shutil.rmtree(box.root, ignore_errors=True)

        # Decoys in comments, strings and test modules are not calls.
        total += 1
        box = Box(base, tmp / "decoys")
        box.append("folded_decade.rs", DECOY, canon=True)
        code, fails, run = run_cli(box.root, "--no-pins")
        hit = [m for m in DECOY_MUST_PASS if m in fails]
        good = not hit and code == 0
        print(f"{'pass' if good else 'MISS'} decoys in comments, strings and a test module: exit {code}; {len(fails)} FAIL {fails[:6]}")
        if not good:
            failures.append("decoys")
        shutil.rmtree(box.root, ignore_errors=True)

        # A pin notices a change the semantic checks have no opinion on.
        total += 1
        box = Box(base, tmp / "pins")
        box.edit_json(box.plan / "cue-delta.json", lambda d: d.__setitem__("claim", d["claim"] + " x"))
        code, fails, run = run_cli(box.root)
        good = code == 1 and "PIN.plan.cue-delta.json" in fails
        print(f"{'pass' if good else 'MISS'} pins catch an unexpected plan byte: {fails[:4]}")
        if not good:
            failures.append("pins")
        shutil.rmtree(box.root, ignore_errors=True)

        # The measured phrase file: a silent fallback becomes a loud failure.
        box = Box(base, tmp / "gate")
        timing, narration = gate_files(box, tmp)
        anchors = json.loads((box.plan / "phrase-anchors.json").read_bytes())["anchors"]
        script = pf.read_script(box.crate / "script.tsv")

        def gate(label, change, want_fail):
            nonlocal total
            total += 1
            copy = json.loads(json.dumps(timing))
            change(copy)
            path = tmp / f"timing-{total}.json"
            path.write_text(json.dumps(copy))
            code, fails, run = run_cli(box.root, "--no-pins", "--phrase-timings", str(path), "--narration", str(narration))
            caught = "GATE.phrase_timings" in fails
            good = caught == want_fail and code == (1 if want_fail else 0)
            print(f"{'pass' if good else 'MISS'} phrase gate, {label}: exit {code}")
            if not good:
                failures.append(f"gate {label}")
                print(run.stdout[-800:])

        def drop_mark(d):
            entry = d["cues"]["11_7"]
            entry["marks"] = [m for m in entry["marks"] if m["byte"] != 226]

        def extra_mark_only(d):
            # The kit would accept this file: every rule holds, yet an anchor's byte has no mark.
            entry = d["cues"]["11_20"]
            entry["marks"] = [{"byte": 0, "seconds": 0.001}]

        gate("filled, every anchor marked", lambda d: None, False)
        gate("one anchor without a mark", drop_mark, True)
        gate("a file the kit would load, missing two anchors", extra_mark_only, True)
        gate("text hash differs", lambda d: d["cues"]["00_1"].__setitem__("text_sha256", ZERO), True)
        gate("WAV hash differs", lambda d: d["cues"]["00_1"].__setitem__("wav_sha256", ZERO), True)
        gate("a cue missing", lambda d: d["cues"].pop("07_3"), True)
        gate("mark past the end of its take", lambda d: d["cues"]["02_1"]["marks"][0].__setitem__("seconds", 61.0), True)
        gate("marks out of time order", lambda d: d["cues"]["10_3"]["marks"][1].__setitem__("seconds", 0.0), True)
        gate("marks out of byte order", lambda d: d["cues"]["10_3"].__setitem__("marks", list(reversed(d["cues"]["10_3"]["marks"]))), True)
        gate("byte past the end of the text", lambda d: d["cues"]["02_1"]["marks"][0].__setitem__("byte", 99999), True)
        total += 1
        path = box.plan / "phrase-timings-unmeasured-template.json"
        code, fails, run = run_cli(box.root, "--no-pins", "--phrase-timings", str(path), "--narration", str(narration))
        good = code == 1 and "GATE.phrase_timings" in fails
        print(f"{'pass' if good else 'MISS'} phrase gate, the unfilled template: exit {code}")
        if not good:
            failures.append("gate template")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print(f"{total - len(failures)} of {total} cases behaved; {len(failures)} did not: {failures}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
