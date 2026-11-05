#!/usr/bin/env python3
"""RubyRaven's stage06 preflight, run unmodified on the proposed stage07 plan, in a mirror tree.

    python3 -I -B <this file> <preflight.py> [--derived-requests]       from a mirror tree only

The preflight has its own lexer and does not import the binder, so it can disagree with it. Only
two of its module constants are replaced in memory: where the stage is, and the map's name. The
mirror holds the candidate's crate, this plan at <candidate>/bound-plan, and stage06's oracle
table beside them, because that is where the preflight looks.

Not run: its pins and its assembly group, which name stage06's own repairs and bytes.

Each FAIL is compared with a list below, each with the reason it is expected. A FAIL outside the
list, or an expected FAIL that did not happen, makes this exit 1.

--derived-requests is the control on the six request FAILs. It rewrites the MIRROR's copy of
narration-request-final80-01/requests.json to bind the new script, with 01_5 taken from
narration-request-rounding01. If 01_5 is the whole of the request gap, all six must go away.
Root's own file is not touched, and this is not a proposed request list.

Text and plan checks only: nothing compiled, rendered, measured or heard.
"""
import hashlib
import importlib.util
import json
import shutil
import sys
from pathlib import Path

sys.dont_write_bytecode = True

LANE = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
STAGE = f"{LANE}/combined-repair-stage07/candidate"
MAP = "narration-direction-map-oliver14.json"
SCRIPT = "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e"
WHY_REQUESTS = "narration-request-final80-01/requests.json still binds stage06's script f201d2dc; 01_5's new text is only in narration-request-rounding01."
EXPECTED_ALWAYS = {
    "STALE.canonical_still_binds_map12": "Canonical has moved on: it was integrated from stage06 and now binds oliver13.",
    "STALE.canonical_map12_matches_canonical": "Same: canonical's chapters no longer match oliver12, by root's integration.",
}
EXPECTED_REQUESTS = {
    "TXT.requests_bind_script": WHY_REQUESTS,
    "TXT.request_files_exact": WHY_REQUESTS,
    "TXT.requests_script_bytes": WHY_REQUESTS,
    "TXT.delta_matches_requests": "01_5's request_id_with_exact_bytes is deliberately null: no older request holds its new text.",
    "TXT.older_request_bytes_same": "01_5's older request holds the old wording.",
    "CON.request_text": WHY_REQUESTS,
}


def derive_requests():
    path = Path(LANE) / "narration-request-final80-01/requests.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    rounding = Path(LANE) / "narration-request-rounding01/01_5.txt"
    digest = hashlib.sha256(rounding.read_bytes()).hexdigest()
    data["script_sha256"] = SCRIPT
    data["script_path"] = f"{STAGE}/explainer_corpus/logarithm_mysteries/script.tsv"
    row = next(r for r in data["requests"] if r["cue"] == "01_5")
    row.update(text_path=str(rounding), text_sha256=digest, file_sha256=digest, prior_request_id=None, prior_text_path=str(rounding))
    path.write_text(json.dumps(data), encoding="utf-8")
    shutil.copyfile(rounding, path.parent / "01_5.txt")


def main():
    if not Path("mirror-tree.marker").is_file():
        sys.exit("refused: this runs only in a mirror tree")
    derived = "--derived-requests" in sys.argv
    if derived:
        derive_requests()
    moved = {"INPUT.plan_inputs_unmoved": "This control rewrites the mirror's request list and adds 01_5's file there, so those two inputs have moved by construction."}
    expected = {**EXPECTED_ALWAYS, **moved} if derived else {**EXPECTED_ALWAYS, **EXPECTED_REQUESTS}
    spec = importlib.util.spec_from_file_location("preflight_under_view", sys.argv[1])
    pre = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = pre
    spec.loader.exec_module(pre)
    pre.STAGE = Path(STAGE)
    pre.MAP_NAME = MAP
    run = pre.Run(".", "stage", pins=False, quiet=True)
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
    print(f"preflight view{' with derived requests' if derived else ''}: {run.ok} ok, {len(fails)} FAIL ({len(fails & set(expected))} expected), {len(run.warn)} WARN {sorted(run.warn)}")
    print(f"unexpected FAIL: {surprise}")
    print(f"expected FAIL that did not happen: {missing}")
    return 1 if surprise or missing else 0


if __name__ == "__main__":
    sys.exit(main())
