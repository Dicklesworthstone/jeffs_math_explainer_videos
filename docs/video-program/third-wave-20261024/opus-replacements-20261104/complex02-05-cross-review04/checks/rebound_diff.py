import json, re
from pathlib import Path
O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
X = O / "complex02-05-cross-review04/rebound"
REL = Path("explainer_corpus/complex_analysis_visual")
CAND = Path("narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-picture-stage01/candidate") / REL
for label, base in (("canonical", REL), ("candidate", CAND)):
    old_map = json.loads((base / "docs/narration-direction-map-oliver10.json").read_text())
    new_map = json.loads((X / label / REL / "docs/narration-direction-map-oliver11.json").read_text())
    print(label, "map sections differing:", [k for k in old_map if old_map[k] != new_map.get(k)], "new keys", set(new_map) - set(old_map))
    a = {r["cue"]: r for r in old_map["visual_holds"]}
    b = {r["cue"]: r for r in new_map["visual_holds"]}
    print("  hold rows moved:", [c for c in a if a[c] != b[c]], "same cue set", set(a) == set(b))
    print("  fields changed:", sorted({k for c in a for k in a[c] if a[c][k] != b[c].get(k)}))
    for h0, h1 in zip(old_map["helpers"], new_map["helpers"]):
        if h0 != h1:
            print("  helper:", {k: (h0.get(k), h1.get(k)) for k in h0 if h0.get(k) != h1.get(k)})
    print("  overrides equal", old_map["overrides"] == new_map["overrides"], "scope:", old_map.get("scope") == new_map.get("scope"))
    old_units = json.loads((base / "narration_units.json").read_text())
    new_units = json.loads((X / label / REL / "narration_units.json").read_text())
    def walk(x, y, path=""):
        if isinstance(x, dict):
            for k in sorted(set(x) | set(y)):
                yield from walk(x.get(k), y.get(k), f"{path}.{k}")
        elif isinstance(x, list) and isinstance(y, list) and len(x) == len(y):
            for i, (p, q) in enumerate(zip(x, y)):
                yield from walk(p, q, f"{path}[{i}]")
        elif x != y:
            yield path, x, y
    print("  units diff:", [(p, str(a)[:44], str(b)[:44]) for p, a, b in walk(old_units, new_units)])
    audit = (X / label / "source-audit.json").read_text()
    print("  temp or home paths in outputs:", len(re.findall(r"cross04-|/private/|/var/folders|/Users/", audit + json.dumps(new_map) + json.dumps(new_units))))
    audit_json = json.loads(audit)
    print("  audit keys:", list(audit_json)[:14], "problems:", audit_json.get("problems"))
