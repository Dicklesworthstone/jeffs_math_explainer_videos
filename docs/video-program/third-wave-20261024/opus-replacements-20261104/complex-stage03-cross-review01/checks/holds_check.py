"""Do a direction map's hold rows and a narration plan still bind the stage's bytes? Reads only.

    python3 -I -B holds_check.py <stage-root> <map.json> [<map.json> ...]

<stage-root> holds explainer_corpus/. For each visual_holds row: is the file's sha256 the row's,
and is the marker on the row's line? If not, where is the marker now (every line holding it)?
Then the crate's narration_units.json: its script hash against script.tsv, its map path and hash,
and every unit's text against the script.
"""
import hashlib
import json
import pathlib
import sys

ROOT = pathlib.Path(sys.argv[1])
CRATE = ROOT / "explainer_corpus/complex_analysis_visual"


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


for name in sys.argv[2:]:
    m = json.loads(pathlib.Path(name).read_text(encoding="utf-8"))
    print(f"== {pathlib.Path(name).name} sha256 {sha(pathlib.Path(name))[:16]}")
    print("   script pinned:", {k: v[:16] for k, v in m.get("scripts", {}).items()}, "script now:", sha(CRATE / "script.tsv")[:16])
    rows = m.get("visual_holds", [])
    stale = 0
    for r in rows:
        f = ROOT / r["path"]
        text = f.read_text(encoding="utf-8").split("\n")
        now = sha(f)
        on_line = 0 < r["line"] <= len(text) and r["marker"] in text[r["line"] - 1]
        where = [i + 1 for i, row in enumerate(text) if r["marker"] in row]
        ok = now == r["sha256"] and on_line
        if not ok:
            stale += 1
            print(f"   STALE {r['cue']}->{r.get('next_cue')} {r['path'].rsplit('/', 1)[-1]}:{r['line']} pinned {r['sha256'][:8]} now {now[:8]}; marker on pinned line {on_line}; marker lines now {where}")
    print(f"   {len(rows)} hold rows, {stale} stale")

u_path = CRATE / "narration_units.json"
u = json.loads(u_path.read_text(encoding="utf-8"))
script = dict(l.split("\t", 1) for l in (CRATE / "script.tsv").read_text(encoding="utf-8").splitlines() if "\t" in l and not l.startswith("#"))
print(f"== narration_units.json sha256 {sha(u_path)[:16]}")
print("   script_sha256", u["script_sha256"][:16], "contract script", u["direction_contract"]["script_sha256"][:16], "script now", sha(CRATE / "script.tsv")[:16])
mp = ROOT / u["direction_contract"]["map"]["path"]
print("   map", u["direction_contract"]["map"]["path"].rsplit("/", 1)[-1], "pinned", u["direction_contract"]["map"]["sha256"][:16], "now", sha(mp)[:16] if mp.is_file() else "MISSING")
diff = [x["id"] for x in u["units"] if " ".join(script.get(l, "?") for l in x["line_ids"]) != x["text"]]
print("   units", len(u["units"]), "text differs from script:", diff)
