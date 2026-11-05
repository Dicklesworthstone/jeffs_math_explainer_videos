"""Tests for rebind_audio.py. Every run works on a scratch copy of the final source; root's stage is only read.

From the repository root, where neither SCRATCH nor RESULTS exists:

    python3 -I -B <this file> SCRATCH RESULTS

The rehearsal uses root's approved final source and integration.json, with paths rewritten to the copy.
Nothing here compiles, builds, renders, assembles, synthesizes or decodes.
"""
import copy
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
HELPER = HERE / "rebind_audio.py"
spec = importlib.util.spec_from_file_location("rebind_audio", HELPER)
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)
REL = h.REL
CH = f"{REL}/src/chapters"
REAL = h.STAGE
INITIAL = json.loads((REAL / "initial-copy.json").read_text())
WS = Path(INITIAL["source"])
OUTPUT_NAMES = (h.AUDIT_NAME, h.SPANS_NAME, h.BEFORE_UNITS_NAME, h.REPORT_NAME)
results = []


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fresh(stage):
    """A stage with root's final source and integration.json, the unit plan as it was before any rebind."""
    (stage / "workspace").mkdir(parents=True)
    shutil.copy2(REAL / "initial-copy.json", stage / "initial-copy.json")
    for row in INITIAL["files"]:
        target = stage / "workspace" / row["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2((WS if row["path"] == h.UNITS else REAL / "workspace") / row["path"], target)
    data = json.loads((REAL / "integration.json").read_text())
    data["source_workspace"] = str(stage / "workspace")
    for row in data["files"] + data["all_pre_rebind_files"]:
        row["path"] = str(stage / "workspace" / row["relative_path"])
    save(stage, data)
    return stage


def load(stage):
    return json.loads((stage / "integration.json").read_text())


def save(stage, data):
    (stage / "integration.json").write_text(json.dumps(data, indent=2) + "\n")


def approve(stage, rel):
    """Root's approval of whatever the file now holds, so a later check is what refuses."""
    data = load(stage)
    digest = sha((stage / "workspace" / rel).read_bytes())
    for row in data["files"] + data["all_pre_rebind_files"]:
        if row["relative_path"] == rel:
            row["sha256"] = digest
    original = next(r["sha256"] for r in INITIAL["files"] if r["path"] == rel)
    if digest != original and rel not in data["changed_files"]:
        data["changed_files"].append(rel)
    save(stage, data)


def edit(stage, rel, old, new):
    path = stage / "workspace" / rel
    text = path.read_text(encoding="utf-8")
    assert old in text, (rel, old)
    path.write_text(text.replace(old, new, 1), encoding="utf-8")
    approve(stage, rel)


def snapshot(stage):
    return {p.relative_to(stage).as_posix(): sha(p.read_bytes()) for p in stage.rglob("*") if p.is_file()}


def run(stage, flag, *extra):
    done = subprocess.run([sys.executable, "-I", "-B", str(HELPER), flag, "--stage", str(stage), *extra], capture_output=True, text=True)
    return done.returncode, done.stdout, done.stderr


def record(name, ok, detail=""):
    results.append({"name": name, "pass": bool(ok), "detail": str(detail).replace(str(ROOT), "<scratch>")[:300]})


def refused(name, setup, fragment, flag="--apply", extra=()):
    """The helper must refuse, say why, and leave the stage exactly as it found it."""
    stage = fresh(ROOT / "cases" / name / "stage")
    if setup:
        setup(stage)
    before = snapshot(stage)
    code, _, err = run(stage, flag, *extra)
    record(name, code == 1 and fragment in err and snapshot(stage) == before, f"exit {code}: {err.strip()}")


def set_key(key, value):
    def setup(stage):
        data = load(stage)
        data[key] = value
        save(stage, data)
    return setup


def set_closing(key, value):
    def setup(stage):
        data = load(stage)
        data["editorial_closing_hold"][key] = value
        save(stage, data)
    return setup


def drop_key(key):
    def setup(stage):
        data = load(stage)
        del data[key]
        save(stage, data)
    return setup


def gate_cases():
    refused("gate_integration_absent", lambda s: os.replace(s / "integration.json", s / "aside.json"), "integration.json is absent")
    refused("gate_not_approved", set_key("approved_for_guarded_build", False), "does not approve")
    refused("gate_other_script", set_key("script_sha256", "0" * 64), "different script")
    refused("gate_audio_changed", set_key("audio_changed", True), "audio is unchanged")
    refused("gate_other_workspace", set_key("source_workspace", "elsewhere/workspace"), "another workspace")

    def short(stage):
        data = load(stage)
        data["all_pre_rebind_files"].pop()
        save(stage, data)

    def before(stage):
        data = load(stage)
        data["all_pre_rebind_files"][0]["before_sha256"] = "0" * 64
        save(stage, data)

    def unlisted(stage):
        data = load(stage)
        data["changed_files"].pop()
        save(stage, data)

    def files_row(stage):
        data = load(stage)
        data["files"][0]["sha256"] = "0" * 64
        save(stage, data)

    def missing(stage):
        aside = stage.parent / "aside"
        aside.mkdir()
        os.replace(stage / "workspace" / REL / "oracle.py", aside / "oracle.py")

    refused("gate_short_file_list", short, "not 51 rows")
    refused("gate_wrong_before_hash", before, "before-hash")
    refused("gate_changed_file_unlisted", unlisted, "changed_files differs")
    refused("gate_files_row_differs", files_row, "files row differs")
    refused("gate_unapproved_edit", lambda s: (s / "workspace" / f"{CH}/draw.rs").open("a").write("\n"), "differs from the approved hash")
    refused("gate_extra_file", lambda s: (s / "workspace" / f"{CH}/extra.rs").write_text("\n"), "outside the 51")
    refused("gate_missing_file", missing, "are missing")
    for name, rel in (
        ("script", h.SCRIPT), ("units", h.UNITS), ("tool", h.TOOL), ("cargo_toml", f"{REL}/Cargo.toml"),
        ("cargo_lock", f"{REL}/Cargo.lock"), ("map16", h.MAP16),
    ):
        def frozen_edit(stage, rel=rel):
            path = stage / "workspace" / rel
            path.write_bytes(path.read_bytes() + b"\n")
            approve(stage, rel)
        refused(f"gate_frozen_{name}", frozen_edit, "must not change")


def source_cases():
    r = f"{CH}/ratio_ruler.rs"
    b = f"{CH}/building_ruler.rs"
    s = f"{CH}/slide_rule.rs"
    w = f"{CH}/what_it_forgets.rs"
    d = f"{CH}/draw.rs"
    pred = "predict(&kit, stage, v(3.6, -2.75), 3.0)?;"
    slide = "predict(&kit, stage, v(4.7, -2.75), 3.0)?;"
    say = lambda cue: f'kit.say(stage, "{cue}")?;'
    closing = "kit.music_breath(stage, CLOSING_HOLD)?;"

    def moved(stage):
        edit(stage, s, slide, "// moved")
        edit(stage, s, say("02_3"), say("02_3") + "\n        " + slide)

    refused("hold_predict_removed", lambda t: edit(t, r, pred, "// gone"), "predict calls moved")
    refused("hold_predict_extra", lambda t: edit(t, b, say("01_1"), say("01_1") + "\n        predict(&kit, stage, v(0.0, 0.0), 3.0)?;"), "predict calls moved")
    refused("hold_predict_moved_past_next_say", moved, "predict calls moved")
    refused("hold_predict_multi_line", lambda t: edit(t, r, pred, "predict(\n            &kit,\n            stage,\n            v(3.6, -2.75),\n            3.0,\n        )?;"), "whole one-line call")
    refused("hold_code_after_predict", lambda t: edit(t, r, pred, pred + "\n        stage.wait(1.0)?;"), "code between predict")
    refused("hold_breath_mid_film", lambda t: edit(t, b, say("01_2"), say("01_2") + "\n        kit.music_breath(stage, 1.0)?;"), "undocumented kit.music_breath")
    refused("hold_reading_marker_changed", lambda t: edit(t, f"{CH}/doing_nothing.rs", "FINE_PRINT_S - read_for)?;", "FINE_PRINT_S - read_for + 0.0)?;"), "reading marker changed")

    def rate(stage):
        path = stage / "workspace" / f"{CH}/unreachable.rs"
        text, count = re.subn(r"^const READING_RATE: f64 = [0-9.]+;", "const READING_RATE: f64 = 12.0;", path.read_text(), flags=re.M)
        assert count == 1
        path.write_text(text)
        approve(stage, f"{CH}/unreachable.rs")

    refused("hold_reading_literals_changed", rate, "literals behind the reading basis")
    refused("say_removed", lambda t: edit(t, b, say("01_2"), "// gone"), "do not say every script cue")
    refused("say_duplicated", lambda t: edit(t, b, say("01_2"), say("01_2") + "\n        " + say("01_2")), "do not say every script cue")
    refused("helper_think_changed", lambda t: edit(t, d, "pub const THINK: f64 = 4.0;", "pub const THINK: f64 = 3.0;"), "helper marker")
    refused("helper_predict_order", lambda t: edit(t, d, 'kit.hold(stage)?;\n    kit.mark(stage, "predict");', 'kit.mark(stage, "predict");\n    kit.hold(stage)?;'), "in that order")

    def fade(stage):
        path = stage / "workspace" / d
        text, count = re.subn(r"(fade_out_all\(stage, &\[bar, [^\]]*probe\], )0\.25\)", r"\g<1>0.5)", path.read_text())
        assert count == 1
        path.write_text(text)
        approve(stage, d)

    refused("helper_bar_fade_changed", fade, "helper marker")
    refused("closing_undocumented", set_key("editorial_closing_hold", None), "not documented")
    refused("closing_key_missing", drop_key("editorial_closing_hold"), "no editorial_closing_hold key")
    refused("closing_wrong_cue", set_closing("cue", "12_20"), "wrong cue")
    refused("closing_wrong_file", set_closing("source", f"{CH}/unreachable.rs"), "wrong file")
    refused("closing_wrong_marker", set_closing("marker", "kit.music_breath(stage, OTHER)?;"), "different marker")
    refused("closing_wrong_hold_seconds", set_closing("hold_seconds", 5.0), "hold_seconds")
    refused("closing_wrong_fade_seconds", set_closing("fade_seconds", 1.0), "fade_seconds")
    refused("closing_claims_map_row", set_closing("prediction_or_reading_direction_row", True), "claims a map row")
    refused("closing_claims_timing", set_closing("native_timing_verified", True), "claims timing")
    refused("closing_before_recording_end", set_closing("after_complete_recording", False), "complete recording")
    refused("closing_absent_from_source", lambda t: edit(t, w, closing, "// gone"), "found 0")
    refused("closing_second_breath", lambda t: edit(t, w, closing, closing + "\n        " + closing), "found 2")
    refused("closing_extra_step", lambda t: edit(t, w, "fade_all(stage, CLOSING_FADE)?;", "stage.wait(1.0)?;\n        fade_all(stage, CLOSING_FADE)?;"), "only one fade")

    def original(stage):
        copy_of = stage.parent / "original"
        for row in INITIAL["files"]:
            (copy_of / row["path"]).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(WS / row["path"], copy_of / row["path"])
        with (copy_of / f"{CH}/draw.rs").open("a") as handle:
            handle.write("\n")

    stage = ROOT / "cases/original_tampered/stage"
    fresh(stage)
    original(stage)
    before = snapshot(stage)
    code, _, err = run(stage, "--apply", "--original", str(stage.parent / "original"))
    record("original_tampered", code == 1 and "does not match initial-copy" in err and snapshot(stage) == before, f"exit {code}: {err.strip()}")


def recheck_map_from_source(stage, map17, map16):
    """Independent of the helper's census: each row's line, say and interval, from the file's own text."""
    problems = []
    fixed = ("video", "cue", "next_cue", "kind", "path", "basis", "reason")
    for new, old in zip(map17["visual_holds"], map16["visual_holds"]):
        if any(new[k] != old[k] for k in fixed):
            problems.append((new["cue"], "fixed field changed"))
        data = (stage / "workspace" / new["path"]).read_bytes()
        lines = data.decode().split("\n")
        say = [n for n, line in enumerate(lines, 1) if line.strip() == f'kit.say(stage, "{new["cue"]}")?;']
        interval = "\n".join(lines[say[0] - 1 : new["line"]]) if len(say) == 1 else ""
        if (len(say) != 1 or lines[new["line"] - 1].strip() != new["marker"] or sha(data) != new["sha256"]
                or new["evidence"] != {"path": new["path"], "sha256": new["sha256"]} or sha(interval.encode()) != new["interval_sha256"]):
            problems.append((new["cue"], "row does not match its file"))
    for row in map17["shared_helpers"]:
        if row["path"].startswith(REL) and sha((stage / "workspace" / row["path"]).read_bytes()) != row["sha256"]:
            problems.append((row["path"], "shared helper hash"))
    return problems


def rehearsal():
    stage = fresh(ROOT / "rehearsal/stage")
    before = snapshot(stage)
    code, out, err = run(stage, "--check")
    predicted = json.loads(out) if code == 0 else {}
    record("check_passes_and_writes_nothing", code == 0 and snapshot(stage) == before, f"exit {code} {err.strip()}")
    code, _, err = run(stage, "--verify")
    record("verify_before_apply_refused", code == 1 and "finished --apply" in err and snapshot(stage) == before, err.strip())
    code, out, err = run(stage, "--apply")
    applied = json.loads(out) if code == 0 else {}
    record("apply_passes", code == 0, f"exit {code} {err.strip()}")
    after = snapshot(stage)
    touched = {k for k in after if after[k] != before.get(k)} | (set(before) - set(after))
    record("apply_touches_only_the_named_files", touched == {f"workspace/{h.MAP17}", f"workspace/{h.UNITS}", *OUTPUT_NAMES}, sorted(touched))
    record("apply_leaves_integration_immutable", after["integration.json"] == before["integration.json"])
    w = stage / "workspace"
    map16_raw = (w / h.MAP16).read_bytes()
    map17 = json.loads((w / h.MAP17).read_text())
    map16 = json.loads(map16_raw)
    units = json.loads((w / h.UNITS).read_text())
    units_old = json.loads((WS / h.UNITS).read_text())
    spans = json.loads((stage / h.SPANS_NAME).read_text())
    spans_old = json.loads((h.PREPARED / "spans.json").read_text())
    audit_raw = (stage / h.AUDIT_NAME).read_bytes()
    audit = json.loads(audit_raw)

    record("apply_equals_prediction", all(predicted["outputs_sha256"][k] == applied["outputs_sha256"][k] for k in predicted["outputs_sha256"]))
    record("map16_and_script_unchanged", sha(map16_raw) == h.PINS["map16"] and sha((w / h.SCRIPT).read_bytes()) == h.PINS["script"])
    record("map17_keeps_overrides_kinds_scripts", all(map17[k] == map16[k] for k in ("overrides", "scripts", "hold_kinds", "schema_version")))
    record("map17_names_predecessor", map17["predecessor"] == {"path": h.MAP16, "sha256": h.PINS["map16"]})
    problems = recheck_map_from_source(stage, map17, map16)
    record("map17_rows_match_source_by_independent_reading", not problems, problems[:3])
    changed_rows = [r["cue"] for r in audit["holds"] if r["changed"]]
    record("map17_is_a_real_rebind", len({r["sha256"] for r in map17["visual_holds"]} - {r["sha256"] for r in map16["visual_holds"]}) > 0 and changed_rows, changed_rows)

    a = copy.deepcopy(units)
    b = copy.deepcopy(units_old)
    bound = a["direction_contract"].pop("map")
    b["direction_contract"].pop("map")
    record("units_change_only_the_map_binding", a == b and bound == {"path": h.MAP17, "sha256": sha((w / h.MAP17).read_bytes())})
    record("before_units_preserved", sha((stage / h.BEFORE_UNITS_NAME).read_bytes()) == h.PINS["units"])

    record("spans_keep_units_and_boundary_review", spans["units"] == spans_old["units"] and spans["boundary_review"] == spans_old["boundary_review"] and len(spans["units"]) == 80)
    record("spans_bind_new_units", spans["units_sha256"] == sha((w / h.UNITS).read_bytes()))
    new_d, old_d = spans["direction_dispositions"], spans_old["direction_dispositions"]
    shape = all((x["override_index"], x["cue"], x["kind"]) == (y["override_index"], y["cue"], y["kind"]) for x, y in zip(new_d, old_d))
    strip = lambda text: re.sub(r"\.rs:\d+", ".rs:N", text)
    record("dispositions_differ_only_in_line_and_evidence", len(new_d) == 15 and shape and all(strip(x["reason"]) == strip(y["reason"]) for x, y in zip(new_d, old_d)))
    lines = {r["cue"]: r["line"] for r in map17["visual_holds"]}
    record("dispositions_cite_map17_lines", all(x["reason"].split(".rs:")[1].split(" ")[0] == str(lines[x["cue"]]) for x in new_d))
    record("dispositions_bind_the_audit", all(x["evidence"]["sha256"] == sha(audit_raw) and Path(x["evidence"]["path"]).resolve() == (stage / h.AUDIT_NAME).resolve() for x in new_d))

    closing = audit["editorial_closing_hold"]
    record("closing_hold_is_separate", closing["cue"] == "12_21" and closing["cue"] not in {r["cue"] for r in map17["visual_holds"]} and closing["cue"] not in {d["cue"] for d in new_d} and len(audit["holds"]) == 15)
    record("no_wav_written", not [p for p in stage.rglob("*") if p.suffix.lower() in (".wav", ".mp3", ".mp4")])
    done = subprocess.run([sys.executable, "-I", "-B", str(w / h.TOOL), "check", "--script", str(w / h.SCRIPT), "--units", str(w / h.UNITS), "--require-directions"], capture_output=True, text=True)
    record("copied_tool_check_cli", done.returncode == 0 and json.loads(done.stdout) == {"cues": 80, "units": 80}, done.stderr)

    code, _, err = run(stage, "--verify")
    record("verify_passes", code == 0, err.strip())
    code, _, err = run(stage, "--apply")
    record("second_apply_refused", code == 1, err.strip())
    code, _, err = run(stage, "--check")
    record("check_after_apply_refused", code == 1, err.strip())

    for name, path, fragment in (
        ("verify_catches_source_edit", w / f"{CH}/draw.rs", "differs from the approved hash"),
        ("verify_catches_spans_edit", stage / h.SPANS_NAME, "differs from what"),
        ("verify_catches_map17_edit", w / h.MAP17, "differs from what"),
        ("verify_catches_units_edit", w / h.UNITS, "differs from what"),
        ("verify_catches_audit_edit", stage / h.AUDIT_NAME, "differs from what"),
        ("verify_catches_preserved_units_edit", stage / h.BEFORE_UNITS_NAME, "preserved unit plan"),
    ):
        saved = path.read_bytes()
        path.write_bytes(saved + b"\n")
        code, _, err = run(stage, "--verify")
        path.write_bytes(saved)
        record(name, code == 1 and fragment in err, err.strip())
    code, _, err = run(stage, "--verify")
    record("verify_passes_after_restoring", code == 0, err.strip())
    return stage, applied, audit


def controls(stage):
    """Negative controls: the checks after the tool call must be able to fail."""
    o = h.load_originals(stage, None)
    w = stage / "workspace"
    tool = h.load_tool(w)
    good = json.loads((stage / h.SPANS_NAME).read_text())
    script, units_path = w / h.SCRIPT, w / h.UNITS

    def expect(name, spans, fragment, sources=None):
        try:
            h.tool_verify(tool, script, units_path, spans, sources or o)
            record(name, False, "no refusal")
        except (h.Refused, ValueError, KeyError) as error:
            record(name, fragment in str(error), error)

    spans = copy.deepcopy(good)
    spans["units"][3]["gap_after_samples"] = [g + 1 for g in spans["units"][3]["gap_after_samples"]]
    expect("control_changed_gap", spans, "outside metadata")
    spans = copy.deepcopy(good)
    spans["boundary_review"]["reviewer"] = "someone else"
    expect("control_changed_review", spans, "outside metadata")
    spans = copy.deepcopy(good)
    spans["direction_dispositions"][0]["evidence"]["sha256"] = "0" * 64
    expect("control_wrong_evidence_hash", spans, "evidence changed")
    spans = copy.deepcopy(good)
    spans["direction_dispositions"].pop()
    expect("control_missing_disposition", spans, "explicit assembly disposition")
    spans = copy.deepcopy(good)
    spans["direction_dispositions"][0]["reason"] = " "
    expect("control_empty_reason", spans, "concrete action")
    reference = copy.deepcopy(o)
    cue = next(iter(reference["reference"]["lines"]))
    reference["reference"]["lines"][cue][0]["out_samples"] += 1
    expect("control_reference_row_differs", good, "sentence rows differ", reference)
    real_pcm = tool.wav_pcm
    tool.wav_pcm = lambda path: (real_pcm(path)[0], b"\0\0") if str(path).endswith("prepared02/narration/04_7.wav") else real_pcm(path)
    expect("control_samples_differ", good, "04_7: samples differ")
    tool.wav_pcm = real_pcm


def determinism(stage, applied):
    os.replace(stage, stage.with_name("stage-first"))
    again = fresh(stage)
    code, out, err = run(again, "--apply")
    names = [f"workspace/{h.MAP17}", f"workspace/{h.UNITS}", *OUTPUT_NAMES]
    first = stage.with_name("stage-first")
    equal = code == 0 and all((first / n).read_bytes() == (again / n).read_bytes() for n in names)
    record("second_rehearsal_byte_equal", equal, err.strip())


def no_integration():
    stage = ROOT / "gate/stage"
    (stage / "workspace").mkdir(parents=True)
    shutil.copy2(REAL / "initial-copy.json", stage / "initial-copy.json")
    for row in INITIAL["files"]:
        target = stage / "workspace" / row["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(WS / row["path"], target)
    before = snapshot(stage)
    code, out, _ = run(stage, "--check")
    record("originals_check_without_integration", code == 3 and json.loads(out)["changed_files"] == [] and snapshot(stage) == before, f"exit {code}")
    code, _, err = run(stage, "--apply")
    record("apply_without_integration_writes_nothing", code == 1 and "absent" in err and snapshot(stage) == before, err.strip())


def main():
    global ROOT
    ROOT = Path(sys.argv[1]).resolve()
    out = Path(sys.argv[2])
    assert not ROOT.exists() and not out.exists()
    assert (Path.cwd() / h.PREPARED).is_dir(), "run from the repository root"
    ROOT.mkdir(parents=True)
    no_integration()
    gate_cases()
    source_cases()
    stage, applied, audit = rehearsal()
    controls(stage)
    determinism(stage, applied)
    failed = [r for r in results if not r["pass"]]
    summary = {
        "counts": {"tests": len(results), "passed": len(results) - len(failed), "failed": len(failed)},
        "rehearsal": {
            "source": "root's approved final source and integration.json, copied; paths rewritten to the copy",
            "verification": applied["verification"],
            "holds_changed": [
                {"cue": r["cue"], "changed": r["changed"], "before": r["before"]["marker"], "after": r["after"]["marker"]}
                for r in audit["holds"] if r["changed"]
            ],
            "map17_sha256": applied["outputs_sha256"][h.MAP17],
            "units_sha256": applied["outputs_sha256"][h.UNITS],
        },
        "results": results,
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, "x") as handle:
        handle.write(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary["counts"]))
    for row in failed:
        print("FAIL", row["name"], row["detail"])
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
