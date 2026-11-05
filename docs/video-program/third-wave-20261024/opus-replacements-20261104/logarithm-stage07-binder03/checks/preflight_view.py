#!/usr/bin/env python3
"""RubyRaven's stage06 preflight, with preflight-stage07.patch applied in memory, on this plan.

    python3 -I -B <this file> <preflight.py> --print-patched            print the patched text only
    python3 -I -B <this file> <preflight.py> [--without direct|requests]   from a mirror tree only

RubyRaven's file is read, never written. The five replaced lines are the delivered patch:

  direct    :734  the expected literal phrase_marks readers gain ("folded_decade.rs", "03_3")
  requests  :383  read narration-request-final80-02/requests.json, root's list for this script
            :576  cue-delta and the list name the same older request, except 01_5, which must be null
            :579  the older request's text equals the final cue, except 01_5, and only 01_5
            :824  read each cue's request file from the row's text_path

Two module constants are also set in memory, as binder01 did: where the stage is, and the map's
name. The mirror holds the candidate's crate, this plan at <candidate>/bound-plan, and stage06's
oracle table beside them, because that is where the preflight looks.

Not run, and not rebased: its pins and its assembly group, which name stage06's own repairs and bytes.

Each FAIL is compared with a list below, each with the reason it is expected. A FAIL outside the
list, or an expected FAIL that did not happen, makes this exit 1. --without leaves one group of
replacements out, as a control: the checks those replacements answer must then fail.

Text and plan checks only: nothing compiled, rendered, measured or heard.
"""
import hashlib
import sys
import types
from pathlib import Path

sys.dont_write_bytecode = True

LANE = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
STAGE = f"{LANE}/combined-repair-stage07/candidate"
MAP = "narration-direction-map-oliver15.json"
PREFLIGHT_SHA256 = "056d58d5acd118d71415b83fa7f0e3dd351d3589173b72de28713da8a2a42307"
REPLACEMENTS = (
    (
        "requests",
        'self.requests = json.loads((self.repo / LANE / "narration-request-final80-01/requests.json").read_bytes())',
        'self.requests = json.loads((self.repo / LANE / "narration-request-final80-02/requests.json").read_bytes())',
    ),
    (
        "requests",
        'self.check("TXT.delta_matches_requests", all(prior[d["cue"]] == d["request_id_with_exact_bytes"] for d in delta["cues"]), "cue-delta and requests.json name the same older request")',
        'self.check("TXT.delta_matches_requests", all(d["request_id_with_exact_bytes"] is None if d["cue"] == "01_5" else prior[d["cue"]] == d["request_id_with_exact_bytes"] for d in delta["cues"]), "cue-delta and requests.json name the same older request; no older request holds 01_5\'s new wording")',
    ),
    (
        "requests",
        'self.check("TXT.older_request_bytes_same", not wrong, f"each older request id\'s text is byte-equal to its final cue; different: {wrong[:5]}")',
        'self.check("TXT.older_request_bytes_same", wrong == ["01_5"], f"each older request id\'s text is byte-equal to its final cue, except reworded 01_5; different: {wrong[:5]}")',
    ),
    (
        "direct",
        'self.check("PHR.direct_mark_readers", direct == {("undoing_growth.rs", "04_2")}, f"{sorted(direct)}")',
        'self.check("PHR.direct_mark_readers", direct == {("undoing_growth.rs", "04_2"), ("folded_decade.rs", "03_3")}, f"{sorted(direct)}")',
    ),
    (
        "requests",
        'data = (self.repo / LANE / "narration-request-final80-01" / f"{u[\'line_ids\'][0]}.txt").read_bytes()',
        'data = (self.repo / next(r["text_path"] for r in self.requests["requests"] if r["cue"] == u["line_ids"][0])).read_bytes()',
    ),
)
EXPECTED_ALWAYS = {
    "STALE.canonical_still_binds_map12": "Canonical has moved on: it was integrated from stage06 and now binds oliver13.",
    "STALE.canonical_map12_matches_canonical": "Same: canonical's chapters no longer match oliver12, by root's integration.",
}
EXPECTED_WITHOUT = {
    "direct": {
        "PHR.direct_mark_readers": "Control: the old expected set does not know folded_decade.rs reads 03_3's mark.",
    },
    "requests": {
        "TXT.requests_bind_script": "Control: narration-request-final80-01 binds stage06's script f201d2dc.",
        "TXT.request_files_exact": "Control: its 01_5 file holds the old wording.",
        "TXT.requests_script_bytes": "Control: the script file it names is stage06's.",
        "TXT.delta_matches_requests": "Control: 01_5's request_id_with_exact_bytes is null, and the old check allows no exception.",
        "TXT.older_request_bytes_same": "Control: 01_5's older request holds the old wording, and the old check allows no exception.",
        "CON.request_text": "Control: its 01_5 file is not what unit_spans would demand.",
    },
}


def patched(original, leave_out=None):
    if hashlib.sha256(original.encode("utf-8")).hexdigest() != PREFLIGHT_SHA256:
        sys.exit("refused: preflight.py is not the bytes this patch was written against")
    text = original
    for group, old, new in REPLACEMENTS:
        if text.count(old) != 1:
            sys.exit(f"refused: expected one occurrence of: {old[:70]}")
        if group != leave_out:
            text = text.replace(old, new)
    return text


def main():
    path = Path(sys.argv[1])
    original = path.read_text(encoding="utf-8")
    if "--print-patched" in sys.argv:
        sys.stdout.write(patched(original))
        return 0
    if not Path("mirror-tree.marker").is_file():
        sys.exit("refused: this runs only in a mirror tree")
    leave_out = sys.argv[sys.argv.index("--without") + 1] if "--without" in sys.argv else None
    expected = {**EXPECTED_ALWAYS, **EXPECTED_WITHOUT.get(leave_out, {})}
    module = types.ModuleType("preflight_under_view")
    module.__file__ = str(path)
    sys.modules[module.__name__] = module
    exec(compile(patched(original, leave_out), str(path), "exec"), module.__dict__)
    module.STAGE = Path(STAGE)
    module.MAP_NAME = MAP
    run = module.Run(".", "stage", pins=False, quiet=True)
    run.load()
    groups = (run.group_inputs, run.group_text, run.group_registry, run.group_holds, run.group_phrases, run.group_consumers, run.group_stale)
    for group in groups:
        try:
            group()
        except Exception as error:
            run.check(f"{group.__name__}.completed", False, f"{type(error).__name__}: {error}")
    fails = set(run.fail)
    surprise = sorted(fails - set(expected))
    missing = sorted(set(expected) - fails)
    for cid in sorted(fails & set(expected)):
        print(f"expected FAIL {cid}: {expected[cid]}")
    print(f"preflight view{' without ' + leave_out if leave_out else ''}: {run.ok} ok, {len(fails)} FAIL ({len(fails & set(expected))} expected), {len(run.warn)} WARN {sorted(run.warn)}")
    print(f"unexpected FAIL: {surprise}")
    print(f"expected FAIL that did not happen: {missing}")
    return 1 if surprise or missing else 0


if __name__ == "__main__":
    sys.exit(main())
