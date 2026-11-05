"""What moved on the registry since the previous glyph job, and which rows hold a character worth tracing.

    python3 -I -B .../checks/registry_delta.py     (from the repo root)

Live numbers are recorded as sampled at 1 fps under scratch timing, so numeric rows come and go between jobs with no source change.
"""
from pathlib import Path

BASE = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
NEW = BASE / "logarithm-stage06-glyph01/glyphs/registry.tsv"
OLD = BASE / "logarithm-combined-repair-glyphs01/glyphs/registry.tsv"


def rows(path):
    out = {}
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        kind, chapters, literal = line.split("\t", 2)
        out[(kind, literal)] = (n, chapters)
    return out


new, old = rows(NEW), rows(OLD)
print(f"previous job {OLD.parent.parent.name}: {len(old)} rows; this job: {len(new)} rows")
print("\n# added")
for key in sorted(set(new) - set(old), key=lambda k: new[k][0]):
    print(f"  row {new[key][0]} {key[0]} {new[key][1]}: {key[1][:110]}")
print("\n# removed")
for key in sorted(set(old) - set(new), key=lambda k: old[k][0]):
    print(f"  old row {old[key][0]} {key[0]} {old[key][1]}: {key[1][:110]}")
print("\n# rows holding U+03B8")
for key, (n, chapters) in sorted(new.items(), key=lambda kv: kv[1][0]):
    if "θ" in key[1] or "\\theta" in key[1]:
        print(f"  row {n} {key[0]} {chapters}: {key[1][:110]}")
