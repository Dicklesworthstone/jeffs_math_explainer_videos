"""Oliver take01 length against the scratch cue length, for cues that have both.

Run from the repo root with python3 -I -B. Reads only. Sizes how far Oliver's clock can
sit from the scratch clock that chapter 17's estimates use; it says nothing about
chapter 17's own takes, because none exist.
"""
import pathlib
import statistics
import wave

ROOT = pathlib.Path("narration_lab/third-wave-20261024/root")
TAKES = ROOT / "elevenlabs-expanded-algebra-20261103"
PREVIEW = ROOT / "five-film-release-20261027/algebra-combined-repair-preview01/render"

scratch = {}
for cues in sorted(PREVIEW.glob("*.cues.tsv")):
    for line in cues.read_text().splitlines():
        cue, _start, length = line.split("\t")[:3]
        scratch[cue] = float(length)

ratios = {}
for cue, length in scratch.items():
    take = TAKES / f"{cue}-take01.wav"
    if not take.is_file():
        continue
    with wave.open(str(take)) as w:
        ratios[cue] = (w.getnframes() / w.getframerate()) / length

print("scratch cues:", len(scratch), " with an Oliver take01:", len(ratios))
print("cue files:", ", ".join(sorted(p.name for p in PREVIEW.glob("*.cues.tsv"))))
values = sorted(ratios.values())
print(f"Oliver/scratch length: min {values[0]:.3f}  p10 {values[len(values)//10]:.3f}  median {statistics.median(values):.3f}  p90 {values[len(values)*9//10]:.3f}  max {values[-1]:.3f}")
by = {}
for cue, r in ratios.items():
    by.setdefault(cue.split("_")[0], []).append(r)
for chapter, rs in sorted(by.items()):
    print(f"  chapter {chapter}: {len(rs)} cues, median {statistics.median(rs):.3f}")
for cue, r in sorted(ratios.items(), key=lambda kv: kv[1])[:5]:
    print(f"  shortest {cue}: {r:.3f}")
for cue, r in sorted(ratios.items(), key=lambda kv: kv[1])[-5:]:
    print(f"  longest  {cue}: {r:.3f}")
