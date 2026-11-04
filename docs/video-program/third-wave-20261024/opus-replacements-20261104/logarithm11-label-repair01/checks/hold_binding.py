"""What the source change does to the narration map and units file, computed against canonical map12. Reads only.

python3 -I -B hold_binding.py REPO_ROOT PATCHED_FILE

Run from anywhere. The rebound hashes below are for canonical map12 only; the root's stage06 binder writes its own map13.
The interval hash is recomputed the way the root's binder does it
(bind_final80-stage05.py: lines[say_line - 1 : hold_line] joined by newlines, sha256 of the UTF-8 bytes, lines from
data.decode("utf-8").split("\n")). Reproducing the stored value on the baseline is the control for that reading.
tools/narration_director.py:124 raises "visual hold evidence changed" when the evidence digest differs.
"""
import hashlib
import json
import re
import sys
from pathlib import Path

CRATE = "explainer_corpus/logarithm_mysteries"
CHAPTER = CRATE + "/src/chapters/unreachable.rs"
MAP = CRATE + "/docs/narration-direction-map-oliver12.json"
UNITS = CRATE + "/narration_units.json"
# Line of each cue's say in the rendered source, from the root's hold-audit.json (cues list).
SAY = {"11_3": 963, "11_20": 1501}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    repo, patched = Path(sys.argv[1]), Path(sys.argv[2])
    base_bytes = (repo / CHAPTER).read_bytes()
    new_bytes = patched.read_bytes()
    old_hash, new_hash = sha(base_bytes), sha(new_bytes)
    map_bytes = (repo / MAP).read_bytes()
    units_bytes = (repo / UNITS).read_bytes()
    print("source baseline", old_hash)
    print("source patched ", new_hash)

    the_map = json.loads(map_bytes)
    occurrences = map_bytes.decode("utf-8").count(old_hash)
    holds = [h for h in the_map["visual_holds"] if h["path"].endswith("/unreachable.rs")]
    print(f"map occurrences of the source hash: {occurrences}; holds on this file: {len(holds)}")
    if occurrences != 4 or len(holds) != 2:
        raise SystemExit("expected 4 occurrences in 2 holds")

    base_lines = base_bytes.decode("utf-8").split("\n")
    new_lines = new_bytes.decode("utf-8").split("\n")
    print(f"lines: baseline {len(base_lines)}, patched {len(new_lines)}")
    changed = [i + 1 for i, (a, b) in enumerate(zip(base_lines, new_lines)) if a != b]
    print("lines that differ:", changed)
    for hold in holds:
        cue, line = hold["cue"], hold["line"]
        start = SAY[cue]
        stored = hold["interval_sha256"]
        recomputed = sha("\n".join(base_lines[start - 1:line]).encode("utf-8"))
        after = sha("\n".join(new_lines[start - 1:line]).encode("utf-8"))
        marker_base = hold["marker"] in base_lines[line - 1]
        marker_new = hold["marker"] in new_lines[line - 1]
        print(f"hold {cue}->{hold['next_cue']} ({hold['kind']}) lines {start}..{line}: stored {stored[:16]}; "
              f"baseline recomputed {recomputed[:16]} ({'control holds' if recomputed == stored else 'CONTROL FAILS'}); "
              f"patched {after[:16]} ({'unchanged' if after == stored else 'CHANGED'}); "
              f"marker on its line, baseline {marker_base}, patched {marker_new}")
        if recomputed != stored or after != stored or not (marker_base and marker_new):
            raise SystemExit("hold check failed")

    # The one condition in tools/narration_director.py:124, on both maps.
    old_map_passes = all(h["evidence"]["sha256"] == new_hash for h in holds)
    print("old map against the patched source, evidence digests match:", old_map_passes,
          "(expected False: the director refuses until rebound)")

    new_map_bytes = map_bytes.decode("utf-8").replace(old_hash, new_hash).encode("utf-8")
    new_map = json.loads(new_map_bytes)
    new_holds = [h for h in new_map["visual_holds"] if h["path"].endswith("/unreachable.rs")]
    print("rebound map against the patched source, evidence digests match:",
          all(h["evidence"]["sha256"] == new_hash and h["sha256"] == new_hash for h in new_holds))

    old_map_hash, new_map_hash = sha(map_bytes), sha(new_map_bytes)
    units_text = units_bytes.decode("utf-8")
    print("units file cites the map hash:", units_text.count(old_map_hash), "time(s)")
    if units_text.count(old_map_hash) != 1:
        raise SystemExit("expected the map hash once in the units file")
    new_units_bytes = units_text.replace(old_map_hash, new_map_hash).encode("utf-8")
    units_old, units_new = json.loads(units_bytes), json.loads(new_units_bytes)
    print("units text unchanged:", [u["text"] for u in units_old["units"]] == [u["text"] for u in units_new["units"]],
          "; script_sha256 unchanged:", units_old["direction_contract"]["script_sha256"] == units_new["direction_contract"]["script_sha256"])
    print("map baseline  ", old_map_hash)
    print("map rebound   ", new_map_hash)
    print("units baseline", sha(units_bytes))
    print("units rebound ", sha(new_units_bytes))

    timing = re.compile(r"until_|\.say|say_|hold|wait|predict|music_breath|phrase_|play|fade|swap")
    print("changed lines holding a timing or animation call:",
          [n for n in changed if timing.search(new_lines[n - 1])] or "none")
    # Wait sites and markers named by the root's stage06 binder (bind_final80-stage06.py), read, not run.
    named = {288: "word_sync_helper", 1555: "reading_voice_end", 1590: "music_breath", 1070: "predict"}
    print("stage06 binder sites on a changed line:", [n for n in named if n in changed] or "none")
    print("stage06 binder sites unchanged:", all(base_lines[n - 1] == new_lines[n - 1] for n in named))
    stage06 = repo / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06" / CHAPTER
    print("stage06 chapter byte-identical to canonical at this run:", stage06.read_bytes() == base_bytes)
    print("script.tsv sha256:", sha((repo / CRATE / "script.tsv").read_bytes()),
          "; units direction_contract.script_sha256 agrees:", units_old["direction_contract"]["script_sha256"] == sha((repo / CRATE / "script.tsv").read_bytes()))


if __name__ == "__main__":
    main()
