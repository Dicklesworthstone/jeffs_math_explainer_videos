"""Tie every selected frame of 13_arithmetic to its time, cue, hash and sheet cell. Read-only on the frames. Run from the repo root.

Prints provenance of the source against what the render recorded, duplicates, whether each sheet cell shows its frame,
and a table per sheet. Writes the table to the scratch folder given as argv[1].
"""
import hashlib
import json
import pathlib
import sys

from PIL import Image, ImageChops, ImageStat

OUT = pathlib.Path(sys.argv[1])
B = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
D = B / "algebra-combined-repair-frames01/13_arithmetic"
R = B / "algebra-combined-repair-preview01/render"
times = json.loads((D / "sampling.json").read_text())["times"]
cues = [(c, float(s), float(l)) for c, s, l in (x.split("\t") for x in (R / "13_arithmetic.cues.tsv").read_text().splitlines())]


def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


inputs = json.loads(pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/combined-preview01/algebra-combined-repair-frames01-inputs.json").read_text())
print("input record keys:", list(inputs["files"][8].keys()))
CRATE = "explainer_corpus/abstract_algebra_expanded/"
for rel in ("src/arithmetic.rs", "src/d4_group.rs", "src/tile_actor.rs", "src/tile_model.rs", "src/axioms.rs", "src/main.rs", "script.tsv", "oracle.py"):
    rec = [f for f in inputs["files"] if f["path"].endswith(CRATE + rel)]
    now = sha(CRATE + rel)
    was = rec[0].get("sha256") if rec else None
    print(f"{rel}: now {now[:16]}; at the frames build {str(was)[:16]}; {'SAME' if was == now else 'DIFFERENT or unrecorded'}")


def cue_of(t):
    now = None
    for c, s, l in cues:
        if s <= t < s + l:
            return c
        if t >= s:
            now = c
    return f"after {now}" if now else "before 13_1"


rows = []
seen = {}
for i, t in enumerate(times):
    p = D / f"frame-{i:04}.png"
    h = sha(p)
    first = seen.setdefault(h, i)
    rows.append(dict(frame=i, t=t, cue=cue_of(t), sha=h, sheet=i // 4, cell=i % 4, same_as=first if first != i else None))

# each sheet cell against its frame, resized the way the sampler did
worst = (0, None)
bad = []
for sheet in range(0, (len(times) + 3) // 4):
    with Image.open(D / f"sheet-{sheet:03}.png") as s:
        s = s.convert("RGB")
        for j in range(4):
            i = sheet * 4 + j
            if i >= len(times):
                break
            x, y = (j % 2) * 960, (j // 2) * 570
            cell = s.crop((x, y + 30, x + 960, y + 570))
            with Image.open(D / f"frame-{i:04}.png") as f:
                mine = f.convert("RGB").resize((960, 540))
            d = max(ImageStat.Stat(ImageChops.difference(cell, mine)).extrema, key=lambda e: e[1])[1]
            if d > worst[0]:
                worst = (d, i)
            if d > 0:
                bad.append(i)
print("sheet cells differing from their frame:", len(bad), "worst channel difference", worst)
print("frames", len(rows), "unique", len(seen), "sheets", (len(times) + 3) // 4, "duration", json.loads((D / "sampling.json").read_text())["duration"])

(OUT / "frames.tsv").write_text(
    "frame\tt\tcue\tsheet\tcell\tsame_as\tsha256\n"
    + "".join(f"{r['frame']}\t{r['t']:.3f}\t{r['cue']}\t{r['sheet']:03}\t{r['cell']}\t{'' if r['same_as'] is None else r['same_as']}\t{r['sha']}\n" for r in rows)
)
sh = [(s, sha(D / f"sheet-{s:03}.png")) for s in range((len(times) + 3) // 4)]
(OUT / "sheets.tsv").write_text("sheet\tframes\tt_from\tt_to\tcues\tsha256\n" + "".join(
    f"{s:03}\t{s * 4}-{min(s * 4 + 3, len(times) - 1)}\t{times[s * 4]:.3f}\t{times[min(s * 4 + 3, len(times) - 1)]:.3f}\t{','.join(sorted(set(r['cue'] for r in rows[s * 4:s * 4 + 4]), key=lambda c: c))}\t{h}\n" for s, h in sh))

by_cue = {}
for r in rows:
    by_cue.setdefault(r["cue"], []).append(r)
for c, rs in by_cue.items():
    print(f"{c:18} frames {rs[0]['frame']:3}-{rs[-1]['frame']:3}  t {rs[0]['t']:7.3f}-{rs[-1]['t']:7.3f}  sheets {rs[0]['sheet']:03}-{rs[-1]['sheet']:03}  unique {len(set(x['sha'] for x in rs))}")
