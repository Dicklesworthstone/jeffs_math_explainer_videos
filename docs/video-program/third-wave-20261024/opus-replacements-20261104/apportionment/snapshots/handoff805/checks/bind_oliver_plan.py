"""Write the candidate's ElevenLabs Oliver source plan for root.

    python3 -I -B bind_oliver_plan.py <candidate topic dir> [--check]

Writes, inside the candidate topic dir (a mirror of explainer_corpus/apportionment):

  docs/narration-direction-map-oliver03.json  one prediction override and one
      visual hold per think pause, each bound to the chapter line that calls
      draw::think (sha256 of that chapter file)
  narration_units.json  one unit per cue (oliver_<cue>), text verbatim from
      script.tsv, bound to the script and to the map above

The existing oliver01/oliver02 maps and the canonical narration_units.json
are not touched; the candidate map is a new version. This binds authored
coverage only: no take exists, nothing was synthesized or listened to.
With --check nothing is written; the files on disk must equal what would be.
"""

import hashlib
import json
import re
import sys
from pathlib import Path

TOPIC = "apportionment"
MAP_NAME = "narration-direction-map-oliver03.json"
THINK = re.compile(r"\bthink(?:_here)?\(&kit, stage")
SAY = re.compile(r'say\(&kit, stage, "(\d\d_\d+)"\)')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_script(path):
    rows = []
    for line in path.read_text().splitlines():
        if line.startswith("#") or not line.strip():
            continue
        cue, text = line.split("\t", 1)
        rows.append((cue, text))
    return rows


def holds(cand, cues):
    """(cue, file, line number, marker) for every think call, by chapter."""
    out = []
    for path in sorted((cand / "src" / "chapters").glob("*.rs")):
        current = None
        for n, line in enumerate(path.read_text().splitlines(), 1):
            m = SAY.search(line)
            if m:
                current = m.group(1)
            if THINK.search(line) and "fn think" not in line:
                if current is None:
                    raise SystemExit(f"{path.name}:{n}: a think pause before any line")
                out.append((current, path, n, line.strip()))
    out.sort(key=lambda h: cues.index(h[0]))
    return out


def build(cand):
    script_path = cand / "script.tsv"
    script = read_script(script_path)
    cues = [c for c, _ in script]
    texts = dict(script)
    script_sha = sha(script_path.read_bytes())
    found = holds(cand, cues)
    if len({h[0] for h in found}) != len(found):
        raise SystemExit("two think pauses after one cue")
    overrides, visual = [], []
    for cue, path, n, marker in found:
        i = cues.index(cue)
        if i + 1 >= len(cues):
            raise SystemExit(f"{cue}: a prediction pause after the last cue")
        if not texts[cue].endswith("?"):
            raise SystemExit(f"{cue}: a prediction pause after a cue that is not a question")
        rel = f"explainer_corpus/{TOPIC}/src/chapters/{path.name}"
        overrides.append({
            "video": TOPIC,
            "cue": cue,
            "after": texts[cue],
            "role": "visual",
            "reason": "Prediction pause after this complete cue before the next cue.",
        })
        visual.append({
            "video": TOPIC,
            "cue": cue,
            "next_cue": cues[i + 1],
            "kind": "prediction",
            "path": rel,
            "sha256": sha(path.read_bytes()),
            "line": n,
            "marker": marker,
            "basis": "draw::think holds the cue to completion, marks predict, then a four-second "
                     "authored visual wait (THINK_S) before the next cue; no listening claim.",
        })
    direction_map = {
        "schema_version": 1,
        "scripts": {TOPIC: script_sha},
        "overrides": overrides,
        "visual_holds": visual,
        "scope": "Candidate rebuild (bead .30): authored prediction pauses only. No take, "
                 "listening, timing or independent acceptance.",
    }
    map_text = json.dumps(direction_map, indent=2, ensure_ascii=False) + "\n"
    units = {
        "schema_version": 1,
        "script_sha256": script_sha,
        "direction_contract": {
            "schema_version": 1,
            "topic": TOPIC,
            "script_sha256": script_sha,
            "map": {
                "path": f"explainer_corpus/{TOPIC}/docs/{MAP_NAME}",
                "sha256": sha(map_text.encode()),
            },
        },
        "scope": "Candidate full-cue source plan for ElevenLabs Oliver: every cue is new text, "
                 "one take per cue, no take selection, listening, timing or independent acceptance.",
        "units": [{"id": f"oliver_{c}", "line_ids": [c], "text": t} for c, t in script],
    }
    for u in units["units"]:
        if not 1 <= len(u["text"].split()) <= 100:
            raise SystemExit(f"{u['id']}: outside the coherent-unit word bound")
    units_text = json.dumps(units, indent=2, ensure_ascii=False) + "\n"
    return {cand / "docs" / MAP_NAME: map_text, cand / "narration_units.json": units_text}, found


def main(argv):
    cand = Path(argv[0])
    check = "--check" in argv[1:]
    files, found = build(cand)
    stale = []
    for path, text in files.items():
        if check:
            if not path.is_file() or path.read_text() != text:
                stale.append(path.name)
        else:
            path.write_text(text)
    for cue, path, n, _ in found:
        print(f"hold after {cue}: {path.name}:{n}")
    print(f"{len(found)} prediction holds; {'stale: ' + ', '.join(stale) if stale else 'files current' if check else 'written'}")
    return 1 if stale else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
