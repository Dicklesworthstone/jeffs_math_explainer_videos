"""Sentence starts on the Oliver takes (ASR estimates recorded by root), and what the 13_12 beats ask of them. Read-only."""
import json
import pathlib

LAB = pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103")
a = json.loads((LAB / "arithmetic-sentence-audit02.json").read_text())
print("corrections:", json.dumps(a["corrections"], ensure_ascii=False)[:900])
print("limits:", json.dumps(a["limits"], ensure_ascii=False)[:500])
starts = {}
for r in a["records"]:
    if r.get("model") != "large":
        continue
    groups = r["punctuation_groups"]
    print(f"\n{r['cue']} {pathlib.Path(r['path']).name}")
    for g in groups:
        print(f"   {g['start']:6.2f}  {g['text'].strip()[:90]}")
    starts[r["cue"]] = [g["start"] for g in groups]

# Time the 13_12 source asks for, per beat, from the literal run times in arithmetic.rs (seconds).
LIGHT = 0.25 + 0.25 + 0.3
beats = {
    "closure": LIGHT + 0.35 + 0.35 + 0.35,
    "associativity": 0.35 + 0.2 + LIGHT + 3 * (0.35 + 0.35) + 0.35 + 0.3,
    "identity": 0.35 + 0.2 + LIGHT + 0.35 + 0.35,
    "inverse": 0.35 + 0.2 + LIGHT + 3 * (0.35 + 0.3) + 0.35 + 0.35,
}
print("\nseconds of animation each 13_12 beat asks for before the next may start:")
for k, s in beats.items():
    print(f"   {k:14} {s:.2f}")
