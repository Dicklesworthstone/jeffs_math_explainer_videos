"""Rebind a direction map's file-bound rows (visual_holds and helpers) to a stage's bytes. Reads only.

    python3 -I -B rebind_delta.py <stage-root> <base-map.json> <delta-out.json|-> [--check-only]

For each row: if the file's sha256 and the marker on the row's line still hold, the row is kept.
Otherwise the new line is the marker's only occurrence in the film code (above the file's
`#[cfg(test)] mod tests`); a marker found nowhere or more than once there is named for a person
to choose (MARKERS below records the choices made). The delta lists each changed row's old and
new sha256, line and marker. It then applies the delta in memory and checks every row binds.
Nothing in the stage or the map is written.
"""
import copy
import hashlib
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(sys.argv[1])
BASE = pathlib.Path(sys.argv[2])
OUT = sys.argv[3]
# a marker that a repair rewrote: (file, old marker) -> new marker, from the repair's own handoff
MARKERS = {
    # complex02-05-repair03/HANDOFF.md, Bindings: 03_3's marker "changes to" this
    ("two_planes.rs", "let asked = predict(&kit, stage, &line, &ASK, out.p(C::new(0.0, 0.4)))?;"):
        "let asked = predict(&kit, stage, &line, &ASK, out.p(ASK_AT))?;",
}
# cue -> which occurrence, when one marker stands at two places in the film (the cue's own)
SAY = re.compile(r'let (?:mut )?line = say\(&kit, stage, "(\d\d_\d+)"\)')


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def film_lines(path):
    text = path.read_text(encoding="utf-8")
    film = re.split(r"#\[cfg\(test\)\]\s*mod tests", text)[0]
    return text.split("\n"), film.count("\n") + 1


def cue_at(lines, n):
    cue = None
    for i in range(n):
        m = SAY.search(lines[i])
        if m:
            cue = m.group(1)
    return cue


base = json.loads(BASE.read_text(encoding="utf-8"))
delta, problems = [], []
for section in ("visual_holds", "helpers"):
    for k, row in enumerate(base.get(section, [])):
        f = ROOT / row["path"]
        lines, film_end = film_lines(f)
        now = sha(f)
        marker = MARKERS.get((f.name, row["marker"]), row["marker"])
        if now == row["sha256"] and marker == row["marker"] and row["marker"] in lines[row["line"] - 1]:
            continue
        hits = [i + 1 for i, l in enumerate(lines[:film_end]) if marker in l]
        if section == "visual_holds" and len(hits) > 1:
            # the hold belongs to the cue it follows: the occurrence inside that cue's code
            hits = [h for h in hits if cue_at(lines, h) == row["cue"]]
        if len(hits) != 1:
            problems.append(f"{section}[{k}] {f.name} {marker!r}: film lines {hits}")
            continue
        delta.append({
            "section": section, "index": k, "cue": row.get("cue"), "next_cue": row.get("next_cue"),
            "path": row["path"], "old": {"sha256": row["sha256"], "line": row["line"], "marker": row["marker"]},
            "new": {"sha256": now, "line": hits[0], "marker": marker},
        })

script_now = sha(ROOT / "explainer_corpus/complex_analysis_visual/script.tsv")
doc = {
    "base_map": {"name": BASE.name, "sha256": sha(BASE)},
    "stage_script_sha256": script_now,
    "scripts_field_needs": {"complex_analysis_visual": script_now} if base["scripts"].get("complex_analysis_visual") != script_now else "unchanged",
    "rows": delta,
    "problems": problems,
}
# apply in memory and check
m = copy.deepcopy(base)
m["scripts"]["complex_analysis_visual"] = script_now
for d in delta:
    m[d["section"]][d["index"]].update(d["new"])
bad = []
for section in ("visual_holds", "helpers"):
    for k, row in enumerate(m.get(section, [])):
        f = ROOT / row["path"]
        lines = f.read_text(encoding="utf-8").split("\n")
        if sha(f) != row["sha256"] or row["marker"] not in lines[row["line"] - 1]:
            bad.append(f"{section}[{k}]")
print(f"base {BASE.name} {sha(BASE)[:16]}: {len(delta)} rows to rebind, {len(problems)} need a person")
for d in delta:
    print(f"  {d['section']}[{d['index']}] {d.get('cue')}->{d.get('next_cue')} {d['path'].rsplit('/', 1)[-1]}: {d['old']['sha256'][:8]}:{d['old']['line']} -> {d['new']['sha256'][:8]}:{d['new']['line']}" + ("  MARKER CHANGES" if d['old']['marker'] != d['new']['marker'] else ""))
for p in problems:
    print("  NEEDS A PERSON", p)
print("after the delta, rows not binding:", bad or "none")
if OUT != "-" and "--check-only" not in sys.argv:
    pathlib.Path(OUT).write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
sys.exit(1 if bad or problems else 0)
